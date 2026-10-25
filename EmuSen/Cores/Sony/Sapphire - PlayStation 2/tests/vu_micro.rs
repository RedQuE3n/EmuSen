//! Stage S13's pipeline (Sapphire_Native.md §23): the VU User's Manual's pipeline figures (§3.4, §3.5, §7.3) run as
//! microprograms on micro mode, each pair's T stage equal to the figure's, and the rules of visibility the figures and
//! pages give (the flags 4 cycles on, Q at the divider's F stage, P at the EFU's, a branch and the integer it reads, the
//! I bit, the write priority, the E, M, D and T bits), each value the page's.

use emusen_mips::vu as table;
use sapphire::vu::micro::Micro;
use sapphire::vu::pipe::Pipe;
use sapphire::vu::{Link, Path1StandIn, Units, VU0_MEM, VU1_MEM, Vu, fbrst, vpu_stat};

const ONE: u32 = 0x3F80_0000;
const TWO: u32 = 0x4000_0000;
const THREE: u32 = 0x4040_0000;
const FOUR: u32 = 0x4080_0000;
const HALF: u32 = 0x3F00_0000;
const FIRST: u64 = 100;

fn units() -> Units {
    Units { vu0: Vu::new(VU0_MEM), vu1: Vu::new(VU1_MEM), link: Link::default() }
}

fn load(units: &mut Units, unit: usize, src: &str) {
    let prog = table::assemble(src).unwrap_or_else(|e| panic!("{e}"));
    let mem = if unit == 0 { &mut units.vu0.micro } else { &mut units.vu1.micro };
    for (i, pair) in prog.iter().enumerate() {
        mem[i * 8..i * 8 + 8].copy_from_slice(&pair.to_le_bytes());
    }
}

/// Runs `src` on `unit` from its first pair at FIRST to its end; each pair's address and T stage less FIRST.
fn run_on(u: &mut Units, unit: usize, src: &str) -> (Pipe, Vec<(u16, u64)>) {
    load(u, unit, src);
    let mut p = Pipe::new(unit);
    p.trace = Some(Vec::new());
    assert!(p.start(u, 0, FIRST));
    p.finish(u, FIRST);
    let tr = p.trace.take().unwrap().into_iter().map(|(a, t)| (a, t - FIRST)).collect();
    (p, tr)
}

fn times(tr: &[(u16, u64)]) -> Vec<u64> {
    tr.iter().map(|&(_, t)| t).collect()
}

const END: &str = "[e] nop | move vf0, vf0\nnop | move vf0, vf0";

/// Figure 3-3: an FMAC result used four pairs on, with no stall.
#[test]
fn figure_3_3_the_fmac_pipeline_without_a_stall() {
    let mut u = units();
    u.vu0.set_vf(10, [TWO; 4]);
    u.vu0.set_vf(20, [THREE; 4]);
    u.vu0.set_vf(24, [HALF; 4]);
    let src = format!(
        "mul.xyzw vf5, vf10, vf20 | move vf0, vf0\nmula.xyzw acc, vf11, vf21 | move vf0, vf0\nmadda.xyzw acc, vf12, vf22 | move vf0, vf0\n\
         madd.xyzw vf6, vf13, vf23 | move vf0, vf0\nmul.xyzw vf7, vf5, vf24 | move vf0, vf0\nmul.xyzw vf8, vf15, vf25 | move vf0, vf0\n{END}"
    );
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr), [0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(u.vu0.vf(7), [THREE; 4], "instruction 5 read instruction 1's 6.0, times 0.5");
}

/// Figure 3-4: a field written by an FMAC instruction stalls its reader to the writer's S stage; another field of the
/// same register does not.
#[test]
fn figure_3_4_stalls_by_field() {
    let mut u = units();
    u.vu0.set_vf(10, [TWO, THREE, ONE, FOUR]);
    let src = format!(
        "mul.xyzw vf10, vf10, vf10 | move vf0, vf0\naddy.x vf10, vf10, vf10 | move vf0, vf0\naddw.z vf10, vf10, vf10 | move vf0, vf0\n\
         nop | move.z vf20, vf10\nnop | move vf0, vf0\n{END}"
    );
    let (_, tr) = run_on(&mut u, 0, &src);
    // The figure's T stages: 2, 6, 7, 11 (instruction 1 at clock 2).
    assert_eq!(&times(&tr)[..4], [0, 4, 5, 9]);
    assert_eq!(u.vu0.vf(10), [0x4150_0000, 0x4110_0000, 0x4188_0000, 0x4180_0000], "x = 4 + 9, z = 1 + 16");
    assert_eq!(u.vu0.vf(20)[2], 0x4188_0000);
}

