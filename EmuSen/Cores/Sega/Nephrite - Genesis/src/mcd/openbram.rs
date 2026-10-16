//! The open BIOS's backup RAM calls (`bios/bram.s`, Nephrite_OpenBios.md §14) run on a bare 68000 over the sub CPU's
//! program: the block code against Nephrite_Bram.md §6.5's vectors, and each call against what the page observed.

use beryl_m68k::{Access, Bus, M68000, Registers, Size, Step};

use super::bios::{OPEN_BIOS, symbols};

const SUB_AT: usize = 0x1_0000;
const SUB_SIZE: usize = 0x6000;
const PRG: usize = 0x8_0000;
const STUB: u32 = 0x7_0000;
const STACK: u32 = 0x7_F000;
const BURAM: u32 = 0x5F16;
const BRAM_AT: u32 = 0xFE_0000;
/// A volume of another size, reached through the file system's own entry as a cartridge's would be.
const CART_AT: u32 = 0x60_0000;
const TABLE: u32 = 0x4_0000;
const DATA: u32 = 0x4_1000;
const BACK: u32 = 0x5_0000;

const INIT: u16 = 0;
const STAT: u16 = 1;
const SERCH: u16 = 2;
const READ: u16 = 3;
const WRITE: u16 = 4;
const DEL: u16 = 5;
const FORMAT: u16 = 6;
const DIR: u16 = 7;
const VERIFY: u16 = 8;

struct Rig {
    prg: Vec<u8>,
    bram: Vec<u8>,
    cart: Vec<u8>,
}

#[derive(Debug)]
struct Out {
    d0: u32,
    d1: u32,
    a0: u32,
    carry: bool,
}

impl Rig {
    fn new(cart: usize) -> Rig {
        let mut prg = vec![0; PRG];
        prg[..SUB_SIZE].copy_from_slice(&OPEN_BIOS[SUB_AT..SUB_AT + SUB_SIZE]);
        Rig { prg, bram: vec![0; 0x2000], cart: vec![0; cart] }
    }

    fn byte(&self, a: u32) -> u8 {
        let a = a & 0xFF_FFFF;
        match a {
            0..0x8_0000 => self.prg[a as usize],
            0xFE_0000..=0xFE_FFFF if a & 1 == 1 => self.bram[((a - BRAM_AT) >> 1) as usize % self.bram.len()],
            0x60_0000..=0x6F_FFFF if a & 1 == 1 => self.cart[((a - CART_AT) >> 1) as usize % self.cart.len()],
            _ => 0xFF,
        }
    }

    fn set(&mut self, a: u32, v: u8) {
        let a = a & 0xFF_FFFF;
        match a {
            0..0x8_0000 => self.prg[a as usize] = v,
            0xFE_0000..=0xFE_FFFF if a & 1 == 1 => {
                let n = self.bram.len();
                self.bram[((a - BRAM_AT) >> 1) as usize % n] = v;
            }
            0x60_0000..=0x6F_FFFF if a & 1 == 1 => {
                let n = self.cart.len();
                self.cart[((a - CART_AT) >> 1) as usize % n] = v;
            }
            _ => {}
        }
    }

    fn put(&mut self, at: u32, bytes: &[u8]) {
        self.prg[at as usize..at as usize + bytes.len()].copy_from_slice(bytes);
    }

    fn get(&self, at: u32, n: usize) -> Vec<u8> {
        self.prg[at as usize..at as usize + n].to_vec()
    }

    /// `target` called by a JSR with the registers `setup` gives, run to its return to the stub's BRA to itself.
    fn call(&mut self, target: u32, setup: impl FnOnce(&mut Registers)) -> Out {
        let stub = [0x4E, 0xB9, (target >> 24) as u8, (target >> 16) as u8, (target >> 8) as u8, target as u8, 0x60, 0xFE];
        self.put(STUB, &stub);
        let vectors: Vec<u8> = self.get(0, 8);
        self.put(0, &[STACK.to_be_bytes(), STUB.to_be_bytes()].concat());
        let mut cpu = M68000::new();
        cpu.reset(self);
        self.put(0, &vectors);
        setup(&mut cpu.regs);
        let mut last = 0;
        for _ in 0..200_000_000u64 {
            let pc = cpu.regs.pc;
            if pc == STUB + 6 {
                let r = cpu.regs;
                return Out { d0: r.d[0], d1: r.d[1], a0: r.a[0], carry: r.sr & 1 != 0 };
            }
            match cpu.step(self) {
                Step::Exception(v) => panic!("exception {v} at {pc:06X} after {last:06X}: {:08X?} {:08X?}", cpu.regs.d, cpu.regs.a),
                Step::Halted => panic!("halted"),
                _ => last = pc,
            }
        }
        panic!("the call did not return");
    }

