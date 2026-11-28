//! The session service, `emusen.session@1`, from a caller's side: the table (`abi`), a client that calls it by the
//! header's rules (`Session`), and what every EmuSen command module shares over it (`words`, `scope`). A module asks
//! the shell for the service by name, hands the table to `Session::from_table`, and formats what comes back in the
//! words the C# commands used. See EmuSen_DianaSh.md §9 and §13.11.

pub mod abi;
pub mod scope;
pub mod space;
pub mod words;

use abi::{emusen_session_v1, status};
use serde_json::Value;
use std::ffi::c_void;

/// A call the session refused: its status, and the words `last_error` gave for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub status: i32,
    pub words: String,
}

impl Refusal {
    pub fn new(status: i32, words: impl Into<String>) -> Refusal {
        Refusal { status, words: words.into() }
    }

    pub fn is(&self, status: i32) -> bool {
        self.status == status
    }
}

/// A refusal read as the exception the C# member threw: its words, which a command prints as `Error: <words>`.
impl From<Refusal> for String {
    fn from(refusal: Refusal) -> String {
        refusal.words
    }
}

pub type Answer<T> = Result<T, Refusal>;

/// The service as one session sees it: the table, and the session it names (0, the host's current one).
#[derive(Clone, Copy)]
pub struct Session<'a> {
    table: *const emusen_session_v1,
    session: u32,
    _marker: std::marker::PhantomData<&'a emusen_session_v1>,
}

/// Text passed in: a pointer and a length, the null pointer for the absent string.
fn text(value: Option<&str>) -> (*const u8, usize) {
    match value {
        Some(text) => (text.as_ptr(), text.len()),
        None => (std::ptr::null(), 0),
    }
}

macro_rules! opt_entry {
    ($self:expr, $f:ident) => {{
        let end = std::mem::offset_of!(emusen_session_v1, $f) + std::mem::size_of::<usize>();
        if $self.table_size() >= end { unsafe { std::ptr::addr_of!((*$self.table).$f).read() } } else { None }
    }};
}

macro_rules! req_entry {
    ($self:expr, $f:ident) => {
        opt_entry!($self, $f).ok_or_else(|| Session::missing(stringify!($f)))?
    };
}

