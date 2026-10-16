//! The processors a scoped command can be aimed at, and the scope words that name them: Crystal's `DebugCpus` read
//! from `processors`. See `man cpus`.

use crate::{Answer, Record, Session};
use emusen_dotnet::json::Value;
use emusen_dotnet::text::equals_ignore_case;

/// One processor as `processors` lists it.
#[derive(Clone, Debug, Default)]
pub struct Cpu {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub can_halt: bool,
    pub code_space: Option<String>,
    pub coverage: bool,
    pub call_stack: bool,
    pub registers: bool,
    pub writable_registers: bool,
    /// Where it is executing, where the core reports it.
    pub pc: Option<i32>,
}

/// `cpu` is the main processor, the first listed, whatever the core names it.
pub const MAIN_NAME: &str = "cpu";

impl Cpu {
    fn read(value: &Value) -> Cpu {
        Cpu {
            index: value.int("index") as u32,
            name: value.text("name"),
            description: value.text("description"),
            can_halt: value.flag("can_halt"),
            code_space: value.optional_text("code_space"),
            coverage: value.flag("coverage"),
            call_stack: value.flag("call_stack"),
            registers: value.flag("registers"),
            writable_registers: value.flag("writable_registers"),
            pc: value.optional_int("pc").map(|pc| pc as u32 as i32),
        }
    }
}

/// The session's processors, in its order; empty for a session that names none.
pub fn cpus(session: &Session) -> Answer<Vec<Cpu>> {
    Ok(session.processors()?.items().iter().map(Cpu::read).collect())
}

/// `DebugCpus.Find`: by name ignoring case, or `cpu` for the first.
pub fn find<'a>(cpus: &'a [Cpu], name: &str) -> Option<&'a Cpu> {
    cpus.iter().find(|c| equals_ignore_case(&c.name, name)).or_else(|| (!cpus.is_empty() && equals_ignore_case(name, MAIN_NAME)).then(|| &cpus[0]))
}

/// `DebugCpus.IsMain`.
pub fn is_main(cpus: &[Cpu], cpu: Option<&Cpu>) -> bool {
    cpu.is_some_and(|cpu| !cpus.is_empty() && equals_ignore_case(&cpu.name, &cpus[0].name))
}

/// `DebugCpus.IsScopeWord`: a processor's name, `cpu`, or `cop`, which predates chips having names.
pub fn is_scope_word(cpus: &[Cpu], word: &str) -> bool {
    find(cpus, word).is_some() || equals_ignore_case(word, "cop")
}

/// `DebugCpus.Coprocessor`: what a bare `cop` means, the first after the main one that is not the sound chip.
pub fn coprocessor(cpus: &[Cpu]) -> Option<&Cpu> {
    cpus.iter().skip(1).find(|c| !equals_ignore_case(&c.name, "spc"))
}

/// `DebugCpus.NameList`.
pub fn name_list(cpus: &[Cpu]) -> String {
    if cpus.is_empty() { "(none)".to_string() } else { cpus.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ") }
}

/// `DebugCommandHelpers.ResolveCpu`: the processor a scoped command is aimed at, and where its own words start.
pub fn resolve<'a>(cpus: &'a [Cpu], parts: &[String], at: usize) -> (Option<&'a Cpu>, usize) {
    if cpus.is_empty() || at >= parts.len() {
        return (None, at);
    }
    if let Some(named) = find(cpus, &parts[at]) {
        return (Some(named), at + 1);
    }
    if equals_ignore_case(&parts[at], "cop") {
        return (coprocessor(cpus), at + 1);
    }
    (Some(&cpus[0]), at)
}

/// The processor argument for a resolved scope: its own index, or the session's own registries where none is named.
pub fn index(session: &Session, cpu: Option<&Cpu>) -> u32 {
    cpu.map_or_else(|| session.no_processor(), |c| c.index)
}
