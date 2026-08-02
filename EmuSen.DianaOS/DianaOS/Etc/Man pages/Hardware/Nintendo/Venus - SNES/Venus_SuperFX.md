# Venus (SNES) — SuperFX (GSU)

Covers everything under `Cores/Nintendo/Venus - SNES/Coprocessors/SuperFx/`, plus `SuperFxMapper`. This is the second coprocessor this core implements, after the SA-1 (`Venus_SA1.md`).

> **Status: incomplete.** The chip is fully built — memory map, the whole instruction set, the plot hardware — and it executes real GSU code from Yoshi's Island and renders real graphics. It does **not** yet render the game correctly end to end. §10 says exactly what works, what doesn't, and where to pick the debugging back up. Everything in §1-§8 is implemented; treat §9's warnings as live.

---

## 1. What the chip is, and how it gets built

The SuperFX is a custom 16-bit RISC processor — the **GSU** — with sixteen general registers, a 512-byte instruction cache, and hardware for plotting pixels directly into a framebuffer held in Game Pak RAM. It runs at 10.74MHz (GSU-1) or 21.48MHz (GSU-2).

Unlike the SA-1, it is **not** a 65816, so it cannot reuse the `ICpuBus` seam that page describes. It gets its own interpreter.

Detection is the header's **cartridge type**: high nibble `1` with a low nibble of 3 or more (`$13`, `$14`, `$15`, `$1A`), at the LoROM header location. `Cartridge`'s constructor checks it and installs `SuperFxMapper`.

### Game Pak RAM sizing

A SuperFX cartridge declares `$00` in the normal SRAM-size byte (`+$18`); its RAM lives behind the **expansion-RAM byte at header −$03** (`$7FBD`) instead. That byte is also **not to be trusted as the working RAM size**: Yoshi's Island declares `$05` (32KB), but the GSU demonstrably addresses a full 64KB during its intro, and wrapping writes at 32KB corrupts the game into a crash.

So `Cartridge` allocates the **architectural maximum the GSU can address, 128KB**, and remembers the header-declared size separately as `_batteryRamSize` — only that much is written to the `.srm`, so save files stay the size other emulators produce. This is the one place the implementation deliberately ignores a header field, and the reason is empirical rather than documentary.

---

## 2. Running the GSU

### 2.1 Ownership

The GSU and the S-CPU share ROM and Game Pak RAM, and `SCMR` bits 3/4 (`RAN`/`RON`) nominally arbitrate who may touch which. This implementation lets both access freely; no game is known to depend on the arbitration, and enforcing it can only turn a working access into a dropped one.

### 2.2 Timebase

`CLSR` bit 0 picks the clock: **2 master clocks per GSU cycle** for GSU-1, **1** for GSU-2. `VenusCore.RunFrame` hands `SuperFx.Run()` the same master-clock figure `Cpu.Step()` just returned, exactly as it does for the SA-1, and an unspent-clock budget carries between calls.

Per-instruction costs are approximate: 1 cycle for register work, 4-7 for anything touching RAM or fetching immediates. The S-CPU normally *waits* on the GSU (polling `SFR` bit 5, see §3.3), so an inaccurate cost changes emulated speed rather than correctness.

---

## 3. Memory map and registers

Both sides see the same ROM and RAM through the same decode — unlike the SA-1 there is no per-side map. What differs is which banks each reaches it through.

| Address | Contents |
|---|---|
| `$00-$3F/$80-$BF:3000-32FF` | GSU registers and cache |
| `$00-$3F/$80-$BF:6000-7FFF` | Game Pak RAM, 8KB window, one block per bank |
| `$00-$3F/$80-$BF:8000-FFFF` | ROM, LoROM-style |
| `$40-$5F`, `$C0-$DF` | ROM, linear 64KB banks |
| `$60-$7D`, `$E0-$FF` | Game Pak RAM, flat |

Bit 15 of the address is **ignored** in the LoROM view, so `$00:0000` and `$00:8000` are the same ROM byte — that is what lets the GSU point `ROMBR:R14` anywhere in a bank.

