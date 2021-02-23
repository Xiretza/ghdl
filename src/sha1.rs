use std::convert::TryInto;
use std::fmt;

const H0: u32 = 0x6745_2301;
const H1: u32 = 0xEFCD_AB89;
const H2: u32 = 0x98BA_DCFE;
const H3: u32 = 0x1032_5476;
const H4: u32 = 0xC3D2_E1F0;

const K0: u32 = 0x5A82_7999;
const K1: u32 = 0x6ED9_EBA1;
const K2: u32 = 0x8F1B_BCDC;
const K3: u32 = 0xCA62_C1D6;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct State {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:x} {:x} {:x} {:x} {:x}",
            self.a, self.b, self.c, self.d, self.e
        )
    }
}

fn rot(v: u32, amount: usize) -> u32 {
    (v << amount) | (v >> (32 - amount))
}

fn p0(v: State, w: [u32; 16], t: usize) -> State {
    let temp = rot(v.a, 5)
        .wrapping_add((v.b & v.c) | (!v.b & v.d))
        .wrapping_add(v.e)
        .wrapping_add(w[t])
        .wrapping_add(K0);
    State {
        a: temp,
        b: v.a,
        c: rot(v.b, 30),
        d: v.c,
        e: v.d,
    }
}

fn p0b(v: State, w: &mut [u32; 16], t: usize) -> State {
    w[t] = rot(
        w[(t + 13) & 15] ^ w[(t + 8) & 15] ^ w[(t + 2) & 15] ^ w[t],
        1,
    );
    let temp = rot(v.a, 5)
        .wrapping_add((v.b & v.c) | (!v.b & v.d))
        .wrapping_add(v.e)
        .wrapping_add(w[t])
        .wrapping_add(K0);
    State {
        a: temp,
        b: v.a,
        c: rot(v.b, 30),
        d: v.c,
        e: v.d,
    }
}

fn p1(v: State, w: &mut [u32; 16], t: usize) -> State {
    w[t] = rot(
        w[(t + 13) & 15] ^ w[(t + 8) & 15] ^ w[(t + 2) & 15] ^ w[t],
        1,
    );
    let temp = rot(v.a, 5)
        .wrapping_add(v.b ^ v.c ^ v.d)
        .wrapping_add(v.e)
        .wrapping_add(w[t])
        .wrapping_add(K1);
    State {
        a: temp,
        b: v.a,
        c: rot(v.b, 30),
        d: v.c,
        e: v.d,
    }
}

fn p2(v: State, w: &mut [u32; 16], t: usize) -> State {
    w[t] = rot(
        w[(t + 13) & 15] ^ w[(t + 8) & 15] ^ w[(t + 2) & 15] ^ w[t],
        1,
    );
    let temp = rot(v.a, 5)
        .wrapping_add((v.b & v.c) | (v.b & v.d) | (v.c & v.d))
        .wrapping_add(v.e)
        .wrapping_add(w[t & 15])
        .wrapping_add(K2);
    State {
        a: temp,
        b: v.a,
        c: rot(v.b, 30),
        d: v.c,
        e: v.d,
    }
}

fn p3(v: State, w: &mut [u32; 16], t: usize) -> State {
    w[t] = rot(
        w[(t + 13) & 15] ^ w[(t + 8) & 15] ^ w[(t + 2) & 15] ^ w[t],
        1,
    );
    let temp = rot(v.a, 5)
        .wrapping_add(v.b ^ v.c ^ v.d)
        .wrapping_add(v.e)
        .wrapping_add(w[t])
        .wrapping_add(K3);
    State {
        a: temp,
        b: v.a,
        c: rot(v.b, 30),
        d: v.c,
        e: v.d,
    }
}

