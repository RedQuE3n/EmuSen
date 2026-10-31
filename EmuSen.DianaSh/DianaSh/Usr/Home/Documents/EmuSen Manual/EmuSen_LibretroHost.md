# EmuSen_LibretroHost — libretro cores run as EmuSen cores

*Design, 2026-10-31. Proposed; nothing is built.* Decided by the tester the same day: "Let's go the libretro route, that
way EmuSen can consume other cores." This widens `EmuSen_LibretroBridge.md` (a stopgap for the PlayStation 2, proposed
2026-10-27) from one core to any libretro core, and is the inbound direction `EmuSen_CoreAPI.md` §14.1 names and §15 Q6
decided in principle. The outbound direction, a v1 core offered to RetroArch, keeps the name `emusen-libretro`
(`EmuSen_Libretro.md` §7); this component is `emusen-libretro-host`. PCSX2's libretro build (`pcsx2_libretro`, the
nightly pinned in `EmuSen.WiseMan/Reference/libretro-cores.sha256`) is the first target.

The marks are those of `EmuSen_Gpu.md`: **[M]** measured, with the date and tool; **[R]** read from a source; **[A]**
argued; **[C]** a choice.

**Sources** [R, read 2026-10-31]: `libretro.h` and `libretro_vulkan.h` from libretro-common's `master`
(<https://github.com/libretro/libretro-common/tree/master/include>, MIT, fetched 2026-10-31; interface version 5 and
negotiation version 2), the only libretro text read; `emusen_core.h` 1.15; `EmuSen_CoreAPI.md` §6.0–§6.22, §7 and §14.1;
`EmuSen_Gpu.md` §5; `EmuSen_DevicePicture.md` §1.3 and §4; `Platform/serenity/src/vulkan.rs` (the lender); the trait of
`emusen-native`'s `core` module. No emulator's source, PCSX2's included, is read for this work.

## 1. The form: a component of its own

Weighed under the project's rule for features (an extension, a module, or a component):

- **Not an extension of a frontend.** Sessions, states, saves, settings and media already cross the core ABI, so a
  libretro core hosted *below* it reaches Mistress, big picture and the shell with no per-frontend code (§2.4 of
  `EmuSen_CoreAPI.md`: no per-core code above the ABI).
- **Not a module of an existing core.** It is a second job for every core there is.
- **A component of its own** [C]: one Rust `cdylib`, `emusen-libretro-host`, that exports the v1 surface through
  `emusen-native`'s `core_exports!` on one side and loads one libretro core with `dlopen` on the other. To the frontend
  it is an ordinary v1 core. Its callbacks from the libretro core land in its own buffers while `retro_run` runs, and
  are pulled by the host after `advance` as any core's results are, so §6.0's first rule (the library never calls the
  host) holds.

### 1.1 One libretro core per copy of the adapter

libretro's callbacks carry no user pointer, so a libretro core is process-global by design [R, libretro.h]. Two
consequences [A]:

1. **One machine per library.** A second `create` while a machine lives is refused with a status of the core's band
   (`-9`, "the core already runs a machine"). Run-ahead by a second instance is therefore not offered, and the kit's
   cases that hold two machines at once (C8, C10) run their machines one after the other.
2. **One copy of the adapter per libretro core.** Discovery keys one sidecar, one `info()` and one approval to one
   library file (§7.1). glibc's `dlopen` returns the handle already loaded for a path that names the same file
   (device and inode), so a symbolic or hard link would make two libretro cores share one adapter and its globals.
   Each installed core therefore gets a folder of its own under the player's `/Cores`:

```
/Cores/libretro/pcsx2/
    emusen-libretro-host.so          a copy of the project's adapter (hash in cores.manifest)
    emusen-libretro-host.so.core.json   the sidecar, written by loading it once after approval
    pcsx2_libretro.so                the libretro core, as downloaded
    pcsx2_libretro.info              its published core information
    NOTICE                           the core's name, licence, source address and SHA-256
    assets/                          any data files the core needs beside firmware (§3.8)
```

The adapter finds its libretro core as the one `*_libretro.<ext>` in its own folder (found from its own address with
`dladdr`, `GetModuleFileNameW` on Windows) and refuses to load it unless its SHA-256 is the one `NOTICE` records.

### 1.2 The crate

