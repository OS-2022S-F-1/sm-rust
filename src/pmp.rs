use crate::error_code::ERROR;

pub type region = u32;
pub type region_id = i32;
pub type uint32_t = u32;

pub const PMP_N_REG: u32 = 8; //number of PMP registers
pub const PMP_MAX_N_REGION: u32 = 16; //maximum number of PMP regions

static region_def_bitmap: uint32_t = 0;
static reg_bitmap: uint32_t = 0;

pub const PMP_ALL_PERM: u8 = (PMP_W | PMP_X | PMP_R);
pub const PMP_NO_PERM: u8 = 0;


fn set_bit(bitmap: usize, n: usize) {
  bitmap = bitmap | (0x1 << (n));
}

fn unset_bit(bitmap: usize, n: usize) {
  bitmap = bitmap & !(0x1 << (n));
}

fn test_bit(bitmap: usize, n: usize) -> usize {
  bitmap & (0x1 << (n))
}

pub enum pmp_priority {
    PMP_PRI_ANY,
    PMP_PRI_TOP,
    PMP_PRI_BOTTOM,
}

fn PMP_SET(n: usize, g: usize, addr: usize, pmpc: usize) {
  
}

fn PMP_UNSET() {

}

fn PMP_ERROR(error: u32, msg: String) -> u32 {
  // sbi_printf("%s:" + msg + "\n", __func__);
  return error
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

pub fn pmp_init() {
    let pmpaddr: u32 = 0;
    let pmpcfg: u32 = 0;
    let mut i = 0;
}

pub fn pmp_region_init(start: uintptr_t, size: uint64_t, priority: pmp_priority, rid: *mut i32, allow_overlap: i32) -> i32 {
  if size == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_SIZE_INVALID, "Invalid PMP size");
  }

  /* overlap detection */
  if allow_overlap == 0 {
    if detect_region_overlap(start, size) != 0 {
      return ERROR::SBI_ERR_SM_PMP_REGION_OVERLAP as i32;
    }
  }

  /* PMP granularity check */
  if (size != -1 as usize) && (size & (RISCV_PGSIZE - 1)) {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY, "PMP granularity is RISCV_PGSIZE");
  }
  if (start & (RISCV_PGSIZE - 1)) != 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY, "PMP granularity is RISCV_PGSIZE");
  }

  /* if the address covers the entire RAM or it's NAPOT */
  if (size == -1 as usize && start == 0) || (!(size & (size - 1)) && !(start & (size - 1))) {
    return napot_region_init(start, size, priority, rid, allow_overlap);
  }
  else {
    if (priority != pmp_priority::PMP_PRI_ANY) && (priority != PMP_PRI_TOP || start != 0) {
      PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_IMPOSSIBLE_TOR, "The top-priority TOR PMP entry must start from address 0");
    }

    return tor_region_init(start, size, priority, rid, allow_overlap);
  }
}

fn napot_region_init(start: uintptr_t, size: uint64_t, priority: pmp_priority, rid: &mut region_id, allow_overlap: i32) -> i32 {
  let reg_idx: pmpreg_id = -1;
  let region_idx: region_id = -1;

  sm_assert(size); // assert.rs
  sm_assert(rid);

  if !(size == -1 as usize && start == 0) {
    sm_assert(!(size & (size-1)));
    sm_assert(!(start & (size - 1)));
    sm_assert(!(size & (RISCV_PGSIZE-1)));
    sm_assert(!(start & (RISCV_PGSIZE-1)));
  }

  //find avaiable pmp region idx
  region_idx = get_free_region_idx();
  if region_idx < 0 || region_idx > PMP_MAX_N_REGION {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "Reached the maximum number of PMP regions");
  }

  rid = region_idx;

  match(priority) {
    pmp_priority::PMP_PRI_ANY => {
      reg_idx = get_free_reg_idx();
      if reg_idx < 0 {
        PMP_ERROR(SBI_ERR_SM_PMP_REGION_MAX_REACHED, "No available PMP register");
      }
      if test_bit(reg_bitmap, reg_idx) || reg_idx >= PMP_N_REG {
        PMP_ERROR(SBI_ERR_SM_PMP_REGION_MAX_REACHED, "PMP register unavailable");
      }
    },
    pmp_priority::PMP_PRI_TOP => {
      reg_idx = 0;
      if test_bit(reg_bitmap, reg_idx) {
        PMP_ERROR(SBI_ERR_SM_PMP_REGION_MAX_REACHED, "PMP register unavailable");
      }
    },
    pmp_priority::PMP_PRI_BOTTOM => {
      /* the bottom register can be used by multiple regions,
       * so we don't check its availability */
      reg_idx = PMP_N_REG - 1;
    },
    _ => {
      sm_assert(0);
    }
  }

  // initialize the region
  region_init(region_idx, start, size, PMP_A_NAPOT, allow_overlap, reg_idx);
  set_bit(region_def_bitmap, region_idx);
  set_bit(reg_bitmap, reg_idx);

  return ERROR::SBI_ERR_SM_PMP_SUCCESS;
}

