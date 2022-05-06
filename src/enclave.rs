use crate::error_code::ERROR;
use crate::mprv::copy_to_sm;
use crate::sm::keystone_sbi_create;
use crate::pmp;
use crate::sm::runtime_va_params_t;
use crate::sm::runtime_pa_params;
use crate::pmp::region_id;
use crate::thread::thread_state;
use crate::platform::platform_enclave_data;
use crate::pmp::pmp_priority;
use crate::thread;
use crate::attest;
use crate::crypto;
use crate::mprv;
use crate::sm;
use crate::page;
use crate::cpu;
use crate::opensbi;
use crate::sbi_trap;

use std::mem;

pub type enclave_id = usize;

const ENCLAVE_REGIONS_MAX: usize = 8;
const MAX_ENCL_THREADS: usize = 1;
const ENCL_MAX: usize = 16;

const STOP_TIMER_INTERRUPT: usize = 0;
const STOP_EDGE_CALL_HOST: usize = 1;
const STOP_EXIT_ENCLAVE: usize = 2;

const ATTEST_DATA_MAXLEN: usize = 1024;
const SEALING_KEY_SIZE: usize = 128;

static encl_lock: spinlock_t = SPIN_LOCK_INITIALIZER;

static mut enclaves: [enclave;ENCL_MAX] = [enclave::new();ENCL_MAX];

/* Metadata around memory regions associate with this enclave
 * EPM is the 'home' for the enclave, contains runtime code/etc
 * UTM is the untrusted shared pages
 * OTHER is managed by some other component (e.g. platform_)
 * INVALID is an unused index
 */
#[derive(PartialEq)]
pub enum enclave_region_type {
  REGION_INVALID,
  REGION_EPM,
  REGION_UTM,
  REGION_OTHER
}

#[derive(PartialEq)]
enum enclave_state {
  INVALID,
  DESTROYING,
  ALLOCATED,
  FRESH,
  STOPPED,
  RUNNING,
}

struct enclave_region {
  pub pmp_rid: region_id,
  pub region_type: enclave_region_type
}

impl enclave_region {
  pub fn new() -> Self {
    Self {
      pmp_rid: 0,
      region_type: enclave_region_type::REGION_INVALID
    }
  }
}

pub struct enclave {
  // let lock: spinlock_t, // local enclave lock. we don't need this until we have multithreaded enclave
  pub eid: enclave_id, //enclave id
  pub encl_satp: usize, // enclave's page table base
  pub state: enclave_state, // global state of the enclave

  /* Physical memory regions associate with this enclave */
  pub regions: [enclave_region; ENCLAVE_REGIONS_MAX],

  /* measurement */
  pub hash: [u8; crypto::MDSIZE],
  pub sign: [u8; crypto::SIGNATURE_SIZE],

  /* parameters */
  pub params: runtime_va_params_t,
  pub pa_params: runtime_pa_params,

  /* enclave execution context */
  pub n_thread: usize,
  pub threads: [thread_state; MAX_ENCL_THREADS], // thread.rs

  pub ped: platform_enclave_data // platform.rs
}

impl enclave {
  pub fn new() -> Self {
    Self {
      eid: 0,
      encl_satp: 0,
      state: enclave_state::FRESH,
      regions: [enclave_region::new(); ENCLAVE_REGIONS_MAX],
      hash: [0; crypto::MDSIZE],
      sign: [0; crypto::SIGNATURE_SIZE],
      params: runtime_va_params_t::new(),
      pa_params: runtime_pa_params::new(),
      n_thread: 0,
      threads: [thread_state::new(); MAX_ENCL_THREADS],
      ped: platform_enclave_data::new(),
    }
  }
}

struct enclave_report {
  hash: [u8; crypto::MDSIZE],
  data_len: u64,
  data: [u8; ATTEST_DATA_MAXLEN],
  signature: [u8; crypto::SIGNATURE_SIZE]
}

