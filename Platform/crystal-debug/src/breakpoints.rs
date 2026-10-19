//! Halting a machine where control flow reaches an address or memory is touched, the C# `BreakpointRegistry` (`man bp`,
//! `man step`, `man runto`). The registry decides; a condition or a logpoint is rendered by the caller's [`Evaluate`],
//! which [`crate::conditions`] makes over the registry's scope.

use emusen_dotnet::text::{equals_ignore_case, fold, to_lower};
use std::collections::{HashMap, VecDeque};

/// How a call stack frame was entered, `CallFrameKind`, numbered as the session service writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CallKind {
    Call = 0,
    Nmi = 1,
    Irq = 2,
    Brk = 3,
    Cop = 4,
}

impl CallKind {
    pub fn from_number(n: i64) -> Option<CallKind> {
        Some(match n {
            0 => CallKind::Call,
            1 => CallKind::Nmi,
            2 => CallKind::Irq,
            3 => CallKind::Brk,
            4 => CallKind::Cop,
            _ => return None,
        })
    }

    /// The member's name, `CallFrameKind.ToString()`.
    pub fn name(self) -> &'static str {
        match self {
            CallKind::Call => "Call",
            CallKind::Nmi => "Nmi",
            CallKind::Irq => "Irq",
            CallKind::Brk => "Brk",
            CallKind::Cop => "Cop",
        }
    }
}

/// What a registry asks of its scope: a condition's truth and a logpoint's text, each or the words of its failure.
pub trait Evaluate {
    fn condition(&mut self, condition: &str) -> Result<bool, String>;
    fn render(&mut self, log: &str) -> Result<String, String>;
}

/// The evaluator of a registry no condition was ever prepared on: every condition holds and a log is its own text.
pub struct Unprepared;

