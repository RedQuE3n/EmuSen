# Venus (SNES) — CPU (65816)

Covers `Cores/Nintendo/Venus - SNES/Cpu/`: the 65816 core (`Cpu.cs`), addressing modes (`Cpu.AddressModes.cs`), the execution opcode table (`Cpu.OpcodeTable.cs`), opcode implementations (eight `Cpu.Opcodes.*.cs` files, one per instruction category — §1), and the standalone disassembler. See `Man pages/Hardware/README.md` for what this doc set is and how it relates to the rest of `Man pages/`.

The disassembler's own quirks and caveats are documented in depth in `Man pages/EmuSen_Debugging_Tools_Reference_v5.md` §3.7 (it's a debug-tooling consumer as much as a CPU component) — this page only covers what's specific to it as a *second, independent* opcode table living alongside the execution one.

---

## 1. Structure

`Cpu` is a `partial class` split by concern, not by convenience: `Cpu.cs` holds core state and the fetch/execute loop, `Cpu.AddressModes.cs` holds every `AddrXxx` addressing-mode delegate, `Cpu.OpcodeTable.cs` holds the 256-case dispatch switch plus the per-opcode name and cycle tables, and every `OpXxx` operation lives in one of eight `Cpu.Opcodes.*.cs` files, split by instruction category the way most 65816 references group them:

| File | Covers |
|---|---|
| `Cpu.Opcodes.System.cs` | NOP, BRK/COP/RTI, WAI/STP, MVN/MVP |
| `Cpu.Opcodes.Stack.cs` | PHP/PLP, PHA/PLA, PHX/PLX, PHY/PLY, PHB/PLB, PHK, PHD/PLD, PEA/PEI/PER |
| `Cpu.Opcodes.LoadStoreTransfer.cs` | LDA/LDX/LDY, STA/STX/STY/STZ, every T__ register transfer, XBA |
| `Cpu.Opcodes.Arithmetic.cs` | INC/DEC (A, memory, X, Y), CMP/CPX/CPY, ADC/SBC |
| `Cpu.Opcodes.Logical.cs` | ORA/AND/EOR, TRB/TSB, BIT/BIT-immediate |
| `Cpu.Opcodes.Shift.cs` | ASL/LSR/ROL/ROR, accumulator and memory forms |
| `Cpu.Opcodes.Branch.cs` | JMP/JML/JSR/JSL/RTS/RTL/BRA/BRL, every conditional branch |
| `Cpu.Opcodes.Flags.cs` | CLC/SEC/CLI/SEI/CLV/CLD/SED, REP/SEP, XCE |

This used to be one 1229-line `Cpu.Opcodes.cs` — split for the same reason `DebugCommandProcessor` and the PPU renderer were: a monolith the size of "every CPU operation in one file" stopped being something you could hold in your head at once, and none of the split boundaries needed to touch any actual instruction logic to fix that. `Cpu.OpcodeTable.cs` and `Cpu.AddressModes.cs` were deliberately **not** split the same way — see the note at the end of this section.

Each `case` in `Cpu.OpcodeTable.cs`'s `Dispatch()` pairs an addressing-mode call (computes an effective address, may consume operand bytes) with an operate call (does the actual work at that address) — mirroring how real 6502/65816 opcode references describe instructions as (addressing mode × operation). This was a table of `Instruction` structs holding two delegates until §9 replaced it with direct calls.

**Why `Cpu.OpcodeTable.cs` stays one file.** Unlike the `Op*` method bodies, its 256 entries are already terse one-liners — the file is long because there are 256 opcodes, not because any individual entry is hard to read. This exact table got a dedicated verification pass against oxyron.de (§7) before being trusted, so any re-derivation of all 256 entries risks introducing a transcription error into a table that's already been verified once — which is why §9's conversion was done by a generator over the old file rather than by hand, and proven equivalent before being kept. `Cpu.AddressModes.cs` (243 lines) was left alone for a simpler reason: it's already a single, cohesive category (every `AddrXxx` delegate) and isn't especially large to begin with.

---

## 2. `LastInstructionPC`/`LastInstructionPB` — why these exist alongside `PC`/`PB`

`PC`/`PB` are live: `Fetch8()` advances `PC` past the opcode/operand bytes almost immediately, well before that instruction's actual side effects (a memory write, say) happen. Debug tooling that wants "which instruction just wrote this value" (e.g. `Dma.cs`'s `LogSourceAddrWrite`, or a write watch's context string) needs the PC/PB *as they were when this instruction started*, not the live ones.

