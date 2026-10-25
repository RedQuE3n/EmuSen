//! Drives the IPU through a stream as an FMV player's program would, by its commands alone (EE User's Manual §8.1.3):
//! the stream behind a preamble of the two default matrices, BCLR, SETIQ for each matrix, IPU_CTRL set for each
//! picture, VDEC and FDEC on each macroblock's symbols, then IDEC on an I-picture's slices or BDEC on each macroblock,
//! FB carrying the pointer to each place `standard` recorded and FDEC with FB 32 crossing longer gaps; a macroblock with
//! neither macroblock_intra nor macroblock_pattern has no blocks and no BDEC, its prediction being the EE's. Every
//! result is checked against `standard`'s decoding and `manual`'s post-processing, and every command is logged with what it gave,
//! which `mod.rs` turns into the referees' programs.

#![allow(dead_code)]

use sapphire::ipu::{Ipu, code, ctrl};

use super::manual;
use super::standard::{self as st, Decoded, Event, Mb};

/// The preamble's length in bits: the default intra matrix and the non-intra one, 64 bytes each in zigzag order.
pub const PREAMBLE: usize = 1024;

/// How IDEC's output is asked for, and whether I-pictures go through IDEC at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub idec: bool,
    pub rgb16: bool,
    pub dither: bool,
    pub sgn: bool,
    pub th: (u32, u32),
}

impl Options {
    pub const BDEC: Options = Options { idec: false, rgb16: false, dither: false, sgn: false, th: (0, 0) };
    pub const IDEC32: Options = Options { idec: true, rgb16: false, dither: false, sgn: false, th: (0, 0) };
}

/// One command as issued, and what the IPU gave: IPU_CMD after it, IPU_CTRL, IPU_BP and its output qwords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issued {
    pub word: u32,
    pub cmd: u64,
    pub ctrl: u32,
    pub bp: u32,
    pub out: Vec<u128>,
}

/// The stream as the IPU receives it: the preamble, the stream, and zeros so that 32 bits follow its last symbol.
pub fn image(stream: &[u8]) -> Vec<u128> {
    let mut bytes = Vec::new();
    for v in 0..64 {
        let n = (0..64).find(|&i| st::ZIGZAG[i / 8][i % 8] == v).unwrap();
        bytes.push(st::DEFAULT_INTRA[n]);
    }
    bytes.extend([16u8; 64]);
    bytes.extend_from_slice(stream);
    bytes.extend([0u8; 32]);
    while bytes.len() % 16 != 0 {
        bytes.push(0);
    }
    bytes.chunks(16).map(|c| u128::from_le_bytes(c.try_into().unwrap())).collect()
}

pub struct Driver {
    pub ipu: Ipu,
    data: Vec<u128>,
    pushed: usize,
    /// Where the IPU's pointer should be, in bits from the image's start.
    cursor: usize,
    pub log: Vec<Issued>,
    matrices: [[u8; 64]; 2],
    frame_pred_frame_dct: bool,
    kind: u32,
}

impl Driver {
    pub fn new(stream: &[u8]) -> Driver {
        Driver {
            ipu: Ipu::new(),
            data: image(stream),
            pushed: 0,
            cursor: 0,
            log: Vec::new(),
            matrices: [st::DEFAULT_INTRA, [16; 64]],
            frame_pred_frame_dct: true,
            kind: 1,
        }
    }

    /// The IPU's pointer in bits from the image's start, from the qwords given and IPU_BP.
    pub fn position(&self) -> usize {
        let bp = self.ipu.read_bp();
        let inside = (bp >> 8 & 0xF) as usize + (bp >> 16 & 3) as usize;
        (self.pushed - inside) * 128 + (bp & 0x7F) as usize
    }

    /// A command run to its end as channel 4 and channel 3 would run it: the input fed while there is room, the output
    /// taken as it appears.
    pub fn run(&mut self, word: u32) -> Result<Issued, String> {
        self.ipu.command(word);
        let mut out = Vec::new();
        for _ in 0..1_000_000 {
            while self.ipu.request_in(1) > 0 && self.pushed < self.data.len() {
                self.ipu.put(self.data[self.pushed]);
                self.pushed += 1;
            }
            while self.ipu.request_out(1) > 0 {
                out.push(self.ipu.get());
            }
            if !self.ipu.busy() {
                let i = Issued { word, cmd: self.ipu.read_cmd(), ctrl: self.ipu.read_ctrl(), bp: self.ipu.read_bp(), out };
                self.log.push(i.clone());
                return Ok(i);
            }
            if self.pushed == self.data.len() && self.ipu.request_out(1) == 0 {
                return Err(format!("command {word:08x} waits at bit {} with the whole image given", self.position()));
            }
        }
        Err(format!("command {word:08x} never ended"))
    }

