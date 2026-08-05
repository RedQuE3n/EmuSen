# Moon (NES)

Hardware notes for the `Moon - NES` core (`EmuSen/Cores/Nintendo/Moon - NES/`).

| Page | Covers |
|---|---|
| `Moon_CPU.md` | The 2A03 / 6502 — the one-access-per-cycle bus model, addressing modes and their dummy reads, flags, reset and interrupts, the undocumented opcodes, and validation status |
| `Moon_Core.md` | `MoonCore`/`ICore` — the master clock and why the CPU budget needs an accumulator, the Crystal schedule, the scanline-granularity deviation, named address spaces, save states |
| `Moon_Memory.md` | The CPU address decode, the iNES image, mirroring, the five implemented boards, MMC1's serial register, OAM DMA, controllers, the write-observer seam |
| `Moon_PPU.md` | The 2C02 — the loopy registers, the `$2007` read buffer, palette holes, background and sprite composition, sprite 0 |
| `Moon_APU.md` | The 2A03 sound half — all five channels, the frame sequencer, the non-linear DAC and the output filters, plus why SMB3's "silence" was not a bug |
| `Moon_Debug.md` | `MoonDebugTarget`/`IDebugTarget` — memory spaces, registers, the tile/tilemap decoders, the disassembler, and what a second implementation did and did not prove |
| `Moon_Cheats.md` | Game Genie and raw code decoding, the bit tables and where they came from, how a patch reaches the CPU, and the Game Genie cartridge that is not built |

The core runs but is incomplete: no audio synthesis, no PAL, scanline rather than per-dot PPU timing, and five of the common mappers rather than all of them. Each page states its own gaps.

See `Man pages/Hardware/README.md` for how this documentation set is organized.
