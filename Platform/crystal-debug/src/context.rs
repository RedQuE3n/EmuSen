//! The contexts an expression is evaluated in, the C# `DebugCpuExpressionContext`, `DebugTargetExpressionContext`,
//! `UnitExpressionContext` and `BreakpointExpressions.Live` (`man eval`, EmuSen_Backend.md §24 and §26.7). Each reads
//! what it names through a [`Source`], which the debugger implements over its machine and a test over plain data.

use crate::expr::Context;
use emusen_dotnet::text::{compare_ignore_case, equals_ignore_case, fold, is_letter, is_letter_or_digit, to_lower};
use std::collections::{HashMap, HashSet};

/// One register as a snapshot shows it, `DebugRegisterValue`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Register {
    pub name: String,
    pub value: u64,
    pub bits: i32,
}

/// A set of registers a context reads: a processor's by its index in machine info, or the rows the zero-per-core
/// context adds (the video unit's, the sound and timer units', the coprocessor's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    Processor(usize),
    Video,
    Apu,
    Coprocessor,
}

/// What a context reads, as the debugger shows it at this moment.
pub trait Source {
    /// The machine's frame count, `FrameCount`.
    fn frame(&mut self) -> i64;
    fn label(&self, name: &str) -> Option<i32>;
    /// Every label's name, in address order.
    fn label_names(&self) -> Vec<String>;
    fn space_count(&self) -> usize;
    fn space_name(&self, index: usize) -> &str;
    /// A space's size as the debugger reports it, at most `i32::MAX`.
    fn space_size(&mut self, index: usize) -> i32;
    /// One byte of a space, at an address within its size.
    fn space_read(&mut self, index: usize, address: i32) -> u8;
    /// A set's registers as its snapshot shows them, `Current`.
    fn registers(&mut self, set: Set) -> Vec<Register>;
    /// The first register of a set with the name, ignoring case, without building the set.
    fn register(&mut self, set: Set, name: &str) -> Option<u64>;
    /// A unit's registers read live into `into`; how many it has, `into` written only when they fit.
    fn unit_registers(&mut self, unit: u32, into: &mut [i64]) -> usize;
    /// The console pack's symbol over the main processor's registers, `ISystemDebugViews.Symbol`.
    fn pack_symbol(&mut self, name: &str) -> Option<i64>;
    /// The call stack's depth where the main processor keeps one.
    fn stack_depth(&self) -> Option<i64>;
    /// `DebugExpressions.RefreshRegisters`: every set a context reads, without the views.
    fn refresh_registers(&mut self);
}

/// `IsIdentifier` of `UnitSymbols`, the evaluator's identifier: a letter, '_' or '.', then letters, digits, '_' and '.'.
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if is_letter(c) || c == '_' || c == '.') && s.chars().all(|c| is_letter_or_digit(c) || c == '_' || c == '.')
}

/// Names distinct ignoring case, the first kept, then ordered ignoring case (a stable sort), as LINQ's `Distinct` and
/// `OrderBy` with `OrdinalIgnoreCase` make them.
pub fn distinct_sorted(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out: Vec<String> = names.into_iter().filter(|n| seen.insert(fold(n))).collect();
    out.sort_by(|a, b| compare_ignore_case(a, b));
    out
}

/// One unit as machine info lists it, for the unit symbols.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitInfo {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub registers: Vec<(String, i32)>,
}

/// One machine info's unit registers by spelling, `UnitSymbols`: "<unit>.<register>" for each, and the register alone
/// where only one unit has it; a spelling the scope answers is left out, as is one the language cannot read.
#[derive(Clone, Debug, Default)]
pub struct UnitSymbols {
    by_name: HashMap<String, (u32, usize, usize)>,
    /// The qualified spellings, which `eval symbols` lists, in the order the table first met them.
    pub names: Vec<String>,
}

