//! Diamond's own state format, "DMND" (Diamond_Plan.md §4.7, §5.10): the magic, the version, the frame count, the
//! R3000A's and the GTE's blocks, the bus with its clock and every device's registers, then every writable memory and
//! the TTY. The layout listing is pinned by a test.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{State, StateReader, StateWriter, Truncated};

use crate::bus::Board;
use crate::gpu::raster::Vertex;
use crate::gpu::vram::Rect;
use crate::gpu::{Gpu, Parser, Polyline, ReadOut, Transfer, Word};
use crate::machine::Machine;
use crate::sio0::{Phase, Sio0};

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"DMND");
pub const STATE_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

impl Board {
    fn write_state(&self, w: &mut StateWriter) {
        w.u64("Now", self.now);
        w.u64s("Queue", &self.queue);
        w.u64("BusFree", self.bus_free);
        w.u64("CpuBusy", self.cpu_busy);
        w.u8("Post", self.post);
        w.u32s("MemControl", &self.memctl.regs);
        w.u32("RamSize", self.memctl.ram_size);
        w.u32("Bcc", self.cache.bcc);
        w.u32s("Tags", &self.cache.tags);
        w.u32s("ICache", &self.cache.words);
        w.u32("IStat", self.irq.stat);
        w.u32("IMask", self.irq.mask);
        for (n, t) in self.timers.t.iter().enumerate() {
            w.group(["Timer0", "Timer1", "Timer2"][n], |w| {
                w.u32("Count", t.count);
                w.u32("Target", t.target);
                w.u32("Mode", t.mode);
                w.u64("Ticks", t.ticks);
                w.u64("Hold", t.hold);
                w.bools("Flags", &[t.wrap, t.wrap_hold, t.seen_blank, t.fired, t.running]);
            });
        }
        let d = &self.dma;
        w.group("Dma", |w| {
            w.u32s("Madr", &d.ch.map(|c| c.madr));
            w.u32s("Bcr", &d.ch.map(|c| c.bcr));
            w.u32s("Chcr", &d.ch.map(|c| c.chcr));
            w.u32("Dpcr", d.dpcr);
            w.u32("Dicr", d.dicr);
            w.i32("Active", d.active.map_or(-1, |n| n as i32));
            w.u64("Due", d.due);
            w.u32("Left", d.left);
            w.bool("Finishing", d.finishing);
        });
        let v = &self.video;
        w.group("Video", |w| {
            w.u32s("Ranges", &[v.mode, v.x1, v.x2, v.y1, v.y2, v.field_lines]);
            w.u64s("Bases", &[v.base_h, v.base_line, v.base_dots, v.base_hblanks, v.field_line, v.done_h]);
            w.bool("Odd", v.odd);
        });
        w.group("Gpu", |w| self.gpu.write_state(w));
        w.bool("VblankSeen", self.vblank_seen);
        w.u16s("SpuRegisters", &self.spu.regs);
        w.u32("SpuCursor", self.spu.cursor);
        w.bytes("Cdrom", &[self.cdrom.index, self.cdrom.mask, self.cdrom.flags]);
        w.u32("MdecStatus", self.mdec.status);
        w.group("Sio0", |w| self.sio0.write_state(w));
        let s = &self.sio1;
        w.group("Sio1", |w| {
            w.u16s("Registers", &[s.mode, s.control, s.baud]);
            w.bool("RxFull", s.rx.is_some());
            w.u8("Rx", s.rx.unwrap_or(0));
        });
        w.bytes("Duart", &[self.duart.mr[0], self.duart.mr[1], self.duart.pointer]);
        w.u32("TtyLength", self.tty.len as u32);
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.now = r.u64()?;
        r.u64s(&mut self.queue)?;
        self.bus_free = r.u64()?;
        self.cpu_busy = r.u64()?;
        self.post = r.u8()?;
        r.u32s(&mut self.memctl.regs)?;
        self.memctl.ram_size = r.u32()?;
        let regs = self.memctl.regs;
        self.memctl = crate::memctl::MemControl::new(regs, self.memctl.ram_size);
        self.cache.bcc = r.u32()?;
        r.u32s(&mut self.cache.tags)?;
        r.u32s(&mut self.cache.words)?;
        self.irq.stat = r.u32()?;
        self.irq.mask = r.u32()?;
        for t in &mut self.timers.t {
            t.count = r.u32()?;
            t.target = r.u32()?;
            t.mode = r.u32()?;
            t.ticks = r.u64()?;
            t.hold = r.u64()?;
            let mut f = [false; 5];
            r.bools(&mut f)?;
            [t.wrap, t.wrap_hold, t.seen_blank, t.fired, t.running] = f;
        }
        let mut words = [[0u32; 7]; 3];
        for w in &mut words {
            r.u32s(w)?;
        }
        for (n, c) in self.dma.ch.iter_mut().enumerate() {
            c.madr = words[0][n];
            c.bcr = words[1][n];
            c.chcr = words[2][n];
        }
        self.dma.dpcr = r.u32()?;
        self.dma.dicr = r.u32()?;
        let active = r.i32()?;
        self.dma.active = (active >= 0).then_some(active as usize);
        self.dma.due = r.u64()?;
        self.dma.left = r.u32()?;
        self.dma.finishing = r.bool()?;
        let v = &mut self.video;
        let mut ranges = [0u32; 6];
        r.u32s(&mut ranges)?;
        [v.mode, v.x1, v.x2, v.y1, v.y2, v.field_lines] = ranges;
        let mut bases = [0u64; 6];
        r.u64s(&mut bases)?;
        [v.base_h, v.base_line, v.base_dots, v.base_hblanks, v.field_line, v.done_h] = bases;
        v.odd = r.bool()?;
        self.gpu.read_state(r)?;
        self.vblank_seen = r.bool()?;
        r.u16s(&mut self.spu.regs)?;
        self.spu.cursor = r.u32()?;
        let mut cd = [0u8; 3];
        r.bytes(&mut cd)?;
        [self.cdrom.index, self.cdrom.mask, self.cdrom.flags] = cd;
        self.mdec.status = r.u32()?;
        self.sio0.read_state(r)?;
        let s = &mut self.sio1;
        let mut sr = [0u16; 3];
        r.u16s(&mut sr)?;
        [s.mode, s.control, s.baud] = sr;
        let full = r.bool()?;
        let rx = r.u8()?;
        s.rx = full.then_some(rx);
        let mut du = [0u8; 3];
        r.bytes(&mut du)?;
        [self.duart.mr[0], self.duart.mr[1], self.duart.pointer] = du;
        self.tty.len = r.u32()? as usize;
        self.field_done = false;
        Ok(())
    }
}

