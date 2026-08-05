# Venus (SNES) — NEC DSP (DSP-1/1B/2/3/4, ST010/ST011)

Covers everything under `Cores/Nintendo/Venus - SNES/Coprocessors/NecDsp/`, plus `CoprocessorOverlayMapper` and the detection code in `Cartridge`. This is the third coprocessor family this core implements, after the SA-1 (`Venus_SA1.md`) and the SuperFX (`Venus_SuperFX.md`).

> **Status: complete and verified against real firmware.** The interpreter, the memory map, the host handshake, and save states are all implemented and tested. All seven firmwares boot to their idle loop, and the DSP-1B computes the documented answer for its multiply command across a range of operands — see `NecDspRealFirmwareTests`. The chip cannot run without a firmware dump, which this project does not and will not ship; §2 says where to put one. Without it the cartridge falls back to a plain LoROM/HiROM map and the game runs with the coprocessor absent (which for most DSP-1 games means the 3D maths never answers).

---

## 1. What the chip is, and how it gets built

Seven SNES enhancement chips are the same processor. The **NEC uPD7725** is a 16-bit Harvard-architecture DSP with a hardware multiplier, sold as a general-purpose part; Nintendo bought it with different mask-programmed firmware and a different sticker each time:

| Variant | Die | Clock | Notable games |
|---|---|---|---|
| DSP-1 | uPD7725 | 7.6MHz | Pilotwings |
| DSP-1B | uPD7725 | 7.6MHz | Super Mario Kart, most others (bug-fixed revision) |
| DSP-2 | uPD7725 | 7.6MHz | Dungeon Master |
| DSP-3 | uPD7725 | 7.6MHz | SD Gundam GX |
| DSP-4 | uPD7725 | 7.6MHz | Top Gear 3000 |
| ST010 | uPD96050 | 11MHz | F1 ROC II |
| ST011 | uPD96050 | 22MHz | Hayazashi Nidan Morita Shougi |

The uPD96050 is the same instruction set with a bigger program ROM (16384 instructions vs 2048), a bigger data ROM, 2KB of **battery-backed** data RAM instead of 512 bytes, and an 8-deep call stack instead of 4. `NecDspProfile` is the whole of the difference; `NecDsp` itself is variant-agnostic.

### Detection

The header's **cartridge type** (`+$16`) with a low nibble of 3 or more:

- high nibble `$0` → a DSP. **Which** one is not in the header at all — every DSP cartridge declares the same byte. The only discriminator is the **title** (`+$00`, 21 bytes), so `Cartridge.DetectNecDsp` keys on it and defaults to DSP-1B, which is what the overwhelming majority of cartridges actually carry.
- high nibble `$F` with **chip subtype** `$01` at header `−$01` → an ST01x, again split by title (only *2DAN MORITA SHOUGI* is an ST011).

The title is read as **Latin-1, not UTF-8**: SD Gundam GX's title is half-width katakana in the SNES's own single-byte encoding, and only a byte-for-byte reading matches the name the DSP-3 is keyed on.

---

## 2. Firmware

The program and data ROMs are **masked into the die**. They were never on the cartridge bus, the S-CPU cannot read them, and no amount of emulating the cartridge recovers them. The chip is inert without a dump, so `Cartridge.BuildNecDsp` looks in two places and gives up quietly if neither has it.

A dump is **program bytes followed by data bytes**, one blob:

| Variant | Program | Data | Total |
|---|---|---|---|
| DSP-1/1B/2/3/4 | `$1800` (2048 × 3 bytes) | `$800` (1024 × 2 bytes) | `$2000` |
| ST010/ST011 | `$C000` (16384 × 3) | `$1000` (2048 × 2) | `$D000` |

Instructions are **24-bit, little-endian, three bytes each**; data words are 16-bit little-endian. `NecDsp`'s constructor decodes the whole program ROM into a `uint[]` once, so the hot loop never re-assembles a three-byte opcode.

