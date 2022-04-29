pub fn sm_assert(cond: usize) {
    if cond == 0 {
        sbi_printf("[SM] assertion_failed\r\n");
        sbi_hart_hang();
    }
}