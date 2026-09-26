# EmuSen_BigPicture — a plan for a big-picture mode in Mistress that renders ES-DE themes

*Written 2026-09-24. Stage (a), the theme loader, was built the same day; its record is §12. Stage (b), the two views drawn statically with LunaP controls, followed on 2026-09-24 and 25; its record is §13. Stage (c), the GPU frame and motion, followed on 2026-09-25; its record is §14. Stage (e), the themed view as the big-screen library driven by the pad, followed the same day; its record is §15. Stage (f), variants and settings, the grid, the triggers against Mistress's media, and themes downloaded on request, followed on 2026-09-25 and 26; its record is §16. Stage (d), ScreenScraper, followed on 2026-09-26; its record is §17. Stage (g) is not built.* The user asked for a big-picture mode in Mistress like EmulationStation's,
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
| Select | | mark favourite (`favorite`), as the grid does today (§4.33). *Since 2026-09-26 the game options menu, as ES-DE's Back button opens it; the favourite is its first entry (§23.4)* |
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
be offered in a desktop session, where §4.43 keeps the menu bar and sidebar. *Answered twice: in big-screen sessions
only (2026-09-25), then, on the user's request of 2026-09-26, also on the desktop, behind a Big Picture entry
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
| P84–P91 | The game options menu and the metadata editor: every control reachable, an edit surviving a re-scrape, the editor's own scrape, no ROM touched, a click on the rating, the mutants, the broad runs (§23.1) | §23.1: all held but P88 (failed, then fixed) and P90 (one unattributed failure) |
| P100–P120 | The remaining passes' predictions: the handheld, controllers, the theme survey, badges and switches, localisation, folders, the modes, scraping extras, manuals, the screensaver, the launch screen, video, TheGamesDB, the passes' pace (§21.6) | Each in its pass |

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
- **Q20–Q35**, the decisions the remaining passes to ES-DE parity wait on (video again, as Q4's revisit; codecs;
  controllers; folders; the modes; languages; the theme list; TheGamesDB; buttons; the launch screen; the screensaver;
  the hardware session), are asked in §21.5.
- **Risk: ScreenScraper's API is in beta** and may change without notice (§5.1). The client keeps the response
  parsing in one place, and its tests are written against the documented shape.
- **Risk: the undocumented behaviours** of §4.4 (the 'S' size) and §4.6 (durations) are measured from ES-DE, so they
  track the ES-DE release measured. The release is recorded with every capture.
- **Risk: the theme changes upstream.** Updating is the player's choice (§6). A new theme version that uses an element
  outside §3.8 is refused visibly, element by element, not rendered in part.

