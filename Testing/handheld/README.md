# handheld

The Qt front end's tests and gates on the handheld (the Legion Go S, SteamOS, 1,920 × 1,200), run inside a box that
leaves Game Mode, Steam and the player's files as they were (`EmuSen_Qt.md` §9.3.6). Nothing here is built or shipped.
Every run on the handheld goes through `hh.sh test` (on the charger only), in a capped scope there:
`systemd-run --user --scope -p CPUWeight=20 -p MemoryMax=6G`.

- `pack.sh <out.tar.zst>` (desktop) — what the runs need, built beforehand by cargo in the checkout it sits in: the
  test executables of `emusen-qt`'s `acceptance` and `emusen-kit-gallery`'s `gate` (newest in `target/debug/deps`,
  debug sections stripped), `emusen-qt` in release built with `--features gate-pad`, the platform library and the test
  core of the `test-fast` profile, the Frontend's sources and Testing's mockups, Moonrise references and scripts the
  tests read, the published tree's program folder for the gates, and the desktop's Qt (`qtbundle.sh`) with a `qt.conf`
  beside each program naming it. `SONAMES` names a file of the handheld's library sonames (`ldconfig -p`).
- `qtbundle.sh <dest> <sonames> <binary>...` (desktop) — the desktop's Qt laid out to travel: SteamOS's Qt 6.11.1
  exports no private API, and cxx-qt's generated code needs `Qt_6.11_PRIVATE_API`, so the programs run on the Qt they
  were built against, as the AppImage of `EmuSen_Qt.md` §10.5 would carry it. Qt's and ICU's libraries and whatever the
  handheld lacks by soname; the Wayland and offscreen platforms, Wayland's integrations, five image formats; the QML
  modules the front end imports.
- `box.sh <scratch> <command>...` (handheld) — the box, by `bwrap`: the root read-only; `/home` empty but for the
  scratch folder and the desktop checkout's path, bound to the unpacked tree, since the tests name their files by the
  path they were built at; `/dev` with the GPU's nodes alone, so the handheld's own controller, hidraw and input devices
  are not there to be read or written; `/tmp`, the runtime folder and a D-Bus session of its own (`session.conf`: no
  service is started on request, so no desktop portal); `HOME`, the caches and `TMPDIR` in the scratch folder. The
  bundled Qt's libraries are named in `QT_BUNDLE_LIBS` and given to the programs built against them alone;
  `shim/kwin_wayland` takes them off the library path before the handheld's own KWin starts.
- `tests.sh <scratch> acceptance|gate` (handheld, inside the box) — one test executable with the prebuilt platform
  library and test core, its output in `<scratch>/logs/<test>.log`, the acceptance's grabs and specs in
  `<scratch>/compare`. The acceptance makes its own `kwin_wayland --virtual` (3,840 × 2,160, for the kit's largest size)
  and presses an SDL virtual pad of its own process; the gallery's gate runs offscreen on Qt's software renderer, as on
  the desktop. The gate's `qmllint` is the handheld's own, over the handheld's Qt modules.

The start-up and memory gates are `../qt-gates/gates.sh` inside the box (`BOXED=1`): `emusen-qt` started with the
bundle's libraries, KWin's virtual compositor at 1,920 × 1,200 without Xwayland (`XWAYLAND=0`), and the gate pad inside
`emusen-qt` (`../qt-gates/pad.py`). The library is the handheld's own ROM folder, read-only as the whole root is.

## A round, as run on 2026-11-08

    # desktop, in the checkout
    cargo test -j6 --no-run -p emusen-qt --test acceptance -p emusen-kit-gallery --test gate
    cargo build -j6 --release -p emusen-qt --features gate-pad
    (cd Platform && cargo build -j6 --profile test-fast -p emusen-platform --lib -p emusen-crystal --example crystal_test_core)
    hh.sh run 'ldconfig -p | awk "{print \$1}" | sort -u' | grep -v '^\[exit' > sonames.txt
    SONAMES=sonames.txt bash Testing/handheld/pack.sh qt-hh.tar.zst
    hh.sh put qt-hh.tar.zst /home/deck/.cache/emusen/qt-hh
    hh.sh run 'cd ~/.cache/emusen/qt-hh && tar --zstd -xf qt-hh.tar.zst'
    # handheld, each line through hh.sh test, with S=~/.cache/emusen/qt-hh and R=$(cat $S/root)
    systemd-run --user --scope -p CPUWeight=20 -p MemoryMax=6G -q bash $S/tree/Testing/handheld/box.sh $S \
      bash $S/tree/Testing/handheld/tests.sh $S gate            # and: ... acceptance
    systemd-run --user --scope -p CPUWeight=20 -p MemoryMax=6G -q bash $S/tree/Testing/handheld/box.sh $S \
      env ROMS=/run/media/deck/LX1TB/ROMs XWAYLAND=0 bash -c \
      "bash $R/Testing/qt-gates/sandbox.sh $S/sb $R/Frontend/target/release && bash $R/Testing/qt-gates/gates.sh $S/sb $S/runs warm"
    systemd-run ... box.sh $S env ROMS=... XWAYLAND=0 ROUND=1 bash $R/Testing/qt-gates/gates.sh $S/sb $S/runs start idle pause
    python3 -I $S/tree/Testing/qt-gates/figures.py $S/runs 1

The scratch folder is removed when the measurement is recorded: `rm -rf ~/.cache/emusen/qt-hh`.
