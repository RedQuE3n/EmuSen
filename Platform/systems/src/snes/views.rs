//! The SNES's debugger views over the spaces its engines publish (EmuSen_CoreAPI.md §31): CGRAM's palettes, VRAM's
//! tiles and tilemap entries, the DMA channels from IO ($4000-$43FF as last written), the B bus's names, the vectors,
//! and what `eval` and a summary derive from the 65816's registers.

use crate::debug::{Memory, find, register};

pub const VRAM: &str = "VRAM";
pub const CGRAM: &str = "CGRAM";
pub const IO: &str = "IO";
pub const IO_SIZE: i32 = 0x400;

fn word(memory: &mut dyn Memory, space: &str, address: i32) -> i32 {
    let b = memory.batch(space, &[address, address + 1]);
    b[0] as i32 | (b[1] as i32) << 8
}

/// CGRAM's 256 words, read low byte then high in order as one batch.
fn cgram_words(memory: &mut dyn Memory, cgram: &str) -> Vec<i32> {
    let addresses: Vec<i32> = (0..512).collect();
    let b = memory.batch(cgram, &addresses);
    (0..256).map(|i| b[2 * i] as i32 | (b[2 * i + 1] as i32) << 8).collect()
}

fn named(memory: &dyn Memory, name: &str) -> Option<String> {
    find(memory, name).map(|s| s.name.clone())
}

/// All 256 entries as sixteen palettes of sixteen, each BGR555 channel scaled to 0-255.
pub fn palettes(memory: &mut dyn Memory) -> Vec<(i32, Vec<(u8, u8, u8)>)> {
    let Some(cgram) = named(memory, CGRAM) else { return Vec::new() };
    let words = cgram_words(memory, &cgram);
    (0..16)
        .map(|p| {
            let colours = (0..16)
                .map(|c| {
                    let raw = words[(p * 16 + c) as usize];
                    (((raw & 0x1F) * 255 / 31) as u8, (((raw >> 5) & 0x1F) * 255 / 31) as u8, (((raw >> 10) & 0x1F) * 255 / 31) as u8)
                })
                .collect();
            (p, colours)
        })
        .collect()
}

