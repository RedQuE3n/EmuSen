# EmuSen — what the core ABI and the runtime gain for disc consoles and their controllers

*Written 2026-10-10. A proposal, not an implementation: nothing in it is built, and nothing was changed or run to
write it. It evaluates, gap by gap, the framework changes that `Diamond_Plan.md` §10 and `Nephrite_Plan.md` §8.3 found
by planning the PlayStation core and the Sega CD, and asks the tester to choose before any is built. Every claim about
the code is cited to a file, read on `WiseMan` at `88bb936b`.*

Every claim carries one of four marks, as the core plans do:

- **[M]** *measured from the code*: read in the file and at the line named, on 2026-10-10. Nothing was executed; a
  count is a `grep` over the tree, said so where it is one.
- **[R]** *read* from a source, named, on 2026-10-10 unless another date is given.
- **[A]** *argued*: reasoning from the code and the documents, with no measurement behind it.
- **[C]** *a choice*: a recommendation awaiting the tester (§12), or a rule this page proposes.
- **P1–P6** are *predictions*, numbered in §11 so that the step which builds each item can retire them.

Companion docs: `EmuSen_CoreAPI.md` (the core ABI v1: §4 its stability policy, §6 the surface, §8 the runtime, §12
and §28 the conformance kit), `EmuSen_Surfaces.md` §6 (the stability rules A to R that each component below keeps),
`EmuSen_DianaSh.md` §16 Q1 and §17.2 (the runtime as a component of its own, Crystal), `EmuSen_Input.md` §7 and §8
(the RetroPad, `PadControl`, ports), `EmuSen_Cheats.md` §7 (tests in a cheat), `Diamond_Plan.md` (§1.2, §3.4, §4,
§5.11, §9.2, §10, §11 Q9–Q12) and `Nephrite_Plan.md` (§4, §5.8, §8.3, §9 Q5).

---

## 0. The result in brief

Of the nine gaps, **five need the core ABI**, each through a kind of addition that v1's policy already permits (§1);
**four need no ABI change at all**, and one of those, conditional cheats, turns out to be largely built already.

| § | Gap | Needed by | Recommended form | Surface change |
|---|---|---|---|---|
| 2 | **Image sets**: a sheet's track files; `.m3u` | Diamond st. 7, Nephrite st. 8, Zoisite, Kunzite, Tiger's Eye | an **extension** (a header constant) plus the shared disc crate's set module, reached by the runtime through the platform library | `EMUSEN_FILE_MEDIA` (65,536): file `which` *n* above it is the sheet's *n*th file; `.m3u` never reaches a core |
| 3 | **Disc change** (lid, swap, sets) | Diamond st. 11, the Sega CD, every later CD core | an **extension**: two optional exports behind a new bit | `MEDIA` (bit 18): `media_lid`, `media_insert` with `emusen_media`; machine info's `drives`; three status codes |
| 4 | **Rumble** and other pad outputs | Diamond st. 6; MarsRT's Rumble Pak; MercuryRT's MBC5 carts; later GameCube, Dreamcast, PS2 | an **extension**: one event kind and a descriptor field, no export | `EMUSEN_EVENT_OUTPUT` (7); machine info's `ports[].outputs` |
| 5 | **The Analog button** | Diamond st. 6; later the PS2, the Saturn's 3D pad, every keypad console | an **extension** of a descriptor field v1.0 already allows, built in the frontends | button entries gain `name`; `PadButton` unchanged |
| 6 | **Conditional cheats** | Diamond st. 11; already the N64 | **inside the runtime**: two appended `CheatWriteType`s and a system pack | none at the ABI |
| 7 | **The kit's memory per image** | Diamond's corpus form; any large medium | **inside the kit** | none at the ABI |
| 8 | **Routing a file by its content** | Diamond and the Sega CD from Diamond's st. 7; every later disc console | **a runtime service** (one function, one owner of the decision) over recognisers in the system packs | none at the ABI; `SystemEntry` gains `Recognise` |
| 9 | **Memory cards, packs and a console's own memory** | Diamond st. 6, the Sega CD, the N64's Controller Pak | an **extension**: a descriptor field; the path policy the runtime's | machine info's `battery[].kind` |
| 10 | **Device kinds; streaming reads; battery files before create** | device kinds later; the other two Diamond, then the Sega CD | device kinds **deferred**, with the shape that would fit recorded; streaming reads an **extension**, the image lent for the machine's life (§10.2, decided 2026-10-12); battery files an **extension** of core info (§10.5, decided 2026-10-12); the drive's place named ahead an **extension**, an event kind (§10.2.6, decided 2026-10-12) | minor 1.3: `BORROW` (bit 20), `emusen_create_params.flags` with `EMUSEN_CREATE_LENT`, `EMUSEN_MEDIA_LENT`; core info's `systems[].battery`. Minor 1.4: `EMUSEN_EVENT_MEDIA_SEEK` (8) and `EMUSEN_FILE_IMAGE` |

Read §1 for the one mechanism the five ABI changes share, §2 before §3 and §8, which both build on it, and §12 for the
questions. §13, added 2026-10-11, is a tenth item of a different origin, the debugger's: read watches and read data
breakpoints as minor 1.2, decided and built that day with Diamond as the first core to claim them.

---

## 1. The forms, and the one mechanism the ABI changes share

### 1.1 The three forms

The project's rule is that a feature which would give a component a second job, couple two components or bloat a
stable surface is first placed in one of three forms: **an extension** of an existing surface (an optional capability,
a table entry, a minor version), **a module or plugin**, or **a component of its own**. Each section below weighs the
three and says why one wins. Two outcomes recur and are worth naming once:

- **No module form exists for anything that must cross the core boundary.** A disc's bytes, a motor's level and a
  pad bit pass between a core and its host only through `emusen_core.h`; a DianaSh module or a runtime plug-in cannot
  carry them [A, from `EmuSen_CoreAPI.md` §2.4, §6.16: the core never calls the host, and nothing else crosses].
  For those gaps the choice is between an extension of v1 and a core's own extension exports (§4.7 there).
- **A core's own extension exports are the wrong form whenever two cores need the same thing.** §4.7 there says that
  an extension which becomes common is added to the baseline and its old name deprecated. Disc change, rumble and the
  memory-card kind are needed by Diamond and Nephrite's Sega CD at least, so starting them as extensions would buy
  only a later deprecation [A].

### 1.2 What a minor may add, and which of it this page uses

`EmuSen_CoreAPI.md` §4.4 lists what does not break version 1. This page uses five of its six kinds and nothing else:

| Kind of addition (§4.4 there) | Used by | Discovered by |
|---|---|---|
| a new optional export with a new capability bit | §3 (`MEDIA`, bit 18; bits 18–47 are free, `emusen_core.h:85` [M]) | the bit and the export's name (§4.2) |
| a new event kind | §4 (`EMUSEN_EVENT_OUTPUT` = 7; kinds 1–6 assigned, `emusen_core.h:132–139` [M]); §10.2.6 (`EMUSEN_EVENT_MEDIA_SEEK` = 8, minor 1.4) | a host skips a kind it does not know |
| a new descriptor field | §3 (`drives`), §4 (`outputs`), §5 (`name`), §9 (`kind`) | a reader ignores fields it does not know |
| a new header constant | §2 (`EMUSEN_FILE_MEDIA`), §3 (`EMUSEN_MEDIA_RESTORE`), §10.2.6 (`EMUSEN_FILE_IMAGE`) | — |
| a new status code in the interface's band | §3 (−265 to −267; −256 to −264 assigned, `emusen_core.h:53–61` [M]) | `status_text`, and a generic report for a code a host does not know |

**One rule governs all five ABI changes**, and each section states how it is kept: *every addition is safe never to be
used* (§4.6 there). A core that gains one behaves as a v1.0 core when the host never calls it; a host that meets an
older core presents fewer features, never a failure. The C# host already skips an event kind it does not know: the
switch in `CoreEngine.Drain` has no default arm (`EmuSen/Cores/Native/CoreEngine.cs:273–286`) [M].

**Minors.** Each addition to the baseline increments `EMUSEN_CORE_ABI_MINOR` (§5.5 there), which is 0 today
(`emusen_core.h:37`) [M]; no minor has been released (§17 there). This page does not number them: whichever lands
first is 1.1. §10.3 proposes the order, by the stage of Diamond or Nephrite that first needs each.

---

## 2. Image sets: a sheet's track files, and `.m3u`

*Listed in `Nephrite_Plan.md` §8.3 ("multi-file images for discs") and implied by `Diamond_Plan.md` §5.11; placed first
because §3 and §8 build on it.*

### 2.1 What exists today

- **`create` receives one image's bytes and an array of numbered files** (`emusen_create_params`, `emusen_file`,
  `emusen_core.h:156–176`) [M]. File 0 is the battery save (`EMUSEN_FILE_BATTERY`, `:92`); other numbers are the
  core's, named in machine info or in `firmware_for`'s answer (§6.2 there). The cores use small numbers: VenusRT's DSP
  firmware is file 1 to 3 (`VenusRT - SNES/src/v1.rs:70, :83`) [M]. A core answers a number it does not know with
  `EMUSEN_BAD_FILE`, as MercuryRT's test pins (`MercuryRT - GB/tests/ffi.rs:40`) [M].
- **Decided 2026-10-04** (`Nephrite_Plan.md` §9 Q5): the system pack parses `.cue` and the host passes the tracks as
  numbered files after the firmware's; the whole disc is copied at create; no streaming minor until a measurement shows
  the copy matters [R].
- **Refined on paper 2026-10-10** (`Diamond_Plan.md` §5.11): the image passed to `create` is the sheet itself (`.cue`,
  **`.m3u`**), the system pack reads only its `FILE` lines, and the shared disc crate `emusen-disc` parses the sheet in
  full, so that one parser of the format exists [R]. Not built: there is no `emusen-disc` and no `.cue` handling in any
  system pack (`EmuSen.DianaSh/DianaSh/Sys/Systems/` holds the SNES and Genesis packs only) [M].
- **No numbering is reserved for track files**, so "after the firmware's" would collide with a core's own numbers
  the day a disc core also takes a cartridge or an add-on's firmware [A].

*Built 2026-10-11, at Diamond's stage 7 (`Diamond_CDROM.md` §8):* the set module in `emusen-disc` (`set::companions`,
`set::playlist`, `set::sbi_beside`) and the core's side, Diamond opening a sheet with its companions as files from
65,536; the header constant and the runtime's reading of the companions are not built, and Diamond defines the number
locally until they are.

*Built 2026-10-11, at Diamond's stage 11 (`Diamond_Players.md` §2):* `EMUSEN_FILE_MEDIA` in `emusen_core.h` and
`emusen-native` as minor 1.1 (`EmuSen_CoreAPI.md` §6.23), Diamond's `MEDIA_FILE` now the header's; the set module
through the platform library as `emusen_disc_companions`, `emusen_disc_playlist` and `emusen_disc_sbi_beside`
(interface 8), read by the runtime's `ImageSets`; the runtime passing a sheet's companions and expanding an `.m3u`; the
kit's `--image` and `--images` forms the same (its `media` module, a path dependency on the crate); the library
shelving a set once (`ImageSets.Shelved`). The platform library now links `emusen-disc`, and through it the `chd`
crate, the stated dependency of §2.3.

*Amended 2026-10-12 (`Diamond_Players.md` §12.4):* the runtime finds a companion by its name as written, else part by
part ignoring case with `/` and `\` both separating parts, so a sheet made where names ignore case opens on a
filesystem that keeps it; the set module still answers the names as written, and the kit's `media` module joins them
as written.

### 2.2 The questions inside the gap

Three facts are mixed in "multi-file images", and each wants its own owner:

1. *What a sheet means* (tracks, modes, indexes, pregaps): the disc crate's, inside the core, since only the core
   needs it [R `Diamond_Plan.md` §9.2].
2. *Which files a sheet names*, so that the host can read them: needed outside the core, by the host and by the
   conformance kit, which loads images without any C# (`emusen-core-conform`, Rust) [M `src/main.rs:188`].
3. *Which discs form one game* (`.m3u`): a property of the player's collection, not of a console or a disc. No core
   needs it to emulate anything; the runtime needs it to offer the next disc (§3) [A].

### 2.3 The forms weighed

- **The system pack reads the `FILE` lines** (Diamond's §5.11). It puts a format that belongs to no console into one
  console's pack, which is the borrowing `EmuSen_CoreAPI.md` §8.2 names as the defect of codecs living with one C#
  core; and the kit, being Rust, cannot use it, so a second reader would be written there [A].
- **The core receives the `.m3u` and every disc's tracks at create** (also Diamond's §5.11, read literally). Final
  Fantasy VII's three discs, on Diamond's recommended bench [R `Diamond_Plan.md` §11 Q16], would be copied whole at
  create, about 2.1 GB, where one disc is played at a time [A]; and a disc change would need no export, only because
  every disc is already inside, which §3.3 rejects for that reason.
- **Recommended** [C]: the *set* module of `emusen-disc`, a component of its own only in the sense that the crate is
  shared, holds the two small readers, **the companions of a sheet** (`.cue`'s `FILE` lines; later `.gdi`'s file
  column and `.ccd`'s sibling `.img`/`.sub`) and **the `.m3u` list**. The kit uses it as a path dependency; the runtime
  uses it through the platform library, under that library's exact-version rule B (`EmuSen_Surfaces.md` §6.1). The
  sheet's meaning stays in the same crate, used by the core. One crate owns the formats; three binaries link it.

  This is a new cross-component dependency and is stated as one: the platform library (`Platform/`) would link a crate
  from `EmuSen/Cores/Shared/`. It already links one, `emusen-native`, for `abi::install_crash_log` only
  [R `EmuSen_Surfaces.md` §6.2].

### 2.4 The surface change

- **A header constant** [C]:

  ```c
  #define EMUSEN_FILE_MEDIA 65536u  /* emusen_file.which of a sheet's first companion; the nth is MEDIA + n */
  ```

  A core that claims an extension listed in the table below receives the sheet as the image and its companions as
  files `EMUSEN_FILE_MEDIA + n`, in the order the sheet names them. Numbers below 65,536 stay the core's.

  | Extension | Companions | Who reads the list |
  |---|---|---|
  | `.cue` | each `FILE` line's file, in order | `emusen-disc`'s set module |
  | `.gdi` (later, Kunzite) | each track line's file | the same |
  | `.ccd` (later) | the sibling `.img` and `.sub` | the same |
  | `.chd`, `.iso`, a bare `.bin` | none: one file | — |

- **`.sbi`** (Diamond's LibCrypt patches, read beside a disc, never opened alone [R `Diamond_Plan.md` §1.2]) is a
  companion found by name: the sheet's or image's stem with `.sbi`, passed after the sheet's own companions. Listed in
  the same table when Diamond's stage 7 builds it.
- **`.m3u` never reaches a core** [C]. The runtime expands it; `create` receives the first disc's image and companions,
  or the disc a resume state's record names (§3.5); the other discs are offered through §3. A system pack lists `.m3u`
  among its console's extensions, so the library shelves the set (§8.5); a core's info need not.
- **The library lists a set once** [C]: a file that an `.m3u` beside it names is hidden from the shelf, as ES-DE's
  convention expects of a multi-disc folder (to be checked against ES-DE's documentation when the library builds it).

### 2.5 Older and newer peers

A host that predates the constant passes no companions, and a core given a bare sheet refuses it with its own words
(`EMUSEN_BAD_IMAGE`), which is the v1.0 behaviour for any image it cannot use [A]. A core that predates the constant
never claims `.cue`, so a newer host never sends it companions. Neither side fails silently.

### 2.6 Cost, and what it breaks

The set module (two readers and their tests); one platform-library call pair (companions of a sheet, entries of a
list) under an incremented `EMUSEN_PLATFORM_ABI_VERSION` (6 today, `Platform/include/emusen_platform.h`) [M]; the
runtime reading the companions before create; the kit's `--images` form learning that a list entry may be a sheet.
It breaks nothing; it **revises two recorded decisions** — Nephrite's Q5 ("the system pack parses `.cue`") and
Diamond's §5.11 ("the system pack reads its `FILE` lines", "`.m3u`" as an image) — so it is put to the tester (Q2).

---

## 3. Disc change: the lid, a swap, and a set

### 3.1 What exists today

- **No export inserts, ejects or opens anything** after `create`; v1.0's 55 names are listed in `EmuSen_CoreAPI.md`
  §6.21, and none concerns media [R]. The header has no media constant [M `emusen_core.h`].
- **The cores model the lid themselves, or will.** Diamond's CD-ROM row puts "the lid and disc swap" in the core
  [R `Diamond_Plan.md` §1.1]; its §10 asks for "an export to open the lid and insert another disc of a set".
  Nephrite's Sega CD is a stub until stage 10 (`EmuSen_CoreAPI.md` §27.4) [R].
- **Who needs it** [A]: every multi-disc game. Diamond's bench names two (Final Fantasy VII, three discs; Metal Gear
  Solid, two) [R `Diamond_Plan.md` §11 Q16]; Sega CD titles shipped on two discs exist, to be counted against the
  tester's discs when there are any; the Saturn, the Dreamcast, the PC Engine CD and the PS2 have the same need.
  Single-disc games use the lid too: a few PlayStation titles ask the player to swap to a music CD or to another
  game's disc [A, not counted].

### 3.2 What the mechanism must do

Three things a player does and one the runtime does [A]:

1. **Open the lid** (or the tray). The drive's controller sees it; on the PlayStation the CD-ROM controller reports
   the open lid, which games watch for [R `Diamond_Plan.md` §1.1, "the lid and disc swap"].
2. **Take the disc out, put another in.** Physically possible only while open.
3. **Close the lid.** The drive spins up and reads the new disc's TOC, in the core's own time.
4. **Restore a drive for a state or a rewind**: a state saved with disc 2 in must load with disc 2 in, although a
   disc's bytes are not part of a state, as a cartridge's are not (§6.9 there: the state is the core's; the image is
   the host's to pass).

### 3.3 The forms weighed

- **(a) Every disc at create, a run-scope setting to choose one and another for the lid.** No new export. Rejected
  [A]: all discs copied at create (§2.3); a setting is a preference the runtime persists per console in
  `graphics.json` and shows in a settings window (§6.13 there), where a lid's state is neither; and settings text
  cannot carry a disc that was not given at create, so a player could not insert an arbitrary disc.
- **(b) A new machine with the next disc, the state carried over.** Rejected [A]: the core would have to accept a
  state made on another image; the period the lid is open, which games observe, could not be modelled; and the
  machine's identity (debugger registries, rewind) would change under the player.
- **(c) A core's own extension exports** (`diamond_media_*`). Rejected by §1.1: at least two cores need the same.
- **(d) A module or a component.** None exists: the bytes cross the core boundary (§1.1).
- **(e) Recommended** [C]: **two optional exports behind a new capability bit**, `MEDIA`, in a minor.

libretro offers the same service as a set of callbacks for the eject state and an image index [R libretro's published
API, as `EmuSen_CoreAPI.md` compares it; nothing of it reproduced]; (e) differs in that the host passes the bytes, so
the core never opens a path, as §6.2 there already requires of every file.

### 3.4 The surface change

```c
#define EMUSEN_CAP_MEDIA       (1ull << 18)  /* emusen_core_media_lid, emusen_core_media_insert */
#define EMUSEN_MEDIA_RESTORE   1u            /* emusen_media.flags: the drive set for a state about to be loaded */

