use crate::crypt::ed25519::precomp_data::{BASE, BI};
use super::fe::{Fe, D2, D, SQRT_M1};

pub struct GeP2 {
    pub x: Fe,
    pub y: Fe,
    pub z: Fe
}

pub struct GeP3 {
    pub x: Fe,
    pub y: Fe,
    pub z: Fe,
    pub t: Fe
}

pub struct GeP1P1 {
    pub x: Fe,
    pub y: Fe,
    pub z: Fe,
    pub t: Fe
}

pub struct GePrecomp {
    pub yplusx: Fe,
    pub yminusx: Fe,
    pub xy2d: Fe
}

pub struct GeCache {
    pub yplusx: Fe,
    pub yminusx: Fe,
    pub z: Fe,
    pub t2d: Fe
}

impl From<GeP1P1> for GeP2 {
    fn from(p: GeP1P1) -> Self {
        Self {
            x: Fe::mul(&p.x, &p.t),
            y: Fe::mul(&p.y, &p.z),
            z: Fe::mul(&p.z, &p.t)

        }
    }
}

impl From<GeP3> for GeP2 {
    fn from(p: GeP3) -> Self {
        Self {
            x: p.x.clone(),
            y: p.y.clone(),
            z: p.z.clone()

        }
    }
}

impl From<GeP1P1> for GeP3 {
    fn from(p: GeP1P1) -> Self {
        Self {
            x: Fe::mul(&p.x, &p.t),
            y: Fe::mul(&p.y, &p.z),
            z: Fe::mul(&p.z, &p.t),
            t: Fe::mul(&p.x, &p.y)

        }
    }
}

impl From<GeP3> for GeCache {
    fn from(p: GeP3) -> Self {
        Self {
            yplusx: Fe::add(&p.y, &p.x),
            yminusx: Fe::sub(&p.y, &p.x),
            z: p.z.clone(),
            t2d: Fe::mul(&p.t, &D2)
        }
    }
}

impl From<GeP3> for [u8; 32] {
    fn from(h: GeP3) -> Self {
        let recip = Fe::invert(&h.z);
        let x = Fe::mul(&h.x, &recip);
        let y = Fe::mul(&h.y, &recip);
        let mut s: [u8; 32] = y.into();
        s[31] ^= x.is_neg() << 7;
        s
    }
}

impl From<[[i32; 10]; 3]> for GePrecomp {
    fn from(x: [[i32; 10]; 3]) -> Self {
        Self {
            yplusx: x[0].into(),
            yminusx: x[1].into(),
            xy2d: x[2].into()
        }
    }
}

impl From<GeP2> for [u8; 32] {
    fn from(h: GeP2) -> Self {
        let recip = Fe::invert(&h.z);
        let x = Fe::mul(&h.x, &recip);
        let y = Fe::mul(&h.y, &recip);
        let mut s:[u8; 32] = y.into();
        s[31] ^= x.is_neg() << 7;
        s
    }
}

impl Clone for GeP3 {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
            t: self.t.clone()
        }
    }
}

impl Clone for GeCache {
    fn clone(&self) -> Self {
        Self {
            yplusx: self.yplusx.clone(),
            yminusx: self.yminusx.clone(),
            z: self.z.clone(),
            t2d: self.t2d.clone()
        }
    }
}

impl Copy for GeCache {

}

impl Copy for GeP3 {

}

impl GeP2 {
    pub fn new() -> Self {
        Self {
            x: Fe::new(),
            y: Fe::new_one(),
            z: Fe::new_one()
        }
    }
}

impl GeP3 {
    pub fn new() -> Self {
        Self {
            x: Fe::new(),
            y: Fe::new_one(),
            z: Fe::new_one(),
            t: Fe::new()
        }
    }

    pub fn frombytes_negate_vartime(s: &[u8]) -> Option<Self> {
        let mut y: Fe = s.into();
        let mut z = Fe::new_one();
        let mut u = Fe::sq(&y);
        let mut v = Fe::mul(&u, &D);
        u = Fe::sub(&u, &z);
        v = Fe::add(&v, &z);
        let mut v3 = Fe::sq(&v);
        v3 = Fe::mul(&v3, &v);
        let mut x = Fe::sq(&v3);
        x = Fe::mul(&x, &v);
        x = Fe::mul(&x, &u);
        x = Fe::pow22523(&x);
        x = Fe::mul(&x, &v3);
        x = Fe::mul(&x, &u);
        let mut vxx = Fe::sq(&x);
        vxx = Fe::mul(&vxx, &v);
        let mut check = Fe::sub(&vxx, &u);
        if !check.is_zero() {
            check = Fe::add(&vxx, &u);
            if !check.is_zero() {
                return None;
            }
            x = Fe::mul(&x, &SQRT_M1)
        }

        if x.is_neg() == (s[31] >> 7) {
            x = Fe::neg(&x);
        }
        let t = Fe::mul(&x, &y);
        Some(Self {
            x,
            y,
            z,
            t
        })
    }
}

