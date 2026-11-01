# EmuSen_LibretroHost — libretro cores run as EmuSen cores

*Design, 2026-10-31; revised the same day. Nothing is built.* Decided by the tester on 2026-10-31: "Let's go the libretro
route, that way EmuSen can consume other cores." The first version's four decisions were approved the same day, with a
condition that governs the whole design: "I would prefer this done right than hacky. So no shortcuts. If we're going to
host libretro cores, it needs to be done right." The revision (§0.1) replaces every place where the first version took a
shortcut or put off the proper form, and adds EmuSen's own cheats for hosted cores, also asked for by the tester. An
independent review the same day found it sound with changes, which §0.2 lists and the sections below carry.

This widens `EmuSen_LibretroBridge.md` (a stopgap for the PlayStation 2, proposed 2026-10-27, superseded) to any libretro
core, and is the inbound direction `EmuSen_CoreAPI.md` §14.1 names and §15 Q6 decided in principle. The outbound
direction, a v1 core offered to RetroArch, keeps the name `emusen-libretro` (`EmuSen_Libretro.md` §7). The first target, decided by the
tester on 2026-10-31, is PCEE2 (`pcee2_libretro`), the libretro build of upstream PCSX2, with memory maps and software
pictures, the nightly pinned in `EmuSen.WiseMan/Reference/libretro-cores.sha256`; LRPS2 (`pcsx2_libretro`, a hard fork of
an older PCSX2) is the second.

The marks are those of `EmuSen_Gpu.md`: **[M]** measured, with the date and tool; **[R]** read from a source; **[A]**
argued; **[C]** a choice. A section number of the form §6.n or §7.n given without a document is `EmuSen_CoreAPI.md`'s;
every other bare number is this page's.

**Sources** [R, read 2026-10-31]: `libretro.h` and `libretro_vulkan.h` from libretro-common's `master`
(<https://github.com/libretro/libretro-common/tree/master/include>, MIT; Vulkan interface version 5, negotiation version
2); PCEE2's and LRPS2's core information files (`pcee2_libretro.info`, `pcsx2_libretro.info`, buildbot's info archive and
libretro-core-info `master`) and LRPS2's published documentation (<https://docs.libretro.com/library/lrps2/>); `emusen_core.h` 1.15; `EmuSen_CoreAPI.md` §6.0–§6.22, §7, §14.1;
`EmuSen_CoreAPI_Extensions.md` §4, §6; `EmuSen_Cheats.md` §1–§4, §10; `EmuSen_Libretro.md` §2.6; `EmuSen_Gpu.md` §5;
`EmuSen_DevicePicture.md` §1.3, §4, §5; `Platform/serenity/src/vulkan.rs`; `emusen-native`'s `core` trait;
`CoreEngine`'s cheat path as `EmuSen_CoreAPI_Extensions.md` §6.1 records it. No emulator's source, PCSX2's builds included,
is read for this work; they are only run, as black boxes.

### 0.1 What the revision changed

| First version | Revised |
|---|---|
| the core in the host's process; process isolation a fallback if signal handlers fought | **every libretro core runs in a process of its own** behind a generic broker, confined by the kernel where it offers it, its crash reported and survived (§3) |
| one machine per library, so no second instance | one process per machine, so machines are independent; run-ahead's second instance is possible where a core is deterministic |
| hardware frames read back through memory; sharing on the GPU "only if measured to matter" | **the picture on a host-owned ring of exported images**, with timeline semaphores, as the form (§5); read-back only where a platform has no way to share |
| the save directory's files as one archive, plus a per-core profile | **named battery files**: every file a core writes is its own save file, owned and stored by the host; profiles add only labels and media (§4.5) |
| the sound rate rounded to the hertz | **the exact rate** as a ratio for the host's resampler (§4.3) |
| the snapshot not claimed | claimed, withdrawn per machine where the core's quirks say states are incomplete (§4.6) |
| states checked by name and version | also by a digest of the payload and, where the core declares it, the platform (§4.6) |
| rumble refused | built as `EmuSen_CoreAPI_Extensions.md` §4's event (§4.4, stage 5) |
| an older host given a temporary copy of a large image | the core states `host_requires: ["image_path"]` and an older host refuses it with that reason (§2) |
| software frames always copied in the callback | the core may draw straight into the frame's shared memory (`GET_CURRENT_SOFTWARE_FRAMEBUFFER`, §4.2) |
| cheats not addressed | **EmuSen's cheat engine applies its codes through the memory the core exposes**; `retro_cheat_set` only for what EmuSen cannot decode, the path shown (§4.9) |

### 0.2 What the review of 2026-10-31 changed

| Found | Now |
|---|---|
| a Landlock allow-list that no Vulkan or recompiling core could run under | the ruleset of §3.3, with the render node, the driver's files and the ioctls a GPU needs; a needed path denied fails `create` and names it, and **a core never runs unconfined** (decided 2026-10-31) |
| crashes contained, hangs not | a deadline on `create`, `advance` and `free`, a setting in seconds, after which the runner is killed (§3.4); the presenter's wait released after a crash (§5.4); the GPU's own reset named as a limit (§3.6) |
| `PR_SET_PDEATHSIG`, which fires when the spawning *thread* ends | the runner watches its control socket for the parent's end (§3.2) |
| the approval keyed to the proxy, a manifest file, and `NOTICE` in a folder the player can write | the approval keyed to the libretro core's own hash, compared by the host before any runner starts, and the bytes the runner loads the bytes that were hashed (§1.3); `NOTICE` is information only |
| every file a core writes treated as a save | a persistent private folder per core; only files in the save directory, or files a profile names, are saves (§4.5); named files' names checked |
| named files numbered from `0x20000000`, inside `BORROW`'s promise for numbers from 65536 | numbered from `0x8000` to `0xFFFF`, below `EMUSEN_FILE_MEDIA` (§2, E3) |
| the ring's ownership hand-over left implicit | `VK_QUEUE_FAMILY_EXTERNAL` release and acquire each frame on both devices; a larger picture mid-frame held until the host lends a new ring (§5.4) |
| memory between frames assumed still | a race check of the window (§3.2, §9); the PS2 cores' exposed memory cited from `Sapphire_Referees.md` §2 (§4.9) |
| the ring and `DEVICE_PICTURE` read as two forms | two boundaries (§5.4) |
| smaller points | trust on first download said (§7); the archive's bounds (§7); options declared in `retro_load_game` (§4.7); `retro_cheat_reset` (§4.9); `-10` owned by the proxy's `status_text` (§3.4) |

## 1. The form: three components, composed

Weighed under the project's rule for features (an extension, a module, or a component):

- **Not a frontend feature.** Sessions, states, saves, settings, cheats and media already cross the core ABI, so a
  libretro core hosted *below* it reaches Mistress, big picture and the shell with no per-frontend code (§2.4 of
  `EmuSen_CoreAPI.md`).
- **Not a module of an existing core**: it is a second job for every core there is.
- **Three components, each with one job** [C], composed as Unix programs are:

| Component | Its one job | Form |
|---|---|---|
| `emusen-libretro-host` | turn one libretro core into a v1 core | a Rust `cdylib` exporting the v1 surface through `core_exports!`; loads `<core>_libretro` with `dlopen` |
| `emusen-core-broker` | run any v1 core in a process of its own, behind the same v1 surface | a Rust `cdylib` (the proxy the host loads) and an executable (`emusen-core-runner`, which loads the real library); `EmuSen/Cores/Shared/emusen-core-broker/` |
| `emusen-libretro-testcore` | a libretro core of the project's own for every test | a Rust `cdylib` |

A libretro core in play is therefore *broker proxy → runner process → libretro host → libretro core*. Each piece stands
alone: the libretro host runs in-process under the conformance kit and in tests; the broker runs any v1 core, so it is
also `EmuSen_CoreAPI.md` §7.3's broker, built; the test core runs under RetroArch too. Neither host (the C# `CoreInterface`
nor the Rust `emusen-core-host`) changes to gain isolation, since the proxy is an ordinary v1 library.

### 1.1 The folder of an installed core

Discovery keys one sidecar, one `info()` and one approval to one library file (§7.1), and glibc's `dlopen` returns the
handle already loaded for a path that names the same file (device and inode), so each installed core has its own folder
and its own *copy* of the proxy:

```
/Cores/libretro/pcsx2/
    emusen-core-broker.so               a copy of the project's proxy (hash in cores.manifest); the file hosts load
    emusen-core-broker.so.core.json     its sidecar, written by loading it once after approval
    broker.target                       one line: the library the runner loads, emusen-libretro-host.so here
    emusen-libretro-host.so             a copy of the project's libretro host (hash in cores.manifest)
    pcsx2_libretro.so                   the libretro core, as downloaded
    pcsx2_libretro.info                 its published core information
    NOTICE                              the core's name, authors, licence, source address and SHA-256
    assets/                             data files the core needs beside firmware (§4.8)
```

The runner is the installation's own executable. The libretro host is copied into the folder because the runner loads
it as a v1 library, and v1 has no way to hand a library a folder: it finds its libretro core as the one
`*_libretro.<ext>` beside itself. Nothing is shared between machines, since each machine's runner is a process of its
own. `NOTICE`'s format, and the one module that reads and writes it, are the Cores window's (`emusen-core-entry`,
`EmuSen_Cores.md` §7); this host uses that module.

