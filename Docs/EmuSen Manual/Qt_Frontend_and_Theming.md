# Qt Frontend and Theming

EmuSen's graphical frontend lives under `Frontend/` and pairs Rust application logic with Qt 6 / Qt Quick (`QML`) presentation via `cxx-qt`. The architecture enforces a strict separation of concerns: all state, navigation logic, formatting, input handling, and platform service calls live in Rust, while QML remains a thin declarative layout layer. Built-in and third-party QML themes run inside a sandboxed `QQmlEngine` backed by the read-only `EmuSen.Theme` module, alongside a dedicated ES-DE compatibility engine for EmulationStation Desktop Edition XML theme packages.

For prerequisites and build commands see [Building and Running](Building_and_Running.md). For the underlying C APIs consumed by the frontend see [Platform C Interfaces](Platform_C_Interfaces.md), and for automated UI checks and compositor gates see [Testing and Verification](Testing_and_Verification.md).

---

## 1. Frontend Workspace and Fenced Crate Architecture

`Frontend/` contains **21 Rust crates** intentionally divided between one Cargo workspace (12 Qt-linked crates) and 9 standalone crates excluded from the workspace root.

### 1.1 Dependency Fence Rationale

The 9 excluded crates carry independent `Cargo.lock` files so dependency-fence tests can inspect their dependency trees directly and guarantee that pure-Rust crates never transitively pull in Qt, C++ toolchains, or platform UI libraries. This keeps layout math, static linters, ES-DE theme loading, and C-ABI clients fast to compile and safe to link from non-Qt crates (for example, `Platform/emusen-dev-module` depends directly on `Frontend/qml-rules` and `Frontend/esde`).

### 1.2 Complete 21-Crate Map