#define EMUSEN_LID_CLOSED      (-265)  /* media_insert while the drive's lid or tray is closed */
#define EMUSEN_WRONG_MEDIA     (-266)  /* state_load: the drive holds another disc than the state's */
#define EMUSEN_NO_SUCH_DRIVE   (-267)

typedef struct emusen_media {   /* 48 bytes */
    uint32_t size;
    uint32_t flags;             /* EMUSEN_MEDIA_RESTORE */
    const uint8_t *image;       /* the disc's image or sheet, as create's; null empties the drive */
    size_t image_len;
    const emusen_file *files;   /* its companions, from EMUSEN_FILE_MEDIA (section 2) */
    size_t file_count;
    size_t file_size;           /* sizeof(emusen_file) as the host knows it */
} emusen_media;

/* MEDIA: open (1) or close (0) the drive's lid or tray, as the player would; the core models what follows. */
int32_t emusen_core_media_lid(emusen_machine *machine, uint32_t drive, uint32_t open);
/* MEDIA: the disc in the drive, replaced; refused with EMUSEN_LID_CLOSED unless open or RESTORE is set.
 * The core copies what it keeps, as at create; a refused image leaves the drive as it was. */
int32_t emusen_core_media_insert(emusen_machine *machine, uint32_t drive, const emusen_media *media);
```

**Machine info** gains `drives` [C]: `[{ "drive": 0, "label": "Disc drive", "kind": "cd", "lid": "closed",
"loaded": true }]`, `kind` an open string (`cd`, later `gd`, `dvd`, `floppy`). The core raises
`EMUSEN_EVENT_MACHINE_INFO` when it changes a drive's state itself, as a motorised tray opened by the console's
software would (whether the Sega CD's drive can be opened by software is to be checked at Nephrite's stage 8 [A]).
Absent, a host assumes no drives.

**The Rust side** [C]: `Core` gains `media_lid` and `media_insert` with `NOT_SUPPORTED` defaults, claimed by the bit
through `check_groups` as every optional group is (`emusen-native/src/core/mod.rs:222`) [M]; `core_exports!` checks
`emusen_media.size` against 48 and turns the files into slices as `create` does.

### 3.5 What the runtime does with it

- **A set** is the `.m3u`'s list (§2.4), or a single disc with "insert another disc…" offering any image file.
- **"Change disc"** is one action for the player: open the lid, insert the chosen disc, close the lid after a fixed
  interval of emulated time, so a game sees the lid open as it would by hand [C; the interval a runtime constant,
  argued at Diamond's stage 11 from what games are measured to require]. The lid's two halves stay separate actions
  for a player who wants them, and the shell's (`disc open`, `disc insert 2`, `disc close`).
- **States and rewind.** The runtime records beside each state the disc in each drive (the set and its index, or the
  path), as it records the state's format and version today (`IStateFormat`, §6.4 there). To load, it inserts that disc
  with `EMUSEN_MEDIA_RESTORE`, then calls `state_load`; on any failure it reinserts the disc that was there, again with
  `RESTORE`. The core's state holds the lid and a fingerprint of the disc it was made on [C: the fingerprint is the
  core's choice, such as a digest of the TOC and the first data sector]; `state_load` answers `EMUSEN_WRONG_MEDIA`
  when the drive holds another, and changes nothing, as every failed load must. Rewind's snapshots carry the disc
  index the same way.
- **Determinism.** Lid and insert calls are inputs, recorded at their frames wherever input is recorded (a movie, the
  kit's script). The kit's input script gains `lid <drive> open|close` and `insert <drive> <file>` at a frame [C].

*Built 2026-10-11, at Diamond's stage 11 (`EmuSen_CoreAPI.md` §6.23, `Diamond_Players.md` §3), as §3.4 gives it, but
for one thing: the struct is 48 bytes, as §3.4's own comment says, and the runtime does not record the disc beside a
state. A state the core refuses with `EMUSEN_WRONG_MEDIA` is loaded again over each other disc of the set put back with
`RESTORE`, and the drive is restored where none is the state's; the core's fingerprint is the record. C16 is in the kit.
Change Disc's interval is 1.0 s of the game's time, from ps1-tests' `disc-swap`, which opens and closes the lid a second
apart; P5 is not tested, no multi-disc game being among the tester's discs.*

### 3.6 Older and newer peers

- **A newer host and a core without `MEDIA`**: the disc menu is not shown; an `.m3u` opens its first disc, and the
  host says once that this core cannot change discs. A multi-disc game is playable to the end of disc 1 [A].
- **An older host and a core with `MEDIA`**: the exports are never called; the lid stays closed with the create-time
  disc in, which is the v1.0 machine. A state made under a newer host with disc 2 refuses to load on the older one with
  `EMUSEN_WRONG_MEDIA`, which an older host reports as an unknown status of the interface's band; nothing is changed
  [A].

### 3.7 The kit

A new case, **C16, media** (if `MEDIA` is claimed) [C]: insert while closed is `EMUSEN_LID_CLOSED`; open, insert the
same image, close, run N frames without fault; a state saved after the swap loads only over the same disc
(`EMUSEN_WRONG_MEDIA` otherwise, the machine's state digest unchanged); C8's determinism holds with a scripted swap; a
null image empties the drive. C2 checks the bit against both exports; C12's skew host never calls them. Synthetic
discs from WiseMan's planned disc builder [R `Diamond_Plan.md` §3.3] serve, so CI needs no commercial disc.

*Amended at 1.6, 2026-10-14 (`EmuSen_CoreAPI.md` §6.26):* a machine of a `MEDIA` core may list no drive, as a cartridge
console beside its disc attachment in one library does; C16 then checks that every media call is refused with
`EMUSEN_NO_SUCH_DRIVE` and changes nothing, and the drive rules above only where a drive is listed.

### 3.8 Cost, and what it breaks

About fifteen baseline lines (two exports, two function types, the struct and its seven fields, three status codes,
two constants, the bit) and one descriptor field; the trait methods, macro arms and C# table entries
(`CoreInterface.cs`, `CoreMachine.cs`); the runtime's set and lid service and the state record's drive entry; a disc
item in Mistress's game menu, held to the fit audit at 1280×800 and 1920×1200; C16. Two to three steps [A]. It breaks
nothing in v1; it asks every disc core's state format to hold the lid and the fingerprint from its first version,
which costs Diamond nothing since its format is not yet written.

---

## 4. Rumble, and the pad's other outputs

### 4.1 What exists today

- **No output path exists anywhere.** `EmuSen_CoreAPI.md` §6.8: "Outputs, such as rumble, would be pulled through
  events (§6.19); v1.0 defines none"; §6.20 lists rumble among "later minors, as events out" [R]. No event kind for it
  (`emusen_core.h:132–139`) [M]. `EmuSen_Input.md` §7.5 lists rumble as not covered [R].
- **The frontends cannot drive a motor.** Endymion's device seam has no rumble or light member
  (`EmuSen.Endymion/Input/IPadDevices.cs`) [M]. The SDL binding it references, SDL3-CS 3.4.2
  (`EmuSen.Endymion.csproj:26`), does export `SDL_RumbleGamepad`, `SDL_RumbleGamepadTriggers` and `SDL_SetGamepadLED`,
  and the capability property `SDL.joystick.cap.rumble` [M, `strings` over the package's `SDL3-CS.dll`].
- **The N64 set no precedent.** Mars and MarsRT model the Controller Pak and nothing else in a pad's slot: a port
  holds "A Controller Pak in the slot, or none" (`Mars - N64/Memory/Controller.cs:15`), and the address range where an
  accessory identifies itself reads zero (`Memory/ControllerPak.cs:28–33`; MarsRT's `src/memory/controller.rs` is its
  port) [M]. A game probing for a Rumble Pak therefore finds a Controller Pak or nothing, so it never drives a motor
  [A]. There is no output to carry, and no design to inherit.
- **The Game Boy is the nearest precedent, and it stops inside the core.** Mercury and MercuryRT model the MBC5's
  rumble line: on a rumble cartridge, bit 3 of the RAM-bank register drives the motor instead of addressing a bank
  (`Mercury - GB-GBC/Memory/Mappers/Mbc5.cs:48–49`; `MercuryRT - GB/src/memory/mappers.rs:126–129`), and the state is
  visible only as a debugger field, `Rumble` (`Mbc5.cs:83`; `mappers.rs:248`) [M]. The motor runs in the emulation and
  reaches nothing.
- **Diamond** models the DualShock's motors from stage 6 [R `Diamond_Plan.md` §4.2, §8 stage 6].

### 4.2 The forms weighed

- **(a) A pull export per frame**, `outputs(m, port, out, len)`, behind a bit. Works, but the host calls it every
  frame for every port whether anything moved or not, and it adds two baseline names for what §6.8 there already
  assigned to events [A].
- **(b) An event per change, with the level in it.** §6.8 and §6.20's stated intent; the event queue exists and is
  drained after every `advance` (§6.19). Chosen [C].
- **(c) A setter into the core** (the host asks for the level). Inverted: the level is the emulation's output.
- **A module or component**: no; the level crosses the core boundary (§1.1). On the frontend side, the motor is
  Endymion's job (a device), which it already is for every other pad matter (`EmuSen_CoreAPI.md` §8.2, "what stays out
  of the runtime") [R].

**One mechanism for every output, not one for rumble** [C]: the DualShock also has a light, the Analog LED, which
Diamond's drawing shows [R `Diamond_Plan.md` §4.2]. A motor and a light are both a level on a port's device; declaring
them as data, as axes are declared (§6.3 there), lets one event serve both and later outputs (a GameCube controller's
motor, a Dreamcast Jump Pack, a GBA cartridge's motor) without another minor [A].

### 4.3 The surface change

**An event kind** [C]:

```c
EMUSEN_EVENT_OUTPUT = 7   /* a: (port << 32) | output; b: its level over the frame, 0 to 65535 */
```

**Machine info's port entries** gain `outputs` [C]:

```json
{ "port": 0, "controller": "psx.dualshock",
  "outputs": [ { "output": 0, "kind": "motor", "motor": "strong", "label": "Large motor" },
               { "output": 1, "kind": "motor", "motor": "weak",   "label": "Small motor" },
               { "output": 2, "kind": "light", "label": "Analog" } ] }
