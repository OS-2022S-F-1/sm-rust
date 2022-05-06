use crate::enclave;
use crate::opensbi;

struct cpu_state {
  is_enclave: i32,
  eid: enclave::enclave_id
}

impl cpu_state {
  pub fn new() -> Self {
    Self {
      is_enclave: 0,
      eid: 0
    }
  }
}

const MAX_HARTS: usize = 16;

static cpus: [cpu_state;MAX_HARTS] = [cpu_state::new();MAX_HARTS];

pub fn cpu_is_enclave_context() -> i32 {
  return cpus[opensbi::csr_read("mhartid")].is_enclave;
}

pub fn cpu_get_enclave_id() -> usize {
  return cpus[opensbi::csr_read("mhartid")].eid;
}

pub fn cpu_enter_enclave_context(eid: enclave::enclave_id) {
  cpus[opensbi::csr_read("mhartid")].is_enclave = 1;
  cpus[opensbi::csr_read("mhartid")].eid = eid;
}

pub fn cpu_exit_enclave_context() {
  cpus[opensbi::csr_read("mhartid")].is_enclave = 0;
}
