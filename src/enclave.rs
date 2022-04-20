
use crate::def::DEF;
use crate::error_code::ERROR;

type enclave_id = u32;

const ENCLAVE_REGIONS_MAX: usize = 8;

/* Metadata around memory regions associate with this enclave
 * EPM is the 'home' for the enclave, contains runtime code/etc
 * UTM is the untrusted shared pages
 * OTHER is managed by some other component (e.g. platform_)
 * INVALID is an unused index
 */
enum enclave_region_type {
  REGION_INVALID,
  REGION_EPM,
  REGION_UTM,
  REGION_OTHER
}

enum enclave_state {
  INVALID,
  DESTROYING,
  ALLOCATED,
  FRESH,
  STOPPED,
  RUNNING,
}

struct enclave_region {
  pmp_rid: region_id,
  enclave_type: enclave_region_type
}

struct enclave {
  // let lock: spinlock_t, // local enclave lock. we don't need this until we have multithreaded enclave
  eid: enclave_id, //enclave id
  encl_satp: u32, // enclave's page table base
  state: enclave_state, // global state of the enclave

  /* Physical memory regions associate with this enclave */
  regions: [enclave_region; ENCLAVE_REGIONS_MAX],

  /* measurement */
  hash: [u8; MDSIZE],
  sign: [u8; SIGNATURE_SIZE],

  /* parameters */
  params: runtime_va_params_t,
  pa_params: runtime_pa_params,

  /* enclave execution context */
  n_thread: u32,
  threads: [thread_state; MAX_ENCL_THREADS], // thread.rs

  ped: platform_enclave_data // platform.rs
}

fn copy_enclave_create_args(src: u32, dest: *mut keystone_sbi_create) -> u32 {

  let region_overlap: i32 = copy_to_sm(dest, src, mem::size_of::<keystone_sbi_create>()); // mprv.rs

  if region_overlap != 0 {
    return ERROR::SBI_ERR_SM_ENCLAVE_REGION_OVERLAPS; // error_code.rs
  }
  else {
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }

}

