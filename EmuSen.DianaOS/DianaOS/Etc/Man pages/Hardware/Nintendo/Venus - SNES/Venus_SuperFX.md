# Venus (SNES) — SuperFX (GSU)

Covers everything under `Cores/Nintendo/Venus - SNES/Coprocessors/SuperFx/`, plus `SuperFxMapper`. This is the second coprocessor this core implements, after the SA-1 (`Venus_SA1.md`).

> **Status: working, with one open issue that is not the chip's.** The chip is fully built — memory map, the whole instruction set, the plot hardware — and since §10.4 (2026-08-02) Yoshi's Island's intro renders correctly: the frame, the sky, and every picture the GSU draws inside the frame. The one remaining defect is the story-text strip, and §10.5 shows the GSU decodes that text correctly into Game Pak RAM, so the fault is downstream on the S-CPU/PPU side. Everything in §1-§8 is implemented; treat §9's warnings as live.

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
| `$00-$3F/$80-$BF:6000-7FFF` | Game Pak RAM, the **first 8KB mirrored** into every bank |
| `$00-$3F/$80-$BF:8000-FFFF` | ROM, LoROM-style |
| `$40-$5F`, `$C0-$DF` | ROM, linear 64KB banks |
| `$60-$7D`, `$E0-$FF` | Game Pak RAM, flat |

Bit 15 of the address is **ignored** in the LoROM view, so `$00:0000` and `$00:8000` are the same ROM byte — that is what lets the GSU point `ROMBR:R14` anywhere in a bank.

The `$6000-$7FFF` window **mirrors the first 8KB of Game Pak RAM into every bank** `$00-$3F`/`$80-$BF`. This paragraph previously claimed the opposite — that the window is packed, one block per bank — which is the bug §10.0 fixed; it is corrected here so the two sections stop contradicting each other.

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

Multi-byte instructions in a delay slot fetch their operands **from the retargeted R15**, so the branch target's first byte is eaten as the operand and execution resumes one byte in. Mesen agrees (its trailing `R[15]++` is skipped only when `_r15Changed`, and `ReadOperand` advances from the already-retargeted R15).

This file used to add "which is why real GSU code never puts one there". **That is wrong — Yoshi's Island puts one there on purpose.** `0A:81B4` is `BNE $81C5` with `IBT R10,#$08` in the delay slot; the branch target `$81C5` *is* the `$08` operand byte of the `IBT R10,#$08` sitting at `$81C4`, so both paths load 8 and both resume at `$81C6`. The overlap is deliberate code compression, and it only works if the operand comes from the target. It is a good check on the pipeline model: get this wrong and `R10` loads `$2B` (the `WITH R11` opcode) as its bit count.

### 4.2 Prefixes

`TO Rn` ($10-$1F) sets the destination. `FROM Rn` ($B0-$BF) sets the source. `WITH Rn` ($20-$2F) sets both *and* raises the `B` flag — and while `B` is set, `TO`/`FROM` stop being prefixes and become the real instructions `MOVE`/`MOVES`. `ALT1`/`ALT2`/`ALT3` ($3D/$3E/$3F) set the flags that pick which of a slot's four meanings runs.

All of it is cleared after the next non-prefix instruction, which is why `FROM R1 : ADD R2 : ADD R2` adds R1+R2 into R0 and then R0+R2 into R0.

**The eleven branch opcodes (`$05-$0F`) are the exception: a branch does not clear the prefix, so it passes it to the delay-slot instruction.** Every other real instruction clears it; `JMP` and `LOOP` clear it too, even though they also jump. Mesen encodes this by calling `ResetFlags()` from every instruction *except* `Branch`, and Yoshi's Island depends on it — `0A:8146` reads `FROM R6 : TO R5 : ALT2 : BRA $8106` with `AND #15` in the delay slot, which is `R5 = R6 & $0F`, the run colour for the next RLE span. Clear the prefix there and that byte decodes instead as `AND R15` into `R0`, which both loses the colour and corrupts the bit-stream buffer. That was the whole of §10.4. Pinned by `A_branch_carries_its_prefix_into_the_delay_slot` and `An_untaken_branch_also_carries_its_prefix`, both of which fail against the clearing form.

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

**The coordinates are the low bytes of R1 and R2, not the whole registers.** R1 is a full 16-bit register and `PLOT`'s increment carries into its high byte, so a long run walks R1 past 255 as a matter of course; the plot hardware only ever sees `R1 & 0xFF` and `R2 & 0xFF`. Passing the full register instead sends every pixel after the wrap into a tile hundreds of tiles away — see §10.2, where this was measured at `PLOTMAXX = 0xFFFF` in Yoshi's Island. Pinned by `Plot_uses_only_the_low_byte_of_its_coordinate_registers`, which fails against the unmasked form.

Colour 0 is transparent unless `POR` bit 0 says otherwise. `POR` also carries dithering (bit 1), high-nibble colour (2), a frozen high nibble (3), and OBJ mode (4). The transparency test reads `COLR` **before** dithering is applied, and compares against the depth's own mask (`0x03` at 2bpp, `0x0F` at 4bpp, the whole byte at 8bpp), with `POR` bit 3 narrowing it to the low nibble. OBJ mode does **not** exempt a pixel from the test — this file previously said it did, which silently disabled transparency for every plot Yoshi's Island makes, since its intro sets `POR` bit 4 throughout.

