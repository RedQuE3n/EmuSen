# EmuSen_BigPicture — a plan for a big-picture mode in Mistress that renders ES-DE themes

*Written 2026-09-24. Stage (a), the theme loader, was built the same day; its record is §12. Stage (b), the two views drawn statically with LunaP controls, followed on 2026-09-24 and 25; its record is §13. Stage (c), the GPU frame and motion, followed on 2026-09-25; its record is §14. Nothing else is built.* The user asked for a big-picture mode in Mistress like EmulationStation's,
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

> **Retired 2026-09-24, before any of it was built.** The single Skia-drawn control argued below was superseded by the
> user's decision of §10.1: "make sure we are drawing this with LunaP and if something is missing from LunaP, add it".
> The argument is kept because its premises were not wrong, only outweighed. It judged the view by what one frontend
> needed, and a single drawn control is the cheaper shape for one frontend. The user judged it by what the toolkit
> should be able to do: every part a theme draws (an image fitted and tinted, text in a font from a file, an SVG, a
> list, a carousel, a rating) is a thing another LunaP consumer can also want, and a drawn control in Mistress would
> have kept all of it out of reach. Whether the three reasons below held up once the controls were written, and the
> cost of a tree against one draw pass (P19), are recorded in §13, not argued here.

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
| P1 | Text boxes match ES-DE's height to within 2 px at 1280×800 | Stage b: held (§13.10) |
| P2 | Our SVG renderer and `Svg.Skia` agree to an IoU of at least 0.98 on §3.5's 36 files | Stage b: held (§13.7) |
| P3 | Our SVG renderer and LunaSVG (from ES-DE captures) agree to at least 0.97 | Stage b: failed (§13.8) |
| P4 | A carousel step settles in 150–400 ms, positions equal to ES-DE's to 1 px | Stage c: held, 396–404 ms (§14.10); settled positions held in stage b (§13.8) |
| P5 | One `video-normalized` clip decodes in under 5% of a Legion Go S core | Stage g |
| P6 | The theme loads for nine systems in under 150 ms (desktop) and 400 ms (Legion Go S) | Stage b |
| P7 | A steady frame costs under 3 ms (desktop) and 8 ms (Legion Go S, 1280×800), and under 12 ms at 1920×1200 | Stage b, handheld in e |
| P8 | Carousel steps hold 60 fps on the handheld with fast scrolling | Stage c: open until the handheld runs deck-gpu (§14.10) |
| P9 | Hash lookup identifies 75–95% of 40 random files; headerless hashes add at most 5 points | Stage d |
| P10 | A new account scrapes the whole library, without videos, over two to three sessions in two days | Stage d, first full run |
| P11 | Art Book Next's archive is 205–230 MB | Stage f |
| P12 | With no videos scraped, the video element's render is identical to ES-DE's (§4.7) | Stage b: failed as identical (§13.8) |
| P13–P18 | Stage (a)'s predictions: coverage, errors, skipped includes, load time, triggers, the default variant (§12.1) | Stage a (§12.5) |
| P19–P23 | Stage (b)'s predictions: frame cost, SVG against `Svg.Skia` by file, properties mapped, geometry from the theme, the font size (§13.1) | Stage b (§13.10) |
| P24–P38 | Stage (c)'s predictions: the GPU route, its frame, first frame and uploads, the handheld; the carousel step, repeat, the list, the name, containers, scrollFadeIn, the video delay, the slide, settled frames and the moving frame (§14.1, §14.5) | Stage c (§14.10); P28 open |

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
- **Q8 and Q9, and navigation sounds (the user, 2026-09-25, before stage e):**
  - **Q8:** the existing big-screen library **stays**, as the fallback when no theme is installed and as a choice
    after one is (Preferences, "Library style"). The themed view is offered **in big-screen sessions only**; the
    desktop keeps its sidebar library.
  - **Q9:** the help bar's button icons **follow the connected pad**: Mistress detects the controller family and draws
    its own set for it (Xbox, PlayStation, Nintendo, and a generic set when unknown). The favourite, folder and badge
    graphics are Mistress's own drawings.
  - **Sounds:** the theme's navigation sounds play through a small UI sound stream, **on by default** with a switch in
    Preferences.
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

### 12.2 What the loader implements

The code is nine files in `EmuSen.Mistress/BigPicture/Theme/`. None names an Avalonia, LunaP or core type.

| File | What it holds |
|---|---|
| `ThemeCatalog.cs` | The schema: 16 element types, 468 properties, each with its type, literal default, "same value as" default, default stated in words, range, allowed values, the view it is limited to, and whether it is deferred |
| `ThemeCapabilities.cs` | `capabilities.xml`: variants with labels and overrides, colour schemes, font sizes, languages, aspect ratios, transition profiles, suppressed built-ins |
| `ThemeChoices.cs` | The system and its twelve system variables; the player's choices; their resolution against the capabilities; the variant triggers |
| `ThemeLoader.cs` | One parse per variant, in the documented parsing order; includes, variables, selection blocks, views and elements |
| `ThemeViewBuilder.cs` | Merging, typing, ranges, view limits, defaults, bindings, drawing order |
| `ThemeValueParser.cs`, `ThemeValues.cs` | The seven documented types, paths, list splitting, clamping |
| `ThemeDiagnostics.cs`, `ResolvedTheme.cs` | Diagnostics with a severity, and the immutable output |

**Where the catalogue came from.** A scratch script read every `` * `name` - type: TYPE `` line of the reference section,
with the default, range, allowed values and view limit written beneath it, and drafted the catalogue from them. The
draft was then corrected by hand in 25 places, each of which the reference states in words rather than as a literal:

- the defaults stated as a rule: the carousel's and grid's `text` (the system's full name); the textlist's
  `selectorWidth` (the element's width) and `selectorHeight` (1.5 times `fontSize`); `interpolation` on image, video,
  animation, badges and rating (nearest at 0, 90, 180 or 270 degrees, else linear); the text's `container` (true for
  `description`) and `containerStartDelay` (4.5 s vertical, 1.5 s horizontal); the datetime's `displayRelative` (true
  for `lastplayed`); the help system's `pos` and `fontSize`, which differ for vertical screens;
- the datetime's `format`, whose default is written as "the ISO 8601 standard notation `%Y-%m-%d`";
- the pair rules: a zero axis of `size` on image, video (and its `imageSize`) and animation, and of `tileSize`, means
  "from the aspect ratio"; `-1` on the grid's `itemSize` and `itemSpacing` and the badges' `itemMargin` means "the other
  axis"; the rating's `size` sizes by one axis and caps y at 0.5;
- the badges' `customBadgeIcon` keys, which are the badge slots.

The catalogue is then checked two ways (§12.6): against 39 entries read from `THEMES.md` by hand, and by a test that
sets every one of the 468 properties on a synthetic theme and reads back its typed value.

**The grammar, as built. Cited unless marked "chosen", in which case §12.4 gives the reason.**

- **Files.** Without `capabilities.xml` the theme is not loaded. An empty file, or one holding only a comment, declares
  nothing, as `THEMES.md` says it may. The entry file is `<theme>/<system.theme>/theme.xml`, else `<theme>/theme.xml`.
- **Parsing order.** Every `<theme>`, `<variant>` and `<aspectRatio>` block is processed in nine passes over its
  children: transitions (in a variant), variables, colour schemes, font sizes, languages, includes, views, variants,
  aspect ratios. Each pass keeps file order. An include runs all nine passes on its file at its own place.
- **Selection blocks.** Name lists split on commas and whitespace. A `<variant>` applies when it names the variant or
  `all`. A `<colorScheme>`, `<fontSize>`, `<language>` or `<aspectRatio>` applies when it names the selected one. Names
  a block uses but capabilities does not declare are warned about once and never selected.
- **Includes.** `./` is the including file's folder, `~` is home, and backslashes are separators. A missing include
  written out is an error. One built from a variable, whether its variable is undefined, it resolves empty, or its file
  is missing, is a debug note, and the load goes on. Include loops are refused as an error, where ES-DE would hang
  (chosen).
- **Variables.** One global namespace. The twelve system variables come first, and their `.autoCollections`,
  `.customCollections` and `.noCollections` forms are empty for the other kinds of system. A variable's value is
  substituted when it is defined (chosen), so a later redefinition of a variable it names does not change it. A
  redefinition inside a variant or aspect ratio changes the global value, from that point in the parse on.
- **Elements.** Definitions of one type and name in one view merge property by property, and the last value wins.
  Names and views may be lists. An element keeps the position of its first definition.
