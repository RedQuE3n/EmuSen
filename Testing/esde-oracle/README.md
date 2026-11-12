# esde-oracle

ES-DE itself, run as a black box, as the oracle the Qt painter of ES-DE themes is compared with (`EmuSen_Qt.md` §4;
the comparison is not here). Each run captures one *state* of one theme at one size, at full resolution, in a
compositor, a home and a library of its own. Nothing here is built or shipped, and nothing touches the player's
settings, ROMs or ES-DE folders.

## The binary

ES-DE 3.4.1, the version every measured rule of `EmuSen_BigPicture.md` came from (§14.7, §16.5, §31–§39). Decided
2026-11-08 by the tester: the official Linux x86_64 AppImage, from ES-DE's own release page.

- file: `~/.cache/emusen/bigpicture/esde/ES-DE_x64.AppImage` (124,555,728 bytes), executed only by `run_esde.sh`, never
  installed
- from: <https://gitlab.com/es-de/emulationstation-de/-/releases/v3.4.1>, the asset `ES-DE_x64.AppImage`,
  <https://gitlab.com/es-de/emulationstation-de/-/package_files/288156961/download> (downloaded 2026-11-08; the
  release is dated 2026-04-10, and the binary logs itself as "ES-DE 3.4.1 (r51), built Apr 5 2026")
- sha256: 3c61a44d738d55163daa58ede70720b425a0df62460c04dc23dd3ca586723581

The line above is the pin: `run_esde.sh` reads it and refuses a binary whose sum differs. The copy downloaded in
September 2026 for the C# painter's work has the same sum. On its first run the rig extracts the AppImage
(`--appimage-extract`) to `~/.cache/emusen/esde-oracle/bin/<sha256>/`, so ES-DE runs as one process tree without FUSE.

## What a run may reach

ES-DE runs inside `bwrap`: the root read-only, the run's home the only place it writes, the ROM library's folders
(`~/Documents`, `/mnt/nas-roms`) and any real ES-DE folder covered by empty ones, and a network namespace of its own,
which holds only an unconfigured loopback, so nothing it starts can reach the network. The settings turn the updater
off (`ApplicationUpdaterFrequency` `never`, and `--no-update-check`) and leave the ScreenScraper account empty;
`settings.py` refuses any other value for those four. A PID namespace of its own means that stopping the sandbox by
PID stops every process in it; after each run `run_esde.sh` checks that no process from `esde-oracle/bin/` is left
(an anchored `ps`, never `pgrep -f`) and that qt-gates' pad is gone.

SDL inside the sandbox sees only the pad of `pad.py` here (vendor 0x1209, product 0x0001, through
`SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT`), finds it by inotify (`SDL_JOYSTICK_DISABLE_UDEV`, since udev's events do
not cross into another network namespace), plays to SDL's dummy audio, and draws through Xwayland.

## The scripts

- `run_esde.sh <theme> system|gamelist[:<j>[:media:<n>]] <W> <H>` — one capture. Checks the sum, writes the library, a fresh home
  under `runs/<theme>-<view>-<W>x<H>/home` (the theme linked in, the gamelists copied, the settings written), then calls
  `../qt-gates/run.sh` with `WIDTH`/`HEIGHT` at W × H, `FULL=1` and one grab a second, and the pad's steps from
  `state.py`. The capture is the last grab taken while ES-DE ran, kept as
  `~/.cache/emusen/esde-oracle/<theme>/<view>-<W>x<H>.png` (with `SYSTEM` other than `snes`, the system's name and a
  hyphen before the view); the grabs are then deleted, and ES-DE's own log is kept
  beside qt-gates' in the run folder. `VARIANT`, `SCHEME`, `FONT_SIZE` (medium) and `ASPECT` choose the theme's
  options; `ASPECT` comes from W × H when the ratio is exactly 16:10, 16:9 or 4:3. `THEMES` is the folder holding the
  theme (`~/.cache/emusen/bigpicture/corpus/whole`). With `DRY=1` it writes the library, the home and the settings,
  prints the pad's steps and the whole command, and starts nothing; a dry run needs no binary. A real run without the
  binary stops with a line naming the path it looked at.
