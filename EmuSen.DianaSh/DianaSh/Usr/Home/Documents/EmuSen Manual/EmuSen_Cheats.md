# EmuSen Cheats — the core-agnostic seam

How a typed cheat code becomes a running cheat, and what a core has to supply to take part. The mechanism itself — write widths, byte order, repeat runs, bit positions, the master switch — is `man cheat` and `CheatRegistry`'s own header; this document is only about the **seam between a core and the cheat engine**, which is what a second core exercised for the first time.

Per-console behaviour: `Hardware/Nintendo/Moon - NES/Moon_Cheats.md` for the NES, `Venus_Memory.md` §6 for the SNES, `Hardware/Nintendo/Mars - N64/Mars_Cheats.md` for the N64, §9 below for the Genesis.

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

- ~~**`.cht` import cannot produce ROM patches.**~~ It can when it is given the explicit codec, since 2026-10-06 (§8). `ChtFile.ReadCode` builds `CheatWrite.Poke` unconditionally, so a database file full of Game Genie codes decodes to RAM pokes at ROM addresses, which do nothing. RetroArch's own model has no ROM-patch concept, which is where the shape came from. For the NES this matters more than it did for the SNES, because NES cheats are overwhelmingly published as Game Genie codes — the auto-detect slot is the raw `AAAA:VV` format precisely so a mis-import is a visible failure rather than a silent wrong cheat.
- **No Pro Action Rocky codec.** The NES equivalent of Action Replay is a real encrypted 8-hex-digit format (Mesen's `ConvertFromNesProActionRocky` has the key and shift table). There are only two codec slots and Game Genie earns the explicit one, so this needs the slots to become a list first.
- **The cheat device ROMs themselves.** A real Game Genie was a passthrough cartridge with its own ROM and code-entry screen. Booting one and handing off to the game is a separate feature — see `Moon_Cheats.md` §5.


## 6. Fixed bug: the registry a frontend handed over was not the one the core read

*2026-09-06, reported as "I was playing Super Mario Land 2 and none of the cheats I used worked".* Every cheat added in Mistress on a Game Boy or NES game went into an object the core never looked at. Both mechanisms were dead at once: ROM patches were never intercepted and RAM pokes were never applied, which is why the symptom was total rather than partial.

**The seam had two owners and only one wire.** Mistress owns a `CheatRegistry`, because its cheat windows exist before any ROM does and its saved cheat lists are restored per game. It hands that registry to `EmulatorSession`, which passes it to `CoreFactory.Load`. `Bundle` gave it to `SnesDebugTarget`'s constructor — and dropped it on the floor for `MercuryCore` and `MoonCore`, each of which constructs its own registry and applies *that* one, both at the ROM read intercept and at the frame boundary. `EmulatorSession.Cheats` even carries the comment "set by the frontend before LoadRom so the debug target shares its registry". It was true of exactly one core.

**Why the tests did not catch it, which is the more useful half.** Every piece was covered and every piece was correct. `MoonCheatTests` drives `_core.Cheats` directly and passes. The codecs have their own decode tests. `CheatRegistry` has its own. What nothing asserted was the **identity** of the registry across the seam — that the object handed in is the object read out — which is the one property a seam exists to provide. A test can cover both sides of a join and still not cover the join.

**The fix is `ICheatRegistryHost`**: a core that owns its registry exposes it settable, plus the `ApplyCheats()` it already ran every frame. `Bundle` assigns the caller's registry to any core implementing it, so a third core wired the same way inherits the behaviour instead of the bug. The setter re-installs `CheatRomPatcher` when a bus already exists, because the patcher captures the registry during `LoadRom` and `Load` bundles afterwards; without that the pokes would have started working and the patches would not, which is a worse failure than either. Venus is untouched — its registry lives in the debug target, which always took the parameter.

**A second, smaller hole in the same report.** `IDebugTarget.ApplyCheats` is a default interface method with an empty body. Venus overrode it; Mercury and Moon did not. Mistress's Apply button (`EmuSen_Settings_Reference.md` §4.15) calls it directly while paused, so even after the registries were shared, applying a cheat to a paused Game Boy game would have done nothing until the next unpause. Both targets now forward to the core method the frame boundary already called, so there is one implementation reached from two places rather than two implementations that can disagree. *The generic v1 target, `CoreDebugTarget`, met the same hole again and was closed on 2026-10-05, together with a core that gates its own pokes (`VenusRT_Native.md` §66.3).*

**Coverage**: `EmuSen.WiseMan/Cores/CoreFactoryCheatWiringTests.cs`, seven tests, built to the report's shape rather than the fix's. Against the unfixed build six fail and only the SNES control passes — a per-core split that is what "none of my cheats worked, and it is not the codec" looks like from the outside. The six include both paused-apply tests, which cannot separate the two holes on the old build: an unshared registry fails them before the missing forward is reached. Removing only Mercury's forward from the fixed build is what separates them, and it reddens exactly the Game Boy paused-apply test. *(An earlier draft of this paragraph named only three of the six; re-measured 2026-09-15.)*

**What this does not reach, and how that step was closed anyway.** The emulation thread is not driven by a test; coverage stops at `EmulatorSession`, one call below `MainWindow`. That last call is confirmed by hand rather than by the harness: *2026-09-15, Super Mario Land 2 in Mistress from a source build, cheats applied and took effect* — the same report that opened §6, re-run against the fix. It is worth naming as a hand measurement, because it is the one step that will silently stop being covered if `MainWindow`'s cheat path is ever rewritten. §5's list is unchanged — `.cht` import still cannot produce ROM patches, and a database of Game Genie codes still imports as pokes at ROM addresses, which is a *different* way for a Game Boy cheat to do nothing and the next thing to look at if one still does.

## 7. Tests, and codes wider than a byte

*2026-09-18, with the N64's GameShark (`Mars_Cheats.md`).* Every format before it decoded one code to one byte, and the seam said so: `ICheatCodeCodec.Decode` returns an address and a byte. A GameShark code is lines — 16-bit writes, a line that tests memory and guards the next, a line that repeats the next — so two things were added, and nothing that existed changed meaning.

**A write can be a test.** `CheatWriteType` gained `IfEqual` and `IfNotEqual`: a write of that type compares its width in its byte order against its value (masked to its width, as a write's value is) and writes nothing. In `ApplyAll`, a failed test holds until the next write in the same cheat that is not a test, which it skips; so a run of tests is their AND, and one passing after one failing does not undo it. That is both N64 references' rule (`Mars_Cheats.md` §2.2). It is **not** RetroArch's: its handler form has the same two comparisons as `cheat_type` 4 and 5 ("run next if …"), but there a failure skips the next *entry*, whatever it is, so a second test after a failed first is skipped rather than evaluated, and what it guards runs. The names were kept apart from RetroArch's for that reason. `AddCheat` refuses a test that carries a repeat run or a bit position, and a ROM patch can hold no test (it already refused every type but `Set`). A test round-trips through the cheat file as its type's name, so a build older than this one skips such an entry rather than misreading it.

**A codec can decode a whole code.** `ICheatCodeCodec.DecodeWrites` is a default interface method returning null, like `DecodeCompare`, so no existing codec changed. When it answers, the three places a typed or imported code becomes a cheat add its writes as one cheat instead of asking `Decode` for a byte: `cheat add`, the Active Cheats window's Add button, and `ChtFile`'s code-string path — which hands such a codec the *whole* string before splitting on `+`, because a repeater spans a `+`. `cheat list` prints a test as `if <space> <address> == <value>`.

**`PrefersExplicitCodec` no longer picks an empty slot.** The N64 is the first console with one codec rather than two, and §2's tie-break sent a code that neither slot claimed to the punctuation guess — so a mistyped N64 code with a separator after its fourth digit (`8033-B21E`) went to the missing Game Genie slot and was answered "no Game Genie-style codec is registered", which is true and useless. With one slot empty the other is the answer, and its own error says what is wrong with the code. Consoles with both slots are unaffected; `A_malformed_n64_code_is_answered_by_the_gameshark` failed before the change and passes after it.

**`cheat export` skips a cheat containing a test**, and counts it beside the ROM patches it already skipped. The handler form holds one write per entry; a test written alone would lose what it guards, and written as `cheat_type` 4 it would round-trip into the defect below.

**A defect this surfaced and did not fix: RetroArch's comparison entries import as writes.** `ChtFile.ReadExplicit` maps `cheat_type` 1, 2 and 3 and sends everything else to `Set`, so an entry of type 4–7 ("run next if equal", "not equal", "less", "greater") imports as a write of its *compare value* to its address. Demonstrated on 2026-09-18 with a two-entry handler-form file — `cheat_type = 4` at address 16 with value 3, then a `Set` of 9 at 32 — which parsed to two `Set` writes, the first writing 3; the file came back with nothing skipped. It predates the test type and is untouched here, because fixing it well means turning RetroArch's cross-entry "next" into one cheat of several writes, which is an import feature rather than a repair. Until then such an entry does something neither RetroArch nor its author meant.

## 8. ROM patches from a `.cht` file, and patches wider than a byte

*2026-10-06, with the Genesis (§9).* The libretro database's Genesis files are two thirds Game Genie codes, and §5's first item made every one of them a skipped entry. `ChtFile.Parse` and `CheatImport` take the explicit codec as an optional last argument, `patchCodec`. An entry's code is a ROM patch when the patch codec claims it, every part of a `+`-joined code if not the whole, and the auto-detect codec does not: §2's rule, with no punctuation tie-break, because a file's code is never typed by hand. Such an entry carries `ChtCheat.Kind`, and the import adds it as a `RomPatch`. A patch codec that answers `DecodeWrites` is handed the whole code; one that does not is asked for a byte per part, and a part with a compare skips the entry, since an entry has nowhere to keep one (§3). `cheat import` and `cheat load` pass the target's explicit codec, and Mistress's cheat database window the one its console's codecs name. Without the argument nothing changes, which is why the SNES and NES tests that count skipped entries are unchanged. With it, the change reaches every console whose explicit codec decodes a code alone: a single SNES Game Genie code in a `.cht` file now imports as a ROM patch through `cheat import` and `cheat load`, where it was skipped, which is the behaviour wanted, and a `+`-joined one as one patch of a byte for each part.

The same day a ROM patch format first decoded to more than a byte. `cheat add`, `cheat gg` and the Active Cheats window's Add button ask the explicit codec for `DecodeWrites` before `Decode`, as they already asked the auto-detect one, and add what it answers as one `RomPatch` cheat; `ResolveRomPatches` already gave such a cheat to the core a byte at a time. No explicit codec before the Genesis's answers, so none of them changed.

**Coverage**: `EmuSen.WiseMan/DianaSh/GenesisCheatFormatsTests.cs`.

## 9. The Genesis's two formats

*2026-10-06, the system pack's codecs (`Sys/Systems/Genesis/GenesisCheatFormats.cs`), shared by the console, the Sega CD and the 32X as `Nephrite_Plan.md` §4.5 has it.*

**The Game Genie** (`ABCD-EFGH`, the explicit codec, a ROM patch). Written from "Sega Genesis Game Genie Conversion Method", a text after Merlyn LeRoy's newsgroup postings, the one Charles MacDonald's `genhw.txt` points to for the encoding, as prose: thirty-two digits, `A` to `Z` without `I`, `O`, `Q` and `U`, then `0` to `9`, each five bits; the forty bits rearranged by the string `ijklm nopIJ KLMNO PABCD EFGHd efgha bcQRS TUVWX` into a 24-bit address (`A` to `X`) and a 16-bit word (`a` to `p`). Its worked example, `SCRA-BJX0` to `$009C76` and `$5478`, is a test. The code is a word on the cartridge port's sixteen data lines; the port carries the 68000's address lines from A1 (`genhw.txt`, "The cartridge port presents the 68000's 23 address lines"), so a code whose address is odd patches the word it falls in, which is argued. The word is sent to the core as its two bytes, high byte first. The dash is required, so eight bare digits stay the Action Replay's.

**The Action Replay** (the auto-detect codec, a RAM poke in `WRAM`). Sega Retro's "Action Replay (Mega Drive)" gives the device's own print, an address and a value as `FFFFE 00001` (`$FFFFE0` set to 1), and the forms emulators print, `FFFFE0:0001` and `FFFFE0:1`. Taken: `AAAAAA:V` to `AAAAAA:VVVV` and `AAAAAA:VVVVVVVV`, `AAAAA AVVVV` with a space or a dash, and the same bare as ten or eight digits. The value's digits give its width, one or two a byte, three or four a word, eight a long word, written high byte first from the address given, odd or even; the libretro database prints a byte as two digits and a word as four, and the device's own form, always four, is a word. An address from `$E00000` up is the main RAM through its mirrors, poked at its low sixteen bits. An address in the cartridge's range, `$000000` to `$3FFFFF`, is a ROM patch, which the explicit codec claims; anything else is refused. Codes joined with `+` are one cheat in both formats; a code that mixes the two is refused, since a cheat is one kind.

**Measured against the libretro database** (the "Sega - Mega Drive - Genesis" files a published build carries, 2,094 games): 36,727 of the 38,546 entries load, 24,427 as Game Genie or cartridge patches and 12,300 as pokes; without the patch codec the same import loads only the 12,300. The 1,819 skipped are mostly entries with no code or a `??` the player is to fill in.

**What is argued, not measured**: a word at an odd address (2,571 of the database's four-digit pokes are at odd addresses, 2,442 of them with a high byte of 0, which may mean a byte); the value's width from its digits; the Game Genie on the Sega CD, which has no cartridge for it, and whose patches Nephrite's machine info gives no range for, so the frontend sends none.

## 10. The PlayStation's two formats (2026-10-11)

*Built at Diamond's stage 11, in the PlayStation's system pack (`Crystal/Systems/Psx/PsxCheatFormats.cs`).* Read for
it: psx-spx (the maintained conversion, `stable` at `6d7d1bc`), `cheatdevices.md`, its opening table "Separating
between Gameshark and Xplorer Codes", "Cheat Devices - Datel Cheat Code Format" and "Cheat Devices - Xplorer Cheat Code
Format" with the decryption below it; and the libretro database's `Sony - PlayStation` files, read for their codes'
forms. No emulator's source and no decoder program was read.

**The codes are RAM pokes in `RAM`**, the PlayStation little-endian, an address's low 21 bits the main RAM's offset, an
8-bit code's value its line's low byte. A line is eight hex digits and four; the database joins lines, and sometimes
an address's halves, with `+`, so a code is read as its hex digits in twelves. **The auto-detect codec, "GameShark or
Xploder"**, tells the two apart by psx-spx's first-digit table: a code whose lines' types are all the Datel device's
(3 and 8 shared, 1, 2, C, D, E, 0 for the dummy line and 5 for the slide) is decoded as a GameShark code, the
Datel table that Game Buster and Action Replay share, and any other as an Xploder code. **The explicit codec,
"Xploder"**, takes the Xploder alone, for a code typed in that form.

| GameShark | Becomes |
|---|---|
| `30aaaaaa 00dd`, `80aaaaaa dddd` | `Set`, width 1 or 2 |
| `10`/`11`, `20`/`21` | `Increase`/`Decrease`, 16 and 8 bits |
| `D0`/`E0`, `D1`/`E1` | `IfEqual`, `IfNotEqual`, 16 and 8 bits |
| `D2`/`E2` "If dddd<[aaaaaa]", `D3`/`E3` "If dddd>[aaaaaa]" | `IfGreater`, `IfLess`: the memory greater than the value, or less [C] |
| `5000nnbb dddd` + its line | one write repeated nn times, the address bb apart and the value ee apart, dddd the first value [C] |
| `00000000 0000` | nothing (the CD version's dummy between slides) |
| `C0`, `C1`, `C2`, `C3`, `D4`-`D6`, `12`/`22` | refused by name |

| Xploder (after decryption) | Becomes |
|---|---|
| `3t`, `8t` | `Set`, width 1 or 2 |
| `00aaaaaa dddd` | `Set`, width 4, `0000dddd` |
| `7t`, `9t` | `IfEqual`, `IfNotEqual`, 16 bits |
| `4t`, `5t`, `6t`, `Bt`, `Ft`, `C0`, `D0` | refused by name |

**The Xploder's decryption** is psx-spx's `decrypt_xplorer_cheat_code`: the key is the type's low three bits, keys 0,
4, 5, 6 and 7 decrypt the six bytes as the chapter gives them, and keys 1 to 3, for which it gives no rule, are
refused. Each key is shown to undo an encryption written from the chapter's steps in reverse, on 2,000 random codes
(`PsxCheatFormatsTests.Each_xploder_key_decrypts_what_it_encrypts_and_keys_1_to_3_are_refused`). The type's bit 3, the
device's default-off flag, is not a code's meaning; a cheat's own switch stands for it.

**The choices, argued.** The chapter's notes say it is "unclear if greater/less compare dddd by [aaaaaa], or
vice-versa" and "unknown if signed or unsigned"; the codec takes the table's text literally (`D2` passes when the
value is less than the memory) and compares unsigned, `IfLess` and `IfGreater` comparing the memory with the value
within the test's width. The slide's second line is the write it repeats (`80aaaaaa ??ee`), its value step the second
line's value, as the chapter's `dddd+(i*??ee)` reads. Chained tests keep the registry's AND rule (§7): no source says
whether the device skips a following test as a line, and the chapter's "exec next code" is what the rule already
does for a single test. `C0` is refused until a source or a test settles whether it latches (`EmuSen_CoreAPI_Extensions.md`
Q7). A 16-bit write at an odd address is applied byte by byte, the chapter leaving alignment unknown.

**Measured against the libretro database** (the 1,961 `Sony - PlayStation` files a published build carries; their
names give the format, 1,168 GameShark, 667 Game Buster, 102 Xploder and 24 neither): 17,869 of 22,116 entries load,
12,768 of 15,445 GameShark entries, 4,422 of 5,760 Game Buster and 546 of 670 Xploder. Of the 4,247 that do not, 3,592
hold `?` or a letter for a value the player picks and 204 are words or a code of odd length, which no codec could
apply; of the 18,320 that are codes, 97.5% load. The rest are refusals by name: 225 types neither table gives, 31 tests
with nothing after them, 23 `C1`, 20 `C2`, 10 Xploder `5t`, 7 `D4`-`D6`, 6 `C0`, the remainder fewer than five each
(`PsxCheatFormatsTests.The_databases_playstation_files_decode_and_each_refusal_is_named`, opt-in by
`EMUSEN_PSX_CHEAT_DB`). `EmuSen_CoreAPI_Extensions.md`'s P4 is retired in part: more than nine in ten codes decode, but
most of the rest are types neither table gives, not `D4`-`D6` and `C1`.

**A guard added on the way.** `ChtFile` took any explicit codec a caller passed as a ROM-patch codec. With a first
form of the codecs, the GameShark alone as the auto-detect codec and the Xploder as the explicit one, an Xploder code in
a `.cht` file would have been imported as a ROM patch, since only the explicit codec claimed it; the combined
auto-detect codec claims every such code, so no PlayStation code reaches that path now. A patch codec is now one whose
kind is `RomPatch` (`ChtFile.ClaimsPatch`); every console's patch codec is that kind, so nothing else changes. It was
argued from the code, not shown by a failing import, since the configuration that would have shown it was never
committed.

## The two combinations `AddCheat` refuses

Both throw rather than silently doing something adjacent, because there is no honest implementation of either:

- **A ROM patch cannot `Increase`/`Decrease`.** There is nothing to accumulate. The substitution happens at read time and the real byte is never written, so "add one each frame" has no meaning — the same byte would be read, incremented, and thrown away every time.
- **A compare byte only works on a single-byte patch.** `TryPatchRom` is handed one byte at a time by the cartridge read hook, so it cannot evaluate a compare spanning several addresses. Allowing it would produce a **torn patch**: the first byte declines to apply while the rest apply anyway, which is worse than either outcome on its own.

`ApplyAll` takes a read delegate as well as a write one for the mirror-image reason — `Increase`/`Decrease` and bit-position writes have to see what is already there — while the registry itself still never touches memory. The write-only overload is kept so existing callers compile; those write kinds read back as zero there.
