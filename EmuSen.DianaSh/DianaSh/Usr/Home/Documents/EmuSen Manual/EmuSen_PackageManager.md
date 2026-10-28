# EmuSen_PackageManager — Packages, one window to manage the library's game packages and their mods

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
