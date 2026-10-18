# EmuSen — Firmware

Covers `EmuSen/Common/Firmware/`, `ICore.GetFirmwareRequirements`, and the picker in `EmuSen.Mistress`. Core-agnostic by design: the SNES's NEC DSP is the first user, but a PlayStation BIOS, a Game Boy boot ROM, or a Saturn IPL differ from it only in the values they fill in.

> **Status: complete.** Discovery, validation, install, and the Avalonia picker all work and are tested. The SNES core is wired up; every other core inherits the empty default and asks for nothing.

---

## 0. The policy (decided 2026-10-03)

For every emulator the project builds, present and future:

1. **An open version first.** Wherever real hardware runs code its maker shipped inside the machine or the cartridge
   (a boot ROM, a BIOS, a coprocessor's program), the core carries its own replacement, written from public
   documentation of what that code does. Every game the core supports then runs with no firmware folder.
2. **The player's own firmware, if they have it.** An image placed in `home/Firmware` is used in place of the
   replacement, as the exact path. It is offered, never required, and no frontend prompts for it. *Since
   2026-10-17* a player may also name a folder of their own in Preferences (`FirmwareDirectory`, §2.3), searched
   before `home/Firmware`. The setting is consistent with this policy: it is optional and blank by default, nothing
   asks for it at launch or says that firmware is missing, and EmuSen only reads the folder.
3. **Nothing copyrighted ships or is fetched.** No firmware is committed, bundled or downloaded on the player's behalf
   (§5); the unofficial BIOS collections hosted online are unauthorised copies and are not a source.
4. **A core may state the checksums of the firmware it accepts.** *Decided 2026-10-18 by the tester, as a standing
   rule:* a core's info may list the MD5s of the real firmware images it accepts (`EmuSen_CoreAPI.md` §6.32), each
   cited from a public description of the images, as libretro's core info files publish them. A checksum identifies an
   image and holds none of its bytes; the images themselves are still never shipped, fetched or included as data.
   Checksums are never computed from a player's dumps, nor from the images in a core's probe cache, which
   `firmwarecheck.py` alone reads (below). The store uses them to recognise a player's file by its content (§7).

Each replacement states its accuracy cost in its core's documentation and in the core's info, as any performance
setting does. Each also states how much of the original it repeats, measured by
`EmuSen.WiseMan/Reference/analysis/firmwarecheck.py <replacement> <original>`: the bytes equal at the same offset and
the longest run common to both at any offsets, by default at most 25% and no run over 6 bytes
(`EmuSen_Debugging_Tools_Reference_v5.md` §3.61). The tool prints numbers and offsets only. The original it compares
against is kept outside the repository, in the core's probe cache, and nothing else reads it. The rule does not bind a foreign core (`EmuSen_CoreAPI.md` §6.2). The first cases are VenusRT's SPC700
boot program and its NEC DSP chips (`VenusRT_Native.md`). The first kernel-sized one is Diamond's open PlayStation
BIOS, measured 2026-10-11 against the 18 images in its probe cache's firmware folder (`Diamond_OpenKernel_Phase2.md`
§22.6): FAIL on all, by the default thresholds, plain and on the image's 92,440 bytes of content, every long run in
its zero fill and the few others forced MIPS idioms. A literal run limit cannot pass compiled code: unrelated compiled
MIPS code shares more with the same images (9,931 runs of 9+ bytes, the longest 32) than the kernel does. *Decided 2026-10-11 (Q5)*: a replacement that is compiled code is judged against a **control**, unrelated code of the same processor and size, with fill left out (`--ignore-fill 8 --min-run 9 --control CONTROL --explained FILE`): it passes when at most 25% of its bytes are equal at the same offset, it shares no more runs of 9 bytes or more with the original than the control does, and every run longer than the control's longest lies in a listed pattern of its own words with the reason the match is forced or generic. For Diamond the control is the first 92,440 bytes of ps1-tests' release programs' bodies, built at run time by `EmuSen.WiseMan/Reference/analysis/firmware/diamond-control.py`, and the patterns are `firmware/diamond-explained.txt` (`setjmp` and `longjmp` storing and loading the registers in the documented buffer order, and the compiler's register save and restore at every frame offset). Measured the same day on the kernel image of SHA-256 7af83c9b… against all 18 images, each compared whole: **PASS on all**, 13.5–15.1% equal at the same offset against the control's 11.0–11.3%, 6,394–16,514 shared runs of 9+ bytes, on every image fewer than the control's 9,916–24,257 with the same image, and 15–26 runs past the control's longest of 32–33 bytes, all explained. The check is rerun whenever the kernel's image changes. *Rerun 2026-10-12* on the image of SHA-256 4e01c2ac… (the font, the asynchronous card calls and the exception path in RAM), with a control of 100,544 bytes to match its length: **PASS on all 18**, 13.2–15.6% equal at the same offset against the control's 10.6–11.9%, 6,764–17,157 shared runs of 9+ bytes, on every image fewer than the control's 10,143–24,766 with the same image, and 20–29 runs past the control's longest of 32–33 bytes, all explained. *Rerun 2026-10-12* on the image of SHA-256 c1a002e8… (Mega Man Legends: fill with sparse returns behind B(5Bh)'s entry, LoadExec and the synchronous CD operations), with a control of 103,256 bytes: **PASS on all 18**, 13.4–15.1% equal at the same offset against the control's 10.6–11.9%, 6,859–17,479 shared runs of 9+ bytes, on every image fewer than the control's 10,915–26,613 with the same image, and 20–29 runs past the control's longest of 32–33 bytes, all explained. An image built before it, with 4 KiB of `jr ra` and `nop` pairs in that place, failed on all 18 by its shared runs, and was not merged (`Diamond_OpenKernel_Phase2.md` §26.8). *Rerun 2026-10-17* on the image of SHA-256 92dad42f… (the console shell in its own section at 60000h, `Diamond_Shell.md` S3–S4), run apart by a checker who wrote neither the kernel nor the shell, with a control of 120,840 bytes, the kernel's 88,072 and the shell's 32,768: **PASS on all 18** compared whole, 12.7–18.6% equal at the same offset against the control's 10.4–11.7%, 7,005–17,881 shared runs of 9+ bytes, on every image fewer than the control's, and 20–27 runs past the control's longest, all in the kernel part and all explained; the kernel and the shell measured apart each pass on all 18. The shell measured alone at its own offsets is 27.9% equal on one of the fifteen others, its aligned runs at most 6 bytes; whether a part alone must meet the 25% is left to the tester (`Diamond_OpenKernel_Phase2.md` §28).

