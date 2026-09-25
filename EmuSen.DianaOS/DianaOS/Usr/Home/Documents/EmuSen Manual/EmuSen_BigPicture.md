# EmuSen_BigPicture — a plan for a big-picture mode in Mistress that renders ES-DE themes

*Written 2026-09-24. Nothing is built.* The user asked for a big-picture mode in Mistress like EmulationStation's,
starting with the theme they like, Art Book Next. They made two choices. First, Mistress reads ES-DE themes, so that
Art Book Next and other ES-DE themes load as their authors made them. A look-alike built from Mistress's own controls
was not wanted. Second, game media comes from ScreenScraper. This page plans that work. It inventories the theme
format and what Art Book Next uses of it, designs the renderer, the ScreenScraper client and theme management, and
stages the work, each stage with its oracle.

The page keeps three things apart:

- **Measured** means counted or run on 2026-09-24 on the development desktop (Ryzen 7 7700X, Fedora 44).
- **Cited** means read from a named source (§11). The source is quoted by section, not reproduced.
- **Argued** means reasoning with no measurement behind it.

Predictions are numbered P1–P12 so that they can be retired later (§9).

**The copyright rule this plan works under.** Art Book Next is licensed CC BY-NC-SA 2.0 and carries third-party
credits. EmuSen is GPL-3.0. Nothing from the theme enters the EmuSen repository or its releases: no file, no XML, no
artwork, no rendered picture of it, and no golden derived from it. The theme is downloaded when the player asks for it
(§6), in the same way the RetroArch shader pack is (`EmuSen_Settings_Reference.md` §4.41). ES-DE's own code is
MIT-licensed (§2.1), which would permit reuse with a notice. By the user's decision the plan copies none of it anyway,
and implements only the format that ES-DE's `THEMES.md` documents. For reading, both theme repositories were cloned
outside the EmuSen tree: `~/Projects/art-book-next-reference` (Batocera edition) and
`~/Projects/art-book-next-es-de-reference` (ES-DE edition). Nothing was copied from either.

---

## 0. Summary

- **The repository the user linked is not an ES-DE theme.** `anthonycaccese/art-book-next-es` is the edition for the
  Batocera fork of EmulationStation, and uses that fork's format (§1.1). ES-DE cannot load it: ES-DE 2.2.0 and later
  refuse every theme in the legacy format. The same author publishes an ES-DE edition,
  `anthonycaccese/art-book-next-es-de`, and the plan targets that edition (Q1).
- **What the ES-DE edition uses (measured, §3).**
  - Thirteen element types: `carousel`, `textlist`, `grid`, `image`, `video`, `text`, `datetime`, `rating`, `badges`,
    `helpsystem`, `clock`, `systemstatus` and `sound`.
  - Configuration: 20 variants with 12 variant-trigger overrides, 31 colour schemes, 4 font sizes, 12 aspect ratios
    and 2 transition profiles.
  - Media: variables drive nearly every colour and path. There are 241 SVG files and 905 PNG files (219 MB), three
    TTF fonts and seven WAV sounds.
  - Video: 12 of the 20 variants play a game video, the default variant among them.
  - Not used: animations (GIF or Lottie), `gameselector`, `gamelistinfo`, languages, wheel carousels and reflections.
- **Recommended architecture (argued, §4).** A theme engine in a folder of Mistress, not in a new assembly and not in
  LunaP. It has three layers:
  - a loader that turns the theme's XML into a resolved element model and knows nothing of Avalonia or of any core;
  - a scene that holds each element's state over time;
  - a single Skia-drawn control that paints the scene, using the same draw path as Serenity's `GameFrameControl`.

  LunaP's sheets, pad menu and keyboard stay the Avalonia layer over it. SVG is drawn by a small renderer for the
  subset of SVG that themes use, written to the subset table in LunaP's `PLAN-icons.md` §5. `Svg.Skia` was
  considered and not recommended, for its licence and its SkiaSharp line (§4.5). Video is deferred. Without video
  files, the render still matches ES-DE's for the same media, because ES-DE then shows the static image in the video
  element's place (§4.7).
- **ScreenScraper (cited, §5).** API v2 `jeuInfos.php` is asked by MD5, CRC32 and SHA-1, file size, file name and
  system ID (NES 3, SNES 4, Game Boy 9, Game Boy Color 10, N64 14). Every call carries a **developer** ID and password
  issued to the software through the ScreenScraper forum, and optionally the player's own account. The user must
  request the developer credentials (§5.7). Those credentials live in a file outside the repository, and neither they
  nor the player's password is ever committed or logged.
- **Stages (§7).** Seven stages, (a) to (g), about 17–27 working days in all, plus waiting time for the developer
  credentials. Each stage has an oracle, and the rendering stages use ES-DE itself as that oracle. ES-DE is run
  locally on inputs Mistress also receives. Its captures are kept outside the repository.

---

## 1. What was asked, and what the brief had to be corrected on

### 1.1 Two editions of Art Book Next, and which one ES-DE can load (measured and cited)

The link in the brief is `github.com/anthonycaccese/art-book-next-es` (cloned at `9a50ef3`, 2026-03-11). Its README
says it targets "this fork of EmulationStation", meaning Batocera's, and names Batocera v40+, Knulli, RockNIX and
RetroBat. Its XML shows the Batocera format:

- `<formatVersion>7`;
- a `<subset>` tag holding `<include if="…">` expressions such as `{screen.ratio} == '16/10'`;
- `ifSubset` and `ifHelpPrompts` attributes on elements;
- `extra="true"`;
- a `stackpanel` element, and a `splash` view.

Python's XML parser rejects its `theme.xml` at line 49, because of the unescaped `&&` inside an `if` expression, so
the file is not well-formed XML at all.

`THEMES.md` settles whether ES-DE can read it:

- "ES-DE as of 2.2.0 can no longer load legacy themes".
- "Attempting to use any of the legacy logic in the new theme structure will make the theme loading fail, for
  example adding the *extra="true"* attribute to any element" (§"Differences to legacy RetroPie themes").

The same author publishes `github.com/anthonycaccese/art-book-next-es-de`, cloned at `d772d07`, 2026-02-06. It is
the entry named "Art Book Next" in ES-DE's official theme list (`themes-list/themes.json`, whose `latestStableRelease`
is 51). **The option set the brief listed belongs to the Batocera edition:**

| Option | Batocera edition (the link) | ES-DE edition |
|---|---|---|
| Aspect ratios | 16:9, 16:10, 4:3, 3:2, 1:1 | 32:9, 21:9, 20:9, 19.5:9, 16:9, 16:10, 3:2, 4:3, 5:4, 8:7, 1:1, 5:3 vertical |
| Colour schemes | Default, Light, Steam OS, SNES, Famicom, OLED, Custom | Six palettes (Dark, OLED, Light, Steam OS, SNES, Famicom), each with five artwork sets (Screenshots, Noir, Circuit, Outline, Original Screenshots), plus Custom: 31 |
| Layout | "Game Artwork" and "Game Metadata" menu options | 20 variants: 7 lists, 3 grids, and each again with the help bar off |
| Font size | Default, Small, Large, Extra Large | small, medium, large, x-large |
| Fonts | Roboto, ChangaOne | Mulish, ChangaOne |
| Metadata icons credited to | FontAwesome | Phosphor Icons |

The ES-DE edition's credits add the Outline artwork set (Joppa Fallston) and artwork by theUnBurn to those the brief
listed. Its licence line is the same: CC BY-NC-SA 2.0.

Q1 (§10) asks the user to confirm the ES-DE edition. The recommendation is yes. It is the same author's same design,
it is in the format the user chose, and ES-DE's own documentation describes that format.

### 1.2 The handheld's resolution (cited, and in conflict)

The brief says the Legion Go S runs at 1280×800. `EmuSen_Settings_Reference.md` §4.45.2 describes "a Legion Go S's
8-inch 1920 by 1200 panel". Both are 16:10, so the aspect-ratio choice is the same either way. The performance budget
is not: 1920×1200 has 2.25 times the pixels. §4.8 prices both, and Q2 asks which the player actually runs, because
gamescope can render at 1280×800 and scale up to the panel.

---

## 2. The ES-DE theme format (cited)

### 2.1 The source and its licence

The source is `THEMES.md` in `gitlab.com/es-de/emulationstation-de`, master branch, 3,812 lines, read in full on
2026-09-24. ES-DE's `LICENSE` is MIT: copyright Northwestern Software AB 2024–2026, Leon Styhre 2020–2024 and Alec
Lofquist 2014. The file header of `ScreenScraper.cpp` carries `SPDX-License-Identifier: MIT`. The format is therefore
documented by an MIT project. The plan implements the document, and consults ES-DE's source only for facts that
belong to other parties, such as ScreenScraper's system IDs and media type names (§5).

### 2.2 The grammar

- **Files.**
  - A theme is a directory. `capabilities.xml` at its root is mandatory; without it, "ES-DE will not attempt to load
    the theme".
  - For each system, `<theme>/<system.theme>/theme.xml` is used when it exists, else `<theme>/theme.xml`.
  - Every XML file's root is `<theme>`. `capabilities.xml`'s root is `<themeCapabilities>`.
- **Views.** There are two views, `system` and `gamelist`. A third, `all`, is used only for navigation sounds. A
  `name` attribute may list several views, separated by commas or whitespace.
- **Elements.** An element is written `<type name="…">`. Definitions that share a type and a name, in the same view,
  merge property by property, and the last definition wins. A `name` attribute may list several elements at once.
