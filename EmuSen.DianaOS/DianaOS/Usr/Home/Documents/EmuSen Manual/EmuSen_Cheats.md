# EmuSen Cheats — the core-agnostic seam

How a typed cheat code becomes a running cheat, and what a core has to supply to take part. The mechanism itself — write widths, byte order, repeat runs, bit positions, the master switch — is `man cheat` and `CheatRegistry`'s own header; this document is only about the **seam between a core and the cheat engine**, which is what a second core exercised for the first time.

Per-console behaviour: `Hardware/Nintendo/Moon - NES/Moon_Cheats.md` for the NES, `Venus_Memory.md` §6 for the SNES, `Hardware/Nintendo/Mars - N64/Mars_Cheats.md` for the N64.

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

The cores install that hook in different places, and the difference is deliberate:

- **Venus** installs it from `SnesDebugTarget`, which owns the registry. ROM patches therefore need a debug target to exist.
- **Moon** installs it from `MoonCore` itself (`CheatRomPatcher`), because the core owns its own `CheatRegistry`. Cheats work with no debug layer attached at all.
- **Mercury** follows Moon, for the same reason and in the same line of `LoadRom`.
- **Mars** follows Moon too *(2026-09-18)*, with the hook in `MemoryBus.ReadCart32`, the one place a cartridge byte enters the machine — processor loads and the PI's transfers both reach it. A patch is asked for by ROM offset, and because an N64 game copies its code out of the cartridge, it takes effect at the next transfer rather than at once (`Mars_Cheats.md` §6).

Moon's arrangement is the better one and is where Venus should end up; it was not changed at the same time because moving Venus's registry ownership is a larger edit than adding a second core's. §6 is the bill for having two arrangements.

## 5. Not done

