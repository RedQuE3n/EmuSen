# Beryl-HW — Shared Sega Chip Crates

`Beryl-HW` (`EmuSen/Cores/Sega/Beryl-HW/`) is a collection of standalone Rust crates implementing chips and shared hardware assets used by more than one Sega console core. Each chip lives in its own crate so any core can depend only on the hardware it carries.

## Crate Index

| Crate | Hardware / Asset | Status | Reference |
|---|---|---|---|
| `beryl-m68k` | Motorola 68000 CPU (Genesis main CPU, Sega CD sub-CPU) | Implemented (`BUILT = true`) | [Beryl_Cpus.md](Beryl_Cpus.md#beryl-m68k) |
| `beryl-z80` | Zilog Z80 CPU (Genesis sound CPU, Master System / Game Gear main CPU) | Implemented | [Beryl_Cpus.md](Beryl_Cpus.md#beryl-z80) |
| `beryl-sh2` | Hitachi SH-2 / SH7604 CPU (32X, Saturn) | Stub (`BUILT = false`) | [Beryl_Cpus.md](Beryl_Cpus.md#beryl-sh2) |
| `beryl-asm` | Shared assembler front end (build/dev tool only) | Implemented | [Beryl_Cpus.md](Beryl_Cpus.md#beryl-asm) |
| `beryl-sms-vdp` | Sega Master System VDP family (`315-5124`, `315-5246`, `315-5378`), `TMS9918A`, and Genesis mode 4 composition | Implemented | [Beryl_Devices.md](Beryl_Devices.md#beryl-sms-vdp) |
| `beryl-sms-colours` | Master System, Game Gear, and TMS9918A RGBA colour tables | Implemented | [Beryl_Devices.md](Beryl_Devices.md#beryl-sms-colours) |
| `beryl-sn76489` | TI SN76489 / Sega integrated PSG with Game Gear stereo | Implemented | [Beryl_Devices.md](Beryl_Devices.md#beryl-sn76489) |
| `beryl-fm-tables` | Yamaha FM operator 256-entry `LOGSIN` and `EXP` ROM tables | Implemented | [Beryl_Devices.md](Beryl_Devices.md#beryl-fm-tables) |
| `beryl-font` | 47-glyph 5×7 open-firmware font (build tool only) | Implemented | [Beryl_Devices.md](Beryl_Devices.md#beryl-font) |

## Processor Crate Contract

Every CPU crate in `Beryl-HW` (`beryl-m68k`, `beryl-z80`, `beryl-sh2`) follows the same architectural boundary:

- **The bus owns the clock.** A processor keeps no clock of its own. Every external bus cycle and every internal delay is reported to the console's `Bus` implementation in the processor's own terms (68000 bus cycles with function codes, Z80 machine cycles with M1 refresh addresses and 16-bit I/O ports, SH-2 external accesses by width). The bus advances the console's master clock and applies wait states, allowing coupled devices to catch up at the exact cycle of an access. The same CPU crate serves a flat test bus, a console's main bus, or a coprocessor's bus.
- **Zero-overhead debugging via `Observer` / `Unobserved`.** Each processor re-exports `emusen_native::debug::{Observer, Unobserved}`. `step(bus)` delegates directly to `step_observed(bus, &mut Unobserved)`, so normal execution monomorphises away all debugger hooks. Observed stepping reports instruction boundaries, memory stores, calls, exceptions, and returns to the shared debugger hooks.
- **State serialization via `StateWriter` / `StateReader`.** Each implemented processor implements `emusen_native::State` (`write_state` / `read_state`), producing a fixed-layout byte block embedded inside the owning core's save state. Truncated reads return `Err(Truncated)` without mutating CPU state.
- **Disassembly and optional compile-time assembler.** Each implemented processor provides a `disasm` module for debugger disassembly and an optional `asm` module gated behind the `asm` Cargo feature (backed by `beryl-asm`). Assemblers are build/development tools for test programs and open firmware built in `build.rs`; no core runtime library links them.
- **Single-step verification.** Where a hardware-derived or microcode-derived single-step corpus exists (`beryl-m68k`, `beryl-z80`), the unit-test harness lives inside the crate (using `emusen-singlestep` and `EMUSEN_BERYL_CORPUS`). Tests skip cleanly when the corpus environment variable is unset.

## Non-Processor Chip Contract

Shared peripheral chips (`beryl-sn76489`, `beryl-sms-vdp`) have no bus of their own, no instructions to assemble, and nothing for a CPU debugger to step. They follow a narrower device contract:

- **The caller owns the clock.** The crate advances in the chip's own clock units (`advance` or `run`). The console core divides its master crystal and owns all cross-chip synchronization; no console crystal frequency appears inside the device crate.
- **Hardware port writes.** Registers and memories are written through the chip's own port interface one byte at a time, at the moment the console bus delivers them.
- **Raw chip outputs, not console outputs.** Devices output hardware-level values — voltage levels for `beryl-sn76489`, colour/CRAM indices for `beryl-sms-vdp` — never resampled audio or RGB pixels. Console-specific DAC curves, analogue filters, and audio mixers stay in the console core (or zero-dependency data crates like `beryl-sms-colours`).
- **Revision differences via `Variant`.** Behaviour that varies between silicon revisions (such as LFSR tap width or VDP table masks) is selected through a `Variant` constant supplied by the caller.
- **Fixed-layout state blocks.** Device state serializes through `StateWriter` and `StateReader` into the owning console core's save state.

## Zero-Dependency Data and Build Crates

- `beryl-sms-colours` and `beryl-fm-tables` have no dependencies so both console cores and platform debugger packs (`emusen-systems`) can read the same authoritative tables without pulling in chip or runtime code.
- `beryl-asm` and `beryl-font` have no dependencies and are used strictly as build-time or test-time dependencies.