    /// `_BURAM` on the internal RAM.
    fn buram(&mut self, f: u16, d1: u32, a0: u32, a1: u32) -> Out {
        self.call(BURAM, |r| {
            r.d[0] = f as u32;
            r.d[1] = d1;
            r.a[0] = a0;
            r.a[1] = a1;
        })
    }

    /// The same file system on the cartridge volume.
    fn cart_call(&mut self, f: u16, a0: u32, a1: u32) -> Out {
        let size = self.cart.len() as u32;
        self.call(symbols::BRAM_CALL, |r| {
            r.d[0] = f as u32;
            r.a[0] = a0;
            r.a[1] = a1;
            r.a[5] = CART_AT + 1;
            r.d[7] = size;
            r.a[6] = symbols::BRAMWORK + symbols::BW_SIZE;
        })
    }

    /// A parameter table: the name, the flag and the blocks.
    fn table(&mut self, name: &str, flag: u8, blocks: u16) -> u32 {
        let mut t = name.as_bytes().to_vec();
        assert_eq!(t.len(), 11);
        t.push(flag);
        t.extend_from_slice(&blocks.to_be_bytes());
        self.put(TABLE, &t);
        TABLE
    }

    fn write(&mut self, name: &str, flag: u8, data: &[u8]) -> Out {
        let unit = if flag == 0 { 64 } else { 32 };
        assert_eq!(data.len() % unit, 0);
        let t = self.table(name, flag, (data.len() / unit) as u16);
        self.put(DATA, data);
        self.buram(WRITE, 0, t, DATA)
    }

    fn read(&mut self, name: &str) -> (Out, Vec<u8>) {
        let t = self.table(name, 0, 0);
        let out = self.buram(READ, 0, t, BACK);
        let n = (out.d0 & 0xFFFF) as usize * if out.d1 as u8 == 0 { 64 } else { 32 };
        let data = self.get(BACK, n);
        (out, data)
    }

    fn stat(&mut self) -> (u16, u16) {
        let out = self.buram(STAT, 0, 0, 0);
        assert!(!out.carry);
        (out.d0 as u16, out.d1 as u16)
    }

    fn formatted() -> Rig {
        let mut rig = Rig::new(0x2000);
        assert!(!rig.buram(FORMAT, 0, 0, 0).carry);
        rig
    }
}

impl Bus for Rig {
    fn read(&mut self, a: Access) -> u16 {
        match a.size {
            Size::Word => (self.byte(a.address & !1) as u16) << 8 | self.byte(a.address | 1) as u16,
            Size::Byte => self.byte(a.address) as u16,
        }
    }

    fn write(&mut self, a: Access, v: u16) {
        match a.size {
            Size::Word => {
                self.set(a.address & !1, (v >> 8) as u8);
                self.set(a.address | 1, v as u8);
            }
            Size::Byte => self.set(a.address, v as u8),
        }
    }

    fn idle(&mut self, _clocks: u32) {}

    fn interrupt_level(&mut self) -> u8 {
        0
    }

    fn acknowledge(&mut self, _level: u8) -> Option<u8> {
        None
    }
}

fn hex(t: &str) -> Vec<u8> {
    t.split_whitespace().map(|b| u8::from_str_radix(b, 16).unwrap()).collect()
}

/// A fresh volume's footer (Nephrite_Bram.md §4).
fn footer(free: u16) -> Vec<u8> {
    let mut f = vec![0x5F; 11];
    f.extend_from_slice(&[0, 0, 0, 0, 0x40]);
    for _ in 0..4 {
        f.extend_from_slice(&free.to_be_bytes());
    }
    f.extend_from_slice(&[0; 8]);
    f.extend_from_slice(b"SEGA_CD_ROM\0\x01\0\0\0RAM_CARTRIDGE___");
    f
}

fn pattern(n: usize, seed: u8) -> Vec<u8> {
    (0..n).map(|i| (i as u8).wrapping_mul(7).wrapping_add(seed)).collect()
}

const SONIC: &str = "00 D4 63 00 00 03 03 00 03 03 03 00 03 00 00 03 00 03 00 00 00 03 03 00 53 D2 F5 3A 48 50 35 0F
    47 14 F5 7E 5C D4 F3 03 00 03 12 00 0A FF CA A6 F5 27 ED 22 47 FA 22 96 6C A5 88 14 48 48 0A BB";
