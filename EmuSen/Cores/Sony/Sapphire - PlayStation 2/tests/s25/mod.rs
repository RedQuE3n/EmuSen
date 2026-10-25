//! The project's S25 programs (Sapphire_Native.md §30): bare IOP programs in S21's form (`tests/s21`'s `Program`) that
//! drive SIO2 at its registers, ps2tek's "SIO2 Registers" and its program flow, and grade what the DualShock 2 and the
//! memory card answer against ps2tek's "SIO2 Controller Commands", "SIO2 Standard Controllers" and "SIO2 PS2 Memcards",
//! psx-spx's "Controllers - Configuration Commands" to "Controllers - Analog Buttons (Dualshock2)", and Ross Ridge's "PlayStation 2 Memory
//! Card File System" for the spare bytes. Every expected byte is derived here from those documents, written apart from
//! `src/sio2/`; where they give no value a word is recorded ungraded. A transfer is one command a SEND3 entry, its
//! bytes written to FIFOIN one at a time (or by SIO2in, channel 11) and its reply read from FIFOOUT (or by SIO2out,
//! channel 12) after the SIO2 interrupt, I_STAT's bit 17, every wait bounded.

#![allow(dead_code)]

use crate::s21::{self, Program};

/// SIO2's registers through kseg1 (ps2tek "SIO2 Registers").
pub mod reg {
    pub const fn send3(n: u32) -> u32 {
        0xBF80_8200 + 4 * n
    }
    pub const fn send1(port: u32) -> u32 {
        0xBF80_8240 + 8 * port
    }
    pub const fn send2(port: u32) -> u32 {
        0xBF80_8244 + 8 * port
    }
    pub const FIFOIN: u32 = 0xBF80_8260;
    pub const FIFOOUT: u32 = 0xBF80_8264;
    pub const CTRL: u32 = 0xBF80_8268;
    pub const RECV1: u32 = 0xBF80_826C;
    pub const RECV2: u32 = 0xBF80_8270;
    pub const RECV3: u32 = 0xBF80_8274;
    pub const ISTAT: u32 = 0xBF80_8280;
}

/// The SIO2 interrupt's bit in I_STAT (ps2tek "IOP Interrupts": IRQ17).
pub const IRQ_SIO2: u32 = 1 << 17;
/// The ports: the two controller ports and the two card slots, not wired together on the PlayStation 2 (psx-spx "PS2
/// multitap"; ps2tek's SEND3 bits 0-1).
pub const PAD1: u32 = 0;
pub const PAD2: u32 = 1;
pub const CARD1: u32 = 2;
pub const CARD2: u32 = 3;
/// RECV1 after a connected and after a disconnected peripheral (ps2tek "SIO2_RECV1").
pub const CONNECTED: u32 = 0x1100;
/// RECV1 after a peripheral that is not there: a controller's command must leave bit 13 set and a card's bits 15-12 other
/// than 1 (ps2tek's two rules), and ps2tek's one value, 1D100h, meets only the second (Sapphire_Disputes.md D-221).
pub const NO_PAD: u32 = 0x2D100;
pub const NO_DEVICE: u32 = 0x1D100;
/// The card's reply byte and its default terminator (ps2tek "SIO2 PS2 Memcards").
pub const ACK: u8 = 0x2B;
pub const TERMINATOR: u8 = 0x55;

/// A SEND3 entry: the port, and the command's length counting the peripheral byte, in both length fields.
pub const fn send3(port: u32, len: u32) -> u32 {
    port | len << 8 | len << 18
}

/// Expected bytes, `None` for any.
pub type Bytes = Vec<Option<u8>>;

pub fn exact(b: &[u8]) -> Bytes {
    b.iter().map(|&x| Some(x)).collect()
}

pub fn any(n: usize) -> Bytes {
    vec![None; n]
}

/// The program's own steps on SIO2, beside `Program`'s.
pub trait Sio2 {
    /// Resets SIO2 (CTRL with bits 2 and 3, the rest as read) and loads `entries` into SEND3 with a zero after them.
    fn queue(&mut self, entries: &[u32]);
    /// Writes `bytes` to FIFOIN.
    fn fifo_in(&mut self, bytes: &[u8]);
    /// Starts the queue (CTRL with bit 0) and waits, bounded, for the SIO2 interrupt, which it acknowledges; the passes
    /// left are in t5.
    fn start(&mut self);
    /// One command to `port` by FIFOIN: queued, written, started.
    fn command(&mut self, port: u32, bytes: &[u8]);
    /// Reads `want.len()` bytes from FIFOOUT, recording them four to a word, little-endian, graded where every byte of a
    /// word is given.
    fn reply(&mut self, want: &Bytes);
    /// Records RECV1.
    fn recv1(&mut self, want: Option<u32>);
    /// One exchange: the command, then RECV1 and its reply.
    fn exchange(&mut self, port: u32, bytes: &[u8], recv: Option<u32>, want: &Bytes);
}

