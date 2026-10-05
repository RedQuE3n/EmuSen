# EmuSen_BigPicture — a plan for a big-picture mode in Mistress that renders ES-DE themes

*Written 2026-09-24. Stage (a), the theme loader, was built the same day; its record is §12. Stage (b), the two views drawn statically with LunaP controls, followed on 2026-09-24 and 25; its record is §13. Stage (c), the GPU frame and motion, followed on 2026-09-25; its record is §14. Stage (e), the themed view as the big-screen library driven by the pad, followed the same day; its record is §15. Stage (f), variants and settings, the grid, the triggers against Mistress's media, and themes downloaded on request, followed on 2026-09-25 and 26; its record is §16. Stage (d), ScreenScraper, followed on 2026-09-26; its record is §17. Stage (g) is not built.* A big-picture mode in Mistress like EmulationStation's was requested,
starting with Art Book Next as its theme. Two choices were made at the outset. First, Mistress reads ES-DE themes, so that
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
MIT-licensed (§2.1), which would permit reuse with a notice. By decision the plan copies none of it anyway,
and implements only the format that ES-DE's `THEMES.md` documents. For reading, both theme repositories were cloned
outside the EmuSen tree: `~/Projects/art-book-next-reference` (Batocera edition) and
`~/Projects/art-book-next-es-de-reference` (ES-DE edition). Nothing was copied from either.

---

## 0. Summary

- **The repository first linked is not an ES-DE theme.** `anthonycaccese/art-book-next-es` is the edition for the
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
  issued to the software through the ScreenScraper forum, and optionally the player's own account. EmuSen's author must
  request the developer credentials (§5.7). Those credentials live in a file outside the repository, and neither they
  nor the player's password is ever committed or logged.
- **Stages (§7).** Seven stages, (a) to (g), about 17–27 working days in all, plus waiting time for the developer
  credentials. Each stage has an oracle, and the rendering stages use ES-DE itself as that oracle. ES-DE is run
  locally on inputs Mistress also receives. Its captures are kept outside the repository.

---

## 1. What was asked, and what the request had to be corrected on

### 1.1 Two editions of Art Book Next, and which one ES-DE can load (measured and cited)

The link in the request is `github.com/anthonycaccese/art-book-next-es` (cloned at `9a50ef3`, 2026-03-11). Its README
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
is 51). **The option set the request listed belongs to the Batocera edition:**

| Option | Batocera edition (the link) | ES-DE edition |
|---|---|---|
| Aspect ratios | 16:9, 16:10, 4:3, 3:2, 1:1 | 32:9, 21:9, 20:9, 19.5:9, 16:9, 16:10, 3:2, 4:3, 5:4, 8:7, 1:1, 5:3 vertical |
| Colour schemes | Default, Light, Steam OS, SNES, Famicom, OLED, Custom | Six palettes (Dark, OLED, Light, Steam OS, SNES, Famicom), each with five artwork sets (Screenshots, Noir, Circuit, Outline, Original Screenshots), plus Custom: 31 |
| Layout | "Game Artwork" and "Game Metadata" menu options | 20 variants: 7 lists, 3 grids, and each again with the help bar off |
| Font size | Default, Small, Large, Extra Large | small, medium, large, x-large |
| Fonts | Roboto, ChangaOne | Mulish, ChangaOne |
| Metadata icons credited to | FontAwesome | Phosphor Icons |

The ES-DE edition's credits add the Outline artwork set (Joppa Fallston) and artwork by theUnBurn to those the request
listed. Its licence line is the same: CC BY-NC-SA 2.0.

Q1 (§10) asks for the ES-DE edition to be confirmed. The recommendation is yes. It is the same author's same design,
it is in the format chosen above, and ES-DE's own documentation describes that format.

### 1.2 The handheld's resolution (cited, and in conflict)

The request says the Legion Go S runs at 1280×800. `EmuSen_Settings_Reference.md` §4.45.2 describes "a Legion Go S's
8-inch 1920 by 1200 panel". Both are 16:10, so the aspect-ratio choice is the same either way. The performance budget
is not: 1920×1200 has 2.25 times the pixels. §4.8 prices both, and Q2 asks which the tester actually runs, because
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
(`inventory.py`, in a scratch folder, not committed). The script counts element types and the properties
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
| `animation` (GIF, Lottie), `gameselector`, `gamelistinfo` | | when a theme a player wants uses them | |
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
> needed, and a single drawn control is the cheaper shape for one frontend. The decision judged it by what the toolkit
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
  screen recording at 60 fps of the same inputs, not read from ES-DE's source. That keeps to the project's rule, and a
  measured curve is what the test needs anyway.
  - **P4.** A carousel step settles in 150–400 ms, and our settled positions equal ES-DE's to 1 px.

### 4.7 Video

Art Book Next plays video in 12 of its 20 variants, the first-declared one included (§3.3).

**The degradation is exact.** A `video` element with an `imageType` shows that image during `delay`, and shows it
again when there is no video file ("If `imageType` is not defined, then the default image will be shown if there is
no video file found"; with `imageType` set, the image is what fills the delay and follows `onIterationsDone image`).

**So if no videos are scraped, Mistress's render and ES-DE's are the same render for the same media.** That makes
deferring playback a scope decision, not a fidelity loss. Q4 asks whether videos are wanted at all. They
cost the most quota and bandwidth (§5.5), and ES-DE plays their sound by default, because `audio` defaults to true
and Art Book Next does not change it.

Options for when playback is wanted:

| Option | Licence | Weight and fit |
|---|---|---|
| LibVLCSharp 3.10.1 plus `LibVLCSharp.Avalonia` | LGPL-2.1-or-later, compatible with GPL-3.0 | `LibVLCSharp.Avalonia` 3.10.1 is built against **Avalonia 11.3.13**, the same Avalonia 12 gap as §4.5. It needs libVLC and its plugins. NuGet has no `VideoLAN.LibVLC.Linux` package, so on SteamOS's read-only system libVLC would have to be bundled (tens of MB). The desktop has `libvlc.so.5`. |
| FFmpeg through bindings (`FFmpeg.AutoGen` 9.0.1.1, or `Sdcb.FFmpeg` 7.0.0, LGPL-3.0-only) into a frame texture | FFmpeg itself is LGPL-2.1+ when built without GPL parts | Decode only: H.264 and AAC in MP4. **ES-DE's own player is FFmpeg-based** (`es-core/src/components/VideoFFmpegComponent.h` exists; no VLC component does). The desktop has FFmpeg 8.1.2 (`libavcodec.so.62`). What SteamOS provides was not established. |
| FFmpeg as a child process piping raw RGBA frames | the same | No bindings. Needs an `ffmpeg` binary on the machine or bundled. The simplest to build, and the one Serenity's frame path already fits (`UpdateFrame(rgba, w, h)`). |
| Defer | nothing | Exact for the media Mistress fetches (above). |

**Recommendation.** Defer to stage (g). If videos are wanted, use FFmpeg, bindings or process, chosen by a
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
| L1, R1 / L2, R2 | | page / first and last. *Since the answer of 2026-09-26 (Q14): ten games back or forward, stopping at the ends, as USERGUIDE's shoulders do; the triggers are unchanged, the first and the last game* |
| North | | the on-screen keyboard's search (§4.45.6). *Since the answer of 2026-09-26 (Q15): the favourite (`favorite`), ES-DE's Y; while a custom collection is edited, adding the game to it or removing it, as ES-DE's Y does then; the search is an entry of Select's menu (§22.13)* |
| Select | | mark favourite (`favorite`), as the grid does today (§4.33). *Since 2026-09-26 the game options menu, as ES-DE's Back button opens it; the favourite is its first entry (§23.4). Since the answer of the same day (Q12), one menu: Jump To, Sort Games By, Filter Gamelist and Search, the custom collection's entries, then the game's (§22.13)* |
| Start | the pad menu, as a `SheetLayer` sheet | the same |
| Guide, or Back and Start together, during a game | the menu over the game (§4.29). "Game Library" returns to the themed view at the same system and game. | |

*Amended 2026-09-26, for collections and the gamelist options (§22.5):* North, while a custom collection is being
edited, adds the selected game to it or removes it, as ES-DE's Y does, and searches otherwise; either thumbstick pressed
in picks a random game (in the system view a random system, when the setting allows it), as ES-DE's thumbstick click
does; South on an entry of the grouped Collections system opens it and East inside it comes back to the list. ES-DE's
gamelist options menu, which ES-DE opens with Back, is reached from Start's pad menu, because Select belongs to the
game options of §23; merging the two menus under Select is left to that merge. *Superseded the same day by the
answers (§10.1, §22.13): the menus are one, under Select, and North is the favourite outside the collection editing.*

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
be offered in a desktop session, where §4.43 keeps the menu bar and sidebar. *Answered twice: in big-screen sessions
only (2026-09-25), then, as requested on 2026-09-26, also on the desktop, behind a Big Picture entry
below the plain Fullscreen one in the View menu (§10.1, §18).* *The Library style row is gone since 2026-09-26: on the
user's request, EmuSen's own library is the first, built-in entry of the Theme Settings sheet's Themes list, and
Preferences' row became **Big Picture Theme**, listing the same entries (§10.1, §19).*

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
  - *Amended 2026-09-27 (§30):* the same guide section keeps a foldered game's media under its folder,
    `<system>/<type>/<folder>/<rom stem>.<ext>`, which this bullet's layout left out; the store follows it now.

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

### 5.7 Credentials: what the author must do, and how they are kept

**What the author must do.**

1. **Create a ScreenScraper member account** at `screenscraper.fr`. That gives an `ssid` (user name) and `sspassword`.
   The member account carries the quota and threads of §5.5, and contributing to the database or donating raises
   them.
2. **Request developer credentials for EmuSen.** The API page says developers should "contactez-nous via le forum pour
   présenter votre logiciel", that is, introduce the software on the ScreenScraper forum. The team then issues an
   identifier and password for the software. Reports say the team does not issue developer credentials to users, only
   to software authors, so the request must come from EmuSen's author. The post should give:
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
    That means only on the author's machines, since players cannot obtain developer credentials themselves (step 2).
  - The recommendation is the first, with Q5 confirming it.
  - *Implemented 2026-09-27 (Q40, which reversed Q5's answer; §28 is the record, §4.65 of the settings reference the
    player's account).* The first option was built as written: `EmuSen.Mistress/Scraping/ScreenScraperDeveloper.targets`
    embeds, at `dotnet publish` only, the file named by `EmuSenScreenScraperDeveloper` (default
    `~/.config/EmuSen/screenscraper-developer.json`), XORed with a 32-byte key generated for that build and stored in the
    same resource. It is obfuscation in exactly ES-DE's sense and no more: it defeats a search of the binary for the
    password and the secret scanners that work that way, and nothing else.

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

| Stage | What it covers | Its oracle | Cost | What the player sees at the end |
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

- **A native look-alike built from LunaP controls.** Refused at the outset. It would also have to be redesigned for
  every theme.
- **Supporting the Batocera format.** It is a different format (§1.1), undocumented beyond its fork, and ES-DE itself
  gave up reading legacy themes. It is not recommended unless a theme exists only in that form (Q1).
- **Bundling a theme.** Refused by the licence (NC and SA against GPL-3.0), and by the original request.
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
| P9 | Hash lookup identifies 75–95% of 40 random files; headerless hashes add at most 5 points | Stage d: failed high, 39 of 40 by the file's own hashes; step 2 added none (§17.10) |
| P10 | A new account scrapes the whole library, without videos, over two to three sessions in two days | Stage d: refuted by projection from the 40-file run, not by a full run: the day's 10,000 requests make it three days (§17.10) |
| P11 | Art Book Next's archive is 205–230 MB | Stage f: held, 220.2 MB (§16.7) |
| P12 | With no videos scraped, the video element's render is identical to ES-DE's (§4.7) | Stage b: failed as identical (§13.8) |
| P13–P18 | Stage (a)'s predictions: coverage, errors, skipped includes, load time, triggers, the default variant (§12.1) | Stage a (§12.5) |
| P19–P23 | Stage (b)'s predictions: frame cost, SVG against `Svg.Skia` by file, properties mapped, geometry from the theme, the font size (§13.1) | Stage b (§13.10) |
| P24–P38 | Stage (c)'s predictions: the GPU route, its frame, first frame and uploads, the handheld; the carousel step, repeat, the list, the name, containers, scrollFadeIn, the video delay, the slide, settled frames and the moving frame (§14.1, §14.5) | Stage c (§14.10); P28 open |
| P39–P45 | Stage (e)'s predictions: a still view draws nothing, the return is exact, the first build, the pad's family, a family change touching only the help bar, the sounds, every sheet reachable (§15.1) | Stage e (§15.12); P41 failed cold |
| P46–P55 | Stage (f)'s predictions: a choice applied at once, choices per theme, the sheet's rows, every control reachable, the grid at rest and moving, the video extensions, the media scan, the downloads, a closed sheet's download (§16.1) | Stage f (§16.7); P50 failed by a pixel, P51's duration and interval refuted |
| P60–P67 | Stage (d)'s predictions: N64 byte order, media against the quota, quota fields without a member, time per game, system IDs, the pacer, the region rule, no leak (§17.1) | Stage d (§17.10); P61 and P66 failed, P60 not measured, P63 partly |
| P68–P71 | Collections and gamelist options: the broad run, a large library's Show with the collections, a step in all games, the options sheet's cost (§22.1) | §22.10 |
| P84–P91 | The game options menu and the metadata editor: every control reachable, an edit surviving a re-scrape, the editor's own scrape, no ROM touched, a click on the rating, the mutants, the broad runs (§23.1) | §23.1: all held but P88 (failed, then fixed) and P90 (one unattributed failure) |
| P100–P120 | The remaining passes' predictions: the handheld, controllers, the theme survey, badges and switches, localisation, folders, the modes, scraping extras, manuals, the screensaver, the launch screen, video, TheGamesDB, the passes' pace (§21.6) | Each in its pass |
| P123–P129 | Pass 3's open questions of fact: ES-DE on the themes the loader refuses, a live download, a live opening, GitHub's allowance, the survey's missing assets, pass 14's first rules (§25.9) | Each when measured |

---

## 10. Risks and open questions

- **Q1, which theme.** Confirm the ES-DE edition (`art-book-next-es-de`). The linked Batocera edition cannot be read by
  ES-DE, and its options differ (§1.1).
- **Q2, the handheld's resolution.** 1280×800 or 1920×1200? Is the Legion Go S's panel driven at its native
  resolution in Game Mode, or does gamescope scale up from 1280×800? §4.8 prices both.
- **Q3, how much of the format.** The recommendation is §3.8's first column, then more only when a theme a player
  wants needs it. The loader is complete from the start, because a partial grammar breaks whole themes.
- **Q4, video.** Scrape and play videos (stage g, FFmpeg, 3–5 days, more quota and bandwidth, sound on by default in
  the theme), or not? Without them the render is exact and the cost is nil (§4.7).
- **Q5, the developer credentials in published builds.** Embed them at publish time from a file outside the
  repository, as ES-DE does, or ship builds that scrape only on the author's own machines? Either way they are never
  committed (§5.7).
- **Q6, the member account.** Contributing to ScreenScraper, or a one-off €10, raises threads and quota and cuts §5.5's
  two days to hours. That is the player's choice, not the software's.
- **Q7, miximages.** One variant needs them. Use ScreenScraper's `mixrbv2` (the nearest, and a different look), build
  a miximage generator (a later stage), or hide that variant?
- **Q8, the existing big-screen library.** Keep it as the fallback and a choice, as recommended (§4.10)? Offer the
  themed view on the desktop too?
- **Q9, engine-supplied graphics.** The help bar's button icons, the favourite and folder indicators and the badge
  overlay must be Mistress's own drawings (§3.6). Is a simple drawn set acceptable, or should they match a particular
  controller family, such as Xbox, PlayStation, or the Legion's own labels?
- **Q10, SVG.** Our own subset renderer (recommended), or `Svg.Skia` 5.1.1 with its MS-PL dependency and SkiaSharp
  pin (§4.5)?
- **Q11–Q14, collections and the gamelist options** (§22.12): the automatic collections off by default as ES-DE's are,
  or on; one menu under Select for the gamelist options and the game options; custom collections shown until switched
  off, or enabled one by one as ES-DE does; and the shoulders' page, which ES-DE was measured to make ten games.
  *All four answered 2026-09-26, with Q15 (§10.1).*
- **Q20–Q35**, the decisions the remaining passes to ES-DE parity wait on (video again, as Q4's revisit; codecs;
  controllers; folders; the modes; languages; the theme list; TheGamesDB; buttons; the launch screen; the screensaver;
  the hardware session), are asked in §21.5.
- **Risk: ScreenScraper's API is in beta** and may change without notice (§5.1). The client keeps the response
  parsing in one place, and its tests are written against the documented shape.
- **Risk: the undocumented behaviours** of §4.4 (the 'S' size) and §4.6 (durations) are measured from ES-DE, so they
  track the ES-DE release measured. The release is recorded with every capture.
- **Risk: the theme changes upstream.** Updating is the player's choice (§6). A new theme version that uses an element
  outside §3.8 is refused visibly, element by element, not rendered in part.

- **Q15–Q19**, the game options menu and the metadata editor's open questions: §23.12. Q15 was answered with Q11–Q14,
  and Q18 and Q19 on 2026-09-26 (§10.1); Q16 and Q17 remain open.
- **Q36–Q39** are left to §25, written on another branch at the same time. **Q40–Q45**, the retirement of OpenEmu's
  sources, are asked in §26.9, and **Q46** in §27.9.
- **Q50–Q54**, pass 4's (the systems' default order, the scroll overlay's timing, Art Book Next's unplaced badges, a
  grid's shoulders and the controller types), are asked in §29.11.

---

### 10.1 Decided (2026-09-24 onward)

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
- **Q5, developer credentials:** **the author's own machines only.** Published builds do not carry them; a build
  without the local file cannot scrape. *Reversed by Q40 on 2026-09-26 and built on 2026-09-27 (§28).*
- Q6–Q10 remain open; each is asked when its stage reaches it.
- **Q8 and Q9, and navigation sounds (decided 2026-09-25, before stage e):**
  - **Q8:** the existing big-screen library **stays**, as the fallback when no theme is installed and as a choice
    after one is (Preferences, "Library style"). The themed view is offered **in big-screen sessions only**; the
    desktop keeps its sidebar library.
  - **Q8, amended 2026-09-26:** the desktop needs a button to enter big picture mode as well. A first build made the
    desktop's full screen and big picture one state; that was corrected the same day: the fullscreen button must not
    trigger big picture on the desktop, and a separate button makes the window full screen and also enters big
    picture, so that the player has the choice between both on the desktop. Its place: under the View menu, below
    Fullscreen. The desktop therefore has **two options**, adjacent in the **View** menu: a plain **Fullscreen** (F11,
    and the window manager's own), which keeps the sidebar library, and directly below it **Big Picture** (its own key, F10), which
    makes the window full screen and enters big picture. The desktop keeps its sidebar library by default, and the
    themed view is offered wherever big picture runs, under §4.52's four conditions. A Game Mode session stays big
    picture throughout. §18 is the record; §4.54 of the settings reference is the player's account.
  - **Q8, amended again (2026-09-26):** EmuSen's normal big-picture theme is to be an option in the themes list.
    The existing big-screen library is no longer a separate "Library style" beside the theme: it is the first
    entry of the Theme Settings sheet's Themes list, **EmuSen (built in)**, never downloaded or removed, and choosing it
    or an ES-DE theme there applies at once. Preferences' "Library style" row became **Big Picture Theme**, a dropdown
    of the same entries over the same two settings (`LibraryStyle`, `BigPictureTheme`). §19 is the record; §4.56 of the
    settings reference is the player's account.
  - **Q9:** the help bar's button icons **follow the connected pad**: Mistress detects the controller family and draws
    its own set for it (Xbox, PlayStation, Nintendo, and a generic set when unknown). The favourite, folder and badge
    graphics are Mistress's own drawings.
  - **Sounds:** the theme's navigation sounds play through a small UI sound stream, **on by default** with a switch in
    Preferences.
- **Q20–Q35, answered 2026-09-26 after §21 was written:**
  - **Q20 and Q21, video:** all three uses (the theme's clips, the media viewer and the screensaver), built late as §21's
    pass 12, through the system's own `ffmpeg` run as a separate process, so EmuSen ships no codec. Video scraping is
    off by default. This supersedes Q4's "not now".
  - **Q22–Q35:** every recommendation §21 made is accepted as written. Where a pass finds that a recommendation cannot
    hold, the question is reopened.
  - **The first pass to build:** pass 2, controllers.
- **OpenVGDB is kept as a fallback (decided 2026-09-27).** This retires §26's retirement plan. The
  direction recorded on 2026-09-26 was to "eventually retire" OpenEmu's library as a source of game information. It is
  withdrawn: OpenVGDB and libretro-thumbnails stay as the failover behind ScreenScraper, asked only inside a scrape
  the player starts (§17.14), as they are today. §26's measurements stand as a record of what the fallback yields.
  Its removal steps (§26.7) are not to be carried out. What survives of the decisions:
  - **Q41 and Q43** still hold.
  - **Q42's Remove button** stays as an optional way to free 42 MB. Removing the database disables the fallback until
    it is downloaded again, and nothing removes it on its own.
  - **Q45** (*Use Another Game's Cover…*) is built as a replacement that stands on its own.
  - **Q44** no longer applies: the fallback is not switched off after pass 8.
  - **Q40,** the embedded credentials, is unaffected.
- **Q189, Q192 and Q193, decided 2026-09-27 (§43):** **Q189:** the fit audit's readability floor measures the drawn capitals' height, not the font size, so a condensed face is held to what the eye sees. **Q192:** `PadAudit`'s walk is fixed to reach every control a pad can reach, as a manual walk does. **Q193:** the controller drawings' label cap stays at 17.1 design pixels.
- **Q160–Q165, Q170–Q178 and Q180–Q185, decided 2026-09-27; every recommendation accepted:**
  - **§40:** **Q160:** the swap setting trades X/Y as well as A/B, as ES-DE's does. **Q161:** the Nintendo family keeps lettering by position. **Q162:** "Clear Metadata" and "Hide Game" stay. **Q163:** text rows say "Select". **Q164:** the themed search follows the On-Screen Keyboard setting. **Q165:** a text row chosen with Enter on a physical keyboard gets the field alone, without Steam's keyboard.
  - **§41 (the window look), approved with its checkpoint:** **Q170:** the Cheat Database's introduction moves into the footer. **Q171:** body text stays at 24 design pixels. **Q172:** capitals for buttons, tabs and headings only. **Q173:** file-dialog buttons stay hidden on sheets. **Q174:** one glyph for the L1/R1 pair. **Q175:** the plain frame's viewport is fixed too. **Q176:** a focused text box keeps Backspace and the arrows. **Q177:** the scraping status collapses its empty picture slot. **Q178:** focusing a Cheat Database row selects it. Also: lines cut at the panel's edge, such as the scraping status's recent games, end in an ellipsis.
  - **§42 (the bindings window), approved with its checkpoint:** leader lines no longer cross the drawing (each label sits on its button's side), and the N64's Z gets a default pad button, the left trigger. **Q180:** Test Buttons stays. **Q181:** only player 1's pad lights the drawing. **Q182:** one capture takes a key or a pad button. **Q183:** the SNES face buttons keep the Super Famicom colours. **Q184:** big-screen space is decided once the window look lands. **Q185:** the General tab gains the generic pad as a raw tester beside the deadzone slider.
- **The controller bindings window, decided 2026-09-27:** it is overhauled. Each console's controller is drawn as vector art, with the bindings mapped onto its buttons, and a button pressed on the pad or keyboard lights up on the drawing, so the window doubles as an input tester. The drawings are EmuSen's own, not copied from any emulator or maker.
- **The look of every window, decided 2026-09-27:** the menus need not be converted one-for-one with ES-DE's. What is kept is ES-DE's look (its colours, the Barlow Condensed face, the rounded panels) and a controller-friendly layout wherever one can be had. Windows that need their own layout, such as the ScreenScraper windows, the cheats window and the controller bindings, keep it and take the look.
- **Q105–Q109 and the help bar, decided 2026-09-27 (§34):**
  - **The help bar:** letter-labelled button glyphs, as ES-DE draws them, following the controller type in use, replacing the four-button diagram.
  - **Q105:** the editor's subtitle is ES-DE's one line, the file name and its system. **Q106:** the editor's help bar changes per row, as ES-DE's does. **Q107:** `MenuPanel` shows ES-DE's scroll indicator when its rows overflow. **Q108:** leaving the editor returns to the gamelist, as now.
  - **Q109, changed:** text entry uses the device's own on-screen keyboard. Under Steam (SteamOS Game Mode, or Desktop Mode with Steam running) Mistress focuses a real text field and asks Steam for its keyboard through `steam://open/keyboard`. Steamworks' `ShowFloatingGamepadTextInput` needs a real Steam app ID, which a non-Steam shortcut does not have. Mistress's own keyboard stays as a fallback, chosen in Preferences (Automatic, Steam, EmuSen's).
  - *Built on 2026-09-27 (§40; settings reference §4.79). Steam's keyboard is not yet checked on the handheld (P262–P266); Q160–Q165 are §40.12's.*
  - *Q160, Q164 and Q165 built on 2026-09-27 (§40.13–§40.15; settings reference §4.79.5–§4.79.7). Steam's keyboard measured on the handheld the same day; the popup moved above it (§40.16, §4.79.8).*
- **Q100–Q104, Q120–Q122 and Q130–Q132, decided 2026-09-27; every recommendation accepted:**
  - **Stage 2's metadata editor (§34):** the look is approved. **Q100:** an ES-DE text popup is built with the rest of stage 2, and the on-screen keyboard's Shift and Done keys are widened meanwhile. **Q101:** Reset moves to a pad button (X), shown in the help bar. **Q102:** ES-DE's keyboard keys drive the big-screen menus as pad buttons do. **Q103:** the desktop editor keeps the Hide metadata fields switch. **Q104:** ES-DE's own metadata editor is captured and the proportions checked against it. Also: the Hide from Library question no longer names ES-DE, and long controller names use a short form.
  - **Pass 14 (§36):** items 1 and 2 are approved. "1 GAMES" becomes "1 GAME", and the Bluetooth indicator draws its symbol rather than the letter B. **Q120:** the carousel's `selectedItemMargins` and `lineSpacing` come next, then `gamelistinfo`, then `animation`. **Q121:** Mistress shows its own word where ES-DE writes "unknown", through Pass 5's text lookup. **Q122:** ES-DE's default help bar is measured and drawn for a theme that has none.
  - **Pass 10 (§37):** **Q130:** the Game Mode switch stays on until the hardware session shows Steam's dimming stacking with it. **Q131:** the overlay keeps the shelf's system name, as the launch screen does. **Q132:** "Render scanlines" is built with Pass 12's video screensaver.
- **Q110–Q111, decided 2026-09-27 (§35):**
  - **Q110:** a theme with no chosen variant draws its first selectable variant from the start, as Mistress already does. ES-DE draws the first declared variant until its settings are first opened; the two agree for all 66 listed themes.
  - **Q111:** a bare relative path (no `./`) resolves against the working directory, as ES-DE's does, as built in §35.
- **Q70–Q72, answered 2026-09-27 (§31):**
  - **Q70:** yes. The other value types are probed against ES-DE as the six refusals were: colours, whole numbers (Canvas's and Iconic's `3.5`, P173), strings, paths, and `capabilities.xml`'s `selectable` (P172). The number with an exponent (P170) goes in the same run. The loader then matches ES-DE rule by rule.
  - **Q71:** yes. A variant that states no `<selectable>` is treated as ES-DE treats it: not offered for selection. This is confirmed in Q70's run before it is built, since only one ES-DE observation supports it ("NONE DEFINED" for a lone variant).
  - **Q72:** no. Theme warnings are never shown to the player; they go to the error log only (settings §4.70), as they do now.
- **Q47–Q49 and the keyboard's menu key, answered 2026-09-27 by accepting the recommendations:**
  - **Q47:** (b). ES-DE is run on the refused themes first, and the loader matches what it does rule by rule (§25.10). Done in §31: ES-DE
    loads all 15, and 66 of 66 listed themes now load.
  - **Q48:** pass 14 draws `gameselector` (20 themes), then wheel carousels (9), then `gamelistinfo` (6), then
    `animation` (4). The recommendation named those four in that order. §25.10's fuller order also
    puts Q47's refusals and the cheap badge, help and carousel properties first. Q47 is answered separately, and
    pass 4 covers most of the badge and help properties, so the two orders do not conflict.
  - **Q49:** full-screen screenshots in the theme browser are built with pass 9's media viewer, not before.
  - **The keyboard's menu key:** F4 opens the Start menu in the themed view. Escape keeps its role of leaving big
    picture or returning to a suspended game (settings reference §4.52a). The alternative, Escape as ES-DE's Start
    with F10 as the only way out, was not chosen.
- **Q41–Q46, answered 2026-09-26 (§26.9, §27.9):**
  - **Q41:** the 670 covers OpenVGDB already fetched keep their rank above ScreenScraper's.
  - **Q42:** OpenVGDB's 42 MB database gets a Remove button in Preferences; nothing deletes it on its own.
  - **Q43–Q46:** every recommendation is accepted. The mix image is not used as a last-resort cover (the placeholder is).
    The fallback stops being asked only after pass 8's search by name exists. *Use Another Game's Cover…* is built.
    Select stays Favourite in the sidebar library.
  - **Q40, answered the same day once ES-DE's practice was known:** do what ES-DE does. Published builds carry
    EmuSen's ScreenScraper developer credentials, scrambled, and generated into `obj/` at publish from the developer
    file named by a publish property, so they never enter the repository (§5.7's first option). Every player can then
    scrape; a member login only raises their limits. **This reverses Q5** ("builds carry no developer credentials").
    The risks are accepted as ES-DE accepts them: the credentials can be extracted, and abuse could get the
    `EmuSen-Mistress` software name blocked (426), which the scraper already reports.
    **Implemented 2026-09-27** (§28): the publish step, the resolution order (the tree's file, `~/.config/EmuSen`, then
    what the build carries), and the redactor over the embedded values.
  - **Q42 and Q45, built 2026-09-27** (§28): OpenVGDB's Remove in Preferences, worded, after OpenVGDB was kept
    (above), as an optional way to free its space that switches the fallback off until the database is downloaded again,
    with a Download beside it; and *Use Another Game's Cover…*, kept as a row of `games.db` rather than a copied file.
- **Q7, miximages (decided 2026-09-26, during stage d):** ScreenScraper's ready-made mix, `mixrbv2`, is fetched as the
  miximage. It looks different from ES-DE's own composed miximages; building our own composite is not wanted now.
- **Q5, the developer credentials, as received (2026-09-26):** issued to EmuSen's developer, kept only in
  `~/.config/EmuSen/screenscraper-developer.json` (mode 0600) with `softname` `EmuSen-Mistress`, verified against
  `ssinfraInfos.php` the same day. No build carries them. *Since Q40 (§28), a publish run on that machine embeds them
  from that file; a plain build still carries none.*
- **Drawn with LunaP (decided 2026-09-24): the view is drawn with LunaP, and whatever LunaP is missing is added to
  it.** This supersedes §4's one Skia-drawn control in Mistress. Every visible part is a LunaP
  control, and what LunaP lacks is added to LunaP under its own conventions (a `docs/LunaP.md` section, tests, the API
  baseline, palette-only colours where a theme does not set one): an SVG image (the renderer §4 argued for, now in
  LunaP), the carousel, the grid and text list as themed, text in a theme's own fonts, rating, badges, the help bar,
  the clock and system status, and a positioned canvas for ES-DE's normalised coordinates and origins. The ES-DE
  loader (XML, variables, includes, variants, colour schemes, aspect ratios) is format-specific and stays in
  `EmuSen.Mistress/BigPicture/`, with no Avalonia types, producing a scene the Mistress layer builds from LunaP
  controls. Q10 (own SVG renderer or `Svg.Skia`) is therefore answered: our own, in LunaP.
- **Q11–Q15, collections, the gamelist options and the favourite's button (decided 2026-09-26, after §22 and §23 were
  merged):**
  - **Q12:** one menu on Select, as ES-DE has it: Jump To, Sort Games By and Filter Gamelist, the custom collection's
    entries, then the game's entries (the favourite, Edit This Game's Metadata, Scrape This Game..., and Hide from
    Library in the editor). Gamelist Options left the pad menu, where ES-DE's documentation keeps only *Game collection
    settings*.
  - **Q15:** North (ES-DE's Y) toggles the favourite; the search moves into Select's menu as an entry. While a custom
    collection is being edited, North adds the game to it or removes it, as USERGUIDE describes ("The option does not
    affect the use of the Y button to add or remove games when editing custom collections").
  - **Q11:** the automatic collections are on by default, a deliberate difference from ES-DE, whose settings file holds
    them empty.
  - **Q14:** the shoulders jump ten games in a gamelist, as USERGUIDE documents; the triggers stay the first and last.
  - **Q13:** custom collections stay shown until switched off, as §22 built them.
  - §22.13 records what was built for these.
- **Q18 and Q19, ScreenScraper's name and the options outside big picture (decided 2026-09-26, after §23 was
  merged):**
  - **Q18:** offer ScreenScraper's name, but never force it. The metadata editor shows ScreenScraper's name for
    the game as an offer under the Name field, taken by one press (*Use This Name*) or put away (*Keep Current Name*).
    The name changes only when the player takes it; the file's name, or the player's own edit, stays otherwise.
  - **Q19:** yes, the options are available in both. The game options menu and the metadata editor are also
    reached from Mistress's sidebar library: the grid's and the list's context menu, the pad menu of that library on the
    desktop and in its big screen, and Ctrl+I for the editor. The same windows are used, as sheets in a big-screen
    session and as LunaP windows on the desktop.
  - **A direction, not yet a decision to build:** eventually retire sourcing game information from OpenEmu's library
    and use ScreenScraper. §26 is the plan for it; §27 records what was built for Q18 and Q19.

- **Q80 to Q86, the big-screen menus' look (decided 2026-09-27, approving §32's stage 1):**
  - **Q80:** keep *Main Menu* as the Start menu's title over the library.
  - **Q81:** over a running game the title is the game's name as the library shows it, with no extension or folder.
  - **Q82:** keep the chosen row's text white on the bar.
  - **Q83:** keep Barlow Condensed.
  - **Q84:** version 0.9.0: the footer reads *EmuSen 0.9.0*, from the product version at its source, not a string
    in the footer (§32.11).
  - **Q85:** match ES-DE: a game's options, and a folder's, are titled *Gamelist Options*.
  - **Q86:** build ES-DE's list screen for an option row in stage 2.

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
| `capabilities.xml` missing or malformed | error | theme not loaded | cited; since §31, a bare `&` and text outside the root are read as ES-DE reads them, with a warning |
| No `theme.xml` for the system | error | unthemed | cited |
| Malformed XML in any file read; a root other than `<theme>` | error | unthemed | cited; since §31, only what ES-DE also refuses (mismatched tags), not a bare `&` or text outside the root |
| An unknown tag, element or property | error | unthemed | cited as "enforced more strictly"; the unknown-property case is inferred |
| The legacy `extra` attribute | error | unthemed | cited |
| `<variant>`, `<aspectRatio>` or `<include>` inside `<view>`; a variant in a variant | error | unthemed | cited |
| A view other than `system`, `gamelist` or `all`; a missing `name` | error | unthemed | cited ("mandatory") |
| A property with no value | error | unthemed | cited (the log line in `THEMES.md`) |
| A value in the wrong format (a pair of one number, a five-digit colour, `yes` for a boolean) | error | unthemed | cited ("sanitization for valid data format"); **superseded by §31 (2026-09-27)**: ES-DE reads FLOATs, pairs with a space and BOOLEANs leniently (`yes` is true), and refuses only a pair with no space; **and by §35**: whole numbers are never refused, and a colour only when it is not 6 or 8 characters long |
| An undefined variable in a property | error | unthemed | cited ("a missing variable") |
| A missing include written out | error | unthemed | cited |
| An include loop | error | unthemed | chosen; ES-DE hangs |
| An `imageType` naming an unknown type | error | that element is not rendered | cited; **superseded by §35 (2026-09-27)**: a warning, the system themed, as in ES-DE |
| An `imageType` repeating a type | warning | the property is ignored | cited |
| An enum value not in its list | warning | default | cited; since §35, compared untrimmed and case-sensitively, as ES-DE compares it |
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
   **Superseded by §35.3 (2026-09-27):** ES-DE does not offer a variant without one, and reads the value by its own rule.
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
10. **A path with neither `./` nor `~`** is resolved from the file's folder. **Superseded by §35.2 (2026-09-27):** ES-DE
    resolves it against its working directory, and so does the loader now.

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

ES-DE was not available when the stage opened. It became available part way through (the AppImage was
downloaded by hand), and is run only from a copy under `~/.cache/emusen/bigpicture/esde/`, with its own `--home` there.

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
- **Full quality was chosen (2026-09-25)**, so only levers whose pixels equal a HighQuality redraw were eligible.
  None helped:

  | Lever | Result |
  |---|---|
  | An immutable bitmap instead of a `WriteableBitmap` | 43.35 ms against 43.60, so Skia is not rebuilding mips per frame |
  | Unfocused opacity carried in the tint instead of a layer | no measurable change |
  | `BitmapCache` on the carousel | no saving; pixels changed by up to 131 |
  | Not redrawing an unchanged view | already what the compositor does |

  One rejected change is recorded so it is not proposed again. Drawing bitmaps prepared at display size 1:1 on whole
  pixels took the system view to ~10 ms, but it resamples once at a different quality. It was rejected.
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
- ES-DE 3.4.1 (r51) ran from a copy of the downloaded AppImage, with checksum 3c61a44d…3581, under
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
  blurring deliberately, which the full-quality rule excludes.
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
about 1/25 of a 60 Hz frame, so motion can be built on plain full redraws. No lever planned for §14 (static layers
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

**Method.** ES-DE was recorded from its behaviour alone; ES-DE's source was not read, and its only text read
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
in the gamelist, found it. The lookup is now null-safe (`6e163696`).

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

### 14.10a The handheld's run: P8 and P28 retired (2026-09-26)

`deck-gpu` was run on the Legion Go S under `systemd-run --user` (13:49–13:52): three rounds, 120 frames a case, on
the charger (`ACAD online 1`), platform profile `custom`, Desktop Mode (no gamescope), the Z1 Extreme's radeonsi
driver through surfaceless EGL. Medians over the three rounds; the raw file is
`~/.cache/emusen/probe/bigpicture/deck-gpu/handheld/results/20260926-134911.txt`.

| Case | Full redraw on the GPU (wall to `glFinish`) | CPU raster, for scale |
| --- | --- | --- |
| System view, 1280×800 | 0.65 ms (p95 0.76–0.86) | 49.3–49.8 ms |
| Gamelist, 1280×800 | 0.72–0.74 ms (p95 0.85–1.36) | 22.4 ms |
| System view, 1920×1200 | 1.83–1.95 ms (p95 2.08–2.37) | 94.2–94.5 ms |
| Gamelist, 1920×1200 | 0.95–1.00 ms (p95 1.04–1.59) | 40.8–41.3 ms |

| Motion, whole frame | 1280×800: median / worst | 1920×1200: median / worst |
| --- | --- | --- |
| Carousel held | 0.66 / 9.44 ms | 2.09 / 13.55 ms |
| List held (the §14.8 lever) | 0.63 / 8.59 ms | 0.90 / 8.97 ms |
| List held, rebuilt every step (no lever) | 0.99 / 13.58 ms | 1.12 / 15.94 ms |

- **P28 held.** A full redraw fits 16.7 ms with a factor of 25 to spare at 1280×800, and of 8 at the panel's
  1920×1200.
- **P8 held.** The carousel held at 1280×800 never exceeded 9.44 ms, so it keeps 60 Hz. The worst frames, as on the
  desktop, are single outliers, not the median; the lever of §14.8 keeps the held list's worst under 9 ms at both
  sizes, where the rebuild-every-step path reached 15.94 ms at 1920×1200, within 0.8 ms of a missed frame.
- **Handing Skia immutable bitmaps** changed neither pixels (0.00 % against the first variant) nor time, as on the desktop.
- **The first GPU frame** costs 21–33 ms (5.7–8.5 ms with immutable bitmaps), paid once when the view is built.
- **Not measured:** Game Mode's gamescope compositing on top of this, and the frame as Mistress's real window presents it.

### 14.10b The handheld at 1920×1200, where it is played (2026-09-26)

The Legion Go S is played at its panel's full 1920×1200, not the 1280×800 §10.1's Q2 read from gamescope's
arguments. A second run at that size only, five rounds, 240 still frames and 1,800 moving frames (30 s at 60 Hz) a
case, on the charger, Desktop Mode (`results/20260926-135821.txt`):

| Case | Median frame | p95 | Worst, per round |
| --- | --- | --- | --- |
| System view, full redraw | 1.66–1.95 ms | — | — |
| Gamelist, full redraw | 0.92–0.98 ms | — | — |
| Carousel held | 1.89–2.05 ms | 2.33–2.62 ms | 12.48, 13.62, 14.28, 15.14, 15.79 ms |
| List held (the §14.8 lever) | 0.89–0.90 ms | 1.18–1.35 ms | 9.56–12.91 ms |
| List held, rebuilt every step | 1.09–1.10 ms | 1.59–1.71 ms | 14.24–15.58 ms |

- **P8 held at 1920×1200, narrowly for the worst frame.** No frame in 9,000 carousel frames missed 16.7 ms; the worst
  was 15.79 ms, 0.9 ms inside. The median is 2 ms, so the worst frames are isolated spikes, not the steady cost:
  one in 1,800 frames or fewer reached 12 ms. What they are was not measured here; on the desktop the list's were
  garbage collections (§14.8).
- **The carousel is the case to watch at this size.** Its GL time is 1.3–1.5 ms against the list's 0.31, the nine
  full-height system images being sampled with mipmaps at full quality; its spikes rose with the round (12.5 → 15.8 ms),
  which may be heat. A carousel spike under gamescope's own compositing, which this run does not include, may cross
  the frame.
- **The lever matters more here.** Without it the held list's worst reached 15.58 ms; with it, 12.91.
- **Q2's reading is superseded for performance targets:** 1920×1200 is the size to measure first on the handheld.

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

---

## 15. Stage (e): the themed view as the big-screen library, driven by the pad

*Opened 2026-09-25.* Stage (e) puts the scene of §13 and §14 into Mistress's one window as the library of a big-screen
session, under §10.1's answers to Q8 and Q9: the existing big-screen library stays as the fallback and as a choice,
the themed view is offered in big-screen sessions only, the help bar's icons follow the connected pad's family, and
the theme's navigation sounds play through a small stream of their own, on by default. Everything over the view is
drawn inside the window; no window is opened.

### 15.1 Predictions, written before the host was built

- **P39, a still view draws nothing.** Once a move has settled, the host requests no frame until something is due to
  change. On Art Book Next's system view at rest it requests none at all. On its gamelist, where the selected name and
  the description scroll by themselves after a delay, it requests none in the 2.9 s after a list step settles, and
  resumes at the first scroll's delay (3 s for a name wider than its row, 6 s for the description).
- **P40, the return is exact.** A game started from the themed gamelist and left through the pad menu's "Game Library"
  comes back to the same system and the same game, and the first frame after the return equals, pixel for pixel, a
  fresh static build of that selection.
- **P41, the first build.** Loading Art Book Next for EmuSen's five systems and building the first system view in the
  window costs under 300 ms headless on the desktop, of which the loader's share is under 10 ms (§12.5 measured
  0.91 ms per system).
- **P42, the pad's family.** SDL's own gamepad type decides the family for every pad SDL recognises: its two Xbox types,
  its three PlayStation types and its four Nintendo types each give one family, and a Standard or Unknown type falls
  back to the pad's name. A pad whose name carries "Legion Go" is taken for an Xbox layout, as reported from the device.
- **P43, a family change touches only the help bar.** Changing the connected pad's family redraws the help bar and
  changes no pixel outside its box.
- **P44, the sounds.** Each of the seven navigation actions plays exactly the theme's file for it, once per step,
  including the steps a held direction repeats; with the switch off nothing reaches the sound stream. Decoding the
  seven WAV files of Art Book Next costs under 20 ms.
- **P45, every control is reachable.** The themed session's pad menu and every sheet it opens, on the 1280×800 sheet
  size, leave no control unreachable to `PadAudit`, as the Mistress library's did (§4.45.3 of the settings reference).

### 15.2 What was built

**In LunaP** (branch `bigpicture-pad`, its `docs/LunaP.md` §103). Nothing there knows about ES-DE:
- `PadGlyph`, with `PadFamily` (Generic, Xbox, PlayStation, Nintendo) and `PadGlyphButton` (the face buttons by
  position, the d-pad whole and by axis, shoulders, triggers, the two middle buttons, the guide button): one button drawn
  in one colour as the toolkit's own geometry;
- `HintEntry.Button` and `HintBar.PadFamily`, so a hint names a button and the bar draws it in the pad's set;
- `TextScroll.NextLoopChange` and `NextRunChange`, `FontText.NextScrollChange` and `TextRowList.NextMarqueeChange`:
  the earliest time a self-scrolling text will look different, or none.

**In Mistress:**
- `BigPicture/ThemedLibrary.cs` holds the stage, the pad's rules, the sounds and the selection. The selection is kept by
  the system's name and each system's game file, so a rebuild from new data (a favourite, a search, a refresh, a return
  from a game) finds it again. *Since 2026-10-03 a favourite's toggle keeps the row instead, and the game moves alone
  (§44).*
- `Views/MainWindow.BigPicture.cs` puts it in the library's place, feeds it the library's shelves, records and covers,
  routes the pad and runs the render loop.
- `Input/PadFamilies.cs` gives a pad's family from SDL's type, then its name. `BigPicture/DeviceStatusReader.cs` reads
  the battery and radios from sysfs.
- The scene gained `SceneView.Jump`, `Stepped`, `SetFamily` and `NextChange`, `SceneStage.Replace` and a `Switch` that
  takes new data, and `SceneData.Family` and `ShowClock`.
- `HelpPrompts` maps each ES-DE help entry to its action and its `PadGlyphButton`. Stage (b) had mapped the entry `x` to
  Search, but §4.9's search is North, the button ES-DE's help entries call `y`; it is `y` now.
- Preferences ▸ Appearance: Library Style, ES-DE Theme, ES-DE Media and Navigation Sounds.

**Elsewhere:** `CoreCatalog.LibraryShelf` carries ES-DE's system name for each shelf (§4.1). `GamepadManager` reports
SDL's gamepad type and lets a pulled pad go. `UiSoundPlayer` (Endymion) is the navigation sounds' stream.

The settings reference's §4.52 is the player's account of all this; this section is the record.

### 15.3 The pad's rules, and their tests

Every row of §4.9's table is a rule with a test in WiseMan, driven through `PadDriver` with a clock the test moves:

| Rule | Test |
|---|---|
| The themed view is the library of a big-screen session with a theme, and Mistress's otherwise (style, no theme, desktop, a broken theme) | `ThemedLibraryPadTests.The_themed_view_is_the_library…`, `Mistress_library_is_kept…` (4 cases) |
| Left and right move the system carousel, with `systembrowse`, repeating at 500 then 200 ms | `Left_and_right_move_the_system_carousel…` |
| South enters the gamelist (`select`), East comes back (`back`) | `South_enters_the_system_s_gamelist…` |
| Up and down move the list (`scroll`) at 500, 114 ms and stop at the end when held; left and right change the system at the game last chosen there (`quicksysselect`) | `Up_and_down_move_the_list…` |
| L1 and R1 page by the rows the list shows; L2 and R2 go to the first and last | `Shoulders_page_by_the_rows…`, `Shoulders_page_a_list_longer_than_a_page…` |
| North searches on the on-screen keyboard; East clears the search before it goes back | `North_searches_with_the_on_screen_keyboard…` |
| Select marks a favourite, which moves first and stays selected. *Superseded 2026-10-03: North and the options menu's entry mark it, it moves first, and the highlight keeps its row (§44)* | `Select_marks_a_favourite…`; *since §44, `North_toggles_the_favourite…`, `Select_opens_the_game_options_whose_favourite_entry…`, `ThemedFavouriteCursorTests`* |
| Start opens the pad menu over the view, and the view hears nothing under it | `Start_opens_the_pad_menu_over_the_view…` |
| South starts the game; the menu's Game Library comes back to the same system and game; East in the system view resumes it; Close Game comes back too | `ThemedLibraryFlowTests.A_pad_walks_systems_searches_favourites_starts_a_game_and_comes_back…` |
| The resume question is asked on a sheet over the view | `The_resume_question_is_asked_on_a_sheet…` |
| The help bar's icons follow the pad's family | `ThemedLibraryHostTests.The_help_bar_follows_the_connected_pad_s_family…`, `SDL_s_pad_types_and_names_give_the_families` |
| The sounds, their switch, and a new sound replacing the last | `With_the_switch_off…`, `A_new_sound_replaces…`, `A_navigation_sound_is_decoded…`, and the sounds asserted in every pad test |
| Preferences' Library Style takes effect when the sheet closes | `Preferences_chooses_the_library_style…` |

**Where the rules are Mistress's, not ES-DE's measured behaviour:**
- **The pad menu is the in-window panel of §4.29, not a `SheetLayer` sheet** as §4.9's table wrote. A sheet presents a
  window; the pad menu is a list the window draws over whatever screen shows, and §4.10 already said the pad menu is
  shared by both library styles. What §4.9 wanted of it, that no window is opened, holds, and every window the menu opens
  is a sheet. `Start_opens_the_pad_menu…` asserts no owned window exists.
- **The shoulders page.** The scratch home ES-DE wrote in stage (b) carries `QuickSystemSelect` =
  `leftrightshoulders`, which suggests ES-DE's default changes the system on the shoulders as well as on left and right.
  §4.9 gave the shoulders to paging, after §4.29's grammar, and this stage kept §4.9. Which ES-DE does on a real pad was
  not measured.
- **Quick system select does not repeat when held.** ES-DE's behaviour was not measured.
- **The search** has no ES-DE counterpart: ES-DE has no search box in a gamelist.
- **The system order** is the shelves' release order; ES-DE's was not established.
- **The clock is off**, as ES-DE's `DisplayClock` is by default (§13.8), with no switch yet.
- **A new sound replaces the one playing.** ES-DE's behaviour under a fast-scrolling list was not measured; queuing would
  make a held list's sounds trail seconds behind it.

### 15.4 Launching and returning

South in the gamelist plays `launch` and calls `StartGameAsync(file, name)`, the path every start takes (§4.31 of the
settings reference): the firmware prompt, then the resume question on a sheet, then the load, which hides the library.
The themed view is not torn down while the game runs; it keeps its selection, and the render loop stops because the
library is hidden. The menu over the game's "Game Library" calls `ToggleLibrary`, which shows the library and refreshes
it; the refresh shows the theme again at the kept system, game and view. "Close Game" goes through `ShowLibrary` and
comes back the same way. East in the system view, with a game suspended, resumes it.

- **P40 holds.** `The_first_frame_after_the_return_is_a_fresh_static_build_of_the_same_selection` starts Cobalt Harbor,
  opens the menu, chooses Game Library, and compares the whole window with `SceneBuilder.Build` of the same data in a
  window of its own: 0 pixels differ at 1280×800.

### 15.5 The pad's family

`PadFamilies.Of(type, name)` takes SDL's `SDL_GetGamepadType`, which SDL derives from the pad's vendor and product ids:
its two Xbox types give Xbox, its three PlayStation types PlayStation, its Switch Pro and three Joy-Con types
Nintendo (a GameCube type, which SDL 3.4's C# binding does not declare, would too). For Standard and Unknown the pad's name decides: "Xbox", "X-Box", "XInput", "Legion Go", "Steam Deck" and
"Steam Virtual Gamepad" give Xbox; "PlayStation", "DualShock", "DualSense", "PS3/4/5" and "Sony" PlayStation;
"Nintendo", "Switch", "Joy-Con", "Pro Controller" and "GameCube" Nintendo; anything else Generic. The window asks at
every pad poll, and the view redraws only its help bar (`SceneView.SetFamily`).

`SDL_s_pad_types_and_names_give_the_families` lists the family each of SDL's type names must give and fails on a type
the binding declares that the list does not name, so a new SDL type has to be classified by a person.

- **P42 holds** for every type SDL 3.4 names, and for the names tested. On the device it is unverified: under Steam's
  Game Mode a pad reaches SDL as Steam's virtual pad, and what SDL reports for the Legion Go S there was not read.
- **P43 holds.** With the family changed from generic to PlayStation, Nintendo and Xbox (the last by the name "Lenovo
  Legion Go S" on a Standard type), 2,091, 2,430 and 2,493 pixels changed inside the help bar and none outside it.

### 15.6 The sounds' stream

`UiSoundPlayer` opens its own SDL audio stream on the default playback device (`SDL_OpenAudioDeviceStream`), 48 kHz
stereo float, on the first sound played. The game's `AudioPlayer` has its own stream on the same device. **SDL mixes the
two**: each opened stream is a logical device, and SDL sums the logical devices of one physical device, so the
interface's stream is neither resampled by the game's rate control nor paused with the game, and clearing it touches
nothing of the game's. Each WAV is decoded once with `SDL_LoadWAV` and converted with `SDL_ConvertAudioSamples`. A new
sound clears the stream before it is queued (`A_new_sound_replaces…`: after a 2 s sound and a 0.5 s one, 192,000 bytes
queued, the second alone). The gain is 0.7, ES-DE's navigation volume as its scratch settings record it.

- **P44 holds.** Every pad test asserts the sound of each action, repeats included (a held carousel's four steps, four
  `systembrowse`); with the switch off none reaches the stream; Art Book Next's seven WAVs decode in 4.8 ms.
- **Not measured:** the stream's latency. The game's stream asks SDL for 4,096-frame device buffers (§4.10 of the
  settings reference), and on a device both streams share that buffer; whether a navigation sound then trails the press
  audibly is for the handheld.

### 15.7 Drawing only while something moves: P39

The window asks the view, after every pad poll and every frame, when it will next look different
(`SceneView.NextChange`). The answer is now while a glide, a held direction, a fade or `scrollFadeIn` runs; the end of a
text's pause, from LunaP's queries (§103.3), while a name or a container waits; and never when all is still. Now asks for
the next frame (`RequestAnimationFrame`); a later time sets a timer; never does nothing. A text not yet laid out answers
now, so its first layout is not missed.

`A_still_view_draws_nothing…` models the window's loop, drawing only when asked:
- the system view at rest: **0 frames in 5 s**;
- a carousel step: 24 frames in 600 ms (400 ms of glide at 16 ms polls), then 0 in 3 s;
- a gamelist whose selected name is too wide: **0 frames in the 2.9 s** after it was entered, frames from 3 s;
- a short name: 0 frames in 8 s; a horizontal container: 0 frames until its start delay of 2 s.

- **P39 holds** on the synthetic theme. On Art Book Next it holds trivially: with Mistress's data no game has a
  description and every synthetic name fits, so the gamelist's next change is never, and the description's 6 s delay was
  not exercised. It waits for stage (d)'s descriptions.

### 15.8 The first build: P41

`Art_Book_Next_s_first_build_for_five_systems` builds a fresh `ThemedLibrary` five times over EmuSen's five systems and
the synthetic library's media, and shows its root in a new headless window. Desktop, Debug build:

| Run | Theme read | Media scan | Stage | First frame (layout, decode, draw) |
|---|---|---|---|---|
| 1 (cold in the process) | 50.3 ms | 1.9 ms | 52.4 ms | 385 ms |
| 2–5 | 9.4–49.0 ms | 1.2–1.4 ms | 0.3–0.8 ms | 49.8–51.3 ms |

- **P41 fails cold and holds warm.** The first build and frame took about 490 ms against a bound of 300; later ones about
  60 ms. The cold run is the process's first decode of the artwork, first SVG rasterising and first JIT of all of it, as
  §13.4's first build was.
- **The loader's share fails too:** 9–50 ms for five systems warm, against under 10. Each run reads `capabilities.xml`
  afresh (31 schemes, 20 variants), which §12.5's 0.91 ms per system did not include. The window keeps the capabilities
  and each resolved theme between showings, so a return to the library pays neither.
- **A large library.** `A_large_library_is_scanned_for_media_once…`: 3,508 games and an ES-DE media folder holding none of
  them took 224 ms to scan on the first showing, on the UI thread, and 0 ms on the next. That is the one cost here that
  grows with the library.

### 15.9 Every control reachable: P45

`Every_control_of_each_sheet_opened_from_the_themed_view_is_reached_by_the_pad` opens Cheats, Graphics Settings,
Shaders, Controller Bindings and Preferences from the pad menu over the themed view at 1280×800, walks every tab with
`PadAudit`, and closes each with East. **P45 holds: nothing unreachable on any of the five**, Preferences' four new rows
included; after East the view takes the pad again.

### 15.10 Pictures

`ThemedLibraryReferenceTests.Stage_e_pictures`, run with `EMUSEN_BIGPICTURE_PNG=1`, renders the whole window at 1280×800
and 1920×1200 into `~/.cache/emusen/bigpicture/png/stage-e/`: the system view, the gamelist, the pad menu over the view,
and the help bar cut out in each of the four families, one above the other, for the synthetic theme and for Art Book
Next. They were looked at:
- Art Book Next's system view shows the Super Nintendo slice centred between the NES and Game Boy slices, its logo, the
  status bar with Bluetooth (the desktop's own, from sysfs), and the help bar's MENU, SELECT and SYSTEM in generic icons.
- The gamelist shows the list with the favourite-first order, the synthetic cover and the metadata with ES-DE's words.
- The pad menu is drawn over the dimmed view.
- The family strip shows the four sets: dots and pills; A, B and the Xbox middle marks in rings; the four shapes; B and
  A cut out of discs, plus and minus.
- The synthetic theme's gamelist shows no cover: the test session has no art folder and no media folder, so the image
  element draws nothing, which is what a theme with no `default` image does. Its help bar lists all eleven entries at
  0.035 of the height and runs off the right edge at both sizes; that is the synthetic theme's layout, not a clip.
- **A flaw seen and left:** at the 1280×800 help bar's size the generic d-pad's up-and-down and left-and-right glyphs
  are hard to tell apart; the filled arms are narrow.

### 15.11 Mutants

The runners are `~/.cache/emusen/probe/bigpicture/mutate_stage_e.py` (32 mutants in Mistress, Endymion and the scene,
against the themed library's tests and the scene's mapping and motion tests) and `mutate_padglyph_lunap.py` (LunaP's
§103.5, ten, all caught). Logs are `mutants-stage-e.txt`, `mutants-stage-e-rerun.txt` and `run-stage-e.log`. Each
mutant was built and run alone and the source restored; the tree was rebuilt clean at the end.

| # | Mutant | Result |
|---|---|---|
| E1 | quick system select forgets each system's game | caught by 5 |
| E2 | a held direction steps once and does not repeat | caught by 2 |
| E3 | left and right in a gamelist do not change the system | caught |
| E4 | entering a gamelist ignores the game last chosen there | **survived**; caught after the test was changed |
| E5 | a page is always ten games | caught |
| E6 | a page wraps past the end of the list | caught |
| E7 | the first-game trigger moves one game | caught by 2 |
| E8 | East with a search goes back instead of clearing it | caught |
| E9 | a favourite is not listed first | caught |
| E10 | Start does not open the pad menu | caught by 3 |
| E11 | the library comes back at the system view, not where it was | caught by 4 |
| E12 | the render loop never sleeps | caught by 3 |
| E13 | a text's pause is not waited for, so the loop sleeps for good | caught |
| E14 | a family change leaves the help bar as it was | not built at first (the mutant left an empty statement); caught once rewritten |
| E15 | a view built after a family change draws the help bar generic | **survived**; caught after the test was changed |
| E16 | SDL's type ignored, the name alone decides | caught |
| E17 | the Legion Go S taken for a generic pad | caught by 2 |
| E18 | a held direction's repeats make no sound | caught by 3 |
| E19 | the sounds switch ignored | caught |
| E20 | quick system select plays the carousel's sound | caught |
| E21 | the status bar stays over the themed view | caught by 2 |
| E22 | Library Style ignored | caught by 2 |
| E23 | the view hears the directions under the pad menu | caught by 2 |
| E24 | the view hears the directions under the on-screen keyboard | caught by 2 |
| E25 | the view hears the directions under a sheet | caught by 2 |
| E26 | a direction also reaches the view as the navigator's press | **survived, equivalent** |
| E27 | the clock drawn although ES-DE's is off | caught |
| E28 | the search on the theme's `x` entry again | **survived twice**; caught after the test was changed |
| E29 | a peripheral's battery taken for the machine's | **survived**; caught after the test was changed |
| E30 | a new sound queues behind the last | caught |
| E31 | South in the system view enters no gamelist | caught by 13 |
| E32 | the launch plays no sound | caught |

**31 of 32 caught; the one survivor is equivalent.** E26 hands each direction the navigator also reports to
`ThemedLibrary.Command`, which has no case for a direction and so does nothing with it; no test can tell it apart, and
none should.

**The four survivors were weak tests, as in stages (b) and (c):**
- E4: every test that entered a gamelist from the system view entered the system the view had been built at, whose game
  the view's data already held. `South_enters…` now leaves one system at its fourth game, enters another never entered
  (its first game) and comes back (the fourth again).
- E15: the family test changed the family and looked only at the help bar already on screen; a new view's was never
  looked at. It now goes back to the system view and in again under the new family.
- E28: no test named the help entries at all. The first version of the new test listed both `y` and `x`, so `x` standing
  in for `y` gave the same list and E28 survived again; the test now lists `y` alone and requires `x` to name nothing.
- E29: the sysfs case named the machine's battery `BAT0` and the mouse's `hidpp_battery_0`, and the reader, which
  takes the first battery in name order, reached `BAT0` first whether or not it skipped the mouse. The mouse's folder
  now sorts first.

### 15.12 Predictions retired

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P39 | no frame when still; none in the 2.9 s after a list step on Art Book Next's gamelist; frames again at the pause's end | 0 frames in 5 s at rest; 0 in the 2.9 s before a wide name scrolls, frames from 3 s; 0 before a container's 2 s delay; on Art Book Next with Mistress's data nothing scrolls, so no frame at all | held; the description's 6 s untested until stage (d) |
| P40 | the return comes back to the same system and game, its frame equal to a fresh build | same system, game and view in every flow; 0 pixels differ at 1280×800 | held |
| P41 | first build under 300 ms, the loader under 10 ms | cold ≈490 ms (Show 105 ms, first frame 385 ms); warm ≈60 ms; the loader 9–50 ms for five systems warm | failed cold, held warm; the loader's share failed |
| P42 | SDL's type decides every recognised pad; the name otherwise; Legion Go as Xbox | every type the binding declares classified as listed; names as listed | held headlessly; unverified on a device |
| P43 | a family change touches only the help bar | 2,091–2,493 pixels changed in the help bar, 0 outside | held |
| P44 | each action's sound, once per step, repeats included; none with the switch off; seven WAVs under 20 ms | as predicted; 4.8 ms | held |
| P45 | nothing unreachable on the themed session's sheets | 0 on all five | held |

### 15.13 Not done in stage (e)

- **Nothing ran on the handheld.** P8, P28, the pad families of real pads (and of Steam Input's virtual pad), the sound
  stream's latency beside the game's 4,096-frame buffer, and whether the help icons read at arm's length are the
  device's.
- **The first showing's media scan is on the UI thread**: 224 ms for 3,508 games with an empty media folder on the
  desktop. It could move to a worker; it was left measured.
- **Badges a theme names no icon for** draw nothing, as before; ES-DE draws its own there. Art Book Next names its own,
  so nothing is lost for it. The folder-link overlay and the controller badge stay deferred (§3.8).
- **Settings the scene needs and does not have:** `DisplayClock` (off), the variant, colour scheme, font size and aspect
  ratio (the theme's defaults and Automatic; stage f), and ES-DE's automatic collections (off, as ES-DE's own default,
  `CollectionSystemsAuto` empty, has them).
- **ES-DE's behaviour where this stage chose:** the shoulders (§15.3), a held quick system select, the system order and
  a sound interrupted by the next were not measured against ES-DE.
- **The generic d-pad glyphs** are hard to tell apart at the 1280×800 help bar's size (§15.10).
- **`GamepadManager` still opens one pad**, the first; a second pad connected beside it is not seen until the first goes.
  *Closed 2026-09-26 by §24: every pad is opened, and every pad steers the interface.*


### 15.14 A closed window kept drawing (found at the merge, 2026-09-25)

**What was seen.** After stage (e) was merged into WiseMan, the Mistress test filter (about 690 tests) failed in two
of four runs, each time on a different test outside stage (e): the Shaders window's live-slider test once, the status
bar's Preferences test once. Each passed alone and in small groups. The build before the merge (`ce5fa675`, 648 tests)
passed three runs of three. An intermittent failure that lands on a different, unrelated test each time is the mark
of work done on the shared UI thread by something the failing test did not create.

**The defect, demonstrated.** `MainWindow`'s `Closing` handler stopped the pad timer but not the themed view's wake
timer, and did not release the UI sound stream. Once a scrolling text had armed the wake, a closed window went on
waking and drawing. `A_closed_themed_window_draws_nothing_after_its_theme_is_gone` closes a themed window, deletes its
theme as `ThemedSession.Dispose` does, and lets 2.6 s of real dispatcher time pass: without the fix the closed window
drew **41 more frames**; with it, none. The fix is `CloseThemedLibrary`, called from `Closing`: it marks the view
closed, stops the wake, and disposes `UiSoundPlayer`. `ScheduleThemedFrame` and `ThemedFrame` refuse to run after it.

**What is not shown.** That this leak caused the two failures above is argued, not proven: they were too rare to
reproduce on demand, and repeating the broad run until they reappear was ruled out after the desktop froze twice
during such runs on 2026-09-25; the freezes left nothing in the kernel log and are not attributed here. After the fix, one broad run without the GPU tests passed 676 of 676, and the Shaders window's ten
tests passed; one pass each is weak evidence against a failure seen at one run in two. A first test written for this,
asserting the wake timer disabled after closing, was dropped: on the unfixed code it passed or failed with timing,
so it could not tell the two builds apart.

**Why the mutants missed it.** §15.11's 32 mutants alter the view's rules while a window is open. None removed a
cleanup, because the defect was an absent line, and a mutation of existing code cannot produce an absence.

---

## 16. Stage (f): variants and settings, the grid, triggers, and themes downloaded on request

*Opened 2026-09-25.* Stage (f) is §7's row: the settings sheet built from `capabilities.xml`, the grid that six of Art
Book Next's variants use, the variant triggers evaluated against the media Mistress actually has, and themes
downloaded, updated and removed on the player's request, with a sheet that attributes each one. §10.1's rules hold:
every visible part is a LunaP control, the themed view is for big-screen sessions only, and nothing of Art Book Next or
of ES-DE enters either repository. ES-DE's grid is measured from its behaviour alone, as its motion was in §14.7.

### 16.1 Predictions, written before any of it was built

- **P46, a choice applies at once.** Choosing a variant, colour scheme, font size, aspect ratio or language on the
  settings sheet redraws the view beneath the sheet before the sheet closes, and the frame after the choice equals, pixel
  for pixel, a fresh static build of the same selection under the new choice.
- **P47, remembered per theme.** Each theme folder keeps its own choices in `appsettings.json`; switching to another
  theme and back restores them, and a stored choice the theme no longer declares falls back to §12.4's defaults without
  an error.
- **P48, the sheet lists what the theme declares.** For Art Book Next: its 20 variants in declared order under their
  `en_US` labels, 31 colour schemes, 4 font sizes, Automatic and 12 aspect ratios, no language row (§3.2: it declares
  none), and its two transition profiles after Automatic.
- **P49, every control reachable.** The settings sheet, the theme list and the attribution sheet leave nothing
  unreachable to `PadAudit` at 1280×800, as §15.9's sheets did.
- **P50, the grid at rest.** Art Book Next's `gamelist-grid-cover` at 1280×800 lays out five columns, and every item box
  of its first two rows lies within 1 px of ES-DE's still of the same state.
- **P51, the grid moving.** A step within a row animates the selected item's scale and the unfocused opacity over the
  carousel's 400 ms quadratic ease-out (§14.7), and a step onto a row that is not shown slides the rows over the same
  400 ms. Held, a direction repeats after 500 ms and then every 114 ms, as the text list does, with no four-item jump.
- **P52, the media Mistress has.** Reading `EsdeMediaFolder`, a `video` is never found: it looks only for `.png`,
  `.jpg` and `.jpeg`, while `USERGUIDE.md` lists `.mp4`, `.mkv`, `.avi`, `.wmv`, `.mov` and `.webm` for videos and
  `.webp` for images. Every `noVideos` trigger therefore fires for a system whose videos are all present. This is a
  defect predicted from the code; a test must show it before it is fixed.
- **P53, the media scan.** Listing each media folder once per system, instead of asking for every game, type and
  extension in turn, takes §15.8's 3,508-game scan from 224 ms to under 20 ms, and gives the same presence for every
  system.
- **P54, download, update and removal.** A download is unpacked beside its final place and swapped in only once its
  `capabilities.xml` reads without error and a `theme.xml` loads; a failed, empty, invalid or cancelled download leaves
  the old theme as it was and no stray file. An update keeps `theme-customizations/` byte for byte. Removal refuses any
  folder Mistress did not download.
- **P55, a closed sheet stops its download.** Closing the sheet during a download cancels the request within one pass of
  the dispatcher and leaves no partial file; a test that fails without that cleanup proves it.
- **P11, the archive** (§6's, retired here): Art Book Next's archive is 205–230 MB.

### 16.2 The variant triggers against Mistress's media: P52 and P53

Stage (a) evaluated the triggers against a `MediaPresence` its caller supplied (§12.7), and stage (e)'s host built one
by asking `ISceneMedia.Find` for every game, media type and extension in turn. Two things were wrong with that, one
predicted and one measured.

- **P52 holds: the defect was real.** `EsdeMediaFolder` looked for `.png`, `.jpg` and `.jpeg` whatever the type, so no
  video was ever found. `The_gamelist_variant_follows_the_media_the_library_has` gives a synthetic theme a variant with
  both triggers, and every SNES game a cover and an `.mp4`; on the unchanged reader the gamelist took `novideo`, the
  `noVideos` override, where the chosen variant was due. The extensions are now `USERGUIDE.md`'s ("Manually copying game
  media files"): `.png`, `.jpg` and `.webp` for pictures, six for videos. `.jpeg` is dropped, because ES-DE would not
  read such a file and a trigger that found it would disagree with the oracle. The same test then takes the videos away
  (`novideo`), the covers (`bare`, the `noMedia` override, which takes precedence) and adds one `.webp` cover back
  (`novideo`), each at the next showing.
- **The showing did not see a change.** The host kept each system's presence for as long as its games and the media key
  (the folder's path and the art index's identity) were the same, so a file added to the media folder was not seen until
  a restart. The key now carries each type folder's last write time, which changes when a file is added to it or taken
  from it; a showing costs ten `stat` calls per system for that.
- **P53 holds.** Presence is now one listing per type folder, stopping at the first file of a game in the list.
  `A_full_media_folder_is_listed_once_rather_than_asked_game_by_game`: 3,508 games with a cover each and a screenshot
  for every other one, 5,262 files, took **167.9 ms asked game by game and 0.8 ms listed**; the two answers are equal,
  as `Presence_from_one_listing_equals_presence_asked_game_by_game` also requires on a folder holding strays (a file of
  no game, an extension of neither list). §15.8's test, with an empty media folder, went from 189 ms (re-measured on the
  unchanged build; §15.8 recorded 224) to 3 ms.
- **The cover from the art folder** (§4.33 of the settings reference) still counts for the cover type, asked game by
  game, because that index answers by title, not by file; the ask stops at the first game that has one.

**What a trigger means here, and where it is Mistress's choice.** THEMES.md says a trigger fires when "no game media
files are found for a system", so a type counts once any game of the system has it; the loader always read it that
way. Videos count as present when their files are there, although Mistress draws a video element as its image (§10.1,
Q4): that is what ES-DE does with the same files, and a theme's `noVideos` variant exists for a system without them, not
for a player whose frontend does not play them.

### 16.3 The settings sheet: P46–P49

`ThemeSettingsWindow` is a `ToolWindow` of LunaP controls (`Tabs`, `FieldRow`, `Dropdown`, `EmptyState`, buttons),
presented as a `SheetLayer` sheet; the pad menu offers it in big-screen sessions outside a game, and Preferences ▸
Appearance has a button for it. Nothing new was needed in LunaP for it. The rows and their rules are §4.53 of the
settings reference.

- **The choices live in `appsettings.json` under `BigPicture`,** keyed by the theme folder's full path (§6 had said
  "under `BigPicture`"; keying by folder is this stage's choice, since two copies of one theme in two folders are two
  themes to the player). `ThemedLibrary.Show` takes them as a `ThemeChoices` and keys its cache of resolved themes by
  the whole record, so a choice builds a new theme and the old one is kept for a return.
- **P46 holds.** `A_choice_applies_at_once_beneath_the_sheet` enters the SNES gamelist at its third game, opens the
  sheet, and chooses a scheme, a variant, a font size and an aspect ratio. After each choice, with the sheet still
  presented, the stage beneath has been rebuilt with the new selection at the same system and game; after the sheet is
  closed, the window's frame and a fresh `SceneBuilder.Build` of the same data differ in **0 pixels**.
- **P47 holds.** Two synthetic themes keep their own choices in the file; a stored variant the theme does not declare
  (`gone`) falls back to the first selectable one with the theme still themed and no error.
- **P48 holds.** On Art Book Next the sheet lists the 20 variants in declared order under their `en_US` labels (the
  first, "List: Metadata & Boxart", selected, as §12.5's default), 31 schemes, Medium, Large, Small and Extra Large,
  Automatic and 12 ratios, Automatic with its two profiles "Instant" and "Slide" (it suppresses the three built-ins), and
  no language row.
- **P49 holds.** `PadAudit` reaches every control of both tabs and of the About sheet at 1280×800.

### 16.4 Themes downloaded, updated and removed: P54, P55 and P11

`ThemeDownloads` (`BigPicture/`, no Avalonia types) fetches GitHub's archive of a branch, `ThemeAttribution` reads a
theme's README, and the sheet's Themes tab drives both. The rules are §4.53 of the settings reference; the record is
here. The class was first named `ThemeStore`, which a WiseMan test fixture of that name in an enclosing namespace
shadowed; it was renamed rather than the fixture.

- **P54 holds** against a fake GitHub (`OnlineCoverTests.FakeServer`, answering the commits API and codeload by URL).
  `ThemeDownloadsTests` downloads and stamps a synthetic theme zipped as GitHub zips (one folder named for the repository
  and branch); updates it, with `theme-customizations/` holding two files of 5,000 bytes each that come through equal
  byte for byte, while the archive's own `theme-customizations/upstream.txt` is discarded; and refuses five broken
  downloads (a server error, no `capabilities.xml`, a malformed one, a `theme.xml` that does not load, an entry named
  `../escaped.txt`), each leaving the old theme's marker and stamp and nothing beside the folder. Removal refuses a
  folder read in place and a look-alike under `home/Themes` without a stamp.
- **The order of the swap is the rule that keeps the player's files.** The new folder is moved in before the
  customizations are moved across from the old one, and the old one is deleted only after that; a swap cut short leaves
  `.old`, which the next download restores or empties before anything else runs. The other order, moving the
  customizations into the staged folder before the swap, is the obvious one and is unsafe: the `finally` that cleans the
  staged folder would delete them on any failure in between. That is argued, not tested; no test reproduces a failure in
  that window.
- **P55 holds, and needed two cleanups.** The download's `CancellationTokenSource` is cancelled when the sheet closes.
  `Closing_the_sheet_or_its_window_stops_the_download` uses a server whose archive sends one byte and then waits until
  the request is cancelled. Closing the sheet with East: the download ends cancelled, and no `.zip.part` or `.part` is
  left. **Without the sheet's cleanup that case fails** ("the download was still running after its sheet closed", 3 s).
  Closing the main window instead **also fails without a second cleanup in `CloseThemedLibrary`**, because a sheet is not
  closed when the window presenting it is: its `Closed` never fires. The window now stops the sheet's download too.
- **P11 holds.** `git archive --format=zip` of the reading clone at `d772d07`, the tree GitHub's codeload serves for that
  commit, is **220,158,364 bytes (220.2 MB)**, inside 205–230 MB. That is a local measurement of the same tree, not of a
  download: codeload's compression level was not observed, since nothing was downloaded without a player asking.
- **The attribution** is read at display time from the theme's `README.md`: the section under the first heading whose
  words contain "licen", and the one containing "credit", with Markdown's marks taken off; else a `LICENSE` file's first
  lines; else a sentence saying none is stated. For Art Book Next the licence line is its README's own, naming
  CC-BY-NC-SA 2.0 and its URL, and eight credits. The author of a downloaded theme is its repository's owner, since
  neither `capabilities.xml` nor THEMES.md has a field for one; a theme read in place states no author.

### 16.5 The grid: ES-DE measured, then built

**The measurement.** ES-DE 3.4.1's grid was measured from its behaviour alone, with §14.7's rig (XWayland
window, uinput pad, 165 fps lossless recording): 26 runs, one window at a time, each under 25 s and closed by PID, of a
synthetic `grid-probe` theme with 16 variants that each change one property, on synthetic libraries of 40, 250 and 37
games whose covers are flat single hues. ES-DE's source was not read; only `THEMES.md` and `USERGUIDE.md`. The report
and every per-run CSV are `~/.cache/emusen/bigpicture/grid/results.md` and `results.json`, outside the repository.

**What it found, as rules** (W×H the element box, w×h the item, sx×sy the spacing, k the scale):

| Rule | Measured |
|---|---|
| Columns | `floor((W + sx − w(k−1)) / (w + sx))`, as many as fit with the selected item scaled; without the `w(k−1)` term under `scaleInwards`. Four variants built to reject the rival rules each drew what this rule predicts |
| Placement | left-aligned, the first column at `w(k−1)/2` (0 inwards), everything left over on the right |
| Rows | first at `h(k−1)/2`, whole rows `floor((H + sy − h(k−1)) / (h + sy))`; `THEMES.md`'s "snapped to the item height multiplied by itemScale" mispredicts the count once the spacing is large |
| Omitted spacing | half the selected item's growth on each axis |
| Scroll | `max(0, row − (V − 1))`: nothing until the selection passes the last row shown, then the selected row stays on the bottom row, both ways. The rule "scroll when it leaves the rows" was refuted by a six-row layout |
| Inward anchors | outer edges fixed on the first and last columns and the first row, the bottom edge on a scrolled bottom row |
| Unfocused | true alpha (0.2983 for 0.3, n=36); dimming 0.5 with saturation 0 gives half of **Rec. 601** luma |
| Item and row motion | quadratic ease-out, 251.2 ms (n=10, rms 0.0014) and 249.8 ms (n=5); the same frame, whatever the distance; each step restarts from the current values, only two items moving |
| Holds | 497 ms, then every 200 ms, on both axes, no faster tier in 8 s |
| Ends | across a row's end to the next row; a tap wraps at the list's ends, a hold stops; up and down stop at the first and last rows; down into a short last row takes its last item |
| Metadata | fades out over 150 ms from the first repeat and back in over 150 ms, as the text list's does (§14.7) |
| No image | the text background fills the item, the name centred |

**What was built.** LunaP's `ImageGrid` and `GridGeometry` (its §104) hold the layout and draw a state given by a
scroll and a focus progress; the scene's `GridElements` maps 47 of the grid's properties to it; `SceneView` moves it by
the rules above, with `SceneMotion.Esde` gaining `GridStep` (250 ms), `GridEasing` (quadratic ease-out) and
`GridRepeat` (500, then 200 ms). In a grid all four directions move the selection, so a grid's gamelist has no quick
system select on left and right; the shoulders page by the whole rows shown. That is this stage's choice, since ES-DE's
own quick system select was never measured (§15.3).

**One correction reached LunaP.** ES-DE's greys fit Rec. 601 weights; LunaP's `FittedImage` had used Rec. 709 since its
§98.2. The weights are now Rec. 601 (LunaP §104.4), which changes every desaturated picture, the carousel's included:
Art Book Next's schemes set `imageSaturation`, so its system view's pixels changed in the desaturated schemes. No
EmuSen test compares them with ES-DE, so the change is argued from the grid's measurement, not measured on the carousel.

**The mapping.** Of the 172 pairs Art Book Next sets, the scene now reads **164** (140 before). The grid's 24 are all
read; §13.1's P21 had said 161 if the grid were built, and the difference is stage (c)'s four time-governed pairs. The
eight left are those §13.3 and §14.9 named: the video's five playback and interpolation pairs, the sound's path, and the
badges' `controllerSize` and `folderLinkSize`. Every grid pair, and the four common ones, has a differential case in
`SceneMappingTests`: 297 cases now, from 247.

**P50 fails, by a pixel.** `Art_Book_Next_s_grid_matches_ES_DE_s_still` renders `gamelist-grid-cover` at 1280×800 on
the SNES games ordered as ES-DE orders them, and finds each synthetic cover's flat interior with the same ratio test the
ES-DE still was read with. Ten of eleven covers lie within 1 px of ES-DE's; the selected cover and one edge of one
unfocused cover lie 2 px off, at rest and after two rows down. ES-DE draws item edges on whole pixels, and Mistress at
fractional ones (§13.8); a 1.2 scale of a fractional box is where the second pixel comes from, argued, not shown. The
covered cover (Ivory Signal, under the selected one) is left out, as the measurement's table marks it.

**A defect the comparison found, in LunaP.** The first run put the selected cover on the scrolled bottom row 22 px low:
`GridGeometry.Anchor` compared two quantities equal by construction to a millionth, and float noise fell on the wrong
side, so the cover scaled about its centre. LunaP §104.4a records it.

### 16.6 Mutants

The runners are `~/.cache/emusen/probe/bigpicture/mutate_stage_f.py` (Mistress, 47 mutants) and `mutate_grid_lunap.py`
(LunaP, 16, on a copy); logs `run-stage-f.log`, `run-stage-f-grid.log`, `mutants-stage-f*.txt`. Each mutant was built and
run alone, under `nice`, against the tests of its area only, and the source restored; the tree was rebuilt clean after
each round.

| Area | Mutants | Caught at once | Survived, then caught |
|---|---|---|---|
| Settings sheet (F1–F12) | 12 | 12 | — |
| Downloads and attribution (F13–F21) | 9 | 8 | F21, a cancel while unpacking ignored |
| Triggers (F22–F26) | 5 | 4 | F25, a changed media folder not seen |
| Grid in the scene (G1–G21) | 21 | 14 | G2, G7, G14, G15, G18 |
| LunaP grid (LG1–LG16) | 16 | 16 | — |

**Every survivor was a weak test, and two exposed real defects.**
- F21: no test cancelled during extraction. `A_download_cancelled_while_it_unpacks_leaves_the_old_theme` now does, through
  an unpack progress the sheet also shows.
- F25: the trigger test deleted whole media folders, which a folder's existence reveals without its write time. It now
  empties them.
- G2: the held test ran long enough for a wrapping hold to come round to the same item. It is shorter now.
- G14: made the page ten items, and survived because the pad test's page happened to equal one row. Making the test
  page by two rows exposed **a defect**: after an up or down, a page moved by one row, because `Jump` read the held
  direction's axis. `Step` and `Jump` now always move across. G21 is that defect as a mutant, caught.
- G7: nothing checked a step taken while the last still moved. The new test exposed **a defect**: the new item's
  starting focus was computed from the old item's level after that level had been overwritten. Both levels are now
  taken together; G20 is the defect as a mutant, caught.
- G15 and G18: no test looked at a `-1` item axis or at the axis corner radii are measured on; one does now.

As in stages (b) to (e), the reference test caught little on its own: among the failing tests the log lists for the 21
grid mutants, `Art_Book_Next_s_grid_matches_ES_DE_s_still` appears only for G10 (a vertical press moving one item). A theme that renders is weak evidence that its mapping is right.

### 16.7 Predictions retired

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P11 | Art Book Next's archive is 205–230 MB | 220.2 MB, by `git archive` of the reading clone at `d772d07` | held (a local measurement of the same tree, not a download) |
| P46 | a choice redraws the view beneath the sheet, equal to a fresh build | rebuilt at the same system and game with the sheet open; 0 pixels differ | held |
| P47 | choices kept per theme; a stale one falls back without error | as predicted | held |
| P48 | Art Book Next: 20 variants in order, 31 schemes, 4 sizes, Automatic and 12 ratios, no language row, two profiles | as predicted | held |
| P49 | every control of the theme sheets reachable | 0 unreachable on both tabs and the About sheet | held |
| P50 | the grid's first two rows within 1 px of ES-DE's still | 2 px worst: the selected cover and one unfocused edge; the rest within 1 | failed, by a pixel |
| P51 | 400 ms quadratic ease-out for items and rows; 500 then 114 ms repeats with no jump | quadratic ease-out, as predicted, but 250 ms; 500 then 200 ms with no faster tier | the curve held; the duration and the interval refuted |
| P52 | videos never found, so `noVideos` fires wrongly | shown by a test on the unchanged reader, then fixed | held (a defect) |
| P53 | the 3,508-game scan under 20 ms by listing | 167.9 ms asked game by game, 0.8 ms listed, on a full folder; 189 to 3 ms on an empty one | held |
| P54 | a download swapped in only when it loads; broken or cancelled ones leave the old theme; customizations kept; removal refused for folders not downloaded | as predicted, against a fake GitHub | held |
| P55 | closing the sheet cancels a download, proved by a test that fails without the cleanup | held, and needed a second cleanup in the window, since a sheet is not closed when its window is | held |

### 16.8 Not done in stage (f)

- **Nothing ran on the handheld.** The grid's frame cost there, and whether 250 ms steps read well at arm's length, are
  the device's.
- **Only Art Book Next can be downloaded from the sheet.** Another GitHub or GitLab theme, and ES-DE's theme list as a
  picker (§6), are not built. Nothing was downloaded from GitHub: every download test used a fake server. *Retired
  2026-09-26 by §25, which built the list as a browser and generalised the download to every listed theme.*
- **The grid's unmeasured parts.** `imageFit contain` and `cover`, selector and background images, corner radii and the
  text's scale law were not measured in ES-DE; they follow `THEMES.md` and are proved only to change the pixels. The
  horizontal clip, a held right that reaches the last item (inferred to stop, as the others do), 60 Hz timing, and the
  unscrolled inward bottom row's centring were not measured either.
- **The grid's own not-mapped properties:** `imageBrightness`, the background's and selector's gradients, the selected
  image's gradient, `textHorizontalScrolling` and its three siblings, the collections' letter cases and
  `fadeAbovePrimary`. Art Book Next sets none of them.
- **Quick system select in a grid gamelist** does not exist: all four directions move the grid. What ES-DE does there
  was not measured.
- **Languages** are listed only when a theme declares them; no theme installed here declares any, so the row was tested
  on a synthetic theme alone.
- **The desaturation change to Rec. 601** was measured on the grid only; the carousel's desaturated schemes were not
  compared with ES-DE.
- **The carousel test flake.** One run of the scene's motion tests failed
  `A_carousel_step_eases_to_the_next_item_and_settles_on_the_static_picture` in under a millisecond, and passed on the
  next run with nothing changed in between. It was not reproduced and is not explained; it is recorded, not attributed.
- **The one broad run failed two unrelated tests.** The Mistress filter without the three GPU classes ran 622 tests:
  617 passed, 3 skipped, and `InputSettingsWindowRenderTests.The_window_renders_its_rows(NES)` and
  `FrameHandOffTests.Once_a_session_ends_the_picture_left_on_screen…` failed. Both classes passed alone (11 of 11), and
  again beside every class this stage added (47 of 47). Unrelated tests failing only in the broad run is §15.14's
  pattern; whether anything of stage (f) causes it was not established, because repeating the broad run until it
  reappears is ruled out by the load rule of 2026-09-25. The failure messages were not captured.
  *Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78, by timing only, since no message was kept: the run
  came after `a98068ef` added `GridSceneTests` with seven plain facts that built scenes off the headless session's
  dispatcher, which makes other tests' start-ups fail (§4.78.2). `FrameHandOffTests.Once_a_session_ends…` also asserts
  on a weak reference, which could fail for another reason; that was not ruled out.*


## 17. Stage (d): ScreenScraper, the quota, the queue and the media store

*Opened 2026-09-26, after the developer credentials arrived (§10.1).* Stage (d) builds §5: the client for
`jeuInfos.php` and the media it names, the quota rules of §5.5, a queue that resumes, the media store of §5.6 and its
`media.db`, the region and language rules of §5.4, and the settings. §10.1's answers bind it: builds carry no developer
credentials (Q5), and the miximage is ScreenScraper's `mixrbv2` (Q7). Stage (f) was being built on another branch at
the same time, so this stage's predictions are numbered from P60, leaving P46–P59 to it.

### 17.1 Predictions, written before the client was built

P9 and P10 (§9) are this stage's. The rest were added before a line of the client existed.

- **P60, N64 byte order.** ScreenScraper's `rommd5` for an N64 game is the big-endian (`.z64`) order's, so step (1) of
  §5.2 finds this library's `.z64` files and step (2), whose transform gives the halfword-swapped order, finds none of
  them. OpenVGDB wanted the swapped order (§4.39 of the settings reference); ScreenScraper, whose ROM lists come from
  No-Intro's DATs, is predicted not to.
- **P61, media do not count against the quota.** `requeststoday` rises by one for each `jeuInfos` and by nothing for a
  media download, which `mediaJeu.php` serves from another host. §5.5 left this open and priced both cases.
- **P62, the quota fields come without a member account.** A `jeuInfos` response made with the developer credentials
  alone still carries an `ssuser` block with `maxthreads`, `maxrequestspermin`, `maxrequestsperday` and
  `maxrequestskoperday`, as §5.5 cites for every response, and its `maxthreads` is 1.
- **P63, the time a game costs on one thread.** A found `jeuInfos` takes 1.0–2.0 s and an unknown one 0.4–1.0 s (the
  site's own figures were 1.52–1.63 s and 0.51–0.76 s, §5.1). A found game with its default media (cover, screenshot,
  marquee and miximage) costs 3–14 s in all.
- **P64, the system IDs.** `systemesListe.php` gives NES 3, SNES 4, Game Boy 9, Game Boy Color 10 and Nintendo 64 14,
  as §5.3 cites from ES-DE's table.
- **P65, the pacing never binds on one thread.** With the quota's `maxrequestspermin` at 50 or more less §5.5's 10%, one
  thread whose requests each take a second or more never waits for the pacer on the live run.
- **P66, the region rule.** Of the found games whose file name carries a USA, Europe or Japan tag, at least 80% get a
  cover of that region; the others take the fallback order's first region with one.
- **P67, nothing leaks.** After the live run, the literal values of the developer credentials appear in no file the run
  wrote: not the media folder, not `media.db`, not the saved responses, not the log.

### 17.2 The scope, as it changed during the stage

The plan was §7's row: scraped media as one more source for the themed view. Two decisions came during the build,
and are recorded here because they reverse parts of §5:

- **ScreenScraper became Mistress's main source of cover art and game text everywhere**, the library's grid and list as
  well as the themed view, whenever the developer file is present. OpenEmu's sources (§4.39 of the settings reference)
  became an opt-in failover, **on by default by decision**, which the player may reverse. It fills a cover
  only where ScreenScraper found none or cannot be used.
- **Ship-ready by default.** Where the developer file exists, scraping works with no action, and it is reachable from the
  pad in Game Mode. *Reversed the same day by the rule of 17.14: nothing is asked until the player starts a run.* `Scraping` is therefore on by default. §5.8's "off by default, one explicit action" is kept in the one
  place it still protects someone: a build without the developer file sends nothing to ScreenScraper, and there the
  failover is what runs, with its hint saying what it sends. That the failover itself is now on by default is a change
  from §4.39's "off unless the player turns it on", and is a decision taken during the build, not an argument made here.

The order of sources was given with the second decision: what the player placed, then ScreenScraper, then an ES-DE
media folder, then the failover, then the placeholder. Stage (e) had drawn the ES-DE folder before the player's covers;
that order is reversed here.

### 17.3 What was built

**In Mistress, `Scraping/`**, with no Avalonia type:
- `ScreenScraperClient`: the URLs (credentials, `output=json`, `romtype=rom`, `systemeid`, `crc`, `md5`, `sha1`,
  `romtaille`, `romnom` as the bare file name), `jeuInfos`, `ssuserInfos`, `systemesListe`, and the media download with
  §5.6's rules (an image of 80 bytes or more, written beside its name and moved, never over a file). `StatusOf` gives every
  code of §5.1's table its own meaning. `ScreenScraperJson` is every reading of the answers in one place, because the API
  is a beta (§10's risk); it takes numbers written as strings or as numbers.
- `ScrapeQuotaManager`: §5.5's rules (17.4).
- `MediaStore`: `media.db` (17.5).
- `Scraper`: the queue's workers (17.4).
- `ScrapeRules`: region, language, genre, rating and date (17.6).
- `MediaSources`: the order of sources, which also answers stage (f)'s presence and stamp questions for the variant
  triggers (§16.2).
- `RomHashes`: MD5, CRC-32 and SHA-1 from one read; the MD5 equals `RomHash.Md5` of the same file.
- `ScreenScraperCredentials` (`DeveloperCredentials`, `MemberAccount`) and `ScrapeRedactor` (17.7).

**In the window** (as first built; 17.14 replaced the automatic parts): `MainWindow.Scrape.cs` starts the workers when the
setting is on and the developer file is found, and again whenever Preferences closes; routes a tile without a cover to
ScreenScraper or the failover; queues the themed gamelist's selected game; feeds the themed view's `SceneGame` metadata from `media.db`; and refreshes the grid and the
view when media arrive, the view only when it is still (`SceneView.NextChange` is null), so a glide is never cut. The
failover (`MainWindow.Covers.cs`) now starts on the first cover wanted from it, writes to `home/Media/openemu/`, and is
silent in the status line unless the player asked. Preferences gained a **Scraping** tab (`ScrapePreferencesPane`).

**In LunaP** (branch `bigpicture-scrape`, `docs/LunaP.md` §110): the on-screen keyboard draws a password box's preview
in the box's mask.

**Elsewhere:** `DataStore.Media`; `CrashLog` writes through the redactor; `ArtworkIndex` locks its dictionary, since the
worker asks it whether the player has a cover while the window may be adding one; `.gitignore` names `media.db` and both
credential files.

### 17.4 The client, the queue and the quota rules

**Identity** is §5.2's, stopping at the first hit: the file's own three hashes with its size, name and system; then, only
after a 404 and only when the core's `OpenVgdbBytes` changes the bytes, the transformed bytes' hashes with their own size.
The Game Boy's transform is the identity, so a Game Boy file is never asked twice. There is no name search.

**The queue** lives in `media.db` (`scrape_queue`), ordered by priority (Shown, Console, Library) and then arrival; a game
shown moves up, never down. A worker holds a game while it asks, so two workers never take the same one. Outcomes:

| Answer | What happens to the game |
|---|---|
| found | its text and each wanted kind kept; off the queue |
| 404 at both steps | recorded Unknown; off the queue; never asked again |
| 400, or a malformed answer or a network failure five times | recorded Error; off the queue |
| a malformed answer or a network failure before the fifth | stays, retried after 2, 4, 8… minutes, at most an hour |
| 429, 401 | stays, retried after the wait below, the attempt not counted |
| 430, 431, 403, 423, 426 | stays, untouched, for the day or the next start |

A game answered before costs no request: an Unknown one is not asked again, and a found one is asked again only for a
kind the player has since turned on **and** the game offered (`scrape_game.offered` lists the kinds its answer had, never
their addresses, which carry the credentials). A renamed file is recognised by its MD5 and its media are moved to the new
stem; a second copy beside the first gets copies. §4.37's orphan pass over `games.db` is not extended: the scraper's own
path cache (`scrape_file`) is what finds a renamed file, at the cost of hashing it once.

**The quota rules**, all from §5.5:
- the limits and counts are read from every answer that carries them, and a server's count replaces Mistress's own;
- workers: never more than `maxthreads`, one until an answer says; a worker whose number is no longer allowed idles;
- pace: `maxrequestspermin` less 10%, media downloads included, 30 a minute until an answer says (under the 50 per
  thread the secondary sources report);
- `maxdownloadspeed`: after a file that came faster, the difference is waited;
- the day stops at `maxrequestsperday` less 2%, or `maxrequestskoperday` less 2%, or on 430 or 431, until midnight in
  Paris; the stop is written to `quota_day` and survives a restart;
- 429 halves the pace and waits 60 s; 401 waits five minutes; 403, 423 and 426 stop until the next start or change in
  Preferences, which is when the worker is rebuilt.

*Where this is Mistress's choice, not ScreenScraper's documented rule:* the day's boundary (Paris time; when
ScreenScraper resets was not measured); the 2% applied to the unrecognised allowance as well as to requests; pacing the
media downloads with the requests (whether they count against the quota is P61, measured below); the assumed 30 a
minute; five attempts; and a 403 stopping only the session, since a corrected developer file should work without waiting
a day.

### 17.5 The media store and `media.db`

The files are §5.6's layout, `home/Media/<system>/<folder>/<rom stem>.<ext>`, so the store is read by stage (f)'s
`EsdeMediaFolder` unchanged: whatever that reader accepts (png, jpg, webp, and its video types) the store's folders are
read with. `media.db` has one migration so far, with `PRAGMA user_version` and a newer file refused:

| Table | Key | Holds |
|---|---|---|
| `scrape_game` | MD5, size | status (Found, Unknown, Error); ScreenScraper's game, ROM and system ids; name; description; developer; publisher; genre; players; rating; release date; the region and language used; which step matched; the kinds offered; the detail of an error; `fetched_at` |
| `scrape_media` | MD5, size, kind | the file relative to the store; its region; its SHA-1; `fetched_at` |
| `scrape_file` | path | size, write time and MD5: a file is hashed again only when it changes |
| `scrape_queue` | path | system, priority, arrival, attempts, `next_try` |
| `quota_day` | day | requests, unrecognised, the day's limits, `stopped_until` and why |

§5.6 listed four tables; `scrape_file` is the fifth, because §5.6 left open how a path finds its MD5 without reading the
file at every showing.

### 17.6 Region, language and text

- **Region.** The preferred region is the player's choice, or the file name's first tag whose every name is a country
  (`(USA, Europe)` is `us`; `(En,Fr,De)` is not a region), or `wor` when there is none. With the fallback on, then
  ES-DE's order, `wor`, `us`, `eu`, `jp`, `ss`, then any region; with it off, the preferred region only. A file with no
  region is taken when no region in the order has one. For marquees `wheel-hd` is tried whole before `wheel`.
- **Language**, for the synopsis and the genre: the preferred, then `en`.
- **Name**: by the region order, then ScreenScraper's own (`ss`). Kept, not shown: the library and the themed view show
  the file's name, as they did.
- **Genre**: the main one (`principale`), else the first, in the language order.
- **Rating**: `note / 20` to the nearest tenth, as §5.4 says, clamped to 0–1.
- **Release date**: by the region order; a full date, a year and month, or a year.

### 17.7 Credentials, as §5.7 asked

- **Where the developer file is read.** `ConfigStore.Directory` first, then `ConfigStore.LegacyDirectory`
  (`~/.config/EmuSen`, where the file was put). A test that moved the config directory without moving the legacy one
  never reaches the real file, the same guard `ConfigFile` uses (§1.4 of the config reference). For a published tree the
  first place is `<tree>/home/etc/EmuSen/screenscraper-developer.json` (`out/linux-x64/Mistress/home/etc/EmuSen/`, or
  `~/Apps/Mistress/home/etc/EmuSen/` for an installed copy); the second serves every tree on the machine.
- **Q5 as decided.** Builds carry nothing; the embedded-resource option was not built.
- **The member account** is `screenscraper.json` in the config directory, created with mode 0600 (`UnixCreateMode`) and
  set to it again before it is moved into place. It is written when a box is left or the sheet closes, and only when it
  changed, so no half-typed password is ever written or registered with the redactor.
- **The redactor.** One function blanks `devid`, `devpassword`, `ssid` and `sspassword` in any text (§5.7 named three;
  `devid` was added because the developer credentials as a whole were to stay out of logs), and blanks each credential's
  value wherever it appears once the credential object has been made. Everything that reaches the status line, a detail,
  an exception message or `CrashLog` passes through it. The media addresses in every `jeuInfos` answer carry the
  credentials in their query, so no answer is ever logged unredacted; the live tool saves only redacted bodies.
- **Never in git.** `.gitignore` names both files. `ScrapeCredentialTests` fails if `git ls-files` lists either, or if a
  tracked file holds a `devpassword=` value that does not start `FAKE` (the tests' spelling), or holds the real password
  when the developer's file is on the machine. The test found its own first draft: the redactor's cases had used a
  made-up password that did not start `FAKE`. It fired a second time on this section, whose first draft quoted that
  case literally.
- `DeveloperCredentials` and `MemberAccount` print as a description, never their values.

### 17.8 Closing what was opened (§15.14's lesson)

The workers, `media.db`, the view's refresh timer and the window's handler in Preferences are the things this stage
opens. `StopScraping`, called from the window's `Closing` before the HTTP client is disposed, marks the window closed,
stops the timer, cancels and joins the workers (a request in flight is cancelled with them), and closes `media.db`.
`A_closed_window_asks_nothing_more_and_closes_its_worker_and_its_store` holds a request open at the fake server, closes the
window, releases the server and waits 0.8 s: no request follows, the worker reports not running and the store closed.
With `StopScraping` removed from `Closing` (mutant D40, 17.11) the test fails. Preferences removes its handler from the
window when it closes (D41), and saves the member account then (D42). The suite's windows start with a handler that
refuses every request and with no developer file (`NoNetwork`, a module initialiser), so no window in any other test can
reach a server or read the developer's file.

### 17.9 The live run (2026-09-26, 13:48, desktop)

`ScrapeLiveTool.Forty_random_files` (run with `EMUSEN_SCRAPE_LIVE=1`) chose 40 files at random (seed 20260926) from the
library's 5,520, read in place and never written; ran the real client, one thread, no member account, the default kinds
(cover, screenshot, marquee, miximage); and wrote the media, `media.db`, every answer (redacted) and its log to
`~/.cache/emusen/bigpicture/scrape-live/20260926-134821/`, outside both repositories. Beforehand it asked
`systemesListe.php` and `ssuserInfos.php`.

**The draw.** 23 NES and 17 Game Boy files; no SNES, N64 or Game Boy Color file was drawn, since the library is 64% NES.
Many were hacks, translations and multicarts ("SMB1 Hack", "[T-Eng]", "68-in-1").

**Identification.** 39 of 40 found, every one by the file's own hashes; step (2) was asked once (the one unknown NES
file) and found nothing. By shelf: NES 22 of 23, Game Boy 17 of 17. The unknown file is a dump with no name
(`ZZZ_UNK_…`). ScreenScraper knew the hacks and translations that OpenVGDB had missed in §4.39's measurement.

**Media.** 144 files, 47.4 MB: 33 covers (631 KB on average), 36 screenshots (5 KB), 36 marquees (88 KB) and 39
miximages (567 KB). Six found games had no `box-2D`: four SMB1 hacks, two multicarts and a Minolta program cartridge.
Those are the games the OpenEmu failover is for.

**Time.** 599 s in all. A found `jeuInfos` took a median 1.17 s (0.70–12.47 s); a 404 a median 1.04 s (0.89–1.04). A
found game cost a median 13.3 s (2.6–18.7), an unknown one 3.3 s. Two `jeuInfos` requests, both for multicarts, timed out
at the tool's 60 s and were retried by the queue two minutes later, when they answered.

**Waiting.** The clock recorded 129 waits totalling 294 s. They were the download-speed rule, not the per-minute pacer:
the answers gave `maxrequestspermin` 3,072, so the pacer's interval was 0.02 s after the first answer, while
`maxdownloadspeed` was 128 KB/s, at which 47.4 MB takes 362 s. Half of the run's time was the bandwidth the account is
allowed.

**The quota.** Every JSON answer (39 of 43; the others were two 404s' text and two timeouts) carried `ssuser`, with no
member account: 1 thread, 128 KB/s, 3,072 a minute, **10,000 requests a day and 1,000 unrecognised**. `requeststoday`
went from 0, in the first found answer, to 180 in the last, over 43 `jeuInfos` and 144 media: **media downloads count
against the day's requests.** The 404s did not move `requestskotoday`, which stayed 0. `ssuserInfos.php` answered
"Erreur de login" without a member account, before and after, so it cannot be the source of the quota for a player
without one. The `jeuInfos` answers are, and that is how the Scraping tab's meter reads it.

**The system IDs.** `systemesListe` names 3 NES, 4 Super Nintendo, 9 Game Boy, 10 Game Boy Color and 14 Nintendo 64.

**Nothing leaked.** No file in the run's folder, the test's log, either working tree, the scratch folder or the mutant
runner's folder holds the developer's password or `devid=` followed by the id (4,038 files scanned by value, the values
never printed).

### 17.10 Predictions retired

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P9 | 75–95% by step 1; step 2 at most 5 points | 39 of 40 (97.5%) by step 1; step 2 none | **failed high**: ScreenScraper lists the hacks, translations and multicarts that this library is full of |
| P10 | the whole library, no videos, in two or three sessions over two days | not run whole. Projected from the run: 4.7 requests a game (43 + 144 over 40) makes 5,520 files about 25,900 requests, against 9,800 a day (10,000 less 2%), so **three days**; about 19 h on one thread at 12.5 s a game, most of it the 128 KB/s allowance | **refuted by projection**. §5.5 assumed 20,000 a day; a developer-only account has 10,000. Covers alone (2 requests a game) would take two days |
| P60 | N64's `rommd5` is the `.z64` order's | no N64 file was drawn | **not measured**; four `.z64` files, about 20 requests, would settle it |
| P61 | media do not count against the quota | `requeststoday` 0 → 180 over 43 `jeuInfos` and 144 media | **failed**: each media file is a request |
| P62 | quota fields without a member account; `maxthreads` 1 | in every JSON answer; `maxthreads` 1 | **held** |
| P63 | found `jeuInfos` 1.0–2.0 s; unknown 0.4–1.0 s; 3–14 s a found game | median 1.17 s (outliers to 12.5 s); median 1.04 s; median 13.3 s, 17 of 39 over 14 s | **partly held**: the found median held; the unknown's median missed the band by 0.04 s; the per-game bound failed, because the pictures are 2–6 times §5.5's 100–500 KB and the account's bandwidth is 128 KB/s |
| P64 | 3, 4, 9, 10, 14 | as predicted | **held** |
| P65 | the pacer never binds on one thread | its interval was 0.02 s and never bound; the download-speed rule waited 294 s of 599 | **held** as stated; the rule it did not name is the one that binds |
| P66 | ≥80% of tagged games get their region's cover | 12 of 16; the four others had no box in their region and took the fallback order's first (Europe → China twice for two Sachen carts, USA → Europe, Europe → USA) | **failed** on the share, held on the fallback |
| P67 | no credential in any file written | none | **held** |

### 17.11 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/mutate_stage_d.py` with its list `mutants-stage-d.json`. The logs are
`mutants-stage-d.txt` (every result, appended) and `run-stage-d.log`. Each mutant was built and run alone under
`nice -n 10` against the scraping, crash-log and cover tests; LunaP's against its keyboard tests. The source was
restored and checked byte for byte after each.

| # | Mutant | Result |
|---|---|---|
| D1–D3 | the redactor misses `sspassword`; ignores a credential's own value; `CrashLog` bypasses it | caught |
| D4–D10 | `romnom` with its folder; a half-set member account sent; 430 read as a network failure; 426 as too many requests; a page that is not an image kept; no smallest size; a file already there asked for again | caught |
| D11–D19 | the pace without its 10%; threads above `maxthreads`; the day stopped at its limit, not 2% short; the unrecognised allowance ignored; 429 not halving the pace; 401 waiting one minute; 430 forgotten at a restart; 403 not stopping; the day ending at UTC midnight | caught |
| D20–D27 | step 2 asked when the transform changes nothing; step 2 never asked; the player's cover not sparing the fetch; an unknown game asked again; a stop dequeuing the game; a renamed file's media copied; media downloads not paced; a 404 not counted as unrecognised | caught |
| D28 | a kind the game never offered asked for again | **survived**; caught after a test was added |
| D29–D33 | the file's region tag ignored; `wheel` before `wheel-hd`; any region without the fallback; the note out of 10; English before the chosen language | caught |
| D34–D37, D39 | the player's cover not winning; OpenEmu's covers shown with the failover off; the ES-DE folder before ScreenScraper; covers never asked of ScreenScraper; a stopped ScreenScraper still taking the covers | caught |
| D38 | the failover asked while ScreenScraper is pending | **survived**; caught after the test was changed |
| D40–D45 | the window not stopping scraping on close; Preferences keeping its handler; the member account not saved on close; its file readable by others; the legacy developer file first; a test's moved config reaching the real legacy file | caught |
| D46–D51 | the old `OnlineCovers` key kept; the themed view without the scraped text; the queue not promoting a game shown; a retry due at once; a newer `media.db` opened; no wait for the download speed | caught (re-run; see below) |
| D52 | LunaP: the password preview drawn in the clear | caught |

**52 of 52 caught, two only after their tests were strengthened.**
- D28: the only test of a kind turned on later used a kind the game offered. `A_kind_turned_on_later_that_the_game_never_offered_costs_no_request` now turns on title screens for a game with none and requires no second request.
- D38: the window's test looked for other servers as soon as ScreenScraper's cover appeared, before the failover's
  quarter-second spacing had passed. It now waits 0.8 s first.

**A defect in the runner, found and corrected.** From D47 on, every mutant also failed the old-setting test. The runner
restored each file by moving its backup back, which keeps the backup's older write time. A mutant in another project
(D46 in Galaxia) therefore left its build newer than the restored source: every later build skipped Galaxia, and the
bin still carried D46. The round's own clean rebuild at the end skipped it too, and so did the last mutant's project in
Mistress (D51) and in LunaP (D52). Running the old-setting test on that build failed, which demonstrated it. The runner
now stamps every restored file with the present time. Every mutated file was touched and the tree rebuilt, and D28,
D38 and D46–D52 were run again on the corrected runner: each was caught by its own test alone, and the scraping tests
passed on the rebuilt tree. The results of D1–D45 were unaffected: each mutant before D46 was followed by one in the
same project, whose own write made the build recompile it. A mutant round can leave its mutant behind in two places:
in the source when it is interrupted, and, as here, in `bin/` when the restore makes the source look older than its build.

### 17.12 Where the developer file goes, for a published build and the handheld

A published tree reads `<tree>/home/etc/EmuSen/screenscraper-developer.json` first, then
`~/.config/EmuSen/screenscraper-developer.json`:
- `out/linux-x64/Mistress/` (the folder holding `.dianaosroot`) reads `out/linux-x64/Mistress/home/etc/EmuSen/screenscraper-developer.json`;
- a copy installed at `~/Apps/Mistress/` reads `~/Apps/Mistress/home/etc/EmuSen/screenscraper-developer.json`;
- both then fall back to `~/.config/EmuSen/screenscraper-developer.json`, so on the handheld one file there, mode 0600,
  serves every build.

The publish recipe's zip must leave out `home/etc/EmuSen/screenscraper*.json` and `home/Media/`, as it leaves out the
sandbox's cheats and saves; the recipe is not a committed script, so this is a rule for whoever zips.

### 17.13 Not done in stage (d)

- **P60, N64's byte order,** was not measured: no N64 file was drawn, and a second run for it was not made. Until it
  is, an N64 file ScreenScraper lists only in the other order costs two requests, one of them unrecognised.
- **Nothing ran on the handheld.** Headlessly, P45's test (§15.9) opens Preferences from the pad menu over the themed
  view and walks every tab with `PadAudit`, the Scraping tab included, and it passed with the tab in place. How the member
  account's keyboard reads at arm's length on the Legion Go S is the device's to say.
- **The whole library was not scraped.** P10's three days are a projection.
- **The default kinds cost 4.7 requests a game.** Miximages and covers are the large files; a player on the 10,000-a-day
  account who wants the library in two days should turn miximages off. No setting orders the kinds by cost.
- **§5.6's orphan pass over `games.db`** was not extended; renames are followed by the scraper's own path cache.
- **ScreenScraper's name is not shown.** The file's name is, as before.
- **No refresh**, no search by name, no video (Q4), no back cover, fan art or 3D box. *All built in Pass 8, §38.*
- **§5.7's publish exclusions** are a rule written here and in §4.60 of the settings reference, not a script.

### 17.14 Scraping only when the player starts it, and the scope the player chooses

**Two rules, decided 2026-09-26 after 17.1–17.13 were built,** which override what 17.2 and 17.3 describe:

1. *Every run is started by the player,* so that the ScreenScraper API is never spammed. Nothing may reach ScreenScraper
   at start, when the library is shown or refreshed, when a game is selected or shown in the themed view, when a game is
   added, when the developer file appears, or from a queue a previous session left. An interrupted run may be offered as
   Resume but never resumes by itself. OpenEmu's failover is bound the same way: only inside a run the player started, and
   only for games ScreenScraper had nothing for. What is kept already is shown offline.
2. *The player chooses the scope:* this game (from the game's menu or the pad menu, in the library and the themed view), a
   console (Game Boy Color its own shelf), games missing their art (in the library or a console), or all games as a
   deliberate choice; with the count and a quota estimate shown and confirmed first, progress shown, and a cancel.

**What changed, and why the first build broke rule 1.** 17.3's window started workers whenever the developer file was
found, queued every tile drawn without a cover and the themed gamelist's selection, and resumed any queue left in
`media.db`; §4.39's failover started on the first tile drawn without art. §5.5's reasoning was that a queue fed by
display is paced and within quota, so it is safe. The rule is not about the quota but about the service's load
and about the player's consent to each run, and a paced request is still a request the player did not ask for. That
argument was not made in §5 and should have been.

**As built now:**
- `Scraper` runs one run over what is queued and ends when its queue is empty, when the quota stops it (what is left stays
  queued), or when cancelled (`ScrapeRunEnd`). A worker no longer idles waiting for work, so nothing queued later is
  asked until the next run is started.
- The window starts a run only through `ConfirmAndScrapeAsync` (the confirm sheet, then `StartScrape`) or `ResumeScrape`.
  `ApplyScraping`, at start and when Preferences closes, reads the settings, whether the developer file is there, and
  opens `media.db` to read; it starts nothing.
- A new run replaces the queue; Resume runs what is there. "Scrape this game" forgets an Unknown or Error answer and the
  failover's recorded outcome for that game, so it is asked again; wider runs do not re-ask what has been answered.
- The failover is asked by the run for a game whose ScreenScraper answer left it without a cover, for every game of the
  run when ScreenScraper cannot be used at all, and, when the quota stops a run, for the games it did not reach. The run
  ends when both have answered. `BindCover` asks nothing.
- The plan shown before a run: games; games ScreenScraper has not been asked about; at most one lookup and one request
  per picture kind for each (17.9 measured that each picture is a request); what is left today; 13 s a game.
- Preferences ▸ Scraping has the console list (Every console, then each shelf), **Only games with no cover**, **Scrape...**,
  **Resume**, **Cancel Scraping** and a progress bar. The pad menu has **Scrape This Game...** (the library's or the
  themed gamelist's selected game) and **Scrape Games...** (Preferences on that tab), which reads "Scraping (n of m)..."
  during a run.

**Tests.** The fake server counts every request. Each path of rule 1 has a test that requires zero requests:
`Starting_showing_and_refreshing_the_library_asks_no_server`, `A_game_added_to_the_library_asks_no_server`,
`The_developer_file_appearing_asks_no_server`, `A_queue_left_by_an_earlier_session_is_offered_as_resume_and_never_resumed_by_itself`,
`What_is_already_kept_is_shown_with_no_request`, `Declining_the_confirm_step_asks_no_server`, and in the themed view
`Moving_through_the_gamelist_asks_nothing_and_scrape_this_game_fills_its_text_and_pictures`; `OnlineCoverWindowTests`
requires a tile drawn without art to ask nothing. Rule 2's scopes, the confirm step, the plan's numbers, progress,
cancel and Resume, and a new run replacing an old queue each have their own test in `ScrapeWindowTests`, and `Scraper`'s
run ends are tested in `ScraperTests`. The pad reaches every new control: stage (e)'s
`Every_control_of_each_sheet_opened_from_the_themed_view_is_reached_by_the_pad` and `PadSettingsWindowTests` walk
Preferences with the Scraping tab in it, and pass.

**Mutants** (`mutants-stage-d-userstarted.json`, the same runner, corrected as 17.11 describes; log `run-stage-d-u.log`):

| # | Mutant | Result |
|---|---|---|
| U1 | a run starts by itself at start and whenever Preferences closes | caught |
| U2 | a queue an earlier session left is resumed by itself | caught |
| U3 | the developer file appearing starts a run | **survived**; caught after the test opened with `media.db` already there |
| U4 | a tile drawn without a cover asks the failover, as `OnlineCovers` did | caught by 17 |
| U5 | showing or refreshing the library scrapes what has no cover | not built at first (the mutant named a type without its namespace); caught by 24 once rewritten |
| U6 | the themed view's selection is scraped as it is shown | caught |
| U7 | no confirm step | caught by 6 |
| U8 | cancel leaves the worker running | caught |
| U9 | a new run adds to an interrupted run's queue | caught |
| U10 | Scrape This Game does not ask again what ScreenScraper did not know | caught |
| U11 | a console scope takes every shelf | caught |
| U12 | "only games with no cover" ignored | caught |
| U13 | the failover asked for a game that already has a cover | **survived**; caught after a test was added |
| U14 | what a quota stop leaves is not handed to the failover | caught by 5 |
| U15 | the plan counts one request a game | caught |
| U16 | the pad menu offers no Scrape This Game | caught by 2 |
| U17 | a run idles on an empty queue instead of ending | caught by 12 |

**17 of 17 caught, two after their tests were strengthened.** U3's test had opened without `media.db`, and the mutant
only fired with the store open, which is the ordinary case after a first session. U13's only route to the failover
without ScreenScraper had one game with no cover, so skipping the "has a cover" check changed nothing; the new test has a
second game with the player's own cover and requires the failover to be asked about the first alone.

**An intermittent failure, recorded and not attributed.** In the blast-radius run after this change (502 tests), stage
(c)'s `SceneMotionTool.List_held_strip` failed once with Avalonia's "the calling thread cannot access this object". It
passed alone (5 of 5 in its class) and in a rerun of the BigPicture and Scraping tests together (394 of 394); the broad
run before this change had passed it. Nothing in this stage touches that tool, but a single failure beside new window
tests is not evidence either way, and repeating broad runs to catch it again was ruled out by the test-load rule.

*Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78: `GridSceneTests`' seven plain facts built scenes off
the headless session's dispatcher, and a control built on another thread while a dispatch starts makes that start-up
fail with this message (§4.78.2 reproduces it from their own scene). Moved onto the dispatcher in `b272675c`.*

**The live run of 17.9 was made before this change**, by a tool that drives `Scraper` directly and is itself a deliberate,
one-off run started by a person; its numbers stand.

---

## 18. Big picture from the desktop: Big Picture below Fullscreen in the View menu (2026-09-26)

*Opened 2026-09-26, on request (§10.1, Q8 amended): the desktop needs a button to enter big picture mode as well.*
Until this section, big screen was a decision the window took once, when it was made (settings
reference §4.29, §4.43), and §15 built the themed view on that premise. This section makes the decision switchable while
the window runs. It was built three times in one day, because the request was first misread and then placed:

| Build | What it was | Why it changed |
|---|---|---|
| 1 (`e649b8c1`–`07f66a9c`) | the desktop's full screen *was* big picture: a toolbar Fullscreen button, F11, the View menu and the window manager all entered it | a restatement of the request (full screen enters EmuSen's big picture) was taken as the design. Corrected the same day: the fullscreen button must not trigger big picture on the desktop; a separate button must, so that the player has the choice between both |
| 2 (`50c571e4`, `138ffabd`) | two toolbar buttons, a plain Fullscreen and a Big Picture, with a View menu entry and a key for each | placed on request: the Big Picture entry goes under the View menu, below Fullscreen |
| 3 (`df49d4de`) | the View menu holds Fullscreen (F11) and directly below it Big Picture (F10); no toolbar buttons | — |

The settings reference's §4.54 is the player's account of the third build; this is the record of all three. What
survived from build 1 into build 3 is the switch itself (`ApplyBigScreen`, `SetBigPicture`), the cleanup, the popup
handling, the desktop's place given back, and the F11 fix.

### 18.1 Expectations, and what the tests said of them

No predictions were written before build 1: the request came in the middle of the day's work and the build went
straight to it. What was expected before the tests first ran is recorded instead, with what they showed. It is weaker
evidence than §15's predictions, since the expectations and the design were written by the same hand in the same hour.

| # | Expected | Found | Verdict |
|---|---|---|---|
| X1 | the themed view's round trip needs no restoring of the desktop's place, since the themed view never moves the desktop's list | the themed case passed while the place was being thrown away at the moment it was taken (§18.4); only the case without a theme, whose pad moves the shared list, caught it | held, and showed that the themed case alone proves nothing about the restore |
| X2 | the headless platform refuses a full-screen `WindowState`, so the window manager's route cannot be tested headlessly | the headless window took every state set on it, and LunaP's `FullScreenChanged` was raised for each | refuted; the route is tested, a real window manager still is not (§18.6) |
| X3 | raising `Button.ClickEvent` on a toolbar button is a click (build 1) | it runs the event's listeners and not the button's command; nothing happened | refuted; build 1's test pressed and released a pointer instead. Build 3 has no button, and its tests invoke the menu's actions |
| X4 | the Game Mode check in the full-screen handler is needed beside the one in `SetBigPicture` (build 1) | mutant D2 of build 1 removed it and survived | refuted; the copy was removed, and with it the same copy in Esc's rule |
| X5 | a hidden button reads as not effectively visible (build 1) | a control never attached to the visual tree reads `IsEffectivelyVisible` true | refuted; moot in build 3 |
| X6 | decoupling (build 2) would need the event handler to tell "leaving by the switch" from "leaving by F11", or restoring the window would re-enter `SetBigPicture` | `SetBigPicture` clears its flag before it restores the window, so the event the restore raises finds nothing to do; one `restoreWindow` argument, false only from the event, was enough | held, as expected |

### 18.2 What build 3 is

- `Views/MainWindow.BigPictureSwitch.cs`: `ApplyBigScreen(bool)` holds every change big screen makes to the window and
  is run at start and by every switch. `SetBigPicture(bool on, bool restoreWindow = true)` is the switch and the one
  guard for Game Mode. Entering records `WindowState` and the desktop's place (console, search, game), then asks for full
  screen. Leaving releases the UI sound stream, gives the place back, and, unless the leaving came from the window's own
  full-screen change, sets `WindowState` back. The only use of `FullScreenChanged` is that leaving full screen leaves big
  picture, keeping the state that route chose.
- The View menu: `_Fullscreen` (checkable, F11) and directly below it `_Big Picture` (a command, F10). A new hotkey
  action, `ToggleBigPicture`, appended to `HotkeyAction` with F10 by default. `HotkeyBindingMap.Load` gives it to an older
  bindings file on a free key, as it does for every added action.
- The pad menu shows Full Screen outside big picture, and Big Picture or Exit Big Picture outside Game Mode. Those are
  the two lines of `MainWindow.Pad.cs`'s menu this changes, to keep the merge with the scraping branch small.
- F11 goes through `ToolWindow.ToggleFullScreen`. It had set `WindowState` itself, to `Normal` on the way out, so a
  maximised window left full screen as a normal one; mutant D24 is that old line. The existing
  `Leaving_fullscreen_returns_a_maximized_window_to_maximized` did not see it, because it goes through the menu, which
  was always right.
- `Program.BuildAvaloniaApp` asks `EmbedsPopupsAtStart`: `--bigscreen` or a Game Mode session. The setting no longer
  takes part, because a start it puts in big picture can now be left, and a platform option read once cannot follow a
  switch. The window embeds its own popups with LunaP's `EmbeddedPopups` while in big picture instead. Nothing was added
  to LunaP.
- The toolbar is as it was on WiseMan.

**Persistence, decided twice.** Build 1 wrote `BigScreen` on every switch, so the next start used the mode last used.
With one control that was both full screen and big picture, that was the obvious reading. Builds 2 and 3 write nothing,
and Preferences' "Start in big screen mode" alone decides, as Steam's own start-in-Big-Picture setting does. With two
controls, the last-mode rule would make one press of Big Picture decide every later start, and it would turn a setting
the player sets into a record the program keeps. Mutant D21 is build 1's rule.

### 18.3 The decisions that were taken once

Found by reading `StartPadNavigation`, `Program.BuildAvaloniaApp`, and every reader of `_bigScreen`
(`ThemedStyleWanted`, the pad menu's Theme Settings entry):
- switchable now: the flag; the menu bar, sidebar, facet and text sizes; the full screen at `Opened`;
  `SheetLayer.PresentsWindows`; process-wide popup embedding; the themed library's setup;
- checked and found not to be big screen's: the pad timer, which serves the desktop too.

Each, and what it is now, is tabled in the settings reference's §4.54 with the test that holds it. `SheetLayer` reads
`PresentsWindows` when a window is shown, so a switch changes where the next window goes and moves no window already
shown.

### 18.4 A defect the cycle test found

Build 1 forgot the desktop's place in the same call that took it: `SetBigPicture` cleared the field at its end whichever
way it switched, so leaving had nothing to give back. The case without a theme failed at its first exit, on the selected
game (Dune Relay, where the pad had moved it, instead of Cobalt Harbor); the themed case passed (X1). It is mutant D13.

### 18.5 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/mutate_desktop_button.py`, and each build's run is kept:
- build 1: `mutate_desktop_button_v1.py`, `mutants-desktop-button-v1.txt`, `mutants-desktop-button-v1-rerun.txt`;
- build 2: `mutants-desktop-button-v2-buttons.txt`;
- build 3: `mutants-desktop-button.txt` and `run-desktop-button.log`.

Each mutant was built and run alone at `nice -n 10` against `BigPictureSwitchTests` and the four big-screen cases of
`LibraryScreenTests`, and the source was restored; the tree was rebuilt clean after each run.

**Build 1:** 28 mutants, 27 caught. The survivor, D2 (a second Game Mode check in the full-screen handler), was
equivalent, and the redundant code was removed (X4).

**Build 2:** 36 mutants, all caught. It retired build 1's D1 and D2, which assumed the coupling, and added D29–D36 for
the decoupling.

**Build 3:** 35 mutants, all caught. D5, D6, D31 and D33 went with the toolbar buttons they mutated; D37–D39 are the
menu's.

| # | Mutant (build 3) | Result |
|---|---|---|
| D1 | leaving full screen (F11, the window manager) does not leave big picture | caught by 2 |
| D2 | full screen enters big picture again (the coupling that was rejected) | caught |
| D3 | a Game Mode session can be switched out of big picture | caught |
| D4 | the pad menu offers Exit Big Picture in Game Mode | caught |
| D7 | entering does not make the window full screen | caught by 2 |
| D8 | a big-screen start is not made full screen when the window opens | caught |
| D9 | leaving does not return to the desktop's place | caught |
| D10 | the desktop's console not given back | caught |
| D11 | the desktop's search not given back | caught |
| D12 | the desktop's game not selected again | caught |
| D13 | the place forgotten as soon as it is taken (§18.4) | caught |
| D14 | leaving keeps the interface's sound stream | caught |
| D15 | leaving leaves the themed view up, its wake running | caught by 2 |
| D16 | sheets fixed at the start's answer | caught by 2 |
| D17 | the window's popups not embedded by a switch | caught by 2 |
| D18 | the library's text stays large on the desktop | caught |
| D19 | the themed library not set up by a switch | caught by 3 |
| D20 | the themed library set up again on every entry | caught by 2 |
| D21 | a switch writes the setting (build 1's persistence) | caught by 3 |
| D22 | Esc leaves with a game suspended behind the library | caught |
| D23 | Esc never leaves | caught |
| D24 | F11 back to the old line: out of full screen always to Normal | caught |
| D25 | process-wide popups ignore the Game Mode session | caught |
| D26 | process-wide popups follow the saved setting again | caught |
| D27 | the menu bar stays hidden after leaving | caught by 2 |
| D28 | the sidebar stays hidden after leaving | caught |
| D29 | leaving goes to Normal, not to the state big picture was entered from | caught |
| D30 | leaving by F11 or the window manager puts back the entry state (plain full screen again) | caught |
| D32 | the Big Picture key does nothing | caught by 5 |
| D34 | the entry state not taken | caught |
| D35 | the View menu's Big Picture does nothing | caught |
| D36 | the pad menu's Full Screen offered in big picture | caught by 2 |
| D37 | Big Picture above Fullscreen, not below it | caught |
| D38 | the View menu's Big Picture shows no key | caught |
| D39 | Big Picture a checkable entry that flips on each choice | caught |

"Caught by n" counts test methods; the cycle test's two cases, with a theme and without, count once.

**The cleanups, proved by tests that fail without them.** D14 removes the sound stream's release, and
`Leaving_big_picture_lets_go_of_the_interface_s_sound_stream…` fails. D15 removes the showing that hides the themed view,
and `Leaving_big_picture_stops_the_themed_view_s_wake…` fails, with frames drawn by the hidden view. That test lets
2.6 s of real dispatcher time pass after leaving with a text in its pause. This is §15.14's defect in the switch's form,
and unlike §15.14's first attempt the test tells the two builds apart. The held direction's release was written and then
removed, not mutated: the pad's poll already lets go of it on every tick outside the themed view, so the line could not
have been told apart from its absence.

**The broad runs.** One for build 1 and one for build 3, each the Mistress filter without `ShaderSettingsWindowTests`,
`ShaderBrowseBench` and `SceneGpuBench`, under the load rule of 2026-09-25:
- **Build 1:** 632 tests; 628 passed, 4 skipped (the picture tools, which need `EMUSEN_BIGPICTURE_PNG=1`), none failed.
- **Build 3:** 632 tests; 627 passed, 4 skipped, and one failed: `SceneMotionTool.List_held_strip`. It failed before its
  own code ran. Inside `HeadlessUnitTestSession.EnsureIsolatedApplication`, Avalonia's headless platform set-up threw
  "The calling thread cannot access this object because a different thread owns it" from `DefaultRenderLoop.Add`. The
  class, run alone with `SceneMotionTests` beside it, passed 17 of 17.

This change does not reach that class: it builds its scene without a `MainWindow`. The failure is a start-up race in the
test session, of the family §15.14 and §16.8 recorded (unrelated tests, only in the broad run). It is recorded, not
attributed, and the broad run was not repeated.

*Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78: the start-up race is real and was caused by
`GridSceneTests`' seven plain facts building scenes off the dispatcher; §4.78.2 reproduces this message at
`DefaultRenderLoop.Add` from their own scene. Fixed in `b272675c`, and a guard now fails a test of that shape (§4.78.5).*

### 18.6 Not done

- **No real window manager was used.** KDE's and GNOME's own full-screen commands, on X11 and on Wayland, reaching
  Avalonia as `WindowState.FullScreen`, and a normal window getting its size back, are assumed from Avalonia and LunaP
  (§75.2 there), not observed. The headless platform takes every state it is given (X2).
- **F10 was chosen, not surveyed.** GTK applications open their menu bar on F10; a desktop binding it globally would take
  it before Mistress. It can be rebound.
- **No pointer way out of big picture.** The menu bar is hidden there; a mouse-only player leaves by the window manager's
  full-screen command. Build 2's toolbar button was such a way, and it went with the requested placement.
- **Nothing ran on the handheld.** Game Mode's refusal to leave is tested with `XDG_CURRENT_DESKTOP=gamescope` set around
  the window's construction, as §4.43's tests are.
- **Windows open at a switch stay what they were**: a desktop window stays a window over big picture, and a sheet open
  when leaving stays a sheet until closed.
- **The library's own view settings** (grid or list, cover size, the Library, Save States and Screenshots choice) are
  shared by both modes and not given back.
- **Steam's Big Picture on the desktop** is still not read (§4.43 of the settings reference).
- **Popups Avalonia parents outside the window's tree** (some tooltips and context menus) are not reached by the
  window's embedding; on the desktop a window manager draws them at their size, which is what they were before.


## 19. EmuSen's own look in the theme list (2026-09-26)

*Opened 2026-09-26, on request (§10.1, Q8 amended again): EmuSen's normal big-picture theme is to be an option in the
themes list.* §4.10 had kept the existing big-screen library as a separate choice, Preferences' "Library style"
(Mistress / ES-DE theme), beside the theme folder. Stage (f)'s Themes tab (§16.4) listed only ES-DE themes. This section
makes the two one list. The player's account is §4.56 of the settings reference; this is the record.

### 19.1 Expectations, written before the tests ran

As in §18.1, these were written by the same hand as the design, in the same hour, so they are weaker evidence than a
stage's predictions.

| # | Expected | Found | Verdict |
|---|---|---|---|
| T1 | the pad menu already offers Theme Settings over EmuSen's own library, since its entry's condition (`_bigScreen && !inGame`) never read the style | it does; `The_sheet_is_reached_by_the_pad_from_both_looks` passed on the first build with no change to `MainWindow.Pad.cs`. Mutant L15 (the entry offered only over the themed view) is caught by four tests | held. The pad menu was not edited, which also kept this branch's merge with §18's small |
| T2 | no new storage is needed: `LibraryStyle` and `BigPictureTheme` already express every entry | held, with one case the list must decide: `Theme` style with no folder. It is shown as EmuSen's row, because that is what the session draws (§4.52's fallback). Mutant L11 is the other reading, and it is caught | held |
| T3 | choosing a theme from the sheet leaves a folder read in place in the list | refuted, and not by this change. `ThemeDownloads.Installed` lists the in-place folder only while it is `BigPictureTheme`, as §16.4 built it, so choosing a downloaded theme drops it. Found while strengthening the Preferences test, and recorded in §19.5 | refuted (a limit of §16's list, kept) |

### 19.2 What was built

- `BigPicture/BigPictureLooks.cs` (no Avalonia types) holds the list: a `BigPictureLook` is a name and an optional
  `InstalledTheme`, with none for EmuSen's own look. `All` gives EmuSen first, then `ThemeDownloads.Installed`.
  `BuiltInCurrent`, `IsCurrent` and `Current` read the two settings. `Choose` writes them and saves. The sheet and
  Preferences both use it, so they cannot list or store differently.
- `ThemeSettingsWindow`:
  - The Themes tab's first row is `ThemeRowBuiltIn`, "EmuSen (built in)", with one button, `ThemeUseBuiltIn`.
  - `Use` takes a look and calls `_applied`, so the view beneath swaps before the sheet closes. It passes `replaced`
    only for a theme, as before.
  - With EmuSen chosen, the Options tab holds an `EmptyState` (`BuiltInOptionsNote`), and the sheet opens on Themes.
  - The old "No theme chosen" state now occurs only for a set folder that is missing, and says so.
- `PreferencesWindow`: the Library Style dropdown is replaced by `BigPictureThemeDropdown`, filled from
  `BigPictureLooks.All`. The picker row became "ES-DE Theme Folder". A choice in either calls `ThemeChosen`, set by
  the window, and applies at once. `ShowBigPictureTheme` refills the row and the picker from the settings, and does
  nothing when they are already shown, so it is safe to call from the row's own choice.
- `MainWindow`:
  - `WatchPreferences` is one line in `ShowPreferencesAt`.
  - `ThemeChanged` also refreshes the open Preferences sheet, so a choice on the Theme Settings sheet stacked over it
    shows there when that sheet closes.
  - Nothing holds a timer or a stream. The one reference kept, to the open Preferences sheet, is dropped on its `Closed`.

Nothing was needed in LunaP.

**Preferences: replace or sync, and why replace.** The request allowed either. Syncing would have kept two controls for
one decision. One of them would name the choice by a style ("Mistress", "ES-DE theme") and not by the list's own names.
It would also carry a state the list cannot show (`Theme` with no folder), and it would take a second path to keep the
two in step. Replacing leaves one list, built by one function, shown in two places. The stored names were kept, so no
settings file changes meaning:
- `LibraryStyle` `Mistress` is EmuSen's row.
- `Theme` with a folder is that folder's row.
- `Theme` without one is EmuSen's row, as the session already drew it.

This does not generalise to Preferences' other rows. It holds here because the sheet and the row show the same
decision, and the sheet is where the list is managed.

**Choosing EmuSen keeps the folder.** Clearing `BigPictureTheme` on choosing EmuSen would be the tidy reading of "EmuSen
has no folder". But the list would then drop a folder read in place, and the player could not choose it again without
Preferences' picker. Mutant L8 is that reading, and it is caught.

### 19.3 Tests

`BigPictureThemeListTests` has 7 cases, on `ThemedSession` and `PadDriver`, headless:
- `EmuSen_is_the_first_entry_of_the_themes_list_and_the_current_one_is_marked`
- `Choosing_EmuSen_swaps_the_view_at_once_and_is_remembered`: by pad; the sheet still presented, the themed view gone,
  the Options note, the folder kept; a second `MainWindow` on the same settings starts on EmuSen's library.
- `From_EmuSen_s_look_the_pad_reaches_the_sheet_and_an_ES_DE_theme_swaps_back`: the sheet opens on Themes; after the
  swap, A enters a gamelist.
- `The_sheet_is_reached_by_the_pad_from_both_looks`: every control of both tabs is reached by `PadAudit`, over each look.
- `The_built_in_entry_cannot_be_removed_and_is_what_a_removed_theme_falls_back_to`: a stamped theme under
  `home/Themes`, removed through the sheet's confirm.
- `Preferences_and_the_themes_list_agree`: two themes of one name, one downloaded and one read in place.

`ThemedLibraryHostTests`' Preferences case now chooses on the new row, and asserts the view changed before Preferences
closed. Before, it asserted the change waited for the close.

**Runs.**
- The blast radius: `ThemeSettingsSheetTests`, `ThemedLibraryHostTests`, `ThemedLibraryPadTests`,
  `ThemedLibraryFlowTests`, `BigPictureSwitchTests`, and the classes that open Preferences (`PreferencesThemeTests`,
  `PadNavigationTests`, `LibraryScreenTests`, `ScrapeWindowTests`, LunaP's `AccessibilityTests` and
  `HandRolledControlTests`). 51 and 85 tests passed.
- The broad run is §19.4's.

**Pictures**, at 1280×800, written by `ThemeListPictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to
`~/.cache/emusen/bigpicture/png/theme-list/`:
- `themes-artbooknext-chosen` and `themes-emusen-chosen`: the Themes tab over Art Book Next, before and after choosing
  EmuSen. EmuSen's list library shows beneath the second.
- `options-emusen-chosen`: the note.
- `preferences-appearance-emusen-chosen`: the row.

They were looked at. The sheet is narrower on the Options tab than on Themes, as it was in §16's pictures: its width
follows its content.

### 19.4 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/mutate_theme_list.py`, with its log in `run-theme-list.log` and its
verdicts in `mutants-theme-list.txt`. Each mutant was built and run alone under `nice -n 10`. Each ran against
`BigPictureThemeListTests`, `ThemeSettingsSheetTests` and the Preferences case of `ThemedLibraryHostTests`. The source
was restored after each, and the tree was rebuilt clean at the end.

**23 mutants, 23 caught at once, none survived:**

| Rule | Mutants |
|---|---|
| The list | L1 no EmuSen row, L2 EmuSen last, L3 a Remove button on it, L4 never marked In Use |
| Choosing | L5 and L6 the style not written, L7 not saved, L8 the folder cleared, L9 and L10 not applied beneath the sheet |
| Which row is current | L11 no folder not counted as EmuSen, L12 the folder marked under EmuSen too |
| The sheet | L13 Options rows under EmuSen, L14 opening on Options, L15 the pad-menu entry only over the themed view |
| Removal | L16 removing the theme in use keeps it chosen |
| Preferences | L17 no EmuSen entry, L18 not written, L19 not applied at once, L20 not following the sheet, L21 two themes of one name not told apart, L22 the folder picker not following, L23 the first entry shown whatever is current |

L21 and L22 were caught only because the Preferences test had been strengthened first. Its first version used
one theme, whose folder never changed, so it could not see either rule. That was noticed while writing the runner, and
the test was changed before any mutant ran. Doing so is what turned up T3.

**The broad run.** It used the Mistress filter without `ShaderSettingsWindowTests`, `ShaderBrowseBench` and
`SceneGpuBench`, once, at the end, under the load rule of 2026-09-25. 770 tests: 764 passed, 6 skipped (the picture tools, which need `EMUSEN_BIGPICTURE_PNG=1`, and the live scrape tool, which needs `EMUSEN_SCRAPE_LIVE=1`), none failed, in 3 min 9 s.

### 19.5 Not done

- **A folder read in place leaves the list** once another theme is chosen (T3). `BigPictureTheme` is its only record,
  as it was in §16. Keeping a list of folders read in place would need a new setting, and it was not asked for.
- **The download's first use.** A theme downloaded while EmuSen is the explicit choice becomes the folder but not the
  choice. That is argued in §4.56 and not tested.
- **Nothing ran on the handheld.**
- **The pad menu's entry is still called "Theme Settings".** With EmuSen chosen, "Themes" might read better. It was left
  alone to keep the merge with §18's pad-menu lines small.

## 20. The scraping status window, and signing in with a member account (2026-09-26)

*Opened 2026-09-26, on two requests.* The first: a status window while ROMs are being scraped. The second, made
during the build: a player can sign in to ScreenScraper with their own account credentials if they prefer. Both
belong to stage (d)'s Scraping tab and to §17.14's rules, which bind them: a run is started only by the player, and nothing is asked of a server outside one. The player's account is
the settings reference's §4.57 (the window) and §4.60's "Signing in"; this is the record.

### 20.1 Expectations, and what the build found

As in §18.1 and §19.1, these were written by the same hand as the design, during it, so they are weaker evidence than a
stage's predictions. Two were refuted, both about the test harness rather than the product, and both would have left a
test that passes on a broken build.

| # | Expected | Found | Verdict |
|---|---|---|---|
| S1 | Pause fits the run as built, with no new state in `media.db`: every request already waits for the quota's turn in one place, so a gate there holds a run cleanly | held. `Scraper.TurnAsync` is that place: it awaits a pause (a `TaskCompletionSource` swapped in and out) and then `TakeTurnAsync`; the worker's loop awaits it too before taking a game. A request in flight finishes; cancelling or disposing while paused releases the wait. W12 and W13 (§20.4) are the two ways of breaking it, and both are caught | held |
| S2 | the estimate needs no model of the download-speed limit, which §17.9 found to be half a run's time: measuring the run's pace includes it | held by construction: the estimate is time worked over games answered. It has not met a live run; §17.9's is the only one, and it came before the window | held, not measured live |
| S3 | the window's timer ticks under the harness as the scraping tests pump the dispatcher (`RunJobs` in a loop) | **refuted.** Under the headless platform a `DispatcherTimer` fires only while the dispatcher runs a frame; `RunJobs` runs posted jobs and no timer. The first close test failed on the unmutated build ("drawn 1 times in 0.6 s"). Had it been written the other way round (asserting that a closed window draws nothing) it would have passed with the cleanup removed. The fixture now pushes a dispatcher frame for real time, as §15.14's test does | refuted; the fixture changed |
| S4 | releasing held requests one at a time stops a run after a chosen game | **refuted** at first: the test released a request before running the job its worker had just posted, so two games' results landed in one `RunJobs` and "exactly one game done" was never seen. A game's result is posted before its worker's next request, so running the posted jobs before each release makes the stop exact | refuted; the fixture changed |
| S5 | a wrong member at `ssuserInfos` is a 403 whose text says "utilisateurs" | measured only without an account, in §17.9's live run: "Erreur de login : Vérifier les identifiants utilisateurs !". The fake server answers a wrong member that way. A wrong password against the live service was not tried | held as far as measured |
| S6 | the sign-in needs no change to the pad's routing | held for the routing; a defect was found beside it. Log In hides itself once signed in, and the focus went with it, so the pad's next press only found the sheet's first control again. The pane now hands the focus to Log Out, and back to the name box after Log Out. `Log_In_and_Log_Out_are_reached_and_used_by_the_pad_on_the_sheet` found it, and L13 is its mutant | held, with a defect found |

### 20.2 What was built

**The run, in `Scraping/` (no Avalonia type):**
- `ScrapeProgress` (in `ScrapeRun.cs`) became the window's model:
  - its start and end;
  - the tallies: found, not found, failed with the last reason, skipped, to be retried, filled by the failover;
  - what a worker is doing now and the last picture that arrived, set from the worker's thread under a lock;
  - the recent list, 200 games newest first;
  - the time paused;
  - the requests sent;
  - a version number that every change bumps.
- `Scraper` reports each step through `Activity` (looking up, downloading a kind, a picture arrived). It marks a result
  that cost no request as `Skipped`. It gained `Pause` and `Resume` (S1).
- `ScreenScraperClient.Member` is read at every request, so it can change during a run, and `MediaUrl` takes `ssid` and
  `sspassword` out of a picture's address when there is no member. `SignInAsync` is the one `ssuserInfos` request.
- `ScrapeSignIn.cs`: `SignInAnswer.From` turns a status and a body into a result and a sentence, each passed through
  the redactor. `ScreenScraperJson.Member` reads `id` and `niveau`. `MemberAccount` gained `Verified` and `Delete`.

**The window**, `Views/ScrapeStatusWindow.cs`, is a LunaP `ToolWindow` of LunaP parts: `Ui` rows and sections,
`HintText`, `MeterRow`, `ButtonBar`, and a `LunaList` for the recent games (a `ListBox`, so its rows are built only in
view). `SheetLayer.Show` makes it a sheet wherever Mistress presents windows as sheets, so there is no second code path
for big-screen sessions. `MainWindow.Scrape.cs` opens it (`ShowScrapeStatus`), pauses and cancels through it, and
closes it in `StopScraping`. The pad menu's entry changed its action and nothing else. `SetUpScraping` wires the status
line's click.

**Four choices, and why.**
- *Polling, not posting.* A worker never calls the UI: it writes the run's current step under a lock, and the window
  reads the run on its own 250 ms timer, drawing only when the version moved, when Mistress raised `ScrapeChanged`, or
  while the run goes, for its clock. Posting each step would put a job on the UI thread for every lookup and picture, so
  a fast run (a queue of games already answered costs no request) would queue as many jobs as it has steps. A timer
  bounds the drawing at four a second whatever the run does, which `A_fast_run_is_drawn_at_the_timer_s_pace_not_once_an_event`
  measures and W18 breaks. Games' results still reach the UI thread as §17.14 built them, one posted job each, because
  they change the library and the queue.
- *Requests sent is counted by the run.* The day's count from ScreenScraper is known only after the first answer, and
  it includes what was used before the run, so its difference is not what this run cost.
- *Hide is Close.* The window holds nothing the run needs, so hiding it is closing it, and reopening builds it again from
  the run. That also means there is no hidden window with a live timer.
- *The sign-in keeps only what ScreenScraper accepted.* The two boxes of §17.7 saved whatever was in them when a box was
  left. An account now exists only after one request has accepted it, and the file records when (`verified`), so an
  older file can be told apart and offered **Check** rather than dropped. Log Out deletes the file rather than emptying
  it, so no file remains that looks like an account.

**The member's name is shown unredacted in two places**, the Scraping tab's row and the window's Quota part. The
redactor of §17.7 blanks it, as a credential, everywhere else. Showing the player their own login on their own screen is
the purpose of those two rows; the password and the developer credentials are never shown anywhere.

**Found by looking at the pictures** (§20.5): the recent list, inside a stacked section, was given unbounded height and
ran under the Close button once a run had more than a few games; it is now a grid row and scrolls. The meter's value
column, 55 px wide in LunaP's template, clipped "12 of 20,000 · 19,988 left", so the meter now shows a percentage and
the counts have their own line. Preferences' Today row (stage (d)'s) had the same clipped meter, "180 of 10,000 ·
unrecognised …" in 55 px, and was changed the same way, its counts moving to the status text beneath it. The sign-in's
two boxes both read "(none)"; they now say what they are for.

Nothing was needed in LunaP.

### 20.3 Tests

On stage (d)'s fake ScreenScraper, headless, never the network. The fake gained a status per MD5, one member account
that `ssuserInfos` accepts (any other is the 403 of S5), and media addresses that carry the account the lookup was made
with, as ScreenScraper's do.
- `ScrapeStatusWindowTests`, 21 cases, and `ScrapeProgressTests`, 6: listed in §4.57 of the settings reference.
- `ScrapeSignInTests`, 14 cases: listed in §4.60.
- `ScrapeWindowTests`: `Preferences_keeps_the_member_account_in_its_own_file` was removed, since typing no longer keeps
  an account; its three assertions (the file, its mode, nothing in `appsettings.json`) moved into the sign-in's first
  test, and `Typing_an_account_without_Log_In_keeps_nothing` asserts the reverse of the old rule. The Scraping tab's
  button test now asks that Preferences' own handler is gone, since the status window, open during that run, is a second
  subscriber.

**Runs.** The blast radius, run when the tests first passed (the Scraping namespace, `OnlineCover*`,
`PadSettingsWindowTests`, `CrashLogTests`, `PadNavigationTests`): 221 tests, one failure, the button test above, which
was fixed. After the last change the same filter with `ThemedLibraryHostTests` passed 238, with 2 picture and live tools
skipped. The broad run, once, at the end, under the load rule of 2026-09-25: the Mistress filter without
`ShaderSettingsWindowTests`, `ShaderBrowseBench` and `SceneGpuBench`, 811 tests, 804 passed, 7 skipped (the picture tools
and the live scrape tool, which need their variables), none failed, in 3 min 49 s. No GPU test was run.

### 20.4 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/mutate_scrape_status.py`. Its list, `mutants-scrape-status.json`, is
written by `mutants_scrape_status_make.py`, which checks that every edit's text occurs exactly once. The log is
`run-scrape-status.log` and every verdict is appended to `mutants-scrape-status.txt`. Each mutant was built and run alone
under `nice -n 10`, against `ScrapeStatusWindowTests`, `ScrapeSignInTests`, `ScrapeProgressTests`, `ScrapeWindowTests`,
`ScreenScraperClientTests`, `ScraperTests` and `ScrapeCredentialTests`. The runner is stage (d)'s, corrected as §17.11
says: each restored file is stamped with the present time and compared byte for byte, and the tree is rebuilt at the end
of a round. It also takes several edits per mutant, so one rule spread over two lines can be broken as one mutant (L12).

**40 mutants: 39 caught by the tests written for their rule, one (W22) caught at first only by an unrelated test, and
caught by its own after that test was strengthened.**

| Rule | Mutants | Result |
|---|---|---|
| The window opens with a run | W1 | caught |
| Tallies, estimate, current step, thumbnail | W2 a failure not tallied, W3 a skipped game counted as found, W4 the estimate the plan's 13 s constant, W5 paused time counted as work, W6 the lookup step not reported, W7 the arrived picture not kept | caught |
| Why it stopped | W8 every stop put down to the quota | caught by 7 |
| Hide, Cancel, Pause | W9 Hide cancels, W10 Cancel without its confirm, W11 Cancel forgets the queue, W12 Pause not reaching the workers, W13 the pause only between games | caught |
| The summary; no request without a run | W14 no Resume note, W15 opening the window resumes a queue | caught |
| Redaction | W16 a row, W17 the last failure | caught by `ScrapeProgressTests` |
| Throttling | W18 a redraw on every change | caught by the fast-run test |
| Close what you open | W19 no cleanup at all, W20 the timer not stopped, W21 still subscribed | caught by the close test; W19 and W20 also by closing Mistress |
| | W22 closing Mistress leaves the window open | **caught only by an unrelated test at first**; caught by its own once it ran as a sheet |
| Where it opens | W23 the pad menu's entry, W24 the status line, W25 a window instead of a sheet, W26 the Status button | caught |
| Log In | L1 kept without asking, L2 kept when refused, L3 a wrong password read as the developer refused, L4 asking with no developer file | caught |
| Log Out | L5 the file kept, L6 a run in progress not told, L7 a picture's address keeping the account | caught |
| The old file | L8 dropped, L9 shown as checked with no Check | caught |
| Keeping it safe | L10 the file readable by others, L11 typing saved without Log In, L12 a sign-in message not redacted | caught |
| From the pad | L13 the focus lost when Log In hides, L14 the password box unmasked | caught |

**W22, and what it showed.** On the desktop, Avalonia closes a window's owned windows when it closes, so removing the
status window's close from `StopScraping` changed nothing there, and `Closing_Mistress_closes_the_status_window_and_its_timer`,
which ran only on the desktop, passed. A sheet is not an owned window: nothing but `StopScraping` closes it. Under the
mutant, the big-screen test's sheet stayed presented on a closed window with its timer running, and the next test that
pushed a dispatcher frame for real time failed. That is §15.14's defect class, reproduced on purpose. The test now runs
as a window and as a sheet, and W22 was run again: caught by the sheet case alone.

**Failures beside the catches.** In four rounds of the list (W3, W12, W13, W22), a test unrelated to the mutant failed as
well as the one that caught it: `Opening_the_window_with_no_run_sends_nothing_and_resumes_nothing`,
`A_closed_status_window_lets_go_of_the_run_and_stops_its_timer`, or stage (d)'s pad-menu test. Each time another test had
already failed under the mutant. W22 shows how such a failure travels: a test that fails midway leaves work on the shared
UI thread, and the next test to run the dispatcher in real time meets it. The three window classes were then run four
times on the unmutated build, 63 of 63 each time. So these are attributed to the mutants' earlier failures and not to
flakiness in the tests, but the attribution is argued from W22's mechanism and not shown for each of the other three.

### 20.5 Pictures

At 1280×800, written by `ScrapeStatusPictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to
`~/.cache/emusen/bigpicture/png/scrape-status/`. The fake server's media are replaced there by real pictures, a coloured
box per kind, so the thumbnail has something to show.
- `mid-run`: the desktop window over the library, the third game downloading its screenshot, the second's cover as the
  thumbnail, a failure's reason.
- `finished`: the summary.
- `quota-stop`: the day's limit less 2% reached after one game, "Why it stopped: today's requests are nearly used up
  (9800 of 10000)", and six games left queued for Resume.
- `sheet-bigscreen` and `sheet-bigscreen-finished`: the sheet in a big-screen session, during and after a run.
- `signin-sheet-bigscreen` and `signed-out-sheet-bigscreen`: the Scraping tab's member row signed in, then after Log Out.
- `preferences-today-bigscreen`: the Scrape row with **Status...**, and the Today meter as a percentage.

The desktop pictures are composed: the window is captured on its own and drawn centred over the main window's capture
with a one-pixel edge, since the headless platform captures one window at a time. They were looked at, and three
defects came from that (§20.2).

### 20.6 Not done

- **Nothing ran on the handheld**, and no member account was signed in against the live service (S5). The
  developer-refused text and the `niveau` field are read as the API page gives them, unmeasured.
- **A player's own developer credentials** are not supported, and the hint says so. ScreenScraper issues them to
  software authors; a player who had some could only use them by writing `screenscraper-developer.json` by hand. Offering
  that in Preferences is a possible follow-up, not built.
- **The estimate is a mean** over the games answered, so a run of found games (five requests each) after a run of
  unknown ones (one each) is estimated short.
- **OpenEmu's failover is not paused** and not counted in the requests sent.

---

## 21. The remaining passes to ES-DE parity (a plan, 2026-09-26)

*Written 2026-09-26, while §22 (collections, filters, sorting, jump to a letter and a random game) and §23 (the game
options menu and the metadata editor) were being built on other branches.* Stages (a) to (f) and §18–§20 built what
§7 planned, except stage (g). This section inventories what ES-DE offers a player that big picture still lacks, groups
the gaps into passes that can each be built and tested in one go, orders them, and states what is still to be decided. It
is a plan: nothing in it has been built, and every cost in it is an estimate.

**Sources, and what was not read.**
- ES-DE's `USERGUIDE.md` (5,074 lines) and `THEMES.md` (3,812 lines), master branch, in the copies stage (c) fetched on
  2026-09-25 to `~/.cache/emusen/bigpicture/motion/docs/`. Sections are cited by their headings. ES-DE's source was not
  read, and ES-DE was not run for this section. `INSTALL.md` and `FAQ.md`, which the guide refers to for command-line
  options, event scripts and controller profiles, were not read.
- ES-DE 3.4.1's own defaults, from the `es_settings.xml` it wrote into the stage (b) scratch home
  (`~/.cache/emusen/bigpicture/esde/home/ES-DE/settings/`). §13.8 lists the settings the stages changed there (the
  directories, theme, variant, scheme, aspect ratio, startup system and view, and `DisplayClock`); none of the values
  cited below is among them.
- TheGamesDB's API description, `api.thegamesdb.net/spec.yaml` (Swagger 2.0, 2,338 lines), and its key page, fetched
  2026-09-26.
- The 43 redacted answers of §17.9's live run, for what ScreenScraper offers beyond the kinds Mistress fetches.
- EmuSen's code at `cf73bbb3`, and the ROM library at `AppSettings.RomDirectory`, listed read-only.

**Numbering.** §22 and §23 were being written at the same time and will number their own predictions and questions.
To keep the three apart, as §17 did for stage (f), this section's predictions start at **P100** and its questions at
**Q20**, leaving P68–P99 and Q11–Q19 to them.

### 21.1 What was measured for this section

- **The library's folders** (read-only listing, 2026-09-26). `NES/` holds 16 folders (`USA`, `Europe`, `World`,
  `Hacks`, `Translated`, `Unlicensed`, `Pirate`, `PD`, `PC10`, `Versus` and six more) and no ROM at its top level;
  `GB/` holds 28 (`0-9`, `A` to `Z`, `[BIOS]`) and none at its top; `N64/` and `SNES/` hold none. No folder is nested a
  second level. No `.m3u`, `.cue` or `.fds` file exists anywhere in the library, and no ROM stem repeats between two
  folders of one console (5,524 files counted by the scratch script's extension list; the library's own count, §5.5, is
  5,520, and the difference was not traced).
- **What ScreenScraper offered beyond Mistress's kinds**, in §17.9's 39 found answers:

  | Kind (API name) | Games offering it | Files offered | Format | Median size | Largest |
  |---|---|---|---|---|---|
  | `video-normalized` | 32 of 39 | 32 | mp4 | 1.38 MB | 2.72 MB |
  | `video` | 32 | 32 | mp4 | 3.36 MB | 9.90 MB |
  | `manuel` (manual) | 27 | 47 (several regions) | pdf | 1.71 MB | 18.2 MB |
  | `box-2D-back` | 33 | 109 | png | 0.53 MB | 5.71 MB |
  | `box-3D` | 33 | 113 | png | 0.30 MB | 0.58 MB |
  | `support-2D` (physical media) | 33 | 77 | png | 0.41 MB | 0.66 MB |
  | `fanart` | 25 | 25 | jpg | 0.27 MB | 0.99 MB |

  The sizes are ScreenScraper's `size` fields; nothing was downloaded. The video codec is not in the answer and was not
  measured.
- **The desktop's decoders.** `/usr/bin/ffmpeg` is package `ffmpeg-8.1.2-3.fc44` and lists the `h264`, `hevc`, `av1`
  and `libdav1d` decoders; `openh264` and `poppler-utils` 26.01 are installed. What SteamOS on the Legion Go S provides
  was not checked.
- **ES-DE 3.4.1's defaults** that bear on the passes: `ScreensaverTimer` 300,000 ms and `ScreensaverType` `video`
  (the guide: with no videos it falls back to Dim); `ScreensaverSwapImageTimeout` 10,000 ms; `LaunchScreenDuration`
  `normal`; `InputOnlyFirstController` false and `InputDeviceNotifications` true; `InputControllerType` `xbox`;
  `QuickSystemSelect` `leftrightshoulders`; `RandomEntryButton` `games`; `ScrapeVideos` and `ScrapeManuals` true;
  `ViewsVideoAudio`, `MediaViewerVideoAudio` and `ScreensaverVideoAudio` true; `SoundVolumeVideos` 80;
  `FoldersOnTop` true; `ShowHiddenGames` true; `ListScrollOverlay` false; `SystemStatusBatteryPercentage` true;
  `MaxPlayTimeTracking` 8 (hours); `UIMode` `full` with `UIMode_passkey` `uuddlrlrba`; `ApplicationLanguage`
  `automatic`; `MenuColorScheme` `dark`.
- **TheGamesDB.** Every call carries an `apikey`. Answers carry `remaining_monthly_allowance`, `extra_allowance` and
  `allowance_refresh_timer` (the spec's example is 2,592,000 s, thirty days), and `/v1/API/Limit` reads them without
  counting against them. Besides name and ID searches there is `/v1/Games/ByGameHash`, taking an MD5 or a CRC and an
  optional platform. `/v1/Games/Images` serves `fanart`, `banner`, `boxart` (with a `side`, front or back), `screenshot`,
  `clearlogo` and `titlescreen`, and `/v1/Games/Videos` exists. The key page answers "You must be logged in to the site
  to view your api key", so a key belongs to a site account. The spec's `license` field names GPL-3.0 and points at the
  server's own repository; terms for the data were found on neither page.
- **EmuSen's code.** `SceneMapping.Drawn` omits `animation`, `gamelistinfo` and `gameselector`, which are therefore
  loaded and not drawn. `ThemeCatalog` marks 12 video properties `VideoDeferred`. `GamepadManager` opens the first pad
  only. `RomLibrary` lists every file below the ROM folder, so a folder never shows: the library is what ES-DE calls
  folder flattening ("Folder flattening"), which ES-DE discourages. `EsdeMediaFolder.Find` looks only at
  `<system>/<type>/<stem>.<ext>`, and its type table has no `manuals` or `custom`. Mistress has no string resources: no
  `.resx` file exists and visible text is written in the code. DianaOS's `FrameRecorder` already runs `ffmpeg` as a child
  process to encode recordings, a precedent for the process route of §4.7.

### 21.2 Inventory

"ES-DE" cites `USERGUIDE.md` (UG) or `THEMES.md` (TH) by section. "Mistress" cites this plan, the settings reference
(SR) or code. Features §22 and §23 are building are listed at the end for completeness, marked, and not planned here.

| # | Feature | ES-DE | Mistress today | Gap |
|---|---|---|---|---|
| 1 | Game video in the theme | `video` element: delay, fade-in from black or transparency, iterations, `onIterationsDone`, audio, pillarboxes, scanlines (TH "video"); video volume and three audio switches (UG "Sound settings") | the static image only; 12 properties deferred (§4.7, §10.1 Q4, `ThemeCatalog`) | playback, its audio stream, its settings |
| 2 | Videos scraped | `ScrapeVideos` on by default (UG "Content settings") | not fetched (SR §4.60 "What it does not cover") | the kind, and its quota cost (§21.1: 32 of 39 games, 1.38 MB median) |
| 3 | Screensaver | Dim, Black, Slideshow, Video; after 5 min by default; controls (random, launch, jump); slideshow of favourites or a custom folder; game-info overlay (UG "Screensaver", "Screensaver settings") | none | all of it; Video waits on 1 |
| 4 | Media viewer | full screen: video, cover, back cover, title screen, screenshot, fan art, miximage, `custom`; left and right, triggers to the ends; settings (UG "Game media viewer", "Media viewer settings") | none in big picture; the desktop library's Screenshots view shows the player's own captures only | the viewer; the `custom` type |
| 5 | PDF manuals | scraped (`ScrapeManuals` on); viewed with pages, zoom and pan; `manual` badge (UG "Game media viewer", TH "badges") | none | scraping, a renderer, the viewer mode, the badge |
| 6 | Kid and Kiosk modes | Kiosk: menu reduced to volume, no metadata editor, collections or favourite toggling; Kid: kidgame games only, no options menu; unlock sequence (UG "UI modes") | none | the modes; kidgame is §23's field |
| 7 | Folders | shown as entries, entered with A; sorted on top; folder badge; folder link; `defaultFolderImage`; `gamelistinfo`'s folder icon (UG "Multiple game files installation", "Metadata editor", TH "grid", "gamelistinfo") | flattened (`RomLibrary`); the test library's NES and GB are all folders (§21.1) | folder entries, entering and leaving, sorting, the badge |
| 8 | Media of games in folders | `downloaded_media/<system>/<type>/<folder>/<stem>` (UG "Manually copying game media files") | `EsdeMediaFolder` and `MediaStore` use `<system>/<type>/<stem>` | an ES-DE tree for this library would not be read (predicted defect, P108) |
| 9 | Directories as files, `.m3u` | a folder named like a file launches the file of its name; `.m3u` for multi-disc (UG "Directories interpreted as files") | none | no consumer: every core is a cartridge core and the library holds no `.m3u` (§21.1) |
| 10 | Launch screen | shown on launch: Normal, Brief, Long, Popup or Disabled; follows the menu colour scheme and opening animation (UG "UI settings") | the resume question on a sheet, then the game (SR §4.52) | the screen; its content is not documented and must be measured |
| 11 | Built-in badge icons | ES-DE draws its own when a theme names none: nine slots, a folder-link overlay, 36 controller icons (TH "badges") | nothing drawn (§15.13); Q9 decided Mistress's own drawings | the drawings |
| 12 | Clock switch | `DisplayClock`, off by default (UG "UI settings"; §13.8) | off, no switch (§15.3, SR §4.52) | the switch |
| 13 | Several controllers | every pad drives the frontend; "Only accept input from first controller" off by default (UG "Input device settings") | the first pad only (`GamepadManager`, §15.13) | every pad; cores model two ports and a Player 2 mirror exists (`EmuSen_Input.md` §5.1, §6) |
| 14 | Controller popup | "Input device notifications", on by default | none (§14.11); LunaP has `NoticeLayer` (§88.5) | the notice |
| 15 | Controller type and button swap | seven icon sets chosen by hand; swap A/B and X/Y (UG "Input device settings") | automatic family from SDL (§15.5), no override, no swap | an override and a swap |
| 16 | Localisation | 21 locales, automatic from the OS (which the guide says fails in the Steam Deck's Game Mode), theme language (UG "UI settings", TH "Languages") | English only; no string resources; the loader reads theme languages (§12.4 item 7) | resources, a setting, translations |
| 17 | The theme list | the official list with screenshots, counts of variants and schemes, update, delete, "LOCAL CHANGES" detection (UG "Theme downloader") | one button, Art Book Next (SR §4.53, §16.8) | the list, other hosts, change detection |
| 18 | Other themes' elements | `animation` (GIF, Lottie), `gameselector`, `gamelistinfo`, vertical and wheel carousels, reflections, rotation, fade transitions, `stationary` (TH) | loaded, not drawn (`SceneMapping.Drawn`; §3.8 "Later") | as a survey of the list finds them used |
| 19 | TheGamesDB | a second source; no hash search in ES-DE's use (UG "Scraping") | none | a client; the API now has `ByGameHash` (§21.1) |
| 20 | Interactive match and search | "Interactive mode", "Refine search", auto-accept single matches (UG "Scraping process", "Other settings") | no name search at all (§5.2, SR §4.60) | an explicit search and a chooser |
| 21 | Refresh | "Overwrite files and data" (UG "Other settings") | never refetched (SR §4.60) | a refresh, cheap by checksum (§5.1) |
| 22 | More media kinds | back covers, 3D boxes, physical media, fan art, manuals (UG "Content settings") | folders exist, nothing fills them (SR §4.60) | the kinds (§21.1: 25–33 of 39 games offer each) |
| 23 | Scraped names | "Game names" scraped and shown | kept in `media.db`, not shown (§17.13) | a choice |
| 24 | Scrape criteria | All, Favourites, No metadata, No game image, No game video, Folders only (UG "Scraper") | a console and "Only games with no cover" (SR §4.60) | the other criteria |
| 25 | Quick system select | six choices; default left/right or shoulders (UG "UI settings") | left and right only (§15.3) | the choice |
| 26 | Shoulders in a list | "jumps 10 games in the gamelists" (UG "General navigation") | a page of the rows shown (§15.3) | a documented behaviour not followed |
| 27 | Startup and order | "System on startup", "Startup view", "Systems sorting" (UG "UI settings") | the first shelf, system view, release order (§15.3) | three settings |
| 28 | Quick-scroll overlay | two letters over a held list, off by default (UG "UI settings") | none | the overlay |
| 29 | System status toggles | Bluetooth, Wi-Fi, battery, battery percentage (UG "System status settings") | always shown when sysfs reports them (SR §4.52) | four switches |
| 30 | Help switch | "Display on-screen help" (UG "UI settings") | always shown | a switch |
| 31 | Navigation sounds | a volume; built-in sounds when a theme has none (UG "Sound settings") | gain fixed at 0.7; nothing when the theme has none (§15.6) | the volume and a fallback set |
| 32 | Per-game engine | "Alternative emulators", per system and per game; `altemulator` badge and filter (UG "Other settings", "Metadata editor") | per console only (Graphics Settings' Engine row, SR §4.44) | per game, the badge; the field belongs in §23's editor |
| 33 | Play-time cap | "Max play time tracking", 8 h (UG "Other settings") | play time recorded uncapped (SR §4.32) | a cap |
| 34 | Orphaned media | "Orphaned data cleanup" (UG "Removing orphaned data") | `games.db`'s orphan pass (SR §4.37); `media.db` follows renames (§17.4) | media of deleted games |
| 35 | Real-hardware checks | — | sign-in, Steam Input's family and Game Mode's compositing unmeasured (§15.13, §20.6, §14.10a) | a session on the device |
| — | Collections, filters, sorting, jump to letter, random game | UG "Game collections", "Gamelist options menu" | **§22, being built** | — |
| — | Options menu, metadata editor, hidden and kidgame flags | UG "Gamelist options menu", "Metadata editor" | **§23, being built** | — |

**Considered and not planned.** The quit menu's reboot, power-off and suspend (Steam owns power in Game Mode, and the
desktop's session does outside it); custom event scripts (no consumer, and a script runner is a security surface);
screen rotation, VRAM limit, MSAA, display index, application update checks, the game importer and the Android and
Windows settings (not applicable); menu colour schemes and the blur behind menus (LunaP's themes own the look, §10.1);
the miximage generator (Q7 declined it); the GPU statistics and debug overlays (ES-DE's own diagnostics).

### 21.3 The passes

Each pass is a unit that can be built, tested and recorded in one go, as a stage was. Costs are in §7's unit, working days,
at §7's scale. The record since then is shorter than §7 estimated: each of stages (a) to (f) was opened and closed within
one or two calendar days, against §7's 2–7 working days. P119 tests whether that holds. Every pass keeps the rules the
stages kept: every visible part is a LunaP control, nothing from ES-DE or a theme enters either repository, ES-DE is an
oracle run only from the scratch copy with its own `--home`, and a request to any server is made only in a run the
player started (§17.14). Tests run headless in WiseMan, blast radius only, and never as repeated broad or GPU runs
(the load rule of 2026-09-25).

**Pass 1. The hardware session.**
- *Scope.* On the Legion Go S, in Game Mode and in Desktop Mode: what SDL reports for the built-in pad under Steam
  Input, and the family `PadFamilies` gives it (P42's open half); the themed view's frame in a real window under
  gamescope against Desktop Mode (P28 was measured surfaceless, §14.10a); the navigation sounds' latency beside the game
  stream (§15.6); the help icons at arm's length. Then, by hand on the device: one member sign-in against
  the live service (§20.6), and, if Q35 allows, the four-file N64 run that settles P60 (about 20 requests).
- *Depends on.* Time with the device, and Q35. No code dependency.
- *Oracle and tests.* The device itself; a frame log in Mistress's themed render loop, written under an environment
  variable, read after the run. Long runs go under `systemd-run --user`, because the device ends processes an ssh
  session leaves.
- *LunaP.* None.
- *Risks.* The member password must be typed into Mistress by its owner; it is never written down or passed on. The
  sign-in and the N64 run are live requests, each started by hand.
- *Cost.* 1 day, and one session on the device.
- *What the player sees.* No new feature: a record of what the device does, the Game Mode frame §14.10a left unmeasured,
  and the open predictions P42 and P60 retired.

**Pass 2. Controllers: every pad, a popup, an override.**
- *Scope.* `GamepadManager` opens every connected pad and routes each to the frontend (inventory 13); a notice when a pad
  connects or goes, through LunaP's `NoticeLayer` (14); Preferences gains "Controller type" (Automatic, then the four
  LunaP families) and "Swap A/B and X/Y" (15), and "Only accept input from the first controller" (off, ES-DE's default).
  The help bar follows the pad last pressed. Whether a second pad also becomes player 2 in a game is Q22.
- *Depends on.* Q22. Nothing of §22 or §23: the code is Endymion's input and the window's pad poll.
- *Oracle and tests.* `SimulatedPad` extended to several pads; SDL's own added and removed events. ES-DE's popup, whose
  fades §14.7 saw but did not time, is recorded once with §14.7's uinput rig (P103).
- *LunaP.* A notice's content may want a `PadGlyph` beside its words (§103); nothing else.
- *Risks.* The blast radius leaves big picture: `GamepadManager` feeds every game's input (`EmuSen_Input.md` §4), so the
  input tests run too. Some wireless pads register twice (UG "Input device settings"); the first-controller switch is
  ES-DE's answer and is kept.
- *Cost.* 2–3 days.
- *What the player sees.* Any pad steers big picture; "Controller connected" and "disconnected" notices; icons that can
  be forced to a family.

**Pass 3. The theme list, and a survey of its themes.**
- *Scope.* The Themes tab lists ES-DE's official list (`themes-list/themes.json`, read at run time, never mirrored; 66
  themes on 2026-09-24, §6), with each entry's variant, scheme and ratio counts and its screenshots, fetched only when the
  picker is open. Any listed theme downloads from GitHub or GitLab through §16.4's checked swap; GitLab's archive URL is
  added. The stamp records every file's hash, so a theme edited in place reads "Local changes" and an update asks before
  replacing them (UG "Theme downloader"). Then a survey: the loader run over every listed theme's XML for EmuSen's five
  systems, counting errors, unknown properties, and the elements and properties used that the scene does not draw. The
  survey decides Pass 14's scope.
- *Depends on.* Q27 and Q28. Nothing of §22 or §23.
- *Oracle and tests.* A fake GitHub and GitLab, as §16.4's; the survey's own counts, recorded here; ES-DE captures of
  two or three surveyed themes on the synthetic library, only if Pass 14 is chosen for them.
- *LunaP.* None; the picker is `LunaList` and `FittedImage`.
- *Risks.* **Licences.** Most listed themes carry their authors' licences, many non-commercial and share-alike, some
  none. Nothing is redistributed: the list is read, a theme is fetched at the player's request, and its About sheet
  (§16.4) reads its licence at display time. The picker shows the licence line, or "states no licence", before the
  download button (Q27). **Load.** Downloading whole archives for a survey would fetch gigabytes (Art Book Next alone is
  220 MB, §16.7); the recommended survey fetches only XML files through the hosts' tree listings, under GitHub's 60
  unauthenticated calls an hour, into `~/.cache/emusen/bigpicture/survey/`, never the repository (Q28).
- *Cost.* 2–3 days.
- *What the player sees.* Every official ES-DE theme in the Themes tab, with pictures and licences, downloadable and
  updatable; a table here of which ones Mistress draws fully.

**Pass 4. Switches, and what the engine draws itself.**
- *Scope.* Built-in badge icons for a theme that names none (inventory 11): the nine slots, the folder-link overlay, and
  controller icons for the controller types EmuSen's consoles use plus generic and unknown, drawn as Mistress's own (Q9).
  The switches of rows 12 and 25–31: clock, help, the four status indicators, quick system select, the shoulders (Q32),
  startup system and view, systems order, the quick-scroll overlay, navigation volume, and a fallback navigation set
  drawn from Mistress's own sounds. Optionally row 32, the per-game engine with its `altemulator` badge, and row 33, the
  play-time cap.
- *Depends on.* §23 for the completed, kidgame, broken and controller fields the badges show, and for the per-game
  engine field; §22 for the collection badge. It edits the Theme Settings sheet and Preferences, which §22 and §23 also
  edit, so it starts after both are merged. Q32.
- *Oracle and tests.* UG's settings text for each switch's meaning; `THEMES.md`'s badge properties for layout; the
  quick-scroll overlay's timing recorded once from ES-DE. Each switch gets a pixel test in §15's style: the change lands
  inside its element's box and nowhere else (P105, P106).
- *LunaP.* A `BadgeGlyph` set beside `PadGlyph` (§103), drawn as LunaP's own geometry; a letter overlay for a held list.
- *Risks.* Small. A fallback navigation sound must be Mistress's own recording or synthesis, not ES-DE's samples.
- *Cost.* 2–3 days; the per-game engine 1 more.
- *What the player sees.* A clock on request, badges on any theme, the ES-DE settings a player expects in Theme Settings
  and Preferences.
- *Built 2026-09-27, rows 32 and 33 with it: §29.*

**Pass 5. Localisation's plumbing.**
- *Scope.* Every string Mistress shows in a big-screen session goes through one lookup, with a language setting
  (Automatic from the OS, then the chosen ones; ES-DE's guide notes that automatic detection does not work in Steam's
  Game Mode, so the setting is needed there). LunaP's own strings (the keyboard's keys, the sheets' buttons) get a
  provider a consumer can fill, because LunaP is a separate MIT package used by Pegasus too. The theme's language follows
  the application's (TH "Languages"). No translation is written in this pass: a pseudo-locale (every string bracketed
  and lengthened) is the test language.
- *Depends on.* Q26 for the languages, not for the plumbing. It follows Pass 4 and §22 and §23, so that their strings
  exist when the lookup is introduced, and it precedes the passes that follow, whose new strings are then held by its test.
- *Oracle and tests.* The pseudo-locale test: every sheet reachable in a big-screen session is walked with `PadAudit`
  and fails on any visible string that did not come through the lookup (P107). Widths: every sheet is rendered in the
  pseudo-locale at 1280×800 and checked for clipping, which §20.5 showed is the defect such text finds.
- *LunaP.* A string provider, with a `docs/LunaP.md` section and its API baseline.
- *Risks.* Churn: the pass touches most files that show text, so it is merged quickly and alone. Translations may not be
  copied from ES-DE (its locale files are MIT, but the project copies nothing of ES-DE).
- *Cost.* 3–4 days.
- *What the player sees.* Nothing changes in English: a language setting that offers English alone until Pass 15, and a
  pseudo-locale, for tests, that proves every string can change.

**Pass 6. Folders.**
- *Scope.* In the themed view, a console's games are shown as ES-DE shows them, with folders as entries that South
  enters and East leaves, folders on top (ES-DE's default), the folder badge and `defaultFolderImage`, and
  `gamelistinfo`'s folder icon once Pass 14 draws it; or flattened, per Q23. The media path rule of inventory 8: media of
  a game in a folder are looked for under the same folder, in an ES-DE tree and in Mistress's store, which is migrated
  once. Directories named like files and `.m3u` playlists wait for a disc core (Q24).
- *Depends on.* Q23 and Q24; §22 (sorting and filters within folders, and the jump index's folder entry); §23 (folder
  metadata and the folder link).
- *Oracle and tests.* UG's folder rules; ES-DE captures of a synthetic library with folders; a test that shows P108's
  defect on the unchanged reader before it is fixed, as §16.2 did for P52.
- *LunaP.* A folder indicator in `TextRowList` beside its favourite star (LunaP §101.8), if it lacks one; the grid's
  `defaultFolderImage` is mapped already (§16.5).
- *Risks.* The selection, the quick system select's memory and the return from a game are keyed by file today (§15.2);
  a folder adds a level to each key. The desktop library is not changed.
- *Cost.* 2–3 days; 1 more for directories as files and `.m3u` if Q24 asks for them now.
- *What the player sees.* NES opening on its 16 folders and GB on its letters, as ES-DE would show them, or the flat list
  kept by choice.

**Pass 7. Kid and Kiosk.**
- *Scope.* A UI mode setting, Full, Kiosk or Kid (UG "UI modes"). Kiosk reduces the menus to a volume setting, as
  ES-DE's does, and removes Theme Settings, the rest of Preferences, scraping, the metadata editor, collection editing
  and favourite marking; Kid shows only kidgame games and removes the options menu too. The unlock sequence, ES-DE's documented default (Up, Up, Down, Down, Left, Right, Left,
  Right, B, A), entered on the pad outside a menu, returns to Full. The in-game pad menu keeps what a player needs to
  leave a game.
- *Depends on.* §23's kidgame field and its editor, §22's filters and collections, Q25.
- *Oracle and tests.* UG's list of what each mode removes; for each mode, `PadAudit` over every sheet of §15.9 reaches no
  removed control (P110); a mutant per restriction.
- *LunaP.* None.
- *Risks.* This is a convenience, not a lock: anyone with a keyboard or the settings file leaves it. The section of the
  settings reference must say so.
- *Cost.* 1.5–2 days.
- *What the player sees.* A mode a child can be handed.

**Pass 8. ScreenScraper's extras.**
- *Scope.* The kinds of inventory 22 as switches, off by default (ES-DE turns videos and manuals on; §21.1's sizes and a
  10,000-a-day account argue against that here): back covers (`box-2D-back`), 3D boxes (`box-3D`), physical media
  (`support-2D`), fan art (`fanart`), manuals (`manuel`), and videos (`video-normalized`) only if Q20 says yes.
  Refresh (21): a kept file is asked again with its checksum, so an unchanged one costs a request and no bytes (§5.1's
  `MD5OK`). Search by name and a chooser (20), started only from
  "Find by name…" or a run the player sets to ask, never as an automatic fallback (§5.2). Scraped names (23, Q30). The
  criteria of 24 that Mistress can answer. Deleted games' media (34).
- *Depends on.* Q20 (videos), Q30; §23 for "exclude from scraper" and a folder's scraping. §17.14's rules bind all of it.
- *Oracle and tests.* The fake ScreenScraper of §17 and §20, extended to `jeuRecherche`, the checksum answers and the new
  kinds; the confirm step's plan gains each kind's cost from §21.1's offer rates (P112). One live run of about ten games,
  started by hand, measures what the fake cannot: the checksum answer's cost (P111).
- *LunaP.* None expected; the chooser is a `LunaList` of names with a thumbnail.
- *Risks.* **Quota and load.** Every kind is a request (P61), and the four picture kinds add about 3.2 requests and
  1.2 MB to a found game by §21.1's rates, nearly doubling a whole-library run; manuals add 0.7 requests and 1.2 MB, with
  a largest file of 18 MB. The defaults stay those of §4.60 and each new kind is the player's choice. **Terms.** A name
  search counts against the day's unrecognised allowance when it finds nothing (§5.2).
- *Cost.* 3–4 days.
- *What the player sees.* More kinds in the Scraping tab, a Refresh, and a "Find by name…" for the games hashes miss.
- *Built 2026-09-27: §38. Videos are among the kinds (Q20), off; a refresh costs one request a found game, because every
  answer states each file's SHA-1; the live run of about ten games was not made, since no request reaches ScreenScraper
  while a pass is built.*

**Pass 9. The media viewer and manuals.**
- *Scope.* A full-screen viewer over the themed gamelist (inventory 4), opened by the button Q32 settles (ES-DE's X,
  which is West, unused in §4.9's grammar): the game's pictures in ES-DE's order, then `custom`, left and right one at a
  time, the triggers to the ends, any other button to close; the manual mode (5) on Up with pages, zoom on the
  shoulders and pan while zoomed; the `manual` badge. Video enters the viewer in Pass 12.
- *Depends on.* Pass 8 for real back covers, fan art and manuals (the pass itself runs on synthetic media); Q31 for the
  PDF renderer; Q32 for the button.
- *Oracle and tests.* UG "Game media viewer"; ES-DE captures of the viewer on the synthetic library; synthetic PDFs of
  known page count and page sizes written by a WiseMan tool; P113 for a page's cost.
- *LunaP.* A pager of pictures, and a page view with zoom and pan that takes pages as bitmaps; the PDF renderer stays in
  Mistress, because LunaP takes no dependency beyond Avalonia (§13.2).
- *Risks.* **Licence of the renderer (Q31).** Poppler is GPL-2.0-or-later and PDFium BSD-style, both usable from a GPL-3.0
  program; MuPDF is AGPL-3.0, which would bring its network clause to the combination. These licences are stated from
  the projects' own statements as commonly published and are to be read again at the pass's start. Poppler's
  `pdftoppm` run as a process, as `FrameRecorder` runs `ffmpeg`, ships nothing. **Load.** A manual of 18 MB rendered at
  1920×1200 is memory: pages are rendered one at a time and kept only near the current one.
- *Cost.* 3–4 days; video in the viewer 0.5–1 more in Pass 12.
- *What the player sees.* West on a game shows its pictures full screen, and its manual.

**Pass 10. The screensaver.**
- *Scope.* Dim (dim and desaturate the view), Black, and Slideshow (the library's pictures, or favourites only, or a
  folder; the game-info overlay), after an idle time, 5 minutes by ES-DE's default; the controls (random game, launch,
  jump to the game), and the setting "Start screensaver after" with 0 for never. Video waits for Pass 12, and falls back
  to Dim meanwhile, as ES-DE does with no videos.
- *Depends on.* Q34. Pass 8 only for richer pictures.
- *Oracle and tests.* UG "Screensaver"; ES-DE recordings of Dim's level and the slideshow's swap and transition with
  §14.7's rig; the idle clock is the scene's own, so tests step it.
- *LunaP.* A dim layer and a cross-fading picture over `FittedImage`; the idle timer is the consumer's.
- *Risks.* Steam dims and sleeps the screen itself in Game Mode, so two savers could stack (Q34). A screensaver must draw
  nothing between swaps, as P39 required of a still view, or it costs the battery it exists to save (P114).
- *Cost.* 1.5–2 days; the video saver 0.5 more in Pass 12.
- *What the player sees.* The view dims, or a slideshow of the library, after five idle minutes.
- *Built 2026-09-27: §37. P114 failed on its first clause: ES-DE's own Dim and Black fade over about ten frames, so
  Mistress draws their fades too, and nothing after; the slideshow draws only through its changes.*

**Pass 11. The launch screen.**
- *Scope.* ES-DE's launch screen and its five durations (inventory 10), after the resume question has been answered and
  before the game's first frame (Q33), in both big-screen and desktop big picture.
- *Depends on.* Q33. Nothing else.
- *Oracle and tests.* `USERGUIDE.md` gives the durations' names and says nothing of the screen's content, so the pass
  opens by capturing and recording ES-DE's launch screen at each duration on the synthetic library (P115). Tests step the
  scene's clock through each duration and require the game to start at its end.
- *LunaP.* Possibly a full-screen card with a scale-up entrance, built from existing controls and a `Glide`.
- *Risks.* The launch path is `StartGameAsync` (§15.4), shared by every start; a screen that delays the start must not
  delay a resume from the pad menu.
- *Cost.* 1–1.5 days.
- *What the player sees.* The game's art and name for a moment before the game, as in ES-DE.
- *Built 2026-09-27: §33. P115 held, at the edges of its ranges; Normal is 3.0 s, Brief 1.7 s, Long 4.5 s.*

**Pass 12. Video (§7's stage g), waiting on Q20 and Q21.**
- *Scope.* A decoder behind one interface; the `video` element's deferred properties (delay, fade-in, iterations,
  `onIterationsDone`, `audio`, pillarboxes and their threshold, scanlines, video corner radius, `path` and `default`);
  the video's audio on a stream of its own beside `UiSoundPlayer`'s, SDL mixing both (§15.6); video volume and the three
  audio switches; then video in the viewer (Pass 9) and the Video screensaver (Pass 10).
- *Depends on.* Q20 and Q21; Pass 8 for scraped clips (the pass runs on synthetic clips); Pass 1 for Game Mode's frame
  budget.
- *Oracle and tests.* Synthetic clips written by `ffmpeg` in WiseMan's tools: a frame counter and a tone, at known sizes
  and frame rates, so a frame's number can be read back from the picture. ES-DE recordings with §14.7's rig for the
  delay, the fade and the iterations (P117). Decode cost measured on the handheld (P116, which carries P5).
- *LunaP.* A video surface that shows frames a consumer hands it (as `RgbaImageView` does), with the fade, pillarboxes and
  corner radius; the decoder stays in Mistress.
- *Risks.* **Codecs and licences (Q21).** ScreenScraper's clips are MP4 (32 of 32, §21.1); their codec was not measured,
  and H.264 is the usual one. FFmpeg is LGPL-2.1-or-later unless built with GPL parts; H.264 is covered by patents
  licensed through a pool in some countries. Running the system's own `ffmpeg` as a process ships no codec, and is what
  the desktop can do today (§21.1). Bundling FFmpeg for SteamOS would make EmuSen a distributor of a patented decoder,
  which is a legal judgement, not a technical one, and is not recommended. **Load.** A decoder process per clip on a
  handheld, and ES-DE recordings at 165 fps on the desktop: one run at a time, short, under `nice`. **Quota.** P120.
- *Cost.* 4–6 days, and 1–1.5 for the viewer and the screensaver.
- *What the player sees.* After three seconds on a game, its clip plays in Art Book Next's frame, once, with sound; clips
  in the viewer and the screensaver.

**Pass 13. TheGamesDB, waiting on Q29.**
- *Scope.* A second client, used only inside a run the player started, and only for games ScreenScraper left without
  something TheGamesDB has: its `ByGameHash` by MD5 and then CRC with the platform, then an explicit name search from the
  chooser of Pass 8; its images (front and back box, screenshot, title screen, clear logo, fan art); its allowance read
  from every answer and from `/v1/API/Limit`, which costs nothing.
- *Depends on.* Q29; Pass 8's chooser.
- *Oracle and tests.* A fake TheGamesDB built from the spec's documented answers; one live run of about ten games,
  started by hand, with a key.
- *LunaP.* None.
- *Risks.* **The key.** A key belongs to a logged-in site account (§21.1), so it is handled as the ScreenScraper developer
  file is: in a file outside the repository, mode 0600, passed through the redactor, never in a build (Q29). **Terms.**
  The data's terms of use were not found; the pass reads them first, and stops if they forbid the use. **Quota.** A
  monthly allowance, not a daily one, so a whole-library run is not an option on it.
- *Cost.* 2–3 days, plus the time to obtain a key.
- *What the player sees.* Games ScreenScraper misses, filled from a second source.

**Pass 14. The elements other themes use.**
- *Scope.* What Pass 3's survey finds used and undrawn, in order of the number of themes that use it: expected among them
  `gamelistinfo` (with §22's filter counts), `gameselector` (the system view's game pictures), `animation` (GIF, and
  Lottie), the vertical and wheel carousels with reflections, rotation, the fade transition and `stationary`.
- *Depends on.* Pass 3; §22 for `gamelistinfo`'s filtered counts.
- *Oracle and tests.* ES-DE captures and recordings of the surveyed themes that use each element, on the synthetic
  library, as §13.8 and §16.5 did for Art Book Next; `SceneMappingTests` extended to each new pair.
- *LunaP.* The carousel's other types, a frame-sequence image for GIF frames decoded in Mistress; Lottie needs Skottie,
  a SkiaSharp dependency LunaP may not take, so it would go to a sibling package or to Mistress, by `PLAN-icons.md`
  §1.1's rule.
- *Risks.* Scope that grows with each theme. The survey's counts bound it, and an element no chosen theme uses is not
  built.
- *Cost.* 3–8 days, as the survey decides.
- *What the player sees.* The themes the player picks from the list drawn as ES-DE draws them.
- *First half built 2026-09-27, `gameselector` and the wheel carousels: §36.*
- *Second half built 2026-09-27, `gamelistinfo`, `animation` (GIF), Q120–Q122 and two defects: §39.*

**Pass 15. Translations.**
- *Scope.* The languages Q26 names, one at a time, on Pass 5's plumbing.
- *Depends on.* Pass 5; Q26; last, so that the strings have settled.
- *Oracle and tests.* A speaker's review; the pseudo-locale test stays the guard; a clipping check per language at
  1280×800.
- *LunaP.* Its own strings, in the same languages.
- *Risks.* Quality, and provenance: a translation's source is recorded; none is copied from ES-DE.
- *Cost.* 1–2 days a language, plus review.
- *What the player sees.* Mistress in the chosen languages.

### 21.4 The recommended order, and why

| # | Pass | Cost (days) | Waits on |
|---|---|---|---|
| 1 | The hardware session | 1 | time with the device, Q35 |
| 2 | Controllers | 2–3 | Q22 |
| 3 | The theme list and the survey | 2–3 | Q27, Q28 |
| — | *§22 and §23 merged* | | |
| 4 | Switches and engine-drawn badges | 2–3 (+1) | §22, §23, Q32 |
| 5 | Localisation's plumbing | 3–4 | — |
| 6 | Folders | 2–3 (+1) | §22, §23, Q23, Q24 |
| 7 | Kid and Kiosk | 1.5–2 | §22, §23, Q25 |
| 8 | ScreenScraper's extras | 3–4 | Q20, Q30, §23 |
| 9 | The media viewer and manuals | 3–4 | Pass 8, Q31, Q32 |
| 10 | The screensaver | 1.5–2 | Q34 |
| 11 | The launch screen | 1–1.5 | Q33 |
| 12 | Video | 4–6 (+1–1.5) | **Q20**, **Q21**, Pass 1 |
| 13 | TheGamesDB | 2–3 | **Q29**, Pass 8 |
| 14 | Other themes' elements | 3–8 | Pass 3, §22 |
| 15 | Translations | 1–2 a language | Q26, Pass 5 |

The fourteen passes before translations come to **31–48 working days** at §7's scale, or 34–51 with the per-game engine,
directories as files, and video in the viewer and the screensaver; §7 estimated 17–27 for stages (a) to (g).

The reasoning, in the order of the table:
- **Passes 1 to 3 touch nothing §22 and §23 touch.** The hardware session writes no product code; controllers live in
  Endymion and the pad poll; the list and survey in `ThemeDownloads` and the loader. They can run while §22 and §23 are
  being built and merged, and each retires open items (the Game Mode frame, P42, P60) or produces the numbers later
  passes need: Pass 1's Game Mode frame for Pass 12's budget, Pass 3's survey for Pass 14's scope.
- **Pass 4 first after the merge** because it is cheap, shows at once, and its badges need §23's fields.
- **Pass 5 before the passes that add text.** Introduced last, it would retrofit the strings of ten passes; introduced
  here, its pseudo-locale test holds every later pass's strings as they are written. It comes after Pass 4 and the merge
  so that the lookup is laid over settled sheets.
- **Folders and Kid and Kiosk next,** since they change what the list is (a level of folders, a filtered set) and every
  later pass draws over the list.
- **Pass 8 before Pass 9,** because the viewer's only real inputs are the kinds Pass 8 fetches.
- **The viewer, the screensaver and the launch screen before video,** for two reasons. Video is the most expensive pass,
  the one with the legal question and the one waiting on a decision (Q20, Q21). And when it is built, two of its three
  consumers already exist, so the decoder is designed once against the element, the viewer and the saver together,
  rather than for the element and then bent.
- **TheGamesDB late,** because ScreenScraper found 39 of 40 by hash (§17.10), so a second source fills little, and it
  waits on a key.
- **Pass 14 late but movable:** if a theme picked from Pass 3's list needs an element, that element's part
  of Pass 14 moves up to follow Pass 3.
- **Translations last,** when the strings have stopped moving.

### 21.5 The decisions still to be made

- **Q20, video: Q4 revisited.** Q4 was "not now" (§10.1). What has changed since: ScreenScraper offers a clip for 32 of 39
  found games (§21.1), the handheld draws a full frame in under 2 ms at 1920×1200 (§14.10b), and three consumers now wait
  on a decoder (the theme's element, the viewer, the screensaver). Options:
  (a) keep it deferred, and Passes 9 and 10 ship without video;
  (b) theme video only, with scraping of `video-normalized`;
  (c) all three consumers.
  **Recommendation: (c), built as Pass 12 after Passes 9 and 10**, with clips off in the scraper by default because each
  adds about 0.8 requests and 1.1 MB to a found game (P120), and video audio on, as ES-DE's defaults have it
  (`ViewsVideoAudio` true), with the volume switch beside it. Pass 8's video kind, the video parts of Passes 9 and 10,
  and Pass 12 wait on this answer.
- **Q21, the decoder and its codecs** (only if Q20 is (b) or (c)). Options:
  (a) the system's own `ffmpeg`, run as a process (as `FrameRecorder` does), shipping nothing;
  (b) FFmpeg bundled, as an LGPL build;
  (c) bindings (`FFmpeg.AutoGen`, `Sdcb.FFmpeg`, §4.7) to the system's libraries.
  **Recommendation: (a).** No codec is distributed, so the patent and licence questions of a bundled H.264 decoder do not
  arise for EmuSen. Where no `ffmpeg` exists, the element shows its image, as today. Whether SteamOS provides one is
  measured in Pass 1; if it does not, bundling is asked again then, with that fact.
- **Q22, what "multiple controllers" means.** Options:
  (a) every pad drives the frontend (ES-DE's sense);
  (b) (a), and a second pad becomes player 2 in a game.
  **Recommendation: (a) in Pass 2; (b) as its own piece of input work,** recorded in `EmuSen_Input.md`, because cores
  model two ports and a Player 2 mirror already exists (§5.1 and §6 there), and port assignment reaches every game, not
  big picture alone.
  *(b) built 2026-10-04 as `EmuSen_Input.md` §8:* a pad for each player up to the running core's ports (four on the
  N64), the seating rule and its edge cases (§8.2, §8.3), and Preferences ▸ Controllers' players, which a big-screen
  session shows as menu rows the pad alone changes (§8.9).
- **Q23, folders.** The test library's NES games are all in 16 region and category folders and the GB games in 28 letter folders
  (§21.1). Options:
  (a) show folders, as ES-DE does;
  (b) keep the flat list, which is ES-DE's discouraged folder flattening;
  (c) show folders, with a per-console "flatten" switch.
  **Recommendation: (c), folders shown by default.** It is ES-DE's behaviour, and the switch serves the letter folders,
  which only repeat what jump-to-letter (§22) does.
- **Q24, directories as files and `.m3u`.** **Recommendation: wait for a disc-based core.** No core and no file in the
  library would use them (§21.1), so tests would be all that exercised them.
- **Q25, Kid and Kiosk.** Options: both; Kid only; neither. **Recommendation: both**, since they cost 1.5–2 days once §23
  exists, and a frontend handed to a child is what they are for. If no one in the house needs them, neither.
- **Q26, languages and translators.** Options:
  (a) the plumbing only (Pass 5), English only;
  (b) the plumbing, then languages named later, drafted and reviewed by a speaker;
  (c) the plumbing, and a file format a community could translate.
  **Recommendation: (a) now, (b) for any language named later.** No translation is taken from ES-DE.
- **Q27, which themes the list offers.** Options: every theme of ES-DE's list; only those that state a licence; only
  those Mistress draws fully after Pass 14. **Recommendation: every theme**, with its licence line or "states no licence"
  shown before the download, since the download is the player's, from the author's own repository, as it is in ES-DE.
- **Q28, the survey's download.** Options: every theme's archive (gigabytes); XML files only, through the hosts' tree
  listings; a hand-picked sample. **Recommendation: XML only** (P104 predicts under 50 MB), kept under
  `~/.cache/emusen/bigpicture/survey/`.
- **Q29, TheGamesDB.** Options:
  (a) not at all;
  (b) the author's own key, in a file outside the repository, used only on the author's machines, as Q5's answer did for
  ScreenScraper;
  (c) (b), and a box where a player enters a key of their own.
  **Recommendation: (c) if the terms read in Pass 13 allow it, else (b)**, since a key comes from any site account
  (§21.1), unlike ScreenScraper's developer credentials. Pass 13 waits on this answer.
- **Q30, scraped names.** ES-DE shows scraped names. Options: show them; keep the file's name; a switch. **Recommendation:
  a switch, off**: the library's names are No-Intro's, already exact, and their tags tell two copies of a game apart,
  which ScreenScraper's names do not.
- **Q31, the PDF renderer.** Options: Poppler's `pdftoppm` as a process; PDFium bundled; MuPDF (AGPL-3.0). **Recommendation:
  Poppler as a process**, shipping nothing, with PDFium as the fallback if SteamOS lacks Poppler. MuPDF is not
  recommended, for its licence's network clause.
- **Q32, two buttons.** ES-DE opens its media viewer with X (West), which Mistress's grammar leaves free (§4.9); and its
  guide says the shoulders "jump 10 games in the gamelists", where Mistress pages by the rows shown (§15.3). Options for
  the second: keep the page; ten games, as documented. **Recommendation: West for the viewer, and ten games** (with
  quick system select on left and right, ES-DE's default for a list). If §22 or §23 have claimed West by then, this is
  asked again.
- **Q33, the launch screen.** Options: ES-DE's default, Normal; Brief; Popup; off. And its place: after the resume
  question, or before it. **Recommendation: Normal, after the resume question**, so the question is not hidden behind a
  timed screen.
- **Q34, the screensaver in Game Mode.** Steam dims and sleeps the handheld itself. Options: on after 5 minutes, as ES-DE's
  default, everywhere; on for the desktop and off in Game Mode; off. **Recommendation: on everywhere, type Dim until
  videos exist, and off in Game Mode if Pass 1 finds the two stacking.** Whether Steam's own dimming starts over an
  application that draws nothing, as a still themed view does (P39), is not known; Pass 1 looks.
- **Q35, the hardware session.** When can it be run, and may it include one sign-in with the author's own member
  account, typed by hand, and the four-file N64 run of about 20 requests that settles P60? **Recommendation: yes to
  both, in one sitting.**

### 21.6 Predictions

Written before any pass is built, to be retired in each pass's record.

| # | Pass | Prediction | Retired when |
|---|---|---|---|
| P100 | 1 | In Game Mode, SDL reports the Legion Go S's pad as Steam's virtual pad (its name or an Xbox type), and `PadFamilies` gives Xbox with no change to §15.5's rules | Pass 1 |
| P101 | 1 | Game Mode's compositing adds under 1 ms to the median frame of the held carousel at 1920×1200 against Desktop Mode, and no frame in 1,800 exceeds 16.7 ms | Pass 1 |
| P102 | 1 | A navigation sound starts within 100 ms of its press while a game is suspended behind the library, the 4,096-frame device buffer of §15.6 included, as SDL's queue and buffer sizes report it | Pass 1 |
| P103 | 2 | Every pad rule of §15.3 holds from either of two pads, and the help bar follows the pad last pressed; ES-DE's device popup fades in and out over 0.4–0.6 s each and holds 2–5 s | Pass 2 |
| P104 | 3 | Of ES-DE's listed themes, at least 90% load for EmuSen's five systems with no loader error, and at least half use an element or carousel type Mistress does not draw; the XML-only survey fetches under 50 MB | Pass 3: failed on all three clauses (§25.9) |
| P105 | 4 | Built-in badges change no pixel of Art Book Next's gamelist, which names its own icons; on a synthetic theme that names none, each of the nine slots draws | Pass 4 |
| P106 | 4 | Turning the clock on changes only pixels inside the clock's box, as P43 found for the help bar | Pass 4 |
| P107 | 5 | At the pass's start, the pseudo-locale walk finds 300–900 distinct visible strings outside the lookup in a big-screen session's sheets; at its end, none | Pass 5 |
| P108 | 6 | `EsdeMediaFolder` finds none of the media of a game in a subfolder of an ES-DE-written tree (a defect predicted from the code and UG's path rule), shown by a test on the unchanged reader before the fix | Pass 6 |
| P109 | 6 | Shown as folders, NES opens on 16 entries and GB on 28; the return from a game in a folder comes back to that folder and game, its first frame equal to a fresh build (P40's test); the first showing costs within 10% of the flat one | Pass 6 |
| P110 | 7 | Kid mode lists exactly the kidgame games, and in Kiosk and Kid `PadAudit` reaches no removed control on any sheet of §15.9 | Pass 7 |
| P111 | 8 | Refreshing a found game whose files are unchanged costs one request per kind and under 1 KB received per kind, each request counted in `requeststoday` (P61) | Pass 8: retired by design, one request a game (§38.13) |
| P112 | 8 | Back covers, 3D boxes, physical media and fan art together raise a found game from 4.7 to 7.5–8.5 requests and add 0.9–1.5 MB, taking a found game from about 13 s to 20–28 s at 128 KB/s | Pass 8: held by arithmetic on §17.9's answers, not run (§38.13) |
| P113 | 9 | One page of a median ScreenScraper manual (1.7 MB) renders at 1920×1200 in under 300 ms on the desktop and under 1 s on the handheld | Pass 9 |
| P114 | 10 | Dim and Black draw at most one frame after they start, and the slideshow draws only at its swaps and their transitions (every 10 s by ES-DE's default) | Pass 10: failed on the first clause, held on the second (§37.3) |
| P115 | 11 | ES-DE's launch screen at Normal lasts 1.5–3 s; Brief is 0.4–0.6 of that and Long 1.5–2.5 times it | Pass 11 |
| P116 | 12 | ScreenScraper's `video-normalized` clips are H.264 in MP4, no larger than 640×480, and one decodes in under 5% of a Legion Go S core (P5, carried) | Pass 12 |
| P117 | 12 | ES-DE starts a clip at the element's `delay` to within one frame at 60 Hz and fades it from black linearly over `fadeInTime` (1 s by default) to within 5% | Pass 12 |
| P118 | 13 | TheGamesDB's `ByGameHash` by MD5 identifies fewer than half of a 40-file sample, where ScreenScraper found 39 (§17.10) | Pass 13 |
| P119 | all | Each pass is opened and closed in no more calendar days than the lower end of its estimate, as stages (a) to (f) were against §7 | each pass |
| P120 | 8, 12 | Fetching `video-normalized` adds 0.7–0.9 requests and 0.9–1.3 MB to a found game on average (§21.1's offer rate and median), about 9 s more a game at 128 KB/s | Pass 12's first live run |

### 21.7 What this section did not do

- **ES-DE was not run,** and its source was not read. Two behaviours the passes need are undocumented and are the passes'
  first measurements, not this plan's: the launch screen's content (Pass 11) and the device popup's timing (Pass 2).
- **`INSTALL.md` and `FAQ.md` were not read,** so the command-line options, event scripts and controller profiles they
  document were judged only from the guide's mentions of them.
- **The theme list was not re-read.** Its size, 66, is §6's count of 2026-09-24.
- **Nothing was downloaded from ScreenScraper or TheGamesDB.** §21.1's media sizes are the servers' declarations; the
  video codec is unknown.
- **The handheld was not touched.** Whether SteamOS carries `ffmpeg` or Poppler, on which Q21 and Q31 lean, is Pass 1's.
- **The licences of FFmpeg, Poppler, PDFium and MuPDF** are stated as those projects commonly publish them, not re-read
  for this section; each pass that takes one reads it first. TheGamesDB's data terms were not found.
- **The costs are estimates** at §7's scale, and §7's scale has run long (P119).

## 22. Collections, and a game list's sort, filters, jump and random game (2026-09-26)

*Opened 2026-09-26, on request that day* to build what big picture lacked against ES-DE: item 1,
collections, ES-DE's automatic collections (All Games, Favorites, Last Played) and custom collections; item 2, a game
list's filters and sorting, plus jump-to-letter and random game. Item 3, the per-game options and a metadata
editor, is §23's, built at the same time on another branch. The player's account is §4.58 of the settings reference;
this is the record.

**Sources.** ES-DE's behaviour is taken from its `USERGUIDE.md` (the sections *General navigation*, *Game collection
settings*, *UI settings*, *Gamelist options menu* and *Game collections*) and `THEMES.md` (*System variables*, the
`textlist`, `helpsystem` and `metadataElement` entries), the copies fetched at master for stage (c) under
`~/.cache/emusen/bigpicture/motion/docs/`. Where they are silent, ES-DE 3.4.1 was run (§22.2). Its source was not read.
Art Book Next's files were read, as in every stage, only to learn which names it asks for (`auto-allgames` and the
others); nothing of it or of ES-DE entered either repository.

### 22.1 Predictions

Written after the code and its pad tests, and before the broad run and the cost measurements below, which they are
about. They are therefore narrower than a stage's.

- **P68, the broad run.** The Mistress filter without `ShaderSettingsWindowTests`, `ShaderBrowseBench` and
  `SceneGpuBench` passes with no failure that this branch causes.
- **P69, a large library.** With 3,508 games over the five systems and the three automatic collections on, a warm
  `ThemedLibrary.Show` on the desktop costs under 1.5 times the same Show without them (median of five). The
  collections' shelves are built from lists the window already holds, and their media presence is asked once per source
  system.
- **P70, a step in all games.** A step in the 3,508-game all games list costs within 1.5 times a step in a system's
  list: the listed games are kept per list, so a step does not sort again.
- **P71, the options sheet.** Opening Gamelist Options over the 3,508-game all games list, its letters and its filter
  values computed from the list, takes under 100 ms on the desktop.

### 22.2 ES-DE 3.4.1, measured where its documentation is silent

The documentation names the collections and describes their settings, but not the system names a theme sees for them,
their place in the carousel, what the grouped system's entries show, or the order inside the automatic collections. The
local AppImage was run for these, in the scratch `--home` only, one window at a time, eight runs of 15–25 s,
each closed by PID, with the scratch settings, gamelists and collections restored afterwards and compared with
`diff -r`. The runner, timelines, fixture and every capture are under `~/.cache/emusen/bigpicture/collections-probe/`.
The fixture was synthetic: scratch ROM files of stage (b)'s names, three favourites, four games with a `lastplayed` and
one with a play count and no date, and two custom collections (`custom-Platform.cfg`, three SNES games; `custom-Beat.cfg`,
one NES and one SNES game), enabled with `CollectionSystemsAuto=all,favorites,recent` and
`CollectionSystemsCustom=Platform,Beat`. The captures quoted below were looked at by this section's author too.

**A defect of the harness, found and fixed.** The desktop's own Xbox controller was connected, and ES-DE, started with
`SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1` as stage (c)'s `record.sh` starts it, acted on that controller's input during
the first two runs: extra carousel steps, and a jump from one gamelist to another. Those runs were discarded. From the
third, the runner sets `SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT` to the virtual pad's vendor and product, and the log
shows only that pad. Stage (c)'s and stage (f)'s recordings (§14.7, §16.5) were made with the same script and without
that variable; whether a real controller was sending input during any of them is not known, and is recorded here as a
hazard to those measurements, not as a finding against them.

**What was found.**
- *Names.* The tokens `all`, `favorites` and `recent` enable the three automatic collections, and the log populates
  systems of those names and `collections` for the grouped custom collections. Art Book Next drew each with its
  `${system.theme}` art: `auto-allgames`, `auto-favorites`, `auto-lastplayed` and `custom-collections` (the last also
  parsed from the theme's own `custom-collections/theme.xml`). The menu names them All Games, Favorites and Last Played.
- *Order.* In the system view, from SNES rightwards: COLLECTIONS, ALL GAMES, FAVORITES, LAST PLAYED, Nintendo 64, NES,
  Game Boy, Game Boy Color, SNES. The regular systems come first, sorted by full name, then the grouped system, then the
  three automatic collections in that order (`runs/r3/sheet.png`).
- *The grouped system.* One entry per collection, named as its file without `custom-`, sorted by name although the
  setting listed them the other way round. The selected entry shows one game's cover and the description "This
  collection contains 3 games: 'Hollow Comet [SNES]', 'Ember Circuit [SNES]' and 'Aurora Drift [SNES]'", the names in
  an order that changed between two visits, the first of them the game pictured. No folder mark, no rating, date,
  players or badge is drawn for an entry (`runs/r2b/g03`, `g05`). A opens the collection, whose games are listed by
  name with no star and no suffix; B returns to the list at that collection; B again to the system view.
- *Last played.* The most recent first, across systems; the game with a play count and no date was listed last; games
  never played were not listed (`runs/r5/f01`).
- *All games and Favorites.* All games lists the favourites first, each with its star, then every other game by name;
  games of the same name follow the systems' full-name order, which cannot here be told from the order the systems were
  loaded in. Favorites lists its games by name with no star before them (`runs/r4/a00`, `runs/r5/f00`).
- *The gamelist options.* The Back button opens GAMELIST OPTIONS: JUMP TO… with the selected game's letter (a star for
  a favourite on top) changed by left and right, SORT GAMES BY (NAME, ASCENDING), FILTER GAMELIST, EDIT THIS GAME'S
  METADATA, then APPLY and CANCEL. The help bar reads "Close (Cancel)" on Back and "Close (Apply)" on B (`runs/r5/f04`).
  JUMP TO's list holds only the characters present: with Kestrel Run the only K game and a favourite, K was absent
  (`runs/r6/j01`, `j02`).
- *Random.* The left thumbstick click moved the selection to another game at each press (`runs/r6/j05`–`j07`).
- *Beside the question.* With `QuickSystemSelect=leftrightshoulders` and a textlist, left and right changed the system
  and the shoulders moved ten rows (`runs/r4`), which is what USERGUIDE's *General navigation* says. §4.9 gives the
  shoulders a page of the rows shown (§15.3); that difference is not this section's to change, and is Q14.

### 22.3 What was built

**The data, with no Avalonia type** (`EmuSen.Mistress/BigPicture/`):
- `CollectionShelves` builds the collections from the library's shelves and Mistress's custom collections: all games,
  favorites and last played (the 50 most recent, a counted game with no date last) as automatic-collection systems;
  each custom collection as a system of its own when the theme has a folder of its name holding `theme.xml` or grouping
  is *Never*, else as a folder of the one `collections` system; the order of §22.2; ES-DE's name rules (`Clean`,
  `Unique`); and a grouped entry's description and pictured game (`Folder`).
- `GamelistOptions`: the eleven sort keys, each way, ES-DE's stored spellings (`GameSort.Stored` is `name, ascending`);
  the filter fields Mistress has data for, their values from a list's games, and `Passes`; the quick selector's
  `Letters`; `RandomIndex`.
- `ThemedLibrary.Gamelists.cs`, a new part of `ThemedLibrary`: per-list sorts and filters for the session, the folder
  open in the grouped system, the collection being edited, the random entry, `Letters`, `ApplyOptions`, and the listed
  games kept per list until something changes.
- `SceneGame` gained `Source` (the system a game belongs to), `Face` (the game a grouped entry pictures), `IsCollection`
  and `HideMetadata`; `SceneSystem` gained `Stars` and `Heading`. Media, the name suffix and `sourceSystemName` now read
  the game's own system (`SourceIn`); before, a collection's games would have been looked up under the collection.
- `ThemedLibrary.cs` changed in four places: `ThemedShelf` gained the collection's rules; `Data` lists each system
  through `Listed`; the cursor is kept per list; and `Command` gained the folder, North-while-editing and random cases.

**The window** (`Views/MainWindow.BigPictureCollections.cs`) builds the collections from `games.db` for each showing,
applies the settings, answers North while editing, adds the two pad-menu entries, and implements
`ICollectionSettingsHost`, through which the settings sheet creates, deletes and finishes editing. It changes the shared
files by one line each: `ShowThemedLibrary` passes `WithCollections(ThemedShelves())` and applies the settings first;
`ThemedPadCommand` gains the collection case; `OpenPadMenu` calls `AddCollectionMenuEntries`; `PadHeld` maps the two
thumbstick buttons to the new `UiButton.Random`.

**Three sheets**, LunaP `ToolWindow`s of LunaP parts (`FieldRow`, `Dropdown`, `LunaSwitch`, `HintText`, `Ui` rows and
buttons), presented by `SheetLayer.Show` as every sheet is: `GamelistOptionsWindow` (an `IPadDriven` window, so that the
Back button cancels as ES-DE's does), `GamelistFilterWindow` (a sheet over the options sheet, as ES-DE's filter screen is
a screen of its own), and `CollectionSettingsWindow`.

**In LunaP** (branch `bigpicture-collections`, its `docs/LunaP.md` §150 and §151): `TextRowMarker.Tick`, a stroked
check mark in the room the star and the folder use, and `PadGlyphButton.ThumbstickClick`, a disc in a thin ring, both
appended so no value moves, with a test each and the API baseline updated.

**Nothing holds a timer or a stream.** The sheets poll nothing and open nothing, and the random entry is a
`System.Random`, so §15.14's rule has no new subject here; no close test was written, since one that cannot fail
without a cleanup proves nothing.

### 22.4 Where the data comes from, and where it is kept

It was decided during the work that the program's data be kept in SQLite where it can be, as `EmuSen_Stack.md` §4
settles: what the program writes and reads back in SQLite, the player's settings in `appsettings.json`.
- **Already in SQLite, and read from there:** favourites, last played, play count, play time, the custom collections and
  their games are `games.db`'s `game`, `collection` and `collection_game` tables (`GameRecords`). Nothing of them was in
  a JSON file before this work, and none is now. A collection made or edited in big picture writes those rows, so the
  library's sidebar shows the same collection.
- **ScreenScraper's text** (genre, players, rating, release date, developer, publisher) is `media.db`'s, read through the
  window's `WithScrapedText` as stage (d) built it.
- **The player's choices** are the eight keys of `BigPictureCollections` in `appsettings.json` (§4.58 of the settings
  reference). `HiddenCustomCollections` names `games.db` ids; a collection deleted from the library's sidebar leaves its
  id there, which matches nothing and is dropped by nothing. That is harmless and was left.
- **A list's sort and filters** are kept in memory for the session and never written, as ES-DE keeps them. Nothing is
  cached on disk: the sorted and filtered lists are recomputed from the records at each showing.

**A game without ScreenScraper's text** has no genre, players, rating, release date, developer or publisher. In a
filter it is **Unknown**, a value of its own, so the unscraped games can be chosen or left out; ES-DE's documentation
says only that the values are assembled from the gamelist. By any of those sort keys it goes after every game that has
the value, in either direction, and among such games the name decides. Completed, kidgame and broken are not recorded by
Mistress, so each filter says "Nothing to filter"; hidden, controller and alternative emulator are not offered.

### 22.5 The buttons

§4.9's table was kept and amended (§4.9). The changes, each taken from what ES-DE's documentation assigns:

| Button | ES-DE (USERGUIDE) | Here |
|---|---|---|
| Y / North | toggles a favourite; adds or removes a game while a collection is edited | adds or removes while a collection is edited; otherwise the search, as before (§4.9 has no favourite on North) |
| Thumbstick clicks | a random game, or system, per *Random entry button* | the same, through `UiButton.Random` |
| A on a grouped collection | enters it | the same; the help bar reads Select |
| B inside it | back to the list | the same, at that collection |
| Back | opens the gamelist options | **not taken**: Back is §23's game options. The options are in Start's pad menu. |
| B / Back in the options | apply / cancel | the same |

**The Back button, and the merge with §23.** In ES-DE the gamelist options menu is one menu, opened with Back, holding
the jump, the sort, the filter, the collection entries and *Edit this game's metadata*. The plan gave Back to §23's game
options, which will hold the favourite toggle and the metadata editor. The two are therefore one menu in ES-DE and two
here until the branches meet. `GamelistOptionsWindow` takes its rows from a model the window fills
(`ShowGamelistOptions`), so the merge can open one sheet under Back with both sets of rows, or keep two; Q12 asks which.

### 22.6 Where the documentation is silent and nothing was measured: the choices made

- **The automatic collections are off by default**, as ES-DE's settings file has them; the custom ones are shown until
  switched off, unlike ES-DE's, because they are the player's own library's (Q11, Q13).
- **A discrete custom collection's place** is after the grouped system and before the automatic ones. Only the grouped
  system's place was measured.
- **"Themed" means a folder of the collection's name holding `theme.xml`**, THEMES.md's per-system folder. Whether ES-DE
  also counts a root `theme.xml` that styles every system by `${system.theme}` was not measured; if it did, every
  collection would be discrete under Art Book Next, which the grouped list in §22.2 contradicts for `Platform`.
- **Missing values sort last** in both directions, and **Unknown is a filter value**, as §22.4 says.
- **Within a field any chosen value passes, across fields all must**, the reading USERGUIDE's list of filters allows.
- **The quick selector's characters** are the first character of each name, upper-cased, digits included; ES-DE's
  treatment of digits and symbols was not measured.
- **The random entry never picks the game already selected**, and plays the list's scroll sound. Neither was measured.
- **A grouped entry's pictured game** is chosen each time the list is built, not at each step as ES-DE's apparently is,
  and Y does not jump to it.
- **A grouped entry hides** its rating, dates, badges and text fields other than the description, as §22.2 saw; the rule
  is THEMES.md's *Hide metadata fields*, applied to these entries only.
- **At the grouped system's top, `systemName` and `systemFullname` are blank**, which THEMES.md implies when it says a
  theme's `defaultValue` fills them "at the root of the custom collections system"; inside a collection they are its
  name. Art Book Next's second "COLLECTIONS" line under the logo disappeared with this (§22.9).
- **The help bar's A reads Select on a folder.** ES-DE's reads Select on every entry, games included (`runs/r4/a00`),
  where §15's table says Launch; that older difference is not this section's.

### 22.7 Tests

Headless in WiseMan, the pad through `PadDriver` over a `ThemedSession` on the synthetic theme, with `games.db` written
through the window's own `GameRecords` and ScreenScraper's text put where `WithScrapedText` reads it.

| Rule | Test |
|---|---|
| The sort keys, each way; missing values last; the name after; favourites on top; System by the source system; ES-DE's spellings | `GamelistOptionsTests.Every_documented_sort_key_orders_with_missing_values_last_and_the_name_ascending_after` |
| Filter values from the list, Unknown, Nothing to filter, stars; any value within a field, every field; the name in any case; a folder always passes | `…Filter_values_come_from_the_list_and_a_field_with_nothing_to_tell_apart_offers_none` |
| The quick selector: only the characters present, the star, a list of favourites alone | `…The_quick_selector_offers_only_the_letters_present_and_a_star_for_favourites_on_top` |
| The random entry never the selected game | `…The_random_entry_never_picks_the_game_already_selected_when_there_is_another` |
| ES-DE's name rules | `…Collection_names_lose_ES_DE_s_forbidden_characters_and_a_taken_name_is_numbered` |
| The shelves' order, grouping by theme folder, Always and Never, hidden collections, favorites' members and stars | `…Collection_shelves_follow_ES_DE_s_measured_order_and_group_what_the_theme_does_not_style` |
| Last played: newest first, fifty, a counted game with no date last | `…Last_played_is_newest_first_across_systems_fifty_at_most_with_a_counted_game_without_a_date_last` |
| A grouped entry's description and pictured game | `…A_grouped_collection_s_folder_names_its_games_in_a_random_order_and_shows_the_first` |
| Off until turned on; then ES-DE's order, names and theme folders | `ThemedCollectionsTests.Automatic_collections_are_off_until_turned_on_then_follow_the_systems_in_ES_DE_s_measured_order` |
| All games, favorites and last played from `games.db`; a game's picture and suffix under its own system | `…All_games_favorites_and_last_played_come_from_Mistress_s_records_and_their_games_keep_their_own_system` |
| The Collections list: folders by name, A in with its sound, B out to the same folder, B to the systems | `…The_grouped_collections_system_lists_each_collection_as_a_folder_that_A_enters_and_B_leaves` |
| An entry: no folder mark, metadata hidden but the description, a blank system name, Select | `…A_collection_s_entry_is_drawn_as_ES_DE_draws_it_no_folder_mark_its_metadata_hidden_and_a_blank_system_name` |
| A theme folder makes a system; Always, Never, a hidden collection | `…A_collection_the_theme_has_a_folder_for_is_a_system_of_its_own_and_the_setting_can_group_it_or_part_all` |
| Created on the sheet by the pad, cleaned, the sidebar's own; North adds and removes with the tick and the help's word; Finish Editing; North searches again | `…A_collection_created_in_big_picture_is_the_library_s_own_and_North_edits_it_until_its_editing_is_finished` |
| The options sheet: Back cancels, B applies; name and rating sorts from ScreenScraper's text; a filter; each list its own; reset; the jump; nothing saved | `…The_options_sheet_sorts_filters_and_jumps_B_applying_and_Back_cancelling_and_each_list_keeps_its_own` |
| Either stick; the help's Random; Disabled; Games and systems | `…Either_stick_pressed_in_jumps_to_another_game_and_the_setting_decides_where_it_works` |
| The settings sheet's switches and choices applied at once and saved | `…The_collection_settings_sheet_applies_each_switch_and_choice_beneath_it_at_once_and_saves_them` |
| Every control of the three sheets reached by the pad (`PadAudit`), no window opened | `…Every_control_of_the_collection_sheets_is_reached_by_the_pad` (3 cases) |
| `collectionIndicators`, symbols against ascii, changes the pixels | `SceneMappingTests.Every_mapped_property_changes_the_rendered_pixels`, a new case over a new custom-collection system |

In LunaP: `ThemedListTests.A_tick_marks_a_row_in_the_star_s_room_with_its_own_shape` and
`PadGlyphTests.The_thumbstick_click_is_a_disc_in_a_ring_unlike_the_guide_and_is_named_for_both_sticks`.

**Every one passed at its first run** except two, both fixed in the test: a tuple of arrays compared by reference, and a
sort stepped from the wrong starting row. That so few failed first is weak evidence of the tests' strength; the mutants
(§22.8) are the stronger.

*Amended after the mutants:* two tests were strengthened when their mutants survived (C25, C50 in §22.8), and a line of
code was removed as redundant (C13).

### 22.8 Mutants

The runners are `~/.cache/emusen/probe/bigpicture/mutate_collections.py` (56 mutants in Mistress and Galaxia, against
`ThemedCollectionsTests`, `GamelistOptionsTests`, `ThemedLibraryPadTests` and `SceneMappingTests`) and
`mutate_collections_lunap.py` (8 in LunaP, against `PadGlyphTests` and `ThemedListTests`). Each mutant was built and run
alone under `nice -n 10`, with builds at `-m:2`, and its file restored; the tree was rebuilt clean after each round. The
logs are `run-collections-part1-C1-C19.log`, `run-collections-part2.log`, `run-collections-part3.log`,
`run-collections-rerun.log`, `run-collections-rerun2.log`, `mutants-collections-lunap-first.txt` and
`mutants-collections-lunap.txt`.

**Two interruptions, and what they left.** The first round was cut off at 17:58 by a hardware reset of the desktop,
an AMD fault that heavy load triggers, as the freezes of §15.14 were; the machine's own log was not read for this
section. The mutant in flight, C21,
was left in `IndicatorElements.cs`; it was found and restored before anything else was built, the tree was rebuilt
clean, and the round was resumed from C20, whose result had not been written. The runner was then changed to keep the
original of the file it mutates beside it and to put it back on its next start. The second interruption was a pause
for a firmware update, taken during C49; the runner's own backup restored that file, and the round resumed from C49 after
a clean rebuild. No verdict below comes from a build that held another mutant.

| # | Rule broken | Result |
|---|---|---|
| C1 | the automatic collections on by default | caught by 6 |
| C2 | custom collections after the automatic ones | caught by 3 |
| C3, C4 | favorites holds every game; favorites draws its stars | caught |
| C5–C7 | last played oldest first; no limit; a counted game with no date left out | caught |
| C8–C10 | a themed collection grouped; Always ignored; a hidden collection shown | caught |
| C11, C12 | the folders in reverse; B inside a collection goes to the systems | caught |
| C13 | leaving a collection does not set the top list's cursor | **survived: equivalent**; the line was removed |
| C14–C16 | A launches a collection; a folder pictures another game than the first named; the description names no source system | caught |
| C17–C21 | a grouped entry: its folder mark kept; its metadata shown; the description hidden too; its system name kept; A reads Launch | caught |
| C22, C23 | a collection's game looked up, or suffixed, under the collection | caught |
| C24 | the sort not applied | caught by 3 |
| C25 | every list takes the current list's sort | **survived**; caught after the test was strengthened |
| C26–C35 | the filter not applied; missing values first; no name after the key; favourites not on top; all of a field's values required; the name filter minding case; a field with one value offered; no Unknown; favourites' letters beside the star; no star | caught |
| C36–C38 | the jump not made; Back not cancelling; B not applying | caught |
| C39–C42 | the random entry staying put; ignoring Disabled; moving systems with Games only; the right stick unmapped | caught |
| C43–C49 | North searching while editing; never taking a game out; no tick; members unmarked; a name keeping the forbidden characters; a new collection not edited; Finish Editing ignored | caught |
| C50 | the help bar's North keeps Search while editing | **survived**; caught after the test was strengthened |
| C51–C56 | no Random in the help bar; a settings change waiting for the next showing; a switch storing nothing; Sort favorites ignored for the systems; the default sort ignored; the heading ignored inside a collection | caught |
| T1–T4 | LunaP's tick drawn as the star, as the folder, keeping no room, its lowest point raised | caught (T3 was written ambiguously first, matching two lines, and rewritten) |
| G1–G4 | the thumbstick drawn as the guide; no ring; the disc filling the ring; named as the guide | caught |

**63 of 64 caught; the one survivor was equivalent, and the code it mutated is gone.** Two were caught only after
their tests were strengthened:
- **C13.** Leaving a folder set the top list's cursor to the folder left. The top list's cursor already held it: it was
  remembered when A was pressed on that folder, and nothing inside the folder can change the top list's key. The
  assignment was dead code and was removed (`1a920bf1`), which the tests pass without.
- **C25.** The mutant sorted every system's list by the current list's sort. The test did look at another system after
  sorting, but only under a rating sort, and that system's games have no rating, so they fell back to the name order that
  is also the default. The test now looks at another system right after a name-descending sort, which exposes it.
- **C50.** The test asserted the help context's flag, not what the help bar shows. It now requires the entry on North
  to read Collection while editing and Search after.

### 22.9 Measurements

`CollectionsBenchTool`, run with `EMUSEN_BIGPICTURE_BENCH=1`, on the desktop, Debug build, headless with no window, so
the scene is built and not laid out or drawn. The library is synthetic: 3,508 games over the five systems, one favourite
in seventeen, one game in five played, three custom collections of 200 games, and the three automatic collections on.
The output is `~/.cache/emusen/probe/bigpicture/collections/bench.txt`.

| Measure | Result |
|---|---|
| `Show` without the collections | median 1.2 ms of five warm |
| `Show` with them, building the shelves included | median 3.9 ms; building the shelves alone 1.15 ms; ratio 3.29 |
| A list step in snes (1,100 games) | median 0.10 ms |
| A list step in all games (3,508) | median 0.46 ms; ratio 4.5 |
| Gamelist Options' model over all games: 27 quick-selector entries, 20 filter values | 6.3 ms the first time, then a median of 2.4 ms |

### 22.10 Predictions retired, and the broad run

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P68 | the broad run passes with no failure this branch causes | 824 passed, 10 skipped, none failed | held |
| P69 | a warm Show with the collections under 1.5 times one without | 3.29 times (3.9 against 1.2 ms) | **failed.** The collections add nine lists to sort at each Show, the all-games one of 3,508; the ratio is large because the Show without them is small on the synthetic theme. The added 2.7 ms is not the cost that matters on a real theme, where §15.8 measured the first frame at about 50 ms warm; that frame was not measured here |
| P70 | a step in all games within 1.5 times a step in a system | 4.5 times (0.46 against 0.10 ms) | **failed, and the cause is not the sort.** The listed games are kept per list, so a step does not sort again; but each step rebuilds the scene, and the text list's rows are built for every game of the list, so a step's cost grows with the list's length (3,508 against 1,100 games, 3.2 times; the rest is not attributed). That is stage (c)'s design (§14.8, whose lever skips the rebuild while a list is held), and it applies to any long system as much as to a collection |
| P71 | Gamelist Options opened over all games in under 100 ms | its model in 6.3 ms, then 2.4 ms | **held for the model only**; the sheet's layout and first drawing were not measured |

**The broad run**, once, at the end, after a clean rebuild: the Mistress filter without `ShaderSettingsWindowTests`,
`ShaderBrowseBench` and `SceneGpuBench`, under `nice -n 10`, 834 tests, 824 passed and 10 skipped (the picture, bench
and live tools, which need their variables), none failed, in 3 min 15 s. No GPU test was run. One pass is weak
evidence against an intermittent failure, as §15.14 says of its own.

**Pictures.** `CollectionsPictureTool`, with `EMUSEN_BIGPICTURE_PNG=1`, writes nine at 1280×800 to
`~/.cache/emusen/bigpicture/png/collections/`, on Art Book Next with the synthetic media, and one on the synthetic
theme. They were looked at:
- `carousel-collections` and `carousel-all-games`: Art Book Next's own tag and database logos and artwork for the two
  systems, as ES-DE drew them in §22.2 (`runs/r3/sheet.png`).
- `grouped-collections-folders`: Beat and Platform with no folder mark, one game's cover, the description naming three
  games with "[SNES]", and no rating or dates. The first build showed a folder mark, the rating, dates and players, and a
  second "COLLECTIONS" line under the logo; each was changed to what `runs/r2b/g03` shows (§22.6).
- `gamelist-options-sheet`, `filter-sheet`: the sheets over the dimmed view; the filter sheet with Racing chosen, the
  ratings in half-star values and Unknown last (Unknown was first until the picture showed it, and was moved), and
  "Nothing to filter" for publisher, players and completed.
- `filtered-list-racing-by-rating`: two games left, the higher rated first.
- `jump-to-letter-open` and `-after`: the list of characters, star first, and Ember Circuit selected after E.
- `editing-platform-ticks-synthetic`: the ticks before two members, the favourite's star on a third, and COLLECTION on
  the help bar's North.

### 22.11 Not done

- **The Back button.** ES-DE's gamelist options open on Back; here they are in Start's menu until §23's game options and
  these are one menu (§22.5, Q12).
- **Create New Custom Collection from Theme**, and the UI modes (Kiosk, Kid) that restrict these menus.
- **The filters for hidden games, controllers and alternative emulators**, and real values for completed, kidgame and
  broken: Mistress records none of them. §23's metadata editor may add some; the filter screen offers whatever field
  holds more than one value, so a field that gains data needs one line in `GamelistOptions.ValueOf`.
- **The game counter** that reads "filtered / total" while a filter is on: ES-DE's `gamelistinfo` element, which the
  scene has never drawn (§13.3) and Art Book Next does not use.
- **ES-DE's *custom collections sortname*.** *Amended at the merge with §23 (2026-09-26):* the plain `sortname` that
  §23's editor stores now orders the name sort and indexes the quick selector (`GamelistOptions.SortKey`), as USERGUIDE
  says of it, and a game marked *Exclude from game counter* is left out of every automatic and custom collection, as
  USERGUIDE says; `GamelistOptionsTests.A_sort_name_orders_the_name_sort_and_indexes_the_quick_selector_and_an_uncounted_game_joins_no_collection`
  covers both. Without the amendment the merge would have lost §23's sort names, since this section's sort replaced the
  order §23 had given each list.
- **The Collections list's random game at each step, and Y to jump to it.**
- **A long filter screen.** Each value is a switch, so a real library's developers and publishers, possibly hundreds,
  make a long sheet. It scrolls and every switch is reached (§22.7), but no library of that size was tried on it.
- **The handheld.** Nothing ran there, as for every stage since (e).
- **ES-DE's shoulders**, measured to move ten rows (§22.2), are left as §4.9 has them (Q14).

### 22.12 Open questions

- **Q11.** The automatic collections are off until turned on, as ES-DE's are. Should they be on by default here, as
  All Games, Favourites and Recently Played always are in EmuSen's own library?
- **Q12.** ES-DE has one gamelist options menu, on Back, holding the jump, the sort, the filter, the collection entries
  and the metadata editor. Should the merge with §23 make it one menu under Select, or keep the game options on Select
  and these in the pad menu?
- **Q13.** ES-DE enables each custom collection by hand; here every collection of the library is shown until switched
  off. Keep that?
- **Q14.** ES-DE's shoulders move ten rows in a textlist (§22.2); §4.9 pages by the rows shown. Change to ES-DE's?

*Answered 2026-09-26 (§10.1): Q11 on by default; Q12 one menu under Select; Q13 kept as built; Q14 ten
games. Q15, from §23, was answered with them: North is the favourite. §22.13 is what was built.*

### 22.13 The answers, built (2026-09-26)

**One menu under Select (Q12).** §23's `GameOptionsWindow` is the one sheet. Its two hooks, left empty by §23, are
filled from `MainWindow.BigPictureCollections.cs`: `AddGamelistOptions` gives the rows Jump To..., Sort Games By and
Filter Gamelist (`GamelistOptionRows`, which replaced §22's `GamelistOptionsWindow`) and the entry Search...;
`AddCollectionOptions` gives *Add/Remove Games to This Collection* in a custom collection's own list and *Finish Editing
'…' Collection* while one is edited. §23's entries follow: the favourite, *Edit This Game's Metadata*, *Scrape This
Game...*. On a grouped collection's entry, which is not a game, the game's entries are left out.

To carry ES-DE's apply and cancel, `GameOption` gained a `Row` (a control in place of a button) and an `Apply`, and the
window gained `Cancel`: B, the **Apply** button and choosing an entry apply the rows (sort, filter, jump); Select (ES-DE's
Back) and **Cancel** drop them. When no entry carries an `Apply`, as in a menu with no rows, the window keeps §23's single
Close. The pad menu keeps *Game Collection Settings*, the one entry ES-DE's documentation puts in its main menu for these
features; *Gamelist Options* left it.

**North (Q15).** In a themed gamelist North toggles the favourite (`ThemedAction.Favourite`, the sound `favorite`), and
reads **Favorite** on the help bar. While a custom collection is edited it adds or removes the game instead, and reads
**Collection**, as USERGUIDE says the Y button still does then. On a grouped collection's entry it does nothing. The
search is the menu's *Search...*, which opens the same box and on-screen keyboard; East still clears a search first. The
`ThemedAction.Search` that North produced is gone. EmuSen's own library keeps North as its search (§4.29), since Q15
concerned the themed view.

**Automatic collections on (Q11).** `BigPictureCollections.AutoCollections` defaults to all three, so a settings file
written before this work, and a new one, show them. The difference from ES-DE is deliberate and recorded in §4.58.
`ThemedSession` now sets the list empty for the tests that count the carousel's systems, and the collection tests turn
them on themselves.

**The shoulders (Q14).** `ThemedLibrary.ShoulderJump` is ten, and a jump stops at the list's ends, as a page did. The
triggers are unchanged: the first and the last game. The help bar calls the shoulders **Jump**. `PageSize`, the rows a
list shows, has no use left and was removed with its grid test. USERGUIDE also gives the shoulders to the quick system
select where the primary element is a grid or a horizontal carousel, since left and right move those; that was not
asked and is not built, so in a grid the shoulders jump ten items too.

**Found while building it.** A custom collection emptied by North vanished from the carousel with the grouped system,
because a system with no games is not listed. The grouped system is now listed while it has folders, empty ones included,
as THEMES.md's image for "any custom collection that does not contain any games when browsing the grouped custom
collections system" implies.

**Tests.** Changed: `ThemedLibraryPadTests` (the shoulders' two tests now require ten, stopping at the end; Search from
Select's menu; a new North-favourite test with its help label), `ThemedLibraryHostTests` (the help entries; the sounds'
switch test marks the favourite with North), `ThemedLibraryFlowTests` (search from the menu, favourite with North),
`ThemedGameOptionsTests` (the menu's buttons now begin with Search...), and `ThemedCollectionsTests`: automatic collections
on for a fresh settings file and a JSON file with none; the menu's order under Select; the pad menu without Gamelist
Options; North adding to the edited collection without touching the favourite, and the favourite after Finish Editing;
Finish Editing from a collection's own entry, whose menu has no game entries; `PadAudit` over the combined sheet inside a
custom collection.

**Mutants.** `~/.cache/emusen/probe/bigpicture/mutate_answers.py`, on `mutate_collections.py`'s runner (one at a time,
`nice -n 10`, builds at `-m:2`, a backup restored on start, a clean rebuild at the end), against the collections', the
pad's, the game options', the host's and the flow's tests. The log is `run-answers.log`. **15 of 15 caught, all on the
first run:** the menu without its rows (A1); Back applying (A2); B not applying (A3); no Search entry (A4); Gamelist
Options kept in the pad menu (A5); a collection's entry offering a game's entries (A6); the collection entries missing
(A7); North not the favourite (A8); North the favourite while editing (A9); the help bar calling North Search (A10); the
shoulders moving five (A11); the automatic collections off by default (A12); an empty custom collection dropping
Collections (A13); an uncounted game joining the collections (A14); and the sort name ignored (A15, the merge's
amendment, §22.11). That none survived its first run is weaker evidence than a round with survivors strengthened: the
tests were written or changed, in this round, with these rules in view.

**The merges, and the last broad run.** WiseMan was merged twice into this branch: with §21 and §23 (game options), and
then with §24 (controllers). At the second, `PadHeld` had moved to `MainWindow.Controllers.cs`, so the thumbstick clicks
were added there, and the help entries now take both the A/B swap of §24 and this section's context. One controllers
test pressed North to search and was changed to the favourite. After it, a clean rebuild, the narrow run (BigPicture,
game metadata and options, scraping, the library screen, Preferences, pad settings, pad devices, controllers and pad
navigation: 566 passed, 15 skipped, none failed) and the one broad Mistress run (without `ShaderSettingsWindowTests`,
`ShaderBrowseBench` and `SceneGpuBench`: 878 tests, 863 passed, 15 skipped, none failed, 3 min 16 s). LunaP's suite after
its merge: 1,350 of 1,350.

## 23. The game options menu and the metadata editor (2026-09-26)

*Opened 2026-09-26, on the third item of the list of what big picture lacks against ES-DE:* the per-game options
menu, and a metadata editor for a game's name, description, rating, release date, developer, publisher, genre, players,
favourite and ES-DE's other documented fields, with ES-DE's deletion of a game offered only in a guarded form. Items 1
and 2 of the same list (collections, filters, sorting, jumping to a letter, a random game) were built at the same time on
another branch (§22); this section leaves named hooks for them (§23.4). Predictions are numbered P84–P99 and questions
Q15–Q19, the ranges this work was given. The player's account is §4.59 of the settings reference; this is the record.

**The source of ES-DE's behaviour.** ES-DE's user guide (`USERGUIDE.md`, the copy fetched on 2026-09-25 for stage (c),
kept at `~/.cache/emusen/bigpicture/motion/docs/`), its sections "Gamelist options menu", "Metadata editor" and
"General navigation". ES-DE's source was not read. Two defaults the guide does not state were read from the settings file
ES-DE 3.4.1 wrote into the scratch home of stage (b) (`~/.cache/emusen/bigpicture/esde/home/ES-DE/settings/es_settings.xml`):
`ShowHiddenGames` is `true` and `FavoritesAddButton` is `true`. ES-DE was not run for this section.

### 23.1 Expectations and predictions

P84–P88 were written by the same hand as the design, during the build, as §18.1 and §20.1 were, and are weaker evidence
than a stage's predictions written before a line of code. P89–P91 were written before the mutants and the runs they
concern were started.

| # | Expected | Found | Verdict |
|---|---|---|---|
| P84 | Every control of the options menu and of the editor is reached by the pad at 1280×800, the editor's fifteen Reset buttons included once each is shown | measured: the options menu's 4 controls and the editor's 38, all fifteen Resets shown, all reached (`Every_control_of_the_options_and_the_editor_is_reached_by_the_pad`) | held |
| P85 | An edit survives any number of re-scrapes by construction: edits are rows of `games.db`, and a scrape writes only `media.db` | an edit kept through a scrape from the menu and another from the pad menu; G6, which clears the edits when a result arrives, is caught | held |
| P86 | The editor's own scrape fills its fields unsaved: Cancel leaves the stored edit, Save replaces it | Cancel after the editor's scrape kept *mine*; Save took *Scraped description.* and left no edit; G10 and G11 caught | held |
| P87 | Hide from Library, Clear and unhiding change no file under the ROM folder: not its size, its write time or its SHA-256 | the ROM folder's fingerprint (size, write time, SHA-256 of every file) equal before and after Hide, unhide and Clear; G15 and G19 caught | held |
| P88 | A click anywhere on the rating row sets the rating, a pad being the first input but not the only one | **refuted by the first click test**: a click in the gap beside a star reached nothing, because the row drew only its stars. Fixed with a transparent fill (`LunaP.md` §160.4); three points now pass, P5 is the mutant | failed, then fixed |
| P89 | Of the mutants of §23.9, at least nine in ten are caught on their first run, and every survivor is a weak test rather than an equivalent mutant | 49 of 49 caught, every one on its first run, none equivalent (§23.9) | held; weak evidence, §23.9 says why |
| P90 | The broad Mistress run passes with only this section's tests added, none of the earlier ones failing | 823 passed, 9 skipped, **1 failed**: `SceneMotionTests.Moving_only_the_list_while_the_metadata_is_faded_out…`, with Avalonia's "The calling thread cannot access this object". That class passed 12 of 12 alone, twice, and the BigPicture namespace passed 294 of 294 in one run | **failed by one test, not attributed**; see below |
| P91 | LunaP's whole suite passes with §160's two controls and nothing else of its behaviour changed | 1343 of 1343 | held |

**P90's one failure.** The failing test belongs to stage (c) and draws scenes built from `SceneGame`. This work added
three properties to `SceneGame` and changed which games the system view counts, but it touches no thread. The exception
is the class of failure §17.14 recorded once for stage (c)'s `SceneMotionTool`: a UI object used from a thread other than
the one that made it, in a broad run and never alone. Before this work the same class passed in the broad run of §20.3.
Whether this change caused the failure cannot be shown either way. The test-load rule of 2026-09-25, made stricter after
that day's resets, ruled out repeating the broad run until the failure came back, and the unmodified build was not run
broadly for comparison.

*Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78: not this work's. `GridSceneTests`' seven plain facts
built scenes off the headless session's dispatcher, which fails other tests' start-ups (§4.78.2); fixed in `b272675c`.*

### 23.2 What ES-DE documents, and what Mistress builds of it

**The menu.** ES-DE opens its gamelist options menu with the Back button (Select on a pad), from the gamelist only, and
closes it with Back again or B. Its entries, as the guide lists them: *Jump to..*, *Sort games by*, *Filter gamelist*,
*Add/remove games to this collection* and *Finish editing* (custom collections only), *Edit this game's metadata*, and
*Enter folder* (folders with a folder link only), then Apply and Cancel buttons for the jump and the sort.

| ES-DE's entry | In Mistress |
|---|---|
| Jump to.., Sort games by, Filter gamelist | the collections work's (§22); the hook `AddGamelistOptions` (§23.4) |
| Add/remove games to this collection, Finish editing | the collections work's; the hook `AddCollectionOptions` |
| Edit this game's metadata | **built**: *Edit This Game's Metadata* |
| Enter folder (override folder link) | not offered: Mistress's library has no folders and no folder links. *Built in §30.* |
| Apply, Cancel | not offered: every entry built here acts when chosen; they belong with the jump and the sort. A *Close* button stands for Cancel |
| *(not in ES-DE's menu)* | **built**: *Add to Favourites* / *Remove from Favourites*, and *Scrape This Game...* |

The two entries ES-DE's menu does not have are there for a reason each. ES-DE toggles a favourite with Y (its option
*Enable toggle favorites button*, on in the scratch home), but Y, North on the pad, is the search in §4.9's grammar, which
the decisions of §10.1 made and which ES-DE has no counterpart for. Select was the favourite until now; this work
gives Select to the menu, as ES-DE does, and the favourite had to go somewhere one press from the list. *Scrape This
Game...* is the pad menu's entry of §17.14 again, with the same confirm step and the same run. ES-DE starts a single-game
scrape only from inside the editor, which Mistress also does (below); it was requested in the menu as well. Both are
places the player starts a run, so §17.14's rule 1 holds.

**The editor.** ES-DE's guide lists the editor's fields in this order and says what each does. Mistress builds those
whose features it has:

| ES-DE's field | In Mistress | Where it shows |
|---|---|---|
| Name | text | the themed view, the library's list and grid, the search |
| Sortname | text | the themed gamelist's order |
| Custom collections sortname | not built: ES-DE shows it only inside a custom collection, which is §22's | |
| Description, Developer, Publisher, Genre, Players | text (the description over several lines) | the themed view |
| Rating | LunaP's `RatingPicker`, half stars | the themed view's rating |
| Release date | LunaP's `DateStepper`, `yyyy-MM-dd` | the themed view |
| Favorite | switch; Mistress's own favourite (§4.32 of the settings reference) | everywhere a favourite shows |
| Completed, Kidgame, Broken/not working | switches | the theme's badges and metadata texts; Mistress has no kid mode |
| Hidden | switch | a hidden game is left out of every library view (§23.5) |
| Exclude from game counter | switch | left out of the system view's game and favourite counts |
| Exclude from multi-scraper | switch | left out of every run but *Scrape This Game* |
| Hide metadata fields | **not built** | the scene decides which elements to build once per view, not per game (§23.11) |
| Times played, Play time | numbers; Mistress's own counters (§4.32) | the theme's play count and play time |
| Controller | **not built**: Mistress draws no controller badge (§3.8) | |
| Alternative emulator | **not built**: Mistress has one core per console | |
| Folder link | not applicable: no folders. *Built in §30, in a folder's own sheet.* | |

**The editor's buttons.** ES-DE documents five: *Scrape*, *Save*, *Cancel*, *Clear* and *Delete*.
- *Scrape* opens ES-DE's single-game scraper, and Y is its shortcut. Mistress's runs stage (d)'s *Scrape This Game*
  (the confirm step, then the run and its status sheet) and, when the run ends, puts ScreenScraper's answer into every
  field the answer has a value for, unsaved, and marks each such field "From this scrape". The guide says ES-DE colours a
  value the scraper changed red and asks whether to save when the editor is left; these fields are the red values. Y
  (North) is the shortcut here too.
- *Save* and *Cancel* as documented. B with unsaved changes asks *Save* or *Discard*, as the guide says ES-DE asks when
  the editor is left; B with none closes at once.
- *Clear*, in ES-DE, "will remove any media files for the file or folder and also remove its entry from the gamelist.xml
  file, effectively deleting all metadata. The actual game file or folder will however not be deleted." Mistress's removes
  the player's edits, ScreenScraper's answer for the game in `media.db`, and the pictures of that game in Mistress's own
  media store, after a confirm. It keeps the favourite and the play counters, which in ES-DE live in the same gamelist
  entry and go with it, and it touches neither the player's own covers nor an ES-DE media folder, which Mistress only
  reads (§4.52 of the settings reference), nor the OpenEmu failover's cover.
- *Delete*, in ES-DE, "will remove the actual game file, its gamelist.xml entry, its entry in any custom collections and
  its media files". Mistress never deletes or moves a file in the player's ROM folder (a project rule), so the button is
  **Hide from Library** in Delete's place, with a confirm that says so and why.

**Where Mistress's editor is its own.** Each field has a *Reset*, shown only while the field differs from what it would
show with no edit, which returns it to ScreenScraper's value or the default. ES-DE has no per-field reset, only *Clear*
for the whole entry; it was requested that a field can be returned on its own. Each field says where its value comes from
("From the file name", "From ScreenScraper", "Your edit", "Your edit, shown in place of ScreenScraper's", "From this
scrape") in words where ES-DE uses gray, blue and red.

### 23.3 Where an edit is kept, and why it wins

**In `games.db`**, its fifth migration: `game_edit (path, field, value, edited)`, one row per edited field. A field with
no row shows what it would anyway; a row, even an empty one, is the player's value. The table sits beside Mistress's
other records of the player's own (favourite, play counts, collections, §4.32), because an edit is the player's statement
about a game and not a cache of a server's answer. Three alternatives were considered:
- *Columns on `media.db`'s `scrape_game`.* A scrape writes that row with `INSERT OR REPLACE` (§17.5), so keeping an edit
  would depend on every writer carrying it across, and the row is keyed by the file's MD5, so two copies of one game
  would share one edit. The rule that a re-scrape never overwrites an edit would then be a property of code paths rather
  than of where the data lives.
- *Columns on `games.db`'s `game`.* Fifteen nullable columns, each later ES-DE field a migration of its own; one row per
  field needs none.
- *JSON beside the library.* Ruled out from the start: Mistress's data is in SQLite.

**The order**, in `GameMetadata.Resolve`: the edit, else ScreenScraper's value, else the default (the file's name for the
name, "no" for a flag, nothing otherwise). ScreenScraper's name is kept and not used, as §17.6 decided, so the name's
default is the file's even where a scrape found another.

**What a re-scrape does.** Nothing to an edit: the scraper writes `media.db` only, and no code that writes `game_edit`
runs during a run. The player asks for an edit to go by *Reset* on that field, by *Clear* on the game, or by saving the
editor after its own *Scrape* has put ScreenScraper's value in the field.

**A value equal to its baseline is no edit.** Saving a field whose value equals what it would show anyway removes the edit
rather than storing a copy, so a later scrape that improves the text reaches the field. This is Mistress's choice; ES-DE's
guide says only that a name equal to the file's is treated as unset.

**A renamed file keeps its edits**: `GameRecords.Move`, which §4.37's rename detection calls, moves `game_edit` rows with
the favourite and the collections.

**A merge note.** §22's work may add a `games.db` migration too. The array is append-only (§4.32), so whichever branch is
merged second renumbers its entry after the other's; neither has been run against the player's own `games.db`.

### 23.4 Buttons, and the hooks left for §22

| Button | Before | Now |
|---|---|---|
| Select, in the themed gamelist | mark or unmark a favourite | open the game options menu, as ES-DE's Back button does |
| Select, over the options menu | | close it, as ES-DE's Back does on its menu; B closes it too |
| Select, in the themed system view | nothing | nothing: ES-DE's menu "can't be accessed from the system view" |
| North (Y), in the editor | | *Scrape*, ES-DE's documented shortcut |
| B, in the editor | | leave; asks *Save* or *Discard* when something changed |
| Left, Right, on a rating or a date | move the focus | change the value (`ISidewaysAdjustable`, §23.6) |
| Select, in EmuSen's own library (built in) | favourite | unchanged: that library is Mistress's own look, not ES-DE's |

West is not used: §21's Q32 proposes it for ES-DE's media viewer. The help bar's `back` entry now reads *Options*.

**The hooks.** `MainWindow.GameOptions.cs` declares two partial methods: `AddGamelistOptions(SceneGame, List<GameOption>)`
for ES-DE's *Jump to..*, *Sort games by* and *Filter gamelist*, and `AddCollectionOptions(SceneGame, List<GameOption>)`
for adding the game to, or removing it from, the custom collection being edited. They are called in ES-DE's order, before
*Edit This Game's Metadata*. A partial method with no body compiles to nothing, so this branch builds without §22, and §22
fills them in a file of its own without editing this one. A `GameOption` is a label, an action, and whether choosing it
puts the menu away first. The editor has no collection field; ES-DE's *Custom collections sortname* is left to §22.

*Amended at §22.13 (2026-09-26), after the answers to Q12 and Q15.* §22 filled both hooks: the menu now opens with
Jump To..., Sort Games By, Filter Gamelist and Search..., then the collection entries, then the entries above. A
`GameOption` may now be a row (`Row`) and carry an `Apply`; Select over the menu cancels the rows' choices, and B, Apply
or choosing an entry applies them, where the table above has Select and B both closing. North in the themed gamelist is
the favourite (ES-DE's Y) outside a collection's editing; the menu's favourite entry was kept. On a grouped collection's
entry the menu has the list's rows and the collection entries only.

### 23.5 Hiding, and the rule that no ROM is touched

A hidden game is left out of the themed gamelists, the library's list and grid, the sidebar's counts and the search.
Preferences ▸ Appearance ▸ **Hidden Games** lists them again, and the editor's *Hidden* switch, or *Reset* on it, unhides
one. ES-DE's default is the reverse (hidden games shown, dimmed; `ShowHiddenGames` true): Mistress hides by default
because *Hide from Library* stands where ES-DE's *Delete* stands, and a player who chose it expects the game to go. That
difference is Q16. A hidden game listed again is not dimmed; ES-DE dims it.

Nothing but `games.db` changes when a game is hidden or unhidden. *Clear* deletes files, and only these: the paths
`media.db` records for the game, and the files named after the game's stem in the store's type folders, each deleted only
when its full path lies inside the store's own folder (`home/Media`). A `media.db` row naming a path outside it, which no
build writes but a damaged or hand-edited file could hold, deletes nothing. Two files of one system with the same stem in
different folders share their pictures in the store (§17.5's layout), so *Clear* on one takes the other's pictures too.

### 23.6 Drawn with LunaP

Every visible part is a LunaP control. The menu and the editor are `ToolWindow`s shown by `SheetLayer`, so in a big-screen
session they are sheets inside the one window and no window is opened (the tests assert none is owned); the rows are
`FieldRow`s, the flags `LunaSwitch`es, the buttons a `ButtonBar`, and the text boxes take the on-screen keyboard of
§4.45.6. LunaP had nothing a pad could set a rating or a date with, so two controls were added there (`docs/LunaP.md` §160,
branch `bigpicture-game-options`): `RatingPicker`, a star row stepped in half stars by Left and Right, and `DateStepper`, a
date whose year, month and day are changed one at a time by Left and Right, with Enter moving between them. Both carry a
new marker, `ISidewaysAdjustable`, which Mistress's pad router and WiseMan's `PadAudit` now honour as they honour a
`Slider`: Left and Right go to the control rather than moving the focus.

### 23.7 Closing what is opened (§15.14's lesson)

The menu and the editor are sheets, and a sheet is not an owned window, so nothing closes one when Mistress closes
(§20.4's W22). `CloseGameSheets`, called from `CloseThemedLibrary` in the window's `Closing`, closes both. The editor is
the one thing here that holds a subscription beyond its own controls: it listens to the window's `ScrapeChanged` while it
waits for its own scrape, and lets go of it when it closes. Neither has a timer.
`Closing_mistress_closes_the_editor_and_its_subscription_to_scraping` opens the editor, closes Mistress, and requires the
sheet gone, the window's reference to it cleared and the editor unsubscribed; G29 and G30 (§23.9) remove the two halves.

### 23.8 Tests

Headless through WiseMan, on `ThemedSession`, `PadDriver`, the synthetic theme and ROMs and, for the scraping, stage (d)'s
fake ScreenScraper; never the network. Every step a player takes is a pad press: the menu is opened with Select, entries
and fields are reached with `PadAudit.Reach`, text is typed on the on-screen keyboard key by key, and every confirm is
answered on its own sheet.

- `ThemedGameOptionsTests` (10): Select opens the menu as a sheet and not in the system view, the view hears nothing
  under it, Select and B put it away; every control of the menu and of the editor reached by the pad, all fifteen Resets
  shown; every kind of field set by the pad and shown in the view and in the library's list; the sort name; Cancel, and B
  with Discard, with Save, and with nothing changed; Reset on a text field and on a flag; *Exclude from multi-scraper*
  against the plan of a whole-library run, a console run and *Scrape This Game*; *Exclude from game counter* in the
  system view's count; Hide from Library taking the game out of every view with the ROM folder's fingerprint (each file's
  size, write time and SHA-256) unchanged, then listed again and unhidden, the fingerprint still unchanged; and Mistress
  closing the editor.
- `ThemedMetadataScrapeTests` (3): an edit surviving a scrape from the menu and another from the pad menu, then Reset
  giving ScreenScraper's text back; the editor's own scrape (Y) filling its fields, Cancel keeping the edit and Save taking
  the answer; Clear removing the edits, the answer and the store's pictures and keeping the favourite, with the ROM
  folder's fingerprint unchanged.
- `GameMetadataTests` (7): the order edit, scraped, default; the draft's changes, including an emptied box over nothing
  and a value equal to its baseline; the editor's scrape against the draft; edits in `games.db` following a renamed file
  and cleared without the favourite and counters; **a schema-4 `games.db` migrated in place** to schema 5, its favourite
  and play rows kept and an edit written with its time (the requirement, set 2026-09-26, that everything the
  program writes and reads back is in SQLite, versioned and migrated); a scrape recorded twice leaving an edit; and
  `MediaStore.Forget` deleting the store's pictures, including a second copy's, and nothing outside the store even when
  `media.db` names it.
- Changed: `ThemedLibraryPadTests`' Select test now goes through the menu's entry; `ThemedLibraryHostTests`' help test
  reads *Options* and its sounds test marks the favourite through the menu; `ThemedLibraryFlowTests`' walk does the same.
- LunaP: `PickerTests` (12), `docs/LunaP.md` §160.4.

Nothing here is kept in JSON. The edits, the hidden flag among them, and each edit's time are rows of `games.db`;
`appsettings.json` gains only the player's setting *Hidden Games* (`ShowHiddenGames`), which is configuration, as
`EmuSen_Stack.md` §4 divides them.

### 23.9 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/mutate_game_options.py`, adapted from §20.4's, with its list
`mutants-game-options.json` written by `game-options/mutants_game_options_make.py`, which checks that each edit's text
occurs exactly once. The log is `run-game-options.log`, and every verdict is appended to `mutants-game-options.txt`.
Each mutant was built with `-m:2` and run alone under `nice -n 10`. The Mistress mutants (G) ran against
`ThemedGameOptionsTests`, `ThemedMetadataScrapeTests`, `GameMetadataTests`, `ThemedLibraryPadTests`,
`ThemedLibraryFlowTests`, `GameRecordsTests` and the help-entry test. LunaP's (P) ran against `PickerTests`. After each
mutant the file was restored from its copy, stamped with the present time and compared byte for byte. Both trees were
rebuilt at the end of the round.

**An interrupted round.** The machine reset at 17:58, a hardware fault under load (see the project's notes on the
desktop's resets). The reset came during G19, and it left G19's mutant in `MediaStore.cs` with its backup beside it. The
file was restored, byte-identical to the commit, and deleted the backup. The tree was then rebuilt
before any further test ran. The runner now restores any backup it finds when it starts, and the round was resumed from
G19 (`--from`). G36 was added before the resumption, for the migration test written in the meantime (§23.8).

| Rule | Mutants | Result |
|---|---|---|
| Where an edit lives and how it wins | G1 ScreenScraper's text over the edit; G2 a field reset keeps its old edit; G3 every field saved as an edit; G4 Reset does nothing; G5 an emptied box over nothing kept as an edit; G6 a scrape's result clears the edits; G7 a renamed file loses its edits; G36 the fifth migration empty | all caught; G1 by 4 tests, G3 by 7, G36 by 32, the migration test among them |
| The editor | G8 Cancel saves; G9 B saves without asking; G10 its own scrape saves at once; G11 its own scrape fills nothing; G12 a refilled box read as a change; G13 Y does not scrape; G14 Hide without its confirm | caught |
| Hiding, and no file touched | G15 Hide touches the ROM's write time; G16 a hidden game stays in the library; G17 it stays in the themed gamelist; G18 Hidden Games ignored; G19 Clear deletes a path outside the store; G20 Clear leaves the store's pictures; G21 Clear leaves the edits | caught |
| The menu and the buttons | G22 Select marks a favourite again; G23 Select opens the menu in the system view; G24 Select over the menu does not close it; G25 the favourite entry always reads Add; G26 no Scrape This Game in the menu; G27 the help bar reads Favorite; G28 the router moves the focus off a rating or a date | caught; G22 by 14 |
| Closing what is opened | G29 Mistress closing leaves the editor open; G30 the editor stays subscribed | caught |
| Where the fields show | G31 the library shows the file's name; G32 the sort name ignored; G33 a game out of the counter counted; G34 Exclude from multi-scraper ignored; G35 the view shows ScreenScraper's rating over the player's | caught |
| LunaP §160 | P1 a rating between steps stepped unsnapped; P2 the rating wraps; P3 no Chose for a key; P4 a click takes the step to the left; P5 the row hit only where a star is drawn; P6 the filled stars cut at nothing; P7 the day not kept in its month; P8 a month step carries into the year; P9 Enter does not move on; P10 the first year clamps instead of leaving no date; P11 no date ignores StartDate; P12 the accent always under the year; P13 a click always chooses the year | caught |

**49 of 49 were caught, each on its first run and each by a test written for its rule.** That is weaker evidence than it
sounds, as §104.6 of `LunaP.md` argued for a round with the same result. Every mutant was written after its test, by the
same hand, and against a rule that hand had already chosen to test, so a rule without a test could not get a mutant.
Two mutants were never written, for that reason:
- one that drops the transparent fill of the rating row, before the click test existed. §160.4 of `LunaP.md` records the
  defect this missing fill caused; P5 is that mutant, written afterwards;
- one that saves an edit to `media.db`. The storage decision (§23.3) rules this out by construction rather than by a
  test.

The tests that failed for G22 (14) and G36 (32) show how far those two rules reach: nearly every flow starts at Select,
and every edit needs the table.

### 23.10 Pictures

At 1280×800, written by `GameOptionsPictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to
`~/.cache/emusen/bigpicture/png/game-options/`, for the synthetic theme (`synthetic-*`) and for Art Book Next with the
synthetic media folder (`artbooknext-*`): the gamelist before; the options menu; the editor as opened; the on-screen
keyboard typing into the name; the editor with a name, a description, a developer, a genre, four and a half stars and a
date set, each row saying "Your edit" beside its Reset; the Hide from Library question; and the gamelist after Save. They
were looked at:
- Art Book Next's gamelist after Save shows the new name in the list, the typed description, four and a half stars and
  1993-01-01 in the metadata panel, and its help bar reads *Options* beside the Select glyph. The same panel before the
  edit showed empty stars and "Unknown".
- The menu lists its three entries and Close over the dimmed view, the first focused. Like every sheet (`LunaP.md` §90) it
  takes the window's full height, which leaves most of a three-entry menu's sheet empty.
- The keyboard covers the lower half of the editor and previews the name with its caret, as §4.45.6 draws it.
- **A flaw seen and corrected:** the first pictures showed the release date as bare grey text, "No date", under its label,
  which read as a caption rather than a field. `DateStepper` now draws itself on the input surface inside a border
  (`LunaP.md` §160.3).
- **Seen and left:** each confirm focuses its accepting button first (LunaP's `DialogWindow`), so a second A on *Clear...*
  clears. *Hide* is reversible; *Clear* is not, since it deletes the scraped pictures, which a new scrape fetches again.

### 23.11 Not done

- **ES-DE was not run.** Everything here is from its user guide and the two settings its scratch home recorded. Where the
  guide is silent, Mistress chose: whether the editor closes after *Clear*; which button its confirms focus first; whether
  B in the editor with nothing changed asks anything.
- ES-DE's fields *Hide metadata fields*, *Controller* and *Alternative emulator* are not built (§23.2). *Hide metadata
  fields* would need the scene to build or skip the theme's `metadataElement` elements per game, where §13's builder
  decides once per view.
- A hidden game listed again is not dimmed.
- The on-screen keyboard has no line break, so a description is one paragraph.
- The editor's scrape shows stage (d)'s status sheet over the editor, as every run does; B returns to the editor.
- ScreenScraper's own name is not offered by the editor's scrape (§17.6 keeps it unused). *Built in §27 (Q18): it is
  offered, and taken only by the player.*
- *Clear* on a game takes the store's pictures of any other file of the same system with the same stem (§23.5).
- The two hooks of §23.4 are empty on this branch; the collections entries are §22's.
- EmuSen's own built-in library keeps Select as the favourite and has no editor. *Amended by §27 (Q19): the sidebar
  library, on the desktop and in its big screen, reaches the menu and the editor; Select there is still the favourite.*
- Nothing ran on the handheld.

### 23.12 Open questions

- **Q15, the favourite's button.** ES-DE toggles a favourite with Y; Mistress's Y is the search (§4.9). The favourite is
  now the first entry of the options menu, two presses from the list. Keep it there, or give Y to the favourite and move
  the search? *Answered 2026-09-26 (§10.1): Y toggles the favourite and the search is an entry of the
  menu; built in §22.13. The menu's favourite entry stays too.*
- **Q16, hidden games.** ES-DE lists hidden games, dimmed, by default; Mistress leaves them out unless Preferences ▸ Hidden
  Games is on, because Hide from Library replaces ES-DE's Delete. Keep that default?
- **Q17, what Clear removes.** It removes ScreenScraper's pictures in Mistress's store, but not the cover OpenEmu's
  failover fetched, nor the favourite and play counters, which ES-DE's Clear takes with the gamelist entry. Should it take
  more?
- **Q18, ScreenScraper's name.** ES-DE's editor scrape replaces the name with the scraper's; Mistress keeps the file's name
  unless the player types one (§17.6). Offer ScreenScraper's name in the editor's scrape? *Answered
  2026-09-26 (§10.1): offered, never forced; built in §27.*
- **Q19, EmuSen's own library.** Should its Select open the same menu and editor, or stay the favourite? *Answered by the
  user on 2026-09-26 (§10.1): the menu and the editor are available in both; built in §27, which leaves Select the
  favourite in the sidebar library and asks Q46 about it.*

---

## 24. Pass 2: controllers (2026-09-26)

*Opened and closed on 2026-09-26, on branch `bigpicture-controllers`, while §22 and §23 were being built on theirs; §23
was merged into it before the pass closed.* Pass 2 of §21.3 as accepted (Q22 (a), §10.1): every connected pad
steers the interface, pads come and go while Mistress runs, a notice says so, and Preferences gains ES-DE's controller
type, an A/B swap and its first-controller switch. The settings reference's §4.61 is the player's account; this
section is the record.

### 24.1 Predictions, and the scope as built

P103 (§21.6) is the pass's prediction: "Every pad rule of §15.3 holds from either of two pads, and the help bar follows
the pad last pressed; ES-DE's device popup fades in and out over 0.4–0.6 s each and holds 2–5 s". Two predictions were
added for Pass 1 to retire, because the headless harness cannot see what Steam does to a pad:

- **P121.** In Game Mode, with the Legion Go S's built-in controls alone, SDL reports exactly one gamepad, and Mistress
  announces nothing at start (§24.5); a Bluetooth pad connected during a session is announced within one second under the
  name Steam gives its virtual pad, not the hardware's own.
- **P122.** In Desktop Mode, where Steam Input is not in the path unless Steam is running, the same Bluetooth pad is
  announced under its own name and its family is detected from SDL's type, as §15.5 found for a pad seen directly.

**Scope.** As §21.3 wrote it, with two readings made explicit. A second pad does **not** become player 2 in a game: that
is Q22 (b), recorded as separate input work in `EmuSen_Input.md` §6. The game reads the first pad opened, as it read the
only pad before; `GamepadBindingMap` is untouched. *(Since 2026-10-04 each pad plays as its own player, `EmuSen_Input.md`
§8; this paragraph describes pass 2 as it was built.)* And the swap trades A and B alone. ES-DE's switch is "Swap the A/B
and X/Y buttons" (UG "Input device settings"), and the first build followed it; but on 2026-09-26 it was decided that
in the themed gamelist North toggles the favourite and search moves into the options menu (§22's branch builds it), and
the swap was then restricted, by instruction, to accept and back, so that it composes with whatever North and West come
to mean. §24.5 records the departure.

### 24.2 What was built

**In Endymion.** `GamepadManager` opens every pad SDL lists, in the order it finds them, and keeps them in `Pads`; the first
is `Primary`, player 1's. SDL is reached through a seam, `IPadDevices`, with two implementations: `SdlPadDevices`, the
calls the manager made before, and `SimulatedPads`, a set of `SimulatedPad`s a test plugs in and pulls out, which counts
the handles opened and closed. `ConnectedPad` is one open pad; once closed it reads as nothing held and keeps its name and
type for the notice. `Poll` lets go of every pad SDL reports detached, then opens any new one when SDL has queued a
`GamepadAdded` or `GamepadRemoved` event (consumed with `SDL_HasEvents` and `SDL_FlushEvents` over those two types alone,
so the queue's other events are left as they were) or when the one-a-second rescan of §4.4 falls due; each change after
`Start` raises `PadChanged`. The rescan fix of 2026-09-26 is kept: `_lastRescan` still starts at `-RescanInterval`, and
since `RescanDue` is now asked at every poll rather than only with no pad, the fix is exercised on every tick. The
single-pad members (`IsPressed`, `Axis`, `ControllerName`, `ControllerType`, `ButtonLabel`, `IsRawPressed`, `RawAxis`) read
`Primary`, so a game and Hotaru see what they saw. `GetAnyPressedButton`, the rebind capture's, reads every pad the
interface reads, since a binding names an SDL button and not a device. `Simulated` is kept for the tests that set it and
now installs a one-pad `SimulatedPads`.

**In Mistress.** `Views/MainWindow.Controllers.cs` holds the rest:
- `PadHeld` asks every pad the interface reads (`FrontendPadCount`: all, or the first with the switch) and is true when
  any holds the button. The press rules above it, `PadNavigator`'s and the themed view's, are unchanged.
- `PadHeldOn` is §4.29's mapping per pad, with the swap: Accept on East and Back on South, and nothing else moved.
- `TrackControllers`, at every poll, hands the first-controller switch to the manager, makes the pad that went from
  nothing held to something held the help bar's pad, and redraws the text hints when the swap changes.
- `HelpFamily` is the Controller Type setting, or with Automatic the family of the help bar's pad (§15.5's rules, from
  that pad's type and name), falling back to the first pad.
- `OnPadChanged` shows the notice and, when the first pad arrives or the last leaves, redraws the Mistress library's
  hint, which names the pad's buttons or the keyboard's.

`Input/PadHints.cs` swaps the letters in the text hints (the sheets' footer, the pad menu's, the library's, the on-screen
keyboard's and the rewind reel's), which name face buttons by an Xbox pad's letters. `HelpPrompts.For` takes the swap and
gives the two functions the button each moved to and the theme's icon for that button, so a theme's `button_a_XBOX`
goes with the function now on A. `SceneData.SwapFaceButtons`, `SceneView.SetPadLayout` and `ThemedLibrary.SetPadLayout` carry the
family and the swap together; as before, a change redraws only the help bar (a swap re-enters its entries).
`ControllerPreferencesPane` is Preferences' new Controllers tab. `MainWindow.axaml` gains `PadNotice`, a `NoticeLayer` over
everything at the top centre.

**In LunaP** (branch `bigpicture-controllers`, `docs/LunaP.md` §170): `NoticeLayer.FadeTime`, so a notice fades over a
fixed time rather than 15% of its duration; null keeps §88.5's curve.

**Nothing learned is persisted, so nothing is in SQLite.** The four settings are the player's choices and live in
`appsettings.json` with the rest, as `EmuSen_Stack.md` §4 places user settings. No pad is remembered between sessions:
no GUID, name or family is written anywhere. A later pass that remembers pads (a per-pad family override, say) would keep
them in Mistress's database, not a new JSON file.

### 24.3 The rules, and their tests

All headless, on `PadDriver`, which now plugs and pulls simulated pads (`Plug`, `Unplug`, `Replug`) through the window's
own `GamepadManager`, so `Poll` runs as it would on a device.

| Rule | Test |
|---|---|
| Every pad present at start is opened, in order, and none is announced | `GamepadManagerPadsTests.Every_pad_present_at_start_is_opened_in_order_and_none_is_announced` |
| A pad plugged in is opened and announced; one pulled out is closed and announced with its name and type | `A_pad_plugged_in_is_opened_and_announced…` |
| The game reads the first pad alone, and the next once the first goes | `The_game_reads_the_first_pad_alone…`; `ControllersTests.Unplugging_one_pad_leaves_the_other_steering_and_playing` |
| The rebind capture hears any pad the interface reads | `The_rebind_capture_hears_any_pad_the_interface_reads` |
| Two pads both steer the library, the menu and a sheet | `ControllersTests.Two_pads_both_steer_the_library_the_menu_and_a_sheet` |
| Every rule of §15.3 holds from a second pad, Select's game options (§23) included | `ThemedControllersTests.Every_rule_of_the_pad_table_holds_from_a_second_pad`, `Two_pads_both_steer_the_themed_view` |
| A pad pulled out and plugged back is picked up without an error, over 120 polls with no pad | `A_pad_pulled_out_and_plugged_back_is_picked_up_without_an_error`; `GamepadManagerPadsTests.A_pad_pulled_out_and_plugged_back_is_opened_again`; `GamepadRescanTests` |
| The notice appears on connect and on disconnect, in a desktop and a big-screen session, and is gone after its duration | `A_notice_says_when_a_pad_connects_and_disconnects_and_then_goes_away` (2 cases; 4.5 s of real dispatcher time) |
| With notifications off, none | `With_notifications_off_no_notice_is_shown` |
| With the first controller only, the second pad steers nothing | `With_the_first_controller_only_the_second_pad_steers_nothing` |
| The help bar follows the pad last pressed, and the first pad's family when that pad goes | `ThemedControllersTests.The_help_bar_follows_the_pad_last_pressed` |
| The controller type changes the help bar's glyphs and no pixel outside it; the buttons do what they did | `The_controller_type_changes_the_help_bar_s_glyphs_and_nothing_else` |
| The swap trades Accept and Back, and nothing else, in the library, the menu, a sheet and the keyboard, and the hints say so | `ControllersTests.The_swap_trades_accept_and_back_everywhere_in_the_interface` (X opens nothing, Y still searches) |
| The swap trades the themed view's buttons, and the help bar names the new ones, with a theme's own icons | `ThemedControllersTests.The_swap_trades_the_view_s_buttons_and_the_help_bar_follows`, `The_swap_gives_each_function_the_theme_s_icon_for_its_new_button` |
| The swap does not reach the game | `The_swap_does_not_reach_the_game` (South is still the SNES B) |
| The settings persist | `The_controller_settings_persist` (set on the Preferences sheet, read back with `AppSettings.Load`; the defaults checked) |
| Every handle is released when its pad goes and when the window closes | `Every_pad_s_handle_is_released_when_the_pad_goes_and_when_the_window_closes`; `GamepadManagerPadsTests.Every_handle_is_closed…` |

Before the merge of §23, the existing pad, themed and Preferences tests ran beside these under one filter (276, all
passing), among them §15.9's audit, which now walks the Controllers tab and reached every control on it. The merge
changed one of this pass's tests: Select in the gamelist now opens §23's game options rather than marking a favourite, so
the second-pad walk opens and closes them instead. One broad run closed the pass, after the merge and the A/B narrowing:
every WiseMan test under `Mistress` and `Input` except `ShaderSettingsWindowTests`, `ShaderBrowseBench` and
`SceneGpuBench`, 902 passed and 12 skipped of 914, no failure; and LunaP's `NoticeLayerTests`, documented defaults, API
baseline and documentation tests, 33 of 33.

### 24.4 ES-DE's popup, measured

§14.7 left the device popup unmeasured, and §21.3 planned to record it with §14.7's rig. The rig had in fact recorded it
once, on 2026-09-25: run `pop` under `~/.cache/emusen/bigpicture/motion/runs/` destroyed its uinput pad while ES-DE 3.4.1
was recording, and `popup.py` wrote the ink of the region 440–860 × 5–58 px of the 1280×800 window, frame by frame
(6.86 ms apart), to `popup_ink.csv`. The recording itself was deleted with the stage's scratch; the series was not, and
this pass read it rather than run ES-DE again (the load rule of 2026-09-25). It is one disconnection popup: n = 1. No
connection popup was recorded: the pad was created before ES-DE started, and the recording began about four seconds
after ES-DE's log shows it adding the pad.

| | ES-DE 3.4.1 (n = 1) | Mistress |
|---|---|---|
| First ink after the device was destroyed | 533 ms (ES-DE's detection included) | the next poll after SDL's event, ≤ 16 ms; at worst the 1 s rescan |
| Place | top centre, within 5–58 px of an 800 px window | top centre, 8 px from the top |
| Fade in | 497 ms to full; a power curve t^1.35 fits (rms 0.009; linear 0.026) | 500 ms, linear |
| Held | 2,988 ms at full | 3,000 ms |
| Fade out | about 500 ms, (1 − t)^1.40 (rms 0.009; linear 0.025) | 500 ms, linear |
| First ink to last | 3,988 ms | 4,000 ms |

Ink is linear in the popup's opacity wherever the popup is brighter than what it covers, so the curves above are its
opacity up to scale. The durations are ES-DE's to within a frame; the shape is not, by at most about 0.1 of full opacity
at mid-fade. A linear fade was kept because it is §88.5's and one recording does not establish a curve.

**P103's second half holds:** each fade about 0.5 s, within 0.4–0.6 s, and a hold of about 3.0 s, within 2–5 s.

### 24.5 Where the rules are Mistress's, not measured ES-DE behaviour

- **Pads present at start are not announced.** ES-DE's log shows it adding the pad at start, but the recording began
  after start, so whether ES-DE announces a pad present at launch was not observed. Mistress opens the pads after its
  first frame (§4.42 of the settings reference), and a notice for the pad already in hand was judged noise.
- **The words.** "Controller connected: *name*" and "Controller disconnected: *name*". ES-DE's wording was in the deleted
  recording and was not read from anywhere else.
- **Held is the union of every pad.** A button counts as held if any pad the interface reads holds it, so one pad holding
  Down and another tapping Down gives no new press. ES-DE's handling of two pads pressing at once was not measured.
- **The pad last pressed** is the one whose buttons went from none held to some held most recently. A held stick counts
  as held.
- **"The first controller"** is the first pad opened that is still attached. ES-DE's guide says "the first controller
  detected during startup" and warns that reconnecting may change it; Mistress's rule changes it in the same case.
- **Over a game,** any pad's guide button or Back and Start together open the pad menu, and the notice is drawn over the
  picture. ES-DE is not on screen while a game runs, so it has no counterpart. The game itself hears the first pad only.
- **The swap is A and B only.** ES-DE's setting swaps X and Y as well. The first build did too (Search on West), and
  was narrowed the same day, before merging, to accept and back alone, after the decision on North (§24.1). The
  departure costs a player with a Nintendo-printed pad the X/Y half of ES-DE's remedy; what it buys is that the swap
  cannot disagree with whatever the collections branch assigns to North and West.
  *Retired 2026-09-27 (Q160, §40.13): the swap trades X and Y as well. The concern it answered did not arise, because
  North and West kept one function each and the swap moves the function with its button, whatever the function is.*
- **The swap is the interface's only.** The keyboard is unaffected, as ES-DE's is; so is every game, whose bindings are
  its own (`GamepadBindingMap`). The text hints' letters swap with it; ES-DE says its help system is "updated
  accordingly".
- **The controller type offers LunaP's four families**, not ES-DE's seven icon sets (Xbox, Xbox 360, PlayStation 1/2/3, 4
  and 5, Switch Pro, SNES): §10.1's Q9 settled Mistress's own four drawings. It changes only the themed help bar's icons,
  as ES-DE's changes only its help icons; the text hints keep an Xbox pad's letters.

### 24.6 Closing what was opened

§15.14's lesson, applied before the fact. Every `ConnectedPad` is closed when SDL reports it detached, and every pad and
the gamepad subsystem when the window closes (the manager's `Dispose`, reached from `Closing`). `SimulatedPads` counts the
handles, and tests fail without each cleanup: mutants C3, C4 and C29 below remove the close on a pad's going, the
manager's disposal of its pads, and the window's disposal of the manager, one at a time.

### 24.7 Pictures

`ControllersPictureTool`, run with `EMUSEN_BIGPICTURE_PNG=1`, writes 1280×800 pictures into
`~/.cache/emusen/bigpicture/png/controllers/`. They were looked at:
- `notice-connected-desktop.png`: the notice over the desktop library, at the top centre over the toolbar, in the desktop's
  15 px type. It covers the Library and Save States buttons for its four seconds; it is not hit-testable, so a click
  reaches them. A first version drew it at the big screen's 20 px on the desktop too, where it covered the toolbar from
  the view buttons to the slider; the desktop's size was reduced after the picture was seen.
- `notice-disconnected-bigpicture.png`: over the synthetic theme's system view, 20 px type at 800 px.
- `synthetic-help-*.png`, `artbooknext-help-*.png` and the two `-help-strip.png`: the gamelist's help bar under Automatic
  (a DualSense in hand, so PlayStation), Xbox, PlayStation, Nintendo and Generic, then Xbox with the swap, where Launch is
  on B and Back on A, and Search still on Y. On Art Book Next the bar's three entries (Options, since §23; Menu; Launch) change icons and
  nothing else moves.
- `preferences-controllers-sheet.png`: the Controllers tab on the Preferences sheet. The tab strip now wraps to a second
  row at this width (Controllers and System Files below the other four).
- **A flaw seen and left:** in the generic set, Launch, Back and Search are the same four dots with a different one
  filled, which at the help bar's size are hard to tell apart; §15.10 recorded the same of the generic d-pad.

### 24.8 Mutants

`~/.cache/emusen/probe/bigpicture/mutate_controllers.py`, one mutant at a time under `nice -n 10` with two build nodes,
each built and run against the controller tests, the manager's, the rescan's and `PadInputPathTests`, the source restored
after each and the tree rebuilt at the end; C30 ran against LunaP's `NoticeLayerTests` in its own checkout. Log:
`mutants-controllers.txt` and `run-controllers.log`.

**A round the machine ended.** The first round was cut off at 17:58 by a hardware reset of the desktop (an AMD fault,
before the BIOS update of that evening), with C16's mutant in `HelpPrompts.cs`. It was found and restored from the commit
before any build was trusted, `bin/` was rebuilt clean, and the runner now writes the file and its original text to a
journal before each mutant and restores a leftover journal when it starts (checked with a planted journal). The eleven
results the first round logged (C1–C11, `mutants-controllers-part1.txt`) agree with the second round's. The table is the
second round's, on the tree after §23's merge, except for the swap's mutants (C11–C17 and C28), which were run again after
the swap was narrowed to A and B, with C13 turned round to put back the X/Y half; and C30, whose first form did not
compile (it left the pattern's variable unassigned) and was rewritten so that it does. `mutants-controllers-final.txt`
holds the table's lines.

| # | Mutant | Result |
|---|---|---|
| C1 | the manager opens only the first pad | caught by 10 |
| C2 | a pad pulled out is never let go | caught by 9 |
| C3 | a pad pulled out is dropped without closing its handle | caught by 4 |
| C4 | the manager's disposal leaves the pads open | caught by 2 |
| C5 | the old rescan: only with no pad, and not on SDL's events | caught by 10 |
| C6 | the rescan clock starts at MinValue again | caught by 23 |
| C7 | the pads present at start are announced | caught by 3 |
| C8 | the interface reads the first pad alone | caught by 3 |
| C9 | the first-controller setting never reaches the manager | caught |
| C10 | the game reads every pad, not player 1's | caught |
| C11 | the swap leaves Accept on South | caught by 3 |
| C12 | the swap leaves Back on East | caught by 2 |
| C13 | the swap moves Search to West as well (ES-DE's X/Y swap, not wanted here) | caught |
| C14 | the swap reaches the game | caught |
| C15 | the text hints are not swapped | caught |
| C16 | the help bar ignores the swap | caught by 2 |
| C17 | a swap changed on a shown view leaves its help bar's entries | caught |
| C18 | the controller type is ignored | caught |
| C19 | a new family rebuilds the view at the system view | caught by 2 |
| C20 | the help bar never follows the pad last pressed | caught |
| C21 | no notice when a pad connects | caught |
| C22 | the notifications switch ignored | caught |
| C23 | the notice fades by OpenEmu's fractions, not ES-DE's measured time | caught |
| C24 | the notice held ten times as long | caught |
| C25 | the library's hint not redrawn when the first pad arrives or the last leaves | caught |
| C26 | the rebind capture hears only the first pad | caught |
| C27 | Preferences' swap switch saves nothing | caught |
| C28 | the swap moves a function's glyph but not the theme's icon | caught |
| C29 | the window's close leaves the manager undisposed | caught |
| C30 | LunaP: the keyframes ignore FadeTime | caught by 2 |

**30 of 30 caught, none after a test was changed.** Two qualifications. C23 and C24 are caught by the notice test's
assertion of the layer's `Duration` and `FadeTime`, a check of configuration rather than of drawn opacity; the drawn fade
is held by LunaP's own tests (C30). And the tests were written after the rules, each with its rival in view, which is why
a clean sweep here is weaker evidence than it would be against tests written first. §15.14's lesson, that a mutation of
existing code cannot produce an absent line, is what C3, C4 and C29 answer: each removes a cleanup that exists.

### 24.9 Predictions retired

| # | Predicted | Measured | Verdict |
|---|---|---|---|
| P103 | every rule of §15.3 from either of two pads, the help bar following the pad last pressed; ES-DE's popup fading 0.4–0.6 s each way and held 2–5 s | every rule from the second pad with the first attached, and both pads in the library, menu and sheets; the help bar follows the pad last pressed; ES-DE's popup 497 ms in, 2,988 ms held, about 500 ms out (n = 1) | held |
| P119 | the pass opened and closed in no more calendar days than the lower end of its estimate (2 days) | opened and closed on 2026-09-26 | held |
| P121, P122 | (§24.1) | not measured; for Pass 1 | open |

### 24.10 The handheld run

For Pass 1's hardware session, on the Legion Go S (build from this branch):

1. **Game Mode, built-in controls only.** Start Mistress: no notice at start (P121). Steer the library and the themed
   view; Preferences ▸ Controllers shows Automatic and the help bar draws the family §15.5 gives Steam's virtual pad
   (P100).
2. **A second pad in Game Mode** (any Bluetooth or USB pad): the notice appears at the top centre within about a second,
   with the name Steam reports (P121); both pads steer the library, the pad menu, a sheet and the game options; pressing
   on either switches the help bar to that pad's family; pulling it shows "disconnected" and the built-in controls carry
   on.
3. **The same in Desktop Mode** (P122), and whether the name and family are the hardware's there.
4. **A game with two pads:** only the first pad plays; the second's guide button opens the pad menu over the game.
5. **Controller Type** set to each family: only the help bar's icons change, legibly at arm's length (the generic set's
   face buttons especially, §24.7).
6. **Swap A and B:** B chooses and A goes back in the library, menus, sheets and the on-screen keyboard; X and Y are
   as before; the hints and the help bar say so; in a game, A and B are as bound.
7. **First controller only:** the second pad steers nothing; games unaffected.
8. **A pad that registers twice** (if one is at hand): two notices, and the switch in 7 as ES-DE's remedy.
9. **Sleep and wake** with a pad connected: whether SDL reports it removed and added again, and what the notices say.

### 24.11 Not done

- **No real device.** SDL's own added and removed events, and how soon it delivers them, were not exercised; the tests
  stand at `IPadDevices`, beneath which `SdlPadDevices` is a list of one-line SDL calls with no test. Everything in §24.10.
- ~~**Player 2 in games** (Q22 (b)): not built; `EmuSen_Input.md` §6.~~ Built 2026-10-04: `EmuSen_Input.md` §8.
- **The notice has no pad glyph** beside its words (§21.3 thought one might be wanted); LunaP's `NoticeLayer` holds text only.
- **The fade's shape** is linear where ES-DE's fits a power of about 1.35–1.4 (§24.4).
- **ES-DE's connection popup** was not recorded, nor its words; the pass did not run ES-DE.
- **The controller type does not reach the text hints**, which keep an Xbox pad's letters in every family, as before.
- **Preferences does not list the pads connected**; the Controller Bindings window still names the first only
  (`ControllerName`, §4.4 of the settings reference).
- **Shared files.** `PadHeld` moved from `MainWindow.Pad.cs` to `MainWindow.Controllers.cs`; a branch that edits it in
  the old place (§22's) will conflict there at its merge.

## 25. Pass 3: ES-DE's theme list as a browser and installer, and a survey of the listed themes (2026-09-26)

*Opened 2026-09-26, on request that day:* a theme browser and installer, similar to ES-DE's. It is §21.3's pass 3, under Q27 (every listed theme, its licence line shown before the download) and Q28 (the
survey fetches XML files only), both accepted in §10.1. While it was being built a standing requirement was added:
store in SQLite wherever possible. The player's account is §4.62 of the settings reference; this is the
record.

**Sources.** ES-DE's behaviour is taken from `USERGUIDE.md`'s "Theme downloader" section, in stage (c)'s copy
(`~/.cache/emusen/bigpicture/motion/docs/`); ES-DE's source was not read and ES-DE was not run. The list's format was
read from the list itself: `themes.json` was fetched once, at 2026-09-26 21:49 UTC, into
`~/.cache/emusen/bigpicture/theme-survey/`, to learn its fields before the parser was written. That copy is the one the
survey used (§25.9); it was not fetched again. Nothing from the list, from any theme or from ES-DE entered either
repository: every test runs on a synthetic list, synthetic screenshots drawn in WiseMan and synthetic themes.

**Numbering.** §22 and §23 numbered their own predictions (to P91), §24 took P121 and P122, and §26 took P130–P135;
this section uses **P123–P129**. The plan set its questions to start at Q40; §10.1 answered Q40–Q46 for other sections
while this one was written, so its questions start at **Q47**.

### 25.1 What the list states, and what it does not (measured)

The fetched `themes.json` holds `comment`, `website`, `latestStableRelease` ("51"), `themes` (66 entries) and
`themesAndroid` (for ES-DE's Android build, which Mistress does not show). Of the 66 desktop entries, 65 name a GitHub
repository and one (Grimmlex) a GitLab one, in a subgroup (`gitlab.com/thraeg-group/grimmlex-es-de`); every URL is a
`.git` clone address. Every entry has `name`, `reponame`, `url`, `author`, `newEntry` (false for all 66 on that day) and
`screenshots` (268 in all, each an `image` path inside the list's own repository and a `caption`); `aspectRatios` is
given for 63, `variants` 61, `colorSchemes` 55, `fontSizes` 45, `transitions` 27 and `languages` 23.

Three things the browser was asked to show are **not in the list**, and each had to come from somewhere else:
- **No licence.** The licence line is read from the theme's repository when the player opens the theme: its README's
  licence section, else the licence its host names, else a statement that it states none (§25.4).
- **No branch.** The list gives a clone URL, which in git means the default branch. The host is asked for it (GitHub's
  `repos/<owner>/<repo>`, GitLab's `projects/<path>`), because branches differ: the synthetic fakes use `main`, `master`
  and `trunk` to keep the code honest about it.
- **No date.** "Last updated" is the date of the newest commit on the default branch, from the host's commits API, asked
  when a theme is opened. Asking it for all 66 themes when the list opens would spend 66 of GitHub's 60 unauthenticated
  requests an hour, so the list shows a date only for themes already asked about.

### 25.2 What was built

In `EmuSen.Mistress/BigPicture/` (no Avalonia types):
- `ThemeSource.cs`: a repository on GitHub or GitLab (a GitLab owner may be a group path), with every address Mistress
  asks: archive, newest commit, repository record, README. `FromUrl` reads the list's clone URL and refuses other hosts,
  plain `http`, and paths with `.`, `..`, `\` or `:`.
- `ThemeList.cs`: `themes.json` parsed into `ThemeListEntry` records. A screenshot's path is resolved against the list's
  repository (`gitlab.com/es-de/themes/themes-list/-/raw/master/`) and refused if it climbs out of it.
- `ThemeRecords.cs`: `themes.db`, schema 1, versioned by `PRAGMA user_version` and refusing a newer file as `media.db`
  does (§17.5). Tables: the list (`list_fetch`, `list_theme`, `list_screenshot`), the screenshot index
  (`screenshot_file`), what hosts said (`theme_remote`), installed themes (`theme_installed`) and every downloaded file's
  size, time and SHA-256 (`theme_file`).
- `ThemeBrowser.cs`: the list, its expiry, screenshots, host answers and downloads for one open browser, all cancelled
  and the client released on `Dispose`.
- `ThemeDownloads.cs`, generalised from stage (f): any host; the record moved into `themes.db`; refusals of folders it
  did not write; local changes; files the player added kept across an update.

In `EmuSen.Mistress/Views/`: `ThemeBrowserWindow` (the list and a preview) and `ThemeDetailWindow` (a theme), both
`ToolWindow`s of LunaP controls (`LunaList`, `FittedImage`, `FieldRow`) presented as sheets; `ThemeSettingsWindow`'s
Themes tab gained the Browse row in place of the Art Book Next button. Nothing was needed in LunaP. In WiseMan:
`FakeThemeHosts`, `ThemeBrowserModelTests`, `ThemeBrowserSheetTests`, `ThemeBrowserPictureTool` and `ThemeSurveyTool`.

### 25.3 Storage: what moved into SQLite, and the one file that stayed

Stage (f) wrote each downloaded theme's record into the theme's own folder (`.emusen-theme`, JSON). Under that
requirement it moved into `themes.db`, together with everything else the browser writes and reads back; the player's
choices stay in `appsettings.json` (§4.62 has the table). The argument for SQLite here is the one `EmuSen_Stack.md`
makes for program-written data, and none of §4.1's reasons for keeping configuration in JSON applies: nobody edits these
records by hand, and nothing reads them at type initialisation.

**Why a stamp file remains.** A row in `themes.db` names a folder by its path. If the player deletes a downloaded folder
and copies another in under the same name, the row still names that path, and a removal trusting the row alone would
delete a folder Mistress never wrote. The stamp now holds only an id, the row's key; removal and replacement need both to
agree. `Removal_and_replacement_refuse_every_folder_Mistress_did_not_write` builds exactly that case, and mutant B27
(the row alone) is caught by it. This reason is specific to deletion: nothing else reads the stamp, and a store that
never deletes would not need one.

**Stage (f)'s folders** carry the old stamp. The first read moves it into `themes.db` (source, commit, date), rewrites
the stamp to the id form, and records no file hashes, so their local changes cannot be told and the detail says so.
`BigPictureThemeListTests`, which writes old stamps, passes unchanged.

### 25.4 The rules, and their tests

`ThemeBrowserModelTests` (14) test the model; `ThemeBrowserSheetTests` (12) test the sheets through `ThemedSession`;
`ThemeDownloadsTests` (9) and two rewritten `ThemeSettingsSheetTests` cases keep stage (f)'s rules. Every server is
`FakeThemeHosts`: a fake GitLab serving a synthetic `themes.json` of three themes (two on GitHub, one on GitLab in a
group path; each with a different licence situation) and its screenshots, drawn by SkiaSharp in the fixture, and fake
repositories answering the repository record, the commits API, the README and the archive, with a counter of every
request.

| Rule | Test |
|---|---|
| Every field the browser shows is parsed; `themesAndroid` and entries on other hosts are left out | `The_list_parses_every_field_the_browser_shows`, `Each_host_s_addresses_are_built_from_the_list_s_url` |
| The list is kept in `themes.db`, read back equal, asked again only after a day or on Refresh | `The_list_is_kept_in_themes_db_and_asked_again_only_after_a_day_or_on_refresh` |
| No request until the browser opens; the first opening asks for the list and one screenshot; a second opening that day asks nothing; Refresh asks for the list | `Nothing_is_asked_until_the_browser_opens_or_refreshes` |
| A screenshot is fetched once and kept a month | `A_screenshot_is_fetched_once_and_kept_for_a_month` |
| The licence line: README, then host, then none; Download waits for it and sits below it | `The_licence_line_comes_from_the_readme_then_the_host_then_says_none`, `Download_waits_for_the_licence_line`, `A_theme_s_detail_shows_its_screenshots_supports_update_and_licence_before_download` |
| Installed, update available and local changes are told apart; a re-timed but unchanged file, an added file and theme-customizations are not changes | `Installed_update_and_local_changes_are_told_apart` |
| An update over local changes asks, and Cancel fetches nothing; agreed, it keeps theme-customizations and added files, and an added file loses to the update's own | `An_update_asks_before_replacing_local_changes_and_keeps_the_player_s_files`, `An_update_over_local_changes_asks_first` |
| GitLab's archive, commit and group path | `A_GitLab_theme_downloads_from_GitLab_s_archive`, `The_themes_tab_checks_for_and_applies_an_update` |
| No folder Mistress did not write is removed or replaced, including a hand-copied folder at a downloaded one's path and a same-named repository of another owner | `Removal_and_replacement_refuse_every_folder_Mistress_did_not_write`, `A_same_named_repository_of_another_owner_does_not_replace_a_theme` |
| Stage (f)'s stamps are imported; `themes.db` is versioned and a newer one refused | `A_stage_f_stamp_is_imported_into_themes_db`, `Themes_db_is_versioned_and_a_newer_one_is_refused` |
| A download stopped by Cancel, by closing the detail, the browser or the window leaves no partial file, no row, and no file open under `home/Themes` (read from `/proc/self/fd`) | `A_stopped_download_leaves_nothing_half_written` (4 cases), `A_closed_browser_stops_its_download_and_releases_its_files` |
| Install from the detail: the first theme becomes the folder, its About sheet opens once, and the Themes tab lists it with Use beside EmuSen's own row | `A_theme_downloads_from_its_detail_and_appears_in_the_themes_tab` |
| Every control of the browser and the detail is reached by the pad, and A on a row opens it | `Every_control_of_the_browser_and_the_detail_is_reached_by_the_pad` |

**A defect the pad test found.** On its first run, A on a list row did nothing. The router sends Enter to a focused row
(§4.45.3 of the settings reference), and the browser listened for it with an ordinary `KeyDown` handler on the list,
which never ran. Registered with `handledEventsToo`, as the shader browser's and the cheat database's handlers are, it
runs, and mutant B36 (the handler removed) is caught. That the list box handles Enter on its way up is the inference that
fits both observations; it was not traced in Avalonia. A mouse's double click had worked throughout, so only a pad would
have met the defect.

**A trap in the tests, not the product.** The first sheet tests hung: they installed a theme with
`FetchAsync(...).GetAwaiter().GetResult()` on the dispatcher, whose awaits then waited for the thread that blocked on
them. `FakeThemeHosts.Install` runs the download on the thread pool. The same tests installed before `ThemedSession`
set `DataStore.OverrideDirectory`, so the first install landed in a previous test's temporary home; nothing reached the
user's own `home/`, which was checked, and the order was corrected.

### 25.5 When Mistress goes to the network

| The player | Requests |
|---|---|
| opens Theme Settings or its Themes tab | none |
| opens the browser | the list, if the kept one is a day old or more; one commit request per installed listed theme; the selected theme's first screenshot after a quarter of a second |
| selects another theme | its first screenshot, after a quarter of a second, if not kept |
| opens a theme | the repository record, the newest commit and the README, once a day at most (two are GitHub API requests) |
| presses Previous or Next | that screenshot, if not kept |
| presses Refresh | the list, and the installed themes' commits |
| presses Download or Update | the newest commit, then the archive |

Nothing runs at start, in the background or on a timer. GitHub's hourly allowance is read from its answer: when it is
spent, the line says so and names the time it resets, rather than reporting the theme as broken.

### 25.6 Pictures

Written by `ThemeBrowserPictureTool` (`EMUSEN_BIGPICTURE_PNG=1`) at 1280×800 to
`~/.cache/emusen/bigpicture/png/theme-browser/`, on the synthetic list and fake hosts: `browser-list`,
`detail-screenshots-licence`, `detail-downloading` (a synthetic archive of 2.5 MB held at 42%) and
`themes-tab-after-install`. They were looked at. The first detail picture put the licence below the fold: at 1280×800
the screenshot, at 640×360, pushed it out of the sheet's scrolling area, so a pad player would have met Download before
the licence. The detail was rebuilt in two columns, with the licence and the buttons below both and outside any
scrolling area, and `A_theme_s_detail_…` now requires the licence row to end above the Download button; mutant B14
(the button above the licence) is caught by it.

### 25.7 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/mutate_theme_browser.py`, with its log in `run-theme-browser.log` and its
verdicts in `mutants-theme-browser.txt` (`-rerun.txt` for the second round). Each mutant was built with `-m:2` and run
alone under `nice -n 10`, against the tests of its area only, and the source was restored in a `finally`. Before
changing a file the runner records it in a state file, and a runner that starts finds and restores any mutant a cut-short
run left behind (the machine had hard-reset once that day). The tree was rebuilt clean after each round.

**38 mutants: 36 caught at once, one survived and one did not build; both then caught.**

| Area | Mutants |
|---|---|
| The list and the hosts | B1 an entry on another host listed, B2 languages not read, B3 a screenshot path climbing out, B4 GitLab's archive address, B5 GitLab's subgroups dropped |
| Network only on the player's action | B6 the Themes tab fetching the list, B7 the list fetched on every opening, B8 the list never expiring, B9 a kept screenshot fetched again, B10 the host asked at every opening |
| The licence line | B11 the README ignored, B12 the host's licence ignored, B13 Download before the line is read, B14 the line below Download |
| States | B15 no update mark, B16 a same-size edit unseen, B17 a re-timed file counted, B18 theme-customizations counted, B19 a deleted file not counted |
| Install and update | B20 local changes replaced without asking, B21 customizations dropped, B22 added files dropped, B23 an added file beating the update's, B24 a download that does not load swapped in, B25 a folder not Mistress's replaced, B26 another owner's same-named repository replaced |
| Removal and the record | B27 the row alone trusted, B28 the row kept after removal, B29 stage (f)'s stamp not imported, B30 a newer `themes.db` written |
| Stopping | B31 disposing cancels nothing, B32 closing the browser stops nothing, B33 closing the detail, B34 the theme sheet's stop not reaching the browser, B35 Cancel Download doing nothing |
| The pad and the Themes tab | B36 A on a row, B37 a first download not becoming the theme, B38 no About sheet after a first download |

- **B26 survived:** no test downloaded a second repository of the same name from another owner, which ES-DE's list
  could hold, since folders are named by repository alone. `A_same_named_repository_of_another_owner_does_not_replace_a_theme`
  now does, and catches it.
- **B27 did not build:** the mutant's text dropped the pattern variable's scope. Rewritten to trust the row whatever the
  stamp's id, it is caught by the removal test.

**The runs.** Narrow runs of the theme classes after each change: at the end, the five theme classes gave 53 tests, 52
passed and 1 skipped (the picture tool). One broad run at the very end, on the tree merged with WiseMan, of the Mistress
filter without `ShaderSettingsWindowTests`, `ShaderBrowseBench` and `SceneGpuBench`, under `nice -n 10` and `-m:2`:
**934 tests, 913 passed, 21 skipped (the picture, survey and live tools, which need their environment variables), none
failed, in 3 min 43 s.**

### 25.8 The survey of the listed themes (Q28)

**Method.** `ThemeSurveyTool` in WiseMan, skipped unless `EMUSEN_THEME_SURVEY` names its phase, run from a copy of the
build so the working tree could be rebuilt meanwhile. Everything it fetched is under `~/.cache/emusen/bigpicture/theme-survey/`
(`xml/`, `trees/`, `fetch.log`, `survey.json`); nothing entered the repository.
- **Fetch.** For each of the 66 themes, one listing of the default branch's tree (GitHub's `git/trees/HEAD?recursive=1`,
  one API request; GitLab's paginated `repository/tree`), then every `.xml` file from the hosts' raw file addresses, one
  at a time, 2 s after each listing and 0.2 s after each file, reading GitHub's remaining allowance from every answer and
  waiting for its reset rather than exceeding it (it was never reached: 66 listings over three sessions). The run was cut
  twice, once by a hard reset of the machine and once when a session ended; each restart skipped the listings and files
  already held.
- **A narrowing, made during the run.** After 17 themes the listings showed per-system files in bulk: of the 66 themes'
  23,675 XML files (203.0 MB as GitHub declares them), most are named for one of ES-DE's 154 systems (`3do.xml`,
  `amiga.xml`, …) or sit under a folder so named. The loader asked for EmuSen's five systems never reads those, so the
  tool stopped fetching any file whose name or folder is one of the 149 other systems' short names (read from
  `USERGUIDE.md`'s systems table). That leaves 8,480 files and 68.8 MB declared. The 17 themes fetched before the change
  kept their other-system files, which the analysis never reads. **Fetched in all: 12,045 files, 97.0 MB**, in about
  12,100 requests; 11,630 files (107.4 MB) were left unfetched as other systems'. GitLab declares no sizes, so Grimmlex's
  are not in the declared totals.
- **Analysis.** For every theme, `ThemeCapabilitiesReader` and then `ThemeLoader.Load` for each of the five systems and
  each selectable variant (or once when it declares none), 1,990 loads in 3.2 s. A theme counts as loading when every
  system comes out themed. Undrawn elements are those not in `SceneMapping.Drawn`; undrawn properties are the explicit
  properties of drawn elements that `SceneMapping.IsMapped` does not map; wheel carousels (`verticalWheel`,
  `horizontalWheel`) are counted as undrawn, since `PrimaryElements.Carousel` draws only straight rows and columns.

**Caveat before any number.** Only XML was fetched, so every image, font and sound a theme names is missing, and the
loader reports each as `PathMissing` (a warning, 63 themes). No such warning unthemes a system, so the counts below are
the loader's verdict on the XML, not on a theme as downloaded.

**Loading.**
- **51 of 66 themes are themed for all five systems, and 46 of those with no loader error at all.** Five more (Aura,
  Canvas, Cathode, DEcaffe, Iconic) are themed but log errors that drop an element or a variant.
- **15 are unthemed for all five**, by six rules:

  | Rule | Themes | What the theme writes |
  |---|---|---|
  | `folderLinkSize` of `badges` must be a number | CarAlt, CodyWheel, Diamond, ES-DE-Mini, Showcase, Slick (Remixed), TateGriddy | a pair, `0.85 0.9` |
  | a malformed `capabilities.xml` | Razor, SimCar, SimpleMenu, X20s | an unescaped `&` in a label (`Game & Watch`) |
  | `size` of `gamelistinfo` must be a pair | Artflix (Revisited), CoinOPS | `w 0.02` |
  | `<transitions>` placed directly in `<theme>` | Grimmlex, X20s | a legacy-looking placement |
  | `visible` must be a boolean | Retrofix (Revisited) | `no` |
  | `metadataElement` must be a boolean | X20s | `flase` |

  THEMES.md types `folderLinkSize` as FLOAT, so the loader follows the document. But these themes are on ES-DE's own
  list, and THEMES.md says an error of this kind "will abort the theme loading"; either ES-DE parses these values
  leniently (a float parser reading the first number, an XML parser accepting a bare `&`) or ES-DE refuses them too and the
  list carries themes it cannot load. Which is not known: ES-DE was not run on them. P123 and P124 state the expectation,
  and Q47 asks what to do. *(§31, 2026-09-27: ES-DE loads all 15, and X20s's malformed file is its `variant_02.xml`,
  not its `capabilities.xml`; §31.5.)*

**What big picture does not draw.**
- **No listed theme is drawn whole.** Every one of the 51 themed themes sets at least one property the scene does not
  map; the median is 12. Four (Art Book Next, Colorful (Revisited), Colorful (Simplified), Game OS) miss only video
  properties and `badges`' `controllerSize` and `folderLinkSize`.
- **Elements not drawn**, by the number of themes that use them: `gameselector` 20, `gamelistinfo` 6, `animation` 4.
  **Wheel carousels:** 9 themes (Artflix, Aura, Canvas, CarAlt, CodyWheel, CoinOPS, Iconic, Showcase, SimCar); across the
  themed loads, carousel types were `horizontal` in 43 themes, `vertical` 18, `verticalWheel` 7 and `horizontalWheel` 1.
  **28 of 66 themes** use an undrawn element or a wheel in the files present, **20 of the 51** that load.
- **Properties not drawn**, 90 in all. By themes: `video.delay` 51, `video.pillarboxes` 45 and ten more video
  properties (pass 12's); then `badges.controllerSize` 37, `carousel.lineSpacing` 32, `carousel.selectedItemMargins` 32,
  `helpsystem`'s dimmed set (`textColorDimmed`, `iconColorDimmed` 24, `originDimmed`, `posDimmed` 22, `fontSizeDimmed`
  19, `opacityDimmed` 14), `carousel.horizontalOffset` 23, `badges.folderLinkSize` 21, `badges.controllerPos` 18,
  `image.stationary` 17, `carousel.imageInterpolation` 17, `image.brightness` 14, `badges.folderLinkPos` 13,
  `carousel.imageCornerRadius` 12, `rating.interpolation` 12, `textlist.selectorImagePath` 11 and `image.gameselector` 10.
  The full table, per theme, is `survey.json`.

**What this sizes for pass 14.** The survey's order differs from §21.3's guess. `gameselector` leads the elements, as
expected, but the widest gaps are properties of elements already drawn: the badges' controller and folder-link icons,
the help system's dimmed state, and the carousel's margins, line spacing and offsets, each used by a fifth to a half of
the list, and each small beside a new element. Wheels matter to 9 themes, `gamelistinfo` to 6 and `animation` to 4. The
loader's six refusals come first of all, since they keep 15 themes from showing anything (Q47).

### 25.9 Predictions

**Retired.**

| # | Predicted (§21.6) | Measured | Verdict |
|---|---|---|---|
| P104 | at least 90% of listed themes load for the five systems with no loader error; at least half use an element or carousel type not drawn; the XML-only survey fetches under 50 MB | 46 of 66 (70%) load with no error, 51 (77%) themed; 28 of 66 (42%) use an undrawn element or a wheel; all XML is 203 MB declared, the five systems' share 68.8 MB, and 97.0 MB was fetched | failed on all three clauses |

The first clause failed on six rules of the loader's (§25.8), not on themes that are broken in any way known. The second
was an underestimate of how much the list is built from elements already drawn: its gaps are properties more than
elements. The third missed how many XML files a theme writes: most write one per system for ES-DE's 154, so a theme's
XML averages 3.1 MB declared across the 65 GitHub themes, and even the five systems' share came to 68.8 MB.

**No predictions were written before the build.** The plan listed P104 as the pass's only prediction, and the build's
rules were stated as tests (§25.4) rather than as predictions. The following are written now, for what this pass could
not measure:

| # | Prediction | Retired when |
|---|---|---|
| P123 | ES-DE 3.4.1, run from the scratch copy, themes CarAlt for `snes` although its `badges` set `folderLinkSize` to a pair, drawing the icon at the pair's first number | ES-DE is run on it |
| P124 | ES-DE 3.4.1 loads Razor, whose `capabilities.xml` carries a bare `&`, and lists its `Game & Watch` label | the same run |
| P125 | Downloading Art Book Next from the browser on the desktop fetches an archive of 205–230 MB (P11, by download this time) and completes, unpacked and checked, within 3 minutes | the player's first live download |
| P126 | A first live opening of the browser costs one list request, one screenshot and one commit request per installed listed theme, and a second opening that day costs nothing | the first live opening, from a request log |
| P127 | A player opening 20 themes' details in an hour stays within GitHub's 60 unauthenticated requests (40 are counted) | a live session, from the rate headers |
| P128 | Fetching the survey's missing assets for the 51 themed themes would clear every `PathMissing` warning and change no count in §25.8 | a later survey with assets, if asked |
| P129 | Mapping `badges.controllerSize`, `folderLinkSize` and their positions, `helpsystem`'s dimmed set, and the carousel's `lineSpacing`, `selectedItemMargins` and offsets would leave at least 8 of the 51 themed themes with only video properties unmapped | pass 14's first step |

### 25.10 Open questions

- **Q47, the loader's six refusals.** Fifteen listed themes are refused whole by rules THEMES.md states (§25.8). Options:
  (a) keep THEMES.md's strictness, and those themes show EmuSen's fallback;
  (b) run ES-DE on them first (P123, P124) and match what it does, rule by rule;
  (c) accept all six leniently now.
  **Recommendation: (b).** The oracle for this format is ES-DE's behaviour where the document and the list disagree, as
  it was for the grid (§16.5); a theme ES-DE draws should not be one Mistress refuses, and a theme ES-DE refuses should
  not be one Mistress draws. It is one short run of the scratch ES-DE on two themes.
- **Q48, pass 14's order.** The survey's order: the loader's refusals (Q47); the badges' and help system's missing
  properties and the carousel's margins and offsets (widest, cheapest); `gameselector` (20 themes); wheel carousels (9);
  `gamelistinfo` (6); `animation` (4). **Recommendation: that order,** with any theme a player chooses moving its own
  elements to the front, as §21.4 already allows.
- **Q49, ES-DE's full-screen screenshots.** ES-DE's downloader shows a theme's screenshots full screen on X; the detail
  here shows them one at a time at a fixed size. Options: build it now; build it with pass 9's media viewer, which is the
  same pager. **Recommendation: with pass 9.**

### 25.11 Not done

- **ES-DE was not run.** The six refusals of §25.8 are the loader's, not measured against ES-DE (P123, P124, Q47).
- **Nothing was downloaded from a real theme host** except the survey's XML and the one fetch of `themes.json`; the
  browser, its screenshots, host answers and downloads were exercised only against `FakeThemeHosts` (P125–P127).
- **The survey fetched no assets,** so every image, font and sound path is missing, and nothing was rendered from a
  surveyed theme (P128).
- **Full-screen screenshots,** sorting and searching the list: not built (Q49).
- **ES-DE's git transfer** fetches only what changed; Mistress fetches the whole archive on every update.
- **Nothing ran on the handheld.**
- **Undrawn properties are counted as set, not as seen.** A property set on an element that a variant then hides still
  counts; the counts are an upper bound on what a player would notice.

---

## 26. Retiring OpenEmu's sources: a plan (2026-09-26)

> **Retired 2026-09-27.** OpenVGDB is kept as a permanent fallback (§10.1). What follows remains
> the record of what the fallback was measured to yield; its removal steps are not to be carried out.
>
> **Built from it, 2026-09-27 (§28):** Q42's Remove and Q45's *Use Another Game's Cover…*. One departure from 26.4 is
> deliberate: the chooser was proposed to *copy* the chosen cover into the art folder under this game's name; it was
> built as a row of `games.db` naming the other game, resolved when the cover is looked up. A copy would overwrite a
> picture the player may already have placed there, would go stale when the chosen game's cover later improves, and
> could be undone only by deleting a file; the row overwrites nothing, follows the other game's cover as it changes, and
> is undone by deleting the row. 26.8's test for it ("a copy into the art folder") is therefore replaced by its
> opposite: the art folder and the ROM folder are byte- and time-identical before and after.

*A plan, written on branch `desktop-game-options` beside §27; nothing is removed by it.* The direction of
2026-09-26 (§10.1): eventually retire sourcing game information from OpenEmu's library and use ScreenScraper.
"OpenEmu's library" is read here as §4.39's pair of sources, which §4.60 made the failover: **OpenVGDB**, OpenEmu's game
database, which names a file, and **libretro-thumbnails** (`thumbnails.libretro.com`), which has the box under that
name, with OpenVGDB's own cover address tried last. §25 is being written on another branch at the same time, so this
section is numbered 26, its predictions start at P130 and its questions at Q40, leaving P123–P129 and Q36–Q39 to §25, as
§17 once left a range to stage (f).

The plan answers five questions: what the failover gives that ScreenScraper does not (26.2–26.3), what replaces it for
the games ScreenScraper cannot help with (26.4), how what has been fetched already survives (26.5), what a machine
without the developer credentials is left with (26.6), and in which order the removal is done and tested (26.7–26.8).
The questions still to be decided are in 26.9.

### 26.1 The argument in brief

ScreenScraper identifies more of this library than OpenVGDB does, by a wide margin, and on the one sample where both
were asked about the same files, the failover would have added no cover that ScreenScraper had not already given. On a
machine with the developer file the failover is therefore, as far as the evidence reaches, dead weight: a second
database of 42 MB, a second server, and a code path that runs only when ScreenScraper has nothing. The failover's one
indispensable property is not its coverage but that it needs no credentials: on a machine without the developer file,
which Q5 makes every distributed build, it is the only online art Mistress has. The retirement is therefore argued in two
halves with different evidence. Where ScreenScraper can be used, the failover can go now, subject to one live
measurement (P130). Where it cannot, removing the failover removes online art altogether, and whether that is acceptable
is a decision to make, not a technical finding (Q40).

### 26.2 What was measured, and how

**No network.** Every number here is from what the machine already held, read without writing:
- `media.db` of §17.9's live run (`~/.cache/emusen/bigpicture/scrape-live/20260926-134821/Media/`): ScreenScraper's
  answers for 40 files drawn at random (seed 20260926) from the library's 5,520, with the kinds each answer offered;
- `games.db` of the published tree (`out/linux-x64/Mistress/home/Library/`), whose `cover_lookup` table holds the
  failover's 1,113 recorded outcomes, made between 2026-09-21 and 2026-09-25 under §4.39's rule that a tile drawn
  without a cover asked for one;
- the same tree's cover art folder, `home/Artwork/<console>/`, where §4.39's switch wrote what it found;
- OpenVGDB v29.0 (`openvgdb.sqlite`, 42 MB, the copy §4.39 downloaded into the same tree), queried as `OpenVgdb.cs`
  queries it: by file name without extension, then by the MD5 of the bytes `CoreDescriptor.OpenVgdbBytes` gives (the
  NES without its iNES header, the SNES without a copier header, the N64 halfword-swapped, the Game Boy as it is);
- the library itself, `AppSettings.RomDirectory`, read to hash each file and never written.

The databases were copied to a scratch folder before they were opened, so no journal or lock file was
written beside the originals. The scripts are `retire/m40.py` (the 40 files) and `retire/whole.py` (the library) in
that folder, with `retire/art.py` for the art folder; the whole-library pass took 3 min 53 s under `nice -n 10`.

**What was not measured.** Whether libretro's server has a box for a game was not asked; it is known only for the 1,113
games the failover asked about before 2026-09-26. ScreenScraper's answers are known only for the 40 files of §17.9. The
cross-tabulation of 26.3 is therefore of 40 files, and the whole-library figures are OpenVGDB's identification alone.

### 26.3 What the failover gives that ScreenScraper does not

**Identification, on the same 40 files.**

| | ScreenScraper (§17.9) | OpenVGDB (by name, then by MD5) |
|---|---|---|
| Games identified | 39 of 40 | 17 of 40 (9 by name, 8 by MD5) |
| The 17 files named as released games | 17 of 17 | 17 of 17 |
| Hacks, translations, pirate, unlicensed and multicart dumps, prototypes, betas, alternate dumps and the unnamed dump (23 of the 40) | 22 of 23 | 0 of 23 |
| Identified by this source alone | 22 | 0 |

Every file OpenVGDB identified, ScreenScraper also identified, and gave a cover for.

**The games ScreenScraper left without a cover.** Seven of the 40: the one file it did not know (`ZZZ_UNK_GOLF_GRA`, a
dump with no name), three it lists as non-games (`ZZZ(notgame)`: the Minolta program cartridge and two multicarts, each
with a mix image and nothing else), and three SMB1 hacks it knows but has no box for. OpenVGDB knows **none of the
seven**, so the failover, which asks libretro only for a name OpenVGDB gives, can find nothing for any of them. Of the
seven, one (the 68-in-1 multicart) had been asked of the failover before, and its recorded outcome is Unknown.

**The measured yield of the failover beside ScreenScraper is 0 of 40.** A sample of 40 bounds the true rate only
loosely: by the rule of three, a rate above 7.5% would have shown at least one case in 40 with 95% probability, so the
sample alone does not exclude a yield of up to about 400 covers over the library. A structural argument narrows it.
OpenVGDB's ROM list is No-Intro's, and ScreenScraper's includes No-Intro's (§17.1, P60's premise), so a game OpenVGDB
names and ScreenScraper does not know should be rare; the failover's yield can come only from games ScreenScraper knows
without a `box-2D` for which libretro nonetheless has one. That case is possible, since the two collections of scans are
independent, and it is what P130 measures.

**OpenVGDB over the whole library** (5,520 files, read-only):

| Console | Files | OpenVGDB identifies | by name | by MD5 | Release with a cover address | with a description |
|---|---|---|---|---|---|---|
| Game Boy (`.gb`) | 1,915 | 1,576 (82.3%) | 1,331 | 245 | 1,213 | 1,294 |
| NES | 3,537 | 998 (28.2%) | 0 | 998 | 917 | 931 |
| SNES | 48 | 44 | 0 | 44 | 43 | 43 |
| Nintendo 64 | 20 | 20 | 20 | 0 | 18 | 19 |
| **All** | **5,520** | **2,638 (47.8%)** | 1,351 | 1,287 | 2,191 | 2,287 |

The NES shelf's names are GoodNES's (`(U)`, `[!]`, `[a1]`), not No-Intro's, so none matches by name and a little over a
quarter matches by content; the rest are hacks, translations, pirate carts and multicarts, which No-Intro does not list.
ScreenScraper identified 22 of the 23 NES files of the sample.

**The failover's recorded outcomes** (`cover_lookup`, 1,113 games asked between 2026-09-21 and 2026-09-25): 670 Found
(60.2%), 335 Unknown to OpenVGDB (30.1%), 108 known to OpenVGDB but with no box on libretro's server under any of the
three names tried (9.7%). By console: Game Boy 554 found, 143 unknown, 104 without art; NES 56, 188, 0; SNES 43, 4, 1;
N64 17, 0, 3. The 670 found covers are the published tree's art folder: each Found row has its file there under libretro's
naming, and the folder holds no other file (checked file by file, `retire/art.py`). They were written there by §4.39's switch before §4.60 moved the failover's writes to
`home/Media/openemu/`. No file exists under `home/Media/openemu/` on this machine: the failover as §4.60 built it has not
run in the published tree.

**What else the failover provides, beyond covers.**
- **Text: nothing.** OpenVGDB holds a description for 2,287 of the 2,638 games it names, and developer, genre and date
  for most, but Mistress never kept any of it (§4.39: "No description, rating or other metadata is kept"). The one datum
  kept is the No-Intro name, in `cover_lookup.rom_name`, and no code reads it. ScreenScraper's text is what every view
  shows (§17.6).
- **OpenVGDB's own cover addresses: nothing.** All eight tried in §4.39 answered 403 from GameFAQs; the address is tried
  last and has not been seen to answer.
- **No credentials.** The failover is the only online art on a machine without the developer file (26.6).
- **No daily quota**, as far as is known: libretro's server states none, and none was measured. ScreenScraper's is 10,000
  requests a day without a member account (§17.9).
- **Not OpenVGDB's to retire: the byte transforms.** `CoreDescriptor.OpenVgdbBytes` is console knowledge (a header to
  drop, a byte order to swap) that ScreenScraper's step 2 uses too (`MainWindow.Scrape.cs`, `TransformFor`). It stays,
  under a name that does not tie it to OpenVGDB.
- **Not a network source: libretro's naming.** The cover art folder of §4.33 is read under libretro-thumbnails' names and
  folders (`ArtworkIndex`, `CoreDescriptor.CheatSystemNames`). That is a convention for the player's own files and stays.

### 26.4 What replaces it for games ScreenScraper cannot help with

The seven games of 26.3 fall into three kinds, and each has a different remedy. None of them is the failover, which
had nothing for any of the seven.

| Kind (of the 40) | Example | Replacement |
|---|---|---|
| Unknown to ScreenScraper (1) | an unnamed dump | the player's own cover, or the placeholder; a name search finds nothing to search by |
| Known, no box, a game derived from another (3) | an SMB1 hack | **ScreenScraper's search by name** (Pass 8's "Find by name…" chooser, `jeuRecherche`), where the player picks the base game's page; or **the base game's cover from the library** |
| Known as a non-game (3) | a multicart, a program cartridge | the placeholder, or the player's own cover; ScreenScraper's mix image exists for these (Q43) |

- **Search by name** is §21's Pass 8 as accepted (Q22–Q35, §10.1): started only from "Find by name…" or a run
  the player sets to ask, never as an automatic fallback, so §17.14's rule 1 holds. It costs a request per search and
  one per picture, like any other lookup. It is the replacement with the widest reach, and the retirement should wait
  for it (26.7).
- **A manual pick** exists already: *Add Cover Art from File…* copies a picture into the art folder, where it wins over
  every other source. A second pick is proposed: **Use Another Game's Cover…**, a chooser over the library that copies
  the chosen game's cover (whatever its source) into the art folder under this game's name. It suits hacks and
  translations, whose base game is usually in the same library, and it sends nothing anywhere.
- **The placeholder** is §4.33's drawn tile, unchanged. It is the answer for what nothing can identify.

### 26.5 How what has been fetched already carries over

The rule is that nothing the player has is deleted, and nothing that is shown today stops being shown, by the removal
itself.

- **The 670 covers in the art folder** stay where they are and go on counting as the player's own, first in §4.60's
  order, above ScreenScraper's. They were fetched, not placed, but since §4.60 they cannot be told from a placed cover by
  where they are. They can be told apart by `cover_lookup`: each has a Found row naming its ROM, and the counts match
  exactly (26.3). Whether to keep them first or to rank them below ScreenScraper's region-matched covers is Q41; the
  default proposed is to keep them first, since that is what the player sees today.
- **`home/Media/openemu/<console>/`**, the failover's folder since §4.60, is kept as a read-only source in its present
  place in the order (after the ES-DE folder), read offline, never written again, shown whatever the retired switch
  held. On this machine it is empty.
- **`cover_lookup`** stays in `games.db`: its migrations are append-only (§4.32), and a table no code writes or reads
  costs nothing. `GameRecords.Move` stops carrying its rows. A later migration may drop it; this plan does not.
- **`openvgdb.sqlite`** (42 MB in `home/Library/`) is not the player's data but a download. It stays until the player
  removes it; Q42 asks whether the removal build should offer that, delete it, or leave it.
- **The settings.** `OpenEmuFallback` is read once and dropped at the next save, as `OnlineCovers` was (§4.60).

### 26.6 A machine without the developer credentials

**Today.** A run the player starts on such a machine asks only the failover (§4.60's second case). That is every
distributed build: Q5 decided that no build carries the credentials, and ScreenScraper issues developer credentials to
software authors, not players, so a player cannot bring their own (§4.60's sign-in section). The author's own machines,
the handheld included, read `~/.config/EmuSen/screenscraper-developer.json` and are unaffected (§17.12).

**After the removal, with nothing else changed,** such a machine has no online art at all: the art folder, *Add Cover Art
from File…*, the proposed *Use Another Game's Cover…*, an ES-DE media folder the player points at (§4.52), and the
placeholder. A scrape run there would have nothing to ask, and the Scraping tab would say so instead of offering a run.

Four ways through, none of them chosen here:
1. **Accept it.** Distributed builds have offline art only. Simple; a clear loss for a player of a published build.
2. **Revisit Q5.** Embed the developer credentials at publish time from a file outside the repository, as ES-DE does
   (the option §10 put as Q5). Every build could then scrape; the credentials would be in every copy, obfuscated but
   recoverable, and a leak would be charged against the project's account.
3. **A second source that needs no project credentials.** TheGamesDB with the player's own key is Pass 13, and Q29's (c)
   was accepted (§10.1): a key is issued to any site account, unlike ScreenScraper's developer credentials. It is
   a second source, not ScreenScraper, so it keeps a failover in a new form; the direction of §10.1 names ScreenScraper
   alone.
4. **Keep libretro's thumbnails without OpenVGDB**, asked by the file's own name, for files already named as No-Intro
   names them. It needs no database and no credentials, but it is half of what is being retired, and on this library it
   would reach the Game Boy shelf (whose names are No-Intro's) and almost none of the NES shelf (GoodNES names, 26.3).

Q40 asks which. The steps of 26.7 are ordered so that the half the evidence supports can be done before Q40 is answered.

### 26.7 The steps, in order

1. **Measure the failover's yield where ScreenScraper is usable (P130).** A run the player starts, of the whole library
   or a console, with the failover on, as §4.60 built it. The status window's tally *Filled by OpenEmu* (§4.57) is the
   yield, and the run's log names each game. No code is needed; the run costs ScreenScraper's quota as any run does
   (§17.10: three days for the whole library, or one shelf at a time). This is the evidence the removal rests on, and it
   is started only by hand (§17.14).
2. **Build the replacements:** Pass 8's search by name and its chooser (26.4), and *Use Another Game's Cover…*.
3. **Stop asking the failover where ScreenScraper is usable.** The first behavioural change: a run with the developer
   file never asks OpenVGDB or libretro, whatever the switch says; the switch's row says it applies only where
   ScreenScraper cannot be used. This is reversible by one condition, and it can ship before Q40 is answered.
4. **Answer Q40**, then either remove the failover altogether (5–7), or put the chosen replacement in its place for
   machines without the developer file before removing it.
5. **Remove the code:**
   - `Library/OpenVgdb.cs`, `Library/OpenVgdbDownload.cs`, `Library/CoverFetcher.cs` (270 lines);
   - `Views/MainWindow.Covers.cs` (106 lines) and its calls: `ApplyOnlineCovers`, `StopOnlineCovers`, `AskForCover`,
     `CoverArrived`, and in `MainWindow.Scrape.cs` `FailoverFor`, `FailoverArrived` and the failover half of a run's end;
   - `ScrapeRun`'s `FailoverAsked`, `FailoverFound`, `FailoverPending`, `FilledByFailover` and `FromFailover`;
     `ScrapePlan.Failover`; the tallies and the sentence in `ScrapeStatusWindow`; the *OpenEmu Failover* row of
     `ScrapePreferencesPane`;
   - `AppSettings.OpenEmuFallback`, read once and dropped;
   - `CoreDescriptor.OpenVgdbSystems`, and `OpenVgdbBytes` renamed (for instance `ScrapeBytes`), with every core's entry
     in `CoreCatalog.cs` kept;
   - `GameRecords`' `CoverLookup`, `RecordCoverLookup` and `ForgetCoverLookup`, and the `cover_lookup` lines of `Move`;
     the table itself stays (26.5).
   - `MediaSources` keeps its OpenEmu source, read-only, as 26.5 describes; its constructor no longer takes the switch.
6. **Change the tests** (26.8).
7. **Retire the documents,** not delete them: §4.39 of the settings reference is marked retired and kept as the record
   of how the failover was built and measured, with a pointer to this section; §4.60 loses its failover paragraphs and
   gains the replacements; the README's credits and `THIRD_PARTY_NOTICES.md` keep OpenVGDB and libretro-thumbnails for
   as long as a build can read covers they supplied, then move them to a list of former sources.

### 26.8 Tests and predictions

**Tests.** All headless, on the fake ScreenScraper, never the network.
- *Deleted:* `OnlineCoverTests` and `OnlineCoverWindowTests` (421 lines), whose rules go with the code; their fixtures
  (a synthetic OpenVGDB, a fake thumbnail server) with them.
- *Rewritten:* `ScrapeWindowTests`' six failover cases become their opposites: a game ScreenScraper does not know, or knows
  without a box, asks **no other server** (`OthersAsked` empty), and the tile keeps the placeholder; a run without the
  developer file sends nothing and the Scraping tab says why; a day's quota used up stops the run and hands nothing on.
- *Kept, and made to guard the carry-over:* a test with a cover in the art folder, one in `home/Media/openemu/`, and one
  from ScreenScraper, each shown from the same place before and after the removal (P133); `ScraperTests`' step-2 cases,
  which now exercise the renamed byte transforms; `GameRecordsTests` with a `cover_lookup` row present and untouched.
- *New, for the replacements:* the chooser (Pass 8's own tests) and *Use Another Game's Cover…* (a copy into the art
  folder, no request, the ROM folder's fingerprint unchanged, as §23.8's Hide test measures it).
- *Mutants,* one at a time, as every round since §17.11: the failover asked again when ScreenScraper has no box; the
  OpenEmu folder dropped from the order; the art folder's fetched covers ranked below ScreenScraper against Q41's answer;
  the byte transform lost in the rename; `OpenEmuFallback` kept after a save.

**Predictions,** written before any of this is built:

| # | Prediction |
|---|---|
| P130 | In a run the player starts with ScreenScraper usable and the failover on, the failover fills a cover for fewer than 1% of the games run (26.3 found 0 of 40). |
| P131 | In the same run, OpenVGDB identifies fewer than 10% of the games ScreenScraper answers as unknown. |
| P132 | Of the games ScreenScraper knows without a `box-2D` in that run, fewer than a quarter get a box from libretro. |
| P133 | After step 5, every cover the synthetic library showed before is shown from the same file; no picture shown before the removal disappears. |
| P134 | Step 5 removes between 1,000 and 1,500 lines, tests included, and adds fewer than 150. |
| P135 | A whole-library run after step 3 costs the same ScreenScraper requests as one before it, to within 1%: the failover never cost ScreenScraper requests. |

### 26.9 Open questions

- **Q40, a machine without the developer credentials.** After the removal such a machine, every distributed build
  included, has no online art. Which of 26.6's four: accept it; revisit Q5 and embed the credentials at publish; bring
  TheGamesDB (Pass 13, the player's own key) forward as the credential-free source; or keep libretro's thumbnails asked
  by file name, without OpenVGDB? **Recommendation: accept it for now and do Pass 13 when its terms have been read**,
  since only (3) gives a player of a published build art without putting the project's credentials in every copy; (2)
  is the stronger remedy if the project is content to carry that risk.
- **Q41, the 670 covers in the art folder.** They were fetched by §4.39's switch, not placed by the player, and they rank
  above ScreenScraper's. Keep them first, as today, or rank the ones `cover_lookup` records as fetched below
  ScreenScraper's? **Recommendation: keep them first**, and offer the re-ranking as a Preferences action rather than a
  silent change, so that a cover the player has grown used to never changes by itself.
- **Q42, OpenVGDB's database.** 42 MB in `home/Library/`. Leave it, offer a *Remove* line in Preferences, or delete it in
  the removal build? **Recommendation: offer the line**; it is a download, not the player's data, but deleting a file the
  player may have copied deliberately is not the software's to decide.
- **Q43, a mix image as the cover of last resort.** ScreenScraper gives a mix image even for the non-games it lists with no
  box (26.3). Show it on the tile when there is no box, or keep the placeholder? **Recommendation: the placeholder**; a
  mix image is a composite made for a theme's panel, and the grid would show one tile unlike the rest.
- **Q44, the order.** Step 3 (no failover where ScreenScraper is usable) before Pass 8 is built, or only after it?
  **Recommendation: after**, so that the player never has fewer remedies than today, even for a day.
- **Q45, *Use Another Game's Cover…*.** Build it with the replacements, or leave the manual pick to *Add Cover Art from
  File…*? **Recommendation: build it**; it is the remedy for the hacks and translations that fill this library, and it
  sends nothing anywhere.

### 26.10 What this section did not do

- **No live call.** ScreenScraper, libretro and GitHub were not asked anything; P130–P132 wait on a run the player starts.
- **ScreenScraper's side of the 1,113 recorded lookups is unknown**, so whether ScreenScraper has a box for the 670 games
  the failover found, or for the 108 it found no box for, is not measured.
- **P60 (N64 byte order) is still open.** It matters here only because the renamed transform must keep serving
  ScreenScraper's step 2.
- **libretro's server was not characterised:** its limits, its terms for software that asks it, and its coverage outside
  the 1,113 games are unknown.
- **Nothing was removed.** The failover is on by default and works as §4.60 describes.

---

## 27. ScreenScraper's name offered, and a game's options from the desktop library (2026-09-26)

*Opened and closed on 2026-09-26, on branch `desktop-game-options`,* on the answers to Q18 and Q19 (§10.1). The
player's account is §4.63 of the settings reference; this is the record. Predictions are numbered from P140, after
§26's, and the one question is Q46.

### 27.1 Predictions, and what was found

Written during the build, by the hand that wrote the code, as §18.1 and §23.1 were, and weaker evidence for it.

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P140 | The desktop needs no window of its own: `SheetLayer.Show` already shows the same `GameOptionsWindow` and `MetadataEditorWindow` as owned windows where no layer presents them | the tests find both in `OwnedWindows` on the desktop and on `Sheets.Current` in the built-in big screen, from one `PresentGameOptions` | held |
| P141 | Every control of the editor is reached by the pad with the offer's two buttons shown | measured on the themed editor after its own scrape: none unreached | held |
| P142 | No test of §22 or §23 changes | the four classes of the mutant runner's filter passed unchanged before the round (28 of 28) | held |
| P143 | Of the round's mutants, at least nine in ten are caught on their first run | 20 of 21; the survivor equivalent (27.5) | held |

### 27.2 Q18: ScreenScraper's name, offered

`MetadataDraft` gained `OfferedName` (ScreenScraper's name, while it differs from the Name field and has not been put
away), `TakeOfferedName` and `DeclineOfferedName`; `MetadataEditorWindow` draws the offer under the Name box, a sentence
and two buttons, *Use This Name* and *Keep Current Name*, shown only while there is an offer. `GameMetadata.Scraped` is
unchanged: it still has no case for the name, so ScreenScraper's name is never a baseline and a scrape never puts it in
the field. That is where "don't force on user" is enforced; the offer is only a way for the player to type it with one
press.

**Choices made where the answer is silent.**
- *When the offer shows.* After the editor's own scrape, as the answer asked, and also when the editor opens on a game
  ScreenScraper has already answered for, since the name is already in `media.db` and offering it costs nothing and
  sends nothing.
- *Declining is not remembered.* It lasts for the editing; the offer is there again at the next opening. A remembered
  refusal would need a row of its own in `games.db` for a choice that changes nothing when it is not taken.
- *Taking it is an edit, saved with Save.* The editor saves nothing until Save, as every field does (§23.2); the offer
  follows the same rule rather than writing at once.
- *Q30* (§21.5, a switch for scraped names, off, accepted) is not built. The offer is per game; the switch would be the
  library-wide form of the same choice, and it waits for Pass 8.

### 27.3 Q19: the menu and the editor from the sidebar library

| Route | Code |
|---|---|
| the grid's and the list's context menu: **Game Options...**, **Edit Metadata...** (Ctrl+I shown) | `MainWindow.Library.cs`, `LibraryContextMenu` |
| **Ctrl+I** over either library | `MainWindow.GameOptions.cs`, `EditMetadataFromTheKeyboard`, called from the window's key tunnel |
| the pad menu over the sidebar library, **Game Options...** | `MainWindow.Pad.cs`, `OpenPadMenu` |

`ShowGameOptions` was split: the game's own entries are built by `AddGameEntries` and shown by `PresentGameOptions`, which
both the themed gamelist and the sidebar library call, so there is one menu and one way it is shown.

**Decided, and why.** The themed gamelist's rows (Jump To, Sort Games By, Filter Gamelist, Search) and its collection
entries are left out of the sidebar library's menu. They act on the themed gamelist's own sort, filter and letters,
which are state of the themed view; the sidebar library has its own search box, console filter and collections, and a
menu that changed the themed view's state from the desktop would change a list that is not on the screen. The desktop's
context menu already has Add to Collection and Remove from the collection shown. Select in the sidebar library stays the
favourite, as §23.4 left it; Q46 asks whether it should open the menu as in the themed gamelist. The themed view's pad
menu has no *Game Options...*: Select is its route.

### 27.4 Tests

Headless through WiseMan; the scraping on stage (d)'s fake ScreenScraper, never the network.
- `DesktopGameOptionsTests` (6), `EmuSen.WiseMan/Mistress/`: a real right-click on a cover (pointer down and up at its
  centre) selects it and opens the grid's context menu, whose entries include Game Options... and Edit Metadata...;
  Game Options... opens the menu as an owned window with exactly the three entries and no rows; its Edit This Game's
  Metadata opens the editor as an owned window, and a saved name is the grid's and the list's label. A right-click on a
  list row, Edit Metadata..., a name and a description saved, shown in the list, the grid and big picture's gamelist;
  then a name typed on the on-screen keyboard in big picture's editor, shown on the desktop after F10. Ctrl+I opens the
  editor for the selected game, I alone does not, and neither does Ctrl+I in the search box. The built-in big screen's
  pad menu opens the menu and the editor as sheets, no window owned, the name typed on the on-screen keyboard. The
  desktop's pad menu has Game Options... (a window there) and the themed gamelist's has not. Ctrl+I over big picture's
  gamelist opens the selected game's editor as a sheet, and in the system view, after a game was chosen, nothing.
- `ThemedMetadataScrapeTests` gained three for Q18: the name offered after the editor's scrape and never in the field,
  every control reached by the pad with the offer shown, a Save storing no name, and the offer shown again on a later
  opening; the offer taken, shown as the player's edit with its Reset, saved and shown in the themed list and the library
  list; the offer declined over the player's own name, which stays in the box, in the draft's changes and in `games.db`.

### 27.5 Mutants

The runner is `~/.cache/emusen/probe/bigpicture/desktop-options/mutate_desktop_options.py`, adapted from §23.9's, with its
list `mutants-desktop-options.json` written by `mutants_desktop_options_make.py`, which checks that each edit's text occurs
exactly once. It restores any mutant a killed round left behind before it starts. Each mutant was built with `-m:2` and
run alone under `nice -n 10` against `DesktopGameOptionsTests`, `ThemedMetadataScrapeTests`, `ThemedGameOptionsTests` and
`GameMetadataTests`; after each, the file was restored from its copy, stamped with the present time and compared byte for
byte, and the tree was rebuilt at the end. Every verdict is appended to `mutants-desktop-options.txt`; the log is
`run-desktop-options.log`.

| Rule | Mutants | Result |
|---|---|---|
| Q18: offered, never forced | N1 the editor's scrape puts ScreenScraper's name in the field; N2 the name never offered; N3 Use This Name does nothing; N4 Keep Current Name takes the name; N5 the offer stays once the name is ScreenScraper's; N6 the offer's row never shown; N7 ScreenScraper's name made the name's baseline, shown with no edit | all caught; N1 by 5 tests, N7 by 6 |
| Q19: the routes | D1 no Game Options in the context menu; D2 no Edit Metadata in it; D3 the sidebar library's editor opens nothing; D9 no Game Options in the pad menu; D10 the themed view's pad menu offers it too | caught |
| Q19: what the menu holds and how it is shown | D4 the themed gamelist's Search in the desktop's menu; D5 the menu always a window, never a sheet | caught; D5 by 17, since the themed menu shares `PresentGameOptions` |
| Ctrl+I | D6 Ctrl+I opens nothing; D7 Ctrl+I taken from a text box being typed in; D8 I alone opens the editor; D13 Ctrl+I ignored over the themed gamelist | caught |
| One edit, every view | D11 the list shows the file's name; D12 the grid shows the file's name | caught; D11 by 5 |
| Ctrl+I in the system view | D14 the gamelist condition removed | **survived, and equivalent**: `ThemedLibrary.SelectedGame` is null outside a gamelist, so the condition could never decide anything. The condition was removed from the code rather than kept untested, and the test that returns to the system view and presses Ctrl+I was kept |

**20 of 21 caught, each on its first run; the survivor is an equivalent mutant.** D13 and D14 were a second round
(`mutants-desktop-options-2.json`), written after the Ctrl+I test over the themed gamelist was added. As §23.9 said of its
own round, a mutant written after its test by the same hand shows that the test is not empty, not that the rule list is
complete; P143 held (20 of 21, 95%).

### 27.6 Pictures

At 1280×800, written by `DesktopOptionsPictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to
`~/.cache/emusen/bigpicture/png/desktop-options/`. A desktop window is a separate top level, which the main window's
capture does not include, so the tool draws the window's own frame over the main window's, centred where
`CenterOwner` puts it; the context menu is drawn by the headless platform inside the window and needs no pasting. They
were looked at:
- `desktop-grid-context-menu`: the right-clicked cover ringed, and the menu beside it with Game Options... and Edit
  Metadata... (its Ctrl+I drawn at the right) between the collection entries and the cover art entries.
- `desktop-options-window`: the three entries and Close, the first focused, the grid behind.
- `desktop-editor-window`, `-edited`: the editor as a window; after typing, the Name row reads "Your edit." with its Reset.
- `desktop-grid-after-save`, then `bigpicture-shows-the-desktop-edit`: the new name on the placeholder and its title band, cut with an ellipsis, then in the
  themed gamelist after Big Picture from the View menu, cut at the theme's text list width, as any long name is there.
- `builtin-pad-menu`, `builtin-options-sheet`, `builtin-editor-sheet`: the pad menu over the built-in big-screen list,
  then the menu and the editor as sheets.
- `themed-editor-name-offered`: under the Name box, still the file's name and "From the file name.", the line
  *ScreenScraper calls this game "Aurora Drift (US title)".* and its two buttons; the status line under the fields says
  the name is offered and changes only if used. `themed-editor-name-taken`: the box holds the name, the row reads "Your
  edit." with Reset, and the offer is gone. `themed-gamelist-name-taken`: the gamelist after Save.
- **Seen and left:** on the desktop the options window, like the sheet, is titled by the game and holds three entries
  in a window sized to them; the sheet in the big screen takes the full height as every sheet does (§23.10).

### 27.7 Not done

- **Nothing ran on the handheld,** and no real pointer or window manager drove the desktop windows.
- **Ctrl+I cannot be rebound**; the hotkey map binds single keys (§4.3 of the settings reference).
- **Select in the sidebar library** is still the favourite (Q46).
- **Q30's switch** for scraped names library-wide is not built (27.2).
- **The desktop's menu has no rows** for sorting or filtering the sidebar library, which has no sort of its own.

### 27.8 The broad run

One broad run at the end, as the test-load rule asks: every test under `EmuSen.WiseMan.Mistress` except
`ShaderSettingsWindowTests`, `ShaderBrowseBench` and `SceneGpuBench`, under `nice -n 10`, no GPU test: **872 passed, 18
skipped (the picture and bench tools gated by their variables), none failed**, of 890, in 3 min 19 s. Before it, the
mutant runner's four classes passed 29 of 29 on the final code.

### 27.9 Open question

- **Q46, Select in the sidebar library.** In the themed gamelist Select opens the game options (§23.4, Q12) and North
  is the favourite (Q15). In the sidebar library, on the desktop's pad and in the built-in big screen, Select is still
  the favourite and the options are an entry of the pad menu. Make Select open the options there too, with the
  favourite as the menu's first entry, or keep it? **Recommendation: keep it**, since the built-in library is Mistress's
  own look (§23.4) and its help line already names Select as the favourite; the pad menu reaches the options in two
  presses.

---

## 28. Q40, Q42 and Q45 built: the developer credentials in published builds, OpenVGDB's Remove, and another game's cover (2026-09-27)

*Built on branch `scrape-embed-and-covers`.* The answers of 2026-09-26 (§10.1): Q40 (do what ES-DE does),
Q42 and Q45 from §26.9. The retirement §26 planned was withdrawn during the work (OpenVGDB kept as a
fallback, 2026-09-27), which changed Q42's wording and nothing else. The player's account is §4.65 of the settings
reference; this is the record.

### 28.1 Predictions, and what was found

Written during the build, by the hand that wrote the code, and weaker evidence for it (as §27.1 said of its own).

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P150 | `_IsPublishing`, which `dotnet publish` sets, is enough to tell a publish from a build before the compile | a plain `dotnet build` of the same project with the same property embeds nothing (test, and mutant E5) | held |
| P151 | A plain build after a publish in the same `obj/` would keep the resource unless the compile's input hash is told about it, so the target must add to `CoreCompileCache` | **wrong**: the SDK already hashes `@(_CoreCompileResourceInputs)` (`Microsoft.Common.CurrentVersion.targets`), so the resource's disappearance recompiles by itself. Mutant E8 removed the line and survived; the line was then removed from the target as redundant | retired |
| P152 | ReadyToRun at publish keeps a manifest resource unchanged | the published `lib/EmuSen/EmuSen.Mistress.dll` (3.4 MB, ReadyToRun, linux-x64, self-contained) carries the 120-byte resource and it decodes to the fake file's values | held |
| P153 | The picker's candidates can be the library's games whose `CoverPathFor` is non-null, with no new index | held in the tests; not measured on a 5,520-game library (§28.6) | held, untested at scale |
| P154 | Of the round's mutants, at least nine in ten are caught on their first run | 30 of 34 (88%); 28.4 | failed |

### 28.2 Q40: the embedding, measured

The target is `EmuSen.Mistress/Scraping/ScreenScraperDeveloper.targets`; the runtime side is
`DeveloperCredentials.Load`, `Embedded` and `Decode` in `ScreenScraperCredentials.cs`. §4.65.1 describes both.

**Choices made where the answer was silent.**
- *The key and the data in one resource.* ES-DE keeps its key in the same header as the scrambled strings; two
  resources would be no harder to read, and one keeps the format in one place.
- *A key per build, not a fixed one.* A fixed key would let the scrambled bytes themselves be searched for across
  builds; a fresh key costs nothing, since a publish recompiles anyway.
- *The file re-encoded, not embedded as it is.* Only the three fields are embedded, so a note or an extra field a
  developer keeps in the file never ships. The target reads the values with a regular expression rather than a JSON
  parser, because an inline task compiled by `RoslynCodeTaskFactory` has no guaranteed reference to `System.Text.Json`;
  the values are copied with their JSON escapes intact and parsed properly at run time.
- *The scrambled file deleted after the compile,* so the scrambled credentials exist on disk only inside the published
  assembly and, for the seconds of one compile, in `obj/`.
- *A warning, not an error, for a missing file*, as the plan required: a publish on a machine without the file (a
  contributor's, CI's) must succeed and produce a build that simply cannot scrape without a file of its own.

**The one publish of Mistress itself** (by hand, not a test; `~/.cache/emusen/probe/q40/publish-fake.sh`), with the
README's recipe for linux-x64 into scratch, a fake developer file whose password was random for the run, and
`XDG_CONFIG_HOME` pointed at an empty scratch folder so the real file could not be read: the publish succeeded; the
resource was present (120 bytes) and decoded to the fake values; the fake password was found nowhere in the published
tree, nor in the build log; no `screenscraper-developer.bin` was left under `obj/`. A first attempt without
`-p:ErrorOnDuplicatePublishOutputFiles=false` failed with NETSDK1152 on DianaOS's apphost, the known reason the recipe
carries that switch, and unrelated to this target.

### 28.3 Tests

Headless through WiseMan, never the network, and never the real developer file: every child process in
`ScreenScraperEmbedTests` runs with `XDG_CONFIG_HOME` in the test's scratch folder, and one test asserts that the
property's default resolves there before it publishes with no property at all.

- `ScreenScraperEmbedTests` (5) publishes a scratch project that imports the target, with a fake file whose id and
  password are random per run (the password holds a quote, an ampersand and a `<`): the resource is present and decodes to
  the file's values; neither value, nor its JSON- or URL-escaped spelling, appears in the assembly (searched as UTF-8 and
  UTF-16), in the build's output, or in any file the build left in `obj/` or `bin/`; a second publish uses a different key;
  a plain build never embeds, including a plain build after a publish in the same `obj/`; a publish naming a missing file
  succeeds with exactly one `EMUSEN0040`; with no property, the default is the (moved) user config directory, a publish
  there warns once and then, with a file placed there, embeds it; a file missing its password is not embedded and the
  warning names the field and not the id; the scrambled file's place under `obj/` is ignored by git.
- `ScrapeCredentialTests` (+9): the order (none; embedded; the per-user config file over it; the tree file over both; a broken tree
  file passed over); the embedded values redacted raw and URL-escaped, never printed, and never written by loading them,
  saving a member account or saving `appsettings.json`; six malformed blobs refused; the scrambled bytes holding neither
  value nor the word `devpassword`. `No_tracked_file_holds_a_real_devpassword` still reads the real file only to look
  for its password in tracked files; widening it to the id was tried and withdrawn, since the id is not a secret and
  is an ordinary word that tracked files contain.
- `ScrapeWindowTests` (+2, Q42): Remove declined deletes nothing; Remove accepted deletes the database with no request,
  turns the fallback off in the window, the switch and `appsettings.json`, keeps the fallback's covers, shows Download;
  a run then asks no server and downloads nothing; Download fetches it, turns the fallback on, and the next run finds the
  cover through the new database. A separate test runs Preferences and runs with the fallback off and on, and finds the
  database byte-identical afterwards.
- `CoverChoiceTests` (8, Q45): the order (the choice before the player's own art, covers only, the stamp changes); a
  chosen game on another console, a chain, a loop, a chosen game with no cover; the `games.db` row through Clear, Move
  of either game, removal and a choice of itself; the picker's ranking and search; the desktop's context menu to the
  picker as an owned window, the candidates, the grid, list and themed lookups, a second window reading the choice back,
  and the ROM and art folders' bytes and times unchanged; Use Its Own Cover from the context menu and from the picker;
  big picture by pad (the options' entry, the picker as a sheet, every control reached, the search typed on the
  on-screen keyboard, the drawn cover image of the themed gamelist following the choice and its undo); the built-in big
  screen's options by pad.
- The options menus' expected entries gained *Use Another Game's Cover...* in `DesktopGameOptionsTests`,
  `ThemedGameOptionsTests` and `ThemedCollectionsTests`, and three sentences about the missing developer file in
  `ScrapeWindowTests`, `ScrapeStatusWindowTests` and `ScrapeSignInTests` now name the build as well.

### 28.4 Mutants

The runner is `~/.cache/emusen/probe/q40/mutants.py`, its list `mutants-q40.json` written by `mutants_make.py`, which
checks that each edit's text occurs exactly once. It keeps the unmutated file beside itself while a mutant is applied,
restores it on start if a round was interrupted, compares it byte for byte after each mutant, and rebuilds the tree at
the end. Each `.cs` mutant was built with `-m:2`; each ran alone under `nice -n 10` against only its rule's tests.
Mutants of the `.targets` file need no rebuild, since the tests publish their scratch project against the file as it
is on disk. Verdicts are in `mutants.txt`, the run's log in `run-q40.log`.

| Rule | Mutants | Result |
|---|---|---|
| Q40: the order | E1 the embedded credentials never consulted; E2 ranked before the files | caught |
| Q40: never shown or logged | E3 the embedded password not registered with the redactor | caught |
| Q40: only what the target writes | E4 a blob of another version accepted | caught |
| Q40: only at publish | E5 a plain build embeds too | caught |
| Q40: scrambled | E6 the JSON embedded as it is | caught (the plain-text search, and the decoder) |
| Q40: nothing left behind | E7 the scrambled file left in `obj/` | caught |
| Q40: a plain build after a publish | E8 the target's `CoreCompileCache` item removed | **survived, and equivalent**: the SDK hashes `@(_CoreCompileResourceInputs)` itself (P151). The line was removed from the target rather than kept untested |
| Q40: a publish without the file | E9 no warning for a missing default file; E10 a file missing a field embedded anyway | caught |
| Q42: Remove is the player's, and switches the fallback off | R1 the window's setting left on; R3 no confirm; R5 the switch left showing on; R7 Remove offered with nothing to remove | caught |
| Q42: the old file let go of | R2 the fallback's worker kept, holding the removed file open | **survived the first round**: the served database was identical to the removed one, so a stale worker gave the same answers. The test now serves a database that knows one more game and asserts that game is found after Download; caught in the second round |
| Q42: Download | R4 the fallback left off after Download | caught |
| Q42: nothing else removes it | R6 the database deleted when the fallback is switched off | caught |
| Q45: first in the order, covers only | C1 the choice never looked up; C2 the player's own art ranked above it; C3 screenshots borrowed too | caught |
| Q45: chains and loops | C4 the loop guard dropped; C5 the chosen game looked up under this game's system | caught |
| Q45: kept, and moved | C6 Clear removes the choice; C7 a renamed chosen game loses its borrowers; C8 a game borrowing its own cover | caught |
| Q45: the picker | C9 the game itself offered; C10 a game borrowing from this one offered; C11 a game with no cover offered; C17 the nearest-by-name order dropped | caught; C17's first form (the ordering line deleted) did not compile and was rewritten as a constant key for the second round |
| Q45: undo and routes | C12 Use Its Own Cover does nothing; C13 the options never offer it; C15 no entry in the context menu | caught |
| Q45: the themed view | C14 the choice's stamp left out of the themed view's media key | **survived, and equivalent**: that key only clears the presence cache, whose own key already carries `MediaSources.Stamp`, which carries the choice; the drawn cover is looked up afresh on every showing (the test reads the drawn image). The stamp was removed from the key |
| Q45: no file touched | C16 a choice also copies the picture into the art folder | caught (the folders' fingerprint) |

**30 of 34 caught on the first run**, since C17's first form did not compile: the runner records a failed build as
caught, but it is no evidence and is not counted. **32 of 34 after the second round** (R2 with its sharper test, C17
rewritten), and the two survivors are equivalent mutants whose code was then removed. P154 **failed**: 30 of 34 is 88%,
under the nine in ten predicted; one of the four misses was a test too weak to tell two databases apart, one a mutant
that did not compile, and two were code that did nothing. As §27.5 said of its
own, a mutant written after its test by the same hand shows that the test is not empty, not that the list of rules is
complete.

### 28.5 Pictures

At 1280×800, written by `CoverChoicePictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to
`~/.cache/emusen/bigpicture/png/q40-q45/`; a desktop window is drawn over the main window where it opens, as §27.6's
tool does. They were looked at:

- `q42-openvgdb-row-downloaded`: the Scraping tab scrolled to its end; under the failover's hint, which now ends with
  "Removing OpenVGDB is optional and only frees its space", the switch on, "OpenVGDB is downloaded: … in home/Library",
  and Remove focused.
- `q42-openvgdb-row-removed`: after Remove, the switch off, "OpenVGDB is not downloaded. Download fetches it…", Download
  in Remove's place, and the message that the fallback is off until it is downloaded again. The synthetic database is
  20 KB, which is why the row reads in KB.
- `q45-desktop-context-menu`, `q45-desktop-picker`: the grid's context menu over the third game, then the picker as a
  window: the game's name, the sentence that nothing is copied or sent, the search box, and the four games with a cover,
  each with its picture; the first focused.
- `q45-desktop-grid-after`: the third game's tile shows the fourth game's cover, the status line says so, and no other
  tile changed.
- `q45-sheet-picker`, `q45-sheet-picker-searched`, `q45-themed-gamelist-after`: in big picture, the picker as a sheet
  with the pad's help line; after "dune" typed on the on-screen keyboard, one row; after A, the themed gamelist's cover
  image shows the fourth game's cover beside the third game's name.
- **Seen and left:** as a sheet the picker opens with the search box focused rather than the first game, because a
  sheet's `Opened` comes before its rows are laid out; A on the box opens the keyboard and the d-pad reaches the rows.
  The picker's thumbnails are square-fitted, as the rows are short.

### 28.6 The broad run

One broad run at the end, under `nice -n 10`: every test under `EmuSen.WiseMan.Mistress` except
`ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and anything named for the GPU or Vulkan: **913
passed, 21 skipped (the picture and bench tools gated by their variables), none failed**, of 934, in 3 min 31 s. The
theme-browser branch ran beside it on the same machine; nothing of its Themes tab was touched here.

### 28.7 Not done

- **Nothing ran on the handheld**, and no real pointer or window manager drove the desktop windows.
- **No real developer file was embedded** by any test, as the plan required; the only Mistress publish used a fake
  one. Whether ScreenScraper accepts the embedded credentials from a published build is therefore not measured, only
  that they decode to what the file held.
- **The picker at scale.** It walks the library's games and asks each for its cover when it opens; on the 5,520-game
  library that has not been timed.
- **Q42's buttons during a run** are disabled by the host's `OpenVgdbBusy`; no test holds a run open to press them.
- **Q43, Q44 and Q46** needed no code; Q44 no longer applies since OpenVGDB is kept (§10.1).

## 29. Pass 4 built: switches, and what the engine draws itself (2026-09-27)

*Opened and closed on 2026-09-27, on branch `bigpicture-pass4-switches` (from WiseMan at `55f7c28c`), with LunaP's
`pass4-badge-glyph` beside it.* §21.3's pass 4 under the answers of §10.1: every §21 recommendation accepted (Q22–Q35),
Q9's "the favourite, folder and badge graphics are Mistress's own drawings", and Q32's buttons (West left free for the
media viewer of pass 9; the shoulders jump ten games, built in §22.13; left and right give quick system select). The
settings reference's §4.66 is the player's account; this section is the record.

**Sources.** ES-DE's `USERGUIDE.md` ("UI settings", "System status settings", "Sound settings", "Other settings",
"Metadata editor") and `THEMES.md` ("badges", "clock", "systemstatus", "Navigation sounds"), stage (c)'s copies in
`~/.cache/emusen/bigpicture/motion/docs/`. ES-DE's source was not read and ES-DE was not run; nothing of ES-DE, its
images or its sounds entered either repository. Art Book Next was read in its reference clone only to see which badge
slots and images it names and where its clock is.

**Numbering.** §21.6 gave this pass P105 and P106. The plan set new predictions to start at P130, but §26 had taken
P130–P135 and another section P140–P143, so this section's are **P160–P163**; its questions start at **Q50**, as planned.

### 29.1 Predictions

P105 and P106 were written in §21.6 before any of this was built. P160–P163 were written by the hand that wrote the
code, after the code and its tests and before the mutants, the broad run and the pictures they concern, as §22.1's
were, and are weaker evidence for it.

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P105 | Built-in badges change no pixel of Art Book Next's gamelist, which names its own icons; on a synthetic theme that names none, each of the nine slots draws | Art Book Next's list drawn through the new entries against its files drawn as before: **0 pixels** differ at 1280×800; its four shown slots all carry its own files. The synthetic theme: nine entries in THEMES.md's order, each cell over 150 pixels changed, 45,836 inside the element and **0** outside | held |
| P106 | Turning the clock on changes only pixels inside the clock's box, as P43 found for the help bar | the synthetic theme's clock: 1,094 pixels inside its box, **0 outside**; turned off again, the frame equals the one before; Art Book Next's carousel clock 4,160 inside, 0 outside | held; on first run the test failed with 13,059 pixels outside, a defect of this pass's own code (§29.4) |
| P160 | Each of the four status switches and the help switch changes no pixel outside its element's box | the help bar 42,228 pixels inside its box and 0 outside; Bluetooth 397, Wi-Fi 992, the percentage 3,604 and the battery 3,229 inside, 0 outside; but the battery switch first failed by 44 pixels, eight columns right of the box, a LunaP defect older than this pass (§29.4) | held after the fix |
| P161 | Of the round's mutants, at least nine in ten are caught on their first run, none survives that is not equivalent | 60 of 63 caught on the first run (95%); the three survivors were weak tests, not equivalent mutants (§29.7) | **failed on its second clause** |
| P162 | Mistress's seven synthesised navigation sounds are each under 300 ms and pairwise distinct | 30–250 ms; seven distinct SHA-256 digests; peaks 0.22–0.31 | held |
| P163 | The broad Mistress run passes with no failure this branch causes | 983 passed, 27 skipped, none failed (§29.9) | held |
| P119 | The pass is opened and closed in no more calendar days than the lower end of its estimate (2 days) | opened and closed on 2026-09-27 | held |

### 29.2 What was built

**In LunaP** (branch `pass4-badge-glyph`, `docs/LunaP.md` §180), nothing of which knows ES-DE:
- `BadgeGlyph` with `BadgeKind` (favourite, completed, kids' game, broken, controller, alternative emulator,
  collection, folder, manual, folder link) and `ControllerGlyph` with `ControllerShape` (unknown, gamepad, NES, SNES,
  Nintendo 64): the toolkit's own geometry in one colour, every badge but the link on one plate;
- `BadgeStrip.Entries` of `BadgeEntry` records, each a file or the drawing, with a controller or a folder link drawn over
  it at a position and size of the badge, and a drawn link cut out of a drawn folder beneath it;
- `ScrollLetterOverlay`: a shade and the letters (or a star) of a list scrolled fast;
- a fix to `DeviceStatusBar`'s Wi-Fi fan, which drew past its box (§180.5 there; §29.4 here).

**In Mistress:**
- `Scene/SceneBadges.cs`: which slots a game shows, `all`'s documented order, each slot's entry (the theme's file,
  else the drawing), the controller type's shape. `IndicatorElements.Badges` maps the eight overlay properties
  (`controllerPos`, `controllerSize`, `customControllerIcon`, `controllerIconColor` and the four `folderLink…` ones),
  and `SceneMapping` claims them, each proved by `SceneMappingTests`.
- `SceneGame` gains `Controller`, `FolderLink` and `Manual`; `SceneData` gains `ShowHelp`, `StatusShown`, `ScrollOverlay`
  and `LiveClock`; `SceneSystem` gains `FavoritesOnTop`.
- `SceneView.ScrollLetters`, the overlay, placed over the view's canvas and shown from the held list's first repeat.
- `ThemedLibrary.Interface` (the settings), `QuickSelect` (the pair of buttons in force), the startup applied once, the
  status mask, the live clock, and `Sound`'s fallback to `NavigationSounds`.
- `SystemsOrder` (three orders), `NavigationSounds` (the synthesis), `InterfaceSettingsPane` (the Theme Settings sheet's
  Interface tab), Preferences' volume slider, and the metadata editor's **Controller** field, a new `Choice` kind of
  field stored in `games.db`'s `game_edit` as every edit is (§23.3).
- `Galaxia/Models/BigPictureInterface`: the settings, in `appsettings.json` (the player's configuration; nothing here is
  program data, so nothing is in SQLite).

**Elsewhere:** `UiSoundPlayer.Remember` takes samples already in the stream's format under a key, so the synthesised
sounds play through the same path as a theme's files. **In WiseMan:** `ThemedSession` gains the window's settings object,
a refresh, a fixed device status (`MainWindow.DeviceStatusSource`, which a test sets instead of reading this machine's
sysfs), a theme with no sounds, and extra system-view elements.

### 29.3 The rules, and where they are Mistress's

**Taken from ES-DE's documentation:** the nine slots and their meanings; a theme's image first, a built-in one otherwise;
the controller and folder-link overlays' properties and defaults; `all`'s order; the clock off by default; the help and
four status switches; the percentage beside the battery; quick system select's six choices and their meaning for a list
against a grid; startup system and view; the navigation volume's default of 70; sounds falling back per file; the overlay
off by default, darkening the view and showing two characters, or a star over favourites sorted on top.

**Mistress's own, because the documentation is silent and ES-DE was not run:**
- **The drawings**, all of them (Q9).
- **The overlay's timing**: from the held list's first repeat, when the list also fades the game's metadata out (ES-DE's
  measured list, §14.7), to its release or its end. Its shade is black at 0x60 alpha; its letters are 0.15 of the
  screen's height, in the default typeface, in the middle of the screen. §21.3 planned to record ES-DE's timing once;
  it was not recorded (Q51).
- **Startup applies once per session**, at the first showing. ES-DE's "on startup" is its own process start; Mistress's
  big picture can be entered and left in one process (§18), and applying it at each entry would lose the player's place.
- **The systems orders offered** are three (§4.66.1): ES-DE's two *HW type* orders rank hardware types in files it
  bundles, which were not read, and *Manufacturer, release year* equals *Release year* for five Nintendo systems. The
  default stays EmuSen's release order, not ES-DE's *Full names* (Q50).
- **The controller types offered** are EmuSen's three pads, a generic pad and unknown, of THEMES.md's thirty-six (Q54).
- **A volume of 0 plays nothing**, rather than a sound at no gain.

**A change of behaviour.** ES-DE's default quick system select, *Left/right or shoulders*, gives a grid's (and a
horizontal carousel's) shoulders the system, since their left and right move through the games. §22.13 had left a
grid's shoulders jumping ten items, recording ES-DE's rule as not built; under the default they now change the system,
and *Left/right* gives the grid its jump back (Q53). A text list, the case Q32 named, is unchanged: left and right change
the system and the shoulders jump ten.

### 29.4 Two defects the pixel tests found

**The help bar computed for the view being left.** The first run of the clock test found 13,059 pixels changed
outside the clock's box, all in the help bar. On entering a game list, `ThemedLibrary.Data()` is built before the view
changes, while the kept view is still the system view, and the help context's quick-select pair was computed for that
view: the system carousel moves left and right, so the game list's help bar was drawn as if it were a grid, with **Jump**
relabelled **System** and no left/right entry, until anything rebuilt the view. This was a defect of this pass's code,
not of earlier stages, and the switch tests caught it only because they compare a frame before a sheet with one after.
The context is now always the game list's (the help context is a game list's only). Mutant Q6 puts the defect back.

**The Wi-Fi fan drew past its box.** The battery switch's test failed by 44 pixels, eight to nine columns right of the
status element's box. With the battery off, the Wi-Fi fan became the last indicator, and LunaP's built-in fan (stage b,
`LunaP.md` §101.5) had arcs whose ends reached 0.66 of a height from the centre of a box one height wide. The defect was
shown on the unmodified LunaP by a new test (ink seven columns past the bar at a 40 px icon, in the two cases where the
fan is last) before it was fixed; `LunaP.md` §180.5 records it. It had been hidden because the fan had always had the
battery after it. Mutant L9 puts it back.

### 29.5 Tests

Headless through WiseMan, on `ThemedSession` and `PadDriver`, the synthetic theme and ROMs, and Art Book Next's
reference clone where the test is marked for it; never the network, never the GPU.

| Rule | Test |
|---|---|
| P106: the clock off by default; on, only its box changes; off again, the frame before | `ThemedSwitchesTests.The_clock_is_off_by_default_and_turning_it_on_changes_only_its_own_box`, `Art_Book_Next_s_clock_turned_on_changes_only_its_own_box` |
| A session's clock is live; a scene's is the time it is given | `A_session_s_clock_is_live_and_a_scene_s_clock_is_the_time_it_is_given` |
| The help switch: no help bar, nothing else changed | `Turning_the_help_off_removes_the_help_bar_and_changes_nothing_else` |
| P160: each status switch removes its indicator and changes nothing outside the element; the percentage goes with the battery | `Each_status_switch_takes_its_indicator_out_of_the_status_bar_alone` (4 cases) |
| Quick system select's six choices over a list: the pair that changes the system, what the others do, the help bar's words | `Quick_system_select_takes_the_pair_ES_DE_documents_for_a_list` (6) |
| The same over a grid, under the default and under *Left/right* | `ThemedGridPadTests.All_four_directions_move_the_grid_and_the_shoulders_change_the_system_or_page_by_the_quick_select` |
| Startup system and view; a missing system; a later showing keeps the player's place | `The_first_showing_opens_at_the_startup_system_and_view` (4) |
| The three orders, and the startup list follows them | `The_systems_follow_the_chosen_order_and_the_startup_list_with_them` (3) |
| The overlay: off by default; on, from the first repeat, letters, then a star over favourites on top, letters again, gone when let go; the shade darkens the list and the letters light the middle | `The_quick_scrolling_overlay_shows_while_a_list_is_held_only_when_turned_on` |
| The volume: the gain, the slider, saved, 0 plays nothing | `The_navigation_volume_is_the_stream_s_gain_and_zero_plays_nothing` |
| Mistress's sound for each one a theme lacks, per sound | `A_theme_without_a_sound_gets_Mistress_s_own_for_it_and_keeps_the_ones_it_has` |
| P162: the synthesis is short, distinct and the same every time | `The_fallback_sounds_are_synthesised_distinct_short_and_the_same_every_time` |
| The editor's controller drawn on the controller badge in its pad's shape | `The_controller_chosen_in_the_editor_is_drawn_on_the_controller_badge` |
| Every control of the Interface tab reached by the pad | `Every_control_of_the_interface_tab_is_reached_by_the_pad`; §16's `ThemeSettingsSheetTests` audit walks the new tab too |
| P105: the nine slots drawn in order, nothing outside the element | `BuiltInBadgesTests.On_a_theme_that_names_no_image_each_of_the_nine_slots_draws_and_nothing_outside_the_badges_changes` |
| A named image wins; a missing one falls back to the drawing | `A_theme_s_named_image_wins_and_a_missing_one_falls_back_to_the_drawing` |
| `all`'s order; each controller type's shape | `All_keeps_the_named_slots_first_and_each_controller_type_has_its_shape` |
| P105: Art Book Next unchanged | `Built_in_badges_change_no_pixel_of_Art_Book_Next_s_gamelist` |
| The eight overlay properties change pixels | `SceneMappingTests.Every_mapped_property_changes_the_rendered_pixels` (eight new cases on a new system of controller and linked-folder games) |

Changed: `SceneSemanticsTests.Badges_show_only_the_slots_the_game_has` reads the entries; `ThemedGameOptionsTests`'
reach test sets the new choice field so its Reset shows. In LunaP: `BadgeGlyphTests` (7),
`IndicatorControlTests.No_indicator_draws_past_the_bar_s_right_edge` (5), and the guards §180.4 lists; LunaP's whole
suite, 1,371 of 1,371.

The blast-radius run before the mutants, every test under `Mistress.BigPicture`, `GameMetadataTests`,
`PreferencesThemeTests`, `ControllersTests`, `DesktopGameOptionsTests` and `PadNavigationTests`: 421 passed, 16 skipped
(the picture, bench and live tools gated by their variables), none failed.

### 29.6 Rows 32 and 33: a game's own engine, and the play-time cap

The plan allowed these last, once everything else was built and tested; they were built after the first round of
mutants had run.

**Row 32, the per-game engine.** ES-DE's *Alternative emulators* choose an emulator per system in *Other settings* and
per game in the metadata editor, the game's winning; the `altemulator` badge and filter follow the per-game value alone
(UG "Other settings", "Metadata editor"). Mistress's per-system choice already existed as Graphics Settings' Engine row
(§4.44 of the settings reference), for the two consoles with two implementations. The editor gains **Alternative
emulator**, a `Choice` field whose values are the game's console's engines (`GameMetadata.ChoicesFor`), stored in
`game_edit` as `altemulator`; for the NES and SNES it offers only *None* and is disabled, as ES-DE greys the row. At
launch `MainWindow.GameEngine` reads the game's row from `games.db` itself, not from the library's snapshot, so a game
started before a refresh still finds it, and passes it to `CoreCatalog.EngineChosen` ahead of the console's choice. A
stored engine the console lacks is ignored (`GameMetadata.EngineFor`), as ES-DE launches with its default after an
invalid choice. **The test runs the window's own launch** on WiseMan's synthetic N64 system with Graphics Settings on
MarsRT and the game's row on Mars (C#), and reads the session's engine: Mars (C#); with no row, MarsRT; with a Game Boy
engine in the row, MarsRT. The game's frames were not run: what is asserted is the choice the session was built with,
since the engines' own tests (§4.44) cover what each then does.

**Row 33, the cap.** ES-DE's *Max play time tracking*, "from 1 to 23 hours … whatever play time is measured will be
ignored if it exceeds the selected value … 0 to disable play time tracking entirely and … 24 to have no limit". The
reading taken: a launch over the limit adds **nothing**, not the limit; that is what "ignored" says, and a player who
fell asleep with a game running has no better estimate to give. `PlayTime.Tracked` is the rule; `RecordPlayTime` applies
it with `AppSettings.MaxPlayTimeTracking`, default 8 (ES-DE's, §21.1); Preferences ▸ Gameplay sets it. The window's test
records a 45-minute launch, drops a nine-hour one, and records it after No limit is chosen, reading the clock through a
test hook (`PlayClockReading`) since a test cannot wait nine hours. The play count still counts every launch.

### 29.7 Mutants

The runner is `~/.cache/emusen/probe/pass4/mutate_pass4.py`. Before each mutant it writes a state file,
`mutant-in-progress.json`, holding the file's path and original text; it restores the file in a `finally` and removes the
state file after; a run that finds a state file at its start restores that file and rebuilds both trees before anything
else (checked with a planted state file, `check_restore.py`). Every mutant's text must occur exactly once. Each was built
with `-m:2` and run alone under `nice -n 10` against its rule's tests only; LunaP's ran in its own checkout against
`BadgeGlyphTests` and `IndicatorControlTests`. Both trees were rebuilt clean after each round. The log is
`run-pass4.log`, the verdicts `mutants-pass4.txt` and `mutants-pass4-rerun.txt`. No round was interrupted.

| Rule | Mutants | Result |
|---|---|---|
| Badges | B1 a theme's image ignored; B2 no built-in badge (the old rule); B3 `all` ignoring the theme's order; B4 the SNES type drawn as the NES pad; B5 no controller over its badge; B6 no folder link; B7 no controller badge; B8 the theme's controller image ignored; B9 `controllerSize` ignored; B10 the editor's controller never reaching the view; B11 the editor's choice not stored | caught |
| Clock, help, status | S1 the clock never drawn; S2 always drawn; S3 a session's clock fixed at its build; S4 the help switch ignored; S5 the status switches ignored; S6 the percentage kept with the battery off; S7 the Bluetooth switch ignored | caught |
| Quick system select | Q1 the default giving a list's shoulders the system; Q2 the triggers never; Q3 Disabled keeping left and right; Q4 the help keeping Jump; Q5 the help keeping left/right; Q6 the help context of the view being left (§29.4's defect); Q7 a grid's shoulders jumping under the default | caught |
| Startup, order | T1 startup ignored; T2 applied at every showing; T3 the view ignored; O1 full names ignored; O2 the Game Boy Color's year; O3 the setting never reaching the window | caught |
| Overlay | V1 shown although off | **survived**; caught after the test was changed |
| | V2 shown before the repeats; V3 no star; V4 left up after release | caught |
| Sounds, volume, sheet | N1 no fallback; N2 the fallback over the theme's own; N3 0 still playing; N4 the gain over 70; N5 the slider saving nothing; N6 the synthesis silent; N7 two sounds alike; I1 a choice waiting for the next showing | caught |
| LunaP §180 | L1 the link on a plate; L2 a controller over any badge; L3 the file not winning; L6 no knockout; L7 Icons counted first; L8 the overlay without its shade; L9 the Wi-Fi fan's old radius (§29.4); L10 the completed badge drawn as the favourite | caught |
| | L4 the overlay's size ignored; L5 its position ignored | **survived**; caught after the test was changed |
| Row 32 | R1 the game's engine never consulted at launch; R2 an engine of another console used; R3 no engines offered; R4 a one-engine console's row enabled; R5 no badge | caught |
| Row 33 | P1 the cap not applied; P2 Disabled recording; P3 the window ignoring the rule; P4 Preferences saving nothing; P5 the default No limit | caught |

**60 of 63 caught on the first run, 63 of 63 after two tests were strengthened.** The three survivors were weak tests,
none an equivalent mutant, which fails P161's second clause:
- **V1.** The overlay test held the list with the overlay off for 900 ms and looked then; by then the five-game list had
  reached its end, which fades the metadata back and so hides the overlay whatever the setting. It now looks at 600 ms
  too, while the list is still moving.
- **L4, L5.** The overlay test moved the controller up and shrank it in one strip and asked only that ink leave the
  middle and appear high up, which either change alone satisfies. Size and position are now asserted each on its own
  against the default: at size 1 the pad reaches left of the half-size box, and at a fifth of the height it sits high
  with the middle empty.

The row 32 and 33 mutants were written after their tests and all caught on their first run, which, as §23.9 said of its
own, shows the tests are not empty rather than that the rules are complete.

### 29.8 Pictures

At 1280×800, written by `Pass4PictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to `~/.cache/emusen/bigpicture/png/pass4/`, on
the synthetic theme with a fixed device (Bluetooth, Wi-Fi, 64% battery) and a clock frozen at 13:45, and on Art Book
Next's reference clone. Every one was looked at:

- `glyph-sheet`: the ten badges at 104 and 36 px on dark and at 48 px on light, the five controllers at 170 and 60 px, a
  strip of six entries (a controller on its plate, a linked folder), and the overlay. **A flaw seen and corrected:** the
  first sheet showed the folder link's white outline running into the folder's, so that neither read; the drawn folder
  is now cut out under a drawn link (`LunaP.md` §180.2).
- `synthetic-default-badges-clock-off`: a favourite marked completed, a kids' game, broken, and with the Super Nintendo
  pad chosen, so the badge strip shows five badges, the last the SNES pad on its plate; no clock; the status bar B, the
  fan (narrower since §29.4) and 64%.
- `synthetic-clock-on`: 13:45 at the theme's clock, nothing else moved.
- `synthetic-status-wifi-off`, `synthetic-status-wifi-and-percentage-off`: the fan gone, then the percentage; the bar
  closes up to the right edge.
- `synthetic-help-off`: no help bar.
- `synthetic-quick-select-shoulders-help`: the help bar with both shoulders reading SYSTEM and no left/right entry.
- `sheet-interface-tab`: the Interface tab, Quick System Select focused, its four dropdowns and the overlay switch in view,
  the rest below the fold.
- `sheet-preferences-volume`: the Appearance tab scrolled to the sounds, the switch and the volume slider at 70.
- `synthetic-overlay-held`, `synthetic-overlay-star`: "Co" in the middle of the dimmed view while the list passes Cobalt
  Harbor; then, with the first four games favourites, a star.
- `artbooknext-overlay-held`, `artbooknext-overlay-star`: the same on Art Book Next's list variant, where the list is
  centred, so the letters stand over the rows below the selection. **Seen and left:** the letters overlap the list in
  a centred layout; where ES-DE draws its overlay was not measured (Q51). The shade darkens an already dark theme only a
  little.
- `artbooknext-badges`: Art Book Next's own favourite and completed images, unchanged by this pass (P105). **Seen and
  left, and not this pass's:** in this variant, one of the list variants without a metadata panel, the theme gives its
  `badges` element no position, so the badges sit at the top-left corner with their upper half off the screen; they did
  the same before this pass, since the images are drawn as before (Q52).
- `artbooknext-system-clock-off`, `artbooknext-system-clock-on`: the carousel, then its clock at the top left, 13:45 in a
  rounded dark plate; its list variants set the game list's clock to `scope none`, so the game list shows none.

### 29.9 The broad run

WiseMan was merged into the branch before it (`b62a3d3e`: ES-DE's default keys steering the themed view, and F4 for the
Start menu), cleanly. Then one clean rebuild and one broad run, under `nice -n 10` with builds at `-m:2`: every test
under `EmuSen.WiseMan.Mistress` except `ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and anything
named for the GPU or Vulkan. **983 passed, 27 skipped (the picture, bench and live tools gated by their variables), none
failed**, of 1,010, in 3 min 46 s. P163 **held**. Another build was running on the machine at the same time. One pass is
weak evidence against an intermittent failure, as §15.14 says of its own. LunaP's whole suite, run twice during the pass,
passed 1,371 of 1,371 both times.

### 29.10 Not done

- **ES-DE was not run.** The overlay's timing, look and place, which §21.3 planned to record once from ES-DE, were not
  recorded: the plan took ES-DE's documents alone as the oracle, and the load rule of 2026-09-25 weighs against a
  recording session. Whether ES-DE's built-in badges sit and scale as Mistress's do is likewise unmeasured.
- **Nothing ran on the handheld**, and no real window: the navigation sounds were never heard, only their samples read.
- **The overlay for held shoulders.** ES-DE's guide shows it for held Up, Down, L1 and R1; the shoulders reach the view
  as repeated presses from `PadNavigator`, not as a held direction, so it shows for Up and Down only.
- **Systems orders by hardware type**, and ES-DE's custom sorting file; the startup list's collections.
- **The folder badge and link** are drawn for any entry that says so; nothing in Mistress marks a folder or a link yet
  (pass 6). **The manual badge** has no data until pass 9. *Amended at the merge with pass 6 (§30), 2026-09-27: pass 6's folder
  entries and folder links now carry the folder badge and its link; `ThemedSwitchesTests.Pass_6_s_folders_show_the_built_in_folder_badge_and_a_linked_folder_its_link`
  shows both on the NES region folders, and mutant B6, rewritten for pass 6's `FolderLink` (a file name, not a flag), is
  caught by it and two others. The merge's narrow run (big picture, folders, engine, metadata, Preferences): 462 passed,
  22 skipped, none failed.*
- **The controller field** offers EmuSen's three pads, a generic pad and unknown (Q54); ScreenScraper gives no controller.
- **Filters** for controller and alternative emulator in Filter Gamelist (§22.11) are still not offered.
- **The status bar's own drawings** besides the Wi-Fi fan were checked only by the tests that exercise them here.
- **Row 32's filter and notice**: Filter Gamelist has no alternative-emulator field, and the editor shows no notice for
  a stored engine its console lacks; the launch simply ignores it.

### 29.11 Open questions

- **Q50, the systems' default order.** The default stays EmuSen's release order (NES, Game Boy, Game Boy Color, SNES,
  Nintendo 64). ES-DE's default is by full name, which §22.2 measured as Nintendo 64, NES, Game Boy, Game Boy Color, SNES
  under ES-DE's own full names; Mistress's *Full names* sorts the names its themes show and gives Game Boy first. Keep
  EmuSen's order as the default, or switch to *Full names*?
- **Q51, the overlay's timing and look.** §21.3 planned to record ES-DE's quick-scrolling overlay once; it was not
  recorded (§29.10). Should a short ES-DE session (in the scratch home, with the uinput rig of §14.7) measure when it
  appears, how dark it is and where its letters sit, so that Mistress's can follow?
- **Q52, Art Book Next's badges in its list variants without a metadata panel.** The theme gives the element no
  position there, so its badges sit at the top-left corner, half off the screen, as they did before this pass. Is that
  what ES-DE does with the same theme (to be checked in the same session as Q51), or should Mistress leave an element
  without a position undrawn?
- **Q53, a grid's shoulders.** Under ES-DE's default quick system select a grid's shoulders now change the system, as its
  guide documents; before this pass they jumped ten games. Keep ES-DE's default, or default to *Left/right* so a grid
  keeps its jump (and a list is unchanged either way)?
- **Q54, the controller types offered.** The editor offers EmuSen's three pads, a generic pad and unknown. THEMES.md names
  thirty-six types. Offer all of them (Mistress drawing any pad it has no shape for as the generic pad), or keep the list
  to EmuSen's consoles?

---

## 30. Pass 6 built: folders (2026-09-27)

*Built on branch `bigpicture-pass6-folders`, from WiseMan at `55f7c28c`.* §21.3's Pass 6, under the answers of
2026-09-26 (§10.1): Q23 (c), folders shown as ES-DE shows them by default with a per-console flatten switch, and Q24,
directories named like files and `.m3u` playlists left until a disc-based core exists. The player's account is §4.67 of
the settings reference; this is the record. §29 is another pass's.

**Numbering.** The plan numbered this section's predictions from P140, but §27 had taken P140–P143 and §28 P150–P154.
They are therefore **P144–P149**, and the questions **Q60–Q67**.

**Sources.** ES-DE's `USERGUIDE.md` (*Multiple game files installation*, *Directories interpreted as files*, *Folder
flattening*, *Manually copying game media files*, *General navigation*, *UI settings*, *Gamelist options menu*,
*Metadata editor*) and `THEMES.md` (`grid` and `carousel`'s `defaultFolderImage`, `textlist`'s `indicators` and
`secondaryColor`, `badges`' `folder` slot and folder-link overlay, `gamelistinfo`), the copies of §21 under
`~/.cache/emusen/bigpicture/motion/docs/`. ES-DE's source was not read and ES-DE was not run for this section; where the
guide is silent, §30.3 says what Mistress chose. The ROM library at `AppSettings.RomDirectory` was listed, never opened
or written (§30.6).

### 30.1 P108: the defect shown, then fixed

P108 predicted that `EsdeMediaFolder` finds none of the media of a game in a subfolder of an ES-DE-written tree. The
test `ThemedFoldersTests.P108_media_of_a_game_in_a_folder_are_found_under_that_folder_in_an_ES_DE_tree_and_in_Mistress_s_store`
was written first and run on the unchanged reader, through the window's own `MediaSourcesNow().Locate`: an ES-DE tree
holding a cover, a screenshot, a marquee and a video of `NES/USA/Tidal Keep` under `nes/<type>/USA/`, and Mistress's
store holding a cover and a screenshot of `NES/Europe/Quartz Mill` under `nes/<type>/Europe/`. **None of the six was
found**; every `Locate` answered none. The test was committed skipped with that result as its reason (`5175e909`), as
§16.2 recorded P52, and the fix (`b8abd28f`) removed the skip. **P108 held.**

The fix is in three places. `GameFolders.Of` gives a game's folder (§30.2). `SceneGame` carries it as `FolderPath`, and
`EsdeMediaFolder` names a game's files `<folder>/<stem>` and then `<stem>` (Q61). `MediaSources`, which the library's
covers call with a bare ROM path, derives the folder from the ROM folder it is now given. The presence listing of §16.2,
which tells the variant triggers which media a system has, lists a type folder's own folders when a game has one, and
the stamp that tells a showing that media changed now includes those folders' write times, since a file added to
`covers/USA/` changes `covers/USA`'s time and not `covers`'.

### 30.2 A game's folder, and the rule that was rejected

ES-DE's folders are relative to `ROMs/<system>/`. Mistress has no system folder: a game's console is its extension
(§4.46 of the settings reference), and the player's console folders are named `NES`, `GB`, `N64`, `SNES`, not ES-DE's
`nes` and the rest. Two rules were considered:

- **Per shelf:** the longest directory all of a console's games share, capped at one level below the ROM folder. It
  adapts to a ROM folder pointed at one console's folder, but a game's folder then depends on every other game: one ROM
  dropped at the top of the ROM folder would move the whole NES library's folders from `USA` to `NES/USA`, and with them
  every stored picture's path.
- **Per file (built):** the first level below the ROM folder is the console's folder, whatever its name, and what lies
  between it and the file is the game's folder. It reads one path, so no file changes another's folder, and it is what
  ES-DE's own layout means when that first level is `nes`.

The rule's one misreading: a ROM folder set to `Roms/NES` makes `USA` the console folder and shows the NES games flat.
It was judged the lesser fault, since ES-DE itself would not recognise `USA` as a system there. A game directly in the
ROM folder, or outside it, has no folder.

### 30.3 What was built, and where the guide is silent

**The view** (`ThemedLibrary.Folders.cs`, a new part of `ThemedLibrary`): a regular system shows folders unless the
player flattened it and once any of its games has a folder. The list at a folder is built from the games the system
keeps (searched and filtered over the whole system, as USERGUIDE's filter section requires) as the child folders that
hold a kept game and the games directly there; folders on top in the list's sort, then games with favourites first.
`_path` holds each system's folder; the selection is keyed by system and folder (`ViewKey`), while sorts and filters
stay keyed by system (`ListKey`), since USERGUIDE says the sort "can be set individually per game system". South
enters a folder, or launches its linked game; East climbs one level. `SceneSystem.Counted` gives the system view every
game to count. The grouped collections of §22 keep their own one-level folder, `_openFolder`, beside this.

**The window** (`MainWindow.BigPictureFolders.cs`): each game's `FolderPath`, each console's `Flatten`, the folder links
from `games.db`, a folder's two menu entries and its sheet, `FolderEditorWindow`. **Settings**: `FoldersOnTop` and
`FlattenedSystems` in `BigPictureCollections`, with their rows in Game Collection Settings (§4.67.3 of the settings
reference says why there). **LunaP**: nothing. `TextRowList` already had `TextRowMarker.Folder` (LunaP §101.8), which
the scene has drawn for grouped collections since stage (b), so no LunaP branch was made.

**The store** (§30.4) and the scraper, which writes `<system>/<type>/<folder>/<stem>` and follows a file moved to
another folder the way it followed a rename.

**Choices where the documentation is silent**, none measured against ES-DE:
- **Folders above favourites.** With both sorted on top, folders come first, then favourite games, then the others.
- **The folder's sort.** Folders are sorted by the list's sort key among themselves; having no metadata, they fall to
  the name under every key but the name's.
- **Each system remembers its folder** across quick system select and across a visit to the system view (Q66).
- **A search that empties the folder shown** moves the list to the nearest folder above that still holds a match,
  rather than leaving an empty list.
- **An empty folder**, or one holding only files no core reads, is not listed: the scan knows only game files.
- **The random entry** picks any entry, folders included (Q67).
- **Jump To…'s folder entry** is the word *Folders* (Q62).
- **The help bar's A** reads *Select* on a folder and *Launch* on a linked folder; USERGUIDE does not say.
- **A folder's picture** is `<system>/<type>/<parent folders>/<folder name>.<ext>`, the name whole, dots included;
  THEMES.md says `physicalName` strips a folder's last dot, and whether media names do was not established.

### 30.4 The store's one layout step

`media.db`'s second migration adds `store_move (from_path, to_path, how, at)` and `store_step (name, done_at)`.
`MediaStore.PutInFolders` runs when Mistress opens the store and knows the ROM folder, and never again once
`store_step` holds `folders`. For each `scrape_media` row naming a flat file, it finds the files `scrape_file` last
hashed to that game with the same stem, and their folders:

| Case | What happens |
|---|---|
| one game, in one folder | the file is **moved** into the folder, and the row follows |
| copies in several folders | **copied** into each, the last folder (in ordinal order) taking the move; the row names the first |
| a copy at the top as well | **copied** into each folder, the flat file kept for the one at the top; the row stays flat |
| a file already at the target | **left** as it is, logged `found`; the row names the file the game now shows |
| a game at the top only, a file with no row | not touched |

Moving was chosen over copying everything because the store is ES-DE's layout by design (§5.6, §17.5): a player can
point ES-DE at it, and in a foldered layout the flat copy would be a second file no reader asks for first. The plan's
rule, that the player's media are never deleted, is held by the test's comparison of every file's SHA-256 before and
after (`Flat_pictures_of_foldered_games_go_into_their_folders_once_and_no_picture_is_lost`), which also runs the step a
second time and on a reopened file and requires nothing to change. A step cut off after a move and before its row is
finished at the next opening (`A_file_of_the_first_schema_is_migrated_in_place…`, which also migrates a schema-1 file
in place with its rows kept). Mistress's own opening of a flat store, and Clear inside a folder afterwards, are
`Mistress_puts_its_own_store_in_folders_when_it_opens_it_and_the_picture_is_still_found`.

The player's own store was not migrated by this work: no test or tool opened `~/.local/share` or wherever the running
Mistress keeps `home/Media`. It will be migrated the first time the merged build opens it.

### 30.5 Tests

Headless in WiseMan; the synthetic library of folders is written by `ThemedFoldersTests.Library` into the session's
temporary ROM folder beside its own top-level games (`ThemedSession` gained a `roms` hook for it).

| Rule | Test |
|---|---|
| P108: an ES-DE tree's and the store's foldered media found | `ThemedFoldersTests.P108_…` |
| NES opens on its folders on top, marked, with Select; South in, East out one level, each level's place kept, two levels deep | `…NES_opens_on_its_folders_on_top_South_enters_East_leaves_and_each_level_keeps_its_place` |
| GB's letter folders; the system view's `Counted` | `…GB_shows_its_letter_folders_and_the_system_view_counts_every_game_inside_them` |
| The count text from `Counted` | `…The_system_view_counts_the_games_inside_folders_not_the_entries` |
| All games flat | `…All_games_lists_the_games_inside_folders_flat` |
| P109's return, with P40's frame comparison | `…A_game_started_inside_a_folder_comes_back_to_that_folder_and_game` |
| Quick system select keeps each system's folder and game | `…Quick_system_select_keeps_each_system_s_folder_and_game` |
| Jump To's folder entry and letters; filter values from every file; one sort for all folders; the selection following its entry; a favourites filter hiding folders; a search walking up | `…Sorting_filters_the_search_and_the_jump_behave_within_folders_as_USERGUIDE_describes` |
| Folders not on top; a flattened console | `…Folders_are_not_on_top_when_the_setting_is_off_and_a_flattened_console_lists_all_its_games` |
| The sheet's switches applied at once and saved | `…The_settings_sheet_flattens_a_console_at_once_and_keeps_it_in_appsettings` |
| The folder menu's entries; the folder sheet reached by the pad; a link saved to `games.db`, launching, Enter Folder, disabled while editing | `…A_folder_link_set_in_the_folder_s_editor_launches_its_game_and_Enter_Folder_still_opens_it` |
| A link to a file the library no longer lists is no link | `…A_folder_link_to_a_file_that_has_gone_is_ignored` |
| A folder's own picture; the flat name second; presence from one listing equal to presence asked game by game; the stamp | `…A_folder_s_picture_is_named_after_it_and_presence_from_one_listing_equals_presence_asked_game_by_game` |
| The store's step, the migration, the store's first opening, Clear | `MediaStoreFoldersTests` (3 facts, 2 theories), `ThemedFoldersTests.Mistress_puts_its_own_store_in_folders…` |
| The scraper writes under the folder; a moved file's media follow; a written file's kind | `ScraperTests.A_game_in_a_folder_keeps_its_media_under_that_folder_and_moved_to_another_folder_takes_them_with_it` |

Ten of the eleven first view tests passed at their first run. The one that failed was the test's error: it assumed the
selection moved to the first row after a sort, where the view keeps it on its entry, which the test now asserts.

### 30.6 P109 on the tester's library, read-only

`FoldersLibraryTool` (`EMUSEN_BIGPICTURE_REALLIB=1`) scans `AppSettings.RomDirectory` with `RomLibrary.Scan`, which
lists files and opens none, builds the shelves as the window does, and shows them in a `ThemedLibrary` on the synthetic
theme with no media. Before and after the run the library's file count (5,615) and byte total (1,910,413,892) were
equal and `find -newer` listed nothing. The output is `~/.cache/emusen/probe/pass6/real-library.txt`.

| Console | Games | Opens on | Deepest folder | File stems in two folders |
|---|---|---|---|---|
| NES | 3,537 | 16 folders (Australia, Canada, Country_Unk, Europe, Hacks, PC10, PD, Pirate, Sweden, Trained Hacks, Translated, Unclassified, Unlicensed, USA, Versus, World) | 1 | 0 |
| GB | 1,898 | 27 folders (0-9, A–Z) | 1 | 0 |
| GBC | 17 | 4 folders (A, G, P, S: its games sit in GB's letter folders) | 1 | 0 |
| SNES | 48 | 48 games | 0 | 0 |
| N64 | 20 | 20 games | 0 | 0 |

The first Show of a new view, median of five: **1.6 ms foldered, 3.2 ms flattened** (ratio 0.49); a Show of a shown view
0.6 against 2.0 ms. Debug build, headless, no window: the scene is built, not laid out or drawn.

### 30.7 Mutants

The runner is `~/.cache/emusen/probe/pass6/mutate_pass6.py`. Before each mutant it writes a state file
(`mutant-state.json`: the mutant, the file, the original's SHA-256) and keeps the original beside it; on start, a state
file left by an interrupted round has its original put back, after its hash is checked, before anything else is built.
Each mutant was built at `-m:2` and its tests run alone under `nice -n 10`, against `ThemedFoldersTests`,
`MediaStoreFoldersTests`, `ScraperTests`, `MediaStoreTests`, `GamelistOptionsTests`, `ThemedCollectionsTests`,
`ThemedLibraryPadTests`, `ThemedLibraryTriggerTests` and `ThemedGameOptionsTests`; the tree was rebuilt clean after each
round, and `git status` showed no source file changed. The log is `run-pass6.log`, the verdicts
`mutants-pass6.txt` and `mutants-pass6-rerun.txt`. No round was interrupted.

| Rule | Mutants | Result |
|---|---|---|
| P108 and the reader | F1 a foldered game looked up by its stem alone; F2 no flat name after the folder's; F3 a folder looked up as a file; F5 the stamp without a type folder's own folders; F6 the desktop's lookup gives a ROM no folder; F7 the window passes no ROM folder | caught; F1 by three tests, the P108 test among them |
| Presence | F4 presence lists only a type folder's top | **survived the first round**: the test's every present type also had a file at the top. The test now adds a type found only under `screenshots/USA/`; caught in the second round |
| A game's folder | F8 the console folder kept in it; F9 a child found by a bare prefix (`USA2` under `USA`); F10 a nested folder's path on disk one level too deep | caught; F8 by 13 |
| The store | S1 a scrape writes flat; S2 a moved file leaves its media; S3 a file's kind read from its parent folder; S4 the step at every opening; S5 a flat picture a top-level game shares moved away; S6 a picture at the target overwritten; S7 rows left flat; S8 a row a top copy shares pointed into a folder; S9 the step not recorded; S10 Mistress never runs it; S11 Clear in the flat type folder; S12 the window's Clear given no folder | caught; S6 by the exception `File.Move` raises rather than by an assertion |
| The view | V1 no folders; V2 the flatten switch ignored; V3 collections with folders; V4 folders never on top; V5 folders by name whatever the sort; V6 a folder with no kept game shown; V7 an emptied folder not left; V8 one place for every level; V9 the selection per system; V10 quick system select forgets the folders; V11 East in a folder goes to the systems; V12 East leaves every level; V13 no sound on entering | caught |
| Folder links | V14 never launches; V15 launches while a collection is edited; V16 no Enter Folder; V17 links not given to the view; V18 saved as none; V19 a folder's menu offers a game's entries; V20 the help bar's Select on a linked folder; L1 a link to a gone file kept | caught; L1 was written in the second round with its test, after reading §4.67.4's sentence against the code showed the help bar would say *Launch* over a link that no longer launches |
| Jump To and the count | V21 no folder entry; V22 folders indexed beside it; V23 the system view counts the entries; V24 no count given | caught |
| Settings | V25 the flatten switch stores nothing; V26 the window ignores it; V27 Sort folders on top ignored; V28 off by default | caught |

**49 of 50 caught on the first run; 51 of 51 after the second round** (F4 with its stronger test, and L1). No mutant
failed to build, and none was equivalent. As §28.4 said of its own round, every mutant was written after its test by the
same hand, so this shows the tests are not empty rather than that the rules are complete; F4's survival is the kind of
gap it does find.

### 30.8 Predictions retired

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P108 | `EsdeMediaFolder` finds none of a foldered game's media in an ES-DE tree, shown on the unchanged reader | 0 of 6 found on the unchanged reader; 6 of 6 after the fix | **held**, then fixed |
| P109 | Shown as folders, NES opens on 16 entries and GB on 28; the return from a game in a folder comes back to that folder and game, its first frame equal to a fresh build; the first showing costs within 10% of the flat one | NES 16; **GB 27**: `GB/[BIOS]` holds four `.7z` archives, which no core reads, so the scan lists nothing there and the folder does not show; the return held, 0 pixels differing at 1280×800; the first showing **half** the flat one's cost, 1.6 against 3.2 ms, since the text list builds a row per entry, 16 rather than 3,537 | **failed** on two clauses: the count, because §21.1 counted folders and not the files in them; the cost, which was predicted as a ceiling and came out far under it |
| P144 | *(written during the build)* The layout step loses no picture in any of its cases, and a second run changes nothing | the SHA-256 comparison in every case of §30.4, and the rerun | held |
| P145 | *(written during the build)* No file stem of the tester's library repeats between two folders of one console, so the flat name second (Q61) never shows one game another's picture there | 0 on each console (§30.6), as §21.1 counted | held |
| P146 | *(written before the round)* Of the round's mutants, at least nine in ten are caught on their first run | 49 of 50 (98%) | held |
| P147 | *(written before the broad run)* The broad Mistress run passes with no failure this branch causes | once, after merging WiseMan at `b62a3d3e` and a rebuild: the Mistress filter without `ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` or any test named for the GPU or Vulkan, under `nice -n 10`: 998 tests, 971 passed, 27 skipped (the picture, bench, live and real-library tools, which need their variables), none failed, in 3 min 50 s | held; one pass is weak evidence against an intermittent failure, as §15.14 says |

### 30.9 Pictures

At 1280×800, written by `FoldersPictureTool` with `EMUSEN_BIGPICTURE_PNG=1` to `~/.cache/emusen/bigpicture/png/pass6/`,
on Art Book Next with synthetic covers stored under their folders, and one on the synthetic theme. The library is the
user's shapes in small: eight NES folders by region and category, eight GB letter folders, two games in each. They were
looked at:
- `nes-opening-on-its-folders`: seven folder rows with Art Book Next's own folder icon before each, *Europe* selected,
  the theme's folder badge in the badge row, no cover (the folder has none and Art Book Next's image has no default),
  and the help bar's *Select*.
- `nes-inside-Europe`: the folder's two games, the first's cover from `nes/covers/Europe/`, *Launch* on the help bar.
- `gb-letter-folders`: `0-9`, then A to F, *B* selected.
- `nes-flattened`: every NES game in one list after the switch, the cover still found under its folder.
- `settings-sheet-nes-flattened`: the Game Lists section with *Sort folders on top of gamelists* on and *Flatten
  Nintendo Entertainment System folders* switched on by the pad, the focus on it.
- `folder-options-menu`: Jump To reading *Folders*, the sort, the filter, Search and *Edit This Folder's Metadata*.
- `folder-editor-with-a-link`: the folder's name, its path from the ROM folder, and the link chosen.
- `synthetic-nes-folders-marked`: LunaP's folder marks in the textlist's secondary colour, then the two top-level games.

**Seen and left:** the synthetic theme's help bar names North *Favorite* on a folder, where North does nothing (§30.10).
The first pictures' GB games were named from their letters and read oddly ("Blpha Beacon"); the fixture's names were
changed and the pictures redrawn.

### 30.10 Not done

- **Directories as files and `.m3u`** (Q24): not built.
- **A folder's other metadata**, scraping a folder, and hiding one (Q63).
- **A folder-link mark in the text list**: LunaP has none, and this pass was not to add LunaP controls beyond a folder
  mark, which existed (Q60).
- **`gamelistinfo`'s folder icon**: the element is not drawn (Pass 14).
- **ES-DE was not run**, so every choice of §30.3 is unmeasured; Q61, Q65, Q66 and Q67 ask for its behaviour.
- **The player's own `media.db`** was not migrated here (§30.4).
- **North and the help bar on a folder** (Q65).
- **The handheld**: nothing ran there.
- **The flat name second** also applies to Mistress's store after its migration, where only hand-placed pictures and
  what the step could not move stay flat.

### 30.11 Open questions

- **Q60, the folder-link mark.** ES-DE marks a linked folder in a text list with its own symbol (`>` in *ascii*). Add a
  `TextRowMarker.FolderLink` to LunaP, or keep the folder mark and leave the link to the badge?
- **Q61, the flat name second.** A foldered game's picture is looked for under its folder, then at the flat name, for an
  ES-DE folder written from a flattened system. Where ES-DE writes media for a system with `flatten.txt` is not in its
  guide; a capture from ES-DE would settle it. Keep the fallback, or look under the folder only?
- **Q62, Jump To's folder entry.** The word *Folders*, or a LunaP glyph like ES-DE's icon?
- **Q63, a folder's metadata.** ES-DE lets a folder have a name, description, rating, hidden flag and the rest, and be
  scraped. Build them, or keep the link alone?
- **Q64, where the switches are.** The plan named Theme Settings or Preferences; they were put in Game Collection
  Settings' Game Lists, beside the other ES-DE game-list settings §22 put there (§4.67.3). Move them?
- **Q65, North on a folder.** USERGUIDE says folders can be marked as favourites. Mistress does nothing on North there
  and its help bar still says *Favorite*. Should North favourite a folder, or the help bar hide the entry?
- **Q66, the folder kept across a visit to the system view.** Mistress re-enters a system at the folder it was left in;
  whether ES-DE does was not measured. Keep, or start at the top?
- **Q67, the random entry.** It may pick a folder. Should it pick games only, as USERGUIDE's "jumps to a random game"
  reads?

---

## 31. Q47: the loader against ES-DE (2026-09-27)

*Opened 2026-09-27, on the answer to Q47 of that day, (b) (§10.1): run ES-DE on the themes the loader refuses
(P123, P124), then match what it does, rule by rule.* The principle is §25.10's: a theme ES-DE draws should not be one
Mistress refuses, and a theme ES-DE refuses should not be one Mistress draws. While the work ran, Q47 turned up in play on
the handheld: the theme browser refused to install Artflix (Revisited) and CarAlt, because the install gate
(`ThemeDownloads.Validate`) is the loader. The player's account is §4.68 of the settings reference.

**Sources.** ES-DE 3.4.1's behaviour, measured by running it, and its `THEMES.md` where it speaks; never its source.
Every real theme was downloaded whole into `~/.cache/emusen/bigpicture/q47-themes/` and deleted afterwards; the
synthetic probe themes were written under `~/.cache/emusen/probe/q47/probe-themes/`. No theme file, image or XML
entered the repository; the tests write their own XML.

**Numbering.** Pass 4 took P160–P163, so this section's predictions start at **P170**; its questions start at **Q70**.

### 31.1 The setup

- **ES-DE.** The copy of the downloaded AppImage under `~/.cache/emusen/bigpicture/esde/`, with a home of its own,
  `home-q47/`, beside the earlier stages' home, which was not touched. Each run: `--home home-q47 --resolution 1280 800
  --fullscreen-padding off --no-update-check --no-splash --debug`, windowed, one window at a time, 8–15 s, closed by
  PID, and the absence of any ES-DE process checked after each (`ps` with an anchored pattern, never `pgrep -f`).
- **What it could reach.** `ApplicationUpdaterFrequency` was `never` and the ScreenScraper account fields were empty;
  nothing asked it to scrape. Its ROM folder was `esde/q47-roms/`: four empty files per system for `nes`, `snes`, `gb`,
  `gbc` and `n64`, and an SNES folder *Tower Set* with a folder link (for the folder-link icon), with one game marked
  *hide metadata* (for `metadataElement`). The real library was never named. `SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT`
  admitted only §22.2's uinput pad, so a real controller could send nothing.
- **The runner** is `~/.cache/emusen/probe/q47/esde_run.py` (settings, launch, window captures by `ffmpeg -f x11grab`,
  pad timelines, log copy, close). 36 runs in all; every `es_log.txt` is kept in `probe/q47/logs/`.
- **Three kinds of run.** Each of the 15 refused themes as downloaded (its log, and a capture of the SNES game list);
  X20s's second variant, at 16:10 and 4:3; and synthetic probe themes, one property per text or badge, each labelled
  with the value it carries, so one capture reads many values at once. A value that might refuse the whole system went
  into a variant of its own (`q47-risky-es-de`), one run per variant, so a refusal could not hide the others.

### 31.2 What ES-DE does, rule by rule

**All 15 themes load in ES-DE.** Every run logged `Finished loading theme` with no `Error:` line, and every capture
shows the SNES game list themed. The six rules, and what ES-DE was measured to do beyond each theme's own value:

| # | §25.8's rule (themes) | ES-DE 3.4.1, measured | The rule it follows, as far as measured |
|---|---|---|---|
| 1 | `folderLinkSize` a pair, `0.85 0.9` (7) | loads, no log line; the link icon is pixel-identical to `0.85`'s and smaller than `0.9`'s | a FLOAT is its leading number, after any leading whitespace: `0.9abc` is 0.9, `abc` is 0 and then clamped (the icon at its minimum). Also measured on `fontSize` (`0.05 0.01` and `0.05abc` drawn as `0.05`), so it is the type's rule, not the property's |
| 2 | an unescaped `&` in `capabilities.xml` (4, but see §31.5) | loads; Razor's colour scheme reads **GAME & WATCH** in UI Settings | a `&` that does not begin a complete reference is text: `A & B`, `C&D`, `E &foo; F` and `L &amp M` are shown as written; `&amp;` and `&#74;` are decoded (`G & H`, `I J K`). The same holds in a theme file (`Q & A` drawn) |
| 3 | `gamelistinfo` `size` `w 0.02` (2) | loads, no log line; the box is laid out as `0 0.02`'s | a NORMALIZED_PAIR is split at its **first space** and each side read as a FLOAT: `w 0.5` is `0 0.5`, `0.5 w` is `0.5 0`, `0.75 0.3 0.9` is `0.75 0.3`, `0.6  0.5` is `0.6 0.5`, and a leading space shifts it, ` 0.02 0.62` being `0 0.02` (pixel positions in `p03`, `p14`). With **no space** the system is unthemed: `w0.02`, `0.6`, `0.75,0.6`, and a pair separated only by a tab or a newline |
| 4 | `<transitions>` directly in `<theme>` (Grimmlex, X20s) | loads, no log line; directly in `theme.xml` it is **ignored** (the first declared profile plays, a slide); at the top of a file **included from a `<variant>`** it applies to that variant (a fade, as when written in the variant) | recordings at 30 fps of one game-list-to-game-list step on four probe themes (`p17-*`) |
| 5 | `visible` `no` (1) | loads, no log line; the element is hidden | a BOOLEAN is true when its **first character** is `t`, `T`, `y`, `Y` or `1`, false otherwise, and never an error: shown for `true`, `TRUE`, `True`, `t`, `tru`, `truex`, `yes`, `Yes`, `YES`, `y`, `yup`, `yes please`, `1`, `1.0`, `true `; hidden for `false`, `False`, `flase`, `no`, `0`, `2`, `-1`, `on`, ` true `, ` 1`. A value of only whitespace unthemes the system, as an empty one does |
| 6 | `metadataElement` `flase` (1) | loads, no log line; the text stays shown on a game marked *hide metadata*, as with `false` | the same BOOLEAN rule: `true`, `yes`, `TRUE` and `1` hide it there; `flase`, `false` and `no` do not |

**And a form §25.8 had not named.** X20s's `variant_02.xml` begins with a stray `f` before its first comment. ES-DE
reads the file: at 4:3 the second variant draws its own logo geometry (0.32 and 0.13×0.14, against the first variant's
0.29 and 0.1×0.13). Probe files with text before the root, and after it, were both read. **Mismatched tags still refuse
the file**, and with it the system:

```
Error:  ThemeData::parseIncludes(): ".../q47-risky-es-de/theme.xml" -> "./broken.xml": Error parsing file: Start-end tags mismatch (system "snes", theme "snes")
Error:  ThemeData::parseElement(): ".../q47-risky-es-de/theme.xml": Invalid normalized pair value "w0.02" for property "size" (system "snes", theme "snes")
Error:  ThemeData::parseElement(): ".../q47-risky-es-de/theme.xml": Property "visible" for element "text" has no value defined (system "snes", theme "snes")
```

These refusals were the only lines ES-DE logged about the six rules; every lenient reading was silent. `THEMES.md` says
as much for numbers ("commonly just clamped to the allowable range without notifying the theme author"). Its
"sanitization for valid data format" (§12.3) turns out to mean a pair with no space, not a pair, float or boolean that
reads oddly.

**The argument, separated from the mechanism.** The table states rules that fit every value tried; it makes no claim
about ES-DE's implementation. The FLOAT rule behaves like C's `atof` on each value, and the pair rule like a split at
the first space followed by two such reads, but whether ES-DE calls those functions was not, and under §25's rule could
not be, looked at. Where the loader extrapolates beyond the values tried, §31.9 says so (P170, P172).

**Negative results on method.** A risky-variant probe with two spaces in a pair drew nothing, which first read as a
refusal. A second probe drew the same pair where the rule puts it, and a third showed the first probe's text lacked a
`fontSize`: ES-DE drew no text without one, whatever its position (`p15`). A first capture of X20s's two variants at
16:10 was pixel-identical, since the variants differ only at 4:3 and in media the scratch folder does not hold; it said
nothing about whether the stray `f` was read, and the 4:3 pair was needed.

### 31.3 P123 and P124 retired

| # | Predicted (§25.9) | Measured | Verdict |
|---|---|---|---|
| P123 | ES-DE themes CarAlt for `snes` although its `badges` set `folderLinkSize` to a pair, drawing the icon at the pair's first number | CarAlt themed with no error; in the probe, `0.85 0.9` draws a link icon pixel-identical to `0.85`'s, and `0.9`'s differs | **held** |
| P124 | ES-DE loads Razor, whose `capabilities.xml` carries a bare `&`, and lists its `Game & Watch` label | Razor loaded; UI Settings shows the colour scheme **GAME & WATCH** | **held** |

Both were written as the lenient reading. The alternative §25.8 left open, that the list carries themes ES-DE cannot
load, is the one that failed, for all 15.

### 31.4 What changed in the loader

- **`ThemeXml`** (new) reads every theme file and `capabilities.xml`: strictly first, and only if that fails, again,
  with each `&` that does not begin one of XML's five named references or a numeric one written as text, any XML
  declaration blanked, and the document read as a fragment whose first element is the root, so text before and after it
  is ignored. What a strict reader refuses inside the root, such as mismatched tags, is refused as before. A file read
  the second way carries a **warning**, `LenientXml`, naming what was tolerated.
- **`ThemeValueParser`** gained `EsdeFloat`, `EsdePair` and `EsdeBool`, the three rules of §31.2. `ThemeViewBuilder`
  applies them to the property's text **as written** (after variable substitution, untrimmed, since ES-DE does not trim:
  ` true ` is false). A value read otherwise than as a plain value carries a **warning**, `LenientValue`, saying what it
  was read as: "`<folderLinkSize>` of badges "b" is "0.85 0.9", not a plain number; it is read as 0.85, as ES-DE reads
  it". A pair with no space stays an **error**, and its message now says why ("which has no space between two values;
  ES-DE refuses such a pair too").
- **`<transitions>` directly in `<theme>`** is ignored with a **warning** (`IgnoredTag`) in a file not reached through a
  variant, and applied, as if written in the variant, at the top of a file included from one (a depth counter in
  `ParseRun`). Everywhere else it is refused as before.
- **The install gate** follows without a change of its own: `Validate` asks the loader, and every leniency is a warning,
  so a download ES-DE draws now installs. §31.6's install test goes through `FakeThemeHosts`, as the browser does.
- **What did not change.** COLOR and UNSIGNED_INTEGER stay strict (not measured; Q70). `capabilities.xml`'s own booleans
  (`selectable`) keep their strict reading, with a warning and a fallback (not measured; P172).

§12.3's rows on malformed XML and on "a value in the wrong format" are superseded by this section for the forms above.
They are kept, with a pointer, as the record of what stage (a) took from the document.

### 31.5 A correction to §25.8

§25.8 counted X20s among the four themes with "a malformed `capabilities.xml`" (an unescaped `&`). X20s's
`capabilities.xml` is well formed; its malformed file is `variant_02.xml`, whose stray leading `f` is what the survey's
sample line reports ("Data at the root level is invalid. Line 1, position 1"). The row should have read "a malformed
XML file", three themes for the `&` and one for text before the root. §31.2 covers both; no count changes.

### 31.6 Tests

`ThemeEsdeRulesTests`, 42 cases, all on synthetic XML written by the tests:

| Rule | Test |
|---|---|
| a FLOAT is its leading number, with a warning when read otherwise | `A_float_is_its_leading_number` (5 values, `folderLinkSize`) |
| a pair split at its first space, each side a FLOAT | `A_pair_is_split_at_its_first_space_and_each_side_read_as_a_float` (7) |
| no space unthemes, and says why | `A_pair_with_no_space_unthemes_the_system_as_in_ES_DE` (`w0.02`, `0.6`, tab, newline, comma) |
| BOOLEAN by its first character, for `visible` and `metadataElement` alike | `A_boolean_is_true_when_its_first_character_is_t_y_or_1` (19 values, each measured in §31.2) |
| a bare `&` kept and references decoded, in capabilities and theme files | `A_bare_ampersand_is_kept_as_text_and_references_are_decoded` |
| text before and after the root ignored | `Text_outside_the_root_element_is_ignored` |
| mismatched tags still refuse | `Mismatched_tags_still_untheme_the_system` |
| `<transitions>` in `<theme>` ignored; from a variant's include, applied | `Transitions_directly_in_theme_are_ignored_with_a_warning`, `Transitions_at_the_top_of_a_file_included_from_a_variant_apply_to_that_variant` |
| **install**: each of the six forms, and X20s's stray text with a variant's included transitions, installs through `FakeThemeHosts`; `w0.02` and mismatched tags are refused with the reason, and leave no folder | `A_download_installs_where_ES_DE_draws_the_theme_and_is_refused_where_it_does_not` |

`ThemeErrorTests` lost three rows of its wrong-format theory (`0.5 0.5 0.5`, `yes` for a boolean, `ninety` for a
rotation), which ES-DE reads; its malformed-XML case, which used `a && b`, now uses mismatched tags. `FakeTheme` gained
`CapabilitiesText` and `ThemeText`, so a fake download can carry any theme. After the change the theme classes (147
tests) and the whole `Mistress.BigPicture` namespace (423: 404 passed, 19 skipped tools) passed.

### 31.7 The survey again

`EMUSEN_THEME_SURVEY=analyse`, on the XML §25.8 fetched (the earlier `survey.json` kept as `survey-before-q47.json`):

| | §25.8 | now |
|---|---|---|
| themes themed for all five systems | 51 of 66 | **66 of 66** |
| themes with no loader error | 46 | **63** |
| errors, by code | `BadFormat` 15, `MalformedXml` 4, `MisplacedTag` 2, `UndefinedVariable` 1 | `BadFormat` 2, `UndefinedVariable` 1 |
| new warnings | | `LenientValue` 15, `LenientXml` 4, `IgnoredTag` 1 |

The three themes still logging an error are themed but lose a variant's element: Aura (an undefined `${glass-size}`),
and Canvas and Iconic (`itemsBeforeCenter` and `itemsAfterCenter` of `3.5`, an UNSIGNED_INTEGER; Q70, P173). Cathode
and DEcaffe, which §25.8 listed among the five themed with errors, now have none: their errors were among the six forms.
The analysis took 3.7 s for 1,990 loads, against 3.2 s; the strict parse is tried first, so only the files that need
the second reading pay for it.

### 31.8 Mutants

The runner is `~/.cache/emusen/probe/q47/mutate_q47.py`, its log `run-q47.log` and its verdicts `mutants-q47.txt`.
Each mutant was built with `-m:2` and tested alone under `nice -n 10`, against `ThemeEsdeRulesTests`,
`ThemeErrorTests`, `ThemeLoaderTests` and `ThemeCapabilitiesTests`. Before changing a file the runner writes it to a
state file, restores it in a `finally`, and on starting restores any file a cut-short run left mutated; the tree was
rebuilt clean at the end.

**19 mutants: 18 caught, 1 survived, none failed to build.**

| Area | Mutants (caught unless marked) |
|---|---|
| BOOLEAN | M1 `y`/`Y` not true; M2 the text trimmed first |
| FLOAT | M3 trailing text makes it 0; M4 leading whitespace not skipped; **M18 no exponent (survived)** |
| NORMALIZED_PAIR | M5 split at the last space; M6 any whitespace separates; M7 trimmed before the split; M8 a pair with no space only warned |
| Severity | M9 a lenient value an error |
| XML | M10 a bare `&` not escaped; M11 any named reference kept as a reference; M12 text outside the root refused; M13 a malformed root read as empty; M17 `capabilities.xml` read strictly; M19 theme files read strictly |
| `<transitions>` | M14 applied wherever it stands; M15 the variant depth never counted; M16 ignored placement an error |

- **M18 survived** because no test writes an exponent, and none was written deliberately: whether ES-DE reads `1e-1`
  as 0.1 was not measured (P170), and a test would pin an assumption as if it were a measurement. The loader keeps the
  exponent, as a C reader of a number would; if P170 fails, the fix is one line and M18 becomes the test's mutant.
- **The install test** caught M10, M12, M16, M17 and M19 alongside the rule tests, so the install path is held by its
  own test for the XML and placement rules, not only through the loader's.

### 31.9 Predictions

| # | Prediction | Retired when |
|---|---|---|
| P170 | ES-DE reads a FLOAT written with an exponent (`1e-1`, `5E-2`) as its value, as the loader does; no listed theme was seen to write one | ES-DE is run on it |
| P171 | Installing Artflix (Revisited) and CarAlt from the theme browser on the handheld with this build succeeds, and big picture draws both | the next install there |
| P172 | ES-DE reads `selectable` in `capabilities.xml` by §31.2's first-character rule (`yes` selectable, `flase` not), where the loader warns and keeps its default | ES-DE is run on it |
| P173 | ES-DE draws Canvas's and Iconic's game-list carousels with `itemsBeforeCenter` of `3.5`, reading it as 3 | ES-DE is run on them (Q70) |

P170, P172 and P173 were retired in §35.4: P170 and P173 held, and P172 failed as a rule.

### 31.10 Not done

- **Only the SNES game list was captured** for the 15 real themes. Their logs cover all five systems and the
  collections, and none logged an error, but the other systems' views and every system view went unseen.
- **No surveyed theme was rendered by Mistress** after the change; the survey counts loads, as §25.8 did.
- **COLOR, UNSIGNED_INTEGER, STRING and PATH** values were not probed (Q70), nor `capabilities.xml`'s own values
  (P172), languages, or `<transitions>` inside `<aspectRatio>` or `<language>`.
- **Malformed text after the root** (an unclosed tag after `</theme>`) is tolerated by the loader; ES-DE was shown only
  plain text there. **A second root element** after the first is ignored by the loader; not measured.
- **Numbers written as `inf`, `nan` or in hexadecimal**, which a C reader might take, are 0 in the loader; not measured.
- **The handheld**: nothing ran there (P171).
- **The value as reported.** The handheld's message gave the value as `w0.02`; Artflix (Revisited) as downloaded on
  2026-09-27 writes `w 0.02` in all three places. The settings reference's §4.70, merged while this section was written,
  records that the detail sheet's status line had been broken by word-wrap, so which spelling the handheld showed could
  not be read from it; `w 0.02`, the downloaded spelling, is the one this section's install test and the survey load. A
  theme that does write `w0.02` is refused by ES-DE and by the loader alike.

### 31.11 Open questions

- **Q70, the other value types.** COLOR, UNSIGNED_INTEGER (Canvas and Iconic write `3.5`), STRING and PATH were left
  strict, as THEMES.md types them. One probe run per type would settle each as §31.2 settled three. **Recommendation:**
  with pass 14, since two themes are affected and both still load.
- **Q71, a variant's `selectable` default.** In the ampersand probe, ES-DE showed **NONE DEFINED** for Theme Variant,
  although the probe declared one variant with no `<selectable>`. Either ES-DE's default is false, or it lists nothing
  for a lone variant; §12.4 chose true, since THEMES.md is silent. **Recommendation:** measure it (two variants, one
  without `<selectable>`) before changing anything.
- **Q72, the warnings.** The lenient readings are warnings in the loader's log only, and ES-DE says nothing at all.
  Should Theme Settings' About sheet list a theme's warnings, for theme authors, or stay silent as ES-DE does?
  **Recommendation:** silent, as now.

### 31.12 The broad run

After merging WiseMan (pass 4 and the error log of settings §4.70), one run of the Mistress filter without
`ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any GPU or Vulkan test, under `nice -n 10`:
**1,083 tests, 1,051 passed, 30 skipped (the picture, survey and live tools), 2 failed, in 3 min 46 s.** Neither
failure is this branch's:
- `ScrapeCredentialTests.No_tracked_file_holds_a_real_devpassword` names `EmuSen.WiseMan/Galaxia/ErrorLogTests.cs`,
  which WiseMan's error-log commit added with a literal `devpassword=` to test its redaction; this branch does not touch
  the file. It fails on WiseMan's own tree for the same reason, by construction; that was not run separately.
- `InputSettingsWindowRenderTests.The_window_renders_its_rows(NES)` threw from Avalonia's headless platform
  initialisation ("The calling thread cannot access this object"), and passed, all three cases, when run alone; it is
  recorded as an order-dependent failure of the headless setup, not investigated further here.
  *Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78: the headless setup failed because `GridSceneTests`'
  seven plain facts built scenes off the session's dispatcher at the same moment (§4.78.2 reproduces the message from
  their own scene); fixed in `b272675c`. Two more classes had the same fault without causing this and were fixed
  (§4.78.3).*

## 32. ES-DE's menu look, stage 1: the Start menu and a game's options (2026-09-27)

*Built on branch `bigpicture-esde-menus`, from WiseMan at `9c8304a5`; LunaP on `esde-menus`, from `openemu-library` at
`1bd782b`.* Requested 2026-09-27: make the big picture options menu look like ES-DE's.
It is built in two stages, and the first stage's look is approved before the second begins. The player's account is
§4.69 of the settings reference; LunaP's record of its pieces is its §181. §29–§31 are other passes'.

**Numbering.** Predictions from P180, questions from Q80; both ranges were free in the tree when this began.

**Sources.** ES-DE's look was read from the captures of 2026-09-26 under
`~/.cache/emusen/bigpicture/collections-probe/runs/` (the main menu `r7/m00_mainmenu.png`, the gamelist options
`r5/f04_options.png`, Game Collection Settings and a selection list `r7/m02`, `r7/m03`). ES-DE was not run for this
section, and its source was not read. No file of ES-DE's was copied: no font, image, sound or colour table. The
measurements below are of those pictures, to set proportions; the colours are Mistress's own, near ES-DE's greys but not
sampled into the code.

### 32.1 What ES-DE's menu measures, at 1280×800

Read off `m00_mainmenu.png` with a pixel scan (not ES-DE's code):

- the panel runs from x 220 to 1060 (840 wide, 0.656 of the width) and y 104 to 716, with corners of about 16 px;
- the title band is 100 px, its capitals 49 px high;
- rows are 54 px apart with a 1 px rule, their capitals 26 px high, text 7 px from the left edge;
- the chosen row is a black bar between two rules, edge to edge;
- the chevron is 15 by 26 px, 8 px from the right edge;
- the footer band is about 78 px;
- the help bar sits at the bottom centre on a dark fill, its icons 22 px and its capitals 18 px.

Mistress lays its menu out in the same proportions, scaled from 800 lines. The panel is 0.66 of the width but no wider
than 1.05 times the height, which gives 840 px at 1280×800. The title is 68 px Barlow Condensed and the rows 36 px
(their capitals about 26 px). The rows are 54 px; the help text is 26 px. It differs on purpose in three places. The
chosen row's text turns near white on the bar, where ES-DE keeps it grey (Q82). The theme's own help bar is hidden, so
one help bar shows. The help bar's buttons are LunaP's pad glyphs (§29) in the pad's family, not ES-DE's icons.

### 32.2 What was built

- **LunaP** (`esde-menus`): `MenuPanel`, `MenuRow`, `MenuRowKind`, `MenuRowLayout`, `MenuRows`, `BlurBackdrop` and
  `SheetLayer.Chromeless` (its §181). Nothing existing changed its behaviour.
- **Mistress**: the pad menu's list moves into a `MenuPanel` in a big-screen session. `PadMenuEntry` gains `Label`,
  `Value` and `Opens`. `GameOptionsWindow` builds an ES-DE panel when given a pad family, and `GameOption` gains
  `MenuRow` and `Opens`. `GamelistOptionRows` gives Jump To…, Sort Games By and Filter Gamelist their row forms. The
  screen is wrapped in `ScreenContent` and blurred by `MenuBackdrop`. The theme's help bar is hidden while a menu is open.
  Barlow Condensed ships in `Assets/Fonts` (§4.69.2, `THIRD_PARTY_NOTICES.md` §1.5).

The desktop is unchanged: `ApplyBigMenuLook(false)` puts the list back in its bordered box with its own rows, and a
game's options on the desktop are built as before. A test checks both (§32.4).

### 32.3 Predictions

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P180 | *(written after the first pictures, before the tests)* The Start menu and a game's options draw nothing outside the panel and the help bar but the backdrop, at 1280×800 and 1920×1200 | 0 differing pixels outside at both sizes, in both menus | held |
| P181 | *(written during the build)* No existing test needs a behavioural expectation changed; only expectations about how something is drawn | two changed, both about the look (the title read from the panel; *1 filter set* read from the row's value); 92 pad and desktop tests and 167 themed tests passed unchanged | held |
| P182 | *(written before the mutants)* Of twelve mutants, at least nine in ten are caught on their first valid run | 12 of 12 | held, but see §32.5: the first round was not valid |
| P183 | *(written before the broad run)* The broad Mistress run passes with no failure this branch causes | 1,039 tests, 1,007 passed, 32 skipped, none failed (§32.7) | held; one pass is weak evidence against an intermittent failure, as §15.14 says |

### 32.4 Tests

`EsdeMenusTests`, six cases, with the detail in §4.69.5:

- the Start menu at both sizes;
- a game's options at both sizes;
- F4 and Backspace with no pad;
- the desktop kept.

The centring and the bar are read from the controls' arranged rectangles and from the pixels. The pixel rule is §15's
comparison between two frames: the menu open, and the same frame with the panel alone hidden. It counts differing
pixels inside the panel and help bar and outside them, and the outside count must be zero. LunaP's `MenuTests` adds ten
cases (its §181.6).

### 32.5 Mutants, and a round that was not valid

Six mutants of Mistress and LunaP ran against `EsdeMenusTests`:

1. the panel placed at a third of the width;
2. the bar 0.9 of the row;
3. the title drawn 40 px above the panel;
4. the theme's help bar left shown;
5. the desktop given the big look;
6. no blur.

Six more of LunaP ran against its `MenuTests` (its §181.6).

**The first round of the six was not valid**, and its results are not counted. The runner restored each file from a copy
made with `shutil.copy`, which gives the copy the time it was made, earlier than the mutant's build. MSBuild therefore
saw the restored file as older than the assembly and did not rebuild it. Every mutant after the first ran on the first
mutant's binary: all reported "caught", and the "theme help left shown" mutant failed on the centring assertion, which
it does not touch. That mismatch is what exposed the problem. It is the trap memory records for the mutation runner. The
runner now touches each restored file and touches every file it mutated before the final rebuild. On the valid round,
each of the twelve was caught by the assertion written for it, and nothing else failed.

### 32.6 Pictures

In `~/.cache/emusen/bigpicture/png/esde-menus/`, from `EsdeMenusPictureTool` (`EMUSEN_BIGPICTURE_PNG=1`). For each of
`synthetic` and `artbooknext`, at each of `1280x800` and `1920x1200`, the pictures are:

- `-start-menu` and `-start-menu-fourth-row`;
- `-game-options`;
- `-game-options-sort-stepped`;
- `-in-game-menu`;
- `-folder-options`.

Two side-by-sides compare with ES-DE's captures: `side-by-side-main-menu.png` against `m00_mainmenu.png`, and
`side-by-side-game-options.png` against `f04_options.png`, both Art Book Next at 1280×800. Every picture was looked at.

### 32.7 The broad run

Once, on the final build of both branches (LunaP `c07279a`), under `nice -n 10` and `-m:2`, headless. The filter was the
Mistress tests without any test named for shaders, the GPU or Vulkan:
`FullyQualifiedName~EmuSen.WiseMan.Mistress&FullyQualifiedName!~Shader&FullyQualifiedName!~Gpu&FullyQualifiedName!~Vulkan`.

- **1,039 tests: 1,007 passed, 32 skipped, none failed**, in 3.6 minutes.
- The skipped are the picture, bench, live and real-library tools, which need their variables; this section's two
  picture tools are among them.
- LunaP's own suite ran in full on its branch: 1,388 tests, 1,387 passed. The one failure was the README's stale count of tests (its §181.6); it was corrected and that test rerun, and passed.

### 32.8 Stage 2, not started

*Begun 2026-09-27, after stage 1's look was approved: item 1, the metadata editor, is built and recorded in §34;
the others wait for its look to be approved. The heading is kept as it was written.*

It waits for the approval of stage 1's look. In the order requested (2026-09-27):

1. **The metadata editor first**, as ES-DE's editor:
   - an upper-case *Edit Metadata* title with the game's name and file name beneath;
   - one row per field, label left and value right;
   - text fields opening an edit popup;
   - the rating as stars, flags as switches, and choices with `<` `>`;
   - the buttons in a row at the bottom.

   The source is USERGUIDE.md's *Metadata editor* section (the local copy under `~/.cache/emusen/bigpicture/motion/docs/`).
   Two fields it lacks today, each stored in `games.db`'s `game_edit` like the rest:
   - *Hide metadata fields*, a flag that hides most fields and the badges in the themed view;
   - *Custom collections sortname*, shown only when the editor is opened inside a custom collection, which sorts custom
     collections only.

   *Hide from Library…* stays in place of ES-DE's Delete, since Mistress never deletes a ROM.
2. **The settings sheets as ES-DE menus**: Theme Settings with pass 4's Interface tab, Game Collection Settings, and
   Preferences in a big-screen session. Each has label-and-value rows, `<` `>` option rows, switches, and chevrons into
   submenus.
3. **An option row's list screen**, in place of the stock dropdown.

`MenuRow`'s `Switch` kind is drawn but not yet driven, and a text popup and a stars row are new. The estimate, which is a
guess from this stage's pace and not a measurement:
- the metadata editor with its two fields and their behaviour: about one working session;
- the three settings sheets, which hold several dozen settings between them: one to two more;
- the pictures, mutants and records: throughout.

### 32.9 Not done

- The settings sheets and the metadata editor (§32.8).
- A real pad, the handheld, and ES-DE running beside Mistress for a live comparison.
- The desktop's pad menu, deliberately.

*Since §45 (2026-10-04) the pad menu is reorganised into pages, on the desktop and in big picture alike. The rows of
this stage are kept in their look and regrouped, and the version moved to the EmuSen submenu's footer.*

### 32.10 Open questions

*All seven were answered on 2026-09-27, with stage 1's look approved; the answers are in §10.1 and were built as §32.11
records. The questions are kept as they were asked.*

- **Q80, the Start menu's title over the library.** *Main Menu*, as ES-DE, is what it shows now; the desktop's says
  *EmuSen*. Keep *Main Menu*?
- **Q81, the in-game title.** The running game's file name with its extension (*Cobalt Harbor (Synthetic).sfc*), as the
  desktop's menu shows it. Show the library's title instead?
- **Q82, the chosen row's text.** Near white on the bar here; ES-DE keeps it grey. Keep the brighter text, or match?
- **Q83, the typeface.** Barlow Condensed Regular (§4.69.2). Keep it, or try Roboto Condensed or Fira Sans Condensed
  side by side first?
- **Q84, the footer.** *EmuSen 1.0.0* under the pad menu, where ES-DE shows its version. Keep, or drop it?
- **Q85, a game's options' title.** The game's name, as before; ES-DE titles the same menu *Gamelist Options*. Which?
- **Q86, A on an option row.** It drops down the stock list today. Build ES-DE's list screen in stage 2 (§32.8, item 3),
  or leave A to step as Right does?

### 32.11 The answers built (2026-09-27)

- **Q81.** Over a running game the Start menu's title is the name the library shows: the themed view's (the player's
  edit over ScreenScraper's name over the file's name) when the library is a theme, else the sidebar's (the player's edit
  over the file's name), with no extension or folder in either. `EsdeMenusTests` checks it against the name the themed
  gamelist showed for the game it launched. The desktop's menu still shows the file's name; it was not asked about.
- **Q85.** A game's options, and a folder's, are titled *Gamelist Options*. `ThemedGameOptionsTests` and `EsdeMenusTests`
  read it; the game's name no longer appears in the menu.
- **Q84, the version.** The 1.0.0 was nobody's decision: no project set a version, so the .NET SDK's default, 1.0.0, was
  every assembly's, and `MainWindow.BuildName` reads `EmuSen.dll`'s informational version (`1.0.0+<commit>`). A new
  `Directory.Build.props` at the repository root sets `Version` to 0.9.0. Every project that does not set its own now
  carries 0.9.0: EmuSen, Mistress, Hotaru, Pharaoh, Serenity, Endymion, DianaOS, WiseMan.
  - EmuSen.Galaxia and EmuSen.Cauldron keep their own 0.1.0.
  - LunaP comes from its own repository and is not affected.
  - The Rust cores' versions are Cargo's and are not affected.

  What reads it, found by searching the tree for the version attributes, `GetName().Version`, `FileVersionInfo` and
  the MSBuild version properties:
  - the menus' footer, now *EmuSen 0.9.0*;
  - the covers' HTTP User-Agent (`EmuSen/0.9.0`, was `EmuSen/1.0.0`);
  - the *Build* a new save state's record stores (`0.9.0+<commit>`), shown in the media view and in the load-state
    messages. Only the state version decides whether a state loads, so an older record that says 1.0.0 is shown as
    it is and still loads.

  Nothing in the publish layout reads the version. ScreenScraper's `softname` is unchanged. The file and product
  version of the published executables now say 0.9.0.
- `EsdeMenusTests` asserts the footer reads *EmuSen 0.9.0*. The affected pictures of §32.6 were rendered again and
  looked at, and the two side-by-sides were composed again.

## 33. Pass 11 built: the launch screen (2026-09-27)

*Built on branch `bigpicture-pass11-launch-screen`, from WiseMan at `b6b85a69`.* §21.3 planned Pass 11 as ES-DE's launch
screen and its five durations (inventory row 10), after the resume question and before the game's first frame. Q33 was
decided on 2026-09-26 (§10.1): Normal by default, shown after the resume question. The request of 2026-09-27 described
it as "a delay, with a popup window showing the game name and art for the game". The player's account is §4.71 of the
settings reference.

**Sources.** ES-DE 3.4.1's behaviour, measured by running it, and its `USERGUIDE.md` ("UI settings") where it speaks;
never its source. `USERGUIDE.md` names the five durations and says that the menu colour scheme and the menu opening
animation apply to the launch screen. It says nothing of what the screen shows or how long each duration lasts.
Popup is "a simple notification popup", and Disabled makes "game launching … instantaneous". No file of ES-DE's entered
the repository: no image, font, colour table or XML. The code's proportions are fractions read off captures, as §32.1's
were. Its colours and typeface are the big-screen menus' (§32).

**Numbering.** Predictions from **P190**, questions from **Q90**; both ranges were free on WiseMan and on §32's branch
when this began.

### 33.1 The setup

- **ES-DE.** The AppImage under `~/.cache/emusen/bigpicture/esde/`, with a home of its own, `home-launch/`. The earlier
  homes were not touched. Each run:
  - `--home home-launch --resolution 1280 800 --fullscreen-padding off --no-update-check --no-splash --debug`,
    windowed;
  - one window at a time, closed by PID;
  - afterwards, a check that no ES-DE process remained (`ps` with an anchored pattern).
- **What it could reach.**
  - `ApplicationUpdaterFrequency` was `never`, and the ScreenScraper account fields were empty. Nothing scraped.
  - The ROM folder was `esde/launch-roms/`: ten empty `.sfc` files and seven empty `.nes` files. The tester's library
    was never named, and the runner asserts that its ROM folder is not the library.
  - `SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT` admitted only §22.2's uinput pad.
- **Something to launch.** `custom_systems/es_systems.xml` gave `snes` and `nes` one command each: a stand-in script
  (`probe/pass11/standin.sh`). The script writes the wall-clock time it starts and ends, sleeps for 1–4 s and exits 0.
  A variant exits 1 after 0.2 s, and a third names a program that does not exist.
- **Media.** `downloaded_media/<system>/<kind>/` held pictures drawn for the probe (`mkmedia.py`). Each kind had its
  own colour and its name written on it, so a capture shows which kind ES-DE chose. The kinds were covers, miximages,
  screenshots, title screens, marquees, fan art, 3D boxes, back covers and physical media. The SNES games carried
  every kind, all but one, one kind alone, or none. The NES games carried marquees of five shapes (800×300, 400×400,
  1600×160, 200×600, 100×40), and one game had a cover and a screenshot but no marquee.
- **Theme.** ES-DE's bundled Linear for most runs. Art Book Next was used for two runs, to see whether the theme
  matters.
- **Capture.** Recordings of the ES-DE window by `ffmpeg -f x11grab`, as §22.2 did:
  - 30 fps for 8–17 s around the press, and 60 fps for 6 s for the entrance;
  - single frames by the same tool where only a picture was wanted.

  The press time is from the uinput pad's log, the start of the "game" from the stand-in's log, both on the wall
  clock.
- **The runner** is `~/.cache/emusen/probe/pass11/esde_run.py`, with `pad2.py`, `frames.py` (per-frame change and
  luma), `card.py`, `cardfiles.py`, `bands.py`, `corner.py`, `px.py` and `delays.py` (press to start, every run).
  There were 27 runs of ES-DE and 37 launches, 36 of them timed (`delays.txt`); captures are under `probe/pass11/captures/`, logs under `probe/pass11/logs/`, and a summary in
  `probe/pass11/RESULTS.txt`.

### 33.2 What ES-DE does

**Durations.** From the press of A to the start of the command:

| Setting | Press to command | Launches | The card on screen |
|---|---|---|---|
| Normal | 3.011–3.016 s | 29 | 90 frames at 30 fps, 3.00 s |
| Brief | 1.718 s | 1 | 51 frames, 1.70 s |
| Long | 4.517 s | 1 | 135 frames, 4.50 s |
| Popup | 1.712–1.718 s | 4 | the notice, 1.7 s |
| Disabled | 0.016 s | 1 | nothing |

The 11–18 ms beyond the round figures is the pad's event reaching ES-DE. Normal does not depend on any of the
following, which the 29 launches varied:
- the game's media;
- the theme (Linear and Art Book Next);
- the menu colour scheme (dark, dark red, light);
- the opening animation (scale-up, none);
- the blur (on, off);
- the resolution (1280×800, 1280×720, 1920×1200);
- whether the command then succeeds.

**What the card shows** (captures `m01-each`, `m02-nes`):
- **The heading and text.** *LAUNCHING GAME*, then the marquee, then the game's name and the system's full name
  (`es_systems.xml`'s `fullname`), all in capitals and centred.
- **Only the marquee.** A game with every kind of picture showed its marquee. A game with a cover, a screenshot or any
  other kind but no marquee showed **no picture**, and the card is shorter, 256 px rather than 437 at 1280×800. A game
  with no media at all showed the same short card.
- **The marquee's box.** It is fitted whole into a box 356×154 px at 800 lines (0.445 × 0.1925 of the height),
  centred. A small marquee is scaled up (100×40 was drawn 356×142).
- **Size and place.** The card is 709 px wide (0.886 of the height) with corners of about 16 px, and centred 356 px
  down (0.445 of the height), not in the middle.
  - At 1280×720 it is 638×394 and centred 319.5 px down: the card scales with the height, not the width.
  - At 1920×1200 it is 1065×655.
- **A long name** widens the card to 1107 px (0.865 of the width at 1280×800). The name then ends in an ellipsis
  (*GRANITE CHOIR AND THE VERY LONG TITLE THAT…*).
- **The lines.** At 1280×800, from the card's top: the heading's capitals start at 50 px and are 34 px high; the name's
  at 302 px, 43 px high; the system's at 364 px, 26 px high. On the short card: 48, 126 and 185 px.
- **Colours.**
  - Dark scheme: card `#121212`, text `#909090`. Its main menu's panel is `#191919` with the same text colour.
  - Light scheme: card `#DFDFDF`, text `#555555`.
  - Dark with red: the card is identical to dark. The scheme's red shows only in the menus' selected row.

**The entrance** (`l02-normal60`, 60 fps):
- **The card.** It appears within a frame of the press at 0.58 of its size, then 0.64, 0.70, 0.78, 0.86, 0.945,
  0.995 and 1.0, about its centre. A linear scale from 0.5 over seven frames (117 ms) fits within 0.015.
- **The view behind.** It is shaded over four frames (67 ms) to 0.8 of its brightness (55 → 44) and blurred as it
  darkens.
- **With the blur off** (`MenuBlurBackground` false) the view is shaded the same and not blurred.
- **With the opening animation None**, the card and the shade are whole in the first frame.
- **The clock and the Bluetooth indicator** of the Linear theme stay sharp above the blur.

**While it shows.**
- Nothing moves after the entrance.
- B pressed twice (at 0.8 s and 1.1 s) changed nothing, and the command started at 3.016 s as always: **the screen
  takes no input.**
- There is no exit animation.

**The game.** At the command, ES-DE's window turns black in one frame and stays black until the command ends.

**The return.**
- The game list comes back without the card, shaded, and the shade lifts over about 0.33 s (mean luma 61.2 → 67.1 over
  ten frames at 30 fps).
- After Popup and Disabled, one unshaded frame comes first.

**The popup** (`l01-popup`, `c06-light-popup`):
- **Shape and place.** A rounded pill at the top centre, 16 px from the top and 51 px high (0.064 of the height),
  about 444 px wide for *LAUNCHING GAME 'AURORA DRIFT (SYNTHETIC)'*.
- **Colours.** Text `#BBBBBB` on `#181818` (dark), `#444444` on `#EFEFEF` (light).
- **Timing.** It fades in over 15 frames (0.5 s) and holds until the command.
- **The view.** It is not shaded or blurred.
- **The theme.** Art Book Next draws the same card and popup as Linear.

**Failures** (`x02-fail`, `x03-notfound`).
- **A command that exits 1 after 0.2 s:** the full screen, the black, then the game list, with nothing shown and
  nothing in the log.
- **A program that does not exist:**
  - the full 3 s screen;
  - then the game list without the black, its shade lifting;
  - and a popup, *ERROR: COULDN'T FIND EMULATOR, HAS IT BEEN PROPERLY INSTALLED?*, in the launch popup's style. It
    was still showing 5.5 s after the press, when the recording ended.

  The log has `Error: Couldn't launch game, emulator not found`.

### 33.3 P115 retired

P115 predicted that Normal lasts 1.5–3 s, Brief 0.4–0.6 of it and Long 1.5–2.5 times it.
- **Normal:** 3.00 s, measured as the card's 90 frames and as 3.011–3.016 s press to command.
- **Brief:** 1.70 s, 0.567 of Normal.
- **Long:** 4.50 s, 1.50 times Normal.

**P115 held**, but two of its three parts only at the edge of their ranges: Normal at the top of 1.5–3 and Long at
the bottom of 1.5–2.5. The prediction guessed a shorter Normal and a longer Long than ES-DE uses. The ranges were wide
enough to hold, not placed well.

### 33.4 What was built

- **`LaunchScreen`** (Mistress, `BigPicture/LaunchScreen.cs`). A panel built from LunaP's pieces:
  - `BlurBackdrop` (§32) over the themed view;
  - a card (`Border`) carrying three `FontText` lines and a `FittedImage` at high-quality sampling;
  - a `FontText` pill for Popup.

  It holds §33.2's timings and proportions as named fractions of the height. The colours and the font path are the
  big-screen menus' (`MenuPanel`'s defaults and its inherited `FontPath`, Barlow Condensed since §32). Its motion is
  a function of the interface clock (`UiClock`), which tests move. No bitmap is captured or scaled beforehand. The
  blur is Avalonia's Gaussian `BlurEffect` at full resolution, 14 px at 800 lines, §32's radius.
- **`MainWindow.LaunchScreen.cs`** opens it, steps it and closes it:
  - an animation frame is asked for only while the entrance moves;
  - after that, one `DispatcherTimer` waits for the end;
  - the pad poll steps it too, so a test's clock reaches it.
- **The start sequence.** `StartGameAsync` gains an optional game from the themed view: the firmware prompt, the
  resume question, then, for that game only, the launch screen, then `LoadGame`, then the screen closed. A failed
  load's reason goes to the notice. The themed view's one launch path passes its game, so every route through it shows
  the screen (§4.71). Nothing else passes one.
- **Input.** `PadTick` steps the screen and returns while it is open, forgetting what is held. The themed view's pad
  guard also refuses while it is open. The keyboard reaches the view only through `PadTick`.
- **Closing the window** ends the screen without starting the game (§15.14's lesson).
- **The setting.** `BigPictureInterface.LaunchScreenDuration` and its row on the Interface tab.

### 33.5 Where Mistress differs, on purpose or by necessity

- **The picture.** Marquee first, as ES-DE; then the cover, which ES-DE never shows (Q90).
- **The font.** Barlow Condensed, the menus' face (§32), not ES-DE's. A name wider than the card at 0.886 of the
  height widens it, as in ES-DE. Since the faces differ, the same name need not widen both cards alike (P190).
- **The colours.** The menus' `#1B1B1E` card and `#B4B4B8` text, not ES-DE's `#121212` and `#909090`. There is no
  colour scheme to follow; §21.2 left the look to LunaP and §32 set the menus' greys.
- **The system name.** The shelf's full name (*Super Nintendo*), not ES-DE's `fullname`.
- **The blur** covers the whole view, the theme's clock and indicators included.
- **No black interval.** The game draws in the same window, so the card gives way to its first frame.
- **The return shade** is not drawn (Q91). **The opening animation** is always the scale-up (Q92).
- **A failed load.** The notice gives the reason for any load that fails, where ES-DE shows its popup only for a
  missing emulator. The error log records it (§4.70).

### 33.6 Tests

`LaunchScreenTests`, 20 cases, headless on WiseMan's `PadDriver` with the clock the test moves (§4.71 lists them).

**Durations.** Each of Normal, Brief, Long and Popup is stepped in 16 ms polls. The game is not running at its end
less one poll, and is running, on screen, with the card gone, at its end.

**Pixel cases, in §15's style:**
- **The card at 1280×800 and 1920×1200.**
  - The card's place and size are the fractions of §33.2.
  - Its four inner corners are exactly the menus' panel colour, and three points just outside it are not.
  - The marquee's two halves are its own colours, so it is drawn unfiltered by the card.
  - Outside the card, at least 98% of the pixels that were not black before are changed by the shade and blur.
- **The scale-up.** At the press the card is half its size about its centre: the panel colour inside the half-size
  box and not beyond it. At 117 ms it is whole.
- **The popup.** Opacity 0 at the press, 0.45–0.55 at 250 ms and 1 at 550 ms. The pill sits 0.02 of the height from
  the top, centred. **Outside the pill, the frame is the view's own: 0 pixels differ.**

**Existing tests.** `ThemedSession` now sets Disabled, so the older themed tests start at once as they did. The
resume tests call `StartGameAsync` by reflection, and pass the new argument as null. No other test changed.

### 33.7 Mutants

The runner is `~/.cache/emusen/probe/pass11/mutate_pass11.py`, its verdicts `mutants-pass11.txt`.
- Each mutant was built with `-m:2` and tested alone under `nice -n 10` against `LaunchScreenTests`.
- Before changing a file the runner writes it to a state file, restores it in a `finally`, and on starting restores
  any file a cut-short run left mutated.
- The tree was rebuilt clean at the end.

**26 mutants: 24 caught on the first round, 2 survived, none failed to build.** One survivor was a weak test, which
was then fixed. The other is equivalent. After the fix, 25 of 26 are caught. Each mutant took about 12 s to build and
test.

| Area | Mutants (caught unless marked) |
|---|---|
| Durations | D1 Normal 2 s; D2 Brief as Normal; D3 Long as Normal; D4 Disabled waits; D5 Popup drawn as the card; D6 Disabled the default |
| Look | L1 no scale-up; L2 no shade; **L3 the card centred at 0.5 (survived, then caught)**; L4 no picture; L5 no fade for the popup; L6 the card in the text colour; L7 Close leaves it on view; L8 the popup mid-screen |
| Picture and text | A1 the cover before the marquee; A2 no cover when there is no marquee; A3 the shelf's short name |
| Timing | T1 the game a poll late; T2 the card's clock offset from the interface's |
| Resume and load | R1 the card before the resume question; R2 no reason shown for a failed load; R3 the card left up; R4 the window's close not ending it |
| Input and routes | P1 the pad reaching the view; **P2 the view's own guard without the card (survived: equivalent)**; B1 the themed view starting without it |

- **L3 survived** because the pixel test compared the card's centre with `LaunchScreen.CardCentre` times the height.
  That is the code's own constant, so moving the constant moved the expectation with it. The test now states ES-DE's
  measurement in pixels (437 high, 356 down, 709 wide at 800 lines). The mutant was caught on its rerun
  (`mutants-pass11-rerun.txt`). The marquee test's two heights were rewritten the same way.
- **P2 is equivalent.** `PadTick` returns before the themed view's guard is read while the card shows, so the guard's
  extra condition cannot change what happens. It is kept, in case a later caller reads `ThemedTakesThePad` outside
  `PadTick`.

### 33.8 Pictures

`LaunchScreenPictureTool` (with `EMUSEN_BIGPICTURE_PNG=1`) writes to `~/.cache/emusen/bigpicture/png/pass11/`, beside
ES-DE's captures in `~/.cache/emusen/probe/pass11/captures/`. Every picture was looked at. At 1280×800 and 1920×1200:
- the view before;
- the Normal card with a marquee at 0, 17, 33, 50, 67, 83, 100, 117 and 1,500 ms;
- the card with a cover, with no picture, and with a long name;
- the popup at 0, 250, 500 and 1,500 ms;
- a failed load's notice;
- the card over Art Book Next.

There is also the Interface tab's row at 1280×800.

The pictures were rendered twice: before WiseMan was merged, in the default face (Noto Sans), and after, in §32's
Barlow Condensed.
- **In Noto Sans** the card for *Aurora Drift (Synthetic)* was 882 px wide. The name did not fit the narrowest card,
  so the card widened.
- **In Barlow Condensed** the card for the same name is 709×437 at 1280×800 (ES-DE: 709×437) and 1064×656 at
  1920×1200 (ES-DE: 1065×655). With no picture it is 709×256 (ES-DE: 709×256), and with the long name 1108 wide
  (ES-DE: 1107). The long name is cut after the same word, *THAT…*.
- **The lines.** The heading's capitals span rows 188–221 and the marquee 462–817 × 264–396, both ES-DE's to the
  pixel. The name and system lines are within 1 px of ES-DE's.
- **The popup.** Its text is 352 px wide where ES-DE's is 405, at the same cap height, because Barlow is the narrower
  face.
- **Art Book Next.** The card over it is the same card.

### 33.9 Predictions

| # | Prediction | Found, or retired when | Verdict |
|---|---|---|---|
| P190 | *(written before WiseMan's merge brought §32's face)* In Barlow Condensed, the card for *Aurora Drift (Synthetic)* at 1280×800 is ES-DE's 709 px wide, because the name then fits the narrowest card | 709 px; the long name's card 1108 against ES-DE's 1107 | held; the pixel test now checks it |
| P191 | On the handheld at 1920×1200 in Game Mode, the game's first frame follows the end of Normal within 50 ms by the wall clock, the timer's lateness and the load together | a run on the device, with a recording | open |
| P192 | ES-DE's *Couldn't find emulator* popup lasts about 4 s, as its device popup does (§24.4) | ES-DE recorded past 6 s | open |
| P193 | Brief and Long vary from run to run as Normal does, within 5 ms of 1.71 s and 4.51 s press to command | five more runs of each | open |

### 33.10 Not done

- **Nothing ran on the handheld** (P191), and no real pad was used; the pad was §22.2's uinput device.
- **Brief, Long and Disabled were measured once each**, and Popup four times (P193).
- **ES-DE's return shade and the opening-animation switch** were not built (Q91, Q92). The **clock and indicators**
  stay blurred.
- **The error popup's length** was not measured (P192), and **a game that exits with an error** has no Mistress
  counterpart: an in-window core either loads or throws.
- **Videos**, which ES-DE's launch screen does not show in 3.4.1 with these settings, were not tried: no video was
  given to any game.
- **Themes other than Linear and Art Book Next**, and ES-DE's other bundled themes, were not tried, though the screen
  is ES-DE's own and not the theme's.
- **The desktop's pad menu route** to a game (none exists: the menu resumes, it does not start) needed nothing.

### 33.11 Open questions

- **Q90, the cover when there is no marquee.** ES-DE shows no picture without a marquee. Mistress shows the cover,
  because the library's covers come from four sources and marquees from two (§4.60), and a request of 2026-09-27 asked
  for "art for the game". **Recommendation:** keep the cover.
- **Q91, the return shade.** ES-DE lifts a shade off its game list over about 0.33 s when a game ends. Mistress
  returns through the pad menu, and P40's test (§15.4) compares the first frame after the return with a fresh build.
  **Recommendation:** no, unless a return straight from a game is built.
- **Q92, the opening animation.** ES-DE's *Menu opening animation* (Scale-up, None) sets the launch screen's entrance
  and its menus'. **Recommendation:** add it to the Interface tab with §32's stage 2, so that one switch covers both.

### 33.12 The broad run

After merging WiseMan (§32 and the neutral rewording of the docs), one run of the Mistress filter, without
`ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any GPU or Vulkan test, under `nice -n 10`:
**1,113 tests, 1,080 passed, 33 skipped (the picture, survey and live tools), 0 failed, in 3 min 47 s.** The two
failures of §31.12 did not recur: the credential guard's was fixed on WiseMan, and the headless-initialisation one
passed in this order.

## 34. ES-DE's menu look, stage 2, item 1: the metadata editor (2026-09-27)

*Built on branch `bigpicture-esde-menus-stage2`, from WiseMan at `b0c6bf75`; LunaP on `esde-menus-2`, from
`openemu-library` at `9872708`.* The first item of §32.8, begun once stage 1's look was approved (§32.11). The
player's account is §4.72 of the settings reference; LunaP's record of its pieces is its §182. The stage stops here, so
that the editor's look can be approved before the settings sheets, the list screen and the opening-animation switch.

**Numbering.** Predictions from P200, questions from Q100; both ranges were free in the tree when this began.

**Sources.** ES-DE's user guide, *Metadata editor* (the local copy
`~/.cache/emusen/bigpicture/motion/docs/USERGUIDE.md`, lines 4595–4721): the fields and their order, the colours of a
value (*"it will change color from gray to blue, and if the scraper has changed a value, it will change to red"*), the
five buttons, Y as the scrape shortcut, and the question on leaving. There is no capture of ES-DE's editor on this
machine; the guide's own picture is not in the local copy. The editor's proportions are therefore §32.1's, measured on
ES-DE's main menu, and not measured on its editor (Q104). ES-DE was not run, and no file of ES-DE's was copied.

### 34.1 What ES-DE's guide documents, set against what was built

| ES-DE | Mistress |
|---|---|
| the editor's title, the game beneath | *EDIT METADATA*, and under it the game's name and its file name, one to a line, smaller |
| the fields, in the guide's order | the same order; Favourite after Players, Times played and Play time after Hide metadata fields |
| text fields edited in a popup | a row with a chevron and the value as written; A opens Mistress's on-screen keyboard (Q100) |
| rating in half stars | five stars at the right of the row, Left and Right stepping half a star |
| release date, ISO 8601 | `YYYY-MM-DD` or *Unknown*, drawn in the menu's typeface, its parts stepped as before |
| flags | switch rows |
| controller, alternative emulator | option rows between `<` and `>`; the alternative emulator faded where the console has one engine |
| grey, blue once changed, red once scraped | the same, for text, stars and dates; the colours are Mistress's shades |
| green stars for a rating rounded when read | not built |
| Scrape, Save, Cancel, Clear, Delete | Scrape, Save, Cancel, Clear, **Hide from Library…**; no Delete (§23, §4.59) |
| Y scrapes | North scrapes, from anywhere in the editor |
| the question on leaving an edited game | the same, as an ES-DE message box over the editor |
| *Hide metadata fields* | built (§34.3) |
| *Custom collections sortname*, inside a custom collection only | built (§34.3) |
| *Folder link*, folders only | not applicable: the editor opens on games |

Two things are Mistress's own and stay. **Reset** beside a field that holds an edit (§4.59) is drawn as a small
outlined button at the row's right (Q101). **ScreenScraper's offered name** (§4.63) is two rows under Name, *Use This
Name* with the name in red, and *Keep Current Name*.

### 34.2 What was built

- **LunaP** (`esde-menus-2`, its §182):
  - `MenuPanel` gains `Subtitle`, `Buttons` (a band under the rows that never scrolls), `FooterMaxLines` and
    `ShowsTitleBand`;
  - `MenuRow` gains `ValueColor` and `ValueLetterCase`;
  - `MenuRows` draws a `TextBox` and a `ToggleSwitch` as rows, and push buttons at a given size;
  - `MenuFieldRow` hosts a control (the stars, the date) at a row's right;
  - `RatingPicker` and `DateStepper` take colours, and the date a font file and no frame;
  - `Dialogs.MenuConfirmAsync` asks in a message box drawn as a small menu;
  - `SheetLayer` leaves a chromeless sheet drawn beneath a chromeless sheet presented over it, and marks it covered.
- **Mistress**:
  - `MetadataEditorWindow` takes the pad family of a big-screen session and builds the layout of §4.72.1 from the same
    field controls its desktop form uses. Where a field's value comes from, which the desktop's `FieldRow` says as a
    hint, is the footer's line while the field has the focus. The editor's questions go through `MenuConfirmAsync` in a
    big-screen session.
  - `GameMetadata` gains `hidemetadata` and `collectionsortname`, stored in `game_edit` like every field.
  - `SceneGame.HideMetadata`, until now a collection entry's, is set from the flag, so `SceneBuilder`'s existing rule
    hides the game's metadata elements. `SceneGame.CustomSortName` is new, and `CollectionShelves` puts it in the
    sortname's place for a custom collection's games, so the order and Jump To…'s letters follow it there and nowhere
    else.
  - `MainWindow` opens the editor with the custom collections sortname only when the themed view's system is a custom
    collection.

### 34.3 The two fields

**Hide metadata fields.** The guide: *"This option will hide most metadata fields as well as any badges … The only
fields shown with this option enabled are the game name and description. … Game images and videos will also still be
displayed."* §22 already hid, for a collection's entry, what ES-DE 3.4.1 was measured to hide: rating, badges, date and
time elements, text bound to any metadata but the name and description, and elements marked `metadataElement`. The
flag reuses that rule unchanged, which is why nothing new was measured for it. Whether ES-DE hides the same set for a
game with the flag as for a collection's entry is assumed from the guide's sentence, not measured.

**Custom collections sortname.** The guide: it *"will only affect the sorting for custom collections, meaning the
normal system gamelists and the automatic collections … will not be affected"*, it takes precedence over the sortname,
it also sets the quick selector's letter, and *"it's not possible to set a different value per collection."* All four
follow from substituting it for the sortname when a custom collection's shelf is built, since the order and the letter
index both read the sortname. The editor shows it only from inside a custom collection, grouped or discrete.

### 34.4 Predictions

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P200 | *(written while the layout test was being written, after the first pictures)* The editor draws nothing outside its panel and the help bar at 1280×800 and 1920×1200 | 0 differing pixels outside at both sizes | held |
| P201 | *(written before the mutants)* Of twenty-seven mutants, at least nine in ten are caught on their first valid run | 26 of 27 (96%); L10 survived on a weak LunaP case, caught once the case was strengthened (§34.6) | held |
| P202 | *(written before the broad run)* The broad Mistress run passes with no failure this branch causes | 1,112 tests, 1 failed, in Avalonia's headless initialisation, in a graphics-settings case this branch does not reach; it passed alone twice (§34.8) | held, with that reservation |

### 34.5 Tests

`MetadataEditorLayoutTests`, seven cases (the detail is in §4.72.5): the editor at both sizes; grey then blue; Hide
metadata fields; the custom collections sortname; Hide from Library's message box; Enter with no pad.
`ThemedMetadataScrapeTests` gains the scraped red. The pixel rule is §32.4's: the frame with the editor, against the
same frame with its panel alone hidden, must differ nowhere outside the panel and the help bar. The colours are counted
in the value's own rectangle, as `MenuRow.Layout` reports it, within 14 of each channel.

LunaP's `MenuEditorTests` adds six cases (its §182.6).

**Keyboard.** A first version of the last case sent Tab and Down to move between rows with no pad, and neither moved
the focus in the harness. The same keys did not move it in stage 1's Gamelist Options either, checked in the same run,
so this is not the editor's new layout. The case now checks what does work, Enter on a focused switch and on a focused
button (Q102).

### 34.6 Mutants

Twenty-seven mutants, one at a time, by `~/.cache/emusen/probe/esde-menus-2/mutate_editor.py`. The runner writes a
state file holding the original text before each mutant, restores any leftover one when it starts, and touches every
restored file, so no build reuses a mutant's binary (§32.5's trap). Sixteen are in Mistress, run against
`MetadataEditorLayoutTests`, `ThemedGameOptionsTests` and the scraped-red case; eleven are in LunaP, run against its
`MenuEditorTests` (its §182.6).

| # | Rule broken | Result |
|---|---|---|
| E1 | the title is not *Edit Metadata* | caught |
| E2 | the file name left out under the title | caught |
| E3 | the custom collections sortname shown everywhere | caught by 2 |
| E4 | the editor never told it is inside a custom collection | caught |
| E5 | the custom collections sortname not applied in a collection | caught |
| E6 | the custom collections sortname ordering the console's list too | caught |
| E7 | Hide metadata fields never reaching the view | caught |
| E8 | Hide metadata fields with no default of off | caught |
| E9 | a changed value not coloured | caught by 2 |
| E10 | a scraped value in the edited blue | caught |
| E11 | the footer keeping a field's words while a button has the focus | caught |
| E12 | Save before Scrape | caught |
| E13 | the questions asked as the desktop's dialog | caught |
| E14 | Times played and Play time after Controller | caught |
| E15 | the editor opening on its default button, not its first row | caught |
| E16 | the stars not coloured | caught |
| L1–L9, L11 | LunaP (its §182.6) | caught |
| L10 | a row value's own casing ignored when drawn | survived; caught after the test was strengthened |

**26 of 27 caught on the first valid run.** L10 survived because LunaP's text-row case read the `ValueLetterCase`
property and never looked at the drawn value. The case now draws the row with the value as written and again with the
row's casing, and requires the two to differ; rerun alone, L10 was caught by it. Each mutant was caught by the case
written for it, and nothing else failed.

### 34.7 Pictures

In `~/.cache/emusen/bigpicture/png/esde-menus-2/`, from `MetadataEditorPictureTool` and one case of
`ThemedMetadataScrapeTests` (`EMUSEN_BIGPICTURE_PNG=1`). For each of `synthetic` and `artbooknext`, at each of
`1280x800` and `1920x1200`:

- `-gamelist`, before;
- `-editor`, as opened;
- `-editor-keyboard`, the name being typed;
- `-editor-rating`, `-editor-date`, `-editor-switch`, `-editor-choice`, each after a change;
- `-editor-reset`, a Reset focused, and `-editor-buttons`, Save focused;
- `-editor-hide-confirm`, the message box over the editor;
- `-editor-custom-collection`, opened inside a custom collection;
- `-gamelist-metadata-hidden`, after Hide metadata fields was saved.

`synthetic-*-editor-after-scrape` shows the scraped red and the offered name. The synthetic theme draws no metadata
elements, so its `-gamelist-metadata-hidden` differs from `-gamelist` only in the name typed earlier; Art Book Next's
pair shows the stars, date, players and play time gone. Every picture was looked at. Three
things seen in them are questions rather than fixes: the on-screen keyboard's own look, with *Shift* and *Done* cut at
1280×800 (Q100); a Reset beside a row stopping that row's bar and rule short of the panel's edge (Q101); and a long
controller name cut with an ellipsis (*NINTENDO ENTERTAINMENT…*).

### 34.8 The broad run

WiseMan had nothing new to merge: the branch was made from its head, `b0c6bf75`, and it had not moved. One run of the
Mistress filter without `ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any shader, GPU or Vulkan
test, under `nice -n 10`, on the final build of both branches (LunaP `05308ba`): **1,112 tests, 1,075 passed, 36 skipped (the picture,
survey, bench and live tools, this section's two picture tools among them), 1 failed, in 3 min 44 s.**

The failure was `GraphicsSettingsWindowLayoutTests.A_change_is_saved_at_once_and_the_console_reported`, which threw
from Avalonia's headless platform initialisation (*"The calling thread cannot access this object because a different
thread owns it"*, from `HeadlessUnitTestSession.EnsureIsolatedApplication`) before its own code ran. It passed, with
its three neighbours, when run alone twice. It is the same failure as §31.12's in `InputSettingsWindowRenderTests`, in
another class, and it is recorded the same way: an order-dependent failure of the headless setup. It was not run on the
unmodified tree, so that it is not this branch's rests on where it threw, in the platform's start-up rather than in any
code this branch changed, and not on a comparison.

*Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78, as §31.12's was: `GridSceneTests`' plain facts, fixed
in `b272675c`.*

### 34.9 Not done

- **The rest of stage 2**: the settings sheets as ES-DE menus, ES-DE's list screen for an option row, and the Menu
  opening animation switch (§32.8, Q92).
- **ES-DE's text popup** (Q100), and ES-DE's green stars for a rounded rating.
- A real pad, the handheld, and a capture of ES-DE's own editor to compare against (Q104).

### 34.10 Open questions

- **Q100, the text popup.** A on a text row opens Mistress's on-screen keyboard, in its own font and look; at 1280×800
  its *Shift* and *Done* keys are cut. ES-DE edits text in a popup drawn as its menus are. **Recommendation:** build an
  ES-DE text popup with the rest of stage 2, the keyboard inside it, and widen those two keys meanwhile.
- **Q101, Reset.** ES-DE has no per-field reset; Mistress's Reset sits at the right of a changed row, and the row's bar
  and rule stop short of it. Keep the button, or move Reset to a pad button in the help bar (X, *Reset*) so every row
  spans the panel? **Recommendation:** the pad button, with Reset kept reachable from the keyboard.
- **Q102, the keyboard in a big-screen sheet.** With no pad, Enter works in the editor, but in the harness neither Tab
  nor the arrows move between rows, and Backspace does not leave; stage 1's Gamelist Options behaves the same. ES-DE's
  keyboard moves with the arrows and goes back with Backspace. Route §4.52a's keys to a big-screen sheet as the pad's
  buttons? **Recommendation:** yes, checked on a real keyboard first, since the harness's key events may differ.
- **Q103, Hide metadata fields on the desktop.** The desktop editor shows the switch, and the desktop library ignores
  it, as ES-DE's flag concerns the themed view. Keep the switch there, or show it only in a big-screen session?
  **Recommendation:** keep it, so the flag can be seen and cleared from either.
- **Q104, a capture of ES-DE's editor.** The layout follows the guide and §32's measurements of ES-DE's main menu. A
  capture of ES-DE's own editor at 1280×800, taken by someone running ES-DE, would let the title band, the subtitle and
  the button row be measured rather than assumed. Worth taking before the look is approved?

### 34.11 The decisions of 2026-09-27 built: the small fixes, Q101 and Q102

§10.1 records the decisions: the editor's look approved, and each recommendation of §34.10 accepted. This subsection
and the ones after it record what was built on them, in the order it was built. §34.1–§34.10 are kept as they were
written.

- **The Hide from Library question** no longer names another program: *"EmuSen never deletes or moves a game's file.
  Hide this game from the library instead? Its file stays where it is; turn on Hidden Games in Preferences to list it
  again."* The desktop's dialog asks the same words.
- **Controller names** have a short form in a big-screen row: *NES*, *SNES*, *N64*, *Gamepad*; *None* and *Unknown*
  were short already. `GameMetadata.ShortText` gives it and `MetadataEditorWindow.ChoicesOf` uses it only in a
  big-screen session, so the desktop's dropdown keeps *Nintendo Entertainment System*. What is stored is the value
  (`gamepad_nintendo_snes`), never the words, so nothing stored changed.
- **The on-screen keyboard's word layout** (LunaP §182.8): *Shift* and *Done* are two keys wide, and *Space* two instead of
  four, with *.* moved down beside *-*, so every row is still ten keys. Measured before the change, at 18 px the
  labels need more than a key's 44 px less its padding, and were cut; LunaP's new case checks each label's own width
  against the room inside its key.
- **Q101, Reset.** The Reset buttons are gone from the big-screen rows, so every row's bar and rule reach the panel's
  edge again. **West** (X on an Xbox pad; **Delete** on the keyboard, ES-DE's X) resets the focused field. The help bar
  names it, *Reset* with the West glyph after *Scrape*, only while the focused field holds an edit, so the bar never
  offers what would do nothing. The desktop keeps its Reset buttons.
- **Q102, ES-DE's keys in a big-screen menu.** §4.52a's keys were the themed view's alone and stopped at any sheet. They
  now drive the pad menu and every chromeless sheet of a big-screen session as held pad buttons, through the same
  `PadHeld`, so a sheet sees exactly what a pad would send. The arrows move, Enter chooses, Backspace goes back, Delete is
  West, Insert is North (*Scrape* in the editor), F1, F4, Page Up and Down, and Home and End as in §4.52a. A text row
  is not typed into: the keys pass it as they pass any row. While the on-screen keyboard is open the keys are not taken,
  as before.

**A defect found on the way, and why it could not have shipped quietly.** The first build of Q102 stopped the stars and
the date from stepping by pad, and `ThemedGameOptionsTests` caught it: two cases failed, one on the rating and the date
being unset after Right. The pad router steps a control that takes Left and Right by raising a key on it, and the
window's keyboard handler, which runs first, now took that key for the player's Right, marked it handled and held it.
`PadWindowRouter.Raising` marks the router's own keys, and the window leaves them alone.

**Keyboard in the harness.** §34.5 recorded that Tab and Down moved nothing in stage 1's Gamelist Options with no pad.
Down now moves there and in the editor. Tab is still Avalonia's and still moves nothing in the harness; it is not one of
ES-DE's keys.

### 34.12 Q104: ES-DE's editor captured and measured

ES-DE 3.4.1 was run on 2026-09-27 in a scratch home of its own (`~/.cache/emusen/bigpicture/esde/home-menus2`), on
empty synthetic ROM files, driven by §22.2's uinput pad from a timeline, every run under the shared lock
`~/.cache/emusen/bigpicture/esde/esde.lock` and its whole process tree killed after. The runner and its timelines are
`~/.cache/emusen/probe/esde-menus-2/esde/`; the captures are under its `captures/`. At 1280 by 800, with a scan of
`editor-1280/g02_editor.png` (a pixel scan, not ES-DE's code):

| Measured | ES-DE's editor | Mistress before | Mistress now |
|---|---|---|---|
| panel | x 221–1059 (838 wide), y 79–753 | 840 wide, centred | unchanged |
| title | capitals 104–152 (49 px), centre 49 below the panel's top | 68 px type, centre 46 below | centre 49 below |
| subtitle | one line, centre 107 below the top, capitals about 18 px | two lines from 92, 32 apart | two lines from 107, 32 apart (Q105) |
| title band | 139 px to the rule, with one subtitle line | 154 with two | 171 with two |
| rows | 42 px apart, capitals 18–19 px, text 7 px in | 54 px, 26 px capitals | 42 px, 27 px type (19 px capitals) |
| rating | five outlined stars, 100 by 22 | 150 by 30 | 21 px a star, 105 by 21 |
| empty developer, publisher, genre, players, release date | "unknown" | nothing, "Unknown" | "unknown" |
| buttons | one row in a 54 px frame, capitals 26 px | 30 px type | 37 px type (26 px capitals) |
| title of the text popup | *ENTER NAME*, from the field's name | none | *Enter <row>* (§34.16) |

What ES-DE's editor has and Mistress does not: its subtitle is the file name and the system in brackets
(*Aurora Drift (Synthetic).sfc [SNES]*), not the game's name over its file (Q105); a scroll indicator at the title's
right (∨, ∧∨) when the rows run past the panel; per-row help (*Add Half Star* on the rating, *Toggle* on a switch,
*Clear File* on Clear). Leaving ES-DE's editor with nothing changed goes back to the gamelist options menu.

### 34.13 The settings sheets as ES-DE menus

ES-DE's own settings menus were captured the same day (`menus-1280/`): *UI SETTINGS* is one list, the theme downloader
a submenu row first, the theme's options (theme, variant, colour scheme, font size, aspect ratio, transitions, language)
as option rows after it, then the rest; *GAME COLLECTION SETTINGS* a list with a *BACK* button under the rows (§32's
`r7/m02`); every submenu a screen of its own that replaces the menu it came from, titled with its name.

`BigMenuForm` builds this from each desktop sheet in a big-screen session, from the sheet's own controls:

- a field row becomes rows by what it holds: a dropdown an option row, a switch a switch row named by the switch's own
  words (ES-DE's: *Display clock*, not the field's *Clock*), a text box or a path picker a text row, a slider an option
  row showing the words beside it (*5 min*), one button a row (a submenu row when it opens a screen), and several buttons
  for one thing, such as a theme's Use, Update, Remove and About, a submenu row of its own with the disabled one's words
  as its value (*In Use*); a group of switches, a row each;
- a field row's hint is the footer's line while one of its rows has the focus;
- a page's tabs become submenu rows; *Theme Settings* shows its Options inline first and Themes and Interface after
  them, as ES-DE's UI settings put the theme's options first; *Preferences* is its six tabs as submenus; *Game
  Collection Settings* is one screen;
- B, and the Back button under the rows, go back a screen, and close the sheet from its first; L1 and R1 still turn
  from one page to the next, as they turned the tabs;
- the controls keep their names and their code: a page that rebuilds its rows (a theme downloaded, a collection
  created) is shown again from them, and the controls of the screens not shown stay in the tree, unseen, so what finds
  a control by name still finds it.

The desktop's sheets are unchanged.

### 34.14 Q92: the menu opening switch

Measured from a 60-frame-a-second recording of ES-DE's main menu opening (`menus-1280/open.mkv`): the panel's width
across its middle was 0.61, 0.69, 0.75, 0.83, 0.91, 0.995 and 1.0 of its own on consecutive frames, which is a linear
growth from half its size over seven frames, 117 ms: the curve §33 measured for the launch screen's card. With
*Menu opening effect* set to *None* the whole panel was there on the first frame it appeared (`open-none-1280/`).

*Menu Opening Animation* (Scale-up, None; ES-DE's `MenuOpeningEffect`, Scale-up by default) is a row of the Interface
screen. With Scale-up, the pad menu, every chromeless sheet, and every submenu turned to grow from half their size to
their own over 117 ms, linearly, about the panel's centre; the help bar does not grow. With None they are whole at once,
and so is the launch screen's card. `MenuPanel.OpeningScale` (LunaP §182.9) draws it; `MenuOpening` steps it on the
interface clock at each pad poll. `ThemedSession` sets None, as it sets the launch screen to Disabled, so the older
pixel tests read a menu as it stands; the opening's own tests set Scale-up.

### 34.15 Q86: ES-DE's list screen for an option row

ES-DE's list screen (`menus-1280/m08`, *THEME FONT SIZE*): the row's name as the title, a row per choice with no
values, the chosen one highlighted, and *BACK* under the rows; the menu it came from is not drawn. In Mistress, A (or
Enter) on an option row of a big-screen menu (a settings screen, the editor, Gamelist Options) opens `OptionListWindow`
as that screen: A on a choice sets the row and goes back to it, B or Back goes back with the row unchanged. A pointer's
click on the row still drops the stock list down. A menu that is a message box's parent stays drawn under the box; a
list screen, like a submenu, replaces its menu (LunaP §182.9).

### 34.16 Q100: the text popup

ES-DE's popup (`editor-1280/g03_text_popup.png`): *ENTER NAME*, the text on a black bar, the keys as grey tiles in its
menu font with *CLEAR* and *CANCEL*. Mistress's on-screen keyboard, opened on a text row of a big-screen menu, is now
drawn so (LunaP §182.10): *Enter <row's name>* in the menu's title type, the text on a dark bar, the keys as tiles, the
current one dark with white words, over the menu shaded. The layouts, the buttons and what they do are the keyboard's
own, unchanged (§4.45.6); ES-DE's thirteen-key layout and its Clear and Cancel keys are not copied.

### 34.17 Predictions for the rest of stage 2

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P203 | *(written before the mutants of §34.18)* Of thirty-one mutants, at least nine in ten are caught on their first valid run | 31 of 31, each by the case written for it (§34.18) | held |
| P204 | *(written before the broad run of §34.20)* The broad Mistress run, after merging WiseMan, fails on nothing this branch causes | 1,406 tests, none failed (§34.20) | held; one run is weak evidence against an intermittent failure |

### 34.18 Tests and mutants for the rest of stage 2

New cases, all headless:

- `MetadataEditorLayoutTests`: the editor's measured proportions (42-pixel rows at 800 lines, *unknown*, the stars'
  width) in the case at both sizes; West resets the focused field and the help bar names *Reset* only on an edited one,
  whose bar now spans the panel; a controller's short name in the row and its full one on the desktop; the Hide
  question naming no other program; ES-DE's keys with no pad in the editor (Down past a text row without typing into
  it, Enter on a switch, Delete to reset, Backspace to the Save Changes box, Enter to save) and in Gamelist Options.
- `EsdeSettingsMenusTests`, twelve cases: Theme Settings, Game Collection Settings and Preferences at 1280 by 800 and
  1920 by 1200, each centred, its focused row's bar across the panel, a Back button in its band, no field row left
  visible, and no pixel changed outside the panel and the help bar; their rows' kinds and words; submenus and B back;
  R1 turning pages; the footer's line; a slider's words; the list screen at both sizes (its title, its choices, the
  current one focused, the menu beneath not drawn, B unchanged, A choosing) and from Gamelist Options; the opening's
  curve (half size at the start, linear, whole after 117 ms, for the pad menu, a sheet and a submenu) and None; the
  launch card whole at once with None; the text popup's title.
- LunaP's `MenuEditorTests` gains one case and `OnScreenKeyboardTests` two (its §182.8–§182.10).

Existing tests changed where a big-screen sheet is now a menu: a sheet left by one B is now left by B until it closes
(`ThemedSwitchesTests.PutAway`, and the same loop in `ThemedLibraryHostTests`, `ThemeSettingsSheetTests`, `PadSettingsWindowTests` and `BigPictureThemeListTests`); the Themes list is read from its rows and a
theme's buttons are reached through its submenu (`BigPictureThemeListTests`); the controller is chosen by its short
name; Reset is West; the launch screen's tests set Scale-up, which `ThemedSession` now turns off.

**Thirty-one mutants**, one at a time, by `~/.cache/emusen/probe/esde-menus-2/mutate_stage2.py` (the same runner as
§34.6, with its state file, its restore at start and its touched files): twenty-four in Mistress run against
`MetadataEditorLayoutTests`, `ThemedGameOptionsTests`, the scraped-red case, `EsdeSettingsMenusTests` and
`EsdeMenusTests`; seven in LunaP, six run against its `MenuEditorTests` and `OnScreenKeyboardTests` and one (a slider's
words ignored) against Mistress's. **All thirty-one were caught on their first valid run, each by the case written for
it.** One caught far more than its own: *None still scales up* failed ten cases, `EsdeMenusTests`' pixel cases among
them, because `ThemedSession`'s None is what lets them read a menu as it stands.

| # | Rule broken | Caught by |
|---|---|---|
| S1–S4 | the Hide question naming ES-DE; the SNES controller by its full name; West not resetting; Reset offered on every field | the case for each |
| S5–S7 | the router's keys taken for the keyboard's; ES-DE's keys driving the pad menu only; a text row typed into | the keyboard cases; S5 by four, as it also stopped the stars |
| S8–S9 | the editor's rows at 54; no *unknown* | the layout case |
| S10–S12, S24 | A dropping the stock list down; a choice not given to the row; the list opening on its first choice; the menu left drawn under the list | the list screen cases |
| S13–S18 | B closing a submenu's sheet; the shoulders not turning pages; the theme's options not inline; a switch named by its field; no footer line; a slider's words not followed | the settings cases |
| S19–S23 | None still scaling; the opening twice as long; the launch card ignoring None; a submenu not growing; the plain keyboard on a text row | the opening and popup cases |
| L12–L18 | the rows or the panel not scaled; no placeholder; Shift one key wide; the current key not drawn apart; a slider's words ignored; a message box no longer keeping its menu drawn | LunaP's cases, L17 by Mistress's |

### 34.19 Pictures

Added to `~/.cache/emusen/bigpicture/png/esde-menus-2/` by `SettingsMenusPictureTool` and the editor's tool, for each of
`synthetic` and `artbooknext` at `1280x800` and `1920x1200`:

- `-theme-settings`, `-theme-settings-interface`, `-theme-settings-interface-lower`, `-theme-settings-themes`;
- `-list-screen` (Launch Screen Duration) and `-list-screen-chosen`;
- `-collection-settings` and `-collection-settings-lower`;
- `-preferences` and a picture of each of its six screens (`-preferences-library` and so on);
- `-menu-opening-0.75`, the pad menu three quarters grown;
- the editor's pictures again, with the rows at ES-DE's size, the text popup and Reset on West.

Four side-by-sides against the captures of §34.12, all at 1280 by 800: `side-by-side-editor`, `side-by-side-text-popup`,
`side-by-side-ui-settings` and `side-by-side-list-screen`. Every picture was looked at. What they show that is not a
defect: the synthetic session's theme folder is named by a random identifier, so its theme rows read as one; Art Book
Next's view behind the menus is its gamelist of synthetic pictures, blurred.

### 34.20 The broad run

After merging WiseMan at `b272675c` (the scrape extras and a gridscene fix, which merged without a conflict) and
`openemu-library` at `5a1e1a2` into LunaP, one run of the Mistress filter without `ShaderSettingsWindowTests`,
`ShaderBrowseBench`, `SceneGpuBench` and any shader, GPU or Vulkan test, under `nice -n 10`, on the final build of
both branches: **1,406 tests, 1,364 passed, 42 skipped (the picture, survey, bench and live tools), 0 failed, in
4 min 10 s.** §34.8's intermittent failure of the headless setup did not recur in this order. LunaP's own suite ran
in full on its branch: **1,414 tests, all passed**.

### 34.21 Not done

- **ES-DE's per-row help** (*Add Half Star* on the rating, *Toggle* on a switch, *Clear File* on Clear); the help bar
  is the editor's and the menus' own, per screen (Q106).
- **The scroll indicator** ES-DE draws at a menu title's right when its rows run past the panel (Q107).
- **ES-DE's text popup's own layout**: thirteen keys a row with its symbols, and Clear and Cancel keys (Q109).
- **The desktop's sheets**, deliberately: every change of this section is for a big-screen session.
- **A message box's opening**: with Scale-up it grows as a menu does, which ES-DE's was not measured to do.
- A real pad, a real keyboard, the handheld; ES-DE was run only in the scratch home, at 1280 by 800.

### 34.22 Open questions

- **Q105, the subtitle.** ES-DE's editor has one line under its title: the file and its system, *Aurora Drift
  (Synthetic).sfc [SNES]*. Mistress's has two, the game's name and its file, as decided for §34. **Recommendation:**
  ES-DE's one line, since the name is the first row.
- **Q106, per-row help.** ES-DE's help bar changes with the focused row. **Recommendation:** build it for the editor's
  rating, switches and Clear, where ES-DE's words differ from *Select*.
- **Q107, the scroll indicator.** **Recommendation:** add it to LunaP's `MenuPanel`, drawn only when the rows scroll.
- **Q108, leaving the editor.** ES-DE goes back to Gamelist Options when its editor closes; Mistress goes back to the
  game list. **Recommendation:** keep Mistress's, one press shorter, unless the two should match everywhere.
- **Q109, the popup's layout.** Mistress's popup keeps its own layouts (ten keys a row, Letters and Code) in ES-DE's
  look. **Recommendation:** keep them: they are shared with the cheats window, whose codes need the Code layout.

## 35. Q70 and Q71: the other value types against ES-DE (2026-09-27)

*Opened on the answers to Q70 and Q71 of 2026-09-27 (§10.1).* §31 matched the loader to ES-DE for FLOAT, NORMALIZED_PAIR,
BOOLEAN and the XML itself, and left COLOR, UNSIGNED_INTEGER, STRING, PATH and `capabilities.xml`'s `selectable` as
`THEMES.md` types them (§31.4, §31.10). This section runs ES-DE on those, retires P170, P172 and P173, and matches the
loader rule by rule. The principle is §25.10's, as in §31: a theme ES-DE draws should not be one Mistress refuses, and a
theme ES-DE refuses should not be one Mistress draws. The player's account is §4.73 of the settings reference. Q72 was
answered "no" (§10.1): the warnings below go to the loader's diagnostics only, as §31's do, and nothing about them is
shown to the player.

**Sources.** ES-DE 3.4.1's behaviour, measured by running it, and `THEMES.md` where it speaks; never its source. All
probe themes are synthetic and were written under `~/.cache/emusen/probe/q70/probe-themes/`. For P173, the XML of Canvas
and Iconic that §25.8's survey had fetched was copied there, with three fonts stood in by a system font (ES-DE shuts
down when a theme's font file is missing), and deleted afterwards. No theme file, image or XML entered the repository;
the tests write their own XML.

**Numbering.** Predictions in this section start at **P210** and questions at **Q110**.

### 35.1 The setup

- **ES-DE.** The downloaded AppImage under `~/.cache/emusen/bigpicture/esde/`, with a home of its own, `home-q70/`, and
  §31.1's arguments (`--resolution 1280 800 --fullscreen-padding off --no-update-check --no-splash --debug`),
  windowed. Other work ran ES-DE on the same machine that day, so every run held the shared lock
  `esde/esde.lock`, waited for any other ES-DE window to close, and closed only the process tree it had started.
- **What it could reach.** `ApplicationUpdaterFrequency` was `never`, the ScreenScraper account fields were empty, and
  nothing asked it to scrape. Its ROM folder, `esde/q70-roms/`, held empty synthetic files: four per system for five
  systems, twelve for the SNES, and one for each of 60 more systems, so that a probe theme could give every value a
  system of its own. The real library was never named. `SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT` admitted only §22.2's
  uinput pad.
- **The runner** is `~/.cache/emusen/probe/q70/esde_run70.py` (settings, launch, a capture by `ffmpeg -f x11grab`, pad
  timelines, the log copied), wrapped by `run_locked.sh`. Every `es_log.txt` is kept in `probe/q70/logs/`.
- **Three kinds of run, in two rounds.** *Refusal themes* (`q70-values1`, `q70-values2`) give each of 129 probe values
  its own system folder, so a value that unthemes one system cannot hide another; the log then says, per system,
  whether it loaded. *Visual themes* draw each value where it can be read back from a capture: a colour's box half over
  white and half over black, from which RGB and alpha are solved; a row of badges whose icon spacing gives the number
  read; a box whose left edge is its x; images whose colour says which file was found. *Capabilities themes* declare
  variants and transitions profiles with each `selectable` value, and a pad timeline steps through UI Settings. The
  second round's predictions were written into `PREDICTIONS-b.txt` before it ran.

### 35.2 What ES-DE does, type by type

Only a `ThemeData::parseElement()` or `parseIncludes()` error unthemes a system. Every other message below is logged by
the element's component while it is drawn (`ImageComponent`, `TextComponent`, `BadgeComponent`), and the system stays
themed whether the line says `Warn` or `Error`.

| Type | ES-DE 3.4.1, measured | The rule it follows, as far as measured |
|---|---|---|
| COLOR | `#FF0000`, ` FF0000`, `FF0000 `, `F00`, `FF000`, `red`, `0`, `FFFFFFFFFF` and the listed X20s's `4c94ff6` unthemed the system; `GG0000`, `FF00ZZ`, `0xFF0000`, `-0000001`, `FF00FF 0` loaded. Read back: `GG0000` black, `FF00ZZ` and `FF00G0` green, `ffff00zz`, `0x00FF00`, `+0FF0000` and ` 00FF000` fully transparent, `-0000001` white, `0x0000FF` black, `0xFFFFFF` cyan; in the second round `0xFF00` green, ` FF000` `0FF000FF`, `-00001` white, `0x00FF` blue, `0X0000` black | the value **as written** must be 6 or 8 characters long, or the system is unthemed. It is then read as C's `strtoul` reads base 16: leading whitespace, a sign, a `0x` prefix, hexadecimal digits up to the first other character, none being 0; a minus negates modulo 2³². Six characters are shifted left by 8 with alpha `FF`; eight are RGBA as they stand. All nine second-round predictions held |
| UNSIGNED_INTEGER | never an error, for any of 57 values. `3.5`, `3.9`, ` 3`, `+3`, `3abc`, `0x3`, `3 4` and `3,5` drew as 3; `1e1` as 1; `2.9` as 2; `abc`, `.5`, `-0`, `0x`, `08` and `09` as 0; `010` as 8, `011` as 9, `012` as 10, `0xA` as 10; `4294967298` and `-4294967294` as 2 | C's `strtoul` with base 0: `0x` is hexadecimal, a leading `0` octal, else decimal; leading whitespace and a sign are read, and the number ends at the first character that is not a digit of its base. The result is kept to 32 bits: ES-DE's own warning prints `-1` as `4294967295`. The octal reading was predicted before round two and held on all eight octal values |
| … the badges' ranges | `itemsPerLine` and `lines` of 0, 11 and `4294967295` logged `BadgeComponent: Invalid theme configuration` and drew as the defaults (4 per line, 3 lines); 1 to 10 drew as written | a range `THEMES.md` does not state, 1–10, outside which the default applies. Measured on these two properties only |
| FLOAT | `1e-1` at 0.1 and `5E-2` at 0.05 (P170). Also `0x1p-2` and `0x0.4p0` at 0.25, `0x0.8` at 0.5, `0.1e` at 0.1, `0.3.4` at 0.3, `0x` at 0. As opacity, `inf`, `INF`, `infinity` and `1e999` drew fully opaque; `-inf` and `-1e999` not at all; `nan` and `NAN` not at all | C's `strtod`: §31.2's leading-number rule, extended to hexadecimal floats with a binary exponent, `inf`, `infinity` and `nan`; an infinity is then clamped to the property's range |
| STRING, an enumeration | `horizontalAlignment` `right` and `center` applied; `RIGHT`, `Right`, ` right`, `right `, `middle`, `LEFT` and `Left` drew as `left`, the default, each with a `Warn:` line. `metadata` ` name`, `Name`, `name ` and `nosuchfield` left the text's own text shown (`Error:` lines, system themed). A carousel's `type` of ` horizontal` warned | the value must equal one of the documented words exactly: case matters and nothing is trimmed |
| STRING, a list | `imageType` `cover screenshot`, `cover, screenshot`, ` cover` and `cover ` drew the cover; `Cover`, `boxart`, `cover;screenshot`, `screenshot,boxart` and `cover,cover` drew nothing, with an `ImageComponent` `Error:` line; the other elements of the view were drawn | split at commas and whitespace; one unknown or repeated type hides the element, and does not untheme the system |
| STRING, text | `  PADDED  `, `PADDED  ` and `  PADDED` drew with their spaces | kept as written |
| PATH | from `theme.xml`: `./img/c.png`, `.\img\c.png`, `./img/./c.png`, `./../<theme>/img/c.png` and the absolute path found the theme's file; `img/c.png` found the file of that name in **ES-DE's working directory**, and `../<theme>/img/c.png` nothing; ` ./img/c.png`, `./img/c.png ` and `./IMG/C.PNG` nothing (a `Warn:` line); `~/q70.png` found the file in **ES-DE's `--home`**; `./img` (a folder) nothing, with an `ImageComponent` `Error:` line. From an included `sub/inc.xml`, `./img/c.png` and `${imgdir}c.png` (with `imgdir` = `./img/` defined in `theme.xml`) both found `sub/img/c.png` | `./` is the folder of the file the property is written in, after variables are substituted; `\` is a separator; nothing is trimmed; `~` is the home; any other relative path is left to the operating system, so it is the working directory's |
| … an include | `inc.xml`, ` ./inc.xml `, `./inc.xml` followed by a newline, `./INC.XML` and a missing file unthemed the system; `.\inc.xml` loaded | the same resolution; a missing include written out is an error, as §12.3 had it |

```
Error:  ThemeData::parseElement(): ".../q70-values1-es-de/gbc/theme.xml": Invalid color property "#FF0000" (must be 6 or 8 characters in length) (system "gbc", theme "gbc")
Error:  ThemeData::parseIncludes(): ".../q70-values2-es-de/msx2/theme.xml" -> "inc.xml" not found (resolved to "inc.xml") (system "msx2", theme "msx2")
Warn:   TextComponent: Invalid theme configuration, property "horizontalAlignment" for element "t" defined as " right"
Error:  ImageComponent: Invalid theme configuration, property "imageType" for element "i" defined as "boxart"
Warn:   BadgeComponent: Invalid theme configuration, property "itemsPerLine" for element "b19" defined as "4294967295"
```

**The argument, separated from the mechanism.** As in §31.2, the table states rules that fit every value tried and
makes no claim about ES-DE's implementation. That the readings behave as C's `strtoul` and `strtod` would is a
description: it predicted the second round's values, which is what it is used for, but whether ES-DE calls
those functions was not, and under §25's rule could not be, looked at. The COLOR length is counted in characters here;
a value with a non-ASCII character was not tried (P210).

**Negative results on method.** A COLOR that ES-DE draws fully transparent says nothing about its other three
channels, so for `0xFF0000`, `ffff00zz` and the like only the alpha was measured; the tests state the channels the rule
gives. The first badge probe used four lines in a small box, where the icon size is set by the height and eight per
line looks like four; the second used one line in a wide box, where every value from 5 to 10 spaces the icons
differently. The `lines` probe (round two) could not show the number of lines, because with one badge per line ES-DE
laid the four badges out in one row whatever `lines` said (except 10); its log lines are the evidence for that range,
and the picture is not. Canvas's own carousel is a wheel that runs off the screen, so its `3.5`, `3` and `4` copies are
pixel-identical and do not tell 3 from 4; a synthetic `verticalWheel` with `1`, `1.5`, `1.9` and `2` does. A first
Canvas run shut ES-DE down on a missing font file, and a second probe of menu closing produced no capture; both were
repeated.

### 35.3 `selectable`, and Q71 confirmed

| Written | Offered, as a variant and as a transitions profile |
|---|---|
| `true`, `TRUE`, `yes`, ` yes`, `1`, `t`, `Y`, `x`, `2`, `on`, ` true `, a newline then `true`, ` false`, ` no`, a tab then `false`, and an empty or blank value | yes |
| `false`, `False`, `FALSE`, `F`, `f`, `flase`, `no`, `NO`, `N`, `0`, `0.0` | no |
| no `<selectable>` at all | a variant **no**; a transitions profile **yes** |

The rule that fits: **not selectable when the first character is `0`, `f`, `F`, `n` or `N`; selectable otherwise.** It is
not §31.2's BOOLEAN rule, which is true only for a first `t`, `T`, `y`, `Y` or `1`: under that rule ` true `, `x`, `2`,
`on` and a blank value would be false, and ES-DE offered all five.

**Q71 is confirmed.** A variant without `<selectable>` was not offered beside one with `true` (`aNone` and `bTrue`: only
`bTrue` listed), nor between a `false` and a `true` (`fNone`), nor in two other probes (`vNone`, `zNone`); two variants without one gave
**NONE DEFINED**, as §31.11's lone variant had. A transitions profile without one was offered in both probes that
declared one.

**Which variant is drawn.** An explicit `ThemeVariant` naming a variant that is not selectable is drawn (`eFalse`,
`fNone`). With the setting empty or naming no declared variant, ES-DE drew the **first declared** variant, selectable or
not (`aNone`, `eFalse`, `cNone`). Merely opening UI Settings and leaving it then saved the **first selectable** one: the
view redrew as `bTrue`, and `es_settings.xml` read `ThemeVariant="bTrue"` afterwards. The loader keeps §12.4's first
selectable (Q110).

### 35.4 P170, P172 and P173 retired

| # | Predicted (§31.9) | Measured | Verdict |
|---|---|---|---|
| P170 | ES-DE reads a FLOAT written with an exponent (`1e-1`, `5E-2`) as its value | `1e-1` drawn at x = 0.1 and `5E-2` at 0.05, pixel for pixel with `0.1` and `0.05` | **held** |
| P172 | ES-DE reads `selectable` by §31.2's first-character rule (`yes` selectable, `flase` not) | the two named values held, but the rule did not: ` true `, `x`, `2`, `on` and a blank value are selectable, `f`, `n` and `0` first make it not | **failed** as a rule; §35.3's rule replaces it |
| P173 | ES-DE draws Canvas's and Iconic's game-list carousels with `itemsBeforeCenter` of `3.5`, reading it as 3 | both drawn at 16:10, medium font size, with no parse error; the synthetic wheel drew `1.5` and `1.9` pixel-identical to `1` and unlike `2`, and `itemsPerLine` `3.5` drew as 3 | **held** |

P172 was written as the lenient reading ES-DE had shown for element properties. The measurement shows that
`capabilities.xml`'s one boolean is read by a different test, one that asks whether the value says no rather than
whether it says yes; that it differs from the BOOLEAN rule is why §31's rule does not generalise to it.

### 35.5 What changed in the loader

- **`ThemeValueParser`** gained `EsdeColor`, `EsdeUInt`, and a C reading of numbers behind `EsdeFloat` that adds
  hexadecimal floats, `inf`, `infinity` and `nan`. `Finite` turns a value ES-DE could not draw at a property with no
  range, a `nan` or an infinite position, into 0; a range clamps an infinity first, as ES-DE's opacity showed.
  `ResolvePath` no longer trims, and resolves a relative path without `./` against the working directory. The 0 for an
  unbounded infinity is a choice: ES-DE's drawing of an infinite position was not measured, and a renderer cannot place one.
- **`ThemeViewBuilder`** applies them to the value as written. A COLOR of another length stays an **error**, whose
  message now says why ("which is not 6 or 8 characters long; ES-DE refuses such a colour too"); any other colour, whole
  number or number that is not written plainly carries a **warning**, `LenientValue`, saying what it was read as. An
  enumerated STRING is compared untrimmed. An unknown `imageType` is now a **warning**: the element is still not drawn,
  but the system stays themed, where before it was unthemed.
- **`ThemeCatalog`** gained `ValidRange`, a range `THEMES.md` does not state, set to 1–10 on the badges' `lines` and
  `itemsPerLine`: outside it the default applies with a warning. It is kept apart from `Min` and `Max`, which remain
  `THEMES.md`'s and are clamped, so `ThemeCatalogTests` still checks the catalogue against the document.
- **`ThemeLoader`** reads an `<include>` untrimmed.
- **`ThemeCapabilities`** reads `selectable` by §35.3's rule, with a `LenientValue` warning for anything but `true`,
  `false`, `1` and `0`. A variant without one is not selectable; a transitions profile without one is.
- **What did not change.** The default variant stays the first selectable one, else the first (§12.4, Q110). `~` stays
  the player's home. A path naming a folder still counts as present. `gameCount`, `iterationCount` and the carousel's
  counts keep `THEMES.md`'s clamping, since their ranges were not probed.

§12.3's rows on a value in the wrong format and on an unknown `imageType`, and §12.4's items 2 and 10, are superseded
by this section; they are kept, with a pointer, as the record of what stage (a) took from the document.

### 35.6 Tests

`ThemeValueRulesTests`, 184 cases, on synthetic XML written by the tests:

| Rule | Test |
|---|---|
| a colour of 6 or 8 characters read as hexadecimal the C way, warned when not plain (26 values, each measured) | `A_colour_of_six_or_eight_characters_is_read_as_hexadecimal_the_C_way` |
| any other length unthemes, and says why (13) | `A_colour_of_any_other_length_unthemes_the_system_as_in_ES_DE` |
| a whole number: base from its prefix, 32 bits (34; `99999999999` was drawn only as out of range, and one value beyond 2⁶⁴ was not run, P211) | `A_whole_number_is_read_the_C_way_with_its_base_from_its_prefix_and_kept_to_32_bits` |
| the badges' 1–10 | `Badge_lines_and_items_per_line_keep_1_to_10_and_reset_the_rest_to_the_default_with_a_warning` |
| P173 on a carousel | `A_carousel_count_written_as_a_fraction_keeps_its_whole_part_and_the_system_themed` |
| exponents, hexadecimal, an unbounded infinity (16) and a bounded one (14) | `A_float_reads_exponents_and_hexadecimal_and_an_unbounded_infinity_is_0`, `An_infinite_float_is_clamped_to_the_range_and_nan_draws_nothing` |
| enumerations exact and untrimmed; metadata; text kept | `An_enumerated_string_must_match_exactly_and_untrimmed_or_the_default_applies`, `A_metadata_name_that_is_not_exact_leaves_the_text_shown`, `Text_keeps_its_spaces` |
| an unknown `imageType` keeps the system themed | `An_unknown_imageType_hides_the_element_and_keeps_the_system_themed` |
| paths and includes | `A_path_is_the_theme_folder_only_after_dot_slash_and_is_not_trimmed`, `An_include_is_read_untrimmed_and_a_bare_name_is_not_the_theme_folder` |
| `selectable` (28 values), Q71, and no variant offered | `Selectable_is_false_only_when_its_first_character_is_0_f_or_n`, `A_variant_without_selectable_is_not_offered_and_a_profile_without_one_is`, `With_no_variant_offered_the_first_declared_is_drawn` |
| **install**: `3.5`, `boxart` and `GG0000` install through `FakeThemeHosts`; `#FF0000` is refused with its reason and leaves no folder | `A_download_installs_where_ES_DE_reads_the_value_and_is_refused_where_it_does_not` |

**The tests against the unchanged loader.** Run on WiseMan's loader first, 101 of the class's then 170 cases failed.
Thirteen of them were the colour-length refusals, which the old loader refused as well: they failed only on the new
message. The other 88 are behaviour the old loader did not have.

`ThemeErrorTests` lost two rows of its wrong-format theory (`GGGGGG` for a colour, `-1` and `1.5` for `lines`), which
ES-DE reads, and gained `#FF0000`; its `imageType` test now expects a warning and a themed system.
`ThemeCapabilitiesTests`' default-variant test and `ThemeSettingsSheetTests`' synthetic theme now state
`<selectable>true</selectable>` on the variants they expect offered, as every listed theme does (§35.7). After the
change the theme classes (370 tests) and the `Mistress.BigPicture` namespace (674: 649 passed, 25 skipped tools)
passed.

### 35.7 The survey again

`EMUSEN_THEME_SURVEY=analyse` on §25.8's XML, first with WiseMan's loader and then with this one (`survey-before-q70.json`,
`survey-after-q70.json`):

| | before | after |
|---|---|---|
| themes themed for all five systems | 66 of 66 | 66 of 66 |
| themes with no loader error | 63 | **65** |
| errors, by code | `BadFormat` 2 (Canvas, Iconic), `UndefinedVariable` 1 (Aura) | `UndefinedVariable` 1 |

Only two rows changed, Canvas's and Iconic's: 20 `BadFormat` errors each became 20 `LenientValue` warnings, and every
other count of theirs, `PathMissing` included (3,110 and 2,225), is the same, so no include or path of theirs resolves
differently. **Neither lost an element**: their undrawn-element lists are unchanged, and `itemsBeforeCenter` and
`itemsAfterCenter` now reach the carousel (they join the survey's list of properties the renderer does not yet draw,
§25.8). The other 64 themes' rows are identical, byte for byte. A scan of the same XML found no include or path written
without `./` or a variable, none with surrounding whitespace, and every one of the 544 `selectable` values `true`,
`True` or `false`, so Q71 and the path rules change nothing for a listed theme.

### 35.8 Mutants

The runner is `~/.cache/emusen/probe/q70/mutate_q70.py`, its log `run-q70.log` and its verdicts `mutants-q70.txt`. Each
mutant was built with `-m:2` and tested alone under `nice -n 10` against `ThemeValueRulesTests`, `ThemeEsdeRulesTests`,
`ThemeErrorTests`, `ThemeLoaderTests`, `ThemeCapabilitiesTests` and `ThemeCatalogTests`. Before changing a file the
runner writes it to a state file, restores and touches it in a `finally`, and on starting restores any file a cut-short
run left mutated; the tree was rebuilt clean at the end.

**31 mutants: 31 caught, none survived, none failed to build.**

| Area | Mutants |
|---|---|
| COLOR | C1 length after trimming; C2 a non-hexadecimal character refuses; C3 no `0x`; C4 six digits not shifted; C5 a minus ignored; C6 leading whitespace not skipped |
| UNSIGNED_INTEGER | U1 a leading 0 decimal; U2 no `0x`; U3 saturated rather than kept to 32 bits; U4 a minus ignored; U5 overflow past 2⁶⁴ wraps; U6 a leading 0 counted plain; U7 the badges' range ignored |
| FLOAT | F1 no hexadecimal; F2 no `inf`; F3 `nan` kept; F4 an infinite pair kept; F5 no exponent (§31.8's M18, which survived there); F6 the binary exponent ignored |
| STRING | S1 an enumeration trimmed; S2 its case ignored; S3 an unknown `imageType` an error again |
| PATH | P1 trimmed; P2 a bare relative path is the file's folder; P3 an include trimmed; P4 backslashes kept |
| `selectable` | B1 a variant without one offered; B2 a profile without one not offered; B3 read by the BOOLEAN rule; B4 trimmed first; B5 a blank value false |

F5 is §31.8's surviving M18, now caught: P170's measurement made the exponent a rule a test may pin.

### 35.9 Predictions

| # | Prediction | Retired when |
|---|---|---|
| P210 | ES-DE counts a COLOR's length in bytes, so a six-character value holding one non-ASCII letter is refused, where the loader, counting characters, reads it | ES-DE is run on one |
| P211 | ES-DE reads a whole number beyond 2⁶⁴ (`99999999999999999999`) as 4294967295, as the loader does; only values below 2⁶⁴ were run | ES-DE is run on one |
| P212 | Other enumerated properties (`verticalAlignment`, `direction`, `stationary`, `letterCase`) follow §35.2's exact, untrimmed match, measured on three | ES-DE is run on them |
| P213 | An unknown `imageType` on a `video` element hides it as it hides an `image`; only images were run | ES-DE is run on one |

### 35.10 Not done

- **Nothing ran on the handheld**, and Mistress drew none of the probe values; the tests check the loader's typed
  values, and the survey counts loads.
- **The ranges of other whole numbers** (`gameCount`, `iterationCount`, `gameselectorEntry`, the carousel's counts)
  were not probed; they keep `THEMES.md`'s clamping. Only the badges' two were.
- **A path naming a folder** counts as present in the loader; ES-DE logged an error for an image and drew nothing. Both
  keep the system themed, and no listed theme writes one; not changed.
- **`~` as ES-DE's `--home`**: ES-DE resolved `~` to its `--home` folder, which is the home only when none is given.
  Mistress has no such argument, and keeps the player's home.
- **Languages and `<transitions>` inside `<aspectRatio>` or `<language>`**, left by §31.10, were not taken up.

### 35.11 Open questions

- **Q110, the default variant.** With no variant chosen, ES-DE draws the first declared one until UI Settings is
  opened, and the first selectable one after (§35.3). Mistress draws the first selectable one from the start.
  **Recommendation:** keep it. The first-declared state lasts only until the settings are opened, and in the 66 listed
  themes every first variant is selectable, so the two agree wherever a player could see a difference.
- **Q111, a relative path without `./`.** ES-DE resolves it against its working directory, which depends on how it was
  started; the loader now does the same with Mistress's. No listed theme writes one. **Recommendation:** keep matching
  ES-DE, since a theme that works by accident in one frontend and not the other is the case §25.10 wants avoided.

### 35.12 The broad run

WiseMan had not moved since the branch was made (b0c6bf75), so the merge before the run was empty. One run of the
Mistress filter, without `ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any GPU or Vulkan test,
under `nice -n 10`: **1,295 tests, 1,261 passed, 33 skipped (the picture, survey and live tools), 1 failed, in 3 min
52 s.** The failure is §31.12's order-dependent one of the headless platform's initialisation ("The calling thread
cannot access this object"), this time in `FrameHandOffTests`, whose eight tests passed when run alone; it is not this
branch's.

*Attributed 2026-09-27 by `EmuSen_Settings_Reference.md` §4.78, as §31.12's was: `GridSceneTests`' plain facts, fixed
in `b272675c`; the failure kept its message, so, unlike §16.8's in the same class, it is attributed by mechanism as
well as timing.*

## 36. Pass 14, first half: `gameselector` and the wheel carousels (2026-09-27)

*Opened on Q48's answer of 2026-09-27 (§10.1): pass 14 draws `gameselector` (20 of the 66 listed themes), then the
wheel carousels (9), then `gamelistinfo` (6), then `animation` (4).* This section records the first two, built,
tested and committed; the pass stops here for a review before the other two. The survey that sized it is §25.8, and
the loader it builds on reads what ES-DE reads since §31 and §35. The player's account is §4.74 of the settings
reference.

**Sources.** ES-DE 3.4.1's behaviour, measured by running it, and `THEMES.md`'s gameselector and carousel entries;
never its source. Two synthetic probe themes were written under `~/.cache/emusen/probe/pass14/probe-themes/`, and three
listed themes were downloaded whole into `~/.cache/emusen/bigpicture/pass14-themes/` for captures: CodyWheel (a
vertical wheel beside a gameselector's fan art), Aura (a horizontal wheel, and reflections under a game carousel) and
Mania Menu (a gameselector's screenshot behind the system list). No theme file, image or XML entered the repository;
the tests write their own themes and pictures.

**Numbering.** §34 is another pass's. Predictions here start at **P220** and questions at **Q120**.

### 36.1 The setup

- **ES-DE.** The downloaded AppImage under `~/.cache/emusen/bigpicture/esde/`, with a fresh home, `home-pass14/`, and
  §31.1's arguments at `--resolution 1280 800` and `1920 1200`, windowed. Other work ran ES-DE on the same machine that
  day, so every run held the shared lock `esde/esde.lock`, then waited for any other ES-DE window to close, and closed
  only the process tree it had started; the absence of any ES-DE process was checked after each.
- **What it could reach.** `ApplicationUpdaterFrequency` was `never`, the ScreenScraper account fields were empty, and
  nothing asked it to scrape. Its ROM folder, `esde/pass14-roms/`, held empty files: the synthetic library of §13.2
  (five systems of twelve games, with the same gamelists) and seven more systems for the wheels. Two of those carry
  metadata for the gameselector's rules: `gba` has one game played three times and two never played, and `genesis` has
  its most recently and most often played game excluded from the game counter and another hidden. Their media are flat
  labelled pictures drawn by the probe's own script. The tester's library was never named.
- **The runner** is `~/.cache/emusen/probe/pass14/esde_run14.py` (settings, launch, a capture by `ffmpeg -f x11grab`,
  pad timelines by §22.2's uinput pad, the log copied), batched by `batch.sh`. 63 runs; every `es_log.txt` is kept in
  `probe/pass14/logs/`, and none logged an error.
- **The probe themes.** `p14-sel-es-de` draws, per gameselector and entry, the picked game's name as a text, so one
  capture reads thirty values. `p14-wheel-es-de` gives every system a flat logo of its own colour, 200 by 100 pixels
  with a white corner square, and one variant per property; `wheelfit.py` reads each logo's centre and angle from the
  pixels of its colour, and `compare.py` sets ES-DE's capture beside Mistress's render and counts the pixels that
  differ.

### 36.2 `gameselector`: what ES-DE does

On the synthetic SNES games (last played on 23, 22, 20, 19 and 17 September for the second, third, fifth, sixth and
eighth, and played `i` times for the `i`th), the probe's rows read:

| Asked | ES-DE 3.4.1 drew |
|---|---|
| `lastplayed`, `gameCount` 5, entries 0–4 | Brass Lantern, Cobalt Harbor, Ember Circuit, Fable of Tiles, Hollow Comet: newest first |
| `mostplayed`, `gameCount` 5 | Lumen Garden, Kestrel Run, Juniper Vault, Ivory Signal, Hollow Comet: most first |
| entry 5, and entry 9, of a `gameCount` of 5 | the fifth, Hollow Comet, both times: clamped to `gameCount − 1`, as `THEMES.md` says |
| the `gba` probe: `lastplayed` and `mostplayed`, five each | only the game played; entries 1–4 **not drawn**. A never-played game is picked by neither |
| `random`, six of three games | three different names, entries 3–5 not drawn |
| `random` with `allowDuplicates`, six of three | six names, the first three different |
| the `genesis` probe | the game excluded from the counter never picked, however recent or frequent; the hidden one neither |
| a text naming no selector, among four | the selector whose **name sorts first**, not the first defined: in two probes with `zlast` and `amost` defined in both orders, `amost` both times |
| a text naming a selector that does not exist | the same, the first name |
| one selector, and a text naming another | that one selector (`THEMES.md`: "this property is ignored") |
| no gameselector in the view | an image with `imageType`, a text with `metadata` and a rating **draw nothing**; a static text draws |
| the carousel moved away and back | new random picks each time; `lastplayed` and `mostplayed` unchanged |
| a date of the last played game | "3 days ago": relative, as a gamelist's `lastplayed` datetime is by default |

`THEMES.md` says that with several selectors and no name "the first entry will be chosen". ES-DE's "first" is the first
by name. The argument is separate from the mechanism: the probes show which selector is taken in both orders of
definition; whether ES-DE keeps its selectors in a sorted map was not, and under §25's rule could not be, looked at.

### 36.3 The wheels, reflections and offsets: what ES-DE does

Every rule below fits every probe capture to within a pixel of the whole logos' centres (`wheelfit.py`), and is how
LunaP's `ImageCarousel` now lays items out (its §190).

- **The hub.** Every item of a wheel starts at one box, `itemSize` large, centred along the wheel's axis and placed
  across it by `wheelHorizontalAlignment` (vertical wheel) or `wheelVerticalAlignment` (horizontal). The picture is
  fitted inside that box and placed by `itemHorizontalAlignment` and `itemVerticalAlignment`.
- **The turn.** The item `k` places from the selection is turned by `k × itemRotation` degrees, clockwise on screen,
  about the origin. For a **vertical** wheel the origin is the box's top left plus `itemRotationOrigin` in multiples of
  the item's width and height, as `THEMES.md` describes. For a **horizontal** wheel, which `THEMES.md` does not
  describe, the same construction turned a quarter turn anticlockwise: the centre lies `(0.5 − X) × width` below the
  box's centre and `(Y − 0.5) × height` to its right. With the default `-3 0.5`, a horizontal wheel's items ride an arch
  whose centre is 3.5 item widths below the selection; with `0.5 4`, a column turning about a centre to the right.
- **`itemAxisHorizontal`.** The items stay level and move by the travel the turn gives the point on the box's left edge
  at the origin's height: `R(θ)·a − a` with `a = (−X × width, 0)` (horizontal wheel: `(0, X × width)`). The origin's Y
  plays no part, which the probe showed: `-1 0.5` and `-1 3` drew identically.
- **`itemsBeforeCenter`, `itemsAfterCenter`**: the items drawn; twelve systems on a default wheel of 8 and 8 repeat, as
  a straight carousel's row does (§13.8).
- **Growth.** A wheel's selected item grows about the box's centre. With `itemHorizontalAlignment` left or right, every
  item of a vertical wheel is also moved by half the growth towards that side, `(itemScale − 1) × width / 2`: at
  `itemScale` 1.5 the neighbours stood 72–74 px left of the plain rule's places. A straight carousel's selected item
  grows about the edge its cross-axis alignment names: the top edge for `itemVerticalAlignment` top, the left for a
  vertical carousel's `itemHorizontalAlignment` left, the centre for center.
- **Clipping.** Every carousel, straight or turned, is clipped to its box.
- **`horizontalOffset`, `verticalOffset`** move every item by fractions of the box's width and height.
- **Reflections**, on a horizontal carousel only (a vertical one and the wheels ignore them): each picture mirrored
  directly beneath itself (not beneath its item box), the same size. The row makes room for them: the unit placed by
  `itemVerticalAlignment` is twice the item's height, the item in its upper half. The reflection's opacity is the
  item's own times `reflectionsOpacity` at the picture's edge, falling linearly to nothing at `1 / reflectionsFalloff`
  of the picture's height: 0.494 at the edge and 0.018 near the end of a 96-pixel reflection at the defaults, nothing at
  48 pixels with falloff 2, and 0.25 at the end with falloff 0.5. It grows with its item.
- **Moving.** A recording of one step of the vertical wheel at 30 frames a second turned every item continuously, the
  selection reaching its new place in about eleven frames on an easing curve like the straight carousel's (§14.7). The
  wheel's position is the same fractional item index, so the scene's existing clock drives it.

**Mistress against ES-DE on the probe theme** (`compare.py`, the help bar's strip left out, since ES-DE draws a default
help bar for a theme that has none and Mistress does not):

| | 1280×800 | 1920×1200 |
|---|---|---|
| logo centres, 41 variants | within 0.8 px for every logo drawn whole; up to 5 px for logos more than half hidden by a neighbour or the screen's edge, whose visible part is a few antialiased pixels | within 0.7 px (4 variants), one logo under ES-DE's help bar 6 px |
| logo angles | within 0.2° where the logo is whole | within 0.1° |
| pixels differing by more than 8 grey levels | 0.00–0.34% per variant | 0.00–0.22% |
| a reflection's column, grey level by level | within 1 level at the defaults; 3–4 levels (under a pixel of a steep fade) with falloff 2 | |

### 36.4 What was built

- **Mistress.** `GameSelectors` picks each gameselector's games from the system's counted games, without folders or
  games excluded from the counter: `lastplayed` and `mostplayed` by §36.2's rules, `random` by a seed that
  `SceneData.Shuffle` carries and that every move of the system view moves on. `SceneBuilder.GameFor` gives an element
  its game: the list's selection in a gamelist, the chosen selector's entry in the system view, or none, in which case
  images with `imageType`, metadata texts and dates, and ratings draw nothing. `PrimaryElements.Carousel` maps the
  wheels, their properties, the offsets and the reflections onto `ImageCarousel`; `SceneMapping` claims 3
  gameselector pairs, the `gameselector` and `gameselectorEntry` of five elements, and 12 carousel pairs.
- **LunaP** (branch `pass14-elements`, its §190). `ImageCarousel` gained a wheel layout, `ContentOffset`, clipping,
  reflections and the growth edge. Each item's placement is now one matrix transform about its top left.
- **Lottie is left out.** `animation`'s Lottie files would need Skottie, a SkiaSharp library, which LunaP may not take
  (it references Avalonia alone); it would go to a sibling package or to Mistress by `PLAN-icons.md` §1.1's rule. This
  is recorded now because §21.3 raised it; `animation` itself is the pass's fourth item.

**Where Mistress differs, and why.** The random picks are Mistress's own sequence, so a random selector shows other
games than ES-DE's in the same state. A text of a metadata field a game lacks shows nothing, where ES-DE writes
"unknown" (§3.6; the same holds in the gamelist). A `mostplayed` tie keeps the list's order, which was not measured.

### 36.5 Predictions

None was written before the runs: the rules were read off the first captures and each later probe was designed from
the last. The following are written now, for what the runs did not measure:

| # | Prediction | Retired when |
|---|---|---|
| P220 | A wheel's step, recorded frame by frame, follows §14.7's carousel curve (400 ms, quadratic ease-out) to within one frame at 30 frames a second | a recording is fitted |
| P221 | A horizontal wheel with `itemVerticalAlignment` top and `itemScale` above 1 moves its items by half the growth, as a vertical wheel's side alignment does | ES-DE is run on it |
| P222 | Two games tied on play count keep ES-DE's list order in a `mostplayed` selector | ES-DE is run on it |
| P223 | CodyWheel, Aura, Mania Menu and the six other wheel themes draw their wheels within 1 px of ES-DE at 1280×800 once §36.9's other differences are removed | the next capture of each |

### 36.6 Tests

| Class | Tests | What it holds |
|---|---|---|
| `GameSelectorTests` | 10 | §36.2's rules, each on the games ES-DE was measured on: newest and most first, the unplayed left out, random with and without duplicates (20 seeds), the clamp, the excluded and folders, no selector, the first name, one selector, new picks on each move, the gamelist unaffected |
| `WheelCarouselTests` | 21 cases | each logo's centre within 1.5 px of the centre read from ES-DE's capture, for 18 probe variants (vertical and horizontal wheels, rotation, origins, upright items, alignments, growth, clipping, offsets, reflections' room, the growth edge); and three reflections' fade by grey level |
| `SceneMappingTests` | 2 (331 cases) | the 25 new pairs change the pixels; the case list equals the mapping's |
| LunaP `CarouselWheelTests` | 6 | its §190.4 |

A wheel logo that ES-DE's own help bar covers is left out of `WheelCarouselTests`, since Mistress draws no help bar a
theme does not ask for. Two LunaP tests that read a `ScaleTransform` read the matrix now (its §190.2).

### 36.7 Mutants

The runner is `~/.cache/emusen/probe/pass14/mutate_pass14.py`, its log `run-pass14.log` and its verdicts
`mutants-pass14.txt`. Each mutant was built with `-m:2` and tested alone under `nice -n 10`: Mistress's against
`GameSelectorTests`, `WheelCarouselTests` and `SceneMappingTests`, LunaP's against its `CarouselWheelTests`,
`ThemedListTests` and `MotionTests` and then the same Mistress tests. Before changing a file the runner writes it to a
state file, restores it in a `finally` and touches it, and on starting restores any file a cut-short run left mutated;
the tree was rebuilt clean at the end and both worktrees were left with no change.

**30 mutants: 29 caught, 1 equivalent, none failed to build.**

| Area | Mutants (caught unless marked) |
|---|---|
| gameselector | G1 `lastplayed` keeps unplayed games; G2 `mostplayed` keeps never-played ones; G3 oldest first; G4 no clamp to `gameCount`; G5 the name sorting last; G6 excluded games picked; G7 folders picked; G8 `allowDuplicates` ignored; G9 random picks not moved on by navigation; G10 a system view without a selector shows the list's game; G11 a rating drawn with no game; G12 one selector's picks for every name |
| the scene's mapping | W1 `horizontalOffset` ignored; W2 `itemAxisHorizontal` ignored; W3 before and after swapped; **W4 reflections passed for a vertical carousel (equivalent)**; W5 `wheelVerticalAlignment` ignored |
| LunaP's carousel | L1 a horizontal wheel's origin read as a vertical one's; L2 turned anticlockwise; L3 upright items' arm through the origin; L4 no side shift; L5 no room for reflections; L6 a reflection that never fades; L7 a reflection that ignores its item's opacity; L8 no clipping; L9 a row grown from its centre; L10 the vertical offset by the width; L11 a wheel drawing the row's reach; L12 a reflection not flipped; L13 the hub not aligned |

- **W4 is equivalent.** LunaP draws reflections under a horizontal row only (its §190.3), so passing the property for a
  vertical carousel changes nothing drawn; Mistress's own check is a second guard, not the only one.
- **L7 was caught by one test alone**, `WheelCarouselTests`' reflection of an unselected item at half opacity, the
  measurement that found the squared opacity of LunaP §190.3; LunaP's own reflection test uses full opacity.
- **G12 was caught by `SceneMappingTests` alone**: `GameSelectorTests` names a selector only to show that an unknown
  or ignored name falls back.

### 36.8 Pictures

In `~/.cache/emusen/bigpicture/png/pass14/`: `mistress-<label>-<size>.png` from `Pass14PictureTool`
(`EMUSEN_BIGPICTURE_PNG=1`), and `side-<label>-<size>.png`, ES-DE's capture on the left and Mistress's render on the
right, for all 41 probe variants and 6 selector states at 1280×800, 4 probe variants and one selector state at
1920×1200, and the three downloaded themes in four states at both sizes. The captures themselves are in
`~/.cache/emusen/probe/pass14/captures/<label>/final.png`.

### 36.9 The downloaded themes

The wheels and the selectors' pictures land where ES-DE puts them in all three themes; what differs belongs to other
passes, and is listed so that P223 can be retired later:

| Theme and state | Pixels > 8 levels, 1280 / 1920 | Pass 14's elements | The rest |
|---|---|---|---|
| CodyWheel, system view | 18.2% / 16.6% | the wheel of logos and the gameselector's fan art match | the description's vertical scroll is at another moment; the help bar's entries; the info card's height |
| Aura, fullscreen system view | 2.5% / 0.4% | the horizontal wheel (a pager, `itemRotation` 0) matches | the help bar; "1 GAME" in ES-DE, "1 GAMES" in Mistress |
| Aura, game carousel | 20.3% / 19.1% | the reflections match | the items stand apart by `selectedItemMargins`, which is not mapped (§25.8's widest carousel gap); a glass panel behind the name is missing |
| Mania Menu, system view | 2.3% / 2.2% | the gameselector's screenshot matches | the list's names are placed at the start of each diagonal band in ES-DE and centred in Mistress |

### 36.10 Not done

- **`gamelistinfo` and `animation`**, the pass's third and fourth items, wait for the checkpoint's review.
- **`selectedItemMargins`, `selectedItemOffset`, `itemStacking`, `itemDiagonalOffset`, `itemAxisRotation`,
  `lineSpacing`, `imageCornerRadius`** and the carousel's other unmapped properties: not in Q48's four items (Q120).
- **A horizontal wheel's growth with a vertical alignment** (P221), **ties** (P222), and the **step's curve** (P220)
  were not measured.
- **Text items** of a carousel get no reflection; ES-DE was not run on a carousel of names with reflections.
- **Nothing ran on the handheld.**

### 36.11 Open questions

- **Q120, what follows.** The carousel's `selectedItemMargins` (32 themes) and `lineSpacing` (33) are wider gaps than
  any element left. Options: (a) `gamelistinfo` and `animation` as Q48 ordered; (b) those two carousel properties
  first, then (a). **Recommendation: (b).** Each is a small measured rule of the same control, and Aura's game carousel
  shows the first missing on the screen.
- **Q121, "unknown".** ES-DE writes "unknown" for a metadata field a game lacks, in a system view's selector as in a
  gamelist. Mistress writes nothing. **Recommendation:** Mistress's own word, through Pass 5's lookup, when that pass
  is built.
- **Q122, a default help bar.** ES-DE draws a help bar for a theme that has no `helpsystem` element; Mistress draws none.
  **Recommendation:** measure its place and draw Mistress's own, since a theme without one otherwise leaves the player
  without the buttons.

### 36.12 The broad run

WiseMan (then at 38324489, §35) was merged into the branch before the run. One run of the Mistress filter, without
`ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any GPU or Vulkan test, under `nice -n 10`:
**1,328 tests, 1,293 passed, 35 skipped (the picture, survey and live tools, `Pass14PictureTool` among them), none
failed, in 3 min 55 s.** LunaP's whole suite on its branch: 1,394 tests, all passed.

## 37. Pass 10 built: the screensaver (2026-09-27)

*Built on branch `bigpicture-pass10-screensaver`, from WiseMan at `b0c6bf75`, with LunaP's branch `pass10-screensaver`
from `openemu-library` at `9872708`.* §21.3 planned Pass 10 as ES-DE's screensaver: Dim, Black and Slideshow, after an
idle time, with its controls and "Start screensaver after", and Video left for Pass 12. Q34 was decided on 2026-09-26
(§10.1): on everywhere, Dim until videos exist, and off in Game Mode if the hardware session finds Steam's dimming
stacking with it. Decided 2026-09-27: that Game Mode switch is a setting, on by default, and the question stays open
(Q130). P114 and P39 set the pass's rule: a screensaver draws nothing between its changes. The player's account is §4.75
of the settings reference.

**Sources.** ES-DE 3.4.1's behaviour, measured by running it, and its `USERGUIDE.md` ("Screensaver", "Screensaver
settings", "Slideshow screensaver settings") where it speaks; never its source. The guide names the four types, the
controls, the overlay's content, the fallback to Dim "if no game images are available", and the settings and their
ranges. It gives no level, time, layout or image order; those are measured here. No file of ES-DE's entered the
repository.

**Numbering.** Predictions from **P230**, questions from **Q130**; both ranges were free on WiseMan when this began.

### 37.1 The setup

- **ES-DE.** The AppImage under `~/.cache/emusen/bigpicture/esde/`, with a home of its own, `home-screensaver/`. Each run:
  - under the shared lock, `flock ~/.cache/emusen/bigpicture/esde/esde.lock`, so one ES-DE window exists at a time;
  - `--home home-screensaver --resolution 1280 800 --fullscreen-padding off --no-update-check --no-splash --debug`,
    windowed, and 1920×1200 for two runs;
  - closed by the PIDs it started, and the run waits until they are gone before it gives the lock back.
- **What it could reach.** `ApplicationUpdaterFrequency` `never`, the ScreenScraper fields empty, nothing scraped. The ROM
  folder was `esde/screensaver-roms/`: ten empty `.sfc` files and three empty `.nes` files. The runner asserts that no
  folder it names lies in the tester's library. `SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT` admitted only §22.2's uinput
  pad.
- **Media.** Pictures drawn for the probe (§33.1's `mkmedia.py`): each kind its own colour with its name on it, so a
  capture shows which kind was chosen. One game had every kind, one every kind but miximages, and single games had only
  a screenshot, a title screen, a cover, fan art, a marquee, a miximage or a 3D box; one had none. The NES games had
  titlescreens and covers, covers and titlescreens, and covers, fan art, 3D boxes and marquees. Four games were
  favourites (`gamelist.xml`). The custom folder `esde/screensaver-custom/` held four flat pictures of different
  shapes (1600×400, 400×1200, 160×100, 1280×800) and one in a subfolder.
- **The timer.** `ScreensaverTimer` 4000 ms for every run, written to `es_settings.xml` directly (the menu offers whole
  minutes). One run set 0 and started the screensaver with X.
- **Capture.** `ffmpeg -f x11grab` recordings of the window at 60 fps (fades) and 30 fps (slideshows, controls), and
  single frames; the pad's presses are logged on the wall clock. Per-frame luma and change, and pixel reads of known
  colours, are from `frames.py` and short scripts.
- **The runner** is `~/.cache/emusen/probe/pass10/esde_run.py`, with `batch.py`, `pad2.py`, `mkmedia.py`, `flat.py`,
  `frames.py` and `overlap.py`. Captures are under `probe/pass10/captures/`, logs under `probe/pass10/logs/`.
- **The runs.** 24, in two batches after a first single run, and three repeats (below): 27 in all.
- **A run that was not isolated, and the repeats.** The first two runs (`d01-dim`, `b01-black`) killed only the process
  tree of the AppImage. After `b01` a process of ES-DE was still running while the lock had been given back: the next
  ten starts found it and refused to run. The runner then learnt to record every ES-DE process present once its window
  appears and to wait for all of them to end. `overlap.py` compares every run's log with the logs of the other
  work using ES-DE that day: no other run started inside any of these. Dim, Black and the slideshow's change were then
  recorded again (`d01r`, `b01r`, `s03r`) with the runner watching for any other ES-DE process during the run; there
  was none, and the three repeats agree with the first recordings to within a frame.

### 37.2 What ES-DE does

**The timer.** The screensaver starts 4.0 s after the last press with the timer at 4000 ms, measured on seven runs
(the first changed frame 4.0–4.1 s after the press, the recording's own start delay included). With a menu open it
did not start in ten seconds. With the timer at 0 it did not start by itself in the 8.6 s after X had ended it.

**Dim** (`d01`, `d01r`, `d03`, `d02` in the system view, `d05` at 1920×1200):
- **The level.** Every pixel becomes the grey of its luminance at 0.4 of it: (40, 161, 61) became (45, 45, 45), where
  0.4 × (0.299 R + 0.587 G + 0.114 B) = 45.4; (40, 80, 201) became (33, 33, 33); white 255 became 102 and 250 became
  100; the background's 55 became 22. The whole window is dimmed, the help bar, the clock and the indicators included.
- **The fade.** Both together: brightness and saturation fall along one ramp. The white text read 0.98, 0.94, 0.87,
  0.81, 0.70, 0.64, 0.58, 0.48, 0.41 and 0.40 of itself frame by frame, a fall of 3.5–3.6 a second, so the fade takes
  0.17 s (four recordings: 0.167–0.19 s). Halfway (brightness 0.71) the colours were halfway to grey.
- **What is under it** holds still.

**Black** (`b01`, `b01r`, `b02`): no desaturation, the picture darkened to black at 7.1–7.7 a second, 0.13–0.14 s.

**Video with no videos** (`v01`): Dim, to the level. **A slideshow with no pictures** (`s16`, an empty media folder):
Dim.

**Waking.** Any button ends it in the next frame (60 fps recordings), with no fade out, and does nothing else: Down did
not move the list, A in Dim did not start the game, Y did not toggle the favourite, right did not change the system,
B did not leave the list (`d01`, `d04`, `s06`, `s17`). The next screensaver came 4 s after that press.

**The slideshow** (`s01` at 10 s, `s02` at 2 s, `s03` and `s03r` at 4 s and 60 fps):
- **The swaps** came exactly every 10.0 s (three swaps, 300 frames apart at 30 fps), and every 2 s and 4 s when set.
- **Each change.** A cut to black in one frame, the old picture and overlay gone; the overlay then fades in and is
  whole about 0.12 s after the cut; the picture is not drawn at all until 0.22–0.23 s after the cut, appears at about
  half its brightness, and rises linearly to full at 0.45 s. Six changes read the same. Read as one ramp, the picture's
  opacity is its time since the cut over 0.45 s, hidden for its first 0.22 s.
- **The first showing** is the same: the view cuts to black, then the picture fades in.
- **Which picture.** Over 20 changes at 2 s: the game with every kind showed its miximage; the one without miximages,
  its screenshot; a game with a title screen and a cover, the title screen; a game with a cover, fan art, a 3D box and
  a marquee, the cover. **Games with only fan art, only a marquee, only a 3D box, or nothing, never appeared.** The
  order is miximage, screenshot, title screen, cover; the other kinds are not used.
- **Which game.** Random, from both systems, and never the same game twice running in over 40 recorded changes (`s02`, `s04`,
  `s10`, `s11`, `s12`).
- **The fit.** Fitted whole and centred on black (a 4:3 picture at full height with bars at the sides); with *Stretch
  images* on, stretched to the window (`s14`).
- **The overlay** (`s01`, `s02`, `s14`, `s15`):
  - a black box of alpha 0xAA (a picture's (200, 160, 30) under it read (67, 53, 10), white read 85), in the upper
    left corner, at x 17 and y 16, 84 px high at 800 lines; at 1920×1200, 1.5 times each;
  - two lines of white capitals: the game's name, then the system's `fullname` from `es_systems.xml`
    (*NINTENDO SNES (SUPER NINTENDO)*); the capitals 19 px high, their tops 12 and 54 px below the box's top, the text
    14 px in from the box's left;
  - the box as wide as the longer line plus about 16 px;
  - a filled white star after a favourite's name, about 25 px across, 13 px after the name, centred on the capitals;
  - with *Only include favorite games* on, no star (`s10`), as the guide says;
  - the face is the one ES-DE's menus use, which §32 matched with Barlow Condensed.
- **Custom images** (`s11`, `s12`): the folder's pictures, at random and never twice running, with no overlay; the
  subfolder's only with *recursive search* on; each fitted whole (the 160×100 picture scaled up to the window).
- **The controls** (`s04`, `s05`, `s06`, on by default):
  - right and left: an immediate change to another game, with the same cut and fade, and the next swap 10 s after;
  - A: the gamelist behind moved to the game shown, and the game started through the launch screen, 3.017 s after the
    press (§33's Normal);
  - Y: the screensaver ended and the gamelist showed that game selected;
  - B: the screensaver ended, nothing else;
  - with the controls off (`s17`), right, A and Y only ended it.
- **X in the system view** (`x01`, timer 0): the slideshow started at once; X while it showed ended it. The system view's
  help bar names X *Screensaver*.

### 37.3 P114 retired

P114 predicted that Dim and Black draw at most one frame after they start, and that the slideshow draws only at its
swaps and their transitions.
- **The first clause failed, for ES-DE itself.** ES-DE fades Dim in over 0.17 s and Black over 0.14 s: ten and eight or
  nine frames at 60 Hz. The prediction assumed a cut. Mistress follows ES-DE, so it draws those frames: **11 for Dim
  and 9 for Black** in the harness's 16 ms polls, then **0 in the 610 s after**, and the themed view beneath 0 as well.
- **The second clause held.** Each change draws through the overlay's fade (to 117 ms) and the picture's (217–450 ms),
  with one wake at 217 ms between them: **23 or 24 frames a change, and 0 in the 9.54 s to the next**, over six changes.

The rule the pass was meant to keep, nothing drawn while nothing changes, holds. What failed is the belief that the
holding is all there is.

### 37.4 What was built

- **LunaP** (its §184), on its branch `pass10-screensaver`:
  - `DimLayer`, which greys what is beneath it by the renderer's own saturation blend mode and darkens it with a black
    fill, so nothing beneath is captured or resampled. In Avalonia 12.1 the blend mode reaches images and not fills, so
    the grey it blends is a 2×2 constant image; a grey fill drew the grey itself. The luma is the blend mode's
    (0.3, 0.59, 0.11), within one level of ES-DE's;
  - `CrossFadeImage`, two `FittedImage`s at high-quality sampling, the new one faded in over the old or over nothing.
- **`Screensaver`** (Mistress, `BigPicture/Screensaver.cs`), a panel of a `DimLayer`, a black fill, a `CrossFadeImage` and
  the overlay (a box, two `FontText` lines in the menus' face, and a star). It holds §37.2's levels, times and
  proportions, and answers when it next changes (`NextChange`): now while a fade runs, the picture's first showing
  while only that is awaited, and never once it is still.
- **`MainWindow.Screensaver.cs`**:
  - **The idle clock** is the interface clock (`UiClock`), which tests move. Each pad poll (`PadTick`, every 16 ms)
    reads every button; anything held, any key, any pointer movement or press, and any moment when it may not start,
    moves the idle time to now. The poll runs whether or not a pad is connected.
  - **It may start** in big picture (a big-screen session or F10's) with the library on screen and no game, pad menu,
    sheet, on-screen keyboard, launch screen or other active window, and, with the Game Mode switch off, not in a
    gamescope session.
  - **Frames.** A frame is asked for only while it changes; otherwise one `DispatcherTimer` waits for the next change or
    swap, and with nothing to come nothing is scheduled. The pad poll steps it too, so a test's clock reaches it. While
    it shows, the themed view's own scheduling (§15.7) is off and its frame does nothing.
  - **Input.** The press that wakes it is consumed, and the button stays consumed until it is let go (the pad
    navigator's `Forget`, as the launch screen does). A key is consumed before anything else reads it.
  - **The slideshow's pictures** come from the library's sources in §4.60's order (`MediaSources.Locate`), each game's
    first of miximage, screenshot, titlescreen and cover. A game is looked up only when it is drawn at random, and one
    with none of the four is dropped from that showing's list, so a large library costs one lookup per shown game.
  - **The controls** call the themed view's new `ShowGame` (its system's gamelist, in the folder the game sits in, with
    it selected) and the start path of §33 (`StartGameAsync` with the game, so the resume question and the launch
    screen come first).
- **X.** `UiButton` gains `Screensaver`, the pad's West button and the keyboard's Delete. The system view's help bar
  gains ES-DE's *Screensaver* entry on X while the controls are on (`HelpContext.Screensaver`).
- **The settings.** `BigPictureInterface` gains ES-DE's eleven screensaver and slideshow keys that this pass uses and
  Mistress's `ScreensaverInGameMode`; the Interface tab gains a *Screensaver* group (§4.75).
- **`ThemedSession`** sets the timer to 0, so the older themed tests are never interrupted.

### 37.5 Where Mistress differs, on purpose or by necessity

- **The default type** is Dim, not ES-DE's Video (Q34), which with no videos shows Dim anyway.
- **The system name** in the overlay is the shelf's full name (*Super Nintendo*), as on the launch screen (§33.5), not
  ES-DE's `fullname` (Q131).
- **The star** is Mistress's own five-pointed path, of ES-DE's size and place.
- **A pointer** moving or pressing counts as activity, and a press wakes it; ES-DE's pointer was not tried.
- **Delete** is X on the keyboard, as ES-DE's default keyboard map gives it; ES-DE's keyboard was not tried.
- **The timer's slider** offers whole minutes 0–30, as ES-DE's menu; a value set in milliseconds by hand is kept but
  shows rounded down.
- **Not offered:** *Render scanlines*, `%ESPATH%` in the custom folder, and every Video setting.
- **The fades** follow ES-DE's measured lengths as straight ramps; ES-DE's first frame or two of a fade are a little
  slower than the rest (0.98, 0.94 before steps of about 0.06), which Mistress does not copy.
- **The picture's hidden 0.22 s** is copied as measured. Whether ES-DE hides it on purpose or is loading the picture
  in that time is not known; it was the same for a 600×800 cover and a 1920×1080 picture, and on every change.

### 37.6 Tests

`ScreensaverTests`, 28 cases (with the theories' rows), headless on WiseMan's `PadDriver` with the clock the test moves
(§4.75 lists them). **The frame counts:**
- Dim: 11 frames through its 167 ms fade, then 0 in 610 s; Black 9, then 0; the themed view 0 throughout.
- The slideshow: 23–24 frames through each of six changes, 0 in the 9.54 s after each, the themed view 0.
- A theme with scrolling text: the view asks for frames before; while the screensaver shows it asks for none and draws
  none in 30 s; once woken it asks again.

**Pixel cases:** every sampled pixel of Dim within 1 level of 0.4 times its luma at 1280×800 (4,267 coloured samples)
and 1920×1200 (9,095), Black all black; the overlay's box at 17, 16, 84 px high at 800 lines and 1.5 times each at 1200,
its colour a third of the picture's, white text inside it and the picture's own colour just outside; a 4:3 picture's
black bar and the stretch filling it.

LunaP's `ScreensaverPieceTests`, six cases, are LunaP's §184.3.

**Existing tests.** The broad run (§37.11) covers the pad's new button and the help entry. No existing test changed
except the fixtures: `ThemedSession`'s timer and `PadDriver.Press` for the new button.

### 37.7 Mutants

The runner is `~/.cache/emusen/probe/pass10/mutate_pass10.py`, its verdicts `mutants-pass10.txt` and
`mutants-pass10-rerun.txt`.
- Each mutant was built with `-m:2` and tested alone under `nice -n 10`: WiseMan's against `ScreensaverTests`, LunaP's
  against `ScreensaverPieceTests`.
- Before changing a file the runner writes `mutant-state.json` (the file and a copy), restores it in a `finally` and
  touches it, and on starting restores any file a cut-short run left mutated. Both trees were rebuilt clean at the end.

**44 mutants: 40 caught on the first round, 4 survived, none failed to build.** Two survivors were weak tests, fixed and
then caught; two are equivalent. Each mutant took about 15 s.

| Area | Mutants (caught unless marked) |
|---|---|
| Timing and levels | T1 Dim's fade 300 ms; T2 Black's 300 ms; T3 Dim at 0.5; T4 the picture from the cut; T5 its fade 300 ms; T6 the overlay at once; L1 Dim not greying; L2 Black greying; L3 the box at 0x80; L4 the box 30 px in; L5 the stretch ignored |
| Starting | I1 a poll late; I2 a held button not counted; I3 under the pad menu; **I4 under a sheet (survived: equivalent)**; I5 the Game Mode switch ignored; I6 a key not counted; I7 0 not never; **I8 under a game (survived: equivalent)** |
| Waking | W1 the waking button acting once let go; **W2 the waking key passed on (survived, then caught)** |
| Frames | F1 the slideshow always moving; F2 Dim asking for frames while it holds; **F3 the view scheduled under it (survived, then caught)**; F4 the view drawn under it |
| The slideshow | C1 screenshots before miximages; C2 the same game twice; C3 favourites only ignored; C4 the star among favourites only; C5 the overlay switch ignored; C6 subfolders always; C7 Video as Black |
| Controls | K1 the switch ignored; K2 right not changing; K3 Y not going to the game; K4 A without the launch screen; K5 X from a gamelist; K6 no help entry |
| Other | X1 the window's close leaving its timer; S1 the timer stored in seconds |
| LunaP | D1 the saturation as a fill; D2 the black at `Brightness`; D3 `Show` keeping the old picture; D4 `Progress` not clamped |

- **W2 survived** because the only key the test woke it with was Down: passed on, it reached the themed view as a held
  button, and the button-held-after-waking rule consumed it anyway. The test now wakes it with F10 as well, which passed
  on would leave big picture; caught on the rerun.
- **F3 survived** because the synthetic theme at rest has nothing to schedule, so a view still scheduled under the
  screensaver asked for nothing either. The new case uses a theme with scrolling text; caught on the rerun.
- **I4 and I8 are equivalent.** A presented sheet is also the window's `OtherWindow()`, and a game on screen hides the
  library view, so each removed condition is implied by one that remains. Both are kept as the reader's statement of
  where it may not start.

### 37.8 Pictures

`ScreensaverPictureTool` (with `EMUSEN_BIGPICTURE_PNG=1`) writes to `~/.cache/emusen/bigpicture/png/pass10/`, beside
ES-DE's captures in `~/.cache/emusen/probe/pass10/captures/`. Every picture was looked at. At 1280×800 and 1920×1200:
- the view before;
- Dim at 0, 50, 100, 167 and 5,000 ms; Black at 0, 70 and 140 ms;
- the slideshow at 0, 60, 117, 200, 217, 300, 450, 5,000, 10,000 and 10,500 ms (a favourite with its star at 10.5 s);
- the slideshow without the overlay, and stretched;

and the Interface tab's *Screensaver* rows at 1280×800, in two pictures.

Set beside ES-DE's frames at the same sizes, the overlay's box, lines and star fall in the same places at the same
sizes. The first letter's capitals start at x 32 and row 28 at 1280×800 (ES-DE: 31 and 28) and the second line ends
on row 88 (ES-DE: 88); at 1920×1200, x 48, row 42 and row 133 (ES-DE: 47, 42, 135), and the box is 24 px down and
126 high, ES-DE's own. The first round of pictures showed
the names cut short with an ellipsis: the lines were measured with the width the previous layout had given them, so a
longer name never widened its box. The lines now measure to their text, and the pictures were taken again.

### 37.9 Predictions

Written after ES-DE was measured and before any of this was run where it is not yet.

| # | Prediction | Found, or retired when | Verdict |
|---|---|---|---|
| P230 | In Game Mode on the handheld, Steam's own dimming starts over a big-picture window that draws nothing, within its own idle time, and stacks with Dim (the screen darker than 0.4) | the hardware session | open |
| P231 | On the desktop's real window, a held Dim draws no frame in 60 s by the compositor's own count, as the harness's count says | a real window with a frame counter | open |
| P232 | ES-DE's picture is hidden for 0.22 s whatever its size, because the hiding is part of the fade and not its loading: a 4000×3000 picture shows the same timing | one more ES-DE run | open |

### 37.10 Not done

- **Nothing ran on the handheld**, in Game Mode or out of it (P230, Q130), and no real pad was used.
- **Video** and its settings wait for Pass 12. *Render scanlines* is not built.
- **ES-DE's pointer and keyboard** were not tried with its screensaver.
- **The desktop's own library** outside big picture has none, as ES-DE's desktop has no such view.
- **A real window's frame count** (P231): the counts above are the frames Mistress asks for, which in the headless
  harness is what is drawn.

### 37.11 Open questions

- **Q130, the Game Mode switch.** Q34's condition, Steam's dimming stacking with Mistress's, is not yet measured.
  **Recommendation:** keep *In Game Mode* on until the hardware session shows the two stacking (P230); then turn the
  default off.
- **Q131, the system's name in the overlay.** ES-DE writes its `es_systems.xml` full name (*Nintendo SNES (Super
  Nintendo)*); Mistress the shelf's (*Super Nintendo*), as on its launch screen. **Recommendation:** keep the shelf's,
  so the two screens agree.
- **Q132, render scanlines.** ES-DE's slideshow can draw scanlines over its pictures (off by default). **Recommendation:**
  build it with the screen filters' CRT scanline pass when Pass 12 builds the video screensaver's, which has the same
  option on by default.

### 37.12 The broad run

After merging WiseMan (§35, the value types), one run of the Mistress filter, without `ShaderSettingsWindowTests`,
`ShaderBrowseBench`, `SceneGpuBench` and any GPU or Vulkan test, under `nice -n 10`: **1,324 tests, 1,290 passed, 34
skipped (the picture, survey and live tools), 0 failed, in 3 min 48 s.** §35.12's order-dependent failure of the
headless platform's initialisation did not recur in this order. The big-picture tests alone (516, the GPU ones and the
benches left out) had passed before the merge, with the pad's new button and the system view's new help entry.

## 38. Pass 8 built: ScreenScraper's extras (2026-09-27)

*Built on branch `bigpicture-pass8-scraper-extras`, from WiseMan at `7ba6fef0`.* §21.3 planned Pass 8 as the rest of
what ES-DE's scraper offers and Mistress lacked: the kinds of inventory row 22 as switches, off by default, with video
only if Q20 allowed it; a Refresh that asks a kept file again by its checksum (row 21); a search by name and a chooser,
started only from *Find by name…* (row 20); ScreenScraper's names behind a switch (row 23, Q30); the criteria of row 24
that Mistress can answer; and the media of deleted games (row 34). The answers that bind it are in §10.1: every §21
recommendation accepted, and Q20 answered "all three, late", so clips are scraped here, off by default, and played in
Pass 12. §17.14's two rules bind all of it: nothing is asked of any server until the player starts it, and the player
chooses the scope. The player's account is §4.76 of the settings reference.

**Sources.** ES-DE's `USERGUIDE.md` ("Scraping", "Scraping process", "Manually copying game media files", "Scraper" and
its "Content settings" and "Other settings", "Removing orphaned data"), in the copy of §21; ES-DE 3.4.1's own
`es_settings.xml` defaults, read from the stage (b) scratch home; ScreenScraper's API page as §5.1 cites it; and the 43
redacted answers of §17.9's live run, read again for the new kinds' offer rates and sizes and for the checksums each
media entry carries. ES-DE was not run for this pass, and nothing was asked of ScreenScraper or any other server.

**Numbering.** Predictions from **P240**, questions from **Q140**; both ranges were free on WiseMan when this began.

### 38.1 What the answers already said (measured 2026-09-27, read-only)

`~/.cache/emusen/probe/pass8/rates.py` reads §17.9's 39 found answers (the files under
`~/.cache/emusen/bigpicture/scrape-live/20260926-134821/answers/`, redacted when they were written) and, for each kind,
counts the games that offer it and takes the size ScreenScraper states for the file the region rule would choose (the
US, then world, EU, Japan and `ss` file, else one with no region, else the first):

| Kind (ES-DE type, ScreenScraper name) | Games offering it | Mean size | Median size |
|---|---|---|---|
| cover (`box-2D`) | 33 of 39 | 653 KB | 594 KB |
| screenshot (`ss`) | 36 | 5 KB | 3 KB |
| marquee (`wheel-hd`, `wheel`) | 36 | 90 KB | 77 KB |
| title screen (`sstitle`) | 36 | 6 KB | 3 KB |
| miximage (`mixrbv2`) | 39 | 565 KB | 597 KB |
| back cover (`box-2D-back`) | 33 | 468 KB | 397 KB |
| 3D box (`box-3D`) | 33 | 351 KB | 295 KB |
| physical media (`support-2D`) | 33 | 449 KB | 427 KB |
| fan art (`fanart`) | 25 | 397 KB | 267 KB |
| manual (`manuel`) | 27 | 2,254 KB | 1,209 KB |
| video (`video-normalized`) | 32 | 1,354 KB | 1,343 KB |

§21.1's table counted every file of a kind, several regions each; this one counts the one file a run would fetch, which
is what a run costs. **Every media entry carries its checksums:** of the 2,062 entries in the 39 answers, 2,062 have
`crc`, `md5` and `sha1`, and 2,032 a `size`. That decided the design of the refresh (38.3).

### 38.2 What was built

- **Six kinds** (`ScrapeRules`): back covers, 3D boxes, physical media, fan art, manuals and videos, filed in ES-DE's
  folders (`backcovers`, `3dboxes`, `physicalmedia`, `fanart`, `manuals`, `videos`) under §30's foldered layout. Each
  kind says what its file must be (`MediaPayload`): a picture by its content type, as before; a manual by the `%PDF-`
  it starts with; a clip by MP4's `ftyp` box or a video content type, since what ScreenScraper sends for either was
  never seen. A manual is written `.pdf`, a clip `.mp4`. Downloads now stream to the `.part` file rather than through
  memory, capped at 128 MB, with ES-DE's transfer timeout (`ScraperTransferTimeout`, 120 s) plus a second for every
  64 KB the answer declares, since the largest manual seen was 18 MB and the account's rate is 128 KB/s.
  `EsdeMediaFolder` learnt `manuals` (`.pdf`), so an ES-DE folder's manuals and the store's are found alike.
- **What an older answer knows** (`media.db` migration 3, `kinds_known`). §17.4 asked a found game again only for a
  kind "the player has since turned on and the game offered". An answer kept before this pass listed only the first
  five kinds, so it says nothing of fan art; read as "not offered", no game scraped before today would ever get it.
  `ScrapedRecord.MayOffer` reads a kind outside an answer's `kinds_known` (the first five for an old row) as possibly
  offered, so such a game is asked once, and an answer kept now records every kind and costs nothing for a kind it
  lacks.
- **Refresh** (38.3).
- **Find by Name…** and its chooser (38.4).
- **The criteria** (38.5) and **the plan's prices** (38.6).
- **Game Names** (Q30): off by default; on, the library's lists, the themed view and the editor's baseline take
  ScreenScraper's name wherever it has one (`GameMetadata.Scraped` gains the name case only with the switch on). A name
  the player gave still wins, and with the switch off the editor's offer of §27.2 is unchanged.
- **Orphaned media** (38.7).
- **Settings:** a *Fetch More* row of six switches and a *Game Names* row in Preferences ▸ Scraping; the *Only games
  with no cover* switch became a dropdown of criteria; *Refresh what is kept*; an *Orphaned Media* row. All stored in
  `appsettings.json` (`ScrapeBackCovers`, `Scrape3DBoxes`, `ScrapePhysicalMedia`, `ScrapeFanArt`, `ScrapeManuals`,
  `ScrapeVideos`, `ScrapeGameNames`, `ScrapeCriteria`, `ScrapeRefresh`); nothing new in `games.db`.
- **The status window** names the new kinds and counts files kept unchanged.

### 38.3 Refresh: the checksum in the answer, then the checksum in the request

ES-DE's *Overwrite files and data* (on in its defaults) re-scrapes and replaces. §21.3 planned Mistress's as a request
per kept file carrying its checksum, which ScreenScraper answers `SHA1OK` without the bytes (§5.1). Since every entry
of every answer already states the file's `sha1` (38.1), the built rule is cheaper:

1. A found game is asked again: one `jeuInfos`, whose text replaces the kept text in `media.db`. A player's edit lives
   in `games.db` and still wins (§23.3).
2. For each kept file, its SHA-1 is computed from the file itself and compared with the answer's. Equal: kept, **no
   request**. The store's own SHA-1 column is not trusted for this, because a file found already there was recorded
   without one.
3. Different, or no checksum in the answer: the file is asked for with `sha1=<the kept file's>` added, so an unchanged
   file comes back as `SHA1OK` (also `MD5OK` or `CRCOK`) and costs a request and no bytes. A changed one is streamed
   beside the kept file and moved over it only once it has passed the kind's checks; a kind whose extension changed (a
   PNG become a JPG) is written under its new name and the old file removed.
4. A game that answers 404 on a refresh keeps what it had, text and files. §17.4's 404 rule, "recorded Unknown", now
   applies only to a game never found: a later 404 no longer turns a found game Unknown, whether the run was a refresh or
   was asking for a kind turned on since.

The refresh is off by default, a switch beside the criteria (`ScrapeRefresh`), and applies to *Scrape This Game* too,
as ES-DE's single-game scraper follows its overwrite setting. Decided 2026-09-27: off, where ES-DE's default is on,
because §17.14 makes every run the player's and a refresh re-asks every found game of the scope; each is one request.

### 38.4 Find by Name…

- **Where.** A game's options (the themed gamelist's Select, the sidebar library's *Game Options...*) and the library's
  context menu, beside *Scrape This Game...*; neither offers it while a run goes. It opens as a LunaP window on the
  desktop and a sheet in a big-screen session, like the cover picker (§28).
- **What it sends.** Nothing when it opens. The box holds the file's name as ES-DE strips it ("Scraping process": the
  extension and every `(...)` and `[...]` removed; underscores become spaces, ES-DE's `ScraperConvertUnderscores`, on
  in its defaults). **Search** sends one `jeuRecherche.php` with the credentials, the console's `systemeid` and the
  text, paced by the quota manager like any request and counted in the day's requests; a search that finds nothing is
  counted as unrecognised as well, as §21.3 assumed (P242). The answer is read as `jeuInfos`' game, up to 30 of them;
  an entry with no id is no game (P241).
- **The chooser.** A list of names in the region order (§17.6), each with its year and publisher, and the platform in
  brackets when ScreenScraper returned another, as ES-DE's list does. **No thumbnails**: each picture would be a
  request (§17.9), thirty for one search (Q140).
- **A pick** starts a one-game run in which the pick stands in for the lookup: no `jeuInfos`, the game's text kept for
  this file with `matched_by` `name` and its id, and each wanted kind fetched through the same worker, pacing and quota
  as any run, with the kept files replaced as a refresh would. The file is never renamed and nothing is written beside
  it. A name-matched game asked again later is asked with `gameid` as well as its hashes (a documented input of
  `jeuInfos`, §5.1), and a 404 leaves it as it is.
- **Never automatic.** Nothing falls back to a name search: not a run, not a 404, not the failover (§5.2, §17.14).
  ES-DE's interactive runs, which stop on each unmatched game to ask, are not built (Q141).

### 38.5 The criteria

USERGUIDE's *Scrape these games*: All games, Favorite games, No metadata ("checks if the game has a description"), No
game image ("checks for a miximage, then screenshot, then title screen and last box cover"), No game video, and Folders
only. Mistress answers the first five from what every view shows: the favourite in `games.db`; the description as
§4.59 resolves it (an edit, else ScreenScraper's); the four pictures and the video by the order of sources of §4.60, so
a player's cover, another game's chosen cover, an ES-DE folder's and OpenEmu's all count. *Games with no cover*, §17.14's
switch, stays as a sixth choice and the default. **Folders only** is not offered: it means scraping folders themselves
(ES-DE's *Scrape actual folders*), and this library's folders are regions and letters, not games (§30; Q143). A single
game is one game whatever the criteria say, and *Exclude from multi-scraper* (§23) still leaves a game out of a wide run.

### 38.6 The plan's prices

The confirm step keeps its bound, one lookup and one request per kind for each game asked, and now adds what 38.1's
offer rates expect: the requests (one lookup plus each wanted kind's offer rate), the megabytes (each kind's rate times
its mean size), and a time of 3.9 s of requests a game plus the bytes at the account's `maxdownloadspeed` (128 KB/s
until an answer says). The 3.9 s is §17.9's median found game, 13.3 s, less its 9.4 s of bytes at 128 KB/s; with the
default kinds the model gives 13.3 s back. A found game wanting a kind it may offer and lacks is counted as asked
again, and under a refresh every found game of the scope is. By the model, for this library of 5,520 files at 9,800
requests a day (10,000 less §17.4's 2%):

| Kinds | Requests a found game | MB | Time at 128 KB/s | The library |
|---|---|---|---|---|
| the defaults (cover, screenshot, marquee, miximage) | 4.69 | 1.18 | 13.3 s | 25,900 requests, 2.6 days |
| and the four new pictures | 7.87 | 2.47 | 23.7 s | 43,500, 4.4 days |
| and manuals | 8.56 | 4.00 | 35.9 s | 47,300, 4.8 days |
| and videos | 8.69 | 3.56 | 32.4 s | 48,000, 4.9 days |
| all eleven, title screens too | 10.31 | 5.09 | 44.6 s | 56,900, 5.8 days |

§17.9 measured 4.7 requests a found game with the defaults; the model's 4.69 agrees, as it should, since both are
counts of the same answers. A refresh of a library whose files are unchanged costs one request a game.

### 38.7 Orphaned media

ES-DE's *Orphaned data cleanup* ("Removing orphaned data") moves the media of games no longer present to
`downloaded_media/CLEANUP/<date_time>/`, keeping each file's path, and removes media folders it leaves empty; it processes
only the systems that are enabled. Mistress's *Clean Up...* (Preferences ▸ Scraping ▸ *Orphaned Media*) does the same to
its own store and nothing else:
- a file of the store's type folders is orphaned when no game of the last library scan has its name there (its stem,
  under its folder since §30); a console with no game in the scan is not looked at, so an unplugged drive or an empty
  ROM folder never reads as every game gone, and nothing is looked at when the ROM folder is missing;
- the count and the size are asked first; declined, nothing moves;
- each file is moved to `home/Media/CLEANUP/<yyyy-MM-dd_HHmmss>/<its path>`, never deleted, with a `cleanup.txt` listing
  them; its `media.db` row goes; a folder the move leaves empty goes too;
- the ROM folder, an ES-DE media folder the player chose, the cover art folder and OpenEmu's folder are never touched,
  and no request is made. Deleting `CLEANUP` is the player's.

### 38.8 Where Mistress differs, on purpose or by necessity

- New kinds, videos, Game Names and Refresh are **off**; ES-DE's defaults have all on (`es_settings.xml`). The kinds and
  names were decided in §21.5 (Q20, Q30); Refresh here (38.3).
- **No Ratings and Other metadata switches** (ES-DE's content settings): the text comes in the same answer as the
  lookup, so leaving it out saves nothing, and the editor already overrides any field (Q142).
- **No interactive mode, no auto-accept, no Folders only, no Scrape actual folders** (Q141, Q143).
- **The manual and the clip are stored, not shown.** The media viewer is Pass 9 and video Pass 12.
- **The cleanup covers media only.** ES-DE's also removes gamelist and collection entries; Mistress's `games.db` keeps
  its rows of games gone, as §4.37's orphan pass decides.

### 38.9 Request counts, from the tests

Every count is the fake server's own (`FakeScreenScraper.Asked`), never the network. The fake now answers
`jeuRecherche`, states each media file's SHA-1 and size as the live answers do, answers `SHA1OK` for a matching
checksum, serves a PDF for `manuel` and an MP4 for the videos, and can change a kind's bytes (`MediaRevision`) or stop
stating checksums (`StateChecksums`).

| Case | Requests |
|---|---|
| a found game, the defaults | 5 (1 lookup, 4 pictures) |
| a found game, all eleven kinds | 12 |
| a found game refreshed, nothing changed | 1, no media bytes |
| the same, where the answer states no checksum | 5, each picture answered `SHA1OK`, no media bytes |
| refreshed with the cover changed upstream | 2; only the cover replaced |
| found without refresh, the cover changed upstream | 0 |
| fan art turned on later: a game that may have it / one whose answer says it has none / one kept before this pass | 2 / 0 / 2 |
| Find by Name: the search / the pick, the default kinds | 1 / 4 more, no lookup |
| the pick asked again later for back covers | 2, the lookup carrying `gameid` |
| three searches at 6 a minute | 3, at least 11.1 s apart after the first answer set the pace |
| twelve requests of all eleven kinds at 6 a minute | 12, spanning at least 111 s of the scrape clock |
| a cleanup, declined and then accepted | 0 |

### 38.10 Tests

Headless through WiseMan, on the fake ScreenScraper; no window of a real platform, no GPU.
- `ScrapeExtrasTests` (18, one a theory of four file names): each new kind into its folder and file type, one request each; the defaults costing what they
  did and the offer recorded for every kind; a kind turned on later asking only a game that may have it (offered, not
  offered, kept before this pass); a `media.db` of schema 2 opening with its rows and `MayOffer` reading the old row;
  the payload checks (PDF, MP4, a page that is neither); Refresh unchanged, changed, without the answer's checksums,
  and on a 404; a pick with no lookup, then asked by `gameid` and never made Unknown; the search's URL, 30 of 36, and
  `[{}]` as none; the search text of four file names; the pacing of twelve requests.
- `ScrapeExtrasWindowTests` (13): Find by Name sending nothing until Search, one request, the pick's run with no lookup
  and the file untouched; a search finding nothing counted as unrecognised; no developer file and ScreenScraper
  switched off sending nothing; the searches paced; the entry in the options and the context menu and not during a run;
  the sheet walked by the pad (`PadAudit`), every control and every result reached, a result chosen with A; each
  criterion's count on a six-game library; the plan's bound, expected requests and megabytes, and a refresh priced and
  run; the Scraping tab's new switches off, saved, and the criteria driving *Scrape...*; Refresh reaching *Scrape This
  Game*; Game Names in the list and the editor, the player's name winning, the offer back when off; the cleanup asked,
  declined, then moving five files and leaving the kept game's, another console's and the ROM folder alone; a missing
  ROM folder cleaning nothing.
- Changed: the three options menus' expected entries gained *Find by Name...*; `MediaStoreFoldersTests`' schema-1 file
  drops the new column too, and asserts the current schema rather than 2.
- `ScrapeExtrasPictureTool` writes 38.12's pictures with `EMUSEN_BIGPICTURE_PNG=1`.

### 38.11 Mutants

The runner is `~/.cache/emusen/probe/pass8/mutate_pass8.py`, its verdicts `mutants-pass8.txt`, its log `run-pass8.log`.
Each mutant replaces one exact piece of one file, checked to occur once; it was built with `-m:2` and tested alone
under `nice -n 10` against its rule's tests (`ScrapeExtrasTests`, `ScrapeExtrasWindowTests`, and for the names also
`GameMetadataTests` and `ThemedMetadataScrapeTests`). Before changing a file the runner writes `mutant-state.json` (the
file and a copy), restores it in a `finally` and touches it, restores any file a cut-short round left mutated when it
starts, and rebuilds the tree at the end. Each mutant took 8–34 s.

**48 mutants, 48 caught on their first run, none failed to build.**

| Rule | Mutants |
|---|---|
| The kinds | K1 back covers never fetched; K2 a manual written `.png`; K3 the full `video`, not `video-normalized`; K4 fan art on by default; K5 `kinds_known` not recorded; K6 a kind an old answer knew nothing of read as not offered; K7 `kinds_known` ignored |
| What a file must be | Y1 any file kept as a manual; Y2 a clip judged by its content type alone; Y3 every kind judged as a picture |
| Refresh | R1 ignored for a found game; R2 the answer's checksum not compared; R3 the kept file's checksum not sent; R4 `SHA1OK` not understood; R5 a changed file never replaced; R6 a later 404 making a found game Unknown; R7 *Scrape This Game* ignoring Refresh; R8 a run's refresh not handed to the worker |
| Find by Name | N1 a search sent on opening; N2 a pick looked up by hash anyway; N3 a pick recorded as matched by the file; N4 a picked game never asked by its id; N5 a search not paced; N6 a search finding nothing not counted as unrecognised; N7 a search sent with ScreenScraper switched off; N8 offered during a run; N9 missing from the context menu; N10 the search text keeping its tags; N11 an empty entry read as a game; N12 the search without its system |
| Criteria and prices | C1 favourites ignored; C2 no metadata read as no cover; C3 no game image ignoring screenshots; C4 no game video always true; C5 the criteria applied to one game; C6 the tab's criteria not used; C7 all games by default; P1 a kind priced as always offered; P2 a refresh priced as nothing asked again; T1 manuals and clips not waiting their turn |
| Game Names | Q1 the list ignoring it; Q2 the editor ignoring it; Q3 ScreenScraper's name always the baseline |
| Orphaned media | O1 deleted, not moved; O2 an emptied folder kept; O3 the moved file's row kept; O4 no confirm; O5 a kept game's foldered media called orphans |

**What the clean round does and does not show.** The list was written with the tests in view, by the same hand, and
reading it against the first tests showed six mutants that would have survived: C2 (the only described game also had a
cover), C6 and R7 (no test pressed *Scrape...* on the tab, or scraped one game with Refresh on), N5 and N7 (no test
paced two searches, or searched with the switch off) and O2 (no folder was emptied). The tests were strengthened before
the round ran (commit `36d6ada4`), and the missing-ROM-folder test gained the rescan that follows an unplugged drive. The ROM-folder guard itself was not made a mutant:
a scan of a missing folder has no games, and a console with no games is never looked at, so the guard is implied by that
rule and a mutant of it would be equivalent; it is kept as defence in depth. A redundant `names.Count == 0` test in
`MediaStore.Orphans`, equivalent for the same reason, was removed rather than kept untested. As §27.5 said of its own
round, a mutant written after its test shows the test is not empty, not that the list of rules is complete.

### 38.12 Pictures

`ScrapeExtrasPictureTool` writes to `~/.cache/emusen/bigpicture/png/pass8/`, at 1280×800 and 1920×1200 each; every one
was looked at:
- `desktop-fetch-more-and-game-names`, `desktop-scrape-criteria`, `desktop-orphaned-media`: Preferences as a window over
  the desktop library, scrolled to the six *Fetch More* switches (all off) and *Game Names*, to *Scrape* with its two
  dropdowns (*Every console*, *Games with no cover*), *Refresh what is kept* and the four buttons, and to *Orphaned Media*
  with *Clean Up...*.
- `desktop-find-by-name`, `desktop-find-by-name-results`: the chooser over the library with "SMW Hack" in its box and
  Search focused; then three results, the first focused, each with its year and publisher.
- `sheet-fetch-more`, `sheet-game-names`, `sheet-scrape-criteria`, `sheet-orphaned-media`, `sheet-find-by-name-results`:
  the same rows on the Preferences sheet of a big-screen session, with the pad's help line, and the chooser as a sheet.
- **Seen and left:** the criteria dropdown has no label of its own, as the console dropdown above it has none; the row's
  hint names both. The OpenEmu switch's long label is cut at the right edge at 1280×800, as it was before this pass.

### 38.13 Predictions

Written during the build, by the hand that wrote the code, as §27.1 and §28.1 were, and weaker evidence for it.

| # | Prediction | Found, or retired when | Verdict |
|---|---|---|---|
| P111 | Refreshing an unchanged found game costs one request per kind and under 1 KB per kind | every media entry of §17.9's answers states its SHA-1, so an unchanged file is known from the lookup: **one request a game** in all, on the fake server, and `SHA1OK` per kind only where an answer states no checksum | retired by design; the live cost of a `SHA1OK` is not measured |
| P112 | The four picture kinds raise a found game from 4.7 to 7.5–8.5 requests, add 0.9–1.5 MB, 20–28 s at 128 KB/s | 7.87 requests, 1.30 MB, 23.7 s by 38.6's model on §17.9's answers | **held by arithmetic** on the answers; no run fetched them |
| P120 | `video-normalized` adds 0.7–0.9 requests and 0.9–1.3 MB a found game | 0.82 and 1.11 MB by the same arithmetic | held by arithmetic; Pass 12's first live run retires it |
| P240 | In a live refresh of about ten found games with nothing changed, every media entry states a `sha1` equal to the kept file's, so the run costs one request a game and downloads nothing | the first live refresh | open |
| P241 | `jeuRecherche` answers a name with no match either with `jeux` holding one empty object or with a 404, never a game of id 0 | a live search for a name that does not exist | open |
| P242 | A search that finds nothing raises `requestskotoday` by one, and one that finds games does not | the same live search, reading the counts before and after | open |
| P243 | `mediaJeu.php` sends a manual starting `%PDF-` and a clip with an MP4 `ftyp` box, whatever content type it declares | the first live run with manuals and videos on | open |
| P244 | Of the round's mutants, at least nine in ten are caught on their first run | 48 of 48 (38.11), after six tests were strengthened before the round | held, and weak evidence for the reason 38.11 gives |

### 38.14 Not done

- **No live run.** §21.3 planned one run of about ten games, started by hand, to measure what the fake cannot: the cost
  of a refresh (P111, P240), a search that finds nothing (P241, P242) and what the manual and clip downloads look like
  (P243). No request reached ScreenScraper or any other server while this pass was built or tested; the run is for the
  player to start.
- **Viewing** a manual (Pass 9) or playing a clip (Pass 12). They are stored and found; nothing shows them.
- **Interactive runs and auto-accept** (Q141), **thumbnails in the chooser** (Q140), **Ratings and Other metadata
  switches** (Q142), **Folders only** and scraping folders (Q143).
- **TheGamesDB** (Pass 13).
- **OpenEmu's failover** still fills covers only; it knows none of the new kinds.
- **The cleanup** covers Mistress's media store only: not `games.db`, not OpenEmu's folder, not an ES-DE folder, and
  it never empties `CLEANUP`.
- **Nothing ran on the handheld**, and no real pad or window manager drove the chooser.
- **The chooser at scale:** thirty results is ScreenScraper's cap and the tests' largest list.

### 38.15 Open questions

- **Q140, thumbnails in the chooser.** Each result's picture is a request. Options: none (built); the highlighted
  result's cover after it has stayed highlighted for a second; every result's. **Recommendation: none**, with the year,
  publisher and platform as the tie-breakers, since a search of 30 results would cost 31 requests with thumbnails.
- **Q141, interactive runs.** ES-DE's multi-scraper can stop on each game its hashes miss and show the chooser. Options:
  not built; built, off by default. **Recommendation: not built.** ScreenScraper found 39 of 40 by hash (§17.10), so a
  run would stop about once in forty games, and *Find by Name…* already reaches each of those.
- **Q142, Ratings and Other metadata switches.** **Recommendation: not built**, since the text costs no request and the
  metadata editor overrides any field.
- **Q143, Folders only.** **Recommendation: wait**, with directories as files (Q24), for a library whose folders are
  games.
- **Q144, Refresh's default.** ES-DE's `ScraperOverwriteData` is on. **Recommendation: keep it off** (38.3).
- **Q145, the cleanup folder.** ES-DE leaves `CLEANUP` for the player to delete. **Recommendation: the same**; its size
  is in the message that moved the files.


### 38.16 The broad run

After merging WiseMan at `c1ef5c6d` (§36, Pass 14's first half), one run of the Mistress filter, without
`ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any GPU or Vulkan test, under `nice -n 10`:
**1,400 tests, 1,360 passed, 40 skipped (the picture, survey and live tools), 0 failed, in 4 min 12 s.** Before the
merge the blast radius (the scraping tests, the metadata, options, collections, cover-choice, OpenEmu, pad-settings and
themed-library tests; 376) had passed but for one: `GridSceneTests.Item_sizes_and_corner_radii_are_in_ES_DE_s_units`,
run in that filter's order, found no render interface, the headless platform's initialisation failure §35.12
recorded as order-dependent. It draws no media of this pass's and passed in the broad run.

## 39. Pass 14, second half: two defects, the carousel's margins, `gamelistinfo`, `animation`, and what ES-DE draws unasked (2026-09-27)

*Opened on the review of §36, 2026-09-27 (§10.1): items 1 and 2 accepted, and every recommendation of Q120–Q122 taken.*
This section records, in the order they were decided: two defects seen in §36's side-by-side pictures; Q120, the
carousel's `selectedItemMargins` and `lineSpacing`; `gamelistinfo`; `animation`; Q122, the help bar and status
indicators ES-DE draws for a theme that defines none; and Q121, Mistress's word where ES-DE writes "unknown". One more
defect, an undefined theme variable, was found on the way (§39.4). The player's account is §4.77 of the settings
reference. §37 and §4.75 are Pass 10's, and §38 and §4.76 Pass 8's.

**Sources.** ES-DE 3.4.1's behaviour, measured by running it, and `THEMES.md`; never its source. The setup is §36.1's:
the scratch home `esde/home-pass14/`, the shared lock, the synthetic library, no scraping, no updater. The probe themes
gained a third, `p14-defaults-es-de` (help and status, the words, `gamelistinfo`, GIFs made by the probe's own script),
and the wheel probe gained nine variants. Two more listed themes were downloaded into `pass14-themes/`: Cathode (it uses
both `gamelistinfo` and `animation`) and Adroit (whose animated colour scheme names a GIF its repository does not hold,
so ES-DE logs "Couldn't open GIF animation file" and draws nothing there; it was not used further). 58 more runs; the
logs are in `probe/pass14/logs/`.

**Numbering.** Predictions here start at **P250** and questions at **Q150**; P230–P232 and Q130–Q132 are Pass 10's, and
P240–P244 and Q140–Q145 Pass 8's.

### 39.1 Two defects from §36's pictures

- **"1 GAMES".** Aura writes `gamecountGames` upper-cased; Mistress said "1 games". Measured in ES-DE on systems of one,
  three and twelve games and none, one and six favourites: `gamecountGames` is "1 game" and "12 games", and
  `gamecountFavorites` "1 favorite" and "0 favorites". Mistress's counts are now singular for one (its own wording for
  `gamecount`, "12 games available, 6 favorites", stays, §3.6). Flags read "yes" and "no" in lower case, as ES-DE wrote
  them; Mistress had "Yes" and "No".
- **Bluetooth as a "B".** LunaP's `DeviceStatusBar` drew Bluetooth and cellular as letters of the typeface. ES-DE was run
  with `SystemStatusDisplayAll` (`THEMES.md`: force every indicator on), and each icon's ink box read at both sizes. All
  four built-in icons are now LunaP geometry in a square box (LunaP §191): the Bluetooth rune, a Wi-Fi fan with its dot,
  four cellular bars, an upright battery with a bar per quarter; and the percentage follows the battery with no spacing.
  Every icon's ink is within 2 px of ES-DE's in width and height and the gaps between icons within 1 px; alone, as on
  the desktop, the Bluetooth icon's ink box lies within 1 px of ES-DE's at 1280×800 and 2 px at 1920×1200, and the rune's coverage IoU is 0.79
  at 1280×800 and 0.74 at 1920×1200, thin strokes as §13.8's icons were. ES-DE's percentage is set in its own narrower
  typeface, so Mistress's row stands about 20 px further left.

### 39.2 Q120: `selectedItemMargins` and `lineSpacing`

Six probe variants, straight carousels of both orientations, halves of the screen, negative margins and a scaled
selection:

- **The margins are fractions of the screen along the row** (its width for a horizontal carousel, its height for a
  vertical one), not of the carousel: a half-width carousel with `0.1 0.1` moved its neighbours 128 px at 1280 wide.
- **Every item on a side moves**, not only the neighbour: the second item before the selection moved as far as the first.
- **`itemScale` plays no part**, and a negative margin brings the items closer.
- **While the row moves**, an item between the selection and its neighbour's slot moves by its fraction of the way: a
  30-frame-a-second recording of one step, read frame by frame, followed that linear rule within a pixel at every frame.
- **`lineSpacing`** is the pitch of a text item's lines as a multiple of `fontSize`: 40, 60 and 80 px at a 40 px font for
  1, the default 1.5, and 2.

LunaP's `ImageCarousel` gained `SelectedItemMargins` and `LineSpacing` (its §192). Mistress's logos land within 0.8 px of
ES-DE's in all six variants, with 0.00–0.29% of pixels differing by more than 8 levels. Aura's game carousel, §36.9's
worst, went from 20.3% to 3.9% at 1280×800 and from 19.1% to 1.5% at 1920×1200, once §39.4's variable was also read.

**Where Mistress differs.** With `lineSpacing` 2, ES-DE set "(Super Nintendo)" on one line wider than its item box,
where Mistress wraps it; the rule behind that was not isolated. The text items are otherwise in Mistress's own font, as
everywhere (§13.8).

### 39.3 `gamelistinfo`

Seven `gamelistinfo` elements of different sizes and alignments, on systems of one to twelve games, inside a folder, and
with a favourites filter set through ES-DE's own menus by the probe pad:

| State | ES-DE 3.4.1 drew |
|---|---|
| a list | a gamepad and the game count, a star and the favourite count: "🎮 12 ★ 6" |
| inside a folder | the same counts, which are the system's over every folder (3 for the `ngp` probe, whose folder holds two of its three games), and an open folder after them |
| a right-aligned line in a folder | the folder first, at the left |
| a favourites filter | a funnel and "kept / all": "6 / 12"; no favourites count |
| no `color` | black |
| `size` `0 0` | a box as wide as the line and 1.5 `fontSize` tall; `w 0` the same height; `w h` the line centred down the box by default |

Mistress draws it with LunaP's new `InfoLine` (its §193): the four pictures are LunaP's own drawings, in proportions of
the em read from the captures, since ES-DE draws them from an icon font it ships. The counts come from a
`GamelistCounts` the library fills for the shown system: every game over every folder, without folders or games
excluded from the counter, the ones search and filters keep, and whether a folder is open. On the probe the line's
pictures fall within 2 px of ES-DE's; the numbers are in Mistress's font. Cathode's `gamelistinfo`, right-aligned in its
list header, lands where ES-DE puts it.

### 39.4 A theme variable nothing defines

Aura's game carousel lacked the glass panel behind the name. Its path is written with `${glass-size}`, which Aura defines
only for its largest font size. The loader refused the property as an error, which dropped the element (§31.7 counted it
as Aura's one remaining error). ES-DE was run on a probe: `A${nosuch}B` drew "AB" with no log line, and `${nosuch}`
alone refused the system with "Property "text" for element "text" has no value defined". An undefined variable in a
property is now read as empty, with a warning (`UndefinedVariable`, no longer an error), and a property left with no
value by it is an error, as there. Aura's panel is drawn. Undefined variables in an include path are skipped as before;
ES-DE was not run on that.

### 39.5 `animation`

Ten animation elements on GIFs made by the probe's own script (four flat frames of different colours with delays of 100,
200, 300 and 400 ms; others of 300/100/100/100 and 100/100/100/700; a GIF with a loop count of 2; a 4×4 checker), recorded
for six seconds at 30 frames a second with a move of the system carousel half way:

- **Every frame is shown for the first frame's delay**: the three GIFs played at 100, 300 and 100 ms a frame.
- **The GIF's own loop count is ignored**; `iterationCount` counts passes, and for `alternate` a pass is a round trip:
  `alternate` with 2 played 0 1 2 3 2 1 0 1 2 3 2 1 0 and held the first frame; `normal` with 1 played 0 1 2 3 and held.
- **`reverse`** went 0 3 2 1 0; **`alternateReverse`** 0 3 2 1 0 1 2 3 2.
- **A move of the system view reset every animation**, and after the reset `normal` and `alternate` held the first frame
  for two frame times, the others for one: the first frame, then the direction's sequence from its start.
- **`speed`** 2 gave 50 ms frames and 0.5 gave 200 ms; **`color`** `FFFFFF80` halved the opacity; **`interpolation`**
  defaults to nearest.

Mistress draws GIFs with LunaP's new `FrameSequenceImage` and a GIF decoder of its own (LunaP §194), since Avalonia reads
a GIF's first frame only. The view's clock drives it: the time restarts when the view opens and at each move of the
system view. Whether a step of the gamelist resets an animation was not measured, and it does not here (P252).
**Lottie is not drawn**: it would need Skottie, a SkiaSharp library, which LunaP may not take (it references Avalonia
alone); by `PLAN-icons.md` §1.1's rule it would go to a sibling package or to Mistress. None of the four listed themes
that use `animation` was found to need it.

Cathode's 600×338 GIF of 338 frames at 30 ms, its camcorder overlay, is drawn over the screenshot as in ES-DE. It showed
why the first decoder, which composited every frame when the file was read, would not do: 274 MB of pixels. LunaP now
composites on demand from every sixteenth frame's canvas.

**Where Mistress differs.** With `interpolation` `linear`, ES-DE's scaled 4×4 checker shows a blur whose edges wrap
round the image; Mistress's high-quality filter does not wrap. A first frame delay of 0 is taken as 100 ms (not
measured, P251).

### 39.6 Q122: the help bar and status ES-DE draws unasked

On `p14-defaults-es-de`, whose views define neither, ES-DE drew a help bar at the bottom left and the status indicators
at the top right in both views. The help bar's ink started at x 16 and y 762 at 1280×800, where `THEMES.md`'s default
`pos` of `0.012 0.9515` and `fontSize` 0.035 put it, and at x 25 and y 1143 at 1920×1200; the Bluetooth icon sat where `systemstatus`'s default `pos`
`0.982 0.016`, `origin` `1 0` and `height` 0.035 put it. A help bar defined only in the gamelist left the system view
with the default; a help bar or status element at `opacity` 0 left no default at all.

So a view with no `helpsystem`, and one with no `systemstatus`, now gets one at the documented defaults (named
`(default)` in the resolved view). Mistress's help bar keeps its own words and pictures (§3.6): its ink starts at the
same place (x 16, y 764; x 24, y 1147 at 1920×1200) and is 6 px taller, set in Mistress's font. `visible` is not a property of either element in
ES-DE: a probe writing it refused the system, as `THEMES.md` lists no such property and the loader agrees.

### 39.7 Q121: Mistress's word for a missing field

Measured on a game with no metadata, in the system view through a gameselector and in the gamelist: `developer`,
`publisher`, `genre`, `players` and `playtime` read "unknown"; `releasedate` "unknown" and `lastplayed` "never";
`description` and `altemulator` nothing; `rating` "0"; the flags "no". A theme's `defaultValue` replaces the word, as
`THEMES.md` says.

Mistress now shows "unknown" and "never" in those places. **Pass 5's text lookup does not exist yet**, so the words are
constants in one place, `SceneWords` in `BigPicture/Scene/`, which that pass will route through its lookup; the plan's
Pass 5 finds them there. The words are ordinary English, not taken from ES-DE's resources.

### 39.8 Tests

| Class | Tests | What it holds |
|---|---|---|
| `EsdeDefaultsTests` | 6 | a default help bar and status in each view at the documented `pos`; per view, and not beside an invisible one; the words; `defaultValue` first; "yes"/"no"; singular counts |
| `InfoAndAnimationTests` | 4 | the counts and the default colour; filtered and in a folder; an animation's frames against §39.5's measured sequences, reset by a system move; no Lottie |
| `WheelCarouselTests` | 6 more cases | the six margin variants, logo centres within 1.5 px of ES-DE's |
| `ThemeLoaderTests` | 1 changed | an undefined variable read as empty, and a property left empty unthemes |
| `SceneMappingTests` | 2 (367 cases) | `selectedItemMargins`, `lineSpacing`, `gamelistinfo`'s 6 and 8 common pairs, `animation`'s 11 and its common pairs |
| LunaP | 12 | `CarouselWheelTests` (margins), `IndicatorControlTests` (the rune), `InfoLineTests` (2), `FrameSequenceTests` (8 cases: its own GIFs and two of forty frames out of order); LunaP §191–§194 |

`ThemedFoldersTests` expected "1 favorites" and `ThemeErrorTests` an empty system view where a default status now
stands; both were corrected with the rules.

### 39.9 Mutants

The runner is `~/.cache/emusen/probe/pass14/mutate_pass14b.py`, its log `run-pass14b.log` and its verdicts
`mutants-pass14b.txt`, with §36.7's protocol (a state file before each mutant, restored on start and in a `finally`, the
file touched after restoring, `-m:2`, `nice -n 10`, a clean rebuild at the end). Mistress's mutants ran against
`GameSelectorTests`, `WheelCarouselTests`, `SceneMappingTests`, `EsdeDefaultsTests`, `InfoAndAnimationTests` and
`ThemeLoaderTests`; LunaP's against its carousel, indicator, `InfoLine` and `FrameSequence` tests and then the same.

**29 mutants: 22 caught at once, 7 survived the first run, all 29 caught after the tests were strengthened.**

| Area | Mutants (caught unless marked) |
|---|---|
| defaults (Q122) | D1 no default help bar; D2 a default beside an invisible element; D3 defaults in the system view only |
| words (Q121, §39.1) | U1 no "unknown" for a missing developer; U2 a missing last played reads "unknown"; U3 the word before `defaultValue`; U4 plural for one |
| variables (§39.4) | V1 an undefined variable still unthemes; V2 a property left empty kept |
| margins (Q120) | M1 margins by half the screen; M2 before and after swapped; M3 `lineSpacing` ignored |
| `gamelistinfo` | I1 a filtered list shows the plain counts; I2 no folder picture; **I3 games excluded from the counter counted**; I4 white by default |
| `animation` | A1 not reset by the system view; **A2 a Lottie file read as a GIF** |
| LunaP | L14–L24 (its §194.1), **L17, L18, L19, L20 and L24 survived first** |

- **I3** survived because no test's list held an excluded game; `The_counts_are_the_games_and_favourites` now excludes
  three.
- **A2** survived because the Lottie test's `.json` held no GIF, so the decoder refused it either way; the test's `.json`
  is now a real GIF under that name.
- **L17–L20 and L24**: LunaP §194.1.

### 39.10 Pictures

In `~/.cache/emusen/bigpicture/png/pass14/`, ES-DE on the left and Mistress on the right: `side-d-*` (defaults, words,
the forced status row `d-all-sys`), `side-i-*` (`gamelistinfo`: `i-snes` at both sizes, `i-ms`, `i-gg`, `i-ngp-top`,
`i-ngp-inside`, `i-filter`), `side-a-still` and `side-a-info-default`, `side-w-m*` and `side-w-ls*` (margins and line
spacing), and the real themes again: `side-r-aura-gl`, `side-r-aura-sys`, `side-r-codywheel`, `side-r-mania` and
`side-r-cathode`, each at 1280×800 and 1920×1200.

| Downloaded theme and state | Pixels > 8 levels, 1280 / 1920, §36.9 → now |
|---|---|
| Aura, game carousel | 20.3% / 19.1% → 3.9% / 1.5% |
| Aura, system view | 2.5% / 0.4% → 2.3% / 0.4% |
| Cathode, list | new: 24.0% / 23.4% |

Cathode's remaining difference is not this pass's: its list stands a row lower in Mistress, and its description text
shows the system's description where ES-DE shows the game's (Q151).

**The survey again** (`EMUSEN_THEME_SURVEY=analyse` on §25.8's XML, the earlier file kept as
`survey-before-pass14b.json`): all 66 listed themes load for all five systems with **no loader error** (63 at §31.7, 65
at §35; Aura's undefined variable was the last), and **no element any of them uses is left undrawn**. The widest
unmapped properties are now the video's (Pass 12), the help system's dimmed set, and `image.stationary`.

### 39.11 Predictions

| # | Prediction | Retired when |
|---|---|---|
| P250 | On the handheld, whose Wi-Fi and battery are real, Mistress's status row matches ES-DE's icon for icon within 2 px | both are run there |
| P251 | ES-DE plays a GIF whose first delay is 0 at 100 ms a frame | ES-DE is run on one |
| P252 | A step of the gamelist does not reset an animation in ES-DE | a recording of the gamelist |
| P253 | Cathode's list stands where ES-DE puts it once Q151's causes are found; no pass 14 element differs there | Q151 |

### 39.12 Not done

- **Lottie**, by the rule above; its files draw nothing.
- **`stationary`** of `gamelistinfo` and `animation`, and `animation`'s `brightness`: not mapped, as `image`'s are not.
- **`gamelistinfo` does not wrap or cut its line** for a narrow box; ES-DE was not seen to.
- **The other metadata fields'** words where ES-DE writes something other than nothing were matched only for those
  §39.7 lists; `rating` as text is still Mistress's "x/5".
- **Nothing ran on the handheld.**

### 39.13 Open questions

- **Q150, Adroit's missing GIF.** Adroit's "Animated" colour scheme names a GIF its repository does not hold; ES-DE logs
  an error and draws the background without it, and so does Mistress (the file is missing). Options: nothing; tell the
  theme's author. **Recommendation:** nothing here; it is the theme's.
- **Q151, Cathode's list and description.** Mistress places Cathode's list about a row lower and shows the system's
  description in the gamelist. **Recommendation:** a short measured pass on Cathode's `list.xml`, since it is the first
  listed theme to show either.
- **Q152, `rating` as text.** ES-DE writes a missing rating as "0" and, by §39.7's probe, a present one as a number;
  Mistress writes "x/5". **Recommendation:** measure the present case and match it, with Pass 5's words.

### 39.14 The broad run

WiseMan (b272675c, with Pass 8's §38) and LunaP's `openemu-library` (5a1e1a2) were merged into the branches before the
run. One run of the Mistress filter, without `ShaderSettingsWindowTests`, `ShaderBrowseBench`, `SceneGpuBench` and any
GPU or Vulkan test, under `nice -n 10`: **1,416 tests, 1,376 passed, 40 skipped (the picture, survey and live tools),
none failed, in 4 min 20 s.** LunaP's whole suite: 1,425 tests, all passed.

WiseMan then moved to b57f224e (menus stage 2, §34) and `openemu-library` to 691a54f (LunaP §182.8–§182.10); both were
merged, the only conflict LunaP's README count, and both runs were repeated: **1,434 tests, 1,392 passed, 42 skipped,
none failed, in 4 min 19 s**; LunaP 1,428, all passed.

## 40. The menus' follow-ups: lettered help glyphs, Q105 to Q107, and the device's own keyboard (Q109) (2026-09-27)

*Built on branch `bigpicture-menus-followups`, from WiseMan at `758e81bb`; LunaP on `menus-followups`, from
`openemu-library` at `18413c7`.* §10.1 records the decisions of 2026-09-27 on §34.22's questions: letter-labelled help
glyphs as ES-DE draws them, following the controller type in use; the editor's one-line subtitle (Q105); per-row help
(Q106); the scroll indicator (Q107); leaving the editor for the gamelist, unchanged (Q108); and the device's own
on-screen keyboard, Mistress's staying as a fallback (Q109). The player's account is §4.79 of the settings reference;
LunaP's record of its pieces is its §195.

**Direction, clarified 2026-09-27.** The menus are not converted one-for-one with ES-DE. What is kept is ES-DE's look
(its colours, Barlow Condensed, the rounded panels) and controller-friendliness. Where one of these items would force
ES-DE's structure on a window that needs its own layout, the window keeps its layout and takes the look only; the
complex windows (scraping, cheats, controller bindings) are restyled separately. None of this section's items needed a
window's layout changed.

**Numbering.** Predictions from P260, questions from Q160; both ranges were free in the tree when this began.

**Sources.** ES-DE 3.4.1 was run on 2026-09-27 in a scratch home of its own (`~/.cache/emusen/bigpicture/esde/home-followups`)
on empty synthetic ROM files, driven by §22.2's uinput pad from a timeline, every run under the shared lock
`~/.cache/emusen/bigpicture/esde/esde.lock` and its whole process tree killed after. The runner and its timelines are
`~/.cache/emusen/probe/menus-followups/esde/`, the captures under its `captures/`: `editor-rows` (every row of the
editor and every button, 29 pictures), `family-<type>` for each of ES-DE's seven controller types (`xbox`, `xbox360`,
`ps123`, `ps4`, `ps5`, `switchpro`, `snes`) and two with its swap on, and `rating-wrap`. §34.12's captures were
measured again for the scroll indicator. The measurements are pixel scans of those pictures. No image, font or colour
table of ES-DE's was copied or traced into the tree.

### 40.1 The help glyphs, measured

| Measured at 1280×800 | ES-DE |
|---|---|
| a face button | a solid disc 22 px across with its letter or shape cut out, centred on the label's capitals |
| a label's capitals | 18 px |
| glyph to its label | 7 px |
| a label to the next glyph | 12 px |
| the d-pad | a solid plus, small arrows at the tips of the arms the entry moves along |
| shoulders and triggers | solid pills lettered LB, RB, LT, RT (L1, R1, L2, R2 on the PlayStation types) |
| Start and Select | per type: Xbox a disc with three lines and one with two squares; Xbox 360 discs with arrows; PlayStation 1/2/3 and SNES the words SELECT and START over small shapes; PlayStation 4 SHARE and OPTIONS; PlayStation 5 a burst and three lines; Switch Pro a minus and a plus |
| a thumbstick click (Random) | a ring with a dot and four small arrows, not filled |

Two findings the design turned on:

- **The letters follow the type, and one type letters by function.** SNES letters by position: its Select is B, its
  Back A. **Switch Pro letters as Xbox does**: its Select is A, although a Switch Pro's A is its right-hand button.
  Mistress's Nintendo family letters by position, as SDL 3 reports the buttons and as Mistress's pad routing reads
  them, which is the SNES type's rule (Q161).
- **ES-DE's swap trades both pairs.** With `InputSwapButtons` on, Xbox's Select shows B, Back A, View Media Y and
  Favorites X. Mistress's swap (§4.61) trades A and B only, and its help follows what it swaps (Q160).

**What was built.** LunaP's `PadGlyphStyle.Filled` (its §195.1): each button a solid disc, pill, plus or shoulder with
its letter, symbol or mark cut out, the disc 0.425 of the glyph's square, so 22 px beside a menu's 26-pixel help text.
The families stay Mistress's four (Xbox, PlayStation, Nintendo, and Generic for an unknown pad), with their own
geometry for Start and Select; ES-DE's seven types were measured to decide what tells the families apart, not to copy
their marks. **An unknown pad is lettered as an Xbox pad**, ES-DE's default type: the four-dot diagram was the one set
ES-DE never shows. The filled set is drawn in every big-screen help bar: `MenuPanel`'s by default (the Start menu,
Gamelist Options, the editor, every settings screen, the list screen, a message box), and the themed view's help
element (`IndicatorElements.Help`), which is also the help bar under the launch screen and the screensaver. The desktop
is unchanged. `HelpFamily` and `SwapPadButtons` still choose the family and which button is named.

### 40.2 Q105: the subtitle

ES-DE's editor has one line under its title, the file's name and its system in brackets, in its own case
(`editor-rows/e00.png`: *Aurora Drift (Synthetic).sfc [SNES]*). `MetadataEditorWindow.SubtitleOf` gives the same from the
shelf's ES-DE system name, upper-cased; a file of no known shelf shows its name alone. The subtitle is no longer
upper-cased. With one line the title band is 139 design pixels, §34.12's measure.

### 40.3 Q106: the help bar per row, in ES-DE's words

Read off `editor-rows`, one picture per focused row and per button:

| Focused | ES-DE's A | Mistress's A | Also shown |
|---|---|---|---|
| a text row (Name, Sortname, Description, Developer, Publisher, Genre, Players, Times played, Play time) | Select | Select | |
| Rating | Add Half Star | Add Half Star | Change (Left and Right step it) |
| Release date | Edit Date | Edit Date | Change |
| a switch (Favorite to Hide metadata fields) | Toggle | Toggle | |
| Controller, Alternative emulator | Select | Select | Change |
| Scrape | Scrape | Scrape | |
| Save | Save Metadata | Save Metadata | |
| Cancel | Cancel Changes | Cancel Changes | |
| Clear | Clear File | **Clear Metadata** | |
| Delete / Hide from Library… | Delete Game | **Hide Game** | |

B is *Back* and Y *Scrape* throughout, in that order after A; the d-pad's *Choose* is up and down on the rows and left
and right on the buttons. Mistress adds *Reset* on West after *Scrape* while the focused field holds an edit (Q101), and
*Change* on the rows Left and Right step, since those rows answer them; ES-DE shows neither.

Two words differ on purpose. *Clear File* would say that a file is touched, and Mistress never touches a game's file:
its Clear removes edits and scraped text, and its own question is titled *Clear Metadata*. *Delete Game* names what
Mistress does not do; its button is *Hide from Library…*, so its A is *Hide Game* (Q162). The request's example for a
text row was *Edit*; ES-DE was measured to say *Select*, and Mistress follows the measurement (Q163).

**A on the stars.** *Add Half Star* named an action Mistress's A did not have: the stars stepped with Left and Right
only. ES-DE was measured pressing A eleven times on an empty rating (`rating-wrap/r00`–`r11`): half a star each press,
five stars at the tenth, none at the eleventh. Mistress's A now does the same, through the router's own key, so the
window's keyboard handler does not take it for the player's Right (§34.11's defect).

### 40.4 Q107: the scroll indicator

Measured on `editor-1280/g02`, `g06` and `g08` and `menus-1280/m01` (§34.12's captures):

- two chevrons in a square 25 px on a side, a round-capped stroke about 3.4 px, the arms about 42° from level;
- the lower square when rows are below, the upper when rows are above, both when both;
- the squares' right edge 11 px inside the panel's (x 1048 against 1059), the two 7 px apart;
- the pair centred on the title's capitals: 126.5 against 128 in the editor, 154.5 against 154.5 in UI Settings;
- the grey of the menu's secondary text (112 of 255 on ES-DE's dark panel).

LunaP's `MenuPanel` draws it (its §195.2) from those numbers scaled from 800 lines, in its own grey (#747478). It
watches a `ScrollViewer` child, or the scroller a list's template holds, so every big-screen menu gets it with no change
to the menu: the editor, Gamelist Options, every settings screen, the list screen, and the Start menu, whose entries
run past the panel at 800 lines.

### 40.5 Q109: the device's own keyboard

**What was found about `steam://open/keyboard`, 2026-09-27.** There is no public documentation of the address. What
was established:

- Valve's issue tracker uses it as a plain command, `steam steam://open/keyboard`, with no parameters
  (ValveSoftware/steam-for-linux issue 11404), and reports that repeated requests while Steam's own menus are open take
  the pad's focus from them.
- The Steam client installed on the desktop (its `steamui` script, read on 2026-09-27) shows how a keyboard request is
  handled: a request carries an app id, a flag for Enter closing the keyboard, and a text field's rectangle. A request
  with app id 0 passes the check that it belongs to the running game. Unless it came from the Steam+X chord, a window
  shows the keyboard only while its route is the running game's, so in Game Mode it should open only while Mistress is
  the game in front. Under gamescope the keyboard is Game Mode's own screen; on a desktop it is a floating window.
- No parameter for position or size was found: the client's strings name none for the address, and the rectangle is
  filled from Steamworks' own call. **The address is used with no parameters.**
- It types into whichever window has the focus. Mistress therefore focuses the field first, then asks.
- Steamworks' `ShowFloatingGamepadTextInput` was not used: it needs a real Steam app ID, which a non-Steam shortcut
  does not have, and no borrowed ID is used.

**What was built.**

- A big-screen text row chosen (A on a pad, Enter on a keyboard) opens **`MenuTextPopup`** (LunaP §195.3): the text
  popup of §34.16 whose bar is a real, focused `TextBox`, titled *Enter <row>*. Enter keeps the text and Escape drops it,
  as before; on a pad Start keeps it, B drops it, and Y asks Steam again for a keyboard put away. A is not taken, since
  with Steam's keyboard on screen an A may be the keyboard's own press.
- **Which keyboard** is `DeviceKeyboard.Choose`: the setting, whether Steam is there, and whether a pad chose the row.
  Automatic gives Steam's keyboard under Steam; else the field alone when the row was chosen from a keyboard; else
  Mistress's keyboard. *Steam* always asks Steam; *EmuSen's* is always Mistress's keyboard.
- **Under Steam** is `DeviceKeyboard.UnderSteam`: Game Mode's session (`InGameModeSession`, reused), a process Steam
  launched (`SteamGameId`, `SteamAppId`, `STEAM_COMPAT_APP_ID`, `STEAM_COMPAT_DATA_PATH`,
  `STEAM_COMPAT_CLIENT_INSTALL_PATH`), `SteamDeck=1`, or a Steam client running, checked with `pgrep -x steam`.
- **Asking** is `PlatformUrlLauncher`: `xdg-open steam://open/keyboard`, and `steam steam://open/keyboard` if `xdg-open`
  is missing or fails quickly; the shell elsewhere. It runs off the interface's thread.
- **Which device chose the row**: a row chosen while the keyboard's Enter is held was chosen from the keyboard; any
  other from a pad.
- **The keys**: while the popup is open the window's keyboard handler leaves typed keys to the field (the rule of
  §4.17), and a steering key let go is released whatever has the focus, since the popup takes the focus on Enter's press.
- **Mistress's keyboard is unchanged** where it is used: every text box that is not a big-screen row (the themed search,
  the cheats window, the desktop), and every row with *EmuSen's*, or with Automatic and a pad off Steam (Q164).

**The harness never launches anything.** A module initialiser in WiseMan (`NoSteam`, beside §17's `NoNetwork`) gives
every window an empty environment, no running Steam, and a launcher that records the address and opens nothing. The
desktop these tests run on has Steam running, so without it Automatic would have chosen Steam in every test.

### 40.6 Predictions

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P260 | *(written before the mutants)* Of 40 mutants, at least nine in ten are caught on their first valid run | 37 of 40 (92.5%) (§40.8) | held |
| P261 | *(written before the mutants)* The likeliest survivors are L11 (no redraw on scrolling, since the harness redraws every frame it captures) and L12 (the lower pair pointing up, since the pixel counts do not look at direction) | both survived; F20, not named, survived too (§40.8) | held for the two it named |
| P262 | *(for the handheld)* In Game Mode, with Mistress launched as a non-Steam shortcut, `steam://open/keyboard` opens Game Mode's keyboard over Mistress | it does, measured on the handheld on 2026-09-27 (§40.16) | held |
| P263 | *(for the handheld)* It takes the lower part of the screen, and the popup's field, centred at about 380 of 800 lines, stays above it | it takes roughly the lower half, and about half of the centred popup was behind it (§40.16) | held for the keyboard's place, failed for the popup's; the popup was moved |
| P264 | *(for the handheld)* What is typed on it arrives in the focused field, and its Enter closes the popup with the text kept | the text arrives in the field (§40.16); its Enter was not reported | first half held; second owed |
| P265 | *(for the handheld)* While it is open the pad drives Steam's keyboard and not Mistress; once it closes, Mistress has the pad again | not reported | owed |
| P266 | *(for the handheld)* In Desktop Mode with Steam running, the same address opens a floating keyboard that types into Mistress's focused window | not reported | owed |
| P267 | *(written before the broad run)* The broad Mistress run, after merging WiseMan, fails on nothing this branch causes | 1,449 tests, none failed (§40.10) | held; one run is weak evidence against an intermittent failure |

The handheld was offline, and nothing here ran on it. P262–P266 are what to look at there, in that order; each is a
prediction, not a result. *Amended 2026-09-27: Game Mode was measured on the handheld later that day; the rows above
record what was reported, and §40.16 what was changed because of it.*

### 40.7 Tests

`MenusFollowupTests`, 26 cases, all headless:

- the choice of keyboard for every setting, Steam or not, pad or keyboard (ten rows), and Steam detected from Game
  Mode's desktop name, `SteamDeck=1`, `SteamGameId`, `SteamAppId`, the Proton variables and a running client, and not
  from a desktop without them or from an empty variable;
- the harness sees no Steam and launches nothing;
- Enter on a text row with no pad opens the popup with its field focused; typed keys reach the field (Backspace erases
  rather than leaving); Enter keeps the text in the row and the draft; nothing is asked of Steam; the next Down moves
  at once;
- Escape drops what was typed and leaves the editor and big picture where they were;
- with *Steam*, A asks for exactly `steam://open/keyboard`, typed text arrives, A is not taken, Y asks again, Start
  keeps, B drops;
- *EmuSen's* opens Mistress's keyboard from a pad and from Enter, and asks nothing; Automatic with a pad and no Steam
  opens Mistress's keyboard;
- Preferences' Controllers screen has the row, as an option row with its three choices, and stores each;
- the subtitle for SNES, NES, a Game Boy file and an unknown file;
- the help bar's words, order and buttons on every kind of row and every button, and Reset added after Scrape;
- A on the stars: half a star, five at the tenth press, none at the eleventh;
- the scroll indicator in the editor at both sizes: down at the top, both in the middle, up at the end, placed as
  measured, and every pixel it changes inside its squares;
- the indicator on Theme Settings' Interface screen, none on Preferences' first, and the Start menu's through its list;
- the help bars filled in the themed view and the editor at both sizes: the A glyph's square at least 30% ink and more
  than twice what the outlined set drew.

Existing tests changed: `MetadataEditorLayoutTests` reads the one-line subtitle; `ThemedLibraryHostTests`' family case
expects no change of pixels from Generic to Xbox, which are now the same drawing. LunaP's `MenuFollowupTests` adds
eleven cases (its §195.4).

### 40.8 Mutants

Forty mutants, one at a time, by `~/.cache/emusen/probe/menus-followups/mutate.py`: the state file before each mutant,
the leftover restored at start, every restored file touched (§32.5's trap). Twenty-five are in Mistress, run against
`MenusFollowupTests`, `MetadataEditorLayoutTests`, `ThemedLibraryHostTests`, `ThemedGameOptionsTests` and
`EsdeSettingsMenusTests`; fifteen in LunaP, against its `MenuFollowupTests`, `PadGlyphTests` and `MenuTests`.

| # | Rule broken | Result |
|---|---|---|
| F1 | the themed view's help outlined | caught by 2 |
| F2, F3 | the subtitle's system in lower case; the subtitle upper-cased | caught |
| F4–F9 | the stars' A called *Select*; a switch's called *Select*; Save's called *Save*; *Choose* up and down on the buttons; *Change* never offered; the help bar changed only for Reset | caught, each by the words case |
| F10, F11 | A on the stars does nothing; the stars not starting again past five | caught |
| F12, F13 | Steam never asked; another address | caught |
| F14–F18 | Automatic ignoring the pad; ignoring Steam; `SteamGameId` not read; a running client not counted; every row taken as chosen by a pad | caught; F14 by ten |
| F19 | the popup's keys taken by the menu | caught by 2 |
| F20 | a steering key's release lost under the popup | **survived**; caught once the case was strengthened |
| F21–F23 | the router passing the popup by; B not cancelling; Y not asking again | caught |
| F24, F25 | the *EmuSen's* setting ignored; the setting not stored | caught |
| L1–L10, L13–L15 | LunaP (its §195.4) | caught |
| L11, L12 | no redraw on scrolling; the lower pair pointing up | **survived**; caught once the cases were added |

**37 of 40 caught on the first valid run (92.5%)**, so P260 held. P261 named L11 and L12 as the likeliest survivors, and
both survived, for the reasons given; it did not name F20, which also survived. F20 left Enter held in the window's
steering keys when it was let go under the popup; the next press of Enter, after the popup closed, let it go again, so
every case still passed. The Escape case now presses Enter a second time after Escape and requires the popup to open;
with F20 it does not. L11 and L12 are LunaP's (its §195.4). Rerun alone after the changes, all three were caught, each
by the case written for it.

### 40.9 Pictures

In `~/.cache/emusen/bigpicture/png/menus-followups/`, from `MenusFollowupPictureTool` (`EMUSEN_BIGPICTURE_PNG=1`). For
each of `synthetic` and `artbooknext`, at each of `1280x800` and `1920x1200`:

- `-gamelist`, and `-gamelist-xbox`, `-playstation`, `-nintendo`, `-generic` and `-xbox-swapped`, the help bar in each
  family by the Controller Type setting;
- `-start-menu`, its lower chevrons, and `-start-menu-last`, the last entry and the upper ones;
- `-editor`, the one-line subtitle and *Select*; `-editor-rating` after three presses of A (*Add Half Star*, *Reset*,
  *Change*); `-editor-switch` (*Toggle*); `-editor-scrolled` and `-editor-lower` with both pairs; `-editor-bottom`
  scrolled to the end, the upper pair alone; `-editor-buttons` (*Save Metadata*, Choose left and right);
- `-editor-field`, the text popup with *Halcyon Works* typed from the keyboard; `-editor-steam`, the popup as Steam's
  keyboard would type into it, with its line of pad buttons;
- `-interface`, Theme Settings' Interface screen and its chevrons.

`glyph-sheet.png` is every button of every family, filled and outlined, at 44 px. Three side-by-sides against ES-DE at
1280 by 800: `side-by-side-help-bar.png` (the editor's help on three rows, the gamelist's in four types and swapped),
`side-by-side-scroll-indicator.png` (the title's right, three times, at the editor's top, middle and end and on UI
Settings against Interface), and `side-by-side-editor.png`. Every picture was looked at. What they show that is not a
defect:

- Art Book Next's own help element lists three entries (Options, Menu, Launch), so its gamelist pictures show those
  alone.
- At 22 px the d-pad's up-and-down and left-and-right glyphs differ only at the arms' tips, as ES-DE's do.
- The popup's field is drawn in the application's typeface, not Barlow Condensed (LunaP §195.3).
- ES-DE's *Choose* on the editor's buttons is a left-and-right d-pad, and so is Mistress's.

### 40.10 The broad run

WiseMan was merged at `2a317145` before the run (it brought §4.78's headless-test guard, `OffDispatcherTests`, which
passes with this branch's tests); LunaP's `openemu-library` had not moved from `18413c7`. One run of the Mistress
filter without any shader, GPU, Vulkan or bench test, under `nice -n 10`, on the final build of both branches:
**1,449 tests, 1,405 passed, 44 skipped (the picture, survey, bench and live tools, this section's picture tool among
them), none failed, in 4 min 22 s.** P267 held; one run is weak evidence against an intermittent failure. LunaP's whole
suite on its branch: **1,442 tests, all passed**.

### 40.11 Not done

- **Anything on the handheld.** Steam's keyboard was never seen: P262–P266 are owed, and until they are checked
  *EmuSen's* is the choice known to work with a pad. *Partly closed 2026-09-27: P262 and the keyboard's half of P263
  were measured there (§40.16); P264's Enter, P265 and P266 are still owed.*
- The themed view's search box and the cheats window keep Mistress's keyboard in every mode (Q164). *The search follows
  the setting since 2026-09-27 (§40.14); the cheats window still keeps Mistress's keyboard.*
- The popup's field is drawn in the application's typeface (LunaP §195.3).
- ES-DE's X/Y swap (Q160) and its Switch Pro lettering (Q161) are recorded, not followed. *The X/Y swap is followed
  since 2026-09-27 (§40.13); the lettering stays Mistress's, as Q161 decided.*
- ES-DE's own text popup's help (*Apply*, *Backspace* on L, *Space* on R, *First* and *Last* on the triggers, *Move
  Cursor*; `rating-wrap/p00_popup.png`) belongs to its keyboard, which Mistress's popup has none of; the popup's line
  names the keys it does take.

### 40.12 Open questions

- **Q160, the swap.** ES-DE's swap trades X and Y as well as A and B; Mistress's trades A and B only, so under the swap
  its help names Y for Favourite where ES-DE's names X. **Recommendation:** match ES-DE, both pairs, since the swap
  exists for a Nintendo layout, whose X and Y are also reversed; the help follows by itself. *Decided 2026-09-27 as
  recommended; built in §40.13.*
- **Q161, Nintendo's letters.** ES-DE letters a Switch Pro as SDL 2 reports its printed labels (Select on A) and a SNES
  pad by position (Select on B). Mistress's Nintendo family letters by position, which is what the button under the
  player's thumb is called on every Nintendo pad Mistress routes. **Recommendation:** keep it; the swap is what makes A
  the right-hand button's job.
- **Q162, the words for Clear and Hide.** *Clear Metadata* and *Hide Game* where ES-DE says *Clear File* and *Delete
  Game*. **Recommendation:** keep, since ES-DE's words would say that a file is touched or deleted.
- **Q163, a text row's word.** The request's example was *Edit*; ES-DE says *Select*, and Mistress says *Select*.
  **Recommendation:** keep ES-DE's.
- **Q164, the other text boxes.** The themed search and the cheats window keep Mistress's keyboard, since neither is a
  big-screen row; the cheats window's codes need its Code layout, and its look is being redone separately. Should the
  search follow the On-Screen Keyboard setting too? **Recommendation:** yes, after P262–P266 are checked, with the
  cheats window left to its restyling. *Decided 2026-09-27 as recommended; built in §40.14, once P262 had been
  measured.*
- **Q165, Automatic with a keyboard under Steam.** On a desktop with the Steam client running, Automatic asks for
  Steam's floating keyboard even when the row was chosen with Enter on a physical keyboard. **Recommendation:** under
  Steam, a row chosen from a keyboard gets the field alone, and Steam is asked only for a pad; checked with P266.
  *Decided 2026-09-27 as recommended; built in §40.15. P266 is still owed.*

### 40.13 Q160: the swap trades X and Y as well

*Built on branch `bigpicture-q160-q165`, from WiseMan at `4e45c49d`; LunaP unchanged at `openemu-library`'s `c20856f`.*
Decided 2026-09-27 (§10.1): *Swap the A/B and X/Y buttons* trades both pairs, as ES-DE's `InputSwapButtons` was
measured to (§40.1). Under the swap East chooses, South goes back, West does what North did and North what West did.

**What the pad reads.** `PadHeldOn` is where a face button becomes an interface function, and nothing else reads the
face buttons for the interface. With the swap on it now reads Search (the favourite in the themed gamelist, §4.58; the
collection toggle while one is edited; *go to* on the slideshow; *Scrape* in the editor; *Space* on Mistress's
keyboard; *Keyboard* on the text popup) from West, and the X function (the screensaver's start in the system view, §37;
*Reset* in the editor, Q101) from North. Every place listed follows from that one mapping, which is why none of them
changed. The pad menu uses A and B alone. The keyboard is unaffected, as ES-DE's is (Insert and Delete stay Search and
the X function), and so is every game, whose bindings are its own.

**What the help draws.** A help entry names the positional button (`PadGlyphButton.South` … `West`) and LunaP's
filled set letters it for the family, so the swap is Mistress's to apply before LunaP sees the entry; LunaP did not
change. `PadHints.Of` and `PadHints.Glyph` give the button a function is on under the swap, both pairs, and every
big-screen entry that names a face button goes through them: the editor's *Select*, *Back*, *Scrape* and *Reset*, the
pad menu's and Gamelist Options' *Select* and *Back*, an option list's and a menu message box's. The themed view's
`HelpPrompts` moved North and West as it already moved South and East, and a theme's own icon moves with its function,
`button_y_` to `button_x_`, as `button_a_` already went to `button_b_`. So with the swap on and an Xbox pad the gamelist
shows *B Launch*, *A Back*, *X Favorite*, the system view *Y Screensaver*, and the editor *X Scrape*, *Y Reset*: the
letter drawn is always the button that acts.

**The text hints** trade X and Y as well as A and B (`PadHints.Face`): Mistress's keyboard reads *B Type, A Erase,
X Space*, and the Steam popup's line *A Cancel, X Keyboard*. Two sentences name buttons in prose and follow through
`PadHints.Letter` and `Glyph`: Interface Settings' *Screensaver Controls* (*Y in the system view starts it … B starts
the game shown and X goes to it*), and the collection line, which names West under the swap.

§24.5's reason for narrowing the swap was that it must not disagree with whatever North and West came to mean. It is
retired, not refuted: the swap moves a function with its button, so it composes with any assignment that gives each
button one function, and that is the assignment Mistress has.

### 40.14 Q164: the themed search follows the On-Screen Keyboard setting

Decided 2026-09-27 (§10.1), once P262 had been measured. `PadKeyboard.FollowSetting` marks a text box that is not a
big-screen row but takes the setting, with its popup's title; the themed search box is the one box so marked, titled
*Search*. When it opens, the window chooses exactly as for a row (`KeyboardFor`: the setting, whether Steam is there,
and whether Enter chose it):

- **Steam's keyboard or the field alone:** `MenuTextPopup` over the search box. What is typed filters the list when it
  is kept (Enter, or Start from a pad) and not before; Escape or B leaves the filter as it was.
- **Mistress's keyboard:** as before, filtering the list at every key.

The difference is deliberate, not a limitation found: the popup edits a copy and writes it back on Enter (LunaP
§195.3), and a live filter under a popup the list is shaded behind would show nothing the player can read. The search
bar stays up while either is open. The cheats window's boxes are not marked and keep Mistress's keyboard in every mode:
its codes need the Code layout, which a system keyboard does not have, and its look is §41's to settle.

### 40.15 Q165: a keyboard's Enter never asks Steam

Decided 2026-09-27 (§10.1). In Automatic, `DeviceKeyboard.Choose` now asks first which device chose the row and only
then whether Steam is there:

| Automatic | Off Steam | Under Steam |
|---|---|---|
| chosen with Enter | the field alone | the field alone (was Steam's keyboard) |
| chosen with a pad's A | Mistress's keyboard | Steam's keyboard |

*Steam* and *EmuSen's* are unchanged, so *Steam* still asks Steam for an Enter: the setting is the player's explicit
choice. The reasoning is §40.12's: a player at a physical keyboard has one, and a floating keyboard over the field can
only hide it; on the handheld, Steam's hides half the screen (§40.16). P266, which concerns this case on a desktop with
Steam running, was not reported and is still owed; the change does not depend on it.

### 40.16 Steam's keyboard on the handheld, and the popup above it

**Measured on the handheld on 2026-09-27**, in Game Mode with Mistress started as a non-Steam shortcut:
`steam://open/keyboard` opens Game Mode's keyboard over Mistress, and what is typed on it reaches the popup's field.
The keyboard covers roughly the lower half of the screen, and about half of the popup, which §40.5 centred, was behind
it. P262 held; P263 held for the keyboard and failed for the popup; P264's first half held (§40.6). The handheld runs
1920×1200.

**What was built.** When Steam's keyboard is asked for, `PadKeyboard` puts the popup at the top of the window, its top a
tenth of the height down (`SteamPopupTop`), centred across as before. Measured headless on the final build:

| Window | Popup, Steam's keyboard asked for | Popup otherwise |
|---|---|---|
| 1280×800 | top 80 (10.0%), bottom 278 (34.7%) | centred, 301 to 499 |
| 1920×1200 | top 120 (10.0%), bottom 417 (34.7%) | centred, 452 to 749 |

Both are clear of a keyboard that covers the lower half with room to spare, and nothing in the popup is cut or
overlapped. Only the Steam case moves: from a physical keyboard (under Steam too, §40.15) and with Mistress's keyboard
the popup is where it was. The place is not taken from the keyboard's real height, which Steam does not give a program
it did not start as a Steam game (§40.5); a keyboard taller than about 65% of the screen would reach the popup again.
Whether the handheld's keyboard leaves the moved popup whole is P268 below; it held on 2026-09-28.

### 40.17 Tests

`SwapAndKeyboardTests`, 18 cases, headless, with Steam simulated for a case by a running client or `SteamGameId`, put
back to the harness's `NoSteam` after:

- **Q160:** both pairs traded in `PadHints.Of`, `Glyph`, `Face` and `Letter`, and nothing else moved; in the gamelist
  under the swap X toggles the favourite and Y does nothing, the help names B Launch, A Back, X Favorite, and the glyph
  drawn for Favorite changes only inside its square when drawn as Y instead; in the system view Y starts the
  screensaver and X does not, and the help names Y; in the editor the help reads *X Scrape*, *Y Reset*, and Y resets
  an edited switch;
- **Q164:** the themed search under every setting, from a pad and from Enter, under Steam and not (eight rows): which
  keyboard, the popup's title, hint and focus, and what was asked of Steam; Enter's search types into the field,
  filters on Enter and keeps the filter on a second search's Escape; with *Steam*, the pad's search asks Steam twice
  (A, then Y) and Start keeps the text;
- **Q165:** under Steam, by a running client and by `SteamGameId`, Enter on the editor's Name gets the field and asks
  nothing, and a pad's A on it asks Steam once;
- **the popup's place:** at 1280×800 and 1920×1200, with Steam's keyboard asked for its top between 8% and 12% and its
  bottom above 45%, centred across; from a keyboard, off Steam and under it, centred as before and the same size; in
  both, the popup inside the window and its title, field and line inside it, whole and not over one another.

`PadCheatsTests` gained one case in two rows: the cheats window's code and description boxes open Mistress's keyboard
with *Steam* and with Automatic under Steam, and nothing is asked of Steam. Existing cases changed: `ControllersTests`'
swap case now expects X to search and the keyboard's hint to read *X Space*; `ThemedControllersTests` expects a
theme's `button_x_` icon and West for Favorite under the swap; `MenusFollowupTests`' table expects the field for Enter
under Steam in Automatic.

### 40.18 Mutants

Twenty-three mutants, one at a time, by `~/.cache/emusen/probe/q160/mutate.py` (§40.8's runner, trimmed to one tree),
against `SwapAndKeyboardTests`, `MenusFollowupTests`, `ControllersTests`, `ThemedControllersTests` and the cheats case.
Twelve break the swap (S1–S12), five the search (K1–K5), two Q165 (E1, E2) and four the popup's place (P1–P4).

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P268 | *(for the handheld)* With the popup moved, Game Mode's keyboard leaves all of it, title, field and line, in view at 1920×1200 | checked on the handheld 2026-09-28, with the four-line field at 5% | **held** |
| P269 | *(written after the run was started and before any of its results were read)* Of 23 mutants, at least 21 are caught on their first valid run; the likeliest survivor is K3 (the search bar hidden under the popup), whose only witness is one visibility assertion made after the view has drawn | 23 of 23 caught on the first run; K3 was caught, by two cases | held for the count; the survivor it named did not survive |

| # | Rule broken | Result |
|---|---|---|
| S1, S2 | the pad's Search, or its X function, read from the unswapped button | caught by 3; by 2 |
| S3, S4 | North, or West, not traded in `PadHints.Of` | caught, each by 2 |
| S5, S6 | the hints' X and Y left alone by the pattern; the letters not traded | caught, each by 2 |
| S7, S8 | the themed help's North, or West, not moved | caught by 3; by 1 |
| S9 | a theme's `button_y_` icon kept for a function moved to West | caught |
| S10, S11 | the editor's *Scrape*, or *Reset*, glyph unswapped | caught |
| S12 | `Glyph` ignoring the swap | caught by 2 |
| K1, K2 | the search not marked; marked boxes ignored when a keyboard opens | caught, each by 3 |
| K3 | the search bar hidden while the popup is open | caught by 2 |
| K4 | the search's popup titled *Enter Search* | caught |
| K5 | every text box following the setting, the cheats window's too | caught, by the cheats case |
| E1 | Automatic asking Steam for an Enter under Steam (the old rule) | caught by 4 |
| E2 | Automatic never asking Steam for a pad | caught by 3 |
| P1–P4 | the Steam popup centred; every popup moved up; its top at 30%; its top at the window's edge | caught, each by the placement case |

**23 of 23 caught on the first valid run.** P269 held for the count. The survivor it named was caught: the search's
Enter case asserts the bar is visible while the popup is open, before anything else draws, which is the moment K3
breaks. The runner restored every file, touched each, and rebuilt clean; `--check` found every snippet once afterwards.
A clean record is weak evidence of its own: the mutants were written by the same hand as the cases, after them.

### 40.19 Pictures

In `~/.cache/emusen/bigpicture/png/q160/`, from `Q160PictureTool` (`EMUSEN_BIGPICTURE_PNG=1`), at 1280×800 and
1920×1200. `EMUSEN_Q160_TAG` names the build: `before-` is WiseMan at `4e45c49d` with the tool copied in, built in a
throwaway worktree since removed; `after-` is this branch. With the swap on and the Xbox family: `-system-swapped`,
`-gamelist-swapped`, `-editor-swapped` (an edited switch, so *Reset* shows) and `-controllers-swapped` (the switch's new
words). Then `-editor-steam` (the popup with Steam's keyboard asked for), `-editor-field` (from a keyboard),
`-search-field` and `-search-filtered`. `side-by-side-help-<size>.png` stacks each help bar before over after. Every
picture was looked at:

- Before, the swap names *Y Favorite*, *X Screensaver* and *Y Scrape*, *X Reset*; after, *X Favorite*, *Y Screensaver*,
  *X Scrape*, *Y Reset*. A and B are the same in both.
- The Steam popup sits in the upper third over the shaded editor, whose title shows above it; the field-alone popup
  and the search's are centred, as before.
- The synthetic theme's gamelist help runs past the right edge at 1280×800 in both builds. It is that test theme's
  element, sized by its `theme.xml`; Art Book Next lists three entries there (§40.9), and nothing here changed it.

### 40.20 The broad run

WiseMan had not moved from `4e45c49d` when the run began, nor LunaP's `openemu-library` from `c20856f`, so there was
nothing to merge. One run of the Mistress filter without any shader, GPU, Vulkan or bench test, under `nice -n 10`, on
the branch's code commit, measured on 2026-09-27: **1,486 tests, 1,436 passed, 50 skipped (the picture, survey, bench
and live tools, this section's picture tool among them), none failed, in 4 min 1 s.** No prediction was written for
it; one run is weak evidence against an intermittent failure.

**Checked on the handheld, 2026-09-28.** With Steam's keyboard up in Game Mode, the four-line popup at 5% of the
height sits wholly clear of the keyboard. P268 held.

## 41. Every window in ES-DE's look, first part: the style layer and three windows (2026-09-27)

*Built on branch `bigpicture-window-look`, from WiseMan at `8539c7e6`; LunaP on `window-look`, from `openemu-library`
at `18413c7`.* The decision is §10.1's *"The look of every window"* (2026-09-27): the windows are not converted one for
one into ES-DE's menus. Each keeps what it is for and takes ES-DE's look: its colours, the Barlow Condensed face and the
rounded panels. Each also gets a layout a controller can use, wherever one can be had.

The work stops at a checkpoint: the style layer and three windows that stand for the rest, so that the look can be
approved before the others are done. The three are:

- the scraping status (a status page with a list);
- Active Cheats (tabs, text boxes and a table);
- the Cheat Database (a path, two lists and a row of buttons).

The controller bindings were to be the third; decided the same day that their own overhaul, a drawn controller with the
bindings on it, takes the look with it, so they keep the plain sheet here. The player's account is the settings
reference §4.80; LunaP's record is its §196.

**Numbering.** Predictions from P270, questions from Q170; both ranges were free in every tree on this machine when this
began.

**Sources.** No new capture of ES-DE was taken. The look's measurements are §32.1's and §34.12's, and its colours are
`MenuPanel`'s and `MenuRow`'s defaults, which those sections set (LunaP §181). No file of ES-DE's was read or copied.

### 41.1 Which windows a big-screen session shows, and which it does not

A survey of `EmuSen.Mistress/Views/` found where each window is opened and whether that path is reachable in a
big-screen session. In one, every window owned by the main window is a sheet on its `SheetLayer`.

| Reachable, as a plain sheet until now | Not reachable in a big-screen session |
|---|---|
| Active Cheats, Cheat Database, the scraping status, Find by Name, Use Another Game's Cover, the gamelist filter, the folder editor, Graphics Settings, Shaders, the resume question, the rewind reel, a screenshot (a sidebar library only), the theme browser, a theme's detail, its About | the ROM browser (File ▸ Browse ROMs…), the runtime dashboard (`VstopWindow`, from Settings and the `vstop` command), Debug Logging, the DianaOS console: all four are opened from the menu strip, which a big-screen session hides |

The option list screen (§34.15) is already an ES-DE menu, and the controller bindings are another task's (above). The
four unreachable windows are left as they are, as §10.1 asked for the developer tools.

### 41.2 One style layer

Two mechanisms, both in LunaP (§196), so that nothing is written twice per window:

- **The look** (`MenuLook`) is a `Styles` of the menus' palette and typeface for stock controls, added to one element's
  own `Styles`. Buttons, lists, text boxes, dropdowns, tabs, switches, sliders, check boxes, scroll bars and progress
  bars under it take:
  - ES-DE's greys;
  - Barlow Condensed, through the resource key `LunaMenuFontFamily`, which Mistress fills from a copy of its font
    shipped as an Avalonia resource; LunaP ships no font;
  - rounded corners;
  - upper-case words on buttons, tabs and headings;
  - the full-width bar on a list's chosen or focused row.
- **The frame** (`SheetLayer.MenuLook`) presents a sheet in a `MenuPanel`:
  - titled by its window;
  - its content laid out under the menus' scale;
  - its help bar the pad's buttons;
  - no fill of its own, so the screen shows blurred behind it, as behind the menus.

Mistress turns the frame on with big screen and names, in `MenuFrameFor`, the windows checked so far. It builds a
window's help bar from what the window holds: *Select*, *Back*, *Tab* where there are tabs, *Change* where a value steps
sideways, and *Choose*. The backdrop's blur and ES-DE's keys count a framed sheet as a menu.

The desktop is unchanged. The look is one attached property, so a desktop window could take it with one call, which is
how a later desktop setting would work; that setting is not built (§10.1).

### 41.3 What each window changed, and why

What the style could not reach, each window changes in `MenuLook.WhenApplied`, which runs once when the window is first
shown in the look.

- **Scraping.** The first version kept the one column, and its pad test failed. The recent games list lay below the
  fold at 1280×800, and no press reached a row, since a row scrolled out below the viewport is geometrically below the
  buttons. The window is now two columns, the run at the left and the recent games at the right (0.9 of the width).
  Every row is in view and in reach. The heading is 30 design pixels and the game's name 28; the buttons are centred.
- **Active Cheats.**
  - *Save As…* and *Load From…* open the platform's file dialog, which a pad cannot drive in Game Mode, so they are
    hidden on a sheet; *Save* and *Load*, the game's own list, stay.
  - The table's *Kind* and *Code* words, set to 11 pixels in code, are bound to a size that is 20 in the look.
  - The status line sits over the buttons, centred.
- **Cheat Database.**
  - 0.8 of the width.
  - The folder's label beside its box, as a menu row's is, and the folder typed on the on-screen keyboard; *Browse…* is
    hidden for the same reason as above.
  - The two lists the same width.
  - The status over the centred buttons.
  - The licence's attribution, which must stay in view, as the panel's footer: `MenuLook.Footer`, three lines of 20
    design pixels.

  At 1280×800 the lists show three systems and two games; Q170 asks about the room.

### 41.4 A defect found by the pictures: rows left unbuilt under a scaled frame

At 1920×1200 the framed Active Cheats drew one of its two cheats, with room for both. It was measured before any fix,
by a throwaway probe test on 2026-09-27:

- the table's list was 774 by 85 pixels, and its scroll viewer's viewport the same;
- its `VirtualizingStackPanel`'s own viewport was 524 by 39.3, and one container was built;
- at 1280×800 both were built.

39.3 is the frame's content host (464 high, clipping) divided by the scale 1.5 once more, less the list's top at 270. So
Avalonia 12.1's effective viewport mis-scales the clip of a clipping control directly under a `LayoutTransformControl`.
That is inferred from the numbers; Avalonia's code was not read.

With the host no longer clipping (the scaler still does), the viewport was 774 by 85 and both rows were built. LunaP's
`MenuLookTests` holds a list at the foot of a sheet scaled 1.5 and requires all three of its rows built; mutant L1
below puts the clip back.

The plain sheet frame has the same structure and was not changed. Whether its lists lose rows was not measured (Q175).

### 41.5 Predictions

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P270 | *(written as the pixel case's assertion, before it first ran)* The three framed windows draw nothing outside the panel and the help bar at 1280×800 and 1920×1200 | 0 pixels at both sizes for all three, once the case made the panel transparent instead of hidden (below) | held |
| P271 | *(written at this checkpoint, before the broad run that ends the work)* The broad Mistress run, after the other windows, fails on nothing this branch causes | not yet run | open |

No prediction was written before the first narrow run or before the mutants; both are recorded as found, in §41.6
and §41.7.

P270's first case hid the panel to compare. With the panel hidden, the focus fell to a button of the window behind,
and that button's focus ring was drawn in the blurred backdrop: 3,307 pixels in a strip across the top of the scraping
status's screen. The strip was the case's own doing, not the frame's. The case now makes the panel transparent, which
keeps the focus where it was.

**The first narrow run, with every sheet framed.** Before `MenuFrameFor` existed, every plain sheet of a big-screen
session was framed. Six of 679 tests failed:

- one scraping-status pad case, the recent games below the fold (§41.3);
- five Graphics Settings and Shaders cases, of pad reach and scroll positions, in windows not yet checked in the look.

The frame then went to the checked windows only. The Graphics and Shaders cases wait for those windows' turn.

### 41.6 Tests

- **`SheetLookTests`** (eight cases) and **`ScrapeStatusLookTests`** (two), headless. The detail is in §4.80.5:
  - the frame at both sizes;
  - the §32.4 pixel rule;
  - the targets at least 40 pixels tall at 800 lines;
  - the focused button's fill and the focused row's bar;
  - the file dialogs gone, the attribution the footer, the width shares, the two columns;
  - every control reached by the pad, and B out of each;
  - the shoulders paging a list of 300;
  - ES-DE's keys with a text box keeping its typing;
  - the desktop and the controller bindings unchanged.
- **LunaP's `MenuLookTests`**, six cases (its §196.6).
- **No existing test's expectation changed.** The pad router now pages a list with L1 and R1 where a window has no
  tabs; no existing test pressed them there.
- **A narrow run** of the Mistress tests in the blast radius: every pad, sheet, cheat, scrape, ES-DE menu, themed,
  cover, rewind, resume and theme-browser test, without shaders, GPU, Vulkan or benches. **693 tests, 672 passed, 21
  skipped** (the picture tools), none failed, in 3.1 minutes, on the final build after the mutants' clean rebuild.
- **LunaP's whole suite**, on the same build: 1,434 tests, all passed, once the API baseline, the documented defaults, two parameter
  descriptions and the README's count were brought up to date.
- **After merging WiseMan at `813c8549`** (§40's follow-ups; one conflict, the text-box line of `SetButtonFromKey`,
  which now keeps both §4.79's popup rule and §4.80's framed box) **and `openemu-library` at `b1fb12e` into LunaP**,
  the same narrow run: **698 tests, 676 passed, 22 skipped, none failed**, and LunaP's whole suite, **1,448 tests, all
  passed**.

### 41.7 Mutants

Twenty-six, one at a time, by `~/.cache/emusen/probe/window-look/mutate_look.py`. It writes a state file with the
original text before each mutant, restores any leftover one when it starts, and touches every restored file (§32.5's
trap). Fourteen are in Mistress, run against `SheetLookTests`, `ScrapeStatusLookTests` and the scraping status's pad
case. Twelve are in LunaP, run against `MenuLookTests`.

**All twenty-six were caught on their first valid run, each by the case written for it.** W7 and W13 were also caught by
the shoulders' case, since a narrower panel or a status beside the buttons changes how many games a page holds.

| # | Rule broken | Caught by |
|---|---|---|
| W1 | the layer never frames a sheet | four cases |
| W2 | the help bar never names the tabs | the cheats' frame |
| W3 | the cheat database left in the plain frame | the database's frame |
| W4 | *Save As…* and *Load From…* offered on a sheet | the cheats' frame |
| W5 | the attribution not in the footer | the database's frame |
| W6 | *Browse…* left shown | the database's frame |
| W7 | the database's status beside its buttons | the database's frame, and the shoulders' |
| W8 | the scraping status in one column | its frame, and the existing pad case |
| W9 | the shoulders do not page a list | the shoulders' |
| W10 | a framed text box gives its keys to the menu | the keyboard's |
| W11 | ES-DE's keys drive chromeless sheets only | the keyboard's |
| W12 | a framed sheet not blurred behind | three frames |
| W13 | the cheat database at a menu's width | the database's frame, and the shoulders' |
| W14 | the controller bindings framed too | the desktop and bindings case |
| L1 | the frame's host clips again (§41.4) | the list at the foot of a scaled sheet |
| L2 | the look's styles left behind when it is taken away | the stock controls' |
| L3 | a button's words not upper case | the stock controls' |
| L4 | the typeface's key not used | the stock controls' |
| L5 | the chosen row not the bar | the stock controls' |
| L6 | a focused button not filled | the focused button's |
| L7 | the host's choice of frame ignored | the framed sheet's |
| L8 | the layer's help never asked for | the framed sheet's |
| L9 | the footer kept to one line | the framed sheet's |
| L10 | the content not scaled with the menus | the scale's |
| L11 | a window's width share ignored | the framed sheet's |
| L12 | `WhenApplied` runs outside the look | its own, and the stock controls' |

Both trees were rebuilt clean at the end (`clean rebuild rc=0,0`).

### 41.8 Pictures

In `~/.cache/emusen/bigpicture/png/window-look/`, from `WindowLookPictureTool` and `WindowLookScrapePictureTool`
(`EMUSEN_BIGPICTURE_PNG=1`, the folder named by `EMUSEN_WINDOW_LOOK_STAGE`):

- `before/` was taken on the unmodified tree, and `after/` on the final build;
- each at 1280×800 and 1920×1200;
- the themed windows over the synthetic theme and over Art Book Next (read in place), the in-game ones over a running
  synthetic game.

The three windows of this part:

- `scrape-status-*` and `scrape-status-finished-*`;
- `active-cheats-*`, `active-cheats-console-*`, `active-cheats-row-focused-*` and `active-cheats-button-focused-*`;
- `cheat-database-*`, `cheat-database-system-*` and `cheat-database-game-focused-*`.

The other windows' pictures are the plain sheet in both folders. Where they differ, a pixel comparison places it in the
running game's frame, a clock or a date, not in the window.

Every picture was looked at. Three things seen in them are questions rather than fixes (§41.10):

- the database's short lists at 1280×800;
- the table's column headings, which stay as written, since the table draws them itself;
- the empty 96-pixel slot the scraping status keeps for the current game's picture before it arrives.

### 41.9 Not done

- **The other twelve windows** of §41.1, until this part's look is approved.
- **The plain frame's viewport** (§41.4).
- **A desktop ES-DE look**, deliberately (§10.1).
- **A real pad, keyboard or handheld.** Nothing here ran on hardware.
- **The broad Mistress run**, which is for the end of the work, not the checkpoint.

### 41.10 Open questions

- **Q170, the Cheat Database's room.** At 1280×800 the two lists show three systems and two games. Two lines of
  introduction sit above them, with the attribution in the footer. **Recommendation:** put the introduction into the
  footer before the attribution, four lines, and give the lists two more rows.
- **Q171, text size.** The body of a framed window is 24 design pixels, headings 28 and buttons 26. A menu's rows are
  27, as measured on ES-DE's editor (§34.12). Keep the denser windows at 24, or bring everything to the rows' 27 at the
  cost of more scrolling?
- **Q172, upper case.** Buttons, tabs and headings are upper case, as ES-DE's are. Body text, list rows and a table's
  column headings stay as written. Keep that split?
- **Q173, file dialogs.** *Save As…*, *Load From…* and *Browse…* are hidden on a sheet, since a pad cannot drive the
  platform's dialog. **Recommendation:** keep them hidden; a typed path on the on-screen keyboard covers *Browse…*, as
  §4.72.8's path rows already do.
- **Q174, the shoulders in the help bar.** L1 and R1 are drawn as two glyphs with one label, *Tab*. The help-bar work of
  §10.1 (letter-labelled glyphs) may give one glyph for the pair; take it when it lands?
- **Q175, the plain frame's viewport.** The controller bindings keep the plain sheet, which has §41.4's structure.
  **Recommendation:** make the plain frame's host not clip as well, with the bindings' overhaul, which owns that frame
  now.
- **Q176, keys in a text box.** In a framed window a focused text box keeps Backspace, Left, Right and Enter for typing,
  and Up and Down leave it. ES-DE edits text in a popup, where Backspace erases too. Keep?
- **Q177, the scraping status's picture slot.** Before a game's picture arrives, the slot is empty space. Collapse it
  until there is a picture?
- **Q178, the database's Load button.** In `cheat-database-game-focused-*`, a game row has the pad's focus and the bar,
  but *Load into Active Cheats* is still disabled. It follows the list's selection, and the picture suggests the focus
  reached the row without selecting it. This was seen in the pictures, not measured. A on the row loads the game
  either way (§4.45.5). Enable the button from the focused row as well, or select the row the focus reaches?
## 42. The controller bindings window: drawn controllers, rebinding from the drawing, and an input tester (2026-09-27)

*Decided 2026-09-27 (§10.1):* the Controller Bindings window is overhauled. Each console's controller is drawn as vector
art with the bindings mapped onto its buttons; a button pressed on the pad or the keyboard lights up on the drawing, so
the window doubles as an input tester, as many emulators' do; the drawings are EmuSen's own, not copied from any
emulator's, maker's or website's art. This section records the first stage: the drawing control, the Super NES and
Nintendo 64 drawings, and the window with its tester, stopped there so the drawings can be looked at before the NES pad,
the Game Boy and a modern pad are drawn in the same way. The player's account is §4.81 of the settings reference; the
control is LunaP's §198. §40 and §41, and §4.79 and §4.80, are other work's.

**Numbering.** Predictions here start at **P280** and questions at **Q180**.

### 42.1 What was built

- **LunaP's `ControllerDiagram` (§198).** A controller drawn from the toolkit's own geometry in a design space of its own,
  fitted to the space between up to four bands of labels. Every button, each arm of a cross, each direction of a stick
  and each trigger is a named region that can be lit, ringed as selected, hit by a click and moved to with a direction.
  Each region has a label, a button that draws itself, showing the key and the pad button bound to it, joined to the
  region by a line. The control takes plain data (a region's two strings, pressed or not, a stick's position) and knows
  nothing of Mistress's controls.
- **The drawings.** The Super NES pad (two round grips joined by a bar, the cross in a round well, two slanted pills, four
  coloured buttons in a diamond on a rounded well, two shoulders) and the Nintendo 64 pad (a wide top on three grips, the
  cross on the left grip, the stick in an octagonal gate on the middle one with Start above it, A, B and four C buttons,
  L, R, and Z showing past the middle grip's edge). Every shape was placed by hand in code; nothing was traced, measured
  from or compared with any picture of the pads. The face buttons carry the colours the pads were sold in, because colour
  is much of what identifies them at a glance (Q183 asks which scheme for the Super NES).
- **The window.** Each console's tab is its drawing, filling the height the page shows, with the list the window had
  scrolled below it under "All Bindings". A region chosen by a click, by the pad's A or by Enter listens for a key and a
  pad button at once. The cross and the arrow keys move between regions by where they are drawn. Test Buttons (Y, or the
  button) makes every press only light up until B is held for a second or Escape is pressed. On a big-screen sheet the
  footer is the window's own.
- **The tester's input.** `GamepadManager.Poll` raises `Polled` at its end, and the window redraws from that poll; the
  main window's 16 ms tick is the only poller, as before. Keys are tracked from the window's own key events.

### 42.2 Decisions made on the way, and why

- **A mode for testing.** The pad cannot steer the window and be tried on it at once. With the default bindings the
  Super NES's A is the pad's East, which is Back, so trying it would close the window; and A would rebind whatever label
  has the focus. The tester is live whenever nothing is captured, as decided, and Test Buttons is how the pad's own
  navigation buttons are tried. Leaving it takes a one-second hold of B because a short press of B is a button being
  tried. This is Q180.
- **One capture for both.** The drawing has one place to choose a button, not a Rebind Key and a Rebind Pad, so a choice
  listens for both and binds whichever comes first. §4.45.4's rules for the pad hold unchanged; when its five seconds run
  out the key listener goes on, so a keyboard player who takes longer is not cancelled. This is Q182.
- **Player 1's pad lights the drawing.** The game hears player 1 alone (§4.61's settings reference); a drawing lit by a
  second pad would claim the game hears it. The capture still takes any pad the interface reads, as it did. This is Q181.
- **Moving by the drawing, not the labels.** The labels stand in columns, so moving between them by position gives the
  column's order, which is not the pad's: up from the Super NES's B went to A. Moving by where the buttons are drawn sends
  B up to X, X left to Y, and so round the diamond. The score is LunaP's (§198.5): the router's own score (distance
  along plus twice across) sent A up to the right shoulder, straight above and far, rather than X, above and to the left
  and near.
- **The list stays.** It is the only place a binding can be cleared, and a screen reader walks it more easily than a
  drawing; every test that used it passes unchanged, and the pad reaches it past the drawing's edge.

### 42.3 Defects found while building, each shown before its fix

- **Keys typed with the focus on nothing were not heard.** The key handler was on the window's content, where §4.45.4 put
  it so a sheet's keys reach it; a desktop window with nothing focused receives a key at the window alone. The tester's
  key test failed on it; the window now also takes a key whose source is the window itself, and only those, so a key is
  not handled twice.
- **The labels did not grow with a sheet's text.** The first big-screen pictures showed labels at nine pixels beside
  twenty-pixel text; the labels now never fall far below the text around them. A second defect hid behind the first: the
  diagram did not measure again when that text changed size, so a window whose font changed kept its old labels.
  `Labels_follow_the_size_of_the_text_around_them` failed on it (54 px high with 14 px text and with 26 px) and passes
  with the fix (54 and 66).
- **A column of labels spilled into the row below.** On a 1920 × 1200 sheet the Nintendo 64's left column overlapped the
  bottom row's first label; a column taller than the space now shrinks every label.
- **A label pressed by the router did nothing.** The router presses a focused button by raising its Click event, which
  does not call the button's own `OnClick`; the label listened in `OnClick`. It now listens to the event. The window was
  not affected, since it takes A itself, but any other host would have been.

### 42.4 Pictures

`ControllerBindingsPictureTool` (§4.81) wrote, for each drawn console, idle and lit, at 1280 × 800 and 1920 × 1200, the
window on the desktop and on a big-screen sheet: sixteen pictures in `~/.cache/emusen/bigpicture/png/bindings/`, named
`desktop-SNES-1280x800-idle.png` and so on. Lit is the pad's East, left shoulder and cross-up held, and on the Nintendo
64 the stick pushed to (0.75, −0.45) and the right stick left; on the desktop the Enter key (Start's) is held too and
the right shoulder selected, and on a sheet the window is in Test Buttons, so its footer shows. Every picture was looked at. What they show:

- Both pads read as themselves at both sizes; every label is beside its button with its line, and no two labels overlap.
- Lit buttons are the accent colour with a halo, the lit arm of a cross has the cross's rounded end, the stick's knob sits
  off centre with a line from the middle, and a lit label's badge and line are the accent colour.
- On a big-screen sheet at 1280 × 800 the drawing is small: the sheet's chrome takes a third of the height, and the
  drawing, bounded by its height, leaves the sides empty (Q184).
- A label's pad line names an unknown pad's button by SDL's position name spelled as words ("Left Shoulder",
  "D-pad Up"); a pad whose printed labels SDL knows shows those.

### 42.5 Tests

LunaP's `ControllerDiagramTests` (33 cases, §198.6) and Mistress's `ControllerBindingsDiagramTests` (13 cases, §4.81). The
window's existing tests (`InputSettingsWindowTests`, its layout and render tests, the pad tests of §4.45 including every
control of the sheet reached by the pad) and LunaP's whole suite (1,462) pass. The blast-radius run of Mistress's tests
(input settings, pad settings, controllers, the gamepad manager, accessibility and the off-dispatcher guard): 111 passed.

### 42.6 Mutants

`~/.cache/emusen/probe/bindings/mutate.py`, written in the manner of §40's runner: one exact snippet per mutant, a state
file written before each and a leftover restored at the start, every restored file touched, builds at `-m:2` under
`nice -n 10`, a hang counted as caught, both trees rebuilt at the end (both rebuilt cleanly). Twenty-five mutants of
Mistress's and Endymion's code (B1–B25), run against `ControllerBindingsDiagramTests`, `InputSettingsWindowTests` and the
Controller Bindings cases of `PadSettingsWindowTests`, and seventeen of LunaP's (L1–L17, listed in LunaP §198.7), run
against `ControllerDiagramTests`. The predictions were written first (§42.7).

**B1–B25, all caught on the first run:** the poll never announced; the N64's Z not mapped; a held key not recorded; a key
let go still lit; lit while capturing; the stick never moved; a key for a direction not pushing the stick; the stick never
standing in for the cross; a stick direction lit the wrong way; a choice listening for a key only; a key answering and
leaving the pad listening, and the reverse; testing letting the pad steer; holding B never stopping testing; any B
stopping it; testing letting keys act; A on a region not rebinding; the cross moving by the labels; Y not starting
testing; a rebound key not shown on the drawing; no prompt while capturing; C up and C down swapped; a key with the focus
on nothing not heard; the sheet's footer not set, and not given back.

**L1–L17: thirteen caught on the first run, four survived.** L2 (a cross's arm hit in the square corner outside its
rounded end), L15 (no ring for the selected region: the test counted the selected region's accent line as a ring) and
L16 (a line meeting a button's middle: nothing looked at where a line ends) were caught on the second run, by a test
clicking each arm's corner, a count on the half of a region away from its label, and a test of each line's two ends
through the new `LeaderOf`. **L7, the labels centred without the drawing, still survives**: both drawn layouts have labels
on all four sides, which leaves the composition centred before the pass moves it, so the pass moves both by less than
the line test's two pixels at all six sizes tried. It will be measurable with the first drawing that leaves a side
without labels.

**Result: 38 of 42 caught on the first valid run, 41 of 42 after the tests were strengthened.**

### 42.7 Predictions

Written before the mutants ran (`probe/bindings/predictions.txt`).

| # | Prediction | Outcome |
|---|---|---|
| P280 | Of 42 mutants, at least 36 are caught on their first valid run | **Held**: 38 |
| P281 | L16, a leader line meeting a button's middle, survives: no test looks at where a line ends | **Held** |
| P282 | L2, a cross's arm hit outside its rounded end, survives: every hit test lands in the middle of a region | **Held** |
| P283 | L7, the labels centred without the drawing, is caught | **Wrong**: it survived, for the reason in §42.6; L15 also survived, which was not predicted |
| P284 | On the handheld with a real pad, a button held lights within one 16 ms tick of the press, and the stick's knob follows without a visible lag | open until the window is run there |

### 42.8 Open questions

- **Q180, testing and steering.** The pad steers the window, and Test Buttons (Y, left by holding B) is how its own
  navigation buttons are tried. Options: keep; make rebinding a hold of A so a short A is a test; keep the tester only in
  Test Buttons. **Recommendation:** keep; it is one extra press, and the words above, below and in the footer say how.
- **Q181, which pad lights the drawing.** Player 1's only, since the game hears player 1 alone; the capture takes any.
  Options: player 1; any pad the interface reads; each pad in its own colour. **Recommendation:** player 1, with its name
  on the line below the drawing, until a second pad is player 2 (§4.61).
- **Q182, one capture or two.** A choice on the drawing listens for a key and a pad button at once. Options: keep; ask
  which, as the list's two buttons do. **Recommendation:** keep.
- **Q183, the Super NES's colours.** The drawing uses the red, yellow, green and blue buttons of the Super Famicom and
  the PAL pad; the North American pad has two shades of purple. Options: keep; the North American ones; a setting.
  **Recommendation:** keep; the four colours tell the four buttons apart at a glance, which is what the drawing is for.
- **Q184, room on a big-screen sheet.** The drawing is bounded by its height and leaves the sides empty on a wide sheet.
  Options: labels in two columns a side when there is width; the words above the tabs dropped on a sheet (done); wait
  for the shared ES-DE style layer and decide with it. **Recommendation:** the last.
- **Q185, the General tab.** The modern pad is to be drawn there as a tester of the pad itself, every button and both
  sticks and triggers as SDL reads them, beside the stick deadzone it would show. **Recommendation:** yes, after the three
  remaining drawings.

### 42.9 Not done at this stage

- The NES pad, the Game Boy and the modern pad; the NES and Game Boy tabs show their list alone until then.
- ES-DE's look on a big-screen sheet, which waits for the shared style layer.
- Clearing a binding from the drawing.
- Nothing ran on the handheld or with a real pad.

### 42.10 The second stage (2026-09-27)

*Opened on the review of the checkpoint, 2026-09-27 (§10.1):* the drawings were approved and every recommendation of
Q180–Q185 taken, with two more decisions: leader lines no longer cross the drawing, and the N64's Z gets a default pad
button, the left trigger. *Decided 2026-09-27 as well, for every window:* nothing may be cut off, overflow its panel, or
overlap other content at 1280 × 800 or 1920 × 1200; what does not fit is shrunk or reworked. WiseMan (4e45c49d, with
§40 and §41) and LunaP's `openemu-library` (c20856f, with its §195 and §196) were merged in first, and WiseMan again at
a8abe77d (Q160's swap of X and Y, and the tests' silent audio) and `openemu-library` at 4c48ba5 during the stage.

- **Lines out of the drawing.** Each label's side, and its line's path, are now worked out from the drawing itself
  (LunaP §198.2): the line leaves its button by the shortest way out of the drawing that crosses no other button, and
  goes on to its label from the drawing's edge. The first stage's hand-chosen sides are gone. Before, the N64's cross
  right arm, Stick Up and Stick Left, and the SNES's cross right arm, ran across much of the shell; now the longest part
  of any line over a drawing is the Game Boy's cross right arm, down across its lower face, because every way to the
  right crosses A or B. A LunaP test holds every line of every drawing to at most 0.35 of the drawing's larger side.
- **The NES pad, the Game Boy and a modern pad** are drawn (LunaP §198.1), in the same way and under the same rule as
  the first two: the toolkit's own geometry, nothing traced. Every console's tab now shows its drawing.
- **Z is the left trigger.** SDL has no trigger button, so a trigger past half its travel has always been read as L2 or
  R2, the N64's Z among them, on every console and whatever a binding file holds (`EmuSen_Input.md` §7.3). The binding
  was right; the window said "unbound" for it. It now shows "Left Trigger" for L2 and "Right Trigger" for R2, and
  "*button* or Left Trigger" when a button is bound beside it. Pulling the trigger while rebinding Z takes the button off
  again, leaving the trigger alone. **No binding file needs to gain anything**: the trigger is not an entry in the file,
  so an old file and a new one read it the same. The "free keys" rule of §7.4 is for keys and stick clicks, which are
  entries.
- **Q185, the pad itself on the General tab.** The Gamepad section now comes first, and beside its switch and its
  deadzone slider stands the modern pad, drawn without labels, lit by player 1's own buttons as SDL reads them whatever
  any console's bindings say; its triggers fill as they are pulled, its sticks move, and a dashed ring on each stick is
  the deadzone, which follows the slider. A connected pad's own letters, where SDL knows them, are printed on its face
  buttons. The stale hint that neither console reads an analog axis now says what the switch does.
- **ES-DE's look (the shared layer of §41).** On a big-screen sheet the window is framed as ES-DE's menus are
  (`MenuFrameFor` names it): Barlow Condensed, ES-DE's greys, the rounded panel at 0.94 of the screen's width, the title
  band, and a help bar of its own: *Rebind*, *Test buttons*, *Back*, and L1/R1 *Console*, or while testing *Hold to stop
  testing*; its Y glyph follows the A–B swap, which since Q160 trades X and Y as well. The desktop keeps LunaP's normal
  theme.

### 42.11 Q184, decided: the room on a big-screen sheet

In the frame the drawing has about 370 pixels of height at 1280 × 800: the panel's title band, the tab strip and the
help bar take the rest. What was done with it:

- the words above the tabs, the line under the drawing, and the Close and Test Buttons buttons are not shown in the
  look: the help bar does their work, and a label says "Press a key" and "Press a button" itself while it listens;
- Reset to Defaults, which resets every console at once, moves to the end of the General tab, and the footer row under
  the tabs is shown only when there is a conflict to report, so the row no longer takes height from every drawing;
- the panel is 0.94 of the screen's width, the widest the frame allows, so a wide drawing is not bounded by width as
  well;
- the labels' size may now rise to `height / 420` of the space rather than `height / 520` (LunaP §198.2). Measured on
  the pictures of the N64 tab framed at 1280 × 800: the labels were 34 pixels tall, with their binding text at about
  eight pixels, before these changes, and are 44 pixels tall, with the text at about eleven, after them; the drawing is
  smaller and still reads as its pad.

Two columns of labels a side, the other option recorded at Q184, was not needed.

### 42.12 Nothing cut off: what was found and fixed

The first pictures of the framed window broke the new rule in two places, both on the General tab, which the first
stage had not pictured on a sheet:

- **The hotkey buttons were cut.** The rows have fixed column widths, chosen on the desktop (§4.6); in the look's larger
  upper-case face, "REBIND KEY" showed as "REBIND K" and "CLEAR" as "CL", and "Pause / Resume" ran into its key. In the
  look every table's columns now take their widest cell, shared by the table's rows (`SharedSizeGroup`), with a gap
  between them; the desktop keeps its fixed columns.
- **The NES's B line was struck through the "B" printed under the button**, since words on a shell were not in the
  routing; they are now (roughly boxed) things a line may not cross, and the SNES's Select and Start lines moved above
  their printed names for the same reason.
- **The Gamepad section's first picture showed the pad's labels at six pixels**, the drawing being bounded by a 300-pixel
  height with nineteen labels around it. The tester of the pad itself needs no labels (it shows the pad, not bindings),
  so LunaP gained `ShowsLabels` and the drawing fills its box.

`In_the_look_the_window_is_a_menu_whose_help_bar_is_its_own_and_no_word_is_cut` walks every tab at both sizes and fails
for any single-line text narrower than its words. It also caught Reset to Defaults vanishing altogether on its first
version of the move above (the button bar is an items control, not a panel, so the button was never moved, and the row it
was in was hidden). Its first run reported the deadzone row's "Stick deadzone" and "50%";
both were false alarms, the measurement counting the text's margin as width, and the test now subtracts it. It is not
the window-look work's audit, which was not in WiseMan at this merge; the window will be run under that audit when it
lands.

**Pictures** (§42.4's tool, now every console and General, idle and lit, both sizes, desktop and sheet: forty in
`~/.cache/emusen/bigpicture/png/bindings/`). The tool now waits for a sheet to finish its opening animation before the
idle picture: the first set caught some panels part-way open. Every one was looked at for anything cut, overflowing or overlapping. Two
things are left, and neither is this window's alone:

- a scrolling area shows part of a row at its lower edge when its content does not end there (the General tab's
  hotkey table under the fold, on the desktop and on the sheet), which is what scrolling is; whether the audit counts it
  is Q186;
- on a framed sheet, the look's focus box around the "Use the left stick as a d-pad" switch is drawn through the foot of
  its words at both sizes; that is the look's switch style (LunaP §196), and is Q187. Giving the switch more height in
  this window did not move the box, so it was left to the style.

### 42.13 Tests

LunaP's `ControllerDiagramTests`: 51 cases, every drawing test now over all five layouts, and the line test above. LunaP
§198.9 also measured the plain sheet's rows that Q175 asked about: none is lost, and the plain frame was left as it is.
Mistress's `ControllerBindingsDiagramTests`: 21 cases, new ones for Z and the left trigger, the General tab's pad, and the
look at both sizes; the click and pad-reach cases now run on all four consoles. `SheetLookTests`' desktop case now holds
that the bindings are framed in the look.

**The broad run**, on 16b8c444 with WiseMan a8abe77d and LunaP 8480521 merged: one run of the Mistress filter without
`ShaderSettingsWindowTests`, the shader browser, the benches and any GPU, Vulkan or slang test, under `nice -n 10`:
**1,504 tests, 1,454 passed, 50 skipped (the picture, survey and live tools), none failed, in 5 min 3 s.** LunaP's whole
suite: 1,514, all passed.

### 42.14 Mutants of the second stage

Fifteen more, run by §42.6's runner (`probe/bindings/mutate.py`, its C and R series), the predictions written first
(§42.15). **C1–C8, Mistress's, all caught on the first run:** the L2 label ignoring its trigger; a trigger pulled while
capturing ignored; the pad tester never updated; the deadzone ring not following the slider; the window not framed in
the look; the look's help bar not the window's; the look's columns kept at the desktop's widths; Close kept in the look.
**R1–R7, LunaP's: four caught on the first run and three survived** (LunaP §198.10 has each): R1, a line's way out
ignoring the drawing it crosses, was caught only by a label-size test, by accident; R2, a way out through another
button, and R3, a band's room ignored, were not caught. After tests comparing each line with the straight ways out,
checking that no line crosses another button, and counting each band's labels, all three were caught. The first of
those tests also found a real fault before it was run against a mutant: the modern pad's cross had no way out that
crossed no other button, and its up arm's line crossed the left bumper; the bumpers were shortened.

**Result: 27 of 30 caught on the first run over both stages' new mutants (C and R), 30 of 30 after.** Mutants of the
first stage whose code this stage rewrote (L2, L3, L8, L9 and L16 no longer match their snippets; B12 matches twice) were
not rerun.

### 42.15 Predictions

| # | Prediction | Outcome |
|---|---|---|
| P285 | Of 15 mutants (C1–C8, R1–R7), at least 13 are caught on their first run | **Wrong**: 12 |
| P286 | R3, a band's room ignored, is caught by the label overlap and edge checks on the modern pad | **Wrong**: it survived; the row shrink of LunaP §198.2 made an over-full row fit |
| P287 | R6, the stick ring never drawn, is caught only by the pixel hash in the host test | **Held** |

### 42.16 Open questions

- **Q186, a scrolling area's last row.** A scroll viewer shows part of a row at its lower edge. Options: count it as cut
  and snap scrolling to whole rows, as ES-DE's menus do; count it as scrolling. **Recommendation:** the latter for
  windows that are not menus, decided once with the window-look audit rather than here.
- **Q187, the look's switch focus box.** It crosses the foot of a switch's words at both sizes. **Recommendation:** fix it
  in LunaP §196's style, with the window-look work, which owns it.

## 43. Every window in ES-DE's look, second part: the fit audit, the other windows, and the bindings window's faults in the style layer (2026-09-27)

*Built on branch `bigpicture-window-look` after §41's checkpoint was merged (WiseMan `4e45c49d`, LunaP `c20856f`).
WiseMan was merged in again at `53201508` and `6b99ec3b`. LunaP's `window-look` merged `openemu-library` at `661148e`
and `8991a6b`.* Three decisions of 2026-09-27 frame this part:

- §41.10's recommendations were accepted;
- the rule: nothing in any window may be cut off, run past its panel or overlap other content. *"We cannot have window
  features cutoff. If they dont fit, shrink them or rework them."*;
- a window is framed as a menu only once an audit passes it at both sizes and its pictures have been looked at.

The player's account is the settings reference §4.83, and LunaP's is its §196.9 and §196.10.

**Numbering.** Predictions from P288 and questions from Q188. P287 and Q187 were the highest in every tree on this
machine when this section was written. Q186 and Q187 were §42.16's, and are answered here.

### 43.1 The audit, and its proof

The audit (`FitAudit`, §4.83.2) was built before any window was reworked. It was then made to fail on purpose on the
three windows whose trial framings had been seen broken: a theme's detail, the theme browser and Shaders. Its output
is kept as the proof, in `~/.cache/emusen/probe/window-look/audit/trial-*.txt`:

- a theme's detail: 39 faults (37 overlaps, 2 past the panel);
- the theme browser: 18;
- Shaders: 7.

Every fault the pictures had shown is in those lists (§4.83.3 has them by kind). Each rule also has a self-test that
makes its fault on purpose (`FitAuditTests`, eleven cases), and a mutant that removes the rule (§43.6).

Trial renders were kept out of the pictures' folder. They went to `~/.cache/emusen/probe/window-look/trials/` and were
deleted at the end. `png/window-look/` holds only `before/` and the audited `after/`.

### 43.2 The windows

The twelve windows §41.9 left were framed, together with the scraping status's running and finished states and Find by
Name's results. §4.83.1 has what changed in each. Three had to be laid out again, not merely restyled:

- a theme's detail: the screenshot at the left, and the facts in their own column;
- the theme browser: two columns, with the screenshot fitted into its own;
- Shaders: the pack's words in the footer, the preset's buttons beside its name, and Close in the tab row.

The accepted questions:

- **Q170** is built. The database's introduction is in the footer before the attribution, four lines, and the
  footer is never cut: `MenuPanel.IsFooterCut` says whether it is, and the audit reports it.
- **Q173** stands as it was: the file dialogs stay hidden.
- **Q174** is built: one glyph for the two shoulders (LunaP §196.8). In a help bar the glyph is two squares wide, each
  shoulder at full size (§196.10).
- **Q175** was sought and not found. A list at the foot of a plain sheet at scale 1.5 built every row (LunaP §196.9),
  as §42.10 also measured at four heights.
- **Q177** is built: no empty picture slot.
- **Q178** is built, per list, by `SheetLook.SelectOnFocus`, in the database's lists, the cheats' table and the recent
  games. A first version in the pad router, which selected any focused row in any list, changed the shader shown during
  a pad walk of Shaders. `PadSettingsWindowTests` failed on it, and it was taken out.
- Q171, Q172 and Q176 carried no change.

**Lines cut at an edge.** A recent game's line and a cheat's cell end in an ellipsis and are whole in the footer while
their row is chosen. The audit accepts these two by name (§4.83.2's allowances), and nothing else.

Faults found on the way, each by the audit or a picture before its fix, are in LunaP §196.9 where the toolkit changed:

- a long title cut;
- a path in the footer upper-cased;
- a button's control content shown by its type's name;
- an open dropdown scrolling the Graphics page by 8 and then 372 pixels;
- a scroll bar's arrows stretched to a button's height;
- a slider over its Reset button.

Two were in Mistress:

- turning a tab moved the page by 32 pixels, since the router's first control on the new page was one cut at its edge.
  It now prefers a control wholly in view;
- at 1920×1200 the audit reported an edge at 1806 against a limit of 1804, which was layout rounding at the menu's
  scale 1.5. The slack is now a pixel and a half there.

### 43.3 The controller bindings, framed by their own overhaul (§42)

§42.16 left two faults to the style layer, and a third was seen in its pictures:

- **Q186, decided: fade.** A scrolling area fades the last 36 units of an edge while more lies beyond it. The fade
  stops where the focused row begins. The alternatives, and why they were not taken, are in LunaP §196.10. The audit
  treats every scrolling area the same way: a control sliced by an edge that does not fade is a fault.

  The first version kept the focused row clear of the fade by making every request to bring a control into view 36
  units taller. It failed two of `PadSettingsWindowTests`' cases, which pass without it:
  - a 944-parameter preset's first slider could no longer be reached;
  - a typed parameter search was found empty after walking back to its preset.

  Switching the two halves on and off separately placed the cause in the widening, not in the fade: with the widening
  off and the fade on, both passed, and with the fade off and the widening on, both failed. Every move of the focus near
  an edge scrolled the area, and so changed where the next press went. An attempt to make `PadAudit`'s walk start its
  replay from the explored state instead, which kept the widening, failed the search case even with the widening off. It
  was reverted. The fade now ends at the focused row, and the focus behaves exactly as before.
- **Q187, decided: the bar.** A focused switch or check box has no outline. The near-black bar behind it shows the
  focus, as it does for a row and a push button.
- **Q174, again.** The window's help bar named the shoulders as two entries, *LB* with no words and *RB CONSOLE*. It now
  has one *Console* entry with the pair's glyph.

**The Nintendo 64's small words.** The audit ran over the window at both sizes, every tab. Measured at 1280×800 in the
framed window, the labels' key and pad lines were:

- 11.6 pixels on the Nintendo 64, and 14.0 on the NES, Game Boy and Super NES, beside 20-pixel text everywhere else;
- at 1920×1200, 17.8 and 21.4.

The pictures agreed: *Right stick* was about 7 pixels high in its capitals. It was too small to read from arm's length on
a handheld, and grew. The labels could not grow within the drawing, since the drawing shrinks them until its tallest
column fits, and the Nintendo 64's right column holds five. So LunaP gained compact labels, which the look turns on
(§196.10). They measured 16.0 and 17.1 at 1280×800, and 24.6 and 25.6 at 1920×1200.

The audit gained the rule that holds them there: words under 16 design pixels fail. Sixteen is four fifths of the
look's small text (20). It was chosen before the
final label spacing, and the spacing was then tightened to meet it (§43.5, P288).

### 43.4 Tests

- **`FitAuditTests`**, eleven cases, one for each rule made to fail on purpose (§4.83.4).
- **`WindowFitAuditTests`**: 38 cases, 19 window states at two sizes, and the case that the framed windows are exactly
  the audited ones. **`WindowFitScrapeAuditTests`**: 6 cases, three states at two sizes. All pass, with no fault.
- **`SheetLookTests`**: the database's footer. The Q178 case enters the games list from outside it, so its first row is
  reached; that is where the defect was. Its first version reached the second row, which the list's own movement
  selects, and so passed without the change (§43.6, R8). The scraping status's case also holds that the picture slot
  shows only with a picture (Q177).
- **`ControllerBindingsDiagramTests`**: the help bar's four entries.
- **LunaP**: the fade along both axes, the switch's focus under a host outline, the pair in a help bar, and the compact
  labels (LunaP §196.10). Its suite is 1,521 tests, all passing.

**The narrow run** is recorded in §43.8.

### 43.5 Predictions

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P288 | *(worked out before the first compact build)* Compact labels with a 30-unit leader gap bring the Nintendo 64's lines to about 16.0 pixels at 1280×800, from the height the tallest column needs | 15.7 | **wrong** by 0.3; the gap went to 26 and the lines measured 16.0 |
| P289 | *(an assumption at the time, not written down before the run)* Widening a request to bring a control into view changes nothing a pad test sees | two pad cases failed (§43.3) | **wrong**; the design changed |

### 43.6 Mutants

Thirty-seven, one at a time, by `~/.cache/emusen/probe/window-look/mutate_fit.py`. It uses §41.7's protocol: a state
file before each mutant, any leftover restored at the start, and every restored file touched. Each mutant names the
tests that should catch it. No prediction of the count was written before the run.

**Thirty-two were caught on their first run, and five survived. All thirty-seven were caught after the tests below were
strengthened.**

| # | Rule broken | First run | After |
|---|---|---|---|
| F1–F10 | each rule of the audit taken out: past its panel; words past their button; sideways scrolling counted as reachable; overlap; too little room; a sliced foot with no fade; an ellipsis; small words; a cut title; a cut footer | all caught, each by its own self-test | — |
| F11 | a strip sliced at its side ignored | caught | — |
| R1–R3 | Theme Detail, Theme Browser, Shaders not reworked | caught by the window audit | — |
| R4 | the scraping status at a menu's width | **survived** | caught by the scraping status's frame case. The runner had named only the window audit, under which the narrower panel still fits |
| R5 | the cheats' table scrolls sideways | caught | — |
| R6 | the database's introduction not in the footer (Q170) | caught | — |
| R7 | an empty picture slot kept (Q177) | **survived** | caught, once the scraping status's case held the slot's visibility to its picture |
| R8 | a game row reached without being chosen (Q178) | **survived** | caught, once the case entered the list from outside (§43.4). The defect was then reproduced: with the change removed, the picture tool's *cheat-database-game-focused* showed the row focused and *Load* disabled |
| R9 | the reel's preview at its desktop height | caught | — |
| R10 | the bindings' help bar naming LB and RB apart (Q174) | caught | — |
| R11, R12 | a window framed with no audit case; the bindings left unframed | caught | — |
| R13 | the reel's strip not faded at its sides | caught by the window audit | — |
| L13–L16 | no edge fades; the fade over the focused row; a text box's scroller fades; a focused switch with no bar | caught | — |
| L17 | a switch keeps the stock outline (Q187) | **survived** | caught, once the case put a host style with an outline above the look (LunaP §196.10) |
| L18, L19 | the pair in one square however wide; the help bar giving the pair one square | caught | — |
| L20–L22, L24 | the look not setting labels close; plain spacing; plain line pitch; the plain size reported | caught | — |
| L23 | compact labels keep the plain height cap | **survived** | caught by a Super NES case bound by the cap |
| L25 | a sideways strip never fades | caught | — |

R13, F11 and L25 were written after the first thirty-four ran, when the pictures showed the reel's sliced tile (§43.7).
They were run once each and all were caught.

### 43.7 Pictures

In `~/.cache/emusen/bigpicture/png/window-look/`:

- `before/` is §41.8's, from the unmodified tree, and was kept;
- `after/` was regenerated from the final build, 64 pictures, and holds only windows that pass the audit.

`after/` covers every framed window at 1280×800 and 1920×1200, the themed ones over the synthetic theme and over Art Book
Next. It adds `controller-bindings-*`, `controller-bindings-general-*`, `controller-bindings-switch-focused-*` and
`controller-bindings-n64-*`.

Every picture was looked at. Three things were seen:

- **The rewind reel's first tile was sliced** at the strip's left edge, *".87 s ago"*. The audit's allowance had
  excused the strip, since it scrolls sideways. That was a cut the rule forbids, so the fade was extended to areas that
  scroll only sideways (LunaP §196.10), and the audit now checks the sides of such an area whatever its allowances.
  The audit, with the sideways fade switched off, reported the reel at both sizes; with it on, nothing.
- **The main window's status line**, under the sheet, shows the performance counters while a game runs unpaused, and is
  cut at the window's right edge. It is not a framed window and is outside this rule's scope; Q191 asks. *Answered
  in §43.11.*
- **The Cheat Database's lists at 1280×800** show three systems and two games, with room to spare now that the
  introduction is in the footer (Q170).

Trial renders went to `~/.cache/emusen/probe/window-look/trials/` and were deleted at the end.

### 43.8 The narrow run and the broad run

**The narrow run**, the blast radius, covered the pad, sheets, cheats, scraping, ES-DE, themes, covers, rewind, resume,
the look, the audit, Shaders, Graphics, folders, filters, input, controllers, bindings and drawings. Benches, GPU and
Vulkan cases were left out. Every case ran under `nice -n 10`. Its first pass after the fade failed two
`PadSettingsWindowTests` cases, which was §43.3's widening. After the redesign it was **1,239 tests: 1,201 passed, 38
skipped, none failed, in 4 min 24 s.**

**The broad run** was one run of the Mistress filter and the audit's self-tests, without `ShaderSettingsWindowTests`,
the shader browser, the benches or any GPU, Vulkan or slang case, under `nice -n 10`. It ran on the final build with
WiseMan `6b99ec3b` and LunaP `8991a6b` merged: **1,569 tests, 1,519 passed, 50 skipped (the picture, survey and live
tools), none failed, in 5 min 31 s.** LunaP's whole suite: 1,521, all passed.

### 43.9 Not done

- **States no case opens** (§4.83.5): long theme names, a hundred cheats, long parameter names. *Long names were added
  on 2026-09-27 (§43.11); a hundred cheats is still not opened.*
- **The Nintendo 64's margin.** The labels meet the floor with none (Q188). *Built on 2026-09-27 (§43.11).*
- **A desktop ES-DE look**, deliberately (§10.1).
- **A real pad, keyboard or handheld.** Nothing here ran on hardware, and nothing from this part goes to the handheld
  until it has been looked at.

### 43.10 Open questions

- **Q188, the Nintendo 64 drawing's margin.** Its labels are at 16.0 pixels at 1280×800, the audit's floor exactly. A
  longer binding name would fail the audit. **Recommendation:** when a drawing is bound by height, let a side column
  with more than four labels take a second column in the width the drawing leaves free. At 1280×800 about 300 pixels
  are free. **Built** (§43.11): 17.1 at both sizes.
- **Q189, the floor's measure.** The audit measures words by font size, so a condensed face counts the same as a wide
  one. Barlow Condensed at 16 pixels has a smaller x-height than the desktop's face at 16. Keep font size, or measure
  the drawn capitals' height? **Open**, waiting on a decision; the audit still measures font size (§43.11). **Decided
  and built** (§43.12): the drawn capitals.
- **Q190, the audit's reach.** It audits the states the tests open. **Recommendation:** add long-name cases (a theme,
  a cheat, a parameter, a game) to `WindowFitAuditTests`, since those are where a cut would first appear. **Built**
  (§43.11), with a long binding name as well; five of the new states failed and were fixed.
- **Q191, the main window's status line.** While a game runs unpaused, the status line under a sheet shows the
  performance counters and is cut at the window's right edge. It is not a framed window. **Recommendation:** in a
  big-screen session hide the counters there, or end the line in an ellipsis. Leave it to the HUD's own work. **Built**
  (§43.11): both, and the line is audited.

### 43.11 The follow-ups: Q188, Q190 and Q191 built, Q189 open (2026-09-27)

*Built on branch `fit-followups` from WiseMan `dd28948b`, with LunaP's `fit-followups` from `openemu-library` at
`2af6a77`.* The player's account is the settings reference §4.83.6; LunaP's is its §196.11 and §198.11. Predictions are
numbered from P290 and questions from Q192; P289 and Q191 were the highest in every tree on this machine when this was
written.

**Q189 is open.** It waits on a decision, and nothing here changes how the audit measures words: by font size. *Built on 2026-09-28 (§43.12).*

#### The status line (Q191)

The defect was measured before it was fixed. The new case, the status line under a game with a 102-character title at
both sizes, in a big-screen session and on the desktop, run on the unmodified window, reported the counters' line 2,268
pixels wide at 1280×800 and 2,276 at 1920×1200, past the bar's edge in all four cases, and in the big-screen cases also
too small to read (14 pixels, 9.3 design pixels at 1920×1200). The messages beside it got no width at all. The first
picture of each is in `png/fit-followups/before/`.

The change, in the order it was decided:

- In a big-screen session the counters are hidden (`ApplyStatusBar`), and the messages take the look's small text at
  the menus' scale and wrap, since a pad has no tooltip.
- On the desktop the counters are a short line, at most half the bar, with the whole line as the tooltip; the messages
  end in an ellipsis with their whole text as the tooltip. The audit accepts a trimmed status word only while its
  tooltip holds it (`WholeInItsTip`), and checks it against the window's own edge besides, which no allowance excuses.
- `FitAudit.Check` gained the smallest size as a parameter. The desktop's line is held to every rule except the big
  screen's floor, since the desktop's words are the desktop's size (§4.83.2).

**The bar's height, and a harness fault it exposed.** The first version let the big screen's 20-pixel words take their
natural line, which made the bar 7 pixels taller at 1280×800. The narrow run then failed two cases of
`PadSettingsWindowTests`, `The_shaders_window_from_the_pad_menu_adjusts_a_built_in_filter_and_resets_it_all` and
`A_long_preset_s_sliders_on_the_sheet_are_reached_and_walked_by_pad`, both at the walk to a slider: *No pad path to the
control asked for.* They are the two §43.3 met. The cause was isolated in three steps:

1. With the status line's size put back to the desktop's, and nothing else changed, both passed.
2. With the larger line, a picture at the failing step showed the slider in view (the 944-parameter preset's first
   slider, whole, under its heading).
3. From the first slider of *CRT (Lottes)*, pressing Down reached every slider in turn, *Mask dark* on the eighth
   press. The window lets the pad reach the control; `PadAudit`'s walk, which replays each path from the start with
   the scroll offsets put back, did not find it under a sheet 7 pixels shorter.

The fault is in the harness, not the window. It was not fixed here: the bar now keeps the height it had at 800 pixels
(a line of 26 design pixels, no padding), which leaves the sheets' geometry at 1280×800 exactly as it was, and both
cases pass. Q192 asks for the walk to be made to replay the same way at any height.

#### The Nintendo 64's margin (Q188)

Built as recommended, in LunaP's `ControllerDiagram` (§198.11 there): a side column of more than four labels takes a
second, staggered column when its labels were shrunk by its height, the drawing is bound by its height, and the width
the second column takes was free. The outer labels' lines run level between the inner labels and turn toward their
buttons only beside the drawing, so the rules of §198.2 hold.

The audit's bindings case now writes each drawing's label size and holds it to a margin of one design pixel over the
floor with the bindings a console starts with, and to the floor with long names:

| Drawing | 1280×800 | 1920×1200 | Over the floor | Long key names on A and Start |
|---|---|---|---|---|
| Nintendo 64 | 17.1, right column split | 17.1, right column split | +1.1 | 16.6 (+0.6) |
| Super NES, NES, Game Boy | 17.1 | 17.1 | +1.1 | 17.1 |

17.1 is where every drawing stops, by the caps of §198.2 (the text around it times 0.85, and its width over 900), not by
its space. Raising the caps for compact labels was measured before deciding (0.95 and the width over 820): the Game Boy
and NES went to 18.2, the Super NES to 17.7 and the Nintendo 64 to 17.2, where its top row of six labels binds. It
would change every drawing to move the Nintendo 64 by a tenth of a pixel, and was not made. Q193 asks whether a larger
margin is wanted, since that is what it would cost.

**Found on the way.** LunaP's new test that no line passes through a label other than its own failed on the unmodified
layout: a row's lines ran down through a column's first label, which could start above the drawing's top. It is visible
in §43.7's `controller-bindings-n64-1280x800.png`, where the R line runs under C Up's label. Columns now stand beside
the drawing, between its top and its foot, when they fit there (LunaP §198.11).

#### The audit's reach (Q190)

Ten window states were added at both sizes (§4.83.6 lists them), and the status line. Run on the unmodified windows,
five failed (the three long titles, the long cheat and the theme browser); each was fixed and passes now:

| State | Before | Fix |
|---|---|---|
| A screenshot with a 117-character title; a theme's detail and About with a 77-character name | *title cut*, at both sizes | a framed sheet's title wraps onto a second line (LunaP §196.11) |
| Active Cheats with a long cheat chosen | the code's column took the code's whole width and pushed the table sideways: *cut at a scrolling edge with no fade: the right*, at both sizes | the code's column is at most 300 design pixels and ends in an ellipsis |
| the same, once that was fixed | the description wrapped to three lines, and the table, 78 design pixels high under an empty status line, scrolled in less than two rows: *scrolls in too little room*. The case was then given a third cheat, so the table scrolls whatever its rows | a table's cell is one line (LunaP §196.11); the status is said in the footer and its line is gone, giving the table about 110 |
| the theme browser with the long theme reached by the pad | the preview stayed on the first theme: the pad's row was not chosen (a Q178 defect, not a cut) | the list chooses the row the pad reaches |

The Resume, Find by Name, cover picker and shader cases passed on the unmodified windows. A case that opened the rewind
reel with a long title was written and dropped, since the reel does not show the title.

#### Tests

- **`WindowFitAuditTests`**: 63 cases, all passing: 29 window states at two sizes, the framed-windows case, and four
  status-line cases. **`WindowFitScrapeAuditTests`**: 6, as before.
- **LunaP**: `ControllerDiagramTests` 72 (10 new), `MenuLookTests` two new (the wrapped title, and a table's cell on one
  line).
- The bindings case asserts the margin, so a Nintendo 64 back at 16.0 fails it (D1M below).

#### Predictions and mutants

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P290 | S1–S7, the status line's seven rules, are all caught by the status-line case; S3 only by its narrowest-window step | all seven caught; S3 by the two desktop cases, where only the narrowest step can fail it (the short counters line is 586 pixels wide) | **right** |
| P291 | D4, two columns keeping one column's spacing, survives: the line still clears the inner labels by about 3.5 pixels | survived | **right**; the split case then gained a clearance assertion, and D4 is caught |
| P292 | T3, a wrapped title kept to the one-line minimum, survives, since the long titles fit two lines at that size | caught by the 117-character screenshot title at both sizes, which needs the smaller size | **wrong** |
| P293 | C2, a table's cell wrapping again, is caught by the long cheat's case | survived: once the status moved to the footer the table has room for a wrapped row, and a wrapped cell is whole, which the audit does not count as a fault | **wrong**; LunaP's own case was added (C2L), and catches it |
| P294 | Of 21 mutants, 19 are caught on the first run and 2 survive (D4, T3) | 18 caught, 3 survived (C2, D3, D4) | **wrong** |

The predictions were written to `~/.cache/emusen/probe/fit-followups/predictions.txt` before the first run.

Twenty-one mutants, one at a time, by `~/.cache/emusen/probe/fit-followups/mutate_followups.py`, with §41.7's protocol: a
state file before each mutant, a leftover restored and rebuilt at the start, every restored file touched, and both trees
rebuilt at the end. Each names the tests that should catch it. A twenty-second, C2L, was added after the first run.

**Eighteen were caught on the first run and three survived.** Four of the eighteen verdicts (C1, C3, T1, T3) also
counted a failure that happened without any mutant: after the bar's height was changed, the long cheat's case failed at
1920×1200 on its own (below), and the run did not know it. Those four were run again once it was fixed, and each was
caught by the cases meant to catch it.

| # | Rule broken | First run | After |
|---|---|---|---|
| S1 | a big screen shows the counters in its status line | caught | — |
| S2 | the desktop's line is not trimmed | caught | — |
| S3 | the counters are not held to their share | caught | — |
| S4 | the counters' tooltip is the short line | caught | — |
| S5 | the line's tooltip does not follow its text | caught | — |
| S6 | a big screen's line does not wrap | caught | — |
| S7 | a big screen's line at the desktop's size | caught | — |
| C1 | the cheats' code column grows without bound | caught | caught on the rerun |
| C2 | a table's cell wraps in the look, against the window audit | **survived** | survives: a wrapped cell is whole and the table then has room, so it is not a fit fault; the one-line rule is the table's own, held by C2L |
| C2L | the same, against LunaP's case for it | — | caught |
| C3 | the cheats' status above the buttons, not in the footer | caught | caught on the rerun |
| C4 | the theme browser's preview does not follow the pad | caught | — |
| T1 | a framed sheet's title keeps to one line | caught | caught on the rerun, by the three long titles at both sizes |
| T2 | a wrapped title's band does not grow | caught | — |
| T3 | a wrapped title may not shrink below the one-line minimum | caught | caught on the rerun, by the screenshot's title |
| D1 | a tall column never takes a second | caught | — |
| D1M | the same, against the window audit's margin | caught | — |
| D2 | an outer label's line runs straight across the inner column | caught | — |
| D3 | a second column kept where it takes the drawing's width | **survived** | survives; see below |
| D4 | two columns keep one column's spacing | **survived** | caught, once the split case asserted the clearance |
| D5 | a column starts above the drawing, where a row's lines run | caught | — |
| D6 | the split measured against one column's height | caught | — |

**D3 survives, and is recorded rather than excused.** It removes the check that the drawing is still bound by its height
after a column is split. The Nintendo 64 was laid out in the look at 1,846 sizes (400 to 1100 wide, 200 to 450 high, in
steps of 10) with short bindings, and at 40 more (700 to 1400 wide, 250 to 450 high) with long ones, with and without
the check: the layouts were the same at every one. Wherever splitting let the labels grow, the width it took was free.
The check stays, since it states when a split is allowed, and the fine grid is kept beside the runner
(`d3-grid-*.txt`). No layout measured reaches it.

**The failure without a mutant, and a unit in the audit.** After the bar kept its 800-pixel height, the long cheat's
case failed at 1920×1200 with the third row *past its ScrollContentPresenter*, 3 pixels below the table's foot. The
table's rows ran 2 design pixels past its view, so the list scrolled by 2 and the row could be brought into view. The
audit decided whether a list scrolls by comparing that overflow in the list's own design units against its slack in
screen pixels: 2 against 2, so it counted the list as not scrolling and the row as cut. It now converts the overflow to
screen pixels first (3 against 2), for this rule and for *scrolls in too little room*, and the case passes. The eleven
`FitAuditTests` pass, and the whole audit reports nothing new.

#### Pictures

In `~/.cache/emusen/bigpicture/png/fit-followups/`:

- `before/`, rendered from the unmodified trees (WiseMan `dd28948b`, LunaP `2af6a77`) with the new cases added, as the
  audit saw each new state at 1280×800 and 1920×1200;
- `after/`, from the final build, the same states, every one passing the audit.

`before/` has 40 pictures and `after/` 50. Two states have no *before* picture, because their case failed before
the audit ran: the theme browser with the long theme (the preview never reached it) and the long cheat with its status
(the row's state, just before it, failed first; its picture is there). `after/` adds the status line's longest message
and, on the desktop, the window at its narrowest.

Every picture was looked at, side by side at both sizes (contact sheets in `~/.cache/emusen/probe/fit-followups/sheets/`).
What they show agrees with the audit, and three things are worth saying:

- **The status line before**, in a big-screen session at 1920×1200, is 14-pixel words across the whole screen, running
  off its right edge; after, *Paused* or *Running: …* in 30-pixel words, and a failure wrapped to three lines.
- **The Nintendo 64 before and after**: five labels in one column, then three and two, the two outer ones' lines passing
  level between the inner three. With the long names the top row spans the panel and the labels are a little smaller,
  as the table says.
- **The desktop's status line at 1920×1200** is the desktop's own size, as the desktop's words are everywhere; nothing
  in it is cut.

Trial renders went to `~/.cache/emusen/probe/window-look/trials/` and were deleted at the end.

#### The narrow run and the broad run

**The narrow run**, the blast radius, covered Active Cheats, the sheets' look, the theme browser, the library screen,
the main window, the bindings, input settings, the Cheat Database, the big-picture switch, screenshots, resume, the
pad's settings sheets, the themed library's pad, rewind and the audit's self-tests, without benches, GPU or Vulkan
cases, under `nice -n 10`. Its first pass, with the bar's natural height, was **371 tests: 367 passed, 2 skipped, 2
failed**, the two `PadSettingsWindowTests` cases above. With the bar at its 800-pixel height they and the status-line
cases passed (26 tests), and the whole audit with its self-tests passed after the unit fix (80 tests).

**The broad run** was one run of the Mistress filter and the audit's self-tests, without `ShaderSettingsWindowTests`,
the shader browser, the benches or any GPU, Vulkan or slang case, under `nice -n 10`, on the final build with WiseMan
`dd28948b` and LunaP `openemu-library` `2af6a77` merged in (both already contained): **1,593 tests, 1,543 passed, 50
skipped (the picture, survey and live tools), none failed, in 5 min 57 s.** LunaP's whole suite: 1,533 tests, all
passing once the README's count was brought up to date.

#### Not done, and open questions

- **Q192, the pad walk's replay.** `PadAudit`'s breadth-first walk failed to find a slider the pad reaches in eight
  presses, under a sheet 7 pixels shorter. **Recommendation:** make its replay independent of the sheet's height, for
  example by recording the scroll offset each step leaves and checking it on replay, and run the two cases at several
  heights. **Built** (§43.12): the walk replays from its start, and two faults the heights found in the window are
  fixed.
- **Q193, a larger margin.** Every drawing stops at 17.1, one design pixel over the floor, by its caps. **Recommendation:**
  keep them unless the pictures on a handheld say the labels are small; the measured cost of raising them is above.
- A cheat list of a hundred rows, a title too long for two lines, and a big-screen message long enough to take a
  quarter of the screen are not opened.
- Nothing ran on hardware.

### 43.12 Q189 and Q192 built: words measured by their capitals, and the pad walk at any height (2026-09-28)

*Built on branch `audit-followups` from WiseMan `7b7bda3b`, with LunaP's `audit-followups` from `openemu-library` at
`7ccc205`.* The player's account is the settings reference §4.83.7; LunaP's is its §97.9. Predictions are numbered
from P295 and questions from Q194; P294 and Q193 were the highest in every tree on this machine when this was written.

#### Q189: the floor measures drawn capitals

`FitAudit` measures how tall a face draws its capitals, from the outline of an *H* drawn at a thousand pixels, since
this Avalonia's glyph metrics report no ink height (every glyph measured came back 0 high). Words are measured in the
face they were drawn in, after any fallback, and a controller drawing's labels in the drawing's own face.

**The calibration.** The floor stays defined by the desktop's face at 16 design pixels; Inter's capitals are 0.7273 of
its size, so the floor is 11.64 design pixels of capitals, and the slack of 0.05 of a pixel becomes 0.036 of capitals.
Inter at 16 passes and at 15.9 fails, as it did by size. The measured shares, beside the fonts' own `OS/2` cap heights
where those were read:

| Face | Measured | `OS/2` | Size at the floor |
|---|---|---|---|
| Inter | 0.72729 | not read | 16.00 |
| Barlow Condensed | 0.69995 | 0.700 | 16.63 (16.57 with the slack) |
| Noto Sans, the headless tests' default | 0.71411 | 0.714 | 16.30 |

**What failed: nothing.** Every state the audit opens passed at both sizes (the settings reference has each window's
shortest capitals). No window needed changing, and no size grew. The nearest is the Nintendo 64 drawing with long key
names, 16.62 pixels, 11.63 of capitals: a hundredth under the exact floor, inside the slack. By size it had kept 0.6
over. Mutant A5, the floor without its slack, fails it at both sizes, which is the measurement's proof that it sits
there. Q195 asks.

#### Q192: the pad walk, and what it found in the window

**The failure, reproduced.** The status bar is made taller in the tests (`TallerBar`), so the sheet above it is
shorter by as many pixels. The first big-screen status line was rebuilt as §43.11 describes it, its words at their
natural line and the bar's old padding: at 1280 × 800 it made the bar 8 pixels taller, not 7. The shaders case failed at
that height, and both cases at +7, with §43.11's *No pad path to the control asked for*.

**Straight walks, to tell the harness from the window.** A walk that presses Down one press at a time from the first
slider of *CRT (Lottes)*, with no search, was run beside the audited walk at every height from 0 to 16 pixels shorter.
It separated the two: where the straight walk passed and the audited one failed, the harness was at fault; where both
failed, the window was. Four causes, in the order found:

1. **The found path pressed from the wrong state (harness).** `TryReach` pressed the path the walk found without
   putting back the scrolling and lists that every replay inside the walk puts back, so it started from wherever the
   last replay had left the sheet. The walk now ends with that replay itself. This is §43.11's failure: at +8 the
   straight walk reached *Mask dark* on the eighth press from the first slider, and the audited walk, after this fix,
   did too.
2. **A preset's rows not there yet (harness).** A replay that passed over other presets left the parameter list empty
   for the shader list's 120 ms settle while the walk refocused its starting slider. The walk now waits for its start's
   rows before putting the scroll back, and a start it cannot focus fails the replay.
3. **`ScrollIntoView` on a built row (harness).** Refocusing a slider always asked the list to bring it into view. With
   that, the list was later found with no row built and its extent its own height, and the shaders case failed at
   every height from 10 to 16. The call is now made only for a row that is not built.
4. **Rows below the view not built (window).** The straight walk itself stopped at every height from 0 to 7, before the
   fifth slider or the eighth. Avalonia's `VirtualizingStackPanel` rebuilds its half-view buffer only when the view
   leaves the range last built for, and a view brought flush under the focused row can end exactly on that range's edge
   (read in its source; the rows built bear it out, as LunaP §97.9 records). The pad router now, when it finds nothing
   inside a scrolling area that can scroll further that way, scrolls the area a page and back and searches again. At +3
   the list could also be left arranged at its own height, not scrolling at all; LunaP's `SliderList` arranges it again.

Two things were ruled out on the way: the XY-focus manifolds Avalonia keeps between moves were read at every replay
and were always reset, and a wait for every shader panel to settle at each replay, instead of for the start's rows
alone, broke the long preset's case on the Nintendo 64, NES and Game Boy tabs and was taken out.

**Each fix alone.** Switching one fix off at a time over the two cases at 18 heights (0 to 16 and the rebuilt status
line; 36 cases):

| Switched off | Failed |
|---|---|
| none | 0 |
| (1) the replay from the start | 14 |
| (2) the wait for the start's rows | 18 |
| (3) `ScrollIntoView` only when needed | 7 |
| (2b) a failed refocus failing the replay | 0 |
| (4) the page's nudge in the router | 16 |
| (4b) `SliderList`'s arranging again | 0; the straight walk at +3 fails without it |

**Before and after.** The cases kept: the two at seven heights (0, 3, 5, 7, 8, 10 and 13) and the straight walk at
seventeen. On the unmodified trees, **19 of 31 failed**; with every fix, all 31 pass, with the rest of
`PadSettingsWindowTests` (51 cases in all).

**Found on the way: a preset row focused but not chosen.** With the walk honest about its start, the parameter search
case reached the *huge* row sideways from the search box, and the list still showed the preset before it. The case
steps off the row and back. The list choosing every row that takes the focus (Q178's `SelectOnFocus`) was tried for
Shaders; it left the focus on nothing after *Use This Shader* and failed the two cases at every height, and was taken
out. Q194 asks.

#### Tests

- **`FitAuditTests`**: 14 cases, three new: the floor is Inter's measured capitals; a condensed face must reach them
  (Barlow Condensed fails at 16 and 16.5, passes at 16.6; Inter passes at 16, fails at 15.9); and a Nintendo 64
  drawing in Barlow Condensed, its labels between 16.1 and 16.45 pixels, fails, which it would pass measured in Inter.
  The third was added after mutant A4 survived.
- **`PadSettingsWindowTests`**: the shaders and long-preset cases at seven heights, and
  `Down_from_a_preset_s_first_slider_reaches_every_slider_at_any_sheet_height` at seventeen.
- **`WindowFitAuditTests`** writes each window's shortest capitals (`EMUSEN_WINDOW_FIT_CAPS=<file>`), and each drawing's
  capitals beside its size.

#### Predictions and mutants

The predictions were written to `~/.cache/emusen/probe/audit-followups/predictions.txt` before the first run.

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P295 | A1 (every face measured as the desktop's) is caught by the condensed-face case | caught by it and by the calibration case | **right** |
| P296 | A3 (a text block's face taken from its typeface, not its drawn run) survives: no audited words fall back | survived | **right** |
| P297 | A4 (a drawing's labels measured in the desktop face) survives: no case holds a drawing's labels between the two floors | survived | **right**; a case was added, and catches it |
| P298 | G4 (a failed refocus of the start not a failed replay) survives, as when switched off alone | survived | **right** |
| P299 | R3 (the page's nudge not undone) survives: the cases check where the focus goes, not how far the view moves | survived | **right**; the straight walk now holds the view to a row a press, and catches it |
| P300 | Of 15 mutants, 11 are caught on the first run and 4 survive (A3, A4, G4, R3) | 11 caught, the same 4 survived | **right** |

Fifteen mutants, one at a time, by `~/.cache/emusen/probe/audit-followups/mutate_audit.py`, with §41.7's protocol: a
state file before each mutant, a leftover restored and rebuilt at the start, every restored file touched, and the tree
rebuilt at the end. Each names the tests that should catch it. **Eleven were caught on the first run and four
survived; after two cases were strengthened, thirteen are caught and two survive.**

| # | Rule broken | First run | After |
|---|---|---|---|
| A1 | every face measured as the desktop's | caught (2) | — |
| A2 | the floor calibrated to the condensed face | caught (2) | — |
| A3 | a text block's face taken from its typeface, not its drawn run | **survived** | survives; see below |
| A4 | a drawing's labels measured in the desktop face | **survived** | caught by `A_drawing_s_condensed_labels_are_held_to_the_desktop_face_s_capitals` |
| A5 | the floor without its slack | caught (4), among them the Nintendo 64 with long names at both sizes | — |
| A6 | words measured by size again | caught (1) | — |
| G1 | the found path pressed from the state the last replay left (§43.11's harness) | caught (5) | — |
| G2 | the start's rows not waited for | caught (7) | — |
| G3 | a built row asked into view again | caught (2), the shaders case at 10 and 13 | — |
| G4 | a failed refocus of the start not a failed replay | **survived** | survives; see below |
| R1 | no row built past the view's edge | caught (15) | — |
| R2 | the router's nudge a pixel, not a page | caught (12) | — |
| R3 | the page's nudge not undone | **survived** | caught (8), once the straight walk held the view to a row a press |
| L1 | a list left arranged at its own height (LunaP) | caught (1), the straight walk at 3 pixels shorter | — |
| S1 | a preset focused sideways taken as chosen | caught (1), the search case | — |

**A3 and G4 survive, and are recorded rather than excused.** A3 needs words drawn in a face other than the one they
asked for; no audited window has any, and a case would depend on which fallback faces the machine has installed.
G4 guards a replay from pressing on after its start could not be focused; switched off alone over the 36 cases of the
toggle table it changed nothing, and it stays because pressing on from the wrong control can end on the right one by
chance and record a path that does not replay.

#### Pictures

In `~/.cache/emusen/bigpicture/png/audit-followups/`, as the audit saw each state at 1280 × 800 and 1920 × 1200:

- `before/`, with LunaP's `SliderList` as it was (the only change here that draws anything);
- `after/`, from the final build.

Only one window changed, Shaders, and only in its list's arranging, so the pictures are its three states: a built-in
filter, the 944-parameter preset with its sliders, and the preset with long parameter names; twelve in all. Q189 changed
no window. Every picture was looked at. Before and after are the same picture in every state; the only difference, in
four of the six pairs, is the test's temporary folder named in the footer, which is a new name each run. The list's
fault shows only under a sheet 3 pixels shorter, which is not an audited size; the straight walk is its evidence.

No trial renders were made.

#### The narrow run and the broad run

**The narrow run**, the blast radius, covered every user of `PadAudit` (the pad's settings sheets, cheats, rewind, the
themed library's sheets and switches, scraping, covers, the bindings and the shader browser), the audit with its
self-tests, and the router, under `nice -n 10` and without benches, GPU or Vulkan cases: **412 tests, 411 passed, 1
skipped, none failed, in 4 min 25 s.** The first narrow run, before Q194's step, failed one: the parameter search case,
as above. LunaP's whole suite: 1,533 tests, all passing.

**The broad run** was one run of the Mistress filter and the audit's self-tests, without `ShaderSettingsWindowTests`,
the shader browser, the benches or any GPU, Vulkan or slang case, under `nice -n 10`, on the final build. WiseMan
`7b7bda3b` and LunaP `openemu-library` `7ccc205` were merged in (both already contained, since neither had moved):
**1,625 tests, 1,575 passed, 50 skipped (the picture, survey and live tools), none failed, in 7 min 11 s.**

#### Open questions

- **Q194, a preset row focused sideways.** Reached sideways from the search box, a shader preset's row takes the focus
  and not the choice, so the sliders beside it are another preset's. **Recommendation:** have the shader list choose
  only a row the pad enters it on from outside, and look again at why choosing every focused row lost the focus after
  *Use This Shader*, before choosing every row as the other five lists do.
- **Q195, the Nintendo 64 with long names at the floor.** Its labels are 16.62 pixels, a hundredth of a pixel of
  capitals under the exact floor, bound by the width of its top row of six. **Recommendation:** leave it while the
  slack holds; if a longer binding is ever audited, let the top row split into two staggered rows as a tall column
  does (§43.11, Q188), not raise the cap Q193 keeps.

## 44. A favourite's toggle moves the game, not the highlight (2026-10-03)

*Built on branch `bp-favorite-cursor` from WiseMan `61998d92`.* In the themed view, marking a game as a favourite moved
the game to the top of the list, as it should with favourites sorted first, and moved the highlight there with it. To
favourite a run of games the tester had to scroll back down after each one. The decision of 2026-10-03 is that the game
moves and the highlight does not. **Numbering.** P301 and Q196 are new here; P300 and Q195 were the highest in every
tree on this machine when this section was written.

### 44.1 The cause (measured)

The selection is kept by the game's file, by system and folder (`ThemedLibrary._cursor`, §15.2), so that a rebuild from
new data finds the same game again. A toggle goes through `MainWindow.ToggleThemedFavourite`
(`Views/MainWindow.GameOptions.cs`). It writes `games.db` and calls `ShowLibraryEntries`, which calls
`ThemedLibrary.Show`. `Show` builds a new stage from `Data()`, and `Data()` looked the kept file up in the re-sorted
list (`BigPicture/ThemedLibrary.cs`, the `int game = _cursor.TryGetValue(...)` line of `Data()`, line 218 at
`61998d92`). With favourites first, the file is at row 0, so the highlight went to row 0. The same line gave two other
faults, both shown by the tests of §44.5 before the fix:

- unfavouriting the game at the top sent the highlight down to wherever the game was sorted to (row 7 in the test);
- in the favorites collection an unfavourited game is no longer listed. `FindIndex` gives −1, and the `Math.Max(0, …)`
  sent the highlight to row 0.

**The scroll needs nothing of its own.** The three primary elements all derive their scroll from the selected index
when they are built: `TextRowList.FirstVisible` centres the selection and clamps at the ends, the grid's `ScrollRow`
starts at `GridGeometry.ScrollFor(index)`, and a carousel's position starts at `Glide.At(index)`. A rebuild at the same
index therefore draws the same window of the list, and keeping the row is enough. The text list, the grid and the
carousel shared the cause, since all three are rebuilt through the same `Data()`. All three had the jump and all three
are fixed by the one change.

### 44.2 What ES-DE's documentation says (cited)

The following were read on 2026-10-03 for where ES-DE puts the cursor after a toggle:

- ES-DE's `USERGUIDE.md` at master (`gitlab.com/es-de/emulationstation-de/-/blob/master/USERGUIDE.md`):
  - "General navigation", the Y button;
  - "UI settings": *Sort favorite games above non-favorites*, *Add star markings to favorite games*, *Enable quick list
    scrolling overlay* and *Enable toggle favorites button*;
  - "Game collection settings": *Sort favorites on top for custom collections*;
  - "Jump to..";
  - "Metadata entries": *Favorite*;
  - "Automatic collections";
- `FAQ.md` and `CHANGELOG.md` at master, in the same repository.

None of them says where the cursor goes when a game is marked or unmarked. None distinguishes marking from unmarking,
and no setting is said to govern it. *Sort favorite games above non-favorites* decides only whether the game moves at
all. ES-DE's source was not read and ES-DE was not run.

The documentation is silent, so the rule below is the one the tester gives as ES-DE's behaviour: the highlight stays
where it was, and only the game moves. That is a claim about ES-DE that nothing here has measured. It is recorded as
**P301**: *in ES-DE 3.4.1, the release §22.2 measured, with favourites sorted first, Y on the game at row i leaves the cursor at row i, both for
marking and for unmarking.* P301 is retired by a measurement on ES-DE, the next time one is run for §22.2's kind of
question.

*Observed 2026-10-03, on the handheld:* the tester compared the fixed build (157dd7ef) with ES-DE and found the
behaviour identical. P301 is retired by that observation. It was a comparison in play, not a scripted measurement, and
which of the cases above it covered was not recorded; Q196's recommendation stands until the editor's case is compared.

### 44.3 The rule (argued)

- **The row is kept, not the game.** When the game at row *i* moves to the top, the highlight stays on row *i*, which
  now holds the game that was at *i − 1*. One press down reaches the game that came after the favourite, as it would
  have before the toggle. Unmarking works the same way: the highlight keeps its row, and the game is sorted back among
  the others.
- **Clamped.** If the list is shorter, as in the favorites collection or under a filter that keeps only favourites, the
  row is clamped to the new last row.
- **The details follow the highlight.** The game under the highlight drives the metadata, art and video, as it does
  after any move of the cursor, because the scene is built from the index.
- **Both ways in.** North (ES-DE's Y) and the game options menu's *Add to Favourites* / *Remove from Favourites* both go
  through `ToggleThemedFavourite`, so they follow the same rule.
- **A list that is gone.** Unmarking the last game of the favorites collection removes the collection from the
  carousel. Before this change the view fell through to the first system's gamelist: the NES list, at its top, which
  is the same kind of jump. It now returns to the system view at the place the collection held, clamped to the last
  system. The documentation says nothing here either. This is a choice, and like P301 it waits on a measurement.

**Why the rule is not general.** Every other rebuild still keeps the game: a search, a sort or filter change, quick
system select, a return from a game, and a save in the metadata editor (§15.2, §23). The argument rests on the action.
A toggle is done in place, often to a run of games in turn, so the player's place in the list is what matters. A
renamed or re-sorted game is the subject of the edit, and the player expects to see it afterwards; `A_sort_name_orders…`
holds the editor to that. With favourites not sorted first, no game moves, and the two rules give the same row.

**Q196, the editor's Favorite field.** The metadata editor's *Favorite* field changes the same flag, but it is saved
with every other field, so it keeps the game. Whether a save that changes only that field should keep the row instead
is left open. **Recommendation:** leave it as it is unless P301's measurement shows that ES-DE's editor keeps the row
too.

### 44.4 The mechanism

`ThemedLibrary.KeepPlace()` records the gamelist's view key and its current index. `ToggleThemedFavourite` calls it
before it writes the record. The next `Data()` for that same key uses the recorded index, clamped, in place of the file
lookup, and then forgets it. A `Show` that fails, or that returns early for a zero-sized screen, forgets it too, so a
later rebuild cannot pick up a stale index. `Remember()` then keeps the file of the game now on that row, so a later
rebuild, such as the one after `IdentifyLater`, stays on that game. The vanished-list case is a check in `Show`: if the
view is a gamelist and its system is no longer among the systems, the view becomes the system view at the old system
index, clamped.

### 44.5 Tests

`ThemedFavouriteCursorTests` drives the pad over a `ThemedSession` with thirty SNES games, enough that the text list
must scroll. A metadata `text` element shows the selected game's name, so the tests can check what the details show.

| Rule | Test |
|---|---|
| A favourite moves to the top; the highlight, the text list's selected row and its first visible row stay; the details show the game now there; one press down reaches the game after the favourite | `A_favourite_moves_to_the_top_while_the_highlight_and_the_scroll_stay_where_they_were` |
| Unmarking the game at the top leaves the highlight on row 0 | `Unfavouriting_the_game_at_the_top_keeps_the_highlight_at_the_top` |
| Marking the last game leaves the highlight on the last row, with the scroll unchanged | `Favouriting_the_last_game_keeps_the_highlight_on_the_last_row` |
| All games, an automatic collection that keeps the game | `In_all_games_a_favourite_moves_to_the_top_and_the_highlight_stays` |
| Favorites, where the game leaves: the row is kept, then clamped to the shorter list | `In_the_favorites_collection_an_unfavourited_game_leaves_and_the_highlight_keeps_its_row_within_the_shorter_list` |
| Favorites losing its last game: the system view at the collection's place | `Unfavouriting_the_last_game_of_favorites_returns_to_the_system_view_at_the_collection_s_place` |
| The game options menu's entry | `The_game_options_favourite_entry_keeps_the_highlight_too` |
| A grid (its scroll row) and a horizontal carousel (its position) | `A_grid_and_a_carousel_keep_the_highlight_where_it_was` (2 cases) |

**Before and after.** The first eight cases were written first and run on the unfixed code. All eight failed, each at
its index assertion and after its assertions on the new order had passed:

| Case | expected row | row on the unfixed code |
|---|---|---|
| a favourite at row 20 | 20 | 0 |
| unmarking at the top | 0 | 7 |
| the last game | 29 | 0 |
| All games, row 12 | 12 | 0 |
| Favorites, row 1 | 1 | 0 |
| the options menu, row 9 | 9 | 0 |
| grid, row 13 | 13 | 0 |
| carousel, row 13 | 13 | 0 |

After the fix all eight passed. The vanished-list case was written with its branch. With that branch disabled it failed,
expecting `("system", "all")` and getting `("gamelist", "nes")`. With the branch restored it passed.

**Tests that held the old rule.** Four tests asserted that the highlight follows the game, which was stage (e)'s rule
(§15.3). They were rewritten to the new rule:

- `ThemedLibraryPadTests.North_toggles_the_favourite…` and `Select_opens_the_game_options_whose_favourite_entry…`, both
  renamed to end `…while_the_highlight_keeps_its_row`;
- `SwapAndKeyboardTests.With_the_swap_X_is_the_favourite…`, which now reads the favourite at the top of the list;
- `ThemedCollectionsTests.A_collection_created_in_big_picture…`, which checked the favourite on whatever game was under
  the highlight after the toggle.

§15.2's sentence that a favourite's rebuild finds the same game, and §15.3's row "Select marks a favourite, which moves
first and stays selected", are superseded by this section.

**The run.** The blast radius was run on the final build under `nice -n 10`: every test in
`EmuSen.WiseMan.Mistress.BigPicture`, with `GameRecordsTests`, `LibraryScreenTests`, `IdentityAndCollectionsTests`,
`DesktopGameOptionsTests` and `GameMetadataTests`. **969 tests: 928 passed, 41 skipped (the picture, survey and live
tools), none failed, in 4 min 17 s.**

## 45. The Start menu reorganised into pages (2026-10-04)

*Built on branch `bigpicture-menus`, from WiseMan at `0c0335c1`. LunaP unchanged.* Decided by the tester on 2026-10-04:
reorganise the pad menu for a player with no technical knowledge, in big picture and on the desktop, with the freedom to
depart from ES-DE's layout so long as the ES-DE theming is not broken. The player's account of the result, with the
structure and the reasons for it, is §4.69.8 of the settings reference. This section is the record: what was measured,
what was predicted, what the audit found, and what the tests hold.

**Scope.** Only Mistress's own menu drawing changed: `MainWindow.PadMenu.cs` (new), `MainWindow.Pad.cs`,
`MainWindow.BigMenus.cs`, `MainWindow.ThemedKeys.cs`, `MainWindow.BigPictureCollections.cs` (one label),
`PadMenuEntry` and the pad menu's part of `MainWindow.axaml`. No file under `BigPicture/` (the theme loader, the scene,
the views) changed, and neither did LunaP. The rows are drawn by §32's `MenuPanel` and `MenuRow` with their own
properties (`Subtitle`, `Footer`, `TitleMinScale`, `TitleMaxLines`, `RuleColor`), none added.

### 45.1 The structure

Over a game: Resume, Rewind, Save State ◂ slot ▸, Load State ◂ slot ▸, Speed, Restart Game... | Game Settings ▸ |
Back to Library, Quit Game..., EmuSen ▸. Over the library: Back to *game*, Game Options..., Scrape This Game... (each
where it applied before) | Library ▸, Settings ▸, EmuSen ▸. The submenus, the questions and every row's condition are
§4.69.8's tables. The first page over a game has ten rows where it had seventeen, and the main menu has three to six
where it had up to fifteen.

### 45.2 Predictions

Written from §32.1's measured constants before the audit was run:

| # | Predicted | Found | Verdict |
|---|---|---|---|
| P400 | With the version footer on the first page, its rows' room at 800 lines is 800 − ~58 (help bar) − 48 (edges) − 100 (title) − 78 (footer) ≈ 516, so nine whole rows of 54. §4.69.3's *nine of eleven* is the same capacity | not re-measured; §4.69.3's record agrees | consistent |
| P401 | Without it (a bare 20), the room is ≈ 574, so ten rows, and the game's ten-row first page shows whole at 1280×800 and 1920×1200 | 10 rows, no scroll indicator, at both sizes | held |
| P402 | Section dividers that add height would push the first page past the whole-row floor of 540, so they must take none | the lighter rule takes none; 10 rows show | held (by construction) |
| P403 | A title of a No-Intro name's length (103 characters) wraps to two lines at the shrink floor and the first page scrolls, showing nine rows | 2 title lines, scroll indicator *Down*, at both sizes | held |
| P404 | The same name as a breadcrumb needs two subtitle lines at 26 design pixels | 2 lines (`…THE DIRECTOR'S CUT` / `(USA, EUROPE) (REV 1) (BETA)`) | held |
| P405 | The slot card fits beside the panel at both sizes: the panel is centred and no wider than 1.05 times the area's height, which on a 16:10 screen leaves more than the card needs at its 150-design-pixel floor | 202 pixels wide at 1280×800 and 303 at 1920×1200, clear of the panel and the help bar | held |
| P406 | The desktop's ten rows and two gaps (about 510 pixels) fit the old 520-pixel list | **failed**: the rows scrolled by 10 to 40 pixels on every page | see §45.3 |

### 45.3 What the audit found

`WindowFitAuditTests.Every_page_of_the_pad_menu_…` failed all six desktop cases on its first run, and passed the six
big-screen ones. On every failing page the list's first row lay above its viewport, so it was drawn cut at the top: by
40 pixels on the game's first page (ten rows and two gaps, 510 pixels of rows in a viewport of 470) and by 10 or 20
pixels on the shorter pages, including pages with no gap of their own. The rows were 49 pixels and a gap 10 (9 of
margin and a 1-pixel line). The shortfall is therefore not the gaps' height alone, and the mechanism inside
`VirtualizingStackPanel` was not isolated. What was established is this. Removing the list's 520-pixel cap changed
nothing: the next run measured the same 470 against 510. Replacing the list's panel with a plain `StackPanel` fixed all
six cases. Restoring the virtualizing panel fails the audit again (mutant M6, §45.4). The menu has at most ten rows on a
page, so virtualization bought nothing there. The cap stays removed, so the list takes the height its rows need and
scrolls only in a window shorter than that.

A reset of a page's scroll offset when it is turned to was written for the same symptom. With the plain panel the
mutant that removed it survived (M8): an offset past a shorter page's rows is clamped by the scroller itself, and no
page but the first can scroll. The reset was taken out.

### 45.4 Tests

- `PadMenuTests`, 27 cases (§4.69.8 lists them): the structure in-game and main, desktop and big picture; submenus, B
  back to the row entered from, and the remembered row; the slot on the Save and Load rows and the card; each question
  with *No* and B changing nothing and *Yes* acting; Back to Library leaving the game paused with no frame run; each
  moved row opening its window; the desktop's keys and the pointer.
- `WindowFitAuditTests.Every_page_of_the_pad_menu_…`, 12 cases: every page, submenu and question, and the card, at
  both sizes, desktop and big picture, with a long title.
- The tests that held the old names and order were updated (§4.69.8 lists them); their helpers walk the menu through the
  shared `Fixtures/PadMenu` as a player walks it.

**The run.** The blast radius was run on the final build under a memory cap of 8 GB: every test in
`EmuSen.WiseMan.Mistress` (the big picture, pad, menu, sheet, scraping and fit-audit tests among them) with the
accessibility and pad audits. **1,753 tests: 1,702 passed, 51 skipped (the picture, survey and live tools), none failed,
in 8 min 51 s.**

**Mutants.** Eight, each applied to `MainWindow.PadMenu.cs` alone, built, and run against `PadMenuTests` and the pad
menu's fit audit, the source restored after each:

| # | Mutant | Caught by |
|---|---|---|
| M1 | the question lists Yes first, so A straight through says Yes | the questions' case |
| M2 | B returns to a page's first row, not the row it was entered from | the submenu case, the keys-and-pointer case, the questions' case |
| M3 | a submenu forgets its row | the submenu case |
| M4 | Back to Library resumes the game | the Back to Library case |
| M5 | Left and Right on the Load row step the slot the other way | the slot case |
| M6 | the virtualizing panel kept | the fit audit (desktop pages) |
| M7 | Start goes back a page instead of closing the menu | the submenu case, the questions' case |
| M8 | a page turned to keeps the scroll offset of the page before | **survived**; the line was found inert and removed (§45.3) |

### 45.5 Pictures

`EMUSEN_WINDOW_FIT_PNG=<folder>` writes every audited page to `~/.cache/emusen/probe/window-look/trials/<folder>/`
as `PadMenu-<menu>-<BigScreen|Desktop>-<size>-<page>.png`. The pictures of this section were looked at page by page.
The dividers read as hairlines between the sections in the big panel and as a gap and a line on the desktop. The
breadcrumb wraps under a long title, and the question's sentence sits under its title in sentence case.

### 45.6 Not done

- The handheld and a real pad were not tried.
- No aspect ratio but 16:10 was audited. On a 4:3 screen the card's room beside the panel falls below its 150-pixel floor
  and it is not shown. That follows from the constants and was not measured.
- The card's caption is drawn in the desktop face (§4.69.8).
