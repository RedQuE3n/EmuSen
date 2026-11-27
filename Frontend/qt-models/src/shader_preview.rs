//! Live Serenity Vulkan preview feed, synthetic native test card, and debounced chain state for Eclipse Shaders (#20).

use crate::display::{pack_preset, PLAIN};
use crate::picture::{Frame, Handoff, PictureFeed, Source};
use emusen_platform_client::{serenity, Platform};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const CARD_WIDTH: i32 = 256;
pub const CARD_HEIGHT: i32 = 224;

/// Unpacks `[width_le, height_le, rgba...]` produced by `SessionModel::pausedFrameRgba`.
pub fn unpack_rgba(bytes: &[u8]) -> Option<(Vec<u8>, i32, i32)> {
    if bytes.len() < 8 {
        return None;
    }
    let w = u32::from_le_bytes(bytes[0..4].try_into().ok()?) as i32;
    let h = u32::from_le_bytes(bytes[4..8].try_into().ok()?) as i32;
    if w <= 0 || h <= 0 {
        return None;
    }
    let need = (w as usize).checked_mul(h as usize)?.checked_mul(4)?;
    if bytes.len() < 8 + need {
        return None;
    }
    Some((bytes[8..8 + need].to_vec(), w, h))
}

/// Crisp native-resolution (256x224) pixel-art test card matching Eclipse #20's palette so CRT masks and scanlines read clearly.
pub fn synthetic_test_card() -> (Vec<u8>, i32, i32) {
    let (w, h) = (CARD_WIDTH, CARD_HEIGHT);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let mut fill = |rx: i32, ry: i32, rw: i32, rh: i32, col: [u8; 4]| {
        let x0 = rx.max(0);
        let y0 = ry.max(0);
        let x1 = (rx + rw).min(w);
        let y1 = (ry + rh).min(h);
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * w + x) * 4) as usize;
                rgba[i..i + 4].copy_from_slice(&col);
            }
        }
    };
    let sky = [0x17, 0x30, 0x6E, 0xFF];
    let moon = [0xB8, 0xFF, 0xE9, 0xFF];
    let ridge_near = [0x1F, 0xC9, 0xA8, 0xFF];
    let ridge_far = [0x2B, 0x8F, 0x86, 0xFF];
    let ground = [0x0E, 0x5A, 0x50, 0xFF];
    let ground_grid = [0x14, 0x72, 0x64, 0xFF];
    let tower = [0xF6, 0xE7, 0xD8, 0xFF];
    let sprite = [0xFF, 0xE2, 0xA8, 0xFF];
    fill(0, 0, w, h, sky);
    fill(174, 24, 38, 48, moon);
    fill(0, 108, 132, 36, ridge_near);
    fill(100, 92, 156, 52, ridge_far);
    fill(0, 144, w, h - 144, ground);
    for y in (148..h).step_by(8) {
        fill(0, y, w, 1, ground_grid);
    }
    fill(118, 120, 19, 36, tower);
    fill(124, 128, 7, 10, sky);
    fill(40, 164, 28, 12, sprite);
    for px in (42..66).step_by(4) {
        fill(px, 168, 2, 4, tower);
    }
    (rgba, w, h)
}