impl<'a> Session<'a> {
    /// A table the shell handed over, if its major is this crate's and it is no smaller than version 1.0's.
    ///
    /// # Safety
    /// `table` must be null or a table that stays valid for `'a`.
    pub unsafe fn from_table(table: *const c_void) -> Option<Session<'a>> {
        if table.is_null() {
            return None;
        }
        let ptr = table as *const emusen_session_v1;
        let min_v10 = std::mem::offset_of!(emusen_session_v1, events_drain) + std::mem::size_of::<usize>();
        let size = unsafe { std::ptr::addr_of!((*ptr).size).read() } as usize;
        if size < min_v10 {
            return None;
        }
        let version = unsafe { std::ptr::addr_of!((*ptr).version).read() };
        (version >> 16 == abi::MAJOR).then_some(Session { table: ptr, session: 0, _marker: std::marker::PhantomData })
    }

    fn table_size(&self) -> usize {
        unsafe { std::ptr::addr_of!((*self.table).size).read() as usize }
    }

    fn ctx(&self) -> *mut c_void {
        unsafe { std::ptr::addr_of!((*self.table).ctx).read() }
    }

    /// The provider's minor version.
    pub fn minor(&self) -> u32 {
        unsafe { std::ptr::addr_of!((*self.table).version).read() & 0xFFFF }
    }

    /// The processor argument that names none, where the provider knows it (1.1); before that, processor 0.
    pub fn no_processor(&self) -> u32 {
        if self.minor() >= 1 { abi::PROCESSOR_SESSION } else { 0 }
    }

    /// The session a module was handed by the shell, or `RequireTarget`'s refusal where the host provides none.
    pub fn from_service(table: Option<*const c_void>) -> Answer<Session<'a>> {
        table.and_then(|t| unsafe { Session::from_table(t) }).ok_or_else(words::no_session)
    }

    /// `CoreName`, as `info` gives it.
    pub fn core_name(&self) -> Answer<String> {
        Ok(self.info()?.text("engine"))
    }

    /// System id, as `info` gives it.
    pub fn system(&self) -> Answer<String> {
        Ok(self.info()?.text("system"))
    }

    /// Native byte order, as `info` gives it.
    pub fn byte_order(&self) -> Answer<space::Endian> {
        Ok(self.info()?.optional_text("byte_order").as_deref().and_then(space::Endian::from_str).unwrap_or(space::Endian::Little))
    }

    /// Whether `info` lists a capability.
    pub fn can(&self, capability: &str) -> Answer<bool> {
        Ok(self.info()?.list("capabilities").iter().any(|c| c.as_str() == Some(capability)))
    }

    fn missing(name: &str) -> Refusal {
        Refusal::new(status::NOT_SUPPORTED, format!("this session service has no {name}"))
    }

    /// The words for the failure just returned, on this thread.
    fn last_error(&self) -> String {
        let Some(last) = opt_entry!(self, last_error) else { return String::new() };
        let ctx = self.ctx();
        let length = unsafe { last(ctx, std::ptr::null_mut(), 0) };
        if length <= 0 {
            return String::new();
        }
        let mut bytes = vec![0u8; length as usize];
        let written = unsafe { last(ctx, bytes.as_mut_ptr(), bytes.len()) };
        bytes.truncate(written.clamp(0, length) as usize);
        String::from_utf8_lossy(&bytes).into_owned()
    }

    fn checked(&self, code: i64) -> Answer<i64> {
        if code < 0 { Err(Refusal::new(code as i32, self.last_error())) } else { Ok(code) }
    }

    /// The length-query idiom: asked once with no buffer, then copied, asked again if the record grew between. A
    /// change answers 0 and copies nothing, so it is never asked twice.
    fn bytes(&self, fill: impl Fn(*mut u8, usize) -> i64) -> Answer<Vec<u8>> {
        let mut length = self.checked(fill(std::ptr::null_mut(), 0))?;
        if length == 0 {
            return Ok(Vec::new());
        }
        loop {
            let mut buffer = vec![0u8; length as usize];
            let total = self.checked(fill(buffer.as_mut_ptr(), buffer.len()))?;
            if total <= length {
                buffer.truncate(total as usize);
                return Ok(buffer);
            }
            length = total;
        }
    }

    fn json(&self, fill: impl Fn(*mut u8, usize) -> i64) -> Answer<Value> {
        let bytes = self.bytes(fill)?;
        let text = String::from_utf8_lossy(&bytes);
        serde_json::from_str::<Value>(&text).map_err(|e| Refusal::new(status::FAULT, format!("the session's answer is not JSON: {e}")))
    }

    fn string(&self, fill: impl Fn(*mut u8, usize) -> i64) -> Answer<String> {
        Ok(String::from_utf8_lossy(&self.bytes(fill)?).into_owned())
    }

    fn done(&self, code: i32) -> Answer<()> {
        self.checked(i64::from(code)).map(|_| ())
    }

    pub fn info(&self) -> Answer<Value> {
        let f = req_entry!(self, info);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn summary(&self) -> Answer<String> {
        let f = req_entry!(self, summary);
        self.string(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn run_control(&self, op: u32, processor: u32, argument: i64) -> Answer<i64> {
        let f = req_entry!(self, run_control);
        self.checked(unsafe { f(self.ctx(), self.session, op, processor, argument) })
    }

    pub fn halt_state(&self, processor: u32) -> Answer<Value> {
        let f = req_entry!(self, halt_state);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, o, l) })
    }

    pub fn spaces(&self) -> Answer<Value> {
        let f = req_entry!(self, spaces);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn space_read(&self, space: &str, address: u32, length: usize) -> Answer<Vec<u8>> {
        let f = req_entry!(self, space_read);
        let mut bytes = vec![0u8; length];
        let read = self.checked(unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, bytes.as_mut_ptr(), bytes.len()) })?;
        bytes.truncate((read as usize).min(length));
        Ok(bytes)
    }

    pub fn space_write(&self, space: &str, address: u32, data: &[u8]) -> Answer<()> {
        let f = req_entry!(self, space_write);
        self.done(unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, data.as_ptr(), data.len()) })
    }

    pub fn resolve_physical(&self, address: u32) -> Answer<Value> {
        let f = req_entry!(self, resolve_physical);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, address, o, l) })
    }

    pub fn processors(&self) -> Answer<Value> {
        let f = req_entry!(self, processors);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn registers(&self, processor: u32) -> Answer<Value> {
        let f = req_entry!(self, registers);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, o, l) })
    }

    pub fn set_register(&self, processor: u32, name: &str, value: u64) -> Answer<()> {
        let f = req_entry!(self, set_register);
        self.done(unsafe { f(self.ctx(), self.session, processor, name.as_ptr(), name.len(), value) })
    }

    pub fn disassemble(&self, processor: u32, space: Option<&str>, address: u32, count: u32, hints: Option<&str>) -> Answer<Value> {
        let f = req_entry!(self, disassemble);
        let (s, sl) = text(space);
        let (h, hl) = text(hints);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, s, sl, address, count, h, hl, o, l) })
    }

    pub fn bp_add(&self, processor: u32, spec: &str) -> Answer<i64> {
        let f = req_entry!(self, bp_add);
        self.checked(unsafe { f(self.ctx(), self.session, processor, spec.as_ptr(), spec.len()) })
    }

    pub fn bp_add_data(&self, processor: u32, spec: &str) -> Answer<i64> {
        let f = req_entry!(self, bp_add_data);
        self.checked(unsafe { f(self.ctx(), self.session, processor, spec.as_ptr(), spec.len()) })
    }

    pub fn bp_set_enabled(&self, processor: u32, id: u32, enabled: bool) -> Answer<()> {
        let f = req_entry!(self, bp_set_enabled);
        self.done(unsafe { f(self.ctx(), self.session, processor, id, u32::from(enabled)) })
    }

    pub fn bp_remove(&self, processor: u32, id: u32) -> Answer<()> {
        let f = req_entry!(self, bp_remove);
        self.done(unsafe { f(self.ctx(), self.session, processor, id) })
    }

    pub fn bp_list(&self, processor: u32) -> Answer<Value> {
        let f = req_entry!(self, bp_list);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, o, l) })
    }

    pub fn bp_log(&self, processor: u32, count: u32) -> Answer<Value> {
        let f = req_entry!(self, bp_log);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, count, o, l) })
    }

    pub fn bp_log_clear(&self, processor: u32) -> Answer<()> {
        let f = req_entry!(self, bp_log_clear);
        self.done(unsafe { f(self.ctx(), self.session, processor) })
    }

    pub fn bp_arm_condition(&self, processor: u32, name: &str, flags: u32) -> Answer<()> {
        let f = req_entry!(self, bp_arm_condition);
        self.done(unsafe { f(self.ctx(), self.session, processor, name.as_ptr(), name.len(), flags) })
    }

    pub fn bp_conditions(&self, processor: u32) -> Answer<Value> {
        let f = req_entry!(self, bp_conditions);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, o, l) })
    }

    pub fn bp_depth_guard(&self, processor: u32, depth: i32) -> Answer<()> {
        let f = req_entry!(self, bp_depth_guard);
        self.done(unsafe { f(self.ctx(), self.session, processor, depth) })
    }

    pub fn bp_forbid_range(&self, processor: u32, address: u32, end: u32) -> Answer<i64> {
        let f = req_entry!(self, bp_forbid_range);
        self.checked(unsafe { f(self.ctx(), self.session, processor, address, end) })
    }

    pub fn bp_uninitialized(&self, processor: u32, space: Option<&str>, size: u32) -> Answer<()> {
        let f = req_entry!(self, bp_uninitialized);
        let (s, sl) = text(space);
        self.done(unsafe { f(self.ctx(), self.session, processor, s, sl, size) })
    }

    pub fn watch_add(&self, space: &str, address: u32, length: u32, kind: u32) -> Answer<i64> {
        let f = req_entry!(self, watch_add);
        self.checked(unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, length, kind) })
    }

    pub fn watch_remove(&self, id: u32) -> Answer<()> {
        let f = req_entry!(self, watch_remove);
        self.done(unsafe { f(self.ctx(), self.session, id) })
    }

    pub fn watch_list(&self) -> Answer<Value> {
        let f = req_entry!(self, watch_list);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn watch_events(&self, id: u32, count: u32) -> Answer<Value> {
        let f = req_entry!(self, watch_events);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, id, count, o, l) })
    }

    pub fn watch_summary(&self, id: u32) -> Answer<Value> {
        let f = req_entry!(self, watch_summary);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, id, o, l) })
    }

    pub fn watch_clear(&self, id: u32) -> Answer<()> {
        let f = req_entry!(self, watch_clear);
        self.done(unsafe { f(self.ctx(), self.session, id) })
    }

    pub fn framelog_add(&self, space: &str, address: u32, width: u32) -> Answer<i64> {
        let f = req_entry!(self, framelog_add);
        self.checked(unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, width) })
    }

    pub fn framelog_remove(&self, id: u32) -> Answer<()> {
        let f = req_entry!(self, framelog_remove);
        self.done(unsafe { f(self.ctx(), self.session, id) })
    }

    pub fn framelog_list(&self) -> Answer<Value> {
        let f = req_entry!(self, framelog_list);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn framelog_samples(&self, id: u32, count: u32) -> Answer<Value> {
        let f = req_entry!(self, framelog_samples);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, id, count, o, l) })
    }

    pub fn framelog_clear(&self, id: u32) -> Answer<()> {
        let f = req_entry!(self, framelog_clear);
        self.done(unsafe { f(self.ctx(), self.session, id) })
    }

    /// `coverage_op` with JSON arguments, its answer as raw bytes (JSON, a map, or nothing for a change).
    pub fn coverage_op(&self, processor: u32, op: u32, args: &[u8]) -> Answer<Vec<u8>> {
        let f = req_entry!(self, coverage_op);
        self.bytes(|o, l| unsafe { f(self.ctx(), self.session, processor, op, args.as_ptr(), args.len(), o, l) })
    }

    pub fn profile_op(&self, processor: u32, op: u32, args: &str) -> Answer<Vec<u8>> {
        let f = req_entry!(self, profile_op);
        self.bytes(|o, l| unsafe { f(self.ctx(), self.session, processor, op, args.as_ptr(), args.len(), o, l) })
    }

    pub fn counters_op(&self, op: u32, args: &str) -> Answer<Vec<u8>> {
        let f = req_entry!(self, counters_op);
        self.bytes(|o, l| unsafe { f(self.ctx(), self.session, op, args.as_ptr(), args.len(), o, l) })
    }

    pub fn call_stack(&self, processor: u32, count: u32) -> Answer<Value> {
        let f = req_entry!(self, call_stack);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, count, o, l) })
    }

    pub fn freeze_add(&self, space: &str, address: u32, value: u32) -> Answer<i64> {
        let f = req_entry!(self, freeze_add);
        self.checked(unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, value) })
    }

    pub fn freeze_add_wide(&self, space: &str, address: u32, value: u64, width: u32, big_endian: bool) -> Answer<i64> {
        if let Some(f) = opt_entry!(self, freeze_add_wide) {
            return self.checked(unsafe {
                f(self.ctx(), self.session, space.as_ptr(), space.len(), address, value, width, u32::from(big_endian))
            });
        }
        let max = match width {
            1 => 0xFF,
            2 => 0xFFFF,
            4 => 0xFFFF_FFFF,
            _ => return Err(Refusal::new(status::BAD_ARGUMENT, "freeze width must be 1, 2, or 4")),
        };
        if value > max {
            return Err(Refusal::new(status::BAD_ARGUMENT, "freeze value exceeds width"));
        }
        let mut ids = Vec::with_capacity(width as usize);
        for i in 0..width {
            let shift = if big_endian { 8 * (width - 1 - i) } else { 8 * i };
            let byte = ((value >> shift) & 0xFF) as u32;
            match self.freeze_add(space, address.wrapping_add(i), byte) {
                Ok(id) => ids.push(id),
                Err(err) => {
                    for prev in ids {
                        drop(self.freeze_remove(prev as u32));
                    }
                    return Err(err);
                }
            }
        }
        Ok(ids[0])
    }

    pub fn freeze_remove(&self, id: u32) -> Answer<()> {
        let f = req_entry!(self, freeze_remove);
        self.done(unsafe { f(self.ctx(), self.session, id) })
    }

    pub fn freeze_list(&self) -> Answer<Value> {
        let f = req_entry!(self, freeze_list);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn freeze_clear(&self) -> Answer<()> {
        let f = req_entry!(self, freeze_clear);
        self.done(unsafe { f(self.ctx(), self.session) })
    }

    pub fn label_add(&self, address: u32, name: &str, comment: Option<&str>) -> Answer<()> {
        let f = req_entry!(self, label_add);
        let (c, cl) = text(comment);
        self.done(unsafe { f(self.ctx(), self.session, address, name.as_ptr(), name.len(), c, cl) })
    }

    pub fn label_remove(&self, name: Option<&str>, address: u32) -> Answer<()> {
        let f = req_entry!(self, label_remove);
        let (n, nl) = text(name);
        self.done(unsafe { f(self.ctx(), self.session, n, nl, address) })
    }

    pub fn label_list(&self) -> Answer<Value> {
        let f = req_entry!(self, label_list);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn label_at(&self, address: u32) -> Answer<Value> {
        let f = req_entry!(self, label_at);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, address, o, l) })
    }

    pub fn label_load(&self, text: &str) -> Answer<Value> {
        let f = req_entry!(self, label_load);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, text.as_ptr(), text.len(), o, l) })
    }

    pub fn label_save(&self) -> Answer<String> {
        let f = req_entry!(self, label_save);
        self.string(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn label_clear(&self) -> Answer<()> {
        let f = req_entry!(self, label_clear);
        self.done(unsafe { f(self.ctx(), self.session) })
    }

    pub fn dma_channels(&self) -> Answer<Value> {
        let f = req_entry!(self, dma_channels);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn dma_log_op(&self, op: u32, args: &str) -> Answer<Vec<u8>> {
        let f = req_entry!(self, dma_log_op);
        self.bytes(|o, l| unsafe { f(self.ctx(), self.session, op, args.as_ptr(), args.len(), o, l) })
    }

    pub fn register_flow_op(&self, op: u32, args: &str) -> Answer<Vec<u8>> {
        let f = req_entry!(self, register_flow_op);
        self.bytes(|o, l| unsafe { f(self.ctx(), self.session, op, args.as_ptr(), args.len(), o, l) })
    }

    pub fn coprocessor_history(&self, count: u32) -> Answer<Value> {
        let f = req_entry!(self, coprocessor_history);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, count, o, l) })
    }

    pub fn vectors(&self) -> Answer<Value> {
        let f = req_entry!(self, vectors);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn tilemap_entry(&self, space: &str, address: u32) -> Answer<Value> {
        let f = req_entry!(self, tilemap_entry);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, o, l) })
    }

    pub fn tile_pixels(&self, space: &str, address: u32, bpp: u32) -> Answer<Vec<u8>> {
        let f = req_entry!(self, tile_pixels);
        self.bytes(|o, l| unsafe { f(self.ctx(), self.session, space.as_ptr(), space.len(), address, bpp, o, l) })
    }

    pub fn sprites(&self) -> Answer<Value> {
        let f = req_entry!(self, sprites);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn palettes(&self) -> Answer<Value> {
        let f = req_entry!(self, palettes);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn audio_channels(&self) -> Answer<Value> {
        let f = req_entry!(self, audio_channels);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn set_channel_muted(&self, channel: u32, muted: bool) -> Answer<()> {
        let f = req_entry!(self, set_channel_muted);
        self.done(unsafe { f(self.ctx(), self.session, channel, u32::from(muted)) })
    }

    pub fn evaluate(&self, processor: u32, expression: &str) -> Answer<Value> {
        let f = req_entry!(self, evaluate);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, expression.as_ptr(), expression.len(), o, l) })
    }

    pub fn symbols(&self, processor: u32, filter: Option<&str>) -> Answer<Value> {
        let f = req_entry!(self, symbols);
        let (t, tl) = text(filter);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, processor, t, tl, o, l) })
    }

    /// `cheat_op`: a read's record, or a change's id or count as the value, with no record.
    pub fn cheat_op(&self, op: u32, args: &[u8]) -> Answer<(Vec<u8>, i64)> {
        let f = req_entry!(self, cheat_op);
        let first = self.checked(unsafe { f(self.ctx(), self.session, op, args.as_ptr(), args.len(), std::ptr::null_mut(), 0) })?;
        if !reads_a_record(op, args) {
            return Ok((Vec::new(), first));
        }
        let mut length = first;
        loop {
            let mut buffer = vec![0u8; length as usize];
            let total = self.checked(unsafe { f(self.ctx(), self.session, op, args.as_ptr(), args.len(), buffer.as_mut_ptr(), buffer.len()) })?;
            if total <= length {
                buffer.truncate(total as usize);
                return Ok((buffer, total));
            }
            length = total;
        }
    }

    pub fn cheat_codecs(&self) -> Answer<Value> {
        let f = req_entry!(self, cheat_codecs);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn trace(&self, processor: u32, count: u32) -> Answer<()> {
        let f = req_entry!(self, trace);
        self.done(unsafe { f(self.ctx(), self.session, processor, count) })
    }

    pub fn trace_off(&self, processor: u32) -> Answer<()> {
        let f = req_entry!(self, trace_off);
        self.done(unsafe { f(self.ctx(), self.session, processor) })
    }

    pub fn telemetry(&self, kind: u32) -> Answer<Value> {
        let f = req_entry!(self, telemetry);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, kind, o, l) })
    }

    pub fn state_save(&self, path: Option<&str>) -> Answer<()> {
        let f = req_entry!(self, state_save);
        let (p, pl) = text(path);
        self.done(unsafe { f(self.ctx(), self.session, p, pl) })
    }

    pub fn state_load(&self, path: Option<&str>) -> Answer<()> {
        let f = req_entry!(self, state_load);
        let (p, pl) = text(path);
        self.done(unsafe { f(self.ctx(), self.session, p, pl) })
    }

    pub fn state_default_path(&self) -> Answer<String> {
        let f = req_entry!(self, state_default_path);
        self.string(|o, l| unsafe { f(self.ctx(), self.session, o, l) })
    }

    pub fn log_switch(&self, on: bool) -> Answer<()> {
        let f = req_entry!(self, log_switch);
        self.done(unsafe { f(self.ctx(), u32::from(on)) })
    }

    pub fn log_status(&self) -> Answer<bool> {
        let f = req_entry!(self, log_status);
        let code = unsafe { f(self.ctx()) };
        self.checked(i64::from(code)).map(|on| on != 0)
    }

    pub fn events_drain(&self, kinds: u32) -> Answer<Value> {
        let f = req_entry!(self, events_drain);
        self.json(|o, l| unsafe { f(self.ctx(), self.session, kinds, o, l) })
    }
}

