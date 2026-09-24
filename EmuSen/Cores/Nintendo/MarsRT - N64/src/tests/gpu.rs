//! The device's picture at a multiple against the processor's at the same multiple, byte for byte: C#'s `MarsGpuRasteriserTests` on
//! MarsRT, over the direct path as those tests take it and through the interface as a game reaches it. Every test stands down
//! with no Vulkan device, which is a CI runner's case. See Mars_Gpu.md §5, §13, §14 and §15, and Mars_Native.md §6.4.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::machine::Machine;
use crate::memory::bus::RDRAM_SIZE;
use crate::memory::bus_access::map::DP_COMMAND_BASE;
use crate::rdp::gpu::{GpuDevice, GpuRasteriser};
use crate::rdp::{Rdp, RdpMemory};
use crate::vi::scan::Scanout;

const FRAMEBUFFER: u32 = 0x0020_0000;
const SECOND: u32 = 0x0030_0000;
const DEPTH_BUFFER: u32 = 0x0028_0000;
const TEXTURE_SOURCE: u32 = 0x0004_0000;
const WIDTH: u32 = 320;
const ROWS: u32 = 240;
const FILL_CYCLE: u64 = (0x2F << 56) | (3 << 52);
const VI_BASE: u32 = 0x0440_0000;
const CURRENT_LINE: u32 = 4;
const SERRATED: u32 = 1 << 6;
const GAMMA_ON: u32 = 1 << 3;
const DIVOT_ON: u32 = 1 << 4;
const DITHER_FILTER: u32 = 1 << 16;

/// The devices a test runs on: the first the loader ranks, which by standing instruction is the discrete card, or every one with
/// `EMUSEN_MARS_GPU_TEST_DEVICES=all`; none without Vulkan, when the test says so and stands down.
fn devices() -> Vec<String> {
    let names = GpuDevice::device_names();
    if names.is_empty() {
        eprintln!("no Vulkan device: the CPU path's machine");
        return names;
    }
    if std::env::var("EMUSEN_MARS_GPU_TEST_DEVICES").is_ok_and(|v| v == "all") { names } else { names[..1].to_vec() }
}

fn rasteriser(name: &str, scale: usize) -> Option<GpuRasteriser> {
    let device = GpuDevice::try_create(Some(name)).unwrap_or_else(|report| panic!("{report}"));
    match GpuRasteriser::try_create(device, RDRAM_SIZE * scale * scale) {
        Ok(gpu) => Some(gpu),
        Err((_, report)) => {
            eprintln!("not run: {report}");
            None
        }
    }
}

fn next(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state >> 8
}

fn pick(state: &mut u32, from: &[i32]) -> i32 {
    from[(next(state) % from.len() as u32) as usize]
}

fn color_image(address: u32, size: u64, width: u32) -> u64 {
    (0x3F << 56) | (size << 51) | (((width - 1) as u64) << 32) | address as u64
}

fn fill_color(color: u32) -> u64 {
    (0x37 << 56) | color as u64
}

/// Quarter pixels, so that edges fall inside pixels.
fn scissor(left: u32, top: u32, right: u32, bottom: u32) -> u64 {
    (0x2D << 56) | ((left as u64) << 44) | ((top as u64) << 32) | ((right as u64) << 12) | bottom as u64
}

fn fill_rectangle(left: u32, top: u32, right: u32, bottom: u32) -> u64 {
    (0x36 << 56) | ((right as u64) << 44) | ((bottom as u64) << 32) | ((left as u64) << 12) | top as u64
}

fn edge(x: f64, slope: f64) -> u64 {
    (((x * 65536.0).round_ties_even() as i32 as u32 as u64) << 32) | ((slope.clamp(-8192.0, 8191.0) * 65536.0).round_ties_even() as i32 as u32 as u64)
}

/// `Triangle`: a shaded, depth-tested triangle from three corners; its shade and depth words are a seed's.
fn triangle(corners: [(f64, f64); 3], s: &mut u32) -> Vec<u64> {
    const ID: u32 = 0x0D;
    let mut sorted = corners;
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let (top, middle, bottom) = (sorted[0], sorted[1], sorted[2]);
    let major = if bottom.1 > top.1 { (bottom.0 - top.0) / (bottom.1 - top.1) } else { 0.0 };
    let upper = if middle.1 > top.1 { (middle.0 - top.0) / (middle.1 - top.1) } else { 0.0 };
    let lower = if bottom.1 > middle.1 { (bottom.0 - middle.0) / (bottom.1 - middle.1) } else { 0.0 };
    let (yh, ym, yl) = ((top.1 * 4.0).floor() as i32, (middle.1 * 4.0).floor() as i32, (bottom.1 * 4.0).floor() as i32);
    let row_top = top.1.floor();
    let (xh, xm, xl) = (top.0 + major * (row_top - top.1), top.0 + upper * (row_top - top.1), middle.0 + lower * (ym as f64 / 4.0 - middle.1));
    let major_on_left = middle.0 > top.0 + major * (middle.1 - top.1);

    let mut words = vec![0u64; crate::rdp::command_length(ID) as usize];
    words[0] = ((ID as u64) << 56) | if major_on_left { 1 << 55 } else { 0 } | (((yl & 0x3FFF) as u64) << 32) | (((ym & 0x3FFF) as u64) << 16) | (yh & 0x3FFF) as u64;
    words[1] = edge(xl, lower);
    words[2] = edge(xh, major);
    words[3] = edge(xm, upper);
    for word in &mut words[4..12] {
        *word = (((next(s) & 0x007F_FFFF) as u64) << 32) | (next(s) & 0x0003_FFFF) as u64;
    }
    words[12] = (((0x1000 + (next(s) & 0xFFFF)) as u64) << 48) | (((next(s) & 0xFFFF) as u64) << 32) | (((next(s) & 0x3FF) as u64) << 16) | (next(s) & 0xFFFF) as u64;
    words[13] = (((next(s) & 0x3FF) as u64) << 48) | (((next(s) & 0xFFFF) as u64) << 32) | (next(s) & 0x03FF_FFFF) as u64;
    words
}

#[allow(clippy::too_many_arguments)]
fn combine_words(a: i32, b: i32, c: i32, d: i32, aa: i32, ab: i32, ac: i32, ad: i32, second: [i32; 8]) -> u64 {
    let [a2, b2, c2, d2, aa2, ab2, ac2, ad2] = second;
    let high = ((a << 20) | (c << 15) | (aa << 12) | (ac << 9) | (a2 << 5) | c2) as u32 as u64;
    let low = ((b << 28) | (b2 << 24) | (aa2 << 21) | (ac2 << 18) | (d << 15) | (ab << 12) | (ad << 9) | (d2 << 6) | (ab2 << 3) | ad2) as u32 as u64;
    (0x3C << 56) | (high << 32) | low
}

/// `Combine`: both cycles alike, from the inputs the device has: no previous pixel's result, no texel, no level of detail (Mars_Gpu.md §6).
fn combine(s: &mut u32) -> u64 {
    let (a, b) = (pick(s, &[3, 4, 5, 6, 7, 15]), pick(s, &[3, 4, 5, 6, 7, 15]));
    let (c, d) = (pick(s, &[3, 4, 5, 6, 10, 11, 12, 14, 15, 31]), pick(s, &[3, 4, 5, 6, 7]));
    let (aa, ab, ac, ad) = (pick(s, &[3, 4, 5, 6, 7]), pick(s, &[3, 4, 5, 6, 7]), pick(s, &[3, 4, 5, 6, 7]), pick(s, &[3, 4, 5, 6, 7]));
    combine_words(a, b, c, d, aa, ab, ac, ad, [a, b, c, d, aa, ab, ac, ad])
}

fn keyed_combine(c: i32) -> u64 {
    let (a, b, d, alpha) = (4, 15, 7, 6);
    combine_words(a, b, c, d, alpha, alpha, alpha, alpha, [a, b, c, d, alpha, alpha, alpha, alpha])
}

