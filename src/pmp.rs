use crate::error_code::ERROR;
use crate::ipi;
use core::arch::asm;
use crate::assert;
use crate::opensbi;
use crate::page;

pub type region_id = i32;
pub type pmpreg_id = i32;

#[derive(PartialEq)]
pub enum pmp_priority {
  PMP_PRI_ANY,
  PMP_PRI_TOP,
  PMP_PRI_BOTTOM
}

struct ipi_msg {
  pending: opensbi::atomic_t, // opensbi
  perm: u8
}

struct pmp_region {
  size: u64,
  addrmode: u8,
  addr: u32,
  allow_overlap: i32,
  reg_idx: i32
}

impl pmp_region {
  pub fn new() -> Self {
    Self {
      size: 0,
      addrmode: 0,
      addr: 0,
      allow_overlap: 0,
      reg_idx: 0
    }
  }
}

pub const PMP_N_REG: usize = 8; //number of PMP registers
pub const PMP_MAX_N_REGION: usize = 16; //maximum number of PMP regions

pub const PMP_ALL_PERM: u8 = opensbi::PMP_W | opensbi::PMP_X | opensbi::PMP_R; // opensbi
pub const PMP_NO_PERM: u8 = 0;

/* PMP global spin locks */
static pmp_lock: opensbi::spinlock_t = opensbi::SPIN_LOCK_INITIALIZER; // opensbi

/* PMP region getter/setters */
static regions: [pmp_region;PMP_MAX_N_REGION] = [pmp_region::new();PMP_MAX_N_REGION];
static reg_bitmap: u32 = 0;
static region_def_bitmap: u32 = 0;

#[cfg(target_pointer_width = "32")]
pub mod PMP {
  pub const LIST_OF_PMP_REGS: [usize;16] = [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3];
  pub const PMP_PER_GROUP: usize = 4;
}

#[cfg(target_pointer_width = "64")]
pub mod PMP {
  pub const LIST_OF_PMP_REGS: [usize;16] = [0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2];
  pub const  PMP_PER_GROUP: usize = 8;
}

fn set_bit(bitmap: usize, n: usize) {
  bitmap = bitmap | (0x1 << (n));
}

fn unset_bit(bitmap: usize, n: usize) {
  bitmap = bitmap & !(0x1 << (n));
}

fn test_bit(bitmap: usize, n: usize) -> usize {
  bitmap & (0x1 << (n))
}

fn PMP_SET(n: usize, g: usize, addr: usize, pmpc: &mut usize) {
  let pmpaddr_n: String = "pmpaddr".to_owned();
  pmpaddr_n.push_str(&n.to_string());

  let pmpcfg_n: String = "pmpcfg".to_owned();
  pmpcfg_n.push_str(&n.to_string());

  let pmpcfg_g: String = "pmpcfg".to_owned();
  pmpcfg_g.push_str(&g.to_string());

  let oldcfg: usize = opensbi::csr_read(&pmpcfg_g); 
  *pmpc = oldcfg & !((0xff as usize) << 8 * (n % PMP::PMP_PER_GROUP));
  let inst1 = format!("csrw {}, %0;", pmpaddr_n);
  let inst2 = format!("csrw {}, %1", pmpcfg_g);
  unsafe {
    asm!(
      "la t0, 1f",
      "csrrw t0, mtvec, t0",
      format!("csrw {}, {0}", pmpaddr_n), 
      format!("csrw {}, {1}", pmpcfg_g),
      "sfence.vma",
      ".align 2", 
      "1: csrw mtvec, t0",
      in(reg) addr, 
      in(reg) pmpc,
      out("t0") _,
    );
  }
  
}

fn PMP_UNSET(n: usize, g: usize) {
  let pmpaddr_n: String = "pmpaddr".to_owned();
  pmpaddr_n.push_str(&n.to_string());

  let pmpcfg_n: String = "pmpcfg".to_owned();
  pmpcfg_n.push_str(&n.to_string());

  let pmpcfg_g: String = "pmpcfg".to_owned();
  pmpcfg_g.push_str(&g.to_string());

  let pmpc: usize = opensbi::csr_read(&pmpcfg_g); 
  pmpc &= !((0xff as usize) << 8 * (n % PMP::PMP_PER_GROUP));
  unsafe {
    asm!(
      "la t0, 1f",
      "csrrw t0, mtvec, t0",
      format!("csrw {}, {0}", pmpaddr_n), 
      format!("csrw {}, {1}", pmpcfg_g),
      "sfence.vma",
      ".align 2", 
      "1: csrw mtvec, t0",
      in(reg) 0, 
      in(reg) pmpc,
      out("t0") _,
    );
  }
}

