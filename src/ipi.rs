pub fn send_and_sync_pmp_ipi(region_idx: i32, type: i32, perm: uint8_t) {
  mask: ulong = 0;
  source_hart: ulong = current_hartid();
  struct sbi_tlb_info tlb_info;
  sbi_hsm_hart_started_mask(sbi_domain_thishart_ptr(), 0, &mask);

  SBI_TLB_INFO_INIT(&tlb_info, type, 0, region_idx, perm,
      sbi_pmp_ipi_local_update, source_hart);
  sbi_tlb_request(mask, 0, &tlb_info);
}