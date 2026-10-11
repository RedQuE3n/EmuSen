//! The MDEC at its register interface (psx-spx "Macroblock Decoder (MDEC)"): MDEC0's commands and parameters, MDEC1's
//! status and reset, the input FIFO DMA0 fills and the output DMA1 drains, the decoder between them with its time,
//! and the order DMA1 writes a colour macroblock's words in. Diamond_MDEC.md records each rule and its source.

pub mod decode;
mod state;
#[cfg(test)]
mod tests;

use decode::{Depth, Format, Rle};

/// The input FIFO's words (ps1-tests' `mdec/frame`: "MDECin fifo size (32*uint32_t)"); once full, or holding the
/// command's last word, it is the decoder's until emptied (`mdec/step-by-step-log`'s console log, Diamond_MDEC.md §4.1).
pub const IN_WORDS: usize = 32;
/// The largest block, 24-bit, in words.
pub const BLOCK_WORDS: usize = 48;
/// The output FIFO's words, sixteen and the word ready to be read (the same log, Diamond_MDEC.md §4.2).
pub const OUT_WORDS: usize = 17;

/// The decoder's time (Diamond_MDEC.md §4.3).
pub mod timing {
    /// Cycles from a block's last halfword to its output, the IDCT and the colour conversion together.
    pub const BLOCK: u64 = 512;
}

/// What MDEC0's parameter words are for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Idle,
    Decode,
    /// MDEC(2): the luminance table, then the colour table if `colour`.
    Quant { colour: bool },
    Scale,
}

/// A decoded block: its words, how many have gone on to the output FIFO, and its number.
#[derive(Clone, Copy, Debug)]
pub struct Out {
    pub words: [u32; BLOCK_WORDS],
    pub len: usize,
    pub taken: usize,
    pub block: u8,
}

impl Default for Out {
    fn default() -> Out {
        Out { words: [0; BLOCK_WORDS], len: 0, taken: 0, block: 0 }
    }
}

impl Out {
    pub fn left(&self) -> usize {
        self.len - self.taken
    }
}

/// A word in the output FIFO, with its block's number and its index in the block, which DMA1's order needs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutWord {
    pub word: u32,
    pub block: u8,
    pub index: u8,
}

#[derive(Clone, Debug)]
pub struct Mdec {
    /// MDEC1's bits 30 and 29, the two requests' enables.
    pub enables: u32,
    /// Status bits 23-26, copied from each command's bits 25-28.
    pub format_bits: u32,
    /// Status bits 0-15.
    pub count: u16,
    pub command: Command,
    /// Parameter words the command still expects.
    pub remaining: u32,
    /// Parameter words of MDEC(2) or (3) taken so far.
    pub loaded: u32,
    pub fifo: [u32; IN_WORDS],
    pub fifo_head: usize,
    pub fifo_len: usize,
    /// The FIFO filled, or given its last word, and not yet emptied by the decoder, a word leaving it once both its
    /// halfwords are taken: status bit 30, no DREQ0.
    pub armed: bool,
    /// The halfwords of the word the decoder is reading that it has not yet taken, the next first.
    pub halves: [u16; 2],
    pub halves_len: usize,
    pub iq_y: [u8; 64],
    pub iq_uv: [u8; 64],
    pub scale: [i16; 64],
    pub format: Format,
    /// The block arriving: 0-3 for Y1-Y4, 4 for Cr (or a monochrome Y), 5 for Cb (psx-spx's numbering).
    pub block: u8,
    pub rle: Rle,
    pub cr: [i32; 64],
    pub cb: [i32; 64],
    /// The block in the IDCT, the cycle it is done, and whether it is: a luminance block then waits there for the
    /// output's block buffer to empty, holding the next block's last halfword back.
    pub computing: Option<u8>,
    pub done_at: u64,
    pub done: bool,
    pub coefficients: [i16; 64],
    pub values: [i32; 64],
    /// The decoder's own clock: when it is next free to take a halfword.
    pub clock: u64,
    pub buffer: Out,
    pub queue: [OutWord; OUT_WORDS],
    pub queue_head: usize,
    pub queue_len: usize,
    /// The last word read from MDEC0, which an empty output answers again [C].
    pub latch: u32,
}