fn PMP_ERROR(error: usize, msg: String) -> usize {
  println!("{}:\n", msg); // opensbi
  return error;
}

fn get_free_reg_idx() -> pmpreg_id {
  return search_rightmost_unset(reg_bitmap as usize, PMP_N_REG, 0x1);
}

fn get_conseq_free_reg_idx() -> pmpreg_id {
  return search_rightmost_unset(reg_bitmap as usize, PMP_N_REG, 0x3);
}

fn get_free_region_idx() -> region_id {
  return search_rightmost_unset(region_def_bitmap as usize, PMP_MAX_N_REGION, 0x1);
}

fn search_rightmost_unset(bitmap: usize, max: usize, mask: usize) -> i32 {
  let i: i32 = 0;

  assert::sm_assert((max < 32) as usize);
  assert::sm_assert(!((mask + 1) & mask));

  while mask < (usize::MAX << max) {
    if (!bitmap & mask) == mask {
      return i;
    }
    mask = mask << 1;
    i += 1;
  }
  return -1;
}

fn region_pmpcfg_val(i: region_id, reg_idx: pmpreg_id, perm_bits: u8) -> usize {
  return ((regions[i as usize].addrmode | perm_bits) as usize) << 8 * (reg_idx as usize % PMP::PMP_PER_GROUP);
}

fn region_clear_all(i: region_id) {
  regions[i as usize].addr = 0;
  regions[i as usize].size = 0;
  regions[i as usize].addrmode = 0;
  regions[i as usize].allow_overlap = 0;
  regions[i as usize].reg_idx = 0;
}

pub fn pmp_init() {
    let pmpaddr: u32 = 0;
    let pmpcfg: u32 = 0;
    let mut i = 0;
}

pub fn pmp_region_init(start: usize, size: usize, priority: pmp_priority, rid: &mut i32, allow_overlap: i32) -> i32 {
  if size == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_SIZE_INVALID, "Invalid PMP size".to_string());
  }

  /* overlap detection */
  if allow_overlap == 0 {
    if detect_region_overlap(start, size) != 0 {
      return ERROR::SBI_ERR_SM_PMP_REGION_OVERLAP as i32;
    }
  }

  /* PMP granularity check */
  if (size != usize::MAX) && (size & (page::RISCV_PGSIZE - 1) != 0) {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY, "PMP granularity is RISCV_PGSIZE".to_string());
  }
  if (start & (page::RISCV_PGSIZE - 1)) != 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY, "PMP granularity is RISCV_PGSIZE".to_string());
  }

  /* if the address covers the entire RAM or it's NAPOT */
  if (size == usize::MAX && start == 0) || ((size & (size - 1)) == 0 && (start & (size - 1)) == 0) {
    return napot_region_init(start, size, priority, rid, allow_overlap) as i32;
  }
  else {
    if (priority != pmp_priority::PMP_PRI_ANY) && (priority != pmp_priority::PMP_PRI_TOP || start != 0) {
      PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_IMPOSSIBLE_TOR, "The top-priority TOR PMP entry must start from address 0".to_string());
    }

    return tor_region_init(start, size, priority, rid, allow_overlap);
  }
}

/* We do an integery overflow safety check here for the inputs (addr +
 * size).  We do NOT do a safety check on epm_base + epm_size, since
 * only valid region should have been created previously.
 *
 * On a failed addr + size overflow, we return failure, since this
 * cannot be a valid addr and size anyway.
 */
