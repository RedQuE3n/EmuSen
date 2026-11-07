# EmuSen_Layout — EmuSen's folder, plain on disk and Unix in the shell

*Proposal, 2026-10-27; its questions answered by the tester the same day (§6). Nothing is built.* Raised by the tester the same day: EmuSen's own folder was laid
out to mimic a Unix tree, and a player opening it in a file manager is left to guess where things are. It absorbs the
to-do's "home → EmuSen" rename and the publish targets' need for the operating system's standard places.

## 1. What a player sees today

A published tree (`hier(7)`, `EmuSen_Galaxia.md` §3): `bin/` and `lib/EmuSen/` beside `home/`, and inside `home`:
`etc/EmuSen/` (every settings file), `Library/` (the databases), `Saves/` with `Saves/Save States/`, `Firmware/`,
`Cheats/`, `Cores/`, `Shaders/`, `Logs/` (dumps, screenshots and recordings together), `Media/` (scraped art and
`media.db`), `Packages/`, `Compressed/`, `Unpacked/`, `Games/` (an empty skeleton), `Documents/`, `tmp/`.

What confuses: lowercase Unix names (`bin`, `lib`, `etc`, `tmp`) beside capitalised ones; the settings two levels down
under `etc`; the program and the player's things side by side; `Packages`, `Compressed` and `Unpacked` with no hint of
which is the player's and which is a cache; screenshots inside `Logs`; and the whole of it in a folder named `home`.

Other emulators' user folders are one level of plain names — RetroArch's `saves`, `states`, `screenshots`, `cheats`,
`config`, `system`; PCSX2's `bios`, `memcards`, `sstates`, `snaps`; Dolphin's `Config`, `StateSaves`, `ScreenShots`.

## 2. The layout

```
EmuSen/
  EmuSen            the launcher (EmuSen.exe on Windows)
  App/              every program file; nothing for a player to change
  Saves/            battery saves and memory cards, a folder per console
  States/           save states, a folder per console
  Screenshots/      screenshots and recordings
  Packages/         game packages, CHD conversions among them
  Firmware/         the player's BIOS files
  Media/            scraped art and videos
  Cheats/  Mods/  Themes/  Shaders/  Cores/
  Settings/         every settings file
  Manual/           the EmuSen Manual
  Logs/             logs and debugger dumps
  Data/             EmuSen's own databases and caches (unpacked archives, scratch): safe to leave alone
```

| Today | Becomes |
|---|---|
| `bin/`, `lib/EmuSen/` | `EmuSen` (launcher), `App/` |
| `home/` | the folder itself (`EmuSen/`) |
| `home/etc/EmuSen/` | `Settings/` |
| `home/Saves/Save States/` | `States/` |
| `home/Logs/` screenshots and recordings | `Screenshots/` |
| `home/Library/`, `Media/media.db` | `Data/` |
| `home/Compressed/` | inside `Packages/` (a converted disc is already a package) |
| `home/Unpacked/`, `home/tmp/` | `Data/Cache/`, `Data/Temp/` |
| `home/Documents/` | `Manual/` |
| `home/Games/` (empty skeleton) | removed: the ROM library is wherever the player keeps it |

- **One owner**: Galaxia locates every directory (`EmuSen_Galaxia.md`), so the layout is one table there, read by
  every frontend, core host and tool; nothing else names a path.
- **The shell keeps its Unix view as a mapping**: in DianaSh `/etc` shows `Settings/`, `/tmp` shows `Data/Temp/`,
  `/Games` the configured ROM library, and `hier(7)` documents the mapping. Unix names live where Unix users are.
- **Names** are capitalised English words, one level deep, the same on every platform.

## 3. Where the folder lives

- **Portable**: everything beside the launcher, as the handheld's install is today, when a file named `portable` sits
  beside it (RetroArch's and PCSX2's convention).
- **Installed** (Flathub, an AppImage, Windows and macOS installs): the operating system's standard places, with the
  same inner layout — on Linux `~/.local/share/EmuSen` for the player's things, `~/.config/EmuSen` for `Settings`,
  `~/.cache/EmuSen` for `Data/Cache` (the XDG base directories); `%APPDATA%\EmuSen` on Windows;
  `~/Library/Application Support/EmuSen` on macOS (as now).

## 4. Moving what is there

- On the first start of a build with the new layout, Galaxia moves each folder to its new place and writes every move to
  `Logs/layout-move.log`.
- **Moved, not copied, within one drive**: a rename, instant and all-or-nothing, so gigabytes of packages and art are
  not doubled. **Copied across drives** (portable to installed), the originals left until the copy is checked.
- A move is never made over something already there: a clash is left in place and named in the log and on screen.
- The operating system's permissions are respected; a folder EmuSen may not move is left and named, never forced (the
  rule for a player's files, `EmuSen_LibraryTools.md` §3.1).
- Settings files written before keep working: the paths inside them are rewritten, the old file kept as `.bak`.

## 5. Stages

| Stage | What | Model |
|---|---|---|
| **L1** | Galaxia's layout table, both modes, the shell's mapping, `hier(7)` | careful: every path passes through it |
| **L2** | The first-start move, its log, clashes, permissions; tests on a synthetic old tree, portable and installed | careful: the player's data |
| **L3** | The publish layout (`App/`, the launcher, the `portable` marker), the handheld's install moved | routine, with the handheld's backup first |

## 6. Decided 2026-10-27 by the tester

1. The layout as in §2.
2. Portable and installed with the same inner layout, portable by a `portable` marker file.
3. Moved within a drive, copied across drives, every step logged.
4. Order: before the AppImage and Flathub work, which need §3; L1 and L2 with the most care after 2026-10-31.
