//! BPS, from "BPS File Format Specification" by byuu (Near), version 1.0 of 27 August 2012, romhacking.net document 746,
//! `bps_spec.md` as the Internet Archive holds the download, read 2026-10-25: the marker `BPS1`; the source's and
//! target's sizes and the metadata's length as variable-length numbers, then the metadata; actions to twelve bytes from
//! the end, each a number holding the action in its low two bits and the length less one above them; then the CRC-32s
//! of the source, the target and the patch, the last over every byte before it. The checksums are little-endian, as
//! vidua's notes say (vidua.readthedocs.io, "BPS"); the specification does not.
//!
//! The four actions as the specification gives them: SourceRead copies from the source at the output's own offset;
//! TargetRead takes bytes from the patch; SourceCopy moves its own source offset by a signed number (the sign in the
//! low bit) and copies from there; TargetCopy does the same over the target already written, a byte at a time, so that a
//! copy may overlap what it writes. No offset may leave its file, nor a TargetCopy reach the byte being written.

use crate::{damaged, footer, number, Checked, Format, Refusal, MAX_IMAGE};

/// The header and footer read, the metadata passed over, and the patch's own checksum checked.
pub(crate) fn header(patch: &[u8]) -> Result<Checked, Refusal> {
    let (end, source_crc32, target_crc32) = footer(patch, b"BPS1", 3)?;
    let mut at = 4;
    let source_size = number(patch, &mut at, end)?;
    let target_size = number(patch, &mut at, end)?;
    let metadata = number(patch, &mut at, end)?;
    let body = usize::try_from(metadata).ok().and_then(|m| at.checked_add(m)).filter(|&b| b <= end).ok_or_else(|| damaged(format!("the BPS patch's metadata of {metadata} bytes runs into its checksums")))?;
    Ok(Checked { source_size, target_size, source_crc32, target_crc32, body, footer: end })
}

/// An offset moved by a signed number, its sign in the low bit; none where it would leave zero or 64 bits.
fn moved(offset: usize, data: u64) -> Option<usize> {
    let by = usize::try_from(data >> 1).ok()?;
    if data & 1 != 0 { offset.checked_sub(by) } else { offset.checked_add(by) }
}

/// The patch applied: the source checked, the target written once in order by the actions, and checked against the
/// target's size and CRC-32.
pub fn apply(source: &[u8], patch: &[u8]) -> Result<Vec<u8>, Refusal> {
    let h = header(patch)?;
    h.source(Format::Bps, source)?;
    if h.target_size > MAX_IMAGE {
        return Err(Refusal::TooLarge(h.target_size));
    }
    let size = h.target_size as usize;
    let mut out: Vec<u8> = Vec::with_capacity(size);
    let (mut source_offset, mut target_offset) = (0usize, 0usize);
    let mut at = h.body;
    while at < h.footer {
        let data = number(patch, &mut at, h.footer)?;
        let length = usize::try_from(data >> 2).ok().and_then(|l| l.checked_add(1)).ok_or_else(|| damaged("a BPS action's length is past any image"))?;
        let written = out.len();
        if length > size - written {
            return Err(damaged(format!("a BPS action at byte {written} writes {length} bytes, past the {size} of its target")));
        }
        match data & 3 {
            0 => {
                let from = source.get(written..written + length).ok_or_else(|| damaged(format!("a BPS SourceRead at byte {written} reads past the source's {} bytes", source.len())))?;
                out.extend_from_slice(from);
            }
            1 => {
                let from = patch.get(at..at + length).filter(|_| at + length <= h.footer).ok_or_else(|| damaged("a BPS TargetRead runs into the patch's checksums"))?;
                out.extend_from_slice(from);
                at += length;
            }
            2 => {
                let by = number(patch, &mut at, h.footer)?;
                source_offset = moved(source_offset, by).filter(|&o| o.checked_add(length).is_some_and(|e| e <= source.len())).ok_or_else(|| damaged(format!("a BPS SourceCopy at byte {written} reads outside the source's {} bytes", source.len())))?;
                out.extend_from_slice(&source[source_offset..source_offset + length]);
                source_offset += length;
            }
            _ => {
                let by = number(patch, &mut at, h.footer)?;
                target_offset = moved(target_offset, by).filter(|&o| o < written).ok_or_else(|| damaged(format!("a BPS TargetCopy at byte {written} reads target bytes not yet written")))?;
                for _ in 0..length {
                    let b = out[target_offset];
                    out.push(b);
                    target_offset += 1;
                }
            }
        }
    }
    if out.len() != size {
        return Err(damaged(format!("the BPS patch wrote {} bytes of a target of {size}", out.len())));
    }
    h.target(Format::Bps, &out)?;
    Ok(out)
}
