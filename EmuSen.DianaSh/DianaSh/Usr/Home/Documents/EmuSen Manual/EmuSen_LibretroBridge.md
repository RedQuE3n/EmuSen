# EmuSen_LibretroBridge — a libretro core hosted behind the core ABI, as a stopgap for the PlayStation 2

*Proposal, 2026-10-27. Not decided; nothing is built.* Asked by the tester the same day: a playable PlayStation 2 on the
handheld **as a temporary option until Sapphire is playable**, by running RetroArch's PlayStation 2 core inside EmuSen.

## 1. Why

Sapphire runs the tester's three games exactly, but Jak and Daxter's in-engine opening measures 0.75 to 2.8 frames a
second on the desktop (`Sapphire_Speed.md` §51), and the levers left (C1, C7, the recompilers' vector and float
instructions, the GS on the device) are several stages away. RetroArch's PlayStation 2 core (`pcsx2_libretro`) holds Jak
and Daxter at the television's 60 frames a second on the handheld (measured 2026-10-26 through RetroArch 1.22.2,
frames 600 to 2,400). A bridge gives the player that now, inside the same library, saves and controls, and costs
Sapphire nothing: Sapphire stays the project's own core and becomes the default the day it is playable.

## 2. The form it takes, and why not another

Weighed against the code-design rules (an extension, a module or plugin, or a component of its own):

- **A component of its own: one native core, the bridge**, implementing core ABI v1 on one side and loading a libretro
  core (`retro_*`) on the other. It is a second job for no existing core, so it cannot be an extension of Sapphire; and
  it is not a frontend feature, because sessions, states and media already pass through the core ABI, so hosting there
  keeps every frontend (Mistress, big picture, the shell) working unchanged.
- **One bridge, any libretro core** in principle; this proposal scopes it to `pcsx2_libretro`, the stopgap asked for.
  Other libretro cores would be a later decision of the tester's.
- **Rust**, as every core is (`EmuSen_Stack.md` §4), in `EmuSen/Cores/` with its own crate and sidecar.
- **Not shipped with a libretro core.** The bridge loads one the player already has (an installed RetroArch's cores
  folder, or a core placed in EmuSen's own `Cores` folder), as firmware is the player's own (the firmware policy). This
  keeps EmuSen from redistributing another project's binary and its source obligations, and fits a temporary option.

## 3. What the bridge does

| Part | libretro side | EmuSen side |
|---|---|---|
| Lifecycle | `retro_init`, `retro_load_game`, `retro_run`, `retro_unload_game`, `retro_deinit` | session open, run a frame, close (core ABI v1) |
| Picture | software frames through `video_refresh`; **hardware frames through `SET_HW_RENDER` with a Vulkan context** (the PS2 core asks for Vulkan) | first: the frame read back to memory and handed through the existing picture path; later: handed on the device through the picture slot (`EmuSen_GpuPass.md` L4, with Sapphire's S46) |
| Sound | `audio_sample_batch` at the core's rate | the session's sound, resampled as other cores are |
| Input | RetroPad with analog sticks | EmuSen's input is already the RetroPad with sticks (PadButton, PadAxis): a direct map |
| Settings | core options (`SET_CORE_OPTIONS_V2`) | the console's settings window and schema, generated from the options |
| States | `retro_serialize` / `retro_unserialize` | the session's states, named by the core and its version |
| Firmware | the system directory | a per-session folder holding links to the player's BIOS from EmuSen's firmware folder, recognised by content as today |
| Saves | the save directory (memory cards, the console's settings memory) | `Saves/PlayStation 2/` under the save-media rules: the console's own memory shared by every game (`EmuSen_SaveMedia.md` §3.6) |

**The hard part is the picture.** A hardware-rendered libretro core asks its host for a Vulkan device and hands back
images on it. The bridge would create or borrow the device (`EmuSen_GpuPass.md` §7: the device is one and lendable),
answer the core's negotiation, and at each frame copy the core's image to memory for the existing presentation. The
copy costs a few tenths of a millisecond a frame at the PS2's resolutions and is removed later by the device path.

## 4. Stages

| Stage | What | Gate |
|---|---|---|
| **B1** | The bridge core: loading, lifecycle, software pictures, sound, input, states, the firmware and save folders | WiseMan tests against a small libretro test core built in the repository (no commercial image, no third-party core in tests); the core conformance kit |
| **B2** | Hardware rendering: the Vulkan context and the frame read back | The PS2 core boots the tester's BIOS and games through the bridge; frame time on the desktop and the handheld against RetroArch's |
| **B3** | Integration: the PS2 shelf offers both engines, Sapphire and the bridge, with a per-console choice (the engine row other consoles already have); options in the settings window; the fit audits | Mistress and big picture run a game either way; saves and the console memory follow the rules; the choice remembered |

About two to four stages, most of it B2.

## 5. What stays true

- **The clean room is unaffected.** The libretro core is only run, as PCSX2 already is as a referee; no emulator source
  is read. The bridge is written from libretro's public interface header and its documentation.
- **Sapphire is unchanged** and keeps its speed work; the bridge touches no Sapphire code.
- **Licensing**: EmuSen and the PS2 core are both GPLv3; EmuSen ships none of the core's code.

## 6. Questions for the tester

1. **Go or no.** *Recommended:* yes, as a stopgap.
2. **When.** *Recommended:* after C1 and C7, which decide how far Sapphire alone can go; earlier if a playable PS2 on
   the handheld matters more this month than Sapphire's speed.
3. **Shipping the libretro core.** *Recommended:* not shipped; the bridge finds the player's own.
4. **The default engine for the PS2 while Sapphire is not playable.** *Recommended:* the bridge where a libretro core is
   found, Sapphire otherwise, with the choice always the player's; Sapphire becomes the default when it is playable.
5. **Scope.** *Recommended:* the PS2 only; other libretro cores a later decision.
