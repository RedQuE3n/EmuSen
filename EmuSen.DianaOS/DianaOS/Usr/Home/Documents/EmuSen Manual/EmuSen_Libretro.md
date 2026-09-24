# EmuSen as libretro cores — what RetroArch requires, and what it would take

*This revision: the first, 2026-09-23. A specification, not an implementation: nothing described here is built. It
answers one question — what an emulator core must be for RetroArch to run it — and then asks it of each of EmuSen's
cores. Every claim about libretro is cited to a file and line or a URL. Claims about EmuSen are cited to the page that
proved them. Where the document argues rather than cites, it says so, and §9 collects what is open.*

**How the sources are cited.**
- **`libretro.h:N`** is `~/Projects/retroarch-reference/libretro-common/include/libretro.h`. That is RetroArch 1.22.2
  (`version.all`: `PACKAGE_VERSION "1.22.2"`), whose `CHANGES.md` opens a "Future" section. The header there is 8,242
  lines long. RetroArch master and libretro-common master both differ from it (8,730 lines at the time of writing), so
  line numbers drift. The names and values quoted do not.
- **`libretro_vulkan.h:N`** is the same folder's Vulkan header.
- **`runloop.c:N`**, **`core_info.c:N`** and the other bare file names are RetroArch's own source in the same checkout.
- **`PN/…`** is parallel-n64 at `~/Projects/parallel-n64-reference`, the local N64 libretro core used as the worked
  example.
- Web sources are given as URLs. Pages that do not exist are named as missing: docs.libretro.com has no page on core
  options v2, on rewind, or on the `.info` format.

---

## 1. The contract

### 1.1 What a core exports

A libretro core is a dynamic library that exports **all twenty-five** `retro_*` functions of `libretro.h`
(`libretro.h:7907–8236`). RetroArch resolves them by name in `CORE_SYMBOLS` (`runloop.c:321–346`), and `SYMBOL` calls
`retroarch_fail` on any that is missing (`runloop.c:286–290`). **None is optional to export.** "Optional" below means
only that a stub body — returning 0, `false` or `NULL`, or doing nothing — is a conforming answer.

| Function | When it is called, and what it must do | A stub is conforming? |
| --- | --- | --- |
| `retro_api_version` | Must return `RETRO_API_VERSION`, which is 1 (`libretro.h:97`, `:8014`) | no |
| `retro_get_system_info` | Callable at any time, even before `retro_init` (`:8019`). It fills `library_name` (no version in it), `library_version`, `valid_extensions` (pipe-delimited), `need_fullpath` and `block_extract` (`:6304–6351`). The strings must be statically allocated. | no |
| `retro_set_environment` | Called before `retro_init` (`:7903`). The core stores the callback, and makes the calls that must come early here: options, the update-display callback, subsystems and no-game (§2) | no |
| `retro_set_video_refresh`, `_audio_sample`, `_audio_sample_batch`, `_input_poll`, `_input_state` | Called before the first `retro_run` (`:7914–7950`). The core stores each pointer. | no (trivial) |
| `retro_init` / `retro_deinit` | Set up and release everything. Global state may not be assumed fresh, because the library may be retained or linked statically, and it must be reset in `deinit` (`:7961–7999`) | no |
| `retro_get_system_av_info` | Callable only after a successful `retro_load_game` (`:8028`). It gives the geometry (base and max width and height, aspect) and the timing (`fps`, `sample_rate`) (`:6567–6640`) | no |
| `retro_load_game` / `retro_unload_game` | Takes a `retro_game_info`: the data and size, or only a path when `need_fullpath` is set (`:6324–6343`). `NULL` means no content (`:8150`). Unload must allow another load (`:8189`) | no |
| `retro_load_game_special` | Subsystems only (`:8163`) | yes: return `false` |
| `retro_run` | One video frame. `input_poll` must be called at least once (`:8081`). A frame the game did not draw still counts, and is duped by passing `NULL` when `GET_CAN_DUPE` is true (`:8083–8086`) | no |
| `retro_reset` | "a soft reset … if possible, but hard resets are acceptable" (`:8073–8074`) | no, but a hard reset is enough |
| `retro_serialize_size` / `retro_serialize` / `retro_unserialize` | Save states. The size may never grow between load and unload (`:8095–8097`). §2.3 gives the rules that matter | yes: 0 means "no states" |
| `retro_cheat_reset` / `retro_cheat_set(index, enabled, code)` | Cheats as text codes (`:8133–8144`) | yes |
| `retro_set_controller_port_device` | Device per port, as a hint (`:8040–8069`) | yes |
| `retro_get_region` | `RETRO_REGION_NTSC` or `_PAL`. This is the television standard, not the origin. Handhelds answer NTSC (`:8197–8208`) | no (trivial) |
| `retro_get_memory_data` / `retro_get_memory_size` | Raw pointers to `SAVE_RAM` 0, `RTC` 1, `SYSTEM_RAM` 2 and `VIDEO_RAM` 3 (`:510–521`), or `NULL` and 0 (`:8210–8236`) | yes, but §2.4 and §2.5 depend on it |

### 1.2 The calling rules

- **One thread calls the core.** The developing-cores guide says: "assume the functions declared in the libretro
  header are neither reentrant nor safe to be called by multiple threads at the same time … It is discouraged to do
  libretro API calls outside of retro_run() i.e. outside of the main thread"
  (https://docs.libretro.com/development/cores/developing-cores/). It says "discouraged", not "forbidden". The
  exceptions are the asynchronous audio callback (`libretro.h:1119`) and Vulkan submission, which "can happen on any
  thread" under the queue lock (`libretro_vulkan.h:109–113`, `:470–480`).
- **The core may run threads of its own.** Nothing in the header forbids it. parallel-n64's paraLLEl-RDP builds and
  submits on worker threads (`libretro_vulkan.h:250`, "Vulkan cores should be able to be freely threaded"). What
  follows from the rule above is that **only the thread inside `retro_run` calls back**. That is argued from the
  guide's wording, not stated in it.
- **Fixed rates, no sleeping.** "Libretro is based on fixed rates; video FPS and audio sampling rates are always
  assumed to be constant … replace any arbitrary sleep() and time() patterns with simply calling video/audio callbacks.
  The frontend will make sure to apply the proper synchronization" (developing-cores guide). The frontend's dynamic
  rate control resamples, so the core never blocks.
- **Video.** The default pixel format is 0RGB1555, which is deprecated (`libretro.h:856–859`). A new core sets
  `RETRO_PIXEL_FORMAT_XRGB8888` (1) or `RGB565` (2) (`:5901–5914`) with `SET_PIXEL_FORMAT` from `retro_load_game`
  (`:865`). The data may have a pitch, but a packed frame is recommended (`:7780–7784`). **RetroArch keeps the core's
  pointer** after the call: `video_driver_cached_frame_publish` stores `data` as `frame_cache_data`
  (`gfx/video_driver.c:3486–3498`), and redraws from it while paused, in the menu and for screenshots. So the buffer a
  core passes must stay valid, and unchanged, until it passes another. This is the reverse of Mistress's lending
  (`Mars_Native.md` §6.13): here the frontend borrows the core's memory.
- **Audio.** Signed 16-bit stereo, interleaved, native-endian (`libretro.h:7801`, `:7815–7817`). "Only one of the
  audio callbacks must ever be used" (`:7812`). The batch callback should not be used for fewer than 32 frames at a
  time, and should be 16-byte aligned (developing-cores guide). The local header adds a float batch
  (`GET_AUDIO_SAMPLE_BATCH_FLOAT`, `:2681`), which must not be mixed with int16 within one game.
- **Input.** Polled, not pushed. `input_state(port, device, index, id)` (`:7895`). The RetroPad's sixteen ids are
  `B, Y, SELECT, START, UP, DOWN, LEFT, RIGHT, A, X, L, R, L2, R2, L3, R3` (`:320–371`). Analog axes are signed 16-bit on
  `RETRO_DEVICE_ANALOG` with index LEFT or RIGHT (`:389–393`). `GET_INPUT_BITMASKS` returns all buttons in one call
  with id `RETRO_DEVICE_ID_JOYPAD_MASK` 256 (`:1809`, `:380`).
- **Changing the picture's size or the rates.** `SET_GEOMETRY` changes the size within `max_width`/`max_height`, in
  constant time, and only from `retro_run` (`:1536–1558`). `SET_SYSTEM_AV_INFO` "may entail a full reinitialization of
  the frontend's audio/video drivers". It is for a larger maximum or a new rate, and must not be used for a size change
  within the maximum (`:1336–1369`).

### 1.3 What EmuSen's `ICore` lacks against it

The survey of `EmuSen/Cores/ICore.cs` shows what is missing, for the record: `ICore` has no reset (only `MoonCore.Reset`,
`MoonCore.cs:115`), no region (only Venus knows one, `ConsoleRegion.cs`), no in-memory battery save (each core reads and
writes its own `.srm`), and no size query for a state. These gaps matter only for a C# core behind libretro, which §6
rules out. A Rust core's wrapper speaks to the machine directly (§4), so the gaps do not follow it. Two things already
line up. `PadButton`'s sixteen values are the RetroPad's ids in the RetroPad's order (`EmuSen.Galaxia/Input/PadButton.cs:3–22`,
"libretro's RetroPad order; add only at the end"). `IFrameSerial`, whose unchanged value promises unchanged pixels, is
exactly the test that decides a `NULL` dupe.

