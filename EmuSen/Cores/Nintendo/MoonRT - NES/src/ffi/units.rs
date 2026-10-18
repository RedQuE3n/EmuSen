//! MoonRT's units (EmuSen_CoreAPI.md §6.33): the PPU, the APU, its five channels, the cartridge's board and the
//! cartridge's own facts, each register read from the part that holds it. The names and widths are those the shim's
//! target showed, so a debugger shows what it showed; the channels' and the cartridge's facts are for the NES pack's
//! views alone (EmuSen_Backend.md §19.4, §22).

use emusen_native::core::{UNIT_FIRST_ID, Unit};

use crate::apu::channels::{Envelope, PulseChannel};
use crate::machine::Machine;

pub const PPU: u32 = UNIT_FIRST_ID;
pub const APU: u32 = UNIT_FIRST_ID + 1;
pub const PULSE1: u32 = UNIT_FIRST_ID + 2;
pub const PULSE2: u32 = UNIT_FIRST_ID + 3;
pub const TRIANGLE: u32 = UNIT_FIRST_ID + 4;
pub const NOISE: u32 = UNIT_FIRST_ID + 5;
pub const DMC: u32 = UNIT_FIRST_ID + 6;
pub const BOARD: u32 = UNIT_FIRST_ID + 7;
pub const CARTRIDGE: u32 = UNIT_FIRST_ID + 8;

/// The APU's register names, `$4000` to `$4013`.
const APU_REGISTERS: [&str; 0x14] = [
    "$4000", "$4001", "$4002", "$4003", "$4004", "$4005", "$4006", "$4007", "$4008", "$4009", "$400A", "$400B", "$400C", "$400D", "$400E",
    "$400F", "$4010", "$4011", "$4012", "$4013",
];

type Registers = Vec<(&'static str, u32, i64)>;

/// One unit's name, kind and registers with their values, read from the part that holds it; `None` for an id no unit
/// has. Machine info's list and `debug_registers` both read it, so the two cannot disagree.
fn unit(m: &Machine, id: u32) -> Option<(&'static str, &'static str, Registers)> {
    let b = |v: bool| v as i64;
    let a = &m.bus.apu;
    let envelope = |e: &Envelope| [("ConstantVolume", 1, b(e.constant_volume)), ("Volume", 4, e.volume as i64), ("Decay", 4, e.decay as i64)];
    let pulse = |c: &PulseChannel| {
        let mut r = vec![
            ("Duty", 2, c.duty as i64),
            ("TimerPeriod", 11, c.timer_period as i64),
            ("SweepEnabled", 1, b(c.sweep_enabled)),
            ("SweepNegate", 1, b(c.sweep_negate)),
            ("SweepShift", 3, c.sweep_shift as i64),
            ("OnesComplement", 1, b(c.ones_complement)),
            ("LengthCounter", 8, c.length_counter as i64),
        ];
        r.extend(envelope(&c.envelope));
        r
    };
    let board = &m.bus.board;
    Some(match id {
        PPU => {
            let p = &m.bus.ppu;
            ("PPU", "video", vec![
                ("PPUCTRL", 8, p.control as i64),
                ("PPUMASK", 8, p.mask as i64),
                ("PPUSTATUS", 8, p.status as i64),
                ("OAMADDR", 8, p.oam_address as i64),
                ("v", 15, p.v as i64),
                ("t", 15, p.t as i64),
                ("x", 3, p.fine_x as i64),
                ("w", 1, b(p.write_toggle)),
                ("Scanline", 16, p.scanline as i64),
                ("VBlank", 1, ((p.status >> 7) & 1) as i64),
                ("Sprite0Hit", 1, ((p.status >> 6) & 1) as i64),
                ("SpriteOverflow", 1, ((p.status >> 5) & 1) as i64),
            ])
        }
        APU => {
            let mut r: Registers = a.registers.iter().enumerate().map(|(i, &v)| (APU_REGISTERS[i], 8, v as i64)).collect();
            r.extend([("FrameCounter", 8, a.frame_counter as i64), ("FrameIRQ", 1, b(a.frame_irq_pending))]);
            ("APU", "audio", r)
        }
        PULSE1 => ("Pulse 1", "channel", pulse(&a.pulse1)),
        PULSE2 => ("Pulse 2", "channel", pulse(&a.pulse2)),
        TRIANGLE => {
            let t = &a.triangle;
            ("Triangle", "channel", vec![("LengthCounter", 8, t.length_counter as i64), ("TimerPeriod", 11, t.timer_period as i64), ("ControlFlag", 1, b(t.control_flag))])
        }
        NOISE => {
            let n = &a.noise;
            let mut r = vec![("LengthCounter", 8, n.length_counter as i64), ("PeriodIndex", 4, n.period_index as i64), ("ShortMode", 1, b(n.short_mode))];
            r.extend(envelope(&n.envelope));
            ("Noise", "channel", r)
        }
        DMC => {
            let d = &a.dmc;
            ("DMC", "channel", vec![
                ("BytesRemaining", 12, d.bytes_remaining as i64),
                ("OutputLevel", 7, d.output_level as i64),
                ("RateIndex", 4, d.rate_index as i64),
                ("SampleAddress", 16, d.sample_address as i64),
                ("SampleLength", 12, d.sample_length as i64),
                ("Loop", 1, b(d.r#loop)),
            ])
        }
        BOARD => {
            let mut r = vec![("Mapper", 12, *board.cart.mapper_number as i64)];
            r.extend(board.mapper.debug_registers());
            ("Board", "cartridge", r)
        }
        CARTRIDGE => ("Cartridge", "other", vec![
            ("Mirroring", 3, board.mapper.mirroring(&board.cart) as i64),
            ("ChrRam", 1, b(*board.cart.chr_is_ram)),
            ("Battery", 1, b(*board.cart.has_battery)),
        ]),
        _ => return None,
    })
}

/// Machine info's `units`, in id order.
pub fn units(m: &Machine) -> Vec<Unit> {
    (PPU..=CARTRIDGE)
        .filter_map(|id| unit(m, id).map(|(name, kind, r)| Unit { id, name: name.to_owned(), kind: kind.to_owned(), registers: r.into_iter().map(|(n, bits, _)| (n.to_owned(), bits)).collect() }))
        .collect()
}

/// A unit's values in its registers' order, or `None` for an id no unit has.
pub fn registers(m: &Machine, id: u32) -> Option<Vec<i64>> {
    unit(m, id).map(|(_, _, r)| r.into_iter().map(|(_, _, v)| v).collect())
}
