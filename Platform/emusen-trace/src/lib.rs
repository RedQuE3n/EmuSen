//! `emusen.trace`: what arms the debugger and controls a run, each command written to print what its C# class
//! printed, over the session service. Run control asks the host's loop to run through the service, which a host reads
//! as its own event (§6.2); `ps` lists this module's breakpoints and watches (§6.8). See EmuSen_DianaSh.md §8.1 and
//! §13.11.

mod control;
mod records;

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_dotnet::number::parse_i32;
use emusen_dotnet::text::{pad_right, starts_with_ignore_case};
use emusen_session_service::scope::{self, Cpu};
use emusen_session_service::words::hex;
use emusen_session_service::{abi, Record, Refusal, Session};

/// What a command gives back, or an exception's words.
pub(crate) type Run = Result<Answer, String>;

/// The commands this module has, with their read-only flags and today's usage lines.
pub const COMMANDS: &[(&str, bool, &str)] = &[
    ("cophist", true, "  cophist [<reg>] [<count>]     Coprocessor register history (default: all regs, last 16)"),
    ("watch", false, concat!(
        "  watch add <space> <addr> <len> [write|read|both]\n",
        "                                register a watch, prints matching accesses live + stores them\n",
        "                                (default write, matching this command's original behavior)\n",
        "  watch list                    list active watches with their IDs and access kind\n",
        "  watch log <id> [<count>]      show a watch's recorded events (default 20)\n",
        "  watch summary <id>            group a watch's recorded events by access site (Context),\n",
        "                                showing each unique site once with a hit count instead of\n",
        "                                every individual event - the dynamic, addressing-mode-agnostic\n",
        "                                equivalent of `readers`/`writers`, which only find statically-\n",
        "                                visible absolute/absolute-long instructions and miss indexed/\n",
        "                                indirect access entirely. Only covers whatever's still in the\n",
        "                                watch's own bounded event buffer, same as `watch log`.\n",
        "  watch clear <id>              clear a watch's stored events (doesn't remove the watch)\n",
        "  watch remove <id>             remove a watch entirely"
    )),
    ("bp", false, concat!(
        "  bp add <addr>[-<end>] [if <expr>] [log <expr>]\n",
        "                                halt just before <addr> runs (24-bit CPU address, or a range);\n",
        "                                with 'log' it records <expr> and carries on instead of halting\n",
        "  bp write <space> <addr>[-<end>] [<v>] [changed] [if <expr>]\n",
        "                                halt just after anything writes that address\n",
        "  bp read <space> <addr>[-<end>] [<v>] [if <expr>]\n",
        "                                halt just after anything reads it\n",
        "  bp uninit <space>             halt the first time never-written memory is read back\n",
        "  bp when <condition> [log|off] halt (or just record) when the hardware does something\n",
        "                                it should not - `bp when` alone lists what this core detects\n",
        "  bp depth <n>                  halt if the call stack ever gets deeper than <n>\n",
        "  bp forbid <addr>-<end>        never halt while the PC is in this range\n",
        "  bp log [<count>] | bp log clear\n",
        "                                show (or forget) what the logpoints have recorded\n",
        "  bp list                       list active breakpoints with their IDs and hit counts\n",
        "  bp on|off <id>                enable/disable one breakpoint without removing it\n",
        "  bp remove <id>                remove a breakpoint entirely\n",
        "  bp <cpu> add|list|remove ...  same, on another processor - `cpus` lists them, and each\n",
        "                                one's addresses are its own address space\n",
        "  (see also: `cpus`, `step`, `runto`, and the F4 prompt's own 'continue'/'c')"
    )),
    ("runto", false, concat!(
        "  runto nmi | irq | brk | cop   resume until the next interrupt of that kind is taken\n",
        "  runto scanline <n>            resume until scanline <n> starts\n",
        "  runto frame [<n>]             resume until frame <n> (default: the next one)\n",
        "  runto <addr>                  resume until <addr> executes (a one-shot breakpoint)"
    )),
    ("bt", false, concat!(
        "  bt [<count>]                  backtrace: the live call chain, innermost frame first\n",
        "  bt reset                      forget the current chain (after a stack-manipulating routine)\n",
        "  bt <cpu> [<count>|reset]      same, on another processor - `cpus` lists them"
    )),
    ("counters", false, concat!(
        "  counters on <space>           start tallying reads/writes/executes for <space>\n",
        "  counters off                  stop tallying (kept counts stay readable)\n",
        "  counters clear                zero every count, stay armed\n",
        "  counters <addr> [<len>]       totals over a range, default length 1\n",
        "  counters top [r|w|x|u] [<n>]  hottest addresses, default reads, default 20\n",
        "  counters cold <addr> <len>    addresses in the range nothing ever touched"
    )),
    ("freeze", false, concat!(
        "  freeze add <space> <addr> [<value>]  pin <addr>; without <value>, at whatever it holds now\n",
        "  freeze list                          list frozen addresses and how many writes each blocked\n",
        "  freeze remove <id>                   unfreeze one address\n",
        "  freeze clear                         unfreeze everything"
    )),
    ("profile", false, concat!(
        "  profile on                    start charging instructions to the innermost routine\n",
        "  profile off                   stop (collected counts stay readable)\n",
        "  profile clear                 zero the counts, stay armed\n",
        "  profile top [<n>]             hottest routines, default 20\n",
        "  profile <cpu> on|off|top ...  same, on another processor - `cpus` lists them"
    )),
    ("copflow", false, concat!(
        "  copflow on [<size>]           start logging coprocessor register-window traffic\n",
        "  copflow off                   stop logging\n",
        "  copflow clear                 forget everything logged so far\n",
        "  copflow tail [<n>]            the last <n> accesses in order (default 20)\n",
        "  copflow stats                 per-register read/write tallies, busiest first\n",
        "  copflow poll                  the longest run of unchanging reads - a stuck handshake"
    )),
    ("cov", false, concat!(
        "  cov on|off                    start/stop recording executed addresses\n",
        "  cov clear                     forget everything recorded so far\n",
        "  cov <addr> [<len>]            report which of <len> bytes from <addr> ever executed\n",
        "                                (default 16); 0 executed means the code never ran\n",
        "  cov mark                      remember what has run so far, so `cov new` can subtract it\n",
        "  cov new <addr> [<len>]        only what has run SINCE the mark - do the thing, then ask\n",
        "  cov funcs [<count>]           routines discovered by watching calls land, busiest first\n",
        "  cov save|load <file>          persist the map to Logs/<CoreName>/<file> and merge it back\n",
        "  cov <cpu> on|off|clear|<addr> same, on another processor's own instruction stream -\n",
        "                                `cpus` lists them, and each has its own address space"
    )),
    ("dma", false, concat!(
        "  dma                           every channel's current configuration\n",
        "  dma log on [<n>] | dma log off\n",
        "                                start/stop recording transfers (<n> retained, default 4096)\n",
        "  dma log [<n>] | dma log clear\n",
        "                                the last <n> transfers in order (default 20), or forget them\n",
        "  dma stats                     per-channel and per-destination totals, biggest first"
    )),
    ("framelog", false, concat!(
        "  framelog add <space> <addr> [<width>]\n",
        "                                register a per-frame value sample (width 1/2/4 bytes, default 1)\n",
        "  framelog list                 list active frame logs with their IDs\n",
        "  framelog show <id> [<count>]  show a frame log's recorded (frame, value) samples (default 20)\n",
        "  framelog clear <id>           clear a frame log's stored samples (doesn't remove it)\n",
        "  framelog remove <id>          remove a frame log entirely"
    )),
    ("trace", false, concat!(
        "  trace <count>                 arm a live CPU instruction trace for the next <count> instructions\n",
        "  trace off                     cancel an in-progress trace early"
    )),
    ("log", false, concat!(
        "  log off                       silence every logging flag at once, without changing them\n",
        "  log on                        restore whatever each logging flag was individually set to\n",
        "  log status                    show whether the master switch is currently on or off"
    )),
    ("resume", false, "  resume | continue | c         resume emulation after an interactive halt (F4/breakpoint/single-step)"),
    ("step", false, concat!(
        "  step | s [<count>]            single-step one CPU instruction (or <count> of them)\n",
        "  step over                     run the call at the current instruction to completion, then halt\n",
        "  step out                      run until the current routine returns to its caller\n",
        "  step <cpu> [<count>|over|out] same, stepping another processor - `cpus` lists them"
    )),
];

