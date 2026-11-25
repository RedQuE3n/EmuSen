# DianaSh and Debugging

**DianaSh** (`Platform/dianash`) is EmuSen's Unix-inspired command shell and core-agnostic hardware debugging console. It runs both as a standalone terminal executable (`dianash`, shipped alongside the full-screen terminal editor `nano` in `Platform/dianash-program`) and as an embeddable shell library inside `emusen_platform` connected to live emulation sessions, library tools, save-media tools, and developer checks.

For the C header contracts powering DianaSh see [Platform C Interfaces](Platform_C_Interfaces.md). Every builtin and module command also ships an individual manual page under [Docs/man/](../man/) accessible directly via `man <command>`.

---

## 1. Standalone `dianash` and `nano`

`Platform/dianash-program` builds two standalone executables:

- **`dianash`:** Starts with its virtual root `/` mapped to Galaxia's home directory (`tree::tree().home()`).
  - **Interactive mode:** Running `dianash` on an interactive terminal opens the prompt (`DianaSh #: `) with command history, line editing (`editor.rs`), and `Ctrl+C` cancellation wired directly to the shell's cancel flag. Standalone `dianash` loads `dianash-text`, `emusen-dev-module`, and `emusen-testing-module` out of the box.
  - **Script / one-shot mode:** `dianash -c <line> [<line>...]` executes each argument as a shell line non-interactively and exits with the last line's `$?` status code.
- **`nano <path>`:** Full-screen terminal text editor shipped beside `dianash`. Honors the `DIANASH_ROOT` sandbox environment variable, refuses paths outside the sandbox or non-interactive redirected terminals with exit status `1`, and supports cursor navigation, `Home`, `End`, `Page Up`, `Page Down`, `Enter`, `Backspace`, `Delete`, `^O` to save, and `^X` to exit.

When created through `emusen_platform` (`dianash.h`), the host supplies live service tables (`emusen.session`, `emusen.library`, `emusen.games`, `emusen.scrape`) and loads all EmuSen emulator, library, save-media, and debugging modules.

---

## 2. Shell Syntax, Builtins, Sandbox, and `man`

`Platform/dianash/src/{lexer,parser,interp,builtins,sandbox,man}.rs` implements a compact POSIX-inspired shell language:

- **Variables & quoting:** Variable assignment (`VAR=value`), expansion (`$VAR`, `${VAR}`), last command exit code (`$?`), and single/double quote escaping.
- **Pipelines & redirection:** Command pipes (`|`), output and append redirection (`>`, `>>`), input redirection (`<`), and command substitution (`$(...)`).
- **Control flow & sequencing:** `if ...; then ...; elif ...; else ...; fi`, `for ... in ...; do ...; done`, `while ...; do ...; done`, `break`, `continue`, and list operators (`;`, `&&`, `||`).
- **Built-in commands (`builtins.rs`):** `echo`, `cd`, `pwd`, `clear`, `true`, `false`, `test` (`[`), `history`, `help`, `man`, `ps`, `kill`, `tmux`, `whoami`, `who`, `su`, `useradd`, `userdel`, `passwd`, and `shutdown`.
- **Path sandbox (`sandbox.rs`):** Resolves paths chroot-style inside Galaxia's home directory (`DIANASH_ROOT` or `tree().home()`). A leading `/` refers to the sandbox root (such as `/Logs`, `/Cheats`, `/Games`, `/Media`); paths resolving outside the root (including after canonicalizing symlinks) are rejected. The sandbox provides a clean, portable workspace layout (a walled garden rather than an OS security boundary).
- **Online manual (`man`):** `man <command>` and `man -k <keyword>` display the plain-text manual pages shipped in `Docs/man/`.

---

## 3. Complete Table of Live DianaSh Command Modules

DianaSh's shell engine knows zero emulator types; every command domain is an independent module crate speaking `dianash_module.h`:

