//! Every v1 core the build lists (`xtask/libraries.tsv`, role `core`, ABI 1) built from this commit with its sidecar, an
//! installation holding them all, and for a console a test image the test makes itself: a program that draws one colour
//! and another while a button of the pad is held, so the picture shows the pad reaching the core (EmuSen_Qt.md §8.5).
//! No image here is, or is copied from, a commercial game or a firmware image.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::cores::{build, installation, sha256};

/// One v1 core of the build's list: its library's name, the core's name as the list gives it, its library built with
/// its sidecar beside it.
pub struct V1Core {
    pub name: String,
    pub core: String,
    pub library: PathBuf,
}

impl V1Core {
    /// The sidecar the conformance runner wrote beside the library.
    pub fn sidecar(&self) -> PathBuf {
        let name = format!("{}.core.json", self.library.file_name().unwrap().to_string_lossy());
        self.library.with_file_name(name)
    }
}

/// The build's list of native libraries.
fn libraries_tsv() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../xtask/libraries.tsv")
}

/// The v1 cores of the build's list, each built and its sidecar written, once a process.
pub fn v1_cores() -> &'static [V1Core] {
    static BUILT: OnceLock<Vec<V1Core>> = OnceLock::new();
    BUILT.get_or_init(|| {
        let text = std::fs::read_to_string(libraries_tsv()).expect("xtask/libraries.tsv");
        let runner = build("Shared/emusen-core-conform", "emusen_core_conform", &["--bin", "emusen-core-conform"]).join("emusen-core-conform");
        let mut cores = Vec::new();
        for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < 8 || fields[2] != "core" || fields[3] != "1" {
                continue;
            }
            let folder = fields[1].strip_prefix("EmuSen/Cores/").expect("a core under EmuSen/Cores");
            let library = build(folder, fields[0], &["--lib"]).join(format!("lib{}.so", fields[0]));
            let status = std::process::Command::new(&runner).arg("--sidecar").arg(&library).stdout(std::process::Stdio::null()).status().expect("the conformance runner runs");
            assert!(status.success(), "{}'s sidecar was not written: {status}", fields[0]);
            cores.push(V1Core { name: fields[0].to_owned(), core: fields[7].to_owned(), library });
        }
        cores
    })
}

/// An installation folder as `installation` makes, with every v1 core and its sidecar in it, vouched for by its manifest.
pub fn installation_with_cores(dir: &Path) -> PathBuf {
    let dir = installation(dir);
    let manifest = dir.join("cores.manifest");
    let mut text = std::fs::read_to_string(&manifest).unwrap();
    for core in v1_cores() {
        let name = core.library.file_name().unwrap();
        std::fs::copy(&core.library, dir.join(name)).unwrap();
        std::fs::copy(core.sidecar(), dir.join(core.sidecar().file_name().unwrap())).unwrap();
        text += &format!("{}  {}\n", sha256(&core.library), name.to_string_lossy());
    }
    std::fs::write(manifest, text).unwrap();
    dir
}