Because the framebuffer is stored as **SNES planar tiles** — so the S-CPU can DMA it straight to VRAM — writing one pixel means touching one bit in each of 2, 4 or 8 bitplane bytes. An 8-pixel cache absorbs that: pixels accumulate for one tile row and flush as whole bitplanes when the row changes, when `RPIX` needs the memory to be current, or at `STOP`. Partially-filled rows read-modify-write so untouched pixels survive.

### 6.1 Normal layout

Tiles are stored in **vertical strips**, which is why the mode register specifies a *height* (128/160/192 pixels): the height is the column stride.

```
tile = (x >> 3) * columnHeightInTiles + (y >> 3)
addr = (SCBR << 10) + tile * (8 * bpp) + (y & 7) * 2
```

### 6.2 OBJ mode

`SCMR`'s height field is split — the two halves are not adjacent — and the value 3 selects OBJ mode, where the buffer is laid out the way the PPU wants *sprite* tiles instead: 128x128 pages of 16x16 tiles, pages arranged 2x2.

**`CMODE` bit 4 selects OBJ mode too**, independently of the height field, so the selector is `height == 3 || (POR & 0x10)`. This file used to key the layout off the height field alone. That happens to be a no-op for Yoshi's Island — measured, its intro sets *both*, `SCMR = $3C`/`$3D` with `POR = $11` — but it is not a no-op in general, and Mesen keys off `CMODE` bit 4 alone.

```
page = ((y >> 7) << 1) | ((x >> 7) & 1)
tile = (page << 8) | (((y >> 3) & 15) << 4) | ((x >> 3) & 15)
```

Within a page the order is **row-major** — `y` supplies the high nibble of the tile index. This file previously said column-major, and claimed the two had been told apart by which one produced streak-free output. **That claim does not hold up**: with the coordinate-truncation bug above still present, both orders produce equally unreadable output, so whatever was compared could not have distinguished them. Row-major is what bsnes and Mesen both compute (`((y & 0x78) << 1) + ((x & 0x78) >> 3)`), and that is the only reason this file now states it. Yoshi's Island uses OBJ mode for its intro, in both 4bpp and 2bpp.

---

## 7. Save states

Appended after the four original blobs and after the SA-1's, and only for a SuperFX cartridge — see `EmuSen_Save_States.md` §3. `Cartridge.SuperFx` is `[SkipInState]` for the same reason `Sa1` is.

---

## 8. Debugging facilities

### 8.1 Reading the chip's code: `GSUBUS` and the disassembler

**The GSU's own code is readable from the shell.** `GSUBUS` is a memory space over the chip's 24-bit program map — ROM through `RomOffset` for banks `$00-$5F`, Game Pak RAM for `$60+`, which is where the S-CPU stages code for it (§5.1) — and `disasm GSUBUS <pbr><r15>` decodes it with a real GSU disassembler rather than the 65816 one. Reads deliberately bypass the instruction cache, so looking at an address never fills a cache line (§4.3).

This exists because §10.4 was found with a throwaway 120-line Python disassembler, and that section's own conclusion was that building one properly is cheap and should be the first tool reached for here. It is now built:

```
disasm GSUBUS 0A8146 6      # FROM R6 / TO R5 / ALT2 / BRA $8106 / AND #15
```

**A static per-opcode table cannot do this job, which is the whole design point.** The same byte is a different instruction under a different prefix — `$41` is `LDW (R1)` normally and `LDB (R1)` after `ALT1` — so the disassembler walks forward carrying `ALT1`/`ALT2`/`WITH`/`TO`/`FROM` exactly as `StepInstruction` does, including §4.2's rule that a branch is the one non-prefix instruction that does *not* clear the prefix. Verified against the two routines this file already hand-disassembled: it reproduces §10.4's `0A:8146` and §10.3's RLE plot loop at `0A:80E9` line for line.

Undefined `ALT3` decodes on the `$Ax`/`$Fx` slots are shown as their `IBT`/`IWT` fallthrough, matching what the dispatcher actually does — see §9.

### 8.2 Knowing whether the chip ran it: `cov gsu`

`cov` records every address executed over a run and then answers "did control flow ever reach here" (`EmuSen_Debugging_Tools_Reference_v5.md` §3.24). `cov gsu` scopes it to the GSU's own instruction stream, which is a separate address space from the S-CPU's.

**This replaces a method §10.1 had to hand-roll twice.** Both the `LJMP` and `ALT3` rounds were retired by patching a `Console.WriteLine` into an opcode handler and running headless to see whether it ever fired — the right instinct, and §10.1's own lesson is *"check that the game actually reaches a feature before spending time on its correctness"*. That is now one command against the running game, with no rebuild.

Recording is off by default and costs one bool test in the dispatcher while disarmed.

### 8.3 Tracing flags