/// Figures 3-7 to 3-9: Q is written at the divider's F stage, 7 cycles on; a reader before then reads the old Q, a
/// second divide waits for the first's D6, and WAITQ lets its own pair's upper instruction read the quotient.
#[test]
fn figures_3_7_to_3_9_the_divider_and_q() {
    let mut u = units();
    u.vu0.set_vf(10, [ONE; 4]);
    u.vu0.set_vf(20, [FOUR; 4]);
    u.vu0.set_vf(23, [ONE; 4]);
    u.vu0.q = TWO;
    let nops = "nop | move vf0, vf0\n".repeat(6);
    let src = format!("nop | div q, vf10x, vf20x\n{nops}mulq.x vf13, vf23, q | move vf0, vf0\n{END}");
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(tr[7].1, 7, "Figure 3-7: six pairs between, no stall");
    assert_eq!(u.vu0.vf(13)[0], 0x3E80_0000, "the quotient 0.25 read at F");

    let mut u = units();
    u.vu0.set_vf(10, [ONE; 4]);
    u.vu0.set_vf(20, [FOUR; 4]);
    u.vu0.set_vf(22, [ONE; 4]);
    u.vu0.set_vf(23, [ONE; 4]);
    u.vu0.q = TWO;
    let src = format!("nop | div q, vf10x, vf20x\nmulq.x vf12, vf22, q | move vf0, vf0\nmulq.x vf13, vf23, q | div q, vf20x, vf10x\n{END}");
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..3], [0, 1, 7], "Figure 3-8: the second DIV stalls to the first's F");
    assert_eq!(u.vu0.vf(12)[0], TWO, "no dependency check on Q: the old value");
    assert_eq!(u.vu0.vf(13)[0], 0x3E80_0000, "after the stall, the first quotient");

    let mut u = units();
    u.vu0.set_vf(10, [ONE; 4]);
    u.vu0.set_vf(20, [FOUR; 4]);
    u.vu0.set_vf(23, [ONE; 4]);
    u.vu0.q = TWO;
    let src = format!("nop | div q, vf10x, vf20x\nmulq.x vf13, vf23, q | waitq\n{END}");
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..2], [0, 7], "Figure 3-9: WAITQ stalls to F");
    assert_eq!(u.vu0.vf(13)[0], 0x3E80_0000, "and the upper instruction of its pair reads the quotient");
}

/// Q read 0 to 7 pairs after DIV, and 0 to 13 after RSQRT: the old value until the latency has passed (§7.4).
#[test]
fn q_lands_at_the_divider_s_latency() {
    for (op, latency) in [("div q, vf10x, vf20x", 7u64), ("sqrt q, vf20x", 7), ("rsqrt q, vf10x, vf20x", 13)] {
        for gap in 1..=latency + 1 {
            let mut u = units();
            u.vu0.set_vf(10, [ONE; 4]);
            u.vu0.set_vf(20, [FOUR; 4]);
            u.vu0.q = THREE;
            let nops = "nop | move vf0, vf0\n".repeat(gap as usize - 1);
            let src = format!("nop | {op}\n{nops}addq.x vf1, vf0, q | move vf0, vf0\n{END}");
            run_on(&mut u, 0, &src);
            let new = match &op[..3] {
                "div" => 0x3E80_0000,
                "sqr" => TWO,
                _ => HALF,
            };
            let want = if gap >= latency { new } else { THREE };
            assert_eq!(u.vu0.vf(1)[0], want, "{op}, read {gap} pairs on");
        }
    }
}

