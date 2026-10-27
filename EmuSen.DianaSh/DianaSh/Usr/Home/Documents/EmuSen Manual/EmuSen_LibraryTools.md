# EmuSen_LibraryTools — the scraping tool, the library manager, and the frame they share with Setsuna

*Proposal, 2026-10-27; its questions answered by the tester the same day (§6). Nothing is built.* Asked by the tester the same day: a proper scraping tool in place
of the many clicks a scrape takes today, laid out as Setsuna is (`EmuSen_PackageManager.md`) and made for theming, with
the system and the games (favourites or all) chosen; and a library manager laid out the same, which finds duplicate and
broken ROMs; every backend tool these lack made as a DianaSh command.

## 1. Three tools, one frame

**Setsuna, the scraping tool and the library manager are three tools**, not one window with three jobs: Setsuna changes
how a game is stored, the scraping tool fetches what is shown of it, the library manager reports on the files
themselves. Each stands alone and each calls only its own backend.

**What they share is a frame, and the frame is LunaP's**: a `ManagerLayout` control with a filter sidebar, a table with
a mark per row and sortable columns, a detail pane, a toolbar, and an apply bar that summarises the marked work and runs
it with one progress line (Synaptic's and Pamac's pattern, `EmuSen_PackageManager.md` §3). It holds no EmuSen knowledge;
a tool hands it rows, filters and actions. Built once, in Setsuna's M1, and reused by the other two.

**Every tool on the frame has a big-picture counterpart and is driven by a pad** (decided 2026-10-27 by the tester), so
the frame carries both forms: the desktop's panes side by side, and a big-screen sheet with the panes stacked in the
menus' type, the focus moved by the pad between the sidebar, the table, the detail and the apply bar, ES-DE's button
grammar, and the docked on-screen keyboard (`EmuSen_HotaruWindow.md` §3) for search. A tool built on it gets its
big-picture form from the frame rather than as a second window.

### 1a. Beneath the frame: one description, two forms (decided 2026-10-27 by the tester, to come first)

Today each window that has a big-screen form arranges it by hand: LunaP's `MenuLook` restyles stock controls and
`SheetLayer` presents a window as a sheet, but how the parts rearrange and where the pad's focus goes is written again in
every window (the shader window keeps a second arrangement for its sheet). Toolkits that serve phones, tablets and
desktops settled this the same way — libadwaita's split views and breakpoints, Material's canonical layouts (list-detail,
supporting pane), SwiftUI's `NavigationSplitView`: **a window says what each part is, and the toolkit arranges it for
the form it is shown in.** LunaP gains that, and the frame is built on it.

- **Roles, not positions.** A window declares its parts by role: *navigation* (a sidebar of filters or sections),
  *content* (the list or the main view), *detail*, *actions*, *status*, *search*. LunaP's `AdaptiveLayout` places them:
  side by side on the desktop; on a big-screen sheet stacked in the menus' type, the navigation a row of sections or a
  page before the content, the detail beneath or on its own page, at the sheet's scale.
- **Focus from roles.** The pad's order follows the roles — navigation, content, detail, actions — with ES-DE's button
  grammar (A choose, B back a level, the shoulders between sections), so no window writes its own focus routing.