impl Sio2 for Program {
    fn queue(&mut self, entries: &[u32]) {
        self.li("t6", reg::CTRL);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
        self.li("t7", !0xF);
        self.emit("and t2, t2, t7");
        self.emit("ori t3, t2, 0xc");
        self.emit("sw t3, 0(t6)");
        for (i, &e) in entries.iter().chain(std::iter::once(&0)).enumerate() {
            self.poke(reg::send3(i as u32), e);
        }
    }

    fn fifo_in(&mut self, bytes: &[u8]) {
        let skip = self.label();
        self.emit(format!("beq zero, zero, {skip}"));
        self.emit("nop");
        let at = self.pc();
        for w in bytes.chunks(4) {
            let v = w.iter().enumerate().fold(0u32, |v, (i, &b)| v | u32::from(b) << (8 * i));
            self.emit(format!(".word 0x{v:08x}"));
        }
        self.place(&skip);
        let l = self.label();
        self.li("t6", reg::FIFOIN);
        self.li("t4", at);
        self.li("t5", bytes.len() as u32);
        self.place(&l);
        self.emit("lbu t7, 0(t4)");
        self.emit("addiu t4, t4, 1");
        self.emit("sb t7, 0(t6)");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
    }

    fn start(&mut self) {
        self.li("t6", reg::CTRL);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
        self.li("t7", !0xF);
        self.emit("and t2, t2, t7");
        self.emit("ori t3, t2, 1");
        self.emit("sw t3, 0(t6)");
        self.wait(s21::reg::I_STAT, IRQ_SIO2, true, 20_000);
        self.poke(s21::reg::I_STAT, !IRQ_SIO2);
    }

    fn command(&mut self, port: u32, bytes: &[u8]) {
        self.queue(&[send3(port, bytes.len() as u32)]);
        self.fifo_in(bytes);
        self.start();
    }

    fn reply(&mut self, want: &Bytes) {
        let l = self.label();
        let n = want.len() as u32;
        self.li("t6", reg::FIFOOUT);
        self.li("t5", n);
        self.place(&l);
        self.emit("lbu t2, 0(t6)");
        self.emit("nop");
        self.emit("sb t2, 0(s1)");
        self.emit("addiu s1, s1, 1");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
        for _ in n..n.next_multiple_of(4) {
            self.emit("sb zero, 0(s1)");
            self.emit("addiu s1, s1, 1");
        }
        for chunk in want.chunks(4) {
            let graded = chunk.iter().all(Option::is_some);
            let w = chunk.iter().enumerate().fold(0u32, |w, (i, b)| w | u32::from(b.unwrap_or(0)) << (8 * i));
            self.expected.push(graded.then_some(w));
        }
    }

    fn recv1(&mut self, want: Option<u32>) {
        self.rec(reg::RECV1, want);
    }

    fn exchange(&mut self, port: u32, bytes: &[u8], recv: Option<u32>, want: &Bytes) {
        assert_eq!(bytes.len(), want.len(), "a reply byte for every byte sent");
        self.command(port, bytes);
        self.recv1(recv);
        self.reply(want);
    }
}

/// A pad command: 01h, the command, 00h, then `data`, padded with zeros to `len` bytes.
pub fn pad(cmd: u8, data: &[u8], len: usize) -> Vec<u8> {
    let mut v = vec![0x01, cmd, 0x00];
    v.extend_from_slice(data);
    v.resize(len, 0);
    v
}

/// A card command: 81h, then `bytes`.
pub fn card(bytes: &[u8]) -> Vec<u8> {
    let mut v = vec![0x81];
    v.extend_from_slice(bytes);
    v
}

/// HiZ, and the pad's header for a mode.
pub const HIZ: u8 = 0xFF;

fn header(mode: u8) -> Bytes {
    exact(&[HIZ, mode, 0x5A])
}

