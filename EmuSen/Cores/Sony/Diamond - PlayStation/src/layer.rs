//! The enhancement layer's frame (Diamond_Enhancements.md): what a layer is told by the R3000A, the GTE and GP0, how
//! one is attached to a board, the bus the processor steps through while it is, and the test observer. A layer only
//! watches; the accurate machine never reads it, and with none attached nothing here is on the frame's path.

use std::any::Any;

use blackmoon_gte::{Observer, Projection};
use blackmoon_r3000::{Bus, Move, R3000, Size, Step};
use emusen_mips::Op;
use emusen_native::{StateReader, StateWriter};

use crate::bus::Board;

/// Where a word GP0 received came from (Diamond_Enhancements.md §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receipt {
    /// A store of the processor's to GP0.
    Cpu,
    /// DMA channel 2, from RAM at `address`.
    Dma { address: u32 },
}

/// An enhancement layer: told each value the processor moves, each projection the GTE makes and each word GP0
/// receives, in the order the machine makes them, and its block of the state (Diamond_Enhancements.md §3, §4). Every
/// method has an empty default; none can change the machine.
pub trait Layer: Observer + Any {
    #[inline(always)]
    fn moved(&mut self, _m: Move) {}
    /// A word GP0 is about to take.
    #[inline(always)]
    fn received(&mut self, _word: u32, _from: Receipt) {}
    /// The tag of the layer's block in a state; none for a layer that keeps nothing across one.
    fn tag(&self) -> Option<[u8; 4]> {
        None
    }
    fn save(&self, _w: &mut StateWriter) {}
    /// A loaded state's block of the layer's tag; a layer that cannot read it starts empty.
    fn load(&mut self, _block: &[u8]) {}
    /// A loaded state held no block of the layer's tag: the layer starts empty.
    fn clear(&mut self) {}
}

/// The layer attached to a board, if any. It is no part of the machine's state, so a copy of the board, as the
/// levers' checks make, carries none. It fills one host cache line, so that the board's fields after it keep their
/// places in a line (Diamond_Enhancements.md §9.2).
#[derive(Default)]
pub struct Attached {
    layer: Option<Box<dyn Layer>>,
    _line: [u64; 6],
}

impl Clone for Attached {
    fn clone(&self) -> Attached {
        Attached::default()
    }
}

impl Attached {
    pub fn is_some(&self) -> bool {
        self.layer.is_some()
    }

    #[inline(always)]
    pub(crate) fn get(&mut self) -> Option<&mut dyn Layer> {
        self.layer.as_deref_mut()
    }

    pub(crate) fn get_ref(&self) -> Option<&dyn Layer> {
        self.layer.as_deref()
    }

    pub(crate) fn set(&mut self, layer: Option<Box<dyn Layer>>) -> Option<Box<dyn Layer>> {
        std::mem::replace(&mut self.layer, layer)
    }
}

/// The bus the processor steps through, chosen once a frame: the board itself, or the board with its layer told.
pub(crate) trait Through {
    fn step<O: emusen_native::debug::Observer>(cpu: &mut R3000, board: &mut Board, o: &mut O) -> Step;
    fn run(cpu: &mut R3000, board: &mut Board, limit: u32) -> (u32, Step);
}

/// The plain frame's: the board itself, whose `MOVES` is the processor crate's false.
pub(crate) struct Plain;

impl Through for Plain {
    #[inline(always)]
    fn step<O: emusen_native::debug::Observer>(cpu: &mut R3000, board: &mut Board, o: &mut O) -> Step {
        cpu.step_observed(board, o)
    }
    #[inline(always)]
    fn run(cpu: &mut R3000, board: &mut Board, limit: u32) -> (u32, Step) {
        cpu.run(board, limit)
    }
}

impl Through for Layered {
    #[inline(always)]
    fn step<O: emusen_native::debug::Observer>(cpu: &mut R3000, board: &mut Board, o: &mut O) -> Step {
        cpu.step_observed(&mut LayeredBus(board), o)
    }
    #[inline(always)]
    fn run(cpu: &mut R3000, board: &mut Board, limit: u32) -> (u32, Step) {
        cpu.run(&mut LayeredBus(board), limit)
    }
}