```

- **In machine info, not core info**, because outputs depend on the game and the port's device: a Game Boy cartridge
  with a motor declares one on port 0, one without declares none; a Rumble Pak in an N64 port declares one while it is
  plugged in. A change raises `MACHINE_INFO`, as a port's device change already does (`EmuSen_Input.md` §8.11) [R].
- **`kind`** is open: `motor`, `light`. **`motor`** is `strong` (the low-frequency, heavier motor) or `weak`, the two
  SDL drives separately (`SDL_RumbleGamepad`'s low- and high-frequency arguments [R SDL3's API reference; the binding's
  export measured above]) and the split libretro's rumble interface also makes [R its published API].
- **The level is normalised and averaged over the frame** [C]: 0 to 65535, the fraction of the frame's time the
  output was driven, times its strength. A core that sees only on and off writes 0 or 65535 at the frame's end; one
  that sees a game pulse a motor faster than frames (a common way to vary a motor's strength [A]) writes the average,
  so the pulse's intent survives the per-frame event. The core does the normalising, as it does for axes (§6.8 there),
  because only it knows the console's units.

**The Rust side** [C]: `Core` gains `fn output_level(&self, port: u32, output: u32) -> u16`, default 0.
`core_exports!` keeps the last level raised for each declared output in a table allocated when machine info is read,
and after `advance`, `state_load`, `reset` and `set_settings` raises `OUTPUT` for each output whose level changed, as it
already raises `GEOMETRY` and `AUDIO_RATE` on observing a change (§6.19 there). No allocation per frame; a frame with
no change raises nothing. After the macro queues `MACHINE_INFO` (a lost event, or a change of ports), it forgets the
table and raises every declared output's level after the next `advance`, so a host never holds a stale level.

### 4.4 What the runtime and the frontends do

- **The runtime** routes an `OUTPUT` for port *p* to the player seated at *p* (`PlayerSlots`, `EmuSen_Input.md` §8.2)
  and drives **zero** whenever the loop is not running the game at its speed for that player: paused, halted at a
  breakpoint, under a menu, rewinding, or loading a state until the core raises the restored levels [C].
- **Endymion** gains `Rumble(pad, strong, weak, ms)` and `Light(pad, level)` on `IPadDevices`, over the SDL calls
  above, refreshed with a duration longer than a frame so a missed refresh stops the motor rather than leaving it on
  [C]. The platform library's Endymion half gains the same pair under its exact-version rule.
- **Settings, the player's**: rumble on or off and its strength, per player, in Preferences; default on, since the
  motor is the hardware's [C, Q4]. A light is shown on the controller drawing (`EmuSen_Settings_Reference.md` §4.81)
  and not sent to a pad's light bar unless the player asks [C].

### 4.5 Older and newer peers, and the kit

An older host skips kind 7 and ignores `outputs`; the game plays without rumble, as today. A newer host with an older
core finds no outputs and drives nothing. Determinism is untouched: outputs never feed back into the machine [A]. The
kit's C6 checks that every `OUTPUT` names a declared output with a level in range, and C8's digests gain the output
stream so that two machines must agree on it [C].

### 4.6 Who would use it, and cost

Diamond (the DualShock, stage 6); MarsRT, once it models a Rumble Pak as a choice of what each port's slot holds,
which is a setting as the Genesis's pad types are (`EmuSen_Input.md` §8.11) [C]; MercuryRT's MBC5 carts, a few lines
since the motor is modelled (it is paused, and resumes after Nephrite, by a recorded decision); later Uranus, Kunzite
and Sapphire [A]. Cost: one event constant, one descriptor field, one trait method and a macro table; the runtime's
routing and stop rule; Endymion's two members and the C interface's pair; Preferences' rows. One step for the ABI and
the runtime, one for the frontends [A]. It breaks nothing.

---

## 5. The Analog button

### 5.1 What exists today

- **`PadButton` is the RetroPad's sixteen, append-only** (`EmuSen.Galaxia/Input/PadButton.cs`), and the header's
  `emusen_control` mirrors it (`emusen_core.h:142–147`) [M]. libretro's joypad ids stop at 15 [R `EmuSen_Input.md`
  §7.1, from `libretro.h`]; SDL calls the extra buttons Guide, Misc1, the paddles and the touchpad [R SDL3's API].
- **The Guide button is taken.** Mistress's pad navigation opens its menu on Guide, or on the two middle buttons
  together (`EmuSen.Mistress/Input/PadNavigator.cs:52–55`; `Views/MainWindow.Pad.cs:144`) [M].
- **v1.0 already allows a control with no canonical name.** A button entry may carry `"control": null`, "and the
  frontend offers it as an unbound extra" (`EmuSen_CoreAPI.md` §6.3) [R]; `emusen-native` writes it (`Button.control:
  Option<Control>`, `src/core/desc.rs:111–117`) and the host parses it (`CoreButton(uint Bit, PadButton? Control,
  string Label)`, `EmuSen/Cores/Native/CoreDescriptors.cs:10`) [M].
- **But nothing can press it.** `CoreEngine.SetButton` routes by `PadButton` alone, so an entry whose control is null
  is never reached (`CoreEngine.cs:97–106`); `SupportedButtons` lists only entries with a control (`:92`) [M]. The
  spec's promise is unbuilt.
- **The Genesis's precedent**: its Mode button maps to Select (`Nephrite - Genesis/src/v1.rs:40`) [M], which works
  because the Genesis pad has no Select. The DualShock has one, so the precedent does not transfer.

### 5.2 Who needs it

The DualShock's Analog button (Diamond's `psx.dualshock`, [R `Diamond_Plan.md` §4.2]) and the DualShock 2's (Sapphire);
the Saturn 3D Control Pad's analog/digital switch [A]; and, more heavily, every console whose controller has keys the
RetroPad lacks: the Atari 5200's and the Jaguar's twelve-key keypads, the Intellivision's and the ColecoVision's
[A]. Folders for the 5200 (Mimete) and the Jaguar (Cyprine & Ptilol) exist under `EmuSen/Cores/Atari/` [M].

### 5.3 The forms weighed

- **(a) Append `PadButton` 16** (a generic `Mode` or `Aux1`). Rejected [A]:
  - it ends `PadButton`'s identity with the RetroPad, so a libretro adapter (`EmuSen_Libretro.md`) would have no id for
    it;
  - `PadControl` numbers its first sixteen as `PadButton` and its stick directions from 16
    (`EmuSen.Galaxia/Input/PadControl.cs`), and `PadControls.From` and `IsButton` depend on that, by a cast and by
    `control <= PadControl.R3` (`PadControls.cs:9, :14`) [M]. A seventeenth `PadButton` collides with
    `LeftStickUp`; both functions would become tables and every binding layer re-checked;
  - it solves one button. The keypad consoles need twelve, which no generic enum should carry.
- **(b) A setting only** (the DualShock's mode at power-on). Needed anyway (Diamond's Q11 recommends digital at
  power-on [R]), but a player changes mode during play and some games tell the player to press the button [A]; a
  setting cannot be pressed.
- **(c) Recommended** [C]: **build the extras path v1.0 already specifies**, adding one descriptor field so that a
  binding has a stable key, plus Diamond's power-on setting.

### 5.4 The surface change

- **Core ABI, additive**: a button entry may carry `name`, `[a-z0-9-]+`, unique within its controller:
  `{ "bit": 16, "control": null, "name": "analog", "label": "Analog" }`. A binding is stored under
  `<controller id>.<name>`, `psx.dualshock.analog`. C2 checks uniqueness and that every entry with a null control has
  a name. An older core without `name` is keyed by its bit (`psx.dualshock.bit16`), so a newer host still offers it; an
  older host ignores the field and offers nothing, as today. Bits stay below 32, the mask's width
  (`emusen_core.h:254`) [M].
- **Frontends, internal surfaces (rule D)** [C]:
  - **Galaxia**: the per-console binding models gain a map from extra name to a key, a pad button or a chord, beside
    the `PadControl` map, so `PadControl` keeps its numbering.
  - **Endymion**: `GamepadBindingMap` may bind an extra to a raw SDL button that no `PadButton` holds (Misc1, a
    paddle, the touchpad's click; never Guide, which the menu holds) or to a chord of two buttons.
  - **The host**: `ICore` gains a defaulted `SetExtra(int port, string name, bool pressed) { }`, beside `SetButton`
    (`EmuSen/Cores/ICore.cs:35`) [M]; `CoreEngine` sets the named entry's bit.
  - **Controller Bindings**: the console's tab lists its extras after its canonical rows, drawn on LunaP's diagram
    with the console's label.
- **Defaults** [C, Q6]: a keyboard key no default and no hotkey holds; on a pad, Misc1 where the pad has it, else
  unbound and shown so. Diamond's power-on mode stays its setting (its Q11).

### 5.5 Cost, and what it breaks

One descriptor field and a C2 check; the binding map, the router entry and the window's rows. The key persistence
rule of `EmuSen_Input.md` §7.4 (a control a file predates takes its default key unless taken) applies to extras
unchanged. It breaks nothing: `PadButton`, `PadControl` and every stored binding are untouched.

---

## 6. Conditional cheat codes

### 6.1 What exists today, which is most of it

- **The runtime's cheat model already has conditionals.** `CheatWriteType` holds `Set`, `Increase`, `Decrease`,
  `IfEqual` and `IfNotEqual` (`EmuSen.DianaSh/DianaSh/Var/CheatWrite.cs:6–15`); a cheat is a list of writes with
  widths of 1, 2 or 4 bytes, either byte order, repeat runs with address and value steps, and bit positions (`:18–45`)
  [M]. `CheatRegistry.ApplyAll` evaluates a test against the space and skips the next write that is not a test when
  it fails, so a run of tests is their AND (`Var/CheatRegistry.cs:292–329`) [M]; `EmuSen_Cheats.md` §7 records the rule
  and why it differs from RetroArch's [R].
- **The N64's GameShark already uses it**: D0–D3 decode to tests, 80/81/A0/A1 to writes, 50 to a repeat
  (`Mars - N64/Cheats/N64GameSharkCodec.cs:45–61`), and the codes it cannot honour are refused by name (`:139–151`) [M].
- **The v1 host already applies them.** `CoreEngine.RefreshCheats` sends to a core that claims `CHEAT_POKES` only the
  cheats that are single-byte `Set`s with no repeat or bit; every other cheat, tests included, goes to the host's own
  registry (`CoreEngine.cs:414–438`), which `EndFrame` applies through `space_read` and `space_write` between `advance`
  and `present` (`:260–266`) [M]. A core that does not claim the bit has every cheat applied that way.

So **`Diamond_Plan.md` §4.5's premise is not the code's**: a conditional code does not need v1's poke quad, because
the runtime never asks the quad to express one [A, from the lines above]. What is missing is narrower.

### 6.2 What the PlayStation's codes need beyond it

psx-spx's "Cheat Devices" chapter lists the PlayStation GameShark's code types [R psx-spx, `cheatdevices`, read
2026-10-10]:

| Code | Meaning, as the chapter gives it | Expressible today |
|---|---|---|
| `30aaaaaa 00dd`, `80aaaaaa dddd` | 8- and 16-bit write | yes: `Set`, width 1 or 2 |
| `10`/`11`/`20`/`21` | 16- and 8-bit increment, decrement | yes: `Increase`, `Decrease` |
| `D0`/`E0`, `D1`/`E1` | 16-/8-bit "if equal", "if not equal", then the next code | yes: `IfEqual`, `IfNotEqual` |
| `D2`/`E2`, `D3`/`E3` | 16-/8-bit "If dddd<[aaaaaa]", "If dddd>[aaaaaa]", then the next code | **no**: no ordered comparison |
| `5000nnbb dddd` + line | slide: *nn* writes, address step *bb*, value step | yes: a repeat run |
| `C0aaaaaa dddd` | "If dddd=[aaaaaa] (turn on all codes)" | **in part** (below) |
| `C1000000 nnnn` | delays activation | **no** |
| `C2ssssss nnnn` + `80tttttt 0000` | copies *nnnn* bytes | **no** |
| `D4`/`D5`/`D6 000000 dddd` | tests the pad's buttons: run the next code, or all codes on, or all off | **no** |

### 6.3 The forms weighed

- **At the ABI**: nothing to add. Cheats are resolved by the runtime's codecs, outside the ABI (§6.12 there), and the
  host applies whatever the quad cannot carry [M, §6.1 above]. A richer `CHEAT_POKES` list would matter only to a core
  whose oracle gates its pokes, as MarsRT's does; Diamond has no such oracle [A].
- **A module**: the PlayStation's codec is a system pack, `Sys/Systems/Psx/`, as the SNES's and Genesis's are
  (`Sys/Systems/SystemPack.cs`) [M]; that is already the project's module form for console knowledge.
- **Recommended** [C]: **inside the runtime's cheat model**, append **`IfLess`** and **`IfGreater`** to
  `CheatWriteType`, with `Passes` comparing unsigned within the test's width; and write the codec. Both are internal
  surfaces of the runtime; the cheat file stores a write's type by name, so a build older than the change skips such
  an entry rather than misreading it, as it did for the tests (`EmuSen_Cheats.md` §7) [R].
- **`C0`** decodes, within one cheat, as an `IfEqual` before each following line, which is expressible today [C]. If
  the device latches ("turn on" once, and stay on), that is a different rule, and the codec refuses `C0` until a
  document or a test settles which (Q7).
- **`C1`, `C2`, `D4`–`D6`** are refused by name at first, as the N64 codec refuses its button and boot-time codes
  [C]: a delay counts from the device's own start, a copy is a new kind of write, and a button test reads the pad in
  the console's own word, which only the core's descriptor maps to bits. Each can be added later as one more
  `CheatWriteType` if the tester's codes need it (P4).

**One rule to settle at Diamond's stage 11** [A]: psx-spx says a conditional runs "the next code". The registry's AND
rule (a failed test skips the next *write*, so a second test after a failed first is skipped as a test, not evaluated)
equals the N64 references' rule (`EmuSen_Cheats.md` §7). Whether the PlayStation device skips the next *line*, test or
not, as RetroArch's handler form does, is not said by the chapter. If it does, the cheat model needs a per-cheat flag
for that rule; until a source or a test says, the codec uses the AND rule and says so (Q7).

**A defect this would let be fixed**: `ChtFile.ReadExplicit` imports RetroArch's `cheat_type` 4–7 ("run next if
equal", "not equal", "less", "greater") as writes of the compare value (`Var/ChtFile.cs:85–87`; demonstrated
2026-09-18, `EmuSen_Cheats.md` §7) [R]. With `IfLess` and `IfGreater`, all four map to tests; the cross-entry "next"
is still an import feature, but the types would no longer be the obstacle.

*Built 2026-10-11, at Diamond's stage 11 (`EmuSen_Cheats.md` §10):* `IfLess` and `IfGreater`, unsigned within the
test's width, the memory against the value; the PlayStation's codecs in `Crystal/Systems/Psx/`, GameShark (with Game
Buster and Action Replay, the same Datel table) and Xploder with its decryption, told apart by psx-spx's first-digit
table; `C0`, `C1`, `C2`, `C3` and `D4`-`D6` refused by name; the AND rule kept, since no source was found that says the
device skips a test as a line. Not done: `ChtFile`'s `cheat_type` 4 to 7, since mapping them means RetroArch's
cross-entry "next code", which is the import's and not a type's.

### 6.4 Cost, and what it breaks

Two enum members and their comparison; the codec (about the N64's 170 lines [M, `N64GameSharkCodec.cs`]) with its
refusals and tests; `cheat list`'s printing of the two new tests. It breaks nothing; the ABI is untouched.

---

## 7. The conformance kit's memory per image

### 7.1 What exists today

- **The corpus form sizes every image at 256 MiB**: `JOB_MEMORY = 256 << 20` (`emusen-core-conform/src/corpus.rs:23`),
  and `default_jobs` runs no more images at once than the tightest `memory.max` of the process's control groups, or
  the memory available, holds at that size, the core's own cases counted as one (`:113–142`) [M].
- **The real cost of an image is a multiple of its size** [M, from the code]:
  - the image is read whole by its thread and held for all its cases (`corpus.rs:145–151`);
  - C4 copies half of it (`lib.rs:678`), freed before C6;
  - C7 holds two machines at once (`a` and `b`, `lib.rs:831, :856`);
  - C8 holds up to three (`b`, `d` and, where skipping is declared state-neutral, `e`, `lib.rs:894, :898, :904`);
  - and each machine copies what it keeps of the image at create (§6.2 there).

  So an image's peak is about its size four times over, plus the machines' own memory [A]. For a Genesis cartridge
  that is a few MiB, and the measured whole was 243 MiB for sixteen images at once (`EmuSen_CoreAPI.md` §28.6) [R].
  For a 700 MiB disc it is about 2.8 GiB, eleven times the assumption: sixteen discs at once under the 8 GiB limit of
  that measurement would ask for about 45 GiB [A].

### 7.2 The forms weighed

- **At the ABI** (machine info declaring a machine's memory, or a core's per-byte factor): rejected [A]. The estimate
  is needed *before* the machine exists, and it is a property of the kit's cases, not of the core.
- **Recommended** [C]: **inside the kit** (rule R, an internal Rust API): an estimate from the image's size and a
  budget.
  - `estimate(len) = JOB_MEMORY + (MACHINES_AT_ONCE + 1) × len`, with `MACHINES_AT_ONCE = 3` as counted above, named in
    the code beside the cases that set it, so a case that holds a fourth machine changes the constant in the same diff.
  - The scheduler admits an image when its estimate fits in what remains of the allowed memory, instead of a fixed
    number of jobs; `--jobs` stays an upper bound. An image whose estimate exceeds the whole allowance runs alone, with
    one line saying so.
  - `summary.json` records the process's peak resident memory (`VmHWM`) and the largest sum of estimates admitted at
    once, so the estimate is checked against each run rather than trusted (P2).
  - `--image-memory N` overrides the factor for a core that expands an image (a `.chd` decompressed at create would be
    one) [C, Q9].

*Built 2026-10-11, at Diamond's stage 11 (`EmuSen_CoreAPI.md` §28.7):* `estimate(len) = 256 MiB + 4 × len`, the factor
`--image-memory` overrides; images admitted by a budget of the memory allowed less the core's own job, one larger than
it run alone with a line saying so; `summary.json`'s `memory` holding the allowance, the largest sum admitted and the
peak resident memory.

### 7.3 Cost, and what it breaks

A few dozen lines in `corpus.rs` and two tests (an image larger than the allowance runs alone; the budget admits two
images of half the allowance and not three). Cartridge corpora are unaffected in practice: at a few MiB an image, the
estimate stays near 256 MiB, and the processor count still bounds the jobs [A]. The single-image form is unchanged.

---

## 8. Routing a file to a core by its content

### 8.1 What exists today

- **Routing is by extension.** `CoreFactory.IsSupported` asks `CoreCatalog` and then discovery for the extension;
  `CoreFactory.Create` takes the first discovered engine whose offered systems claim it, unless a C# core's catalog row
  does (`EmuSen/Cores/CoreFactory.cs:39–68`; `Native/CoreDiscovery.cs:148–150`) [M]. A second engine claiming an
  extension is reached only when named (`Diamond_Plan.md` §4.1 says the same) [R].
- **The console of a file is decided by extension in many places.** `CoreCatalog.ConsoleForRom`, `ShelfFor` and
  `ByExtension` (`CoreCatalog.cs:256–261`, `:169–170`) are called at 22 sites in 12 files of the frontends and the core
  assembly: the shelf, the per-console settings sent at create, the save path, the crop, scraping, identity [M, `grep`
  over `EmuSen`, `EmuSen.Mistress`, `EmuSen.Hotaru`, `EmuSen.Pharaoh` and `EmuSen.DianaSh`]. Discovered engines are
  answered the same way, by the first system that lists the extension (`CoreInfo.SystemFor`,
  `Native/CoreDescriptors.cs:41`) [M].
- **Inside one core, content already decides.** Nephrite tells a Sega CD disc by `SEGADISCSYSTEM` at the start of the
  first sector's data, a 32X cartridge by its header's system name, and anything else as a Genesis cartridge
  (`Nephrite - Genesis/src/media.rs:140–148`; `Nephrite_Plan.md` §4.1), and reports its decision as machine info's
  `system` (§6.4 there) [M, R].
- **The fact "which extensions belong to which console" already has two owners.** The Genesis's are listed in its
  system pack (`Sys/Systems/Genesis/GenesisSystems.cs:22, :37`) and again in Nephrite's core info
  (`src/v1.rs:251–255`) [M]. They agree today; nothing checks that they do.
- **The collision is coming**: Diamond will claim `.cue`, `.chd` and perhaps `.iso`; Nephrite claims `.bin` for the
  Genesis and `.iso` for the Sega CD and will claim `.cue` and `.chd` (`Diamond_Plan.md` §4.1; `Nephrite_Plan.md` §4.1)
  [R]. `.chd` and `.cue` also serve the Saturn, the Dreamcast, the PC Engine CD and the PS2 later [A].
- **The player's library is filed by console**: `AppSettings.RomDirectory`'s folders are `GB`, `GBC`, `genesis`,
  `N64`, `NES`, `SNES` and `X360` [M, a listing of the folder, read only]. Each but the last equals a system pack's or
  catalog row's ES-DE system name when case is ignored [A, against ES-DE's names in the packs and `CoreCatalog`].

### 8.2 Three questions inside the gap

1. **Who decides?** One component, once per file, with every caller asking it: today 22 call sites decide for
   themselves by extension [M, above].
2. **Where does the console knowledge live?** What a PlayStation disc looks like (the licence text in the system area,
   `SYSTEM.CNF`'s `BOOT` line [R `Diamond_Plan.md` §4.1]) and what a Sega CD disc looks like (`SEGADISCSYSTEM`) are
   facts about consoles, not engines, which `EmuSen_CoreAPI.md` §8.2 gives to system packs [R].
3. **How are a disc's sectors read** for the test, when the file is a `.cue` with separate tracks or a compressed
   `.chd`? That is a format, owned by `emusen-disc` (§2) [A].

### 8.3 The forms weighed

- **(a) Each core recognises its own images**, through a new library-level export (`identify(params) -> JSON`) asked
  of every candidate. Rejected [A]: the signature would live in each engine, so a second PlayStation engine repeats it
  (two owners); the host would have to read a whole disc set into memory to ask; a console with no engine installed
  could not be shelved by content; and §8.2's facts are a system pack's by the runtime's own table.
- **(b) The extension's first claimant wins, the player chooses an engine per console** (today, extended). Rejected:
  a library holding both PlayStation and Sega CD `.chd` files would open half of them in the wrong core [A].
- **(c) Folders alone** (the ES-DE convention). Cheap and right for a library filed by console (§8.1), wrong for a file
  opened from anywhere else; kept as a tiebreak, not as the rule [A].
- **(d) Recommended** [C]: **a runtime service**, one function and one owner of the decision, over **recognisers in the
  system packs**, which read the file through **`emusen-disc`** for disc formats.

### 8.4 The surface change

All internal to the runtime (Crystal, which takes the system packs at `EmuSen_DianaSh.md` §13.2's step S2 [R]); none
at the core ABI.

*Since 2026-10-17 the recognisers, the decision among candidates, the disc's sectors and its ISO 9660 lookup are
`emusen-systems`' (`probe`, `genesis`, `psx::disc`), reached through the platform library unless
`EMUSEN_SYSTEMS_NATIVE=0`; a recogniser asks the host for the bytes it reads, so the file is read in C# as before
(`EmuSen_Backend.md` §13.3).*

- **`SystemEntry` gains a recogniser** [C]: `Func<IMediaProbe, Recognition?> Recognise`. A function in the entry is
  the pattern already used for OpenVGDB's hashing rule (`OpenVgdbBytes`, `Sys/Systems/SystemPack.cs:18`) [M].
  `IMediaProbe` offers the file's first bytes, and for a disc a 2,048-byte sector by its address, reading `.iso`, a raw
  `.bin`, a `.cue`'s first data track and a `.chd` through `emusen-disc` in the platform library (§2.3).
  `Recognition` is the system id and, where the pack can read them, the **serial** and the **region**: Diamond's
  OpenVGDB and cheat matching key on the serial from `SYSTEM.CNF` [R `Diamond_Plan.md` §4.1], so the recogniser has a
  second use. The ISO 9660 lookup that finds `SYSTEM.CNF` is a small helper of the runtime's media module, shared by
  every pack whose discs carry that filesystem; the disc crate keeps no filesystem [R `Diamond_Plan.md` §9.2].
- **`SystemOf(path)`, the one decision** [C]:
  1. the systems whose packs list the extension, and the systems of discovered engines without a pack that list it;
  2. one candidate: it, with nothing read;
  3. several: each candidate's recogniser; exactly one claim decides;
  4. none or several claims: the parent folder, when its name equals a candidate's ES-DE system name, ignoring case;
  5. otherwise the player's choice, asked once and kept with the game's per-game settings; a headless client refuses
     with words that name the candidates and the option that chooses one.

  The answer is cached by path, size and modification time. Every `ConsoleForRom`, `ShelfFor` and `SystemFor` caller
  asks `SystemOf`; the engine is then the chosen console's engine, as today
  (`CoreFactory.ConfiguredEngine`, `CoreFactory.cs:80–81`) [M].
- **One owner for each fact** [C]:
  - *A console's extensions* are its system pack's. A core's info lists the extensions **it opens**, which must be a
    subset of its system's pack where one exists; WiseMan's registration-equivalence test
    (`EmuSen_CoreAPI.md` §25) checks the subset for every discovered core with a pack.
  - *A console's signatures* are its pack's. A core that must still decide among its own systems to boot, as Nephrite
    must, keeps its decision; a WiseMan test holds the two to agreement over the synthetic images and the tester's
    library (pack's `Recognise` against machine info's `system`). The core cannot rely on the host's decision, because
    it must stand alone under the kit and the probe [A].
  - *Container formats* (sheets, sets, CHD) are `emusen-disc`'s.

*Built 2026-10-11, at Diamond's stage 11 (`Diamond_Players.md` §1):* `SystemEntry.Recognise` with `IMediaProbe` and
`Recognition`; `SystemRouting.Decide`, steps 1 to 4 with a cache by path, length and time; the PlayStation's recogniser
(the licence, and the serial from `SYSTEM.CNF` through the media module's ISO 9660 lookup, `DiscSectors`), the
Genesis's (anything that is not a compact disc image) and the Sega CD's (`SEGADISCSYSTEM`); `CoreCatalog.CoreForRom`
and `Route` the one decision, `ConsoleForRom`, `ShelfFor`, `DiscoveredConsoleForFile`, the factory's engine and the
battery folder asking it, and Mistress's `ByExtension` callers moved to it. Step 5, the player's choice, is not built:
a file no step decides opens nowhere, and `Route`'s words name the folders that would decide it. A probe reads a
`.cue`'s first track, an `.m3u`'s first disc and an archive's entry, but not a `.chd`, which only the PlayStation
takes. Pharaoh's `LibraryIndexer` keeps its own table of extensions; it is a tool's index, not a player's library.

### 8.5 Order, and the gap before the crate exists

`.cue` and `.chd` collide only when both Diamond and the Sega CD claim them, from Diamond's stage 7 [R
`Diamond_Plan.md` §10]. Until `emusen-disc` exists in the platform library, `IMediaProbe` reads `.iso`, `.bin` and a
`.cue`'s first track directly, and a `.chd` falls through to the folder and the player's choice (steps 4 and 5) [C,
Q11]. The service, the recognisers and the 22 callers can move before that, on cartridge consoles, where the
answer must not change: the registration-equivalence golden is the oracle (P3).

### 8.6 Cost, and what it breaks

The service and its cache; a recogniser per pack (the Genesis's three from Nephrite's existing rules, the
PlayStation's at Diamond's stage 7); the 22 call sites moved to one function; the player's chooser, held to the fit
audit; the two tests. It changes behaviour only where the extension is ambiguous, which today is nowhere a player
reaches (the Sega CD is in development) [A]. It breaks nothing at any surface; it narrows `CoreCatalog`'s role, as
`EmuSen_Surfaces.md` §4.4 asks.

---

## 9. Memory cards, packs, and a console's own memory

*Built 2026-10-15 for the removable kind only, as the core ABI's minor 1.7 (`EmuSen_CoreAPI.md` §6.27,
`Diamond_Cards.md`): battery entries' `kind`, `"game"` or `"removable"`, and a create flag under which a state load
leaves a removable file as it stands, which answers §9.3's risk in the core rather than by a warning. The runtime's
option has a third mode, named cards; `"console"` waits for a core that needs it.*

*`"console"` built the same day as the minor 1.8 (`EmuSen_CoreAPI.md` §6.28), with a slot's medium, label and port and
a second create flag; the framework's model, the card manager and the default are `EmuSen_SaveMedia.md`.*

*Diamond's §10 lists "a shared memory card option (one card for every game) in the runtime, if the tester wants
one"; Nephrite's §4.4 has the same question for the Sega CD's internal backup RAM and its RAM cartridge.*

### 9.1 What exists today

- **Battery files are per image.** Machine info lists each file as `{ which, suffix, length }` and "the path rule
  stays the runtime's (`SaveLibrary.SramPathFor`)" (§6.4 there) [R]; `SramPathFor(romPath, console)` names a file
  beside the game (`EmuSen.Galaxia/Library/SaveLibrary.cs:16–17`) [M]; `emusen-native`'s `Battery` carries `which`
  and `suffix` only (`src/core/desc.rs:235–238`) [M].
- **Three kinds of memory are treated alike** [A]: a cartridge's own SRAM, which belongs with the game; memory inside
  the console, such as the Sega CD's internal backup RAM (`.brm`, Nephrite §4.4 [R]), which every game shares on the
  hardware; and a removable card or pak, such as the PlayStation's memory cards (Diamond §4.4 [R]), the Sega CD's RAM
  cartridge (`.crm`) and the N64's Controller Pak, which Mars keeps "beside the cartridge's" as `.mpk`
  (`Mars - N64/MarsCore.cs:54–55`) [M].

### 9.2 The forms weighed

- **The system pack says which file is a card.** Rejected [A]: `which` numbers are the core's (§6.2 there); two
  engines of one console may number differently, so the fact must come with the numbers.
- **A runtime option per file number**, with no descriptor change: the same objection.
- **Recommended** [C]: **an extension of machine info**: each battery entry gains `kind`, one of `game` (the default
  when absent: the save belongs with the image), `console` (memory inside the console) or `removable` (a card or pak a
  player carries between games). The runtime owns the path policy for each kind, with a per-system option for the
  player: **per game** (today's rule, and the default for every kind) or **shared** (one file per system and slot, under
  the save library's root).

### 9.3 Cost, and what a shared card risks

One optional field with its default, C14 checking it is one of the three; `SaveLibrary` gaining a shared path rule;
the per-system option in the settings. **The risk of sharing**, stated for the tester [A]: a core's state usually holds
its battery memory, so loading an old state of one game rolls the shared card back for every game; and two sessions
open on one system would write one file. The runtime would refuse a second session on a shared card and warn once
about states. These are why per game stays the default (Q12). Older peers: a host that ignores `kind` keeps per-game
paths; a core without it reads as `game`.

---

## 10. Deferred, with the shape that would fit

### 10.1 Device kinds beyond pads

`Nephrite_Plan.md` §8.3 and `EmuSen_CoreAPI.md` §6.8 foresaw a later minor's `set_device` and device kinds for
multitaps, light guns and mice [R]. **The multitap no longer needs it**: the Genesis's Team Player and 4 Way Play are
a port setting, the pack answers each player's pad (`SystemPack.PlayerControllers`, `Sys/Systems/SystemPack.cs:24–27`;
`Genesis/GenesisSystems.cs:74–90`), and the core raises `MACHINE_INFO` when its ports change, which the host follows
(`CoreEngine.cs:280–285`; `EmuSen_Input.md` §8.11) [M, R]. Diamond's multitap (SCPH-1070) can take the same form
[A]. What remains is input that is not a button or an axis: a light gun's screen position and its trigger, a mouse's
relative motion. No core's player gate needs one (Diamond's Q12 puts them after its gate [R]); when one does, the
form is an optional `set_pointer` behind a bit and a controller `kind` in the descriptor [C]. Not proposed now.

### 10.2 Streaming reads

Decided 2026-10-04: no streaming minor until a measurement shows the whole copy at create matters (`Nephrite_Plan.md`
§9 Q5) [R]; Diamond measures it at stage 7 on the handheld (§5.11 there) [R]. If it is needed, the shape that keeps
v1's rule that the core never calls the host is an event asking for a range of sectors and an export supplying them,
with the core reading ahead; it is recorded here so that §3's `emusen_media` is not mistaken for it [A].

#### 10.2.1 The measurement (2026-10-12)

*Proposed 2026-10-12, after the tester reported a delay launching PlayStation games from big picture; decided the
same day (§10.2.3).* The measurement the 2026-10-04 decision waited for was taken on the
tester's seven discs, copied, with the opt-in probes of `Diamond_Players.md` §12.8 [M, 2026-10-12]:

- **The whole read is the launch's one remaining cost of size.** A cold read of a 430–660 MB image took 0.16–0.27 s
  from the desktop's NVMe drive and 4.8–6.9 s under a 90 MB/s read cap, the speed `Diamond_CDROM.md` §9.7 assumed for
  the handheld's microSD card; a sheet's tracks, or a zip's unpacked tracks, 1.7–6.8 s. **On the handheld itself**
  (`Diamond_Players.md` §12.8.6) the card read at 88–92 MiB a second: 6.3 s for a 556 MiB track, 6.2 s for a 572 MiB
  one, 4.6 s for a sheet's 424 MiB track.
- **On the handheld a second launch is not a warm one.** The launch's own 1.1 GB, the host's buffer and the core's
  copy, pushes the pages just read out of a page cache that fills the memory: 294 of a track's 556 MiB were still
  cached after its first launch, and its second read took 3.0 s.
- **The copy at create** costs 150–290 ms on the desktop per create, the disc's parse being constant in its size
  (`emusen-disc`'s `Disc::from_image` places the tracks and reads nothing): it is the copy into freshly faulted pages,
  as `EmuSen_CoreAPI.md` §6.2 requires ("the core copies what it keeps"). Host and core hold the image twice while
  create runs.
- **What the host could do without the ABI is done** (`Diamond_Players.md` §12.8.3): the firmware question reads no
  image, and big picture loads under its launch screen. After it a cold launch is held only by the read: from the pad's
  press to the first frame 5.1–9.4 s for the loose tracks and the zips' unpacked tracks under the 90 MB/s cap, against
  the launch screen's 3.0 s; and with the screen Disabled, or from the desktop's list, the read is waited for with
  nothing moving on screen.
- **A game reads little of its disc at launch.** The first frame reads no sector. In their first 30 s (1,800 frames,
  on the open kernel, through the run tool's drive trace) Crash Bandicoot read 1,258 sectors, Spyro the Dragon 997 and
  Ape Escape 1,537: 2.3–3.6 MB, under 1% of each image.

So the copy at create matters where the medium is slow, which is where the project's weak-machine target keeps its
library, and the cure is to read the sectors as the game asks for them [A].

#### 10.2.2 Two shapes

Both keep v1's rule that the core never calls the host, both are a minor's addition by `EmuSen_CoreAPI.md` §4.4, and
both leave a host that does not offer them, and a core that does not claim them, exactly as now [A].

- **A. The core asks, the host supplies** (the shape recorded above). A capability bit; create receives the image's
  length and each companion's, without their bytes; the core raises an event naming a file, an offset and a length; the
  host answers with an export supplying the bytes before the next advance. The core must ask ahead of its emulated
  drive, by more than the longest the host may take: at the drive's double speed 150 sectors a second, so a second's
  read-ahead is 350 KB, about 4 ms from the card. It is the more general of the two (a network source, a medium the
  host decodes), and the larger: a new event, a new export, a read-ahead policy in every disc core, and the
  emulation's timing made to depend on the host answering in time, which a test must pin.
- **B. The image borrowed for the machine's life.** A capability bit by which the host promises that the image and the
  companions passed at create stay readable, unchanged, at the same addresses until `destroy`, and the core may keep
  pointers into them instead of copying. The host maps the files read-only rather than reading them; the core's sector
  reads touch the mapping, and the system reads the pages from the card on first touch, ahead by its own read-ahead.
  No event, no export, no read-ahead in the core; one disc crate's source kind (`emusen-disc`'s `Source::Bytes` gains a
  borrowed form) serves every disc core. Its costs: a page not yet read is read on the emulation thread, the system's
  read-ahead (128 KB by default) covering the drive's sequential reads; a file changed or truncated under a running
  machine is seen by it, which the host prevents only by holding the files open and refusing to unpack over a mapped
  image; and the promise binds every host, the kit's included.

#### 10.2.3 Recommendation, and the decision

*Recommended* [C, Q15]: **B**, as the smaller change that removes both the cold read and the doubled memory, with a kit
case that creates from a mapping, runs, and compares every frame, sample and state with a run from copied bytes; A
kept as recorded, for a source a mapping cannot serve. Prediction **P7** (§11) states what B must show.

**Decided 2026-10-12: shape B is built, as minor 1.3, with §10.5's battery files in the same minor.** The tester asked
with it that disc images be handled in one shared place, so that no later disc console writes this again, and the
ownership was decided the same day:

| Owner | What it owns | Where |
|---|---|---|
| **`emusen-disc`** | what a disc is and how its bytes are read: the formats, the sectors, CHD, what a set means, the kinds of backing, and opening a disc from an image and its companions | `EmuSen/Cores/Shared/emusen-disc`, beside the core kit; never depends on Galaxia, since a core builds and runs with no frontend |
| **Galaxia** | where a disc's files are and keeping them available: a sheet's companions, an `.m3u`'s discs, the unpack cache for archives, and the files mapped read-only and held for a machine's life | `EmuSen.Galaxia/Media/` and `Platform/galaxia`; it asks `emusen-disc` for format knowledge through the platform library, never the reverse |
| **Crystal** | which system a disc is for: `SystemRouting`, the packs' recognisers, `MediaProbe` and `DiscSectors`, reading through Galaxia | `EmuSen.Crystal` |
| **The core ABI** | the loan's words | `emusen_core.h`, minor 1.3 |

#### 10.2.4 The design (minor 1.3)

*Written 2026-10-12 before the code and committed first; built the same day as written, but where §10.2.5 says
otherwise. §10.2.5 is the record of what was built and measured.* Marks as above.

**1. The ABI's words.** One capability bit, one appended field and two flag values; no export, no event, no status.

```c
#define EMUSEN_CAP_BORROW    (1ull << 20)  /* @since 1.3: the core keeps pointers into a lent image and its media files */
#define EMUSEN_CREATE_LENT   1u            /* @since 1.3: emusen_create_params.flags */
#define EMUSEN_MEDIA_LENT    2u            /* @since 1.3: emusen_media.flags, beside EMUSEN_MEDIA_RESTORE */

