# Diamond_Shell — the console with no game: an empty drive, the shell and its memory card manager

*Written 2026-10-17, before any code, at the tester's request of the same day: "Let's add the ability to Diamond to boot
into the PlayStation with no game and be able to go to its memory card manager. Make it so that if later we want to add
the ability to play music cds and use the visualizer, we can."* This page was written as a design for the tester's
approval. *Decided 2026-10-17 by the tester: Q1–Q13 and P1–P4, each as recommended* (§9): S5 is built, the visualizer's
home is decided when the player is planned, and P1–P4 are standing rules for this work. §1 gives the form of each part and why, §2 the empty drive as the documents describe it, §3 the
memory card manager's scope, §4 how the shell is written and tested without a window, §5 the seams the audio CD player
and its visualizer will use, §6 the phasing, §7 the risks, §8 the choices where the documents are silent, §9 the
questions for the tester, and §10 the reading log.

The marks are the plan's (`Diamond_Plan.md` §1): **[R]** read from the source named; **[A]** argued; **[C]** a choice
where the documents are silent, each listed in §8; **[M]** measured, with the tool named.

---

## 0. What exists, and what the request adds

Three facts frame the design. Each was read from the record and the code on 2026-10-17 [R].

1. **The machine already runs with an empty drive.** Diamond's CD-ROM controller answers every command with no disc as
   psx-spx describes (`Diamond_CDROM.md` §4, §5). A PS-EXE is a program, not a disc, so a machine made for one has an
   empty drive (`v1.rs`, `a program's console has an empty drive`). `media_insert` with a null image empties the drive,
   and with a disc fills it (core ABI 1.1, `EmuSen_CoreAPI.md` §6.23). Only `create` cannot make an empty machine: it
   must be given an image, and `Media::read` refuses an empty one as `NotPlayStation`.
2. **The open kernel already has a screen with no disc.** Stage 10 built a plain screen and a minimal card manager into
   the kernel (`Diamond_OpenKernel_Phase2.md` §16): `screen.rs`, `font.rs` and `manager.rs`. Both cards are listed by
   title; Cross copies a save, Triangle twice deletes it, Square twice formats the card. A disc put in is booted within
   two seconds. `the_screen_with_no_disc_formats_a_card_and_boots_a_disc_put_in` holds it. No player has reached that
   screen, because no host can create the machine without an image.
3. **The host's path is a game's path.** `CoreEngine.LoadRom(path)` derives everything from a file: the system by
   routing, firmware by `firmware_for(image)`, the battery and card files by the game's name, and the states, cheats,
   play time and Continue Playing by the path. Mistress's `LoadGame` does the same.

So the request needs four additions:

- a way for any host to start any console with nothing in it, defined once for every console;
- Diamond's acceptance of that start;
- a shell worth reaching;
- a frontend entry.

It also needs named seams for the audio CD player and its visualizer, none of which is built here.

## 1. The form of each part

The project's rule for a feature is to weigh three forms before building: an extension of an existing surface, a
module or plugin, or a component of its own. Each part is weighed below.

### 1.1 Starting with no media: an extension of the core ABI, minor 1.11

Starting a console empty is not the PlayStation's alone. The Sega CD's BIOS has a backup RAM manager and a CD player
behind an empty drive, and Nephrite already models an empty drive (`Nephrite_OpenBios.md` §16). The PC Engine CD-ROM²,
the Saturn and later disc consoles have the same. A console of a computer kind, starting into its own firmware, is the
same case without a drive. The surface is therefore the core ABI's, written once.

**The forms weighed.**

| Form | For | Against |
|---|---|---|
| (a) A capability bit, "create accepts no image" | one bit; the kit tests it beside C2 | a bit belongs to the library, and one library may run a system that can start empty and one that cannot. The Genesis cannot; the Sega CD beneath it can. `EmuSen_CoreAPI.md` §6.26 rejected a library bit for the same reason. |
| (b) A second export, `create_empty(system, params)` | the null image never reaches `create` | a second way to make a machine copies `create`'s handling of files, settings, flags, loans and refusals; the ABI grows by an export where a field suffices |
| (c) An empty image as a convention, with nothing declared | no surface change | a host cannot know which consoles it may offer to start, so no frontend can show the entry; a core that predates it refuses with its own status, which a host cannot tell from a bad game |
| (d) **A per-system fact in core info, and a null image in `create` for such a system** | the fact lives where the other per-system facts live (`extensions`, `firmware`, `battery`); `create` keeps its one path | a minor, so that the text and the kit say it |

**Chosen: (d)** [A]. Its surface:

- **Core info.** A system entry gains `starts_empty`. It is true when a machine of the system can be created with no
  image and runs as the console does when powered on with nothing in it. When absent it is false, so no existing core
  info changes by a byte.
- **`create`.** For a system whose entry says `starts_empty`, `params.image` may be null with `image_len` 0. The machine
  is the console with every drive empty and its lid closed, every slot as the settings and files make it, and its
  firmware chosen as the core chooses it with no game.
  - `emusen_create_params` gains one field at its end: `const char *system`, a system id from core info as UTF-8 with a
    NUL. The struct grows from 104 bytes to 112.
  - A host sets `system` only with a null image. A core reads it only then, and ignores it otherwise; a later minor may
    give it a meaning with an image.
  - A null image is refused with `EMUSEN_BAD_IMAGE` (−264) when `system` is missing, is not one of the library's, or
    names a system without `starts_empty`. No new status code is needed.
