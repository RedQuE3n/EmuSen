//! Stage E5's internal resolution (Diamond_Enhancements.md §39): the renderer drawing at 2x, 3x and 4x, each sample
//! against a reference computed here from the rules of §39.1 and §39.2. Triangles cover the samples their edges hold;
//! sprites, rectangles and quads drawn a texel a pixel stay on the console's grid, each block the console's pixel; a
//! picture drawn at N x and read back as a texture, by a sprite or a quad, or by a copy, keeps its samples; the mask bit
//! and semi-transparency act on each sample. A machine without a Vulkan device stands each test down with a line.

use std::any::Any;

use diamond::machine::{Config, Gpu as Chip, Machine};
use diamond::media::{Market, Media};
use diamond_renderer::{Options, Renderer};
use precision::{Precise, Precision};

#[path = "support/goldens.rs"]
mod goldens;
use goldens::*;

const SCALES: [u32; 3] = [2, 3, 4];

fn renderer(scale: u32) -> Option<Renderer> {
    match Renderer::with(None, Options { scale, ..Options::default() }) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("no Vulkan device ({e}): not run");
            None
        }
    }
}

fn li(r: &str, v: u32) -> String {
    format!("lui {r}, 0x{:X}\nori {r}, {r}, 0x{:X}\n", v >> 16, v & 0xFFFF)
}

fn gp0(v: u32) -> String {
    li("t0", v) + "sw t0, 0x1810(a0)\n"
}

fn gp1(v: u32) -> String {
    li("t0", v) + "sw t0, 0x1814(a0)\n"
}

/// The background every program clears VRAM's top half to first, and its halfword.
const BACK: u32 = 0x0010_2040;
const BACK_HALF: u16 = 0x0888;

/// A program that sets the GPU up (the whole area, no offset, no mask, drawing to the displayed area allowed, dithering
/// off), clears the top half of VRAM, sends `words` to GP0 once and then waits.
fn program(words: &[u32]) -> String {
    let mut s = String::from("lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\n");
    s += &(gp1(0) + &gp1(0x0800_0001) + &gp1(0x0300_0000));
    for w in [0xE100_0400, 0xE300_0000, 0xE400_0000 | 1023 | 511 << 10, 0xE500_0000, 0xE600_0000, 0x0200_0000 | BACK, 0, 0x0100_0200, 0x0200_0000 | BACK, 0x200, 0x0100_0200] {
        s += &gp0(w);
    }
    for &w in words {
        s += &gp0(w);
    }
    s + "loop: j loop\nnop\n"
}