const LUNAR: &str = "1D 6F B7 5C 44 17 F6 30 56 53 E7 05 50 8D 27 7E 30 82 00 03 0F C0 00 39 55 D1 47 7E 4E 17 55 3A
    42 56 22 CA 5C CF 33 00 00 03 12 00 0C B8 27 11 78 C6 1D 2B 53 F6 BB 74 AF EB 95 E7 21 FA E8 8B";
const ZEROS: &str = "00 00 00 00 00 00 00 00 03 03 03 00 00 03 03 03 00 00 00 00 00 00 00 00 03 03 03 00 00 03 00 00
    00 03 03 00 00 00 03 03 00 03 03 00 00 3C FF FF 44 09 09 00 00 00 00 00 78 F6 F6 00 00 00 03 03";
const FOX: &str = "53 0C A6 50 4B 11 50 81 53 56 55 24 42 D2 B1 81 41 95 24 3F 56 D1 E1 81 44 90 F6 63 21 12 A6 55
    4D 55 05 4E 22 11 F6 59 47 56 22 82 55 2B FF 14 24 99 99 CA 60 14 21 60 6C 30 BD A6 9F D8 8D E2";

// Nephrite_Bram.md §6.5: the two directory blocks and the two protected blocks the player's BIOS wrote, made by the open BIOS's own calls.
#[test]
fn the_block_code_gives_the_pages_four_vectors() {
    let mut rig = Rig::formatted();
    assert!(!rig.write("SONICCD____", 0, &pattern(11 * 64, 1)).carry);
    assert_eq!(rig.bram[0x1F80..0x1FC0], hex(SONIC)[..]);

    let mut rig = Rig::formatted();
    assert!(!rig.write("WD_LUNAR2_3", 0, &[0; 14 * 64]).carry);
    assert!(!rig.write("WD_LUNAR2_2", 0, &[0; 14 * 64]).carry);
    assert_eq!(rig.bram[0x1F80..0x1FC0], hex(LUNAR)[..]);

    let mut rig = Rig::formatted();
    assert!(!rig.write("PROTECTED__", 0xFF, &[0; 32]).carry);
    assert_eq!(rig.bram[0x40..0x80], hex(ZEROS)[..]);

    let mut rig = Rig::formatted();
    assert!(!rig.write("PROTECTED__", 0xFF, b"THE QUICK BROWN FOX JUMPS OVER T").carry);
    assert_eq!(rig.bram[0x40..0x80], hex(FOX)[..]);
}

// §4: a format writes the footer and nothing else, with S/64 - 3 blocks free.
#[test]
fn a_format_writes_only_the_footer() {
    let mut rig = Rig::new(0x2_0000);
    rig.bram.fill(0xA5);
    rig.cart.fill(0xA5);
    assert!(!rig.buram(FORMAT, 0, 0, 0).carry);
    assert!(rig.bram[..0x1FC0].iter().all(|&b| b == 0xA5));
    assert_eq!(rig.bram[0x1FC0..], footer(125)[..]);
    assert!(!rig.cart_call(FORMAT, 0, 0).carry);
    assert!(rig.cart[..0x1_FFC0].iter().all(|&b| b == 0xA5));
    assert_eq!(rig.cart[0x1_FFC0..], footer(2045)[..]);
}

// §3: what BRMINIT and BRMSTAT make of the footer: the identifier exact, one of three longwords, the counts by three of four copies.
#[test]
fn init_and_stat_read_the_footer_as_the_page_says() {
    let mut rig = Rig::new(0x2000);
    let out = rig.buram(INIT, 0, 0, BACK);
    assert!(out.carry && out.d1 as u16 == 1, "zeros are unformatted");
    let mut rig = Rig::formatted();
    let out = rig.buram(INIT, 0, 0, BACK);
    assert!(!out.carry && out.d0 as u16 == 2);
    assert_eq!(rig.get(BACK, 12), b"SEGA_CD_ROM\0");
    assert_eq!(rig.stat(), (125, 0));

    let at = 0x2000 - 0x20;
    rig.bram[at + 9] = b'A';
    let out = rig.buram(INIT, 0, 0, BACK);
    assert!(out.carry && out.d1 as u16 == 2, "SEGA_CD_RAM is another format");
    rig.bram[at + 9] = b'O';
    rig.bram[at + 3] = b'@';
    assert_eq!(rig.buram(INIT, 0, 0, BACK).d1 as u16, 1, "a name outside 0-9 A-Z _ is unformatted");
    rig.bram[at + 3] = b'A';

    let cart = 0x2000 - 0x10;
    rig.bram[cart..cart + 12].fill(0);
    assert_eq!(rig.buram(INIT, 0, 0, BACK).d1 as u16, 1, "only the fourth longword whole");
    rig.bram[cart + 8..cart + 12].copy_from_slice(b"RIDG");
    assert!(!rig.buram(INIT, 0, 0, BACK).carry, "the third longword whole");

    let free = 0x2000 - 0x30;
    rig.bram[free + 1] = 0x70;
    assert_eq!(rig.stat(), (125, 0), "one copy changed");
    rig.bram[free + 3] = 0x70;
    assert_eq!(rig.stat(), (0, 0), "two against two");
    rig.bram[free + 5] = 0x70;
    assert_eq!(rig.stat(), (0x70, 0), "three of four, read and not recomputed");
}

