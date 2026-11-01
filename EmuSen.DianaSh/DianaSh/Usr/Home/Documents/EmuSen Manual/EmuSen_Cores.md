# EmuSen_Cores — Cores, the window and command that download and manage EmuSen's cores

*Design, 2026-10-31; its questions answered by the tester the same day (§11). C0–C2 built the same day (§3.3, §6.1, §8).* Asked by the tester the same day: "We should also make a core download and
management window. Base it on our designs for the interface overhaul." The condition set for the libretro work holds
here too: done properly, with no shortcuts.

**Its name is Cores**, the window `CoresWindow`, the command `cores`. It manages every core EmuSen can run: the
project's own (Sapphire, Diamond, MarsRT and the rest, shipped with the build), libretro cores hosted through
`emusen-libretro-host` (`EmuSen_LibretroHost.md`), which are downloaded only when the player asks, and any other core
the player put in `/Cores`.

The marks are those of `EmuSen_Gpu.md`: **[R]** read from a source, **[A]** argued, **[C]** a choice.

**Sources** [R, read 2026-10-31]: `EmuSen_Packages.md` §1–§6; `EmuSen_LibraryTools.md` §1–§6; `EmuSen_Mistress_LibraryPlan.md`
§0–§3; `EmuSen_InterfaceSpec.md` §1, §5; `EmuSen_LibretroHost.md` (revised 2026-10-31) §0.1, §1, §2, §4.1, §4.7, §4.8,
§7, §10; `EmuSen_CoreAPI.md` §7; `EmuSen_Backend.md` §29.40.4–§29.40.5; `THIRD_PARTY_NOTICES.md` §2; the buildbot's
nightly listing <https://buildbot.libretro.com/nightly/linux/x86_64/latest/> and its `.index-extended`; its stable listing
<https://buildbot.libretro.com/stable/1.22.2/linux/x86_64/>; <https://buildbot.libretro.com/assets/frontend/>; libretro's
example core information file, `00_example_libretro.info` in libretro-core-info `master`
(<https://github.com/libretro/libretro-core-info>). No emulator's source is read for this work.

## 1. The form

Weighed under the project's rule for features (an extension, a module, or a component):

- **A tool of its own** [C], as Packages, Scraper and Library are (`EmuSen_LibraryTools.md` §1): managing cores is not
  a game package's job, nor a library file's, nor the runtime's. It is one more member of the family (§1c there).
- **Three parts, each with one job**, as the library tool's are:

| Part | Its one job | Form |
|---|---|---|
| `emusen-core-tool` | the catalogue, the download, the checks, the install, update and removal of a core folder | a Rust crate in `Platform/core-tool/`, run as jobs (§6) |
| `emusen.cores` | the shell's face: the `cores` command | a DianaSh command module (`Platform/emusen-cores-module/`), with `man cores` |
| `CoresWindow` | the window's face over the same jobs | a tool library (`EmuSen.Tools.Cores`, §3.4) on LunaP's `ManagerLayout` |

- **What it does not own.** Trust stays the runtime's (`EmuSen_CoreAPI.md` §7.2: the manifest, the approvals, the
  checks before a load); the console's and a game's engine stay the runtime's `Engine` setting (§4.3); a core's options
  stay the settings schema's (§4.4); firmware stays the firmware settings' (§4.5). Cores calls each of those through its
  surface and keeps no second copy of any of them.

## 2. Layout

The layout is Packages' (`EmuSen_Packages.md` §3): filters at the side, a table with a mark per row, a detail pane
beneath, changes marked and then applied together, after Synaptic and Pamac. Storefront cards (GNOME Software, KDE
Discover) were weighed again for cores, which number about two hundred, not thousands, and were again set aside:
the player chooses among several cores *for one console* and compares their licences, sizes and states, which a table
does and a card wall does not.

### 2.1 Desktop