/// `TwoCycleCombine`: the first cycle never reads the combiner's previous result, the pixel before; the second may and should.
fn two_cycle_combine(s: &mut u32) -> u64 {
    let (a1, b1) = (pick(s, &[1, 2, 3, 4, 5, 6, 7]), pick(s, &[1, 2, 3, 4, 5, 7]));
    let (c1, d1) = (pick(s, &[1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 31]), pick(s, &[1, 2, 3, 4, 5, 6, 7]));
    let (aa1, ab1) = (pick(s, &[1, 2, 3, 4, 5, 6, 7]), pick(s, &[1, 2, 3, 4, 5, 7]));
    let (ac1, ad1) = (pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]), pick(s, &[1, 2, 3, 4, 5, 6, 7]));
    let (a2, b2) = (pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]), pick(s, &[0, 1, 2, 3, 4, 5, 7]));
    let (c2, d2) = (pick(s, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 31]), pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]));
    let (aa2, ab2) = (pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]), pick(s, &[0, 1, 2, 3, 4, 5, 7]));
    let (ac2, ad2) = (pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]), pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]));
    combine_words(a1, b1, c1, d1, aa1, ab1, ac1, ad1, [a2, b2, c2, d2, aa2, ab2, ac2, ad2])
}

/// `TexturedCombine`: selectors that read the texels and the level of detail.
fn textured_combine(s: &mut u32) -> u64 {
    let (a, b) = (pick(s, &[1, 2, 3, 4, 5, 6, 7]), pick(s, &[1, 2, 3, 4, 5, 7]));
    let (c, d) = (pick(s, &[1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 31]), pick(s, &[1, 2, 3, 4, 5, 6, 7]));
    let (aa, ab) = (pick(s, &[1, 2, 3, 4, 5, 6, 7]), pick(s, &[1, 2, 3, 4, 5, 7]));
    let (ac, ad) = (pick(s, &[0, 1, 2, 3, 4, 5, 6, 7]), pick(s, &[1, 2, 3, 4, 5, 6, 7]));
    combine_words(a, b, c, d, aa, ab, ac, ad, [a, b, c, d, aa, ab, ac, ad])
}

/// `OneCycleModes`: everything below the cycle type a seed's; the first blend's second alpha never memory alpha, a carry the device declines.
fn one_cycle_modes(s: &mut u32, two_cycle: bool) -> u64 {
    let mut modes = (0x2F << 56) | (((next(s) & 3) as u64) << 38) | (((next(s) & 3) as u64) << 36);
    if next(s).is_multiple_of(3) {
        modes |= 1 << 40;
    }
    let first_second_alpha = if two_cycle { pick(s, &[0, 2, 3]) as u64 } else { (next(s) & 3) as u64 };
    modes |= (((next(s) & 3) as u64) << 30) | (((next(s) & 3) as u64) << 26) | (((next(s) & 3) as u64) << 22) | (first_second_alpha << 18);
    if two_cycle {
        modes |= (1 << 52) | (((next(s) & 3) as u64) << 28) | (((next(s) & 3) as u64) << 24) | (((next(s) & 3) as u64) << 20) | (((next(s) & 3) as u64) << 16);
    }
    modes | (next(s) & 0x7FFF) as u64
}

fn colors(list: &mut Vec<u64>, s: &mut u32) {
    for id in [0x38u64, 0x39, 0x3A, 0x3B, 0x2C, 0x2E] {
        list.push((id << 56) | (((next(s) & 0xFF_FFFF) as u64) << 32) | ((next(s) as u64) << 8) | (next(s) & 0xFF) as u64);
    }
}

fn cleared(size: u64) -> Vec<u64> {
    vec![
        FILL_CYCLE,
        (0x3E << 56) | DEPTH_BUFFER as u64,
        color_image(DEPTH_BUFFER, 2, WIDTH),
        scissor(0, 0, WIDTH * 4, ROWS * 4),
        fill_color(0xFFFC_FFFC),
        fill_rectangle(0, 0, WIDTH * 4 - 4, ROWS * 4 - 4),
        color_image(FRAMEBUFFER, size, WIDTH),
        fill_color(0x2109_8421),
        fill_rectangle(0, 0, WIDTH * 4 - 4, ROWS * 4 - 4),
    ]
}

/// `Shaded`: depth and picture cleared, then seventy-two triangles over and past the image, each under modes, colours and a combiner of its own.
fn shaded(seed: u32, size: u64, keyed: bool, two_cycle: bool) -> Vec<u64> {
    let mut s = seed;
    let mut list = cleared(size);
    for _ in 0..72 {
        if keyed {
            list.push((0x2F << 56) | (((next(&mut s) & 3) as u64) << 38) | (1 << 40) | (1 << 22) | (1 << 14));
            list.push(keyed_combine(pick(&mut s, &[10, 12, 15])));
        } else {
            list.push(one_cycle_modes(&mut s, two_cycle));
            list.push(if two_cycle { two_cycle_combine(&mut s) } else { combine(&mut s) });
        }
        colors(&mut list, &mut s);
        let width_mask: u32 = if next(&mut s).is_multiple_of(2) { 0x3 } else { 0xFFF };
        list.push((0x2A << 56) | (((next(&mut s) & width_mask) as u64) << 44) | (((next(&mut s) & width_mask) as u64) << 32) | ((next(&mut s) as u64) << 8) | (next(&mut s) & 0xFF) as u64);
        list.push((0x2B << 56) | (((next(&mut s) & width_mask) as u64) << 16) | (next(&mut s) & 0xFFFF) as u64);
        let mut corner = |extent: u32| (next(&mut s) % (extent + 60)) as i32 as f64 - 30.0 + (next(&mut s) & 3) as f64 / 4.0;
        let c = [(corner(WIDTH), corner(ROWS)), (corner(WIDTH), corner(ROWS)), (corner(WIDTH), corner(ROWS))];
        list.extend(triangle(c, &mut s));
    }
    list
}

/// `Small`: a frame's worth of small triangles, which is what a game's list looks like.
fn small(seed: u32, triangles: usize) -> Vec<u64> {
    let mut s = seed;
    let mut list = shaded(seed, 2, false, false)[..9].to_vec();
    for i in 0..triangles {
        if i.is_multiple_of(16) {
            list.push(one_cycle_modes(&mut s, false));
            list.push(combine(&mut s));
        }
        let (cx, cy) = ((next(&mut s) % WIDTH) as f64, (next(&mut s) % ROWS) as f64);
        let mut near = |c: f64| c + (next(&mut s) % 40) as i32 as f64 - 20.0 + (next(&mut s) & 3) as f64 / 4.0;
        let c = [(near(cx), near(cy)), (near(cx), near(cy)), (near(cx), near(cy))];
        list.extend(triangle(c, &mut s));
    }
    list
}

fn pack(v: [i32; 4], shift: u32) -> u64 {
    ((((v[0] >> shift) & 0xFFFF) as u32 as u64) << 48) | ((((v[1] >> shift) & 0xFFFF) as u32 as u64) << 32) | ((((v[2] >> shift) & 0xFFFF) as u32 as u64) << 16) | ((v[3] >> shift) & 0xFFFF) as u32 as u64
}

/// `TexturedTriangle`: a triangle carrying shade, texture and depth, its coordinates swept through the divider's shifts by `mode` and `index`.
fn textured_triangle(corners: [(f64, f64); 3], mode: i32, tile: u64, index: usize, max_level: u64, s: &mut u32) -> Vec<u64> {
    let shaded = triangle(corners, s);
    let mut words = vec![0u64; crate::rdp::command_length(0x0F) as usize];
    words[..12].copy_from_slice(&shaded[..12]);
    words[0] = (words[0] & !(0xFF << 56)) | (0x0F << 56) | (tile << 48) | (max_level << 51);
    words[20] = shaded[12];
    words[21] = shaded[13];

    let (mut value, mut dx, mut de, mut dy) = ([0i32; 4], [0i32; 4], [0i32; 4], [0i32; 4]);
    let kind: i32 = if mode == 0 { -1 } else if mode == 1 { 0 } else { 1 + (index % 3) as i32 };
    if kind <= 1 {
        let w: i32 = if kind < 0 { 1 } else { 1 << (index % 14) };
        let start = if w <= 1 { 0 } else { (next(s) % (if kind == 0 { w } else { w * 8 }) as u32) as i32 };
        let step = ((w << 16) >> 8).max(1);
        for c in 0..2 {
            value[c] = if kind < 0 { (((next(s) % 32) << 21) as i32) | (next(s) & 0x1F_FFFF) as i32 } else { (start << 16) | (next(s) & 0xFFFF) as i32 };
            dx[c] = if kind < 0 { (next(s) & 0x3_FFFF) as i32 - 0x2_0000 } else { step - (next(s) % (2 * step) as u32) as i32 };
            de[c] = if kind < 0 { (next(s) & 0x3_FFFF) as i32 - 0x2_0000 } else { step - (next(s) % (2 * step) as u32) as i32 };
            dy[c] = if kind < 0 { (next(s) & 0x3_FFFF) as i32 - 0x2_0000 } else { step - (next(s) % (2 * step) as u32) as i32 };
        }
        value[2] = if kind < 0 { 0 } else { (w << 16) | (next(s) & 0xFFFF) as i32 };
        let w_step = ((w << 16) >> 10).max(1);
        let w_delta = if kind < 0 { 0 } else { w_step - (next(s) % (2 * w_step) as u32) as i32 };
        (dx[2], de[2], dy[2]) = (w_delta, w_delta, w_delta);
    } else {
        let low = (next(s) & 0xFFFF) as i32;
        for c in 0..2 {
            value[c] = (((next(s) % 32) << 21) as i32) | (next(s) & 0x1F_FFFF) as i32;
            dx[c] = (next(s) & 0x3_FFFF) as i32 - 0x2_0000;
            de[c] = (next(s) & 0x3_FFFF) as i32 - 0x2_0000;
            dy[c] = (next(s) & 0x3_FFFF) as i32 - 0x2_0000;
        }
        value[2] = if kind == 2 { (((next(s) & 3) << 16) as i32) | low } else { 0xFFFF_0000u32 as i32 | low };
        let delta = (next(s) & 0x1FFF) as i32 - 0x1000;
        (dx[2], de[2], dy[2]) = (delta, delta, delta);
    }
    (words[12], words[14]) = (pack(value, 16), pack(value, 0));
    (words[13], words[15]) = (pack(dx, 16), pack(dx, 0));
    (words[16], words[18]) = (pack(de, 16), pack(de, 0));
    (words[17], words[19]) = (pack(dy, 16), pack(dy, 0));
    words
}