/// Whether a cheat op answers with a record: the reads, and the master switch asked rather than set.
fn reads_a_record(op: u32, args: &[u8]) -> bool {
    use abi::cheat::*;
    match op {
        LIST | EXPORT | DECODE | DB_SYSTEMS | DB_GAMES | DB_FIND | CLAIMS | CHT_PARSE | CHT_EXPORT | FILES | FILE_READ | DB_STATUS | DB_PARSE | DB_PRUNE | DB_UPDATE => true,
        MASTER => !String::from_utf8_lossy(args).contains("\"enabled\""),
        _ => false,
    }
}

/// Reading the records the service answers with: a missing or null member reads as zero, false or empty, as the
/// C# member it stands for would have read.
pub trait Record {
    fn field(&self, key: &str) -> Option<&Value>;
    fn int(&self, key: &str) -> i64;
    fn float(&self, key: &str) -> f64;
    fn flag(&self, key: &str) -> bool;
    fn text(&self, key: &str) -> String;
    fn optional_text(&self, key: &str) -> Option<String>;
    fn optional_int(&self, key: &str) -> Option<i64>;
    fn items(&self) -> &[Value];
    fn list(&self, key: &str) -> &[Value];
    /// An address, which the service writes unsigned and the C# held as a signed 32-bit `int`.
    fn address(&self, key: &str) -> i32 {
        self.int(key) as u32 as i32
    }
}

