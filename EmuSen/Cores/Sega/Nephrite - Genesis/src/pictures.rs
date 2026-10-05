//! The board bench's picture programs, built here as `mdboard.py` builds them, each test holding Nephrite's picture to
//! what Nuked-MD's board showed for the same program (Nephrite_Disputes.md D-4 to D-8, Nephrite_Native.md §15).

use crate::machine::Machine;
use crate::media::Media;
use crate::render::{vram4_address, MAX_W};

fn w(words: &[u16]) -> Vec<u8> {
    words.iter().flat_map(|v| v.to_be_bytes()).collect()
}

fn vram(a: u32) -> u32 {
    (0x4000 | (a & 0x3FFF)) << 16 | a >> 14
}

fn cram(a: u32) -> u32 {
    (0xC000 | (a & 0x3FFF)) << 16 | a >> 14
}

/// A cartridge whose code sets `regs`, writes each block through the data port after its command, then runs `tail`.
fn program(regs: &[u16], blocks: &[(u32, Vec<u16>)], tail: &[u16]) -> Vec<u8> {
    let mut r = vec![0xFFu8; 0x8000];
    r[0..8].copy_from_slice(&w(&[0x00FF, 0xFE00, 0x0000, 0x0200]));
    r[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    let mut code = w(&[0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000]);
    for &v in regs {
        code.extend(w(&[0x32BC, v]));
    }
    let (mut data, at) = (Vec::new(), 0x2000usize);
    for (cmd, words) in blocks {
        let op = 0x200 + code.len();
        code.extend(w(&[0x22BC, (cmd >> 16) as u16, *cmd as u16]));
        code.extend(w(&[0x45FA, (at + data.len()).wrapping_sub(op + 8) as u16]));
        code.extend(w(&[0x303C, words.len() as u16 - 1, 0x309A, 0x51C8, 0xFFFC]));
        data.extend(w(words));
    }
    code.extend(w(tail));
    r[0x200..0x200 + code.len()].copy_from_slice(&code);
    r[at..at + data.len()].copy_from_slice(&data);
    r
}

const MODE5: [u16; 10] = [0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C81, 0x8D3F, 0x8F02, 0x9001];

fn colours(n: usize) -> Vec<u16> {
    (0..n as u16).map(|k| (k & 7) << 1 | (k >> 3 & 7) << 5 | ((k * 5) >> 2 & 7) << 9).collect()
}

/// The machine after `frames` frames of `image`, and its picture.
fn run(image: &[u8], frames: u32) -> (Machine, usize, usize) {
    let mut m = Machine::new(image, Media::read(image));
    for _ in 0..frames {
        m.advance();
    }
    let f = &m.genesis.hw.frame;
    let (wd, ht) = (f.width, f.height);
    (m, wd, ht)
}

fn px(m: &Machine, x: usize, y: usize) -> [u8; 3] {
    let p = &m.genesis.hw.frame.rgba[(y * MAX_W + x) * 4..][..3];
    [p[0], p[1], p[2]]
}

const LEVEL: [u8; 8] = [0, 52, 87, 116, 144, 172, 206, 255];

/// Register 0's bit 2 clear shows each component's lowest bit at the level of step 1 (the board, D-8).
#[test]
fn the_eight_colour_mode_keeps_each_components_low_bit() {
    for full in [true, false] {
        let names: Vec<u16> = (0..32 * 64).map(|i| (((i / 64) % 4) << 13 | (i % 64) % 16) as u16).collect();
        let tiles: Vec<u16> = (0..16u16).flat_map(|i| std::iter::repeat_n((i | i << 4) * 0x101, 16)).collect();
        let mut regs = MODE5.to_vec();
        regs[0] = if full { 0x8004 } else { 0x8000 };
        let image = program(&regs, &[(cram(0), colours(64)), (vram(0), tiles), (vram(0xC000), names)], &[0x60FE]);
        let (m, wd, _) = run(&image, 2);
        assert_eq!(wd, 320);
        let cram = colours(64);
        for k in 0..64 {
            let (x, y) = ((k % 16) * 8 + 4, (k / 16) * 8 + 4);
            let c = cram[k];
            let level = |v: u16| if full { LEVEL[(v & 7) as usize] } else { LEVEL[(v & 1) as usize] };
            let want = if k % 16 == 0 { px(&m, 0, 0) } else { [level(c >> 1), level(c >> 5), level(c >> 9)] };
            assert_eq!(px(&m, x, y), want, "colour {k}, full {full}");
        }
    }
}

/// The 68000 writing an unseen CRAM entry as fast as it can: the dots' columns on a line, and the three on the last
/// line where the FIFO drains into the first blank line's free slots, are the board's (D-6).
#[test]
fn cram_dots_fall_where_the_board_shows_them() {
    let tail = [0x22BC, (cram(4) >> 16) as u16, cram(4) as u16, 0x3081, 0x5241, 0x60FA];
    let h40 = [0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C81, 0x8D3F, 0x8F00, 0x9001];
    let h32 = [0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8D3F, 0x8C00, 0x8F00, 0x9000];
    let cases: [(&[u16], &[usize], &[usize]); 2] = [
        (&h40, &[13, 45, 61, 77, 109, 125, 141, 173, 189, 205, 237, 253, 269, 299, 301], &[311, 313, 315]),
        (&h32, &[13, 45, 61, 77, 109, 125, 141, 173, 189, 205, 235, 237], &[247, 249, 251]),
    ];
    for (regs, line5, last) in cases {
        let image = program(regs, &[(cram(0), vec![0])], &tail);
        let (m, wd, _) = run(&image, 2);
        let dots = |y: usize| -> Vec<usize> { (0..wd).filter(|&x| px(&m, x, y) != px(&m, 0, y)).collect() };
        assert_eq!(dots(5), line5, "line 5, width {wd}");
        let tail = dots(223);
        assert!(last.iter().all(|x| tail.contains(x)), "line 223, width {wd}: {tail:?}");
    }
}

/// Double resolution: the 68000 sets colour 1 in vertical blanking from the status register's odd-field bit, red for
/// odd and green for even. On the board each field shows its own rows, the odd field (field pin and status bit set)
/// the odd ones (D-8); Nephrite draws both rows each field from the field's state, so every row shows its colour.
#[test]
fn each_field_draws_in_the_colour_its_status_bit_chose() {
    let tail = [0x3011, 0x0240, 0x0010, 0xB240, 0x67F6, 0x3200, 0x22BC, 0xC002, 0x0000, 0x4A40, 0x6706, 0x30BC, 0x000E, 0x60E4, 0x30BC, 0x00E0, 0x60DE];
    let regs = [0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C87, 0x8D3F, 0x8F02, 0x9001];
    let image = program(&regs, &[(vram(0), vec![0x1111; 32])], &tail);
    let mut m = Machine::new(&image, Media::read(&image));
    for f in 0..8 {
        m.advance();
        if f < 2 {
            continue;
        }
        assert_eq!(m.genesis.hw.frame.height, 448);
        // The field just drawn is the one before the flag's change at this frame's vertical blanking.
        let want = if m.genesis.hw.vdp.odd { [0, 255, 0] } else { [255, 0, 0] };
        for y in [0, 101, 446, 447] {
            assert_eq!(px(&m, 100, y), want, "frame {f}, row {y}");
        }
    }
}

/// Double resolution's patterns are 64 bytes, name n at n * 64, its sixteen rows down the cell (the board, D-8).
#[test]
fn double_resolution_patterns_are_sixty_four_bytes() {
    let rows: Vec<u16> = (0..16u16).flat_map(|y| [((y % 15) + 1) * 0x1111; 2]).collect();
    let tiles: Vec<u16> = rows.iter().copied().cycle().take(rows.len() * 4).collect();
    let names: Vec<u16> = (0..32 * 64).map(|i| ((i % 64 + i / 64) % 4) as u16).collect();
    let regs = [0x8004, 0x8144, 0x8230, 0x8407, 0x8578, 0x8700, 0x8C87, 0x8D3F, 0x8F02, 0x9001];
    let image = program(&regs, &[(cram(0), colours(16)), (vram(0), tiles), (vram(0xC000), names)], &[0x60FE]);
    let (m, _, _) = run(&image, 4);
    let cram = colours(16);
    for y in 0..48 {
        let c = cram[(y % 16) % 15 + 1];
        assert_eq!(px(&m, 3, y), [LEVEL[(c >> 1 & 7) as usize], LEVEL[(c >> 5 & 7) as usize], LEVEL[(c >> 9 & 7) as usize]], "row {y}");
    }
}

/// Mode 4: words land where its map of VRAM puts their address, a byte's write fills its word, the address steps by
/// one whatever register 15 holds, and CRAM keeps a Master System colour byte an entry (the board's read-back, D-7).
#[test]
fn mode_4_writes_land_where_the_board_put_them() {
    let mut code = vec![0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000, 0x32BC, 0x8004, 0x32BC, 0x8100];
    for (a, v) in [(0x0001, 0xC0), (0x0200, 0xC9), (0x0100, 0xC8), (0x0030, 0x51)] {
        code.extend([0x32BC, 0x4000 | a, 0x10BC, v]);
    }
    code.extend([0x32BC, 0x4000 | 0x3011, 0x30BC, 0x5678, 0x32BC, 0x8F02, 0x32BC, 0x4000 | 0x1100]);
    code.extend([0x10BC, 0x61, 0x10BC, 0x62, 0x10BC, 0x63, 0x10BC, 0x64]);
    code.extend([0x32BC, 0xC000, 0x10BC, 0x02, 0x10BC, 0x10, 0x60FE]);
    let mut image = program(&[], &[], &[]);
    let at = 0x200;
    let bytes = w(&code);
    image[at..at + bytes.len()].copy_from_slice(&bytes);
    let (m, _, _) = run(&image, 2);
    let v = &m.genesis.hw.vdp.vram;
    let word = |a: usize| u16::from_be_bytes([v[a], v[a + 1]]);
    assert_eq!((word(0x0000), word(0x0002), word(0x0200), word(0x0060)), (0xC0C0, 0xC9C9, 0xC8C8, 0x5151));
    assert_eq!(word(0x3020), 0x7856);
    assert_eq!((word(0x1200), word(0x1204)), (0x6262, 0x6464));
    let c = &m.genesis.hw.vdp.cram;
    assert_eq!((u16::from_be_bytes([c[0], c[1]]) & 0xEEE, u16::from_be_bytes([c[2], c[3]]) & 0xEEE), (0x204, 0x040));
    assert_eq!((vram4_address(0), vram4_address(1), vram4_address(0x200), vram4_address(0x3FFF)), (1, 0, 3, 0x3FFE));
}

/// Mode 4's 64 colours at the board's levels, --BBGGRR (D-7).
#[test]
fn mode_4_shows_the_master_system_colours() {
    const RG: [u8; 4] = [0, 95, 161, 255];
    const B: [u8; 4] = [0, 109, 161, 255];
    for first in [0u8, 32] {
        let mut tiles = Vec::new();
        for c in 0..16u8 {
            for _ in 0..8 {
                tiles.extend((0..4).map(|b| if c >> b & 1 != 0 { 0xFFu8 } else { 0 }));
            }
        }
        let mut names = Vec::new();
        for r in 0..28u16 {
            for col in 0..32u16 {
                let e = (col / 2) % 16 | if r >= 12 { 0x800 } else { 0 };
                names.extend([e as u8, (e >> 8) as u8]);
            }
        }
        let pair = |b: &[u8]| b.chunks(2).map(|p| (p[1] as u16) << 8 | p[0] as u16).collect::<Vec<u16>>();
        let mut code = vec![0x46FC, 0x2700, 0x43F9, 0x00C0, 0x0004, 0x41F9, 0x00C0, 0x0000];
        for v in [0x8004, 0x8100, 0x82FF, 0x85FF, 0x86FB, 0x8700, 0x8800, 0x8900] {
            code.extend([0x32BC, v]);
        }
        let mut words = Vec::new();
        for (base, block) in [(0x4000u16, pair(&tiles)), (0x7800, pair(&names))] {
            for (i, &d) in block.iter().enumerate() {
                words.push((base + 2 * i as u16, d));
            }
        }
        for (a, d) in words {
            code.extend([0x32BC, a, 0x30BC, d]);
        }
        code.extend([0x32BC, 0xC000]);
        for k in 0..32u16 {
            code.extend([0x10BC, first as u16 + k]);
        }
        code.extend([0x32BC, 0x8140, 0x60FE]);
        let bytes = w(&code);
        let mut image = vec![0xFFu8; 0x10000];
        image[0..8].copy_from_slice(&w(&[0x00FF, 0xFE00, 0x0000, 0x0200]));
        image[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
        image[0x200..0x200 + bytes.len()].copy_from_slice(&bytes);
        let (m, wd, ht) = run(&image, 3);
        assert_eq!((wd, ht), (256, 192));
        for k in 0..32usize {
            let b = first as usize + k;
            let (x, y) = ((k % 16) * 16 + 8, if k < 16 { 4 } else { 100 });
            assert_eq!(px(&m, x, y), [RG[b & 3], RG[b >> 2 & 3], B[b >> 4 & 3]], "colour {b}");
        }
    }
}

/// V30 on an NTSC board: 240 lines shown, and the V counter's 512 lines a frame, as the board's display enable shows
/// (D-4).
#[test]
fn ntsc_v30_runs_the_counter_to_512_lines() {
    let names: Vec<u16> = (0..32 * 64).map(|i| ((i / 64) % 15 + 1) as u16).collect();
    let tiles: Vec<u16> = (0..16u16).flat_map(|i| std::iter::repeat_n((i | i << 4) * 0x101, 16)).collect();
    let mut regs = MODE5.to_vec();
    regs[1] = 0x814C;
    let image = program(&regs, &[(cram(0), colours(16)), (vram(0), tiles), (vram(0xC000), names)], &[0x60FE]);
    let (mut m, _, ht) = run(&image, 3);
    let before = m.genesis.hw.clock;
    m.advance();
    let lines = ((m.genesis.hw.clock - before) as f64 / 3420.0).round();
    assert_eq!((ht, m.genesis.hw.frame.height, lines), (240, 240, 512.0));
}
