use crate::pmp;
use crate::opensbi;
use crate::sbi_trap;

pub const SBI_PMP_IPI_TYPE_SET: usize = 0;
pub const SBI_PMP_IPI_TYPE_UNSET: usize = 1;

struct sbi_pmp_ipi_info {
  info_type: usize,
  __dummy: usize,
  rid: usize,
  perm: usize
}

// opensbi
pub fn sbi_pmp_ipi_local_update(__info: *mut opensbi::sbi_tlb_info) {
  unsafe {
    let info: *mut sbi_pmp_ipi_info = __info as *mut sbi_pmp_ipi_info;
    if (*info).info_type == SBI_PMP_IPI_TYPE_SET {
      pmp::pmp_set_keystone((*info).rid as i32, (*info).perm as u8);
    }
    else {
      pmp::pmp_unset((*info).rid as i32);
    }
  }
}

pub fn send_and_sync_pmp_ipi(region_idx: i32, ipi_type: i32, perm: u8) {
  let mask: usize = 0;
  let source_hart: usize = opensbi::current_hartid(); // opensbi
  let tlb_info: opensbi::sbi_tlb_info; // opensbi
  sbi_hsm_hart_started_mask(opensbi::sbi_domain_thishart_ptr(), 0, &mask); // opensbi

  tlb_info.start = ipi_type;
  tlb_info.size = 0;
  tlb_info.asid = region_idx;
  tlb_info.vmid = perm;
  tlb_info.local_fn = sbi_pmp_ipi_local_update;
  SBI_HARTMASK_INIT_EXCEPT(tlb_info.smask, source_hart);

  // SBI_TLB_INFO_INIT(&tlb_info, ipi_type, 0, region_idx, perm, // opensbi
  //     sbi_pmp_ipi_local_update, source_hart);
  sbi_tlb_request(mask, 0, &tlb_info);
}
