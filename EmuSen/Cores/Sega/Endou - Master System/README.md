# Endou (Sega Master System)

The Sega Master System core, crate `endou`, in Rust on the core ABI v1, with the SG-1000's cartridges as a second
system. Stage 1 (2026-10-19) is a stub marked in development: it reads a cartridge's header and shows a black picture.
The plan is `Man pages/Hardware/Sega/Endou - Master System/Endou_Plan.md`, the build record `Endou_Native.md` beside it.

Renamed from **Endymion**, which is now the audio output layer (`EmuSen.Endymion`) — see
`Man pages/EmuSen_Core_Naming_Scheme.md` §3 for the full core naming scheme, and §10 for
why the name moved.
