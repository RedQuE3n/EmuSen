# Endou (Sega Master System)

The Sega Master System core, crate `endou`, in Rust on the core ABI v1, with the SG-1000's cartridges as a second
system, offered to players since stage 7 (2026-10-19). Stage 2 (2026-10-19) runs the machine: Beryl's Z80 on the bus, the cartridge boards,
the I/O chip, the Pause and Reset buttons, a frame at a time; stage 3 (2026-10-19) gives the Master System its picture
through Beryl's `beryl-sms-vdp`; stage 4 (2026-10-19) the SG-1000 its TMS9918A and the Mark III as its second model; stage 5 (2026-10-19) mixes and
outputs Beryl's SN76489 (`src/sound.rs`); stage 6 (2026-10-19) adds the open boot program (`src/boot/`); stage 7
(2026-10-19) the Game Genie's patches, the battery table from SMS Power!'s tag and the kit's sample, `kit/sms-sample.txt`.
Its test programs are in `programs/`, assembled with Beryl's Z80 assembler.
The plan is `Man pages/Hardware/Sega/Endou - Master System/Endou_Plan.md`, the build record `Endou_Native.md` beside it.

Renamed from **Endymion**, which is now the audio output layer (`EmuSen.Endymion`) — see
`Man pages/EmuSen_Core_Naming_Scheme.md` §3 for the full core naming scheme, and §10 for
why the name moved.