/// A command's own usage text, as the C# returned its `Usage` for a malformed line.
pub(crate) fn usage(name: &str) -> &'static str {
    COMMANDS.iter().find(|(n, _, _)| *n == name).map_or("", |c| c.2)
}

/// The module: one value, no state of its own.
pub struct Trace;

pub static MODULE: Trace = Trace;

/// The session the host provides, or `RequireTarget`'s refusal when it provides none.
pub(crate) fn session<'a>(call: &Call<'a>) -> Result<Session<'a>, Refusal> {
    Session::from_service(call.service(abi::SERVICE, abi::MAJOR))
}

/// `RequireTarget` caught and returned as a failure, as the commands that wrap it in try/catch do.
pub(crate) fn required<'a>(call: &Call<'a>) -> Result<Session<'a>, Answer> {
    session(call).and_then(|s| s.info().map(|_| s)).map_err(|refusal| Answer::fail(refusal.words))
}

/// The session's processors, and the one a scope word at `at` aims at.
pub(crate) fn scoped<'p>(cpus: &'p [Cpu], parts: &[String], at: usize) -> (Option<&'p Cpu>, usize) {
    scope::resolve(cpus, parts, at)
}

/// Runs a closure for a session's answer, an exception's words in place of the answer where it fails.
fn answered(call: &Call, run: impl FnOnce() -> Run) -> i32 {
    call.answer(run().unwrap_or_else(Answer::fault))
}