The `$6000-$7FFF` window is *packed*, not mirrored: bank `$00` shows RAM `$0000-$1FFF`, bank `$01` shows `$2000-$3FFF`, and so on.

As with the SA-1, **no `MemoryBus` change was needed** — the `MapsAddress` escape hatch and the existing fall-through to `Cartridge.Write8` already cover all of it.

### 3.1 Register file (`$3000-$301F`)

R0-R15 as little-endian pairs. R0 is the default accumulator, **R15 is the program counter**, and R1/R2 are the plot hardware's X and Y. R4 receives the low word of `LMULT`, R6 is `FMULT`'s implicit operand, R11 is `LINK`'s target, and R12/R13 are `LOOP`'s counter and branch target.

### 3.2 Status (`$3030`) and control

`SFR` carries Z (bit 1), CY (2), S (3), OV (4), **GO (5)**, ROM-pending (6), ALT1 (8), ALT2 (9), and the WITH/`B` flag (12), plus IRQ in bit 15. Reading the **high** byte acknowledges the interrupt. `CFGR` bit 7 masks the IRQ line into the S-CPU without clearing the flag.

The rest: `PBR` (`$3034`) program bank, `ROMBR` (`$3036`), `RAMBR` (`$303C`), `CBR` (`$303E`), `SCBR` (`$3038`) screen base in 1KB units, `SCMR` (`$303A`) screen mode, `CLSR` (`$3039`) clock, `VCR` (`$303B`) version — reported as `$04`, a GSU-2.

### 3.3 Starting and stopping

The S-CPU sets up the registers and then **writes R15's high byte (`$301F`), which is the launch trigger** — so it must be written last. Yoshi's Island does exactly this with a 16-bit `STA $301E`. `STOP` clears GO and raises the IRQ; the S-CPU's idiom is `LDA #$0020 : BIT $3030 : BNE -`, spinning until GO drops.

Register writes from the S-CPU are ignored while GO is set.

---

## 4. The instruction set

One byte per opcode. What makes it dense is that most slots have **four meanings**, chosen by prefix flags.

### 4.1 The pipeline, R15, and delay slots

The GSU prefetches one byte. The invariant this implementation maintains is:

> **R15 always names the byte currently sitting in the pipeline.**

That is not a free choice — it is forced by real code. Yoshi's Island opens a loop with `WITH R15 : TO R13`, capturing R15 as the loop's start address. Get the invariant one byte too high and R13 lands *past* the loop's first instruction, so the loop body silently loses its first instruction on every iteration after the first. That was the first real bug this implementation hit, and the symptom was a memory-fill loop writing the wrong register.

Because R15 is a normal register, **every write to it is a jump** — including `TO R15`, `MOVE R15`, `LOOP`, `JMP` and the branches. And because the prefetched byte has already been fetched, **every jump runs one delay-slot instruction before it takes effect**. `_jumpPending` carries that across: after a jump R15 already names the destination, so the next `Pipe()` consumes the delay-slot byte *without* advancing, and only then refills from the target.

Multi-byte instructions in a delay slot fetch their operands from the retargeted R15, which is why real GSU code never puts one there.

### 4.2 Prefixes

`TO Rn` ($10-$1F) sets the destination. `FROM Rn` ($B0-$BF) sets the source. `WITH Rn` ($20-$2F) sets both *and* raises the `B` flag — and while `B` is set, `TO`/`FROM` stop being prefixes and become the real instructions `MOVE`/`MOVES`. `ALT1`/`ALT2`/`ALT3` ($3D/$3E/$3F) set the flags that pick which of a slot's four meanings runs.

All of it is cleared after the next non-prefix instruction, which is why `FROM R1 : ADD R2 : ADD R2` adds R1+R2 into R0 and then R0+R2 into R0.

The four meanings follow a consistent shape: base, ALT1 = a variant operation, ALT2 = the same with a 4-bit immediate, ALT3 = both. So `$5n` is `ADD Rn / ADC Rn / ADD #n / ADC #n`, and `$7n` is `AND / BIC / AND #n / BIC #n`. The subtract slot breaks the pattern: ALT3 there is `CMP Rn`, which sets flags without writing.