/// A pad reply: the header, then `body` (graded), then `rest` bytes ungraded.
fn pad_reply(mode: u8, body: &[u8], rest: usize) -> Bytes {
    [header(mode), exact(body), any(rest)].concat()
}

/// A card reply to a command of `len` bytes: HiZ for the peripheral and command bytes, then ps2tek's reply list from
/// byte 2 (Sapphire_Disputes.md D-222), `None` where the list has XX.
fn card_reply(list: &[Option<u8>], len: usize) -> Bytes {
    let mut v = vec![Some(HIZ), Some(HIZ)];
    v.extend_from_slice(list);
    v.resize(len, None);
    v.truncate(len);
    v
}

fn ack() -> Vec<Option<u8>> {
    vec![Some(ACK), Some(TERMINATOR)]
}

/// `s25-pad`: the DualShock 2's identification, configuration mode and its commands, analogue mode and the reply bytes
/// 4Fh enables, the motors' map, and the port with no controller (psx-spx "Controllers - Configuration Commands" to
/// "Controllers - Analog Buttons (Dualshock2)"; ps2tek "SIO2 Controller Commands"). Nothing is pressed: the buttons read
/// FFh FFh and the pressures 00h; the sticks' bytes are the player's and ungraded.
pub fn pad_program() -> Program {
    let mut p = Program::uncached("s25-pad");
    p.exchange(PAD1, &pad(0x42, &[], 5), Some(CONNECTED), &pad_reply(0x41, &[0xFF, 0xFF], 0));
    p.exchange(PAD2, &pad(0x42, &[], 5), Some(NO_PAD), &exact(&[HIZ; 5]));
    p.exchange(PAD1, &pad(0x43, &[1], 9), Some(CONNECTED), &pad_reply(0x41, &[0xFF, 0xFF], 4));
    let cfg = |p: &mut Program, cmd: u8, data: &[u8], six: Bytes| {
        p.exchange(PAD1, &pad(cmd, data, 9), Some(CONNECTED), &[header(0xF3), six].concat());
    };
    cfg(&mut p, 0x45, &[], exact(&[0x03, 0x02, 0x00, 0x02, 0x01, 0x00]));
    cfg(&mut p, 0x41, &[], exact(&[0; 6]));
    cfg(&mut p, 0x46, &[0], exact(&[0x00, 0x00, 0x01, 0x02, 0x00, 0x0A]));
    cfg(&mut p, 0x46, &[1], exact(&[0x00, 0x00, 0x01, 0x01, 0x01, 0x14]));
    cfg(&mut p, 0x46, &[2], exact(&[0; 6]));
    cfg(&mut p, 0x47, &[], exact(&[0x00, 0x00, 0x02, 0x00, 0x01, 0x00]));
    cfg(&mut p, 0x48, &[0], exact(&[0x00, 0x00, 0x00, 0x00, 0x01, 0x00]));
    cfg(&mut p, 0x4C, &[0], exact(&[0x00, 0x00, 0x00, 0x04, 0x00, 0x00]));
    cfg(&mut p, 0x4C, &[1], exact(&[0x00, 0x00, 0x00, 0x07, 0x00, 0x00]));
    cfg(&mut p, 0x44, &[1, 3], exact(&[0; 6]));
    cfg(&mut p, 0x45, &[], exact(&[0x03, 0x02, 0x01, 0x02, 0x01, 0x00]));
    cfg(&mut p, 0x44, &[2, 0], exact(&[0x00, 0x00, 0xFF, 0x00, 0x00, 0x00]));
    cfg(&mut p, 0x41, &[], exact(&[0xFF, 0xFF, 0x03, 0x00, 0x00, 0x5A]));
    cfg(&mut p, 0x4F, &[0xFF, 0xFF, 0x03], [exact(&[0; 5]), any(1)].concat());
    cfg(&mut p, 0x41, &[], exact(&[0xFF, 0xFF, 0x03, 0x00, 0x00, 0x5A]));
    cfg(&mut p, 0x4D, &[0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF], exact(&[0xFF; 6]));
    cfg(&mut p, 0x4D, &[0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF], exact(&[0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF]));
    cfg(&mut p, 0x40, &[0, 3], [exact(&[0x00, 0x00, 0x02, 0x00, 0x00]), any(1)].concat());
    cfg(&mut p, 0x40, &[0, 2], [exact(&[0x00, 0x00, 0x03, 0x00, 0x00]), any(1)].concat());
    cfg(&mut p, 0x40, &[0x0C, 0], [exact(&[0x00, 0x00, 0xFF, 0x00, 0x00]), any(1)].concat());
    cfg(&mut p, 0x42, &[], [exact(&[0xFF, 0xFF]), any(4)].concat());
    cfg(&mut p, 0x43, &[0], exact(&[0; 6]));
    // Analogue with every reply byte: mode 79h, the sticks ungraded, twelve pressures of 00h.
    p.exchange(PAD1, &pad(0x42, &[0x00, 0x00], 21), Some(CONNECTED), &[header(0x79), exact(&[0xFF, 0xFF]), any(4), exact(&[0; 12])].concat());
    // 43h outside configuration mode answers as 42h without the pressures (ps2tek), and enters it.
    p.exchange(PAD1, &pad(0x43, &[1], 21), Some(CONNECTED), &[header(0x79), exact(&[0xFF, 0xFF]), any(16)].concat());
    cfg(&mut p, 0x44, &[0, 0], exact(&[0; 6]));
    cfg(&mut p, 0x43, &[0], exact(&[0; 6]));
    p.exchange(PAD1, &pad(0x42, &[], 5), Some(CONNECTED), &pad_reply(0x41, &[0xFF, 0xFF], 0));
    p
}

