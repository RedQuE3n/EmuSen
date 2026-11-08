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
- **Not yet in the module**: `CoverTile` (it moves into the kit with the reference theme's package), `VideoTile` (stage 4),
  and `Bells` with `ladder(count)` and `strike(i)` (the kit's `Bells` renders cues and has neither yet). Each is added by
  name when the kit has it.
