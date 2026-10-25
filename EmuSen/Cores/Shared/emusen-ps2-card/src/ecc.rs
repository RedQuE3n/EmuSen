//! The spare bytes' error correction (Ross Ridge, "PlayStation 2 Memory Card File System", "Error Correction Code"): a
//! page's 512 bytes in four chunks of 128, each with a 20-bit Hamming code in three bytes, the four codes in order in the
//! spare area's first twelve bytes. The first byte holds the column parities, the even groups in the low nibble and the
//! odd in the high; the second and third the line parities' even and odd groups. Each parity is stored complemented, a
//! bit set where its group's parity is even, and the unused bits 3 and 7 of the first byte and 7 of the others clear, so
//! that an erased chunk's code reads 77h 7Fh 7Fh (Sapphire_Disputes.md D-226). Hardware/Shared/EmuSen_MediumFormats.md §6.3.

/// The chunk a code covers.
pub const CHUNK: usize = 128;

fn parity(b: u8) -> u8 {
    (b.count_ones() & 1) as u8
}

/// The code of one 128-byte chunk: [columns, even lines, odd lines].
pub fn chunk(data: &[u8]) -> [u8; 3] {
    let mut column = 0u8;
    let (mut even, mut odd) = (0u8, 0u8);
    for (i, &b) in data.iter().enumerate().take(CHUNK) {
        column ^= b;
        if parity(b) != 0 {
            for k in 0..7 {
                if i >> k & 1 == 0 { even ^= 1 << k } else { odd ^= 1 << k }
            }
        }
    }
    // Column group k's even half holds the bits whose index has bit k clear, its odd half those with it set.
    let groups = [(0x55u8, 0xAAu8), (0x33, 0xCC), (0x0F, 0xF0)];
    let mut cp = 0u8;
    for (k, &(e, o)) in groups.iter().enumerate() {
        cp |= parity(column & e) << k;
        cp |= parity(column & o) << (k + 4);
    }
    [!cp & 0x77, !even & 0x7F, !odd & 0x7F]
}

/// The twelve code bytes of a 512-byte page.
pub fn page(data: &[u8; 512]) -> [u8; 12] {
    let mut out = [0u8; 12];
    for c in 0..4 {
        out[3 * c..3 * c + 3].copy_from_slice(&chunk(&data[c * CHUNK..(c + 1) * CHUNK]));
    }
    out
}

/// What checking a chunk against its stored code found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Clean,
    /// One data bit was wrong and has been put right: its byte and bit.
    Corrected { byte: usize, bit: u8 },
    /// One bit of the stored code itself was wrong; the data stand.
    CodeBit,
    Uncorrectable,
}

/// Checks a chunk against its stored code, correcting a single wrong data bit in place.
pub fn correct(data: &mut [u8], stored: &[u8]) -> Check {
    let now = chunk(data);
    let s = [(now[0] ^ stored[0]) & 0x77, (now[1] ^ stored[1]) & 0x7F, (now[2] ^ stored[2]) & 0x7F];
    if s == [0, 0, 0] {
        return Check::Clean;
    }
    let weight: u32 = s.iter().map(|b| b.count_ones()).sum();
    let (lo, hi) = (s[0] & 7, s[0] >> 4 & 7);
    if lo ^ hi == 7 && s[1] ^ s[2] == 0x7F {
        let byte = s[2] as usize;
        let bit = hi;
        if byte < data.len() {
            data[byte] ^= 1 << bit;
            return Check::Corrected { byte, bit };
        }
    }
    if weight == 1 { Check::CodeBit } else { Check::Uncorrectable }
}

/// Checks and corrects a 512-byte page against its spare bytes; false when any chunk cannot be put right.
pub fn correct_page(data: &mut [u8], spare: &[u8]) -> bool {
    (0..4).all(|c| correct(&mut data[c * CHUNK..(c + 1) * CHUNK], &spare[3 * c..3 * c + 3]) != Check::Uncorrectable)
}
