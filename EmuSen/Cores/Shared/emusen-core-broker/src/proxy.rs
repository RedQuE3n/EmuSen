//! The proxy: every v1 export answered by a runner, a library-level one for the descriptors and one per machine, so the
//! host loads an ordinary v1 library and the core runs in a confined process of its own (EmuSen_LibretroHost.md §3).

use std::ffi::{CStr, c_char};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use emusen_native::core::sys::{CreateParams, Event, FileEntry, FrameInfo, Machine as RawMachine, Media, RingFrame, RingImage, RingLend, caps, flags, status};

use crate::client::{Link, STATUS_ENDED, STATUS_HUNG, Spawn};
use crate::wire::{Op, Reply, Writer};

/// What a proxy library knows of the core it serves.
pub trait Target: 'static {
    /// The capability bits, the same as the served library's, and the exports this proxy has.
    const CAPABILITIES: u64;
    /// How a runner starts: for the library's descriptors with no image, or for a machine with its image's path.
    fn spawn(image_path: Option<&str>) -> Result<Spawn, String>;
    /// Members added to core info's object, each `"key":value` with no comma at either end, or empty.
    fn info_extra() -> String {
        String::new()
    }
    /// Whether the served core's memory holds still between calls, measured, so the frame-end window is exact for it;
    /// a core whose threads run between frames reads live (EmuSen_LibretroHost.md §10c).
    fn window() -> bool {
        false
    }
    /// For a target claiming `EXPECT`: the bytes it serves sealed now, before any runs, and compared with the approved
    /// hash; the refusal names both (EmuSen_CoreAPI.md §6.42).
    fn expect(_sha256: &str) -> Result<(), String> {
        Err("this proxy serves nothing it can compare".to_owned())
    }
    /// Whether the served core's firmware answer reads the image; one that does not is sent none, so a disc image is
    /// never read through and copied to the runner to be ignored (EmuSen_LibretroHost.md §10r).
    fn firmware_reads_image() -> bool {
        true
    }
}

/// The proxy's own setting: the deadline of a frame, in seconds; create and free get six times it.
pub const DEADLINE_KEY: &str = "broker.deadline";
const DEADLINE_DEFAULT: u64 = 10;

static LIBRARY: Mutex<Option<Link>> = Mutex::new(None);
static CRASH_LOG: Mutex<Option<String>> = Mutex::new(None);
static LIBRARY_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static INFO: OnceLock<Result<(String, String, u32), String>> = OnceLock::new();
/// Since 1.17: the hash `expect` was given and its outcome; a mismatch leaves the library broken for the process.
static EXPECTED: OnceLock<(String, Result<(), String>)> = OnceLock::new();

/// The words of a failed `expect`, after which nothing the proxy serves runs.
fn broken() -> Option<&'static str> {
    EXPECTED.get().and_then(|e| e.1.as_ref().err()).map(String::as_str)
}

/// One machine's handle: its runner, what the proxy itself says of it, and the frame-end window.
pub struct Proxied {
    link: Link,
    log: Vec<String>,
    frame: Duration,
    window: Window,
}

/// The memory a host read between the last two frames, copied by the runner at each frame's end so that reads in it
/// cost no round trip; exact, since the machine does not run between calls (EmuSen_LibretroHost.md §3.2).
#[derive(Default)]
pub struct Window {
    /// (space, address, bytes) as the runner copied them, valid until a call that changes the machine.
    held: Vec<(u32, u32, Vec<u8>)>,
    valid: bool,
    enabled: bool,
    /// The ranges read this frame, asked for at the next frame's end.
    wanted: Vec<(u32, u32, u32)>,
}

/// The most the window copies at a frame's end.
pub const WINDOW_MAX: u32 = 1 << 20;

impl Window {
    fn read(&self, space: u32, address: u32, out: &mut [u8]) -> bool {
        if !self.valid {
            return false;
        }
        let end = address as u64 + out.len() as u64;
        match self.held.iter().find(|(s, a, b)| *s == space && *a <= address && end <= *a as u64 + b.len() as u64) {
            Some((_, a, b)) => {
                let at = (address - a) as usize;
                out.copy_from_slice(&b[at..at + out.len()]);
                true
            }
            None => false,
        }
    }

    fn wrote(&mut self, space: u32, address: u32, data: &[u8]) {
        for (s, a, b) in self.held.iter_mut().filter(|(s, ..)| *s == space) {
            let _ = s;
            for (i, &v) in data.iter().enumerate() {
                let at = address as u64 + i as u64;
                if at >= *a as u64 && at < *a as u64 + b.len() as u64 {
                    b[(at - *a as u64) as usize] = v;
                }
            }
        }
    }

    /// A read the window did not hold, remembered for the next frame: merged with a neighbour within 256 bytes.
    fn want(&mut self, space: u32, address: u32, len: u32) {
        if !self.enabled {
            return;
        }
        let total: u64 = self.wanted.iter().map(|w| w.2 as u64).sum();
        if total + len as u64 > WINDOW_MAX as u64 {
            return;
        }
        let (lo, hi) = (address as u64, address as u64 + len as u64);
        match self.wanted.iter_mut().find(|(s, a, l)| *s == space && lo <= *a as u64 + *l as u64 + 256 && *a as u64 <= hi + 256) {
            Some(w) => {
                let new_lo = lo.min(w.1 as u64);
                let new_hi = hi.max(w.1 as u64 + w.2 as u64);
                (w.1, w.2) = (new_lo as u32, (new_hi - new_lo).min(u32::MAX as u64) as u32);
            }
            None => self.wanted.push((space, address, len)),
        }
    }
}

