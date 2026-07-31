**EmuSen** — a multi-system emulator built around a Unix-like shell

## DESCRIPTION

**EmuSen** is a from-scratch emulator written in C# / .NET 10, built against published
hardware documentation rather than ported from an existing project. It differs from
conventional emulators in one structural way: **the emulator is not a window with a debug
menu attached — it is a machine you log into.**

That machine is **DianaOS**, a genuine Unix-like shell embedded in the emulator, and it is
the primary interface to every piece of emulated hardware. Registers, memory spaces, the
tilemap, the sprite table, the palette, the DSP's voice mixers — all of it is reachable as
text, from a prompt, with pipes and redirection and control flow, the same way you would
inspect a running process on any Unix system.

Most emulators treat introspection as a feature bolted onto playback: a hex viewer here, a
sprite viewer there, each a separate GUI panel with a fixed set of things it can show you.
EmuSen inverts that. Playback is one thing the machine does; the shell is how you talk to
it. Because the shell composes — `mem` into `grep` into a file, `watch` inside a `for`
loop — the set of questions you can ask the hardware is not enumerated in advance by
whoever wrote the debugger.

The second consequence of that design is **multi-system by construction**. The shell does
not know what a SNES is. It talks to an interface called `IDebugTarget`, which describes
*memory spaces, registers, sprites, palettes and breakpoints* in the abstract. A new
console core implements that interface and inherits the entire toolchain — the shell, the
scripting harness, the watchpoint system, the screenshot and digest tooling — without a
line of it being rewritten.

**What is actually built today: one core, the SNES.** Every other console listed below is
a reserved, empty folder with a documentation stub. The multi-system claim in this document
is a claim about *architecture*, not about a shipping feature list — the debug toolchain is
core-agnostic and has been used in anger for months, but it has never been proven against a
second implementation. That proof is the project's stated long-term goal, not a checked box.

## DIANAOS

DianaOS is a bash-alike, not a command prompt with a fixed verb list. It has:

- quoting, `$VAR` and `$(...)` expansion
- pipes, and `>` / `>>` / `<` redirection to real files
- `;`, `&&`, `||`, and `if` / `for` / `while`
- history recall, `man` pages for every command
- a coreutils subset — `ls`, `cd`, `grep`, `awk`, `sed`, `find`, `xxd`, `nano`, …
- a sandboxed, Unix-shaped filesystem tree walled to the project directory

On top of that base sit the hardware verbs: `mem`, `regs`, `disasm`, `watch`, `bp`,
`sprites`, `pal`, `layers`, `vramsheet`, and `coretop` — an htop-style live dashboard of
the running machine.

The same interpreter is reachable three ways: on the terminal that launched **Hotaru**,
through **Mistress**'s `Settings → DianaOS Console…` window, and — critically — as a
*script* fed to **Pharaoh**, the headless harness.

Pharaoh runs the real core with no window, no audio device and no human, driven by the same
commands the interactive shell accepts. It is how most debugging in this project actually
happens: a bug is reproduced as a deterministic script, and a fix is proven with
`framesum` / `audiosum` — output-identity digests that show a renderer or mixer change
altered exactly the games it was meant to and nothing else, across the whole ROM library,
rather than being eyeballed on one frame.

## ARCHITECTURE

Strictly layered; each layer depends only on the ones below it.

| Project | Role |
|---|---|
| `EmuSen` | The emulation core — CPU, PPU, APU, memory, save states, resampling. A pure library: no `Main`, no window, no frontend knowledge |
| `EmuSen.DianaOS` | The shell, `IDebugTarget`, and every debug command. **Core-agnostic** |
| `EmuSen.Cauldron` | Small realtime-provider abstractions the debug layer polls |
| `EmuSen.Serenity` | Shared presentation — the Avalonia/Skia frame control, shader pipeline, graphics settings |
| `EmuSen.Mistress` | The fuller Avalonia GUI frontend |
| `EmuSen.Hotaru` | The console-first Avalonia frontend |
| `EmuSen.Pharaoh` | The headless scripted harness |
| `EmuSen.Tomoe` | CLI runner for ground-truth CPU/hardware test vectors |
| `EmuSen.WiseMan` | The xUnit test suite |

The layering is enforced in practice, not merely described. The PPU exposes a small
`IWriteObserver` hook and has no idea a watchpoint exists; the debug layer implements that
interface and supplies the meaning. The memory bus references no debug type at all. The
core can be compiled, tested and run with the entire shell absent.