### 1.2 The crates

`emusen-libretro-host` (`EmuSen/Cores/Libretro/emusen-libretro-host/`), one job per module: `retro` (the bindings,
written by hand from `libretro.h` for the subset used, with layout tests against the header), `env` (the environment
call), `video`, `audio`, `input`, `files` (the private directories and named battery files), `state`, `options`,
`memory` (spaces and cheats), `vulkan`, `profile`, and a C file of a few lines for the one variadic callback, the log
(stable Rust cannot define a variadic function; the C formats with `vsnprintf` and hands a finished line to Rust).
Dependencies: `emusen-native`, `libloading`, `cc`, `ash` 0.38 (the version `emusen-gpu` and Serenity use).
`emusen-core-broker`: `proxy` (the exports, generated from the header's export list so none is forgotten), `wire` (the
protocol), `runner`, `sandbox`. The probe's libretro backend (`Reference/probe-rs/src/backends/libretro.rs`) keeps its
own bindings; moving it onto `retro` is a later step of its own.

### 1.3 Trust, keyed to the libretro core

The approval names what runs [C, after the review]: `cores.approved` records, for each folder, the SHA-256 of its
libretro core, read from the file and never from `NOTICE`, which sits in a folder the player can write and is
information only (`emusen-core-entry`'s `verify` compares the two as an integrity check, nothing more). The proxy and the
libretro host are the project's, vouched for by `cores.manifest`. The chain is closed so that the bytes approved are
the bytes run:

1. At its first library-level call the proxy copies the libretro core into a sealed `memfd` (`F_SEAL_WRITE`,
   `F_SEAL_SHRINK`, `F_SEAL_GROW`) and hashes that copy; its `info()` carries the hash as
   `"libretro": { "file", "sha256" }`, a field older hosts ignore.
2. The host (`emusen-core-host`'s trust, and the C# `CoreTrust` beside it) compares that hash with the folder's entry in
   `cores.approved` before any `create`; a hash not approved is "needs approval" and nothing starts.
3. Each runner receives the sealed copy by descriptor and loads it from `/proc/self/fd/<n>`, so the file on disk may
   change and nothing unapproved runs; the change is a new hash, asked about again.

**Trust on first download** [A]: the buildbot publishes no signatures (`EmuSen_Cores.md` §5.2), so the first approval
trusts what was downloaded over HTTPS; the hash then holds it to that build.

**When EmuSen is updated**, the proxy's and the libretro host's hashes change. Discovery lists a folder whose copies differ
from the installation's as "needs refresh", not "needs approval"; the core tool's `cores verify` refreshes the copies, and
the approval, keyed to the libretro core, stands.

## 2. The core ABI: what v1.15 lacks, and minor 1.16

Each item is an addition under §4.2 of `EmuSen_CoreAPI.md`: no export, field or meaning changes.

| # | libretro needs | v1.15 | Proposed for 1.16 |
|---|---|---|---|
| E1 | a **path** to the game for a core with `need_fullpath` (PCEE2 and LRPS2: `needs_fullpath = "true"` [R, their `.info`]; DVD images to 8.5 GB) | the image's bytes | **`IMAGE_PATH`** (capability bit 26): `emusen_create_params.image_path` and `emusen_media.image_path`, `const char *` UTF-8, appended. Filled for a core claiming the bit whenever the bytes passed are an unaltered file's (no decompression, no soft patch); the image may then be lent mapped. A core that cannot run without it lists `"image_path"` in `host_requires` (§4.6 of `EmuSen_CoreAPI.md`), so an older host refuses it with that reason. A soft patch with a need-fullpath core is refused in words, since the core opens the file itself |
| E2 | options whose cost **nobody has classified** | `effect` from five values, the accurate value the default for two | **`effect: "unrated"`** (the field is an open enumeration): shown under the core's own categories as the core's options, with no claim about cost; a profile may rate an option (§4.7) |
| E3 | **files a core names at run time** (LRPS2 keeps memory cards in `system/pcsx2/memcards/` [R, its documentation]; PCEE2 says `libretro_saves = "false"` [R, its `.info`], so its cards are files of its own) | battery files numbered and listed ahead in core info | **named battery files**: `emusen_file.name` (`const char *`, a relative path, appended; arrays already carry their element size). Core info's `battery` may hold a *family*, `{ "family": "files", "kind": "console" }`; the host passes every file it keeps for it at create, numbered from `EMUSEN_FILE_NAMED` (`0x8000`, so below `EMUSEN_FILE_MEDIA` and outside `BORROW`'s promise for numbers from 65536; at most 32,768 files) with its name; a name is UTF-8, relative, with no `..` part, no empty part and no `\0`, and the host refuses any other and never follows a link in its own folder; machine info lists the family's files present, `{ which, name, kind, length }`, a new one raising `MACHINE_INFO` and a change `BATTERY`; a file the core deletes is listed `removed`, and the host keeps its last copy beside the folder rather than deleting the player's data |
| E4 | a sound rate that is **not a whole number of hertz** (`retro_system_timing.sample_rate` is a `double`) | `audio_rate`, an `int32_t` | machine info's **`audio.rate_exact`**, `{ num, den }`, beside `audio_rate` (its rounding); a change raises `AUDIO_RATE` and `MACHINE_INFO` |
| E5 | a hardware picture that **crosses devices and processes** | `DEVICE_PICTURE` (1.9): one device lent within one process | **`DEVICE_RING`** (capability bit 27), §5.4: the host lends a ring of exported images and two exported timeline semaphores; `emusen_core_ring_lend` and `emusen_core_frame_ring` |

**Later in this work** (stage 5): E6, rumble, as event kind 7 of `EmuSen_CoreAPI_Extensions.md` §4, minor 1.17; LRPS2
supports rumble for the DualShock 2 [R, its documentation], and PCEE2's support is found by running it.

**Outside the core ABI**: (a) discovery and trust learn the entry of §1.1: the proxy and libretro host vouched for by the
manifest, the libretro core by its own hash in `cores.approved`, compared before any runner starts (§1.3); a changed
libretro core is asked about again (§7.2, rule 2); (b) `emusen-native` lets a core claiming `SNAPSHOT` withdraw kind 1 for one
machine, machine info's `state.kinds` then `[0]` and `state_save(1)` `NOT_SUPPORTED`, which C14 already reads as
consistent; (c) `emusen-gpu` exposes its choice of card (its next minor, 1.5, since 1.4 went to the GS on 2026-10-31) so that the choice keeps one owner (§5.3).

**What v1's rules do not promise for these cores**, named rather than hidden [A]: `deterministic` is false unless a
core's profile says otherwise after the kit's C8 has passed on it; a foreign core's defaults are its author's, not the
accurate configuration §6.13 binds the project's cores to; within its process it runs its own threads, signal handlers
and file I/O.

## 3. Isolation: every libretro core in a process of its own

### 3.1 Why, and what it settles

A libretro core is foreign native code downloaded at the player's request. In the host's process it could crash the
frontend, and it may install signal handlers (a recompiler's fast memory traps faults with `SIGSEGV`) that the .NET
runtime under Mistress also installs. §7.2 already says no check after `dlopen` protects a process from a library, and
§7.3 designed the broker for this. In a process of its own [C]: a crash ends the runner, not the frontend; its signal
handlers, threads, working directory and environment are its own; and on Linux the kernel confines what it may read,
write and reach. One process per machine also removes libretro's one-instance-per-library limit.

### 3.2 The protocol

- **Start.** The proxy starts `emusen-core-runner` at the first library-level call (to answer `info`, `settings_schema`,
  `firmware_for`, then cached; the runner exits) and one runner per `create`. It passes a Unix socket pair (control,
  and descriptors by `SCM_RIGHTS`) and a shared memory region (`memfd`), and the runner loads `broker.target`.
- **Calls.** Every v1 export is forwarded: the proxy writes the call and its arguments into the shared region and wakes
  the runner (a futex word in the region; a socket message for calls that carry descriptors), and waits for the answer.
  Buffers the core fills (pictures, sound, states, descriptors) are written by the runner straight into the region and
  copied once into the host's buffer by the proxy, so a software picture is copied the same number of times as
  in-process (§4.2).
- **One round trip a frame.** `set_buttons` and `set_axis` are latched in the proxy and sent with `advance`;
  `advance`'s answer carries the frame's `frame_info`, its events and its sound, and the picture is already in the
  region; so `advance`, `events`, `frame_info`, `frame_copy` and `audio_drain` together cost one wake each way. Its
  cost is measured at stage 2 (the prediction, P1: under 50 µs a frame on this machine).
- **The frame-end window.** The host's cheat engine and debugger read and write memory between frames (§4.9), one
  call per access. Between two calls on a machine the machine does not run (§6.16: a core's threads are idle or
  invisible), so memory is stable there, and the proxy may answer from a copy [A]: at each `advance`'s end the runner
  copies into the region the address ranges the host read in the previous window; reads inside them are answered there,
  a read outside them costs one round trip and joins the set, and writes are queued in order and applied before the
  next call that runs the machine. The window is exact by construction; a test reads and writes through it and through
  a direct round trip and compares.
- **Descriptors.** The ring of §5.4, the image file (opened by the proxy and passed as a descriptor where the sandbox
  denies the path) and lent images (a `memfd`) cross by `SCM_RIGHTS`.
- **Lifetime.** `PR_SET_PDEATHSIG` fires when the *thread* that started the runner ends, not the process [R, prctl(2)],
  so it is not used. The runner keeps a thread that waits on its control socket and ends the process with `_exit` when
  the socket closes, which happens when the proxy closes it or its process ends for any reason; on Windows a job object
  with kill-on-close does the same. `free` asks the runner to free the machine and exit, then reaps it.
- **Memory between frames.** A foreign core runs threads of its own, so "the machine does not run between calls" is a
  claim to prove per core, not an assumption: the check (§9) reads 1 MiB of each space twice between frames, apart in
  time, over 1,000 frames, and requires the two reads equal. A core that fails it has its window turned off and every
  access made as a direct round trip after its threads are idle as far as the host can make them, and the finding named
  in its profile.

### 3.3 The sandbox

On Linux, before loading the target, the runner sets `no_new_privs` and restricts itself with Landlock to this ruleset
[C, after the review]:

| Access | Paths | Why |
|---|---|---|
| read, execute | `/usr`, `/lib`, `/lib64`, the installation's cores, the core's folder | the system's libraries, the loader, the driver's libraries |
| read | `/etc` | `ld.so.cache`, the Vulkan ICD and layer files |
| read | `/proc/self`, `/sys/class/drm`, `/sys/devices` (the GPU's nodes as the driver reads them) | the driver's discovery of the card |
| read, write, and `LANDLOCK_ACCESS_FS_IOCTL_DEV` where the Landlock ABI is 5 or later | `/dev/dri/renderD*` | the render node, and the ioctls every GPU submission is |
| read | the image's folder | a sheet's or `.m3u`'s companions beside the image |
| read, write | the core's private folders (§4.5), its Mesa shader cache among them | the core's own data |
| none | the network, where the Landlock ABI is 4 or later | |

The Mesa shader cache is pointed into the core's private cache folder (`MESA_SHADER_CACHE_DIR`, set by the runner before
the target loads), so no wider write is needed. **A needed path denied fails `create`, naming the path, and is reported;
a kernel without Landlock fails `create` the same way: the core never runs unconfined** (decided 2026-10-31). The set
is proven, not guessed (§3.5): `strace -f -e trace=openat,ioctl` on the runner while the test core draws through Vulkan
and while the PS2 core runs, every denial and every path opened read against the table. The handheld's kernel is
checked when the handheld is back. Windows and macOS keep the process boundary; their sandboxes (a restricted token, the
App Sandbox) are later steps of their own.

### 3.4 A crash

When the runner ends without being asked, every later call on the machine answers `-10`, "the core's process ended", a
status of the core's band that the proxy's `status_text` owns; `last_error` says how (the signal, or the exit code), and
the crash log the runner was given names the libretro core. **A hang is treated as a crash**: `create`, `advance` and
`free` each have a deadline, the proxy's setting `broker.deadline` in seconds (default 10 for `advance`, 60 for `create` and
`free`, which load and save), after which the proxy kills the runner (`SIGKILL`) and answers `-11`, "the core stopped
answering". The frontend shows either as an error, offers to reload from the last state, and never hangs: every wait on
the runner also watches the socket's end and the deadline.

### 3.5 How it is proven

The test core has options that, in its process: install a `SIGSEGV` handler and recover from a deliberate fault, as a
fast-memory recompiler does; fault without a handler; call `abort`; write outside its private directories; open a
network socket; never return from `retro_run`. Through the C# host in WiseMan (the .NET runtime present) and through
`emusen-core-host`: the first runs on unharmed, the next two end the runner with `-10` while the host runs on, the next
two are denied by the kernel, and the last ends at its deadline with `-11`. The ruleset's `strace` reading of §3.3, and a
run with Landlock made unavailable that must fail `create`, complete the proof. The PS2 core then loads, runs and frees
in the .NET host the same way.

### 3.6 What the boundary does not isolate

The GPU. A core that makes the driver reset the card (a hung submission, a fault the kernel answers with a GPU reset)
takes every context on that card with it, the frontend's presenter included: process isolation separates memory and
signals, not the device [A]. The frontend's presenter sees `VK_ERROR_DEVICE_LOST` and makes its device again, as any
Vulkan program must; that path is tested on the presenter's side, and the limit is said in the core's details.

## 4. The mapping, call by call

### 4.1 Lifecycle and the frame

| EmuSen | libretro |
|---|---|
| a library-level call, the first time | `retro_set_environment`, `retro_get_system_info`, `retro_init` (options may be declared in either); answers cached |
| `info()` | `retro_system_info` and the `.info` file: system through a table from libretro's `systemid` to EmuSen's (`playstation2` → `ps2`), extensions, licence, authors, firmware; an unknown system offered under its own name |
| `create` | private directories populated (§4.5, §4.8); `retro_load_game` with the path (E1) or the data; save RAM copied in; `retro_get_system_av_info`; each port's device (§4.4); the Vulkan context (§5) and `context_reset` |
| `free` | `context_destroy`, `retro_unload_game`, `retro_deinit`; the runner exits, so a core's global state never outlives its machine |
| `advance` | one `retro_run`, then the frame-end work: the cheat window's writes (§4.9), the files check (§4.5), the picture's hand-over (§5) |
| `reset` (`RESET`) | `retro_reset` |
| `set_options` bit 0 | `GET_AUDIO_VIDEO_ENABLE` answers video off for that frame |
| `frame_rate` | `timing.fps` as the exact ratio its `double` holds where the denominator stays under 10⁶, else the nearest with one (59.94005994 → 60000/1001); a change raises `MACHINE_INFO`, and pacing is the presenter's, as for every core |
| `phases` (`PHASES`) | `retro_run`, picture, sound, files, as nanoseconds, so that the host's own tools measure a hosted core as any other |

### 4.2 Video

| libretro format | handed as | in the callback |
|---|---|---|
| `XRGB8888` | `BGRA8888` where the host accepts it, else `RGBA8888` | the X byte written as 255 (or a swizzle) |
| `RGB565` | `RGB565` where accepted, else `RGBA8888` | a copy (or an expansion) |
| `0RGB1555` (the default) | `RGB565` where accepted, else `RGBA8888` | a conversion |

The adapter answers `GET_CURRENT_SOFTWARE_FRAMEBUFFER` with the frame's place in the shared region, in the format agreed,
so a core that uses it draws where the host reads and nothing is copied in the callback; otherwise the picture is
copied there from the core's pitch. The region holds `max_width × max_height` and is never resized per frame. A null
picture (`GET_CAN_DUPE` true) keeps the last one, and `FRAME_SERIAL` is claimed. `SET_GEOMETRY` and
`SET_SYSTEM_AV_INFO` raise `GEOMETRY`, `AUDIO_RATE` and `MACHINE_INFO` as each changes; the aspect (`base_width /
base_height` when not positive) becomes `aspect_num/den` by the same rule as the frame rate. `SET_ROTATION` is refused
and logged: v1 has no rotated picture, and the core then draws unrotated or rotates itself.

### 4.3 Sound, and its rate without drift

`audio_sample_batch` and `audio_sample` append interleaved stereo `int16` into a ring in the shared region, in runs
tagged with the rate they were made at, so that `audio_drain` never crosses a change (§6.7). The rate is reported twice:
`audio_rate`, rounded to the hertz as the export requires, and `audio.rate_exact` (E4), the ratio of the core's
`double`. **The adapter does not resample** [C]; the frontend's resampler is the one owner of rate conversion, and it
converts from the exact ratio to the device's rate, its rate control then correcting only the device clock's error, as
for every core. The proof is a test (§9) that runs the test core at 32,040.5 Hz for 36,000 frames and counts the
samples the frontend's resampler consumed against the exact rate and time: the error stays within one sample, with no
growth over the run. `SET_AUDIO_CALLBACK` (sound pulled on the frontend's thread) and
`SET_AUDIO_BUFFER_STATUS_CALLBACK` are refused and logged; cores fall back to the batch path [R].

### 4.4 Input, through Endymion

The player's devices reach the core as every core's do: Endymion maps them onto the canonical controls and axes, and
the host sends those by `set_buttons` and `set_axis`. The canonical controls are the RetroPad's ids in its order
(`EMUSEN_CONTROL_B` = 0 … `R3` = 15), so a port's bit *n* is `RETRO_DEVICE_ID_JOYPAD` *n* with no table; `input_state`
answers from what was latched before `advance`, and `GET_INPUT_BITMASKS` answers true. A stick's −1…1 (right and down
positive, libretro's convention too [R]) is scaled by 32,767 and rounded half away from zero; a trigger's 0…1 becomes
`ANALOG_BUTTON` L2/R2 at 0…32,767; any other analog button reads 0 or 32,767 from its digital bit, since Endymion
carries no pressure for it (named in the core's losses: the DualShock 2's pressure-sensitive buttons read fully on or
off). Each port is set to the device the core's `SET_CONTROLLER_INFO` lists as its pad with sticks (LRPS2's "DualShock
2" [R], on which its rumble depends; PCEE2's `input_descriptors = "false"` [R] means its labels are the RetroPad's), else `RETRO_DEVICE_ANALOG`, else `JOYPAD`; `PRESENCE` sets `RETRO_DEVICE_NONE`.
`GET_INPUT_MAX_USERS` answers the ports machine info lists; labels from `SET_INPUT_DESCRIPTORS` become the controller's
labels. Keyboard, mouse, light gun and pointer read zero until device kinds exist (`EmuSen_CoreAPI_Extensions.md` §10),
and a core that asks for them is shown as asking, never silently. Rumble (stage 5): `GET_RUMBLE_INTERFACE` answered, each
`set_rumble_state` queued as event 7 for the host to send to the player's pad through Endymion.

### 4.5 Saves: named battery files

- **Save RAM** (`RETRO_MEMORY_SAVE_RAM`) is battery file 0, `.srm`, kind `game`, copied in after `retro_load_game` as
  libretro asks; `RETRO_MEMORY_RTC` is file 1, `.rtc`. `BATTERY_DIRTY` is claimed: `battery` compares with the copy last
  marked saved, so nothing is done per frame.
- **The core's own folders.** §7.2 forbids handing a core a host path to write to, so the runner gives the core folders
  of its own, persistent and on disk (never `tmpfs`: a core's pipeline cache alone can be large), beside the entries
  (`/Cores/libretro/.data/<stem>/`, with `system/`, `save/` and `cache/`; a dot folder, which Cores' listing skips and its
  updates leave in place, and which a removal deletes with the core), answered to
  `GET_SYSTEM_DIRECTORY` and `GET_SAVE_DIRECTORY`. What a core keeps there for itself (its settings files, logs, patch
  folders, pipeline caches) stays there and is **not** a save: such files are the core's working data, kept between
  games and removed with the core, and listed nowhere as the player's.
- **Which files are saves** [C, after the review]: those the core writes in its **save directory**, which libretro means
  for saves, and those its **profile** names elsewhere. Each is a named battery file (E3) the host owns and stores in the
  console's save folder by name, shared by the games of that core as the console's own memory is (kind `console`,
  `EmuSen_SaveMedia.md` §3.6). Before `retro_load_game` the runner writes into place every such file the host passed;
  after each frame, at most every 250 ms (an `inotify` watch on Linux, sizes and times elsewhere), each new, changed or
  removed one is reported by E3's rules.
- **Profiles** (data in the libretro host, per core, versioned) name the saves outside the save directory and give them a
  medium: LRPS2's legacy cards in `system/pcsx2/memcards/` [R, its documentation] become kind `removable`, medium
  `ps2.card`, ports 0 and 1, so the card manager shows them as cards; PCEE2's cards are found where it writes them. What a PS2 core writes is listed by one probe run
  and `find` over its folders (stage 1), and only then written into its profile. Whether those bytes are the
  same image as Sapphire's `ps2.card` is checked (both are 8,650,752-byte raw cards if so) before a card is shared
  between the two engines; until then each engine keeps its own.
- **States and shared files.** Under `CREATE_SHARED_REMOVABLE` and `CONSOLE` a state load leaves such files as they stand
  (§6.27). The runner snapshots the family's files before `retro_unserialize` and compares after; a core that rewrites
  them on load has them restored and the event logged, and the kit's case for shared media runs on each core.

### 4.6 States

Kind 0 is an 80-byte header followed by `retro_serialize`'s bytes: magic `ESLR`, header version, the libretro core's name
and version, the adapter's state version, a SHA-256 of the payload, the payload's length, the platform (architecture
and operating system) where the core declares `ENDIAN_DEPENDENT` or `PLATFORM_DEPENDENT`, and the disc in the drive (its
length and the digest of its first MiB), so that a state over another disc is `WRONG_MEDIA`. Another core's state is
`FOREIGN`; another version's, adapter's or platform's is `STATE_VERSION` with words saying which; a payload whose digest
fails is `TRUNCATED`. Machine info's `state` gives `format: "ESLR"`, its version and `loads_from`. A failed load changes
nothing, which `retro_unserialize` does not promise, so the runner serializes the machine first and restores that copy
when the load fails, and says so in `last_error`. `state_size` is the header plus `retro_serialize_size`, read again
after each frame where the core declares `CORE_VARIABLE_SIZE` (raising `STATE_SIZE`); the adapter declares
`FRONT_VARIABLE_SIZE` back. `state_layout` names the header's fields and one opaque payload. `SNAPSHOT` is claimed: kind
1 is the same bytes saved under `GET_SAVESTATE_CONTEXT`'s run-ahead context, withdrawn for a machine (§2 (b)) whose core
declares `INCOMPLETE` or `SINGLE_SESSION`; `SINGLE_SESSION` also keeps kind 0 from being offered across sessions.

### 4.7 Core options, in EmuSen's settings model

`SET_CORE_OPTIONS_V2` (and v1 and `SET_VARIABLES`, converted) become the schema at library level: `key` the option's
key (keys outside `[A-Za-z0-9_.-]+` make the core's install fail with a report naming the key), `label` and `help` its
description and information, `kind: choice` with its values and labels, `default` its default, `category` its
category's description, `scope: run` (libretro applies options between frames), `effect: "unrated"` (E2) unless the
profile rates the option with its effect, cost and hardware value. Values reach the core by `GET_VARIABLE`;
`set_settings` marks them for the next `GET_VARIABLE_UPDATE`. `SET_VARIABLE` (the core changing its own option) is
reported through `setting_notes` and an event so that the settings window shows the value the core holds. Options a
core hides for the current values (`SET_CORE_OPTIONS_DISPLAY`, the update callback) carry a note, "not used with the
current settings", which is `setting_notes`' meaning. The schema and values live in the frontend's existing per-console
settings, so a hosted core's settings window, reset and storage are every core's. **Options declared in `retro_load_game`**
rather than at initialisation are taken for the running machine at their defaults, with a note saying so; the adapter
records the declarations in the core's private folder, so that every later library load includes them in the schema, and
discovery writes the sidecar again when the schema has grown (the schema is not part of the altered-library check).

### 4.8 Firmware and the system directory

`firmware_for` answers from the `.info` file's firmware entries and the MD5s its notes list. PCEE2's and LRPS2's name a
folder, not a file: "`pcsx2/bios` folder", with no particular file name required [R]; the entry therefore takes the player's BIOS
files from the firmware settings, recognised there by content, and the runner writes them (mode 0600) under
`system/pcsx2/bios/` by their own names at `create`. Nothing is fetched or shipped. Non-firmware data a core needs
(PCEE2's `pcsx2/resources` folder, whose `.info` says the core will not boot without it [R], though no PCEE2 run of an ELF
needed it [M, `Sapphire_Referees.md` §2.3]; LRPS2's optional `GameIndex.yaml`, which it carries built in [R]) is in the
core's `assets/`, downloaded with the
core at the player's request and linked read-only into the same directory. `GET_CORE_ASSETS_DIRECTORY` answers it;
`GET_CONTENT_DIRECTORY` answers the image's folder.

### 4.9 Cheats: EmuSen's engine first

**The primary path is EmuSen's own** [C]. The runtime's cheat model already applies writes, compares, repeats, bit
writes and the conditional tests (`IfEqual`, `IfNotEqual`, `IfLess`, `IfGreater`) through `space_read` and
`space_write`, between `advance` and `present`, for any v1 core (`EmuSen_CoreAPI_Extensions.md` §6.1, `CoreEngine`'s
`EndFrame`) [R]. A hosted core takes part by exposing its memory as spaces:

- **Spaces from the core's memory.** `SET_MEMORY_MAPS` descriptors become spaces at the console's addresses; without
  them, `retro_get_memory_data`/`size` for `SYSTEM_RAM` and `VIDEO_RAM` become spaces `RAM` and `VRAM`. The space named
  by the systems table's `cheat_space` for the system carries the `cheats` flag, so codecs resolve addresses into it as
  they do for the project's own core of that console. Reads and writes go to the core's memory in its process, at the
  frame's end, through the broker's frame-end window (§3.2), so a frame's cheats cost no round trips once their
  addresses are known.
- **Then the same codes, database, windows and `cheat` commands** work for a hosted core as for a native one: the
  console's codecs decode, `CheatRegistry` applies, the emusen-cheats collection supplies, and `cheat list` shows them.
  `CHEAT_POKES` is not claimed, so every code goes through the host's registry, one rule for all.
- **The secondary path**, `retro_cheat_set(index, enabled, code)`, is used only for a code no codec of the console
  decodes, or for a core that exposes no memory, and only where the core's profile says it honours the call (libretro
  gives no answer to tell). The cheat window and `cheat list` show which path a code took ("EmuSen", "the core's own",
  or "refused: this core takes no codes"), and a code neither path takes is refused in words, never dropped.
  `retro_cheat_reset` is called before the secondary path's list is sent again whenever it changes, and at `reset`, so the
  core holds exactly the codes the player has on.
- **The PlayStation 2.** PCEE2's core information says `memory_descriptors = "true"` and `cheats = "false"` [R], so
  EmuSen's engine reaches its memory by maps and the secondary path is closed for it. LRPS2's documentation lists "Memory Monitoring (achievements)" as supported and
  "RetroArch Cheats" as not, while its core information file says `memory_descriptors = "false"` [R]. Read together
  [A]: it exposes memory by `retro_get_memory_data` rather than by maps, which RetroAchievements reads as the PS2's main
  RAM, and `retro_cheat_set` does nothing, so the secondary path is closed for it; its own `.pnach` patches, kept in
  `system/pcsx2/cheats/` by disc hash [R], stay the core's, offered through `assets/` and shown as "the core's own".
  What the two PCSX2 libretro builds expose is already measured [M, `Sapphire_Referees.md` §2]: LRPS2's `SYSTEM_RAM` is
  33,570,816 bytes from address 0, and PCEE2's is 33,554,432 bytes with memory maps at 0x00000000 (32 MiB) and
  0x70000000 (16 KiB, the scratchpad), and with no BIOS it exposes none. Which build comes first is the tester's
  question; it was decided the same day: PCEE2 first. The mapping, for PCEE2's maps and for LRPS2's main RAM alone: the PS2's main RAM at
  physical 0x00000000–0x01FFFFFF, reached by codes through its kernel-segment and uncached views (0x20000000,
  0x30000000 mirrors), resolves to space `RAM`, which is Sapphire's cheat space too; a code naming the scratchpad
  (0x70000000) or IOP RAM is refused by name, "this core does not expose the scratchpad". The PS2's systems-table entry
  has no codecs yet (`ps2.cheat_systems` is empty): the unencrypted forms are decodable from published descriptions and
  would serve Sapphire and the hosted core alike, which is a system-pack step of its own (§11, Q3).

### 4.10 The environment calls

**Answered**: `GET_CAN_DUPE` (true), `SET_PIXEL_FORMAT`, `GET_SYSTEM_DIRECTORY`, `GET_SAVE_DIRECTORY`,
`GET_CORE_ASSETS_DIRECTORY`, `GET_CONTENT_DIRECTORY`, `GET_LIBRETRO_PATH`, `SET_INPUT_DESCRIPTORS`,
`SET_CONTROLLER_INFO`, `GET_INPUT_BITMASKS`, `GET_INPUT_MAX_USERS`, `GET_INPUT_DEVICE_CAPABILITIES`, `GET_VARIABLE`,
`GET_VARIABLE_UPDATE`, `SET_VARIABLES`, `SET_VARIABLE`, `GET_CORE_OPTIONS_VERSION` (2), `SET_CORE_OPTIONS`,
`SET_CORE_OPTIONS_V2`, the `_INTL` forms (the English text), `SET_CORE_OPTIONS_DISPLAY` and its update callback,
`SET_GEOMETRY`, `SET_SYSTEM_AV_INFO`, `GET_LOG_INTERFACE` (the log queue, from any thread), `SET_MESSAGE`,
`SET_MESSAGE_EXT` and `GET_MESSAGE_INTERFACE_VERSION` (1) (log records of category `message`, which a frontend shows as
notices), `GET_PERF_INTERFACE`, `SET_FRAME_TIME_CALLBACK` (the nominal frame time each frame, never a measured one),
`GET_AUDIO_VIDEO_ENABLE`, `GET_FASTFORWARDING` (false), `GET_THROTTLE_STATE` (the core's own rate),
`GET_TARGET_REFRESH_RATE` (the core's own rate), `GET_LANGUAGE` (English, the schema's language), `SET_SUPPORT_NO_GAME`
(`starts_empty`), `SET_MEMORY_MAPS` and the `RETRO_MEMORY_*` regions (§4.9), `SET_SERIALIZATION_QUIRKS`,
`GET_SAVESTATE_CONTEXT`, `GET_GAME_INFO_EXT`, `SET_CONTENT_INFO_OVERRIDE`, `GET_JIT_CAPABLE` (true),
`SET_PERFORMANCE_LEVEL`, `SET_SUPPORT_ACHIEVEMENTS`, `SET_PROC_ADDRESS_CALLBACK`, `SHUTDOWN` (the next `advance`
answers a status saying the core stopped), `SET_MINIMUM_AUDIO_LATENCY` (recorded), `GET_CURRENT_SOFTWARE_FRAMEBUFFER`,
`SET_DISK_CONTROL_INTERFACE`, `SET_DISK_CONTROL_EXT_INTERFACE` and `GET_DISK_CONTROL_INTERFACE_VERSION` (1) (onto
`MEDIA`: the lid onto the eject state, an inserted disc onto an added image by path, so multi-disc games change discs as
on the console; LRPS2 lists disk control, PCEE2 `disk_control = "false"` [R]), `SET_HW_RENDER`, `GET_PREFERRED_HW_RENDER` (Vulkan),
`SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE`, `GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT` (2),
`GET_HW_RENDER_INTERFACE` (§5), `SET_HW_SHARED_CONTEXT` (accepted: one context), and from stage 5
`GET_RUMBLE_INTERFACE`.

**Refused**, each logged once per machine with the reason, as libretro specifies for a frontend without the call:
`SET_ROTATION` and `GET_OVERSCAN` (no rotated or cropped picture in v1), `GET_USERNAME` (nothing of the player's is handed
to foreign code), `SET_KEYBOARD_CALLBACK`, `SET_AUDIO_CALLBACK`, `SET_AUDIO_BUFFER_STATUS_CALLBACK`, the sensor, camera,
location, LED, MIDI, microphone and device-power interfaces, `GET_VFS_INTERFACE` (the core's own file I/O stays inside
its sandbox), `SET_SUBSYSTEM_INFO` loads, `SET_FASTFORWARDING_OVERRIDE`, `EXEC_MEM_ALLOC`/`FREE`,
`SET_NETPACKET_INTERFACE`, the HDR, VR, video-view, video-decode and upscaler calls, OpenGL, OpenGL ES and Direct3D
contexts, and any command the adapter does not know. A refusal a core cannot run without (a GL-only core) fails `create`
with words naming the call.

### 4.11 Errors, reported

Every refusal of §4.10 is a log record; every failed `create` names its cause in `create`'s error (the core's hash, a
missing firmware folder, a refused context, the image path absent); the libretro core's own log reaches the host's log
queue with its level; Vulkan failures and the runner's end reach `last_error` and a status. Nothing a core asks for and
does not get passes unrecorded.

## 5. Hardware rendering through Vulkan

### 5.1 What libretro asks

A Vulkan core calls `SET_HW_RENDER` with `RETRO_HW_CONTEXT_VULKAN`, usually after `GET_PREFERRED_HW_RENDER`, and may set a
negotiation interface. With version 2 the core supplies `create_instance` and `create_device2` and builds its own
`VkDeviceCreateInfo`, which it hands to a wrapper of the frontend's that may add what the frontend needs; "the frontend
must not disable features requested by application" [R]. The core's queue must support graphics and compute [R]. After
creation the frontend calls `context_reset`, and the core takes `retro_hw_render_interface_vulkan` (version 5): the
device, the queue and its lock, `set_image` (the frame's image, view, layout, semaphores to wait for, queue family), the
frame-in-flight index (`get_sync_index`, `_mask`, `wait_sync_index`), `set_command_buffers` and `set_signal_semaphore`.
A frame is handed by `video_refresh(RETRO_HW_FRAME_BUFFER_VALID, w, h, 0)`. PCEE2 with its default renderer asks for
Vulkan 1.1 and, refused, still runs [M, `Sapphire_Referees.md` §2.3]; its `.info` says the core negotiates its context
[R]. LRPS2 lists Vulkan 1.3 among its requirements and offers ParaLLEl-GS, a compute renderer on Vulkan [R].

### 5.2 Why not the lent device

`EmuSen_Gpu.md` §5's lent device cannot serve a libretro core as built: (1) the lender (Serenity) lends a queue of a
compute family and keeps the graphics queue (`lend_queues` [R]), and the RX 6800 under RADV has one graphics queue
(`EmuSen_DevicePicture.md` §1.3 [M]); (2) the lent device is made at the frontend's start with timeline semaphores and the
external-memory extensions only, while a negotiating core names its features at `retro_load_game`; (3) a shared queue
behind a lock across the ABI would be a callback (`EmuSen_DevicePicture.md` §5.2). And a `VkDevice` cannot cross into
the runner's process at all. So the core's device is its own, on the host's card, and the picture crosses as memory.

### 5.3 The core's device

1. **The card**: the physical device whose `deviceUUID` and `driverUUID` match those the host gives with the ring
   (§5.4); with no ring, `emusen-gpu`'s choice (`EMUSEN_GPU_DEVICE`, then its ranking).
2. **Instance and device** through the core's `create_instance` and `create_device2` where offered, the adapter's
   wrapper adding `VK_KHR_external_memory_fd`, `VK_KHR_external_semaphore_fd` and timeline semaphores and nothing that
   changes behaviour; through v1's `create_device` otherwise; made by the adapter, with one graphics-and-compute queue,
   for a core with no negotiation interface. A card that lacks the external extensions is reported, and the picture is
   read back (§5.5). Then `context_reset`.
3. **The interface** the core receives is the adapter's: the queue's lock a mutex; two frames in flight, each with its
   fence, which `wait_sync_index` waits for; `set_command_buffers` queued for the frame's submission.

### 5.4 The picture on a ring the host owns (E5)

**The form** [C], `DEVICE_RING`:

- **The host makes the ring** on its presenting device (Serenity's): three images, `R8G8B8A8_UNORM`, optimal tiling,
  dedicated memory exportable as `OPAQUE_FD` (`OPAQUE_WIN32` on Windows), sized to machine info's `max_width` and
  `max_height`, and two timeline semaphores, also exportable: `ready`, which the core signals, and `done`, which the host
  signals. It passes them once by `emusen_core_ring_lend(machine, const emusen_ring_lend *)`: the descriptors, each
  image's create parameters, allocation size and memory type, and the device's `deviceUUID` and `driverUUID`. The broker
  carries the descriptors across by `SCM_RIGHTS`. The host owns the memory, as the ABI's ownership rules want (§6.17); a
  larger maximum (`MACHINE_INFO`) is answered with a new ring.
- **The core imports** the images and semaphores into its device once. Each frame, in the frame's submission: wait for
  the core's semaphores and for `done` to reach the value the slot last held; **acquire** the slot from
  `VK_QUEUE_FAMILY_EXTERNAL` with an image barrier; take the core's image to a transfer source (with an ownership transfer
  from its queue family where it differs); **one copy on the device** of the frame's rectangle into the slot
  (`vkCmdCopyImage`, a blit where the core's format is not `R8G8B8A8`); **release** the slot to
  `VK_QUEUE_FAMILY_EXTERNAL`; return the core's image to its layout; signal `ready` with the frame's number. The host makes
  the matching acquire from and release to `VK_QUEUE_FAMILY_EXTERNAL` on its own device each frame, so ownership crosses
  explicitly on both sides, as the specification asks for memory shared between devices.
  `emusen_core_frame_ring(machine, emusen_ring_frame *)` answers the slot, the rectangle and the value.
- **A larger picture mid-game.** `SET_SYSTEM_AV_INFO` with larger maxima can arrive inside `retro_run`. The runner grows
  its shared region for software pictures there and then; for the ring, a frame larger than the ring's images is held, not
  cut, and the machine raises `MACHINE_INFO` with the new maxima; the host lends a new ring, and the held frame is copied
  into it at the next `frame_ring`. A host that does not lend a new ring gets the read-back for such frames (§5.5).
- **The host** waits for `ready` on its device and reads the slot as the input of its chain (`EmuSen_DevicePicture.md`
  §4), then signals `done` with that value once it no longer reads it. **The CPU never waits for the GPU**: both waits
  are on the devices, as in `EmuSen_DevicePicture.md` §4.2. A host that stops signalling stalls the core's device and
  then its machine, and corrupts nothing (§6.29's rule, kept). **After a crash or a kill** (§3.4) the core will never
  signal `ready` again, so the host signals its own `ready` from the CPU (`vkSignalSemaphore`) up to the value its queue
  waits for, before it destroys the ring; otherwise the presenter's queue would wait forever and the driver call it lost.
- **Two boundaries, not two forms** (decided 2026-10-31): `DEVICE_PICTURE` (1.9) stays the form within one process and one
  device, for the project's cores; `DEVICE_RING` is the form across processes and devices, for hosted cores. A core
  claims the one its boundary needs.
- **The one copy is inherent**: a libretro core makes its own images, not exportable ones, so its image can be shared only
  by copying it on the device into memory that can (0.02–0.05 ms at 1080p for such a copy, `EmuSen_DevicePicture.md`
  §4.1 [R]). No copy passes through the CPU.
- **What the PS2 cores hand.** PCEE2, the first target, gives a picture from stage 2. LRPS2 hands the frontend no software picture: refused a hardware context it falls back to
  its software renderer and still hands nothing by frame 600, while PCEE2 with `software` pinned hands 640×448 XRGB8888
  [M, `Sapphire_Referees.md` §2.3]. So LRPS2 shows nothing through this host until stage 3; PCEE2 shows its software
  picture from stage 2.

### 5.5 The read-back, where sharing is impossible

Only where the host lends no ring (a host without the minor, a host without a Vulkan presenter), or the card lacks the
external extensions, or the platform has no shared handles (macOS, where no Vulkan driver is shipped,
`EmuSen_DevicePicture.md` §4.4), the copy goes into a host-visible buffer instead and the picture is handed by
`frame_copy`. Its fence is waited for at the frame's end; if that costs the emulation thread more than a tenth of a frame,
the picture handed becomes the frame before's, a `latency` setting the player chooses. Which path a machine uses is in
its log and its details window.

### 5.6 Who touches which device

| Thread | Device | What |
|---|---|---|
| the runner's main thread, inside `advance`/`create`/`free` | the core's | `retro_run`; the frame's copy into the slot; context creation and teardown |
| the core's own threads | the core's | its submissions, under the adapter's queue lock |
| the host's presenter | the host's | the ring's creation; the wait on `ready`, the chain, the signal of `done` |
| the host's emulation thread | none | the calls to the proxy |

## 6. Held to EmuSen's standards where the host decides

| Standard | How |
|---|---|
| the sound rate exact, converted without drift | E4's exact ratio into the frontend's one resampler; the drift test (§4.3) |
| frame pacing by the presenter | the exact `frame_rate` (§4.1); the nominal frame time to `SET_FRAME_TIME_CALLBACK`; nothing in the adapter sleeps or paces |
| save states versioned and checked | the header of §4.6: identity, versions, digest, platform; a failed load restored |
| controls through Endymion | the canonical controls and axes (§4.4); rumble back through Endymion (stage 5) |
| settings in EmuSen's model | the schema of §4.7 in the per-console settings |
| saves owned by the host | named battery files (§4.5) |
| cheats by EmuSen's engine | spaces from the core's memory (§4.9) |
| errors reported, never hidden | §4.11 |
| a foreign core contained | §3 |

## 7. Distribution and licences

A libretro core is downloaded only when the player asks, from the libretro buildbot
(`buildbot.libretro.com/nightly/<platform>/<arch>/latest/<core>_libretro.<ext>.zip`, and the core information archive
beside it), into the folder of §1.1 by the Cores window's core tool, which bounds the archive (exactly one member named
`<corename>_libretro.<ext>`, no folder, link or absolute path in its name, 512 MB at most, `EmuSen_Cores.md` §5.2), then
approved as §1.3 describes, showing its name, authors, licence as its
`.info` states (PCEE2: GPLv3, "PCSX2 Team|WizzardSK"; LRPS2: GPLv3, "PCSX2 Team" [R]) and its SHA-256. `THIRD_PARTY_NOTICES.md` §2 gains a row: libretro cores
and their core information, from buildbot.libretro.com, each under its own licence. Firmware is never fetched or
shipped (§4.8). Tests never download: the test core is built from the repository, and measurement uses the PCEE2
and LRPS2 nightlies the probe already pins.

## 8. Platforms

Linux first. Kept possible: Windows (`.dll`; named pipes and `DuplicateHandle` for the broker; a job object; `OPAQUE_WIN32`
for the ring) and macOS (`.dylib`; the socket pair; read-back, §5.5). Nothing in the core ABI changes per platform. LRPS2
is x86-64 only [R], which covers the desktop and the handheld.

## 9. Tests

- **The test core** (GPL-3.0, Rust, built in the test run, so no commercial core or game enters the repository) has an
  option for each behaviour to exercise: a deterministic picture from a frame counter and the input in each pixel
  format, drawn by copy or into `GET_CURRENT_SOFTWARE_FRAMEBUFFER`; sound through both calls at a chosen rate, whole or
  fractional; save RAM, RTC, and files written, changed and removed in its system and save directories; states, with
  a load that fails half-way; options v2 with categories, hidden options and `SET_VARIABLE`; log lines with formatted
  arguments from a thread of its own; geometry and timing changes at set frames; duplicate frames; `SHUTDOWN`; two discs
  through disk control; memory exposed by maps, or by `SYSTEM_RAM` alone, or not at all, with a game loop whose values
  the cheats change visibly; `retro_cheat_set` honoured or ignored; the crash and sandbox behaviours of §3.5; a Vulkan
  picture drawn with `vkCmdClearColorImage` on rectangles (no shader to build) through negotiation version 2; and
  **every environment command of §4.10, answered and refused**, called in a table-driven pass whose answers it writes
  into its memory for the test to read and compare with the design's table.
- **In Rust**: the libretro host's own tests through `emusen-core-host`, in-process and through the broker; the broker's
  tests on a project core (MoonRT) beside the test core, the window compared with direct round trips; the conformance
  kit (`emusen-core-conform`) on the test core, in-process and brokered.
- **Through WiseMan, headless**: a test class through the C# host: pictures, sound and its drift, input, saves and named
  files, states and their refusals, settings, cheats by both paths (a poke, a compare byte, each conditional, a code
  refused by name and one sent to `retro_cheat_set`, and the path each reports), crash and sandbox, the ring with the
  validation layer on the RX 6800; and PCEE2 (then LRPS2) when present, the BIOS path an argument and nothing commercial committed or
  listed.
- **Seeded faults**, three to seven a stage, aimed at what each stage could break.
- **Measurement**: PCEE2 on Jak and Daxter's arena, its software renderer from stage 2 and Vulkan from stage 3, milliseconds a frame on the desktop (two rounds) and on the handheld,
  at each stage that renders, with the bench used for `Sapphire_Speed.md`'s arena figures, against RetroArch's own
  figure for the same core (`EmuSen_LibretroBridge.md` §1); the arena reached by an input script or a state the core
  itself saved. The broker's round trip and the ring's hand-over are measured on their own.
- **Cheap checks first**: a futex ping-pong between two processes over a shared page, the floor of P1's 50 µs (stage 2);
  a 20-line Vulkan probe of whether RADV exports a timeline semaphore as `OPAQUE_FD`
  (`vkGetPhysicalDeviceExternalSemaphoreProperties` with a timeline type), which the ring rests on (before stage 3).

## 10. Stages

| Stage | What | Gate |
|---|---|---|
| **1** | The libretro host, in-process: software pictures, sound, input, saves and named files, states, options, firmware, memory spaces and both cheat paths, every environment call; the test core; minor 1.16's E1–E4 in the header, the Rust definitions, the C# host and the ABI baseline; `emusen-native`'s per-machine snapshot kinds | the kit and the libretro host's tests pass on the test core; the PS2 core loads, and one probe run with `find` records what it writes |
| **2** | The broker: protocol, frame-end window, sandbox, crash containment; the libretro host behind it; the C# host's 1.16; options a core declares in `retro_load_game` kept in its private folder for the next load (§4.7) | the kit through the broker; §3.5's proofs in WiseMan, the `strace` reading among them; the round trip and the window's race check measured; the PS2 core loads, runs and frees in the .NET host (PCEE2's software picture shown and timed on Jak's arena; LRPS2's none until stage 3) |
| **3** | Vulkan: the core's device, `DEVICE_RING` (E5) in the ABI, the Rust host and Serenity, and the read-back where sharing is impossible | the test core's Vulkan pictures match its software ones on both paths, clean under synchronisation validation; PCEE2 shows Jak's arena through Vulkan, timed on the desktop and the handheld; LRPS2 shows its first picture |
| **4** | Frontend: the core list and download, the approval of the whole entry, the settings window from the schema, the exact rate in the resampler, cards in the card manager, cheats with their path shown, big picture, the engine choice per console; the fit and pad audits | Mistress and big picture run a game either way; cards follow the save-media rules; the choice is remembered |
| **5** | Rumble: `EmuSen_CoreAPI_Extensions.md` §4's event 7 (minor 1.17) through Endymion to the pad | a DualShock 2 game's rumble reaches a pad; the test core's rumble pattern read back by the test |

## 10a. Stage 1, as built (2026-10-31)

**Built**: `emusen-libretro-sys` (the interface, its layouts held to gcc's for `libretro.h` [M]); `emusen-libretro-testcore`;
the core ABI's minor 1.16 (E1–E4, `EmuSen_CoreAPI.md` §6.41) with the Rust host and the kit; the systems table's libretro
names and `for_libretro`; and `emusen-libretro-host` in-process: pictures, sound, input, save RAM and named files, states,
options, memory spaces, discs through disk control, and every environment command answered or refused.

**What building it changed in the design** [M]:
- **Images are copied, not borrowed.** The kit's C16, run with `BORROW` claimed, aborted the process: a libretro core may
  keep every image it was given until it is unloaded, which `BORROW`'s loan per drive cannot promise. `BORROW` is not
  claimed; a `need_fullpath` core reads by path and copies nothing, and a small image is copied once.
- **The state header records the disc** (§4.6), from C16's state over an empty drive; and a restore passes the tray, since
  a libretro core changes discs only with it open.
- **`retro_get_memory_data`'s blocks are asked again** after a load and a reset: the test core moved its memory on a
  load, and the space read freed memory until it was.
- **`retro_deinit` per machine**, and `retro_init` again at the next create: the test core and PCEE2 both load again.
- **Hardware rendering's commands are refused** until stage 3 builds them; PCEE2, refused, "falling back to readback
  present", hands software pictures.

**Tests** [M, 2026-10-31]: 22 unit tests and 19 through the Rust host on the test core, among them every environment
command of §4.10 compared with this page's table, the exact rate over 3,600 frames at 48,000 and 32,040.5 Hz with no
sample of drift, and a load refused half-way restored byte for byte. The kit on the test core passes every case but C7,
C10, C12 and part of C14, each of which holds two machines in one process, which a libretro core cannot (§1.1) and the
broker of stage 2 can. Seeded faults, each caught by the test aimed at it: the X byte left unwritten, a drain crossing a
rate change, a refused load not restored, a changed save not raised, the payload's digest unchecked, an option update
never reported.

**PCEE2, measured** [M, the pinned nightly `00275dbf…`, no firmware, a one-instruction loop ELF built by the runner]:
60 frames in 0.65 s; 640 × 480 pictures in BGRA8888 from the software renderer; 48,000 Hz exact; one space, `RAM`,
33,554,432 bytes; it asks for Vulkan, an audio buffer status callback and rumble, each refused and logged. It names its
folders under the system directory (`pcsx2/bios`, `memcards`, `sstates`, `snaps`, `logs`, `cheats`, `patches`) and
reports no `resources` folder; with no firmware it wrote no file, so its profile (the memory cards in
`system/pcsx2/memcards/` as `ps2.card` saves) waits for a run with the player's BIOS.

**Not done in stage 1**: the profile of PCEE2's cards (above); options declared in `retro_load_game` recorded for the next
load (they are noted, not yet kept; carried to stage 2, §10); the C# host's 1.16 (the image path and named files), which comes with stage 2's
WiseMan class; and the secondary cheat path, which needs a way for the host to hand a core a code it cannot decode, which
v1 has not (§11a).

## 10b. Stage 2, as built (2026-10-31)

**Built**: `emusen-core-broker` (the protocol, the runner, the client with its deadlines, the Landlock confinement, the
frame-end window, and `broker_exports!`, which writes a proxy's exports for a target and its optional groups) and
`emusen-libretro-proxy` (the library a host loads for an entry: the sealed copy of the core, the runner's policy, the
`"libretro"` member of core info). Options a core declares while loading are kept in its folder and join the schema at
every later load (§4.7).

**What building it changed** [M]:
- **A private `/dev/shm`.** PCEE2 failed to map its machine under the first ruleset ("Failed to create shared memory
  file"): it creates `/dev/shm/pcsx2_<pid>`. Opening `/dev/shm` to a core would open every other program's segments to
  it, so the runner makes a user and a mount namespace and a tmpfs over `/dev/shm` of its own before Landlock, and only
  then grants it; where user namespaces are unavailable, `create` fails in words.
- **`/proc/cpuinfo`, `/proc/meminfo` and `/proc/stat`** are readable: PCEE2 reads the first to know the processor.
- **The ruleset, traced** with `strace -f -e trace=openat,ioctl,connect` (strace 7.3 unpacked user-locally) on PCEE2
  brokered: the only refusals left are `/dev/input/event*`, the host's input devices, which the core reaches through the
  host; refused, as they should be. The Vulkan half of §3.3 is traced in stage 3, with the render node in use.
- **The runner's descriptors** are moved above 100 before being placed at 3 and 4: a sealed copy that happened to be
  descriptor 3 was overwritten by the socket, and the core loaded the socket.

**Measured** [M, 2026-10-31, the RX 6800 machine, debug builds of the proxy and runner]: a round trip 8.9–9.4 µs (P1,
under 50 µs, holds); 1,000 reads of a held range 12.1 ms through round trips and 0.064 ms through the window; PCEE2 with
no firmware, 60 frames in 0.74 s brokered against 0.68 s in-process (start-up included). Memory held still between two
reads 0.5 ms apart in each of 1,000 frames, for the test core brokered and for PCEE2 in-process (1 MiB of `RAM`; with no
firmware, so its threads were idle; to be repeated with the player's BIOS before the window is relied on for PCEE2).

**Tests**: the kit passes all 24 cases on the test core through the broker (C7, C10 and C12 among them, which stage 1
could not); 10 brokered tests: the hash of the sealed copy in core info, two machines side by side, a fault the core
handles running on, a crash and an abort ending with `-10` and the signal while the host runs on, a hung frame stopped at
a one-second deadline with `-11`, a write outside the core's folders and a network connection refused by the kernel, a
kernel that cannot confine the core failing `create`, the window exact across writes and a load, and the race check.
Seeded faults, each caught: no deadline on a read, Landlock skipped, the window outliving a load, a stale window (the
first seed of it, a redundant line, was not a fault; the second was caught), the descriptors placed over each other.

**Not done in stage 2**: the C# host's 1.16 (the image path, named files stored by name, the two new obligations) and
the WiseMan class through it; the host's comparison of core info's `"libretro"` hash with `cores.approved` (§1.3, step 2),
which belongs with the approval the Cores window shows; the frame's picture and sound batched into `advance`'s reply
(each is one round trip now, about 9 µs).

## 10c. PCEE2 with the player's firmware (2026-10-31)

Approved the same day: the referee folder's Americas BIOS and the Jak and Daxter disc, each given by path only. The core's
system folder is the referee's (`EMUSEN_LIBRETRO_SYSTEM_DIR`, a referee run's switch of the libretro host's that answers
`GET_SYSTEM_DIRECTORY` with a path and copies nothing), the BIOS chosen by the core's own option `pcsx2_bios`; the folder
was never listed, opened or hashed by the project's tools, and no RAM or disc data was kept.

**Timing** [M, the release libretro host in-process through the Rust host, PCEE2's `software` renderer, frames 600 to
2,400 of Jak and Daxter's start with no input, CPU weight 20 beside other work]: 10.37, 5.43 and 6.50 ms a frame over three
rounds (the first two differed, so a third was run); every round under the 16.7 ms of a 59.94 Hz frame. The arena needs a
route of input or a state of this core's own, which a later stage records; the start's frames match the span RetroArch's
own figure was taken over (`EmuSen_LibretroBridge.md` §1).

**What running it changed** [M]:
- **Memory maps arrive after load.** PCEE2 gives its maps only once the game runs; the host now takes maps or blocks that
  arrive after `create` and raises `MACHINE_INFO`. Its space is `EE RAM`, 33,554,432 bytes.
- **The race check fails for PCEE2.** The first MiB of `EE RAM` moved between two reads 0.5 ms apart in 62 of 1,000
  frames (34 with MTVU off): its emulation runs on threads of its own between `retro_run`s. So the frame-end window is
  off for it, and for every core not measured still; it stays on for the test core, which was. Cheats and the debugger
  read and write PCEE2's memory live, as RetroArch's do, and a value can change between a read and the frame's end:
  named in its losses.
- **Its saves.** PCEE2 writes its two memory cards, `pcsx2/memcards/Mcd001.ps2` and `Mcd002.ps2`, 8,650,752 bytes each,
  in its system folder, and nothing in its save folder. Its profile makes them battery files 2 and 3, kind `removable`,
  medium `ps2.card`, slots 1 and 2. Sapphire's card reader opens both as unformatted cards of the right length, which
  is consistent with one format but does not prove it; the two engines share a card only after a card formatted and
  written by one is read by the other (stage 4).

## 10d. Stage 2b: the C# host's 1.16, named files stored, the entry's trust (2026-10-31)

**Built**:
- **`CoreInterface` at 1.16**: `File.Name`, `CreateParams.ImagePath` (the struct 120 bytes), `Media.ImagePath`,
  `CapImagePath` (bit 26), `FileNamed` (`0x8000`); `CoreAbiTests` hold them to the baseline. `HostObligations` gained
  `image_path` and `named_files` only with the engine below, so a hosted core's `host_requires` is now met by the C# host.
- **`CoreMachine`** passes the image's path to a core claiming `IMAGE_PATH`, at create and at `media_insert`, and the named
  files with their names (the names pinned beside the bytes for the call). `CoreMachineInfo`'s battery entries read `name`
  and `removed`; core info's read `family`, and core info's `"libretro"` member (`CoreInfo.Libretro`).
- **The path is given only for an unaltered file** [C]: Galaxia's `DiscFiles.UnalteredPath` is the image's file where it
  was neither unpacked from an archive nor patched, else null. An unpacked image is a copy whose hold ends when a core
  that copies has taken its bytes, so a path into it could be pruned under a core reading by path; such a game reaches a
  `need_fullpath` core with no path, and the core's `host_requires` refusal says why.
- **Named files stored by name** (`EmuSen/Cores/NamedFiles.cs`, the folders in Galaxia's `SaveLibrary`): a family of
  kind `console` under `Saves/<console>/Console/<core id>/<name>`, `removable` under `Cards/<core id>/`, `game` in
  `<the game's save>.files/`. Every stored file is passed at create in ordinal order of names, numbered from
  `FILE_NAMED`, at most 32,768; `SaveSram` writes each listed file that changed (by `BATTERY_DIRTY`'s flags) on a
  `BATTERY` event and at the flush period; a file listed `removed` has its stored copy moved to `<folder>.removed/`, so it
  is not passed again and nothing of the player's is deleted. Names are held to §6.41's rule before any write, and no
  part of a path that is a link is followed, read or written. A console or removable family sets
  `CREATE_SHARED_CONSOLE`, since its files are shared by the core's games. `--nobattery` passes and keeps none. A family
  entry is not a slot: the card manager's media and the listed battery files leave it out.
- **The entry's trust before any runner starts** (§1.3, with `EmuSen_Cores.md` §5.2 and §7: the same `cores.approved`,
  the same `sha256sum` lines, the libretro core's own hash). Discovery lists each `/Cores/libretro/<stem>/` holding a
  sidecar for its proxy copy (`LibretroEntries`). Before the proxy is loaded, `DiscoveredCore.Open` requires: the proxy
  copy to be the sidecar's; both copies to be libraries an installation's `cores.manifest` lists under their built names
  (else *needs refresh*); the sidecar's `libretro.sha256` approved (else *needs approval*); the libretro file on disk
  still that hash (else *needs approval*, a new build). Only then is the proxy loaded, which starts the library's runner;
  its `info()` must equal the sidecar's, which carries the hash of the sealed copy the runners load, so a file changed
  between the host's hash and the proxy's seal is refused as altered. That interval, between two reads of one file in
  the player's folder, is the one place a changed build can run (confined) before the host refuses it; closing it would
  need the host to hand the proxy the approved bytes, which v1 has no call for. *Named* as a limit [A].
- **`RustCores.props`** gains a `Role`: `part` (the proxy and the libretro host: built, published, in `cores.manifest`,
  no sidecar), `program` (`emusen-core-runner`, beside the assemblies, where the proxy looks for it) and `test` (the test
  core: build output only, never published or in the manifest). The rows are Linux's alone while Landlock is the only
  confinement the broker has; `rust-cores.yml`'s plan is not extended until another platform can confine a core.
- **The test core** logs the game's path when given one, so a test can see which games reach a core by path.

**Tests** [M, 2026-10-31]: `LibretroHostTests` lays out an entry as Cores does, approves the test core's hash, writes the
sidecar by loading it once, and runs it brokered through the C# engine, headless; 10 tests pass: the entry trusted by
its core's hash; a rebuilt core, an unapproved one and a stale copy each refused with the proxy never loaded; a game's
picture, pad and file path; named files stored, passed back and a removed one's copy kept; a crash and an abort ending
with `-10` and the signal while the engine loads the game again; a hung frame stopped at a one-second deadline with
`-11`; a write outside the core's folders and a network connection refused by Landlock; a kernel that cannot confine the
core failing the load in words. `NamedFilesTests` (14) hold the name rule, links and the unaltered path. The runner's
environment is set with libc's `setenv`, since .NET's `SetEnvironmentVariable` does not reach native code on Unix.
Seeded faults, each caught by the test aimed at it: a named file passed without its name (the core refused the game), no
image path given, the libretro file's hash not compared (the proxy loaded, then refused as altered: caught, and the
reason the comparison precedes loading), the adapter copy not checked, a removed file's copy not kept.

**Left for later**: the Rust host's (`emusen-core-host`) trust of entries, which the shell's `cores` listing needs with
Cores' stage C5; the listing's `"libretro"` field (`EmuSen_Cores.md` §7.3); `audio.rate_exact` in the resampler and
cards in the card manager (stage 4).

## 11. Risks

1. **The round trip** per frame and per cheat window miss; measured at stage 2 against P1.
2. **Assets** a core needs beyond firmware are known only by running it; PCEE2's resources folder at stage 1.
3. **Determinism**: the PCSX2 builds run threads of their own, so its runs may differ; its kit report is a report, not a gate.
4. **Card images**: whether a PCSX2 build's cards are Sapphire's format is unknown until compared (§4.5).
5. **Landlock** may be absent from the handheld's kernel; then no hosted core runs there, which `create` says.
6. **The GPU** is shared: a reset the core causes takes the presenter's device too (§3.6).

## 11a. Deferred surfaces, each with its trigger

| Surface | What it is | Built when |
|---|---|---|
| `CHEAT_CODES` | an optional capability with `emusen_core_set_cheat_codes(machine, text, len)`: the codes no codec of the console decodes, one per line with its index and switch, handed to a core that decodes its own (`retro_cheat_set`), the core answering which it took | a target core honours `retro_cheat_set` (decided 2026-10-31; PCEE2's `.info` says `cheats = "false"`) |

## 12. Questions for the tester

1. **[decision]** Every libretro core in a process of its own, behind a generic broker (`emusen-core-broker`), with the
   Linux sandbox; this replaces the first version's in-process design and its one-machine limit. *Recommended:* yes.
2. **[decision]** Minor 1.16 with E1–E5 (image path, unrated options, named battery files, the exact sound rate, the
   host-owned picture ring), and rumble as minor 1.17 in stage 5. *Recommended:* yes.
3. **[decision]** The PS2's unencrypted cheat formats as a system-pack step, shared by Sapphire and the hosted core, so
   that EmuSen's own codes reach the hosted PS2 core. *Recommended:* yes (approved 2026-10-31), after stage 1 has recorded
what PCEE2 exposes.
4. **Decided 2026-10-31: deferred.** The secondary cheat path's surface, `CHEAT_CODES` (§11a), is built when a target core
   honours `retro_cheat_set`.
