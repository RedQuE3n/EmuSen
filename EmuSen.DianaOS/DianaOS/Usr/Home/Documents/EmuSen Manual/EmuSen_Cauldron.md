# EmuSen.Cauldron — the core-agnostic telemetry layer

*This revision: split `ICoreTelemetry` out of DianaOS's `IDebugTarget` and moved the five snapshot value types into this assembly, so a dashboard depends on Cauldron rather than on the debugger; measured the per-frame refresh cost and settled it (§5.1). Previous revision: providers only (`IRealtimeProvider`, `PollingProvider`, `HistoryProvider`).*

---

## 1. What Cauldron is

Cauldron answers one question: **what is a running core doing right now**, for anything that wants to watch without touching it. `coretop` is the reference consumer — the htop-shaped dashboard this exists to feed — but the two GUI coretop windows, `regs`, `sprites`, `pal` and the harness all read the same way.

It is a leaf assembly. It references nothing (`EmuSen.Cauldron.csproj` has no `ProjectReference` at all) and knows nothing about DianaOS, `ICore`, any console, or any specific core. That is the whole point: a consumer that only wants to *observe* a core should not have to depend on the debugger to do it.

Two rules define it:

- **Read-only.** Nothing in this assembly writes to a core. Writes — memory pokes, channel muting, cheat application, breakpoints — live on DianaOS's `IDebugTarget` (§3.1).
- **Snapshot-published.** A consumer never reads live core state. It reads an immutable snapshot that the emulation thread published at a moment of its choosing (§2).

### 1.1 Why it is a separate assembly

The same reasoning that made `EmuSen.Crystal` a leaf: *agnostic mechanism, core-owned policy*. Cauldron owns the vocabulary (what a register value, a sprite, a load meter looks like) and the publication mechanism. Each core owns the policy — which registers are worth reporting, what "load" means for its hardware, how a native palette format becomes RGB. Cauldron never encodes a per-core answer, and there is no declarative per-core profile, for the same reason Crystal rejected one.

---

## 2. The provider contract

`IRealtimeProvider<T>` supplies one kind of information as an immutable snapshot.

- **`Current`** — the last published snapshot. Never blocks, never touches live core state, safe from any thread at any time, including concurrently with a `Refresh()` on another thread.
- **`Refresh()`** — pulls a fresh snapshot from the live core and publishes it. **Only ever call this from the thread that owns the core.** It is the single member that touches live, mutable core state; calling it from two threads at once, or concurrently with whatever else mutates that state, defeats the entire purpose.

`T` must be an immutable snapshot — a readonly struct, or a reference to a list never mutated after being handed to `Refresh`. `Current` is published without a lock, so a consumer reading a half-mutated `T` would be exactly the race this exists to prevent.

### 2.1 PollingProvider

Wraps a plain "go read the live core" `Func<T>`, so a core supplies a delegate instead of writing its own `IRealtimeProvider` boilerplate. Constrained to `T : class` — every current use is an `IReadOnlyList<...>` snapshot — so `Current` can be published with `Volatile.Write` and read with `Volatile.Read`: a plain reference swap, not a lock, which is what makes the "never blocks" contract above true even under concurrent access.

The constructor takes an `initial` value and reads it once, rather than leaving `Current` null until the first `Refresh()`. A provider should never hand back "nothing" just because the thread that owns the core has not got round to refreshing yet.

### 2.2 HistoryProvider

Also remembers the last `capacity` snapshots. `PollingProvider` answers "what is the machine doing now", which is what a live dashboard wants; this answers "what was it doing fifty frames ago", which is what an investigation into a transition wants — when did the display go blank, when did the coprocessor stop, what changed either side of it.

Deliberately a separate type rather than a flag on `PollingProvider`: keeping history costs a retained reference per `Refresh`, so a consumer that only reads `Current` should not pay for a ring it never looks at.

