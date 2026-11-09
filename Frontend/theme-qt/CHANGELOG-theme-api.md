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
  renders cues and has neither yet). Each is added by name when the kit has it. (`Bells` was added later the same day,
  below.)

## 1.0, amended for the reference theme as a package (2026-11-08)

Decided 2026-11-08: writing the reference theme "EmuSen" against the module found what follows missing; each is an
addition, appended after the members it joins so that no index in the registry moves.

- **`Session.key`, `console`, `hue`**: the suspended game's file (so a shelf can leave it out and Continue can play it),
  its console's name as its facts begin (`Words.pausedFacts` takes it), and its console's hue, -1 where the library does
  not hold the game.
- **`Library.shelf`**: `recent` less the suspended game, which Continue holds: Home's shelf. **`Library.firstOf(key,
  count)`**: the first games of `gamesOf(key)`, so a fan of four covers makes four delegates and not one a game.
- **Console rows gain `libretro`**: whether the console runs on a libretro core, which a theme cannot tell from
  `engineKind` without writing the word.
- **Lists change in place**: a list fed the same rows by key changes their cells (`dataChanged`) and keeps its delegates,
  so a theme's focus and its items stay when a line or a count changes; other rows replace it whole.
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
  `searching`, `typing`, `listOpen`, `searched` (the search's words narrow it), and `games`, the rows in the order shown.
- **`Marks`** (added): the drawn marks, `heart`, `sparkle`, `crescent`, `attention`, `badge`, `alien` and `path(name)`
  for a shelf's icon or an engine's mark, each an SVG path.
- **The kit's types by name**: `CoverTile` (a cover whole at its own shape on its box's foot, the generated card
  otherwise, the favourite's mark), `RoundPicture`, `PadGlyph` (with `PadGlyph.Button`, the pad's buttons in the kit's
  order) and `PlanetLayout` (Choose a console's rows of bodies and their walk).

## 1.0, amended for the four themes (2026-11-08)

Decided 2026-11-08: the gaps 9 to 12 found by the four themes' specifications (Horizon, Meridian, Constellation, Orrery)
and the console screens' maker groups are added, each after the members it joins.

- **`Help.entries(kind, {group, back})`** (gap 9): the kind's list with **LB RB Group** (the shoulders jump a group of the
  screen's own row) and **B Back** (where the kind has none) added as asked, at the places the canvases put them: Group
  after Home's three game entries or a place's Open, and after Choose a console's Open and Back; Back before it. A kind
  whose shoulders already mean something (the grid, the search) takes no Group. Neither names an action: the theme's
  own walk answers them, the shoulders reaching it as PageUp and PageDown. `entries(kind)` is unchanged. The words and
  places are `help.rs`'s (`GROUP`, `BACK`, `entries_with`).
- **`Words.noGames`, `noRecent`, `noConsoles`, `addFolder`, `places`, `pausedAgo(pausedSince)`** (gap 10): the empty
  states, "No games yet", "Nothing played yet", "No consoles have games yet", "Settings › Library adds a folder"; the
  PLACES eyebrow; "Paused 12 minutes ago". From `words.rs`; no theme writes them.
- **`Library.places` rows gain `available`, and `Actions.enabled(key)`** (gap 11): a place is not available when the host
  would decline its `openPlace` (`ThemeLibrary::show_declined`), and `enabled(key)` says whether the host would take
  `act(key)`, asked without acting (`ActionSink::would`, true unless a sink says otherwise; false with no sink). The role
  is not named `enabled`, as the specifications had it: a delegate declaring it would override `Item.enabled`, and a
  disabled item could not hold the focus and the ring that the disabled look keeps.
- **`Bells`** (gap 12): `ladder(count)` makes the ladder's first `count` rungs ready and returns the count; `strike(i)`
  rings rung i, the bell world i rings in Moonrise and a console in a picker. Both are handed to the host's `Striker`,
  which renders through the kit's `Bells` (`ladder(count)` and `rungFrames(i)`, each rung rendered once and kept) and
  rings at the Navigation volume; with no striker, silence. The theme's `Bells` is a singleton of this module, not the
  kit's type, because the kit's renders and does not ring.
- **Console rows gain `group`, `groupIndex`, `groupSize`**: the console's maker's ordinal in order of first appearance
  in the table, its place within that group and the group's size, from the catalogue's maker; a console of unknown
  maker is a group of its own. Every console screen lays out its makers and jumps them with the shoulders from these.

- **Console rows gain `groupStart`** (added after the three above, the same day): how many consoles the groups before
  this one hold, so a console's place in maker order is `groupStart + groupIndex` and a group's heading stands before
  `groupStart`. The rows stay in the table's order and a theme's QML neither sorts nor sums, so a screen drawing makers
  together could not place a console whose maker returns later in the table (as theme-audit's 120-console crowd has
  them) without it. Found by Meridian's rail.
- **`Words.continueEntry` and `recentEntry`** ("Continue", "Recent"), and **`Marks.path` of Home's seven places**
  (`games`, `scraper`, `packages`, `library`, `cores`, `settings`, `power`), the same day: a theme listing Continue and
  the Recent shelf beside the places as destinations (Meridian's rail) names and marks each, and had neither word nor
  mark. The words are `words.rs`'s and the paths `marks.rs`'s, as the canvases draw them.
- **`Help.entries(kind, {accept})` and an `enabled` role on the help's rows** (the same day): `accept: false` gives the
  kind's list with A's entry not `enabled`, which the kit's `HelpBar` draws at 38% and does not press; every other entry,
  and every list asked for without the flag, is enabled. A theme draws the disabled look of a place the host would
  decline (`available` false) with its bar's A dimmed, and could not mark one entry of a list it may not build. Each kind
  now has eight lists, made once. A delegate reads the role as `model.enabled`. Found by Horizon's disabled state.