| Crate | Path | Workspace | Primary Responsibility |
|---|---|---|---|
| `emusen-kit-core` | `Frontend/kit-core` | Standalone | Pure-Rust UI heart: design tokens, geometry, focus/navigation walks, controller glyphs, settings/table models, motion/orbit/wheel helpers, and automated UI audits (`fit`, `pad_reach`, `pointer_reach`, `no_chrome`, `named`, `contrast`). |
| `emusen-kit-qt` | `Frontend/kit-qt` | Member | Qt 6 / `cxx-qt` wrapper over `kit-core`, providing the `EmuSen.Kit` QML module, reusable controls, fonts, audio bells, focus router (`PadRouter`), reset gate, and the `EmuSen` `QQuickStyle`. |
| `emusen-kit-gallery` | `Frontend/kit-gallery` | Member | Standalone gallery executable (`kit-gallery`) showcasing every `EmuSen.Kit` control and supporting headless frame capture via `--grab`. |
| `emusen-qt-models` | `Frontend/qt-models` | Member | View models bridging `platform-client` and `crystal-client` to QML: library shelves, consoles, games, game pages, settings, pause card, Moments rewind reel, cheats, firmware, shaders, screenshots, scraping, and themes. |
| `emusen-qt-endymion` | `Frontend/qt-endymion` | Member | Qt-facing input and UI audio bridge over Endymion: SDL3 gamepad polling, multiplayer seat tracking, hotkey handling, keyboard routing, and navigation cue playback. |
| `emusen-qt-app` | `Frontend/qt-app` | Member | Primary desktop and handheld application binary (`emusen-qt`), managing the main window, startup sequence, splash sound, style setup, theme loading, and optional `gate-pad` test harness. |
| `emusen-qt-play` | `Frontend/qt-play` | Member | Standalone gameplay host binary (`qt-play`) for launching a title directly into a Vulkan-backed Qt Quick gameplay window. |
| `emusen-theme-qt` | `Frontend/theme-qt` | Member | Registers the read-only `EmuSen.Theme` 1.0 QML module exposing every singleton and row model visible to QML themes. |
| `emusen-theme-host` | `Frontend/theme-host` | Member | Sandboxed QML engine host (`ThemeHost`) enforcing URL interception, module allow-listing, package loading, and verified engine teardown. |
| `emusen-qml-rules` | `Frontend/qml-rules` | Standalone | Pure-Rust static analyzer enforcing the QML zero-logic boundary on internal views and external theme packages. |
| `emusen-theme-check` | `Frontend/theme-check` | Standalone | Static theme package validator verifying directory layout, `theme.json`, tokens, QML rules, and asset constraints. |
| `emusen-theme-manifest` | `Frontend/theme-check/manifest` | Standalone | Plain-data parser and schema validator for `theme.json` (`SCHEMA = 2`, API `1.24`). |
| `theme-audit` | `Frontend/theme-check/audit` | Member | Loaded offscreen theme auditor (`theme-audit`) running `ThemeHost` across viewport sizes, variants, reduced motion, and 15 synthetic library states. |
| `emusen-esde` | `Frontend/esde` | Standalone | Pure-Rust ES-DE theme engine: XML theme loader, element resolution, scene timeline, built-in icon marks, shelf text mapping, font lookup, and corpus measurement (`esde-check`). |
| `emusen-esde-text` | `Frontend/esde-text` | Standalone | HarfRust + `read-fonts` text shaper reproducing ES-DE's pixel-rounded glyph advances, word wrapping, `lineSpacing` pitch, and `"..."` ellipsis rules. |
| `emusen-esde-qt` | `Frontend/esde-qt` | Member | Thin Qt Quick painter exposing the `EmuSen.Esde` QML module; renders exact records computed by `emusen-esde` without adding layout decisions. |
| `emusen-qt-test` | `Frontend/qt-test` | Member | Offscreen Qt Quick harness, layout tree extractor, `spec 1` specification recorder, mockup comparator, virtual pad queue, and scoped Wayland compositor helper. |
| `emusen-platform-client` | `Frontend/platform-client` | Standalone | Safe Rust client over `emusen_platform.h`, `emusen_library.h`, and platform service tables. |
| `emusen-crystal-client` | `Frontend/crystal-client` | Standalone | Safe Rust client over `emusen_crystal.h` and `emusen_session.h`. |
| `emusen-c-calls` | `Frontend/c-calls` | Standalone | Shared FFI primitives: dynamic library loading, upfront version verification, length-query copy-out, thread-local `last_error` capture, and JSON decoding. |
| `emusen-client-fixture` | `Frontend/client-fixture` | Standalone | Deterministic synthetic fixtures for testing client crates and `qt-models` without launching live cores. |

### 1.3 View-Model Layer (`qt-models`) and Kit Core (`kit-core`)

`Frontend/qt-models` translates platform and session data into immutable, signal-driven Qt models grouped by domain:

- **Browsing & Library:** `library_model.rs`, `console_list.rs`, `games.rs`, `gamelists.rs`, `home_card.rs`, `list_card.rs`, `rows_model.rs`, `esde_shelves.rs`, `esde_texts.rs`, `art_hues.rs`.
- **Game Page & Session:** `game_page_model.rs`, `launch_model.rs`, `session_model.rs`, `pause_model.rs`, `reel.rs` (Moments rewind strip), `resume.rs`, `screenshots.rs`, `cheats.rs`.
- **Settings & Hardware:** `settings_model.rs`, `settings_controllers.rs`, `settings_saves.rs`, `settings_shaders.rs`, `bindings.rs`, `controllers.rs`, `display.rs`, `firmware.rs`, `themes_model.rs`, `battery.rs`, `clock.rs`.
- **Interactive Chrome:** `help_model.rs`, `pane_model.rs`, `question_model.rs`, `tool_model.rs`, `scrape_run.rs`, `marks_object.rs`, `words_object.rs`.

`Frontend/kit-core` provides the shared UI algorithms beneath both `kit-qt` and `theme-audit`:

