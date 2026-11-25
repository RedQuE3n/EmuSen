# EmuSen.Theme API changes

The theme API is the QML module `EmuSen.Theme` and the `theme.json` schema (EmuSen_Qt.md §3.2–§3.4). A minor adds and
never removes; before 1.0 (§5.1) a breaking change is allowed and listed here.

## 1.18, an addition: Settings Controllers inner pages and Output boards (2026-11-24)

- **`Settings.boardLines`** and **`Settings.boxMenu`**: Eclipse secondary aside lines (20px light) for Players seat summaries and Hotkeys key/trigger ordering without altering reference-theme fields, plus rounded-box Start glyph flag for Players, Profiles, One profile, Hotkeys, Output and Memory cards pages (boards #52–#56; R-20261116-20, -22, R-20261123-32).
- **`Help.SettingsHotkeys`**: maps `Screen::SettingsHotkeys` (`A Change`, `X Clear`, `B Back`, `START Home`) for board #55.
- The module is 1.18 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.18.*`.

## 1.17, an addition: Settings Memory cards and card saves boards (2026-11-24)

- **`Settings.boardWrapAbout`** and **`Settings.cardsHelp`**: whether rows wrap their short line at 1.35 line-height (`SettingsRow`) and whether the Memory cards help bar (`◀ ▶ Value`, `A Change`, `B Back`, `START Home`) is shown on Memory cards, Drawer, Saves on the cards, One save and Delete pages (boards #58, #63, #64; R-20261116-20, R-20261123-32).
- **`Help.entries(kind, {settings: true, cards: true})`** (no new method): Memory cards' help bar in the boards' wording and order (`help::cards_settings_named`).
- The module is 1.17 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.17.*`.

## 1.16, an addition: Settings Consoles grid, one console, Core and Firmware boards (2026-11-24)

- **`Settings.rows` additive board roles** (`boardLabel`, `boardAbout`, `boardValue`, `boardForm`, `planet`, `subtitle`, `chip`, `chipAttention`, `runs`): board-word projections and console card metadata (`PlanetInk` index, libretro checkerboard, hollow ring, firmware/unsupported chips) without changing the reference theme's mirrored roles (R-20261111-20, R-20261116-20, -21, -22, R-20261123-13, -14, -27).
- **`Settings.boardHeading`**, **`boardAboutHeading`**, **`boardSentence`**, **`boardMuted`**, **`consolesSubtitle`**, **`consolesFiltered`**, **`hasConsolesWithGames`**, **`pageGlow`**, **`cardPlanet`**, **`cardTitle`**, **`cardSubtitle`**, **`firmwareFiles`** (roles `label`, `status`, `state`), and **`firmwareCount`**: Consoles' filter/subtitle, dynamic planet page glow, Console's Core planet card, Core's About panel, and Firmware's `"Found, N of M"` file status card (R-20261123-13, -14, -15, -21, -27).
- **`Help.entries(kind, {settings: true, consoles: true, filtered, filterable})`** and **`Actions.act("consolesFilter" | "consolesLetterBack" | "consolesLetterOn")`** (no new method): Consoles' `LT`/`RT` letter jump and `Y` consoles-with-games filter (R-20261116-22, R-20261123-20).
- The module is 1.16 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.16.*`.

## 1.15, an addition: Eclipse's game page Overview, Details and Options (2026-11-23)

Decided 2026-11-23 by the tester (R-20261111-20, R-20261116-14, -27, -29, -30, -32): Eclipse draws the game page's
Overview, Details and Options tabs as their boards draw them.

- **`Page.hasDescription`**: whether `Page.words` is the game's own description rather than the empty fallback.
- **`Page.boardRows`** (`key`, `label`, `value`, `steps`, `opens`, `isSwitch`, `switchOn`, `sentence`, `row`): Details
  and Options rows in the boards' words, aside hints and underlying indices (`Page.rows`).
- **`Page.rowSentence`**, **`Page.rowPlace`** and **`Page.defaultable`**: the focused board row's aside sentence, its
  place (`"2 of 5"`), and whether X (`default`) resets an Options override back to the console's default.
- **`Help.PageResumeBoard`**, **`Help.PageOpen`**, **`Help.PageDetail`**, **`Help.PageOption`**, and `defaultable` in
  `Help.entries(kind, { defaultable })`: the boards' help lines for Overview, Details and Options.
- The module is 1.15 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.15.*`.

## 1.14, an addition: Library's folder card, EmuSen's eclipse mark and Scraping's toggle chips (2026-11-23)

- **`Settings.preview` 7, 8 and 9** (no new member): Library's ROM folder count block (7, `samples` carries count, label and path), EmuSen's About eclipse mark (8, `lines` carries the bare version) and Scraping's toggle chips (9) (R-20261116-22; EmuSen_Qt.md §3.4.11).
- **`Settings.chips`** (roles `key`, `label`, `switchOn`, `row`), **`Settings.chipCount`** and **`Settings.total`**: Scraping's eleven media kinds as aside toggle chips with their model row index and total item count for `Words.place` (R-20261116-22).
- The module is 1.14 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.14.*`.

## 1.13, an addition: Eclipse's game page, its tabs and its States tab (2026-11-23)

Decided 2026-11-16 by the tester (R-20261116-27, -29, -30, -36) and 2026-11-23 (R-20261123-33 to -36): Eclipse draws the
game page as its board does, six tabs in the board's order and the States tab's cards, and the program draws every tab
and deeper page the theme does not.

- **`ThemeScreen.pageSections`** and **`Page.PageSection`**: on the game page, the tabs the screen draws as flags, one bit
  at each tab's place, written `1 << Page.States` (the enum's names and order are qt-models' `game_page::TABS`); the program
  draws every other tab, every deeper page and every question. 0, the default, draws them all, as in 1.9.
- **`Page.tabs`** and **`Page.tab`**: the board's six tabs in its order, Cheats only for a game with cheats, and the one
  shown; **`Actions.pageTab(i)`** chooses one (Cheats opens Options' Cheats, as its row does).
- **`Page.kicker`** and **`Page.line`**: console · year · genre, and last played, time played and players, each part the
  game has; **`Page.pills`** (`key`, `label`, `button`, `filled`, `heart`, `on`, `act`, `row`): Resume, Start from the
  beginning and Favourite, or Start and Favourite where there is no resume point; `act` is the action `pageAct` takes,
  `row` the resume point's card Resume loads (-1 none).
- **`Page.cards`** (`key`, `label`, `line`, `picture`, `marked`, `badge`, `row`): the resume point, the slots in their
  order and one empty slot while a slot is free, `row` the card's place among `rows` (-1 for the empty one, which is not
  pressed); **`Page.note`**, the words beside the empty card where the game has no state.
- **`Page.trail`**, **`Page.trailSection`** and **`Page.console`**: Consoles, the console and the title, the section the
  first link opens and the console the second opens.
- **`Page.boardHelp`** and the `Help` kinds **`PageState`**, **`PageStart`** and **`PageSections`**: the help in the
  boards' words for a state's card and for Start, -1 where the boards draw none and `Page.help` is shown.
- **`Tokens.dashPattern`**: mirrors the kit's 1.12 `Tokens.dashPattern`.
- The module is 1.13 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.13.*`.

## 1.12, an addition: the face buttons' preview (2026-11-23)

- **`Settings.preview` 6** (no new member): the help bar's face buttons as they are drawn now, the preview of Controller
  type's row, which Eclipse's Controllers board draws as four faces (R-20261116-22; EmuSen_Qt.md §3.4.11).
- **`Settings.trailLine`**: every level above the page shown, joined by "  ›  " as the boards join them
  (gen_settings.py:272; `words::TRAIL_BETWEEN`), empty on a section's own page (decided 2026-11-23, R-20261123-39).
- The module is 1.12 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.12.*`.

## 1.11, an addition: what Eclipse's Settings boards draw (2026-11-23)

Decided 2026-11-23 by the tester (R-20261123-30): "Add all four, one minor."

- **`Settings.sectionKeys`**: each section's id in the description (`controllers`, `display`, …), in `sections`' order, by
  which a theme picks the board's icon for a section (`SettingsView::section_keys`).
- A row's `about` (no new member): the description's short line, else the Galaxia row's, else the row's hint (decided
  2026-11-23, R-20261123-31), so a row no board draws still has a line under its name.
- **`Settings.title`**, **`heading`** and **`aboutHeading`**: the screen's name (Home's place that opens it), the page's
  heading (the innermost page's name, else the section's) and the About panel's (the focused row's name), the words a
  theme drew by its own reasoning before (`SettingsView::title`, `heading`).
- **`Settings.rows`' `form`**: a `SettingsForm` (`Settings.Choice`, `Switch`, `Door` or `Fact`), how the boards draw a
  row's value (`settings_shown::Form`), so a theme no longer works it out from `kind` and `value`; a number, since the
  theme rules read a word compared in QML as words the player would read.
- **`Words.keyPair`**: Left with Right named as one key, "◀ ▶", the word the Settings boards' help line draws
  (`words::KEY_PAIR`).
- **X, back to the default** (R-20261123-30 (b), R-20261116-22): `Settings.defaultable` says whether the focused row has a
  default X restores; `Actions.act("default")` restores it (`SettingsView::to_default`: Galaxia's default, or on a row
  set per console each console's own); `Help.entries(kind, {settings: true, defaultable})` is a Settings kind's list as
  the boards name and order it, ◀ ▶ Change, A Choose or open, X Back to the default, B Back, then LB RB Section and
  Start Home (R-20261123-20), X's entry not `enabled` where `defaultable` is false. The program's own bar is unchanged.
- **The ◀ ▶ key** (R-20261123-30 (c)): no new member here; the kit's `HelpBar` gives a theme's glyph the entry's `pair`
  where the glyph declares `drawsPair`, and leaves the second glyph out (`EmuSen_Kit.md` §7.54).
- **`Settings.shot`** (R-20261123-30 (d)): the newest screenshot of any game as a file URL, on a picture preview's
  pane, empty for none. Eclipse does not draw it yet: R-20261116-25 asks for it through the chosen filter.
- The module is 1.11 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.11.*`.

## 1.10, an addition: a splash that enters Home (2026-11-17)

Decided 2026-11-17 by the tester (R-20261117-1, R-20261111-122): Eclipse's splash is the original board, with its sound,
entering Home as the board does. A splash over Moonrise, faded with it, cannot: the clock, the skip and the end are
Moonrise's, Home cannot time its arrival, and the board draws Home's words over the splash's disc and its stars under it.

- **`ThemeScreen.entersHome`**: a splash that leads the start in Moonrise's place; the program keeps its clock to the
  screen's `length` (held at 0 until the start has read), skips it on a press, and draws no Moonrise under it. Every other
  splash is as before.
- **`ThemeScreen.sound.source`**: the splash's sound, a WAV file of the theme's own as its screen's file names it, played by
  the program from the first frame at the Sound volume, and let go softly on a skip.
- **`ThemeScreen.splashPlace`**: on Home, the item a splash that enters Home is placed in, so Home draws under it and over
  it.
- **`Splash`** (added): `shown` and `time`, the splash's moment in ms, which the splash and Home both read.
- **`Splash.ease`** (added to 1.10, decided 2026-11-23, R-20261123-24): the hand-over's length in ms, from Home whole to
  the splash sound's last sample, 0 where there is no sound or it was given up; the program keeps `time` running through
  `length` + `ease` while `shown` is false, so Home eases its own arrival in the time the sound's tail has.
- **`Library.cores`** (rows of `key`) and **`Library.gameCount`**: the installed cores each once in the systems table's
  order (`library::installed_cores`) and the library's games as the All shelf counts them; **`Words.splashCounts(games,
  cores)`**, "1,284 GAMES · 8 CORES" (`words::splash_counts`).
- Nothing smaller would do: the time must reach two screens, Home and the splash, in step, and only the program owns the
  clock the skip and the start's reads stop; a splash cannot name Home's layers, so Home names the place.
- The module is 1.10 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.10.*`.

