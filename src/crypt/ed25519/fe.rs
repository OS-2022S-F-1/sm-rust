use super::{load_3, load_4};

pub struct Fe(pub [u32; 10]);

impl From<Fe> for [u8; 32] {
    fn from(x: &Fe) -> Self {
        let mut h = x.0.clone();
        let mut q: u32 = (19 * h[9] + (1 << 24)) >> 25;
        q = (h[0] + q) >> 26;
        q = (h[1] + q) >> 25;
        q = (h[2] + q) >> 26;
        q = (h[3] + q) >> 25;
        q = (h[4] + q) >> 26;
        q = (h[5] + q) >> 25;
        q = (h[6] + q) >> 26;
        q = (h[7] + q) >> 25;
        q = (h[8] + q) >> 26;
        q = (h[9] + q) >> 25;
        /* Goal: Output h-(2^255-19)q as u8, which is between 0 and 2^255-20. */
        h[0] += 19 * q;
        /* Goal: Output h-2^255 q as u8, which is between 0 and 2^255-20. */
        let carry = h[0] >> 26;
        h[1] += carry;
        h[0] -= carry << 26;
        let carry = h[1] >> 25;
        h[2] += carry;
        h[1] -= carry << 25;
        let carry = h[2] >> 26;
        h[3] += carry;
        h[2] -= carry << 26;
        let carry = h[3] >> 25;
        h[4] += carry;
        h[3] -= carry << 25;
        let carry = h[4] >> 26;
        h[5] += carry;
        h[4] -= carry << 26;
        let carry = h[5] >> 25;
        h[6] += carry;
        h[5] -= carry << 25;
        let carry = h[6] >> 26;
        h[7] += carry;
        h[6] -= carry << 26;
        let carry = h[7] >> 25;
        h[8] += carry;
        h[7] -= carry << 25;
        let carry = h[8] >> 26;
        h[9] += carry;
        h[8] -= carry << 26;
        let carry = h[9] >> 25;
        h[9] -= carry << 25;

        [(h[0] >> 0) as u8, (h[0] >> 8) as u8, (h[0] >> 16) as u8, ((h[0] >> 24) | (h[1] << 2)) as u8,
        (h[1] >> 6) as u8, (h[1] >> 14) as u8, ((h[1] >> 22) | (h[2] << 3)) as u8, (h[2] >> 5 )as u8,
        (h[2] >> 13) as u8, ((h[2] >> 21) | (h[3] << 5)) as u8, (h[3] >> 3) as u8, (h[3] >> 11) as u8,
        ((h[3] >> 19) | (h[4] << 6)) as u8, (h[4] >> 2) as u8, (h[4] >> 10) as u8, (h[4] >> 18) as u8,
        (h[5] >> 0) as u8, (h[5] >> 8) as u8, (h[5] >> 16) as u8, ((h[5] >> 24) | (h[6] << 1)) as u8,
        (h[6] >> 7) as u8, (h[6] >> 15) as u8, ((h[6] >> 23) | (h[7] << 3)) as u8, (h[7] >> 5) as u8,
        (h[7] >> 13) as u8, ((h[7] >> 21) | (h[8] << 4)) as u8, (h[8] >> 4) as u8, (h[8] >> 12) as u8,
        ((h[8] >> 20) | (h[9] << 6)) as u8, (h[9] >> 2) as u8, (h[9] >> 10) as u8, (h[9] >> 18)]
    }
}

impl From<[u8; 32]> for Fe {
    fn from(x: &[u8]) -> Self {
        let mut h = [load_4(&x[0..4]), load_3(&x[4..7]) << 6,
            load_3(&x[7..10]) << 5, load_3(&x[10..13]) << 3,
            load_3(&x[13..16]) << 2, load_4(&x[16..20]),
            load_3(&x[20..23]) << 7, load_3(&x[23..26]) << 5,
            load_3(&x[26..29]) << 4, (load_3(&x[29..32]) & 8388607) << 2];
        let carry = (h[9] + (1 << 24)) >> 25;
        h[0] += carry * 19;
        h[9] -= carry << 25;

        let carry = (h[1] + (1 << 24)) >> 25;
        h[2] += carry;
        h[1] -= carry << 25;

        let carry = (h[3] + (1 << 24)) >> 25;
        h[4] += carry;
        h[3] -= carry << 25;


        let carry = (h[5] + (1 << 24)) >> 25;
        h[6] += carry;
        h[5] -= carry << 25;

        let carry = (h[7] + (1 << 24)) >> 25;
        h[8] += carry;
        h[7] -= carry << 25;

        let carry = (h[0] + (1 << 25)) >> 26;
        h[1] += carry;
        h[0] -= carry << 26;

        let carry = (h[2] + (1 << 25)) >> 26;
        h[3] += carry;
        h[2] -= carry << 26;

        let carry = (h[4] + (1 << 25)) >> 26;
        h[5] += carry;
        h[4] -= carry << 26;

        let carry = (h[6] + (1 << 25)) >> 26;
        h[7] += carry;
        h[6] -= carry << 26;

        let carry = (h[8] + (1 << 25)) >> 26;
        h[9] += carry;
        h[8] -= carry << 26;

        Self([h[0] as u32, h[1] as u32, h[2] as u32, h[3] as u32, h[4] as u32,
            h[5] as u32, h[6] as u32, h[7] as u32, h[8] as u32, h[9] as u32])
    }
}

