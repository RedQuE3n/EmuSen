//! A zip archive of files deflated at the best level, each stamped with its local time as DOS stores it: what
//! `ZipArchive.CreateEntryFromFile` with `CompressionLevel.Optimal` writes, for any reader of the format.

use flate2::Compression;
use flate2::write::DeflateEncoder;
use std::io::Write;

type LocalTime = (i64, u32, u32, u32, u32, u32, u32);

fn dos(t: LocalTime) -> (u16, u16) {
    let (y, mo, d, h, mi, s, _) = t;
    let time = (h << 11 | mi << 5 | s / 2) as u16;
    let date = (((y - 1980).clamp(0, 127) as u32) << 9 | mo << 5 | d) as u16;
    (time, date)
}

/// The archive of `(name, bytes, local time)` entries, in order.
pub fn write(entries: &[(String, Vec<u8>, LocalTime)]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, bytes, time) in entries {
        let mut crc = flate2::Crc::new();
        crc.update(bytes);
        let mut e = DeflateEncoder::new(Vec::new(), Compression::best());
        e.write_all(bytes)?;
        let packed = e.finish()?;
        let (t, d) = dos(*time);
        let offset = out.len() as u32;
        let header = |sig: u32, central: bool| {
            let mut h = Vec::new();
            h.extend_from_slice(&sig.to_le_bytes());
            if central {
                h.extend_from_slice(&20u16.to_le_bytes());
            }
            h.extend_from_slice(&20u16.to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes());
            h.extend_from_slice(&8u16.to_le_bytes());
            h.extend_from_slice(&t.to_le_bytes());
            h.extend_from_slice(&d.to_le_bytes());
            h.extend_from_slice(&crc.sum().to_le_bytes());
            h.extend_from_slice(&(packed.len() as u32).to_le_bytes());
            h.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            h.extend_from_slice(&(name.len() as u16).to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes());
            if central {
                // The comment's length, the disk, the internal attributes.
                h.extend_from_slice(&[0u8; 6]);
                h.extend_from_slice(&0u32.to_le_bytes());
                h.extend_from_slice(&offset.to_le_bytes());
            }
            h.extend_from_slice(name.as_bytes());
            h
        };
        out.extend_from_slice(&header(0x0403_4b50, false));
        out.extend_from_slice(&packed);
        central.extend_from_slice(&header(0x0201_4b50, true));
    }
    let at = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&at.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    Ok(out)
}
