//! The software picture: libretro's three formats onto what the host accepts, copied once into a buffer sized at the
//! machine's largest picture (EmuSen_LibretroHost.md §4.2).

use emusen_libretro_sys as sys;

/// EmuSen's pixel formats, `enum emusen_pixel_format`.
pub const RGBA8888: u32 = 0;
pub const BGRA8888: u32 = 1;
pub const RGB565: u32 = 2;

/// The format handed to the host for a libretro format, given the host's mask (bit n accepts format n; RGBA8888 always).
pub fn handed_format(retro: i32, accepted: u64) -> u32 {
    let takes = |f: u32| f == RGBA8888 || accepted & (1 << f) != 0;
    match retro {
        sys::PIXEL_FORMAT_XRGB8888 if takes(BGRA8888) => BGRA8888,
        sys::PIXEL_FORMAT_RGB565 | sys::PIXEL_FORMAT_0RGB1555 if takes(RGB565) => RGB565,
        _ => RGBA8888,
    }
}

pub fn bytes_per_pixel(format: u32) -> usize {
    if format == RGB565 { 2 } else { 4 }
}

pub fn retro_bytes_per_pixel(retro: i32) -> usize {
    if retro == sys::PIXEL_FORMAT_XRGB8888 { 4 } else { 2 }
}

/// One row of `width` pixels from libretro's format into the handed one.
pub fn convert_row(retro: i32, handed: u32, src: &[u8], dst: &mut [u8], width: usize) {
    match (retro, handed) {
        (sys::PIXEL_FORMAT_XRGB8888, BGRA8888) => {
            for (d, s) in dst[..width * 4].chunks_exact_mut(4).zip(src[..width * 4].chunks_exact(4)) {
                d.copy_from_slice(&[s[0], s[1], s[2], 0xFF]);
            }
        }
        (sys::PIXEL_FORMAT_XRGB8888, _) => {
            for (d, s) in dst[..width * 4].chunks_exact_mut(4).zip(src[..width * 4].chunks_exact(4)) {
                d.copy_from_slice(&[s[2], s[1], s[0], 0xFF]);
            }
        }
        (sys::PIXEL_FORMAT_RGB565, RGB565) => dst[..width * 2].copy_from_slice(&src[..width * 2]),
        (sys::PIXEL_FORMAT_0RGB1555, RGB565) => {
            for (d, s) in dst[..width * 2].chunks_exact_mut(2).zip(src[..width * 2].chunks_exact(2)) {
                let v = u16::from_le_bytes([s[0], s[1]]);
                let (r, g, b) = ((v >> 10) & 0x1F, (v >> 5) & 0x1F, v & 0x1F);
                let g6 = (g << 1) | (g >> 4);
                d.copy_from_slice(&((r << 11) | (g6 << 5) | b).to_le_bytes());
            }
        }
        (sys::PIXEL_FORMAT_RGB565, _) => {
            for (d, s) in dst[..width * 4].chunks_exact_mut(4).zip(src[..width * 2].chunks_exact(2)) {
                let v = u16::from_le_bytes([s[0], s[1]]);
                d.copy_from_slice(&[expand5((v >> 11) as u8), expand6((v >> 5) as u8 & 0x3F), expand5(v as u8 & 0x1F), 0xFF]);
            }
        }
        _ => {
            for (d, s) in dst[..width * 4].chunks_exact_mut(4).zip(src[..width * 2].chunks_exact(2)) {
                let v = u16::from_le_bytes([s[0], s[1]]);
                d.copy_from_slice(&[expand5((v >> 10) as u8 & 0x1F), expand5((v >> 5) as u8 & 0x1F), expand5(v as u8 & 0x1F), 0xFF]);
            }
        }
    }
}

/// Five bits to eight, the top bits repeated below, so that full is 255 and none is 0.
pub fn expand5(v: u8) -> u8 {
    (v << 3) | (v >> 2)
}

pub fn expand6(v: u8) -> u8 {
    (v << 2) | (v >> 4)
}

/// The picture the host reads: its buffer, allocated once at the largest size, and the frame's shape.
pub struct Picture {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub format: u32,
    pub serial: i64,
}

impl Picture {
    pub fn new(max_width: u32, max_height: u32, format: u32) -> Picture {
        let stride = max_width as usize * bytes_per_pixel(format);
        Picture { buffer: vec![0; stride * max_height as usize], width: 0, height: 0, stride, format, serial: 0 }
    }

    /// The bytes `frame_copy` hands: `height` rows of `stride`.
    pub fn bytes(&self) -> &[u8] {
        &self.buffer[..self.stride * self.height as usize]
    }

