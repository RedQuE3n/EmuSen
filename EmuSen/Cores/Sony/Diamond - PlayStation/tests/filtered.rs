//! Stage E5's texture filtering (Diamond_Enhancements.md §41): bilinear after the palette on a packed page with no texel
//! of the neighbour mixed in, a cut-out's silhouette the console's with no fringe of colour 0, a texture window wrapping
//! its tile, and the two choices: 3D polygons leaving sprites and quads drawn a texel a pixel as they were, Everything
//! filtering them too. A machine without a Vulkan device stands each test down with a line.

use std::any::Any;

use diamond::machine::{Config, Gpu as Chip, Machine};
use diamond::media::{Market, Media};
use diamond_renderer::{Filter, Options, Renderer};

fn li(r: &str, v: u32) -> String {
    format!("lui {r}, 0x{:X}\nori {r}, {r}, 0x{:X}\n", v >> 16, v & 0xFFFF)
}

fn gp0(v: u32) -> String {
    li("t0", v) + "sw t0, 0x1810(a0)\n"
}

fn gp1(v: u32) -> String {
    li("t0", v) + "sw t0, 0x1814(a0)\n"
}

fn vertex(x: i32, y: i32) -> u32 {
    (x as u16 as u32) | (y as u16 as u32) << 16
}

const BLACK: u16 = 0;

/// The 4-bit page at (512, 0) and its palette at (0, 480): texels 0-15 of each row alternate two reds by column and row,
/// 16-31 are blue, 32-47 are transparent on the left half and green on the right, 48-55 a gradient of greys and 56-63
/// blue again.
fn page(m: &mut Machine) {
    let palette = [0x0000u16, 0x000C, 0x001F, 0x7C00, 0x03E0, 0x0842, 0x1084, 0x18C6, 0x2108, 0x294A, 0x318C, 0x39CE, 0x4210, 0x4A52, 0x5294, 0x5AD6];
    for (i, &c) in palette.iter().enumerate() {
        m.board.gpu.vram.set(i as u32, 480, c);
    }
    for v in 0..64u32 {
        for h in 0..16u32 {
            let texel = |u: u32| -> u16 {
                match u {
                    0..=15 => 1 + ((u + v) & 1) as u16,
                    16..=31 => 3,
                    32..=47 => if u < 40 { 0 } else { 4 },
                    48..=55 => 5 + (u - 48) as u16,
                    _ => 3,
                }
            };
            let word = (0..4).fold(0u16, |w, k| w | texel(h * 4 + k) << (4 * k));
            m.board.gpu.vram.set(512 + h, v, word);
        }
    }
}

/// A program clearing VRAM's top left quarter to black and sending `words` once, the 4-bit page at (512, 0) set, then
/// waiting.
fn program(words: &[u32]) -> String {
    let mut s = String::from("lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\n");
    s += &(gp1(0) + &gp1(0x0800_0001) + &gp1(0x0300_0000));
    for w in [0xE100_0408, 0xE300_0000, 0xE400_0000 | 1023 | 511 << 10, 0xE500_0000, 0xE600_0000, 0x0200_0000, 0, 0x0100_0200] {
        s += &gp0(w);
    }
    for &w in words {
        s += &gp0(w);
    }
    s + "loop: j loop\nnop\n"
}

/// The program at `options`, two frames: every sample of the renderer's mirror.
fn run(words: &[u32], options: Options) -> Option<Vec<u32>> {
    let r = match Renderer::with(None, options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no Vulkan device ({e}): not run");
            return None;
        }
    };
    let code = emusen_mips::assemble(&program(words), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = diamond::exe::build(&code);
    let mut m = Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Chip::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    page(&mut m);
    m.attach(Box::new(r));
    for _ in 0..2 {
        m.advance();
    }
    let layer: &mut dyn Any = m.layer().expect("attached");
    Some(layer.downcast_mut::<Renderer>().expect("the renderer").samples().to_vec())
}

/// A raw textured quad from (x, y), `size` pixels square, of the texels (u0, v0) to (u1, v1) of the page, CLUT (0, 480).
fn quad(x: i32, y: i32, size: i32, (u0, v0): (u32, u32), (u1, v1): (u32, u32)) -> [u32; 9] {
    let (clut, page) = (480 << 6, 8u32);
    [0x2D80_8080, vertex(x, y), u0 | v0 << 8 | clut << 16, vertex(x + size, y), u1 | v0 << 8 | page << 16, vertex(x, y + size), u0 | v1 << 8, vertex(x + size, y + size), u1 | v1 << 8]
}

fn channels(h: u16) -> [u16; 3] {
    [h & 31, h >> 5 & 31, h >> 10 & 31]
}

/// The samples of the square (x, y, size) at `scale`.
fn square(s: &[u32], scale: u32, x: u32, y: u32, size: u32) -> Vec<u16> {
    (y * scale..(y + size) * scale).flat_map(|sy| (x * scale..(x + size) * scale).map(move |sx| s[(sy * 1024 * scale + sx) as usize] as u16)).collect()
}

