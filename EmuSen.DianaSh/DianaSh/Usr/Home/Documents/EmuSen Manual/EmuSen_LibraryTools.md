# EmuSen_LibraryTools — the scraping tool, the library manager, and the frame they share with Packages

*Proposal, 2026-10-27; its questions answered by the tester the same day (§6). T2, the commands `rom hash`, `rom check` and `rom dups` and `uniq -D -w`, built 2026-10-31 (§7); the rest is not built.* Asked by the tester the same day: a proper scraping tool in place
of the many clicks a scrape takes today, laid out as Packages is (`EmuSen_Packages.md`) and made for theming, with
the system and the games (favourites or all) chosen; and a library manager laid out the same, which finds duplicate and
broken ROMs; every backend tool these lack made as a DianaSh command.

## 1. Three tools, one frame

**Packages, Scraper and Library (the library manager) are three tools**, not one window with three jobs: Packages changes
how a game is stored, the scraping tool fetches what is shown of it, the library manager reports on the files
themselves. Each stands alone and each calls only its own backend.

**What they share is a frame, and the frame is LunaP's**: a `ManagerLayout` control with a filter sidebar, a table with
a mark per row and sortable columns, a detail pane, a toolbar, and an apply bar that summarises the marked work and runs
it with one progress line (Synaptic's and Pamac's pattern, `EmuSen_Packages.md` §3). It holds no EmuSen knowledge;
a tool hands it rows, filters and actions. Built once, in Packages' M1, and reused by the other two.

**Every tool on the frame has a big-picture counterpart and is driven by a pad** (decided 2026-10-27 by the tester), so
the frame carries both forms: the desktop's panes side by side, and a big-screen sheet with the panes stacked in the
menus' type, the focus moved by the pad between the sidebar, the table, the detail and the apply bar, ES-DE's button
grammar, and the docked on-screen keyboard (`EmuSen_Shell.md` §3) for search. A tool built on it gets its
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
- **Text entry** in a big-screen form opens the docked on-screen keyboard (`EmuSen_Shell.md` §3) beneath the
  form, never over it.
- **Checked by the toolkit.** LunaP's own tests render every canonical layout in both forms at 1280×800, 1920×1200 and
  the handheld's screen, and walk the pad's focus through each, so a window built on them inherits fit and reach rather
  than proving them again.
- **`ManagerLayout` is one canonical layout** on `AdaptiveLayout` (navigation, a marked list, detail, an apply bar);
  the Shell window's is another (content over the docked keyboard); the settings windows (Mods, Cheats) will likely be a third
  (sections and their rows). The existing windows move onto it as each is remade, their hand-written sheet arrangements
  removed.

Recorded in LunaP's own document when built. A stage of its own, **T0**, before Packages' M1, built with the most care
(everything else stands on it).

### 1b. Where a tool is shown: tools as libraries, thin hosts, and Home the hub (decided 2026-10-27 by the tester)

After the Xfce Settings Manager, one window that opens the others, each of which also runs alone:

- **Every tool is a library** of its own (an assembly), needing only LunaP, Galaxia and the platform library — never
  Mistress. Its content is declared once (§1a) and **hosted** in any of: its own window; **Home**, the hub; a
  big-picture sheet; or the **standalone launcher** `emusen-tools <tool>`, a multi-call program (as busybox is) with a
  link and a desktop entry per tool, so a player can open, say, the package manager without starting EmuSen.
- **Home is the hub** and the settings: a grid of tiles by category with a search, each tile a tool, which fills
  Home's pane with a back button; the present Preferences window's tabs become small tiles of their own (General,
  Paths, Saves, Account, Big Picture). Galaxia is the one owner of the settings files every tool reads and writes, as
  xfconf is Xfce's. In big picture Home is the settings menu, a tile grid the pad moves through.
- **Home's first-run part, Setup,** is the welcome a new player meets: the same tools' panes in order — the ROM folder, the
  BIOS files, the controllers, a theme, the ScreenScraper sign-in — with Next and Back, built from the panes and not
  written again.
- **Limits**: a tool that needs a running game (the Debugger, Shell) opens one itself when standalone, as `emusen-shell`
  does; attaching to a running Mistress is a later feature. Mistress and a standalone tool open together are safe through
  Galaxia and SQLite, and Mistress picks up what changed; each tool has a test for it. Xfce embeds separate programs
  through XEmbed, which does not exist under Wayland; these are in-process libraries, which works on every platform.

### 1c. The family (consolidated 2026-10-27 by the tester)

