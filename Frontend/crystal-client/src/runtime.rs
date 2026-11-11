//! A runtime: the reads and changes of emusen_crystal.h that are not per frame, each answered as a typed record. The
//! per-frame calls are frame.rs's; the session service's table is service.rs's; subscriptions are events.rs's.

use crate::api::Api;
use crate::error::{Error, Result};
use crate::out::{check, copy_out, copy_out_absent, json, string, text, to_json};
use crate::raw;
use crate::shapes::*;
use serde_json::Value;
use std::ptr::NonNull;
use std::sync::Arc;

/// A library the host has loaded and lends for the runtime's life (`emusen_crystal_config.sqlite`): its loader's handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LentLibrary(NonNull<std::ffi::c_void>);

// A loader's handle, never closed, which the runtime reads from any thread.
unsafe impl Send for LentLibrary {}
unsafe impl Sync for LentLibrary {}

impl LentLibrary {
    /// # Safety
    /// `handle` is a loader's handle (`dlopen`'s) that stays loaded for the life of every runtime made with it.
    pub unsafe fn from_handle(handle: *mut std::ffi::c_void) -> Option<LentLibrary> {
        NonNull::new(handle).map(LentLibrary)
    }
}

/// How a runtime is made; `None` for a directory list is the implementation's default.
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Cores still in development are listed.
    pub development: bool,
    /// The cores directories, scanned in order.
    pub cores: Option<Vec<String>>,
    /// The player's own cores directories, their libraries loaded once approved.
    pub player_cores: Option<Vec<String>>,
}

/// What `open` opens: a game by path, or a console started empty, with an engine by name or the settings' choice.
#[derive(Clone, Debug, Default)]
pub struct Open {
    pub flags: u32,
    pub path: Option<String>,
    pub console: Option<String>,
    pub engine: Option<String>,
    pub audio_buffer_max_samples: u32,
    /// The presenter's Serenity device (the platform library's `emusen_serenity_device`), lent to each machine.
    pub device: Option<NonNull<std::ffi::c_void>>,
}

/// A settings scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Console,
    Game,
}

impl Scope {
    fn raw(self) -> u32 {
        match self {
            Scope::Console => raw::EMUSEN_CRYSTAL_SCOPE_CONSOLE,
            Scope::Game => raw::EMUSEN_CRYSTAL_SCOPE_GAME,
        }
    }
}

pub(crate) struct Inner {
    pub(crate) api: Arc<Api>,
    pub(crate) rt: NonNull<raw::emusen_crystal_runtime>,
}

// The header: a read is answered on any thread, a change from another thread waits for the frame in progress.
unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe { self.api.lib.emusen_crystal_free(self.rt.as_ptr()) };
    }
}

/// A runtime, shared by its clones and freed with the last of them; every session still open is closed then.
#[derive(Clone)]
pub struct Runtime(pub(crate) Arc<Inner>);

impl Runtime {
    /// A runtime made by the library `api` loaded, lent no SQLite, so it keeps no state records.
    pub fn new(api: &Arc<Api>, config: &Config) -> Result<Runtime> {
        Runtime::lent(api, config, None)
    }

    /// A runtime lent the host's SQLite (`emusen_crystal_config.sqlite`), so it keeps its state records (states.db).
    pub fn lent(api: &Arc<Api>, config: &Config, sqlite: Option<LentLibrary>) -> Result<Runtime> {
        let cores = config.cores.as_ref().map(to_json);
        let player = config.player_cores.as_ref().map(to_json);
        let (c, cl) = text(cores.as_deref());
        let (p, pl) = text(player.as_deref());
        let raw_config = raw::emusen_crystal_config {
            size: std::mem::size_of::<raw::emusen_crystal_config>() as u32,
            flags: if config.development { raw::EMUSEN_CRYSTAL_CONFIG_DEVELOPMENT } else { 0 },
            sqlite: sqlite.map_or(std::ptr::null_mut(), |l| l.0.as_ptr()),
            cores: c,
            cores_len: cl,
            player_cores: p,
            player_cores_len: pl,
        };
        let rt = unsafe { api.lib.emusen_crystal_new(&raw_config) };
        match NonNull::new(rt) {
            Some(rt) => Ok(Runtime(Arc::new(Inner { api: api.clone(), rt }))),
            None => Err(crate::error::status(raw::EMUSEN_SVC_FAULT, api.last_error(std::ptr::null_mut()))),
        }
    }