- **Tokens & Colour Math (`tokens/`):** Semantic colour tokens, contrast calculation, dark/light variant derivation, and typography scales.
- **Navigation & Input (`input/`, `controller/`, `keyboard.rs`, `group_grid.rs`):** Deterministic focus walks (`WalkForm`), directional reachability graphs, and keyboard/pad event normalization.
- **Procedural Visuals (`glyph/`, `planet_mark.rs`, `orbits.rs`, `wheels.rs`, `canvas.rs`, `motion.rs`):** Vector controller glyphs, console planet emblems, orbital transitions, and reduced-motion motion curves.

---

## 2. QML Zero-Logic Boundary (`qml-rules`)

QML files in EmuSen are strictly declarative layout descriptions. Business logic, localized strings, state machines, and list construction belong in Rust (`kit-core`, `qt-models`, or `theme-qt`).

`Frontend/qml-rules` strips QML comments and scans `.qml` files against **seven rules**, emitting one `file:line rule words` diagnostic per violation:

| Rule | Tag | What It Rejects |
|---|---|---|
| `Rule::Function` | `function` | Any JavaScript `function` definition, except a `Connections` signal handler whose body is a single call into Rust. |
| `Rule::CallLater` | `callLater` | Calls to `Qt.callLater`, which mask ordering bugs inside QML event loops. |
| `Rule::Literal` | `literal` | Inline array or object literals constructed in QML rather than supplied by a Rust model. |
| `Rule::Words` | `words` | Hardcoded player-facing strings outside the 13 structural `NAMED_BINDINGS` (`objectName`, `role`, `font.family`, `source`, `fragmentShader`, `vertexShader`, `property`, `properties`, `pragma`, `Accessible.role`, `placeholderText`, `sequence`, `contextType`). All visible prose must come from Rust (`Words` or model roles). |
| `Rule::Completed` | `onCompleted` | `Component.onCompleted` blocks performing anything beyond taking initial focus. |
| `Rule::RowsInQml` | `rows` | Inline `ListModel` declarations carrying rows directly in QML. |
| `Rule::Import` | `import` | Importing modules outside the allow-list, importing banned modules, or importing `EmuSen.Kit` without its required alias (`import EmuSen.Kit as Kit`). |

### 2.1 Import Allow-Lists and Modes

- **Application views (`ALLOWED_IMPORTS`):** Prefix-matched against `QtQuick`, `QtQuick.Controls`, `QtQuick.Templates`, `QtQuick.Layouts`, `QtQuick.Shapes`, `QtQml`, `EmuSen.`, and `QtMultimedia` (used by `VideoTile` in `kit-qt`). `QtQuick.LocalStorage` is explicitly banned (`FORBIDDEN_IMPORTS`). `QtQml.Models` is allowed for `DelegateChooser`, while `ListModel` remains rejected by `Rule::RowsInQml`.
- **Sandboxed themes (`THEME_IMPORTS`):** Exact-match allow-list shared with `theme-host`: `QtQuick`, `QtQuick.Layouts`, `QtQuick.Effects`, `QtQuick.Shapes`, `QtQuick.Controls`, and `EmuSen.Theme`, plus quoted relative local directories (such as `import "parts"`). Themes cannot import `EmuSen.Kit`, `QtMultimedia`, `QtQml`, or arbitrary Qt modules.
- **Fixture mode (`Options { fixture: true }`):** Reserved exclusively for test fixtures and `kit-gallery`, where placeholder strings, rows, and literals stand in for live models.

---

## 3. `EmuSen.Theme` API and Sandboxed Theme Host

Themes can completely redesign EmuSen's visual experience without gaining filesystem, network, or engine privileges.

### 3.1 Sandboxed Execution (`theme-host`)

`Frontend/theme-host` isolates every loaded theme inside its own `QQmlEngine` (`Sandbox`) protected by a Rust `Policy` wired into Qt's URL interceptor:

