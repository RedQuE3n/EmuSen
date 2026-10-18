//! The Sega CD's facts for a frontend, which the assembly composes with the Genesis's and numbers (Nephrite_Attachments.md
//! §3.6): its boot ROM's images, its setting, its two save memories, its drive and how a disc is opened and known.

use emusen_native::core::{Battery, Choice, Drive, Effect, Firmware, Replacement, Scope, Setting, SettingKind, Settings};

use super::cdd::{self, Disc, Mode};
use super::{MegaCd, RamCart};

/// Where a disc image's system area begins: the user data of its first sector, at 0 in a 2048-byte-sector image and
/// after the 16 bytes of sync and sector header in a raw 2352-byte one.
pub const DISC_SIGNATURE: &[u8; 14] = b"SEGADISCSYSTEM";

pub fn disc_offset(image: &[u8]) -> Option<usize> {
    [0usize, 16].into_iter().find(|&o| image.get(o..o + DISC_SIGNATURE.len()) == Some(DISC_SIGNATURE.as_slice()))
}

/// Whether an image is a disc's: a cue sheet, a CHD, or sectors that carry the system area's signature.
pub fn is_disc(image: &[u8]) -> bool {
    emusen_disc::is_sheet(image) || image.starts_with(emusen_disc::disc::CHD_MAGIC) || disc_offset(image).is_some()
}

/// A Sega CD disc from an image and its companions: a sheet with its tracks, a CHD, a raw track, or 2,048-byte
/// sectors that begin with the system area's signature, the console's own mark of a disc (Nephrite_Native.md §51).
pub fn open_disc(image: emusen_native::core::Kept, companions: Vec<emusen_native::core::Kept>) -> Result<Disc, emusen_disc::Refusal> {
    let lone = if image.starts_with(DISC_SIGNATURE) { emusen_disc::Lone::User } else { emusen_disc::Lone::Detect };
    emusen_disc::open(image, companions, emusen_disc::IsoForm::Mode1, lone)
}

/// The disc's fingerprint, which a state is held to: its tracks, its lead-out and its first two sectors, the system
/// area's.
pub fn fingerprint(disc: &mut Disc) -> u64 {
    disc.fingerprint([0, 1])
}

/// The system area's first 2,048 bytes as the drive reads them from sector 00:02:00, for telling what the disc is.
pub fn system_area(disc: &mut Disc) -> Vec<u8> {
    let mut raw = [0u8; 2352];
    let k = disc.read(emusen_disc::msf::PROGRAM_START, &mut raw);
    if k.audio() { Vec::new() } else { raw[16..16 + 2048].to_vec() }
}

/// The firmware page quotes the first sentence alone, in the big-screen menu as its footer, so it stays short (Nephrite_OpenBios.md §17).
const BIOS_COST: &str = "Discs boot, and saves keep the console's own backup RAM format. It is Nephrite's own Sega CD BIOS, written from Sega's manuals, and the documented calls answer as the manuals describe. A game that calls a routine no manual names, reads the BIOS's code or depends on the exact timing of its loops may run differently; discs of every region start, since the security check is not made; games cannot yet reach a backup RAM cartridge; the screen without a disc is Nephrite's own, with no CD player or memory manager.";

/// The boot ROM of a market's console (`U`, `E` or `J`), as the frontend's file `which`.
pub(crate) fn bios_firmware(region: char, which: u32) -> Firmware {
    let market = match region {
        'J' => "Japanese Mega-CD",
        'E' => "European Mega-CD",
        _ => "American Sega CD",
    };
    let name = format!("bios_CD_{region}.bin");
    Firmware {
        which,
        name: name.clone(),
        label: format!("The {market}'s BIOS (131,072 bytes)"),
        size: super::BIOS_SIZE as u64,
        required: false,
        parts: vec![vec![name]],
        replacement: Some(Replacement::Accuracy { cost: BIOS_COST.into() }),
        md5: Vec::new(),
    }
}

/// The setting that puts a Sega CD beneath a cartridge (mode 1), read at create.
pub(crate) const BENEATH_KEY: &str = "mcd_beneath";
pub(crate) const BENEATH_VALUE: &str = "mcd.beneath";

pub(crate) fn beneath_setting() -> Setting {
    Setting {
        key: BENEATH_KEY.into(),
        label: "Sega CD beneath the cartridge".into(),
        help: "For a cartridge made to use the Sega CD from the cartridge slot: whether a Sega CD is attached beneath the console, running the player's Sega CD BIOS where one is given and Nephrite's own otherwise. Its backup RAM is kept in a file of its own. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(vec![
            Choice { value: "none".into(), label: "None".into(), help: None },
            Choice { value: BENEATH_VALUE.into(), label: "Sega CD".into(), help: None },
        ]),
        default: "none".into(),
        scope: Scope::Create,
        category: Some("Sega CD".into()),
        effect: Effect::None,
        advanced: false,
        // Not shown while the Sega CD is offered to no player (Nephrite_Native.md §53.2).
        hidden: true,
        restart: true,
    }
}