impl Default for Mdec {
    fn default() -> Mdec {
        Mdec::new()
    }
}

/// A word's place in a colour macroblock as DMA1 writes it, relative to where the block order would put it.
pub fn reorder(depth: Depth, block: u8, index: usize) -> i32 {
    let (row_words, line_words) = match depth {
        Depth::Fifteen => (4, 8),
        Depth::TwentyFour => (6, 12),
        _ => return 0,
    };
    let b = block as usize & 3;
    let (r, c) = (index / row_words, index % row_words);
    let at = (r + 8 * (b >> 1)) * line_words + (b & 1) * row_words + c;
    at as i32 - (b * depth.words() + index) as i32
}

impl Mdec {
    pub fn new() -> Mdec {
        Mdec {
            enables: 0,
            format_bits: 0,
            count: 0,
            command: Command::Idle,
            remaining: 0,
            loaded: 0,
            fifo: [0; IN_WORDS],
            fifo_head: 0,
            fifo_len: 0,
            armed: false,
            halves: [0; 2],
            halves_len: 0,
            iq_y: [0; 64],
            iq_uv: [0; 64],
            scale: [0; 64],
            format: Format { depth: Depth::Four, signed: false, bit15: false },
            block: 4,
            rle: Rle::default(),
            cr: [0; 64],
            cb: [0; 64],
            computing: None,
            done_at: 0,
            done: false,
            coefficients: [0; 64],
            values: [0; 64],
            clock: 0,
            buffer: Out::default(),
            queue: [OutWord::default(); OUT_WORDS],
            queue_head: 0,
            queue_len: 0,
            latch: 0,
        }
    }

    /// MDEC1's reset: any command aborted, both FIFOs emptied, the status 80040000h; the tables are kept [A].
    fn reset(&mut self) {
        let (iq_y, iq_uv, scale, latch) = (self.iq_y, self.iq_uv, self.scale, self.latch);
        *self = Mdec { iq_y, iq_uv, scale, latch, ..Mdec::new() };
    }

    pub fn status(&self) -> u32 {
        let in_full = self.armed || self.fifo_len == IN_WORDS;
        let block = self.computing.unwrap_or(self.block) as u32;
        ((self.queue_len == 0) as u32) << 31
            | (in_full as u32) << 30
            | (self.busy() as u32) << 29
            | (self.request_in() as u32) << 28
            | (self.request_out() as u32) << 27
            | self.format_bits << 23
            | block << 16
            | self.count as u32
    }

    fn busy(&self) -> bool {
        self.command != Command::Idle
            || self.fifo_len > 0
            || self.halves_len > 0
            || self.computing.is_some()
            || self.buffer.left() > 0
            || self.queue_len > 0
    }

    /// DREQ0: enabled, the command expecting words, and room in the FIFO.
    pub fn request_in(&self) -> bool {
        self.enables & 1 << 30 != 0 && self.can_take()
    }

    /// DREQ1: enabled and words to give, but off once the block buffer is down to its last eight words with nothing
    /// behind them while input is still to come, and off for the very last word (the step-by-step log, §4.2).
    pub fn request_out(&self) -> bool {
        let input_done = self.remaining == 0 && self.fifo_len == 0 && self.computing.is_none();
        let behind = self.computing.is_some_and(|b| !self.format.depth.colour() || b < 4);
        let enough = behind || self.buffer.left() > 8 || (input_done && self.queue_len + self.buffer.left() > 1);
        self.enables & 1 << 29 != 0 && self.can_give() && enough
    }

    /// Whether a word can go into the FIFO now, and whether one can be read.
    pub fn can_take(&self) -> bool {
        self.remaining > 0 && !self.armed && self.fifo_len < IN_WORDS
    }