| Tool | What it is |
|---|---|
| **Home** | the hub and settings; its first-run part is *Setup*, the welcome |
| **Scraper** | the scraping tool (§2) |
| **Library** | the library manager (§3) |
| **Controllers** | the controller window (`EmuSen_Controllers.md`) |
| **Display** | graphics settings and shaders (`EmuSen_Display.md`) |
| **Mods** | the mod workshop: graphics out and in, later translation (`EmuSen_Modding.md` §4) |
| **Packages** | packages, CHD and a package's mods (`EmuSen_Packages.md`) |
| **Shell** | the shell (`EmuSen_Shell.md`) |
| **Debugger** | the debugger and its Explore mode (`EmuSen_Debugger.md`) |
| **Cheats** | the cheat manager, in place of the cheat database and active cheats windows |

Each tool has one name, the one the menus show; the codenames it carried before 2026-10-28 are listed in
`EmuSen_Core_Naming_Scheme.md` §13.

Two tools that turn out to be one job are merged; a tool whose commands cover everything stays a command.

**The list of games is one answer**: the library scan's entries (`RomLibrary`, `RomEntry`), with what each tool adds as
columns: Packages its package mode and state, the scraping tool what each game has of each media type, the library
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
| **T1** | `ManagerLayout` in LunaP on `AdaptiveLayout` (Packages' M1 builds it) | with M1 |
| **T2** | `rom hash`, `rom check` (the size, sheet, archive and CHD checks), `rom dups`, `uniq -D -w` | routine |
| **T3** | The header checks, per system pack, with seeded faults | careful: a false "broken" sends a player hunting |
| **T4** | `emusen.scrape` and `scrape` | routine |
| **T5** | The scraping tool window | routine |
| **T6** | The library manager window | routine |
| **T7** | The trash operation in the platform library (Linux and its portal first), `trash`, and Move to Trash in the library manager | careful: it is the only write to the library |

## 6. Decided 2026-10-27 by the tester

1. Three tools on one frame, the library manager apart from Packages.
2. Duplicates in two tiers, the same dump and the same game.
3. The library manager reports and hides, and moves to the Trash where the player switches it on (§3.1), respecting
   the operating system's permissions; it never deletes.
4. Broken means the structural checks now; catalogues of good dumps come with the game database.
5. Names: the scraping tool is **Scraper**, the library manager **Library**; the codenames Ami and Rei were dropped 2026-10-28.
6. Order: Packages' M1 with T1, then the commands (T2, T4), then the windows; T3 and T7 with the most care, after
   2026-10-31.

## 7. Built 2026-10-31: T2, the `rom` commands and `uniq -D -w`

`rom hash`, `rom check` and `rom dups` are the command `rom` of the module `emusen.library`, beside `chd` and `pkg`, and
each is a job of the library tool (`Platform/library-tool/src/rom.rs`), reached through the service `emusen.library`,
now 1.5, as `chd` and `pkg` are: a module never links Galaxia, and the tool already reads the settings' library, routes
a file to its system and owns the disc reader's use. No new crate and no new surface beyond the service's minor were
needed. The page is `man rom` (`Lib/dianash/emusen.library/man/rom.1`), which holds what the player needs; this section
holds the rest.

### 7.1 The requests and events

Requests (since 1.5): `{"op": "rom-hash" | "rom-check", "paths": [...], "recursive": false, "comparable": false}`, the
paths relative to the library as `chd`'s and `pkg`'s are and at least one of them; `comparable` is `rom-hash`'s alone;
and `{"op": "rom-dups", "console": null}`, a console by its ES-DE name or id, no paths.

| Event | Fields |
|---|---|
| `start` | `op`, `files`, `refused` (the words for each path refused: no such path, or a folder without `-r`); for `rom-check` over a CHD `chdman` (its version) or `chdman_words` (why it is not run); for `rom-dups` `console` |
| `file` | `index`, `of`, `source`, `system` (null where no system routes it), `state`; `rom-hash`: `hashed` with `lines` (each `crc32`, `md5`, `sha1`, `size`, `name`), or `failed`/`stopped` with `words`; `rom-check`: `ok`, `failed` or `stopped` with `failed` (each `check`, `words`); `rom-dups`: `hashed`, or `failed`/`stopped` with `words` |
| `group` | `rom-dups` only, after every file: `members`, each as a `lines` entry with `package` where a package claims it |
| `end` | as every job's: `op`, `counts`, `seconds`, `error` |

The module prints only data on standard output (the hash lines, the check lines, the groups as paragraphs) and its
notes on standard error (a file it could not read, the summary of `check` and `dups`, chdman not run, a job gone to the
background), so that each pipes. `ps` reading a background `rom` job's progress keeps the lines it takes, and `rom
wait` prints them first.

### 7.2 The comparable form

Two copies of one dump give one comparable line. The form is owned where it already was: a cartridge's is
`emusen_systems::openvgdb::bytes`, the transform `pkg make` records as a package's `dump.transformed` (the iNES header,
a SNES copier header, an N64 image put in big-endian order); a zip's is its image entry, copied out by Galaxia's
`archive::copy_entry`, which checks its CRC-32, then the same transform. A disc's is new here, and is the one decision
of the stage: every sector as `emusen-disc` reads it, opened by `open_medium` over the files mapped, a compact disc's
2,352 raw bytes from track 1's index 01 to the lead-out and a DVD's 2,048 user bytes; 2,048-byte images are a compact
disc where the system takes sheets, else a DVD, as the console's word decides it in the runtimes. Reading through the
one disc reader is what makes a sheet of one file, the same disc split a file a track, and its CHD equal: none of
their files' bytes agree, and the reader is the one place their sectors do. The tests show all three equal
(`tests/rom.rs`, the CHD made by the pinned chdman).

Kept out, and why: a zipped disc compares as its image entry's bytes, so only with another zip of it, since a disc
built in memory from a zip would hold the whole disc; an `.m3u` has no line, its discs follow it; a disc whose sheet
records an unstored pause where another copy stores silence of another mode could differ in the synthesised sectors.
Each misses a duplicate; none makes a false one, since equal hashes over every sector are equal discs.

### 7.3 The checks

`size`, `sheet`, `archive` and `chd`, as §3 lists them; their words are in `man rom`. A plain file is read through
(an I/O error mid-file is "unreadable"), since only reading finds a file a network share or a failing disk will not
give back. A sheet is checked as the disc reader would take it: it parses, every file is there and reads, the last
track of each file ends on a whole sector (`emusen_disc::set::track_spans` places each track), and the disc opens.
A CHD is held first to chdman's verify, the tool's own check 1, where the pinned chdman is there, and only then read
sector by sector through EmuSen's reader, whose CHD code refuses a hunk whose CRC is not the map's; the order is the
library tool's (§12.4 of `EmuSen_GamePackages.md`: a damaged CHD is not read after chdman fails it). Without chdman
`rom check` says so once and reads each CHD through the reader alone. "Shorter than its header declares" and the header
checksums wait for T3, since each is a system pack's to own.

### 7.4 Duplicates

`rom dups` groups the library's files by comparable CRC-32, MD5, SHA-1 and size; a group whose files are all claimed by
one package (Galaxia's claim index: a CHD written beside its original and that original) is one game and is not shown.
No group is given when the job was stopped, since a group of what was read is not the library's. The same pairs come
from `rom hash -r --comparable / | sort | uniq -D -w 40`: a line begins with the CRC-32, a space and the MD5, so 40
characters hold 124 bits of hash; the platform test runs both in one shell and requires the same lines. The shell's
ten seconds bound the pipe as they bound any line, so a whole library is `rom dups`, followed by `rom wait`.

`uniq` gained `-D` (every line of each run of two or more) and `-w <chars>` (the key is a line's first characters), as
GNU uniq has them, in the Rust `dianash.text` and in the C# class it replaces, which the text-parity test now also
compares on both options.

### 7.5 Tests and seeded faults

`Platform/library-tool/tests/rom.rs` (3 tests over a generated library: headered and bare NES, plain, copier and zipped
SNES, one disc as one file, as two and as a CHD, and a broken file of each check's kind; the library compared byte for
byte before and after), the module's request and line tests, `dianash-text`'s `uniq` test, the platform test running
`rom dups` and the pipe in one shell, and in WiseMan the golden files `rom`, `uniq`, `help` and `man` and the
text-parity lines (the golden corpus and parity classes, 104 tests, measured on 2026-10-31). Seven faults seeded
on 2026-10-31, each reverted, each caught: a disc read from sector 0 rather than track 1's index 01 (`rom hash`'s
test, by the size of the program area); the system pack's transform dropped (the headered and copier copies no longer
equal, in `hash` and `dups`); a track ending inside a sector passed (`check`'s test); the package rule dropped, so a
package's own files made a group (`dups`' test); a zip's entries not inflated (`check`'s test, `Bad.zip` passed);
`uniq -D` printing runs of one (`uniq`'s test); and the hash line's CRC-32 and MD5 swapped (the module's test).

### 7.6 Left

- The size of a full run: `rom dups` reads every byte of the library, measured on no real library yet. Sizes could be
  compared first where the comparable size is known without reading (a file with no transform), if a run proves slow.
- Zipped discs, as §7.2 says.
- T3's header checks, and the second tier of duplicates (the same ScreenScraper game id), which is the Library window's.