/// Figures 3-11 and 3-12: a second EFU operation waits to the first's last N stage; P lands the cycle after, which
/// WAITP waits for, and MFP before it reads the old P.
#[test]
fn figures_3_11_and_3_12_the_efu_and_p() {
    let mut u = units();
    u.vu1.set_vf(10, [ONE, TWO, ONE, 0]);
    u.vu1.p = THREE;
    let src = format!("nop | esadd p, vf10\nnop | esadd p, vf10\nnop | mfp.x vf1, p\n{END}");
    let (_, tr) = run_on(&mut u, 1, &src);
    assert_eq!(times(&tr)[..3], [0, 10, 11], "ESADD's throughput 10, latency 11 (§7.4)");
    assert_eq!(u.vu1.vf(1)[0], 0x40C0_0000, "MFP at the first's P stage reads 6.0");

    let mut u = units();
    u.vu1.set_vf(10, [ONE, TWO, ONE, 0]);
    u.vu1.p = THREE;
    let src = format!("nop | esadd p, vf10\nnop | mfp.x vf1, p\nnop | waitp\nnop | mfp.x vf2, p\n{END}");
    let (_, tr) = run_on(&mut u, 1, &src);
    assert_eq!(times(&tr)[..4], [0, 1, 10, 11], "WAITP issues at the last N stage, its follower at P");
    assert_eq!(u.vu1.vf(1)[0], THREE, "the old P before the wait");
    assert_eq!(u.vu1.vf(2)[0], 0x40C0_0000);
}

/// Figure 3-14: the integer unit's results bypass to the next pair; no stall.
#[test]
fn figure_3_14_integer_results_reach_the_next_pair() {
    let mut u = units();
    u.vu0.set_vi(2, 5);
    u.vu0.set_vi(3, 7);
    u.vu0.set_vi(5, 20);
    let src = format!(
        "mula.xyzw acc, vf20, vf10 | iadd vi4, vi3, vi2\nmadda.xyzw acc, vf21, vf11 | iaddi vi5, vi5, -1\nmadda.xyzw acc, vf22, vf12 | iadd vi6, vi5, vi4\n\
         madd.xyzw vf14, vf23, vf13 | iadd vi7, vi6, vi4\n{END}"
    );
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..4], [0, 1, 2, 3]);
    assert_eq!((u.vu0.vi(6), u.vu0.vi(7)), (19 + 12, 31 + 12));
}

/// Figure 3-15: a loop whose conditional branch reads an integer written two pairs before, with its delay slot.
#[test]
fn figure_3_15_a_loop_with_a_branch_and_its_delay_slot() {
    let mut u = units();
    u.vu0.set_vi(3, 3);
    let src = format!(
        "top: nop | iaddi vi3, vi3, -1\nnop | iaddi vi4, vi4, 1\nnop | ibne vi3, vi0, top\nnop | iaddi vi5, vi5, 1\n{END}"
    );
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(tr.iter().map(|&(a, _)| a).collect::<Vec<_>>(), [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5]);
    assert_eq!(times(&tr), (0..14).collect::<Vec<_>>(), "no stall: the branch is taken without a bubble");
    assert_eq!((u.vu0.vi(4), u.vu0.vi(5)), (3, 3), "the delay slot ran on every pass");
}

/// §3.4.8: a conditional branch right after the integer write it reads sees the value before it; with one pair between
/// it sees the new one; after a flag check it sees the check's result at once (Sapphire_Disputes.md D-80).
#[test]
fn a_branch_reads_an_integer_written_just_before_as_it_was() {
    let taken = |setter: &str, gap: usize| {
        let mut u = units();
        u.vu0.set_vi(1, 1);
        u.vu0.status = 0x8;
        let nops = "nop | move vf0, vf0\n".repeat(gap);
        let src = format!("nop | {setter}\n{nops}nop | ibne vi1, vi0, away\nnop | move vf0, vf0\n{END}\naway: nop | iaddiu vi9, vi0, 1\n{END}");
        run_on(&mut u, 0, &src);
        u.vu0.vi(9) == 1
    };
    assert!(taken("iaddiu vi1, vi0, 0", 0), "the branch read the old 1 and was taken");
    assert!(!taken("iaddiu vi1, vi0, 0", 1), "one pair between: it read 0");
    assert!(!taken("fsand vi1, 0x0", 0), "after FSAND, the result at once (0)");
    assert!(taken("fsand vi1, 0x8", 0), "after FSAND, the result at once (8)");
}