- **Q15–Q19**, the game options menu and the metadata editor's open questions: §23.12.

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
  - **Q8, amended by the user (2026-09-26):** "There needs to be a button to enter big picture mode on desktop as
    well", and, correcting a first build that made the desktop's full screen and big picture one state: "i did not
    want the fullscreen button to trigger big picture on desktop automatically, i wanted a separate button that
    triggers big picture mode separate from the fullscreen button. Clicking the button would make emusen fullscreen,
    but also put it into big picture mode. i want the user to have the option between both in desktop", and then
    placed it: "The big picture button for desktop mode should be placed under the view menu, below fullscreen". The
    desktop therefore has **two options**, adjacent in the **View** menu: a plain **Fullscreen** (F11, and the window
    manager's own), which keeps the sidebar library, and directly below it **Big Picture** (its own key, F10), which
    makes the window full screen and enters big picture. The desktop keeps its sidebar library by default, and the
    themed view is offered wherever big picture runs, under §4.52's four conditions. A Game Mode session stays big
    picture throughout. §18 is the record; §4.54 of the settings reference is the player's account.
  - **Q8, amended again (the user, 2026-09-26):** "EmuSens normal big picture theme should be an option in the themes
    list". The existing big-screen library is no longer a separate "Library style" beside the theme: it is the first
    entry of the Theme Settings sheet's Themes list, **EmuSen (built in)**, never downloaded or removed, and choosing it
    or an ES-DE theme there applies at once. Preferences' "Library style" row became **Big Picture Theme**, a dropdown
    of the same entries over the same two settings (`LibraryStyle`, `BigPictureTheme`). §19 is the record; §4.56 of the
    settings reference is the player's account.
  - **Q9:** the help bar's button icons **follow the connected pad**: Mistress detects the controller family and draws
    its own set for it (Xbox, PlayStation, Nintendo, and a generic set when unknown). The favourite, folder and badge
    graphics are Mistress's own drawings.
  - **Sounds:** the theme's navigation sounds play through a small UI sound stream, **on by default** with a switch in
    Preferences.
- **Q20–Q35, answered by the user on 2026-09-26 after §21 was written:**
  - **Q20 and Q21, video:** all three uses (the theme's clips, the media viewer and the screensaver), built late as §21's
    pass 12, through the system's own `ffmpeg` run as a separate process, so EmuSen ships no codec. Video scraping is
    off by default. This supersedes Q4's "not now".
  - **Q22–Q35:** every recommendation §21 made is accepted as written. Where a pass finds that a recommendation cannot
    hold, the user is asked again.
  - **The first pass to build:** pass 2, controllers.
- **Q7, miximages (the user, 2026-09-26, during stage d):** ScreenScraper's ready-made mix, `mixrbv2`, is fetched as the
  miximage. It looks different from ES-DE's own composed miximages; building our own composite is not wanted now.
- **Q5, the developer credentials, as received (2026-09-26):** issued to the user as EmuSen's developer, kept only in
  `~/.config/EmuSen/screenscraper-developer.json` (mode 0600) with `softname` `EmuSen-Mistress`, verified against
  `ssinfraInfos.php` the same day. No build carries them.
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

### 14.10b The handheld at 1920×1200, where the user plays (2026-09-26)

The user runs the Legion Go S at its panel's full 1920×1200, not the 1280×800 §10.1's Q2 read from gamescope's
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
  back to the pad's name. A pad whose name carries "Legion Go" is taken for an Xbox layout, as the user reports it.
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
  from a game) finds it again.
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
| Select marks a favourite, which moves first and stays selected | `Select_marks_a_favourite…` |
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


### 15.14 A closed window kept drawing (found at the merge, 2026-09-25)

**What was seen.** After stage (e) was merged into WiseMan, the Mistress test filter (about 690 tests) failed in two
of four runs, each time on a different test outside stage (e): the Shaders window's live-slider test once, the status
bar's Preferences test once. Each passed alone and in small groups. The build before the merge (`eda6c55f`, 648 tests)
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
  download: codeload's compression level was not observed, since nothing was downloaded without the user asking.
- **The attribution** is read at display time from the theme's `README.md`: the section under the first heading whose
  words contain "licen", and the one containing "credit", with Markdown's marks taken off; else a `LICENSE` file's first
  lines; else a sentence saying none is stated. For Art Book Next the licence line is its README's own, naming
  CC-BY-NC-SA 2.0 and its URL, and eight credits. The author of a downloaded theme is its repository's owner, since
  neither `capabilities.xml` nor THEMES.md has a field for one; a theme read in place states no author.

### 16.5 The grid: ES-DE measured, then built

**The measurement.** A subagent measured ES-DE 3.4.1's grid from its behaviour alone, with §14.7's rig (XWayland
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
covered cover (Ivory Signal, under the selected one) is left out, as the subagent's table marks it.

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
  picker (§6), are not built. Nothing was downloaded from GitHub: every download test used a fake server.
- **The grid's unmeasured parts.** `imageFit contain` and `cover`, selector and background images, corner radii and the
  text's scale law were not measured in ES-DE; they follow `THEMES.md` and are proved only to change the pixels. The
  horizontal clip, a held right that reaches the last item (inferred to stop, as the others do), 60 Hz timing, and the
  unscrolled inward bottom row's centring were not measured either.
- **The grid's own not-mapped properties:** `imageBrightness`, the background's and selector's gradients, the selected
  image's gradient, `textHorizontalScrolling` and its three siblings, the collections' letter cases and
  `fadeAbovePrimary`. Art Book Next sets none of them.
- **Quick system select in a grid gamelist** does not exist: all four directions move the grid. What ES-DE does there
  was not measured.
- **Languages** are listed only when a theme declares them; no theme the user has declares any, so the row was tested
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

The brief was §7's row: scraped media as one more source for the themed view. Two decisions came during the build,
through the coordinator, and are recorded here because they reverse parts of §5:

- **ScreenScraper became Mistress's main source of cover art and game text everywhere**, the library's grid and list as
  well as the themed view, whenever the developer file is present. OpenEmu's sources (§4.39 of the settings reference)
  became an opt-in failover, **on by default by the coordinator's choice**, which the user may reverse. It fills a cover
  only where ScreenScraper found none or cannot be used.
- **Ship-ready by default.** Where the developer file exists, scraping works with no action, and it is reachable from the
  pad in Game Mode. *Reversed the same day by the user's rule of 17.14: nothing is asked until the player starts a run.* `Scraping` is therefore on by default. §5.8's "off by default, one explicit action" is kept in the one
  place it still protects someone: a build without the developer file sends nothing to ScreenScraper, and there the
  failover is what runs, with its hint saying what it sends. That the failover itself is now on by default is a change
  from §4.39's "off unless the player turns it on", and is the coordinator's decision, not an argument made here.

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

**Nothing leaked.** No file in the run's folder, the test's log, either worktree, the scratch folder or the mutant
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
- **No refresh**, no search by name, no video (Q4), no back cover, fan art or 3D box.
- **§5.7's publish exclusions** are a rule written here and in §4.60 of the settings reference, not a script.

### 17.14 Scraping only when the player starts it, and the scope the player chooses

**The user's two rules (2026-09-26, after 17.1–17.13 were built),** which override what 17.2 and 17.3 describe:

1. *Every run is initiated by the user.* "I do not want to spam the screenscraper api." Nothing may reach ScreenScraper
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
display is paced and within quota, so it is safe. The user's rule is not about the quota but about the service's load
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

**The live run of 17.9 was made before this change**, by a tool that drives `Scraper` directly and is itself a deliberate,
one-off run started by a person; its numbers stand.

---

## 18. Big picture from the desktop: Big Picture below Fullscreen in the View menu (2026-09-26)

*Opened 2026-09-26, on the user's request (§10.1, Q8 amended):* "There needs to be a button to enter big picture mode
on desktop as well". Until this section, big screen was a decision the window took once, when it was made (settings
reference §4.29, §4.43), and §15 built the themed view on that premise. This section makes the decision switchable while
the window runs. It was built three times in one day, because the request was relayed wrong once and then placed:

| Build | What it was | Why it changed |
|---|---|---|
| 1 (`7e9befdd`–`bff37489`) | the desktop's full screen *was* big picture: a toolbar Fullscreen button, F11, the View menu and the window manager all entered it | a relay of the user's second message ("fullscreen mode enters emusens big picture mode") was taken as the design. The user: "i did not want the fullscreen button to trigger big picture on desktop automatically, i wanted a separate button … i want the user to have the option between both in desktop" |
| 2 (`83ff0243`, `d48cce2e`) | two toolbar buttons, a plain Fullscreen and a Big Picture, with a View menu entry and a key for each | the user: "The big picture button for desktop mode should be placed under the view menu, below fullscreen" |
| 3 (`d79dbdf1`) | the View menu holds Fullscreen (F11) and directly below it Big Picture (F10); no toolbar buttons | — |

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
| D2 | full screen enters big picture again (the coupling the user rejected) | caught |
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

### 18.6 Not done

- **No real window manager was used.** KDE's and GNOME's own full-screen commands, on X11 and on Wayland, reaching
  Avalonia as `WindowState.FullScreen`, and a normal window getting its size back, are assumed from Avalonia and LunaP
  (§75.2 there), not observed. The headless platform takes every state it is given (X2).
- **F10 was chosen, not surveyed.** GTK applications open their menu bar on F10; a desktop binding it globally would take
  it before Mistress. It can be rebound.
- **No pointer way out of big picture.** The menu bar is hidden there; a mouse-only player leaves by the window manager's
  full-screen command. Build 2's toolbar button was such a way, and it went with the user's placement.
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

*Opened 2026-09-26, on the user's request (§10.1, Q8 amended again):* "EmuSens normal big picture theme should be an option
in the themes list". §4.10 had kept the existing big-screen library as a separate choice, Preferences' "Library style"
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

*Opened 2026-09-26, on two requests of the user's.* The first: "We need to add a status window for when you are scraping
roms". The second, relayed during the build: "we also need to add the ability for the user to log into screenscraper
with their own account credentials if they prefer". Both belong to stage (d)'s Scraping tab and to §17.14's rules, which
bind them: a run is started only by the player, and nothing is asked of a server outside one. The player's account is
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
the gaps into passes an agent can build and test in one go, orders them, and states what only the user can decide. It
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
- EmuSen's code at `5dc543aa`, and the ROM library at `AppSettings.RomDirectory`, listed read-only.

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
| 7 | Folders | shown as entries, entered with A; sorted on top; folder badge; folder link; `defaultFolderImage`; `gamelistinfo`'s folder icon (UG "Multiple game files installation", "Metadata editor", TH "grid", "gamelistinfo") | flattened (`RomLibrary`); the user's NES and GB are all folders (§21.1) | folder entries, entering and leaving, sorting, the badge |
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

Each pass is a unit an agent can build, test and record in one go, as a stage was. Costs are in §7's unit, working days,
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
  stream (§15.6); the help icons at arm's length. Then, with the user's hands on the device: one member sign-in against
  the live service (§20.6), and, if Q35 allows, the four-file N64 run that settles P60 (about 20 requests).
- *Depends on.* The user's time, and Q35. No code dependency.
- *Oracle and tests.* The device itself; a frame log in Mistress's themed render loop, written under an environment
  variable, read after the run. Long runs go under `systemd-run --user`, because the device ends processes an ssh
  session leaves.
- *LunaP.* None.
- *Risks.* The user's password must be typed by the user into Mistress; the agent never sees or relays it. The sign-in
  and the N64 run are live requests, each started by the user.
- *Cost.* 1 day, and one session of the user's.
- *What the user sees.* No new feature: a record of what the device does, the Game Mode frame §14.10a left unmeasured,
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
- *What the user sees.* Any pad steers big picture; "Controller connected" and "disconnected" notices; icons that can
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
- *What the user sees.* Every official ES-DE theme in the Themes tab, with pictures and licences, downloadable and
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
- *What the user sees.* A clock on request, badges on any theme, the ES-DE settings a player expects in Theme Settings
  and Preferences.

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
- *What the user sees.* Nothing changes in English: a language setting that offers English alone until Pass 15, and a
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
- *What the user sees.* NES opening on its 16 folders and GB on its letters, as ES-DE would show them, or the flat list
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
- *What the user sees.* A mode a child can be handed.

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
  started by the user, measures what the fake cannot: the checksum answer's cost (P111).
- *LunaP.* None expected; the chooser is a `LunaList` of names with a thumbnail.
- *Risks.* **Quota and load.** Every kind is a request (P61), and the four picture kinds add about 3.2 requests and
  1.2 MB to a found game by §21.1's rates, nearly doubling a whole-library run; manuals add 0.7 requests and 1.2 MB, with
  a largest file of 18 MB. The defaults stay those of §4.60 and each new kind is the player's choice. **Terms.** A name
  search counts against the day's unrecognised allowance when it finds nothing (§5.2).
- *Cost.* 3–4 days.
- *What the user sees.* More kinds in the Scraping tab, a Refresh, and a "Find by name…" for the games hashes miss.

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
- *What the user sees.* West on a game shows its pictures full screen, and its manual.

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
- *What the user sees.* The view dims, or a slideshow of the library, after five idle minutes.

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
- *What the user sees.* The game's art and name for a moment before the game, as in ES-DE.

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
- *What the user sees.* After three seconds on a game, its clip plays in Art Book Next's frame, once, with sound; clips
  in the viewer and the screensaver.

**Pass 13. TheGamesDB, waiting on Q29.**
- *Scope.* A second client, used only inside a run the player started, and only for games ScreenScraper left without
  something TheGamesDB has: its `ByGameHash` by MD5 and then CRC with the platform, then an explicit name search from the
  chooser of Pass 8; its images (front and back box, screenshot, title screen, clear logo, fan art); its allowance read
  from every answer and from `/v1/API/Limit`, which costs nothing.
- *Depends on.* Q29; Pass 8's chooser.
- *Oracle and tests.* A fake TheGamesDB built from the spec's documented answers; one live run of about ten games,
  started by the user, with their key.
- *LunaP.* None.
- *Risks.* **The key.** A key belongs to a logged-in site account (§21.1), so it is handled as the ScreenScraper developer
  file is: in a file outside the repository, mode 0600, passed through the redactor, never in a build (Q29). **Terms.**
  The data's terms of use were not found; the pass reads them first, and stops if they forbid the use. **Quota.** A
  monthly allowance, not a daily one, so a whole-library run is not an option on it.
- *Cost.* 2–3 days, plus the time to obtain a key.
- *What the user sees.* Games ScreenScraper misses, filled from a second source.

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
- *What the user sees.* The themes the user picks from the list drawn as ES-DE draws them.

**Pass 15. Translations.**
- *Scope.* The languages Q26 names, one at a time, on Pass 5's plumbing.
- *Depends on.* Pass 5; Q26; last, so that the strings have settled.
- *Oracle and tests.* A speaker's review; the pseudo-locale test stays the guard; a clipping check per language at
  1280×800.
- *LunaP.* Its own strings, in the same languages.
- *Risks.* Quality, and provenance: a translation's source is recorded; none is copied from ES-DE.
- *Cost.* 1–2 days a language, plus review.
- *What the user sees.* Mistress in the chosen languages.

### 21.4 The recommended order, and why

| # | Pass | Cost (days) | Waits on |
|---|---|---|---|
| 1 | The hardware session | 1 | the user's time, Q35 |
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
  the one with the legal question and the one waiting on the user (Q20, Q21). And when it is built, two of its three
  consumers already exist, so the decoder is designed once against the element, the viewer and the saver together,
  rather than for the element and then bent.
- **TheGamesDB late,** because ScreenScraper found 39 of 40 by hash (§17.10), so a second source fills little, and it
  waits on a key.
- **Pass 14 late but movable:** if the user picks a theme from Pass 3's list that needs an element, that element's part
  of Pass 14 moves up to follow Pass 3.
- **Translations last,** when the strings have stopped moving.

### 21.5 The decisions only the user can make

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
- **Q23, folders.** The user's NES games are all in 16 region and category folders and the GB games in 28 letter folders
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
  (b) the plumbing, then languages the user names, drafted and reviewed by a speaker;
  (c) the plumbing, and a file format a community could translate.
  **Recommendation: (a) now, (b) for any language the user names.** No translation is taken from ES-DE.
- **Q27, which themes the list offers.** Options: every theme of ES-DE's list; only those that state a licence; only
  those Mistress draws fully after Pass 14. **Recommendation: every theme**, with its licence line or "states no licence"
  shown before the download, since the download is the player's, from the author's own repository, as it is in ES-DE.
- **Q28, the survey's download.** Options: every theme's archive (gigabytes); XML files only, through the hosts' tree
  listings; a sample the user picks. **Recommendation: XML only** (P104 predicts under 50 MB), kept under
  `~/.cache/emusen/bigpicture/survey/`.
- **Q29, TheGamesDB.** Options:
  (a) not at all;
  (b) the user's own key, in a file outside the repository, used only on the user's machines, as Q5's answer did for
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
- **Q35, the hardware session.** When can the user run it, and may it include one sign-in with the user's own member
  account, typed by the user, and the four-file N64 run of about 20 requests that settles P60? **Recommendation: yes to
  both, in one sitting.**

### 21.6 Predictions

Written before any pass is built, to be retired in each pass's record.

| # | Pass | Prediction | Retired when |
|---|---|---|---|
| P100 | 1 | In Game Mode, SDL reports the Legion Go S's pad as Steam's virtual pad (its name or an Xbox type), and `PadFamilies` gives Xbox with no change to §15.5's rules | Pass 1 |
| P101 | 1 | Game Mode's compositing adds under 1 ms to the median frame of the held carousel at 1920×1200 against Desktop Mode, and no frame in 1,800 exceeds 16.7 ms | Pass 1 |
| P102 | 1 | A navigation sound starts within 100 ms of its press while a game is suspended behind the library, the 4,096-frame device buffer of §15.6 included, as SDL's queue and buffer sizes report it | Pass 1 |
| P103 | 2 | Every pad rule of §15.3 holds from either of two pads, and the help bar follows the pad last pressed; ES-DE's device popup fades in and out over 0.4–0.6 s each and holds 2–5 s | Pass 2 |
| P104 | 3 | Of ES-DE's listed themes, at least 90% load for EmuSen's five systems with no loader error, and at least half use an element or carousel type Mistress does not draw; the XML-only survey fetches under 50 MB | Pass 3 |
| P105 | 4 | Built-in badges change no pixel of Art Book Next's gamelist, which names its own icons; on a synthetic theme that names none, each of the nine slots draws | Pass 4 |
| P106 | 4 | Turning the clock on changes only pixels inside the clock's box, as P43 found for the help bar | Pass 4 |
| P107 | 5 | At the pass's start, the pseudo-locale walk finds 300–900 distinct visible strings outside the lookup in a big-screen session's sheets; at its end, none | Pass 5 |
| P108 | 6 | `EsdeMediaFolder` finds none of the media of a game in a subfolder of an ES-DE-written tree (a defect predicted from the code and UG's path rule), shown by a test on the unchanged reader before the fix | Pass 6 |
| P109 | 6 | Shown as folders, NES opens on 16 entries and GB on 28; the return from a game in a folder comes back to that folder and game, its first frame equal to a fresh build (P40's test); the first showing costs within 10% of the flat one | Pass 6 |
| P110 | 7 | Kid mode lists exactly the kidgame games, and in Kiosk and Kid `PadAudit` reaches no removed control on any sheet of §15.9 | Pass 7 |
| P111 | 8 | Refreshing a found game whose files are unchanged costs one request per kind and under 1 KB received per kind, each request counted in `requeststoday` (P61) | Pass 8 |
| P112 | 8 | Back covers, 3D boxes, physical media and fan art together raise a found game from 4.7 to 7.5–8.5 requests and add 0.9–1.5 MB, taking a found game from about 13 s to 20–28 s at 128 KB/s | Pass 8 |
| P113 | 9 | One page of a median ScreenScraper manual (1.7 MB) renders at 1920×1200 in under 300 ms on the desktop and under 1 s on the handheld | Pass 9 |
| P114 | 10 | Dim and Black draw at most one frame after they start, and the slideshow draws only at its swaps and their transitions (every 10 s by ES-DE's default) | Pass 10 |
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

## 23. The game options menu and the metadata editor (2026-09-26)

*Opened 2026-09-26, on the third item of the user's list of what big picture lacks against ES-DE:* the per-game options
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
| Enter folder (override folder link) | not offered: Mistress's library has no folders and no folder links |
| Apply, Cancel | not offered: every entry built here acts when chosen; they belong with the jump and the sort. A *Close* button stands for Cancel |
| *(not in ES-DE's menu)* | **built**: *Add to Favourites* / *Remove from Favourites*, and *Scrape This Game...* |

The two entries ES-DE's menu does not have are there for a reason each. ES-DE toggles a favourite with Y (its option
*Enable toggle favorites button*, on in the scratch home), but Y, North on the pad, is the search in §4.9's grammar, which
the user's decisions of §10.1 made and which ES-DE has no counterpart for. Select was the favourite until now; this work
gives Select to the menu, as ES-DE does, and the favourite had to go somewhere one press from the list. *Scrape This
Game...* is the pad menu's entry of §17.14 again, with the same confirm step and the same run. ES-DE starts a single-game
scrape only from inside the editor, which Mistress also does (below); the brief asked for it in the menu as well. Both are
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
| Folder link | not applicable: no folders | |

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
for the whole entry; the brief asked that a field can be returned on its own. Each field says where its value comes from
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
- *JSON beside the library.* Ruled out by the brief: Mistress's data is in SQLite.

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
  and play rows kept and an edit written with its time (the user's requirement, relayed 2026-09-26, that everything the
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
coordinator restored the file, which was byte-identical to the commit, and deleted the backup. The tree was then rebuilt
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
- ScreenScraper's own name is not offered by the editor's scrape (§17.6 keeps it unused).
- *Clear* on a game takes the store's pictures of any other file of the same system with the same stem (§23.5).
- The two hooks of §23.4 are empty on this branch; the collections entries are §22's.
- EmuSen's own built-in library keeps Select as the favourite and has no editor.
- Nothing ran on the handheld.

### 23.12 Questions for the user

- **Q15, the favourite's button.** ES-DE toggles a favourite with Y; Mistress's Y is the search (§4.9). The favourite is
  now the first entry of the options menu, two presses from the list. Keep it there, or give Y to the favourite and move
  the search?
- **Q16, hidden games.** ES-DE lists hidden games, dimmed, by default; Mistress leaves them out unless Preferences ▸ Hidden
  Games is on, because Hide from Library replaces ES-DE's Delete. Keep that default?
- **Q17, what Clear removes.** It removes ScreenScraper's pictures in Mistress's store, but not the cover OpenEmu's
  failover fetched, nor the favourite and play counters, which ES-DE's Clear takes with the gamelist entry. Should it take
  more?
- **Q18, ScreenScraper's name.** ES-DE's editor scrape replaces the name with the scraper's; Mistress keeps the file's name
  unless the player types one (§17.6). Offer ScreenScraper's name in the editor's scrape?
- **Q19, EmuSen's own library.** Should its Select open the same menu and editor, or stay the favourite?