typedef struct emusen_create_params {      /* 96 bytes; 88 at 1.0, which a callee still accepts */
    /* ... the twelve fields of 1.0, unchanged ... */
    uint32_t flags;                        /* @since 1.3: EMUSEN_CREATE_LENT; zero from a host that predates it */
} emusen_create_params;
```

- **The loan.** A host that sets `LENT` promises that the image, and every file of the call numbered
  `EMUSEN_FILE_MEDIA` or above (a sheet's tracks, an `.sbi`), **stay readable, unchanged and at the same addresses until
  the loan ends**. The core may then keep pointers into them instead of copying, and may read them from any thread of
  the machine at any time the machine exists. Every other pointer of the call (the settings, the battery and firmware
  files, the error buffer, the structs themselves) is valid for the call only, as in 1.0.
- **The loan's end.** A loan made at `create` ends when `emusen_core_free` returns, or when a `media_insert` on the
  drive holding that disc returns success, whichever is first; the disc `create` receives is in the first drive machine
  info lists. A loan made at `media_insert` ends the same way, at `free` or at that drive's next successful insert. **A
  refused call lends nothing**: a core that refuses a create or an insert keeps no pointer from it, and a refused
  insert leaves the drive's earlier loan standing with its disc. A null image, which empties a drive, ends that drive's
  loan and makes none.
- **How the promise is carried: a flag in the request, on each call.** The bit says what the core can do, once, for
  the library; the flag says what this host does, for this call. A core therefore knows on every create and every insert
  whether it may borrow, and nothing is inferred from the host's version, which gates nothing (`EmuSen_CoreAPI.md`
  §4.1).

  | Host | Core | What happens |
  |---|---|---|
  | sets `LENT` (only for a core that claims `BORROW`) | claims `BORROW` | the core keeps pointers; the host keeps the files mapped until the loan ends |
  | does not set it (an older host, or one that read the bytes into a buffer of its own) | claims `BORROW` | the flag is zero, or absent in an 88-byte struct, which reads as zero: **the core copies what it keeps, as in 1.0** |
  | any | does not claim `BORROW` | the host does not set the flag; the core copies what it keeps, as in 1.0. The host may still pass a mapping's address as the image, which removes the host's own buffer and nothing else |

  A host must not set `LENT` for a core that does not claim `BORROW`: nothing would go wrong, but the promise would bind
  it for nothing. A core must not keep a pointer from a call without `LENT`, whatever it claims; the kit holds it to
  that by overwriting its own buffers after such a create (C17, item 12).
- **Why a field and not an export.** A second `create` behind the bit was weighed and rejected: it would double the
  lifecycle's most intricate call to carry one bit. `emusen_media` has had `flags` since 1.1; `emusen_create_params` gains
  it now. This is the first use of §4.2's rule for a struct that grows (`EmuSen_CoreAPI.md` §4.2: "every struct begins
  with `size` … the callee reads and writes only the prefix both know … a struct shorter than its v1.0 size is
  refused"), and the first to test it: `emusen-native` read a whole struct of its own size, which is correct only
  while no struct has grown, and now reads `min(size, its own)` into a zeroed copy.
- **The baseline.** A struct's `struct … size … align …` line is its size at the minor that introduced it, the least a
  callee accepts, and is not edited; the appended field's own `field` line, tagged `@1.3`, records the growth. The guard
  holds the header's size to be no smaller than the line's and its alignment the same, and every earlier field line
  holds the prefix (`EmuSen_CoreAPI.md` §5.4).
- **No state holds a pointer.** A state's bytes are the same whether the disc was lent or copied, and a state made on
  one loads on the other.

**2. The kit's type.** `emusen-native` gives a core `core::keep(bytes) -> Kept`, called inside `create` or
`media_insert` on the image or a media file (or any part of one): **the bytes to keep for the machine's life, the
host's own where this call lends them and a copy otherwise.** A core calls it wherever it would have written
`to_vec()`, and its own code never tests the flag and never holds a raw pointer. `Kept` dereferences to `[u8]`, is
`Send` and `Sync`, and is made from a `Vec<u8>` for a test or a tool. The `unsafe` is in `Kept` alone, and its
invariant is checked rather than trusted: each lending call makes a *loan*, every `Kept` made under it holds a
reference to it, and the export that ends a loan (`free`, a successful `media_insert` on its drive, a refusal) counts
the references left; one still held is a defect in the core, and the export panics with the loan named, which aborts
the process before the host can unmap anything, so no safe code in a core can read bytes the host has released.

**3. `emusen-disc`'s backing.** `Disc<B>` for any `B: AsRef<[u8]>`: a `Vec<u8>` (the default, and every existing
caller), a `&[u8]` for a probe that opens a disc for one call, or the kit's `Kept`. The crate does not depend on
`emusen-native`; the kit's type meets it through `AsRef<[u8]>` alone. One code path reads every backing, so a disc
opened from borrowed bytes reads sector for sector as one opened from owned bytes, which the crate's tests hold for
every format it has (`EmuSen_Disc.md` §9). The crate also takes what is not PlayStation's from Diamond's `medium.rs`:
opening a disc from an image and its companions (a sheet with its tracks, a lone track, a user-data image, a CHD, an
`.sbi` among the companions), and the fingerprint's arithmetic, the console naming only which sectors it covers.

**4. Galaxia's part.** `EmuSen.Galaxia/Media/`: `ImageSets` and `ArchiveImages`, moved from the runtime's media module
(`EmuSen.Crystal/Media/`), `MappedFile`, a file mapped read-only, and `DiscFiles`, a disc's image and companions
mapped and held, with the unpack folder they came from. `MappedFile` has both of Galaxia's halves, the C# over
`System.IO.MemoryMappedFiles` and the platform library's (`Platform/galaxia/src/mapped.rs`, interface 9), held to one
answer by `GalaxiaParityTests`; the kit, which has no C#, maps through the same Rust module. `CoreEngine`'s open path
asks Galaxia for a disc's files and hands them over; it reads no image itself.

**5. Archives.** A zip still unpacks once into Galaxia's `Unpacked` folder, and the unpacked files are what is mapped
(§10.4). **A folder in use is never unpacked over or cleaned away**, enforced in two layers: `Unpack` never writes
into a folder that has its marker (it unpacks beside it and moves the result into place whole, as before), and
`DiscFiles` holds a *lease* on the folder for as long as it holds a mapping in it, a handle open on the folder's marker
that shares reading and not deletion. `Prune` removes a folder only after it has opened the marker with no sharing,
which fails while any process holds a lease, on Windows by the file system's own rule and elsewhere by the advisory lock
.NET takes for the same call. A process that does not use Galaxia can still delete the folder; on Linux and macOS the
mapping stays readable until it is released, since the file's blocks outlive its name.

**6. CHD.** The compressed file is what is mapped; the `chd` crate reads its header, map and metadata at open and
`emusen-disc` decodes one hunk at a time into its cache, as before. Only the hunks a game reads are touched.

**7. Sets and Change Disc.** One path: `InsertDisc` asks Galaxia for the next disc's `DiscFiles`, passes it with
`EMUSEN_MEDIA_LENT`, and **releases the previous disc's files only after `media_insert` has returned success**, which
is when the core has let go of them (item 1). A refused insert releases the new disc's files and keeps the old. The
loop that puts a state's disc back (`EMUSEN_MEDIA_RESTORE`, §3.5) is the same call with both flags.

**8. Save states, rewind and the fingerprint.** A state holds no disc bytes and no address. The fingerprint a state
records (DMND 9) is FNV-1a over the tracks' starts and kinds, the lead-out and nine sectors (4, and 16 to 23): a read of
21 KB whatever the disc's size, two hunks of a CHD. It is kept bit for bit, so that every state made before 1.3 loads,
and a test holds the bound by counting the bytes a fingerprint touches.

**9. Threads.** A lent page not yet in memory is read from the medium on the thread that first touches it, which is
the emulation thread. Two cases are to be measured on the slow medium, under the 90 MB/s cap and on the handheld's
card: a cold sequential read at the drive's double speed, and a seek to a region never read. If either can hold a
frame past its period, the remedy is the host's and changes nothing a core can observe: advice to the system that the
mapping is read in order, so that it reads ahead of the drive on its own threads, or a reader thread of Galaxia's that
touches pages ahead. Either way a read returns the same bytes; only when they arrive in memory differs, and the
emulated drive's timing is the core's own count of cycles, never the wall clock.

**10. Platforms.**
- *Linux and macOS:* `mmap`, read-only. A mapped file may be renamed or deleted while a game runs and stays readable.
  Nothing stops another program truncating or rewriting it; the machine then reads the new bytes, or the process is
  ended by the system's signal for a mapped page that no longer exists. **The same holds when the medium is removed**: a
  game whose disc was copied at create survived its card being pulled, and one whose disc is mapped does not. This is
  the cost of reading in place and is put to the tester with the measurements.
- *Windows:* a file mapping. While a game runs its image and tracks cannot be deleted, renamed, moved or written by
  the player or another program, which answer that the file is in use; they are released when the game is closed or
  its disc changed.
- *All three:* the address space a mapping takes is not memory in use; what counts against the machine is the pages
  the game has touched, and the system may reclaim those at will, since they are the file's own.

**11. The firmware question, and routing's probes.** `firmware_for` is passed the mapped image, and the core opens it
as a borrowed disc for the call, so the question touches the sheet, or the sectors that name the market, and nothing
else. A recogniser's probe reads the first bytes of the image, of a sheet's first track or of an archive's entry
through Galaxia's `DiscFiles` and `ArchiveImages`, never a whole file.

**12. The kit.** A new case, **C17, borrow** (where `BORROW` is claimed): the image and its companions mapped, created
with `LENT`, run for C6's frames with the script's input, and every frame's digest, every sample's and the states'
compared with a run from copied bytes; the same again through `media_insert` where `MEDIA` is claimed; and a create
without the flag after which the kit overwrites its own buffers, the run's digests unchanged. C12's skew host passes
an 88-byte `emusen_create_params`, and a struct longer than the core's with bytes past it that the core must not read
as its own. C2 checks the bit's name in core info.

#### 10.2.5 What was built and measured (2026-10-12)

[M] here is measured or run on 2026-10-12, on this branch.

**Built as §10.2.4 gives it**, items 1 to 8 and 10 to 12, with these differences and findings:

- *The loan's check* (item 2) is `emusen_native::core::lend`; `Kept` and `keep` are re-exported from `core`. A test
  holds that a slice inside a lending call's ranges is lent and one outside them (a battery or firmware file) copied,
  that nothing is lent outside a call, and that a loan ended while a piece is held panics with the loan named.
- *The struct's growth* (item 1) needed `emusen-native` to read a host's struct as far as both know it: until 1.3 it
  read a whole struct of its own size, correct only while no struct had grown. A test creates from 88 bytes whose
  flag's bytes say `LENT`, and the core copies; one byte fewer is `BAD_STRUCT`.
- *The guard* keeps `struct emusen_create_params size 88 align 8 @1.0` and records `field emusen_create_params.flags
  offset 88 @1.3`; it now refuses a struct that shrinks, changes alignment or grows with nothing appended
  (`a_struct_grows_only_by_appended_fields_and_keeps_its_line`). The new baseline has 423 facts, 9 lines added at 1.3
  and the revision's two moved; `--check --against` the base branch's baseline passes.
- *The kit* (item 12) gained C17 as written, and C14 checks core info's battery list against machine info and creates
  with every listed file empty. **C13 had a race**, shown by the kit's own tests in parallel: its four threads could
  make no round before a busy machine's run ended, so a seeded drift of the library-level answers passed. The case
  now starts the run once each thread is calling and ends it once they have called beside it; three parallel runs of
  three pass.
- *Galaxia's half* (item 4): the parity test found the library answering a folder with the system's "no such device"
  where .NET refuses access; it now refuses one as .NET does. .NET's advisory locks are not seen by a mapping the
  library makes, so `DiamondPlayersTests`' proof that the firmware question reads no image, which held the track under
  an exclusive .NET lock, now moves the track away instead.
- *Threads* (item 9) are measured below, and needed the prefetch the item foresaw.

**Exactness** [M]. The kit's C17 on the seven test discs at 1,800 frames, the image and tracks mapped through Galaxia
and lent, passes on all seven: every frame's picture, every sample and the state after every frame are those of a run
from copied bytes, across a disc swap of 120 frames more; and a create that lent nothing, its bytes overwritten by the
kit, runs unchanged. WiseMan's `DiscStreamingTests` hold the same through the C# host: a sheet with its tracks, a lone
track and an `.iso` on Diamond and the test core's image, 300 to 600 frames, with the prefetch running. The same
corpus run found C7 and C8 failing on four of the seven at 1,800 frames (a state saved at frame 900 and loaded gives
another picture, the sound and state the same); the base branch's library and kit fail the same cases with the same
digests to the bit, so it is a defect of Diamond's states that predates 1.3, recorded here for the branch that owns
the states (at 600 frames, `Diamond_Players.md` §6, they passed).

**Launch and memory** [M]: `Diamond_Players.md` §12.8.9 has the tables. In brief: a launch of 1.3 from the press of the
engine's steps to the first frame is 106–126 ms on the desktop, cold or warm and under the 90 MB/s cap alike, and
95–120 ms on the handheld from its card, where 1.2's took 4.7–7.1 s cold; peak resident 53–71 MiB, where it was
878–1,382 MiB; a zip's first launch is its unpacking, which 1.3 does not change.

**First touches** [M]. Traced over the first 30 s (1,800 frames) from a cold cache, unpaced, each frame's time and
major faults: on the desktop under the cap, 2 to 8 major faults in 30 s on six games, at most 7 ms a frame; Crash
Bandicoot's one fault at frame 282 held its frame 65–80 ms in three runs of three, and 3.9 ms without the cap. On the
handheld's card, Mega Man X5's frame 355 took 20.6 ms with one fault, where its emulation takes 2.7 ms: **a first
touch can hold a frame past its period.** The remedy is item 9's, and is built: `DiscFiles.Prefetch`, which the engine
starts for a lent disc at the load and at an insert, reads each of the disc's files once in order, 256 KiB at a time,
through a handle of its own on a background thread of the lowest priority, resting after each read twice as long as
it took, so that it takes at most a third of the medium. At full speed it made the game's one fault in 30 s wait
160 ms behind it under the cap; rested, paced at 60 frames a second, Crash Bandicoot's faults fell from 8 to 1 and
Spyro's from 3 to 0, every frame within 6.1 and 7.5 ms. What a read of the mapping returns is the file's, whether
the prefetch has passed or not.

**On the handheld's card, paced** [M, `Diamond_Players.md` §12.8.9]: with the prefetch Mega Man X5 and Spyro the Dragon
play their first 30 s with no first touch and Crash Bandicoot with one of 6.6 ms, where they had 4, 3 and 16 of up to
13.1 ms. Mega Man X6's four to six first touches come two to six seconds into play, before a prefetch at a third of
the card has reached them: the slowest held a frame 8.0–31.5 ms in four runs without the prefetch and 23.8–49.6 ms in
four with it, zero to three frames over the period in 30 s either way. **So the design's item 9 is met in part: a
first touch still can hold a frame in a game's first seconds on a slow medium, and the prefetch, which removes the
later ones, appears to lengthen those.** Two remedies are recorded and not built, the choice being the tester's:

- *host only:* the prefetch at the medium's full speed until the machine's first frame, and rested from then; under
  the launch screen it would have read about 270 MiB before play. It narrows the window and closes it for no game in
  particular.
- *exact, and a surface:* an event by which a core says where its drive is seeking (a file and an offset), raised at
  the seek's start, so that the host advises the system to read there during the emulated seek, which on the
  PlayStation's drive takes far longer than the card's read. It is shape A's event without its export: the bytes
  still come through the loan, nothing waits on the host, and a host that ignores the event behaves as now. It would
  be a minor of its own.

*The exact remedy was chosen and built as minor 1.4 (§10.2.6, §10.2.7), and the prefetch removed with it.*

#### 10.2.6 The drive's place named ahead: the design (minor 1.4)

*Decided 2026-10-12: of §10.2.5's two remedies the tester chose the exact one, a core event naming where its drive
is going, the host reading ahead during the emulated seek, over the prefetch at full speed, because it is exact
whatever a game reads first and the other only narrows the window. Written 2026-10-12 before the code and committed
first; §10.2.7 is the record of what was built and measured.* Marks as above; [M] here is measured on 2026-10-12 with
the scratch tools named, on the desktop and on the handheld's card, before anything of the design was built.

**What was measured first** [M]. The run tool, with a trace of each seek's start and each sector the head reads
added for the measurement and not kept, ran Mega Man X5, Mega Man X6, Crash Bandicoot and Spyro the Dragon for their
first 1,800 frames on the open kernel:

- *A seek's start is known long before its first read.* Of 111 seeks with a read after them, the first read came in
  the frame the seek started for 1, in the next frame for 24, and two to sixteen frames later for 86. The one in its
  own frame was a seek of no distance, to the place the seek before it had named. Every seek farther than 87 sectors
  (41 of them) had its first read at least one frame's end later and 23.5 ms or more of the game's time later; every
  one farther than 822 sectors (34), at least two frames' ends later. The figures follow from the drive
  (`Diamond_CDROM.md` §5): a jump takes 400,000 cycles and more, 11.8 ms, and a read's first sector a period after
  it, 6.7 ms at double speed, together 18.5 ms against a frame of 16.7 ms; only a seek that ends where the head holds,
  whose target the held head reads at the seek's end, or a seek of a few sectors along the spiral, can be read in
  the frame it starts, and then only within 87 sectors, 205 KiB, of where the head was [A].
- *What a game reads after a seek.* The median read after a seek is 1 to 26 sectors by game; the long ones are 920
  and 2,633 sectors (Mega Man X5), 3,452 (Mega Man X6, 8 MiB at the drive's 345 KiB a second), 2,495 (Spyro) and 192
  (Crash Bandicoot).
- *The system's advice* (`madvise` with `MADV_WILLNEED` on the mapping, `willneed.py`). The call returns before the
  bytes arrive but is not free: 0.1–2.8 ms on the desktop by the range's length, 0.3–0.6 ms on the handheld. **One
  call reads no more than the device's largest request**: 4.1 MiB on the desktop's drive and 512 KiB on the
  handheld's card, whatever was asked, and the range arrives whole: 64 KiB after 2.3 ms on the card, 256 KiB after
  4.4 ms, 512 KiB after 7.3 ms. A cold page read by touching it took 2.5–3.1 ms on the card in fifty readings after
  rests of 0.2 to 10 s (`rest.py`); in another run of eight, 2.6–5.9 ms in seven and 27.9 ms in one.
- *A read that runs past what was advised* (`readon.py`, 8 MiB read on from a place, 2 MiB of it advised first): on
  both machines exactly one read waited for the medium, at the advised range's end (2,046–2,099 KiB), 1.6–4.3 ms, and
  none after it: the system's own read-ahead starts at a miss and then keeps ahead, but an advised range ends with no
  mark to start it, so the miss is paid once.

**1. The event.**

```c
EMUSEN_EVENT_MEDIA_SEEK = 8   /* @since 1.4. a: (drive << 32) | file; b: the byte offset in that file */
#define EMUSEN_FILE_IMAGE 0xFFFFFFFFu  /* @since 1.4: in MEDIA_SEEK's `a`, the image itself, which no emusen_file numbers */
```

- **What it says.** Drive `drive` (machine info's `drives[].drive`) is about to read file `file` of its disc at byte
  `offset`: `file` is an `emusen_file.which` of the call that gave the drive its disc, `create` or the last
  `media_insert` to succeed on it, so `EMUSEN_FILE_MEDIA + n` for a sheet's nth file, or `EMUSEN_FILE_IMAGE` for the
  image where the image is the disc (a lone track, an `.iso`, a `.chd`). The drive and the file share `a` as §4's
  `OUTPUT` shares a port and an output; `b` is the whole offset. Kind 7 stays §4's `EMUSEN_EVENT_OUTPUT`, decided and
  not yet built.
- **No length** [C]. How far past a place to read is the medium's question, its speed and its latency, and the
  host's pacing, none of which a core knows; what a core knows is where its head is going, and it says that. In its
  place the event carries a rule a host can size a window by, the next item.
- **When a core raises it** [C]. (a) At a seek's start, for the seek's target: the earliest moment the place is
  known, with the emulated seek still to run. (b) While the drive reads or plays on, **before the head reads a place
  1 MiB or more past the offset last named in that file, or in another file**: without this a long read runs off the
  end of what a host advised and waits once for the medium, as measured above. (c) A place within the megabyte after
  the one last named need not be named again. A host that reads at least 1 MiB and its own delay past each named
  place therefore always has the head inside what it advised. The megabyte is three seconds of a double-speed
  drive; it is a number of bytes and not of sectors or seconds so that a host needs nothing of the drive's speed.
- **A core without the knowledge raises nothing**, and so does a core for a sector its image does not hold (a pause
  a sheet does not store, the lead-out). A core may raise it whether or not its disc was lent; a host that lent
  nothing holds no file to read ahead in and skips it, which keeps the flag out of a core's drive [C].
- **A hint, never state.** Nothing the machine does depends on the event having been raised, drained or acted on:
  the emulated drive's timing is the core's own count of cycles, the bytes come through the loan as in 1.3, and no
  state holds what was named. After a state load or a disc change a core names the next place it reads as if nothing
  had been named.

**2. No capability bit** [C]. `EmuSen_CoreAPI.md` §4.2 has a host skip a kind it does not know, so a bit is owed
only where a host must know *before* the first event. It need not: it acts on each event as it comes, holds nothing
ready beforehand (the reader's thread starts at the first event), and a core that never raises one leaves it exactly
as at 1.3. An older host skips kind 8; an older core raises none. No bit is taken, and bits 21–47 stay free.

**3. When the host learns of it** [M, A]. Events are drained by the host, and the reference host drains them once a
frame, after `advance`, the frame-end work and `present` (`CoreEngine.RunFrame`); `emusen_core_events` with a null
buffer costs one call and no allocation when nothing waits. So a place named during frame *n* is heard of at frame
*n*'s end, and the question is whether its first read can come before that. By the measurement above it cannot,
but for places within 87 sectors of the head; those lie inside what was advised for the place the head came from,
given item 4's window, so nothing is read in the frame that first named it [A]. Past the frame's end the host has
the rest of the emulated seek in real time: paced at 60 frames a second, with a frame's emulation taking *c* ms of
its 16.7, a first read *k* frames' ends after the seek's start leaves the host at least 16.7 *k* − *c* ms. For the 24
seeks of *k* = 1 that is 12 ms or more at the handheld's median frame (1.3–4.4 ms by game, §10.2.5's traces) and 7 ms
at its 99th percentile (5.0–9.6 ms); for every seek farther than 822 sectors, 24 ms or more; against 2.5–3.1 ms for
the card's usual read of a cold page and 2.3–4.4 ms for an advised piece of 64 to 256 KiB.
**The drain's timing therefore does not change**, and nothing is added per frame. Unpaced, in fast forward, the lead
shrinks with the frame's time and a read may be late, as it is today.

**4. What the host does** [C].
- *Where.* `CoreEngine.Drain` routes the event: it finds the drive's lent `DiscFiles`, the file by the numbering it
  gave (`MediaOf`'s), and calls `DiscFiles.ReadAhead(file, offset)`. That posts the request to the disc's reader and
  returns: a lock and a signal, no allocation, no call into the system. The engine names no path and Galaxia no
  ABI constant.
- *On which thread.* A reader thread of the `DiscFiles`, started at the first request, because the advice call is
  not free (0.3–0.6 ms a piece on the handheld). **Nothing in the machine waits on it**: the emulation thread never
  joins it, never reads what it wrote and takes no lock it holds across a call into the system.
- *What it advises.* The range from 256 KiB before the named offset to 2 MiB past it, clipped to the file, in pieces
  of 128 KiB, the piece holding the offset first, then onward, then the part before it. 128 KiB because one call
  reads no more than the device's largest request and the system's read-ahead size, 128 KiB by default, is the least
  of those a host can count on; 2 MiB past because item 1's rule bounds the head to 1 MiB past a named place and
  the second megabyte is the three seconds of reading the host may take to hear of the next; 256 KiB before because
  the held head steps back up to 8 sectors and a seek that ends in the frame it starts reaches back at most 87. A
  newer request replaces one not yet finished between two pieces, so a second seek waits behind at most one call.
- *Through what.* `MappedFile.Advise(offset, length)`, in both of Galaxia's halves: `madvise` with `MADV_WILLNEED`
  on Linux and macOS and `PrefetchVirtualMemory` on Windows, in the C# over the mapping it holds and in
  `Platform/galaxia/src/mapped.rs` (`emusen_galaxia_mapped_advise`; the platform library's interface goes from 9 to
  10). One call each; a range outside the file is clipped; an empty file has nothing to advise; a refusal by the
  system is not an error, since the read then happens at the touch as before.
- *Exactness.* Advice changes when a page arrives in memory and nothing else: a read of the mapping returns the
  file's bytes whether advised or not, so a run with a host that acts on the event is bit for bit the run with one
  that skips it. The kit holds it (item 7).

**5. Reads that follow no seek** [C, to be confirmed by §10.2.7]. Item 1's rule (b) is the answer: the same event,
raised as the head moves on, so that every place a drive reads has been named, by a seek or by the read before it.
The system's own read-ahead was weighed and is not relied on: it costs one wait at the end of each advised range on
Linux, as measured, and what a mapped view reads ahead on Windows and macOS was not measured at all. **1.3's rested
background prefetch** (`DiscFiles.Prefetch`) then has nothing left to do that the event does not do sooner and
without reading the whole disc: it is removed if §10.2.7's traces show no first touch the event leaves and it would
have caught, and kept, with what it is for said, if they show one.

**6. Loss.** The queue holds 256 events and drops the oldest when full, queueing `MACHINE_INFO`. A lost
`MEDIA_SEEK` costs a late read of that place and nothing else; the next one is acted on as usual. A host that drains
once a frame meets a full queue only if a core names 256 places in a frame, which rule (c) keeps it from.

**7. The core's side, the disc crate and the kit** [C].
- *`emusen-native`*: `core::seek(drive, file, offset)`, `file` being `None` for the image or `Some(n)` for the nth
  media file, so the numbering has one owner; it queues the event and returns. A machine's event queue is allocated
  whole at create, so raising one allocates nothing.
- *`emusen-disc`* owns "which file and byte hold this sector": `Disc::place(sector)` answers the file, by its
  position among those the disc was opened from or the image itself, and the byte, or nothing for a sector the image
  does not store. For a `.chd` it answers the compressed hunk's offset in the file, from the `chd` crate's map
  (`Map::get_entry`, a compressed or uncompressed entry's `block_offset`), which is where the system must read
  before the hunk can be decoded; a hunk that is a copy of another names nothing. The megabyte's rule then counts
  the file's bytes, which a compressed disc spends more slowly than a raw one.
- *Diamond* raises it from its drive: at `start_seek` for the target, and at each sector read or played for the
  place 75 sectors ahead of the head, half a second at double speed, whenever that place is not in the megabyte
  after the one last named. The look-ahead names a sheet's next track file before the head crosses into it.
- *The kit*, **C18, the drive's place** (run with the image cases): the image mapped and lent as C17's third leg
  lends it, C6's frames run twice with the events drained after every frame, once skipping `MEDIA_SEEK` and once
  advising the mapped file at each as a host does. Every `MEDIA_SEEK` raised must name a drive machine info lists, a
  file the call passed (or the image) and an offset inside it; and the two runs' pictures, sound and states must be
  equal frame for frame. A core that raises none passes with that said. A seeded fault, a core naming an offset past
  its image's end, fails C18 alone.

**8. Prediction P8** (§11): with the event, no frame is held past its period by a first touch on the handheld's card
in a game's first 30 s, on Mega Man X5, Mega Man X6, Crash Bandicoot and Spyro the Dragon, paced, cold; at least six
runs each way of Mega Man X6, every run reported. A frame counts as held by a first touch when it took longer than
16.7 ms and the emulation thread waited in it, by a major fault or by time off the processor that was not spent
waiting to run; a frame that was only slow is reported beside it.

#### 10.2.7 What was built and measured (2026-10-12)

[M] here is measured or run on 2026-10-12, on this branch, after WiseMan `a253a250` was merged into it.

**Built as §10.2.6 gives it**, items 1 to 7, with these differences and findings:

- *The header and the baseline*: `EMUSEN_EVENT_MEDIA_SEEK` (8) and `EMUSEN_FILE_IMAGE` (`0xFFFFFFFF`), the minor 4;
  the baseline has 425 facts, two lines added at 1.4 and the revision's two moved, and `--check --against` the base
  branch's baseline passes. No bit, no export, no struct.
- *`emusen-native`*: `core::seek` and `SEEK_SPAN`. Raising an event into a full `VecDeque` had allocated once per
  machine, on the queue's first growth; the queue is now allocated whole at create, and a test holds that 768 events
  through a full queue leave it where create put it.
- *`emusen-disc`*: `Disc::place` (`EmuSen_Disc.md` §10). For a CHD it answers the stored hunk's offset from the
  `chd` crate's map, its documented API, the crate's source not read; a hunk copying another names nothing.
- *Diamond* names at `start_seek`, and 75 sectors ahead of the head while reading or playing. Building it found a
  third case the design had not: **the held head** reads the header of the sector it passes over, a few sectors to
  either side of the place last named, and named the place behind it each time the head stepped back; it now names
  its place only where nothing within a megabyte of it was named (`Diamond_CDROM.md` §5).
- *The host*: `DiscFiles.ReadAhead` and its reader thread, `MappedFile.Advise` in both halves (interface 10),
  `CoreEngine`'s routing through `CoreMachine.LoanOf`; `EmuSen_Galaxia.md` §3.9.
- *The kit*: C18 as written, with seeded fault 512.

**Exactness** [M]. The kit's C18 on the seven test discs at 1,800 frames passes on all seven, and so does every other
case, C7 and C8 among them now that Diamond's state holds its picture (DMND 10, from WiseMan): every frame's picture,
sound and state the same with the places acted on and skipped. The places named and the bytes advised in 30 s:

| Disc | Places named | Advised |
|---|---|---|
| Ape Escape | 17 | 40,078 KiB |
| Castlevania: Symphony of the Night | 14 | 32,641 KiB |
| Crash Bandicoot | 22 | 51,321 KiB |
| Rayman | 12 | 24,145 KiB |
| Silent Hill | 20 | 47,281 KiB |
| Spyro the Dragon | 12 | 27,933 KiB |
| Tekken 3 | 18 | 41,861 KiB |

WiseMan's `DiscStreamingTests` hold the same through the C# host on a synthetic sheet, and the run tool's digests
(`bench`) are the base branch's on all three of the cost's workloads below.

**First touches** [M, `Diamond_Players.md` §12.8.10 has every run]. *Decided 2026-10-12: the measurement is the
desktop's, under the 90 MiB/s cap; its confirmation on the handheld's card is left to play.* The first 30 s of play
(1,800 frames), paced at 60 frames a second, from a cold cache, under the cap, four ways in turn: 1.3 as shipped (the
prefetch), 1.4 (the event acted on), both, and neither. Six runs each way of Mega Man X6, three of Mega Man X5, Crash
Bandicoot and Spyro the Dragon; load averages 0.8–3.7, other work running on the machine.

| Game | 1.3, the prefetch: runs with a frame held, the held frames | 1.4, the event: frames with a major fault | Neither: frames with a major fault, the slowest |
|---|---|---|---|
| Mega Man X6 | 5 of 6: 194.7, 116.2, 47.4, 130.3, 27.0 ms | 0 in every run | 3–4 a run, 4.9–6.5 ms |
| Mega Man X5 | 0 of 3 | 0 in every run | 2–3 a run, 4.8–8.0 ms |
| Crash Bandicoot | 1 of 3: 131.7 ms | 0 in every run | 8 a run, 5.4–5.8 ms |
| Spyro the Dragon | 0 of 3 | 0 in every run | 3 a run, 6.8–7.7 ms |

**With the event no run of any game took a major fault in 30 s**, and its slowest frame was 4.4–8.7 ms. "Both" was the
event's figures. The prefetch held a frame in six of nine runs of the two games whose first touches come before it
has passed them, each with one major fault and 23–191 ms of the emulation thread's off the processor: the fault's read
waited in the cap's throttle behind the bandwidth the prefetch had spent. That is the cap's own way of sharing the
medium and exaggerates the card's, where the same frames took 24–50 ms (§10.2.5) [A].

**Reads that follow no seek** [M]. Mega Man X6's run of 3,452 sectors from frame 422 on was named at frames 571, 749,
927 and so on to 1,638, every 1,048,992 bytes, by the read's rule (§10.2.6 item 1 (b)); it, X5's run of 2,633 sectors
and Spyro's of 2,495 took no fault. So the event covers
the long reads and the seeks alike, and **the prefetch is removed** (§10.2.6 item 5): it did nothing the event did not
do sooner, and under a capped medium it made the game's own read wait behind its own.

**What the desktop's cap did not model** [M, A]. Under the cap a cold page took 1.9–3.3 ms, as on the card (2.5–3.1
ms), and the sustained rate is the card's. A short read is not: an advised 512 KiB arrived in 1.4 ms on the desktop
and in 7.3 ms on the card, since the cap delays a cgroup's requests only once it has spent its slice's allowance. A
request-rate cap was tried beside it to add the card's time per request, and does not model it: at 150 and 250
requests a second it held single reads 50–154 ms, as the throttle's slice is spent, where the card's never exceeded
28 ms. The argument for the card is therefore §10.2.6 item 3's: the reader's first piece, the one holding the place,
takes the card 2.3–4.4 ms, and every seek farther than 822 sectors leaves 24 ms or more of real time before its first
read, the nearer ones being inside a window already advised.

**Cost** [M]. Aligned builds (`RUSTFLAGS="-C llvm-args=-align-all-functions=6"`), the run tool's `bench`, fastest of
five, alternating with the base branch (`a253a250`), load averages 2.8–8.2: `gpu/benchmark` (1,200 frames) 2.321 →
2.298 ms a frame (−1.0%), Crash Bandicoot's first 3,000 frames 2.306 → 2.299 ms (−0.3%), Tekken 3's 2.370 → 2.375 ms
(+0.2%): within the noise, the picture, sound and state digests the same. The host's part is one drain a frame, which
it made before, and a lock and a pulse for each place named, 12 to 22 in 30 s.

### 10.3 The order

By the first stage that needs each [C, Q13]:

| When | Items |
|---|---|
| Now, before any disc core | §6 (the two comparison types and the codec's place); §7 (the kit's estimate); §8's service on cartridge consoles, held to the registration golden |
| Diamond stage 6 (SIO0) | §4 outputs; §5 extras; §9 `kind` |
| Diamond stage 7 (discs) | §2 sets and `EMUSEN_FILE_MEDIA`; `emusen-disc` in the platform library; §8's PlayStation recogniser and the `.chd` probe |
| Diamond stage 11, or Nephrite's Sega CD gate if earlier | §3 `MEDIA` |

### 10.4 Archives (proposed 2026-10-11, at Diamond's stage 7; built at its stage 11)

The tester's PlayStation folder holds seventeen `.zip`s, each a disc's sheet with its tracks or a "(Track n)" file
with no sheet (`Diamond_CDROM.md` §8.2). Weighed by §1.1's forms [C]: not the core's (a core receives bytes, never a
path, and must stand alone under the kit and the probe), not `emusen-disc`'s (an archive is not a disc, and the
cartridge consoles' zips need the same reading), so **a component of its own**: the frontends' image loader reading an
archive's entries through Galaxia, which owns paths and would gain the naming of a path inside an archive, choosing
the entry the system pack's extensions name (a sheet first, then a one-file image), reading its companions from the
same archive, and handing bytes to create as for loose files; with routing by content (§8) deciding the console from
the chosen entry. Its cost is the inflation at load, on top of the copy at create (`Diamond_CDROM.md` §9.7). ~~Nothing
is built.~~ *Approved and built 2026-10-11 (`EmuSen_Galaxia.md` §3.7, `Diamond_Players.md` §2): `ArchiveImages` in the
runtime's media module, for `.zip` only, reading entries through .NET and unpacking into Galaxia's `Unpacked` folder
rather than handing bytes from the archive, because a sheet's tracks are read by path and a C# core opens its file
itself; Galaxia names the folder and gained no reading of archives.*

### 10.5 Battery files in core info (proposed and decided 2026-10-12)

*Decided 2026-10-12: built in minor 1.3, with §10.2, as the design below gives it. Diamond lists files 0 (`.mcr`) and
4 (`.2.mcr`); the test core file 0 (`.srm`); the other cores write no field and behave as before. The engine passes
the listed files that exist at the first create, so a PlayStation game with its cards on disc is created once: the
launch tables of `Diamond_Players.md` §12.8.9 have no second create.* `EmuSen_CoreAPI.md` §19.1 records that a v1.0 host cannot know a core's
battery files before a game is created, so the engine creates once with file 0 as `.srm`, reads machine info's
`battery`, and creates again when the files differ and one exists on disk; it names the remedy, core info carrying
them per system in a later minor, and leaves it undecided. Diamond names two cards, file 0 as `.mcr` and file 4 as
`.2.mcr`, so **every launch of a PlayStation game with a memory card on disc creates twice**: a second 150–290 ms on the
desktop [M, 2026-10-12, `Diamond_Players.md` §12.8.2] and 45–214 ms on the handheld, where five games already have a
card on disc (§12.8.6 there). Under big picture's launch screen
it is now hidden whenever the whole load ends within the screen's time; from the desktop's list it is waited for.

Weighed by §1.1's forms [C]: **an extension**, a descriptor field `battery` in each system entry of core info, of the
shape machine info's already has (`which`, `suffix`, `length`), listing every battery file a machine of the system can
name, machine info staying the authority for the machine created. A host that reads it passes those files at the first
create and creates again only where machine info then differs; a host that does not, and a core that does not write
it, behave as now. Rejected: the host remembering the last machine's files for the same library and settings, which is
a guess that the first launch after every start of the program still pays for, needs a fallback create where a foreign core refuses a
file number it does not name, and gives a fact the core owns a second owner in the host.

**The design** [C]. `info.systems[].battery` is an array of `{ "which": n, "suffix": ".mcr" }`, optional and additive
in `info.schema.json`; `length` is left to machine info, since it is a property of the machine made.
- **A core that writes the field** lists every battery file any machine of the system can name, whatever its settings,
  and accepts each of them at create whether or not the machine made keeps it: a card's file passed for a slot the
  settings leave empty is ignored, not refused. An empty array says that the system keeps no battery file.
- **A host that reads it** passes, at the first create, each listed file that exists on disk, by the runtime's path
  rule for its suffix. Machine info stays the authority: after create the host keeps the files machine info names, and
  **creates again only when machine info names a file that exists on disk and was not passed**, which a core that
  lists its files truthfully never causes.
- **A core without the field** gets what it gets now: file 0 by the `.srm` rule at the first create, and a second
  create where machine info then names other files that exist. **A host that does not read the field** behaves as now
  with every core.
- **The kit** (C14): a listed `which` is unique within its system, a suffix begins with a full stop, and every battery
  file machine info names for a machine of that system is listed, where the system lists any.

---

## 11. Predictions to be retired

- **P1.** Each ABI addition here leaves the registration-equivalence golden (`EmuSen_CoreAPI.md` §25) and every
  existing core's info and machine info byte-identical, since none writes a new field or claims the new bit.
- **P2.** On Diamond's corpus form the process's peak resident memory stays below the allowance, and below the
  largest sum of estimates admitted at once, by no more than a factor of two.
- **P3.** Moving the 22 call sites to `SystemOf` changes no console, shelf or save path for any file in the tester's
  library, on the cartridge consoles.
- **P4.** Of the PlayStation GameShark codes in the libretro cheat database, more than nine in ten decode into the
  existing write types plus `IfLess` and `IfGreater`; most of the rest are `D4`–`D6` and `C1`.
- **P5.** At least one of Diamond's multi-disc bench games refuses a disc inserted without the lid seen open for a
  measurable time, which fixes §3.5's interval.
- **P6.** No game on Diamond's bench reads the DualShock's motors as anything but a level per poll, so the per-frame
  average loses nothing a player can feel.
- **P7** (2026-10-12, §10.2). With §10.2's shape B, a cold launch of a 660 MB loose track from a 90 MB/s medium
  reaches its first frame within 0.5 s of the same launch warm, the process's peak memory falls by the image's size,
  and no frame of the first 30 s of play takes longer than its period on the handheld. *Measured 2026-10-12
  (§10.2.5):* the first two clauses hold, Spyro the Dragon's 662 MB track launching in 115 ms cold under the cap and
  114 ms warm, and in 99 ms and 95 ms from the handheld's card; peak memory fell by more than the image, 1,317 to
  54 MiB, since the host's buffer went with the core's copy. **The third is not met**: unpaced, one frame of Mega Man
  X5 took 20.6 ms on the card for a first touch; the prefetch built for it leaves three of four games with none or
  one, and Mega Man X6 with up to three frames of 24–50 ms in its first six seconds, where without it they were
  8–32 ms (§10.2.5, `Diamond_Players.md` §12.8.9). It stays open with the two remedies §10.2.5 records. *Its third clause is
  carried by P8 since 2026-10-12.*
- **P8** (2026-10-12, §10.2.6). With minor 1.4's event acted on, no frame is held past its period by a first touch on
  the handheld's card in a game's first 30 s, paced and from a cold cache, on Mega Man X5, Mega Man X6, Crash
  Bandicoot and Spyro the Dragon. It is P7's third clause, restated for what a host can be held to: a frame the
  emulation thread waited in, not one that was merely slow beside the handheld's interface. *Tested 2026-10-12 on the
  desktop under the 90 MiB/s cap (§10.2.7), and its confirmation on the handheld's card left to play, as decided that
  day:* **it holds there**, and more than it claims: in fifteen runs with the event, six of Mega Man X6, no frame of
  any of the four games took a major fault, where 1.3's prefetch held one frame 27–195 ms in six of the nine runs of
  Mega Man X6 and Crash Bandicoot. What the cap does not model, a short read's time on the card, is said in §10.2.7.

---

## 12. Questions for the tester

**Decided 2026-10-11:** all fourteen, each as recommended. By Q13 each item lands at the first stage that needs it: the conditional cheat comparisons, the kit's memory estimate and routing by content now; outputs, named extras and memory kinds at Diamond's stage 6; image sets and the `.chd` probe at its stage 7; disc change at its stage 11 or the Sega CD's gate. Q5's two rumble items join the to-do list with their triggers.

Each with a recommendation.

1. **Q1, disc change.** *Recommended:* the optional exports `media_lid` and `media_insert` behind `MEDIA` (bit 18),
   with `emusen_media`, machine info's `drives`, three status codes, the drive recorded beside every state and
   restored before a load, and kit case C16 (§3). Rejected: all discs at create with settings; a new machine per disc;
   per-core extensions.
2. **Q2, image sets.** *Recommended:* the companions of a sheet and the `.m3u` list read by `emusen-disc`'s set module,
   used by the kit directly and by the runtime through the platform library (a new dependency of the platform library
   on a shared core crate, stated); companions numbered from `EMUSEN_FILE_MEDIA` (65,536); `.m3u` never passed to a
   core (§2). This **revises** Nephrite's Q5 ("the system pack parses `.cue`") and Diamond's §5.11 (the system pack
   reading `FILE` lines; `.m3u` as an image).
3. **Q3, outputs.** *Recommended:* one event kind, `EMUSEN_EVENT_OUTPUT`, and machine info's `ports[].outputs` for
   motors and lights alike, each level averaged over the frame and normalised by the core (§4).
4. **Q4, rumble for the player.** *Recommended:* on by default with a strength, per player, in Preferences; motors
   stopped whenever the game is not running at speed; the DualShock's Analog light drawn on screen and not sent to a
   pad's light bar unless asked.
5. **Q5, the other rumble cores.** *Recommended:* the N64's Rumble Pak modelled in MarsRT as a choice of what each
   port's slot holds, once MarsRT is on v1; MercuryRT's MBC5 motor reported when MercuryRT resumes after Nephrite.
   Both added to the to-do with those triggers.
6. **Q6, the Analog button.** *Recommended:* the extras path v1.0 already specifies, with a `name` per extra and no
   new `PadButton`; defaults a free keyboard key and Misc1 where a pad has one; Diamond's power-on mode a setting, as
   its Q11 has it (§5).
7. **Q7, conditional cheats.** *Recommended:* `IfLess` and `IfGreater` appended to `CheatWriteType`; the PlayStation
   codec in `Sys/Systems/Psx/`; `C0` as tests within its cheat once a source or a test says it does not latch, refused
   until then; `C1`, `C2` and `D4`–`D6` refused by name; the chained-conditional rule settled at Diamond's stage 11
   against a document or a measurement, with the AND rule until then; and `ChtFile`'s `cheat_type` 6 and 7 mapped to
   the new tests (§6).
8. **Q8, `CHEAT_POKES` for Diamond.** *Recommended:* Diamond does not claim it, since nothing in its documents gates
   pokes, and every code is applied by the runtime between frames.
9. **Q9, the kit's estimate.** *Recommended:* `256 MiB + 4 × the image's size`, admitted by a memory budget, the peak
   recorded in the summary, and an override for cores that expand an image (§7).