impl Clone for Fe {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Fe {
    pub fn new() -> Self {
        Self([0; 10])
    }

    pub fn new_one() -> Self {
        Self([1, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    pub const D2: Fe = Fe([
        -21827239, -5839606, -30745221, 13898782, 229458, 15978800, -12551817, -6495438, 29715968, 9444199
    ]);

    pub fn add(f: &Fe, g: &Fe) -> Self {
        let mut ret = Fe::new();
        let mut overflow = false;
        for i in 0..10 {
            ret.0[i] = f.0[i] + g.0[i];
        }
        ret
    }

    pub fn sub(f: &Fe, g: &Fe) -> Self {
        let mut ret = Fe::new();
        let mut overflow = false;
        for i in 0..10 {
            ret.0[i] = f.0[i] - g.0[i];
        }
        ret
    }

    pub fn cmov(&mut self, g: &Fe, flag: bool) {
        if flag {
            self.copy(g);
        }
    }

    pub fn cswap(f: &mut Fe, g: &mut Fe, flag: bool) {
        if flag {
            for i in 0..10 {
                let tmp = f.0[i];
                f.0[i] = g.0[i];
                g.0[i] = f.0[i];
            }
        }
    }

    pub fn copy(&mut self, g: &Fe) {
        self.0.copy_from_slice(&g.0);
    }

    pub fn is_neg(&self) -> u8 {
        let s: [u8; 32] = self.into();
        s[0] & 1
    }

    pub fn mul(f: &Fe, g: &Fe) -> Self {
        let f0 = f.0[0] as usize;
        let f1 = f.0[1] as usize;
        let f2 = f.0[2] as usize;
        let f3 = f.0[3] as usize;
        let f4 = f.0[4] as usize;
        let f5 = f.0[5] as usize;
        let f6 = f.0[6] as usize;
        let f7 = f.0[7] as usize;
        let f8 = f.0[8] as usize;
        let f9 = f.0[9] as usize;
        let g0 = g.0[0] as usize;
        let g1 = g.0[1] as usize;
        let g2 = g.0[2] as usize;
        let g3 = g.0[3] as usize;
        let g4 = g.0[4] as usize;
        let g5 = g.0[5] as usize;
        let g6 = g.0[6] as usize;
        let g7 = g.0[7] as usize;
        let g8 = g.0[8] as usize;
        let g9 = g.0[9] as usize;
        let g1_19 = 19 * g1; /* 1.959375*2^29 */
        let g2_19 = 19 * g2; /* 1.959375*2^30; still ok */
        let g3_19 = 19 * g3;
        let g4_19 = 19 * g4;
        let g5_19 = 19 * g5;
        let g6_19 = 19 * g6;
        let g7_19 = 19 * g7;
        let g8_19 = 19 * g8;
        let g9_19 = 19 * g9;
        let f1_2 = 2 * f1;
        let f3_2 = 2 * f3;
        let f5_2 = 2 * f5;
        let f7_2 = 2 * f7;
        let f9_2 = 2 * f9;
        let f0g0    = f0   * g0;
        let f0g1    = f0   * g1;
        let f0g2    = f0   * g2;
        let f0g3    = f0   * g3;
        let f0g4    = f0   * g4;
        let f0g5    = f0   * g5;
        let f0g6    = f0   * g6;
        let f0g7    = f0   * g7;
        let f0g8    = f0   * g8;
        let f0g9    = f0   * g9;
        let f1g0    = f1   * g0;
        let f1g1_2  = f1_2 * g1;
        let f1g2    = f1   * g2;
        let f1g3_2  = f1_2 * g3;
        let f1g4    = f1   * g4;
        let f1g5_2  = f1_2 * g5;
        let f1g6    = f1   * g6;
        let f1g7_2  = f1_2 * g7;
        let f1g8    = f1   * g8;
        let f1g9_38 = f1_2 * g9_19;
        let f2g0    = f2   * g0;
        let f2g1    = f2   * g1;
        let f2g2    = f2   * g2;
        let f2g3    = f2   * g3;
        let f2g4    = f2   * g4;
        let f2g5    = f2   * g5;
        let f2g6    = f2   * g6;
        let f2g7    = f2   * g7;
        let f2g8_19 = f2   * g8_19;
        let f2g9_19 = f2   * g9_19;
        let f3g0    = f3   * g0;
        let f3g1_2  = f3_2 * g1;
        let f3g2    = f3   * g2;
        let f3g3_2  = f3_2 * g3;
        let f3g4    = f3   * g4;
        let f3g5_2  = f3_2 * g5;
        let f3g6    = f3   * g6;
        let f3g7_38 = f3_2 * g7_19;
        let f3g8_19 = f3   * g8_19;
        let f3g9_38 = f3_2 * g9_19;
        let f4g0    = f4   * g0;
        let f4g1    = f4   * g1;
        let f4g2    = f4   * g2;
        let f4g3    = f4   * g3;
        let f4g4    = f4   * g4;
        let f4g5    = f4   * g5;
        let f4g6_19 = f4   * g6_19;
        let f4g7_19 = f4   * g7_19;
        let f4g8_19 = f4   * g8_19;
        let f4g9_19 = f4   * g9_19;
        let f5g0    = f5   * g0;
        let f5g1_2  = f5_2 * g1;
        let f5g2    = f5   * g2;
        let f5g3_2  = f5_2 * g3;
        let f5g4    = f5   * g4;
        let f5g5_38 = f5_2 * g5_19;
        let f5g6_19 = f5   * g6_19;
        let f5g7_38 = f5_2 * g7_19;
        let f5g8_19 = f5   * g8_19;
        let f5g9_38 = f5_2 * g9_19;
        let f6g0    = f6   * g0;
        let f6g1    = f6   * g1;
        let f6g2    = f6   * g2;
        let f6g3    = f6   * g3;
        let f6g4_19 = f6   * g4_19;
        let f6g5_19 = f6   * g5_19;
        let f6g6_19 = f6   * g6_19;
        let f6g7_19 = f6   * g7_19;
        let f6g8_19 = f6   * g8_19;
        let f6g9_19 = f6   * g9_19;
        let f7g0    = f7   * g0;
        let f7g1_2  = f7_2 * g1;
        let f7g2    = f7   * g2;
        let f7g3_38 = f7_2 * g3_19;
        let f7g4_19 = f7   * g4_19;
        let f7g5_38 = f7_2 * g5_19;
        let f7g6_19 = f7   * g6_19;
        let f7g7_38 = f7_2 * g7_19;
        let f7g8_19 = f7   * g8_19;
        let f7g9_38 = f7_2 * g9_19;
        let f8g0    = f8   * g0;
        let f8g1    = f8   * g1;
        let f8g2_19 = f8   * g2_19;
        let f8g3_19 = f8   * g3_19;
        let f8g4_19 = f8   * g4_19;
        let f8g5_19 = f8   * g5_19;
        let f8g6_19 = f8   * g6_19;
        let f8g7_19 = f8   * g7_19;
        let f8g8_19 = f8   * g8_19;
        let f8g9_19 = f8   * g9_19;
        let f9g0    = f9   * g0;
        let f9g1_38 = f9_2 * g1_19;
        let f9g2_19 = f9   * g2_19;
        let f9g3_38 = f9_2 * g3_19;
        let f9g4_19 = f9   * g4_19;
        let f9g5_38 = f9_2 * g5_19;
        let f9g6_19 = f9   * g6_19;
        let f9g7_38 = f9_2 * g7_19;
        let f9g8_19 = f9   * g8_19;
        let f9g9_38 = f9_2 * g9_19;
        let mut h0 = f0g0 + f1g9_38 + f2g8_19 + f3g7_38 + f4g6_19 + f5g5_38 + f6g4_19 + f7g3_38 + f8g2_19 + f9g1_38;
        let mut h1 = f0g1 + f1g0   + f2g9_19 + f3g8_19 + f4g7_19 + f5g6_19 + f6g5_19 + f7g4_19 + f8g3_19 + f9g2_19;
        let mut h2 = f0g2 + f1g1_2 + f2g0   + f3g9_38 + f4g8_19 + f5g7_38 + f6g6_19 + f7g5_38 + f8g4_19 + f9g3_38;
        let mut h3 = f0g3 + f1g2   + f2g1   + f3g0   + f4g9_19 + f5g8_19 + f6g7_19 + f7g6_19 + f8g5_19 + f9g4_19;
        let mut h4 = f0g4 + f1g3_2 + f2g2   + f3g1_2 + f4g0   + f5g9_38 + f6g8_19 + f7g7_38 + f8g6_19 + f9g5_38;
        let mut h5 = f0g5 + f1g4   + f2g3   + f3g2   + f4g1   + f5g0   + f6g9_19 + f7g8_19 + f8g7_19 + f9g6_19;
        let mut h6 = f0g6 + f1g5_2 + f2g4   + f3g3_2 + f4g2   + f5g1_2 + f6g0   + f7g9_38 + f8g8_19 + f9g7_38;
        let mut h7 = f0g7 + f1g6   + f2g5   + f3g4   + f4g3   + f5g2   + f6g1   + f7g0   + f8g9_19 + f9g8_19;
        let mut h8 = f0g8 + f1g7_2 + f2g6   + f3g5_2 + f4g4   + f5g3_2 + f6g2   + f7g1_2 + f8g0   + f9g9_38;
        let mut h9 = f0g9 + f1g8   + f2g7   + f3g6   + f4g5   + f5g4   + f6g3   + f7g2   + f8g1   + f9g0   ;
        let carry= (h0 + (1 << 25)) >> 26;
        h1 += carry;
        h0 -= carry << 26;
        let carry= (h4 + (1 << 25)) >> 26;
        h5 += carry;
        h4 -= carry << 26;

        let carry= (h1 + (1 << 24)) >> 25;
        h2 += carry;
        h1 -= carry << 25;
        let carry= (h5 + (1 << 24)) >> 25;
        h6 += carry;
        h5 -= carry << 25;

        let carry= (h2 + (1 << 25)) >> 26;
        h3 += carry;
        h2 -= carry << 26;
        let carry= (h6 + (1 << 25)) >> 26;
        h7 += carry;
        h6 -= carry << 26;

        let carry= (h3 + (1 << 24)) >> 25;
        h4 += carry;
        h3 -= carry << 25;
        let carry= (h7 + (1 << 24)) >> 25;
        h8 += carry;
        h7 -= carry << 25;

        let carry= (h4 + (1 << 25)) >> 26;
        h5 += carry;
        h4 -= carry << 26;
        let carry= (h8 + (1 << 25)) >> 26;
        h9 += carry;
        h8 -= carry << 26;

        let carry= (h9 + (1 << 24)) >> 25;
        h0 += carry * 19;
        h9 -= carry << 25;

        let carry= (h0 + (1 << 25)) >> 26;
        h1 += carry;
        h0 -= carry << 26;
        Self([h0 as u32, h1 as u32, h2 as u32, h3 as u32, h4 as u32,
            h5 as u32, h6 as u32, h7 as u32, h8 as u32, h9 as u32])
    }
    
    pub fn sq(f: &Fe) -> Self {
        let f0 = f.0[0] as usize;
        let f1 = f.0[1] as usize;
        let f2 = f.0[2] as usize;
        let f3 = f.0[3] as usize;
        let f4 = f.0[4] as usize;
        let f5 = f.0[5] as usize;
        let f6 = f.0[6] as usize;
        let f7 = f.0[7] as usize;
        let f8 = f.0[8] as usize;
        let f9 = f.0[9] as usize;
        let f0_2 = 2 * f0;
        let f1_2 = 2 * f1;
        let f2_2 = 2 * f2;
        let f3_2 = 2 * f3;
        let f4_2 = 2 * f4;
        let f5_2 = 2 * f5;
        let f6_2 = 2 * f6;
        let f7_2 = 2 * f7;
        let f5_38 = 38 * f5; /* 1.959375*2^30 */
        let f6_19 = 19 * f6; /* 1.959375*2^30 */
        let f7_38 = 38 * f7; /* 1.959375*2^30 */
        let f8_19 = 19 * f8; /* 1.959375*2^30 */
        let f9_38 = 38 * f9; /* 1.959375*2^30 */
        let f0f0    = f0   *  f0;
        let f0f1_2  = f0_2 *  f1;
        let f0f2_2  = f0_2 *  f2;
        let f0f3_2  = f0_2 *  f3;
        let f0f4_2  = f0_2 *  f4;
        let f0f5_2  = f0_2 *  f5;
        let f0f6_2  = f0_2 *  f6;
        let f0f7_2  = f0_2 *  f7;
        let f0f8_2  = f0_2 *  f8;
        let f0f9_2  = f0_2 *  f9;
        let f1f1_2  = f1_2 *  f1;
        let f1f2_2  = f1_2 *  f2;
        let f1f3_4  = f1_2 *  f3_2;
        let f1f4_2  = f1_2 *  f4;
        let f1f5_4  = f1_2 *  f5_2;
        let f1f6_2  = f1_2 *  f6;
        let f1f7_4  = f1_2 *  f7_2;
        let f1f8_2  = f1_2 *  f8;
        let f1f9_76 = f1_2 *  f9_38;
        let f2f2    = f2   *  f2;
        let f2f3_2  = f2_2 *  f3;
        let f2f4_2  = f2_2 *  f4;
        let f2f5_2  = f2_2 *  f5;
        let f2f6_2  = f2_2 *  f6;
        let f2f7_2  = f2_2 *  f7;
        let f2f8_38 = f2_2 *  f8_19;
        let f2f9_38 = f2   *  f9_38;
        let f3f3_2  = f3_2 *  f3;
        let f3f4_2  = f3_2 *  f4;
        let f3f5_4  = f3_2 *  f5_2;
        let f3f6_2  = f3_2 *  f6;
        let f3f7_76 = f3_2 *  f7_38;
        let f3f8_38 = f3_2 *  f8_19;
        let f3f9_76 = f3_2 *  f9_38;
        let f4f4    = f4   *  f4;
        let f4f5_2  = f4_2 *  f5;
        let f4f6_38 = f4_2 *  f6_19;
        let f4f7_38 = f4   *  f7_38;
        let f4f8_38 = f4_2 *  f8_19;
        let f4f9_38 = f4   *  f9_38;
        let f5f5_38 = f5   *  f5_38;
        let f5f6_38 = f5_2 *  f6_19;
        let f5f7_76 = f5_2 *  f7_38;
        let f5f8_38 = f5_2 *  f8_19;
        let f5f9_76 = f5_2 *  f9_38;
        let f6f6_19 = f6   *  f6_19;
        let f6f7_38 = f6   *  f7_38;
        let f6f8_38 = f6_2 *  f8_19;
        let f6f9_38 = f6   *  f9_38;
        let f7f7_38 = f7   *  f7_38;
        let f7f8_38 = f7_2 *  f8_19;
        let f7f9_76 = f7_2 *  f9_38;
        let f8f8_19 = f8   *  f8_19;
        let f8f9_38 = f8   *  f9_38;
        let f9f9_38 = f9   *  f9_38;

        let mut h0 = f0f0  + f1f9_76 + f2f8_38 + f3f7_76 + f4f6_38 + f5f5_38;
        let mut h1 = f0f1_2 + f2f9_38 + f3f8_38 + f4f7_38 + f5f6_38;
        let mut h2 = f0f2_2 + f1f1_2 + f3f9_76 + f4f8_38 + f5f7_76 + f6f6_19;
        let mut h3 = f0f3_2 + f1f2_2 + f4f9_38 + f5f8_38 + f6f7_38;
        let mut h4 = f0f4_2 + f1f3_4 + f2f2   + f5f9_76 + f6f8_38 + f7f7_38;
        let mut h5 = f0f5_2 + f1f4_2 + f2f3_2 + f6f9_38 + f7f8_38;
        let mut h6 = f0f6_2 + f1f5_4 + f2f4_2 + f3f3_2 + f7f9_76 + f8f8_19;
        let mut h7 = f0f7_2 + f1f6_2 + f2f5_2 + f3f4_2 + f8f9_38;
        let mut h8 = f0f8_2 + f1f7_4 + f2f6_2 + f3f5_4 + f4f4   + f9f9_38;
        let mut h9 = f0f9_2 + f1f8_2 + f2f7_2 + f3f6_2 + f4f5_2;

        let carry = (h0 +  (1 << 25)) >> 26;
        h1 += carry;
        h0 -= carry << 26;
        let carry = (h4 +  (1 << 25)) >> 26;
        h5 += carry;
        h4 -= carry << 26;
        let carry = (h1 +  (1 << 24)) >> 25;
        h2 += carry;
        h1 -= carry << 25;
        let carry = (h5 +  (1 << 24)) >> 25;
        h6 += carry;
        h5 -= carry << 25;
        let carry = (h2 +  (1 << 25)) >> 26;
        h3 += carry;
        h2 -= carry << 26;
        let carry = (h6 +  (1 << 25)) >> 26;
        h7 += carry;
        h6 -= carry << 26;
        let carry = (h3 +  (1 << 24)) >> 25;
        h4 += carry;
        h3 -= carry << 25;
        let carry = (h7 +  (1 << 24)) >> 25;
        h8 += carry;
        h7 -= carry << 25;
        let carry = (h4 +  (1 << 25)) >> 26;
        h5 += carry;
        h4 -= carry << 26;
        let carry = (h8 +  (1 << 25)) >> 26;
        h9 += carry;
        h8 -= carry << 26;
        let carry = (h9 +  (1 << 24)) >> 25;
        h0 += carry * 19;
        h9 -= carry << 25;
        let carry = (h0 +  (1 << 25)) >> 26;
        h1 += carry;
        h0 -= carry << 26;
        Self([h0 as u32, h1 as u32, h2 as u32, h3 as u32, h4 as u32,
            h5 as u32, h6 as u32, h7 as u32, h8 as u32, h9 as u32])
    }

    pub fn sq2(f: &Fe) -> Self {
        let f0 = f.0[0] as usize;
        let f1 = f.0[1] as usize;
        let f2 = f.0[2] as usize;
        let f3 = f.0[3] as usize;
        let f4 = f.0[4] as usize;
        let f5 = f.0[5] as usize;
        let f6 = f.0[6] as usize;
        let f7 = f.0[7] as usize;
        let f8 = f.0[8] as usize;
        let f9 = f.0[9] as usize;
        let f0_2 = 2 * f0;
        let f1_2 = 2 * f1;
        let f2_2 = 2 * f2;
        let f3_2 = 2 * f3;
        let f4_2 = 2 * f4;
        let f5_2 = 2 * f5;
        let f6_2 = 2 * f6;
        let f7_2 = 2 * f7;
        let f5_38 = 38 * f5; /* 1.959375*2^30 */
        let f6_19 = 19 * f6; /* 1.959375*2^30 */
        let f7_38 = 38 * f7; /* 1.959375*2^30 */
        let f8_19 = 19 * f8; /* 1.959375*2^30 */
        let f9_38 = 38 * f9; /* 1.959375*2^30 */
        let f0f0    = f0   * f0;
        let f0f1_2  = f0_2 * f1;
        let f0f2_2  = f0_2 * f2;
        let f0f3_2  = f0_2 * f3;
        let f0f4_2  = f0_2 * f4;
        let f0f5_2  = f0_2 * f5;
        let f0f6_2  = f0_2 * f6;
        let f0f7_2  = f0_2 * f7;
        let f0f8_2  = f0_2 * f8;
        let f0f9_2  = f0_2 * f9;
        let f1f1_2  = f1_2 * f1;
        let f1f2_2  = f1_2 * f2;
        let f1f3_4  = f1_2 * f3_2;
        let f1f4_2  = f1_2 * f4;
        let f1f5_4  = f1_2 * f5_2;
        let f1f6_2  = f1_2 * f6;
        let f1f7_4  = f1_2 * f7_2;
        let f1f8_2  = f1_2 * f8;
        let f1f9_76 = f1_2 * f9_38;
        let f2f2    = f2   * f2;
        let f2f3_2  = f2_2 * f3;
        let f2f4_2  = f2_2 * f4;
        let f2f5_2  = f2_2 * f5;
        let f2f6_2  = f2_2 * f6;
        let f2f7_2  = f2_2 * f7;
        let f2f8_38 = f2_2 * f8_19;
        let f2f9_38 = f2   * f9_38;
        let f3f3_2  = f3_2 * f3;
        let f3f4_2  = f3_2 * f4;
        let f3f5_4  = f3_2 * f5_2;
        let f3f6_2  = f3_2 * f6;
        let f3f7_76 = f3_2 * f7_38;
        let f3f8_38 = f3_2 * f8_19;
        let f3f9_76 = f3_2 * f9_38;
        let f4f4    = f4   * f4;
        let f4f5_2  = f4_2 * f5;
        let f4f6_38 = f4_2 * f6_19;
        let f4f7_38 = f4   * f7_38;
        let f4f8_38 = f4_2 * f8_19;
        let f4f9_38 = f4   * f9_38;
        let f5f5_38 = f5   * f5_38;
        let f5f6_38 = f5_2 * f6_19;
        let f5f7_76 = f5_2 * f7_38;
        let f5f8_38 = f5_2 * f8_19;
        let f5f9_76 = f5_2 * f9_38;
        let f6f6_19 = f6   * f6_19;
        let f6f7_38 = f6   * f7_38;
        let f6f8_38 = f6_2 * f8_19;
        let f6f9_38 = f6   * f9_38;
        let f7f7_38 = f7   * f7_38;
        let f7f8_38 = f7_2 * f8_19;
        let f7f9_76 = f7_2 * f9_38;
        let f8f8_19 = f8   * f8_19;
        let f8f9_38 = f8   * f9_38;
        let f9f9_38 = f9   * f9_38;
        let mut h0 = f0f0  + f1f9_76 + f2f8_38 + f3f7_76 + f4f6_38 + f5f5_38;
        let mut h1 = f0f1_2 + f2f9_38 + f3f8_38 + f4f7_38 + f5f6_38;
        let mut h2 = f0f2_2 + f1f1_2 + f3f9_76 + f4f8_38 + f5f7_76 + f6f6_19;
        let mut h3 = f0f3_2 + f1f2_2 + f4f9_38 + f5f8_38 + f6f7_38;
        let mut h4 = f0f4_2 + f1f3_4 + f2f2   + f5f9_76 + f6f8_38 + f7f7_38;
        let mut h5 = f0f5_2 + f1f4_2 + f2f3_2 + f6f9_38 + f7f8_38;
        let mut h6 = f0f6_2 + f1f5_4 + f2f4_2 + f3f3_2 + f7f9_76 + f8f8_19;
        let mut h7 = f0f7_2 + f1f6_2 + f2f5_2 + f3f4_2 + f8f9_38;
        let mut h8 = f0f8_2 + f1f7_4 + f2f6_2 + f3f5_4 + f4f4   + f9f9_38;
        let mut h9 = f0f9_2 + f1f8_2 + f2f7_2 + f3f6_2 + f4f5_2;
        h0 += h0;
        h1 += h1;
        h2 += h2;
        h3 += h3;
        h4 += h4;
        h5 += h5;
        h6 += h6;
        h7 += h7;
        h8 += h8;
        h9 += h9;
        let carry = (h0 + (1 << 25)) >> 26;
        h1 += carry;
        h0 -= carry << 26;
        let carry = (h4 + (1 << 25)) >> 26;
        h5 += carry;
        h4 -= carry << 26;
        let carry = (h1 + (1 << 24)) >> 25;
        h2 += carry;
        h1 -= carry << 25;
        let carry = (h5 + (1 << 24)) >> 25;
        h6 += carry;
        h5 -= carry << 25;
        let carry = (h2 + (1 << 25)) >> 26;
        h3 += carry;
        h2 -= carry << 26;
        let carry = (h6 + (1 << 25)) >> 26;
        h7 += carry;
        h6 -= carry << 26;
        let carry = (h3 + (1 << 24)) >> 25;
        h4 += carry;
        h3 -= carry << 25;
        let carry = (h7 + (1 << 24)) >> 25;
        h8 += carry;
        h7 -= carry << 25;
        let carry = (h4 + (1 << 25)) >> 26;
        h5 += carry;
        h4 -= carry << 26;
        let carry = (h8 + (1 << 25)) >> 26;
        h9 += carry;
        h8 -= carry << 26;
        let carry = (h9 + (1 << 24)) >> 25;
        h0 += carry * 19;
        h9 -= carry << 25;
        let carry = (h0 + (1 << 25)) >> 26;
        h1 += carry;
        h0 -= carry << 26;
        Self([h0 as u32, h1 as u32, h2 as u32, h3 as u32, h4 as u32,
            h5 as u32, h6 as u32, h7 as u32, h8 as u32, h9 as u32])
    }

    pub fn mul121666(f: &Fe) -> Self {
        let mut h: [usize; 10] = [0; 10];
        for i in 0..10 {
            h[i] = (f.0[i] * 121666) as usize;
        }
        let carry = (h[9] + (1 << 24)) >> 25;
        h[0] += carry * 19;
        h[9] -= carry << 25;

        let carry = (h[1] + (1 << 24)) >> 25;
        h[2] += carry;
        h[1] -= carry << 25;

        let carry = (h[3] + (1 << 24)) >> 25;
        h[4] += carry;
        h[3] -= carry << 25;


        let carry = (h[5] + (1 << 24)) >> 25;
        h[6] += carry;
        h[5] -= carry << 25;

        let carry = (h[7] + (1 << 24)) >> 25;
        h[8] += carry;
        h[7] -= carry << 25;

        let carry = (h[0] + (1 << 25)) >> 26;
        h[1] += carry;
        h[0] -= carry << 26;

        let carry = (h[2] + (1 << 25)) >> 26;
        h[3] += carry;
        h[2] -= carry << 26;

        let carry = (h[4] + (1 << 25)) >> 26;
        h[5] += carry;
        h[4] -= carry << 26;

        let carry = (h[6] + (1 << 25)) >> 26;
        h[7] += carry;
        h[6] -= carry << 26;

        let carry = (h[8] + (1 << 25)) >> 26;
        h[9] += carry;
        h[8] -= carry << 26;

        Self([h[0] as u32, h[1] as u32, h[2] as u32, h[3] as u32, h[4] as u32,
            h[5] as u32, h[6] as u32, h[7] as u32, h[8] as u32, h[9] as u32])
    }

    pub fn neg(f: &Fe) -> Self {
        let mut ret = Fe::new();
        for i in 0..10 {
            ret.0[i] = -f.0[i];
        }
        ret
    }

    pub fn invert(z: &Fe) -> Self {
        let mut t0 = Fe::sq(z);
        let mut t1 = Fe::sq(&t0);
        t1 = Fe::sq(&t1);
        t1 = Fe::mul(&t1, z);
        t0 = Fe::mul(&t0, &t1);

        let mut t2 = Fe::sq(&t0);
        t1 = Fe::mul(&t1, &t2);
        t2 = Fe::sq(&t1);

        for _ in 1..5 {
            t2 = Fe::sq(&t2);
        }

        t1 = Fe::mul(&t1, &t2);
        t2 = Fe::sq(&t1);

        for _ in 1..10 {
            t2 = Fe::sq(&t2);
        }

        t2 = Fe::mul(&t1, &t2);
        let mut t3 = Fe::sq(&t2);

        for _ in 1..20 {
            t3 = Fe::sq(&t3);
        }

        t2 = Fe::mul(&t3, &t2);
        t2 = Fe::sq(&t2);


        for _ in 1..10 {
            t2 = Fe::sq(&t2);
        }

        t1 = Fe::mul(&t1, &t2);
        t2 = Fe::sq(&t1);

        for _ in 1..50 {
            t2 = Fe::sq(&t2);
        }

        t2 = Fe::mul(&t1, &t2);
        t3 = Fe::sq(&t2);

        for _ in 1..100 {
            t3 = Fe::sq(&t3);
        }

        t2 = Fe::mul(&t3, &t2);
        t2 = Fe::sq(&t2);

        for _ in 1..50 {
            t2 = Fe::sq(&t2);
        }

        t1 = Fe::mul(&t1, &t2);
        t1 = Fe::sq(&t1);

        for _ in 1..5 {
            t1 = Fe::sq(&t1);
        }
        Fe::mul(&t0, &t1)
    }

    pub fn pow22523(z: &Fe) -> Self {
    let mut t0 = Fe::sq(z);
    let mut t1 = Fe::sq(&t0);
    t1 = Fe::sq(&t1);

    t1 = Fe::mul(&z, &t1);
    t0 = Fe::mul(&t0, &t1);
    t0 = Fe::sq(&t0);

    t0 = Fe::mul(&t1, &t0);
    t1 = Fe::sq(&t0);

    for _ in 1..5 {
        t1 = Fe::sq(&t1);
    }

    t0 = Fe::mul(&t1, &t0);
    t1 = Fe::sq(&t0);

    for _ in 1..10 {
        t1 = Fe::sq(&t1);
    }

    t1 = Fe::mul(&t1, &t0);
    let mut t2 = Fe::sq(&t1);

    for _ in 1..20 {
        t2 = Fe::sq(&t2);
    }

    t1 = Fe::mul(&t2, &t1);
    t1 = Fe::sq(&t1);

    for _ in 1..10 {
        t1 = Fe::sq(&t1);
    }

    t0 = Fe::mul(&t1, &t0);
    t1 = Fe::sq(&t0);

    for _ in 1..50 {
        t1 = Fe::sq(&t1);
    }

    t1 = Fe::mul(&t1, &t0);
    t2 = Fe::sq(&t1);

    for _ in 1..100 {
        t2 = Fe::sq(&t2);
    }

    t1 = Fe::mul(&t2, &t1);
    t1 = Fe::sq(&t1);

    for _ in 1..50 {
        t1 = Fe::sq(&t1);
    }

    t0 = Fe::mul(&t1, &t0);
    t0 = Fe::sq(&t0);
    t0 = Fe::sq(&t0);
    Fe::mul(&t0, &z)
    }

}