**This was a real, found-the-hard-way bug**, not a preemptive design choice: every DMA source-address write trace was showing the *same* next-instruction bytes regardless of which actual `STA` had just fired, because by the time the write's side effect ran, `PC` had already moved on to the next instruction. `LastInstructionPC`/`PB` are snapshotted at the top of `Step()`, before `Fetch8()` runs, specifically to fix this.

---

## 3. NMI / IRQ entry sequences

Both `Nmi()` and `Irq()` run the real 65816 interrupt sequence: push `PB` (native mode only — emulation mode's stack can't hold it), push `PC`, push `P`, clear `D`, set `I`, then jump to the vector. The handler is expected to end with `RTI`.

- **NMI is non-maskable** — always services regardless of the `I` flag, and always wakes a WAI-halted CPU (see §4). Vector: `$FFEA` native / `$FFFA` emulation.
- **IRQ is maskable** — only actually jumps to the vector if `I` is clear, matching real hardware holding off IRQs while that flag is set. `Irq()` returns whether it actually fired, so the caller (which decides *when* to attempt this, based on H/V-IRQ timer conditions — see `Man pages/Hardware/Venus_Memory.md` §4) knows whether to also acknowledge the pending condition. Vector: `$FFEE` native / **`$FFFE` emulation — shares BRK's vector**, a real, documented 6502/65816 quirk, not a typo.
- **WAI wakes on any interrupt condition, masked or not** — only whether it's actually *serviced* (jumps to the vector) respects the `I` flag. Both `Nmi()` and `Irq()` clear `_waitingForInterrupt` unconditionally, before any masking check.

---

## 4. WAI / STP

Verified via 6502.org/apprize.best's "Programming the 65816" excerpt and the NESDev WAI thread.