- **Tags that select configuration:**

  | Tag | Where it may sit |
  |---|---|
  | `<variant name="…">` | `<theme>` only; `all` is reserved |
  | `<colorScheme name="…">` | `<theme>`, `<variant>`, `<aspectRatio>`; holds variables only |
  | `<fontSize name="…">` | variables only |
  | `<language name="…">` | variables only |
  | `<aspectRatio name="…">` | `<theme>`, `<variant>`; ratios come from a fixed table of 12 horizontal names and their vertical forms |

- **Variables.**
  - System variables: `system.name`, `system.fullName` and `system.theme`, each also in `.autoCollections`,
    `.customCollections` and `.noCollections` forms.
  - Variables defined by the theme may nest.
  - All variables share one global namespace. Redefining a variable inside `<variant>` or `<aspectRatio>` modifies
    the global one, and the result depends on when the redefinition is parsed ("Theme variables").
- **Parsing order**, applied recursively to each included file at the point of its `<include>`:
  1. transitions;
  2. variables;
  3. colour schemes;
  4. font sizes;
  5. languages;
  6. included files;
  7. general (non-variant) configuration;
  8. variants;
  9. aspect ratios.

  Within a step, the file's order is kept. Variant configuration is parsed after general configuration even when it
  is written above it ("Configuration parsing order").
- **Includes.**
  - `<include>` may sit in `<theme>`, `<variant>` and `<aspectRatio>`, not in `<view>`.
  - Paths beginning `./` are relative to the including file, and `~` is the home directory.
  - A missing include is an error, and unthemes the system, when the path is written out explicitly. It is only a
    debug message when the path was built from a variable.
  - Include loops are not detected.
- **Property types.** `NORMALIZED_PAIR`, `PATH`, `BOOLEAN`, `COLOR` (6 or 8 hex digits, RGBA), `UNSIGNED_INTEGER`,
  `FLOAT` and `STRING`.
- **Coordinates.**
  - `pos` and `size` are fractions of the parent, which is nearly always the screen. `0 0` is the top left and y
    points down. Values outside 0–1 are allowed.
  - `origin` is the point of the element that `pos` names.
  - `fontSize` is a fraction of the screen height on a horizontal screen, "based on the reference 'S' character".
  - Every `cornerRadius` is a fraction of the screen **width**.
- **Error policy.**
  - Structural errors untheme the system they occur in.
  - Invalid values are reset to their defaults with a warning.
  - An invalid `imageType` or similar value stops that element from rendering.
- **zIndex defaults.** image and video 30; animation and badges 35; text and datetime 40; gamelistinfo and rating 45;
  carousel, grid and textlist 50. `helpsystem`, `clock` and `systemstatus` have no zIndex and are drawn on top of
  everything else.
- **Variant triggers.** `noVideos` and `noMedia` (the latter with an optional `mediaType` list) replace the chosen
  variant with another, for the gamelist view only. They are one step deep: an override is not itself overridden.
- **Transitions.** Profiles declared in `capabilities.xml` give instant, slide or fade for six kinds of move. The
  first profile applies when the player chooses "Automatic".

### 2.3 The elements

| Group | Element | Purpose |
|---|---|---|
| Primary (one per view per variant; takes input) | `carousel` | horizontal, vertical or wheel list of images or text |
| | `grid` | X by Y tiles |
| | `textlist` | a scrolling list of names |
| Secondary (unlimited) | `image` | raster or SVG, static or chosen by `imageType` |
| | `video` | a video with a static image before and after it |
| | `animation` | GIF or Lottie |
| | `badges` | icons for favourite, completed and so on |
| | `text` | a literal, `systemdata` or `metadata`, optionally in a scrolling container |
| | `datetime` | a release date or last played, absolute or relative |
| | `gamelistinfo` | game count and filter |
| | `rating` | 0–5 stars from filled and unfilled images |
| Special | `gameselector` | picks games for the system view |
| | `helpsystem` | the button hints |
| | `systemstatus` | Bluetooth, Wi-Fi, cellular and battery |
| | `clock` | the time |
| | `sound` | navigation sounds (in the `all` view) |

---

## 3. What Art Book Next (ES-DE edition) uses (measured)

The inventory below was produced by walking every XML file of the ES-DE clone with a throwaway script
(`inventory.py`, in the session's scratch space, not committed). The script counts element types and the properties
set on them. The 213 per-system files in `_inc/systems/_metadata-global/` are counted separately: they define only
variables.

### 3.1 The repository

| Kind | Files | Size |
|---|---|---|
| PNG | 905 | 218.7 MB |
| XML | 251 | 4.8 MB |
| SVG | 241 | 2.2 MB |
| WAV | 7 | 0.5 MB |
| TTF | 3 | 0.2 MB |
| Everything, excluding `.git` | 1,414 | 226.7 MB |

The PNGs are the system-view artwork, five sets of about 131–213 files each.

- **Dimensions.** Almost every artwork image is 452–454 by 1,080 pixels, RGBA: a tall angled slice. Six images are
  301 by 720, and five are 134 by 320.
- **Coverage of EmuSen's systems.** Every system and collection EmuSen would show (`nes`, `snes`, `n64`, `gb`, `gbc`,
  `auto-allgames`, `auto-favorites`, `auto-lastplayed`, `custom-collections`) has an SVG logo and an image in all five
  artwork sets. `_default` has artwork but no logo.

### 3.2 `capabilities.xml`

- `themeName` "Art Book Next".
- **No `<language>` is declared.** The metadata files nevertheless carry `<language>` blocks, which §2.2's rule
  suggests are not applied. What ES-DE does with them is to be checked in stage (b).
- 12 aspect ratios, 4 font sizes, and 31 colour schemes (§1.1).
- 2 transition profiles, `instant` (the first, so the Automatic choice) and `slide`, which slides between the system
  and gamelist views. All three built-in profiles are suppressed.
- 20 variants. 12 of them carry a `noMedia` override that falls back to `gamelist-list-basic` or its `-nh` twin. The
  mediaType named is `cover`, `screenshot` or `miximage`, and twice `screenshot,marquee`.
- **The first variant declared is `gamelist-list-metadata-cover`** ("List: Metadata & Boxart"). THEMES.md does not
  say which variant is the default when none has been chosen; stage (a) records what ES-DE picks.

### 3.3 Elements, and the properties set on them

Counts are definitions: one element is often defined in several places and merged.

| Element | Defs | Properties set |
|---|---|---|
| `carousel` | 13 | `pos size origin color textColor itemTransitions itemVerticalAlignment fastScrolling defaultImage staticImage imageColor imageSelectedColor imageSaturation itemScale itemSize maxItemCount zIndex` |
| `textlist` | 86 | `pos size origin horizontalAlignment fontPath fontSize lineSpacing systemNameSuffix selectorColor selectedColor primaryColor secondaryColor selectedBackgroundColor selectedBackgroundCornerRadius selectedBackgroundMargins zIndex` |
| `grid` | 157 | `pos size origin itemSize itemScale scaleInwards itemSpacing unfocusedItemOpacity unfocusedItemDimming unfocusedItemSaturation fractionalRows textColor textBackgroundColor backgroundColor fontPath fontSize textRelativeScale imageRelativeScale imageCornerRadius backgroundCornerRadius textBackgroundCornerRadius imageType imageFit zIndex` |
| `image` | 170 | `pos size maxSize origin path default imageType tile color cornerRadius metadataElement visible zIndex` |
| `video` | 71 | `pos maxSize origin cropSize imageMaxSize imageType imageCornerRadius delay iterationCount onIterationsDone interpolation pillarboxes opacity zIndex` |
| `text` | 84 | `pos size origin text metadata defaultValue fontPath fontSize color horizontalAlignment verticalAlignment letterCase lineSpacing container containerType containerStartDelay containerVerticalSnap systemNameSuffix metadataElement visible` |
| `datetime` | 22 | `pos origin metadata format displayRelative defaultValue fontPath fontSize color lineSpacing visible` |
| `rating` | 13 | `pos size origin filledPath unfilledPath overlay color visible` |
| `badges` | 20 | `pos size origin horizontalAlignment direction lines itemsPerLine itemMargin slots customBadgeIcon controllerSize folderLinkSize` |
| `helpsystem` | 145 | `pos origin scope entries fontSize iconColor textColor backgroundColor backgroundCornerRadius backgroundHorizontalPadding backgroundVerticalPadding entryRelativeScale entrySpacing iconTextSpacing` |
| `clock` | 14 | `pos origin scope fontPath fontSize color backgroundColor backgroundCornerRadius backgroundHorizontalPadding backgroundVerticalPadding` |
| `systemstatus` | 13 | `pos origin height entries fontPath textRelativeScale color backgroundColor backgroundCornerRadius backgroundHorizontalPadding backgroundVerticalPadding customIcon` |
| `sound` | 7 | `path` (`systembrowse quicksysselect select back scroll favorite launch`) |

**Not used anywhere in the theme:**

- the elements `animation`, `gameselector`, `gamelistinfo`;
- carousel types other than the default `horizontal`, and reflections;
- rotation, `stationary`, gradients (`colorEnd`), `brightness`, mipmaps;
- `scrollFadeIn`, `gameOverridePath`, `systemdata`;
- `noVideos` triggers, and languages.

The two views are built as follows.

