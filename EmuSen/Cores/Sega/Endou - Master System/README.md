# Endou (Sega Master System)

The Sega Master System core, crate `endou`, in Rust on the core ABI v1, with the SG-1000's cartridges as a second
system, marked in development. Stage 2 (2026-10-19) runs the machine without its picture: Beryl's Z80 on the bus, the
cartridge boards, the I/O chip, the Pause and Reset buttons, a frame at a time; the VDP arrives at stage 3. Its test
programs are in `programs/`, assembled with Beryl's Z80 assembler.
The plan is `Man pages/Hardware/Sega/Endou - Master System/Endou_Plan.md`, the build record `Endou_Native.md` beside it.

Renamed from **Endymion**, which is now the audio output layer (`EmuSen.Endymion`) — see
`Man pages/EmuSen_Core_Naming_Scheme.md` §3 for the full core naming scheme, and §10 for
why the name moved.
