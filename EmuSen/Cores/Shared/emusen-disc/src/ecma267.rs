//! A DVD's Data Frame as ECMA-267 (3rd edition, April 2001) defines it: 2,064 bytes of the Identification Data (§16.1),
//! its error detection code IED (§16.2), the copyright management bytes CPR_MAI (§16.3), 2,048 bytes of Main Data and
//! the EDC (§16.4). The physical sector numbers of §24 and §25. Written from the standard's definitions; the
//! scrambling, ECC blocks and modulation below the Data Frame (§17 to §22) are the drive's, not a reader's.

use crate::ecma130::{alpha, gf_mul};

/// A Data Frame's bytes, and where its fields lie (§16, figure 16).
pub const FRAME: usize = 2064;
pub const USER: usize = 2048;
pub const DATA_AT: usize = 12;
pub const EDC_AT: usize = 2060;
/// The first physical sector of the Data Zone (§24 table 5, §25).
pub const DATA_ZONE: u32 = 0x03_0000;
/// The sector number is the ID's low three bytes (§16.1).
pub const SECTOR_MASK: u32 = 0xFF_FFFF;

/// The Sector Information's zone type, bits b27 and b26 (§16.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Data,
    LeadIn,
    LeadOut,
    Middle,
}

/// The Sector Information byte, bits b31..b24 of the ID (§16.1, figure 18): the layer number in b24, read-only data
/// (b25 ZERO), the zone in b27 and b26, the reflectivity in b29 (ONE for 40 % or less, which a Dual Layer disk's
/// 18 % to 30 % is, §12.6), pit tracking and the CLV format (b30 and b31 ZERO).
pub fn sector_information(layer: u8, zone: Zone, low_reflectivity: bool) -> u8 {
    let zone = match zone {
        Zone::Data => 0,
        Zone::LeadIn => 1,
        Zone::LeadOut => 2,
        Zone::Middle => 3,
    };
    (layer & 1) | zone << 2 | (low_reflectivity as u8) << 5
}

/// The ID's four bytes, most significant first: the Sector Information, then the sector number in binary.
pub fn id(information: u8, sector: u32) -> [u8; 4] {
    let n = sector & SECTOR_MASK;
    [information, (n >> 16) as u8, (n >> 8) as u8, n as u8]
}

/// The IED's two check bytes (§16.2): I(x)·x² mod GE(x), I(x) the ID's bytes C0,0..C0,3 from x³ down, and
/// GE(x) = (x + α⁰)(x + α¹) over GF(2⁸) by x⁸ + x⁴ + x³ + x² + 1, the field ECMA-130's RSPC uses.
pub fn ied(id: [u8; 4]) -> [u8; 2] {
    let g1 = alpha(0) ^ alpha(1);
    let g0 = gf_mul(alpha(0), alpha(1));
    let mut r = [0u8; 2];
    for &c in &id {
        let f = c ^ r[0];
        r = [r[1] ^ gf_mul(f, g1), gf_mul(f, g0)];
    }
    r
}

/// G(x) = x³² + x³¹ + x⁴ + 1 without its x³² term, the most significant bit first (§16.4).
pub const EDC_POLY: u32 = 0x8000_0011;

const fn edc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = (i as u32) << 24;
        let mut k = 0;
        while k < 8 {
            c = if c & 0x8000_0000 != 0 { c << 1 ^ EDC_POLY } else { c << 1 };
            k += 1;
        }
        t[i] = c;
        i += 1;
    }
    t
}

static EDC_TABLE: [u32; 256] = edc_table();

/// The EDC of §16.4: the bytes as one bit field from the first byte's most significant bit, times x³², mod G(x).
pub fn edc(data: &[u8]) -> u32 {
    data.iter().fold(0, |c, &b| EDC_TABLE[((c >> 24) as u8 ^ b) as usize] ^ c << 8)
}

/// A whole Data Frame: the ID of `sector` with `information`, its IED, `cpr_mai` and `user`, the EDC over the 2,060
/// bytes before it, stored most significant byte first so that its msb is the field's b31.
pub fn frame(information: u8, sector: u32, cpr_mai: [u8; 6], user: &[u8], out: &mut [u8; FRAME]) {
    let i = id(information, sector);
    out[..4].copy_from_slice(&i);
    out[4..6].copy_from_slice(&ied(i));
    out[6..12].copy_from_slice(&cpr_mai);
    out[DATA_AT..EDC_AT].copy_from_slice(&user[..USER]);
    let e = edc(&out[..EDC_AT]);
    out[EDC_AT..].copy_from_slice(&e.to_be_bytes());
}

