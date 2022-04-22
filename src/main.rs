// mod sm;
// mod thread;
// mod mprv;
// mod pmp;
// mod error_code;
// mod enclave;
// mod def;
// mod sm_sbi;
mod crypt;

use crypt::ed25519::{create_keypair, sign, verify};
use crypt::sha3::sha3_keccakf;

fn main() {
    let mut private_key: [u8; 64] = [0; 64];
    let seed: [u8; 32] = [0; 32];
    let public_key = create_keypair(&mut private_key, &seed);
    let message: [u8; 128] = [0; 128];
    let signature = sign(&message, &public_key, &private_key);
    let flag = verify(&signature, &message, &public_key);
    for i in 0..64 {
        print!("{:x}", signature[i]);
    }
    println!();
    println!("{}", flag);
}