- **`.cht` import cannot produce ROM patches.** `ChtFile.ReadCode` builds `CheatWrite.Poke` unconditionally, so a database file full of Game Genie codes decodes to RAM pokes at ROM addresses, which do nothing. RetroArch's own model has no ROM-patch concept, which is where the shape came from. For the NES this matters more than it did for the SNES, because NES cheats are overwhelmingly published as Game Genie codes — the auto-detect slot is the raw `AAAA:VV` format precisely so a mis-import is a visible failure rather than a silent wrong cheat.
- **No Pro Action Rocky codec.** The NES equivalent of Action Replay is a real encrypted 8-hex-digit format (Mesen's `ConvertFromNesProActionRocky` has the key and shift table). There are only two codec slots and Game Genie earns the explicit one, so this needs the slots to become a list first.
- **The cheat device ROMs themselves.** A real Game Genie was a passthrough cartridge with its own ROM and code-entry screen. Booting one and handing off to the game is a separate feature — see `Moon_Cheats.md` §5.


## 6. Fixed bug: the registry a frontend handed over was not the one the core read

*2026-09-06, reported as "I was playing Super Mario Land 2 and none of the cheats I used worked".* Every cheat added in Mistress on a Game Boy or NES game went into an object the core never looked at. Both mechanisms were dead at once: ROM patches were never intercepted and RAM pokes were never applied, which is why the symptom was total rather than partial.

**The seam had two owners and only one wire.** Mistress owns a `CheatRegistry`, because its cheat windows exist before any ROM does and its saved cheat lists are restored per game. It hands that registry to `EmulatorSession`, which passes it to `CoreFactory.Load`. `Bundle` gave it to `SnesDebugTarget`'s constructor — and dropped it on the floor for `MercuryCore` and `MoonCore`, each of which constructs its own registry and applies *that* one, both at the ROM read intercept and at the frame boundary. `EmulatorSession.Cheats` even carries the comment "set by the frontend before LoadRom so the debug target shares its registry". It was true of exactly one core.

**Why the tests did not catch it, which is the more useful half.** Every piece was covered and every piece was correct. `MoonCheatTests` drives `_core.Cheats` directly and passes. The codecs have their own decode tests. `CheatRegistry` has its own. What nothing asserted was the **identity** of the registry across the seam — that the object handed in is the object read out — which is the one property a seam exists to provide. A test can cover both sides of a join and still not cover the join.

**The fix is `ICheatRegistryHost`**: a core that owns its registry exposes it settable, plus the `ApplyCheats()` it already ran every frame. `Bundle` assigns the caller's registry to any core implementing it, so a third core wired the same way inherits the behaviour instead of the bug. The setter re-installs `CheatRomPatcher` when a bus already exists, because the patcher captures the registry during `LoadRom` and `Load` bundles afterwards; without that the pokes would have started working and the patches would not, which is a worse failure than either. Venus is untouched — its registry lives in the debug target, which always took the parameter.

**A second, smaller hole in the same report.** `IDebugTarget.ApplyCheats` is a default interface method with an empty body. Venus overrode it; Mercury and Moon did not. Mistress's Apply button (`EmuSen_Settings_Reference.md` §4.15) calls it directly while paused, so even after the registries were shared, applying a cheat to a paused Game Boy game would have done nothing until the next unpause. Both targets now forward to the core method the frame boundary already called, so there is one implementation reached from two places rather than two implementations that can disagree.

**Coverage**: `EmuSen.WiseMan/Cores/CoreFactoryCheatWiringTests.cs`, seven tests, built to the report's shape rather than the fix's. Against the unfixed build six fail and only the SNES control passes — a per-core split that is what "none of my cheats worked, and it is not the codec" looks like from the outside. The six include both paused-apply tests, which cannot separate the two holes on the old build: an unshared registry fails them before the missing forward is reached. Removing only Mercury's forward from the fixed build is what separates them, and it reddens exactly the Game Boy paused-apply test. *(An earlier draft of this paragraph named only three of the six; re-measured 2026-09-15.)*

**What this does not reach, and how that step was closed anyway.** The emulation thread is not driven by a test; coverage stops at `EmulatorSession`, one call below `MainWindow`. That last call is confirmed by hand rather than by the harness: *2026-09-15, Super Mario Land 2 in Mistress from a source build, cheats applied and took effect* — the same report that opened §6, re-run against the fix. It is worth naming as a hand measurement, because it is the one step that will silently stop being covered if `MainWindow`'s cheat path is ever rewritten. §5's list is unchanged — `.cht` import still cannot produce ROM patches, and a database of Game Genie codes still imports as pokes at ROM addresses, which is a *different* way for a Game Boy cheat to do nothing and the next thing to look at if one still does.

## 7. Tests, and codes wider than a byte

*2026-09-18, with the N64's GameShark (`Mars_Cheats.md`).* Every format before it decoded one code to one byte, and the seam said so: `ICheatCodeCodec.Decode` returns an address and a byte. A GameShark code is lines — 16-bit writes, a line that tests memory and guards the next, a line that repeats the next — so two things were added, and nothing that existed changed meaning.

**A write can be a test.** `CheatWriteType` gained `IfEqual` and `IfNotEqual`: a write of that type compares its width in its byte order against its value (masked to its width, as a write's value is) and writes nothing. In `ApplyAll`, a failed test holds until the next write in the same cheat that is not a test, which it skips; so a run of tests is their AND, and one passing after one failing does not undo it. That is both N64 references' rule (`Mars_Cheats.md` §2.2). It is **not** RetroArch's: its handler form has the same two comparisons as `cheat_type` 4 and 5 ("run next if …"), but there a failure skips the next *entry*, whatever it is, so a second test after a failed first is skipped rather than evaluated, and what it guards runs. The names were kept apart from RetroArch's for that reason. `AddCheat` refuses a test that carries a repeat run or a bit position, and a ROM patch can hold no test (it already refused every type but `Set`). A test round-trips through the cheat file as its type's name, so a build older than this one skips such an entry rather than misreading it.

**A codec can decode a whole code.** `ICheatCodeCodec.DecodeWrites` is a default interface method returning null, like `DecodeCompare`, so no existing codec changed. When it answers, the three places a typed or imported code becomes a cheat add its writes as one cheat instead of asking `Decode` for a byte: `cheat add`, the Active Cheats window's Add button, and `ChtFile`'s code-string path — which hands such a codec the *whole* string before splitting on `+`, because a repeater spans a `+`. `cheat list` prints a test as `if <space> <address> == <value>`.

**`PrefersExplicitCodec` no longer picks an empty slot.** The N64 is the first console with one codec rather than two, and §2's tie-break sent a code that neither slot claimed to the punctuation guess — so a mistyped N64 code with a separator after its fourth digit (`8033-B21E`) went to the missing Game Genie slot and was answered "no Game Genie-style codec is registered", which is true and useless. With one slot empty the other is the answer, and its own error says what is wrong with the code. Consoles with both slots are unaffected; `A_malformed_n64_code_is_answered_by_the_gameshark` failed before the change and passes after it.

**`cheat export` skips a cheat containing a test**, and counts it beside the ROM patches it already skipped. The handler form holds one write per entry; a test written alone would lose what it guards, and written as `cheat_type` 4 it would round-trip into the defect below.

**A defect this surfaced and did not fix: RetroArch's comparison entries import as writes.** `ChtFile.ReadExplicit` maps `cheat_type` 1, 2 and 3 and sends everything else to `Set`, so an entry of type 4–7 ("run next if equal", "not equal", "less", "greater") imports as a write of its *compare value* to its address. Demonstrated on 2026-09-18 with a two-entry handler-form file — `cheat_type = 4` at address 16 with value 3, then a `Set` of 9 at 32 — which parsed to two `Set` writes, the first writing 3; the file came back with nothing skipped. It predates the test type and is untouched here, because fixing it well means turning RetroArch's cross-entry "next" into one cheat of several writes, which is an import feature rather than a repair. Until then such an entry does something neither RetroArch nor its author meant.

## The two combinations `AddCheat` refuses

Both throw rather than silently doing something adjacent, because there is no honest implementation of either:

- **A ROM patch cannot `Increase`/`Decrease`.** There is nothing to accumulate. The substitution happens at read time and the real byte is never written, so "add one each frame" has no meaning — the same byte would be read, incremented, and thrown away every time.
- **A compare byte only works on a single-byte patch.** `TryPatchRom` is handed one byte at a time by the cartridge read hook, so it cannot evaluate a compare spanning several addresses. Allowing it would produce a **torn patch**: the first byte declines to apply while the rest apply anyway, which is worse than either outcome on its own.

`ApplyAll` takes a read delegate as well as a write one for the mirror-image reason — `Increase`/`Decrease` and bit-position writes have to see what is already there — while the registry itself still never touches memory. The write-only overload is kept so existing callers compile; those write kinds read back as zero there.
