//! The GPU (Diamond_Plan.md §5.4, Diamond_GPU.md): GP0's words through the 16-word FIFO to a command processor that
//! draws each command whole into the one VRAM when it starts and is then busy for its drawing time; GP1's display
//! control; GPUSTAT with its ready bits; GPUREAD; and the display read from VRAM line by line. Times are CPU cycles,
//! the drawing time counted in GPU cycles of the video clock (`timing`).

pub mod display;
pub mod draw;
pub mod raster;
pub mod texture;
pub mod timing;
pub mod vram;

use display::Display;
use draw::{Area, Paint, Painter, Work};
use raster::Vertex;
use texture::{ClutCache, Texture, TextureCache};
use timing::cpu_cycles;
use vram::{Mask, Rect, Vram};

use crate::video::Video;

pub const FIFO_WORDS: usize = 16;

/// GPUSTAT's bits after GP1(00h) apart from those the video timing and the FIFO give (psx-spx "GP1(00h)").
pub const STATUS_RESET: u32 = 0x1480_2000;

/// What a GP0 word is to the command processor, as the FIFO's tail parsed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// A command's first word, and how many words its fixed part takes.
    Command(u8),
    Parameter,
    /// A CPU-to-VRAM transfer's data.
    Data,
    /// A polyline's further vertex: its colour (shaded) or its position.
    Colour,
    Position,
    End,
}

/// How the FIFO's tail reads the stream: what the next word is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Parser {
    pub command: u32,
    pub parameters: u32,
    pub data: u32,
    /// In a polyline past its first segment: shaded or not, and whether the next word is a vertex's colour.
    pub polyline: Option<bool>,
    pub colour_next: bool,
}

/// The command words a rendering command takes in all (psx-spx's render and transfer commands).
pub fn length(command: u32) -> u8 {
    let c = command >> 24;
    match c >> 5 {
        1 => {
            let n = if c & 8 != 0 { 4 } else { 3 };
            (1 + n * (1 + (c >> 2 & 1)) + if c & 0x10 != 0 { n - 1 } else { 0 }) as u8
        }
        2 => (3 + (c >> 4 & 1)) as u8,
        3 => (2 + (c >> 2 & 1) + (c & 0x18 == 0) as u32) as u8,
        4 => 4,
        5 | 6 => 3,
        _ if c == 0x02 => 3,
        _ => 1,
    }
}

impl Parser {
    /// The next word's meaning, or None for a NOP, which takes no place in the FIFO (psx-spx "GP0(00h)"); GP0(E3h) to
    /// (E5h) go through it in order (Diamond_Disputes.md D-9).
    pub fn classify(&mut self, word: u32) -> Option<Word> {
        if self.parameters > 0 {
            self.parameters -= 1;
            if self.parameters == 0 {
                match self.command >> 29 {
                    5 => {
                        let r = Rect::copy_params(0, word);
                        self.data = r.pixels().div_ceil(2);
                    }
                    2 if self.command & 1 << 27 != 0 => {
                        self.polyline = Some(self.command & 1 << 28 != 0);
                        self.colour_next = self.command & 1 << 28 != 0;
                    }
                    _ => {}
                }
            }
            return Some(Word::Parameter);
        }
        if self.data > 0 {
            self.data -= 1;
            return Some(Word::Data);
        }
        if let Some(shaded) = self.polyline {
            let first = self.colour_next || !shaded;
            if first && word & 0xF000_F000 == 0x5000_5000 {
                self.polyline = None;
                return Some(Word::End);
            }
            if shaded && self.colour_next {
                self.colour_next = false;
                return Some(Word::Colour);
            }
            self.colour_next = shaded;
            return Some(Word::Position);
        }
        let c = word >> 24;
        match c {
            0x00 | 0x04..=0x1E | 0xE0 | 0xE7..=0xEF => None,
            _ => {
                let n = length(word);
                self.command = word;
                self.parameters = n as u32 - 1;
                Some(Word::Command(n))
            }
        }
    }
}