/// `s25-pressure`, on Sapphire alone, the player holding Right, Cross and R2: analogue mode with every reply byte, the
/// three buttons' bits clear and their pressures FFh in psx-spx's order (D-227), the others 00h.
pub fn pressure_program() -> Program {
    let mut p = Program::uncached("s25-pressure");
    for (cmd, data) in [(0x43u8, &[1u8][..]), (0x44, &[1, 0]), (0x4F, &[0xFF, 0xFF, 0x03]), (0x43, &[0])] {
        p.exchange(PAD1, &pad(cmd, data, 9), None, &any(9));
    }
    // Buttons: Right is bit 5, Cross 14, R2 9, each 0 when pressed.
    let buttons = !(1u16 << 5 | 1 << 14 | 1 << 9);
    let mut pressure = [0u8; 12];
    pressure[0] = 0xFF;
    pressure[6] = 0xFF;
    pressure[11] = 0xFF;
    let want = [header(0x79), exact(&buttons.to_le_bytes()), exact(&[0x80, 0x80, 0x80, 0x80]), exact(&pressure)].concat();
    p.exchange(PAD1, &pad(0x42, &[], 21), Some(CONNECTED), &want);
    p
}

/// A card command's bytes: 81h, the command, its parameters, padded with zeros to `len`.
fn card_cmd(bytes: &[u8], len: usize) -> Vec<u8> {
    let mut v = card(bytes);
    v.resize(len, 0);
    v
}

/// The page's 528 bytes the programs write: a pattern of the page number, the spare bytes included.
pub fn page_data(page: u32) -> Vec<u8> {
    (0..528u32).map(|i| (i.wrapping_mul(13) ^ page.wrapping_mul(29)) as u8).collect()
}

/// Writes `data` (528 bytes) to `page` of the card on `port`: Start Write, four writes of 128 bytes and one of 16, then
/// Read/Write End, each reply graded.
fn write_page(p: &mut Program, port: u32, page: u32, data: &[u8]) {
    let a = page.to_le_bytes();
    let list5 = |p: &mut Program, c: u8| {
        let mut l = vec![None; 5];
        l.extend(ack());
        p.exchange(port, &card_cmd(&[c, a[0], a[1], a[2], a[3]], 9), Some(CONNECTED), &card_reply(&l, 9));
    };
    list5(p, 0x22);
    for (i, size) in [128usize, 128, 128, 128, 16].into_iter().enumerate() {
        let mut c = card(&[0x42, size as u8]);
        c.extend_from_slice(&data[128 * i..128 * i + size]);
        c.resize(134, 0);
        let mut l = ack();
        l.extend(vec![None; 129]);
        l.push(Some(TERMINATOR));
        p.exchange(port, &c, Some(CONNECTED), &card_reply(&l, 134));
    }
    p.exchange(port, &card_cmd(&[0x81], 4), Some(CONNECTED), &card_reply(&ack(), 4));
}

