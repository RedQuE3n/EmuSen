# qt-gates

The scripts that measure the Qt front end's gates against Mistress in a compositor of their own (`EmuSen_Qt.md` §9.3).
Nothing here is built or shipped; each script is plain text in and out, and none touches the player's own home.

- `run.sh <run-dir> <seconds> <program> [<arg>...]` — one run: KWin's virtual compositor at 1,920 × 1,200 with
  Xwayland, on a D-Bus of its own, every window fullscreen by a rule; the program started with polled grabs and, when
  `PAD` names steps, a uinput pad; everything it started stopped at the end, and the whole run under a watchdog.
  `BOX` is the game's colour box for the grabs, `INTERVAL` their interval in ms (35).
- `grab.py` — the grabs, through KWin's `ScreenShot2`: one line each (time, share of the screen lit, share in the
  colour box, the PNG), the PNGs at 480 × 300.
- `pad.py` — the pad, "gate pad" (vendor 1209, product 0001), pressed by steps: `wait:<s>`, `tap:<button>`,
  `hold:<button>:<s>`. Every program on the desk reads it, so no `guide`, and nothing else reading pads while it runs.
- `first.py <run-dir>` — the program's exec to its window and to the game's first picture, from the grabs.
- `shell.py <tree>/lib/EmuSen` — a terminal over a published tree's platform library: its shell with every EmuSen
  module and the runtime's settings service, as Mistress's shell has them, so `cores fetch`, `get`, `approve` and
  `default` run as typed. Lines come from stdin. Its executable is the Python interpreter, so an approved entry is read
  by the next program started from the tree (the proxy looks for `emusen-core-runner` beside the executable).

## The hosted core's first picture (§9.3.3)

With a sandbox copy of a publish in `$SB` (`lib/EmuSen` with `emusen-qt-play` beside the platform library, a `home/`
of its own, `HOME` and `XDG_*` pointing into the sandbox) and a game in the library in `$SMB`:

    printf 'cores fetch\ncores get nestopia\ncores approve nestopia\n' | python3 -I shell.py $SB
    # start any program from the copy once, so the entry is read; then
    printf 'cores default NES "Nintendo - NES / Famicom (Nestopia)"\n' | python3 -I shell.py $SB
    export BOX="60-150,90-190,200-255" SDL_AUDIODRIVER=dummy
    QT_QPA_PLATFORM=wayland EMUSEN_PLATFORM_LIBRARY=$SB/libemusen_platform.so EMUSEN_CORES=$SB \
      bash run.sh runs/r1-qt 9 $SB/emusen-qt-play --seconds 8 --game "$SMB"
    bash run.sh runs/r1-mistress 9 $SB/EmuSen --play "$SMB"
    python3 -I first.py runs/r1-qt; python3 -I first.py runs/r1-mistress

`cores fetch` and `cores get` are network requests to buildbot.libretro.com; run them only with the tester's go. Each run
uses the GPU, so one at a time, inside a capped scope.
