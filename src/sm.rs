use crate::crypto;
use crate::pmp;
use crate::error_code::ERROR;
use crate::enclave;
use crate::opensbi;
use crate::platform;

const SMM_BASE: usize = 0x80000000;
const SMM_SIZE: usize = 0x200000;

/* 0-1999 are not used (deprecated) */
const FID_RANGE_DEPRECATED: usize = 1999;
/* 2000-2999 are called by host */
const SBI_SM_CREATE_ENCLAVE: usize = 2001;
const SBI_SM_DESTROY_ENCLAVE: usize = 2002;
const SBI_SM_RUN_ENCLAVE: usize = 2003;
const SBI_SM_RESUME_ENCLAVE: usize = 2005;
const FID_RANGE_HOST: usize = 2999;
/* 3000-3999 are called by enclave */
const SBI_SM_RANDOM: usize = 3001;
const SBI_SM_ATTEST_ENCLAVE: usize = 3002;
const SBI_SM_GET_SEALING_KEY: usize = 3003;
const SBI_SM_STOP_ENCLAVE: usize = 3004;
const SBI_SM_EXIT_ENCLAVE: usize = 3006;
const FID_RANGE_ENCLAVE: usize = 3999;
/* 4000-4999 are experimental */
const SBI_SM_CALL_PLUGIN: usize = 4000;
const FID_RANGE_CUSTOM: usize = 4999;

/* error codes */
const SBI_ERR_SM_ENCLAVE_SUCCESS: usize = 0;
const SBI_ERR_SM_ENCLAVE_UNKNOWN_ERROR: usize = 100000;
const SBI_ERR_SM_ENCLAVE_INVALID_ID: usize = 100001;
const SBI_ERR_SM_ENCLAVE_INTERRUPTED: usize = 100002;
const SBI_ERR_SM_ENCLAVE_PMP_FAILURE: usize = 100003;
const SBI_ERR_SM_ENCLAVE_NOT_RUNNABLE: usize = 100004;
const SBI_ERR_SM_ENCLAVE_NOT_DESTROYABLE: usize = 100005;
const SBI_ERR_SM_ENCLAVE_REGION_OVERLAPS: usize = 100006;
const SBI_ERR_SM_ENCLAVE_NOT_ACCESSIBLE: usize = 100007;
const SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT: usize = 100008;
const SBI_ERR_SM_ENCLAVE_NOT_RUNNING: usize = 100009;
const SBI_ERR_SM_ENCLAVE_NOT_RESUMABLE: usize = 100010;
const SBI_ERR_SM_ENCLAVE_EDGE_CALL_HOST: usize = 100011;
const SBI_ERR_SM_ENCLAVE_NOT_INITIALIZED: usize = 100012;
const SBI_ERR_SM_ENCLAVE_NO_FREE_RESOURCE: usize = 100013;
const SBI_ERR_SM_ENCLAVE_SBI_PROHIBITED: usize = 100014;
const SBI_ERR_SM_ENCLAVE_ILLEGAL_PTE: usize = 100015;
const SBI_ERR_SM_ENCLAVE_NOT_FRESH: usize = 100016;
const SBI_ERR_SM_DEPRECATED: usize = 100099;
const SBI_ERR_SM_NOT_IMPLEMENTED: usize = 100100;

const SBI_ERR_SM_PMP_SUCCESS: usize = 0;
const SBI_ERR_SM_PMP_REGION_SIZE_INVALID: usize = 100020;
const SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY: usize = 100021;
const SBI_ERR_SM_PMP_REGION_NOT_ALIGNED: usize = 100022;
const SBI_ERR_SM_PMP_REGION_MAX_REACHED: usize = 100023;
const SBI_ERR_SM_PMP_REGION_INVALID: usize = 100024;
const SBI_ERR_SM_PMP_REGION_OVERLAP: usize = 100025;
const SBI_ERR_SM_PMP_REGION_IMPOSSIBLE_TOR: usize = 100026;

