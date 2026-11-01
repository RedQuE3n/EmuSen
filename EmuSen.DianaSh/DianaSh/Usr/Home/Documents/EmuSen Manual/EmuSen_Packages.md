# EmuSen_Packages — Packages, one window to manage the library's game packages and their mods

*Proposal, 2026-10-27; its questions answered by the tester the same day (§6). Nothing is built.* Asked by the tester: a
window where the player manages the library's games as packages — makes them, unmakes them, with the options the
packages have — laid out after the graphical package managers.

**Its name is Packages** (the package manager; named 2026-10-27 and given this plain name 2026-10-28 by the tester). The
window is `PackagesWindow`; menus call it *Packages*.

## 1. What exists, and what does not

- **The jobs exist.** The library tool's `pkg` jobs (`EmuSen_GamePackages.md` §11–§19) make a package (linked or
  self-contained, `--zip`), list the packages with their state (ok, moved, missing, changed), check every file against
  the manifest, migrate a game's saves, states, records, settings, art and cheats in, add and remove soft patches, zip
  a self-contained package, and set the identity the scraper sends. Each runs in the background with progress events.
  The CHD tool (`chd convert`, §4) is its sibling.
- **One game at a time.** Mistress's *Game Package* window (`GamePackageWindow`, §15 and §19) makes one game's package
  and edits its patches and identity.
- **Missing:** a view of the whole library, packaged and not, with sizes; and **unmaking**: no job takes a package
  apart.

## 2. The form it takes

Weighed against the code-design rules:

- **The window is Mistress's**, built from LunaP, and calls nothing but the library tool's jobs (`IToolJob`), as the
  Game Package window does. It is a front end over an existing surface, not a second owner of any rule.
- **Unmake is a new `pkg` job** in the library tool (`pkg unmake`), a minor extension of its request, so the shell and
  the window get it alike: `pkg unmake <package>... [--to <folder>]`.
- **The listing** gains sizes and the library's unpackaged games, as a minor extension of `pkg ls`'s outcome (a size per
  package and per game, and an `unpackaged` list), so the window draws from one answer.

## 3. Layout

Storefronts (GNOME Software, KDE Discover) show cards to browse, which suits finding one thing among many and does not
suit managing hundreds. The managers this follows are **Synaptic** and **Pamac**: filters at the side, a table with a
mark per row, a detail pane under it, and changes marked first and applied together; and **Steam's storage manager**
for the space used.

```
┌ Toolbar: [Make] [Unmake] [Check] [Zip] [Convert to CHD]           Search… ┐
├────────────┬──────────────────────────────────────────────────────────────┤
│ Consoles   │ ☐ Game                Console   Mode     State     Size      │
│  All       │ ☑ Sonic CD (US)       Sega CD   linked   ok        156 MB    │
│  …         │ ☐ Mickey Mania        Sega CD   linked   ok        410 MB    │
│ State      │ ☑ Jak and Daxter      PS2       —        —         4.1 GB    │
│  Packaged  ├──────────────────────────────────────────────────────────────┤
│  Not yet   │ Detail: files, patches, identity, what the package holds     │
│  Problems  │ (saves, states, art, settings), last check   [Open Folder]   │
├────────────┴──────────────────────────────────────────────────────────────┤
│ 2 marked: make 1 (linked), unmake 1                [Apply…]   space ▮▮▮▯  │
└───────────────────────────────────────────────────────────────────────────┘
```

