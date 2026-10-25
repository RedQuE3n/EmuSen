//! A session's save states, the C# `ManagedSession`'s states: the slots and the resume state in the player's state
//! folder, a state written whole with its record and its picture, a state loaded unless its record says it is another
//! machine's, and the history cleared after the jump (EmuSen_Backend.md §14.12, §29.5).

use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::Tree;
use emusen_session_service::abi::status as svc;

use crate::abi;
use crate::engine::{Ctx, Engine};
use crate::json::Json;
use crate::session::{Refused, Run, Session};
use crate::states::{StateRecord, StateRecords};

pub const SLOTS: u32 = 8;
pub const DEFAULT_SLOT: i32 = 1;

/// The build a record names: the runtime's version.
pub fn build_name() -> String {
    format!("{} (Rust)", env!("CARGO_PKG_VERSION"))
}

/// The player's state folder, read at each use, `AppSettings.StateDirectory`.
fn state_folder(tree: &Tree) -> Option<String> {
    let model = &emusen_galaxia::models::MODELS[emusen_galaxia::models::APP_SETTINGS_ID as usize];
    let (settings, _) = emusen_galaxia::config::load_or_new(tree, model, None, emusen_galaxia::models::APP_SETTINGS_FILE);
    settings.get("StateDirectory").and_then(|v| v.as_str()).map(str::to_string)
}

fn check_slot(slot: u32) -> Result<i32, Refused> {
    if (1..=SLOTS).contains(&slot) { Ok(slot as i32) } else { Err((svc::BAD_ARGUMENT, format!("no state slot {slot}; the slots are 1 to {SLOTS}"))) }
}

impl Session {
    pub fn state_path(&self, tree: &Tree, slot: i32) -> String {
        match &self.package {
            Some(root) => emusen_galaxia::saves::package_state_path(root, slot, self.save_key.as_deref()),
            None => emusen_galaxia::saves::state_path(tree, &self.game_key(), slot, state_folder(tree).as_deref()),
        }
    }

    pub fn resume_path(&self, tree: &Tree) -> String {
        match &self.package {
            Some(root) => emusen_galaxia::saves::package_resume_state_path(root, self.save_key.as_deref()),
            None => emusen_galaxia::saves::resume_state_path(tree, &self.game_key(), state_folder(tree).as_deref()),
        }
    }

    /// `[{"slot","path","exists","saved_at"}]`, the resume state as slot 0.
    pub fn slots_json(&self, tree: &Tree) -> String {
        let mut j = Json::new();
        j.begin_array();
        for slot in 0..=SLOTS as i32 {
            let at = if slot == 0 { self.resume_path(tree) } else { self.state_path(tree, slot) };
            let exists = emusen_galaxia::dotnet_files::file_exists(&at);
            j.begin_object().int("slot", slot as i64).str("path", &at).bool("exists", exists);
            match emusen_galaxia::clock::last_write(&at).filter(|_| exists) {
                Some(t) => j.str("saved_at", &t),
                None => j.null("saved_at"),
            };
            j.end_object();
        }
        j.end_array();
        j.finish()
    }

    fn record(&self, e: &Engine) -> StateRecord {
        let (md5, bytes) = self.identity.lock().unwrap_or_else(|p| p.into_inner()).clone();
        let game = self.game_path.is_some();
        StateRecord {
            console: self.console.clone(),
            core: e.display_name.clone(),
            state_version: e.state_version(),
            build: build_name(),
            saved_at: emusen_galaxia::clock::now(),
            rom_file: path::file_name(Style::HOST, &self.game_key()).to_string(),
            rom_md5: if game { md5 } else { None },
            rom_bytes: if game { bytes } else { 0 },
        }
    }

    /// Why the record says the state is not this machine's to load, or none.
    fn refusal(&self, e: &Engine, r: &StateRecord) -> Option<String> {
        if !emusen_dotnet::text::equals_ignore_case(&r.console, &self.console) {
            return Some(format!("That state was saved by the {} core, and {} runs on the {} core.", r.console, path::file_name(Style::HOST, &self.game_key()), self.console));
        }
        let loads = &e.mi.loads_from;
        if r.core == e.display_name && e.machine.is_some() && !loads.is_empty() && !loads.contains(&(r.state_version as i64)) {
            let (min, max) = (loads.iter().min().copied().unwrap_or(0), loads.iter().max().copied().unwrap_or(0));
            return Some(format!("That state is {}'s version {}; this machine reads versions {min} to {max}.", r.core, r.state_version));
        }
        None
    }

