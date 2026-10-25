//! IPS, from zerosoft's "IPS file format" (zerosoft.zophar.net) and SnesLab's "IPS file format" page, read
//! 2026-10-25: the marker `PATCH`; records of a three-byte offset and a two-byte size, both big-endian, then that many
//! bytes, or, where the size is zero, a two-byte run length and the one byte repeated; the marker `EOF`. After it a
//! patch may carry three bytes more, big-endian, the size the patched image is cut to (the truncation extension SnesLab
//! and the nesdev thread "Patch formats" name). An IPS patch records no checksum and no sizes.
//!
//! Decided here, where the descriptions are silent (EmuSen_GamePackages.md §15.1): a record past the source's end
//! lengthens the image, the bytes between left zero; a record whose offset reads as the three bytes `EOF` (0x454F46)
//! is the end marker, as every reader must take it; anything after `EOF` other than nothing or exactly three bytes is
//! refused; and a truncation longer than the patched image is refused, since the extension only shortens.

use crate::{damaged, Refusal, MAX_IMAGE};

/// The records' furthest end, and the truncation where the patch has one.
pub struct Scan {
    pub extent: u64,
    pub truncate: Option<u64>,
}

fn be(p: &[u8], at: usize, n: usize) -> u64 {
    p[at..at + n].iter().fold(0u64, |v, &b| (v << 8) | b as u64)
}

/// Every record visited in order as `(offset, Some(bytes))` or, for a run, `(offset, None)` with its length and byte.
fn records(patch: &[u8], mut each: impl FnMut(u64, Record)) -> Result<Option<u64>, Refusal> {
    if !patch.starts_with(b"PATCH") {
        return Err(Refusal::NotAPatch);
    }
    let len = patch.len();
    let mut at = 5;
    loop {
        if at + 3 > len {
            return Err(damaged("the IPS patch ends before its EOF marker"));
        }
        if &patch[at..at + 3] == b"EOF" {
            at += 3;
            return match len - at {
                0 => Ok(None),
                3 => Ok(Some(be(patch, at, 3))),
                n => Err(damaged(format!("{n} bytes follow the IPS patch's EOF marker, and only a truncation of exactly three may"))),
            };
        }
        let offset = be(patch, at, 3);
        if at + 5 > len {
            return Err(damaged(format!("the IPS record at byte {at} ends inside its size")));
        }
        let size = be(patch, at + 3, 2) as usize;
        at += 5;
        if size == 0 {
            if at + 3 > len {
                return Err(damaged(format!("the IPS run at byte {} ends inside its length", at - 5)));
            }
            let run = be(patch, at, 2);
            if run == 0 {
                return Err(damaged(format!("the IPS run at byte {} has a length of zero", at - 5)));
            }
            each(offset, Record::Run(run, patch[at + 2]));
            at += 3;
        } else {
            if at + size > len {
                return Err(damaged(format!("the IPS record at byte {} holds {size} bytes and the patch ends first", at - 5)));
            }
            each(offset, Record::Bytes(&patch[at..at + size]));
            at += size;
        }
    }
}

enum Record<'a> {
    Bytes(&'a [u8]),
    Run(u64, u8),
}

impl Record<'_> {
    fn len(&self) -> u64 {
        match self {
            Record::Bytes(b) => b.len() as u64,
            Record::Run(n, _) => *n,
        }
    }
}

/// The patch's structure checked, with the records' furthest end and the truncation.
pub fn scan(patch: &[u8]) -> Result<Scan, Refusal> {
    let mut extent = 0u64;
    let truncate = records(patch, |offset, r| extent = extent.max(offset + r.len()))?;
    Ok(Scan { extent, truncate })
}

/// The patch applied: the source copied once into an image of the size the records and the truncation make, and each
/// record written over it in order, clipped to that size.
pub fn apply(source: &[u8], patch: &[u8]) -> Result<Vec<u8>, Refusal> {
    let scan = scan(patch)?;
    let extent = scan.extent.max(source.len() as u64);
    let size = match scan.truncate {
        Some(t) if t > extent => return Err(damaged(format!("the IPS patch truncates to {t} bytes, more than the {extent} its image has"))),
        Some(t) => t,
        None => extent,
    };
    if size > MAX_IMAGE {
        return Err(Refusal::TooLarge(size));
    }
    let size = size as usize;
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&source[..source.len().min(size)]);
    out.resize(size, 0);
    records(patch, |offset, r| {
        let start = (offset as usize).min(size);
        let end = ((offset + r.len()) as usize).min(size);
        match r {
            Record::Bytes(b) => out[start..end].copy_from_slice(&b[..end - start]),
            Record::Run(_, v) => out[start..end].fill(v),
        }
    })?;
    Ok(out)
}