10. **Q10, routing.** *Recommended:* one runtime function deciding each file's system — the single claimant, else the
    packs' recognisers, else the folder's ES-DE name, else the player's choice remembered per game — with every caller
    of `ConsoleForRom`, `ShelfFor` and `SystemFor` moved to it; a console's extensions and signatures owned by its
    system pack, a core's info listing a checked subset (§8).
11. **Q11, reading `.chd` for routing before Diamond's stage 7.** *Recommended:* the folder and the player's choice
    only, until `emusen-disc` is in the platform library; no CHD decoder written in C#.
12. **Q12, memory cards.** *Recommended:* machine info's `battery[].kind` (`game`, `console`, `removable`); every kind
    per game by default; a per-system option to share a console's or a removable memory, with a second session on a
    shared file refused and states' effect on it stated to the player (§9).
13. **Q13, the order.** *Recommended:* each item when its first stage needs it (§10.3), so that each minor's addition
    lands with the core that tests it, and nothing enters the baseline unexercised.
14. **Q14, device kinds.** *Recommended:* none now; adapters such as Diamond's multitap as port settings, as the
    Genesis's are; pointer devices taken up when a gate needs one (§10.1).

*Raised 2026-10-12, by the launch-time measurement of `Diamond_Players.md` §12.8. **Decided 2026-10-12:** both as
recommended, in one minor, 1.3, with disc images handled in one shared place (§10.2.3).*