impl Sio0 {
    /// The port's registers, FIFO, transfer and /ACK; the pad's switches are the host's input, not state.
    fn write_state(&self, w: &mut StateWriter) {
        w.u16s("Registers", &[self.mode, self.control, self.baud]);
        w.bytes("Fifo", &self.fifo.entries);
        let (phase, step) = match self.phase {
            Phase::Idle => (0, 0),
            Phase::Address => (1, 0),
            Phase::Controller(n) => (2, n),
            Phase::Silent => (3, 0),
        };
        let f = &self.fifo;
        w.bytes("Bytes", &[f.read, f.count, self.shifting.unwrap_or(0), self.pending.unwrap_or(0), phase, step]);
        w.bools("Flags", &[self.shifting.is_some(), self.pending.is_some(), self.latched_txen, self.dsr, self.irq]);
        w.u64s("Times", &[self.baud_base, self.started, self.sampled_at, self.ends_at, self.ack_at, self.ack_ends]);
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut regs = [0u16; 3];
        r.u16s(&mut regs)?;
        [self.mode, self.control, self.baud] = regs;
        r.bytes(&mut self.fifo.entries)?;
        let mut b = [0u8; 6];
        r.bytes(&mut b)?;
        (self.fifo.read, self.fifo.count) = (b[0] & 7, b[1].min(8));
        let mut f = [false; 5];
        r.bools(&mut f)?;
        self.shifting = f[0].then_some(b[2]);
        self.pending = f[1].then_some(b[3]);
        [self.latched_txen, self.dsr, self.irq] = [f[2], f[3], f[4]];
        self.phase = match b[4] {
            1 => Phase::Address,
            2 => Phase::Controller(b[5]),
            3 => Phase::Silent,
            _ => Phase::Idle,
        };
        let mut t = [0u64; 6];
        r.u64s(&mut t)?;
        [self.baud_base, self.started, self.sampled_at, self.ends_at, self.ack_at, self.ack_ends] = t;
        Ok(())
    }
}

fn word_code(k: Word) -> u8 {
    match k {
        Word::Command(n) => n,
        Word::Parameter => 100,
        Word::Data => 101,
        Word::Colour => 102,
        Word::Position => 103,
        Word::End => 104,
    }
}

fn word_of(code: u8) -> Word {
    match code {
        100 => Word::Parameter,
        101 => Word::Data,
        102 => Word::Colour,
        103 => Word::Position,
        104 => Word::End,
        n => Word::Command(n),
    }
}

fn rect_of(v: &[u32]) -> Rect {
    Rect { x: v[0], y: v[1], w: v[2].max(1), h: v[3].max(1) }
}