fn detect_region_overlap(addr: usize, size: usize) -> i32 {
  let epm_base: u32;
  let epm_size: usize;
  let region_overlap: i32 = 0;

  
  // Safety check the addr+size
  let input_end: usize;

  match addr.checked_add(size) {
    Some(num) => input_end = num,
    None => return 1
  }
  // if CHECKED_ADD(addr, size, &input_end) { // detect overflow
  //   return 1;
  // }

  for i in 0..PMP_MAX_N_REGION {
    if is_pmp_region_valid(i as i32) == 0 {
      continue;
    }

    if region_allows_overlap(i as i32) == 0 {
      continue;
    }

    epm_base = region_get_addr(i as i32);
    epm_size = region_get_size(i as i32) as usize;

    // Only looking at valid regions, no need to check epm_base+size
    region_overlap |= ((epm_base < input_end as u32) &&
                      (epm_base as usize + epm_size > addr)) as i32; // TODO
  }

  return region_overlap;
}

fn tor_region_init(start: usize, size: usize, priority: pmp_priority, rid: &mut region_id, allow_overlap: i32) -> i32 {
  let reg_idx: pmpreg_id = -1;
  let region_idx: region_id = -1;

  assert::sm_assert(size as usize);
  assert::sm_assert(!(size as usize & (page::RISCV_PGSIZE - 1)));
  assert::sm_assert(!(start & (page::RISCV_PGSIZE - 1)));
  assert::sm_assert(*rid as usize);
  assert::sm_assert((priority != pmp_priority::PMP_PRI_BOTTOM) as usize);

  region_idx = get_free_region_idx();
  if region_idx < 0 || region_idx > PMP_MAX_N_REGION as i32 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "Reached the maximum number of PMP regions".to_string());
  }

  *rid = region_idx;
  match priority {
    pmp_priority::PMP_PRI_ANY => {
      reg_idx = get_conseq_free_reg_idx();
      if reg_idx < 0 {
        PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "No available PMP register".to_string());
      }
      if test_bit(reg_bitmap as usize, reg_idx as usize) != 0 || test_bit(reg_bitmap as usize, reg_idx as usize + 1) != 0 || reg_idx + 1 >= PMP_N_REG as i32 {
        PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "PMP register unavailable".to_string());
      }
    }
    pmp_priority::PMP_PRI_TOP => {
      assert::sm_assert((start == 0) as usize);
      reg_idx = 0;
      if test_bit(reg_bitmap as usize, reg_idx as usize) != 0 {
        PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "PMP register unavailable".to_string());
      }
    }
    _ => {
      assert::sm_assert(0);
    }
  }

  // initialize the region
  region_init(region_idx, start, size, opensbi::PMP_A_TOR, allow_overlap, reg_idx); // opensbi
  set_bit(region_def_bitmap as usize, region_idx as usize);
  set_bit(reg_bitmap as usize, reg_idx as usize);

  if reg_idx > 0 {
    set_bit(reg_bitmap as usize, reg_idx as usize + 1);
  }

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

fn region_init(i: region_id, addr: usize, size: usize, addrmode: u8, allow_overlap: i32, reg_idx: pmpreg_id) {
  regions[i as usize].addr = addr as u32;
  regions[i as usize].size = size as u64;
  regions[i as usize].addrmode = addrmode;
  regions[i as usize].allow_overlap = allow_overlap;
  if addrmode == opensbi::PMP_A_TOR && reg_idx > 0 {
    regions[i as usize].reg_idx = reg_idx + 1;
  }
  else {
    regions[i as usize].reg_idx = reg_idx;
  }
}

