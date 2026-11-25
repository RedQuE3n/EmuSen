# EmuSen Hardware Reference Manuals

Factual hardware specifications, chip contracts, and accuracy boundaries for EmuSen's Rust cores (`EmuSen/Cores/`).
For project architecture, Core ABI, and naming conventions, see [`Docs/EmuSen Manual/README.md`](../EmuSen%20Manual/README.md). For DianaSh CLI commands, see [`Docs/man/`](../man/).

## Shared Crates (`EmuSen/Cores/Shared/`)
- [`Hardware/Shared/Disc.md`](Hardware/Shared/Disc.md) — `emusen-disc`: CD (ECMA-130, CUE/BIN/ISO/WAV/CHD, subQ, SBI) and DVD (ECMA-267) sector reading.
- [`Hardware/Shared/Gpu.md`](Hardware/Shared/Gpu.md) — `emusen-gpu`: shared Vulkan compute device, buffer/dispatch API, and host-lent device seam.
- [`Hardware/Shared/MediumFormats.md`](Hardware/Shared/MediumFormats.md) — `emusen-{psx-card,n64-pak,mcd-bram,ps2-card}`: `no_std` save-medium filesystems.
- [`Hardware/Shared/CpuTables.md`](Hardware/Shared/CpuTables.md) — `emusen-mips`, `emusen-6502`, and `emusen-singlestep`: ISA tables, disassemblers, and test harness.
- [`Hardware/Shared/Recompiler.md`](Hardware/Shared/Recompiler.md) — `emusen-jit`, `emusen-assembler`, and `emusen-mips-planner`: shared JIT runtime and x86-64 emitter.

## Nintendo (`EmuSen/Cores/Nintendo/`)
- [`SilverMillennium-HW/README.md`](Hardware/Nintendo/SilverMillennium-HW/README.md) — `silvermillennium-sm83` and Nintendo shared-chip placement rules.
- [`Moon - NES/Moon_Hardware.md`](Hardware/Nintendo/Moon%20-%20NES/Moon_Hardware.md) — `MoonRT`: 2A03 CPU, 2C02 PPU, APU, iNES/mappers, raw-picture format, and accuracy boundaries.
- [`Moon - NES/Moon_PAL.md`](Hardware/Nintendo/Moon%20-%20NES/Moon_PAL.md) — `MoonRT`: PAL (2A07/2C07) and Dendy clock, PPU, APU, and DMA differences.
- [`Mercury - GB-GBC/Mercury_Hardware.md`](Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Hardware.md) — `MercuryRT`: SM83, LCD/PPU, APU, timers/DMA, MBC boards, and accuracy boundaries.
- [`Mercury - GB-GBC/Mercury_CgbAndModels.md`](Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_CgbAndModels.md) — `MercuryRT`: CGB extensions, HDMA, double speed, and DMG-on-CGB compatibility.
- [`Venus - SNES/Venus_Hardware.md`](Hardware/Nintendo/Venus%20-%20SNES/Venus_Hardware.md) — `VenusRT`: 65816, memory/DMA/HDMA, S-PPU, SPC700/S-DSP, and accuracy boundaries.
- [`Venus - SNES/Venus_Coprocessors.md`](Hardware/Nintendo/Venus%20-%20SNES/Venus_Coprocessors.md) — `VenusRT`: SA-1, SuperFX (GSU), OBC1, and NEC µPD77C25/µPD96050 LLE.
- [`Venus - SNES/Venus_DspHle.md`](Hardware/Nintendo/Venus%20-%20SNES/Venus_DspHle.md) — `VenusRT`: open HLE replacements for DSP-1/1B/2/4 and ST010.
- [`Mars - N64/Mars_CpuAndMemory.md`](Hardware/Nintendo/Mars%20-%20N64/Mars_CpuAndMemory.md) — `MarsRT`: VR4300, COP0/TLB/FPU, memory/PI/SI/AI/PIF/saves, and block JIT.
- [`Mars - N64/Mars_Rsp.md`](Hardware/Nintendo/Mars%20-%20N64/Mars_Rsp.md) — `MarsRT`: RSP scalar core, vector unit (VU), DMA, and SIMD/block execution.
- [`Mars - N64/Mars_RdpAndVi.md`](Hardware/Nintendo/Mars%20-%20N64/Mars_RdpAndVi.md) — `MarsRT`: RDP pipeline, Vulkan compute multi-res GPU, and VI filtering/timing.

