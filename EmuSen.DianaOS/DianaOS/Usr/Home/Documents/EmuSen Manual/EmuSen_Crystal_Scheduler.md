# EmuSen.Crystal — the core-agnostic timing scheduler

*This revision: Venus now schedules against Crystal (`VenusCore.Schedule.cs`), digest-clean. The event decomposition and the PPU-as-lagging-device work are still ahead — see §7.*

*Previous revision: initial, mechanism only.*

---

## 1. Why this exists

Before Crystal there was no scheduler. `VenusCore.RunFrame()` was an imperative per-scanline loop with a fixed service order and events written as `if (_currentScanline == SOME_CONSTANT)`. That works, and it is fast, but it costs three things:

- **Timing that games control at runtime cannot be expressed.** `$4207`/`$4208` write `HTime`, the horizontal position an IRQ should fire at. `InterruptController` stores it and **nothing ever reads it** — the H-only IRQ path fires every scanline unconditionally. There is nowhere in a hardcoded loop to put "fire at a position the game chose two instructions ago".
- **Devices are serviced in whole-scanline slices.** `sa1.Run(cpuCycles)` runs ~1364 master clocks of SA-1 before the S-CPU steps again, so a handshake *within* a scanline cannot interleave.
- **Event ordering is a comment rather than a property.** The HDMA-before-render rule is correct, and the long comment explaining it exists because it was once wrong and produced a visible band in an A Link to the Past rain scene.

It also blocks the thing that motivated it: moving the PPU off the emulation thread. The measured ceiling for that is **27–35%** (`Venus_PPU.md` §13.4), and the design it needs — a device that lags and is forced to catch up when something reads its state — is a scheduler feature, not a renderer feature.

**EmuSen is committed to more than one core.** `EmuSen/Cores/` reserves ten Nintendo consoles plus Atari, Microsoft, NEC, Sega and Sony. Every one of them needs the same mechanism, so it is built agnostic from the start rather than extracted from Venus later.

---

## 2. Devices — `IClockedDevice`

```csharp
long Clock { get; }              // where this device has reached on the master timeline
void SyncTo(long masterClock);   // run forward until Clock >= masterClock
```

That is the entire contract. A device's `Clock` **may lag** `Scheduler.Now`, and that is the point: a lagging device is one that has not done its work yet, which is legal right up until something observes it. `SyncTo` must never run a device backwards.

---

## 3. Events — an `int` and a time

```csharp
void OnScheduledEvent(int eventId, long now);
```

The scheduler only ever knows `(long when, int id)`. It has no idea what 225 means. A core defines an event enum and gives the ids meaning in one `switch`. Nothing about scanlines, NMI or vblank appears anywhere in Crystal.

**Ids are an array index, not a heap key.** `Scheduler` holds one `long[]` of pending times indexed by event id, and finding the next event is a linear scan. This is deliberate:

- Event ids are dense small ints from a core-defined enum, and there are few of them.
- A scan over ≤32 longs beats a priority queue at this size, and allocates nothing.
- It serialises as a plain `long[]` (§6).

The constraint it buys: **one pending instance per event id.** Scheduling an id that is already pending *moves* it rather than adding a second. That is exactly the semantics wanted for "the next H-IRQ" or "the next scanline start", and it is what makes `HTime` changing mid-frame a one-line reschedule.

An `int` rather than a delegate keeps dispatch allocation-free and closure-free.

---

## 4. The timeline — `RunUntil`

`RunUntil(deadline)` fires every event due at or before `deadline`, then leaves `Now` exactly at `deadline`. Four rules, all pinned by tests:

- **Time order**, earliest first.
- **Ties break on the lower event id.** Which order wins matters far less than it being fixed — the `framesum`/`audiosum` digests that verify every change here depend on the dispatch order being reproducible.
- **A past-due event is legal and fires at the current instant.** An instruction can overshoot a deadline (`Venus_CPU.md` §8.5a already carries that spill today), so scheduling slightly behind `Now` must not be an error, and `Now` never runs backwards.
- **Firing clears the event first**, so a handler reschedules itself naturally to become recurring.

**The runaway guard.** A handler that reschedules itself at the current instant forever would hang the frame with no diagnostic. After `MaxDispatchesPerRun` (4096) dispatches without time advancing, `RunUntil` throws naming the offending event id. A hang in a timing core is far more expensive to debug than an exception.

---

## 5. Clock ratios — the part that is universal

Every console runs chips at unrelated rates. The SNES master clock is 21.477 MHz, the SPC700 runs at 1.024 MHz, the S-DSP at 32 kHz. Converting between them is where subtle bugs live, and this project already has the scar: `Venus_APU.md` §2.8 is a fixed bug where *"CPU→SPC700 cycle-scaling used the wrong master-clock ratio, making audio play ~1.3× too fast."*

`ClockRatio` is an exact reduced rational with a named constructor (`FromHz(deviceHz, masterHz)`) because the direction is the part that gets inverted by mistake. `MasterTicksFor` rounds **up**, so a deadline never lands short of the ticks asked for.

`ClockAccumulator` is the one that matters. Truncating conversion loses the remainder on every call, and a per-scanline conversion repeated 15,734 times a second accumulates real drift. The accumulator carries its remainder, so a long run of small steps totals exactly what one big conversion would — asserted directly in `ClockRatioTests`.