/// The frame's form while a layer is attached (Diamond_Enhancements.md §2.3).
pub(crate) struct Layered;

/// The board as the processor's bus while a layer is attached: the board's own answers, with the processor's moves
/// and the GTE's projections told to the layer.
struct LayeredBus<'a>(&'a mut Board);

impl Bus for LayeredBus<'_> {
    #[inline(always)]
    fn fetch(&mut self, address: u32) -> u32 {
        self.0.fetch(address)
    }
    #[inline(always)]
    fn fetch_decoded(&mut self, address: u32) -> (u32, Op) {
        self.0.fetch_decoded(address)
    }
    #[inline(always)]
    fn read(&mut self, address: u32, size: Size) -> u32 {
        self.0.read(address, size)
    }
    #[inline(always)]
    fn write(&mut self, address: u32, size: Size, value: u32) {
        self.0.write(address, size, value)
    }
    #[inline(always)]
    fn isolated_read(&mut self, address: u32, size: Size) -> u32 {
        self.0.isolated_read(address, size)
    }
    #[inline(always)]
    fn isolated_write(&mut self, address: u32, size: Size, value: u32) {
        self.0.isolated_write(address, size, value)
    }
    #[inline(always)]
    fn idle(&mut self, cycles: u32) {
        self.0.idle(cycles)
    }
    #[inline(always)]
    fn cycles(&mut self) -> u64 {
        self.0.cycles()
    }
    #[inline(always)]
    fn interrupt(&mut self) -> bool {
        self.0.interrupt()
    }
    #[inline(always)]
    fn cop2_data(&mut self, register: u8) -> u32 {
        self.0.cop2_data(register)
    }
    #[inline(always)]
    fn set_cop2_data(&mut self, register: u8, value: u32) {
        self.0.set_cop2_data(register, value)
    }
    #[inline(always)]
    fn cop2_control(&mut self, register: u8) -> u32 {
        self.0.cop2_control(register)
    }
    #[inline(always)]
    fn set_cop2_control(&mut self, register: u8, value: u32) {
        self.0.set_cop2_control(register, value)
    }
    #[inline(always)]
    fn cop2_command(&mut self, command: u32) {
        self.0.cop2_command_observed(command)
    }
    #[inline(always)]
    fn bus_error(&mut self) -> bool {
        self.0.bus_error()
    }
    #[inline(always)]
    fn running(&mut self) -> bool {
        self.0.running()
    }
    const MOVES: bool = true;
    #[inline(always)]
    fn moved(&mut self, m: Move) {
        if let Some(l) = self.0.layer.get() {
            l.moved(m);
        }
    }
}

/// The test observer (Diamond_Enhancements.md §7): a layer that counts what it is told and folds every value into a
/// digest, and keeps its counts across a state. The exactness harness attaches it, and the bench's `--observed` costs
/// it; it does nothing a layer of E4 would not, short of the work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub loads: u64,
    pub stores: u64,
    /// MFC2, MTC2, LWC2 and SWC2.
    pub cop2: u64,
    pub copies: u64,
    pub projections: u64,
    pub from_cpu: u64,
    pub from_dma: u64,
    /// FNV-1a of every value told, in order.
    pub digest: u64,
    /// How many times a state without the tally's block was loaded.
    pub cleared: u64,
}

pub const TALLY_TAG: [u8; 4] = *b"TALY";

impl Tally {
    pub fn new() -> Tally {
        Tally { digest: 0xcbf2_9ce4_8422_2325, ..Tally::default() }
    }

    pub fn total(&self) -> u64 {
        self.loads + self.stores + self.cop2 + self.copies + self.projections + self.from_cpu + self.from_dma
    }

    fn fold(&mut self, words: &[u32]) {
        for &w in words {
            self.digest = (self.digest ^ w as u64).wrapping_mul(0x0000_0100_0000_01B3);
        }
    }

    fn counts(&self) -> [u64; 9] {
        [self.loads, self.stores, self.cop2, self.copies, self.projections, self.from_cpu, self.from_dma, self.digest, self.cleared]
    }
}

impl Observer for Tally {
    fn projected(&mut self, p: &Projection) {
        self.projections += 1;
        let [a, b, c] = p.sums;
        self.fold(&[p.vertex as u32, a as u32, b as u32, c as u32, p.quotient, p.x as u32, p.y as u32, p.z as u32, p.word]);
    }
}

