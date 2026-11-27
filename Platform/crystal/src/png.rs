//! A state's picture as a PNG: RGBA at eight bits a channel, each row unfiltered, deflated by `flate2`. Its bytes
//! differ from SkiaSharp's encoder and its pixels do not (EmuSen_Backend.md §29.5).

use std::io::Write;

use emusen_galaxia::rom_hash::Crc32;

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut crc = Crc32::default();
    crc.append(kind);
    crc.append(data);
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc.value().to_be_bytes());
}

/// The PNG of `rgba`, `width` by `height`, returning an error when dimensions or buffer size are invalid.
pub fn try_encode(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 {
        return Err("empty picture".into());
    }
    let row = (width as usize).checked_mul(4).ok_or_else(|| "picture width overflow".to_string())?;
    let needed = row.checked_mul(height as usize).ok_or_else(|| "picture size overflow".to_string())?;
    if rgba.len() < needed {
        return Err(format!("picture buffer too short: {} < {needed}", rgba.len()));
    }
    let cap = row.checked_add(1).and_then(|r| r.checked_mul(height as usize)).ok_or_else(|| "picture raw overflow".to_string())?;
    let mut raw = Vec::with_capacity(cap);
    for y in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * row..(y + 1) * row]);
    }
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    z.write_all(&raw).map_err(|e| e.to_string())?;
    let deflated = z.finish().map_err(|e| e.to_string())?;
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &deflated);
    chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

/// The PNG of `rgba`, `width` by `height`.
pub fn encode(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    try_encode(rgba, width, height).unwrap_or_default()
}

/// `PngFile.Write`: the picture atomically written to `path`.
pub fn write(path: &str, rgba: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    let encoded = try_encode(rgba, width, height).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    emusen_galaxia::atomic::write(path, &encoded).map_err(std::io::Error::other)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_picture_is_a_png_whose_rows_inflate_back() {
        let rgba: Vec<u8> = (0..2 * 3 * 4).map(|i| i as u8).collect();
        let png = super::encode(&rgba, 2, 3);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        let idat = 8 + 25;
        let len = u32::from_be_bytes(png[idat..idat + 4].try_into().unwrap()) as usize;
        let mut d = flate2::read::ZlibDecoder::new(&png[idat + 8..idat + 8 + len]);
        let mut raw = Vec::new();
        std::io::Read::read_to_end(&mut d, &mut raw).unwrap();
        assert_eq!(raw.len(), 3 * 9);
        assert_eq!(&raw[1..9], &rgba[..8]);
    }

    #[test]
    fn short_or_overflowing_rgba_is_rejected_without_panic() {
        assert!(super::try_encode(&[0u8; 4], 2, 2).is_err());
        assert!(super::try_encode(&[], 0, 2).is_err());
        assert!(super::try_encode(&[0u8; 16], u32::MAX, 2).is_err());
        assert!(super::write("/tmp/should-not-write.png", &[0u8; 4], 2, 2).is_err());
    }
}