/// Encodes RGBA8888 pixels as a stored-DEFLATE `data:image/png;base64,...` URI for unfiltered preview display.
pub fn rgba_to_png_data_url(rgba: &[u8], width: i32, height: i32) -> String {
    if width <= 0 || height <= 0 || rgba.len() < (width as usize * height as usize * 4) {
        return String::new();
    }
    let w = width as usize;
    let h = height as usize;
    let mut raw = Vec::with_capacity(h * (1 + w * 4));
    for y in 0..h {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * w * 4..(y + 1) * w * 4]);
    }
    let mut zlib = Vec::with_capacity(raw.len() + 64);
    zlib.extend_from_slice(&[0x78, 0x01]);
    for chunk in raw.chunks(65535) {
        let last = chunk.as_ptr() as usize + chunk.len() == raw.as_ptr() as usize + raw.len();
        zlib.push(if last { 0x01 } else { 0x00 });
        let len = chunk.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(chunk);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut png = Vec::with_capacity(zlib.len() + 64);
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = [0u8; 13];
    ihdr[0..4].copy_from_slice(&(width as u32).to_be_bytes());
    ihdr[4..8].copy_from_slice(&(height as u32).to_be_bytes());
    ihdr[8] = 8;
    ihdr[9] = 6;
    push_png_chunk(&mut png, *b"IHDR", &ihdr);
    push_png_chunk(&mut png, *b"IDAT", &zlib);
    push_png_chunk(&mut png, *b"IEND", &[]);
    let mut url = String::from("data:image/png;base64,");
    encode_base64(&png, &mut url);
    url
}

