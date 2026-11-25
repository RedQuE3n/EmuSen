# Documentation Migration Map

This lookup table maps every archived document under `retired-donotuse/` to its rewritten live replacement under `Docs/` or the repository root, or marks it `retired-only (no live replacement)`. Use it to resolve older filenames cited in historical source comments.

---

## Root Documents (`retired-donotuse/root-docs/`)

| Archived Path | Live Replacement |
|---|---|
| `README.md` | `README.md` |
| `CONTRIBUTING.md` | `CONTRIBUTING.md` |
| `DESCRIPTION.md` | `retired-only (no live replacement)` — superseded by `README.md` and `Docs/EmuSen Manual/Overview_and_Architecture.md` |

---

## EmuSen Manual (`retired-donotuse/Docs/EmuSen Manual/`)

| Archived Filename | Live Replacement |
|---|---|
| `Commands.txt` | `retired-only (no live replacement)` |
| `EmuSen_Audio_Sync.md` | `Docs/EmuSen Manual/Audio_Video_CRT_and_Shaders.md` |
| `EmuSen_Backend.md` | `Docs/EmuSen Manual/Overview_and_Architecture.md`, `Docs/EmuSen Manual/Platform_C_Interfaces.md` |
| `EmuSen_BigPicture.md` | `Docs/EmuSen Manual/Qt_Frontend_and_Theming.md`, `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_CRT.md` | `Docs/EmuSen Manual/Audio_Video_CRT_and_Shaders.md` |
| `EmuSen_Cheats.md` | `Docs/EmuSen Manual/User_Guide.md`, `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_Config_Reference.md` | `Docs/EmuSen Manual/Settings_and_Configuration.md` |
| `EmuSen_Controllers.md` | `retired-only (no live replacement)` |
| `EmuSen_CoreAPI.md` | `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_CoreAPI_Extensions.md` | `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_Core_Gameplan.md` | `retired-only (no live replacement)` |
| `EmuSen_Core_Naming_Scheme.md` | `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `EmuSen_Cores.md` | `Docs/EmuSen Manual/User_Guide.md`, `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_Debugger.md` | `retired-only (no live replacement)` |
| `EmuSen_Debugging_Tools_Reference_v5.md` | `Docs/EmuSen Manual/DianaSh_and_Debugging.md` |
| `EmuSen_DevicePicture.md` | `retired-only (no live replacement)` — live Vulkan handoff is in `Docs/EmuSen Manual/Audio_Video_CRT_and_Shaders.md` and `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_DianaSh.md` | `Docs/EmuSen Manual/DianaSh_and_Debugging.md`, `Docs/EmuSen Manual/Platform_C_Interfaces.md` |
| `EmuSen_Display.md` | `retired-only (no live replacement)` |
| `EmuSen_Firmware.md` | `Docs/EmuSen Manual/User_Guide.md`, `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_Frontend_Driver.md` | `retired-only (no live replacement)` |
| `EmuSen_Galaxia.md` | `Docs/EmuSen Manual/Settings_and_Configuration.md` |
| `EmuSen_GamePackages.md` | `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_Games_Tested.md` | `retired-only (no live replacement)` |
| `EmuSen_GpuPass.md` | `retired-only (no live replacement)` |
| `EmuSen_Input.md` | `Docs/EmuSen Manual/Input_and_Controllers.md` |
| `EmuSen_Interface.md` | `Docs/EmuSen Manual/Qt_Frontend_and_Theming.md`, `Docs/EmuSen Manual/User_Guide.md` |
| `EmuSen_InterfaceSpec.md` | `retired-only (no live replacement)` — `qt-test` spec recording is in `Docs/EmuSen Manual/Testing_and_Verification.md` |
| `EmuSen_Kit.md` | `Docs/EmuSen Manual/Qt_Frontend_and_Theming.md` |
| `EmuSen_Launcher_Multicore_Gameplan.md` | `retired-only (no live replacement)` |
| `EmuSen_Layout.md` | `retired-only (no live replacement)` — live directory tree is in `Docs/EmuSen Manual/Settings_and_Configuration.md` |
| `EmuSen_LibraryTools.md` | `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md`, `Docs/EmuSen Manual/DianaSh_and_Debugging.md` |
| `EmuSen_Libretro.md` | `retired-only (no live replacement)` |
| `EmuSen_LibretroBridge.md` | `retired-only (no live replacement)` |
| `EmuSen_LibretroHost.md` | `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_LunaP.md` | `retired-only (no live replacement)` |
| `EmuSen_LunaP_Adoption_Gameplan.md` | `retired-only (no live replacement)` |
| `EmuSen_LunaP_Kit.md` | `retired-only (no live replacement)` |
| `EmuSen_Mistress_LibraryPlan.md` | `retired-only (no live replacement)` |
| `EmuSen_Modding.md` | `retired-only (no live replacement)` — ROM patching is in `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_Multicore.md` | `retired-only (no live replacement)` — replaced by `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_NativeCores.md` | `retired-only (no live replacement)` — replaced by `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_Packages.md` | `retired-only (no live replacement)` — replaced by `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_Project_Overview_v2.md` | `Docs/EmuSen Manual/Overview_and_Architecture.md` |
| `EmuSen_Qt.md` | `Docs/EmuSen Manual/Qt_Frontend_and_Theming.md`, `Docs/EmuSen Manual/Building_and_Running.md`, `Docs/EmuSen Manual/Testing_and_Verification.md` |
| `EmuSen_Rewind_And_FastForward.md` | `Docs/EmuSen Manual/User_Guide.md`, `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_RustPlatform.md` | `Docs/EmuSen Manual/Platform_C_Interfaces.md`, `Docs/EmuSen Manual/Building_and_Running.md` |
| `EmuSen_RustState.md` | `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md`, `Docs/EmuSen Manual/Core_ABI_and_Libretro_Host.md` |
| `EmuSen_SaveMedia.md` | `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_Save_States.md` | `retired-only (no live replacement)` — replaced by `Docs/EmuSen Manual/Save_Media_and_Game_Packages.md` |
| `EmuSen_Serenity.md` | `Docs/EmuSen Manual/Audio_Video_CRT_and_Shaders.md` |
| `EmuSen_Settings_Reference.md` | `Docs/EmuSen Manual/Settings_and_Configuration.md` |
| `EmuSen_Shell.md` | `retired-only (no live replacement)` |
| `EmuSen_Stack.md` | `retired-only (no live replacement)` — live stack is in `Docs/EmuSen Manual/Overview_and_Architecture.md` |
| `EmuSen_Surfaces.md` | `retired-only (no live replacement)` — live boundaries are in `Docs/EmuSen Manual/Overview_and_Architecture.md` and `Docs/EmuSen Manual/Platform_C_Interfaces.md` |
| `EmuSen_Testing.md` | `Docs/EmuSen Manual/Testing_and_Verification.md` |
| `EmuSen_TwoFaces.md` | `Docs/EmuSen Manual/Overview_and_Architecture.md` |

---

## Hardware Reference Manuals (`retired-donotuse/Docs/Man pages/`)

### Indexes, `Old/`, and Unstarted Placeholders

| Archived Path (under `Docs/Man pages/`) | Live Replacement |
|---|---|
| `README.md` | `Docs/Man pages/README.md` |
| `Hardware/README.md` | `retired-only (no live replacement)` — consolidated into `Docs/Man pages/README.md` |
| `Old/EmuSen_Crystal_Scheduler.md` | `retired-only (no live replacement)` |
| `Old/EmuSen_Debugging_Tools_Reference.md` | `retired-only (no live replacement)` |
| `Old/EmuSen_Debugging_Tools_Reference_v2.md` | `retired-only (no live replacement)` |
| `Old/EmuSen_Debugging_Tools_Reference_v3.md` | `retired-only (no live replacement)` |
| `Old/EmuSen_Debugging_Tools_Reference_v4.md` | `retired-only (no live replacement)` |
| `Old/EmuSen_LunaP_Gameplan.md` | `retired-only (no live replacement)` |
| `Old/EmuSen_Project_Overview.md` | `retired-only (no live replacement)` |
| `Hardware/Atari/Cyprine & Ptilol - Atari Jaguar/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Atari/Eudial - Atari 2600/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Atari/Mimete - Atari 5200/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Atari/Tellu - Atari 7800/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Atari/Viluy - Atari Lynx/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Microsoft/CereCere - Xbox/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Microsoft/JunJun - Xbox 360/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Microsoft/PallaPalla - Xbox One/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Microsoft/VesVes - Xbox Series/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/NEC/Fish Eye - SuperGrafx/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/NEC/Hawks Eye - PC-FX/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/NEC/Tigers Eye - PC Engine/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Artemis - 3DS-New3DS/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Jupiter - GBA/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Luna - DS/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Neptune - Wii/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Pluto - Wii U/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Saturn - Virtual Boy/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Nintendo/Uranus - GameCube/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Sega/Kunzite - Dreamcast/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Sega/Zoisite - Saturn/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Sony/Esmeraude - PSP/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Sony/Rubeus - PlayStation 3/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |
| `Hardware/Sony/Wiseman - PS Vita/README.md` | `retired-only (no live replacement)` — naming in `Docs/EmuSen Manual/Overview_and_Architecture.md#core-and-component-naming-scheme` |

