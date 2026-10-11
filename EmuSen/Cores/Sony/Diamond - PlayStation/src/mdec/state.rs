//! The MDEC's part of the state (Diamond_MDEC.md §6): its registers, the input FIFO, the tables, the block being
//! decoded with its time, the chroma of the macroblock, and the output with the block waiting behind it.

use emusen_native::{StateReader, StateWriter, Truncated};

use super::decode::{Depth, Format};
use super::{Command, Mdec, Out, OutWord};

fn command_code(c: Command) -> u8 {
    match c {
        Command::Idle => 0,
        Command::Decode => 1,
        Command::Quant { colour: false } => 2,
        Command::Quant { colour: true } => 3,
        Command::Scale => 4,
    }
}

fn command_of(c: u8) -> Command {
    match c {
        1 => Command::Decode,
        2 => Command::Quant { colour: false },
        3 => Command::Quant { colour: true },
        4 => Command::Scale,
        _ => Command::Idle,
    }
}

fn depth_code(d: Depth) -> u8 {
    match d {
        Depth::Four => 0,
        Depth::Eight => 1,
        Depth::TwentyFour => 2,
        Depth::Fifteen => 3,
    }
}

fn write_out(w: &mut StateWriter, name: &str, o: &Out) {
    w.group(name, |w| {
        w.u32s("Words", &o.words);
        w.u32s("Lengths", &[o.len as u32, o.taken as u32]);
        w.u8("Block", o.block);
    });
}

fn read_out(r: &mut StateReader, o: &mut Out) -> Result<(), Truncated> {
    r.u32s(&mut o.words)?;
    let mut l = [0u32; 2];
    r.u32s(&mut l)?;
    o.len = (l[0] as usize).min(o.words.len());
    o.taken = (l[1] as usize).min(o.len);
    o.block = r.u8()?;
    Ok(())
}

fn i16s(v: &[i16; 64]) -> [u16; 64] {
    v.map(|x| x as u16)
}

fn read_i16s(r: &mut StateReader, out: &mut [i16; 64]) -> Result<(), Truncated> {
    let mut v = [0u16; 64];
    r.u16s(&mut v)?;
    for (o, x) in out.iter_mut().zip(v) {
        *o = x as i16;
    }
    Ok(())
}

impl Mdec {
    pub fn write_state(&self, w: &mut StateWriter) {
        w.u32s("Registers", &[self.enables, self.format_bits, self.count as u32, self.remaining, self.loaded, self.latch]);
        w.bytes("Command", &[command_code(self.command), depth_code(self.format.depth), self.format.signed as u8, self.format.bit15 as u8, self.block]);
        w.u32s("Fifo", &self.fifo);
        w.u32s("FifoPlace", &[self.fifo_head as u32, self.fifo_len as u32]);
        w.bool("Armed", self.armed);
        w.u16s("Halves", &self.halves);
        w.u8("HalvesLeft", self.halves_len as u8);
        w.bytes("QuantY", &self.iq_y);
        w.bytes("QuantUV", &self.iq_uv);
        w.u16s("Scale", &i16s(&self.scale));
        w.u16s("Coefficients", &i16s(&self.rle.coefficients));
        w.bytes("Run", &[self.rle.k.map_or(0xFF, |k| k), self.rle.q_scale]);
        w.i32s("Cr", &self.cr);
        w.i32s("Cb", &self.cb);
        w.u8("Computing", self.computing.map_or(0xFF, |b| b));
        w.bool("Done", self.done);
        w.u16s("Computed", &i16s(&self.coefficients));
        w.i32s("Values", &self.values);
        w.u64s("Times", &[self.done_at, self.clock]);
        write_out(w, "Buffer", &self.buffer);
        w.u32s("Queue", &self.queue.map(|q| q.word));
        w.bytes("QueueBlocks", &self.queue.map(|q| q.block));
        w.bytes("QueueIndexes", &self.queue.map(|q| q.index));
        w.u32s("QueuePlace", &[self.queue_head as u32, self.queue_len as u32]);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut regs = [0u32; 6];
        r.u32s(&mut regs)?;
        [self.enables, self.format_bits] = [regs[0], regs[1]];
        self.count = regs[2] as u16;
        [self.remaining, self.loaded, self.latch] = [regs[3], regs[4], regs[5]];
        let mut c = [0u8; 5];
        r.bytes(&mut c)?;
        self.command = command_of(c[0]);
        self.format = Format { depth: Depth::of(c[1] as u32), signed: c[2] != 0, bit15: c[3] != 0 };
        self.block = c[4].min(5);
        r.u32s(&mut self.fifo)?;
        let mut place = [0u32; 2];
        r.u32s(&mut place)?;
        self.fifo_head = place[0] as usize % self.fifo.len();
        self.fifo_len = (place[1] as usize).min(self.fifo.len());
        self.armed = r.bool()?;
        r.u16s(&mut self.halves)?;
        self.halves_len = (r.u8()? as usize).min(2);
        r.bytes(&mut self.iq_y)?;
        r.bytes(&mut self.iq_uv)?;
        read_i16s(r, &mut self.scale)?;
        read_i16s(r, &mut self.rle.coefficients)?;
        let mut run = [0u8; 2];
        r.bytes(&mut run)?;
        self.rle.k = (run[0] <= 63).then_some(run[0]);
        self.rle.q_scale = run[1];
        r.i32s(&mut self.cr)?;
        r.i32s(&mut self.cb)?;
        let computing = r.u8()?;
        self.computing = (computing <= 5).then_some(computing);
        self.done = r.bool()?;
        read_i16s(r, &mut self.coefficients)?;
        r.i32s(&mut self.values)?;
        let mut t = [0u64; 2];
        r.u64s(&mut t)?;
        [self.done_at, self.clock] = t;
        read_out(r, &mut self.buffer)?;
        let mut words = [0u32; super::OUT_WORDS];
        let mut blocks = [0u8; super::OUT_WORDS];
        let mut indexes = [0u8; super::OUT_WORDS];
        r.u32s(&mut words)?;
        r.bytes(&mut blocks)?;
        r.bytes(&mut indexes)?;
        for (i, q) in self.queue.iter_mut().enumerate() {
            *q = OutWord { word: words[i], block: blocks[i], index: indexes[i] };
        }
        let mut place = [0u32; 2];
        r.u32s(&mut place)?;
        self.queue_head = place[0] as usize % super::OUT_WORDS;
        self.queue_len = (place[1] as usize).min(super::OUT_WORDS);
        Ok(())
    }
}
