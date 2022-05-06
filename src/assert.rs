use crate::opensbi;

pub fn sm_assert(cond: usize) {
    if cond == 0 {
        println!("[SM] assertion_failed\r\n");
        opensbi::sbi_hart_hang();
    }
}