/// A 16x16 grid of 12-pixel swatches, each channel shifted left three as the frame's colours are.
pub fn palette_swatch(memory: &mut dyn Memory) -> (Vec<u8>, i32, i32) {
    let Some(cgram) = named(memory, CGRAM) else { return (Vec::new(), 0, 0) };
    const COLS: usize = 16;
    const SWATCH: usize = 12;
    let (width, height) = (COLS * SWATCH, 16 * SWATCH);
    let mut buffer = vec![0u8; width * height * 4];
    let words = cgram_words(memory, &cgram);
    for i in 0..256 {
        let raw = words[i];
        let (r, g, b) = (((raw & 0x1F) << 3) as u8, (((raw >> 5) & 0x1F) << 3) as u8, (((raw >> 10) & 0x1F) << 3) as u8);
        let (x0, y0) = ((i % COLS) * SWATCH, (i / COLS) * SWATCH);
        for y in 0..SWATCH {
            for x in 0..SWATCH {
                let o = ((y0 + y) * width + x0 + x) * 4;
                buffer[o..o + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
    }
    (buffer, width as i32, height as i32)
}

/// VRAM's 2048 tiles as 4bpp, 32 a row, each index n drawn as grey 17n; its 64 KiB read in order, as the C# read them.
pub fn tile_sheet(memory: &mut dyn Memory) -> (Vec<u8>, i32, i32) {
    let Some(vram) = named(memory, VRAM) else { return (Vec::new(), 0, 0) };
    const WIDTH: usize = 256;
    const HEIGHT: usize = 512;
    let bytes = memory.run(&vram, 0x10000);
    let mut buffer = vec![0u8; WIDTH * HEIGHT * 4];
    for tile in 0..2048 {
        let (x0, y0, at) = ((tile % 32) * 8, (tile / 32) * 8, tile * 32);
        for r in 0..8 {
            let (p0, p1, p2, p3) = (bytes[at + r * 2], bytes[at + r * 2 + 1], bytes[at + 16 + r * 2], bytes[at + 16 + r * 2 + 1]);
            for col in 0..8 {
                let bit = 7 - col;
                let pixel = ((p0 >> bit) & 1) | (((p1 >> bit) & 1) << 1) | (((p2 >> bit) & 1) << 2) | (((p3 >> bit) & 1) << 3);
                let v = pixel * 17;
                let o = ((y0 + r) * WIDTH + x0 + col) * 4;
                buffer[o..o + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }
    }
    (buffer, WIDTH as i32, HEIGHT as i32)
}

/// Two bytes: tile 0-9, palette 10-12, priority 13, flips 14-15.
pub const TILEMAP_ENTRY_STRIDE: i32 = 2;

pub fn decode_tilemap_entry(memory: &mut dyn Memory, space: &str, address: i32) -> String {
    let entry = word(memory, space, address);
    format!(
        "{:03X}{}{}{}{}",
        entry & 0x3FF,
        (entry >> 10) & 7,
        if entry & 0x2000 != 0 { 'P' } else { '.' },
        if entry & 0x4000 != 0 { 'H' } else { '.' },
        if entry & 0x8000 != 0 { 'V' } else { '.' }
    )
}

/// Planar, bpp/2 row-interleaved pairs of bit planes 16 bytes apart; another depth refused with the C# words.
pub fn decode_tile_pixels(memory: &mut dyn Memory, space: &str, address: i32, bpp: i32) -> Result<Vec<u8>, (String, Option<&'static str>)> {
    if !matches!(bpp, 2 | 4 | 8) {
        return Err((format!("bpp must be 2, 4, or 8 (got {bpp}) - the SNES has no other planar tile depth."), None));
    }
    let mut pixels = vec![0u8; 64];
    let addresses: Vec<i32> = (0..8).flat_map(|row| (0..bpp / 2).flat_map(move |pair| [address + pair * 16 + row * 2, address + pair * 16 + row * 2 + 1])).collect();
    let read = memory.batch(space, &addresses);
    let mut next = 0;
    for row in 0..8 {
        for pair in 0..bpp / 2 {
            let (lo, hi) = (read[next], read[next + 1]);
            next += 2;
            for col in 0..8 {
                let bit = 7 - col;
                pixels[(row * 8 + col) as usize] |= ((((lo >> bit) & 1) | (((hi >> bit) & 1) << 1)) << (pair * 2)) as u8;
            }
        }
    }
    Ok(pixels)
}

/// One channel, `DebugDmaChannel`'s fields in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    pub index: i32,
    pub general: bool,
    pub hdma: bool,
    pub control: u8,
    pub destination: u8,
    pub source: i32,
    pub size: i32,
    pub hdma_active: bool,
    pub table: i32,
    pub line_counter: u8,
    pub indirect: i32,
}

/// The eight channels from $4300-$437F, MDMAEN and HDMAEN; an HDMA channel is running while enabled and its line
/// counter is not the terminating 0. None unless IO is the 0x400 bytes VenusRT publishes.
pub fn dma_channels(memory: &mut dyn Memory) -> Vec<Channel> {
    let Some(io) = find(memory, IO).filter(|s| s.size == IO_SIZE).map(|s| s.name.clone()) else { return Vec::new() };
    let head = memory.batch(&io, &[0x20B, 0x20C]);
    let (general, hdma) = (head[0], head[1]);
    (0..8)
        .map(|c| {
            let at = 0x300 + c * 16;
            // In the C#'s order: bank and count, then the constructor's arguments left to right, R(10) only for an enabled HDMA.
            let first = memory.batch(&io, &[at + 4, at + 5, at + 6, at + 0, at + 1, at + 2, at + 3]);
            let (r4, r5, r6, control, destination, r2, r3) = (first[0], first[1], first[2], first[3], first[4], first[5], first[6]);
            let bank = (r4 as i32) << 16;
            let count = r5 as i32 | (r6 as i32) << 8;
            let hdma_enabled = hdma & (1 << c) != 0;
            let hdma_active = hdma_enabled && memory.read(&io, at + 10) != 0;
            let last = memory.batch(&io, &[at + 8, at + 9, at + 10, at + 7]);
            let (r8, r9, r10b, r7) = (last[0], last[1], last[2], last[3]);
            Channel {
                index: c,
                general: general & (1 << c) != 0,
                hdma: hdma_enabled,
                control,
                destination,
                source: bank | r2 as i32 | (r3 as i32) << 8,
                size: if count == 0 { 0x10000 } else { count },
                hdma_active,
                table: bank | r8 as i32 | (r9 as i32) << 8,
                line_counter: r10b,
                indirect: (r7 as i32) << 16 | count,
            }
        })
        .collect()
}

const B_BUS: [&str; 64] = [
    "INIDISP", "OBSEL", "OAMADDL", "OAMADDH", "OAMDATA", "BGMODE", "MOSAIC", "BG1SC", "BG2SC", "BG3SC", "BG4SC", "BG12NBA", "BG34NBA", "BG1HOFS",
    "BG1VOFS", "BG2HOFS", "BG2VOFS", "BG3HOFS", "BG3VOFS", "BG4HOFS", "BG4VOFS", "VMAIN", "VMADDL", "VMADDH", "VMDATAL", "VMDATAH", "M7SEL", "M7A",
    "M7B", "M7C", "M7D", "M7X", "M7Y", "CGADD", "CGDATA", "W12SEL", "W34SEL", "WOBJSEL", "WH0", "WH1", "WH2", "WH3", "WBGLOG", "WOBJLOG", "TM", "TS",
    "TMW", "TSW", "CGWSEL", "CGADSUB", "COLDATA", "SETINI", "MPYL", "MPYM", "MPYH", "SLHV", "RDOAM", "RDVRAML", "RDVRAMH", "RDCGRAM", "OPHCT",
    "OPVCT", "STAT77", "STAT78",
];

/// The B bus's $2100-$21FF as fullsnes names them, the APU's four ports mirrored through $217F.
pub fn name_dma_destination(register: u8) -> String {
    let name: Option<String> = match register {
        0x00..=0x3F => Some(B_BUS[register as usize].into()),
        0x40..=0x7F => Some(format!("APUI0{}", register & 3)),
        0x80 => Some("WMDATA".into()),
        0x81 => Some("WMADDL".into()),
        0x82 => Some("WMADDM".into()),
        0x83 => Some("WMADDH".into()),
        _ => None,
    };
    let address = 0x2100 + register as i32;
    match name {
        Some(n) => format!("{n} ${address:04X}"),
        None => format!("${address:04X}"),
    }
}

/// Where each 65816 vector's pointer lives in bank 0, native mode's then emulation mode's.
pub const VECTORS: [(&str, i32, i32, &str); 10] = [
    ("COP", 0x00FFE4, 2, "Native (E=0)"),
    ("BRK", 0x00FFE6, 2, "Native (E=0)"),
    ("ABORT", 0x00FFE8, 2, "Native (E=0)"),
    ("NMI", 0x00FFEA, 2, "Native (E=0)"),
    ("IRQ", 0x00FFEE, 2, "Native (E=0)"),
    ("COP", 0x00FFF4, 2, "Emulation (E=1)"),
    ("ABORT", 0x00FFF8, 2, "Emulation (E=1)"),
    ("NMI", 0x00FFFA, 2, "Emulation (E=1)"),
    ("RESET", 0x00FFFC, 2, "Emulation (E=1)"),
    ("IRQ/BRK", 0x00FFFE, 2, "Emulation (E=1)"),
];

/// P's bits by the names `eval` takes, and the registers' other names.
const FLAGS: [(&str, u32); 8] = [("flag.c", 0), ("flag.z", 1), ("flag.i", 2), ("flag.d", 3), ("flag.x", 4), ("flag.m", 5), ("flag.v", 6), ("flag.n", 7)];
const ALIASES: [(&str, &str); 4] = [("sp", "S"), ("dp", "D"), ("pbr", "PB"), ("dbr", "DB")];

pub fn symbol_names() -> Vec<&'static str> {
    ALIASES.iter().map(|a| a.0).chain(FLAGS.iter().map(|f| f.0)).collect()
}

pub fn symbol(name: &str, registers: &[(String, u64)]) -> Option<i64> {
    use emusen_dotnet::text::equals_ignore_case;
    if let Some(&(_, bit)) = FLAGS.iter().find(|(f, _)| equals_ignore_case(name, f)) {
        return register(registers, "P").map(|p| ((p >> bit) & 1) as i64);
    }
    if let Some(&(_, r)) = ALIASES.iter().find(|(a, _)| equals_ignore_case(name, a)) {
        return register(registers, r).map(|v| v as i64);
    }
    None
}

/// The 65816's state on one line, P's letters upper case when set; in emulation mode bits 5 and 4 read '-' and B.
pub fn summary(registers: &[(String, u64)]) -> Option<String> {
    let r = |n: &str| register(registers, n);
    let (pb, pc, a, x, y, s, d, db, p, e) = (r("PB")?, r("PC")?, r("A")?, r("X")?, r("Y")?, r("S")?, r("D")?, r("DB")?, r("P")?, r("E")?);
    let p = p as u8;
    let emulation = e != 0;
    let bit = |b: u32, letter: char| if p & (1 << b) != 0 { letter.to_ascii_uppercase() } else { letter };
    let middle = if emulation { ['-', bit(4, 'b')] } else { [bit(5, 'm'), bit(4, 'x')] };
    let flags: String = [bit(7, 'n'), bit(6, 'v'), middle[0], middle[1], bit(3, 'd'), bit(2, 'i'), bit(1, 'z'), bit(0, 'c')].iter().collect();
    Some(format!(
        "[STATE] CPU  PC={:02X}:{:04X}  A={:04X} X={:04X} Y={:04X} S={:04X} D={:04X} DB={:02X}  P={p:02X} [{flags}]  E={}",
        pb as u8,
        pc as u16,
        a as u16,
        x as u16,
        y as u16,
        s as u16,
        d as u16,
        db as u8,
        if emulation { 1 } else { 0 }
    ))
}