impl Evaluate for Unprepared {
    fn condition(&mut self, _: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn render(&mut self, log: &str) -> Result<String, String> {
        Ok(log.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Breakpoint {
    pub id: i32,
    pub address: i32,
    pub end: i32,
    pub enabled: bool,
    pub hits: i64,
    pub condition: Option<String>,
    pub log: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataBreakpoint {
    pub id: i32,
    pub space: String,
    pub address: i32,
    pub end: i32,
    pub value: i32,
    pub on_read: bool,
    pub changed_only: bool,
    pub enabled: bool,
    pub hits: i64,
    pub condition: Option<String>,
    pub log: Option<String>,
    last_seen: Option<HashMap<i32, u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForbidRange {
    pub id: i32,
    pub address: i32,
    pub end: i32,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionBreak {
    pub name: String,
    pub log_only: bool,
    pub enabled: bool,
    pub hits: i64,
}

/// One logpoint's line, or a log-only condition's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub id: i32,
    pub address: i32,
    pub text: String,
}

const LOG_CAPACITY: usize = 4096;

pub struct BreakpointRegistry {
    breakpoints: Vec<Breakpoint>,
    data: Vec<DataBreakpoint>,
    forbid: Vec<ForbidRange>,
    /// Armed hardware conditions by name ignoring case, in the order a .NET dictionary enumerates them.
    conditions: Vec<(String, ConditionBreak)>,
    next_id: i32,
    data_break_pending: bool,
    last_data_break: String,
    single_step_armed: bool,
    steps_remaining: i32,
    step_depth_target: Option<i32>,
    event_break_pending: bool,
    last_event_break: String,
    run_to_interrupt: Option<CallKind>,
    run_to_scanline: i32,
    run_to_frame: i64,
    uninit_space: String,
    uninit_written: Option<Vec<bool>>,
    depth_guard: i32,
    log: VecDeque<LogEntry>,
    log_recorded: i64,
    /// Whether a condition was ever prepared, `ConditionEvaluator` set; without, every condition holds.
    pub prepared: bool,
    /// Whether the registry is told a call stack, `CallStack` set.
    pub knows_stack: bool,
    pub last_break_reason: String,
    pub last_condition_error: String,
    any_condition_armed: bool,
    /// The breakpoint whose condition failed this call, for the caller to drop it.
    dropped: Option<i32>,
}

impl Default for BreakpointRegistry {
    fn default() -> Self {
        BreakpointRegistry {
            breakpoints: Vec::new(),
            data: Vec::new(),
            forbid: Vec::new(),
            conditions: Vec::new(),
            next_id: 1,
            data_break_pending: false,
            last_data_break: String::new(),
            single_step_armed: false,
            steps_remaining: 0,
            step_depth_target: None,
            event_break_pending: false,
            last_event_break: String::new(),
            run_to_interrupt: None,
            run_to_scanline: -1,
            run_to_frame: -1,
            uninit_space: String::new(),
            uninit_written: None,
            depth_guard: -1,
            log: VecDeque::new(),
            log_recorded: 0,
            prepared: false,
            knows_stack: false,
            last_break_reason: String::new(),
            last_condition_error: String::new(),
            any_condition_armed: false,
            dropped: None,
        }
    }
}

impl BreakpointRegistry {
    pub fn new() -> BreakpointRegistry {
        BreakpointRegistry::default()
    }

    pub fn log_recorded(&self) -> i64 {
        self.log_recorded
    }

    pub fn log_tail(&self, count: i32) -> Vec<LogEntry> {
        let skip = self.log.len().saturating_sub(count.max(0) as usize);
        self.log.iter().skip(skip).cloned().collect()
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
        self.log_recorded = 0;
    }

    fn push_log(&mut self, entry: LogEntry) {
        if self.log.len() >= LOG_CAPACITY {
            self.log.pop_front();
        }
        self.log.push_back(entry);
        self.log_recorded += 1;
    }

    /// True when the caller should halt, false when it only logged.
    fn record_log(&mut self, log: Option<&str>, id: i32, address: i32, eval: &mut dyn Evaluate) -> bool {
        let Some(expression) = log.filter(|l| !l.is_empty()) else { return true };
        let text = if self.prepared {
            match eval.render(expression) {
                Ok(t) => t,
                Err(e) => e,
            }
        } else {
            expression.to_string()
        };
        self.push_log(LogEntry { id, address, text });
        false
    }

    pub fn add_breakpoint(&mut self, address: i32, end: i32, condition: Option<&str>, log: Option<&str>) -> i32 {
        let (address, end) = if end < address { (end, address) } else { (address, end) };
        let id = self.next_id;
        self.next_id += 1;
        self.breakpoints.push(Breakpoint { id, address, end, enabled: true, hits: 0, condition: condition.map(str::to_string), log: log.map(str::to_string) });
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_data_breakpoint(&mut self, space: &str, address: i32, end: i32, value: i32, on_read: bool, changed_only: bool, condition: Option<&str>, log: Option<&str>) -> i32 {
        let (address, end) = if end < address { (end, address) } else { (address, end) };
        let id = self.next_id;
        self.next_id += 1;
        self.data.push(DataBreakpoint {
            id,
            space: space.to_string(),
            address,
            end,
            value,
            on_read,
            changed_only,
            enabled: true,
            hits: 0,
            condition: condition.map(str::to_string),
            log: log.map(str::to_string),
            last_seen: changed_only.then(HashMap::new),
        });
        id
    }

    pub fn add_forbid_range(&mut self, address: i32, end: i32) -> i32 {
        let (address, end) = if end < address { (end, address) } else { (address, end) };
        let id = self.next_id;
        self.next_id += 1;
        self.forbid.push(ForbidRange { id, address, end, enabled: true });
        id
    }

    pub fn remove(&mut self, id: i32) -> bool {
        let before = self.breakpoints.len() + self.data.len() + self.forbid.len();
        self.breakpoints.retain(|b| b.id != id);
        self.data.retain(|b| b.id != id);
        self.forbid.retain(|b| b.id != id);
        self.breakpoints.len() + self.data.len() + self.forbid.len() < before
    }

    pub fn breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    pub fn data_breakpoints(&self) -> &[DataBreakpoint] {
        &self.data
    }

    pub fn forbid_ranges(&self) -> &[ForbidRange] {
        &self.forbid
    }

    pub fn last_data_break(&self) -> &str {
        &self.last_data_break
    }

    pub fn last_event_break(&self) -> &str {
        &self.last_event_break
    }

    /// `NoteWrite`: from the drain of the machine's stores.
    pub fn note_write(&mut self, space: &str, address: i32, value: u8, eval: &mut dyn Evaluate) {
        self.note_uninitialized_write(space, address);
        self.note_data(false, space, address, value, eval);
    }

    /// `NoteRead`: from the drain of the machine's loads, with the uninitialised-read check.
    pub fn note_read(&mut self, space: &str, address: i32, value: u8, eval: &mut dyn Evaluate) {
        self.note_uninitialized_read(space, address);
        self.note_data(true, space, address, value, eval);
    }

    fn note_data(&mut self, read: bool, space: &str, address: i32, value: u8, eval: &mut dyn Evaluate) {
        for i in 0..self.data.len() {
            let bp = &self.data[i];
            if !bp.enabled || bp.on_read != read || address < bp.address || address > bp.end {
                continue;
            }
            if !equals_ignore_case(&bp.space, space) || (bp.value >= 0 && bp.value != value as i32) {
                continue;
            }
            if !read && bp.changed_only {
                let seen = self.data[i].last_seen.get_or_insert_with(HashMap::new);
                if seen.get(&address) == Some(&value) {
                    continue;
                }
                seen.insert(address, value);
            }
            let (id, condition) = (self.data[i].id, self.data[i].condition.clone());
            if !self.condition_holds(condition.as_deref(), id, eval) {
                continue;
            }
            if self.condition_dropped(id) {
                self.data[i].condition = None;
            }
            self.data[i].hits += 1;
            let log = self.data[i].log.clone();
            if !self.record_log(log.as_deref(), id, address, eval) {
                return;
            }
            self.data_break_pending = true;
            self.last_data_break = if read { format!("#{id} read {space} 0x{address:X} = 0x{value:02X}") } else { format!("#{id} {space} 0x{address:X} = 0x{value:02X}") };
            return;
        }
    }

    /// `ArmUninitializedReadBreak`, sized by the caller, which knows the space.
    pub fn arm_uninitialized(&mut self, space: &str, size: i32) -> bool {
        if size <= 0 {
            return false;
        }
        self.uninit_space = space.to_string();
        self.uninit_written = Some(vec![false; size as usize]);
        true
    }

    pub fn disarm_uninitialized(&mut self) {
        self.uninit_space.clear();
        self.uninit_written = None;
    }

    pub fn uninitialized_armed(&self) -> bool {
        self.uninit_written.is_some()
    }

    pub fn uninitialized_space(&self) -> &str {
        &self.uninit_space
    }

    fn note_uninitialized_write(&mut self, space: &str, address: i32) {
        let Some(written) = self.uninit_written.as_mut() else { return };
        if address < 0 || address as usize >= written.len() || !equals_ignore_case(space, &self.uninit_space) {
            return;
        }
        written[address as usize] = true;
    }

    fn note_uninitialized_read(&mut self, space: &str, address: i32) {
        let Some(written) = self.uninit_written.as_mut() else { return };
        if address < 0 || address as usize >= written.len() || !equals_ignore_case(space, &self.uninit_space) || written[address as usize] {
            return;
        }
        // Reported once an address, or a boot loop halts on the same byte forever.
        written[address as usize] = true;
        self.data_break_pending = true;
        self.last_data_break = format!("uninitialized read of {space} 0x{address:X}");
    }

    pub fn any_condition_armed(&self) -> bool {
        self.any_condition_armed
    }

    fn condition_index(&self, name: &str) -> Option<usize> {
        let key = fold(name);
        self.conditions.iter().position(|(k, _)| *k == key)
    }

    pub fn is_condition_armed(&self, name: &str) -> bool {
        self.condition_index(name).is_some_and(|i| self.conditions[i].1.enabled)
    }

    pub fn arm_condition(&mut self, name: &str, log_only: bool) {
        let entry = ConditionBreak { name: name.to_string(), log_only, enabled: true, hits: 0 };
        match self.condition_index(name) {
            Some(i) => self.conditions[i].1 = entry,
            None => self.conditions.push((fold(name), entry)),
        }
        self.any_condition_armed = true;
    }

    pub fn disarm_condition(&mut self, name: &str) -> bool {
        let removed = match self.condition_index(name) {
            Some(i) => {
                self.conditions.remove(i);
                true
            }
            None => false,
        };
        self.any_condition_armed = !self.conditions.is_empty();
        removed
    }

    pub fn conditions(&self) -> impl Iterator<Item = &ConditionBreak> {
        self.conditions.iter().map(|(_, c)| c)
    }

    /// `NoteCondition`: a hardware condition the core detected; halts unless armed log-only.
    pub fn note_condition(&mut self, name: &str, address: i32, detail: &str) {
        let Some(i) = self.condition_index(name) else { return };
        if !self.conditions[i].1.enabled {
            return;
        }
        self.conditions[i].1.hits += 1;
        if self.conditions[i].1.log_only {
            self.push_log(LogEntry { id: 0, address, text: detail.to_string() });
            return;
        }
        self.event_break_pending = true;
        self.last_event_break = detail.to_string();
    }

    /// `NoteInterrupt`: from an interrupt the call log names.
    pub fn note_interrupt(&mut self, kind: CallKind) {
        if self.any_condition_armed && matches!(kind, CallKind::Brk | CallKind::Cop) {
            let name = to_lower(kind.name());
            self.note_condition(&name, 0, &format!("{} taken", name.to_uppercase()));
        }
        if self.run_to_interrupt != Some(kind) {
            return;
        }
        self.run_to_interrupt = None;
        self.event_break_pending = true;
        self.last_event_break = format!("{} taken", kind.name().to_uppercase());
    }

    pub fn note_scanline(&mut self, scanline: i32) {
        if self.run_to_scanline < 0 || scanline != self.run_to_scanline {
            return;
        }
        self.run_to_scanline = -1;
        self.event_break_pending = true;
        self.last_event_break = format!("scanline {scanline} reached");
    }

    /// `NoteFrame`: once a completed frame, with its number.
    pub fn note_frame(&mut self, frame: i64) {
        if self.run_to_frame < 0 || frame < self.run_to_frame {
            return;
        }
        self.run_to_frame = -1;
        self.event_break_pending = true;
        self.last_event_break = format!("frame {frame} reached");
    }

    pub fn set_enabled(&mut self, id: i32, enabled: bool) -> bool {
        if let Some(b) = self.breakpoints.iter_mut().find(|b| b.id == id) {
            b.enabled = enabled;
            return true;
        }
        if let Some(b) = self.data.iter_mut().find(|b| b.id == id) {
            b.enabled = enabled;
            return true;
        }
        if let Some(r) = self.forbid.iter_mut().find(|r| r.id == id) {
            r.enabled = enabled;
            return true;
        }
        false
    }

    pub fn arm_single_step(&mut self) {
        self.arm_step(1);
    }

    pub fn arm_step(&mut self, instructions: i32) {
        self.steps_remaining = instructions.max(1);
        self.step_depth_target = None;
        self.single_step_armed = true;
    }

    /// `ArmStepToDepth`: halt once the stack is back to `depth`; false for a registry told no call stack.
    pub fn arm_step_to_depth(&mut self, depth: i32) -> bool {
        if !self.knows_stack {
            return false;
        }
        self.step_depth_target = Some(depth);
        self.steps_remaining = 0;
        self.single_step_armed = false;
        true
    }

    pub fn arm_depth_guard(&mut self, depth: i32) -> bool {
        if !self.knows_stack {
            return false;
        }
        self.depth_guard = depth;
        true
    }

    pub fn disarm_depth_guard(&mut self) {
        self.depth_guard = -1;
    }

    pub fn depth_guard(&self) -> i32 {
        self.depth_guard
    }

    pub fn single_step_armed(&self) -> bool {
        self.single_step_armed
    }

    pub fn step_depth_target(&self) -> Option<i32> {
        self.step_depth_target
    }

    pub fn has_pending_break(&self) -> bool {
        self.data_break_pending || self.event_break_pending
    }

    pub fn arm_run_to_interrupt(&mut self, kind: CallKind) -> bool {
        self.run_to_interrupt = Some(kind);
        true
    }

    pub fn arm_run_to_scanline(&mut self, scanline: i32) {
        self.run_to_scanline = scanline;
    }

    pub fn arm_run_to_frame(&mut self, frame: i64) {
        self.run_to_frame = frame;
    }

    /// `DisarmSteps`: every armed step and run-to cancelled, the breakpoints left alone.
    pub fn disarm_steps(&mut self) {
        self.single_step_armed = false;
        self.steps_remaining = 0;
        self.step_depth_target = None;
        self.run_to_interrupt = None;
        self.run_to_scanline = -1;
        self.run_to_frame = -1;
        self.event_break_pending = false;
    }

    /// A condition that fails to evaluate is reported and dropped, and holds this once.
    fn condition_holds(&mut self, condition: Option<&str>, id: i32, eval: &mut dyn Evaluate) -> bool {
        let Some(condition) = condition.filter(|c| !c.is_empty()) else { return true };
        if !self.prepared {
            return true;
        }
        match eval.condition(condition) {
            Ok(result) => result,
            Err(error) => {
                self.last_condition_error = format!("breakpoint #{id}: {error} (condition dropped)");
                self.dropped = Some(id);
                true
            }
        }
    }

    fn condition_dropped(&mut self, id: i32) -> bool {
        if self.dropped == Some(id) {
            self.dropped = None;
            return true;
        }
        false
    }

    fn forbidden(&self, address: i32) -> bool {
        self.forbid.iter().any(|r| r.enabled && address >= r.address && address <= r.end)
    }

    /// `CouldBreak`: false only when `should_break` would answer false and change nothing.
    pub fn could_break(&self) -> bool {
        self.single_step_armed || self.step_depth_target.is_some() || self.depth_guard >= 0 || self.data_break_pending || self.event_break_pending || !self.breakpoints.is_empty()
    }

    /// `WatchesWrites`: false exactly when `note_write` would do nothing.
    pub fn watches_writes(&self) -> bool {
        !self.data.is_empty() || self.uninit_written.is_some()
    }

    /// `IsQuiet`: nothing armed can halt before a command changes the registry.
    pub fn is_quiet(&self) -> bool {
        !self.could_break()
            && self.data.is_empty()
            && self.uninit_written.is_none()
            && !self.any_condition_armed
            && self.run_to_interrupt.is_none()
            && self.run_to_scanline < 0
            && self.run_to_frame < 0
    }

    /// `ShouldBreak`: once an instruction, before it runs; `depth` is the call stack's.
    pub fn should_break(&mut self, address: i32, depth: i32, eval: &mut dyn Evaluate) -> bool {
        if self.single_step_armed {
            self.steps_remaining -= 1;
            if self.steps_remaining <= 0 {
                self.single_step_armed = false;
                self.last_break_reason = "single-step".to_string();
                return true;
            }
        }
        if let Some(target) = self.step_depth_target {
            if self.knows_stack && depth <= target {
                self.step_depth_target = None;
                self.last_break_reason = "step over/out complete".to_string();
                return true;
            }
        }
        if !self.forbid.is_empty() && self.forbidden(address) {
            self.data_break_pending = false;
            return false;
        }
        if self.depth_guard >= 0 && self.knows_stack && depth > self.depth_guard {
            self.last_break_reason = format!("call depth {depth} exceeded the guard of {}", self.depth_guard);
            self.depth_guard = -1;
            return true;
        }
        if self.data_break_pending {
            self.data_break_pending = false;
            self.last_break_reason =
                if self.last_data_break.starts_with("uninitialized") { self.last_data_break.clone() } else { format!("data breakpoint {}", self.last_data_break) };
            return true;
        }
        if self.event_break_pending {
            self.event_break_pending = false;
            self.last_break_reason = self.last_event_break.clone();
            return true;
        }
        for i in 0..self.breakpoints.len() {
            let bp = &self.breakpoints[i];
            if !bp.enabled || address < bp.address || address > bp.end {
                continue;
            }
            let (id, condition) = (bp.id, bp.condition.clone());
            if !self.condition_holds(condition.as_deref(), id, eval) {
                continue;
            }
            if self.condition_dropped(id) {
                self.breakpoints[i].condition = None;
            }
            self.breakpoints[i].hits += 1;
            let log = self.breakpoints[i].log.clone();
            if !self.record_log(log.as_deref(), id, address, eval) {
                continue;
            }
            let bp = &self.breakpoints[i];
            let where_ = if bp.address == bp.end { format!("at ${address:06X}") } else { format!("at ${address:06X} (in ${:06X}-${:06X})", bp.address, bp.end) };
            self.last_break_reason = match &bp.condition {
                None => format!("breakpoint #{id} {where_}"),
                Some(c) => format!("breakpoint #{id} {where_} (if {c})"),
            };
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failing_condition_is_dropped_and_holds_once() {
        struct Failing;
        impl Evaluate for Failing {
            fn condition(&mut self, _: &str) -> Result<bool, String> {
                Err("Unknown symbol 'Q'.".into())
            }
            fn render(&mut self, l: &str) -> Result<String, String> {
                Ok(l.into())
            }
        }
        let mut r = BreakpointRegistry::new();
        r.prepared = true;
        let id = r.add_breakpoint(0x10, 0x10, Some("Q"), None);
        assert!(r.should_break(0x10, 0, &mut Failing));
        assert_eq!(r.last_condition_error, format!("breakpoint #{id}: Unknown symbol 'Q'. (condition dropped)"));
        assert_eq!(r.last_break_reason, "breakpoint #1 at $000010");
        assert_eq!(r.breakpoints()[0].condition, None);
    }
}
