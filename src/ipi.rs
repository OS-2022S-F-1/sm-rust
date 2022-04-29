use crate::pmp;

pub const SBI_PMP_IPI_TYPE_SET: usize = 0;
pub const SBI_PMP_IPI_TYPE_UNSET: usize = 1;

struct sbi_pmp_ipi_info {
  info_type: usize,
  __dummy: usize,
  rid: usize,
  perm: usize
}

// opensbi
pub fn sbi_pmp_ipi_local_update(info: &mut sbi_tlb_info) {
  if info.info_type == SBI_PMP_IPI_TYPE_SET {
    pmp::pmp_set_keystone(info.rid, info.perm);
  } else {
    pmp::pmp_unset(info.rid);
  }
}

pub fn send_and_sync_pmp_ipi(region_idx: i32, ipi_type: i32, perm: u8) {
  let mask: usize = 0;
  let source_hart: usize = current_hartid(); // opensbi
  let tlb_info: sbi_tlb_info; // opensbi
  sbi_hsm_hart_started_mask(sbi_domain_thishart_ptr(), 0, &mask); // ?

  SBI_TLB_INFO_INIT(&tlb_info, ipi_type, 0, region_idx, perm, // opensbi
      sbi_pmp_ipi_local_update, source_hart);
  sbi_tlb_request(mask, 0, &tlb_info);
}
