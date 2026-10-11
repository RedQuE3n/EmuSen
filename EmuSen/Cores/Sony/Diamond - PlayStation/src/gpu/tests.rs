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

/// amidog's POLY, LINE and POLYLINE flags (Diamond_Disputes.md D-10, D-23): bit 28 clear from a polygon's or a line's
/// command word to its last vertex, and while a polyline is open; a rectangle's words leave it set.
#[test]
fn bit_28_clears_while_a_polygons_or_a_lines_vertices_arrive() {
    let (mut g, v) = gpu();
    let wfep = |g: &mut Gpu, t| g.status(&v, t) >> 28 & 1;
    let mut t = 10;
    for (words, open) in [
        (&[0x2000_0080, 0x0010_0010, 0x0010_0011][..], true),
        (&[0x0011_0010][..], false),
        (&[0x4000_0080, 0x0020_0020][..], true),
        (&[0x0020_0021][..], false),
        (&[0x4800_0080, 0x0030_0030, 0x0030_0031, 0x0031_0031][..], true),
        (&[0x5555_5555][..], false),
        (&[0x6000_0080, 0x0040_0040][..], false),
    ] {
        for &w in words {
            t = g.gp0(w, t.max(g.busy_until), &v) + 1;
        }
        assert_eq!(wfep(&mut g, t), (!open) as u32, "after {:08X}", words[words.len() - 1]);
    }
}

/// amidog's textured quads (Diamond_Disputes.md D-23): the page attribute reaches GPUSTAT once the quad's first
/// triangle has its three vertices, before the fourth arrives; a triangle's with its last word.
#[test]
fn a_textured_quads_page_takes_effect_with_its_third_vertex() {
    let (mut g, v) = gpu();
    let page = |g: &mut Gpu, t| g.status(&v, t) & 0x7FF;
    let quad = [0x2C80_8080, 0x0000_0000, 0x0000_0000, 0x0000_0010, 0x0069_0000, 0x0010_0000, 0x0000_0000, 0x0010_0010, 0x0000_0000];
    let mut t = 10;
    let mut seen = Vec::new();
    for &w in &quad {
        t = g.gp0(w, t.max(g.busy_until), &v) + 1;
        seen.push(page(&mut g, t));
    }
    assert_eq!(seen, [0, 0, 0, 0, 0, 0, 0x69, 0x69, 0x69]);
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
    assert_eq!(old.gpuread(&v, 0).0, 0, "the 160-pin chip has no index 7");
}

/// amidog's REG (Diamond_Disputes.md D-23): a word read from VRAM, then GP1(10h) with each index.
#[test]
fn gp1_10h_keeps_the_latchs_high_bits_and_reads_zero_for_no_register() {
    for (version, index, e, wanted) in [
        (2, 2, 0xE200_0000, 0x1230_0000),
        (2, 3, 0xE3FF_FFFF, 0x123F_FFFF),
        (2, 5, 0xE5FF_FFFF, 0x123F_FFFF),
        (2, 5, 0xE500_0000, 0x1200_0000),
        (2, 0, 0xE500_0000, 0),
        (2, 6, 0xE500_0000, 0),
        (2, 9, 0xE500_0000, 0),
        (2, 7, 0xE500_0000, 2),
        (0, 3, 0xE3FF_FFFF, 0x1237_FFFF),
        (0, 6, 0xE500_0000, 0),
        (0, 7, 0xE500_0000, 0),
    ] {
        let video = Video::new(false);
        let mut v = video;
        let mut g = Gpu::new(version, v.clock);
        g.gp1(0, &mut v, 0);
        send(&mut g, &v, &[e, 0xA000_0000, 0x01FF_0000, 0x0001_0002, 0x1234_5678, 0xC000_0000, 0x01FF_0000, 0x0001_0002]);
        let t = g.busy_until + 1000;
        assert_eq!(g.gpuread(&v, t).0, 0x1234_5678);
        g.gp1(0x1000_0000 | index, &mut v, t);
        assert_eq!(g.gpuread(&v, t).0 & 0x7FFF_FFFF, wanted, "version {version}, index {index}, {e:08X}");
    }
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

/// amidog's VRAM READ (Diamond_Disputes.md D-23): a read of one pixel ends with the pixel after it in its row, and the
/// GPU takes no command from GP0(C0h)'s words to the last of its data.
#[test]
fn an_odd_read_ends_with_the_next_pixel_and_holds_the_gpu_until_read() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0xA000_0000, 0, 0x0001_0002, 0x0102_0304, 0xC000_0000, 0, 0x0001_0001]);
    let t = g.busy_until + 1000;
    assert_eq!(g.status(&v, t) >> 26 & 3, 2, "VRAM ready to send, no command taken");
    assert_eq!(g.gpuread(&v, t).0, 0x0102_0304);
    assert_eq!(g.status(&v, t) >> 26 & 3, 1, "the read done");
}

