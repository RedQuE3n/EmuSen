# Moon (NES)

Hardware notes for the `Moon - NES` core (`EmuSen/Cores/Nintendo/Moon - NES/`).

| Page | Covers |
|---|---|
| `Moon_CPU.md` | The 2A03 / 6502 — the one-access-per-cycle bus model, addressing modes and their dummy reads, flags, reset and interrupts, the undocumented opcodes, and validation status |
| `Moon_Core.md` | `MoonCore`/`ICore` — the master clock and why the CPU budget needs an accumulator, the core's own timeline, the scanline-granularity deviation, named address spaces, save states |
| `Moon_Memory.md` | The CPU address decode, the iNES image, mirroring, the ten implemented boards, MMC1's serial register, MMC3's IRQ counter (§4.6b: the A12 filter's units and the empty sprite slots), OAM DMA, controllers, the write-observer seam |
| `Moon_PPU.md` | The 2C02 — the loopy registers, the `$2007` read buffer, palette holes, background and sprite composition, sprite 0, and (§7) the `$2000-$2007` write log |
| `Moon_APU.md` | The 2A03 sound half — all five channels, the frame sequencer, the non-linear DAC and the output filters, why SMB3's "silence" was not a bug, and (§8) the re-measured SMB3 comparison and the open `$4010` write gap |
| `Moon_Debug.md` | `MoonDebugTarget`/`IDebugTarget` — memory spaces, registers, the tile/tilemap decoders, the disassembler, and what a second implementation did and did not prove |
| `Moon_Cheats.md` | Game Genie and raw code decoding, the bit tables and where they came from, how a patch reaches the CPU, and the Game Genie cartridge that is not built |
| `Moon_TestRoms.md` | The blargg `$6000` reporting protocol, the `--testroms` runner, why a verdict waits for the running status, and what the runner does not cover |

The core runs, renders and makes sound, but is incomplete: no PAL, no emphasis bits, no expansion audio, and ten of the common mappers rather than all of them. The PPU's *clock* is per dot as of 2026-08-05; its *renderer* is still per line. Each page states its own gaps.

See `Man pages/Hardware/README.md` for how this documentation set is organized.