/// The setting that plugs a backup RAM cartridge into the cartridge slot, read at create.
pub(in crate::mcd) const RAM_CART_KEY: &str = "ramcart";
const RAM_CART_VALUE: &str = "mcd.ramcart";

pub(crate) fn ram_cart_setting() -> Setting {
    Setting {
        key: RAM_CART_KEY.into(),
        label: "Backup RAM cartridge".into(),
        help: "For the Sega CD: whether Sega's Backup RAM Cartridge, 128 KiB of saves beside the 8 KiB inside the unit, is plugged into the cartridge slot. Its saves are kept in a file of their own. Takes effect when a game is next loaded.".into(),
        kind: SettingKind::Choice(vec![
            Choice { value: "none".into(), label: "None".into(), help: None },
            Choice { value: RAM_CART_VALUE.into(), label: "Backup RAM Cartridge (128 KiB)".into(), help: None },
        ]),
        default: "none".into(),
        scope: Scope::Create,
        category: Some("Sega CD".into()),
        effect: Effect::None,
        advanced: false,
        // Not shown while the Sega CD is offered to no player (Nephrite_Native.md §53.2).
        hidden: true,
        restart: true,
    }
}

/// The Sega CD's two save memories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveMemory {
    /// The backup RAM inside the unit, the console's own (EmuSen_SaveMedia.md §7.3).
    Backup,
    /// The backup RAM cartridge in the slot, a removable one.
    Cartridge,
}

/// Each save memory's battery file as the frontend's file `which`: its kind, suffix, medium and label.
pub(crate) fn battery(memory: SaveMemory, which: u32) -> Battery {
    match memory {
        SaveMemory::Backup => Battery::console(which, ".brm").medium("mcd.backup").label("Inside the console"),
        SaveMemory::Cartridge => Battery::removable(which, ".crm").medium("mcd.ramcart").label("Cartridge slot"),
    }
}

impl MegaCd {
    /// The settings read at create: the backup RAM cartridge plugged in when asked for.
    pub(crate) fn apply_settings(&mut self, settings: &Settings) {
        if settings.get(RAM_CART_KEY) == Some(RAM_CART_VALUE) {
            self.hw.cart = Some(RamCart::new(RamCart::SEGA));
        }
    }

    /// The save memories present, the backup RAM first.
    pub(crate) fn save_memories(&self) -> Vec<SaveMemory> {
        let mut v = vec![SaveMemory::Backup];
        if self.hw.cart.is_some() {
            v.push(SaveMemory::Cartridge);
        }
        v
    }

    pub(crate) fn save_memory(&self, memory: SaveMemory) -> Option<&[u8]> {
        match memory {
            SaveMemory::Backup => Some(&self.hw.bram),
            SaveMemory::Cartridge => self.hw.cart.as_ref().map(|c| c.bytes.as_slice()),
        }
    }

    /// A battery file's bytes loaded into its memory, as far as both reach.
    pub(crate) fn load_save_memory(&mut self, memory: SaveMemory, data: &[u8]) {
        let into = match memory {
            SaveMemory::Backup => Some(&mut self.hw.bram),
            SaveMemory::Cartridge => self.hw.cart.as_mut().map(|c| &mut c.bytes),
        };
        if let Some(b) = into {
            let n = b.len().min(data.len());
            b[..n].copy_from_slice(&data[..n]);
        }
    }

    /// The drive as the frontend lists it.
    pub(crate) fn drive(&self) -> Drive {
        Drive { drive: cdd::DRIVE, label: "CD drive".into(), kind: "cd".into(), lid_open: self.lid_open(), loaded: self.hw.cdd.disc.is_some() }
    }

    pub(crate) fn lid_open(&self) -> bool {
        self.hw.cdd.mode == Mode::TrayOpen
    }

    /// The tray, opened or closed as the player would: the drive stops; closed, it reads its disc's TOC again.
    pub(crate) fn set_lid(&mut self, open: bool) {
        self.hw.cdd.set_tray(open);
    }

    /// A disc put in the drive, or taken out, and the fingerprint a state is held to with it.
    pub(crate) fn insert(&mut self, mut disc: Option<Disc>) {
        self.disc_print = disc.as_mut().map_or(0, fingerprint);
        self.hw.cdd.disc = disc;
        self.hw.cdd.named = None;
    }
}