### 4.3 The cache

512 bytes, 32 lines of 16. An instruction fetch whose address falls within `[CBR, CBR+512)` comes from the cache, filling the line from ROM or RAM on first touch. `CACHE` points `CBR` at the current address (16-byte aligned) and flushes if it moved; `LJMP` and a `PBR` change flush unconditionally.

The S-CPU can also stage code into the cache directly through `$3100-$32FF`, marking a line valid once its last byte arrives. **The index there is `offset - $3100`, not `offset & $1FF`** — `$3100 & $1FF` is `$100`, so the mask form silently writes every byte 256 positions off. That was the second real bug, and it is exactly the kind a unit test catches and a framebuffer does not.

---

## 5. Data access

### 5.1 Game Pak RAM

`RAMBR` supplies the bank for every RAM access: `LDW`/`LDB`/`STW`/`STB` through a register, `LM`/`SM` with a full 16-bit address, and `LMS`/`SMS` with a byte operand that indexes **words**, so the address is `operand << 1`. `SBK` re-stores to whatever address the last load or store used.

### 5.2 ROM

Writing **R14** starts a ROM fetch at `ROMBR:R14`; `GETB`/`GETBH`/`GETBL`/`GETBS` collect the byte, whole or merged into half of the destination. `INC R14` re-triggers the fetch, which is what makes a byte-at-a-time ROM walk cheap. The fetch is modelled as instantaneous, so the `SFR` ROM-pending bit never actually sets.

---

## 6. The plot hardware

`COLOR` loads `COLR` from the source register (`GETC` loads it from the ROM buffer instead); `CMODE` loads the plot options `POR`. `PLOT` writes `COLR` at (R1, R2) **and increments R1**, so a run of `PLOT`s fills a row. `RPIX` reads a pixel back.

Colour 0 is transparent unless `POR` bit 0 says otherwise. `POR` also carries dithering (bit 1), high-nibble colour (2), a frozen high nibble (3), and OBJ mode (4).

Because the framebuffer is stored as **SNES planar tiles** — so the S-CPU can DMA it straight to VRAM — writing one pixel means touching one bit in each of 2, 4 or 8 bitplane bytes. An 8-pixel cache absorbs that: pixels accumulate for one tile row and flush as whole bitplanes when the row changes, when `RPIX` needs the memory to be current, or at `STOP`. Partially-filled rows read-modify-write so untouched pixels survive.

### 6.1 Normal layout

Tiles are stored in **vertical strips**, which is why the mode register specifies a *height* (128/160/192 pixels): the height is the column stride.

```
tile = (x >> 3) * columnHeightInTiles + (y >> 3)
addr = (SCBR << 10) + tile * (8 * bpp) + (y & 7) * 2
```

### 6.2 OBJ mode

`SCMR`'s height field is split — **HT1 is bit 2 and HT0 is bit 5**, not adjacent — and the value 3 selects OBJ mode, where the buffer is laid out the way the PPU wants *sprite* tiles instead: 128x128 pages of 16x16 tiles, pages arranged 2x2.

```
page = ((y >> 7) << 1) | ((x >> 7) & 1)
tile = (page << 8) | (((x >> 3) & 15) << 4) | ((y >> 3) & 15)
```

Within a page the order is **column-major**, matching the normal modes; the row-major reading of the same layout produces recognisable but badly streaked output, which is how the two were told apart. Yoshi's Island uses OBJ mode for its intro, in both 4bpp and 2bpp.

---

## 7. Save states

Appended after the four original blobs and after the SA-1's, and only for a SuperFX cartridge — see `EmuSen_Save_States.md` §3. `Cartridge.SuperFx` is `[SkipInState]` for the same reason `Sa1` is.

---

## 8. Debugging facilities

`DebugSettings.SuperFxTraceCountdown` logs one line per GSU instruction — PC, opcode, `SFR`, `CBR` and the interesting registers — and counts itself down. Through the headless harness:

```
dotnet run --project EmuSen.Pharaoh -- <rom> <frames> --flag SuperFxTraceCountdown=200
```

That trace is what found the R15 invariant bug: the loop in §4.1 was visibly re-entering one instruction late.

