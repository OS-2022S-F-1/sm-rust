struct keystone_sbi_pregion { // physical memory
  paddr: u32,
  size: u32
}

struct runtime_va_params_t {
  runtime_entry: u32,
  user_entry: u32,
  untrusted_ptr: u32,
  untrusted_size: u32
}

struct keystone_sbi_create {
  epm_region: keystone_sbi_pregion, // enclave private memory
  utm_region: keystone_sbi_pregion, // untrusted shared pages

  runtime_paddr: u32,
  user_paddr: u32,
  free_paddr: u32,

  params: runtime_va_params_t,
  eid_pptr: *mut u32
}