- **Actions declared once.** An action (LunaP's `LunaAction`, extended) carries its name, its key, its pad button and
  whether it is the main one; the desktop draws it as a toolbar or menu item, the big screen as a button and its entry
  in the help bar. A window never writes "Select / Choose" hints by hand.
- **Text entry** in a big-screen form opens the docked on-screen keyboard (`EmuSen_HotaruWindow.md` §3) beneath the
  form, never over it.
- **Checked by the toolkit.** LunaP's own tests render every canonical layout in both forms at 1280×800, 1920×1200 and
  the handheld's screen, and walk the pad's focus through each, so a window built on them inherits fit and reach rather
  than proving them again.
- **`ManagerLayout` is one canonical layout** on `AdaptiveLayout` (navigation, a marked list, detail, an apply bar);
  Hotaru's is another (content over the docked keyboard); the settings windows (Michiru, Haruka) will likely be a third
  (sections and their rows). The existing windows move onto it as each is remade, their hand-written sheet arrangements
  removed.

Recorded in LunaP's own document when built. A stage of its own, **T0**, before Setsuna's M1, built with the most care
(everything else stands on it).

**The list of games is one answer**: the library scan's entries (`RomLibrary`, `RomEntry`), with what each tool adds as
columns: Setsuna its package mode and state, the scraping tool what each game has of each media type, the library
manager its hashes and check result.

## 2. The scraping tool

Today a scrape starts from a game's menu, from Preferences ▸ Scraping or from the pad menu, with its choices spread over
the preferences pane. The tool gathers them.

- **Choose the games**: a system or all; all games, **favourites**, a collection, the games missing a chosen media type,
  or the rows marked by hand. Games marked "exclude from multi-scraper" stay out unless marked by hand.
- **Made for theming**: the media types are chosen per run, and a **"what my theme shows" preset** ticks exactly the
  types the active big picture theme draws for that system, read from the theme loader (the elements and the media
  types its variant triggers and images name). The table shows, per game, which of those types it has and from where
  (the package, the store, an ES-DE folder), so the gaps a theme would show are visible before scraping.
- **Options**: overwrite or keep what is there (ES-DE's "Overwrite files and data"), region and language, videos (off by
  default, §21 Q20 of `EmuSen_BigPicture.md`), and the account's limits (threads, per minute, today's requests left)
  shown beside them.
- **Apply** shows the plan before anything is sent ("38 games, about 70 requests, 9 already complete") — the existing
  `ScrapePlan` — and the run's progress replaces the separate status window while the tool is open.
- The quota rules, the order of the sources and the stores are the scraper's and do not change.

## 3. The library manager

- **Duplicates**, two tiers:
  - **The same dump**: equal content hashes in each system's comparable form (`emusen_galaxia::rom_hash`, the header a
    system pack strips taken off), so a headered and a headerless copy, a `.sfc` and the same game in a `.zip`, or a
    CUE/BIN and its CHD match. A package and the library file it claims are one game already and are never shown as
    duplicates.
  - **The same game, another dump**: the same ScreenScraper game id in `media.db` (a revision, a region, a hack),
    shown as a softer group, only for scraped games.
- **Broken or suspect**, by checks a dump can be held to without any database:
  - the file unreadable, empty, or shorter than its header declares;
  - a header checksum that does not hold, where the console's header has one (the SNES, Mega Drive, Game Boy and Game
    Boy Advance headers, the N64's boot CRCs), each check owned by that system's pack beside its header transform;
  - a disc sheet naming a missing track, or a track whose size is not a whole number of its sectors;
  - an archive whose stored CRC-32 does not match its contents; a CHD failing `chd check`;
  - a game ScreenScraper did not recognise by any hash, shown as "unrecognised", which is a hint, not a verdict.
  Checking dumps against No-Intro's and Redump's catalogues of good dumps waits on the game database decision (the
  to-do's "own OpenVGDB").
- **What it does with them.** A duplicate or a broken game can be **hidden** from the library (an edit in `games.db`,
  undone the same way), the list saved as text, the file shown in the system's file manager, and, where the player has
  switched it on, **moved to the system's Trash** (§3.1). Nothing is ever deleted outright.

### 3.1 Move to Trash: the one exception to a read-only library (decided 2026-10-27 by the tester)

EmuSen otherwise never writes the ROM library. Precedent was weighed: RetroArch removes only playlist entries; ES-DE's
metadata editor and LaunchBox delete files after a confirmation; OpenEmu offers the Trash; RomVault and clrmamepro move
unwanted files aside rather than delete them. The form chosen is OpenEmu's, the recoverable one.

- **Off by default**, switched on in the library manager's settings; offered only in the library manager.
- **Confirmed per batch**, every file listed by name and size, and the Trash named as where they go.
- **Never offered** for a file a package claims, a track another game's sheet names, a file inside an archive (the
  archive is the unit), or anything outside the ROM library.