15. **Q15, streaming reads.** *Recommended:* §10.2's shape B, the image borrowed for the machine's life behind a
    capability bit, mapped by the host, with a kit case comparing a mapped run with a copied one; shape A kept on
    record. Rejected for now: nothing, which leaves a cold launch from a card at the speed of the whole read.
16. **Q16, battery files in core info.** *Recommended:* a `battery` field in core info's system entries, of machine
    info's shape, read by the host for the first create (§10.5). Rejected: a host-side memory of the last machine's
    files.


*Raised 2026-10-12 by §10.2.5's handheld traces. **Decided 2026-10-12:** the exact remedy, as minor 1.4 (§10.2.6).*

17. **Q17, a first touch in a game's first seconds on a slow medium.** *Chosen:* an event by which a core names
    where its drive is going, the host reading ahead during the emulated seek, because it is exact whatever the game
    reads first. *Not chosen:* the prefetch at the medium's full speed until the first frame, which narrows the
    window and closes it for no game in particular.

---

## 13. Read watches and read data breakpoints (1.2), decided 2026-10-11

*Decided 2026-10-11: the tester approved the extension `Diamond_Debugger.md` §5.1 proposed, and it was built the same
day with Diamond as its first user. This section is its specification and its record; the normative text it adds to
the core ABI is `EmuSen_CoreAPI.md` §6.14.1, and the header's lines carry `@since 1.2`. Marks as above; [M] here is
measured or run on 2026-10-11.*

