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
build order, §11 for stage U1 as built, §12 for the sources. A bracketed number cites a source in §12.

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
Barlow project (`THIRD_PARTY_NOTICES.md` §1.5).

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
  the stock controls of a framed sheet are in Barlow, its headings and tabs in Barlow Condensed; the desktop is in Barlow
  at 13; `NavigationSounds` renders LunaP's cues under ES-DE's names, the tone code moved unchanged so the sounds are the
  same; the navigation volume defaults to 60.
- **Not yet in U1.** The drawn menus' rows stay in Barlow Condensed at their present sizes until the Card form rebuilds
  them (U2, U3); the player's switches for light or dark, reduced motion and Play videos arrive with Settings (U6); the
  tile ring is a token until the first tile grid takes it (U5).
- **Recordings.** Every recorded state changed, by the causes listed in `EmuSen_InterfaceSpec.md` §6.3.17.

---

## 12. Sources (opened 2026-11-01)

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
15. Libretro docs, "The User Interface", https://docs.libretro.com/guides/gui/
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
