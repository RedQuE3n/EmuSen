# Diamond_Cards — memory cards a player carries between games

*Since 2026-10-15 the card mode is the framework's, for every console with save media: `EmuSen_SaveMedia.md` holds the
model, the core ABI's minors 1.7 and 1.8, the runtime's modes, the files, the lock, the card manager and the default.
This page keeps what is the PlayStation's: the hazard as Diamond met it, how Diamond keeps its cards apart, its states
across modes, its card format, and what was measured on it. §3 to §5, §9, §12 and §13 are now short pointers to the
framework's page; their text of the morning of 2026-10-15 is in this file's history.*

*Written 2026-10-15, before its code.* Decided 2026-10-15 by the tester: the memory card a player uses is to appear in
every game played, as on a console, through a **card mode** with three modes, *Per game* (the behaviour of
`Diamond_Plan.md` §4.4 and `Diamond_Players.md` §4, unchanged), *Shared* and *Named cards*. This page records the form
chosen before any code was written (§1 to §9), what was built and measured afterwards (§10 on), the card manager that
is written and not built (§12), and the questions left to the tester (§13). The marks are the plan's: **[M]** measured,
**[R]** read, **[A]** argued, **[C]** a choice.

---

## 1. What exists, and the hazard it carries

- **Each game has its own cards** [R, `Diamond_Pad.md` §6, `Diamond_Players.md` §4]: battery file 0 is the card in
  slot 1, file 4 the card in slot 2, each the raw card of 131,072 bytes, written by the runtime's battery rule as
  `Saves/PlayStation/<game>.mcr` and `<game>.2.mcr`, the game being the file the player chose (an `.m3u` for a set).
  Slot 1 holds a card by default and slot 2 none (`card1`, `card2`).
- **A state holds every card's contents** [R, `src/state.rs`; `Diamond_Native.md` §14: "each card adds its 131,072"],
  with the card's flag, its transfer registers and its buffer. A load restores them and **marks each card changed**,
  so that the host writes what the state holds into the card's file (`Diamond_Pad.md` §6).

Under *Per game* that is the intended behaviour: the card is the game's, and a state of the game rolls the game's card
back with the rest of the machine, as a player of a per-game card expects. With one card used by several games it is
destructive [A]: a player saves in game A, makes a state of A, plays game B and saves there, returns to A and loads the
state. The load puts back the card as it was when the state was made, marks it changed, and the runtime writes it over
the shared file. **B's save is gone**, though B was never loaded again. Rewind does the same with every snapshot it
loads. The proof of it on the unchanged code is §10.1. `EmuSen_CoreAPI_Extensions.md` §9.3 named this risk on
2026-10-10 as the reason a shared card was not built with stage 11.

A second, quieter hazard follows from any fix that merely keeps the card [A]: the game's RAM, restored by the state,
holds what the game and the kernel last read of the card's directory. If the card now holds blocks the state's RAM
does not know of, a save written from that stale copy can allocate a block B already uses. On a console the same
situation arises when a card is pulled and another pushed in, and the hardware's answer is the card's **flag byte**:
bit 3 is set at power-on and on insertion, and cleared by the first good write (psx-spx "Memory Card Read/Write
Commands", "FLAG Byte" [R]); the kernel's card functions report it as a new card (`EvSpNEW`), after which a game reads
the directory again before writing. Diamond's card and its open kernel model both halves already (`src/card.rs`,
`FLAG_NEW`; `kernel/src/card.rs`, `SPEC_NEW`).

## 2. The three modes

| Mode | Slot 1 | Slot 2 (when the `card2` setting holds a card) | A state load or a rewind |
|---|---|---|---|
| **Per game** | `Saves/PlayStation/<game>.mcr` | `Saves/PlayStation/<game>.2.mcr` | restores the cards the state holds, marked changed, as today |
| **Shared** | `Saves/PlayStation/Cards/Card 1.mcr` | `Saves/PlayStation/Cards/Card 2.mcr` | leaves each card as it stands; a card that differs from the state's copy is presented as newly inserted |
| **Named cards** | `Saves/PlayStation/Cards/<name>.mcr`, the name the player chose for slot 1 | `Saves/PlayStation/Cards/<name>.mcr`, the name chosen for slot 2 | as Shared |

