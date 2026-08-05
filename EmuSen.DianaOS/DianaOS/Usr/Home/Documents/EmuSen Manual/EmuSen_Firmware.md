# EmuSen — Firmware

Covers `EmuSen/Common/Firmware/`, `ICore.GetFirmwareRequirements`, and the picker in `EmuSen.Mistress`. Core-agnostic by design: the SNES's NEC DSP is the first user, but a PlayStation BIOS, a Game Boy boot ROM, or a Saturn IPL differ from it only in the values they fill in.

> **Status: complete.** Discovery, validation, install, and the Avalonia picker all work and are tested. The SNES core is wired up; every other core inherits the empty default and asks for nothing.

---

## 1. The contract

Some hardware needs code the software never shipped with. A DSP-1's program is inside the chip, not on the cartridge; a PlayStation cannot boot without its BIOS. Emulating that hardware means having a dump, and **EmuSen does not ship one** — see §5.

Three pieces make that tractable:

**`FirmwareRequest`** — one image, described in the abstract: which core wants it, which chip it is, the canonical filename, the exact byte count, and a one-line human explanation of what it's for. Nothing console-specific.

**`ICore.GetFirmwareRequirements(romPath)`** — what a given ROM will need, answered **without loading it**. This is a default interface method returning nothing, so a core whose hardware needs no firmware inherits the right behaviour for free. `VenusCore` overrides it and delegates to `Cartridge.FirmwareRequirements`, which reads the header and stops.

Answering before loading is the whole point. Once `LoadRom` has run, the core has already come up with the chip absent, and the only fix is to load again — so a frontend that wants to *offer* the user a way out has to ask first.

**A missing image is never fatal.** `LoadRom` still succeeds; the core runs with that chip missing. That is what makes the prompt optional rather than a gate, and it is why `Pharaoh` and `Tomoe` need no firmware code at all.

---

## 2. The library

`FirmwareLibrary` is `home/Firmware/` treated as a store, and it knows nothing about any console.

| Call | Meaning |
|---|---|
| `TryLoad(request)` | the bytes, or null |
| `IsInstalled(request)` | as above, as a bool |
| `MissingFrom(requests)` | filters a batch down to what's absent |
| `Install(request, sourcePath)` | copies a file the user picked in, under the canonical name |
| `PathFor(request)` | where it belongs, for an error message |

`Directory` is settable, and `ResetDirectory()` puts it back. That exists so tests can point at a temp folder — without it every test's result would depend on which dumps happen to be on the machine, which is a bug this project has already been bitten by once (see §6).

### 2.1 Size is the only validation

An image whose length doesn't match `Size` is treated as **absent**, not loaded. A truncated or mismatched dump executes as garbage and produces a bewildering failure a long way from its cause; "not found" is a far better diagnostic. `Install` applies the same rule, and refuses to copy anything that fails it.

Checksums would be stricter still, and `FirmwareRequest` has room for them later. Size alone already rejects every accident seen in practice — the wrong chip's dump, a half-downloaded file, a ROM picked by mistake.

### 2.2 Canonical names and alternates

`AlternateNames` lists other filenames the same dump circulates under. They are checked after the canonical name and **never written to**: an install always lands under `FileName`, so the next launch finds it first and the store converges on one layout no matter what the user picked.

---

## 3. What each frontend does

The prompt is deliberately *not* in the core. A core deep in a constructor cannot open a dialog, and the frontends do not agree on what "ask the user" means.

**`EmuSen.Mistress`** (Avalonia, the ROM-opening GUI) — `PromptForMissingFirmwareAsync` runs before `LoadRom` on both open paths (the OS picker and the ROM browser). For each missing image it opens a file picker whose title names the chip, the expected filename and the expected size, installs whatever comes back, and reports the outcome in the status bar. **Cancelling is a supported answer**: the status bar says the chip will not be emulated, and the ROM loads anyway.

Because `Install` copies into the library, this is a once-ever prompt per chip, not once per launch.

**`EmuSen.Hotaru`** (Avalonia, but shell-driven) — no OS picker; ROMs arrive from the DianaOS shell. It gets the log path below.

**`EmuSen.Pharaoh` / `EmuSen.Tomoe`** (headless CLI) — must never block on a prompt. They get the log path too.

**The log path** — when a core can't find firmware it prints what is missing *and the exact path it wants it at*:

```
[Cartridge] Dsp1B firmware not found - the chip will not be emulated.
[Cartridge]   expected 8,192 bytes at .../home/Firmware/dsp1b.rom
```

One line of core code serves every non-picker frontend, which is why this is worth having even though Mistress has a real dialog.

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
2. **The user picks the right file and it still isn't found.** Size mismatch — §2.1. Check the expected `Size` against the file; the MAME and bsnes DSP layouts differ in length, see `Venus_NecDSP.md` §2.3.
3. **A test passes locally and fails on another machine, or vice versa.** It is reading the real firmware directory instead of overriding `FirmwareLibrary.Directory`. Two tests were written this way once and only passed because the machine happened to have no dumps installed; they broke the moment one did. Any test whose outcome depends on what's installed must set `Directory` to a temp folder and `ResetDirectory()` after.
4. **A firmware-gated test silently passes without testing anything.** That is by design — `NecDspRealFirmwareTests` returns early when no dump is present. It is the only way to keep real-firmware verification in a repo that cannot contain firmware.
