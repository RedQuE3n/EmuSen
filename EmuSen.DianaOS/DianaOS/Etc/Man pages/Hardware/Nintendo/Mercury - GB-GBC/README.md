# Mercury (Game Boy / Game Boy Color)

- **`Mercury_Gameplan.md`** — start here: what is built and verified, the decisions already settled, and the phased plan for what comes next.
- **`Mercury_Core.md`** — why one core covers both consoles, frame timing, and an explicit list of what is not built yet.
- **`Mercury_Cpu.md`** — the SM83: how it differs from the 8080 and Z80 people assume it is, interrupts, the HALT bug, and the instruction tables.
- **`Mercury_Ppu.md`** — the LCD controller: a per-cycle clock under a per-line renderer, the mode machine, STAT as one level rather than four events, the window's own line counter, the two sprite orderings, and why VRAM access blocking is deliberately absent.
- **`Mercury_Cgb.md`** — the colour extensions on the same core: when colour mode is entered and why the `A` register decides a game's rendering path, the palette ports, the three things the CGB changes about rendering, HDMA's two modes, and what double speed does not double.
- **`Mercury_Apu.md`** — the four sound channels: the frame sequencer as a DIV bit rather than a divider, the envelope period that means off, the sweep's two overflow checks, and why a live DAC at digital zero is not silence.
- **`Mercury_Debug.md`** — `MercuryDebugTarget`/`IDebugTarget`: what registration was waiting on, the memory spaces, the telemetry that carries something other than the obvious, and the SM83 disassembler's template table.
- **`Mercury_Cheats.md`** — Game Genie and GameShark: the two mechanisms, the Game Genie cipher and its three easy mistakes, and why there is no plain address:value form.
- **`Mercury_Memory.md`** — the header, the memory map, the five cartridge boards, the timer's edge detector, OAM DMA and the joypad matrix.