/// A test image for a system of the systems table: its extension and its bytes; or why none is made.
pub fn image(system: &str) -> Result<(&'static str, Vec<u8>), &'static str> {
    Ok(match system {
        "nes" => (".nes", nes()),
        "snes" => (".sfc", snes()),
        "gb" => (".gb", game_boy(false)),
        "gbc" => (".gbc", game_boy(true)),
        "sms" => (".sms", master_system()),
        "sg1000" => (".sg", sg1000()),
        "md" => (".md", genesis(b"SEGA MEGA DRIVE ")),
        "32x" => (".32x", genesis(b"SEGA 32X        ")),
        "mcd" => (".iso", sega_cd()),
        "psx" => (".psexe", playstation()),
        "n64" => (".z64", nintendo_64()),
        "ps2" => return Err("Sapphire runs nothing without a PlayStation 2 BIOS, whose replacement is none, and the tree holds none (Sapphire's v1.rs, create_empty and the fast boot)"),
        _ => return Err("this fixture makes no image for it"),
    })
}

/// An NROM cartridge (16 KiB of program, 8 KiB of blank tiles): the pad read through $4016 each frame, and the backdrop,
/// palette entry $3F00, blue ($11) with nothing held and white ($30) while A or B is, the background on with every tile
/// blank so the backdrop fills the picture.
fn nes() -> Vec<u8> {
    #[rustfmt::skip]
    let code: [u8; 90] = [
        0x78, 0xD8, 0xA2, 0xFF, 0x9A, 0xA9, 0x00, 0x8D, 0x00, 0x20, 0x8D, 0x01, 0x20, // sei cld ldx txs, PPU off
        0x2C, 0x02, 0x20, 0x10, 0xFB, 0x2C, 0x02, 0x20, 0x10, 0xFB, // two vblanks
        0xA9, 0x01, 0x8D, 0x16, 0x40, 0xA9, 0x00, 0x8D, 0x16, 0x40, // loop: strobe the pad
        0xAD, 0x16, 0x40, 0x29, 0x01, 0x85, 0x00, 0xAD, 0x16, 0x40, 0x29, 0x01, 0x05, 0x00, // A | B
        0xF0, 0x04, 0xA9, 0x30, 0xD0, 0x02, 0xA9, 0x11, 0xAA, // white held, else blue, into X
        0x2C, 0x02, 0x20, 0x10, 0xFB, // the next vblank
        0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0x8E, 0x07, 0x20, // $3F00 = X
        0xA9, 0x00, 0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20, 0xA9, 0x08, 0x8D, 0x01, 0x20, // address away, background on
        0x4C, 0x17, 0xC0, // jmp loop
    ];
    let mut prg = vec![0u8; 0x4000];
    prg[..code.len()].copy_from_slice(&code);
    for v in [0x3FFA, 0x3FFC, 0x3FFE] {
        prg[v..v + 2].copy_from_slice(&0xC000u16.to_le_bytes());
    }
    let mut image = b"NES\x1a\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.extend(prg);
    image.extend(vec![0u8; 0x2000]);
    image
}

/// A 128 KiB LoROM: the auto-joypad read on, and each vblank colour 0 (the backdrop) blue with nothing held and white
/// while B, Y, Select or Start is, the screen on at full brightness; its header as `cores::snes_image` fills it.
fn snes() -> Vec<u8> {
    #[rustfmt::skip]
    let code: [u8; 63] = [
        0x78, 0x18, 0xFB, 0xA9, 0x8F, 0x8D, 0x00, 0x21, 0xA9, 0x01, 0x8D, 0x00, 0x42, // sei, native, forced blank, auto-joypad
        0xAD, 0x12, 0x42, 0x10, 0xFB, // loop: wait for vblank
        0xAD, 0x12, 0x42, 0x29, 0x01, 0xD0, 0xF9, // wait for the auto-joypad read
        0xAD, 0x19, 0x42, 0x29, 0xF0, 0xF0, 0x06, // B Y Select Start
        0xA9, 0xFF, 0xA2, 0x7F, 0x80, 0x04, 0xA9, 0x00, 0xA2, 0x7C, // white held, else blue
        0x9C, 0x21, 0x21, 0x8D, 0x22, 0x21, 0x8E, 0x22, 0x21, // colour 0
        0xA9, 0x0F, 0x8D, 0x00, 0x21, // screen on
        0xAD, 0x12, 0x42, 0x30, 0xFB, 0x80, 0xCE, // vblank over, loop
    ];
    let mut rom = crate::cores::snes_image("PAD COLOURS");
    rom[..code.len()].copy_from_slice(&code);
    rom[0x7FDC..0x7FE0].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    let sum = rom.iter().fold(0u16, |s, &b| s.wrapping_add(u16::from(b)));
    rom[0x7FDC..0x7FDE].copy_from_slice(&(!sum).to_le_bytes());
    rom[0x7FDE..0x7FE0].copy_from_slice(&sum.to_le_bytes());
    rom
}

/// A 32 KiB cartridge with no mapper, its header's checksum right: the action buttons read through $FF00, and the
/// background shade (BGP, and the Color's palette 0 colour 0 where `color` marks it the Color's) white with nothing
/// held and black while any is.
fn game_boy(color: bool) -> Vec<u8> {
    #[rustfmt::skip]
    let code: [u8; 39] = [
        0xF3, 0x31, 0xFE, 0xFF, // di, the stack
        0x3E, 0x10, 0xE0, 0x00, 0xF0, 0x00, 0xF0, 0x00, 0xE6, 0x0F, 0xFE, 0x0F, // loop: the buttons
        0x06, 0x00, 0x28, 0x02, 0x06, 0xFF, // white, black while held
        0x78, 0xE0, 0x47, // BGP
        0x3E, 0x80, 0xE0, 0x68, 0x78, 0x2F, 0xE0, 0x69, 0xE6, 0x7F, 0xE0, 0x69, // the Color's colour 0
        0x18, 0xDD, // jr loop
    ];
    let mut rom = vec![0u8; 0x8000];
    rom[0x100..0x104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]);
    rom[0x134..0x13F].copy_from_slice(b"PADCOLOURS ");
    rom[0x143] = if color { 0x80 } else { 0x00 };
    rom[0x150..0x150 + code.len()].copy_from_slice(&code);
    let sum = rom[0x134..0x14D].iter().fold(0u8, |s, &b| s.wrapping_sub(b).wrapping_sub(1));
    rom[0x14D] = sum;
    rom
}

