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
    let f = &m.genesis.hw.vdp.frame;
    let (wd, ht) = (f.width, f.height);
    (m, wd, ht)
}

fn px(m: &Machine, x: usize, y: usize) -> [u8; 3] {
    let p = &m.genesis.hw.vdp.frame.rgba[(y * MAX_W + x) * 4..][..3];
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
        assert_eq!(m.genesis.hw.vdp.frame.height, 448);
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
    assert_eq!((ht, m.genesis.hw.vdp.frame.height, lines), (240, 240, 512.0));
}

fn solid_tiles() -> Vec<u16> {
    (0..16u16).flat_map(|i| std::iter::repeat_n((i | i << 4) * 0x101, 16)).collect()
}

/// `mdboard.py`'s `dma_into`: transfers from the 68000's RAM into `target` (a command's two words) started by every line
/// interrupt with register 15 at `step`, the values copied to RAM first; with `flip` the frame interrupt inverts them.
fn dma_into(target: u32, values: &[u16], regs: &[u16], blocks: &[(u32, Vec<u16>)], flip: bool, step: u16) -> Vec<u8> {
    let n = values.len() as u16;
    let tail = [0x45F9, 0x0000, 0x4000, 0x47F9, 0x00FF, 0x0000, 0x303C, n - 1, 0x36DA, 0x51C8, 0xFFFC, 0x46FC, 0x2000, 0x60FE];
    let mut r = program(regs, blocks, &tail);
    for (k, v) in values.iter().enumerate() {
        r[0x4000 + 2 * k..0x4002 + 2 * k].copy_from_slice(&v.to_be_bytes());
    }
    let cmd = target | 0x80;
    let hint = w(&[0x32BC, 0x8F00 | step, 0x32BC, 0x9300 | (n & 0xFF), 0x32BC, 0x9400 | n >> 8, 0x32BC, 0x9500, 0x32BC, 0x9680, 0x32BC, 0x977F, 0x22BC, (cmd >> 16) as u16, cmd as u16, 0x4E73]);
    let mut vint = Vec::new();
    if flip {
        vint.extend(w(&[0x49F9, 0x00FF, 0x0000, 0x3E3C, n - 1, 0x0A5C, 0xFFFF, 0x51CF, 0xFFFA]));
    }
    vint.extend(w(&[0x4E73]));
    r[0x1000..0x1000 + hint.len()].copy_from_slice(&hint);
    r[0x1100..0x1100 + vint.len()].copy_from_slice(&vint);
    r[0x70..0x74].copy_from_slice(&0x1000u32.to_be_bytes());
    r[0x78..0x7C].copy_from_slice(&0x1100u32.to_be_bytes());
    r
}

/// Each row's FNV-1a hash of red, green and blue, as `board_rows.rs` holds the board's.
fn row_hashes(m: &Machine) -> Vec<u32> {
    let f = &m.genesis.hw.vdp.frame;
    (0..f.height)
        .map(|y| {
            let mut h = 0x811c9dc5u32;
            for x in 0..f.width {
                for &c in &f.rgba[(y * MAX_W + x) * 4..][..3] {
                    h = (h ^ c as u32).wrapping_mul(0x01000193);
                }
            }
            h
        })
        .collect()
}

