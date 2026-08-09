# Mercury — where this stands, and what comes next

*Pinned 2026-08-04, updated 2026-08-08 when Phases A, D and B landed. This is the "what should I work on next" doc for Mercury, the same role `EmuSen_Core_Gameplan.md` plays for Venus. It is a living document: when a phase lands, move it into §1 and delete it from §3.*

---

## 1. What is built and verified

All of it is covered by `EmuSen.WiseMan` — 123 tests across `MercuryCpuTests`, `MercuryCartridgeTests`, `MercuryCoreTests`, `MercuryPpuTests`, `MercuryCgbTests` and `MercuryDebugTargetTests`, run headless against `SyntheticGbRom`. No real cartridge is needed or committed.

| Piece | State |
|---|---|
| **SM83 instruction set** | Complete. Unprefixed and `$CB`, including the illegal-opcode set (which throws rather than acting as NOP). |
| **Flags** | All four, with the rules that differ from the 8080/Z80 people assume — `RLCA` clearing Z where `CB RLC` sets it, `INC` preserving carry, `BIT` leaving carry alone, `ADD SP,e8` carrying off the unsigned low byte, `DAA`. |
| **Interrupts** | Five sources, priority order, vectoring, the `EI` one-instruction delay, `RETI`'s immediate enable, HALT waking regardless of IME, and the HALT bug. |
| **Timer** | DIV as the top half of the real 16-bit counter; TIMA as a falling-edge detector on a selected bit, with the 4-cycle reload window. |
| **Joypad** | The two-nibble matrix, pressed-reads-0, both-halves-selected ANDing. |
| **OAM DMA** | Copies, immediately rather than over 160 cycles (§4). |
| **PPU** | The mode machine on a per-cycle clock, LY/LYC, STAT as one level-triggered line, background, window with its own line counter, sprites with both DMG orderings. Renderer is per scanline. `Mercury_Ppu.md`. |
| **Colour** | VRAM and WRAM banking, both palette ports, map and sprite attributes, CGB sprite priority, HDMA in both modes, double speed. `Mercury_Cgb.md`. |
| **Cartridge** | Header, title (both lengths), colour flag, checksum, ROM/RAM sizing. |
| **Boards** | No-MBC, MBC1 (both modes), MBC2, MBC3 with a latched RTC, MBC5. |
| **Core** | `ICore` surface, frame budget, breakpoints, coverage, cheats, named debug spaces, save states. |
| **Debug** | `MercuryDebugTarget`, the SM83 disassembler, Game Genie and GameShark, and registration in `CoreCatalog`/`CoreFactory`. `Mercury_Debug.md`. |

## 2. Decisions already made — do not relitigate these

- **One core for GB and GBC.** DMG first, colour additive. Reasoning in `Mercury_Core.md` §1. `Cartridge.Cgb` already reads `$0143`.
- **Registered in `CoreCatalog`/`CoreFactory` as of Phase B**, claiming `.gb` and `.gbc`. The condition this waited on was real and is now met: `CoreFactory.Load` builds a `CoreBundle` whose `IDebugTarget` is non-nullable, and one now exists.
- **Illegal opcodes throw.** A game reaching one means something upstream already went wrong; treating them as NOPs hides the real bug.
- **`.gb` and `.gbc` both reach the same core**, and which console it becomes is the header's answer rather than the extension's — a `.gb` file with `$0143 = $C0` runs in colour. The prediction that the extension would arrive with Phase B rather than Phase D held.
- **Colour mode is decided by the header, once, at load.** Both `$80` and `$C0` select it, matching the real console. No runtime toggle — `Mercury_Cgb.md` §1.

## 3. Phases, in the order they should be done

### Phase C — the APU

The last phase, and now the only one. Four channels: two pulse with sweep and envelope, a programmable wave channel, and noise with an LFSR. `AudioSampleRate` already answers 44100 and `DequeueAudioSamples` already returns empty, so the seam is in place.

Three places already hold a documented empty waiting for it, and each should be filled in the same change: `MercuryDebugTarget`'s `AudioChannels` and `SetChannelMuted`, and its `ApuRegisters`, which currently carries the timer and the cartridge board because they had nowhere better to live (`Mercury_Debug.md` §3).

## 4. Known simplifications, and when each will matter

- **Instruction-granular timing.** `Step` returns the whole instruction's T-cycles and the bus is ticked afterwards. The timer is unaffected (`MemoryBus.Tick` loops one cycle at a time internally), but a mid-instruction write cannot land at an exact cycle relative to a PPU mode change. Revisit only if a real game proves it needs sub-instruction timing — this is the same call Moon made.
- **OAM DMA copies immediately** rather than over 160 machine cycles with the bus locked. The PPU now exists, so this is observable in principle — a game reading OAM during the transfer sees the finished copy — but not in practice while access blocking is absent (`Mercury_Ppu.md` §7). Both want the same cycle-granular bus, and should be done together.
- **No VRAM or OAM access blocking**, and this is a deliberate consequence of the instruction-granular bus rather than a shortcut. `Mercury_Ppu.md` §7 states the trade and the condition that reverses it.
- **The renderer is per scanline** under a per-cycle clock. `Mercury_Ppu.md` §1.
- **No boot ROM.** Mercury starts at `$0100` with the post-boot register state hardcoded, which is also why the header checksum is computed but never enforced.
- **`STOP` performs the speed switch on a CGB** and still does nothing on a DMG. The switch is instantaneous rather than the ~2050 cycles hardware spends on it — `Mercury_Cgb.md` §5.
- **No serial link.** Nothing needs it yet.
- **No call stack, expression context, access counters or freezes** on the debug target. Each is a seam the core does not have; the interface allows null for all of them - `Mercury_Debug.md` §6.

## 5. Resuming

```
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~Mercury"
```

Read `Mercury_Cpu.md` before touching the CPU — §1 lists the ways the SM83 is not the chip people assume it is, and most of them are one-line mistakes that pass casual review.