**Slot 2 under Shared is a second shared card, `Card 2`, and stays empty by default** [C]. A console's two slots hold
two physical cards; one file behind both slots would be two cards writing one file, which no console can be. Keeping
slot 2 empty by default is the plan's §4.4 default and Q12's recommendation, unchanged: few games look at slot 2, and
a game that finds two cards asks the player which to use. A player who turns slot 2 on gets the same second card in
every game, as a console owner who leaves a second card plugged in.

*Named cards*, the drawer, the rule that switching modes moves, copies and deletes nothing, and the default (*Shared*
since 2026-10-15, with the import offered once) are the framework's: `EmuSen_SaveMedia.md` §3.3 and §9. The PlayStation's
card is the medium `psx.card`, its two slots `Slot 1` and `Slot 2`, its noun `Card`, so its shared and named cards keep
the names and the folder of this table.

## 3. Who owns which fact, the forms weighed, and the core ABI's minor 1.7

Now the framework's: who owns which fact is `EmuSen_SaveMedia.md` §4; the forms weighed for the removable kind and its
create flag, and the minor 1.7 they became, are `EmuSen_CoreAPI.md` §6.27, with the console kind and a slot's medium,
label and port added at 1.8 (§6.28 there). What the PlayStation added to them is §6 below.

## 6. Diamond under the flag

- **Its files are removable**: files 0 and 4, `.mcr` and `.2.mcr`, in core info and in machine info; since 1.8 core
  info gives each the medium `psx.card` and the labels `Slot 1` and `Slot 2` (`EmuSen_SaveMedia.md` §7.1).
- **A state load restores a card's contents only when neither the machine nor the state had its cards apart from the
  game** [C]. Otherwise the card the machine holds stays, byte for byte, with its `changed` mark as it was, so the load
  makes the runtime write nothing; the state's copy is read and set aside.
- **A card that differs from the state's copy is presented as newly inserted** [C]: its flag gains bit 3, and a
  transfer the state left under way with that card is ended, the port silent until the slot is deselected, as a card
  pulled mid-transfer leaves it. A card equal to the state's copy is the card the state's RAM knows, so it takes the
  state's flag and transfer, and the game cannot tell the load from a console's own continuation [A]. The comparison is
  128 KiB per card per load, outside any frame.
- **The state marks its cards as kept apart** with an empty optional block, tag `SHRD`, after the picture, of the form
  `Diamond_Enhancements.md` §4 gave the enhancement layer's blocks: 8 bytes, which a reader that does not know the tag
  skips, so the version stays 10 and every build that reads version 10 reads such a state [C]. A machine made per game
  writes no such block, so **per-game states are unchanged byte for byte**.

## 7. Rewind, in each mode

Mistress's rewind loads a state for each step back (Diamond claims no `SNAPSHOT`, so the snapshot is the state)
[R, `EmuSen_Rewind_And_FastForward.md`]. Under *Per game*, rewinding past a save takes the save off the game's card,
as today: the card is the game's and rewinds with it. Under *Shared* and *Named cards* the card does not rewind: a save
made and then rewound past stays on the card, as a save made on a console stays on the card whatever the player does
next, and the game, finding the card changed, reads its directory again [A]. A rewind that crosses no save finds the
card equal to its copy and changes nothing the game can see.

## 8. A state made in one mode and loaded in another

| Made under | Loaded under | The card after the load |
|---|---|---|
| Per game | Per game | the state's, as today |
| Per game | Shared or Named | the shared or named card as it stands (the flag decides) |
| Shared or Named | Shared or Named | the card in the slot as it stands, whichever card it is |
| Shared or Named | Per game | the game's own card as it stands (the state's `SHRD` block decides) |

