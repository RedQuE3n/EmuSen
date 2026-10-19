//! One open session: its engine, what it was opened with, and its answers about itself, the C# `ManagedSession`'s
//! life and `Info` (EmuSen_Backend.md §29.2). The frame, input, speed, rewind and states join it in their pieces.

use std::sync::Mutex;

use emusen_galaxia::tree::Tree;
use emusen_native::core::sys::caps;
use emusen_native::json::Value;

use crate::catalog::Catalog;
use crate::engine::Engine;
use crate::json::Json;

/// What a session was opened with besides its game.
#[derive(Clone, Copy, Debug, Default)]
pub struct OpenedWith {
    pub flags: u32,
    pub audio_buffer_max_samples: u32,
}

/// The state a session's frame thread and run control share.
#[derive(Debug, Default)]
pub struct Run {
    pub paused: bool,
    pub halted: bool,
}

pub struct Session {
    pub id: u32,
    pub engine: Mutex<Engine>,
    pub game_path: Option<String>,
    pub console: String,
    /// The console a session started empty was asked for.
    pub empty_console: Option<String>,
    pub opened: OpenedWith,
    pub run: Mutex<Run>,
}

fn controllers_of_system(info: &Value, system: &str) -> Vec<Value> {
    let systems = info.get("systems").and_then(Value::as_array).unwrap_or(&[]);
    let chosen = systems.iter().find(|s| s.get("id").and_then(Value::as_str) == Some(system)).or(systems.first());
    chosen.and_then(|s| s.get("controllers")).and_then(Value::as_array).map(<[Value]>::to_vec).unwrap_or_default()
}

impl Session {
    pub fn repeat_rows(&self) -> bool {
        self.opened.flags & crate::abi::OPEN_SINGLE_ROWS == 0
    }

    /// The name states, settings and saves know the game by: its file, or the console started empty.
    pub fn game_key(&self) -> String {
        self.game_path.clone().unwrap_or_else(|| emusen_galaxia::saves::console_session_name(self.empty_console.as_deref().unwrap_or(&self.console)))
    }