```
┌ Toolbar: [Install] [Update] [Remove] [Make Default] [Options…] [Check for Updates]          Search… ┐
├──────────────┬───────────────────────────────────────────────────────────────────────────────────────┤
│ Consoles     │ ☐ Core              Console        Source     Version      State          Default  Size │
│  All         │ ☐ Sapphire          PlayStation 2  EmuSen     0.44         built in       ●        9 MB │
│  PlayStation │ ☑ LRPS2             PlayStation 2  libretro   2026-10-31   available             41 MB │
│  …           │ ☐ Diamond           PlayStation    EmuSen     1.12         built in       ●        6 MB │
│ Show         │ ☑ Beetle PSX        PlayStation    libretro   2026-10-29   update ↑       ○        8 MB │
│  Installed   ├───────────────────────────────────────────────────────────────────────────────────────┤
│  Available   │ LRPS2 — PCSX2 Team · GPLv3 [Licence…] · buildbot.libretro.com · nightly 2026-10-31     │
│  Updates     │ Firmware: PlayStation 2 BIOS (pcsx2/bios), required — yours: found                    │
│  Approval    │ Notes: needs Vulkan · x86-64 only · disc images by path      [Options…] [Open Folder] │
│  Unsupported │ SHA-256 9f2c…e01a (after download) · not yet approved                                 │
├──────────────┴───────────────────────────────────────────────────────────────────────────────────────┤
│ 2 marked: install LRPS2 (41 MB), update Beetle PSX (8 MB)         [Apply…]   /Cores 312 MB · 410 GB free │
└──────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Columns**: core (its display name), console, source (*EmuSen*, *libretro*, *added by you*), version (a project
  core's `info()` version; a libretro core's `display_version` and build date), state (§4.2), default (● the console's,
  ○ chosen by one or more games), size (installed, or the download's where known). Sortable, as every
  `ManagerLayout` table is.
- **Filters**: console; show (installed, available, updates, waiting for approval, not supported); search by name.
- **Marks**: install, update, remove, approve. A project core is offered none of them: it ships with the build and is
  updated with EmuSen. A core the player added by hand is offered approve and *Revoke Approval*, never remove (§5.6).
- **Apply…** shows the plan before anything runs ("download 2 cores, 49 MB, from buildbot.libretro.com; update Beetle
  PSX, its new build to be approved before it runs; remove 0") and then runs the jobs with one progress line.
- **The detail pane**: name, authors, licence (the `.info`'s words, with the licence's full text opened in a sheet where
  the folder holds one), source address, build date and the date fetched, SHA-256, size, firmware (§4.5), compatibility
  notes (§4.6), the core's own notes from its `.info`, the permissions its `.info` names, and three actions: *Approve…*,
  *Options…* (§4.4) and *Open Folder*.
- **Make Default** on a row sets that core as its console's engine (§4.3); the column shows the result at once.
- **Space**: `/Cores`' total and the free space of its disk, and what the marked work will add or free.

### 2.2 Big picture

The big-picture form is the frame's (`EmuSen_LibraryTools.md` §1, §1a), not a second window: the roles stack in the
menus' type, the consoles a row of sections changed by the shoulders, the table a list, the detail its own page (A on a
row), the apply bar's summary and *Apply* the help bar's main action, search through the docked on-screen keyboard.
Marking is the X button on a row, cycling through the marks the row is offered; the toolbar's actions are declared once
as `LunaAction`s with their pad buttons, so the help bar is generated, never written by hand. In big picture Cores is a
tile of Home's settings grid (`EmuSen_LibraryTools.md` §1b), under *System* beside the firmware and the controllers.

### 2.3 Fit, reach and records

Both forms fit 1280×800, 1920×1200 and the handheld's screen with nothing cut off or overlapping; every control has an
accessibility name; every action is reachable by the pad and appears in the command palette. The window's states — the
list, a row's detail, the Apply summary, a job running, the licence sheet, the approval question — are added to
`WindowFitAuditTests` and recorded under `EmuSen.WiseMan/Mistress/Specification/` (`EmuSen_InterfaceSpec.md` §1, §5),
in both forms. `ManagerLayout`'s own tests in LunaP carry the fit and reach of the layout itself, so Cores' tests hold
only what Cores adds.

## 3. What is shared with Packages, and what is Cores' own

### 3.1 Shared, not copied

| Part | Owner | Used by Cores as |
|---|---|---|
| `AdaptiveLayout` (roles, two forms, focus from roles, actions declared once) | LunaP, stage T0 of `EmuSen_LibraryTools.md` §5 | the window's description |
| `ManagerLayout` (filter sidebar, marked sortable table, detail pane, toolbar, apply bar with one progress line and the space meter) | LunaP, stage T1 | the whole window |
| the Apply summary sheet | LunaP, part of `ManagerLayout` | the plan before a run |
| the jobs model: a request as JSON, events as JSON lines, polled, stopped, freed (`IToolJob`, `emusen_tool_job_*`) | the platform library | every action (§6) |
| the job runner (a thread, its event queue, the stop flag) | a crate drawn out of `library-tool/src/job.rs`, `emusen-tool-job`, used by both tools | `emusen-core-tool`'s jobs |
| the HTTP client (`emusen-net`: `rustls` over the system's roots, `Download` with drained progress) | the platform library | the catalogue and core downloads |
| zip reading and atomic writes (`emusen_galaxia::archive`, `::atomic`), SHA-256 (`sha2`, already Galaxia's) | Galaxia | unpacking and the install |
| the approval question (`MainWindow.CoreApproval`'s dialog) | Mistress today; moved into the tool library (§3.4) | *Approve…* |
| the console settings window from the schema | Mistress (`EmuSen_Settings_Reference.md` §4) | *Options…* |

`ManagerLayout` holds no knowledge of EmuSen: Cores hands it rows, filters, marks and actions, as Packages does.

### 3.2 Cores' own

The row model (§4.1), the detail pane's content, the marks' meanings, and the core tool. Nothing else.

### 3.3 The frame did not exist, and was built first

**Built 2026-10-31** in LunaP (`docs/LunaP.md` §202, §203, branch `cores-window`): `AdaptiveLayout` and
`ManagerLayout<T>`, with `LunaAction.PadButton` and `IsMain`, held by LunaP's tests to fit at 1280×800, 1920×1200 and
1280×720 in both forms with Packages' columns and with Cores'. What follows is the reasoning recorded before.

On 2026-10-31 neither `AdaptiveLayout` nor `ManagerLayout` is built, nor Packages' M1, which was to build T1. Cores
cannot follow the frame without it, and building its own arrangement first would be the hand coding the frame exists
to remove. §9's first two stages are therefore T0 and T1 themselves, built against Packages' needs (§3 of its page:
its columns, filters, detail and space meter) as well as Cores', so that Packages' M1 then uses them unchanged (Q1).

### 3.4 Where the window lives

After `EmuSen_LibraryTools.md` §1b, Cores is a tool library, `EmuSen.Tools.Cores`, needing only LunaP, Galaxia, the
platform library and the frontend API's binding (`EmuSen.Crystal.Client`) — never Mistress. Mistress hosts it as a
window (*Tools ▸ Cores*) and as a big-picture sheet; Home and `emusen-tools cores` host it once they exist. The approval
question moves into the tool library so that Mistress's open path and Cores ask it in one form.

## 4. The data model

### 4.1 Three sources, each the owner of its facts

| Fact | Owner | Read by Cores through |
|---|---|---|
| what cores are installed, their trust and their claimed systems | the runtime's discovery (`EmuSen_Backend.md` §29.40.5) | the frontend API's `cores` listing |
| what a libretro core in `/Cores/libretro/<stem>/` is: its source, build, hashes, licence | the folder itself: `NOTICE` and the `.info` (`EmuSen_LibretroHost.md` §1.1) | the shared entry module (§7.1) |
| what libretro cores exist to download | the cached catalogue (§4.2) | `emusen-core-tool` |
| which engine runs a console, and a game | the runtime's `Engine` setting, console and game scope | `settings_get`/`settings_set`, `game_setting_get`/`_set` |

There is no database of Cores' own: an installed libretro core is its folder, and everything else is asked of its
owner. A row joins the four by the core's id (a libretro core's file stem, or a project core's engine name).

**A libretro core's id is its file stem, never its `corename`** [R, found 2026-10-31 by the libretro host's work]: the
buildbot names a core's files on the stem, and the `.info`'s `corename` can differ from it (`corename = "LRPS2"` ships
as `pcsx2_libretro.so` and `pcsx2_libretro.info`; `"PCEE2"` as `pcee2_libretro.so`). The stem is taken from the
zip's and the `.info`'s file names, names the folder and its files, and is `NOTICE`'s `stem`; the `corename` is
recorded as information only.

### 4.2 The catalogue

- **What it is**: libretro's core information archive (`assets/frontend/info.zip`, 294 KB on 2026-10-31 [R]) and the
  platform's nightly index (`nightly/<platform>/<arch>/latest/.index-extended`: one line per core zip, date, CRC-32 of
  the zip, file name [R]). Fetched only when the player asks (*Check for Updates*, or the first time Cores is asked to
  list what is available), into `/Cores/libretro/.catalogue/` as `info.zip` and `index-<platform>-<arch>`, each
  with the date fetched (Galaxia's tree has no cache directory, and the catalogue belongs beside the cores it lists); Cores works offline from the copies it has, showing their date.
- **Its rows**: each `.info` whose zip is in the platform's index. From the `.info` [R]: `display_name`, `corename`,
  `authors`, `license`, `permissions`, `display_version`, `systemid` and `database` (to place it under a console),
  `supported_extensions`, `firmware_count` and `firmwareN_desc`/`_path`/`_opt`, `notes`, `hw_render` and
  `required_hw_api`, `needs_fullpath`, `is_experimental`, `savestate`, `description`. From the index: the build date
  and the zip's CRC-32.
- **States**: *built in* (a project core), *installed*, *update* (the index's CRC-32 or date for the core differs from
  the one its `NOTICE` records), *available*, *waiting for approval* (installed, `trust` `needs_approval`), *needs refresh* (the proxy's and adapter's copies older
  than the installation's, §5.4), *not
  supported* (with the reason, §4.6), *added by you*.
- **Console placement** uses one table from libretro's `systemid` (and the `database` names where `systemid` is blank)
  to EmuSen's system ids. The libretro host needs the same table for `info()` (`EmuSen_LibretroHost.md` §4.1), so it
  has one owner, the systems table (`Platform/systems`), beside each system's other names (§7.2, Q7). Until the host's
  `emusen_systems::for_libretro` lands, the core tool takes the placement from its caller, and switches to that
  function when it does.

### 4.3 The default core, per console and per game

The runtime's settings already hold the engine per console (`consoles.json`, key `Engine`, its choices the console's
engines in discovery's order, the first the default) and per game (the game's own settings file, since SR10). Cores
reads and writes exactly those, through the frontend API, so that the game options, the metadata editor's
*Alternative emulator*, `cores default` and this window agree because they are one value. A libretro core becomes a
choice there as soon as discovery lists its engine, with no change to the settings code. The detail pane says how many
games choose a core other than their console's, and *Show Games* lists them in the library's filter; changing a game's
choice stays in that game's options, where the frontend already offers it.

### 4.4 A core's options

*Options…* opens the console's existing settings window with the engine named (the runtime's
`settings_schema_json(session, console, engine)` already takes one), so a libretro core's options, laid out from its
schema under the core's own categories (`EmuSen_LibretroHost.md` §4.7), are edited, reset and stored as every core's are.
Cores adds no editor of its own.

### 4.5 Firmware

For an installed core, the firmware it needs is the runtime's `firmware_for` answer; before download, the `.info`'s
`firmwareN_*` entries. Each row says what is needed, whether it is optional, and whether the player's own file was
found (recognised by content in the firmware settings), with *Open Firmware…* to the existing pane. Nothing is fetched,
shipped or prompted for beyond pointing at the player's own files.

### 4.6 Compatibility notes

Three kinds, kept apart so the player can tell them apart:

1. **What EmuSen's host can run**, decided from the `.info` alone: a core whose `required_hw_api` names only OpenGL,
   OpenGL ES or Direct3D is *not supported*, with that reason, since the host renders through software and Vulkan only
   (`EmuSen_LibretroHost.md` §5); a core for a system EmuSen does not know is *not supported* ("no EmuSen console runs
   this system"); an `is_experimental` core is shown only under its filter.
2. **What the project has found**, a small data file shipped with the build, `cores-notes.tsv`, one line per core id:
   the date tested, the build tested (its SHA-256), the conformance kit's result, and a sentence. Shown as "This exact
   build was tested by EmuSen on <date>" where the installed or offered build's SHA-256 is the one recorded, "An earlier
   build was tested on <date>" otherwise, or "Not tested by EmuSen". Never a gate (§11.4).
3. **The core's own `notes`**, as its `.info` writes them, escaped as descriptor text (`EmuSen_CoreAPI.md` §7.2 rule 5).

## 5. Download, verification and install

### 5.1 The source

The libretro buildbot's nightlies, one zip per core: `buildbot.libretro.com/nightly/<platform>/<arch>/latest/
<stem>_libretro.<ext>.zip` [R]. The stable releases carry no per-core archives, only one `RetroArch_cores.7z` of
274 MB per release (1.22.2, 2025-11-20 [R]), so they cannot serve one core at a time (Q3).

### 5.2 What can and cannot be verified

The buildbot publishes **no signatures and no cryptographic checksums** [R, listing read 2026-10-31]: its index carries
a CRC-32 per zip, and each zip member its own CRC-32. A CRC detects corruption and nothing more. So the chain is:

1. **Transport**: HTTPS to `buildbot.libretro.com`, certificate checked against the system's own store by `emusen-net`;
   no other host, and a redirect off it refused.
2. **Corruption**: the zip's CRC-32 equals the index line the plan was made from (a mismatch means the build changed
   or the bytes did; the job refuses and says which); every member's CRC-32 checked on unpacking.
3. **Shape, before anything is kept**: exactly one member, named `<stem>_libretro.<ext>`, with no folder, `..`,
   link or absolute path in its name (no zip-slip: the name is compared whole, never joined to a path first); its size
   under the runtime's bound (512 MB, `EmuSen_CoreAPI.md` §7.2 rule 3) both as the zip declares it and as the bytes
   actually inflate, the inflating stopped at the bound, and the zip's own size under the same; its header (ELF,
   Mach-O or PE) naming this machine's architecture.
4. **Identity**: its SHA-256, written into `NOTICE` with the source address, the build date, the index's CRC-32, the
   date fetched, the licence and the authors.
5. **Trust**: **nothing downloaded runs before the player approves it.** The runtime lists the new folder as
   `needs_approval`; the approval shows the name, authors, licence, systems and SHA-256 of the libretro core and names
   the project's proxy by the manifest (`EmuSen_LibretroHost.md` §2 (a)), says plainly that the core runs with the
   player's permissions inside its sandbox, and records the hash. A later change to the file is a new hash and is asked
   about again (§7.2 rule 2). Cores offers *Approve…* right after an install finishes, and never approves by itself.

**The approval is trust on first download**, and the question says so in those terms: libretro signs nothing, so the
player is trusting the build the buildbot served that day, and the recorded hash makes sure nothing else is ever run
in its place. The hash approved is the libretro core's own, recorded in `cores.approved` against its folder; the
host compares it before any runner starts: the proxy seals a `memfd` copy of the libretro core, hashes that copy, and
the runner loads the sealed copy, so the bytes approved are the bytes run whatever happens to the file afterwards
(`EmuSen_LibretroHost.md` §1.3, after its review). `NOTICE`'s SHA-256 and `cores verify` are integrity
checks only, never trust, since `NOTICE` sits in a folder the player (or anything running as the player) can write
(an independent review of the libretro host's design, 2026-10-31). Where a core is in `cores-notes.tsv` and its
SHA-256 equals the one tested, the approval adds "this exact build was tested by EmuSen on <date>": a second signal for
the player's decision, not a substitute for it.

### 5.3 The install

- **Staging** under `/Cores/libretro/.staging/<stem>-<pid>/`, on the same file system as its destination: the core
  unpacked, its `.info` copied from the catalogue, `NOTICE` written, the project's proxy (`emusen-core-broker.so`) and
  adapter (`emusen-libretro-host.so`) copied from the installation, and `broker.target` written naming the adapter's
  copy, as `EmuSen_LibretroHost.md` §1.1 lays the folder out (agreed with the libretro host 2026-10-31: the runner
  loads the adapter as a v1 library with no folder argument, so the adapter finds its core beside itself, and a copy per
  core is safe since each machine has its own runner); files 0644, folders 0755, owned by the player, so that §7.2
  rule 3's checks pass.

```
/Cores/libretro/<stem>/
    emusen-core-broker.so        the proxy hosts load (hash in cores.manifest)
    broker.target                one line: emusen-libretro-host.so
    emusen-libretro-host.so      the adapter's copy (hash in cores.manifest)
    <stem>_libretro.so           the libretro core, as downloaded
    <stem>_libretro.info         its core information, from the catalogue
    NOTICE                       §5.2 step 4, read and written by emusen-core-entry (§7.1)
    assets/                      data files beside firmware