    pub(crate) fn lib(&self) -> &raw::CrystalLibrary {
        &self.0.api.lib
    }

    pub(crate) fn rt(&self) -> *mut raw::emusen_crystal_runtime {
        self.0.rt.as_ptr()
    }

    pub(crate) fn words(&self) -> String {
        self.0.api.last_error(self.rt())
    }

    pub(crate) fn status(&self, n: i64) -> Result<i64> {
        check(n, || self.words())
    }

    pub(crate) fn read(&self, call: impl FnMut(*mut u8, usize) -> i64) -> Result<Vec<u8>> {
        copy_out(call, || self.words())
    }

    pub(crate) fn read_absent(&self, call: impl FnMut(*mut u8, usize) -> i64) -> Result<Option<Vec<u8>>> {
        copy_out_absent(call, || self.words())
    }

    fn record<T: serde::de::DeserializeOwned>(&self, name: &'static str, call: impl FnMut(*mut u8, usize) -> i64) -> Result<T> {
        json(name, &self.read(call)?)
    }

    fn record_absent<T: serde::de::DeserializeOwned>(
        &self,
        name: &'static str,
        call: impl FnMut(*mut u8, usize) -> i64,
    ) -> Result<Option<T>> {
        self.read_absent(call)?.map(|b| json(name, &b)).transpose()
    }

    // ---- The systems and the engines -------------------------------------------------------------------------

    /// The systems table's rows, then the systems only an engine names.
    pub fn systems(&self) -> Result<Vec<SystemRow>> {
        self.record("systems", |o, l| unsafe { self.lib().emusen_crystal_systems(self.rt(), o, l) })
    }

    /// Every session the runtime has open.
    pub fn sessions(&self) -> Result<Vec<SessionRow>> {
        self.record("sessions", |o, l| unsafe { self.lib().emusen_crystal_sessions(self.rt(), o, l) })
    }

    /// The library's shelves in release order.
    pub fn shelves(&self) -> Result<Vec<Shelf>> {
        self.record("shelves", |o, l| unsafe { self.lib().emusen_crystal_shelves(self.rt(), o, l) })
    }

    /// Every engine discovery found.
    pub fn engines(&self) -> Result<Vec<Engine>> {
        self.record("engines", |o, l| unsafe { self.lib().emusen_crystal_engines(self.rt(), o, l) })
    }

    /// Each core library found, unloaded.
    pub fn cores(&self) -> Result<Vec<CoreRow>> {
        self.record("cores", |o, l| unsafe { self.lib().emusen_crystal_cores(self.rt(), o, l) })
    }

