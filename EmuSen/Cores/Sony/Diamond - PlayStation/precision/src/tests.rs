use super::*;
use diamond::machine::{Config, Gpu as Chip, Machine};
use diamond::media::{Market, Media};
use diamond::gpu::decode::Drawing;
use emusen_native::ffi::StateMachine;

/// The polygons a drawer was given, with their precise vertices.
#[derive(Default)]
struct Log(Vec<(Vec<u32>, Option<[Fraction; 4]>)>);

impl Observer for Log {}

impl Layer for Log {}

impl Draw for Log {
    fn executed_with(&mut self, c: &Command, precise: Option<&[Fraction; 4]>) {
        if c.part == Part::Whole && c.words[0] >> 29 == 1 {
            self.0.push((c.words.to_vec(), precise.copied()));
        }
    }
}

fn machine(source: &str) -> Machine {
    let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = diamond::exe::build(&words);
    Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Chip::Pin208 }, [Some("psx.dualshock"), None], &image, None)
}

/// RTPT of three vertices with an identity rotation, H = 100h and the offset (160, 120): V0 (32, 16, 400h) lands on
/// (168, 124) exactly, V1 (16, -16, 800h) on (162, 118), V2 (-16, 5, 200h) on (152, 122.5). Its three words carried to a
/// list node by MFC2 and SW (`{adjust}` between them), by SWC2, and by SWC2, LW and a copy, the triangle then sent to GP0
/// by the processor's stores and by DMA's list.
fn projected(adjust: &str) -> String {
    format!(
        "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\nlui s0, 0x8010\n\
        ori t0, zero, 0x1000\nctc2 t0, $0\nctc2 t0, $2\nctc2 t0, $4\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\nori t0, zero, 0x100\nctc2 t0, $26\n\
        lui t0, 0x0010\nori t0, t0, 0x0020\nmtc2 t0, $0\nori t0, zero, 0x400\nmtc2 t0, $1\nlui t0, 0xFFF0\nori t0, t0, 0x0010\nmtc2 t0, $2\nori t0, zero, 0x800\nmtc2 t0, $3\n\
        lui t0, 0x0005\nori t0, t0, 0xFFF0\nmtc2 t0, $4\nori t0, zero, 0x200\nmtc2 t0, $5\nnop\n.word 0x4A280030\n\
        mfc2 t1, $12\nnop\n{adjust}\nlui t0, 0x04FF\nori t0, t0, 0xFFFF\nsw t0, 0x100(s0)\nlui t0, 0x20FF\nsw t0, 0x104(s0)\nsw t1, 0x108(s0)\nswc2 $13, 0x10C(s0)\nswc2 $14, 0x110(s0)\n\
        lui t0, 0x20FF\nsw t0, 0x1810(a0)\nlw t2, 0x108(s0)\nnop\nsw t2, 0x1810(a0)\nlw t2, 0x10C(s0)\nnop\nor t3, t2, zero\nsw t3, 0x1810(a0)\nlw t2, 0x110(s0)\nnop\nsw t2, 0x1810(a0)\n\
        lui t0, 0x0400\nori t0, t0, 2\nsw t0, 0x1814(a0)\nlui t0, 0x0765\nori t0, t0, 0x4B21\nsw t0, 0x10F0(a0)\nlui t0, 0x0010\nori t0, t0, 0x0100\nsw t0, 0x10A0(a0)\nsw zero, 0x10A4(a0)\n\
        lui t0, 0x0100\nori t0, t0, 0x0401\nsw t0, 0x10A8(a0)\nloop: j loop\nnop"
    )
}

fn run(source: &str) -> (Vec<(Vec<u32>, Option<[Fraction; 4]>)>, Counts) {
    let mut m = machine(source);
    m.attach(Box::new(Precise::new(Precision::default(), Log::default())));
    for _ in 0..3 {
        m.advance();
    }
    let layer: Box<dyn std::any::Any> = m.detach().expect("attached");
    let p = *layer.downcast::<Precise<Log>>().unwrap();
    (p.drawer.0, p.precision.counts())
}