fn napot_region_init(start: usize, size: usize, priority: pmp_priority, rid: &mut region_id, allow_overlap: i32) -> usize {
  let reg_idx: pmpreg_id = -1;
  let region_idx: region_id = -1;

  assert::sm_assert(size as usize); // assert.rs
  assert::sm_assert(*rid as usize);

  if !(size == (2 ^ 32 - 1) && start == 0) {
    assert::sm_assert(!(size & (size - 1)) as usize);
    assert::sm_assert(!(start & (size - 1)));
    assert::sm_assert(!(size & (page::RISCV_PGSIZE - 1)) as usize);
    assert::sm_assert(!(start & (page::RISCV_PGSIZE - 1)));
  }

  //find avaiable pmp region idx
  region_idx = get_free_region_idx();
  if region_idx < 0 || region_idx > PMP_MAX_N_REGION as i32 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "Reached the maximum number of PMP regions".to_string());
  }

  *rid = region_idx;

  match priority {
    pmp_priority::PMP_PRI_ANY => {
      reg_idx = get_free_reg_idx();
      if reg_idx < 0 {
        PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "No available PMP register".to_string());
      }
      if test_bit(reg_bitmap as usize, reg_idx as usize) != 0 || reg_idx >= PMP_N_REG as i32 {
        PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "PMP register unavailable".to_string());
      }
    },
    pmp_priority::PMP_PRI_TOP => {
      reg_idx = 0;
      if test_bit(reg_bitmap as usize, reg_idx as usize) != 0 {
        PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_MAX_REACHED, "PMP register unavailable".to_string());
      }
    },
    pmp_priority::PMP_PRI_BOTTOM => {
      /* the bottom register can be used by multiple regions,
       * so we don't check its availability */
      reg_idx = (PMP_N_REG - 1) as i32;
    },
    _ => {
      assert::sm_assert(0);
    }
  }

  // initialize the region
  region_init(region_idx, start, size, opensbi::PMP_A_NAPOT, allow_overlap, reg_idx);
  set_bit(region_def_bitmap as usize, region_idx as usize);
  set_bit(reg_bitmap as usize, reg_idx as usize);

  return ERROR::SBI_ERR_SM_PMP_SUCCESS;
}

fn region_is_tor(i: region_id) -> bool {
  return regions[i as usize].addrmode == opensbi::PMP_A_TOR;
}

fn region_needs_two_entries(i: region_id) -> i32 {
  return (region_is_tor(i) && regions[i as usize].reg_idx > 0) as i32;
}

fn region_is_napot_all(i: region_id) -> bool {
  return regions[i as usize].addr == 0 && regions[i as usize].size as usize == usize::MAX;
}

fn region_is_napot(i: region_id) -> bool {
  return regions[i as usize].addrmode == opensbi::PMP_A_NAPOT;
}

fn region_pmpaddr_val(i: region_id) -> usize {
  if region_is_napot_all(i) {
    return usize::MAX;
  }
  else if region_is_napot(i) {
    return (regions[i as usize].addr as usize | (regions[i as usize].size / 2 - 1) as usize) >> 2;
  }
  else if region_is_tor(i) {
    return (regions[i as usize].addr as usize + regions[i as usize].size as usize) >> 2;
  }
  else {
    return 0;
  }
}

pub fn pmp_region_init_atomic(start: usize, size: usize, priority: pmp_priority, rid: &mut region_id, allow_overlap: i32) -> i32 {
  let ret: i32;
  opensbi::spin_lock(&pmp_lock);
  ret = pmp_region_init(start, size, priority, rid, allow_overlap); // pmp.rs
  opensbi::spin_unlock(&pmp_lock);
  return ret;
}