    /// FB for a command at bit `to` of the stream, after FDECs with FB 32 cross any longer gap.
    fn reach(&mut self, to: usize) -> Result<u32, String> {
        let to = to + PREAMBLE;
        if to < self.cursor {
            return Err(format!("a place behind the pointer: {to} < {}", self.cursor));
        }
        while to - self.cursor > 32 {
            self.run(code::FDEC << 28 | 32)?;
            self.cursor += 32;
            self.check_cursor("FDEC 32")?;
        }
        let fb = (to - self.cursor) as u32;
        self.cursor = to;
        Ok(fb)
    }

    fn check_cursor(&self, what: &str) -> Result<(), String> {
        if self.position() != self.cursor {
            return Err(format!("after {what}: the pointer at {}, expected {}", self.position(), self.cursor));
        }
        Ok(())
    }

    fn fdec(&mut self, at: usize, bits: usize, want: u32, what: &str) -> Result<(), String> {
        let fb = self.reach(at)?;
        let i = self.run(code::FDEC << 28 | fb)?;
        let got = (i.cmd as u32) >> (32 - bits);
        if got != want || i.cmd >> 63 != 0 {
            return Err(format!("FDEC for {what} at {at}: {:08x}, its top {bits} bits expected {want:x}", i.cmd));
        }
        self.check_cursor(what)
    }

    fn vdec(&mut self, at: usize, tbl: u32, len: usize, value: u16, what: &str) -> Result<(), String> {
        let fb = self.reach(at)?;
        let i = self.run(code::VDEC << 28 | tbl << 26 | fb)?;
        let want = (len as u32) << 16 | u32::from(value);
        if i.cmd as u32 != want {
            return Err(format!("VDEC {tbl} for {what} at {at}: {:08x}, expected {want:08x}", i.cmd as u32));
        }
        self.cursor += len;
        self.check_cursor(what)
    }

    fn mbai(&mut self, mb: &Mb) -> Result<(), String> {
        let mut at = mb.pos_mbai;
        for &(len, v) in &mb.mbai_codes {
            let value = match v {
                -1 => 35,
                -3 => 34,
                v => v as u16,
            };
            self.vdec(at, 0, len, value, "macroblock_address_increment")?;
            at += len;
        }
        Ok(())
    }

    fn modes(&mut self, mb: &Mb) -> Result<(), String> {
        let [q, f, b, p, i] = mb.kind;
        let flags = u16::from(q) << 4 | u16::from(f) << 3 | u16::from(b) << 2 | u16::from(p) << 1 | u16::from(i);
        self.vdec(mb.pos_type, 1, mb.type_len, flags, "macroblock_type")?;
        if let Some(at) = mb.pos_motion_type {
            self.fdec(at, 2, 2, "frame_motion_type")?;
        }
        if let Some(at) = mb.pos_dct_type {
            self.fdec(at, 1, u32::from(mb.dct_type), "dct_type")?;
        }
        if let Some(at) = mb.pos_qsc {
            self.fdec(at, 5, mb.qsc, "quantiser_scale_code")?;
        }
        for m in &mb.motion {
            self.vdec(m.pos, 2, m.len, m.code as i16 as u16, "motion_code")?;
            if m.residual_bits > 0 {
                self.fdec(m.pos + m.len, m.residual_bits, m.residual, "motion_residual")?;
            }
        }
        Ok(())
    }

    /// After IDEC or BDEC: the pointer at the start code that follows (SCD) or after the last EOB.
    fn after(&mut self, mb: &Mb, i: &Issued, what: &str) -> Result<(), String> {
        if i.ctrl & ctrl::ECD != 0 {
            return Err(format!("{what}: ECD set, the pointer at {}", self.position()));
        }
        let (to, scd) = match mb.next_start_code {
            Some(p) => (p, true),
            None => (mb.pos_end, false),
        };
        if (i.ctrl & ctrl::SCD != 0) != scd {
            return Err(format!("{what}: SCD {}, expected {scd}", i.ctrl & ctrl::SCD != 0));
        }
        self.cursor = to + PREAMBLE;
        self.check_cursor(what)
    }

