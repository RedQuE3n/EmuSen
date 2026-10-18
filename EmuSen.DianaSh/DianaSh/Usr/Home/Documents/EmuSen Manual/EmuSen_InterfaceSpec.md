# EmuSen_InterfaceSpec — the interface's specification, recorded from Mistress

Built 2026-10-15 as step K0 of `EmuSen_LunaP_Kit.md` (§7.3, §9.1). It records what today's Mistress shows in every state
its audits open, as structured text, so that a frontend on another toolkit is built against a written record of the
interface rather than against Avalonia's code, and is held to the same audit by reading the same files.

**Reading order.** §1 for what is recorded, §2 for the format, §3 for how the focus graph is found, §4 for the rules
as checks, §5 for recording and re-recording, §6 for the gate and what the recording found, §7 for what it does not
cover.

**Marks.** **[M]** measured on 2026-10-15, with how; **[A]** argued.

---

## 1. What is recorded

One file per state and size, in `EmuSen.WiseMan/Mistress/Specification/`, named `<state>-<width>x<height>.spec`:

- **Every window state the fit audit opens** (`WindowFitAuditTests`, `EmuSen_Settings_Reference.md` §4.83.2): its 36
  in-game and 15 themed cases, and `WindowFitScrapeAuditTests`' three scraping states, 54 in all, at 1280 × 800 and
  1920 × 1200. A window of tabs is recorded on every tab, as the audit audits it: the seven Controller Bindings cases
  give one file per console's tab, named `<case>-<tab>-<size>`.
- **Every Big Picture view**, at both sizes: the system scene and the gamelist scene of the synthetic theme, and over
  them the main menu (Start), the game options (Select), the metadata editor, Theme Settings, Game Collection Settings,
  the on-screen keyboard opened by Search, the launch screen and the screensaver's slideshow.

Each file holds, for its state:

- **the tree of controls**, as the audit sees it (§2.2): every control the window placed, and of a control's own
  template parts those that draw words, pictures or something to press, each with its kind, its name, its rectangle,
  its words, and the facts §2.4's rules read;
- **the focus graph** the pad walk finds from where the focus is when the state opens (§3);
- **which control each press reaches**: the four directions from every control in the graph, and the button grammar's
  pages, ends and back pressed in order afterwards, each with where the focus went and what the window then showed. A
  scene records instead a script of presses with the view, system and game each reaches, since the pad moves a scene's
  selection and not its focus.

## 2. The format

Plain text, one fact a line, deterministic in order, so that a change to the interface is a readable diff, as
DianaSh's golden outputs are (`EmuSen_DianaSh.md` §13.4). Numbers are written to two places; a value with a space or a
quote is quoted with C escapes. `InterfaceSpec.cs` (WiseMan, `SpecDocument`) reads and writes it and names no Avalonia
type, so it is the format's reader for any frontend.

### 2.1 The header