1. Every QML source, import, SVG/PNG image, font, and shader URL passes through `policy_reads` and `policy_judge`.
2. Any path outside the theme package's own directory and Qt's installed QML module root is rejected and logged (`sandbox_take_log`).
3. Every `.qml` file read from the package is checked by `qml-rules::check_theme` before the engine compiles it, blocking JavaScript logic or unauthorized module imports at load time.
4. On theme switch or unload, `theme-host` destroys the sandbox engine and checks via `Watch` (`watch_alive`) that the `QObject` is genuinely freed.

### 3.2 `EmuSen.Theme` QML Module (`theme-qt`)

`Frontend/theme-qt` registers the `EmuSen.Theme` QML module. Every type exposed to themes is a read-only data view or a host-gated action dispatcher:

| QML Element | Backing File | Responsibility |
|---|---|---|
| `Actions` | `actions.rs` | Host-validated actions a theme may invoke (launch game, switch screen, toggle favorite, open settings/scraper). The host may decline individual actions cleanly. |
| `Bells` | `bells.rs` | Triggers theme or built-in UI sound cues. |
| `Browse` | `browse.rs` | Current browsing state: active shelf, selected console, search/filter state, letter-jump index, sort direction, and focus restoration. |
| `Collections` | `collections.rs` | Read-only game collections directory (`focused`, `totalCount`, `footerRight`, `items`) for Desktop Mode themes. |
| `Cues` | `cues.rs` | Resolves the 7 theme audio cue slots (`scroll`, `systembrowse`, `select`, `back`, `favorite`, `launch`, `quicksysselect`). |
| `Device` | `device.rs` | Host environment state: battery charge/status, clock, reduced-motion mode, text scale, and active controller glyph family. |
| `Help` / `HelpEntries` | `help.rs` | Context-sensitive bottom help-bar prompts (button glyph + action label) tailored to the active screen and focused control. |
| `Library` | `library.rs` | Library totals, suspended game status, recent/favorite collections, and shelf rows. |
| `Marks` | `marks.rs` | Built-in console planet marks, logos, and semantic vector glyphs exposed as image sources. |
| `Page` | `page.rs` | Full game-detail state for the `game` screen: metadata, cover/screenshot art, save states, Moments reel, cheats, disc media, and package actions. |
| `Rows` | `rows.rs` | Generic read-only list model backing consoles, games, shelves, states, and settings items. |
| `Session` | `session.rs` | Active or suspended emulation session identity, pause/resume availability, and live Desktop HUD state (`slots`, `cheats`, `consoleOptions`, `rewindMarks`). |
| `Settings` | `settings.rs` | Structured settings sections, row metadata, badges, filter, and modified-only state for `settings` and `desktopSettings`. |
| `Splash` | `splash.rs` | Startup splash timeline, readiness progress, and completion/skip state. |
| `Themes` | `themes.rs` | Read-only installed EmuSen and ES-DE theme catalog, active theme metadata, validation findings, and preview URLs. |
| `Tools` | `tools.rs` | Read-only Tools directory (`count`, `focused`, `activeKey`, `busy`, `summary`, `entries`) for Desktop Mode themes. |
| `Words` | `words.rs` | Complete dictionary of host-owned UI strings so theme QML never hardcodes visible labels. |
| `Workspace` | `workspace.rs` | Read-only Desktop Mode workspace, catalog, console scope, table sort/filter, batch-selection, and game inspector projection. |

### 3.3 `theme.json` Manifest Schema

Every QML theme package is described by `theme.json`, validated by `Frontend/theme-check/manifest` (`SCHEMA = 2`, current runtime API `1.24`):