`DebugSettings.SuperFxTraceCountdown` logs one line per GSU instruction — PC, opcode, `SFR`, `CBR` and the interesting registers — and counts itself down. Through the headless harness:

```
dotnet run --project EmuSen.Pharaoh -- <rom> <frames> --flag SuperFxTraceCountdown=200
```

That trace is what found the R15 invariant bug: the loop in §4.1 was visibly re-entering one instruction late.

**The GSU is now visible to the rest of the toolchain, so reach for that first.** `regs` prints a Coprocessor section with `SFR`, `PBR`, `CBR`, `SCBR`, `SCMR`, `ROMBR`, `RAMBR` and the whole `R0`-`R15` file; `GSURAM` is a memory space, so `snapshot`/`diff`/`search`/`watch` reach Game Pak RAM — work RAM and framebuffer both — the same way they reach WRAM; and `cophist [<reg>] [<count>]` replays the last 600 frames of that register file with a "unchanged for N refresh(es)" counter that answers *when did the chip stop* without any instrumentation at all. See `EmuSen_Debugging_Tools_Reference_v5.md` §3.23/§3.23a.

**Sampling a register once per frame is not enough, and mis-sampling sent §10.2 down two dead ends.** `SCMR`, `SCBR` and `POR` are all set and cleared *inside* one frame — `cophist SCMR` reads `00` all through Yoshi's Island's intro while every plot in that same frame runs with `SCMR = $3C`. So the plot-time state is published separately, latched by `Plot` itself rather than by the per-frame refresh: `PLOTS` and `PLOTSOBJ` (how many pixels, and how many took the OBJ layout — equal means the layout question is settled), `PLOTSCMR`/`PLOTSCBR`/`PLOTPOR` (the register values actually in force at the last plot), and `PLOTMAXX`/`PLOTMAXY`/`PLOTADRLO`/`PLOTADRHI` (the coordinate and Game Pak RAM ranges the chip was asked to cover). `PLOTMAXX` is what exposed the coordinate-truncation bug in §6, in one command.

`SCPURAMHOT` counts S-CPU accesses to Game Pak RAM while `GO = 1` — the bus-arbitration violations §2.1 does not enforce. It reads **0** for Yoshi's Island, which is how "the S-CPU is DMAing the framebuffer out from under the GSU" was retired.

`DebugSettings.SuperFxPlotTraceSkip` / `SuperFxPlotTraceCountdown` log the plot stream itself — `x`, `y`, the raw `R1`/`R2`, `COLR`, `POR`, `SCMR`, `SCBR`, the target address and the PC — after skipping the first N plots, so a later drawing pass can be reached without drowning in the first. `SuperFxPlotTraceInstr` logs one line per GSU instruction over the same window. Rendering an image straight from the plot stream is what separated "the wrong colours are being decoded" from "the framebuffer encoding is wrong" in §10.3.

`DebugSettings.SuperFxRamWriteTraceAddr` (a flat Game Pak RAM offset, `-1` off) plus `SuperFxRamWriteTraceCountdown` log GSU-side writes to one address with the GSU PC that made them. Reach for it when the chip builds output with **stores rather than `PLOT`** — the plot-trace flags above see nothing then. That is what proved the BG3 tilemap is GSU output written by a literal-run copy loop at `08:A9FA` (§10.5); a per-frame `GSURAM` dump could not, because the buffer is overwritten within the same frame it is DMAed. `--flag` takes decimal, so pass `23880`, not `0x5D48`.

`DebugSettings.SuperFxSpeedDivisor` scales the chip's cycle cost. If a failure is a GSU/S-CPU synchronisation problem, the output changes character with the divisor; if the handshake is sound, only the frame the work lands on moves. Cheaper than reasoning about the handshake from the code.

Reads through all of these are side-effect-free by construction, which matters here specifically: `$3031` acknowledges the GSU's interrupt, so a debugger routed through the real register window would clear a flag simply by looking. Prefer these to hand-patching a `Console.WriteLine` into `SuperFx.Execute.cs` — the `LJMP` and `ALT3` rounds in §10 both did that, and both would have been one command against the running game.

---

## 9. Known-wrong and unverified