impl Layer for Tally {
    fn moved(&mut self, m: Move) {
        match m {
            Move::Load { at, register, address, size, value } => {
                self.loads += 1;
                self.fold(&[at, register as u32, address, size.bytes(), value]);
            }
            Move::Store { at, register, address, size, value } => {
                self.stores += 1;
                self.fold(&[at, register as u32, address, size.bytes(), value]);
            }
            Move::FromCop2 { at, register, cop2, value } | Move::ToCop2 { at, register, cop2, value } => {
                self.cop2 += 1;
                self.fold(&[at, register as u32, cop2 as u32, value]);
            }
            Move::LoadCop2 { at, cop2, address, value } | Move::StoreCop2 { at, cop2, address, value } => {
                self.cop2 += 1;
                self.fold(&[at, cop2 as u32, address, value]);
            }
            Move::Copy { at, register, from, value } => {
                self.copies += 1;
                self.fold(&[at, register as u32, from as u32, value]);
            }
        }
    }

    fn received(&mut self, word: u32, from: Receipt) {
        match from {
            Receipt::Cpu => self.from_cpu += 1,
            Receipt::Dma { address } => {
                self.from_dma += 1;
                self.fold(&[address]);
            }
        }
        self.fold(&[word]);
    }

    fn tag(&self) -> Option<[u8; 4]> {
        Some(TALLY_TAG)
    }

    fn save(&self, w: &mut StateWriter) {
        w.u64s("Counts", &self.counts());
    }

    fn load(&mut self, block: &[u8]) {
        let mut c = [0u64; 9];
        if StateReader::new(block).u64s(&mut c).is_err() || block.len() != 72 {
            return self.clear();
        }
        [self.loads, self.stores, self.cop2, self.copies, self.projections, self.from_cpu, self.from_dma, self.digest, self.cleared] = c;
    }

