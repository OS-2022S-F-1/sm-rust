use super::hmac_sha3::{hmac_sha3, HmacSha3Ctx, SHA3_512_HASH_LEN};

pub fn hkdf_sha3_512(mut salt: &[u8], ikm: &[u8], info: &[u8], mut okm: &[u8]) -> isize {
    if okm.len() > 255 * SHA3_512_HASH_LEN {
        -1
    } else {
        let mut prk: [u8; SHA3_512_HASH_LEN] = [0; SHA3_512_HASH_LEN];
        hmac_sha3(&mut salt, &ikm, &mut prk);
        hkdf_expand(&mut prk, &info, &mut okm)
    }

}

fn hkdf_expand(prk: &mut [u8], info: &[u8], okm: &mut [u8]) -> isize {
    if prk.len() < SHA3_512_HASH_LEN || okm.len() > 255 * SHA3_512_HASH_LEN {
        -1
    } else {
        let n = (okm.len() + SHA3_512_HASH_LEN - 1) / SHA3_512_HASH_LEN;
        let mut t:[u8; SHA3_512_HASH_LEN] = [0; SHA3_512_HASH_LEN];
        // Compute T(1) - T(n) and copy resulting key to okm
        for i in 1..(n + 1) {
            let mut ctx = HmacSha3Ctx::new(&prk);
            if i > 1 {
                ctx.update(&t);
            }
            let temp: [u8; 1] = [i as u8];
            ctx.update(&info);
            ctx.update(&temp);
            ctx.finalize(&mut t);

            if i < n {
                okm[(i - 1) * SHA3_512_HASH_LEN..i * SHA3_512_HASH_LEN].copy_from_slice(&t);
            } else {
                okm[(i - 1) * SHA3_512_HASH_LEN..].copy_from_slice(&t);
            }
        }
        0
    }

}