| Key | Required | Description |
|---|---|---|
| `schema` | Yes | Manifest schema version (`1` or `2`). |
| `api` | Yes | Target `EmuSen.Theme` minor API version string (for example `"1.24"` or `"1.23"`). |
| `name` | Yes | Human-readable theme name. |
| `author` | Yes | Author or project credit. |
| `licence` | Yes | Licence identifier or text. |
| `variants` | Yes | Non-empty subset of `["dark", "light"]`. |
| `fontSizes` | Optional | Supported font scale multipliers (for example `[1, 1.3]`). |
| `screens` | Schema 1 | Subset of the 6 Big Picture screen roles: `home`, `consoles`, `games`, `splash`, `settings`, `game`. |
| `faces` | Schema 2 | `{ "bigPicture": [...], "desktop": [...] }` listing Big Picture screens and Desktop Mode screens (`library`, `table`, `playing`, `settings`). |
| `tokens` | Optional | Per-variant token overrides (`tokens.dark`, `tokens.light`, and Schema 2 `tokens.<variant>.desktop`) tuning palette, radii, glow, corona, and fonts. |
| `cues` | Optional | Custom audio cue files overriding any of the 7 canonical cue names. |
| `previews` | Optional | Relative paths to preview PNGs displayed in the theme selector. |

---

## 4. Bundled QML Themes: `Eclipse` vs Reference `EmuSen`

Two QML themes ship under `Frontend/themes/`:

| Attribute | `Eclipse` (`Frontend/themes/eclipse/`) | `EmuSen` (`Frontend/themes/emusen/`) |
|---|---|---|
| Role | Primary flagship dark theme | Reference baseline and fallback theme |
| `theme.json` API | `1.20` | `1.3` |
| Variants | `["dark"]` | `["dark", "light"]` |
| Declared `fontSizes` | Host default scaling | `[1, 1.3]` |
| Screens Provided | All 6 (`home`, `games`, `consoles`, `splash`, `settings`, `game`) | 3 browsing screens (`home`, `consoles`, `games`) |
| Reusable QML Parts | Modular components under `parts/` (`Board`, `Brand`, `CoverCard`, `EmptyCard`, `MediaCard`, `PlanetSphere`, `Horizon`, `HelpLine`, `Nav`, `Pill`, `PageRow`, `LineBox`, etc.) | Minimal reference screen views |
| Preview Assets | Verified via `theme-audit` | `assets/previews/{home,consoles,games}.png` |

### 4.1 `Eclipse` Token Reference (`Frontend/themes/eclipse/theme.json`)

| Token Group | Keys and Values |
|---|---|
| **Surfaces & Scrims** | `surface: #07080F`, `surfaceRaised: #151826`, `inputSurface: #11131C`, `scrim: #9E04050A`, `overArtScrim: #F004050A`, `border: #5E647B`, `rule: #59F6E7D8` |
| **Ink & Accent** | `text: #F4F1EC`, `textStrong: #F4F1EC`, `muted: #B9BED0`, `accent: #F6E7D8`, `overAccent: #07080F` |
| **Focus Ring & Glow** | `ringGapped: true`, `ringThickness: 2.0`, `ringOffset: 2.0`, `ringGapColor: #07080F`, `hoverBorder: 1.33`, `glowColor: #66F0B4C4`, `glowBlur: 26.67`, `glowSpread: 5.33` |
| **Corona & Depth** | `coronaColor: #F6E7D8`, `coronaRim: 1.33`, `coronaGlowColor: #99F6D6C8`, `coronaGlowBlur: 29.33`, `coronaGlowSpread: 5.33`, `coronaHaloColor: #47ECA0BA`, `coronaHaloBlur: 113.33`, `coronaHaloSpread: 30.67`, `shadowOpacity: 0.6`, `shadowOffsetY: 12.0` |
| **Geometry & Layout** | `rowHeight: 56.0`, `controlHeight: 48.0`, `helpBar: 50.67`, `rowRadius: 13.33`, `tileRadius: 14.67`, `buttonRadius: 24.0`, `dashPattern: "3 2"` |
| **Typography** | `faceText` / `faceLabel` / `faceInterface: "Sora"`, `faceCode: "JetBrains Mono"`, `weightTitle: 600`, `weightSentence: 300`, `weightWordmark: 600`, `fontBody: 14.0`, `fontCaption: 15.33`, `fontRow: 16.0`, `fontTitle: 29.33`, `fontDisplay: 69.33` |

