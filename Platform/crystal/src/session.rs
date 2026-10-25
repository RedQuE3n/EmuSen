//! One open session: its engine, what it was opened with, and its answers about itself, the C# `ManagedSession`'s
//! life and `Info` (EmuSen_Backend.md §29.2). The frame, input, speed, rewind and states join it in their pieces.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use emusen_core_host::InputLatch;

use emusen_galaxia::tree::Tree;
use emusen_native::core::sys::caps;
use emusen_native::json::Value;

use crate::catalog::Catalog;
use crate::abi::{self, emusen_crystal_frame_info};
use crate::engine::Engine;
use crate::input::Routes;
use crate::json::Json;
use crate::seam::Debugger as _;
use crate::speed::{NORMAL_PERCENT, Speed};
use emusen_session_service::abi::status as svc;

/// A refusal: its status and words.
pub type Refused = (i32, String);

/// 1.0's `emusen_crystal_frame_info`, the shortest taken.
pub const FRAME_INFO_SIZE_10: u32 = 104;

/// A number for the calling thread, never 0, as `Environment.CurrentManagedThreadId` is one.
pub fn thread_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    thread_local! { static ID: u64 = NEXT.fetch_add(1, Ordering::Relaxed); }
    ID.with(|id| *id)
}

/// What a session was opened with besides its game.
#[derive(Clone, Copy, Debug, Default)]
pub struct OpenedWith {
    pub flags: u32,
    pub audio_buffer_max_samples: u32,
    /// Since 1.1: the presenter's device, each machine lent it afresh; 0 for none.
    pub device: usize,
}

/// The state a session's frame thread and run control share: whether it runs, at what speed, and its rewind.
pub struct Run {
    pub paused: bool,
    pub halted: bool,
    pub run_requested: bool,
    pub speed: Speed,
    pub draw_none: bool,
    pub rewind: crate::rewind::Rewind,
    base_percent: u32,
    mode: u32,
}

impl Default for Run {
    fn default() -> Run {
        Run { paused: false, halted: false, run_requested: false, speed: Speed::default(), draw_none: false, rewind: crate::rewind::Rewind::default(), base_percent: NORMAL_PERCENT as u32, mode: abi::SPEED_NORMAL }
    }
}

impl Run {
    fn apply_mode(&mut self) {
        let percent = match self.mode {
            abi::SPEED_TURBO => self.speed.turbo(),
            abi::SPEED_SLOW => self.speed.slow(),
            _ => self.base_percent.min(i32::MAX as u32) as i32,
        };
        self.speed.set_percent(percent);
    }
}

/// The gate held: the engine, and the thread that holds it marked so that it cannot take it again.
pub struct Held<'a> {
    owner: &'a AtomicU64,
    pub engine: MutexGuard<'a, Engine>,
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        self.owner.store(0, Ordering::Release);
    }
}

pub struct Session {
    pub id: u32,
    pub engine: Mutex<Engine>,
    pub game_path: Option<String>,
    /// The package the game opened as, whose folder keeps its states (EmuSen_GamePackages.md §3.5).
    pub package: Option<String>,
    /// The patch list's save key the game opened with, naming its states apart (EmuSen_GamePackages.md §5.5).
    pub save_key: Option<String>,
    pub console: String,
    /// The console a session started empty was asked for.
    pub empty_console: Option<String>,
    pub opened: OpenedWith,
    pub run: Mutex<Run>,
    pub latch: Arc<InputLatch>,
    pub routes: Arc<Mutex<Routes>>,
    pending: AtomicU32,
    in_frame: AtomicU64,
    frame_thread: AtomicU64,
    owner: AtomicU64,
    closed: AtomicBool,
    woken: Mutex<bool>,
    wake: Condvar,
    /// The picture as shown, for the recorder.
    shown: Mutex<Vec<u8>>,
    /// The game's MD5 and size, read once on a worker as the session opens; none until then.
    pub identity: Arc<Mutex<(Option<String>, i64)>>,
    pub events: Arc<crate::events::Events>,
    /// The core's log lines already raised as events.
    logged_lines: std::sync::atomic::AtomicI64,
    /// The session service's own state for this session: run control's requests and the records kept.
    pub provider: crate::service::Provider,
}

