//! Stage S32's counts (Sapphire_Native.md §43): the units' own counts the bench reads, each equal to the work done, and
//! none of them in the state.

use sapphire::board::{Board, System};
use sapphire::clock;
use sapphire::vu::micro::Micro;
use sapphire::vu::pipe::Pipe;
use sapphire::vu::{Link, Units, VU0_MEM, VU1_MEM, Vu};

const ORIGIN: u32 = 0x0010_0000;
/// ADDIU $t0, $t0, 1: each instruction run adds one to $t0, so $t0 is the count run.
const ADD_ONE: u32 = 0x2508_0001;

#[test]
fn the_ee_counts_its_steps_by_the_mode_each_ran_in() {
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &[ADD_ONE; 4096]);
    for _ in 0..300 {
        sys.step().unwrap();
    }
    sys.ee.cop0[12] = sys.ee.cop0[12] & !0x1E | 0x10;
    for _ in 0..200 {
        sys.step().unwrap();
    }
    sys.ee.cop0[12] |= 0x02;
    for _ in 0..100 {
        sys.step().unwrap();
    }
    assert_eq!(sys.ee.modes, [200, 0, 300, 100, 0], "user, supervisor, kernel, level 1, level 2");
    assert_eq!(sys.ee.gpr[8] as u64, 600);
    assert_eq!(sys.ee.timing.instructions, 600);
}

#[test]
fn the_iop_counts_the_instructions_it_runs() {
    let mut b = Board::new(clock::NTSC);
    for w in 0..16_384 {
        b.iop.bus.ram[0x1000 + 4 * w..0x1004 + 4 * w].copy_from_slice(&ADD_ONE.to_le_bytes());
    }
    b.iop.enter(0x1000);
    b.iop.run_to(80_000);
    let run = b.iop.cpu.regs.r[8] as u64;
    assert!(run > 1_000, "the IOP ran {run}");
    assert_eq!(b.iop.instructions, run);
}

#[test]
fn each_vu_counts_its_pairs_and_keeps_the_count_across_a_reset() {
    let mut u = Units { vu0: Vu::new(VU0_MEM), vu1: Vu::new(VU1_MEM), link: Link::default(), halts: false };
    let prog = emusen_mips::vu::assemble("nop | iaddiu vi1, vi0, 1\nnop | iaddiu vi2, vi0, 2\n[e] nop | iaddiu vi3, vi0, 3\nnop | iaddiu vi4, vi0, 4").unwrap();
    for (i, pair) in prog.iter().enumerate() {
        u.vu1.micro[i * 8..i * 8 + 8].copy_from_slice(&pair.to_le_bytes());
    }
    let mut p = Pipe::new(1);
    assert!(p.start(&mut u, 0, 100));
    p.finish(&mut u, 100);
    assert_eq!(p.pairs_run(), 4, "three pairs and the E bit's delay slot");
    p.reset();
    assert_eq!(p.pairs_run(), 4);
    assert!(p.start(&mut u, 0, 1_000));
    p.finish(&mut u, 1_000);
    assert_eq!(p.pairs_run(), 8);
}
