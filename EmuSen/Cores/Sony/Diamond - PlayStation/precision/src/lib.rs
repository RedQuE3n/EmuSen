//! Diamond's precision layer (stage E4, Diamond_Enhancements.md §21): told each projection, each value the processor
//! moves and each word GP0 receives, it keeps beside every place that can hold a projected word (the GTE's SXY FIFO,
//! the general registers, RAM and the scratchpad) the serial of the projection the word came from, and when the GPU
//! starts a polygon it finds the polygon's words among the receipts and gives its vertices' sub-pixel offsets and
//! depths, if every vertex is still the integer its projection made. `Precise` pairs it with whoever draws.

use blackmoon_gte::{Observer, Projection};
use blackmoon_r3000::{Move, Size};
use diamond::gpu::Gpu;
use diamond::layer::{Command, Layer, Part, Receipt, Shown};
use emusen_native::{StateReader, StateWriter};

/// The projections kept, the receipts kept, and the words of RAM and of the scratchpad shadowed.
pub const RING: usize = 32 * 1024;
pub const QUEUE: usize = 64;
const RAM_WORDS: usize = 1 << 19;
const SCRATCH_WORDS: usize = 256;
const GP0: u32 = 0x1F80_1810;

/// A precise position is used within this many pixels of the centre of the pixel the console chose (§21.2).
pub const GUARD: f64 = 2.0;
/// The offsets' unit: 256ths of a pixel, the renderer's sub-pixel grid (§22.2).
pub const SUBPIXEL: f64 = 256.0;

pub const TAG: [u8; 4] = *b"PREC";

/// A precise vertex as the drawer is given it: its offset from the integer vertex the GPU decodes, in 256ths of a
/// pixel, and its depth in the camera's space with 12 fraction bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fraction {
    pub dx: i32,
    pub dy: i32,
    pub z: i32,
}

/// How a projection is made precise: the projection without its truncations, the one chosen (§21.2); MAC0's sums with
/// their 16 fraction bits, the alternative it was measured against; or every offset zero at one depth, the form the
/// zero-fraction gate draws through the precise record (§23).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Form {
    #[default]
    Exact,
    Sums,
    Zero,
}

/// What the layer found, for its tools and the gate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub projections: u64,
    /// Projections the guard refused: those the console cut (saturated, or at or behind the camera), and those whose
    /// precise position lies farther than `GUARD` from the console's pixel.
    pub refused: u64,
    pub far: u64,
    pub polygons: u64,
    /// Polygons drawn with every vertex precise, and those with some but not all.
    pub precise_polygons: u64,
    pub mixed_polygons: u64,
    pub vertices: u64,
    pub precise_vertices: u64,
    /// Vertex words whose serial had been written over in the ring, or whose integer was no longer the projection's.
    pub dead: u64,
    pub changed: u64,
    /// Commands whose words were not among the receipts (received before the layer was attached).
    pub unmatched: u64,
}

/// Whoever draws with the precise vertices: told each command as `Layer::executed` would be, with a polygon's
/// vertices when every one is precise, in the command's order.
pub trait Draw: Layer {
    fn executed_with(&mut self, c: &Command, precise: Option<&[Fraction; 4]>);
}

pub struct Precision {
    form: Form,
    /// The last serial given; serials start at 1, 0 meaning none.
    serial: u32,
    /// Four words a projection, at its serial's place: the serial, the integer word, the offsets (x low, y high), the depth.
    ring: Box<[u32]>,
    ram: Box<[u32]>,
    scratch: Box<[u32]>,
    regs: [u32; 32],
    fifo: [u32; 3],
    /// The serial of the processor's next word to GP0, from the store told before it (§21.4).
    cpu: u32,
    /// Each receipt's word and serial, `head` the next place and `cursor` the first not used by a command.
    queue: [u32; 2 * QUEUE],
    head: u32,
    cursor: u32,
    counts: Counts,
}

impl Default for Precision {
    fn default() -> Precision {
        Precision::new(Form::Exact)
    }
}

