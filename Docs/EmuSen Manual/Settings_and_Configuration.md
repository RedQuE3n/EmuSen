# Settings and Configuration

All EmuSen paths, configuration files, and SQLite databases are owned by **`Platform/galaxia`** (`tree.rs`, `atomic.rs`, `config_json.rs`, `config.rs`, `stores.rs`, `models.rs`). This reference documents the directory layout, file durability and JSONC rules, database locations, and every field in `appsettings.json`, `audio.json`, `graphics.json`, and `consoles.json`.

---

## Directory Layout and Root Discovery (`Platform/galaxia/src/tree.rs`)

### How EmuSen Finds Its Root

At startup, `tree::root_for()` determines `<root>` using this order:

1. **macOS App Bundle:** When the binary sits inside `*.app/Contents/MacOS`, `<root>` is `~/Library/Application Support/EmuSen`, and `Contents/Resources/home` serves as the read-only `Seed` directory.
2. **Marker Walk (up to 10 parents):** Starting from the program directory (passed via `tree::init()` or `current_exe().parent()`), Galaxia walks upward up to 10 levels and stops at the first directory containing a marker file (`.dianashroot`, legacy `.dianaosroot`, or the repository checkout root marker).
3. **Fallback:** On macOS, `~/Library/Application Support/EmuSen`. On Linux and Windows, `<program_dir>/DianaShRoot` (or `<program_dir>/DianaOSRoot` if only the legacy directory exists).

For testing and isolation, `Tree` supports four independent overrides (`Override::Config`, `Override::Data`, `Override::Legacy`, `Override::PowerSupply`) and per-thread `ThreadScope` redirects. Moving `Data` does not move `Config`, and redirecting `Config` without also setting `Legacy` blocks fallback to the player's real legacy config folder.

### Complete Galaxia Directory Tree (24 Entries)

Inside `dianash`, `<root>/home` is mounted as `/`. All 24 `Directory` entries defined in `tree.rs`:

| Directory | Default Path | Purpose |
|---|---|---|
| `Root` | `<root>` | Portable installation or checkout root |
| `Seed` | `Contents/Resources/home` *(macOS bundle only)* | Read-only bundle skeleton |
| `Home` | `<root>/home` | Player data root (`/` in `dianash`) |
| `Config` | `<root>/home/etc/EmuSen` | Active JSONC configuration files (`appsettings.json`, `audio.json`, `graphics.json`, `consoles.json`, `cheats/`, input bindings) |
| `ConfigPrevious` | `<root>/etc/EmuSen` | Previous pre-home config path checked during migration |
| `ConfigLegacy` | `<app_data>/EmuSen` (`$XDG_CONFIG_HOME/EmuSen` or `~/.config/EmuSen`) | Pre-Galaxia config location (copied non-destructively once if missing from `Config`) |
| `Logs` | `<root>/home/Logs` | Session logs, dumps, and coverage captures |
| `Saves` | `<root>/home/Saves` | Battery SRAM, memory cards, Controller Paks, and internal save media |
| `SaveStates` | `<root>/home/Saves/Save States` | Versioned binary save-state files |
| `Library` | `<root>/home/Library` | Game and state SQLite stores (`games.db`, `states.db`, `records.db`) |
| `Screenshots` | `<root>/home/Screenshots` | Captured frame screenshots (`F9` / `Guide + R1`) |
| `Artwork` | `<root>/home/Artwork` | Player-supplied custom game artwork |
| `Media` | `<root>/home/Media` | Scraped covers, screenshots, marquees, videos, and `media.db` |
| `Firmware` | `<root>/home/Firmware` | Optional player-supplied console BIOS/DSP images |
| `Shaders` | `<root>/home/Shaders` | Slang shader presets and SPIR-V cache |
| `Themes` | `<root>/home/Themes` | Installed EmuSen and ES-DE themes plus `themes.db` |
| `Cheats` | `<root>/home/Cheats` | Imported `.cht` files and cheat databases |
| `Games` | `<root>/home/Games` | Default ROM and disc image library folder |
| `Unpacked` | `<root>/home/Unpacked` | Temporary unpacked archives opened for play |
| `Cores` | `<root>/home/Cores` | Downloaded opt-in hosted Libretro cores |
| `Packages` | `<root>/home/Packages` | Per-game package directories (`pkg`) |
| `Compressed` | `<root>/home/Compressed` | Converted `.chd` disc images (`chd`) |
| `LegacyRoot` | `<root>/EmuSen.DianaSh/DianaSh/Usr/Home` | Historical pre-collapse home path for migration checks |
| `LogDefault` | `<app_data>/EmuSen/Logs` | Fallback error log root outside `Home` |