/// Reads `page` of the card on `port`, graded against `want` (528 bytes): Start Read, four reads of 128 bytes with
/// their checksums, one of 16 for the spare bytes, Read/Write End.
fn read_page(p: &mut Program, port: u32, page: u32, want: &[u8]) {
    let a = page.to_le_bytes();
    let mut l = vec![None; 5];
    l.extend(ack());
    p.exchange(port, &card_cmd(&[0x23, a[0], a[1], a[2], a[3]], 9), Some(CONNECTED), &card_reply(&l, 9));
    for (i, size) in [128usize, 128, 128, 128, 16].into_iter().enumerate() {
        let d = &want[128 * i..128 * i + size];
        let mut l = ack();
        l.extend(d.iter().map(|&b| Some(b)));
        l.extend(vec![None; 128 - size]);
        l.push(Some(d.iter().fold(0, |x, b| x ^ b)));
        l.push(Some(TERMINATOR));
        p.exchange(port, &card_cmd(&[0x43, size as u8], 134), Some(CONNECTED), &card_reply(&l, 134));
    }
    p.exchange(port, &card_cmd(&[0x81], 4), Some(CONNECTED), &card_reply(&ack(), 4));
}

/// The flash's block erase, 2 ms typical (Toshiba's TC58128AFT data sheet, tBERASE), in IOP cycles at 36.864 MHz.
pub const ERASE_CYCLES: u32 = 73_728;

/// Erases the block holding `page` on the card on `port`, and times the command after the erase with the IOP's timer 3
/// on the system clock: it waits out the erase, so it ends 2 ms and its own four bytes after it starts, graded within a
/// margin for the program's own instructions.
fn erase(p: &mut Program, port: u32, page: u32) {
    let a = page.to_le_bytes();
    let mut l = vec![None; 5];
    l.extend(ack());
    p.exchange(port, &card_cmd(&[0x21, a[0], a[1], a[2], a[3]], 9), Some(CONNECTED), &card_reply(&l, 9));
    p.exchange(port, &card_cmd(&[0x82], 4), Some(CONNECTED), &card_reply(&ack(), 4));
    p.poke(s21::reg::mode(3), 0);
    p.command(port, &card_cmd(&[0x12], 4));
    p.peek(s21::reg::count(3));
    p.record_within("t2", ERASE_CYCLES, ERASE_CYCLES + 4_000);
    p.recv1(Some(CONNECTED));
    p.reply(&card_reply(&ack(), 4));
}

/// `s25-card`: the card in slot 1 probed, its specs and terminator read and changed, a block erased, a page written with
/// its spare bytes and read back, a page written twice without an erase reading the AND of the two, the block erased
/// again; the authentication commands unanswered (Sapphire_Plan.md §11 Q7(b)); a controller's command on a card port
/// meeting nothing (D-220). ps2tek "SIO2 PS2 Memcards"; Ridge "NAND Flash Memory Basics".
pub fn card_program() -> Program {
    let mut p = Program::uncached("s25-card");
    p.exchange(CARD1, &card_cmd(&[0x11], 4), Some(CONNECTED), &card_reply(&ack(), 4));
    let specs = [ACK, 0x00, 0x02, 0x10, 0x00, 0x00, 0x40, 0x00, 0x00, 0x52, TERMINATOR];
    p.exchange(CARD1, &card_cmd(&[0x26], 13), Some(CONNECTED), &card_reply(&exact(&specs), 13));
    p.exchange(CARD1, &card_cmd(&[0x28], 5), Some(CONNECTED), &card_reply(&exact(&[ACK, TERMINATOR, 0x55]), 5));
    p.exchange(CARD1, &card_cmd(&[0x27, 0xAA], 5), Some(CONNECTED), &card_reply(&[None, Some(ACK), Some(TERMINATOR)], 5));
    p.exchange(CARD1, &card_cmd(&[0x28], 5), Some(CONNECTED), &card_reply(&exact(&[ACK, 0xAA, 0x55]), 5));
    p.exchange(CARD1, &card_cmd(&[0x11], 4), Some(CONNECTED), &card_reply(&exact(&[ACK, 0xAA]), 4));
    p.exchange(CARD1, &card_cmd(&[0x27, TERMINATOR], 5), Some(CONNECTED), &card_reply(&[None, Some(ACK), Some(0xAA)], 5));
    p.exchange(CARD1, &card_cmd(&[0xBF], 5), Some(CONNECTED), &card_reply(&[None, Some(ACK), Some(TERMINATOR)], 5));
    p.exchange(CARD1, &card_cmd(&[0x12], 4), Some(CONNECTED), &card_reply(&ack(), 4));
    erase(&mut p, CARD1, 32);
    let data = page_data(32);
    write_page(&mut p, CARD1, 32, &data);
    read_page(&mut p, CARD1, 32, &data);
    write_page(&mut p, CARD1, 33, &[0x0F; 528]);
    write_page(&mut p, CARD1, 33, &[0xF0; 528]);
    read_page(&mut p, CARD1, 33, &[0x00; 528]);
    read_page(&mut p, CARD1, 34, &[0xFF; 528]);
    erase(&mut p, CARD1, 47);
    read_page(&mut p, CARD1, 32, &[0xFF; 528]);
    read_page(&mut p, CARD1, 33, &[0xFF; 528]);
    // The card in slot 2 answers on its own port.
    p.exchange(CARD2, &card_cmd(&[0x11], 4), Some(CONNECTED), &card_reply(&ack(), 4));
    p.exchange(CARD2, &pad(0x42, &[], 5), Some(NO_PAD), &exact(&[HIZ; 5]));
    for c in [0xF3u8, 0xF7, 0xF0] {
        p.exchange(CARD1, &card_cmd(&[c], 5), Some(NO_DEVICE), &exact(&[HIZ; 5]));
    }
    p
}

