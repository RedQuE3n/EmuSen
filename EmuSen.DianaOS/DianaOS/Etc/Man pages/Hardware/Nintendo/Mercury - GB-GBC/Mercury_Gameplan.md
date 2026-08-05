# Mercury — where this stands, and what comes next

*Pinned 2026-08-04, at the end of the first session on this core. This is the "what should I work on next" doc for Mercury, the same role `EmuSen_Core_Gameplan.md` plays for Venus. It is a living document: when a phase lands, move it into §1 and delete it from §3.*

---

## 1. What is built and verified

All of it is covered by `EmuSen.WiseMan` — 41 tests across `MercuryCpuTests`, `MercuryCartridgeTests` and `MercuryCoreTests`, run headless against `SyntheticGbRom`. No real cartridge is needed or committed.

| Piece | State |
|---|---|
| **SM83 instruction set** | Complete. Unprefixed and `$CB`, including the illegal-opcode set (which throws rather than acting as NOP). |
| **Flags** | All four, with the rules that differ from the 8080/Z80 people assume — `RLCA` clearing Z where `CB RLC` sets it, `INC` preserving carry, `BIT` leaving carry alone, `ADD SP,e8` carrying off the unsigned low byte, `DAA`. |
| **Interrupts** | Five sources, priority order, vectoring, the `EI` one-instruction delay, `RETI`'s immediate enable, HALT waking regardless of IME, and the HALT bug. |
| **Timer** | DIV as the top half of the real 16-bit counter; TIMA as a falling-edge detector on a selected bit, with the 4-cycle reload window. |
| **Joypad** | The two-nibble matrix, pressed-reads-0, both-halves-selected ANDing. |
| **OAM DMA** | Copies, immediately rather than over 160 cycles (§4). |
| **Cartridge** | Header, title (both lengths), colour flag, checksum, ROM/RAM sizing. |
| **Boards** | No-MBC, MBC1 (both modes), MBC2, MBC3 with a latched RTC, MBC5. |
| **Core** | `ICore` surface, frame budget, breakpoints, coverage, cheats, named debug spaces, save states. |

## 2. Decisions already made — do not relitigate these

- **One core for GB and GBC.** DMG first, colour additive. Reasoning in `Mercury_Core.md` §1. `Cartridge.Cgb` already reads `$0143`.
- **Not registered in `CoreCatalog`/`CoreFactory`.** Deliberate, and it stays that way until Phase B lands — `CoreFactory.Load` builds a `CoreBundle` that requires an `IDebugTarget`, so registering first would put a `NotSupportedException` behind a `.gb` file the catalog claims to handle.
- **Illegal opcodes throw.** A game reaching one means something upstream already went wrong; treating them as NOPs hides the real bug.
- **`.gbc` is not claimed** as an extension yet. It arrives with colour, not before — running a CGB-only cart in DMG mode would fail confusingly rather than cleanly.

## 3. Phases, in the order they should be done

### Phase A — the PPU

The largest remaining piece and the one that makes this visibly a Game Boy. Nothing draws pixels today.

Wants: LCDC/STAT/SCY/SCX/LY/LYC/WY/WX/BGP/OBP0/OBP1, the four PPU modes with their real scanline timing, background and window rendering, sprite rendering with the 10-per-line limit and the DMG x-coordinate priority rule, the STAT interrupt on all four sources, and LY=LYC.

Start here because it also removes a stand-in: `EndFrame` currently raises VBlank on the frame boundary because there is no LY to drive it from (`Mercury_Core.md` §3). Once the PPU exists, the interrupt comes from LY reaching 144 like the hardware.

Scanline granularity is the right first target, matching Moon. A pixel FIFO is only needed if a game turns out to depend on mid-scanline register writes.

### Phase B — `MercuryDebugTarget`

Unblocks factory registration and gets the core into both frontends and DianaOS. Cheaper than it looks now that `ICoreTelemetry` exists (`EmuSen_Cauldron.md` §3) — the read surface is a well-defined list, and `MercuryCore.Spaces.cs` already supplies `ReadSpace`/`WriteSpace`/`SpaceSize` for the memory-space half.

Needs an SM83 disassembler, which does not exist yet. Everything else is either already available on the core or can return a documented empty.

When this lands, register in `CoreCatalog` (`.gb`, `"Nintendo - Game Boy"` cheat system, `MercuryCore.PadButtons`) and add the `MoonCore`-shaped arm to `CoreFactory.Create`/`Bundle`.

### Phase C — the APU

Four channels: two pulse with sweep and envelope, a programmable wave channel, and noise with an LFSR. `AudioSampleRate` already answers 44100 and `DequeueAudioSamples` already returns empty, so the seam is in place.

### Phase D — Game Boy Color

Double-speed mode, VRAM bank 2, WRAM banks 1-7, BG/OBJ colour palettes with their auto-increment index registers, HDMA/GDBA, and the CGB sprite priority rule. This is where `Cartridge.Cgb` finally gets read for something, `.gbc` joins the catalog, and `CoreName` stops being unconditionally `"GB"`.

## 4. Known simplifications, and when each will matter

- **Instruction-granular timing.** `Step` returns the whole instruction's T-cycles and the bus is ticked afterwards. The timer is unaffected (`MemoryBus.Tick` loops one cycle at a time internally), but a mid-instruction write cannot land at an exact cycle relative to a PPU mode change. Revisit only if a real game proves it needs sub-instruction timing — this is the same call Moon made.
- **OAM DMA copies immediately** rather than over 160 machine cycles with the bus locked. Nothing can observe the difference until the PPU exists.
- **No boot ROM.** Mercury starts at `$0100` with the post-boot register state hardcoded, which is also why the header checksum is computed but never enforced.
- **`STOP` consumes its second byte and does nothing.** It only matters for CGB double-speed switching, which is Phase D.
- **No serial link.** Nothing needs it yet.

## 5. Resuming

```
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~Mercury"
```

Read `Mercury_Cpu.md` before touching the CPU — §1 lists the ways the SM83 is not the chip people assume it is, and most of them are one-line mistakes that pass casual review.