impl GeP1P1 {
    pub fn new() -> Self {
        Self {
            x: Fe::new(),
            y: Fe::new_one(),
            z: Fe::new_one(),
            t: Fe::new()
        }
    }
}

impl GeCache {
    pub fn new() -> Self {
        Self {
            yplusx: Fe::new(),
            yminusx: Fe::new(),
            z: Fe::new(),
            t2d: Fe::new()
        }
    }
}

pub fn slide(r: &mut [i8], a: &[u8]) {

    for i in 0..256 {
        r[i] = (1 & (a[i >> 3] >> (i & 7))) as i8;
    }

    for i in 0..256 {
        if r[i] > 0 {
            let mut b = 1;
            while b <= 6 && i + b < 256 {
                if r[i + b] > 0 {
                    if r[i] + (r[i + b] << b) <= 15 {
                        r[i] += r[i + b] << b;
                        r[i + b] = 0;
                    } else if r[i] - (r[i + b] << b) >= -15 {
                        r[i] -= r[i + b] << b;
                        for k in (i+b)..256 {
                            if r[k] == 0 {
                                r[k] = 1;
                                break;
                            }
                            r[k] = 0;
                        }
                    } else {
                        break;
                    }
                }
                b += 1;
            }
        }
    }
}

pub fn add(p: &GeP3, q: &GeCache) -> GeP1P1 {
    let mut r = GeP1P1::new();
    r.x = Fe::add(&p.y, &p.x);
    r.y = Fe::sub(&p.y, &p.x);
    r.z = Fe::mul(&r.x, &q.yplusx);
    r.y = Fe::mul(&r.y, &q.yminusx);
    r.t = Fe::mul(&q.t2d, &p.t);
    r.x = Fe::mul(&p.z, &q.z);
    let t0 = Fe::add(&r.x, &r.x);
    r.x = Fe::sub(&r.z, &r.y);
    r.y = Fe::add(&r.z, &r.y);
    r.z = Fe::add(&t0, &r.t);
    r.t = Fe::sub(&t0, &r.t);
    r
}

pub fn sub(p: &GeP3, q: &GeCache) -> GeP1P1 {
    let mut r = GeP1P1::new();
    r.x = Fe::add(&p.y, &p.x);
    r.y = Fe::sub(&p.y, &p.x);
    r.z = Fe::mul(&r.x, &q.yminusx);
    r.y = Fe::mul(&r.y, &q.yplusx);
    r.t = Fe::mul(&q.t2d, &p.t);
    r.x = Fe::mul(&p.z, &q.z);
    let t0 = Fe::add(&r.x, &r.x);
    r.x = Fe::sub(&r.z, &r.y);
    r.y = Fe::add(&r.z, &r.y);
    r.z = Fe::sub(&t0, &r.t);
    r.t = Fe::add(&t0, &r.t);
    r
}

pub fn madd(p: &GeP3, q: &GePrecomp) -> GeP1P1  {
        let mut r = GeP1P1::new();
        r.x = Fe::add(&p.y, &p.x);
        r.y = Fe::sub(&p.y, &p.x);
        r.z = Fe::mul(&r.x, &q.yplusx);
        r.y = Fe::mul(&r.y, &q.yminusx);
        r.t = Fe::mul(&q.xy2d, &p.t);
        let t0 = Fe::add(&p.z, &p.z);
        r.x = Fe::sub(&r.z, &r.y);
        r.y = Fe::add(&r.z, &r.y);
        r.z = Fe::add(&t0, &r.t);
        r.t = Fe::sub(&t0, &r.t);
        r
}


/*
r = p - q
*/

pub fn msub(p: &GeP3, q: &GePrecomp) -> GeP1P1  {
    let mut r = GeP1P1::new();
    r.x = Fe::add(&p.y, &p.x);
    r.y = Fe::sub(&p.y, &p.x);
    r.z = Fe::mul(&r.x, &q.yminusx);
    r.y = Fe::mul(&r.y, &q.yplusx);
    r.t = Fe::mul(&q.xy2d, &p.t);
    let t0 = Fe::add(&p.z, &p.z);
    r.x = Fe::sub(&r.z, &r.y);
    r.y = Fe::add(&r.z, &r.y);
    r.z = Fe::sub(&t0, &r.t);
    r.t = Fe::add(&t0, &r.t);
    r
}

pub fn p2_dbl(p: &GeP2) -> GeP1P1 {
    let mut r = GeP1P1 {
        x: Fe::mul(&p.x, &p.x),
        y: Fe::add(&p.x, &p.y),
        z: Fe::mul(&p.y, &p.y),
        t: Fe::sq2(&p.z)
    };
    let t0 = Fe::sq(&r.y);
    r.y = Fe::add(&r.z, &r.x);
    r.z = Fe::sub(&r.z, &r.x);
    r.x = Fe::sub(&t0, &r.y);
    r.t = Fe::sub(&r.t, &r.z);
    r
}

