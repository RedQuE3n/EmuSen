# Diamond_Cards — memory cards a player carries between games

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

**Named cards** are the console owner's drawer of cards. A slot's choice is a name, and its file is that name in the
`Cards` folder; the names offered are `Card 1` to `Card 8` and every card file already in the folder, so that a card
copied there by hand, or by the card manager later (§12), can be chosen [C]. A name with no file yet is a new,
unformatted card, which the game or the kernel formats, as a new physical card. The same name in both slots is
refused (§9.3).

**Switching modes moves, copies and deletes nothing** [C]. The per-game files stay where they are while another mode is
chosen, and are used again when *Per game* is chosen again; the shared and named cards stay in `Cards` likewise. That
is the way back the tester asked for: a player's existing per-game saves are never lost, and they are hidden only
while the player has chosen a mode that does not use them, with the setting itself as the way back. Moving saves from
per-game cards onto a shared card is the card manager's import (§12), never a side effect of the setting.

## 3. Who owns which fact

By the project's rule of one owner for each fact:

| Fact | Owner | Why |
|---|---|---|
| Which files are cards a player carries, as against memory that belongs to the game | **the core** | which numbers are the core's (`EmuSen_CoreAPI.md` §6.2); another engine of the same console may number its files otherwise, so the fact must come with the numbers (`EmuSen_CoreAPI_Extensions.md` §9.2) |
| The card mode, and which card is in which slot | **the runtime**, as a setting of the console | the place of a battery file is the runtime's path rule (`EmuSen_CoreAPI.md` §6.11); no core sees a path |
| Where a shared or named card lives | **Galaxia** | it owns every save path; the rule is `Saves/<console>/Cards/<name><extension>`, beside the per-game files' folder |
| What a state load does to a card that is not the game's own, and how the machine learns of a different card | **the core** | it is the machine's behaviour: the flag byte and the transfer under way are Diamond's state, which no host can reach |
| Whether a card is the game's own | **the runtime tells the core**, once, at create | the runtime chose the file, so only it knows |

The last row is the one crossing that needs a surface, which §4 weighs.

## 4. The forms weighed

By the design rules' order, an extension of an existing surface first:

1. **A setting in Diamond's own schema** (`cards=own|shared`), which the runtime would set from the mode. Rejected [A]:
   the runtime would have to know a key of one core's schema to place files, which is per-core code above the ABI
   (`EmuSen_CoreAPI.md` §2.4), and the mode would have two owners, the runtime's setting and the core's.
2. **The host puts the card back after each load**, through `space_write` into a card space, or a new export that
   hands a battery file's bytes to a running machine (`battery_insert`). Rejected for this step [A]: a space write
   needs the host to know which space is which file, which no descriptor says, and cannot reach the flag or the
   transfer; an export is the larger addition (an export, a capability, a typedef, the host's call after every load
   and every rewind step, two copies of 128 KiB each time), and still needs the core to compare the card with the
   state's copy to decide whether it was "reinserted". Its shape is kept for the card manager, whose card swapped while
   a game runs would need it (§12.4).
3. **Chosen: an extension of the core ABI, minor 1.7** [C], in two parts, each the smallest the fact allows:
   - **`kind` on battery entries**, in core info's `systems[].battery[]` and machine info's `battery[]`: absent or
     `"game"` for memory that belongs with the game, `"removable"` for a card or pak a player carries between games.
     This is `EmuSen_CoreAPI_Extensions.md` §9.2's recommendation, less its third value, `"console"` (memory inside the
     console, as the Sega CD's), which no core yet needs and which would need a rule of its own [C].
   - **`EMUSEN_CREATE_SHARED_REMOVABLE`**, bit 1 of `emusen_create_params.flags`: the host keeps this machine's
     removable battery files apart from the game, so that a state load must leave them as they stand.

A host offers a card mode for a system only where core info lists a removable file, and sets the flag only on such a
core, so that a core older than 1.7, which lists no kind and ignores the flag, is only ever run per game [C]. This is
the coupling that makes a skewed pair safe: a core that writes `"removable"` promises to honour the flag.