Shipped shell binaries and modules live outside the sandbox at `<program_dir>/dianash` (`shell_modules_directory()`).

---

## File Durability, JSONC Parsing, and Diagnostics

### `AtomicFile` (`src/atomic.rs`)

All configuration and save writes go through `atomic::write()`:

1. Parent directories are created automatically.
2. The complete payload is written to `<path>.tmp` beside the target file.
3. `<path>.tmp` is atomically renamed over `<path>`.

A concurrent reader or killed process always sees either the complete previous file or the complete new file—never a truncated half-write. Abandoned `.tmp` files are never read as live files. No physical device `fsync` is performed (the guarantee is process-crash atomicity, not power-loss durability).

### JSONC Rules and Error Handling (`src/config_json.rs`, `src/config.rs`, `Platform/suggest`)

- **Allowed syntax:** Standard JSON plus single-line `//` comments, block `/* ... */` comments, and trailing commas:
  ```jsonc
  {
    // Hand-edited comments and trailing commas are accepted
    "SampleRate": 48000,
    "MasterVolume": 0.85,
  }
  ```
- **Refused syntax:** Unquoted keys, single-quoted strings, hexadecimal numbers, unary `+`, bare decimal points (`.5`), `NaN`/`Infinity`, missing commas, and empty or comment-only files are rejected.
- **Case-insensitive keys:** Struct property names are matched case-insensitively at every depth; if a key appears twice, the last occurrence wins. Unknown properties are ignored on read and omitted when written back.
- **Formatting:** Files are written pretty-printed with 2-space indentation and LF line feeds.
- **Best-effort load:** A missing file or a literal `null` document silently loads defaults. Malformed syntax or wrong field types report a JSON-path diagnostic (for example `$.SampleRate: invalid type: string "fast", expected i32. Falling back to defaults.`) and continue with defaults.
- **"Did you mean" suggestions:** `Platform/suggest` computes case-insensitive optimal string alignment (edit tolerance: `1` for ≤3 chars, `2` for 4–6 chars, `3` for ≥7 chars) to suggest close matches for mistyped setting or command names.

---

## SQLite Databases (`Platform/galaxia/src/stores.rs`)

| File | Location | Owner & Contents |
|---|---|---|
| `games.db` | `home/Library/games.db` | Discovered games, ROM hashes, system assignments, play time, favourites, and collections |
| `states.db` | `home/Library/states.db` | Save-state slot metadata and thumbnail records written by `Platform/crystal` |
| `records.db` | `home/Library/records.db` | Historical file/fetch records migrated once into `states.db` |
| `media.db` | `home/Media/media.db` | Scraped media index and asset metadata (`Platform/scrape`) |
| `themes.db` | `home/Themes/themes.db` | Installed theme catalog and metadata |

---

## `appsettings.json` (`AppSettings`)

Stored at `home/etc/EmuSen/appsettings.json`. All 73 active fields from `Platform/galaxia/src/models.rs` are listed below (the retired `OnlineCovers` key is skipped on write whenever `null`):

### Paths and Library Selection