---

## 5. Validating Themes with `theme-check` and `theme-audit`

Theme packages are validated in two stages: pure-Rust static analysis (`Frontend/theme-check`) and full Qt Quick offscreen rendering (`Frontend/theme-check/audit`, binary `theme-audit`). Both stages can also be invoked through DianaSh's `theme-check` command (`Platform/emusen-dev-module`).

### 5.1 Static Checks (`emusen-theme-check`)

Static checking runs without Qt or a display server and verifies:

1. **Manifest (`manifest`):** Schema version (`1`), API version (`<= 1.20`), recognized keys, valid variant and screen names, and valid cue slot names.
2. **Package (`package`):** Presence of every declared screen QML file and preview PNG, safe relative paths (no traversal outside package root), and zero-logic compliance via `qml-rules::check_theme`.
3. **Tokens (`tokens`):** Valid hex colours, positive dimensions, finite numeric values, and contrast compliance across semantic token pairs.

### 5.2 Loaded Offscreen Audit (`theme-audit`)

`theme-audit` instantiates each screen declared by a theme package inside `ThemeHost`, feeds it synthetic libraries, and evaluates `kit-core`'s automated UI audits across viewport sizes and variants:

```bash
theme-audit [--gpu] [--state DIR] [--states LIST] [--sizes LIST] \
            [--crowd with|without|only] [--catalogue FILE] \
            [--section KEY] [--focus NAME] [--mockups DIR] <folder>
theme-audit --list-states
```

Exit codes follow Unix convention: `0` for no findings, `1` if one or more findings are emitted (`file:line kind words`), and `2` if the run failed to start.

Every rendered state is checked against seven automated rules from `kit-core::audit`:

- **Fit (`fit`):** Zero unintended clipping, overflow, or sibling overlap across the five canonical viewport sizes.
- **Gamepad Reach (`pad_reach`):** Every interactive control is reachable strictly via directional pad and shoulder navigation without focus traps.
- **Pointer Reach (`pointer_reach`):** Every interactive control exposes an unobstructed pointer hit area.
- **No Unintended Chrome (`no_chrome`):** No raw unstyled Qt widgets appear in theme surfaces.
- **Accessible Naming (`named`):** Every interactive or semantic element exposes a valid accessible name and role.
- **Contrast (`contrast`):** Rendered text and focus rings satisfy contrast thresholds against their backing surfaces.
- **Reduced-Motion Stillness:** Settled screens cease all animations when reduced motion is enabled.

### 5.3 The 15 Audit States

| State | Screens Checked | Scenario Covered |
|---|---|---|
| `default` | All 6 screens | Synthetic library with a suspended game; splash finished under reduced motion; Consoles tested normally and over 120 consoles (`--crowd`); Settings on Display; Game page on States tab. |
| `nogame` | `home` | No suspended or resumable game available. |
| `empty` | `home`, `consoles`, `games` | Empty library with zero consoles and zero games (`Games` shows the All shelf). |
| `one-console` | `home`, `consoles`, `games` | Single console with games, one suspended. |
| `one-game` | `games` | A console containing exactly one title. |
| `many` | `games` | Stress test: a console holding 9,000 games. |
| `nocover` | `home`, `consoles`, `games` | Games lacking cover artwork (verifies placeholders and text fallback). |
| `norecent` | `home`, `consoles`, `games` | No recently played or favorited titles. |
| `long` | `home`, `consoles`, `games` | Extra-long game title overflowing normal card widths, last played and suspended. |
| `pressed` | `home`, `consoles`, `games` | Accept button held down on the initially focused control. |
| `disabled` | `home` | Host-declined action (`scraper`) with focus walked to it. |
| `loud` | `home` | Navigation sound at full volume (exercises the longest settings status string). |
| `splash-1s` | `splash` | Splash screen sampled at `1000 ms`. |
| `splash-3s` | `splash` | Splash screen sampled at `3000 ms`. |
| `splash-5s` | `splash` | Splash screen sampled at `5000 ms`. |