### 2.1 Appended to the ROM file

Most DSP ROM images in circulation carry the firmware glued to the end. `NecDspFirmware.EmbeddedSize` recognises it from the file size alone: `size & $7FFF == $2000` means a uPD7725 dump is attached, `size & $FFFF == $D000` a uPD96050 one. This is a heuristic — a ROM of exactly the wrong size would false-positive — so it is only consulted for a cartridge already identified as carrying a DSP.

When one is found it is **removed from the addressable ROM**. `Cartridge` reallocates `_rom` shorter, because on hardware those bytes are not on the bus and a game mirroring past the end of its own ROM must see open bus, not firmware.

### 2.2 home/Firmware

Failing that, `home/Firmware/` (gitignored) is checked for either a combined `dsp1.rom` (or `dsp1b`, `dsp2`, `dsp3`, `dsp4`, `st010`, `st011`) or the split pair `dsp1.program.rom` + `dsp1.data.rom`. A file of the wrong size is ignored rather than padded — a truncated dump would run as garbage and be much harder to diagnose than a missing one.

The combined form goes through the **core-agnostic firmware layer** (`EmuSen_Firmware.md`), which is also what lets the Avalonia frontend offer a file picker when a dump is missing, and what installs the picked file so nothing asks twice. The split pair is a NEC-DSP-specific convention and stays local to `NecDspFirmware`. `NecDspFirmware.RequestFor(variant)` is the bridge: it turns a variant into the generic `FirmwareRequest` the shared layer understands.

`Cartridge.FirmwareRequirements(romPath)` answers "what will this ROM need?" from the header alone, without loading — a cartridge with its firmware already appended correctly reports **nothing**.

### 2.3 The other dump format, which is *not* supported

There are two layouts in circulation and only one is read here.

| | Program | Data | DSP-1 total |
|---|---|---|---|
| **bsnes / Mesen** (supported) | 3 bytes per instruction, **little-endian** | 2 bytes per word, little-endian | `$2000` |
| **MAME** (not supported) | 4 bytes per instruction — 24 bits **big-endian** plus an `$FF` pad | 2 bytes per word, **big-endian** | `$2800` |

A MAME set is easy to spot: the file is `$2800` rather than `$2000` (or `$11000` rather than `$D000` for an ST01x), and every fourth byte of the program half is `$FF`. `NecDspFirmware` rejects it on size, so the symptom is "firmware not found" rather than a garbage run.

Supporting it would be a second branch in `FromBlob` keyed on the total length — roughly ten lines. It has not been written because nothing needs it: BIOS packs that carry the MAME sets generally carry the split `.program.rom`/`.data.rom` pair too, and that pair is already the supported format.

---

## 3. Memory map and registers

Unlike the SA-1 and the GSU, a NEC DSP does **not** replace the cartridge's memory map. It claims a small window and leaves everything else alone, which is what `CoprocessorOverlayMapper` exists for: it asks the chip first and falls through to a plain `LoRomMapper`/`HiRomMapper` on a miss.

| Board | Window | SR select |
|---|---|---|
| LoROM | `$30-$3F/$B0-$BF:8000-FFFF` | address bit 14 |
| LoROM (alt) | `$60-$6F/$E0-$EF:0000-7FFF` | address bit 14 |
| HiROM | `$00-$1F/$80-$9F:6000-7FFF` | address bit 12 |
| ST010/ST011 | `$60/$E0:0000-0FFF` | address bit 0 |

The LoROM "alt" window is real hardware, not a compatibility hack: Super Bases Loaded 2 reaches its DSP through `$60` rather than `$30`.

On an ST01x, banks `$68-$6F/$E8-$EF:0000-0FFF` are a **third** region — the chip's 2KB data RAM, exposed to the S-CPU as 4096 bytes of little-endian word pairs.

Because a mapper's `CartridgeAddress.Offset` is an `int`, the DSP stores the **whole 24-bit address** in it rather than just the low 16 bits; the ST01x needs the bank to tell its RAM window from its register pair.

