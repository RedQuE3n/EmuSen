//! The project's EE loader for IOP modules (Sapphire_Native.md §40): an EE program, entered as a program is on the
//! player's BIOS, that speaks the SIF's RPC to the IOP's module loader (LOADFILE, server 80000006h) in the layouts
//! ps2tek's "SIF RPC Structs and Definitions" and "SIF RPC Commands" give, and asks it to load and start modules, each
//! call's argument block given here as bytes. Its packets cross on the SIF's DMA channels as S22 models them: the EE's
//! channel 6 in normal mode, each block led by the IOP's chain tag in its first quadword (D-143), and the IOP's replies
//! received by the EE's channel 5 in chain mode, armed by the kernel's SifSetDChain (ps2tek "BIOS EE Syscalls", 78h).
//! It keeps a record of 32-bit words from `RECORD`, every packet received from `LOG`, and writes `DONE_WORD` at
//! `DONE_AT` when its calls have ended, for a referee's RAM or Sapphire's.

#![allow(dead_code)]

/// Where the loader is placed and entered.
pub const ORIGIN: u32 = 0x0010_0000;
/// The EE's receive buffer for the IOP's command packets, given to the IOP by SIFCMD's init packet.
pub const RECV: u32 = 0x0018_0000;
/// Where each SIF1 transfer is laid out, its tags and packets.
pub const STAGE: u32 = 0x0018_1000;
/// The client's place, a word the packets name and the IOP echoes.
pub const CLIENT: u32 = 0x0018_0800;
/// Each call's reply buffer, `REPLY_STRIDE` bytes apart, filled with `FILL` before the call.
pub const REPLY: u32 = 0x0018_4000;
pub const REPLY_STRIDE: u32 = 0x400;
pub const FILL: u32 = 0xA5A5_A5A5;
/// The record, the log of received packets (16 words each), the done word.
pub const RECORD: u32 = 0x0019_0000;
pub const LOG: u32 = 0x0019_8000;
pub const LOG_BYTES: u32 = 0x8000;
pub const DONE_AT: u32 = 0x001F_FFF0;
pub const DONE_WORD: u32 = 0x5A99_D00E;

/// The record's words, by byte offset from `RECORD`.
pub mod at {
    /// The furthest step reached (`step`).
    pub const STEP: u32 = 0x00;
    /// SMCOM, the IOP's receive address for SIF1, and SMFLG when the loader began.
    pub const SMCOM: u32 = 0x04;
    pub const SMFLG: u32 = 0x08;
    /// SMFLG once the first init packet was answered, and the packets received.
    pub const SMFLG_INIT: u32 = 0x0C;
    pub const PACKETS: u32 = 0x10;
    /// Binds sent before the server answered.
    pub const BINDS: u32 = 0x14;
    /// Calls made, and each call's outcome (`OUTCOME_*`) from `CALL_OUTCOMES`, a word each, at most `MAX_CALLS`.
    pub const CALLS: u32 = 0x18;
    pub const CALL_OUTCOMES: u32 = 0x3F00;
    /// The bind's end packet, 16 words.
    pub const BIND_END: u32 = 0x40;
    /// The last packet received, 16 words.
    pub const LAST: u32 = 0x80;
    /// Call i's end packet (16 words) at `CALL + i * CALL_STRIDE`, and the first `REPLY_WORDS` words of its reply
    /// buffer after it.
    pub const CALL: u32 = 0x100;
    pub const CALL_STRIDE: u32 = 0x100;
    pub const REPLY_WORDS: u32 = 48;
    pub const MAX_CALLS: usize = 62;
}

/// The steps the record's first word names.
pub mod step {
    pub const STARTED: u32 = 1;
    pub const IOP_BOOTED: u32 = 2;
    pub const INIT_ANSWERED: u32 = 3;
    pub const BOUND: u32 = 4;
    pub const CALLED: u32 = 5;
}

/// A call's outcome: its end packet arrived, or none did in the time allowed.
pub const OUTCOME_ENDED: u32 = 1;
pub const OUTCOME_TIMEOUT: u32 = 2;

/// The SIF's registers (ps2tek "SIF Registers"), through the EE's user mapping, and the DMAC's (EE User's Manual §2.2.1).
const SIF: u32 = 0x1000_F200;
const SMCOM: i32 = 0x10;
const SMFLG: i32 = 0x30;
const CH5: u32 = 0x1000_C000;
const CH6: u32 = 0x1000_C400;
const D_STAT: u32 = 0x1000_E010;
/// The uncached view of RAM a program's buffers are reached through, so that what the DMAC moves is what is read.
const UNCACHED: u32 = 0x2000_0000;