- **System view.**
  - A full-screen horizontal carousel of `staticImage` artwork, one per system. `maxItemCount` is 4.95 at 16:10,
    `itemSize` is 1×1 and `itemScale` is 1, so each slice is laid out in a full-screen box, about a fifth of the
    width apart, and the angled slices tile across the screen.
  - The unfocused items keep the documented default `unfocusedItemOpacity` of 0.5.
  - The colour multiply and saturation come from the scheme.
  - The system's SVG logo is drawn centred above the carousel.
  - A clock, the system status and the help bar sit on top.
- **Gamelist view, list variants.**
  - A tinted tiled background (a 16×16 `space.png`, tiled and multiplied by a colour).
  - The system logo, and a textlist on the left.
  - A `video` element named `game-art` that shows the cover, screenshot or miximage and, after 3 s, plays the game's
    video once (`iterationCount` 1, `onIterationsDone` image).
  - In the metadata variants: a description in a vertically auto-scrolling container (start delay 6 s), rating stars
    drawn from SVG, and release date, players and play time, each beside an SVG icon.
  - Badges in one row of five slots: folder, favourite, completed, collection and alternative emulator.
- **Gamelist view, grid variants.**
  - A grid of covers or screenshots, whose `itemSize` comes from a per-system include chosen by the variable
    `${systemCoverSize}`.
  - A menu bar and a game-name text. The video is commented out.

### 3.4 Variables and includes

- **Includes.** 23 in all. `theme.xml` includes `_metadata-global/_default.xml` and then
  `_metadata-global/${system.theme}.xml`, which is a variable path and so may be missing. It then includes
  `colors.xml` and one aspect-ratio file per `<aspectRatio>`, and the grid variants include
  `_coversize/${systemCoverSize}.xml`.
- **`colors.xml` ends with `<include>${customizationPath}</include>`.** That variable is defined only by the `custom`
  scheme. In every other scheme, therefore, an include names a variable that is not defined. The theme loads in ES-DE,
  so the loader must treat an unresolved variable in an include as the "missing, via variable" case, not as an error.
  Stage (a) holds this with a synthetic test and confirms it against ES-DE.
- **Variables.** 43 are referenced. `${gamelistMetadataFontSize}` is referenced most (42 times), then
  `${helpSystemViewEntries}` (36). Of the per-system metadata variables, only `systemCoverSize` is read by the theme
  itself. `systemDescription`, `systemColor` and the rest are defined and never used. `${system.fullName}` appears 18
  times, all inside those unused definitions.

### 3.5 Fonts, SVG, sounds

- **Fonts:** `Mulish-Light.ttf`, `Mulish-Medium.ttf` and `ChangaOne-Italic.ttf`, loaded by path from the theme.
- **SVG.** Across all 241 files, the elements are:
  - `path` (2,064), `rect` (323), `polygon` (294), `g` (217), `stop` (170), `circle` (118), `defs` (77);
  - `linearGradient` (49, in 10 files) and `style` with CSS classes (27 files);
  - `clipPath` (4 files), `feColorMatrix` and other filters (1 file), `text` (3 files).

  **The 36 SVGs EmuSen's systems and the theme's icons need use only `path`, `polygon`, `circle` and `g`, with `fill`,
  `fill-opacity`, `fill-rule`, `clip-rule`, stroke joins and `transform`.** `THEMES.md` notes that ES-DE draws SVG with
  LunaSVG, which renders no text and no embedded bitmaps (intro, and "Differences to legacy RetroPie themes").
- **Sounds:** seven WAV files in the `all` view.

### 3.6 What the engine supplies itself (cited)

A theme does not carry everything that ES-DE draws. For Art Book Next, ES-DE itself supplies:

- **the help system's button icons**, since the theme sets no `customButtonIcon` in the ES-DE edition, and the labels
  beside them;
- **the textlist's favourite and folder indicators.** The `indicators` default is `symbols`, "Font Awesome graphics";
- **the badges' folder-link overlay;**
- **every text a metadata field turns into:** "unknown", relative dates such as "x days ago", and play time in "the
  same logic as in Steam";
- **the battery level and charging state**, and the Wi-Fi and Bluetooth state, for `systemstatus`;
- **fallback navigation sounds**, although this theme supplies all seven.

Mistress must draw each of these from its own resources. None of them may be taken from ES-DE's repository (Q9).

### 3.7 The media the variants ask for

| Variant family | Media it shows | ScreenScraper media (§5.4) |
|---|---|---|
| List: Metadata & Boxart (the first variant) | cover, video | `box-2D`, `video-normalized` |
| List: Metadata & Screenshot + Marquee | screenshot, marquee, video | `ss`, `wheel-hd` or `wheel`, video |
| List: Metadata & Screenshot + Boxart | screenshot, cover, video | `ss`, `box-2D`, video |
| List: Metadata & Miximage | miximage, video | ES-DE builds miximages itself; ScreenScraper's nearest is `mixrbv2` (Q7) |
| List: Boxart / Screenshot + Marquee | as named, video | as above |
| Grid: Boxart / Screenshot | cover then screenshot, or screenshot | `box-2D`, `ss` |
| All metadata variants | description, rating, release date, players, genre, developer, publisher, last played, play time | from `jeuInfos`, except the last two, which are Mistress's own (`games.db`, §4.32 of the settings reference) |

The Batocera README's advice (image = screenshot, box = Box 2D, logo = wheel) is the same three media under
Batocera's scraper names.

### 3.8 What a first version must support, against the full format

| Feature | First version (faithful Art Book Next) | Later | Refused, or never |
|---|---|---|---|
| Loader: every tag, name list and variable rule of §2.2, `capabilities.xml`, the parsing order, the include rules, variant triggers | **yes**, the whole grammar (it is small, and a partial grammar fails whole themes) | | |
| `carousel`, horizontal only | **yes** | vertical and the two wheels, `reflections`, `itemAxisRotation` | |
| `textlist`, including horizontal scroll of the selected name | **yes** | | |
| `grid` | yes, in stage (f) | | |
| `image` (path, `imageType`, default, `tile`, `maxSize`, `cropSize`, colour multiply, saturation, `cornerRadius`, `metadataElement`) | **yes** | rotation, flips, gradients, `gameOverridePath`, `mipmap` | |
| `text`, including the vertical and horizontal containers, `metadata`, `letterCase`, `defaultValue` | **yes** | `systemdata`, rotation | |
| `datetime`, `rating`, `badges` | **yes** | the `controller` badge and its 36 controller icons | |
| `helpsystem`, `clock`, `systemstatus` | **yes** | | |
| `video` shown as its static image | **yes** | playback: stage (g) | |
| `sound` | stage (e) | | |
| `animation` (GIF, Lottie), `gameselector`, `gamelistinfo` | | when a theme the user wants uses them | |
| Transitions | instant | slide, fade, `stationary` | |
| Languages | en_US values | the others | |
| SVG `text`, `filter` | | | refused visibly, as LunaSVG does for `text` |

---

## 4. The rendering design in Avalonia and LunaP (argued)

### 4.1 Where the engine lives

Three homes were considered.

- **LunaP.** Refused. LunaP is a separate MIT repository, published to NuGet and consumed by Pegasus. Its rule is
  that a control earns its place when a consumer asks for it (`LunaP.md` §21). An ES-DE theme engine is a whole
  frontend format with game metadata in its vocabulary, and would push SVG and font loading onto every LunaP
  consumer. `PLAN-icons.md` §1.1 states the narrower rule for a sibling package: "a new package is justified when it
  needs a dependency LunaP may not take". That rule is about icons, not about a theme format.
- **A new assembly.** Refused for now. `EmuSen_Multicore.md` §9.1–§9.2 records the rule that pruning taught: a
  project boundary must buy separability that some consumer actually uses, and a shared abstraction "is cheap to add
  once two callers are visibly doing the same work, and expensive to carry when its second caller is hypothetical".
  Mistress is the only consumer. The roadmapped core-less launcher (`EmuSen_Launcher_Multicore_Gameplan.md`) is a
  possible second caller, but not a visible one.
- **Mistress. Recommended:** `EmuSen.Mistress/BigPicture/`, with three sub-folders.
  - `Theme/`, the loader. It references neither Avalonia, nor any core, nor `MainWindow`. It takes a directory and
    the selection, and gives back plain records. Written that way, it can be lifted into an assembly unchanged on the
    day a second consumer exists.
  - `Scene/`, element state and time.
  - `Render/`, the Skia painter and the one control.

  The SVG renderer is written to `PLAN-icons.md` §5's subset table and seam, so that it can move into that plan's
  sibling package when its stage I1 starts.

Console knowledge, meaning the ES-DE system name (`nes`, `snes`, `n64`, `gb`, `gbc`) and the ScreenScraper system ID,
goes beside each core's declaration, in `CoreDescriptor`. `CoverAspect` and `OpenVgdbBytes` already sit there, for the
reason `EmuSen_Mistress_LibraryPlan.md` §2 gives. Game Boy and Game Boy Color share one descriptor but are separate
shelves (§4.46 of the settings reference), so the mapping is per shelf.

### 4.2 Three layers, and why the view is one drawn control rather than a tree of controls

1. **The loader** reads `capabilities.xml` and then, per system, the theme's files, applying §2.2's order. Its output
   is a `ResolvedView` for each system and view: a list of elements, each a type, a name and a dictionary of typed
   property values with every variable substituted. The output is fixed once the player's selection is fixed.
2. **The scene** turns a `ResolvedView` into live elements. It binds each to data (the selected system, the selected
   game, media paths, metadata, the clock) and advances time-based state: carousel and list scrolling, the delays of
   text containers, the video element's delay, fades during fast scrolling, and transitions. It is a function of
   (resolved view, data, time). Tests can therefore step it with a clock they own, as `PadNavigator` is tested
   (§4.29 of the settings reference).