impl UnitSymbols {
    pub fn new(units: &[UnitInfo], taken: &[String]) -> UnitSymbols {
        let scope: HashSet<String> = taken.iter().map(|t| fold(t)).collect();
        // Insertion order kept, as a .NET dictionary enumerates entries never removed.
        let mut qualified: Vec<(String, Option<(u32, usize, usize)>)> = Vec::new();
        let mut bare: Vec<(String, Option<(u32, usize, usize)>)> = Vec::new();
        fn note(into: &mut Vec<(String, Option<(u32, usize, usize)>)>, spelling: String, at: (u32, usize, usize)) {
            if !is_identifier(&spelling) {
                return;
            }
            match into.iter_mut().find(|(n, _)| equals_ignore_case(n, &spelling)) {
                Some(entry) => entry.1 = None,
                None => into.push((spelling, Some(at))),
            }
        }
        for unit in units {
            let prefix = to_lower(&unit.name.chars().filter(|&c| is_letter_or_digit(c) || c == '_').collect::<String>());
            for (i, (register, _)) in unit.registers.iter().enumerate() {
                let at = (unit.id, i, unit.registers.len());
                if !prefix.is_empty() {
                    note(&mut qualified, format!("{prefix}.{register}"), at);
                }
                note(&mut bare, register.clone(), at);
            }
        }
        let mut table = UnitSymbols::default();
        for (name, at) in qualified {
            let Some(found) = at else { continue };
            if scope.contains(&fold(&name)) {
                continue;
            }
            table.by_name.insert(fold(&name), found);
            table.names.push(name);
        }
        for (name, at) in bare {
            if let Some(found) = at {
                if !scope.contains(&fold(&name)) {
                    table.by_name.entry(fold(&name)).or_insert(found);
                }
            }
        }
        table
    }

    /// The unit's id, the register's index and the unit's register count.
    pub fn find(&self, name: &str) -> Option<(u32, usize, usize)> {
        if self.by_name.is_empty() {
            return None;
        }
        self.by_name.get(&fold(name)).copied()
    }
}

/// `TryReadMemory` of the register contexts: `space` by name, else `fallback`'s, else the first space; every byte within
/// the space's size.
fn read_memory(source: &mut dyn Source, space: Option<&str>, fallback: Option<&str>, address: i32, width: i32) -> Option<i64> {
    let count = source.space_count();
    if count == 0 || address < 0 {
        return None;
    }
    let index = match space.or(fallback) {
        None => 0,
        Some(wanted) => (0..count).find(|&i| equals_ignore_case(source.space_name(i), wanted))?,
    };
    let mut value = 0i64;
    for i in 0..width {
        let at = address.checked_add(i)?;
        if at < 0 || at >= source.space_size(index) {
            return None;
        }
        value |= (source.space_read(index, at) as i64) << (8 * i);
    }
    Some(value)
}

/// What a processor's context answers besides its registers: the pack's symbols and `stackdepth` for the main one.
pub struct Extra<'a> {
    pub names: &'a [String],
    /// Whether the context asks the pack and the call stack, the main processor's alone.
    pub main: bool,
}

/// `DebugCpuExpressionContext`: a processor's registers, then the extra symbols, `frame` first of all and labels last;
/// an unnamed read goes to its own space.
pub struct CpuContext<'s, 'a> {
    pub source: &'s mut dyn Source,
    pub set: Set,
    pub default_space: Option<&'a str>,
    pub extra: Extra<'a>,
}

impl Context for CpuContext<'_, '_> {
    fn symbol(&mut self, name: &str) -> Option<i64> {
        if equals_ignore_case(name, "frame") {
            return Some(self.source.frame());
        }
        if let Some(v) = self.source.register(self.set, name) {
            return Some(v as i64);
        }
        if self.extra.main {
            // stackdepth where the core keeps a call stack, else the pack's symbol over the registers.
            let depth = if equals_ignore_case(name, "stackdepth") { self.source.stack_depth() } else { None };
            if let Some(v) = depth.or_else(|| self.source.pack_symbol(name)) {
                return Some(v);
            }
        }
        self.source.label(name).map(i64::from)
    }

    fn read_memory(&mut self, space: Option<&str>, address: i32, width: i32) -> Option<i64> {
        read_memory(self.source, space, self.default_space, address, width)
    }

    fn symbol_names(&mut self) -> Vec<String> {
        let mut names: Vec<String> = self.source.registers(self.set).into_iter().map(|r| r.name).collect();
        names.extend(self.extra.names.iter().cloned());
        names.push("frame".to_string());
        names.extend(self.source.label_names());
        distinct_sorted(names)
    }
}

/// `DebugTargetExpressionContext`, the zero-per-core context of a machine that names no processor: the video, sound
/// and coprocessor rows as symbols, `frame`, labels, and the first space.
pub struct TargetContext<'s> {
    pub source: &'s mut dyn Source,
}