    /// A frame from the core, or `false` when it does not fit the largest size the core declared, which is reported.
    pub fn take(&mut self, retro: i32, data: &[u8], width: u32, height: u32, pitch: usize) -> bool {
        let row_in = width as usize * retro_bytes_per_pixel(retro);
        if width as usize * bytes_per_pixel(self.format) > self.stride || self.stride * height as usize > self.buffer.len() || pitch < row_in || data.len() < pitch * (height as usize).saturating_sub(1) + row_in {
            return false;
        }
        for y in 0..height as usize {
            let src = &data[y * pitch..y * pitch + row_in];
            let dst = &mut self.buffer[y * self.stride..(y + 1) * self.stride];
            convert_row(retro, self.format, src, dst, width as usize);
        }
        self.width = width;
        self.height = height;
        self.serial += 1;
        true
    }

    /// A frame the core drew straight into this buffer, in the handed format.
    pub fn drawn_in_place(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.serial += 1;
    }
}

/// A ratio of whole numbers for a `double`: exact where the value is a ratio with a denominator under `limit`, else the
/// nearest such, by continued fractions.
pub fn ratio(value: f64, limit: u64) -> (u64, u64) {
    if !(value.is_finite() && value > 0.0) {
        return (0, 1);
    }
    let (mut h0, mut h1, mut k0, mut k1) = (0u64, 1u64, 1u64, 0u64);
    let mut x = value;
    for _ in 0..64 {
        let a = x.floor();
        if a > u32::MAX as f64 {
            break;
        }
        let a = a as u64;
        let (h2, k2) = (a.saturating_mul(h1).saturating_add(h0), a.saturating_mul(k1).saturating_add(k0));
        if k2 >= limit {
            break;
        }
        (h0, h1, k0, k1) = (h1, h2, k1, k2);
        let frac = x - a as f64;
        if (h1 as f64 / k1 as f64 - value).abs() <= value * 1e-12 || frac < 1e-12 {
            break;
        }
        x = 1.0 / frac;
    }
    let g = gcd(h1, k1.max(1));
    (h1 / g, k1.max(1) / g)
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a.max(1) } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratios_of_the_rates_cores_report() {
        assert_eq!(ratio(60.0, 1_000_000), (60, 1));
        assert_eq!(ratio(60000.0 / 1001.0, 1_000_000), (60000, 1001));
        assert_eq!(ratio(32040.5, 1_000_000), (64081, 2));
        assert_eq!(ratio(4.0 / 3.0, 100_000), (4, 3));
        assert_eq!(ratio(0.0, 100), (0, 1));
    }

    #[test]
    fn xrgb8888_hands_bgra_with_the_x_byte_opaque() {
        let mut dst = [0u8; 8];
        convert_row(sys::PIXEL_FORMAT_XRGB8888, BGRA8888, &[1, 2, 3, 0x5A, 4, 5, 6, 0], &mut dst, 2);
        assert_eq!(dst, [1, 2, 3, 255, 4, 5, 6, 255]);
        convert_row(sys::PIXEL_FORMAT_XRGB8888, RGBA8888, &[1, 2, 3, 0x5A, 4, 5, 6, 0], &mut dst, 2);
        assert_eq!(dst, [3, 2, 1, 255, 6, 5, 4, 255]);
    }

    #[test]
    fn sixteen_bit_formats_convert_to_full_range() {
        let mut dst = [0u8; 4];
        convert_row(sys::PIXEL_FORMAT_0RGB1555, RGBA8888, &0x7FFFu16.to_le_bytes(), &mut dst, 1);
        assert_eq!(dst, [255, 255, 255, 255]);
        convert_row(sys::PIXEL_FORMAT_RGB565, RGBA8888, &0xF800u16.to_le_bytes(), &mut dst, 1);
        assert_eq!(dst, [255, 0, 0, 255]);
        let mut d2 = [0u8; 2];
        convert_row(sys::PIXEL_FORMAT_0RGB1555, RGB565, &0x7FFFu16.to_le_bytes(), &mut d2, 1);
        assert_eq!(u16::from_le_bytes(d2), 0xFFFF);
    }

    #[test]
    fn the_format_handed_follows_what_the_host_accepts() {
        assert_eq!(handed_format(sys::PIXEL_FORMAT_XRGB8888, 0b011), BGRA8888);
        assert_eq!(handed_format(sys::PIXEL_FORMAT_XRGB8888, 0b001), RGBA8888);
        assert_eq!(handed_format(sys::PIXEL_FORMAT_RGB565, 0b101), RGB565);
        assert_eq!(handed_format(sys::PIXEL_FORMAT_0RGB1555, 0b001), RGBA8888);
    }

    #[test]
    fn a_frame_larger_than_declared_is_refused() {
        let mut p = Picture::new(4, 2, RGBA8888);
        assert!(!p.take(sys::PIXEL_FORMAT_XRGB8888, &[0; 64], 5, 2, 20));
        assert!(p.take(sys::PIXEL_FORMAT_XRGB8888, &[0; 32], 4, 2, 16));
        assert_eq!((p.width, p.height, p.serial), (4, 2, 1));
    }
}