/// Transfers through the shown lines into CRAM (40 entries, and one entry alone), a pattern's rows and VSRAM, whose
/// writes land slot by slot whatever the 68000's timing: with the write path's start and the transfer's first read of
/// D-11, the CRAM and VSRAM pictures are the board's on every row, and the pattern picture on all but four, each two
/// pixels wide (Nephrite_Disputes.md D-9; `mdboard.py picture cram-dma`, `cram-dma-one`, `vsram-dma`, `pattern-dma`).
#[test]
fn mid_line_transfers_draw_as_the_board_does() {
    use crate::board_rows::*;
    let regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A63, 0x8C81, 0x8D3F, 0x8F02, 0x9001];
    let column_names: Vec<u16> = (0..32 * 64).map(|i| ((i % 64) % 16 | ((i % 64) / 16 % 4) << 13) as u16).collect();
    let cram_values: Vec<u16> = (0..1024u16).map(|k| (k.wrapping_mul(0x2B5).wrapping_add((k >> 3).wrapping_mul(0x13))) & 0xEEE).collect();
    let cram_dma = dma_into(0xC000_0000, &cram_values, &regs, &[(cram(0), colours(64)), (vram(0), solid_tiles()), (vram(0xC000), column_names)], false, 2);
    let one_values: Vec<u16> = cram_values.iter().enumerate().map(|(k, _)| ((k as u16).wrapping_mul(0x2B5).wrapping_add((k as u16 >> 3).wrapping_mul(0x13)).wrapping_add(0x222)) & 0xEEE).collect();
    let cram_one = dma_into(0xC002_0000, &one_values, &regs, &[(cram(0), colours(16)), (vram(0x20), vec![0x1111; 16]), (vram(0xC000), vec![1; 2048])], false, 0);
    let tile_names: Vec<u16> = (0..32 * 64).map(|i| (i % 64 % 15 + 1) as u16).collect();
    let pattern_values: Vec<u16> = (0..240u32).map(|k| ((k * 0x3A7) ^ (k >> 2) * 0x1111) as u16).collect();
    let pattern_dma = dma_into(vram(0x20), &pattern_values, &regs, &[(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), tile_names)], true, 2);
    let band_names: Vec<u16> = (0..32 * 64).map(|i| (i / 64 % 15 + 1) as u16).collect();
    let vsram_values: Vec<u16> = (0..1024u16).map(|k| (k * 5 + (k >> 4)) & 0x3FF).collect();
    let mut column_regs = regs.to_vec();
    column_regs.insert(7, 0x8B04);
    let vsram_dma = dma_into(0x4000_0010, &vsram_values, &column_regs, &[(cram(0), colours(16)), (vram(0), solid_tiles()), (vram(0xC000), band_names)], false, 2);
    for (name, image, board, least) in [
        ("cram-dma", &cram_dma, &CRAM_DMA, 224),
        ("cram-dma-one", &cram_one, &CRAM_DMA_ONE, 224),
        ("pattern-dma", &pattern_dma, &PATTERN_DMA, 220),
        ("vsram-dma", &vsram_dma, &VSRAM_DMA, 224),
    ] {
        let mut m = Machine::new(image, Media::read(image));
        while m.frames < 3 {
            m.advance();
        }
        let ours = row_hashes(&m);
        let equal = ours.iter().zip(board.iter()).filter(|(a, b)| a == b).count();
        assert!(equal >= least, "{name}: {equal} rows of 224 are the board's");
    }
}

/// The 68000 reading the HV counter and writing a register in a loop, each read stored to RAM: for each colour change
/// on the lines of frame 3, its pixel less the pixel of the HV read that preceded the write, counted.
fn register_offsets(first: u16, second: u16, regs: &[u16], blocks: &[(u32, Vec<u16>)]) -> std::collections::BTreeMap<(u8, i32), u32> {
    let tail = [0x303C, first, 0x323C, second, 0x47F9, 0x00FF, 0x0000, 0x3639, 0x00C0, 0x0008, 0x3280, 0x36C3, 0x3639, 0x00C0, 0x0008, 0x3281, 0x36C3, 0x60EA];
    let image = program(regs, blocks, &tail);
    let (m, wd, _) = run(&image, 3);
    let ram = &m.genesis.hw.wram;
    let records: Vec<u16> = (0..0x8000).map(|i| u16::from_be_bytes([ram[2 * i], ram[2 * i + 1]])).take_while(|&r| r != 0).collect();
    // The records split into frames where the V counter comes back past its jump; the last two are frames 2 and 3.
    let mut starts = vec![0];
    for i in 1..records.len() {
        if records[i] >> 8 < 0x10 && records[i - 1] >> 8 >= 0xE5 {
            starts.push(i);
        }
    }
    let from = starts[starts.len() - 2];
    let mut counts = std::collections::BTreeMap::new();
    for y in 20..200 {
        for x in 1..wd {
            let (c, before) = (px(&m, x, y), px(&m, x - 1, y));
            if c == before {
                continue;
            }
            let colour = if c[0] > c[1] { 1 } else { 2 };
            let read = (from..records.len())
                .filter(|&i| (records[i] >> 8) as usize == y && (if i % 2 == 0 { 1 } else { 2 }) == colour)
                .map(|i| 2 * (records[i] & 0xFF) as i32 - 0x18)
                .filter(|&r| r <= x as i32)
                .max();
            if let Some(r) = read {
                *counts.entry((colour, x as i32 - r)).or_insert(0) += 1;
            }
        }
    }
    counts
}