/// Every format and size a tile can name.
fn formats() -> Vec<(u64, u64)> {
    (0..5).flat_map(|f| (0..4).map(move |z| (f, z))).collect()
}

/// `Textured`: a tile loaded from a sixteen-bit image and read under a format and size of its own, and triangles over it.
fn textured(seed: u32, mode: i32, two_cycle: bool) -> Vec<u64> {
    let mut s = seed;
    let mut list = cleared(2);
    list.push((0x3D << 56) | (2 << 51) | (63 << 32) | TEXTURE_SOURCE as u64);
    let formats = formats();
    for i in 0..formats.len() * 3 {
        let (format, size) = formats[i % formats.len()];
        let tile = (next(&mut s) % 7) as u64;
        let line = 4 + (next(&mut s) & 7) as u64;
        let memory = (next(&mut s) % 3) as u64 * 64;
        let (shift_s, shift_t) = ((next(&mut s) % 16) as u64, (next(&mut s) % 16) as u64);
        let (mask_s, mask_t) = ((next(&mut s) % 11) as u64, (next(&mut s) % 11) as u64);
        let wraps = next(&mut s) as u64;
        list.push(
            (0x35 << 56)
                | (format << 53)
                | (size << 51)
                | (line << 41)
                | (memory << 32)
                | (tile << 24)
                | (((next(&mut s) & 0xF) as u64) << 20)
                | ((wraps & 1) << 19)
                | ((wraps & 2) << 17)
                | (mask_t << 14)
                | (shift_t << 10)
                | ((wraps & 4) << 7)
                | ((wraps & 8) << 5)
                | (mask_s << 4)
                | shift_s,
        );
        list.push((0x34 << 56) | (tile << 24) | ((31 << 2) << 12) | (31 << 2));
        list.push((0x35 << 56) | (2 << 51) | (0x100 << 32) | (7 << 24));
        list.push((0x30 << 56) | (7 << 24) | (((255u64) << 2) << 12));

        let low = if mode == 1 { next(&mut s) & 0x7FEE } else { next(&mut s) & 0x7FFF } as u64;
        let cycles = if two_cycle {
            (1 << 52) | (((next(&mut s) & 3) as u64) << 28) | (((next(&mut s) & 3) as u64) << 24) | (((next(&mut s) & 3) as u64) << 20) | (((next(&mut s) & 3) as u64) << 16)
        } else {
            0
        };
        let mut modes = cycles | (0x2F << 56) | (((next(&mut s) & 3) as u64) << 38) | (((next(&mut s) & 3) as u64) << 36) | if mode == 0 { 0 } else { 1 << 51 };
        modes |= ((next(&mut s) & 1) as u64) << 43;
        for bit in [45u32, 47, 46, 44, 48, 49, 50] {
            modes |= ((next(&mut s) & 1) as u64) << bit;
        }
        for bit in [30u32, 26, 22] {
            modes |= ((next(&mut s) & 3) as u64) << bit;
        }
        modes |= (if two_cycle { pick(&mut s, &[0, 2, 3]) as u64 } else { (next(&mut s) & 3) as u64 }) << 18;
        list.push(modes | low);
        list.push(if two_cycle { two_cycle_combine(&mut s) } else { textured_combine(&mut s) });
        colors(&mut list, &mut s);

        // One primitive of each list carries no texture coordinates at all: a w of zero is a state the divider reaches and random values do not.
        if i == formats.len() && mode != 0 {
            let mut degenerate = textured_triangle([(10.0, 10.0), (300.0, 40.0), (40.0, 200.0)], 2, tile, i, 0, &mut s);
            for word in &mut degenerate[12..20] {
                *word = 0;
            }
            list.extend(degenerate);
        }
        let (cx, cy) = ((next(&mut s) % WIDTH) as f64, (next(&mut s) % ROWS) as f64);
        let mut near = |c: f64| c + (next(&mut s) % 90) as i32 as f64 - 45.0 + (next(&mut s) & 3) as f64 / 4.0;
        let c = [(near(cx), near(cy)), (near(cx), near(cy)), (near(cx), near(cy))];
        list.extend(textured_triangle(c, mode, tile, i, (next(&mut s) & 7) as u64, &mut s));
    }
    list
}

/// `Copied`: copy-mode rectangles over every tile format, some flipped, some reaching texel rows far past the tile.
fn copied(seed: u32) -> Vec<u64> {
    let mut s = seed;
    let mut list = vec![
        FILL_CYCLE,
        color_image(FRAMEBUFFER, 2, WIDTH),
        scissor(0, 0, WIDTH * 4, ROWS * 4),
        fill_color(0x2109_8421),
        fill_rectangle(0, 0, WIDTH * 4 - 4, ROWS * 4 - 4),
        (0x3D << 56) | (2 << 51) | (63 << 32) | TEXTURE_SOURCE as u64,
        (0x35 << 56) | (2 << 51) | (0x100 << 32) | (7 << 24),
        (0x30 << 56) | (7 << 24) | (((255u64) << 2) << 12),
    ];
    let formats = formats();
    for i in 0..formats.len() * 3 {
        let (format, size) = formats[i % formats.len()];
        let tile = (i % 7) as u64;
        let line = if i % 4 == 3 { 0x40 + (next(&mut s) % 0x1C0) as u64 } else { 8 + (i % 3) as u64 };
        let wraps = next(&mut s) as u64;
        list.push(
            (0x35 << 56)
                | (format << 53)
                | (size << 51)
                | (line << 41)
                | (((i % 3) as u64 * 64) << 32)
                | (tile << 24)
                | (((next(&mut s) & 0xF) as u64) << 20)
                | ((wraps & 1) << 19)
                | ((wraps & 2) << 17)
                | (((next(&mut s) % 11) as u64) << 14)
                | (((next(&mut s) % 16) as u64) << 10)
                | ((wraps & 4) << 7)
                | ((wraps & 8) << 5)
                | (((next(&mut s) % 11) as u64) << 4)
                | (next(&mut s) % 16) as u64,
        );
        list.push((0x34 << 56) | (tile << 24) | ((31 << 2) << 12) | (31 << 2));
        let mut modes = (0x2F << 56) | (2 << 52) | (next(&mut s) & 1) as u64;
        for bit in [47u32, 46, 51, 48, 50, 49] {
            modes |= ((next(&mut s) & 1) as u64) << bit;
        }
        list.push(modes);
        list.push((0x3A << 56) | (((next(&mut s) & 0xFF_FFFF) as u64) << 32) | ((next(&mut s) as u64) << 8) | (next(&mut s) & 0xFF) as u64);
        let (left, top) = ((next(&mut s) % (WIDTH * 4 - 200)) as u64, (next(&mut s) % (ROWS * 4 - 120)) as u64);
        let (right, bottom) = (left + 16 + (next(&mut s) % 180) as u64, top + 8 + (next(&mut s) % 110) as u64);
        let id: u64 = if i & 1 == 0 { 0x24 } else { 0x25 };
        list.push((id << 56) | (right << 44) | (bottom << 32) | (tile << 24) | (left << 12) | top);
        let far = i % 3 == 1;
        let (s_start, t_start) = ((next(&mut s) % if far { 0x8000 } else { 1024 }) as u64, (next(&mut s) % if far { 0x8000 } else { 1024 }) as u64);
        let dsdx: u16 = match i % 3 {
            0 => 4 << 10,
            1 => 1 << 10,
            _ => (next(&mut s) & 0x1FFF) as u16,
        };
        let dtdy: u16 = if i % 3 == 2 { (next(&mut s) & 0x1FFF) as u16 } else { 1 << 10 };
        list.push((s_start << 48) | (t_start << 32) | ((dsdx as u64) << 16) | dtdy as u64);
    }
    list.push(0x29 << 56);
    list
}

