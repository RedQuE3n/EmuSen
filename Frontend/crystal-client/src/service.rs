//! The session service, emusen.session@1, wrapped once: the table `emusen_crystal_session_service` hands out, its
//! major checked, each entry checked against the table's size before it is read, and every failure's words read with
//! the table's own last_error. The entries a front end calls are here; the debugger's are owed (EmuSen_Qt.md §1.1.1).

use crate::error::Result;
use crate::events::Event;
use crate::out::{check, copy_out, json, string, text, to_json};
use crate::raw;
use crate::runtime::Runtime;
use crate::shapes::{ServiceInfo, SessionCodecs, SessionId, SessionRow, SystemRow};
use serde_json::Value;
use std::ptr::NonNull;

/// The table's entry `$f` when the provider's table reaches it and has it, as EMUSEN_SESSION_HAS asks.
macro_rules! entry {
    ($self:ident, $f:ident) => {{
        let t = $self.table.as_ptr();
        let end = std::mem::offset_of!(raw::emusen_session_v1, $f) + std::mem::size_of::<usize>();
        let size = unsafe { std::ptr::addr_of!((*t).size).read() } as usize;
        match if size >= end { unsafe { std::ptr::addr_of!((*t).$f).read() } } else { None } {
            Some(f) => f,
            None => {
                let words = format!("The session service's table has no {}.", stringify!($f));
                return Err(crate::error::status(raw::EMUSEN_SVC_NOT_SUPPORTED, words));
            }
        }
    }};
}

/// A session's table, or the runtime's with no game; it lives as long as the runtime, which it keeps.
pub struct SessionService {
    rt: Runtime,
    table: NonNull<raw::emusen_session_v1>,
    ctx: *mut std::ffi::c_void,
}

// The table's calls are answered on the calling thread, a change waiting for the frame boundary.
unsafe impl Send for SessionService {}
unsafe impl Sync for SessionService {}

/// session_service's flags.
pub mod flags {
    pub use crate::raw::{EMUSEN_CRYSTAL_SERVICE_NO_GAME as NO_GAME, EMUSEN_CRYSTAL_SERVICE_READ_ONLY as READ_ONLY};
}

impl Runtime {
    /// The session's emusen.session@1 table; with flags::NO_GAME and session 0, the runtime's with no game.
    pub fn session_service(&self, session: SessionId, flags: u32) -> Result<SessionService> {
        let table = unsafe { self.lib().emusen_crystal_session_service(self.rt(), session, flags) };
        let table = NonNull::new(table as *mut raw::emusen_session_v1)
            .ok_or_else(|| crate::error::status(raw::EMUSEN_SVC_NO_SESSION, self.words()))?;
        let (version, ctx) = unsafe { ((*table.as_ptr()).version, (*table.as_ptr()).ctx) };
        if version >> 16 != raw::EMUSEN_SESSION_MAJOR {
            let words = format!(
                "The session service is version {}.{}; this program needs {}.x.",
                version >> 16,
                version & 0xffff,
                raw::EMUSEN_SESSION_MAJOR
            );
            return Err(crate::error::status(raw::EMUSEN_SVC_NOT_SUPPORTED, words));
        }
        Ok(SessionService { rt: self.clone(), table, ctx })
    }
}

impl SessionService {
    /// The words for the last failing call of the table on this thread.
    fn words(&self) -> String {
        let t = self.table.as_ptr();
        match unsafe { std::ptr::addr_of!((*t).last_error).read() } {
            Some(f) => copy_out(|o, l| unsafe { f(self.ctx, o, l) }, String::new)
                .ok()
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default(),
            None => String::new(),
        }
    }

    fn status(&self, n: i64) -> Result<i64> {
        check(n, || self.words())
    }

    fn read(&self, call: impl FnMut(*mut u8, usize) -> i64) -> Result<Vec<u8>> {
        copy_out(call, || self.words())
    }

    /// The runtime the table belongs to.
    pub fn runtime(&self) -> &Runtime {
        &self.rt
    }

    /// The table's own sessions: its session, or none for the table with no game.
    pub fn sessions(&self) -> Result<Vec<SessionRow>> {
        let f = entry!(self, sessions);
        json("sessions", &self.read(|o, l| unsafe { f(self.ctx, o, l) })?)
    }

    /// The service's reading of a session.
    pub fn info(&self, session: SessionId) -> Result<ServiceInfo> {
        let f = entry!(self, info);
        json("info", &self.read(|o, l| unsafe { f(self.ctx, session, o, l) })?)
    }

    /// The core's own words for its session.
    pub fn summary(&self, session: SessionId) -> Result<String> {
        let f = entry!(self, summary);
        string("summary", self.read(|o, l| unsafe { f(self.ctx, session, o, l) })?)
    }