impl Module for Trace {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| {
            Manifest {
                id: "emusen.trace",
                version: env!("CARGO_PKG_VERSION"),
                services: &[(abi::SERVICE, abi::MAJOR, 0, false)],
                commands: COMMANDS,
                aliases: &[("c", "resume"), ("s", "step")],
                jobs: &["bp", "watch"],
            }
            .json()
        })
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        answered(call, || match call.command().as_str() {
            "bp" => control::bp(call, &args),
            "watch" => control::watch(call, &args),
            "step" => control::step(call, &args),
            "runto" => control::runto(call, &args),
            "bt" => control::bt(call, &args),
            "resume" => control::resume(call),
            "trace" => control::trace(call, &args),
            "log" => control::log(call, &args),
            "counters" => records::counters(call, &args),
            "freeze" => records::freeze(call, &args),
            "profile" => records::profile(call, &args),
            "cov" => records::cov(call, &args),
            "framelog" => records::framelog(call, &args),
            "copflow" => records::copflow(call, &args),
            "cophist" => records::cophist(call, &args),
            "dma" => records::dma(call, &args),
            other => Err(format!("emusen.trace has no command '{other}'")),
        })
    }

    /// `PsCommand.JobLines`: the session's own breakpoints, then its watches.
    fn jobs(&self, call: &Call) {
        let Ok(session) = session(call) else { return };
        let mut lines = String::new();
        if let Ok(list) = session.bp_list(session.no_processor()) {
            for b in list.list("code") {
                let id = format!("bp{}", b.int("id"));
                lines += &format!(
                    "  {} breakpoint  ${} ({}, hit {}x)\n",
                    pad_right(&id, 10),
                    hex(b.address("address"), 6),
                    if b.flag("enabled") { "enabled" } else { "disabled" },
                    b.int("hits")
                );
            }
        }
        if let Ok(watches) = session.watch_list() {
            for w in watches.items() {
                let id = format!("watch{}", w.int("id"));
                let start = w.address("address");
                lines += &format!(
                    "  {} watch       {} 0x{start:X}-0x{:X} ({})\n",
                    pad_right(&id, 10),
                    w.text("space"),
                    start.wrapping_add(w.int("length") as i32).wrapping_sub(1),
                    control::watch_kind(w.int("kind"))
                );
            }
        }
        if !lines.is_empty() {
            call.out(&lines);
        }
    }

    /// `KillCommand.TryJob`: `bp<N>` or `watch<N>` removed, any other id not this module's.
    fn kill(&self, call: &Call, id: &str) -> Option<i32> {
        let (number, breakpoint) = if starts_with_ignore_case(id, "watch") {
            (parse_i32(&id[5..])?, false)
        } else if starts_with_ignore_case(id, "bp") {
            (parse_i32(&id[2..])?, true)
        } else {
            return None;
        };
        Some(answered(call, || {
            let session = session(call)?;
            session.info()?;
            let removed = if breakpoint { session.bp_remove(session.no_processor(), number as u32) } else { session.watch_remove(number as u32) };
            let what = if breakpoint { "breakpoint" } else { "watch" };
            Ok(match removed {
                Ok(()) => Answer::ok(format!("{} #{number} removed.", if breakpoint { "Breakpoint" } else { "Watch" })),
                Err(refusal) if refusal.is(abi::status::NO_SUCH_ID) => Answer::fail(format!("kill: no {what} #{number} found.")),
                Err(refusal) => return Err(refusal.words),
            })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_names_every_command_once_and_claims_its_jobs() {
        let manifest = Trace.manifest();
        for (name, _, _) in COMMANDS {
            assert_eq!(manifest.matches(&format!("\"name\":\"{name}\"")).count(), 1, "{name}");
        }
        assert!(manifest.contains("\"jobs\":[\"bp\",\"watch\"]"));
        assert!(manifest.contains("\"aliases\":{\"c\":\"resume\",\"s\":\"step\"}"));
        assert_eq!(COMMANDS.len(), 16);
    }
}