// §5.4, §7.2: Lunar's four files at blocks 1, 15, 29 and 43, and the free count after each; then each found, read and verified.
#[test]
fn files_are_placed_counted_found_and_read() {
    let mut rig = Rig::formatted();
    let mut free = vec![];
    for (k, name) in ["WD_LUNAR2_3", "WD_LUNAR2_2", "WD_LUNAR2_1", "WD_LUNAR2_0"].iter().enumerate() {
        assert!(!rig.write(name, 0, &pattern(14 * 64, k as u8)).carry);
        free.push(rig.stat().0);
    }
    assert_eq!(free, [111, 96, 82, 67]);
    assert_eq!(rig.stat().1, 4);
    for (k, name) in ["WD_LUNAR2_3", "WD_LUNAR2_2", "WD_LUNAR2_1", "WD_LUNAR2_0"].iter().enumerate() {
        let t = rig.table(name, 0, 0);
        let out = rig.buram(SERCH, 0, t, 0);
        assert!(!out.carry);
        assert_eq!((out.d0 as u16, out.d1 as u8), (14, 0));
        assert_eq!(out.a0, BRAM_AT + 128 * (1 + 14 * k as u32), "the first block's address, $FE0080 for block 1");
        let (out, data) = rig.read(name);
        assert!(!out.carry);
        assert_eq!(data, pattern(14 * 64, k as u8));
        let t = rig.table(name, 0, 14);
        rig.put(DATA, &pattern(14 * 64, k as u8));
        assert!(!rig.buram(VERIFY, 0, t, DATA).carry);
    }
    let t = rig.table("WD_LUNAR2_1", 0, 14);
    rig.put(DATA, &pattern(14 * 64, 9));
    let out = rig.buram(VERIFY, 0, t, DATA);
    assert!(out.carry && out.d0 as i16 == -1, "different data");
    let t = rig.table("NOT_THERE__", 0, 14);
    let out = rig.buram(VERIFY, 0, t, DATA);
    assert!(out.carry && out.d0 as u16 == 0, "not found");
    assert!(rig.buram(SERCH, 0, t, 0).carry);
    assert!(rig.write("TOO_BIG____", 0, &vec![0; 68 * 64]).carry, "67 blocks free, the next entry's block among them");
    assert!(!rig.write("JUST_FITS__", 0, &vec![0; 67 * 64]).carry);
    assert_eq!(rig.stat(), (0, 5));
}

// §7.3: the second of three files erased: the third moved down, its entry's start lowered, the vacated blocks and the directory block past the files left as they were.
#[test]
fn a_delete_moves_the_later_files_down() {
    let mut rig = Rig::formatted();
    let marked = |c: u8, n: usize| -> Vec<u8> { (0..n).flat_map(|k| vec![c, k as u8].repeat(32)).collect() };
    rig.write("AAAA_______", 0, &marked(b'A', 3));
    rig.write("BBBB_______", 0, &marked(b'B', 2));
    rig.write("CCCC_______", 0, &marked(b'C', 4));
    assert_eq!(rig.stat(), (115, 3));
    let block1 = rig.bram[0x1F40..0x1F80].to_vec();
    let t = rig.table("BBBB_______", 0, 0);
    assert!(!rig.buram(DEL, 0, t, 0).carry);
    assert_eq!(rig.stat(), (117, 2));
    assert_eq!(rig.bram[4 * 64..8 * 64], marked(b'C', 4)[..]);
    assert_eq!(rig.bram[8 * 64..10 * 64], marked(b'C', 4)[2 * 64..]);
    assert_eq!(rig.bram[0x1F40..0x1F80], block1[..]);
    let t = rig.table("CCCC_______", 0, 0);
    assert_eq!(rig.buram(SERCH, 0, t, 0).a0, BRAM_AT + 128 * 4);
    assert_eq!(rig.read("CCCC_______").1, marked(b'C', 4));
    assert_eq!(rig.read("AAAA_______").1, marked(b'A', 3));
    let t = rig.table("BBBB_______", 0, 0);
    assert!(rig.buram(DEL, 0, t, 0).carry);
}

