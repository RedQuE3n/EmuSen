# EmuSen — save media: memory cards, paks and a console's own memory, for every console

*Written 2026-10-15, before its code.* The PlayStation's card mode (`Diamond_Cards.md`, merged the same day) made a
player's memory card follow them between games. This page makes the concept the framework's: one model of save media
for every console that has them, a minor of the core ABI that lets a core declare them, one runtime rule for where each
medium's files live and what a state does to them, and one card manager over modules that read each medium's own file
system. §1 to §10 were written before any code; §11 on records what was built and measured. The marks are those of the
project's plans: **[M]** measured, **[R]** read, **[A]** argued, **[C]** a choice.

---

## 0. Decided 2026-10-15

The tester's go, after `Diamond_Cards.md` was merged: the memory card concept is to be **agnostic and modular**, so that
another console's cards and similar media are handled as the PlayStation's are; and every recommendation of
`Diamond_Cards.md` §13 is accepted:

- **Q1.** The default stays *Per game* until the card manager's import exists; then **Shared becomes the default, with
  the import offered once**, copying and never moving, so that nothing a player has is lost or hidden without a visible
  way back.
- **Q2.** Under *Shared*, slot 2 is a second card, `Card 2`, empty by default.
- **Q3.** An advisory lock, so that two programs never write one card at once, built with the card manager.
- **Q4.** New cards are named in the card manager, not by a text box in the settings.
- **Q5.** Other removable media are marked removable when their cores are worked on, and **a `console` kind** (save
  memory inside the console, shared by every game, the Sega CD's internal backup RAM its first case) comes in a minor of
  its own.
- **The card manager** (`Diamond_Cards.md` §12) is **built**: a DianaSh command module first, then a LunaP window in
  Mistress, both over one reader of each card format.

Decided the same day, later: **a menu in Mistress where the player manages memory cards** is a requirement, reachable
from Mistress's own menus, from the in-game overlay menu (the running game's media shown and refused writes, with the
reason said) and in big picture by the controller grammar, and listing every console that declares save media with
each medium and its slots, not only the PlayStation's.

Also decided 2026-10-15, for the Sega CD: whether a new machine starts with a formatted internal backup RAM is deferred
to Nephrite's stage 10, whose BIOS replacement derives the format from the manual. This page supplies no formatted
image of it.

## 1. What was PlayStation-shaped (read 2026-10-15)

The card mode was built for one console, and its code says so in six places [R]:

1. `MemoryCards.SlotKeys` is exactly `Card1`, `Card2`: two slots, whatever the core lists.
2. One extension per console, the first removable file's (`MemoryCards.NamesFor`), so a console with two media of
   different extensions could offer only the first's cards.
3. The names `Card N` and the word "card" in rows that the code offers for any console with a removable file.
4. One flag for one kind: there was no kind for memory inside the console, so the Sega CD's internal backup RAM could be
   only a game's file.
5. The concept documented under Diamond's pages, where a reader of another core would not look.
6. The card manager's reader planned inside `EmuSen.Crystal/Systems/Psx` with no interface for another console's.

Nothing in the core ABI of 1.7 was PlayStation-shaped: `kind` and the create flag are per file and per machine. What a
frontend lacked was everything a slot is besides its file: which medium it holds, which socket it is, what to call it.

## 2. The media to design for

