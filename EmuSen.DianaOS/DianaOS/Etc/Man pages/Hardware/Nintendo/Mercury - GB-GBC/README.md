# Mercury (Game Boy / Game Boy Color)

- **`Mercury_Gameplan.md`** — start here: what is built and verified, the decisions already settled, and the phased plan for what comes next.
- **`Mercury_Core.md`** — why one core covers both consoles, frame timing, and an explicit list of what is not built yet.
- **`Mercury_Cpu.md`** — the SM83: how it differs from the 8080 and Z80 people assume it is, interrupts, the HALT bug, and the instruction tables.
- **`Mercury_Ppu.md`** — the LCD controller: a per-cycle clock under a per-line renderer, the mode machine, STAT as one level rather than four events, the window's own line counter, the two sprite orderings, and why VRAM access blocking is deliberately absent.
- **`Mercury_Cgb.md`** — the colour extensions on the same core: when colour mode is entered and why the `A` register decides a game's rendering path, the palette ports, the three things the CGB changes about rendering, HDMA's two modes, and what double speed does not double.
- **`Mercury_Memory.md`** — the header, the memory map, the five cartridge boards, the timer's edge detector, OAM DMA and the joypad matrix.