// D-50: a name written again with its blocks and flag in place, with other blocks deleted and written anew.
#[test]
fn a_name_written_again_replaces_its_file() {
    let mut rig = Rig::formatted();
    rig.write("SONICCD____", 0, &pattern(11 * 64, 1));
    rig.write("OTHER______", 0, &pattern(2 * 64, 5));
    assert!(!rig.write("SONICCD____", 0, &pattern(11 * 64, 2)).carry);
    assert_eq!(rig.stat(), (111, 2));
    assert_eq!(rig.read("SONICCD____").1, pattern(11 * 64, 2));
    assert!(!rig.write("SONICCD____", 0, &pattern(12 * 64, 3)).carry);
    assert_eq!(rig.stat(), (110, 2));
    assert_eq!(rig.read("SONICCD____").1, pattern(12 * 64, 3));
    assert_eq!(rig.read("OTHER______").1, pattern(2 * 64, 5));
    let t = rig.table("OTHER______", 0, 0);
    assert_eq!(rig.buram(SERCH, 0, t, 0).a0, BRAM_AT + 128, "moved down to block 1");
}

// §6.7: one symbol error a codeword corrected, the low bits ignored; two in one codeword fail BRMREAD (D-51).
#[test]
fn protected_files_are_corrected_as_the_page_observed() {
    let mut rig = Rig::formatted();
    let fox = b"THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG 0123456789__________".to_vec();
    rig.write("PROTECTED__", 0xFF, &fox);
    let (out, data) = rig.read("PROTECTED__");
    assert_eq!((out.carry, out.d0 as u16, out.d1 as u8), (false, 2, 0xFF));
    assert_eq!(data, fox);
    for i in 0..8 {
        rig.bram[64 + i] ^= 0xFC;
    }
    for b in &mut rig.bram[128..192] {
        *b ^= 3;
    }
    let (out, data) = rig.read("PROTECTED__");
    assert!(!out.carry);
    assert_eq!(data, fox);
    rig.bram[64 + 9] ^= 0xFC;
    assert!(rig.read("PROTECTED__").0.carry, "symbols 0 and 9 are both codeword 0's");
}

// BRMDIR (D-52): `*` matches the rest, the files to skip, carry set when a match finds the buffer full.
#[test]
fn the_directory_lists_by_template() {
    let mut rig = Rig::formatted();
    for name in ["WD_LUNAR2_3", "SONICCD____", "WD_LUNAR2_2", "WD_LUNAR2_1"] {
        rig.write(name, 0, &[0; 64]);
    }
    rig.put(TABLE, b"WD_LUNAR2*_\0");
    let out = rig.buram(DIR, 8, TABLE, BACK);
    assert!(!out.carry);
    let names: Vec<Vec<u8>> = (0..3).map(|k| rig.get(BACK + 16 * k, 11)).collect();
    assert_eq!(names, [b"WD_LUNAR2_3".to_vec(), b"WD_LUNAR2_2".to_vec(), b"WD_LUNAR2_1".to_vec()]);
    assert_eq!(rig.get(BACK + 12, 4), [0, 1, 0, 1]);
    assert!(rig.buram(DIR, 2, TABLE, BACK).carry);
    rig.prg[BACK as usize..BACK as usize + 32].fill(0);
    assert!(!rig.buram(DIR, 2 << 16 | 1, TABLE, BACK).carry);
    assert_eq!(rig.get(BACK, 11), b"WD_LUNAR2_1");
}

// §8: the same code on a 128 KiB volume, the cartridge's format: a 121-block file leaves 1,924 free.
#[test]
fn the_file_system_runs_on_a_cartridge_sized_volume() {
    let mut rig = Rig::new(0x2_0000);
    assert!(!rig.cart_call(FORMAT, 0, 0).carry);
    let data = pattern(121 * 64, 4);
    let t = rig.table("UNITS______", 0, 121);
    rig.put(DATA, &data);
    assert!(!rig.cart_call(WRITE, t, DATA).carry);
    assert_eq!(rig.cart_call(STAT, 0, 0).d0 as u16, 1924);
    let t = rig.table("UNITS______", 0, 0);
    assert!(!rig.cart_call(READ, t, BACK).carry);
    assert_eq!(rig.get(BACK, data.len()), data);
    assert_eq!(rig.cart_call(INIT, 0, 0).d0 as u16, 0x20);
}