/// Figures 3-16 and 3-17: the status flag an instruction writes at its S stage is what FSAND reads four cycles on;
/// with stalls, the flag of the instruction whose S stage has passed.
#[test]
fn figures_3_16_and_3_17_the_status_flag_s_timing() {
    let neg = 0xBF80_0000;
    let mut u = units();
    u.vu0.set_vf(10, [neg; 4]);
    u.vu0.set_vf(20, [ONE; 4]);
    let src = format!(
        "mul.xyzw vf5, vf10, vf20 | move vf0, vf0\nadd.xyzw vf6, vf0, vf0 | move vf0, vf0\nadd.xyzw vf7, vf0, vf0 | move vf0, vf0\n\
         add.xyzw vf8, vf0, vf0 | move vf0, vf0\nadd.xyzw vf9, vf0, vf0 | fsand vi1, 0xfff\nnop | ibne vi1, vi0, out\nnop | move vf0, vf0\n{END}\n\
         out: nop | iaddiu vi9, vi0, 1\n{END}"
    );
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..6], [0, 1, 2, 3, 4, 5]);
    assert_eq!(u.vu0.vi(1), 0x2 | 0x80, "FSAND at the MUL's S stage reads its S and SS (and no later Z)");
    assert_eq!(u.vu0.vi(9), 1, "and the branch right after it read that");

    let mut u = units();
    u.vu0.set_vf(10, [neg; 4]);
    u.vu0.set_vf(20, [ONE; 4]);
    u.vu0.set_vf(11, [ONE; 4]);
    let src = format!(
        "mul.xyzw vf5, vf10, vf20 | move vf0, vf0\nadd.xyzw vf6, vf11, vf5 | move vf0, vf0\nadd.xyzw vf7, vf12, vf6 | move vf0, vf0\n\
         add.xyzw vf8, vf13, vf23 | move vf0, vf0\nadd.xyzw vf9, vf14, vf24 | fsand vi1, 0xf\n{END}"
    );
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..5], [0, 4, 8, 9, 10], "Figure 3-17's stalls");
    assert_eq!(u.vu0.vi(1), 0x1, "the flag is instruction 2's (a zero sum), not instruction 3's");
}

/// The MAC and clipping flags have the status flag's timing: FMAND and FCGET three cycles after the writer read the
/// old flags, four after it the new (§3.4.4).
#[test]
fn the_mac_and_clipping_flags_land_at_s() {
    for gap in 1..=5usize {
        let mut u = units();
        u.vu0.set_vf(1, [0xBF80_0000; 4]);
        u.vu0.set_vf(2, [TWO, 0, 0, ONE]);
        u.vu0.set_vi(10, 0xFFFF);
        let nops = "nop | move vf0, vf0\n".repeat(gap - 1);
        let src = format!("add.xyzw vf3, vf1, vf0 | move vf0, vf0\n{nops}clipw.xyz vf2, vf2 | fmand vi4, vi10\n{}nop | fcget vi5\n{END}", "nop | move vf0, vf0\n".repeat(gap - 1));
        run_on(&mut u, 0, &src);
        assert_eq!(u.vu0.vi(4), if gap >= 4 { 0x00E1 } else { 0 }, "FMAND {gap} pairs after ADD");
        assert_eq!(u.vu0.vi(5), if gap >= 4 { 0x0001 } else { 0 }, "FCGET {gap} pairs after CLIP");
    }
}

/// FSSET and FCSET write at the S stage, and the flag they write ignores what the upper instruction of their pair does
/// to it (§3.3.4).
#[test]
fn fsset_and_fcset_write_at_s_and_win_over_their_pair() {
    let mut u = units();
    u.vu0.status = 0xFC0;
    u.vu0.set_vf(1, [0xBF80_0000; 4]);
    let src = format!(
        "add.xyzw vf3, vf1, vf0 | fsset 0x0\nnop | fsand vi1, 0xfff\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | fsand vi2, 0xfff\n\
         clipw.xyz vf1, vf1 | fcset 0x123456\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | fcget vi3\n{END}"
    );
    run_on(&mut u, 0, &src);
    assert_eq!(u.vu0.vi(1), 0xFC0, "before the S stage, the old sticky flags");
    assert_eq!(u.vu0.vi(2), 0x000, "FSSET's 0, the ADD's S and SS ignored");
    assert_eq!(u.vu0.vi(3), 0x456, "FCSET's value, CLIP's judgement ignored");
    assert_eq!(u.vu0.mac, 0x00E1, "the MAC flags are still the ADD's");
}