- `settings.py <es_settings.xml> Name=value...` — the settings file of one state. Only the settings it names are
  written, each with ES-DE's type, and ES-DE keeps its own default for the rest. Fixed for every state: no updater, no
  scraper account, no navigation sounds, no screensaver, no notice when a pad arrives (it would sit in the capture),
  help prompts on, clock on.
- `library.py <out>` — the synthetic library, generated, never copied: the five systems `nes`, `snes`, `n64`, `gb` and
  `gbc`, twelve invented games in each as empty files, with ES-DE gamelists carrying their metadata; in `snes`, the
  folder *Tower Set* (two games, and a folder link to the first) and *Lumen Garden* marked *hide metadata*
  (`EmuSen_BigPicture.md` §31.1). `downloaded_media/<system>/<type>/` holds one flat-hue PNG per game and type, labelled
  with its system, type, name and size. The titles, the metadata, the picture sizes and the colour of each picture
  (red 40 + 17*i mod 180, green 60 + 23*len(type) mod 160, blue 90 + 31*i mod 150, for game i) are the WiseMan
  fixture `SyntheticLibrary`'s, so ES-DE, the C# painter and the Qt painter can be given one library. `library.tsv`
  and `media.tsv` list every game and every picture with its size and colour, as plain data for a comparison to read.
  The media viewer's game, the NES's first (*Aurora Drift*), also has the four types the viewer shows that no other game
  has (USERGUIDE.md, "Game media viewer"): `videos` (the labelled picture held four seconds, H.264 by the system's
  ffmpeg, so the playing video is the same picture at any time), `backcovers`, `fanart` and `custom`. They are kept off
  the SNES, whose system view (Artflix's background video, the gameselectors' pictures) and gamelist the watched states
  capture.
- `pad.py <step>...` — the uinput pad "gate pad" pressed by the steps (qt-gates' `run.sh` runs it with `UINPUT_PAD`).
  ES-DE reads only real pads, so this one is an input device every program on the desk can see: it exists only while
  its steps run, and it is refused while Steam runs (`pgrep -x steam`), as `run_esde.sh` refuses the run before ES-DE
  starts. Decided 2026-11-09 by the tester: the uinput pad is allowed for the oracle, used with care, never while Steam
  can see it, each run watched.
- `state.py <system> system|gamelist[:<j>[:media:<n>]]` — a state as what reaches it: the settings, the pad's steps and the run's
  length, as three lines of text.
- `helpcopy.py <theme-folder> <out> <name> <fontSize> [property=value...]` — a test copy of a theme, written outside
  the repository and never committed as a theme, whose gamelist help shows four entries with the same disc (a, b, x, y)
  at full opacity at the given `fontSize`, and any of `entryRelativeScale`, `entrySpacing`, `iconTextSpacing`,
  `letterCase` and `entries` given, the rest unchanged; run with `THEMES=<out>`. Its captures give a help bar's spacing
  at sizes the watched themes do not show (EmuSen_Qt.md §4.12).
- `helpstates.tsv` and `helpstates.sh` — the help copies of 2026-11-09 that change one property at a time from a bar
  whose every size is whole (EmuSen_Qt.md §4.12.1), each written and captured in turn at 1280 × 800, `gamelist:1`
  (`caralt-help-<name>-es-de/gamelist-1-1280x800.png`). `helpstates.sh <table>` runs another table the same way:
  `helpwidths.tsv` holds a single Select label at eleven font sizes from 19.2 to 20.8 px, which separates the label's
  measured width from its scale.
- `manualstates.sh` — the media viewer's PDF mode on the NES's manual game in CarAlt at both sizes (below).
- `viewerstates.sh` — the media viewer under its settings (USERGUIDE.md, "Media viewer settings") on the NES's first
  game in CarAlt at both sizes, scanlines off, each capture named by a `TAG`.
  `run_esde.sh` takes `ESDE_SETTINGS` ("Name=value ...", names `settings.py` knows) and `TAG` (added to the name).
- `captures.sha256` — every capture the oracle comparison (`Frontend/esde-qt/tests/oracle.rs`) reads, as `sha256sum`
  lines relative to `~/.cache/emusen/esde-oracle`: 86 files, hashed on 2026-11-11. The captures stay outside the
  repository; the comparison refuses one that is not listed or whose sum differs, so a recapture changes the
  comparison only through a commit of its new line. `cd ~/.cache/emusen/esde-oracle && sha256sum -c <this file>`
  checks them by hand.
- The comparison's pictures for a human look (`oracle/qt/`, `oracle/difference/`, `qt/`, the primitives' grabs) go to
  `~/.cache/emusen/tmp/qt-s4/compare/<checkout>/`, the checkout's folder name keying them, so runs from two checkouts
  never write one folder (on 2026-11-11 one run's grabs were overwritten by another's while being read).
