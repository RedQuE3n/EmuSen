# Mercury — the debug target

*Written 2026-08-08, Phase B of the Mercury gameplan. This is the third
`IDebugTarget` implementation the interface has had, after Venus and Moon, and
the first one written when the interface was already stable.*

---

## 1. What this unblocked

Mercury was deliberately absent from `CoreCatalog` and `CoreFactory` for four
days, and the reason was mechanical rather than aesthetic: `CoreFactory.Load`
returns a `CoreBundle` whose `DebugTarget` is non-nullable, so registering the
core before a target existed would have put a `NotSupportedException` behind
every `.gb` file the catalog claimed to handle. A core the file picker offers and
then refuses is worse than one it does not offer.

With the target written, the registration is four small edits and they all landed
in the same change: the catalog descriptor (`.gb` and `.gbc`, both libretro cheat
folders, `MercuryCore.PadButtons`), the `Create` arm, the `Bundle` arm, and
`CheatCodecsFor`.

Three existing tests failed on the registration and all three were right to. They
assert the exact console list — `ConsolesInReleaseOrder`, the input window's tabs,
the cheat window's tabs — and the list genuinely grew. Ordering is by
manufacturer then release year, so the Game Boy (1989) sits between the NES (1983)
and the SNES (1990). That those three broke is the evidence that the catalog is
in fact the single list `CoreCatalog`'s own header says it is.

## 2. Memory spaces

The seven the core already names, unchanged: `ROM`, `VRAM`, `CARTRAM`, `WRAM`,
`OAM`, `HRAM`, `CPUBUS`. `MercuryCore.Spaces.cs` had supplied
`ReadSpace`/`WriteSpace`/`SpaceSize` since the first session, so this half of the
target is a thin wrapper and nothing more.

Two properties are worth stating because they are decisions rather than
transcription:

- **`ROM` is not writable.** A debug write must not corrupt the loaded image;
  Game Genie-style patches go through the read intercept instead (§5).
- **`CPUBUS` is the only space with `HasSideEffects`.** Reading `$FF00` runs the
  joypad matrix and reading `$FF41` recomputes STAT from live PPU position, so a
  bulk tool like `search` must not sweep it. Every other space is a plain array.

`VRAM` and `WRAM` report their *colour* sizes on a colour cartridge — 16K and 32K
rather than 8K and 8K — because `SpaceSize` asks the array. A memory viewer
therefore sees both VRAM banks as one flat space, which is also how the renderer
addresses them, and how the CGB map attributes are reachable at all
(`Mercury_Cgb.md` §3).

## 3. Telemetry

`ICoreTelemetry`'s eight providers are all supplied, three of them with something
other than the obvious content:

- **`ApuRegisters` carries the timer and the cartridge board.** There is no APU
  until Phase C. An empty panel would be a worse answer than a useful one, and
  DIV/TIMA/TMA/TAC have nowhere else to live in this model — they are neither CPU
  registers nor video ones. This is a placement of convenience and should be
  revisited when the APU arrives.
- **`HardwareLoad` is empty**, which the interface documents as "this core models
  none". Mercury does not time its own subsystems the way Moon does, and
  inventing a number would be worse than reporting none.
- **`AudioChannels` is empty** for the same reason as the APU.

### 3.1 Palettes report what the console has, not a fixed count

A DMG has three palette registers — BGP, OBP0, OBP1 — each four shades, so the
target reports three palettes. A CGB has eight background and eight sprite
palettes of real colour, so it reports sixteen. The list length is the console's
answer, not a padded constant, because a viewer showing five empty CGB palettes
next to three real DMG ones would be inventing hardware.

`RenderPaletteSwatch` is built from the same list rather than from a second walk
of palette memory, so the swatch grid and the palette table cannot disagree.

*Added 2026-09-24.* A Game Boy game on a Game Boy Color (the Model setting's
compatibility mode, `Mercury_Model.md` §4) reports three palettes too, because the
game has three palette registers. Each is its register's four shades looked up in
the colour palette the Color renders it through: BGP in background palette 0, OBP0
and OBP1 in object palettes 0 and 1. Before this, the list showed greys the screen
did not, which a test on both engines caught (`Mercury_Model.md` §6.4).

## 4. The disassembler

A second opcode table, independent of the one the CPU executes from, for the same
reason Moon keeps two (`Moon_Debug.md` §5): a disassembler that shares the
executor's table cannot be used to check the executor.

It is written as **256 template strings** rather than as parallel mnemonic and
addressing-mode arrays. The templates carry placeholders — `d8`, `d16`, `a8`,
`a16`, `r8`, `s8` — and the instruction's length is *derived* from which
placeholder appears rather than stored separately. That removes the failure mode
where a table says three bytes and the operand text only consumes two.

Three details in the operand formatting are deliberate:

- **`r8` prints a target, `s8` prints a displacement.** Both are one signed byte
  in the encoding, and treating them alike is the easy mistake. `JR` prints where
  it lands (`JR $0150`); `ADD SP,e8` and `LD HL,SP+e8` print a signed offset
  (`ADD SP,-2`). Printing `-2` as `$FE` in the second case would read as a jump to
  somewhere, and printing an address in the first would read as an offset.
- **`a8` expands to the whole address.** `LDH ($FF44),A` is what a reader needs;
  `LDH ($44),A` makes them do the arithmetic that the mnemonic already implies.
- **`STOP` is two bytes.** It prints its ignored operand because the CPU really
  does consume it, and a disassembly that walked one byte would desynchronise from
  the executor at the next instruction.

The eleven illegal opcodes print `???` and consume one byte. A test asserts that
this set is exactly the set `Cpu` throws on, so the two tables cannot drift apart
on the one question where drift is silent.