```

  Every name in that folder, and `NOTICE`'s format (`format: 1` then the ten keys of §7.1, `#` lines skipped, a later
  format's extra keys kept and written back, every field checked on read and on write), is `emusen-core-entry`'s
  (`Platform/core-entry`, built 2026-10-31, 12 tests; keyed on the stem the same day).
- **Into place** by one rename to `/Cores/libretro/<stem>/`. A job stopped or failed at any point leaves the staging
  folder, which the next job of the tool removes; the destination is never half-written.
- **Assets** a core needs beside firmware, where its `.info` or the host's profile names them, are downloaded into
  `assets/` in the same job, under the same checks.
- An installation without the libretro host (an older build) installs nothing and says why.

### 5.4 Update

The update feed is the same index: a core is *update* when its index line's CRC-32 or date differs from its `NOTICE`.
Checked only when the player asks (Q5). An update stages the new build as an install does, renames the installed folder
to `/Cores/libretro/.previous/<stem>/`, renames the new one into place, and on any failure renames the previous
back. The new build has a new hash and is approved again before it runs. The previous build is kept until the next
update and offered as *Roll Back* (Q6); its approval is kept with it. A core loaded in this process is replaced on
disk only; the running game keeps the build it opened, and the next game opened uses the new one.

**Refresh, when EmuSen itself is updated.** The proxy's and the adapter's copies in each folder then differ from the
installation's, and discovery lists the folder as *needs refresh*, not *needs approval* (`EmuSen_LibretroHost.md`
§1.3). `cores verify` (and the window's *Verify*, offered on those rows and by the state's filter) copies them again
from the installation, rewrites `broker.target`, and checks the libretro core against its `NOTICE`; the libretro core
is not touched, so its hash and its approval stand and nothing is asked of the player. A folder whose libretro core
no longer matches its approval is the other case, *needs approval*, and Verify never clears it.