/// A 32 KiB Master System cartridge with Sega's header (export region, its checksum right, as Endou's test programs
/// write it): port $DC read, and colour 0 blue with nothing held and white while button 1 or 2 is, the display on.
fn master_system() -> Vec<u8> {
    #[rustfmt::skip]
    let code: [u8; 44] = [
        0xF3, 0x31, 0xF0, 0xDF, // di, the stack
        0x3E, 0x04, 0xD3, 0xBF, 0x3E, 0x80, 0xD3, 0xBF, 0x3E, 0x40, 0xD3, 0xBF, 0x3E, 0x81, 0xD3, 0xBF, // mode 4, display on
        0xDB, 0xDC, 0xE6, 0x30, 0xFE, 0x30, 0x06, 0x3F, 0x20, 0x02, 0x06, 0x30, // loop: white held, else blue
        0xAF, 0xD3, 0xBF, 0x3E, 0xC0, 0xD3, 0xBF, 0x78, 0xD3, 0xBE, // colour 0
        0x18, 0xE8, // jr loop
    ];
    let mut rom = vec![0u8; 0x8000];
    rom[..code.len()].copy_from_slice(&code);
    rom[0x7FF0..0x7FF8].copy_from_slice(b"TMR SEGA");
    rom[0x7FFF] = 4 << 4 | 0xC;
    let sum = rom[..0x7FF0].iter().fold(0u16, |s, &b| s.wrapping_add(u16::from(b)));
    rom[0x7FFA..0x7FFC].copy_from_slice(&sum.to_le_bytes());
    rom
}

/// A 16 KiB SG-1000 cartridge: port $DC read, and the backdrop (register 7) dark blue with nothing held and white while
/// button 1 or 2 is, the display on over blank patterns.
fn sg1000() -> Vec<u8> {
    #[rustfmt::skip]
    let code: [u8; 41] = [
        0xF3, 0x31, 0xF0, 0xC3, // di, the stack
        0x3E, 0x00, 0xD3, 0xBF, 0x3E, 0x80, 0xD3, 0xBF, 0x3E, 0xC0, 0xD3, 0xBF, 0x3E, 0x81, 0xD3, 0xBF, // graphics I, display on
        0xDB, 0xDC, 0xE6, 0x30, 0xFE, 0x30, 0x06, 0x0F, 0x20, 0x02, 0x06, 0x04, // loop: white held, else dark blue
        0x78, 0xD3, 0xBF, 0x3E, 0x87, 0xD3, 0xBF, // register 7
        0x18, 0xEB, // jr loop
    ];
    let mut rom = vec![0u8; 0x4000];
    rom[..code.len()].copy_from_slice(&code);
    rom
}