/// SMFLG's bits (ps2tek "SIF Registers"): SIFCMD initialised, and the IOP finished booting.
const SIFCMD_UP: u32 = 0x2_0000;
const IOP_UP: u32 = 0x4_0000;

/// The SIF commands (ps2tek "SIF RPC Commands") and the loader's server and function (ps2tek "BIOS IOP LOADFILE").
pub const CMD_INIT: u32 = 0x8000_0002;
pub const CMD_END: u32 = 0x8000_0008;
pub const CMD_BIND: u32 = 0x8000_0009;
pub const CMD_CALL: u32 = 0x8000_000A;
pub const LOADFILE: u32 = 0x8000_0006;
pub const LOAD_MODULE: u32 = 0;

/// The IOP's chain tag's bits (ps2tek "IOP DMA"): interrupt when the packet has moved, and the chain's end.
const TAG_IRQ: u32 = 1 << 30;
const TAG_END: u32 = 1 << 31;

/// SifSetDChain's number (ps2tek "BIOS EE Syscalls").
const SIF_SET_DCHAIN: u32 = 0x78;

/// Spins of a wait: SMFLG's flags, a bind's answer, a call's answer (which runs the module to its end).
const SPIN_FLAG: u32 = 0x0080_0000;
const SPIN_BIND: u32 = 0x0040_0000;
const SPIN_CALL: u32 = 0x4000_0000;
const BIND_TRIES: u32 = 64;

/// A call to the loader's function: the argument block sent (padded to quadwords) and the bytes asked back.
#[derive(Clone, Debug)]
pub struct Call {
    pub function: u32,
    pub args: Vec<u8>,
    pub recv: u32,
}

struct Asm {
    lines: Vec<String>,
    labels: u32,
}

impl Asm {
    fn op(&mut self, s: impl Into<String>) {
        self.lines.push(format!("        {}", s.into()));
    }
    fn label(&mut self, l: &str) {
        self.lines.push(format!("{l}:"));
    }
    fn fresh(&mut self, stem: &str) -> String {
        self.labels += 1;
        format!("{stem}{}", self.labels)
    }
    /// A register loaded with a 32-bit value, always in two instructions so that the program's length does not depend on it.
    fn li(&mut self, r: &str, v: u32) {
        self.op(format!("lui {r}, 0x{:04x}", v >> 16));
        self.op(format!("ori {r}, {r}, 0x{:04x}", v & 0xFFFF));
    }
    /// A word of `RECORD` set to a register.
    fn record(&mut self, r: &str, offset: u32) {
        self.op(format!("sw {r}, {offset}(s7)"));
    }
    fn record_value(&mut self, v: u32, offset: u32) {
        self.li("t0", v);
        self.record("t0", offset);
    }
    /// `words` words copied from `from` to `to` (registers left changed: t0, t1, t2, t3).
    fn copy(&mut self, from: u32, to: u32, words: u32) {
        if words == 0 {
            return;
        }
        let l = self.fresh("copy");
        self.li("t1", from);
        self.li("t2", to);
        self.li("t3", words);
        self.label(&l);
        self.op("lw t0, 0(t1)");
        self.op("addiu t1, t1, 4");
        self.op("sw t0, 0(t2)");
        self.op("addiu t3, t3, -1");
        self.op(format!("bne t3, zero, {l}"));
        self.op("addiu t2, t2, 4");
    }
    /// `words` words at `to` set to `v`.
    fn fill(&mut self, to: u32, words: u32, v: u32) {
        let l = self.fresh("fill");
        self.li("t0", v);
        self.li("t2", to);
        self.li("t3", words);
        self.label(&l);
        self.op("sw t0, 0(t2)");
        self.op("addiu t3, t3, -1");
        self.op(format!("bne t3, zero, {l}"));
        self.op("addiu t2, t2, 4");
    }
    /// Waits, at most `spins` times round, for SMFLG to hold `bit`; t0, t1 and t9 changed.
    fn wait_flag(&mut self, bit: u32, spins: u32) {
        let (l, done) = (self.fresh("flag"), self.fresh("flagged"));
        self.li("t9", spins);
        self.li("t1", bit);
        self.label(&l);
        self.op(format!("lw t0, {SMFLG}(s6)"));
        self.op("and t0, t0, t1");
        self.op(format!("bne t0, zero, {done}"));
        self.op("addiu t9, t9, -1");
        self.op(format!("bne t9, zero, {l}"));
        self.op("nop");
        self.label(&done);
    }
    /// One block of a SIF1 transfer at `STAGE + offset`: the IOP's tag quadword (its address in t-register `dest`,
    /// ORed with `flags`), then the words given by `body`, which writes them from `STAGE + offset + 16`; the offset past
    /// the block.
    fn tag(&mut self, offset: u32, dest: &str, flags: u32, words: u32) {
        self.li("t1", flags);
        self.op(format!("or t1, t1, {dest}"));
        self.li("t2", UNCACHED | (STAGE + offset));
        self.op("sw t1, 0(t2)");
        self.li("t1", words.next_multiple_of(4));
        self.op("sw t1, 4(t2)");
        self.op("sw zero, 8(t2)");
        self.op("sw zero, 12(t2)");
    }
    /// A packet's words written from `STAGE + offset`, each a value or a register.
    fn packet(&mut self, offset: u32, words: &[Word]) {
        self.li("t2", UNCACHED | (STAGE + offset));
        for (i, w) in words.iter().enumerate() {
            match w {
                Word::V(v) => {
                    self.li("t1", *v);
                    self.op(format!("sw t1, {}(t2)", 4 * i));
                }
                Word::R(r) => self.op(format!("sw {r}, {}(t2)", 4 * i)),
            }
        }
    }
    /// The transfer laid out at `STAGE`, `qwords` quadwords, sent on channel 6 and waited for.
    fn send(&mut self, qwords: u32) {
        self.li("a0", STAGE);
        self.li("a1", qwords);
        self.op("jal send");
        self.op("nop");
    }
    /// Waits for an end packet whose command field is `cid`, at most `spins` times round each time; v0 is 1 when it
    /// came, the packet at `RECORD + at::LAST`.
    fn wait_end(&mut self, cid: u32, spins: u32) {
        self.li("a0", cid);
        self.li("a1", spins);
        self.op("jal wait");
        self.op("nop");
    }
}

