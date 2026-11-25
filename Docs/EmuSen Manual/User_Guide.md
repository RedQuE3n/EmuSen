# User Guide

This guide covers everyday play in EmuSen using **`emusen-qt`** (the unified six-screen frontend) and **`emusen-qt-play`** (the direct game launcher). For complete configuration file keys see [Settings_and_Configuration.md](Settings_and_Configuration.md); for deeper controller, shader, and save-media details see [Input_and_Controllers.md](Input_and_Controllers.md), [Audio_Video_CRT_and_Shaders.md](Audio_Video_CRT_and_Shaders.md), and [Save_Media_and_Game_Packages.md](Save_Media_and_Game_Packages.md).

---

## Launching EmuSen

- **Main frontend (`emusen-qt`):** Opens full screen on Vulkan (`QSG_RHI_BACKEND=vulkan`, threaded render loop), plays the optional startup splash (`StartupSplash` in `appsettings.json`), and lands on **Home**.
  ```bash
  emusen-qt --platform-library /path/to/libemusen_platform.so
  ```
  Pass `--theme /path/to/theme-dir` (or set `EMUSEN_THEME`) to override the active EmuSen theme for that run.
- **Direct game launcher (`emusen-qt-play`):** Opens a single window and boots a ROM or disc image immediately without loading the library screens:
  ```bash
  EMUSEN_PLATFORM_LIBRARY=/path/to/libemusen_platform.so \
  EMUSEN_CORES=/path/to/cores \
  emusen-qt-play --game /path/to/game.sfc
  ```
  Set `EMUSEN_ENGINE=<engine>` to override the default core for a single launch, or `EMUSEN_GPU_DEVICE` to pick a specific Vulkan device on multi-GPU systems.

---

## The Six Unified Screens

`emusen-qt` organizes every player workflow into six themeable screens (implemented in full by the `Eclipse` dark theme, with `Home`, `Consoles`, and `Games` also provided by the reference `EmuSen` theme and ES-DE themes):

| Screen | Purpose |
|---|---|
| **Splash** | Startup transition (Moonrise). Skipped when `StartupSplash` is `false`, or collapsed to its final still frame when `ReducedMotion` is `"on"`. |
| **Home** | Hub showing recently played games, quick resume for a suspended session, and direct access to Consoles, Library tools (**Cores**, **Themes**), and **Settings**. |
| **Consoles** | System shelf listing every console with discovered games or installed engines. |
| **Games** | Browseable library in `Grid` or list layout with collections (`all`, `favorites`, `recent`, and custom collections), search, sorting, and cover artwork. |
| **Game** | Per-game detail view showing metadata, artwork, save slots, cheats, per-game engine/controller overrides, and the live gameplay surface with the Pause Card overlay. |
| **Settings** | Hierarchical configuration for Library, Display & Shaders, Sound, Controllers & Hotkeys, Themes, Firmware, Scraping, and Accessibility. |

---

## Controller and Keyboard Grammar

Every screen shares one consistent navigation grammar (`Frontend/kit-core`), with automatic button-glyph switching (`ControllerType: "Automatic"`) for the connected gamepad family.

### Menu Navigation

| Action | Gamepad (Default) | Gamepad (`SwapPadButtons`) | Keyboard |
|---|---|---|---|
| Move focus | D-pad / Left stick | D-pad / Left stick | Arrow keys |
| **Accept** | South (A / Cross) | East (B / Circle) | `Enter` |
| **Back** | East (B / Circle) | South (A / Cross) | `Escape` |
| **Menu** (open Home/menu card) | Start | Start | Menu key |
| **Options** (List card / favourite) | Select | Select | `Tab` |
| **Page / Section previous & next** | L1 / R1 | L1 / R1 | `PageUp` / `PageDown` |
| **Jump to first / last entry** | L2 / R2 | L2 / R2 | `Home` / `End` |
| **Search** | North (Y / Triangle) | West (X / Square) | — |
| **Screensaver** | West (X / Square) | North (Y / Triangle) | — |
| **Random entry** | L3 or R3 | L3 or R3 | — |

Directional presses and shoulders repeat cleanly while held. `Accept`, `Back`, and action buttons never auto-repeat so a held press cannot accidentally confirm the next dialog.

### In-Game Controls, Hotkeys, and Guide Chords