/// `Fills`: overlapping rectangles in colours whose halves and low bits differ, into a sixteen-bit image and then a thirty-two-bit one.
fn fills(seed: u32, scissor_right: u32) -> Vec<u64> {
    let mut s = seed;
    let mut list = vec![FILL_CYCLE, color_image(FRAMEBUFFER, 2, WIDTH), scissor(0, 0, scissor_right, ROWS * 4), fill_color(0x1357_2468), fill_rectangle(0, 0, WIDTH * 4 - 4, ROWS * 4 - 4)];
    for (image, size) in [(SECOND, 3u64), (FRAMEBUFFER, 2), (SECOND, 3)] {
        list.push(color_image(image, size, WIDTH));
        list.push(scissor(next(&mut s) % 40, next(&mut s) % 40, scissor_right - next(&mut s) % 40, ROWS * 4 - next(&mut s) % 40));
        for _ in 0..60 {
            let (left, top) = (next(&mut s) % (WIDTH * 4), next(&mut s) % (ROWS * 4));
            list.push(fill_color(next(&mut s).wrapping_mul(2).wrapping_add(next(&mut s) & 1)));
            list.push(fill_rectangle(left, top, (left + next(&mut s) % 600).min(0xFFF), (top + next(&mut s) % 400).min(0xFFF)));
        }
    }
    list
}

/// The same bytes under both processors, since a load reads the machine's own memory at either multiple.
fn seed_texture_source(rdram: &mut [u8]) {
    let mut s = 0x00C0_FFEEu32;
    for at in (0..0x4000usize).step_by(4) {
        let word = (next(&mut s) << 8) | (next(&mut s) & 0xFF);
        rdram[TEXTURE_SOURCE as usize + at..TEXTURE_SOURCE as usize + at + 4].copy_from_slice(&word.to_be_bytes());
    }
}

/// `OnTheCpu`: the list on a processor at the multiple, over a shadow of its own.
fn on_the_cpu(list: &[u64], scale: usize) -> (Vec<u8>, Vec<u8>) {
    let mut rdram = vec![0u8; RDRAM_SIZE];
    seed_texture_source(&mut rdram);
    let (mut frame, mut hidden) = (vec![0u8; RDRAM_SIZE * scale * scale], vec![0u8; RDRAM_SIZE / 2 * scale * scale]);
    let mut processor = Rdp::new_scaled(&Rdp::default(), scale as i32);
    let mut memory = RdpMemory::scaled(&mut frame, &mut hidden, &rdram);
    for &word in list {
        processor.accept(word, &mut memory);
    }
    (frame, hidden)
}

/// `OnTheDevice`: the same, the processor recording for the device, and the device's memory read back whole.
fn on_the_device(gpu: &mut GpuRasteriser, list: &[u64], scale: usize) -> (Vec<u8>, Vec<u8>) {
    let mut rdram = vec![0u8; RDRAM_SIZE];
    seed_texture_source(&mut rdram);
    let (mut frame, mut hidden) = (vec![0u8; RDRAM_SIZE * scale * scale], vec![0u8; RDRAM_SIZE / 2 * scale * scale]);
    let mut processor = Rdp::new_scaled(&Rdp::default(), scale as i32);
    processor.shade_on_device(true);
    {
        let mut memory = RdpMemory::scaled(&mut frame, &mut hidden, &rdram).with_gpu(gpu as *mut GpuRasteriser);
        for &word in list {
            processor.accept(word, &mut memory);
        }
    }
    gpu.read(0, frame.len(), &mut frame, &mut hidden);
    (frame, hidden)
}

fn first_difference(a: &[u8], b: &[u8]) -> Option<usize> {
    a.iter().zip(b).position(|(x, y)| x != y).or((a.len() != b.len()).then_some(a.len().min(b.len())))
}

fn assert_identical(cpu: &(Vec<u8>, Vec<u8>), device: &(Vec<u8>, Vec<u8>), where_: &str) {
    if let Some(at) = first_difference(&cpu.0, &device.0) {
        panic!("{where_}: byte {at:X} is {:02X} on the device and {:02X} on the CPU", device.0.get(at).copied().unwrap_or(0), cpu.0.get(at).copied().unwrap_or(0));
    }
    if let Some(at) = first_difference(&cpu.1, &device.1) {
        panic!("{where_}: hidden bits of word {at:X} differ");
    }
}

/// `Fills_on_the_device_are_the_fills_on_the_cpu_byte_for_byte`.
#[test]
fn fills_on_the_device_are_the_fills_on_the_processor_byte_for_byte() {
    for name in devices() {
        for scale in [2usize, 3, 4] {
            let Some(mut gpu) = rasteriser(&name, scale) else { continue };
            for seed in [1u32, 2, 3] {
                gpu.clear();
                let list = fills(seed, WIDTH * 4);
                let cpu = on_the_cpu(&list, scale);
                assert!(cpu.0.iter().any(|&b| b != 0));
                assert_identical(&cpu, &on_the_device(&mut gpu, &list, scale), &format!("seed {seed} at {scale}x on {name}"));
            }
            assert_eq!(gpu.counters.columns_past_the_width, 0);
            assert_eq!(gpu.counters.primitives_not_shaded, 0);
            assert!(gpu.counters.flushes >= 12, "four images a list should end four batches a list, and {} were flushed", gpu.counters.flushes);
            eprintln!("{name} at {scale}x: {} rows in {} flushes", gpu.counters.rows_shaded, gpu.counters.flushes);
        }
    }
}

/// `Shaded_depth_tested_triangles_on_the_device_are_the_cpus_byte_for_byte`: one and two cycles, sixteen and thirty-two bits, keyed.
#[test]
fn shaded_depth_tested_triangles_on_the_device_are_the_processor_s_byte_for_byte() {
    for name in devices() {
        for scale in [2usize, 3, 4] {
            let Some(mut gpu) = rasteriser(&name, scale) else { continue };
            for (seed, size, keyed, two_cycle) in [
                (0x1111_2222u32, 2u64, false, false),
                (0x3333_4444, 3, false, false),
                (0x5555_6666, 2, false, false),
                (0x7777_8888, 2, true, false),
                (0x9999_AAAA, 3, true, false),
                (0xBBBB_CCCC, 2, false, true),
                (0xDDDD_EEEE, 3, false, true),
                (0x1357_9BDF, 2, false, true),
            ] {
                gpu.clear();
                let list = shaded(seed, size, keyed, two_cycle);
                let cpu = on_the_cpu(&list, scale);
                assert_identical(&cpu, &on_the_device(&mut gpu, &list, scale), &format!("seed {seed:08X}, {}-bit, {} cycle, at {scale}x on {name}", if size == 2 { 16 } else { 32 }, if two_cycle { "two" } else { "one" }));
            }
            assert_eq!(gpu.counters.primitives_not_shaded, 0);
            assert_eq!(gpu.counters.columns_past_the_width, 0);
            eprintln!("{name} at {scale}x: {} rows in {} flushes", gpu.counters.rows_shaded, gpu.counters.flushes);
        }
    }
}

