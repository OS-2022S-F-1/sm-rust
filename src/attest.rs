use crate::enclave::enclave;
use crate::page;
use std::mem;
use crate::error_code::ERROR;
use crate::crypto;
use crate::sm;

pub fn validate_and_hash_enclave(enclave: &mut enclave) -> usize {

    let hash_ctx: crypto::hash_ctx;
    let ptlevel: i32 = page::RISCV_PGLEVEL_TOP as i32;
  
    crypto::hash_init(&mut hash_ctx); // crypto.rs
  
    // hash the runtime parameters
    crypto::hash_extend(&mut hash_ctx, enclave.params as &[u8], mem::size_of::<sm::runtime_va_params_t>());
    // crypto.rs
  
    let runtime_max_seen: usize = 0;
    let user_max_seen: usize = 0;
  
    // hash the epm contents including the virtual addresses
    let valid: i32 = validate_and_hash_epm(&hash_ctx,
                                      ptlevel,
                                      (pte_t*) (enclave.encl_satp << RISCV_PGSHIFT),
                                      0, 0, enclave, &runtime_max_seen, &user_max_seen);
  
    if valid == -1 {
      return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_PTE;
    }
  
    hash_finalize(enclave.hash, &hash_ctx); // crypto.rs
  
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }

fn validate_and_hash_epm(hash_ctx* hash_ctx, int level,
    pte_t* tb, usize vaddr, int contiguous,
    struct enclave* encl,
    usize* runtime_max_seen,
    usize* user_max_seen) {
  pte_t* walk;
  int i;

  //TODO check for failures
  usize epm_start, epm_size;
  usize utm_start, utm_size;
  int idx = get_enclave_region_index(encl->eid, REGION_EPM);
  epm_start = pmp_region_get_addr(encl->regions[idx].pmp_rid);
  epm_size = pmp_region_get_size(encl->regions[idx].pmp_rid);
  idx = get_enclave_region_index(encl->eid, REGION_UTM);
  utm_start = pmp_region_get_addr(encl->regions[idx].pmp_rid);
  utm_size = pmp_region_get_size(encl->regions[idx].pmp_rid);



  /* iterate over PTEs */
  for (walk=tb, i=0; walk < tb + (RISCV_PGSIZE/sizeof(pte_t)); walk += 1,i++)
  {
  if (*walk == 0) {
  contiguous = 0;
  continue;
  }
  usize vpn;
  usize phys_addr = (*walk >> PTE_PPN_SHIFT) << RISCV_PGSHIFT;

  /* Check for blatently invalid mappings */
  int map_in_epm = (phys_addr >= epm_start &&
  phys_addr < epm_start + epm_size);
  int map_in_utm = (phys_addr >= utm_start &&
  phys_addr < utm_start + utm_size);

  /* EPM may map anything, UTM may not map pgtables */
  if(!map_in_epm && (!map_in_utm || level != 1)){
  goto fatal_bail;
  }


  /* propagate the highest bit of the VA */
  if ( level == RISCV_PGLEVEL_TOP && i & RISCV_PGTABLE_HIGHEST_BIT )
  vpn = ((-1UL << RISCV_PGLEVEL_BITS) | (i & RISCV_PGLEVEL_MASK));
  else
  vpn = ((vaddr << RISCV_PGLEVEL_BITS) | (i & RISCV_PGLEVEL_MASK));

  usize va_start = vpn << RISCV_PGSHIFT;

  /* include the first virtual address of a contiguous range */
  if (level == 1 && !contiguous)
  {

  hash_extend(hash_ctx, &va_start, sizeof(usize));
  //printm("VA hashed: 0x%lx\n", va_start);
  contiguous = 1;
  }

  if (level == 1)
  {

  /*
  * This is where we enforce the at-most-one-mapping property.
  * To make our lives easier, we also require a 'linear' mapping
  * (for each of the user and runtime spaces independently).
  *
  * That is: Given V1->P1 and V2->P2:
  *
  * V1 < V2  ==> P1 < P2  (Only for within a given space)
  *
  * V1 != V2 ==> P1 != P2
  *
  * We also validate that all utm vaddrs -> utm paddrs
  */
  int in_runtime = ((phys_addr >= encl->pa_params.runtime_base) &&
    (phys_addr < encl->pa_params.user_base));
  int in_user = ((phys_addr >= encl->pa_params.user_base) &&
  (phys_addr < encl->pa_params.free_base));

  /* Validate U bit */
  if(in_user && !(*walk & PTE_U)){
  goto fatal_bail;
  }

  /* If the vaddr is in UTM, the paddr must be in UTM */
  if(va_start >= encl->params.untrusted_ptr &&
  va_start < (encl->params.untrusted_ptr + encl->params.untrusted_size) &&
  !map_in_utm){
  goto fatal_bail;
  }

  /* Do linear mapping validation */
  if(in_runtime){
  if(phys_addr <= *runtime_max_seen){
  goto fatal_bail;
  }
  else{
  *runtime_max_seen = phys_addr;
  }
  }
  else if(in_user){
  if(phys_addr <= *user_max_seen){
  goto fatal_bail;
  }
  else{
  *user_max_seen = phys_addr;
  }
  }
  else if(map_in_utm){
  // we checked this above, its OK
  }
  else{
  //printm("BAD GENERIC MAP %x %x %x\n", in_runtime, in_user, map_in_utm);
  goto fatal_bail;
  }

  /* Page is valid, add it to the hash */

  /* if PTE is leaf, extend hash for the page */
  hash_extend_page(hash_ctx, (void*)phys_addr);



  //printm("PAGE hashed: 0x%lx (pa: 0x%lx)\n", vpn << RISCV_PGSHIFT, phys_addr);
  }
  else
  {
  /* otherwise, recurse on a lower level */
  contiguous = validate_and_hash_epm(hash_ctx,
                    level - 1,
                    (pte_t*) phys_addr,
                    vpn,
                    contiguous,
                    encl,
                    runtime_max_seen,
                    user_max_seen);
  if(contiguous == -1){
  sbi_printf("BAD MAP: %lx->%lx epm %x %lx uer %x %lx\n",
  va_start,phys_addr,
  //in_runtime,
  0,
  encl->pa_params.runtime_base,
  0,
  //in_user,
  encl->pa_params.user_base);
  goto fatal_bail;
  }
  }
  }

  return contiguous;

  fatal_bail:
  return -1;
  }