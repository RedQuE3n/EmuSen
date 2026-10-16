//! `emusen.debug`: the inspection and editing of a machine's state, each command written to print what its C# class
//! printed, over the session service. A C# command's returned text is standard output with status 0, its `Fail` is
//! standard error with status 1, and an exception its dispatcher caught is `Error: <message>` on standard error.
//! See EmuSen_DianaSh.md §8.1 and §13.11.

mod code;
mod memory;
mod state;
mod views;

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_session_service::{abi, Refusal, Session};

/// What a command gives back, or an exception's words.
pub(crate) type Run = Result<Answer, String>;

/// The commands this module has, with their read-only flags and today's usage lines.
pub const COMMANDS: &[(&str, bool, &str)] = &[
    ("spaces", true, "  spaces                        list available memory spaces"),
    ("mem", true, "  mem <space> <addr> [<len>]    hexdump <len> bytes (default 16) from <space> at <addr>"),
    ("write", false, "  write <space> <addr> <value>  write one byte (only if <space> is writable)"),
    ("regs", true, concat!(
        "  regs                          CPU + video (+ APU/coprocessor) registers\n",
        "  regs <cpu>                    just one processor's registers - `cpus` lists them\n",
        "  (to WRITE a register, see `setreg` - kept separate so `regs` stays read-only)"
    )),
    ("sprites", true, "  sprites                       active sprite/OBJ table"),
    ("pal", true, "  pal [<index>]                 one palette, or all of them if omitted"),
    ("channels", true, "  channels                      list audio channels/voices - index, active, envelope level (0-100), muted, core-specific detail (see `mute` to isolate one)"),
    ("mute", false, "  mute <index> <on|off>         mute/unmute one audio channel - channel's own playback state still advances, it's just excluded from the mix"),
    ("eval", true, concat!(
        "  eval <expr>                   evaluate an expression against live state (decimal, hex, binary)\n",
        "  eval symbols [<filter>]       list every symbol an expression can name\n",
        "  eval <cpu> <expr>             evaluate against another processor's registers - `cpus` lists them"
    )),
    ("label", false, concat!(
        "  label add <addr> <name> [<comment...>]   name an address\n",
        "  label list [<filter>]                    list labels, address order\n",
        "  label at <addr>                          what is named at (or nearest below) <addr>\n",
        "  label remove <name|addr>                 drop one label\n",
        "  label clear                              drop every label\n",
        "  label load <path>                        merge a label file in\n",
        "  label save <path>                        write every label out"
    )),
    ("cpus", true, "  cpus                          list the processors a debug command can be scoped to"),
    ("setreg", false, concat!(
        "  setreg <reg> <value>          write one of the main CPU's registers\n",
        "  setreg <cpu> <reg> <value>    same, on another processor - `cpus` lists them"
    )),
    ("vectors", true, "  vectors                       where each interrupt vector points, resolved through labels"),
    ("addr", true, "  addr <addr>                   what a CPU-bus address decodes to (ROM offset, RAM, a register)"),
    ("search", false, concat!(
        "  search <space> <val> [<w>]    start a new search: find every address currently equal to <val>\n",
        "                                (width <w> in bytes: 1, 2, or 4 - default 1, little-endian)\n",
        "  search refine <val>           narrow the current search to addresses now equal to <val>\n",
        "  search changed|unchanged      narrow to addresses whose value did/didn't change since last search/refine\n",
        "  search increased|decreased    narrow to addresses whose value went up/down since last search/refine\n",
        "  search list [<count>]         list current candidate addresses + values (default 20)\n",
        "  search reset                  clear the current search"
    )),
    ("memfind", true, concat!(
        "  memfind <space> <bytes> [<max>]  find every offset where <bytes> occurs (hex pairs, '??' matches\n",
        "                                any byte, separators optional) - default at most 20 hits"
    )),
    ("snapshot", false, concat!(
        "  snapshot <space> <name>       capture <space>'s full current contents under <name>\n",
        "  snapshot list                 list saved snapshots (name, space, size)\n",
        "  snapshot remove <name>        delete a saved snapshot"
    )),
    ("diff", true, concat!(
        "  diff <name> [<count>]         compare snapshot <name> against that space's CURRENT contents,\n",
        "                                print addresses that changed (default 20 shown)"
    )),
    ("dump", true, "  dump <space> <addr> <len> <file> write raw bytes to home/Logs/<CoreName>/<file>"),
    ("load", false, "  load <space> <addr> <file>    write home/Logs/<CoreName>/<file>'s raw bytes into <space> starting at <addr>"),
    ("tile", true, "  tile <space> <addr> <bpp>     ASCII-decode one 8x8 tile (bpp meaning is core-specific - 2/4/8 on SNES)"),
    ("tilemap", true, concat!(
        "  tilemap <space> <addr> <cols> <rows>\n",
        "                                decode <cols>x<rows> tilemap entries starting at <addr>\n",
        "                                as a text grid, one already-formatted label per cell -\n",
        "                                see IDebugTarget.DecodeTilemapEntry for what a label\n",
        "                                means on the current core. Caller supplies the exact base\n",
        "                                address (see BG1SC/etc via `regs`) - this doesn't resolve\n",
        "                                BG-layer-to-tilemap-address mapping itself."
    )),
    ("disasm", true, concat!(
        "  disasm <space> <addr> [<n>]   disassemble <n> instructions (default 10)\n",
        "  disasm <cpu> [<addr>] [<n>]   same, in that processor's own code space and ISA -\n",
        "                                with no <addr>, starts at where it is executing now\n",
        "  ... [m8|m16] [x8|x16]         65816 only: force the starting operand widths"
    )),
    ("callers", true, concat!(
        "  callers <addr> [<scanstart> <scanlen>]\n",
        "                                find instructions statically calling/jumping to <addr>,\n",
        "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)"
    )),
    ("writers", true, concat!(
        "  writers <addr> [<scanstart> <scanlen>]\n",
        "                                find instructions statically writing to <addr>,\n",
        "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)"
    )),
    ("readers", true, concat!(
        "  readers <addr> [<scanstart> <scanlen>]\n",
        "                                find instructions statically reading <addr>,\n",
        "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)"
    )),
];

