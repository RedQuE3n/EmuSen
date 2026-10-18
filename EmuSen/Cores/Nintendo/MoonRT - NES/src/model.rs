//! The console a game runs on, the NTSC NES, the PAL NES or the Dendy, and every timing fact in which they differ. See
//! Moon_PAL.md.

/// The console whose clocks the machine runs: its CPU, its PPU and its APU's tables.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Model {
    /// The 2A03 and 2C02.
    #[default]
    Ntsc,
    /// The 2A07 and 2C07.
    Pal,
    /// The famiclones of the former USSR: a 6538 PPU at PAL's clocks, a CPU at three dots a cycle with the NTSC APU.
    Dendy,
}

impl Model {
    /// The master clock, as a ratio: 21,477,272 Hz, or six times PAL's 4.43361875 MHz subcarrier (Moon_PAL.md §2).
    pub const fn master_clock_hz(self) -> (u64, u64) {
        match self {
            Model::Ntsc => (21_477_272, 1),
            Model::Pal | Model::Dendy => (53_203_425, 2),
        }
    }

    pub const fn master_clocks_per_cpu_cycle(self) -> i64 {
        match self {
            Model::Ntsc => 12,
            Model::Pal => 16,
            Model::Dendy => 15,
        }
    }

    pub const fn master_clocks_per_dot(self) -> i64 {
        match self {
            Model::Ntsc => 4,
            Model::Pal | Model::Dendy => 5,
        }
    }

    pub const fn master_clocks_per_scanline(self) -> i64 {
        341 * self.master_clocks_per_dot()
    }

    pub const fn scanlines(self) -> i32 {
        match self {
            Model::Ntsc => 262,
            Model::Pal | Model::Dendy => 312,
        }
    }

    /// The line whose dot 1 sets the vblank flag: the Dendy draws 51 lines after the picture, not 1.
    pub const fn vblank_scanline(self) -> i32 {
        match self {
            Model::Dendy => 291,
            Model::Ntsc | Model::Pal => 241,
        }
    }

    pub const fn pre_render_scanline(self) -> i32 {
        self.scanlines() - 1
    }

    /// Only the 2C02 drops a dot from the pre-render line of an odd frame while rendering.
    pub const fn skips_odd_dot(self) -> bool {
        matches!(self, Model::Ntsc)
    }

    /// The 2C07 and the 6538 blank the top line and two pixels at each side to black.
    pub const fn black_border(self) -> bool {
        !matches!(self, Model::Ntsc)
    }

    /// 3.2 dots a CPU cycle: one cycle in five takes a fourth dot.
    pub const fn long_cycles(self) -> bool {
        matches!(self, Model::Pal)
    }

    /// The 2A07 halts for DMA only on an instruction's first cycle.
    pub const fn dma_on_first_cycle(self) -> bool {
        matches!(self, Model::Pal)
    }

    /// The 2A07's noise, DMC and frame counter tables; the Dendy's CPU uses the NTSC ones.
    pub const fn pal_apu(self) -> bool {
        matches!(self, Model::Pal)
    }

    /// The CPU's clock as the mixer counts it; NTSC's is the figure the goldens were recorded with.
    pub fn cpu_clock_hz(self) -> f64 {
        match self {
            Model::Ntsc => 1789773.0,
            Model::Pal | Model::Dendy => {
                let (num, den) = self.master_clock_hz();
                num as f64 / den as f64 / self.master_clocks_per_cpu_cycle() as f64
            }
        }
    }

    /// The frame's length as the frontend paces it: the master clock over a frame of whole lines.
    pub const fn frame_rate(self) -> (u64, u64) {
        let (num, den) = self.master_clock_hz();
        (num, den * self.scanlines() as u64 * self.master_clocks_per_scanline() as u64)
    }

    /// NES 2.0's numbering of byte 12's timing, which a state records.
    pub const fn code(self) -> u8 {
        match self {
            Model::Ntsc => 0,
            Model::Pal => 1,
            Model::Dendy => 3,
        }
    }

    pub const fn from_code(code: u8) -> Option<Model> {
        match code {
            0 => Some(Model::Ntsc),
            1 => Some(Model::Pal),
            3 => Some(Model::Dendy),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Model::Ntsc => "NTSC",
            Model::Pal => "PAL",
            Model::Dendy => "Dendy",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_consoles_clocks_are_the_documented_ones() {
        let hz = |m: Model| m.frame_rate().0 as f64 / m.frame_rate().1 as f64;
        assert_eq!(Model::Ntsc.frame_rate(), (21_477_272, 262 * 341 * 4));
        assert!((hz(Model::Pal) - 50.0070).abs() < 0.0001, "{}", hz(Model::Pal));
        assert!((hz(Model::Dendy) - 50.0070).abs() < 0.0001);
        assert!((Model::Pal.cpu_clock_hz() - 1_662_607.0).abs() < 0.1, "{}", Model::Pal.cpu_clock_hz());
        assert!((Model::Dendy.cpu_clock_hz() - 1_773_448.0).abs() < 0.6, "{}", Model::Dendy.cpu_clock_hz());
        let dots = |m: Model| (m.scanlines() as i64 * 341 * m.master_clocks_per_dot()) as f64 / m.master_clocks_per_cpu_cycle() as f64;
        assert_eq!((dots(Model::Pal), dots(Model::Dendy)), (33247.5, 35464.0));
        for m in [Model::Ntsc, Model::Pal, Model::Dendy] {
            assert_eq!(Model::from_code(m.code()), Some(m));
        }
    }
}
