const KECCAKF_ROUNDS: usize = 24;

const KECCAKF_RNDC: [usize; 24] = [
    0x0000000000000001, 0x0000000000008082, 0x800000000000808a,
    0x8000000080008000, 0x000000000000808b, 0x0000000080000001,
    0x8000000080008081, 0x8000000000008009, 0x000000000000008a,
    0x0000000000000088, 0x0000000080008009, 0x000000008000000a,
    0x000000008000808b, 0x800000000000008b, 0x8000000000008089,
    0x8000000000008003, 0x8000000000008002, 0x8000000000000080,
    0x000000000000800a, 0x800000008000000a, 0x8000000080008081,
    0x8000000000008080, 0x0000000080000001, 0x8000000080008008
];

const KECCAKF_ROTC: [usize; 24] = [
    1,  3,  6,  10, 15, 21, 28, 36, 45, 55, 2,  14,
    27, 41, 56, 8,  25, 43, 62, 18, 39, 61, 20, 44
];

const KECCAKF_PILN: [usize; 24] = [
    10, 7,  11, 17, 18, 3, 5,  16, 8,  21, 24, 4,
    15, 23, 19, 13, 12, 2, 20, 14, 22, 9,  6,  1
];

fn rotl64(x: usize, y: usize) -> usize {
    (x << y) | (x >> (64 - y))
}

fn sha3_keccakf(st: &mut [usize]) {
    let mut bc: [usize; 5] = [0; 5];
    for r in 0..KECCAKF_ROUNDS {
        for i in 0..5 {
            bc[i] = st[i] ^ st[i + 5] ^ st[i + 10] ^ st[i + 15] ^ st[i + 20];
        }

        for i in 0..5 {
            let t = bc[(i + 4) % 5] ^ rotl64(bc[(i + 1) % 5], 1);
            let mut j = 0;
            while j < 25 {
                st[j + i] ^= t;
                j += 5;
            }
        }

        let mut t = st[1];
        for i in 0..24 {
            let j = KECCAKF_PILN[i];
            bc[0] = st[j];
            st[j] = rotl64(t, keccakf_rotc[i]);
            t = bc[0];
        }

        let mut j = 0;
        while j < 25 {
            for i in 0..5 {
                bc[i] = st[j + i];
            }
            for i in 0..5 {
                st[j + i] ^= (!(bc[(i + 1) % 5])) & bc[(i + 2) % 5];
            }
            j += 5;
        }
        st[0] ^= keccakf_rndc[r];
    }
}

pub struct Sha3Ctx {
    st: [u8; 200],
    pt: usize,
    rsiz: usize,
    mdlen: usize
}

impl Sha3Ctx {
    pub fn new(mdlen: usize) -> Self {
        Sha3Ctx {
            st: [0; 200],
            pt: 0,
            rsiz: 200 - 2 * mdlen,
            mdlen
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        let mut j = self.pt;
        data.iter().for_each(|item| {
            self.st[j] ^= item;
            j += 1;
            if j >= self.rsiz {
                sha3_keccakf(&self.st as &[usize; 25]);
                 //sha3_keccakf(c->st.q);
                j = 0;
            }
        });
        self.pt = j;
    }

    pub fn finalize(&mut self, md: &mut [u8]) {
        self.st[self.pt] ^= 0x06;
        self.st[self.rsiz - 1] ^= 0x80;
        sha3_keccakf(&self.st as &[usize; 25]);
        md.iter_mut().zip(self.st.iter()).for_each(|(dst, src)| {*dst = *src; });
    }
}

pub fn compute(data: &[u8], md: &mut [u8]) {
    let mut ctx = Sha3Ctx::new(md.len());
    ctx.update(data);
    ctx.finalize(md);
}
