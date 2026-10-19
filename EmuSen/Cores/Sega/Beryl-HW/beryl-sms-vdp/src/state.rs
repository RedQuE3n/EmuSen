//! The chip's state block (Beryl_SmsVdp.md §6), a fixed layout in its console's state. The picture is not in it: a
//! console that saves at its frame's end, as Endou does, saves between pictures.

use emusen_native::{StateReader, StateWriter, Truncated};

use crate::Vdp;
use crate::compose::{self, Sprite};
use crate::tms;

impl Vdp {
    pub fn write_state(&self, w: &mut StateWriter) {
        w.u8("VdpVariant", self.variant.id);
        w.bools("PalGgMode", &[self.pal, self.gg_mode]);
        w.bytes("VdpRegisters", &self.regs);
        w.bytes("Vram", &self.vram);
        w.bytes("Cram", &self.cram);
        w.u16("Address", self.address);
        w.bytes("CodeBufferLatch", &[self.code, self.buffer, self.gg_latch]);
        w.bools("SecondFlags", &[self.second, self.frame_flag, self.line_flag, self.overflow, self.collision, self.tms_sprites]);
        w.bytes("CounterScrollsHFifth", &[self.line_counter, self.hscroll, self.vscroll, self.h_latch, self.fifth]);
        w.u64("VdpClock", self.clock);
        w.u64("LineStart", self.line_start);
        w.u32s("LineDrawnHeight", &[self.line, self.drawn, self.height]);
        w.u8("Event", self.event);
        w.u8("SpriteCount", self.sprites.count);
        for s in &self.sprites.sprites {
            w.bytes("Sprite", &[s.x as u8, (s.x >> 8) as u8, s.wide as u8, s.planes[0], s.planes[1], s.planes[2], s.planes[3]]);
        }
    }

    /// A block of this chip's variant and standard, else nothing is changed and `false` is returned; the sprite pixels
    /// are made again from the sprites.
    pub fn read_state(&mut self, r: &mut StateReader) -> Result<bool, Truncated> {
        let id = r.u8()?;
        let mut pg = [false; 2];
        r.bools(&mut pg)?;
        let mut v = self.clone();
        r.bytes(&mut v.regs)?;
        r.bytes(&mut v.vram)?;
        r.bytes(&mut v.cram)?;
        v.address = r.u16()?;
        let mut b = [0u8; 3];
        r.bytes(&mut b)?;
        (v.code, v.buffer, v.gg_latch) = (b[0], b[1], b[2]);
        let mut f = [false; 6];
        r.bools(&mut f)?;
        (v.second, v.frame_flag, v.line_flag, v.overflow, v.collision, v.tms_sprites) = (f[0], f[1], f[2], f[3], f[4], f[5]);
        let mut c = [0u8; 5];
        r.bytes(&mut c)?;
        (v.line_counter, v.hscroll, v.vscroll, v.h_latch, v.fifth) = (c[0], c[1], c[2], c[3], c[4]);
        v.clock = r.u64()?;
        v.line_start = r.u64()?;
        let mut l = [0u32; 3];
        r.u32s(&mut l)?;
        (v.line, v.drawn, v.height) = (l[0], l[1], l[2]);
        v.event = r.u8()?;
        v.sprites.count = r.u8()?;
        for s in v.sprites.sprites.iter_mut() {
            let mut b = [0u8; 7];
            r.bytes(&mut b)?;
            *s = Sprite { x: i16::from_le_bytes([b[0], b[1]]), wide: b[2] != 0, planes: [b[3], b[4], b[5], b[6]] };
        }
        let sane = id == self.variant.id
            && pg[0] == self.pal
            && v.line < v.lines()
            && v.event < 3
            && v.sprites.count <= 8
            && v.drawn <= 256
            && matches!(v.height, 192 | 224 | 240)
            && v.clock >= v.line_start
            && v.clock - v.line_start < crate::timing::LINE_CLOCKS;
        if !sane {
            return Ok(false);
        }
        v.gg_mode = pg[1];
        if v.tms_sprites { tms::sprite_pixels(&mut v.sprites) } else { compose::sprite_pixels(&mut v.sprites) }
        *self = v;
        Ok(true)
    }
}
