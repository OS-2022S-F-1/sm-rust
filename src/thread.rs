use crate::opensbi;
use crate::sbi_trap;

pub struct thread_state {
  prev_mpp: i32,
  prev_mepc: usize,
  prev_mstatus: usize,
  prev_csrs: csrs,
  prev_state: ctx
}

impl thread_state {
  pub fn new() -> Self {
    Self {
      prev_mpp: 0,
      prev_mepc: 0,
      prev_mstatus: 0,
      prev_csrs: csrs::new(),
      prev_state: ctx::new()
    }
  }
}

pub struct ctx {
  slot: usize,
  ra: usize,
  sp: usize,
  gp: usize,
  tp: usize,
  t0: usize,
  t1: usize,
  t2: usize,
  s0: usize,
  s1: usize,
  a0: usize,
  a1: usize,
  a2: usize,
  a3: usize,
  a4: usize,
  a5: usize,
  a6: usize,
  a7: usize,
  s2: usize,
  s3: usize,
  s4: usize,
  s5: usize,
  s6: usize,
  s7: usize,
  s8: usize,
  s9: usize,
  s10: usize,
  s11: usize,
  t3: usize,
  t4: usize,
  t5: usize,
  t6: usize
}

impl ctx {
  pub fn new() -> Self {
    Self {
      slot: 0,
      ra: 0,
      sp: 0,
      gp: 0,
      tp: 0,
      t0: 0,
      t1: 0,
      t2: 0,
      s0: 0,
      s1: 0,
      a0: 0,
      a1: 0,
      a2: 0,
      a3: 0,
      a4: 0,
      a5: 0,
      a6: 0,
      a7: 0,
      s2: 0,
      s3: 0,
      s4: 0,
      s5: 0,
      s6: 0,
      s7: 0,
      s8: 0,
      s9: 0,
      s10: 0,
      s11: 0,
      t3: 0,
      t4: 0,
      t5: 0,
      t6: 0
    }
  }
}

pub struct csrs {
  sstatus: usize,    //Supervisor status register.
  sedeleg: usize,    //Supervisor exception delegation register.
  sideleg: usize,    //Supervisor interrupt delegation register.
  sie: usize,        //Supervisor interrupt-enable register.
  stvec: usize,      //Supervisor trap handler base address.
  scounteren: usize, //Supervisor counter enable

  /*  Supervisor Trap Handling */
  sscratch: usize,   //Scratch register for supervisor trap handlers.
  sepc: usize,       //Supervisor exception program counter.
  scause: usize,     //Supervisor trap cause.
  //NOTE: This should be stval, toolchain issue?
  sbadaddr: usize,   //Supervisor bad address.
  sip: usize,        //Supervisor interrupt pending.

  /*  Supervisor Protection and Translation */
  satp: usize    //Page-table base register.
}

impl csrs {
  pub fn new() -> Self {
    Self {
      sstatus: 0,
      sedeleg: 0,
      sideleg: 0,
      sie: 0,
      stvec: 0,
      scounteren: 0,
      sscratch: 0,
      sepc: 0,
      scause: 0,
      sbadaddr: 0,
      sip: 0,
      satp: 0
    }
  }
}

pub fn switch_vector_host() { // ?
  extern void _trap_handler();
  csr_write("mtvec", &_trap_handler);
}

pub fn switch_vector_enclave() {
  // opensbi
  extern void trap_vector_enclave();
  csr_write("mtvec", &trap_vector_enclave);
}

pub fn swap_prev_mepc(thread: &mut thread_state, regs: &mut sbi_trap::sbi_trap_regs, current_mepc: usize) {
  let tmp: usize = thread.prev_mepc;
  thread.prev_mepc = current_mepc;
  regs.mepc = tmp;
}

/* Swaps the entire s-mode visible state, general registers and then csrs */
pub fn swap_prev_state(thread: &mut thread_state, regs: &mut sbi_trap::sbi_trap_regs, return_on_resume: usize) {
  let i: i32;

  unsafe {
    let prev: &[usize] = any_as_usize_slice(&thread.prev_state);
    let regs_ptr: &[usize] = any_as_usize_slice(&regs);
    for i in 0..32 {
      let tmp: usize = prev[i];
      prev[i] = regs_ptr[i];
      regs_ptr[i] = tmp;
    }
    
    prev[0] = !return_on_resume;
  }

  swap_prev_smode_csrs(thread);

  return;
}