    fn clear(&mut self) {
        *self = Tally { cleared: self.cleared + 1, ..Tally::new() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::machine::tests::{machine_for, program};
    use crate::media::Market;
    use emusen_native::ffi::{StateMachine, Status};

    /// Everything a layer is told, in order.
    #[derive(Debug, Clone, PartialEq)]
    enum Told {
        Moved(Move),
        Projected(Projection),
        Received(u32, Receipt),
    }

    #[derive(Default)]
    struct Log(Vec<Told>);

    impl Observer for Log {
        fn projected(&mut self, p: &Projection) {
            self.0.push(Told::Projected(*p));
        }
    }

    impl Layer for Log {
        fn moved(&mut self, m: Move) {
            self.0.push(Told::Moved(m));
        }
        fn received(&mut self, word: u32, from: Receipt) {
            self.0.push(Told::Received(word, from));
        }
    }

    /// RTPT, its three screen words carried to a list node in RAM by MFC2 and SW, by SWC2, and by LW and a copy, the
    /// same triangle sent to GP0 by stores and then by DMA channel 2's list.
    const PROJECTED: &str = "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\nlui s0, 0x8010\n\
        ori t0, zero, 0x1000\nctc2 t0, $0\nctc2 t0, $2\nctc2 t0, $4\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\nori t0, zero, 0x100\nctc2 t0, $26\n\
        lui t0, 0x0010\nori t0, t0, 0x0020\nmtc2 t0, $0\nori t0, zero, 0x400\nmtc2 t0, $1\nlui t0, 0xFFF0\nori t0, t0, 0x0010\nmtc2 t0, $2\nori t0, zero, 0x800\nmtc2 t0, $3\n\
        lui t0, 0x0005\nori t0, t0, 0xFFF0\nmtc2 t0, $4\nori t0, zero, 0x200\nmtc2 t0, $5\nnop\n.word 0x4A280030\n\
        mfc2 t1, $12\nlui t0, 0x04FF\nori t0, t0, 0xFFFF\nsw t0, 0x100(s0)\nlui t0, 0x20FF\nsw t0, 0x104(s0)\nsw t1, 0x108(s0)\nswc2 $13, 0x10C(s0)\nswc2 $14, 0x110(s0)\n\
        lui t0, 0x20FF\nsw t0, 0x1810(a0)\nlw t2, 0x108(s0)\nnop\nsw t2, 0x1810(a0)\nlw t2, 0x10C(s0)\nnop\nor t3, t2, zero\nsw t3, 0x1810(a0)\nlw t2, 0x110(s0)\nnop\nsw t2, 0x1810(a0)\n\
        lui t0, 0x0400\nori t0, t0, 2\nsw t0, 0x1814(a0)\nlui t0, 0x0765\nori t0, t0, 0x4B21\nsw t0, 0x10F0(a0)\nlui t0, 0x0010\nori t0, t0, 0x0100\nsw t0, 0x10A0(a0)\nsw zero, 0x10A4(a0)\n\
        lui t0, 0x0100\nori t0, t0, 0x0401\nsw t0, 0x10A8(a0)\nloop: j loop\nnop";

    fn state(m: &Machine) -> Vec<u8> {
        let mut s = vec![0; m.state_size()];
        let n = m.save_state(&mut s).unwrap();
        s.truncate(n);
        s
    }

    fn logged(m: &mut Machine) -> Vec<Told> {
        let layer: Box<dyn Any> = m.detach().expect("attached");
        layer.downcast::<Log>().unwrap().0
    }

    /// Diamond_Enhancements.md §3: the three projections with the words the FIFO kept, each move that carries a word to
    /// the node and to GP0 with its register, address and value, each word GP0 takes with where it came from, in the
    /// order the machine made them; and the machine the same, to the state's last byte, as with no layer.
    #[test]
    fn a_layer_is_told_each_projection_move_and_receipt_with_its_values() {
        let image = program(PROJECTED);
        let mut plain = machine_for(&image, Market::Americas);
        let mut m = machine_for(&image, Market::Americas);
        m.attach(Box::new(Log::default()));
        for _ in 0..3 {
            plain.advance();
            m.advance();
        }
        let sxy = [12, 13, 14].map(|r| m.board.gte.data(r));
        let told = logged(&mut m);
        assert!(state(&m) == state(&plain), "the layer changed the machine");

        let projections: Vec<Projection> = told.iter().filter_map(|t| if let Told::Projected(p) = t { Some(*p) } else { None }).collect();
        assert_eq!(projections.iter().map(|p| (p.vertex, p.word)).collect::<Vec<_>>(), [(0, sxy[0]), (1, sxy[1]), (2, sxy[2])]);
        assert_eq!(projections[0].sums, [32 << 12, 16 << 12, 0x400 << 12]);

        let moved = |f: &dyn Fn(&Move) -> bool| told.iter().position(|t| matches!(t, Told::Moved(m) if f(m))).unwrap_or_else(|| panic!("not told"));
        let node = 0x8010_0100;
        let from_cop2 = moved(&|m| matches!(m, Move::FromCop2 { register: 9, cop2: 12, value, .. } if *value == sxy[0]));
        let stored = moved(&|m| matches!(m, Move::Store { register: 9, address, size: Size::Word, value, .. } if *address == node + 8 && *value == sxy[0]));
        let swc2 = moved(&|m| matches!(m, Move::StoreCop2 { cop2: 13, address, value, .. } if *address == node + 12 && *value == sxy[1]));
        let loaded = moved(&|m| matches!(m, Move::Load { register: 10, address, value, .. } if *address == node + 12 && *value == sxy[1]));
        let copy = moved(&|m| matches!(m, Move::Copy { register: 11, from: 10, value, .. } if *value == sxy[1]));
        let to_gp0 = moved(&|m| matches!(m, Move::Store { register: 11, address: 0x1F80_1810, value, .. } if *value == sxy[1]));
        let last_projection = told.iter().rposition(|t| matches!(t, Told::Projected(_))).unwrap();
        assert!(last_projection < from_cop2 && from_cop2 < stored && stored < swc2 && swc2 < loaded && loaded < copy && copy < to_gp0, "in the order made");
        assert!(told.iter().any(|t| matches!(t, Told::Moved(Move::ToCop2 { register: 8, cop2: 5, value: 0x200, .. }))));

        let received: Vec<(u32, Receipt)> = told.iter().filter_map(|t| if let Told::Received(w, r) = t { Some((*w, *r)) } else { None }).collect();
        let triangle = [0x20FF_0000, sxy[0], sxy[1], sxy[2]];
        let from_cpu: Vec<u32> = received.iter().filter(|r| r.1 == Receipt::Cpu).map(|r| r.0).collect();
        assert_eq!(from_cpu[..4], triangle, "the stores to GP0, the GP1 setup aside");
        let by_dma: Vec<(u32, Receipt)> = received.iter().filter(|r| r.1 != Receipt::Cpu).copied().collect();
        assert_eq!(by_dma, (0..4).map(|i| (triangle[i], Receipt::Dma { address: 0x10_0104 + 4 * i as u32 })).collect::<Vec<_>>(), "the node's words with their addresses");
        let gp0_store = told.iter().position(|t| matches!(t, Told::Moved(Move::Store { register: 11, address: 0x1F80_1810, .. }))).unwrap();
        assert!(matches!(told[gp0_store - 1], Told::Received(w, Receipt::Cpu) if w == sxy[1]), "a word is received before the store that sent it is told");
    }

    /// Diamond_Enhancements.md §4: a state made with a layer's block loads on a machine without the layer, the block
    /// skipped, and continues as the machine that saved it; a state made without loads with the layer, which starts
    /// empty; one made with the block loads the layer's counts back. The bytes before the block are the state without it.
    #[test]
    fn a_layers_block_is_skipped_without_it_and_a_state_without_one_clears_it() {
        let image = program(PROJECTED);
        let mut a = machine_for(&image, Market::Americas);
        a.attach(Box::new(Tally::new()));
        a.advance();
        let with = state(&a);
        let counts = *(a.detach().unwrap() as Box<dyn Any>).downcast::<Tally>().unwrap();
        let without = state(&a);
        assert_eq!((counts.loads, counts.stores, counts.cop2, counts.copies, counts.projections, counts.from_cpu, counts.from_dma), (3, 12, 9, 1, 3, 4, 4), "{counts:?}");
        assert_eq!(&with[..without.len()], &without[..], "the accurate blocks are the same bytes");
        assert_eq!(with.len(), without.len() + 8 + 72);
        assert_eq!(&with[without.len()..without.len() + 8], &[b'T', b'A', b'L', b'Y', 72, 0, 0, 0]);
        let layout = a.layout();
        a.attach(Box::new(Tally::new()));
        assert!(a.layout().starts_with(&layout) && a.layout().ends_with("TALY.Tag\n5267536 4 u32 TALY.Length\n5267540 72 u64[9] TALY.Counts\n"), "{}", a.layout());
        a.detach();

        let mut b = machine_for(&image, Market::Americas);
        b.load_state(&with).unwrap();
        a.advance();
        b.advance();
        assert!(state(&b) == state(&a), "loaded without the layer, the block skipped, it continues as the first");

        let mut c = machine_for(&image, Market::Americas);
        c.attach(Box::new(Tally { loads: 7, ..Tally::new() }));
        c.load_state(&without).unwrap();
        let cleared = *(c.detach().unwrap() as Box<dyn Any>).downcast::<Tally>().unwrap();
        assert_eq!(cleared, Tally { cleared: 1, ..Tally::new() }, "a state without the block: the layer starts empty");

        c.attach(Box::new(Tally::new()));
        c.load_state(&with).unwrap();
        let restored = *(c.detach().unwrap() as Box<dyn Any>).downcast::<Tally>().unwrap();
        assert_eq!(restored, counts, "the block read back");

        let mut unknown = without.clone();
        unknown.extend_from_slice(b"ZZZZ\x03\x00\x00\x00abc");
        unknown.extend_from_slice(&with[without.len()..]);
        c.attach(Box::new(Tally::new()));
        c.load_state(&unknown).unwrap();
        assert_eq!(*(c.detach().unwrap() as Box<dyn Any>).downcast::<Tally>().unwrap(), counts, "an unknown tag skipped by its length");
        let before = state(&c);
        for cut in [without.len() + 3, with.len() - 1] {
            assert_eq!(c.load_state(&with[..cut]).unwrap_err().status(), emusen_native::ffi::status::TRUNCATED, "a block cut short at {cut}");
        }
        assert!(state(&c) == before, "and changes nothing");
    }
}