/// The 68000 program the Genesis, 32X and Sega CD images share, position-independent: port 1 read in both of its TH
/// phases, and colour 0 (the backdrop over blank tiles) blue with nothing held and white while A, B, C or Start is.
#[rustfmt::skip]
const M68K: [u16; 56] = [
    0x46FC, 0x2700, // move #$2700,sr
    0x13FC, 0x0040, 0x00A1, 0x0009, // TH an output
    0x33FC, 0x8004, 0x00C0, 0x0004, 0x33FC, 0x8144, 0x00C0, 0x0004, 0x33FC, 0x8700, 0x00C0, 0x0004, // mode 5, display on
    0x13FC, 0x0040, 0x00A1, 0x0003, 0x4E71, 0x4E71, 0x1039, 0x00A1, 0x0003, // loop: TH high, B C
    0x13FC, 0x0000, 0x00A1, 0x0003, 0x4E71, 0x4E71, 0x1239, 0x00A1, 0x0003, // TH low, A Start
    0xC001, 0x0200, 0x0030, 0x0C00, 0x0030, 0x6706, 0x323C, 0x0EEE, 0x6004, 0x323C, 0x0E40, // white held, else blue
    0x23FC, 0xC000, 0x0000, 0x00C0, 0x0004, 0x33C1, 0x00C0, 0x0000, // colour 0
    0x60B4, // bra loop
];

fn m68k_bytes() -> Vec<u8> {
    M68K.iter().flat_map(|w| w.to_be_bytes()).collect()
}

/// A 128 KiB cartridge whose header at $100 names `system` (the Genesis's or the 32X's) and the Americas, its reset
/// vector at the shared program at $200.
fn genesis(system: &[u8; 16]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x20000];
    rom[0..4].copy_from_slice(&0x00FF_FE00u32.to_be_bytes());
    rom[4..8].copy_from_slice(&0x0000_0200u32.to_be_bytes());
    rom[0x100..0x110].copy_from_slice(system);
    rom[0x120..0x12B].copy_from_slice(b"PAD COLOURS");
    rom[0x1F0..0x1F3].copy_from_slice(b"JUE");
    let code = m68k_bytes();
    rom[0x200..0x200 + code.len()].copy_from_slice(&code);
    rom
}

