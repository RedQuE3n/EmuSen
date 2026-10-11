//! The GPU's rules as ps1-tests' pictures and logs on a console pinned them (Diamond_GPU.md §5, §6), held here without
//! the corpus.

use super::*;
use crate::video::Video;

fn gpu() -> (Gpu, Video) {
    let video = Video::new(false);
    let mut g = Gpu::new(2, video.clock);
    let mut v = video;
    g.gp1(0, &mut v, 0);
    g.gp0(0xE300_0000, 0, &v);
    g.gp0(0xE407_FFFF, 0, &v);
    (g, v)
}

fn send(g: &mut Gpu, v: &Video, words: &[u32]) {
    for &w in words {
        let t = g.busy_until;
        g.gp0(w, t, v);
    }
    let t = g.busy_until;
    g.run(t, v);
}

fn px(g: &Gpu, x: u32, y: u32) -> u16 {
    g.vram.get(x, y)
}

#[test]
fn a_quads_texture_steps_at_the_half_of_a_truncated_12_bit_gradient() {
    let (mut g, v) = gpu();
    g.vram.set(512, 0, 0x001F);
    g.vram.set(513, 0, 0x03E0);
    for (w, first_green) in [(4, 2), (6, 4), (8, 4), (10, 6), (200, 103), (255, 128)] {
        let y = w;
        send(&mut g, &v, &[0x2C80_8080, y << 16, 0, (y << 16) | w, 0x0108_0001, (y + 1) << 16, 0, (y + 1) << 16 | w, 1]);
        let row: Vec<u16> = (0..w).map(|x| px(&g, x, y)).collect();
        assert_eq!(row.iter().position(|&p| p == 0x03E0), Some(first_green as usize), "width {w}");
        assert!(row[..first_green as usize].iter().all(|&p| p == 0x001F));
    }
}

#[test]
fn shading_takes_the_base_vertexs_value_with_a_half_added() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0x3800_00FF, 0x0000_0000, 0x0000_FF00, 0x0000_000A, 0x0000_00FF, 0x0001_0000, 0x0000_FF00, 0x0001_000A]);
    let red: Vec<u16> = (0..10).map(|x| px(&g, x, 0) & 31).collect();
    assert_eq!(red, [31, 28, 25, 22, 19, 16, 12, 9, 6, 3], "255 down by trunc(255*4096/10) a pixel, a half added");
}

#[test]
fn a_triangle_owns_its_top_and_left_edges_only() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0x2000_FFFF, 223 << 16 | 40, 223 << 16 | 280, 16 << 16 | 160]);
    let span = |y: u32| {
        let xs: Vec<u32> = (0..320).filter(|&x| px(&g, x, y) != 0).collect();
        xs.first().map(|&a| (a, *xs.last().unwrap_or(&a)))
    };
    assert_eq!((span(16), span(17), span(222), span(223)), (None, Some((160, 160)), Some((41, 279)), None));
}

#[test]
fn a_line_draws_both_ends_and_rounds_a_half_by_its_major_axis() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0x4000_00FF, 24 << 16 | 16, 25 << 16 | 96]);
    assert_eq!((px(&g, 16, 24), px(&g, 55, 24), px(&g, 56, 25), px(&g, 96, 25)), (0x1F, 0x1F, 0x1F, 0x1F), "x-major: a half steps");
    send(&mut g, &v, &[0x4000_00FF, 100 << 16 | 118, 180 << 16 | 119]);
    assert_eq!((px(&g, 118, 140), px(&g, 119, 141)), (0x1F, 0x1F), "y-major: a half stays");
}

#[test]
fn an_x_flipped_rectangle_starts_from_u_with_its_low_bit_set() {
    let (mut g, v) = gpu();
    for u in 0..4 {
        g.vram.set(640 + u, 0, 0x100 + u as u16);
    }
    send(&mut g, &v, &[0xE100_110A, 0x6580_8080, 260, 0, 0x0001_0004]);
    assert_eq!((0..4).map(|x| px(&g, 260 + x, 0)).collect::<Vec<_>>(), [0x101, 0x100, 0, 0], "u + 1, u, then 255 and 254, zero here");
    send(&mut g, &v, &[0x6580_8080, 1 << 16 | 261, 3, 0x0001_0004]);
    assert_eq!((0..4).map(|x| px(&g, 261 + x, 1)).collect::<Vec<_>>(), [0x103, 0x102, 0x101, 0x100], "an odd u starts at u");
}

/// A shaded triangle's pixels do not depend on which vertex is sent first, where one vertex is leftmost (D-7).
#[test]
fn a_shaded_triangle_reads_the_same_in_every_vertex_order() {
    let pts = [(2u32, 1u32, 0x0000_0AFAu32), (40, 9, 0x005A_C803), (13, 30, 0x00FF_854D)];
    let orders = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
    let mut pictures = Vec::new();
    for (k, o) in orders.iter().enumerate() {
        let (mut g, v) = gpu();
        let x0 = 64 * k as u32;
        let words: Vec<u32> = o.iter().enumerate().flat_map(|(i, &j)| [(if i == 0 { 0x3000_0000 } else { 0 }) | pts[j].2, pts[j].1 << 16 | (pts[j].0 + x0)]).collect();
        send(&mut g, &v, &words);
        pictures.push((0..32).flat_map(|y| (0..48).map(move |x| (x, y))).map(|(x, y)| px(&g, x0 + x, y)).collect::<Vec<_>>());
    }
    assert!(pictures.iter().all(|p| p == &pictures[0]));
}

