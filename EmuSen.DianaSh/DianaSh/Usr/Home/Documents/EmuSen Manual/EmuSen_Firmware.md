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
   replacement, as the exact path. It is offered, never required, and no frontend prompts for it.
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
MIPS code shares more with the same images (9,931 runs of 9+ bytes, the longest 32) than the kernel does. *Decided 2026-10-11 (Q5)*: a replacement that is compiled code is judged against a **control**, unrelated code of the same processor and size, with fill left out (`--ignore-fill 8 --min-run 9 --control CONTROL --explained FILE`): it passes when at most 25% of its bytes are equal at the same offset, it shares no more runs of 9 bytes or more with the original than the control does, and every run longer than the control's longest lies in a listed pattern of its own words with the reason the match is forced or generic. For Diamond the control is the first 92,440 bytes of ps1-tests' release programs' bodies, built at run time by `EmuSen.WiseMan/Reference/analysis/firmware/diamond-control.py`, and the patterns are `firmware/diamond-explained.txt` (`setjmp` and `longjmp` storing and loading the registers in the documented buffer order, and the compiler's register save and restore at every frame offset). Measured the same day on the kernel image of SHA-256 7af83c9b… against all 18 images, each compared whole: **PASS on all**, 13.5–15.1% equal at the same offset against the control's 11.0–11.3%, 6,394–16,514 shared runs of 9+ bytes, on every image fewer than the control's 9,916–24,257 with the same image, and 15–26 runs past the control's longest of 32–33 bytes, all explained. The check is rerun whenever the kernel's image changes. *Rerun 2026-10-12* on the image of SHA-256 4e01c2ac… (the font, the asynchronous card calls and the exception path in RAM), with a control of 100,544 bytes to match its length: **PASS on all 18**, 13.2–15.6% equal at the same offset against the control's 10.6–11.9%, 6,764–17,157 shared runs of 9+ bytes, on every image fewer than the control's 10,143–24,766 with the same image, and 20–29 runs past the control's longest of 32–33 bytes, all explained. *Rerun 2026-10-12* on the image of SHA-256 c1a002e8… (Mega Man Legends: fill with sparse returns behind B(5Bh)'s entry, LoadExec and the synchronous CD operations), with a control of 103,256 bytes: **PASS on all 18**, 13.4–15.1% equal at the same offset against the control's 10.6–11.9%, 6,859–17,479 shared runs of 9+ bytes, on every image fewer than the control's 10,915–26,613 with the same image, and 20–29 runs past the control's longest of 32–33 bytes, all explained. An image built before it, with 4 KiB of `jr ra` and `nop` pairs in that place, failed on all 18 by its shared runs, and was not merged (`Diamond_OpenKernel_Phase2.md` §26.8).

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

`FirmwareLibrary` is `home/Firmware/` treated as a store, and it knows nothing about any console.

| Call | Meaning |
|---|---|
| `TryLoad(request)` | the bytes, or null |
| `IsInstalled(request)` | as above, as a bool |
| `MissingFrom(requests)` | filters a batch down to what's absent |
| `Install(request, sourcePath)` | copies a file the player picked in, under the canonical name |
| `PathFor(request)` | where it belongs, for an error message |

`Directory` is settable, and `ResetDirectory()` puts it back. That exists so tests can point at a temp folder — without it every test's result would depend on which dumps happen to be on the machine, which is a bug this project has already been bitten by once (see §6).

### 2.1 Size is the only validation

An image whose length doesn't match `Size` is treated as **absent**, not loaded. A truncated or mismatched dump executes as garbage and produces a bewildering failure a long way from its cause; "not found" is a far better diagnostic. `Install` applies the same rule, and refuses to copy anything that fails it.

Checksums would be stricter still, and `FirmwareRequest` has room for them later. Size alone already rejects every accident seen in practice — the wrong chip's dump, a half-downloaded file, a ROM picked by mistake.

### 2.2 Canonical names and alternates

`AlternateNames` lists other filenames the same dump circulates under. They are checked after the canonical name and **never written to**: an install always lands under `FileName`, so the next launch finds it first and the store converges on one layout no matter what the player picked.

---

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
[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin
[Firmware] The American Sega CD's BIOS (131,072 bytes): Nephrite's own replacement
```

The source is machine info's (`file`, `replacement`, `absent`), the core's own account of what it booted, not the
host's account of what it passed: a file the host handed over that the core did not use would otherwise be reported
as running. The line is generic over cores, the label and the file name being the core's firmware entry. In Mistress
it lands in the game's `general.log` under the log directory (`EmuSen_Settings_Reference.md` §4.22), since logging
starts before the load. Tested by `NephriteFirmwareTests`.

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
3. **A test passes locally and fails on another machine, or vice versa.** It is reading the real firmware directory instead of overriding `FirmwareLibrary.Directory`. Two tests were written this way once and only passed because the machine happened to have no dumps installed; they broke the moment one did. Any test whose outcome depends on what's installed must set `Directory` to a temp folder and `ResetDirectory()` after.
4. **A firmware-gated test silently passes without testing anything.** That is by design — `NecDspRealFirmwareTests` returns early when no dump is present. It is the only way to keep real-firmware verification in a repo that cannot contain firmware.