- **Mark, then apply.** A row is marked for an action (make linked, make self-contained, unmake, check, zip, convert);
  **Apply…** shows what will happen before anything runs ("make 1 linked; unmake 1, its saves and art copied back to
  the stores first") and then runs the jobs in order with one progress line. Nothing destructive happens on a click.
- **Filters**: console; state (packaged, not packaged, problems: moved, missing, changed, a failed check); search by
  title.
- **The detail pane** is the Game Package window's controls (identity, migrate), reused, not copied, and the game's
  layers (§3b).
- **Space**: the packages' total and the free space of the disk they are on.
- The window fits 1280×800 and 1920×1200 (the fit audits).

## 3a. Converting discs to CHD (asked by the tester 2026-10-27)

Packages converts disc games to CHD through the CHD tool's existing jobs (`chd convert`, `chd check`, `chd ls`;
`EmuSen_GamePackages.md` §4 and §12), so it is the Compressed Discs window's work in the same list, and the Compressed
Discs window stays as it is.

- **Which rows**: disc games only (a sheet, an ISO, a GDI); the mark is offered on nothing else. A disc already in CHD
  shows its form in a **Format** column (CUE/BIN, ISO, CHD) and is not offered it again.
- **What a conversion makes**: as the tool already does, a converted disc gets a package whose game is the CHD and whose
  `dump` is the original (§4.4 there); the row's mode says linked, or self-contained where the player chose the CHD
  inside the package.
- **Options in Apply's summary**: linked (the CHD in `Compressed`) or self-contained; the threads chdman may use; and
  the check after each disc, on by default. The originals are **never removed** by Packages; the tool's replacement,
  with its per-batch confirmation (§4.5 there), stays in the Compressed Discs window.
- **Progress and outcome**: the job's events, a disc at a time, with the space saved; a refusal or a failed check in the
  row's state, in the tool's own words.
- **Space**: the space bar shows what the marked conversions are expected to save where the tool can estimate it, and what each saved once it is done.

## 3b. A package's layers: Packages is also the mod manager (consolidated 2026-10-27 by the tester)

A mod is a layer inside a game's package (`EmuSen_Modding.md` §1), so the tool that manages packages manages what is
in them; a separate mod manager would have shown the same game twice. In a game's detail pane:

- **Layers**: patches, texture packs, scripts, audio and enhancement codes, each on or off and in order, with the
  conflicts named (two layers writing the same bytes) and a patch checked against the dump before it is switched on.
- **Install** from the emusen-mods repository or a file; **Export Patched Game** to a folder the player chooses, never
  into the ROM library.
- Through the `mod` commands (`emusen.mod`), as the rest of Packages goes through `pkg` and `chd`; the *Game Package*
  window's patch controls move here, and `pkg patch add/rm` stay as aliases of `mod add/rm`.

## 4. Unmake

- **Its stores first.** A package holds the game's saves, states and their records, settings, art and cheats
  (`EmuSen_GamePackages.md` §14, §18). Unmaking copies each back to its usual store — `pkg migrate` in reverse — and
  only then removes the package folder; a store that already holds a file of the same name keeps it, and the package's
  copy is kept beside it under a dated name, so nothing is overwritten.
- **Linked**: the game never left the library, so the package folder goes and the library's file is the game again.
- **Self-contained**: the package holds the game itself, and **EmuSen never writes to the ROM library**. The game is
  exported to a folder the player chooses (`--to`); the window asks for it. With no folder given, the job refuses.
- **Patched**: the soft patches go with the package; the game unmade is the plain dump.

## 5. Stages

| Stage | What | Gate |
|---|---|---|
| **M1** | The window: the listing and its extension, filters and search, make, check, zip, and CHD conversion (§3a) through the existing jobs, mark and apply | WiseMan, headless, on a synthetic library; the fit audits; the jobs' outcomes shown |
| **M2** | `pkg unmake`: the stores copied out, linked and self-contained, `--to` | Library tool tests: every store copied out and found where the game unpackaged reads it; nothing overwritten; the ROM library untouched; seeded faults |
| **M3** | The detail pane (identity, migrate) from the Game Package window's controls, and the layers of §3b over the `mod` commands | WiseMan; the Game Package window unchanged until its controls have moved |

M1 and M3 are routine; M2 carries the risk of losing a save and is done with the most care.

## 6. Decided 2026-10-27 by the tester