Use `ClockRatio.ToDevice` for a one-off. Use `ClockAccumulator` for anything repeated, which in practice is every device.

---

## 6. Save states

Crystal is a leaf (§8) and so cannot carry the core's `[SkipInState]` attribute. Instead the scheduler exposes its state as a flat span — `StateLength`, `CaptureState`, `RestoreState` — of `Now` followed by the pending event times. A core serialises that `long[]` however it already serialises everything else, and the device list and handler (which are object graphs, not state) never enter a save state at all.

**Adopting Crystal in a core will invalidate that core's existing `.state` files.** `StateSerializer` has no field-name tagging, so a changed layout loads without error and produces a machine that runs but renders nothing (`Venus_PPU.md` §13.5). The three saved Venus states are already in that condition, so nothing is lost — but it is worth stating rather than discovering.

---

## 7. How a core teaches it, and what is not built yet

A core supplies exactly three things, all of them code:

1. **Devices** — `AddDevice` for each chip that consumes time.
2. **Events** — an enum of ids, and one `switch` that gives them meaning and reschedules them.
3. **Sync points** — an explicit `Sync(device)` at each place the core reads state a lagging device owns. For Venus that is roughly six sites in `MemoryBus`: `$213E` (STAT77), and VRAM/CGRAM/OAM writes. These cannot be inferred, and having them written down is a feature — each one is a place where a device stops being allowed to lag, which is the list you want when debugging a threading bug.

**Venus does this in `VenusCore.Schedule.cs`** (a partial of `VenusCore`, so the boundary work keeps direct access to the state it already used). `Scheduler.Now` is now the authoritative master clock: `_lineCycles` is `Now - _lineStartClock` rather than a running total, which means §8.5a's scanline-overshoot carry falls out of the subtraction instead of needing its own statement. The conversion is digest-clean — byte-identical `framesum` and `audiosum` on all 12 ROMs.

**It has exactly one event, and that is a finding rather than a shortcut.** Every event in the old `RunFrame` fired at a scanline boundary, and end-of-line-N is separated from start-of-line-(N+1) only by the scanline increment. So a conversion that changes no behaviour necessarily collapses to one `ScanlineBoundary` event; splitting HDMA, render, vblank and the IRQ check apart means moving them to their true hardware times, which *changes* digests by definition. **This commit buys the timeline, not the decomposition.**

What that unlocks is the point: with a master clock in place, the PPU can become a device that lags. That is the 27–35% (`Venus_PPU.md` §13.4), and it is the next step. `HTime` comes after, as its own commit, because it will legitimately change digests for any game using mid-scanline IRQs and that diff must not be tangled with a refactor that is supposed to change nothing.

**Known gap: the timeline is not in the save state.** `VenusCore.SaveState` serialises `Cart`/`Cpu`/`Bus`/`Spc700`, not `VenusCore` itself, so `Scheduler.Now` and `_lineStartClock` are not written — exactly as `_lineCycles` and `_scanlineStarted` never were. A loaded state therefore resumes on the running instance's timeline rather than the saved one. That is pre-existing behaviour and not a regression (the digests confirm it), but `Scheduler.CaptureState`/`RestoreState` (§6) exist for when it is fixed properly.

---

## 8. Why it is a leaf, and where the layering actually stands

`EmuSen.Crystal.csproj` has no `ProjectReference` and no `PackageReference`, the same constraint `EmuSen.Galaxia` carries and for a stronger reason: every core sits above Crystal, so anything it depended on upward would be inherited by every future core. It is also what keeps the abstraction honest — nothing in Crystal *can* name a scanline, an NMI or a `SnesButton`, so "agnostic" is enforced by the build rather than by discipline.

Crystal deliberately does **not** live in `EmuSen.DianaOS`. DianaOS is the debug shell; putting the thing that drives emulation inside it would mean every future core needed the shell to execute a single cycle. What DianaOS should get is the *observability* — a read-only view of pending events, device clocks and how far each device lags, rendered as a `sched` command beside `coretop` and `regs`. That dependency runs the correct way, and it is the payoff worth having: one scheduler view that works identically on every core, for free. The existing `ScanlineObserver`/`runto` hooks become scheduler observers on the same basis, which is how `runto` starts working on a second core without anyone writing code for it.

**A caveat about the README's layering claim.** The project overview states that the emulation core is a pure library and that debug tooling "sits above it and depends on it one-directionally." That is not true today: `EmuSen.csproj` references `EmuSen.DianaOS`, and 41 files in the core use DianaOS types — `MemoryBus.Breakpoints` is a `DianaOS.Var.BreakpointRegistry`, `Cartridge.RegisterFlow` is a `DianaOS.Var.RegisterFlowRegistry`, `Spc700`'s trace is a `DianaOS.Lib.DebugTools.RepeatCollapsingTrace`. Most are `[SkipInState]` nullable hooks so the coupling is shallow, but the project reference is real and the core will not build without the shell. Crystal does not add to that and is not the place to fix it; untangling it is its own piece of work, and doing it *while* rewriting the timing core would make a digest-clean refactor impossible to verify.