- **WAI** halts instruction fetch/execute until any interrupt condition wakes the CPU (see §3's masking note — NMI always services, IRQ wakes but only services if unmasked; if masked, execution just resumes at the instruction after WAI with nothing serviced).
- **STP** stops the CPU entirely. Real hardware only wakes on a hardware RESET — modeled as `Cpu.Reset()` clearing `_stopped`; no other instruction or interrupt clears it. This matches real hardware, where STP is meant for power-down, not a resumable pause.

Neither fetches or executes anything while active — `Step()` just returns idle cycles (3 for STP, 2 for WAI) so the caller's per-scanline cycle budget still advances.

---

## 5. Addressing modes — notes on the less obvious ones

- **`(sr,S),Y`** (`AddrStackRelativeIndirectY`): the 16-bit pointer lives on the stack (always Bank 0, same as plain stack-relative), combined with `DB` and indexed by `Y` — same combine-then-index pattern as `(dp),Y`. Verified against the documented 65816 opcode matrix (oxyron.de, cross-checked against softpixel's table) for the eight `0x_3` opcodes that use it (`ORA`/`AND`/`EOR`/`ADC`/`STA`/`LDA`/`CMP`/`SBC (sr,S),Y`).
- **`BRL` / `AddrRelativeLong`**: 16-bit signed offset, unlike `BRA`'s 8-bit (`AddrRelative`) — that's the entire reason `BRL` exists, to reach branch targets too far away for the short form.
- **`JMP (abs)` / `AddrAbsoluteIndirect`**: the 16-bit pointer lives in bank 0; the 16-bit target read from it combines with the *current* program bank.
- **`JML [abs]` / `AddrAbsoluteIndirectLong`**: the 16-bit pointer lives in bank 0; a full 24-bit target (offset low, offset high, bank) is read from it — the bank comes from the pointer table itself, not the current PB.
- **`JMP/JSR (abs,X)` / `AddrAbsoluteIndexedIndirect`**: the pointer address is `(abs + X)` inside the *current* program bank, and the 16-bit target read from it also stays in the program bank.
- **`[dp]` / `AddrDirectIndirectLong`**: 24-bit pointer read from the direct page, no `Y` indexing (contrast with `AddrDirectIndirectLongY`, which does index by `Y` after reading the same 24-bit pointer shape).

---

## 6. `BIT` — immediate vs. non-immediate is a real behavioral split, not just an addressing-mode difference

`BIT` (non-immediate): `Z` from `A & mem`; `N` and `V` are copied from the top two bits of the memory operand (bits 7/6 in 8-bit mode, 15/14 in 16-bit). `BIT` immediate (`#$nn`) is the documented special case: it **only** affects `Z`, leaving `N` and `V` alone — there's no "memory operand" to read flag bits from when the operand is the instruction stream itself. `OpBIT`'s two branches implement this distinction directly.

---

## 7. Opcode table verification status

The *execution* opcode table (`Cpu.OpcodeTable.cs`) got a dedicated verification pass against oxyron.de before being trusted — completed 236→256/256, verified against oxyron.de, caught one cross-reference typo (from the original project handoff). This is the table `Step()` actually dispatches through; treat it as verified.

The *disassembly* table (`Snes65816Disassembler`, separate by design — see this page's intro) has **not** had the equivalent treatment, per its own header comment and `EmuSen_Debugging_Tools_Reference_v5.md` §3.7 — no way to build/run this project from wherever it's being edited to cross-check it the same way. Built carefully against the same standard 65816 opcode matrix, spot-checked against a couple of real captured instruction bytes, but short of a real verification pass. Treat disassembly output as a solid first draft, not a verified reference, until it gets one.

---

## 8. Dynamic cycle-penalty tracking — `Step()` returns real master clocks, not an opaque "CPU cycle" count

Investigated chasing an SMW audio report ("coin chime sounds off, breaks in the title music") — see `Venus_APU.md` §2.9 for the full audio-side story. This section covers the CPU-side change: replacing a single flat "1 CPU cycle = 6 master clocks" assumption (see `Venus_Memory.md` §3.1's older text, and `Venus_APU.md` §2.8) with real, per-instruction, region-aware master-clock accounting.

### 8.1 Why `inst.Cycles` alone was never going to be accurate

`Cpu.OpcodeTable.cs`'s `Instruction.Cycles` is a fixed `byte` per opcode — a correct total *access count* (verified against oxyron.de, §7), but real 65816 hardware doesn't run every access at the same speed. Each of those accesses costs 6, 8, or 12 *master clocks* depending purely on which memory region it touches (Nocash's fullsnes "Memory Access Speed" table) — WRAM and most of the `$8000-FFFF` ROM window are 8 (SlowROM) or 6 (FastROM, region-dependent on the `$420D` MEMSEL bit), the old-style joypad registers (`$4000-41FF`) are a slow 12, and the `$2000-3FFF`/`$4200-5FFF` register windows are a fast 6 regardless of ROM speed. A single flat multiplier applied to the whole machine can only ever be correct for games that happen to match whichever one rate it assumes.

### 8.2 `MemoryBus.GetAccessSpeedCycles(address)`

New method implementing that region-speed table directly, including the `$420D` bit-0 FastROM enable (previously entirely unhandled — writes to it silently fell into the open-bus fallback, so this project always behaved as SlowROM regardless of what a game actually wrote there). `MemoryBus.FastRomEnabled` is the live state; banks `$40-$7D`/`$7E-$7F` are always slow, banks `$00-$3F`/`$80-$BF`'s `$8000-FFFF` window and all of `$C0-FF` depend on the bit, everything else is a fixed speed by offset range.

### 8.3 `Cpu.Step()`'s master-clock conversion

Per instruction: `bytesFetched` (however far `PC` actually advanced — opcode + operand bytes) is charged at the *opcode's own* region speed; any remaining cycles (`inst.Cycles + _addrModeExtraCycles - bytesFetched` — the instruction's real memory read/write plus internal/dummy cycles) are charged at the *addressing target's* region speed. This correctly splits the common case where a ROM-resident instruction reads or writes WRAM (or vice versa) at two different real speeds, rather than pretending the whole instruction ran at one rate.

**Known, accepted approximation**: implied/register-only opcodes and branches have no separate memory target (`AddrImplied` returns 0, `AddrRelative*` returns a same-bank branch destination) — their "remainder" cycles get charged at whatever `GetAccessSpeedCycles(targetAddr)` says about that address instead of the opcode's own bank. For `AddrImplied` specifically this coincidentally lands on WRAM's speed (8), which happens to equal SlowROM's speed too — so this only actually diverges from correct for a handful of pure-register (no real stack/memory access) opcodes running from FastROM code, a narrow residual accepted rather than threading a same-bank flag through every implied-addressing opcode for it.

### 8.4 New addressing-mode-level penalties (`_addrModeExtraCycles`)

Two of the four commonly-documented dynamic 65816 cycle penalties are now modeled, reported by the addressing-mode methods themselves (in `Cpu.AddressModes.cs`) via a per-instruction side-channel field rather than a per-opcode table entry, since both are properties of the addressing mode, not the opcode using it:

- **+1 for a nonzero Direct Page register low byte** (`ChargeDirectPagePenalty`) — every direct-page-relative addressing mode (`AddrDirectPage[X/Y]`, `AddrDirectIndirect[X/Y]`, `AddrDirectIndirectLong[Y]`) charges this when `(D & 0xFF) != 0`.
- **+1 for indexed addressing crossing a page boundary** (`ChargePageCrossingPenalty`) — `AddrAbsoluteX`/`AddrAbsoluteY`/`AddrDirectIndirectY` (`(dp),Y`) charge this when adding the index carries into a different high byte. Applied unconditionally on a crossing; real hardware's actual rule is narrower (no penalty for a 16-bit index in some cases, and store instructions don't always take it) — accepted as a simplification rather than threading opcode-level read/write and index-width flags through every addressing mode for it.

**Not implemented**: the third commonly-documented penalty, +1 for a 16-bit (`M`/`X`=0) memory operand on non-immediate addressing modes, is already captured automatically for *immediate* addressing (`AddrImmediateM`/`AddrImmediateX` fetch an extra byte when 16-bit, which `bytesFetched` picks up for free) but not for memory-operand forms (e.g. `LDA $1234` reading 2 bytes instead of 1 when `M=0` doesn't change the opcode's fixed 3-byte length). Itemizing which of the ALU/load/store/compare opcode family needs this, per operand width (`M` for accumulator-sized ops, `X` for index-sized ones), across every non-immediate addressing mode they support, is a real remaining gap — deferred as a larger, mechanical per-opcode tagging exercise rather than folded into this pass.

### 8.5 Downstream unit changes

`VenusCore.CyclesPerScanline` changed from `227` (a CPU-cycle-unit figure back-derived assuming FastROM's 6-master-clock rate uniformly) to `1364` (real master clocks per scanline: 341 dots × 4). The CPU→SPC700 pacing conversion (`Venus_APU.md` §1) dropped its `*6` factor accordingly, since `Cpu.Step()`'s return value is already real master clocks now. `MemoryBus`'s H-blank approximation (`Venus_Memory.md` §1.5) and `Dma.PendingCpuCycles`'s drain-side conversion (`Venus_Memory.md` §3.1) were rescaled to match — see those sections.

### 8.5a Scanline overshoot must be carried, not discarded

`VenusCore`'s scanline loop is `while (_lineCycles < CyclesPerScanline)`, so the instruction that crosses the boundary always overshoots — `_lineCycles` ends the scanline somewhere in `1364 .. 1364 + (that instruction's master clocks - 1)`, never exactly `1364`. The scanline-start block then used to reset `_lineCycles = 0`, **throwing that overshoot away**. Every scanline therefore consumed slightly *more* than 1364 master clocks of emulated machine time while the counter pretended it had consumed exactly 1364.

It now carries the remainder instead (`_lineCycles -= CyclesPerScanline`), the same explicit-remainder discipline `_spc700CycleRemainder` already used a few lines below for the SPC700 conversion — the SPC700 side had it right, the scanline side did not.

**Size of the error:** average overshoot is roughly half an instruction (~10-11 master clocks), across 262 scanlines ≈ **2,800 extra master clocks per frame, ~0.79%**. Emulated frames were that much *longer* than real hardware's 262 × 1364 = 357,368.

**How it was measured.** Audio production is the most sensitive observable available for this, since the DSP emits exactly one sample per 32 SPC700 cycles and SPC700 cycles come straight from the master-clock count. `EmuSen.Pharaoh`'s `audiodump` verb over short runs (short enough that `AudioBuffer`'s resync can't fire and truncate the count — see `EmuSen_Settings_Reference.md` §2):

| frames | before (samples) | per frame | after (samples) | per frame |
|---|---|---|---|---|
| 4  | 4266  | 533.3 | 4254  | 531.75 |
| 8  | 8572  | 535.8 | 8508  | 531.75 |
| 12 | 12864 | 536.0 | 12762 | 531.75 |

Before, the per-frame figure was both too high *and* not constant (overshoot varies with whichever instruction happens to straddle each boundary). After, it is exactly 531.75 every time, matching the 531.80 predicted by 357,368 / 21 / 32 for the standard master/21 SPC700 approximation. True hardware is 32000 / 60.0988 = 532.457; the residual 0.13% is the `/21` approximation itself (real ratio 20.974), not this bug.

This is a general timing-accuracy fix, not only an audio one — every frame was 0.79% long, so anything paced off emulated frame time (IRQ timing, game speed, DMA budgets) inherited it. It also removes the systematic audio over-production that was driving `AudioBuffer`'s periodic resync discard; see `EmuSen_Settings_Reference.md` §2 for why that discard is still the wrong mechanism regardless.

### 8.5b Two independent crystals, and why frame pacing belongs on the core

The SNES has **two unrelated oscillators**: the main/video clock at 21.477272 MHz and the APU's own at 24.576 MHz (divided to 1.024 MHz for the SPC700). Nothing derives one from the other, which has three consequences this emulator now models explicitly.

**1. The frame rate is 60.0985 Hz, not 60.** `21477272 / (262 x 1364) = 60.0985`. Both frontends previously hardcoded `TimeSpan.FromSeconds(1.0 / 60.0)` for their emulation-loop pacing — a flat 0.164% slow, and duplicated in two places where nothing tied it to the core actually being run. `ICore.FrameRateHz` now reports it, `VenusCore` computes it from `TotalScanlines`/`CyclesPerScanline` rather than restating a magic number, and both `GameWindow` and `MainWindow` pace off that. A future NES/Game Boy core reports its own (the Game Boy's ~59.727 Hz is not the SNES's) with no frontend change. `EmulatorSession` passes it through, falling back to NTSC before a ROM is loaded.

**2. The SPC700 conversion uses the exact ratio.** Master clocks converted to SPC700 cycles by `/ 21`, the near-universal approximation; the true figure is `21477272 / 1024000 = 20.9739`, so `/21` runs the APU **0.125% slow**. It's now exact integer math against both clock constants, with the same remainder carry as before:

```
long scaled = (long)cpuCycles * ApuClockHz + _spc700CycleRemainder;
_spc700CycleRemainder = (int)(scaled % MasterClockHz);
Spc700.CycleBudget    += (int)(scaled / MasterClockHz);
```

`cpuCycles` never exceeds a few dozen, so `cpuCycles * 1024000` cannot overflow `long`, and the remainder is always `< MasterClockHz` and fits `int`.

**3. Measured effect.** Audio production per real second against the 32000 Hz the output device consumes — the sensitive observable described in §8.5a:

| | production/sec | drift |
|---|---|---|
| original | 32160.0 | **+0.500%** |
| + §8.5a scanline carry | 31905.0 | -0.297% |
| + core-rate frame pacing | 31957.4 | -0.133% |
| + exact clock ratio | 31997.4 | **-0.008%** |

A ~60x reduction. The sign flip in the middle row matters: fixing only the scanline carry would have turned a systematic *over*-production into a systematic *under*-production, trading periodic discards for periodic underruns. Both halves are needed.

**What this does not fix.** Because the two crystals are genuinely independent, real hardware drifts too — a residual mismatch is physically correct, not a bug to be eliminated. Any emulator therefore still needs a mechanism to absorb it continuously. `AudioBuffer`'s current discard-based resync is the wrong such mechanism; see `EmuSen_Settings_Reference.md` §2.

### 8.5c PAL support — both crystals move, the dot clock doesn't

§8.5b's two constants are NTSC's. A PAL console differs in exactly two of them, and the reason the change stayed small is that the *third* number everything else is built on doesn't move at all:

| | NTSC | PAL |
|---|---|---|
| Master clock | 21477272 Hz | **21281370 Hz** |
| Scanlines/frame | 262 | **312** |
| Master clocks/scanline | 1364 | 1364 (unchanged) |
| Active display lines | 224 | 224 (unchanged) |
| Frame rate | 60.0985 Hz | **50.0070 Hz** |

Both machines run 341 dots x 4 master clocks per scanline; PAL simply spends more scanlines per frame in vblank at a slightly slower clock. So `CyclesPerScanline` stays a `const`, the whole per-scanline CPU/HDMA/render loop is untouched, and vblank still starts at line 225 (`InterruptController.AutoJoypadScanline`) because the active display is 224 lines either way. Only `TotalScanlines` and `MasterClockHz` became per-instance fields, set by `LoadRom` from `Cart.Region` (`Venus_Memory.md` §2.5).

**`FrameRateHz` and the SPC700 conversion both fell out for free**, which is the payoff for §8.5b having derived them from the constants instead of restating magic numbers. `FrameRateHz` is still `masterClock / (totalScanlines * CyclesPerScanline)` and now yields 50.0070 for PAL with no other change; frontend pacing follows automatically, as §8.5b designed it to for a hypothetical future core. The SPC700 remainder-carry math is unchanged apart from reading the instance field — and it is *correct* that it changes, because the APU crystal genuinely does not: the SPC700 still runs at 1.024 MHz on a PAL console, so the same real time buys the same number of APU cycles while the CPU's master-clock count per frame goes up.

**What this does not model.** PAL games' extra 50 vblank lines are real time the CPU gets to work in, and that is modelled; what isn't is that the *display* on a real PAL set is 239 lines tall, with games choosing either 224-line output (letterboxed, what nearly everything including DKC2 does) or true 239-line mode via `SETINI` bit 2 (§10 of `Venus_PPU.md`, stored but not implemented). A 239-line PAL game would render its bottom 15 lines missing. Frame *pacing* at 50 Hz is honest, but this is emulating a PAL machine's timing, not a PAL machine's overscan.

**Validated by** `EmuSen.WiseMan/Memory/ConsoleRegionTests.cs` (country-byte classification, both frame rates to 3 decimal places, the `STAT78` bit in both directions, and that the region bit doesn't disturb the PPU2 version nibble sharing that byte), and end-to-end by Donkey Kong Country 2 — the European dump that motivated the work — now booting past the region lockout into playable gameplay (`EmuSen_Games_Tested.md`).

### 8.6 Validation

Re-run against the SingleStepTests/65816 ground-truth suite (§7) after this change — no regressions (state-only checks, since that suite's vectors don't assert cycle counts, only resulting registers/memory). This change's actual timing effect was cross-checked separately by re-measuring SPC700 audio pacing (`Venus_APU.md` §2.9): the per-frame master-clock budget turned out to already be correctly calibrated either way (see that section for why), so this is a genuine general CPU/PPU timing accuracy improvement, but it did **not** turn out to be the fix for the audio symptom that motivated it.

---

## 9. Opcode dispatch — direct-call `switch`, not a delegate table

`Cpu.OpcodeTable.cs` used to build a 256-entry `Instruction[]`, each entry a struct holding a `Func<uint> AddrMode`, an `Action<uint> Operate`, a `Name` and a `Cycles`. `Step()` loaded the struct and made two indirect delegate calls per instruction. It now calls `Dispatch(opcode)`, a flat `switch` over `0x00`–`0xFF` whose cases call the addressing-mode and operate methods directly; `Name` and `Cycles` moved to two `static readonly` arrays (`OpcodeNames`, `OpcodeCycles`), the first used only by the verbose trace.

Three things fell out of the shape change, independent of speed:

- **The per-instruction string comparison is gone.** The old `Step()` tested `inst.Name == "NOP/UNK"` on *every* executed instruction to detect an unimplemented opcode. That test had been dead since the table reached 256/256 (§7) — every entry is explicitly assigned, so the placeholder name never survives initialisation. The `switch`'s `default:` arm now carries that job and costs nothing on the taken paths. `OpUnknown` was the placeholder's operate delegate and is deleted; the SPC700 keeps its own separate `OpUnknown`, which is still live.
- **512 delegate objects per `Cpu` are no longer allocated.** Construction cost only, but the SA-1 builds a second `Cpu` (`Venus_SA1.md` §4.1), so it was paid twice.
- **The table is now in opcode order rather than registration order**, which makes it checkable against a printed 65816 matrix top to bottom. The old organic ordering (its inline comments read "0xE6 is what the ROM halted on") is preserved as history here rather than in the file.

### 9.1 Notes that used to live in the table's inline comments

- **`0x87` is `STA [dp]`**, the 24-bit long-indirect store — a 3-byte pointer read from the direct page, no `DBR` involved — *not* `0x92`'s `STA (dp)` (16-bit pointer + current `DBR`). It was wired to `AddrDirectIndirect` instead of `AddrDirectIndirectLong` at one point. Real-world effect: any code doing `STA [dp]` with `DBR` != the pointer's own bank byte silently wrote to the wrong bank. Common in practice — ALTTP's `AddReceivedItem` sets `DBR` to its own bank via `PHK`/`PLB`, then writes an item's equipment-table byte through a pointer whose bank byte is `$7E`. Found via the LttP "lamp appears then never enters inventory" investigation.
- **`0x42` is `WDM`**, officially reserved for future expansion. Every real 65816 treats it as a 2-byte NOP: fetch the opcode, fetch and discard one operand byte, do nothing. Games don't use it intentionally, but some copy-protection/anti-emulation checks have historically probed for correct handling.
- **The `(sr,S),Y` family (`0x_3`) cycle counts** were resolved against two sources: oxyron.de lists `EOR`'s as 6 where every other opcode in the family — and softpixel's independent table — lists 7. Treated as a typo in that one source rather than a real asymmetry, since nothing about `EOR`'s addressing differs from the others.

### 9.2 Validation

The conversion was generated mechanically from the old table rather than retyped (§1), then the two builds were compared directly: all **43 ROMs** in `Usr/Home/Roms` run 1200 frames headless, with a SHA-256 taken every 200 frames over the full save-state image (every serialized CPU/PPU/APU/DMA register, WRAM and VRAM — so a divergence is caught long before it could reach the screen) concatenated with the framebuffer. All 43 digests are identical between the delegate-table and `switch` builds.

Two harness traps worth recording, since both initially looked like real regressions:

- **Battery saves leak between runs.** `Cartridge` writes `.srm` files next to the executable, so the second run of a game that touches SRAM during those 1200 frames boots from different state than the first. Nine ROMs "diverged" purely from this. Any A/B of this kind must delete the `Saves` directory before *every* run, not once at the start.
- **A stable hash across two runs does not prove determinism** if both runs already read the same leftover `.srm`. The re-check that appeared to confirm determinism was doing exactly that.
- **Save-state images embed absolute filesystem paths.** `Cartridge.SavePath` is an ordinary serialized string field, so publishing the same build to a different output directory changed all 43 digests at once while the emulation was byte-for-byte unchanged. Both sides of a state-hash comparison must run from the same directory. This is also a latent bug in its own right, independent of any benchmarking: `LoadState` writes that string back, so a state file shared between two machines (or two install locations) restores the *other* machine's save path. Noted here, not fixed.

### 9.3 Measured effect, and where the real headroom is

Throughput (mean ms per `RunFrame()`, headless, 2400 frames after 600 warmup, interleaved A/B, .NET 10 Release):

| ROM | delegate table | `switch` |
|---|---|---|
| LttP | 2.626 / 2.670 | 2.561 / 2.642 |
| DKC | 2.430 / 2.438 | 2.377 / 2.389 |
| SMW | 3.219 / 3.116 | 3.108 / 3.092 |
| FFVI | 3.095 / 3.078 | 3.040 / 3.005 |
| CT | 2.286 / 2.237 | 2.242 / 2.264 |

About **2%** — real but small, and worth having mainly for the string comparison and the allocations rather than the dispatch itself.

**The dispatch table was not what dynamic PGO was buying.** Running with `DOTNET_TieredPGO=0` costs ~16% *with either dispatch style* (LttP 17.1% delegate / 15.5% switch; DKC 15.5% / 17.8%), so profile-guided devirtualisation is earning that ~16% somewhere other than the opcode table. The obvious suspect was `ICpuBus` — §10 tests that and rules it out.

This also settles the NativeAOT question that prompted the change: AOT stays ~10–12% *slower* than the tiered JIT with the `switch` in place (LttP +10.4%, DKC +10.8%, SMW +12.2%), for the same reason — it has no dynamic profile, and the `switch` did nothing to reduce the dependence on one. See `EmuSen_Project_Overview_v2.md` for the AOT trimming hazards (`StateSerializer` silently produces a 20-byte save state under AOT) that make it a correctness question as well as a speed one.

## 10. The `ICpuBus` interface call is not the bottleneck — measured, not assumed

§9.3 left an open hypothesis: `_bus.Read8`/`Write8`/`GetAccessSpeedCycles` are `ICpuBus` interface calls made several times per instruction with exactly two runtime implementations, which looks like exactly the thing dynamic PGO's guarded devirtualisation would be worth ~16% on. It was tested and it is wrong.

The experiment gave the CPU a concrete, statically-typed fast path — a `MemoryBus?` field set in the constructor (`bus as MemoryBus`, non-null for every CPU except the SA-1's own), with all 151 bus call sites routed through three `AggressiveInlining` helpers that branch to the concrete reference when it is present. `MemoryBus` was sealed and its `Read8`/`Write8` made non-virtual at the same time (§10.1) so the concrete call is a direct, statically-bound one.

Result, headless mean ms per `RunFrame()`, 2400 frames after 300 warmup:

| ROM | PGO on: `switch` / + fast path | PGO off: `switch` / + fast path |
|---|---|---|
| LttP | 2.421, 2.459 / 2.452, 2.373 | 2.781 / 2.783 |
| DKC | 2.376, 2.363 / 2.351, 2.336 | 2.716 / 2.751 |
| CT | 2.205, 2.114 / 2.139, 2.132 | 2.702 / 2.670 |
| SMW | 3.086, 3.106 / 3.115, 3.080 | — |
| FFVI | 3.049, 2.991 / 2.984, 3.061 | — |

Signs flip between repetitions in the PGO-on column, so that is noise around zero. The PGO-off column is the decisive one: hand-devirtualising the bus, with no profile available to do it dynamically, is worth **nothing** (2.783 vs 2.781, 2.751 vs 2.716, 2.670 vs 2.702). If the interface call were where PGO's 16% lived, removing it statically would have recovered most of that gap with PGO off. It recovers none of it.

The reason is inlining, not dispatch. `MemoryBus.Read8` → `ReadInternal` is a long bank/offset decode chain far past any inlining budget, so both the manual fast path and PGO's guarded devirtualisation end at the same place: a direct call to a method that is called, not inlined. Removing the interface indirection saves one load and one indirect branch against a callee that costs far more than that. Whatever PGO is actually earning is elsewhere — most plausibly profile-driven basic-block layout inside `Dispatch`'s 256 arms and inside that same decode chain, which no source-level change replicates.

The fast path was therefore reverted; it added a null test to every bus access for no measurable gain. **Do not re-propose bus devirtualisation** — generic-over-bus-type, struct bus shims, or another concrete fast path — without first showing a profile that contradicts the PGO-off column above.

### 10.1 What was kept: `MemoryBus` is sealed, and the flat test bus stands alone

`MemoryBus.Read8`/`Write8` were `virtual` for exactly one reason: `FlatTestMemoryBus`, the 16 MB flat-RAM double used by the `SingleStepTests/65816` harness (§8.6, `EmuSen.Tomoe`), subclassed `MemoryBus` and overrode them to bypass all SNES bank/register decoding. A test double was making two of the hottest methods in the emulator overridable.

`FlatTestMemoryBus` now implements `ICpuBus` directly instead. It never used anything from `MemoryBus` other than the two methods it overrode, so the inheritance was buying nothing — and it forced the double to construct a `Cartridge`, which reads a real file, which is why the harness used to generate a throwaway 32 KB dummy ROM into the temp directory just to satisfy a constructor. That is gone too. `GetAccessSpeedCycles` returns a flat 6 and `TakePendingDmaCycles` returns 0, both unused: the suite discards `Step()`'s return value and checks only registers and memory.

With the last subclass gone, `MemoryBus` is `sealed` and both methods are ordinary non-virtual instance methods. Measured as neutral (the numbers above are with the sealing in place), so this is a structural cleanup, not a performance change.

The vectors themselves are third-party data this repo does not ship, so nothing in `EmuSen.WiseMan` was covering this adapter. `Validation/Cpu65816SingleStepTargetTests.cs` now does, pinning the property that matters: every address including `$2100`/`$4210` register space behaves as plain RAM, in both directions, through real executed instructions.

---

## 11. Interpreter dispatch: threaded dispatch / "help the branch predictor" is not the next win

Raised as a follow-up to §9 and §10: would giving the interpreter some concept of branch prediction help? Recorded here so it isn't re-derived.

**What the technique is.** For an interpreter it means threaded (replicated) dispatch: instead of one `switch` at the top of the loop, every handler ends with its own copy of the dispatch jump, giving the CPU's indirect predictor ~256 separate branch sites that can learn opcode-pair correlations, rather than one site that jumps everywhere. It needs computed `goto`. C# has no equivalent — a `switch` compiles to a single jump-table indirect branch, and the only way to replicate it is to tail-duplicate all 256 cases into every handler.

**The lever is already pulled.** Profile-driven basic-block layout is the compiler-side form of helping branch prediction, and .NET's dynamic PGO does it automatically. §9.3 measured it: `DOTNET_TieredPGO=0` costs ~15–16%, and NativeAOT — which has no profile at all — is ~11% slower (§9.3, and it silently corrupts save states besides). That is the largest single effect measured across this whole investigation, and it is on by default. A manual scheme would be competing for a slice of something already mostly captured.

**The hardware has moved.** The classic "threading is ~2× faster" results (Ertl & Gregg) are Pentium 4 / PowerPC era. The development machine here is a Zen 4 (Ryzen 7 7700X) whose TAGE-class indirect predictor recovers most of the single-dispatch-site penalty on its own.

**The ceiling is small anyway.** Per `Venus_PPU.md` §13.1, `cpu+spc700` is 0.45–0.98 ms of a 1.4–3.2 ms frame for every ROM except KSS (3.22 ms, which is the SA-1's second CPU — `Venus_SA1.md` §4.1). Per-scanline PPU compositing is the larger half and is straight-line pixel code, not dispatch. Eliminating interpreter dispatch *entirely* would cap out near 30% of frame time, on a core already running 3.5–12× real time.

**What is worth doing in this family.** The SPC700 still has the shape the 65816 had before §9: `Spc700.Step()` loads `SpcInstruction inst = _instructions[opcode]` and makes two delegate calls per instruction. Converting it to a direct-call `switch` is mechanical and worth roughly what the 65816 conversion was (~2%, plus the dead per-instruction string compare and the per-instance delegate allocations). That is a cleanup with a known small payoff — not a branch-prediction strategy.

**If this gets re-opened**, the decisive measurement is an actual mispredict rate (`perf stat -e branches,branch-misses,instructions,cycles`). `perf` is not installed on the dev machine; `perf_event_paranoid` is 2, so it needs installing but not root to run. Do that before writing any code.