---

## 2. What RetroArch users expect beyond the minimum

### 2.1 Core options (v2, with categories)

- **The call.** `SET_CORE_OPTIONS_V2` (67, `libretro.h:2327`), or `_V2_INTL` (68) with translations. It is registered
  "ideally in retro_set_environment (but retro_load_game is acceptable)". Values are read back with `GET_VARIABLE`
  (15), and `GET_VARIABLE_UPDATE` (17) says whether any changed.
- **The fallback chain.** Ask `GET_CORE_OPTIONS_VERSION` (52). A `false` answer means version 0 (`:1814–1824`). RetroArch
  answers 2. For version 1, copy the definitions into v1's arrays. For version 0, build `"Desc; default|v2|v3"` strings
  for `SET_VARIABLES`. The template that does all three is libretro-common's
  `samples/core_options/example_categories/libretro_core_options.h` (header `VERSION: 2.0`;
  https://github.com/libretro/libretro-common/tree/master/samples/core_options). `example_default/` is the v1 template.
- **The shape.**
  - `retro_core_option_v2_definition {key, desc, desc_categorized, info, info_categorized, category_key, values[128],
    default_value}` (`:6919–7036`).
  - Keys are prefixed with the core's name ("foo_resolution").
  - `default_value` must be one of the values, "or else this option will be ignored".
  - Values carry no units, and paths use `/` (`:6758–6799`).
  - Categories are `{key, desc, info}` (`:6872`).
  - `SET_CORE_OPTIONS_DISPLAY` (55) and the update-display callback (69, registered in `retro_set_environment`) hide and
    show options.
- **The guiding rule** is that "all common interactions … should be possible without a keyboard", so options should
  be few (`:2304–2308`).

**For EmuSen this is close to free.** `CoreSetting(Key, Label, Hint, Kind, Default, Min, Max, Choices)` holds every
value as text, of three kinds (`EmuSen/Cores/CoreCapabilities.cs:39–51`). Switch becomes `enabled|disabled`, Count
becomes its range written out, and Choice becomes its names. `Hint` becomes `info`. The one mismatch is timing.
`ICoreSettings.Set` is applied between frames, and so is a `GET_VARIABLE_UPDATE` read at the top of `retro_run`. A
setting that rebuilds the machine (`ExpansionPak`) is the exception, taken up in §4.4.

### 2.2 Input descriptors and controllers

`SET_INPUT_DESCRIPTORS` (11) names each button and axis for the remap menu. It should be called no later than
`retro_load_game` (`:875–886`). Its strings must live until unload (`:6283–6295`). `SET_CONTROLLER_INFO` (35) lists the
device types each port offers (`:1450–1510`). The `.info` template has an `input_descriptors` flag. **RetroArch never
reads it**: a grep of 1.22.2 finds no reader for `cheats`, `input_descriptors`, `memory_descriptors`, `libretro_saves`,
`core_options`, `core_options_version`, `load_subsystem`, `hw_render`, `needs_fullpath` or `disk_control`. They are
documentation for people. What RetroArch does parse is listed in `core_info_parse_config_file`
(`core_info.c:1657–1869`).

### 2.3 Save states, and rewind, run-ahead and netplay

This is the requirement that matters most, because RetroArch's three most-used features are built on it.

- **The size is fixed per session, in practice.** The header says the size may never grow between load and unload
  (`libretro.h:8095–8097`). RetroArch goes further and sizes all three features once:
  - rewind (`state_manager.c:574`; a later larger state fails, `tasks/task_save.c:508–518`);
  - run-ahead (`runahead.c:1242–1269`);
  - netplay (`netplay_frontend.c:7180–7226`).

  `RETRO_SERIALIZATION_QUIRK_CORE_VARIABLE_SIZE` exists (`libretro.h:3719`), but no code in RetroArch reads it. **A core
  should report one size for the whole session.**
- **The `.info` file gates the features.** `savestate_features` sets a level: basic, serialized (rewind), or
  deterministic (netplay and run-ahead) (`core_info.h:27–43`). The gates are `core_info.c:2976–2991`: rewind needs
  serialized, and netplay and run-ahead need deterministic.

  Parsing has a trap (`core_info.c:1842–1869`). The default is **deterministic**. Only the strings `"basic"` and
  `"serialized"` are compared, so `"deterministic"`, a missing key and a missing `.info` file all mean deterministic.
  `mesen-s_libretro.info` and `sameboy_libretro.info` have no savestate keys at all, and so count as deterministic.
- **What deterministic means operationally.**
  - *Netplay* hashes the core's serialized bytes with CRC32 every `check_frames` and asks for a whole state when the
    hashes differ (`netplay_frontend.c:2218–2228`, `3286–3326`). So `retro_serialize` must be **byte-deterministic**:
    no uninitialised padding, no pointers, no timestamps.
  - *Netplay refuses to start* if the core sets `INCOMPLETE` or `SINGLE_SESSION` (`netplay_frontend.c:9215–9225`).
  - *Run-ahead* in one instance saves a state, runs hidden frames, and loads it again, every frame
    (`runahead.c:1550–1600`). "Run Ahead relies on save states so they need to be clean and fast enough"
    (https://docs.libretro.com/guides/runahead/).
  - *The second-instance mode* **copies the core's binary to a temporary file with a random name and loads the copy**
    (`runahead.c:379`, `:477`), so two copies of the library live in one process. It exists because "many of the
    cores do not leave audio emulation in a clean state after loading state" (the run-ahead guide).
- **The context a state is taken in.** `GET_SAVESTATE_CONTEXT` (72|EXP, `libretro.h:2418`) answers one of four values
  (`:5979–6034`):
  - `NORMAL`: to disk, loadable anywhere, no pointers;
  - `RUNAHEAD_SAME_INSTANCE`: pointers acceptable, and it must be fast;
  - `RUNAHEAD_SAME_BINARY`: no pointers;
  - `ROLLBACK_NETPLAY`: another machine, no pointers, and "integers must be saved in big-endian format".

  RetroArch sets a special context only on its special paths (`runloop.c:3194–3227`). **Rewind is not one of them.**
  It calls plain `core_serialize` (`tasks/task_save.c:478`), so a core sees `NORMAL` during rewind, whatever the
  header's comment at `:5997` suggests.
- **Frames that will not be seen.** `GET_AUDIO_VIDEO_ENABLE` (47|EXP, `:1755`) clears the video bit on run-ahead's
  hidden frames and on netplay's replays. A core may skip rendering then, but skipping "must leave the next frame and
  the save state identical", and "Emulation accuracy should not be compromised" (`:1741–1744`).
- **Quirks.** `SET_SERIALIZATION_QUIRKS` takes seven flags (`:3698–3738`). RetroArch stores the value and uses it only
  in netplay (`runloop.c:3072–3079`). **The call's number is disputed.** It was 44 in stable releases. RetroArch PRs
  #19161 and #19169 moved it to 87 (the local header's value, `:2716`) to end a clash with `SET_HW_SHARED_CONTEXT`.
  Open PR https://github.com/libretro/RetroArch/pull/19594 proposes moving it back. A core should not depend on either
  number until that settles.

### 2.4 Battery saves

RetroArch owns the file. It reads the `.srm` into `retro_get_memory_data(RETRO_MEMORY_SAVE_RAM)` after the game loads,
and writes it back at unload or on an interval. That is the behaviour the `libretro_saves` key documents. So the core
must hold its battery memory as **one buffer at a stable address** that the frontend may fill after `retro_load_game`.
It must not read the save itself. For the N64, the two local-world cores use one layout: `eeprom[0x800]`,
`mempack[4][0x8000]`, `sram[0x8000]`, `flashram[0x20000]` (`PN/libretro/libretro_memory.h:7–14`). A core that uses it
gets `.srm` files that move between N64 cores. That is a user expectation, argued here from the shared layout, not a
rule anyone states.

### 2.5 RetroAchievements

- **The calls.** `SET_SUPPORT_ACHIEVEMENTS` (42|EXP, `libretro.h:1664`) declares support, and the address space is
  exposed through `retro_get_memory_data` or `SET_MEMORY_MAPS` (36|EXP, `:1534`; descriptors `:3837`).
- **The N64 regions.** rcheevos wants three, all system RAM: RDRAM 0x000000–0x1FFFFF at 0x80000000, 0x200000–0x3FFFFF,
  and the Expansion Pak's 0x400000–0x7FFFFF (`deps/rcheevos/src/rcheevos/consoleinfo.c:729–731`). So `SYSTEM_RAM` of up
  to 8 MB is enough. parallel-n64 returns `g_dev.rdram.dram` with `RDRAM_MAX_SIZE` (`PN/libretro/libretro.c:2817–2835`).
- **Byte order is the catch.** rcheevos has no N64 byte swap: the only `NINTENDO_64` cases in its source are region
  tables and hashing. mupen-derived cores keep RDRAM as host-endian 32-bit words. The existing sets are therefore
  written against that layout: on a little-endian host, byte address XOR 3. **That last step is inferred.** A
  third-party note says the same (https://github.com/odelot/Main_MiSTer, "`addr ^ 3` byte-order correction").
  docs.retroachievements.org does not say (https://docs.retroachievements.org/developer-docs/console-specific-tips.html).
  §4.6 draws the consequence for MarsRT.
- **Hardcore mode is the frontend's.** It blocks state loads, rewind, slow motion, cheats and memory writes
  (`command.c:74–81`, `runloop.c:6915–6924`, `cheevos.c:884–917`). There is no environment call that tells a core
  hardcore is on. The official rules are https://docs.retroachievements.org/general/hardcore-compliance-requirements.html.
  rcheevos keeps per-core option blocklists keyed by `library_name` (`rc_libretro.c`), so the library name is part of
  the contract.

### 2.6 Cheats

`retro_cheat_set(index, enabled, code)` hands the core a code as text. The core decodes it: GameShark for the N64, Game
Genie and raw for the NES, and so on. RetroArch's cheat manager can also search and write memory directly through the
memory map (`command.c:1045`, `:1310`). That path bypasses the core.

### 2.7 Timing and audio rate

- **The frame-time callback** (21, `libretro.h:1092`) gives the core the real time since the last frame. It is for
  engines, not for emulators whose time is the console's.
- **The frame rate** in `av_info` should be the console's. parallel-n64 reports 50 for PAL and 60.13 otherwise, with a
  "TODO: Actual timing" (`PN/libretro/libretro.c:1055`). FFmpeg recording "relies on exact information" (the
  developing-cores guide).
- **A rate the game changes** (the N64's AI DAC) is handled one of two ways:
  - *Declare it.* The local parallel-n64 declares the game's rate and sends `SET_SYSTEM_AV_INFO` when it changes,
    leaving the resampling to the frontend (`PN/mupen64plus-core/src/plugin/audio_libretro/audio_backend_libretro.c:24–38`,
    `:251–279`).
  - *Resample in the core* to a fixed rate.

### 2.8 Hardware rendering, and Vulkan in particular

A core that renders on the GPU asks for a context in `retro_load_game`: `SET_HW_RENDER` (14, `libretro.h:926–949`) with
`context_type = RETRO_HW_CONTEXT_VULKAN` (6, `:5356`), `context_reset` and `context_destroy` (`:5375–5457`). It then
passes only `RETRO_HW_FRAME_BUFFER_VALID` or `NULL` to the video callback (`:946`). RetroArch forces the matching video
driver (`gfx/video_driver.c:2992–3017`). The Vulkan protocol:

1. **Negotiation (optional).** `SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE` (43|EXP) with
   `retro_hw_render_context_negotiation_interface_vulkan` (`libretro_vulkan.h:116–234`, interface version 2):
   - `get_application_info` sets the API version. The header advises 1.1 for Android's sake.
   - `create_device2` lets the core create the **one** device both share. It calls the frontend's
     `create_device_wrapper` so that the frontend can add its own extensions. "Device provided to frontend is owned by
     the frontend" (`:159`).
   - The core must return `false` if it cannot use the GPU the frontend chose (`:201`).
   - RetroArch tries the frontend's GPU, then `VK_NULL_HANDLE` ("letting core decide"), then v1's `create_device`
     (`gfx/common/vulkan_common.c:790–830`).
2. **The interface.** After `context_reset`, `GET_HW_RENDER_INTERFACE` (41|EXP) returns
   `retro_hw_render_interface_vulkan` (`libretro_vulkan.h:236–492`): the instance, GPU, device, the proc-address
   functions, and one queue with its family, which is constant for the context's life.
3. **Handing over a picture.** Each frame, `set_image` is called before `video_refresh(RETRO_HW_FRAME_BUFFER_VALID)`
   (`:279–389`).
   - The image needs at least `TRANSFER_SRC` and `SAMPLED` usage, `MUTABLE_FORMAT` for 8-bit formats, optimal tiling,
     and `SHADER_READ_ONLY_OPTIMAL` or `GENERAL` layout.
   - Barriers are preferred to semaphores.
   - The core must not touch the image until `wait_sync_index` has completed.
4. **Synchronisation.**
   - `get_sync_index` and `get_sync_index_mask` index the core's per-frame resources by swapchain image (`:391–440`).
   - `set_command_buffers` lets the frontend submit the core's work with its own (`:442–463`).
   - `lock_queue`/`unlock_queue` **must** surround any submission the core makes itself, from any thread (`:470–480`).
5. **Reset.** RetroArch waits the device idle and drops the cached image before `retro_reset`
   (`gfx/drivers/vulkan.c:4953–4993`).

parallel-n64 is the pattern on the local disk:
- `retro_init_vulkan` requests the context (`PN/libretro/libretro.c:1118–1146`);
- `parallel_create_device` builds Granite on the frontend's instance and releases the device to it
  (`PN/mupen64plus-video-paraLLEl/rdp.cpp:467–505`);
- per-frame images are sized from the mask, and Granite's queue lock goes through the interface (`rdp.cpp:213–240`);
- `set_image` is called with no semaphores and no ownership transfer (`rdp.cpp:448`).

### 2.9 Shaders

RetroArch applies its own shader chain (slang, GLSL, Cg) to whatever the core hands it. The core's part is to hand
over the **console's own raster**, at its real line count, with the right `aspect_ratio`. A CRT shader's scanlines are
drawn per source line, so a line-doubled frame gets twice the scanlines. This is argued from how the shaders work, not
stated by libretro. A core should also not apply a filter of its own.

### 2.10 What the `.info` file carries

The `.info` file lives in libretro-super's `dist/info`, mirrored to libretro-core-info ("Submit changes to
libretro-super instead"). Its only documentation is the template, `00_example_libretro.info`
(https://github.com/libretro/libretro-super/tree/master/dist/info).

The keys RetroArch reads (`core_info.c:1657–1869`):
- `display_name`, `display_version`, `corename`, `systemname`, `systemid`, `manufacturer`;
- `supported_extensions`, `authors`, `permissions`, `license`, `categories`, `database`, `notes`, `required_hw_api`,
  `description`;
- `supports_no_game`, `single_purpose`, `database_match_archive_member`, `is_experimental`, `savestate`,
  `savestate_features`;
- `firmware_count` and `firmwareN_desc`/`_path`/`_opt`.

Three have effects beyond display:
- `savestate_features` gates rewind, run-ahead and netplay (§2.3).
- `is_experimental` **hides the core from the Core Downloader** unless the user enables "Show Experimental Cores"
  (`menu/menu_displaylist.c:14011–14018`; default off, `config.def.h:1555`).
- `supports_no_game` and `single_purpose` feed the contentless-cores menu.

`required_hw_api` ("Vulkan >= 1.0 | …") is **only displayed** in 1.22.2 (`menu/menu_displaylist.c:765–776`). What gates
hardware rendering is the runtime negotiation.

The real N64 files, for comparison:
- `parallel_n64_libretro.info`: `supported_extensions = "n64|v64|z64|bin|u1|ndd"`, `license = "GPLv2"`,
  `permissions = "dynarec_optional"`, `required_hw_api = "OpenGL >= 3.0 | OpenGL ES >= 2.0 | Vulkan >= 1.0"`,
  `savestate_features = "serialized"`, and an optional 64DD IPL.
- `mupen64plus_next_libretro.info`: the same, with `systemid = "nintendo_64"`.
- Both claim only *serialized*, so neither offers run-ahead or netplay.

---

## 3. Packaging and distribution

### 3.1 File names

- **The extension** is `dll` on Windows, `dylib` on other Apple builds, `framework` for iOS and for the macOS App
  Store, and `so` otherwise (`frontend/frontend_driver.c:180–190`).
- **The buildbot's names**, checked on buildbot.libretro.com on 2026-09-23:
  - `<name>_libretro.so` for Linux;
  - `<name>_libretro.dll` for Windows;
  - `<name>_libretro.dylib` for macOS, arm64 and x86_64 separately;
  - `<name>_libretro_android.so` for each Android ABI;
  - `<name>_libretro_ios.dylib` and `<name>_libretro_tvos.dylib` for iOS and tvOS.
- **Apple's frameworks** turn underscores into dots, so `mupen64plus_next_libretro_ios.dylib` ships as
  `mupen64plus.next.libretro.framework` (`pkg/apple/make-frameworks.sh:45–53`). RetroArch changes the dots back to find
  the `.info` (`core_info.c:1535–1543`).
- **How the `.info` is found.** The name is cut at its last underscore unless that is `_libretro`. So
  `x_libretro_android.so` finds `x_libretro.info`, and a core name with an underscore of its own works, but only one
  platform suffix is stripped (`core_info.c:1524–1553`).

### 3.2 How a core reaches the Online Updater

The buildbot is GitLab CI at git.libretro.com. A core carries a `.gitlab-ci.yml` that `include`s templates from
`libretro-infrastructure/ci-templates` and `extends` them with a `.core-defs` block. **Rust has official templates:**
`rust-linux-x64`, `rust-linux-aarch64`, `rust-linux-i686`, `rust-windows-x64`, `rust-windows-i686`, `rust-osx`,
`rust-apple`, `rust-ios`, `rust-android-jni` and `rust-webos`, from the repository's tree listing
(https://git.libretro.com/libretro-infrastructure/ci-templates). There is **no Rust tvOS template**.

The templates fix the shape of the build:
- `rust-linux-x64.yml` runs `cargo build --release --target ${TARGET_ARCH}` in the repository's root, then
  `mv target/${TARGET_ARCH}/release/lib${CORENAME}.so ${CORENAME}_libretro.so`, and strips it.
- It runs in the image `libretro-infrastructure/libretro-build-rust`.
- `rust-ios.yml` does the same for `aarch64-apple-ios` with `IPHONEOS_DEPLOYMENT_TARGET: "12.0"`.

So a buildbot Rust core is a crate at a repository's root whose `cdylib` is named after the core.

The steps that recent Rust cores have followed (https://github.com/AloysHF/PlaydiaEmu/issues/5;
https://github.com/libretro/libretro-super/issues/2079, `2099`):
1. a `.gitlab-ci.yml` on the templates;
2. a pull request of `<core>_libretro.info` to libretro-super's `dist/info`;
3. a request to the libretro team to create the GitLab project and its pipeline;
4. optionally, a page in libretro/docs, icons in retroarch-assets, and database entries.

**Unverified:** which `rustc` the `libretro-build-rust` image carries. MarsRT's edition 2024 needs Rust 1.85 or later,
and Cranelift 0.136 may need later still. This is the first thing to ask upstream (§9).

### 3.3 The platform matrix

| Platform | Buildbot form | What decides it for MarsRT |
| --- | --- | --- |
| Linux x86-64, aarch64 | `.so` | The glibc floor. MarsRT's CI already builds on `ubuntu-22.04` for glibc 2.35 (`Mars_Native.md` §6.3) |
| Windows x86-64 | `.dll` | Nothing new. `marsrt.dll`'s exports already survive the linker (§6.3) |
| macOS arm64, x86-64 | `.dylib` | Cranelift's writable-then-executable pages run in an unsigned process (§6.3's first run). A **notarised** RetroArch with the hardened runtime needs the `allow-jit` entitlement, and that is RetroArch's signature, not the core's. Untested |
| Android arm64-v8a (and others) | `_android.so` | `rust-android-jni`. Cranelift supports aarch64. W^X under Android's SELinux policy is untested |
| iOS / tvOS | bundled into the app | "iOS does not support installing/updating cores at runtime. Instead, all cores must be built and added to the RetroArch source tree" (https://docs.libretro.com/development/retroarch/compilation/ios/). "JIT is not enabled on the App Store version of RetroArch" (https://docs.libretro.com/guides/install-ios/). So MarsRT runs interpreted there (§4.5) |
| Emscripten, consoles | static builds | Out of scope: no threads that match §5.6's design, and no Cranelift target |

### 3.4 Licensing

- **The licence itself.** GPL-3.0 is accepted: `bsnes_libretro.info` and `mesen-s_libretro.info` both say
  `license = "GPLv3"`, and RetroArch is GPLv3 itself. What libretro screens out is **non-commercial** licences: such
  software "MAY NOT BE SOLD, NOR MAY THEY BE USED IN A COMMERCIAL PRODUCT OR ACTIVITY WITHOUT COPYRIGHT HOLDERS'
  APPROVAL" (https://docs.libretro.com/development/licenses/). MarsRT's dependencies are compatible, as its
  `Cargo.toml` records: ash is MIT or Apache-2.0, libloading ISC, Cranelift Apache-2.0 WITH LLVM-exception. The
  libretro header is MIT (`libretro.h:9–25`), so bindings generated from it add no obligation.
- **Marketplaces are a matter of consent, not of the licence.** The App Store build includes only cores "we maintain
  in-house or whose upstream teams have given us explicit approval to distribute it via marketplaces"
  (https://docs.libretro.com/guides/install-ios/). Steam ships cores as free DLC, each "given approval by the authors of
  the core code" (https://www.libretro.com/index.php/retroarch-finally-released-on-steam/). **For EmuSen's cores the
  approval is the user's to give**, as sole copyright holder. The long-running argument over whether GPL terms can be
  met under App Store terms is not settled here, and libretro's practice of written approval sidesteps it.

---

## 4. MarsRT as a libretro core

*In this section a bare § number (§5.5, §6.13) is a section of `Mars_Native.md`.*

### 4.1 Where the wrapper sits

MarsRT is already built for this. Its crate is `crate-type = ["cdylib", "rlib"]` (`MarsRT - N64/Cargo.toml`). The
`rlib` lets a second crate link the machine as Rust, with no C ABI in between.

**The proposal.** A crate `marsrt-libretro`, a `cdylib` that depends on `marsrt` and on the shared crate of §7, and
exports the twenty-five functions. It calls `Machine` and `Core` methods directly: `run_frame`, `present`, `press`,
`set_stick`, `ai.drain`, `save_state`, `restore_state`, `set_threads`, `set_recompiler` and `set_multiple`, whose C
forms are `src/ffi/mod.rs:219–706`, `threads.rs`, `blocks.rs` and `multiple.rs`.

**What stays out of the libretro build:**
- the C# shim, and with it the frame lending, which libretro inverts (§1.2);
- the debugger's tables (`ffi/debug.rs`);
- the test ABIs (`ffi/rdp.rs`, `ffi/vi.rs`).

**Why not wrap the existing C ABI instead.** A wrapper in C over `mars_machine_*` would work, but it would add a
language and keep a boundary that §3.4's lesson says to avoid. Rust over the `rlib` does neither. The machine's global
state is small: the crash-log `OnceLock` (`src/lib.rs:55`), constant tables, and a few `OnceLock`s read from the
environment (`rsp/mod.rs:145`, `rsp/decoded.rs:48`, `vi/scan/bands.rs:19`). Every machine is a value, so run-ahead's
second copy of the library (§2.3) and a second machine in one copy both work. The environment-read defaults are
per-process. That is harmless, but the libretro build should not read `EMUSEN_*` variables at all, so that a user's
shell cannot change what RetroArch runs.

### 4.2 What maps directly

| libretro | MarsRT | Note |
| --- | --- | --- |
| `retro_load_game` (data, `need_fullpath = false`) | `Machine::load_rom(image, expansion_pak, saved, pak)` (`ffi/mod.rs:338–351`) | Pass no save. RetroArch fills the save buffer afterwards (§4.3) |
| `retro_run` | `input_poll`, then input into `press`/`set_stick`, then `advance`, then cheats, then `present`, then video, then audio | The order `MarsCore.RunFrame` keeps (`Mars_Native.md` §5.5) |
| Input | `press(port, mask, pressed)` and `set_stick(port, axis, value)` (`ffi/mod.rs:426–441`) | The shim's `PadButton`→joybus table (`Shim/MarsRtCore.cs:488–525`) applies unchanged, because `PadButton` is the RetroPad. Z is L2, the right stick drives the C buttons, and the stick scales ±32767 to ±127 with Y inverted |
| Audio | `ai.drain`, interleaved s16 stereo (`ffi/mod.rs:457–465`) | One `audio_batch` per `retro_run` |
| Save states | `state_size` / `save_state` / `restore_state` | The C# state format, byte for byte (`Mars_Native.md` §5.1). A RetroArch state loads in Mistress and the reverse |
| Options | Mars's nine keys: `ThreadedRdp`, `RdpWorkers`, `DeferredPresentation`, `SkipRepeatedScans`, `RenderScale`, `Antialiasing`, `Gpu`, `ExpansionPak`, `Recompiler` | Prefixed `marsrt_`. Categories "Video", "Performance" and "System" |
| `retro_get_memory_data(SYSTEM_RAM)` | RDRAM's raw allocation (`memory/ram.rs:33–37`) | Byte order: §4.6 |
| Dupes | `frame_info`'s serial (`ffi/mod.rs:482–489`) | An unchanged serial passes `NULL` |
| `retro_get_region` | The cartridge header's country code | As parallel-n64 does (`PN/libretro/libretro.c:1040–1057`) |
| `retro_reset` | Rebuild the machine from the ROM and the current save buffer | A hard reset, which the header allows (`libretro.h:8073`). MarsRT has no soft reset today |

### 4.3 What needs work

**1. The picture: format, lines and lifetime.** `frame_bytes` is RGBA in byte order (`ffi/mod.rs:491–503`), with alpha
0xFF. libretro's XRGB8888 is a native-endian `0x00RRGGBB` word, which is B, G, R, X in memory on every host MarsRT
builds for. The wrapper swaps red and blue into a buffer it owns and keeps until the next frame (§1.2). At one
multiple that is 1.2 MB a frame, about 0.1 ms at the 12 GB/s §6.13.4 measured. At four it is 19.7 MB, about 1.6 ms.
Composing BGRX in the scan-out itself would remove the pass, at the price of a second output format to test. That is
a later lever, not a first step.

**Lines.** The shim asks for rows repeated (bit 4, `RepeatRows`, `ffi/mod.rs:407–418`) because Mistress's filters were
written for it. The libretro build should not: 320×240 with `aspect_ratio` 4/3 is what RetroArch's shaders expect
(§2.9), and it halves the copy. `row_repeat` is already reported (`frame_info`).

**Geometry.** base 320×240; max 640×480 times the largest multiple, that is 2,560×1,920; `SET_GEOMETRY` when the
game's resolution changes.

**2. Audio rate.** The AI's rate is the game's DAC rate, 44,100 until the game sets one (`memory/ai.rs:67–82`). Two
routes, as §2.7 found:
- *Declare the rate*, with `SET_SYSTEM_AV_INFO` on a change. This is what the local parallel-n64 does. It costs a
  driver reinitialisation per change, which games make rarely.
- *Resample to 48 kHz in the wrapper.* That adds a resampler whose state is not machine state, and run-ahead's hidden
  frames then run it too.

The first is recommended. It keeps the samples exact, and it leaves the resampling to the frontend, which does it with
rate control in any case.

**3. The battery save.** MarsRT takes the chip's and the pak's bytes at load (`load_rom`'s `saved` and `pak`) and hands
out copies (`save_data`, `pak_data`, `ffi/mod.rs:539–567`). RetroArch wants one live buffer that it fills after load
(§2.4). The wrapper needs one of two things:
- **(a)** the save chip and pak backed by a wrapper-owned buffer in parallel-n64's layout, which is a change inside
  `memory/` to take a borrowed slice; or
- **(b)** a copy in each direction: the buffer into the machine before the first frame, and the machine into the
  buffer at the end of every `retro_run` in which `dirty()` is set.

(b) touches nothing in the machine and costs almost nothing, since `dirty()` exists. It is recommended. The four pak
slots follow: MarsRT has a pak on port 0 only (`ffi/mod.rs:552–567`).

**4. Save states under threads.** MarsRT has two kinds (`Mars_Native.md` §5.6.4):
- a **state**, which joins every RDP worker and writes v1;
- a **snapshot**, which holds the workers at a word boundary, writes v1 and a **fixed** 32k-word tail of the words not
  yet run, and resumes them. Its fixed tail is why rewind's chain works (`EmuSen_Rewind_And_FastForward.md`).

libretro wants one size per session, and bytes that are the same whenever the machine is the same. The snapshot's
tail depends on how far the workers had got when the hold landed. **So two runs to the same machine can write
different snapshots.** They load the same, and they hash differently, which netplay reads as a desync.

The proposal:
- `retro_serialize_size` returns the **snapshot's** size, which is constant for a machine
  (`mars_machine_save_state_size(core, 1)`).
- `retro_serialize` asks `GET_SAVESTATE_CONTEXT`.
  - Under `ROLLBACK_NETPLAY`, it **joins first and then writes a snapshot**, whose tail is then empty and whose bytes
    are determined.
  - Otherwise it writes a snapshot without joining. That covers rewind and user states, which both arrive as `NORMAL`,
    and run-ahead. It is the fast case §5.6.4 built for rewind.
- `retro_unserialize` accepts either kind. `restore_state` already detects the kind (`mars_machine_state_kind`).

This is argued. The claim to measure is that serialize-every-frame, which is what run-ahead does, leaves every frame's
state hash equal to a run without it. §8's test T3 is that measurement. One more reason to measure it: `save_state`
calls `settle()` before writing (`ffi/mod.rs:313`). If settling were not observationally neutral, a core that
serializes every frame would diverge from one that does not.

**5. The Expansion Pak and the fixed size.** A 4 MB and an 8 MB machine have different state sizes. A state of the
other size rebuilds the machine (`ffi/mod.rs:257–282`), which would change `retro_serialize_size` mid-session. RetroArch
cannot allow that (§2.3). The rules, argued:
- the Pak option takes effect at the next load, as it already does after the first frame (§5.5);
- `retro_unserialize` **refuses** a state of the other size, with a message through `SET_MESSAGE_EXT`.

The alternative, always sizing for 8 MB and padding a 4 MB state, would make a RetroArch state and a Mistress state
differ in length. It is rejected for that reason.

**6. Deferred presentation and the one-frame-late picture.** With presentation deferred, the picture shown after frame
*n*+1 is frame *n*'s (§5.6.5). Under libretro that is one frame of added latency, and it works against run-ahead, whose
purpose is removing latency. Run-ahead of *k* frames with the picture deferred removes only *k*−1. The shim's default
is on because the walk then overlaps the next frame (§6.2). The libretro default should be **off**, with the option
kept and its `info` saying what it costs. RetroArch users compare cores' latency, and a deferred core would look one
frame worse than it is.

**7. Frames that will not be seen.** `GET_AUDIO_VIDEO_ENABLE`'s video bit maps to `skip_rendering`
(`set_options` bit 0). The header's condition is that the next frame and the state come out identical (§2.3). **That
is not proven for MarsRT with the picture deferred.** `Prepare` advances the borders and the held lines on the
emulation thread (§5.6.5), and whether a skipped scan leaves them as a shown one would is exactly what test T4
measures. Until it passes, the wrapper renders every frame and skips only the copy and the callback.

**8. RDP worker threads.** They are the core's own threads, which is allowed (§1.2). They never call back, because the
native side never calls out (§3.2's rule, kept by every stage since). Two consequences follow:
- **Memory the frontend reads.** RetroArch reads `SYSTEM_RAM` between frames for achievements, and its cheat manager
  writes it (§2.6). A worker may still be drawing into RDRAM then. The shim's host reads and writes wait for the drain
  (§5.6.4). A raw pointer handed to C cannot.
  - *Proposal:* when achievements are on (`SET_SUPPORT_ACHIEVEMENTS` has been sent), or a cheat is active, the wrapper
    joins the drain at the end of `retro_run`.
  - The cost is unmeasured. That the join matters is argued from §5.6.1's rule, not observed.
- **Unload** joins and frees every worker and the presenter. `retro_deinit` must leave nothing running, because the
  library may stay loaded (`libretro.h:7984–7999`).

**9. The recompiler on platforms that forbid JIT.** `GET_JIT_CAPABLE` (74, `libretro.h:2458`) answers `false` on an
iOS build without JIT (`runloop.c:3650–3657`). The wrapper then keeps tier 2 and 3 off. What that costs is already
measured, on the desktop (`Mars_Native.md` §5.8.8, production):

| ms a frame | interpreted | tier 1, decoded (no executable memory) | tier 2, compiled |
| --- | --- | --- | --- |
| Super Mario 64 | 6.06–6.11 | 6.11–6.38 | 5.92–6.01 |
| Ocarina of Time | 7.15–7.19 | 7.44–7.50 | 6.92–6.98 |
| GoldenEye, the Dam | 17.87–17.95 | 14.95–15.14 | 14.95–14.99 |

Tier 1 compiles nothing: it is the block cache over the interpreter's handlers. It holds GoldenEye's whole gain. The
compiled tier adds three to four per cent in Super Mario 64 and Ocarina of Time, and nothing in GoldenEye. **So without
a JIT, MarsRT loses at most about four per cent on this desktop**, if the wrapper picks tier 1 for mapped-code games
and the interpreter otherwise, or simply tier 1 everywhere at a one-to-four-per-cent loss.

This does not generalise to the iPhone: the handheld and ARM figures are unmeasured, and Cranelift has never run on
`aarch64-apple-ios`. The local header also defines `RETRO_ENVIRONMENT_EXEC_MEM_ALLOC` (83, `libretro.h:2626`), which
lets a frontend provision JIT memory, dual-mapped if need be. **No code in RetroArch 1.22.2 implements it:** the name
appears only in the header. Cranelift's `cranelift-jit` allocates its own pages, so using it would mean a custom
`JITMemoryProvider`. That is a later question, and it rests on upstream implementing the call.

**10. The GPU path: its own device, or RetroArch's.** MarsRT's device path is compute only. It opens its own instance
and device through `ash` (`rdp/gpu/device.rs`), runs the five shared SPIR-V programs, and reads the picture back to
the host (`Mars_Native.md` §6.4.2). The two routes:

- **(a) Its own device, output in software.** The path runs unchanged, beside whatever video driver RetroArch uses:
  GL, D3D, Metal, or Vulkan.
  - Two devices on one GPU are legal in Vulkan. Nothing here measures what they cost.
  - The readback stays. `Mars_Gpu.md` §16 measured the copy and upload it would remove at about 3 ms a frame at four
    multiples, off the thread that bounds the frame rate, and did not build the interop.
- **(b) RetroArch's device.** `SET_HW_RENDER` with Vulkan, `create_device2` so that the device has a compute queue
  and the extensions the path needs, and `set_image` of the raster the device keeps (§6.4's average already keeps
  one).
  - The frame then never leaves the GPU, and RetroArch's shaders read it in place.
  - The costs:
    - it forces RetroArch's Vulkan driver (`gfx/video_driver.c:2992–3017`), so a user on GL gets a driver switch;
    - it needs a software fallback for when the context is refused, which is route (a) or the processor path anyway;
    - `lock_queue` around every submission from the RDP's thread;
    - a per-sync-index ring of images;
    - `context_reset` and `context_destroy` handled at any time, which MarsRT's device, carried across state reads by
      moving it (§6.4.2), does not now expect.

**The recommendation is (a) first**, for the reason §16 gives: at one multiple the emulation thread is the bound and
the device helps nothing, so (b) pays only at the multiple, and only where the readback is the bound. (b) is priced in
§8 as a separate stage, to be built only once (a) has measured the readback as the cost on a machine where it
matters, such as the handheld.

**11. A panic.** The release profile aborts on panic (`Cargo.toml`, `panic = "abort"`). Inside RetroArch that kills
the frontend, and the user's unsaved state with it. §2's reasoning still holds ("a panic in an emulator component is a bug"): unwinding through a machine whose
invariants have failed is not safe to continue. So the libretro build keeps abort. It installs a hook that writes the
message and the backtrace to `<save directory>/marsrt_crash.txt`, as `emusen_native_set_crash_log` does now
(`src/lib.rs:55–62`). The alternative, a `panic = "unwind"` build that catches at each export and stops the core, is
recorded and not recommended. §9 asks the user.

**12. Cheats.** `retro_cheat_set` receives GameShark text. The GameShark codec is C#, in DianaOS (`ICheatCodeCodec`,
`CoreFactory.cs:271–303`), so the wrapper needs a Rust decoder of the N64 GameShark format. That is a small, closed
format. It applies codes where `MarsCore.RunFrame` applies them (§5.5: after the frame, before the picture, while
Status.IE is set), through `cheat_write8`.

### 4.4 Options, key by key

| Key | Default in the libretro build | Category | Note |
| --- | --- | --- | --- |
| `marsrt_recompiler` | enabled (tier 2), forced to tier 1 when `GET_JIT_CAPABLE` is false | Performance | §4.3.9 |
| `marsrt_threaded_rdp` | enabled | Performance | Proven exact frame by frame (§5.6.7) |
| `marsrt_rdp_workers` | a third of the cores, 1–8 | Performance | The shim's default (§6.2) |
| `marsrt_deferred_presentation` | **disabled** | Performance | §4.3.6 |
| `marsrt_skip_repeated_scans` | enabled | Performance | |
| `marsrt_render_scale` | 1 | Video | Needs `max_width` of 2,560, so it is fixed in `av_info` from the start |
| `marsrt_antialiasing` | Off | Video | |
| `marsrt_gpu` | disabled | Video | Route (a) of §4.3.10 |
| `marsrt_expansion_pak` | enabled | System | Takes effect at the next load (§4.3.5) |

### 4.5 Rewind, run-ahead and netplay: the claim available

MarsRT's state is proven exact frame by frame against the C# core's, with threads, deferred presentation and the
recompiler on (`Mars_Native.md` §5.6.7, §5.8.5). A state holds no pointer: it is the C# format, fields in a fixed
order. Nothing it holds depends on the host's pointer width. So MarsRT meets `RUNAHEAD_SAME_BINARY`'s rule already,
and `ROLLBACK_NETPLAY`'s apart from that context's "big-endian integers" clause. No shipping N64 core is known to
honour that clause, and netplay's own handshake instead refuses a platform mismatch that a core declares through
`ENDIAN_DEPENDENT` (`netplay_frontend.c:9234–9237`).

**The `.info` can therefore claim `deterministic`**, which no N64 core on the buildbot does today (§2.10). The claim
is to be earned by §8's T3 and T5 before the file says so. Rewind is off in Mistress for the N64 until a snapshot is
proven in play (§5.5). In RetroArch the `.info` level decides, so the libretro build is where that proof in play would
first happen.

### 4.6 Achievements and RDRAM's byte order

MarsRT's RDRAM is big-endian: `be32` reads `u32::from_be` (`memory/ram.rs:40–53`). The existing N64 sets are written
against mupen's host-endian words (§2.5). **So MarsRT's `SYSTEM_RAM`, exposed as it is, would very likely read the
existing sets wrong.** This is inferred, not tested. Three responses:
- **(a)** A word-swapped copy of RDRAM, refreshed at the end of each `retro_run` and exposed as `SYSTEM_RAM`. It costs
  about 8 MB copied a frame, roughly 0.7 ms at §6.13.4's rate. Frontend writes to it would be lost, so memory cheats
  through RetroArch would be too.
- **(b)** RDRAM stored as host-endian words in MarsRT itself. That changes the byte order under every proven access,
  the recompiler's inline loads included. Rejected: it would reopen what §5.2 to §5.8 closed, to suit one consumer.
- **(c)** Report achievements unsupported until (a) has been checked against a known set, compared with
  parallel-n64's reads of the same addresses on the same state.

**(c), then (a)**, is recommended. It is test T6.

---

## 5. What the C# cores are, as libretro cores

**As they stand, Moon, Venus, Mercury and the C# Mars are not libretro cores, and should not be made into them.** A
libretro core is a native library that a C host loads, possibly twice (§2.3). A C# core in that position has two
routes, and both have measured costs or unmeasured risks.

- **Native AOT**, exporting the twenty-five functions through `UnmanagedCallersOnly`.
  - *Speed.* Measured slower: 10 to 12 per cent on Venus (`Venus_CPU.md` §9.3) and a fifth on Mars's interpreter:
    24.9 against 31.1 fps in Ocarina of Time, 29.4 against 36.4 in Super Mario 64 (`Mars_Performance.md` §25).
  - *No recompiler.* Mars's recompiler is `System.Reflection.Emit`, which Native AOT cannot run (§25), so AOT Mars is
    37 per cent behind the JIT with blocks.
  - *No save state.* Trimming leaves the reflection-walking serializer writing **48 bytes** for Mars (§25) and 20 for
    Venus (`Venus_CPU.md` §9.3, citing `EmuSen_Project_Overview_v2.md`). The serializer would have to be rewritten before one save state, let alone a
    deterministic one, could exist.
  - *Loading twice.* Whether two Native AOT libraries, each with its own runtime and garbage collector, coexist in one
    process as run-ahead's second instance needs is unverified.
- **Hosting the CLR** from a C shim through `hostfxr`.
  - *What it adds.* The runtime is either required on the user's machine or bundled at tens of megabytes per platform.
  - *Where it cannot go.* iOS and tvOS have no CoreCLR, and Android's is not what RetroArch's Android build could load.
  - *Loading twice.* The runtime is process-wide, so the second instance's copied library meets a runtime already
    loaded. That is inferred, and the run-ahead guide does not say.
  - *Pauses.* The garbage collector pauses inside `retro_run`. That is the pause `Mars_Native.md` §6.13 removed from
    Mistress at some cost.

Neither route buys anything the plan does not already buy. `EmuSen_Stack.md` §2.1 now says every core is to be ported
to Rust after Mars, with the C# core as the port's oracle. **A `<Name>RT` core is a libretro core through §7's wrapper,
at the cost of implementing one trait.** The ports are the route. Mercury's port to Rust is in progress at the time of
writing, in another worktree. The mapping it will need is §1's: 160×144, 59.7275 Hz (`MercuryCore.cs:58–68`), 8
buttons on port 0, a battery save, and GB/GBC as one core with the model chosen at load, not two subsystems
(`libretro.h:1414–1425`).

---

## 6. Screen filters

RetroArch's shaders replace EmuSen's filters inside RetroArch. There is little to lose. Serenity's CRT filter is a port
of libretro's own `crt-lottes` ("crt-lottes by Timothy Lottes, public domain, from libretro's slang-shaders",
`EmuSen.Serenity/Shaders/CrtFilters.cs:122–125`). Serenity also runs RetroArch `.slangp` presets directly
(`EmuSen.Serenity/Slang/`). The handheld LCD filters (`HandheldFilters.cs`) matter only to the 2D cores, and slang
equivalents of them were not surveyed. What the libretro build owes the shaders is §4.3.1's raster: native lines, the
right aspect, no filter applied.

---

## 7. One wrapper for every `<Name>RT`

The boilerplate is the same for every core: the twenty-five exports, the environment callback's plumbing, the option
chain with its three fallbacks, input polling, the pixel swap, the log, and the rules of §2.3. It should be written
once.

**`emusen-libretro`**, a library crate beside the cores (`EmuSen/Cores/Libretro/`), holds:

- **The bindings.** A hand-written `#[repr(C)]` subset of `libretro.h` and `libretro_vulkan.h`: the calls this
  document names and no others. That keeps the build free of `bindgen` and a C header search. The probe does use
  `bindgen` (`EmuSen.WiseMan/Reference/probe-rs/build.rs:52–78`), and its bindings are the cross-check in a test that
  compares struct sizes and constants.
- **The trait** each core implements:

  ```rust
  pub trait RetroCore: Sized {
      const INFO: SystemInfo;                              // name, version, extensions, need_fullpath
      fn options() -> &'static [OptionDef];                // from the core's CoreSetting list
      fn load(game: &[u8], env: &mut LoadContext) -> Result<Self, LoadError>;
      fn av_info(&self) -> AvInfo;
      fn set_option(&mut self, key: &str, value: &str);    // between frames
      fn run_frame(&mut self, input: &PadState, out: &mut FrameOut);  // picture, samples, geometry change
      fn serialize_size(&self) -> usize;                   // constant for the session
      fn serialize(&mut self, ctx: SaveContext, out: &mut [u8]) -> bool;
      fn unserialize(&mut self, data: &[u8]) -> bool;
      fn battery(&mut self) -> Option<&mut [u8]>;          // the stable SAVE_RAM buffer
      fn system_ram(&mut self) -> Option<RamView>;         // pointer, length, and whether it is safe between frames
      fn reset(&mut self);
      fn cheat(&mut self, index: u32, enabled: bool, code: &str) {}
      fn region(&self) -> Region;
  }
  ```

- **A macro**, `export_core!(MarsRtCore)`. It generates the twenty-five `#[unsafe(no_mangle)] extern "C"` functions
  over one `static` slot that holds the callbacks and `Option<T>`. `retro_deinit` resets the slot, as `libretro.h:7994`
  requires.
- **What the crate does for every core**, so that no core gets it wrong alone:
  - the calls in the right places: options in `retro_set_environment`, and pixel format and descriptors in `load`;
  - `GET_INPUT_BITMASKS` when offered;
  - RGBA to XRGB8888;
  - `NULL` dupes from an unchanged serial;
  - `GET_AUDIO_VIDEO_ENABLE` and `GET_SAVESTATE_CONTEXT` read once per frame and handed to the core;
  - the size check that refuses a buffer smaller than the size reported;
  - the log interface, with stderr as its fallback;
  - the panic hook;
  - a crate feature that, when set, forbids executable memory and reports it to the core.

**What stays per core:** the button table, the option list, the save layout, the memory map, and the cheat format. For
MarsRT that is §4. For a MercuryRT it is a quarter of it, with no threads, no GPU and no JIT.

**The packaging consequence.** The buildbot's template builds `lib${CORENAME}` at a repository's root (§3.2). A
per-core crate named `<name>` at the root of its own mirror repository fits that shape. EmuSen's monorepo, with a path
containing spaces, does not. There are two routes:
- per-core mirror repositories that vendor `emusen-libretro` and the core's crate;
- a custom `.gitlab-ci.yml` script rather than the template's.

Upstream decides between them, so §9 asks.

**Existing crates.** `libretro-rs`, `rust-libretro` (with `rust-libretro-sys`) and `libretro-backend` all exist
(https://github.com/libretro-rs/libretro-rs, https://lib.rs/crates/rust-libretro,
https://github.com/koute/libretro-backend). None was evaluated here: their licences, API coverage (v2 options, Vulkan
negotiation, the savestate context) and maintenance are unchecked. The proposal writes its own because the surface
this document needs is small, and because the save-state and threading rules of §2.3 and §4.3 are where a
general-purpose wrapper would have to be overridden anyway. That is argued, and adopting one of them is the
alternative.

---

## 8. A staged plan for MarsRT, with its tests

Each stage names its oracle before it begins. The effort figures are estimates in the unit this project's record
supports: `Mars_Native.md` §5.5, the shim's frontend stage, and §6.3, CI on four platforms, were each built within a day,
by their dates. Hold them loosely, as §5 holds its own.

| Stage | What | Oracle | Estimate |
| --- | --- | --- | --- |
| **L0** | Decisions: §9's questions answered, the crate's place, the core's name | — | an hour with the user |
| **L1** | `emusen-libretro`: bindings, trait, macro, option chain, input, pixel swap, log, panic hook; a synthetic test core (a colour-bar machine) | the probe (T1) and a struct-layout test against `bindgen` | 1–2 days |
| **L2** | `marsrt-libretro`, minimum: load, run, video (native lines, BGRX), audio with `SET_SYSTEM_AV_INFO`, input, hard reset, battery buffer by copy, states (§4.3.4), the Expansion Pak refusal | T2, T3 | 1–2 days |
| **L3** | Options and categories, descriptors and controller info, dupes, `GET_AUDIO_VIDEO_ENABLE` behind T4, `GET_JIT_CAPABLE`, the GameShark decoder, the drain join for memory consumers | T4, T5 | 1–2 days |
| **L4** | Achievements behind T6; the `.info` file | T6 | half a day, and longer if (a) of §4.6 is needed |
| **L5** | CI: add `marsrt_libretro.{so,dll,dylib}` to `.github/workflows/marsrt.yml`'s four runners; later the buildbot's `.gitlab-ci.yml` | the tests run on each runner, as §6.3's do | half a day, plus upstream's time |
| **L6** (optional) | Vulkan through RetroArch's device (§4.3.10 b) | the processor path at the same multiple, picture for picture, as `Mars_Gpu.md` grades the device | several days; built only on measurement |

**Tests, all headless.** CLAUDE.md's rule, that tests run through WiseMan and not a window, applies. The probe is the
host.

- **T1, the contract.** `probe-rs` already loads any libretro core with `dlopen` (`src/backends/libretro.rs:593–612`),
  checks the API version, and answers pixel format, directories, `GET_CAN_DUPE`, logs and the option calls
  (`:89–165`).
  - It needs to grow: `GET_VARIABLE` answering a given set of options (today it returns `NULL` on purpose,
    `:113–116`), `GET_SAVESTATE_CONTEXT`, `GET_AUDIO_VIDEO_ENABLE`, `GET_INPUT_BITMASKS`, `GET_JIT_CAPABLE` false on
    request, serialize and unserialize, and a second `dlopen` of a copied library, as run-ahead does.
  - T1 is the list of §1.1's rules, checked against the synthetic core and then MarsRT.
- **T2, one machine, two hosts.** Super Mario 64 and Ocarina of Time, with a scripted input, run 600 frames through
  the libretro core in the probe and through `MarsRtCore` in WiseMan. **Their save states must be byte-identical after
  every frame**, since both write the C# format. That is the oracle of the whole port, applied to a new host. The
  picture is compared after the swap back.
  - Other games stay out: the N64 probe grades only these two, by standing instruction.
- **T3, serialization does not perturb.** The same run, with `retro_serialize` called before every frame (run-ahead's
  pattern) and with a serialize, unserialize and rerun of each frame (rollback's pattern). Every frame's state hash
  must equal T2's. Both kinds are covered, the joined state and the snapshot.
- **T4, skipping a picture.** Frames with the video bit cleared at random. Each shown frame's picture and each frame's
  state must equal a run that showed every frame, with presentation deferred and immediate.
- **T5, netplay's bytes.** Under `ROLLBACK_NETPLAY`, two runs of the same inputs on different worker counts (1 and 4)
  must write byte-identical states every frame. Without the join of §4.3.4 this is expected to fail. Showing that it
  does is the positive control: a test that cannot fail proves nothing.
- **T6, achievements.** The word-swapped view (§4.6 a) and parallel-n64's `SYSTEM_RAM`, read at the same addresses from
  the same state, must agree on every aligned word.
- **RetroArch itself, once.** RetroArch accepts `-L <core> <rom> --max-frames N --max-frames-ss --verbose`
  (`retroarch.c:7442–7491`) and has `null` video and audio drivers (`gfx/video_driver.c:349`,
  `audio/audio_driver.c:139`). So a build of the local checkout can run the core with no window, and its log shows
  which environment calls were answered.
  - Whether the null video driver takes the screenshot `--max-frames-ss` asks for is unverified.
  - This is a smoke test, not the oracle.

---

## 9. Open questions

**For the user.**
1. **The name in the Online Updater.** The proposal is `corename = "MarsRT"`, `library_name = "MarsRT"`, file
   `marsrt_libretro`, and `display_name = "Nintendo - Nintendo 64 (MarsRT)"`. The name is hard to change later:
   rcheevos keys its per-core rules by `library_name` (§2.5), and RetroArch keeps per-core configuration under the
   core's name (not re-read here).
2. **Whether to submit upstream,** or to build the libretro core for local use and EmuSen's own releases only.
   Submission means a mirror under libretro's GitLab, an `.info` pull request, and the buildbot's toolchain, whose
   `rustc` version is unknown (§3.2).
3. **iOS and tvOS.** Whether to give libretro's App Store build the approval it requires (§3.4), knowing that MarsRT
   would run there without its JIT (§4.3.9) and that nothing has been measured on an iPhone. tvOS has no Rust template,
   and whether `aarch64-apple-tvos` is usable without a nightly toolchain was not checked.
4. **Panic behaviour** in RetroArch: abort with a log, as recommended, or unwind and stop the core (§4.3.11).
5. **Deferred presentation's default** in the libretro build: off, as recommended (§4.3.6).

**Upstream, unverified here.**
6. The `libretro-build-rust` image's Rust version, and whether Cranelift 0.136 builds in it.
7. Whether `SET_SERIALIZATION_QUIRKS` settles at 44 or 87 (§2.3).
8. Whether RetroArch will implement `EXEC_MEM_ALLOC` (§4.3.9).
9. Whether the existing RetroAchievements N64 sets assume mupen's word layout (§2.5, §4.6). The inference is strong,
   and it is still an inference.

**To measure.**
10. T3 and T4 (§8): whether serializing and skipping pictures leave MarsRT's states unchanged.
11. What joining the drain at every frame's end costs when achievements are on (§4.3.8).
12. Two Vulkan devices on one GPU, MarsRT's and RetroArch's, against route (b) (§4.3.10).

**Not covered.** The 64DD, the Transfer Pak and rumble were not surveyed for MarsRT. `SET_CONTROLLER_INFO` would list
them when they exist. RetroArch's own dynamic rate control, and what it does with a core whose declared rate changes,
was not read beyond the header.