- `capacity` is a hard bound. The ring overwrites its oldest entry rather than growing, so a run of any length costs the same fixed memory.
- `RefreshCount` is how many times `Refresh()` has run — the clock the staleness signal is measured in. A caller wanting real frame numbers pairs it with its own frame counter.
- `RefreshesSinceChange` is how long `Current` has been sitting still. It answers "when did the coprocessor stop updating" directly, without storing or diffing any history. Always 0 when no comparer was supplied, since nothing can be judged unchanged.
- `GetHistory()` copies out under a lock rather than handing back the live ring, so a caller iterating it cannot have entries overwritten underneath it by a `Refresh()` on the emulation thread. That makes it the one member here that can block — briefly, on an uncontended lock — which is why it is a method rather than a property that looks as cheap as `Current`.

`SnesDebugTarget` is the only current user, for coprocessor registers; `cophist` is the command on top of it.

### 2.3 ListEqualityComparer

Element-wise equality for the `IReadOnlyList<T>` snapshots every provider publishes. Exists so `HistoryProvider`'s staleness signal has something meaningful to compare: snapshots are freshly built each `Refresh`, so reference equality would report "changed" every single time and the signal would always read 0. It compares whatever `TItem`'s own `Equals` says, so register, sprite and load snapshots all work without it knowing what any of them are.

This is why `DebugRegisterValue` implements `IEquatable<>` (§4.1).

---

## 3. `ICoreTelemetry` — the read surface

Everything a live dashboard can read from a running core, and nothing that writes to one.

