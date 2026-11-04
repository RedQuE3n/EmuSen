# EmuSen_LibraryTools — the scraping tool, the library manager, and the frame they share with Packages

*Proposal, 2026-10-27; its questions answered by the tester the same day (§6). T2, the commands `rom hash`, `rom check` and `rom dups` and `uniq -D -w`, built 2026-10-31 (§7); T4, `scrape` (§8), T5, the Scraper window (§9), and T6, the Library Manager window (§10), built 2026-10-31 and 2026-11-01; T3 and T7 are not built.* Asked by the tester the same day: a proper scraping tool in place
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
| **T4** | `emusen.scrape` and `scrape` — built 2026-10-31 (§8) | routine |
| **T5** | The scraping tool window, composing `scrape`; Mistress's `ScopeGames` routed through the command's choice so that it has one owner (§8.6) — built 2026-11-01 (§9) | routine |
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

`rom hash`, `rom check` and `rom dups` are the command `rom` of the module `emusen.library`, beside `chd` and `pkg`,
and each is a job of the library tool (`Platform/library-tool/src/rom.rs`), reached through the service
`emusen.library`, now 1.6 (1.5 being the core tool's `core-` ops, `EmuSen_Cores.md` §6.1), as `chd` and `pkg` are: a
module never links Galaxia, and the tool already reads the settings' library, routes a file to its system and owns the
disc reader's use. No new crate and no new surface beyond the service's minor were needed. The page is `man rom`
(`Lib/dianash/emusen.library/man/rom.1`), which holds what the player needs; this section holds the rest.

### 7.1 The requests and events

Requests (since 1.6): `{"op": "rom-hash" | "rom-check", "paths": [...], "recursive": false, "comparable": false}`, the
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

## 8. T4 as built: the `emusen.scrape` module and `scrape` (2026-10-31)

### 8.1 The form chosen

Decided 2026-10-31, before building: the command reaches the scraper through a service, as `chd`, `pkg` and `rom`
reach the library tool, since a command module links neither the scraper nor Galaxia. Three forms were weighed (the
rule of the code-design section): a scraper of the module's own was refused outright, being a second scraper; an
extension of `emusen_scrape.h` was not needed, since every request the command makes is already one of its reads or
changes; so the scraper is lent whole as a **new service, `emusen.scrape` 1.0** (`Platform/include/emusen_scrape_service.h`),
and the command is a module of its own, `emusen.scrape` (`Platform/emusen-scrape-module`), as §4 says. Nothing of the
scraper changed: its quota rules, the order of its sources and its store are what a run from a game's menu meets. The one
addition to the crate is `Scraper::sqlite()`, so that the service's scan uses the SQLite the host lent rather than
loading another.

### 8.2 One scraper a process

The service's calls take no handle. Each speaks for the process's one scraper, held in a slot of the platform library
under a lock (`Platform/platform/src/scraping.rs`):

1. A host that holds a scraper lends it with `emusen_scraping_lend` (platform interface 28), with the `games.db`
   library it was opened with, the systems the build opens and, where it loads themes, a function answering the kinds
   its active theme draws for a system. Mistress lends its native scraper as soon as it opens it (`OpenNative`), and
   says at start, before any shell exists, that it scrapes with none open (a null lend). Closing the scraper or its
   library withdraws the lend first (`ScrapingLend.Release`), and a lend or withdrawal waits for the calls in flight,
   so a handle is never closed under a shell's call. A service `wait` is held to 250 ms for the same reason.
2. In a process whose host never lent one, the platform library opens one on the first `scrape` that needs it, from the
   settings file as the library tool reads it: the store under the tree's `Media`, `games.db` under `Library`, the
   art and ES-DE folders, OpenEmu's switch, and the developer file where it is (no embedded credentials outside the
   program). The runtime catalogue is not here, so each system is taken as its own engine for the art index: a picture
   placed by hand under a folder two systems share is found only through the system that names it.
3. After any lend the platform library never opens a scraper of its own; a lend made after one was opened closes it.
   A process therefore holds one scraper and one quota manager. Across processes the store lock of §8.8 refuses a
   second.

### 8.3 Choosing the games

The choice is the module's alone. The games are a scan of the settings' ROM folder, with `Packages` and `Compressed`
beside it, on a database of the service's own, as `RomLibrary.ScanNative` makes it, each game given its ES-DE system
from the systems table; a game with none is not one the scraper can ask about. The criteria are read through
`emusen_library.h`: `records` for favourites, `collections` and `members` for a collection (its name matched ignoring
case), `all_edits` for `nomultiscrape`. A game marked "Exclude from multi-scraper" is left out of a wide choice, as
Mistress's `ScopeGames` leaves it out, and is chosen when it is named by hand, as an argument or a line of standard
input (its first tab-separated field, so that `scrape ls` pipes back in). `--missing <kind>` keeps a game only when
the scraper's `media` read finds no picture of the kind from any source; `--missing any` when it lacks one of the kinds
the run asks. The run's priority is `console` for one system and `library` otherwise.

### 8.4 The kinds, the theme and videos

With no `--kinds` a run asks the scraper's own default (cover, screenshot, marquee, miximage). `--kinds theme` asks the
host for each chosen system's kinds; Mistress answers by loading the active theme for that system as though every type
were present, so that no trigger hides what it can draw, and taking the gamelist's `imageType` lists, `video` for a
video element, and the media types of the selected variant's triggers (`ThemeMediaKinds`). The theme and its choices
are a snapshot taken when the library is shown, so the shell's thread never reads the window's settings. Outside
EmuSen no theme loader is lent and `--kinds theme` is refused with its reason. A run over several systems asks the
union. Videos are asked only with `--videos` (`EmuSen_BigPicture.md` §21 Q20); a video named otherwise is dropped with
a note.

### 8.5 What it prints