## 1.9, an addition: Settings and the game page as a theme's screens (2026-11-16)

Decided 2026-11-16 by the tester (R-20261116-19, R-20261116-27): Settings and the game page become screens a theme can
supply, each with a read-only view; a theme that supplies neither gets the program's own.

- **`theme.json`'s `screens`**: `settings` and `game` (`screens/Settings.qml`, `screens/Game.qml`); **`ThemeScreen.Settings`**
  and **`ThemeScreen.Game`** (added to `Kind`).
- **`Settings`** (added): the sections, the section, the trail, the page's key, the rows (with `about`, a switch's
  `isSwitch` and `switchOn`, a slider's fields and a console row's tile), the focus and the count, the help kind and the
  focused row's pane with each sample's scale (`sampleScales`), read from the program's `SettingsModel` (EmuSen_Qt.md
  §3.2).
- **`Page`** (added): a game's page, its words and pictures, the actions, the tabs, the rows or cards, the focus, the help
  kind and the question, read from the program's `GamePageModel`.
- **`Actions`**: `settingsStep`, `settingsChoose`, `settingsSlide`, `settingsSection`, `settingsFocus`, `settingsShoulder`,
  `settingsBack`, `pageSection`, `pageAct`, `pageStepAction`, `pageChoose`, `pageStep`, `pageFocus`, `pageDeclared`,
  `pageAnswer`, `pageBack` (added), each one call of the model the program's own screen calls, taken only while the theme
  draws that screen and it is shown, and answering what the model did.