/// The places a polygon command's vertex words have among its words, and how many vertices it has.
pub fn vertex_words(command: u32) -> ([usize; 4], usize) {
    let c = command >> 24;
    let (n, shaded, textured) = (if c & 8 != 0 { 4 } else { 3 }, c & 0x10 != 0, c & 4 != 0);
    let (mut at, mut k) = (1, [0; 4]);
    for (i, place) in k.iter_mut().enumerate().take(n) {
        if i > 0 && shaded {
            at += 1;
        }
        *place = at;
        at += if textured { 2 } else { 1 };
    }
    (k, n)
}

impl Precision {
    pub fn new(form: Form) -> Precision {
        Precision {
            form,
            serial: 0,
            ring: vec![0; RING * 4].into_boxed_slice(),
            ram: vec![0; RAM_WORDS].into_boxed_slice(),
            scratch: vec![0; SCRATCH_WORDS].into_boxed_slice(),
            regs: [0; 32],
            fifo: [0; 3],
            cpu: 0,
            queue: [0; 2 * QUEUE],
            head: 0,
            cursor: 0,
            counts: Counts::default(),
        }
    }

    pub fn counts(&self) -> Counts {
        self.counts
    }

    /// The precise offsets and depth of a projection, or why the guard refuses it (§21.2): `Err(false)` for what the
    /// console cut, `Err(true)` for a position far from the console's.
    fn precise(&self, p: &Projection) -> Result<(i32, i32, i32), bool> {
        let z = p.sums[2];
        let (sx, sy) = (p.word as u16 as i16 as i64, (p.word >> 16) as u16 as i16 as i64);
        if z <= 0 || z > i32::MAX as i64 || p.x >> 16 != sx || p.y >> 16 != sy {
            return Err(false);
        }
        let (fx, fy) = match self.form {
            Form::Zero => return Ok((0, 0, 1 << 12)),
            Form::Sums => ((p.x & 0xFFFF) as f64 / 65536.0, (p.y & 0xFFFF) as f64 / 65536.0),
            Form::Exact => {
                let h = p.h as f64;
                let x = p.offset[0] as f64 / 65536.0 + h * p.sums[0] as f64 / z as f64;
                let y = p.offset[1] as f64 / 65536.0 + h * p.sums[1] as f64 / z as f64;
                (x - sx as f64, y - sy as f64)
            }
        };
        if (fx - 0.5).abs() > GUARD || (fy - 0.5).abs() > GUARD {
            return Err(true);
        }
        Ok(((fx * SUBPIXEL).round() as i32, (fy * SUBPIXEL).round() as i32, z as i32))
    }

    /// The shadow of the memory word at `address`, for RAM and the scratchpad.
    fn slot(&mut self, address: u32) -> Option<&mut u32> {
        let pa = address & 0x1FFF_FFFF;
        if pa < 0x0080_0000 {
            Some(&mut self.ram[(pa as usize & 0x1F_FFFF) >> 2])
        } else if (0x1F80_0000..0x1F80_0400).contains(&pa) {
            Some(&mut self.scratch[(pa as usize & 0x3FF) >> 2])
        } else {
            None
        }
    }

    fn word(&mut self, address: u32) -> u32 {
        self.slot(address).map_or(0, |s| *s)
    }

    /// A store's serial to memory, or to the processor's next receipt when the store is to GP0.
    fn store(&mut self, address: u32, serial: u32) {
        if address & 0x1FFF_FFFF == GP0 {
            self.cpu = serial;
        } else if let Some(s) = self.slot(address) {
            *s = serial;
        }
    }

    fn sxy(&self, cop2: u8) -> u32 {
        match cop2 {
            12..=14 => self.fifo[cop2 as usize - 12],
            15 => self.fifo[2],
            _ => 0,
        }
    }

    fn set_sxy(&mut self, cop2: u8, serial: u32) {
        match cop2 {
            12..=14 => self.fifo[cop2 as usize - 12] = serial,
            15 => self.fifo = [self.fifo[1], self.fifo[2], serial],
            _ => {}
        }
    }