fn record(log: &mut Vec<String>, level: &str, text: &str) {
    log.push(format!("{level}\tbroker\t{}\n", text.replace(['\t', '\n'], " ")));
}

/// A text the export copies out: the whole length, and as much as `len` holds.
unsafe fn copy_out(text: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() {
        let n = text.len().min(len);
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, n) };
    }
    text.len() as i64
}

fn library_call(op: Op, out_null: bool, out_len: u64, args: &[u8]) -> Result<Reply, (i32, String)> {
    if let Some(why) = broken() {
        return Err((status::NOT_EXPECTED, why.to_owned()));
    }
    let mut guard = LIBRARY.lock().unwrap_or_else(|e| e.into_inner());
    for attempt in 0..2 {
        if guard.as_ref().is_none_or(|l| l.dead.is_some()) {
            let spawn = Target_spawn_library()?;
            match Link::spawn(&spawn, Duration::from_secs(60)) {
                Ok(l) => {
                    record(&mut LIBRARY_LOG.lock().unwrap_or_else(|e| e.into_inner()), "info", &format!("the library's runner started: {}", l.hello));
                    *guard = Some(l);
                }
                Err(why) => return Err((STATUS_ENDED, why)),
            }
        }
        match guard.as_mut().unwrap().call(op, out_null, out_len, args, Duration::from_secs(60)) {
            Ok(r) => return Ok(r),
            Err(e) if attempt == 1 => return Err(e),
            Err(_) => {}
        }
    }
    unreachable!()
}

/// The library runner's spawn, set by the export macro for its target.
static SPAWN_LIBRARY: OnceLock<fn() -> Result<Spawn, String>> = OnceLock::new();

#[allow(non_snake_case)]
fn Target_spawn_library() -> Result<Spawn, (i32, String)> {
    let f = SPAWN_LIBRARY.get().ok_or((STATUS_ENDED, "the proxy was given no target".to_owned()))?;
    f().map_err(|why| (STATUS_ENDED, why))
}

/// Core info, the schema and the served library's version, asked once.
fn descriptors<T: Target>() -> &'static Result<(String, String, u32), String> {
    let _ = SPAWN_LIBRARY.set(|| T::spawn(None));
    INFO.get_or_init(|| {
        let caps = library_call(Op::Capabilities, false, 0, &[]).map_err(|e| e.1)?;
        let forwarded = T::CAPABILITIES & !caps::EXPECT;
        if caps.ret as u64 != forwarded {
            return Err(format!("the served library claims capabilities {:#x}, and this proxy forwards {:#x}", caps.ret, forwarded));
        }
        let text = |op| -> Result<String, String> {
            let n = library_call(op, true, 0, &[]).map_err(|e| e.1)?.ret;
            if n < 0 {
                return Err(format!("{op:?} answered {n}"));
            }
            let r = library_call(op, false, n as u64, &[]).map_err(|e| e.1)?;
            Ok(String::from_utf8_lossy(&r.bytes).into_owned())
        };
        let mut info = text(Op::Info)?;
        if T::CAPABILITIES & caps::EXPECT != 0 {
            let list = info.find(r#""capabilities":["#).ok_or("the served library's info lists no capabilities")? + r#""capabilities":["#.len();
            let empty = info[list..].starts_with(']');
            info.insert_str(list, if empty { r#""EXPECT""# } else { r#""EXPECT","# });
        }
        let extra = T::info_extra();
        if !extra.is_empty()
            && let Some(end) = info.rfind('}')
        {
            info.insert_str(end, &format!(",{extra}"));
        }
        let mut schema = text(Op::SettingsSchema)?;
        let deadline = format!(
            r#"{{"key":"{DEADLINE_KEY}","label":"Answer within","help":"Seconds the core has to finish a frame before it is stopped as hung; loading and closing get six times as long.","kind":"count","default":"{DEADLINE_DEFAULT}","min":1,"max":600,"step":1,"scope":"run","category":"System","effect":"none","advanced":true}}"#
        );
        match schema.rfind(']') {
            Some(end) if schema[..end].trim_end().ends_with('[') => schema.insert_str(end, &deadline),
            Some(end) => schema.insert_str(end, &format!(",{deadline}")),
            None => schema = format!("[{deadline}]"),
        }
        Ok((info, schema, caps.aux as u32))
    })
}

pub fn abi_version<T: Target>() -> u32 {
    descriptors::<T>().as_ref().map_or(emusen_native::core::sys::ABI_VERSION, |d| d.2)
}

pub fn capabilities<T: Target>() -> u64 {
    T::CAPABILITIES
}

pub unsafe fn info<T: Target>(out: *mut u8, len: usize) -> i64 {
    match descriptors::<T>() {
        Ok(d) => unsafe { copy_out(d.0.as_bytes(), out, len) },
        Err(why) => {
            let text = format!(r#"{{"abi":"","id":"broken-proxy","name":"broken proxy","version":"0","license":"NOASSERTION","description":{why:?},"systems":[],"capabilities":[]}}"#);
            unsafe { copy_out(text.as_bytes(), out, len) }
        }
    }
}

pub unsafe fn settings_schema<T: Target>(out: *mut u8, len: usize) -> i64 {
    match descriptors::<T>() {
        Ok(d) => unsafe { copy_out(d.1.as_bytes(), out, len) },
        Err(_) => unsafe { copy_out(b"[]", out, len) },
    }
}

fn whole(r: Result<Reply, (i32, String)>, out: *mut u8, len: usize) -> i64 {
    match r {
        Ok(r) => {
            if !out.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr(), out, r.bytes.len().min(len)) };
            }
            r.ret
        }
        Err((code, _)) => code as i64,
    }
}

