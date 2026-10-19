//! The session service, `emusen.session@1`, served by the runtime over each session's debugger: the C# provider's
//! `SessionService`, its JSON written as `Utf8JsonWriter` writes it and its refusals in its words, so that a module or a
//! frontend gets the same answer from either implementation (EmuSen_Backend.md §29.3, §29.21). The C layer's twins and
//! the table's entries both call these functions; `Call` says which session a call names and whether its table may
//! change it.

pub mod args;
pub mod cheat_file;
pub mod cheats;
pub mod machine;
pub mod registries;
pub mod views;

use std::sync::Mutex;

use emusen_crystal_debug::Debugger;
use emusen_session_service::abi::{PROCESSOR_SESSION, status};

use crate::engine::Engine;
use crate::json::Json;
use crate::runtime::Runtime;
use crate::session::{Refused, Run, Session};

pub use args::Raw;

pub type Answer<T> = Result<T, Refused>;

/// What a provider keeps of its own: run control's requests not yet drained and the last one asked, and the records of
/// changes a short buffer could not yet take.
pub struct Provider {
    requests: Mutex<(Vec<String>, String)>,
    pub(crate) pending: Mutex<Vec<(u32, Vec<u8>)>>,
}

impl Default for Provider {
    fn default() -> Provider {
        Provider { requests: Mutex::new((Vec::new(), "none".to_string())), pending: Mutex::new(Vec::new()) }
    }
}

impl Provider {
    /// `Raise(new SessionEvent("run", op))`: a request kept for `events_drain`, and the last one `halt_state` names.
    fn raise(&self, op: &str) {
        let mut r = self.requests.lock().unwrap_or_else(|p| p.into_inner());
        r.0.push(op.to_string());
        r.1 = op.to_string();
    }

    fn last_request(&self) -> String {
        self.requests.lock().unwrap_or_else(|p| p.into_inner()).1.clone()
    }
}

/// One call of the service: the runtime, the session the table or twin serves, the session the caller named, and
/// whether the table is a read-only context's.
pub struct Call<'a> {
    pub rt: &'a Runtime,
    pub s: &'a Session,
    pub asked: u32,
    pub read_only: bool,
}

/// `SessionException`'s refusals.
pub fn refused(status: i32, words: impl Into<String>) -> Refused {
    (status, words.into())
}

fn not_supported(words: impl Into<String>) -> Refused {
    refused(status::NOT_SUPPORTED, words)
}

/// The words of a change in a read-only context (EmuSen_DianaSh.md §6.5).
pub const READ_ONLY_WORDS: &str = "refused: this is a read-only context, and the call changes the session";

impl Call<'_> {
    /// `Target(session)`: 0 or the session served, else NO_SESSION.
    pub fn target(&self) -> Answer<()> {
        if self.asked != 0 && self.asked != self.s.id {
            return Err(refused(status::NO_SESSION, format!("no session {}; this provider serves session {}", self.asked, self.s.id)));
        }
        Ok(())
    }

    /// A read: the session as it stands, under its gate so the machine is whole.
    pub fn read<T>(&self, f: impl FnOnce(&mut Engine, &Run) -> Answer<T>) -> Answer<T> {
        let mut held = self.s.lock()?;
        let run = self.s.run.lock().unwrap_or_else(|p| p.into_inner());
        f(&mut held.engine, &run)
    }

    /// A call the C#'s export holds the gate around whether it reads or changes: an op-coded group's, which refuses a
    /// read-only table's change itself, after the op is known.
    pub fn gated<T>(&self, f: impl FnOnce(&mut Engine, &mut Run) -> Answer<T>) -> Answer<T> {
        self.s.change(f)
    }

    /// A change: the gate held around it, refused first where the table is a read-only context's, as `GateScope` and
    /// `Binding.Change` order them.
    pub fn change<T>(&self, f: impl FnOnce(&mut Engine, &mut Run) -> Answer<T>) -> Answer<T> {
        let read_only = self.read_only;
        self.s.change(|e, run| if read_only { Err(refused(status::READ_ONLY, READ_ONLY_WORDS)) } else { f(e, run) })
    }
}

/// Which registries a processor argument reaches: the session's own (none named, or processor 0 of a session that
/// names none), or a listed processor's, `ProcessorOf`.
pub fn processor_of(d: &Debugger, core_name: &str, processor: u32) -> Answer<Option<usize>> {
    if processor == PROCESSOR_SESSION {
        return Ok(None);
    }
    let cpus = d.processors();
    if !cpus.is_empty() {
        if processor as usize >= cpus.len() {
            return Err(refused(status::BAD_ARGUMENT, format!("no processor {processor}; this session has {}: {}", cpus.len(), name_list(&cpus))));
        }
        return Ok(Some(processor as usize));
    }
    if processor == 0 {
        return Ok(None);
    }
    Err(refused(status::BAD_ARGUMENT, format!("no processor {processor} on this {core_name}")))
}

/// `DebugCpus.NameList`.
fn name_list(cpus: &[emusen_crystal_debug::debugger::Cpu]) -> String {
    if cpus.is_empty() { "(none)".to_string() } else { cpus.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ") }
}

/// `Space(target, name)`: a listed space by its name in any case, its own spelling answered.
pub fn space(d: &Debugger, m: Option<&emusen_core_host::Machine>, name: Option<&str>) -> Answer<String> {
    let Some(name) = name else { return Err(refused(status::BAD_ARGUMENT, "a memory space must be named")) };
    let spaces = m.map(|m| d.spaces(m)).unwrap_or_default();
    match spaces.iter().find(|s| emusen_dotnet::text::equals_ignore_case(&s.name, name)) {
        Some(s) => Ok(s.name.clone()),
        None => Err(refused(status::NO_SUCH_SPACE, format!("No memory space named '{name}'. Available: {}", spaces.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", ")))),
    }
}

/// `SessionJson.Registers`.
pub fn registers_json(j: &mut Json, registers: &[emusen_crystal_debug::context::Register]) {
    j.begin_array();
    for r in registers {
        j.begin_object().str("name", &r.name).u64("value", r.value).int("bits", r.bits as i64).end_object();
    }
    j.end_array();
}

/// `SessionJson.Int32`: a count or index the session takes as an `int`.
pub fn int32(value: i64, what: &str) -> Answer<i32> {
    i32::try_from(value).map_err(|_| refused(status::BAD_ARGUMENT, format!("{what} {value} is out of range")))
}

/// `NoSuchId(what, id)`.
pub fn no_such_id(what: &str, id: i64) -> Refused {
    refused(status::NO_SUCH_ID, format!("No {what} #{id}."))
}