3. **The painter** draws the scene into one Skia canvas each frame. It runs inside a control that uses the same
   `ICustomDrawOperation` and Skia lease path as Serenity's `GameFrameControl` (`EmuSen_Serenity.md` §2).

**Why not one Avalonia control per element.**

- ES-DE's semantics are paint operations: colour multiply and saturation applied before it, corner radii as fractions
  of the screen width, text truncated with an ellipsis, `maxSize` and `cropSize` fits, tiled textures, zIndex order.
  Each would need a custom control anyway, and the layout system would add nothing, because every position is
  absolute.
- The view takes no pointer input. The pad drives it (§4.9), so hit testing and focus inside it buy nothing.
- One draw pass and one set of cached `SKImage`s is the cheaper shape on the handheld (§4.8).

**What stays Avalonia.** Everything over the view: the pad menu, `SheetLayer` sheets, the on-screen keyboard, dialogs
and the resume question (§4.45 of the settings reference, `LunaP.md` §90–§91). The themed view replaces the library's
content area. It does not replace the window.

### 4.3 Coordinates

For a screen of W×H pixels:

- `pos` becomes (x·W, y·H), and the top-left corner is `pos` − origin × the element's size.
- `size` becomes (w·W, h·H). When one axis is 0, it is derived from the image's aspect ratio.
- `maxSize` fits the image inside its box and keeps its aspect ratio. `cropSize` fills its box and is centred, or
  placed by `cropPos`.
- `fontSize` is multiplied by H. On a vertical screen it would be multiplied by W; no ratio EmuSen targets is
  vertical.
- `cornerRadius` values and `backgroundCornerRadius` are multiplied by **W**.
- Paddings and margins use the axis their name gives.

Art Book Next's 16:10 file states its coordinates as fractions of a 768×480 design. Its XML comments give the pixel
figures, for example 0.02994792 beside "23". Those fractions give the expected pixel boxes at any 16:10 size, so a
layout test can derive expected geometry from the theme itself.

**Aspect-ratio selection.** "Automatic" picks the declared ratio nearest the window's own. 1280×800 and 1920×1200 are
both exactly 16:10.

### 4.4 Fonts and text

Fonts are loaded from the theme by path, as `SKTypeface.FromFile`. They are cached per path and never installed into
Avalonia's font manager: that manager is shared with the rest of the window, and a theme's fonts are not the
window's.

**The one semantic that cannot be read off the page.** `fontSize` is "based on the reference 'S' character"
(§"textlist", §"text"). Whether that means the pixel size handed to the rasteriser, or a size scaled so that a
rasterised 'S' is that tall, decides every text box's height. It is measured against ES-DE in stage (b).

- **P1.** A text box's measured height matches ES-DE's to within 2 pixels at 1280×800.

Text differs in two further ways that make pixel equality with ES-DE the wrong test:

- ES-DE rasterises with FreeType and Mistress would use Skia's rasteriser, so glyph edges differ.
- The textlist's `lineSpacing` is documented as relative to the defined font size, "regardless of what's actually
  being rasterized", unlike every other element's.

Text is therefore compared by box and baseline in stage (b), not by pixels.

### 4.5 SVG

Avalonia draws no SVG (measured in `PLAN-icons.md` §1). The candidates were looked up on NuGet on 2026-09-24:

| Option | Licence | Fit with Avalonia 12.1 and SkiaSharp 3.119.4 (the versions in use) |
|---|---|---|
| `Avalonia.Svg.Skia` / `Avalonia.Svg` | MIT | **Newest is 11.3.0, built against Avalonia 11.3.** There is no Avalonia 12 build. Unusable without a fork. |
| `Svg.Skia` 5.2.x | MIT, but depends on `Svg.Custom` (**MS-PL**) and `ExCSS` | **Depends on SkiaSharp 4.148.0** and HarfBuzzSharp 14.2. Avalonia 12.1 pins SkiaSharp 3.119.4. A major-version clash. |
| `Svg.Skia` 5.1.1 | the same as above | The last release on SkiaSharp 3.119 (it asks for 3.119.2 and HarfBuzzSharp 8.3.1.3). It would work, and would stay pinned there. |
| Our own renderer for the subset themes use | the project's | Parses SVG elements and CSS classes into `SKPath`s: SkiaSharp's `SKPath.ParseSvgPathData` already exists, and fills, strokes, linear gradients, transforms and clip paths map directly onto `SKPaint` and the canvas. |

**The licence question.** The FSF's licence list describes the Microsoft Public License as a free software licence
that is incompatible with the GNU GPL ("Ms-PL", gnu.org/licenses/license-list). That was cited and not re-fetched
here, because the page answered 429 on 2026-09-24. A GPL-3.0 build that bundles `Svg.Custom` would therefore
distribute GPL code combined with an MS-PL library. Whether that combination is acceptable is a legal judgement, not
a technical one. The project's precedent is to make such a judgement deliberately (`EmuSen_Mistress_LibraryPlan.md`
§4, §5).

**Recommendation: our own renderer.** The reasons:

- §3.5 measured that EmuSen's systems and the theme's icons need only paths, polygons, circles and groups, with fill
  rules and transforms.
- The whole theme adds gradients and CSS classes. `PLAN-icons.md` §5 already names both as required, and LunaP's CSS
  parser (`Theme/Css/`, 769 lines) handles a wider vocabulary.
- An element that cannot be drawn is refused visibly (`PLAN-icons.md` §5), not drawn in part.

`Svg.Skia` 5.1.1 is still useful as a **test-time oracle** in WiseMan, which is never distributed. Each logo can be
rendered by both and compared.

- **P2.** Our renderer and `Svg.Skia` agree on the 36 files of §3.5 to an intersection-over-union of at least 0.98
  per filled shape at 256 px.
- **P3.** Our renderer and ES-DE's LunaSVG agree to 0.97, judged from ES-DE captures in stage (b).

Cost: 2–3 days inside stage (b).

### 4.6 Carousel, textlist, grid, and time

- **Carousel (horizontal).** Items sit at equal spacing of `size.x / maxItemCount` about the selected one, which is
  centred. `itemVerticalAlignment` places them, `unfocusedItemOpacity`, `…Saturation` and `…Dimming` apply to the
  unselected ones, and `itemScale` applies to the selected one. With `itemTransitions animate`, a move is "a slide,
  scale and opacity fade animation" (§"carousel"). `fastScrolling` adds a faster tier while the button is held.
- **Textlist.** Rows are `fontSize·H·lineSpacing` apart. The selected row gets `selectedBackgroundColor` with its
  margins and corner radius. A selected name wider than the list scrolls sideways after `textHorizontalScrollDelay`
  (default 3 s).
- **Grid** (stage f). Columns are derived from `itemSize` and `itemSpacing`. `scaleInwards` and `fractionalRows` are
  documented, and they are the only layout rules the grid variants rely on.
- **What THEMES.md does not give: durations and easing.** The slide time of the carousel and grid, the speed of
  container scrolling, and the fade on fast scrolling are not documented. They will be measured from ES-DE, in a
  screen recording at 60 fps of the same inputs, not read from ES-DE's source. That keeps to the user's rule, and a
  measured curve is what the test needs anyway.
  - **P4.** A carousel step settles in 150–400 ms, and our settled positions equal ES-DE's to 1 px.

### 4.7 Video

Art Book Next plays video in 12 of its 20 variants, the first-declared one included (§3.3).

**The degradation is exact.** A `video` element with an `imageType` shows that image during `delay`, and shows it
again when there is no video file ("If `imageType` is not defined, then the default image will be shown if there is
no video file found"; with `imageType` set, the image is what fills the delay and follows `onIterationsDone image`).

**So if no videos are scraped, Mistress's render and ES-DE's are the same render for the same media.** That makes
deferring playback a scope decision, not a fidelity loss. Q4 asks the user whether videos are wanted at all. They
cost the most quota and bandwidth (§5.5), and ES-DE plays their sound by default, because `audio` defaults to true
and Art Book Next does not change it.

Options for when playback is wanted:

| Option | Licence | Weight and fit |
|---|---|---|
| LibVLCSharp 3.10.1 plus `LibVLCSharp.Avalonia` | LGPL-2.1-or-later, compatible with GPL-3.0 | `LibVLCSharp.Avalonia` 3.10.1 is built against **Avalonia 11.3.13**, the same Avalonia 12 gap as §4.5. It needs libVLC and its plugins. NuGet has no `VideoLAN.LibVLC.Linux` package, so on SteamOS's read-only system libVLC would have to be bundled (tens of MB). The desktop has `libvlc.so.5`. |
| FFmpeg through bindings (`FFmpeg.AutoGen` 9.0.1.1, or `Sdcb.FFmpeg` 7.0.0, LGPL-3.0-only) into a frame texture | FFmpeg itself is LGPL-2.1+ when built without GPL parts | Decode only: H.264 and AAC in MP4. **ES-DE's own player is FFmpeg-based** (`es-core/src/components/VideoFFmpegComponent.h` exists; no VLC component does). The desktop has FFmpeg 8.1.2 (`libavcodec.so.62`). What SteamOS provides was not established. |
| FFmpeg as a child process piping raw RGBA frames | the same | No bindings. Needs an `ffmpeg` binary on the machine or bundled. The simplest to build, and the one Serenity's frame path already fits (`UpdateFrame(rgba, w, h)`). |
| Defer | nothing | Exact for the media Mistress fetches (above). |

**Recommendation.** Defer to stage (g). If the user wants videos, use FFmpeg, bindings or process, chosen by a
measurement on the Legion Go S. ScreenScraper's `video-normalized` files are small and low resolution (ES-DE's header
describes them as "smaller file sizes with lower audio quality"), so software decoding is cheap.