fn process_block(blk: &mut State, dat: [u32; 16]) {
    let mut w: [u32; 16] = dat;

    // a. Divide Mi into 16 words W0, W1, ..., W15 where W0 is the
    // left-most word.

    // b. For t = 16 to 79...

    // c. Let A=H0, ...
    let mut v = *blk;

    // d. For t = 0 to 79 do ...
    //   0..=19
    v = p0(v, w, 0);
    v = p0(v, w, 1);
    v = p0(v, w, 2);
    v = p0(v, w, 3);
    v = p0(v, w, 4);
    v = p0(v, w, 5);
    v = p0(v, w, 6);
    v = p0(v, w, 7);
    v = p0(v, w, 8);
    v = p0(v, w, 9);
    v = p0(v, w, 10);
    v = p0(v, w, 11);
    v = p0(v, w, 12);
    v = p0(v, w, 13);
    v = p0(v, w, 14);
    v = p0(v, w, 15);
    v = p0b(v, &mut w, 0);
    v = p0b(v, &mut w, 1);
    v = p0b(v, &mut w, 2);
    v = p0b(v, &mut w, 3);
    //  20..=39
    v = p1(v, &mut w, 4);
    v = p1(v, &mut w, 5);
    v = p1(v, &mut w, 6);
    v = p1(v, &mut w, 7);
    v = p1(v, &mut w, 8);
    v = p1(v, &mut w, 9);
    v = p1(v, &mut w, 10);
    v = p1(v, &mut w, 11);
    v = p1(v, &mut w, 12);
    v = p1(v, &mut w, 13);
    v = p1(v, &mut w, 14);
    v = p1(v, &mut w, 15);
    v = p1(v, &mut w, 0);
    v = p1(v, &mut w, 1);
    v = p1(v, &mut w, 2);
    v = p1(v, &mut w, 3);
    v = p1(v, &mut w, 4);
    v = p1(v, &mut w, 5);
    v = p1(v, &mut w, 6);
    v = p1(v, &mut w, 7);
    //  40..=59
    v = p2(v, &mut w, 8);
    v = p2(v, &mut w, 9);
    v = p2(v, &mut w, 10);
    v = p2(v, &mut w, 11);
    v = p2(v, &mut w, 12);
    v = p2(v, &mut w, 13);
    v = p2(v, &mut w, 14);
    v = p2(v, &mut w, 15);
    v = p2(v, &mut w, 0);
    v = p2(v, &mut w, 1);
    v = p2(v, &mut w, 2);
    v = p2(v, &mut w, 3);
    v = p2(v, &mut w, 4);
    v = p2(v, &mut w, 5);
    v = p2(v, &mut w, 6);
    v = p2(v, &mut w, 7);
    v = p2(v, &mut w, 8);
    v = p2(v, &mut w, 9);
    v = p2(v, &mut w, 10);
    v = p2(v, &mut w, 11);
    //  60..=79
    v = p3(v, &mut w, 12);
    v = p3(v, &mut w, 13);
    v = p3(v, &mut w, 14);
    v = p3(v, &mut w, 15);
    v = p3(v, &mut w, 0);
    v = p3(v, &mut w, 1);
    v = p3(v, &mut w, 2);
    v = p3(v, &mut w, 3);
    v = p3(v, &mut w, 4);
    v = p3(v, &mut w, 5);
    v = p3(v, &mut w, 6);
    v = p3(v, &mut w, 7);
    v = p3(v, &mut w, 8);
    v = p3(v, &mut w, 9);
    v = p3(v, &mut w, 10);
    v = p3(v, &mut w, 11);
    v = p3(v, &mut w, 12);
    v = p3(v, &mut w, 13);
    v = p3(v, &mut w, 14);
    v = p3(v, &mut w, 15);

    // e. Let...
    blk.a = blk.a.wrapping_add(v.a);
    blk.b = blk.b.wrapping_add(v.b);
    blk.c = blk.c.wrapping_add(v.c);
    blk.d = blk.d.wrapping_add(v.d);
    blk.e = blk.e.wrapping_add(v.e);
}

fn to_u32x16(v: &[u8]) -> [u32; 16] {
    let mut res: [u32; 16] = [0; 16];
    for i in 0..16 {
        res[i] = u32::from_be_bytes(v[i * 4..(i + 1) * 4].try_into().unwrap());
    }
    res
}

pub fn sha1(dat: &[u8]) -> State {
    //  Init
    let mut res = State {
        a: H0,
        b: H1,
        c: H2,
        d: H3,
        e: H4,
    };

    let len = dat.len();
    let mut off: usize = 0;

    while off + 64 <= len {
        process_block(&mut res, to_u32x16(&dat[off..off + 64]));
        off += 64;
    }
    let mut pad: [u8; 64] = [0; 64];
    pad[..(len - off)].clone_from_slice(&dat[off..len]);
    pad[len - off] = 0x80;
    let mut padu32 = to_u32x16(&pad[..]);

    if len - off >= 56 {
        process_block(&mut res, padu32);
        padu32 = [0; 16];
    }

    let len_bits = (len as u64) * 8;

    padu32[14] = (len_bits >> 32) as u32;
    padu32[15] = (len_bits & 0xffff_ffff) as u32;
    process_block(&mut res, padu32);

    res
}

pub fn sha1_str(s: &str) -> State {
    sha1(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn test1() {
        let r = super::sha1_str("hello world");
        println!("{:?}", r);

        let r = super::sha1_str("abc");
        assert_eq!(
            r,
            State {
                a: 0xA9993E36,
                b: 0x4706816A,
                c: 0xBA3E2571,
                d: 0x7850C26C,
                e: 0x9CD0D89D
            }
        );
        println!("{:?}", r);

        let r = super::sha1_str("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            r,
            State {
                a: 0x84983E44,
                b: 0x1C3BD26E,
                c: 0xBAAE4AA1,
                d: 0xF95129E5,
                e: 0xE54670F1
            }
        );
        println!("{:?}", r);

        let r = super::sha1_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(
            r,
            State {
                a: 0x0098ba82,
                b: 0x4b5c1642,
                c: 0x7bd7a112,
                d: 0x2a5a442a,
                e: 0x25ec644d
            }
        );
        println!("{:?}", r);

        let r = super::sha1_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(
            r,
            State {
                a: 0xc1c8bbdc,
                b: 0x22796e28,
                c: 0xc0e15163,
                d: 0xd20899b6,
                e: 0x5621d65a
            }
        );
        println!("{:?}", r);

        let r = super::sha1_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(
            r,
            State {
                a: 0xb05d71c6,
                b: 0x4979cb95,
                c: 0xfa74a33c,
                d: 0xdb31a40d,
                e: 0x258ae02e
            }
        );
        println!("{:?}", r);

        let r = super::sha1(&[0; 64]);
        assert_eq!(
            r,
            State {
                a: 0xC8D7D0EF,
                b: 0x0EEDFA82,
                c: 0xD2EA1AA5,
                d: 0x92845B9A,
                e: 0x6D4B02B7
            }
        );

        let r = super::sha1(&[0; 55]);
        assert_eq!(
            r,
            State {
                a: 0x8e8832c6,
                b: 0x42a6a38c,
                c: 0x74c17fc9,
                d: 0x2ccedc26,
                e: 0x6c108e6c
            }
        );

        let r = super::sha1(&[0; 56]);
        assert_eq!(
            r,
            State {
                a: 0x9438e360,
                b: 0xf578e12c,
                c: 0x0e0e8ed2,
                d: 0x8e2c125c,
                e: 0x1cefee16
            }
        );
    }
}