/// A transfer in progress: the rectangle and the pixels done.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Transfer {
    pub rect: Rect,
    pub done: u32,
}

/// A VRAM-to-CPU transfer: its words made ready from `start` at the read rate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadOut {
    pub rect: Rect,
    pub words: u32,
    pub start: u64,
}

/// A polyline being drawn: its flags, the last vertex, and a colour received for the next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Polyline {
    pub command: u32,
    pub last: Vertex,
    pub colour: u32,
}

#[derive(Clone)]
pub struct Gpu {
    pub version: u32,
    pub clock: (u64, u64),
    pub vram: Vram,
    pub display: Display,
    pub cache: TextureCache,
    pub clut: ClutCache,
    /// GP0(E1h)..(E6h) as last written, E1h with its bits 11 to 13 as the version keeps them.
    pub env: [u32; 6],
    /// GP1(09h): whether E1h's bit 11 takes the texture page into a second megabyte of VRAM (208-pin only).
    pub second_megabyte: bool,
    pub irq: bool,
    pub dma_direction: u32,
    /// GPUREAD's latch.
    pub latch: u32,
    /// The FIFO: words that arrived while the processor was busy, with their arrival and meaning.
    pub fifo: [(u32, u64, Word); FIFO_WORDS],
    pub head: usize,
    pub len: usize,
    pub tail: Parser,
    /// The command being received, its words so far and its length.
    pub words: [u32; 16],
    pub have: usize,
    pub need: usize,
    pub write: Option<Transfer>,
    pub read: Option<ReadOut>,
    pub polyline: Option<Polyline>,
    /// The cycle the current command's drawing time ends, and the part of a cycle it ran past that, in
    /// `timing::carry` units, which the next command started at once takes up.
    pub busy_until: u64,
    pub carry: u64,
    /// An interrupt GP0(1Fh) raised that the bus has not taken.
    pub irq_rose: bool,
}

fn sext11(v: u32) -> i32 {
    ((v << 21) as i32) >> 21
}

fn rgb(word: u32) -> [i32; 3] {
    [(word & 0xFF) as i32, (word >> 8 & 0xFF) as i32, (word >> 16 & 0xFF) as i32]
}

impl Gpu {
    /// `version` is GP1(10h)'s index 7: 2 for the 208-pin chip, 0 for the 160-pin one; `clock` the video clock's half
    /// cycles per CPU cycle.
    pub fn new(version: u32, clock: (u64, u64)) -> Gpu {
        Gpu {
            version,
            clock,
            vram: Vram::default(),
            display: Display::default(),
            cache: TextureCache::default(),
            clut: ClutCache::default(),
            env: [0; 6],
            second_megabyte: false,
            irq: false,
            dma_direction: 0,
            latch: 0,
            fifo: [(0, 0, Word::Parameter); FIFO_WORDS],
            head: 0,
            len: 0,
            tail: Parser::default(),
            words: [0; 16],
            have: 0,
            need: 0,
            write: None,
            read: None,
            polyline: None,
            busy_until: 0,
            carry: 0,
            irq_rose: false,
        }
    }

    fn v2(&self) -> bool {
        self.version != 0
    }

    fn cycles(&self, gpu: u64) -> u64 {
        cpu_cycles(gpu, self.clock)
    }

    /// The commands whose words are in and whose turn came by `now`, drawn in order.
    pub fn run(&mut self, now: u64, video: &Video) {
        while self.len > 0 {
            let (w, arrived, kind) = self.fifo[self.head];
            let t = arrived.max(self.busy_until);
            if t > now {
                break;
            }
            self.head = (self.head + 1) % FIFO_WORDS;
            self.len -= 1;
            self.feed(w, kind, t, video);
        }
    }