const TARGET_SETS: [Set; 4] = [Set::Processor(0), Set::Video, Set::Apu, Set::Coprocessor];

impl Context for TargetContext<'_> {
    fn symbol(&mut self, name: &str) -> Option<i64> {
        if equals_ignore_case(name, "frame") {
            return Some(self.source.frame());
        }
        for set in TARGET_SETS {
            if let Some(v) = self.source.register(set, name) {
                return Some(v as i64);
            }
        }
        self.source.label(name).map(i64::from)
    }

    fn read_memory(&mut self, space: Option<&str>, address: i32, width: i32) -> Option<i64> {
        read_memory(self.source, space, None, address, width)
    }

    fn symbol_names(&mut self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for set in TARGET_SETS {
            names.extend(self.source.registers(set).into_iter().map(|r| r.name));
        }
        names.push("frame".to_string());
        names.extend(self.source.label_names());
        distinct_sorted(names)
    }
}

/// `UnitExpressionContext`: a scope's own symbols, then the machine's unit registers read live; a unit's spelling is
/// looked up first so that it never pays for the scope's miss.
pub struct UnitContext<'u, C: Context> {
    pub scope: C,
    pub units: &'u UnitSymbols,
}

/// The source a scope reads, which the unit context reads too.
pub trait Sourced {
    fn source(&mut self) -> &mut dyn Source;
}

impl Sourced for CpuContext<'_, '_> {
    fn source(&mut self) -> &mut dyn Source {
        self.source
    }
}

impl Sourced for TargetContext<'_> {
    fn source(&mut self) -> &mut dyn Source {
        self.source
    }
}

impl<C: Context + Sourced> Context for UnitContext<'_, C> {
    fn symbol(&mut self, name: &str) -> Option<i64> {
        let found = self.units.find(name);
        let Some((unit, index, count)) = found.filter(|_| self.scope.source().label(name).is_none()) else { return self.scope.symbol(name) };
        let mut stack = [0i64; 256];
        let mut heap;
        let values: &mut [i64] = if count <= 256 {
            &mut stack[..count]
        } else {
            heap = vec![0i64; count];
            &mut heap
        };
        let n = self.scope.source().unit_registers(unit, values);
        if index >= n || n > values.len() {
            return None;
        }
        Some(values[index])
    }

    fn read_memory(&mut self, space: Option<&str>, address: i32, width: i32) -> Option<i64> {
        self.scope.read_memory(space, address, width)
    }

    fn symbol_names(&mut self) -> Vec<String> {
        let own = self.scope.symbol_names();
        if self.units.names.is_empty() {
            return own;
        }
        let units: Vec<String> = self.units.names.iter().filter(|n| self.scope.source().label(n).is_none()).cloned().collect();
        distinct_sorted(own.into_iter().chain(units))
    }
}

/// `BreakpointExpressions.Live`: a scope whose registers (not its views) are refreshed once an evaluation first names
/// a symbol, so a condition reads them as they stand at the hit.
pub struct Live<C: Context + Sourced> {
    pub scope: C,
    pub fresh: bool,
}

impl<C: Context + Sourced> Context for Live<C> {
    fn symbol(&mut self, name: &str) -> Option<i64> {
        if !self.fresh {
            self.fresh = true;
            self.scope.source().refresh_registers();
        }
        self.scope.symbol(name)
    }

    fn read_memory(&mut self, space: Option<&str>, address: i32, width: i32) -> Option<i64> {
        self.scope.read_memory(space, address, width)
    }

    fn symbol_names(&mut self) -> Vec<String> {
        self.scope.symbol_names()
    }
}

impl<C: Context + Sourced> Sourced for UnitContext<'_, C> {
    fn source(&mut self) -> &mut dyn Source {
        self.scope.source()
    }
}

/// `OwnNames`: what a processor's context answers besides labels, which no unit spelling may take.
pub fn own_names(registers: &[(String, i32)], pack: &[String], stack: bool) -> Vec<String> {
    let mut names: Vec<String> = registers.iter().map(|(n, _)| n.clone()).collect();
    names.extend(pack.iter().cloned());
    names.push("frame".to_string());
    if stack {
        names.push("stackdepth".to_string());
    }
    names
}

/// The lower-case name a scope word matches, `DebugCpu.Name`.
pub fn cpu_name(name: &str) -> String {
    to_lower(name)
}