| Line | Meaning |
|---|---|
| `spec 1` | the format's version; a reader refuses another |
| `state`, `case`, `size` | the file's name, the audit case or view it records, and the window's size in pixels |
| `kind` | `sheet` (a framed window), `menu` (the pad menu's panel), `overlay` (drawn over a scene) or `scene` |
| `root` | the control recorded from, by kind and name |
| `unit` | the menu's scale: screen pixels to one design pixel (a screen pixel at 1280 × 800) |
| `slack` | the audit's slack in screen pixels: one, or the scale and a half above a scale of one |
| `floor` | the smallest text in design pixels the audit allows, 16 for a framed sheet, 0 where none is held |
| `frame` | the framed panel's rectangle, which "past its panel" is measured against |
| `title`, `footer` | the panel's words, with `cut` after them where the panel cuts them |
| `palette` | every colour the state's resources declare (§4.4) |
| `view` | a scene's view, system and selected game |
| `walk` | `truncated at N` when the walk stopped at its limit (§3), or `none` where the pad moves a view's own selection rather than the focus (the pad menu, the launch screen, the screensaver) |
| `entered` | how a state whose focus was not in it when it opened was entered before its walk: `shoulders` (a tab away and back) or `down` (§3) |

### 2.2 The tree

After `tree`, one control a line, indented two spaces a level, with its id (in depth-first order), its kind and its
facts as `key=value` or a bare flag:

| Key | Meaning |
|---|---|
| `role`, `heard`, `live` | the accessibility role, the name a reader speaks and a live region's politeness, from Avalonia's automation peers |
| `name` | the control's name in the window, the key a recording is matched on (§5.3) |
| `at`, `seen` | its rectangle in the root's pixels, and the part of it no clip hides where that differs |
| `text` | the words it shows |
| `element` | in a scene, the ES-DE element it draws, as `type:name` |
| `op`, `unseen`, `focus`, `off`, `sideways`, `part` | operable (what a player would want to reach), operable but a row a virtualising list keeps built out of view, focusable, disabled, takes left and right itself, one of a control's own template parts |
| `drawn`, `nohit` | drawn as words, a picture or a control a player presses; not hit-testable |
| `limit`, `by` | the rectangle the audit holds it to and what made it: `panel`, `clip`, `scroll` (a scroller that can bring vertical overflow into view) or `fixed-scroll` |
| `owner` | for words inside a button, a tab, a field or a row, the owner's id (or `@rectangle`) |
| `in` | the scrollers whose content it is part of |
| `overflow`, `viewport`, `fades`, `vfixed` | a scroller's overflow in pixels, its viewport, which edges fade (`tblr`) and whether it never scrolls vertically |
| `layout`, `box`, `ellipsis` | a text's laid-out width and height, the room it has, and whether it was cut with an ellipsis |
| `size`, `caps`, `face` | a text's size and the height of its capitals in design pixels, and the face it was drawn in |
| `paints` | the colours it and its unrecorded parts paint: background, foreground, border (§4.4) |
| `allow` | an allowance of the audit's that excuses it, by its reason (`WindowAllowances`) |
| `volatile` | why the control is recorded without its rectangle, words or measure, and without what is inside it (§6.1) |

### 2.3 Focus and presses

`focus <id>` names where the focus starts; `edge <from> <direction> <to>` is one press of the walk, `to` being a
control's id, `same` when the focus stayed, `none` when it was lost, or a reference to a control built only once
scrolled to: `row:<list>/<index>/<name>`, `item:<owner>/<index>/<path>/<name>` or `path:<path>`. `press <button> key=value...`
is a press of the grammar and what it reached: `focus` (the control, described), `top` (the sheet on top, or `none`),
`tab`, `title` and `menu`; or, in a scene, `view`, `system`, `game` and `sheet`.

## 3. The focus graph

`PadAudit.Graph` is the audit's own walk (`EmuSen_Settings_Reference.md` §4.48.5): breadth first over the four
directions from every control reached, each left from as the walk found it by replaying its path from the start,
lists' selections and scroll offsets put back first; left and right are not pressed from a control that takes them
itself. Two changes make it a recorder:

- **Every press is kept as an edge**, not only the paths that found something.
- **It stops when every operable control shown at the start has been left in every direction**, rather than exploring
  to its limit of 400. A long list's rows built only by scrolling are reached and recorded as references, but not
  exhausted: the first recording that explored to the limit took eight minutes on Shaders' 944-parameter preset alone,
  and 3 s with the stop **[M]**.

**A state whose focus is not in it.** A test that chooses a tab by setting it, as the audit's openers do, can leave
the focus outside the sheet, where no player's presses would leave it. Such a state is entered first, the way a
player's would be: one tab away and back with the shoulders, which leaves the focus where Mistress puts it on a tab
change; where that does not bring the focus in, a press down. Each recorded window of tabs is opened on its tab with
the focus on the tab's header, as choosing it by the pad leaves it. 20 of the 212 recordings were entered, 14 by the
shoulders and 6 by a press down **[M]**.

A control is keyed as the audit keys it: a row by its list, its index and the name a reader hears for it; a control
inside an item's container by the items control, the index, its way down and its name; anything else by its path of child indices from the window. The
last can change when a press rebuilds part of the window, which is why a control can appear in the graph twice, once
by id and once by `path:`: the walk sees it as two, and both are recorded.

**Why a row carries its name (2026-10-15).** Until then a row was its list and index alone. A list that another
selection refills, such as the Cheat Database's games, which follow the system chosen beside them, then had one key for
the third row of every system's list, and the walk explored each key once, in the state it first found it in. At
1280 × 800 it first reached the games list from the Nintendo Entertainment System's row, whose list holds two games, so
down from the second row left the list, correctly for that list; the Super Nintendo's four were never walked, and the
recording reported their third row unreachable (§6.3.1). With the name in the key a refilled list's rows are rows the
walk has not seen, explored in their own state. A row whose name a reader cannot hear keys as before, by list and index.

### 3.1 A window that reads after a selection: the shader settings (2026-10-18)

**The failure.** On 2026-10-18 two combined runs each failed one Shaders recording, a different one each time:
`ShaderSettingsPlayStation` at 1280 × 800, then `ShaderSettings` at 1920 × 1200, whose line 134 had recorded
`edge 75 down path:0/3/0/0/2/3/…` (In Use, down, to a control of the parameter area) and now read `edge 75 down 30` (a
row of the shader list on the other side of the window). The window had changed in that merge only in where it reads
its consoles and a console's host, both now from the runtime catalog, whose answers `RuntimeCatalogTests` holds equal
to the catalog's.

**Not a change of the window.** The twelve Shaders recordings were run ten times each at the commit before the merge
(`2493ce6e`) and ten times at the merge (`701dade7`), alone: 120 of 120 passed at both **[M]**. Re-recording would
have recorded one timing.

**A dependence on the clock, shown.** The panel shows a shader's parameters once the selection has held still for its
settle, 120 ms, by a timer (`EmuSen_Settings_Reference.md` §4.48.9 and §4.48.10); a move after a pause reads at once.
So the walk's state at a press depended on whether a timer had fired since the last move, which depends on how fast
the walk ran. Forcing it, with the settle set to 1 ms so every settle fires before the walk's next step, all twelve
recordings differed, among them `edge 41 right path:0/3/0/0/2/3/…` where `75` was recorded, the control of the
failure above appearing where it had not **[M]**. A second dependence is the read itself: a preset's parameters are
read on the thread pool, and the two presets beside it are read ahead, at most two at a time, the rest dropped. With
the pool kept busy (twice as many three-second sleeps as threads, queued as the walk began), `ShaderSettingsSliders`
recorded `edge 71 right 25` where `"item:88/2/0/0/1/Scanline weight"` was recorded, in both runs **[M]**.

**What the walk now does.** The audit's opener, once a Shaders window is up, waits until every panel has built its
sliders and no read-ahead is running, then sets the window's settle to a day, so that in the walk only the first
keyed move reads at once and every later one is left settling, as a quick player's presses leave them, never as the
clock happens to. `PadAudit.Graph` takes an optional `settled` condition, which the walk waits on each time it puts
the lists' selections back before replaying a path; for this window it is that no read and no read-ahead is in
flight (`ShaderPanel.ReadInFlight`, which a selection still settling does not count, and
`ShaderSettingsWindow.PrefetchesRunning`). A wait longer than 20 s fails the case with its reason, rather than
recording what was there. The three openers that waited for a preset's sliders with a loop of 2,000 one-millisecond
sleeps, about two seconds, now wait on the same condition with the audit's `WaitFor`.

**After.** The twelve recordings pass unchanged: 10 runs of 12 alone **[M]**, and 2 runs of 12 with the pool kept busy
as above **[M]**. Before the read-ahead was part of the condition, the busy pool still turned the Sliders edge in 2
runs of 2, which is how it was found.

## 4. §2.4's rules as checks

`InterfaceRules.cs` (WiseMan) reads only a `SpecDocument`, so a recording from any frontend is held to them.

### 4.1 The fit audit

`InterfaceRules.Fit` is `FitAudit.Check` (§4.83.2 of the settings reference) restated over recorded facts: past its
limit (excused only along a scroller that can scroll that way), words past their owner, overlap of two drawn,
hit-testable controls neither inside the other by more than the slack both ways, a scroller in under two rows of 42
design pixels, an edge that slices a drawn control without fading, an ellipsis, words past their box or cut at its foot,
capitals under Inter's at the floor, and a cut title or footer. **It is held to `FitAudit.Check`, not trusted**: for
every recorded sheet the two must report the same kinds of fault, in the same numbers, and the same holds over
`FitAuditTests`' sheets, each built with a fault on purpose, so every planted fault the live audit catches the recorded
one catches too.

### 4.2 Pad reachability

`PadReach`: from `focus`, the recorded edges reach every control marked `op`. A control marked `unseen` (a row kept
built out of view) is not required, as `PadAudit.Operable` does not require it.

### 4.3 Accessibility names

`Named`: every operable control has a name a reader speaks (`heard`), from `AutomationProperties.Name`, the control
that labels it, or its own words, as Avalonia's automation peer resolves it.

### 4.4 Palette reach

`PaletteReach`: every colour a control paints (its background, foreground and border, as LunaP's `PaletteSweep`
reads them, its unrecorded template parts' colours counted as its own) is one the state's resources declare: the
application's resources and styles, every resource and style scope from the root up and within it (where the menu look
puts its own), and `LunaPalette`'s colours. A scene is exempt: its colours are its theme author's, and it is held to
ES-DE's captures instead (`EmuSen_BigPicture.md` §13.8).

### 4.5 Faults that stand

`WindowFitAuditTests.Known` lists the faults the rules find in today's Mistress, each with why it stands; the tests
pass over them and fail on any other, and an entry is removed when its fault is fixed. §6.2 lists them. Since 2026-10-15 it is
empty: every fault §6.2 lists was worked through (§6.3), and a new one fails the tests until it is fixed or entered
with its reason.

## 5. Recording, reading and re-recording

### 5.1 The tests

| Test (WiseMan) | What it does |
|---|---|
| `WindowFitAuditTests.Every_audited_window_state_is_as_its_specification_records_it` | opens each window state as the fit audit does, records it and holds it to its file and the rules |
| `WindowFitScrapeAuditTests.Every_scraping_state_is_as_its_specification_records_it` | the same for the three scraping states |
| `WindowFitAuditTests.Every_big_picture_view_is_as_its_specification_records_it` | the same for the ten views |
| `WindowFitAuditTests.Every_state_has_a_recording_and_every_recording_a_state` | the files and the cases agree, both ways |
| `WindowFitAuditTests.A_recording_reads_back_whole_and_holds_to_the_rules` | each file parses and writes back to the same bytes, and holds to the rules read from the file alone |
| `WindowFitAuditTests.Another_frontend_s_recordings_match_these_within_the_tolerances` | with `EMUSEN_INTERFACE_SPEC_OTHER=<folder>`, another frontend's files compared with these (§5.3) and held to the rules; without it, nothing |
| `FitAuditTests` (every case) | the recorded fit rule finds the kinds of fault `FitAudit.Check` finds (§4.1) |

### 5.2 Re-recording

`EMUSEN_INTERFACE_SPEC=record` makes the three recording tests write their files instead of comparing, as
`EMUSEN_DIANASH_GOLDEN=record` writes DianaSh's corpus. A change to the interface is re-recorded on purpose and its
diff read before it is committed; without the variable a state that differs from its file fails at the first line that
differs, printing both.

### 5.3 Another frontend against these

`InterfaceSpec.Compare` holds a recording to Mistress's within §7.3's tolerances: every named control present with the
same role and words, each rectangle within two design pixels, no named control extra, and the same focus graph read by
names. Ids, template parts, unnamed panels and pixel colours are not compared: two toolkits build different trees for the
same screen, and the rules (§4) hold what matters of them.

## 6. The gate, and what the recording found

### 6.1 The gate

**It covers every audited state at both sizes and every Big Picture view [M].** 232 recordings: the 47 window cases
without tabs and the 3 scraping states at two sizes (100), the seven Controller Bindings cases on each of their eight
tabs at two sizes (112), and the ten views at two sizes (20): 116 states, each at both sizes. Together they hold 9,368
controls and 8,802 focus edges, 2.1 MB of text, as re-recorded on 2026-10-16 (§6.3.7, §6.3.8); the first recording had
212 files and 8,284 controls. `Every_state_has_a_recording_and_every_recording_a_state` holds the
files to the cases both ways.

**Two recordings in a row are identical [M].** The whole set was recorded twice in a row, into the repository and into
a second folder (`EMUSEN_INTERFACE_SPEC_DIR`), and the two compared with `diff -r`: no difference in 212 files. The
first pairs of recordings were not, and each difference was traced and removed:

| What differed | Its source | Removed by |
|---|---|---|
| paths in Cheat Database's folder field and status line, Theme Detail's state line, and every test's own folders | the run's temporary folders, named by a random identifier | the text written with `<run>`, `<temp>` and `<home>` in their place; where the words' **width** also changed with the identifier, as a proportional face draws `a` and `1` differently, the control is recorded as volatile |
| the time a game was left, in Continue Playing | the state file's time, from the wall clock | the audit's opener now sets the state file's time to 2026-10-15 09:30 before the question opens |
| the time the theme list was fetched | the wall clock, inside the theme browser | the line recorded as volatile |
| the day a theme was downloaded, in its About sheet | the wall clock, when the theme is installed | the statement recorded as volatile (§6.3.7) |
| the rewind reel's moments and their "1.55 s ago" | the emulation thread, run against the wall clock | the reel's strip recorded as volatile, without its moments |
| where the focus was after Back on the search keyboard | work the keyboard posts on closing, done or not when the press was read | a scene's presses read after 200 ms of its clock |
| a walk that explored Shaders' 944 rows to its limit of 400 | not a difference, but eight minutes a state | the walk's stop (§3) |

A volatile control keeps its kind, name, role, the colours it paints and, where it does not come from the volatile source,
the name a reader speaks; it loses its rectangle, its words and its measure, and nothing inside it is recorded. Eight
controls in seven states are volatile at each size **[M]**. What they show is the part of the interface a recording cannot fix; the
rules (§4) read them where they can and skip them where they cannot.

**The rules, against today's Mistress.** The recorded fit rule finds exactly what `FitAudit.Check` finds in every
recorded sheet (none, as the audit requires) and in every one of `FitAuditTests`' planted faults. The other three rules
found what §6.2 lists; since §6.3 they find nothing.

### 6.2 Faults the rules found

These are what the rules found, when the specification was first recorded, that no audit had looked for. Each stood
in `WindowFitAuditTests.Known` with its reason, and `Every_fault_recorded_as_standing_still_stands` fails when one
stops being found, so the table cannot outlive its faults. The recording step changed none of them; each was worked
through on 2026-10-15 (§6.3).

| Rule | What | Where | Worked through |
|---|---|---|---|
| Pad reachability | down from the games list's second row left the list; the third row, showing 5 pixels, and the fourth were reached by no direction | Cheat Database at 1280 × 800 | the walk's, not the window's (§6.3.1) |
| Accessibility names | 10 choices whose label is drawn beside them but not given to them; 4 big-screen option rows and 7 metadata rows whose label `MenuRows` draws; 4 text boxes labelled only by a placeholder | Graphics Settings, Memory Cards, the big-screen menus, the metadata editor, Active Cheats, Cover Picker, Find by Name | named (§6.3.2) |
| Palette reach | `#55555A` on the big-screen menus' push buttons, `#FFFFFFCC` on a focused dropdown, the screensaver's box and the launch screen's card painted as literals | the big-screen menus, Memory Cards, the N64's tab, the screensaver, the launch screen | tokens (§6.3.3) |
| Fit (a scene) | the synthetic theme's help bar asks for every entry in one row and is laid out wider than the screen | the gamelist scene, both sizes | the test theme's (§6.3.4) |

### 6.3 The faults of §6.2 worked through (2026-10-15)

Each was proven before it was changed, and the recordings re-recorded after; `git diff` of the `.spec` files shows the
interface each change made.

#### 6.3.1 The Cheat Database's games: the walk's, not the window's

**What was reported.** Down from the games list's second row went to Download from libretro; the third row, showing
5 pixels, and the fourth were reached by no direction.

**What a player meets [M].** `The_cheat_database_s_games_are_walked_to_the_last_by_the_pad`, at both sizes: from the
folder box, down three times to the Super Nintendo's row, right into its four games and down three times reaches and
selects each row in turn, the third scrolled into view from its 5 pixels. Mistress was right before the change; the
router sends Down to a list's row unless it is the last (`PadWindowRouter`), and Avalonia's list brings the next row
into view.

**What the walk did.** A row was keyed by its list and index (§3). Breadth first, the walk first reached the games list
by right from the Nintendo Entertainment System's row, where the list holds two games; down from the second of those
correctly left the list. Each key is explored once, so index 2 was never explored, and the Super Nintendo's third row,
recorded in the tree, had no edge to it. At 1920 × 1200 the walk reached the Super Nintendo's list first by up from the
buttons, which is why only the smaller size reported it.

**The fix** is in the walk: `PadAudit.KeyOf` keys a row by its list, index and the name a reader hears, so every
audit that walks (the pad audits and the recorder) treats a refilled list's rows as new. The re-recording changed six
files: the Cheat Database at both sizes, where `44 down 46` and `46 down "row:40/3/Cobalt Harbor (USA)"` now stand and
the other systems' rows appear by name; and the Shaders sheets with sliders and a long parameter, whose `row:` edges
gained their names. No other list in a recorded state is refilled by another control's selection, so no other graph
changed.

## 7. What it does not cover

- **Only the states the audits open.** A state no case opens is not recorded, as it is not audited (§4.83.5 of the
  settings reference).
- **Pixels.** Two rasterisers never agree to the pixel; a scene's look is held to ES-DE's captures, and a sheet's to the
  rules.
- **Animation.** Every state is recorded at rest; durations are ES-DE's measured ones (`EmuSen_BigPicture.md` §14.7) and
  stay with Big Picture's own tests.
- **Presses that change the screen.** A, Start, Select and North are recorded only where they open a state that is
  itself recorded; their effects on settings are other tests' business.

#### 6.3.2 25 controls a reader could not name

**Before [M].** `Named` found 10 choices whose label is drawn beside them (Overscan and Picture Shape on four consoles'
tabs, the N64's Antialiasing, the PlayStation's card), 4 big-screen option rows and 7 metadata rows whose label
`MenuRows` draws, and 4 text boxes labelled only by their placeholders.

**Where each belonged.** The first ten sit in a `FieldRow` whose content is a panel: the choice with a note under it,
or beside the Shaders button. The row lent its label to the panel alone. LunaP's `FieldRow` now lends it on to the
dropdowns, text boxes, sliders and number boxes inside, not to a button or switch, which have words of their own
(LunaP's `docs/LunaP.md` §200.1). The next eleven are rows `MenuRows` draws from a label it was given and never told a
reader; `MenuRows.Label` is now the name, below any the caller sets (§200.2). Those 21 are fixed once, in the two kit
controls, and every other window built from them gains the same. The last four have no visible label but a
placeholder, and the window names them with the same words: "Cheat code" and "Cheat description" on every console's
tab of Active Cheats, "Search the library" and "The game's name". A placeholder vanishes as the player types, so the
toolkit does not use one as a name on its own (§24.2 of LunaP's page).

**After [M].** No recording has an operable control without `heard`. The picture shape's dropdown hears "Shader and
shape", its row's label, which is the words beside it; four main-menu rows and Create New Custom Collection now hear
the label their rows draw ("Library", not "Library  ›"). The presses that land on a named text box describe it with
its name.

#### 6.3.3 Four colours no theme could reach

**Before [M].** `PaletteReach` found `#55555A` on the big-screen menus' push buttons, `#FFFFFFCC` on a focused
dropdown, and the screensaver's box and the launch screen's card painted from literals.

**Where each belonged.** The menu look's colours were resources only under an element the look was applied to; a
menu drawing itself had none in reach. LunaP's `Theme/MenuPalette.axaml` now holds them, every `MenuPanel` merges a
copy, and `MenuRows.ApplyButton` paints from the theme's button keys (§200.3 of LunaP's page). The focused dropdown's
words and arrow came from two stock keys the look never declared; they are now the look's bar text, `#EEEEF0`, which
is the one change a player can see. The launch screen merges the same palette and paints its card and words from
`LunaSurface` and `LunaSectionHeader`, the same values as before. The screensaver's box is ES-DE's black at two
thirds; no token held it (`LunaHudSurface` is a notice plate of another measured colour), so LunaP gained one,
`LunaCaptionSurface`, and `Screensaver.BoxColour` is gone.

**After [M].** No recording paints outside its palette. Every sheet's `palette` line gained the menu palette's
colours, since each sheet's frame is a `MenuPanel`; the big-screen menus' rows record the look's colours for their
hidden chrome (a row is drawn by `MenuRow`, which does not read them), and the two focused dropdowns `#EEEEF0`
where they had white.

#### 6.3.4 The help bar wider than the screen: the test theme's

**What it was.** The synthetic theme the Big Picture tests write (`ThemedSession.Write`) asks for its help bar at
3% from the left, every entry (`<entries>all</entries>`) in one row, at a font of 3.5% of the height. The gamelist's
twelve entries then measure 1,650 pixels at 1280 × 800 and 2,475 at 1920 × 1200, from 38 and 58; the system view's
four, 630 and 946, fit. Mistress lays a help bar out as the theme asks, in one row from its position, as ES-DE does;
it does not shrink or wrap a bar a theme sized, and a real theme chooses its entries to fit (Art Book Next's gamelist
asks for two, `back,a`).

**Proven the theme's [M].** With nothing changed but the theme's font, 2.5% of the height, the same bar measures
1,230 and 1,846 pixels and ends 11 and 17 pixels inside the screen, and the fault is gone at both sizes; the
system view's bar shrinks with it. The theme is a fixture, so it is the theme that is changed: it still asks for every
entry, at a size that fits. The re-recording changes the help bar's rectangle in the four scene recordings and
nothing else.

#### 6.3.5 Two tests that counted the consoles before the PlayStation was offered

`ThemedSwitchesTests.The_systems_follow_the_chosen_order_and_the_startup_list_with_them` (three cases) expected seven
choices for System on Startup and got eight, and `PlayerBindingsWindowTests.Each_console_offers_as_many_players_as_it_has_ports`
expected five consoles' ports and got six. Both lists are every system the frontend offers, whether the library holds
its games or not (the Sega Genesis is among the seven with no Genesis game in the test's library).

**Measured on the commits either side [M]**, each built in a scratch checkout with its Rust cores: on 6cd5627f, the
last before the PlayStation was offered, both tests pass; on cf4554c9, "Diamond's stage 11 gate, and the PlayStation
offered to players", both fail, the startup list's eighth entry being "Sony PlayStation" in each order and the sixth
console's count 2. The expectations were stale from that commit, not a defect; they now count the PlayStation, and the
startup test also asks for it by name.

#### 6.3.6 Two goldens recorded before the PlayStation was offered and Mercury's audit

**`core.golden`** (DianaSh's corpus, recorded 2026-10-10) lists, under `core nosuch x.sfc`, the cores `core` knows.
Re-recorded with `EMUSEN_DIANASH_GOLDEN=record`, its only change is that line, which gained `diamond` and
`playstation`, and its byte count (110 to 132).

**`CoreRegistrationGolden.txt`** (`CoreRegistrationEquivalenceTests`) still held the Game Boy's state version 8; the
Mercury audit's merge (9c0b8721) made it 11. Re-recorded with `EMUSEN_RECORD_REGISTRATION=1`, its 40 changed lines are
all `factory[GB …].state-version`, 8 to 11, in the four sections that record it; nothing else moved.

#### 6.3.7 Seventy failures after the Sega CD was offered (2026-10-16)

At 19427372 `WindowFitAuditTests` failed 70 of its 474 cases, 35 at each size. **Measured [M]**, each commit built in a
scratch checkout with its Rust cores and run on the audit alone, bisecting WiseMan's first-parent history from
63644fa3, the last recording:

| Commit | Failures | What failed |
|---|---|---|
| 63644fa3, e8964dfa, 8f2aafab | 2 | ThemeAbout's recording at both sizes |
| 4e8be3d2, nephrite-s10: the open Sega CD BIOS and mode 1 | 12 | and the Genesis's graphics tab; both Memory Cards cases, in both tests |
| a3c5dff0, mcd-bram-spec, and 4a453787, dianash-s3 | 12 | the same 12 |
| d84975d9, nephrite-s10: the Sega CD offered to players | 70 | and every console tab strip, Collection Settings, the Cheat Database and the firmware page, the same 70 as 19427372 |

Four causes. Only the first was a window drawn wrong.

1. **The firmware page's footer was cut [a fault].** The three Sega CD BIOS rows reached the big-screen firmware
   menu when the Sega CD was offered, and the first sentence of their replacement's words, which the footer quotes, ran
   the footer past its two lines at both sizes. The words were reordered so that their first sentence is short; the
   footer is unchanged (`Nephrite_OpenBios.md` §17).
2. **The recordings were stale [not a fault].** 4e8be3d2 declares the backup RAM of a Sega CD beneath a cartridge
   (mode 1) as file 11 of every Genesis cartridge, so the Genesis gained a console medium: an Internal Backup RAM row on
   its graphics tab and a Genesis tab in Memory Cards. d84975d9 offers the Sega CD (`development` false in the core's
   info), so a Sega CD tab joined Graphics Settings, Shaders, Active Cheats, Controller Bindings and Memory Cards, and a
   Sega CD entry joined Collection Settings' rows and the Cheat Database's systems. Re-recorded with
   `EMUSEN_INTERFACE_SPEC=record`, the diff, read with the controls' ids taken out, holds those and what follows from
   them alone: the new tabs and rows; the PlayStation's tab moved right by the Sega CD's (109 design pixels at
   1280 × 800, 163.5 at 1920 × 1200); the lists and scroll extents that grew; the focus edges and paths numbered around
   the new controls; and the presses that now reach the Sega CD's tab. The seven Controller Bindings cases each gained
   a recording of their Sega CD tab at both sizes (14 files). The fit audit finds nothing in any of them.
3. **The Memory Cards cases opened the wrong tab [the fixture's].** The window opens on the running game's console, else
   on its first tab. The cases run an SNES game, and until 4e8be3d2 the PlayStation was the only console with a medium,
   so its tab was the first. With the Genesis's and the Sega CD's before it in release order, the window opened on the
   Genesis, and the fixture's check for the PlayStation card's second save failed before either test looked at the
   window. The fixture now chooses the PlayStation's tab, as its description always said.
4. **ThemeAbout's recording held the day it was made [the recorder's].** The About sheet says "Downloaded at your
   request on" the day the test installs its theme. Recorded on 2026-10-15, it failed on every later day: on 63644fa3,
   run on 2026-10-16, its only difference was the date and the 0.02-pixel width that came with it. The statement is now
   recorded as volatile, as the theme list's fetched line is (§6.1).

Outside the audit, the same merge made `FirmwarePageTests.The_tab_shows_every_system_and_each_file_s_words_from_the_overview`
throw: it found each file's row by its title alone, and the TMSS ROM's row now stands under both the Genesis and the
Sega CD, as each declares it. The page is right to show it under each; the test now holds every row of a title to the
file's words, and expects one per system that declares it.

The Memory Cards window's Genesis and Sega CD tabs were opened by no case; §6.3.8 adds them.

#### 6.3.8 Three later merges, and every Memory Cards tab audited (2026-10-16)

Three merges reached WiseMan after 19427372, and the recordings were taken again over them: a8c114cf (segacd-player),
950d14c5 (dianash-s6) and 5000bc5a (marsrt-v1). Read with the controls' ids, their rectangles and the focus edges taken
out, the differences are these, and nothing else **[M]**:

| Recording (both sizes) | What changed | From |
|---|---|---|
| ShaderSettingsGenesis, ShaderSettingsPlayStation | a CRT (Lottes) row in the built-in list, its count 4 to 5, and the sheet 44 design pixels taller at 1280 × 800 | a8c114cf, CRT (Lottes) offered to the Genesis and the PlayStation |
| MemoryCardsPlayStation, MemoryCardsPlayStationHeld | an N64 tab, first in release order | 5000bc5a, the Controller Paks as save media |
| GraphicsSettingsN64Note | the N64 tab 213 design pixels longer above the note, so the note is scrolled further | 5000bc5a, the Controller Pak rows |
| GraphicsSettingsGenesis | "one backup ram" now "one backup RAM" in Internal Backup RAM's help | `MediumWords.InText` (`EmuSen_SaveMedia.md` §6.2) |

950d14c5 changes no recording. 5000bc5a added the case `MemoryCardsN64` but no file for it in `Specification/`, which
`Every_audited_window_state_is_as_its_specification_records_it` requires; its two files are recorded here.

Two cases are added, `MemoryCardsGenesis` and `MemoryCardsSegaCD`, each opening the window on that console's tab with a
formatted backup RAM in the drawer under the name its slot holds. With the PlayStation's two and the N64's, every tab the
window has is audited. The fit audit and the recorded rules find nothing in any of the 500 cases at either size.

Outside the audit, `memcard.golden` (DianaSh's corpus) was re-recorded for that file alone with
`EMUSEN_DIANASH_GOLDEN=record`. Its two changed lines are `memcard ls notes.txt`'s list of known media, which gained
".mpk (Controller Pak)" with 5000bc5a, and `memcard new`'s "a formatted backup RAM cartridge", which said "backup ram".