1. **Unmaking a self-contained package exports the game to a folder the player chooses**; with none, it refuses.
2. **A big-screen (pad) version comes later**, as its own step. *Superseded the same day: every tool on the shared frame has a big-picture counterpart driven by a pad, carried by the frame itself (`EmuSen_LibraryTools.md` §1), so Packages has its own from M1.*
3. The name: **Packages** (above), the menus saying *Packages*.
4. The proposal as written: Synaptic-style mark and apply, unmake as a `pkg` job, the window over the library tool's
   jobs, in stages M1–M3.

## 7. Built 2026-10-31: M1, the window

*Started on 2026-10-31 by the tester's go for the new windows.* Packages is the first tool built as §1b of
`EmuSen_LibraryTools.md` asks: a library of its own, hosted by a thin host, on LunaP's manager frame
(`ManagerLayout<T>`, LunaP §203).

### 7.1 The listing: `pkg ls --all` (service 1.7)

§2's extension of the listing, in the library tool (`Platform/library-tool`) so that the shell and the window draw
from one answer:

- **The request**: `pkg-ls` gains `"unpackaged": true`; on any other op it is refused `PARSE`. The shell's
  `pkg ls [<console>] --all` sends it (`man pkg`).
- **A size per package**: each `package` event of `pkg-ls` gains `size` (the bytes its folder holds, hidden files
  included and links counted as links), `game_size` (its game's bytes where the game is found: the file and, for a cue
  sheet, every file it names), `console` (its system's console, named as the library's games are named, where the
  manifest's `system` is the ES-DE name) and `convertible` where `chd convert` takes its game.
- **The unpackaged games**: after the packages, one `game` event for each game of the library no package claims, found
  and routed as `pkg make` finds them: `index`, `of`, `source`, `system` (the console), `state: unpackaged`, `size`,
  and `convertible`. The start event gains `games`, and the end's counts `unpackaged`. A console narrows both lists.
- **Which discs convert** has one owner: the CHD tool's own test (`discs::item_of`, the predicate `chd convert`'s
  batch uses), exposed as `discs::convertible`; a playlist is never offered, since the tool does not convert one.
- `emusen_library_tool.h` 1.6 → **1.7** (built as 1.5 and renumbered when the core tool took 1.5 and `rom` 1.6, which landed first); the module asks for 1.7. The table is unchanged.

### 7.2 The tool library and its host

- **`EmuSen.Tools.Packages`** references LunaP and Galaxia and nothing else of EmuSen (`LeafAssemblyTests`). It holds
  `PackagesPane` (the content, a `ManagerLayout<PackageRow>`), `PackageRow` (one event of the listing as a row),
  `PackagesPlan` (the marks each row is offered and the marked rows as requests, free of any control so that it is
  tested alone), `PackagesWords`, `PackageDetail`, and `ToolWords`: the tool's refusals and reasons in a player's
  words, moved out of Mistress's `CompressedDiscsWords`, which now forwards to it, so one table serves both windows.
- **`PackagesWindow`** in Mistress is the thin host: a `ToolWindow` whose content is the pane, *Settings → Packages…*
  on the desktop and *Packages* on the pad menu's Library page (not Settings, whose page is full at nine rows;
  Settings Reference §4.69.8). Framed as a sheet in big picture it switches the pane to
  the big-screen form, hands the frame's generated hints to the help bar (face buttons as the player's swap draws
  them) and passes the pad's buttons to the frame in ES-DE's grammar (`IPadDriven`); what the frame declines, B at the
  top among them, is the router's. No focus routing or hint is written by hand.

### 7.3 What the window does