    /// The first run of receipts from the cursor whose words are `words`, the cursor moved past it.
    fn find(&mut self, words: &[u32]) -> Option<u32> {
        let n = words.len() as u32;
        let mut p = self.cursor;
        while self.head.wrapping_sub(p) >= n && self.head.wrapping_sub(p) <= QUEUE as u32 {
            if (0..n).all(|i| self.queue[(p.wrapping_add(i) as usize % QUEUE) * 2] == words[i as usize]) {
                self.cursor = p.wrapping_add(n);
                return Some(p);
            }
            p = p.wrapping_add(1);
        }
        None
    }

    /// The precise vertices of the command the GPU starts, when it is a polygon whose every vertex is precise; and the
    /// receipts it used passed over (§21.4).
    pub fn resolve(&mut self, c: &Command) -> Option<[Fraction; 4]> {
        match c.part {
            Part::Vertex => {
                self.find(&c.words[1..2]);
                return None;
            }
            Part::Data => {
                self.find(&c.words[..1]);
                return None;
            }
            Part::Whole => {}
        }
        let Some(at) = self.find(c.words) else {
            self.counts.unmatched += 1;
            return None;
        };
        if c.words[0] >> 29 != 1 {
            return None;
        }
        let (k, n) = vertex_words(c.words[0]);
        let mut out = [Fraction::default(); 4];
        let mut precise = 0;
        for i in 0..n {
            let serial = self.queue[(at.wrapping_add(k[i] as u32) as usize % QUEUE) * 2 + 1];
            if serial == 0 {
                continue;
            }
            let e = &self.ring[(serial as usize % RING) * 4..][..4];
            if e[0] != serial {
                self.counts.dead += 1;
            } else if e[1] != c.words[k[i]] {
                self.counts.changed += 1;
            } else {
                out[i] = Fraction { dx: e[2] as i16 as i32, dy: (e[2] >> 16) as i16 as i32, z: e[3] as i32 };
                precise += 1;
            }
        }
        let counts = &mut self.counts;
        counts.polygons += 1;
        counts.vertices += n as u64;
        counts.precise_vertices += precise as u64;
        if precise == n {
            counts.precise_polygons += 1;
            Some(out)
        } else {
            counts.mixed_polygons += (precise > 0) as u64;
            None
        }
    }

    fn reset(&mut self) {
        let counts = self.counts;
        *self = Precision { counts, ..Precision::new(self.form) };
    }
}

impl Observer for Precision {
    fn projected(&mut self, p: &Projection) {
        self.counts.projections += 1;
        let serial = match self.precise(p) {
            Ok((dx, dy, z)) => {
                self.serial = self.serial.wrapping_add(1).max(1);
                let at = (self.serial as usize % RING) * 4;
                self.ring[at..at + 4].copy_from_slice(&[self.serial, p.word, (dx as u16 as u32) | (dy as u16 as u32) << 16, z as u32]);
                self.serial
            }
            Err(far) => {
                self.counts.refused += !far as u64;
                self.counts.far += far as u64;
                0
            }
        };
        self.fifo = [self.fifo[1], self.fifo[2], serial];
    }
}

impl Layer for Precision {
    fn moved(&mut self, m: Move) {
        match m {
            Move::Load { register, address, size, .. } => {
                self.regs[register as usize] = if size == Size::Word { self.word(address) } else { 0 };
            }
            Move::Store { register, address, size, .. } => {
                let serial = if size == Size::Word { self.regs[register as usize] } else { 0 };
                self.store(address, serial);
            }
            Move::FromCop2 { register, cop2, .. } => self.regs[register as usize] = self.sxy(cop2),
            Move::ToCop2 { register, cop2, .. } => {
                let serial = self.regs[register as usize];
                self.set_sxy(cop2, serial);
            }
            Move::LoadCop2 { cop2, address, .. } => {
                let serial = self.word(address);
                self.set_sxy(cop2, serial);
            }
            Move::StoreCop2 { cop2, address, .. } => {
                let serial = self.sxy(cop2);
                self.store(address, serial);
            }
            Move::Copy { register, from, .. } | Move::Shift { register, from, .. } => self.regs[register as usize] = self.regs[from as usize],
        }
    }