    /// `{"id","engine","system","console","game",…}`, `ManagedSession.Info`.
    pub fn info_json(&self) -> String {
        let e = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        let run = self.run.lock().unwrap_or_else(|p| p.into_inner());
        let loaded = e.machine.is_some();
        let mut j = Json::new();
        j.begin_object().int("id", self.id as i64).str("engine", &e.display_name).str_or_null("system", loaded.then_some(e.mi.system.as_str())).str("console", &self.console);
        match &self.game_path {
            Some(p) => {
                j.key_object("game").str("path", p).str("name", emusen_galaxia::dotnet_path::file_name_without_extension(emusen_galaxia::dotnet_path::Style::HOST, p)).end_object();
            }
            None => {
                j.null("game");
            }
        }
        j.str("run", if run.halted { "halted" } else if run.paused { "paused" } else { "running" });
        j.null("halted");
        let controllers = controllers_of_system(&e.info, &e.mi.system);
        let named = e.mi.ports.iter().filter(|p| p.controller.is_some()).map(|p| p.port as usize + 1).max().unwrap_or(0);
        let ports = if loaded && named > 0 {
            named
        } else {
            let listed = e
                .info
                .get("systems")
                .and_then(Value::as_array)
                .unwrap_or(&[])
                .iter()
                .flat_map(|s| s.get("controllers").and_then(Value::as_array).unwrap_or(&[]).iter())
                .flat_map(|c| c.get("ports").and_then(Value::as_array).unwrap_or(&[]).iter())
                .filter_map(Value::as_i64)
                .map(|p| p as usize + 1)
                .max()
                .unwrap_or(0);
            listed.max(1)
        };
        j.key_array("ports");
        for p in 0..ports {
            let id = if loaded { e.mi.ports.iter().find(|x| x.port as usize == p).and_then(|x| x.controller.clone()) } else { None };
            j.begin_object().int("port", p as i64).str_or_null("controller", id.as_deref()).end_object();
        }
        j.end_array();
        let pad = {
            let id = e.mi.ports.iter().find(|p| p.port == 0).and_then(|p| p.controller.clone());
            let by_id = controllers.iter().find(|c| id.is_some() && c.get("id").and_then(Value::as_str) == id.as_deref()).cloned();
            if !loaded { controllers.first().cloned() } else { by_id.or_else(|| if e.mi.ports.is_empty() { controllers.first().cloned() } else { None }) }
        };
        let controls = |list: &str, names: &[&str]| -> Vec<i64> {
            let mut out: Vec<i64> = Vec::new();
            for b in pad.as_ref().and_then(|c| c.get(list)).and_then(Value::as_array).unwrap_or(&[]) {
                let text = b.get("control").and_then(Value::as_str);
                let found = text.and_then(|t| names.iter().position(|n| *n == t).or_else(|| t.trim().parse::<usize>().ok().filter(|n| *n < names.len())));
                if let Some(i) = found
                    && !out.contains(&(i as i64))
                {
                    out.push(i as i64);
                }
            }
            out
        };
        j.key_array("buttons");
        for b in controls("buttons", &crate::info::BUTTONS) {
            j.int_value(b);
        }
        j.end_array().key_array("axes");
        if e.claims(caps::AXES) {
            for a in controls("axes", &crate::info::AXES) {
                j.int_value(a);
            }
        }
        j.end_array();
        let repeat = self.repeat_rows();
        let rows = e.shape.row_repeat.max(1);
        j.key_object("discs")
            .int("count", e.discs.len() as i64)
            .int("index", e.disc_index as i64)
            .bool("can_change", e.can_change_disc())
            .bool("lid_open", e.can_change_disc() && e.lid_open())
            .end_object()
            .key_object("picture")
            .int("width", e.shape.width as i64)
            .int("height", (e.shape.height * if repeat { rows } else { 1 }) as i64)
            .int("row_repeat", if repeat { 1 } else { rows } as i64)
            .end_object()
            .f64("frame_rate_hz", if loaded { e.mi.frame_rate_hz() } else { 60.0 })
            .int("total_frames", e.total_frames())
            .int("state_version", e.mi.state_version as i32 as i64)
            .key_object("state")
            .str_or_null("format", loaded.then_some(e.mi.state_format.as_str()))
            .int("version", e.mi.state_version as i32 as i64)
            .key_array("loads_from");
        if loaded {
            for v in &e.mi.loads_from {
                j.int_value(*v);
            }
        }
        j.end_array()
            .end_object()
            .bool("rewind_capture", true)
            .null("rewind_withheld")
            .key_object("notices")
            .null("engine")
            .str_or_null("firmware", e.firmware_notice.as_deref())
            .str_or_null("media", e.media_notice.as_deref())
            .end_object()
            .str_or_null("television_standard", e.mi.region.as_deref())
            .end_object();
        j.finish()
    }

    /// `{"notice","media":[…]}`, `ManagedRuntime.SaveMediaJson`.
    pub fn save_media_json(&self, catalog: &Catalog, tree: &Tree) -> String {
        let e = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        let stored = emusen_galaxia::consoles::load(tree);
        let value = |key: &str| stored.value(&self.console, key).map(str::to_string);
        let mut j = Json::new();
        j.begin_object().str_or_null("notice", e.media_notice.as_deref()).key_array("media");
        for m in crate::media::media_for(catalog, &self.console) {
            let choice = m.read(&value);
            j.begin_object()
                .str("medium", &m.id)
                .str("title", &m.title)
                .str("noun", &m.noun)
                .str("extension", &m.extension)
                .str("kind", if m.is_console() { "console" } else { "removable" })
                .str("mode", &choice.mode)
                .key_array("slots");
            for slot in 0..m.slots.len() {
                j.begin_object().str("label", &m.slot_label(slot));
                if choice.apart() {
                    let name = if choice.mode == crate::media::NAMED { m.name_for(&choice, slot) } else { m.default_name(slot) };
                    j.str("name", &name);
                    j.str("path", &m.path_of(tree, &name));
                } else {
                    j.null("name").null("path");
                }
                j.end_object();
            }
            j.end_array().end_object();
        }
        j.end_array().end_object();
        j.finish()
    }
}