During play, controller input routes directly to the emulated console using RetroPad bindings (`Platform/endymion`). EmuSen intercepts the pause chord and keyboard hotkeys:

- **Open Pause Card:** Press **`Guide`** alone, hold **`Start + Select`** together, or press **`Escape`** (Exit to Library / Pause).
- **Guide Chords (when enabled):** Hold **`Guide`** and press:
  - **`Guide + Up`** — Save state to the active slot
  - **`Guide + Down`** — Load state from the active slot
  - **`Guide + Right Shoulder`** — Take screenshot (`home/Screenshots/`)
  - `GuideChords` in `appsettings.json` defaults to `"Auto"`, which enables chords on standard desktops and disables them automatically under Steam Game Mode (`gamescope` / `SteamDeck=1`) so Steam keeps its own Guide shortcuts.

Default keyboard hotkeys (customizable in **Settings → Controllers → Hotkeys**):

| Action | Default Key | Behavior |
|---|---|---|
| Fast Forward | `Tab` | Run at turbo speed while held |
| Rewind | `Backspace` | Step backward through the rewind ring while held |
| Save State | `F5` | Save to the active state slot |
| Load State | `F8` | Load from the active state slot |
| Toggle Pause | `P` | Pause or resume emulation |
| Frame Advance | `.` (Period) | Advance one frame while paused |
| Toggle Fullscreen | `F11` | Switch between fullscreen and windowed mode |
| Screenshot | `F9` | Capture current frame to `home/Screenshots/` |
| Toggle Big Picture | `F10` | Toggle Big Picture layout mode |
| Exit to Library | `Escape` | Suspend game and open the Pause Card / library |

---

## Setting Up Your ROM Library

1. Place game files in `home/Games/` (the default Galaxia games directory), or point **`RomDirectory`** in **Settings → Library** (`appsettings.json`) to an existing folder of ROMs, disc images (`.cue`/`.bin`, `.iso`, `.chd`), or game packages.
2. EmuSen scans the library folder, hashes images, matches systems from `Platform/systems/data/*.json`, and stores records in `home/Library/games.db`.
3. Optional artwork and metadata can be fetched via the built-in ScreenScraper integration (configured through the `Scrape*` settings in `appsettings.json` or `scrape(1)` in DianaSh) and saved under `home/Media/` (`media.db`), or loaded from an existing `EsdeMediaDirectory`.
4. Multi-track CD/DVD images can be compressed to `.chd` with `chd(1)` and stored under `home/Compressed/`. ROM patches (IPS/BPS/UPS) can be bundled into game packages under `home/Packages/` with isolated saves so patched runs never overwrite unpatched save files (see [Save_Media_and_Game_Packages.md](Save_Media_and_Game_Packages.md)).

---

## Playing, Pausing, Save States, Rewind, and Speed

Opening the **Pause Card** over a running game suspends emulation cleanly and offers:

- **Resume** — Return immediately to the live frame.
- **Rewind (Moments reel)** — Browse recent snapshots visually and jump back to any captured moment.
- **Save State / Load State** — Step left/right across state slots and save or restore with thumbnail previews. When launching a game with saved progress, `ResumeOnLaunch` (default `"Ask"`) asks whether to resume or start fresh.
- **Speed** — Switch between normal speed (`100%`), slow motion (default `25%`), turbo fast-forward (default `300%`), or unthrottled (`0%`).
- **Change Disc** *(for multi-disc games)* or **Swap Players 1 and 2** *(when a second controller is connected)*.
- **Game Options** — Open per-session **Cheats** (shown whenever cheats exist for the active game), **Controllers**, **Display** (CRT/shaders), and **Sound**.
- **Leave** — **Quit Game** (returning to the library) or **Quit EmuSen**.

### How Rewind and Moments Work

`Platform/crystal` captures compressed XOR-delta snapshots on a background worker thread (default cadence: every 4 frames; default memory budgets: 96 MiB for delta history and 32 MiB for Moments thumbnails). Once the budget fills, buffers are reused without extra allocations. Rewinding restores machine state without modifying shared removable memory cards.

### Save Media and Memory Cards

EmuSen distinguishes three save categories under `home/Saves/`:

