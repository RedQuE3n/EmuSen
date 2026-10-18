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

Checksums would be stricter still, and `FirmwareRequest` has room for them later. Size alone already rejects every accident seen in practice — the wrong chip's dump, a half-downloaded file, a ROM picked by mistake.

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

Within a folder the order of §2.2 holds (the canonical name, then the alternates, then each split form), and a
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