pub unsafe fn firmware_for<T: Target>(image: *const u8, image_len: usize, out: *mut u8, len: usize) -> i64 {
    let _ = descriptors::<T>();
    let bytes = if image.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(image, image_len) } };
    whole(library_call(Op::FirmwareFor, out.is_null(), len as u64, &firmware_request::<T>(bytes)), out, len)
}

/// The firmware call's arguments: the image, or none for a core whose answer does not read it.
fn firmware_request<T: Target>(image: &[u8]) -> Vec<u8> {
    let mut w = Writer::default();
    w.bytes(if T::firmware_reads_image() { image } else { &[] });
    w.0
}

pub unsafe fn status_text<T: Target>(code: i32, out: *mut u8, len: usize) -> i64 {
    let own = match code {
        STATUS_ENDED => Some("the core's process ended"),
        STATUS_HUNG => Some("the core stopped answering and was stopped"),
        status::NOT_EXPECTED => Some(broken().unwrap_or("the library serves other bytes than the approved build")),
        _ => None,
    };
    if let Some(words) = own {
        return unsafe { copy_out(words.as_bytes(), out, len) };
    }
    let _ = descriptors::<T>();
    let mut w = Writer::default();
    w.u32(code as u32);
    whole(library_call(Op::StatusText, out.is_null(), len as u64, &w.0), out, len)
}

pub unsafe fn set_crash_log<T: Target>(path: *const c_char) -> i32 {
    if path.is_null() {
        return status::NULL;
    }
    let text = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let mut c = CRASH_LOG.lock().unwrap_or_else(|e| e.into_inner());
    if c.is_none() {
        *c = Some(text.clone());
    }
    drop(c);
    let mut w = Writer::default();
    w.bytes(text.as_bytes());
    library_call(Op::SetCrashLog, false, 0, &w.0).map_or(0, |r| r.ret as i32)
}

unsafe fn proxied<'a>(m: *mut RawMachine) -> Option<&'a mut Proxied> {
    unsafe { (m as *mut Proxied).as_mut() }
}

pub unsafe fn log_drain<T: Target>(m: *mut RawMachine, out: *mut u8, len: usize) -> i64 {
    let (own, rest) = match unsafe { proxied(m) } {
        Some(p) => {
            let own: String = p.log.concat();
            let rest = if p.link.dead.is_some() { Ok(Reply { ret: 0, aux: 0, bytes: Vec::new() }) } else { p.link.call(Op::LogDrain, out.is_null(), len.saturating_sub(own.len()) as u64, &[], p.frame) };
            if !out.is_null() && own.len() <= len {
                p.log.clear();
            }
            (own, rest)
        }
        None => {
            let mut l = LIBRARY_LOG.lock().unwrap_or_else(|e| e.into_inner());
            let own: String = l.concat();
            if !out.is_null() && own.len() <= len {
                l.clear();
            }
            drop(l);
            let room = len.saturating_sub(own.len()) as u64;
            (own, library_call(Op::LogDrain, out.is_null(), room, &[]))
        }
    };
    let own_n = if out.is_null() || own.len() <= len { own.len() } else { 0 };
    if !out.is_null() && own_n > 0 {
        unsafe { std::ptr::copy_nonoverlapping(own.as_ptr(), out, own_n) };
    }
    let more = match rest {
        Ok(r) => {
            if !out.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr(), out.add(own_n), r.bytes.len().min(len - own_n)) };
            }
            r.ret.max(0)
        }
        Err(_) => 0,
    };
    (own_n as i64) + more
}

/// The settings text without the proxy's own key, and that key's deadline.
fn split_settings(text: &[u8]) -> (Vec<u8>, Option<u64>) {
    let text = String::from_utf8_lossy(text);
    let mut deadline = None;
    let mut kept = Vec::new();
    for line in text.lines() {
        match line.split_once('=') {
            Some((k, v)) if k.trim() == DEADLINE_KEY => deadline = v.trim().parse().ok().filter(|d| (1..=600).contains(d)),
            _ => kept.push(line),
        }
    }
    (kept.join("\n").into_bytes(), deadline)
}

fn files_of(w: &mut Writer, files: *const FileEntry, count: usize, size: usize) {
    w.u32(count as u32);
    for i in 0..count {
        let mut f: FileEntry = unsafe { std::mem::zeroed() };
        unsafe { std::ptr::copy_nonoverlapping((files as *const u8).add(i * size), &mut f as *mut FileEntry as *mut u8, size.min(size_of::<FileEntry>())) };
        w.u32(f.which);
        w.text((!f.name.is_null()).then(|| unsafe { CStr::from_ptr(f.name) }.to_bytes()));
        w.bytes(if f.data.is_null() { &[] } else { unsafe { std::slice::from_raw_parts(f.data, f.len) } });
    }
}