- **P5.** Decoding one `video-normalized` clip costs under 5% of one Legion Go S core.

Game audio needs a second audio stream beside Endymion's game sink (`EmuSen_Audio_Sync.md` §7). It also meets the
navigation sounds of §4.9.

### 4.8 Performance (argued; measured in stage b)

- **Per frame.** At most one carousel of nine full-height images, one logo, a clock, the status and the help bar, or
  one list of about 12 rows, one cover and about 15 texts and icons. That is 30–60 draw calls through Skia on the GPU
  backend Avalonia already uses.
- **Memory.**
  - Each system artwork is decoded at display height: at 800 px, 336×800×4 ≈ 1.1 MB, and nine of them ≈ 10 MB.
  - Covers are decoded to at most the element's box (`imageMaxSize` 0.52×0.5625 of 1280×800 is 666×450 ≈ 1.2 MB) on
    a worker, as `CoverArtCache` already does. A 50-entry cache is ≈ 60 MB.
  - SVGs are rasterised once at their drawn size.
- **Loading.** The files one system reads total about 100 KB of XML: `theme.xml` 25 KB, `colors.xml` 12 KB, the 16:10
  file 22 KB, two metadata files about 28 KB, and `capabilities.xml` 11 KB.
- **P6.** Loading the theme for all nine systems and collections takes under 150 ms on the desktop and under 400 ms on
  the Legion Go S.
- **P7.** A steady frame of either view costs under 3 ms on the desktop and under 8 ms on the Legion Go S at
  1280×800. At 1920×1200 it may cost up to 2.25 times the fill share of that, and stays under 12 ms.
- **P8.** Carousel steps hold 60 fps on the handheld with `fastScrolling`, because every system image is preloaded.

The bench is a harness run like `startbench` (§4.42 of the settings reference). Frames are timed inside the render
tick, and runs are interleaved.

### 4.9 Pad, Game Mode, launching, returning

**The grammar is already ES-DE's.** §4.29 of the settings reference took EmulationStation's buttons: south accepts,
east goes back, Start opens the menu, shoulders page, triggers jump, and left and right change the system. In the
themed view:

| Button | System view | Gamelist view |
|---|---|---|
| Left, right | move the carousel (sound `systembrowse`) | change system (`quicksysselect`) |
| Up, down | | move the list (`scroll`), with the repeat of `PadNavigator` |
| South | enter the system's gamelist (`select`) | start the game (`launch`), through `StartGameAsync`: firmware prompt, then the resume question on a sheet (§4.31, §4.45.2) |
| East | | back to the system view (`back`) |
| L1, R1 / L2, R2 | | page / first and last |
| North | | the on-screen keyboard's search (§4.45.6) |
| Select | | mark favourite (`favorite`), as the grid does today (§4.33) |
| Start | the pad menu, as a `SheetLayer` sheet | the same |
| Guide, or Back and Start together, during a game | the menu over the game (§4.29). "Game Library" returns to the themed view at the same system and game. | |

The help system's entries are the theme's layout filled with these actions. Its icons are Mistress's own (§3.6).

**Sounds.** The theme's seven WAV files need a UI sound player. Endymion's `AudioPlayer` carries the game's stream, so
UI sounds want a small second SDL stream or a mix into the same device. That is stage (e), and optional behind a
setting.

**Game Mode.** The themed view is drawn inside the one main window, and everything over it is a sheet (§4.45.2). No
new window is ever opened in a big-screen session.

### 4.10 What happens to the existing big-screen library

It stays. It is the fallback when no theme is installed, and remains a choice after one is. Preferences > Appearance
gains **Library style: Mistress / ES-DE theme**. The pad menu, sheets, resume question and pausing rules are shared by
both, so the themed view changes only what is drawn in the library's place. Q8 asks whether the themed view should also
be offered in a desktop session, where §4.43 keeps the menu bar and sidebar.

---

## 5. ScreenScraper

### 5.1 The API (cited)

The source is `screenscraper.fr/webapi2.php`, read on 2026-09-24. It is in French and says that version 2 is in beta
and may change without notice.

- **Who may use it.** "L'API ScreenScraper ne peut être intégré que dans les applications entièrement gratuites et
  distribuées, ou […] avec l'autorisation préalable" of the ScreenScraper team. The API may be built into free,
  distributed applications, or into others only with the team's prior permission. EmuSen, free and GPL-3.0, is the
  first kind.
- **Credentials.** Every call carries `devid`, `devpassword` and `softname`, which identify the software, and
  optionally `ssid` and `sspassword`, the player's member account. Calls are HTTPS GETs, so the passwords travel in
  the query string. Output is XML by default, or JSON.
- **`jeuInfos.php`** identifies a game.
  - It takes `crc`, `md5` and/or `sha1`, with the note "you must send at least one (best all three) … AND the size",
    plus `systemeid`, `romtype` (rom, iso or folder), `romnom` (the file name with its extension), `romtaille` (the
    size in bytes), `serialnum` and `gameid`.
  - It returns the member's quota fields (§5.5) and the game:
    - `id`;
    - names by region;
    - publisher and developer;
    - players;
    - `note`, out of 20;
    - synopsis by language;
    - dates by region;
    - genres by language;
    - `medias`;
    - the list of known ROMs with their CRC, MD5 and SHA-1;
    - and the matched `rom`.
- **Media** come as URLs in the response. `mediaJeu.php` and `mediaVideoJeu.php` serve them.
  - Passing the local file's `crc`, `md5` or `sha1` returns `CRCOK`, `MD5OK` or `SHA1OK` instead of the file when
    they match, which makes refreshing cheap.
  - `NOMEDIA` means there is no such file.
  - `maxwidth`, `maxheight` and `outputformat` resize an image on the server.
- **`jeuRecherche.php`** searches by name and returns up to 30 games ranked by probability. **`ssuserInfos.php`**
  returns the member's limits.
- **Errors:**

  | Code | Meaning | Response |
  |---|---|---|
  | 400 | malformed request; a file name containing a path is named specifically | fix the request |
  | 401 | API closed to non-members or inactive members because the server is over 60% CPU | back off |
  | 403 | wrong developer credentials | stop |
  | 404 | game or ROM not found | record as unknown |
  | 423 | API closed | stop |
  | 426 | **the scraping software has been blacklisted** (non-conforming or obsolete) | stop; needs a new build |
  | 429 | too many threads, or too many per minute | slow down |
  | 430 | daily quota exceeded | stop until tomorrow |
  | 431 | too many unrecognised ROMs today | stop until tomorrow |

- **The site's own figures on 2026-09-24:** average processing time "Game Info OK" 1.52–1.63 s, "Game Info KO"
  0.51–0.76 s, "Game Search" 0.84–1.48 s, "Game Media" 0.15 s and "Game Video" 0.20 s. 984,580 members, and about 33
  million API calls the day before.

### 5.2 Identity, and `RomHash`

- **EmuSen's identity.** `RomHash.Md5` is "the MD5 of every byte of the file, header included"
  (`EmuSen_Galaxia.md` §5.4). Its job is to recognise a renamed file, and §5.4 already says "a lookup against an
  external database would need the headerless hash as well, and would add it rather than replace this one".
- **ES-DE's practice.** ES-DE sends the whole-file MD5 and the size (`&md5=…&romtaille=…` in `ScreenScraper.cpp`) and
  strips nothing. Its guide advises unzipping single-file games so that hashes match (USERGUIDE "Scraping process"),
  and `EmuSen_Settings_Reference.md` §4.39 records the same.

**The plan, in order, stopping at the first hit:**

1. `jeuInfos` with the whole file's MD5 (the `RomHash` value), CRC32 and SHA-1, all computed in one read, plus
   `romtaille`, `romnom` and `systemeid`. This is the request ES-DE makes, with the other two hashes the API asks for.
2. **Only when (1) returns 404 and the core declares a transform:** the same with the hashes of
   `CoreDescriptor.OpenVgdbBytes`'s output. That output is the NES file without its iNES header, the SNES file
   without a copier header, or the N64 file in the other byte order.
3. A name search, `jeuRecherche`, is used **only** by an explicit "Find by name…" action, never automatically. An
   automatic name match is how the wrong game's box art arrives (ES-DE's guide says as much), and a KO costs from the
   smaller daily allowance.

The KO allowance decides the order. Every 404 at step (1) that step (2) then resolves costs two requests, one of them
a KO.

- **P9.** On 40 files drawn at random from the library, as for §4.39's OpenVGDB measurement, step (1) identifies 75–95%
  and step (2) adds at most 5 points. §4.39 measured 21 of 40 for OpenVGDB, whose misses were hacks, prototypes and
  bad dumps. ScreenScraper also lists many hacks.
- **N64 byte order.** OpenVGDB's N64 hashes needed the byte-swapped order (§4.39). Which order ScreenScraper's
  `rommd5` uses for N64 is unknown and is measured at stage (d).

### 5.3 System IDs (cited from `ScreenScraper.cpp`'s `screenscraper_platformid_map`, which cites `systemesListe.php`)