| Console | Medium | Slots | A slot belongs to | Size | Its file system, and where it is documented |
|---|---|---|---|---|---|
| PlayStation | Memory Card | 2, and 3 more per port behind a multitap | the console's card slots | 128 KiB | 15 blocks of 8 KiB behind a directory block; psx-spx "Memory Card Data Format" [R] |
| N64 | Controller Pak | 4, one per controller | a controller port | 32 KiB | a note table and an index of 256-byte pages; Nintendo's public N64 documentation as gathered by N64brew's "Controller Pak" pages [to be read when built] |
| Sega CD | Backup RAM Cartridge | 1 | the console's cartridge slot | 128 KiB (Sega's) | the BIOS's backup RAM format; Sega's *Mega-CD BIOS Manual* [to be read at Nephrite's stage 10] |
| Sega CD | Internal Backup RAM | 1 | the console | 8 KiB | the same format as the cartridge |
| Saturn | Internal memory; Backup RAM cartridge | 1 each | the console; its cartridge slot | 32 KiB; 512 KiB and more | the BIOS's backup library |
| Dreamcast | Visual Memory Unit | 8, two per controller | a controller's socket | 128 KiB | the VMU's FAT-like file system |
| PC Engine CD | Backup RAM | 1 | the console (the CD unit) | 2 KiB | the System Card's directory |
| Neo Geo | Memory Card | 1 | the console's card slot | 2 KiB | the BIOS's card directory |
| GameCube | Memory Card | 2 | the console's card slots | 59 to 2,043 blocks | the card's own FAT and directory |
| PS2 | Memory Card | 2, more behind a multitap | the console's card slots | 8 MiB | a FAT file system with clusters |

They differ in size, extension, number of slots, what a slot belongs to and file system, and agree in everything the
runtime needs [A]: each is a fixed-size block of bytes the console reads and writes, which either belongs to one game
(a cartridge's SRAM), travels with the player (a card, a pak, a VMU, a RAM cartridge), or stays in the console (an
internal memory). Those three are the kinds of §3.1, and everything else is a fact of one medium.

## 3. The model

### 3.1 Kinds

A battery file is of one of three **kinds**, each defined by where the memory is in the world, not by the core:

| Kind | What it is | Modes the runtime offers | A state load, with the medium apart from the game |
|---|---|---|---|
| `game` | memory that belongs with the game: a cartridge's SRAM, EEPROM or flash | none: always the game's | — (the core's own rule) |
| `removable` | a card, pak, VMU or RAM cartridge a player carries between games (since 1.7) | *Per game*, *Shared*, *Named* | leaves the file as it stands (1.7's rule) |
| `console` | memory inside the console, which every game on it shares (since 1.8) | *Per game*, *Shared*, *Named* | leaves the file as it stands (1.8, the same rule) |

The `console` kind is kept on a load for the reason `removable` is (`Diamond_Cards.md` §1): once one file serves
several games, a state of one game that puts back the copy it holds removes every other game's save made since. Its
modes are the removable kind's, because what decides them is the same question, whether the file is the game's alone:
*Per game* is today's behaviour, one `.brm` per game; *Shared* is the console's own, one memory used by every game; and
*Named* is a drawer of consoles, a Japanese and an American unit kept apart, which costs nothing to offer since the
runtime's rule is the same [C].

### 3.2 A medium and its slots

A **medium** is one physical kind of save memory of one console: the PlayStation's memory card, the N64's Controller
Pak. It has an id (`psx.card`, `n64.pak`, `mcd.ramcart`, `mcd.backup`), a kind, an extension and one or more
**slots**. A slot is one battery file the core declares, with:

- **which** file it is, and its **suffix** (since 1.3);
- its **kind** (since 1.7; `console` since 1.8);
- its **medium**, the id above (since 1.8);
- its **label**, the words a frontend shows for the slot: `Slot 1`, `Controller 2`, `Cartridge slot`, `Inside the
  console` (since 1.8);
- its **port**, the controller port it belongs to, absent for a slot of the console itself (since 1.8);
- its **length**, which machine info has given since 1.0 and which a medium's format module knows too (§6.2).

The slots of a medium are ordered as core info lists them, and a slot's **index** is its place among its medium's
entries. Its extension is the last part of its suffix (`.mcr` of `.2.mcr`), and every slot of one medium has the same
one, which the kit checks (§5.4). A core of 1.7 gives no `medium`: the runtime then takes each kind and extension as one
medium, so that Diamond at 1.7 is one medium of two slots, as it was.

### 3.3 The runtime's modes, per medium

Each medium of a console has its own mode, *Per game*, *Shared* or *Named*, and under *Named* a name for each slot:

| Mode | Slot *n* (from 1) of the medium | Default |
|---|---|---|
| **Per game** | `Saves/<console>/<game><suffix>`, the runtime's rule for any battery file | the default for every medium until §9 |
| **Shared** | `<Noun> n`, the same file in every game | the default for a medium whose import exists (§9) |
| **Named** | the name the player chose for slot *n* | — |

**A packaged game's** *Per game* media are in its package instead: `<package>/saves/<package's name><suffix>`, the
battery file's rule there (`EmuSen_GamePackages.md` §3.5, built 2026-10-25 at step 5, §14). The plain
`Saves/<console>/<game><suffix>` is read until the first write copies it in, and is never removed. A game opened with
the package's soft patches applied has its own, `<package>/saves/<package's name>.patched-<key><suffix>`, and reads
nothing before it (decided 2026-10-25, `EmuSen_GamePackages.md` §16). *Shared* and
*Named* files stay in `Saves/<console>/Cards` and `Console`: they are the console's, not a game's. A packaged game's
medium settings (*Per game*, *Shared* or *Named* for each medium, when chosen for the one game) are among its own
settings, which since 2026-10-25 are its package's `settings.json` once it has one (`EmuSen_GamePackages.md` §18.2); a
medium's mode chosen for the console stays the console's.

The **noun** is the medium's word for one of its kind, `Card`, `Pak`, `Cartridge`, `Backup RAM`, from the console's
system pack (§4), and `Card` for a removable medium no pack knows, `Memory` for a console one. So the PlayStation's
shared cards stay `Card 1` and `Card 2`, as 1.7 named them, and the Sega CD's are `Cartridge 1` and `Backup RAM 1`.

**Where the files are**, Galaxia's rule, one folder per kind beside the console's per-game saves:

- a removable medium's shared or named file: `Saves/<console>/Cards/<name><extension>`, as 1.7 put it;
- a console medium's: `Saves/<console>/Console/<name><extension>`.

Different media of one kind in one folder are told apart by extension, as the PlayStation's `.mcr` and a later
multitap's `.mcr` would be the same medium and the Sega CD's `.crm` and `.brm` are not. A name is one file name: no
separator, no leading dot, nothing a path would read otherwise (`MemoryCards.IsName`, unchanged).

**The settings keys**, in `graphics.json` under the console as since 1.7, never passed to the core: the first medium
of each kind takes its kind's plain keys and any further one its id's:

| Medium | Mode key | Slot *n*'s name under *Named* |
|---|---|---|
| a console's first removable medium | `Cards` | `Card<n>` |
| its first console medium | `ConsoleMemory` | `ConsoleMemory<n>` |
| any further medium of a kind | `Media.<id>` | `Media.<id>.<n>` |

The PlayStation's keys are therefore 1.7's, unchanged, and every console known today has at most one medium of each
kind (§2), so the third row is for a console not yet met [C].

**Two slots on one name.** Under *Named*, a name already in an earlier slot of the same medium, or in a slot of
another medium of the console with the same extension, leaves the later slot per game, with words in the log and a
note under its row, as 1.7 did for slot 2.

**The flags a machine is made with.** `EMUSEN_CREATE_SHARED_REMOVABLE` when any removable slot is given a shared or
named file, and `EMUSEN_CREATE_SHARED_CONSOLE` (§5) when any console slot is. Every known console has at most one
medium of each kind, so the flag of a kind says exactly whether its medium is apart. Where one kind's media ever
differed, a slot left per game beside one shared would be kept on a load as well: the compromise 1.7 already makes for
a named slot 2 that falls back to the game's card, harmless in that it loses nothing, and the per-file form of §5.1's
fourth row is the minor that would remove it.

### 3.4 What a state load does, by kind and mode

| Kind | Per game | Shared or Named |
|---|---|---|
| `game` | the core's own rule, unchanged | (no such mode) |
| `removable` | the core's own rule: Diamond restores the game's card, as before | kept as it stands; a medium that differs from the state's copy is the core's to present as changed (Diamond's flag byte) |
| `console` | the core's own rule: Nephrite restores the game's `.brm`, as before | kept as it stands |

A state made apart and loaded per game leaves the game's own file alone where the core marks its states so
(`Diamond_Cards.md` §8's `SHRD`); a core that does not mark them may restore a shared medium's copy into the game's
own file under *Per game*, which loses nothing shared and is that core's choice to refine.

### 3.5 The lock

Two programs of the project running games of one console on one shared card would each write the card from their own
copy, the last writer winning (`Diamond_Cards.md` §9.3). The lock is **advisory and per file**: `MediumLock` in
Galaxia, which owns every save path, holds `<file>.lock` beside the medium's file, opened exclusively for as long as
it is held, so that the operating system releases it if the program ends without releasing it. On Linux and macOS
.NET takes `flock` for an exclusive open, on Windows a sharing mode; either is per open file, so a second open fails in
the same program as in another. A lock held in the same program also names its holder (the running game's title), so
that a refusal can say who holds the card.

- **Within one program the machines share a lock** [C]: a program runs one game at a time, and Mistress makes the next
  game's machine under its launch screen before it leaves the last (`Diamond_Players.md` §12.8.3), so an exclusive lock
  per machine would send the next game's shared card per game at every change of game. The program's own machines
  therefore take the lock shared, the latest naming the holder, while the card manager takes it alone; another
  program is refused either way.

- **A running machine holds the lock** on every removable and console file it was given, from its creation to its
  release, in every mode.
- **A shared or named file already locked** is not given to a second machine: that slot runs per game, with words in
  the log and a notice the frontend shows, which is `Diamond_Cards.md` §13 Q3's recommendation.
- **A per-game file already locked** (one game run by two programs at once) is still given, as before 1.8, since
  refusing it would refuse the game; the lock serves the card manager's refusal.
- **The card manager writes nothing locked**, and says by whom it is held where it knows.

*Since 2026-10-17* the lock has a Rust half in Galaxia's crate (`emusen-galaxia`'s `medium_lock`), the same `flock`
on the same file with the same words in it and the same table of shared handles, which `MediumLock` forwards to under
Galaxia's own switch, so a program's games and its card manager hold one table between them; a lock either half takes
refuses the other as another program's would (`EmuSen_Backend.md` §15.2).

## 4. Who owns which fact

| Fact | Owner | Surface |
|---|---|---|
| Which battery file is which slot of which medium; its kind, label and port | **the core** | core info's `battery[]` (1.3, 1.7, 1.8) |
| What a state load does to a file kept apart | **the core** | the create flags (1.7, 1.8) |
| A medium's title and noun, its capacity, its on-medium file system | **the console's system pack**, by medium id | `SaveMediumEntry`, `ISaveMediumFormat` (§6.2) |
| Each medium's mode and which file is in each slot | **the runtime**, a setting of the console | `graphics.json`, `MemoryCards` |
| Where every save file lives | **Galaxia** | `SaveLibrary` |
| Whether a file is in use | **Galaxia's lock**, held by the runtime | `MediumLock` |
| What a player can do to a card | **the card manager** over the format modules | `SaveMediaManager`; `memcard`; the Memory Cards window |

The medium id is the join between the core's declaration and the system pack's knowledge, as a controller id
(`psx.dualshock`) joins a core's port to the bindings screen. A title or noun in the core's descriptor would be a second
owner of a fact every core of the console shares; a slot's label is the core's because which file is slot 2 is the
core's numbering [A].

## 5. The core ABI's minor 1.8

### 5.1 The forms weighed

| Form | For | Against |
|---|---|---|
| 1. A `media` table per system, `{ id, kind, label }`, with battery entries naming it | one entry per medium | `kind` is on the battery entry since 1.7 and cannot leave it, so the kind would have two owners; a new array for three strings |
| 2. **`medium`, `label` and `port` on each battery entry of core info**, and `"console"` as a value of `kind` | each optional string beside the entry's `kind`; absent means what 1.7 meant | the medium's id is repeated on each of its slots, as the extension is in each suffix |
| 3. The same in machine info too | the machine's own list complete | machine info already names which slots this machine has; which slot a file number is never changes between machines, so it is core info's fact alone |
| 4. A per-file "kept apart" bit, in `emusen_file` or a mask | each file's mode independent | `emusen_file` would grow, every core's reading of files with it; no console needs it (§3.3) |
| 5. **A second create flag, `EMUSEN_CREATE_SHARED_CONSOLE`** | the kinds independent: the Sega CD's cartridge shared while its internal memory stays per game | one more bit |
| 6. `SHARED_REMOVABLE` read as covering `console` files | no new bit | the two kinds could not differ in mode, which the Sega CD's two media can; and 1.7's text, which says "removable", would change meaning |

**Chosen: 2 and 5** [C], the smallest forms that carry the facts. No export, struct, status or capability bit.

### 5.2 The surface

```c
/* @since 1.8 (section 6.28): the host keeps every battery file machine info lists with kind "console" apart from the
 * game, shared between games. A state_load of either kind then leaves each such file's contents as they stand and
 * raises no BATTERY_CHANGED of its own. Set only for a core whose info lists a console file; a core that lists one
 * honours it. */
#define EMUSEN_CREATE_SHARED_CONSOLE 4u  /* emusen_create_params.flags */
```

- **`kind`**, in core info's `systems[].battery[]` and machine info's `battery[]`, gains `"console"`: memory inside the
  console, shared by every game. A new value of a descriptor's enumerated string (§4.4 of `EmuSen_CoreAPI.md`).
- **`medium`** (string, `[a-z0-9.]+`), **`label`** (string) and **`port`** (integer, from 0), each optional, on core
  info's `systems[].battery[]` entries only. Absent, a host takes the kind and extension as the medium, the medium's
  title and the slot's number as its label, and the console as its owner.
- **`emusen-native`**: `BatteryKind::Console`; `Battery` gains `medium`, `label` and `port`, written only when set, so
  that a core that sets none writes what it wrote at 1.7, byte for byte; `Battery::console(which, suffix)` beside
  `game` and `removable`, and `.medium(..)`, `.label(..)`, `.port(..)` to set the three; `Create.shared_console`, read
  from the flag by `core_exports!`'s create.
- **The rule a core keeps** under `SHARED_CONSOLE` is 1.7's for removable files, word for word with the kind changed:
  after a successful `state_load`, `battery(which)` of each console file returns the bytes it returned before the load,
  and the load raises no `BATTERY_CHANGED` of its own.

### 5.3 Skew, both ways

- **A host of 1.7 with a core of 1.8.** It reads no `medium`, `label` or `port` (unknown fields are skipped). It reads a
  `"console"` kind as not removable, as the reference host of 1.7 does (`Str(b, "kind") == "removable"`), so it offers
  that file no mode and never sets `SHARED_CONSOLE`: the core's own rule stands, a per-game rollback included, which is
  the behaviour of 1.6. A host that validates descriptors against 1.7's schema would find `"console"` outside its enum;
  the schema is the minor's, and an enumerated string is open (`EmuSen_CoreAPI.md` §4.2), so such a host must read an
  unknown kind as `"game"`, and the 1.8 page says so.
- **A host of 1.8 with a core of 1.7.** It finds no `medium`, `label`, `port` or `"console"`, and groups each kind and
  extension as one medium (§3.2), titled by the system pack where it knows the console's medium and by its extension
  where not, so Diamond at 1.7 runs exactly as it did; and it never sets `SHARED_CONSOLE` on a core that lists no
  console file, the pairing that made 1.7's flag safe.
- **A core of 1.8 that lists a console file** is checked by a kit of 1.8 or later, whose C14 accepts the value (the kit
  of 1.7 fails it, as the kit of 1.5 failed 1.6's driveless machines; `EmuSen_CoreAPI_Extensions.md` §14.4's rule).

### 5.4 The kit

- **C14** accepts `"console"`; checks that a `medium` is `[a-z0-9.]+`, a `port` is from 0 and one machine info lists, a
  `label` is not empty, and that the entries of one medium agree in kind and extension; and, as since 1.7, that a
  machine's file has core info's kind.
- **C19** keeps removable files as since 1.7 and keeps console files the same way under `SHARED_CONSOLE`: a machine made
  with both flags and each listed file filled with one pattern takes a state; a second, its files holding another
  pattern, loads it; every removable and console file must read as before the load, its changed flag not raised by it.
  A machine with neither kind passes with that said.
- **The test core** lists a console file 2 on an image whose payload begins `CEh`, and its seeded fault
  `FAULT_CONSOLE_ROLLED_BACK` (1 << 21), a load that blanks it, fails C19 alone; a second seeded fault,
  `FAULT_MEDIUM_DISAGREES` (1 << 22), gives two entries of one medium different extensions and fails C14 alone.

## 6. The card manager

### 6.1 What it does

Over any medium of any console: **list** the cards a player has (the drawer's and each game's own) and, where a
format module reads the medium, the saves on one with their titles, sizes and free space; **copy** a save onto another
card; **move** one (a copy, read back, then the delete); **delete** one as the console's own manager does; **import**
every save of chosen cards onto one card, refused as a whole when the card's capacity would be exceeded, with words
naming the saves and the blocks each needs; make a **new** card under a name; **duplicate** a whole card; **rename**
one. It never writes a file a running machine holds (§3.5), and it reads a game's own card for an import without
writing it.

*Since 2026-10-17* the manager is `emusen-savemedia` in Rust, behind `EMUSEN_SERVICES_NATIVE`, with the C# kept as
`SaveMediaManager.Managed`; which medium a file is, by its extension, still crosses from the C# (`EmuSen_Backend.md`
§14.2).

Three operations need no format module, since they treat a medium as bytes: a new card (an absent file, which the game
or the kernel formats, as a new physical card), duplicating a card, and importing a game's whole card onto a drawer
name that is not in use. Every other needs the medium's module, and a medium with none offers only these three, with
that said.

### 6.2 The format modules

A **format module** reads and writes one medium's file system, as pure functions, bytes in and bytes out, with no file
I/O, no clock and no state, so that both clients and a test drive it the same way:

```csharp
// One medium's file system; every method pure, every image exactly Length bytes.
public interface ISaveMediumFormat
{
    string Medium { get; }                       // "psx.card"
    int Length { get; }                          // 131072
    int Capacity { get; }                        // 15, in Unit
    string Unit { get; }                         // "block"
    MediumListing Read(ReadOnlySpan<byte> image);                  // the saves and the free units, or why it cannot be read
    MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to);   // the new 'to', or the refusal
    MediumEdit Delete(ReadOnlySpan<byte> image, string save);      // the new image, the save undeletable where the medium allows
    byte[]? Format();                            // a formatted blank, or null where the medium's format is not known
}

public sealed record SaveEntry(string Id, string Title, int Units, string? Game, bool Damaged);
public sealed record MediumListing(bool Formatted, IReadOnlyList<SaveEntry> Saves, int FreeUnits, string? Problem);
public sealed record MediumEdit(byte[]? Image, string? Refusal);
```

A save's `Id` is the medium's own name for it (the PlayStation's directory file name, `BASLUS-00067DRACULA`), which
is what a copy carries and what a clash is judged by. `Title` is what the console's manager shows (the Shift-JIS title
frame), `Game` the serial where the name gives one. The import is the generic tool's, not the module's: it lists every
save of every source, adds their units, refuses when they exceed the target's free units or when two different saves
share an id (an identical save already there is skipped as present), and otherwise folds `Copy` over a copy of the
target in memory, writing once.

*Since 2026-10-17 (`EmuSen_Backend.md` §13, its step B3) each module's file system is a crate with no standard library
in `EmuSen/Cores/Shared/` (`Hardware/Shared/EmuSen_MediumFormats.md`), its words a family module of `emusen-systems`,
and the three C# classes forward to them through the platform library unless `EMUSEN_SYSTEMS_NATIVE=0`; each class's
`Managed` twin is the C# alone. A medium's title and noun are the systems table's `media`, its extension and module
still the pack's literal.*

A medium's **entry** in its console's system pack names it whether or not a module reads it:

```csharp
public sealed record SaveMediumEntry(string Id, string Title, string Noun, string Extension, ISaveMediumFormat? Format = null);
```

**Where the modules live** [C], as `Diamond_Cards.md` §12.2 argued for the PlayStation's: in each console's system
pack in `EmuSen.Crystal/Systems/<console>`, beside its cheat codecs and its disc recogniser, as that is where the
runtime keeps a console's knowledge above the ABI and both clients reach it (Crystal references Galaxia alone). The
format is not the core's: it is a convention of the console's kernel or BIOS, and a host reaching into a core for it
is what the surfaces forbid. `SystemPack` gains `Media`, the console's `SaveMediumEntry` list, and
`SaveMediumFormats.For(id)` and `.ForExtension(ext)` find them across the packs.

**The modules, by medium.**

- **`psx.card`, built** (§11): psx-spx's layout [R]. The directory block's header frame (`MC`, checksum 0Eh), fifteen
  directory frames (allocation state 51h/52h/53h in use, A0h free, A1h–A3h deleted; the file size and the name in the
  first; the next block's number; the XOR checksum of bytes 00h–7Eh in byte 7Fh), the broken-sector list and its
  replacements, and the title frame (`SC`, the icon's frame count, the Shift-JIS title, the palette) with the icon's
  16×16 4-bit frames. `Delete` marks a file's frames A1h–A3h and keeps its blocks, as the BIOS does, so that an undelete
  stays possible; `Copy` takes the lowest free frames and writes the chain with its checksums; `Format` writes the
  directory block as psx-spx lists a formatted card's.
- **`n64.pak`, built 2026-10-16** (§11.8), as designed here: 32 KiB of 256-byte pages; the module will read its ID area, its index table and its note
  table from N64brew's public "Controller Pak" pages, its unit the page (123 free on a formatted pak), a note's id its
  game code, publisher code and name. Written when MarsRT joins the ABI (§7.2).
- **`mcd.ramcart` and `mcd.backup`, designed**: one module for both, the BIOS's backup RAM directory, read from Sega's
  *Mega-CD BIOS Manual* at Nephrite's stage 10, whose BIOS replacement must write the same format; until then the two
  media have entries with no `Format`, so they offer §6.1's three whole-medium operations.
  *Built 2026-10-16* (`SegaCdBramFormat`, §11.7): one module for both, from `Nephrite_Bram.md`, the format observed on
  the player's BIOS by the two-person method of `Nephrite_OpenBios.md` §5 Q2, which the open BIOS writes too; its unit
  the 64-byte block (125 free on a formatted internal RAM, 2,045 on Sega's cartridge).

An entry's `Title` and `Noun` are written as a heading writes them ("Internal Backup RAM", "Backup RAM", "Card"). Where
a sentence uses one, `MediumWords.InText` gives it in running text: every word in lower case except an acronym, so
"one backup RAM in each slot" and "a formatted backup RAM cartridge". Lower-casing the whole name, as every caller did
before 2026-10-16, wrote "backup ram" in the Genesis's and the Sega CD's settings and in `memcard`'s replies.

### 6.3 The generic tool

`SaveMediaManager` in `EmuSen.Crystal/SaveMedia`, over Galaxia's paths, `AtomicFile` and `MediumLock`: each operation
takes a **card**, a file of a medium (a drawer card by console and name, or any card file by its path), reads it once,
asks the module, and writes the result atomically, or refuses with words. A card's medium is found by its extension
through `SaveMediumFormats.ForExtension`, so the tool needs no core: it runs where DianaSh runs, without an emulator.
The runtime's view (which card is in which slot of which running game) is the frontend's, layered over it.

### 6.4 The DianaSh command module: `memcard`

The module `emusen.memcard`, one command, `memcard`, its page `Lib/dianash/emusen.memcard/man/memcard.1`:

```
memcard ls [<card>]                every card under Saves by console, or the saves on one card
memcard cp <card>:<save> <card>    copy a save onto another card of its medium
memcard mv <card>:<save> <card>    copy a save, read it back, then delete the first
memcard rm <card>:<save>           delete a save as the console's own manager does
memcard import <card> <from>...    every save of <from> onto <card>, all or none
memcard new <card>                 a new card, formatted where its medium allows
memcard dup <card> <new>           a whole card copied to a new name
memcard rename <card> <new>        a card renamed
```

A `<card>` is a path in the sandbox (`Saves/PlayStation/Cards/Card 1.mcr`, `Saves/PlayStation/Amber Saga.mcr`), the
shell's own way of naming a file, so that `ls`, `cp` and `xxd` see the same objects; `<save>` is a save's id or its
position in `memcard ls`. Writes refuse a locked card with its holder, an unknown medium with the extensions known, and
an import past capacity with the saves and units it needed. The module is in the shell's default set, since it needs
no running game, and its outputs join the golden corpus (`EmuSen_DianaSh.md` §13.4).

### 6.5 The Mistress window: Memory Cards

A `ToolWindow` of LunaP, framed as a menu in big picture, controller-navigable, audited for fit at 1280×800 and
1920×1200 by WiseMan's fit audit with nothing cut off or overlapping.

**What it shows.** One tab per console whose engine declares save media (removable or console slots), in release
order. On a tab, each medium in turn: its title, its mode and which card is in each slot (`Shared: Card 1 in slot 1,
Card 2 in slot 2`), and a card chooser holding the drawer's cards and then each game's own; under the chosen card, its
saves with title, units and game, and its free space, or, where no module reads the medium, its size and the three
whole-medium operations. Actions: a save is chosen by pressing it, then *Copy Save* or *Move Save* to the card chosen
in *Copy or move to*, a list of the medium's other cards, or *Delete Save*; for the card, *New Card* under a typed name
(the on-screen keyboard from a pad), prefilled with the first free `<Noun> n`, and *Import Every Game's Saves* onto the
chosen drawer card. Duplicating and renaming a card are `memcard`'s (§6.4) and not the window's, where a pad would
reach them badly and a player seldom needs them [C]. What an action did, or why it did nothing, is said under the
tabs.

**Entry points.**

- **Mistress's menus**: *Settings → Memory Cards…*, beside *Graphics Settings…*, *Cheat Database…* and *Active Cheats…*,
  where a player finds the other management windows.
- **The pad menu**: *Settings → Memory Cards* in the main menu, and *Game Settings → Memory Cards* in a game's menu,
  which opens on the running console's tab. In a game, the media the running machine holds are shown and their write
  actions refused, the window saying why: "In use by *Amber Saga*, which is running: its cards can be looked at, and
  changed when the game is closed." The refusal is the lock's (§3.5), so the window and `memcard` agree.
- **Big picture**: the same pad menu entries, which big picture draws as its own panel, and the window framed as a
  sheet, so it is reached and used by the controller grammar alone (A choose, B back, shoulder buttons for tabs).

### 6.6 Importing a game's cards

*Import Every Game's Saves* on a drawer card, and `memcard import` with any cards named, copy every save on the console's
per-game cards of the same medium onto the card, all or none. The per-game cards are read and never written, so *Per game* chosen
again finds them as they were. The refusal past capacity is the medium's own: the PlayStation's 15 blocks, from its
module.

## 7. Each medium, and how it joins

### 7.1 The PlayStation, built

Diamond's two cards become `Battery::removable(..).medium("psx.card").label("Slot n")`; nothing else of the core
changes. A multitap's further slots, when Diamond offers them as files, are further slots of the same medium, labelled
`Port 1, slot B` and so on, and under *Shared* named `Card 3` onward.

### 7.2 The N64's Controller Pak: how MarsRT joins

*Built 2026-10-16 (§11.8, `Mars_Native.md` §7.7). As built, the cartridge's save keeps the name C# Mars and MarsRT
have always written, `<game>.srm`, whatever the chip; the suffixes below were this section's guess.* MarsRT is not on
the core ABI v1 yet, and this page does not move it. When it moves, it lists its cartridge's save
(`.eep`, `.sra`, `.fla`) as `game`, and four paks as `removable`, `medium: "n64.pak"`, `port` 0 to 3, labels
`Controller 1` to `Controller 4`, suffixes `.mpk` for port 0, which keeps the name C# Mars and MarsRT write today
(`Mars_Save.md` §7), and `.2.mpk` to `.4.mpk`; it honours `SHARED_REMOVABLE` by leaving each pak's bytes as they stand
on a load (the pak is in its state today, `src/memory/controller.rs`), and presents a pak that differs as one pulled and
pushed in, which the pak's status byte to the joybus reports (to be read in the public N64 documentation when built).
C# Mars stays as it is: it is the oracle of the port and is not offered a mode.

### 7.3 The Sega CD, built

Nephrite's two files gain kinds and labels: the internal backup RAM, file 0 `.brm`, `console`, medium `mcd.backup`,
label `Inside the console`; the RAM cartridge, file 10 `.crm`, `removable`, medium `mcd.ramcart`, label `Cartridge
slot`. Under the flags, a load leaves each as it stands. Whether the BIOS sees a changed cartridge needs nothing of the
core: the BIOS's backup RAM calls read the directory from the RAM itself (to be confirmed against the manual at stage
10 [A]). *Confirmed 2026-10-16*: every call of the open BIOS reads the footer and the directory from the RAM
at each call (`Nephrite_OpenBios.md` §14). The Genesis's cartridge SRAM (`.srm`) stays `game`.

### 7.4 Later

Each joins by its core's battery declarations and its system pack's entry, and a format module when its file system is
read: the Saturn's internal memory (`console`) and backup cartridge (`removable`), the Dreamcast's VMUs (`removable`,
two per port), the PC Engine CD's backup RAM (`console`), the Neo Geo's card, the GameCube's and the PS2's cards.
Nothing of the runtime, Galaxia, the manager or the window changes for any of them.

## 8. The migration

| 1.7's piece | What it becomes |
|---|---|
| `kind` `game`/`removable` on battery entries | unchanged; `console` added |
| Diamond's `Battery::removable(0, ".mcr")`, `(4, ".2.mcr")` | the same, with `medium` and `label`; core info gains two fields per card, machine info unchanged |
| `MemoryCards` (the PlayStation's two slots) | the card mode for every medium: `MemoryCards.MediaFor(console)`, a choice per medium with N slots; its 1.7 members kept, meaning the console's first removable medium |
| `CardChoice(Mode, Slot1, Slot2)` | a medium's choice, `Mode` and N names; `Slot1`, `Slot2` kept as its first two |
| keys `Cards`, `Card1`, `Card2` | the first removable medium's keys (§3.3), read and written as before |
| `SaveLibrary.CardsFolder`, `CardPathFor`, `CardNames` | unchanged, the removable kind's; `MediumPathFor(console, kind, name, ext)` and `MediumNames` beside them |
| `CoreEngine.Cards` | `CoreEngine.Media`, a choice per medium by its id; `CoreEngine.ReleaseMedia` lets go of the locks when a frontend leaves the game |
| the rows of Graphics Settings | one group per medium: its mode, and under *Named* a row per slot labelled by the core; the PlayStation's rows keep their keys and words |
| `Diamond_Cards.md` | the PlayStation's part only: the hazard as Diamond met it, its flag byte, `SHRD`, its measurements; the framework is this page |

**Unchanged byte for byte**: a per-game PlayStation state (its digests, `Diamond_Cards.md` §10.3), its per-game files,
and machine info. Core info gains `medium` and `label` on Diamond's two entries, which the registration golden then
records (`EmuSen_CoreAPI.md` §25). `DiamondCardsTests` pass unedited until the default changes (§9), and each edit
there is argued.

## 9. The default, and the import offered once

When the import exists (§6.6), **a removable medium whose format module reads it defaults to *Shared***, the
PlayStation's card first; a medium with no module, and every console medium, stays *Per game* by default until its
module exists, since without a module the import could only copy one game's whole card, which would hide every other
game's [C]. A mode the player chose is kept: only a console with nothing stored changes. *Since 2026-10-16* (§10 Q2) a console
medium whose module exists defaults to *Shared* as well, the Sega CD's internal backup RAM first, and a new machine's
console memory with no file yet is given the module's formatted blank (§11.7).

**The import offered once.** The first time a game of such a console is started under the default *Shared* while
per-game cards of that medium exist and the shared card does not, Mistress asks, after the firmware question and
before the game starts: "Your PlayStation games each kept their own memory card. Copy their saves onto the shared one,
Card 1, so that every game sees them? Each game's own stays as it is: Memory Cards can copy from them later, and Per
game in Graphics Settings uses them again." with **Copy Saves** and **Not Now**. Copy runs §6.6's import onto `Card 1`;
past capacity it is refused as a whole, its first line is shown, and the window opens to choose. Either way the offer is
recorded (the medium's mode key followed by `Offered`, `CardsOffered` for the PlayStation, under the console in
`graphics.json`) and not made again; the way back stays visible: *Import Every Game's Saves* in the Memory Cards
window, `memcard import`, and *Per game* in Graphics Settings, under which each game finds its own card as it was.

## 10. Questions for the tester

*Decided 2026-10-15 by the tester: Q1 to Q3, each as recommended.*

1. **Q1, the window's name.** Built as **Memory Cards**, the word players use, though it also manages paks and backup
   RAM, each named inside by its own title. *Recommended:* as built; the alternative, *Save Media*, is exact but not a
   player's word.
2. **Q2, console memory's default.** Built: *Per game*, as today, until the Sega CD's format module exists. On the
   hardware the internal memory is shared, so *Shared* is the faithful default. *Recommended:* make *Shared* its
   default with stage 10's module and its import, as §9 does for the PlayStation.
   *Built 2026-10-16* with `SegaCdBramFormat` (§11.7).
3. **Q3, the N64 when MarsRT joins.** *Recommended:* four paks, one per port, `.mpk` keeping port 1's present name
   (§7.2); and *Shared* by default once the pak's module and import exist.

## 11. Build record

*Each step was committed when proven, on 2026-10-15.* Marks as the plan's.

### 11.1 The core ABI's minor 1.8

[M] The header's `EMUSEN_CORE_ABI_MINOR` is 8 with `EMUSEN_CREATE_SHARED_CONSOLE` (4u); the schemas accept `"console"`
and core info's `medium` (`^[a-z0-9.]+$`), `label` and `port`; the baseline gains four lines tagged `@1.8` and moves its
minor, and `emusen-core-abi-check --check --against` WiseMan's baseline reports 433 facts agreeing on four triples.
`emusen-native` writes a slot's three fields only when set (`a_slots_medium_label_and_port_are_written_only_where_set`,
with a medium outside the pattern refused by the schema), so every core of 1.7 writes the same bytes; 75 tests pass.
The kit's 30 tests pass, among them C19 on the test core's console memory (`CEh` payload) and the two new seeded faults
each failing its case alone. The C# host reads a kind it does not know as the game's
(`Core_infos_battery_slots_are_read_with_kind_medium_and_label`), and `CoreAbiTests` hold both flags to the baseline.

### 11.2 The runtime

[M] `MemoryCards` groups a system's slots into media (`MediaOf`), reads each medium's choice with 1.7's keys for the first
removable medium (`DiamondCardsTests`, seven, unedited at this step), places each slot at a load (`Place`), and
`CoreEngine` makes the machine with each kind's flag and holds the locks. `SaveMediaTests`: the test core's two media
and their keys, a core of 1.7 grouped by kind and extension with a second removable medium on `Media.<id>` keys, the
console's memory one file for every game when shared and each game's own when not, and the lock: held for a machine's
life, shared with a second machine of the same program, refused to the card manager and, through `flock -n` from
another process, to another program, released by `ReleaseMedia`, its file gone with it, and a game started while the
manager holds its card run on its own card with the notice. Diamond's 173 unit, 81 program and the rest of its tests
pass with its slots declared.

### 11.3 Nephrite

[M] `Nephrite_Native.md` §61: the crate's 172 unit tests; `NephriteMediaTests` through the host. The kit on Nephrite's
Genesis sample: the core's cases and 64 of 64 images, 87.9 s, at core ABI 1.8.

### 11.4 The card manager

[M] `SaveMediaManagerTests` (seven): a formatted card is psx-spx's directory block; a card's saves listed with their
blocks, names, Shift-JIS titles and serials, a broken chain listed damaged; a copy into the lowest free blocks with its
chain and checksums, refused onto a card that holds the name or lacks the blocks; a delete marking A1h–A3h and keeping
the blocks; the import refused whole at 15 blocks needed against 14 free, naming each save and its blocks, then done
with one save already present left as it was and the games' cards byte for byte unchanged; a held card refused with
its holder's name, and a move that deletes only after the copy reads back; new, duplicate and rename never over a file,
and a medium with no module only named. `memcard.golden` records four cases over every usage line and every failure the
command words; `help.golden` and `man.golden` were recorded again for its eight usage lines and its page, no other line
changing. The Memory Cards window passes the fit audit with saves listed and with its card held, at both sizes, and
`PadMenuTests` open it from the main menu's Settings and a game's Game Settings.

### 11.5 The default

[M] `MemoryCardsOfferTests`: the PlayStation's card defaults to *Shared* and the Sega CD's media, read by no module, to
*Per game*; accepted, the offer copies both games' saves onto `Card 1`, leaves each game's card unchanged, is recorded and
is not made again; declined or needless (no game's card, or a mode the player chose) it copies nothing and asks once at
most. Two tests were edited for the new default, each argued: `DiamondCardsTests`' rows assert the default `shared`,
and its per-game case now chooses *Per game* as a player would; `DiamondPlayersTests`' card round trip, whose subject is
the per-game file, chooses *Per game*, and the class now redirects the configuration folder, which it read from the
player's before.

### 11.7 The Sega CD's module, and console memory Shared (2026-10-16)

[M] `SegaCdBramFormat` (`EmuSen.Crystal/Systems/Genesis`) is the `Format` of both Sega CD entries. `SegaCdBramFormatTests`
check it against `Nephrite_Bram.md`: the block code's four vectors, one symbol error a codeword corrected, a format's
footer, Lunar's four files placed and counted as the BIOS writes them, §7.3's delete. `MemoryCards.DefaultMode` now
gives *Shared* to a console medium with a module; `MemoryCardsOfferTests` asserts it for the Sega CD's two media.
`MediaPlacement.Fresh` names each console slot's module and `CoreEngine` gives a slot with no file the module's
`Format()`; `NephriteMediaTests.A_new_machines_backup_ram_is_formatted` checks it per game and shared. The core is
unchanged: with no file it starts from zeros, as the kit and the digests expect.

### 11.6 The PlayStation, unchanged per game

[M] A machine made per game is made as before: no flag, no `SHRD` block, so its states and their digests are 1.7's
(`Diamond_Cards.md` §10.3's three, reproduced by Diamond's unedited tests). The kit on the sample's nine discs at 1,800
frames passes the core's cases and 9 of 9 images, C19 among them, in 275.3 s at core ABI 1.8.

### 11.8 The N64's Controller Paks (2026-10-16)

**The core.** MarsRT, now on the core ABI v1 (`Mars_Native.md` §7), declares file 0 (`.srm`) the cartridge's, `game`,
and files 1 to 4 the paks of ports 1 to 4, `removable`, medium `n64.pak`, labels `Controller 1` to `Controller 4`,
ports 0 to 3, suffixes `.mpk` to `.4.mpk`. Per game, the first port holds a pak always and the others when their file
exists, the machine MarsRT always made; with the paks apart, all four are in. Apart, a load leaves each pak as it
stood, and a port whose pak differs from the state's answers the joybus's next status as a newly inserted accessory
(3), as N64brew's *Joybus Protocol* page describes the status byte. `Mars_Native.md` §7.7 has the rules and tests.

**The format module**, `EmuSen.Crystal/Systems/N64/N64PakFormat.cs`, from N64brew's *Controller Pak/Filesystem* page
[R]: the ID page's four copies of the 32-byte ID block (at 0x20, 0x60, 0x80 and 0xC0; two checksums, the sum of its
first fourteen big-endian words and 0xFFF2 less it; bit 0 of byte 0x19 a controller pak); the index of pages 1 and its
backup 2 (two big-endian bytes a page, 1 the last of a note, 3 free, 5 and up the next; byte 1 the low byte of the sum
of bytes 2 onward); the note table of pages 3 and 4, sixteen 32-byte entries (game code, publisher code, start page,
status, extension, name in the pak's own characters, which the module maps to text as the page lists them). Its unit
is the page, 123 free on a formatted pak; a note's id is its game code, publisher code, name and extension. `Format`
writes the blank pak the page describes, which is byte for byte the pak MarsRT formats where a port has no file.

**Where it is named.** The N64 has no system pack yet: its facts are its catalog core's while C# Mars runs it. Its
medium is listed beside the packs' in `SaveMediumFormats` until it has one, and `MemoryCards` takes a catalog
console's media from its v1 engine's info (`CoreCatalog.EngineSystem`), as it takes a discovered console's from its
own; the PlayStation's and the Sega CD's answers are unchanged. *Since 2026-10-17 the N64 has a system pack,
`N64System`, which names the Controller Pak; the list beside the packs is gone (`EmuSen_Backend.md` §12).*

**The default.** With the module and the import, the N64's paks are *Shared* by default (§9), and Mistress offers the
import once before an N64 game starts where per-game `.mpk` files exist and `Pak 1` does not; nothing of that was
written for the N64, since every piece is the generic one.

## 12. Reading log

Read for this page: psx-spx `stable` `6d7d1bc`, "Memory Card Data Format" and "Memory Card Images"; the project's
own pages and code (`Diamond_Cards.md`; `EmuSen_CoreAPI.md` §4, §5.5, §6.3, §6.4, §6.11, §6.26, §6.27, §12, §17;
`EmuSen_CoreAPI_Extensions.md` §9; `EmuSen_DianaSh.md` §13.4–§13.5; `EmuSen_Settings_Reference.md` §4.114; Nephrite's
`v1.rs`, `state.rs` and `mcd/`; MarsRT's `src/memory/controller.rs` and C# Mars's `MarsCore.cs`). N64brew's Controller
Pak pages and Sega's *Mega-CD BIOS Manual* are named for the modules they will be read for and were not read here. No
emulator's source, no BIOS and no game was read; the ROM library and the player's saves were not opened.