/// A packed page's region of two reds magnified four times, its blue neighbour beside it in the page: filtered at 1x and
/// 2x, the reds are mixed into shades between them and no sample has any blue.
#[test]
fn a_packed_page_mixes_no_texel_of_its_neighbour() {
    let words = quad(20, 20, 64, (0, 0), (16, 16));
    for scale in [1u32, 2] {
        let Some(s) = run(&words, Options { scale, filter: Filter::Polygons, ..Options::default() }) else { return };
        let q = square(&s, scale, 20, 20, 64);
        let blue = q.iter().filter(|&&h| channels(h)[2] != 0).count();
        let mut shades: Vec<u16> = q.iter().map(|&h| channels(h)[0]).collect();
        shades.sort();
        shades.dedup();
        eprintln!("{scale}x: {} shades of red, {blue} samples with blue", shades.len());
        assert!(blue == 0 && shades.len() > 4, "at {scale}x");
    }
}

/// A cut-out magnified four times, transparent on its left and green on its right: filtered, the samples drawn are those
/// the unfiltered picture draws, every one wholly green, with no fringe of colour 0.
#[test]
fn a_cut_out_keeps_the_consoles_silhouette_with_no_fringe() {
    let words = quad(20, 20, 64, (32, 0), (48, 16));
    for scale in [1u32, 2] {
        let Some(off) = run(&words, Options { scale, ..Options::default() }) else { return };
        let on = run(&words, Options { scale, filter: Filter::Polygons, ..Options::default() }).unwrap();
        let (a, b) = (square(&off, scale, 20, 20, 64), square(&on, scale, 20, 20, 64));
        let silhouette = a.iter().zip(&b).filter(|(x, y)| (**x == BLACK) != (**y == BLACK)).count();
        let fringe = b.iter().filter(|&&h| h != BLACK && h != 0x03E0).count();
        eprintln!("{scale}x: {} samples drawn, {silhouette} of the silhouette otherwise, {fringe} not wholly green", b.iter().filter(|&&h| h != BLACK).count());
        assert_eq!((silhouette, fringe), (0, 0), "at {scale}x");
    }
}

/// A texture window repeating the grey gradient's 8 texels across a quad of four repeats, blue texels past the tile in
/// the page: filtered, each repeat's edge mixes the tile's first texel, never the blue past it.
#[test]
fn a_texture_window_wraps_its_tile() {
    // GP0(E2h): u masked to 8 texels at offset 48 (in units of 8: mask 31 - 0 leaves bits 3-7, offset 6).
    let window = 0xE200_0000 | 0x1F | 6 << 10;
    let mut words = vec![window];
    words.extend(quad(20, 20, 64, (0, 0), (32, 8)));
    let Some(s) = run(&words, Options { filter: Filter::Polygons, ..Options::default() }) else { return };
    let q = square(&s, 1, 20, 20, 64);
    let blue = q.iter().filter(|&&h| channels(h)[2] != channels(h)[0]).count();
    let mut greys: Vec<u16> = q.iter().map(|&h| channels(h)[0]).collect();
    greys.sort();
    greys.dedup();
    eprintln!("{blue} samples not grey, {} greys", greys.len());
    assert!(blue == 0 && greys.len() > 8, "{blue} not grey, {} greys", greys.len());
}

/// The choices: with 3D polygons a sprite and a quad drawn a texel a pixel are what they are unfiltered, sample for
/// sample, while a magnified quad is filtered; with Everything at 2x the sprite is filtered too.
#[test]
fn three_d_polygons_leave_sprites_and_flat_quads_crisp() {
    let mut words = vec![0x6580_8080, vertex(100, 20), 48 | (480 << 6) << 16, 0x0010_0010];
    words.extend(quad(140, 20, 16, (48, 0), (64, 16)));
    words.extend(quad(20, 100, 64, (48, 0), (64, 16)));
    let sprite_and_quad = |s: &[u32], scale: u32| [square(s, scale, 100, 20, 16), square(s, scale, 140, 20, 16)].concat();
    for scale in [1u32, 2] {
        let Some(off) = run(&words, Options { scale, ..Options::default() }) else { return };
        let three_d = run(&words, Options { scale, filter: Filter::Polygons, ..Options::default() }).unwrap();
        let all = run(&words, Options { scale, filter: Filter::All, ..Options::default() }).unwrap();
        let crisp = sprite_and_quad(&off, scale) == sprite_and_quad(&three_d, scale);
        let magnified = square(&off, scale, 20, 100, 64) != square(&three_d, scale, 20, 100, 64);
        let sprite_filtered = square(&off, scale, 100, 20, 16) != square(&all, scale, 100, 20, 16);
        eprintln!("{scale}x: sprite and flat quad unchanged with 3D polygons {crisp}; magnified quad filtered {magnified}; sprite filtered with Everything {sprite_filtered}");
        assert!(crisp && magnified && (scale == 1 || sprite_filtered), "at {scale}x");
        if scale == 1 {
            assert!(!sprite_filtered, "a sprite at 1x reads whole texels, which filtering leaves as they are");
        }
    }
}