/// The module: one value, its per-shell state in the call's slot.
pub struct Debug;

pub static MODULE: Debug = Debug;

/// The session the host provides, or `RequireTarget`'s refusal when it provides none.
pub(crate) fn session<'a>(call: &Call<'a>) -> Result<Session<'a>, Refusal> {
    Session::from_service(call.service(abi::SERVICE, abi::MAJOR))
}

/// `CoreName`, as the session's `info` gives it.
pub(crate) fn core_name(session: &Session) -> Result<String, Refusal> {
    session.core_name()
}

/// The C# `RequireTarget` caught and returned as a failure, as the commands that wrap it in try/catch do.
pub(crate) fn required<'a>(call: &Call<'a>) -> Result<Session<'a>, Answer> {
    session(call).and_then(|s| s.info().map(|_| s)).map_err(|refusal| Answer::fail(refusal.words))
}

impl Module for Debug {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| {
            Manifest { id: "emusen.debug", version: env!("CARGO_PKG_VERSION"), services: &[(abi::SERVICE, abi::MAJOR, 0, false)], commands: COMMANDS, aliases: &[], jobs: &[] }.json()
        })
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        let answer = match call.command().as_str() {
            "spaces" => memory::spaces(call),
            "mem" => memory::mem(call, &args),
            "write" => memory::write(call, &args),
            "search" => state::search(call, &args),
            "memfind" => memory::memfind(call, &args),
            "snapshot" => state::snapshot(call, &args),
            "diff" => state::diff(call, &args),
            "dump" => memory::dump(call, &args),
            "load" => memory::load(call, &args),
            "regs" => code::regs(call, &args),
            "setreg" => code::setreg(call, &args),
            "cpus" => code::cpus(call),
            "eval" => code::eval(call, &args),
            "label" => code::label(call, &args),
            "vectors" => code::vectors(call),
            "addr" => code::addr(call, &args),
            "disasm" => code::disasm(call, &args),
            "callers" => code::scan(call, &args, "callers", "call"),
            "writers" => code::scan(call, &args, "writers", "write"),
            "readers" => code::scan(call, &args, "readers", "read"),
            "tile" => views::tile(call, &args),
            "tilemap" => views::tilemap(call, &args),
            "sprites" => views::sprites(call),
            "pal" => views::pal(call, &args),
            "channels" => views::channels(call),
            "mute" => views::mute(call, &args),
            other => Err(format!("emusen.debug has no command '{other}'")),
        };
        call.answer(answer.unwrap_or_else(Answer::fault))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_names_every_command_once_and_asks_for_the_session() {
        let manifest = Debug.manifest();
        for (name, _, _) in COMMANDS {
            assert_eq!(manifest.matches(&format!("\"name\":\"{name}\"")).count(), 1, "{name}");
        }
        assert!(manifest.contains("\"services\":[{\"name\":\"emusen.session\",\"major\":1,\"minor\":0,\"required\":false}]"));
        assert_eq!(COMMANDS.len(), 26);
    }
}
