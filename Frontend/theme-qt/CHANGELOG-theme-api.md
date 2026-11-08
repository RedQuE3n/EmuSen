# EmuSen.Theme API changes

The theme API is the QML module `EmuSen.Theme` and the `theme.json` schema (EmuSen_Qt.md §3.2–§3.4). A minor adds and
never removes; before 1.0 (§5.1) a breaking change is allowed and listed here.

## 1.0, amended before release (2026-11-08)

Decided 2026-11-08 by the tester: the additions found by writing four original themes against §3.2 (the stage 3 design,
§5 there) are accepted. As built:

- **`Library.places`** is every Home destination in Home's order: Games, Scraper, Packages, Library, Cores, Settings,
  Power, with roles `key`, `name`, `kind` (`library`, `tool`, `settings`, `power`), `line`, `attention`, `count` (Games
  counts its consoles). `Actions.openPlace(key)` takes any of them.
- **`Library.shelves`** (added): the library's own three places, All, Favourites and Recent, with `key`, `name`, `line`,
  `count`. §3.2 had them as `places`; they moved here so that `places` is Home's seven alone, which every design draws.
- **`gamesOf(key)`** takes a console's key or a shelf's (`all`, `favourites`, `recent`), and its rows gain `hero` (fan
  art, else a screenshot, else the title screen, else the cover) and `lastPlayedWords` ("12 min ago").
- **Every row's identity is the role `key`**, not `id`: QML reserves `id`, so a delegate could not declare it as a
  required property. A game's key is its file, which `Actions` take.
- **`Words`** (added): the three screens' labels and the counted and timed phrases, `games(n)`, `ago(t)`,
  `pausedFacts(console, pausedSince)`, `facts(maker, year, count, runsOn)`, `runsOn(engineName, engineKind)`,
  `statusLine(controllers, clock)`, `splashCaption(n)`. The label of Choose a console is `gamesHeading`, since `games` is
  the counting phrase.
- **`Help`** (added): `entries(kind)` for `Help.Continue`, `Game`, `Console`, `Place`, `Section`, `Search`, a list with
  `button`, `label`, `pair` and the `actionLines` the kit's `Frame` declares; `Frame.help` binds to it. The role is
  `label`, the name the kit's `HelpBar` reads.
- **`Screen` is renamed `ThemeScreen`**: QtQuick's attached `Screen` has the same name, and which one a theme got
  depended on the order of its imports. It gains the role `ThemeScreen.Splash` and, for the splash, `length` (ms),
  `still` and the signal `finished`. Its roles are `ThemeScreen.Home`, `Consoles`, `Games`, `Splash`; `focusRole` takes
  `ThemeScreen.Navigation`, `Content`, `Detail`, `Actions`, `Search`, the kit's walk roles in the kit's order.
- **`Device.family`** (added): the pads' glyph family, as `Frame.padFamily` takes it.
- **`Tokens`** is a read-only face of the kit's tokens, every one of them, with `Tokens.Dark`/`Light` and
  `Tokens.Desktop`/`BigScreen`/`Card` to compare `variant` and `form` with.
- **The kit's types by name**: `Frame`, `HelpBar`, `Walkable`, `Body`, `Sky`, `Scene`, `Carousel`, `NormalizedCanvas`,
  `Moonrise`. `Walkable` (every item the pad and the pointer reach) and `Scene` (a theme's own colours, §7.20 of
  EmuSen_Kit.md) are added: the design's rule that every item is a `Walkable` needs them.
- **Not yet in the module**: `VideoTile` (stage 4), and `Bells` with `ladder(count)` and `strike(i)` (the kit's `Bells`
  renders cues and has neither yet). Each is added by name when the kit has it.

## 1.0, amended for the reference theme as a package (2026-11-08)

Decided 2026-11-08: writing the reference theme "EmuSen" against the module found what follows missing; each is an
addition, appended after the members it joins so that no index in the registry moves.

- **`Session.key`, `console`, `hue`**: the suspended game's file (so a shelf can leave it out and Continue can play it),
  its console's name as its facts begin (`Words.pausedFacts` takes it), and its console's hue, -1 where the library does
  not hold the game.
- **`Library.shelf`**: `recent` less the suspended game, which Continue holds: Home's shelf.
- **Game rows gain `facts`**: the console's name and when the game was last played, "Super Nintendo · 3 h ago". `cover`
  and `hero` are now file URLs, as an `Image` reads them; they were bare paths, which QML resolved against the theme's
  own folder.
- **`Words.summary(consoles, games)`**: Choose a console's "11 consoles · 1,566 games".
- **`Help.SectionSearched`**: the grid's help while a search narrows it (B clears the search).
- **`Actions.act(key)`**: a help bar's declared action by the key `Frame.actionInvoked` gives, since a theme writes no
  literal to tell one from another. **`Actions.focus(key)`**: the game the focus is on, which the cards and Y act on.
- **`Browse`** (added): the section a Games screen shows as the program holds it, because the List card and the search
  are EmuSen's and sort, narrow and move it: `section`, `heading`, `world`, `hue`, `countLine`, `previous` and `next`
  (the shoulders' neighbours), `focusRow` (where the program holds the focus; the List card's jumps move it),
  `searching`, `typing`, `listOpen`, and `games`, the rows in the order shown.
- **`Marks`** (added): the drawn marks, `heart`, `sparkle`, `crescent`, `attention`, `badge`, `alien` and `path(name)`
  for a shelf's icon or an engine's mark, each an SVG path.
- **The kit's types by name**: `CoverTile` (a cover whole at its own shape on its box's foot, the generated card
  otherwise, the favourite's mark), `RoundPicture`, `PadGlyph` (with `PadGlyph.Button`, the pad's buttons in the kit's
  order) and `PlanetLayout` (Choose a console's rows of bodies and their walk).