/// amidog's OFFSET (Diamond_Disputes.md D-23): a vertex and the drawing offset add in 11 bits, wrapping, so -992 and
/// -1022 put a rectangle at 34.
#[test]
fn a_vertex_and_the_offset_add_in_eleven_bits() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0xE500_0402, 0x6080_8080, 0x0002_0420, 0x0001_0002]);
    assert_eq!((px(&g, 33, 2), px(&g, 34, 2), px(&g, 35, 2), px(&g, 36, 2)), (0, 0x4210, 0x4210, 0));
}

/// amidog's OFFSET (Diamond_Disputes.md D-23): on the 208-pin chip the area reaches line 1023, and a line past 511 is
/// drawn on VRAM's line modulo 512 while GP1(09h) is clear (psx-spx "GP1(09h)": "wrap to 0-511").
/// amidog's OFFSET for polygons (Diamond_Disputes.md D-23): the offset moves a primitive whole, its vertices keeping
/// the distances the command gave, which the size limit judges: 32 to 1007 spans 290 to 1265 after an offset of 258,
/// and 288 to -785, 1073 apart, is not drawn though the offset of 2 brings both ends within 11 bits of each other.
#[test]
fn the_offset_moves_a_polygon_whole_and_its_size_is_judged_before() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0xE500_0102, 0x2080_8080, 0x0002_0020, 0x0002_03EF, 0x0022_0020]);
    assert_eq!((px(&g, 289, 2), px(&g, 290, 2), px(&g, 1023, 2), px(&g, 0, 2)), (0, 0x4210, 0x4210, 0));
    send(&mut g, &v, &[0xE500_0002, 0x2080_8080, 0x0040_0120, 0x0040_04EF, 0x0060_0120]);
    assert_eq!((px(&g, 0, 64), px(&g, 290, 64), px(&g, 1023, 64)), (0, 0, 0));
    send(&mut g, &v, &[0xE500_0102, 0x4880_8080, 0x0080_0020, 0x0080_03EF, 0x00A0_0020, 0x5555_5555]);
    assert_eq!((px(&g, 1023, 136), px(&g, 290, 160)), (0x4210, 0x4210), "a polyline's next segment from where the last ended");
}

#[test]
fn rows_past_511_wrap_into_vram_inside_a_tall_area() {
    let (mut g, mut v) = gpu();
    send(&mut g, &v, &[0xE40F_FFFF, 0x6080_8080, 0x0222_0002, 0x0001_0001]);
    assert_eq!(px(&g, 2, 34), 0x4210, "line 546 drawn on line 34");
    g.gp1(0x0900_0001, &mut v, g.busy_until);
    send(&mut g, &v, &[0x6080_8080, 0x0222_0003, 0x0001_0001]);
    assert_eq!(px(&g, 3, 34), 0, "not with the second megabyte allowed");
}

/// amidog's LINE and OFFSET (Diamond_Disputes.md D-23): an x-major line's ties round toward its right end, so rising from
/// (11,20) to (15,17) x 13 takes line 18, and from (418,2) leftward to (290,34) x 416 keeps line 2; a y-major line's
/// round toward less x, so from (18,10) to (11,20) line 15 takes x 14.
#[test]
fn a_lines_ties_round_toward_its_right_end_across_and_toward_less_x_down() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0x4000_00FF, 0x0014_000B, 0x0011_000F, 0x4000_FF00, 0x000A_0012, 0x0014_000B]);
    assert_eq!((px(&g, 13, 18), px(&g, 13, 19)), (0x001F, 0));
    assert_eq!((px(&g, 14, 15), px(&g, 15, 15)), (0x03E0, 0));
    send(&mut g, &v, &[0x4000_00FF, 0x0002_01A2, 0x0022_0122]);
    assert_eq!((px(&g, 416, 2), px(&g, 416, 3), px(&g, 415, 3)), (0x001F, 0, 0x001F));
}