| Member | Notes |
| --- | --- |
| `CoreName` | Short display name — "SNES", "NES". |
| `FrameCount` | Monotonic, once per rendered frame. The shared "what moment is this" reference for correlating a screenshot, a log line and a register dump against the same instant, whichever core is running. |
| `MaxSprites` | Real OAM capacity (128 on the SNES, 64 on the NES), or 0 when a core models no fixed limit. Lets a dashboard draw "N/max" as a real percentage bar instead of a context-free count. |
| `CpuRegisters` / `VideoRegisters` | Grouped separately because almost every console draws this exact line in its own documentation, which keeps a generic register panel able to show two sections without knowing which registers belong to which half. |
| `ApuRegisters` | The sound coprocessor's own registers plus the CPU↔APU communication ports, both directions. Added investigating a Super Metroid boot hang where the only way to see SPC700 state at all was raw verbose-trace text. Empty list when a core has no distinct sound coprocessor. |
| `CoprocessorRegisters` | Whatever cartridge coprocessor is present — SA-1, SuperFX GSU, NEC DSP. **Reads must be side-effect-free:** the real register windows acknowledge interrupts (`$3031` on the GSU) and advance transfer handshakes (the NEC DSP's DR/SR), so implementations read dedicated `Debug*` views rather than routing through the chip's own `ReadRegister`. Same trap `APURAM` avoids by wrapping raw RAM instead of `Spc700.Read8`. Empty for the common no-coprocessor cartridge. |
| `Sprites` / `Palettes` / `AudioChannels` | See §4. |
| `HardwareLoad` | Per-subsystem load bars. An empty list means this core models no timing breakdown; `coretop` skips the section rather than drawing fake bars. |
| `RefreshProviders()` | Republishes every provider above. Host-driven, emulation thread only — §5. |
| `RenderTileSheet()` | Tile/character memory decoded to an RGBA image, for a headless harness to write straight to disk without a window. Deliberately not "VRAM sheet": an NES core's CHR pattern tables are not VRAM in the SNES sense, but "some tile memory decoded to a sheet" is the same shape across consoles. A core with nothing analogous returns a 0x0 buffer. |
| `RenderPaletteSwatch()` | Same idea for palette memory. SNES CGRAM and an NES core's completely different palette RAM are both just "N colors resolved to RGB" once decoded. 0x0 when a core has no palette memory. |
| `GetAudioSamples()` | Non-destructive copy of the currently buffered output samples — `Queue<short>.ToArray()` on the SNES side, specifically so it never steals samples from a live playback consumer of the same queue. Interleaved 16-bit PCM plus its sample rate, so a harness can write a `.wav` without knowing anything about the source sound chip. |
| `GetSummaryText()` | Free-text escape hatch for whatever is not modeled structurally — lets a new target be useful on day one. |

The 0x0 return on the two render methods is a real contract, not a defensive nicety: both GUI coretop windows call them unconditionally and let their `ToBitmap` helper turn an empty buffer into a null image.

### 3.1 What stayed in `IDebugTarget`, and why

`IDebugTarget` (DianaOS, `Lib/IDebugTarget.cs`) now extends `ICoreTelemetry` and keeps everything else. A core implements `IDebugTarget` exactly as before and satisfies `ICoreTelemetry` automatically.

The split is not read-versus-write alone. What stayed is everything shaped like a **debugger**:

- **The twelve registries** — `WatchRegistry`, `BreakpointRegistry`, `CoverageRegistry`, `CallStackRegistry`, `LabelRegistry`, `AccessCounterRegistry`, `FreezeRegistry`, `RegisterFlowRegistry`, `DmaLogRegistry`, `CheatRegistry`, `FrameLogRegistry`, plus `IExpressionContext`. These are stateful subsystems that are neither pure-read nor pure-write, they total roughly 2,750 lines, and they are legitimately DianaOS's own. Moving `IDebugTarget` wholesale would have relocated the debugger into Cauldron rather than building an information provider.
- **Memory spaces** — `GetMemorySpaces()` and `IDebugMemorySpace`, which carries `Write(int, byte)` and `IsWritable`. Splitting that into read and write halves is a larger change than this one, and no dashboard needs it.
- **Disassembly and decoders** — `Disassemble`, `DecodeTilemapEntry`, `DecodeTilePixels`, `TilemapEntryStride`, `ClassifyStaticReference`. These take `IDebugMemorySpace`, so they cannot move until it does.
- **The remaining writes** — `SetChannelMuted`, `ApplyCheats`.
- **Console-shaped descriptors** — `InterruptVector`, `PhysicalAddress`, `BreakCondition`, `DebugDmaChannel`, `DisassembledInstruction`, `DebugCpu`.

The test of whether the split is real: `EmuSen.Hotaru` and `EmuSen.Mistress`'s `CoretopWindow` both take `ICoreTelemetry`, not `IDebugTarget`. `CoretopCommand` still takes `IDebugTarget` because it draws a tilemap section, and it lives in DianaOS anyway.

---

## 4. The snapshot types

All five are immutable readonly structs with no dependency on anything outside Cauldron.

### 4.1 `DebugRegisterValue`

One named register or flag. `BitWidth` drives formatting — pad to 2 hex digits for an 8-bit register, 4 for 16-bit — and the type is kept generic rather than typed per-register because different CPUs have wildly different register sets. The 65816 has PB/DB/D and an E flag the 6502 does not; a 6502 target simply reports a different list through the same shape.

`IEquatable<>` is implemented so `EqualityComparer<T>.Default` takes the fast path rather than reflection-based `ValueType.Equals`, which boxes. This is compared per-register per-frame by `HistoryProvider`'s staleness signal (§2.2), so the boxing would be real.

### 4.2 `DebugSpriteInfo`

One sprite/OBJ entry, shaped generically enough to cover the SNES's OAM low+high table split and the NES's flatter 4-byte-per-sprite OAM. A sprite viewer needs a rectangle, a tile and palette reference, and flip/priority flags, regardless of how the hardware stores them.

### 4.3 `DebugPaletteInfo`

One palette's colors, already resolved to display-ready RGB, so a palette widget never needs to know a console's native format — SNES BGR555, the NES's 64-color master palette. Each target converts its own format once, here.

### 4.4 `DebugAudioChannelInfo`

One channel or voice. The SNES reports its 8 S-DSP voices through this shape and the NES its 5 APU channels (2 pulse, triangle, noise, DMC), even though BRR sample playback and simple waveform generators have nothing in common.

`Level` is a plain 0-100 scale rather than the SNES's native 0-2047 envelope range, so a generic viewer needs no core's internal units. `Info` is a free-text escape hatch for what is too core-specific to model — ADSR stage, sample source, pitch. Same "structured where cheap, free-text where not" split `GetSummaryText()` uses.

Built diagnosing a "part of the music is missing" report, where the toolchain could only inspect the final mixed output or a KeyOn event as it happened — neither answers "is voice N active right now, and what is its envelope doing".

### 4.5 `DebugLoadInfo`

One named "how hard is this working" meter. `Percent` is 0-100, already normalized against whatever the core considers full — typically the wall-clock budget of one native frame.

Worth being precise about what this measures: on the SNES it is **emulator cost**, not guest hardware utilization. `ReadHardwareLoadLive` normalizes real millisecond timings against a 60fps frame's ~16.67ms budget and clamps to 100, since a frame running behind can otherwise report over 100% and look like a rendering bug in a bar only meant to reach "full". The type has no field distinguishing emulator cost from guest utilization, so a reader will conflate them; emulator cost is the more useful number and the one every core can produce.

Both cores publish real bars: `CPU+SPC700`/`PPU`/`HDMA` on the SNES, `CPU+APU`/`PPU` on the NES (`Moon_Debug.md` §3.2). The NES published an empty list until 2026-08-04, which is why `coretop` drew no load section on it at all — the empty list is the real "this core models no breakdown" signal, and a core that genuinely has none should still publish it rather than fake zeroes.

---

## 5. Refresh cadence, and who calls it

**The host calls `RefreshProviders()`. The core never refreshes itself.**

This was not always true, and the history is the argument. Until 2026-08-04 `RefreshProviders()` was a defaulted no-op on `IDebugTarget`; the SNES implemented it and every host loop called it once per frame, while the NES never implemented it and instead pushed from its own scheduler via a `MoonCore.FrameRefresh` hook. Both cores worked, by opposite mechanisms, and nothing checked that a core had chosen one.

The push side cannot express the case that broke it. A core's `EndFrame` only fires when a frame *completes*, so it can say "a frame ran" and nothing else. A rewind, a breakpoint halt, a single step and a `loadstate` all move the machine without finishing a frame — which is exactly why both frontends call `RefreshProviders()` in their rewind branch and `EmuSen.Pharaoh` wires it to `OnHalted`. On the NES all of those landed on the empty default, so a rewinding or halted NES kept serving state the machine had already left.

The host knows about frame boundaries *and* about rewind, halt, step and state loads. The core's scheduler only knows the first. Pull is therefore strictly more expressive, and it is now the only rule: `RefreshProviders()` is a required interface member, so a core that forgets it fails to compile rather than going quietly stale. `Moon_Debug.md` §3 records the NES side.

### 5.1 Refreshing unconditionally is measured and settled

Every host calls `RefreshProviders()` once per frame whether or not anything is reading — no dashboard open, DianaOS not running, still eight lists rebuilt on the SNES. That looks like obvious waste, and gating it on a live-reader count was proposed on exactly that reasoning. **It was measured and rejected.** Do not re-propose it without new numbers.

Headless, Release, 600 frames on a synthetic ROM after a 60-frame warm-up:

| Core | `RunFrame` | `RefreshProviders` | Refresh as % of emulation | Refresh as % of the 16.64ms budget |
|---|---|---|---|---|
| SNES | 1.1249 ms | 0.0161 ms | 1.4% | **0.10%** |
| NES | 1.1224 ms | 0.0039 ms | 0.4% | **0.02%** |

A tenth of one percent of a frame. `ReadSpritesLive` walks all 128 OAM entries unconditionally — the parked-sprite `continue` skips the list add, not the iteration — so the dominant cost is already fixed and a real game with a full OAM only changes how much the `List<>` grows. Even tripling the sprite portion leaves this under 0.3% of budget, and the ratio holds on a slower machine because emulation and refresh scale together.

Gating would also have cost more than the plumbing. DianaOS's whole "Diana always live" model rests on read-only lines running immediately **on whatever thread submitted them** (`DianaOSInterpreterScheduler`, `TryGetReadOnlyFastPath`), which is safe precisely because reading `Current` never touches the core. A dormant provider breaks that: a one-shot `regs` or `sprites` has no safe way to refresh from the console thread, and reclassifying those commands as emulation-thread work would make them hang whenever the loop is not ticking — paused, halted at a breakpoint, no ROM loaded. That is exactly when you most want to read registers.

So the unconditional refresh is not a debt. It is the price of the property that makes the read-only fast path safe, and it costs 0.1% of a frame.
