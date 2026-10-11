//! The controller's audio output to the SPU's CD input (psx-spx "ATV0".."ADPCTL"): CD-DA and XA-ADPCM at 44,100 Hz
//! through the volume matrix, held in a fixed ring the SPU takes one frame from per sample (Diamond_CDROM.md §7).

/// The ring's frames: about three XA sectors' output, so that a sector's burst never overruns it.
pub const RING: usize = 8192;

#[derive(Clone)]
pub struct Output {
    pub left: Box<[i16; RING]>,
    pub right: Box<[i16; RING]>,
    pub head: u16,
    pub len: u16,
    /// ATV0-ATV3 as written (L→L, L→R, R→R, R→L) and as applied by ADPCTL's CHNGATV.
    pub written: [u8; 4],
    pub volume: [u8; 4],
    /// ADPCTL's ADPMUTE, which silences XA-ADPCM, and the Mute command's, which silences everything.
    pub xa_muted: bool,
    pub muted: bool,
    /// Whether the newest frames came from XA, for HSTS's ADPBUSY.
    pub from_xa: bool,
}

impl Default for Output {
    fn default() -> Output {
        Output { left: Box::new([0; RING]), right: Box::new([0; RING]), head: 0, len: 0, written: [0x80, 0, 0x80, 0], volume: [0x80, 0, 0x80, 0], xa_muted: false, muted: false, from_xa: false }
    }
}

impl Output {
    /// A frame through the matrix: left = L·ATV0 + R·ATV3, right = R·ATV2 + L·ATV1, over 80h, saturated.
    pub fn push(&mut self, frame: [i16; 2], xa: bool) {
        let [l, r] = [frame[0] as i32, frame[1] as i32];
        let v = self.volume.map(|x| x as i32);
        let silent = self.muted || (xa && self.xa_muted);
        let out = if silent { [0, 0] } else { [((l * v[0] + r * v[3]) >> 7).clamp(-0x8000, 0x7FFF), ((r * v[2] + l * v[1]) >> 7).clamp(-0x8000, 0x7FFF)] };
        if self.len as usize == RING {
            self.head = ((self.head as usize + 1) % RING) as u16;
            self.len -= 1;
        }
        let at = (self.head as usize + self.len as usize) % RING;
        self.left[at] = out[0] as i16;
        self.right[at] = out[1] as i16;
        self.len += 1;
        self.from_xa = xa;
    }

    /// The SPU's next CD frame, silence when the ring is empty.
    pub fn take(&mut self) -> [i16; 2] {
        if self.len == 0 {
            return [0, 0];
        }
        let f = [self.left[self.head as usize], self.right[self.head as usize]];
        self.head = ((self.head as usize + 1) % RING) as u16;
        self.len -= 1;
        f
    }

    pub fn apply_volume(&mut self) {
        self.volume = self.written;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_matrix_mixes_and_saturates() {
        let mut o = Output::default();
        o.push([1000, -2000], false);
        assert_eq!(o.take(), [1000, -2000], "80h is unity");
        o.written = [0x40; 4];
        o.apply_volume();
        o.push([1000, -2000], false);
        assert_eq!(o.take(), [-500, -500], "40h each way is mono");
        o.written = [0xFF, 0, 0xFF, 0];
        o.apply_volume();
        o.push([30000, -30000], false);
        assert_eq!(o.take(), [0x7FFF, -0x8000]);
        o.xa_muted = true;
        o.push([100, 100], true);
        o.push([100, 0], false);
        assert_eq!((o.take(), o.take()[0] > 0, o.take()), ([0, 0], true, [0, 0]));
    }
}
