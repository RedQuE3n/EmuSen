# Input and Controllers

EmuSen manages physical gamepads, keyboards, emulator hotkeys, multiplayer seats, procedural controller glyphs, and force feedback across four coordinated layers: **`Platform/endymion`** (SDL3 gamepad I/O, JSONC binding files, `PlayerSlots`, and `PortRouter`), **`Platform/crystal`** (core input delivery and rumble gating), **`Frontend/kit-core`** (vector button glyphs and UI focus walks), and **`Frontend/qt-endymion`** (Qt-side polling, seat notifications, and hotkey dispatch).

For configuration file locations and persisted setting keys see [Settings and Configuration](Settings_and_Configuration.md). For Core ABI v1 input exports see [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md).

---

## 1. Unified RetroPad Button and Axis Contract

Rather than leaking console-specific controller layouts into frontends or platform services, Endymion and Core ABI v1 normalize every input device onto one canonical **RetroPad** vocabulary (`Platform/endymion/src/vocabulary.rs` and `emusen_core.h`).

### 1.1 Canonical Digital Controls (`emusen_control` / `PadButton`)

| Index | Control | Positional Placement |
|---:|---|---|
| `0` | `B` | South face button |
| `1` | `Y` | West face button |
| `2` | `SELECT` | Back / View / Minus / Select |
| `3` | `START` | Start / Menu / Plus / Options |
| `4` | `UP` | Directional pad up |
| `5` | `DOWN` | Directional pad down |
| `6` | `LEFT` | Directional pad left |
| `7` | `RIGHT` | Directional pad right |
| `8` | `A` | East face button |
| `9` | `X` | North face button |
| `10` | `L` | Left shoulder (`L1` / `LB`) |
| `11` | `R` | Right shoulder (`R1` / `RB`) |
| `12` | `L2` | Left trigger digital / `L2` |
| `13` | `R2` | Right trigger digital / `R2` |
| `14` | `L3` | Left stick click |
| `15` | `R3` | Right stick click |

### 1.2 Canonical Analog Axes (`emusen_axis` / `PadAxis`)

Six analog axes cross the ABI in fixed order:

- **Left stick X / Y** (`0`, `1`): `-1.0 .. 1.0`, with right and down positive.
- **Right stick X / Y** (`2`, `3`): `-1.0 .. 1.0`, with right and down positive.
- **Left trigger / Right trigger** (`4`, `5`): `0.0 .. 1.0`.

Each core declares its active ports, digital controls, and analog axes in machine info JSON, and Endymion routes player inputs strictly to the ports and axes the running machine accepts.

---

## 2. SDL3 Hotplug and Virtual Controllers

`Platform/endymion/src/pads.rs`, `pad.rs`, and `sdl.rs` drive physical controllers directly through **SDL3**'s gamepad subsystem:

- **Live hotplug and replug:** SDL3 connection and disconnection events update the active controller roster immediately without restarting `emusen-qt` or interrupting gameplay. Each opened controller is tracked by its SDL3 GUID and OS device path so replugging a controller restores its seat cleanly.
- **Connection notifications (`ControllerNotifications`):** Enabled by default (`true`), emitting short arrival/departure notices (`qt-endymion/src/seats.rs` `Came { name, player, connected }`) whenever a pad joins or leaves a seat.
- **In-process virtual gamepads:** Endymion exposes simulated gamepads (`emusen_endymion_sim_pad_*` in `emusen_platform.h`) used by headless tests and compositor gates (`Testing/qt-gates/pad.py`) via a private FIFO (`EMUSEN_GATE_PAD`), avoiding host-wide `uinput` devices that could trigger external desktop or launcher shortcuts.

---

## 3. Per-Console Bindings and Named Controller Profiles

Endymion reads and writes **four JSONC binding files** (`Platform/endymion/src/bindings.rs`) using human-readable, case-insensitive control names with automatic "did you mean" suggestions (`emusen_suggest`) on typos:

| Binding File | Enum Variant | Scope & Contents |
|---|---|---|
| `keybindings.json` | `File::Keys` | Keyboard-to-RetroPad and stick-direction mappings per console or global default. |
| `gamepadbindings.json` | `File::Gamepad` | SDL3 gamepad-to-RetroPad mappings per console and per player (`Player(1..=8)`), grouped under named profiles. |
| `gamepadprofiles.json` | `File::Profiles` | Named profile definitions (`DEFAULT_PROFILE = "Default"`) plus per-console and per-game profile selections. |
| `hotkeybindings.json` | `File::Hotkeys` | Keyboard shortcuts bound to the 10 emulator actions. |