| EmuSen shelf | ES-DE `system.theme` | ScreenScraper `systemeid` |
|---|---|---|
| NES | `nes` | 3 |
| SNES | `snes` | 4 |
| Game Boy | `gb` | 9 |
| Game Boy Color | `gbc` | 10 |
| Nintendo 64 | `n64` | 14 |
| (not shelves today) Famicom Disk System, Satellaview, Sufami Turbo, Super Game Boy | `fds`, `satellaview`, `sufami`, `sgb` | 106, 107, 108, 127 |

Stage (d) confirms these against `systemesListe.php` once the credentials exist.

### 5.4 Media types, region and language

- **Media type names, as ScreenScraper's API spells them:**

  | Name | What it is |
  |---|---|
  | `box-2D` | cover |
  | `ss` | screenshot |
  | `sstitle` | title screen |
  | `wheel`, `wheel-hd` | marquee |
  | `video`, `video-normalized` | video |
  | `box-3D` | 3D box |
  | `box-2D-back` | back cover |
  | `fanart` | fan art |
  | `support-2D` | physical media |
  | `manuel` | manual |
  | `mixrbv1`, `mixrbv2` | mix images |

  The names are in the API's `jeuInfos` description, and `ScreenScraper.h` names the same.
- **Fetched by default** (§3.7): `box-2D`, `ss` and `wheel-hd` (else `wheel`), plus the text metadata. Behind
  settings: `sstitle` (the fallback when there is no screenshot), `video-normalized` (Q4) and `mixrbv2` (Q7).
- **Region.** The preferred region comes from the file name's No-Intro tag: USA → `us`, Europe → `eu`, Japan → `jp`,
  World → `wor`. The player can override it. The fallback order is ES-DE's documented one, "world, USA, EU, Japan and
  custom" (USERGUIDE, "Enable fallback to additional regions"), then any region at all behind that same opt-in.
  Videos and fan art carry no region.
- **Language.** Preferred, then `en`, as ES-DE does for synopsis and genre. Defaults to `en`.
- **Rating.** `note` is out of 20. ES-DE stores a rating as a fraction shown in half stars, so the value becomes
  note / 20, rounded to the nearest 0.1 (one half star).

### 5.5 Quotas, threads, pacing

**Cited.**

- `ssuserInfos` and every `jeuInfos` response return `maxthreads`, `maxdownloadspeed` (KB/s), `requeststoday`,
  `requestskotoday`, `maxrequestspermin`, `maxrequestsperday` and `maxrequestskoperday`.
- The API page says that managing quota inside the software "est désormais obligatoire": it is now mandatory for
  software to manage quota itself.