- **OBJ page arrangement (§6.2)** — the 2x2 page arrangement and its stride are inferred, not verified. The within-page order is now row-major on Mesen's and bsnes's authority, not on output evidence; the previous "confirmed by output" claim was withdrawn (§6.2).
- **`SCMR` height-field bit order** — this file and the implementation read **HT1 from bit 2 and HT0 from bit 5**; **Mesen reads them the other way round** (`ScreenHeight = ((v & 0x04) >> 2) | ((v & 0x20) >> 4)`). The two disagree only for height values 1 and 2, i.e. 160 vs 192 pixels, and they agree that 3 means OBJ mode. Yoshi's Island plots with both bits set, so this cannot be settled from that game and is currently untested either way. Do not "fix" it to match Mesen without a game that distinguishes them.
- **`COLOR`/`GETC` nibble handling** — bsnes and Mesen genuinely fork here, and `ColorValue` follows **bsnes**: `POR` bit 2 rewrites the source as `(source & $F0) | (source >> 4)` and then bit 3 may also apply, whereas Mesen early-returns `(COLR & $F0) | (value >> 4)` on bit 2 so the two bits are mutually exclusive. The two agree on the low nibble, so they are indistinguishable at 2bpp and 4bpp; they differ only in the high nibble, i.e. at 8bpp or when a later `COLR`-freeze reads it back. Untested either way — Yoshi's Island runs with both bits clear.
- **`ALT1`/`ALT2`/`ALT3` and the `B` flag** — Mesen's `ALT1()`/`ALT2()`/`ALT3()` each set `Prefix = false`, i.e. an `ALT` prefix *clears* the `WITH` flag while leaving `SrcReg`/`DestReg` alone. This implementation leaves `B` set. The two differ only for `WITH Rn : ALT? : TO Rm` / `FROM Rm`, where Mesen decodes the third instruction as a prefix and we decode it as `MOVE`/`MOVES`. **Untested either way — Yoshi's Island never writes that sequence**, so it cannot be settled from the one SuperFX game here. Found while fixing §10.4; left alone deliberately, on the same reasoning as the `SCMR` height field above.
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