- `pick.py <run-dir> <seconds> <out.png>` — keeps the run's capture, and prints its time and its difference from the
  grab one second before, which is 0 when the picture had settled.

## States

A state is a theme, a view, a size and the theme's options. The name of its capture is `<view>-<W>x<H>.png` under the
theme's folder, the view written as given with `:` turned into `-`: `system-1280x800.png`,
`gamelist-1-1920x1200.png`. The watched states are `system` and `gamelist:1` (the first game) at 1280 × 800 and
1920 × 1200.

- **The system is named, not walked to.** ES-DE starts in the named system's gamelist (`StartupSystem`, `StartupView`
  `gamelist`); a carousel's axis and the systems' order belong to the theme and to ES-DE, so a count of steps would
  be a guess per theme. The pad walks from there.
- **system**: one press of *b*, back to the system view at that system. A capture that still shows the gamelist means
  the pad did not reach ES-DE.
- **gamelist:j**: *down* pressed *j* times; entry *j* as ES-DE lists it, favourites first and folders on top. In the
  SNES list that puts *Tower Set* at entry 0, so the first game is entry 1 (see the measurements below).
- **gamelist:j:media:n**: then *x*, the media viewer (1 s to open), and *right* pressed *n* times. With `SYSTEM=nes` and
  entry 0, the viewer's game and every type it shows, one capture per *n* (`nes-gamelist-0-media-<n>-<W>x<H>.png`);
  at the SNES's entry 0, *x* on the folder *Tower Set* (`gamelist-0-media-0-<W>x<H>.png`).
- **gamelist:j:manual:n[:zoom|zoom-out|pan|pan-y:k]...**: then *x*, *up* (the game's PDF manual, 1 s to open), *right*
  pressed *n* times (a page each), then each step in the order written: `zoom` the right shoulder *k* times, `zoom-out`
  the left shoulder, `pan` *right*, `pan-y` *down*. With `SYSTEM=nes` and entry 1, Cobalt
  Harbor, whose manual `library.py` writes: three labelled pages of 600 × 800 points.

The pad exists only while its steps run: it appears when ES-DE starts, presses after 4 s, and is removed about 5 s
after it appeared, since any program reading pads on the desk (Steam among them) sees it too. ES-DE draws its first
picture within a second of its start, so 4 s is a wide margin. A run fails when ES-DE's log has no "Added controller
… gate pad" line, or when the sandbox can see a network interface other than its loopback.

## Use

    cd Testing/esde-oracle
    DRY=1 bash run_esde.sh art-book-next-es-de system 1280 800
    TMPDIR=$HOME/.cache/emusen/tmp systemd-run --user --scope -p CPUWeight=20 -p MemoryMax=7G -q \
      bash run_esde.sh art-book-next-es-de system 1280 800

Each run uses the GPU and a compositor: one at a time.

## Measured 2026-11-08

Art Book Next (`gamelist-list-metadata-cover`, `dark-screenshots`, medium font, 16:10), SNES, the four watched states:
each run 9–10 s, the capture taken 9.0 s after ES-DE's start and identical to the grab one second before (difference
0.000), ES-DE gone after each run, no pad left. ES-DE logged the pad as "Added controller with default configuration:
"gate pad"", so SDL's own mapping of the uinput pad serves and no `es_input.xml` is written.

- The system view shows the SNES carousel entry, the clock and the help bar (MENU, SELECT, SCREENSAVER, CHOOSE). Beside
  the C# painter's grab of the same state from 2026-09-25, the art, the logo and their places agree; the help bar
  differs in its words and icons, as `EmuSen_BigPicture.md` §36.3 leaves it.
- The gamelist shows *Tower Set* first with ES-DE's folder icon, then the favourites with their stars and *Aurora
  Drift* selected, then *Brass Lantern*; the cover, the description, the rating, the date, the players and the play
  time agree with the C# grab, whose library had no folder, so its list starts one row higher.