- **Columns**: Game, Console, Format (§3a: CUE/BIN, ISO, CHD, else the cartridge's extension), Mode, State, Size; the
  big screen shows Game and State. Game and Size sort.
- **Filters**: All, each console, then Packaged, Not yet and Problems (moved, missing, changed, unreadable).
  `AdaptiveLayout` holds one chosen section, so a console and a state are not combined; the search narrows either.
- **Marks** (`PackagesPlan.MarksFor`): a game not packaged, *make, linked*, *make self-contained*, *make and zip*, and
  *convert to CHD* where it converts; a package, *check*, and unless it has a problem *convert to CHD* where it
  converts, *make self-contained* and *make and zip* where it is linked to the library, *zip* where it is
  self-contained. A package with a problem is offered only *check*. The toolbar's Make, Check, Zip and Convert to CHD
  mark the chosen row where it is offered that change and do nothing otherwise; they stay enabled whichever row is
  chosen, since the pad's walk through the action row must not change with the row (the interface recording found
  Convert to CHD unreachable while it was enabled only for discs). X cycles the row's marks.
- **Apply** shows the plan first (`PackagesPlan.Text`): each kind of change with its games, ending "Nothing in the
  library is changed, moved or removed." Confirmed, the steps run one job each in a fixed order — make, make
  self-contained, make and zip, zip, convert, check — through `pkg-make`, `pkg-zip`, `convert` and `pkg-check`, with one
  progress line (the step, n of m, the game). Each event's outcome is kept and shown in the State column and the
  detail's *Last run*; then the library is listed again.
- **Refusals** (no library, chdman missing, the platform library not in use) are said in `ToolWords`' sentences in the
  apply bar's line.
- **Space**: the packages' folders' total and the free space of the disk `Packages` is on.
- **The detail** (M1): the package and its mode, the game and where it lies, the state with the tool's reason, the
  sizes, the format, the soft patches and *Open Folder* (Avalonia's launcher, so the platform's own file manager).
  M3 adds identity, migrate and the layers.

### 7.4 The gate, the tests, and the faults seeded

`PackagesWindowTests` (8): through real jobs of the platform library on a synthetic library (a SNES and an NES
cartridge and a PlayStation disc) the window lists three games not packaged with their sizes, the disc as CUE/BIN and
convertible; a linked and a self-contained package are made only once the plan is confirmed, then checked and zipped
through the pad's Start, each outcome on its row, and the library is byte for byte as it was. With scripted jobs: every
kind of row offered only its changes; the plan's requests one per kind in the fixed order; a conversion followed to its
outcome and chdman's absence said in words; the window without the platform library saying why and starting nothing;
the pad marking with X, opening and cancelling the plan, opening and closing the detail's page, stepping sections and
listing again; the desktop window whole at 1280×800 and 1920×1200. `WindowFitAuditTests` frames `Packages` and
`PackagesDetail` at both sizes, recorded as interface specifications (`EmuSen_InterfaceSpec.md` §6.3.12);
`LeafAssemblyTests` holds the library to LunaP and Galaxia; the library tool's `the_listing_with_unpackaged_…` holds
§7.1. Measured on 2026-10-31, each fault reverted after its run:

| Fault | Caught by |
|---|---|
| the unpackaged list keeps games a package claims | `the_listing_with_unpackaged_names_every_game_no_package_claims_with_its_size` |
| a package with a problem is offered more than *check* | `Each_row_is_offered_only_the_changes_that_fit_it` |
| *check* runs first | `A_plan_runs_one_request_for_each_kind_of_change_in_a_fixed_order` |
| the pad's X reaches North rather than *Mark* | `The_pad_marks_applies_and_climbs_back_through_the_frame` |
| a step's refusal is not put on its rows | `A_conversion_is_followed_to_its_outcome_and_a_refusal_is_said_in_words` |
| a package made self-contained is said to be linked | `Packages_lists_makes_checks_and_zips_through_real_jobs_…` |

### 7.5 Otherwise than §3, and left

- **Unmake** is not offered until M2 builds `pkg unmake`.
- **Converting self-contained in one mark**, chdman's threads and the check after each disc are not options of the
  plan: the frame's plan is text with Apply and Cancel, and the tool always checks each disc. A converted disc's
  package is made self-contained by marking it after the conversion. The space a conversion is expected to save is
  not estimated beforehand; each conversion's outcome says what it saved.
