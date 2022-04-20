const PMP_N_REG: u32 = 8; //number of PMP registers
const PMP_MAX_N_REGION: u32 = 16; //maximum number of PMP regions

fn set_bit(bitmap, n) {
    bitmap = bitmap | (0x1 << (n));
}

fn unset_bit(bitmap, n) {
    bitmap = bitmap & ~(0x1 << (n));
}

fn test_bit(bitmap, n) -> u32 {
    bitmap & (0x1 << (n))
}

enum pmp_priority {
    PMP_PRI_ANY,
    PMP_PRI_TOP,
    PMP_PRI_BOTTOM,
}

fn PMP_SET(n, g, addr, pmpc) {
  
}

fn PMP_UNSET() {

}

fn PMP_ERROR() {

}

struct ipi_msg {
    pending: atomic_t,
    perm: u8
}

struct pmp_region {
    size: u64,
    addmode: u8,
    addr: u32,
    allow_overlap: i32,
    reg_idx: i32
}

fn pmp_init() {
    let pmpaddr: u32 = 0;
    let pmpcfg: u32 = 0;
    let mut i = 0;
    loop {
        if i == PMP_N_REG {
            break;
        }
        switch(i) {

        }
        i += 1;
    }
}

fn pmp_region_init_atomic(start: uintptr_t, size: uint64_t, priority: pmp_priority, rid: *mut region_id, allow_overlap: i32) -> i32 {
  spin_lock(&pmp_lock);
  let ret: i32 = pmp_region_init(start, size, priority, rid, allow_overlap); // pmp.rs
  spin_unlock(&pmp_lock);
  return ret;
}

fn pmp_region_init(start: uintptr_t, size: uint64_t, priority: pmp_priority, rid: *mut i32, allow_overlap: i32) -> i32 {
  if !size {
    PMP_ERROR(SBI_ERR_SM_PMP_REGION_SIZE_INVALID, "Invalid PMP size");
  }

  /* overlap detection */
  if !allow_overlap {
    if detect_region_overlap(start, size) {
      return SBI_ERR_SM_PMP_REGION_OVERLAP;
    }
  }

  /* PMP granularity check */
  if size != -1UL && (size & (RISCV_PGSIZE - 1)) {
    PMP_ERROR(SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY, "PMP granularity is RISCV_PGSIZE");
  }
  if(start & (RISCV_PGSIZE - 1))
    PMP_ERROR(SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY, "PMP granularity is RISCV_PGSIZE");

  /* if the address covers the entire RAM or it's NAPOT */
  if ((size == -1UL && start == 0) ||
      (!(size & (size - 1)) && !(start & (size - 1)))) {
    return napot_region_init(start, size, priority, rid, allow_overlap);
  }
  else
  {
    if(priority != PMP_PRI_ANY &&
      (priority != PMP_PRI_TOP || start != 0)) {
      PMP_ERROR(SBI_ERR_SM_PMP_REGION_IMPOSSIBLE_TOR, "The top-priority TOR PMP entry must start from address 0");
    }

    return tor_region_init(start, size, priority, rid, allow_overlap);
  }
}

fn pmp_region_free_atomic() -> i32 {

}

fn pmp_set_keystone() -> i32 {

}

fn pmp_set_global(region_idx: i32, perm: uint8_t) -> i32 {
  if !is_pmp_region_valid(region_idx) {
    PMP_ERROR(SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index");
  }

  send_and_sync_pmp_ipi(region_idx, SBI_PMP_IPI_TYPE_SET, perm); // ipi.rs

  return SBI_ERR_SM_PMP_SUCCESS;
}

fn pmp_unset() -> i32 {

}

fn pmp_unset_global() -> i32 {

}

fn pmp_detect_region_overlap_atomic() -> i32 {

}

fn hand_pmp_ipi() -> i32 {

}

fn pmp_region_get_addr() -> {

}

fn pmp_region_get_size() -> {

}

