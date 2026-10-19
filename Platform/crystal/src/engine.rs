//! The session over one v1 machine, the C# `CoreEngine`'s part that loads and keeps it: a game or a console started
//! empty, its firmware found, its save media placed and its battery files kept, its discs, its settings, and the
//! machine's events drained (EmuSen_Backend.md §29.2). The frame and its order are `frame`'s; this holds what they
//! work on.

use std::ffi::CString;
use std::sync::Mutex;

use emusen_core_host::{Create, InputLatch, Library, Machine};
use emusen_galaxia::disc_set::{DiscFiles, DiscSet};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::Tree;
use emusen_native::core::sys::{FrameInfo, caps, event, flags};
use emusen_native::json::Value;

use crate::battery::BatterySave;
use crate::firmware::Entry as Firmware;
use crate::info::{self, BatteryFile, SystemDetail};
use crate::machine_info::{self, MachineInfo, SettingDesc};
use crate::media::{self, Choice, Placement};

/// What every machine of the session is made with.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub battery_disabled: bool,
    pub audio_buffer_max_samples: u32,
}

/// A refusal in the C# engine's words, and the status the API answers it with.
#[derive(Clone, Debug)]
pub struct Refused {
    pub status: i32,
    pub words: String,
}

impl Refused {
    pub fn fault(words: String) -> Refused {
        Refused { status: emusen_session_service::abi::status::FAULT, words }
    }

    pub fn io(words: String) -> Refused {
        Refused { status: crate::abi::status::IO, words }
    }
}

/// What a machine is made again from, when a create-time setting changes before its first frame.
#[derive(Clone, Debug)]
enum Made {
    Nothing,
    Game(Option<SystemDetail>),
    Console(String),
}

/// What loading needs of the runtime: the tree, which extensions an archive's entry may have, the firmware folders.
pub struct Ctx<'a> {
    pub tree: &'a Tree,
    pub is_image: &'a dyn Fn(&str) -> bool,
    pub folders: &'a [String],
}

/// Where a firmware entry the last machine was given was found: its folder, the names read, whether by content.
struct Located {
    which: u32,
    folder: String,
    names: Vec<String>,
    content: bool,
}

pub struct Engine {
    pub lib: &'static Library,
    pub info: Value,
    /// Core info's `name` and its display name, the engine's name a player sees.
    pub name: String,
    pub display_name: String,
    pub settings: Vec<SettingDesc>,
    values: Vec<(String, String)>,
    pub machine: Option<Machine>,
    pub mi: MachineInfo,
    pub rom_path: String,
    pub console: String,
    pub started_empty: Option<String>,
    set: Option<DiscSet>,
    pub discs: Vec<String>,
    pub disc_index: i32,
    close_lid_at: i64,
    loans: Vec<(u32, DiscFiles)>,
    battery: Vec<(u32, BatterySave)>,
    placement: Option<Placement>,
    pub firmware_notice: Option<String>,
    pub media_notice: Option<String>,
    pub notes: Vec<(String, String)>,
    pub latch: InputLatch,
    said: Mutex<Vec<String>>,
    fresh: bool,
    pub options: Options,
    pub media_choices: Vec<(String, Choice)>,
    located: Vec<Located>,
    pub shape: FrameInfo,
    pub audio_rate: i32,
    pub skip_rendering: bool,
    log: std::collections::VecDeque<String>,
    pub logged_lines: i64,
    log_text: String,
    made: Made,
}