`EmuSen/Cores/Libretro/emusen-libretro-host/` [C], one job per module: `retro` (the bindings, written by hand from
`libretro.h` for the subset used, with layout tests), `env` (the environment call), `video`, `audio`, `input`, `saves`,
`state`, `options`, `dirs` (the private directories), `profile` (per-core data, §3.5), `vulkan` (stage 2), and a
fifteen-line C file for the one variadic callback, the log (stable Rust cannot define a variadic function; the C
formats with `vsnprintf` and hands a finished line to Rust). Beside it, `emusen-libretro-testcore/` (§8). Dependencies:
`emusen-native`, `libloading`, `cc` (for the log shim), and from stage 2 `ash` 0.38, the version `emusen-gpu` and
Serenity use. The probe's libretro backend (`Reference/probe-rs/src/backends/libretro.rs`) keeps its own bindings for
now; the adapter's `retro` module is the one meant to be shared, and moving the probe onto it is a later, separate step.

## 2. Where the v1 ABI falls short, and what is proposed

Everything a software-rendered libretro core needs maps onto v1.15 except the four items below. **No export, struct
field or meaning changes**; each proposal is an addition under §4.2 of `EmuSen_CoreAPI.md`.

| # | libretro needs | v1.15 gives | Proposed |
|---|---|---|---|
| E1 | a **path** to the game for a core with `need_fullpath` (PCSX2 reads a DVD image of up to 8.5 GB by path) | the image's bytes only | **minor 1.16, `IMAGE_PATH`** (capability bit 26): `emusen_create_params.image_path` and `emusen_media.image_path`, `const char *` UTF-8, appended (create 120 bytes, media 56). A host fills it for a core claiming the bit whenever the bytes it passes are an unaltered file's (no decompression, no soft patch), and may then lend the image mapped (`BORROW`) instead of reading it. Null otherwise. An older host leaves it absent: the adapter then writes images under 512 MiB to its private directory and refuses larger ones with words saying why |
| E2 | options whose cost **nobody has classified** | `effect` must be `none`, `exact`, `accuracy`, `latency` or `enhancement`, with an accurate default for the last two (§6.13) | **the same minor: `effect: "unrated"`** (the field is an open enumeration). A host groups such settings under the core's own categories, labelled as the core's options, with no claim about cost. A per-core profile may rate known options (§3.7) |
| E3 | the picture of a **hardware-rendered** core on a device the host can read without a copy to memory | `DEVICE_PICTURE` (1.9): the host lends its device and the core draws on it | **nothing for stage 2**: the picture is read back and handed by `frame_copy` (§5.4). A later minor, `EXPORTED_PICTURE`, only if measurement shows the read-back matters |
| E4 | **rumble** (`GET_RUMBLE_INTERFACE`) | none; event kind 7 is kept for a controller's outputs | refused until `EmuSen_CoreAPI_Extensions.md` §4 is built, then mapped onto it |

**Outside the core ABI**, discovery and trust (`emusen-core-host`, the frontend API) learn one thing: a folder of
§1.1 is one entry whose approval names *both* files, the adapter by the manifest and the libretro core by its hash,
licence and author as `NOTICE` records them. A changed libretro core is a new entry and is asked about again (§7.2,
rule 2). This is an addition to the frontend API's minor.

**What v1's rules do not promise for these cores**, named rather than hidden [A]: core info's `deterministic` is false
unless a core's profile says otherwise after the kit's C8 has passed on it; a foreign core's defaults are its author's,
not the accurate configuration §6.13 binds the project's cores to; and its own threads, signal handlers and file I/O
are its own (§4).

## 3. The mapping, call by call

### 3.1 Lifecycle and the frame