### 3.1 Built-In Defaults (`bindings.rs`)

Bindings are mapped by **physical button position** rather than vendor-printed labels so layouts remain consistent across Nintendo, Xbox, PlayStation, and generic pads. When upgrading older binding files, `add_new_defaults` inserts newly added controls (from index `12` onward: `L2`, `R2`, `L3`, `R3`, stick directions) only if the default key/button is not already claimed by the player.

| RetroPad Control | Default Keyboard Key (`KEY_DEFAULTS`) | Default Gamepad Input (`PAD_DEFAULTS`) |
|---|---|---|
| `Up` / `Down` / `Left` / `Right` | `Up` / `Down` / `Left` / `Right` | `DPadUp` / `DPadDown` / `DPadLeft` / `DPadRight` |
| `B` / `A` / `Y` / `X` | `Z` / `X` / `A` / `S` | `South` / `East` / `West` / `North` |
| `L` / `R` | `Q` / `W` | `LeftShoulder` / `RightShoulder` |
| `Start` / `Select` | `Return` / `RightShift` | `Start` / `Back` |
| `L2` / `R2` / `L3` / `R3` | `E` / `R` / `C` / `V` | Triggers / `LeftStick` / `RightStick` |
| Left Stick (`Up`/`Down`/`Left`/`Right`) | `I` / `K` / `J` / `L` | Physical left stick axes |
| Right Stick (`Up`/`Down`/`Left`/`Right`) | `T` / `G` / `F` / `H` | Physical right stick axes |

### 3.2 Profile Resolution Order

When a session opens, `bindings::resolve(profiles, console, game)` selects the active controller profile in priority order:

1. Per-game override in `gamepadprofiles.json` (keyed by canonical game path key).
2. Per-console profile assignment in `gamepadprofiles.json`.
3. Built-in `"Default"` profile (`pad_defaults()`).

### 3.3 Emulator Hotkeys (`qt-endymion/src/hotkeys.rs`)

| Hotkey Action | Default Key (`HOTKEY_DEFAULTS`) | Trigger Mode |
|---|---|---|
| `FastForward` | `Tab` | Held (active while key is held down) |
| `Rewind` | `Back` (Backspace) | Held (active while key is held down) |
| `SaveState` | `F5` | Edge-triggered |
| `LoadState` | `F8` | Edge-triggered |
| `TogglePause` | `P` | Edge-triggered |
| `ToggleFullscreen` | `F11` | Edge-triggered |
| `ExitToLibrary` | `Escape` | Edge-triggered |
| `Screenshot` | `F9` | Edge-triggered |
| `ToggleBigPicture` | `F10` | Edge-triggered |
| `FrameAdvance` | `OemPeriod` (`.`) | Edge-triggered |

---

## 4. Local Multiplayer Seats, Claiming, and Mirroring

EmuSen manages up to **8 local player seats** (`MAX_PLAYERS = 8` in `Platform/endymion/src/slots.rs`):

### 4.1 Seat Assignment and Reservation (`PlayerSlots`)

- Disconnecting a controller leaves its seat **reserved** with its SDL GUID and device path rather than collapsing higher player numbers downward.
- Reconnecting or adding a controller places it via `PlayerSlots::seat()` in priority order:
  1. Its reserved seat matching **both** GUID and OS device path.
  2. Otherwise, the lowest reserved seat matching its GUID.
  3. Otherwise, the lowest unconnected seat (empty or vacated).
- In the frontend (`qt-endymion/src/seats.rs`), players can inspect live held buttons/axes per connected pad (`Connected`), swap seats between players (`Ask::Move { key, player }`), or release a reserved disconnected seat (`Ask::Forget { player }`).

### 4.2 Multiplayer Behavior Settings (`appsettings.json`)

| Setting | Default | Behavior |
|---|---|---|
| `ClaimPlayersByPress` | `true` | Allows connected pads to claim seats dynamically by pressing buttons when entering or joining play. |
| `ExternalControllerFirst` | `true` | On handheld PCs with integrated controls, gives Player 1 preference to an external gamepad when connected. |
| `FirstControllerOnly` | `false` | Restricts UI menu navigation to Player 1's controller so idle secondary pads cannot navigate screens. |
| `KeyboardPlayer` | `1` | 1-based player seat controlled by keyboard bindings (`PortRouter.keyboard_player`). |
| `MirrorPlayer1ToPlayer2` | `false` | Mirrors Player 1's inputs simultaneously onto Port 2 (`PortRouter.mirror_player1_to_player2`) for testing or accessibility. |

