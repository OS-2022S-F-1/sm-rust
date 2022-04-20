use crate::crypt::ed25519::precomp_data::{BASE, BI};
use super::fe::Fe;

struct GeP2 {
    pub x: Fe,
    pub y: Fe,
    pub z: Fe
}

struct GeP3 {
    pub x: Fe,
    pub y: Fe,
    pub z: Fe,
    pub t: Fe
}

struct GeP1P1 {
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

struct GeCache {
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
    fn from(p: GeP1P1) -> Self {
        Self {
            x: Fe::mul(&p.x, &p.x),
            y: Fe::mul(&p.y, &p.y),
            z: Fe::mul(&p.z, &p.z)

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
            t2d: Fe::mul(&p.2, &Fe::D2)
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
            yplusx: Fe(x[0] as [u32; 10]),
            yminusx: Fe(x[1] as [u32; 10]),
            xy2d: Fe(x[2] as [u32; 10])
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

fn slide(r: &mut [u8], a: &[u8]) {

    for i in 0..256 {
        r[i] = 1 & (a[i >> 3] >> (i & 7));
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
        r.x = Fe::mul(&p.z, &q.z);
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
    r.x = Fe::mul(&p.z, &q.z);
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
    let q: GeP2 = p.into();
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
        let babs = b - (((-(flag as u8)) & b) << 1);
        let mut r = match babs {
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


pub fn scalarmult_base(a: &[u8]) -> GeP3 {
    let mut e: [u8; 64] = [0; 64];
    for i in 0..32 {
        e[2 * i + 0] = (a[i] >> 0) & 15;
        e[2 * i + 1] = (a[i] >> 4) & 15;
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
    let mut r = GeP1P1::new();
    let mut s = GeP2::new();
    let mut t = GePrecomp {
        yplusx: Fe::new(),
        yminusx: Fe::new(),
        xy2d: Fe::new()
    };
    for i in 0..32 {
        t = GePrecomp::new(i, e[2 * i + 1]);
        r = madd(&h, &t);
        h = r.into();
    }

    r = p3_dbl(&h);
    s = r.into();
    r = p2_dbl(&s);
    s = r.into();
    r = p2_dbl(&s);
    s = r.into();
    r = p2_dbl(&s);
    h = r.into();

    for i in 0..32 {
        t = GePrecomp::new(i, e[2 * i]);
        r = madd(&h, &t);
        h = r.into();
    }
    h
}

pub fn double_scalarmult_vartime(a: &[u8], A: &GeP3, b: &[u8]) -> GeP2 {
    let mut aslide: [u8; 256] = [0; 256];
    let mut bslide: [u8; 256] = [0; 256];
    let mut Ai: [GeCache; 8] = [GeCache::new(); 8];
    slide(&aslide, &a);
    slide(&bslide, &b);
    Ai[0] = A.into();
    let mut t = p3_dbl(&A);
    let mut A2: GeP3 = t.into();
    t = add(&A2, &Ai[0]);
    let mut u: GeP3 = t.into();
    Ai[1] = u.into();

    t = add(&A2, &Ai[1]);
    u = t.into();
    Ai[2] = u.into();

    t = add(&A2, &Ai[2]);
    u = t.into();
    Ai[3] = u.into();

    t = add(&A2, &Ai[3]);
    u = t.into();
    Ai[4] = u.into();

    t = add(&A2, &Ai[4]);
    u = t.into();
    Ai[5] = u.into();

    t = add(&A2, &Ai[5]);
    u = t.into();
    Ai[6] = u.into();

    t = add(&A2, &Ai[6]);
    u = t.into();
    Ai[7] = u.into();
    let mut r = GeP2::new();
    let mut i = 255;
    while i >= 0 {
        if aslide[i] > 0 || bslide[i] > 0 {
            break;
        }
        i -= 1;
    }
    while i >= 0 {
        t = p2_dbl(&r);
        if aslide[i] > 0 {
            u = t.into();
            t = add(&u, &Ai[aslide[i] / 2]);
        } else if aslide[i] < 0 {
            u = t.into();
            t = sub(&u, &Ai[-aslide[i] / 2]);
        }

        if bslide[i] > 0 {
            u = t.into();
            t = madd(&u, &BI[bslide[i] / 2]);
        } else if bslide[i] < 0 {
            u = t.into();
            t = msub(&u, &BI[-bslide[i] / 2]);
        }
        r = t.into();
        i -= 1;
    }
    r
}