fn get_free_region_idx() -> region_id {
  return search_rightmost_unset(region_def_bitmap, PMP_MAX_N_REGION, 0x1);
}


pub fn pmp_region_init_atomic(start: uintptr_t, size: uint64_t, priority: pmp_priority, rid: &mut region_id, allow_overlap: i32) -> i32 {
  let ret: i32;
  spin_lock(&pmp_lock);
  ret = pmp_region_init(start, size, priority, rid, allow_overlap); // pmp.rs
  spin_unlock(&pmp_lock);
  return ret;
}

pub fn pmp_region_free_atomic(region_idx: i32) -> i32{

  spin_lock(&pmp_lock);

  if is_pmp_region_valid(region_idx) == 0 {
    spin_unlock(&pmp_lock);
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index");
  }

  let reg_idx = region_register_idx(region_idx);
  unset_bit(region_def_bitmap, region_idx as usize);
  unset_bit(reg_bitmap as usize, reg_idx);
  if region_needs_two_entries(region_idx) != 0 {
    unset_bit(reg_bitmap as usize, reg_idx - 1);
  }
  
  region_clear_all(region_idx);

  spin_unlock(&pmp_lock);

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

pub fn pmp_set_keystone(region_idx: i32, perm: u8) -> i32 {
  if(!is_pmp_region_valid(region_idx))
    PMP_ERROR(SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index");

  uint8_t perm_bits = perm & PMP_ALL_PERM;
  pmpreg_id reg_idx = region_register_idx(region_idx);
  uintptr_t pmpcfg = region_pmpcfg_val(region_idx, reg_idx, perm_bits);
  uintptr_t pmpaddr;

  pmpaddr = region_pmpaddr_val(region_idx);

  //sbi_printf("pmp_set() [hart %d]: reg[%d], mode[%s], range[0x%lx-0x%lx], perm[0x%x]\r\n",
  //       current_hartid(), reg_idx, (region_is_tor(region_idx) ? "TOR":"NAPOT"),
  //       region_get_addr(region_idx), region_get_addr(region_idx) + region_get_size(region_idx), perm);
  //sbi_printf("  pmp[%d] = pmpaddr: 0x%lx, pmpcfg: 0x%lx\r\n", reg_idx, pmpaddr, pmpcfg);

  int n=reg_idx;

  switch(n) {
#define X(n,g) case n: { PMP_SET(n, g, pmpaddr, pmpcfg); break; }
  LIST_OF_PMP_REGS
#undef X
    default:
      sm_assert(FALSE);
  }

  /* TOR decoding with 2 registers */
  if(region_needs_two_entries(region_idx))
  {
    n--;
    pmpcfg = 0;
    pmpaddr = region_get_addr(region_idx) >> 2;
    switch(n) {
#define X(n,g) case n: { PMP_SET(n, g, pmpaddr, pmpcfg); break; }
  LIST_OF_PMP_REGS
#undef X
    default:
      sm_assert(FALSE);
    }
  }
  return SBI_ERR_SM_PMP_SUCCESS;
}

fn region_get_addr(i: region_id ) -> uintptr_t {
  return regions[i].addr;
}

pub fn pmp_region_get_addr(i: region_id) -> uintptr_t {
  if is_pmp_region_valid(i) != 0 {
    return region_get_addr(i);
  }
  return 0;
}

fn region_get_size(i: region_id) -> uint64_t{
  return regions[i].size;
}

pub fn pmp_region_get_size(i: region_id) -> uint64_t{
  if is_pmp_region_valid(i) != 0 {
    return region_get_size(i);
  }
  return 0;
}

fn is_pmp_region_valid(region_idx: region_id) -> i32 {
  return test_bit(region_def_bitmap as usize, region_idx as usize) as i32;
}

pub fn pmp_set_global(region_idx: i32, perm: uint8_t) -> i32 {
  if is_pmp_region_valid(region_idx) == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index");
  }

  send_and_sync_pmp_ipi(region_idx, ERROR::SBI_PMP_IPI_TYPE_SET, perm); // ipi.rs

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

pub fn pmp_unset() -> i32 {
  0
}

pub fn pmp_unset_global(region_idx: i32) -> i32 {
  if is_pmp_region_valid(region_idx) == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index");
  }
  
  send_and_sync_pmp_ipi(region_idx, ERROR::SBI_PMP_IPI_TYPE_UNSET, PMP_NO_PERM);

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

pub fn pmp_detect_region_overlap_atomic() -> i32 {
  0
}

pub fn hand_pmp_ipi() -> i32 {
  0
}

pub fn pmp_region_get_addr() -> i32 {
  0
}

pub fn pmp_region_get_size() -> i32 {
  0
}