Cores live under `EmuSen/Cores/<Manufacturer>/<Codename> - <Console>/`, namespaced to
match — the SNES is `EmuSen.Cores.Nintendo.Venus.*`. Hardware documentation mirrors that
tree exactly under `Etc/Man pages/Hardware/`, so a core's reference notes sit at the same
path as its code.

Adding a console therefore means: implement the core in its reserved folder, implement
`IDebugTarget` over it, and the shell, the harness, the watch system, rewind/fast-forward
(already core-agnostic, an XOR-delta chain over opaque state) and the frontends all work
against it unchanged.

---

## CORES

Cores are codenamed after *Bishoujo Senshi Sailor Moon* characters, grouped by
manufacturer. Nintendo, as the project's origin point, gets the heroes; every other
manufacturer gets a villain faction. Codenames govern folders and namespaces only — classes
and log output still say `Snes65816Disassembler` and `"SNES"`, because that is what the
hardware is called.

**Nintendo — Sailor Guardians** (`Cores/Nintendo/`)

| Console | Codename | State |
|---|---|---|
| SNES | **Venus** | **Implemented** — the only working core |
| NES | **Moon** | Reserved — the planned second core |
| Game Boy / Color | **Mercury** | Reserved |
| Game Boy Advance | **Jupiter** | Reserved |
| Nintendo 64 | **Mars** | Reserved |
| Virtual Boy | **Saturn** | Reserved |
| DS | **Luna** | Reserved |
| 3DS / New 3DS | **Artemis** | Reserved |
| GameCube | **Uranus** | Reserved |
| Wii | **Neptune** | Reserved |
| Wii U | **Pluto** | Reserved |

Virtual Boy being **Saturn**, Guardian of Death and Destruction, is a joke rather than a
coincidence.

**Sega — Dark Kingdom** (`Cores/Sega/`) — Master System **Endymion**, Game Gear
**Jadeite**, Genesis **Beryl**, 32X **Nephrite**, Saturn **Zoisite**, Dreamcast
**Kunzite**.

**Sony — Black Moon Clan** (`Cores/Sony/`) — PlayStation **Diamond**, PlayStation 2
**Sapphire**, PlayStation 3 **Rubeus**, PSP **Esmeraude**, PS Vita **Wiseman**.

**Atari — Death Busters** (`Cores/Atari/`) — 2600 **Eudial**, 5200 **Mimete**, 7800
**Tellu**, Lynx **Viluy**, Jaguar **Cyprine & Ptilol** (canonically inseparable, hence one
slot rather than two).

**NEC — Dead Moon Circus, Amazon Trio** (`Cores/NEC/`) — PC Engine / TurboGrafx-16
**Tiger's Eye**, SuperGrafx **Fish Eye**, PC-FX **Hawk's Eye**.

Shadow Galactica is the one major faction still unassigned, held for whichever
manufacturer is added next.

**Ordering.** NES (**Moon**) is next, and is the milestone the whole architecture was built
toward: it is the first real test of whether `IDebugTarget` is genuinely generic or merely
asserted to be. Nothing beyond it is scheduled — the reserved folders are a naming and
layout commitment, not a promise of delivery dates.

---

## STATUS

The SNES core runs real commercial games; several boot and play. Very few have been
verified end to end, and per-title ratings live in `EmuSen_Games_Tested.md`, which is the
source of truth over any summary. Known gaps include 65816 decimal mode, save states
lacking a version header, and scanline- rather than per-dot timing granularity (a
deliberate, documented tradeoff).

Linux x64 is launch-tested. Windows and macOS builds are produced and structurally correct
but have not been executed; macOS builds are unsigned.

**No ROMs are included, and none will be.**

---

## SEE ALSO

`README.md` — building, running, publishing, and the full status tables.

Inside the emulator, `man <command>` documents every DianaOS verb. The project's own
documentation set lives under
`EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/` (overview, debugging tools
reference, save states, audio sync, settings, games tested, roadmap) and
`EmuSen.DianaOS/DianaOS/Etc/Man pages/Hardware/` (per-console hardware notes, including
full root-cause writeups for real bugs found and fixed).

## LICENSE

GPL-3.0 — chosen so anything built on this stays open, matching how the project itself was
built from openly-published documentation.