### 13.1 What existed

- **v1.0 watched stores only.** `debug_set_ranges` took kind 0 (watches) and kind 1 (data breakpoints), both over
  stores (`emusen_core.h`, `EmuSen_NativeCores.md` §3.14) [R]; the debug flags had `WRITES` and no counterpart for
  loads; the logs had no read log.
- **The host skipped what it could not send.** `CoreDebugBridge.PushTables` left out every watch of
  `WatchKind.Read` and every data breakpoint `OnRead` [M], so `watch add … read` and `bp read …` on a v1 core were
  accepted by DianaSh and then saw nothing.
- **The shared observer had no load.** `emusen_native::debug::Observer` reported a step's boundary, each stored byte,
  calls and returns; a CPU crate had nowhere to report a load [M].
- **A defect of 1.0's crate, shown and fixed here.** `Hooks::set_ranges` treated every kind but 0 as the data
  breakpoints, so a kind 2 sent to a 1.0 core would have replaced its data breakpoints, against §4.2's rule that an
  unknown value of an open enumeration is refused (`EmuSen_CoreAPI.md` §4.2). Shown by
  `a_kind_the_hooks_do_not_know_changes_no_range` failing on `WiseMan` at `29c1afb9` (kind 9 replaced the data
  breakpoints) and passing after the fix [M]. No host had ever sent a kind above 1, so nothing a conforming peer
  could observe changed; the pre-stable `emusen_native_debug_set_ranges` keeps its two kinds exactly as before.