The last row is why the state carries its mark [A]: without it, a state made with a shared card and loaded per game
would write the shared card's contents, other games' saves included, over the game's own per-game card, and the
per-game saves made since would be lost, which is exactly the loss the tester ruled out. A state made by a build from
before this page carries no mark and is treated as per game, which it was.

## 9. The runtime

The runtime's modes, rows, files and lock are the framework's (`EmuSen_SaveMedia.md` §3; `EmuSen_Settings_Reference.md`
§4.114). For the PlayStation: the rows are **Memory Card** (`Cards`) and, under *Named*, **Memory Card: Slot 1** and
**Memory Card: Slot 2** (`Card1`, `Card2`), the keys of 1.7 unchanged; the card's format module is `PsxCardFormat`
(§12), so its default is *Shared* and its import is offered once (`EmuSen_SaveMedia.md` §9). The core's own *Memory card
in slot 1/2* rows still decide whether a slot holds a card at all. Two programs on one card are now refused by the lock
(`EmuSen_SaveMedia.md` §3.5), which answers §13's Q3.

## 10. What was measured

### 10.1 The hazard, on the code before the card mode

**The program** [M]: `programs/card-save.s` is a game's save in miniature. It takes the card's ID, writes one sector
with 128 bytes of one value through the card's write command, byte by byte as the kernel's driver sends them, and then
takes the ID again and again, keeping the flag each answer carries. Which sector and which value are two words after
its first jump, so that two images of it are two games that save in two places of one card. It is synthetic: no game
or BIOS is involved, and the open kernel's card functions are not used, so the test isolates the protocol and the
state.

**The proof** [M, 2026-10-15, `v1::tests::a_shared_cards_save_survives_a_state_of_another_game` written first and run
on WiseMan 76472894's core]: game A (sector 100h, A5h) saves on a blank card file and the host writes the file; a state
of A is taken; game B (sector 140h, B6h) is made over the file, saves, and the host writes the file, which then holds
both saves; A is made over the file again and its state loaded, the host writing the card whenever the machine reports
it changed. The test asserted that B's sector still held B6h, and **failed**: "B's save survives A's state: [00, 00,
00, 00, 00, 00, 00, 00]". The two assertions before it, that each save reached the file, passed. The test is kept as
`per_game_a_state_puts_its_card_back`, which asserts the loss, since under *Per game* the same steps are the intended
rollback of the game's own card.

### 10.2 The core, built

[M, 2026-10-15] `emusen-native` and the header at 1.7 (§5); Diamond's cards removable in core info and machine info;
`Machine::cards_apart` set from the flag at create; `state.rs`'s `keep_cards` applied on a load when the machine or the
state had its cards apart, and the empty `SHRD` block written by a machine with its cards apart. With the flag the
hazard's steps keep both saves, the card the machine holds is the file byte for byte, and the program's next ID after
the load answers flag 08h (`a_shared_cards_save_survives_a_state_of_another_game`). The rules of §6 and §8 hold on
unit cases (`state::tests`): a per-game state is unchanged in size and bytes and a kept-apart one is the same followed
by `SHRD` and four zero bytes (`a_state_made_with_the_cards_apart_says_so_and_one_made_per_game_does_not`); a card
equal to the state's copy keeps the state's flag 00h, its command and the transfer under way, while one that differs
reads 08h, its command cleared and the port silent, and a change the host has not yet saved stays to be saved
(`kept_apart_a_card_stays_and_only_a_different_one_reads_as_inserted`); and the four rows of §8's table
(`a_state_made_in_one_mode_and_loaded_in_another`).

### 10.3 Per game, unchanged

