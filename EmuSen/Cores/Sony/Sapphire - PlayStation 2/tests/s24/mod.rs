//! The project's S24 programs (Sapphire_Native.md §35): measurement programs for what the player's BIOS showed the
//! board lacked, in S22's paired form (`tests/s22`). `s24-sif1-slice` asks how the IOP's SIF1 channel reads the EE's
//! packets when its CHCR has bit 8 and transfer mode 1, the value the BIOS was seen to write (D-323); its record is a
//! measurement, every word ungraded.

#![allow(dead_code)]

use crate::s21;
use crate::s22::*;

/// `s24-sif1-slice`: the IOP's channel 10 started with STR, bit 8 and mode 1, BCR 20h and MADR at `d0`, then two EE
/// normal-mode packets, each an IOP tag for its own address (`d1`, then `d2`, the second with the end bit) and eight words; the IOP records
/// `d0`'s first sixteen words, `d1`'s and `d2`'s first nine, its channel's CHCR, MADR, BCR and TADR and DICR2.
pub fn sif1_slice() -> Pair {
    use s21::reg::{DICR2, bcr, chcr as ichcr, madr as imadr, tadr as itadr};
    pair("s24-sif1-slice", |n| {
        let mut e = ee_side("s24-sif1-slice", n);
        let mut i = iop_side("s24-sif1-slice");
        iop_sif_setup(&mut i);
        let (d0, d1, d2) = (IOP_AREA, IOP_AREA + 0x400, IOP_AREA + 0x800);
        let pad = [0xEEEE_0000, 0xEEEE_0001];
        let (a, b) = (pattern(0x5100_0000, 8), pattern(0x5200_0000, 8));
        let (e0, e1) = (EE_AREA, EE_AREA + 0x100);
        e.fill(e0, &[&[iop_tag(d1, false, false), 8][..], &pad, &a].concat());
        e.fill(e1, &[&[iop_tag(d2, false, true), 8][..], &pad, &b].concat());
        for d in [d0, d1, d2] {
            iop_fill(&mut i, d, &marks(16));
        }
        i.poke(imadr(10), d0);
        i.poke(bcr(10), 0x20);
        i.poke(ichcr(10), 0x0100_0300);
        meet(&mut e, &mut i, 1);
        for at in [e0, e1] {
            e.poke(ee::madr(ee::SIF1), at);
            e.poke(ee::qwc(ee::SIF1), 3);
            e.poke(ee::chcr(ee::SIF1), ee::STR | ee::DIR);
            e.wait(ee::chcr(ee::SIF1), ee::STR, false, 200_000);
            e.rec(ee::qwc(ee::SIF1), None);
        }
        meet(&mut e, &mut i, 2);
        i.delay(2_000);
        for (d, words) in [(d0, 16), (d1, 9), (d2, 9)] {
            for k in 0..words {
                i.rec(s21::K1 | (d + 4 * k), None);
            }
        }
        for r in [ichcr(10), imadr(10), bcr(10), itadr(10), DICR2] {
            i.rec(r, None);
        }
        (e, i)
    })
}

/// `s24-sif0-odd`: an IOP chain of two four-word tags with bit 8, the first IOP tag of six words carrying an EE cnt tag
/// of two quadwords, the second of four words carrying an EE end tag of one; the IOP's first data area holds eight
/// distinct words, so the record shows whether the six are padded to the quadwords the EE expects, and from where. The
/// EE records its two areas, its channel's MADR, QWC and CHCR; the IOP its channel's MADR, TADR and CHCR (D-325).
pub fn sif0_odd() -> Pair {
    use ee::*;
    use s21::reg::{chcr as ichcr, madr as imadr, tadr as itadr};
    pair("s24-sif0-odd", |n| {
        let mut e = ee_side("s24-sif0-odd", n);
        let mut i = iop_side("s24-sif0-odd");
        iop_sif_setup(&mut i);
        let s = IOP_AREA;
        let (a, b) = (EE_AREA + 0x400, EE_AREA + 0x500);
        let (p1, p2) = (pattern(0x6100_0000, 8), pattern(0x6200_0000, 4));
        let cnt = ee_tag(id::CNT, 2, a, false);
        let end = ee_tag(id::END, 1, b, false);
        iop_fill(&mut i, s, &[iop_tag(s + 0x100, false, false), 6, cnt[0], cnt[1], iop_tag(s + 0x200, false, true), 4, end[0], end[1]]);
        iop_fill(&mut i, s + 0x100, &p1);
        iop_fill(&mut i, s + 0x200, &p2);
        e.fill(a, &marks(12));
        e.fill(b, &marks(8));
        meet(&mut e, &mut i, 1);
        e.poke(qwc(SIF0), 0);
        e.poke(chcr(SIF0), STR | CHAIN);
        meet(&mut e, &mut i, 2);
        i.poke(itadr(9), s);
        i.poke(ichcr(9), 0x0100_0701);
        i.delay(3_000);
        i.rec(itadr(9), None);
        i.rec(imadr(9), None);
        i.rec(ichcr(9), None);
        meet(&mut e, &mut i, 3);
        e.delay(20_000);
        for k in 0..12 {
            e.rec(a + 4 * k, None);
        }
        for k in 0..8 {
            e.rec(b + 4 * k, None);
        }
        e.rec(madr(SIF0), None);
        e.rec(qwc(SIF0), None);
        e.rec(chcr(SIF0), None);
        (e, i)
    })
}