`ClassifyStaticReference` answers for the forms whose target is in the bytes:
absolute `JP`/`CALL` with every condition, `RST` (whose target is in the opcode),
relative `JR`, and the four absolute/high-page load-store forms. Everything
register-indirect — `JP (HL)`, `LD (HL),A`, `LD (C),A` — returns null rather than
a guess, which is the same rule the 65816 and 6502 implementations follow.

## 5. Cheats

Two formats, and the split between them is not cosmetic:

- **Game Genie** is a *ROM patch*. Six hex digits patch unconditionally; nine
  carry a compare byte. Its cipher is small but not obvious — the address's top
  nibble is stored last and inverted, and the compare byte is rotated right by two
  and XOR'd with `$BA`. See `Mercury_Cheats.md` §2 for the derivation.
- **GameShark** is a *RAM poke*, eight hex digits, applied to `CPUBUS` every
  frame. Its address is stored little-endian inside the code, which is the one
  thing people get wrong when transcribing by hand.

Making Game Genie work needed one addition to the bus: `RomPatcher`, an
`IRomReadPatcher` hook on every cartridge-routed read, wired to the core's own
`CheatRegistry` through the already-core-agnostic `CheatRomPatcher`. That is the
same wire Moon uses, and it means Game Genie codes work with no debug target
attached at all — a player enabling a cheat should not have to open a debugger.

The two formats cannot be confused by length: 6 or 9 digits is Game Genie, 8 is
GameShark. Note that this is *within* Mercury only. A six-digit code is also the
NES's raw `address:value` form, which is why Mercury does not offer a raw form of
its own — GameShark already fills that role, and adding one would make the
auto-detector ambiguous for no gain.

## 6. What is not implemented

- ~~**No call stack.** `CallStackRegistry` wants a seam in the core's own `CALL`
  and `RET` handling; the seam does not exist yet and the interface allows null.~~
  *Retired 2026-09-24: the seam exists, §7.*
- **No expression context**, no access counters, no freezes, no DMA log. All
  default to null, which the interface treats as "unavailable for this core".
  Without an expression context a breakpoint's `if` condition is always true and a
  logpoint records its expression's text, on both engines.
- **Reads are not reported.** `watch r`, `bp read` and `bp uninit` arm and never
  fire, as on Mars (`Mars_Debug.md` §3).
- **`runto scanline` is not fed**, and `bp when` lists no condition: the core
  detects none.
- **`SetChannelMuted` is a no-op** until there are channels.
- **`DmaChannels` reports HDMA as one row**, and only on a colour cartridge.
  OAM DMA is not in the table: it completes within the write that starts it
  (`Mercury_Memory.md` §6), so there is never a moment at which a channel view
  could show it in progress.

## 7. The seams the registries were waiting for (2026-09-24)

*Written for MercuryRT's stage 6 (`Mercury_Native.md` §8.5), which needed a C# oracle
for every claim it makes, and found that five of the registries' own seams had never
been wired on this core.*

The debugger's claims were written as one abstract set, `MercuryDebugClaims`, and run
first against the unmodified C# core. 13 of 26 failed there. None is a fault in the
machine: each is a registry the target hands out and the core never feeds.

- **No call stack** (§6): `step over`, `step out`, `bt`, `bp depth` and `profile`
  were refused or empty, and `cov funcs` found nothing.
- **Data breakpoints never fired.** `bp write` armed a breakpoint that
  `BreakpointRegistry.NoteWrite` would have honoured, but `OnWrite` fed only the
  watches.
- **`runto irq` and `runto frame` never fired.** Nothing called `NoteInterrupt` or
  `NoteFrame`.
- **A watch went deaf after a reload.** The target set itself as the observer of the
  bus that existed when it was made, and `LoadRom` builds a new bus.

**What was added.**

- **The SM83 reports calls and returns.** `Cpu` has three `[SkipInState]`
  delegates. `CALL` when taken and `RST` report (source, target) after the push. The
  interrupt dispatch reports (return address, vector). `RET`, a taken `RET cc` and
  `RETI` report a return. `MercuryCore.LoadRom` wires them to `CallStack` as
  `NotePush`/`NotePop`. An interrupt is a frame of kind `Irq` and also calls
  `Breakpoints.NoteInterrupt(Irq)`.
- **An interrupt is a frame, unlike on Mars.** The SM83's dispatch pushes a return
  address, and `RETI` pops it. Leaving the dispatch off the stack would make every
  `RETI` an unmatched return. Mars's exceptions push nothing and return by `ERET`, so
  there the stack is calls only.
- **The profiler** is charged once a step, before it runs, as `Coverage.Record` is,
  and the registry's entry-point observer feeds `cov funcs`.
- **`OnWrite` feeds `NoteWrite`**, and `EndFrame` calls `NoteFrame` after the cheats.
- **The core keeps the write observer.** `MercuryCore.WriteObserver` hands it to each
  bus `LoadRom` builds, so a watch outlives a reload.

**What it costs a plain frame.** A C# Mercury frame already asked `ShouldBreak` before
every instruction, and that was 6.7% of the samples (`Mercury_Native.md` §6.3). The
additions are one null-checked delegate call per call, return and dispatch, and a
profiler test per step. They were not timed, because the C# core's frame is not the
product (`Mercury_Native.md` §1).

**The C# core tracks the stack on every frame.** Its delegates are always set, so
even a frame with nothing armed pushes and pops. MercuryRT tracks the stack only in a
frame that something observes (`Mercury_Native.md` §8.5.2). Both are held to that
difference by a named test: `A_call_made_in_a_frame_nothing_observes_is_on_the_stack`
for C#, and `A_plain_frame_leaves_the_stack_where_the_last_observed_frame_left_it` for
MercuryRT.