| Key | Type | Default | Meaning |
|---|---|---|---|
| `LogDirectory` | `string \| null` | `null` | Custom log output directory (`null` uses `home/Logs`) |
| `RomDirectory` | `string \| null` | `null` | Custom ROM library folder (`null` uses `home/Games`) |
| `StateDirectory` | `string \| null` | `null` | Custom save-state directory (`null` uses `home/Saves/Save States`) |
| `CheatDatabaseDirectory` | `string \| null` | `null` | Custom cheat database folder (`null` uses `home/Cheats`) |
| `ArtworkDirectory` | `string \| null` | `null` | Custom artwork folder (`null` uses `home/Artwork`) |
| `FirmwareDirectory` | `string \| null` | `null` | Optional external firmware folder searched before `home/Firmware` |
| `EsdeMediaDirectory` | `string \| null` | `null` | Optional external ES-DE media folder for cover/media lookup |
| `ChdOutputDirectory` | `string \| null` | `null` | Custom output directory for CHD conversion (`null` uses `home/Compressed`) |
| `SelectedCore` | `string \| null` | `"All consoles"` | Active console filter in the library |
| `SelectedCoreUpgraded` | `bool` | `false` (`true` after load) | One-time migration upgrading legacy default `"SNES (Venus)"` to `"All consoles"` |
| `LibrarySearch` | `string` | `""` | Saved library search query |
| `CheatSearch` | `string` | `""` | Saved cheat search query |
| `LibraryView` | `string` | `"Grid"` | Library layout mode (`"Grid"` or list) |
| `LibraryTileScale` | `f64` | `1.0` | Cover tile zoom factor in Grid view |
| `LibraryCollection` | `string` | `"all"` | Active library collection filter |
| `ShowHiddenGames` | `bool` | `false` | Show hidden games in the library |
| `MaxPlayTimeTracking` | `i32` | `8` | Maximum continuous hours recorded per session |

### Controllers, Input, and Session Behavior

| Key | Type | Default | Meaning |
|---|---|---|---|
| `MirrorPlayer1ToPlayer2` | `bool` | `false` | Mirror Player 1 inputs onto Player 2 |
| `KeyboardPlayer` | `i32` | `1` | Player seat driven by keyboard controls |
| `AnalogStickAsDpad` | `bool` | `true` | Map left analog stick onto D-pad navigation/controls |
| `StickDeadzone` | `f64` | `0.5` | Analog stick dead zone threshold (`0.0`–`1.0`) |
| `ControllerType` | `string` | `"Automatic"` | Controller glyph family (`"Automatic"` or explicit family) |
| `SwapPadButtons` | `bool` | `false` | Swap Accept/Back (South/East) and Search/Screensaver (North/West) |
| `FirstControllerOnly` | `bool` | `false` | Restrict UI navigation to the first controller |
| `ControllerNotifications` | `bool` | `true` | Show controller connect/disconnect notifications |
| `GuideChords` | `string` | `"Auto"` | `"Auto"` (on desktop, off in Steam Game Mode), `"On"`, or `"Off"` |
| `ClaimPlayersByPress` | `bool` | `true` | Assign controller seats dynamically when buttons are pressed |
| `ExternalControllerFirst` | `bool` | `true` | Prioritize external gamepads ahead of built-in handheld controls |
| `Rumble` | `bool` | `true` | Enable controller vibration/rumble motors |
| `OnScreenKeyboard` | `string` | `"Automatic"` | On-screen keyboard mode |
| `PauseInBackground` | `bool` | `true` | Automatically pause emulation when window loses focus |
| `ResumeOnLaunch` | `string` | `"Ask"` | Behavior when launching a game with an existing save state (`"Ask"`, etc.) |

### Interface, Themes, and Audio UI

| Key | Type | Default | Meaning |
|---|---|---|---|
| `BigScreen` | `bool` | `false` | Big-screen layout scaling flag |
| `ShowStatusBar` | `bool` | `true` | Show overlay bar background behind status/FPS text |
| `ShowStatusText` | `bool` | `true` | Show on-screen status messages over gameplay |
| `ShowFpsBar` | `bool` | `true` | Show frame-rate counter over gameplay |
| `Appearance` | `string` | `"dark"` | Theme color variant (`"dark"` or `"light"`) |
| `HomeArt` | `bool` | `true` | Show background game artwork on Home |
| `PlayVideos` | `bool` | `true` | Play preview videos in library views |
| `TextSize` | `string` | `"default"` | Interface font size preset |
| `ReducedMotion` | `string` | `"system"` | `"on"`, `"off"`, or `"system"` |
| `StartupSplash` | `bool` | `true` | Play startup Moonrise splash animation |
| `LibraryStyle` | `string` | `"Theme"` | Library presentation mode (`"Theme"` enables EmuSen/ES-DE themes) |
| `BigPictureTheme` | `string \| null` | `null` | Selected ES-DE theme directory name |
| `EmuSenTheme` | `string \| null` | `null` | Selected native EmuSen theme identifier |
| `NavigationSounds` | `bool` | `true` | Play UI navigation sound cues |
| `SoundOutputDevice` | `string` | `"System default"` | Named SDL3 audio output device |
| `Volume` | `f64` | `1.0` | Frontend master volume (`0.0`–`1.0`) |
| `Mute` | `bool` | `false` | Mute frontend audio |