/// IOP RAM the DMA program uses for its buffers.
pub const DMA_IN: u32 = 0x001A_0000;
pub const DMA_OUT: u32 = 0x001A_1000;

/// `s25-dma`: a queue of two commands, a controller read on port 0 and a card probe on port 2, its bytes sent by SIO2in
/// (channel 11) and its replies taken by SIO2out (channel 12), both in slice mode (ps2tek "IOP DMA", "SIO2 Registers":
/// "The SIO2in DMA channel also writes to this register"); the replies equal those sent by FIFOIN, each channel's
/// completion flag set.
pub fn dma_program() -> Program {
    use s21::reg::*;
    let mut p = Program::uncached("s25-dma");
    let bytes: Vec<u8> = [pad(0x42, &[], 8), card_cmd(&[0x11], 4)].concat();
    for (i, w) in bytes.chunks(4).enumerate() {
        p.poke(s21::K1 | (DMA_IN + 4 * i as u32), u32::from_le_bytes([w[0], w[1], w[2], w[3]]));
    }
    for i in 0..4 {
        p.poke(s21::K1 | (DMA_OUT + 4 * i), 0xDEAD_BEEF);
    }
    p.queue(&[send3(PAD1, 8), send3(CARD1, 4)]);
    p.poke(DPCR2, 8 << 16 | 8 << 20);
    p.poke(madr(11), DMA_IN);
    p.poke(bcr(11), 3 | 1 << 16);
    p.poke(chcr(11), 0x0100_0201);
    p.poke(madr(12), DMA_OUT);
    p.poke(bcr(12), 3 | 1 << 16);
    p.poke(chcr(12), 0x0100_0200);
    p.delay(200);
    p.start();
    p.wait(chcr(12), 0x0100_0000, false, 20_000);
    p.rec(chcr(11), Some(0x0000_0201));
    p.rec(chcr(12), Some(0x0000_0200));
    p.recv1(Some(CONNECTED));
    p.rec(s21::K1 | DMA_OUT, Some(u32::from_le_bytes([HIZ, 0x41, 0x5A, 0xFF])));
    p.rec(s21::K1 | (DMA_OUT + 4), None);
    p.rec(s21::K1 | (DMA_OUT + 8), Some(u32::from_le_bytes([HIZ, HIZ, ACK, TERMINATOR])));
    p.rec(s21::K1 | (DMA_OUT + 12), Some(0xDEAD_BEEF));
    p
}

