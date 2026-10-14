//! Stage E5's true colour (Diamond_Enhancements.md §40): a shaded gradient's levels and banding with it and without, a
//! picture of what is drawn at the console's own five bits shown as the console shows it, and the three bits it keeps
//! carried by a blend and by a texture read from what was drawn. A machine without a Vulkan device stands each test down.

use std::any::Any;

use diamond::machine::{Config, Gpu as Chip, Machine};
use diamond::media::{Market, Media};
use diamond_renderer::{Options, Renderer};

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

/// A program that shows VRAM's (0, 0) at 320 by 240, clears the top half to black, sends `words` once and waits; the
/// drawing mode `e1` (dithering is its bit 9).
fn program(e1: u32, words: &[u32]) -> String {
    let mut s = String::from("lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\n");
    s += &(gp1(0) + &gp1(0x0800_0001) + &gp1(0x0300_0000));
    for w in [e1, 0xE300_0000, 0xE400_0000 | 1023 | 511 << 10, 0xE500_0000, 0xE600_0000, 0x0200_0000, 0, 0x0100_0200, 0x0200_0000, 0x200, 0x0100_0200] {
        s += &gp0(w);
    }
    for &w in words {
        s += &gp0(w);
    }
    s + "loop: j loop\nnop\n"
}

/// The program run three frames with the renderer at `options` attached, `prepare` having set the machine first: the
/// machine, the renderer's picture, its samples and its plane.
fn run(e1: u32, words: &[u32], options: Options, prepare: impl Fn(&mut Machine)) -> Option<(Machine, Vec<u8>, Vec<u32>, Vec<u32>)> {
    let r = match Renderer::with(None, options) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no Vulkan device ({e}): not run");
            return None;
        }
    };
    let code = emusen_mips::assemble(&program(e1, words), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = diamond::exe::build(&code);
    let mut m = Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Chip::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    prepare(&mut m);
    m.attach(Box::new(r));
    for _ in 0..3 {
        m.advance();
    }
    let layer: &mut dyn Any = m.layer().expect("attached");
    let r = layer.downcast_mut::<Renderer>().expect("the renderer");
    let picture = r.picture().0.to_vec();
    let samples = r.samples().to_vec();
    let fine = r.fine().to_vec();
    Some((m, picture, samples, fine))
}

const COLOUR: Options = Options { scale: 1, colour: true };

/// Along a row of a picture 320 wide, one channel: how many distinct values, the mean run of equal values, and how many
/// times it falls where the gradient only rises.
fn row_figures(picture: &[u8], row: usize, channel: usize, from: usize, to: usize) -> (usize, f64, usize) {
    let v: Vec<u8> = (from..to).map(|x| picture[(row * 320 + x) * 4 + channel]).collect();
    let mut d = v.clone();
    d.sort();
    d.dedup();
    let runs = 1 + v.windows(2).filter(|w| w[0] != w[1]).count();
    (d.len(), v.len() as f64 / runs as f64, v.windows(2).filter(|w| w[1] < w[0]).count())
}

/// A Gouraud gradient from black to (255, 128, 64) across 256 pixels, dithered as games draw them: the console's
/// picture has 32 levels of red with the dither's grain between them, falling back hundreds of times along a row that
/// only rises; with true colour every level of red is there, and red never falls (P-E5f).
#[test]
fn a_shaded_gradient_keeps_its_levels_with_true_colour() {
    let quad = [0x3800_0000, vertex(32, 40), 0x0040_80FF, vertex(288, 40), 0x0000_0000, vertex(32, 120), 0x0040_80FF, vertex(288, 120)];
    let dithered = 0xE100_0400 | 1 << 9;
    let Some((m, off, _, _)) = run(dithered, &quad, Options::default(), |_| {}) else { return };
    let (_, on, _, _) = run(dithered, &quad, COLOUR, |_| {}).unwrap();
    assert!(off == m.board.gpu.display.frame(), "with true colour off the picture is the console's");
    let (mut off_f, mut on_f) = ((0, 0.0, 0), (0, 0.0, 0));
    for row in 50..110 {
        let (a, b) = (row_figures(&off, row, 0, 32, 288), row_figures(&on, row, 0, 32, 288));
        off_f = (off_f.0.max(a.0), off_f.1 + a.1 / 60.0, off_f.2 + a.2);
        on_f = (on_f.0.max(b.0), on_f.1 + b.1 / 60.0, on_f.2 + b.2);
    }
    eprintln!("red along 60 rows of 256 pixels (distinct levels, mean run of one value px, falls): console {off_f:?}, true colour {on_f:?}");
    assert!(off_f.0 <= 32 && on_f.0 >= 200 && on_f.2 == 0 && off_f.2 > 100, "{off_f:?} {on_f:?}");
}