/// A JSON number as an integer; a fraction is cut toward zero.
pub fn number(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64().or_else(|| number.as_f64().map(|f| f as i64)),
        _ => None,
    }
}

impl Record for Value {
    fn field(&self, key: &str) -> Option<&Value> {
        self.get(key).filter(|v| !v.is_null())
    }

    fn int(&self, key: &str) -> i64 {
        self.optional_int(key).unwrap_or(0)
    }

    fn float(&self, key: &str) -> f64 {
        match self.field(key) {
            Some(Value::Number(number)) => number.as_f64().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn flag(&self, key: &str) -> bool {
        self.field(key).and_then(Value::as_bool).unwrap_or(false)
    }

    fn text(&self, key: &str) -> String {
        self.optional_text(key).unwrap_or_default()
    }

    fn optional_text(&self, key: &str) -> Option<String> {
        self.field(key).and_then(Value::as_str).map(str::to_string)
    }

    fn optional_int(&self, key: &str) -> Option<i64> {
        self.field(key).and_then(number)
    }

    fn items(&self) -> &[Value] {
        match self {
            Value::Array(items) => items,
            _ => &[],
        }
    }

    fn list(&self, key: &str) -> &[Value] {
        self.field(key).map_or(&[], |v| v.items())
    }
}

/// A JSON object written from members already rendered as JSON values.
pub fn object(members: &[(&str, String)]) -> String {
    let body: Vec<String> = members.iter().map(|(key, value)| format!("{}:{value}", quoted(key))).collect();
    format!("{{{}}}", body.join(","))
}

/// A string as a JSON value, written by serde_json.
pub fn quoted(text: &str) -> String {
    serde_json::to_string(&text).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockCtx {
        single_calls: RefCell<Vec<(String, u32, u32)>>,
        wide_calls: RefCell<Vec<(String, u32, u64, u32, u32)>>,
        removed_ids: RefCell<Vec<u32>>,
        fail_on_call: usize,
    }

    unsafe extern "C" fn mock_freeze_add(ctx: *mut c_void, _s: u32, sp: *const u8, sp_len: usize, addr: u32, val: u32) -> i64 {
        let c = unsafe { &*(ctx as *const MockCtx) };
        let space = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(sp, sp_len)) }.to_string();
        let mut calls = c.single_calls.borrow_mut();
        if c.fail_on_call != 0 && calls.len() + 1 == c.fail_on_call {
            return abi::status::FAULT as i64;
        }
        calls.push((space, addr, val));
        calls.len() as i64
    }