### 13.2 The forms weighed

As `Diamond_Debugger.md` §5.1 weighed them [C, decided]: **an extension** of the debug surface, since a read watch is
one more of the debugger's tables and not a second job for any component; no module form exists, because loads are
observed inside the core and cross the core boundary only through `emusen_core.h` (§1.1). A core's own extension
export would have served Diamond alone and been deprecated the day a second core added reads (§1.1's second point).

### 13.3 The surface change

```c
#define EMUSEN_CAP_DEBUG_READS       (1ull << 19)  /* @since 1.2, with DEBUG: emusen_core_debug_reads, range kinds 2 and 3 */
#define EMUSEN_DEBUG_READS       32u   /* debug_set: loads reported into the read ranges */
#define EMUSEN_RANGE_WATCH       0u    /* the two kinds of 1.0, named */
#define EMUSEN_RANGE_BREAK       1u
#define EMUSEN_RANGE_READ_WATCH  2u    /* loads logged after they read */
#define EMUSEN_RANGE_READ_BREAK  3u    /* a stop in front of the load, nothing of it run */
#define EMUSEN_STOP_READ         64u   /* the next instruction's load reads a byte in a READ_BREAK range */

/* The read log as (space, address, value, pc, kind) records, kind the range kind that logged the entry. */
int64_t emusen_core_debug_reads(emusen_machine *machine, uint32_t *out, size_t len);
```

and one value of a descriptor's open string, **`reports_loads`** in machine info's `spaces[].flags`: the spaces whose
loads reach the read ranges, as `reports_stores` names those whose stores reach the store ranges.

- **The log's layout.** Five 32-bit words an entry: the space's id, the address within it, the value, the loading
  instruction's address, and the range kind that logged it. Kind 2 (`READ_WATCH`) is a byte a load read, logged after
  the read, with the value read. Kind 3 (`READ_BREAK`) is a byte a load is about to read, logged in front of it, with
  the value `space_read` gives at that address then: for a memory that is the value the load will read unless a DMA
  lands between; for a bus view it is what the view shows, which may differ from a device's answer. A byte covered by
  both kinds is logged twice, once ahead and once read. The log drains as the other logs do (copied and forgotten only
  when it fits, `RING` when full).
- **The flag.** `READS` gates both read kinds, as `WRITES` gates both store kinds.
- **The stop.** A read data breakpoint stops the frame **in front of the loading instruction, nothing of it run**, with
  `STOP_READ`, as a breakpoint stops; continued with `RUN_UNCHECKED`, the instruction runs unasked and its load is
  made. This differs from a store's data breakpoint, which can stop only once the store has run (`STOP_DATA`, in
  front of the next instruction), and it is why the stop has a bit of its own.
- **The bit.** `DEBUG_READS` is claimed only with `DEBUG` (`check_groups` refuses the build otherwise). A core
  without it answers `EMUSEN_NOT_SUPPORTED` to kinds 2 and 3, and every core to any kind above 3. A host sends kinds 2
  and 3, and sets `READS`, only to a core that claims the bit, since a core built on 1.0's crate would have taken kind
  2 for data breakpoints (§13.1).
- **Bit 19, minor 2.** Bit 18 and minor 1 are the disc change's (`MEDIA`, §3), built on its own branch at Diamond's
  stage 11. Whichever of the two reaches `WiseMan` second is renumbered to follow the other: the bit stays 19
  either way, and only the minor and the `@since` tags move.

### 13.4 Older and newer peers

- **A newer host and an older core**, or any core without the bit: the host pushes no kind 2 or 3 and does not set
  `READS`; `watch add … read` and `bp read …` see nothing, as before 1.2, and a `both` watch sees the stores.
- **An older host and a core with the bit**: the host never sets `READS`, never sends kinds 2 or 3 and never calls
  `debug_reads`; the core's read ranges stay empty and its observed frame is 1.0's. The extension is safe never to be
  used (`EmuSen_CoreAPI.md` §4.6).
- **The core's cost when unused.** A processor crate reports loads through `Observer::read`, whose default is empty,
  so a plain frame compiled with `Unobserved` contains no trace of it; Diamond measures it (`Diamond_Debugger.md` §9.5).

### 13.5 The kit and the host

- **The kit.** C2 checks the bit against the export, as for every optional export (`EXPORTS`). C15, where `DEBUG_READS`
  is claimed, arms `READS` and both read kinds over a space no core has beside the store kinds, and requires the
  plain run's digests; and on every core claiming `DEBUG` it requires kinds 2 and 3 to answer 0 with the bit and
  `NOT_SUPPORTED` without it, and kind 4 `NOT_SUPPORTED` always.
- **The host** (`CoreDebugBridge`): read and both watches over spaces flagged `reports_loads` become kind 2 ranges, and
  `OnRead` data breakpoints kind 3; drained, a kind 2 entry goes to `WatchRegistry.RecordRead` with its value and the
  loading instruction as context, a kind 3 entry to each `BreakpointRegistry.NoteRead`, whose value test, condition and
  log run as they do for a C# core's read; the registry then halts the host at the load. DianaSh needed no change: its
  `watch` and `bp` commands already took `read` and `both`. Mistress's debugger window lists no watch kinds, so it
  needed none either.
- **What a C# core's read breakpoint does differently.** A C# core notes a read as it happens and halts after the
  instruction (`man bp`); a v1 core claiming `DEBUG_READS` halts in front of it. The registry is the same and so is
  the halt message; the place differs by what each loop can do, and `man bp` says so.

### 13.6 Cost, and what it breaks

Nine baseline lines (the bit, the flag, four kinds, the stop, the export's two facts) and one string value; the
`Hooks` fields, `note_read`, `note_read_ahead` and `Observer::read` in `emusen-native`; the export and its group in
`core_exports!`; the C# table entry and the bridge's push and drain; the kit's checks. Nothing of v1.0 changes for a
conforming peer. The registration-equivalence golden (`EmuSen_CoreAPI.md` §25) is unchanged except for Diamond, the
one core that claims the bit (§13.7).

### 13.7 What was built and measured

The record is `Diamond_Debugger.md` §9 for Diamond and its numbers, and `BlackMoon_R3000.md` §12 for the processor's
two additions. In brief [M]: the header and the baseline (`emusen-core-abi-check`: 392 facts agree on 4 triples, its
4 tests pass), `emusen-native`'s 62 tests, the processor's 33 (the suite's 55,000 cases unchanged), Diamond's 147 and
57 program tests, the kit's 5 and 15 (the 15 pass run one at a time; run in parallel, one seeded-fault test failed in
each of two runs, a different one each time, and passed alone, a race among the tests and not this change), and WiseMan's 324 debugger, ABI and DianaSh golden tests across the v1 cores all pass;
C15 with the read tables armed gives the plain run's digests on Diamond. Because the minor itself now moves, the check
treats `EMUSEN_CORE_ABI_MINOR` and `EMUSEN_CORE_ABI_VERSION` as the header's revision, which only goes up, written as
the disc change's branch wrote it so that the two merge as one. The baseline also gained `json info.systems[].development`,
a field `info.schema.json` has had since 2026-10-06 that no `--emit-baseline` had recorded; it enters at whichever
minor first records it.

## 14. Pictures that settle after a load (1.5), decided 2026-10-14

*Written at Diamond's seeded state (`Diamond_Enhancements.md` §52), before its code.*

### 14.1 What existed

A state reproduced the machine and, through §6.9's determinism rule as the kit's C7 and C8 test it, every picture after
a load: a machine given another's state handed the pictures that machine handed from the next frame on. Diamond's
renderer draws its picture at up to four times the console's resolution beside the accurate machine; to meet the rule
its state kept the renderer's own copy of VRAM and its picture at that scale, 81.9 MB a state at 4× with every
enhancement on against 5.4 MB for the machine, and rewind's deltas up to 28 times the machine's (§47.2 there). A renderer
seeded from the machine's VRAM at the load hands the kept form's pictures again within six frames on drawn pictures and
within 141 on a feedback effect (§52.1 there), but not from the first frame.

### 14.2 The forms weighed

| Form | For | Against |
|---|---|---|
| (a) The kit excuses pictures after a load whenever a setting with `effect: enhancement` is not at its hardware value, for a bound of its own | no surface change | the kit chooses one bound for every core without knowing why a picture differs; it excuses cores whose states need no excuse; a host that loads states every frame learns nothing |
| (b) **Machine info's `state.picture_settles`**, declared by the core | the core states its own bound once, per configuration; the kit holds it to it; a host reads it; absent is today's rule | a new descriptor field, so a minor |
| (c) A capability bit, or a state kind that keeps the full picture for hosts that need it | a host could ask for exact pictures | a bit carries no bound; a second state kind doubles a core's state code for a need no host has |

**Chosen: (b)** [C], with a maximum of 600 frames so that the field cannot excuse a whole run, and the kit's rule
(`EmuSen_CoreAPI.md` §28.10).

### 14.3 The surface change

- `machine_info.state.picture_settles`, integer, optional, from 0 to 600: the baseline's line
  `json machine_info.state.picture_settles integer optional @1.5`, the schema's property with its range, and
  `EMUSEN_CORE_ABI_MINOR` 5. No export, struct, constant or bit changes.
- `emusen-native`: `Core::picture_settles(&self) -> u32`, 0 by default, written by `core_exports!` into machine info
  only when it is not 0, so that no other core's machine info changes by a byte. A core of minor 4 or older never writes
  it, and a newer host reads its absence as 0.
- The C# host: `CoreMachineInfo.PictureSettles`, 0 when absent. No host behaviour depends on it today.
- The kit: C7 and C8 as `EmuSen_CoreAPI.md` §28.10 states, with seeded faults in its test cores (a picture that settles
  within its bound, one that never settles, one that differs without declaring, sound that differs within the bound,
  a bound above the maximum).

### 14.4 Older and newer peers

An older host ignores the field and is unaffected: it never relied on a loaded picture's being exact, and its rewind
and state slots already take a state of any length. An older kit run on a core declaring a bound fails that core's C7
or C8 when a picture differs after a load, which is the rule it was built with; a core author runs the kit of the
core's own minor or later.

### 14.5 Cost, and what it breaks

Nothing for a core that does not declare it. For one that does, the kit's comparison of a loaded run's first frames is
weaker by exactly the frames declared, and what it compares past them is unchanged. The bound is a promise the kit
checks on the images it is given, not a proof: a game outside them can break it (`Diamond_Enhancements.md` §52.4).

---

## 15. B4's six items (1.13), decided 2026-10-18

*Proposed by `EmuSen_Backend.md` §17.3 (the Game Boy's shim), §18.4 (MarsRT's) and §19.4 (MoonRT's and VenusRT's),
which weigh each item's forms; decided by the tester on 2026-10-18 as one minor (`EmuSen_Backend.md` §20); built on
2026-10-18 (`EmuSen_Backend.md` §22). The normative text is `EmuSen_CoreAPI.md` §6.33 to §6.38.* Each item is
something only a core holds, which a shim read past the core ABI or through a private export, and which a generic
host must reach without per-core code once the shims retire (SR8).

| Item | Form | Surface | First core to claim it | `EmuSen_CoreAPI.md` |
|---|---|---|---|---|
| **Units** (§17.3's `units`, §19.4's `devices`, built once as `units`) | an extension of `DEBUG_REGISTERS` | machine info's `units`, ids from 256, read by `debug_registers`; no bit, no export | MoonRT (PPU, APU, board), MercuryRT (PPU, APU, timer, channels, HDMA) | §6.33 |
| **`TELEMETRY`** | a capability | bit 23, `emusen_core_telemetry`, machine info's `telemetry` | MarsRT (58 counters) | §6.34 |
| **`frame_info.lines`** | a struct field appended | `emusen_frame_info` 64 bytes, 56 still taken | MarsRT at an internal resolution above 1x | §6.35 |
| **`PRESENCE`** | a capability, the narrow form | bit 24, `emusen_core_set_present` | MarsRT | §6.36 |
| **`EMUSEN_FRAME_CHEATS_HELD`** | a frame flag | `frame_info.flags` bit 1; no bit | MarsRT (`Status.IE` clear) | §6.37 |
| **`DEBUG_TRANSLATE`** | a capability, with `DEBUG` | bit 25, `emusen_core_debug_translate` | MarsRT (the TLB); VenusRT's §30.6 item 11 and Diamond's segments may follow | §6.38 |

**Why one name.** The two phase-1 sections proposed the same field under two names; `devices` would have been the
third meaning of "device" in the ABI after 1.9's lent Vulkan device and §10.1's controller kinds, so `units` was
taken. Ids from 256 were §19.4's, kept because processors' ids are bounded below 24 by the coverage flags.

**Older and newer peers.** Each item is optional: a field a 1.12 host ignores, a bit it never sees claimed, a flag bit
it never tests, and a struct it passes at 56 bytes, which a 1.13 core fills as far as it goes. A 1.13 host reading an
older core finds no field, no bit and no flag, and drives it exactly as before.

**Cost.** None a frame for a core that claims nothing: the host reads `frame_info` once after a frame only while it
holds cheats of its own to apply. `telemetry` and `debug_registers` copy into buffers the host made once; presence is
sent only on a change.

**What is deferred.** §10.1's general `set_device` stays deferred: presence is what the N64 needs and is part of its
state, and a device kind would restate the controller choice the settings already make. The session service's
telemetry kind and `coretop`'s counters are the frontend API's next minor.

---

## 16. A space's byte order (1.14), decided 2026-10-18

*Proposed by `Nephrite_Native.md` §66.2, when the Genesis family's systems table entry was set `big` and the Z80's
spaces began to read swapped; decided by the tester on 2026-10-18 and built the same day (`EmuSen_Backend.md` §25). The
normative text is `EmuSen_CoreAPI.md` §6.39.*

### 16.1 What existed

A space was a byte array read through `space_read`, and a host that put two or more bytes together (the frame log)
took the order from the systems table's `byte_order`, one a system (`EmuSen_Backend.md` §18.9). On the Genesis every
space of the 68000's side holds a word high byte first, and the Z80's RAM, and its bus where the RAM shows, hold the
Z80's words low byte first (`Nephrite_Native.md` §66.1). One order a system could not be right for both.

### 16.2 The forms weighed

| Form | For | Against |
|---|---|---|
| (a) **Machine info's `spaces[].byte_order`**, stated by the core | the core owns how it stores a word, and states it beside the space it describes; the table keeps a system's order and the field is needed only where a space differs | a descriptor field, so a minor |
| (b) A per-space exception in the systems data file | no ABI change | ties a system's data to one core's space names, which a second Genesis core need not share; the table would hold a core's fact |
| (c) Splitting a mixed space into spaces of one order each | every space one order | renames and renumbers spaces the debugger, the cheats and saved frame logs key on; a processor's bus is one space because the processor sees one |

**Chosen: (a)** [C], an extension of an existing surface: one optional field of a descriptor every core already
writes, which gives no component a second job and couples none. A space of two orders states the order its readers
most often need and documents the rest (`EmuSen_CoreAPI.md` §6.39).

### 16.3 The surface change

- `machine_info.spaces[].byte_order`, a string, optional, `big` or `little`: the baseline's line
  `json machine_info.spaces[].byte_order string optional @1.14`, the schema's property with its two values, and
  `EMUSEN_CORE_ABI_MINOR` 14. No export, struct, constant or bit changes.
- `emusen-native`: `Space::byte_order`, `None` by default, written only when set, so that no other core's machine info
  changes by a byte.
- The C# host: `CoreSpace.ByteOrder`; `CoreEngine`'s frame log reads the space's order first and the system's after.
- The kit: C14 holds a stated order to the two spellings, with a seeded fault in its test core.

### 16.4 Older and newer peers

An older host ignores the field and reads every space in the system's order, which was its behaviour; for Nephrite's Z80
spaces that is the swapped reading §66.2 recorded. An older kit ignores the field. A newer host reading an older core
finds none and reads the system's order.

### 16.5 Cost, and what it breaks

Nothing a frame for a core that states none: the field is read with machine info. The frame log looks its space up by
name once more for each value, beside the lookup it made for every byte, and the lookup became a loop over the
descriptors, which allocates nothing where the query it replaced made a closure at every byte. Nothing breaks: a value the frame log reads from a
space that states an order changes only where that order differs from the system's, which is the defect it fixes.