### 3.1 The two registers

There are only two, and the select bit above picks between them:

- **DR**, the data register — a bidirectional 16-bit mailbox.
- **SR**, the status register — read-only from the host, and read as its **high byte alone**; the low byte is firmware-private.

### 3.2 The handshake

`RQM` (SR bit 15) means *the chip wants the host to do something*. The firmware raises it by writing DR (or by reading DR as source `$08`), and the host clears it by completing a DR transfer.

DR is 16 bits behind an 8-bit bus, so a transfer is two accesses. `DRS` (SR bit 12) tracks which half the host is on: the first access sets `DRS` and moves the low byte, the second clears `DRS` **and** `RQM` and moves the high byte. `DRC` (SR bit 10) switches the port to 8-bit mode, where every access is a whole transfer.

This is why an SR read mid-transfer returns `$90` (RQM | DRS) rather than `$80` — a detail worth knowing before assuming a handshake has gone wrong.

There is **no IRQ line** to the S-CPU. The game polls SR.

---

## 4. Running the chip

### 4.1 Timebase

The DSP has its own crystal, so unlike the SA-1 it cannot take the S-CPU's master-clock figure directly. `NecDsp.Run` keeps its budget in **master clocks scaled by the DSP's own rate**: each call adds `masterClocks × ClockHz` and retires one instruction per `MasterClockHz` accumulated. That is exact integer arithmetic with no drift, and `VenusCore.LoadRom` sets `MasterClockHz` from the cartridge's region.

Every instruction costs exactly one cycle. The real chip has multi-cycle memory accesses, but the S-CPU almost always *waits* on the DSP rather than racing it, so the cost model changes emulated speed rather than correctness.

`VenusCore.RunFrame` calls `Run()` with the same per-instruction master-clock figure `Cpu.Step()` just returned, exactly as it does for the SA-1 and the GSU. The DSP is therefore at most one S-CPU instruction behind when a register access lands — close enough that no game can observe the difference, and it avoids threading an absolute clock through `Cartridge`.

### 4.2 The multiplier

`M` and `N` are not written by any instruction. They are the **combinational** output of `K × L`, re-latched after *every* instruction from whatever `K` and `L` currently hold: `M` is the product's top 16 bits and `N` the bottom, both after a one-bit left shift (the product is treated as 1.15 fixed point). A great deal of DSP-1 firmware depends on loading `K`, then loading `L`, and finding the product already waiting on the next instruction.

### 4.3 The RQM idle loop

Firmware that has published a result spins on a one-instruction branch back onto itself testing RQM, sometimes for tens of thousands of cycles while the S-CPU gets round to reading DR. Stepping that loop is pure waste.

`Jump()` detects the shape — a taken RQM-conditioned branch whose target is the branch itself — and sets `_inRqmLoop`, after which `Run()` drops its budget on the floor instead of iterating. Any DR access clears it. **SR reads deliberately do not**, because polling SR is exactly what the S-CPU does while the chip is legitimately idle.

---

## 5. The instruction set

Every instruction is 24 bits, and bits 23-22 pick one of four forms.

| Bits 23-22 | Form | Meaning |
|---|---|---|
| `00` | OP | ALU operation, then source → destination |
| `01` | RT | the same, then pop the stack into PC |
| `10` | JP | branch, call, or return-to-`SO` |
| `11` | LD | 16-bit immediate → destination |

### 5.1 OP / RT field layout

| Bits | Field |
|---|---|
| 21-20 | ALU second operand: RAM / instruction source / M / N |
| 19-16 | ALU operation |
| 15 | accumulator select (A or B) |
| 14-13 | DP low-nibble step: none / +1 / −1 / clear |
| 12-9 | DP high-nibble XOR modifier |
| 8 | decrement RP |
| 7-4 | source |
| 3-0 | destination |