struct sm_report {
  hash: [u8; crypto::MDSIZE],
  public_key: [u8; crypto::SIGNATURE_SIZE],
  signature: [u8; crypto::SIGNATURE_SIZE]
}

pub struct report {
  enclave: enclave_report,
  sm: sm_report,
  dev_public_key: [u8; crypto::PUBLIC_KEY_SIZE]
}

struct sealing_key {
  key: [u8; SEALING_KEY_SIZE],
  signature: [u8; crypto::SIGNATURE_SIZE]
}

pub fn copy_enclave_create_args(src: usize, dest: &mut keystone_sbi_create) -> usize {
  unsafe {
    let dst: usize = dest as *const keystone_sbi_create as usize;
    let region_overlap: i32 = copy_to_sm(dst, src, mem::size_of::<keystone_sbi_create>()); // mprv.rs

    if region_overlap != 0 {
      return ERROR::SBI_ERR_SM_ENCLAVE_REGION_OVERLAPS; // error_code.rs
    }
    else {
      return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
    }
  }
}

pub fn create_enclave(eidptr: *mut usize, create_args: keystone_sbi_create) -> usize {
  /* EPM and UTM parameters */
  let base: usize = create_args.epm_region.paddr as usize;
  let size: usize = create_args.epm_region.size as usize;
  let utbase: usize = create_args.utm_region.paddr as usize;
  let utsize: usize = create_args.utm_region.size as usize;

  let eid: enclave_id = 0;
  let ret: usize = 0;
  let region: i32 = 0;
  let shared_region: i32 = 0;

  /* Runtime parameters */
  if !is_create_args_valid(&mut create_args) != 0 { // enclave.rs
    return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT;
  } 

  /* set va params */
  let params: runtime_va_params_t = create_args.params;
  let pa_params: runtime_pa_params;
  pa_params.dram_base = base;
  pa_params.dram_size = size;
  pa_params.runtime_base = create_args.runtime_paddr as usize;
  pa_params.user_base = create_args.user_paddr as usize;
  pa_params.free_base = create_args.free_paddr as usize;


  // allocate eid
  ret = ERROR::SBI_ERR_SM_ENCLAVE_NO_FREE_RESOURCE;
  if encl_alloc_eid(&mut eid) != ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS {
    return ret;
  }

  // create a PMP region bound to the enclave
  ret = ERROR::SBI_ERR_SM_ENCLAVE_PMP_FAILURE;
  if pmp::pmp_region_init_atomic(base, size, pmp_priority::PMP_PRI_ANY, &mut region, 0) != 0 { // pmp.rs
    encl_free_eid(eid);
    return ret;
  }

  // create PMP region for shared memory
  if pmp::pmp_region_init_atomic(utbase, utsize, pmp_priority::PMP_PRI_BOTTOM, &mut shared_region, 0) != 0 { // pmp.rs
    pmp::pmp_region_free_atomic(region); // pmp.rs
    encl_free_eid(eid);
    return ret;
  }

  // set pmp registers for private region (not shared)
  if pmp::pmp_set_global(region, pmp::PMP_NO_PERM) != 0 { // pmp.rs
    pmp::pmp_region_free_atomic(shared_region);
    pmp::pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  // cleanup some memory regions for sanity See issue #38
  clean_enclave_memory(utbase, utsize); // enclave.rs

  // initialize enclave metadata
  enclaves[eid].eid = eid;
  enclaves[eid].regions[0].pmp_rid = region;
  enclaves[eid].regions[0].region_type = enclave_region_type::REGION_EPM;
  enclaves[eid].regions[1].pmp_rid = shared_region;
  enclaves[eid].regions[1].region_type = enclave_region_type::REGION_UTM;

  if cfg!(target_pointer_width = "32") {
    enclaves[eid].encl_satp = (base >> page::RISCV_PGSHIFT) | (opensbi::SATP_MODE_SV32 << opensbi::Const::HGATP_MODE_SHIFT); // opensbi
  }
  else {
    enclaves[eid].encl_satp = (base >> page::RISCV_PGSHIFT) | (opensbi::SATP_MODE_SV39 << opensbi::Const::HGATP_MODE_SHIFT); // opensbi
  }

  enclaves[eid].n_thread = 0;
  enclaves[eid].params = params;
  enclaves[eid].pa_params = pa_params;

  /* Init enclave state (regs etc) */
  thread::clean_state(&mut enclaves[eid].threads[0]); // thread.rs

  /* Platform create happens as the last thing before hashing/etc since
     it may modify the enclave struct */
  ret = platform_create_enclave(&enclaves[eid]);
  if ret != 0 {
    pmp::pmp_unset_global(region);
    pmp::pmp_region_free_atomic(shared_region);
    pmp::pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  /* Validate memory, prepare hash and signature for attestation */
  opensbi::spin_lock(&encl_lock); // FIXME This should error for second enter.
  ret = attest::validate_and_hash_enclave(&mut enclaves[eid]) as usize;
  /* The enclave is fresh if it has been validated and hashed but not run yet. */
  if ret != 0 {
    opensbi::spin_unlock(&encl_lock);
    platform_destroy_enclave(&enclaves[eid]);
    pmp::pmp_unset_global(region);
    pmp::pmp_region_free_atomic(shared_region);
    pmp::pmp_region_free_atomic(region);
    encl_free_eid(eid);
    return ret;
  }

  enclaves[eid].state = enclave_state::FRESH;
  /* EIDs are unsigned int in size, copy via simple copy */
  *eidptr = eid;

  opensbi::spin_unlock(&encl_lock);
  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;

}

fn is_create_args_valid(args: &mut keystone_sbi_create) -> i32 {
  let epm_start: u32;
  let epm_end: u32;

  /* printm("[create args info]: \r\n\tepm_addr: %llx\r\n\tepmsize: %llx\r\n\tutm_addr: %llx\r\n\tutmsize: %llx\r\n\truntime_addr: %llx\r\n\tuser_addr: %llx\r\n\tfree_addr: %llx\r\n", */
  /*        args.epm_region.paddr, */
  /*        args.epm_region.size, */
  /*        args.utm_region.paddr, */
  /*        args.utm_region.size, */
  /*        args.runtime_paddr, */
  /*        args.user_paddr, */
  /*        args.free_paddr); */

  // check if physical addresses are valid
  if args.epm_region.size <= 0 {
    return 0;
  }

  // check if overflow
  if args.epm_region.paddr >= args.epm_region.paddr + args.epm_region.size {
    return 0;
  }
  if args.utm_region.paddr >= args.utm_region.paddr + args.utm_region.size {
    return 0;
  }

  epm_start = args.epm_region.paddr;
  epm_end = args.epm_region.paddr + args.epm_region.size;

  // check if physical addresses are in the range
  if args.runtime_paddr < epm_start || args.runtime_paddr >= epm_end {
    return 0;
  }
  if args.user_paddr < epm_start || args.user_paddr >= epm_end {
    return 0;
  }
  if args.free_paddr < epm_start || args.free_paddr > epm_end {
    // note: free_paddr == epm_end if there's no free memory
    return 0;
  }
  
  // check the order of physical addresses
  if args.runtime_paddr > args.user_paddr {
    return 0;
  }
    
  if args.user_paddr > args.free_paddr {
    return 0;
  }

  return 1;
}

pub fn get_enclave_region_index(eid: enclave_id, entype: enclave_region_type) -> i32 {
  let i: usize;
  for i in 0..ENCLAVE_REGIONS_MAX {
    if enclaves[eid].regions[i].region_type == entype {
      return i as i32;
    }
  }
  // No such region for this enclave
  return -1;
}

/*
 * Fully destroys an enclave
 * Deallocates EID, clears epm, etc
 * Fails only if the enclave isn't running.
 */
pub fn destroy_enclave(eid: enclave_id) -> usize {
  let destroyable: i32;

  opensbi::spin_lock(&encl_lock);
  destroyable = (enclave_exists(eid as usize) && enclaves[eid as usize].state <= enclave_state::STOPPED) as i32;
  /* update the enclave state first so that
   * no SM can run the enclave any longer */
  if destroyable != 0 {
    enclaves[eid as usize].state = enclave_state::DESTROYING;
  }
  opensbi::spin_unlock(&encl_lock);

  if destroyable == 0 {
    return ERROR::SBI_ERR_SM_ENCLAVE_NOT_DESTROYABLE;
  }

  // 0. Let the platform specifics do cleanup/modifications
  platform_destroy_enclave(&enclaves[eid]);


  // 1. clear all the data in the enclave pages
  // requires no lock (single runner)
  let i: i32;
  let base: *mut usize;
  let size: usize;
  let rid: region_id;
  for i in 0..ENCLAVE_REGIONS_MAX {

    if enclaves[eid].regions[i].region_type == enclave_region_type::REGION_INVALID || enclaves[eid].regions[i].region_type == enclave_region_type::REGION_UTM {
        continue;
       }
      
    //1.a Clear all pages
    rid = enclaves[eid].regions[i].pmp_rid;
    base = pmp::pmp_region_get_addr(rid) as *mut usize;
    size = pmp::pmp_region_get_size(rid) as usize;
    opensbi::sbi_memset(base as *mut usize, 0, size); // opensbi 函数

    //1.b free pmp region
    pmp::pmp_unset_global(rid);
    pmp::pmp_region_free_atomic(rid);
  }

  // 2. free pmp region for UTM
  rid = get_enclave_region_index(eid, enclave_region_type::REGION_UTM);
  if rid != -1 {
    pmp::pmp_region_free_atomic(enclaves[eid as usize].regions[rid as usize].pmp_rid);
  }

  enclaves[eid].encl_satp = 0;
  enclaves[eid].n_thread = 0;
  enclaves[eid].params = runtime_va_params_t::new();
  enclaves[eid].pa_params = runtime_pa_params::new();
  for i in 0..ENCLAVE_REGIONS_MAX {
    enclaves[eid].regions[i].region_type = enclave_region_type::REGION_INVALID;
  }

  // 3. release eid
  encl_free_eid(eid);

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

// opensbi
pub fn run_enclave(regs: &mut sbi_trap::sbi_trap_regs, eid: enclave_id) -> usize {
  let runable: bool;

  opensbi::spin_lock(&encl_lock);
  runable = enclave_exists(eid) && enclaves[eid].state == enclave_state::FRESH;
  if runable {
    enclaves[eid].state = enclave_state::RUNNING;
    enclaves[eid].n_thread += 1;
  }
  opensbi::spin_unlock(&encl_lock);

  if !runable {
    return ERROR::SBI_ERR_SM_ENCLAVE_NOT_FRESH;
  }

  // Enclave is OK to run, context switch to it
  context_switch_to_enclave(regs, eid, 1);

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

pub fn exit_enclave(regs: &mut sbi_trap::sbi_trap_regs, eid: enclave_id) -> usize {

  opensbi::spin_lock(&encl_lock);
  let exitable = enclaves[eid].state == enclave_state::RUNNING;
  if exitable {
    enclaves[eid].n_thread -= 1;
    if enclaves[eid].n_thread == 0 {
      enclaves[eid].state = enclave_state::STOPPED;
    }
  }
  opensbi::spin_unlock(&encl_lock);

  if !exitable {
    return ERROR::SBI_ERR_SM_ENCLAVE_NOT_RUNNING;
  }
  context_switch_to_host(regs, eid, false);

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;

}
// opensbi函数
pub fn stop_enclave(regs: &mut sbi_trap::sbi_trap_regs, request: usize, eid: enclave_id) -> usize {
  let stoppable: bool;

  opensbi::spin_lock(&encl_lock);
  stoppable = enclaves[eid].state == enclave_state::RUNNING;
  if stoppable {
    enclaves[eid].n_thread -= 1;
    if enclaves[eid].n_thread == 0 {
      enclaves[eid].state = enclave_state::STOPPED;
    }
  }
  opensbi::spin_unlock(&encl_lock);

  if !stoppable {
    return ERROR::SBI_ERR_SM_ENCLAVE_NOT_RUNNING;
  }

  context_switch_to_host(regs, eid, request == STOP_EDGE_CALL_HOST);

  match request {
    STOP_TIMER_INTERRUPT => return ERROR::SBI_ERR_SM_ENCLAVE_INTERRUPTED,
    STOP_EDGE_CALL_HOST => return ERROR::SBI_ERR_SM_ENCLAVE_EDGE_CALL_HOST,
    _ => return ERROR::SBI_ERR_SM_ENCLAVE_UNKNOWN_ERROR
  }
}

pub fn resume_enclave(regs: &mut sbi_trap::sbi_trap_regs, eid: enclave_id) -> usize {
  let resumable: bool;

  opensbi::spin_lock(&encl_lock);
  resumable = enclave_exists(eid) && (enclaves[eid].state == enclave_state::RUNNING || enclaves[eid].state == enclave_state::STOPPED) && enclaves[eid].n_thread < MAX_ENCL_THREADS;

  if !resumable {
    opensbi::spin_unlock(&encl_lock);
    return ERROR::SBI_ERR_SM_ENCLAVE_NOT_RESUMABLE;
  }
  else {
    enclaves[eid].n_thread += 1;
    enclaves[eid].state = enclave_state::RUNNING;
  }
  opensbi::spin_unlock(&encl_lock);

  // Enclave is OK to resume, context switch to it
  context_switch_to_enclave(regs, eid, 0);

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

pub fn attest_enclave(report_ptr: usize, data: usize, size: usize, eid: enclave_id) -> usize {
  let attestable: bool;
  let report: report;
  let ret: usize;

  if size > ATTEST_DATA_MAXLEN {
    return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT;
  }

  opensbi::spin_lock(&encl_lock);
  attestable = enclave_exists(eid) && (enclaves[eid].state >= enclave_state::FRESH);

  if !attestable {
    ret = ERROR::SBI_ERR_SM_ENCLAVE_NOT_INITIALIZED;
    opensbi::spin_unlock(&encl_lock);
    return ret;
  }

  unsafe {
    let enclave_data: usize = (&report.enclave.data as *const u8) as usize;
    
    /* copy data to be signed */
    ret = copy_enclave_data(enclave_data, data, size);
    
    if ret != 0 {
      ret = ERROR::SBI_ERR_SM_ENCLAVE_NOT_ACCESSIBLE;
      opensbi::spin_unlock(&encl_lock);
      return ret;
    }
  }
  

  report.enclave.data_len = size as u64;

  opensbi::spin_unlock(&encl_lock); // Don't need to wait while signing, which might take some time

  // opensbi 函数，复制 src 到 dst
  opensbi::sbi_memcpy(report.dev_public_key, dev_public_key, crypto::PUBLIC_KEY_SIZE);
  opensbi::sbi_memcpy(report.sm.hash, sm_hash, crypto::MDSIZE);
  opensbi::sbi_memcpy(report.sm.public_key, sm_public_key, crypto::PUBLIC_KEY_SIZE);
  opensbi::sbi_memcpy(report.sm.signature, sm_signature, crypto::SIGNATURE_SIZE);
  opensbi::sbi_memcpy(report.enclave.hash, enclaves[eid].hash, crypto::MDSIZE);
  sm::sm_sign(&report.enclave.signature, &report.enclave, mem::size_of::<enclave_report>() - crypto::SIGNATURE_SIZE - ATTEST_DATA_MAXLEN + size);

  opensbi::spin_lock(&encl_lock);

  /* copy report to the enclave */
  ret = copy_enclave_report(&mut enclaves[eid], report_ptr, &mut report);

  if ret != 0 {
    ret = ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT;
    opensbi::spin_unlock(&encl_lock);
    return ret;
  }

  ret = ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;

  opensbi::spin_unlock(&encl_lock);
  return ret;
}

/* copies data from enclave, source must be inside EPM */
fn copy_enclave_data(dest: usize, source: usize, size: usize) -> usize {

  let illegal: i32 = mprv::copy_to_sm(dest, source, size);

  if illegal != 0 {
    return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT;
  }
  else {
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }
}

/* copies data into enclave, destination must be inside EPM */
fn copy_enclave_report(enclave: &mut enclave, dest: usize, source: &mut report) -> usize {

  let illegal: i32 = mprv::copy_from_sm(dest, source as *const report as usize, mem::size_of::<report>());

  if illegal != 0 {
    return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT;
  }
  else {
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }

}

pub fn get_sealing_key(seal_key: &mut sealing_key, key_ident: &[u8], key_ident_size: usize, eid: enclave_id) -> usize {
  let key_struct: *mut sealing_key = seal_key as *mut sealing_key;
  let ret: i32;

  /* derive key */
  ret = sm::sm_derive_sealing_key(&mut seal_key.key, key_ident, key_ident_size, &enclaves[eid].hash);
  if ret != 0 {
    return ERROR::SBI_ERR_SM_ENCLAVE_UNKNOWN_ERROR;
  }

  /* sign derived key */
  sm::sm_sign(&(*key_struct).signature, (*key_struct).key,
  SEALING_KEY_SIZE);

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

fn enclave_exists(eid: usize) -> bool {
  eid >= 0 && eid < ENCL_MAX && enclaves[eid].state >= 0
}

fn encl_alloc_eid(_eid: &mut enclave_id) -> usize {
  let eid: enclave_id;

  opensbi::spin_lock(&encl_lock); // opensbi

  for i in 0..ENCL_MAX {
    eid = i;
    if enclaves[eid].state == enclave_state::INVALID {
      break;
    }
  }

  if eid != ENCL_MAX {
    enclaves[eid].state = enclave_state::ALLOCATED;
  }

  opensbi::spin_unlock(&encl_lock); // opensbi 函数

  if eid != ENCL_MAX {
    *_eid = eid;
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }
  else {
    return ERROR::SBI_ERR_SM_ENCLAVE_NO_FREE_RESOURCE;
  }
}

fn encl_free_eid(eid: enclave_id) -> usize {
  spin_lock(&encl_lock);
  enclaves[eid].state = enclave_state::INVALID;
  spin_unlock(&encl_lock);
  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS as usize;
}

fn clean_enclave_memory(utbase: usize, utsize: usize) -> usize {

  // This function is quite temporary. See issue #38

  // Zero out the untrusted memory region, since it may be in
  // indeterminate state.
  // opensbi
  opensbi::sbi_memset(utbase as *mut usize, 0, utsize); // opensbi

  return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

fn context_switch_to_enclave(regs: &mut sbi_trap_regs, eid: enclave_id, load_parameters: i32) {
  /* save host context */
  thread::swap_prev_state(&mut enclaves[eid].threads[0], regs, 1);
  thread::swap_prev_mepc(&mut enclaves[eid].threads[0], regs, regs.mepc);
  thread::swap_prev_mstatus(&mut enclaves[eid].threads[0], regs, regs.mstatus);

  let interrupts: usize = 0;
  // opensbi
  opensbi::csr_write("mideleg", interrupts);

  if load_parameters != 0 {
    // passing parameters for a first run
    opensbi::csr_write("sepc", enclaves[eid].params.user_entry as usize);
    regs.mepc = (enclaves[eid].params.runtime_entry - 4) as usize; // regs->mepc will be +4 before sbi_ecall_handler return
    // opensbi
    regs.mstatus = 1 << opensbi::MSTATUS_MPP_SHIFT; // opensbi
    // $a1: (PA) DRAM base,
    regs.a1 = enclaves[eid].pa_params.dram_base as usize;
    // $a2: (PA) DRAM size,
    regs.a2 = enclaves[eid].pa_params.dram_size as usize;
    // $a3: (PA) kernel location,
    regs.a3 = enclaves[eid].pa_params.runtime_base as usize;
    // $a4: (PA) user location,
    regs.a4 = enclaves[eid].pa_params.user_base as usize;
    // $a5: (PA) freemem location,
    regs.a5 = enclaves[eid].pa_params.free_base as usize;
    // $a6: (VA) utm base,
    regs.a6 = enclaves[eid].params.untrusted_ptr as usize;
    // $a7: (usize) utm size
    regs.a7 = enclaves[eid].params.untrusted_size as usize;

    // switch to the initial enclave page table
    opensbi::csr_write(satp, enclaves[eid].encl_satp);
  }

  thread::switch_vector_enclave();

  // set PMP
  sm::osm_pmp_set(pmp::PMP_NO_PERM as u8);
  let memid: i32;
  for memid in 0..ENCLAVE_REGIONS_MAX {
    if enclaves[eid].regions[memid].region_type != enclave_region_type::REGION_INVALID {
      pmp::pmp_set_keystone(enclaves[eid].regions[memid].pmp_rid, pmp::PMP_ALL_PERM as u8);
    }
  }

  // Setup any platform specific defenses
  platform_switch_to_enclave(&(enclaves[eid]));
  cpu::cpu_enter_enclave_context(eid);
}

fn context_switch_to_host(regs: &mut sbi_trap_regs, eid: enclave_id, return_on_resume: bool) {

  // set PMP
  let memid: i32;
  for memid in 0..ENCLAVE_REGIONS_MAX {
    if enclaves[eid].regions[memid].region_type != enclave_region_type::REGION_INVALID {
      pmp::pmp_set_keystone(enclaves[eid].regions[memid].pmp_rid, pmp::PMP_NO_PERM as u8);
    }
  }
  sm::osm_pmp_set(pmp::PMP_ALL_PERM as u8);

  let interrupts: usize = MIP_SSIP | MIP_STIP | MIP_SEIP; // opensbi
  opensbi::csr_write("mideleg", interrupts);

  /* restore host context */
  thread::swap_prev_state(&mut enclaves[eid].threads[0], regs, return_on_resume as usize);
  thread::swap_prev_mepc(&mut enclaves[eid].threads[0], regs, regs.mepc);
  thread::swap_prev_mstatus(&mut enclaves[eid].threads[0], regs, regs.mstatus);

  thread::switch_vector_host();

  let pending: usize = csr_read(mip);

  if pending & MIP_MTIP != 0 {
    csr_clear(mip, MIP_MTIP);
    csr_set(mip, MIP_STIP);
  }
  if pending & MIP_MSIP != 0 {
    csr_clear(mip, MIP_MSIP);
    csr_set(mip, MIP_SSIP);
  }
  if pending & MIP_MEIP != 0 {
    csr_clear(mip, MIP_MEIP);
    csr_set(mip, MIP_SEIP);
  }

  // Reconfigure platform specific defenses
  platform_switch_from_enclave(&(enclaves[eid]));

  cpu::cpu_exit_enclave_context();

  return;
}


// TODO: This function is externally used.
// refactoring needed
/*
 * Init all metadata as needed for keeping track of enclaves
 * Called once by the SM on startup
 */
pub fn enclave_init_metadata(){
  let eid: enclave_id;
  let i: i32 = 0;

  /* Assumes eids are incrementing values, which they are for now */
  for(eid=0; eid < ENCL_MAX; eid++){
    enclaves[eid].state = INVALID;

    // Clear out regions
    for(i=0; i < ENCLAVE_REGIONS_MAX; i++){
      enclaves[eid].regions[i].type = REGION_INVALID;
    }
    /* Fire all platform specific init for each enclave */
    platform_init_enclave(&(enclaves[eid]));
  }

}
