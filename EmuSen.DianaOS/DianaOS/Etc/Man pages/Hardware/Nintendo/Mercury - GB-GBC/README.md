# Mercury (Game Boy / Game Boy Color)

- **`Mercury_Gameplan.md`** — start here: what is built and verified, the decisions already settled, and the phased plan for what comes next.
- **`Mercury_Core.md`** — why one core covers both consoles, frame timing, and an explicit list of what is not built yet.
- **`Mercury_Cpu.md`** — the SM83: how it differs from the 8080 and Z80 people assume it is, interrupts, the HALT bug, and the instruction tables.
- **`Mercury_Ppu.md`** — the LCD controller: a per-cycle clock under a per-line renderer, the mode machine, STAT as one level rather than four events, the window's own line counter, the two sprite orderings, and access blocking — including the argument for its absence that stood until the bus became cycle-granular.
- **`Mercury_Cgb.md`** — the colour extensions on the same core: when colour mode is entered and why the `A` register decides a game's rendering path, the palette ports, the three things the CGB changes about rendering, HDMA's two modes, and what double speed does not double.
- **`Mercury_Apu.md`** — the four sound channels: the frame sequencer as a DIV bit rather than a divider, the envelope period that means off, the sweep's two overflow checks, and why a live DAC at digital zero is not silence.
- **`Mercury_Debug.md`** — `MercuryDebugTarget`/`IDebugTarget`: what registration was waiting on, the memory spaces, the telemetry that carries something other than the obvious, and the SM83 disassembler's template table.
- **`Mercury_Cheats.md`** — Game Genie and GameShark: the two mechanisms, the Game Genie cipher and its three easy mistakes, and why there is no plain address:value form.
- **`Mercury_RealCartridges.md`** — what running seven commercial images found: the `--nobattery` switch that was lying, a screen-and-WRAM diff against gambatte, why Super Mario Land is silent and why that is not the emulator, and the header scan showing the Link's Awakening DX proto is not colour coverage.
- **`Mercury_Memory.md`** — the header, the memory map, the five cartridge boards, the timer's edge detector, OAM DMA as a 160-cycle transfer, the joypad matrix, and the serial port as a sink rather than a link.
- **`Mercury_HardwareTests.md`** — where Mercury's evidence comes from: why the Game Boy's own test corpus is a better oracle than another emulator, how blargg's and mooneye's verdicts are read off the link port, and the first audio differential — a 9 dB level convention, aligned note onsets, and the boot-offset explanation ruled out.