    /// Run control, EMUSEN_SESSION_RUN_*; 0, or RUN_TO_ADDRESS's breakpoint id.
    pub fn run_control(&self, session: SessionId, op: u32, processor: u32, argument: i64) -> Result<i64> {
        let f = entry!(self, run_control);
        self.status(unsafe { f(self.ctx, session, op, processor, argument) })
    }

    /// A cheat op, EMUSEN_SESSION_CHEAT_*, its arguments and answer as JSON; a change that answers a number gives it.
    pub fn cheat_op(&self, session: SessionId, op: u32, args: &Value) -> Result<Value> {
        let f = entry!(self, cheat_op);
        let args = to_json(args);
        let bytes = self.read(|o, l| unsafe { f(self.ctx, session, op, args.as_ptr(), args.len(), o, l) })?;
        if bytes.is_empty() { Ok(Value::Null) } else { json("cheat_op", &bytes) }
    }

    /// The session's automatic and explicit codec.
    pub fn cheat_codecs(&self, session: SessionId) -> Result<SessionCodecs> {
        let f = entry!(self, cheat_codecs);
        json("cheat_codecs", &self.read(|o, l| unsafe { f(self.ctx, session, o, l) })?)
    }

    /// A telemetry reading, EMUSEN_SESSION_TELEMETRY_*, as JSON.
    pub fn telemetry(&self, session: SessionId, kind: u32) -> Result<Value> {
        let f = entry!(self, telemetry);
        json("telemetry", &self.read(|o, l| unsafe { f(self.ctx, session, kind, o, l) })?)
    }

    /// The sound channels and whether each is muted, as JSON.
    pub fn audio_channels(&self, session: SessionId) -> Result<Value> {
        let f = entry!(self, audio_channels);
        json("audio_channels", &self.read(|o, l| unsafe { f(self.ctx, session, o, l) })?)
    }

    /// A sound channel muted or heard.
    pub fn set_channel_muted(&self, session: SessionId, channel: u32, muted: bool) -> Result<()> {
        let f = entry!(self, set_channel_muted);
        self.status(unsafe { f(self.ctx, session, channel, muted as u32) }.into()).map(drop)
    }

    /// A state saved at `path`, or at the default path with none.
    pub fn state_save(&self, session: SessionId, path: Option<&str>) -> Result<()> {
        let f = entry!(self, state_save);
        let (p, pl) = text(path);
        self.status(unsafe { f(self.ctx, session, p, pl) }.into()).map(drop)
    }

    /// The state at `path` loaded.
    pub fn state_load(&self, session: SessionId, path: &str) -> Result<()> {
        let f = entry!(self, state_load);
        self.status(unsafe { f(self.ctx, session, path.as_ptr(), path.len()) }.into()).map(drop)
    }

    /// Where a state goes by default.
    pub fn state_default_path(&self, session: SessionId) -> Result<String> {
        let f = entry!(self, state_default_path);
        string("state_default_path", self.read(|o, l| unsafe { f(self.ctx, session, o, l) })?)
    }

    /// The process's log switch.
    pub fn log_switch(&self, on: bool) -> Result<()> {
        let f = entry!(self, log_switch);
        self.status(unsafe { f(self.ctx, on as u32) }.into()).map(drop)
    }

    /// Whether the process's log is on.
    pub fn log_status(&self) -> Result<bool> {
        let f = entry!(self, log_status);
        self.status(unsafe { f(self.ctx) }.into()).map(|n| n == 1)
    }

    /// The table's events of `kinds`, EMUSEN_SESSION_EVENT_*, oldest first.
    pub fn events_drain(&self, session: SessionId, kinds: u32) -> Result<Vec<Event>> {
        let f = entry!(self, events_drain);
        let bytes = self.read(|o, l| unsafe { f(self.ctx, session, kinds, o, l) })?;
        if bytes.is_empty() { Ok(Vec::new()) } else { json("events_drain", &bytes) }
    }

    /// What the runtime can open: `systems`' answer.
    pub fn games(&self) -> Result<Vec<SystemRow>> {
        let f = entry!(self, games);
        json("games", &self.read(|o, l| unsafe { f(self.ctx, o, l) })?)
    }

    /// The game at `path` opened as a new session with the named session's options; its id.
    pub fn open_game(&self, session: SessionId, path: &str, engine: Option<&str>) -> Result<SessionId> {
        let f = entry!(self, open_game);
        let (e, el) = text(engine);
        self.status(unsafe { f(self.ctx, session, path.as_ptr(), path.len(), e, el) }).map(|n| n as SessionId)
    }

    /// A console started empty as a new session; its id.
    pub fn open_console(&self, session: SessionId, console: &str, engine: Option<&str>) -> Result<SessionId> {
        let f = entry!(self, open_console);
        let (e, el) = text(engine);
        self.status(unsafe { f(self.ctx, session, console.as_ptr(), console.len(), e, el) }).map(|n| n as SessionId)
    }
}
