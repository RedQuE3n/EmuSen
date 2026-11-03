# EmuSen_Interface — the unified interface: structure, grammar, visual language and build order

Decided 2026-11-01 by the tester. This is the written specification of EmuSen's interface as one design: the places a
player moves between, the one frame every screen is built from, the button grammar, the visual language (colour, type,
spacing, focus, motion, sound, art and video), the three forms it is rendered in, how it shares the screen with an ES-DE
theme, and the order it is built in. It supersedes the in-game and library menus as separate designs; the Pause Card
(§2.8) and the Library Card (§2.10) are folded into it.

**What it does not repeat.** How a state of the present interface is recorded and audited is `EmuSen_InterfaceSpec.md`;
every stage below is held by those recordings and audits (§10). The frame's roles and the tools on it are
`EmuSen_LibraryTools.md` §1 (§1a one description in two forms, §1b tools as libraries with Home the hub, §1c the
family); this document builds on them and cites them rather than restating them. Big Picture's themes, scenes and their
measurements are `EmuSen_BigPicture.md`; the toolkit's side of every token is LunaP's `docs/LunaP.md` §206.

**Reading order.** §1 for the idea, §2 for the structure, §3 for the frame, §4 for the six screens, §5 for the visual
language, §6 for the forms, §7 for themes, §8 for accessibility, §9 for what becomes of what is built, §10 for the
build order, §11 for stage U1 as built, §12 for stage U2 as built, §13 for stage U3 as built, §14 for the sources, §15 for Cores
redesigned, §16 for the family's screens held against the grammar, §17 for the desktop, §18 for the remaining windows. A
bracketed number cites a source in §14.

**Units.** One design unit is one pixel at 1280 × 800; the big-screen form scales with the screen's height (× 1.5 at
1920 × 1200, on a monitor and on the handheld alike), as `MenuPanel.Scale` does. Desktop measurements are device pixels
at 1× for a 1280 × 800 window.

**Marks.** **[D]** decided 2026-11-01 by the tester; **[M]** measured, with the date; **[A]** argued.

---

## 0. The decisions of 2026-11-01

The design study's seven open questions and one addition, as the tester answered them:

1. **[D]** On a game, **A plays it and X opens its page**: the shortest path to play, as Nintendo and Microsoft argue
   [3][18]. Steam's alternative (A opens the page with Play focused) was not taken.
2. **[D]** **Power ▸ holds Exit Big Picture and Quit EmuSen.** The device's sleep and shut down appear only where
   EmuSen is the device's shell, never in Steam's Game Mode, where Steam owns power.
3. **[D]** **Barlow everywhere, the desktop included**: Barlow Condensed for display words and titles, Barlow for the
   rest. This overrides the study's proposal of Inter on the desktop. The desktop keeps its own type scale (§5.2).
4. **[D]** **A light variant is offered on the big screen too**, as Switch and Ozone offer one [4][16]; the big screen
   is dark unless the player chooses light.
5. **[D]** **Navigation sounds are on by default at 60%.**
6. **[D]** **Home's background art comes from the focused tile**, on by default, with a switch to turn it off.
7. **[D]** **Themes never style a card**, not even with an accent (§7).
8. **[D]** **Video is part of the design from the start** (§5.9), not added later.

The remaining windows' eleven questions (§18.15), decided 2026-11-01 by the tester:

9. **[D]** Over EmuSen's own look **Start goes to Home the screen**, and no card is drawn over the grid: one Home in the
   own look, one card over a theme (§2.2, §2.9).
10. **[D]** **Compressed Discs retires into Packages**: Convert to CHD is the package's action, the discs a filter of
    the List card, the batch's outcome the apply bar's (§18.0).
11. **[D]** **The Shell leaves Home and the Home card**, reached from Settings ▸ EmuSen ▸ Shell ▸ and the desktop's
    Tools menu; `vstop` is a view of the Shell window (§18.9).
12. **[D]** **A on Rewind opens the reel page** rather than rewinding at once; a held Left scrubs (§18.5).
13. **[D]** **The themed folder's link moves to the List card ▸ Folder ▸**, and the game's page's five pages replace the
    metadata editor, the cover picker, the scrape chooser, the game package window and the screenshot window (§18.2).
14. **[D]** **The launch screen and the slideshow are EmuSen's under every theme**, replacing ES-DE's measured ones
    (§18.3).
15. **[D]** **Memory cards get two doors from one description**: the game's Game Options and the console's settings
    (§18.5, §18.10).
16. **[D]** **The Resume question is the question form**, `Continue` first (§18.3).
17. **[D]** **Collections are a tool and a section of the library**; the automatic ones listed and greyed (§18.4).
18. **[D]** **One notice at a time, for 2 s** (§18.6).
19. **[D]** **Setup's five steps**: ROM folder, Firmware, Controllers, Theme, ScreenScraper (§16.2;
    `EmuSen_LibraryTools.md` §1b).

Cores, decided 2026-11-02 by the tester ("Yes to all", and "In core window, we should also sort by manufacturer"):

20. **[D]** **The catalogue is read once.** A home that has never read libretro's catalogue reads it when Cores opens,
    with its progress in the apply bar; a failure is said in the header's facts (`catalogue could not be read: <why>`)
    and in the error log. A home that has one never reads it again unasked; checking when Cores opens stays opt-in and
    off (`EmuSen_Cores.md` §3.8, §11 decision 11).
21. **[D]** **A console's name in a list is the systems table's `display_name`** (`Super Nintendo`, `Nintendo 64`), one
    owner beside `console`, which stays the settings' key (§15.3; `EmuSen_Cores.md` §11 decision 12).
22. **[D]** **Cores sorts by maker**: the strip and the rows by maker in the order the systems table first names them
    (Nintendo, Sega, Sony, then the rest), within a maker by `release_year`, the table's order breaking a tie; within a
    console the default first, then the installed cores that can run, then by name. Each maker is a heading above its
    first row, in Caption Muted upper case, a band of 32 units that the pad never lands on, drawn only while the rows
    shown hold more than one maker (§15.3; LunaP §211.1).
23. **[D]** **The frame is held to the mockups**, not Cores: full-width rows with the chosen one raised and its 4-unit
    accent bar, no empty strip above the first row, the title, strip and list on the grid of §15.2, `Apply…` with its
    Start glyph beside the meter, the help bar `A · B · Console · More · List · Apply`, and the search box's word
    `Search` (`EmuSen_Cores.md` §3.8; LunaP §211.2, §211.3).

**Departure from 3.** Decided 2026-11-01: a framed sheet's words stay in Barlow Condensed at their present sizes until
the Card form rebuilds the sheets in U2 and U3, where the departure ends. Set in Barlow at the big screen's Body, the
sheets failed the fit audit in 23 states at both sizes (§11; LunaP §206.3).

---

## 1. The idea, and the principles it rests on

EmuSen should feel like a console that happens to hold every console. Switched on, the game last played is in front of
the player, one press from resuming; everything else is one layer behind the game, reached by the same few buttons
wherever the player is, and it gets out of the way the moment a choice is made. The library is shelves of art, not
folders of files. A game has one page that holds everything about it. Tools and settings are places with names a player
understands and a line that says what needs doing, never a list of components. Every screen shares one frame, one type
scale, one focus mark and one set of sounds, so nothing is learned twice; and when a player chooses an ES-DE theme, the
theme owns the library's two views and nothing else. RetroArch has been called "a program about cores, not about
games" [1]; this design is the opposite.

1. **Play first, one press.** Sony's Control Center gives "immediate access to almost everything you need from the
   system" from one press, over the game [5]; Xbox's 2023 Home put Quick Resume on the screen and "core features no more
   than a couple of button presses away" [8]; the Steam Deck opens on "continue playing recent games" [10]; Nintendo
   took the NES, which launched a game at power-on, as the ideal of speed [3]. The first row of every screen is the way
   back to play.
2. **A game is a place.** "Each game has its own hub, which collects everything related to that game" (PS5 [6]).
   Analogue's library shows a game's developer, publisher, release date, time added and play time, and lists its
   Memories with dates, pinnable [12][13].
3. **Few steps, few words, plain words.** Nintendo kept the HOME menu under 200 KB, its animations "as short as
   possible", asked "are you sure" sparingly and preferred text to "icons whose meaning isn't clear" [3]. RetroArch's
   critics ask "Load core… What is a core?" and want "per-core, per-directory, per-game settings" rather than
   "overrides" [1][2]. Things are named by what they do for the player.
4. **Depth and breadth are both costs; status pays for a row.** RetroArch's users complain of "too many categories"
   and of "too many nested submenus" at once [1]. The answer is fewer things, each saying what it holds: Switch's bottom
   row of named places [4], Xbox's quick-access row [8]. Home shows at most eight destinations, each with one live line.
5. **One grammar.** RetroArch ships four menu drivers over one structure [15][16][17]; the skin changes and the
   confusion stays. Microsoft asks that focus be "clear and unmistakable" and movement "consistent and predictable"
   [18]. One frame (§3) and one button grammar (§2.1) everywhere.
6. **Legible from the sofa, dense as a phone.** Microsoft: information on a television "comparable to what you'd see on
   a mobile phone", nothing essential within 5% of the edges [18]. Amazon: at least 28 px body text at 1080p, the outer
   5% clear [19]. Apple: body 29 pt, a 23 pt minimum, "avoid light font weights" [21].
7. **Focus is the interface.** Apple: a focused item "stands out… through elevation… illumination, and animation", and
   "avoid changing focus without people's interaction" [20]. Amazon: show focus "at all times" [19].
8. **Motion and sound confirm, never decorate.** Nielsen Norman: 100–400 ms for most transitions, entering slightly
   longer than leaving, "the more frequent the animation, the more subtle and shorter" [22]. ES-DE, measured by this
   project: a carousel step 400 ms quadratic ease-out, a view slide 402 ms cubic ease-out, metadata fades 150 ms linear
   (`EmuSen_BigPicture.md` §14.7). Nintendo: sounds tied to actions, "call-and-response", no background music [3].
9. **Art carries the screen; chrome stays out of the picture.** Apple: "edge-to-edge artwork… legible… from across the
   room" [20]; Xbox's 2023 Home "creates space for beautiful backgrounds" [7]; OpenEmu files games "along with original
   box art" [11]. Critics dislike the opposite, "a big soupy mess of game and app tiles" of recommendations [9]. EmuSen
   shows the player's own games and nothing it was not asked for.

---

## 2. The structure

Seven places: Home, the library, a game's page, the tools, Settings, the Pause Card and Power. Every screen's first row
is the way back to the game.

### 2.1 The button grammar

The same on every screen: **A** choose (on a game, Play — §0.1), **B** back one level, **X** act on the thing in hand
(on a game, open its page), **Y** favourite, **the shoulders** change section, **Select** the list's tools (the List
card), **Start** Home (inside a game, the Pause Card). One press away everywhere: Resume (A on the first row),
screenshot (X in a game), favourite (Y in the library), search (Select, then A), Home (Start).

### 2.2 Home, the hub

When EmuSen's own look is chosen, Home is the big screen's first screen. At the top, *Continue*: the suspended game's
paused frame, its name and "Paused 12 min ago"; A resumes. Then a shelf of recent games, and a row of places with a
status line each: Games, Scraper, Packages, Library Manager, Cores, Settings, Power (Switch's bottom row [4]; Xbox's
quick access [7]). Focus starts on Continue, or on the first recent game when nothing is suspended. On the desktop, Home
is the hub pane of `EmuSen_LibraryTools.md` §1b, the same sections as tiles, with Setup as its first-run face.

### 2.3 The Home card

Over an ES-DE theme (and on the desktop when a pad opens it), Home is a card rather than a screen: *Continue game*,
*Selected game* ▸, Search, Start Console (only where the console starts empty), Tools ▸, Settings ▸, Power ▸, over the
dimmed library, with the context pane showing what the focused row holds (the paused frame, the game's cover, each
tool's status). Same sections as Home, two renderings, one description (`EmuSen_LibraryTools.md` §1a).

### 2.4 The library

With a theme: the theme's system and gamelist views, untouched (§7). In EmuSen's own look: a grid of covers under a
console strip; the shoulders step consoles (All, Favourites, Recent, then each console), A plays, X opens the game's
page, Y toggles favourite, Select opens the List card (Search, Sort, Filter, Jump to, Random). Covers keep the console's
box shape; a missing cover is a generated card with the title and the console's mark.

### 2.5 A game's page

One screen for one game: a hero of its best art (or its clip, §5.9), the cover, title, console · year · developer ·
publisher · play time · last played [12]; actions Play (A), Resume ◂ state ▸, Favourite; sections stepped by the
shoulders: *Overview*, *States* (a strip of pictures and times; Load, Pin, Delete), *Options* (Engine, from Cores;
Cheats; Display crop; Controller profile; Package and its layers, from Packages), *Media* (screenshots and clips),
*Details* (metadata, Scrape This Game, Edit). PS5's hub [6] and Analogue's library detail [13] are the models.

### 2.6 Tools as destinations

Scraper, Packages, Library Manager, Cores, Cheats, Themes, Collections: each a tile on Home with a live line, read from
its listing (`pkg ls`, `core-ls`, `scrape`, `rom check`), each opening its `ManagerLayout` sheet, B returning to where it
was opened. No tool is a row named after a component without its status. The tools are libraries with thin hosts, as
`EmuSen_LibraryTools.md` §1b decided.

### 2.7 Settings, for players

Six sections, not emulator internals: *Controllers* (players, bindings, profiles), *Display* (picture, shaders, the CRT
per console, crop, text size, light or dark, background art on Home, Play videos, reduced motion), *Sound* (volume,
latency, the interface's sounds), *Consoles* (one page per console: its default engine, firmware, memory cards and
media, console-wide settings), *Library* (ROM folder, art, scraping account, themes, collections), *EmuSen* (saves and
states, big picture, updates, about). Each setting has one sentence under it; a change applies at once and nothing asks
to save. The present Preferences tabs, Graphics, Shaders, Memory Cards, Firmware and BigMenuForm's converted sheets fold
into these.

### 2.8 The Pause Card

Approved 2026-11-01. A left panel of 40% of the width; the paused frame on the right, **dimmed, not blurred**, hosting a
context pane; a title row of a small cover and the game's name on one line; at most eight rows and no scrolling:
Resume; Rewind (the pane shows the newest reel pictures and the seconds available); Save ◂ Slot ▸ and Load ◂ Slot ▸ (the
pane shows the slots as pictures and times; Load first drops one undo snapshot into rewind); Speed; console rows only
where they apply (Change Disc, Swap Players); Game Options ▸ (Cheats, Controllers, Display, Sound — deferring to the
handheld's own quick menu there — Memory Cards on disc consoles only, Rumble where reported); Leave ▸ (Back to Library,
Restart…, Quit Game…, Quit EmuSen…, confirmed). X takes a screenshot on every page. Guide-held chords: Up saves, Down
loads, R1 a screenshot, off where they clash with Steam's Game Mode. Preferences and the shell leave the in-game menu.
Rejected: a radial menu (made for a touchpad), OpenEmu's heads-up bar as the pad's menu (made for a mouse), tabs (too few
sections in a game). It is the first instance of the Card form (§3.2) and shares the Home card's title row, rows, pane
and help bar.

### 2.9 Moving between places

Start toggles the Home card (or Home) from anywhere outside a game and the Pause Card inside one; B climbs one level
and from a first page closes the card; a tool opened from Home returns to Home; a game's page opened from the library
returns to the library on the same game; Play from anywhere goes to the game; Back to Library from the Pause Card
returns to the page the player left. Power ▸ follows §0.2.

### 2.10 The Library Card, folded in

The Library Card (a proposal of 2026-11-01, not adopted as a design of its own) contributed: its row order (the way
back to play, the thing in hand, tools, settings, the way out), the Continue row with the frame and the time, Y as
favourite, tools with live status, dim rather than blur, and Start Console only where it applies. Its Settings list
becomes the six sections of §2.7, Cheats moving to the game's page and the Cheats tool; its List ▸ leaves the game's hub
for the Select card; its Tools rows become tiles on Home and rows on the Home card from one description. The
Switch-style HOME it rejected is adopted for EmuSen's own look, because that look already exists as a second library
("EmuSen (built in)" in the theme list, `EmuSen_Settings_Reference.md` §4.56) and today draws only the desktop grid on a
big screen; giving it Home's sections makes it a look rather than a fallback. Its objection stands for themed sessions,
where the card form is used instead. **[A]**

---

## 3. One grammar: the frame

### 3.1 Roles

Every screen is the same frame: **header** (a small picture and one line, never wrapped, with facts or a status),
**sections** (stepped by the shoulders), **content** (rows, a grid or a table), **detail** (the context pane: what the
focused thing holds), **actions** (declared once, drawn as the help bar and as buttons where needed) and the **help
bar**. These are `AdaptiveLayout`'s roles (navigation, content, detail, actions, search, status;
`EmuSen_LibraryTools.md` §1a, LunaP §202) with one role added, the header. `ManagerLayout` is already one canonical
layout on it (LunaP §203).