/// amidog's XYFLIP (Diamond_Disputes.md D-23): an unflipped textured rectangle's pixel at an odd column whose texel lies
/// in another texture cache entry than its left neighbour's takes that neighbour's texel; a 15-bit entry holds four.
#[test]
fn a_rectangles_pixel_pair_reads_its_texels_from_one_cache_entry() {
    let (mut g, v) = gpu();
    for u in 0..16u32 {
        g.vram.set(64 + u, 0, 0x8000 | u as u16);
    }
    send(&mut g, &v, &[0xE100_0101, 0x6500_0000, 0x0000_0008, 0x0000_0003, 0x0001_0008]);
    let row: Vec<u16> = (8..16).map(|x| px(&g, x, 0) & 0xFF).collect();
    assert_eq!(row, [3, 3, 5, 6, 7, 7, 9, 10]);
    send(&mut g, &v, &[0xE100_1101, 0x6500_0000, 0x0001_0008, 0x0000_0003, 0x0001_0004]);
    let flipped: Vec<u16> = (8..12).map(|x| px(&g, x, 1) & 0xFF).collect();
    assert_eq!(flipped, [3, 2, 1, 0], "an x-flipped rectangle is read texel by texel");
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

/// amidog's INTERLACE (Diamond_Disputes.md D-23): in 480-line interlace with drawing to the displayed field prohibited, a
/// fill and a rectangle leave alone the lines whose parity, counted from the display's first line, is GPUSTAT bit 31's,
/// which reads 0 in vblank whichever field is displayed.
#[test]
fn interlaced_drawing_skips_bit_31s_lines_counted_from_the_display_start() {
    for (start_y, skipped) in [(0u32, 0u32), (1, 1)] {
        let (mut g, mut v) = gpu();
        g.gp1(0x0800_0027, &mut v, 0);
        g.gp1(0x0500_0000 | start_y << 10, &mut v, 0);
        v.odd = true;
        assert_eq!(g.status(&v, 0) >> 31, 0, "vblank");
        send(&mut g, &v, &[0xE100_0000, 0x0200_00FF, 0x0000_0000, 0x0004_0010, 0x6000_FF00, 0x0000_0020, 0x0004_0004]);
        for y in 0..4 {
            let drawn = y & 1 != skipped;
            assert_eq!((px(&g, 0, y) != 0, px(&g, 32, y) != 0), (drawn, drawn), "start {start_y}, line {y}");
        }
        send(&mut g, &v, &[0xE100_0400, 0x0200_00FF, 0x0000_0040, 0x0004_0010]);
        assert!((0..4).all(|y| px(&g, 64, y) != 0), "drawing to the display allowed");
    }
}

/// amidog's FIFO full test (Diamond_Disputes.md D-23): a fill written while a VRAM-to-CPU transfer still has words to
/// give waits in the FIFO, so the transfer's words are VRAM as it was, and the fill is drawn once the last is read.
#[test]
fn a_command_behind_an_unread_transfer_waits_until_it_is_read() {
    let (mut g, v) = gpu();
    send(&mut g, &v, &[0xA000_0000, 0, 0x0001_0004, 0x0202_0101, 0x0404_0303, 0xC000_0000, 0, 0x0001_0004]);
    let t = g.busy_until + 1000;
    g.gp0(0x02FF_FFFF, t, &v);
    g.gp0(0, t, &v);
    g.gp0(0x0001_0010, t, &v);
    g.run(t + 100_000, &v);
    assert_eq!((px(&g, 0, 0), g.len), (0x0101, 3), "the fill waits");
    assert_eq!(g.gpuread(&v, t + 100_000).0, 0x0202_0101);
    assert_eq!(g.gpuread(&v, t + 100_001).0, 0x0404_0303);
    g.run(t + 200_000, &v);
    assert_eq!((px(&g, 0, 0), g.len), (0x7FFF, 0), "drawn once the transfer is read");
}

/// amidog's FIFO read test (Diamond_Disputes.md D-23): once a VRAM-to-CPU transfer of two words or more has given its
/// last, GPUREAD repeats its first word; after a transfer of one word it repeats what it gave before that transfer; a
/// GP1(10h) register read takes over until the next transfer ends.
#[test]
fn gpuread_repeats_a_transfers_first_word_once_it_is_read() {
    let (mut g, mut v) = gpu();
    send(&mut g, &v, &[0xA000_0000, 0, 0x0001_0006, 0x0202_0101, 0x0404_0303, 0x0606_0505]);
    let mut t = g.busy_until + 1000;
    let mut read = |g: &mut Gpu, v: &Video, words: &[u32]| {
        send(g, v, words);
        t = t.max(g.busy_until) + 1000;
        (0..5).map(|_| g.gpuread(v, t).0).collect::<Vec<_>>()
    };
    assert_eq!(read(&mut g, &v, &[0xC000_0000, 0, 0x0001_0006]), [0x0202_0101, 0x0404_0303, 0x0606_0505, 0x0202_0101, 0x0202_0101]);
    assert_eq!(read(&mut g, &v, &[0xC000_0000, 1, 0x0001_0001]), [0x0303_0202, 0x0202_0101, 0x0202_0101, 0x0202_0101, 0x0202_0101]);
    g.gp1(0x1000_0007, &mut v, t);
    assert_eq!(g.gpuread(&v, t).0, 2);
}

/// An 8-bit page's palette at (0, 500), entry i reading 8000h | i, and its texels at page 1 all `byte`.
fn eight_bit_page(g: &mut Gpu, byte: u16) {
    for i in 0..256 {
        g.vram.set(i, 500, 0x8000 | i as u16);
    }
    for v in 0..64 {
        for x in 64..128 {
            g.vram.set(x, v, byte * 0x101);
        }
    }
}

/// An 8-bit rectangle of 64 by 1 from texel (0, v) at line `y`, and its first and last pixels.
fn eight_bit_row(g: &mut Gpu, v: &Video, line: u32, tv: u32) -> (u16, u16) {
    send(g, v, &[0x6500_0000, line << 16, 0x7D00_0000 | tv << 8, 0x0001_0040]);
    (px(g, 0, line), px(g, 63, line))
}

/// amidog's CACHE BLOCK (Diamond_Disputes.md D-23): an 8-bit page is cached in blocks of 64 texels by 32 lines, as a
/// 15-bit one is in VRAM's terms, where psx-spx gives 32 by 64; so a row of 64 texels stays whole, and line 32 takes
/// line 0's entries.
#[test]
fn an_8_bit_page_is_cached_64_texels_by_32_lines() {
    let (mut g, v) = gpu();
    eight_bit_page(&mut g, 1);
    send(&mut g, &v, &[0xE100_0081]);
    assert_eq!(eight_bit_row(&mut g, &v, 10, 0), (0x8001, 0x8001));
    eight_bit_page(&mut g, 2);
    assert_eq!(eight_bit_row(&mut g, &v, 11, 0), (0x8001, 0x8001), "the whole row from the cache");
    assert_eq!(eight_bit_row(&mut g, &v, 12, 32), (0x8002, 0x8002), "line 32 fetched");
    assert_eq!(eight_bit_row(&mut g, &v, 13, 0), (0x8002, 0x8002), "line 0 fetched again, line 32 having taken its entries");
}

/// amidog's CACHE TAG (Diamond_Disputes.md D-23): a texture page set at another X or Y, or between 4 bits and the other
/// depths, clears the texture cache; between 8 and 15 bits it does not.
#[test]
fn a_page_change_clears_the_texture_cache() {
    let (mut g, v) = gpu();
    eight_bit_page(&mut g, 1);
    send(&mut g, &v, &[0xE100_0081]);
    eight_bit_row(&mut g, &v, 10, 0);
    eight_bit_page(&mut g, 2);
    send(&mut g, &v, &[0xE100_0101, 0xE100_0081]);
    assert_eq!(eight_bit_row(&mut g, &v, 11, 0).0, 0x8001, "15 bits and back: kept");
    send(&mut g, &v, &[0xE100_0001, 0xE100_0081]);
    assert_eq!(eight_bit_row(&mut g, &v, 12, 0).0, 0x8002, "4 bits and back: cleared");
    eight_bit_page(&mut g, 3);
    send(&mut g, &v, &[0xE100_0082, 0xE100_0081]);
    assert_eq!(eight_bit_row(&mut g, &v, 13, 0).0, 0x8003, "another page and back: cleared");
}

/// GP1(00h) does what GP1(01h) does (psx-spx "GP1(00h)"), the CLUT cache's clear among it, as amidog's CACHE TAG
/// expects of its RESET case (Diamond_Disputes.md D-23).
#[test]
fn a_reset_clears_the_clut_cache() {
    let (mut g, mut v) = gpu();
    g.vram.set(0, 1, 0x0001);
    g.vram.set(1, 20, 0x1234);
    let draw = |g: &mut Gpu, v: &Video, y: u32| send(g, v, &[0xE100_0000, 0xE300_0000, 0xE407_FFFF, 0x6580_8080, y << 16, (20 << 6) << 16 | 0x0100, 0x0001_0001]);
    draw(&mut g, &v, 30);
    g.vram.set(1, 20, 0x7FFF);
    g.gp1(0, &mut v, g.busy_until);
    draw(&mut g, &v, 31);
    assert_eq!((px(&g, 0, 30), px(&g, 0, 31)), (0x1234, 0x7FFF));
}