### 5.5 Remove

Removal deletes the folder Cores itself made under `/Cores/libretro/`, and revokes the approvals of its hashes
(`core_revoke`). The player's saves are the host's battery files, kept in EmuSen's stores and never in the folder
(`EmuSen_LibretroHost.md` §4.5), so they stay; so do the core's settings, so that reinstalling restores them. A console
whose `Engine` named the removed core falls back to its first engine, which the runtime already does for an engine no
longer listed; the detail pane says so before Apply.

### 5.6 The player's own files, and the system's permissions

Cores writes only under `/Cores` in the player's home, as the player, through ordinary file calls: it never asks for
elevated rights, never changes a file's owner or permissions outside the folders it makes, and never falls back to
another place when a write is refused; the refusal is shown with the system's reason. A core the player put in `/Cores`
by hand is the player's file: Cores approves or revokes it and shows it in the file manager, and never deletes it. A
project core is the installation's and is never touched.

## 6. The jobs

`emusen-core-tool`'s requests run as the library tool's do (`EmuSen_GamePackages.md` §12.3): `{"op": "core-…", …}`
handed to the platform library's `emusen_tool_job_start`, which routes the `core-` operations to the core tool (a minor
extension of that surface's request, as each `pkg` operation was), events polled as JSON lines. The window follows them
through the existing `IToolJob`; the shell's module through the same service. Operations: `core-fetch` (the catalogue and
index), `core-ls`, `core-info`, `core-get`, `core-update`, `core-rollback`, `core-rm`, `core-verify`. Events: `plan`,
`progress` (bytes and the step), `checked` (each check of §5.2 by name), `installed`, `refused` (with the reason in the
tool's words), `done`. Defaults and approvals are not jobs: they are the runtime's calls (§4.3, §5.2), made by the
window directly and by the module through its host's runtime.

### 6.1 The requests and events built (C2, 2026-10-31)

The platform library's `emusen.library` service is at **minor 1.5**: a request whose `op` begins `core-` is handed to
`emusen-core-tool`, whose context is the player's `/Cores` (`DataStore.Cores`), the buildbot for this platform and,
until the systems table's `for_libretro` lands, no placement of its own (a request's `"systems"` serves meanwhile).