[M, 2026-10-15] The digests of three per-game states, FNV-1a over the whole state, were taken on WiseMan 76472894 in
a probe checkout (`~/.cache/emusen/probe/diamond/cards/base`, removed once recorded) and on this branch, from
`card-save.s` saving on a blank card: the state after the save, the same state loaded over a blank card and saved again,
and five frames on. Both builds give `886135FFEF135A26`, `886135FFEF135A26` and `55CDED3F58B6961E`
(`per_game_states_are_the_ones_before_the_card_mode`). The project's program goldens hold all 49 lines unchanged, the
new program adding its fiftieth (`card-save.s`, recorded by Diamond_Enhancements.md §6.2's command and checked), and
the state's pinned layout test is unchanged.

### 10.4 The runtime and Mistress, built

[M, 2026-10-15] `MemoryCards` (`EmuSen/Cores`) owns the mode's facts: its keys, its values, the rows of §9.1, the
stored choice read safely (an unknown mode is *Per game*, a name with a separator or a leading dot is the slot's
default, so no hand edit of `graphics.json` reaches outside the folder), a removable file's slot as its place among
the removable files core info lists, and the card a slot holds. Galaxia's `SaveLibrary` gains `CardsFolder`,
`CardPathFor` and `CardNames`, composed on `DataStore.Saves`; `BatterySave.Card` opens a card by its own path and keeps
`--nobattery`'s latch. `CoreEngine.Cards`, set by `CoreFactory` from the console's stored values, places each file at
create: a removable one under *Shared* or *Named cards* at its card, every other at the game's, and the machine is made
with the flag when any listed file is removable. The C# host reads `kind` into `CoreBatteryFile.Removable` and is at
1.7. In Mistress the rows are built by `BuildCardRows`, after *Engine* and before the core's rows, each slot's row
hidden unless *Named cards* is chosen, as the crop's row is (`EmuSen_Settings_Reference.md` §4.114).

Through the host, on two synthetic games that each write one frame of the card in slot 1 (`DiamondCardsTests`): per
game, A's save is in `Amber Saga.mcr` and B finds a blank card, and no `Cards` folder is made; shared, A's save is in
`Cards/Card 1.mcr`, B's machine is made with it on its card, B's save lands beside it, no per-game file is written, and
*Per game* chosen again gives B its own blank card back; named, A saves to `Blue`, B to `Red` without seeing A's save,
and B with `Blue` chosen sees A's and not its own; the hazard's steps through `CoreEngine` leave both saves on the
shared file; and a state from before a save, loaded and the cards written as at the next frame's end, empties the
game's own card per game and leaves the shared card's save. The fit audit's new case shows the tab under *Named cards*
with both slots' rows and the note, nothing cut off or overlapping at 1280×800 and 1920×1200.

### 10.5 The kit on the sample's discs

[M, 2026-10-15] `kit/psx-sample.txt`'s command, the library this branch's build made, the nine discs at 1,800 frames,
nine at once under a 10 GiB limit: the core's cases pass and **9 of 9 images pass**, C19 among their cases, each
reporting its one removable file (file 0, 131,072 bytes) kept as it stood across a state made 120 frames into a
machine whose card held another pattern. Ape Escape's C6 digests are those recorded on 2026-10-12
(`Diamond_Native.md` §23.4), `A02BFAE09D41CF89` for the pictures, and every disc's state is 5,398,770 bytes as there:
the kit sees the per-game machine unchanged. The run took 277.9 s against §23.4's 190 s, beside other builds on the
desktop, with 9.2 GB peak resident.

## 11. Tests

Run on 2026-10-15, the blast radius only:

| Where | What | Result |
|---|---|---|
| `diamond`, `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS` | the crate's unit tests, among them the seven of §10.1–§10.3, and its program, golden, renderer and precision tests | 173 unit, 81 program, 14 precise, 7 scaled, 4 filtered, 3 colour and 3 renderer tests pass |
| `diamond-v1` | the published library's tests | 4 pass |
| `emusen-native` | the crate's tests, the header's numbers at 1.7, every descriptor against its schema, core info's removable file | 74 pass |
| `emusen-core-abi-check` | the header, the Rust and the baseline | 429 facts agree on 4 triples; 6 tests pass |
| `emusen-core-conform` | the kit's tests, C19 with the test core's card and its seeded fault failing C19 alone | 29 pass |
| MercuryRT, MoonRT, VenusRT, Nephrite | their `Battery` and `Create` literals moved | build with their tests |
| WiseMan | `DiamondCardsTests` (7), `DiamondPlayersTests`, `DiamondTests`, `CoreAbiTests`, `CoreAdapterTests`, `CoreDiscoveryTests`, the `GraphicsSettings` tests, `PlayStationLibraryTests`, `SaveLibrary` tests, the fit audit's PlayStation cases with the new one at both sizes | all pass |
| the kit on the sample's nine discs | §10.5 | 9 of 9 |

## 12. The card manager, and the PlayStation card's format module

*Built 2026-10-15 as the framework's* (`EmuSen_SaveMedia.md` §6): one generic tool, `SaveMediaManager` in Crystal, over a
format module per medium; DianaSh's `memcard` and Mistress's Memory Cards window over it. The form argued here on the
morning of 2026-10-15 (the reader in the PlayStation's system pack as pure functions, the command first, then the
window, the import refused whole past 15 blocks and copying, never moving) is the form built.

**The PlayStation's module**, `PsxCardFormat` in `EmuSen.Crystal/Systems/Psx`, reads and writes psx-spx's "Memory Card
Data Format" [R]: the header frame (`MC`, checksum 0Eh), the fifteen directory frames (allocation states 51h, 52h and
53h in use, A0h free, A1h to A3h deleted; the file size and the name in a file's first frame; the next block's number,
FFFFh for none; the XOR of bytes 00h–7Eh in byte 7Fh), the broken-sector list and its replacement frames, the title
frame (`SC`, the Shift-JIS title, read with code page 932 and its full-width letters folded to ASCII) and the blocks
themselves. A save is named by its directory name (`BASLUS-00067DRACULA`), its game by the serial the name carries
(`SLUS-00067`). A copy takes the lowest free frames and writes the chain with its checksums; a delete marks a file's
frames A1h–A3h and keeps its blocks, as the BIOS does, so the open kernel's `_card_undelete` could bring it back; a
chain that points at a free frame, loops or ends wrongly is listed damaged and never copied; formatting writes block 0
as psx-spx lists a formatted card's, which is what the open kernel's `format_card` writes. Its tests are
`SaveMediaManagerTests`, against cards whose saves the tests write into the directory themselves.

## 13. Questions for the tester

*Answered 2026-10-15*: all five recommendations were accepted (`EmuSen_SaveMedia.md` §0). Q1's default is *Shared* now
that the import exists, with the import offered once; Q2 is as built; Q3's lock is built (§9); Q4's naming is the card
manager's; Q5 is `EmuSen_SaveMedia.md` §7.

## 14. Reading log, and exposures

Read for this page: psx-spx `stable` `6d7d1bc`, "Memory Card Read/Write Commands" and its "FLAG Byte", and "Memory Card
Data Format" for §12; the project's own pages and code (`Diamond_Plan.md` §4.4 and §11, `Diamond_Pad.md` §6 and §8,
`Diamond_Players.md` §2 to §4, `Diamond_Native.md` §14, `Diamond_Enhancements.md` §4 and §6, `EmuSen_CoreAPI.md`
§5.5, §6.9, §6.11 and §6.24, `EmuSen_CoreAPI_Extensions.md` §9 and §10.5, `EmuSen_Settings_Reference.md` §4.44 and
§4.99, Galaxia's `SaveLibrary`). No emulator's source, no BIOS and no game was read; the player's BIOS folder and the
ROM library were not opened. Every test uses temporary folders and synthetic programs.
