//! C#'s sixteen `IMapper` boards as one enum; each arm is its C# class, line for line. See Moon_Memory.md §4.

use super::cartridge::{CHR_BANK_SIZE, Cartridge, PRG_BANK_SIZE, RomError, mirroring as m};
use crate::state::{StateReader, StateResult, StateWriter};
use crate::{at_i32, rem};

/// `A12Watcher`: a low-to-high A12 edge, filtered by how many dots it stayed low. See Moon_Memory.md §4.6a.
#[derive(Clone, Debug, PartialEq)]
pub struct A12Watcher {
    pub minimum_dots_low: crate::Skip<i64>,
    pub low_since: i64,
}

impl A12Watcher {
    fn new(minimum_dots_low: i64) -> Self {
        A12Watcher { minimum_dots_low: crate::Skip(minimum_dots_low), low_since: -1 }
    }

    #[inline(always)]
    fn rose(&mut self, address: u16, ppu_clock: i64) -> bool {
        if (address & 0x1000) == 0 {
            if self.low_since < 0 {
                self.low_since = ppu_clock;
            }
            return false;
        }
        let rising = self.low_since >= 0 && ppu_clock - self.low_since >= *self.minimum_dots_low;
        self.low_since = -1;
        rising
    }

    fn write_state(&self, w: &mut StateWriter) {
        w.i64("_lowSince", self.low_since);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.low_since = r.i64()?; // _lowSince
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mmc1 {
    pub chr_bank0: i32,
    pub chr_bank1: i32,
    pub control: i32,
    pub last_write_cycle: i64,
    pub prg_bank: i32,
    pub shift: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UxRom {
    pub bank: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CnRom {
    pub chr_bank: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mmc3 {
    pub a12: A12Watcher,
    pub bank_select: i32,
    pub chr_mode: i32,
    pub irq_counter: i32,
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub irq_reload: bool,
    pub irq_reload_value: i32,
    pub irqs_fired: u64,
    pub mirroring: i32,
    pub prg_mode: i32,
    pub registers: [i32; 8],
    pub wram_enabled: bool,
    pub wram_write_protected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AxRom {
    pub prg_bank: i32,
    pub upper_nametable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mmc2 {
    pub horizontal: bool,
    pub left_bank: [i32; 2],
    pub left_latch: i32,
    pub prg_bank: i32,
    pub right_bank: [i32; 2],
    pub right_latch: i32,
}

/// Color Dreams (11), GxROM (66) and NINA-003-006 (79) keep the same two registers.
#[derive(Clone, Debug, PartialEq)]
pub struct TwoBanks {
    pub chr_bank: i32,
    pub prg_bank: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rambo1 {
    pub a12: A12Watcher,
    pub bank_select: i32,
    pub cpu_clock_divider: i32,
    pub force_clock: bool,
    pub horizontal: bool,
    pub irq_counter: i32,
    pub irq_cycle_mode: bool,
    pub irq_delay: i32,
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub irq_reload: bool,
    pub irq_reload_value: i32,
    pub registers: [i32; 16],
}

#[derive(Clone, Debug, PartialEq)]
pub struct IremH3001 {
    pub chr_banks: [i32; 8],
    pub horizontal: bool,
    pub irq_counter: i32,
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub irq_reload_value: i32,
    pub prg_banks: [i32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sunsoft3 {
    pub chr_banks: [i32; 4],
    pub irq_counter: i32,
    pub irq_enabled: bool,
    pub irq_low_next: bool,
    pub irq_pending: bool,
    pub mirroring: i32,
    pub prg_bank: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sunsoft4 {
    pub chr_banks: [i32; 4],
    pub mirroring: i32,
    pub nametable_banks: [i32; 2],
    pub prg_bank: i32,
    pub prg_ram_enabled: bool,
    pub use_chr_for_nametables: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SunsoftFme7 {
    pub chr_banks: [i32; 8],
    pub command: i32,
    pub irq_counter: i32,
    pub irq_counter_enabled: bool,
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub mirroring: i32,
    pub prg_banks: [i32; 3],
    pub window_control: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Camerica {
    pub mirroring_control_seen: bool,
    pub prg_bank: i32,
    pub upper_page: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mapper {
    Nrom,
    Mmc1(Mmc1),
    UxRom(UxRom),
    CnRom(CnRom),
    Mmc3(Mmc3),
    AxRom(AxRom),
    Mmc2(Mmc2),
    ColorDreams(TwoBanks),
    Rambo1(Rambo1),
    IremH3001(IremH3001),
    GxRom(TwoBanks),
    Sunsoft3(Sunsoft3),
    Sunsoft4(Sunsoft4),
    SunsoftFme7(SunsoftFme7),
    Camerica(Camerica),
    Nina003(TwoBanks),
}

/// `Mmc3.A12MinimumLowDots`: three CPU cycles of three dots.
pub const MMC3_A12_MINIMUM_LOW_DOTS: i64 = 9;
pub const RAMBO1_A12_MINIMUM_LOW_DOTS: i64 = 30;

/// Sunsoft's two-bit mirroring register, as all three boards decode it.
fn sunsoft_mirroring(data: u8) -> i32 {
    match data & 0x03 {
        0 => m::VERTICAL,
        1 => m::HORIZONTAL,
        2 => m::SINGLE_SCREEN_LOWER,
        _ => m::SINGLE_SCREEN_UPPER,
    }
}

impl Mapper {
    /// `Cartridge.CreateMapper`, each board's constructor included.
    pub fn new(cart: &Cartridge) -> Result<Mapper, RomError> {
        let two = || TwoBanks { chr_bank: 0, prg_bank: 0 };
        Ok(match *cart.mapper_number {
            0 => Mapper::Nrom,
            1 => Mapper::Mmc1(Mmc1 { chr_bank0: 0, chr_bank1: 0, control: 0x0C, last_write_cycle: -1, prg_bank: 0, shift: 0x10 }),
            2 => Mapper::UxRom(UxRom { bank: 0 }),
            3 => Mapper::CnRom(CnRom { chr_bank: 0 }),
            4 => Mapper::Mmc3(Mmc3 {
                a12: A12Watcher::new(MMC3_A12_MINIMUM_LOW_DOTS),
                bank_select: 0,
                chr_mode: 0,
                irq_counter: 0,
                irq_enabled: false,
                irq_pending: false,
                irq_reload: false,
                irq_reload_value: 0,
                irqs_fired: 0,
                mirroring: 0,
                prg_mode: 0,
                registers: [0; 8],
                wram_enabled: true,
                wram_write_protected: false,
            }),
            7 => Mapper::AxRom(AxRom { prg_bank: 0, upper_nametable: false }),
            9 => Mapper::Mmc2(Mmc2 { horizontal: false, left_bank: [0; 2], left_latch: 1, prg_bank: 0, right_bank: [0; 2], right_latch: 1 }),
            11 => Mapper::ColorDreams(two()),
            64 => Mapper::Rambo1(Rambo1 {
                a12: A12Watcher::new(RAMBO1_A12_MINIMUM_LOW_DOTS),
                bank_select: 0,
                cpu_clock_divider: 0,
                force_clock: false,
                horizontal: false,
                irq_counter: 0,
                irq_cycle_mode: false,
                irq_delay: 0,
                irq_enabled: false,
                irq_pending: false,
                irq_reload: false,
                irq_reload_value: 0,
                registers: [0; 16],
            }),
            65 => Mapper::IremH3001(IremH3001 {
                chr_banks: [0, 1, 2, 3, 4, 5, 6, 7],
                horizontal: false,
                irq_counter: 0,
                irq_enabled: false,
                irq_pending: false,
                irq_reload_value: 0,
                prg_banks: [0, 1, cart.prg_len() / 0x2000 - 2],
            }),
            66 => Mapper::GxRom(two()),
            67 => Mapper::Sunsoft3(Sunsoft3 { chr_banks: [0; 4], irq_counter: 0, irq_enabled: false, irq_low_next: false, irq_pending: false, mirroring: m::VERTICAL, prg_bank: 0 }),
            68 => Mapper::Sunsoft4(Sunsoft4 { chr_banks: [0; 4], mirroring: m::VERTICAL, nametable_banks: [0; 2], prg_bank: 0, prg_ram_enabled: false, use_chr_for_nametables: false }),
            69 => Mapper::SunsoftFme7(SunsoftFme7 {
                chr_banks: [0; 8],
                command: 0,
                irq_counter: 0,
                irq_counter_enabled: false,
                irq_enabled: false,
                irq_pending: false,
                mirroring: m::VERTICAL,
                prg_banks: [0; 3],
                window_control: 0,
            }),
            71 => Mapper::Camerica(Camerica { mirroring_control_seen: false, prg_bank: 0, upper_page: false }),
            79 => Mapper::Nina003(two()),
            n => return Err(RomError::UnsupportedMapper(n)),
        })
    }

    /// `ClocksOnCpuCycle`, asked once by the bus.
    pub fn clocks_on_cpu_cycle(&self) -> bool {
        matches!(self, Mapper::Rambo1(_) | Mapper::IremH3001(_) | Mapper::Sunsoft3(_) | Mapper::SunsoftFme7(_))
    }

    /// Whether `OnPpuAddress` does anything, so the PPU can skip the call for the boards whose C# body is empty.
    #[inline(always)]
    pub fn watches_ppu_address(&self) -> bool {
        matches!(self, Mapper::Mmc3(_) | Mapper::Rambo1(_))
    }

    #[inline(always)]
    pub fn irq_pending(&self) -> bool {
        match self {
            Mapper::Mmc3(b) => b.irq_pending,
            Mapper::Rambo1(b) => b.irq_pending,
            Mapper::IremH3001(b) => b.irq_pending,
            Mapper::Sunsoft3(b) => b.irq_pending,
            Mapper::SunsoftFme7(b) => b.irq_pending,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn supplies_nametables(&self) -> bool {
        match self {
            Mapper::Sunsoft4(b) => b.use_chr_for_nametables,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn mirroring(&self, cart: &Cartridge) -> i32 {
        match self {
            Mapper::Mmc1(b) => match b.control & 0x03 {
                0 => m::SINGLE_SCREEN_LOWER,
                1 => m::SINGLE_SCREEN_UPPER,
                2 => m::VERTICAL,
                _ => m::HORIZONTAL,
            },
            Mapper::Mmc3(b) => {
                if *cart.header_mirroring == m::FOUR_SCREEN {
                    m::FOUR_SCREEN
                } else if (b.mirroring & 0x01) != 0 {
                    m::HORIZONTAL
                } else {
                    m::VERTICAL
                }
            }
            Mapper::AxRom(b) => {
                if b.upper_nametable {
                    m::SINGLE_SCREEN_UPPER
                } else {
                    m::SINGLE_SCREEN_LOWER
                }
            }
            Mapper::Mmc2(b) => {
                if b.horizontal {
                    m::HORIZONTAL
                } else {
                    m::VERTICAL
                }
            }
            Mapper::Rambo1(b) => {
                if *cart.header_mirroring == m::FOUR_SCREEN {
                    m::FOUR_SCREEN
                } else if b.horizontal {
                    m::HORIZONTAL
                } else {
                    m::VERTICAL
                }
            }
            Mapper::IremH3001(b) => {
                if b.horizontal {
                    m::HORIZONTAL
                } else {
                    m::VERTICAL
                }
            }
            Mapper::Sunsoft3(b) => b.mirroring,
            Mapper::Sunsoft4(b) => b.mirroring,
            Mapper::SunsoftFme7(b) => b.mirroring,
            Mapper::Camerica(b) => {
                if b.mirroring_control_seen {
                    if b.upper_page { m::SINGLE_SCREEN_UPPER } else { m::SINGLE_SCREEN_LOWER }
                } else {
                    *cart.header_mirroring
                }
            }
            _ => *cart.header_mirroring,
        }
    }

    /// `ReadPrg`, CPU `$4020`-`$FFFF`.
    #[inline]
    pub fn read_prg(&self, cart: &Cartridge, address: u16) -> u8 {
        let a = address as i32;
        match self {
            Mapper::Nrom | Mapper::CnRom(_) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                crate::at(&cart.prg_rom, (a - 0x8000) & (cart.prg_len() - 1))
            }
            Mapper::Mmc1(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let last = cart.prg_banks() - 1;
                let upper = address >= 0xC000;
                let bank = match (b.control >> 2) & 0x03 {
                    0 | 1 => (b.prg_bank & 0x0E) + if upper { 1 } else { 0 },
                    2 => {
                        if upper {
                            b.prg_bank
                        } else {
                            0
                        }
                    }
                    _ => {
                        if upper {
                            last
                        } else {
                            b.prg_bank
                        }
                    }
                };
                cart.prg_mod(bank * PRG_BANK_SIZE + (a & 0x3FFF))
            }
            Mapper::UxRom(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let bank = if address < 0xC000 { b.bank } else { cart.prg_banks() - 1 };
                cart.prg_mod(bank * PRG_BANK_SIZE + (a & 0x3FFF))
            }
            Mapper::Mmc3(b) => {
                if address < 0x8000 {
                    return if b.wram_enabled { cart.prg_ram_at(address) } else { 0 };
                }
                let window = (a - 0x8000) / 0x2000;
                let last = cart.prg_len() / 0x2000 - 1;
                let bank = if b.prg_mode == 0 {
                    match window {
                        0 => b.registers[6],
                        1 => b.registers[7],
                        2 => last - 1,
                        _ => last,
                    }
                } else {
                    match window {
                        0 => last - 1,
                        1 => b.registers[7],
                        2 => b.registers[6],
                        _ => last,
                    }
                };
                cart.prg_mod(bank * 0x2000 + (a & 0x1FFF))
            }
            Mapper::AxRom(AxRom { prg_bank, .. }) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                cart.prg_mod(prg_bank * 0x8000 + (a & 0x7FFF))
            }
            Mapper::Mmc2(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let window = (a - 0x8000) / 0x2000;
                let last = cart.prg_len() / 0x2000 - 1;
                let bank = if window == 0 { b.prg_bank } else { last - (3 - window) };
                cart.prg_mod(bank * 0x2000 + (a & 0x1FFF))
            }
            Mapper::ColorDreams(b) | Mapper::GxRom(b) | Mapper::Nina003(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                cart.prg_mod(b.prg_bank * 0x8000 + (a & 0x7FFF))
            }
            Mapper::Rambo1(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let window = (a - 0x8000) / 0x2000;
                let last = cart.prg_len() / 0x2000 - 1;
                let bank = if window == 3 {
                    last
                } else if (b.bank_select & 0x40) != 0 {
                    match window {
                        0 => b.registers[15],
                        1 => b.registers[6],
                        _ => b.registers[7],
                    }
                } else {
                    match window {
                        0 => b.registers[6],
                        1 => b.registers[7],
                        _ => b.registers[15],
                    }
                };
                cart.prg_mod(bank * 0x2000 + (a & 0x1FFF))
            }
            Mapper::IremH3001(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let window = (a - 0x8000) / 0x2000;
                let bank = if window == 3 { cart.prg_len() / 0x2000 - 1 } else { b.prg_banks[window as usize] };
                cart.prg_mod(bank * 0x2000 + (a & 0x1FFF))
            }
            Mapper::Sunsoft3(Sunsoft3 { prg_bank, .. }) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let last = cart.prg_len() / 0x4000 - 1;
                let bank = if address < 0xC000 { *prg_bank } else { last };
                cart.prg_mod(bank * 0x4000 + (a & 0x3FFF))
            }
            Mapper::Sunsoft4(b) => {
                if address < 0x8000 {
                    return if b.prg_ram_enabled { cart.prg_ram_at(address) } else { 0 };
                }
                let last = cart.prg_len() / 0x4000 - 1;
                let bank = if address < 0xC000 { b.prg_bank } else { last };
                cart.prg_mod(bank * 0x4000 + (a & 0x3FFF))
            }
            Mapper::SunsoftFme7(b) => {
                if (0x6000..0x8000).contains(&address) {
                    if (b.window_control & 0x40) == 0 {
                        return cart.prg_mod((b.window_control & 0x3F) * 0x2000 + (a & 0x1FFF));
                    }
                    return if (b.window_control & 0x80) != 0 { cart.prg_ram_at(address) } else { 0 };
                }
                if address < 0x6000 {
                    return 0;
                }
                let window = (a - 0x8000) / 0x2000;
                let bank = if window == 3 { cart.prg_len() / 0x2000 - 1 } else { b.prg_banks[window as usize] };
                cart.prg_mod(bank * 0x2000 + (a & 0x1FFF))
            }
            Mapper::Camerica(b) => {
                if address < 0x8000 {
                    return cart.prg_ram_at(address);
                }
                let bank = if address < 0xC000 { b.prg_bank } else { cart.prg_banks() - 1 };
                cart.prg_mod(bank * PRG_BANK_SIZE + (a & 0x3FFF))
            }
        }
    }

    /// `WritePrg`; `cart.cpu_cycle` was stamped by the bus first.
    pub fn write_prg(&mut self, cart: &mut Cartridge, address: u16, data: u8) {
        let d = data as i32;
        match self {
            Mapper::Nrom => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                }
            }
            Mapper::Mmc1(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                let consecutive = cart.cpu_cycle == b.last_write_cycle.wrapping_add(1);
                b.last_write_cycle = cart.cpu_cycle;
                if consecutive {
                    return;
                }
                if (data & 0x80) != 0 {
                    b.shift = 0x10;
                    b.control |= 0x0C;
                    return;
                }
                let complete = (b.shift & 0x01) != 0;
                b.shift = (b.shift >> 1) | ((d & 0x01) << 4);
                if !complete {
                    return;
                }
                let value = b.shift & 0x1F;
                b.shift = 0x10;
                match address & 0xE000 {
                    0x8000 => b.control = value,
                    0xA000 => b.chr_bank0 = value,
                    0xC000 => b.chr_bank1 = value,
                    _ => b.prg_bank = value & 0x0F,
                }
            }
            Mapper::UxRom(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                } else {
                    b.bank = d & 0x0F;
                }
            }
            Mapper::CnRom(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                } else {
                    b.chr_bank = d & 0x03;
                }
            }
            Mapper::Mmc3(b) => {
                if address < 0x8000 {
                    if b.wram_enabled && !b.wram_write_protected {
                        cart.set_prg_ram(address, data);
                    }
                    return;
                }
                match address & 0xE001 {
                    0x8000 => {
                        b.bank_select = d & 0x07;
                        b.prg_mode = (d >> 6) & 0x01;
                        b.chr_mode = (d >> 7) & 0x01;
                    }
                    0x8001 => {
                        let value = if b.bank_select <= 1 { d & 0xFE } else { d };
                        set_i32(&mut b.registers, b.bank_select, value);
                    }
                    0xA000 => b.mirroring = d,
                    0xA001 => {
                        b.wram_enabled = (d & 0x80) != 0;
                        b.wram_write_protected = (d & 0x40) != 0;
                    }
                    0xC000 => b.irq_reload_value = d,
                    0xC001 => {
                        b.irq_counter = 0;
                        b.irq_reload = true;
                    }
                    0xE000 => {
                        b.irq_enabled = false;
                        b.irq_pending = false;
                    }
                    0xE001 => b.irq_enabled = true,
                    _ => {}
                }
            }
            Mapper::AxRom(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                b.prg_bank = d & 0x07;
                b.upper_nametable = (d & 0x10) != 0;
            }
            Mapper::Mmc2(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                match address >> 12 {
                    0xA => b.prg_bank = d & 0x0F,
                    0xB => b.left_bank[0] = d & 0x1F,
                    0xC => b.left_bank[1] = d & 0x1F,
                    0xD => b.right_bank[0] = d & 0x1F,
                    0xE => b.right_bank[1] = d & 0x1F,
                    0xF => b.horizontal = (d & 0x01) != 0,
                    _ => {}
                }
            }
            Mapper::ColorDreams(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                b.prg_bank = d & 0x03;
                b.chr_bank = (d >> 4) & 0x0F;
            }
            Mapper::GxRom(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                b.prg_bank = (d >> 4) & 0x03;
                b.chr_bank = d & 0x03;
            }
            Mapper::Nina003(b) => {
                if (address & 0xE100) == 0x4100 {
                    b.prg_bank = (d >> 3) & 0x01;
                    b.chr_bank = d & 0x07;
                    return;
                }
                if (0x6000..0x8000).contains(&address) {
                    cart.set_prg_ram(address, data);
                }
            }
            Mapper::Rambo1(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                match address & 0xE001 {
                    0x8000 => b.bank_select = d,
                    0x8001 => b.registers[(b.bank_select & 0x0F) as usize] = d,
                    0xA000 => b.horizontal = (d & 0x01) != 0,
                    0xC000 => b.irq_reload_value = d,
                    0xC001 => {
                        let cycle_mode = (d & 0x01) != 0;
                        if b.irq_cycle_mode && !cycle_mode {
                            b.force_clock = true;
                        }
                        b.irq_cycle_mode = cycle_mode;
                        if cycle_mode {
                            b.cpu_clock_divider = 0;
                        }
                        b.irq_reload = true;
                    }
                    0xE000 => {
                        b.irq_enabled = false;
                        b.irq_pending = false;
                        b.irq_delay = 0;
                    }
                    0xE001 => b.irq_enabled = true,
                    _ => {}
                }
            }
            Mapper::IremH3001(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                match address {
                    0x8000 => b.prg_banks[0] = d,
                    0xA000 => b.prg_banks[1] = d,
                    0xC000 => b.prg_banks[2] = d,
                    0x9001 => b.horizontal = (d & 0x80) != 0,
                    0x9003 => {
                        b.irq_enabled = (d & 0x80) != 0;
                        b.irq_pending = false;
                    }
                    0x9004 => {
                        b.irq_counter = b.irq_reload_value;
                        b.irq_pending = false;
                    }
                    0x9005 => b.irq_reload_value = (b.irq_reload_value & 0x00FF) | (d << 8),
                    0x9006 => b.irq_reload_value = (b.irq_reload_value & 0xFF00) | d,
                    0xB000..=0xB007 => b.chr_banks[(address & 0x07) as usize] = d,
                    _ => {}
                }
            }
            Mapper::Sunsoft3(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                match address & 0xF800 {
                    0x8800 => b.chr_banks[0] = d,
                    0x9800 => b.chr_banks[1] = d,
                    0xA800 => b.chr_banks[2] = d,
                    0xB800 => b.chr_banks[3] = d,
                    0xC800 => {
                        b.irq_counter = if b.irq_low_next { (b.irq_counter & 0xFF00) | d } else { (b.irq_counter & 0x00FF) | (d << 8) };
                        b.irq_low_next = !b.irq_low_next;
                    }
                    0xD800 => {
                        b.irq_enabled = (d & 0x10) != 0;
                        b.irq_low_next = false;
                        b.irq_pending = false;
                    }
                    0xE800 => b.mirroring = sunsoft_mirroring(data),
                    0xF800 => b.prg_bank = d,
                    _ => {}
                }
            }
            Mapper::Sunsoft4(b) => {
                if address < 0x8000 {
                    if b.prg_ram_enabled {
                        cart.set_prg_ram(address, data);
                    }
                    return;
                }
                match address & 0xF000 {
                    0x8000 => b.chr_banks[0] = d,
                    0x9000 => b.chr_banks[1] = d,
                    0xA000 => b.chr_banks[2] = d,
                    0xB000 => b.chr_banks[3] = d,
                    0xC000 => b.nametable_banks[0] = d | 0x80,
                    0xD000 => b.nametable_banks[1] = d | 0x80,
                    0xE000 => {
                        b.mirroring = sunsoft_mirroring(data);
                        b.use_chr_for_nametables = (d & 0x10) != 0;
                    }
                    0xF000 => {
                        b.prg_bank = d & 0x07;
                        b.prg_ram_enabled = (d & 0x10) != 0;
                    }
                    _ => {}
                }
            }
            Mapper::SunsoftFme7(b) => {
                if (0x6000..0x8000).contains(&address) {
                    if (b.window_control & 0xC0) == 0xC0 {
                        cart.set_prg_ram(address, data);
                    }
                    return;
                }
                match address & 0xE000 {
                    0x8000 => b.command = d & 0x0F,
                    0xA000 => match b.command {
                        c if c <= 7 => set_i32(&mut b.chr_banks, c, d),
                        8 => b.window_control = d,
                        c if c <= 0xB => b.prg_banks[(c - 9) as usize] = d & 0x3F,
                        0xC => b.mirroring = sunsoft_mirroring(data),
                        0xD => {
                            b.irq_enabled = (d & 0x01) != 0;
                            b.irq_counter_enabled = (d & 0x80) != 0;
                            b.irq_pending = false;
                        }
                        0xE => b.irq_counter = (b.irq_counter & 0xFF00) | d,
                        _ => b.irq_counter = (b.irq_counter & 0x00FF) | (d << 8),
                    },
                    _ => {}
                }
            }
            Mapper::Camerica(b) => {
                if address < 0x8000 {
                    cart.set_prg_ram(address, data);
                    return;
                }
                if (0x9000..0xA000).contains(&address) {
                    b.mirroring_control_seen = true;
                    b.upper_page = (d & 0x10) != 0;
                    return;
                }
                if address >= 0xC000 {
                    b.prg_bank = d;
                }
            }
        }
    }

    /// Where a PPU pattern address lands in CHR, before the `% Chr.Length` every banked board applies.
    #[inline(always)]
    fn chr_offset(&self, address: u16) -> i32 {
        let a = address as i32;
        match self {
            Mapper::Mmc1(b) => {
                if (b.control & 0x10) != 0 {
                    let bank = if address < 0x1000 { b.chr_bank0 } else { b.chr_bank1 };
                    bank * 0x1000 + (a & 0x0FFF)
                } else {
                    (b.chr_bank0 & 0x1E) * 0x1000 + (a & 0x1FFF)
                }
            }
            Mapper::CnRom(b) => b.chr_bank * CHR_BANK_SIZE + (a & 0x1FFF),
            Mapper::Mmc3(b) => {
                let mut window = (a & 0x1FFF) / 0x0400;
                if b.chr_mode != 0 {
                    window ^= 0x04;
                }
                let r = &b.registers;
                let page = match window {
                    0 => r[0] & 0xFE,
                    1 => r[0] | 0x01,
                    2 => r[1] & 0xFE,
                    3 => r[1] | 0x01,
                    4 => r[2],
                    5 => r[3],
                    6 => r[4],
                    _ => r[5],
                };
                page * 0x0400 + (a & 0x03FF)
            }
            Mapper::Mmc2(b) => {
                let bank = if address < 0x1000 { at_i32(&b.left_bank, b.left_latch) } else { at_i32(&b.right_bank, b.right_latch) };
                bank * 0x1000 + (a & 0x0FFF)
            }
            Mapper::ColorDreams(b) | Mapper::GxRom(b) | Mapper::Nina003(b) => b.chr_bank * CHR_BANK_SIZE + (a & 0x1FFF),
            Mapper::Rambo1(b) => {
                let mut window = (a & 0x1FFF) / 0x0400;
                if (b.bank_select & 0x80) != 0 {
                    window ^= 0x04;
                }
                let one_k = (b.bank_select & 0x20) != 0;
                let r = &b.registers;
                let page = match window {
                    0 => r[0],
                    1 => {
                        if one_k {
                            r[8]
                        } else {
                            r[0] + 1
                        }
                    }
                    2 => r[1],
                    3 => {
                        if one_k {
                            r[9]
                        } else {
                            r[1] + 1
                        }
                    }
                    4 => r[2],
                    5 => r[3],
                    6 => r[4],
                    _ => r[5],
                };
                page * 0x0400 + (a & 0x03FF)
            }
            Mapper::IremH3001(b) => b.chr_banks[((a & 0x1FFF) / 0x0400) as usize] * 0x0400 + (a & 0x03FF),
            Mapper::Sunsoft3(b) => b.chr_banks[((a & 0x1FFF) / 0x0800) as usize] * 0x0800 + (a & 0x07FF),
            Mapper::Sunsoft4(b) => b.chr_banks[((a & 0x1FFF) / 0x0800) as usize] * 0x0800 + (a & 0x07FF),
            Mapper::SunsoftFme7(b) => b.chr_banks[((a & 0x1FFF) / 0x0400) as usize] * 0x0400 + (a & 0x03FF),
            Mapper::Nrom | Mapper::UxRom(_) | Mapper::AxRom(_) | Mapper::Camerica(_) => a & 0x1FFF,
        }
    }

    /// `ReadChr`; MMC2's latch moves after the fetch is served, which is why this takes `&mut self`.
    #[inline]
    pub fn read_chr(&mut self, cart: &Cartridge, address: u16) -> u8 {
        match self {
            Mapper::Nrom | Mapper::UxRom(_) | Mapper::AxRom(_) | Mapper::Camerica(_) => crate::at(&cart.chr, (address & 0x1FFF) as i32),
            Mapper::Mmc2(_) => {
                let value = cart.chr_mod(self.chr_offset(address));
                if let Mapper::Mmc2(b) = self {
                    if address == 0x0FD8 {
                        b.left_latch = 0;
                    } else if address == 0x0FE8 {
                        b.left_latch = 1;
                    } else if (0x1FD8..=0x1FDF).contains(&address) {
                        b.right_latch = 0;
                    } else if (0x1FE8..=0x1FEF).contains(&address) {
                        b.right_latch = 1;
                    }
                }
                value
            }
            _ => cart.chr_mod(self.chr_offset(address)),
        }
    }

    /// `WriteChr`: a CHR ROM board swallows the write.
    pub fn write_chr(&mut self, cart: &mut Cartridge, address: u16, data: u8) {
        match self {
            Mapper::Nrom | Mapper::UxRom(_) | Mapper::AxRom(_) | Mapper::Camerica(_) => cart.write_chr_ram((address & 0x1FFF) as i32, data),
            Mapper::CnRom(_) => cart.write_chr_ram((address & 0x1FFF) as i32, data),
            Mapper::Mmc2(_) => {
                if !*cart.chr_is_ram {
                    return;
                }
                let offset = rem(self.chr_offset(address), cart.chr_len());
                cart.write_chr_ram(offset, data);
            }
            _ => {
                if *cart.chr_is_ram {
                    let offset = rem(self.chr_offset(address), cart.chr_len());
                    cart.write_chr_ram(offset, data);
                }
            }
        }
    }

    /// `ReadNametable`: Sunsoft-4's CHR standing in for CIRAM.
    pub fn read_nametable(&self, cart: &Cartridge, address: u16) -> u8 {
        let Mapper::Sunsoft4(b) = self else { return 0 };
        let index = (address as i32 - 0x2000) & 0x0FFF;
        let table = index / 0x0400;
        let register = match b.mirroring {
            m::HORIZONTAL => table >> 1,
            m::VERTICAL => table & 1,
            m::SINGLE_SCREEN_UPPER => 1,
            _ => 0,
        };
        cart.chr_mod(b.nametable_banks[register as usize] * 0x0400 + (index % 0x0400))
    }

    /// `OnPpuAddress`: every address the PPU drives, for the two boards that count A12.
    #[inline]
    pub fn on_ppu_address(&mut self, address: u16, ppu_clock: i64) {
        match self {
            Mapper::Mmc3(b) => {
                if !b.a12.rose(address, ppu_clock) {
                    return;
                }
                if b.irq_counter == 0 || b.irq_reload {
                    b.irq_counter = b.irq_reload_value;
                } else {
                    b.irq_counter = b.irq_counter.wrapping_sub(1);
                }
                if b.irq_counter == 0 && b.irq_enabled {
                    if !b.irq_pending {
                        b.irqs_fired = b.irqs_fired.wrapping_add(1);
                    }
                    b.irq_pending = true;
                }
                b.irq_reload = false;
            }
            Mapper::Rambo1(b) => {
                if b.irq_cycle_mode {
                    return;
                }
                if b.a12.rose(address, ppu_clock) {
                    rambo_clock(b, 2);
                }
            }
            _ => {}
        }
    }

    /// `OnCpuCycle`, for the boards whose `ClocksOnCpuCycle` is true.
    #[inline]
    pub fn on_cpu_cycle(&mut self) {
        match self {
            Mapper::Rambo1(b) => {
                if b.irq_delay > 0 {
                    b.irq_delay -= 1;
                    if b.irq_delay == 0 {
                        b.irq_pending = true;
                    }
                }
                if !b.irq_cycle_mode && !b.force_clock {
                    return;
                }
                b.cpu_clock_divider = (b.cpu_clock_divider + 1) & 0x03;
                if b.cpu_clock_divider != 0 {
                    return;
                }
                rambo_clock(b, 1);
                b.force_clock = false;
            }
            Mapper::IremH3001(b) => {
                if !b.irq_enabled {
                    return;
                }
                b.irq_counter = (b.irq_counter - 1) & 0xFFFF;
                if b.irq_counter != 0 {
                    return;
                }
                b.irq_enabled = false;
                b.irq_pending = true;
            }
            Mapper::Sunsoft3(b) => {
                if !b.irq_enabled {
                    return;
                }
                b.irq_counter = b.irq_counter.wrapping_sub(1);
                if b.irq_counter >= 0 {
                    return;
                }
                b.irq_counter = 0xFFFF;
                b.irq_enabled = false;
                b.irq_pending = true;
            }
            Mapper::SunsoftFme7(b) => {
                if !b.irq_counter_enabled {
                    return;
                }
                b.irq_counter = b.irq_counter.wrapping_sub(1);
                if b.irq_counter >= 0 {
                    return;
                }
                b.irq_counter = 0xFFFF;
                if b.irq_enabled {
                    b.irq_pending = true;
                }
            }
            _ => {}
        }
    }

    /// The board's fields as C# walks `Cart.Mapper`.
    pub fn write_state(&self, w: &mut StateWriter) {
        match self {
            Mapper::Nrom => {}
            Mapper::Mmc1(b) => b.write_state(w),
            Mapper::UxRom(b) => w.i32("_bank", b.bank),
            Mapper::CnRom(b) => w.i32("_chrBank", b.chr_bank),
            Mapper::Mmc3(b) => b.write_state(w),
            Mapper::AxRom(b) => b.write_state(w),
            Mapper::Mmc2(b) => b.write_state(w),
            Mapper::ColorDreams(b) | Mapper::GxRom(b) | Mapper::Nina003(b) => b.write_state(w),
            Mapper::Rambo1(b) => b.write_state(w),
            Mapper::IremH3001(b) => b.write_state(w),
            Mapper::Sunsoft3(b) => b.write_state(w),
            Mapper::Sunsoft4(b) => b.write_state(w),
            Mapper::SunsoftFme7(b) => b.write_state(w),
            Mapper::Camerica(b) => b.write_state(w),
        }
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        match self {
            Mapper::Nrom => Ok(()),
            Mapper::Mmc1(b) => b.read_state(r),
            Mapper::UxRom(b) => {
                b.bank = r.i32()?; // _bank
                Ok(())
            }
            Mapper::CnRom(b) => {
                b.chr_bank = r.i32()?; // _chrBank
                Ok(())
            }
            Mapper::Mmc3(b) => b.read_state(r),
            Mapper::AxRom(b) => b.read_state(r),
            Mapper::Mmc2(b) => b.read_state(r),
            Mapper::ColorDreams(b) | Mapper::GxRom(b) | Mapper::Nina003(b) => b.read_state(r),
            Mapper::Rambo1(b) => b.read_state(r),
            Mapper::IremH3001(b) => b.read_state(r),
            Mapper::Sunsoft3(b) => b.read_state(r),
            Mapper::Sunsoft4(b) => b.read_state(r),
            Mapper::SunsoftFme7(b) => b.read_state(r),
            Mapper::Camerica(b) => b.read_state(r),
        }
    }
}

/// C#'s `array[index] = value` on an `int` table whose index a state can set.
#[inline(always)]
fn set_i32(table: &mut [i32], index: i32, value: i32) {
    match table.get_mut(index as usize) {
        Some(v) => *v = value,
        None => crate::fault(crate::Fault::IndexOutOfRange),
    }
}

/// `Rambo1.ClockCounter`: a reload lands one or two above the latch. See Moon_Memory.md §4.15.
#[inline]
fn rambo_clock(b: &mut Rambo1, delay: i32) {
    if b.irq_reload {
        b.irq_counter = (b.irq_reload_value + if b.irq_reload_value <= 1 { 1 } else { 2 }) & 0xFF;
        b.irq_reload = false;
    } else if b.irq_counter == 0 {
        b.irq_counter = (b.irq_reload_value + 1) & 0xFF;
    }
    b.irq_counter = (b.irq_counter - 1) & 0xFF;
    if b.irq_counter == 0 && b.irq_enabled {
        b.irq_delay = delay;
    }
}

impl Mmc1 {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("_chrBank0", self.chr_bank0);
        w.i32("_chrBank1", self.chr_bank1);
        w.i32("_control", self.control);
        w.i64("_lastWriteCycle", self.last_write_cycle);
        w.i32("_prgBank", self.prg_bank);
        w.i32("_shift", self.shift);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.chr_bank0 = r.i32()?; // _chrBank0
        self.chr_bank1 = r.i32()?; // _chrBank1
        self.control = r.i32()?; // _control
        self.last_write_cycle = r.i64()?; // _lastWriteCycle
        self.prg_bank = r.i32()?; // _prgBank
        self.shift = r.i32()?; // _shift
        Ok(())
    }
}

impl Mmc3 {
    fn write_state(&self, w: &mut StateWriter) {
        w.group_class("_a12", |w| self.a12.write_state(w));
        w.i32("_bankSelect", self.bank_select);
        w.i32("_chrMode", self.chr_mode);
        w.i32("_irqCounter", self.irq_counter);
        w.bool("_irqEnabled", self.irq_enabled);
        w.bool("_irqPending", self.irq_pending);
        w.bool("_irqReload", self.irq_reload);
        w.i32("_irqReloadValue", self.irq_reload_value);
        w.u64("_irqsFired", self.irqs_fired);
        w.i32("_mirroring", self.mirroring);
        w.i32("_prgMode", self.prg_mode);
        w.i32s("_registers", &self.registers);
        w.bool("_wramEnabled", self.wram_enabled);
        w.bool("_wramWriteProtected", self.wram_write_protected);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        if r.present()? {
            self.a12.read_state(r)?;
        }
        self.bank_select = r.i32()?; // _bankSelect
        self.chr_mode = r.i32()?; // _chrMode
        self.irq_counter = r.i32()?; // _irqCounter
        self.irq_enabled = r.bool()?; // _irqEnabled
        self.irq_pending = r.bool()?; // _irqPending
        self.irq_reload = r.bool()?; // _irqReload
        self.irq_reload_value = r.i32()?; // _irqReloadValue
        self.irqs_fired = r.u64()?; // _irqsFired
        self.mirroring = r.i32()?; // _mirroring
        self.prg_mode = r.i32()?; // _prgMode
        r.i32s(&mut self.registers)?; // _registers
        self.wram_enabled = r.bool()?; // _wramEnabled
        self.wram_write_protected = r.bool()?; // _wramWriteProtected
        Ok(())
    }
}

impl AxRom {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("_prgBank", self.prg_bank);
        w.bool("_upperNametable", self.upper_nametable);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.prg_bank = r.i32()?; // _prgBank
        self.upper_nametable = r.bool()?; // _upperNametable
        Ok(())
    }
}

impl Mmc2 {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("_horizontal", self.horizontal);
        w.i32s("_leftBank", &self.left_bank);
        w.i32("_leftLatch", self.left_latch);
        w.i32("_prgBank", self.prg_bank);
        w.i32s("_rightBank", &self.right_bank);
        w.i32("_rightLatch", self.right_latch);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.horizontal = r.bool()?; // _horizontal
        r.i32s(&mut self.left_bank)?; // _leftBank
        self.left_latch = r.i32()?; // _leftLatch
        self.prg_bank = r.i32()?; // _prgBank
        r.i32s(&mut self.right_bank)?; // _rightBank
        self.right_latch = r.i32()?; // _rightLatch
        Ok(())
    }
}

impl TwoBanks {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("_chrBank", self.chr_bank);
        w.i32("_prgBank", self.prg_bank);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.chr_bank = r.i32()?; // _chrBank
        self.prg_bank = r.i32()?; // _prgBank
        Ok(())
    }
}

impl Rambo1 {
    fn write_state(&self, w: &mut StateWriter) {
        w.group_class("_a12", |w| self.a12.write_state(w));
        w.i32("_bankSelect", self.bank_select);
        w.i32("_cpuClockDivider", self.cpu_clock_divider);
        w.bool("_forceClock", self.force_clock);
        w.bool("_horizontal", self.horizontal);
        w.i32("_irqCounter", self.irq_counter);
        w.bool("_irqCycleMode", self.irq_cycle_mode);
        w.i32("_irqDelay", self.irq_delay);
        w.bool("_irqEnabled", self.irq_enabled);
        w.bool("_irqPending", self.irq_pending);
        w.bool("_irqReload", self.irq_reload);
        w.i32("_irqReloadValue", self.irq_reload_value);
        w.i32s("_registers", &self.registers);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        if r.present()? {
            self.a12.read_state(r)?;
        }
        self.bank_select = r.i32()?; // _bankSelect
        self.cpu_clock_divider = r.i32()?; // _cpuClockDivider
        self.force_clock = r.bool()?; // _forceClock
        self.horizontal = r.bool()?; // _horizontal
        self.irq_counter = r.i32()?; // _irqCounter
        self.irq_cycle_mode = r.bool()?; // _irqCycleMode
        self.irq_delay = r.i32()?; // _irqDelay
        self.irq_enabled = r.bool()?; // _irqEnabled
        self.irq_pending = r.bool()?; // _irqPending
        self.irq_reload = r.bool()?; // _irqReload
        self.irq_reload_value = r.i32()?; // _irqReloadValue
        r.i32s(&mut self.registers)?; // _registers
        Ok(())
    }
}

impl IremH3001 {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32s("_chrBanks", &self.chr_banks);
        w.bool("_horizontal", self.horizontal);
        w.i32("_irqCounter", self.irq_counter);
        w.bool("_irqEnabled", self.irq_enabled);
        w.bool("_irqPending", self.irq_pending);
        w.i32("_irqReloadValue", self.irq_reload_value);
        w.i32s("_prgBanks", &self.prg_banks);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.i32s(&mut self.chr_banks)?; // _chrBanks
        self.horizontal = r.bool()?; // _horizontal
        self.irq_counter = r.i32()?; // _irqCounter
        self.irq_enabled = r.bool()?; // _irqEnabled
        self.irq_pending = r.bool()?; // _irqPending
        self.irq_reload_value = r.i32()?; // _irqReloadValue
        r.i32s(&mut self.prg_banks)?; // _prgBanks
        Ok(())
    }
}

impl Sunsoft3 {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32s("_chrBanks", &self.chr_banks);
        w.i32("_irqCounter", self.irq_counter);
        w.bool("_irqEnabled", self.irq_enabled);
        w.bool("_irqLowNext", self.irq_low_next);
        w.bool("_irqPending", self.irq_pending);
        w.i32("_mirroring", self.mirroring);
        w.i32("_prgBank", self.prg_bank);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.i32s(&mut self.chr_banks)?; // _chrBanks
        self.irq_counter = r.i32()?; // _irqCounter
        self.irq_enabled = r.bool()?; // _irqEnabled
        self.irq_low_next = r.bool()?; // _irqLowNext
        self.irq_pending = r.bool()?; // _irqPending
        self.mirroring = r.i32()?; // _mirroring
        self.prg_bank = r.i32()?; // _prgBank
        Ok(())
    }
}

impl Sunsoft4 {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32s("_chrBanks", &self.chr_banks);
        w.i32("_mirroring", self.mirroring);
        w.i32s("_nametableBanks", &self.nametable_banks);
        w.i32("_prgBank", self.prg_bank);
        w.bool("_prgRamEnabled", self.prg_ram_enabled);
        w.bool("_useChrForNametables", self.use_chr_for_nametables);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.i32s(&mut self.chr_banks)?; // _chrBanks
        self.mirroring = r.i32()?; // _mirroring
        r.i32s(&mut self.nametable_banks)?; // _nametableBanks
        self.prg_bank = r.i32()?; // _prgBank
        self.prg_ram_enabled = r.bool()?; // _prgRamEnabled
        self.use_chr_for_nametables = r.bool()?; // _useChrForNametables
        Ok(())
    }
}

impl SunsoftFme7 {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32s("_chrBanks", &self.chr_banks);
        w.i32("_command", self.command);
        w.i32("_irqCounter", self.irq_counter);
        w.bool("_irqCounterEnabled", self.irq_counter_enabled);
        w.bool("_irqEnabled", self.irq_enabled);
        w.bool("_irqPending", self.irq_pending);
        w.i32("_mirroring", self.mirroring);
        w.i32s("_prgBanks", &self.prg_banks);
        w.i32("_windowControl", self.window_control);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.i32s(&mut self.chr_banks)?; // _chrBanks
        self.command = r.i32()?; // _command
        self.irq_counter = r.i32()?; // _irqCounter
        self.irq_counter_enabled = r.bool()?; // _irqCounterEnabled
        self.irq_enabled = r.bool()?; // _irqEnabled
        self.irq_pending = r.bool()?; // _irqPending
        self.mirroring = r.i32()?; // _mirroring
        r.i32s(&mut self.prg_banks)?; // _prgBanks
        self.window_control = r.i32()?; // _windowControl
        Ok(())
    }
}

impl Camerica {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("_mirroringControlSeen", self.mirroring_control_seen);
        w.i32("_prgBank", self.prg_bank);
        w.bool("_upperPage", self.upper_page);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.mirroring_control_seen = r.bool()?; // _mirroringControlSeen
        self.prg_bank = r.i32()?; // _prgBank
        self.upper_page = r.bool()?; // _upperPage
        Ok(())
    }
}