DP's low nibble steps **without carrying** into the high nibble, and the high nibble is XOR-modified rather than assigned — that pair of quirks is what makes the DSP's 16-word table walks work. DP is not stepped when the instruction's destination *was* DP, and RP is not decremented when the destination was RP.

### 5.2 ALU

`$00` no-op, `$01` OR, `$02` AND, `$03` XOR, `$04` SUB, `$05` ADD, `$06` SBC, `$07` ADC, `$08` DEC, `$09` INC, `$0A` NOT, `$0B` arithmetic shift right, `$0C` shift left through carry, `$0D` shift left 2 filling with 1s, `$0E` shift left 4 filling with 1s, `$0F` byte swap.

Each accumulator carries six flags: `C`, `Z`, `OV0`, `OV1`, `S0`, `S1`. `S0` is the result's sign; **`S1` is a latched sign that only tracks `S0` while `OV1` is clear**, and `OV1` itself is sticky across consecutive overflows in a way that only resets when two overflows agree in sign. That is the saturation machinery the firmware's clamping code branches on, and it is the single easiest part of this chip to get subtly wrong.

`SBC`/`ADC` take their carry-in from the **other** accumulator's carry flag, not their own.

### 5.3 Source and destination encodings

| Code | Source | Destination |
|---|---|---|
| `$0` | TRB | (none) |
| `$1` | A | A |
| `$2` | B | B |
| `$3` | TR | TR |
| `$4` | DP | DP |
| `$5` | RP | RP |
| `$6` | data ROM at RP | DR (raises RQM) |
| `$7` | `$8000 − A.S1` (saturation constant) | SR (bits `$907C` are chip-owned) |
| `$8` | DR, raising RQM | SO |
| `$9` | DR, quietly | SO |
| `$A` | SR | K |
| `$B` | SI | K, and L ← data ROM at RP |
| `$C` | SI | L, and K ← RAM at `DP\|$40` |
| `$D` | K | L |
| `$E` | L | TRB |
| `$F` | RAM at DP | RAM at DP |

The two paired loads at `$B`/`$C` are the chip's whole reason for being fast at dot products: one instruction fills both multiplier inputs.

### 5.4 JP

Bits 21-13 are the condition code, bits 12-2 the target within an 8KB half, bits 1-0 the 2KB bank. The `$2000` bit of PC is **preserved** by a conditional branch and **assigned** by `$100`/`$101` (jump) and `$140`/`$141` (call), which is how the uPD96050 reaches its larger program ROM. Serial in/out are not emulated — no SNES cartridge wires them — but their condition codes exist because firmware branches on them.

---

## 6. Battery-backed RAM

Only the uPD96050 pair has it. On an ST010/ST011 the `.srm` is **the DSP's own 2KB data RAM**, not cartridge SRAM — so `Cartridge` allocates no SRAM at all for those boards and routes `LoadSram`/`SaveSram` through `NecDsp.ImportBatteryRam`/`ExportBatteryRam` instead, which serialise it as little-endian word pairs.

The header on an ST01x cartridge does declare an SRAM size. It is ignored: the chip that holds it is the DSP.

---

## 7. Save states

Format **v3** appends the DSP's state after the SA-1's and the GSU's, and only for a cartridge that carries one — see `EmuSen_Save_States.md` §3. Firmware is excluded (`[SkipInState]`), because `LoadRom` re-reads it before any state load, exactly as `Cartridge` handles ROM bytes.

A v2 file cannot be a NEC DSP cartridge, since none could run when v2 was written, so no migration path is needed.

---

## 8. What is not implemented

- **Serial in/out.** No SNES cartridge wires these pins. The condition codes are decoded; the registers never change.
- **Per-instruction cycle costs.** Every instruction is one cycle — see §4.1 for why this is safe here.
- **The `$P` interrupt.** `EnableInterrupt` (SR bit 7) is stored and never acted on; there is no interrupt line on an SNES cartridge to act on it with.
- **DSP-3 and DSP-4 verification.** The interpreter is variant-agnostic and there is no reason either should behave differently, but neither has been run against real firmware here.

