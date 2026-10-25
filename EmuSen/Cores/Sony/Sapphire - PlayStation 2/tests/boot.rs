//! Stage S24's measurements on the board (Sapphire_Native.md §35): what the player's BIOS showed the board lacked,
//! graded by the record PCSX2 gave the same program, since no document reaches it.

mod s21;
mod s22;
mod s24;

use sapphire::board::System;

/// Runs a pair until the EE's done word, as `tests/sif.rs` does.
fn run(p: &s22::Pair) -> System {
    let mut sys = System::new(s22::EE_ORIGIN);
    sys.board.map.load(s22::EE_ORIGIN, &p.image);
    let mut entered = false;
    let limit = 400_000_000;
    let n = sys
        .run(limit, &mut |s| {
            if !entered && s.board.iop.bus.word(0x88) == s21::vector()[2] {
                s.board.iop.enter(s21::K1 | s21::ORIGIN);
                entered = true;
            }
            s.peek(s22::EE_DONE_AT, 4) as u32 == s22::DONE
        })
        .unwrap_or_else(|h| panic!("{}: {h:?}", p.name));
    assert!(n < limit, "{}: no done word", p.name);
    sys
}

/// `s24-sif1-slice` on PCSX2 2.8.2 (measured 2026-10-25): the IOP's SIF1 started with mode 1 and bit 8 reads its tags
/// from the FIFO as a chain does, `d0` untouched, each packet at its tag's address, the channel stopped at the end tag
/// (D-323). DICR2's flag is the one word kept apart: PCSX2 sets it with the channel's mask clear, where ps2tek's "IOP
/// DMA" says a flag is raised only under its mask, which S21 follows.
#[test]
fn sif1_with_mode_1_and_bit_8_reads_the_ees_tags_as_a_chain() {
    let p = s24::sif1_slice();
    let mut sys = run(&p);
    let cc = 0xCCCC_CCCC;
    let mut want = vec![0, 0, s22::DONE];
    want.extend([cc; 16]);
    want.extend((0..8).map(|k| 0x5100_0000 + k));
    want.push(cc);
    want.extend((0..8).map(|k| 0x5200_0000 + k));
    want.push(cc);
    want.extend([0x0000_0300, s22::IOP_AREA + 0x820, 0x20, 0, 0]);
    let got: Vec<u32> = (0..want.len()).map(|i| sys.peek(s22::EE_RECORD + 4 * i as u32, 4) as u32).collect();
    assert_eq!(got, want);
}

/// `s24-sif0-odd` on PCSX2 2.8.2 (measured 2026-10-25): a six-word IOP packet under an EE tag of two quadwords fills both,
/// and the next packet lands at its own tag's address; the IOP's MADR moves by the six alone. The two words after the six
/// are zeros here and two earlier words of the packet on PCSX2, the one difference (D-325).
#[test]
fn a_sif0_packet_short_of_its_quadwords_is_padded_and_the_next_stays_aligned() {
    let p = s24::sif0_odd();
    let mut sys = run(&p);
    let cc = 0xCCCC_CCCC;
    let mut want: Vec<u32> = (0..6).map(|k| 0x6100_0000 + k).collect();
    want.extend([0, 0]);
    want.extend([cc; 4]);
    want.extend((0..4).map(|k| 0x6200_0000 + k));
    want.extend([cc; 4]);
    want.extend([s22::EE_AREA + 0x510, 0, 0x7000_0004, s22::DONE, s22::IOP_AREA + 0x20, s22::IOP_AREA + 0x210, 0x0000_0701]);
    let got: Vec<u32> = (0..want.len()).map(|i| sys.peek(s22::EE_RECORD + 4 * i as u32, 4) as u32).collect();
    assert_eq!(got, want);
}

/// The EE's CTRL bit 19 holds the IOP in reset while set and releases it at its reset vector when cleared, its own
/// units reset and its RAM kept (ps2tek "BIOS PS Compatibility": "Bit 19 is set in SIF_CTRL ... to assert the reset line
/// on the IOP ... then unsets bit 19 ... allowing the IOP to run"; D-320).
#[test]
fn ctrl_bit_19_holds_the_iop_in_reset_until_it_clears() {
    use sapphire::ee::Bus;
    let mut sys = System::new(s22::EE_ORIGIN);
    let b = &mut sys.board;
    b.iop.bus.rom = vec![0; 4 << 20];
    b.iop.enter(0x8000_1000);
    b.iop.bus.ram[0x100] = 0x5A;
    b.iop.bus.dmac.dpcr = 0;
    b.write(0x1000_F240, 4, 1 << 19);
    assert!(b.iop.held && b.iop.cpu.regs.pc == sapphire::iop::RESET && b.iop.cpu.regs.sr == sapphire::iop::SR_BEV);
    assert_eq!((b.iop.bus.dmac.dpcr, b.iop.bus.ram[0x100], b.iop_resets), (0x0777_7777, 0x5A, 1), "the DMAC reset, RAM kept");
    b.write(0x1000_F240, 4, 1 << 19 | 0x100);
    assert_eq!(b.iop_resets, 1, "a write keeping the bit set does not reset again");
    b.write(0x1000_F240, 4, 0x100);
    assert!(!b.iop.held && b.iop.cpu.regs.pc == sapphire::iop::RESET, "released at the reset vector");
}

/// Power-on values the player's BIOS relies on without writing them: D_ENABLER 1201h, as ps2tek says the BIOS expects
/// (D-321), and DMACINTEN with channel interrupts on (D-324); the memory controller answers the RDRAMs' identity reads
/// (D-322).
#[test]
fn the_power_on_values_the_bios_relies_on() {
    use sapphire::ee::Bus;
    let mut sys = System::new(s22::EE_ORIGIN);
    let b = &mut sys.board;
    assert_eq!(b.read(0x1000_F520, 4) as u32, 0x1201);
    assert_eq!(b.iop.bus.dmac.dmacinten, 1);
    b.write(0x1000_F440, 4, 0);
    b.write(0x1000_F430, 4, 0x8021_0040);
    b.write(0x1000_F430, 4, 0x8021_0000);
    let ids: Vec<u32> = (0..3).map(|_| b.read(0x1000_F440, 4) as u32).collect();
    assert_eq!(ids, [0x1F, 0x1F, 0], "two RDRAMs answer");
}

/// A halfword written to a DMA channel's register changes its own half alone: BCR's block count written after its block
/// size (D-327); the DVD player's ROMs' place and the SPU2's window past its registers read zero without a bus error
/// (D-326).
#[test]
fn halfword_dma_writes_merge_and_the_absent_places_read_zero() {
    use blackmoon_r3000::{Bus as IopBus, Size};
    let mut sys = System::new(s22::EE_ORIGIN);
    let bus = &mut sys.board.iop.bus;
    bus.write(0xBF80_10C4, Size::Word, 0x10);
    bus.write(0xBF80_10C6, Size::Half, 0x30);
    assert_eq!(bus.read(0xBF80_10C4, Size::Word), 0x0030_0010);
    for a in [0xBE00_0000u32, 0xBE7F_FFFC, 0xBF90_0B60] {
        assert_eq!(bus.read(a, Size::Word), 0, "{a:08x}");
        assert!(!bus.bus_error(), "{a:08x}");
    }
}