### Scraping (`Scrape*`)

| Key | Type | Default | Meaning |
|---|---|---|---|
| `OpenEmuFallback` | `bool` | `true` | Allow OpenEmu cover fallback lookup |
| `Scraping` | `bool` | `true` | Master enable for artwork/metadata scraping |
| `ScrapeCovers` | `bool` | `true` | Download box front covers |
| `ScrapeScreenshots` | `bool` | `true` | Download gameplay screenshots |
| `ScrapeMarquees` | `bool` | `true` | Download game logos/marquees |
| `ScrapeTitleScreens` | `bool` | `true` | Download title screens |
| `ScrapeMiximages` | `bool` | `true` | Download composite miximages |
| `ScrapeBackCovers` | `bool` | `true` | Download box back covers |
| `Scrape3DBoxes` | `bool` | `true` | Download 3D box renders |
| `ScrapePhysicalMedia` | `bool` | `true` | Download cartridge/disc media images |
| `ScrapeFanArt` | `bool` | `true` | Download fan-art backgrounds |
| `ScrapeManuals` | `bool` | `false` | Download PDF game manuals |
| `ScrapeVideos` | `bool` | `false` | Download video previews |
| `ScrapeGameNames` | `bool` | `false` | Overwrite display names with scraped titles |
| `ScrapeRegion` | `string` | `"auto"` | Preferred region (`"auto"`, `"us"`, `"eu"`, `"jp"`, etc.) |
| `ScrapeLanguage` | `string` | `"en"` | Preferred metadata language |
| `ScrapeRegionFallback` | `bool` | `true` | Fall back to other regions if preferred asset is missing |
| `ScrapeThreads` | `i32` | `0` | Concurrent scraper workers (`0` = automatic) |
| `ScrapeCriteria` | `string` | `"nocover"` | Default game selection filter for scraping |
| `ScrapeRefresh` | `bool` | `false` | Re-scrape games that already have media |
| `ScrapeKindsUpgraded` | `bool` | `false` (`true` after load) | One-time migration expanding the legacy 4-kind default to all 9 picture kinds |

### Nested Big Picture & ES-DE Theme Objects

- **`BigPicture`** (`map<string, BigPictureChoices>`, default `{}`): Per-ES-DE-theme overrides where every field is `string | null` defaulting to `null` (Automatic / theme default): `Variant`, `ColorScheme`, `FontSize`, `AspectRatio`, `Language`, `Transitions`.

**`BigPictureCollections`** (nested object in `appsettings.json`):

| Key | Type | Default |
|---|---|---|
| `AutoCollections` | `list<string>` | `["all", "favorites", "recent"]` |
| `HiddenCustomCollections` | `list<i64>` | `[]` |
| `GroupCustomCollections` | `string` | `"unthemed"` |
| `FavoritesFirst` / `FavoritesFirstCustom` | `bool` | `true` / `false` |
| `StarsCustom` / `FoldersOnTop` | `bool` | `false` / `true` |
| `DefaultSortOrder` | `string` | `"name, ascending"` |
| `RandomEntryButton` | `string` | `"games"` |
| `FlattenedSystems` | `list<string>` | `[]` |

**`BigPictureInterface`** (nested object in `appsettings.json`):

| Group | Keys and Defaults |
|---|---|
| **Navigation & Startup** | `LaunchScreenDuration` (`"normal"`), `MenuOpeningEffect` (`"scale-up"`), `QuickSystemSelect` (`"leftrightshoulders"`), `StartupSystem` (`""`), `StartupView` (`"system"`), `SystemsSorting` (`"release"`), `ListScrollOverlay` (`false`), `NavigationVolume` (`60`) |
| **Status & Help Bar** | `DisplayClock` (`false`), `DisplayHelp` (`true`), `StatusBluetooth` (`true`), `StatusWifi` (`true`), `StatusBattery` (`true`), `StatusBatteryPercentage` (`true`) |
| **Screensaver** | `ScreensaverTimer` (`300000` ms), `ScreensaverType` (`"dim"`), `ScreensaverControls` (`true`), `ScreensaverSwapImageTimeout` (`10000` ms), `ScreensaverSlideshowOnlyFavorites` (`false`), `ScreensaverStretchImages` (`false`), `ScreensaverSlideshowGameInfo` (`true`), `ScreensaverSlideshowCustomImages` (`false`), `ScreensaverSlideshowRecurse` (`false`), `ScreensaverSlideshowCustomDir` (`""`), `ScreensaverInGameMode` (`true`) |
| **Media Viewer** | `MediaViewerHelpPrompts` (`"top"`), `MediaViewerShowTypes` (`false`), `MediaViewerKeepVideoRunning` (`true`), `MediaViewerStretchVideos` (`false`), `MediaViewerVideoBlur` (`false`), `MediaViewerVideoAudio` (`true`) |