| Module ID | Crate (`Platform/`) | Key Commands | Summary |
|---|---|---|---|
| `dianash.text` | `dianash-text` | `sed`, `grep`, `wc`, `sort`, `uniq`, `awk`, `ls`, `cp`, `cat`, `head`, `tail`, `touch`, `find`, `xxd` (`hexdump`) *(note: `mv` and `rm` are intentionally suspended in the sandbox)* | Portable Unix text processing and read-safe file utilities over sandboxed paths. |
| `emusen.debug` | `emusen-debug` | `cpus`, `spaces`, `mem`, `write`, `regs`, `setreg`, `vectors`, `addr`, `sprites`, `pal`, `channels`, `mute`, `eval`, `label`, `search`, `memfind`, `snapshot`, `diff`, `dump`, `load`, `tile`, `tilemap`, `disasm`, `callers`, `writers`, `readers` | Core-agnostic CPU, memory, graphics, audio, disassembly, symbol, and memory-diff inspection over `emusen.session` (26 commands). |
| `emusen.trace` | `emusen-trace` | `bp`, `watch`, `step`, `runto`, `resume` (`continue` / `c`), `bt`, `cov`, `profile`, `counters`, `freeze`, `trace`, `framelog`, `dma`, `cophist`, `copflow`, `log` | Execution control, breakpoints, read/write watchpoints, call stacks, code coverage, instruction profiling, DMA/coprocessor traces, and RAM freezes (16 commands). |
| `emusen.cheat` | `emusen-cheat` | `cheat` (`add`, `list`, `enable`, `disable`, `remove`, `clear`, `master`, `import`, `export`, `db`, `db find`, `db load`, `db update`, `db prune`) | Multi-console RAM poke and ROM patch cheats, `.cht` import/export, and cheat database lookup/maintenance. |
| `emusen.session` | `emusen-session` | `summary`, `coretop` | `summary` prints the attached core's free-text state dump (`No target attached.` when idle). Because `dianash_module.h` buffers command output rather than owning a live TTY, `coretop` currently returns an explicit refusal stating that an interactive terminal or frontend dashboard window is required. |
| `emusen.cores` | `emusen-cores-module` | `cores` (`ls`, `info`, `fetch`, `get`, `update`, `rollback`, `rm`, `verify`, `approve`, `default`) | Lists native and hosted cores, downloads/verifies SHA-256 allowlisted Libretro cores, manages player approvals, and assigns default engines per console. |
| `emusen.library` | `emusen-library-module` | `chd`, `pkg`, `rom` | Converts/verifies CHD disc images, creates/checks/patches/zips game packages, and hashes/checks/deduplicates ROMs through `emusen.library`. |
| `emusen.memcard` | `emusen-memcard-module` | `memcard` | Inspects, creates, copies, moves, deletes, imports, and exports saves on PS1, PS2, N64 Controller Pak, and Sega CD BRAM images without launching a core. |
| `emusen.scrape` | `emusen-scrape-module` | `scrape` | Plans ScreenScraper quota usage, runs artwork/metadata scraping jobs, checks queue status, and promotes covers via `emusen.scrape`. |
| `emusen.dev` | `emusen-dev-module` | `dupes`, `hwcount`, `qml-rules`, `esde-check`, `theme-check` | Code quality and performance tools: CPD duplicate checks, Linux hardware performance counters (`hwcount`), QML zero-logic linting, ES-DE corpus audits, and theme package validation. |
| `emusen.testing` | `emusen-testing-module` | `units`, `impact`, `coverage`, `shard`, `trial`, `results`, `ingest`, `queue` | Testing studio for blast-radius impact analysis, trial execution, result queries, log ingestion, and runner queue management. |

---

## 4. Core-Agnostic Hardware Debugging Workflows

Every command in `emusen.debug` and `emusen.trace` discovers processors, hardware units, memory spaces, and register schemas dynamically from the attached core's machine info. The exact same recipes work across NES, SNES, Game Boy, Genesis, N64, PlayStation, and PlayStation 2 cores.

### 4.1 Exploring Hardware Topology, Memory, and Disassembly

```sh
# Discover processors, hardware units, and memory spaces on the live core
cpus
spaces

# Inspect CPU/unit registers and resolved interrupt vectors
regs
vectors

# Hexdump 64 bytes from WRAM, write a byte, and decode a bus address
mem wram 0x0000 64
write wram 0x0010 0x42
addr 0x8000

# Disassemble 16 instructions (SNES 65816 accepts optional m8|m16 x8|x16 flags)
disasm main 0x8000 16

# Evaluate expressions and manage symbol labels
label load /Logs/game.labels
eval main "pc + 4"
eval main symbols player
```

### 4.2 Finding RAM Variables, Watchpoints, and Callers

```sh
# Snapshot a memory space before and after an in-game change, then diff
snapshot before wram
# ... perform action in game ...
snapshot after wram
diff before after

# Search RAM interactively or find byte patterns directly
memfind wram 0x03 0x00 0xFF

# Break or log on memory writes and reads (requires EMUSEN_CAP_DEBUG_READS for reads)
watch write wram 0x007E break
watch read wram 0x007E log pc
resume

# Inspect recent writers, readers, callers, and the live call stack
writers wram 0x007E
readers wram 0x007E
callers main 0x8120
bt
```

### 4.3 Breakpoints, Stepping, Coverage, and Profiling