/// A register write shows two and a half pixels after the 68000 makes it: the backdrop's change falls 9 to 11 pixels
/// after the HV read before it, the board's 9 to 11 (410, 828 and 432 of them); the display's blanking 21 to 23 after
/// (the board's 21 to 23); D-9, `mdboard.py`'s register programs.
#[test]
fn register_writes_show_where_the_board_shows_them() {
    let backdrop = register_offsets(0x8701, 0x8702, &MODE5, &[(cram(0), [0x000, 0x00E, 0x0E0, 0xE00].into_iter().chain(std::iter::repeat_n(0, 60)).collect())]);
    let total: u32 = backdrop.values().sum();
    let near: u32 = backdrop.iter().filter(|((_, d), _)| (9..=11).contains(d)).map(|(_, n)| n).sum();
    assert!(near * 10 >= total * 9, "the backdrop's offsets {backdrop:?}");
    let regs = [0x8004, 0x8174, 0x8230, 0x8407, 0x8578, 0x8702, 0x8C81, 0x8D3F, 0x8F02, 0x9001];
    let blocks = [(cram(0), [0x000, 0x00E, 0x0E0, 0xE00].into_iter().chain(std::iter::repeat_n(0, 60)).collect()), (vram(0x20), vec![0x1111; 16]), (vram(0xC000), vec![1; 2048])];
    let display = register_offsets(0x8174, 0x8134, &regs, &blocks);
    let off: Vec<_> = display.iter().filter(|((c, _), _)| *c == 2).collect();
    let total: u32 = off.iter().map(|(_, n)| **n).sum();
    let near: u32 = off.iter().filter(|((_, d), _)| (21..=23).contains(d)).map(|(_, n)| **n).sum();
    assert!(near * 10 >= total * 9, "the blanking's offsets {off:?}");
}

/// The first change along line 150 of frame 3: the first write's pixel.
fn first_write(image: &[u8]) -> Option<usize> {
    let (m, wd, _) = run(image, 3);
    (140..175).find_map(|y| (0..wd).find(|&x| px(&m, x, y) != if x == 0 { px(&m, wd - 1, y - 1) } else { px(&m, x - 1, y) }))
}

/// `mdboard.py`'s `write_landing`: the line interrupt after line 150 sets CRAM's address to entry 1 (or, the display
/// off, entry 0, the backdrop), waits `nops` NOPs and writes one word, a new colour each frame.
fn write_landing(nops: usize, display: bool, h40: bool) -> Vec<u8> {
    let regs = [0x8014, if display { 0x8174 } else { 0x8134 }, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A96, if h40 { 0x8C81 } else { 0x8C00 }, 0x8D3F, 0x8F02, 0x9001];
    let mut r = program(&regs, &[(cram(0), colours(16)), (vram(0x20), vec![0x1111; 16]), (vram(0xC000), vec![1; 2048])], &[0x46FC, 0x2000, 0x60FE]);
    let entry = if display { cram(2) } else { cram(0) };
    let mut hint = vec![0x32BC, 0x8F00, 0x22BC, (entry >> 16) as u16, entry as u16];
    hint.extend(std::iter::repeat_n(0x4E71, nops));
    hint.extend([0x0645, 0x0246, 0x3085, 0x5445, 0x4E73]);
    let hint = w(&hint);
    r[0x1000..0x1000 + hint.len()].copy_from_slice(&hint);
    r[0x1100..0x1102].copy_from_slice(&w(&[0x4E73]));
    r[0x70..0x74].copy_from_slice(&0x1000u32.to_be_bytes());
    r[0x78..0x7C].copy_from_slice(&0x1100u32.to_be_bytes());
    r
}

