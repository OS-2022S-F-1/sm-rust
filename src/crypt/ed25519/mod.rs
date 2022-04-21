mod fe;
mod ge;
mod precomp_data;
mod sc;

use ge::scalarmult_base;
use super::sha3::{compute, Sha3Ctx};
use sc::*;

fn load_3(x: &[u8]) -> usize {
    let mut result = x[0] as usize;
    result |= (x[1] as usize) << 8;
    result |= (x[2] as usize) << 16;
    result
}

fn load_4(x: &[u8]) -> usize {
    let mut result = x[0] as usize;
    result |= (x[1] as usize) << 8;
    result |= (x[2] as usize) << 16;
    result |= (x[3] as usize) << 24;
    result
}

pub fn create_keypair(private_key: &mut [u8], seed: &[u8]) -> [u8; 32] {
    assert!(seed.len() >= 32 && private_key.len() >= 64);
    compute(seed, private_key);
    private_key[0] &= 248;
    private_key[31] &= 63;
    private_key[31] |= 64;
    scalarmult_base(private_key).into()
}

pub fn sign(message: &[u8], public_key: &[u8], private_key: &[u8]) -> [u8; 64] {
    let mut signature: [u8; 64] = [0; 64];
    let mut r: [u8; 64] = [0; 64];
    let mut hram: [u8; 64] = [0; 64];

    let mut hash = Sha3Ctx::new(64);
    hash.update(&private_key[32..64]);
    hash.update(message);
    hash.finalize(&mut r);
    reduce(&mut r);
    let base = scalarmult_base(&r);
    let signature_front: [u8; 32] = base.into();
    signature[0..32].copy_from_slice(&signature_front);

    let mut hash = Sha3Ctx::new(64);
    hash.update(&signature[..32]);
    hash.update(&public_key[..32]);
    hash.update(message);
    hash.finalize(&mut hram);
    reduce(&mut hram);
    muladd(&mut signature[32..], &hram, private_key, &r);
    signature
}