/// `Point_sampled_textures_on_the_device_are_the_cpus_byte_for_byte`: no perspective, the divider in its range, and at its edges.
#[test]
fn textures_on_the_device_are_the_processor_s_byte_for_byte() {
    for name in devices() {
        for scale in [2usize, 3, 4] {
            let Some(mut gpu) = rasteriser(&name, scale) else { continue };
            for (seed, mode, two_cycle) in [(0x0BAD_F00Du32, 0, false), (0x1234_ABCD, 1, false), (0x5EED_1234, 2, false), (0x2468_ACE0, 1, false), (0x0FED_CBA9, 0, true), (0x7531_ECA8, 1, true), (0xFACE_B00C, 2, true)] {
                gpu.clear();
                let list = textured(seed, mode, two_cycle);
                let cpu = on_the_cpu(&list, scale);
                assert!(cpu.0[FRAMEBUFFER as usize * scale * scale..].iter().take(WIDTH as usize * ROWS as usize * 2 * scale * scale).any(|&b| b != 0x21));
                assert_identical(&cpu, &on_the_device(&mut gpu, &list, scale), &format!("seed {seed:08X}, mode {mode}, {} cycle, at {scale}x on {name}", if two_cycle { "two" } else { "one" }));
            }
            assert_eq!(gpu.counters.primitives_not_shaded, 0);
            eprintln!("{name} at {scale}x: {} rows in {} flushes", gpu.counters.rows_shaded, gpu.counters.flushes);
        }
    }
}

/// `Copy_mode_rectangles_on_the_device_are_the_cpus_byte_for_byte`.
#[test]
fn copy_mode_rectangles_on_the_device_are_the_processor_s_byte_for_byte() {
    for name in devices() {
        for scale in [2usize, 3, 4] {
            let Some(mut gpu) = rasteriser(&name, scale) else { continue };
            for seed in [0xC0B1_0001u32, 0xC0B1_0002, 0xC0B1_0003] {
                gpu.clear();
                let list = copied(seed);
                let cpu = on_the_cpu(&list, scale);
                assert_identical(&cpu, &on_the_device(&mut gpu, &list, scale), &format!("seed {seed:08X} at {scale}x on {name}"));
            }
            assert_eq!(gpu.counters.primitives_not_shaded, 0);
            eprintln!("{name} at {scale}x: {} rows in {} flushes", gpu.counters.rows_shaded, gpu.counters.flushes);
        }
    }
}

/// `A_scissor_wider_than_the_image_is_clipped_on_the_device_and_counted`: the CPU path lets a span run into the next row's bytes; the device clips it and says how much.
#[test]
fn a_scissor_wider_than_the_image_is_clipped_on_the_device_and_counted() {
    let Some(name) = devices().into_iter().next() else { return };
    let Some(mut gpu) = rasteriser(&name, 2) else { return };
    on_the_device(&mut gpu, &fills(4, (WIDTH + 16) * 4), 2);
    assert!(gpu.counters.columns_past_the_width > 0);
}

/// `The_device_is_never_given_the_machines_own_picture`: only a processor at a multiple shades on the device.
#[test]
fn the_device_is_never_given_the_machine_s_own_picture() {
    let mut processor = Rdp::default();
    let refused = catch_unwind(AssertUnwindSafe(|| processor.shade_on_device(true))).is_err();
    assert!(refused, "a processor at one took the device");
    assert!(!processor.scaled());
}

/// The five binaries are the C# path's files on disk, byte for byte: a rebuilt shader there is a rebuild here, and nothing else can change one.
#[test]
fn the_shaders_are_the_csharp_path_s_files_byte_for_byte() {
    use crate::rdp::gpu::shaders;
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/../Mars - N64/Rdp/Gpu/Shaders");
    for (name, bytes) in [("shade", shaders::SHADE), ("scan", shaders::SCAN), ("clear", shaders::CLEAR), ("average", shaders::AVERAGE), ("add", shaders::ADD)] {
        let on_disk = std::fs::read(format!("{folder}/{name}.spv")).unwrap_or_else(|e| panic!("{name}.spv: {e}"));
        assert!(on_disk == bytes, "{name}.spv differs from the binary this library carries");
    }
}

// Through the interface, as a game reaches the device.

fn hand(m: &mut Machine, list: &[u64], at: u32) {
    for (i, &word) in list.iter().enumerate() {
        m.bus.write64(at + i as u32 * 8, word);
    }
    m.bus.write32(DP_COMMAND_BASE, at);
    m.bus.write32(DP_COMMAND_BASE + 4, at + list.len() as u32 * 8);
    m.join_rdp();
}

fn machine(scale: i32, gpu: bool, workers: usize) -> Machine {
    let mut m = Machine::new(RDRAM_SIZE).unwrap();
    let mut s = 0x00C0_FFEEu32;
    for at in (0..0x4000u32).step_by(4) {
        m.bus.write32(TEXTURE_SOURCE + at, (next(&mut s) << 8) | (next(&mut s) & 0xFF));
    }
    m.set_scale(scale);
    m.set_gpu(gpu);
    if workers > 1 {
        m.set_verify_rdp(true);
        m.set_rdp_workers(workers);
        m.set_threaded_rdp(true);
    }
    m
}

fn shadow(m: &mut Machine) -> (Vec<u8>, Vec<u8>) {
    let len = m.bus.dp.multiple.rdram.len() as u32;
    m.bus.dp.read_back_scaled(0, len);
    (m.bus.dp.multiple.rdram.to_vec(), m.bus.dp.multiple.hidden.to_vec())
}

/// `The_interface_draws_the_multiple_on_the_device_as_it_does_on_the_cpu`: the same lists through the interface with the device on and off leave the same shadow, threaded or not.
#[test]
fn the_interface_draws_the_multiple_on_the_device_as_it_does_on_the_processor() {
    // Without a device the asking is refused with a reason and the processor draws, which is the CI runner's case.
    let no_device = devices().is_empty();
    for (scale, workers) in [(2, 1), (2, 3), (3, 4), (4, 1)] {
        let through = |gpu: bool| {
            let mut m = machine(scale, gpu, workers);
            for list in [shaded(0x2222_7777, 2, false, false), textured(0x3333_8888, 1, false), shaded(0x4444_9999, 2, false, true), copied(0xC0B1_0007)] {
                hand(&mut m, &list, 0x0010_0000);
            }
            assert!(m.bus.dp.scaled_drawn(), "nothing was drawn at the multiple");
            let report = m.bus.dp.gpu_report().to_string();
            (shadow(&mut m), report)
        };
        let (cpu, _) = through(false);
        let (device, report) = through(true);
        if no_device {
            assert!(report.starts_with("Vulkan is not available") || report.starts_with("no Vulkan device"), "{report}");
        } else {
            assert!(!report.contains("Vulkan"), "{report}");
        }
        assert_identical(&cpu, &device, &format!("{scale}x with {workers} workers on {report}"));
        eprintln!("{scale}x with {workers} workers on {report}: identical");
    }
}

/// `At_one_the_device_setting_changes_nothing_and_holds_no_device`.
#[test]
fn at_one_the_device_setting_changes_nothing_and_holds_no_device() {
    let through = |gpu: bool| {
        let mut m = machine(1, gpu, 1);
        hand(&mut m, &shaded(0x5151_0101, 2, false, false), 0x0010_0000);
        (m.bus.rdram.to_vec(), m.bus.rdram_hidden.to_vec(), m.bus.dp.gpu_report().to_string(), m.bus.dp.multiple.can_scan_out())
    };
    let off = through(false);
    let on = through(true);
    assert_eq!(off.2, "off");
    assert_eq!(on.2, "off at one");
    assert!(!on.3);
    assert!(off.0 == on.0, "the machine's own memory changed with the device setting at one");
    assert!(off.1 == on.1);
}

#[allow(clippy::too_many_arguments)]
fn vi_registers(kind: u32, antialias: u32, width: u32, step_x: u32, step_y: u32, left: u32, columns: u32, top: u32, rows: u32, bias_x: u32, bias_y: u32) -> [u32; 14] {
    let mut r = [0u32; 14];
    r[0] = (kind & 3) | ((antialias & 3) << 8);
    r[1] = FRAMEBUFFER;
    r[2] = width;
    r[6] = 525;
    r[9] = ((left & 0x3FF) << 16) | ((left + columns) & 0x3FF);
    r[10] = ((top & 0x3FF) << 16) | ((top + rows * 2) & 0x3FF);
    r[12] = ((bias_x & 0xFFF) << 16) | (step_x & 0xFFF);
    r[13] = ((bias_y & 0xFFF) << 16) | (step_y & 0xFFF);
    r
}

fn with(mut r: [u32; 14], control: u32, sync: u32) -> [u32; 14] {
    r[0] |= control;
    r[6] = sync;
    r
}

fn program(m: &mut Machine, registers: &[u32; 14]) {
    for (i, &word) in registers.iter().enumerate() {
        m.bus.write32(VI_BASE + i as u32 * 4, word);
    }
}