fn path_of(p: *const c_char) -> Option<String> {
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

/// Whether the served library reads its image by path alone, so its bytes need not cross.
fn reads_by_path<T: Target>() -> bool {
    descriptors::<T>().as_ref().is_ok_and(|d| d.0.contains(r#""image_path""#))
}

/// `emusen_core_expect`: the first hash given stands, and a later one that differs is refused without changing it.
pub unsafe fn expect<T: Target>(sha256: *const c_char) -> i32 {
    if sha256.is_null() {
        return status::NULL;
    }
    let given = unsafe { CStr::from_ptr(sha256) }.to_string_lossy().into_owned();
    let (first, outcome) = EXPECTED.get_or_init(|| {
        let outcome = if given.len() == 64 && given.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            T::expect(&given)
        } else {
            Err(format!("the approved hash {given:?} is not 64 lower-case hexadecimal digits"))
        };
        (given.clone(), outcome)
    });
    if *first != given || outcome.is_err() { status::NOT_EXPECTED } else { 0 }
}

pub unsafe fn create<T: Target>(params: *const CreateParams, status_out: *mut i32) -> *mut RawMachine {
    let set = |code: i32| {
        if !status_out.is_null() {
            unsafe { *status_out = code };
        }
    };
    let Some(given) = (unsafe { params.as_ref() }) else {
        set(status::NULL);
        return std::ptr::null_mut();
    };
    let size = given.size as usize;
    if size < CreateParams::SIZE_1_0 {
        set(status::BAD_STRUCT);
        return std::ptr::null_mut();
    }
    let mut p: CreateParams = unsafe { std::mem::zeroed() };
    unsafe { std::ptr::copy_nonoverlapping(params as *const u8, &mut p as *mut CreateParams as *mut u8, size.min(size_of::<CreateParams>())) };
    let refuse = |code: i32, why: &str| {
        if !p.error.is_null() && p.error_len > 0 {
            let n = why.len().min(p.error_len - 1);
            unsafe {
                std::ptr::copy_nonoverlapping(why.as_ptr(), p.error, n);
                *p.error.add(n) = 0;
            }
        }
        set(code);
        std::ptr::null_mut()
    };
    if let Some(why) = broken() {
        return refuse(status::NOT_EXPECTED, why);
    }
    if let Err(why) = descriptors::<T>() {
        return refuse(STATUS_ENDED, why);
    }
    if p.file_count > 0 && p.file_size < FileEntry::SIZE_1_0 {
        return refuse(status::BAD_STRUCT, "file entries shorter than 1.0's");
    }
    let image_path = path_of(p.image_path);
    let spawn = match T::spawn(image_path.as_deref()) {
        Ok(s) => s,
        Err(why) => return refuse(STATUS_ENDED, &why),
    };
    let settings = if p.settings.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(p.settings, p.settings_len) } };
    let (settings, deadline) = split_settings(settings);
    let frame = Duration::from_secs(deadline.unwrap_or(DEADLINE_DEFAULT));
    let mut link = match Link::spawn(&spawn, frame * 6) {
        Ok(l) => l,
        Err(why) => return refuse(STATUS_ENDED, &why),
    };
    let mut log = Vec::new();
    record(&mut log, "info", &format!("the core runs in process {}: {}", link.pid, link.hello));
    if let Some(path) = CRASH_LOG.lock().unwrap_or_else(|e| e.into_inner()).clone() {
        let mut w = Writer::default();
        w.bytes(path.as_bytes());
        let _ = link.call(Op::SetCrashLog, false, 0, &w.0, frame);
    }
    let mut w = Writer::default();
    w.bytes(&settings);
    files_of(&mut w, p.files, p.file_count, p.file_size);
    w.u64(p.pixel_formats).u32(p.flags & !flags::CREATE_LENT);
    w.text(path_of(p.system).as_deref().map(str::as_bytes));
    let image = if p.image.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(p.image, p.image_len) } };
    let by_path = image_path.is_some() && reads_by_path::<T>();
    w.text((!by_path).then_some(image));
    w.text(image_path.as_deref().map(str::as_bytes));
    match link.call(Op::Create, false, 0, &w.0, frame * 6) {
        Ok(r) if r.ret == 0 => Box::into_raw(Box::new(Proxied { link, log, frame, window: Window { enabled: T::window(), ..Window::default() } })) as *mut RawMachine,
        Ok(r) => refuse(r.ret as i32, &String::from_utf8_lossy(&r.bytes)),
        Err((code, why)) => refuse(code, &why),
    }
}

pub unsafe fn free<T: Target>(m: *mut RawMachine) -> i32 {
    if m.is_null() {
        return status::NULL;
    }
    let mut p = unsafe { Box::from_raw(m as *mut Proxied) };
    let frame = p.frame;
    match p.link.call(Op::Free, false, 0, &[], frame * 6) {
        Ok(r) => r.ret as i32,
        Err(_) => 0,
    }
}

/// A call on a machine: its reply, or the proxy's status for a runner gone. A call that may change the machine's memory
/// other than by a write ends the window.
fn on(m: *mut RawMachine, op: Op, out_null: bool, out_len: u64, args: &[u8]) -> Result<Reply, i32> {
    let Some(p) = (unsafe { proxied(m) }) else { return Err(status::NULL) };
    if matches!(op, Op::StateLoad | Op::Reset | Op::MediaInsert | Op::MediaLid | Op::SetSettings | Op::SetPresent | Op::BatterySaved) {
        p.window.valid = false;
    }
    let frame = p.frame;
    p.link.call(op, out_null, out_len, args, frame).map_err(|(code, why)| {
        if p.log.len() < 64 && !p.log.iter().any(|l| l.contains(&why)) {
            record(&mut p.log, "error", &why);
        }
        code
    })
}

fn code(r: Result<Reply, i32>) -> i64 {
    r.map_or_else(|c| c as i64, |r| r.ret)
}

fn text_on(m: *mut RawMachine, op: Op, args: &[u8], out: *mut u8, len: usize) -> i64 {
    match on(m, op, out.is_null(), len as u64, args) {
        Ok(r) => {
            if !out.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr(), out, r.bytes.len().min(len)) };
            }
            r.ret
        }
        Err(c) => c as i64,
    }
}