    /// Brought to `now`: the commands run, and the display read up to it.
    pub fn sync(&mut self, now: u64, video: &Video) {
        self.run(now, video);
        self.display.scan(&self.vram, video, now);
    }

    /// A GP0 word at `now`; returns the cycle it was taken, later than `now` when the FIFO was full.
    pub fn gp0(&mut self, word: u32, now: u64, video: &Video) -> u64 {
        self.run(now, video);
        let Some(kind) = self.tail.classify(word) else { return now };
        let mut t = now;
        while self.len == FIFO_WORDS {
            let (_, arrived, _) = self.fifo[self.head];
            t = t.max(arrived.max(self.busy_until));
            self.run(t, video);
        }
        if self.len == 0 && self.busy_until <= t {
            self.feed(word, kind, t, video);
        } else {
            self.fifo[(self.head + self.len) % FIFO_WORDS] = (word, t, kind);
            self.len += 1;
        }
        t
    }

    /// The next cycle the FIFO moves, if it holds anything.
    pub fn next_event(&self) -> u64 {
        if self.len == 0 { u64::MAX } else { self.fifo[self.head].1.max(self.busy_until) }
    }

    /// GPUSTAT bit 28: the FIFO empty, and no polygon's or line's vertices awaited (Diamond_Disputes.md D-10).
    fn write_fifo_empty(&self) -> bool {
        self.len == 0 && self.polyline.is_none() && !(self.have > 0 && matches!(self.words[0] >> 29, 1 | 2))
    }

    /// Whether the GPU requests DMA at `now` (GPUSTAT bit 25), and when that may next change.
    pub fn dreq(&self, now: u64) -> bool {
        match self.dma_direction {
            0 => false,
            1 => self.len < FIFO_WORDS,
            2 => self.write_fifo_empty(),
            _ => self.read_ready(now),
        }
    }

    pub fn dreq_change(&self) -> u64 {
        let read = self.read.map_or(u64::MAX, |r| r.start + self.cycles(timing::READ_SETUP));
        self.next_event().min(read)
    }

    fn read_ready(&self, now: u64) -> bool {
        self.read.is_some_and(|r| now >= r.start + self.cycles(timing::READ_SETUP))
    }

    /// GPUSTAT at `now` (psx-spx "GPU Status Register"; the ready bits as Diamond_GPU.md §2.3 gives them).
    pub fn status(&mut self, video: &Video, now: u64) -> u32 {
        self.run(now, video);
        let e1 = self.env[0];
        let mut s = e1 & 0x7FF | (self.env[5] & 3) << 11 | (video.mode & 0x3F) << 17 | (video.mode >> 6 & 1) << 16;
        if self.v2() {
            s |= (e1 >> 11 & 1) << 15 | (video.mode >> 7 & 1) << 14;
        }
        s |= (self.display.blanked as u32) << 23 | (self.irq as u32) << 24 | self.dma_direction << 29;
        let idle = self.len == 0 && self.have == 0 && self.write.is_none() && self.read.is_none() && self.polyline.is_none() && self.busy_until <= now;
        s |= (idle as u32) << 26 | (self.read_ready(now) as u32) << 27 | (self.write_fifo_empty() as u32) << 28 | (self.dreq(now) as u32) << 25;
        let mut bits = video.status_bits(now);
        if !self.v2() && !video.interlaced() {
            bits &= !(1 << 13);
        }
        s | bits
    }

    /// GPUREAD at `now`: the next word of a VRAM-to-CPU transfer, ready at the read rate, or the latch; and the cycle
    /// it was read, later than `now` when the word was not yet ready.
    pub fn gpuread(&mut self, video: &Video, now: u64) -> (u32, u64) {
        self.run(now, video);
        let Some(mut r) = self.read else { return (self.latch, now) };
        let k = (r.words as u64).min(r.rect.pixels() as u64 / 2);
        let t = now.max(r.start + self.cycles(timing::READ_SETUP + timing::READ_WORD * k));
        let mut value = 0;
        for half in 0..2 {
            let i = r.words * 2 + half;
            let (x, y) = r.rect.at(i.min(r.rect.pixels() - 1));
            let x = if i < r.rect.pixels() { x } else { x + 1 };
            value |= (self.vram.get(x, y) as u32) << (16 * half);
        }
        r.words += 1;
        self.read = (r.words * 2 < r.rect.pixels()).then_some(r);
        self.latch = value;
        (value, t)
    }

