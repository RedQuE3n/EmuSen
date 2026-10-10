# Moon (NES) — Cheats

*This revision: the NES decodes Game Genie and raw codes, and patches reach the CPU.*

Covers `Cores/Nintendo/Moon - NES/Cheats/` and the ROM-patch hook in `Memory/MemoryBus.cs`.

The core-agnostic half — the codec slots, the compare byte, where each core installs its hook — is `EmuSen_Cheats.md`. This document is the NES-specific part.

---

## 1. What works

| Format | Example | Becomes |
| --- | --- | --- |
| Game Genie, 6 letters | `SXIOPO` | ROM patch, unconditional |
| Game Genie, 8 letters | `SLXPLOVS` | ROM patch, compare-gated |
| Raw address:value | `0075:09` | RAM poke into `CPUBUS` |

All three reach `cheat add`, `cheat gg`, and Mistress's Active Cheats window. `poke`, `rompatch`, `list`, `enable`/`disable` and the master switch were already core-agnostic and needed nothing.

## 2. The two decoders

### Game Genie

A genuine cipher, and a **different** one from the SNES device — different alphabet, different bit layout, and NES codes carry a compare byte where SNES codes do not.

The alphabet is `APZLGITYEOXUKSVN`; a letter's index is the nibble it decodes to. Each letter contributes its nibble at its own 4-bit slot (letter 0 is the *low* nibble), and the address and value are then gathered out of that 32-bit word from fixed, non-contiguous bit positions:

```
address bits: 14 13 12 19 22 21 20 7 10 9 8 15 18 17 16   (+ $8000)
value bits:    3  6  5  4 23  2  1 0
compare bits: 27 30 29 28 23 26 25 24                      (8-letter only)
```

For an 8-letter code, **bit 4 of the value moves from 23 to 31**, because the compare byte takes the position it occupied. Getting that one substitution wrong silently corrupts the value of every 8-letter code, which is the kind of bug that looks like a broken game rather than a broken decoder.

The address is a 15-bit offset plus `$8000`, which is why a Game Genie code can only ever reach the cartridge's half of the CPU map.

**These tables were ported, not derived.** They come from Mesen's `CheatManager::ConvertFromNesGameGenie`. `NesGameGenieCodecTests` pins 89 codes — a mix of published ones and randomly generated 6- and 8-letter codes — against output produced by transcribing Mesen's algorithm and running it as an oracle. Regenerating those expectations from this implementation would prove nothing, so the file says where they came from.

Spot check: `SXIOPO` decodes to `$91D9 = $AD`. On Super Mario Bros. the byte there really is `$CE` — `DEC absolute`, the instruction that decrements the life counter — and `$AD` is `LDA absolute`. That is the whole of "infinite lives": turn the decrement into a load.

### Raw address:value

Four hex digits of CPU address, two of value, separators ignored — the shape Mesen calls a custom code. No cipher. Six hex digits rather than the SNES form's eight is also what keeps the two from claiming each other's codes.

## 3. How a patch reaches the CPU

`MemoryBus.Read`'s cartridge branch — everything the mapper answers — consults `RomPatcher` with the **CPU address**, not a ROM offset:

```csharp
value = Cart.Mapper.ReadPrg(address);
if (RomPatcher is not null && RomPatcher.TryPatch(address, value, out byte patched)) value = patched;
```

The CPU address is the right key because that is what the real device saw: a Game Genie sat in the edge connector between console and cartridge, watching the address bus. It could not see WRAM or a hardware register, and neither can this hook — a ROM patch aimed below `$4020` is inert, and there is a test for that.

`MoonCore` installs the hook itself (`CheatRomPatcher`, wrapping the core's own `CheatRegistry`), so NES cheats work with no debug target attached. RAM pokes were already applied from `EndFrame` and needed no change.

## 4. Not done

- **Bus conflicts.** A real Game Genie ANDs its patched byte with what the cartridge drives when the board maps anything below `$8000`. Mesen models this (`NesConsole::ProcessCheatCode`) and offers a setting to disable it. Nothing here does, so a patch on a work-RAM board reads back cleaner than hardware would.
- **Pro Action Rocky**, the NES Action Replay. Encrypted 8-hex-digit codes; see `EmuSen_Cheats.md` §5 for why it has nowhere to plug in yet. You have two PAR ROM dumps locally but no decoder.
- **More than three codes at once** is not a limit here — the real device took three, this takes as many as you add. Worth knowing if you are comparing against hardware.

## 5. The Game Genie cartridge itself

Not built. The real device was a passthrough cartridge: its own small ROM and a handful of registers, with the game plugged into its back. It boots to a code-entry screen, and on Start it writes the codes into its registers, unmaps itself, and the game appears with the patches applied in hardware — the same `(address, value, compare)` triples §3's hook already applies.

Two things to know before starting it:

- **Mesen does not implement it**, so unlike everything else here there is no running reference to diff against. It would be built from NESdev documentation and verified against the real ROM's observed behaviour.
- The local `Game Genie (Unl).nes` dump is **suspect**: its header declares 16K PRG + 8K CHR, 24 KB in total, but the file is 280 KB. Resolve that before building against it.
