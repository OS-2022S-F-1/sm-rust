use crate::crypt::hkdf_sha3;
use crate::crypt::ed25519;
use crate::crypt::sha3;
use crate::page;

pub const MDSIZE: usize = 64;
pub const PUBLIC_KEY_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;
pub const PRIVATE_KEY_SIZE: usize = 64;

pub type hash_ctx = sha3::Sha3Ctx;

pub fn hash_init(hash_ctx: &mut hash_ctx) {
  hash_ctx = &mut sha3::Sha3Ctx::new(MDSIZE);
}

pub fn hash_extend(hash_ctx: &mut hash_ctx, data: &[u8], len: usize) {
  hash_ctx.update(&data[0..len]);
}

pub fn hash_extend_page(hash_ctx: &mut hash_ctx, data: &[u8]) {
  hash_ctx.update(&data[0..page::RISCV_PGSIZE]);
}

pub fn hash_finalize(md: &mut [u8], hash_ctx: &mut hash_ctx) {
  hash_ctx.finalize(md);
}

pub fn sign(data: &[u8], public_key: &[u8], private_key: &[u8]) -> [u8; 64] {
    return ed25519::sign(data, public_key, private_key);
}

pub fn kdf(salt: &mut [u8], ikm: &[u8], info: &[u8], okm: &mut [u8]) -> i32 {
    return hkdf_sha3::hkdf_sha3_512(salt, ikm, info, okm) as i32;
}