    /// One word to the command processor at `t`, its turn come.
    fn feed(&mut self, w: u32, kind: Word, t: u64, video: &Video) {
        match kind {
            Word::Command(n) => {
                self.words[0] = w;
                self.have = 1;
                self.need = n as usize;
                if n == 1 {
                    self.execute(t, video);
                }
            }
            Word::Parameter => {
                if self.have < self.words.len() {
                    self.words[self.have] = w;
                }
                self.have += 1;
                if self.have == self.need {
                    self.execute(t, video);
                } else {
                    self.quad_page();
                }
            }
            Word::Data => self.data(w, t, video),
            Word::Colour => {
                if let Some(p) = self.polyline.as_mut() {
                    p.colour = w;
                }
            }
            Word::Position => self.polyline_vertex(w, t, video),
            Word::End => self.polyline = None,
        }
    }

    fn mask(&self) -> Mask {
        Mask::from_e6(self.env[5])
    }

    fn area(&self) -> Area {
        let y = |v: u32| (v >> 10 & if self.v2() { 0x3FF } else { 0x1FF }) as i32;
        let y2 = if self.two_megabytes() { y(self.env[3]).min(511) } else { y(self.env[3]) };
        Area { x1: (self.env[2] & 0x3FF) as i32, y1: y(self.env[2]), x2: (self.env[3] & 0x3FF) as i32, y2 }
    }

    /// A vertex with the drawing offset added in 11 bits, wrapping (Diamond_Disputes.md D-23).
    fn vertex(&self, xy: u32, colour: u32, uv: u32) -> Vertex {
        let off = self.env[4];
        let x = sext11((sext11(xy) + sext11(off)) as u32);
        let y = sext11((sext11(xy >> 16) + sext11(off >> 11)) as u32);
        Vertex { x, y, rgb: rgb(colour), u: uv as u8, v: (uv >> 8) as u8 }
    }

    /// `v` placed from `first` by their distance as the command gave them, the offset moving the primitive whole (D-23).
    fn place(&self, first: Vertex, v: Vertex) -> Vertex {
        let (ox, oy) = (sext11(self.env[4]), sext11(self.env[4] >> 11));
        let raw = |p: i32, o: i32| sext11((p - o) as u32);
        Vertex { x: first.x + raw(v.x, ox) - raw(first.x, ox), y: first.y + raw(v.y, oy) - raw(first.y, oy), ..v }
    }

    fn paint(&self, blended: bool, texture: Option<Texture>, raw: bool, dither: bool, video: &Video) -> Paint {
        let e1 = self.env[0];
        let hires = video.interlaced() && video.mode & 1 << 2 != 0;
        Paint {
            blend: blended.then_some(e1 >> 5 & 3),
            texture,
            raw,
            dither: dither && e1 & 1 << 9 != 0,
            mask: self.mask(),
            area: self.area(),
            skip_parity: (hires && e1 & 1 << 10 == 0).then_some(video.odd as i32),
            crop_modulation: !self.v2(),
        }
    }

    /// Whether the texture page may be in VRAM's second megabyte (GP1(09h), 208-pin only).
    fn two_megabytes(&self) -> bool {
        self.v2() && self.second_megabyte
    }