/// The program run two frames with the renderer at `scale` attached from power-on, after `prepare` set the machine's
/// VRAM; the machine, and every sample of the renderer's mirror.
fn run(words: &[u32], scale: u32, prepare: impl Fn(&mut Machine)) -> Option<(Machine, Vec<u32>)> {
    let r = renderer(scale)?;
    let code = emusen_mips::assemble(&program(words), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = diamond::exe::build(&code);
    let mut m = Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Chip::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    prepare(&mut m);
    m.attach(Box::new(r));
    for _ in 0..2 {
        m.advance();
    }
    let layer: &mut dyn Any = m.layer().expect("attached");
    let samples = layer.downcast_mut::<Renderer>().expect("the renderer").samples().to_vec();
    Some((m, samples))
}

/// The sample at (x, y) of a mirror at `scale`, the console's halfword.
fn sample(s: &[u32], scale: u32, x: u32, y: u32) -> u16 {
    s[(y * 1024 * scale + x) as usize] as u16
}

fn vertex(x: i32, y: i32) -> u32 {
    (x as u16 as u32) | (y as u16 as u32) << 16
}

/// Whether the point (x/N, y/N) lies in the triangle `v` by the console's rule: every edge function of the triangle
/// wound with positive area at least 0 on an edge it owns, a top or a left one, and at least 1 otherwise (§39.1).
fn covers(v: [(i32, i32); 3], n: i32, x: i32, y: i32) -> bool {
    let area = (v[1].0 - v[0].0) * (v[2].1 - v[0].1) - (v[2].0 - v[0].0) * (v[1].1 - v[0].1);
    let order = if area > 0 { [0, 1, 2] } else { [0, 2, 1] };
    (0..3).all(|e| {
        let (a, b) = (v[order[e]], v[order[(e + 1) % 3]]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let own = dy < 0 || (dy == 0 && dx > 0);
        dx * (y - n * a.1) - dy * (x - n * a.0) >= if own { 0 } else { 1 }
    })
}

fn half(rgb: u32) -> u16 {
    ((rgb >> 3 & 31) | (rgb >> 11 & 31) << 5 | (rgb >> 19 & 31) << 10) as u16
}

/// Flat triangles of every slope and winding, some sharing edges, against the samples their edges hold, drawn in order.
#[test]
fn triangles_at_n_cover_the_samples_their_edges_hold() {
    let triangles: [((i32, i32), (i32, i32), (i32, i32), u32); 6] = [
        ((10, 10), (70, 20), (30, 60), 0x0000_00F8),
        ((70, 20), (30, 60), (90, 75), 0x0000_F800),
        ((100, 5), (101, 90), (160, 40), 0x00F8_0000),
        ((170, 10), (230, 10), (200, 11), 0x00F8_F8F8),
        ((240, 80), (180, 30), (250, 20), 0x0080_8000),
        ((20, 100), (300, 103), (21, 140), 0x0000_8080),
    ];
    let mut words = Vec::new();
    for &(a, b, c, rgb) in &triangles {
        words.extend([0x2000_0000 | rgb, vertex(a.0, a.1), vertex(b.0, b.1), vertex(c.0, c.1)]);
    }
    for scale in SCALES {
        let Some((_, s)) = run(&words, scale, |_| {}) else { return };
        let n = scale as i32;
        let mut wrong = 0;
        for y in 0..150 * n {
            for x in 0..320 * n {
                let want = triangles.iter().rev().find(|t| covers([t.0, t.1, t.2], n, x, y)).map_or(BACK_HALF, |t| half(t.3));
                wrong += (sample(&s, scale, x as u32, y as u32) != want) as u32;
            }
        }
        eprintln!("{scale}x: {wrong} samples otherwise than their edges hold");
        assert_eq!(wrong, 0, "at {scale}x");
    }
}

/// The texture page at (512, 0), 15-bit, each texel naming its coordinate: bit 15, u in bits 0-7, v in bits 8-14.
fn coordinate_texture(m: &mut Machine) {
    for v in 0..128 {
        for u in 0..256 {
            m.board.gpu.vram.set(512 + u, v, 0x8000 | u as u16 | (v as u16) << 8);
        }
    }
}

/// Sprites, rectangles and quads drawn a texel a pixel, raw and modulated, flipped and not, side by side and touching:
/// every sample of every pixel the console's pixel, so that 2D stays on the console's grid scaled, crisp, each block
/// one texel (§39.2).
#[test]
fn sprites_and_quads_stay_on_the_grid_with_every_block_one_texel() {
    let page = 8u32;
    let mut words = vec![0xE100_0400 | page | 2 << 7];
    for k in 0..6 {
        let (x, y) = (10 + 16 * k, 10);
        words.extend([0x7D80_8080, vertex(x, y), (k as u32 * 16) | 32 << 8]);
    }
    words.extend([0x6580_8080, vertex(10, 30), 0x0000_4010, 0x0018_0035]);
    words.extend([0x6460_A0C0, vertex(70, 30), 0x0000_2030, 0x0018_0035]);
    for k in 0..4 {
        let (x, y) = (10 + 24 * k, 60);
        let (u, v) = (k as u32 * 24, 40);
        words.extend([0x2D80_8080, vertex(x, y), u | v << 8, vertex(x + 24, y), (u + 24 | v << 8) | (page | 2 << 7) << 16, vertex(x, y + 24), u | (v + 24) << 8, vertex(x + 24, y + 24), (u + 24) | (v + 24) << 8]);
    }
    words.extend([0xE100_0400 | page | 2 << 7 | 1 << 12, 0x7D80_8080, vertex(130, 10), 0x0000_5050]);
    words.extend([0xE100_0400 | page | 2 << 7 | 1 << 13, 0x7580_8080, vertex(150, 10), 0x0000_6060]);
    for scale in SCALES {
        let Some((m, s)) = run(&words, scale, coordinate_texture) else { return };
        let mut wrong = 0;
        for y in 0..100 * scale {
            for x in 0..200 * scale {
                wrong += (sample(&s, scale, x, y) != m.board.gpu.vram.get(x / scale, y / scale)) as u32;
            }
        }
        eprintln!("{scale}x: {wrong} samples not their console pixel");
        assert_eq!(wrong, 0, "at {scale}x");
    }
}

/// A shaded triangle drawn at N x into a page, then read back as a 15-bit texture a texel a pixel by a sprite, a quad
/// and a flipped sprite, and moved by a VRAM-to-VRAM copy, once beside itself and once over itself downward: every
/// sample of each copy is the sample of the picture drawn, and the copy over itself is `vram.rs`' row by row, each
/// console row's samples read before they are written, so that what a game draws and reads back keeps its N x detail
/// (§39.2, §39.3).
#[test]
fn a_picture_drawn_at_n_is_read_back_and_copied_at_n() {
    let page = 10u32;
    let mut words = vec![0xE100_0400 | page | 2 << 7];
    words.extend([0x3000_00FF, vertex(640, 0), 0x0000_FF00, vertex(703, 10), 0x00FF_0000, vertex(650, 63)]);
    words.extend([0x6580_8080, vertex(100, 100), 0, 0x0040_0040]);
    words.extend([0x2D80_8080, vertex(200, 100), 0, vertex(264, 100), 64 | (page | 2 << 7) << 16, vertex(200, 164), 64 << 8, vertex(264, 164), 64 | 64 << 8]);
    words.extend([0xE100_0400 | page | 2 << 7 | 1 << 12, 0x6580_8080, vertex(300, 100), 63, 0x0040_0040]);
    words.extend([0x8000_0000, vertex(640, 0), vertex(100, 200), 0x0040_0040]);
    words.extend([0x8000_0000, vertex(640, 0), vertex(640, 5), 0x0040_0040]);
    for scale in SCALES {
        let Some((_, s)) = run(&words, scale, |_| {}) else { return };
        // The drawn triangle as it was before the copy over itself, from the first copy, which took it whole.
        let drawn = |x: u32, y: u32| sample(&s, scale, 100 * scale + x, 200 * scale + y);
        // The copy over itself as vram.rs makes it, row after row, on the picture as it was, the rows below it background.
        let mut rows: Vec<Vec<u16>> = (0..69 * scale).map(|y| (0..64 * scale).map(|x| if y < 64 * scale { drawn(x, y) } else { BACK_HALF }).collect()).collect();
        for j in 0..64 * scale {
            rows[(j + 5 * scale) as usize] = rows[j as usize].clone();
        }
        let (mut sprite, mut quad, mut flipped, mut over, mut texels) = (0, 0, 0, 0, 0);
        for y in 0..64 * scale {
            for x in 0..64 * scale {
                let d = drawn(x, y);
                texels += (d != BACK_HALF) as u32;
                sprite += (sample(&s, scale, 100 * scale + x, 100 * scale + y) != d) as u32;
                quad += (sample(&s, scale, 200 * scale + x, 100 * scale + y) != d) as u32;
                flipped += (sample(&s, scale, 300 * scale + x, 100 * scale + y) != drawn(64 * scale - 1 - x, y)) as u32;
                over += (sample(&s, scale, 640 * scale + x, 5 * scale + y) != rows[(y + 5 * scale) as usize][x as usize]) as u32;
            }
        }
        let distinct = {
            let mut v: Vec<u16> = (0..64 * scale).flat_map(|y| (0..64 * scale).map(move |x| (x, y))).map(|(x, y)| drawn(x, y)).collect();
            v.sort();
            v.dedup();
            v.len()
        };
        eprintln!("{scale}x: {texels} samples drawn, {distinct} values; read back otherwise by the sprite {sprite}, the quad {quad}, the flipped sprite {flipped}; the copy over itself {over}");
        assert_eq!((sprite, quad, flipped, over), (0, 0, 0, 0), "at {scale}x");
        assert!(texels > 1500 * scale * scale, "the triangle drew {texels} samples");
    }
}

/// The mask bit and semi-transparency per sample: a triangle drawn with the mask bit set, a rectangle over it with the
/// bit checked, which leaves exactly the triangle's samples; and a semi-transparent rectangle over a second triangle,
/// each sample the blend of what was there (§39.2).
#[test]
fn the_mask_bit_and_blending_act_on_each_sample() {
    let (a, b) = ([(20, 20), (90, 30), (40, 80)], [(120, 20), (190, 35), (150, 80)]);
    let (ta, tb, over, glass) = (0x0000_00F8, 0x0000_F800, 0x00F8_0000, 0x0080_8080);
    let mut words = vec![0xE600_0001, 0x2000_0000 | ta, vertex(a[0].0, a[0].1), vertex(a[1].0, a[1].1), vertex(a[2].0, a[2].1)];
    words.extend([0xE600_0002, 0x6000_0000 | over, vertex(10, 10), 0x0050_0060, 0xE600_0000]);
    words.extend([0x2000_0000 | tb, vertex(b[0].0, b[0].1), vertex(b[1].0, b[1].1), vertex(b[2].0, b[2].1)]);
    words.extend([0xE100_0400, 0x6200_0000 | glass, vertex(110, 10), 0x0050_0060]);
    for scale in SCALES {
        let Some((_, s)) = run(&words, scale, |_| {}) else { return };
        let n = scale as i32;
        let blend = |back: u16, front: u16| -> u16 { (0..3).fold(0, |o, k| o | (((back >> (5 * k) & 31) + (front >> (5 * k) & 31)) >> 1) << (5 * k)) };
        let mut wrong = 0;
        for y in 10 * n..90 * n {
            for x in 10 * n..200 * n {
                let want = if x < 106 * n {
                    if covers(a, n, x, y) { half(ta) | 0x8000 } else { half(over) }
                } else if x >= 110 * n {
                    blend(if covers(b, n, x, y) { half(tb) } else { BACK_HALF }, half(glass))
                } else {
                    continue;
                };
                wrong += (sample(&s, scale, x as u32, y as u32) != want) as u32;
            }
        }
        eprintln!("{scale}x: {wrong} samples otherwise than masked or blended");
        assert_eq!(wrong, 0, "at {scale}x");
    }
}

/// A state at N x (§42.2): saved by a machine with the renderer at 4x keeping its block, loaded by a second, which then
/// hands the same pictures from its first frame, as the kit's C7 and C8 ask; saved, loaded and saved again it is the
/// same bytes; and a state from another scale seeds the renderer from the machine rather than failing.
#[test]
fn a_state_at_n_continues_its_picture() {
    use emusen_native::ffi::StateMachine;
    let page = 10u32;
    let mut words = vec![0xE100_0400 | page | 2 << 7, 0x3000_00FF, vertex(640, 0), 0x0000_FF00, vertex(703, 10), 0x00FF_0000, vertex(650, 63)];
    words.extend([0x6580_8080, vertex(10, 10), 0, 0x0040_0040, 0x3000_FF00, vertex(100, 20), 0x00FF_0000, vertex(200, 30), 0x0000_00FF, vertex(120, 120)]);
    let make = |scale: u32| {
        let r = renderer(scale)?.keeping(false);
        let code = emusen_mips::assemble(&program(&words), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
        let image = diamond::exe::build(&code);
        let mut m = Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Chip::Pin208 }, [Some("psx.dualshock"), None], &image, None);
        m.attach(Box::new(r));
        Some(m)
    };
    let handed = |m: &mut Machine| {
        let layer: &mut dyn Any = m.layer().unwrap();
        let r = layer.downcast_mut::<Renderer>().unwrap();
        r.frame_end();
        let (p, w, h) = r.handed();
        (p.to_vec(), w, h)
    };
    let saved = |m: &mut Machine| {
        m.settle();
        let mut s = vec![0; m.state_size()];
        let n = m.save_state(&mut s).unwrap();
        s.truncate(n);
        s
    };
    let Some(mut a) = make(4) else { return };
    for _ in 0..3 {
        a.advance();
        handed(&mut a);
    }
    let state = saved(&mut a);
    a.load_state(&state).unwrap();
    assert!(saved(&mut a) == state, "save, load, save gives other bytes");
    let mut b = make(4).unwrap();
    b.load_state(&state).unwrap();
    for f in 0..3 {
        a.advance();
        b.advance();
        let (pa, pb) = (handed(&mut a), handed(&mut b));
        assert!(pa == pb, "frame {f} after the load: the pictures handed differ");
        assert_eq!((pa.1, pa.2), (4 * 320, 4 * 240));
    }
    let mut c = make(2).unwrap();
    c.load_state(&state).unwrap();
    c.advance();
    assert_eq!(handed(&mut c).1, 2 * 320, "a state of another scale seeds the renderer at its own");
    eprintln!("a state at 4x: {} bytes", state.len());
}

/// The renderer at `options` paired with the precision layer, or alone; and the renderer of whichever is attached.
fn attached(m: &mut Machine) -> &mut Renderer {
    let layer: &mut dyn Any = m.layer().expect("attached");
    if layer.is::<Renderer>() {
        return layer.downcast_mut::<Renderer>().unwrap();
    }
    &mut layer.downcast_mut::<Precise<Renderer>>().expect("the pair").drawer
}

/// The eight bench discs by their press scripts to frame 3,000 with every enhancement of `options` on and precise
/// geometry with them, the machine's goldens at every anchor against E1's; with `EMUSEN_DIAMOND_PICTURES` a folder, the
/// renderer's picture and the machine's at each anchor written there for looking at (none is committed).
fn discs_at(options: Options, name: &str) {
    let Some(folder) = std::env::var_os("EMUSEN_DIAMOND_DISCS") else {
        eprintln!("not run: EMUSEN_DIAMOND_DISCS names no folder of the bench discs");
        return;
    };
    if renderer(1).is_none() {
        return;
    }
    let pictures = std::env::var_os("EMUSEN_DIAMOND_PICTURES").map(std::path::PathBuf::from);
    let folder = std::path::PathBuf::from(folder);
    let mut goldens = Vec::new();
    for (file, script) in GOLDEN_DISCS {
        let (anchor, presses, sticks) = press_script(script);
        goldens.push(format!("{file}\t{script}\tanchor {anchor}"));
        let mut m = disc_maker(folder.join(file))();
        m.insert_card(0, None);
        let r = Renderer::with(None, options).unwrap().keeping(true);
        m.attach(Box::new(Precise::new(Precision::default(), r)));
        let mut fold = Fold::new();
        while m.frames < GOLDEN_DISC_FRAMES {
            press(&mut m, &presses, &sticks);
            m.advance();
            attached(&mut m).frame_end();
            fold.frame(&mut m);
            if m.frames % GOLDEN_DISC_ANCHOR == 0 {
                goldens.push(format!("\t{}\tpicture {:016X}\tsound {:016X}\tstate {:016X}", m.frames, fold.picture, fold.sound, state_digest(&m)));
                if let Some(dir) = &pictures {
                    let stem = format!("{}-{:05}", script.trim_end_matches(".txt"), m.frames);
                    let (w, h) = (m.board.gpu.display.width, m.board.gpu.display.height);
                    std::fs::write(dir.join(format!("{stem}.machine.{w}x{h}.rgba")), m.board.gpu.display.frame()).unwrap();
                    let (pic, rw, rh) = attached(&mut m).picture();
                    std::fs::write(dir.join(format!("{stem}.{name}.{rw}x{rh}.rgba")), pic).unwrap();
                }
            }
        }
        eprintln!("{file}: {:?}", attached(&mut m).counts());
    }
    let kept = kept_goldens("discs.txt");
    let differ: Vec<String> = goldens.iter().zip(&kept).filter(|(a, b)| a != b).map(|(a, b)| format!("  made {a}\n  kept {b}")).collect();
    assert!(differ.is_empty() && goldens.len() == kept.len(), "goldens with {name}:\n{}", differ.join("\n"));
}

/// The bench discs at 2x and at 4x with precise geometry: every machine golden unchanged (§42.3).
#[test]
fn the_bench_discs_keep_their_goldens_at_2x_and_4x() {
    discs_at(Options { scale: 2, ..Options::default() }, "2x");
    discs_at(Options { scale: 4, ..Options::default() }, "4x");
}

/// The bench discs with every enhancement on, true colour and filtering among them, at 2x filtering 3D polygons and at
/// 4x everything: every machine golden unchanged (§42.3).
#[test]
fn the_bench_discs_keep_their_goldens_with_every_enhancement() {
    use diamond_renderer::Filter;
    discs_at(Options { scale: 2, colour: true, filter: Filter::Polygons }, "2x-all");
    discs_at(Options { scale: 4, colour: true, filter: Filter::All }, "4x-all");
}