fn args(f: impl FnOnce(&mut Writer)) -> Vec<u8> {
    let mut w = Writer::default();
    f(&mut w);
    w.0
}

pub unsafe fn machine_info(m: *const RawMachine, out: *mut u8, len: usize) -> i64 {
    text_on(m as *mut _, Op::MachineInfo, &[], out, len)
}

pub unsafe fn last_error(m: *const RawMachine, out: *mut u8, len: usize) -> i64 {
    if let Some(p) = unsafe { proxied(m as *mut _) }
        && let Some((_, why)) = &p.link.dead
    {
        return unsafe { copy_out(why.as_bytes(), out, len) };
    }
    text_on(m as *mut _, Op::LastError, &[], out, len)
}

pub unsafe fn advance(m: *mut RawMachine, detail: *mut u64) -> i32 {
    let wanted = match unsafe { proxied(m) } {
        Some(p) => {
            p.window.valid = false;
            std::mem::take(&mut p.window.wanted)
        }
        None => return status::NULL,
    };
    let request = args(|w| {
        w.u32(wanted.len() as u32);
        for &(s, a, l) in &wanted {
            w.u32(s).u32(a).u32(l);
        }
    });
    match on(m, Op::Advance, false, 0, &request) {
        Ok(r) => {
            if !detail.is_null() {
                unsafe { *detail = r.aux as u64 };
            }
            if r.ret >= 0
                && let Some(p) = unsafe { proxied(m) }
            {
                let mut at = 0usize;
                p.window.held.clear();
                for &(s, a, l) in &wanted {
                    let n = l as usize;
                    if let Some(b) = r.bytes.get(at..at + n) {
                        p.window.held.push((s, a, b.to_vec()));
                    }
                    at += n;
                }
                p.window.valid = at == r.bytes.len();
            }
            r.ret as i32
        }
        Err(c) => c,
    }
}

pub unsafe fn set_options(m: *mut RawMachine, f: u32) -> i32 {
    code(on(m, Op::SetOptions, false, 0, &args(|w| {
        w.u32(f);
    }))) as i32
}

pub unsafe fn frame_count(m: *const RawMachine) -> i64 {
    code(on(m as *mut _, Op::FrameCount, false, 0, &[]))
}

pub unsafe fn events(m: *mut RawMachine, out: *mut Event, count: usize, event_size: usize) -> i64 {
    if event_size < size_of::<Event>() && !out.is_null() {
        return status::BAD_STRUCT as i64;
    }
    match on(m, Op::Events, out.is_null(), (count * size_of::<Event>()) as u64, &[]) {
        Ok(r) => {
            if !out.is_null() {
                for (i, e) in r.bytes.chunks_exact(size_of::<Event>()).enumerate().take(count) {
                    unsafe { std::ptr::copy_nonoverlapping(e.as_ptr(), (out as *mut u8).add(i * event_size), size_of::<Event>()) };
                }
            }
            r.ret
        }
        Err(c) => c as i64,
    }
}

pub unsafe fn frame_info(m: *const RawMachine, out: *mut FrameInfo) -> i32 {
    let Some(o) = (unsafe { out.as_mut() }) else { return status::NULL };
    let size = o.size as usize;
    if size < FrameInfo::SIZE_1_0 {
        return status::BAD_STRUCT;
    }
    match on(m as *mut _, Op::FrameInfo, false, 0, &[]) {
        Ok(r) => {
            if r.ret >= 0 {
                let n = size.min(r.bytes.len()).min(size_of::<FrameInfo>());
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr().add(4), (out as *mut u8).add(4), n - 4) };
            }
            r.ret as i32
        }
        Err(c) => c,
    }
}

pub unsafe fn frame_copy(m: *const RawMachine, out: *mut u8, len: usize) -> i64 {
    text_on(m as *mut _, Op::FrameCopy, &[], out, len)
}

pub unsafe fn audio_rate(m: *const RawMachine) -> i32 {
    code(on(m as *mut _, Op::AudioRate, false, 0, &[])) as i32
}

pub unsafe fn audio_buffered(m: *const RawMachine) -> i64 {
    code(on(m as *mut _, Op::AudioBuffered, false, 0, &[]))
}

pub unsafe fn audio_drain(m: *mut RawMachine, out: *mut i16, len: usize, max_frames: i64, rate: *mut i32) -> i64 {
    match on(m, Op::AudioDrain, out.is_null(), (len * 2) as u64, &args(|w| {
        w.i64(max_frames);
    })) {
        Ok(r) => {
            if !out.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr(), out as *mut u8, r.bytes.len().min(len * 2)) };
            }
            if !rate.is_null() {
                unsafe { *rate = r.aux as i32 };
            }
            r.ret
        }
        Err(c) => c as i64,
    }
}

pub unsafe fn set_audio_limit(m: *mut RawMachine, samples: u64) -> i32 {
    code(on(m, Op::SetAudioLimit, false, 0, &args(|w| {
        w.u64(samples);
    }))) as i32
}

pub unsafe fn set_buttons(m: *mut RawMachine, port: u32, mask: u32, changed: u32) -> i32 {
    code(on(m, Op::SetButtons, false, 0, &args(|w| {
        w.u32(port).u32(mask).u32(changed);
    }))) as i32
}

pub unsafe fn state_size(m: *const RawMachine, kind: u32) -> i64 {
    code(on(m as *mut _, Op::StateSize, false, 0, &args(|w| {
        w.u32(kind);
    })))
}