---

## `audio.json` (`AudioConfig`)

Stored at `home/etc/EmuSen/audio.json`. Controls `Platform/endymion`'s SDL3 audio pipeline:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `SampleRate` | `i32` | `32000` | Initial/default output sample rate in Hz (sessions dynamically reopen when a core changes rate) |
| `AudioBufferMaxSamples` | `i32` | `128000` | Maximum queued sample buffer size |
| `OutputTargetLatencyMs` | `i32` | `256` | Target output ring latency in milliseconds for dynamic rate control |
| `RateControlMaxDeviation` | `f64` | `0.005` | Maximum fractional pitch/rate adjustment (`±0.5%`) to keep audio locked to video |
| `MasterVolume` | `f32` | `1.0` | Audio engine output gain (`f32`; non-finite overflow values are rejected) |
| `Muted` | `bool` | `false` | Mute audio engine output |
| `AudioEnabled` | `bool` | `true` | Enable or disable audio stream creation |

---

## `graphics.json` (`GraphicsConfig`)

Stored at `home/etc/EmuSen/graphics.json`. Controls window geometry, presentation sync, and per-console shader state:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `WindowWidth` | `i32` | `1060` | Windowed-mode width in pixels |
| `WindowHeight` | `i32` | `580` | Windowed-mode height in pixels |
| `TargetFps` | `i32` | `60` | Target frame pacing rate |
| `VSyncEnabled` | `bool` | `true` | Enable vertical sync presentation |
| `WindowResizable` | `bool` | `true` | Allow resizing when running in windowed mode |
| `BilinearFiltering` | `bool` | `true` | Linear vs nearest-neighbor texture sampling |
| `SyncToDisplay` | `bool` | `true` | Pace emulated frames to display refresh rate |
| `Consoles` | `map<string, map<string, string \| null>>` | `{}` | Per-console graphics/filter settings (legacy core settings here also seed `consoles.json` once if absent) |
| `ShaderParameters` | `map<string, map<string, map<string, string \| null>>>` | `{}` | Saved slang shader parameters keyed by `[console][shader][param]` |
| `RecentShaders` | `map<string, list<string \| null>>` | `{}` | Recently selected shader presets per console |

---

## `consoles.json` (`ConsolesConfig`)

Stored at `home/etc/EmuSen/consoles.json` (`Platform/galaxia/src/consoles.rs`). Holds per-console core settings as key/value string pairs:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `Consoles` | `map<string, map<string, string \| null>>` | `{}` | Map from console name to core setting key/value pairs (validated against each core's ABI v1 setting schema) |

On first load when `consoles.json` does not exist yet, Galaxia copies any existing `Consoles` entries from `graphics.json` into `consoles.json` once without overwriting an existing `consoles.json`.

---

## Additional Categorized Configuration Files

- **Per-game cheat files (`home/etc/EmuSen/cheats/<game>.json`):** Use `CheatFile` (`Cheats` array of `CheatFileEntry` objects with `Kind` `"RamPoke"` / `"RomPatch"`, `Writes` array of `CheatFileWrite` items specifying `Space`, `Address`, `Value`, `Width`, `Type`, `BitPosition`, `BigEndian`, `RepeatCount`, `RepeatAddAddress`, `RepeatAddValue`, plus `Compare`, `Description`, and `Enabled`).
- **Controller and hotkey bindings (`home/etc/EmuSen/`):** `keybindings.json`, `gamepadbindings.json`, and `gamepadprofiles.json`, owned by `Platform/endymion/src/bindings.rs` (see [Input_and_Controllers.md](Input_and_Controllers.md)).
