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

/// The PNG of `rgba`, `width` by `height`.
pub fn encode(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row = width as usize * 4;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for y in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * row..(y + 1) * row]);
    }
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let _ = z.write_all(&raw);
    let deflated = z.finish().unwrap_or_default();
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &deflated);
    chunk(&mut out, b"IEND", &[]);
    out
}

/// `PngFile.Write`: the picture written to `path`.
pub fn write(path: &str, rgba: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    std::fs::write(path, encode(rgba, width, height))
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
}