- **Per-game battery SRAM** (cartridge RAM and flash saves tied to one game)
- **Removable memory cards and paks** (PS1 memory cards, PS2 memory cards, N64 Controller Paks, Sega CD backup RAM carts—manageable via `memcard(1)` in DianaSh)
- **Console internal memory** (such as Sega CD internal BRAM)

---

## Display, CRT Filter, and Accessibility

- **Modelled CRT & Shaders:** Under **Settings → Display** (or **Pause Card → Game Options → Display**), select built-in filters—including the physically modelled CRT shader in `Accurate`, `Balanced`, or `Performance` tiers—or load RetroArch slang shader presets from `home/Shaders/`.
- **Accessibility options (`appsettings.json`):**
  - `ReducedMotion`: Set `"on"` to disable animations and collapse the startup splash to a still frame.
  - `TextSize`: Adjust interface text scaling across screens.
  - `SwapPadButtons`: Swap Accept/Back (South/East) and Search/Screensaver (North/West) for Nintendo-style face-button layouts.
  - `NavigationSounds`: Toggle UI navigation audio cues.
  - `ShowStatusText` / `ShowFpsBar` / `ShowStatusBar`: Toggle on-screen gameplay status and frame-rate overlays.

---

## Cheats

EmuSen supports core-agnostic **RAM pokes** (applied every frame) and **ROM patches** (applied to read-only memory, with an optional compare byte so banked code is patched safely).

- **Formats:** RetroArch `.cht` files and EmuSen JSON cheat files stored under `home/etc/EmuSen/cheats/`.
- **In-game toggling:** Open **Pause Card → Game Options → Cheats** to toggle individual cheats during play.
- **Database and CLI management:** Use `cheat(1)` in `dianash` to add codes, import/export `.cht` files, search the database (`cheat db find`), and load or prune entries.

---

## Firmware and Built-In Open Replacements

EmuSen **never bundles copyrighted console firmware**. Every shipping core either requires no BIOS or ships with a clean-room open replacement so games boot out of the box:

| Core / System | Built-In Clean-Room Replacement | Optional Authentic Image |
|---|---|---|
| **VenusRT (SNES)** | Open HLE for DSP-1, DSP-1B, DSP-2, DSP-4, and ST010 | Authentic µPD77C25 / µPD96050 ROMs for LLE |
| **MarsRT (N64)** | Open PIF/CIC boot handoff | — |
| **Endou (SMS / Game Gear)** | Open Master System & Game Gear boot ROMs | Regional SMS / GG BIOS ROMs |
| **Nephrite (Sega CD)** | Open Sega CD BIOS and BRAM formatting | Regional Sega CD / Mega-CD BIOS |
| **Diamond (PlayStation)** | Open BIOS Kernel and empty-drive shell | Authentic PlayStation BIOS image |
| **Sapphire (PlayStation 2)** | ELF fast-boot | Authentic PlayStation 2 BIOS image |

To supply your own authentic firmware dumps:

1. Copy the files into `home/Firmware/` (or set `FirmwareDirectory` in `appsettings.json` to an external folder).
2. `Platform/firmware` searches `FirmwareDirectory` first, then `home/Firmware/`. Within each folder it checks the canonical filename, known alternate filenames, multi-part split forms, and finally any file of the expected byte size whose MD5 digest matches the core's firmware declaration.
3. Choose between open replacements and detected images in **Settings** or per-console core options.

---

## Opt-In Hosted Libretro Cores

On Linux, EmuSen can host ten allowlisted Libretro cores (`Platform/systems/data/libretro-cores.tsv`) in Landlock-confined child processes (`emusen-core-runner`): `pcee2` (PS2), `flycast` (Dreamcast), `snes9x` (SNES), `nestopia` (NES), `genesis_plus_gx` (Genesis/Sega CD/SMS/SG-1000), `picodrive` (32X), `mupen64plus_next` (N64), `sameboy` (GB/GBC), `mednafen_psx_hw` (PS1), and `mednafen_saturn` (Saturn).

- **Opt-in only:** No hosted core is bundled. Open the **Cores** screen from the frontend—or run `cores get <stem>` in `dianash`—to download a core on demand. Every download is checked against its pinned SHA-256 digest before activation.
- **Defaults and overrides:** Native Rust cores are the default engine on every system except PlayStation 2, where `pcee2` is the default while Sapphire continues to mature. Switch the default engine per system or per game in **Cores** or with `cores default`.