/// One scan, at once or deferred and then joined, and the frame it shows.
fn scan(m: &mut Machine, out: &mut Scanout, deferred: bool) -> Vec<u8> {
    if deferred {
        m.present(out);
        m.join_presentation(out);
    } else {
        m.present_now(out);
    }
    out.frame.clone()
}

fn assert_same_frame(cpu: &[u8], device: &[u8], where_: &str) {
    assert_eq!(cpu.len(), device.len(), "{where_}: the frames differ in size");
    if let Some(at) = first_difference(cpu, device) {
        panic!("{where_}: byte {at} is {:02X} on the device and {:02X} on the CPU (pixel {}, channel {})", device[at], cpu[at], at / 4, at % 4);
    }
}

/// `The_scan_out_on_the_device_is_the_cpus_byte_for_byte`: every filter the VI has, over sixteen- and thirty-two-bit frame buffers, at once and deferred.
#[test]
fn the_scan_out_on_the_device_is_the_processor_s_byte_for_byte() {
    if devices().is_empty() {
        return;
    }
    let modes: [([u32; 14], u64); 13] = [
        (vi_registers(3, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), 3),
        (vi_registers(3, 1, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), 3),
        (with(vi_registers(3, 0, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525), 3),
        (with(vi_registers(3, 1, 320, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40), DIVOT_ON | GAMMA_ON, 525), 3),
        (vi_registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), 2),
        (vi_registers(2, 1, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), 2),
        (vi_registers(2, 2, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), 2),
        (vi_registers(2, 3, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), 2),
        (with(vi_registers(2, 0, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0), DITHER_FILTER, 525), 2),
        (with(vi_registers(2, 1, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0), DIVOT_ON, 525), 2),
        (with(vi_registers(2, 0, 320, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40), DITHER_FILTER | DIVOT_ON | GAMMA_ON, 525), 2),
        (with(vi_registers(2, 1, 320, 0x200, 0x355, 108, 640, 44, 288, 0, 0), DITHER_FILTER | DIVOT_ON, 625), 2),
        (with(vi_registers(2, 0, 320, 0x333, 0x2CC, 100, 400, 30, 260, 0x155, 0x0AA), GAMMA_ON, 525), 2),
    ];
    for scale in [2, 3, 4] {
        for deferred in [false, true] {
            let through = |gpu: bool, workers: usize, registers: &[u32; 14], size: u64| {
                let mut m = machine(scale, gpu, workers);
                m.set_deferred(deferred);
                hand(&mut m, &shaded(0x7E57_5CA2, size, false, false), 0x0010_0000);
                program(&mut m, registers);
                let mut out = Scanout::default();
                scan(&mut m, &mut out, deferred)
            };
            for (i, (registers, size)) in modes.iter().enumerate() {
                let cpu = through(false, 1, registers, *size);
                assert!(cpu.iter().any(|&b| b != 0));
                // Unthreaded, and on a drain whose leader runs the scan (Mars_Native.md §6.15).
                for workers in [1, 3] {
                    let device = through(true, workers, registers, *size);
                    assert_same_frame(&cpu, &device, &format!("mode {i} at {scale}x {} on {workers} workers", if deferred { "deferred" } else { "immediate" }));
                }
            }
        }
    }
}

/// `Averaging_on_the_device_is_the_cpus_scan_after_scan`: the average over a raster the device keeps, through borders, fades, blanks, fields and repeats (Mars_Gpu.md §15).
#[test]
fn averaging_on_the_device_is_the_processor_s_scan_after_scan() {
    if devices().is_empty() {
        return;
    }
    let bordered = with(vi_registers(2, 0, 320, 0x200, 0x400, 124, 600, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525);
    let shorter = with(vi_registers(2, 0, 320, 0x200, 0x400, 124, 600, 34, 200, 0, 0), DITHER_FILTER | DIVOT_ON, 525);
    let interlaced = with(vi_registers(2, 1, 320, 0x200, 0x200, 108, 640, 34, 240, 0, 0), SERRATED | DIVOT_ON, 525);
    let blank = vi_registers(0, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0);
    let blank_smaller = vi_registers(0, 0, 320, 0x200, 0x400, 124, 600, 34, 200, 0, 0);
    let blank_bordered = with(vi_registers(0, 0, 320, 0x200, 0x400, 124, 600, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525);
    let no_signal = vi_registers(2, 0, 320, 0x200, 0x400, 108, 0, 34, 240, 0, 0);
    let off_the_edge = vi_registers(2, 0, 320, 0x200, 0x400, 808, 100, 34, 240, 0, 0);
    let wide = with(vi_registers(3, 0, 320, 0x200, 0x400, 116, 620, 34, 240, 0, 0), GAMMA_ON, 525);
    let steps: [(&[u32; 14], u32, bool); 23] = [
        (&bordered, 0, false),
        (&bordered, 0, false),
        (&shorter, 0, false),
        (&shorter, 0, true),
        (&shorter, 0, false),
        (&interlaced, 0, false),
        (&interlaced, 1, false),
        (&interlaced, 0, true),
        (&interlaced, 1, false),
        (&blank_smaller, 0, false),
        (&bordered, 0, false),
        (&bordered, 0, false),
        (&blank_bordered, 0, false),
        (&off_the_edge, 0, false),
        (&blank, 0, false),
        (&blank, 0, false),
        (&bordered, 0, false),
        (&no_signal, 0, false),
        (&no_signal, 0, false),
        (&bordered, 0, true),
        (&wide, 0, false),
        (&wide, 0, false),
        (&bordered, 0, false),
    ];
    // Every scan walked, as C# walks them with SkipRepeatedScans off, and repeats skipped, where MarsRT still walks one over an edited raster (§6.4.4).
    // Each again on a drain of three, whose leader runs the device's clears, walks and averages (Mars_Native.md §6.15).
    let cases = [(2, 2, false, true), (4, 2, false, true), (4, 4, false, true), (2, 2, true, true), (4, 2, true, true), (4, 4, true, true), (2, 2, true, false), (4, 4, true, false)];
    for (scale, side, deferred, walk_repeats, workers) in cases.iter().flat_map(|&(a, b, c, d)| [(a, b, c, d, 1), (a, b, c, d, 3)]) {
        let make = |gpu: bool| {
            let mut m = machine(scale, gpu, if gpu { workers } else { 1 });
            m.set_deferred(deferred);
            hand(&mut m, &shaded(0x7E57_5CA2, 2, false, false), 0x0010_0000);
            let mut out = Scanout::default();
            out.average = side;
            out.walk_repeats = walk_repeats;
            (m, out)
        };
        let (mut cpu, mut cpu_out) = make(false);
        let (mut device, mut device_out) = make(true);
        assert!(device.bus.dp.multiple.can_scan_out());
        let mut seed = 0x0DDF_00D5u32;
        let step = |m: &mut Machine, out: &mut Scanout, step: &(&[u32; 14], u32, bool), scene: u32| {
            if step.2 {
                hand(m, &shaded(scene, 2, false, false), 0x0011_0000);
            }
            program(m, step.0);
            m.bus.write32(VI_BASE + CURRENT_LINE * 4, step.1);
            scan(m, out, deferred)
        };
        for (k, s) in steps.iter().enumerate() {
            if s.2 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            }
            let (expected, actual) = (step(&mut cpu, &mut cpu_out, s, seed), step(&mut device, &mut device_out, s, seed));
            assert_same_frame(&expected, &actual, &format!("{scale}x averaged by {side}, {}, walking repeats {walk_repeats}, {workers} workers, step {k}", if deferred { "deferred" } else { "immediate" }));
        }
        assert!(device_out.repeated_captures > 0 || !deferred, "some scan repeats the one before it, which is the repeat's own path");
        eprintln!("{scale}x by {side} {}: {} repeated captures", if deferred { "deferred" } else { "immediate" }, device_out.repeated_captures);
    }
}

/// `Averaging_moved_onto_the_device_keeps_the_raster_the_processor_made`: turned on between interlaced fields, the device's raster starts as the processor's.
#[test]
fn averaging_moved_onto_the_device_keeps_the_raster_the_processor_made() {
    if devices().is_empty() {
        return;
    }
    let interlaced = with(vi_registers(2, 1, 320, 0x200, 0x200, 108, 640, 34, 240, 0, 0), SERRATED | DIVOT_ON, 525);
    let covered = |seed: u32| {
        let mut list = vec![FILL_CYCLE, color_image(FRAMEBUFFER, 2, WIDTH), scissor(0, 0, WIDTH * 4, ROWS * 4), fill_color(0x2468_1357), fill_rectangle(0, 0, WIDTH * 4 - 4, ROWS * 4 - 4)];
        list.extend(shaded(seed, 2, false, false));
        list
    };
    for deferred in [false, true] {
        let make = || {
            let mut m = machine(2, false, 1);
            m.set_deferred(deferred);
            hand(&mut m, &covered(0x7E57_5CA2), 0x0010_0000);
            let mut out = Scanout::default();
            out.average = 2;
            (m, out)
        };
        let (mut cpu, mut cpu_out) = make();
        let (mut device, mut device_out) = make();
        let field = |m: &mut Machine, out: &mut Scanout, line: u32| {
            program(m, &interlaced);
            m.bus.write32(VI_BASE + CURRENT_LINE * 4, line);
            scan(m, out, deferred)
        };
        for line in 0..3 {
            assert!(field(&mut cpu, &mut cpu_out, line & 1) == field(&mut device, &mut device_out, line & 1));
        }
        device.set_gpu(true);
        assert!(device.bus.dp.multiple.can_scan_out());
        hand(&mut cpu, &covered(0x0DDF_00D5), 0x0011_0000);
        hand(&mut device, &covered(0x0DDF_00D5), 0x0011_0000);
        for line in 1..4 {
            let (expected, actual) = (field(&mut cpu, &mut cpu_out, line & 1), field(&mut device, &mut device_out, line & 1));
            assert_same_frame(&expected, &actual, &format!("{}, field {line} after the device came on", if deferred { "deferred" } else { "immediate" }));
        }
    }
}

/// `A_walk_left_pending_shows_the_frame_it_captured`: what reaches the device after the capture, a frame or a state read, must not reach the picture the walk shows (Mars_Gpu.md §14).
#[test]
fn a_walk_left_pending_shows_the_frame_it_captured() {
    if devices().is_empty() {
        return;
    }
    for scale in [2, 4] {
        for (state_read_between, workers) in [(false, 1), (true, 1), (false, 3), (true, 3)] {
            let through = |gpu: bool| {
                let mut m = machine(scale, gpu, if gpu { workers } else { 1 });
                m.set_deferred(true);
                hand(&mut m, &shaded(0x7E57_5CA2, 2, false, false), 0x0010_0000);
                program(&mut m, &with(vi_registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525));
                let snapshot = state_read_between.then(|| m.save_state_vec(true).unwrap());
                let mut out = Scanout::default();
                m.present(&mut out);
                if let Some(snapshot) = snapshot {
                    m.restore_state(&snapshot).unwrap();
                } else {
                    hand(&mut m, &shaded(0x0DDF_00D5, 2, false, false), 0x0011_0000);
                    m.bus.dp.read_back_scaled(0, 4);
                }
                m.join_presentation(&mut out);
                out.frame.clone()
            };
            let (cpu, device) = (through(false), through(true));
            assert!(cpu.iter().any(|&b| b != 0));
            assert_same_frame(&cpu, &device, &format!("at {scale}x with {} between, {workers} workers", if state_read_between { "a state read" } else { "a frame drawn" }));
        }
    }
}

/// `A_repeated_capture_walked_again_on_the_device_is_the_cpus`: with every scan walked, a repeat reuses the device's picture only while nothing has replaced it.
#[test]
fn a_repeated_capture_walked_again_on_the_device_is_the_processor_s() {
    if devices().is_empty() {
        return;
    }
    for scale in [2, 4] {
        for (scanned_between, workers) in [(false, 1), (true, 1), (false, 3), (true, 3)] {
            let through = |gpu: bool| {
                let mut m = machine(scale, gpu, if gpu { workers } else { 1 });
                m.set_deferred(true);
                hand(&mut m, &shaded(0x7E57_5CA2, 2, false, false), 0x0010_0000);
                let registers = vi_registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0);
                program(&mut m, &registers);
                let mut out = Scanout::default();
                out.walk_repeats = true;
                let first = scan(&mut m, &mut out, true);
                if scanned_between {
                    program(&mut m, &with(vi_registers(2, 1, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0), DIVOT_ON, 525));
                    assert!(m.present_now(&mut out).walked);
                    program(&mut m, &registers);
                }
                let before = out.repeated_captures;
                let second = scan(&mut m, &mut out, true);
                (first, second, out.repeated_captures > before)
            };
            let (cpu, device) = (through(false), through(true));
            assert!(cpu.2 && device.2, "the second capture repeats the first on both paths");
            assert!(cpu.0 == cpu.1);
            assert_same_frame(&cpu.1, &device.1, &format!("at {scale}x{} the repeated walk, {workers} workers", if scanned_between { " with a scan between" } else { "" }));
        }
    }
}