    pub fn can_give(&self) -> bool {
        self.queue_len > 0
    }

    pub fn control(&mut self, value: u32, now: u64) {
        self.run(now);
        if value & 1 << 31 != 0 {
            self.reset();
            self.clock = now;
        }
        self.enables = value & 0x6000_0000;
    }

    /// A word to MDEC0: a command when none is receiving parameters, else its next parameter, into the FIFO.
    pub fn write(&mut self, value: u32, now: u64) {
        self.run(now);
        if self.remaining == 0 {
            self.begin(value, now);
        } else if self.fifo_len < IN_WORDS {
            self.fifo[(self.fifo_head + self.fifo_len) % IN_WORDS] = value;
            self.fifo_len += 1;
            self.remaining -= 1;
            self.count = self.count.wrapping_sub(1);
            self.run(now);
        }
    }

    fn begin(&mut self, value: u32, now: u64) {
        self.format_bits = value >> 25 & 15;
        let words = match value >> 29 {
            1 => {
                self.command = Command::Decode;
                self.format = Format { depth: Depth::of(value >> 27), signed: value & 1 << 26 != 0, bit15: value & 1 << 25 != 0 };
                self.block = 4;
                self.rle = Rle::default();
                self.halves_len = 0;
                value & 0xFFFF
            }
            2 => {
                self.command = Command::Quant { colour: value & 1 != 0 };
                if value & 1 != 0 { 32 } else { 16 }
            }
            3 => {
                self.command = Command::Scale;
                32
            }
            _ => {
                self.count = value as u16;
                return;
            }
        };
        self.loaded = 0;
        self.remaining = words;
        self.count = (words as u16).wrapping_sub(1);
        self.clock = self.clock.max(now);
        if words == 0 {
            self.command = Command::Idle;
        }
    }

    /// MDEC0's read: the next output word, the reader held until the block being computed gives one, and the cycle it
    /// is given; the last word again when none is coming (Diamond_MDEC.md §4.4).
    pub fn read(&mut self, now: u64) -> (u32, u64) {
        let mut t = now;
        loop {
            self.run(t);
            if self.queue_len > 0 {
                return (self.pop(t).word, t);
            }
            match self.computing {
                Some(_) => t = self.done_at,
                None => return (self.latch, t),
            }
        }
    }

    /// The next output word, with its block's number and its index in the block; the decoder then brought on.
    pub fn pop(&mut self, now: u64) -> OutWord {
        if self.queue_len == 0 {
            return OutWord { word: self.latch, block: 0, index: 0 };
        }
        let w = self.queue[self.queue_head];
        self.queue_head = (self.queue_head + 1) % OUT_WORDS;
        self.queue_len -= 1;
        self.latch = w.word;
        self.run(now);
        w
    }

    /// The cycle at which the decoder next changes what it shows, if it is working.
    pub fn next_event(&self) -> u64 {
        match self.computing {
            Some(_) if !self.done => self.done_at,
            _ => u64::MAX,
        }
    }

    /// Brings the decoder to `now`: the output FIFO filled from the block buffer, the buffer from the IDCT, the IDCT's
    /// blocks done, and the halfwords taken until a block's last one finds the IDCT busy.
    pub fn run(&mut self, now: u64) {
        loop {
            self.drain(now);
            if self.computing.is_some() && !self.done && self.done_at <= now {
                self.idct_done();
                continue;
            }
            if !self.take() {
                if self.computing.is_none() {
                    self.clock = self.clock.max(now);
                }
                return;
            }
        }
    }

    /// The IDCT's result: chroma kept for its macroblock, freeing the IDCT; luminance left to wait for the buffer.
    fn idct_done(&mut self) {
        decode::idct(&self.coefficients, &self.scale, &mut self.values);
        self.done = true;
        self.clock = self.clock.max(self.done_at);
        match self.computing {
            Some(4) if self.format.depth.colour() => self.cr = self.values,
            Some(5) if self.format.depth.colour() => self.cb = self.values,
            _ => return,
        }
        self.computing = None;
    }

