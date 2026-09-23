//! The vector unit's SIMD path against the element-by-element unit it sits beside, which is its oracle: random programs weighted
//! toward the vector unit with operands, accumulators and flags drawn from the edges, compared after every step and as whole machines
//! at the end; and every function at every selector, one instruction at a time. See Mars_Native.md §6.10.

use super::blocks::Rng;
use crate::machine::Machine;
use crate::memory::bus::RDRAM_SIZE;
use crate::memory::sp::Rsp;
use crate::rsp::simd_supported;

/// The values the clamps, carries and sign tests turn on.
const EDGES: [u16; 12] = [0, 1, 2, 0x7FFE, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF, 0x00FF, 0xFF00, 0x0100];

fn edge(r: &mut Rng) -> u16 {
    if r.below(2) == 0 { r.bits(16) as u16 } else { EDGES[r.below(EDGES.len() as u64) as usize] }
}

/// A word of every class the processor decodes, weighted toward the vector unit and its loads and stores.
fn instruction(r: &mut Rng) -> u32 {
    match r.below(100) {
        0..40 => 0x4A00_0000 | r.bits(4) << 21 | r.bits(5) << 16 | r.bits(5) << 11 | r.bits(5) << 6 | r.bits(6),
        40..58 => (if r.below(2) == 0 { 0x32 } else { 0x3A }) << 26 | r.bits(5) << 21 | r.bits(5) << 16 | r.bits(4) << 11 | r.bits(4) << 7 | r.bits(7),
        58..64 => 0x4800_0000 | (r.below(4) as u32 * 2) << 21 | r.bits(5) << 16 | r.bits(5) << 11 | r.bits(4) << 7,
        64..74 => {
            let immediate = if r.below(2) == 0 { r.bits(16) } else { r.range(-8, 8) as i16 as u16 as u32 };
            (r.range(0x08, 0x10) as u32) << 26 | r.bits(5) << 21 | r.bits(5) << 16 | immediate
        }
        74..82 => {
            const FUNCTIONS: [u32; 18] = [0, 2, 3, 4, 6, 7, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B, 0x08, 0x09];
            r.bits(5) << 21 | r.bits(5) << 16 | r.bits(5) << 11 | r.bits(5) << 6 | FUNCTIONS[r.below(18) as usize]
        }
        82..88 => {
            const OPS: [u32; 9] = [0x20, 0x21, 0x23, 0x24, 0x25, 0x27, 0x28, 0x29, 0x2B];
            OPS[r.below(9) as usize] << 26 | r.bits(5) << 21 | r.bits(5) << 16 | r.bits(16)
        }
        88..93 => (r.range(1, 8) as u32) << 26 | r.bits(5) << 21 | r.bits(5) << 16 | r.bits(6),
        93..95 => 0x0000_000D,
        95..97 => 0x4000_0000 | r.bits(5) << 16 | r.bits(3) << 11,
        _ => r.u64() as u32,
    }
}

/// A scalar register: a third random, a third small or at the sign, a third an address near an alignment or the end of DMEM.
fn scalar(r: &mut Rng) -> u32 {
    match r.below(3) {
        0 => r.u64() as u32,
        1 => (r.range(-8, 8) as i32 as u32).wrapping_add(r.below(2) as u32 * 0x7FFF_FFF8),
        _ => {
            let near = [0x000, 0x010, 0x7F0, 0x800, 0xFE0, 0xFF0, 0xFF8, 0x1000][r.below(8) as usize];
            (near as u32).wrapping_add(r.range(-17, 18) as i32 as u32) & 0xFFF
        }
    }
}