/// `s25-tap`, with a multitap in slot 1 holding a DualShock 2 in its slot A: its frame on the controller port and the
/// card port, and its selection of an empty slot leaving the controller port with nothing (the one witness's frame and
/// Sapphire's choice of selection, D-228; no document gives either).
pub fn tap_program() -> Program {
    let mut p = Program::uncached("s25-tap");
    let four = exact(&[HIZ, 0x80, 0x5A, 0x04, 0x00, 0x5A, HIZ]);
    let sel = exact(&[HIZ, 0x80, 0x5A, 0x00, 0x00, 0x00, 0x5A]);
    for port in [PAD1, CARD1] {
        p.exchange(port, &[0x21, 0x12, 0, 0, 0, 0, 0], Some(CONNECTED), &four);
        p.exchange(port, &[0x21, 0x13, 0, 0, 0, 0, 0], Some(CONNECTED), &four);
    }
    p.exchange(PAD1, &[0x21, 0x21, 0, 0, 0, 0, 0], Some(CONNECTED), &sel);
    p.exchange(PAD1, &pad(0x42, &[], 5), Some(CONNECTED), &pad_reply(0x41, &[0xFF, 0xFF], 0));
    p.exchange(PAD1, &[0x21, 0x21, 0, 1, 0, 0, 0], Some(CONNECTED), &sel);
    p.exchange(PAD1, &pad(0x42, &[], 5), Some(NO_PAD), &exact(&[HIZ; 5]));
    p.exchange(PAD1, &[0x21, 0x21, 0, 0, 0, 0, 0], Some(CONNECTED), &sel);
    p.exchange(PAD1, &pad(0x42, &[], 5), Some(CONNECTED), &pad_reply(0x41, &[0xFF, 0xFF], 0));
    p.exchange(CARD1, &[0x21, 0x22, 0, 0, 0, 0, 0], Some(CONNECTED), &sel);
    p.exchange(CARD1, &card_cmd(&[0x11], 4), Some(CONNECTED), &card_reply(&ack(), 4));
    p.exchange(PAD1, &[0x21, 0x41, 0, 0, 0, 0, 0], Some(NO_DEVICE), &exact(&[HIZ; 7]));
    p
}