    /// Draws with `paint`, returning the work done and the CLUT entries loaded.
    fn draw(&mut self, paint: Paint, f: impl FnOnce(&mut Painter)) -> (Work, u32) {
        let loaded = paint.texture.map_or(0, |t| self.clut.prepare(&self.vram, &t));
        let mut p = Painter { vram: &mut self.vram, cache: &mut self.cache, clut: &self.clut, paint, work: Work::default() };
        f(&mut p);
        (p.work, loaded)
    }

    /// A whole command, its words in, at `t`: drawn now, the processor busy for its time.
    fn execute(&mut self, t: u64, video: &Video) {
        self.display.scan(&self.vram, video, t);
        let w = self.words;
        self.have = 0;
        let c = w[0] >> 24;
        let cost = match c >> 5 {
            1 => self.polygon(&w, video),
            2 => self.line_start(&w, video),
            3 => self.rectangle(&w, video),
            4 => {
                let src = Rect::copy_params(w[1], w[3]);
                let dst = Rect::copy_params(w[2], w[3]);
                let mask = self.mask();
                self.vram.copy(src, dst, mask);
                timing::COMMAND + src.pixels() as u64 * timing::COPY_PIXEL
            }
            5 => {
                self.write = Some(Transfer { rect: Rect::copy_params(w[1], w[2]), done: 0 });
                timing::COMMAND
            }
            6 => {
                self.read = Some(ReadOut { rect: Rect::copy_params(w[1], w[2]), words: 0, start: t });
                timing::COMMAND
            }
            _ => self.environment(w[0]),
        };
        self.charge(t, cost);
    }

    /// The processor busy from `t` for `cost`, the fraction of a cycle carried while commands run back to back.
    fn charge(&mut self, t: u64, cost: u64) {
        let carry = if t == self.busy_until { self.carry } else { 0 };
        (self.busy_until, self.carry) = timing::advance(t, carry, cost, self.clock);
    }

    fn environment(&mut self, w: u32) -> u64 {
        match w >> 24 {
            0x01 => {
                self.cache.clear();
                self.clut.clear();
            }
            0x02 => {
                let r = Rect::fill_params(self.words[1], self.words[2]);
                self.vram.fill(r, w);
                return timing::COMMAND + if r.w == 0 { 0 } else { r.h as u64 * (timing::FILL_ROW + (r.w as u64).div_ceil(16) * timing::FILL_16) };
            }
            0x1F => {
                self.irq_rose |= !self.irq;
                self.irq = true;
            }
            0xE1 => self.set_e1(w),
            0xE2 => self.env[1] = w & 0x000F_FFFF,
            0xE3..=0xE5 => self.env[(w >> 24) as usize - 0xE1] = w & 0x00FF_FFFF,
            0xE6 => self.env[5] = w & 3,
            _ => {}
        }
        timing::COMMAND
    }

    /// GP0(E1h): bits 0-10; bit 11 only while GP1(09h) allows it, and the flips, on the 208-pin chip.
    fn set_e1(&mut self, w: u32) {
        let keep = if self.v2() { 0x37FF | if self.second_megabyte { 0x800 } else { 0 } } else { 0x7FF };
        self.env[0] = w & keep;
    }

    /// A polygon's page attribute into GP0(E1h): bits 0-8, and bit 11 while GP1(09h) allows it.
    fn set_page(&mut self, page: u32) {
        let bits = 0x1FF | if self.two_megabytes() { 0x800 } else { 0 };
        self.env[0] = self.env[0] & !bits | page & bits;
    }

    /// A textured quad's page, applied once its first triangle's words are in (Diamond_Disputes.md D-23).
    fn quad_page(&mut self) {
        let c = self.words[0] >> 24;
        if c >> 5 == 1 && c & 0xC == 0xC {
            let shaded = c & 0x10 != 0;
            if self.have == if shaded { 9 } else { 7 } {
                self.set_page(self.words[if shaded { 5 } else { 4 }] >> 16);
            }
        }
    }