pub static mut sm_init_done: i32 = 0;
pub static mut sm_region_id: i32 = 0; 
pub static mut os_region_id: i32 = 0;

pub static mut sm_hash: [u8;crypto::MDSIZE] = [0;crypto::MDSIZE];
pub static mut sm_signature: [u8;crypto::SIGNATURE_SIZE] = [0;crypto::SIGNATURE_SIZE];
pub static mut sm_public_key: [u8;crypto::PUBLIC_KEY_SIZE] = [0;crypto::PUBLIC_KEY_SIZE];
pub static mut sm_private_key: [u8;crypto::PRIVATE_KEY_SIZE] = [0;crypto::PRIVATE_KEY_SIZE];
pub static mut dev_public_key: [u8;crypto::PUBLIC_KEY_SIZE] = [0;crypto::PUBLIC_KEY_SIZE];

// /* from Sanctum BootROM */
// extern sanctum_sm_hash[u8;MDSIZE];
// extern sanctum_sm_signature[u8;SIGNATURE_SIZE];
// extern sanctum_sm_secret_key[u8;PRIVATE_KEY_SIZE];
// extern sanctum_sm_public_key[u8;PUBLIC_KEY_SIZE];
// extern sanctum_dev_public_key[u8;PUBLIC_KEY_SIZE];

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
  ::std::slice::from_raw_parts(
      (p as *const T) as *const u8,
      ::std::mem::size_of::<T>(),
  )
}

struct keystone_sbi_pregion { // physical memory
  pub paddr: u32,
  pub size: u32
}

pub struct runtime_pa_params {
  pub dram_base: usize,
  pub dram_size: usize, 
  pub runtime_base: usize,
  pub user_base: usize,
  pub free_base: usize 
}

impl runtime_pa_params {
  pub fn new() -> Self {
    Self {
      dram_base: 0,
      dram_size: 0,
      runtime_base: 0,
      user_base: 0,
      free_base: 0
    }
  }
} 

pub struct runtime_va_params_t {
  pub runtime_entry: u32,
  pub user_entry: u32,
  pub untrusted_ptr: u32,
  pub untrusted_size: u32
}

impl runtime_va_params_t {
  pub fn new() -> Self {
    Self {
      runtime_entry: 0,
      user_entry: 0,
      untrusted_ptr: 0,
      untrusted_size: 0
    }
  }
} 

pub struct keystone_sbi_create {
  pub epm_region: keystone_sbi_pregion, // enclave private memory
  pub utm_region: keystone_sbi_pregion, // untrusted shared pages

  pub runtime_paddr: u32,
  pub user_paddr: u32,
  pub free_paddr: u32,

  pub params: runtime_va_params_t,
  pub eid_pptr: *mut u32
}

pub fn osm_pmp_set(perm: u8) -> i32{
  /* in case of OSM, PMP cfg is exactly the opposite.*/
  return pmp::pmp_set_keystone(os_region_id, perm);
}

pub fn smm_init() -> i32 {
  let region: i32 = -1;
  let ret: i32 = pmp::pmp_region_init_atomic(SMM_BASE, SMM_SIZE, pmp::pmp_priority::PMP_PRI_TOP, &mut region, 0);
  if ret != 0 {
    return -1;
  }

  return region;
}

pub fn osm_init() -> i32 {
  let region: i32 = -1;
  let ret: i32 = pmp::pmp_region_init_atomic(0, usize::MAX, pmp::pmp_priority::PMP_PRI_BOTTOM, &mut region, 1);
  if ret != 0 {
    return -1;
  }
  return region;
}