/// A Sega CD disc of 2,048-byte sectors whose system area names the Americas and holds the shared program as its initial
/// program at $200, which the boot ROM copies to work RAM and enters (its open BIOS: `mcd/bios/sub.s`, `load_ip`).
fn sega_cd() -> Vec<u8> {
    let mut d = vec![0u8; 40 * 2048];
    d[..16].copy_from_slice(b"SEGADISCSYSTEM  ");
    d[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    d[0x120..0x12B].copy_from_slice(b"PAD COLOURS");
    d[0x1F0] = b'U';
    let code = m68k_bytes();
    d[0x200..0x200 + code.len()].copy_from_slice(&code);
    d
}

/// MIPS words with labels, for the PlayStation's and the Nintendo 64's programs.
#[derive(Default)]
struct Mips {
    words: Vec<u32>,
    labels: Vec<(&'static str, usize)>,
    branches: Vec<(usize, &'static str)>,
}

const ZERO: u32 = 0;
const T0: u32 = 8;
const T1: u32 = 9;
const T2: u32 = 10;
const T3: u32 = 11;
const T4: u32 = 12;
const T5: u32 = 13;
const S0: u32 = 16;
const T6: u32 = 14;
const T7: u32 = 15;
const S1: u32 = 17;
const S2: u32 = 18;

impl Mips {
    fn w(&mut self, word: u32) -> &mut Mips {
        self.words.push(word);
        self
    }
    fn label(&mut self, name: &'static str) -> &mut Mips {
        self.labels.push((name, self.words.len()));
        self
    }
    fn i(&mut self, op: u32, rs: u32, rt: u32, imm: u32) -> &mut Mips {
        self.w(op << 26 | rs << 21 | rt << 16 | (imm & 0xFFFF))
    }
    fn lui(&mut self, rt: u32, imm: u32) -> &mut Mips {
        self.i(0x0F, 0, rt, imm)
    }
    fn ori(&mut self, rt: u32, rs: u32, imm: u32) -> &mut Mips {
        self.i(0x0D, rs, rt, imm)
    }
    fn addiu(&mut self, rt: u32, rs: u32, imm: i32) -> &mut Mips {
        self.i(0x09, rs, rt, imm as u32)
    }
    fn sw(&mut self, rt: u32, offset: i32, rs: u32) -> &mut Mips {
        self.i(0x2B, rs, rt, offset as u32)
    }
    fn sh(&mut self, rt: u32, offset: i32, rs: u32) -> &mut Mips {
        self.i(0x29, rs, rt, offset as u32)
    }
    fn sb(&mut self, rt: u32, offset: i32, rs: u32) -> &mut Mips {
        self.i(0x28, rs, rt, offset as u32)
    }
    fn lhu(&mut self, rt: u32, offset: i32, rs: u32) -> &mut Mips {
        self.i(0x25, rs, rt, offset as u32)
    }
    fn srl(&mut self, rd: u32, rt: u32, sa: u32) -> &mut Mips {
        self.w(rt << 16 | rd << 11 | sa << 6 | 0x02)
    }
    /// A spin of `n` turns in T1, under a label of its own.
    fn delay(&mut self, n: u32) -> &mut Mips {
        let spin: &'static str = Box::leak(format!("spin{}", self.words.len()).into_boxed_str());
        self.lui(T1, n >> 16).ori(T1, T1, n & 0xFFFF).label(spin).addiu(T1, T1, -1).bne(T1, ZERO, spin).nop()
    }
    fn lw(&mut self, rt: u32, offset: i32, rs: u32) -> &mut Mips {
        self.i(0x23, rs, rt, offset as u32)
    }
    fn lbu(&mut self, rt: u32, offset: i32, rs: u32) -> &mut Mips {
        self.i(0x24, rs, rt, offset as u32)
    }
    fn andi(&mut self, rt: u32, rs: u32, imm: u32) -> &mut Mips {
        self.i(0x0C, rs, rt, imm)
    }
    fn xori(&mut self, rt: u32, rs: u32, imm: u32) -> &mut Mips {
        self.i(0x0E, rs, rt, imm)
    }
    fn and(&mut self, rd: u32, rs: u32, rt: u32) -> &mut Mips {
        self.w(rs << 21 | rt << 16 | rd << 11 | 0x24)
    }
    fn nop(&mut self) -> &mut Mips {
        self.w(0)
    }
    fn branch(&mut self, op: u32, rs: u32, rt: u32, to: &'static str) -> &mut Mips {
        self.branches.push((self.words.len(), to));
        self.i(op, rs, rt, 0)
    }
    fn beq(&mut self, rs: u32, rt: u32, to: &'static str) -> &mut Mips {
        self.branch(0x04, rs, rt, to)
    }
    fn bne(&mut self, rs: u32, rt: u32, to: &'static str) -> &mut Mips {
        self.branch(0x05, rs, rt, to)
    }
    /// The words as bytes, big-endian or little, each branch's offset filled in.
    fn bytes(&mut self, big_endian: bool) -> Vec<u8> {
        for (at, to) in std::mem::take(&mut self.branches) {
            let target = self.labels.iter().find(|l| l.0 == to).unwrap_or_else(|| panic!("no label {to}")).1;
            let offset = target as i32 - at as i32 - 1;
            self.words[at] |= offset as u32 & 0xFFFF;
        }
        self.words.iter().flat_map(|w| if big_endian { w.to_be_bytes() } else { w.to_le_bytes() }).collect()
    }
}

/// A PS-EXE loaded at $80010000: the GPU reset, its display set to 320 x 240 and on; then, again and again, port 1's
/// pad read over SIO0 as a console's kernel reads it (01h, 42h and three bytes; psx-spx "Controller and Memory Card
/// Signals"), and the screen filled blue with nothing held and white while any of its buttons is. The kernel is not
/// called, so the program's view of the pad is the serial port's alone.
fn playstation() -> Vec<u8> {
    let mut m = Mips::default();
    m.lui(S0, 0x1F80).sw(ZERO, 0x1814, S0); // GP1(00h) reset
    for (hi, lo) in [(0x0800, 0x0001), (0x06C6, 0x0260), (0x0704, 0x2018)] {
        m.lui(T0, hi).ori(T0, T0, lo).sw(T0, 0x1814, S0); // display mode, horizontal and vertical ranges
    }
    for (hi, lo) in [(0xE300, 0x0000), (0xE403, 0xBD3F), (0xE500, 0x0000)] {
        m.lui(T0, hi).ori(T0, T0, lo).sw(T0, 0x1810, S0); // drawing area and offset
    }
    m.lui(T0, 0x0300).sw(T0, 0x1814, S0); // display on
    m.addiu(T0, ZERO, 0x40).sh(T0, 0x104A, S0).addiu(T0, ZERO, 0x0D).sh(T0, 0x1048, S0).addiu(T0, ZERO, 0x88).sh(T0, 0x104E, S0); // SIO0 reset, 8 bits
    m.label("loop").addiu(T0, ZERO, 0x1003).sh(T0, 0x104A, S0); // port 1 selected
    m.delay(500);
    for (k, (byte, keep)) in [(0x01, T6), (0x42, T5), (0x00, T7), (0x00, T3), (0x00, T4)].into_iter().enumerate() {
        let waiting: &'static str = Box::leak(format!("rx{k}").into_boxed_str());
        let got: &'static str = Box::leak(format!("got{k}").into_boxed_str());
        m.addiu(T0, ZERO, byte).sb(T0, 0x1040, S0).addiu(T0, ZERO, !0x80).sw(T0, 0x1070, S0).addiu(T0, ZERO, 0x1013).sh(T0, 0x104A, S0);
        m.addiu(T1, ZERO, 4000).label(waiting).lhu(T2, 0x1044, S0).nop().andi(T2, T2, 2).bne(T2, ZERO, got).addiu(T1, T1, -1);
        m.bne(T1, ZERO, waiting).nop().label(got).lbu(keep, 0x1040, S0);
        m.delay(300);
    }
    m.sh(ZERO, 0x104A, S0); // deselected
    m.lui(T0, 0x02FF).xori(T5, T5, 0xFF).beq(T5, ZERO, "fill").nop(); // blue; no pad answered
    m.and(T3, T3, T4).xori(T3, T3, 0xFF).beq(T3, ZERO, "fill").nop();
    m.lui(T0, 0x02FF).ori(T0, T0, 0xFFFF); // white
    m.label("fill").sw(T0, 0x1810, S0).sw(ZERO, 0x1810, S0).lui(T0, 0x00F0).ori(T0, T0, 0x0140).sw(T0, 0x1810, S0);
    m.delay(0x10000);
    m.beq(ZERO, ZERO, "loop").nop();
    let code = m.bytes(false);
    let size = code.len().div_ceil(0x800) * 0x800;
    let mut exe = vec![0u8; 0x800 + size];
    exe[..8].copy_from_slice(b"PS-X EXE");
    for (at, v) in [(0x10, 0x8001_0000u32), (0x18, 0x8001_0000), (0x1C, size as u32), (0x30, 0x801F_FFF0)] {
        exe[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    let region = b"Sony Computer Entertainment Inc. for North America area";
    exe[0x4C..0x4C + region.len()].copy_from_slice(region);
    exe[0x800..0x800 + code.len()].copy_from_slice(&code);
    exe
}

/// A 1 MiB cartridge, big-endian, whose boot code (run from the RSP's DMEM at $A4000040 once the PIF's boot is done)
/// sets the video interface to 320 x 240 in 16 bits over a frame buffer at $00100000, then again and again reads port
/// 1's controller through the PIF (a joybus read command, by the serial interface's DMA) and fills the frame buffer blue
/// with nothing held and white while A, B, Z or Start is; its audio interface plays silence, a buffer queued whenever
/// the interface's queue has room.
fn nintendo_64() -> Vec<u8> {
    let mut m = Mips::default();
    m.lui(S0, 0xA440); // VI
    #[rustfmt::skip]
    let vi: [u32; 14] = [0x0000_320E, 0x0010_0000, 320, 2, 0, 0x03E5_2239, 0x0000_020D, 0x0000_0C15, 0x0C15_0C15, 0x006C_02EC, 0x0025_01FF, 0x000E_0204, 0x0000_0200, 0x0000_0400];
    for (n, v) in vi.iter().enumerate() {
        m.lui(T0, v >> 16).ori(T0, T0, v & 0xFFFF).sw(T0, 4 * n as i32, S0);
    }
    // The PIF command block at $A0200000: read controller 1 (1 byte out, 4 back, command 1), the end, and the start bit.
    m.lui(S1, 0xA020);
    m.lui(T0, 0xFF01).ori(T0, T0, 0x0401).sw(T0, 0, S1);
    m.lui(T0, 0xFFFF).ori(T0, T0, 0xFFFF).sw(T0, 4, S1);
    m.lui(T0, 0xFE00).sw(T0, 8, S1);
    for at in (12..60).step_by(4) {
        m.sw(ZERO, at, S1);
    }
    m.addiu(T0, ZERO, 1).sw(T0, 60, S1);
    // The audio interface at 32 kHz from the NTSC clock (48,681,812 / 32,000 - 1), fed silence from $00300000.
    m.lui(S2, 0xA450).addiu(T0, ZERO, 1).sw(T0, 8, S2).addiu(T0, ZERO, 1520).sw(T0, 0x10, S2).addiu(T0, ZERO, 15).sw(T0, 0x14, S2);
    m.lui(S0, 0xA480); // SI
    m.label("loop").lw(T1, 0xC, S2).nop().srl(T1, T1, 31).bne(T1, ZERO, "queued").nop(); // a buffer more unless full
    m.lui(T0, 0x0030).sw(T0, 0, S2).addiu(T0, ZERO, 0x2000).sw(T0, 4, S2);
    m.label("queued").lui(T0, 0x0020).sw(T0, 0, S0).lui(T0, 0x1FC0).ori(T0, T0, 0x07C0).sw(T0, 0x10, S0); // write the block
    m.label("w1").lw(T1, 0x18, S0).andi(T1, T1, 3).bne(T1, ZERO, "w1").nop();
    m.lui(T0, 0x0020).sw(T0, 0, S0).lui(T0, 0x1FC0).ori(T0, T0, 0x07C0).sw(T0, 4, S0); // read it back
    m.label("w2").lw(T1, 0x18, S0).andi(T1, T1, 3).bne(T1, ZERO, "w2").nop();
    m.lbu(T3, 4, S1).andi(T3, T3, 0xF0); // A B Z Start
    m.lui(T0, 0x003F).ori(T0, T0, 0x003F); // blue
    m.beq(T3, ZERO, "fill").nop();
    m.lui(T0, 0xFFFF).ori(T0, T0, 0xFFFF); // white
    // The block's first word set again, as the PIF's answer writes over it.
    m.label("fill").lui(T4, 0xFF01).ori(T4, T4, 0x0401).sw(T4, 0, S1).lui(T4, 0xFFFF).ori(T4, T4, 0xFFFF).sw(T4, 4, S1);
    m.lui(T2, 0xA010).lui(T1, 0x0002).ori(T1, T1, 0x5800); // 320 x 240 x 2 bytes
    m.label("pixels").sw(T0, 0, T2).addiu(T2, T2, 4).addiu(T1, T1, -4).bne(T1, ZERO, "pixels").nop();
    m.beq(ZERO, ZERO, "loop").nop();
    let code = m.bytes(true);
    let mut rom = vec![0u8; 0x10_0000];
    rom[0..4].copy_from_slice(&0x8037_1240u32.to_be_bytes());
    rom[8..12].copy_from_slice(&0x8000_0400u32.to_be_bytes());
    rom[0x20..0x2B].copy_from_slice(b"PAD COLOURS");
    rom[0x3E] = b'E';
    rom[0x40..0x40 + code.len()].copy_from_slice(&code);
    rom
}