### Shared Hardware Crates (`Hardware/Shared/`)

| Archived Path (under `Docs/Man pages/`) | Live Replacement |
|---|---|
| `Hardware/Shared/EmuSen_Disc.md` | `Docs/Man pages/Hardware/Shared/Disc.md` |
| `Hardware/Shared/EmuSen_Gpu.md` | `Docs/Man pages/Hardware/Shared/Gpu.md` |
| `Hardware/Shared/EmuSen_MediumFormats.md` | `Docs/Man pages/Hardware/Shared/MediumFormats.md` |
| `Hardware/Shared/EmuSen_Mips.md` | `Docs/Man pages/Hardware/Shared/CpuTables.md` |
| `Hardware/Shared/EmuSen_SingleStep.md` | `Docs/Man pages/Hardware/Shared/CpuTables.md` |
| `Hardware/Shared/EmuSen_Jit.md` | `Docs/Man pages/Hardware/Shared/Recompiler.md` |
| `Hardware/Shared/EmuSen_Recompiler.md` | `Docs/Man pages/Hardware/Shared/Recompiler.md` |

### Nintendo (`Hardware/Nintendo/`)

| Archived Path (under `Docs/Man pages/`) | Live Replacement |
|---|---|
| `Hardware/Nintendo/SilverMillennium-HW/README.md` | `Docs/Man pages/Hardware/Nintendo/SilverMillennium-HW/README.md` |
| `Hardware/Nintendo/Moon - NES/Moon_CPU.md` | `Docs/Man pages/Hardware/Nintendo/Moon - NES/Moon_Hardware.md` |
| `Hardware/Nintendo/Moon - NES/Moon_PPU.md` | `Docs/Man pages/Hardware/Nintendo/Moon - NES/Moon_Hardware.md` |
| `Hardware/Nintendo/Moon - NES/Moon_APU.md` | `Docs/Man pages/Hardware/Nintendo/Moon - NES/Moon_Hardware.md` |
| `Hardware/Nintendo/Moon - NES/Moon_Memory.md` | `Docs/Man pages/Hardware/Nintendo/Moon - NES/Moon_Hardware.md` |
| `Hardware/Nintendo/Moon - NES/Moon_Native.md` | `Docs/Man pages/Hardware/Nintendo/Moon - NES/Moon_Hardware.md` |
| `Hardware/Nintendo/Moon - NES/Moon_PAL.md` | `Docs/Man pages/Hardware/Nintendo/Moon - NES/Moon_PAL.md` |
| `Hardware/Nintendo/Moon - NES/Moon_Core.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Moon - NES/Moon_Debug.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Moon - NES/Moon_Cheats.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Moon - NES/Moon_TestRoms.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Moon - NES/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Cpu.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_Hardware.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Ppu.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_Hardware.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Apu.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_Hardware.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Memory.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_Hardware.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Native.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_Hardware.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Cgb.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_CgbAndModels.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Model.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_CgbAndModels.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Referee.md` | `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_Hardware.md`, `Docs/Man pages/Hardware/Nintendo/Mercury - GB-GBC/Mercury_CgbAndModels.md` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Core.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Cheats.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Debug.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_Gameplan.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_HardwareTests.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mercury - GB-GBC/Mercury_RealCartridges.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mercury - GB-GBC/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_CPU.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Hardware.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_Memory.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Hardware.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_PPU.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Hardware.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_APU.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Hardware.md` |
| `Hardware/Nintendo/Venus - SNES/VenusRT_Native.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Hardware.md` |
| `Hardware/Nintendo/Venus - SNES/VenusRT_Disputes.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Hardware.md`, `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Coprocessors.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_SA1.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Coprocessors.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_SuperFX.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Coprocessors.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_NecDSP.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Coprocessors.md` |
| `Hardware/Nintendo/Venus - SNES/Venus_OBC1.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_Coprocessors.md` |
| `Hardware/Nintendo/Venus - SNES/VenusRT_DspHle.md` | `Docs/Man pages/Hardware/Nintendo/Venus - SNES/Venus_DspHle.md` |
| `Hardware/Nintendo/Venus - SNES/VenusRT_Plan.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Venus - SNES/Venus_Referee.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Cpu.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Cop0.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Tlb.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Privilege.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_ReverseEndian.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Fpu.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_FpuMath.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Memory.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Boot.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Serial.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Save.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Audio.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Rom.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Recompiler.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Native.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_CpuAndMemory.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Rsp.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_Rsp.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RspVector.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_Rsp.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Rdp.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpTriangles.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpCoverage.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpDepth.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpTextures.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpFiltering.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpLod.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpTwoCycle.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpCopy.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpChromaKey.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Gpu.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Video.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_VideoFilter.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_VideoPasses.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_VideoTiming.md` | `Docs/Man pages/Hardware/Nintendo/Mars - N64/Mars_RdpAndVi.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Cheats.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Core.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Corpus.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Debug.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Disassembler.md` | `retired-only (no live replacement)` — moved to `Docs/Man pages/Hardware/Shared/CpuTables.md` |
| `Hardware/Nintendo/Mars - N64/Mars_Documentation.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_GameProbe.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Gameplan.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_GpuPlan.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Microcode.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_Performance.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpDifferential.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_RdpReferee.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_References.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_SaveStates.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/Mars_TestOracle.md` | `retired-only (no live replacement)` |
| `Hardware/Nintendo/Mars - N64/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |

### Sega (`Hardware/Sega/`)

| Archived Path (under `Docs/Man pages/`) | Live Replacement |
|---|---|
| `Hardware/Sega/Beryl-HW/README.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/README.md` |
| `Hardware/Sega/Beryl-HW/Beryl_M68k.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Cpus.md` |
| `Hardware/Sega/Beryl-HW/Beryl_Z80.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Cpus.md` |
| `Hardware/Sega/Beryl-HW/Beryl_SH2.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Cpus.md` |
| `Hardware/Sega/Beryl-HW/Beryl_SmsVdp.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Devices.md` |
| `Hardware/Sega/Beryl-HW/Beryl_SN76489.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Devices.md` |
| `Hardware/Sega/Beryl-HW/Beryl_FmTables.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Devices.md` |
| `Hardware/Sega/Beryl-HW/Beryl_Font.md` | `Docs/Man pages/Hardware/Sega/Beryl-HW/Beryl_Devices.md` |
| `Hardware/Sega/Endou - Master System/Endou_Native.md` | `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_Hardware.md` |
| `Hardware/Sega/Endou - Master System/Endou_OpenBoot.md` | `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_Hardware.md` |
| `Hardware/Sega/Endou - Master System/Endou_Opll.md` | `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_Hardware.md` |
| `Hardware/Sega/Endou - Master System/Endou_Disputes.md` | `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_Hardware.md`, `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_GameGear.md` |
| `Hardware/Sega/Endou - Master System/Endou_Plan.md` | `retired-only (no live replacement)` |
| `Hardware/Sega/Endou - Master System/Endou_References.md` | `retired-only (no live replacement)` |
| `Hardware/Sega/Endou - Master System/Endou_Pacing.md` | `retired-only (no live replacement)` |
| `Hardware/Sega/Endou - Master System/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |
| `Hardware/Sega/Jadeite - Game Gear/Jadeite_Native.md` | `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_GameGear.md` |
| `Hardware/Sega/Jadeite - Game Gear/Jadeite_OpenBoot.md` | `Docs/Man pages/Hardware/Sega/Endou - Master System/Endou_GameGear.md` |
| `Hardware/Sega/Jadeite - Game Gear/README.md` | `retired-only (no live replacement)` — Game Gear is inside `endou` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_Native.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_Genesis.md`, `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_SegaCD.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_OperatorTables.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_Genesis.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_LfoTables.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_Genesis.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_Disputes.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_Genesis.md`, `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_SegaCD.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_Attachments.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_SegaCD.md`, `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_32X.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_Bram.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_SegaCD.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_OpenBios.md` | `Docs/Man pages/Hardware/Sega/Nephrite - Genesis/Nephrite_SegaCD.md` |
| `Hardware/Sega/Nephrite - Genesis/Nephrite_Plan.md` | `retired-only (no live replacement)` |
| `Hardware/Sega/Nephrite - Genesis/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |

### Sony (`Hardware/Sony/`)

| Archived Path (under `Docs/Man pages/`) | Live Replacement |
|---|---|
| `Hardware/Sony/BlackMoon-HW/README.md` | `Docs/Man pages/Hardware/Sony/BlackMoon-HW/README.md` |
| `Hardware/Sony/BlackMoon-HW/BlackMoon_R3000.md` | `Docs/Man pages/Hardware/Sony/BlackMoon-HW/BlackMoon_R3000.md` |
| `Hardware/Sony/BlackMoon-HW/BlackMoon_GTE.md` | `Docs/Man pages/Hardware/Sony/BlackMoon-HW/BlackMoon_GTE.md` |
| `Hardware/Sony/BlackMoon-HW/BlackMoon_SPU.md` | `Docs/Man pages/Hardware/Sony/BlackMoon-HW/BlackMoon_SPU.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Native.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_System.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Pad.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_System.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Cards.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_System.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Speed.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_System.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Disputes.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_System.md`, `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_VideoAndMedia.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_GPU.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_VideoAndMedia.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Enhancements.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_VideoAndMedia.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_CDROM.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_VideoAndMedia.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_MDEC.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_VideoAndMedia.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_SPU.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_VideoAndMedia.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel_Phase2.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Shell.md` | `Docs/Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_OpenKernel.md` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Debugger.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_FPGA_Referees.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Plan.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Players.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Records.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_Referees.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/Diamond_References.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Diamond - PlayStation/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Native.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_EeAndVu.md`, `Sapphire_GsAndVideo.md`, `Sapphire_IopAndIo.md`, `Sapphire_System.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Disputes.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_EeAndVu.md`, `Sapphire_GsAndVideo.md`, `Sapphire_IopAndIo.md`, `Sapphire_System.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Recompiler.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_EeAndVu.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Redesign.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_EeAndVu.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Speed.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_EeAndVu.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Scheduler.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_EeAndVu.md`, `Sapphire_System.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Gs.md` | `Docs/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_GsAndVideo.md` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Plan.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Referees.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_References.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Sapphire - PlayStation 2/Sapphire_SpeedHacks.md` | `retired-only (no live replacement)` |
| `Hardware/Sony/Sapphire - PlayStation 2/README.md` | `retired-only (no live replacement)` — indexed in `Docs/Man pages/README.md` |

---

## DianaSh Man Pages (`retired-donotuse/Docs/man/`)

Every live command man page kept its exact module folder and filename stem in `Docs/man/` and was rewritten in place; seven unported/retired host and session pages were retired with no live replacement:

| Archived Path (under `Docs/man/`) | Live Replacement |
|---|---|
| `dianash/*` (35 pages) | Rewritten in place at `Docs/man/dianash/<page>` |
| `dianash.text/*` (16 pages) | Rewritten in place at `Docs/man/dianash.text/<page>` |
| `emusen.cheat/cheat.1` | Rewritten in place at `Docs/man/emusen.cheat/cheat.1` |
| `emusen.cores/cores.1` | Rewritten in place at `Docs/man/emusen.cores/cores.1` |
| `emusen.debug/*` (26 pages) | Rewritten in place at `Docs/man/emusen.debug/<page>` |
| `emusen.dev/*` (5 pages) | Rewritten in place at `Docs/man/emusen.dev/<page>` |
| `emusen.library/{chd,pkg,rom}.1` (3 pages) | Rewritten in place at `Docs/man/emusen.library/<page>` |
| `emusen.memcard/memcard.1` | Rewritten in place at `Docs/man/emusen.memcard/memcard.1` |
| `emusen.scrape/scrape.1` | Rewritten in place at `Docs/man/emusen.scrape/scrape.1` |
| `emusen.session/coretop.1` | Rewritten in place at `Docs/man/emusen.session/coretop.1` |
| `emusen.session/summary.1` | Rewritten in place at `Docs/man/emusen.session/summary.1` |
| `emusen.session/core.1` | `retired-only (no live replacement)` |
| `emusen.session/state.1` | `retired-only (no live replacement)` |
| `emusen.testing/*` (8 pages) | Rewritten in place at `Docs/man/emusen.testing/<page>` |
| `emusen.trace/*` (16 pages) | Rewritten in place at `Docs/man/emusen.trace/<page>` |
| `nano/nano.1` | Rewritten in place at `Docs/man/nano/nano.1` |
| `host/feed.1` | `retired-only (no live replacement)` |
| `host/hotaru.1` | `retired-only (no live replacement)` |
| `host/pause.1` | `retired-only (no live replacement)` |
| `host/theme.5` | `retired-only (no live replacement)` — Qt/QML theme authoring is in `Docs/EmuSen Manual/Qt_Frontend_and_Theming.md` |
| `host/vstop.1` | `retired-only (no live replacement)` |