/// `mdboard.py`'s `dma_start`: 64 words from `source` into CRAM entry 1 from the line interrupt after line 150, the
/// command after `nops` NOPs.
fn dma_start(nops: usize, source: u32) -> Vec<u8> {
    let regs = [0x8014, 0x8174, 0x8230, 0x8407, 0x8578, 0x8700, 0x8A96, 0x8C81, 0x8D3F, 0x8F02, 0x9001];
    let values: Vec<u16> = (0..64u16).map(|j| (j * 0x2B5 + (j >> 3) * 0x13 + 0x222) & 0xEEE).collect();
    let mut r = dma_into(0xC002_0000, &values, &regs, &[(cram(0), colours(16)), (vram(0x20), vec![0x1111; 16]), (vram(0xC000), vec![1; 2048])], false, 0);
    let mut hint = vec![0x32BC, 0x8F00, 0x32BC, 0x9340, 0x32BC, 0x9400, 0x32BC, 0x9500 | (source >> 1 & 0xFF) as u16, 0x32BC, 0x9600 | (source >> 9 & 0xFF) as u16, 0x32BC, 0x9700 | (source >> 17 & 0x7F) as u16];
    hint.extend(std::iter::repeat_n(0x4E71, nops));
    hint.extend([0x22BC, 0xC002, 0x0080, 0x4E73]);
    let hint = w(&hint);
    r[0x1000..0x1100].fill(0xFF);
    r[0x1000..0x1000 + hint.len()].copy_from_slice(&hint);
    r
}

/// Where a write through the data port, and a transfer's first word, land after the line interrupt, as the board's
/// sweeps show them: the write path's start of 176 master clocks for a word arriving at an empty FIFO, and a
/// transfer's first read 88 after its command (D-11; `mdboard.py picture write-landing-*`, `dma-start-*`).
#[test]
fn writes_land_where_the_board_lands_them() {
    // The board's first changed pixel (or the two it alternates between) for 10, 12, ... 24 NOPs.
    let board: [(bool, bool, [&[usize]; 8]); 4] = [
        (true, true, [&[13, 45], &[45], &[45], &[45], &[45], &[61], &[61], &[77]]),
        (true, false, [&[13], &[13, 45], &[45], &[45], &[45], &[45], &[45], &[61]]),
        (false, true, [&[11, 15], &[19, 23], &[25, 31], &[33, 37], &[39, 43], &[47, 51], &[53, 57], &[61, 65]]),
        (false, false, [&[7, 9], &[13, 17], &[19, 21], &[23, 27], &[31, 35], &[37, 39], &[41, 45], &[47, 49]]),
    ];
    for (display, h40, xs) in board {
        for (k, xs) in xs.iter().enumerate() {
            let x = first_write(&write_landing(10 + 2 * k, display, h40)).unwrap();
            // The display off, a residual of up to 4 pixels is open (D-11).
            let near = |b: &usize| if display { x == *b } else { x.abs_diff(*b) <= 4 };
            assert!(xs.iter().any(near), "{} NOPs, display {display}, H40 {h40}: {x} against the board's {xs:?}", 10 + 2 * k);
        }
    }
    for (nops, board) in [(0, 45), (4, 45), (6, 61), (8, 61), (10, 77)] {
        for source in [0xFF0000, 0x4000] {
            assert_eq!(first_write(&dma_start(nops, source)), Some(board), "{nops} NOPs from {source:06X}");
        }
    }
}