- **`Help`**: the kinds `Settings`, `SettingsPage`, `SettingsConsole`, `SettingsChoices`, `Page`, `PageResume`,
  `PageResumeSlots`, `PageFavourite`, `PageLoad`, `PageView`, `PageValue`, `PageChange`, `PageChoose`, `PageNone`,
  `PageEdit`, `PageCoverArt` (added), `help.rs`'s bars for those screens.
- **`MeasuredWalk`** (the kit's, by name; 1.9 in the qmldir): the rows' walk over rows of their own heights
  (`EmuSen_Kit.md` §7.122).
- The module is 1.9 and `theme.json`'s `api` may name it; its registry and qmldir were `Testing/qmltypes/EmuSen.Theme-1.9.*` until 1.10.

## 1.8, an addition: the program's sections (2026-11-15)

Decided 2026-11-15 by the tester (R-20261115-7): Eclipse's top navigation is built as its own piece, a model for the
sections and an addition to what a theme may read.

- **`Library.sections`** (a list, roles `key` and `name`): the program's top sections in a nav's order, from their one
  owner (`qt-models/src/screens.rs`, `SECTIONS`; the words from `words.rs`): Home, Library, Consoles, Collections and
  Tools, as the boards draw them, each opening the screen the program has for it (EmuSen_Qt.md §3.4.8).
- **`Library.section`**: the key of the section shown, empty where the screen shown is none.
- **`Actions.openSection(key)`**: opens a section by its key; the host declines a key that names none, the one shown, or a
  move from a screen that is no section.
- **`Help.sections`**: the shoulders' "Sections" entry alone, for a nav's help. It declares no action, since the frame's
  walk steps the sections (`controller.sections`, `EmuSen_Kit.md` §7.27) and the theme opens the one chosen.
- No type is added, so a theme that imports `EmuSen.Theme 1.0` finds what it found before. The module is 1.8 and
  `theme.json`'s `api` may name it; its registry and qmldir were `Testing/qmltypes/EmuSen.Theme-1.8.*` until 1.9.

## 1.7, an addition: the grid's listing (2026-11-15)

Decided 2026-11-15 by the tester (R-20261115-10 to -13): a theme reads and drives the listing the List card holds.

- **`Browse.filters`** (rows of `key`, `name`, `count`), **`Browse.filter`** (the key in force) and
  **`Actions.setFilter(key)`**: the Filter page's choices, each counted once a read (`listing::choices`): Every game,
  Favourites (not in the Favourites section), Not played (new: the games with no played time), then each collection.
  The List card's Filter page shows the same list.
- **`Browse.sort`** (the sort in force, in the List card's words) and **`Actions.stepSort(by)`**: the List card's Sort
  row, one state.
- **`Browse.letters`** (rows of `letter`), **`Browse.letter`** (the focused game's) and **`Actions.stepLetter(by)`**: the
  List card's Jump to, one state.
- **`Help.entries(kind, {letters: true})`**: LT RT Letter added after the grid's Console entry, and the triggers declared
  as the letter before and after (`jump-1`, `jump+1`), carried out as Jump to. A bar not asked for it keeps LT and RT as
  the first and last entry.
- **`Browse.kind`** (`all`, `favourites`, `recent` or `console`) and **`Browse.emptyWords`**: which list is shown and what
  it says with nothing in it ("No favourites yet", "Nothing played yet", "No games found" for a search or filter, else
  "No games yet").
- **`Browse.filters`' `chip`** (added role): a choice's words on Eclipse's games view's chips, "All · 214" for every game
  (`listing::chip_name`, `words::ALL`), Favourites and Not played as named; empty for a collection, which has no chip.
- **`Browse.sortLine`**: the sort in force with its direction, "Name, A to Z" (`listing::sort_line`).
- **`Words.gameFactLabels`** ("Last played", "Time played", "States") and **`Words.crumb`** ("/"), the games board's words.
- **`Help.entries(kind, {games: true})`**: A Start, B Back and Y Favourite, in that order, as Eclipse's games board's foot
  names them (`help::games_named`, `words::START`); the action lines the list's own.
- **`Browse.factsKey`, `year`, `genre`, `players`, `description`, `timePlayed`, `states`**: the focused game's facts as
  Eclipse's games board draws them beside it, read off the GUI thread each time the focus moves to another game
  (`library::facts_of`: one scraper answer for the game, games.db's seconds played, its states counted where Galaxia's save
  paths put them). `factsKey` names the game they belong to, set last, so a theme draws them only while it is the focused
  game's; each is empty, or 0, where the program holds none. `players` is in words ("1 to 2 players", `words::players`),
  `timePlayed` too ("3 h 20 min", `words::time_played`, none under a minute).
- These members were added as 1.7, which landed together with 1.8; every type keeps the minor it was added in.

## 1.5, an addition: a console's played games (2026-11-15)

Decided 2026-11-15 by the tester (R-20261115-8): each console's number of played games is added to what a theme may read.

- **`Library.consoles`' `played`** (added role): how many of the console's games were played, a game counting as played
  when the catalogue holds a last-played time for it (`Catalog::played`, `qt-models/src/library.rs`), the same rule as a
  game row's `played`. Counted once, where the library's rows are built (`library::rows`), not per frame or per console list.
- **`Library.consoles`' `glow`** (added role): the console's own colour as `#RRGGBB`, the glow kit-core gives its body
  (`world::glow`), so a theme's corona and the Sky agree (R-20261115-9).
- **`Words.line(parts)`** (the parts joined as the facts join theirs, " · ", the empty
  ones left out; `words::facts` now calls it) (added).
- **`Help.entries(kind, {triggers: true})`** (added flag): with `group`, the group's entry is LT RT Maker on the triggers
  (`help.rs`'s `MAKER`) in place of LB RB Group, as Eclipse's Choose a console steps its makers (decided 2026-11-15 by the
  tester, R-20261115-17). The pair is drawn as one glyph, LT with RT.
- **`Library.consoles`' `screen`** (added role): the screen the console drove, in words ("CRT, television", "LCD";
  `words::screen`), from Serenity's catalogue of screens (`emusen_serenity_filters`' `screens`, read through
  `LibrarySource::screens` once a refresh); empty where the catalogue names none. For Eclipse's facts table
  (EclipseConsoles:87).
- **`Words.factLabels`** ("Games", "Played", "Core", "Screen", `words::FACT_LABELS`) and **`Words.count(n)`** (a count
  grouped by thousands, `words::grouped`) (added), for the same table's labels and values.
- **`Help.entries(kind, {actions: false})`** (added flag): the entries that carry the screen's own actions (Choose a
  console's Favourites and Random game) left out (`help::without_actions`), for a board that draws neither in its bar
  (EclipseConsoles:99-104); Open, Back and the group's entry stay.
- **`Library.consolesBy(show, order, shortened)`** (added): the consoles a core runs, those with no game among them, as Choose a
  console's chips show them (0 All, 1 With games, 2 Favourites) in an order (0 maker, 1 year, 2 name, 3 games), each
  maker's together (`console_list.rs`), by name the consoles keyed in `shortened` by their short name, as their plates show them; **`Words.consoleShows`**, **`Words.consoleSorts`** ("Sorted by maker"),
  **`Words.noGamesLine`** ("no games yet") and **`Words.percent(charge)`** ("82%") (added).
- **`Help.entries(kind, {options: true})`** (added flag): Choose a console's Y named Sort and X Options, their actions
  `sort` and `consoleOptions`, the list's action lines its own; the entries' new **`action`** role names each entry's
  action. **`Actions.consoleOptions(key)`** (added): Settings opened at the console's page.
- **`GroupGridLayout`** (the kit's, by name; 1.5 in the qmldir): groups as a column and the chosen group's items as a
  grid, with the walk inside it (`EmuSen_Kit.md` §7.120), so a theme lays such a screen out without logic of its own.
- The module is 1.5 and `theme.json`'s `api` may name it; its registry was `Testing/qmltypes/EmuSen.Theme-1.5.*` until 1.8.
  A role is no QML type, so the qmldir's types keep their minors and `import EmuSen.Theme 1.0` finds what it found before.

## 1.4, an addition: the backdrop and the glow (2026-11-15)

Decided 2026-11-15 by the tester (R-20261115-6): Eclipse gets two kit parts themes may use, a blurred backdrop and the glow
or shadow. 1.3 is another piece's.

- **`Backdrop`**, **`BackdropField`**, **`Glow`** (the kit's, by name): a screen's ground with its blurred colour fields or a
  blurred picture, and a rounded box's glow or shadow, outside the box alone, with a box-shadow's offsets (`EmuSen_Kit.md`
  §7.118, §7.119). Each keeps 1.4 in the qmldir, so a theme importing 1.0 or 1.2 finds what it found before.
- **`consoleName`** (added to every game row): the console's name alone, the same name `facts` begins with, from the
  one owner of both (`qt-models/src/library.rs`); Eclipse's Home tile draws it under the title, as its board does.
- **`PadGlyph.Style`** (`Outlined`, `Filled`), the kit's glyph styles by name, and the kit's **`HelpBar.leadStyle`** and
  **`leadColor`**, the first entry's glyph drawn apart: Eclipse's filled A in the help line and the filled pill.
- **`Words.name`**, **`Words.playedFacts(at, seconds)`**, **`Words.continueLine(console)`**: the program's name in its own
  case, "EmuSen", Eclipse's wordmark, where `title` stays "EMUSEN" for the other screens; a game's last play and its time
  played, "Last played yesterday · 3 h 20 min played"; and "Continue · Super Nintendo" (`qt-models/src/words.rs`). The host
  now feeds `Session.played` from games.db's seconds played. Eclipse's Home draws all three, as its board does.
- **`Library.artHues(key, most)`**, **`Library.artHuesRead`**: a game's art's hues in degrees, most prominent first, read
  once a picture, and again when its file has changed since, by one reading thread off the GUI thread (pictures read one
  after another, so a library of covers starts no thread each) with `qt-models/src/art_hues.rs`; `artHuesRead` moves when a reading lands or the
  library is fed, so a binding asks again. Eclipse's Home colours its backdrop's fields and its disc's corona with them.
- **`Help.entries(kind, { card: true })`**: the kind's entries as Eclipse's Home names them, X "Options" and Start "Home
  card" (the card it opens), and no Select search in the bar, its action still the frame's (`qt-models/src/help.rs`,
  `card_named`; the words `words::OPTIONS`, `HOME_CARD`). The kit's `HelpBar.glyph`, `snapped` and `PadGlyph.letter` reach
  themes through the kit's types.
- **`Words.resumePoint(pausedSince)`**: "Resume point · saved 9:12 PM", the time of pausing in the machine's short form
  (`qt-models/src/words.rs`, `resume_point`; `qt-models/src/clock.rs`, `short_time`). Eclipse's Home draws it beside the
  paused picture, as its board does (Main.dc.html:59).
- **`Words.place(index, count)`** ("3 of 14", the game page's `place`, its one owner) and **`Words.gameArt(title)`**
  ("Game art · Aurora Drift", `qt-models/src/words.rs`, `game_art`): Eclipse's Home draws the shown tile's place among
  every game played beside Recent and names the art behind its disc under the status, as its board does
  (Main.dc.html:75, 80).
- **`Device.shortTime`**, **`Words.shortTime(at)`**: the time, and a moment in seconds since 1970, in the machine's zone
  and its locale's short form (`QLocale::system()`, `QLocale::ShortFormat`): "9:41 PM" under en_US, "21:41" under de_DE.
  `Device.clock` keeps the program's "hh:mm", so the other themes' clocks are unchanged (R-20261114-21). Eclipse's status
  draws `Device.shortTime` (Main.dc.html:49).
- The module is 1.4 and `theme.json`'s `api` may name it; its registry was `Testing/qmltypes/EmuSen.Theme-1.4.*` until 1.5.

## 1.3, an addition: the face tokens (2026-11-15)

Decided 2026-11-14 by the tester (R-20261114-15): a theme names its face through a token.

- **`Tokens.faceText`, `faceLabel`, `faceInterface`, `faceCode`** (added, read-only), as the kit's `Tokens` gained them
  (Kit 1.7, `EmuSen_Kit.md` §7.117): the family of the words, the labels, the kit's rows and controls, and codes. A
  `theme.json` may set each to a face the kit bundles: Barlow, Barlow Condensed, Sora or JetBrains Mono; theme-check finds
  any other.
- **`Tokens.weightTitle`, `weightSentence`, `weightWordmark`** (added, read-only): the weights of a title, the pane's
  sentence and the wordmark, as the kit's; a `theme.json` may set each as a whole number. Eclipse sets 600, 300 and 600,
  its boards' (`EmuSen_Kit.md` §7.117); the reference's Home reads `weightWordmark`.
- The reference's `parts/Faces.qml` reads `faceText` and `faceLabel` and loads no font of its own; its manifest declares
  api 1.3. Eclipse sets Sora for the first three and JetBrains Mono for codes, and declares api 1.3.
- The module is 1.3; `PlanetMark`, `CoronaGlow` and `CoronaDisc` keep 1.2 in the qmldir, every other type its own minor.

## 1.2, an addition: the corona disc and the planet mark (2026-11-14)

Decided 2026-11-14 by the tester (R-20261114-16): Eclipse's shared parts are kit parts, exported to themes as an addition.

- **`PlanetMark`, `CoronaDisc`, `CoronaGlow`** (the kit's, by name): a core's mark from its one owner, and the eclipse's
  dark disc with its rim and glows (`EmuSen_Kit.md` §7.115, §7.116). The reference theme's `EngineMark` is now a
  `PlanetMark`.
- **`Tokens.coronaColor`, `coronaRim`, `coronaGlowColor`, `coronaGlowBlur`, `coronaGlowSpread`, `coronaHaloColor`,
  `coronaHaloBlur`, `coronaHaloSpread`** (added, read-only), as the kit's `Tokens` gained them; a theme may override each
  in `theme.json`.
- `CoronaDisc`'s `corona` is a game's corona, transparent where none is given, and colours the rim and both glows
  (decided 2026-11-14 by the tester, R-20261114-38; `EmuSen_Kit.md` §7.116). `FocusRing`'s `glowReach` is gone.
- **`Tokens.ringGapped`** (added, read-only): every focus ring past its gap, as the kit's `Tokens` gained it (Kit 1.6,
  R-20261114-21); a theme may set it in `theme.json`, and Eclipse sets it true.
- The module is 1.2 and `theme.json`'s `api` may name it; the registry and qmldir are `Testing/qmltypes/EmuSen.Theme-1.2.*`.
  Every type of 1.0 keeps 1.0 in the qmldir and the three added are 1.2, so a theme that imports `EmuSen.Theme 1.0` finds
  what it found before; the module's minor alone, given to every file, had made `import EmuSen.Theme 1.0` find no type
  (seen 2026-11-14 in theme-audit's fixture themes: "ThemeScreen is not a type").

## 1.1, an addition: the focus ring (2026-11-14)

The tester's ruling of 2026-11-14 (R-20261114-16): Eclipse's shared parts are kit parts, exported to themes as an addition.

- **`FocusRing`** (added, qmldir): the kit's focus mark by name, a gap, the ring and the glow under them from the tokens
  (`EmuSen_Kit.md` §7.113). A theme places it around a focused box; the built-in themes' own rings are unchanged.
- **`Tokens.ringGapColor`, `glowColor`, `glowBlur`, `glowSpread`, `buttonRadius`** (added, read-only): as the kit's
  `Tokens` gained them (Kit 1.5); `Tokens` mirrors every kit token. A `theme.json` may override them.
- The module is 1.1 and `theme-check`'s API is 1.1; a package written for 1.0 still loads. `FocusRing` keeps 1.1 in the
  qmldir and every 1.0 type keeps 1.0, so `import EmuSen.Theme 1.0` finds what it found before and not `FocusRing`.

## 1.0, an addition: the raised shadow (2026-11-09)

- **`Tokens.shadowColor`, `shadowOpacity`, `shadowOffsetX`, `shadowOffsetY`, `shadowBlur`** (added, read-only): the one
  shadow of `EmuSen_Interface.md` §5.4, as the kit's `Tokens` gained them (`EmuSen_Kit.md` §7.55). `Tokens` mirrors every
  kit token, so the mirror gains them too. `RaisedShadow`, the kit's item that casts it for the page frame, is not
  re-exported.

## 1.0, an addition: `Words.now` (2026-11-09)

Decided 2026-11-09 by the tester.

- **`Words.now`** (added, a number): the time `ago`, `pausedFacts` and `pausedAgo` say their phrases against, in seconds
  since 1970; 0, the default, is the wall clock. The program sets it from its clock, which follows the wall clock unless
  a test holds it, so a theme's time phrases are held with the program's own (`EmuSen_Qt.md` §7.8). A theme reads it and
  never writes it; the moment a game paused (`Session.pausedSince`) is read from the same clock.

## 1.0, an addition (2026-11-09)

- **`Tokens.textSize`** (added, read-only): the text size by the word the setting stores, beside `textScale`, as the
  kit's `Tokens` gained it (`EmuSen_Kit.md` §7.47). `Tokens` mirrors every kit token, so the mirror gains it too.

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
- **`FigureLayout`** (the kit's, by name), the same day: a list model's groups packed as figures on a chart, every
  item's place, its figure's line and its walk, and the three pointers that close the first row (`EmuSen_Kit.md` §7.31).
  Constellation's Choose a console is a shelf packer over its makers, justified rows, a compact tier and a walk to the
  nearest body of the next line; QML may hold no function and the rows give no list of groups, so a theme could not lay
  it out without one. Additions only, in both registries.
- **`Help.entries(kind, {accept})` and an `enabled` role on the help's rows** (the same day): `accept: false` gives the
  kind's list with A's entry not `enabled`, which the kit's `HelpBar` draws at 38% and does not press; every other entry,
  and every list asked for without the flag, is enabled. A theme draws the disabled look of a place the host would
  decline (`available` false) with its bar's A dimmed, and could not mark one entry of a list it may not build. Each kind
  now has eight lists, made once. A delegate reads the role as `model.enabled`. Found by Horizon's disabled state.
- **`WheelLayout`** (the kit's, by name), the same day: a list model's groups as wheels turned under a fixed top and a dial
  of the screen's places and one stop per group, with a walk that holds still while the wheel turns, the slots by distance,
  the dots' count and the dial's clamped camera (`EmuSen_Kit.md` §7.32). Orrery's Choose a console needs one stop per
  maker and a walk across worlds and stops; QML may hold neither a function nor a list of groups. Additions only, in both
  registries.
- **`OrbitSlot`** (the kit's, by name), 2026-11-09: a cover of a list placed on an ellipse by its distance from the
  list's turning position, tilted and scaled, the covers one stack whose members the fit rule lets overlap in their
  order (`EmuSen_Kit.md` §7.33). Orrery's orbits overlap a neighbour under the apex by design; the overlap may only be
  declared by the kit's layout, never by a theme. Additions only, in both registries.
- **`OrbitLayout`** (the kit's, by name), 2026-11-08: a list model's groups on nested half-ellipses about the top centre,
  the smallest innermost, three shown and scrolled past, a group longer than its window turning under the focus, with the
  labels, the ellipses and a walk by steps (`EmuSen_Kit.md` §7.30). The reference theme's Choose a console needs the
  makers sorted by size and each orbit's window; QML may hold neither a loop nor a list of groups. Additions only, in both
  registries.
- **`EdgeFade`** (the kit's, by name), the same day: a scroller's faded edges, every theme's one fade. Meridian's rail
  faded through a `MultiEffect` kept off on the software backend, where its rows were cut instead; every theme that fades
  a scroller needs the same. Additions only, in both registries. Changed before release, 2026-11-09: `EdgeFade` holds
  what it fades (its children, a scroller placed inside it) and fades it to transparency, so what lies behind shows
  through, a picture included, on the GPU and on the software backend alike (`EmuSen_Kit.md` §7.35); `fadeTop` …
  `fadeRight` and `length` are as they were. The first form, standing beside a `source` and ramping a `backdrop` colour
  over it, was a fade to a colour, wrong over a picture, and no theme used it; `source` and `backdrop` are gone.
- **`CoverTile.drawnScale`** (2026-11-09): the scale a cover is drawn at, the tile's own `scale` unless set by a theme
  that scales an ancestor; the generated card's title keeps its size on the screen under it, so it stays at or above
  the legibility floor (`EmuSen_Kit.md` §7.29). Found by Orrery's far covers, drawn at 0.8.