---

## 6. ES-DE Theme Compatibility Engine

EmuSen renders EmulationStation Desktop Edition (ES-DE) XML theme packages natively through three layered crates that mirror the pure-Rust/thin-painter split of `kit-core` and `kit-qt`:

1. **`Frontend/esde` (Pure Rust, zero Qt):**
   - `loader`: Parses ES-DE `theme.xml`, `<include>` trees, variables, aspect ratios, color schemes, and view definitions (`system`, `gamelist`, `menu`) into resolved plain-data views.
   - `scene`: Evaluates element geometry, z-order, animation timelines, carousels, grids, textlists, badges, clocks, ratings, system status icons, and video slots into per-frame painter records.
   - `marks` & `shelves`: Provides built-in system marks and bridges EmuSen library metadata (`esde` system slugs declared in `Platform/systems/data/`) into ES-DE views.
   - `fonts`: Discovers ES-DE's default typeface when installed alongside theme-bundled `.ttf`/`.otf` fonts.
   - `check`: Runs headless theme corpus validation (`esde-check`) producing TSV compatibility tables tracked in `Testing/esde-corpus/`.
2. **`Frontend/esde-text` (Pure Rust text shaping):**
   - Built on `harfrust` and `read-fonts` to match ES-DE's exact text layout rules: whole-pixel size rounding for ES-DE's default face, exact theme sizes for custom fonts, whole-pixel pen advances rounded from unquantized `1/65536` pixel units, word breaks at spaces within fixed boxes, `lineSpacing` line pitch, whole-line vertical clipping, and `"..."` ellipsis truncation on single-line boxes.
3. **`Frontend/esde-qt` (`EmuSen.Esde` QML module):**
   - Renders the precomputed geometry, shaped text runs (`text.rs`, `hinted.rs`), animated GIFs (`gif.rs`), help prompts (`help.rs`), and lists (`lists.rs`) onto Qt Quick items without making independent layout decisions.

---

## 7. Previewing Components with `kit-gallery`

`Frontend/kit-gallery` builds the standalone `kit-gallery` binary for visual inspection and screenshot regression checks of every `EmuSen.Kit` control:

- **Vulkan RHI default:** Sets `QSG_RHI_BACKEND=vulkan` unless already set in the environment and configures `QQuickStyle` to `"EmuSen"` with `"Basic"` fallback.
- **Shared scene:** Loads the exact same `GALLERY_URL` QML scene evaluated headlessly by `emusen-qt-test`, ensuring interactive review matches automated tests.
- **Automated frame grab:** Running `kit-gallery --grab <file.png>` waits until the swapped frame matches the window's logical dimensions and device pixel ratio (or reaches `FRAMES`), writes the PNG, prints frame size, window size, and pixel ratio, and exits cleanly.

---

## Related Documentation

- [Overview and Architecture](Overview_and_Architecture.md) — Complete 4-layer architecture and repository map.
- [User Guide](User_Guide.md) — Navigating the 6 screens and switching themes in `emusen-qt`.
- [Platform C Interfaces](Platform_C_Interfaces.md) — `platform-client`, `crystal-client`, and the C headers behind `qt-models`.
- [Input and Controllers](Input_and_Controllers.md) — Pad glyph routing and `qt-endymion` focus navigation.
- [Testing and Verification](Testing_and_Verification.md) — `qt-test`, `Testing/esde-corpus/`, and compositor gates.
