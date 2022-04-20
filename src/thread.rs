struct thread_state {
  prev_mpp: i32,
  prev_mepc: uintptr_t,
  prev_mstatus: uintptr_t,
  prev_csrs: csrs,
  prev_state: ctx
}

fn clean_state(state: *mut thread_state) {
  let i: i32;
  prev: *mut uintptr_t = &state.prev_state as *mut uintptr_t;
  for i in 0..32 {
    prev[i] = 0;
  }

  (*state).prev_mpp = -1; // 0x800;
  clean_smode_csrs(state); // thread.rs
}

fn clean_smode_csrs(state: *mut thread_state){

  (*state).prev_csrs.sstatus = 0;

  // We can't read these or set these from M-mode?
  (*state).prev_csrs.sedeleg = 0;
  (*state).prev_csrs.sideleg = 0;

  (*state).prev_csrs.sie = 0;
  (*state).prev_csrs.stvec = 0;
  // For now we take whatever the OS was doing
  (*state).prev_csrs.scounteren = csr_read(scounteren); // ?
  (*state).prev_csrs.sscratch = 0;
  (*state).prev_csrs.sepc = 0;
  (*state).prev_csrs.scause = 0;
  (*state).prev_csrs.sbadaddr = 0;
  (*state).prev_csrs.sip = 0;
  (*state).prev_csrs.satp = 0;

}