Data lines on standard output, notes and sentences on standard error, as `rom` does: `ls` a line a game (path, system,
each kind's source or `-`); `plan` the scraper's `ScrapePlan` as `key<TAB>value` lines and its sentence; `run` the
plan's sentence first and then a line a finished game, the counts at the end; `status` the quota, whether a member is
signed in and since when, the developer credentials' origin, and the run going. No account name, password or developer
credential is printed; the credentials are the scraper's, read as they always were. A run past the line's budget
becomes `scrape<n>`, listed by `ps`, followed by `scrape wait`, cancelled by `kill`. The page is `man scrape`.

### 8.6 For T5

The Scraper window composes `scrape`. Mistress's own `ScopeGames` and its `Meets` criteria remain until then and are
not copied; T5's first step routes Scrape Games through the command's choice, so that the choice has one owner. Done 2026-11-01 (§9.2).

### 8.7 Tests, measured 2026-10-31

- `emusen-scrape-module` (7): the command line, a piped list, the choice and its exclusion rule, the theme's kinds per
  system and the videos rule, the plan, start, result, end and status lines (no account name), the manifest.
- `Platform/platform/tests/scraping.rs`: a shell this library makes, over a scraper lent as a host lends it and a
  recorded-response HTTPS server on 127.0.0.1 answering the scraper's own recorded `jeuInfos` fixture, with a
  certificate made at test time; status, `ls` by system and by the theme, `plan` with and without favourites, a game
  named by hand through a pipe, nothing sent before `run`, a run that finds the game and saves its cover, `--missing`
  afterwards, the library unwritten, and a withdrawal after which the service refuses and has opened nothing.
  `scraping_default.rs`: a process with no lend opens its own store and `games.db` from the settings, and a later lend
  closes it. Both load SQLite through `emusen_sqlite::for_tests`, which makes `libm` global first: the NuGet build
  leaves `trunc` to its host, and tests that loaded it otherwise passed without reaching SQLite.
- `ScrapingLendTests` (WiseMan): Mistress's binding lends a `MediaService`, a native shell answers from it, and
  disposing it withdraws the lend; `ThemeMediaKinds` on a synthetic theme.

Open question: whether the other Rust tests that load SQLite should all use `emusen_sqlite::for_tests`, which since
2026-10-31 loads `libm` first and fails loudly where no copy loads; the scraping tests use it.

### 8.8 One scraper a store across processes (`emusen_scrape.h` 1.1)

Decided 2026-10-31 by the tester: every scraper handle holds its store alone, so that a second process (Mistress and a
standalone shell at once) refuses rather than paces on its own and the account's quota is counted once. The lock is
Galaxia's advisory lock (`medium_lock`, the `flock` on `<store>/media.db.lock` that save media use), taken in
`Scraper::open` before `media.db` is opened and let go after everything else of the handle has closed. A second open
of a held store, in another process or in the same one, is `EMUSEN_SCRAPE_STORE_IN_USE` (−1167, the header's 1.1)
with words naming the holder ("another program", or "EmuSen's scraper in this program") and what to do. Mistress shows
the words when its store cannot be opened; a shell's `scrape` prints them. `tests/store_lock.rs` holds the store in a
child process of the test binary and is refused in the parent, then opens once the child lets go, and is refused a
second handle in its own process.

## 9. T5 as built: the Scraper window (2026-11-01)

### 9.1 The form chosen

The window composes `scrape`, so it needs the command's own choice, plan and run as data rather than its printed lines.
Three forms were weighed, as the code-design section asks. Driving a DianaSh shell from the window and parsing what
`scrape` prints was refused: the tool library would need the shell's binding, a run past the line's budget turns into a
background job, and progress would be read back out of sentences. Copying the choice into C# was refused outright, being
a second owner of it. The form taken is an **extension** of two existing surfaces: the module crate gains
`emusen_scrape_module::ask`, the command's subcommands as JSON in and JSON out over the same `Backend` a shell's call
uses, and the platform library exports it as **`emusen_scraping_command`** (platform interface **29**). The request is
`{"args": [...], "paths": [...], "sources": bool}`, `args` the words after `scrape`: `ls` answers the chosen games with
each kind's source, `plan` the scraper's `ScrapePlan` with the shell's sentence, `run` a started run and the serial to
follow it from, `progress` the scraper's progress with the shell's end sentence once it is not running, and `status`,
`wait`, `cancel` and `collections` what their names say. Failures carry `emusen_scrape.h`'s statuses and words. Galaxia's
`ScrapeCommand` is the C# binding; nothing of the scraper or its service changed.

`--missing` was extended so that ES-DE's four criteria are the module's: a comma list of kinds keeps a game with none
of them (ES-DE's "No game image" is `miximage,screenshot,titlescreen,cover`), and `--missing metadata` a game whose
description, edited or scraped, is empty, read through the scraper's `metadata` read. A single kind and `any` mean what
they meant.

### 9.2 One owner of the choice

Mistress's wide runs from the native scraper — Scrape Games from Preferences, the pad menu and a shelf — are now chosen by
`scrape ls` through `ChosenGames`: the shelf becomes `--system <its ES-DE name>`, favourites `--favorites`, and "No
cover", "No game image", "No game video" and "No metadata" the `--missing` forms above. Both the confirm step's
`ScrapePlan` and the run take that list. Two paths keep `ScopeGames`: one game named by hand, which is no choice, and the
managed scraper, which runs only where the platform library is not in use and so has no module to ask; it goes when the
managed half does. A shelf's run chooses its ES-DE system's games, which is the scraper's own unit.

### 9.3 The tool library and its host

- **`EmuSen.Tools.Scraper`** references LunaP and Galaxia only (`LeafAssemblyTests`): `ScraperPane` (a
  `ManagerLayout<ScrapeRow>`), `ScrapeRow` (one game of `ls`'s answer), and `ScraperPlan`, the words and command lines,
  free of any control.
- **Mistress's hosts share one base**, `ToolPaneWindow`: the form taken from the sheet's framing, the frame's generated
  hints, and the pad's buttons offered to the pane in ES-DE's grammar. `PackagesWindow` moved onto it; `ScraperWindow`
  is the second. Neither writes a hint or a focus route.
- **Placement** (§1b): Home is not built yet, so the existing entry points open the Scraper — *Settings → Scraper…* on
  the desktop, and the pad menu's Library page, whose *Scrape Games…* row opened Preferences' Scraping tab before. A
  game's own *Scrape This Game* is one game and stays where it is.
- **Retired as §2 says**: while the Scraper is open, the status sheet is not opened; a request for it brings the Scraper
  forward, whose apply bar carries the run's progress. Preferences' Scraping tab keeps its controls (the question in
  §9.6).
- **ToolWords and ToolEvents** are the library tool's job events and refusals; the scraper's answers are neither, so
  nothing of them is shared and nothing was moved.

### 9.4 What the window does

- **Sections**: All Games, Favourites, Missing Media (`--missing any`), No Game Image, No Description, each system the
  listing found, and each collection. A section is one choice of the module's, so changing it asks `scrape ls` again;
  the window filters nothing itself beyond the frame's search.
- **Columns**: Game, System, Media (*n of m* of the kinds asked), Missing, Last Run; the big screen shows Game and Media.
- **Options**, as checkable actions: *Theme* (`--kinds theme`, the "what my theme shows" preset, answered by
  Mistress's theme loader and refused with its reason where none is lent), *Videos* (off by default), *Overwrite*.
  *Mark All* marks the shown rows (in Missing Media, every game lacking a kind asked); X marks one. There is no
  Refresh: a section or an option lists again, and so does a run's end. Six actions keep the big screen's row on one
  line, which the plan sheet needs (§9.5).
- **Apply** shows the plan first, from `scrape plan` with the marked games named by hand: the scraper's sentence ("2
  game(s), about 7 request(s) …"), the kinds, whether what is there is kept, and that nothing is sent until Apply.
  Confirmed, one `scrape run` starts and the apply bar follows it: *n of m games* and the game being asked, each result
  in Last Run, the end's sentence after. *Stop* cancels. A run started elsewhere (a game's menu, a shell) is followed
  when the window opens.
- **Refusals** are said plainly in the apply bar's line: a store another program holds (the scraper's own words, §8.8),
  a run already going, the platform library not in use.
- **The detail**: each kind's source in words (the game's package, ScreenScraper, an ES-DE folder, placed by hand,
  OpenEmu); with no game chosen, the options and the account's limits — threads, a minute, today's requests left, and
  whether a member is signed in; never a name or password.
- **One quota**: inside Mistress the command speaks for Mistress's own scraper, lent as §8.2 says, so a run the window
  starts is counted by the same quota manager; standalone, the store lock refuses a second scraper (§8.8).

### 9.5 Tests and seeded faults, measured 2026-11-01

- `emusen-scrape-module` (9, two new): `--missing` as a list, `any` and `metadata`; the window's `ls` with each kind's
  source and without, a plan naming an excluded game by hand, nothing started before `run`, a run's priority, a refusal,
  `status`, `progress` with its end sentence and a serial past every result, `collections`.
- `ScraperWindowTests` (7): through Mistress's own scraper lent to the command and the fake ScreenScraper over HTTPS,
  the library's game listed with its four kinds, marked, planned with nothing sent, run and found, the request counted
  by Mistress's quota, and the status sheet not opened while the Scraper is; with a scripted command, each section's
  words, the plan and run naming the marked games by hand with Overwrite, the run followed to its end sentence, a held
  store said plainly with nothing listed, a run started elsewhere followed and stopped, and the desktop window clean
  under the fit audit at 1280×800 and 1920×1200. `LeafAssemblyTests` holds the library to LunaP and Galaxia.
- `WindowFitAuditTests`: `Scraper`, `ScraperDetail`, `ScraperPlan` and `ScraperRunning` framed over a running game at
  both sizes, audited and recorded (`EmuSen_InterfaceSpec.md` §6.3.14); Packages' eight cases unchanged on the shared
  host base.
- Changed for the new entry points: the pad menu's *Scrape Games…* opens the Scraper (`ScrapeWindowTests`,
  `ScrapeStatusWindowTests`), and the sign-in sheet test opens Preferences' Scraping tab as Settings does
  (`ScrapeSignInTests`). The golden corpus holds `man scrape` and the new `--missing` refusal.
- Found by these tests and fixed: a `since` past every result read through a double and rounded past the scraper's
  range, so the window never followed a run going (the big-screen status test); a quota field the scraper leaves null
  read as a number (the sign-in test); the plan's words 38 units high and the progress line cut (the framed audit,
  LunaP §203.7 and §6.3.14).

