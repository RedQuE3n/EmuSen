//! UPS, from "UPS File Format Specification" of 18 April 2008 (individual.utoronto.ca/dmeunier/ups-spec.pdf, as the
//! Internet Archive holds it, read 2026-10-25): the marker `UPS1`; the input's and the output's exact sizes as
//! variable-length numbers; blocks to twelve bytes from the end, each a relative offset to the next differing byte,
//! then the XOR of input and output bytes until a terminating zero, the input read as zero past its end; then the
//! CRC-32s of the input, the output and the patch, the last over every byte before it.
//!
//! Decided here (EmuSen_GamePackages.md §15.1): the terminating zero stands for one more byte, equal in both files, so
//! the position moves past it, which is the reading in which a block ends at the first byte the files share. The
//! specification makes the format two-way; an image that is already the output is refused rather than unpatched,
//! since a package's list only ever patches forward.

use crate::{damaged, footer, number, Checked, Format, Refusal, MAX_IMAGE};

/// The header and footer read and the patch's own checksum checked.
pub(crate) fn header(patch: &[u8]) -> Result<Checked, Refusal> {
    let (end, source_crc32, target_crc32) = footer(patch, b"UPS1", 2)?;
    let mut at = 4;
    let source_size = number(patch, &mut at, end)?;
    let target_size = number(patch, &mut at, end)?;
    Ok(Checked { source_size, target_size, source_crc32, target_crc32, body: at, footer: end })
}

/// The patch applied: the source checked, copied once into an image of the output's size, and each block's bytes
/// XORed into it; the result checked against the output's CRC-32.
pub fn apply(source: &[u8], patch: &[u8]) -> Result<Vec<u8>, Refusal> {
    let h = header(patch)?;
    h.source(Format::Ups, source)?;
    if h.target_size > MAX_IMAGE {
        return Err(Refusal::TooLarge(h.target_size));
    }
    let size = h.target_size as usize;
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&source[..source.len().min(size)]);
    out.resize(size, 0);
    let mut at = h.body;
    let mut pos = 0usize;
    while at < h.footer {
        let skip = number(patch, &mut at, h.footer)?;
        pos = usize::try_from(skip).ok().and_then(|s| pos.checked_add(s)).ok_or_else(|| damaged("a UPS block's offset is past any image"))?;
        loop {
            if at >= h.footer {
                return Err(damaged("a UPS block runs into the patch's checksums"));
            }
            let x = patch[at];
            at += 1;
            if x == 0 {
                pos += 1;
                break;
            }
            if pos >= size {
                return Err(damaged(format!("a UPS block writes at byte {pos}, past the {size} bytes of its output")));
            }
            out[pos] ^= x;
            pos += 1;
        }
    }
    h.target(Format::Ups, &out)?;
    Ok(out)
}
