use crate::enclave;
use crate::page;
use std::mem;
use crate::error_code::ERROR;
use crate::crypto;
use crate::sm;
use crate::pmp;

type pte_t = usize;

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
  ::std::slice::from_raw_parts(
      (p as *const T) as *const u8,
      ::std::mem::size_of::<T>(),
  )
}

pub fn validate_and_hash_enclave(enclave: &mut enclave::enclave) -> usize {

    let hash_ctx: crypto::hash_ctx;
    let ptlevel: i32 = page::RISCV_PGLEVEL_TOP as i32;
  
    crypto::hash_init(&mut hash_ctx); // crypto.rs

    unsafe {
      let params: &[u8] = any_as_u8_slice(&enclave.params);
      // hash the runtime parameters
      crypto::hash_extend(&mut hash_ctx, params, mem::size_of::<sm::runtime_va_params_t>());
      // crypto.rs
    }
    
    let runtime_max_seen: usize = 0;
    let user_max_seen: usize = 0;
  
    // hash the epm contents including the virtual addresses
    let valid: i32 = validate_and_hash_epm(&mut hash_ctx, ptlevel, enclave.encl_satp << page::RISCV_PGSHIFT, 0, 0, enclave, &mut runtime_max_seen, &mut user_max_seen);
  
    if valid == -1 {
      return ERROR::SBI_ERR_SM_ENCLAVE_ILLEGAL_PTE;
    }
  
    crypto::hash_finalize(&mut enclave.hash, &mut hash_ctx); // crypto.rs
  
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
  }

fn validate_and_hash_epm(hash_ctx: &mut crypto::hash_ctx, level: i32, tb: pte_t, vaddr: usize, contiguous: i32, encl: &mut enclave::enclave, runtime_max_seen: &mut usize, user_max_seen: &mut usize) -> i32 {
      
  let walk: pte_t;
  let i: i32;

  //TODO check for failures
  let epm_start: usize;
  let epm_size: usize;
  let utm_start: usize;
  let utm_size: usize;

  let idx: i32 = enclave::get_enclave_region_index(encl.eid, enclave::enclave_region_type::REGION_EPM);
  epm_start = pmp::pmp_region_get_addr(encl.regions[idx as usize].pmp_rid) as usize;
  epm_size = pmp::pmp_region_get_size(encl.regions[idx as usize].pmp_rid) as usize;
  idx = enclave::get_enclave_region_index(encl.eid, enclave::enclave_region_type::REGION_UTM);
  utm_start = pmp::pmp_region_get_addr(encl.regions[idx as usize].pmp_rid) as usize;
  utm_size = pmp::pmp_region_get_size(encl.regions[idx as usize].pmp_rid) as usize;

  /* iterate over PTEs */
  walk = tb;
  loop {
    if walk >= tb + (page::RISCV_PGSIZE / mem::size_of::<pte_t>()) {
      break;
    }
    if walk == 0 {
      contiguous = 0;
      continue;
    }

    let vpn: usize;
    let phys_addr: usize = (walk >> page::PTE_PPN_SHIFT) << page::RISCV_PGSHIFT;

    /* Check for blatently invalid mappings */
    let map_in_epm: bool = phys_addr >= epm_start && phys_addr < epm_start + epm_size;
    let map_in_utm: bool = phys_addr >= utm_start && phys_addr < utm_start + utm_size;

    /* EPM may map anything, UTM may not map pgtables */
    if !map_in_epm && (!map_in_utm || level != 1) {
      return -1;
    }

    /* propagate the highest bit of the VA */
    if level == ((page::RISCV_PGLEVEL_TOP != 0) && (i as usize & page::page::RISCV_PGTABLE_HIGHEST_BIT) != 0) as i32 {
      vpn = (usize::MAX << page::page::RISCV_PGLEVEL_BITS) | (i as usize & page::page::RISCV_PGLEVEL_MASK);
    }
    else {
      vpn = (vaddr << page::page::RISCV_PGLEVEL_BITS) | (i as usize & page::page::RISCV_PGLEVEL_MASK);
    }
    let va_start: usize = vpn << page::RISCV_PGSHIFT;

    /* include the first virtual address of a contiguous range */
    if level == 1 && contiguous == 0 {
      unsafe {
        let va_start_ptr: &[u8] = any_as_u8_slice(&va_start);
        crypto::hash_extend(hash_ctx, va_start_ptr, mem::size_of::<usize>());
      }
      //printm("VA hashed: 0x%lx\n", va_start);
      contiguous = 1;
    }

    if level == 1 {
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
      let in_runtime: bool = (phys_addr >= encl.pa_params.runtime_base) &&
        (phys_addr < encl.pa_params.user_base);
      let in_user: bool = (phys_addr >= encl.pa_params.user_base) &&
      (phys_addr < encl.pa_params.free_base);

      /* Validate U bit */
      if in_user && (walk & page::PTE_U) == 0 {
        return -1;
      }

      /* If the vaddr is in UTM, the paddr must be in UTM */
      if va_start >= encl.params.untrusted_ptr as usize && va_start < (encl.params.untrusted_ptr + encl.params.untrusted_size) as usize && !map_in_utm {
        return -1;
      }

      /* Do linear mapping validation */
      if in_runtime {
        if phys_addr <= *runtime_max_seen {
          return -1;
        }
        else {
          *runtime_max_seen = phys_addr;
        }
      }
      else if in_user {
        if phys_addr <= *user_max_seen {
          return -1;
        }
        else {
          *user_max_seen = phys_addr;
        }
      }
      else if map_in_utm {
        // we checked this above, its OK
      }
      else {
        //printm("BAD GENERIC MAP %x %x %x\n", in_runtime, in_user, map_in_utm);
        return -1;
      }
  
      /* Page is valid, add it to the hash */
    
      /* if PTE is leaf, extend hash for the page */
      unsafe {
        let phy_addr_ptr: &[u8] = any_as_u8_slice(&phys_addr);
        crypto::hash_extend_page(hash_ctx, phy_addr_ptr);
      }

      //printm("PAGE hashed: 0x%lx (pa: 0x%lx)\n", vpn << RISCV_PGSHIFT, phys_addr);
    }
    else {
      /* otherwise, recurse on a lower level */
      contiguous = validate_and_hash_epm(hash_ctx,
                        level - 1,
                        phys_addr,
                        vpn,
                        contiguous,
                        encl,
                        runtime_max_seen,
                        user_max_seen);
      if contiguous == -1 {
        println!("BAD MAP: {}->{} epm {} {} uer {} {}\n",
        va_start, phys_addr, 
        //in_runtime,
        0,
        encl.pa_params.runtime_base,
        0,
        //in_user,
        encl.pa_params.user_base);
        return -1;
      }
    }
    walk += 1;
  }
  
  return contiguous;
}