| Seeded fault | Caught by |
|---|---|
| a list after `--missing` keeps a game lacking any one kind | `missing_names_a_list_of_kinds_none_present_or_a_description` |
| `plan` and `run` as data drop the games named by hand | `a_window_asks_the_same_choice_plan_and_run_as_data` |
| the Favourites section asks for every game | `Each_section_asks_the_command_for_its_choice_rather_than_filtering_here` |
| the status sheet opens over an open Scraper | `Mistress_s_scraper_lists_plans_…`, `The_pad_menu_and_the_scraping_tab_reopen_the_window_during_a_run` |
| the frame's plan floor at 0 | LunaP's `A_host_sized_to_its_content_…(Desktop)`, `ScraperPlan` at both sizes |

### 9.6 Left

- Region, language and threads are not options of the window; the scraper's settings in Preferences hold them.
- Listing a large library looks up every kind's source of every game (as `scrape ls` does) and rescans the ROM folder
  for each section; it runs off the interface thread. The plan is made on the interface thread when Apply is pressed.
- Preferences' Scraping tab still offers its own scope and *Scrape…*: §2 gathers the choices into the tool but does not
  say to remove them there, so that is put to the tester.


### 9.7 The kinds asked: every picture a theme can draw (decided 2026-11-02)

Decided 2026-11-02 by the tester: the Scraper window's Theme switch goes, "if a user changes themes they may end up
needing to scrape again". The scraper does not follow the active theme; with no `--kinds` a run asks for every
picture a theme can draw (cover, screenshot, marquee, titlescreen, miximage, backcover, 3dbox, physicalmedia, fanart,
the module's `DEFAULT_KINDS`, widened from cover, screenshot, marquee and miximage), so that changing themes never
needs a second scrape. Videos stay a switch the player turns on, and Overwrite stays. `--kinds theme` remains the
command's for whoever names it (`man scrape`). Mistress's Preferences follow the same decision: their nine picture
switches are on by default, and a home that stored the old default of four (every save wrote it) is upgraded once to
the nine (`ScrapeKindsUpgraded`, in Galaxia's settings model and `AppSettings.Upgraded` alike), while a list the player
changed is kept. Tests: Galaxia's `the_old_four_kinds_become_the_nine_and_a_list_the_player_changed_is_kept`, and
`ScraperWindowTests`' fresh home, old-four home and changed home. The cost to the day's quota of nine kinds against
four is for the tester to weigh.

### 9.8 Three defects the tester found on the handheld (2026-11-02)

Reported on build b924305f6:

1. **Marking moved the highlight to the search box.** A mark redraws the rows, which built their containers afresh,
   and the focused one went with them. The frame now gives the focus back to the chosen row for every tool (LunaP
   §211.6). Caught first by `ManagerLayoutTests.Marking_a_row_keeps_the_focus_on_it` (both forms), and held in the
   Scraper by the pad in `ScraperWindowTests`' handheld cases.
2. **The scraper did not work, and said nothing in the error log.** Reproduced on 2026-11-02 in the harness with a
   home laid out as the handheld's: the player's `screenscraper.json` in `home/etc/EmuSen`, EmuSen's developer file
   outside the home in the account's config directory, fake files of the test's own and the fake server, on the Rust
   runtime. The candidate that the build reads the developer file only inside its home is **refuted**: Galaxia's
   legacy config path is the account's config directory whenever the config directory is not redirected, the scraper in
   Rust reads it (`credentials::developer`), and that layout scrapes (`A_home_laid_out_as_the_handheld_s_scrapes_with
   _the_developer_file_outside_the_home`). With no developer file and a build that carries none, the run stops at
   once with "this build carries no developer credentials and EmuSen's developer file is not on this computer": said
   in the apply bar, but written nowhere, which matches the handheld's log having no scraper line. **Every scraper
   failure is now written to the error log as well as said** (`ScraperPane.Failed`): a refused command, a refused
   listing, each game a run failed or left to retry with the scraper's detail, and a run the scraper stopped by
   itself rather than at the player's Stop. `A_scrape_that_cannot_run_is_said_on_the_screen_and_in_the_error_log`
   failed before the change. The handheld's own cause is for its next log to say; the developer file's place there,
   and whether the session's `XDG_CONFIG_HOME` moves the account's config directory, are the first things to read.
3. **Three switches no one could read.** The plan's big-screen rows drew no names at all, only `Off` (LunaP §211.7),
   and the switches were Theme, Videos and Overwrite. Each row now shows its name and a sentence: Videos, "Also
   fetches each game's video. Videos are large, so this stays off unless you turn it on."; Overwrite, "Fetches again
   and replaces the pictures and details you already have; off fetches only what is missing." Theme went (§9.7).

Seeded on 2026-11-02, each caught: no focus given back after a mark; no sentence under a switch; a run the scraper
stopped by itself not logged. The Scraper's plan state was re-recorded for the rows' names and sentences after its
picture was checked.

### 9.9 The consoles by their names, as Cores names them (2026-11-03)

The strip and the System column showed the library's system ids (`gb`, `gbc`, `genesis`, `ps2`), where Cores shows the
systems table's `display_name` (`EmuSen_Cores.md` §3.8) and the Scraper's mockup shows `Super Nintendo`. The pane is
now handed the runtime's `systems` listing by its host (`ScraperPane(Func<string>? systems)`, Mistress passing
`CrystalRuntime.Systems`), the tool keeping no table of its own, and names a system by the listing's `display_name`,
found by its `esde_system` or its id; a system the listing does not name keeps its id, and so does every system of a
host that hands nothing in. The strip's sections are ordered by that name; each still asks `scrape ls --system <id>`,
so the module's choice is unchanged. The column is *Console*, the pane's line `Console: Super Nintendo`, and the help
bar's section word *Console*, as the mockup's. The strip keeps its order by name rather than Cores' order by maker and
release, which lives in Cores alone; one order for both waits until the frame owns it.

`ScraperWindowTests.The_strip_the_column_and_the_pane_name_the_consoles_as_Cores_does` opens the Scraper from
Mistress over its Rust runtime and holds the strip, the column's cells, the pane and the section's request. The
Scraper's eight audited states were re-recorded on 2026-11-03 for this one cause (the strip's, the pane's and the help
bar's words), after the ScraperDetail picture was held against the mockup at 1280 × 800 and 1920 × 1080.

### 9.10 Why a run is slow: the scrape log and a measured run (2026-11-03)

**The symptom.** On the handheld on 2026-11-03 a run over the new PS2 games, the nine default kinds (§9.7), managed
about 4 games a minute over 20 minutes and about 1 a minute in the last 5: 570 pictures for about 76 games, with
EmuSen under 10% CPU, the account at ScreenScraper's highest membership and `ScrapeThreads` 0 (so Mistress asks for
the account's `maxthreads`). Nothing logged the time of a request, so the cause was unproven.

**The scrape log.** Every run now writes `scrape_<yyyyMMdd>.log` into the error log's folder (Galaxia's
`error_log::root`: the `LogDirectory` setting, else its fallback), one line a ScreenScraper request and one when the
run's workers end (`Platform/scrape/src/runlog.rs`). Lines are tab-separated `name=value` fields after the local time:

    <time>  request  game=<file>  kind=info|cover|...  status=<HTTP, 0 for none>  outcome=<answer or media outcome>
            bytes=  ms=<send to last byte>  worker=<index>  threads=<live>/<allowed>  pace_ms=  cap_ms=  hash_ms=
            maxthreads=  maxrequestspermin=  maxdownloadspeed=  requeststoday=
    <time>  run  end=done|stopped|cancelled  games=  requests=  bytes=  wall_ms=  workers=  worker_ms=  pace_ms=
            cap_ms=  server_ms=  hash_ms=  idle_ms=  own_ms=

