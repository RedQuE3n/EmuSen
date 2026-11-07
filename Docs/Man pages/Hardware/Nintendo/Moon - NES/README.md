# Moon (NES)

Hardware notes for the NES. C# Moon, which these pages were written for, was deleted on 2026-10-16; MoonRT, in `EmuSen/Cores/Nintendo/MoonRT - NES/`, is the NES's only engine (`Moon_Native.md` §9). The pages below that describe the C# core are marked historical at their top: the machine they describe is the one MoonRT implements, under the C# core's names.

| Page | Covers |
|---|---|
| `Moon_CPU.md` | The 2A03 / 6502 — the one-access-per-cycle bus model, addressing modes and their dummy reads, flags, reset and interrupts, the undocumented opcodes, and validation status |
| `Moon_Core.md` | `MoonCore`/`ICore` — the master clock and why the CPU budget needs an accumulator, the core's own timeline, the scanline-granularity deviation, named address spaces, save states |
| `Moon_Memory.md` | The CPU address decode, the iNES image (§2.2: the archaic header, worth more coverage than six boards), mirroring, the sixteen implemented boards, MMC1's serial register, MMC3's IRQ counter (§4.6b: the A12 filter's units and the empty sprite slots), the CPU-cycle hook and the DMA cycles it withholds (§4.8), MMC2's PPU-driven latch (§4.10), Sunsoft-4's CHR nametables (§4.13), RAMBO-1's two clock sources (§4.15), OAM DMA, controllers, the write-observer seam |
| `Moon_PPU.md` | The 2C02 — the loopy registers, the `$2007` read buffer, palette holes, background and sprite composition, sprite 0, and (§7) the `$2000-$2007` write log |
| `Moon_APU.md` | The 2A03 sound half — all five channels, the frame sequencer, the non-linear DAC and the output filters, why SMB3's "silence" was not a bug, and (§8) the re-measured SMB3 comparison and the open `$4010` write gap |
| `Moon_Debug.md` | `MoonDebugTarget`/`IDebugTarget` — memory spaces, registers, the tile/tilemap decoders, the disassembler, and what a second implementation did and did not prove |
| `Moon_Cheats.md` | Game Genie and raw code decoding, the bit tables and where they came from, how a patch reaches the CPU, and the Game Genie cartridge that is not built |
| `Moon_TestRoms.md` | The blargg `$6000` reporting protocol, the `--testroms` runner, why a verdict waits for the running status, and what the runner does not cover |
| `Moon_PAL.md` | MoonRT's PAL NES and Dendy (2026-10-17): every fact in which the three consoles differ and its source, the header's byte 12 and the region setting, 3.2 dots a cycle, 312 lines, the 2C07's border, the raw picture's emphasis bits left for the CRT filter, the 2A07's APU tables checked against NES_MiSTer and blargg's PAL suite, its DMA on an instruction's first cycle, the state's version 7, the tests and mutants and the bench |
| `Moon_Native.md` | The port to Rust as MoonRT: the measured C# baseline (and the tenth of a core where Moon falls below full speed), the speed prediction stated before the work in the light of Mercury's refuted one, why Moon's renderer is part of its machine, the state format with its twice-written bus, the stages, the defects the review found (the APU running ahead of the picture by every DMA cycle), NES_MiSTer as the auditor of every component before it is ported (§3.7: its provenance, the reading rule, the first rules to audit), and each stage's record (§8; stage 1, the state byte for byte, with its mutants; §8.6, the bus cycle out of line, which took the dist build from seven minutes to four seconds with every output unchanged); §9, the goldens recorded from MoonRT, the corpus against the C# core's last transcript, and the C# core's retirement with what left with it |

The core runs, renders and makes sound, but is incomplete: no emphasis bits in its RGB frame, no expansion audio, and ten of the common mappers rather than all of them. The PPU's *clock* is per dot as of 2026-08-05; its *renderer* is still per line. Each page states its own gaps.

See `Man pages/Hardware/README.md` for how this documentation set is organized.