impl Gpu {
    /// The GPU's registers, FIFO, command processor and caches; VRAM is the state's own block, the picture not kept.
    fn write_state(&self, w: &mut StateWriter) {
        w.u32s("Env", &self.env);
        let t = &self.tail;
        w.bools("Flags", &[self.second_megabyte, self.irq, self.irq_rose, self.display.blanked, t.colour_next, t.polyline.is_some(), t.polyline == Some(true)]);
        w.u32s("Registers", &[self.dma_direction, self.latch, self.display.start, self.head as u32, self.len as u32, self.have as u32, self.need as u32]);
        w.u32s("FifoWords", &self.fifo.map(|f| f.0));
        w.u64s("FifoTimes", &self.fifo.map(|f| f.1));
        w.bytes("FifoKinds", &self.fifo.map(|f| word_code(f.2)));
        w.u32s("Parser", &[t.command, t.parameters, t.data]);
        w.u32s("Words", &self.words);
        let x = self.write.unwrap_or_default();
        w.u32s("Write", &[self.write.is_some() as u32, x.rect.x, x.rect.y, x.rect.w, x.rect.h, x.done]);
        let r = self.read.unwrap_or_default();
        w.u32s("Read", &[self.read.is_some() as u32, r.rect.x, r.rect.y, r.rect.w, r.rect.h, r.words]);
        w.u64("ReadStart", r.start);
        let p = self.polyline.unwrap_or_default();
        w.i32s("Polyline", &[self.polyline.is_some() as i32, p.command as i32, p.colour as i32, p.last.x, p.last.y, p.last.rgb[0], p.last.rgb[1], p.last.rgb[2]]);
        w.u64s("BusyUntil", &[self.busy_until, self.carry]);
        w.u32s("CacheTags", &self.cache.tags);
        w.u16s("CacheData", self.cache.data.as_flattened());
        w.u16s("Clut", &self.clut.colours);
        w.u32s("ClutAt", &[self.clut.x, self.clut.y, self.clut.loaded]);
        w.u64("ScanField", self.display.field_line);
        w.u32s("Scan", &[self.display.next, self.display.width, self.display.height]);
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        r.u32s(&mut self.env)?;
        let mut f = [false; 7];
        r.bools(&mut f)?;
        [self.second_megabyte, self.irq, self.irq_rose, self.display.blanked] = [f[0], f[1], f[2], f[3]];
        let (colour_next, polyline) = (f[4], f[5].then_some(f[6]));
        let mut g = [0u32; 7];
        r.u32s(&mut g)?;
        (self.dma_direction, self.latch, self.display.start) = (g[0], g[1], g[2]);
        (self.head, self.len, self.have, self.need) = (g[3] as usize % 16, (g[4] as usize).min(16), (g[5] as usize).min(16), (g[6] as usize).min(16));
        let mut words = [0u32; 16];
        let mut times = [0u64; 16];
        let mut kinds = [0u8; 16];
        r.u32s(&mut words)?;
        r.u64s(&mut times)?;
        r.bytes(&mut kinds)?;
        for i in 0..16 {
            self.fifo[i] = (words[i], times[i], word_of(kinds[i]));
        }
        let mut p = [0u32; 3];
        r.u32s(&mut p)?;
        self.tail = Parser { command: p[0], parameters: p[1], data: p[2], polyline, colour_next };
        r.u32s(&mut self.words)?;
        let mut t = [0u32; 6];
        r.u32s(&mut t)?;
        self.write = (t[0] != 0).then(|| Transfer { rect: rect_of(&t[1..5]), done: t[5] });
        let mut rd = [0u32; 6];
        r.u32s(&mut rd)?;
        let start = r.u64()?;
        self.read = (rd[0] != 0).then(|| ReadOut { rect: rect_of(&rd[1..5]), words: rd[5], start });
        let mut pl = [0i32; 8];
        r.i32s(&mut pl)?;
        self.polyline = (pl[0] != 0).then(|| Polyline { command: pl[1] as u32, colour: pl[2] as u32, last: Vertex { x: pl[3], y: pl[4], rgb: [pl[5], pl[6], pl[7]], u: 0, v: 0 } });
        let mut busy = [0u64; 2];
        r.u64s(&mut busy)?;
        [self.busy_until, self.carry] = busy;
        r.u32s(&mut self.cache.tags)?;
        r.u16s(self.cache.data.as_flattened_mut())?;
        r.u16s(&mut self.clut.colours)?;
        let mut c = [0u32; 3];
        r.u32s(&mut c)?;
        [self.clut.x, self.clut.y, self.clut.loaded] = c;
        self.display.field_line = r.u64()?;
        let mut d = [0u32; 3];
        r.u32s(&mut d)?;
        self.display.next = d[0];
        self.display.width = d[1].clamp(1, crate::gpu::display::MAX_WIDTH);
        self.display.height = d[2].clamp(1, crate::gpu::display::MAX_HEIGHT);
        Ok(())
    }
}

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("Frames", self.frames);
        w.structure("Cpu", &self.cpu);
        w.structure("Gte", &self.board.gte);
        w.group("Bus", |w| self.board.write_state(w));
        w.bool("ExePending", self.exe.is_some());
        let b = &self.board;
        w.bytes("RAM", &b.ram);
        w.bytes("SCRATCH", &b.scratch);
        w.bytes("VRAM", &b.gpu.vram.bytes);
        w.bytes("SPURAM", &b.spu.ram);
        w.bytes("TTY", &b.tty.text);
    }
}

