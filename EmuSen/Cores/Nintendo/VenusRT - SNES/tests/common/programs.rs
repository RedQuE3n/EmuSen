//! VenusRT's synthetic programs for its goldens: minimal LoROM boards, the display, its effects, the sound processor,
//! each coprocessor board and random code, byte for byte as the traces' ROM hashes record them (VenusRT_Native.md §68.2).

use std::collections::HashMap;

use emusen_goldens::DotNetRandom;

/// Bytes of 65816, SPC700 or GSU code at an address, with labels whose branches and addresses are filled in at the end.
pub struct Code {
    origin: usize,
    bytes: Vec<u8>,
    labels: HashMap<&'static str, usize>,
    fixups: Vec<(usize, &'static str, bool)>,
}

impl Code {
    pub fn new(origin: usize) -> Code {
        Code { origin, bytes: Vec::new(), labels: HashMap::new(), fixups: Vec::new() }
    }

    pub fn b(mut self, bytes: &[u8]) -> Code {
        self.bytes.extend_from_slice(bytes);
        self
    }

    pub fn label(mut self, name: &'static str) -> Code {
        let here = self.origin + self.bytes.len();
        assert!(self.labels.insert(name, here).is_none(), "label {name} twice");
        self
    }

    /// An opcode whose last operand byte is a relative branch to the label.
    pub fn branch(mut self, op: u8, label: &'static str, before: &[u8]) -> Code {
        self.bytes.push(op);
        self.bytes.extend_from_slice(before);
        self.fixups.push((self.bytes.len(), label, true));
        self.bytes.push(0);
        self
    }

    /// An opcode followed by the label's 16-bit address.
    pub fn address(mut self, op: u8, label: &'static str) -> Code {
        self.bytes.push(op);
        self.fixups.push((self.bytes.len(), label, false));
        self.bytes.extend_from_slice(&[0, 0]);
        self
    }

    pub fn at(&self, label: &str) -> usize {
        self.labels[label]
    }