pub fn sm_derive_sealing_key(key: &mut [u8], key_ident: &[u8], key_ident_size: usize, enclave_hash: &[u8]) -> i32 {

  let info: usize;

  // opensbi ??????
  opensbi::sbi_memcpy(info, enclave_hash.as_ptr() as usize, crypto::MDSIZE);
  opensbi::sbi_memcpy(info + crypto::MDSIZE, key_ident.as_ptr() as usize, key_ident_size);

  /*
  * The key is derived without a salt because we have no entropy source
  * available to generate the salt.
  */
  unsafe {
    let info_ptr: &[u8] = any_as_u8_slice(&info);
    return crypto::kdf(&mut [0], &sm_private_key, info_ptr, key);
  }
}

pub fn sm_sign(signature: &[u8], data: &[u8], len: usize) {
  crypto::sign(data, &sm_public_key, &sm_private_key);
}

fn sm_copy_key() {
  opensbi::sbi_memcpy(&sm_hash as *const u8 as usize, sanctum_sm_hash, crypto::MDSIZE);
  opensbi::sbi_memcpy(&sm_signature as *const u8 as usize, sanctum_sm_signature, crypto::SIGNATURE_SIZE);
  opensbi::sbi_memcpy(&sm_public_key as *const u8 as usize, sanctum_sm_public_key, crypto::PUBLIC_KEY_SIZE);
  opensbi::sbi_memcpy(&sm_private_key as *const u8 as usize, sanctum_sm_secret_key, crypto::PRIVATE_KEY_SIZE);
  opensbi::sbi_memcpy(&dev_public_key as *const u8 as usize, sanctum_dev_public_key, crypto::PUBLIC_KEY_SIZE);
}

fn sm_print_hash() {
  for i in 0..crypto::MDSIZE {
    // opensbi
    println!("%{}", sm_hash[i]);
  }
  println!("\n");
}

fn sm_init(cold_boot: bool) {
	// initialize SM
  if cold_boot {
    /* only the cold-booting hart will execute these */
    // opensbi
    println!("[SM] Initializing ... hart {}\n", opensbi::csr_read("mhartid"));

    // opensbi
    // lib: Add dynamic registration of SBI extensions

    // This patch extends our SBI ecall implementation to allow
    // dynamic registration of various SBI extensions. Using this
    // dynamic registration we can break-up SBI ecall implementation
    // into multiple files and even register experimental/custom
    // SBI extensions from platform code.

    // Signed-off-by: Anup Patel <anup.patel@wdc.com>
    // Reviewed-by: Atish Patra <atish.patra@wdc.com>
    // opensbi::sbi_ecall_register_extension(&ecall_keystone_enclave);

    sm_region_id = smm_init();
    if sm_region_id < 0 {
      // opensbi
      println!("[SM] intolerable error - failed to initialize SM memory");
      opensbi::sbi_hart_hang();
    }

    os_region_id = osm_init();
    if os_region_id < 0 {
      // opensbi
      println!("[SM] intolerable error - failed to initialize OS memory");
      opensbi::sbi_hart_hang();
    }

    if platform::platform_init_global_once() != SBI_ERR_SM_ENCLAVE_SUCCESS {
      println!("[SM] platform global init fatal error");
      opensbi::sbi_hart_hang();
    }
    // Copy the keypair from the root of trust
    sm_copy_key();

    // Init the enclave metadata
    enclave::enclave_init_metadata(); // enclave.rs

    sm_init_done = 1;
    opensbi::mb(); // opensbi
  }

  /* wait until cold-boot hart finishes */
  while sm_init_done == 0 {
    opensbi::mb(); // opensbi
  }

  /* below are executed by all harts */
  pmp::pmp_init();
  pmp::pmp_set_keystone(sm_region_id, pmp::PMP_NO_PERM);
  pmp::pmp_set_keystone(os_region_id, pmp::PMP_ALL_PERM);

  /* Fire platform specific global init */
  if platform::platform_init_global() != ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS {
    // opensbi
    println!("[SM] platform global init fatal error");
    opensbi::sbi_hart_hang();
  }

  // opensbi
  println!("[SM] Keystone security monitor has been initialized!\n");

  sm_print_hash();

  return;
  // for debug
  // sm_print_cert();
}