| Screen | navigation | content | detail | header |
|---|---|---|---|---|
| Home (own look) | none | shelves and the places row | the focused tile's art, as background | Continue |
| Home card, Pause Card, List card | none | the rows | the context pane | cover and name |
| Library (own look) | consoles | the cover grid | none (X opens the page) | none |
| A game's page | its sections | the section's rows or grid | the context pane | hero and facts |
| Tools | filters | the table | the row's detail | the tool's name and status |
| Settings | the six sections | the section's rows | the row's sentence and preview | none |

### 3.2 What LunaP gains, by stage

- **U1:** the tokens of §5 in one palette for both forms, the type scale, spacing, focus, motion and sound (LunaP §206).
- **U2:** a third `Form`, **Card**: a left-anchored panel of 40% over a scrim, detail in the remaining 60%, navigation
  hidden, actions in the help bar. The Pause Card's LunaP side (LunaP §205: `MenuPanel.Anchor`, `WidthFraction`,
  `Pane`, `TitleIcon`, `BlurBackdrop.Blurs = false`) already draws the panel; the Card form is `AdaptiveLayout` placing
  its roles into it, so the Home card and the List card get it without a second implementation. `SheetLayer` keeps the
  centred sheet for questions and tools. A **header** role (picture, title, subtitle, facts), so no window draws its own
  title row; `MenuPanel`'s title, subtitle and footer move into it.
- **U4, U5:** a **Status** on `AdaptiveSection` and on rows (`MenuRow.Value` exists), one line drawn right-aligned on a
  row and under a tile; a **TileGrid** form of content for the big screen (the desktop's `TileGrid<T>` reused, with the
  focus ring and a shelf's horizontal mode); a poster slot shaped for video (§5.9).
- **U6:** a third canonical layout, **SettingsLayout** (sections and rows with a sentence each), retiring
  `BigMenuForm`.
- **Throughout:** `FitAudit` gains the ring: a focused control's ring whole on screen and 3:1 against what it sits on.

### 3.3 Reconciliation with what is decided

- **`EmuSen_LibraryTools.md` §1a** (one description, two forms) is the mechanism every screen here uses; the Card form
  is a third form of the same description, not a new mechanism.
- **§1b** (tools as libraries, thin hosts, Home the hub) is kept: Home's places are §1b's hub, rendered as a screen in
  the own look and as a card over a theme.
- **§1c** (the family) is kept: Packages, Scraper, Library Manager and Cores stay on `ManagerLayout`, gaining the header
  role and the tokens from the frame without changes of their own.
- **`EmuSen_InterfaceSpec.md`** records every audited state; each stage below re-records the states it changes on
  purpose (§5.2 there), and its audits (fit, pad reachability, names, palette reach, §4 there) hold every new screen.

---

## 4. The six screens

The mockup's screens, condensed to what a builder needs; positions are x,y and sizes w × h in design units on the big
screen and pixels on the desktop. Shared parts on the big screen: the safe area x 64–1216, y 40–760; the **help bar**
0,702 1280 × 58 on Surface at 92% with a hairline Border on top, entries from x 64, each a lettered glyph in a 30 × 30
ring and its word in Caption, 40 between; a **row** 54 tall, label at x+24 in Row Text, value right-aligned in Row
Muted, `◂ value ▸` where Left and Right change it, `▸` where A opens a page, `…` where A asks first, a 1-unit Rule under
each; a **focused row** the raised bar with a 4-unit Accent bar at its left and TextStrong words; a **focused tile** the
ring (§5.4); a **section strip** 44 tall, names in Caption upper case, padding 16, the chosen one TextStrong with a
3-unit Accent underline, only whole names drawn (`WholeRow`).

1. **Home (own look).** Status line 64,40 (`2 controllers · 21:42`, the handheld adding its battery); Continue frame
   64,84 373 × 280 with `CONTINUE PLAYING` (Caption, upper, Muted), the title in Display TextStrong (two lines, shrink
   floor 32) and facts in Body Muted beside it; `RECENT` and a shelf of 120 × 160 tiles at 64,416, gap 24, stepping 144
   over Reel; the focused tile's title under the shelf; seven places at 64,622, 150 × 64 each, gap 17, label Caption
   TextStrong and status Caption Muted, a status needing the player in Attention with `⚠`. Background: Surface with the
   focused tile's art at 40% opacity, blurred 40, under a Scrim-to-Surface gradient (§0.6). Focus starts on Continue;
   Down to the shelf, Down to the places. Desktop: the Mistress window's sidebar (Home, Games, Favourites, Recently
   Played, the consoles), a Continue card 1032 × 180 with a `Resume` button, the shelf of 100 × 133 tiles, places as a
   4 × 2 grid of 246 × 64; F4 opens the same card a pad would.
2. **Games (own look).** The console strip at 64,40 with the count at its right; a grid of 180 × 240 tiles, gap 24,
   five across at 142,108; a missing cover the generated card (the console's mark 48 × 48 Muted, the title in Title
   TextStrong); a favourite a 24-unit Accent heart; the focused game's title (Title) and facts (Body Muted) under the grid,
   cross-fading over Fade when the focus settles. Select opens the List card (Search · Sort · Filter · Jump to ·
   Random). Desktop: the present OpenEmu library (`EmuSen_Settings_Reference.md` §4.33) with the generated placeholder
   and a 2-px Accent ring on the selected cell; Space or `Info` opens the game's page.
3. **A game's page.** Hero 0,0 1280 × 360 under the OnArt gradient (or the clip, §5.9); cover 64,180 240 × 300 raised
   with the shadow; title 336,232 in Display OnArt; facts in Body at 70%; actions 44 tall at 336,328 (`A Play` filled
   Accent, `Resume ◂ Slot 3 · Long Library · 3 days ago ▸`, `Y ♥ Favourite`); the section strip at 336,396; content from
   64,464 (States: cards 200 × 150 with `Slot n` and the time; Options: rows; Media: pictures and clips in the console's
   shape). Focus opens on Play. Desktop: in the content area with `‹ Back`, hero 1080 × 200, tabs, States as a wrapping
   grid of 160 × 120 cards.
4. **The Pause Card.** As §2.8: panel 0,0 512 × 800 on Surface with the shadow on its right edge, sliding 48 units over
   Enter with the scrim's fade; the paused frame exactly where it was, under the Scrim, never blurred, moved or scaled;
   title row 64,40 424 × 100 (cover 60 × 80, title in Title TextStrong, shrink floor 22); rows from 64,156; context pane
   576,156 640 × 530 (three reel pictures for Rewind; three slot cards 192 × 144, the current one ringed, for Save and
   Load, with a sentence such as `A loads Slot 3. The moment before is kept in Rewind.`). A question is a page of `No`
   then `Yes`. Desktop: with a pad the same card at 40% of the content area; with mouse and keyboard OpenEmu's heads-up
   bar (§4.34 of the settings reference) and the menu bar carry the actions.
5. **Packages (a tool on the frame).** The header row 64,40 (`PACKAGES` in Title, its status right-aligned in Caption
   Muted: `3 changed · 263 KB in packages · 410 GB free`); the section strip with the search box at its right; the
   table, seven rows of 54 on InputSurface, mark boxes filled Accent with an OnAccent tick; actions wrapping rather than
   scrolling; the apply bar with `Apply…` filled Accent and the space meter. This is `ManagerLayout`'s big-screen form
   as built (LunaP §203), plus the header row. Desktop: the Packages window of `EmuSen_Packages.md` §3 in the tokens.
6. **Settings.** The six sections as a strip at 64,40; rows 704 wide from 64,108, eight visible; a detail pane 800,108
   416 × 578 on SurfaceRaised with a preview where the setting has a picture (the shader on a real frame, the crop's
   edges, the three text sizes, the two variants) and the row's sentence in Body. Desktop: the Preferences window at
   960 × 640, a `SourceList` of the six sections, `FieldRow`s 48 tall with the sentence in Caption Muted under each, a
   search field that filters rows across all six.

---

## 5. The visual language

### 5.1 Colour tokens

Dark and light; the big screen is dark unless the player chooses light (§0.4). One palette for the desktop and the big
screen, written once (LunaP §206.1). Pairs are held to WCAG's 4.5:1 for words (1.4.3 [23]) and 3:1 for a control's edge,
fill or ring (1.4.11 [24]) against Surface, SurfaceRaised and InputSurface in both variants, 50 pairs in all
(`InterfaceTokenTests`).

| Token | Dark | Light | Use |
|---|---|---|---|
| Surface | #1B1B1E | #F3F3F3 | the screen and the desktop window |
| SurfaceRaised | #26262B | #FFFFFF | cards, tiles, a focused row's bar |
| InputSurface | #0E0E10 | #FFFFFF | fields, tables |
| Void | #000000 | #000000 | behind a frame |
| Scrim | #000000 at 55% | the same | over a frame or a theme under a card; never a blur |
| CaptionSurface | #000000 at 67% | the same | a caption over art |
| Border | #707070 | #8C8C8C | edges and dividers, 3:1 or better |
| Rule | #333337 | #D6D6D6 | the line under a row (decoration, not held to 3:1) |
| Text | #D4D4D4 | #1F1F1F | body and rows |
| TextStrong | #EEEEF0 | #000000 | titles and the focused row |
| Muted | #8D8D91 | #5F5F5F | status lines and facts |
| Accent | #4C9AE8 | #005FB8 | the ring, the focus bar, filled tracks, checks |
| OnAccent | #0E0E10 | #FFFFFF | on the accent |
| Success / Attention / Error | #35A064 / #DAA520 / #D37070 | #1B6E3C / #7A5B00 / #B3261E | status, always beside a word |
| OnArt | #FFFFFF over a gradient of black 0→72% | the same | words over a picture |

**Corrections to the study's values [M] 2026-11-01.** The study measured its dark column against Surface only. Against
SurfaceRaised, where status lines sit under a tile and a value sits on a focused row, four fell short and were moved by
the least change in lightness that clears the floor: Muted #86868A (4.15:1) to #8D8D91 (4.55:1), Border #6E6E6E (2.95:1)
to #707070 (3.04:1), Success #2E8B57 (3.55:1) to #35A064 (4.56:1) and Error #CD5C5C (3.79:1) to #D37070 (4.53:1). OnArt
over a white picture under the 72% scrim measures 9.3:1. `Info` is kept as the older name of Attention.

### 5.2 Type

Barlow Condensed for display words and titles, Barlow for everything else, on every form (§0.3); regular and medium
weights only, never light [21]. Both families ship with LunaP under the SIL Open Font License 1.1, version 1.422 from the
Barlow project (`THIRD_PARTY_NOTICES.md` §1.5). The symbol glyphs Barlow lacks come from one bundled face each, declared to the
font manager so they resolve the same on every machine (decided 2026-11-01): the arrows, triangles, ticks, warning sign,
dot, stars and boxes from Adwaita Mono, version 34.08 under the same licence, whose line beside Barlow's stays within
2 px of a plain one (LunaP §208.13).

| Role | Big screen (units) | Desktop (px) | Face |
|---|---|---|---|
| Display | 44, medium | 24 | Barlow Condensed |
| Title | 32, upper case | 16 | Barlow Condensed |
| Strong | 26 | 14 | Barlow |
| Row | 26 | 13 | Barlow |
| Body | 22 | 13 | Barlow |
| Caption | 20 | 11 | Barlow |
| Pad glyph | 22 | 13 | the glyph set |

Body 22 at 800 lines is 29.7 px at 1080p, above Apple's 29 pt body and Amazon's 28 px [21][19]; Caption 20 is 27 px,
above Apple's 23 pt minimum. On the handheld (1920 × 1200 at 8 inches) a 22-unit body is 33 px. **[A]**

### 5.3 Spacing