    pub fn store(self, register: u16, value: u8) -> Code {
        self.b(&[0xA9, value, 0x8D, register as u8, (register >> 8) as u8])
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut out = self.bytes.clone();
        for &(at, label, relative) in &self.fixups {
            let target = self.labels[label];
            if relative {
                let delta = target as i64 - (self.origin + at + 1) as i64;
                assert!((-128..=127).contains(&delta), "branch to {label} is {delta} bytes away");
                out[at] = delta as i8 as u8;
            } else {
                out[at] = target as u8;
                out[at + 1] = (target >> 8) as u8;
            }
        }
        out
    }
}

// SyntheticRom: a NOP-filled 32 KiB LoROM, reset at $8000, auto-joypad on.
const MAP_MODE: usize = 0x7FD5;
const CART_TYPE: usize = 0x7FD6;
const SRAM_SIZE: usize = 0x7FD8;
const CHIP_TYPE: usize = 0x7FBF;
const CART_NAME: usize = 0x7FC0;

pub fn board(patches: &[(usize, &[u8])]) -> Vec<u8> {
    let mut rom = vec![0xEAu8; 0x8000];
    rom[..5].copy_from_slice(&[0xA9, 0x01, 0x8D, 0x00, 0x42]);
    rom[0x7FFC] = 0x00;
    rom[0x7FFD] = 0x80;
    for (at, bytes) in patches {
        rom[*at..*at + bytes.len()].copy_from_slice(bytes);
    }
    rom
}

pub fn super_fx_board() -> Vec<u8> {
    board(&[(MAP_MODE, &[0x20]), (MAP_MODE + 1, &[0x15])])
}

pub fn sa1_board() -> Vec<u8> {
    board(&[(MAP_MODE, &[0x23]), (CART_TYPE, &[0x35]), (SRAM_SIZE, &[0x05]), (0x7FD6, &[0x35])])
}

pub fn nec_dsp_board(name: &str) -> Vec<u8> {
    let mut title = [b' '; 0x15];
    let latin1: Vec<u8> = name.chars().map(|c| c as u32 as u8).collect();
    let n = latin1.len().min(title.len());
    title[..n].copy_from_slice(&latin1[..n]);
    board(&[(MAP_MODE, &[0x20]), (CART_TYPE, &[0x03]), (CHIP_TYPE, &[0x00]), (CART_NAME, &title)])
}

pub fn obc1_board() -> Vec<u8> {
    board(&[(MAP_MODE, &[0x20]), (CART_TYPE, &[0x25]), (SRAM_SIZE, &[0x03])])
}

const CODE: usize = 0x8000;
const TABLES: usize = 0xC000;
const NATIVE_NMI: usize = 0x7FEA;
const EMULATION_NMI: usize = 0x7FFA;
const SCROLL: u8 = 0x10;
const FRAME: u8 = 0x12;
const RESULTS: u8 = 0x20;

/// Native mode, 16-bit index, 8-bit accumulator, the stack at $1FFF, the screen blanked and interrupts off.
fn start() -> Code {
    Code::new(CODE).b(&[0x78, 0x18, 0xFB, 0xC2, 0x10, 0xE2, 0x20, 0xA2, 0xFF, 0x1F, 0x9A]).b(&[0xA9, 0x8F, 0x8D, 0x00, 0x21, 0x9C, 0x00, 0x42])
}

/// DMA channel 0 from bank 0 to a B-bus register.
fn dma(c: Code, mode: u8, bbus: u8, source: usize, length: usize) -> Code {
    c.store(0x4300, mode)
        .store(0x4301, bbus)
        .b(&[0xA2, source as u8, (source >> 8) as u8, 0x8E, 0x02, 0x43, 0x9C, 0x04, 0x43])
        .b(&[0xA2, length as u8, (length >> 8) as u8, 0x8E, 0x05, 0x43])
        .store(0x420B, 0x01)
}

/// Mode 1, BG1's sixteen 4bpp tiles at VRAM $0000 and its 32x32 map at $0400, a 16-colour palette, BG1 on the main screen.
fn screen(c: Code) -> Code {
    let c = dma(c.b(&[0x9C, 0x21, 0x21]), 0x00, 0x22, TABLES, 32);
    let c = dma(c.store(0x2115, 0x80).b(&[0x9C, 0x16, 0x21, 0x9C, 0x17, 0x21]), 0x01, 0x18, TABLES + 0x20, 512);
    let c = dma(c.store(0x2116, 0x00).store(0x2117, 0x04), 0x01, 0x18, TABLES + 0x220, 2048);
    c.store(0x2105, 0x01).store(0x2107, 0x04).store(0x210B, 0x00).store(0x212C, 0x01)
}

/// The tables at $00:C000: the palette, the tiles, the map; each byte a function of its place.
fn put_tables(rom: &mut [u8], at: usize) {
    for i in 0..16usize {
        let colour = (i * 2) | (31 - i * 2) << 5 | (i * 7 % 32) << 10;
        rom[at + 2 * i] = colour as u8;
        rom[at + 2 * i + 1] = (colour >> 8) as u8;
    }
    for i in 0..512usize {
        rom[at + 0x20 + i] = (i * 37 + (i >> 5) * 11) as u8;
    }
    for i in 0..1024usize {
        let entry = ((i * 5 + (i >> 5)) & 0x0F) | if i & 0x40 != 0 { 0x4000 } else { 0 };
        rom[at + 0x220 + 2 * i] = entry as u8;
        rom[at + 0x221 + 2 * i] = (entry >> 8) as u8;
    }
}

/// The NMI handler's common part: the pad added to the horizontal scroll, the frame counter the vertical one.
fn scroll_by_pad(c: Code) -> Code {
    c.b(&[0xC2, 0x20, 0xAD, 0x18, 0x42, 0x18, 0x6D, SCROLL, 0x00, 0x8D, SCROLL, 0x00, 0xE2, 0x20])
        .b(&[0x8D, 0x0D, 0x21, 0xEB, 0x8D, 0x0D, 0x21])
        .b(&[0xEE, FRAME, 0x00, 0xAD, FRAME, 0x00, 0x8D, 0x0E, 0x21, 0x9C, 0x0E, 0x21])
}

/// The screen on, NMI and auto-joypad on, and the main loop waiting for interrupts, with the work before each wait.
fn run(c: Code, work: impl FnOnce(Code) -> Code) -> Code {
    work(c.store(0x2100, 0x0F).store(0x4200, 0x81).label("main")).b(&[0xCB]).branch(0x80, "main", &[])
}

/// A LoROM image from the board, with the code at $8000, the tables and the NMI vector set.
fn lo_rom(board: Vec<u8>, code: &Code, nmi: usize) -> Vec<u8> {
    let mut rom = board;
    let bytes = code.bytes();
    rom[..bytes.len()].copy_from_slice(&bytes);
    put_tables(&mut rom, TABLES - CODE);
    for v in [NATIVE_NMI, EMULATION_NMI] {
        rom[v] = nmi as u8;
        rom[v + 1] = (nmi >> 8) as u8;
    }
    rom
}

pub fn display() -> Vec<u8> {
    let c = scroll_by_pad(run(screen(start()), |c| c).label("nmi")).b(&[0x40]);
    lo_rom(board(&[]), &c, c.at("nmi"))
}

pub fn effects() -> Vec<u8> {
    let c = screen(start()).store(0x2130, 0x00).store(0x2131, 0x20).store(0x4310, 0x00).store(0x4311, 0x32).b(&[0xA2, 0x00, 0xD0, 0x8E, 0x12, 0x43, 0x9C, 0x14, 0x43]);
    let c = run(c, |w| {
        w.b(&[0xAD, FRAME, 0x00, 0x8D, 0x02, 0x42, 0xAD, 0x18, 0x42, 0x8D, 0x03, 0x42, 0xEA, 0xEA, 0xEA])
            .b(&[0xAE, 0x16, 0x42, 0x8E, RESULTS, 0x00])
            .b(&[0xA2, 0x34, 0x12, 0x8E, 0x04, 0x42, 0xAD, FRAME, 0x00, 0x09, 0x01, 0x8D, 0x06, 0x42, 0xEA, 0xEA, 0xEA, 0xEA])
            .b(&[0xAE, 0x14, 0x42, 0x8E, RESULTS + 2, 0x00, 0xAE, 0x16, 0x42, 0x8E, RESULTS + 4, 0x00])
    });
    let c = scroll_by_pad(c.label("nmi")).store(0x420C, 0x02).b(&[0x40]);
    let mut rom = lo_rom(board(&[]), &c, c.at("nmi"));
    for i in 0..28usize {
        rom[0x5000 + 2 * i] = 8;
        rom[0x5001 + 2 * i] = ((0x20 << (i % 3)) | (i & 31)) as u8;
    }
    rom[0x5000 + 56] = 0;
    rom
}

/// The SPC700's program at $0200: voice 0 plays a looped square, its pitch from port 0, keyed again while port 1 is not zero.
fn sound_program() -> Vec<u8> {
    let mut s = Code::new(0x0200);
    let dsp = |s: Code, reg: u8, value: u8| s.b(&[0x8F, reg, 0xF2, 0x8F, value, 0xF3]);
    for (r, v) in [(0x6C, 0x20), (0x0C, 0x7F), (0x1C, 0x7F), (0x2C, 0x00), (0x3C, 0x00), (0x4D, 0x00), (0x3D, 0x00), (0x2D, 0x00), (0x5D, 0x03), (0x00, 0x40), (0x01, 0x40), (0x02, 0x00), (0x03, 0x10), (0x04, 0x00), (0x05, 0x8F), (0x06, 0xE0), (0x5C, 0x00), (0x4C, 0x01)] {
        s = dsp(s, r, v);
    }
    let s = s
        .label("loop")
        .b(&[0xE4, 0xF4, 0x28, 0x1F, 0x08, 0x08, 0x8F, 0x03, 0xF2, 0xC4, 0xF3, 0xE4, 0xF5])
        .branch(0xF0, "loop", &[])
        .b(&[0x8F, 0x4C, 0xF2, 0x8F, 0x01, 0xF3])
        .branch(0x2F, "loop", &[]);
    let mut image = vec![0u8; 0x0209];
    let code = s.bytes();
    image[..code.len()].copy_from_slice(&code);
    image[0x100..0x104].copy_from_slice(&[0x00, 0x04, 0x00, 0x04]);
    image[0x200] = 0xC3;
    for i in 1..=4 {
        image[0x200 + i] = 0x77;
        image[0x204 + i] = 0x99;
    }
    image
}

pub fn sound() -> Vec<u8> {
    let program = sound_program();
    let length = program.len();
    let c = start()
        .label("aa").b(&[0xAD, 0x40, 0x21, 0xC9, 0xAA]).branch(0xD0, "aa", &[])
        .label("bb").b(&[0xAD, 0x41, 0x21, 0xC9, 0xBB]).branch(0xD0, "bb", &[])
        .b(&[0xA2, 0x00, 0x02, 0x8E, 0x42, 0x21]).store(0x2141, 0x01).store(0x2140, 0xCC)
        .label("cc").b(&[0xCD, 0x40, 0x21]).branch(0xD0, "cc", &[])
        .b(&[0xA2, 0x00, 0x00])
        .label("byte").b(&[0xBD, 0x00, 0xE0, 0x8D, 0x41, 0x21, 0x8A, 0x8D, 0x40, 0x21])
        .label("echo").b(&[0xCD, 0x40, 0x21]).branch(0xD0, "echo", &[])
        .b(&[0xE8, 0xE0, length as u8, (length >> 8) as u8]).branch(0xD0, "byte", &[])
        .b(&[0xA0, 0x00, 0x02, 0x8C, 0x42, 0x21, 0x9C, 0x41, 0x21, 0x8A, 0x1A, 0x8D, 0x40, 0x21])
        .label("go").b(&[0xCD, 0x40, 0x21]).branch(0xD0, "go", &[]);
    let c = run(screen(c), |c| c);
    let c = scroll_by_pad(c.label("nmi")).b(&[0xAD, 0x18, 0x42, 0x8D, 0x40, 0x21, 0xAD, 0x19, 0x42, 0x8D, 0x41, 0x21, 0x40]);
    let mut rom = lo_rom(board(&[]), &c, c.at("nmi"));
    rom[0x6000..0x6000 + length].copy_from_slice(&program);
    rom
}

pub fn sa1() -> Vec<u8> {
    let c = screen(start()).store(0x2226, 0x80).store(0x2229, 0xFF).b(&[0xA2, 0x00, 0xA0, 0x8E, 0x03, 0x22]).store(0x2200, 0x00);
    let c = run(c, |c| c);
    let c = scroll_by_pad(c.label("nmi")).b(&[0xAD, 0x00, 0x30, 0x8D, 0x0D, 0x21, 0xAD, 0x01, 0x30, 0x8D, 0x0D, 0x21, 0x40]);
    let mut rom = lo_rom(sa1_board(), &c, c.at("nmi"));
    let sa1 = Code::new(0xA000)
        .b(&[0x18, 0xFB, 0xE2, 0x20]).store(0x2227, 0x80).store(0x222A, 0xFF).b(&[0xC2, 0x20])
        .label("count").b(&[0xEE, 0x00, 0x30, 0xAD, 0x00, 0x30, 0x8F, 0x00, 0x00, 0x40]).branch(0x80, "count", &[]);
    let code = sa1.bytes();
    rom[0x2000..0x2000 + code.len()].copy_from_slice(&code);
    rom
}

pub fn super_fx() -> Vec<u8> {
    let wram = Code::new(0x0200)
        .label("again")
        .b(&[0xAD, FRAME, 0x00, 0x8D, 0x00, 0x30, 0x9C, 0x01, 0x30]).store(0x301E, 0x00).store(0x301F, 0x90)
        .label("go").b(&[0xAD, 0x30, 0x30, 0x29, 0x20]).branch(0xD0, "go", &[])
        .label("vb").b(&[0xAD, 0x12, 0x42]).branch(0x10, "vb", &[])
        .label("ve").b(&[0xAD, 0x12, 0x42]).branch(0x30, "ve", &[])
        .b(&[0xEE, FRAME, 0x00]).branch(0x80, "again", &[]);
    let lp = wram.bytes();
    let c = screen(start()).store(0x2100, 0x0F).store(0x4200, 0x01)
        .store(0x3034, 0x00).store(0x3038, 0x00).store(0x303A, 0x18).store(0x3037, 0x80).store(0x3039, 0x00)
        .b(&[0xC2, 0x30, 0xA2, 0x00, 0xB0, 0xA0, 0x00, 0x02, 0xA9, (lp.len() - 1) as u8, 0x00, 0x54, 0x00, 0x00, 0xE2, 0x20])
        .b(&[0x5C, 0x00, 0x02, 0x00]);
    let mut rom = lo_rom(super_fx_board(), &c, 0x8000);
    rom[0x3000..0x3000 + lp.len()].copy_from_slice(&lp);
    let gsu = Code::new(0x9000).b(&[0x4E, 0xFC, 0x00, 0x02]).address(0xFD, "plot").b(&[0xA1, 0x00, 0xA2, 0x10]).label("plot").b(&[0x4C, 0x3C, 0x01, 0x00, 0x01]);
    let code = gsu.bytes();
    rom[0x1000..0x1000 + code.len()].copy_from_slice(&code);
    rom
}

pub fn nec_dsp() -> Vec<u8> {
    let c = run(screen(start()), |c| c);
    let c = scroll_by_pad(c.label("nmi"))
        .b(&[0xA9, 0x00]).address(0x20, "send").b(&[0xAD, FRAME, 0x00]).address(0x20, "send").b(&[0xA9, 0x00]).address(0x20, "send")
        .b(&[0xAD, 0x18, 0x42]).address(0x20, "send").b(&[0xA9, 0x40]).address(0x20, "send")
        .address(0x20, "receive").b(&[0x8D, RESULTS, 0x00]).address(0x20, "receive").b(&[0x8D, RESULTS + 1, 0x00]).b(&[0x40])
        .label("send").b(&[0x48]).label("rqm").b(&[0xAF, 0x00, 0xC0, 0x30]).branch(0x10, "rqm", &[]).b(&[0x68, 0x8F, 0x00, 0x80, 0x30, 0x60])
        .label("receive").label("ready").b(&[0xAF, 0x00, 0xC0, 0x30]).branch(0x10, "ready", &[]).b(&[0xAF, 0x00, 0x80, 0x30, 0x60]);
    lo_rom(nec_dsp_board("PILOTWINGS"), &c, c.at("nmi"))
}

pub fn obc1() -> Vec<u8> {
    let c = run(screen(start()), |c| c);
    let c = scroll_by_pad(c.label("nmi"))
        .b(&[0xAD, FRAME, 0x00, 0x29, 0x7F, 0x8D, 0xF6, 0x7F, 0x8D, 0xF0, 0x7F, 0x49, 0xFF, 0x8D, 0xF1, 0x7F])
        .b(&[0xAD, 0x18, 0x42, 0x8D, 0xF2, 0x7F, 0x8D, 0xF3, 0x7F, 0x8D, 0xF4, 0x7F, 0x40]);
    lo_rom(obc1_board(), &c, c.at("nmi"))
}

/// HiROM, 64 KiB: the display's code at file $8000 under a header at $FFC0 that scores as HiROM.
pub fn hi_rom() -> Vec<u8> {
    let lo = display();
    let mut rom = vec![0u8; 0x10000];
    rom[0x8000..0x10000].copy_from_slice(&lo[..0x8000]);
    rom[0xFFC0..0xFFE0].fill(0);
    rom[0xFFC0..0xFFC0 + 21].copy_from_slice(b"VENUSRT GOLDEN HIROM ");
    rom[0xFFD5] = 0x21;
    rom[0xFFD7] = 0x09;
    rom[0xFFD8] = 0x00;
    rom[0xFFDC..0xFFE0].copy_from_slice(&[0xCB, 0xED, 0x34, 0x12]);
    rom
}

/// The boards a random program runs on, in the catalogue's order.
pub const BOARDS: [&str; 6] = ["lorom", "hirom", "sa1", "gsu", "dsp1", "obc1"];

pub fn board_named(name: &str) -> Vec<u8> {
    match name {
        "lorom" => board(&[]),
        "hirom" => hi_rom(),
        "sa1" => sa1_board(),
        "gsu" => super_fx_board(),
        "dsp1" => nec_dsp_board("PILOTWINGS"),
        "obc1" => obc1_board(),
        _ => panic!("no board {name}"),
    }
}

/// Random bytes as code from $8005 to the header, the board's header and vectors kept; seeds 4-7 make STP and WAI NOPs.
pub fn random(board: &str, seed: i32) -> Vec<u8> {
    let mut rom = board_named(board);
    let (start, end) = if rom.len() == 0x10000 { (0x8005, 0xFFB0) } else { (0x0005, 0x7FB0) };
    let index = BOARDS.iter().position(|b| *b == board).unwrap() as i32;
    DotNetRandom::new(seed * 131 + index * 7).next_bytes(&mut rom[start..end]);
    if seed >= 4 {
        for b in &mut rom[start..end] {
            if *b == 0xDB || *b == 0xCB {
                *b = 0xEA;
            }
        }
    }
    rom
}
