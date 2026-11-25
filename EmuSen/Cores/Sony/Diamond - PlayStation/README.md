# Diamond (Sony PlayStation)

Shipping Core ABI v1 Sony PlayStation (`psx`) core in Rust. Uses [`BlackMoon-HW`](../BlackMoon-HW/README.md) (`blackmoon-r3000`, `blackmoon-gte`, `blackmoon-spu`), [`emusen-disc`](../../Shared/emusen-disc/), [`emusen-gpu`](../../Shared/emusen-gpu/), and [`emusen-psx-card`](../../Shared/emusen-psx-card/).

## Crate Layout

- `src/` (`diamond`) — accurate PlayStation machine: bus/wait states, 4 KiB decoded I-cache, 7-channel DMA, root-counter timers, IRQ, SIO0 (digital/DualShock/multitap/memory cards), software GPU, CD-ROM (XA-ADPCM, CD-DA, subQ/SBI), MDEC, SPU, exact idle skipping, and `DMND` save states.
- `precision/` — sub-pixel vertex and depth tracking side-channel (`precision` enhancement).
- `renderer/` — Vulkan compute multi-resolution renderer (`1×`–`4×` internal resolution, true color, texture filtering, host device lend).
- `kernel/`, `kernel-shell/`, `guest/`, `kernel-harness/` — clean-room `no_std` MIPS open BIOS kernel (`diamond-kernel.bin`), embedded memory-card manager shell (`diamond-shell.exe`), and standalone verification harness.
- `v1/` (`diamond-v1`) — published `libdiamond.so` Core ABI v1 library built by `xtask/libraries.tsv`.

## Documentation

- [Diamond System Architecture](../../../../Docs/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_System.md)
- [Diamond Video, Media, Audio, and Enhancements](../../../../Docs/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_VideoAndMedia.md)
- [Diamond Open BIOS Kernel and Shell](../../../../Docs/Man%20pages/Hardware/Sony/Diamond%20-%20PlayStation/Diamond_OpenKernel.md)
- [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme) (Black Moon Clan codename: Prince Demande/Diamond)