    /// Words from the block buffer into the output FIFO while it has room, and the IDCT's finished luminance block,
    /// converted with its macroblock's chroma, into the buffer once it is empty.
    fn drain(&mut self, now: u64) {
        loop {
            while self.buffer.left() > 0 && self.queue_len < OUT_WORDS {
                let i = self.buffer.taken;
                self.queue[(self.queue_head + self.queue_len) % OUT_WORDS] = OutWord { word: self.buffer.words[i], block: self.buffer.block, index: i as u8 };
                self.queue_len += 1;
                self.buffer.taken += 1;
            }
            let Some(b) = self.computing else { return };
            if !self.done || self.buffer.left() > 0 {
                return;
            }
            self.buffer = Out { len: self.format.depth.words(), block: b, ..Out::default() };
            if self.format.depth.colour() {
                decode::pack_colour(&self.values, &self.cr, &self.cb, (b as usize & 1) * 8, (b as usize >> 1) * 8, self.format, &mut self.buffer.words);
            } else {
                decode::pack_mono(&self.values, self.format, &mut self.buffer.words);
            }
            self.computing = None;
            self.clock = self.clock.max(now);
        }
    }

    /// The decoder's next word from the FIFO, once it is full or holds the command's last word, until it is empty.
    fn next_word(&mut self) -> Option<u32> {
        if !self.armed {
            if self.fifo_len == 0 || (self.fifo_len < IN_WORDS && self.remaining > 0) {
                return None;
            }
            self.armed = true;
        }
        let w = self.fifo[self.fifo_head];
        self.fifo_head = (self.fifo_head + 1) % IN_WORDS;
        self.fifo_len -= 1;
        self.armed = self.fifo_len > 0 || self.command == Command::Decode;
        Some(w)
    }

    /// The next halfword through the command's parameters; false when there is none, or when it would end a block
    /// while the IDCT is still busy with the last.
    fn take(&mut self) -> bool {
        if self.halves_len == 0 {
            let Some(w) = self.next_word() else {
                if self.command == Command::Decode && self.remaining == 0 && self.computing.is_none() {
                    self.command = Command::Idle;
                }
                return false;
            };
            match self.command {
                Command::Quant { colour } => {
                    let at = self.loaded as usize * 4;
                    let t = if at < 64 { &mut self.iq_y[at..at + 4] } else { &mut self.iq_uv[at - 64..at - 60] };
                    t.copy_from_slice(&w.to_le_bytes());
                    self.loaded += 1;
                    if self.loaded == if colour { 32 } else { 16 } {
                        self.command = Command::Idle;
                    }
                    return true;
                }
                Command::Scale => {
                    let at = self.loaded as usize * 2;
                    self.scale[at] = w as u16 as i16;
                    self.scale[at + 1] = (w >> 16) as u16 as i16;
                    self.loaded += 1;
                    if self.loaded == 32 {
                        self.command = Command::Idle;
                    }
                    return true;
                }
                Command::Idle => return true,
                Command::Decode => {}
            }
            self.halves = [w as u16, (w >> 16) as u16];
            self.halves_len = 2;
        }
        let n = self.halves[2 - self.halves_len];
        if self.computing.is_some() && self.rle.ends_with(n) {
            return false;
        }
        self.halves_len -= 1;
        self.armed = self.fifo_len > 0 || self.halves_len > 0;
        let colour = self.format.depth.colour();
        let table = if colour && self.block >= 4 { &self.iq_uv } else { &self.iq_y };
        if self.rle.take(n, table) {
            self.coefficients = self.rle.coefficients;
            self.computing = Some(self.block);
            self.done = false;
            self.done_at = self.clock + timing::BLOCK;
            if colour {
                self.block = match self.block {
                    4 => 5,
                    5 => 0,
                    3 => 4,
                    b => b + 1,
                };
            }
        }
        true
    }
}