fn controllers_of_system(info: &Value, system: &str) -> Vec<Value> {
    let systems = info.get("systems").and_then(Value::as_array).unwrap_or(&[]);
    let chosen = systems.iter().find(|s| s.get("id").and_then(Value::as_str) == Some(system)).or(systems.first());
    chosen.and_then(|s| s.get("controllers")).and_then(Value::as_array).map(<[Value]>::to_vec).unwrap_or_default()
}

impl Session {
    pub fn new(id: u32, engine: Engine, game_path: Option<String>, console: String, empty_console: Option<String>, opened: OpenedWith, events: Arc<crate::events::Events>) -> Session {
        let (latch, routes) = (engine.latch.clone(), engine.routes.clone());
        let identity = Arc::new(Mutex::new((None, 0)));
        if let Some(at) = game_path.clone() {
            let theirs = identity.clone();
            // A patched package's identity, set at open, is never overwritten by the file's (EmuSen_GamePackages.md §5.5).
            let _ = std::thread::Builder::new().name("crystal-identify".to_string()).spawn(move || {
                let bytes = std::fs::metadata(&at).map_or(0, |m| m.len() as i64);
                let md5 = emusen_galaxia::rom_hash::md5(&at).ok();
                let mut known = theirs.lock().unwrap_or_else(|p| p.into_inner());
                if known.0.is_none() {
                    *known = (md5, bytes);
                }
            });
        }
        Session {
            id,
            engine: Mutex::new(engine),
            game_path,
            package: None,
            save_key: None,
            console,
            empty_console,
            opened,
            run: Mutex::new(Run::default()),
            latch,
            routes,
            pending: AtomicU32::new(0),
            in_frame: AtomicU64::new(0),
            frame_thread: AtomicU64::new(0),
            owner: AtomicU64::new(0),
            closed: AtomicBool::new(false),
            woken: Mutex::new(false),
            wake: Condvar::new(),
            shown: Mutex::new(Vec::new()),
            identity,
            events,
            logged_lines: std::sync::atomic::AtomicI64::new(0),
            provider: crate::service::Provider::default(),
        }
    }

    fn closed_refusal(&self) -> Refused {
        (svc::NO_SESSION, format!("session {} was closed", self.id))
    }