**The GSU is now visible to the rest of the toolchain, so reach for that first.** `regs` prints a Coprocessor section with `SFR`, `PBR`, `CBR`, `SCBR`, `SCMR`, `ROMBR`, `RAMBR` and the whole `R0`-`R15` file; `GSURAM` is a memory space, so `snapshot`/`diff`/`search`/`watch` reach Game Pak RAM — work RAM and framebuffer both — the same way they reach WRAM; and `cophist [<reg>] [<count>]` replays the last 600 frames of that register file with a "unchanged for N refresh(es)" counter that answers *when did the chip stop* without any instrumentation at all. See `EmuSen_Debugging_Tools_Reference_v5.md` §3.23/§3.23a.

Reads through all of these are side-effect-free by construction, which matters here specifically: `$3031` acknowledges the GSU's interrupt, so a debugger routed through the real register window would clear a flag simply by looking. Prefer these to hand-patching a `Console.WriteLine` into `SuperFx.Execute.cs` — the `LJMP` and `ALT3` rounds in §10 both did that, and both would have been one command against the running game.

---

## 9. Known-wrong and unverified

- **OBJ page arrangement (§6.2)** — the column-major order within a page is confirmed by output; the 2x2 page arrangement and its stride are inferred, not verified.
- **Cycle costs (§2.2)** — approximate.
- ~~**`LJMP` operand direction**~~ — **resolved, and it was wrong.** It had been implemented as "the named register supplies the address, the source register supplies the bank", by analogy with `JMP Rn`. It is the other way round: `Rn` carries the **bank**, `Sreg` the **address**. Corrected, and pinned by `Ljmp_takes_its_address_from_the_source_register`, which fails against the old direction.

  Worth recording *why* this was inverted, because the trap is still there for anything else in this file: the official Nintendo SuperFX documentation has the two operands swapped, and SnesLab's `LJMP` page reproduces the official wording ("the low byte of the source register is loaded into the program bank register") while separately noting that fullsnes considers the official docs mixed up on exactly this point. bsnes settles it — `regs.pbr = regs.r[n] & 0x7f; regs.r[15] = regs.sr();`. **Do not treat the official docs as authoritative for this instruction.**
- **`RAMBR` width** — masked to 5 bits, so a game writing a bank beyond the 128KB chip mirrors rather than faulting. Yoshi's Island does write `RAMBR = 2`.
- **`MERGE` flag thresholds** and `FMULT` rounding are implemented from the shape of the instruction set rather than from confirmed documentation.
- ~~**`ALT3` on the `$Ax`/`$Fx` slots**~~ — **resolved: there is nothing to implement.** `ALT3` is genuinely *undefined* on both slots. bsnes decodes `$A0-$AF` as `IBT`/`LMS`/`SMS` and `$F0-$FF` as `IWT`/`LM`/`SM` for `ALT0`/`ALT1`/`ALT2` and carries **no `ALT3` case at all** for either. No assembler emits it and no correct program contains it, so any behaviour here is arbitrary.

  What matters is only that the operand consumption stays right, and it does: every `$Ax` form consumes exactly one operand byte and every `$Fx` form exactly two, on all four `ALT` states, because `OpIwtLmSm` reads its two bytes before it ever inspects the prefix. Falling through to the immediate form therefore cannot desynchronise the instruction stream — which is the only way an undefined opcode could do real damage. Left as-is deliberately.
- **Bus arbitration (§2.1)** is not enforced.

---

## 10. Status, and where to pick it up