pub fn pmp_region_free_atomic(region_idx: i32) -> i32{
  
  opensbi::spin_lock(&pmp_lock);

  if is_pmp_region_valid(region_idx) == 0 {
    opensbi::spin_unlock(&pmp_lock);
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index".to_string());
  }

  let reg_idx = region_register_idx(region_idx);
  unset_bit(region_def_bitmap as usize, region_idx as usize);
  unset_bit(reg_bitmap as usize, reg_idx as usize);
  if region_needs_two_entries(region_idx) != 0 {
    unset_bit(reg_bitmap as usize, reg_idx as usize - 1);
  }
  
  region_clear_all(region_idx);

  opensbi::spin_unlock(&pmp_lock);

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

pub fn pmp_set_keystone(region_idx: i32, perm: u8) -> i32 {
  if is_pmp_region_valid(region_idx) == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index".to_string());
  }

  let perm_bits: u8 = perm & PMP_ALL_PERM;
  let reg_idx: pmpreg_id = region_register_idx(region_idx);
  let pmpcfg: usize = region_pmpcfg_val(region_idx, reg_idx, perm_bits);
  let pmpaddr: usize = region_pmpaddr_val(region_idx);

  //sbi_printf("pmp_set() [hart %d]: reg[%d], mode[%s], range[0x%lx-0x%lx], perm[0x%x]\r\n",
  //       current_hartid(), reg_idx, (region_is_tor(region_idx) ? "TOR":"NAPOT"),
  //       region_get_addr(region_idx), region_get_addr(region_idx) + region_get_size(region_idx), perm);
  //sbi_printf("  pmp[%d] = pmpaddr: 0x%lx, pmpcfg: 0x%lx\r\n", reg_idx, pmpaddr, pmpcfg);

  let n: i32 = reg_idx;
  if n >= 0 && n < 16 {
    PMP_SET(n as usize, PMP::LIST_OF_PMP_REGS[n as usize], pmpaddr, &mut pmpcfg)
  }
  else {
    assert::sm_assert(0);
  }
  
  /* TOR decoding with 2 registers */
  if region_needs_two_entries(region_idx) != 0 {
    n -= 1;
    pmpcfg = 0;
    pmpaddr = (region_get_addr(region_idx) >> 2) as usize;

    if n >= 0 && n < 16 {
      PMP_SET(n as usize, PMP::LIST_OF_PMP_REGS[n as usize], pmpaddr, &mut pmpcfg)
    }
    else {
      assert::sm_assert(0);
    }
  }
  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

fn region_get_addr(i: region_id) -> u32 {
  return regions[i as usize].addr;
}

pub fn pmp_region_get_addr(i: region_id) -> u32 {
  if is_pmp_region_valid(i) != 0 {
    return region_get_addr(i);
  }
  return 0;
}

fn region_get_size(i: region_id) -> u64 {
  return regions[i as usize].size;
}

pub fn pmp_region_get_size(i: region_id) -> u64 {
  if is_pmp_region_valid(i) != 0 {
    return region_get_size(i);
  }
  return 0;
}

fn is_pmp_region_valid(region_idx: region_id) -> i32 {
  return test_bit(region_def_bitmap as usize, region_idx as usize) as i32;
}

fn region_allows_overlap(i: region_id) -> i32 {
  return regions[i as usize].allow_overlap;
}

pub fn pmp_set_global(region_idx: i32, perm: u8) -> i32 {
  if is_pmp_region_valid(region_idx) == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index".to_string());
  }

  ipi::send_and_sync_pmp_ipi(region_idx, ipi::SBI_PMP_IPI_TYPE_SET as i32, perm); // ipi.rs

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

fn region_register_idx(i: region_id) -> i32 {
  return regions[i as usize].reg_idx;
}

pub fn pmp_unset(region_idx: i32) -> usize {
  if is_pmp_region_valid(region_idx) == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index".to_string());
  }
  let reg_idx: pmpreg_id = region_register_idx(region_idx);
  let n: i32 = reg_idx;

  if n >= 0 && n < 16 {
    PMP_UNSET(n as usize, PMP::LIST_OF_PMP_REGS[n as usize]);
  }
  else {
    assert::sm_assert(0);
  }

  if region_needs_two_entries(region_idx) != 0 {
    n -= 1;
    if n >= 0 && n < 16 {
      PMP_UNSET(n as usize, PMP::LIST_OF_PMP_REGS[n as usize]);
    }
    else {
      assert::sm_assert(0);
    }
  }

  return ERROR::SBI_ERR_SM_PMP_SUCCESS;
}

pub fn pmp_unset_global(region_idx: i32) -> i32 {
  if is_pmp_region_valid(region_idx) == 0 {
    PMP_ERROR(ERROR::SBI_ERR_SM_PMP_REGION_INVALID, "Invalid PMP region index".to_string());
  }
  
  ipi::send_and_sync_pmp_ipi(region_idx, ipi::SBI_PMP_IPI_TYPE_UNSET as i32, PMP_NO_PERM);

  return ERROR::SBI_ERR_SM_PMP_SUCCESS as i32;
}

pub fn pmp_detect_region_overlap_atomic(addr: usize, size: usize) -> i32 {
  let region_overlap: i32 = 0;
  opensbi::spin_lock(&pmp_lock); // opensbi
  region_overlap = detect_region_overlap(addr, size);
  opensbi::spin_unlock(&pmp_lock);
  return region_overlap;
}