- Secondary sources report the tiers (RomM issue #3978; not a ScreenScraper page):
  - a new free account has 1 thread at 128 KB/s;
  - contributors rise to 8 threads;
  - a one-off donation of €10 adds 5 threads;
  - per minute, 50 requests per thread;
  - per day, 20,000 to 100,000 requests, with a KO allowance a tenth of that.

**The client's rules (argued):**

- Read the limits from the first response, and again from every response.
- Never open more workers than `maxthreads`.
- Pace to `maxrequestspermin` less 10%.
- Stop for the day at `maxrequestsperday` less 2%, or on 430 or 431.
- On 429, halve the pace and retry after 60 s. On 401, wait five minutes.
- On 403, 423 or 426, stop and say why.
- Show the player the day's remaining quota.

**Cost of a whole library (argued from the figures above).**

- The library holds 5,520 files (§4.33).
- Each game costs one `jeuInfos` of about 1.5 s, plus three images. At 128 KB/s, three images of 100–500 KB take 2–12
  s together. That is about 5–14 s a game, and **8–21 hours** for the whole library on one thread.
- If media downloads count against the daily quota, which is unknown and measured in stage (d) by reading
  `requeststoday` before and after, 4 requests a game make 22,080 requests. That is **more than one day** of a new
  account's 20,000.
- Videos add 1–3 MB a game, or 8–25 s at 128 KB/s.

A full scrape is therefore a job that must resume across days.

- **P10.** A new account scrapes the whole library without videos in two to three sessions spread over two days.

**Priority.** A player browsing a gamelist should not wait for a nine-hour queue. Games are queued in the order in
which they are shown, as §4.39's covers are. Then the console the player is looking at is queued, and then the rest.
The whole-library pass is one explicit action that the player starts.

### 5.6 Storage

- **Media files: `home/Media/<es-de system>/<type>/<rom stem>.<ext>`.** A new `DataStore.Media`. The layout is
  ES-DE's documented `downloaded_media/<system>/<type>` (USERGUIDE, "Manually copying game media files"): `covers`,
  `screenshots`, `marquees`, `videos`, `titlescreens`, `miximages`. Three things follow:
  - a player's existing ES-DE media folder can be chosen instead, as a setting, and read in place;
  - the naming rule is someone else's documented one, not a new invention;
  - Mistress writes only inside `home/Media`, never beside a ROM. The ROM library is read, never written
    (`EmuSen_Galaxia.md` §3.3a).

  Writes follow §4.39's rule: to a temporary name beside the final one, moved into place, never replacing an
  existing file. Only a response that says it is an image, or an MP4, and is at least 80 bytes is kept.
- **Metadata: SQLite, a new `home/Media/media.db`**, with `PRAGMA user_version` and an append-only migration list, as
  `GameRecords` does (§4.32). Tables:
  - `scrape_game`: key the RomHash MD5 plus size; the ScreenScraper game and ROM IDs; `systemeid`; name;
    description; developer; publisher; genre; players; rating; release date; the region and language used;
    `fetched_at`; and a status of Found, Unknown or Error;
  - `scrape_media`: MD5, type, relative path, source region, the file's SHA-1, `fetched_at`;
  - `scrape_queue`: MD5, path, priority, attempts and `next_try`, so that a stopped job resumes where it was;
  - `quota_day`: the day and the counts, so that the stop rule survives a restart.

  **Why a new file and not `games.db`.** §4.32 keeps `games.db` for what the player made, which cannot be re-fetched.
  §2.3 of `EmuSen_Stack.md` calls the catalogue a deletable cache. Scraped data is neither: it can be re-fetched, but
  only at the cost of days of quota. Keeping it beside the files it describes means the media folder and its index
  are copied, or deleted, together. The driver sits in Mistress, its only reader, as `GameRecords` does.
- **Renames.** Rows are keyed by hash, so §4.37's orphan pass extends to them. When a game's file is renamed, its
  media files are renamed to the new stem. They live in Mistress's own folder, so that is allowed where renaming save
  states was not.
- **Local media first.** A cover already found by `ArtworkIndex` in `home/Artwork` (§4.33, §4.39) satisfies the
  `cover` media type with no request.

### 5.7 Credentials: what the user must do, and how they are kept

**What the user must do.**

1. **Create a ScreenScraper member account** at `screenscraper.fr`. That gives an `ssid` (user name) and `sspassword`.
   The member account carries the quota and threads of §5.5, and contributing to the database or donating raises
   them.
2. **Request developer credentials for EmuSen.** The API page says developers should "contactez-nous via le forum pour
   présenter votre logiciel", that is, introduce the software on the ScreenScraper forum. The team then issues an
   identifier and password for the software. Reports say the team does not issue developer credentials to users, only
   to software authors, so the request must come from the user as EmuSen's author. The post should give:
   - the software's name as it will be sent in `softname`. This is fixed forever, because blacklisting (426) is by
     name. Suggested: `EmuSen-Mistress`.
   - that EmuSen is free, GPL-3.0 and distributed, with the repository's link;
   - what it fetches (§5.4), and that it honours the quota fields (§5.5).
3. When the credentials arrive, **put them in a file outside the repository**: `screenscraper-developer.json`, holding
   `{ "devid": "…", "devpassword": "…", "softname": "EmuSen-Mistress" }`, in the config directory (`ConfigStore`;
   `~/.config/EmuSen/` on the development machine), with mode 0600.
4. **Enter the member account in Mistress:** Preferences > Library > ScreenScraper, on the on-screen keyboard from a
   pad.

**How they are kept.**

- **Never in git.** The developer file lives outside every repository tree. `.gitignore` gets a line for its name
  anyway, as a second guard. A WiseMan test fails if `git ls-files` lists a file of that name, or if any tracked file
  contains the literal `devpassword=` followed by something other than a placeholder.
- **Never in a published zip by accident.** The member account goes in its own file, `screenscraper.json`, in the
  config directory, with mode 0600. It is **not** in `appsettings.json`, which people paste into bug reports. The
  publish recipe excludes both files from the zip, as it already excludes the live sandbox's cheats and saves
  (§"Publishing to out/").
- **Never logged.** Every URL that reaches a log, the status line, `CrashLog` or an exception message passes through
  one redactor, which blanks `devpassword`, `sspassword` and `ssid`. A test holds it.
- **Shipping the developer credentials (Q5).**
  - ES-DE compiles its developer credentials into its binary, XOR-scrambled with a key held beside them in
    `ScreenScraper.h`. That is obfuscation, not secrecy: anyone with the binary can recover them. The same holds for
    any published build that works without the player's own developer credentials.
  - The option that keeps them out of the repository: at publish time, MSBuild reads the developer file named by an
    `EmuSenScreenScraperDeveloper` property and generates an embedded resource into `obj/`, which is gitignored.
  - The alternative: builds carry no developer credentials, and scraping works only where the developer file exists.
    That means only on the user's machines, since players cannot obtain developer credentials themselves (step 2).
  - The recommendation is the first, with Q5 confirming it.

### 5.8 What is sent, and the licence posture

What is sent:

- the file's name, size and three hashes;
- the system ID;
- the developer credentials;
- the member account.

What is received is community-contributed data and publishers' art. EmuSen stores it for the player and redistributes
none of it.

As with §4.39, the feature is **off by default**: it takes one explicit action to turn on, and the switch says what it
sends and to whom. The API's own condition (free, distributed software) is met.

---

## 6. Theme management

- **Where themes live: `home/Themes/<repository name>/`**, a new `DataStore.Themes`. A theme folder the player already
  has, for example an ES-DE `themes` directory, can instead be read in place. Mistress never writes into a folder it
  did not download.
- **Downloading Art Book Next.** One button, with the pattern of `SlangPackDownload` (§4.41):
  - Mistress fetches the GitHub archive of the default branch (`codeload.github.com/<owner>/<repo>/zip/refs/heads/<branch>`).
    No git client is needed. ES-DE's own downloader clones the repository, and the list gives `.git` URLs.
  - The archive is written beside its final place and unpacked into a sibling folder. Entries that would land outside
    it are refused.
  - The unpacked theme is checked for `capabilities.xml` and a loadable `theme.xml` before it replaces anything.
  - **P11.** The download is 205–230 MB, because the working tree is 226.7 MB and PNG barely compresses.
- **Any other theme** comes from a GitHub or GitLab repository URL, turned into its archive URL, from a local folder,
  or from ES-DE's official list, `themes-list/themes.json`. That list holds 66 themes, each with its variants, colour
  schemes, aspect ratios and screenshots, and it can be shown as a picker. The list is read, never mirrored.
- **Updating.**
  - `.emusen-theme` beside the theme records the source URL, the branch, the commit hash (GitHub's
    `commits/<branch>` API, which allows 60 unauthenticated calls an hour) and the date.
  - "Update available" is shown when the upstream hash differs, and the player chooses to update.
  - **An update keeps `theme-customizations/`.** That is where Art Book Next's README tells a player to put a custom
    `colors.xml`, artwork and logos, "while also allowing you to continue to get updates from the theme downloader".
- **Settings.** A Big Picture sheet is built from `capabilities.xml`: theme, variant, colour scheme, font size, aspect
  ratio (Automatic, then the declared ones), transitions, and a sounds switch.
  - Only `selectable` variants are listed.
  - The labels are the theme's own, in `en_US` when a label has several languages.
  - The values are stored in `appsettings.json` under `BigPicture`. They are configuration a person chooses, so they
    stay JSON (`EmuSen_Stack.md` §4.1).
  - A change reloads the loader's output and takes effect at once.
- **Attribution.** An "About this theme" sheet shows:
  - the theme's name and author;
  - its licence line and its credits, both **read from the downloaded README at display time**, so that EmuSen
    carries no copy of either and the text follows upstream;
  - the source URL and commit;
  - a statement that the theme was downloaded at the player's request and is not part of EmuSen.

  It opens once after the first download and is always reachable from the settings sheet. For a theme with no README,
  the sheet shows its `LICENSE` file, or says that it has none.
- **Removing** deletes `home/Themes/<name>` after a confirmation. It never touches a folder read in place.

---

## 7. Stages

| Stage | What it covers | Its oracle | Cost | What the user sees at the end |
|---|---|---|---|---|
| a | The loader and `capabilities.xml`; a `theme inspect <dir>` inventory (a DianaOS command, or a WiseMan tool) printing §3.3's tables for any theme | Golden parse tests on **synthetic themes written for the tests** (a `SyntheticTheme` builder beside `SyntheticRom`), one per rule of §2.2, including THEMES.md's own examples of the parsing order and the variable trap, and §3.4's undefined-variable include. Against Art Book Next, a local golden kept outside the repo. | 2–3 days | `theme inspect` prints Art Book Next's inventory. Every combination of variant, scheme, font size and aspect ratio loads for the five systems and collections. |
| b | The painter and every first-version element of §3.8 drawn statically: the system view at rest; the gamelist view with media, metadata and video shown as its image. The SVG subset renderer, fonts, the help icons. | **ES-DE itself** (below). Element boxes to within 1–2 px; pictures by region, with text compared by box. `Svg.Skia` as a second opinion on SVG. | 5–7 days | A still of the system view and a gamelist, next to ES-DE's, at 1280×800. |
| c | Time: carousel slide and fade, list scroll and repeat, horizontal scrolling of the selected name, the text containers, fast-scroll fades, the video element's delay, the instant transitions; then slide | 60 fps recordings of ES-DE under the same inputs; settled positions exact; durations within 10% | 2–3 days | The view moves as ES-DE's does. |
| d | ScreenScraper client, quota manager, queue and resume, the media store and `media.db`, the region and language rules, the settings | A fake server built on documented responses we write ourselves (as `OnlineCoverTests` does), never the network. Then one live run of 40 random files once the credentials exist (P9, P10). | 3–4 days, plus waiting for the credentials | Games gain covers, screenshots, marquees and metadata. Quota is shown. |
| e | Pad: the themed view as the big-screen library; launching; the menu over the game; returning to the same place; sheets; help entries; navigation sounds | `PadDriver` tests (§4.45.1): a simulated pad drives the view, starts a synthetic game, returns; each rule gets a mutant | 2–3 days | The handheld starts in Art Book Next, plays a game and comes back. |
| f | Variants and settings: the settings sheet from capabilities; the grid variants; variant triggers against the media store; downloading, updating and removing themes; the attribution sheet; `theme-customizations` kept | Synthetic themes; a fake archive server (as §4.41's tests); ES-DE captures of the grid | 2–3 days | Every variant and colour scheme from the pad; update and about. |
| g | Video, if Q4 says yes | ES-DE recordings; decode cost on the Legion Go S (P5) | 3–5 days | Clips play after 3 s, once. |

**How ES-DE is used as the oracle, and whether that is legal and practical.**

- **Legal.** ES-DE is MIT-licensed free software, and running it is unrestricted. Its captures of Art Book Next are
  renders of a CC BY-NC-SA work, made privately and non-commercially. They are kept under `~/.cache/emusen/bigpicture/`
  and never committed. Tests that need them skip when they are absent, as `RealRom` tests do.
- **Practical.**
  - ES-DE is not installed on the desktop, and there is no Xvfb, xdotool or gamescope (checked 2026-09-24). Stage (b)
    therefore starts by installing ES-DE's Linux AppImage from es-de.org. It is run windowed with
    `es-de --resolution 1280 800 --home <scratch>`, the flags THEMES.md documents, so that it writes nothing into the
    real home, and with `--debug`.
  - The captures are taken with the desktop's screenshot tool.
- **The inputs are shared.**
  - A ROM directory of empty dummy files, which THEMES.md itself recommends for theme testing.
  - A `downloaded_media` tree generated by a test tool: flat-coloured PNG covers, screenshots and marquees of known
    sizes, each labelled by the tool. No real game art enters a reference.
  - Mistress reads the same tree through §5.6's "use an existing ES-DE media folder".
  - The same pictures therefore go into both engines, and any difference is the renderer's.
- **Geometry from ES-DE's own debug mode.** Debug mode's `Ctrl+i` and `Ctrl+t` draw the box of every image and text
  element (§"Debugging during theme development"). Those boxes are the geometric oracle, which anti-aliasing cannot
  blur.
- **A sanity check, not a test.** The four screenshots in ES-DE's theme list, and the two in the ES-DE edition's
  README, are 16:9 and use the author's own media. They serve as a visual check.

**Finished** means:

- a player on the handheld reaches Art Book Next without a desktop;
- every element of §3.8's first column matches ES-DE on the shared inputs, or the difference is written down here;
- scraping keeps the quota rules;
- nothing from the theme or from ScreenScraper is in the repository.

---

## 8. What was considered and not recommended

- **A native look-alike built from LunaP controls.** Refused by the user. It would also have to be redesigned for
  every theme.
- **Supporting the Batocera format.** It is a different format (§1.1), undocumented beyond its fork, and ES-DE itself
  gave up reading legacy themes. It is not recommended unless a theme exists only in that form (Q1).
- **Bundling a theme.** Refused by the licence (NC and SA against GPL-3.0), and by the brief.
- **An automatic name search.** Refused (§5.2).
- **Generating miximages as ES-DE does.** Deferred (Q7). It is a compositing tool of its own, and ScreenScraper's
  `mixrbv2` is the cheaper substitute.

---

## 9. Predictions to be retired

| # | Prediction | Retired when |
|---|---|---|
| P1 | Text boxes match ES-DE's height to within 2 px at 1280×800 | Stage b |
| P2 | Our SVG renderer and `Svg.Skia` agree to an IoU of at least 0.98 on §3.5's 36 files | Stage b |
| P3 | Our SVG renderer and LunaSVG (from ES-DE captures) agree to at least 0.97 | Stage b |
| P4 | A carousel step settles in 150–400 ms, positions equal to ES-DE's to 1 px | Stage c |
| P5 | One `video-normalized` clip decodes in under 5% of a Legion Go S core | Stage g |
| P6 | The theme loads for nine systems in under 150 ms (desktop) and 400 ms (Legion Go S) | Stage b |
| P7 | A steady frame costs under 3 ms (desktop) and 8 ms (Legion Go S, 1280×800), and under 12 ms at 1920×1200 | Stage b, handheld in e |
| P8 | Carousel steps hold 60 fps on the handheld with fast scrolling | Stage c |
| P9 | Hash lookup identifies 75–95% of 40 random files; headerless hashes add at most 5 points | Stage d |
| P10 | A new account scrapes the whole library, without videos, over two to three sessions in two days | Stage d, first full run |
| P11 | Art Book Next's archive is 205–230 MB | Stage f |
| P12 | With no videos scraped, the video element's render is identical to ES-DE's (§4.7) | Stage b |
| P13–P18 | Stage (a)'s predictions: coverage, errors, skipped includes, load time, triggers, the default variant (§12.1) | Stage a (§12.5) |

---

## 10. Risks and open questions for the user

- **Q1, which theme.** Confirm the ES-DE edition (`art-book-next-es-de`). The linked Batocera edition cannot be read by
  ES-DE, and its options differ (§1.1).
- **Q2, the handheld's resolution.** 1280×800 or 1920×1200? Is the Legion Go S's panel driven at its native
  resolution in Game Mode, or does gamescope scale up from 1280×800? §4.8 prices both.
- **Q3, how much of the format.** The recommendation is §3.8's first column, then more only when a theme the user
  wants needs it. The loader is complete from the start, because a partial grammar breaks whole themes.
- **Q4, video.** Scrape and play videos (stage g, FFmpeg, 3–5 days, more quota and bandwidth, sound on by default in
  the theme), or not? Without them the render is exact and the cost is nil (§4.7).
- **Q5, the developer credentials in published builds.** Embed them at publish time from a file outside the
  repository, as ES-DE does, or ship builds that scrape only on the user's own machines? Either way they are never
  committed (§5.7).
- **Q6, the member account.** Contributing to ScreenScraper, or a one-off €10, raises threads and quota and cuts §5.5's
  two days to hours. That is the user's choice, not the software's.
- **Q7, miximages.** One variant needs them. Use ScreenScraper's `mixrbv2` (the nearest, and a different look), build
  a miximage generator (a later stage), or hide that variant?
- **Q8, the existing big-screen library.** Keep it as the fallback and a choice, as recommended (§4.10)? Offer the
  themed view on the desktop too?
- **Q9, engine-supplied graphics.** The help bar's button icons, the favourite and folder indicators and the badge
  overlay must be Mistress's own drawings (§3.6). Is a simple drawn set acceptable, or should they match a particular
  controller family, such as Xbox, PlayStation, or the Legion's own labels?
- **Q10, SVG.** Our own subset renderer (recommended), or `Svg.Skia` 5.1.1 with its MS-PL dependency and SkiaSharp
  pin (§4.5)?
- **Risk: ScreenScraper's API is in beta** and may change without notice (§5.1). The client keeps the response
  parsing in one place, and its tests are written against the documented shape.
- **Risk: the undocumented behaviours** of §4.4 (the 'S' size) and §4.6 (durations) are measured from ES-DE, so they
  track the ES-DE release measured. The release is recorded with every capture.
- **Risk: the theme changes upstream.** Updating is the player's choice (§6). A new theme version that uses an element
  outside §3.8 is refused visibly, element by element, not rendered in part.

---

### 10.1 Decided by the user (2026-09-24)

- **Q1, the theme:** the **ES-DE edition**, `art-book-next-es-de`.
- **Q2, the resolution:** answered from the device, not asked. Game Mode's gamescope session is started with
  `-w 1280 -h 800` (read from the running process on 2026-09-24), so Mistress draws at 1280×800 there; the panel
  itself is 1920×1200. Both are 16:10, so the variant chosen is the same; performance is to be measured at 1280×800
  first and at 1920×1200 on the desktop's Desktop Mode window.
- **Q3, how much of the format:** **the full ES-DE format**, not only what Art Book Next uses. §3's inventory becomes
  the first test theme and the order in which elements are built, not the boundary of the work; §7's estimate grows
  accordingly, and stage (a)'s golden tests must cover every element type and property ES-DE documents.
- **Q4, videos:** **not now.** Stage (g) is deferred; the video element renders its fallback image, as ES-DE does
  when no video file exists.
- **Q5, developer credentials:** **the user's own machines only.** Published builds do not carry them; a build
  without the local file cannot scrape.
- Q6–Q10 remain open; each is asked when its stage reaches it.
- **Drawn with LunaP (the user, 2026-09-24): "make sure we are drawing this with LunaP and if something is missing
  from LunaP, add it".** This supersedes §4's one Skia-drawn control in Mistress. Every visible part is a LunaP
  control, and what LunaP lacks is added to LunaP under its own conventions (a `docs/LunaP.md` section, tests, the API
  baseline, palette-only colours where a theme does not set one): an SVG image (the renderer §4 argued for, now in
  LunaP), the carousel, the grid and text list as themed, text in a theme's own fonts, rating, badges, the help bar,
  the clock and system status, and a positioned canvas for ES-DE's normalised coordinates and origins. The ES-DE
  loader (XML, variables, includes, variants, colour schemes, aspect ratios) is format-specific and stays in
  `EmuSen.Mistress/BigPicture/`, with no Avalonia types, producing a scene the Mistress layer builds from LunaP
  controls. Q10 (own SVG renderer or `Svg.Skia`) is therefore answered: our own, in LunaP.

## 11. Sources

- ES-DE: `THEMES.md`, `USERGUIDE.md`, `LICENSE`, `es-app/src/scrapers/ScreenScraper.cpp` and `.h`, read at master on
  2026-09-24. `gitlab.com/es-de/emulationstation-de`.
- ES-DE's theme list: `gitlab.com/es-de/themes/themes-list`, `themes.json`.
- Art Book Next, ES-DE edition: `github.com/anthonycaccese/art-book-next-es-de` at `d772d07`, 2026-02-06. README,
  `capabilities.xml`, `theme.xml`, `colors.xml`, `aspect-ratio-16-10.xml`, `_inc/`.
- Art Book Next, Batocera edition: `github.com/anthonycaccese/art-book-next-es` at `9a50ef3`, 2026-03-11.
- ScreenScraper: `screenscraper.fr/webapi2.php` (WebAPI v2 beta), read 2026-09-24.
- Secondary, on ScreenScraper's tiers and credentials: RomM issue #3978 (`github.com/rommapp/romm/issues/3978`);
  RockNIX fork issue #64 (`github.com/maxengel/rocknix/issues/64`); JellyEmu issue #219.
- NuGet package manifests, 2026-09-24: `Svg.Skia` 5.0.0–5.2.3, `Svg.Custom` 5.2.3, `Svg.Model` 5.2.3,
  `Avalonia.Svg(.Skia)` 11.3.0, `Avalonia.Skia` 12.1.0, `LibVLCSharp(.Avalonia)` 3.10.1, `FFmpeg.AutoGen` 9.0.1.1,
  `Sdcb.FFmpeg` 7.0.0.
- FSF, "Various Licenses and Comments about Them", entry Ms-PL (`gnu.org/licenses/license-list.html#ms-pl`). Cited,
  not re-fetched: the page answered 429.
- EmuSen: `EmuSen_Settings_Reference.md` §4.29, §4.31–§4.33, §4.37, §4.39, §4.41, §4.42, §4.43, §4.45;
  `EmuSen_Galaxia.md` §3, §5.4; `EmuSen_Stack.md` §2.3, §4.1, §5; `EmuSen_Multicore.md` §9;
  `EmuSen_Serenity.md` §2; `EmuSen_Mistress_LibraryPlan.md` §2, §4; LunaP `docs/LunaP.md` §1, §21, §88, §90, §91,
  §95, and `PLAN-icons.md` §1, §5, §8.

---

## 12. Stage (a): the ES-DE theme loader

*Opened 2026-09-24.* Stage (a) builds the loader of §4.1 in `EmuSen.Mistress/BigPicture/Theme/`, to §10.1's scope: the
whole of `THEMES.md`'s format, not only Art Book Next's part of it. It has no Avalonia types. Its output is a resolved,
immutable scene model per view. Drawing is stage (b).

### 12.1 Predictions, written before the loader was built

The reference section of `THEMES.md` ("Element types and their properties") documents **15 element types and 467
properties**. They were counted by a scratch script that reads the section's `` * `name` - type: TYPE `` lines. The
navigation-sounds section adds a sixteenth type, `sound`, with one property (`path`). The format therefore has **468
(element type, property) pairs**.

- **P13.** Art Book Next sets **172 of the 468 pairs (37%)**, the static count of §3.3. The loader's union over every
  combination it is run with will find exactly 172. A difference will name either a block that the static walk counted
  but no choice selects, or a property the walk missed.
- **P14.** Every combination run loads with **zero errors, zero unknown elements and zero unknown properties**, for the
  five EmuSen systems (`nes`, `snes`, `n64`, `gb`, `gbc`) and the four collections of §3.1.
- **P15.** Each load skips exactly **one** include, silently, as "built from a variable": `colors.xml`'s
  `${customizationPath}`. It is undefined in 30 of the 31 colour schemes, and in `custom` it names a file the clone does
  not have. `_metadata-global/${system.theme}.xml` exists for all nine systems (checked), and the grid variants'
  `_coversize/${systemCoverSize}.xml` exists for every value those files set, so neither is skipped.
- **P16.** Loading one system's two views takes **under 20 ms** on the desktop once warm, so the loader's share of P6
  (nine systems in 150 ms) holds with room to spare.
- **P17.** With no scraped media, the 12 `noMedia` overrides of §3.2 replace each of the 12 variants that carry one
  with `gamelist-list-basic` or its `-nh` twin, in the gamelist view only. The system view keeps the chosen variant.
- **P18.** With no variant chosen, the loader picks the **first declared**, `gamelist-list-metadata-cover`. `THEMES.md`
  does not say which variant ES-DE picks, so this is the loader's rule, and not yet shown to be ES-DE's, until stage (b)
  runs ES-DE.