    fn polygon(&mut self, w: &[u32; 16], video: &Video) -> u64 {
        let c = w[0] >> 24;
        let (quad, shaded, textured, blended, raw) = (c & 8 != 0, c & 0x10 != 0, c & 4 != 0, c & 2 != 0, c & 1 != 0);
        let n = if quad { 4 } else { 3 };
        let mut v = [Vertex::default(); 4];
        let mut at = 1;
        let (mut clut, mut page) = (0, 0);
        for (i, vx) in v.iter_mut().enumerate().take(n) {
            let colour = if i > 0 && shaded {
                at += 1;
                w[at - 1]
            } else {
                w[0]
            };
            let xy = w[at];
            at += 1;
            let uv = if textured {
                at += 1;
                w[at - 1]
            } else {
                0
            };
            if i == 0 {
                clut = uv >> 16;
            } else if i == 1 {
                page = uv >> 16;
            }
            *vx = self.vertex(xy, colour, uv);
        }
        for i in 1..n {
            v[i] = self.place(v[0], v[i]);
        }
        let mut texture = None;
        if textured {
            self.set_page(page);
            texture = Some(Texture::new(self.env[0], clut, self.env[1], self.two_megabytes()));
        }
        let modulated = texture.is_some() && !raw;
        let paint = self.paint(blended, texture, raw && texture.is_some(), modulated || (shaded && texture.is_none()), video);
        let tex = texture.is_some();
        let (work, loaded) = self.draw(paint, |p| {
            raster::triangle(p, [v[0], v[1], v[2]], shaded, tex);
            if quad {
                raster::triangle(p, [v[1], v[2], v[3]], shaded, tex);
            }
        });
        timing::drawing(timing::TRIANGLE * (n as u64 - 2) + loaded as u64 * timing::CLUT_ENTRY, work)
    }

    fn rectangle(&mut self, w: &[u32; 16], video: &Video) -> u64 {
        let c = w[0] >> 24;
        let (textured, blended, raw) = (c & 4 != 0, c & 2 != 0, c & 1 != 0);
        let mut at = 2;
        let uv = if textured {
            at += 1;
            w[2]
        } else {
            0
        };
        let (sw, sh) = match c >> 3 & 3 {
            0 => ((w[at] & 0x3FF) as i32, (w[at] >> 16 & 0x1FF) as i32),
            1 => (1, 1),
            2 => (8, 8),
            _ => (16, 16),
        };
        let v = self.vertex(w[1], w[0], uv);
        let texture = textured.then(|| Texture::new(self.env[0], uv >> 16, self.env[1], self.two_megabytes()));
        let flip = if self.v2() { (self.env[0] & 1 << 12 != 0, self.env[0] & 1 << 13 != 0) } else { (false, false) };
        let paint = self.paint(blended, texture, raw, false, video);
        let sprite = raster::Sprite { x: v.x, y: v.y, w: sw, h: sh, rgb: v.rgb, uv: (v.u, v.v), flip };
        let (work, loaded) = self.draw(paint, |p| raster::rectangle(p, sprite));
        timing::drawing(work.rows as u64 * timing::RECTANGLE_ROW + loaded as u64 * timing::CLUT_ENTRY, work)
    }

    fn line_start(&mut self, w: &[u32; 16], video: &Video) -> u64 {
        let c = w[0] >> 24;
        let shaded = c & 0x10 != 0;
        let a = self.vertex(w[1], w[0], 0);
        let b = self.place(a, if shaded { self.vertex(w[3], w[2], 0) } else { self.vertex(w[2], w[0], 0) });
        if c & 8 != 0 {
            self.polyline = Some(Polyline { command: w[0], last: b, colour: w[0] });
        }
        self.segment(w[0], a, b, video)
    }

    fn segment(&mut self, command: u32, a: Vertex, b: Vertex, video: &Video) -> u64 {
        let c = command >> 24;
        let (shaded, blended) = (c & 0x10 != 0, c & 2 != 0);
        let paint = self.paint(blended, None, false, true, video);
        let (work, _) = self.draw(paint, |p| raster::line(p, a, b, shaded));
        timing::drawing(timing::LINE, work)
    }