/// The processor's whole state drawn at random, the vector registers, the accumulator's thirds and the flags from the edges.
fn randomise(m: &mut Machine, r: &mut Rng) {
    for at in (0..0x1000).step_by(4) {
        m.bus.sp_imem[at..at + 4].copy_from_slice(&instruction(r).to_be_bytes());
    }
    for byte in m.bus.sp_dmem.iter_mut() {
        *byte = if r.below(4) == 0 { [0x00, 0x7F, 0x80, 0xFF][r.below(4) as usize] } else { r.bits(8) as u8 };
    }
    let p = &mut m.bus.sp.processor;
    for i in 1..32 {
        p.gpr[i] = scalar(r);
    }
    for register in p.vector.iter_mut() {
        for e in register.iter_mut() {
            *e = edge(r);
        }
    }
    for third in p.accumulator.iter_mut() {
        for e in third.iter_mut() {
            *e = edge(r);
        }
    }
    // A state no machine wrote can carry bits above 47; a round in four has them, which a whole write clears and a low write keeps.
    let noisy = r.below(4) == 0;
    for e in p.accumulator_top.iter_mut() {
        *e = if noisy { r.bits(16) as u16 } else { 0 };
    }
    let flags = |r: &mut Rng| if r.below(3) == 0 { [0, 0xFFFF, 0x00FF, 0xFF00][r.below(4) as usize] } else { r.bits(16) as u16 };
    p.vco = flags(r);
    p.vcc = flags(r);
    p.vce = r.bits(8) as u8;
    p.divide_input = edge(r);
    p.divide_output = edge(r);
    p.divide_input_loaded = r.below(2) == 0;
    p.start(r.bits(10) * 4);
    p.broke = false;
}

/// Where two processors part, in words a reader can act on.
fn parting(want: &Rsp, got: &Rsp, want_dmem: &[u8], got_dmem: &[u8]) -> String {
    let mut out = Vec::new();
    for (i, (a, b)) in want.vector.iter().zip(&got.vector).enumerate() {
        if a != b {
            out.push(format!("V{i} {a:04X?} against {b:04X?}"));
        }
    }
    for (k, name) in ["high", "middle", "low"].iter().enumerate() {
        if want.accumulator[k] != got.accumulator[k] {
            out.push(format!("accumulator {name} {:04X?} against {:04X?}", want.accumulator[k], got.accumulator[k]));
        }
    }
    if want.accumulator_top != got.accumulator_top {
        out.push(format!("accumulator bits 63:48 {:04X?} against {:04X?}", want.accumulator_top, got.accumulator_top));
    }
    let flags = |p: &Rsp| (p.vco, p.vcc, p.vce, p.divide_input, p.divide_output, p.divide_input_loaded);
    if flags(want) != flags(got) {
        out.push(format!("VCO, VCC, VCE and the divide unit {:04X?} against {:04X?}", flags(want), flags(got)));
    }
    if want.gpr != got.gpr || (want.pc, want.next_pc, want.halted, want.broke) != (got.pc, got.next_pc, got.halted, got.broke) {
        out.push("the scalar half".into());
    }
    if let Some(at) = want_dmem.iter().zip(got_dmem).position(|(a, b)| a != b) {
        out.push(format!("DMEM from {at:03X}"));
    }
    out.join("; ")
}

fn machines() -> Option<(Machine, Machine)> {
    if !simd_supported() {
        eprintln!("this host has no SSSE3 and SSE4.1: the SIMD path cannot run, not run");
        return None;
    }
    let (mut plain, mut simd) = (Machine::new(RDRAM_SIZE).unwrap(), Machine::new(RDRAM_SIZE).unwrap());
    plain.set_rsp_simd(false);
    simd.set_rsp_simd(true);
    assert!(*simd.bus.sp.processor.simd && !*plain.bus.sp.processor.simd);
    Some((plain, simd))
}

fn copy_processor(from: &Machine, to: &mut Machine) {
    let on = to.bus.sp.processor.simd;
    to.bus.sp.processor = from.bus.sp.processor.clone();
    to.bus.sp.processor.simd = on;
    *to.bus.sp_imem = *from.bus.sp_imem;
    *to.bus.sp_dmem = *from.bus.sp_dmem;
}