## 5. The core ABI's minor 1.7

By `EmuSen_CoreAPI.md` §5.5's rules: the declarations tagged `@since 1.7` in `emusen_core.h` with
`EMUSEN_CORE_ABI_MINOR` raised to 7, the baseline's new lines tagged `@1.7`, `emusen-native`'s definitions with their
defaults, the C# host's constants, a conformance case and the changelog. The text added to `EmuSen_CoreAPI.md` is its
§6.27; its essentials are:

```c
/* @since 1.7 (section 6.27): the host keeps every battery file machine info lists with kind "removable" apart from the
 * game, shared between games. A state_load of either kind leaves each such file's contents as they stand; what else
 * the machine does to present a card that differs from the state's copy is the core's. Set only for a core whose
 * info lists a removable file; a core that lists one honours it. */
#define EMUSEN_CREATE_SHARED_REMOVABLE 2u  /* emusen_create_params.flags */
```

- **JSON**: `info.systems[].battery[].kind` and `machine.battery[].kind`, optional strings, `"game"` or `"removable"`;
  absent is `"game"`, so every core that writes nothing is unchanged byte for byte, and the registration golden
  (`EmuSen_CoreAPI.md` §25) with it.
- **`emusen-native`**: `Battery` gains `kind: BatteryKind` (`Game` by default, written only when `Removable`), and
  `Create` gains `shared_removable: bool`, read from the flag by `core_exports!`'s create.
- **The rule a core keeps** under the flag: after a successful `state_load`, `battery(which)` of each removable file
  returns the bytes it returned before the load, and the load does not raise its `BATTERY_CHANGED` flag. A refused load
  changes nothing, as ever.
- **The kit**: C14 checks that a `kind` is one of the two values and that a machine's removable files are listed
  removable in core info; **C19**, where a machine lists a removable file, creates it with the flag and its file filled
  with a pattern, saves a state, writes another pattern into the core's view through nothing but frames and loads (a
  second machine made with the second pattern loads the first's state), and checks that the second pattern survives
  the load with its changed flag clear. Without the flag C19 asks nothing, since per game the core's choice stands.
- **Skew** (`EmuSen_CoreAPI.md` §4.6): a 1.6 host never sets the flag and never reads `kind`, so it runs every card
  per game, as now; a 1.6 core lists no kind, so a 1.7 host offers it no mode.

## 6. Diamond under the flag

- **Its files are removable**: files 0 and 4, `.mcr` and `.2.mcr`, in core info and in machine info.
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

### 9.1 The setting

The PlayStation's tab in Graphics Settings gains a group of rows of the runtime's own, as the *Engine* row is the
runtime's and not the core's (`EmuSen_Settings_Reference.md` §4.44), stored in `graphics.json` under the console and
never passed to the core: **Memory cards** (`Cards`: `game`, `shared`, `named`; *Per game* the default), and, with
*Named cards*, **Card in slot 1** and **Card in slot 2** (`Card1`, `Card2`). The rows appear for any console whose
engine lists a removable battery file, so a later core with cards gets them with no code of its own. They sit beside
the core's own *Memory card in slot 1/2*, which still decide whether a slot holds a card at all.

### 9.2 The files

The engine's battery files are placed by the runtime's rule as now, with one change: a removable file, under *Shared*
or *Named cards*, is `SaveLibrary.CardPathFor(console, name, extension)`, the extension being the last part of the
core's suffix (`.mcr` of `.2.mcr`), and the machine is created with `EMUSEN_CREATE_SHARED_REMOVABLE`. A game file that
is not removable keeps its per-game place in every mode.

### 9.3 Two slots and two machines on one card

The same name in both slots under *Named cards* would be one file behind two cards. The setting says so under the
row, and the engine then runs slot 2 per game, with words in the log [C]. Two machines on one card file at once (two
programs of the project running PlayStation games) would each write the file from its own copy; one program runs one
game at a time, and a lock across programs is the question of §13 rather than built [C].