    fn polyline_vertex(&mut self, xy: u32, t: u64, video: &Video) {
        let Some(p) = self.polyline else { return };
        self.display.scan(&self.vram, video, t);
        let shaded = p.command & 1 << 28 != 0;
        let b = self.place(p.last, self.vertex(xy, if shaded { p.colour } else { p.command }, 0));
        let cost = self.segment(p.command, p.last, b, video);
        if let Some(q) = self.polyline.as_mut() {
            q.last = b;
        }
        self.charge(t, cost);
    }

    fn data(&mut self, w: u32, t: u64, video: &Video) {
        let Some(mut x) = self.write else { return };
        self.display.scan(&self.vram, video, t);
        let mask = self.mask();
        for half in 0..2 {
            if x.done < x.rect.pixels() {
                let (px, py) = x.rect.at(x.done);
                self.vram.put(px, py, (w >> (16 * half)) as u16, mask);
                x.done += 1;
            }
        }
        self.write = (x.done < x.rect.pixels()).then_some(x);
        self.charge(t, timing::WRITE_WORD);
    }

    /// GP1, at `now`, brought up to it first; the video timing takes the display's ranges and mode.
    pub fn gp1(&mut self, word: u32, video: &mut Video, now: u64) {
        self.sync(now, video);
        let p = word & 0x00FF_FFFF;
        match word >> 24 & 0x3F {
            0x00 => {
                self.reset_fifo();
                self.irq = false;
                self.display.blanked = true;
                self.dma_direction = 0;
                self.display.start = 0;
                self.env = [0; 6];
                video.retime(now, |v| {
                    v.mode = 0;
                    v.x1 = 0x200;
                    v.x2 = 0x200 + 256 * 10;
                    v.y1 = 0x10;
                    v.y2 = 0x10 + 240;
                });
            }
            0x01 => {
                self.reset_fifo();
                self.clut.clear();
            }
            0x02 => self.irq = false,
            0x03 => self.display.blanked = p & 1 != 0,
            0x04 => self.dma_direction = p & 3,
            0x05 => self.display.start = p & if self.v2() { 0xFFFFF } else { 0x7FFFF },
            0x06 => video.retime(now, |v| {
                v.x1 = p & 0xFFF;
                v.x2 = p >> 12 & 0xFFF;
            }),
            0x07 => video.retime(now, |v| {
                v.y1 = p & 0x3FF;
                v.y2 = p >> 10 & 0x3FF;
            }),
            0x08 => video.retime(now, |v| v.mode = p & 0xFF),
            0x09 if self.v2() => self.second_megabyte = p & 1 != 0,
            0x10..=0x1F => {
                let index = p & if self.v2() { 0xF } else { 0x7 };
                let area = if self.v2() { 0xFFFFF } else { 0x7FFFF };
                let keep = |latch: u32, bits: u32, value: u32| latch & !bits | value & bits;
                self.latch = match index {
                    2 => keep(self.latch, 0xFFFFF, self.env[1]),
                    3 | 4 => keep(self.latch, area, self.env[index as usize - 1]),
                    5 => keep(self.latch, 0x3F_FFFF, self.env[4]),
                    7 if self.v2() => self.version,
                    _ => 0,
                };
            }
            _ => {}
        }
    }

    fn reset_fifo(&mut self) {
        self.len = 0;
        self.head = 0;
        self.tail = Parser::default();
        self.have = 0;
        self.write = None;
        self.read = None;
        self.polyline = None;
    }

    /// Whether GP0(1Fh) raised the interrupt since last asked.
    pub fn take_irq(&mut self) -> bool {
        std::mem::take(&mut self.irq_rose)
    }
}

#[cfg(test)]
mod tests;