/// `After_a_state_is_read_the_device_holds_what_the_cpu_path_holds`: a state read empties the memory at the multiple on the device as in the shadow (Mars_Gpu.md §14.3).
#[test]
fn after_a_state_is_read_the_device_holds_what_the_processor_path_holds() {
    if devices().is_empty() {
        return;
    }
    for (shaded_before_the_state, workers) in [(false, 1), (true, 1), (false, 3), (true, 3)] {
        let through = |gpu: bool| {
            let mut m = machine(2, gpu, if gpu { workers } else { 1 });
            assert_eq!(gpu, m.bus.dp.multiple.can_scan_out());
            hand(&mut m, &shaded(0x51A7_E000, 2, false, false), 0x0010_0000);
            if shaded_before_the_state {
                m.bus.dp.read_back_scaled(0, 4);
            }
            let snapshot = m.save_state_vec(true).unwrap();
            m.restore_state(&snapshot).unwrap();
            assert_eq!(gpu, m.bus.dp.multiple.can_scan_out(), "the device did not outlive the state read");
            assert_eq!(gpu, m.bus.dp.gpu_report() != "off", "the report did not outlive the state read: {}", m.bus.dp.gpu_report());
            hand(&mut m, &[FILL_CYCLE, color_image(FRAMEBUFFER, 2, WIDTH), scissor(0, 0, WIDTH * 4, ROWS * 4), fill_color(0x1357_2468), fill_rectangle(0, 0, 255, 127)], 0x0011_0000);
            shadow(&mut m)
        };
        let cpu = through(false);
        assert!(cpu.0.iter().any(|&b| b != 0));
        assert_identical(&cpu, &through(true), &format!("after a state is read, {} before it, {workers} workers", if shaded_before_the_state { "shaded" } else { "recorded" }));
    }
}

// The scan's device work on the drain's leader (Mars_Native.md §6.15).

/// The words of a list written and handed over, without joining the drain.
fn hand_only(m: &mut Machine, list: &[u64], at: u32) {
    for (i, &word) in list.iter().enumerate() {
        m.bus.write64(at + i as u32 * 8, word);
    }
    m.bus.write32(DP_COMMAND_BASE, at);
    m.bus.write32(DP_COMMAND_BASE + 4, at + list.len() as u32 * 8);
}

/// The whole sixteen-bit picture at `FRAMEBUFFER`, a pixel of the device's memory a pixel of the scan.
fn whole_picture(scale: u32) -> crate::rdp::gpu::ScanParameters {
    crate::rdp::gpu::ScanParameters { origin: FRAMEBUFFER * scale * scale, width: WIDTH * scale, step_x: 1 << 10, step_y: 1 << 10, rows: ROWS * scale, columns: WIDTH * scale, anti_alias: 3, ..Default::default() }
}