---

## 1. The contract

Some hardware needs code the software never shipped with. A DSP-1's program is inside the chip, not on the cartridge; a PlayStation cannot boot without its BIOS. Emulating that hardware means having a dump, and **EmuSen does not ship one** — see §5.

Three pieces make that tractable:

**`FirmwareRequest`** — one image, described in the abstract: which core wants it, which chip it is, the canonical filename, the exact byte count, and a one-line human explanation of what it's for. Nothing console-specific.

**`ICore.GetFirmwareRequirements(romPath)`** — what a given ROM will need, answered **without loading it**. This is a default interface method returning nothing, so a core whose hardware needs no firmware inherits the right behaviour for free. `VenusCore` overrides it and delegates to `Cartridge.FirmwareRequirements`, which reads the header and stops.

Answering before loading is the whole point. Once `LoadRom` has run, the core has already come up with the chip absent, and the only fix is to load again — so a frontend that wants to *offer* the player a way out has to ask first.

**A missing image is never fatal.** `LoadRom` still succeeds; the core runs with that chip missing. That is what makes the prompt optional rather than a gate, and it is why `Pharaoh` needs no firmware code at all.

---

## 2. The library

`FirmwareLibrary` is `home/Firmware/` treated as a store, with the player's chosen folder searched before it (§2.3), and it knows nothing about any console.

| Call | Meaning |
|---|---|
| `Folders` | the folders searched, first to last (§2.3) |
| `Find(request)` | the first folder holding the image, whole or as a split form, with its bytes; or null |
| `TryLoad(request)` | the bytes, or null |
| `IsInstalled(request)` | as above, as a bool |
| `MissingFrom(requests)` | filters a batch down to what's absent |
| `Install(request, sourcePath)` | copies a file the player picked into `home/Firmware`, under the canonical name |
| `PathFor(request)` | where it belongs in `home/Firmware`, for an error message |

`Directory` is settable, and `ResetDirectory()` puts it back. While it is set it is the only folder searched, and the
player's setting is not read. That exists so tests can point at a temp folder — without it every test's result would depend on which dumps happen to be on the machine, which is a bug this project has already been bitten by once (see §6).

### 2.1 Size is the only validation

An image whose length doesn't match `Size` is treated as **absent**, not loaded. A truncated or mismatched dump executes as garbage and produces a bewildering failure a long way from its cause; "not found" is a far better diagnostic. `Install` applies the same rule, and refuses to copy anything that fails it.

*Since 2026-10-17* a file given as a symbolic link is measured by what it links to, which is what is read, and a link
to nothing is neither the image nor a file of the wrong size. Before, the size compared was the link's own, the length
of the path it holds, so a linked image of the right size was passed over and a link whose path happened to be as
long as the image passed off a file of another size (`EmuSen_Backend.md` §15.1, shown by two tests of
`FirmwareLibraryTests`).

*Since 2026-10-18* a request may also list checksums, and a file of the right size in a searched folder is then
recognised by its content whatever its name (§7). A checksum adds finds and refuses none: a file under a declared name
and of the declared size is taken as before, whatever its content. Size alone already rejects every accident seen in practice — the wrong chip's dump, a half-downloaded file, a ROM picked by mistake.