pub unsafe fn state_save(m: *mut RawMachine, kind: u32, out: *mut u8, len: usize) -> i64 {
    text_on(m, Op::StateSave, &args(|w| {
        w.u32(kind);
    }), out, len)
}

pub unsafe fn state_load(m: *mut RawMachine, data: *const u8, len: usize) -> i32 {
    let bytes = if data.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    code(on(m, Op::StateLoad, false, 0, &args(|w| {
        w.bytes(bytes);
    }))) as i32
}

pub unsafe fn state_layout(m: *const RawMachine, kind: u32, out: *mut u8, len: usize) -> i64 {
    text_on(m as *mut _, Op::StateLayout, &args(|w| {
        w.u32(kind);
    }), out, len)
}

pub unsafe fn space_size(m: *const RawMachine, space: u32) -> i64 {
    code(on(m as *mut _, Op::SpaceSize, false, 0, &args(|w| {
        w.u32(space);
    })))
}

pub unsafe fn space_read(m: *mut RawMachine, space: u32, address: u32, out: *mut u8, len: usize) -> i64 {
    if let Some(p) = unsafe { proxied(m) }
        && !out.is_null()
        && len > 0
    {
        let buf = unsafe { std::slice::from_raw_parts_mut(out, len) };
        if p.window.read(space, address, buf) {
            return len as i64;
        }
        p.window.want(space, address, len.min(u32::MAX as usize) as u32);
    }
    text_on(m, Op::SpaceRead, &args(|w| {
        w.u32(space).u32(address);
    }), out, len)
}

pub unsafe fn space_write(m: *mut RawMachine, space: u32, address: u32, data: *const u8, len: usize) -> i64 {
    let bytes = if data.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    let r = code(on(m, Op::SpaceWrite, false, 0, &args(|w| {
        w.u32(space).u32(address).bytes(bytes);
    })));
    if r >= 0
        && let Some(p) = unsafe { proxied(m) }
    {
        p.window.wrote(space, address, bytes);
    }
    r
}

pub unsafe fn battery(m: *const RawMachine, which: u32, out: *mut u8, len: usize, flags_out: *mut u32) -> i64 {
    match on(m as *mut _, Op::Battery, out.is_null(), len as u64, &args(|w| {
        w.u32(which);
    })) {
        Ok(r) => {
            if !out.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr(), out, r.bytes.len().min(len)) };
            }
            if !flags_out.is_null() {
                unsafe { *flags_out = r.aux as u32 };
            }
            r.ret
        }
        Err(c) => c as i64,
    }
}

pub unsafe fn battery_saved(m: *mut RawMachine, which: u32) -> i32 {
    code(on(m, Op::BatterySaved, false, 0, &args(|w| {
        w.u32(which);
    }))) as i32
}

pub unsafe fn reset(m: *mut RawMachine) -> i32 {
    code(on(m, Op::Reset, false, 0, &[])) as i32
}

pub unsafe fn phases(m: *const RawMachine, out: *mut i64, len: usize) -> i64 {
    text_on(m as *mut _, Op::Phases, &[], out as *mut u8, len * 8)
}

pub unsafe fn set_axis(m: *mut RawMachine, port: u32, axis: u32, value: f64) -> i32 {
    code(on(m, Op::SetAxis, false, 0, &args(|w| {
        w.u32(port).u32(axis).f64(value);
    }))) as i32
}

pub unsafe fn set_settings(m: *mut RawMachine, text: *const u8, len: usize) -> i32 {
    let bytes = if text.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(text, len) } };
    let (rest, deadline) = split_settings(bytes);
    if let (Some(d), Some(p)) = (deadline, unsafe { proxied(m) }) {
        p.frame = Duration::from_secs(d);
    }
    if rest.iter().all(u8::is_ascii_whitespace) {
        return 0;
    }
    code(on(m, Op::SetSettings, false, 0, &args(|w| {
        w.bytes(&rest);
    }))) as i32
}

pub unsafe fn setting_notes(m: *const RawMachine, out: *mut u8, len: usize) -> i64 {
    text_on(m as *mut _, Op::SettingNotes, &[], out, len)
}

pub unsafe fn media_lid(m: *mut RawMachine, drive: u32, open: u32) -> i32 {
    code(on(m, Op::MediaLid, false, 0, &args(|w| {
        w.u32(drive).u32(open);
    }))) as i32
}

pub unsafe fn media_insert<T: Target>(m: *mut RawMachine, drive: u32, media: *const Media) -> i32 {
    let Some(given) = (unsafe { media.as_ref() }) else { return status::NULL };
    let least = std::mem::offset_of!(Media, image_path);
    if (given.size as usize) < least {
        return status::BAD_STRUCT;
    }
    let mut d: Media = unsafe { std::mem::zeroed() };
    unsafe { std::ptr::copy_nonoverlapping(media as *const u8, &mut d as *mut Media as *mut u8, (given.size as usize).min(size_of::<Media>())) };
    let path = path_of(d.image_path);
    let by_path = path.is_some() && reads_by_path::<T>();
    let image = (!d.image.is_null()).then(|| unsafe { std::slice::from_raw_parts(d.image, d.image_len) });
    code(on(m, Op::MediaInsert, false, 0, &args(|w| {
        w.u32(drive).u32(d.flags & !flags::MEDIA_LENT);
        files_of(w, d.files, d.file_count, d.file_size);
        w.text(if by_path { None } else { image });
        w.text(path.as_deref().map(str::as_bytes));
        w.u32(image.is_none() as u32);
    }))) as i32
}