/// Whether a frame's IED and EDC both hold.
pub fn holds(f: &[u8; FRAME]) -> bool {
    let i = [f[0], f[1], f[2], f[3]];
    ied(i) == [f[4], f[5]] && edc(&f[..EDC_AT]).to_be_bytes() == f[EDC_AT..]
}

/// How a Dual Layer disk numbers its second layer (§25): Parallel Track Path, each layer from (030000) again, or
/// Opposite Track Path, layer 1 from layer 0's last sector number inverted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackPath {
    Parallel,
    Opposite,
}

/// The physical sector number of logical sector `lba` (counted from the Data Zone's first sector over both layers),
/// on a disk whose layer 1 begins at logical sector `layer_break`: (030000) + lba on layer 0; on layer 1 by `path`.
pub fn physical(lba: u32, layer_break: Option<u32>, path: TrackPath) -> (u8, u32) {
    match layer_break {
        Some(b) if lba >= b => {
            let first = match path {
                TrackPath::Parallel => DATA_ZONE,
                TrackPath::Opposite => !(DATA_ZONE + b - 1) & SECTOR_MASK,
            };
            (1, first.wrapping_add(lba - b) & SECTOR_MASK)
        }
        _ => (0, (DATA_ZONE + lba) & SECTOR_MASK),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A polynomial over GF(2⁸), highest power first, evaluated at x.
    fn eval(p: &[u8], x: u8) -> u8 {
        p.iter().fold(0, |a, &c| gf_mul(a, x) ^ c)
    }

    #[test]
    fn the_ied_makes_the_six_bytes_a_codeword_of_ge() {
        for i in [[0u8, 3, 0, 0], [0x21, 0x0C, 0xFF, 0xEF], [1, 0xFC, 0xFF, 0xF0], [0xFF; 4], [0; 4]] {
            let c = [i[0], i[1], i[2], i[3], ied(i)[0], ied(i)[1]];
            assert_eq!((eval(&c, alpha(0)), eval(&c, alpha(1))), (0, 0), "{i:02X?}: GE's roots α⁰ and α¹ are the codeword's");
        }
        assert_eq!(ied([0; 4]), [0, 0]);
        assert_ne!(ied([0, 3, 0, 0]), ied([1, 3, 0, 0]), "the layer bit moves the IED");
    }

    #[test]
    fn the_edc_is_the_remainder_of_a_bitwise_division_by_g() {
        let data: Vec<u8> = (0..EDC_AT as u32).map(|k| (k * 7 + 3) as u8).collect();
        let mut r: u64 = 0;
        let bits = data.iter().flat_map(|&b| (0..8).rev().map(move |k| (b >> k) & 1)).chain(std::iter::repeat_n(0, 32));
        for bit in bits {
            r = r << 1 | bit as u64;
            if r & 1 << 32 != 0 {
                r ^= 1 << 32 | EDC_POLY as u64;
            }
        }
        assert_eq!(edc(&data), r as u32);
        let mut f = [0u8; FRAME];
        frame(0, DATA_ZONE + 9, [0; 6], &data[12..12 + USER], &mut f);
        assert!(holds(&f));
        assert_eq!(edc(&f), 0, "a frame with its EDC divides by G(x)");
        f[700] ^= 0x10;
        assert!(!holds(&f));
    }

    #[test]
    fn the_sector_information_and_numbers_are_section_16_and_25s() {
        assert_eq!(sector_information(0, Zone::Data, false), 0x00);
        assert_eq!(sector_information(1, Zone::Data, true), 0x21);
        assert_eq!(sector_information(0, Zone::LeadOut, false), 0x08);
        assert_eq!(sector_information(1, Zone::Middle, true), 0x2D);
        assert_eq!(id(0x21, 0xFC_FFF0), [0x21, 0xFC, 0xFF, 0xF0]);
        assert_eq!(physical(0, None, TrackPath::Opposite), (0, 0x03_0000));
        let b = 0x2000;
        assert_eq!(physical(b - 1, Some(b), TrackPath::Opposite), (0, 0x03_1FFF));
        assert_eq!(physical(b, Some(b), TrackPath::Opposite), (1, 0xFC_E000), "layer 0's last, 031FFFh, inverted");
        assert_eq!(physical(b + 5, Some(b), TrackPath::Opposite), (1, 0xFC_E005));
        assert_eq!(physical(b + 5, Some(b), TrackPath::Parallel), (1, 0x03_0005));
        assert_eq!(!(DATA_ZONE + b - 1) & SECTOR_MASK & 15, 0, "§25: the inverse of layer 0's last is a multiple of 16");
    }
}