enum Word {
    V(u32),
    R(&'static str),
}

/// The loader's source: the SIF's RPC set up, the loader's server bound, and each call made in turn.
pub fn source(calls: &[Call]) -> String {
    let first = body(calls, 0);
    let code = first.lines.iter().filter(|l| !l.ends_with(':')).count() as u32;
    let data = ORIGIN + 4 * code;
    let mut asm = body(calls, data);
    let mut at = data;
    for c in calls {
        for chunk in padded(&c.args).chunks(4) {
            asm.op(format!(".word 0x{:08x}", u32::from_le_bytes(chunk.try_into().unwrap())));
            at += 4;
        }
    }
    let _ = at;
    asm.lines.join("\n") + "\n"
}

/// The program image, assembled for the R5900.
pub fn image(calls: &[Call]) -> Vec<u32> {
    emusen_mips::assemble(&source(calls), ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("the loader: {e:?}"))
}

/// The argument block padded with zeros to whole quadwords.
pub fn padded(args: &[u8]) -> Vec<u8> {
    let mut v = args.to_vec();
    v.resize(args.len().next_multiple_of(16).max(16), 0);
    v
}

fn body(calls: &[Call], data: u32) -> Asm {
    assert!(calls.len() <= at::MAX_CALLS, "at most {} calls", at::MAX_CALLS);
    let mut a = Asm { lines: Vec::new(), labels: 0 };
    a.li("s7", UNCACHED | RECORD);
    a.li("s6", SIF);
    a.li("s5", CH5);
    a.li("s4", CH6);
    a.li("s3", UNCACHED | LOG);
    a.fill(UNCACHED | RECORD, 0x2000, 0);
    a.fill(UNCACHED | LOG, LOG_BYTES / 4, 0);
    a.fill(UNCACHED | RECV, 64, 0);
    a.record_value(step::STARTED, at::STEP);
    a.op(format!("lw t0, {SMFLG}(s6)"));
    a.record("t0", at::SMFLG);
    // The IOP's modules up, as EESYNC says (ps2tek "BIOS IOP EESYNC").
    a.wait_flag(IOP_UP, SPIN_FLAG * 8);
    a.record_value(step::IOP_BOOTED, at::STEP);
    a.op(format!("lw s2, {SMCOM}(s6)"));
    a.record("s2", at::SMCOM);
    // SIF0 armed for the IOP's packets, SIFCMD's flag cleared, and the first init packet naming the receive buffer.
    a.op(format!("addiu v1, zero, 0x{SIF_SET_DCHAIN:x}"));
    a.op("syscall 0");
    a.op("nop");
    a.li("t0", SIFCMD_UP);
    a.op(format!("sw t0, {SMFLG}(s6)"));
    a.tag(0, "s2", TAG_IRQ | TAG_END, 8);
    a.packet(16, &[Word::V(0x14), Word::V(0), Word::V(CMD_INIT), Word::V(0), Word::V(RECV), Word::V(0), Word::V(0), Word::V(0)]);
    a.send(3);
    a.wait_flag(SIFCMD_UP, SPIN_FLAG);
    a.op(format!("lw t0, {SMFLG}(s6)"));
    a.record("t0", at::SMFLG_INIT);
    a.record_value(step::INIT_ANSWERED, at::STEP);
    a.tag(0, "s2", TAG_IRQ | TAG_END, 4);
    a.packet(16, &[Word::V(0x10), Word::V(0), Word::V(CMD_INIT), Word::V(1)]);
    a.send(2);
    // Bound to LOADFILE: binds sent until the end packet names a server.
    let (bind, bound, fail) = (a.fresh("bind"), a.fresh("bound"), a.fresh("unbound"));
    a.op("addiu s1, zero, 0");
    a.label(&bind);
    a.op("addiu s1, s1, 1");
    a.record("s1", at::BINDS);
    a.tag(0, "s2", TAG_IRQ | TAG_END, 16);
    a.packet(
        16,
        &[
            Word::V(0x40),
            Word::V(0),
            Word::V(CMD_BIND),
            Word::V(0),
            Word::V(0),
            Word::V(STAGE),
            Word::R("s1"),
            Word::V(CLIENT),
            Word::V(LOADFILE),
            Word::V(0),
            Word::V(0),
            Word::V(0),
            Word::V(0),
            Word::V(0),
            Word::V(0),
            Word::V(0),
        ],
    );
    a.send(5);
    a.wait_end(CMD_BIND, SPIN_BIND);
    a.op(format!("lw s0, {}(s7)", at::LAST + 4 * 9));
    a.op(format!("bne s0, zero, {bound}"));
    a.op("nop");
    a.li("t9", SPIN_BIND);
    let pause = a.fresh("pause");
    a.label(&pause);
    a.op("addiu t9, t9, -1");
    a.op(format!("bne t9, zero, {pause}"));
    a.op("nop");
    a.op(format!("addiu t0, s1, -{BIND_TRIES}"));
    a.op(format!("bne t0, zero, {bind}"));
    a.op("nop");
    a.op(format!("beq zero, zero, {fail}"));
    a.op("nop");
    a.label(&bound);
    a.copy(UNCACHED | (RECORD + at::LAST), UNCACHED | (RECORD + at::BIND_END), 16);
    a.record_value(step::BOUND, at::STEP);
    // s0 the server, s1 its receive buffer.
    a.op(format!("lw s1, {}(s7)", at::LAST + 4 * 10));
    let mut data_at = data;
    for (i, c) in calls.iter().enumerate() {
        let args = padded(&c.args);
        let words = args.len() as u32 / 4;
        let reply = REPLY + i as u32 * REPLY_STRIDE;
        a.fill(UNCACHED | reply, REPLY_STRIDE / 4, FILL);
        a.tag(0, "s1", 0, words);
        a.copy(data_at, UNCACHED | (STAGE + 16), words);
        let p = 16 + 4 * words;
        a.tag(p, "s2", TAG_IRQ | TAG_END, 16);
        a.packet(
            p + 16,
            &[
                Word::V(0x40 | (4 * words) << 8),
                Word::R("s1"),
                Word::V(CMD_CALL),
                Word::V(0),
                Word::V(0),
                Word::V(STAGE),
                Word::V(i as u32),
                Word::V(CLIENT),
                Word::V(c.function),
                Word::V(4 * words),
                Word::V(reply),
                Word::V(c.recv),
                Word::V(1),
                Word::R("s0"),
                Word::V(0),
                Word::V(0),
            ],
        );
        a.send(1 + words / 4 + 1 + 4);
        a.wait_end(CMD_CALL, SPIN_CALL);
        let (ended, next) = (a.fresh("ended"), a.fresh("next"));
        a.op(format!("bne v0, zero, {ended}"));
        a.op("nop");
        a.record_value(OUTCOME_TIMEOUT, at::CALL_OUTCOMES + 4 * i as u32);
        a.op(format!("beq zero, zero, {next}"));
        a.op("nop");
        a.label(&ended);
        a.record_value(OUTCOME_ENDED, at::CALL_OUTCOMES + 4 * i as u32);
        a.label(&next);
        let slot = RECORD + at::CALL + i as u32 * at::CALL_STRIDE;
        a.copy(UNCACHED | (RECORD + at::LAST), UNCACHED | slot, 16);
        a.copy(UNCACHED | reply, UNCACHED | (slot + 0x40), at::REPLY_WORDS);
        a.record_value(i as u32 + 1, at::CALLS);
        data_at += args.len() as u32;
    }
    a.record_value(step::CALLED, at::STEP);
    a.label(&fail);
    a.li("t1", UNCACHED | DONE_AT);
    a.li("t0", DONE_WORD);
    a.op("sw t0, 0(t1)");
    a.label("end");
    a.op("beq zero, zero, end");
    a.op("nop");
    subroutines(&mut a);
    a
}

/// `send` (a0 the transfer's address, a1 its quadwords) and `wait` (a0 the command field wanted in an end packet, a1
/// the spins).
fn subroutines(a: &mut Asm) {
    // SIF1 in normal mode from memory (EE User's Manual §5.6: DIR 1, MOD 0, STR).
    a.label("send");
    a.op("sw a0, 16(s4)");
    a.op("sw a1, 32(s4)");
    a.op("addiu t0, zero, 0x101");
    a.op("sw t0, 0(s4)");
    a.li("t9", SPIN_FLAG);
    a.label("send_wait");
    a.op("lw t0, 0(s4)");
    a.op("andi t0, t0, 0x100");
    a.op("beq t0, zero, send_done");
    a.op("addiu t9, t9, -1");
    a.op("bne t9, zero, send_wait");
    a.op("nop");
    a.label("send_done");
    a.op("jr ra");
    a.op("nop");
    // A packet is in when the receive buffer's command field is set; the channel has stopped at the packet's tag
    // interrupt (TIE), so the packet is copied out, the buffer cleared, CIS5 acknowledged and the channel armed again.
    a.label("wait");
    a.op("sll fp, a0, 0");
    a.op("sll gp, ra, 0");
    a.op("sll t8, a1, 0");
    a.label("wait_poll");
    a.li("t1", UNCACHED | RECV);
    a.op("lw t0, 8(t1)");
    a.op("bne t0, zero, wait_in");
    a.op("addiu t8, t8, -1");
    a.op("bne t8, zero, wait_poll");
    a.op("nop");
    a.op("addiu v0, zero, 0");
    a.op("jr gp");
    a.op("nop");
    a.label("wait_in");
    a.li("t9", SPIN_FLAG);
    a.label("wait_stop");
    a.op("lw t0, 0(s5)");
    a.op("andi t0, t0, 0x100");
    a.op("beq t0, zero, wait_stopped");
    a.op("addiu t9, t9, -1");
    a.op("bne t9, zero, wait_stop");
    a.op("nop");
    a.label("wait_stopped");
    a.li("t1", UNCACHED | RECV);
    a.li("t2", UNCACHED | (RECORD + at::LAST));
    for k in 0..16 {
        a.op(format!("lw t0, {}(t1)", 4 * k));
        a.op(format!("sw t0, {}(t2)", 4 * k));
        a.op(format!("sw t0, {}(s3)", 4 * k));
        a.op(format!("sw zero, {}(t1)", 4 * k));
    }
    a.li("t0", LOG_BYTES - 64);
    a.li("t1", UNCACHED | LOG);
    a.op("subu t1, s3, t1");
    a.op("sltu t0, t1, t0");
    a.op("beq t0, zero, wait_full");
    a.op("nop");
    a.op("addiu s3, s3, 64");
    a.label("wait_full");
    a.op(format!("lw t0, {}(s7)", at::PACKETS));
    a.op("addiu t0, t0, 1");
    a.op(format!("sw t0, {}(s7)", at::PACKETS));
    a.li("t1", D_STAT);
    a.op("addiu t0, zero, 0x20");
    a.op("sw t0, 0(t1)");
    a.op("lw t0, 0(s5)");
    a.op("andi t0, t0, 0x100");
    a.op("bne t0, zero, wait_check");
    a.op("nop");
    a.op(format!("addiu v1, zero, 0x{SIF_SET_DCHAIN:x}"));
    a.op("syscall 0");
    a.op("nop");
    a.label("wait_check");
    a.op(format!("lw t0, {}(s7)", at::LAST + 8));
    a.li("t1", CMD_END);
    a.op("bne t0, t1, wait_poll");
    a.op("nop");
    a.op(format!("lw t0, {}(s7)", at::LAST + 4 * 8));
    a.op("bne t0, fp, wait_poll");
    a.op("nop");
    a.op("addiu v0, zero, 1");
    a.op("jr gp");
    a.op("nop");
}