    /// The state written whole and its record kept, `ManagedSession.SaveState`; with a picture beside it when asked.
    pub(crate) fn save_in(&self, e: &mut Engine, at: &str, records: Option<&StateRecords>, picture: bool) -> Result<(), Refused> {
        let full = path::full_path(at);
        if let Some(folder) = path::directory_name(Style::HOST, &full).filter(|f| !f.is_empty()) {
            std::fs::create_dir_all(&folder).map_err(|err| (abi::status::IO, emusen_dotnet::file::io_message(&err, &folder)))?;
        }
        let bytes = e.state_bytes().map_err(|r| (r.status, r.words))?;
        std::fs::write(at, &bytes).map_err(|err| (abi::status::IO, emusen_dotnet::file::io_message(&err, at)))?;
        crate::states::write(records, at, &self.record(e))?;
        self.events.raise(self.id, abi::EVENT_STATE, "state", |j| {
            j.str("op", "saved").str("path", at);
        });
        if picture {
            let (rgba, width, height) = self.shown_picture(e);
            if width > 0 && height > 0 {
                let png = emusen_galaxia::saves::picture_path(at);
                crate::png::write(&png, &rgba, width as u32, height as u32).map_err(|err| (abi::status::IO, emusen_dotnet::file::io_message(&err, &png)))?;
            }
        }
        Ok(())
    }

    /// A state loaded unless its record refuses it, then the jump's sequence, `ManagedSession.LoadState`.
    pub(crate) fn load_in(&self, e: &mut Engine, run: &mut Run, at: &str, records: Option<&StateRecords>, ctx: &Ctx) -> Result<(), Refused> {
        if !emusen_galaxia::dotnet_files::file_exists(at) {
            return Err((svc::ABSENT, format!("no state at {at}")));
        }
        if let Some(r) = crate::states::read(records, at)?
            && let Some(why) = self.refusal(e, &r)
        {
            return Err((abi::status::STATE_REFUSED, why));
        }
        let bytes = std::fs::read(at).map_err(|err| (abi::status::IO, emusen_dotnet::file::io_message(&err, at)))?;
        e.load_state(&bytes, Some(ctx)).map_err(|r| (r.status, r.words))?;
        run.rewind.clear();
        run.halted = false;
        e.refresh_debugger();
        self.events.raise(self.id, abi::EVENT_STATE, "state", |j| {
            j.str("op", "loaded").str("path", at);
        });
        Ok(())
    }

    pub fn save_state(&self, at: &str, records: Option<&StateRecords>, picture: bool) -> Result<(), Refused> {
        self.change(|e, _| self.save_in(e, at, records, picture))
    }

    pub fn load_state(&self, at: &str, records: Option<&StateRecords>, ctx: &Ctx) -> Result<(), Refused> {
        self.change(|e, run| self.load_in(e, run, at, records, ctx))
    }

    pub fn save_slot(&self, slot: u32, records: Option<&StateRecords>, ctx: &Ctx) -> Result<(), Refused> {
        let at = self.state_path(ctx.tree, check_slot(slot)?);
        self.save_state(&at, records, true)
    }

    pub fn load_slot(&self, slot: u32, records: Option<&StateRecords>, ctx: &Ctx) -> Result<(), Refused> {
        let at = self.state_path(ctx.tree, check_slot(slot)?);
        self.load_state(&at, records, ctx)
    }

    pub fn save_resume(&self, records: Option<&StateRecords>, ctx: &Ctx) -> Result<(), Refused> {
        self.save_state(&self.resume_path(ctx.tree), records, true)
    }

    pub fn load_resume(&self, records: Option<&StateRecords>, ctx: &Ctx) -> Result<(), Refused> {
        self.load_state(&self.resume_path(ctx.tree), records, ctx)
    }
}

/// `{"console","core","state_version","build","saved_at","rom_file","rom_md5","rom_bytes"}`.
pub fn record_json(r: &StateRecord) -> String {
    let mut j = Json::new();
    j.begin_object()
        .str("console", &r.console)
        .str("core", &r.core)
        .int("state_version", r.state_version as i64)
        .str("build", &r.build)
        .str("saved_at", &r.saved_at)
        .str("rom_file", &r.rom_file)
        .str_or_null("rom_md5", r.rom_md5.as_deref())
        .int("rom_bytes", r.rom_bytes)
        .end_object();
    j.finish()
}