fn create_enclave(eidptr: *mut u32, create_args: keystone_sbi_create) -> u32 {
  /* EPM and UTM parameters */
  let base: uintptr_t = create_args.epm_region.paddr;
  let size: size_t = create_args.epm_region.size;
  let utbase: uintptr_t = create_args.utm_region.paddr;
  let utsize: size_t = create_args.utm_region.size;

  let eid: enclave_id = 0;
  let ret: u32 = 0;
  let region: i32 = 0;
  let shared_region: i32 = 0;

  /* Runtime parameters */
  if !is_create_args_valid(&create_args) { // enclave.rs
    return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT;
  } 

  /* set va params */
  let params: runtime_va_params_t = create_args.params;
  let pa_params: runtime_pa_params;
  pa_params.dram_base = base;
  pa_params.dram_size = size;
  pa_params.runtime_base = create_args.runtime_paddr;
  pa_params.user_base = create_args.user_paddr;
  pa_params.free_base = create_args.free_paddr;


  // allocate eid
  ret = ERROR::SBI_ERR_SM_ENCLAVE_NO_FREE_RESOURCE;
  if encl_alloc_eid(&eid) != ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS {
    return ret;
  }

  // create a PMP region bound to the enclave
  ret = ERROR::SBI_ERR_SM_ENCLAVE_PMP_FAILURE;
  if pmp_region_init_atomic(base, size, PMP_PRI_ANY, &region, 0) { // pmp.rs
    encl_free_eid(eid);
    return ret;
  }

  // create PMP region for shared memory
  if pmp_region_init_atomic(utbase, utsize, PMP_PRI_BOTTOM, &shared_region, 0) { // pmp.rs
    pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  // set pmp registers for private region (not shared)
  if pmp_set_global(region, PMP_NO_PERM) { // pmp.rs
    pmp_region_free_atomic(shared_region);
    pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  // cleanup some memory regions for sanity See issue #38
  clean_enclave_memory(utbase, utsize); // enclave.rs


  // initialize enclave metadata
  enclaves[eid].eid = eid;
  enclaves[eid].regions[0].pmp_rid = region;
  enclaves[eid].regions[0].type = REGION_EPM;
  enclaves[eid].regions[1].pmp_rid = shared_region;
  enclaves[eid].regions[1].type = REGION_UTM;

  if cfg!(target_pointer_width = "32") {
    enclaves[eid].encl_satp = ((base >> RISCV_PGSHIFT) | (SATP_MODE_SV32 << HGATP_MODE_SHIFT));
  }
  else {
    enclaves[eid].encl_satp = ((base >> RISCV_PGSHIFT) | (SATP_MODE_SV39 << HGATP_MODE_SHIFT));
  }

  enclaves[eid].n_thread = 0;
  enclaves[eid].params = params;
  enclaves[eid].pa_params = pa_params;

  /* Init enclave state (regs etc) */
  clean_state(&enclaves[eid].threads[0]); // thread.rs

  /* Platform create happens as the last thing before hashing/etc since
     it may modify the enclave struct */
  ret = platform_create_enclave(&enclaves[eid]);
  if ret {
    pmp_unset_global(region);
    pmp_region_free_atomic(shared_region);
    pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  /* Validate memory, prepare hash and signature for attestation */
  spin_lock(&encl_lock); // FIXME This should error for second enter.
  ret = validate_and_hash_enclave(&enclaves[eid]);
  /* The enclave is fresh if it has been validated and hashed but not run yet. */
  if ret {
    spin_unlock(&encl_lock);
    platform_destroy_enclave(&enclaves[eid]);
    pmp_unset_global(region);
    pmp_region_free_atomic(shared_region);
    pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  enclaves[eid].state = FRESH;
  /* EIDs are unsigned int in size, copy via simple copy */
  *eidptr = eid;

  spin_unlock(&encl_lock);
  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;

}

fn is_create_args_valid(args: *mut keystone_sbi_create) -> i32 {
  let epm_start: uintptr_t;
  let epm_end: uintptr_t;

  /* printm("[create args info]: \r\n\tepm_addr: %llx\r\n\tepmsize: %llx\r\n\tutm_addr: %llx\r\n\tutmsize: %llx\r\n\truntime_addr: %llx\r\n\tuser_addr: %llx\r\n\tfree_addr: %llx\r\n", */
  /*        (*args).epm_region.paddr, */
  /*        (*args).epm_region.size, */
  /*        (*args).utm_region.paddr, */
  /*        (*args).utm_region.size, */
  /*        (*args).runtime_paddr, */
  /*        (*args).user_paddr, */
  /*        (*args).free_paddr); */

  // check if physical addresses are valid
  if (*args).epm_region.size <= 0 {
    return 0;
  }

  // check if overflow
  if (*args).epm_region.paddr >= (*args).epm_region.paddr + (*args).epm_region.size {
    return 0;
  }
  if (*args).utm_region.paddr >= (*args).utm_region.paddr + (*args).utm_region.size {
    return 0;
  }

  epm_start = (*args).epm_region.paddr;
  epm_end = (*args).epm_region.paddr + (*args).epm_region.size;

  // check if physical addresses are in the range
  if (*args).runtime_paddr < epm_start || (*args).runtime_paddr >= epm_end {
    return 0;
  }
  if (*args).user_paddr < epm_start || (*args).user_paddr >= epm_end {
    return 0;
  }
  if (*args).free_paddr < epm_start || (*args).free_paddr > epm_end {
    // note: free_paddr == epm_end if there's no free memory
    return 0;
  }
      

  // check the order of physical addresses
  if (*args).runtime_paddr > (*args).user_paddr {
    return 0;
  }
    
  if (*args).user_paddr > (*args).free_paddr {
    return 0;
  }

  return 1;
}


/*
 * Fully destroys an enclave
 * Deallocates EID, clears epm, etc
 * Fails only if the enclave isn't running.
 */
fn destroy_enclave(eid: enclave_id) -> u32 {
  let destroyable: i32;

  spin_lock(&encl_lock);
  destroyable = (ENCLAVE_EXISTS(eid) && enclaves[eid].state <= STOPPED);
  /* update the enclave state first so that
   * no SM can run the enclave any longer */
  if destroyable {
    enclaves[eid].state = DESTROYING;
  }
  spin_unlock(&encl_lock);

  if !destroyable {
    return ERROR::SBI_ERR_SM_ENCLAVE_NOT_DESTROYABLE;
  }

  // 0. Let the platform specifics do cleanup/modifications
  platform_destroy_enclave(&enclaves[eid]);


  // 1. clear all the data in the enclave pages
  // requires no lock (single runner)
  let i: i32;
  void* base;
  size_t size;
  region_id rid;
  for(i = 0; i < ENCLAVE_REGIONS_MAX; i++){
    if(enclaves[eid].regions[i].type == REGION_INVALID ||
       enclaves[eid].regions[i].type == REGION_UTM)
      continue;
    //1.a Clear all pages
    rid = enclaves[eid].regions[i].pmp_rid;
    base = (void*) pmp_region_get_addr(rid);
    size = (size_t) pmp_region_get_size(rid);
    sbi_memset((void*) base, 0, size);

    //1.b free pmp region
    pmp_unset_global(rid);
    pmp_region_free_atomic(rid);
  }

  // 2. free pmp region for UTM
  rid = get_enclave_region_index(eid, REGION_UTM);
  if(rid != -1)
    pmp_region_free_atomic(enclaves[eid].regions[rid].pmp_rid);

  enclaves[eid].encl_satp = 0;
  enclaves[eid].n_thread = 0;
  enclaves[eid].params = (struct runtime_va_params_t) {0};
  enclaves[eid].pa_params = (struct runtime_pa_params) {0};
  for(i=0; i < ENCLAVE_REGIONS_MAX; i++){
    enclaves[eid].regions[i].type = REGION_INVALID;
  }

  // 3. release eid
  encl_free_eid(eid);

  return SBI_ERR_SM_ENCLAVE_SUCCESS;
}

fn encl_alloc_eid(_eid: *mut enclave_id) -> u32 {
  let eid: enclave_id;

  spin_lock(&encl_lock); // ?

  for i in 0..ENCL_MAX {
    eid = i;
    if enclaves[eid].state == INVALID {
      break;
    }
  }

  if eid != ENCL_MAX {
    enclaves[eid].state = ALLOCATED;
  }

  spin_unlock(&encl_lock); // ?

  if eid != ENCL_MAX {
    *_eid = eid;
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }
  else {
    return ERROR::SBI_ERR_SM_ENCLAVE_NO_FREE_RESOURCE;
  }
}

fn clean_enclave_memory(utbase: uintptr_t, utsize: uintptr_t) -> u32 {

  // This function is quite temporary. See issue #38

  // Zero out the untrusted memory region, since it may be in
  // indeterminate state.
  sbi_memset((void*)utbase, 0, utsize);

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}