fn seeds() -> u64 {
    std::env::var("EMUSEN_MARSRT_RSP_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(150)
}

/// 150 seeds of 4,000 steps, restarted at every break as C#'s `MarsNativeRspTests` restarts them, the two processors compared after
/// every step and the two machines' states at the end.
#[test]
fn the_simd_vector_unit_leaves_the_state_the_element_by_element_one_leaves() {
    let Some((mut plain, mut simd)) = machines() else { return };
    const STEPS: u64 = 4000;
    let (mut vector_ops, mut transfers, mut restarts) = (0u64, 0u64, 0u64);
    for seed in 1..=seeds() {
        let mut r = Rng::new(seed);
        randomise(&mut plain, &mut r);
        copy_processor(&plain, &mut simd);
        for step in 0..STEPS {
            for m in [&mut plain, &mut simd] {
                if m.bus.sp.processor.halted {
                    let pc = m.bus.sp.processor.pc;
                    m.bus.sp.processor.start(pc);
                    m.bus.sp.processor.broke = false;
                }
            }
            let pc = plain.bus.sp.processor.pc as usize;
            let word = u32::from_be_bytes(plain.bus.sp_imem[pc..pc + 4].try_into().unwrap());
            match word >> 26 {
                0x12 if word & (1 << 25) != 0 => vector_ops += 1,
                0x32 | 0x3A => transfers += 1,
                0x00 if word & 0x3F == 0x0D => restarts += 1,
                _ => {}
            }
            plain.bus.sp_step(1);
            simd.bus.sp_step(1);
            let (a, b) = (&plain.bus.sp.processor, &simd.bus.sp.processor);
            if a != b || plain.bus.sp_dmem != simd.bus.sp_dmem {
                panic!("seed {seed}, step {step}, {word:08X} at {pc:03X}: {}", parting(a, b, &plain.bus.sp_dmem[..], &simd.bus.sp_dmem[..]));
            }
        }
        plain.settle();
        simd.settle();
        assert!(plain.save_state_vec(false).unwrap() == simd.save_state_vec(false).unwrap(), "seed {seed}: the machines' states part");
    }
    eprintln!("{} seeds of {STEPS} steps identical: {vector_ops} vector operations, {transfers} vector loads and stores, {restarts} breaks", seeds());
}

/// Every one of the 64 functions at every one of the 16 selectors, twelve rounds each, from fresh edge-drawn registers: a round in
/// three names one register twice or three times, where reading before writing shows (Mars_RspVector.md §2).
#[test]
fn every_function_at_every_selector_agrees_one_instruction_at_a_time() {
    let Some((mut plain, mut simd)) = machines() else { return };
    let mut r = Rng::new(0x5EED);
    let mut cases = 0;
    for function in 0..64u32 {
        for selector in 0..16u32 {
            for round in 0..12 {
                randomise(&mut plain, &mut r);
                let (mut vd, mut vs, mut vt) = (r.bits(5), r.bits(5), r.bits(5));
                match round % 6 {
                    0 => vd = vs,
                    1 => vd = vt,
                    2 => (vs, vt) = (vd, vd),
                    _ => {}
                }
                let word = 0x4A00_0000 | selector << 21 | vt << 16 | vs << 11 | vd << 6 | function;
                plain.bus.sp_imem[..4].copy_from_slice(&word.to_be_bytes());
                plain.bus.sp.processor.start(0);
                copy_processor(&plain, &mut simd);
                plain.bus.sp_step(1);
                simd.bus.sp_step(1);
                let (a, b) = (&plain.bus.sp.processor, &simd.bus.sp.processor);
                assert!(a == b, "function {function:02X} selector {selector} ({word:08X}): {}", parting(a, b, &plain.bus.sp_dmem[..], &simd.bus.sp_dmem[..]));
                cases += 1;
            }
        }
    }
    eprintln!("{cases} cases identical");
}

/// The vector loads and stores of the byte to quad formats at every element, at every address of the last 48 bytes of DMEM and the
/// first 32, where the whole-element fast case must stand down for the bytes that wrap.
#[test]
fn the_whole_element_loads_and_stores_move_what_the_bytewise_ones_move_at_every_edge() {
    let Some((mut plain, mut simd)) = machines() else { return };
    let mut r = Rng::new(0xD0);
    let mut cases = 0;
    for store in [false, true] {
        for format in 0..5u32 {
            for element in 0..16u32 {
                for address in (0xFD0..0x1000u32).chain(0..0x20) {
                    randomise(&mut plain, &mut r);
                    plain.bus.sp.processor.gpr[1] = address;
                    let word = (if store { 0x3A } else { 0x32 }) << 26 | 1 << 21 | 7 << 16 | format << 11 | element << 7;
                    plain.bus.sp_imem[..4].copy_from_slice(&word.to_be_bytes());
                    plain.bus.sp.processor.start(0);
                    copy_processor(&plain, &mut simd);
                    plain.bus.sp_step(1);
                    simd.bus.sp_step(1);
                    let (a, b) = (&plain.bus.sp.processor, &simd.bus.sp.processor);
                    assert!(
                        a == b && plain.bus.sp_dmem == simd.bus.sp_dmem,
                        "{word:08X} at {address:03X}: {}",
                        parting(a, b, &plain.bus.sp_dmem[..], &simd.bus.sp_dmem[..])
                    );
                    cases += 1;
                }
            }
        }
    }
    eprintln!("{cases} cases identical");
}

/// The state carries the accumulator as C#'s eight words: the thirds and the bits above them widen to them and narrow back, at every edge.
#[test]
fn the_accumulator_in_thirds_is_the_words_the_state_carries() {
    let mut r = Rng::new(3);
    for _ in 0..1000 {
        let thirds: [[u16; 8]; 3] = std::array::from_fn(|_| std::array::from_fn(|_| edge(&mut r)));
        let top: [u16; 8] = std::array::from_fn(|_| if r.below(2) == 0 { 0 } else { edge(&mut r) });
        let words = crate::rsp::widen(&thirds, &top);
        assert_eq!(crate::rsp::narrow(&words), (thirds, top));
        for (i, w) in words.iter().enumerate() {
            assert_eq!(*w, (top[i] as u64) << 48 | (thirds[0][i] as u64) << 32 | (thirds[1][i] as u64) << 16 | thirds[2][i] as u64);
        }
    }
}

/// The accumulating multiplies, the rounds and the quarter with each lane's accumulator set so that the sum lands on a clamp's
/// boundary or one either side: uniform and edge-drawn accumulators almost never land there, which is how §3.3's unsigned clamp
/// boundary survived its differential (Mars_Native.md §3.3, §6.10).
#[test]
fn the_accumulating_multiplies_agree_where_each_clamp_turns() {
    let Some((mut plain, mut simd)) = machines() else { return };
    const BOUNDARIES: [u64; 12] = [
        0,
        0x7FFF_0000,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFF,
        0x1_0000_0000,
        0x7FFF_FFFF_FFFF,
        0x8000_0000_0000,
        0xFFFF_0000_0000,
        0xFFFF_7FFF_FFFF,
        0xFFFF_8000_0000,
        0xFFFF_FFFF_FFFF,
    ];
    let mut r = Rng::new(0xC1A);
    let mut cases = 0;
    for function in [0x02u32, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F] {
        for round in 0..2000 {
            randomise(&mut plain, &mut r);
            let (vd, vs, vt) = (r.bits(5), r.bits(5), r.bits(5));
            let (s, t) = (plain.bus.sp.processor.vector[vs as usize], plain.bus.sp.processor.vector[vt as usize]);
            let mut words = [0u64; 8];
            for i in 0..8 {
                let (a, b) = (s[i], t[i]);
                let addend: i64 = match function {
                    0x08 | 0x09 => ((a as i16 as i64) * (b as i16 as i64)) << 1,
                    0x0C => ((a as u32 * b as u32) >> 16) as i64,
                    0x0D => (a as i16 as i64) * (b as i64),
                    0x0E => (a as i64) * (b as i16 as i64),
                    0x0F => ((a as i16 as i64) * (b as i16 as i64)) << 16,
                    // The rounds add vt on one sign only, and the quarter moves by 2^21: the boundary itself is the case.
                    _ => 0,
                };
                let target = BOUNDARIES[r.below(12) as usize].wrapping_add(r.range(-1, 2) as u64);
                words[i] = target.wrapping_sub(addend as u64) & 0xFFFF_FFFF_FFFF;
            }
            plain.bus.sp.processor.accumulator = crate::rsp::narrow(&words).0;
            let selector = if round % 4 == 0 { r.bits(4) } else { 0 };
            let word = 0x4A00_0000 | selector << 21 | vt << 16 | vs << 11 | vd << 6 | function;
            plain.bus.sp_imem[..4].copy_from_slice(&word.to_be_bytes());
            plain.bus.sp.processor.start(0);
            copy_processor(&plain, &mut simd);
            plain.bus.sp_step(1);
            simd.bus.sp_step(1);
            let (a, b) = (&plain.bus.sp.processor, &simd.bus.sp.processor);
            assert!(a == b, "function {function:02X} ({word:08X}) from {words:012X?}: {}", parting(a, b, &plain.bus.sp_dmem[..], &simd.bus.sp_dmem[..]));
            cases += 1;
        }
    }
    eprintln!("{cases} cases identical");
}