- **The operating system's permissions are respected, never worked round.** The move is the platform's own trash
  operation, run as the player with the rights they already have: the freedesktop Trash on Linux (the home trash, or
  the volume's own `.Trash-<uid>` as the specification says), and through the Trash portal inside a Flatpak; the Recycle
  Bin on Windows; the Finder's Trash on macOS; MediaStore's trash request on Android, which asks the player itself.
  EmuSen never changes a file's permissions or owner, never asks for elevated rights, and **never falls back to
  deleting**: a file the player may not move (read-only, owned by another user, on a read-only mount, on a volume with
  no trash such as many network shares) is refused, by name, with the system's reason.
- **One owner**: the trash operation lives in the platform library (beside Galaxia's paths) behind its C interface,
  and is also the DianaSh command `trash <path>...` (§4), so the shell and the window do it the same way.

## 4. The DianaSh commands

Every one is a command first and the window's backend second, prints plain lines that pipe, and has a man page.

| Command | Module | What it does | Built on |
|---|---|---|---|
| `rom hash [-r] [--comparable] <path>...` | `emusen.library` | one line per file (or per track): CRC-32, MD5, SHA-1, size, path; `--comparable` in its system's comparable form | `rom_hash`, unchanged |
| `rom check [-r] <path>...` | `emusen.library` | one line per file: `ok`, or each failed check by name | the system packs' checks (new), `emusen-disc`, `chd check` |
| `rom dups [<console>]` | `emusen.library` | the duplicate groups, one per paragraph | `rom hash --comparable` and the claim index; the same as `rom hash -r --comparable \| sort \| uniq -D -w 40`, which the shell can also run |
| `scrape plan\|run\|ls\|status ...` | `emusen.scrape` (new) | `--system`, `--favorites`, `--collection`, `--missing <type>`, `--kinds`, `--overwrite`; `plan` prints what a run would ask, `status` the quota | the scraper's existing requests (`Platform/scrape`) |

| `trash <path>...` | `dianash.text` | moves files to the system's Trash as §3.1 says; `trash -l` lists what EmuSen moved there, to restore | the platform's trash operation (new) |

`uniq` gains `-D` and `-w` if it lacks them, so the duplicate pipe works with the shell's own tools. `rom` joins `chd`
and `pkg` in the library module, which is the module for the library's files; `scrape` is its own module, as the
scraper is its own crate.

## 5. Stages

| Stage | What | Model |
|---|---|---|
| **T0** | `AdaptiveLayout` in LunaP: roles, the two forms, focus from roles, actions declared once, its own fit and reach tests (§1a) | careful, after 2026-10-31 |
| **T1** | `ManagerLayout` in LunaP on `AdaptiveLayout` (Setsuna's M1 builds it) | with M1 |
| **T2** | `rom hash`, `rom check` (the size, sheet, archive and CHD checks), `rom dups`, `uniq -D -w` | routine |
| **T3** | The header checks, per system pack, with seeded faults | careful: a false "broken" sends a player hunting |
| **T4** | `emusen.scrape` and `scrape` | routine |
| **T5** | The scraping tool window | routine |
| **T6** | The library manager window | routine |
| **T7** | The trash operation in the platform library (Linux and its portal first), `trash`, and Move to Trash in the library manager | careful: it is the only write to the library |

## 6. Decided 2026-10-27 by the tester

1. Three tools on one frame, the library manager apart from Setsuna.
2. Duplicates in two tiers, the same dump and the same game.
3. The library manager reports and hides, and moves to the Trash where the player switches it on (§3.1), respecting
   the operating system's permissions; it never deletes.
4. Broken means the structural checks now; catalogues of good dumps come with the game database.
5. Names: the scraping tool is **Ami**, the library manager **Rei**; menus say *Scraper* and *Library Manager*.
6. Order: Setsuna's M1 with T1, then the commands (T2, T4), then the windows; T3 and T7 with the most care, after
   2026-10-31.