    fn received(&mut self, word: u32, from: Receipt) {
        let serial = match from {
            Receipt::Cpu => std::mem::take(&mut self.cpu),
            Receipt::Dma { address } => self.word(address),
        };
        let at = (self.head as usize % QUEUE) * 2;
        self.queue[at..at + 2].copy_from_slice(&[word, serial]);
        self.head = self.head.wrapping_add(1);
        if self.head.wrapping_sub(self.cursor) > QUEUE as u32 {
            self.cursor = self.head.wrapping_sub(QUEUE as u32);
        }
    }

    fn executed(&mut self, c: &Command) {
        self.resolve(c);
    }

    fn tags(&self) -> Vec<[u8; 4]> {
        vec![TAG]
    }

    fn save(&self, _tag: [u8; 4], w: &mut StateWriter) {
        w.u32s("Serials", &[self.serial, self.cpu, self.head, self.cursor]);
        w.u32s("Registers", &self.regs);
        w.u32s("Fifo", &self.fifo);
        w.u32s("Receipts", &self.queue);
        w.u32s("Ring", &self.ring);
        w.u32s("RAM", &self.ram);
        w.u32s("Scratch", &self.scratch);
    }

    fn load(&mut self, tag: [u8; 4], block: &[u8]) {
        let mut next = Precision { counts: self.counts, ..Precision::new(self.form) };
        let mut r = StateReader::new(block);
        let mut serials = [0u32; 4];
        let read = (|| {
            r.u32s(&mut serials)?;
            r.u32s(&mut next.regs)?;
            r.u32s(&mut next.fifo)?;
            r.u32s(&mut next.queue)?;
            r.u32s(&mut next.ring)?;
            r.u32s(&mut next.ram)?;
            r.u32s(&mut next.scratch)
        })();
        if read.is_err() || r.position() != block.len() {
            return self.clear(tag);
        }
        [next.serial, next.cpu, next.head, next.cursor] = serials;
        *self = next;
    }

    fn clear(&mut self, _tag: [u8; 4]) {
        self.reset();
    }
}

/// The precision layer and whoever draws, attached as one layer: every event told to the precision layer first, and
/// each command then to the drawer with its polygon's precise vertices (Diamond_Enhancements.md §24.1).
pub struct Precise<D> {
    pub precision: Precision,
    pub drawer: D,
}

impl<D: Draw> Precise<D> {
    pub fn new(precision: Precision, drawer: D) -> Precise<D> {
        Precise { precision, drawer }
    }
}

impl<D: Draw> Observer for Precise<D> {
    fn projected(&mut self, p: &Projection) {
        self.precision.projected(p);
        self.drawer.projected(p);
    }
}

impl<D: Draw> Layer for Precise<D> {
    fn moved(&mut self, m: Move) {
        self.precision.moved(m);
    }

    fn received(&mut self, word: u32, from: Receipt) {
        self.precision.received(word, from);
        self.drawer.received(word, from);
    }

    fn executed(&mut self, c: &Command) {
        let v = self.precision.resolve(c);
        self.drawer.executed_with(c, v.as_ref());
    }

    fn shown(&mut self, s: Shown) {
        self.drawer.shown(s);
    }

    fn seed(&mut self, gpu: &Gpu) {
        self.precision.seed(gpu);
        self.drawer.seed(gpu);
    }

    fn tags(&self) -> Vec<[u8; 4]> {
        let mut t = vec![TAG];
        t.extend(self.drawer.tags());
        t
    }

    fn save(&self, tag: [u8; 4], w: &mut StateWriter) {
        if tag == TAG { self.precision.save(tag, w) } else { self.drawer.save(tag, w) }
    }

    fn load(&mut self, tag: [u8; 4], block: &[u8]) {
        if tag == TAG { self.precision.load(tag, block) } else { self.drawer.load(tag, block) }
    }

    fn clear(&mut self, tag: [u8; 4]) {
        if tag == TAG { self.precision.clear(tag) } else { self.drawer.clear(tag) }
    }

    fn settle(&mut self) {
        self.drawer.settle();
    }
}

#[cfg(test)]
mod tests;