### 2.2 Canonical names and alternates

`AlternateNames` lists other filenames the same dump circulates under. They are checked after the canonical name and **never written to**: an install always lands under `FileName`, so the next launch finds it first and the store converges on one layout no matter what the player picked.

### 2.3 The player's firmware folder (2026-10-17)

*Requested by the tester 2026-10-17:* a player can choose a firmware folder in Mistress. Many players already keep
their dumps in one folder (`~/Documents/BIOS`, or the `system` folder of another emulator) and should not have to
copy them into the sandbox.

**The setting.** `AppSettings.FirmwareDirectory`, blank by default, set on Preferences ▸ Firmware
(`EmuSen_Settings_Reference.md` §4.121). It is in the platform library's model of `appsettings.json` as well
(`Platform/galaxia/src/models.rs`), since that model drops a field it does not list and a library-side save would
otherwise erase the choice. Its value is read by the folder rules of `EmuSen_Config_Reference.md` §7.7: blank is
unset, `~` and `~/` are the home, a relative path or a file is refused, and a folder that does not exist is said.

**The order.** `SettingFolders.Firmware(settings)` in Galaxia is the one owner of the lookup order, and
`FirmwareLibrary.Folders` asks it (*since 2026-10-17 its Rust half, `emusen-firmware`'s `folders` over
`emusen-galaxia`'s `setting_folders`, answers under `EMUSEN_SERVICES_NATIVE`, `EmuSen_Backend.md` §15.1*); the cores' hand-over (`CoreEngine.LoadRom` and `StartConsole`), the question
before a launch (`EmulatorSession.MissingFirmwareFor`) and the Firmware page (`FirmwareOverview.Build`) all search
those folders, through `FirmwareLibrary.Find`, and no other code builds a firmware path to read.

1. The chosen folder, when the setting names one that the rules accept. A folder that does not exist is still
   searched, and holds nothing; a drive mounted later then makes it real without a second visit to Preferences.
2. `home/Firmware`, always. When the chosen folder is `home/Firmware` itself it is searched once.

Within a folder the order of §2.2 holds (the canonical name, then the alternates, then each split form, then, *since
2026-10-18*, the folder's files of the declared size by their checksum, §7.4), and a
split form is taken only when all of its parts are in the same folder; parts are never gathered from two folders,
since two halves of different dumps would pass the size test and run as garbage. The first folder that holds a
usable form wins, so on a tie of name and size the chosen folder's file is the one used.

*Why the chosen folder first.* The order matters only when both folders hold a file that passes the size test, and
then the question is which of two plausible files the player meant. `home/Firmware` is filled by EmuSen's own
install path (`Install`, §3) and by whatever an older build or another session left there; the chosen folder is a
choice made today, in Preferences, for exactly this purpose. Taking the explicit choice over the implicit one is the
rule a player can predict without reading this page, and it is the order a search path has elsewhere (a user's
directory ahead of the system's). The opposite order would make the setting do nothing for every file the sandbox
already holds, which is the case a player sets it to fix. A file of the wrong size is not a tie (§2.1): it is not
the image, and the search continues to `home/Firmware`.

**Read only.** EmuSen never writes, renames or deletes anything in the chosen folder, the same rule as the ROM
library (`EmuSen_Galaxia.md` §3.3a). `Install` writes only to `home/Firmware`; the page's Open Folder makes and opens
only `home/Firmware`; nothing makes the chosen folder when it is absent. Tested by
`NephriteFirmwareTests.A_bios_in_the_chosen_folder_boots_the_log_names_the_folder_and_the_folder_is_never_written`,
which compares the folder's entries, lengths and modification times, and the folder's own, before and after two
sessions, and by `FirmwareFolderTests.Install_writes_into_home_firmware_and_never_into_the_chosen_folder`.

**What it costs.** `FirmwareLibrary.Folders` reads `appsettings.json` once per load, per launch question and per
build of the page, not once per file; the folders are passed down from there.

**Not covered.** A Hotaru or Pharaoh session reads the same setting through the same library, but neither has a
place to set it; a hand edit of `appsettings.json` is the way there. No DianaSh command shows or sets folder
settings (`EmuSen_Config_Reference.md` §7.2), so none was given this one; `man hier` names it beside `/Firmware`.

---

*Decided 2026-10-17 by the tester, each as recommended:* the order above, the player's folder first and then
`home/Firmware`; the setting's place on the Firmware page of Preferences and of Big Picture's settings, not among the
Library tab's folders; and, as a standing rule for every folder setting, a folder that does not exist is accepted and
said, never refused, so that a drive mounted later is found. The other folder settings take that rule when the queued
work on the folder-setting rules (Config Reference §7) is done.

## 3. What each frontend does

The prompt is deliberately *not* in the core. A core deep in a constructor cannot open a dialog, and the frontends do not agree on what "ask the player" means.

**`EmuSen.Mistress`** (Avalonia, the ROM-opening GUI) — `PromptForMissingFirmwareAsync` runs before `LoadRom` on both open paths (the OS picker and the ROM browser). For each missing image it opens a file picker whose title names the chip, the expected filename and the expected size, installs whatever comes back, and reports the outcome in the status bar. **Cancelling is a supported answer**: the status bar says the chip will not be emulated, and the ROM loads anyway.

Because `Install` copies into the library, this is a once-ever prompt per chip, not once per launch.

*Since 2026-10-03 only a request a core marks `required` is prompted for, and no core the project builds marks one
(§0).* What a player sees instead is **Preferences ▸ Firmware** (2026-10-05, `EmuSen_Settings_Reference.md` §4.89): per
system, each file an engine can use, whether EmuSen's open version or the player's own file is running, what the open
version changes, and the folder a file goes in. It lists and explains; it installs, fetches and asks for nothing.

*Since 2026-10-12* the question itself reads no image where it cannot find anything to prompt for: for a v1 engine,
`EmulatorSession.MissingFirmwareFor` asks `CoreEngine.RequiredFirmwareFor`, which answers empty without the image when
every system of the core's info lists its firmware and none of it is required, since those lists name every file
`firmware_for` can (`EmuSen_CoreAPI.md` §6.3); any other info reads the image as before. The load still asks
`firmware_for` of the image and passes what it names. A PlayStation disc of 600 MB had been read whole, and a zip
unpacked, before every launch only to learn of an optional BIOS (`Diamond_Players.md` §12.8).

**`EmuSen.Hotaru`** (Avalonia, but shell-driven) — no OS picker; ROMs arrive from the DianaSh shell. It gets the log path below.

**`EmuSen.Pharaoh`** (headless CLI, including its no-ROM `--singlestep`/`--diffshot`/`--tracediff` verbs) — must never block on a prompt. It gets the log path too.

**The log path** — when a core can't find firmware it prints what is missing *and the exact path it wants it at*:

```
[Cartridge] Dsp1B firmware not found - the chip will not be emulated.
[Cartridge]   expected 8,192 bytes at .../home/Firmware/dsp1b.rom
```

One line of core code serves every non-picker frontend, which is why this is worth having even though Mistress has a real dialog.

### 3.1 Which firmware a session ran on (2026-10-16)

Since a core always has an open replacement (§0), a running game says nothing about which of the two it is on, and a
report from a player cannot be read without knowing. The tester's first day with the Sega CD showed the gap: films
that would not load were first suspected of running on Nephrite's own BIOS, whose Sonic CD film is black (D-54),
and nothing in the session could settle it. `CoreEngine.LoadRom` now writes one line per firmware entry that machine
info reports, after the machine is made:

```
[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin, in /home/player/Documents/BIOS
[Firmware] The American Sega CD's BIOS (131,072 bytes): Nephrite's own replacement
```

The source is machine info's (`file`, `replacement`, `absent`), the core's own account of what it booted, not the
host's account of what it passed: a file the host handed over that the core did not use would otherwise be reported
as running. The line is generic over cores, the label and the file name being the core's firmware entry. In Mistress
it lands in the game's `general.log` under the log directory (`EmuSen_Settings_Reference.md` §4.22), since logging
starts before the load. Tested by `NephriteFirmwareTests`.

*Since 2026-10-17* a line for the player's file ends with the folder it was read from (§2.3), the chosen folder or
`home/Firmware`, since with two folders searched the file name alone no longer says which copy ran. The folder is the
host's record of where `Find` took the file it passed; the source word is still the core's.

---

## 4. Adding a core

1. Override `GetFirmwareRequirements(romPath)` and return a `FirmwareRequest` per image, answering from the file header only.
2. Where the core loads the image, call `FirmwareLibrary.TryLoad(request)`.
3. When it comes back null, log and continue **without** the chip. Do not throw.

That is the entire integration. The picker, the install, the validation and the missing-file report all already work, because none of them know what console they are serving.

A chip with a non-standard on-disk layout can keep that locally — the NEC DSP also accepts a split `dsp1.program.rom` + `dsp1.data.rom` pair, which is a NEC-DSP convention and lives in `NecDspFirmware`, while the combined `dsp1.rom` form goes through the shared library. Put the general case in the library and the quirk next to the chip.

---

## 5. Why none of this ships

The mask ROM inside a DSP-1 is Nintendo's. A BIOS pack's own MIT or GPL license covers that pack's *tooling*, not the dumps it redistributes — you can only license what you own — so no LICENSE file anywhere makes those bytes redistributable.

EmuSen is GPL-3.0, which obliges us to grant downstream recipients rights we would not hold. Bundling firmware would therefore be both a copyright problem and a licensing contradiction, which is why bsnes and Mesen refuse to do it either.

`home/Firmware/` is gitignored for exactly this reason. Dumps a developer puts there are theirs, stay local, and never enter version control.

---

## 6. Where to look when something is wrong

1. **The picker never appears.** `GetFirmwareRequirements` returned nothing. Either the ROM carries its firmware appended (which is correct — nothing is needed), or header detection didn't recognise the chip. Check `Cartridge.FirmwareRequirements` before suspecting the dialog.
2. **The player picks the right file and it still isn't found.** Size mismatch — §2.1. Check the expected `Size` against the file; the MAME and bsnes DSP layouts differ in length, see `Venus_NecDSP.md` §2.3.
3. **A test passes locally and fails on another machine, or vice versa.** It is reading the real firmware directory, or a `FirmwareDirectory` set in the real `appsettings.json`, instead of overriding `FirmwareLibrary.Directory` (which also stops the setting being read). Two tests were written this way once and only passed because the machine happened to have no dumps installed; they broke the moment one did. Any test whose outcome depends on what's installed must set `Directory` to a temp folder and `ResetDirectory()` after.
4. **A player's file is in their folder and the open version still runs.** Read the session's `[Firmware]` line (§3.1), then Preferences ▸ Firmware: a message under the folder row says when the setting is refused or the folder is absent, and a file's row names the folder it was found in. A file of the wrong size in the chosen folder is passed over for one in `home/Firmware` (§2.3).
5. **A firmware-gated test silently passes without testing anything.** That is by design — `NecDspRealFirmwareTests` returns early when no dump is present. It is the only way to keep real-firmware verification in a repo that cannot contain firmware.

---

## 7. A file recognised by its content (2026-10-18)

> **Status: built, but for Diamond's checksums.** Proposed and decided on 2026-10-18 (§7.9). The store, both its
> halves, the core ABI's `md5` (1.12, `EmuSen_CoreAPI.md` §6.32), kit case C23, the host and the session's line are in
> (§7.10). Diamond lists no checksum yet: the public description its checksums are to be cited from was not read in
> this step (§7.8).

### 7.1 The report and the decision

The tester's PlayStation images are named as the collection they came from names them (`PSX - SCPH5502.BIN`), while
Diamond's info declares `scph5500.bin`, `scph5501.bin` and `scph5502.bin`. The store takes a file only by a declared
name and the declared size (§2.2, §2.3), so Mistress runs the open kernel on a folder that holds the player's BIOS, and
says nothing; and the opt-in run of ps1-tests on the player's BIOS fails, naming the folder and the names it looked
for (`VenusRT_Native.md` §70.5, `Diamond_Native.md` §11). A read-only listing of the tester's folder on 2026-10-18
found 15 files of 524,288 bytes, the size Diamond declares, none of them under a declared name; the store found none.

*Decided 2026-10-18 by the tester:* the store also recognises a file by a checksum of its content, whatever it is
called; each core lists the checksums it accepts in its own info, beside the names it declares; nothing is shipped or
fetched; and the tests find firmware through the same store as Mistress, so that the two agree and the lookup has one
owner.

### 7.2 What the surfaces carried before

- **The core ABI's firmware entry** (`EmuSen_CoreAPI.md` §6.2, `emusen-native/abi/v1/firmware.schema.json`) has
  `which`, `name`, `label`, `size`, `required`, `parts` and `replacement`, and no checksum. The same entries appear in
  core info's per-system `firmware` list (§6.3). A checksum is therefore a new optional member: an **additive minor,
  1.12**, with a kit case.
- **`FirmwareRequest`** has no checksum either; §2.1 noted the room for one. Neither does `emusen-firmware`'s
  `Request`, nor the JSON its C layer takes (`emusen_firmware_find`, `_locate`, `_install`, platform interface 17).
- **No core's info carries a checksum** of real firmware. The only checksums of firmware in the project's text are the
  SHA-256 sums of Diamond's own open kernel images (§0).

### 7.3 Which checksum: MD5, as Galaxia's `RomHash`

The checksum is MD5, computed as `RomHash` computes a ROM's (`EmuSen_Galaxia.md` §5.4), for three reasons.

1. **It is the same job.** `RomHash` is the project's identity of a player's file across a rename, which is exactly
   what recognising a renamed firmware file is. Its Rust and C# halves exist already and are held to one answer
   (`emusen-galaxia`'s `rom_hash`, `RomHash.Managed`), so the store adds no second owner of a hash.
2. **It is what the public descriptions of firmware state.** The core info files of libretro give each firmware file's
   MD5 in their notes (read on 2026-10-18 in the reference probe's copy of `mednafen_psx_libretro.info`: the three
   PlayStation names Diamond declares, each with an MD5). A core author can then fill the field from a cited public
   description rather than by hashing dumps, which matters under §0, where the originals are kept apart and read by
   one tool only.
3. **Collision resistance buys nothing here.** A file under a declared name and of the declared size is taken today
   with no check of its content at all, and stays so (§7.4). A file crafted to share an MD5 with a real image would
   gain nothing that naming it would not.

*Alternatives weighed.* SHA-256 is the sidecar's hash (`EmuSen_CoreAPI.md` §7) and is resistant to collisions; on the
machine measured (§7.6) it is faster where the processor has SHA extensions and slower where it has none, so speed
does not decide it, and reason 3 removes its advantage. CRC-32 is not yet in `RomHash` (it is planned,
`EmuSen_GamePackages.md` §3.3) and 32 bits is a weak identity among the hundreds of images of one size that a
collection can hold. The member is named for its algorithm, `md5`, so a later minor can add another without
ambiguity.

### 7.4 The rule

For each folder of §2.3, in order, the first of these that finds the image wins:

1. the canonical name, then the alternates, each at the declared size (unchanged, §2.2);
2. each split form by its names, whole from the folder (unchanged, §2.3);
3. **by content**, when the request lists checksums: the folder's own files whose length is the declared size (a link
   measured by its final target, as §2.1 has it since 2026-10-17), and among them the first whose MD5 is listed,
   taking the checksums in the core's order and, for files of the same content, the file names in ordinal order.

Then the next folder. The scan reads only the folder's own entries, never a subfolder, as the name search does.

- **A file of the declared size whose content is not listed is not taken** by step 3: it is not the image, and the
  search goes on to the next folder.
- **A name match is kept ahead of a content match within a folder,** including a named file whose content is not
  listed. Players are told by other emulators' documentation to rename a dump to the name a core expects, and a dump
  of a version a core does not list, so renamed, runs today; ranking content first would make it lose, without a word,
  to another file in the same folder. Step 3 adds finds and removes none.
- **A content match in the chosen folder is ahead of a name match in `home/Firmware`.** The argument of §2.3 applies
  unchanged: the chosen folder is the explicit choice, and a content match is the stronger evidence. The alternative,
  every folder searched by name before any by content, would let a file left in `home/Firmware` by an older build
  override a verified image in the folder the player chose. *Decided 2026-10-18 by the tester: the player's folder is
  searched fully, by name and then by content, before `home/Firmware`.*
- **Each request is matched alone.** Diamond's three entries, one per market, each list their own checksums, and a
  file is taken for a market only by that market's list. An image listed in two entries may be taken for both.
- **Never written.** The store neither copies, renames nor touches a file it recognised; `Install` is unchanged and
  still writes only `home/Firmware`, under the canonical name.
- **Split forms by content** are left out of 1.12 (decided 2026-10-18, §7.9). A part has no declared size, so recognising parts would
  either hash every file of the folder or need a further member of part sizes, and no core has asked: the one split
  form in use, the NEC DSP's program and data files, is a naming convention of one layout whose holders hold it under
  those names.

### 7.5 What changed, piece by piece

| Piece | Change |
|---|---|
| Core ABI 1.12 (`EmuSen_CoreAPI.md` §6.32) | a firmware entry, in `firmware_for`'s answer and in core info's lists, may carry `md5`, an array of 32 lower-case hexadecimal digits, the images of `size` bytes the core accepts in that slot. Absent or empty is today's behaviour, so no existing info changes by a byte; an older host ignores it; an older core lists none. No export, struct, status or capability bit. `emusen_core.h`'s minor becomes 12 |
| `emusen-native` | `Firmware` gains `md5: Vec<String>`, written only when not empty; the struct's literal sites (18, in VenusRT, Nephrite, Diamond, the test core and `emusen-native`'s tests) gain the field as empty, with no other change |
| The kit | **C23** (`EmuSen_CoreAPI.md` §28.18): every `md5` of `firmware_for`'s answer and of core info's lists is 32 lower-case hexadecimal digits, none repeated within an entry, and an entry that appears in both, by `which` and `name`, lists the same checksums. A seeded fault in the test core, the checksum a digit short, fails C23 alone (an upper-case digit, first tried, fails C2's schema check too). The kit carries no firmware and checks no content |
| Host (`CoreDescriptors`, `CoreEngine.Request`) | `CoreFirmware.Md5` read from the entry and passed into the request |
| `FirmwareRequest` | `Md5`, a list of checksums, empty by default |
| `FirmwareFound`, `FirmwareLocated` | `ByContent`, true when step 3 found it; `Names` then holds the player's own file name |
| `emusen-firmware` and the C# twin | step 3 in `search` (`by_content`) and in `Managed.Find` (`ByContent`); MD5 by Galaxia's `rom_hash::md5` and, in the C#, `MD5.HashData` over the file as `RomHash.Managed` reads it; the cache of §7.6 in each |
| Platform interface 18 (`EmuSen_RustPlatform.md`) | the request's JSON gains `md5`, and `_find`'s and `_locate`'s answers `content`. The number moves 17 to 18, so an older library is refused and the C# answers, rather than a library that ignores `md5` disagreeing with its twin |
| `FirmwareOverview`, Preferences ▸ Firmware | `FirmwareItem.FoundAs`, the player's file name when the file was recognised by its content. Preferences ▸ Firmware reads the overview through the frontend API (`emusen_crystal_firmware_overview`), whose items carry no such key; adding `found_as` is a frontend API minor, and the API's step (B2) was changing it in the same days, so the key and the page's words ("Your own file, PSX - SCPH5502.BIN, recognised by its content") wait for that step. Until then the row says the player's file is in use and in which folder, as for a file found by name |
| The session's line (§3.1) | "the player's PSX - SCPH5502.BIN, recognised by its content, in …" |
| Diamond's opt-in test (`DiamondTests.Ps1_tests_pass_by_their_tty_on_a_players_bios`) | each declared entry found by `FirmwareLibrary.Find(CoreEngine.Request(core, f), [folder])`, not by its own `File.Exists` on the name; one owner of the lookup |
| Diamond's info | its three entries carry `md5: Vec::new()` in `bios()` of `src/v1.rs`, the field's only change there; the checksums wait for §7.8 |

Diamond's crate tests on the player's BIOS (`tests/programs.rs`, the disc programs) read the American image by its
declared name from `EMUSEN_DIAMOND_FIRMWARE`. They are a core's own tests and cannot call the store, which is the
platform's (a core does not depend on the platform library); they keep the name, or take a file's path in a variable
of their own. The decision's "the tests use the same store" is met by the WiseMan tests, which run on the store.

### 7.6 What it costs (measured 2026-10-18)

On the development machine (Ryzen 7 7700X), in a scratch folder of 200 files of random bytes: 185 of other sizes from
2 KiB to 4 MiB and 15 of 524,288 bytes, the tester's folder's count. The page cache was warm; a cold read was not
measured.

| Step | Time |
|---|---:|
| list the folder and measure every file | 0.28 ms |
| MD5 of the 15 files of the declared size, 7.9 MB | 9.1 ms (864 MB/s) |
| SHA-256 of the same, with the processor's SHA extensions | 3.5 ms |
| SHA-256 of the same, the software path | 15.0 ms |
| MD5 of a folder of 200 files all of the declared size | 124 ms |

Hashing only files of a declared size is what keeps the scan cheap: the listing costs a quarter of a millisecond, and
the time is in the files of the right size. The overview asks once per entry, and Diamond's three entries would hash
the same 15 files three times, 27 ms for one build of the page. Each half therefore keeps a cache, from a
file's path, length and modification time to its MD5, for the life of the process, holding only files of a declared
size: the second and third entries then cost the listing alone, and a file changed or replaced under the same name is
hashed again. The cache is in the store and in no caller. A launch asks for one entry of Diamond's and so costs one
scan, 9 ms here on first use.

### 7.7 The gates

- **Parity**: `FirmwareParityTests` extended with checksum cases on synthetic files of random bytes, whose MD5 the test
  computes and declares in its own request: a renamed file found by content; a file of the right size and other
  content refused; a name match preferred over a content match in the same folder; a content match in the first folder
  ahead of a name match in the second, the order decided (§7.9); a missing folder; a file given as a link, found by
  its target's content and size; a dangling link passed over; the generated trees of §15.1 with checksums added to
  their requests. No test carries a real firmware checksum.
- **Seeded faults**, each made, run and reverted: step 3 before step 1; the size filter dropped, so that a listed file
  of another size is taken; the checksums compared in upper case; a subfolder scanned; the cache keyed without the
  modification time; files ahead of checksums in the pick; the facade dropping `content` (§7.10).
- **The blast radius** with `EMUSEN_SERVICES_NATIVE` unset and `0`, as §15.1 lists it, with the kit's C23 and the cores
  whose firmware literals change.
- **Read-only, last**: the store run on the tester's folder of PlayStation images, reporting which declared entry each
  file matched, with nothing copied; then the opt-in ps1-tests run on that folder.

### 7.8 What Diamond lists, and from where

*Decided 2026-10-18 by the tester:* each of Diamond's three entries lists **every retail BIOS version of its market**,
with MD5s cited from a public BIOS database, named here the way libretro's core info files name theirs. They are
never computed from the player's dumps, and the images in Diamond's probe cache are read by `firmwarecheck.py` alone
(§0). The ps1-tests programs name no market and run as an American console (the Region setting's text in Diamond's
`src/v1.rs`), so the American entry's list is what lets the opt-in run use a player's American dump of any version.

**Not yet done.** Reading the public database's list was not possible in this step: the tools that would have read it,
from the network or from a copy of libretro's descriptions already on the development machine, were not permitted, and
no checksum was taken from memory, since a list that cannot be cited is not one. `bios()` therefore lists none, the
store recognises no PlayStation file by content yet, and the read-only check of §7.10 found none in the tester's
folder. What remains is one change to `bios()` in `src/v1.rs`, each market's list with its citation, and the check
and the opt-in run of §7.10 repeated.

### 7.9 The decisions (2026-10-18)

The tester answered §7's five questions on 2026-10-18:

1. **Policy.** A core's info may list the checksums of the real firmware it accepts, cited from a public source; the
   images are still never shipped, fetched or included as data. Recorded as a standing rule in §0, item 4.
2. **Order across folders.** The player's folder is searched fully, by name and then by content, before
   `home/Firmware` (§7.4).
3. **Diamond's scope.** Every retail BIOS version of each market, cited from a public BIOS database (§7.8).
4. **Split forms by content.** Left out of 1.12.
5. **The ABI and the hash.** Core ABI 1.12 with an optional `md5` array and kit case C23, MD5 computed as `RomHash`
   computes it; the platform interface from 17 to 18.

### 7.10 What was built, and the gates (measured 2026-10-18)

Built on branch `firmware-by-hash` from `WiseMan` at `1ff6af65`, as §7.5 lists it. Nothing in this step writes to a
player's folder; the store reads a file's bytes to hash it and never opens one to write.

| Gate | Result |
|---|---|
| `cargo test --release -p emusen-firmware -p emusen-galaxia -p emusen-platform` | 11, 72, 1, 31 and 4 pass; the store's six new cases: a renamed file found and another content of the size refused, a name before content and the chosen folder wholly before the store, the checksums' order then the names', a listed file of another size, a subfolder and a missing folder, a link by its target and a dangling one, a file rewritten in place hashed again |
| `emusen-native`, `emusen-core-abi-check` | 78 and 6 pass; `--emit-baseline` wrote 485 facts, the two `md5` lines tagged 1.12; `--check` and `--against` 1.11 agree |
| `emusen-core-conform` | 24 pass: the test cores pass C1-C23, and `FAULT_MD5_SHORT` fails C23 alone, in the single-image and the corpus forms |
| Diamond, Nephrite, VenusRT | `cargo check --all-features --all-targets` clean; their `v1` tests 18, 21 and 18 pass |
| `FirmwareParityTests` | 7 pass, among them the checksum cases of §7.7 compared between the halves and against the rule, a file rewritten in place, and the 150 generated trees with checksums of their own files of every size among made-up ones (more than 20 requests found by content) |
| The blast radius, `EMUSEN_SERVICES_NATIVE` unset and `0` | 411 pass of 411 each way: the firmware classes, the parity classes of B5 and the interface's number, the core ABI's classes (`CoreAbiTests`, `CoreAdapterTests`, `CoreDiscoveryTests`, `MarsRtCoreAbiTests`, `VenusRtCoreAbiTests`, `DiamondTests`, `NephriteTests`), `VenusGoldenTests`, `GoldenCorpusTests` (the `firmware` cases unchanged) and the firmware page's window audit |

**Seeded faults**, each made, built, run and reverted:

| Fault | Caught by |
|---|---|
| step 3 before the names (Rust) | the crate's name-before-content case; the parity class (the checksum cases and the generated trees) |
| the size filter dropped (C#) | the parity class (a listed file of another size; the generated trees) |
| the checksums compared in upper case (Rust) | five of the crate's cases; the parity class; `FirmwareLibraryTests` through the public calls |
| a subfolder scanned (C#) | the parity class's checksum cases. Its first form put the subfolder in a folder the case did not search, and the fault passed; the case was corrected and the fault then failed it |
| the cache keyed without the modification time (C#) | the parity class's rewritten file |
| files ahead of checksums in the pick (Rust) | the crate's order case; the parity class's order case, added after the generated trees let this fault pass |
| the facade dropping `content` (C#) | the parity class; `FirmwareLibraryTests` |

**The tester's folder, read only.** The store, pointed at the tester's folder of PlayStation images and given Diamond's
three entries as its info declares them, found none: no file bears a declared name, and with no checksums listed
(§7.8) step 3 has nothing to match. The folder's entries, lengths, modification times and modes, and the folder's own
times, were the same before and after. `DiamondTests.Ps1_tests_pass_by_their_tty_on_a_players_bios`, run on that
folder with the release in the probe folder, failed as the rule of `VenusRT_Native.md` §70.5 requires, naming the
American and European BIOS it lacks; it now asks the store, so the same run passes once Diamond's lists are in and
the folder holds an image of each market the programs run on, or `EMUSEN_DIAMOND_REGION` names one it holds.