    /// Lists the cores directories again and reads the player's approvals again.
    pub fn rescan(&self) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_rescan(self.rt()) }.into()).map(drop)
    }

    /// The player's approval of the core library with this SHA-256.
    pub fn core_approve(&self, sha256: &str) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_core_approve(self.rt(), sha256.as_ptr(), sha256.len()) }.into()).map(drop)
    }

    /// Every approval of this SHA-256 removed.
    pub fn core_revoke(&self, sha256: &str) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_core_revoke(self.rt(), sha256.as_ptr(), sha256.len()) }.into()).map(drop)
    }

    /// What opens the file at `path`, and with which engine.
    pub fn route(&self, path: &str) -> Result<Route> {
        self.record("route", |o, l| unsafe { self.lib().emusen_crystal_route(self.rt(), path.as_ptr(), path.len(), o, l) })
    }

    // ---- A session's life ----------------------------------------------------------------------------------

    /// Opens a game, or starts a console empty; the session's id.
    pub fn open(&self, open: &Open) -> Result<SessionId> {
        let (p, pl) = text(open.path.as_deref());
        let (c, cl) = text(open.console.as_deref());
        let (e, el) = text(open.engine.as_deref());
        let params = raw::emusen_crystal_open_params {
            size: std::mem::size_of::<raw::emusen_crystal_open_params>() as u32,
            flags: open.flags,
            path: p,
            path_len: pl,
            console: c,
            console_len: cl,
            engine: e,
            engine_len: el,
            audio_buffer_max_samples: open.audio_buffer_max_samples,
            reserved: 0,
            device: open.device.map_or(std::ptr::null(), |d| d.as_ptr().cast_const()),
        };
        self.status(unsafe { self.lib().emusen_crystal_open(self.rt(), &params) }).map(|id| id as SessionId)
    }

    /// Writes the battery save, lets go of the save media and ends the session.
    pub fn close(&self, session: SessionId) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_close(self.rt(), session) }.into()).map(drop)
    }

    /// The console's reset.
    pub fn reset(&self, session: SessionId) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_reset(self.rt(), session) }.into()).map(drop)
    }

    /// The session as a front end shows it.
    pub fn session_info(&self, session: SessionId) -> Result<SessionInfo> {
        self.record("session_info", |o, l| unsafe { self.lib().emusen_crystal_session_info(self.rt(), session, o, l) })
    }

    // ---- Discs -------------------------------------------------------------------------------------------

    /// A disc by path into the drive.
    pub fn disc_insert(&self, session: SessionId, path: &str) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_disc_insert(self.rt(), session, path.as_ptr(), path.len()) }.into()).map(drop)
    }

    /// The drive left empty.
    pub fn disc_eject(&self, session: SessionId) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_disc_eject(self.rt(), session) }.into()).map(drop)
    }

    /// The game's disc of that index into the drive.
    pub fn disc_change(&self, session: SessionId, index: u32) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_disc_change(self.rt(), session, index) }.into()).map(drop)
    }

    /// The lid opened or shut.
    pub fn disc_lid(&self, session: SessionId, open: bool) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_disc_lid(self.rt(), session, open as u32) }.into()).map(drop)
    }

    // ---- Rewind and states -------------------------------------------------------------------------------

    /// Rewind on or off, with its interval, budget and thumbnail width; 0 keeps each as it was.
    pub fn rewind_set(
        &self,
        session: SessionId,
        enabled: bool,
        interval_frames: u32,
        budget_bytes: u64,
        thumbnail_width: u32,
    ) -> Result<()> {
        let n = unsafe {
            self.lib().emusen_crystal_rewind_set(self.rt(), session, enabled as u32, interval_frames, budget_bytes, thumbnail_width)
        };
        self.status(n.into()).map(drop)
    }

    /// The rewind history.
    pub fn rewind_reel(&self, session: SessionId) -> Result<RewindReel> {
        self.record("rewind_reel", |o, l| unsafe { self.lib().emusen_crystal_rewind_reel(self.rt(), session, o, l) })
    }

    /// The picture of the moment at `frame`: width and height as two little-endian u32, then RGBA8888 rows.
    pub fn rewind_picture(&self, session: SessionId, frame: u64) -> Result<Option<Vec<u8>>> {
        self.read_absent(|o, l| unsafe { self.lib().emusen_crystal_rewind_picture(self.rt(), session, frame, o, l) })
    }

    /// Back to the moment at `frame`.
    pub fn rewind_to(&self, session: SessionId, frame: u64) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_rewind_to(self.rt(), session, frame) }.into()).map(drop)
    }

    /// Slots 1 to 8 and the resume state, slot 0.
    pub fn state_slots(&self, session: SessionId) -> Result<Vec<StateSlot>> {
        self.record("state_slots", |o, l| unsafe { self.lib().emusen_crystal_state_slots(self.rt(), session, o, l) })
    }

    /// A slot from 1 to 8 saved.
    pub fn state_slot_save(&self, session: SessionId, slot: u32) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_state_slot_save(self.rt(), session, slot) }.into()).map(drop)
    }

    /// A slot's state loaded; false where none exists.
    pub fn state_slot_load(&self, session: SessionId, slot: u32) -> Result<bool> {
        absent_as_false(self.status(unsafe { self.lib().emusen_crystal_state_slot_load(self.rt(), session, slot) }.into()))
    }

    /// The resume state saved.
    pub fn resume_save(&self, session: SessionId) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_resume_save(self.rt(), session) }.into()).map(drop)
    }

    /// The resume state loaded; false where none exists.
    pub fn resume_load(&self, session: SessionId) -> Result<bool> {
        absent_as_false(self.status(unsafe { self.lib().emusen_crystal_resume_load(self.rt(), session) }.into()))
    }

    /// The record states.db keeps of the state at `path`.
    pub fn state_record(&self, path: &str) -> Result<Option<StateRecord>> {
        self.record_absent("state_record", |o, l| unsafe {
            self.lib().emusen_crystal_state_record(self.rt(), path.as_ptr(), path.len(), o, l)
        })
    }

    /// The record of the state at `path` forgotten.
    pub fn state_forget(&self, path: &str) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_state_forget(self.rt(), path.as_ptr(), path.len()) }.into()).map(drop)
    }

    // ---- Settings ----------------------------------------------------------------------------------------

    /// A console's settings for an engine, or the session's with no console named.
    pub fn settings_schema(&self, session: SessionId, console: Option<&str>, engine: Option<&str>) -> Result<Vec<SettingRow>> {
        let (c, cl) = text(console);
        let (e, el) = text(engine);
        self.record("settings_schema", |o, l| unsafe { self.lib().emusen_crystal_settings_schema(self.rt(), session, c, cl, e, el, o, l) })
    }

    /// A value as the scope sees it; none for a key with no value and no default.
    pub fn settings_get(&self, session: SessionId, scope: Scope, console: Option<&str>, key: &str) -> Result<Option<String>> {
        let (c, cl) = text(console);
        let bytes = self.read_absent(|o, l| unsafe {
            self.lib().emusen_crystal_settings_get(self.rt(), session, scope.raw(), c, cl, key.as_ptr(), key.len(), o, l)
        })?;
        bytes.map(|b| string("settings_get", b)).transpose()
    }

    /// A value stored at the scope; none forgets the scope's own.
    pub fn settings_set(&self, session: SessionId, scope: Scope, console: Option<&str>, key: &str, value: Option<&str>) -> Result<()> {
        let (c, cl) = text(console);
        let (v, vl) = text(value);
        let n = unsafe { self.lib().emusen_crystal_settings_set(self.rt(), session, scope.raw(), c, cl, key.as_ptr(), key.len(), v, vl) };
        self.status(n.into()).map(drop)
    }

    /// A game's own value of a key, with no session.
    pub fn game_setting_get(&self, path: &str, key: &str) -> Result<Option<String>> {
        let bytes = self.read_absent(|o, l| unsafe {
            self.lib().emusen_crystal_game_setting_get(self.rt(), path.as_ptr(), path.len(), key.as_ptr(), key.len(), o, l)
        })?;
        bytes.map(|b| string("game_setting_get", b)).transpose()
    }

    /// A game's own value set, or forgotten with none.
    pub fn game_setting_set(&self, path: &str, key: &str, value: Option<&str>) -> Result<()> {
        let (v, vl) = text(value);
        let n = unsafe { self.lib().emusen_crystal_game_setting_set(self.rt(), path.as_ptr(), path.len(), key.as_ptr(), key.len(), v, vl) };
        self.status(n.into()).map(drop)
    }

    // ---- Firmware ----------------------------------------------------------------------------------------

    /// The required firmware the game needs on the engine and the library does not hold.
    pub fn firmware_needed(&self, path: &str, engine: Option<&str>) -> Result<Vec<FirmwareNeed>> {
        let (e, el) = text(engine);
        let entries: Vec<Value> = self.record("firmware_needed", |o, l| unsafe {
            self.lib().emusen_crystal_firmware_needed(self.rt(), path.as_ptr(), path.len(), e, el, o, l)
        })?;
        let read = |v: Value| serde_json::from_value::<FirmwareNeed>(v.clone()).map(|n| FirmwareNeed { request: v, ..n });
        entries
            .into_iter()
            .map(read)
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Shape { call: "firmware_needed", words: e.to_string() })
    }

    /// The file at `source` copied into the firmware folder under the request's name.
    pub fn firmware_install(&self, request: &FirmwareNeed, source: &str) -> Result<()> {
        let request = to_json(&request.request);
        let n = unsafe {
            self.lib().emusen_crystal_firmware_install(self.rt(), request.as_ptr(), request.len(), source.as_ptr(), source.len())
        };
        self.status(n.into()).map(drop)
    }

    /// Each discovered system's firmware.
    pub fn firmware_overview(&self) -> Result<FirmwareOverview> {
        self.record("firmware_overview", |o, l| unsafe { self.lib().emusen_crystal_firmware_overview(self.rt(), o, l) })
    }

    /// Each firmware file the system's engines declare; none for a system nothing names.
    pub fn firmware_status(&self, system: &str) -> Result<Option<FirmwareStatus>> {
        self.record_absent("firmware_status", |o, l| unsafe {
            self.lib().emusen_crystal_firmware_status(self.rt(), system.as_ptr(), system.len(), o, l)
        })
    }

    // ---- Save media and cards ----------------------------------------------------------------------------

    /// The session's save media.
    pub fn save_media(&self, session: SessionId) -> Result<SaveMedia> {
        self.record("save_media", |o, l| unsafe { self.lib().emusen_crystal_save_media(self.rt(), session, o, l) })
    }

    /// One of the card manager's ops, EMUSEN_CRYSTAL_CARDS_*, its arguments and answer as JSON.
    pub fn cards_op(&self, op: u32, args: &Value) -> Result<Option<Value>> {
        let args = to_json(args);
        self.record_absent("cards_op", |o, l| unsafe { self.lib().emusen_crystal_cards_op(self.rt(), op, args.as_ptr(), args.len(), o, l) })
    }

    // ---- Cheat codes -------------------------------------------------------------------------------------

    /// Every cheat codec and each system's pair.
    pub fn cheat_codec_list(&self) -> Result<CheatCodecs> {
        self.record("cheat_codec_list", |o, l| unsafe { self.lib().emusen_crystal_cheat_codec_list(self.rt(), o, l) })
    }

    /// A code's answers from one codec; none for a codec there is not.
    pub fn cheat_decode(&self, codec: &str, code: &str) -> Result<Option<CheatDecode>> {
        self.record_absent("cheat_decode", |o, l| unsafe {
            self.lib().emusen_crystal_cheat_decode(self.rt(), codec.as_ptr(), codec.len(), code.as_ptr(), code.len(), o, l)
        })
    }

    /// The Game Genie code that patches `value` at `address`; none for a codec with no encoder.
    pub fn cheat_encode(&self, codec: &str, address: i32, value: u32) -> Result<Option<String>> {
        let bytes = self.read_absent(|o, l| unsafe {
            self.lib().emusen_crystal_cheat_encode(self.rt(), codec.as_ptr(), codec.len(), address, value, o, l)
        })?;
        bytes.map(|b| string("cheat_encode", b)).transpose()
    }

    // ---- The recorder ------------------------------------------------------------------------------------

    /// Records every stride-th frame drawn into a new folder under `base`, which it returns.
    pub fn record_start(&self, session: SessionId, base: &str, stride: u32) -> Result<String> {
        let bytes = self
            .read(|o, l| unsafe { self.lib().emusen_crystal_record_start(self.rt(), session, base.as_ptr(), base.len(), stride, o, l) })?;
        string("record_start", bytes)
    }

    /// Ends the recording; the folder.
    pub fn record_stop(&self, session: SessionId) -> Result<String> {
        let bytes = self.read(|o, l| unsafe { self.lib().emusen_crystal_record_stop(self.rt(), session, o, l) })?;
        string("record_stop", bytes)
    }

    /// The phases the core times its frames in.
    pub fn phase_names(&self, session: SessionId) -> Result<Vec<String>> {
        self.record("phase_names", |o, l| unsafe { self.lib().emusen_crystal_phase_names(self.rt(), session, o, l) })
    }
}

/// A change that answers ABSENT where there was nothing to change, as false.
fn absent_as_false(r: Result<i64>) -> Result<bool> {
    match r {
        Ok(_) => Ok(true),
        Err(e) if crate::error::absent(&e) => Ok(false),
        Err(e) => Err(e),
    }
}
