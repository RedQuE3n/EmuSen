# esde-baseline

The C# painter of ES-DE themes (`EmuSen.Mistress/BigPicture/`), measured once in the oracle's watched states before it
retires, so that the Qt painter is held to it (`EmuSen_Qt.md` §4; the comparison itself is the Qt side's). Two things
are kept: each state's rectangles, as `mockup 1` specs, and the C# painter's mean absolute error against ES-DE, as
`mae.tsv`. Nothing here is built or shipped.

## The states

`states.tsv` lists the watched themes with the options both ES-DE and the C# painter are given: the theme's folder
under `~/.cache/emusen/bigpicture/corpus/whole`, its variant, colour scheme (`-` for none) and font size. Each theme is
measured in four states: the system view at the SNES (`system`) and the SNES gamelist at entry 1 (`gamelist-1`, the
first game, since *Tower Set* is entry 0), at 1280 × 800 and 1920 × 1200, as `../esde-oracle/README.md` defines them.
The library is the oracle's (`../esde-oracle/library.py`, the one owner of the synthetic library), read by both.

- Art Book Next: `gamelist-list-metadata-cover`, `dark-screenshots`, the variant and scheme the oracle's own
  measurements of 2026-11-08 used.
- Artflix: `Default`, the theme's first variant; it has no colour schemes.
- CarAlt: `withoutVideo` and `gold`. The library has no videos, and the video element is a family the Qt painter maps
  after the switch, so the variant without one is the fair state; `gold` is the theme's first scheme.

## The specs

`<theme>-<state>-<W>x<H>.spec`, one per state, in the `mockup 1` format of `EmuSen_Qt.md` §7.1, read by
`emusen-qt-test`'s `mockup` module. Each holds one `element` line per element the C# scene drew (a `SceneEntry` with a
control that is visible), in drawing order, with `x`, `y`, `w` and `h`:

- **Names** are ES-DE's own key for an element, its type and name joined by an underscore (`image_md_image`,
  `helpsystem_help`); no type holds an underscore, so the first one splits them.
- **Units** are design units of 1280 × 800 at both sizes: a 1920 × 1200 box is divided by 1.5, so the Qt comparison's
  tolerance of two design units is the same fraction of the screen at both sizes (design §11). The `source` line names
  the window's size and the unit.
- **Boxes** are the control's layout box on the scene's canvas (`Control.Bounds`), before any rotation or scale, rounded
  to 0.01 of a unit. An element the C# did not draw is absent; the test's output lists the skipped ones with the reason.
- **Fixed for repeatability**: the gameselectors' shuffle (0), the clock (the capture's modification time, so the clock
  and the relative dates show what ES-DE showed), no live clock, and the device status empty, as the sandboxed ES-DE
  shows none.

## ES-DE's own boxes (2026-11-09)

Decided 2026-11-09 by the tester: the Qt painter is held to ES-DE 3.4.1, not to the C# painter (`EmuSen_Qt.md` §4.8.7).
One file holds what the specs cannot:

- `esde-boxes.tsv`: ES-DE's box of an element where the C# scene's box is not ES-DE's, measured on the 2026-11-08
  captures, one row per theme, state, width and spec key, in design units of 1280 × 800, with how it was measured (a
  background's edges less the theme's paddings, an icon's ink less its box's margin with the edge the theme anchors, a
  drawn picture's edges, or nothing drawn). The oracle holds every spec element to this row where there is one and to
  the spec's box otherwise.

## The MAE

`compare.py` writes `mae.tsv`: per state, the mean absolute error of the C# grab against the ES-DE capture over R, G and
B on the 0–255 scale, over every row but the help bar's strip. The strip is the rows the state's `helpsystem_*` elements
cover in its spec, the full width (`strip_top` inclusive to `strip_bottom` exclusive, in window pixels), because the help
bar's words and icons are Mistress's own (`EmuSen_BigPicture.md` §36.3). The Qt figure for a state must not exceed the
C# one, measured over the same rows.

The side-by-sides (ES-DE, C#, and the difference ×4 with the strip shaded blue) go to
`~/.cache/emusen/tmp/qt-s4/compare/<theme>-<state>-<W>x<H>.png`, outside the repository.

## Use

    bash capture.sh                         # ES-DE's captures, one run at a time (GPU, compositor); kept ones skipped
    EMUSEN_ESDE_BASELINE=$PWD dotnet test EmuSen.WiseMan -p:UseSharedCompilation=false \
      --filter FullyQualifiedName~EsdeBaselineTool    # the specs, and the C# grabs to ~/.cache/emusen/tmp/qt-s4/compare/csharp/
    python3 -I compare.py                   # mae.tsv and the side-by-sides

The C# test is skipped unless `EMUSEN_ESDE_BASELINE` names this folder. It renders headless (Avalonia's software
renderer), reads the library and the captures from `~/.cache/emusen/esde-oracle/` (`EMUSEN_ESDE_ORACLE`), the themes
from `THEMES`, and fails naming the command to run when either is missing. It asserts, per state, that the spec holds
as many elements as the scene drew and that no key repeats.

## Measured 2026-11-08

The twelve states, ES-DE 3.4.1 in the oracle rig (Art Book Next's four captures the rig's own of the same day, the
other eight taken for this baseline; each settled, the grab identical to the one a second before, except Artflix's
system view at 1920 × 1200 at 0.024), and the C# painter as it stood at WiseMan e72f348f0. MAE in `mae.tsv`:

| Theme | system 1280 | gamelist 1280 | system 1920 | gamelist 1920 | elements drawn (system, gamelist) |
|---|---|---|---|---|---|
| Art Book Next | 1.12 | 2.61 | 0.38 | 2.00 | 6, 16 |
| Artflix | 6.56 | 9.29 | 6.47 | 9.14 | 9, 16 |
| CarAlt | 24.72 | 16.30 | 24.69 | 16.08 | 20, 28 |

A second run wrote byte-identical specs and grabs, and `emusen-qt-test`'s `mockup::parse` read all twelve specs with
the element counts above. What the side-by-sides show, each a difference of the C# painter's the Qt painter need not
repeat:

- **Art Book Next.** Art, logo, cover, description and metadata agree. The gamelist shows seven entries where ES-DE
  shows eight (*Brass Lantern* missing), the folder's mark is a folder where ES-DE draws its enter icon, and the list's
  stars are smaller.
- **Artflix.** The system view's title is Mistress's full name for the system, "Super Nintendo", where ES-DE's is
  "Nintendo SNES (Super Nintendo)", and the count reads "14 games available, 7 favorites" for ES-DE's "14 games (7
  favorites)"; the description wraps at a larger size. In the gamelist, *Tower Set* wraps onto two lines and the
  description's lines are spaced wider.
- **CarAlt.** The carousel's and the gamelist's screenshots are tiled at a small size where ES-DE draws each picture once,
  filling its box, which is most of both views' error. The system view's three screenshots come from a gameselector that
  picks at random in ES-DE, so its pictures differ by chance as well; the C# picks are fixed (shuffle 0) so that the
  baseline repeats, and the Qt comparison should fix its own the same way.