impl StateMachine for Machine {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let mut r = StateReader::new(data);
        let magic = r.u32().map_err(|_| StateError::Foreign(0))?;
        if magic != STATE_MAGIC {
            return Err(StateError::Foreign(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION {
            return Err(StateError::Version(version));
        }
        let frames = r.i64()?;
        let mut cpu = self.cpu.clone();
        cpu.read_state(&mut r)?;
        let mut board = self.board.clone();
        board.gte.read_state(&mut r)?;
        board.read_state(&mut r)?;
        let pending = r.bool()?;
        r.bytes(&mut board.ram)?;
        r.bytes(&mut board.scratch)?;
        r.bytes(&mut board.gpu.vram.bytes)?;
        r.bytes(&mut board.spu.ram)?;
        r.bytes(&mut board.tty.text)?;
        board.schedule();
        self.frames = frames;
        self.cpu = cpu;
        self.board = board;
        if !pending {
            self.exe = None;
        } else if self.exe.is_none() {
            self.exe = self.image.clone();
        }
        Ok(())
    }

    fn state_size(&self) -> usize {
        let mut w = StateWriter::counter();
        self.write(&mut w);
        w.len()
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        let needed = self.state_size();
        if out.len() < needed {
            return Err(StateError::BufferTooSmall { needed });
        }
        let mut w = StateWriter::new(out);
        self.write(&mut w);
        Ok(w.len())
    }

    fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.write(&mut w);
        w.into_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::RAM_ID;
    use crate::machine::tests::{machine_for, program};
    use crate::media::Market;

    fn machine() -> Machine {
        machine_for(&program("loop: j loop\nnop"), Market::Americas)
    }

    #[test]
    fn the_version_1_layout_is_pinned() {
        let layout = machine().layout();
        let head: Vec<&str> = layout.lines().take(5).collect();
        assert_eq!(head, ["0 4 u32 Magic", "4 4 i32 Version", "8 8 i64 Frames", "16 128 u32[32] Cpu.R", "144 4 u32 Cpu.Hi"]);
        let tail: Vec<&str> = layout.lines().rev().take(6).collect();
        assert_eq!(
            tail,
            [
                "3682219 65536 u8[65536] TTY",
                "3157931 524288 u8[524288] SPURAM",
                "2109355 1048576 u8[1048576] VRAM",
                "2108331 1024 u8[1024] SCRATCH",
                "11179 2097152 u8[2097152] RAM",
                "11178 1 bool ExePending"
            ]
        );
        assert!(layout.contains("Bus.Timer2.Flags") && layout.contains("u32[16] Bus.Gpu.FifoWords") && layout.contains("u16[1024] Bus.Gpu.CacheData") && layout.contains("u64[6] Bus.Sio0.Times") && layout.contains("Cpu.InterruptView.Held"), "{layout}");
        assert_eq!(machine().state_size(), 3_747_755);
    }

    #[test]
    fn a_state_round_trips_mid_run_and_a_refused_one_changes_nothing() {
        let mut a = machine();
        a.advance();
        a.space_mut(RAM_ID).unwrap()[5] = 0x5A;
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        a.advance();
        let after = (a.board.now, a.cpu.regs.pc);
        let mut b = machine();
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.space(RAM_ID).unwrap()[5]), (1, 0x5A));
        b.advance();
        assert_eq!((b.board.now, b.cpu.regs.pc), after, "the run continues the same");

        let before = b.space(RAM_ID).unwrap().to_vec();
        assert_eq!(b.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(&s[..300]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(b"NPHR0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(b.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        let mut v2 = s.clone();
        v2[4] = 2;
        assert_eq!(b.load_state(&v2).unwrap_err().status(), status::VERSION);
        assert_eq!((b.frames, b.space(RAM_ID).unwrap()), (2, &before[..]));
    }
}