/* Swaps all s-mode csrs defined in 1.10 standard */
/* TODO: Right now we are only handling the ones that our test
platforms support. Realistically we should have these behind
defines for extensions (ex: N extension)*/
fn swap_prev_smode_csrs(thread: &mut thread_state) {

  let tmp: usize;

  // sstatus
  tmp: usize = thread.prev_csrs.sstatus;
  thread.prev_csrs.sstatus = opensbi::csr_read("sstatus");
  opensbi::csr_write("sstatus", tmp);

  // These only exist with N extension.
  //LOCAL_SWAP_CSR(sedeleg);
  //LOCAL_SWAP_CSR(sideleg);

  // sie
  tmp: usize = thread.prev_csrs.sie;
  thread.prev_csrs.sie = opensbi::csr_read("sie");
  opensbi::csr_write("sie", tmp);

  // stvec
  tmp: usize = thread.prev_csrs.stvec;
  thread.prev_csrs.stvec = opensbi::csr_read("stvec");
  opensbi::csr_write("stvec", tmp);

  // scounteren
  tmp: usize = thread.prev_csrs.scounteren;
  thread.prev_csrs.scounteren = opensbi::csr_read("scounteren");
  opensbi::csr_write("scounteren", tmp);

  // sscratch
  tmp: usize = thread.prev_csrs.sscratch;
  thread.prev_csrs.sscratch = opensbi::csr_read("sscratch");
  opensbi::csr_write("sscratch", tmp);

  // sepc
  tmp: usize = thread.prev_csrs.sepc;
  thread.prev_csrs.sepc = opensbi::csr_read("sepc");
  opensbi::csr_write("sepc", tmp);

  // scause
  tmp: usize = thread.prev_csrs.scause;
  thread.prev_csrs.scause = opensbi::csr_read("scause");
  opensbi::csr_write("scause", tmp);

  // sbadaddr
  tmp: usize = thread.prev_csrs.sbadaddr;
  thread.prev_csrs.sbadaddr = opensbi::csr_read("sbadaddr");
  opensbi::csr_write("sbadaddr", tmp);

  // sip
  tmp: usize = thread.prev_csrs.sip;
  thread.prev_csrs.sip = opensbi::csr_read("sip");
  opensbi::csr_write("sip", tmp);

  // satp
  tmp: usize = thread.prev_csrs.satp;
  thread.prev_csrs.satp = opensbi::csr_read("satp");
  opensbi::csr_write("satp", tmp);

}

unsafe fn any_as_usize_slice<T: Sized>(p: &T) -> &[usize] {
  ::std::slice::from_raw_parts(
      (p as *const T) as *const usize,
      ::std::mem::size_of::<T>(),
  )
}

pub fn clean_state(state: &mut thread_state) {
  let i: i32;
  
  unsafe {
    let prev: &[usize] = any_as_usize_slice(&state.prev_state);
    for i in 0..32 {
      prev[i] = 0;
    }
  }

  state.prev_mpp = -1; // 0x800;
  clean_smode_csrs(state); 
}

fn clean_smode_csrs(state: &mut thread_state){

  state.prev_csrs.sstatus = 0;

  // We can't read these or set these from M-mode?
  state.prev_csrs.sedeleg = 0;
  state.prev_csrs.sideleg = 0;

  state.prev_csrs.sie = 0;
  state.prev_csrs.stvec = 0;
  // For now we take whatever the OS was doing
  state.prev_csrs.scounteren = opensbi::csr_read("scounteren"); // opensbi 函数
  state.prev_csrs.sscratch = 0;
  state.prev_csrs.sepc = 0;
  state.prev_csrs.scause = 0;
  state.prev_csrs.sbadaddr = 0;
  state.prev_csrs.sip = 0;
  state.prev_csrs.satp = 0;

}

pub fn swap_prev_mstatus(thread: &mut thread_state, regs: &mut sbi_trap::sbi_trap_regs, current_mstatus: usize) {
  //Time interrupts can occur in either user mode or supervisor mode
  let mstatus_mask: usize = opensbi::MSTATUS_SIE | opensbi::MSTATUS_SPIE | opensbi::MSTATUS_SPP | opensbi::MSTATUS_MPP | opensbi::MSTATUS_FS | opensbi::MSTATUS_SUM | opensbi::MSTATUS_MXR; // opensbi

  let tmp: usize = thread.prev_mstatus;
  thread.prev_mstatus = (current_mstatus & !mstatus_mask) | (current_mstatus & mstatus_mask);
  regs.mstatus = (current_mstatus & !mstatus_mask) | tmp;
}