`pace_ms` is the time the worker waited for the per-minute pace before this request, `cap_ms` the time it was held
after its previous file for `maxdownloadspeed`, `hash_ms` the time spent hashing this game's file before its lookup;
the quota fields are the answer's own (a media file carries none, `-`). The run line sums the workers' time:
`own_ms` is what is left of it after the waits, the requests, the hashing and `idle_ms` (no game due, or paused), that
is the scraper's own work between requests. Every line passes the redactor, and none holds an address, a key or the
account's name. The file is a day's, and is kept as the error log's are: once in a process's life, the first run removes
the `scrape_*.log` files last written more than fourteen days before (`runlog::prune`, Galaxia's `KEEP_DAYS`); a full
run of a day's 20,000 requests writes about 3 MB.

`ScrapeLogTests` (WiseMan, over the fake ScreenScraper): `Each_request_is_one_line_with_its_kind_status_timing_worker
_and_quota_and_the_run_one_summary` holds a line for every request the server answered (seven for three games), the
fields, the quota of a lookup and the one run line; `Nothing_the_scrape_log_holds_is_a_credential_or_an_address`
holds that no developer id or password, member name or password, credential parameter or URL is written. Both failed
with the log switched off. Seeded on 2026-11-03, each caught: the request's URL written unredacted with its
`devpassword` (the second test); the media lines left out (the count); a lookup's quota left out (the quota
assertion); the run line left out (the single run line). Two unit tests in `runlog.rs` hold the line's format and
the lane's hand-over of its waits.