    fn bdec(&mut self, mb: &Mb) -> Result<(), String> {
        self.mbai(mb)?;
        self.modes(mb)?;
        let intra = mb.kind[4] == 1;
        if !intra && mb.kind[3] == 0 {
            return Ok(());
        }
        let fb = self.reach(mb.pos_cbp)?;
        let word = code::BDEC << 28 | u32::from(intra) << 27 | u32::from(mb.dc_reset && intra) << 26 | u32::from(mb.dct_type) << 25 | mb.qsc << 16 | fb;
        let i = self.run(word)?;
        let want = manual::raw16_qwords(&mb.blocks, mb.dct_type);
        if i.out != want {
            let first = i.out.iter().zip(&want).position(|(a, b)| a != b);
            return Err(format!(
                "BDEC {word:08x} of macroblock {} ({} qwords, expected 48): first differing qword {first:?}; IPU_CTRL {:08x}, pointer {} from {}, the macroblock {:?}",
                mb.address,
                i.out.len(),
                i.ctrl,
                self.position(),
                mb.pos_cbp + PREAMBLE,
                (mb.kind, mb.cbp, mb.pos_end - mb.pos_cbp, mb.qsc, mb.dc_reset)
            ));
        }
        if !intra && (i.ctrl >> 8 & 0x3F) != mb.cbp {
            return Err(format!("BDEC of macroblock {}: CBP {}, expected {}", mb.address, i.ctrl >> 8 & 0x3F, mb.cbp));
        }
        self.after(mb, &i, "BDEC")
    }

    fn idec(&mut self, s: &st::Slice, o: Options) -> Result<(), String> {
        let first = &s.mbs[0];
        self.mbai(first)?;
        let fb = self.reach(first.pos_type)?;
        let word = code::IDEC << 28
            | u32::from(o.rgb16) << 27
            | u32::from(o.dither) << 26
            | u32::from(o.sgn) << 25
            | u32::from(!self.frame_pred_frame_dct) << 24
            | s.qsc << 16
            | fb;
        let i = self.run(word)?;
        let per = if o.rgb16 { 32 } else { 64 };
        if i.out.len() != per * s.mbs.len() {
            return Err(format!("IDEC of slice {}: {} qwords, expected {}", s.vertical, i.out.len(), per * s.mbs.len()));
        }
        for (k, mb) in s.mbs.iter().enumerate() {
            let want = manual::idec_qwords(&mb.blocks, mb.dct_type, o.th, o.sgn, o.rgb16, o.dither);
            if i.out[per * k..per * (k + 1)] != want[..] {
                let q = i.out[per * k..].iter().zip(&want).position(|(a, b)| a != b).unwrap();
                return Err(format!("IDEC of macroblock {}: qword {q} is {:032x}, expected {:032x}", mb.address, i.out[per * k + q], want[q]));
            }
        }
        self.after(s.mbs.last().unwrap(), &i, "IDEC")
    }

    pub fn set_ctrl(&mut self, c: st::Ctrl) {
        let v = c.intra_dc_precision << 16
            | u32::from(c.alternate_scan) << 20
            | u32::from(c.intra_vlc_format) << 21
            | u32::from(c.q_scale_type) << 22
            | u32::from(c.mpeg1) << 23
            | c.picture_coding_type << 24;
        self.ipu.write_ctrl(v);
    }
}

/// The whole stream through the IPU; the commands issued, or the first disagreement.
pub fn drive(stream: &[u8], dec: &Decoded, o: Options) -> Result<Vec<Issued>, String> {
    let mut d = Driver::new(stream);
    d.run(code::BCLR << 28)?;
    d.run(code::SETTH << 28 | o.th.1 << 16 | o.th.0)?;
    d.run(code::SETIQ << 28)?;
    d.run(code::SETIQ << 28 | 1 << 27)?;
    d.cursor = PREAMBLE;
    d.check_cursor("the preamble's SETIQs")?;
    for e in &dec.events {
        match e {
            Event::Matrix { pos: Some(p), intra, w } => {
                let fb = d.reach(*p)?;
                d.run(code::SETIQ << 28 | u32::from(!*intra) << 27 | fb)?;
                d.cursor += 512;
                d.check_cursor("SETIQ")?;
                d.matrices[usize::from(!*intra)] = *w;
            }
            Event::Matrix { pos: None, intra, w } => {
                if d.matrices[usize::from(!*intra)] != *w {
                    return Err("a default matrix restored after a loaded one, which the driver cannot feed".into());
                }
            }
            Event::Picture { ctrl, frame_pred_frame_dct, .. } => {
                d.set_ctrl(*ctrl);
                d.frame_pred_frame_dct = *frame_pred_frame_dct;
                d.kind = ctrl.picture_coding_type;
            }
            Event::Slice(s) => {
                if o.idec && d.kind == 1 {
                    d.idec(s, o)?;
                } else {
                    for mb in &s.mbs {
                        d.bdec(mb)?;
                    }
                }
            }
        }
    }
    Ok(d.log)
}