    /// The gate, taken after the frame in progress and before the next: counted while waiting, so that the frame thread
    /// lets every waiter in, a read naming the machine as well as a change (EmuSen_Backend.md §29.28).
    pub fn lock(&self) -> Result<Held<'_>, Refused> {
        let me = thread_id();
        if self.owner.load(Ordering::Acquire) == me {
            return Err((abi::status::BUSY, format!("this thread already holds session {}'s gate", self.id)));
        }
        self.pending.fetch_add(1, Ordering::AcqRel);
        let engine = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        self.pending.fetch_sub(1, Ordering::AcqRel);
        self.owner.store(me, Ordering::Release);
        let held = Held { owner: &self.owner, engine };
        if self.closed.load(Ordering::Acquire) {
            return Err(self.closed_refusal());
        }
        Ok(held)
    }

    /// A change between two frames, the caller waiting for the frame in progress, `ManagedSession.Change`.
    pub fn change<T>(&self, change: impl FnOnce(&mut Engine, &mut Run) -> Result<T, Refused>) -> Result<T, Refused> {
        let (result, said) = {
            let mut held = self.lock()?;
            let mut run = self.run.lock().unwrap_or_else(|p| p.into_inner());
            let result = change(&mut held.engine, &mut run);
            (result, held.engine.take_said())
        };
        self.events.raise_runtime_lines(self.id, &said);
        if thread_id() != self.frame_thread.load(Ordering::Acquire) {
            self.wake_up();
        }
        result
    }

    /// The engine's lines said since the last time, as one log event of source "runtime", `RaiseSaid`.
    pub fn raise_said(&self, e: &Engine) {
        let said = e.take_said();
        self.events.raise_runtime_lines(self.id, &said);
    }

    /// The core's lines logged since the last time, as one log event of source "core", `RaiseLog`.
    fn raise_log(&self, e: &Engine) {
        let total = e.logged_lines;
        let seen = self.logged_lines.swap(total, Ordering::AcqRel);
        if total == seen {
            return;
        }
        let recent = e.recent_log();
        let fresh = ((total - seen).max(0) as usize).min(recent.len());
        self.events.raise(self.id, abi::EVENT_LOG, "log", |j| {
            j.key_array("lines");
            for l in recent.iter().skip(recent.len() - fresh) {
                j.str_value(l);
            }
            j.end_array().str("source", "core");
        });
    }

    /// The session closed under its gate: whatever comes after is refused.
    pub fn close(&self, done: impl FnOnce(&mut Engine)) -> Result<(), Refused> {
        self.change(|e, _| {
            done(e);
            self.closed.store(true, Ordering::Release);
            Ok(())
        })?;
        self.wake_up();
        Ok(())
    }

    fn wake_up(&self) {
        let mut woken = self.woken.lock().unwrap_or_else(|p| p.into_inner());
        *woken = true;
        self.wake.notify_all();
    }

    /// The paused frame thread's wait: true when a change was applied or run control asked to run.
    pub fn wait(&self, timeout_ms: u32) -> bool {
        self.frame_thread.store(thread_id(), Ordering::Release);
        let mut woken = self.woken.lock().unwrap_or_else(|p| p.into_inner());
        if !*woken {
            woken = self.wake.wait_timeout(woken, std::time::Duration::from_millis(timeout_ms.min(i32::MAX as u32) as u64)).unwrap_or_else(|p| p.into_inner()).0;
        }
        std::mem::replace(&mut *woken, false)
    }

    /// One frame, or none while paused or halted, its facts into `info`, `ManagedSession.Frame`.
    pub fn frame(&self, info: Option<&mut emusen_crystal_frame_info>) -> Result<(), Refused> {
        if let Some(i) = &info
            && i.size < FRAME_INFO_SIZE_10
        {
            return Err((abi::status::BAD_STRUCT, format!("emusen_crystal_frame_info of {} bytes; version 1.0's is {FRAME_INFO_SIZE_10}", i.size)));
        }
        let me = thread_id();
        let other = match self.in_frame.compare_exchange(0, me, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => 0,
            Err(other) => other,
        };
        if other != 0 && other != me {
            return Err((abi::status::BUSY, format!("another thread is inside a frame of session {}; a session has one frame thread", self.id)));
        }
        let result = (|| {
            while self.pending.load(Ordering::Acquire) > 0 {
                std::thread::yield_now();
            }
            let mut held = self.lock()?;
            self.frame_thread.store(me, Ordering::Release);
            let mut run = self.run.lock().unwrap_or_else(|p| p.into_inner());
            let ran = if (!run.paused && !run.halted) || run.run_requested { self.advance(&mut held.engine, &mut run)? } else { false };
            if let Some(info) = info {
                self.fill(&held.engine, &run, info, ran);
            }
            Ok(())
        })();
        if other == 0 {
            self.in_frame.store(0, Ordering::Release);
        }
        result
    }

    /// The frame-end order, written once, EmuSen_Backend.md §29.4.
    fn advance(&self, e: &mut Engine, run: &mut Run) -> Result<bool, Refused> {
        run.run_requested = false;
        let recording = e.debug.recording();
        let skip = !recording && (run.draw_none || !run.speed.should_render(e.total_frames()));
        e.set_skip_rendering(skip);
        run.halted = false;
        e.run_frame().map_err(|r| (r.status, r.words))?;
        if let Some((processor, address)) = e.halted {
            run.halted = true;
            e.refresh_debugger();
            let name = e.halted_name(processor);
            self.events.raise(self.id, abi::EVENT_RUN, "run", |j| {
                j.str("op", "halted").str("processor", &name).int("address", address as i64);
            });
            return Ok(false);
        }
        e.refresh_debugger();
        if run.rewind.enabled {
            let took = run.rewind.on_frame_completed(&mut crate::engine::Snap { engine: e, ctx: None })?;
            if took && run.rewind.thumbnail_width > 0 && !e.skip_rendering {
                self.attach_thumbnail(e, &mut run.rewind);
            }
        }
        if recording && !skip {
            self.capture(e);
        }
        self.raise_said(e);
        self.raise_log(e);
        Ok(true)
    }

    /// The picture on show attached to the snapshot just taken, `AttachThumbnail`.
    fn attach_thumbnail(&self, e: &mut Engine, rewind: &mut crate::rewind::Rewind) {
        let repeat = self.repeat_rows();
        let rr = e.shape.row_repeat.max(1);
        let mut shown = self.shown.lock().unwrap_or_else(|p| p.into_inner());
        let bytes = e.copy_picture(&mut [], repeat).unwrap_or(0);
        shown.resize(bytes, 0);
        let _ = e.copy_picture(&mut shown, repeat);
        let (width, rows) = (e.shape.width, e.shape.height * if repeat { rr } else { 1 });
        rewind.attach_thumbnail(&shown, width, rows, if repeat { 1 } else { rr });
    }

    /// Rewind's settings; turned on, it takes its first snapshot; turned off, it lets go of its chain.
    pub fn rewind_set(&self, enabled: bool, interval: u32, budget: u64, thumbnail_width: u32) -> Result<(), Refused> {
        self.change(|e, run| {
            let r = &mut run.rewind;
            if interval > 0 {
                r.set_interval(interval.min(i32::MAX as u32) as i32);
            }
            if budget > 0 {
                r.budget = budget.min(i64::MAX as u64) as i64;
            }
            r.thumbnail_width = thumbnail_width.min(i32::MAX as u32) as i32;
            if enabled && !r.enabled {
                r.enabled = true;
                r.capture_now(&mut crate::engine::Snap { engine: e, ctx: None })?;
            } else if !enabled && r.enabled {
                r.enabled = false;
                r.clear();
                self.events.raise(self.id, abi::EVENT_REWIND, "rewind", |j| {
                    j.str("op", "cleared");
                });
            }
            Ok(())
        })
    }

    /// Held rewind's step in place of a frame: the sound it skipped dropped, the picture the snapshot left on show.
    pub fn rewind_step(&self, count: u32, info: Option<&mut emusen_crystal_frame_info>, ctx: &crate::engine::Ctx) -> Result<i64, Refused> {
        if let Some(i) = &info
            && i.size < FRAME_INFO_SIZE_10
        {
            return Err((abi::status::BAD_STRUCT, format!("emusen_crystal_frame_info of {} bytes; version 1.0's is {FRAME_INFO_SIZE_10}", i.size)));
        }
        self.change(|e, run| {
            e.set_skip_rendering(false);
            let mut done = 0i64;
            while done < count as i64 && run.rewind.step(&mut crate::engine::Snap { engine: e, ctx: Some(ctx) })? {
                done += 1;
            }
            e.refresh_debugger();
            e.drop_audio();
            if let Some(info) = info {
                self.fill(e, run, info, false);
                info.rendered = (done > 0) as u32;
            }
            if done > 0 {
                let frame = run.rewind.frame();
                self.events.raise(self.id, abi::EVENT_REWIND, "rewind", |j| {
                    j.str("op", "stepped").int("frame", frame);
                });
            }
            Ok(done)
        })
    }

    /// Straight to a held moment.
    pub fn rewind_to(&self, frame: u64, ctx: &crate::engine::Ctx) -> Result<(), Refused> {
        self.change(|e, run| {
            let at = frame.min(i64::MAX as u64) as i64;
            if !run.rewind.to(&mut crate::engine::Snap { engine: e, ctx: Some(ctx) }, at)? {
                return Err((svc::ABSENT, format!("no moment at frame {frame} is held")));
            }
            run.halted = false;
            e.refresh_debugger();
            self.events.raise(self.id, abi::EVENT_REWIND, "rewind", |j| {
                j.str("op", "moved").int("frame", at);
            });
            Ok(())
        })
    }

    /// `{"enabled",…,"moments":[…]}`, `ReelJson`: the counts read before the chain settles, as the C#'s are.
    pub fn rewind_reel_json(&self) -> Result<String, Refused> {
        self.change(|e, run| {
            let r = &mut run.rewind;
            let mut j = Json::new();
            j.begin_object()
                .bool("enabled", r.enabled)
                .int("interval_frames", r.interval() as i64)
                .int("budget_bytes", r.budget)
                .int("depth", r.depth())
                .int("buffered_bytes", r.buffered_bytes())
                .int("snapshot_bytes", r.snapshot_bytes())
                .f64("seconds", r.buffered_seconds(e.frame_rate_hz()))
                .key_array("moments");
            for m in r.moments() {
                j.begin_object().int("frame", m.frame).int("core_frames", m.core_frames);
                match &m.thumbnail {
                    None => {
                        j.null("picture");
                    }
                    Some(t) => {
                        j.key_object("picture").int("width", t.width as i64).int("height", t.height as i64).end_object();
                    }
                }
                j.end_object();
            }
            j.end_array().end_object();
            Ok(j.finish())
        })
    }

    /// A moment's picture: its width and height, little-endian, then RGBA8888.
    pub fn rewind_picture(&self, frame: u64) -> Result<Vec<u8>, Refused> {
        self.change(|_, run| {
            let moment = run.rewind.moments().iter().find(|m| m.frame as u64 == frame);
            let Some(t) = moment.and_then(|m| m.thumbnail.as_ref()) else { return Err((svc::ABSENT, format!("no picture is held for frame {frame}"))) };
            let mut bytes = Vec::with_capacity(8 + (t.width * t.height * 4) as usize);
            bytes.extend_from_slice(&(t.width as u32).to_le_bytes());
            bytes.extend_from_slice(&(t.height as u32).to_le_bytes());
            t.to_rgba(&mut bytes);
            Ok(bytes)
        })
    }

    /// The picture on show, rows repeated, as a state's or a recording's picture: RGBA, width and height.
    pub fn shown_picture(&self, e: &mut Engine) -> (Vec<u8>, i32, i32) {
        let bytes = e.copy_picture(&mut [], true).unwrap_or(0);
        let mut rgba = vec![0; bytes];
        let _ = e.copy_picture(&mut rgba, true);
        (rgba, e.shape.width, e.shape.height * e.shape.row_repeat.max(1))
    }

    /// The recorder's picture of the frame, its rows repeated as shown, `CaptureFrame`.
    fn capture(&self, e: &mut Engine) {
        let mut shown = self.shown.lock().unwrap_or_else(|p| p.into_inner());
        let bytes = e.copy_picture(&mut [], true).unwrap_or(0);
        shown.resize(bytes, 0);
        let _ = e.copy_picture(&mut shown, true);
        let (width, height) = (e.shape.width, e.shape.height * e.shape.row_repeat.max(1));
        let frame = e.total_frames();
        e.debug.capture(frame, &shown, width, height);
    }

    fn fill(&self, e: &Engine, run: &Run, info: &mut emusen_crystal_frame_info, ran: bool) {
        let hz = e.frame_rate_hz();
        let repeat = self.repeat_rows();
        let rows = e.shape.row_repeat.max(1);
        info.ran = ran as u32;
        info.halted = run.halted as u32;
        info.rendered = (ran && !e.skip_rendering) as u32;
        info.picture_serial = e.frame_serial();
        info.width = e.shape.width;
        info.height = e.shape.height * if repeat { rows } else { 1 };
        info.row_repeat = if repeat { 1 } else { rows };
        info.display_lines = e.shape.lines * if repeat { rows } else { 1 };
        info.display_aspect = e.display_aspect();
        info.audio_frames = e.buffered_audio_frames().max(0) as u32;
        info.audio_rate = e.audio_sample_rate().max(0) as u32;
        info.play_audio = run.speed.should_play_audio() as u32;
        info.speed_percent = run.speed.percent() as u32;
        info.frame_interval_ns = run.speed.frame_interval_ns(hz);
        info.total_frames = e.total_frames().max(0) as u64;
        info.frame_rate_hz = hz;
        info.paused = run.paused as u32;
        info.halted_processor = if run.halted { e.halted_index() } else { 0 };
        info.halted_address = if run.halted { e.halted.map_or(0, |(_, address)| address as u64) } else { 0 };
        if (info.size as usize) < std::mem::size_of::<emusen_crystal_frame_info>() {
            return;
        }
        info.picture_on_device = (ran && e.picture_on_device) as u32;
        info.reserved = 0;
    }

    /// The current machine's lend for the presenter to borrow, or 0.
    pub fn picture_lend(&self) -> Result<usize, Refused> {
        Ok(self.lock()?.engine.lend.as_ref().map_or(0, |l| l.handle))
    }

    /// The raw encoding kept from the next frame; none keeps none.
    pub fn raw_select(&self, encoding: Option<&str>) -> Result<(), Refused> {
        self.lock()?.engine.raw_wanted = encoding.map(str::to_string);
        Ok(())
    }

    /// The last frame's raw picture into `into` and its shape; ABSENT for none.
    pub fn raw_copy(&self, into: &mut [u8]) -> Result<(emusen_native::core::sys::RawFrame, usize), Refused> {
        let held = self.lock()?;
        held.engine.copy_raw(into).map_err(|r| (r.status, r.words))?.ok_or_else(|| (svc::ABSENT, "no picture before the palette is kept".to_string()))
    }

    /// The picture as RGBA8888 into `into`, read whole; the bytes it takes.
    pub fn copy_picture(&self, into: &mut [u8]) -> Result<usize, Refused> {
        let mut held = self.lock()?;
        held.engine.copy_picture(into, self.repeat_rows()).map_err(|r| (r.status, r.words))
    }

    /// The sound drained into `into`, or with none the frames held; and the rate.
    pub fn drain_audio(&self, into: Option<&mut [i16]>) -> Result<(i64, u32), Refused> {
        let mut held = self.lock()?;
        let e = &mut held.engine;
        match into {
            None => Ok((e.buffered_audio_frames(), e.audio_sample_rate().max(0) as u32)),
            Some(into) => {
                let (n, rate) = e.drain_audio(into);
                Ok((n as i64, rate.max(0) as u32))
            }
        }
    }

    /// The phases' times into `into`; how many there are.
    pub fn phases(&self, into: &mut [f64]) -> Result<usize, Refused> {
        Ok(self.lock()?.engine.phases(into))
    }

    /// `["name",…]`, the phases' names.
    pub fn phase_names_json(&self) -> Result<String, Refused> {
        let names = self.lock()?.engine.phase_names();
        let mut j = Json::new();
        j.begin_array();
        for n in &names {
            j.str_value(n);
        }
        j.end_array();
        Ok(j.finish())
    }

    pub fn button(&self, port: u32, control: u32, pressed: bool) -> Result<(), Refused> {
        if control as usize >= crate::info::BUTTONS.len() {
            return Err((svc::NOT_SUPPORTED, format!("no control {control} in 1.{}", abi::MINOR)));
        }
        crate::input::button(&self.routes, &self.latch, port, control as usize, pressed);
        Ok(())
    }

    pub fn axis(&self, port: u32, axis: u32, value: f64) -> Result<(), Refused> {
        if axis as usize >= crate::info::AXES.len() {
            return Err((svc::NOT_SUPPORTED, format!("no axis {axis} in 1.{}", abi::MINOR)));
        }
        if value.is_nan() {
            return Err((svc::BAD_ARGUMENT, "an axis value that is not a number".to_string()));
        }
        crate::input::axis(&self.routes, &self.latch, port, axis as usize, value);
        Ok(())
    }

    pub fn connected(&self, port: u32, connected: bool) {
        self.latch.connect(port as usize, connected);
    }

    pub fn speed_set(&self, percent: u32, turbo: u32, slow: u32, flags: u32) -> Result<(), Refused> {
        if flags & !abi::SPEED_DRAW_NONE != 0 {
            return Err((svc::NOT_SUPPORTED, format!("speed flags 0x{flags:X} are not known to 1.{}", abi::MINOR)));
        }
        self.change(|_, run| {
            run.base_percent = percent;
            run.speed.set_turbo(turbo.min(i32::MAX as u32) as i32);
            run.speed.set_slow(slow.min(i32::MAX as u32) as i32);
            run.draw_none = flags & abi::SPEED_DRAW_NONE != 0;
            run.apply_mode();
            Ok(())
        })
    }

    pub fn speed_mode(&self, mode: u32) -> Result<(), Refused> {
        if mode > abi::SPEED_SLOW {
            return Err((svc::NOT_SUPPORTED, format!("no speed mode {mode} in 1.{}", abi::MINOR)));
        }
        self.change(|_, run| {
            run.mode = mode;
            run.apply_mode();
            Ok(())
        })
    }

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
        match e.halted.filter(|_| run.halted) {
            Some((processor, address)) => {
                j.key_object("halted").str("processor", &e.halted_name(processor)).int("address", address as i64).end_object();
            }
            None => {
                j.null("halted");
            }
        }
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