**What is verified.** 57 tests: 41 driving hand-assembled GSU programs through the real chip (arithmetic and its flags, every shift, the prefix and delay-slot semantics, `LOOP`, `LJMP`'s operand direction, RAM round-trips, plotting, transparency, and the plot coordinate mask) and 16 covering detection, the S-CPU address map and the register window. All pass.

**What Yoshi's Island does.** It boots and plays its whole intro: the bordered frame, the night sky, and — since §10.4 — **the pictures inside the frame, which now render correctly** through the Nintendo-logo quilt, the sunrise and the cloud page. §10.1/§10.2/§10.3 are retained as the record of how that was measured, but the failure they describe is fixed; do not work from them as if it were open. **What remains is the story text strip below the frame**, and §10.5 shows that one is *not* a GSU bug — the chip decodes the text correctly into Game Pak RAM.

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

### 10.2 The intro picture, measured

The frame-1759 crash is gone, so the game now reaches its intro and stays there. The failure that remains is a rendering one, and this is the shape of it, measured end to end rather than guessed.

**Which layer is wrong.** `--flag LayerEnableMask=N` isolates one layer at a time. **BG2** (the night sky) and **BG3** (the ornate frame) both draw correctly. **BG1 is the broken layer** — it carries the picture inside the frame. **OBJ draws nothing at all**, which is worth knowing before spending any more time on OBJ-mode *display* correctness: the OBJ-shaped framebuffer §6.2 describes never reaches a sprite. `BGMODE = $09`, so this is Mode 1, not a hi-res mode.

**How the picture gets to BG1.** The GSU plots into Game Pak RAM and the S-CPU DMAs the result to VRAM. `DmaVerboseLogging` shows the whole path: six transfers, all on channel 0, all `src = $70:5800`, all 8192 bytes, landing at VRAM `$0000/$2000/$4000/$6000/$8000` (BG1's character data) and `$A000`. The copy is verbatim — matching 1KB blocks line up exactly, and the freshest chunk is 75% identical to live Game Pak RAM — so **nothing is corrupted in transit**. Whatever is wrong is wrong in Game Pak RAM before the DMA reads it.

**What the chip is asked to draw.** Two plot phases, and only two:

| Phase | Frames | Plots | `SCBR` | `SCMR` | `POR` |
|---|---|---|---|---|---|
| 1 | before 250 | 24,064 | `$13` (`$4C00`) | `$3C` (2bpp) | `$00` |
| 2 | 274-283 | 81,920 | `$16` (`$5800`) | `$3D` (4bpp) | `$11` (OBJ) |

Phase 2 is exactly 5 x 16,384 pixels, matching the five 8KB DMAs. Coordinates cover x 0-255, y 0-127, and the addresses touched run `$5800`-`$97EE` — two OBJ pages, 16KB.

**Retired, with the measurement that retired each.** Do not re-run these.

- *"The S-CPU DMAs the framebuffer while the GSU is still drawing."* `SCPURAMHOT` (§8) counts S-CPU Game Pak RAM accesses while `GO = 1`. It is **0**. The handshake is sound. Scaling the chip's speed with `SuperFxSpeedDivisor` moves which frame the work lands on but does not change the character of the corruption, which is the same conclusion from the other direction.
- *"The OBJ within-page tile order is the wrong way round."* Tried both orders with the coordinate bug fixed. **Both are unreadable**, so this is not what is left. Row-major is retained because bsnes and Mesen agree on it (§6.2), not because it fixed anything.
- *"VMAIN address translation is unimplemented."* It is implemented, and all three rotate modes match the standard formulas.

**Fixed along the way**, each a real divergence from bsnes/Mesen, none of them sufficient: the plot coordinate mask (§6, the significant one — x was reaching `$FFFF`), OBJ-mode selection ignoring `CMODE` bit 4 (§6.2), and the transparency test being skipped outright in OBJ mode and evaluated after dithering (§6).

### 10.3 The blitter, traced

The intro picture is drawn by an RLE row decoder at `0A:80E9-0A:8113`, reached through a bit-stream reader at `0A:809C-0A:80B5` that refills from ROM via a subroutine at `0A:81B3`. Disassembled and confirmed against a live trace:

```
8108: B5        FROM R5           ; run colour
8109: 3D 31     ALT1: STB (R1)    ; scratch[R1] = colour
810B: 3C        LOOP              ; R12 = run length
810C: E1        DEC R1            ; delay slot - the row fills DOWNWARD
810D: 0A 87     BPL 8096          ; another run while R1 >= 0
810F: AC 00     IBT R12,#$00
8111: 05 D7     BRA 80EA          ; row done
80EB: D1        INC R1            ; -1 -> 0
80EC: FC 80 00  IWT R12,#$0080    ; 128 pixels
80F1: 3D 41     ALT1: LDB (R1)    ; colour = scratch[R1]
80F3: 4E        COLOR
80F4: 3C        LOOP
80F5: 4C        PLOT              ; delay slot; PLOT increments R1
```

**This gives a free oracle: every row's fill must end with `R1 = -1`,** because `INC R1` then has to leave `R1 = 0` for the 128-pixel plot loop. **77 of 128 rows in a pass end somewhere else** — mostly `-2` or `-3`, occasionally `-96`. Those rows are drawn shifted, and their spill lands at `x >= 128` in OBJ page 1, which is never DMAed.

`R1` doubles as the scratch pointer and the plot X, so `x >= 128` plots are the *symptom*, not the fault: `scratch[0]` still reaches `x = 0`, and the spill is invisible. **Do not chase `PLOTXHIGH` again** — it undercounts badly (a row off by two contributes only two hits), which is why 60% broken rows first read as 9%.

Rendering straight from the plot stream — bypassing the framebuffer encoding entirely — reproduces the same streaked garbage as `GSURAM` does, so the fault is upstream of `PLOT`: **the wrong colours are being decoded, not mis-stored.**

**Retired here, with the measurement.** Do not redo these.

- *"The pixel cache or the bitplane encoding is wrong."* Rendering from the plot stream is identically broken, so the encoding is not involved. Our `FlushPixelCache` also matches Mesen's `WritePixelCache` byte for byte, including the `ValidBits` read-merge and the `(x & 7) ^ 7` bit order.
- *"The OBJ tile index is wrong."* Mesen's `GetTileIndex` case 3 is `((y & 0x80) << 2) + ((x & 0x80) << 1) + ((y & 0x78) << 1) + ((x & 0x78) >> 3)`, which is algebraically our page/row/column expression. All four `ScreenHeight` cases agree too.
- *"The colour transform is wrong."* `POR = $11` for the whole pass: neither `ColorHighNibble` (bit 2) nor `ColorFreezeHigh` (bit 3) is set, so `ColorValue` is a pass-through and cannot be the fault **for this game**. bsnes and Mesen disagree about it in general — see §9.
- *"The GSU ROM map is wrong."* Mesen registers `00-3f:8000-ffff` and `40-5f:0000-ffff`, matching `RomOffset`. `ROMBR = $5E`, `R14 = $8305` resolves to `$1E8305` in both.
- *"The ALU or shift flags are wrong."* `LSR`, `ROL`, `ROR`, `ASR`/`DIV2`, `ADD`/`ADC`, `SUB`/`SBC`/`CMP`, `AND`/`BIC`, `OR`/`XOR`, `NOT`, `SWAP`, `LOB`, `HIB`, `MERGE`, `MULT`/`UMULT` and `IBT`'s sign extension were all diffed against `Gsu.Instructions.cpp`. The overflow expressions differ in form but are algebraically identical.

**Two facts that constrain whatever is left.** First, the failures are wrong *values*, not lost sync: one run decoded as 46 where 44 was needed differs in a single bit, and the bit **count** stays right — `R10` and `R14` carry across rows without re-anchoring, so a mis-consumed bit would desync the rest of the picture permanently, and it does not. Second, do **not** read "40% of rows land exactly on `-1`" as evidence the decoder works. At these run lengths random data lands there about a third of the time, which is the mistake that made the decode look sound for most of this round.

**Where to pick it up.** The bit reader is `R0` = bit buffer, `R10` = bits left, `R4` = bit weight, `R12` = accumulator; a value bit is the carry out of `LSR` tested by `BCC` at `80A5`, and a continuation bit is the carry tested by `BCS` at `80B4`. `R0` is parked in `R4` across the plot loop (`WITH R0 : TO R4` at `80EA`) and restored on the way back in. Take one row that ends wrong, dump `80A5`/`80B4` with `R0`/`R10`/`R4`/`R12` for its final run, and hand-decode the same bits out of the ROM bytes `R14` walked. That says in one pass whether the emulator built the wrong number from the right bits, or the game is being pointed at the wrong bits to begin with — the two remaining possibilities. The refill path is worth reading closely first: `80A0 LINK #4` / `80A1 IWT R15,#$81B3` calls out with `GETB` in the delay slot, the return at `81C7` jumps back through `R11` with an `LSR` in *its* delay slot, and both `IBT` operands in that path are fetched from the branch target rather than from after the opcode.

Trace with `--flag SuperFxPlotTraceSkip=N --flag SuperFxPlotTraceInstr=M` (§8).

### 10.4 Resolved: a branch was clearing the prefix before its delay slot

**Root cause of the unreadable picture, found and fixed (2026-08-02).** `Branch` fell through to the dispatcher's `if (!_prefixInstruction) ClearPrefix()`, so `TO`/`FROM`/`WITH`/`ALT` set ahead of a branch were gone by the time the delay-slot instruction ran. Branches are the one non-prefix instruction that must *not* clear it — see §4.2 for the rule and the Mesen cross-check.

**The game's own code proves it, without needing a reference emulator.** `0A:8146` is `FROM R6 : TO R5 : ALT2 : BRA $8106`, with the byte at `814B` as the delay slot. With the prefix intact that byte is `AND #15`, giving `R5 = R6 & $0F` — the run colour the RLE span loop at `8108` immediately stores. With the prefix cleared it is `AND R15`, which ANDs `R0` with the program counter. So the bug both dropped the colour *and* corrupted `R0`, which is the bit-stream buffer §10.3 traced. That is why the failure looked like "wrong values, right bit count": `R0` was being clobbered between reads, not desynchronised.

**Measured before and after**, on the §10.3 oracle rather than by eye:

| | before | after |
|---|---|---|
| `PLOTXHIGH` (plots at x ≥ 128) | 10,347 | **0** |
| `PLOTMAXX` | `$FF` | **`$7F`** |
| `PLOTADRHI` | `$97EE` (spilling into OBJ page 1) | **`$77EE`** (one 8KB page) |

`PLOTMAXX = $7F` with `PLOTXHIGH = 0` *is* §10.3's invariant restated: every row now fills exactly `R1 = 127 … -1`, so `INC R1` leaves 0 and the 128-pixel plot loop starts at x = 0. Phase 2 lands as exactly five 16,384-pixel passes inside `$5800-$77FF`, matching the five 8KB DMAs §10.2 recorded. 1074 tests pass.

**Two lessons worth more than the fix.** First, §10.3's ranked suspect list did not contain this, because every entry on it was an *instruction semantic* and this is a *dispatcher* semantic — the prefix rule lives in `StepInstruction`, not in any opcode. When a list of plausible causes has been worked through twice without result, suspect the layer the list is not written at. Second, the answer was legible in the game's own instruction stream: an `ALT2` immediately before a branch is meaningless unless the prefix survives, so **disassembling the failing routine and asking "what would make this code sensible" beat auditing the emulator against a reference.** The disassembler used is 120 lines of Python over the GSU opcode table; building one is cheap and it is now the first tool to reach for here.

### 10.5 Open: the strip below the frame is 18 tiles that are never uploaded

**This section replaces an earlier version whose two leads were both wrong.** It said the BG3 tilemap was suspect because its entries "repeat on a 9-tile cycle", and it called the strip "the story text". Both are disproved below. Read this version.

**The GSU is not at fault, and both halves of that are now measured.**

- *The pixels it draws are right.* Dumping `GSURAM $4C00` (the 2bpp pass, `SCBR = $13`) and rendering it as an OBJ-layout page shows the story text fully legible — "A stork hurries … in his bill … across the … sky".
- *The tilemap it decompresses is right.* The BG3 tilemap is **GSU output**, decompressed into Game Pak RAM `$5800` and DMAed to VRAM `$E800` (2048 bytes, from `$00:B1A5`); `SuperFxRamWriteTraceAddr` (§8) caught the chip writing it from `08:A9FA`, a literal-run copy loop. The 9-tile cycle is **genuine ROM data**: the literal run `A1 3D A2 3D … A9 3D` sits at ROM `0x1B19DF`, and rows 17-22 decompress from strictly increasing offsets (1775925, 1775980, 1776035, 1776095, 1776120) in one stream. The repeat is an **overlapping LZ back-reference** — a standard idiom for a repeating pattern — so the strip is a decorative 9-tile motif tiled three times, by design. Not text, and not a decode error.

**What is actually wrong: 18 referenced tiles have no graphics.** BG3's tilemap uses 92 distinct tiles. 74 of them fall in VRAM `$F000-$F7FF` (tiles `$100-$17F`), which is cleared and then filled with the frame graphics from `$70:5800`. The other **18 — `$1A1-$1A9` and `$1B1-$1B9`, at VRAM `$FA10-$FBA0` — are never written by anything except a constant fill** (`$0F:C09E`, value `$0130`, `step=0`, covering `$F800-$FBFF`). That constant, read as 2bpp character data, is exactly the vertical-stripe pattern on screen.

Confirmed by injection: `dump GSURAM 4C00 800` then `load VRAM F800` puts plausible data under those tile indices, and the strip immediately renders as three identical repeats of one motif — the shape the tilemap describes. So the tilemap semantics are right and only the graphics are missing.

**Ruled out, with the measurement.**

- *"The VRAM destination is computed wrong."* It is hard-coded game data: `7E:C8AC LDA #$7000 / STA $2116` with source `$4C00`, gated on `$0D15`. Nothing computes it.
- ~~*"Some code uploads there and we lose it."*~~ — **the measurement that retired this was wrong, and the claim is withdrawn.** It said a regex sweep of WRAM bank `$7E` and the whole ROM for `LDA #imm16 : STA $2116` finds no site at all targeting `$7C00` (VRAM `$F800`). A *runtime* sweep — `watch add IO 2116 2 write` then `watch summary`, which is addressing-mode-agnostic where a regex is not — finds **18 distinct sites writing VMADD, and two of them write `$7C00`**: `$00:E477` and `$00:E4DA`. So code targeting that window does exist and does run. Both appear to belong to the clear rather than to an upload (the only DMA reaching `$F800-$FBFF` is still the constant fill), but "nothing addresses `$F800`" is no longer a fact this section may lean on.

  **Two traps worth more than the correction.** First, a regex for `LDA #imm16 : STA $2116` can only ever find destinations written as immediates, and in this game the destination normally arrives through a queue node — the sweep was structurally incapable of answering the question it was asked. Second, `watch summary` used to group only the **last 500** events, silently reporting a truncated site list as the complete one; the first run of this sweep returned exactly one site because of it. It now aggregates per-site totals outside the ring and says so when the ring is short (`EmuSen_Debugging_Tools_Reference_v5.md` §3.9). Note also that the headless harness pre-registers two WRAM watches, so a `watch add` of your own gets id **#3**, not #1.
- *"It is drawn with sprites."* `sprites` reports **0 active sprites**, and `OBSEL = $02`.
- *"A base register moves under us."* `BG3SC`, `BG34NBA` and `BGMODE` are each written exactly twice for the whole intro, both times from `$00:BA1A`, settling at `$74` / `$77` / `$09`. No HDMA touches them.

**Measured since, with `cov` and the runtime VMADD sweep (§8.2).** All of this is new evidence, not re-derived from the above:

- **The four bank-`$10` `$2116` sites really never execute** — `cov 10F452 10` and its three siblings all report *Never reached* over 300 frames. But the inference drawn from that is weaker than it looked: **a different bank-`$10` routine does run**, at `$10:87CE-$10:88D3`, and performs **eleven** VRAM uploads. Their destinations are word `$6000`, `$7800` ×2, `$78A1`, `$78C1`, `$7903`, `$7923`, `$7960`, `$7980`, `$79C0`, `$79E0` — i.e. VRAM `$C000` and `$F000-$F3C0`. None reaches `$F800`. So bank `$10` is not simply skipped; part of it runs and covers only the low half of the tile range.
- **The `$7000` destination at `7E:C8AC` is genuine, not a corrupted WRAM copy.** That block is copied from ROM `0x48AC`, and the ROM bytes are `A9 00 70 8D 16 21 A9 00 4C 85 F7` — byte-identical, and the only variant of that pattern in the whole 2MB image. The game really does hard-code `$7000` for the `$4C00` job, so "the copy got damaged on the way into WRAM" is retired.
- **`7E:C8AF` executes exactly once** in 300 frames, and the `$0CF9`-gated sibling at `7E:C8CA` (destination word `$5000`, i.e. VRAM `$A000`) **never executes at all** — `cov 7EC890 A0` shows control jumping straight from the `BEQ` at `$C8C8` to `$C8E0`.

**Where to pick it up.** The 2048-vs-4096 hypothesis is still open and still the cheapest test: `watch add IO 4305 2 write` now reports four sites (`7E:D4BE` and `7E:D526` dominating at 414 each, plus `$00:82AB` and `$00:B1AD`), so the size actually programmed for the `$70:5800 -> $F000` transfer can be read off directly rather than inferred.

The more interesting thread is the one the corrected sweep opened: **2048 bytes at `$F800` would cover tiles `$180-$1FF`, which contains all 18 missing tiles exactly** — and §10.5's own injection test showed that putting `GSURAM $4C00`'s 2KB there renders the motif correctly. That is the same 2KB the game uploads to `$E000`. So the shape to test next is whether the `$4C00` page is meant to land in *both* windows (one upload we are missing, gated by a flag that never gets set), rather than whether one upload is the wrong size. `cov` on the `$0D15`/`$0CF9` writers will say which flag never gets set and who was supposed to set it.

**A caveat on reading `disasm` output in this region.** `7E:C8AC` is `LDA #$7000` (3 bytes), but a `disasm` started at the wrong offset renders it as two 2-byte instructions, because the static disassembler has to guess M/X (see `SnesDebugTarget.Disassemble`'s own comment). `cov` recovers the true boundaries — the recorded addresses *are* the opcode boundaries — so disassemble, then check the boundaries against `cov` before trusting a decode in WRAM-resident code.

#### 10.5a Measured 2026-08-02: the upload map, and a correction that invalidates one earlier measurement

**Correction first, because it makes an earlier result meaningless.** The four "bank-`$10` `$2116` sites" above were recorded as `$10:F452`, `$10:F492`, `$10:F58A`, `$10:F5CA`. Those are **VRAM destinations, not code addresses**. The code that writes them lives at `$10:8759`, `$10:8779`, `$10:878E`, `$10:87A3` (ROM `0x80759`-`0x807A3`). So the `cov 10F452 10` measurement reported "Never reached" about an address that was never code in the first place — it proved nothing. Re-measured at the real addresses, the finding survives and gets sharper (below), but the old form of it must not be quoted.

**The complete upload map for the intro.** `watch add IO 2116 2 write` plus `IO 4302 3` and `IO 4305 2` over 300 frames, correlated by event index, gives every VRAM transfer with its destination, source and size. Two things fall straight out:

- **VMADD `$7C00` (VRAM `$F800`) is written exactly twice, both by the fill at `$00:E477`/`$00:E4DA`, `$0400` bytes each from a one-byte bank-`$0F` source.** That is the `$0130` constant fill and nothing else. A direct `watch add VRAM FA10 10 write` over 800 frames confirms it from the destination side: sixteen writes, all fill, no upload, ever.
- **The story-text page really is uploaded — to the wrong window for this tilemap.** Job A at `7E:C8AF` DMAs GSU RAM `$70:4C00`, size `$0800`, to VMADD `$7000` = VRAM `$E000`, which is BG3 tiles `$000-$07F`. **The BG3 tilemap references no tile below `$100`.** So the page the tilemap needs and the page the game uploads never meet.

**Enumerating the whole ROM settles the "does any code target `$F800`" question.** Not a regex for one shape this time — every `A9 <lo> <hi> 8D 16 21` in all 2MB, tabulated by destination. There are 49 distinct immediate VMADD destinations. **`$7C00` is not one of them.** The nearest are `$7A29`/`$7A49`/`$7AC6`/`$7AE6` (VRAM `$F452`/`$F492`/`$F58C`/`$F5CC`) and `$7F00` (VRAM `$FE00`). So no immediate-mode site in the game ever targets `$F800`, and the only runtime writes there arrive through the queue path as the fill.

**The BG3 tilemap, dumped in full.** `tilemap VRAM E800 20 1C` shows a coherent, deliberate layout: rows 0-1 and 20/23/26/27 are the blank tile `$130`; rows 2-19 are the frame border, tiles `$100-$15F`; rows 21-22 and 24-25 are the strip, `$1A1-$1A9` and `$1B1-$1B9`, each 9-tile motif tiled three times. The blank tile is `$130` — **the same value as the `$0130` fill constant**, which is worth noticing but is a coincidence of layout, not evidence: `$130` as a tile index resolves to VRAM `$F300`, inside the region that does get real graphics.

**The tilemap is genuine ROM data, verified independently.** `A1 3D A2 3D A3 3D` occurs at ROM `0x1B19DF` and `B1 3D B2 3D B3 3D` at `0x1B19F8`, each exactly once in the 2MB image; `A1 3C …` (the tile-`$0A1` variant that would resolve into the uploaded `$E000` page) does not occur at all. So the GSU decompressor is faithful and the tilemap really does ask for tiles `$1A1+`.

**Where that leaves it.** BG3's character base is `$E000` (BG34NBA `$77`, 2bpp), so tile `$1A1` resolves to VRAM `$FA10`. The border tiles `$100-$15F` resolve to `$F000-$F7FF` and render correctly, which independently confirms the base. Tiles `$180-$1FF` therefore need `$F800-$FFFF`, and `$FC00`-up does receive real uploads (`$FCC0`, `$FD00`, `$FD40`, `$FD80`, `$FDC0`, `$FE40`, from `7E:E400`) — only `$F800-$FBFF` is left holding the fill.

So the question is no longer "which flag never gets set" — `$0CF9` is indeed never set (only cleared, by `$00:82CC`), but the job it gates targets VRAM `$A000`, not `$F800`, so it is not the missing upload. The open question is now narrow and concrete: **what puts the `$70:4C00` page at VRAM `$F800`, given that no immediate-mode VMADD in the ROM names it?** The runtime queue path at `$00:E477`/`$00:E4DA` demonstrably *can* address `$7C00`, so the next measurement is to break on that queue node (`bp write IO 2117 7C`, §3.26) and read where its destination came from — the same "stop at the write and read the state" step that traced a VRAM byte back to a compressed stream in the Super Mario World work.

**Also measured, for the record.** Layer isolation (`layers bg1|bg2|bg3|obj`) at frame 701 puts the strip unambiguously on **BG3**: BG1 carries the quilt picture, BG2 the night sky, BG3 the border *and* the strip, BG4 and OBJ are empty. And the routine containing the four never-run sites is entered part-way through: `cov` over `$10:8700-$1087BF` reports never reached, `$1087C0-$10881F` reports reached from `$1087C4`. The skipped half performs a `$70:4C00 -> $C800` transfer plus the four small `$F452`-`$F5CC` ones; the half that runs covers `$C000` and `$F000-$F3C0`.