fn push_png_chunk(out: &mut Vec<u8>, tag: [u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let crc_start = out.len();
    out.extend_from_slice(&tag);
    out.extend_from_slice(data);
    let crc = crc32(&out[crc_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc = (crc >> 8) ^ CRC32_TABLE[((crc ^ u32::from(b)) & 0xFF) as usize];
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn encode_base64(input: &[u8], out: &mut String) {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in input.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(*chunk.get(1).unwrap_or(&0));
        let b2 = u32::from(*chunk.get(2).unwrap_or(&0));
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((n >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(n & 0x3F) as usize] as char } else { '=' });
    }
}

/// Independent Serenity Vulkan preview feed + unfiltered source + debounced preset state for `#20`.
pub struct ShaderPreview {
    host_feed: u64,
    feed: Option<(PictureFeed, Handoff)>,
    presets_dir: Option<PathBuf>,
    frame_serial: u64,
    frame_rgba: (Vec<u8>, i32, i32),
    unfiltered_url: String,
    active_key: Option<String>,
    pending_target: Option<PreviewTarget>,
    committed_preset: Vec<u8>,
    committed_uniforms: Vec<(String, f32)>,
    problem: String,
    platform: Option<(String, std::sync::Arc<Platform>)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewTarget {
    pub stored: String,
    pub console: Option<String>,
    pub values: BTreeMap<String, f32>,
}

impl Default for ShaderPreview {
    fn default() -> Self {
        ShaderPreview {
            host_feed: 0,
            feed: None,
            presets_dir: None,
            frame_serial: 1,
            frame_rgba: synthetic_test_card(),
            unfiltered_url: String::new(),
            active_key: None,
            pending_target: None,
            committed_preset: Vec::new(),
            committed_uniforms: Vec::new(),
            problem: String::new(),
            platform: None,
        }
    }
}

impl ShaderPreview {
    fn ensure_platform(&mut self, platform_library: &str) -> Result<Option<std::sync::Arc<Platform>>, String> {
        if platform_library.is_empty() {
            return Ok(None);
        }
        if let Some((ref cached, ref platform)) = self.platform {
            if cached == platform_library {
                return Ok(Some(platform.clone()));
            }
        }
        let loaded = Platform::load(platform_library).map_err(|e| e.to_string())?;
        self.platform = Some((platform_library.to_owned(), loaded.clone()));
        Ok(Some(loaded))
    }

    /// Attaches to `host_feed`'s Vulkan device if non-zero and seeds the current source frame.
    pub fn attach(&mut self, host_feed: u64, platform_library: &str, paused_bytes: &[u8]) {
        self.set_source_frame(paused_bytes);
        if host_feed == 0 || (self.host_feed == host_feed && self.feed.is_some()) {
            self.push_frame();
            return;
        }
        self.release();
        // SAFETY: `host_feed` is `Host.feed.address()` whose `Arc<Device>` outlives the window.
        let Some((host_handoff, device)) = (unsafe { PictureFeed::at(host_feed) }) else {
            return;
        };
        let presets = host_handoff.presets().to_path_buf();
        let initial = self
            .ensure_platform(platform_library)
            .ok()
            .flatten()
            .and_then(|p| serenity::filter_preset(&p, PLAIN, None, None, &presets.to_string_lossy()).ok())
            .unwrap_or_default();
        let feed = PictureFeed::new(device, initial.clone(), presets.clone());
        let handoff = feed.handoff();
        self.host_feed = host_feed;
        self.presets_dir = Some(presets);
        self.committed_preset = initial;
        self.feed = Some((feed, handoff));
        self.push_frame();
    }

    pub fn release(&mut self) {
        self.feed = None;
        self.host_feed = 0;
        self.active_key = None;
        self.pending_target = None;
    }

    /// Updates the source frame from `paused_bytes` when a game runs, else falls back to the native test card.
    pub fn set_source_frame(&mut self, paused_bytes: &[u8]) {
        let next = unpack_rgba(paused_bytes).unwrap_or_else(synthetic_test_card);
        if next != self.frame_rgba {
            self.frame_rgba = next;
            self.unfiltered_url.clear();
            self.frame_serial = self.frame_serial.wrapping_add(1);
            self.push_frame();
        }
    }

    fn push_frame(&mut self) {
        let Some((_, handoff)) = &self.feed else { return };
        let mut frame = Frame {
            rgba: self.frame_rgba.0.clone(),
            width: self.frame_rgba.1,
            height: self.frame_rgba.2,
            row_repeat: 1,
            serial: self.frame_serial,
            source: Source::Pixels,
        };
        handoff.hand_over(&mut frame);
    }

    pub fn feed_address(&self) -> u64 {
        self.feed.as_ref().map_or(0, |(f, _)| f.address())
    }

    pub fn unfiltered_url(&mut self) -> &str {
        if self.unfiltered_url.is_empty() {
            self.unfiltered_url = rgba_to_png_data_url(&self.frame_rgba.0, self.frame_rgba.1, self.frame_rgba.2);
        }
        &self.unfiltered_url
    }

    pub fn problem(&self) -> &str {
        &self.problem
    }

    pub fn active_key(&self) -> Option<&str> {
        self.active_key.as_deref()
    }

    pub fn has_pending(&self) -> bool {
        self.pending_target.is_some()
    }

    /// Schedules a debounced chain rebuild for `target` without re-reading presets until `commit` fires.
    pub fn schedule(&mut self, target: PreviewTarget) -> bool {
        if self.active_key.as_deref() == Some(target.stored.as_str()) && self.pending_target.is_none() {
            return false;
        }
        self.pending_target = Some(target);
        true
    }

    /// Commits any pending preset change through Serenity's preset reader onto the preview `PictureFeed`.
    pub fn commit(&mut self, platform_library: &str) -> bool {
        match self.ensure_platform(platform_library) {
            Ok(platform) => self.commit_with_platform(platform.as_deref()),
            Err(words) => {
                self.pending_target = None;
                self.problem = words;
                false
            }
        }
    }

    /// Commits any pending preset change using an already-loaded `Platform` handle.
    pub fn commit_with_platform(&mut self, platform: Option<&Platform>) -> bool {
        let Some(target) = self.pending_target.take() else { return false };
        let resolved = resolve_preset_on(platform, self.presets_dir.as_deref(), &target);
        match resolved {
            Ok((preset, uniforms)) => {
                self.problem.clear();
                self.active_key = Some(target.stored);
                self.committed_preset = preset.clone();
                self.committed_uniforms = uniforms.clone();
                if let Some((_, handoff)) = &self.feed {
                    handoff.draw_through(preset, uniforms);
                    self.push_frame();
                }
                true
            }
            Err(words) => {
                self.problem = words;
                self.active_key = Some(target.stored);
                false
            }
        }
    }

    /// Pushes a single slider value directly onto the live chain via `PictureFeed::set_value` without rebuilding passes.
    pub fn set_live_value(&mut self, id: &str, value: f32) -> bool {
        self.committed_uniforms.retain(|(k, _)| k != id);
        self.committed_uniforms.push((id.to_owned(), value));
        if let Some((_, handoff)) = &self.feed {
            if handoff.set_value(id, value) {
                return true;
            }
            if !self.committed_preset.is_empty() {
                handoff.draw_through(self.committed_preset.clone(), self.committed_uniforms.clone());
                return true;
            }
        }
        false
    }
}

fn resolve_preset_on(platform: Option<&Platform>, presets_dir: Option<&std::path::Path>, target: &PreviewTarget) -> Result<(Vec<u8>, Vec<(String, f32)>), String> {
    let Some(platform) = platform else {
        return Ok((Vec::new(), Vec::new()));
    };
    let uniforms: Vec<(String, f32)> = target.values.iter().map(|(k, v)| (k.clone(), *v)).collect();
    if let Some(pack_rel) = pack_preset(&target.stored) {
        if !crate::settings_shaders::is_safe_pack_rel(pack_rel) {
            return Err("Invalid shader preset path.".to_owned());
        }
        let dir = platform.directory(emusen_platform_client::galaxia::consts::DIR_SHADERS).map_err(|e| e.to_string())?.ok_or("No shader pack folder.")?;
        let preset = serenity::preset_document(platform, &dir.join(pack_rel).to_string_lossy()).map_err(|e| e.to_string())?;
        return Ok((preset, uniforms));
    }
    let root = presets_dir
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("emusen-shader-preview-{}", std::process::id())));
    let chosen = serde_json::to_value(&target.values).ok();
    let key = if target.stored == "None" || target.stored.is_empty() { PLAIN } else { target.stored.as_str() };
    let preset = serenity::filter_preset(platform, key, target.console.as_deref(), chosen.as_ref(), &root.to_string_lossy()).map_err(|e| e.to_string())?;
    Ok((preset, uniforms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_card_dimensions_and_palette() {
        let (rgba, w, h) = synthetic_test_card();
        assert_eq!((w, h), (256, 224));
        assert_eq!(rgba.len(), 256 * 224 * 4);
        assert_eq!(&rgba[0..4], &[0x17, 0x30, 0x6E, 0xFF]);
        let moon_idx = ((30 * w + 180) * 4) as usize;
        assert_eq!(&rgba[moon_idx..moon_idx + 4], &[0xB8, 0xFF, 0xE9, 0xFF]);
    }

    #[test]
    fn png_data_url_has_valid_header_and_unpack_roundtrips() {
        let (rgba, w, h) = synthetic_test_card();
        let url = rgba_to_png_data_url(&rgba, w, h);
        assert!(url.starts_with("data:image/png;base64,iVBORw0KGgo"));
        let mut packed = Vec::new();
        packed.extend_from_slice(&(w as u32).to_le_bytes());
        packed.extend_from_slice(&(h as u32).to_le_bytes());
        packed.extend_from_slice(&rgba);
        let (out, ow, oh) = unpack_rgba(&packed).unwrap();
        assert_eq!((ow, oh), (w, h));
        assert_eq!(out, rgba);
        assert!(unpack_rgba(&[]).is_none());
    }

    #[test]
    fn debounce_schedule_and_software_fallback() {
        let mut preview = ShaderPreview::default();
        assert_eq!(preview.feed_address(), 0);
        assert!(preview.unfiltered_url.is_empty(), "default constructor skips eager PNG encoding");
        assert!(preview.unfiltered_url().starts_with("data:image/png;base64,"));
        assert!(!preview.unfiltered_url.is_empty());
        assert!(preview.schedule(PreviewTarget { stored: "CRT".into(), console: Some("SNES".into()), values: BTreeMap::new() }));
        assert!(preview.has_pending());
        assert!(preview.commit(""));
        assert!(!preview.has_pending());
        assert_eq!(preview.active_key(), Some("CRT"));
        assert!(!preview.schedule(PreviewTarget { stored: "CRT".into(), console: Some("SNES".into()), values: BTreeMap::new() }));
    }
}