Base 4. Big screen: row 54 (the big panel's measured row); control at least 44; gutter 24 inside a pane, 32 between
panes; safe area 64 at the sides and 40 top and bottom, 5% of each dimension [18][19]; card panel 512 (40%); title row
100; help bar 58; a cover tile 180 × 240 with 24 between, five across; radius 4 on rows and buttons, 8 on tiles and
cards. Desktop: menu bar 28, status bar 24, sidebar 200, `SourceList` rows 24, gutter 24, gap 16, `FieldRow` 48,
buttons 28 tall, tiles 100 × 133 with 16 between, radius 4 and 6.

### 5.4 Surfaces, depth and focus

Three levels only: the world (the frame or the theme, under the Scrim while a card is open), the surface, and raised
(cards, tiles, the focused row). One shadow, on a focused tile and on a card's open edge: offset 0,8, blur 24, black at
50%. No gradient except the OnArt scrim.

**Focus, for a pad.** A tile takes a ring of 3 units in Accent, drawn 4 outside it with its radius matching, scales to
1.06 over Quick and takes the shadow. A row takes the bar: SurfaceRaised, a 4-unit Accent bar at its left edge and
TextStrong words, no ring. After two seconds idle the ring's opacity breathes 100 → 70 → 100% over 1.6 s, so the place
is found after looking away [19]; under reduced motion it holds still. Exactly one focus mark on screen; the pointer
hides it and hover shows a 1-unit Border instead; focus never moves without a press [20].

### 5.5 Motion

| Token | Duration | Easing | Use |
|---|---|---|---|
| Instant | 0 | — | a list's cursor, the help bar's change (as ES-DE measured) |
| Quick | 120 ms | quadratic out | the ring between tiles, a tile's scale, a stepped value |
| Enter | 200 ms | quadratic out | a card slides 48 units from its edge while fading; the scrim; the context pane's cross-fade [22] |
| Exit | 160 ms | quadratic in | the same, leaving |
| Pan | 320 ms | cubic out | Home ↔ library ↔ a game's page, under ES-DE's 402 ms and Nielsen Norman's 400 ms ceiling |
| Reel | 400 ms | quadratic out | carousel and shelf steps, ES-DE's own, so themed and own views feel alike |
| Fade | 150 ms | linear | facts while scrolling |
| Poster | 1.5 s delay, then Enter's cross-fade | quadratic out | a hero picture turning into its clip (§5.9) |

Only what the player acted on moves. The paused frame never moves, blurs or scales; tiles never bounce; the help bar
never animates. Reduced motion (the system's preference or a switch under Settings ▸ Display) sets every movement to 0,
every fade to 120 ms, stops the ring's breathing (WCAG 2.3.3 [25]) and shows posters instead of clips.

### 5.6 Sound

Nine cues, synthesised by the project and never copied from anyone's sounds: Move (1.5 kHz, 30 ms), Section, Choose,
Back, Favourite, Launch (the six EmuSen already synthesised, unchanged), Saved (880 then 1320 Hz, 60 ms each), Loaded
(the reverse) and Refused (220 Hz, 120 ms). On by default at 60% (§0.5), one switch and one volume under Settings ▸
Sound. Cues play in menus and cards, never over running gameplay except Saved, Loaded and the screenshot, 12 dB under the
game. A theme's seven navigation sounds replace the library view's sounds and nothing else, so cards and tools sound the
same under every theme. No music [3].

### 5.7 Iconography

Words first; an icon never stands alone [3][18]. Three sets: the lettered pad glyphs (they exist); console marks (the
theme's system art, or EmuSen's monochrome set on a 24-unit grid with a 2-unit stroke, drawn by the SVG subset
renderer); status glyphs (● default, ○ chosen by games, ↑ update, ⚠ attention, ✓ ok), each beside its word. No tooltips
on the big screen [18]; the desktop keeps them.

### 5.8 Art

Covers in the console's box shape (`CoreDescriptor.CoverAspect`), never stretched, on SurfaceRaised; screenshots in the
console's picture shape; the paused frame as art on Continue and the Pause Card; a game's best art as its page's hero
under the OnArt scrim; the focused tile's art as Home's background at 40% opacity, blurred 40 units, under the Scrim,
on by default with a switch (§0.6); a state's picture at 160 × 120; a missing cover a generated card with the title in
Barlow Condensed and the console's mark, on both forms, replacing the scanline placeholder.

### 5.9 Video

Decided 2026-11-01 (§0.8). Video is designed in from the start, so the screens that will play it leave room for it now.

**Source.** ScreenScraper's gameplay clips, through EmuSen's own scraper; downloading them is built and off by default
(`EmuSen_LibraryTools.md` §8.4). Playback is Pass 12 (`EmuSen_BigPicture.md` §21, with Q20 and Q21 answered 2026-09-26:
the theme's video element, the media viewer and the screensaver), built on the LunaP video surface that pass plans.

**Where it plays.**
1. **A game's page:** after the Poster delay (1.5 s, §5.5), the hero picture cross-fades into the clip, muted and
   looping, as console game hubs do. The picture stays as the poster frame and the fallback.
2. **The Media section:** clips play in full beside the screenshots; A opens a clip full screen, with sound only there.
3. **Home:** the focused tile's clip plays in the Continue and shelf preview, never with sound.

**Rules.**
- Never sound over a game, and never on Home or the hero; clips are muted until the player opens one.
- Reduced motion, or a Settings ▸ Display switch *Play videos*, shows posters only.
- On the handheld the default stays on. Battery and decoding cost is measured when Pass 12 is built and may change the
  handheld's default.
- A theme's own video elements stay the theme's: the same playback engine, the theme's layout.

**In the build order.** The game's page (U4) and Home (U5) leave a poster slot shaped for video; Pass 12 lands the
playback; a later stage turns the slots on.

---

## 6. Forms

One description, three renderings, decided by `AdaptiveLayout.Form` and the scale.

- **Big screen** (1280 × 800 and 1920 × 1200, 16:10; 1280 × 720 audited): the screens of §4 at scale height / 800, dark
  unless the player chooses light, pad-driven, the docked keyboard for text (`EmuSen_Shell.md` §3).
- **Handheld** (Legion Go S, 1920 × 1200 at 8 inches, at arm's length): the big-screen form at scale 1.5, touch accepted
  on anything 44 units or larger, Sound's volume row deferring to the device's quick menu, the Guide chords off where
  Steam owns them; type not enlarged beyond the scale (§5.2).
- **Desktop** (windowed, 1280 × 800 to 1920 × 1200): the Mistress window with the menu bar, Home as a pane, the OpenEmu
  library, tools as windows in the desktop form of `ManagerLayout`, Barlow and pointer affordances; a pad connected on
  the desktop opens the cards and the ring as on the big screen. The standalone launcher hosts Home's tools' panes in
  the same forms.

---

## 7. Themes

An ES-DE theme owns the system view and the gamelist view: their layout, colours, fonts, system art and the library's
seven sounds, exactly as ES-DE renders them (`EmuSen_BigPicture.md` §13.8 holds Mistress to ES-DE's captures). EmuSen
owns everything else: the Home card, the Pause Card, the List card, the game's page, tools, Settings, questions and the
launch screen. They share the screen without clashing because EmuSen's screens are a separate layer with their own
light: a card opens over the theme under the Scrim, in EmuSen's tokens and type, so the theme is seen through it dimmed.
**No theme colour is ever read into a card, and a card never styles a theme** (§0.7); the theme's help bar is the
theme's inside its views, EmuSen's inside its cards. In the theme list, "EmuSen" is the first entry and means Home and
the cover grid; a theme's variants, colour schemes and font sizes stay the theme's options as ES-DE offers them [14].
LaunchBox's Big Box lets themes reach its startup, shutdown and pause screens [26]; that reach is what this design
withholds, so the Pause Card looks the same under every theme. **[A]**

---

## 8. Accessibility

- **Contrast:** words 4.5:1, every control and ring 3:1 (WCAG 1.4.3, 1.4.11 [23][24]), held by `InterfaceTokenTests`
  for every token pair in both variants (§5.1).
- **Text size:** a Settings ▸ Display row *Text size* (Small 0.9, Default, Large 1.15) scaling the type scale within the
  fit audit, as ES-DE 3.0 added font sizes [14].
- **Reduced motion:** §5.5.
- **Screen readers:** every operable control has a name (`FitAudit.Named` requires it); a tile's status line is its
  description; the help bar's glyphs carry their words.
- **Colour is never the only signal:** a status is a glyph and a word. Questions keep No first.

---

## 9. What becomes of what is built

- **Packages, Cores, Scraper, Library Manager** stay on `ManagerLayout`, gaining the header role (name and status), the
  tokens and the type scale from the frame without changes of their own; each becomes a tile on Home with its status
  line read from its listing.
- **The pad menu** (`MainWindow.PadMenu.cs`): its main page becomes the Home card and its game page the Pause Card, both
  instances of the Card form; its submenus Library ▸, Settings ▸ and EmuSen ▸ dissolve into Tools, the six Settings
  sections and Power.
- **BigMenuForm** is retired page by page as each settings sheet moves to SettingsLayout.
- **The desktop library** (`EmuSen_Settings_Reference.md` §4.33) is kept, with the tokens and the generated placeholder;
  the big-screen "EmuSen (built in)" look becomes Home and the grid.
- **The Pause Card**, as approved, is the first Card.

---

## 10. The build order

Each stage is recorded in `EmuSen_InterfaceSpec.md`, held by the fit and pad audits (`WindowFitAuditTests`, LunaP's fit
audits, `PadMenuTests`), and re-records only the states it changes on purpose.

| Stage | What it builds | Holds |
|---|---|---|
| **U1** | The tokens: colour in both variants and one palette for both forms, the type scale in Barlow and Barlow Condensed, spacing, the focus treatment, motion and sound resources, in LunaP so every window changes together | contrast for every token pair; every window still fits at 1280 × 800 and 1920 × 1200 |
| **U2** | The Card form and the header role in LunaP, then the Pause Card on them | the Pause Card's states recorded and audited |
| **U3** | The Home card over themes, replacing the Start menu | the themed views' menus re-recorded |
| **U4** | A game's page, with its poster slot shaped for video | the page's sections in both forms |
| **U5** | Home and the cover grid as EmuSen's own look, with Home's poster slot; the desktop Home pane and Setup | Home's and the grid's states |
| **U6** | SettingsLayout and the six sections, retiring BigMenuForm | every settings sheet moved |
| **U7** | The tools' header rows and their tiles on Home | the four tools' states |

Video's playback is Pass 12 (§5.9), independent of this order; a later stage turns the poster slots on.

---

## 11. U1 as built (2026-11-01)

- **LunaP (§206 there).** `Theme/Tokens/Dark.axaml` and `Light.axaml` write every colour once; the desktop's
  `Palette.axaml` and the big screen's `MenuPalette.axaml` both read them, so the big screen is the same palette rather
  than a copy, and `MenuLook.IsLight` gives a big screen the light column. `LunaMetrics` and `LunaTokens` hold the type
  scale, spacing and focus of each form; `LunaFonts` the bundled Barlow and Barlow Condensed; `LunaMotion` the motion
  tokens and reduced motion; `LunaSounds` the nine cues. The look's rows, buttons and switches and `MenuRow` draw the
  focus bar; `MenuRow` and `MenuPanel` take their colours from the palette in reach.
- **EmuSen.** The drawn menus use LunaP's Barlow Condensed (`LunaFonts.CondensedFile`) and Mistress's own copy is gone;
  a framed sheet's stock controls, headings and tabs stay in Barlow Condensed at their present sizes; the desktop is in Barlow
  at 13; `NavigationSounds` renders LunaP's cues under ES-DE's names, the tone code moved unchanged so the sounds are the
  same; the navigation volume defaults to 60.
- **Not yet in U1.** The drawn menus' rows and a framed sheet's words stay in Barlow Condensed at their present sizes
  until the Card form rebuilds them (U2, U3): moved to Barlow at 22, the sheets failed the fit audit in 23 states,
  wider words past their bars and a controller drawing's labels under the floor (LunaP §206.3). The player's switches for light or dark, reduced motion
  and Play videos arrive with Settings (U6); the tile ring is a token until the first tile grid takes it (U5).
- **Recordings.** Every recorded state changed, by the causes listed in `EmuSen_InterfaceSpec.md` §6.3.17.

---

## 12. U2 as built (2026-11-01)

- **LunaP (§207 there).** `AdaptiveForm.Card` places a layout's roles into a `MenuPanel` anchored left at 40 per cent
  over the Scrim: the content as its rows, the detail in the room beside it, the header as its title row, the actions in
  the help bar, no navigation. Its rows are Barlow at Row 26 in their own case, so the departure of §0 ends for what a
  card draws. `AdaptiveRole.Header` with `AdaptiveHeader` (picture, title, subtitle, facts) is drawn as a title row on
  the desktop and the big screen and as the card's title row; `MenuPanel`'s title, subtitle, icon and footer are what
  the card's header sets, and the centred sheets keep setting them until they are rebuilt. `SwapsAcceptAndBack` keeps a
  card's help bar true to a player's swap.
- **EmuSen.** The Pause Card of §2.8 on that form (`EmuSen_BigPicture.md` §48): the first page, Game Options and Leave,
  the slots and reel in the pane, X for a screenshot, the volume row deferring in Game Mode, the Guide chords with their
  setting, and the library menu's Back row with the paused frame. The library menu keeps its centred, blurred look.
- **The runtime.** A state load with rewind on keeps the chain and first takes a snapshot, so Rewind undoes a wrong
  load (`EmuSen_Backend.md` §36), in the Rust runtime alone.
- **Not in U2.** Rumble has no row, since nothing reports it. Console rows still come from what the running machine
  reports; a capability table in the core ABI is later.
- **Recordings.** Eighteen new states, the card's and the library menu's Back row, at 1280 × 800 and 1920 × 1200, and two
  on the handheld (`EmuSen_InterfaceSpec.md` §6.3.18).

## 13. U3 as built (2026-11-01)

- **EmuSen (`EmuSen_BigPicture.md` §49).** The Home card of §2.3 on the Card form, the same `PadCard` as the Pause Card:
  Continue with the paused frame, its name and the time since it was left; the game in hand, whose page holds the game's
  options until U4, the favourite on Y; Search where the view has one; Start a Console only where a console starts
  empty; Tools with the Scraper's and Cores' live lines and the other tools after them; Settings as they were; Power.
  Over the library dimmed, not blurred; the theme's help bar hidden under the card; Y a favourite where a game is in
  hand; on the desktop the same card when a pad opens it. The centred library menu, its page set and its Back row are
  deleted, so the departure of §0 ends for every drawn menu.
- **LunaP.** Nothing new: the Card form and header role of U2 held the Home card as they were.
- **Not in U3.** Sleep and shut down, for want of a way to know EmuSen is the device's shell; live lines for Packages
  and the Library Manager, whose counts come from jobs over every file (`EmuSen_BigPicture.md` §49.6). Each waits on a
  surface proposed first.
- **Recordings.** Twenty-one new states, the themed menu's two taken again and the Back row's two retired
  (`EmuSen_InterfaceSpec.md` §6.3.19).

---

## 13a. U4 as built (2026-11-01)

- **LunaP (§209 there).** `PageLayout`, a canonical layout for one item's page in both forms: the hero to the edges in
  a `PosterSlot` shaped for video (its `Video` kept, `PlaysClip` false until Pass 12, the new `Poster` motion token of
  1.5 s), the cover, the title and facts, the actions with a stepped value, the strip with the frame's look, rows,
  tiles and text as the content, deeper pages named `Parent › Page`, the viewer, and the help bar generated from what
  the host declares. `AdaptiveSection.Status` and `PageRow.Status` are §3.2's Status on sections and rows;
  `MenuRow.Inset` gives a row the page's 24.
- **EmuSen (`EmuSen_BigPicture.md` §50).** Screen 3 and §18.2's deeper pages from one plain class, `GamePage`, on both
  forms; X on a game, A on the Home card's game, `Space` and the desktop's context menu open it. The metadata editor,
  cover picker, name chooser, package window and screenshot window, the game options sheet's game entries and the
  library's two media views are deleted (§17.9's D3).
- **Departures, measured.** The cover's box stops at 268 units, not 300, which would have reached 16 into the content
  at 464; Cover Art's cards are 150 × 176 so a two-line name fits above the help bar; Edit Details saves each field at
  once on the desktop too, with no Save button.
- **Not in U4.** Controller profile, a layer's switch and order, ScreenScraper's other covers and the generated one, X
  Pin and Set as cover, and the docked keyboard, each waiting on a surface proposed first (§50.4 there).
- **Recordings.** Thirty-two new states, the themed Select sheet and the Home card's game row taken again, and twenty-six retired with their
  windows (`EmuSen_InterfaceSpec.md` §6.3.20).

## 13b. U5 as built (2026-11-03)

- **LunaP (§212 there).** `HubLayout`, a canonical layout for a hub in both forms (Continue in a poster slot shaped for
  video, a shelf of whole covers on one baseline, places with a live line each, the help bar generated from the focus);
  `GalleryLayout`, the big screen's grid of covers under the strip with the focused item's name and facts, and search with
  the keyboard docked at the foot (`OnScreenKeyboard.CreateDocked`, `KeyboardLayout.Search`); `CoverCard`, a cover in its
  box shape on its cell's foot or the generated card with its mark and title, with the ring, scale and shadow of §5.4.
- **EmuSen (`EmuSen_BigPicture.md` §51).** Screens 1 and 2 from two plain classes, `HomeScreen` and `GamesScreen`; the List
  card of §18.1 and the Start Console page of §18.3 on the Card form (the Home card's `PadCard`); Collections of §18.4 as a
  tool screen; the desktop's Home in the sidebar's first row. Start over the own look goes to Home the screen (§0.9).
- **Retired.** The desktop library drawn on the big screen, with its console facet and big-screen sizes; the Home card over
  the own look, which over a theme and on the desktop stays.
- **Departures, measured.** A
  generated card's mark keeps its size and its title is shrunk no further than 18 units, so a short box stays legible under
  the fit audit; the facts under the grid change at once rather than over Fade.
- **Not in U5.** Live lines for Packages, Library and the desktop's Cheats, waiting on summaries no surface gives; the
  desktop's Home as the window's first view and its grid's header row, which are D1's; Setup; the ES-DE collection settings,
  which move with U6 (`EmuSen_BigPicture.md` §51.6).
- **Recordings.** `EmuSen_InterfaceSpec.md` §6.3.22.

## 13c. U6a as built (2026-11-03)

- **LunaP (§213 there).** `SettingsLayout`, the third canonical layout, in both forms: the strip of sections or the trail
  of pages, whole rows with a value and a sentence, the pane with a picture, meter or variants preview, the generated help
  bar; on the desktop a source list, rows of 48 with their controls, the pane where a row has a picture, and search.
- **EmuSen (`EmuSen_BigPicture.md` §52).** Screen 6 and D5 from one plain class, `SettingsScreen`: the six sections, every
  console a row with its page (Engine as Cores' Make Default, Firmware from `firmware_status`, Memory cards), and every
  door to Settings reaching the same screens.
- **Retired.** `CollectionSettingsWindow`, `DebugSettingsWindow`, the pad menu's Settings submenu; in U6b Preferences
  with every tab, Graphics Settings, Shader Settings and `BigMenuForm`, their tests walking Settings. Active Cheats is
  reached through Library ▸ Cheats (`EmuSen_BigPicture.md` §52.5).
- **Not in U6a.** The rows awaiting their surfaces and the accessibility rows (`EmuSen_InterfaceSpec.md` §6.3.23).
- **Recordings.** `EmuSen_InterfaceSpec.md` §6.3.23.

---

## 14. Sources (opened 2026-11-01)

1. Libretro forums, "The UX problems of RetroArch", https://forums.libretro.com/t/the-ux-problems-of-retroarch-the-program-that-uses-cores/29421
2. The same thread, post 37, https://forums.libretro.com/t/the-ux-problems-of-retroarch-the-program-that-uses-cores/29421/37
3. Nintendo Everything, "Nintendo talks about Switch's OS/home menu design", https://nintendoeverything.com/nintendo-talks-about-switchs-os-home-menu-design-resources-have-less-than-200kb/
4. Nintendo, "Nintendo Switch HOME Menu Overview", https://en-americas-support.nintendo.com/app/answers/detail/a_id/22308
5. PlayStation Blog, "First look: PlayStation 5's next-generation user experience", https://blog.playstation.com/2020/10/15/first-look-playstation-5s-next-generation-user-experience/
6. PlayStation, "PS5 features", https://www.playstation.com/en-us/ps5/ps5-features/ ; control center, https://www.playstation.com/en-us/support/games/customize-ps5-control-center/
7. Xbox Wire, the new Home (26 July 2023), https://news.xbox.com/en-us/?p=194834
8. Kotaku, "Xbox Series X/S dashboard update", https://kotaku.com/xbox-series-x-s-dashboard-update-game-pass-ui-1850679127
9. Pure Xbox, "The new Xbox dashboard looks clean, but it still needs work", https://www.purexbox.com/features/soapbox-the-new-xbox-dashboard-looks-clean-but-it-still-needs-work
10. GamingOnLinux, "Steam Deck UI comes to desktop in beta", https://www.gamingonlinux.com/2022/10/steam-deck-ui-comes-to-desktop-in-beta-to-replace-big-picture-mode/
11. OpenEmu, https://openemu.org/
12. Analogue, 3DOS user guide, https://www.analogue.co/support/3d/guide/3dos
13. Analogue, "What's new in Analogue OS v1.1 beta", https://www.analogue.co/announcements/os-v1-1-available-now-what-s-new-in-analogue-os-v1-1-beta
14. ES-DE user guide, https://gitlab.com/es-de/emulationstation-de/-/raw/master/USERGUIDE.md ; GamingOnLinux on ES-DE 3.0, https://gamingonlinux.com/2024/02/emulationstation-desktop-edition-es-de-gets-new-default-look-and-different-font-sizes
15. Libretro docs, the guide to RetroArch's interface, https://docs.libretro.com/guides/gui/
16. Libretro docs, Ozone, https://docs.libretro.com/guides/ozone ; the Ozone announcement, https://www.libretro.com/?p=46928
17. Libretro docs, XMB, https://docs.libretro.com/guides/xmb
18. Microsoft, "Designing for Xbox and TV", https://learn.microsoft.com/en-us/windows/apps/design/devices/designing-for-tv
19. Amazon, Fire TV "Design and User Experience Guidelines", https://developer.amazon.com/docs/fire-tv/design-and-user-experience-guidelines.html
20. Apple HIG, "Designing for tvOS" and "Focus and selection", https://developer.apple.com/design/human-interface-guidelines/designing-for-tvos , https://developer.apple.com/design/human-interface-guidelines/focus-and-selection
21. Apple HIG, "Typography" (tvOS table), https://developer.apple.com/design/human-interface-guidelines/typography
22. Nielsen Norman Group, "Executing UX Animations: Duration and Motion Characteristics", https://www.nngroup.com/articles/animation-duration/
23. WCAG 2.1, Understanding 1.4.3 Contrast (Minimum), https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html
24. WCAG 2.1, Understanding 1.4.11 Non-text Contrast, https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html
25. WCAG 2.1, Understanding 2.3.3 Animation from Interactions, https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html
26. Playnite, "Comparison with other software", https://api.playnite.link/docs/manual/gettingStarted/comparisonWithOtherSoftware.html ; LaunchBox Premium, https://www.launchbox-app.com/premium
27. The Barlow project, https://github.com/jpt/barlow (fonts and licence fetched 2026-11-01)

---

## 15. Cores, redesigned (2026-11-01)

Decided 2026-11-01 by the tester, after seeing the Cores window of `EmuSen_Cores.md` §9.1 on the big screen: it is
rebuilt on the frame of §3 as a screen of its own, in the form of screen 5 (§4) with the detail beside the content.
The stage U7 of §10 had given the four tools only a header row; this section is the screen that stage lacked,
written for Cores and, in §16, held against the other tools.

### 15.1 The window as built, measured

Recorded on 2026-11-01 from `WindowFitAuditTests`' five Cores states at 1280 × 800 and 1920 × 1200 (the pictures under
`~/.cache/emusen/probe/window-look/trials/cores-redesign/`; the recording `CoresDetail-1280x800.spec`). **[M]**

1. The tool is drawn inside the centred sheet frame, 812 × 668 at 1280 × 800: 63 per cent of the width and 83 per
   cent of the height, with the title `CORES` alone taking 100 of the 668.
2. The section strip holds 23 chips (All, every console, then Installed, Available, Updates, Waiting for Approval, Not
   Supported); four are drawn whole at 1280 × 800 and the rest are reached only by stepping.
3. The list has three columns, Mark, Core and State; a libretro core's `.info` name is cut in the row (`Sony -
   PlayStation 2 (LRPS2, the PCSX2 lib…`), and nothing in the row says which cores ship with EmuSen.
4. The detail is a page under A, scrolling, whose lines are the tool's facts as it holds them: the firmware line with
   the `.info`'s path (`pcsx2/bios/ps2.bin`), `Notes: (!) needs a BIOS · disc images by path`, the address and the
   index's CRC-32, the whole SHA-256 on one line.
5. Nine buttons in two rows (Install, Update, Remove, Approve…, Make Default, Options…, Verify, Check for Updates, Mark)
   stand between the list and the apply bar, and the page repeats four of them; two help bars are drawn, the frame's
   hint bar inside the sheet and the host's below it.
6. The common path, installing and approving one core, is Mark, Apply…, Apply, then Open, then Approve… on the page,
   then the dialog: six presses after finding the row, with the approval two levels down.

The cause is that the window was built on `ManagerLayout` before §4 existed, and §9 then let the tools "gain the header
role … without changes of their own". A tool is a destination (§2.6) and takes the screen as Packages' screen 5 does.

### 15.2 The screen

The big-screen form, full screen in the safe area, dark unless the player chooses light, at scale height / 800. Positions
in design units at 1280 × 800; at 16:9 the extra width goes to the list and the pane in the same shares.

| Element | Position, size | Content |
|---|---|---|
| Header (§3.2) | 64,40 1152 × 48 | `CORES` in Title; facts right-aligned in Caption Muted: `17 cores · 5 installed · ↑ 1 update · catalogue read 31 Oct 2026`, a count needing the player in Attention with `⚠` (`⚠ 1 to approve`) |
| Section strip | 64,100 1152 × 44 | the consoles only: `ALL`, then each console the runtime names, in the systems table's order; the search box 240 × 44 at the right, opening the docked keyboard |
| List | 64,160 691 × 432 | eight rows of 54 on InputSurface; columns Core (star), Console, Status (§15.3); `FitsWidth` |
| Apply bar | 64,610 691 × 88 | the frame's two lines under the list: the summary, the last outcome, or the progress bar and its line; then `Apply…` 170 × 44 with the Start glyph, the space meter 200 × 12 and `/Cores 54 MB · 410 GB free` |
| Pane | 771,160 445 × 538 | SurfaceRaised, radius 8, padding 24: the focused core in sentences (§15.4), spanning the list and the apply bar |
| Help bar | 0,702 1280 × 58 | generated from the roles and the focused row's actions (§15.5) |

The content's share is 0.6 (`AdaptiveLayout.ContentShare`), the detail beside it (`DetailPlacement.Beside`, LunaP
§205.5). The pane spans the content and the status rows, which the frame's big-screen arrangement does not yet do
(§15.9). Measured in the mockup at 1280 × 800: the name column 302–331 units, the console column 167–195, the status
column 117–150; a fourth column (the source) would leave the name 150, under `Mupen64Plus-Next`'s 190, so the source is
said in the pane and in the status words instead. **[M]**

### 15.3 The list: one list, built in and hosted together

Every core EmuSen can run, in one list: the project's, libretro's and the player's own. Rows are ordered by console in
the strip's order, then the console's default first. The PlayStation 2's default is PCEE2 while Sapphire is under
development (`EmuSen_Cores.md` §4.8, `man cores`), so Sapphire reads `Built in` there and PCEE2 `● Default` once approved. The name shown is the core's own, `Snes9x`, `Nestopia UE`, `PCEE2`:
a libretro core's `corename`, which `EmuSen_Cores.md` §4.1 records as information and which the hosted list's `Core`
column already uses; the `.info`'s display name (`Nintendo - SNES / SFC (Snes9x - Current)`) is in the pane's caption.
The status words, one owner (`CoresWords.Status`), a glyph beside a word (§5.7):

| Status | Colour | Meaning |
|---|---|---|
| `● Default` | Text | the console's engine, whoever made it |
| `Built in` | Muted | ships with EmuSen, not the default |
| `✓ Installed` | Success | a hosted core installed and approved |
| `↑ Update` | Attention | the index offers a newer build |
| `⚠ To approve` | Attention | installed, or added by the player, and not approved |
| `Available` | Muted | listed, not installed |
| `Not supported` | Muted | the host cannot run it; the pane says why |
| `Installing…`, `Updating…`, `Removing…` | Muted | while its job runs |

A player's own core reads as the others do; the pane's first line says `Added by you`. The desktop's table keeps a
Source column (§15.7). A marked row shows its mark's word in Accent in the status column (`install`, `update`,
`remove`); the frame's mark box is not drawn on an unmarked row (Q4 of §15.11).

### 15.4 The pane: what the player is told

The focused core in sentences, Body 22, each fact once and in order, with nothing raw: no address, path, CRC-32, hash,
`corename` or `.info` line appears here (they are the core's Details page, §15.5). The title is the core's name in Title;
the caption in Caption Muted its source, console and, for a libretro core, the name libretro lists it under.

1. What it runs, and through what: `Runs PlayStation 2 games.` for a project core, `… through EmuSen's libretro host, in
   a sandbox of its own.` for a hosted one.
2. Who made it, under what licence, and the build: `By the PCSX2 Team, under the GPLv3. Nightly build of 31 October
   2026, 41 MB, downloaded today.`; for a project core `Built into EmuSen and updated with it, so there is nothing to
   install or approve. Version 0.44.`
3. Firmware, rewritten for the player: the console's firmware in the console's words from the systems table (`a
   PlayStation 2 BIOS`), whether it is required or optional, that EmuSen never downloads it, and then on its own line
   `✓ You have one.` in Success or `⚠ None found yet. Firmware, under X, says where it goes.` in Attention, from the
   firmware settings' recognition by content (`EmuSen_Cores.md` §4.5). The `.info`'s `firmwareN_path` is never shown.
4. The trust state: `⚠ Waiting for your approval. Nothing downloaded runs until you approve it.` in Attention; `✓
   Approved on 1 November 2026, held to its hash.` in Success; `Not supported: it needs OpenGL, and EmuSen's host
   draws through software and Vulkan.`; nothing for a project core.
5. What the project found: `This exact build was tested by EmuSen on <date>.`, `An earlier build was tested on
   <date>.` or `Not tested by EmuSen.`, Muted.
6. The default: `● The PlayStation 2's engine: every PlayStation 2 game opens on it unless the game's own options choose
   another.`; for an update, `A newer build of <date> is offered; the build installed now is kept for Roll Back.`; where
   a build is kept, `The build of <date> is kept for Roll Back.`
7. While a job runs, the steps as `EmuSen_Cores.md` §5.2 orders them, ticked as each `checked` event arrives: `✓ Secure
   connection to buildbot.libretro.com`, `↓ Download · 18 MB of 41 MB`, then `Checksum, one file of the right name, its
   size`, `Built for this machine`, `SHA-256 recorded`, and `Then it waits for your approval`.

The mockups hold the longest case (a hosted core waiting for approval, with firmware and a tested line) inside the pane
at both sizes with 60 units to spare at 1280 × 800 **[M]**; the pane scrolls only for a core whose `.info` names several
firmware files.

### 15.5 The pad map

The grammar of §2.1, with nothing window-specific: **A** does the focused row's one action, **X** opens the core's
page, **Select** the list's tools, **the shoulders** step consoles, **Start** applies the marked changes, **B** goes
back to Home. The help bar names A by its action, so the row says what A does.

| Row's status | A | Help bar |
|---|---|---|
| Available | `Install…` | `A Install… · B Back · L1/R1 Console · X More · Select List · Start Apply…` |
| To approve | `Approve…` | `A Approve… · …` |
| Update | `Update…` | `A Update… · …` |
| Installed, Built in, not the default | `Make Default` | `A Make Default · …` |
| Default | `Options ▸` (the console's settings at this engine's options) | `A Options · …` |
| Not supported, or a job running | nothing | `B Back · L1/R1 Console · Select List` |

**X, the core's page**, in the same frame with the breadcrumb `Cores`: `Mark for install` (or update, or remove, so a
batch is still made), `Options ▸`, `Firmware ▸` (the existing pane), `Verify`, `Roll Back…`, `Remove…`, `Revoke
Approval…` (a core the player added, through the runtime's `core_revoke`), `Open Folder`, `Licence ▸` (the folder's
licence text), `Details ▸` (every fact as the tool holds it: the address, the build and the date fetched, the index's
CRC-32, the SHA-256 in groups of eight, the `corename`, the `.info`'s notes and permissions). Each row is offered only
where it applies; a project core's page holds `Options ▸`, `Firmware ▸` and `Details ▸`.

**Select, the List card** (§2.4's card over the list): `Search`, `Filter ◂ All ▸` (All · Installed · Available ·
Updates · To approve · Not supported, the five state chips of §15.1), `Sort ◂ Console ▸` (Console · Name · Status),
`Check for Updates`, `Update All…`, `Verify All`. The filter and the sort apply across the chosen console section; the
context pane shows the current filter in words.

**Start** opens the plan for the marked rows, as the frame does, in the question form of §15.6, and is named in the help
bar only while something is marked.

### 15.6 The question form

One form for every question the tool asks, the plan included: a centred sheet (`SheetLayer`'s, §3.2) 720 wide on
SurfaceRaised with the Border edge, over the screen under the Scrim; the question in Title (`Approve PCEE2?`, `Install
PCEE2?`, `Apply 3 changes?`); the sentences in Body; then two rows, `No` first and focused, `Yes`; the help bar `A Choose
· B No`. B answers No. The approval's sentences are `CoreApproval.Message`'s, rewritten in this register: what it is and
where it came from; by whom, under what licence, for what consoles; for a downloaded core, that libretro signs no build,
so approving trusts the build the buildbot served that day, that the hash holds EmuSen to exactly that build, and that
EmuSen's own proxy runs it in a sandbox; that it runs with the player's permissions, and that a changed file is asked
about again; what the project found of this build; and the SHA-256 in groups of eight on two lines, in Caption. The
install question carries the plan's words (`EmuSen_Cores.md` §2.1). On the desktop the same words are a dialog with the
verb on its buttons (`Not Now` / `Approve`, `Cancel` / `Install`), as desktop dialogs name them.

### 15.7 The desktop form

`ManagerLayout`'s desktop form in the tokens, with the same facts in the same words. The toolbar carries the focused
row's action under its own name (`Install…`, `Approve…`, `Update…`, `Make Default`), then `Options…`, a `More ▾` menu
holding the rows of the core's page, a separator and `Check for Updates`; the search box at its right with a `Show`
dropdown beside it (All · Installed · Available · Updates · To approve · Not supported), the List card's desktop
rendering. The sidebar lists `All` and the consoles. The table's columns: Core, Console, Source (`EmuSen`, `libretro`,
`Added by you`), Version, Status, Size; the Default column is folded into Status. The detail beneath the table is the
pane's sentences in Body 13, and holds no buttons; a right click on a row opens the `More` menu. Enter does what A does,
Space marks, F6 moves between the roles. The approval and the plan are dialogs (§15.6).

### 15.8 The tile on Home

`Cores` on Home's places row (§4.1) with one live line, read offline from the runtime's `cores` listing and the core
tool's `core-ls` (never a fetch): `⚠ 1 to approve` in Attention where a core waits, else `↑ 2 updates`, else `17 cores
· 5 installed`. A opens the screen of §15.2; B returns to Home.

### 15.9 What is retired, and what LunaP gains

Retired from the window of §15.1: the centred sheet frame around the tool; the state chips in the strip; the Mark
column as the first idiom, and X as its cycle; the detail page under A; the action row and the page's duplicate
buttons; the raw lines; the lone-glyph Default column; the second help bar.

Built in LunaP first, each an optional property or a minor version, each general to the kit and taken up by Packages,
Scraper and Library next (§16):

1. **A tool screen**: a host presents an `AdaptiveLayout` full screen in the safe area (`SheetLayer.ShowScreen`, or a
   `Frame` the host owns), the sheet's centred panel kept for questions and the sheets not yet rebuilt.
2. **The pane spanning the status**: in the Beside placement the detail column stands beside the content and the status
   rows both, since the apply bar belongs to the list.
3. **A row's action**: `ManagerLayout.ActionFor(row)` gives the `LunaAction` A invokes for the focused row, named in the
   help bar in place of `Select`; null leaves A idle.
4. **A row's page**: `ManagerLayout.PageFor(row)` gives the rows X opens as a page in the frame, the detail page's
   mechanism with rows and a breadcrumb.
5. **The question form**: the plan sheet drawn as §15.6 on the big screen, and `Ask(title, sentences)` for a tool's own
   question, so the plan and the approval are one form.
6. **The List card**: `AdaptiveLayout.ListTools`, a Card of Search, Filter and Sort over the content, with the tool's
   list-wide actions as its further rows; the filters are `AdaptiveSection`s of a second kind, drawn on the desktop as a
   dropdown beside the search box.

Nothing in Cores reaches past these: `CoresPane` hands the frame its rows, status words, sentences, actions and
questions, and the frame draws both forms.

### 15.10 Built 2026-11-01

`CoresPane` on the frame of §15.9, after the three tools (§16.1) and with the frame as the tester accepted it on the
comparison pictures: the pane beside the list at 0.6; `ALL` and the consoles on the shoulders (`Console`); the six
states as the List card's filter; the header `CORES` with its facts, a count needing the player first (`⚠ 1 to
approve`, `↑ 1 update`), then the catalogue's date; the list's columns Core · Console · Status, the status words and
their colours one owner's (`CoresWords.Status`, `CoresWords.Tone`), the name a libretro core's `corename` and the
`.info`'s name in the pane's caption (`CoreRow.ListedAs`); A the row's one action (`Approve…`, `Install…`,
`Update…`, `Make Default`, `Options`), named in the help bar and unnamed where a row has none; X the core's page
(marks, Options, Firmware, Approve…, Verify, Remove…, Open Folder, Details); Select the List card with Check for
Updates and Verify; the install, update and remove questions titled by `CoresPlan.Title` (`Install PCEE2?`) and the
approval asked as the frame's card on the big screen (`Approve PCEE2?`, the hash in its words) and as the host's
dialog on the desktop; the pane in the sentences of §15.4; the Details page with the raw facts; no button on the big
screen. The desktop keeps its toolbar.

Not built here: the firmware's `✓ You have one` line, since the tool has no answer from the firmware settings yet
(§15.4 item 3 waits for that call); the page's `Roll Back…`, `Licence` and `Revoke Approval…` rows, which C4 had not
built either (`EmuSen_Cores.md` §9.1); the tile on Home (§15.8), which comes with U5.

Tests, measured on 2026-11-01, on the Rust runtime: `CoresWindowTests` and `CoresWindowRuntimeTests` 36 (the listing's
filters, the status words with their glyphs and colours, the help bar naming the row's action, the approval asked as the
frame's card by pad and recorded, the desktop's dialog, the fit at both sizes), `LeafAssemblyTests`; the fit audit and the
recordings of the five Cores states at both sizes, 20; the comparison pictures at 1280 × 800 and 1920 × 1080, 4. Five
seeded faults, each caught: the update's status without its glyph and case; no action on a core waiting for approval;
the To approve filter matching nothing; the facts leaving out the count to approve; the approval asked through the
host's dialog on the big screen.

**Landing, 2026-11-01.** The first landing on the merged tree failed 39 recordings of the Pause Card, the Home card and
the main menu: since the List card and the question's cover, LunaP's `AdaptiveLayout` holds its arranged form in a
layers panel, so every recording of a card read an unnamed panel at its root where `AdaptiveCardScrim` had been, one
level up; the dim was still drawn beneath the card. The panel is named `AdaptiveLayers`, `CardFormTests` asserts the
card's scrim paints the dim across the layout (LunaP §208.12), and every audited state was re-recorded for the one
level. Found alongside: the sheet layer's handler had set the status bar visible whenever a sheet other than a screen
was presented, over the bar's own rules (the setting, its parts, the themed library); the screen case is now one more
term of `ApplyStatusBar`, which the handler calls. Two failures of that landing were not this stage's: `ScrapeStatusIdle`
is recorded at a frame of 383 and measures 385 in some whole-suite runs and vice versa (the landing of 2026-10-31 at
23:27 failed it the other way round), and `GraphicsSettingsWindowLayoutTests` reads `ConsolesConfig` through the
process-wide `ConfigStore.OverrideDirectory` that 141 test classes set, so a parallel neighbour can move its file; both
pass on their own.

Decided 2026-11-01, on the tester's standing rule that no test is left flaky: the symbol glyphs resolve to one bundled
face on every machine. `ScrapeStatusIdle`'s 2 px was the fallback for `▸`, a glyph Barlow lacks, chosen by what the
process had loaded first; LunaP now declares the face (§5.2): every symbol block to Adwaita Mono, bundled under the OFL
beside Barlow (LunaP §208.13), for Mistress and for the headless harness alike. Noto Sans Symbols 2 and STIX Two Math
were each tried first and rejected the same night: a Caption line holding one glyph grew from 26 to 34 px under the
first (its descent) and to 30 under the second (its line gap), where Adwaita Mono keeps the 26 the machine's own
fallback had given. Every audited state was re-recorded for the faces that changed.

### 15.11 Questions

1. **[decision]** A does the row's one action where the detail is beside the content, and opens the page where the
   detail is a page: Cores acts, Packages (screen 5) opens. *Recommended:* yes, as a rule of the family, so that the
   help bar's A always says which.
2. **[decision]** The six kit pieces of §15.9, built in LunaP before `CoresPane` is touched. *Recommended:* yes.
3. **[decision]** The tool screen (§15.9 item 1) replaces the centred sheet for all four tools at once, since the frame
   is one control. *Recommended:* yes; each tool's states are re-recorded as it is rebuilt.
4. **[routine]** The mark box drawn only on a marked row, as the mark's word in the status column. *Recommended:* yes;
   on Cores a mark is the exception and A's action the rule.
5. **[routine]** A libretro core named by its `corename` in the list, the `.info`'s display name in the pane.
   *Recommended:* yes.
6. **[routine]** The source said in the status words and the pane on the big screen, and as a column on the desktop,
   rather than a fourth column at 1280 (§15.2). *Recommended:* yes.

**Decided 2026-11-01 by the tester:** the three decisions as recommended, the three routine items accepted, with one
correction: the PlayStation 2's default is PCEE2 while Sapphire is under development (§15.3).

---

## 16. The family's screens, held against the grammar (2026-11-01)

Each window built or queued, checked against §2.1, §3 and §4 for the faults §15.1 found: a tool in the centred sheet,
filters as a chip wall, a detail as a page of raw lines, actions as a wall of buttons. A short entry each; a mockup
only where the change is large (Cores, §15).

### 16.1 Built

- **Packages** (screen 5, `EmuSen_Packages.md` §3). The same frame faults as Cores: the centred sheet, and a strip of
  `All`, the consoles and three states (`Packaged`, `Not yet`, `Problems`). Changes: the tool screen; the three states to
  the List card's Filter. Kept: A opens the package's page, since its detail is a list of files, patches and layers,
  not sentences; X marks; Start applies. Five actions wrap on one line.
- **Scraper** (`EmuSen_LibraryTools.md` §9). The strip holds two groups, the systems and the collections: the
  collections are a filter and go to the List card; the systems stay on the shoulders. Three checkable actions
  (`Theme`, `Videos`, `Overwrite`) are the run's options, not actions, and become the plan question's rows (`Scrape 41
  games? Kinds ◂ The theme's ▸ · Videos ◂ Off ▸ · What is there ◂ Keep ▸`); `Mark All` goes to the List card; `Stop`
  is the apply bar's while a run goes. The detail is the game's media as pictures, which the Beside placement fits.
- **Library Manager** (`EmuSen_LibraryTools.md` §10). Six findings on the strip and no consoles: fits as it is. A
  becomes the file's one action, `Hide` or `Show again`; X opens the file's page (`Show in File Manager`, its hashes and
  matches); `Save List…` and `Check Again` go to the List card; `Stop` to the bar. The detail beside, in sentences
  (what the file is, what it matches, which others are the same dump).
- **Cores**: §15.

**Built 2026-11-01, the three tools on the frame.** LunaP's six pieces (its `docs/LunaP.md` §208) and the tool screen
first; then each tool's host presents it as a screen (`ToolPaneWindow`, `SheetLayer.Screen`), full screen in the safe
area with the header role and the frame's own help bar, in place of the centred sheet of 812 × 668 and its second hint
bar. Packages: the consoles on the shoulders, `Packaged`, `Not yet` and `Problems` as the List card's filter, `Refresh` a
row of the card, the header `PACKAGES` with its counts; A opens the package's page, X marks, Start applies, and the big
screen draws no action buttons. Scraper: `All` and the systems on the shoulders; the five criteria and the collections as
the filter, sent to `scrape ls` beside the system's words (`ScraperPlan.Line` takes both); `Theme`, `Videos` and
`Overwrite` as rows of the plan's question, read back when the run is confirmed; `Mark All` a row of the card; `Stop` the
one button; the detail beside the list at 0.6. Library Manager: its six findings stay the strip; A asks to hide the file
or to show it again, through the plan's question; X opens the file's page (`Mark to hide`, `Show in File Manager`);
`Save List…` and `Check Again` rows of the card; `Stop` the one button; the detail beside the list. The states of all
four tools re-recorded at both sizes (`EmuSen_InterfaceSpec.md`), seventeen states, held by `WindowFitAuditTests`.
Not yet moved: the Scraper's and the Library Manager's detail content, which stays each tool's own lines rather than
the sentences of §15.4; a stage of their own.
Tests, measured on 2026-11-01: `LibraryWindowTests`, `PackagesWindowTests`, `ScraperWindowTests`, `CoresWindowTests` and
`CoresWindowRuntimeTests`, 33; the fit audit and the recordings of the four tools' states, 72 cases; LunaP's `ToolFrameTests`,
20, with its four seeded faults (its §208.9). Two seeded faults here, each caught: the scraper's filter not sent beside the
system (`ScraperWindowTests`, two cases); the Library Manager's A left idle (`LibraryWindowTests`, two cases).

### 16.2 Queued

- **Themes** (a tile of §2.6). Three sheets today (`ThemeSettingsWindow`, `ThemeBrowserWindow`, `ThemeDetailWindow`)
  become one screen on the frame with `TileGrid` content (§3.2, U4–U5) and the detail beside: the theme's picture,
  licence, variants and colour schemes; A uses the focused theme, X opens its page (variants, font size, About, Remove),
  the List card searches and filters installed from available. The nesting is the cramped pattern here.
- **Controllers** (§2.7, Settings ▸ Controllers). `Players ▸`, `Bindings ▸`, `Profiles ▸` as rows fit the
  SettingsLayout. The bindings page shows the controller drawing as content with the focused control's binding in the
  pane, so the drawing carries regions and the pane the words: with four stick regions the drawing's labels fell under
  the floor at 1280 × 800 (`EmuSen_Cores.md` §4.11). The per-console tabs retire; the console is the section.
- **Display** (§2.7, screen 6). The rows and the preview pane fit as drawn. The shader window keeps a second,
  hand-written arrangement for its sheet (`EmuSen_LibraryTools.md` §1a) and a sliders page: both become rows of
  `Shader ◂ ▸` and a page of its levers, each row with the preview; the per-console profile lives under Consoles ▸ the
  console ▸ `CRT profile`.
- **Cheats** (§2.5, §2.6). Two windows today, the database (three lists: systems, games, cheats) and the active cheats:
  the game in hand gets `Options ▸ Cheats` on its page, rows `◂ On ▸`; the database is a tool on the frame, the consoles
  on the shoulders, games as rows, A opening the game's cheats as a page of rows, the List card searching. The three-list
  sheet is the cramped pattern.
- **Home and Setup** (screen 1, §1b). Fits as drawn; the places read `Library`, not `Library Manager`, at 150 wide.
  Setup is the tools' panes in order under one header whose subtitle counts the steps (`Step 2 of 5`), `Next` on A and
  `Back` on B, from the same descriptions.
- **Mods, X8** (`EmuSen_Modding.md` §4). No form is declared yet. It is to be declared as roles before it is built: a
  tool on the frame, the game's packs as rows switched `◂ On ▸` by A, X the pack's page, the detail beside with the
  pack's preview; else it repeats §15.1.
- **Debugger**. A desktop tool that needs a running game (`EmuSen_LibraryTools.md` §1b); no big-screen form, so the
  pad grammar does not apply; the fit audit holds it at 1280 × 800 and 1920 × 1200 as every window.
- **Settings' six sections, U6** (screen 6). Fit as drawn. `Consoles ▸` a console `▸ Engine ◂ ▸` is the same value as
  Cores' `Make Default` (one setting, two doors, `EmuSen_Cores.md` §4.3); the Consoles section lists every console as a
  row, eight visible, the rest scrolling; `Rumble` only where it is reported (§12).
- **A game's page, U4** (screen 3). Fits as drawn at both sizes and at 1280 × 720, where the scale by height gives the
  same units with more width. Its Options rows take their values from Cores (`Engine ◂ Diamond (EmuSen, default) ▸`),
  the longest value 360 units at Row 26, inside a 1152 row.

---

## 17. The desktop (2026-11-01)

Asked by the tester on 2026-11-01: the desktop mode as a whole, designed as a native application within the unified
design, the same structure as the big screen (§2, §3) in the desktop's own scale (§5.2, §5.3), tokens and type. The
mockups are under `~/.cache/emusen/tmp/desktop-design/`, at 1280 × 800 and 1920 × 1200: `desktop-library` (and
`-light`), `desktop-home`, `desktop-game-page`, `desktop-cores-waiting`, `desktop-cores-approve`,
`desktop-cores-installing`, `desktop-packages`, `desktop-settings-light`, `desktop-game-running`. Positions are pixels
in a 1280 × 800 window; a larger window gives the room to the content and the panes, never to the type.

### 17.1 The desktop as built, measured

Rendered on 2026-11-01 (`DesktopButtonPictureTool`, the pictures under `~/.cache/emusen/bigpicture/png/desktop-button/`
and `desktop-options/`). **[M]** The window is OpenEmu's shape: a menu bar (File, Emulation, View, Settings), a sidebar
of Library, Consoles and Collections with counts, a segmented toolbar (`as Grid · as List · Library · Save States ·
Screenshots`), the cover-size slider, a search field, a line `8 games under /tmp/…/Roms`, the generated cards, the hint
`Double-click a title, or press Enter, to start it.` and a status bar. Four things are not the desktop's:

1. The pad menu is the big screen's `MAIN MENU`, blurred over the library at the big screen's size, with a help bar of
   pad glyphs, when a pad is pressed on the desktop; and a game's options open as a sheet of stacked full-width buttons.
2. `Save States` and `Screenshots` are views of the library's toolbar, though they belong to one game.
3. Settings are ten windows: Preferences (five tabs), Graphics, Shaders, Memory Cards, Firmware, Controllers, Cheats, the
   cheat database, the console settings and Themes.
4. The tools (`ManagerLayout`'s desktop form) put the detail beneath the table in a vertical split and every action in
   the toolbar, so the detail reads as a log and the toolbar as a wall.

### 17.2 The window

One window, in the platform's conventions. **Menu bar** 28: File · Game · View · Tools · Settings · Help (`Emulation`
becomes `Game`). **Sidebar** 200 (160–320, `Ctrl+\` hides it): `Home` first, then LIBRARY (Games, Favourites, Recently
Played), CONSOLES with counts, COLLECTIONS; `SourceList` rows 24 in Body 13, headings in Caption upper Muted, the chosen
row the bar. **Content** with a **header row** 48: the view's name in Title 20 upper (`SUPER NINTENDO`, `HOME`, `CORES`)
and its facts in Caption Muted, the view's tools at the right (search 220 × 28, sort, grid or list, the cover size).
**Status bar** 24: one line at the left (the focused game, a job's progress, `State saved to Slot 3`), the runtime and
the count at the right; it replaces the hint line. Hover is a 1-px Border on a tile and SurfaceRaised on a row;
selection the 2-px Accent ring on a tile and the bar on a row; focus never moves without the player. Panes are
resizable by their dividers and remembered per window. Light or dark follows the system unless Settings ▸ Display says
otherwise; every colour is a token of §5.1 (`desktop-library-light`, `desktop-settings-light`).

| Menu | Items |
|---|---|
| File | Open ROM… · Add ROM Folder… · Import Collection… · Quit |
| Game | Play `Enter` · Play from the Start · Game Page `Space` · Favourite `Ctrl+D` · Edit Details… `Ctrl+I` · Scrape This Game… · — · Pause `Esc` · Resume · Save State ▸ `F5` · Load State ▸ `F8` · Rewind `Backspace` · Speed ▸ · — · Screenshot `F12` · Cheats… · Game Options… · — · Pause Card `F4` · Back to Library · Restart… · Quit Game… `Ctrl+W` |
| View | Grid `Ctrl+1` · List `Ctrl+2` · Cover Size ▸ · Sort ▸ · Filter ▸ · — · Sidebar `Ctrl+\` · Full Screen `F11` · Big Picture `F10` |
| Tools | Cores… · Packages… · Scraper… · Library Manager… · Cheats… · Themes… · — · Shell… · Debugger… |
| Settings | Controllers… · Display… · Sound… · Consoles… · Library… · EmuSen… · — · Settings… `Ctrl+,` |

### 17.3 The library

The header `SUPER NINTENDO · 5 games · 1 favourite · 2 h 40 min played`; the grid of generated cards 100 × 133 (the
slider 80–200), gap 16, the title in Body and `console · year` in Caption under each, a favourite's Accent heart; the list
view's columns Title · Console · Year · Developer · Play time · Last played, sortable, rows 24. Double-click or `Enter`
plays, `Space` opens the game's page, a right click the context menu of `desktop-library` (Play, Play from the Start,
Game Page, Favourite, Add to Collection, Options…, Edit Details…, Scrape This Game…, Cover Art, Show in File Manager);
a game dragged onto a collection joins it. Retired: `Save States` and `Screenshots` as library views (they are the
game's page's States and Media), the `n games under <path>` line (the folder is Settings ▸ Library's) and the hint line.

### 17.4 A game's page

Screen 3's desktop paragraph (§4), as `desktop-game-page` draws it: `‹ Back` 80 × 28; the hero 1080 × 200 under the OnArt
gradient; the cover 160 × 200 raised over its foot; the title in Display 24 OnArt and the facts in Body; `Play` filled
Accent, `Resume: Slot 3 · Marble Gallery · yesterday ▾` a split button, `♥ Favourite`; tabs OVERVIEW · STATES · OPTIONS ·
MEDIA · DETAILS in Strong 14 with the Accent underline. STATES: cards 160 × 120, `Load` and `Delete` on the selected and
hovered card, an outlined `Empty`; OPTIONS: `FieldRow`s 48 with the sentence under each (Engine, Cheats, Display crop,
Controller profile, Package); MEDIA: thumbnails, double-click full screen; DETAILS: the metadata as a form with Scrape
This Game…, Edit, Open Folder, Hide from Library…. Back returns to the grid on the same game. Retired: the game options
sheet of stacked buttons, the metadata editor window (DETAILS' Edit).

### 17.5 Home, the hub

Screen 1's desktop paragraph, as `desktop-home` draws it: the Continue card 1032 × 180 with the paused frame, `CONTINUE
PLAYING`, the title in Display 24, the facts, and `Resume` with the focus; RECENT, a shelf of 100 × 133 cards; PLACES as a
4 × 2 grid of 246 × 64 tiles with a live line each: Games, Scraper, Packages, Library Manager, Cores (`⚠ 1 to approve` in
Attention), Cheats, Settings, Big Picture. A place fills the content area with `‹ Back` (`EmuSen_LibraryTools.md` §1b);
Tools ▸ opens the same pane as a window beside the library, two hosts of one description. Home is the window's first
view, and Setup is its first-run face: the same panes in order under one header, `Next` and `Back`.

### 17.6 The tools on the frame

Cores in full (`desktop-cores-waiting`, `-approve`, `-installing`), Packages for the family (`desktop-packages`). The
frame's desktop form, remade for a desk:

- **The header row**: the tool's name and facts at the left; at the right the focused row's one action under its own
  name as the first button, filled Accent (`Approve…`, `Install…`, `Update…`, `Make Default`), then the row's other
  actions (`Make Default`, `Options…`), `More ▾` (the row's page of §15.5 as a menu) and the list-wide actions (`Check
  for Updates`). The buttons change with the row, so the toolbar is never a wall.
- **The sidebar** of sections (`All`, the consoles), 180 wide.
- **The filter row** above the table: `Show: All ▾` (the List card's filter) and the search box (`Ctrl+F`).
- **The table**: a header row of sortable columns (Core · Console · Source · Version · Status · Size; Console sorted by
  default; Version not sortable), rows 24 at Body 13 on InputSurface, the status words of §15.3 with their glyphs, the
  selected row's bar, a hovered row on SurfaceRaised; on the selected row its one action as a 20-px button at the
  right, so the mouse has it in place. A marked row shows its mark's word in the status column. A right click opens the
  row's page as a context menu.
- **The pane** at the right (320 at 1280, 440 at 1920, resizable 280–520): the core's name in Display 24, its caption,
  then the sentences of §15.4 in Body 13, and a last Caption line of the keys (`Enter approves · Space marks · Ctrl+I
  shows every detail · Delete removes`). No buttons in the pane.
- **The apply bar** under the table: the summary, the last outcome or the progress bar and its line; `Apply…`
  (`Ctrl+Enter`), the space meter and its words.
- **The question**: a dialog 520 wide over the dimmed window with §15.6's words, the verbs on its buttons (`Not Now` ·
  `Approve`, the default filled Accent), `Esc` answering no.
- **Keys**: `Enter` the row's action · `Space` mark · `Delete` remove… · `Ctrl+I` details · `F5` check for updates ·
  `Ctrl+Enter` apply · `Esc` clears the search, then closes.

Packages, Scraper and Library Manager take the same frame with their own columns, filters and actions (§16.1); a row 24
tall at Body 13 is the density, never the big screen's 54. Retired: the vertical split with the detail beneath, the
toolbar of every action, the Mark column, and the big screen's hint bar drawn on a desktop window.

### 17.7 Settings

One window, 960 × 640 at 1280 and growing with the screen (`desktop-settings-light`): the six sections in a `SourceList`
200 wide; rows 48 with the setting in Body, its sentence in Caption Muted under it, and its control at the right (a
dropdown `▾`, a switch, `Edit…`); a preview pane 276 wide where the row has a picture (the shader on a frame, the crop's
edges, the text sizes, the two variants) with its sentence under it; a search field at the top right filtering rows
across all six. Settings ▸ a section in the menu bar opens the window at it; `Ctrl+,` opens it. A change applies at
once. Retired into its sections: Preferences' five tabs, Graphics Settings, Shader Settings, Memory Cards, Firmware, the
console settings window (Consoles ▸ the console), the controller window's tabs (Controllers ▸ Bindings, the console a
section).

### 17.8 A game running

`desktop-game-running`: the game fills the content area under the menu bar, the sidebar hidden, letterboxed on Void; the
pointer hides after two seconds and moving it shows the heads-up bar (OpenEmu's, 442 × 42, 19 px above the bottom,
`EmuSen_Settings_Reference.md` §4.34) in the tokens: `Pause · Save ▾ · Load ▾ · Rewind · Volume · Full Screen · Stop`.
`Esc` pauses and shows the bar; `F4` opens the Pause Card for the keyboard, the same card at 40 per cent of the content
area over the dimmed frame (§4.4); the Game menu carries every action with its key; a pad connected opens the cards and
the ring as on the big screen (§6). The status bar reads the game, its frame rate and the last state saved. Retired: the
big screen's pad menu drawn over the desktop library (a pad opens the Home card, §2.3), the game options sheet.

### 17.9 What is retired or rebuilt, in order

| Stage | What it builds | Holds |
|---|---|---|
| **D1** | The window: Home in the sidebar and first; the header row with the view's tools; the Game, Tools and Settings menus; the status bar's words; the view switch, the path line and the hint line retired | the main window's states re-recorded at 1280 × 800 and 1920 × 1200 |
| **D2** | The tools' desktop frame in LunaP (the pane at the right, the row's action in the header and on the row, the filter row, the question dialog, the context menu), Cores first, then Packages, Scraper and Library Manager | each tool's desktop states; the fit audit |
| **D3** | A game's page, U4's desktop half; the game options sheet, the metadata editor window and the two library views retired | the page's states |
| **D4** | Home and Setup, U5's desktop half | Home's states |
| **D5** | Settings, U6's desktop half; Preferences' tabs and the nine settings windows retired | every settings window's states moved |
| **D6** | In-game: the heads-up bar in the tokens, the Game menu, `F4`'s card; the pad menu's desktop rendering retired | the in-game states |

D1 and D2 go beside the big screen's Cores work; D3 to D6 go with U4 to U6, each stage building both halves of its
screen from one description (`EmuSen_LibraryTools.md` §1a).

### 17.10 Questions

1. **[decision]** The tools' pane at the right of the table on the desktop, as `desktop-cores-waiting` draws it, in
   place of the frame's detail beneath. *Recommended:* yes; a list and its pane are the desktop's list-detail form.
2. **[decision]** `Save States` and `Screenshots` retired as library views, into the game's page. *Recommended:* yes.
3. **[routine]** `Emulation` renamed `Game`, and Home first in the sidebar and the window's first view.
   *Recommended:* yes.

**Decided 2026-11-01 by the tester:** §17 as pictured, the three questions as recommended. D1 to D6 are built after the
big screen's Cores work and U4 to U7, not before.

---

## 18. The remaining windows (2026-11-01)

Decided 2026-11-01 by the tester: every window Mistress has today has a form under the unified design, as this section
gives it, and the questions of §18.15 as recommended. The design was drawn on 2026-11-01 at 1280 × 800 and 1920 × 1080
for the big screen and 1280 × 800 for the desktop, in the tokens of §5, on the synthetic library's pictures (a flat
colour with the game's name on it, as `SyntheticLibrary` describes them).

Positions are design units at 1280 × 800 (§4); at 16:9 the extra width goes to the list and the pane in the same shares
(§15.2). Nothing here adds a form: every screen is the frame of §3 (header, sections, content, detail, actions, help
bar), the Card form of §3.2, the tool screen of §15.9 or the question form of §15.6, with the pad grammar of §2.1.

### 18.0 The inventory

Every window, sheet, popup and menu Mistress has today (`EmuSen.Mistress/Views`, `BigPicture`, the states of
`WindowFitAuditTests`, `EmuSen_InterfaceSpec.md`), with what it becomes and whether that is (a) designed and approved,
(b) designed before this section and built later (U4 to U7, the queued entries of §16.2, the thin designs filled in
below) or (c) designed here.

| Today | Under the redesign | State |
|---|---|---|
| The theme's system view, gamelist view, help bar, sounds | the theme's, untouched (§7) | a |
| The big-picture grid ("EmuSen (built in)") | Home and Games, the own look (U5, screens 1 and 2) | b |
| The pad menu's Main Menu (Start over a theme) | the Home card (U3, built) | a |
| Pad menu ▸ Library ▸ (Packages, Cores, Compressed Discs, Theme Settings, Gamelist Options) | Home card ▸ Tools ▸, each tool with its live line; Compressed Discs into Packages (Q2) | a |
| Pad menu ▸ Settings ▸ (Controllers, Display, Graphics, Shaders, Preferences, Cheats, Memory Cards) | the six sections of §2.7 (U6, screen 6) | b |
| Pad menu ▸ EmuSen ▸ (Shell, Big Picture, Full Screen, Exit) | Power ▸ (§0.2); the Shell under Settings ▸ EmuSen (Q3) | a / c |
| Gamelist Options (Select in a themed view: Jump To, Sort, Filter, Search, favourite, Edit Metadata, Scrape) | the List card (§18.1); the game's own actions on its page | c |
| `GamelistFilterWindow` | List card ▸ Filter | c |
| The on-screen keyboard over a sheet, big-picture search (North) | the docked keyboard under the content (§18.1) | c |
| `OptionListWindow` (a row's choices as a screen) | `◂ ▸` on the row; a page of choices where the values are many (§18.11) | c |
| `GameOptionsWindow` (the game options sheet: Engine, Game names, favourite) | the game's page ▸ Options (U4, screen 3) | b |
| `ActiveCheatsWindow` (per console tabs, this game's cheats) | Options ▸ Cheats on the page; Game Options ▸ Cheats in the card; the same rows | b |
| `CheatDatabaseWindow` (three lists) | Cheats, a tool on the frame (§16.2) | b |
| `GamePackageWindow` | Options ▸ Package ▸ (§18.2) | c |
| `MetadataEditorWindow` | Details ▸ Edit Details ▸ (§18.2) | c |
| `CoverPickerWindow` | Details ▸ Cover Art ▸ (§18.2) | c |
| `FindByNameWindow` (the scrape chooser) | Details ▸ Scrape This Game…, the question form with the matches as rows (§18.2) | c |
| `FolderEditorWindow` (a themed folder's link) | List card ▸ Folder ▸ when a folder is focused (Q5) | c |
| `ScreenshotWindow` | the picture full screen, from Media (§18.2) | c |
| `ScraperWindow`, `ScrapeStatusWindow` | Scraper on the tool screen (built); its detail in sentences (§18.7) | a / b |
| `PackagesWindow` | Packages on the tool screen (built) | a |
| `LibraryWindow` (Library Manager) | on the tool screen (built); its detail in sentences (§18.7) | a / b |
| `CoresWindow` | §15 (built) | a |
| `CompressedDiscsWindow` | retired into Packages: Convert to CHD is its action, the discs its filter (Q2) | c |
| `ThemeSettingsWindow`, `ThemeBrowserWindow`, `ThemeDetailWindow`, `ThemeAboutWindow` | Themes, one screen on the frame with tiles and a page (§16.2) | b |
| `CollectionSettingsWindow`, the collections | Collections, a tool on the frame, and a section of the library (§18.4) | c |
| `RomBrowserWindow` | the desktop's File ▸ Open ROM… (the platform's file dialog); Setup's ROM folder row | c |
| `LaunchScreen` (ES-DE's) | EmuSen's launch screen under every theme (§18.3) | c |
| `Screensaver` (Dim, Black, Slideshow) | kept; the slideshow with EmuSen's caption bar (§18.3) | c |
| `ResumeWindow` | the Resume question in the question form (§18.3) | c |
| `MainWindow.ConsoleStart` (a console started empty) | Home card ▸ Start Console ▸ (§18.3) | c |
| `InsertDiscWindow` | Pause Card ▸ Change Disc ▸ (§18.5) | c |
| The pad menu's game page (Start in a game) | the Pause Card (U2, built) | a |
| Game Options ▸ and its rows | the card's second page (§2.8); its pages filled in (§18.5) | a / b |
| `RewindReelWindow` (the reel as tiles) | the Rewind pane, and the reel page (§18.5) | a / c |
| `MemoryCardsWindow` | Game Options ▸ Memory Cards on disc consoles; Consoles ▸ the console ▸ Memory cards (§18.10) | c |
| Leave ▸, the quit question | the card's page and the question form | a |
| The HUD notices (`Screenshot saved`, `State saved`) | the notice (§18.6) | b |
| `GraphicsSettingsWindow` (per console tabs, crop, engines, memory card modes), `GameCropWindow` | Settings ▸ Consoles ▸ the console; Display's rows; This game's crop under the game's Display (U6, §16.2) | b |
| `ShaderSettingsWindow` (sheet, sliders, CRT) | Display ▸ CRT shader `◂ ▸` and Shader levers ▸ (§16.2) | b |
| `InputSettingsWindow` (bindings, the diagram, per console tabs) | Controllers ▸ Controller bindings, the drawing as content and the pane (§16.2) | b |
| `FirmwarePane` | Consoles ▸ the console ▸ Firmware ▸ (§18.10) | c |
| `PreferencesWindow` (Library, Gameplay, Appearance, Scraping, Firmware) | the six sections (U6) | b |
| `DebugSettingsWindow` | Settings ▸ EmuSen ▸ Logging ▸ | c |
| `HotaruWindow` (the Shell) | a tool on the frame with the docked keyboard (§18.9) | c |
| `VstopWindow` (the runtime dashboard) | a view of the Shell window on the desktop (§18.9) | c |
| Setup, first run (today only the defaults written) | Setup, the tools' panes in order (§16.2) | b |
| Mods (X8, no form declared) | a tool on the frame, as §16.2 asks (§18.8) | c |
| The Debugger (`EmuSen_Debugger.md` §4) | its desktop form on the tools' frame (§18.12) | b |
| Questions (`SheetLayer`'s) | the question form (§15.6) | a |
| The Guide-held chords, the Quick Access Menu | unchanged (§2.8) | a |
| The desktop window, library, menus, context menu, Home, the game's page, the tools' frame, Settings, the game running, F4's card, the pad's card | §17 as pictured, decided 2026-11-01 | a |
| Desktop Scraper, Library Manager, Cheats, Themes, Setup, Consoles | the family's frame, pictured first here | b |
| Desktop Mods, Shell, a collection's view | §18.8, §18.9, §18.4 | c |

Counting the rows' states (a row with two marks counts once for each): **a 22, b 21, c 27**.

### 18.1 The List card and search

The List card is the Card form over the content (§2.4, §15.5): panel 0,0 512 × 800, title row with the breadcrumb
`List` in Caption Muted and the section's name and count in Title (`Super Nintendo · 1,284 games`), rows from 64,156:
`Search ▸`, `Sort ◂ Name ▸` (Name · Last played · Most played · Year · Rating), `Filter ▸`, `Jump to ◂ G ▸`, `Random
game`, then the tool's own list-wide rows where a tool opened it (§15.5). The pane says what the focused row does and
what the list shows now, in Body (`Sorted by name, A to Z.`). Help bar `A Choose · B Back · ◂ ▸ Change · Start
Close`. Over a themed view the card's Filter and Sort set the theme's own gamelist sort and filter, so the theme's
`GamelistFilterWindow`, Jump To and Sort By sheets retire into it.

**Search docks the keyboard.** A on Search closes the card and docks the keyboard (`OnScreenKeyboard`'s docked form,
`EmuSen_Shell.md` §3, H2) at the foot of the screen: 0,398 to 702 (decided 2026-11-03, as the mockup draws it), five rows of 72 × 48 keys on SurfaceRaised, gap 8,
the focused key ringed, `Shift`, `Symbols`, `Space` and `Done` as wide keys. The search field takes the strip's row at
64,100 1152 × 44 on InputSurface with the typed text in TextStrong and a caret; the grid above narrows as the player
types, one row visible, and the strip's count reads `12 of 1,284 games`. Help bar `A Type · B Delete · Y Space · X
Done · L1/R1 Move to grid · Start Home`; the shoulders move the focus between the keyboard and the grid, so a result
is played without closing the keyboard. B on an empty field closes it. The same docked keyboard serves every text row
(Edit Details, the Shell, a tool's search).

### 18.2 The game's page, its deeper pages

Each is a page in the same frame, the breadcrumb in the section strip (`Options › Cheats`), B returning to the section.

- **Options ▸ Cheats ▸**: the game's cheats as rows `◂ On ▸`, then `Add a code…` (the docked keyboard); the same rows
  the Pause Card's Game Options ▸ Cheats shows and the Cheats tool's game page, one switch with three doors.
- **Options ▸ Package ▸**: `Mode ◂ Linked · CHD · 412 MB ▸`, then one row per layer `◂ On ▸` (patches, textures,
  scripts, audio, codes, in order), X moving the focused layer up or down; `EmuSen_Modding.md` §4's layer control.
  `GamePackageWindow` retires; Packages' own page keeps the files, identity and checks.
- **Details ▸ Edit Details ▸**: the metadata as rows (`Name`, `Console ◂ ▸`, `Year`, `Developer`, `Publisher`, `Genre`,
  `Players ◂ ▸`, `Description ▸`, `Hidden ◂ Off ▸`), A opening the docked keyboard on a text row, Y resetting the row
  to what was scraped; saved at once. `MetadataEditorWindow` retires.
- **Details ▸ Cover Art ▸**: the covers to choose from as 150 × 200 cards: `Scraped (in use)`, ScreenScraper's
  others, `Borrow from <game>` (the cover picker's borrowing), `Generated`; A uses the focused one. `CoverPickerWindow`
  retires.
- **Details ▸ Scrape This Game…**: the question form with the matches as rows (`Granite Choir (1994, Super Nintendo)`
  first and focused, then the others, then `Search by another name…` and `No`), the pane-less form's sentence saying
  what will be fetched. `FindByNameWindow` retires.
- **Media ▸ A**: the picture full screen on Void, a caption bar 80 tall at the foot on CaptionSurface (`Screenshot 2
  of 5 · 1 November 2026 · 512 × 448`), Left and Right stepping, `X Set as cover`, `Y Delete…`, `B Back`; a clip plays
  here with sound (§5.9). `ScreenshotWindow` retires.

Desktop: the same pages in the content area under the tabs (Edit Details as a form with `Save`; Cover Art as a row of
cards; a picture full screen in the window on double click).

### 18.3 Start Console, the launch screen, the screensaver, the Resume question

- **Home card ▸ Start Console ▸**: a page of the consoles that start empty (PlayStation, PlayStation 2, Sega CD; those
  not yet supported greyed with `Not supported yet`), the engine as the value; the pane says what A does and whether
  firmware was found (`✓ Firmware found.` in Success). A starts the console with no disc, the Pause Card's Change Disc
  puts a game in. The row is offered only where a console starts empty (§2.3).
- **The launch screen** is EmuSen's under every theme (§7): Void with the game's hue at 35% blurred, the cover 240 ×
  320 and, beside it, `STARTING` in Caption upper Muted, the name in Display, the console and engine in Body Muted, a
  progress bar 320 × 8 and its line (`Loading the state in Slot 3`); held at least Pan (320 ms) and until the core's
  first frame, then the frame. ES-DE's measured launch screen and popup (`EmuSen_Settings_Reference.md` §4.71) retire.
- **The screensaver** keeps Dim, Black and Slideshow; the slideshow shows the library's own pictures (screenshots,
  title screens, clips when §5.9 lands) letterboxed on Void with a caption bar 120 tall on CaptionSurface: the cover
  60 × 80, the name in Title, the console and `any button wakes` in Caption Muted. A on the caption opens that game's
  page. ES-DE's overlay (§4.75) retires with it.
- **The Resume question** at a game's start is the question form: `Continue where you left off?`, one sentence,
  `Continue` first and focused, `Start again`, the caption naming the setting that stops the asking (Settings ▸ EmuSen ▸
  Saves and states ▸ Resume ◂ Ask each time ▸). B goes back to the library. `ResumeWindow` retires.

### 18.4 Collections

A tool on the frame: header `COLLECTIONS · 3 collections · 14 games`; sections `All · Mine · Automatic`; rows with the
count as the value (the automatic ones, Favourites and Recently played, greyed and not editable); `New collection…`
last; the pane shows the collection's covers and its play time. A opens the collection as a section of the library
(the grid with the collection's name in the strip's place, B returning); X renames, adds the selected game or removes
(`…`); Y adds the game in hand (the one Home's card or the grid last focused). In the own look a collection is also
reached from the List card's Filter ▸ Collections; with a theme, the theme's collection views stay the theme's. Desktop:
COLLECTIONS in the sidebar as today, a collection a view of the grid with `Rename…` and `Remove…` in the header, a game
dragged onto it joins it (§17.3). `CollectionSettingsWindow` retires.

### 18.5 The Pause Card's remaining pages

All are pages of the panel with the game as the breadcrumb (§4.4); X takes a screenshot on each; B returns.

Decided 2026-11-02 by the tester, as the card is played on the handheld, because a card that redraws at every choice
looks hokey and should stay put:
- **Turning a page** keeps the same panel, the scrim, the paused frame and the help band. Only the rows, the header
  with its breadcrumb and the pane change. There is no close, no reopening and no opening effect; the effect plays
  when the card first comes on screen.
- **A screen opened from the card** (Graphics, Shaders, Controller Bindings and the rest) lays over the card as a
  sheet. The card stays mounted beneath, covered, and keeps its page, its row and the game's pause. Closing the sheet
  uncovers the card where it was, without opening it again. On the big screen, under a sheet that draws its own menu,
  the card is covered by the blurred screen the sheet stands on: the panel is drawn transparent and stays laid out. A
  first attempt left the card drawn sharp beside the sheet's panel, which `SheetLookTests` caught as 6 pixels changing
  outside the panel at 1920 × 1200.
- The game runs no frame throughout.
- This is the Pause Card's rule only. A tool or settings sheet opened from the Home card still closes the card first:
  Theme Settings, Collections and the Interface tab apply their choices to the themed view beneath them, which a Home
  card kept on screen would cover. The first build applied the rule to both cards, and 48 tests of the themed views
  failed on it (2026-11-02). §2.9's return to Home from such a tool is a separate step.
- Built in `ChoosePadMenuRow` (an entry that opens a screen leaves the card open beneath it), `ShowPadMenuPage` (no
  opening effect) and `UpdateMenuBackdrop` (a card uncovered does not open again).
- Proved by WiseMan `PadMenuTests.The_card_stays_on_screen_from_page_to_page_into_a_sheet_and_back`, which steps
  page → page → sheet → back → back.

- **Rewind, the reel.** The Rewind row's pane shows the three newest pictures (§2.8). A on the row opens the reel page:
  the pane holds the focused moment at 592 × 330 and, under it, a strip of six moments 88 × 66 (`Now · 1.5 s · 4 s · 8
  s · 12 s · 28 s`), the focused one ringed; the panel's rows `Go back to 4 s ago` (A), `Moment ◂ 4 s ago ▸` (Left and
  Right step the strip), `Hold ◂ to scrub` (greyed, a hint). A returns the game to that moment and closes the card; the
  moments after it are kept until the player plays on. `RewindReelWindow` retires.
- **Change Disc ▸** (disc consoles): the game's own discs first (`Basalt Run (Disc 1) · In the console`), then the
  console's other games in the library, then `Open the tray`; the pane shows the focused disc's cover and one sentence
  (`A opens the tray, swaps the disc and closes it, as the console sees it.`). `InsertDiscWindow` retires; on the desktop
  the same page with `Any file…` last, opening the file dialog.
- **Game Options ▸ Controllers**: `Player 1 ◂ Wireless pad ▸`, `Player 2 ◂ None ▸`, `Profile ◂ Default ▸`, `Bindings
  ▸` (the drawing page of §16.2 for this console), `Swap Players`; the pane says who plays on what.
- **Game Options ▸ Display**: `Picture shape ◂ ▸`, `CRT shader ◂ ▸`, `Crop overscan ◂ ▸`, `This game's crop ◂ Off ▸`
  (`GameCropWindow`'s exception, a page of four edges when On), `Full screen ◂ ▸`; the pane previews the shader on the
  paused frame itself.
- **Game Options ▸ Sound**: `Volume ◂ 80% ▸`, `Mute ◂ Off ▸`, `Latency ◂ Normal ▸`, `Interface sounds ◂ On · 60% ▸`; on
  the handheld the Volume row reads `the device's quick menu` and does nothing (§6).
- **Game Options ▸ Memory Cards** (disc consoles only): `Slot 1 ◂ Card 1 · 11 of 15 blocks ▸`, `Slot 2 ◂ Card 2 ▸`,
  `Mode ◂ Shared · Per game ▸`, `Cards ▸` (the cards as rows: new, rename, copy); Left and Right put another card in
  and the game sees a card swap. The same rows under Consoles ▸ the console ▸ Memory cards (§18.10). `MemoryCardsWindow`
  retires.

### 18.6 Notices

One notice at a time, at the top right inside the safe area (x to 1216, y 40): SurfaceRaised, radius 8, a 1-unit
Border, the shadow, padding 14 × 24, a status glyph in Success or Attention and the words in Body TextStrong
(`Screenshot saved`, `State saved to Slot 3`, `Loaded Slot 3`, `Core approved`); it enters over Enter and leaves after
2 s over Exit; a second notice replaces the first. Over running gameplay only Saved, Loaded and the screenshot appear
(§5.6); every other notice waits for a card or a screen. The desktop's notices are the status bar's line (§17.2), and
the same notice over the game's frame. `NoticeLayer` draws both.

### 18.7 The Scraper's and the Library Manager's sentences

The pane in the form of §15.4, replacing each tool's own lines (§16.1, "a stage of their own"):

- **Scraper**: the game's name in Title; `Super Nintendo · matched to ScreenScraper 12345 by its hash` in Caption
  Muted; the theme's kinds as three 120 × 90 slots, each the picture or a dashed `none`; one sentence on what is
  missing and what ScreenScraper has; the marked count and the time at the quota's pace. The plan question's rows are
  `Kinds ◂ The theme's ▸ · Videos ◂ Off ▸ · What is there ◂ Keep ▸`, then `No` and `Yes`.
- **Library Manager**: the file's name in Title; the console, size and folder in Caption Muted; the finding in one
  sentence with its glyph (`⚠ Its hash matches no known dump; a byte near the end differs from the verified one, as a
  bad copy or a trained ROM would.`); which other files are the same dump; what A and X do. Findings: `✓ verified`,
  `⚠ suspect dump`, `duplicate`, `unknown`, `not a game`, `hidden`.

### 18.8 Mods

A tool on the frame, as §16.2 asks, for the game in hand (opened from its page's Options ▸ Package ▸, from Tools on the
desktop, or from Home's Mods place once X1 lands): header `MODS · Granite Choir · 5 packs · 2 on`; sections `Granite
Choir · Installed · Workshop`; rows Pack · Kind · State, A switching `On`/`Off`, X the pack's page (`Order ◂ 1 ▸`,
`Files ▸`, `Conflicts`, `Check against the dump`, `Save as Patch…`, `Remove…`); the pane with the pack's preview
picture, its source and version, one sentence on what it does and whether it conflicts. Workshop lists the emusen-mods
repository's packs for this game with `Install…` as A. The desktop frame adds `Export Patched Game…` to the apply bar.
The browsing and touch-up views of `EmuSen_Modding.md` §4 are the Debugger's renderers, reached from a pack's page.

### 18.9 The Shell, and vstop

The Shell is not a player's tool and is not on Home's places or the Home card's Tools; it is reached from Settings ▸
EmuSen ▸ Shell ▸ on the big screen and Tools ▸ Shell… on the desktop. On the big screen it is a tool on the frame:
header `SHELL · DianaSh 1.4 · Rust runtime · home/Games`; sections `Shell · History · Man pages`; the terminal as the
content (InputSurface, monospace 20) with the docked keyboard beneath; help bar `A Type · B Delete · X Run · Y Space ·
L1/R1 History · Start Home`. The History section lists the session's commands as rows (A re-runs); Man pages lists the
commands as rows (A opens the page as scrolling Body). The desktop window keeps `HotaruWindow`'s toolbar (Run, Stop,
Clear, Man Pages ▾, Find in output). `vstop`'s dashboard becomes a view of the Shell window on the desktop (the command's
output drawn by its renderer), not a window of its own; it has no big-screen form.

### 18.10 Consoles ▸ the console ▸ Firmware and Memory cards

- **Firmware ▸**: one row per firmware the console's engines name, in the console's words (`PlayStation BIOS`), its
  value `✓ Found · SCPH-1001` in Success or `⚠ None found` in Attention; `Open replacement ◂ In use when none is found ·
  Always · Never ▸` where one exists; `Folder ▸` (the path picker); `Check again`. The pane: recognised by content, never
  downloaded, what runs without one. `FirmwarePane` retires. Never a file list of the folder.
- **Memory cards ▸**: `Mode ◂ Shared · Per game ▸`, `Slot 1 ◂ Card 1 ▸`, `Slot 2 ◂ Card 2 ▸`, `Cards ▸`, `New card…`; the
  pane says what Shared means (`every PlayStation game sees the same two cards, as the console did`). The in-game page
  of §18.5 is the same description.

### 18.11 A page of choices

A row whose values are more than Left and Right can step comfortably (above six, or long names: an engine, a region, a
language, a profile) opens a page under A: the breadcrumb (`Consoles › Super Nintendo › Engine`), the choices as rows
with their status words (`VenusRT (EmuSen) · ● Default`, `Snes9x (libretro) · ✓ Installed`, an unavailable one greyed),
the pane explaining the focused one; A chooses and returns, B keeps what was chosen. `◂ ▸` still steps the row in
place. `OptionListWindow` retires.

### 18.12 The Debugger's desktop form

`EmuSen_Debugger.md` §4 on the tools' desktop frame (§17.6): the header `DEBUGGER · Granite Choir · VenusRT · paused at
frame 4,812 · Explore` with the actions as buttons (`Step Frame` filled, `Step Instruction`, `Run`, `Breakpoint`, the
`Explore ▾` / `Debug` mode); the sidebar of views grouped (Video, Memory, Processors, Sound, Load, Frame); the filter row
(`Show: All sprites ▾`, `Find an address`); the view drawn by its renderer as the content, zoomable, the focused cell
ringed; the pane with the selected thing in full and its explanation; the Shell's pane docked beneath (`debug>`), as
§4 asks, with `bp`, `mem` and `step` typed beside what they change. No big-screen form (§16.2); the fit audit holds it
at 1280 × 800 and 1920 × 1200.

### 18.13 Desktop counterparts

Where §17 did not picture a window, the family's frame holds it with its own columns and actions, rows 24 at Body 13,
the row's one action first in the header (§17.6): Scraper (`Scrape…`, Show: No art · Cover only · Complete), Library
Manager (`Hide…`), Cheats (the game's cheats switched in the pane), Themes (tiles 160 × 154 with the options in the
pane), Mods (§18.8), Collections (§18.4), the Shell and the Debugger (§18.9, §18.12), Setup (the steps as tabs under
one header, `Back` and `Next`), Settings ▸ Consoles (one row per console with its engine and firmware line, `Open…`).
The pad opens the Home card over the library and the Pause Card over the game at 40 per cent of the content area, as
§17.8 draws F4's card.

### 18.14 Fits

Measured in the design's drawings on 2026-11-01 **[M]**: every screen's words whole at 1280 × 800 and 1920 × 1080 (16:9 gives the
list and the pane 142 more units, the grid a sixth column, the shelf a ninth tile); the handheld's 1920 × 1200 is the
1280 × 800 picture at scale 1.5 (§6). The longest cases held: the Cores pane with firmware and the approval line (as
§15.4); the Pause Card's title with a cover at Title 32; the Library Manager's finding column at 0.30 of the list; the
docked keyboard with one grid row above it; the question form's five rows (the Scraper's plan). Two rows were left out
of a picture rather than cut: the game's page's fifth Options row (Package) and Settings ▸ Display's ninth row, which
scroll (Reel) as §4 allows. The section strip draws only whole names (`WholeRow`), so `DREAMCAST` and `PLAYSTATION 2`
are reached by the shoulders at 1280 and drawn at 16:9.

### 18.15 Questions

1. **[decision]** Over EmuSen's own look Start goes to Home the screen, as §2.2 and §2.9 say, and no card is drawn over
   the grid. *Recommended:* keep the screen; one Home in the own look, one card over a theme.
2. **[decision]** Compressed Discs retires into Packages: Convert to CHD is the package's action, the discs a filter of
   the List card, the batch's outcome the apply bar's. *Recommended:* yes; one tool for what a game's files are.
3. **[decision]** The Shell leaves the Home card and Home's places, reached from Settings ▸ EmuSen ▸ Shell ▸ and the
   desktop's Tools menu; `vstop` a view of the Shell window. *Recommended:* yes; it is not a player's destination.
4. **[decision]** A on Rewind opens the reel page (§18.5) rather than rewinding at once. *Recommended:* yes; the pane
   previews, the page chooses, and a held Left scrubs for the player who wants it at once.
5. **[decision]** The themed folder's one edit (its link) moves to the List card ▸ Folder ▸, and the five sheets of the
   game's page (§18.2) replace the metadata editor, the cover picker, the scrape chooser, the game package window and
   the screenshot window. *Recommended:* yes.
6. **[decision]** The launch screen and the slideshow are EmuSen's under every theme (§7), replacing ES-DE's measured
   ones. *Recommended:* yes; the theme keeps its two views and nothing else.
7. **[decision]** Memory cards get two doors, in the game's Game Options and under the console's settings, from one
   description. *Recommended:* yes, as Engine has (§16.2).
8. **[routine]** The Resume question uses the question form with `Continue` first, not `No`/`Yes`. *Recommended:* yes.
9. **[routine]** Collections as a tool and as a section of the library; the automatic ones listed and greyed.
   *Recommended:* yes.
10. **[routine]** The notice as §18.6, one at a time, 2 s. *Recommended:* yes.
11. **[routine]** Setup's five steps: ROM folder, Firmware, Controllers, Theme, ScreenScraper (`EmuSen_LibraryTools.md`
    §1b). *Recommended:* yes.

**Decided 2026-11-01 by the tester:** §18 as written, the seven decisions and the four routine items as recommended;
recorded in §0 as 9 to 19.

---

## 19. U7 as built (2026-11-03)

The tools' header rows of §4 screen 5 and §15.2, and the tools' live lines on Home's places, on the Home card's Tools
page and on the desktop's Home (§10, U7). The tools' bodies are as they were: the Scraper's screen, plan and running
states and the other tools' pages are the window follow-ons.

### 19.1 One summary per tool

Each tool says its one-line state in one place, its own library, through the surface it already has; the host reads it
and draws it, and never words it. The record is Galaxia's `ToolSummary` (beside `ToolWords`, the tools' shared words):
the line, a shorter one for a 150-unit place, and whether it needs the player, whose sign `⚠` the place or header adds.

| Tool | Owner | The line (desktop) / the short one (big screen, Home card) | Before anything is said |
|---|---|---|---|
| Games | Mistress's library scan | `1,284 games` | — |
| Scraper | `ScraperPlan.StateOf` over `scrape ls`'s rows | `41 without art` / `41 no art`; `Every game has art` | `Art and facts` |
| Packages | `PackagesWords.StateOf` over `pkg-ls`'s rows | `3 changed` (a package moved, missing, changed or unreadable), else `5 packaged`, else `None packaged yet` | `Game packages` |
| Library Manager | `LibraryFindings.StateOf` after a whole check | `⚠ 2 suspect dumps` / `⚠ 2 suspect`, else `1 duplicate`, else `No problems found` | `Checks dumps` |
| Cores | `CoresWords.StateOf` over the runtime's `cores` and `core-ls` (offline) | `⚠ 1 to approve`, else `↑ 2 updates`, else `17 cores · 5 installed` (§15.8) | `The engines` |
| Cheats | Galaxia's `GameCheats.Summary` over every saved list, the consoles' and the packages' | `12 on · 3 games`, else `4 games with cheats` / `none on` | `Codes for games` |

Each pane raises `SummaryChanged` with the same summary when its listing ends (the Library Manager's when a whole check
ends), and builds its header's facts from the same words, so the header and Home cannot disagree. Accepted 2026-11-03
as **[routine]**: Packages' `changed` counts the packages in a problem state, as the mockup words it.

### 19.2 The header row

LunaP's `AdaptiveHeaderRow` (its §215), which the frame now draws its header with, so Cores' row is reused and not
copied: the name at the left; on the big screen the facts right-aligned, on the desktop after the name; a fact that
begins with `⚠` in Attention and the rest Muted; on the desktop the focused row's one action at the right, filled Accent
(`ManagerLayout.ActionFor`, the action A does on the big screen). The facts, as built:

- Packages: `3 changed · 263 KB in packages · 410 GB free`.
- Scraper: `1,284 games · 41 without art`.
- Library Manager: `1,284 files · ⚠ 2 suspect dumps · 1 duplicate · checked today` (`checking`, `not checked`). The
  count of files had been the length of the library folder's path, a defect found here and fixed.
- Cores: as §15.10.
- Cheats: the database window, which is not yet on the frame (§16.2), carries the row on the desktop (`CHEATS`,
  `12 on · 3 games · 7 cheat files in the database`). On the big screen its centred sheet keeps its own title. Measured
  on 2026-11-03: the facts as the sheet's subtitle took 40 units and left the list of games 75 high, under the fit
  audit's floor, so the big screen's row waits for the Cheats rebuild.

The desktop toolbars are unchanged; trimming them to the mockups' few buttons and a More menu is the follow-on.

### 19.3 When the lines are read

Measured on 2026-11-03 on the desktop (Ryzen 7 7700X), each read twice, with its managed allocation, by
`WindowFitAuditTests.Each_tools_summary_read_is_timed` (run under `EMUSEN_U7_COST=1`; `EMUSEN_U7_LIBRARY` names a real
library, read only). **[M]**

| Read | The library of 9,328 files | 1,000 synthetic games |
|---|---|---|
| Scraper, `scrape ls` | 2,124 and 2,140 ms, 19.5 MB | 132 and 131 ms, 2.1 MB |
| Packages, `pkg-ls` | 208 and 206 ms, 8.7 MB | 7 and 18 ms, 0.8 MB |
| Cores, `cores` and `core-ls` | 0.4 and 0.3 ms, 43 KB | 0.3 and 0.3 ms, 43 KB |
| Cheats, every saved list | 0.2 and 0.1 ms, 141 KB (2 lists) | 2.4 and 4.6 ms, 5.1 MB (300 lists) |

Decided 2026-11-03 on these numbers:

1. The Scraper's and Packages' reads, which cost seconds on a large library, are made once a session at the first Home,
   and again only when their data changes: their own window's listing (which reports its summary), a scrape run's end.
2. The Cores' and Cheats' reads are made at most once a minute; a saved cheat list asks again at the next Home.
3. Every read runs on a thread of the lowest priority, off the interface thread, only while Home, a card over it or the
   desktop's library shows, never while a game is on screen; a game starting cancels a read still running, and its
   answer is dropped.
4. Until a read answers, the place shows its description, never a stale or zero count. A read that cannot say (a
   command not in use, a refusal) keeps the last line heard.
5. The Library Manager has no read of its own, since its check reads every file: its line is its description until a
   whole check runs in the session.

### 19.4 Departures, measured against the mockups

- The desktop's places draw a `⚠` line in Attention where the desktop mockup draws it Muted, so every `⚠` line on
  Home is drawn alike; the Home card's rows keep the row's own colour, as its mockup draws them.
- The Home card's Tools page gains `Cheats` with its line, as its mockup lists it; `Compressed Discs` stays until
  decision 10 of §0 retires it into Packages.
- The big screen's Cheats has no header row (§19.2).

### 19.5 Tests, measured on 2026-11-03

`ToolSummaryTests` 4 (each tool's words, the cheats' count, a Packages listing through a real job saying the line its
read without a window says, the host's schedule and keeping), `OwnLookScreenTests`, `HomeCardTests`, the tool windows'
classes (`PackagesWindowTests`, `ScraperWindowTests`, `LibraryWindowTests`, `CoresWindowTests`,
`CoresWindowRuntimeTests`), and the audits and recordings of `EmuSen_InterfaceSpec.md` §6.3.25. LunaP's whole suite,
1,990, with `HeaderRowTests` 4. Seeded faults, each caught: a window's listing not kept (a stale line;
`ToolSummaryTests`, `HomeCardTests`); Packages counting packaged games as changed (`ToolSummaryTests`); the Library
Manager's warning dropped (`ToolSummaryTests`); in LunaP the warning fact drawn Muted, the header action left at the
previous row's, and the desktop's facts at the right edge (`HeaderRowTests`; the last after the test measured the name
by its text rather than its stretched box).

Comparison pictures, mockup left and built right, at 1280 × 800 and 1920 × 1080 and the desktop's 1280 × 800:
`~/.cache/emusen/tmp/u7/compare/`.