/// What is drawn at the console's own precision shows as the console shows it: an uploaded picture, a fill of a colour
/// with low bits, and flat polygons of colours the five bits hold, with true colour on, equal to the console's picture
/// pixel for pixel (`min(255, 33v >> 5)` is `c<<3 | c>>2` for `v = 8c`, §40).
#[test]
fn what_the_console_draws_alike_shows_alike() {
    let words = [0x0280_8080, vertex(0, 0), 0x0040_0040, 0x2000_F8F8, vertex(80, 10), vertex(200, 30), vertex(120, 200), 0x2008_1018, vertex(10, 100), vertex(300, 120), vertex(50, 230)];
    let upload = |m: &mut Machine| {
        for y in 0..64 {
            for x in 0..64 {
                m.board.gpu.vram.set(200 + x, 150 + y, (x * 997 + y * 31) as u16 & 0x7FFF);
            }
        }
    };
    let Some((m, on, _, _)) = run(0xE100_0400, &words, COLOUR, upload) else { return };
    let differ = on.chunks(4).zip(m.board.gpu.display.frame().chunks(4)).filter(|(a, b)| a != b).count();
    eprintln!("{differ} pixels differ from the console's picture");
    assert_eq!(differ, 0);
}

/// The three bits kept: a gradient drawn twice, and a semi-transparent rectangle over the second, each sample its blend
/// at eight bits with what was there; and the first read back by a raw sprite as a 15-bit texture, each sample with
/// its plane's bits, but where its halfword is 0000h, transparent as on the console.
#[test]
fn a_blend_and_a_texture_read_back_carry_the_low_bits() {
    let page = 10u32;
    let gradient = |x: i32, y: i32| [0x3800_0000, vertex(x, y), 0x0010_A0F0, vertex(x + 63, y), 0x00F0_1040, vertex(x, y + 63), 0x0080_8080, vertex(x + 63, y + 63)];
    let mut words = vec![0xE100_0400 | page | 2 << 7];
    words.extend(gradient(640, 0));
    words.extend(gradient(640, 100));
    words.extend([0xE100_0400 | page | 2 << 7 | 1 << 5, 0x6200_6090, vertex(640, 100), 0x0040_0040]);
    words.extend([0xE100_0400 | page | 2 << 7, 0x6580_8080, vertex(10, 10), 0, 0x0040_0040]);
    let Some((_, _, s, f)) = run(0xE100_0400, &words, COLOUR, |_| {}) else { return };
    let eight = |at: usize| -> [i32; 3] { std::array::from_fn(|k| ((s[at] >> (5 * k) & 31) << 3 | (f[at] >> (3 * k) & 7)) as i32) };
    let (mut lows, mut blended, mut read) = (0, 0, 0);
    for y in 0..64 {
        for x in 0..64 {
            let (a, b, c) = ((y * 1024 + 640 + x) as usize, ((100 + y) * 1024 + 640 + x) as usize, ((10 + y) * 1024 + 10 + x) as usize);
            lows += (f[a] != 0) as u32;
            let glass = [0x90, 0x60, 0x00];
            let want: [i32; 3] = std::array::from_fn(|k| (eight(a)[k] + glass[k]).min(255));
            blended += (eight(b) != want) as u32;
            // A texel whose halfword is 0000h is transparent, as the console tests it, whatever its low bits.
            read += if s[a] as u16 == 0 { s[c] as u16 != 0 || f[c] != 0 } else { s[c] as u16 != s[a] as u16 || f[c] != f[a] } as u32;
        }
    }
    eprintln!("{lows} samples of the gradient with low bits; {blended} blended otherwise than at eight bits; {read} read back otherwise");
    assert!(lows > 2000 && blended == 0 && read == 0);
}