- **Firmware.** `firmware_for` takes an image, and there is none, so it is not called. The host passes every file of
  the system's `firmware` list (core info, `EmuSen_CoreAPI.md` §6.3) that it finds in the player's firmware library. That list already
  names "every firmware file the system can use", so no new call is needed. Which file the machine runs is the core's
  choice, as with a game (§1.2 for Diamond's rule).
- **Machine info.** It is unchanged. A machine with a drive lists it as `loaded: false`.
- **The drive afterwards.** Nothing new is needed. A disc goes in through `MEDIA` (1.1): `media_lid` opens the lid,
  `media_insert` puts the disc in, `media_lid` closes the lid. A machine started empty with a drive must therefore come
  from a core that claims `MEDIA`. The kit fails a core that lists `starts_empty` and a drive without `MEDIA`, since
  that console could never take a disc. A system with `starts_empty` and no drive is allowed: a console that starts
  into its own firmware with an empty cartridge slot, or a computer.
- **States.** These are unchanged. A state holds its disc's fingerprint as the core chooses (`EmuSen_CoreAPI.md` §6.23); Diamond's is 0
  for an empty drive. `WRONG_MEDIA` still refuses a state over the wrong disc, and an empty drive is one more "disc" a
  host may restore with `EMUSEN_MEDIA_RESTORE` and a null image.
- **Older and newer peers.**
  - An older host never reads `starts_empty`, so it never sends a null image.
  - An older core never lists it, so a newer host never sends a null image to that core.
  - A newer host that passes 112 bytes to an older core is read as it always was: the core knows a prefix and has no
    system that starts empty.
  - The minor gates nothing in a host (`EmuSen_CoreAPI.md` §4.1).
- **The kit**, as **C22**, for each system whose entry says `starts_empty`:
  - a create with a null image and the system named;
  - two such machines equal frame for frame, sample for sample and state for state over the kit's frames;
  - a state round trip;
  - where a drive is listed, the drive empty and `MEDIA` claimed;
  - a null image refused with `BAD_IMAGE` for every system without the field, and with `system` missing.
- **The Rust side** (`emusen-native`): `System.starts_empty: bool`, `Create.image: Option<&[u8]>` and
  `Create.system: Option<&str>`. A core that does not list the field never sees `None`, because the export refuses a
  null image on its behalf. The core's own code is therefore unchanged unless it opts in.
- **The C# side**: `CoreInterface`'s struct and constant, `CoreSystem.StartsEmpty`, and `CoreMachine` accepting a null
  image with a system id.

Why a minor: the text of 1.0 to 1.10 requires an image, and their kits create every machine with one. 1.11 is the
revision whose text and kit accept a machine without one. Of the open questions in `EmuSen_CoreAPI_Extensions.md`,
none claims 1.11 [R, a search of the manual on 2026-10-17].

### 1.2 Diamond with an empty drive: the core, by its existing parts

This part needs no new component. It is a few lines in `v1.rs` and `media.rs` over parts that already exist:

- **`starts_empty: true`** on the `psx` system.
- **`create` with no image.** `Media` becomes optional. With none, no disc is inserted and no program is side-loaded.
  The machine starts at the reset vector on its kernel, as `Machine::new` already does for an image that is neither
  kind. The open kernel's shell finds an empty drive and shows its screen (§1.3). A player's BIOS shows its own.
- **The market with no disc** [C, §9 Q4]. Today `region=auto` takes the market from the disc or program, and from
  the Americas when the image names none. With no image there is nothing to name it. The rule proposed: the market of a
  player's BIOS passed at create, preferring the Americas, then Europe, then Japan when several are present; with none,
  the Americas. Without this rule, a player who has only the Japanese BIOS would start an American console on the open
  kernel, and their own BIOS would go unused. A region chosen in the settings overrules it, as it overrules a disc.
- **Audio discs** (the seam of §5.1, built only if §9 Q9 says so). `medium::open` refuses a disc whose sector 4 lacks
  the licence text, and an audio disc has no data track to hold it. The controller already answers GetID for an audio
  disc as psx-spx's table does: INT5 with flags 90h and the ID error set (`Diamond_CDROM.md` §11). Accepting one is a
  third outcome of `medium::open`, a disc with no data track, market none. It is accepted by `media_insert` and never
  by `create`, since a disc of music is not a game.

### 1.3 The open kernel's shell: a program of its own, loaded from the kernel's ROM

There are two forms for the shell. Neither copies anything of Sony's.

| Form | For | Against |
|---|---|---|
| (a) **A module inside the kernel**, as `manager.rs` is today | built; one image; no hand-over to define | the kernel gains a second job, a user interface, beside being the kernel. The shell shares the kernel's RAM, whose variables end at A310h of a limit of DF80h, so about 15 KiB remain (`Diamond_OpenKernel_Phase2.md` §20.4 and `kernel.ld`) [R]. It reaches the kernel's internals by Rust paths rather than by the documented call tables, so nothing holds it to the surface a game uses. A CD player and a visualizer would grow inside the kernel. |
| (b) **A program of its own**: a PS-EXE built from its own crate, carried in the BIOS image, loaded into user RAM and started with `Exec` when no disc boots | one job each: the kernel boots and serves calls, the shell is a client. The shell reaches the kernel only through the A, B and C tables that games use, a documented surface already versioned by its history. Its RAM is the user RAM that no program holds while it runs. It runs as a PS-EXE in a test on either kernel, and in the kit as an image. The CD player and the visualizer become modules of a program, not of the kernel. | a second crate on the pinned toolchain; the hand-over to define and test; the image grows, and `firmwarecheck.py` runs again |

**Chosen: (b)** [A, §9 Q2]. psx-spx documents the means: `Exec` "calls the executables entrypoint, with param1 and
param2 passed in r4,r5", and "if the executable (should) return, then R16,R28,R30,SP,RA are restored from headerbuf,
and the function returns with r2=1" (`kernelbios.md`, "A(43h) Exec") [R]. The shell returns, and the kernel looks at the
drive again.

**The hand-over, version 1** [C]. It is written down so that each side can be tested against it alone.

- **What the kernel does.** When `shell::disc()` finds no licensed data disc, `shell_main` copies the shell's PS-EXE
  from its place in ROM to the RAM its header names. It calls `Exec(header, 1, reason)`, where `reason` is 0 for an
  empty drive or an open lid, 1 for an audio disc and 2 for a refused data disc. When `Exec` returns, it starts the boot
  again from `_96_init`. 80030000h keeps its four instructions, so a side-loaded program still never meets the shell
  (`Diamond_OpenKernel_Phase2.md` §15).
- **The shell's address.** The shell loads at 80010000h, as a PS-EXE usually does, with its own stack below the
  kernel's default stack top. It never loads at 80030000h.
- **What the shell may use.** Every documented A, B and C call. The hardware is reached directly only where the kernel
  has no call: the GPU, the CD controller's audio commands ("The Kernel doesn't include any functions for playing Audio
  tracks", `kernelbios.md`, "Absent CD-Audio Support" [R]), and the SPU's volumes and capture buffers.
- **What the shell restores before it returns.** It stops its pads and cards (StopPAD2, StopCARD2), masks vblank in
  I_MASK as `manager.rs` does today, leaves the CD controller with no command outstanding, and returns 0. A return
  value other than 0 is reserved.
- **A disc put in.** When the drive reports a licensed data disc, the shell returns, and the kernel boots it. The
  kernel stays the only owner of the boot.

**What moves where.**

- `manager.rs` and the drawing half of `screen.rs` leave the kernel.
- The font (`font.rs`), the Shift-JIS conversion (`charset.rs`) and the GPU's primitives (a command, a fill, a glyph)
  are needed by both programs: the kernel draws Krom2RawAdd's characters from the same font. They go into one shared
  `no_std` crate, `diamond-guest`, which the kernel and the shell both link. This is the project's rule that two users
  of one thing share one copy.
- The kernel keeps `shell.rs`'s boot and gains the loader of the hand-over.

**The image.** The shell's crate builds on the kernel's pinned toolchain (`kernel/rust-toolchain.toml`) to a PS-EXE,
which is committed with its SHA-256 as the kernel image is. The kernel includes it with `include_bytes!`, and
`DIAMOND_KERNEL_REBUILD=require` rebuilds both to the same bytes. The image grows by the shell's size, an estimated 30
to 60 KiB, within the 512 KiB, of which 103,256 bytes are used today [M, the committed image's last non-zero byte,
2026-10-17]. `firmwarecheck.py` is run on the new image by the rule of `EmuSen_Firmware.md` §0 (§9 P3).

### 1.4 The memory card manager: a screen module of the shell

The manager is the shell's first screen module (§3). It holds no card file system of its own:

- it lists saves with `firstfile2` and `nextfile`;
- it reads a save's title and icon frames by `read`;
- it copies by `read`, then `open` with the create bit, then `write`;
- it deletes by `erase`;
- it formats by `format`, all on "bu00:" and "bu10:" as `manager.rs` does today.

The on-card format inside the console therefore keeps one owner, the kernel's `bu.rs`. On the host it keeps one owner,
`PsxCardFormat` (`EmuSen_SaveMedia.md` §6.2). The shell adds only the reading of a file's own first frames: the title
frame and the icon frames, which psx-spx documents as file content, not directory. Two implementations of the format
across the emulation boundary cannot be avoided, since a C# assembly cannot run in the guest. §4.3's test sets each
side as the other's judge.

### 1.5 The frontend: the host's launch, and Mistress's entries

Two components gain one job each, and no new component is needed.

- **The runtime (`CoreEngine`, `EmulatorSession`)** gains a launch that is not a file.
  - `StartConsole(system)` makes the machine with a null image. It passes the system's listed firmware and places the
    cards by the medium's mode, under a session name of the console's own ("PlayStation"), and it routes nothing.
  - `InsertDisc(path)` opens any file as Galaxia's `DiscSet` and adds it to the session's discs. It opens the lid,
    inserts the disc and closes the lid `LidOpenSeconds` later, as `ChangeDisc` does. The core judges the disc: a Sega
    CD disc put in a PlayStation is refused with the core's words, so routing by content is not consulted.
  - `EjectDisc()` opens the lid and inserts nothing.
  - What today keys on `RomPath` (states, screenshots, cheats, play time, Continue Playing, the launch screen's art)
    takes a launch record, `{ Path?, Console, Name }`, instead. The record has no path for a console session. Each
    reader of the path is listed and tested in the step that builds it (§6).
- **Mistress** offers the start only for a console whose engine lists a system with `starts_empty`, read from core
  info and never from a list of consoles:
  - the desktop library's console filter, when it shows that console, gains *Start PlayStation*;
  - the menu bar gains *Library → Start Console ▸ PlayStation*;
  - in big picture, *Start PlayStation With No Disc* is the first row of the gamelist's *Gamelist Options* menu, and
    the pad menu's *Main Menu → Library* gains *Start a Console*;
  - no pseudo-game is added to any gamelist [C, §9 Q6].
  - In a running session whose machine lists a drive, the game's menu gains *Insert Disc…* and *Eject Disc* beside
    the existing *Change Disc*.
  - *Insert Disc…* opens a chooser built with LunaP. It is a list of the console's games in the library, in the
    library's order with their covers, with *Browse…* on the desktop for any file. It is framed as a sheet in big
    picture and reached by the button grammar alone. It fits at 1280 × 800 and 1920 × 1200 under WiseMan's fit audit,
    and every row is reachable under the pad audit.
- **DianaSh**, for parity (`EmuSen_CoreAPI.md` §8.6): `core <corename> --console <system>` starts a console empty
  where a session exists. It is written in the step that builds the runtime's launch, its page beside `core.1`.

### 1.6 The audio CD player and its visualizer: a later screen module, and a frontend option

None of it is built here. The seams are named in §5. In form, the player is a second screen module of the shell. The
visualizer is a part of that module fed by a shell service. A visualizer drawn by the frontend over any core's sound is
a different thing, a component of its own, and §9 Q10 puts the choice.

## 2. The empty drive, from the documents

What a program sees from the controller with no disc, each line from psx-spx's "CDROM Drive" chapter [R], with
Diamond's answer as `Diamond_CDROM.md` records it. Where the documents are silent, Diamond's choice is marked.

- **The status byte.**
  - Bit 4 is "ShellOpen: Once shell open (0=Closed, 1=Is/was Open)"; bit 1 is the spindle motor; bit 3 is the ID
    error.
  - With no disc the motor does not spin up, so bit 1 reads 0. Diamond sets bit 4 at power-on whether or not a disc is
    present, as psx-spx's tables show a first response of 11h with a disc [R]. With no disc, that choice is Diamond's
    own [C, already in `Cdrom::new`].
  - Nop (01h) answers INT3 with the status and clears bit 4 once the lid is closed ("does reset the shell open flag").
    The first Nop after power-on therefore reads 10h and the next reads 00h.
- **Commands that need a disc.** "80h appears on some commands (02h..09h, 0Bh..0Dh, 10h..16h, 1Ah, 1Bh?, and 1Dh) when
  the disk is missing": INT5 with the status's error bit and 80h. Diamond answers so (`needs_disc`), Mute and Demute
  included.
- **GetID (1Ah).** psx-spx's table gives, for "No Disk", INT3(stat) and then INT5(08h, 40h, 00h, 00h, 00h, 00h, 00h,
  00h). The second byte's bit 6 is "Missing (0=Disk Present, 1=Disk Missing)". Diamond answers exactly that after its
  delay. For comparison:
  - door open: INT5(11h, 80h);
  - spin-up: INT5(01h, 80h);
  - an audio disc: INT3, then INT5(0Ah, 90h, …), where bit 4 is "Audio CD" and bit 7 is "Denied Data CD or Audio CD";
  - a licensed data disc: INT2 with "SCEx".
  - psx-spx cautions that "the status byte … may differ in some cases". **The shell decides by the flag bits, never by
    whole bytes** [A]: bit 6 means empty, bit 4 an audio disc, bit 7 without bit 4 a refused data disc, and INT2 a disc
    to boot.
- **The lid.** "When the shell is opened, INT5 is triggered regardless of whether a command was executing or not … The
  error byte in the INT5 is set to 08h." Closing the lid on a disc spins the drive up. On an empty drive it changes
  nothing but bit 4, which the next Nop clears. "The game can then send Nop commands, looping until bit 4 is cleared to
  detect when the new disc has been inserted." The shell polls the same way.
- **What the shell sees.** The kernel's `disc()` asks GetID until it answers. INT5 with 80h and the shell bit clear is
  a spin-up, so it waits. Any other answer than a licensed disc goes to the shell with its reason (§1.3). The shell then
  polls the drive itself: about every two seconds a Nop, as today, and a GetID once bit 4 has been seen set and then
  cleared. A licensed data disc ends the shell, an audio disc enables the CD player's entry (§5), and an empty drive
  leaves the screen as it is. The kernel owns no CD operation while the shell runs, and its interrupt element then
  "answers nothing and acknowledges nothing" (`Diamond_OpenKernel_Phase2.md` §12.1, choice 1). The shell is therefore
  free to drive the controller, as the CD player will.
- **With a player's BIOS.** Its own shell meets the same answers. Whether it reaches its menu with an empty drive is an
  observation of outputs, made by a writer who does not write the open shell (§9 P1). The Sega CD gives the warning:
  there, a player's BIOS stayed on its first picture with an empty drive, and the documents did not explain why
  (`Nephrite_Bram.md` §11, `Nephrite_OpenBios.md` §16). A defect found this way is decided from the documents as a
  controller dispute and never from the BIOS's code.

## 3. The memory card manager: scope

### 3.1 What it does

| Operation | How | Refused, in words, when |
|---|---|---|
| **List** both slots' saves | per slot: each save's icon, title, blocks and the game's serial from its file name; the free blocks; the card's state | — (shown as no card, not formatted, or unreadable) |
| **Copy** a save to the other slot | read whole into user RAM, create, write; a progress bar by sectors | the other card absent or unformatted; the same file name already there; too few free blocks |
| **Delete** a save | `erase`, after a confirmation | — |
| **Format** a card | `format`, after a confirmation naming what is lost | no card |

**Out of scope here** [C]: move, which is copy and delete; undelete, which the kernel has; a whole card copied;
multitap slots ("bu01:" and on), which the kernel serves; and renaming. Each is a later row of the same module, and none
needs a new surface.

**Save icons.** From psx-spx's "Title Frame" and "Icon Frame(s)" [R]:

- The title frame's byte 02h is 11h, 12h or 13h for one, two or three 16 × 16 frames of 4 bits.
- Bytes 60h–7Fh hold its 16-colour CLUT, where 0000h is fully transparent.
- Two frames change "every 16 PAL frames", three "every 11".
- "Values other than 11h..13h seem to be treated as corrupted file". Such a save is listed as *Damaged save* with its
  blocks, so that it can still be deleted [C].

The shell uploads each icon and its CLUT to VRAM outside the display and draws it as a textured rectangle at 1:1. The
animation follows the documented rates, measured in the console's own frames [C]: the rates describe how the game's art
is meant to move, not Sony's look. psx-spx's note that Sony's menu draws icons "via GP0(2Ch)", as quadrilaterals, is not
followed. The drawing call is the shell's own.

**Titles.** The title frame's Shift-JIS is shown by `charset.rs`'s conversion, which already serves `manager.rs` and
Krom2RawAdd. The full-width Latin letters, digits and punctuation become their ASCII forms. Single bytes 20h–7Eh are
kept, "the BIOS memory card manager does also accept 8bit characters 20h..7Fh" [R]. Every other character is drawn as a
box, the kernel's rule since `Diamond_OpenKernel_Phase2.md` §25.1. A title of boxes alone is followed by the serial,
so a Japanese save can still be told apart (§9 Q12).

**Pace.** The kernel carries out a card request one sector a slot every second frame, as psx-spx describes the original
("Poor Memcard Performance"; `Diamond_OpenKernel_Phase2.md` §13) [R]. A copy reads and writes 64 sectors a block, so
about four seconds a block, and about a minute for 15 blocks [A]. The progress bar counts sectors, and the screen says
not to remove the card. That a copy takes time is the console's behaviour, not a defect to optimise away.

### 3.2 Layout and controls

The layout is EmuSen's own [C, §9 Q11]. No picture, video or description of Sony's shell beyond psx-spx's paragraph
"BIOS Bootmenu" was consulted for it.

- **Home.** A title line, a line for the drive's state ("No disc", "Audio CD", "Disc found, starting"), and the
  available screens as a list: *Memory Cards* always; *CD Player* once the module of §5 exists and the drive holds an
  audio disc.
- **Memory Cards.**
  - One slot at a time, as two tabs, *Slot 1* and *Slot 2*.
  - Under the chosen tab, a vertical list of rows: the icon, the title and the blocks, with the serial in a dimmer
    colour. Nine rows are visible at 320 × 240, and the list scrolls through up to fifteen.
  - A footer for the free blocks, and a status line for what an action did or why it did nothing.
- **Confirmation.** A sheet over the list names the action and its object ("Delete *Castlevania* from the card in Slot
  1?", "Format the card in Slot 2? Every save on it is erased."). It offers *Delete* or *Format* and *Cancel*, with the
  cursor on *Cancel* [C, §9 Q13]. This replaces stage 10's press-twice confirmation, which gives no sight of what will
  be lost.
- **Colours.** LunaP's dark theme's tokens, reduced to 15 bits, so the shell reads as part of EmuSen. Text is
  `diamond-guest`'s font.

**The controls** follow EmuSen's button grammar (`EmuSen_LunaP_Kit.md`; `EmuSen_Settings_Reference.md` §4.29) on the
emulated pad. Positions are the grammar's, so a player's bindings that put Cross in the south place keep it.

| Pad | Home | Memory Cards | A sheet |
|---|---|---|---|
| D-pad up and down | move | move through the saves | move |
| Cross (south) | open | the chosen save's actions: *Copy to Slot n*, *Delete*, *Cancel* | accept |
| Circle (east) | — | back to Home | cancel |
| Triangle (north) | — | the card's actions: *Format*, *Cancel* | — |
| L1, R1 | — | Slot 1, Slot 2 | — |
| L2, R2 | first, last | first save, last save | — |
| Start | Home from any screen | Home | — |

Directions and shoulders repeat after 400 ms and then every 80 ms, counted in vblanks. Nothing else repeats. The
analogue sticks are not read, so a digital pad and a DualShock in either mode both work [C].

### 3.3 What the shell prints

Each action writes one line to the TTY, the kernel's documented channel, which Diamond's TTY reader collects. Examples
are `shell: copy 1 BASLUS-00067DRACULA 2 ok` and `shell: format 2 ok`. This is a plain-text record that tests read
(§4) and costs a player nothing.

## 4. How the shell is written and tested without a window

### 4.1 The crate

`kernel-shell/`, beside `kernel/` and `kernel-harness/`, is a `no_std` binary for `mipsel-sony-psx`. Its modules:

- **The logic**, with no hardware access: the screens' state machines driven by presses, the layout, the title
  conversion, and the decoding of the icon and its CLUT. Its unit tests run on the host by `cargo test`.
- **The services**:
  - pads, through InitPAD2's buffers;
  - cards, through the file calls;
  - the drive: Nop and GetID at the controller's ports;
  - the clock, in vblanks;
  - drawing, through `diamond-guest`.
- **The screens**: Home, and Memory Cards.

The kernel's reproducible build is extended to the shell, which is committed with its hash.

### 4.2 Tests in Diamond's crate

`tests/shell.rs` runs the machine on the open kernel with no disc, as `the_screen_with_no_disc_…` does today. Its
inputs are synthetic:

- cards built by a test module from psx-spx's layout, holding saves with title frames, icons of one to three frames,
  a damaged title frame, and a full card;
- presses scripted by frame.

Each test asserts the TTY's lines and the cards' bytes afterwards. The cases:

1. Every operation of §3.1, and each refusal in its row.
2. A card changed while the shell runs: removed, put back, swapped.
3. A disc put in, which ends the shell and is booted by the kernel. The test also checks the hand-over's restorations
   of §1.3 at the boot program's entry (I_MASK, the pads and cards stopped, no CD command outstanding).
4. An audio disc, reason 1: the shell's home says so. The test is written once §5.1 exists.
5. The shell as a PS-EXE side-loaded on the open kernel, which proves it uses no path but the call tables.
6. A negative control: a build whose delete skips its confirmation fails case 1.

Stage 10's test moves to the shell, with its assertions kept.

### 4.3 Tests in WiseMan

`DiamondShellTests` start the console through `CoreEngine.StartConsole("psx")` headless, with drawer cards made by
`PsxCardFormat`, and press through the input path a frontend uses. Each test asserts what `PsxCardFormat.Read` finds on
the card file afterwards. So the host's owner of the format judges the guest's writes, and the guest lists cards the
host made. The cases:

- the session's lifecycle: started empty, a synthetic disc inserted by `InsertDisc(path)` and booted (its TTY
  marker), ejected, a state saved and loaded with the drive empty and with a disc;
- the launch record's readers: no play time, no Continue Playing entry, states under the console's name;
- in Mistress, the new rows and the *Insert Disc…* chooser under `WindowFitAuditTests` at both sizes and under the
  pad audit, and the entries shown only for a console whose core info says `starts_empty`.

### 4.4 The kit

C22 (§1.1) is run on Diamond's library and on `emusen-native`'s test core, which gains a system that starts empty, so
that the case is held without Diamond.

### 4.5 With a player's BIOS, by another writer

A step local to the tester's machine and never in CI checks two things. First, that the player's BIOS reaches its own
menu with an empty drive: the picture is not black after a set number of frames and changes when a button is pressed.
Only that is recorded. Second, that the open shell, run as a program on the player's BIOS, leaves cards equal by the
on-card format to the same presses on the open kernel. This is the cross-kernel method of
`Diamond_OpenKernel_Phase2.md` §22, done by a writer who does not write the shell (§9 P1, P2).

## 5. The seams for the audio CD player and its visualizer

Nothing below is built in this work. Each seam is a place where the player plugs in without changing a surface named
above.

### 5.1 The disc

- **Into the core.** An audio disc is a cue sheet with audio tracks only, given to `media_insert`, with Diamond
  accepting the third outcome of §1.2. `create` keeps refusing it, since the console starts empty and the disc goes in
  afterwards. Whether `emusen-disc` opens a sheet with no data track is to be checked when this is built. Its tracks
  are a sheet's like any other (`EmuSen_Disc.md`).
- **Into the host.** *Insert Disc…* passes any file to the running core, and no routing is involved. A folder for
  music discs in Galaxia, and how the chooser lists them, are decided with the player.
- **With a player's BIOS.** Once the core accepts the disc, the BIOS's own CD player has what it needs. GetID answers
  "Audio Disk", and Play, the reports, AutoPause, GetTN and GetTD are served (`Diamond_CDROM.md` §4, §7). Nothing in the
  open shell is involved.

### 5.2 The player

- **A module of the shell.** It is a second screen beside Memory Cards, listed on Home when the drive holds an audio
  disc. The module registers itself in the shell's screen table. Home and Memory Cards are not changed by its arrival.
- **It drives the controller directly**, as psx-spx says a program must: the kernel has no CD audio calls. Its parts
  are already modelled for games:
  - Demute and Play, with Setmode's report and autopause bits;
  - GetTN and GetTD for the track list;
  - Forward and Backward;
  - the report's track, index and times for its display;
  - the SPU's CD volume (`AVOLL`, `AVOLR`) and the controller's matrix (`ATV0`–`ATV3`).
  The kernel leaves the controller to it while the kernel owns no operation (§2). The shell's drive service, the one
  owner of the drive's polling, hands the player the drive's state.

### 5.3 The visualizer

- **A part of the player, fed by a shell service.** psx-spx documents that SPU RAM 00000h–003FFh and 00400h–007FFh
  hold "CD Audio left" and "CD Audio right … before Volume processing, signed 16bit samples at 44.1kHz", and that
  "CD-Audio capture is always active" [R]. The capture IRQ can pace a reader (`soundprocessingunitspu.md`, "Capture
  Interrupt"). Diamond records the capture buffers sample for sample (`Diamond_SPU.md` §8).
- **Its parts.** A `capture` service reads the buffers through the SPU's data transfer each frame. The visualizer
  turns them into a picture drawn by the emulated GPU, so it is exact, deterministic and part of a state. The report's
  peak is a cheaper second source.
- **Who has it.** This visualizer exists only on the open shell. A player on their own BIOS sees that BIOS's CD player
  as it is.

### 5.4 A visualizer in the frontend instead, or as well

A frontend visualizer would draw from the host's audio stream over any core's picture. It would serve the Sega CD and a
player's BIOS too. In form it is a component of its own, a renderer over `AudioPlayer`'s samples that touches no core.
It does not displace the in-shell form. §9 Q10 asks which the tester means, or whether both.

## 6. Phasing

Each step ends with its tests passing, its record written, and the tree merged into WiseMan. A player can use what each
step delivers.

| Step | What it builds | Its gate | What a player gets |
|---|---|---|---|
| S1 | Core ABI 1.11 (the normative section, the header, the baseline, `emusen-native`, the C# struct); C22 on the test core; Diamond's `starts_empty`, optional `Media` and the market rule | the kit compliant at 1.11 on Diamond's sample and on an empty start; Diamond's goldens unchanged | nothing visible yet |
| S2 | The runtime's launch record, `StartConsole`, `InsertDisc(path)` and `EjectDisc`; Mistress's entries and rows; the chooser; DianaSh's `core --console` | `DiamondShellTests`' lifecycle cases; the fit and pad audits; Diamond's and Nephrite's media tests | the console started empty: stage 10's screen and manager on the open kernel, the BIOS's own menu with a player's BIOS; a disc put in afterwards |
| S3 | `diamond-guest`; `kernel-shell` with stage 10's manager moved into it, behaviour unchanged; the hand-over; the image and `firmwarecheck.py` | stage 10's test passing on the moved shell; case 5 of §4.2; the kernel harness's 49 tests; every game of `Diamond_OpenKernel_Phase2.md` §18 booting as before; the image rebuilt to its hash | the same screen, now a program |
| S4 | The manager of §3: icons, titles, the layout, the sheets, the controls, the TTY lines | §4.2's cases; WiseMan's card cases judged by `PsxCardFormat` | the memory card manager |
| S5 | Optional, by §9 Q9: audio discs accepted (§5.1) | an audio sheet inserted; GetID's answer; Play reaching the output through the open shell's reason 1 | with a player's BIOS, its CD player; on the open shell, "Audio CD" on Home |
| — | The CD player and the visualizer (§5.2–§5.4) | — | not in this work |

S1 and S2 touch the framework. S3 and S4 touch only Diamond's guest programs. S2 needs no change to the open kernel.

## 7. Risks

1. **Paths assumed to be games'.** `RomPath` is read by states, screenshots, cheats, play time, Continue Playing, the
   launch screen and the resume prompt. A reader missed would misfile a console session's data. *Mitigation*: the launch
   record replaces the path in one step, every reader is listed in that step's record, and WiseMan's lifecycle test
   covers each one.
2. **The player's BIOS with an empty drive.** It may stall as the Sega CD's did. The cause would be the controller's
   answers, found only by observation. *Mitigation*: the step of §4.5; a dispute decided from psx-spx. The open shell
   does not depend on it.
3. **The look.** A shell that resembled Sony's by accident would undo the clean room's purpose. *Mitigation*: the
   layout is the project's own interface (§3.2); §9 P1 asks for the rule that keeps the shell's writer away from Sony's
   shell.
4. **The image.** It grows, and `firmwarecheck.py` measures repetition of the original. The shell's code shares
   nothing with Sony's, but the image's statistics change. *Mitigation*: the policy's rule run unchanged (§9 P3).
5. **Card writes during a state load.** The shell writes through the kernel, so a state loaded mid-copy rolls the card
   back exactly as it would a game's save, under the card's mode (`EmuSen_SaveMedia.md` §3.4). Nothing new arises, but
   the lifecycle test includes it.
6. **Kernel RAM.** The hand-over takes a few words, and the kernel's other variables shrink as the manager leaves.
   *Mitigation*: the linker script's existing assertion at DF80h.
7. **Japanese titles.** The kernel carries no Japanese font, by policy. Japanese saves show boxes and their serial.
   *Mitigation*: said in the shell's words and in the firmware entry's cost, which already names the boxes.

## 8. Choices where the documents are silent

1. With no disc, the market is the first player's BIOS passed, in the order Americas, Europe, Japan, else the Americas
   (§1.2).
2. The hand-over: the shell at 80010000h; `Exec`'s second parameter the reason; the restorations; return 0 (§1.3).
3. Shift-JIS titles shown as ASCII where `charset.rs` maps them, otherwise as boxes and the serial (§3.1).
4. Damaged title frames are listed, so they can be deleted (§3.1).
5. Icon animation at the documented rates in console frames; icons drawn as 1:1 textured rectangles (§3.1).
6. The layout, words, colours and controls (§3.2), with the confirmation sheet replacing press-twice.
7. Move, undelete, whole-card copy, multitap slots and rename are left out (§3.1).
8. The TTY line per action (§3.3).
9. The pads read digitally only (§3.2).
10. In the ABI, `system` is ignored when an image is given (§1.1).

## 9. Questions for the tester

*Decided 2026-10-17 by the tester: Q1–Q13 and P1–P4, each as recommended.*

1. **Q1, the ABI's form.** Core ABI 1.11: `starts_empty` per system in core info, a null image in `create` for such a
   system, and `create_params.system` appended (§1.1). *Recommended*: yes. The alternatives are a library bit (wrong
   for Nephrite's two systems), a second create export, or an undeclared convention.
2. **Q2, the shell's form.** A program of its own, carried in the BIOS image and started by `Exec`, reaching the kernel
   only through the call tables, against a module inside the kernel as today (§1.3). *Recommended*: a program of its
   own.
3. **Q3, a shared guest crate.** `diamond-guest`, holding the font, the Shift-JIS conversion and the GPU's primitives,
   linked by the kernel and the shell (§1.3). *Recommended*: yes, by the rule of one copy.
4. **Q4, the region with no disc.** The market of a player's BIOS present (Americas, then Europe, then Japan), else the
   Americas, unless the Region setting names one (§1.2). *Recommended*: yes. The alternative, always the Americas,
   leaves a player's only BIOS unused when it is Japanese or European.
5. **Q5, cards under *Per game*.** A console session has no game whose card to use. *Recommended*: the session uses the
   shared cards, `Card 1` and `Card 2`, whatever the mode, and says so in the status line. A disc booted from the
   session keeps them, as a card stays in a console's slot. The alternative is a card of the session's own, which no
   game would ever see.
6. **Q6, the entries.** *Start PlayStation* on the desktop library's console filter and in *Library → Start Console*;
   in big picture the first row of *Gamelist Options* and *Main Menu → Library → Start a Console*; *Insert Disc…* and
   *Eject Disc* in a running session's menu (§1.5). *Recommended*: these, with no pseudo-game in any gamelist. A
   pseudo-game would be scraped, counted, filtered and favourited as if it were a game.
7. **Q7, the chooser.** *Insert Disc…* lists the console's library games, with *Browse…* on the desktop.
   *Recommended*: yes. Music discs' place is decided with the player.
8. **Q8, a console session's records.** States are saved under the console's name. The session is not recorded in
   Continue Playing or play time, and no resume is offered on leaving. A state made with a disc in the drive needs that
   disc in the drive to load, and the refusal says so. *Recommended*: yes.
9. **Q9, audio discs now or later.** Accepting an audio disc (S5) is small, and with a player's BIOS it makes the
   BIOS's CD player work at once. The open shell would only say "Audio CD" until its player exists. *Recommended*:
   build S5 in this work, since it is the seam's own test. The request "build none of the player" would then still
   hold for the open shell.
10. **Q10, which visualizer.** In the open shell's CD player, drawn by the emulated GPU from the SPU's capture buffers
    (PlayStation only, open shell only), or a frontend component over any core's sound (works with a player's BIOS and
    the Sega CD), or both (§5.3, §5.4). *Recommended*: decide when the player is planned. This design leaves both seams
    open, and neither is built.
11. **Q11, the look.** One slot at a time as tabs, a scrolling list with icons, LunaP's dark theme reduced to 15 bits,
    and the controls of §3.2. *Recommended*: yes.
12. **Q12, Japanese titles.** Boxes for characters the font lacks, followed by the serial. A Shift-JIS font of the
    project's own is out of scope. *Recommended*: yes.
13. **Q13, confirmations.** A sheet with the cursor on *Cancel*, replacing stage 10's press-twice. *Recommended*: yes.

**Standing rules and policies, each asked on its own:**

- **P1, a clean-room rule for the shell's look.** The shell's writer does not view Sony's shell: no pictures, videos,
  screenshots, or descriptions of its layout beyond psx-spx's "BIOS Bootmenu" paragraph and its note on icons. The
  checks with a player's BIOS (§4.5) are made by another writer, and the record keeps only "reached its menu" and
  "responds to a press", never a picture. *Recommended*: adopt it for this work and for Nephrite's open BIOS's screen
  with no disc.
- **P2, the open shell run on a player's BIOS.** Running the shell as a program on the player's BIOS, to compare the
  cards it writes by their on-card format, treats it as a game run as a black box, which the plan's §1.3 allows a
  writer. *Recommended*: allowed, by a writer who does not write the shell, with outputs only.
- **P3, `firmwarecheck.py` on the enlarged image.** The image grows by the shell. *Recommended*: run the policy's rule
  of `EmuSen_Firmware.md` §0 unchanged, the fill excluded and the control as decided on 2026-10-11, and record the
  result before S3 merges.
- **P4, a second committed guest binary.** The shell's PS-EXE, built by the pinned toolchain, is committed with its
  hash and rebuilt to the same bytes, as the kernel image is (`Diamond_OpenKernel.md` §6). *Recommended*: the same rule.

## 10. Reading log

Every document and file opened for this page, 2026-10-17. **Never opened**:

- anything under `~/Documents/BIOS/` or the probe's system folders;
- any BIOS image, listing, disassembly or trace;
- any picture, video or description of Sony's shell other than psx-spx's paragraph;
- any emulator's source, OpenBIOS, PSn00bSDK or PsyQ;
- `Diamond_References.md`;
- psx-spx's excluded ranges in `DOCS.txt`.

No machine was run.

- **psx-spx** at `6d7d1bc`:
  - `cdromdrive.md`: its heading list, and lines 818–1100 (the status byte, the error bytes, Nop, Getparam, GetTN,
    GetTD, GetQ, GetID, the CD audio commands, Report, AutoPause);
  - `controllersandmemorycards.md`: its heading list, and lines 2454–2640 (the data format, the title and icon frames,
    Shift-JIS, the images);
  - `kernelbios.md`: its heading list to line 1300, lines 32–44 (the overview and "BIOS Bootmenu"), 734–745 (Exec)
    and 856–860 ("Absent CD-Audio Support");
  - `soundprocessingunitspu.md`: a search for the capture buffers and the CD volume, its matching lines read (53–54,
    60–74, 411–418, 604–636, 809–838);
  - `DOCS.txt` first.
- **EmuSen's pages**:
  - `Diamond_OpenKernel_Phase2.md` §12–§21;
  - `Diamond_OpenKernel.md`'s headings;
  - `Diamond_Plan.md` §1.3 and §5.9–§5.11;
  - `Diamond_CDROM.md` §4–§8 and §11–§14;
  - the Diamond folder's `README.md`;
  - `EmuSen_CoreAPI.md` §6.2, §6.3, §6.23–§6.26, the changelog's opening, and its headings;
  - `EmuSen_SaveMedia.md` §3.3, §3.4, §6, §9, §11.5–§11.7;
  - `Nephrite_Native.md` §49.2–§49.6, §55, §56 and §63;
  - `Nephrite_OpenBios.md` §1.5, §16 and §17;
  - `Nephrite_Bram.md` §11;
  - `EmuSen_LunaP_Kit.md`'s grammar line;
  - `EmuSen_Settings_Reference.md` §4.29's opening;
  - DianaSh's `core.1`.
- **Code**:
  - the kernel's `shell.rs`, `screen.rs`, `manager.rs`, `Cargo.toml`, `kernel.ld` and `build.rs`;
  - Diamond's `media.rs` and `medium.rs`;
  - parts of `v1.rs` (create, the media exports, core info);
  - parts of `machine.rs` (`new`, the disc and lid calls);
  - parts of `cdrom/mod.rs` and `cdrom/commands.rs` (power-on, the status, GetID);
  - `tests/programs.rs` lines 1160–1215;
  - `emusen_core.h`'s header, status codes, capabilities and structs;
  - `CoreEngine.cs` lines 200–376;
  - Mistress's `MainWindow.axaml.cs` lines 920–990;
  - a search of `PsxCardFormat.cs`.

---

## 11. Build record

### 11.1 S1: core ABI 1.11 and Diamond's empty start (2026-10-17)

**Built.** `EmuSen_CoreAPI.md` §6.31 is the normative text, §28.17 the kit's case and its changelog the summary. The
header's `emusen_create_params.system` (offset 104, 112 bytes), core info's `systems[].starts_empty`, the baseline
regenerated to 483 facts with `--check` and `--against` 1.10 passing, `emusen-native`'s check in the create export, the
guard's checker, the C# host's `CoreInterface.CreateParams.System`, `CoreSystem.StartsEmpty` and `CoreMachine`'s
`system`, kit case C22, the test core's empty start and `FAULT_EMPTY_LOADED`.

**One departure from §1.1, argued.** §1.1 proposed `Create.image: Option<&[u8]>` and `Create.system: Option<&str>` on
the Rust side. Both are fields of a struct every core builds in its tests by a literal, so either would have changed
six cores' code for a feature one of them uses. The surface became two defaulted trait functions instead,
`Core::starts_empty(system)` and `Core::create_empty(system, request)`, beside `battery_files(system)`, which is how
per-system facts already reach core info. A core that does not opt in compiles and behaves as before. The C ABI is as
§1.1 gave it.

**Diamond.** `starts_empty("psx")`; `create_empty` makes the machine of `Media { kind: Empty, market }` through the same
`Machine::make` a disc's create now calls, so files, settings, kernels, cards and multitaps are checked and placed by one
path. The market is that of the first player's BIOS given of the Americas, Europe and Japan, else the Americas, and the
Region setting overrules it (§9 Q4). With no kind to side-load, the machine starts at the reset vector on its kernel.
`the_console_starts_empty_and_its_market_comes_from_the_bios_given`: the drive is listed empty with its lid closed;
machine info names the Americas and the open kernel; after 420 frames VRAM holds the open kernel's screen with no disc;
a Japanese, a European-and-Japanese and all three BIOS files give Japan, Europe and the Americas, each named `File`;
`region=psx.us` beside a European BIOS gives the Americas on the open kernel; an unnamed file is refused.

**Nephrite** does not claim the field. With a player's BIOS its Sega CD stays on its first picture with an empty
drive (`Nephrite_Bram.md` §11, `Nephrite_OpenBios.md` §16), so it does not yet start empty correctly. That is
Nephrite's follow-up, not built here: an empty start for `mcd`, once the stall is explained.

**Measured (2026-10-17).**
- `emusen-native`: 77 tests, one new.
- The guard: its 6 tests.
- The kit: its 24 tests and 8 unit tests.
- Diamond: its 174 unit tests, and `diamond-v1`'s 4.
- The kit on Diamond's library with Crash Bandicoot's disc at 600 frames: C1–C22 pass, 71 s.