/// The device work handed to the leader runs after every word handed before it and before every word handed after it, as the scan at
/// once does: the words and the scan are handed over while the workers stand, so none has run when the scan is asked for.
#[test]
fn the_leader_runs_a_scan_after_the_words_handed_before_it_and_before_those_handed_after() {
    if devices().is_empty() {
        return;
    }
    // The last word before the scan fills one rectangle and the first after it another, in the same colour, over a cleared picture.
    let before = [FILL_CYCLE, color_image(FRAMEBUFFER, 2, WIDTH), scissor(0, 0, WIDTH * 4, ROWS * 4), fill_color(0x0843_0843), fill_rectangle(0, 0, WIDTH * 4 - 4, ROWS * 4 - 4), fill_color(0xF83F_F83F), fill_rectangle(40 * 4, 30 * 4, 120 * 4, 90 * 4)];
    let after = [fill_rectangle(160 * 4, 120 * 4, 240 * 4, 180 * 4)];
    for (scale, workers) in [(2, 1), (2, 3), (4, 4)] {
        let run = |threaded: bool| {
            let mut m = Machine::new(RDRAM_SIZE).unwrap();
            m.set_scale(scale as i32);
            m.set_gpu(true);
            if threaded {
                m.set_verify_rdp(true);
                m.set_rdp_workers(workers);
                m.set_threaded_rdp(true);
                m.bus.dp.threads.as_ref().unwrap().hold();
            }
            hand_only(&mut m, &before, 0x0010_0000);
            m.bus.dp.scan_out(&whole_picture(scale));
            hand_only(&mut m, &after, 0x0011_0000);
            if threaded {
                assert!(m.bus.dp.threads.as_ref().unwrap().pending() > 0, "the words ran before the scan was asked for");
                m.bus.dp.threads.as_ref().unwrap().resume();
            }
            let mut words = Vec::new();
            m.bus.dp.multiple.pictures().expect("a device").scanned(|w| words = w.to_vec());
            m.join_rdp();
            words
        };
        let (at_once, on_the_leader) = (run(false), run(true));
        let pixel = |x: u32, y: u32| at_once[(y * scale * WIDTH * scale + x * scale) as usize];
        assert!(pixel(80, 60) != pixel(200, 150) && pixel(200, 150) == pixel(10, 10), "the scan at once shows the first rectangle and not the second");
        let at = at_once.iter().zip(&on_the_leader).position(|(a, b)| a != b);
        assert!(at.is_none() && at_once.len() == on_the_leader.len(), "{scale}x on {workers} workers: the leader's scan differs from the scan at once at pixel {at:?}");
    }
}

/// A test's hold on the leader's device work, let go from another thread after a while.
fn let_go_later(m: &Machine, millis: u64) -> std::thread::JoinHandle<()> {
    let release = m.bus.dp.threads.as_ref().expect("a drain").hold_device();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(millis));
        release();
    })
}

/// A picture read while the leader has not yet run the scan it was handed waits for it, at once and deferred: the leader is held short
/// of its device work for fifty milliseconds while the machine's thread, or the presenter, reads the picture.
#[test]
fn a_picture_read_before_the_leader_has_run_its_scan_waits_for_the_scan() {
    if devices().is_empty() {
        return;
    }
    let registers = with(vi_registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525);
    for (scale, deferred, average) in [(2, true, 1), (2, false, 1), (4, true, 1), (4, true, 2), (4, false, 4)] {
        let through = |workers: usize| {
            let mut m = machine(scale, true, workers);
            m.set_deferred(deferred);
            hand(&mut m, &shaded(0x7E57_5CA2, 2, false, false), 0x0010_0000);
            program(&mut m, &registers);
            let mut out = Scanout::default();
            out.average = average;
            let held = (workers > 1).then(|| let_go_later(&m, 50));
            let frame = scan(&mut m, &mut out, deferred);
            if let Some(held) = held {
                held.join().unwrap();
            }
            frame
        };
        let (unthreaded, on_the_leader) = (through(1), through(3));
        assert!(unthreaded.iter().any(|&b| b != 0));
        assert_same_frame(&unthreaded, &on_the_leader, &format!("{scale}x averaged by {average}, {}", if deferred { "deferred" } else { "at once" }));
    }
}

/// The drain stopped, restarted or replaced by a state read while the leader owes a deferred scan: the scan runs first, and the walk
/// out shows it. The leader is held short of it for a hundred milliseconds while the machine's thread makes the change.
#[test]
fn a_drain_stopped_while_its_leader_owes_a_scan_runs_the_scan_first() {
    if devices().is_empty() {
        return;
    }
    let registers = with(vi_registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525);
    for change in ["unthreaded", "two workers", "a state read"] {
        let through = |workers: usize| {
            let mut m = machine(2, true, workers);
            m.set_deferred(true);
            hand(&mut m, &shaded(0x7E57_5CA2, 2, false, false), 0x0010_0000);
            program(&mut m, &registers);
            let snapshot = m.save_state_vec(true).unwrap();
            let mut out = Scanout::default();
            let held = (workers > 1).then(|| let_go_later(&m, 100));
            m.present(&mut out);
            match change {
                "unthreaded" => m.set_threaded_rdp(false),
                "two workers" => m.set_rdp_workers(2),
                _ => m.restore_state(&snapshot).unwrap(),
            }
            if let Some(held) = held {
                held.join().unwrap();
            }
            m.join_presentation(&mut out);
            out.frame.clone()
        };
        let (unthreaded, on_the_leader) = (through(1), through(3));
        assert!(unthreaded.iter().any(|&b| b != 0));
        assert_same_frame(&unthreaded, &on_the_leader, &format!("{change} while the leader owes the scan"));
    }
}

/// The device read back straight after a scan was handed to the leader, no word between: the read's join waits for the leader's scan,
/// or the two threads meet in the device; an equality cannot see that, ThreadSanitizer can (Mars_Native.md §6.15).
#[test]
fn a_read_back_straight_after_a_scan_waits_for_the_leader_s_scan() {
    if devices().is_empty() {
        return;
    }
    let registers = with(vi_registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 240, 0, 0), DITHER_FILTER | DIVOT_ON, 525);
    let through = |workers: usize| {
        let mut m = machine(2, true, workers);
        m.set_deferred(true);
        let mut out = Scanout::default();
        let mut shadows = Vec::new();
        for scene in 0..8u32 {
            hand(&mut m, &shaded(0x7E57_5CA2 ^ scene, 2, false, false), 0x0010_0000);
            program(&mut m, &registers);
            m.present(&mut out);
            m.bus.dp.read_back_scaled(FRAMEBUFFER * 4, WIDTH * 2 * 4 * 16);
            shadows.push(m.bus.dp.multiple.rdram[(FRAMEBUFFER * 4) as usize..(FRAMEBUFFER * 4 + WIDTH * 2 * 4 * 16) as usize].to_vec());
        }
        m.join_presentation(&mut out);
        (shadows, out.frame.clone())
    };
    let (unthreaded, on_the_leader) = (through(1), through(3));
    assert!(unthreaded.0 == on_the_leader.0, "the shadow read back differs");
    assert_same_frame(&unthreaded.1, &on_the_leader.1, "the last scan");
}

/// A join returns only once the leader has run every scan handed to it, so the device is the machine's thread's again after it: the
/// leader is held short of the scan while the join is asked, with no word left to run, and the device's own count of scans is read.
#[test]
fn a_join_waits_for_the_scans_handed_to_the_leader() {
    if devices().is_empty() {
        return;
    }
    let mut m = machine(2, true, 3);
    hand(&mut m, &shaded(0x7E57_5CA2, 2, false, false), 0x0010_0000);
    let scans = |m: &Machine| m.bus.dp.multiple.gpu.as_ref().expect("a device").counters.scans;
    let before = scans(&m);
    let held = let_go_later(&m, 50);
    m.bus.dp.scan_out(&whole_picture(2));
    m.join_rdp();
    let after = scans(&m);
    held.join().unwrap();
    assert_eq!(after, before + 1, "the join returned before the leader ran the scan it was handed");
}

/// The stop-or-go measurement of Mars_Gpu.md §6.5 on MarsRT, by hand with `EMUSEN_MARS_GPU_BENCH=1`: not a test of anything.
#[test]
fn bench_shading_at_a_multiple_on_the_processor_and_on_the_device() {
    if !std::env::var("EMUSEN_MARS_GPU_BENCH").is_ok_and(|v| v == "1") {
        return;
    }
    let Some(name) = devices().into_iter().next() else { return };
    for scale in [2usize, 4] {
        let Some(mut gpu) = rasteriser(&name, scale) else { continue };
        for (what, list) in [("72 large", shaded(0x1111_2222, 2, false, false)), ("2000 small", small(0x2222_3333, 2000))] {
            let (mut cpu, mut total) = (Vec::new(), Vec::new());
            for _ in 0..9 {
                let started = std::time::Instant::now();
                on_the_cpu(&list, scale);
                cpu.push(started.elapsed().as_secs_f64() * 1e3);
                let started = std::time::Instant::now();
                on_the_device(&mut gpu, &list, scale);
                total.push(started.elapsed().as_secs_f64() * 1e3);
            }
            let median = |v: &mut Vec<f64>| {
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                v[v.len() / 2]
            };
            eprintln!("{scale}x, {what}: CPU one worker {:.1} ms; device {:.1} ms including the readback of the whole memory", median(&mut cpu), median(&mut total));
        }
    }
}