/// A probe for the referee, ungraded: what each port answers to each documented command, eight bytes more read after
/// each reply to see where it ends, and the status registers.
pub fn probe() -> Program {
    let mut p = Program::uncached("s25-probe");
    let x = |p: &mut Program, port: u32, b: &[u8]| {
        p.exchange(port, b, None, &any(b.len()));
        p.reply(&any(8));
    };
    for port in 0..4 {
        x(&mut p, port, &pad(0x42, &[], 5));
        x(&mut p, port, &card(&[0x11, 0]));
        p.rec(reg::RECV2, None);
        p.rec(reg::RECV3, None);
        p.rec(reg::ISTAT, None);
    }
    // The DualShock 2's configuration on port 1.
    let seq: [(u8, &[u8], usize); 16] = [
        (0x43, &[1], 9),
        (0x45, &[], 9),
        (0x41, &[], 9),
        (0x46, &[0], 9),
        (0x46, &[1], 9),
        (0x47, &[], 9),
        (0x4C, &[0], 9),
        (0x4C, &[1], 9),
        (0x48, &[0], 9),
        (0x44, &[1, 3], 9),
        (0x4F, &[0xFF, 0xFF, 0x03], 9),
        (0x41, &[], 9),
        (0x4D, &[0, 1, 0xFF, 0xFF, 0xFF, 0xFF], 9),
        (0x40, &[0, 2], 9),
        (0x42, &[], 9),
        (0x43, &[0], 9),
    ];
    for (cmd, data, len) in seq {
        x(&mut p, PAD1, &pad(cmd, data, len));
    }
    x(&mut p, PAD1, &pad(0x42, &[], 21));
    x(&mut p, PAD1, &pad(0x43, &[], 21));
    x(&mut p, PAD1, &pad(0x42, &[0x01, 0x80], 21));
    // The card in slot 1 on both ports a card might answer: its specs, terminator, and page 0 with its spare bytes.
    for port in [CARD1, PAD1] {
        for c in [card(&[0x26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), card(&[0x28, 0, 0]), card(&[0xBF, 0, 0]), card(&[0x12, 0]), card(&[0xF3, 0, 0])] {
            x(&mut p, port, &c);
        }
    }
    for page in [0u32, 16, 18, 82] {
        let a = page.to_le_bytes();
        x(&mut p, CARD1, &card(&[0x23, a[0], a[1], a[2], a[3], 0, 0]));
        for size in [128u8, 128, 128, 128, 16] {
            let mut c = card(&[0x43, size]);
            c.resize(133, 0);
            x(&mut p, CARD1, &c);
        }
        x(&mut p, CARD1, &card(&[0x81, 0]));
    }
    p
}

/// A second probe for the referee, ungraded: a block erased, pages written with their spare bytes and read back, and
/// a page written twice without an erase, on the card in each slot.
pub fn probe_write() -> Program {
    let mut p = Program::uncached("s25-probe-write");
    let x = |p: &mut Program, port: u32, b: &[u8]| {
        p.exchange(port, b, None, &any(b.len()));
        p.reply(&any(8));
    };
    for port in [CARD1, CARD2] {
        let addr = |page: u32| page.to_le_bytes();
        let a = addr(16);
        x(&mut p, port, &card(&[0x12, 0, 0]));
        x(&mut p, port, &card(&[0x21, a[0], a[1], a[2], a[3], 0, 0, 0]));
        x(&mut p, port, &card(&[0x82, 0, 0]));
        x(&mut p, port, &card(&[0x12, 0, 0]));
        for (page, seed) in [(16u32, 3u8), (17, 0x0F), (17, 0xF0)] {
            let a = addr(page);
            x(&mut p, port, &card(&[0x22, a[0], a[1], a[2], a[3], 0, 0, 0]));
            for chunk in 0..5u8 {
                let size = if chunk < 4 { 128 } else { 16 };
                let mut c = card(&[0x42, size]);
                for i in 0..128u32 {
                    c.push(match seed {
                        0x0F | 0xF0 => seed,
                        _ => (u32::from(chunk) * 128 + i).wrapping_mul(7).wrapping_add(u32::from(seed)) as u8,
                    });
                }
                c.resize(134, 0);
                x(&mut p, port, &c);
            }
            x(&mut p, port, &card(&[0x81, 0, 0]));
            x(&mut p, port, &card(&[0x12, 0, 0]));
        }
        for page in [16u32, 17, 18] {
            let a = addr(page);
            x(&mut p, port, &card(&[0x23, a[0], a[1], a[2], a[3], 0, 0, 0]));
            for size in [128u8, 128, 128, 128, 16] {
                let mut c = card(&[0x43, size]);
                c.resize(134, 0);
                x(&mut p, port, &c);
            }
            x(&mut p, port, &card(&[0x81, 0, 0]));
        }
    }
    p
}

/// A third probe for the referee, ungraded: what a multitap answers to peripheral byte `peripheral` with each command
/// byte, on port `port`.
pub fn probe_tap(name: &'static str, port: u32, peripheral: u8) -> Program {
    let mut p = Program::uncached(name);
    for cmd in (0x00..=0x2Fu8).chain([0x41, 0x42, 0x43, 0x81, 0xFF]) {
        let b = [peripheral, cmd, 0, 0, 0, 0, 0];
        p.exchange(port, &b, None, &any(b.len()));
        p.reply(&any(8));
    }
    let b = pad(0x42, &[], 5);
    p.exchange(PAD1, &b, None, &any(b.len()));
    p
}

/// A fourth probe for the referee, ungraded: a multitap's slot selection and what each slot then answers.
pub fn probe_tap_slots() -> Program {
    let mut p = Program::uncached("s25-tap-slots");
    let x = |p: &mut Program, port: u32, b: &[u8]| {
        p.exchange(port, b, None, &any(b.len()));
        p.reply(&any(8));
    };
    for (cmd, port) in [(0x21u8, PAD1), (0x22, CARD1), (0x21, CARD1), (0x22, PAD1)] {
        for slot in [1u8, 0, 3, 4, 0xFF, 0] {
            x(&mut p, port, &[0x21, cmd, 0, slot, 0, 0, 0]);
            x(&mut p, PAD1, &pad(0x42, &[], 5));
            x(&mut p, CARD1, &card(&[0x11, 0, 0]));
        }
    }
    x(&mut p, PAD1, &[0x21, 0x21, 0, 0, 1, 0, 0]);
    x(&mut p, PAD1, &pad(0x42, &[], 5));
    x(&mut p, PAD1, &[0x21, 0x21, 0, 0, 0, 0, 0]);
    x(&mut p, PAD1, &[0x21, 0x12, 0, 0]);
    x(&mut p, PAD1, &[0x21, 0x12, 0, 0, 0, 0, 0, 0, 0]);
    p
}

/// The gate's programs that every referee configuration runs.
pub fn gate() -> Vec<Program> {
    vec![pad_program(), card_program(), dma_program()]
}

/// Every program the referees are given: the gate's, the tap's (run with a multitap), and the probes.
pub fn all() -> Vec<Program> {
    let mut v = gate();
    v.push(tap_program());
    v.extend([probe(), probe_write(), probe_tap("s25-tap-21-0", PAD1, 0x21), probe_tap("s25-tap-21-2", CARD1, 0x21), probe_tap("s25-tap-61-0", PAD1, 0x61), probe_tap_slots()]);
    v
}
