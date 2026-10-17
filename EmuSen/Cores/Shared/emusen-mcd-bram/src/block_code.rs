//! §6 of Nephrite_Bram.md: 32 data bytes in 64 stored bytes: a CRC-16, six-bit symbols, eight Reed-Solomon codewords
//! over GF(64), and a column check in each byte's low two bits.

/// Each codeword's eight symbols: six of data and its two parity symbols.
const ORDER: [[usize; 8]; 8] = {
    let mut order = [[0; 8]; 8];
    let mut k = 0;
    while k < 8 {
        order[k] = [k, k + 9, k + 18, k + 27, k + 36, if k < 3 { 45 + k } else { 9 * k - 19 }, 48 + k, 56 + k];
        k += 1;
    }
    order
};

/// The column check's contribution of each of a symbol's six bits.
const T: [u32; 6] = [0x000E, 0x003E, 0x00FE, 0x03FE, 0x0FFE, 0x3FFE];
const INVERSE_3: u32 = 0x3E;

/// A symbol times α in GF(64) over x⁶ + x + 1.
fn alpha(x: u32) -> u32 {
    if (x << 1) & 0x40 != 0 { (x << 1) ^ 0x43 } else { x << 1 }
}

fn multiply(x: u32, y: u32) -> u32 {
    let mut r = 0;
    for b in (0..6).rev() {
        r = alpha(r);
        if (y >> b) & 1 != 0 {
            r ^= x;
        }
    }
    r
}

/// CRC-16 with the polynomial 1021h from zero.
pub fn crc(data: &[u8]) -> u16 {
    let mut c: u32 = 0;
    for &b in data {
        c ^= (b as u32) << 8;
        for _ in 0..8 {
            c = if c & 0x8000 != 0 { ((c << 1) ^ 0x1021) & 0xFFFF } else { (c << 1) & 0xFFFF };
        }
    }
    c as u16
}

/// The 32 bytes of `u` stored as the 64 of `stored`.
pub fn encode(u: &[u8], stored: &mut [u8]) {
    let mut m = [0u8; 36];
    let c = crc(&u[..32]);
    m[..2].copy_from_slice(&c.to_be_bytes());
    m[2..34].copy_from_slice(&u[..32]);
    m[34..].copy_from_slice(&(!c).to_be_bytes());
    let mut s = [0u32; 64];
    for g in 0..12 {
        let bits = (m[3 * g] as u32) << 16 | (m[3 * g + 1] as u32) << 8 | m[3 * g + 2] as u32;
        for k in 0..4 {
            s[4 * g + k] = (bits >> (18 - 6 * k)) & 63;
        }
    }
    for o in ORDER {
        let (mut s0, mut h) = (0, 0);
        for &i in &o[..6] {
            s0 ^= s[i];
            h = alpha(h) ^ s[i];
        }
        let p0 = multiply(s0 ^ alpha(alpha(h)), INVERSE_3);
        s[o[6]] = p0;
        s[o[7]] = s0 ^ p0;
    }
    for c in 0..8 {
        let mut r: u32 = 0;
        for row in 0..8 {
            let top = r >> 14;
            r = ((r << 2) & 0xFFFF) ^ (top << 8) ^ (top << 6) ^ (top << 4) ^ top;
            for (b, t) in T.iter().enumerate() {
                if (s[8 * row + c] >> b) & 1 != 0 {
                    r ^= t;
                }
            }
        }
        for row in 0..8 {
            stored[8 * row + c] = ((s[8 * row + c] << 2) | ((r >> (14 - 2 * row)) & 3)) as u8;
        }
    }
}

/// The 32 data bytes of `stored` into `u`, one symbol error a codeword corrected, the low bits and the CRC not used
/// (§6.7): 0 whole, 1 corrected, -1 a codeword past correcting; `u` is written in every case.
pub fn decode(stored: &[u8], u: &mut [u8]) -> i32 {
    let mut s: [u32; 64] = core::array::from_fn(|i| (stored[i] >> 2) as u32);
    let mut result = 0;
    for o in ORDER {
        let (mut s0, mut s1) = (0, 0);
        for &i in &o {
            s0 ^= s[i];
            s1 = alpha(s1) ^ s[i];
        }
        if s0 | s1 == 0 {
            continue;
        }
        let mut at = None;
        let mut t = s0;
        if s0 != 0 && s1 != 0 {
            for p in 0..8 {
                if t == s1 {
                    at = Some(7 - p);
                    break;
                }
                t = alpha(t);
            }
        }
        match at {
            None => result = -1,
            Some(at) => {
                s[o[at]] ^= s0;
                if result == 0 {
                    result = 1;
                }
            }
        }
    }
    let mut m = [0u8; 36];
    for g in 0..12 {
        let bits = s[4 * g] << 18 | s[4 * g + 1] << 12 | s[4 * g + 2] << 6 | s[4 * g + 3];
        m[3 * g] = (bits >> 16) as u8;
        m[3 * g + 1] = (bits >> 8) as u8;
        m[3 * g + 2] = bits as u8;
    }
    u[..32].copy_from_slice(&m[2..34]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_decodes_whole_then_corrects_one_symbol_and_refuses_two() {
        let u: [u8; 32] = core::array::from_fn(|i| (i as u8).wrapping_mul(37).wrapping_add(5));
        let mut stored = [0u8; 64];
        encode(&u, &mut stored);
        let mut back = [0u8; 32];
        assert_eq!(decode(&stored, &mut back), 0);
        assert_eq!(back, u);
        stored[9] ^= 0x40;
        assert_eq!(decode(&stored, &mut back), 1);
        assert_eq!(back, u);
        stored[18] ^= 0x80;
        assert_eq!(decode(&stored, &mut back), -1);
    }

    #[test]
    fn zeros_store_as_the_pages_vector() {
        // Nephrite_Bram.md §6.5's all-zero block, which the open BIOS's tests also hold.
        let zeros = "00 00 00 00 00 00 00 00 03 03 03 00 00 03 03 03 00 00 00 00 00 00 00 00 03 03 03 00 00 03 00 00 \
                     00 03 03 00 00 00 03 03 00 03 03 00 00 3C FF FF 44 09 09 00 00 00 00 00 78 F6 F6 00 00 00 03 03";
        let want: [u8; 64] = {
            let mut w = [0u8; 64];
            for (i, b) in zeros.split_whitespace().enumerate() {
                w[i] = u8::from_str_radix(b, 16).unwrap();
            }
            w
        };
        let mut stored = [0u8; 64];
        encode(&[0u8; 32], &mut stored);
        assert_eq!(stored, want);
    }
}
