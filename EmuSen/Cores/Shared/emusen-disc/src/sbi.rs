//! A `.sbi` (psx-spx "CDROM File Formats", SBI): the sectors whose subchannel Q was recorded bad, with what was read
//! there. The format cannot hold the bad CRC itself, so every listed sector is taken as one whose CRC fails.

use crate::msf::Msf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Patch {
    /// The absolute sector.
    pub at: u32,
    /// Q's bytes 0–9 as recorded, those the entry does not give left as the drive would have synthesised them.
    pub q: [Option<u8>; 10],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SbiError {
    NotSbi,
    Truncated { at: usize },
    BadEntry { at: usize },
}

impl std::fmt::Display for SbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SbiError::NotSbi => write!(f, "not an SBI file (no \"SBI\\0\" at its start)"),
            SbiError::Truncated { at } => write!(f, "SBI entry at byte {at} is cut short"),
            SbiError::BadEntry { at } => write!(f, "SBI entry at byte {at} has a bad time or format"),
        }
    }
}

/// The patches, sorted by sector.
pub fn parse(data: &[u8]) -> Result<Vec<Patch>, SbiError> {
    if data.get(..4) != Some(b"SBI\0") {
        return Err(SbiError::NotSbi);
    }
    let mut out = Vec::new();
    let mut i = 4;
    while i < data.len() {
        let head = data.get(i..i + 4).ok_or(SbiError::Truncated { at: i })?;
        let msf = Msf::from_bcd([head[0], head[1], head[2]]).ok_or(SbiError::BadEntry { at: i })?;
        let (len, from) = match head[3] {
            1 => (10, 0),
            2 => (3, 3),
            3 => (3, 7),
            _ => return Err(SbiError::BadEntry { at: i }),
        };
        let body = data.get(i + 4..i + 4 + len).ok_or(SbiError::Truncated { at: i })?;
        let mut q = [None; 10];
        for (k, &b) in body.iter().enumerate() {
            q[from + k] = Some(b);
        }
        out.push(Patch { at: msf.sector(), q });
        i += 4 + len;
    }
    out.sort_by_key(|p| p.at);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_entry_formats_read() {
        let mut f = b"SBI\0".to_vec();
        f.extend_from_slice(&[0x03, 0x08, 0x05, 3, 0x03, 0x08, 0x07]);
        f.extend_from_slice(&[0x03, 0x08, 0x10, 1, 0x41, 1, 1, 3, 6, 0x10, 0, 3, 8, 0x10]);
        f.extend_from_slice(&[0x03, 0x09, 0x56, 2, 0x03, 0x07, 0x56]);
        let p = parse(&f).unwrap();
        assert_eq!(p.iter().map(|p| p.at).collect::<Vec<_>>(), [14105, 14110, 14231]);
        assert_eq!(p[0].q[7..], [Some(3), Some(8), Some(7)]);
        assert_eq!(p[0].q[0], None);
        assert_eq!(p[1].q[0], Some(0x41));
        assert_eq!(parse(b"XYZ\0"), Err(SbiError::NotSbi));
        assert_eq!(parse(&f[..f.len() - 1]), Err(SbiError::Truncated { at: 25 }));
    }
}