**What is verified.** 54 tests: 38 driving hand-assembled GSU programs through the real chip (arithmetic and its flags, every shift, the prefix and delay-slot semantics, `LOOP`, `LJMP`'s operand direction, RAM round-trips, plotting and transparency) and 16 covering detection, the S-CPU address map and the register window. All pass.

**What Yoshi's Island does.** It boots, the GSU executes genuine game code, and it renders: the intro's bordered frame draws correctly, and the GSU-rendered interior draws recognisable scenery. It then stops drawing a few thousand frames in. The three fixes that got it this far were the R15 invariant (§4.1), OBJ tile order (§6.2), and RAM sizing (§1).

### 10.0 Resolved: the `$6000-$7FFF` window was bank-indexed

**Root cause of the frame-1759 crash below, found and fixed.** `RamOffsetWindow` computed `((bank & 0x3F) << 13) | (offset & 0x1FFF)`, giving each low bank its own 8KB slice of Game Pak RAM. Real hardware mirrors **the first 8KB** into `$6000-$7FFF` of *every* bank `$00-$3F` and `$80-$BF` — bsnes's own manifest says so outright: `map address=00-3f,80-bf:6000-7fff size=0x2000`. The fix is `offset & 0x1FFF`.

The chain from that one expression to a dead console, which is worth reading as an example of how far a mapping error propagates:

1. Yoshi's Island's NMI path reads a jump-table index with `LDY $6F00,X`, landing on `$0F:6F0C`. Correct offset `$0F0C`; we served `$1EF0C`, 120KB further in.
2. That returned `Y = $FD` — **odd**, and this is a table of 16-bit pointers.
3. `LDA $C356,Y` therefore read *across* an entry boundary, at `$0F:C453` instead of `$C452`.
4. The game dispatches with the `PHA`/`RTS` idiom (`$0F:C36E`), so the misread pointer put the PC at `$0F:9969` — **one byte before** the real instruction boundary at `$0F:996A`, where the actual code is four clean `JSR $A9D7` calls (`20 D7 A9` ×4).
5. Misaligned, that byte stream decodes as `TSB` + three junk `LDA #imm16` + **a `PLY` that is not in the real code at all**. Both paths then resynchronise at `$0F:9979` — so the machine kept running, two bytes light on the stack.
6. The routine's closing `RTS` at `$0F:99CB` pulled that corrupted return address and jumped to `$0F:00C4`.
7. From there the S-CPU crawled through zeroed WRAM executing `BRK` (`$00`), each vectoring to a bare `RTI` at `$00:814F` and advancing two bytes, until an `XCE` dropped it into emulation mode with the screen force-blanked.

**The test suite had the bug written into it.** `SuperFxMemoryMapTests` carried `The_scpu_ram_window_advances_one_block_per_bank`, asserting the wrong behaviour in as many words, with a comment stating "not mirrored". It was written from the same assumption as the implementation, so it locked the bug in rather than catching it. Replaced with `The_scpu_ram_window_mirrors_the_first_8kb_into_every_bank`. **A test only pins a fact if the fact was checked against something other than the code it tests.**

**Result:** the game now runs past frame 1759 with output continuing to change through frame 4500 and beyond, `E=0`, `INIDISP=$8F`, layers enabled. §10.1 below is retained as the record of how the failure was measured.

### 10.1 The failure, measured

Earlier revisions of this section assumed the S-CPU "stops producing display output while still running normally", and concluded it must be acting on GSU results that were wrong rather than absent. **That was wrong, and it sent two investigations (§9's `LJMP`, the `ALT3` decodes) after instruction-set details that had nothing to do with it.** The actual sequence, measured with `framesum`, `regs` and `mem` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.21, §3.23):

| Frame | What the machine is doing |
|---|---|
| ≤1758 | Normal. S-CPU alternating between banks `$0F` and `$7E`, `INIDISP = $8F`, GSU `Running = 1`, Game Pak RAM actively changing. |
| **1759** | **S-CPU derails into `$7E:0E6A`** — WRAM — and loops around `$7E:0E6A-0E6D`. |
| 1762-1764 | Wanders: `$00:814F`, then `$04:0749`. |
| **1765** | `XCE` puts it in **emulation mode** (`E=1`), parked at `$00:814F`. `INIDISP = $81` (forced blank), `TM = TS = BGMODE = 0`. |
| ≥1765 | Byte-identical every frame forever. GSU `Running = 0`, `SFR = 0`. |

**`$7E:0E60-0EA7` is all zeroes.** Opcode `$00` is `BRK`, so the S-CPU is executing zeroed WRAM. This is not a new failure mode — **it is the same "crashed the S-CPU into zeroed WRAM" bug this section used to describe as belonging to an earlier stage of development.** It was never fixed. The R15/OBJ/RAM-sizing fixes pushed it later, from boot to frame 1759, which looked like progress from the framebuffer but was not a different bug.

**So the framing to work from is: what transfers control to zeroed WRAM at frame 1759.** The GSU was still working normally right up to that point — `snapshot`/`diff` over `GSURAM` shows 1083 bytes changing across the crash window, and `Running = 1` at frame 1750 — so the chip does not stop and then starve the S-CPU. The S-CPU dies first, and the GSU stops afterwards simply because nothing starts it again. Any theory that begins "the GSU computed the wrong value" has to explain a *control-flow* derailment, not a bad pixel.

**`LJMP` was the leading suspect, and it has been ruled out.** It really was implemented backwards (§9), and that is fixed — but it is *not* what breaks this game. **Yoshi's Island never executes `LJMP` at all**: instrumenting `OpLjmp` with a print and running `SMW2.smc` headless for 5000 frames — well past the point the display goes blank — produced zero hits, while `SuperFxTraceCountdown` over the same boot confirms the GSU is busily executing real game code the whole time (`08:BD16` onward, plotting loops around `08:BD24`). The fix is a genuine latent-correctness win for some other game; it changes nothing here.

That result is worth keeping in mind for the rest of this list: **check that the game actually reaches a feature before spending time on its correctness.** A one-line print plus a long headless run costs a couple of minutes and can retire a suspect outright.

**The `ALT3` decodes were the next suspect, and they are ruled out twice over.** First, they are unreachable: probes in both `OpImmediateByteOrShort` and `OpIwtLmSm` over the same 5000-frame run recorded **zero** `ALT3` executions, against a control probe on the same `$Fx` slot that fires constantly (`08:BD1A`, `08:BD1D`, `08:BD22`, `0A:8011`, …), so the instrumentation was demonstrably live. Second, per §9 there is no defined `ALT3` behaviour on these slots to get wrong. Retired, no code change.

**Two suspects retired by the same cheap method, which says something about the method.** Working down a ranked list of *plausible-looking* inaccuracies has now cost two rounds and produced one latent fix unrelated to this game. The list was built by reading the implementation for things that looked shaky, not by following evidence from the failure. Prefer evidence next: find the frame where output stops, and work backwards from what the machine is actually doing at that moment.

**Where to look next.** Work backwards from frame 1759 and find what puts `$7E:0E6A` into the program counter. `bp add` plus `--cpulog` over frames 1755-1760 should show the transfer directly — a `JMP`/`JSR` through a pointer, or an `RTS`/`RTI` to a corrupted return address. The distinction matters: a bad indirect jump means whatever computes the pointer is wrong, while a bad return means the stack was corrupted earlier and the derailment is a symptom with its own separate cause.

Given the GSU is demonstrably alive right up to the crash, the likeliest shapes are:

1. A jump table or handler pointer the game builds from data the GSU produced (or that the GSU's DMA/RAM mapping should have produced), landing on zero.
2. Stack corruption — an unbalanced interrupt, or a `Sa1`/GSU IRQ delivered when it should not be. Note `VenusCore` polls `gsu.ScpuIrqPending` per instruction and calls `Cpu.Irq()`; that path is gated by the I flag and by `CFGR` bit 7, but it has never been examined against a real game's expectations. `SFR` reads `$0028` at frame 1750, so the IRQ flag is *not* set there — this is a lead to check, not a diagnosis.
3. A mapping hole: the S-CPU reading zeroes from an address the SuperFX mapper should be serving, then using them as a pointer.

The read-the-code suspects that remain (`CACHE`-side invalidation, `SCBR` granularity, `MERGE`/`FMULT` rounding) are all *pixel* correctness issues. Per §10.1 they cannot by themselves explain a control-flow derailment, so they are no longer the front of the queue.

`SuperFxTraceCountdown` (§8) plus a diff against a known-good trace is the practical route; without a reference to diff against, the instruction-level tests in `EmuSen.WiseMan/Coprocessors/SuperFxInstructionTests.cs` are the place to encode each new fact as it is established.
