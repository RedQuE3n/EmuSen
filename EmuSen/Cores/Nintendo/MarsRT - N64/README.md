# MarsRT (Nintendo 64)

Native Rust Nintendo 64 emulation core (`marsrt`, `cdylib` + `rlib`), shipping on Core ABI v1.

- **Hardware covered:** NEC VR4300 CPU (COP0, 32-entry TLB, soft-float FPU, and Cranelift block recompiler on `emusen-jit`), MI/PI/SI/SP/DP/AI/RI interfaces, PIF/CIC boot handoff, EEPROM/SRAM/FlashRAM/Controller Pak saves, RSP scalar + 128-bit vector unit, software and multi-resolution Vulkan compute RDP (`emusen-gpu`), and Video Interface filtering/timing.
- **Hardware manuals:** [`Mars_CpuAndMemory.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_CpuAndMemory.md), [`Mars_Rsp.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_Rsp.md), [`Mars_RdpAndVi.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Mars%20-%20N64/Mars_RdpAndVi.md).
- **Naming:** Inner Senshi codename — see [`Overview_and_Architecture.md#core-and-component-naming-scheme`](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).