#[test]
fn the_clut_cache_keeps_a_palette_until_cleared_or_moved() {
    let (mut g, v) = gpu();
    g.vram.set(0, 1, 0x0001);
    g.vram.set(1, 20, 0x1234);
    let draw = |g: &mut Gpu, y: u32, clut_x: u32| send(g, &v, &[0xE100_0000, 0x6580_8080, y << 16, (20 << 6 | clut_x) << 16 | 0x0100, 0x0001_0001]);
    draw(&mut g, 30, 0);
    g.vram.set(1, 20, 0x7FFF);
    draw(&mut g, 31, 0);
    send(&mut g, &v, &[0x0100_0000]);
    draw(&mut g, 32, 0);
    assert_eq!((px(&g, 0, 30), px(&g, 0, 31), px(&g, 0, 32)), (0x1234, 0x1234, 0x7FFF));
}

#[test]
fn the_fifo_holds_what_arrives_while_a_command_draws() {
    let (mut g, mut v) = gpu();
    let stat = |g: &mut Gpu, v: &Video, t| g.status(v, t) >> 25 & 0xF;
    assert_eq!(stat(&mut g, &v, 10), 0b1010, "ready for a command, the FIFO empty");
    for w in [0x6000_0080, 0, 0x0080_0080] {
        g.gp0(w, 10, &v);
    }
    assert_eq!(stat(&mut g, &v, 11), 0b1000, "drawing: the FIFO empty, not ready for a command");
    for w in [0x6000_8000, 0x80, 0x0080_0080] {
        g.gp0(w, 12, &v);
    }
    g.gp0(0xE300_0400, 12, &v);
    assert_eq!((stat(&mut g, &v, 13), g.len, g.env[2]), (0, 4, 0), "a second rectangle waits, and GP0(E3h) behind it");
    let first = g.busy_until;
    g.gp1(0x0400_0002, &mut v, first);
    assert_eq!((g.len, g.status(&v, first) >> 25 & 1), (1, 0), "the second rectangle taken as the first is spent, GP0(E3h) not yet");
    let second = g.busy_until;
    let dreq = g.status(&v, second) >> 25 & 1;
    assert_eq!((g.len, dreq, g.env[2]), (0, 1, 0x400), "then GP0(E3h), and the FIFO empty");
}

#[test]
fn e1_and_gp1_10h_read_as_the_208_pin_chip_keeps_them() {
    let (mut g, mut v) = gpu();
    send(&mut g, &v, &[0xE100_3FFF]);
    assert_eq!(g.status(&v, 0) & 0x87FF, 0x07FF, "bit 11 held off while GP1(09h) forbids it");
    g.gp1(0x0900_0001, &mut v, 0);
    send(&mut g, &v, &[0xE100_0FFF]);
    assert_eq!(g.status(&v, 0) & 0x87FF, 0x87FF);
    g.gp1(0x1000_0007, &mut v, 0);
    assert_eq!(g.gpuread(&v, 0).0, 2);
    let mut old = Gpu::new(0, v.clock);
    old.latch = 5;
    old.gp1(0x1000_0007, &mut v, 0);
    assert_eq!(old.gpuread(&v, 0).0, 5, "the 160-pin chip has no index 7");
}

#[test]
fn a_cpu_transfer_writes_under_the_mask_and_reads_back() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0xE600_0001, 0xA000_0000, 0x0020_0010, 0x0001_0003, 0x5678_1234, 0x0000_0001, 0xE600_0002]);
    assert_eq!((px(&g, 16, 32), px(&g, 17, 32), px(&g, 18, 32)), (0x9234, 0xD678, 0x8001));
    send(&mut g, &v, &[0xA000_0000, 0x0020_0010, 0x0001_0001, 0x0000_0000, 0xC000_0000, 0x0020_0010, 0x0001_0002]);
    assert_eq!(px(&g, 16, 32), 0x9234, "a masked pixel kept");
    let t = g.busy_until + 1000;
    assert_eq!(g.gpuread(&v, t).0, 0xD678_9234);
}

#[test]
fn the_display_is_the_ranges_size_read_a_line_at_a_time() {
    let (mut g, mut v) = gpu();
    g.vram.set(0, 0, 0x001F);
    g.gp1(0x0300_0000, &mut v, 0);
    g.gp1(0x0800_0001, &mut v, 0);
    g.gp1(0x0600_0000 | (0x260 + 2560) << 12 | 0x260, &mut v, 0);
    let field = v.t_at(263 * crate::video::NTSC_LINE);
    g.sync(field, &v);
    assert_eq!((g.display.width, g.display.height), (320, 240));
    assert_eq!(&g.display.frame()[..4], &[255, 0, 0, 255]);
}
