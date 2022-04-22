mod fe;
mod ge;
mod precomp_data;
mod sc;

use ge::{scalarmult_base, GeP3, double_scalarmult_vartime};
use super::sha3::{compute, Sha3Ctx};
use sc::*;
use fe::Fe;

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

fn print_fe(a: &Fe) {
    a.0.iter().for_each(|i| {print!("{:08x} ", i)});
    println!();
}

pub fn create_keypair(private_key: &mut [u8], seed: &[u8]) -> [u8; 32] {
    assert!(seed.len() >= 32 && private_key.len() >= 64);
    compute(seed, private_key);
    private_key[0] &= 248;
    private_key[31] &= 63;
    private_key[31] |= 64;
    let A = scalarmult_base(private_key);
    A.into()
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

pub fn verify(signature: &[u8], message: &[u8], public_key: &[u8]) -> bool {
    if signature[63] & 224 > 0 {
        return false;
    }

    if let Some(a) = GeP3::frombytes_negate_vartime(&public_key) {
        let mut hram: [u8; 64] = [0; 64];
        let mut hash = Sha3Ctx::new(64);
        hash.update(&signature[..32]);
        hash.update(&public_key[..32]);
        hash.update(message);
        hash.finalize(&mut hram);
        reduce(&mut hram);
        let r = double_scalarmult_vartime(&hram, &a, &signature[32..]);
        let checker: [u8; 32] = r.into();
        for i in 0..32 {
            if signature[i] != checker[i] {
                return false;
            }
        }
        true
    } else {
        false
    }
}