/// Figure 3-18: a second XGKICK while PATH1 is busy delays the pair after it, not itself, to the end of the transfer.
#[test]
fn figure_3_18_xgkick_waits_for_path1() {
    let mut u = units();
    u.link.path1 = Box::new(Path1StandIn { kicks: Vec::new(), busy: 20 });
    u.vu1.set_vi(1, 0x10);
    u.vu1.set_vi(2, 0x20);
    let src = format!("mul.xyzw vf3, vf10, vf20 | xgkick vi1\nmul.xyzw vf4, vf11, vf21 | xgkick vi2\nmul.xyzw vf5, vf12, vf22 | move vf0, vf0\n{END}");
    let (_, tr) = run_on(&mut u, 1, &src);
    assert_eq!(times(&tr)[..3], [0, 1, 20], "instruction 3 waits for the first transfer's end");
}

/// Figure 3-19: the E bit's delay slot runs, the unit stops after it, and TPC holds the next pair's address (in
/// doublewords, Sapphire_Disputes.md D-82); the instructions §3.5.3 forbids in the slot are not checked.
#[test]
fn figure_3_19_the_e_bit_and_its_delay_slot() {
    let mut u = units();
    let src = "nop | iaddiu vi1, vi0, 1\nnop | iaddiu vi2, vi0, 2\n[e] nop | iaddiu vi3, vi0, 3\nnop | iaddiu vi4, vi0, 4\nnop | iaddiu vi5, vi0, 5";
    let (p, tr) = run_on(&mut u, 0, src);
    assert_eq!(tr.len(), 4);
    assert_eq!((u.vu0.vi(4), u.vu0.vi(5)), (4, 0), "the delay slot ran, the pair after it did not");
    assert_eq!(u.vu0.tpc, 4);
    assert!(!p.busy());
    assert_eq!(p.stat(), 0, "VBS clear: stopped by the E bit");
}

/// §3.4.3: an upper and a lower instruction writing one register in the same pair: the upper's result is kept and
/// the lower's discarded, whatever their fields.
#[test]
fn the_upper_instruction_wins_a_register_both_write() {
    let mut u = units();
    u.vu0.set_vf(23, [ONE; 4]);
    u.vu0.set_vf(9, [FOUR; 4]);
    u.vu0.set_vf(1, [TWO; 4]);
    let src = format!("add.xy vf1, vf1, vf23 | move.w vf1, vf9\nnop | move vf2, vf1\n{END}");
    run_on(&mut u, 0, &src);
    assert_eq!(u.vu0.vf(1), [THREE, THREE, TWO, TWO], "MOVE.w discarded");
    // And the two halves read the registers as they were at T: the lower reads VF01 before the upper writes it.
    let mut u = units();
    u.vu0.set_vf(1, [TWO; 4]);
    u.vu0.set_vf(23, [ONE; 4]);
    let src = format!("add.xyzw vf1, vf1, vf23 | move.xyzw vf5, vf1\n{END}");
    run_on(&mut u, 0, &src);
    assert_eq!(u.vu0.vf(5), [TWO; 4]);
}

/// The I bit: the lower word goes to I at the T stage, which the next pair reads; the upper instruction of its own
/// pair reads I as it was (Sapphire_Disputes.md D-81).
#[test]
fn the_i_bit_loads_i_for_the_next_pair() {
    let mut u = units();
    u.vu0.i = ONE;
    let src = format!("[i] addi.x vf1, vf0, i | 0x40800000\naddi.x vf2, vf0, i | move vf0, vf0\n{END}");
    run_on(&mut u, 0, &src);
    assert_eq!((u.vu0.vf(1)[0], u.vu0.vf(2)[0], u.vu0.i), (ONE, FOUR, FOUR));
}

/// ILW's and ILWR's loads into an integer register follow the FMAC pipeline: a reader waits 4 cycles (§3.4.1, §7.4).
#[test]
fn an_integer_load_holds_its_reader() {
    let mut u = units();
    u.vu0.set_mem_q(3, 0x0000_0000_0000_0000_0000_0007_0000_0000);
    let src = format!("nop | ilw.y vi1, 0x3(vi0)\nnop | iadd vi2, vi1, vi1\n{END}");
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..2], [0, 4]);
    assert_eq!(u.vu0.vi(2), 14);
}