---

## 9. The debug seam

The chip is a first-class debug target: `bp dsp`, `step dsp`, `bt dsp`, `cov dsp`,
`regs dsp` and `disasm DSPPRG` all work. Earlier revisions of this project said the
DSP could not be halted "because its firmware runs from mask ROM this core steps as
a block". That was never quite the reason. The mask ROM makes the firmware
*unreadable from the cartridge*, which is a different problem, and one a debugger
does not care about — `Step()` already runs exactly one instruction at a time, and
one instruction at a time is all a breakpoint needs.

**Addresses are word indices, not bytes.** The DSP's PC counts 24-bit instruction
words, so `bp dsp add 100` breaks on the 257th instruction and `HaltedAddress`
reports the same number. `disasm DSPPRG` indexes the same way, so a number seen in
one can be pasted into the other without conversion. This is deliberately *not*
multiplied by three to look like a byte address: the DSP's own PC is the only
number a user ever has to reason about, and inventing a second coordinate system
for it would mean every value needed to be labelled with which one it was in.

**`BreakpointChecker`** has the same pull-hook shape as `Sa1.BreakpointChecker`
(Venus_SA1.md §11.5) and `SuperFx.BreakpointChecker`. It is consulted inside
`Run()`'s budget loop, *before* the instruction executes, and a halt returns with
`_clockBudget` intact so resuming re-enters the same loop rather than losing the
unspent clocks. `ResumeFromBreakpoint()` sets a skip-one-check flag so `continue`
leaves the breakpoint it is sitting on instead of instantly re-halting on the same
PC. `VenusCore.HaltedCpu` carries `"dsp"` for the same reason it carries `"sa1"`:
arming the wrong chip's flag would let this one re-break forever.

**The call stack is real, not inferred.** The uPD7725 has a hardware stack with a
CALL/RET pair — jump types `$140`/`$141` push, and the `$400000` opcode class
executes-and-returns. `CallObserver` and `ReturnObserver` fire from exactly those
two sites, which is why `bt dsp` reports frames the chip actually pushed rather
than a guess reconstructed from a memory stack. That in turn feeds `cov dsp funcs`,
so a firmware nobody has source for can still be mapped by running it: every
routine the game actually calls shows up with an entry count.

**The RQM idle loop is a blind spot, on purpose.** `Run()` returns immediately
while `_inRqmLoop` is set (§4.3), so no breakpoint can fire during it. This is
correct — the chip is not executing anything worth stopping on — but it does mean
a breakpoint placed *inside* the two-instruction spin will never be hit. If that
is what you need, clear the loop by completing the DR handshake first.

**Cost.** One null test per instruction with nothing wired, which is what the SA-1
and GSU already pay. The observers are null unless a debug target is attached.

---

## 10. Where to look when something is wrong

1. **The game hangs at a loading screen.** Almost always missing firmware — check the console for `firmware not found`, which now names the exact path it wants. In the Avalonia frontend you should have been offered a picker instead; if you weren't, see `EmuSen_Firmware.md` §6. `Cartridge` deliberately continues without the chip rather than refusing the ROM, so the symptom is a hang rather than an error.
2. **3D geometry is subtly wrong rather than absent.** Suspect the `S1`/`OV1` latch in §5.2 before anything else. It is the only part of the ALU whose behaviour is not obvious from the operation name, and every clamped coordinate passes through it.
3. **The game hangs immediately after a DSP call.** Check `_inRqmLoop` (§4.3). If the loop-shape detection fires on a branch that isn't actually an idle spin, the chip stops dead.
4. **Values arrive byte-swapped.** The `DRS` half-transfer state (§3.2) has almost certainly been desynchronised by a stray DR access — a debugger read of `$30:8000` counts as a real access and *will* corrupt a transfer in progress.