pub fn p3_dbl(p: &GeP3) -> GeP1P1 {
    let q: GeP2 = (*p).into();
    p2_dbl(&q)
}

impl Clone for GePrecomp {
    fn clone(&self) -> Self {
        Self {
            yplusx: self.yplusx.clone(),
            yminusx: self.yminusx.clone(),
            xy2d: self.xy2d.clone()
        }
    }
}

impl GePrecomp {
    pub fn cmov(&mut self, g: &GePrecomp, flag: bool) {
        if flag {
            self.xy2d.copy(&g.xy2d);
            self.yminusx.copy(&g.yminusx);
            self.yplusx.copy(&g.yplusx);
        }
    }

    pub fn new(pos: usize, b: u8) -> Self {
        let flag = (b as i8) < 0;
        let babs = b - (((-(flag as i8)) as u8 & b) << 1);
        let r = match babs {
            1 => BASE[pos][0].clone(),
            2 => BASE[pos][1].clone(),
            3 => BASE[pos][2].clone(),
            4 => BASE[pos][3].clone(),
            5 => BASE[pos][4].clone(),
            6 => BASE[pos][5].clone(),
            7 => BASE[pos][6].clone(),
            8 => BASE[pos][7].clone(),
            _ => Self {
                yplusx: Fe::new_one(),
                yminusx: Fe::new_one(),
                xy2d: Fe::new()
            }
        };
        if flag {
            GePrecomp {
                yplusx: r.yminusx.clone(),
                yminusx: r.yplusx.clone(),
                xy2d: Fe::neg(&r.xy2d)
            }
        } else {
            r
        }
    }
}

fn print_fe(a: &Fe) {
    a.0.iter().for_each(|i| {print!("{} ", i)});
    println!();
}

pub fn scalarmult_base(a: &[u8]) -> GeP3 {
    let mut e: [i8; 64] = [0; 64];
    for i in 0..32 {
        e[2 * i + 0] = ((a[i] >> 0) & 15) as i8;
        e[2 * i + 1] = ((a[i] >> 4) & 15) as i8;
    }

    let mut carry = 0;

    for i in 0..63 {
        e[i] += carry;
        carry = e[i] + 8;
        carry >>= 4;
        e[i] -= carry << 4;
    }
    e[63] += carry;
    let mut h = GeP3::new();
    for i in 0..32 {
        let t = GePrecomp::new(i, e[2 * i + 1] as u8);
        let r = madd(&h, &t);
        h = r.into();
    }

    let r = p3_dbl(&h);
    let s = r.into();
    let r = p2_dbl(&s);
    let s = r.into();
    let r = p2_dbl(&s);
    let s = r.into();
    let r = p2_dbl(&s);
    h = r.into();

    for i in 0..32 {
        let t = GePrecomp::new(i, e[2 * i] as u8);
        let r = madd(&h, &t);
        h = r.into();
    }
    h
}

pub fn double_scalarmult_vartime(a: &[u8], a_gep3: &GeP3, b: &[u8]) -> GeP2 {
    let mut aslide: [i8; 256] = [0; 256];
    let mut bslide: [i8; 256] = [0; 256];
    let mut ai: [GeCache; 8] = [GeCache::new(); 8];
    slide(&mut aslide, &a);
    slide(&mut bslide, &b);
    ai[0] = (*a_gep3).into();
    let mut t = p3_dbl(&a_gep3);
    let a2: GeP3 = t.into();
    t = add(&a2, &ai[0]);
    let mut u: GeP3 = t.into();
    ai[1] = u.into();

    t = add(&a2, &ai[1]);
    u = t.into();
    ai[2] = u.into();

    t = add(&a2, &ai[2]);
    u = t.into();
    ai[3] = u.into();

    t = add(&a2, &ai[3]);
    u = t.into();
    ai[4] = u.into();

    t = add(&a2, &ai[4]);
    u = t.into();
    ai[5] = u.into();

    t = add(&a2, &ai[5]);
    u = t.into();
    ai[6] = u.into();

    t = add(&a2, &ai[6]);
    u = t.into();
    ai[7] = u.into();
    let mut r = GeP2::new();
    let mut i = 255;
    while i as isize >= 0 {
        if aslide[i] > 0 || bslide[i] > 0 {
            break;
        }
        i -= 1;
    }
    while i as isize >= 0 {
        t = p2_dbl(&r);
        if aslide[i] > 0 {
            u = t.into();
            t = add(&u, &ai[(aslide[i] / 2) as usize]);
        } else if aslide[i] < 0 {
            u = t.into();
            t = sub(&u, &ai[(-aslide[i] / 2) as usize]);
        }

        if bslide[i] > 0 {
            u = t.into();
            t = madd(&u, &BI[(bslide[i] / 2) as usize]);
        } else if bslide[i] < 0 {
            u = t.into();
            t = msub(&u, &BI[(-bslide[i] / 2) as usize]);
        }
        r = t.into();
        i -= 1;
    }
    r
}