/// The D and T bits stop the unit after their pair, with no delay slot, when FBRST enables them; on a branch the delay
/// slot runs and TPC holds the target (§7.3.1). Disabled, they do nothing.
#[test]
fn the_d_and_t_bits_halt_when_enabled() {
    let src = "nop | iaddiu vi1, vi0, 1\n[d] nop | iaddiu vi2, vi0, 2\nnop | iaddiu vi3, vi0, 3\n[e] nop | move vf0, vf0\nnop | move vf0, vf0";
    let mut u = units();
    let (p, _) = run_on(&mut u, 0, src);
    assert_eq!((u.vu0.vi(3), p.stat(), u.link.irq), (3, 0, 0), "D disabled: ignored");
    let mut u = units();
    u.link.fbrst = fbrst::DE0;
    let (p, tr) = run_on(&mut u, 0, src);
    assert_eq!((tr.len(), u.vu0.vi(3), u.vu0.tpc), (2, 0, 2));
    assert_eq!((p.stat(), u.link.irq), (vpu_stat::VDS, 1), "Stop by D, VU0's interrupt");
    let mut u = units();
    u.link.fbrst = fbrst::TE0 << fbrst::VU1;
    let src_t = "nop | iaddiu vi1, vi0, 1\n[t] nop | b there\nnop | iaddiu vi2, vi0, 2\nnop | iaddiu vi3, vi0, 3\nthere: nop | iaddiu vi4, vi0, 4\n[e] nop | move vf0, vf0\nnop | move vf0, vf0";
    let (p, _) = run_on(&mut u, 1, src_t);
    assert_eq!((u.vu1.vi(2), u.vu1.vi(4), u.vu1.tpc), (2, 0, 4), "on a branch: the delay slot, then TPC at the target");
    assert_eq!((p.stat(), u.link.irq), (vpu_stat::VTS, 2));
}

/// The M bit: QMTC2.I and CTC2.I may issue the cycle after the first pair with M set (§5.4.6's figure).
#[test]
fn the_m_bit_releases_the_interlocked_transfers() {
    let mut u = units();
    load(&mut u, 0, "nop | move vf0, vf0\n[m] nop | move vf0, vf0\n[m] nop | move vf0, vf0\n[em] nop | move vf0, vf0\n[m] nop | move vf0, vf0");
    let mut p = Pipe::new(0);
    p.start(&mut u, 0, FIRST);
    assert_eq!(p.release(&mut u, FIRST), Some(FIRST + 2));
    assert_eq!(p.finish(&mut u, FIRST), Some(FIRST + 4));
}

/// JALR with one register for both: the target is read before the link is written (ps2autotests' `vu/lower/branch`,
/// a console's text: "JALR /w same regs -> targetReg: 0004 returnReg: 0004 resultReg: 0001").
#[test]
fn jalr_reads_its_target_before_it_links() {
    let mut u = units();
    let src = format!("nop | iaddiu vi4, vi0, 6\nnop | move vf0, vf0\nnop | jalr vi4, vi4\nnop | move vf0, vf0\n{END}\nnop | move vf0, vf0\nnop | iaddiu vi1, vi0, 1\n{END}");
    run_on(&mut u, 0, &src);
    assert_eq!((u.vu0.vi(4), u.vu0.vi(1)), (4, 1));
}

/// The divider and the EFU as resources: a second divide waits for the first's F stage, a second EFU operation for the
/// first's last N stage (§3.4.5, §3.4.6), whatever pairs lie between.
#[test]
fn the_divider_and_the_efu_are_resources() {
    let mut u = units();
    let src = format!("nop | rsqrt q, vf1x, vf2x\nnop | move vf0, vf0\nnop | sqrt q, vf2x\n{END}");
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..3], [0, 1, 13]);
    let mut u = units();
    let src = format!("nop | eexp p, vf1x\nnop | ercpr p, vf1x\nnop | waitp\nnop | move vf0, vf0\n{END}");
    let (_, tr) = run_on(&mut u, 1, &src);
    assert_eq!(times(&tr)[..4], [0, 43, 54, 55], "EEXP 43/44, then ERCPR 11/12 (§7.4)");
}

/// VU0 has no EFU (§1.1.2): an EFU word there leaves P alone and holds nothing.
#[test]
fn vu0_has_no_efu() {
    let mut u = units();
    u.vu0.set_vf(1, [FOUR; 4]);
    let src = format!("nop | esqrt p, vf1x\nnop | waitp\n{END}");
    let (_, tr) = run_on(&mut u, 0, &src);
    assert_eq!(times(&tr)[..2], [0, 1]);
    assert_eq!(u.vu0.p, 0);
}

