# Mercury (Game Boy) — the core

*Started 2026-08-04, feature-complete 2026-08-08: CPU, memory, cartridge, the LCD
controller, the Game Boy Color extensions, the debug target and the sound
hardware. §5 is now a list of gaps rather than a list of absences.*

---

## 1. One core, not two

`Mercury` covers the Game Boy **and** the Game Boy Color. The naming scheme left this open ("whether GB and GBC are different enough hardware to warrant two separate cores… revisit once there's a real basis for comparison"), and the basis now exists: the CGB is the same SM83 at the same instruction set with the same memory map, plus double-speed mode, a second VRAM bank, seven WRAM banks, colour palettes and HDMA. That is an *extension*, the way an NES mapper extends a cartridge — not different hardware the way NES and SNES are. Splitting would duplicate almost everything and force every later fix in two places.

The build order was DMG first, colour as an additive mode on this same core, and as of 2026-08-08 both halves exist. The prediction held: colour cost one extra file each in the bus and the PPU plus a handful of gated branches in the renderer, against the two full copies of everything a split would have required. `Mercury_Cgb.md` is the whole of the difference.

## 2. Frame timing

A frame is **70224 T-cycles**: 154 scanlines of 456. At the 4194304 Hz CPU clock that is 59.7275 Hz, which is what `FrameRateHz` reports rather than a rounded 60.

`RunFrame` steps the CPU until the PPU completes a frame, or until that budget is spent if the LCD is off and the PPU is therefore frozen (§3). The budget carries its overshoot into the next frame rather than discarding it — an instruction that straddles the boundary must not have its remainder lost, or the clock drifts. When the PPU ends the frame the carry is dropped instead, because the PPU's own dot counter is then holding the phase.

Timing is **instruction-granular**, not cycle-granular: `Cpu.Step` returns the whole instruction's T-cycles and the bus is ticked by that amount afterwards, rather than the bus advancing between each of an instruction's individual memory accesses. The timer still sees every intermediate cycle because `MemoryBus.Tick` loops one at a time internally (`Mercury_Memory.md` §5), so DIV and TIMA are correct; what is not modelled is a mid-instruction write landing at exactly the right cycle relative to a PPU mode change. This is the same class of simplification as Moon's scanline-granularity PPU, and it is the thing to revisit first if a game turns out to depend on sub-instruction timing. It is also why Mercury does not enforce VRAM access blocking — see `Mercury_Ppu.md` §7, which is the clearest statement of what this simplification actually costs.

## 3. What raises VBlank

The PPU does, from LY reaching 144, like the hardware.

This section used to describe a stand-in: with no LY to drive it from, `EndFrame` requested VBlank on the frame boundary so a game's main loop would tick over. That is gone as of 2026-08-08. The frame boundary itself has moved with it — `RunFrame` now returns when the PPU wraps LY to 0, and the 70224-cycle budget survives only as a watchdog for a disabled LCD, which does not advance at all. See `Mercury_Ppu.md` §1.1 for why the budget could not stay the authority.

## 4. Audio

`AudioSampleRate` answers 44100 and is told to the APU at load rather than read back from it. The rate has to be known before any samples exist for a caller opening a real device, which is why it is a property rather than something bundled into the drain call.

`DequeueAudioSamples` drains the APU's own queue. The output is genuinely stereo, unlike Moon's — see `Mercury_Apu.md` §1.

## 5. What is not built

- **No serial.** Nothing needs it.
- **No boot ROM,** on either console, which is why the post-boot register file is hardcoded and why DMG-on-CGB colourisation does not exist (`Mercury_Cgb.md` §1.1 and §6).

Every subsystem states its own gaps: `Mercury_Ppu.md` §2.2, §3.2 and §7 (of which the absent VRAM access blocking is the one with real consequences), `Mercury_Cgb.md` §6, `Mercury_Apu.md` §7 and `Mercury_Debug.md` §6.

The pieces that *are* built are real: the full unprefixed and `$CB` instruction sets, interrupts with the EI delay and the HALT bug, the DIV/TIMA timer with falling-edge detection, the joypad matrix, OAM DMA, five cartridge boards, background/window/sprite rendering, the colour palettes, VRAM and WRAM banking, HDMA, double speed, all four sound channels, a debug target, and save states.
