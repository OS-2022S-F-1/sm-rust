use super::sha3::Sha3Ctx;

// Internal block length of sha3_512 in bytes
pub const SHA3_512_BLOCK_LEN: usize = 72;
// Output hash length of sha3_512 in bytes
pub const SHA3_512_HASH_LEN: usize = 64;

pub struct HmacSha3Ctx {
    sha3_ctx: Sha3Ctx,
    key: [u8; SHA3_512_BLOCK_LEN]
}

fn prepare_key(key: &[u8], new_key: & mut[u8])
{
    if key.len() > SHA3_512_BLOCK_LEN {
        let mut ctx = Sha3Ctx::new(SHA3_512_HASH_LEN);
        ctx.update(key);
        ctx.finalize(new_key);
    } else {
        new_key.copy_from_slice(key);
    }
}

impl HmacSha3Ctx {
    pub fn new(key: &[u8]) -> Self {
        let mut ret = Self {
            sha3_ctx: Sha3Ctx::new(SHA3_512_HASH_LEN),
            key: [0; SHA3_512_BLOCK_LEN]
        };
        prepare_key(key, &mut ret.key);
        let mut temp_key: [u8; SHA3_512_BLOCK_LEN] = [0; SHA3_512_BLOCK_LEN];
        // XOR with ipad
        temp_key.iter_mut().zip(key.iter())
            .for_each(|(dst, src)| {*dst = *src ^ 0x36; });
        ret.sha3_ctx.update(&temp_key);
        ret
    }

    pub fn update(&mut self, text: &[u8]) {
        self.sha3_ctx.update(text);
    }

    pub fn finalize(&mut self, hash: &mut [u8]) {
        let mut temp_key: [u8; SHA3_512_BLOCK_LEN] = [0; SHA3_512_BLOCK_LEN];
        let mut inner_hash: [u8; SHA3_512_HASH_LEN] = [0; SHA3_512_HASH_LEN];
        self.sha3_ctx.finalize(&mut inner_hash);
        temp_key.iter_mut().zip(self.key.iter())
            .for_each(|(dst, src)| {*dst = *src ^ 0x5c; });

        self.sha3_ctx = Sha3Ctx::new(SHA3_512_HASH_LEN);
        self.sha3_ctx.update(&temp_key);
        self.sha3_ctx.update(&inner_hash);
        self.sha3_ctx.finalize(hash);
    }
}

pub fn hmac_sha3(key: &mut [u8], text: &[u8], hmac: &mut [u8]) {
    let mut ctx = HmacSha3Ctx::new(key);
    ctx.update(text);
    ctx.finalize(hmac);
}