/// A force break stops a running microprogram with VFS; a reset clears the state (§1.3, §5.1.3).
#[test]
fn force_break_and_reset() {
    let mut u = units();
    load(&mut u, 1, "top: nop | b top\nnop | move vf0, vf0");
    let mut p = Pipe::new(1);
    p.start(&mut u, 0, FIRST);
    p.run_to(&mut u, FIRST + 50);
    assert!(p.busy());
    p.force_break(&mut u, FIRST + 60);
    assert_eq!(p.stat(), vpu_stat::VFS);
    p.reset();
    assert_eq!(p.stat(), 0);
}

/// ps2autotests' `vu/lower/fdivdelay` text, a console's output: "DIV single delayed" prints VF01 to VF14 with 1.00
/// six times and then 0.25, "SQRT single delayed" 1.00 six times and then 2.00, "RSQRT single delayed" 1.00 twelve
/// times and then 0.50. Read as a divide followed by one pair a register, each adding Q (the test's source is not
/// read; the lines' shape is the inference), the quotient reaches the seventh pair and the thirteenth (D-51's micro
/// form).
#[test]
fn the_fdivdelay_console_text_s_single_delayed_lines_hold() {
    for (op, a, b, new, old_count) in [("div q, vf1w, vf2w", ONE, FOUR, 0x3E80_0000, 6), ("sqrt q, vf2w", 0, FOUR, TWO, 6), ("rsqrt q, vf1w, vf2w", ONE, FOUR, HALF, 12)] {
        let mut u = units();
        u.vu0.set_vf(1, [0, 0, 0, a]);
        u.vu0.set_vf(2, [0, 0, 0, b]);
        u.vu0.q = ONE;
        let reads: String = (1..=14).map(|n| format!("addq.w vf{}, vf0, q | move vf0, vf0\n", n + 2)).collect();
        let src = format!("nop | {op}\n{reads}{END}");
        run_on(&mut u, 0, &src);
        let ws: Vec<u32> = (1..=14).map(|n| u.vu0.vf(n + 2)[3]).collect();
        let want: Vec<u32> = (1..=14).map(|n| if n <= old_count { vf00_w_plus(ONE) } else { vf00_w_plus(new) }).collect();
        assert_eq!(ws, want, "{op}");
    }
}

/// VF00's w (1.0) plus `q`, as ADDq.w forms it.
fn vf00_w_plus(q: u32) -> u32 {
    sapphire::float::add(ONE, q).bits
}

/// ps2autotests' `vu/upper/clip` text, a console's output: its "Multi test" prints 0000 four times, then 0015, 056a,
/// 0a99 and 0666 three times. Read as four CLIPs in four pairs (judgements 15h, 2Ah, 19h and 26h, the "Basic test"
/// lines' FULL POS, FULL NEG, HALF POS and HALF NEG against 1), each beside an FCGET, then six FCGETs, the flag is seen
/// four cycles after its CLIP.
#[test]
fn the_clip_console_text_s_multi_test_holds() {
    let mut u = units();
    let h = 0x4000_0000;
    let (n, m) = (h | 0x8000_0000, ONE);
    for (r, v) in [(1, [h, h, h, m]), (2, [n, n, n, m]), (3, [h, n, h, m]), (4, [n, h, n, m])] {
        u.vu0.set_vf(r, v);
    }
    let src = format!(
        "clipw.xyz vf1, vf1 | fcget vi1\nclipw.xyz vf2, vf2 | fcget vi2\nclipw.xyz vf3, vf3 | fcget vi3\nclipw.xyz vf4, vf4 | fcget vi4\n\
         nop | fcget vi5\nnop | fcget vi6\nnop | fcget vi7\nnop | fcget vi8\nnop | fcget vi9\nnop | fcget vi10\n{END}"
    );
    run_on(&mut u, 0, &src);
    let got: Vec<u16> = (1..=10).map(|r| u.vu0.vi(r)).collect();
    assert_eq!(got, [0, 0, 0, 0, 0x015, 0x56A, 0xA99, 0x666, 0x666, 0x666]);
}