### 4.3 Delta Routing (`PortRouter`)

`Platform/endymion/src/router.rs` caches the last button and axis state delivered to every console port (`sent` and `axes`) and emits `Change` updates only when a control actually transitions. If a core dynamically changes its port count (for example connecting or disconnecting a multitap), `PortRouter::resize()` cleanly releases all held buttons on removed ports without disturbing active ports.

---

## 5. Analog Stick as D-Pad and Dead Zones

For 8-bit and 16-bit consoles lacking native analog sticks (or players preferring thumbstick movement), Endymion projects left-stick deflection onto digital D-pad directions:

- **`AnalogStickAsDpad` (default `true`):** Converts left-stick X/Y deflection into `UP`, `DOWN`, `LEFT`, and `RIGHT` whenever deflection exceeds the configured dead zone.
- **`StickDeadzone` (default `0.5`):** Threshold (`0.0 .. 1.0`) guarding digital directional triggers and menu navigation against analog stick drift.

---

## 6. Guide Chords and Accept / Back Swap

- **`GuideChords` (default `"Auto"`):** Enables Guide/Home button chords for quick pause, save/load state, screenshot, and exit without touching a keyboard. `"Auto"` adapts cleanly under handheld shells such as SteamOS / gamescope where the system overlay may claim the Guide button.
- **`SwapPadButtons` (default `false`):** Swaps Accept and Back face-button roles across UI navigation (for example toggling between East-accept Nintendo conventions and South-accept Western conventions) without changing in-game console button mappings.

---

## 7. Controller Glyphs and UI Input Routing

EmuSen draws resolution-independent vector controller glyphs across help bars and settings pages without bundling proprietary vendor art (`Frontend/kit-core/src/glyph.rs`):

- **Four glyph families (`PadFamily`):** `Generic` (positional RetroPad labels), `Xbox`, `PlayStation`, and `Nintendo`. Configured via `ControllerType` (`"Automatic"` infers the active pad's family from SDL3 metadata through `Platform/endymion/src/family.rs`, or locks to one explicit family).
- **Two visual styles (`GlyphStyle`):** `Outlined` (crisp stroked marks) and `Filled` (solid shapes with cut-out button legends for compact help bars and ES-DE compatibility).
- **Input routing (`qt-endymion` and `kit-qt`):** `qt-endymion/src/pad_reader.rs` polls SDL3 pads and dispatches directional repeats, chords, and face buttons into `kit-qt`'s `PadRouter` queue (`UiButton`). During active gameplay (`qt-endymion/src/game.rs`), gameplay controls flow through `PortRouter` into `Platform/crystal`, while hotkeys and pause chords stay active on the host layer.

---

## 8. Rumble and Force Feedback

Force feedback flows end-to-end from emulated console peripherals through Core ABI v1.17 (`EMUSEN_EVENT_OUTPUT`) to SDL3 gamepad motors (`Platform/crystal/src/rumble.rs`):

1. A core declares its motor outputs (`Strong` and `Weak`) per port in machine info JSON and emits `EMUSEN_EVENT_OUTPUT` events packing `(port << 32) | output` with a `0 .. 65535` intensity level.
2. `Platform/crystal`'s `Rumble` controller enables hardware output only when **both** gates are open (`is_open()`):
   - The console's entry in `Platform/systems/data/` advertises vibration support (`table`).
   - `AppSettings.Rumble` (`true` by default) is enabled (`switch`), which can be toggled live mid-session via `input_rumble_switch` without restarting the core.
3. Crystal exposes `(strong: u16, weak: u16)` per port to the frontend, which drives SDL3 motor rumble on the controller seated in that player slot. Whenever `EMUSEN_EVENT_MACHINE_INFO` fires (such as on reset or peripheral detach), `Rumble::clear()` immediately zeroes all motors so no controller remains stuck vibrating.

---

## Related Documentation

- [Settings and Configuration](Settings_and_Configuration.md) — Input keys in `appsettings.json` and binding file locations.
- [Qt Frontend and Theming](Qt_Frontend_and_Theming.md) — `Help`, `Device`, and `PadRouter` integration in QML themes.
- [Audio, Video, CRT, and Shaders](Audio_Video_CRT_and_Shaders.md) — Endymion's audio pipeline alongside input.
- [Core ABI and Libretro Host](Core_ABI_and_Libretro_Host.md) — `EMUSEN_CAP_AXES`, `EMUSEN_CAP_PRESENCE`, and `EMUSEN_EVENT_OUTPUT`.
