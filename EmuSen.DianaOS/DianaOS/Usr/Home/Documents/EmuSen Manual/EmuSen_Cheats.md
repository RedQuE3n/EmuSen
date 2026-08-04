# EmuSen Cheats — the core-agnostic seam

How a typed cheat code becomes a running cheat, and what a core has to supply to take part. The mechanism itself — write widths, byte order, repeat runs, bit positions, the master switch — is `man cheat` and `CheatRegistry`'s own header; this document is only about the **seam between a core and the cheat engine**, which is what a second core exercised for the first time.

Per-console behaviour: `Hardware/Nintendo/Moon - NES/Moon_Cheats.md` for the NES, `Venus_Memory.md` §6 for the SNES.

---

## 1. What a core supplies

Three things, all optional — a core that supplies none of them still gets `poke`/`rompatch`/`list`/`enable`/`disable` for free, because those only ever touch `CheatRegistry`.

| Piece | Type | What it buys |
| --- | --- | --- |
| Auto-detect codec | `ICheatCodeCodec` | `cheat add`'s fallback, and `.cht` import |
| Explicit codec | `ICheatCodeCodec` | `cheat gg`, and `add`'s other guess |
| A `CheatRegistry` reachable from the bus | — | ROM patches actually applying |

`CoreFactory.Bundle` hands the two codecs to whoever asks; `CoreFactory.CheatCodecsFor(console)` answers the same question for a console picked in the UI with no ROM loaded, which is what lets Mistress's cheat windows work before you launch anything.

## 2. Two codec slots, and how `add` chooses between them

The slots are named by **role**, not by format: one produces RAM pokes, the other ROM patches. `cheat gg` always uses the explicit slot. `cheat add` has to guess.

It used to guess from punctuation alone: a separator after the 4th character (`XXXX-XXXX`) meant Game Genie, anything else meant Pro Action Replay. That works for the SNES, where both formats draw from the same sixteen hex characters and genuinely cannot be told apart by content. It does not work for the NES, whose Game Genie alphabet is letters (`SXIOPO`) that the hex format cannot contain — the punctuation rule sent every NES code to the wrong decoder.

`CheatCommand.PrefersExplicitCodec` asks the codecs first: **where exactly one of them claims the code, that is an answer rather than a guess**, and only a tie falls through to the punctuation convention. SNES behaviour is unchanged, because there every code ties.

It is `public static` because Mistress's Active Cheats window makes the identical choice, and two copies of a guess drift apart. The window used to carry its own call to the punctuation helper, which is exactly how it ended up unable to add a NES code.

## 3. The compare byte

`ICheatCodeCodec.Decode` returns an address and a value. Some formats also carry a **compare**: apply this patch only where the byte already there matches. `CheatRegistry` has supported it since the beginning (`Cheat.Compare`, checked in `TryPatchRom`), but no codec could express it, so every decoded code became unconditional.

NES Game Genie's 8-letter codes are the reason it now can. On a bank-switched cartridge one CPU address is many different ROM bytes over time, and the compare is what picks which one the code means. Dropping it does not produce a lesser cheat, it produces a **wrong** one — the patch fires in every bank.

`byte? DecodeCompare(string code) => null;` is a default interface implementation, so a format without one (SNES Game Genie, Pro Action Replay) says nothing and needed no edit.

## 4. Where patches are applied

RAM pokes are re-applied at every frame boundary — `CheatRegistry.ApplyAll` with the core's own read/write delegates — because a game that writes the address every frame would otherwise win.

ROM patches are a **read intercept**, not a write: `IRomReadPatcher.TryPatch` is consulted on every cartridge-routed read and the underlying ROM byte is never touched. That is what the hardware did, and it is why disabling a patch restores the original byte with no bookkeeping.

The two cores install that hook in different places, and the difference is deliberate:

- **Venus** installs it from `SnesDebugTarget`, which owns the registry. ROM patches therefore need a debug target to exist.
- **Moon** installs it from `MoonCore` itself (`CheatRomPatcher`), because the core owns its own `CheatRegistry`. Cheats work with no debug layer attached at all.

Moon's arrangement is the better one and is where Venus should end up; it was not changed at the same time because moving Venus's registry ownership is a larger edit than adding a second core's.

## 5. Not done

- **`.cht` import cannot produce ROM patches.** `ChtFile.ReadCode` builds `CheatWrite.Poke` unconditionally, so a database file full of Game Genie codes decodes to RAM pokes at ROM addresses, which do nothing. RetroArch's own model has no ROM-patch concept, which is where the shape came from. For the NES this matters more than it did for the SNES, because NES cheats are overwhelmingly published as Game Genie codes — the auto-detect slot is the raw `AAAA:VV` format precisely so a mis-import is a visible failure rather than a silent wrong cheat.
- **No Pro Action Rocky codec.** The NES equivalent of Action Replay is a real encrypted 8-hex-digit format (Mesen's `ConvertFromNesProActionRocky` has the key and shift table). There are only two codec slots and Game Genie earns the explicit one, so this needs the slots to become a list first.
- **The cheat device ROMs themselves.** A real Game Genie was a passthrough cartridge with its own ROM and code-entry screen. Booting one and handing off to the game is a separate feature — see `Moon_Cheats.md` §5.