## Sega (`EmuSen/Cores/Sega/`)
- [`Beryl-HW/README.md`](Hardware/Sega/Beryl-HW/README.md) — Sega shared-chip crate contracts (bus-owns-clock, `Observer`, non-CPU chips, `beryl-asm`).
- [`Beryl-HW/Beryl_Cpus.md`](Hardware/Sega/Beryl-HW/Beryl_Cpus.md) — `beryl-m68k`, `beryl-z80`, `beryl-sh2` (stub), and `beryl-asm`.
- [`Beryl-HW/Beryl_Devices.md`](Hardware/Sega/Beryl-HW/Beryl_Devices.md) — `beryl-sms-vdp`, `beryl-sms-colours`, `beryl-sn76489`, `beryl-fm-tables`, and `beryl-font`.
- [`Endou - Master System/Endou_Hardware.md`](Hardware/Sega/Endou%20-%20Master%20System/Endou_Hardware.md) — `endou`: Master System, Mark III, SG-1000, YM2413 OPLL, and open boot ROM.
- [`Endou - Master System/Endou_GameGear.md`](Hardware/Sega/Endou%20-%20Master%20System/Endou_GameGear.md) — `endou` (`gg`/Jadeite module): Game Gear LCD, stereo, EEPROM, and open boot ROM.
- [`Nephrite - Genesis/Nephrite_Genesis.md`](Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Genesis.md) — `nephrite`: Genesis/Mega Drive buses, VDP/FIFO, YM2612 tables/LFO, and PSG.
- [`Nephrite - Genesis/Nephrite_SegaCD.md`](Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_SegaCD.md) — `nephrite`: attachment trait interface, Sega CD hardware, open BIOS, and BRAM/`_BURAM`.
- [`Nephrite - Genesis/Nephrite_32X.md`](Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_32X.md) — `nephrite`: 32X attachment stub contract and unbuilt status.

## Sony (`EmuSen/Cores/Sony/`)
- [`BlackMoon-HW/README.md`](Hardware/Sony/BlackMoon-HW/README.md) — Sony shared-chip crate contracts (bus-owns-clock, COP2, `Move::Shift` v2).
- [`BlackMoon-HW/BlackMoon_R3000.md`](Hardware/Sony/BlackMoon-HW/BlackMoon_R3000.md) — `blackmoon-r3000`: MIPS I R3000A, load-delay slots, COP0, multiplier, and quirks.
- [`BlackMoon-HW/BlackMoon_GTE.md`](Hardware/Sony/BlackMoon-HW/BlackMoon_GTE.md) — `blackmoon-gte`: PlayStation GTE registers, commands, fixed-point saturation, and interlock.
- [`BlackMoon-HW/BlackMoon_SPU.md`](Hardware/Sony/BlackMoon-HW/BlackMoon_SPU.md) — `blackmoon-spu`: shared PS1/PS2 voice engine (ADPCM, Gaussian, ADSR, reverb).
- [`Diamond - PlayStation/Diamond_System.md`](Hardware/Sony/Diamond%20-%20PlayStation/Diamond_System.md) — `diamond`: bus/I-cache/DMA/timers/IRQ, SIO0/pads/cards, and exact speed levers.
- [`Diamond - PlayStation/Diamond_VideoAndMedia.md`](Hardware/Sony/Diamond%20-%20PlayStation/Diamond_VideoAndMedia.md) — `diamond`: GPU, Vulkan 1x-4x renderer & precision layer, CD-ROM, MDEC, and SPU.
- [`Diamond - PlayStation/Diamond_OpenKernel.md`](Hardware/Sony/Diamond%20-%20PlayStation/Diamond_OpenKernel.md) — `diamond-kernel` & shell: open BIOS memory map, boot, hand-over, and verified §5 call table.
- [`Sapphire - PlayStation 2/Sapphire_EeAndVu.md`](Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_EeAndVu.md) — `sapphire`: R5900/MMI/COP0/TLB/float-RZ, VU0/VU1, EE/VU1 JIT, and Two Faces (`round`).
- [`Sapphire - PlayStation 2/Sapphire_GsAndVideo.md`](Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_GsAndVideo.md) — `sapphire`: GIF/VIF, GS local memory/raster/Vulkan compute, PCRTC, and IPU.
- [`Sapphire - PlayStation 2/Sapphire_IopAndIo.md`](Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_IopAndIo.md) — `sapphire`: IOP/SIF/DMAC/INTC/timers, SPU2, CDVD/NVM/RTC, and SIO2/DS2/MC.
- [`Sapphire - PlayStation 2/Sapphire_System.md`](Hardware/Sony/Sapphire%20-%20PlayStation%202/Sapphire_System.md) — `sapphire`: memory map, BIOS/ELF fast-boot, SPH2 state, and accuracy gates G1–G11.