fn int(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

impl Engine {
    /// An engine over a loaded library, its values the schema's defaults with each stored one it takes.
    pub fn new(lib: &'static Library, info: Value, stored: &[(String, String)], options: Options) -> Engine {
        let name = info.get("name").and_then(Value::as_str).unwrap_or("").to_string();
        let display_name = info.get("display_name").and_then(Value::as_str).map_or_else(|| name.clone(), str::to_string);
        let settings = lib.settings_text().map(|t| machine_info::settings(&t)).unwrap_or_default();
        let mut values: Vec<(String, String)> = settings.iter().map(|s| (s.key.clone(), s.default.clone())).collect();
        for (key, value) in stored {
            if let Some(s) = settings.iter().find(|s| s.key == *key)
                && s.accepts(value)
                && let Some(slot) = values.iter_mut().find(|(k, _)| k == key)
            {
                slot.1 = value.clone();
            }
        }
        Engine {
            lib,
            info,
            name,
            display_name,
            settings,
            values,
            machine: None,
            mi: MachineInfo::default(),
            rom_path: String::new(),
            console: String::new(),
            started_empty: None,
            set: None,
            discs: Vec::new(),
            disc_index: 0,
            close_lid_at: -1,
            loans: Vec::new(),
            battery: Vec::new(),
            placement: None,
            firmware_notice: None,
            media_notice: None,
            notes: Vec::new(),
            latch: InputLatch::default(),
            said: Mutex::new(Vec::new()),
            fresh: false,
            options,
            media_choices: Vec::new(),
            located: Vec::new(),
            shape: FrameInfo::default(),
            audio_rate: 44100,
            skip_rendering: false,
            log: std::collections::VecDeque::new(),
            logged_lines: 0,
            log_text: String::new(),
            made: Made::Nothing,
        }
    }

    pub fn claims(&self, bit: u64) -> bool {
        self.lib.claims(bit)
    }

    /// A line of the runtime's own, kept until the session raises it (§29.11, Q2).
    pub fn say(&self, line: String) {
        self.said.lock().unwrap_or_else(|p| p.into_inner()).push(line);
    }

    /// The runtime's lines said since the last call.
    pub fn take_said(&self) -> Vec<String> {
        std::mem::take(&mut *self.said.lock().unwrap_or_else(|p| p.into_inner()))
    }

    pub fn value(&self, key: &str) -> Option<&str> {
        self.values.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// Every value as `key=value` lines: create-time and run-time keys at create, run-time keys only after it.
    fn settings_text(&self, create_scope: bool) -> String {
        self.settings
            .iter()
            .filter(|s| create_scope || !s.create_scope)
            .map(|s| format!("{}={}", s.key, self.value(&s.key).unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn systems(&self) -> Vec<SystemDetail> {
        info::systems(&self.info)
    }

    fn system_entries(&self) -> Vec<(String, bool, Vec<Firmware>)> {
        crate::firmware::systems_of(&self.info).into_iter().map(|(id, _, _, fw)| (id.clone(), false, fw)).collect()
    }

    /// The firmware entries a machine is given: each whole by its name and size, else its first split form, from the
    /// first folder holding it, part n as file `which + n`.
    fn firmware_files(&mut self, entries: &[Firmware], folders: &[String]) -> Vec<(u32, Vec<u8>)> {
        let mut files = Vec::new();
        self.located.clear();
        for f in entries {
            let Some(found) = emusen_firmware::find(&f.request(), folders) else { continue };
            for (n, part) in found.parts.iter().enumerate() {
                files.push((f.which + n as u32, part.clone()));
            }
            self.located.push(Located { which: f.which, folder: found.folder.clone(), names: found.names.clone(), content: found.content });
        }
        files
    }

    /// The machine's firmware line for each entry it reports, `FirmwareLines`.
    fn firmware_lines(&self, wanted: &[Firmware]) -> Vec<String> {
        self.mi
            .firmware
            .iter()
            .filter_map(|(which, source)| {
                let f = wanted.iter().find(|f| f.which == *which)?;
                let tail = match source.as_str() {
                    "file" => match self.located.iter().find(|l| l.which == *which) {
                        Some(at) => format!("the player's {}, in {}", if at.content { format!("{}, recognised by its content", at.names[0]) } else { f.name.clone() }, at.folder),
                        None => format!("the player's {}", f.name),
                    },
                    "replacement" => format!("{}'s own replacement", self.display_name),
                    _ => "absent".to_string(),
                };
                Some(format!("[Firmware] {}: {tail}", f.label))
            })
            .collect()
    }

    /// One line when a file is absent and its replacement is not exact, `NoticeFor`.
    fn notice_for(&self, wanted: &[Firmware]) -> Option<String> {
        for (which, source) in self.mi.firmware.iter().rev() {
            let Some(f) = wanted.iter().find(|f| f.which == *which) else { continue };
            let Some((effect, cost)) = &f.replacement else { continue };
            if effect == "exact" {
                continue;
            }
            if source == "absent" {
                return Some(format!("{} {} would supply the chip.", cost.as_deref().unwrap_or(""), f.name));
            }
        }
        None
    }

    fn battery_file(&self, tree: &Tree, rom: &str, console: &str, suffix: &str) -> BatterySave {
        BatterySave::open(tree, rom, console, self.options.battery_disabled, suffix)
    }

    pub fn release_media(&mut self) {
        self.placement = None;
    }

    fn lends(&self) -> bool {
        self.claims(caps::BORROW)
    }

    /// A machine for a disc and its files.
    fn create(&self, disc: &DiscFiles, firmware: &[(u32, Vec<u8>)], saves: &[(u32, BatterySave)], placement: &Placement, system: Option<&str>) -> Result<Machine, Refused> {
        let mut owned: Vec<(u32, Vec<u8>)> = firmware.to_vec();
        for (which, save) in saves {
            let bytes = save.read().or_else(|| placement.fresh.iter().find(|(w, _)| w == which).and_then(|(_, module)| emusen_systems::media::format(module)));
            if let Some(bytes) = bytes {
                owned.push((*which, bytes));
            }
        }
        let mut files: Vec<(u32, &[u8])> = owned.iter().map(|(w, b)| (*w, b.as_slice())).collect();
        for (n, c) in disc.companions.iter().enumerate() {
            files.push((flags::FILE_MEDIA + n as u32, c.bytes()));
        }
        let settings = self.settings_text(true);
        let system = system.map(|s| CString::new(s).unwrap_or_default());
        let mut create = Create::new(disc.image.bytes());
        create.settings = &settings;
        create.files = &files;
        create.flags = if self.lends() { 1 } else { 0 } | if placement.removable_apart { 2 } else { 0 } | if placement.console_apart { 4 } else { 0 };
        create.system = system.as_deref();
        Machine::create(self.lib, &create).map_err(|r| Refused::fault(format!("{} refused the game: {}.", self.display_name, r.words)))
    }

    /// A game loaded, `CoreEngine.LoadRom`.
    #[allow(clippy::too_many_arguments)]
    pub fn load_rom(&mut self, tree: &Tree, at: &str, system: Option<SystemDetail>, console: String, is_image: &dyn Fn(&str) -> bool, folders: &[String]) -> Result<(), Refused> {
        let set = DiscSet::of(tree, at, is_image).map_err(Refused::io)?;
        let disc = set.open(tree, 0, is_image).map_err(Refused::io)?;
        let asked = self.lib.firmware_for(disc.image.bytes()).ok().and_then(|t| emusen_native::json::parse(t.as_bytes()).ok()).map(|v| crate::firmware::entries(&v)).unwrap_or_default();
        let firmware = self.firmware_files(&asked, folders);
        self.release_media();
        let stem = path::file_name_without_extension(Style::HOST, at).to_string();
        let disabled = self.options.battery_disabled;
        let own = |b: &BatteryFile| BatterySave::open(tree, at, &console, disabled, &b.suffix);
        let placement = media::place(tree, &console, system.as_ref(), &self.media_choices, &stem, &own, disabled);
        let save_for = |b: &BatteryFile| placement.file(b.which).cloned().unwrap_or_else(|| self.battery_file(tree, at, &console, &b.suffix));
        let saves: Vec<(u32, BatterySave)> = match system.as_ref().and_then(|s| s.battery.as_ref()) {
            Some(listed) => listed.iter().map(|b| (b.which, save_for(b))).collect(),
            None => vec![(0, self.battery_file(tree, at, &console, emusen_galaxia::saves::SRAM_EXTENSION))],
        };
        let mut machine = self.create(&disc, &firmware, &saves, &placement, None)?;
        let mut kept = self.lends();
        let mut disc = Some(disc);
        let mi = MachineInfo::read(&machine.machine_info().unwrap_or_default());
        let wanted: Vec<(u32, BatterySave)> = mi
            .battery
            .iter()
            .map(|b| (b.which, save_for(&BatteryFile { which: b.which, suffix: b.suffix.clone(), kind: b.kind, medium: None, label: None, port: None })))
            .collect();
        if wanted.iter().any(|(w, s)| s.read().is_some() && !saves.iter().any(|(sw, ss)| sw == w && ss.path == s.path)) {
            drop(machine);
            let again = if self.lends() { set.open(tree, 0, is_image).map_err(Refused::io)? } else { disc.take().expect("held") };
            machine = self.create(&again, &firmware, &wanted, &placement, None)?;
            kept = self.lends();
            disc = Some(again);
        }
        let loan = if kept { disc } else { None };
        self.made = Made::Game(system);
        self.adopt(tree, machine, placement, &asked, wanted, Some(set), at, console, None, loan);
        Ok(())
    }

    /// The console of `system_id` with nothing in it, `CoreEngine.StartConsole`.
    pub fn start_console(&mut self, tree: &Tree, system_id: &str, console: String, folders: &[String]) -> Result<(), Refused> {
        let systems = self.systems();
        let system = systems.iter().find(|s| s.id == system_id && info_starts_empty(&self.info, system_id)).cloned();
        let Some(system) = system else {
            return Err(Refused { status: emusen_session_service::abi::status::NOT_SUPPORTED, words: format!("{} does not start {system_id} with nothing in it.", self.display_name) });
        };
        let wanted_fw = self.system_entries().into_iter().find(|(id, _, _)| id == system_id).map(|(_, _, fw)| fw).unwrap_or_default();
        let firmware = self.firmware_files(&wanted_fw, folders);
        self.release_media();
        let media = media::media_of(&console, Some(&system));
        let choices: Vec<(String, Choice)> = media
            .iter()
            .map(|m| {
                let c = self.media_choices.iter().find(|(id, _)| *id == m.id).map(|(_, c)| c.clone());
                let choice = match c {
                    Some(c) if c.apart() => c,
                    other => Choice { mode: media::SHARED.to_string(), names: other.map(|c| c.names).unwrap_or_default() },
                };
                (m.id.clone(), choice)
            })
            .collect();
        let holder = emusen_galaxia::saves::console_session_name(&console);
        let placement = media::place(tree, &console, Some(&system), &choices, &holder, &|_| BatterySave::NONE, self.options.battery_disabled);
        let saves: Vec<(u32, BatterySave)> = system.battery.iter().flatten().map(|b| (b.which, placement.file(b.which).cloned().unwrap_or(BatterySave::NONE))).collect();
        let empty = DiscFiles::empty();
        let machine = self.create(&empty, &firmware, &saves, &placement, Some(system_id))?;
        self.adopt(tree, machine, placement, &wanted_fw, saves, None, "", console, Some(system_id.to_string()), None);
        self.made = Made::Console(system_id.to_string());
        Ok(())
    }

    /// The new machine in place of the last, `CoreEngine.Adopt`.
    #[allow(clippy::too_many_arguments)]
    fn adopt(
        &mut self,
        _tree: &Tree,
        machine: Machine,
        placement: Placement,
        asked: &[Firmware],
        wanted: Vec<(u32, BatterySave)>,
        set: Option<DiscSet>,
        at: &str,
        console: String,
        empty: Option<String>,
        loan: Option<DiscFiles>,
    ) {
        self.media_notice = if placement.notices.is_empty() { None } else { Some(placement.notices.join(" ")) };
        let notices = placement.notices.clone();
        self.placement = Some(placement);
        self.discs = set.as_ref().map(|s| s.images()).unwrap_or_default();
        self.disc_index = if set.is_some() { 0 } else { -1 };
        self.set = set;
        self.close_lid_at = -1;
        self.started_empty = empty;
        self.loans.clear();
        self.machine = None;
        self.mi = MachineInfo::read(&machine.machine_info().unwrap_or_default());
        if let Some(loan) = loan {
            self.loans.push((self.mi.drives.first().map_or(0, |d| d.drive), loan));
        }
        self.machine = Some(machine);
        self.firmware_notice = self.notice_for(asked);
        for line in self.firmware_lines(asked) {
            self.say(line);
        }
        for n in notices {
            self.say(format!("[MemoryCards] {n}"));
        }
        for (_, save) in &wanted {
            save.settle(&|line| self.say(line));
        }
        self.battery = wanted;
        self.rom_path = at.to_string();
        self.console = console;
        let axes = self.axes_of();
        self.latch.reset(self.mi.ports.len().max(2), axes);
        self.fresh = true;
        let skip = self.skip_rendering;
        let run = self.settings_text(false);
        let settings = self.claims(caps::SETTINGS);
        let notes = self.claims(caps::SETTING_NOTES);
        let m = self.machine.as_mut().expect("adopted");
        let _ = m.set_options(if skip { flags::OPTION_SKIP_RENDERING } else { 0 });
        self.audio_rate = m.audio_rate();
        if !run.is_empty() && settings {
            let _ = m.set_settings(&run);
        }
        self.notes = if notes { m.setting_notes().ok().map(|t| machine_info::notes(&t)).unwrap_or_default() } else { Vec::new() };
        self.drain();
        self.shape = self.machine.as_ref().and_then(|m| m.frame_info().ok()).unwrap_or_default();
    }

    /// One past the highest axis any controller of the machine's system names.
    fn axes_of(&self) -> usize {
        let systems = info::systems(&self.info);
        let system = systems.iter().find(|s| s.id == self.mi.system).or(systems.first());
        let raw = self.info.get("systems").and_then(Value::as_array).and_then(|list| {
            let id = system.map(|s| s.id.clone()).unwrap_or_default();
            list.iter().find(|s| s.get("id").and_then(Value::as_str) == Some(id.as_str()))
        });
        raw.and_then(|s| s.get("controllers"))
            .and_then(Value::as_array)
            .map(|cs| cs.iter().flat_map(|c| c.get("axes").and_then(Value::as_array).unwrap_or(&[]).iter()).map(|a| int(a, "axis") as usize + 1).max().unwrap_or(0))
            .unwrap_or(0)
    }

    fn keep_log(&mut self, text: &str) {
        let lines: Vec<&str> = text.split('\n').filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            return;
        }
        self.logged_lines += lines.len() as i64;
        for l in lines {
            self.log.push_back(l.to_string());
        }
        while self.log.len() > 512 {
            self.log.pop_front();
        }
    }

    pub fn recent_log(&self) -> &std::collections::VecDeque<String> {
        &self.log
    }

    /// The core's queued log text, as the C# reads it in up to eight rounds.
    fn drain_log(&mut self) {
        let Some(m) = self.machine.as_mut() else { return };
        self.log_text.clear();
        for _ in 0..8 {
            match m.log_drain(&mut self.log_text) {
                Ok(n) if n > 0 => {}
                _ => break,
            }
        }
        let text = std::mem::take(&mut self.log_text);
        self.keep_log(&text);
        self.log_text = text;
    }

    /// Events after a call that can raise them, `CoreEngine.Drain`.
    pub fn drain(&mut self) {
        let Some(m) = self.machine.as_mut() else { return };
        let events: Vec<(u32, i64)> = m.events().map(|e| e.iter().map(|e| (e.kind, e.a)).collect()).unwrap_or_default();
        for (kind, a) in events {
            match kind {
                event::GEOMETRY => self.shape = self.machine.as_ref().and_then(|m| m.frame_info().ok()).unwrap_or(self.shape),
                event::AUDIO_RATE => self.audio_rate = a as i32,
                event::LOG => self.drain_log(),
                event::BATTERY => {
                    if self.claims(caps::BATTERY_DIRTY) {
                        self.save_sram();
                    }
                }
                event::MACHINE_INFO => {
                    if let Some(m) = self.machine.as_ref() {
                        self.mi = MachineInfo::read(&m.machine_info().unwrap_or_default());
                        self.shape = m.frame_info().unwrap_or(self.shape);
                    }
                    self.latch.grow(self.mi.ports.len());
                }
                _ => {}
            }
        }
    }

    /// Each battery file the machine keeps: written when it changed where the core tracks changes, else whenever asked.
    pub fn save_sram(&mut self) {
        let dirty = self.claims(caps::BATTERY_DIRTY);
        let Some(m) = self.machine.as_mut() else { return };
        let mut data = Vec::new();
        for (which, save) in &self.battery {
            if save.path.is_none() {
                continue;
            }
            data.clear();
            let Ok(state) = m.battery(*which, &mut data) else { continue };
            if data.is_empty() {
                continue;
            }
            if dirty && state & flags::BATTERY_TRACKED != 0 && state & flags::BATTERY_CHANGED == 0 {
                continue;
            }
            if save.write(&data) {
                let _ = m.battery_saved(*which);
            }
        }
    }

    pub fn total_frames(&self) -> i64 {
        self.machine.as_ref().map_or(0, |m| m.frame_count())
    }

    /// The console's own reset, refused where the core claims none.
    pub fn reset(&mut self) -> Result<(), Refused> {
        let words = self.display_name.clone();
        let Some(m) = self.machine.as_mut() else { return Err(Refused::fault(format!("{words} has no machine"))) };
        if !self.lib.claims(caps::RESET) {
            return Err(Refused { status: emusen_session_service::abi::status::NOT_SUPPORTED, words: format!("{words} has no reset button: {words} has no reset button.") });
        }
        m.reset().map_err(|s| Refused::fault(format!("{words} could not reset: {}.", m.words(s as i64))))?;
        self.drain();
        Ok(())
    }

    /// Whether the core changes discs: it claims MEDIA and names a drive.
    pub fn can_change_disc(&self) -> bool {
        self.claims(caps::MEDIA) && self.machine.is_some() && !self.mi.drives.is_empty()
    }

    pub fn lid_open(&self) -> bool {
        self.mi.drives.first().is_some_and(|d| d.lid_open)
    }

    fn reread(&mut self) {
        if let Some(m) = self.machine.as_ref() {
            self.mi = MachineInfo::read(&m.machine_info().unwrap_or_default());
        }
    }

    pub fn set_lid(&mut self, open: bool) -> Result<(), Refused> {
        let name = self.display_name.clone();
        let drive = self.mi.drives[0].drive;
        let m = self.machine.as_mut().expect("a machine with a drive");
        m.media_lid(drive, open).map_err(|s| Refused::fault(format!("{name} could not {}: {}.", if open { "open the drive's lid" } else { "close the drive's lid" }, m.words(s as i64))))?;
        self.reread();
        self.close_lid_at = -1;
        Ok(())
    }

    /// Disc `index` of the session's discs into the drive, or -1 to empty it.
    pub fn insert_index(&mut self, tree: &Tree, index: i32, restore: bool, is_image: &dyn Fn(&str) -> bool) -> Result<(), Refused> {
        let name = self.display_name.clone();
        let drive = self.mi.drives[0].drive;
        let restore = if restore { flags::MEDIA_RESTORE } else { 0 };
        if index < 0 {
            let m = self.machine.as_mut().expect("a machine with a drive");
            m.media_insert(drive, None, &[], restore).map_err(|s| Refused::fault(format!("{name} could not empty the drive: {}.", m.words(s as i64))))?;
            self.loans.retain(|(d, _)| *d != drive);
            self.reread();
            self.disc_index = -1;
            return Ok(());
        }
        let disc = match &self.set {
            Some(set) if (index as usize) < set.count() => set.open(tree, index as usize, is_image),
            _ => DiscSet::of(tree, &self.discs[index as usize], is_image).and_then(|s| s.open(tree, 0, is_image)),
        }
        .map_err(Refused::io)?;
        let lent = if self.lends() { 2 } else { 0 };
        let files: Vec<(u32, &[u8])> = disc.companions.iter().enumerate().map(|(n, c)| (flags::FILE_MEDIA + n as u32, c.bytes())).collect();
        let m = self.machine.as_mut().expect("a machine with a drive");
        m.media_insert(drive, Some(disc.image.bytes()), &files, restore | lent).map_err(|s| Refused::fault(format!("{name} could not take the disc: {}.", m.words(s as i64))))?;
        self.loans.retain(|(d, _)| *d != drive);
        if lent != 0 {
            self.loans.push((drive, disc));
        }
        self.reread();
        self.disc_index = index;
        Ok(())
    }

    /// A disc by path, the lid opened and shut around it.
    pub fn insert_path(&mut self, tree: &Tree, at: &str, is_image: &dyn Fn(&str) -> bool) -> Result<(), Refused> {
        let index = match self.discs.iter().position(|d| d == at) {
            Some(i) => i,
            None => {
                self.discs.push(at.to_string());
                self.discs.len() - 1
            }
        };
        match self.change_disc(tree, index as i32, is_image) {
            Ok(()) => Ok(()),
            Err(e) => {
                if index == self.discs.len() - 1 && index as i32 != self.disc_index && self.set.as_ref().is_none_or(|s| index >= s.count()) {
                    self.discs.remove(index);
                }
                Err(e)
            }
        }
    }

    /// Change disc as one action: the lid opened, the disc in, the lid closed a second of the game's time later.
    pub fn change_disc(&mut self, tree: &Tree, index: i32, is_image: &dyn Fn(&str) -> bool) -> Result<(), Refused> {
        if !self.lid_open() {
            self.set_lid(true)?;
        }
        self.insert_index(tree, index, false, is_image)?;
        self.close_lid_at = self.total_frames() + ((1.0 * self.mi.frame_rate_hz()).round_ties_even() as i64).max(1);
        Ok(())
    }

    /// The settings a window lists, `CoreEngine.Settings`.
    pub fn listed(&self) -> Vec<SettingDesc> {
        self.settings.iter().filter(|s| s.listed()).cloned().collect()
    }

    /// A value set: a run-time key reaches the machine between frames, a create-time one makes a machine not yet run
    /// again, `CoreEngine.Set`; the words of the C#'s `ArgumentException` where it is refused.
    pub fn set(&mut self, ctx: &Ctx, key: &str, value: &str) -> Result<(), String> {
        let Some(s) = self.settings.iter().find(|s| s.key == key).cloned() else { return Err(format!("{} has no setting {key}.", self.name)) };
        if !s.accepts(value) {
            return Err(format!("{key}={value} is outside what {} takes.", self.name));
        }
        if !s.create_scope && self.claims(caps::SETTINGS) {
            let notes = self.claims(caps::SETTING_NOTES);
            if let Some(m) = self.machine.as_mut() {
                let _ = m.set_settings(&format!("{key}={value}"));
                self.notes = if notes { m.setting_notes().ok().map(|t| machine_info::notes(&t)).unwrap_or_default() } else { Vec::new() };
                self.drain();
            }
        }
        let was = self.value(key).unwrap_or("").to_string();
        if let Some(slot) = self.values.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value.to_string();
        }
        if s.create_scope && self.machine.is_some() && self.fresh && was != value {
            let again = match self.made.clone() {
                Made::Console(system) => self.start_console(ctx.tree, &system, self.console.clone(), ctx.folders),
                Made::Game(system) => {
                    let (at, console) = (self.rom_path.clone(), self.console.clone());
                    self.load_rom(ctx.tree, &at, system, console, ctx.is_image, ctx.folders)
                }
                Made::Nothing => Ok(()),
            };
            if let Err(r) = again {
                if let Some(slot) = self.values.iter_mut().find(|(k, _)| k == key) {
                    slot.1 = was;
                }
                return Err(format!("{} refused {key} at create: {}", self.name, r.words));
            }
        }
        Ok(())
    }

    /// A value applied as the layers give it, one the engine refuses falling back to its default, `ApplySetting`.
    pub fn apply(&mut self, ctx: &Ctx, key: &str, value: Option<&str>) {
        let Some(s) = self.listed().into_iter().find(|s| s.key == key) else { return };
        let wanted = value.unwrap_or(&s.default).to_string();
        if self.value(key) == Some(wanted.as_str()) {
            return;
        }
        if self.set(ctx, key, &wanted).is_err() {
            let _ = self.set(ctx, key, &s.default);
        }
    }

    /// Lets go of the media and the machine.
    pub fn dispose(&mut self) {
        self.release_media();
        self.machine = None;
        self.loans.clear();
    }
}

fn info_starts_empty(info: &Value, system: &str) -> bool {
    info.get("systems")
        .and_then(Value::as_array)
        .unwrap_or(&[])
        .iter()
        .any(|s| s.get("id").and_then(Value::as_str) == Some(system) && matches!(s.get("starts_empty"), Some(Value::Bool(true))))
}