pub unsafe fn set_present(m: *mut RawMachine, port: u32, present: u32) -> i32 {
    code(on(m, Op::SetPresent, false, 0, &args(|w| {
        w.u32(port).u32(present);
    }))) as i32
}

/// The host's ring (since 1.18): its images and semaphores cross as descriptors by `SCM_RIGHTS`; a null ring withdraws it.
pub unsafe fn ring_lend(m: *mut RawMachine, ring: *const RingLend) -> i32 {
    let Some(r) = (unsafe { ring.as_ref() }) else { return code(on(m, Op::RingLend, false, 0, &args(|w| {
        w.u32(0);
    }))) as i32 };
    if (r.size as usize) < size_of::<RingLend>() || (r.count > 0 && (r.images.is_null() || r.image_size < size_of::<RingImage>())) || r.count as usize + 2 > crate::wire::MAX_RING_FDS {
        return status::BAD_STRUCT;
    }
    let images: Vec<RingImage> = (0..r.count as usize).map(|i| unsafe { std::ptr::read_unaligned((r.images as *const u8).add(i * r.image_size) as *const RingImage) }).collect();
    let handles: Vec<u64> = images.iter().map(|i| i.handle).chain([r.ready, r.done]).collect();
    if handles.iter().any(|&h| h > i32::MAX as u64 || unsafe { libc::fcntl(h as i32, libc::F_GETFD) } < 0) {
        return status::BAD_STRUCT;
    }
    let fds: Vec<i32> = handles.iter().map(|&h| h as i32).collect();
    let Some(p) = (unsafe { proxied(m) }) else { return status::NULL };
    let frame = p.frame;
    match p.link.call_with_fds(Op::RingLend, &crate::wire::ring_args(&images, &r.device_uuid, &r.driver_uuid), &fds, frame) {
        Ok(rep) => rep.ret as i32,
        Err((c, why)) => {
            record(&mut p.log, "error", &why);
            c
        }
    }
}

/// The frame's picture in the ring (since 1.18), as the runner's machine answered it.
pub unsafe fn frame_ring(m: *mut RawMachine, out: *mut RingFrame) -> i32 {
    if out.is_null() {
        return status::NULL;
    }
    if (unsafe { std::ptr::read_unaligned(out as *const u32) } as usize) < size_of::<RingFrame>() {
        return status::BAD_STRUCT;
    }
    match on(m, Op::FrameRing, false, 0, &[]) {
        Ok(r) => {
            if r.ret == 1 && r.bytes.len() == size_of::<RingFrame>() {
                unsafe { std::ptr::copy_nonoverlapping(r.bytes.as_ptr().add(4), (out as *mut u8).add(4), size_of::<RingFrame>() - 4) };
            }
            r.ret as i32
        }
        Err(c) => c,
    }
}

