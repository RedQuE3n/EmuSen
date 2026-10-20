//! The consoles Endou models and what each differs in: the I/O chip's control port, the region it reads back, the Reset
//! button, what an undriven read returns, the VDP, the PSG and the work RAM (Endou_Native.md §9.4, §11), and the Game
//! Gear's (Jadeite_Native.md §2). The one owner of these facts.

/// A real console. The Japanese Master System joins with the YM2413 at stage 9 (Endou_Plan.md §5.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    /// Sega's Mark III: no TH function, its TH outputs reading low as a Japanese console's do, and no Reset button [R
    /// SMS Power!'s "Peripheral Ports", "Reset Button"; Endou_Disputes.md D-3].
    MarkIII,
    /// The first Master System (315-5124): a Reset button, undriven reads the bus's last byte [R MacDonald's
    /// `smstech` §3, §4].
    Sms1,
    /// The Master System II (315-5246): no Reset button, undriven reads $FF [R `smstech` §3, §4].
    Sms2,
    /// The SG-1000: no memory or I/O control port, its pads read through the joystick ports [R SMS Power!'s "I/O Port
    /// Map - SG-1000 and SC-3000"], a TMS9918A, a discrete SN76489AN and 1 KiB of work RAM.
    Sg1000,
    /// The Game Gear: "a NTSC SMS2" to its software [R MacDonald's `msvdp` §14] with the 315-5378 and an LCD. Its own
    /// ports $00-$06 and its stereo are stage J2's; until then it reads as the Master System II there.
    GameGear,
}

impl Model {
    pub fn id(self) -> &'static str {
        match self {
            Model::MarkIII => "mark3",
            Model::Sms1 => "sms1",
            Model::Sms2 => "sms2",
            Model::Sg1000 => "sg1000",
            Model::GameGear => "gg",
        }
    }

    pub fn byte(self) -> u8 {
        match self {
            Model::MarkIII => 0,
            Model::Sms1 => 1,
            Model::Sms2 => 2,
            Model::Sg1000 => 3,
            Model::GameGear => 4,
        }
    }

    pub fn of_byte(b: u8) -> Option<Model> {
        [Model::MarkIII, Model::Sms1, Model::Sms2, Model::Sg1000, Model::GameGear].into_iter().find(|m| m.byte() == b)
    }

    /// The VDP: the 315-5124 in the Mark III and the first Master System, the 315-5246 in the second [R MacDonald's
    /// `msvdp` §1], the TMS9918A in the SG-1000 [R SMS Power!'s "Palette"; MacDonald's `sc3000h` §2 for its family].
    pub fn vdp(self) -> beryl_sms_vdp::Variant {
        match self {
            Model::MarkIII | Model::Sms1 => beryl_sms_vdp::Variant::SMS1,
            Model::Sms2 => beryl_sms_vdp::Variant::SMS2,
            Model::Sg1000 => beryl_sms_vdp::Variant::TMS9918A,
            Model::GameGear => beryl_sms_vdp::Variant::GG,
        }
    }

    /// Whether the console runs PAL where the market is Europe: the Game Gear, every one of whose units is NTSC, does not
    /// [R MacDonald's `msvdp` §14: "262 scanlines per frame", the machines "the same throughout the world"].
    pub fn has_pal(self) -> bool {
        self != Model::GameGear
    }

    /// The PSG: the discrete SN76489AN in the SG-1000, Sega's copy in the VDP from the Mark III on (Endou_Disputes.md
    /// D-0).
    pub fn psg(self) -> beryl_sn76489::Variant {
        if self == Model::Sg1000 { beryl_sn76489::Variant::DISCRETE } else { beryl_sn76489::Variant::SEGA }
    }

    /// Work RAM: the SG-1000's 1 KiB, the others' 8 KiB [D: `smstech` §2; `sc3000h`].
    pub fn ram(self) -> usize {
        if self == Model::Sg1000 { 0x400 } else { 0x2000 }
    }

    /// Whether the V and H counters answer at ports $40-$7F: the Sega VDPs'; the TMS9918A has none, and the SG-1000
    /// reads the open bus there [R SMS Power!'s "I/O Port Map - SG-1000 and SC-3000"].
    pub fn has_counters(self) -> bool {
        self != Model::Sg1000
    }

    /// Whether ports $3E and $3F exist: every console from the Mark III on.
    pub fn has_control_ports(self) -> bool {
        self != Model::Sg1000
    }

    /// Whether TH and TR follow port $3F's directions and levels: the export consoles. The Mark III's TR stays an
    /// input [A]; its TH is `io`'s (Endou_Disputes.md D-3).
    pub fn io_control(self) -> bool {
        matches!(self, Model::Sms1 | Model::Sms2)
    }

    pub fn has_reset_button(self) -> bool {
        self == Model::Sms1
    }

    /// Whether a read nothing drives returns the bus's last byte, as MacDonald found on the first Master System, rather
    /// than $FF, as on the Master System II. The Mark III and the SG-1000 are taken as the first [A], the Game Gear as the
    /// second [R SMS Power!'s "I/O Port Map - Game Gear": its undecoded reads $FF].
    pub fn floating_bus(self) -> bool {
        !matches!(self, Model::Sms2 | Model::GameGear)
    }

    /// Whether the console has a BIOS: the Master System and the Master System II; the Mark III has none
    /// (Endou_Disputes.md D-14), nor the SG-1000.
    pub fn has_bios(self) -> bool {
        matches!(self, Model::Sms1 | Model::Sms2)
    }

    /// Whether a read with the BIOS and the cartridge both on is the two ANDed, as on the Master System II, rather than
    /// the cartridge's, as on the first Master System [R MacDonald's `smstech` §4].
    pub fn bios_and_cartridge_are_anded(self) -> bool {
        self == Model::Sms2
    }

    /// The memory control port's value when the cartridge starts with no BIOS run before it: $AB, the value both export
    /// BIOSes MacDonald watched leave for a cartridge [R `smstech` §4]; the SG-1000 has no such port.
    pub fn memory_control_at_start(self) -> u8 {
        0xAB
    }
}
