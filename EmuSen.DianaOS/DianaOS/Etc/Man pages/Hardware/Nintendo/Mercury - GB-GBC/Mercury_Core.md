# Mercury (Game Boy) — the core

*Started 2026-08-04. CPU, memory and cartridge only; there is no PPU and no APU yet — see §5 for what that means in practice.*

---

## 1. One core, not two

`Mercury` covers the Game Boy **and** the Game Boy Color. The naming scheme left this open ("whether GB and GBC are different enough hardware to warrant two separate cores… revisit once there's a real basis for comparison"), and the basis now exists: the CGB is the same SM83 at the same instruction set with the same memory map, plus double-speed mode, a second VRAM bank, seven WRAM banks, colour palettes and HDMA. That is an *extension*, the way an NES mapper extends a cartridge — not different hardware the way NES and SNES are. Splitting would duplicate almost everything and force every later fix in two places.

The build order is DMG first, colour as an additive mode on this same core. Nothing in the code branches on model yet; `Cartridge.Cgb` already reads the `$0143` flag (`Mercury_Memory.md` §2.1) so the seam is there when colour lands.

## 2. Frame timing

A frame is **70224 T-cycles**: 154 scanlines of 456. At the 4194304 Hz CPU clock that is 59.7275 Hz, which is what `FrameRateHz` reports rather than a rounded 60.

`RunFrame` steps the CPU until that budget is spent, carrying the overshoot into the next frame rather than discarding it — an instruction that straddles the boundary must not have its remainder lost, or the clock drifts.

Timing is **instruction-granular**, not cycle-granular: `Cpu.Step` returns the whole instruction's T-cycles and the bus is ticked by that amount afterwards, rather than the bus advancing between each of an instruction's individual memory accesses. The timer still sees every intermediate cycle because `MemoryBus.Tick` loops one at a time internally (`Mercury_Memory.md` §5), so DIV and TIMA are correct; what is not modelled is a mid-instruction write landing at exactly the right cycle relative to a PPU mode change. This is the same class of simplification as Moon's scanline-granularity PPU, and it is the thing to revisit first if a game turns out to depend on sub-instruction timing.

## 3. What raises VBlank today

With no PPU there is no LY, no STAT and no real vblank period, so `EndFrame` requests the VBlank interrupt on the frame boundary. That is enough for a game's main loop to tick over — most Game Boy games are structured as "wait for vblank, then do everything" — but it is a stand-in, not the hardware. It goes away when the PPU lands and starts driving the interrupt from LY 144.

## 4. Audio

`AudioSampleRate` answers 44100 and `DequeueAudioSamples` returns nothing. The rate has to be known before any samples exist for a caller opening a real device, which is why it is a property rather than something bundled into the drain call.

## 5. What is not built

- **No PPU.** `GetFrameBufferRgba` returns a correctly-sized black buffer. Nothing writes pixels.
- **No APU.** The four channels do not exist.
- **No `IDebugTarget`.** This is why Mercury is deliberately *not* registered in `CoreCatalog` or `CoreFactory` yet: `CoreFactory.Load` builds a `CoreBundle` that requires a debug target, so registering the core before one exists would put a `NotSupportedException` behind a ROM the catalog claims to support. The core is reachable from tests and nowhere else until that is written.
- **No serial, no STOP/double-speed.** `STOP` consumes its second byte and otherwise does nothing.

The pieces that *are* built are real: the full unprefixed and `$CB` instruction sets, interrupts with the EI delay and the HALT bug, the DIV/TIMA timer with falling-edge detection, the joypad matrix, OAM DMA, five cartridge boards, and save states.