| Request | Events |
|---|---|
| `{"op": "core-fetch"}` | `progress` (`step`), then `fetched` (`cores`, `skipped`: index lines not one of this platform's core zips, `date`), or `refused` (`reason`); `done` |
| `{"op": "core-ls", "show": "all" \| "installed" \| "available" \| "updates" \| "unsupported", "console": null \| <id>}` | one `core` per row; `done` with `counts` per state and `catalogue`, the date fetched or null |
| `{"op": "core-info", "core": <stem>}` | one `core` with the detail, or `refused`; `done` |

Each may carry `"systems": {<libretro database or systemid>: <id> | [<id>...]}`, matched as `for_libretro` matches
(the `database` names first, the `systemid` only where they are blank). A `core` event holds `stem`, `name`,
`corename`, `systems`, `source` (`libretro`), `version` (`display_version`), `build` (the installed build's date, else
the index's), `offered` (the index's date), `state` (`installed`, `update`, `available`, `unsupported` with `reason`,
or `added`), `license`, `authors` and `size`; `core-info` adds `description`, `notes`, `permissions`, `extensions`,
`needs_fullpath`, `experimental`, `savestate`, `hw_render`, `required_hw_api`, `firmware` (`desc`, `path`,
`optional`), `from` and `index_crc32`, and for an installed core `sha256` and `fetched`. `done` is `{op, counts,
catalogue, error?}`.

**The fetch.** Both files are fetched with `emusen-net`'s `same_host` (a redirect off the buildbot fails the fetch),
bounded at 16 MB and 4 MB, and checked (the archive's directory reads and holds at least one `.info`; the index has
at least one core line for this platform) before either cached copy is replaced, each by an atomic rename, so a
failed fetch leaves the catalogue as it was. Only `linux/x86_64` has a buildbot folder in this build, the only
listing read; another platform's is added once its listing is read, and until then `core-fetch` refuses with
"no buildbot listing is known for this platform".

**Measured on 2026-10-31**: `emusen-core-tool` 7 tests against a catalogue and index made by the test and served by
its own TLS server; `emusen-net` 19 (the same-host test new); `emusen-cores-module` 3; the platform library's
`cores` in a shell it makes over a cached catalogue; the library tool's own 18 tests unchanged after the job runner
was drawn out. Seeded faults, each caught: the update test inverted; the cache written before the check; a core
called unsupported when any of its APIs is OpenGL; `same_host` off; the same-host check ignoring the host; `core-`
requests sent to the library tool; the listing's fields out of order.

## 7. Surfaces shared with the libretro host

The libretro host is being built on its own branch. Four places need one owner between the two (Q7):

1. **The installed core's folder and `NOTICE`.** Written by the core tool, read by discovery and by the libretro host
   (which compares the libretro core with `cores.approved`, `NOTICE` being information only). `NOTICE` is `key: value` lines (`stem`, `name`,
   `corename` (information only), `authors`, `license`, `source`, `build`, `index_crc32`, `sha256`, `fetched`), read and written by one crate, Cores'
   `emusen-core-entry`, on which the libretro host depends (decided, §11.7).
2. **The table from libretro's `systemid` to EmuSen's systems**, the libretro host's: each system file of the systems
   table gains `"libretro_databases"` and `"libretro_systemids"`, and `emusen_systems::for_libretro(systemid, databases)`
   answers the systems a core runs, matching the `.info`'s `database` list (split on `|`) first and its `systemid` only
   where that list is blank, since one core may run several systems (PlayStation 2: `"Sony - PlayStation 2"`,
   `"playstation2"`). Read by the host's `info()` and by the catalogue (agreed 2026-10-31).
3. **The runtime's `cores` listing**, the libretro host's: each row gains `"libretro"`, null for a project core and
   otherwise `{"folder", "file", "sha256", "corename", "name", "version", "license", "authors": [...], "source",
   "build", "trust"}`, every field from `NOTICE` but `version` (the core's own library version) and `trust` (the
   libretro file's own standing, apart from the row's), so the window's rows and the approval read one answer (agreed
   2026-10-31).
