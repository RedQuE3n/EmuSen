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
| 10 | **Device kinds; streaming reads** | later | **deferred**, with the shape that would fit recorded | none now |

Read §1 for the one mechanism the five ABI changes share, §2 before §3 and §8, which both build on it, and §12 for the
questions.

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
| a new event kind | §4 (`EMUSEN_EVENT_OUTPUT` = 7; kinds 1–6 assigned, `emusen_core.h:132–139` [M]) | a host skips a kind it does not know |
| a new descriptor field | §3 (`drives`), §4 (`outputs`), §5 (`name`), §9 (`kind`) | a reader ignores fields it does not know |
| a new header constant | §2 (`EMUSEN_FILE_MEDIA`), §3 (`EMUSEN_MEDIA_RESTORE`) | — |
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