/// §21.3, §21.4: the triangle sent by the processor's stores, its words carried through registers, a copy and memory,
/// and by DMA from the list node, each given the same precise vertices, the projection's offsets from the console's
/// pixels in 256ths and its depths with 12 fraction bits.
#[test]
fn a_projected_triangle_is_precise_by_the_processor_and_by_dma() {
    let (drawn, counts) = run(&projected(""));
    let want = [Fraction { dx: 0, dy: 0, z: 0x400 << 12 }, Fraction { dx: 0, dy: 0, z: 0x800 << 12 }, Fraction { dx: 0, dy: 128, z: 0x200 << 12 }, Fraction::default()];
    assert_eq!(drawn.len(), 2, "{drawn:?}");
    for (words, precise) in &drawn {
        assert_eq!((words[0], precise.as_ref()), (0x20FF_0000, Some(&want)), "{words:08X?}");
    }
    assert_eq!((counts.projections, counts.refused, counts.polygons, counts.precise_polygons, counts.vertices, counts.precise_vertices), (3, 0, 2, 2, 6, 6));
}

/// §21.3: a word the processor changes by arithmetic keeps its serial but not its integer, and is found changed at the
/// receipt; the polygon, two of its three vertices precise, is given none (§21.4's whole polygons).
#[test]
fn a_vertex_changed_on_the_processor_leaves_its_polygon_integer() {
    let (drawn, counts) = run(&projected("addiu t1, t1, 1"));
    assert!(drawn.iter().all(|d| d.1.is_none()), "{drawn:?}");
    assert_eq!((counts.polygons, counts.precise_polygons, counts.mixed_polygons, counts.changed, counts.precise_vertices), (2, 0, 2, 2, 4));
}

/// A halfword store ends a word's precision, and a byte load a register's.
#[test]
fn a_partial_store_or_load_ends_precision() {
    let mut p = Precision::default();
    let projection = |word: u32| Projection { vertex: 0, sums: [16 << 12, 0, 0x400 << 12], quotient: 0, x: 164 << 16, y: 120 << 16, z: 0x400, word, h: 0x100, offset: [160 << 16, 120 << 16] };
    p.projected(&projection(0x0078_00A4));
    p.moved(Move::StoreCop2 { at: 0, cop2: 14, address: 0x8010_0000, value: 0x0078_00A4 });
    p.moved(Move::StoreCop2 { at: 0, cop2: 14, address: 0x8010_0004, value: 0x0078_00A4 });
    p.moved(Move::Store { at: 0, register: 8, address: 0x8010_0004, size: Size::Half, value: 0 });
    p.moved(Move::Load { at: 0, register: 9, address: 0x8010_0000, size: Size::Byte, value: 0xA4 });
    assert!(p.word(0x0010_0000) != 0 && p.word(0x0010_0004) == 0 && p.regs[9] == 0);
    p.moved(Move::Load { at: 0, register: 9, address: 0x8010_0000, size: Size::Word, value: 0x0078_00A4 });
    let r = p.regs[9];
    assert_eq!(r, p.word(0x0010_0000), "a word load carries it, from a mirror of the address too");
}