4. **Stage 4 of `EmuSen_LibretroHost.md` §10** lists "the core list and download" and "the engine choice per console".
   Those are Cores; the host's stage 4 keeps the approval's content, the settings window from the schema, the sound
   rate, the cards and the cheats.

## 8. The `cores` command

Module `emusen.cores` (`core`, singular, is already the session's command that loads a game). Plain lines that pipe;
long jobs go on in the background as `cores<n>`, followed by `cores wait`, listed by `ps`, stopped by `kill`, as `pkg`'s.

```
cores ls [--installed|--available|--updates|--approval|--unsupported] [<console>]
                         one line per core: state, console, id, source, version, size, licence
cores info <core>        the detail pane as key: value lines
cores fetch              the catalogue and index fetched again (a network request; only when typed)
cores get <core>...      download, check and install; prints the SHA-256 to approve
cores update [<core>...] the installed cores whose index line changed, or those named
cores rollback <core>    the previous build back in place
cores rm <core>...       a downloaded core's folder removed and its approvals revoked
cores verify [<core>...] each installed core hashed against its NOTICE; stale proxy and adapter copies refreshed
cores approve <core>     the approval, at the prompt only, as 'trust': refused inside 'source' and $(...)
cores default <console> [<core>]          print, or set, the console's engine
cores default --game <path> [<core>|-]    print, set, or clear a game's choice
```

**Built in C2 (2026-10-31)**: `ls`, `info`, `fetch` and `wait`, in `Platform/emusen-cores-module`, reading events
with `emusen-library-module`'s `next_event`, shared rather than copied, since both follow jobs of the same service.
A core is named by its file stem (§4.1). `cores default` is held back: a module reaches the runtime's settings only
through a service, and none gives a module the console and game `Engine` setting yet (§10, Q11). The window sets the
default through the frontend API directly (C4).

Composes with the shell's tools: `cores ls --updates ps2 | wc -l`; `cores ls --installed | grep GPL`;
`for c in $(cores ls --updates | awk '{print $3}'); do cores info $c; done`; `cores verify | grep -v ok`. Its man page, `cores.1`, is written with the module.

## 9. Stages

| Stage | What | Gate |
|---|---|---|
| **C0** | `AdaptiveLayout` in LunaP (T0): roles, the two forms, focus from roles, actions declared once, its own fit and reach tests | LunaP's tests at 1280×800, 1920×1200 and the handheld's screen, both forms, the pad walk; seeded faults |
| **C1** | `ManagerLayout` on it (T1), checked against Packages' §3 and Cores' §2 | LunaP's tests; a harness window with Packages' columns and one with Cores' |
| **C2** | `emusen-tool-job` drawn out of the library tool unchanged in behaviour; `emusen-core-tool`'s catalogue (`info.zip`, the index), the installed listing, `core-ls`, `core-info`, `core-fetch`; the `emusen.cores` module with `ls`, `info`, `fetch`, `default`; `man cores` | the library tool's tests unchanged; the core tool's tests on a catalogue and index built in the test, served by a local TLS server with the test's own roots; DianaSh golden corpus |
| **C3** | `get`, `update`, `rollback`, `rm`, `verify`; staging and the renames | tests of every check of §5.2 refusing, a stopped job leaving the destination as it was, an update failing back to the previous build, nothing written outside `/Cores`; seeded faults |
| **C4** | `CoresWindow` in `EmuSen.Tools.Cores` on `ManagerLayout`, desktop and big picture; Make Default, Options…, firmware rows; hosted in Mistress | WiseMan headless: the window over a synthetic `/Cores` and catalogue; the fit audits and InterfaceSpec recordings in both forms; the pad walk |
| **C5** | With the libretro host merged: the proxy copied and `broker.target` written at install, the approval of the whole entry, the `libretro` field of the listing | WiseMan: a core installed from the local server (the host's test core, zipped by the test) approved, opened and run, then updated and removed, saves kept |

Tests never make a live network request and never download a real core: the catalogue, index and zips are made by the
tests. Seeded faults, three to seven a stage, aimed at the checks of §5.2 and the renames of §5.3–§5.4.

## 10. Questions for the tester

1. **[decision]** Build `AdaptiveLayout` (T0) and `ManagerLayout` (T1) as Cores' first stages, designed for Packages
   too, since neither exists and Packages' M1 has not started. *Recommended:* yes, T0 with the care
   `EmuSen_LibraryTools.md` §5 asks; the alternative is to wait for Packages' M1.
2. **[decision]** Cores takes over the libretro host's stage-4 items "the core list and download" and "the engine choice
   per console" (§7.4). *Recommended:* yes, so there is one window for cores.
3. **[decision]** The source is the buildbot's nightlies, the only per-core downloads libretro publishes; no other
   mirror. *Recommended:* yes.
4. **[decision]** Integrity as §5.2: HTTPS, the CRC-32s against corruption, the shape checks, the SHA-256 recorded and
   the player's approval as the trust decision, since libretro publishes no signatures; and a shipped
   `cores-notes.tsv` of what the project has tested, shown as information, never as a gate. *Recommended:* yes.
5. **[decision]** The catalogue and updates are fetched only when the player asks (*Check for Updates*, or the first
   listing of what is available); an opt-in "check when Cores opens", off by default. *Recommended:* yes.
6. **[decision]** Keep the previous build of each updated core for *Roll Back* until the next update (about one core's
   size again on disk). *Recommended:* yes.
7. **[decision]** The four shared surfaces of §7 (the `NOTICE` format and its one module, the `systemid` table in the
   systems table, the listing's `libretro` field, the stage-4 split), agreed with the libretro host's work before C2.
   *Recommended:* as §7 proposes.
8. **[decision]** Cores lives in a tool library, `EmuSen.Tools.Cores`, hosted by Mistress now and by Home later, with
   the approval question moved into it. *Recommended:* yes, as §1b of `EmuSen_LibraryTools.md` decided for every tool.
9. **[routine]** The command is `cores` in a new module `emusen.cores`, since `core` is taken. *Recommended:* yes.
10. **[routine]** Cores whose host cannot run them (OpenGL-only, an unknown system) are listed under *Not supported*
    with the reason rather than hidden. *Recommended:* yes.
11. **[decision]**, asked 2026-10-31 after C2: give command modules the runtime's `Engine` setting, console and game
    scope, through a narrow service (`emusen.settings`, get and set of one key, the runtime's own checks kept), so that
    `cores default` can be built. *Recommended:* yes, the service limited to the keys a module names in its manifest;
    the alternative, the module writing `consoles.json` itself, would bypass the setting's owner.

## 11. Decided 2026-10-31 by the tester

Every recommendation of §10 was taken, with these particulars:

1. T0 and T1 are built here (C0, C1), for Packages too, in LunaP as its README asks of general controls.
2. Cores takes the core list, the download and the engine choice per console from the libretro host's stage 4; the host
   keeps the runtime's side.
3. The buildbot's nightlies are the only source.
4. Integrity as §5.2, and `cores-notes.tsv` records the **SHA-256 of each build the project tested**, so the window can
   say "this exact build was tested"; information, never a gate.
5. The catalogue and updates are fetched only when the player asks; checking when Cores opens is opt-in, off by default.
6. One previous build per core is kept for *Roll Back*.
7. The owners of §7's surfaces: **Cores owns the `NOTICE` format and the one module that reads and writes it**, in a
   small crate both depend on (`Platform/core-entry`, `emusen-core-entry`); **the libretro host owns** the `systemid`
   table in the systems table and the `libretro` field of the runtime's `cores` listing. Cores builds against those two
   once they land, their exact shape agreed with the host's work beforehand.
8. Cores lives in the tool library `EmuSen.Tools.Cores`, hosted by Mistress now and by Home later.
9. The command is `cores`, in `emusen.cores`.
10. Cores the host cannot run are listed with the reason.