- **Interim forms, dropped once the manager frame's fixes landed** (found by these audits, 2026-10-31; LunaP be0112e):
  a refusal about no row is now the frame's notice in the apply bar's line (`SetNotice`); the space meter carries its
  full words; the Game and State columns are no longer capped, since the frame's tables fit their width (`FitsWidth`);
  the big screen offers the console sections again, its section row drawn whole around the chosen one; and
  `PackagesPlan` and `PackagesRunning` are audited and recorded. The detail shown when no game is chosen keeps the
  library's and the packages' totals.

## 8. Built 2026-10-31: M3, the detail and a package's layers

*Decided 2026-10-31 by the tester:* the `mod` commands (`EmuSen_Modding.md` X1) are not built, so the layers go over
`pkg patch`, which §3b keeps as the alias of `mod add/rm`; X1 is a stage of its own, queued next. The Game Package
window is kept for the per-game menus as a second host of the same control: one control, two hosts.

### 8.1 One control, two hosts

- **`GamePackageDetail`** (the tool library) holds the Game Package window's controls, moved rather than copied:
  *Make Package*; the package's **layers**, today its soft patches, each with *Remove*, and *Add* from the IPS, UPS and
  BPS files beside the game (or beside the library file a self-contained game was copied from) or, on the desktop, a
  file chosen with *Browse…*; *Scraped as*, over `pkg identity`; and, new, **Stores: Copy Saves and Art In**, over
  `pkg migrate` (§14 and §18.4 there), which copies the game's saves, states and their records, settings, art and
  cheats in and overwrites nothing. Its status line says each job's outcome in the tool's words.
- **The layers are a list of `PackageLayer`** (kind, name, format, place), read from the manifest, and the rows are
  drawn from it, so X1 adds kinds (texture packs, scripts, audio, codes) and the on/off and order controls to the list
  without reworking the control. `Patches()` is the list's patch kind.
- **Packages' detail** shows the facts of §7.3 and then the control for any game whose file can be named — in the
  library, or kept in its package — with *Browse…* on the desktop only; a change made there lists the library again.
- **`GamePackageWindow`** is a thin host: the control in a scroller and *Close*, its constructor, properties and the
  controls' names unchanged, so its menus, the game options sheet and the pad reach it as before.

### 8.2 The gate, the tests, and the faults seeded

`PackagesWindowTests.A_games_package_controls_in_the_detail_…`, through real jobs on a synthetic library: a cartridge
made into a package from the detail and the list taken again; the patch beside it added, its layer read back as
`patch 1 IPS`; the identity set to patched; the save in `Saves/SNES` copied in; the patch taken out; and the library
byte for byte as it was. `GamePackageWindowTests` (4) pass unchanged against the thin host. The six Game Package
recordings and Packages' two detail recordings were recorded again: the status line moved inside the control and the
Stores row is new. Their first recording found *Close* not reached by the pad, since a second button bar (Stores') gave
the recorder's walk two items of the same name; *Copy Saves and Art In* stands alone, and the rules find nothing.
Measured on 2026-10-31, each fault reverted after its run:

| Fault | Caught by |
|---|---|
| a change does not list the library again | `A_games_package_controls_in_the_detail_…` |
| *Copy Saves and Art In* runs `pkg check` | `A_games_package_controls_in_the_detail_…` |
| the detail names no game in the library | `A_games_package_controls_in_the_detail_…` |
| the layers numbered from 0, so *Remove* takes out the wrong one | `A_games_package_controls_in_the_detail_…`, `GamePackageWindowTests.A_package_is_made_and_a_patch_…` |

### 8.3 Left, for X1, X2 and X8

A layer switched on and off, the layers' order, conflicts named (two layers writing the same bytes), layer kinds other
than soft patches, installing from the emusen-mods repository, and *Export Patched Game*.