---

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

## 12. Step 2, written and not built: a card manager

*Written 2026-10-15; nothing of it is built.* What a player of *Shared* and *Named cards* will want next is a console's
card manager: to see what is on a card, to copy a save to another card, to delete one, and, the step that makes
*Shared* a default anyone could choose, to bring the saves of the per-game cards they already have onto a shared card.

### 12.1 What it must do

- **List** a card's saves from its directory: frame 0's header, frames 1 to 15 one per block, each with its state
  (first, middle or last block of a file, free, or deleted), the file's size, the next block of its chain and its name
  (a region prefix and the game's serial, as `BASLUS-00067...`), each frame's checksum the XOR of its first 127 bytes
  (psx-spx "Memory Card Data Format" [R]); and for each file its title and icon from its first block's header (the
  `SC` magic, the icon's frame count, the block count, the Shift-JIS title of 64 bytes, the palette and the frames).
- **Copy** a save to another card: free blocks found, the chain written with its directory frames, the checksums made;
  a name already on the target refused unless the player replaces it.
- **Delete** a save as the BIOS does, marking its directory frames deleted rather than clearing its blocks, so that an
  undelete stays possible (the open kernel's `_card_undelete` does the same, `Diamond_OpenKernel_Phase2.md` §14).
- **Move** a save between cards: a copy, then the delete, the delete only after the copy is written and read back.
- **Import** the saves of a player's per-game cards (`Saves/PlayStation/<game>.mcr` and `.2.mcr`) onto a shared or named
  card: every save of the chosen games copied, **refused as a whole when the 15 blocks of the card would be exceeded**,
  with words naming the saves and the blocks each needs, so that a player never gets half an import. The per-game cards
  are read and never written: the import copies, never moves, so the way back of §2 stays.
- **Never write a card a running machine holds.** The machine writes its card's file whenever the game writes the
  card (`BATTERY`), so a manager's write while it runs would be overwritten, or would overwrite a save the game made.
  The manager refuses such a card with words; writing one under a running game would need the core ABI to hand a
  card's bytes to a machine (§4's second form, an export `battery_insert` under its own capability, the core
  presenting the card as reinserted), a minor of its own when a player needs it.

### 12.2 The forms weighed

By the design rules (`Diamond_Plan.md` §2, the project's rules on modules and surfaces), with one question first:
**where the card format is read and written**, since every form needs it.

| Where the format lives | For | Against |
|---|---|---|
| In Diamond, the core | Diamond has the card | the core is a machine behind the ABI, which has no surface for a file system and should not gain one; the on-card format is a convention of Sony's kernel, not of the card's hardware, so it is not the machine's fact; a host reaching into a core for it is what the surfaces forbid |
| The open kernel's own code (`kernel/src/bu.rs`, `blocks.rs`) | the format already written once, in Rust | it is a program the emulated R3000A runs, `no_std` MIPS inside the console's RAM, not a host library; sharing it would couple the kernel's build to the frontend's |
| **The PlayStation's system pack** (`EmuSen.Crystal/Systems/Psx`, C#) | it is where the runtime keeps the console's knowledge above the ABI: the cheat codecs (`EmuSen_Cheats.md` §10), the disc recogniser (`Diamond_Players.md` §1); both clients below are C# and reach it; it needs no surface of its own | a later Rust runtime (`EmuSen_Stack.md` §6, deferred) would port it, which a reader of pure functions over 131,072 bytes makes mechanical |
| A Rust crate behind the platform library's C interface | one implementation for a Rust future | a new interface version for a feature with two C# clients today; the platform library's interfaces are for the runtime's services, not a console's knowledge |

**The reader belongs in the system pack** [C]: a `PsxCard` of pure functions, bytes in and bytes out (parse the
directory, list the saves, copy, delete, the checksums), with no file I/O; the paths stay Galaxia's (`CardPathFor`,
`CardNames`, the per-game rule), and the writes go through `AtomicFile`, as every save does.

Then the two clients:

| Form | For | Against |
|---|---|---|
| **A DianaSh command module**, `memcard` (`memcard ls CARD`, `memcard cp CARD:SAVE CARD`, `memcard rm`, `memcard mv`, `memcard import GAME... --to CARD`) | Unix-like, one job per command, scriptable for the import over a whole library; testable headless; runs without an emulator, as DianaSh must (`EmuSen_DianaSh.md`); its man page is where the rules above are written | no icons; not reachable from a controller |
| **A Mistress window on LunaP**, "Manage cards…" from the PlayStation's tab and big picture's game menu | what a player at a television needs: two cards side by side with their saves' icons and titles, a pad's A to copy and X to delete, as a console's manager | a window's fit and pad audits to pass; the window alone leaves the import unscriptable |

**Recommended: both, over the system pack's reader, the command first** [C]: the command proves the reader and the
import on synthetic cards built by the tests (the open kernel's own card program, `kernel-card.s`, writes real
directory frames to borrow from), and the window then adds nothing to the format but the drawing. Each client is a
module of its own: the command a DianaSh module with its man page under `Lib/dianash/emusen.memcard/man/memcard.1`, the window a LunaP
window of Mistress. The `.gme` and `.vmp` conversions the plan's §4.4 left to "a later tool" fit the same reader as an
import and an export, in the command first.

## 13. Questions for the tester

1. **Q1, the default mode.** Built: *Per game*. *Recommended:* keep *Per game* as the default until the card
   manager's import (§12.1) exists, then make *Shared* the default and offer the import once, the first time a game
   is loaded under *Shared* while per-game cards exist (copied, never moved). The reason is the player who updates and
   starts a game: under a *Shared* default today their saves are all still on disk but the game finds none, and a player
   who does not know the setting has no way back they can see. Per game first, then Shared with the import, never
   hides a save.
2. **Q2, slot 2 under *Shared*.** Built: `Card 2`, a second shared card, with slot 2 empty by default as the plan's
   §4.4 keeps it (§2). *Recommended:* as built.
3. **Q3, two programs on one card.** Two of the project's programs running PlayStation games at once on the same
   shared card would each write the file from its own copy, the last writer winning. Not built (§9.3). *Recommended:* an
   advisory lock on the card's file held for a machine's life, a second machine given the card unshared with a notice,
   built with the card manager, which needs the same lock to refuse a card in use.
4. **Q4, naming a new card.** Built: *Named cards* offers `Card 1` to `Card 8` and any card file in the folder.
   *Recommended:* naming and renaming in the card manager, not a text box in the settings, which a pad reaches badly.
5. **Q5, other consoles' cards.** The N64's Controller Pak (Mars's `.mpk`) and the Sega CD's RAM cartridge (`.crm`)
   are removable too; neither core lists them so yet, and each would honour the flag as Diamond does. *Recommended:*
   each when its core is next worked on, a step of its own; the Sega CD's internal backup RAM is memory inside the
   console, `EmuSen_CoreAPI_Extensions.md` §9.2's `"console"` kind, and would need a rule and a minor of its own.

## 14. Reading log, and exposures

Read for this page: psx-spx `stable` `6d7d1bc`, "Memory Card Read/Write Commands" and its "FLAG Byte", and "Memory Card
Data Format" for §12; the project's own pages and code (`Diamond_Plan.md` §4.4 and §11, `Diamond_Pad.md` §6 and §8,
`Diamond_Players.md` §2 to §4, `Diamond_Native.md` §14, `Diamond_Enhancements.md` §4 and §6, `EmuSen_CoreAPI.md`
§5.5, §6.9, §6.11 and §6.24, `EmuSen_CoreAPI_Extensions.md` §9 and §10.5, `EmuSen_Settings_Reference.md` §4.44 and
§4.99, Galaxia's `SaveLibrary`). No emulator's source, no BIOS and no game was read; the player's BIOS folder and the
ROM library were not opened. Every test uses temporary folders and synthetic programs.
