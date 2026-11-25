# Sapphire — Sony PlayStation 2 Core

Shipping Rust PlayStation 2 core implementing EmuSen Core ABI v1 (`cdylib` + `rlib`, system `"ps2"`). Requires a retail PlayStation 2 BIOS image (`ps2-0230{j,a,e}-20080220.bin` or any retail Redump revision) and opens `.elf`, `.iso`, `.chd`, `.cue`, and `.bin` images or boots empty to OSDSYS.

## Architecture & Modules

- **EE & Vector Units (`src/ee/`, `src/vu/`, `src/float.rs`, `src/blocks.rs`)**: R5900 interpreter and timing model (128-bit MMI, COP0, 48-entry TLB, I/D caches, scratchpad, PS2 single-precision float), VU0 (COP2 macro + micro) and VU1 (micro, EFU, `XGKICK`), tailored x86-64 EE block recompiler (`emusen-assembler` + `emusen-mips-planner`), and Cranelift VU0/VU1 microprogram JIT (`emusen-jit`).
- **Graphics & Video (`src/gif/`, `src/vif/`, `src/gs/`, `src/pcrtc/`, `src/ipu/`)**: GIF (PATH1/2/3, PACKED/REGLIST/IMAGE), VIF0/VIF1 (UNPACK, double buffering), 4 MiB swizzled GS with multi-threaded CPU workers and Vulkan compute renderer (`src/gs/device/`, `shaders/walk.comp` via `emusen-gpu`), scanline PCRTC output, and MPEG-2/1 IPU decoder.
- **IOP & I/O (`src/iop/`, `src/sif/`, `src/dmac.rs`, `src/intc.rs`, `src/timer.rs`, `src/spu2/`, `src/cdvd/`, `src/sio2/`)**: R3000A IOP (`blackmoon-r3000`), SIF mailboxes/FIFOs, 10-channel EE DMAC, EE INTC and timers, dual-core 48 kHz SPU2 (`blackmoon-spu`), CDVD/NVM/RTC (`emusen-disc`), and SIO2 DualShock 2, multitap, and memory cards (`emusen-ps2-card`).
- **System & ABI (`src/board.rs`, `src/machine.rs`, `src/state.rs`, `src/idle.rs`, `src/debugger.rs`, `src/v1.rs`)**: Event-driven scheduler with live Two Faces batching (`board.round`), direct ELF fast-boot, `SPH2` save states, proof-based idle skipping, 4-processor debugger, and Core ABI v1 exports.

## Build & Test

```bash
cargo test --release
cargo build --profile dist
```

## Hardware Manuals

- [Sapphire_EeAndVu.md](../../../../Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_EeAndVu.md) — EE R5900, `float`/`float_rz`, VU0/VU1, EE/VU recompilers, scheduler, and live Two Faces split
- [Sapphire_GsAndVideo.md](../../../../Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_GsAndVideo.md) — GIF, VIF0/1, GS swizzling, rasteriser, Vulkan compute GS, PCRTC, and IPU
- [Sapphire_IopAndIo.md](../../../../Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_IopAndIo.md) — IOP, SIF, EE DMAC/INTC/timers, SPU2, CDVD, and SIO2
- [Sapphire_System.md](../../../../Docs/Man%20pages/Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_System.md) — Clocks, memory maps, MCH init, BIOS/ELF fast-boot, `SPH2` states, Core ABI v1, idle skipping, and gates

Black Moon Clan codename (Saphir) — see [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).