- **Views.** `system`, `gamelist` and `all`, which holds only `sound`. An element type in a view the reference does not
  list for it is ignored with a warning. A property limited to the other view ("can only be used in the `system`
  view") is ignored with a warning, and its default applies.
- **Primary elements.** One per view: a second carousel, grid or textlist is ignored with a warning (chosen).
- **Drawing order.** zIndex low to high; ties by first definition (chosen); `helpsystem`, `clock` and `systemstatus`,
  which have no zIndex, last.
- **Triggers.** For the gamelist view only, `noMedia` before `noVideos`, one step deep. `mediaType` defaults to
  `miximage`. The system view is parsed with the chosen variant and the gamelist view with the triggered one.
- **Transitions.** A chosen built-in, unless suppressed, or a chosen theme profile; else, for Automatic, the variant's
  `<transitions>`, then the first profile declared, then instant.

**The output.** `ResolvedTheme` holds the selection, the gamelist's variant after the triggers, the two views, the
sounds, the transition profile, the files read and every diagnostic. Each `ResolvedElement` holds its type, name,
first-definition order, zIndex, the typed values the theme set (`Explicit`), those values plus every documented default
that has one (`Effective`), and its **bindings**: the data the theme does not supply, by name. These are the media an
`imageType` asks for (with `image` expanded to miximage, screenshot, titlescreen and cover), a text's `metadata` or
`systemdata`, a datetime's field, the rating, the badge slots, the help entries, the clock, the system status, the
gamelist info, the gameselector's selection, and the carousel's, grid's or textlist's list of systems or games. A video
element binds `videoFallbackImage`, the image it shows in place of the deferred video (§10.1, Q4). Every path is
absolute and carries whether it exists and whether it was built from a variable.

### 12.3 Error behaviour

`THEMES.md` ("Debugging during theme development") gives three outcomes. An error unthemes the system. An invalid value
is reset to its default with a warning. Numbers out of range are clamped without a word. The loader keeps the
partially parsed views of an unthemed system readable for inspection, and `IsThemed` is false.

| Condition | Severity | Effect | Source |
|---|---|---|---|
| `capabilities.xml` missing or malformed | error | theme not loaded | cited |
| No `theme.xml` for the system | error | unthemed | cited |
| Malformed XML in any file read; a root other than `<theme>` | error | unthemed | cited |
| An unknown tag, element or property | error | unthemed | cited as "enforced more strictly"; the unknown-property case is inferred |
| The legacy `extra` attribute | error | unthemed | cited |
| `<variant>`, `<aspectRatio>` or `<include>` inside `<view>`; a variant in a variant | error | unthemed | cited |
| A view other than `system`, `gamelist` or `all`; a missing `name` | error | unthemed | cited ("mandatory") |
| A property with no value | error | unthemed | cited (the log line in `THEMES.md`) |
| A value in the wrong format (a pair of one number, a five-digit colour, `yes` for a boolean) | error | unthemed | cited ("sanitization for valid data format") |
| An undefined variable in a property | error | unthemed | cited ("a missing variable") |
| A missing include written out | error | unthemed | cited |
| An include loop | error | unthemed | chosen; ES-DE hangs |
| An `imageType` naming an unknown type | error | that element is not rendered | cited |
| An `imageType` repeating a type | warning | the property is ignored | cited |
| An enum value not in its list | warning | default | cited |
| An element or property in a view it does not support | warning | ignored | chosen |
| A second primary element | warning | ignored | chosen |
| A property empty once its variables are substituted | warning | ignored | chosen |
| A missing file written out in a property | warning | kept, `Exists` false | cited |
| An unknown attribute | warning | ignored | chosen |
| Languages declared without `en_US` | warning | languages not loaded | cited; ES-DE logs it as an error, but the theme loads |
| A missing include or file built from a variable | debug | skipped, or kept with `Exists` false | cited |
| A number out of range | none | clamped | cited |

### 12.4 Where `THEMES.md` is silent: the loader's choices

Each of these is a decision, not a finding. Each is to be checked against ES-DE when stage (b) installs it, and each
has a test that fixes the choice, so that a correction shows up as a failing test.

1. **The default variant** is the first selectable one declared, else the first. P18 was written as "the first
   declared", and was refined while building to skip a non-selectable first variant. For Art Book Next the two rules
   give the same answer.
2. **A variant's `selectable` defaults to true.** It is documented as defaulting to true only for transition profiles.
3. **A variable is substituted when it is defined, not when it is used.** `THEMES.md` documents nesting and the global
   namespace, but not whether `<b>${a}</b>` follows a later redefinition of `a`. Only a theme that redefines a variable
   another variable names can tell the difference. Art Book Next does not.
4. **`<variant name="all">` applies even when the theme declares no variants.**
5. **Font size**: `medium` if declared, else the first in the menu order.
6. **Automatic aspect ratio**: the declared ratio nearest the screen's, measured as |log(r₁/r₂)|, so that 2:1 is as far
   from 1:1 as 1:2 is.
7. **Languages.** When capabilities declares none, every `<language>` block is skipped. The one selected is the chosen
   language if declared, else `en_US`. An `<include>` inside a `<language>` block runs in the language pass. §3.2
   predicted this for Art Book Next's metadata files, which carry `<language>` blocks for locales the table in
   `THEMES.md` does not list (`ar_SA`, for instance).
8. **A bare child of `<colorScheme>` or `<fontSize>` is taken as a variable,** with a warning. `THEMES.md`'s own example
   in "Color schemes" writes `<panelColor>` directly inside `<colorScheme>`, although the text says that colour schemes
   hold `<variables>`.
9. **"Can only be used if …" conditions are not enforced.** For example, `verticalAlignment` "can only be used if
   `container` is `false`", and Art Book Next sets both. The loader keeps the typed value, and whether it takes effect
   is the renderer's business.
10. **A path with neither `./` nor `~`** is resolved from the file's folder.

`THEMES.md` also contradicts itself twice. The textlist's `textHorizontalScrolling` is typed BOOLEAN, but its entry
lists "valid values are `vertical` or `horizontal`". The loader types it as a boolean. The language example declares
`pt_PR`, which is not in the language table, and the loader drops it with a warning.

### 12.5 Results on Art Book Next (measured 2026-09-24)

The ES-DE edition was read in place from `~/Projects/art-book-next-es-de-reference` (`d772d07`). Nothing from it was
copied into the repository or into a test fixture.

**Predictions retired.**

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P13 | 172 of 468 pairs set | **172 of 468**. The union was taken over 9 systems × 20 variants × 12 aspect ratios × 2 schemes (`dark-screenshots`, `custom`), 4,320 loads, and its per-element lists equal §3.3's word for word | held exactly |
| P14 | no errors, no unknown elements or properties | **0 errors, 0 warnings, 0 unknown** in 2,700 loads (9 systems × 20 variants × 16:10, 16:9, 4:3 × 5 schemes), and in all 48 aspect-ratio × font-size pairs | held |
| P15 | one silent include skip per load | **exactly one debug note per load, the same include**: `${customizationPath}` undefined in 30 schemes; in `custom`, `theme-customizations/colors.xml` not found. No other include was skipped and no path was missing | held |
| P16 | under 20 ms per system | **0.91 ms per system, both views**, warm, mean of 200 loads across the nine systems. Nine systems take about 8 ms, against P6's 150 ms | held, by a factor of 20 |
| P17 | 12 overrides fall to the basic list, gamelist only | **12 of 12**; each `-nh` variant falls to `gamelist-list-basic-nh` and each other to `gamelist-list-basic`; the system view kept the chosen variant in all 20 | held |
| P18 | the first declared variant by default | **`gamelist-list-metadata-cover`**; 16:10 is automatic at 1280×800; scheme `dark-screenshots`, font size `medium`, transitions `instant` | held for this theme; ES-DE's own rule is still unknown (§12.4, item 1) |

**The inventory.** The tool is `ThemeInventoryTool` in WiseMan, run as
`EMUSEN_THEME_INSPECT=<theme folder> EMUSEN_THEME_CHOICES="system=snes;aspect=16:10;scheme=…;variant=…" dotnet test
--filter ThemeInventoryTool`. It prints every element in drawing order, with its bindings and every property the theme
set as a typed value (with `defaults` in the choices, the defaults too), then what is unknown, what is unsupported, and
every diagnostic. It was run for `snes` over 16:10, 16:9 and 4:3, four schemes (`dark-screenshots`, `light-noir`,
`snes-outline`, `custom`) and four variants, 48 runs:

| Variant | Files read | System view | Gamelist view | Properties set (16:10 / 16:9 / 4:3) | Unsupported |
|---|---|---|---|---|---|
| `gamelist-list-metadata-cover` | 5 | 8 elements | 28 elements | 332 / 331 / 322 | `video.delay`, `iterationCount`, `onIterationsDone`, `pillarboxes` |
| `gamelist-list-screenshot-marquee` | 5 | 8 | 13 | 206 | the same four |
| `gamelist-list-basic-nh` | 5 | 8 | 11 | 174 | none |
| `gamelist-grid-cover` | 6 (adds `_coversize/4-3.xml`) | 8 | 12 | 206 | none |

- **The colour scheme changes no count.** All four schemes give the same elements and the same properties, and differ
  only in values. That is what §3.2 implies, since Art Book Next's schemes hold variables only.
- **4:3 sets fewer properties than 16:10.** It leaves `pos` and `size` unset on three of the hidden metadata icons, and
  `fontSize` on two hidden texts. Those elements are `visible` false in that variant, so nothing is lost.
- **The only unsupported properties are the four video playback ones** the theme sets. §10.1 deferred them. The video
  element `game-art` resolves to `videoFallbackImage:cover` in the metadata-and-boxart variant, as §4.7 requires.
- **Every path resolved to a file that exists** in all 48 runs, fonts, SVGs and PNGs included, and all seven sounds.

**Negative results.** No property of the theme needed a catalogue change, no element or property was unknown, and no
choice was ambiguous. The run therefore tested the loader's handling of what Art Book Next uses and not of the other 296
pairs. Those rest on the synthetic tests of §12.6 alone, which have no oracle but `THEMES.md`.

### 12.6 Tests and mutants

**Tests.** 154 in `EmuSen.WiseMan/Mistress/BigPicture/`, plus the inventory tool, all on themes written by the tests
through `SyntheticTheme` (`EmuSen.WiseMan/Fixtures/`), except the five reference tests. Theory cases count one each.

| Class | Tests | Covers |
|---|---|---|
| `ThemeCapabilitiesTests` | 28 | labels, overrides, fallbacks, table orders, duplicates and reserved names, languages without `en_US`, transition defaults, the default variant, scheme, font size and language, automatic aspect ratio (7 screens), triggers (precedence, one step, gamelist only), transitions resolution |
| `ThemeLoaderTests` | 36 | `THEMES.md`'s own variables example, all nine parsing steps in one file written in reverse order, includes at their own place, scheme order, scheme and font-size blocks, bare scheme children, languages, nested and eagerly substituted variables, the twelve system variables, include paths from the including file, the system folder, `~` and backslashes, missing and variable includes, Art Book Next's undefined-variable include, loops, a file included twice, misplaced blocks, `all`, name lists, undeclared names, merging, views and name lists, sounds, drawing order |
| `ThemeErrorTests` | 32 | every row of §12.3, and a Batocera-format theme refused |
| `ThemeCatalogTests` | 53 | the counts per element, 39 entries checked against `THEMES.md` by hand, views and zIndex, **all 468 properties set and read back typed**, **all 336 literal defaults**, same-as and computed defaults, colours, booleans, clamping, zero and `-1` axes, list splitting, bindings, the deferred set |
| `ArtBookNextReferenceTests` | 5 | §12.5; each skips with its reason when the clone is absent (checked by pointing `EMUSEN_ARTBOOKNEXT` at a missing folder: 5 skipped, reason shown) |

**Mutants.** 28, applied one at a time by `~/.cache/emusen/probe/bigpicture/mutate_loader.py` (outside the repository,
like the other workbench runners), each built and run against the BigPicture tests, the source restored after each and the tree
rebuilt clean at the end. **All 28 were caught.**

| # | Mutant | Caught by (synthetic / reference tests) |
|---|---|---|
| M1 | variables stored unsubstituted, so nesting fails | 4 / 0 |
| M2 | include paths resolved from the entry file, not the including file | 1 / 1 |
| M3 | colour-scheme blocks applied in reverse file order | 1 / 0 |
| M4 | Automatic takes the first declared ratio, ignoring the screen | 1 / 1 |
| M5 | colour schemes before plain variables | 2 / 0 |
| M6 | variants before general configuration | 4 / 0 |
| M7 | aspect ratios before variants | 3 / 0 |
| M8 | includes before colour schemes and font sizes | 2 / 0 |
| M9 | an include naming an undefined variable is an error | 1 / 3 |
| M10 | a missing explicit include is skipped | 1 / 0 |
| M11 | an undefined variable in a property is kept literally | 1 / 0 |
| M12 | include loops not detected | test host crashed (stack overflow) |
| M13 | `noVideos` before `noMedia` | 2 / 0 |
| M14 | triggers two steps deep | 2 / 0 |
| M15 | the triggered variant used for the system view | 1 / 0 |
| M16 | `all` not applied | 2 / 0 |
| M17 | merging keeps the first value | 3 / 0 |
| M18 | zIndex ties by last definition | 1 / 0 |
| M19 | six-digit colours transparent | 13 / 0 |
| M20 | floats not clamped | 1 / 0 |
| M21 | an invalid `imageType` does not stop the element | 1 / 0 |
| M22 | view-limited properties accepted in the other view | 1 / 0 |
| M23 | a second primary element kept | 1 / 0 |
| M24 | same-as defaults ignored | 1 / 0 |
| M25 | kind-specific system variables filled for every kind | 1 / 0 |
| M26 | language blocks applied with no language declared | 1 / 0 |
| M27 | start-up transitions default to instant | 1 / 0 |
| M28 | a zero size axis clamped | 1 / 0 |

**What the mutants say about Art Book Next as an oracle.** The reference tests caught three of the 28: M2, M4 and M9.
M12 crashed the whole run, so it says nothing either way. Under the other 24 broken loaders Art Book Next still loads
with no error and its five tests pass, because it does not nest variables across redefinitions, never lets two scheme blocks set one variable, and never relies
on the parsing order beyond `colors.xml`'s include. A theme that loads is weak evidence that a loader is right. The
synthetic tests carry the grammar.

### 12.7 Not done in stage (a)

- **Nothing was checked against ES-DE itself.** ES-DE is not installed (§7). Every choice of §12.4, and the claim that
  the unknown-property case is an error, wait for stage (b).
- **The "can only be used if" conditions** are not enforced (§12.4, item 9). The renderer decides.
- **No drawing, no LunaP controls, no ScreenScraper, no UI.** Those are stages (b) to (f).
- **No settings persistence.** `ThemeChoices` is a record, and nothing yet stores it under `BigPicture` in
  `appsettings.json` (§6).
- **Media presence for the triggers is supplied by the caller.** Nothing scans a media folder yet; stage (f) connects the
  triggers to the media store.
- **Only one theme was run.** The other 65 themes of ES-DE's list were not loaded. They would test the 296 pairs Art Book
  Next does not use.
- **`gameselector` links** (an element's `gameselector` property naming a gameselector element) are kept as strings and
  not checked against the view's gameselectors.

---

## 13. Stage (b): the two views drawn statically, with LunaP controls

*Opened 2026-09-24.* Stage (b) draws a `ResolvedView` (§12) at rest: the system view with its carousel, and a gamelist
view with its list, media and metadata. Under §10.1 every visible part is a LunaP control, and what LunaP lacked was
added to LunaP (its `docs/LunaP.md` §98 onwards). The mapping from ES-DE properties to control properties lives in
Mistress, in `EmuSen.Mistress/BigPicture/Scene/`, and is a pure function of the view and the data. Nothing moves;
time is stage (c).

ES-DE was not available when the stage opened. It became available part way through (the user downloaded the
AppImage), and is run only from a copy under `~/.cache/emusen/bigpicture/esde/`, with its own `--home` there.

### 13.1 Predictions, written before the controls were built

The inventory of §3.3 gives the 172 (element, property) pairs Art Book Next sets. By element: carousel 17, textlist
16, grid 24, image 13, video 14, text 20, datetime 11, rating 8, badges 12, helpsystem 14, clock 10, systemstatus 12,
sound 1.

- **P19, frame cost.** With every image decoded and cached, one steady headless render of either view (layout and
  Skia's CPU raster, as `Avalonia.Headless` does it, not the GPU path P7 prices) costs **under 8 ms at 1280×800** and
  under 2.25 times that at 1920×1200. The first build of a view, decoding, SVG rasterising and shaping included, costs
  **under 250 ms**.
- **P20, SVG against `Svg.Skia`.** Of Art Book Next's 241 SVG files, our renderer refuses **exactly the six** that hold
  `text`, `filter` or `script` (`coco`, `emulators`, `epic`, `symbian`, `vpinball`, `windows3x`), and draws the other
  235 at an intersection-over-union of coverage of **at least 0.98 each at 256 px**, the median above 0.995. This is
  P2 with its population named: P2 was written for §3.5's 36 files, which are a subset.
- **P21, properties mapped.** Without the grid (stage f, §7), **137 of the 172** pairs change what is drawn. The 35
  left are the grid's 24; the carousel's `itemTransitions` and `fastScrolling`, the text's `containerStartDelay` and
  `containerVerticalSnap`, and the video's `delay`, `iterationCount`, `onIterationsDone` and `pillarboxes`, all of
  which are about time or playback; the sound's `path`; and the badges' `controllerSize` and `folderLinkSize`, whose
  data (the controller badge and the folder link) §3.8 defers. If the grid is built, 161.
- **P22, geometry from the theme.** Every `pos`, `size` and `maxSize` in `aspect-ratio-16-10.xml` that carries a pixel
  comment of its 768×480 design gives the box the scene lays out, to within 1 px at 1280×800 and 1920×1200, for the
  elements drawn in the views tested.
- **P23, the font size.** `fontSize` × screen height is the em size handed to the rasteriser, not the height of a
  rasterised 'S'. §4.4 left this open. It decides every text box's height, and it can only be told from ES-DE.

P1 (text heights against ES-DE within 2 px), P3 (the SVG renderer against LunaSVG, 0.97) and P12 (the video element
identical to ES-DE's without videos) are this stage's too, and need ES-DE.

### 13.2 What was built

**In LunaP** (branch `bigpicture-controls`; its `docs/LunaP.md` §98–§101.8). None of these controls knows about ES-DE:
- `NormalizedCanvas`, which places children by fractions of itself;
- `FittedImage`, which fills, contains, covers or tiles an image, tints it by a colour or a gradient, and sets its
  saturation, corner radius and sampling;
- `FontText`, with `FontFiles`, which reads a typeface from a file once per path and never registers it with
  Avalonia's font manager;
- `SvgDocument` and `SvgPicture`, which draw the SVG subset of §4.5 and refuse a whole file that needs more;
- `TextRowList` and `ImageCarousel`;
- `StarRating`, `BadgeStrip`, `HintBar`, `ClockLabel` and `DeviceStatusBar`.

LunaP still references Avalonia and nothing else. The image effects are applied on the CPU once per picture, size
and effect, because a Skia lease would have broken that rule (LunaP §98.2).

**In Mistress**, in `BigPicture/Scene/`:
- `SceneBuilder.Build(view, data)` is a pure function of a `ResolvedView` and a `SceneData`. `SceneData` holds the
  systems with their resolved themes, the games, the selection, a media source, the time and the device's status.
- One file per group of elements holds the mapping.
- `SceneMapping` lists every pair the scene reads.
- `EsdeMediaFolder` reads an ES-DE `downloaded_media` tree in place.
- `HelpPrompts` holds Mistress's own help words and glyphs (§3.6, Q9 still open).

**In WiseMan:**
- `SyntheticLibrary` writes the shared inputs: invented games with invented metadata, flat labelled PNGs of known
  sizes, empty ROM files and ES-DE `gamelist.xml` files. No real art is used.
- `SceneAssets` holds the test pictures, an icon and a font.
- The tools render PNGs and time frames (`SceneRenderTool`, `SceneFrameBench`), write the ES-DE inputs
  (`EsdeInputsTool`), and render Mistress in the state an ES-DE capture was taken in (`EsdeCompareTool`).

### 13.3 What is mapped: P21

- **Of the 172 pairs Art Book Next sets, the scene reads 136.** The prediction was 137.
- **The one it lost after the prediction is `video.interpolation`.** Against ES-DE, the static image a video element
  shows in the video's place is filtered linearly even though the theme sets `nearest` (§13.8). The property is taken
  to govern the video alone, which §10.1 defers.
- **The other 35 are those §13.1 named:**
  - the grid's 24 (stage f);
  - the carousel's `itemTransitions` and `fastScrolling`, and the text's `containerStartDelay` and
    `containerVerticalSnap`, which are about time (stage c);
  - the video's `delay`, `iterationCount`, `onIterationsDone` and `pillarboxes`, which are about playback (stage g);
  - the sound's `path` (stage e);
  - the badges' `controllerSize` and `folderLinkSize`, whose data §3.8 defers.
- **Beyond Art Book Next the scene reads 98 more pairs,** the ones that serve the same elements.
- **The claim is checked, not asserted.** `SceneMappingTests` builds a synthetic theme for every pair `SceneMapping`
  lists, renders it with two values and requires the pixels to differ: 234 cases. A second test requires the case list
  and the mapping's list to be the same set.

### 13.4 Frame cost: P19, and what the harness costs

`SceneFrameBench` times 30 headless frames after 3 warm-up frames, and reports the median. For every configuration it
also counts the pixels that differ from a full HighQuality redraw.

| 1280×800 / 1920×1200 | System view | Gamelist view |
|---|---|---|
| Harness: empty window, full redraw | 5.9 / 14.7 ms | the same |
| Harness: empty window, nothing invalidated (readback only) | 1.1 / 3.0 ms | the same |
| Full redraw, every visual invalidated | **44.1 / 87.4 ms** | **15.2 / 26.2 ms** |
| Unchanged view, nothing invalidated | **1.1 / 2.7 ms**, pixels identical | **1.2 / 3.0 ms**, identical |
| Only the clock invalidated (a minute's tick) | 5.8 / 14.4 ms | — |
| First build and first frame | 513 / 194 ms | 58 / 65 ms |

- **P19 fails for a full redraw and holds at rest.** At rest the controls cost nothing: the compositor does not
  redraw an unchanged view, and what remains is the harness's readback. A full redraw of the system view costs 38 ms
  of controls at 1280×800 (44.1 − 5.9), against a prediction of under 8 ms. The first build costs 513 ms against a
  prediction of under 250. That run is also the process's first decode, first SVG rasterisation and first JIT; the
  second size takes 194 ms.
- **Where the time goes.** It is Skia's mipmapped sampling of the carousel's downscaled artwork, measured three ways:
  - MediumQuality gives pixels and times identical to HighQuality when shrinking, so HighQuality already samples
    mipmapped there;
  - LowQuality takes 10.0 ms;
  - invalidating only the carousel costs as much as invalidating everything.
- **The user chose full quality (2026-09-25)**, so only levers whose pixels equal a HighQuality redraw were eligible.
  None helped:

  | Lever | Result |
  |---|---|
  | An immutable bitmap instead of a `WriteableBitmap` | 43.35 ms against 43.60, so Skia is not rebuilding mips per frame |
  | Unfocused opacity carried in the tint instead of a layer | no measurable change |
  | `BitmapCache` on the carousel | no saving; pixels changed by up to 131 |
  | Not redrawing an unchanged view | already what the compositor does |

  One rejected change is recorded so it is not proposed again. Drawing bitmaps prepared at display size 1:1 on whole
  pixels took the system view to ~10 ms, but it resamples once at a different quality. The user rejected it.
- **The gamelist rose from 7.8 to 15.2 ms at 1280×800 during the stage.** That is the cost of drawing the video's
  static image linearly, as ES-DE does (§13.8), where the theme's `nearest` had been cheap.
- **What was not measured.** Headless Skia renders on the CPU; the application renders on the GPU, where this sampling
  is cheap. The GPU frame, and anything on the Legion Go S, is P7's and stage (e)'s.

### 13.5 Tests

| Class | Tests | What it holds |
|---|---|---|
| `SceneMappingTests` | 2 (234 cases in one) | every mapped pair changes the pixels; the case list equals the mapping's list |
| `SceneSemanticsTests` | 15 | what a property maps to: the zIndex order; corner radii and horizontal paddings by the width, font sizes and vertical paddings by the height; the outward background of §13.8; `size` over `maxSize` and a video's image sizes over its own; a container's missing ellipsis; strftime; badge slots; status entries; the name suffix for collections only |
| `SceneReferenceTests` | 2 | P21 and P22 on Art Book Next, skipping visibly without it |
| `SvgOracleTests` | 1 | P20 on Art Book Next |

**A defect the semantics tests found.** `SceneBuilder.Place` replaced a control's size with the element's own
`size` whenever the control's size equalled (0, 0). A fitted image sets (0, 0) on purpose, so a video with both
`size` and `imageMaxSize` was drawn at its `size`. `Place` now asks whether the size was set. Art Book Next's
`game-art` sets no `size`, which is why its reference tests passed.

### 13.6 Geometry from the theme: P22

Art Book Next's `aspect-ratio-16-10.xml` writes 38 of its values beside a comment giving the value in pixels of a
768×480 design. `SceneReferenceTests` reads them in place and lays out every variant's two views at 1280×800 and
1920×1200. It then checks the box of every element that uses a commented value: 644 checks.

- **P22 holds. The worst error is 0.6 px.**
- **One comment disagrees with its own fraction.** The grid variants' logo writes `0.05` beside "22", and 0.05 × 480
  is 24. The engine lays out by the fraction, so the check uses the fraction there and reports the disagreement.
- **The 0.6 px** is `game-name`'s `0.062` against its comment's 30/480 = 0.0625.
- **Two corrections came out of this check.**
  - The theme's `0.41666667`, read as a float, is 800.00064 px of 1920, and Avalonia's layout rounding took the
    ceiling of 801. LunaP now keeps computed positions and sizes to hundredths (its §101.7).
  - The scene then turned layout rounding off altogether, because ES-DE places at fractional pixels (§13.8). That took
    the worst error from 1.0 to 0.6 px.

### 13.7 SVG against `Svg.Skia`: P2 and P20

- **The setup.** `Svg.Skia` 5.1.1 is the oracle. It is MS-PL through `Svg.Custom`, so it is referenced by WiseMan
  only, which is never distributed, and never by LunaP or Mistress (§4.5).
- **The metric.** Both renderers draw every SVG of Art Book Next with the same 256-pixel `width` and `height` written
  into the markup. The score is the IoU of coverage, weighting each pixel by its alpha.
- **Refused: 7 of 241.** The prediction was 6. It missed `lowresnx.svg`, which puts a `<g>` inside a `clipPath`, where
  SVG 1.1 allows only shapes.
- **Drawn: 234,** at IoU **0.9959 at worst, median 1.0000,** none below 0.98. The largest mean colour difference is
  1.02 of 255. P20 holds for the drawn files, and P2 holds.
- **A negative result on method.** The first comparison left each renderer to infer its own viewport, and reported
  IoUs of 0 on four files and under 0.98 on 24. Every one of those was the harness (LunaP §99.5).

### 13.8 Against ES-DE 3.4.1: P1, P3, P4, P12 and P23

**The setup.**
- ES-DE 3.4.1 (r51) ran from a copy of the user's AppImage, with checksum 3c61a44d…3581, under
  `~/.cache/emusen/bigpicture/esde/`.
- Every run used `--home` in that folder, at `--resolution 1280 800` and `1920 1200` with `--fullscreen-padding off`.
- Settings were written into the scratch home's `es_settings.xml`:

  | Setting | Value |
  |---|---|
  | ROM and media directories | the synthetic library under `~/.cache/emusen/bigpicture/` |
  | Theme | Art Book Next, linked into the scratch themes folder |
  | Variant | `gamelist-list-metadata-cover` |
  | Colour scheme | `dark-screenshots` |
  | Aspect ratio | 16:10 |
  | Startup system | `snes` |
  | Startup view | system or gamelist |
  | `DisplayClock` | true |

- Runs lasted 5–14 s. Each window was captured with `spectacle`, and ES-DE was then closed by PID.
- Mistress rendered the same state: ES-DE's system order, its game order, the first game selected, the captured
  clock's time, and Bluetooth only.
- The client area was located in each capture by the crop origin that best aligned the logo: (82, 101) at 1280×800
  and (81, 100) at 1920×1200. Captures and scripts stay under `~/.cache/emusen/bigpicture/`, and no test depends on
  them.

**What ES-DE showed that the scene then followed:**
- **The carousel fills its row with repeats.** With five systems in a row that holds about seven, the item after the
  last is the first again (LunaP §101.7).
- **The help bar's, clock's and status's backgrounds grow outward from the positioned box.** The clock's box started
  at `pos` − padding: 26.6 across (38.3 − 11.7) and 25 down (38.3 − 13.3). The status's right edge was at
  `pos` + padding.
- **Favourites come first in the list, each marked with a star.** Names sit centred in a 44 px band at the top of each
  58.33 px pitch, and the selected background is as wide as the name plus the margins (LunaP §101.8). The documented
  default for that band is 1.5 × `fontSize` = 45 px, 1 px more than measured.
- **The video element's static image is filtered linearly,** although the theme sets `interpolation nearest`: the
  cover's white border is blended at both edges in ES-DE.
- **ES-DE places at fractional pixels.** Turning off the scene's layout rounding took the cover's mean difference
  from 1.75 to 1.51 levels.
- **The clock is off by default,** by the setting `DisplayClock` false, whatever the theme sets. Mistress will need
  that setting.

**Measured after those changes:**

| | 1280×800 | 1920×1200 |
|---|---|---|
| List: selected background, ES-DE / Mistress | x 47–278, y 210–251 / x 46–279, y 208–252 | — |
| List: name ink | within 1 px across and down | — |
| Metadata text ink (description, date, players, play time) | within 1–2 px | within 1–2 px |
| Carousel: settled slice edges | within 0.5–1 px | within 0.5–1 px |
| Cover (P12): mean difference; pixels differing by more than 8 | 1.51 levels; 4.2% | 4.32 levels; 4.6% |
| Logo IoU (P3) | 0.922 | 0.944 |
| Favourite badge IoU | 0.899 | 0.836 |
| Rating stars IoU | 0.712 | 0.492 |
| Metadata icons IoU (30 px thin strokes at 0x33 alpha) | 0.595–0.616 | 0.429–0.502 |
| System view: pixels differing by more than 8 | 11.9% | 13.6% |

**The predictions against these numbers:**
- **P1 holds.** Every text measured agrees with ES-DE to within 2 px, and the list's names to within 1.
- **P23 holds.** The names' ink heights agree when the font size is the em size handed to the rasteriser, so
  `fontSize` × height is that em size, not the height of a rasterised 'S'.
- **P4, its settled half, holds** to 1 px. The durations are stage (c).
- **P12 does not hold as "identical".** The flat colour inside the cover is equal, and 95.8% of pixels are within 8
  levels. The rest lie on the label's glyph edges and the border, where the two engines' filters differ.
- **P3 fails.** The large logo reaches 0.92–0.94, and the thin 30-pixel icons 0.43–0.62. Their ink boxes agree to
  1–2 px, and ES-DE draws each icon about 6% smaller inside the same 40-pixel box: 29 × 30 against 30 × 32.
  Mistress's size follows the viewBox mapping exactly. ES-DE's cannot be explained without reading its source, which
  is out of bounds, so the difference is recorded, not copied. With such thin, faint strokes a 1 px difference halves
  the overlap, so this IoU measures placement as much as rasterisation. P3's threshold was the wrong instrument for
  icons of this size.
- **The system view's remaining difference is image softness.** ES-DE's pixel-art slices are visibly softer, as if
  resampled twice. Mistress's are sharper in all four of Avalonia's sampling modes (within 0.4 points of each other).
  The best alignment between the two images is a zero shift, so the geometry agrees. Matching the softness would mean
  blurring deliberately, which the user's full-quality rule excludes.
- **The help bar's words and icons are ES-DE's own** ("MENU, SELECT, SCREENSAVER, CHOOSE"), so they differ by design
  (§3.6).

### 13.9 Mutants

The runners are `~/.cache/emusen/probe/bigpicture/mutate_lunap.py` and `mutate_scene.py`, outside the repositories.
Each builds, runs its tests and restores the source, and the scene runner rebuilds clean at the end.

- **LunaP: 34 mutants, all caught** (its §101.6 and §101.7). Two survived their first run, on weak tests (CSS
  specificity, the rating's cut), and were caught once the tests were strengthened.
- **The scene: 25 mutants, all caught.** A 26th, pixels kept to hundredths, was retired with the code it tested, which
  §13.6's switch to fractional placement made dead.
  - **13 survived the first run.** The differential test proves that each property has an effect, not the right one:
    the drawing order; the axis of corner radius, font size and vertical padding; size against maxSize; a video's own
    image sizes; the container's ellipsis; strftime's `%m`; which badges show; the battery entry; the outward padding;
    the name suffix on regular systems.
  - **All 13 were caught once `SceneSemanticsTests` existed,** and writing those tests found the defect of §13.5.
  - **The reference tests caught one mutant alone, S1 (origin ignored).** As in stage (a), a real theme that renders is
    weak evidence that the mapping is right.

### 13.10 Predictions retired

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P1 | text boxes within 2 px of ES-DE at 1280×800 | list names within 1 px; metadata texts within 1–2 px at both sizes | held |
| P2 | IoU ≥ 0.98 against `Svg.Skia` on §3.5's files | 0.9959 or better on all 234 drawn | held |
| P3 | IoU ≥ 0.97 against LunaSVG in ES-DE captures | logo 0.92–0.94; thin icons 0.43–0.62, ES-DE's ~6% smaller in the same box | failed; the instrument recorded as unfit for thin icons |
| P4 | settled positions equal to ES-DE's to 1 px | slice edges within 0.5–1 px; durations untested | the settled half held |
| P12 | the video element's render identical to ES-DE's | same colours and box; 4.2–4.6% of pixels differ on edges, and ES-DE filters linearly despite `nearest` | failed as "identical" |
| P19 | steady frame under 8 ms headless at 1280×800; first build under 250 ms | at rest 1.1 ms (harness readback); full redraw 44 ms system, 15 ms gamelist; first build 513 ms | failed for a full redraw; held at rest |
| P20 | exactly six refused; the rest at 0.98 or more | seven refused (`lowresnx`); the rest 0.9959 or more | half held |
| P21 | 137 of 172 mapped | 136, after `video.interpolation` was dropped on ES-DE's evidence | held, less the one |
| P22 | commented geometry within 1 px | 644 checks, worst 0.6 px; one comment disagrees with its fraction | held |
| P23 | `fontSize` × height is the em size | name ink agrees with ES-DE to 1 px on that reading | held |

### 13.11 Not done in stage (b)

- **No grid** (stage f), so none of its 24 pairs is mapped.
- **Nothing moves** (stage c). That covers the carousel's slide, the list's scroll, the text containers' scrolling and
  the delays, and P4's durations.
- **The ES-DE comparison ran on one variant and one colour scheme**, at two sizes. The other 19 variants and 30
  schemes were laid out (§13.6), not captured.
- **The scene is not in Mistress's window.** Nothing shows it outside the tests; that is stage (e).
- **Settings the scene will need** are not carried in `SceneData`: `DisplayClock`, favourites-first ordering (the host
  orders the games for now) and ES-DE's hidden-metadata switch.
- **P3's icons are not resolved.** Why ES-DE draws a 40-pixel icon 6% smaller is not known.
- **P19 on the GPU and on the handheld** is not measured.
- **The `indicators` value `ascii`** is drawn as the symbols, unverified.
- **No capture of a collection, a folder, the grid or the other help scopes** was taken.
- **§12.4's ten loader choices were not checked against ES-DE.** The default variant, for one, was set explicitly in
  every run.

---

## 14. Stage (c): the GPU frame, then motion

*Opened 2026-09-25.* §13.4 measured a full redraw of the system view at 44 ms on the CPU, in `Avalonia.Headless`, and
left the GPU frame unmeasured. Stage (c) redraws every frame while something moves, so that number decides how motion
is built. This section first measures the frame on a GPU (§14.1–§14.4), then builds motion (§14.5 onwards).

### 14.1 Predictions for the GPU frame, written before it was measured

The route (§14.2) draws the scene's LunaP control tree with Avalonia's own Skia drawing code onto a `GRContext` over a
surfaceless EGL context, the device and API a Mistress window on Linux uses. Stage (b)'s numbers are the baseline:
full redraw on the CPU 44.1 ms (system) and 15.2 ms (gamelist) at 1280×800, 87.4 and 26.2 ms at 1920×1200, most of it
mipmapped sampling of downscaled pictures (§13.4).

- **P24, the route is faithful.** Its picture equals the headless compositor's CPU picture except where the two
  rasterisers anti-alias or filter differently: at least 99% of pixels within 8 levels in both views at both sizes.
- **P25, the desktop.** On the RX 6800, a full redraw of either view takes **under 3 ms** from the start of recording
  to `glFinish` at 1280×800, and under 4 ms at 1920×1200. The CPU's share (the controls' `Render`, Avalonia's drawing
  code and Skia's GPU recording) is the larger part, about 1.5–2.5 ms; the GL time is under 1 ms, because trilinear
  sampling is what the hardware does natively.
- **P26, the first frame.** The first GPU frame of a view, which uploads every picture and builds its mip levels,
  costs under 50 ms on the desktop.
- **P27, no per-frame upload.** The processed pictures are `WriteableBitmap`s. Skia uploads each once and reuses the
  texture while the bitmap is unchanged, so replacing them with immutable bitmaps changes the GPU frame by under 5%.
- **P28, the handheld** (Legion Go S at 1280×800). A full redraw of the system view takes **under 8 ms** (P7's
  figure), so a frame in which everything moves fits 16.7 ms with room to spare, and stage (c) can be built on full
  redraws with no lever. The CPU's share is about twice the desktop's.

### 14.2 The route, and whether it is faithful

`SceneGpuBench` (WiseMan) builds a view exactly as §13's tools do, shows it in a headless window so that layout runs,
and then draws the window three ways:

1. through the headless compositor on the CPU, every visual invalidated, as §13.4 measured;
2. with `DrawingContextHelper.RenderAsync`, Avalonia's public entry for drawing a visual tree onto any `SKCanvas`,
   onto a raster surface;
3. with the same call onto a GPU surface, `SKSurface.Create(GRContext, …)`. The `GRContext` is Skia's GL backend over
   a surfaceless EGL context on a named device. `ShaderBench` already opens that context for Serenity's measurements
   (`EmuSen_Serenity.md` §8.1). A Mistress window on Linux draws through Avalonia's GLX context, which is the same
   driver (radeonsi) and the same Skia backend.

Routes 2 and 3 use Avalonia's own `DrawingContextImpl`, so every `DrawImage`, sampling option, layer and glyph run is
issued by the code a window runs. They differ from a window in one respect: a window records on the UI thread and
replays on the render thread, and the route does both on one thread. The bench therefore also times the controls'
`Render` alone, into Skia's no-draw canvas, which is the UI thread's share.

Each frame is timed from the start of recording to `glFinish`, with a GL timer query around the GPU work. Frames are
medians of 120 after five warm-up frames. The GPU frame's pixels are read back and compared with the other two.

**Faithfulness (measured 2026-09-25, RX 6800).**

| | System 1280×800 | Gamelist 1280×800 | System 1920×1200 | Gamelist 1920×1200 |
|---|---|---|---|---|
| GPU against CPU route: pixels over 8 levels apart, largest difference | 0.025%, 35 | 0.069%, 47 | 0.017%, 40 | 0.064%, 50 |
| GPU against compositor | 0.27%, 142 | 1.86%, 140 | 0.17%, 144 | 1.21%, 140 |
| CPU route against compositor | 0.24%, 142 | 1.80%, 139 | 0.16%, 143 | 1.15%, 139 |

- **The GPU rasterises the scene as the CPU does.** 99.93% or more of pixels agree to 8 levels. The rest lie on the
  edges of slices, glyphs and the cover. `diff-gpu` images under `~/.cache/emusen/bigpicture/gpu/` show them.
- **The larger difference against the compositor belongs to the route, not to the GPU,** because the CPU route shows
  it too. It lies entirely on glyph edges. The headless compositor draws text with subpixel (LCD) anti-aliasing: 353
  colour-fringed pixels in the help bar's words, against none from `RenderAsync`, which draws text in grey levels. Text
  is a small share of any frame's cost, so this does not bear on the timings.

### 14.3 The desktop's GPU frame: P24–P27

`SceneGpuBenchTool`, run as `EMUSEN_BIGPICTURE_GPU=1 dotnet test --filter SceneGpuBenchTool`. All times are in ms,
medians of 120 frames.

| | System 1280×800 | Gamelist 1280×800 | System 1920×1200 | Gamelist 1920×1200 |
|---|---|---|---|---|
| CPU raster, route 2 (stage b's cost) | 45.6 | 20.6 | 89.8 | 38.1 |
| Controls' `Render` alone (no-draw canvas) | 0.04–0.24 | 0.11–0.13 | 0.05 | 0.12–0.15 |
| GPU: recording and flush on the CPU | 0.16–0.20 | 0.31–0.42 | 0.19 | 0.32–0.46 |
| GPU: GL timer | 0.03 | 0.04 | 0.16 | 0.06 |
| **GPU: start to `glFinish`** | **0.33–0.35** | **0.46–0.60** | **0.46–0.47** | **0.50–0.71** |
| GPU: first frame (upload and mip levels) | 4–22 | 3–11 | 5–8 | 4–6 |

The ranges span the two variants and the runs.

- **The GPU frame is 130 times cheaper than the CPU's.** The system view's 45.6 ms is 0.35 ms on the GPU. Mipmapped
  sampling, which §13.4 found to be the CPU's whole cost, is what texture hardware does for free.
- **P24 fails as written, and the failure is the route's text, not the GPU.** The prediction compared against the
  compositor, and the gamelist has 1.86% and 1.21% of its pixels over 8 levels from it, against a bound of 1%. Against
  the CPU route, whose text is drawn the same way, the GPU is within 8 levels on 99.93% or more (§14.2).
- **P25 holds, and its split was wrong.** Every frame is under 0.75 ms, against bounds of 3 and 4. The prediction
  gave the CPU's share as 1.5–2.5 ms, and it is 0.16–0.46 ms; the GL time is under 0.2 ms.
- **P26 holds.** The first frame, which uploads every picture and builds its mip levels, takes 3–22 ms.
- **P27 holds.** Handing Skia immutable bitmaps of the same premultiplied pixels gives an identical picture and the
  same frame time, within run-to-run spread: 0.33 against 0.33 ms, 0.46 against 0.60. Skia uploads a `WriteableBitmap`
  once and reuses its texture while the bitmap is unchanged.

**What this settles for stage (c).** On the desktop a frame in which everything moves costs under a millisecond,
about 1/25 of a 60 Hz frame, so motion can be built on plain full redraws. No lever of §14's brief (static layers
composed once, finished carousel items cached, mip levels kept on the GPU) has anything to save here: the last
already happens, and the first two would save part of a fraction of a millisecond.

### 14.4 The handheld bench: P28

The bench runs as a self-contained program, so the handheld needs neither the SDK nor the repository. It is
`~/.cache/emusen/probe/bigpicture/deck-gpu/`, outside the repository, with a README. It holds:

- `out/`, a linux-x64 publish of a console host that compiles `SceneGpuBench.cs`, `SyntheticLibrary.cs` and
  `ShaderBench.cs`, as the shader bench's host does (§8.4 of `EmuSen_Serenity.md`); its natives ask for glibc 2.38 at
  most;
- `theme/`, a private copy of Art Book Next;
- `run.sh`, which runs three rounds and writes the results with the CPU, the power state and whether gamescope runs.

The device ends processes an ssh session leaves behind, so the README starts the bench as a transient user unit:
`systemd-run --user --unit=emusen-bigpicture-gpu --collect /bin/bash "$HOME/emusen-bench/bigpicture-gpu/run.sh" 3`.
It needs no window and no display server: EGL's device platform opens the render node, so it runs the same in Game
Mode, where gamescope's compositing shares the GPU with it.

The bench was republished at the end of the stage with the moving cases of §14.8 (the carousel held, the list held
with and without its lever), so one run answers P8 and P28 together.

**P28 is not retired.** It waits for the handheld's results. Until then, §14.3 is the design's evidence. A handheld
twenty times slower than the RX 6800 on both the CPU and GPU shares would still draw the system view in about 7 ms.

### 14.5 Predictions for motion, written before ES-DE was recorded

`THEMES.md` gives none of the durations, speeds or curves below. These were written from memory of using ES-DE and
from the property defaults, before any recording, and are retired in §14.10.

- **P29, the carousel step.** One step settles in 150–400 ms (P4's range), decelerating (an ease-out), and takes the
  same time whatever the distance still to go. The unfocused opacity and the scale follow the same curve as the slide.
- **P30, key repeat.** Holding a direction repeats after 400–500 ms, then every 60–150 ms. `fastScrolling` adds a
  faster tier after about a second or two of holding.
- **P31, the text list.** Moving the selection by one row does not animate: the selector and the rows jump.
- **P32, the selected name's horizontal scroll.** It starts `textHorizontalScrollDelay` (3 s) after the selection
  settles, moves at a constant speed of 50–150 px/s at 1280×800 for the default speed of 1, and repeats with a gap.
- **P33, the vertical text container.** It starts after `containerStartDelay` (Art Book Next: 6 s), moves at a
  constant 20–40 px/s at 1280×800, stops when the last line shows, waits `containerResetDelay` (7 s), then returns to
  the top with a fade and starts again.
- **P34, `scrollFadeIn`.** A game's image fades in over 150–300 ms when the selection changes.
- **P35, the video element without a video file.** The static image appears with the selection, with no delay and no
  fade.
- **P36, the slide transition** between the system and gamelist views takes 200–500 ms, decelerating.
- **P37, settled frames.** When a motion ends, Mistress's frame equals its static render of the new state pixel for
  pixel, and ES-DE's settled positions to 1 px.
- **P38, the moving frame's cost.** A frame of the carousel mid-slide costs no more on the GPU than a full redraw at
  rest (§14.3), under 1 ms on the desktop.

### 14.6 The time model, as built

**The scene owns the clock, and nothing reads wall time.** Every time is a `TimeSpan` that the host passes in, as
`PadNavigator.Feed(held, now)` takes its time (§4.29 of the settings reference). Tests step it with round numbers. In
the window (stage e), the host will pass the render loop's frame time.

- **LunaP's half (its §102).**
  - `Glide` is a value moving between two numbers over a span of the host's clock, with an Avalonia easing.
  - `ImageCarousel.Position` draws the row at a fractional item.
  - `TextScroll` is the rule a self-scrolling text follows: a loop for one line, and for a column a run to the end, a
    pause and a fade-in at the top.
  - `FontText` and `TextRowList`'s selected row take that rule with a time since they were shown or selected.
  - None of them knows about ES-DE, and none keeps a clock.
- **Mistress's half, in `BigPicture/Scene/`.**
  - `SceneView` is one view that moves.
    - A change of selection (`Step`, or `Press` and `Release` for a held direction) rebuilds the view from its
      `ResolvedView` and the new data, exactly as §13's static builder does.
    - `Advance(now)` fires the key repeats that have fallen due, then pushes every time-derived state into the
      controls: the carousel's position from its glide, the list's and the containers' times since the selection,
      `scrollFadeIn`, and the metadata fade.
  - `SceneRepeat` is a held direction's repeats: a first delay, an interval, and a faster tier after a while held.
  - `SceneStage` holds the two views and the move between them, instant or sliding, as the theme's transition
    profile says.
  - `SceneMotion` holds every duration, curve and rate, and `SceneMotion.Esde` holds the values measured from ES-DE
    (§14.7).
  - Each theme property that governs motion (`itemTransitions`, `fastScrolling`, `textHorizontalScroll*`,
    `container*`, `scrollFadeIn`) is read where §13's mapping reads the rest.

**Why the view is rebuilt on each step, not updated in place.** The rebuild is §13's pure function of (view, data), so
a frame at any time is the static frame of its data with the time-derived state applied. That makes the strongest
test possible: when a motion settles, its frame must equal a fresh static render of the new state, pixel for pixel.
`A_carousel_step_eases_to_the_next_item_and_settles_on_the_static_picture` checks exactly that, and so does the slide
between views. The cost is a rebuild per step, not per frame. Between steps, a frame only sets positions, times and
opacities. §14.8 measures what a rebuild costs while a direction is held.

**The motion state is the scene's, not the controls'.** The rebuild makes new controls, so any state a control kept
would be lost at each step. The carousel's glide, the time of the selection and the metadata fade therefore live in
`SceneView`, and are applied to whichever controls exist.

### 14.7 ES-DE 3.4.1 measured moving, and the rules taken from it

**Method.** A subagent recorded ES-DE from its behaviour alone; ES-DE's source was not read, and its only text read
was `THEMES.md` and `USERGUIDE.md`. The recordings, scripts and per-run CSVs are under
`~/.cache/emusen/bigpicture/motion/` and are not committed.

- **Recording.** ES-DE ran as an XWayland client (`SDL_VIDEODRIVER=x11`) at 1280×800 in the scratch home, and ffmpeg's
  `x11grab` recorded its window losslessly at 165 fps, with wall-clock timestamps 6.06 ms apart.
  - A still frame of the recording equals a spectacle capture of the same state on every flat area. The edges differ by
    a sub-pixel shift (mean 0.39 levels), because spectacle's copy goes through the compositor.
- **Input.** A uinput gamepad (`pad.py`, vendor 0x1209, product 0x5E5D, named "EmuSen Motion Probe Pad"), with an
  explicit `SDL_GAMECONTROLLERCONFIG` and `SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1`.
  - The ids are ones Steam does not recognise, so Steam ignored the pad, and focus did not matter.
  - Every event's time was logged on the same clock as the frames. The first changed frame follows a press by 7–16 ms.
  - The device was destroyed after each run. Nothing was installed, and no keyboard was simulated.
- **Inputs.** Art Book Next on the shared synthetic library, and a synthetic theme written for the purpose
  (`theme/motion-probe`). It has flat squares, `itemScale` 1.5 and `unfocusedItemOpacity` 0.3, so that scale and fade
  can be read, and six variants that change the font size, speeds, gaps and sizes.
- **Fits.** Durations are taken within a recording. Twenty candidate curves are fitted to normalised progress, with
  the start time and the duration both free. "rms" is the residual in progress.

**What was measured, and what the scene now does** (n is the number of repetitions; `SceneMotion.Esde` holds the values):

| Motion | ES-DE 3.4.1 | Mistress |
|---|---|---|
| Carousel step: slide, scale and opacity | one quadratic ease-out; Art Book Next's slide 396 ms (387–407, n=10, rms 0.003; sine-out next at 0.009); the synthetic theme's slide, scale and fade 400–404 ms each (n=10, rms 0.003–0.005) | `Glide` over 400 ms, `QuadraticEaseOut`; opacity and scale by distance (LunaP §102.2) |
| A step while one is moving | a fresh 400 ms from the carousel's current position | `Glide.Toward` |
| Logo and system name on a step | change at the press, not animated | the view is rebuilt at the press |
| Carousel held, `fastScrolling` true | step at the press, the next at 497 ms, six at 179.8 ms, then 80 ms from about 1.59 s; no further tier in 3 s (n=3; Art Book Next agrees: 25 steps in 3 s) | 500 ms, 180 ms, then 80 ms from 1.58 s |
| Carousel held, `fastScrolling` false | 497 ms, then a constant 200 ms (n=4) | 500 ms, then 200 ms |
| Text list, one step | no animation: the cursor and the rows jump in one frame (n=15) | none |
| Text list held | 497 ms, then 11 steps at about 114 ms (n=50), then at about 1.716 s a jump of **four entries** in one frame (5 of 5 holds), then one entry every 15.9 ms | 500 ms, 114 ms, a four-entry step at 1.703 s, then 15.9 ms |
| Text list ends | a held direction stops at the end; a tap wraps | the same |
| Metadata and media while scrolling | a linear fade-out of 149 ms (148.3–149.4, n=5) from the **first repeat**; a linear fade-in of 150 ms (n=4) when scrolling stops, on release or at the end of the list | the same; the elements are those `THEMES.md`'s `metadataElement` entry lists |
| Selected name (`textHorizontalScrolling`) | still for the delay (3.0 s; 1.0 s when the theme sets 1), then constant speed; one full loop (text width plus gap), a frame bit-identical to the start, the delay again | LunaP's loop (§102.3) with the delay before each pass |
| Its speed | proportional to the font size and independent of the text's length and the list's width; 3.08 font sizes a second with ES-DE's own font (24, 32, 40 px: 74.1, 98.8, 122.8 px/s), twice that at speed 2; **131.5 px/s for Art Book Next's 30 px Mulish**, 4.38 font sizes a second | 131.5 / 30 font sizes a second, times `textHorizontalScrollSpeed` |
| Its gap | the gap factor times the distance travelled in one second at speed 1: 113, 150, 186 px at 24, 32, 40 px for 1.5; 298 px for 3; 201 px in Art Book Next | the factor times one second of travel |
| Vertical container | still for `containerStartDelay` (6.03 s for 6); constant speed in whole-pixel steps to the end; still for `containerResetDelay` (6.995 s for 7, 1.497 s for 1.5); back at the top at opacity 0, a **linear fade-in of about 298 ms** (295–303, n=7); the start delay again | LunaP's column rule with `WholePixels` |
| Its speed | depends on the font size only, not the container or the text's length, and doubles at speed 2; 20.0, 60.6 and 74.6 px/s at 24, 36 and 48 px; **37.03 px/s for Art Book Next's 30 px Mulish** | 37.03 / 30 font sizes a second |
| Horizontal container | the name's rule: about 3.09 font sizes a second at speed 1 with ES-DE's font (148.3 px/s at speed 2, 24 px); gap 223 px at 3 | the name's rule |
| `scrollFadeIn` | on every change of game, the image appears at **about half opacity** and rises linearly to full over **326 ms** (321–328, n=5, rms 0.005) | from 0.5 over 326 ms |
| Video element without a video (Art Book Next's `game-art`, delay 3) | the cover appears in the selection's frame, with no fade; nothing changes at the delay or in the 4.4 s after (n=5) | nothing: `delay` stays unmapped |
| View transition, `instant` | no intermediate frame (n=5 each way) | instant |
| View transition, `slide` | a **vertical camera pan** of 800 px, cubic ease-out, 402 ms (397–411, n=5; back 403.5 ms); the gamelist comes up from below; the help bar changes at the press and the status stays still | the same pan; the new view's help bar, clock and status held still, the old view's hidden |

**Checked against a recording.** `EsdeMotionCompareTool` replays ES-DE's recorded carousel taps (10, both ways, in
the synthetic theme) on Mistress's scene and compares every fully visible item in every one of about 2,000 recorded
frames, 10,301 item boxes in all. Allowing ES-DE 5 ms of input latency, the best of 0, 5, 10 and 15 ms:

| | Median | p95 | Worst |
|---|---|---|---|
| Centre while moving (px) | 0.50 | 2.20 | 6.22 |
| Centre when settled (px) | 0.50 | 0.50 | 0.50 |
| Drawn width (px) | 0.60 | 0.62 | 2.48 |
| Opacity | 0.00 | 0.00 | 0.02 |

The test asserts the settled centres within 1 px, the moving ones within 3 px at p95, and opacity within 0.05. The
worst moving error, 6 px, falls in the first frames of a step, where ES-DE's speed is highest and a millisecond of
latency is worth several pixels.

**Where Mistress's rules are narrower than ES-DE's behaviour.**
- **Neither speed is a law, only calibrations.** The name's speed is proportional to the font size, but the constant
  differs between fonts: 3.08 font sizes a second with ES-DE's font, 4.38 with Mulish. A font's own metrics
  presumably decide it, but which metric was not established. The vertical container's speed fits no simple law of the
  font size at all: its figures per font size are 0.83, 1.68 and 1.55 at 24, 36 and 48 px. Both constants in
  `SceneMotion.Esde` are Art Book Next's own, measured on its font. For another theme's font they are an estimate,
  and are recorded as such.
- **The carousel's selected tint, dimming and saturation change hands half-way** (LunaP §102.2). Art Book Next sets
  the same colour for both, so nothing shows. ES-DE was not measured with different values.
- **Unmeasured:** the fast tiers at 60 Hz, where ES-DE's 15.9 ms interval may be bounded by the frame rate; the
  device-notification popup (it fades in and out over about 0.5 s each); one unexplained 300 ms fade of a description,
  seen once in the synthetic theme on a change from a game without a description, and not reproduced.

### 14.8 The moving frame's cost, and a lever for the held list

`SceneGpuBench.RunMotions` times moving frames on the scene's own clock at 60 Hz: 600 frames each, the view stepped,
laid out and drawn on the RX 6800. The cases:
- `carousel held`: a step started as each one settles;
- `list held`: a direction held through ES-DE's repeat tiers down a list of 120 games and back up.

Each frame is split into the clock step and layout, the GPU recording, and the whole frame. The runs below are from
the self-contained Release host (`deck-gpu`); the test host's Debug build is noisier but agrees.

| 1280×800 | Frame median | p95 | Worst | Allocated a frame | Collections (gen0/1/2) |
|---|---|---|---|---|---|
| Carousel held | 0.33–0.52 ms | 0.56–1.23 | 6.9–10.0 | 52 KiB | 1/0/0 |
| List held, rebuilt on every step | 0.80–1.91 ms | 1.22–2.95 | 15.8–16.8 | 199 KiB | 11/11/3 |
| List held, with the lever below | 0.40–0.67 ms | 0.56–0.93 | 10.3–11.9 | 75 KiB | 3/3/0 |

**The median frame is well under a millisecond, and the worst frames are collections.** In the slowest runs the
frames over 8 ms fell on no step in particular. Their cost moved between the clock step and the recording, and the
collection count and the summed pauses accounted for them. A list scrolling at 62 entries a second rebuilt its view 62 times a second, allocated 199 KiB a frame, and paused
for 70–80 ms in 600 frames. On the desktop the worst of those frames reached 16.8 ms, the edge of a 60 Hz frame. On
a slower handheld it would cross it.

**The lever: move only the list while the metadata is faded out.** ES-DE fades the game's metadata and media out from
the first repeat of a held direction (§14.7). Once that fade has finished, everything in the view that follows the
selected game is at opacity 0, and only the list shows the selection. `SceneView` then changes the existing list's
`SelectedIndex` instead of rebuilding the view, and rebuilds the rest when the fade-in starts.
- **It is pixel-identical by construction, and proved so.** `SceneView.RebuildEveryStep` turns the lever off. Two views
  were run in lockstep with the lever on and off: 150 frames of Art Book Next at 60 Hz through a hold and its release
  (`Held_list_lever_is_pixel_identical_on_Art_Book_Next`), and a synthetic test with its own frames. Not one pixel
  differed.
- **Measured at 1280×800:** allocation 199 → 75 KiB a frame, collections 11/11/3 → 3/3/0, the median frame
  1.16 → 0.67 ms and the worst 15.4 → 10.3 ms in the test host; 0.80 → 0.40 ms and 15.8 → 11.9 ms in the Release host.
- **Not done:** the remaining 50–75 KiB a frame. It comes from the draw path's per-frame objects: new transforms in
  arrange and apply, and text shaped on every render. The worst frames are still collections. Pooling those objects would be the next
  lever, and it is also pixel-identical by construction. It is not needed on the desktop and waits for the handheld's
  numbers.

**Negative result, recorded so it is not repeated.** The first version of the bench reused one held-list script for
both sizes. Its direction leaked from the first size into the second, so the second size's list never moved and
measured 0.01 ms. The cases are now made fresh for each size.

### 14.9 Tests and mutants

**Tests.**

| Class | Tests | What it holds |
|---|---|---|
| `SceneMotionTests` | 12 | ES-DE's values pinned; a step easing and settling on the static picture pixel for pixel; wrapping and `itemTransitions instant`; key repeat and the fast tier; the list's jump, wrap and stop; fast scrolling only when the theme asks; the name's scroll and its restart; a container's rule; `scrollFadeIn`; the metadata fade; the lever against a rebuild on every step; the slide |
| `SceneMappingTests` | 247 cases | 13 new time-governed pairs, each rendered at a time on the scene's clock with two values and required to differ |
| `EsdeMotionCompareTool` | 1 | the recording comparison above; it skips when the recording is absent |
| `SceneMotionTool` | 5 | the PNG strips below, and the lever on Art Book Next |
| LunaP `MotionTests` | 10 | LunaP §102.5 |

**The mapping.** Of the 172 pairs Art Book Next sets, the scene now reads **140**. The four new ones are the
carousel's `itemTransitions` and `fastScrolling`, and the text's `containerStartDelay` and `containerVerticalSnap`.
The video's `delay` stays unmapped on ES-DE's evidence: without a video file it changes nothing.

**Mutants.** The runners are `~/.cache/emusen/probe/bigpicture/mutate_motion_scene.py` (27 mutants) and
`mutate_motion_lunap.py` (13 mutants), and the logs are `mutants-motion-*.txt`. **All 40 were caught.** Four survived
their first run:
- the marquee's gap measured in font sizes rather than seconds of travel;
- the help bar panning with the view;
- a horizontal text drawing no second copy;
- the marquee scrolling every row.

In each case the test had not looked at the pixels that decide, and each test was then strengthened.

The lever has three mutants:
- moving nothing;
- never rebuilding;
- engaging before the fade-out had finished.

The first two were caught by both lockstep tests. The third was at first caught only by Art Book Next's lockstep run:
the synthetic test's fade-out was shorter than a repeat interval, so no step fell inside it. That test now fades over
250 ms against 100 ms repeats, and catches the third as well.

**A defect found on the way.** The lever's first version looked up the list with `FirstOrDefault(…).Control`, which
throws in a gamelist whose primary element is a carousel. `SceneMappingTests`' held `fastScrolling` case, a carousel
in the gamelist, found it. The lookup is now null-safe (`0ee9f2e4`).

The recording comparison caught the step's easing and its duration (300 ms against 400) as well as the synthetic
tests did.

**PNG strips** (under `~/.cache/emusen/bigpicture/motion-png/`, not committed):
- `carousel-step-1280x800.png`, with the mid-slide frame at 100 ms in `carousel-step-1280x800-frame.png`;
- `list-held-1280x800.png`, showing the metadata fading out at the first repeat and the list alone while held, and
  `list-released-1280x800.png`, showing it fading back;
- `description-1280x800.png`;
- `slide-1280x800.png`, with its middle frame in `slide-1280x800-frame.png`.

They were looked at. The carousel's middle frame shows two slices half-focused. The held strip shows the cover,
badges and description gone while the list runs. The slide's middle frame shows the system view leaving upward and
the gamelist arriving from below, under a help bar that holds its place.

### 14.10 Predictions retired

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P4 | a step settles in 150–400 ms | 396–404 ms | held, at the edge of its range |
| P8 | carousel steps hold 60 fps on the handheld with fast scrolling | desktop: worst 6.9–10 ms, median 0.33–0.52 ms; handheld not run | open (handheld) |
| P24 | the GPU route's picture within 8 levels on 99% of pixels | failed against the compositor on text; 99.93% against the CPU route | failed as written (§14.3) |
| P25 | a GPU full redraw under 3 ms on the desktop | 0.33–0.71 ms | held |
| P26 | the first GPU frame under 50 ms | 3–22 ms | held |
| P27 | immutable bitmaps change the GPU frame under 5% | same pixels, same time | held |
| P28 | the handheld's full redraw under 8 ms | not yet run | open |
| P29 | a step of 150–400 ms, ease-out, scale and opacity on the same curve, whatever the distance | 400 ms quadratic ease-out for all three; a new step restarts 400 ms from where the row is | held |
| P30 | repeats after 400–500 ms, then every 60–150 ms, a faster tier after 1–2 s | 497 ms for all; the list every 114 ms, the carousel every 180 or 200 ms (outside the range); tiers at 1.58 s and 1.70 s, at 80 and 15.9 ms | held for the delay and the tiers; the carousel's interval refuted |
| P31 | a list step does not animate | none, in 15 of 15 | held |
| P32 | the name scrolls after 3 s at 50–150 px/s, repeating with a gap | 3.0 s, 131.5 px/s, a gap, and the delay again before each loop | held |
| P33 | the container starts after its delay at 20–40 px/s, stops, waits `containerResetDelay`, fades back at the top | 6.03 s; 37.03 px/s; 6.995 s; a 298 ms linear fade-in | held |
| P34 | `scrollFadeIn` over 150–300 ms | 326 ms, and from half opacity, not from none | refuted |
| P35 | the video element's image with no delay and no fade | none, in 5 of 5 | held |
| P36 | the slide takes 200–500 ms, decelerating | 402 ms, cubic ease-out; vertical, which was not predicted, and which the first build had horizontal | held for timing |
| P37 | settled frames equal the static render pixel for pixel, and ES-DE's positions to 1 px | pixel-equal in every test; ES-DE's settled centres within 0.5 px | held |
| P38 | a moving frame costs no more than a full redraw at rest, under 1 ms on the desktop | carousel median 0.33–0.52 ms; held list 0.80–1.91 ms median rebuilding on every step, 0.40–0.67 ms with the lever; worst frames up to 16.8 ms, from collections | held for the carousel and for the list with the lever; refuted for worst frames and for a list rebuilt on every step |

### 14.11 Not done in stage (c)

- **Nothing was measured on the handheld.** P8 and P28 wait for `deck-gpu` (§14.4), which now also runs the moving
  cases.
- **The draw path's remaining allocation** (50–75 KiB a frame) is not pooled (§14.8).
- **The vertical container's speed law and the name's per-font constant** are calibrations on Art Book Next's font
  (§14.7).
- **The carousel's selected tint changes at half-way** rather than blending (LunaP §102.2); unmeasured in ES-DE.
- **The popup** that ES-DE shows when an input device goes was not built. Mistress has its own notices.
- **The scene is not yet in Mistress's window.** No render loop drives `SceneStage.Advance` from frame times; that is
  stage (e), with the pad.
- **Grid motion** (`rowTransitions` and the grid's `itemTransitions`) waits for the grid in stage (f).