**The measured run.** On 2026-11-03 on the desktop, through the platform library's own `scrape run --system ps2
--threads 99` (as Mistress's `ThreadsWanted` asks), in a throwaway home with the desktop's member account: five PS2
discs of 0.88 to 2.17 GB, copied from the NAS to local NVMe, no art yet, the nine default kinds. 50 requests (5
lookups, 45 pictures), all found, 22.1 MB, **41.6 s wall**, about 7 games a minute. The account's answers said
`maxthreads=6`, `maxrequestspermin=7168`, `maxdownloadspeed=2176` (KB/s). Where the workers' 149.4 s went:

| bucket | s | share | what it is |
|---|---:|---:|---|
| server | 65.2 | 44% | the requests themselves |
| idle | 57.0 | 38% | workers with no game due (six workers, five games) |
| hash | 26.9 | 18% | MD5, CRC32 and SHA-1 of each disc, before its lookup |
| pace | 0.01 | 0% | the per-minute pace, 9 ms apart at 7168 a minute |
| cap | 0 | 0% | the download cap: no file came faster than 2176 KB/s |
| own | 0.3 | 0% | the scraper's own work between requests |

Read off the lines:

1. **The pace and the cap cost nothing** at this membership: 12 ms of pace in the whole run, no cap wait.
2. **A request is about 0.9 s whatever its size.** The 45 pictures averaged 485 KB and 1.10 s; fitted, ms = 875 +
   465 per MB, so 80% of a picture's time is the exchange's fixed cost and the transfer runs at about 2.1 MB/s a
   stream (near `maxdownloadspeed`, which the server appears to apply itself). Lookups took 1.3 to 4.5 s. Smaller
   images would save at most the 20% that is transfer.
3. **A game's pictures are fetched one after another** by its worker: nine pictures, about 10 s, plus the lookup.
   The six threads work six games at once, so a worker's game takes hash + lookup + about 10 s.
4. **The first game runs alone.** Workers are added after a game ends (`ensure_workers`), and until the first
   answer says `maxthreads` the quota allows one: the first 20.1 s of the 41.6 s ran on one thread
   (`threads=1/6`).
5. **Hashing reads the whole disc.** 2.8 to 6.9 s a disc, 313 to 322 MB/s, one core, from the page cache; four discs
   hashed at once kept that rate each. A disc's MD5 is cached by size and time, but its CRC32 and SHA-1 are not, so
   a later run that asks again hashes every disc again (`process_inner`: the cached MD5 leaves `hashes` empty and
   the lookup hashes the file).

**The cause on the handheld, predicted from these numbers.** The server side caps a worker at a game every 11 to
15 s, so six workers could do 24 to 33 games a minute; the handheld's 4 a minute means about 90 s a game a worker,
and 75 s of that is not the server, the pace or the cap. The handheld's PS2 discs average about 3.5 GB (the NAS's
57 run 0.9 to 7.9 GB), and six workers hash six of them at once from one storage device. At the desktop's 315 MB/s a
disc would take 11 s and the CPU would show several busy cores; under 10% CPU over 20 minutes says the hashing waits
on the storage instead, about 180 MB/s shared by six readers (76 games of about 3.5 GB in about 25 minutes). The
slowdown at the end fits the larger discs or more readers on a slower card. **Prediction 1**: the handheld's log
shows `hash_ms` of 40 s or more for a 4 GB disc and `server_ms` under a fifth of the workers' time. **Prediction 2**:
`pace_ms` and `cap_ms` stay near zero there as here. Both are for the handheld's next run's log; the desktop
cannot show the handheld's storage.

**The fix the numbers support (proposed, not built).** First, measure the handheld (the two predictions). If the
hashing is the cost: (a) keep a file's CRC32 and SHA-1 with its MD5 in the store, so that no disc is hashed twice
across runs (moves `hash_ms` of every later run to 0; for the 76 games about 75 s a game a worker); (b) for a disc
image above a size threshold, look it up by its name and size before hashing it, and hash only when that misses: the
gain is the whole of `hash_ms` for the games found that way, at the cost of a second lookup for the rest, and its
matching must first be measured against the hashed lookup on the same games before it can be the default; (c) hash
on one reader at a time per storage device while the other workers fetch pictures, if the handheld's log shows six
concurrent readers slower than one. Second, wherever the cost lies: add the workers as soon as the first answer
gives `maxthreads` rather than after the first game ends (saves one game's time a run, 20 s here). Fetching a game's
pictures in parallel would not help with six games in flight, since `maxthreads` bounds the requests in flight.

**A two-pass order** (cover, screenshot, marquee and mix image for every game first, then the rest) does not make a
run faster on these numbers: the same 50 requests are made, each at the same fixed cost, and the second pass needs
each game's answer again, a second lookup a game (10% more requests) and, with the CRC32 and SHA-1 uncached, a second
hash of every disc, the cost that dominates. It would bring the first pictures sooner; it is worth it only after (a),
and only with the first answer's media kept for the second pass.

### 9.11 A file hashed once, and the workers started with the first answer (2026-11-03)

Items (a) and the early workers of §9.10, built on 2026-11-03; the name-and-size lookup (b) waits for proof that it
matches the same games as the hashed lookup.

- **The three hashes are kept.** `media.db` schema 4 adds `crc32` and `sha1` to `scrape_file`, beside the MD5, keyed
  as before by the file's path, size and time (`Store::cached_file`, `Store::remember_file`). A file is read once,
  for all three digests in one pass (`hash_once`), and a later lookup of it at the same size and time reads nothing;
  a file whose size or time changed misses and is read again. A row from schema 3 has no CRC32 or SHA-1, so its file
  is read once more and the row completed. The C# `MediaStore` takes the same migration and the C# `Scraper` the same
  rule (`CachedFile`, `HashOnce`), so the two write the same rows (`ScrapeReplayParityTests`); a row written with an
  MD5 alone has its other two cleared, so a changed file is never sent with old digests.
- **The workers start with the first answer.** `ask` adds the workers as soon as an answer carries the account's
  limits (`quota.observe`), rather than after the first game ends.

Measured on 2026-11-03 on the desktop, the same five discs, kinds and command as §9.10, in a fresh home (50 requests,
all found):

| | before (§9.10) | after |
|---|---:|---:|
| wall | 41.6 s | **26.6 s** (−36%) |
| hash, all discs | 26.9 s | 27.0 s |
| server | 65.2 s | 61.7 s |
| idle | 57.0 s | 34.2 s |
| first game alone | 20.1 s | 0 (the other four began at the first answer) |

The hashing is unchanged in a first run, as it must be: each disc is read once in both. The cache's gain is in every
later run over the same files (a refresh, a resume, a game asked again), where `hash_ms` is `-` instead of the 2.8 to
6.9 s a disc here, and on the handheld the predicted 40 s or more a 4 GB disc.

Tests, each failing first with its change switched off: `ScrapeLogTests.A_file_asked_again_is_not_hashed_again`
(the second lookup of a game writes `hash_ms=-`) and `The_other_workers_start_with_the_first_answer` (the first
picture's line says `threads=3/3` where the account allows three); `tests/file_hashes.rs` holds the three hashes kept
at a size and time and a changed file missing. Seeded on 2026-11-03, each caught by `file_hashes.rs`: the CRC32
stored as the SHA-1; the cache keyed without the size. `ScrapeStoreParityTests` holds the two stores' migration
alike at every version, its seed's `scrape_file` row now naming its columns; the two tests that build an older file
from a new one (`ScrapeExtrasTests`, `MediaStoreFoldersTests`) drop schema 4's columns as well.

**The name-and-size lookup (b) is dropped.** Measured on 2026-11-03 on the same five discs (17 requests): a
`jeuInfos` with no hashes, by the file's name and size, named the same game and rom as the hashed lookup for all five.
But the answer's own rom record carries the file's MD5, CRC32 and SHA-1 for two of them only (Ape Escape 2, ATV
Offroad Fury); for Dark Cloud and Armored Core 3 the size agrees and no rom ScreenScraper lists has the file's MD5, and
for Ape Escape 3 not even the size agrees. So three of the five "hash" lookups were name matches on ScreenScraper's
side, and the agreement says nothing about a disc whose name is not its dump's. Nor can the answer's digests stand in
for the file's in `media.db`, which keys every record by the file's own MD5; a stand-in key would change what that
column means. The hash cache and the early workers stand, and the handheld's scrape log decides what comes next.

**Hashing several discs at once**, measured the same day on the desktop's NVMe with each file's pages dropped first
(`posix_fadvise(POSIX_FADV_DONTNEED)` through `dd iflag=nocache`, checked with `fincore`): the five discs, 8.49 GB,
took 31.5 s and 29.4 s one at a time (270 to 289 MB/s, one core's hashing) and 8.6 s and 10.9 s all at once (777 to
986 MB/s). On this device concurrency helps 2.7 to 3.7 times; whether a cap per device is wanted waits for the
handheld's log, whose storage this does not measure.

### 9.12 A big disc found by its name and size, its checksums taken later (2026-11-04)

Decided by the tester on 2026-11-04 after the handheld's own scrape log proved the cause: one run of
`Clock Tower 3 (USA).iso` took 81,654 ms of wall time, of which `hash_ms` 74,720 (the whole disc read for its three
digests at about 50 MB/s from the handheld's storage) and `server_ms` 6,805 (the lookup and six pictures). Reading the
disc, not ScreenScraper, was 92% of the game's time. §9.11 had measured that a lookup by name and size names the same
game as the hashed one for the five desktop discs, and had dropped it only because `media.db` keys every record by the
file's MD5. The key now has a second form.

**What changed.** For a file above 1 GB (1,000,000,000 bytes, `fingerprint::ABOVE`); a file of 1 GB or less is
hashed in full exactly as before:

1. **A fingerprint.** The SHA-1 of the file's first and last MiB, read in one pass of 2 MiB (`fingerprint::of`). With
   the size it keys the game. It lives in its own column, `fingerprint`, never in `md5`.
2. **Name and size first.** `jeuInfos` is sent with `romnom` and `romtaille` and no `crc`, `md5` or `sha1`
   (`Hashes::name_and_size`). Found, the record is kept under the fingerprint, `matched_by` is `name+size`, and the
   path index's MD5 is empty and its CRC32 and SHA-1 null.
3. **The hash lookup after a miss only.** A miss reads the file in full (`hash_once`), keeps the three digests beside
   the fingerprint, and asks by them as before; found, `matched_by` is `file`. The record keeps the fingerprint key
   either way, so a file is keyed once, when first seen, by its size.
4. **The checksums later.** When a run's last worker ends, a thread (`Scraper::hash_later`) takes the digests of every
   fingerprinted file whose path index lacks them, at the lowest priority the system has (Linux: nice 19 and the idle
   I/O class through `ioprio_set`; macOS: `PRIO_DARWIN_BG`; Windows: `THREAD_MODE_BACKGROUND_BEGIN`). It runs only
   while no game is on screen: the host says so with the change `playing {running}` (Mistress at `StartEmulationThread`,
   `StopEmulationThread` and when the scraper opens), the thread waits while a game runs, and the hashing asks every
   64 KiB and abandons the file the moment one starts, to begin it again afterwards. A file whose size or time changed
   is skipped. Nothing waits on the thread; the handle's close stops and joins it. The digests land in the path index
   (`Store::fill_hashes`), where a file's CRC32 and SHA-1 already lived; the game's rows keep their key and do not move.
5. **One owner of "which key".** `store::Key` holds the rule: a file with a fingerprint is found by it, one without by
   its MD5 (`Key::of_file`). The game's rows carry the key in `md5` and `fingerprint` (`md5` empty under a
   fingerprint), and every lookup that went by MD5 goes by the key: the path index joined to its game (`store::joined`,
   behind `found`, `record`, `outcomes`, `forget`, `forget_unfound`, `metadata`), the plan, a run's media, a rename's
   copies (`paths_of`) and the layout step. The library, Home, the game page and the Scraper's rows all read through
   those. The reads `game` and `stored` take `fingerprint` beside `md5`, and a record's JSON carries `fingerprint`.
6. **The scrape log.** Each lookup's line gains `match=name+size|hash` and every request line `fingerprint_ms`, owed
   to the next line as `hash_ms` is; the run line gains `fingerprint_ms` and `read_bytes`, the bytes of game files
   read to identify them.

**Schema 5.** `media.db` schema 5 rebuilds `scrape_game` and `scrape_media` with `fingerprint TEXT NOT NULL DEFAULT ''`
in their primary keys (`(md5, bytes, fingerprint)`, and `type` for media), copying every row, and adds the same
column to `scrape_file`. Two discs of one size keep their own records. A record from schema 4 keeps its MD5 key: a big
file already in the path index with its MD5 is not fingerprinted again, so nothing is re-scraped. The C# `MediaStore`
takes the same migration and the same join while it exists (B10 retires it): its `RememberFile` keys a file read in
full by its MD5, as the Rust `remember_file` does, and its `CachedFile` passes over a fingerprinted row with no MD5 yet.

**What it cannot tell apart.** Two files of one size whose first and last MiB agree share a fingerprint, so a patched
image of a disc that keeps its size and both ends would share the original's record. ScreenScraper's name-and-size
lookup has the same blind spot (§9.11: three of its five "hash" matches were name matches on its side). A file read in
full anew (its size or time changed) is keyed by its fingerprint again only when it is above 1 GB.

**Measured on 2026-11-04** on the desktop, the same five discs, kinds and member account as §9.10 and §9.11, the
discs' pages warm as they were then, through the scraper's own run (`Scraper::start` with 99 threads asked):

| | before (§9.11) | after |
|---|---:|---:|
| wall | 26.6 s | **15.9 s** (−40%) |
| hash, all discs | 27.0 s | **2.7 s** (ATV Offroad Fury alone, 0.88 GB, below the threshold) |
| fingerprints, four discs | - | 5 ms (about 1 ms each) |
| read to identify the discs | 8.49 GB | 0.89 GB |
| server | 61.7 s | 60.2 s |
| requests | 50, all found | 50, all found |

The four discs above 1 GB were each found by their first lookup, `match=name+size`, with `hash_ms=-`; none fell back.
ATV Offroad Fury, at 0.88 GB, was hashed and asked by its digests as before (`match=hash`). The run is now bound by
ScreenScraper: the lookup and nine pictures of the slowest game. On the handheld the prediction for its logged run is
that the 74.7 s of `hash_ms` for Clock Tower 3 becomes about 0.04 s (2 MiB at its 50 MB/s), so that game's 81.7 s
should fall to about 7 s, the server's share; main checks it there. The checksums of the four big discs were then owed
to the background thread, which the driver's exit stopped and joined; they are taken in the next session that sits
with no game on screen.

**Tests**, on the fake server and synthetic files only. `ScrapeFingerprintTests` (WiseMan; the test lowers the
threshold to 32 MiB through the scraper's `test.fingerprint_above` seam, over sparse 64 MiB files):
`A_big_file_is_found_by_name_and_size_with_two_mib_read` (no digest sent, `read_bytes=2097152`, the record empty of
MD5 under its fingerprint, its cover found by the key), `A_big_file_not_found_by_name_falls_back_to_its_hashes` (two
lookups, `name+size` then `hash`, 2 MiB plus the whole file read, the path index holding MD5 and fingerprint),
`The_checksums_arrive_later_and_never_while_a_game_is_on_screen` (nothing for 1.5 s while marked playing, then the
three digests, the record's key unchanged) and `A_big_file_s_art_is_found_again_by_its_fingerprint_after_a_restart`
(a new window, no request, `read_bytes=0`, the cover still found). `tests/fingerprint_store.rs` holds the migration
from schema 4 keeping every row, two discs of one size apart, a fingerprinted game found again after the store is
reopened, and the digests arriving without moving it; `fingerprint.rs` the 1 GB boundary and the 2 MiB read;
`ScrapeStoreParityTests` the two stores' schema 5 at every version and the fingerprinted rows read alike through both.
Seeded on 2026-11-04, each caught: a full read before the name-and-size lookup (three `ScrapeFingerprintTests`, by
`read_bytes`); the fingerprint written into the `md5` column (`fingerprint_store.rs`); the background hashing
ignoring a game on screen, its wait and its per-chunk check both removed (`The_checksums_arrive_later...`); the join
going by MD5 alone (`fingerprint_store.rs`, the two discs of one size); the migration dropping `kinds_known`
(`fingerprint_store.rs`). Removing only the wait was not caught, rightly: the per-chunk check still abandoned every
file while the game ran, so no digest arrived during it.

## 10. T6 as built: the Library Manager window (2026-11-01)

### 10.1 The tool library and its host

- **`EmuSen.Tools.Library`** references LunaP and Galaxia only (`LeafAssemblyTests`): `LibraryPane` (a
  `ManagerLayout<LibraryRow>`), `LibraryRow` (one finding of one file), and `LibraryFindings`, which turns the library
  tool's events and the host's records into rows, marks, the plan and the saved list, free of any control.
- **One owner of the job's words.** `ToolWords` (the library tool's refusals and reasons in a player's words) and
  `ToolEvents` (how an event's field is read) moved from `EmuSen.Tools.Packages` to Galaxia's `EmuSen.Galaxia.Native`,
  beside `LibraryToolJob`, since two tools now read the same jobs; Packages and Mistress's `CompressedDiscsWords` read
  them there and nothing else changed.
- **`LibraryWindow`** is the third host on `ToolPaneWindow` (§9.3), and writes no hint or focus route.
- **Placement** (§1b): *Settings → Library Manager…* on the desktop, and *Library Manager* on the pad menu's Library
  page, beside Packages; the page holds at most six rows (Start Console, Collections, Scrape Games, Packages, Library
  Manager, Theme Settings), inside the nine the fit audit checks. The menus say *Library Manager* rather than §6's
  *Library*, since the pad menu's page is already *Library* and the desktop's View menu has *Game Library*; the
  question is put to the tester (§10.6).

### 10.2 Where each finding comes from

- **The same dump**: `rom dups` through the library tool's job (§7.4). Members a package claims (the `package` of a
  group's member) are one game: they are counted once, and a group left with fewer than two games is not shown.
- **The same game in another dump**: ScreenScraper's game id, for scraped files only, read from the host's records.
  Two files are one dump when one package claims both, else when `rom dups` grouped them, else when ScreenScraper was
  asked by the same MD5 and size; a game id is shown only where its files are two or more dumps, so a headered and a
  plain copy, whose raw MD5s differ, are not shown again here. Files outside the library, or no longer there, are left
  out. Galaxia's claim index (`GamePackages.Index`) names a file's package.
- **Broken or suspect**: `rom check -r` over the whole library (§7.3: size, sheet, archive, CHD), each failed check by
  name and words. The header checksums are T3's: `LibraryFindings.CheckName` already names a `header` check, and the
  detail says "Header checksum: not checked yet", so T3 adds a check to `rom check` and the window shows it unchanged.
- **Unrecognised**: a file ScreenScraper answered `Unknown`, shown as a hint ("a hint, not a fault"), never as broken.
- **The records** are an interface the tool defines, `ILibraryRecords` (the hidden games, hide or show one, the
  scraped files), which a host lends. Mistress lends games.db's `hidden` edit (`GameMetadata.Hidden`, the field its
  game menu's *Hide* already sets) and the scraper's answers (`OutcomesByPath`, `FoundByPath`). Without a lend the
  window lists the two `rom` findings and says that hiding and ScreenScraper's findings need games.db. Two other forms
  were weighed: reaching games.db and media.db from the tool library would have needed Crystal.Client or a second
  binding of `emusen_library.h`, breaking the leaf rule; a `scrape`/`games` command answering ids and the hidden flag
  as data would serve the standalone launcher too, and is put to the tester (§10.6).

### 10.3 What the window does

- **Reading**: on opening, `rom check -r ""` and then `rom dups`, followed in the apply bar's progress line ("Checking
  each file: n of m", "Comparing dumps: n of m"); *Stop* asks the job to stop, the findings read so far stay, no dump
  group is given for a stopped comparison (§7.4), and the window says so. *Check Again* (Select) reads again.
- **Sections**: All, Same Dump, Same Game, Broken or Suspect, Unrecognised, Hidden. **Columns**: Game, Console (the
  system's id as the tool gives it), Finding, Group (*Dump n*, *Game n*), Size, State; the big screen shows Game and
  Finding. A file found twice (a duplicate that is also broken) has a row for each finding.
- **Hide**: a row is offered *hide*, or *show again* where it is hidden; X or the *Hide* action marks it, Apply shows
  the plan by file name, ending "Only games.db changes: no file in the library is written, moved or removed", and only
  confirming writes the edit. Marks on two rows of one file are one change.
- **Save List…** writes every finding as text: a heading per finding, a paragraph per group, one path per line with
  the ScreenScraper id, the package or the failed checks after a tab, so the list greps and pipes as `rom dups` does.
  The desktop asks where with the platform's save picker; the big screen, which has no picker a pad drives, writes
  `Library <date time>.txt` under EmuSen's Logs folder and says where. A path inside the library is refused.
- **Show in File Manager** opens the folder holding the chosen file through Avalonia's launcher (the folder, not the
  file, which the system would open in an emulator).
- **Nothing writes the library.** No Move to Trash (T7). The only writes are games.db's edit and the saved list.

### 10.4 Tests and seeded faults, measured 2026-11-01

- `LibraryWindowTests` (7): through real jobs of the platform library on a synthetic library (a SNES cartridge plain
  and with a copier header, an empty cartridge, two NES dumps of one game, a cue sheet naming a missing track) with
  scripted records, the same dump found by `rom dups`, the same game by ScreenScraper's id with the copier pair not
  shown again, both broken files by their checks, the unrecognised sheet as a hint, a file outside the library and one
  gone left out; a row hidden by X and Start and shown again from the Hidden section, only after the plan; the list saved
  outside the library and refused inside it; and the library byte for byte as it was. `LibraryFindings` alone: a
  package and its files one game in both tiers. With scripted jobs: no marks without records and nothing read without
  the platform library; a stopped read keeping its findings and comparing nothing; the pad marking, planning, opening
  the detail, stepping sections and reading again, and *Show in File Manager* opening the file's folder; the desktop
  window clean under the fit audit at 1280×800 and 1920×1200. `LeafAssemblyTests` holds the library to LunaP and
  Galaxia; `PadMenuTests` the Library page's rows and its *Library Manager* row opening the window.
- `WindowFitAuditTests`: `Library`, `LibraryDetail`, `LibraryPlan` and `LibraryRunning` framed over a running game at
  both sizes, clean at the first audit, and recorded (`EmuSen_InterfaceSpec.md` §6.3.15).
- Found by these tests and fixed: `rom dups` reports a file it cannot read as `failed`, which the window first took as
  a second *broken* row of the same file (the real-jobs test). Also corrected here, carried from §9: `PadMenuTests`
  still expected the pad menu's *Scrape Games…* to open Preferences.

| Seeded fault | Caught by |
|---|---|
| a package's files counted apart in a dump group | `A_package_and_the_files_it_claims_are_one_game_in_both_tiers` |
| the same game ignoring `rom dups`' groups, so a headered pair shows in both tiers | `Library_finds_both_tiers_…` |
| *hide* offered with no records lent | `Without_records_nothing_is_offered_…` |
| *show again* written as *hide* | `Library_finds_both_tiers_…` |
| the list saved inside the library | `Library_finds_both_tiers_…` |
| a comparison's unreadable file taken as broken | `Library_finds_both_tiers_…` |

### 10.5 Left

- The two reads each take every byte of the library (§7.6); a check and a comparison in one pass of the tool would
  halve it, if a real library proves slow.
- The Console column shows the system's id (`snes`, `psx`), since the `rom` events carry no console name.

### 10.6 Questions for the tester

- The menus' name: *Library Manager*, or §6's *Library* (which repeats the pad menu's page).
- A records surface for the standalone launcher: a command answering the scraped ids and the hidden flag as data, so
  `emusen-tools library` can show the second tier and hide without Mistress.