    unsafe extern "C" fn mock_freeze_remove(ctx: *mut c_void, _s: u32, id: u32) -> i32 {
        let c = unsafe { &*(ctx as *const MockCtx) };
        c.removed_ids.borrow_mut().push(id);
        0
    }

    unsafe extern "C" fn mock_freeze_add_wide(
        ctx: *mut c_void,
        _s: u32,
        sp: *const u8,
        sp_len: usize,
        addr: u32,
        val: u64,
        w: u32,
        be: u32,
    ) -> i64 {
        let c = unsafe { &*(ctx as *const MockCtx) };
        let space = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(sp, sp_len)) }.to_string();
        let mut calls = c.wide_calls.borrow_mut();
        calls.push((space, addr, val, w, be));
        99
    }

    fn empty_table(ctx: *mut MockCtx, size: u32, version: u32) -> emusen_session_v1 {
        emusen_session_v1 {
            size,
            version,
            ctx: ctx as *mut c_void,
            freeze_add: Some(mock_freeze_add),
            freeze_remove: Some(mock_freeze_remove),
            ..abi::blank_table()
        }
    }

    #[test]
    fn freeze_add_wide_uses_v14_or_decomposes_on_v13() {
        let mut ctx = MockCtx::default();
        let mut t14 = empty_table(&mut ctx, std::mem::size_of::<emusen_session_v1>() as u32, abi::VERSION);
        t14.freeze_add_wide = Some(mock_freeze_add_wide);
        let s14 = unsafe { Session::from_table((&t14 as *const emusen_session_v1).cast()) }.unwrap();
        assert_eq!(s14.freeze_add_wide("RDRAM", 0x100, 0x1234_5678, 4, true).unwrap(), 99);
        assert_eq!(ctx.wide_calls.borrow().as_slice(), &[("RDRAM".to_string(), 0x100, 0x1234_5678, 4, 1)]);

        let v13_size = std::mem::offset_of!(emusen_session_v1, freeze_add_wide) as u32;
        let t13 = empty_table(&mut ctx, v13_size, (1 << 16) | 3);
        let s13 = unsafe { Session::from_table((&t13 as *const emusen_session_v1).cast()) }.unwrap();
        assert!(s13.freeze_add_wide("WRAM", 0x20, 0x1_0000, 2, false).unwrap_err().is(abi::status::BAD_ARGUMENT));
        assert!(ctx.single_calls.borrow().is_empty());
        assert_eq!(s13.freeze_add_wide("WRAM", 0x20, 0xABCD, 2, false).unwrap(), 1);
        assert_eq!(s13.freeze_add_wide("RDRAM", 0x40, 0x1122_3344, 4, true).unwrap(), 3);
        assert_eq!(
            ctx.single_calls.borrow().as_slice(),
            &[
                ("WRAM".to_string(), 0x20, 0xCD),
                ("WRAM".to_string(), 0x21, 0xAB),
                ("RDRAM".to_string(), 0x40, 0x11),
                ("RDRAM".to_string(), 0x41, 0x22),
                ("RDRAM".to_string(), 0x42, 0x33),
                ("RDRAM".to_string(), 0x43, 0x44),
            ]
        );

        let mut fail_ctx = MockCtx { fail_on_call: 2, ..MockCtx::default() };
        let t13_fail = empty_table(&mut fail_ctx, v13_size, (1 << 16) | 3);
        let s13_fail = unsafe { Session::from_table((&t13_fail as *const emusen_session_v1).cast()) }.unwrap();
        assert!(s13_fail.freeze_add_wide("WRAM", 0x20, 0xABCD, 2, false).unwrap_err().is(abi::status::FAULT));
        assert_eq!(fail_ctx.removed_ids.borrow().as_slice(), &[1]);
    }
}