| EmuSen | libretro |
|---|---|
| any library-level call, the first time (`info`, `settings_schema`, `firmware_for`) | once per process, under a `OnceLock`: load the core, check its hash, `retro_set_environment`, `retro_get_system_info`, `retro_init` (options may be declared in either), private directories made (§3.5). Answers are cached |
| `info()` | `retro_system_info` (name, version, extensions, `need_fullpath`) and the `.info` file (system, licence, authors, firmware): system id through a small table from libretro's `systemid` to EmuSen's (`playstation2` → `ps2`); an unknown one is offered under its own name |
| `create` | callbacks installed; firmware and battery files written into the private directories; `retro_load_game` (path from E1, or data when `need_fullpath` is false, lent if the host lends); save RAM copied in; `retro_get_system_av_info`; each port's device set; from stage 2 the Vulkan context made and `context_reset` called |
| `free` | `context_destroy`, `retro_unload_game`. `retro_deinit` is never called, since a library is never unloaded (§6.16); a second `create` after `free` loads again, which stage 1 tests on the test core and on PCSX2 |
| `advance` | one `retro_run`; a `SHUTDOWN` request turns the next `advance` into a status |
| `reset` (`RESET`) | `retro_reset` |
| `set_options` bit 0 | `GET_AUDIO_VIDEO_ENABLE` answers video off for that frame |
| `machine_info` | `retro_system_av_info`, the input descriptors and controller info, the memory maps |
| `frame_rate` | `timing.fps` as a ratio, by continued fractions with a denominator under 100,000 (59.94005994 → 60000/1001) |

### 3.2 Video

| libretro format | handed as | work in the callback |
|---|---|---|
| `XRGB8888` | `BGRA8888` where the host accepts it, else `RGBA8888` | one copy, the X byte written as 255 (or a swizzle) |
| `RGB565` | `RGB565` where accepted, else `RGBA8888` | one copy (or an expansion) |
| `0RGB1555` (the default) | `RGB565` where accepted, else `RGBA8888` | a conversion |

`video_refresh`'s pointer is valid only during the call, so the picture is copied there, row by row from the core's
pitch into the adapter's buffer at the host's stride, into a buffer allocated once at `max_width × max_height` and
never per frame. A null pointer (a duplicate frame, since `GET_CAN_DUPE` answers true) leaves the picture as it was,
and `FRAME_SERIAL` is claimed so that the serial moves only when the picture does. `SET_GEOMETRY` and
`SET_SYSTEM_AV_INFO` raise `GEOMETRY` (and `AUDIO_RATE`, and `MACHINE_INFO` for a new frame rate); the aspect ratio
(`base_width/base_height` when not positive) becomes `aspect_num/den` by the same continued fraction. `SET_ROTATION` is
refused (v1 has no rotated picture; a core then draws unrotated or rotates itself).
`GET_CURRENT_SOFTWARE_FRAMEBUFFER`, which would save the copy, is refused until the copy is measured to matter.

### 3.3 Sound