```sh
# Set an execution breakpoint and continue execution
bp add main 0x8042
resume

# Step instructions or run to a scanline, interrupt, or frame number
step 5
runto scanline 224
runto irq
runto frame 120

# Measure executed code coverage and hottest functions
cov arm
profile arm
# ... play section ...
cov status
profile hottest 20
cov export /Logs/VenusRT/stage1.cov
```

> **Loop scripting tip:** `c` is an interactive alias for `resume`. Inside a `for` or `while` loop, `continue` keeps its standard shell loop-control semantics, so use `resume` or `c` when resuming emulation inside shell scripts.

### 4.4 Inspecting Graphics, Audio, DMA, and Binary Dumps

```sh
# Render an 8x8 tile or tilemap as ASCII art, and inspect sprites/palettes
tile vram 0x1000 4
tilemap vram 0x0000
sprites
pal 0

# Inspect audio voices and isolate channels cleanly
channels
mute 3 on

# Inspect DMA and coprocessor histories
dma
cophist
copflow

# Dump raw memory to /Logs/<CoreName>/<file> or reload it into a writable space
dump wram 0x0000 0x2000 ram_dump.bin
load wram 0x0000 ram_dump.bin
```

---

## 5. Extending DianaSh (`dianash_module.h` and `emusen_session.h`)

New shell commands never need to be hardcoded into `Platform/dianash`. Modules compile independently against `Platform/dianash/include/dianash_module.h` (ABI **v1.1**) — in Rust via `Platform/dianash-module` or in C/C++.

### 5.1 Module Manifest and Execution Contract

A module implements two entry points: `manifest()` and `run(call)`:

```json
{
  "manifest": 1,
  "id": "example.inspect",
  "version": "0.1.0",
  "services": [
    { "name": "emusen.session", "major": 1, "min_minor": 0, "required": true }
  ],
  "commands": [
    { "name": "inspect", "read_only": true, "usage": "  inspect  read-only example" }
  ],
  "aliases": [],
  "jobs": [],
  "settings": [],
  "approvals": false
}
```

- **Zero storage coupling:** Modules never link `Platform/galaxia`, `Platform/crystal`, or `Platform/library` directly. Every host capability arrives as a versioned service table requested in `services` and fetched during `run(call)` via `call.service(name, major)`.
- **Missing service handling:** If a required service is unavailable in the current host (for example running a debugger module in standalone `dianash` without a live session), DianaSh reports `DIANASH_SERVICE_MISSING` (`-266`) cleanly.
- **Read-only enforcement:** Commands marked `"read_only": true` remain usable even when the host attaches a read-only `emusen.session` table (`EMUSEN_CRYSTAL_SERVICE_READ_ONLY`).
- **Structured answers:** `run(call)` completes by calling `call.answer(...)` with stdout, stderr, and an exit status (`Answer::ok`, `Answer::fail`, or `Answer::fault`).

### 5.2 Session Service Table (`emusen_session.h` v1.3)

Modules requesting `"emusen.session"` major `1` receive a borrowed `emusen_session_v1` function table exposed by `emusen_crystal_session_service`. The table exposes:

- **Run control (`EMUSEN_SESSION_RUN_*`):** Pause, resume, step (`n`), step over, step out, run to address/interrupt/scanline/frame.
- **Inspection & watches (`EMUSEN_SESSION_WATCH_*`, `EMUSEN_SESSION_CONDITION_*`):** Memory spaces, registers (CPUs and hardware units), disassembly, write/read/both watchpoints, conditional or log-only breakpoints.
- **Analysis (`EMUSEN_SESSION_COVERAGE_*`, `PROFILE_*`, `COUNTERS_*`, `TELEMETRY_*`):** Coverage bitmaps (arm, mark, query, export/import/merge), instruction profiler (`hottest`, `reset_stack`), per-address hardware counters (reads/writes/executes/uninitialized reads), and live telemetry.
- **Cheats (`EMUSEN_SESSION_CHEAT_*`):** Add, list, enable, disable, remove, clear, master toggle, import, decode, and cheat database status.

Callers check optional minor-version entries using `EMUSEN_SESSION_HAS(table, field)` against the table's leading `size` field.

---

## Related Documentation

- [Platform C Interfaces](Platform_C_Interfaces.md) — Complete catalog of all 9 Platform C headers (`dianash.h`, `dianash_module.h`, `emusen_session.h`).
- [Save Media and Game Packages](Save_Media_and_Game_Packages.md) — Architecture behind `pkg`, `chd`, `rom`, `memcard`, and `scrape`.
- [Testing and Verification](Testing_and_Verification.md) — Using the `emusen.testing` studio and `emusen.dev` checks.
- [Docs/man/](../man/) — Individual Unix-style manual pages for every DianaSh command.