/// The exports of a proxy for target `$t`, its optional groups named as `core_exports!` names them.
#[macro_export]
macro_rules! broker_exports {
    ($t:ty; $($group:ident),* $(,)?) => {
        use $crate::proxy as __p;
        use emusen_native::core::sys::{CreateParams as __Cp, Event as __Ev, FrameInfo as __Fi, Machine as __M, Media as __Md};
        #[unsafe(no_mangle)] pub extern "C" fn emusen_core_abi_version() -> u32 { __p::abi_version::<$t>() }
        #[unsafe(no_mangle)] pub extern "C" fn emusen_core_capabilities() -> u64 { __p::capabilities::<$t>() }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_info(o: *mut u8, l: usize) -> i64 { unsafe { __p::info::<$t>(o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_settings_schema(o: *mut u8, l: usize) -> i64 { unsafe { __p::settings_schema::<$t>(o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_firmware_for(i: *const u8, il: usize, o: *mut u8, l: usize) -> i64 { unsafe { __p::firmware_for::<$t>(i, il, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_status_text(c: i32, o: *mut u8, l: usize) -> i64 { unsafe { __p::status_text::<$t>(c, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_crash_log(p: *const std::ffi::c_char) -> i32 { unsafe { __p::set_crash_log::<$t>(p) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_log_drain(m: *mut __M, o: *mut u8, l: usize) -> i64 { unsafe { __p::log_drain::<$t>(m, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_create(p: *const __Cp, s: *mut i32) -> *mut __M { unsafe { __p::create::<$t>(p, s) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_free(m: *mut __M) -> i32 { unsafe { __p::free::<$t>(m) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_machine_info(m: *const __M, o: *mut u8, l: usize) -> i64 { unsafe { __p::machine_info(m, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_last_error(m: *const __M, o: *mut u8, l: usize) -> i64 { unsafe { __p::last_error(m, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_advance(m: *mut __M, d: *mut u64) -> i32 { unsafe { __p::advance(m, d) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_options(m: *mut __M, f: u32) -> i32 { unsafe { __p::set_options(m, f) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_frame_count(m: *const __M) -> i64 { unsafe { __p::frame_count(m) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_events(m: *mut __M, o: *mut __Ev, c: usize, s: usize) -> i64 { unsafe { __p::events(m, o, c, s) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_frame_info(m: *const __M, o: *mut __Fi) -> i32 { unsafe { __p::frame_info(m, o) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_frame_copy(m: *const __M, o: *mut u8, l: usize) -> i64 { unsafe { __p::frame_copy(m, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_audio_rate(m: *const __M) -> i32 { unsafe { __p::audio_rate(m) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_audio_buffered(m: *const __M) -> i64 { unsafe { __p::audio_buffered(m) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_audio_drain(m: *mut __M, o: *mut i16, l: usize, f: i64, r: *mut i32) -> i64 { unsafe { __p::audio_drain(m, o, l, f, r) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_audio_limit(m: *mut __M, s: u64) -> i32 { unsafe { __p::set_audio_limit(m, s) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_buttons(m: *mut __M, p: u32, k: u32, c: u32) -> i32 { unsafe { __p::set_buttons(m, p, k, c) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_state_size(m: *const __M, k: u32) -> i64 { unsafe { __p::state_size(m, k) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_state_save(m: *mut __M, k: u32, o: *mut u8, l: usize) -> i64 { unsafe { __p::state_save(m, k, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_state_load(m: *mut __M, d: *const u8, l: usize) -> i32 { unsafe { __p::state_load(m, d, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_state_layout(m: *const __M, k: u32, o: *mut u8, l: usize) -> i64 { unsafe { __p::state_layout(m, k, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_space_size(m: *const __M, s: u32) -> i64 { unsafe { __p::space_size(m, s) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_space_read(m: *mut __M, s: u32, a: u32, o: *mut u8, l: usize) -> i64 { unsafe { __p::space_read(m, s, a, o, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_space_write(m: *mut __M, s: u32, a: u32, d: *const u8, l: usize) -> i64 { unsafe { __p::space_write(m, s, a, d, l) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_battery(m: *const __M, w: u32, o: *mut u8, l: usize, f: *mut u32) -> i64 { unsafe { __p::battery(m, w, o, l, f) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_battery_saved(m: *mut __M, w: u32) -> i32 { unsafe { __p::battery_saved(m, w) } }
        $( $crate::broker_group!($t, $group); )*
        const _: () = assert!(<$t as $crate::proxy::Target>::CAPABILITIES & emusen_native::core::EXPORTING == 0 $(| $crate::__group_bit!($group))*, "the groups named and CAPABILITIES disagree");
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __group_bit {
    (reset) => { emusen_native::core::caps::RESET };
    (phases) => { emusen_native::core::caps::PHASES };
    (axes) => { emusen_native::core::caps::AXES };
    (settings) => { emusen_native::core::caps::SETTINGS };
    (setting_notes) => { emusen_native::core::caps::SETTING_NOTES };
    (media) => { emusen_native::core::caps::MEDIA };
    (presence) => { emusen_native::core::caps::PRESENCE };
    (expect) => { emusen_native::core::caps::EXPECT };
    (device_ring) => { emusen_native::core::caps::DEVICE_RING };
}

/// One optional group's exports.
#[macro_export]
#[doc(hidden)]
macro_rules! broker_group {
    ($t:ty, reset) => { #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_reset(m: *mut __M) -> i32 { unsafe { __p::reset(m) } } };
    ($t:ty, phases) => { #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_phases(m: *const __M, o: *mut i64, l: usize) -> i64 { unsafe { __p::phases(m, o, l) } } };
    ($t:ty, axes) => { #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_axis(m: *mut __M, p: u32, a: u32, v: f64) -> i32 { unsafe { __p::set_axis(m, p, a, v) } } };
    ($t:ty, settings) => { #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_settings(m: *mut __M, t: *const u8, l: usize) -> i32 { unsafe { __p::set_settings(m, t, l) } } };
    ($t:ty, setting_notes) => { #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_setting_notes(m: *const __M, o: *mut u8, l: usize) -> i64 { unsafe { __p::setting_notes(m, o, l) } } };
    ($t:ty, media) => {
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_media_lid(m: *mut __M, d: u32, o: u32) -> i32 { unsafe { __p::media_lid(m, d, o) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_media_insert(m: *mut __M, d: u32, x: *const __Md) -> i32 { unsafe { __p::media_insert::<$t>(m, d, x) } }
    };
    ($t:ty, presence) => { #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_set_present(m: *mut __M, p: u32, x: u32) -> i32 { unsafe { __p::set_present(m, p, x) } } };
    ($t:ty, device_ring) => {
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_ring_lend(m: *mut __M, r: *const emusen_native::core::sys::RingLend) -> i32 { unsafe { __p::ring_lend(m, r) } }
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_frame_ring(m: *mut __M, o: *mut emusen_native::core::sys::RingFrame) -> i32 { unsafe { __p::frame_ring(m, o) } }
    };
    ($t:ty, expect) => {
        #[unsafe(no_mangle)] pub unsafe extern "C" fn emusen_core_expect(s: *const std::ffi::c_char) -> i32 { unsafe { __p::expect::<$t>(s) } }
        const _: emusen_native::core::sys::ExpectFn = emusen_core_expect;
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Reads;
    struct Ignores;

    impl Target for Reads {
        const CAPABILITIES: u64 = 0;
        fn spawn(_: Option<&str>) -> Result<Spawn, String> {
            Err(String::new())
        }
    }

    impl Target for Ignores {
        const CAPABILITIES: u64 = 0;
        fn spawn(_: Option<&str>) -> Result<Spawn, String> {
            Err(String::new())
        }
        fn firmware_reads_image() -> bool {
            false
        }
    }

    #[test]
    fn a_core_whose_firmware_does_not_read_the_image_is_sent_none_of_it() {
        let image = vec![7u8; 1 << 20];
        let (sent, empty) = (firmware_request::<Reads>(&image), firmware_request::<Ignores>(&[]));
        assert!(sent.len() > image.len());
        assert_eq!(firmware_request::<Ignores>(&image), empty);
        assert!(empty.len() < 16);
    }
}