`audio_sample_batch` and `audio_sample` append interleaved stereo `int16` to a queue of runs, each run tagged with the
rate it was made at, so that `audio_drain` never crosses a rate change (§6.7). The rate reported is `timing.sample_rate`
rounded to the hertz; the adapter does **not** resample [C], since the frontend's resampler is the one owner of rate
conversion and its rate control absorbs the rounding (32,040.5 Hz rounds by 16 parts per million; PCSX2 reports a
whole 48,000 [A, to be confirmed by machine info at stage 1]). `set_audio_limit` bounds the queue, oldest dropped.
`SET_AUDIO_CALLBACK` (sound pulled on the frontend's thread) and `SET_AUDIO_BUFFER_STATUS_CALLBACK` are refused, and
cores fall back to the batch path [R].

### 3.4 Input

The canonical controls are the RetroPad's ids in the RetroPad's order (`EMUSEN_CONTROL_B` = 0 … `R3` = 15, as
`PadButton` was made), so a port's bit *n* is `RETRO_DEVICE_ID_JOYPAD` *n* with no table. `input_state` answers from
what `set_buttons` and `set_axis` latched before `advance`; `GET_INPUT_BITMASKS` answers true, and
`RETRO_DEVICE_ID_JOYPAD_MASK` gets the whole mask. Axes: a stick's −1…1, down and right positive, is libretro's
convention too [R], scaled by 32,767 and rounded half away from zero; a trigger's 0…1 becomes `ANALOG_BUTTON` L2/R2 at
0…32,767, and any other analog button reads 0 or 32,767 from its digital bit. Each port is set to the device the core's
`SET_CONTROLLER_INFO` lists as its pad with sticks (`RETRO_DEVICE_ANALOG` or a subclass of it), or `JOYPAD`; `PRESENCE`
(claimed) sets `RETRO_DEVICE_NONE`. Button labels from `SET_INPUT_DESCRIPTORS` ("Cross", "Circle") become machine
info's controller labels. Keyboard, mouse, light gun and pointer read zero until device kinds exist
(`EmuSen_CoreAPI_Extensions.md` §10).

### 3.5 Saves and memory cards

- **Save RAM** (`RETRO_MEMORY_SAVE_RAM`) is battery file 0, `.srm`, kind `game`: copied into the core after
  `retro_load_game`, as libretro asks, and copied out by `battery`. `BATTERY_DIRTY` is claimed: `battery` compares
  against the copy last marked saved, so nothing is done per frame. `RETRO_MEMORY_RTC` is file 1, `.rtc`.
- **The save directory.** §7.2 forbids the runtime to give a core a path to write to, so no host path is given
  [C]. The adapter makes its own private directory (`$XDG_RUNTIME_DIR/emusen-libretro/<pid>-<core>/save/`, mode 0700,
  removed at `free`; the platform's temporary folder elsewhere) and answers `GET_SAVE_DIRECTORY` with it. Before
  `retro_load_game` it writes there the battery files the host passed; after each frame, at most once a second, it
  checks the files' sizes and times and raises `BATTERY` for those that changed. **Which file is which battery file**
  is per-core data, the core's *profile*, found by running the core as a black box and listing what it writes:
  PCSX2's memory cards become files of kind `removable`, medium `ps2.card`, ports 0 and 1, so that the player's cards
  behave as Sapphire's do (the console's memory shared by every game, `EmuSen_SaveMedia.md` §3.6), and whether
  Sapphire's card image and the core's are the same bytes is checked before the two share a card. Everything else the
  core writes there is kept as one battery file of kind `console`, an uncompressed archive of the folder, so nothing a
  core saves is lost for want of a profile.
- **States and shared media.** Under `CREATE_SHARED_REMOVABLE`/`CONSOLE` a state load must leave the cards as they
  stand (§6.27). Whether a core's `retro_unserialize` rewrites its cards is tested per core; where it does, the
  adapter writes the cards back after the load, and where that cannot work the exception is named in the profile.

### 3.6 States

Kind 0 is a 32-byte header (magic `ESLR`, header version, the libretro core's name and version hashed, payload length)
followed by `retro_serialize`'s bytes, so that another core's state is `FOREIGN` and another version's is
`STATE_VERSION` rather than a crash. A failed load must change nothing, which `retro_unserialize` does not promise, so
the adapter serializes the machine first and restores that copy when the load fails [C]; the cost is one save per load,
never per frame. `state_size` is the header plus `retro_serialize_size`, read again after each frame only when the core
declares `CORE_VARIABLE_SIZE` (raising `STATE_SIZE`), and the adapter declares `FRONT_VARIABLE_SIZE` back
(`SET_SERIALIZATION_QUIRKS`). `state_layout` names the header's fields and one opaque payload. `GET_SAVESTATE_CONTEXT`
answers `NORMAL`. `SNAPSHOT` is **not claimed** [C]: §6.4 derives machine info's kinds from the bit, so it cannot be
withdrawn for one core, and run-ahead and rewind need determinism these cores do not promise. A core's
`SINGLE_SESSION` quirk is written into machine info's `state` so that a frontend does not offer such states across
sessions.

### 3.7 Core options

`SET_CORE_OPTIONS_V2` (and v1 and the older `SET_VARIABLES`, converted) become the schema at library level: `key` the
option's key (libretro's keys fit `[A-Za-z0-9_.-]+`; one that does not is refused with a log line), `label` and `help`
its description and information, `kind: choice` with its values and labels, `default` its default, `category` its
category's description, `scope: run` (libretro applies options between frames through `GET_VARIABLE_UPDATE`), and
`effect: "unrated"` (E2) unless the profile rates the option. Values reach the core by `GET_VARIABLE`; `set_settings`
marks them changed for the next `GET_VARIABLE_UPDATE`. `SET_VARIABLE` (the core changing its own option) updates the
value and is reported through `setting_notes`. Options a core hides at run time (`SET_CORE_OPTIONS_DISPLAY`, the update
callback) are shown with a note rather than hidden, since the schema is fixed for a library. Per-console settings in
the frontend are the existing ones (`ICoreSettings` and the schema engine); nothing per core is written there.

### 3.8 Firmware and the system directory

`firmware_for` answers from the `.info` file's firmware entries (path, description, whether optional) and the MD5s its
notes list, so the firmware window recognises the player's file by content (§6.32). At `create` the files the host
passes are written under the private `system/` directory at the paths the `.info` names, mode 0600, and removed at
`free`. They are the player's own files, given by the existing firmware settings; nothing is fetched or shipped.
Data files a core needs that are not firmware (`assets/` of §1.1, downloaded with the core at the player's request
where its `.info` or its first run asks for them) are linked into the same directory. `GET_SYSTEM_DIRECTORY` and
`GET_CORE_ASSETS_DIRECTORY` answer with it; `GET_CONTENT_DIRECTORY` with the image's folder when E1 gives a path.

### 3.9 The environment calls

**Answered**: `GET_CAN_DUPE` (true), `SET_PIXEL_FORMAT`, `GET_SYSTEM_DIRECTORY`, `GET_SAVE_DIRECTORY`,
`GET_CORE_ASSETS_DIRECTORY`, `GET_CONTENT_DIRECTORY`, `GET_LIBRETRO_PATH`, `SET_INPUT_DESCRIPTORS`,
`SET_CONTROLLER_INFO`, `GET_INPUT_BITMASKS`, `GET_INPUT_MAX_USERS` (machine info's ports), `GET_VARIABLE`,
`GET_VARIABLE_UPDATE`, `SET_VARIABLES`, `SET_VARIABLE`, `GET_CORE_OPTIONS_VERSION` (2), `SET_CORE_OPTIONS`,
`SET_CORE_OPTIONS_V2`, the `_INTL` forms (the English text taken), `SET_CORE_OPTIONS_DISPLAY` and its callback,
`SET_GEOMETRY`, `SET_SYSTEM_AV_INFO`, `GET_LOG_INTERFACE` (the log queue, §6.18, from any thread),
`SET_MESSAGE` and `SET_MESSAGE_EXT` with `GET_MESSAGE_INTERFACE_VERSION` (1) (log records of category `message`, which a
frontend may show), `GET_PERF_INTERFACE` (the host's clock: such a core's pacing is its own), `SET_FRAME_TIME_CALLBACK`
(called each frame with the *nominal* frame time, never a measured one, so that the core runs the same at any speed),
`GET_AUDIO_VIDEO_ENABLE`, `GET_FASTFORWARDING` (false), `GET_THROTTLE_STATE` (unthrottled at the core's rate),
`GET_TARGET_REFRESH_RATE` (the core's own rate), `GET_LANGUAGE` (English until the frontend's language is passed),
`SET_SUPPORT_NO_GAME` (core info's `starts_empty`, 1.11), `SET_MEMORY_MAPS` and the `RETRO_MEMORY_*` regions (memory
spaces, and the shape kept for achievements, §6.22), `SET_SERIALIZATION_QUIRKS`, `GET_SAVESTATE_CONTEXT`,
`GET_GAME_INFO_EXT`, `SET_CONTENT_INFO_OVERRIDE`, `GET_JIT_CAPABLE` (true), `SET_PERFORMANCE_LEVEL`,
`SET_SUPPORT_ACHIEVEMENTS`, `SET_PROC_ADDRESS_CALLBACK` (recorded), `SET_SHUTDOWN`, `SET_MINIMUM_AUDIO_LATENCY`
(accepted and ignored), and from stage 2 the four hardware-rendering calls of §5.

**After stage 1**: `SET_DISK_CONTROL_INTERFACE` and `_EXT` onto `MEDIA` (lid and insert; needs E1's path), the first
addition once software games run.

**Refused** (answered false, as libretro specifies for a frontend that lacks a call): `SET_ROTATION`, `GET_OVERSCAN`,
`GET_USERNAME` (nothing of the player's is handed to foreign code), `SET_KEYBOARD_CALLBACK`, `SET_AUDIO_CALLBACK`,
`SET_AUDIO_BUFFER_STATUS_CALLBACK`, `GET_RUMBLE_INTERFACE` (E4), the sensor, camera, location, LED, MIDI, microphone
and device-power interfaces, `GET_VFS_INTERFACE`, `SET_SUBSYSTEM_INFO` loads, `GET_CURRENT_SOFTWARE_FRAMEBUFFER`,
`SET_FASTFORWARDING_OVERRIDE`, `EXEC_MEM_ALLOC`/`FREE`, `SET_NETPACKET_INTERFACE`, the HDR, VR, video-view, video-decode
and upscaler calls, OpenGL, OpenGL ES and Direct3D contexts, and every command the adapter does not know.

## 4. Threads, the process, and what a foreign core may do

- **Callbacks** reach one process-global record, set for the machine's life (one machine per library makes that
  sound); `video_refresh`, `audio_*`, `input_*` and the environment arrive on the thread inside `retro_run`, which is
  the host's emulation thread inside `advance`. The log and the Vulkan interface may be called from the core's own
  threads and are guarded (§5.5).
- **Library-level calls** are safe from any thread (§6.16): the one-time initialisation is behind a `OnceLock`, and
  later answers are cached text.
- **What v1 asks of a core that this one cannot promise** (§6.16): a libretro core runs its own threads, may install
  signal handlers (a recompiler's fast memory traps faults with `SIGSEGV`), and does its own file I/O in the private
  directories. The signal handlers are the real hazard: the .NET runtime that runs Mistress handles `SIGSEGV` itself,
  where RetroArch, written in C, does not. Stage 1 runs PCSX2 inside a .NET host (a WiseMan test) before anything
  else; if the two fight, the remedy already designed is the broker of §7.3, the core in a helper process, at the
  cost of one copy of each picture.
- **A crash in foreign code** ends the process, as §7.2 states for any core; the crash log names the adapter and the
  libretro core.

## 5. Hardware rendering through Vulkan

### 5.1 What libretro asks

A Vulkan core calls `SET_HW_RENDER` with `RETRO_HW_CONTEXT_VULKAN`, usually after `GET_PREFERRED_HW_RENDER`, and may
set a negotiation interface (`SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE`). With version 2 of it the core supplies
`create_instance` and `create_device2` and builds its own `VkDeviceCreateInfo` (features, extensions, queues), which it
hands to a wrapper of the frontend's that may add what the frontend needs; "the frontend must not disable features
requested by application" [R, libretro_vulkan.h]. The core's queue must support graphics and compute [R]. After
creation the frontend calls `context_reset`, and the core takes `retro_hw_render_interface_vulkan` (version 5) by
`GET_HW_RENDER_INTERFACE`: the device, the queue and its lock (`lock_queue`, `unlock_queue`), `set_image` (the frame's
image, its view, layout, the semaphores to wait for and its queue family), the frame-in-flight index
(`get_sync_index`, `get_sync_index_mask`, `wait_sync_index`), `set_command_buffers` and `set_signal_semaphore`. A frame
is handed by `video_refresh(RETRO_HW_FRAME_BUFFER_VALID, w, h, 0)`.

### 5.2 Why not the lent device as it stands

The plan of 2026-10-31 named `emusen-gpu`'s lent device (`EmuSen_Gpu.md` §5). It cannot serve a libretro core as
built, for three reasons:

1. **The queue.** The lender (Serenity) lends a queue of a compute family and keeps the graphics queue for itself
   (`lend_queues` in `vulkan.rs` [R]); the RX 6800 under RADV has one queue in its graphics family
   (`EmuSen_DevicePicture.md` §1.3 [M]). libretro requires graphics and compute on the core's queue.
2. **The features.** The lent device is made at the frontend's start with timeline semaphores and the external-memory
   extensions only; a negotiating core names its features at `retro_load_game`, and the frontend may not refuse them.
3. **A shared queue** behind a lock across the ABI would be a callback, which `EmuSen_DevicePicture.md` §5.2 rejected.

### 5.3 The design: the adapter's own device, on the host's card

[C] The adapter makes the device the core asks for, on the physical device the host draws with, and owns it for the
machine's life:

1. **The card.** Where the host lends a device (the adapter claims `DEVICE_PICTURE` for this purpose only), the
   physical device whose `deviceUUID` matches the lend's; otherwise `emusen-gpu`'s choice (`EMUSEN_GPU_DEVICE`, then
   its ranking), exposed for this as a small addition to that crate (version 1.4), so that the choice of card keeps one
   owner.
2. **The instance and device.** Through the core's `create_instance` and `create_device2` where it offers them, the
   adapter's wrapper adding `VK_KHR_external_memory_fd`, `VK_KHR_external_semaphore_fd` and timeline semaphores where
   offered (for §5.4's second step) and nothing that changes behaviour; through v1's `create_device` otherwise; made by
   the adapter itself, with one queue of the graphics family, for a core with no negotiation interface. Then
   `context_reset`.
3. **The interface** the core receives is the adapter's: the queue's lock is a mutex in the adapter; two frames in
   flight (`get_sync_index_mask` 0b11), each with its fence, which `wait_sync_index` waits for; `set_command_buffers`
   queued for submission at the frame's end.
4. **Each frame**, after `retro_run`, one submission on the emulation thread under the queue's lock: wait for the
   core's semaphores, take the image from its layout (and queue family) to a transfer source, copy the frame's
   rectangle into a host-visible buffer (`vkCmdCopyImageToBuffer` for `R8G8B8A8`/`B8G8R8A8`, a blit to `R8G8B8A8`
   first for any other format), return the image to its layout, signal the frame's fence. Libretro's Vulkan images are
   top-left origin [R], so no flip.
5. **Teardown**: `context_destroy`, the core's `destroy_device`, then the device and instance.

### 5.4 How frames reach the presenter

- **Stage 2, first step: read back.** The adapter waits for the frame's fence at the end of `advance` and copies the
  buffer into its picture, which `frame_copy` hands as any software picture; the frontend's existing path (the upload
  to Serenity's device, `EmuSen_DevicePicture.md` §4.5) shows it. No ABI change. The wait is measured; if it costs the
  emulation thread more than a tenth of a frame, the picture handed becomes the frame before's, which is a `latency`
  setting the player chooses, as Diamond's read-back is (`Diamond_Enhancements.md` §38.1).
- **Second step, only if the read-back is measured to matter: export.** A ring of three exportable images on the
  adapter's device (`OPAQUE_FD`, dedicated memory), exported once each, and an exported timeline semaphore, imported
  once into Serenity's device where its `deviceUUID` is the same, as the compositor already imports Serenity's
  (`EmuSen_DevicePicture.md` §4.2). That is a new optional capability and minor, `EXPORTED_PICTURE`, proposed then with
  its measurement; on another card, another platform or an older host the read-back stays.

### 5.5 Who touches the adapter's device

| Thread | What it does there |
|---|---|
| the host's emulation thread, inside `advance`/`create`/`free` | `retro_run` (the core records and submits); the adapter's copy submission and fence wait; context creation and teardown |
| the core's own threads, if any | the core's own submissions, each under `lock_queue`/`unlock_queue`, the adapter's mutex |
| the host's threads (presenter, interface) | nothing in the first step; in the second, only the imported copies on Serenity's own device |

## 6. Distribution and licences

- **Never shipped.** A libretro core is downloaded only when the player asks, from the libretro buildbot
  (`buildbot.libretro.com/nightly/<platform>/<arch>/latest/<core>_libretro.<ext>.zip`, and the core information
  archive beside it), into the folder of §1.1, then approved as §2 describes, showing its name, authors, licence as its
  `.info` states (PCSX2's libretro build: GPL-3.0, per `EmuSen_LibretroBridge.md` §5, read again from the file at
  download) and its SHA-256. `THIRD_PARTY_NOTICES.md` §2 gains a row: libretro cores and their core information, from
  buildbot.libretro.com, each under its own licence. `NOTICE` in the folder records the source address.
- **Firmware** is never fetched or shipped; a core needing a BIOS takes the player's file through the existing firmware
  settings (§3.8).
- **Tests** never download: the test core is built from the repository (§8), and measurement uses the PCSX2 nightly
  the probe already pins, run as a black box.

## 7. Platforms

Linux first. Kept possible: Windows (`.dll` through `libloading`, the temporary folder for the private directories,
`OPAQUE_WIN32` handles in the export step) and macOS (`.dylib`; Vulkan only where a driver is shipped, which is paused,
`EmuSen_DevicePicture.md` §4.4, so hardware cores read back or are refused there). Nothing in the core ABI changes per
platform. Android stays open: a libretro core there is a `.so` like any other.

## 8. Tests

- **The test core**, `emusen-libretro-testcore`, the project's own (GPL-3.0, a Rust `cdylib`, built in the test run,
  so no commercial core or game enters the repository): a deterministic picture from a frame counter and the input, in
  the pixel format an option chooses; sound through both the batch and the single-sample calls; save RAM and RTC;
  `retro_serialize` with an option that makes `retro_unserialize` fail half-way; options v2 with categories; log lines
  with formatted arguments from a thread of its own; `SET_GEOMETRY` and `SET_SYSTEM_AV_INFO` at set frames; null
  (duplicate) frames; `SHUTDOWN`; files written into its save directory; and from stage 2 a Vulkan picture drawn with
  `vkCmdClearColorImage` on rectangles (no shader to build) through negotiation version 2.
- **In Rust**: the adapter's own tests drive it through `emusen-core-host` with the test core, and the conformance kit
  (`emusen-core-conform`) runs every case against the pair; C8 and C10 run their machines one at a time (§1.1).
- **Through WiseMan**, headless: one test class loads the adapter and the test core through the C# host, runs frames,
  saves, loads and checks pictures, sound and battery files, and (stage 1) the same with the PCSX2 nightly when it is
  present, to settle §4's signal question. The BIOS path is an argument; nothing commercial is committed or listed.
- **Seeded faults**, three to seven a stage, aimed at what each stage could break (the X byte left unwritten, a drain
  crossing a rate change, a failed load not restored, a battery file not raised, the image's layout left unrestored).
- **Measurement**: PCSX2's libretro build on Jak and Daxter's arena, milliseconds a frame on the desktop (two rounds)
  and on the handheld, at each stage that renders, with the same bench used for `Sapphire_Speed.md`'s arena figures,
  against RetroArch's own figure for the same core (`EmuSen_LibretroBridge.md` §1). Sapphire's states are not this
  core's, so the arena is reached by an input script or by a state the core itself saved.

## 9. Stages

| Stage | What | Gate |
|---|---|---|
| **1** | The adapter with software pictures, sound, input, saves and the private directories, states, options, firmware; the test core; E1 and E2 (minor 1.16) in the header, the Rust definitions and the ABI baseline | the kit passes on adapter and test core; the WiseMan class passes; PCSX2 loads, runs and frees inside the .NET host (pictures black until stage 2 if it renders only through Vulkan) |
| **2** | Vulkan: §5.3 and §5.4's read-back; the test core's Vulkan picture | the test core's Vulkan pictures match its software ones; clean under the validation layer with synchronisation validation; PCSX2 shows Jak's arena, timed on the desktop and the handheld |
| **2b** | `EXPORTED_PICTURE`, only if stage 2's measurement asks for it | proposed then with its numbers |
| **3** | Frontend: the core list and download, the approval naming both files, the settings window from the schema, memory cards in the card manager, big picture, the engine choice per console (Sapphire or the libretro core); the fit and pad audits | Mistress and big picture run a game either way; cards follow the save-media rules; the choice is remembered |

## 10. Risks

1. **Signal handlers** under .NET (§4); remedy the broker, at a copy per picture.
2. **A second load after `free`** may not work in every core without `retro_deinit`; such a core then needs a new
   process per game, which the frontend would have to learn, so it is tested first.
3. **Assets** a core needs beyond firmware are known only by running it; PCSX2's are found in stage 1.
4. **Determinism**: PCSX2 runs threads of its own, so its runs may differ; its kit report is a report, not a gate.
5. **Option keys or values** outside the schema's character set would be refused; none is expected [A].

## 11. Questions for the tester

1. **[decision]** The adapter makes its own Vulkan device on the host's card instead of using the lent device (§5.2),
   with frames read back first and exported only if measured to matter. *Recommended:* yes.
2. **[decision]** Minor 1.16: `IMAGE_PATH` (E1) and `effect: "unrated"` (E2). *Recommended:* both; without E1 no
   disc-based libretro core can run.
3. **[decision]** The save directory is the adapter's private one, mapped to EmuSen battery files by a per-core
   profile, rather than a host folder handed to the core, which §7.2 forbids. *Recommended:* yes.
4. **[decision]** One copy of the adapter per installed libretro core, one machine at a time, no run-ahead.
   *Recommended:* yes.
5. **[routine]** The log callback through a small C file built with `cc`. *Recommended:* yes; the alternative logs
   format strings without their values.
6. **[routine]** The component's name, `emusen-libretro-host`, in `EmuSen/Cores/Libretro/`, rather than a codename,
   since it is not a console. *Recommended:* yes.