/// §21.2's guard: a vertex whose screen position saturated, one behind the camera and one whose console pixel is far
/// from the geometry (IR saturated) are refused; the projection itself is the exact one.
#[test]
fn the_guard_refuses_what_the_console_cut_and_keeps_the_projections_fraction() {
    let p = Precision::default();
    let at = |sums: [i64; 3], x: i64, y: i64| {
        let word = (x >> 16).clamp(-0x400, 0x3FF) as u16 as u32 | ((y >> 16).clamp(-0x400, 0x3FF) as u16 as u32) << 16;
        p.precise(&Projection { vertex: 0, sums, quotient: 0, x, y, z: 0, word, h: 0x100, offset: [160 << 16, 120 << 16] })
    };
    assert_eq!(at([-16 << 12, 5 << 12, 0x200 << 12], 152 << 16, (122 << 16) + 0x8000), Ok((0, 128, 0x200 << 12)));
    assert_eq!(at([(1 << 12) + 0x800, 0, 0x100 << 12], 161 << 16, 120 << 16), Ok((128, 0, 0x100 << 12)), "IR's lost half of a unit, H/SZ3 = 1");
    assert_eq!(at([-5000 << 12, 0, 0x100 << 12], -4840 << 16, 120 << 16), Err(false), "SX saturated");
    assert_eq!(at([16 << 12, 0, -0x100 << 12], 144 << 16, 120 << 16), Err(false), "behind the camera");
    assert_eq!(at([40000 << 12, 0, 0x4000 << 12], (160 + 499) << 16, 120 << 16), Err(true), "the console's pixel 126 from the geometry's");
}

/// §21.3: a serial written over in the ring is dead at the receipt, though the word still holds its integer.
#[test]
fn a_serial_the_ring_has_written_over_is_dead() {
    let mut p = Precision::default();
    let projection = Projection { vertex: 0, sums: [16 << 12, 0, 0x400 << 12], quotient: 0, x: 164 << 16, y: 120 << 16, z: 0x400, word: 0x0078_00A4, h: 0x100, offset: [160 << 16, 120 << 16] };
    p.projected(&projection);
    p.moved(Move::StoreCop2 { at: 0, cop2: 14, address: 0x8010_0004, value: 0x0078_00A4 });
    let drawing = Drawing::default();
    let triangle = [0x2000_0000, 0x0078_00A4, 0x0078_00A4, 0x0078_00A4];
    let send = |p: &mut Precision| {
        for (i, &w) in triangle.iter().enumerate() {
            p.received(w, Receipt::Dma { address: 0x10_0000 + 4 * i as u32 });
        }
        p.resolve(&Command { words: &triangle, part: Part::Whole, at: 0, drawing: &drawing })
    };
    for i in 0..3 {
        p.moved(Move::StoreCop2 { at: 0, cop2: 14, address: 0x8010_0004 + 4 * i, value: 0x0078_00A4 });
    }
    assert!(send(&mut p).is_some(), "live");
    for _ in 0..RING {
        p.projected(&projection);
    }
    assert!(send(&mut p).is_none() && p.counts().dead == 3, "{:?}", p.counts());
}

/// §52.3: the layer writes no block, so a state made with it is the machine's to the byte, and a load starts it empty,
/// whatever it had been told before, as an attach does.
#[test]
fn the_layer_keeps_nothing_in_a_state_and_a_load_starts_it_empty() {
    let mut m = machine(&projected(""));
    let mut plain = machine(&projected(""));
    m.attach(Box::new(Precise::new(Precision::default(), Log::default())));
    for _ in 0..3 {
        m.advance();
        plain.advance();
    }
    let mut state = vec![0; m.state_size()];
    let n = m.save_state(&mut state).unwrap();
    let mut bare = vec![0; plain.state_size()];
    assert_eq!(plain.save_state(&mut bare).unwrap(), n);
    assert!(state[..n] == bare[..n], "the state with the layer is the machine's");
    let layer: &mut dyn std::any::Any = m.layer().unwrap();
    assert!(layer.downcast_mut::<Precise<Log>>().unwrap().precision.serial > 0, "the layer saw projections");
    m.load_state(&state[..n]).unwrap();
    let layer: &mut dyn std::any::Any = m.layer().unwrap();
    let p = &layer.downcast_mut::<Precise<Log>>().unwrap().precision;
    assert!(p.serial == 0 && p.head == 0 && p.ram.iter().all(|&s| s == 0) && p.ring.iter().all(|&w| w == 0), "a load starts the layer empty");
    assert!(p.counts().projections > 0, "its counts describe the run and are kept");
}
