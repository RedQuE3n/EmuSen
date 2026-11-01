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

### 0.3 Decided 2026-11-01: two cores, and their pictures through the console's filters

**Two cores, named in a list.** The libretro host supports exactly two cores for now: **PCEE2**, the libretro build of
upstream PCSX2 (PlayStation 2), and **Flycast**, upstream Flycast's libretro build (Dreamcast, with Naomi and Atomiswave).
They are listed in one data file of the systems table, `Platform/systems/data/libretro-cores.tsv` (the core's file
stem, the systems it runs, its upstream source; `EmuSen_Cores.md` §4.7), read by its one reader,
`emusen_systems::libretro_cores::hosted()`. Cores offers only those, and every host refuses an entry whose stem the list
does not hold, in plain words, **before** the approval and hash checks of §1.3, which are unchanged. The Rust host calls
`hosted()`; the C# host reaches it through a platform-library export, `emusen_systems_libretro_cores`, a minor of
`emusen_platform.h` (accepted 2026-11-01, built once Cores' list has landed), and a test lays out the test core under a
list of its own, never a build's. Adding a core later is one line and its tests, with no change of code; a field the host
comes to need (a least interface version, a hardware context) is added to the file and its reader together [C].

**Stage 3 is designed for these two and no other** [C]. Both draw with Vulkan; Flycast can also draw with OpenGL, which
the host does not offer, so it is asked for Vulkan (`GET_PREFERRED_HW_RENDER`) and a request for any other context is
refused in words. Generality that only another core would need (an OpenGL context, a core with no negotiation interface,
formats neither core hands) is left out of §5 and named in §11a, built only when a listed core needs it.

**The picture reaches the console's screen filters**, the accurate CRT filter among them, keyed on the console (the
systems table's `ps2`, and `dreamcast`, `naomi` and `atomiswave`), never on the core: a hosted PlayStation 2 game and Sapphire's are one
console to the filter. What the filter needs with each frame is carried by the ring's frame record (§5.4): the
console, the picture's native size, and whether it is one field or two woven. With the CRT filter on, the host sets the
core's own options so that the picture is the console's: its native internal resolution and no deinterlacing, each such
option a **named exception** in the core's profile (the core's key and value, the reason, the date), the same for every
game, never per game; with the filter off the core's upscaling is the player's to choose. The option keys are taken
from each core's published option list when stage 3 records them [C].
- *PCEE2*: native resolution and deinterlacing off give the GS's own output, which the PlayStation 2 profile already
  expects (640 or 512 wide, 448, 224, 512 or 256 tall, `CrtFilter`'s timing rows). A height of 224 or 256 is one field;
  448 or 512 is a frame, its field structure read from the picture's height against the mode's.
- *Flycast*: the Dreamcast's picture is a television's (interlaced 480, or progressive 480 where the game and cable allow)
  or VGA's (progressive 640 × 480). Which one is the core's cable option, so the host reads it with the picture: TV
  interlaced is two woven fields at 640 × 480 with deinterlacing off, TV progressive and VGA are progressive frames.
  The Dreamcast's own profile is separate work; the picture path carries the console, size and field structure it will
  need.

**What the Dreamcast needs of the project** beside the core [C], none of it the core's: the systems table's entries
`dreamcast`, `naomi` and `atomiswave` (Cores' list names them), each with its libretro databases, extensions and media;
the Dreamcast pad as a controller of the table, its stick and **two analog triggers** onto Endymion's `PadAxis`, and the
arcade boards' sticks and buttons; the VMU as a removable medium (`dc.vmu`), Flycast's VMU files taken as named battery
files under a profile as PCEE2's cards are (§10c), read with a run of the core; a console screen row and its CRT profile
(separate work). Firmware is never fetched or shipped: Flycast boots without the console's own (its high-level boot), and
a player's own dump is given only as a path, through the firmware settings (§4.8).

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

### 4.1a Which games a hosted core takes (2026-11-01)

A hosted core takes a game file of a system when three conditions hold: its line in `libretro-cores.tsv` names the
system; the file's name ends, past a dot and ignoring case, in one of the extensions its own `.info` lists under
`supported_extensions`; and no named exception of `libretro-extensions.tsv` drops that extension for that core and
system (one may also add one). The rule is one pure function in `emusen_systems::libretro_extensions`,

    pub fn takes(stem: &str, supported: &[String], system: &str, game: &Path) -> bool

with `info_extensions(info_text)` reading the `.info`'s value and `takes_under` the same rule over lists a test supplies.
The ordering of a console's engines (§4.8 of `EmuSen_Cores.md`: the native core first, PCEE2 first on the PlayStation 2
while Sapphire is developed) is the discovery's, which calls this function to filter and keeps its own order.

**Sources.** The extensions are the cores' public `.info` files (libretro-core-info), never a core's source. Two are
cited by the tests: PCEE2 lists `iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx`, so it refuses a `.cue` and a cue/bin set
reaches it through its `.bin`; Flycast lists `chd|cdi|iso|elf|bin|cue|gdi|lst|zip|dat|7z|m3u`, for each of `dreamcast`,
`naomi` and `atomiswave`, and refuses a `.ccd` although the Dreamcast's table row lists one.

**Exceptions as data.** Each line names a core, one system its line names, the extensions added and dropped, and the
rule that sets it; the list is checked against `libretro-cores.tsv` when read, and a line of another shape is refused
with its number. The shipped file names none: no rule of EmuSen's yet sets one, and every difference from a core's own
list is to be a named exception decided by the tester, never one per game.

**Design notes.** (1) The rule lives in `emusen-systems` because the list it reads does; the discovery, which knows the
installed `.info`, passes its value in, so the function reads no file. (2) `info_extensions` reads one key with the
quoting of `emusen-core-tool`'s `CoreInfo` (first key kept, `#` lines skipped); that reader carries the network and TLS
crates the catalogue needs, which the systems table should not depend on. (3) A suffix match lets a listed compound
extension (`iso.gz`) and a single one (`gz`) both work without a second rule.

**Tests** (`libretro_extensions::tests`, 9): each listed extension of both cores, the `.cue` refusal and the `.bin`
path, a system off a core's line, names with no extension, the `.info` value quoted, bare, commented and absent, an
exception's adding and dropping for its system only, a malformed exception refused, and the shipped file read whole.
Seeded faults, each reverted, each caught by one test: the dot before a suffix not required; the system condition
removed; drops ignored; a commented `.info` line read; case not folded.

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
and a core that asks for them is shown as asking, never silently. Rumble (since 1.17, §10e): `GET_RUMBLE_INTERFACE` answered,
each port then declaring its two motors, and each motor's level raised as event 7 when it changes; the host sends it to
the player's pad through Endymion (stage 5).

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
`GET_HW_RENDER_INTERFACE` (§5), `SET_HW_SHARED_CONTEXT` (accepted: one context), and since 1.17
`GET_RUMBLE_INTERFACE` (§10e).

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

### 5.4a `DEVICE_RING` as specified for minor 1.18 (2026-11-01)

Bit 27, kept since 1.17, in a minor of its own. Decided 2026-11-01: the ring is built on exported images that the core's
device imports, never on a read-back, and it is measured against the in-process `DEVICE_PICTURE` path for frame time and
for the delay from input to picture.

```c
#define EMUSEN_CAP_DEVICE_RING (1ull << 27)   /* emusen_core_ring_lend, emusen_core_frame_ring */
typedef struct emusen_ring_image {           /* 40 bytes; one exported image of the ring */
    uint32_t size;
    uint32_t memory_type;                    /* the memory type index on the card, the same for both devices */
    uint64_t handle;                         /* the memory's OPAQUE_FD (OPAQUE_WIN32 on Windows), the host's */
    uint64_t allocation_size;                /* of the dedicated allocation */
    uint32_t width, height;                  /* the image's extent, machine info's maxima at lending */
    uint32_t format;                         /* VkFormat, R8G8B8A8_UNORM */
    uint32_t usage;                          /* the VkImageUsageFlags it was made with; tiling optimal */
} emusen_ring_image;
typedef struct emusen_ring_lend {            /* 72 bytes */
    uint32_t size;
    uint32_t count;                          /* images, three */
    const emusen_ring_image *images;
    size_t image_size;                       /* sizeof(emusen_ring_image) as the host knows it */
    uint64_t ready;                          /* timeline semaphore the core signals, OPAQUE_FD */
    uint64_t done;                           /* timeline semaphore the host signals, OPAQUE_FD */
    uint8_t device_uuid[16], driver_uuid[16];
} emusen_ring_lend;
typedef struct emusen_ring_frame {           /* 40 bytes */
    uint32_t size;
    uint32_t slot;                           /* the image holding the picture */
    int32_t width, height;                   /* the picture, from the image's top left */
    uint64_t value;                          /* ready reaches it once the slot holds the picture */
    uint32_t fields;                         /* 0 a progressive frame, 1 one field, 2 two fields woven */
    uint32_t field;                          /* with one field: 0 the top, 1 the bottom */
    uint32_t aspect_num, aspect_den;         /* the picture's display aspect; 0/0 for square pixels */
} emusen_ring_frame;
int32_t emusen_core_ring_lend(emusen_machine *machine, const emusen_ring_lend *ring);
int32_t emusen_core_frame_ring(emusen_machine *machine, emusen_ring_frame *out);
```

- **Handles are the host's.** The core duplicates what it keeps before importing it, since an import takes the handle it is
  given; the host closes its own when it frees the ring. A ring lent again replaces the last after every picture handed on
  it has been released (`done` reached its value); a null ring withdraws it, and the machine hands its pictures by
  `frame_copy` again. A ring whose UUIDs are not the core's card's is refused with `NOT_SUPPORTED`, and the read-back
  (§5.5) serves.
- **`frame_ring`** after `advance` answers 1 with the picture in the ring as `out` says, 0 when this frame's picture is
  not there (none, or `frame_copy` has it), or a status. The values count the pictures handed on this ring from 1.
- **The console's picture.** `fields` and `field` are what the screen filters need beside the console (machine info's
  `system`) and the size (§0.3); a libretro core says nothing of fields, so the libretro host derives them from the size
  against the console's modes and the core's own options the profile pins (PCEE2: 224 or 256 rows one field, 448 or 512
  a frame; Flycast: its cable option, TV interlaced two fields woven, TV progressive and VGA progressive).
- **A larger picture** (§5.4): held, `MACHINE_INFO` raised with the new maxima, copied at the first `frame_ring` after the
  host lends a ring that holds it.
- **Through the broker** the handles cross by `SCM_RIGHTS` on the control socket, in a message of their own after the
  `RingLend` request, which the runner receives with `recvmsg`; the struct's handles are rewritten to the runner's.

**The probe** [M, 2026-10-31, the RX 6800 under RADV, Vulkan 1.4.354, validation with synchronisation checks on, a
40-line `ash` program run headless]: a timeline semaphore is exportable and importable as `OPAQUE_FD`
(`EXPORTABLE | IMPORTABLE`); one made exportable on a device, exported, imported into a second device on the same card,
signalled there to 7 from its queue, was waited for and read as 7 on the first, with no validation message; an
`R8G8B8A8_UNORM` image of optimal tiling with transfer and sampling usage can have its memory exported and imported as
`OPAQUE_FD`, as a dedicated allocation only (`DEDICATED_ONLY`), which the ring's dedicated memory already is.

**The measurement** (decided 2026-11-01), on the RX 6800 and headless: the test core's Vulkan picture through the broker
and the ring, against the same picture handed in-process by `DEVICE_PICTURE` (a project core's form), for (1) the frame's
cost on the host's emulation thread and (2) the delay from `set_buttons` to the first picture showing the press, read
on the host's device; and PCEE2 on Jak's arena through the ring.

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

### 10d.1 Decided 2026-10-31: the interval closed at 1.17

Decided by the tester the same day: a changed file must never run, even confined; the interval §10d names is closed in
minor 1.17, beside rumble (E6). Two forms were weighed [A]:

- **The runner waits for the host's go** after the proxy reports the sealed copy's hash: a new step in the broker's
  protocol and a new call for the host to give it, with the library's runner, which `info()` needs, held at the step too.
- **The approved hash given to the proxy before it runs anything** [C, chosen]: an optional library-level export,
  `emusen_core_expect(const char *sha256)` (capability bit 28, `EXPECT`), which a host calls right after loading and
  before any other export, as `set_crash_log` may be. The proxy seals its copy at that call and compares; on a mismatch
  it starts no runner, and every later call answers as a broken library with the two hashes in words. Loading the proxy
  runs only the project's own initialisers, which `cores.manifest` vouches for, so nothing of the libretro core runs
  before the comparison. `create` needs no field, since every runner loads the copy sealed at `expect`.

The second is one call and no change to the protocol, and it keeps the comparison in the proxy, which holds the
bytes. A host that does not call it (an older one) gets today's behaviour, the post-load check of §10d. The C# and Rust
hosts call it for every entry from the sidecar's approved hash. Proof: a test that swaps the libretro file between the
host's hash and the load, and sees no runner started and the load refused; and a seeded fault, the comparison skipped.

Also decided: the Rust host's trust of entries, the listing's `"libretro"` field and `rust-cores.yml`'s Linux-only
entries come with Cores' stage C5, or at the latest in stage 3.

## 10e. Minor 1.17: the approved hash given first, and rumble (2026-10-31)

**Built** (`EmuSen_CoreAPI.md` §6.42):
- **`EXPECT`** (capability bit 28), `emusen_core_expect(const char *sha256)`, and `EMUSEN_NOT_EXPECTED` (-268). The broker's
  proxy answers it for any target that can compare what it serves (`Target::expect`): the libretro proxy seals its copy
  of the core at that call and compares the copy's SHA-256 with the approved one. On a mismatch the proxy keeps the two
  hashes and starts nothing: every library call that would start the library's runner, and every `create`, answers
  `NOT_EXPECTED` with them; `info` is a broken library's naming both; `status_text(-268)` gives the same words. A later
  `expect` with another hash is refused and changes nothing. The proxy's capabilities are the served library's and
  `EXPECT`, which it adds to core info's list, so the host's comparison of bits and names holds.
- **The hosts.** `emusen-core-host`'s `Library::load_expecting` and the C# `CoreLibrary.Open(path, expect)` call it right
  after loading, before any other export, where the library exports it; a library loaded before in the process under
  another approval, or none, is refused rather than trusted twice. `DiscoveredCore.Open` passes the sidecar's approved
  libretro hash for an entry, and `CoreSidecar.Write` takes the hash too, so that the load that writes a sidecar is held
  to it as well. A library without the export, an older proxy, keeps §10d's check after loading.
- **Rumble** (E6): event kind 7, `OUTPUT`, and machine info's `ports[].outputs`, as `EmuSen_CoreAPI_Extensions.md` §4.3
  designed them; `emusen-native`'s `Core::outputs` and `Core::output_level`, the crate raising an event for each declared
  output whose level changed, and every declared output after machine info is read. The libretro host answers
  `GET_RUMBLE_INTERFACE`; a core that took it has each port declare `Strong motor` and `Weak motor`, and a motor's level is
  the strength the core last set, read at the frame's end [C]: libretro's interface gives a level and no time within the
  frame, so the frame's mean that §4.3 asks of a project core is not known to the host; a core that sets a motor once
  per frame, as cores do, loses nothing. The levels are atomics, since a core may set them from threads of its own. The
  kit's play loop refuses an `OUTPUT` that names an output machine info does not declare, or a level outside 0 to 65535.
- **Bit 27** stays kept for `DEVICE_RING`, which comes with stage 3 in a minor of its own.

**Tests** [M, 2026-10-31]: a libretro entry whose core file is changed after its hash was taken and before the proxy is
loaded is refused by `load_expecting` in words naming both hashes; its `info` is a broken library's, its `create`
answers `NOT_EXPECTED`, and no runner of this process is alive that was not before (read from `/proc`); the same in
WiseMan through `CoreLibrary.Open` (`LibretroHostTests`). Every other brokered test now loads its entry with its hash
expected. The test core's `tc_rumble` drives both motors of two ports in a pattern; every level arrives as an `OUTPUT`,
only changes are raised (63 events over 30 frames), and machine info declares the motors on both ports. The
`emusen-native` test core declares a motor and a lamp, and its seeded fault, an `OUTPUT` for an output not declared,
fails C6. Seeded faults, each caught: the proxy's comparison skipped (the swapped core loaded); the library calls left
ungated after a failed `expect` (a runner started and served the swapped core's info); the C# host not calling
`expect` (`LibretroHostTests`: the swapped core loaded); levels raised once and never on a change (the rumble test, at
frame 2). WiseMan's blast radius (`LibretroHostTests`, `CoreAbiTests`, `CoreAdapterTests`, `CoreDiscoveryTests`,
`CoreHostParityTests`, `NamedFilesTests`): 89 of 89.

**Left**: the runtime's routing of `OUTPUT` to the seated player, Endymion's motors and the Preferences rows (stage 5,
`EmuSen_CoreAPI_Extensions.md` §4.4); C8's digests of the output stream (§4.5 there); the Rust host's trust of entries,
which calls `load_expecting` (with Cores' C5).

## 10f. Stage 3: the core's Vulkan device and the ring, as built (2026-11-01)

**The broker carries the ring** (`emusen-core-broker`). Two operations are appended, `RingLend` and `FrameRing`. A
ring's request holds its images but their handles and the two UUIDs; the descriptors follow in a one-byte message of
their own (`SCM_RIGHTS`, order: images, `ready`, `done`), which the runner receives with `recvmsg` (`MSG_CMSG_CLOEXEC`)
before it answers, writes over the host's handles, and closes after the core's `ring_lend`, since the core duplicates
what it keeps. The proxy checks the struct as `emusen-native` does, at most sixteen descriptors, each one open, before
anything is sent; a count other than the request's is refused, never left in the stream. The runner's Landlock rule for
device nodes gained `ReadDir`: a Vulkan driver lists `/dev/dri` to find its render node, and without it the confined
core saw no device at all (measured 2026-11-01).

**`libretro_vulkan.h`** is in `emusen-libretro-sys::vulkan` on ash's types (no loader), its layout held to gcc 16.2.1's
sizes and offsets for the header (96, 48, 48 and 136 bytes).

**The core's device** (`emusen-libretro-host::vulkan`). `SET_HW_RENDER` is accepted for Vulkan only, any other context
refused in words; `GET_PREFERRED_HW_RENDER` answers Vulkan; the negotiation interface is taken at version 2 or read up
to `destroy_device` below it. After a successful `retro_load_game` the device is made: the core's `create_instance`
(the wrapper raises the API version to 1.1 at least, nothing else), then the card `emusen-gpu` chooses
(`choose_device`, since emusen-gpu 1.5: `EMUSEN_GPU_DEVICE`, then its ranking), then the core's `create_device2`, whose
wrapper adds `VK_KHR_external_memory_fd`, `VK_KHR_external_semaphore_fd` and `VK_KHR_timeline_semaphore` where offered
and turns timeline semaphores on, in the core's own `Vulkan12Features` or timeline struct where its chain has one (the
only write into the core's structures, setting a feature, never clearing one), else in a struct put in front; v1's
`create_device` and a device made here are the fallbacks. Then `context_reset`. Teardown: `context_destroy`,
`retro_unload_game`, the negotiation's `destroy_device`, the device. The interface: two frames in flight, each with its
fence (`wait_sync_index`); the queue lock a spin lock; `set_command_buffers` submitted ahead of the host's copy in the
frame's one submission, which waits for the core's semaphores and signals its `set_signal_semaphore`.

**The copy.** A frame that fits the ring goes into slot `count mod n`, waiting on the device for `done` to reach the
value that slot last held. Ownership crosses `VK_QUEUE_FAMILY_EXTERNAL` in barriers that keep the layout
(`SHADER_READ_ONLY_OPTIMAL` both ways), with the transitions to and from the copy as barriers of their own, so the
release and acquire on the two devices name the same layouts as the specification asks; a slot's first use is a plain
transition from `UNDEFINED`. `vkCmdCopyImage` for an `R8G8B8A8` picture, `vkCmdBlitImage` (nearest) for any other.
`ready` is signalled with the picture's number. **No CPU wait** on this path. A frame larger than the ring's images is
copied into an image of the core's device (held), read back for `frame_copy`, and `MACHINE_INFO` raised with the larger
maxima; the first `frame_ring` after a ring that holds it is lent copies it in. **With no ring** (no lend, a ring
refused, a card without the extensions) the picture is read back (§5.5): copied into a host-visible buffer and its fence
waited. `frame_ring`'s field structure is `fields_of(system, height)`: PlayStation 2, 224 or 256 rows one field, 448 or
512 two woven, else progressive; the Dreamcast's waits for Flycast's cable option, whose key is recorded with the core.
Aspect from the geometry's `aspect_ratio`.

**The host's side is Serenity's** (`emusen-serenity::ring`): `Device::create_ring(width, height, count)` makes the images
with dedicated memory exported as `OPAQUE_FD` and the two timeline semaphores, exported; `Ring::crossing` gives the
ownership barriers for the chain; `Ring::read` is the check a test or a details window makes (wait `ready`, take, copy
out, give back, signal `done`); `Ring::release_after_crash` raises `ready` from the CPU before a ring of a dead core is
destroyed. Serenity's device now keeps its driver UUID and whether timeline semaphores are on (`rings()`). Feeding a
slot into the chain as its input is the next step (§5.4).

**The test core** draws its software picture into an image of its own device (`tc_hw`: `vulkan` RGBA, `vulkan_bgra`
BGRA), uploaded and handed by `set_image`, its device made through `create_instance` and `create_device2`.

**Tests**, on the RX 6800, with validation and its synchronisation checks clean (`VK_LOADER_LAYERS_ENABLE=*validation`,
`VK_KHRONOS_VALIDATION_VALIDATE_SYNC`, `VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`): in-process, six frames' ring
pictures equal to the software pictures by copy and by blit; with no ring and with a ring whose driver UUID is another
card's (refused `NOT_SUPPORTED`), the read-back equal to the software picture; a picture larger than the ring held and
copied into the next; through the broker, five frames across the process boundary equal. Seeded faults, each caught:
the descriptor count unchecked; a BGRA picture copied unswizzled; a held picture never copied; the woven PlayStation 2
heights read as one field; the runner passing the host's handle numbers.

**Measured 2026-11-01** (the RX 6800 under RADV, release builds, the test core's 320 × 240 picture, 540 frames after
60 of warm-up, two rounds agreeing within 0.03 ms; `measure_the_ring_against_the_read_back_and_software` and
`measure_the_ring_through_the_broker`, run with `--ignored`). The cost is `advance` and then `frame_ring` (ring) or
`frame_copy` (software, read-back) on the host's emulation thread; the delay is from `set_buttons` to the picture on the
host's side, which for the ring includes `Ring::read` waiting for `ready` on the host's device and copying the picture
out, so it overstates what a presenter that samples the slot would see.

| Path | In-process cost, mean / p99 (ms) | Delay, mean (ms) | Brokered cost, mean / p99 (ms) | Delay, mean (ms) |
|---|---|---|---|---|
| software (`frame_copy`) | 0.107 / 0.121 | 0.106 | 0.217 / 0.364 | 0.222 |
| read-back (§5.5) | 0.264 / 0.389 | 0.277 | 0.406 / 0.642 | 0.415 |
| ring | 0.121 / 0.160 | 0.256 | 0.150 / 0.247 | 0.291 |

Through the broker the ring costs the emulation thread less than a software picture, since no picture crosses the
socket; the press shows in the frame it was made in on every path, this core reading its pad in `retro_run`. A defect
the first round proved: the read-back mapped host-visible memory that was not cached, and reading it cost 17.9 ms a
frame; taking cached memory where offered brought it to 0.26 ms. The comparison §5.4a names, against
`DEVICE_PICTURE`, follows.

**The ring against `DEVICE_PICTURE`, stage 3's acceptance check** (decided 2026-11-01). *One picture on both paths.*
An image of emusen-native's test core whose payload begins with `PAINTED` (0xD1) draws the libretro test core's
320 × 240 picture: red the column plus the frame, green twice the row plus forty for each life (three, one lost every
60 frames, three again after none), blue the pad's low byte against the column divided by eight, alpha 255. Its machine
keeps the picture in host memory for `frame_copy`; `device_test_core`, an example of `emusen-libretro-host` built from
the same machine, claims `DEVICE_PICTURE` and hands it on the lent device: copied on the lent queue from host memory
into one of two buffers both families read, after `done` has reached that buffer's last picture (§6.29 of
EmuSen_CoreAPI.md). It is built here, where ash already is, so that none of the seven crates that build
`v1_test_core` by path gains a Vulkan dependency; the shared machine names its painter through a `PAINTER` constant
each includer sets, `None` but here. `the_v1_test_cores_device_picture_is_the_libretro_test_cores_software_picture`
holds the two pictures equal over 130 frames with presses, so the formula written in both cores has one test as its
owner across the two ABIs; a device picture of the wrong lives, a copy cut short and a picture handed under a stale
value each fail it (seeded 2026-11-01).

[M, 2026-11-01, the RX 6800 under RADV, release builds, 540 frames after 60 of warm-up, two rounds;
`measure_the_ring_against_device_picture`, run with `--ignored`.] The cost is `advance` with the sound drained, then
`frame_ring`, or `frame_device` and Serenity's `Lend::take` (the intake the emulation thread makes, §17 of
EmuSen_DevicePicture.md); the delay is from `set_buttons` to the picture in host memory, by `Ring::read` or by
`Lend::read`, the same check added to the lend.

| Path | Cost, mean / p99 (ms), rounds 1 and 2 | Delay, mean / p99 (ms), rounds 1 and 2 |
|---|---|---|
| ring (libretro host, its core's own device) | 0.122 / 0.167; 0.121 / 0.153 | 0.257 / 0.328; 0.255 / 0.301 |
| `DEVICE_PICTURE` (v1 test core, Serenity's lent device) | 0.100 / 0.168; 0.100 / 0.129 | 0.778 / 1.009; 0.797 / 1.233 |

The ring costs the emulation thread 0.02 ms more a frame (the libretro host's calls, and its frame's one submission
carrying the copy into the slot) and brings the press to host memory 0.52 ms sooner. The lent path makes three
submissions on two queues (the core's upload on the lent queue, the intake's copy into Serenity's own buffers on its
intake queue, the read) where the ring makes two (the core's copy into the slot, the read); the difference is put down
to the extra copy and its crossing between queues, not separated further. **The ring is accepted**: within 0.02 ms of
the lent device on the emulation thread, and sooner to the picture. A finding on the way: with its sound left
undrained, the test core's queue, full at 65,536 values, dropped two from its front for every pair it made, and the
first round's `DEVICE_PICTURE` cost read 0.66 ms; a host drains every frame, so the measurement does.

**A ring's slot as the chain's input** (2026-11-01). `Chain::advance_ring(InRing)` makes the picture a core's
`frame_ring` answered the chain's next frame; `Ring::picture(slot, value, width, height)` gives the `InRing` (the slot's
image, `ready` to wait for, `done` to signal), and `InRing::record` the crossing `Ring::crossing` names: the slot taken
from `VK_QUEUE_FAMILY_EXTERNAL` in `SHADER_READ_ONLY_OPTIMAL`, moved for a transfer read, copied, moved back, given back.
Nothing is copied at `advance_ring`: the copy into the history rides the next render's or present's one submission,
as a lent picture's does (§13 and §17 of EmuSen_DevicePicture.md), that submission waiting for `ready` at the transfer
stage and signalling `done` with the picture's value; `run_unwaited`, the present's submission, gained a timeline
signal beside its binary one for it. `Ring::read` is now the same `record` with a copy into host memory. A ring outlives
every frame a chain advanced from it and has not rendered. The platform library carries it as interface 28
(`EmuSen_RustPlatform.md` §42): `emusen_serenity_ring_new`, `_image`, `_free` and `emusen_serenity_chain_advance_ring`,
with `SlangRing`, `SlangChain.AdvanceRing` and `CoreMachine.RingLend`/`FrameRing` on the C# side.

*Proof, headless, keyed on the console* (`RingChainTests`, the RX 6800): the test core's entry with `systemid =
"playstation2"` in its `.info`, brokered through the C# host as the frontend runs it, reports the system `ps2`, whose
systems-table console is "PlayStation 2"; its `vulkan` picture goes into a 640 × 480 ring of the slang device across
the process boundary, and for six frames each frame from the ring, drawn through the accurate CRT filter written for
"PlayStation 2", through CRT Lottes for it and through no filter, is byte for byte the same frame's software picture
from a machine lent no ring, uploaded and drawn through the same filter; through no filter it is the picture itself.
Seeded faults: the copy into the history a column short, which fails it at frame 0; and `done` never signalled,
which hangs it (the core's device waiting on a slot never given back, the host's render on a `ready` never raised)
until the test host was stopped, so that fault shows as a run that never ends, not as a failure. A finding: with a ring lent, the libretro host does not read the ring's picture back, so its
`frame_copy` holds no picture (0 bytes in the first run of the test); §6.43 of EmuSen_CoreAPI.md promises `frame_copy`
only with no ring lent, and a screenshot or a state's picture of a ringed frame needs a read-back on demand, put to the
tester as a question (§12).

**PCEE2 on Jak and Daxter through the ring** [M, 2026-11-01, the RX 6800, the release libretro host in-process through
the Rust host, `examples/run.rs` with `EMUSEN_RUN_RING=1`, PCEE2's `vulkan` renderer at `pcsx2_upscale_multiplier=1`,
the BIOS given by path through `pcsx2_bios` and the referee system folder as in §10c, frames 600 to 2,400 of the start
with no input, CPU weight 20 beside a landing sweep]: 5.150 and 5.004 ms a frame over two rounds, of which 4.054 and
4.395 ms on the emulation thread (`advance` and `frame_ring`; the rest is `Ring::read`, the check copy); 2,398 of 2,400
pictures in the ring, 512 × 448 at 4:3. The ring was lent at the core's declared maximum, 2,732 × 2,048, and lent again
when `MACHINE_INFO` came. PNGs of the ring's picture at frames 600, 1,200 and 2,399 (the last the title, "Press Start",
drawn by PCEE2's Vulkan renderer) are kept outside the repository, in `~/.cache/emusen/probe/libretro/ring-png/`.
Two findings: (1) PCEE2 aborted at exit while its device was destroyed before `retro_deinit`; the device now outlives
`retro_deinit` and the run exits cleanly. (2) Without the core's `.info` the machine's system is `libretro-pcee2`, so
`fields_of` reads its 448 rows as progressive; with the entry's `.info` naming the PlayStation 2 it is `ps2` and two woven
fields.

**PCEE2 on Jak and Daxter from a real entry** [M, 2026-11-01, the RX 6800, the release libretro host in-process through
the Rust host, `run.rs` with `EMUSEN_RUN_RING=1`, the entry laid as Cores lays one (the host's copy, `pcee2_libretro.so`
and its published `pcee2_libretro.info`, whose `systemid` is `playstation2`), `pcsx2_renderer=vulkan`,
`pcsx2_upscale_multiplier=1` and `pcsx2_deinterlace_mode=1` (Off, the CRT path's exception of §0.3), the BIOS by path as
before, frames 600 to 2,400 with no input, CPU weight 20 beside a WiseMan test run]: machine info's system is `ps2`, and
every ring frame kept (600, 1,200 and 2,399) is 512 × 448 with `fields` 2, two woven fields, at 4:3, as `fields_of`
reads the PlayStation 2's 448 rows. 4.067 and 4.087 ms a frame over two rounds, of which 3.471 and 3.497 ms on the
emulation thread; 2,399 of 2,400 pictures in the ring. With the core's deinterlacing off the frame is lighter than the
first run's (5.15 and 5.00 ms with its automatic mode). The PNGs are in `~/.cache/emusen/probe/libretro/ring-png-info/`.
Two findings: (1) PCEE2 draws its own notice into the game's picture for the first seconds of the title, that the
detected blending accuracy is Basic where High is recommended for this game, with no option in its published list to
turn notices off; the CRT path will draw it as part of the picture, and whether `pcsx2_blending_accuracy` belongs among
the profile's named exceptions is put to the tester (§12). (2) `frame_info` at the end reads 0 × 0, the ring's frames
never read back, as `RingChainTests` found (§12, question 5).

## 10g. Stage 4: the frontend, planned (2026-11-01; a plan, nothing built)

Stage 4 puts a hosted core in front of the player through components that exist; it adds no component. Each part
names the surface it uses and what is new on it. Nothing reaches past a surface: Mistress knows a hosted core only as an
EmuSen core with an entry, and the libretro host stays the only code that knows libretro.

**4.1 Launching a hosted core from Mistress's library.** The library already routes a game to a system and the system
to a core (`CoreCatalog`, `CoreDiscovery`); a libretro entry is a discovered core whose `.info` names its systems
(`ps2`, and for Flycast `dreamcast`, `naomi`, `atomiswave`). New: (a) the systems table's `libretro-cores.tsv` list
(§0.3) decides which entries Cores offers and which the hosts accept, refused in words before trust; (b) where a system
has both a project core and a hosted one (PlayStation 2: Sapphire and PCEE2), the per-console core choice the settings
already keep picks, the project core the default; (c) the game is started through `CoreEngine` as today, the broker and
proxy unchanged. Tests: a WiseMan launch of the test core's entry from a synthetic library (its `.info` naming `ps2`)
through the same path a click takes, headless; an unlisted entry refused in words. Exit: the test core and PCEE2 start
from the library with no libretro knowledge in Mistress.

**4.2 Its settings window from the options schema.** The libretro host already publishes the core's options as a
settings schema (§4.7) and `ICoreSettings`/`CoreCatalog.SettingsFor` already build the per-console Graphics Settings
window from a schema. New: (a) the schema's groups (libretro's option categories) become the window's sections, a value
list a choice, and the restart flag a note "applies at the next start"; (b) the named exceptions the CRT path sets
(4.4) shown locked with their reason while the CRT filter is on; (c) the window built through `AdaptiveLayout`, with its
big-picture counterpart and pad navigation, and the fit audit at 1280 × 800 and 1920 × 1200. Tests: the window built
from the test core's schema (every option type it declares), the fit audit, the locked rows. Exit: PCEE2's published
options adjustable from the window, kept per console.

**4.3 Memory cards and VMUs as named files.** Named battery files (1.16) and removable media (`ps2.card`, `dc.vmu`)
already exist; PCEE2's two cards are battery files 2 and 3 of kind `removable` in its profile (§10c). New: (a) the
Memory Card manager lists a hosted core's cards beside Sapphire's, through the save-media service, which already reads
named files; (b) a card formatted and written by PCEE2 read by Sapphire's card reader and the reverse, the proof §10c
left owed, run on synthetic cards only; (c) Flycast's VMU files taken the same way once Flycast is installed (its
profile entry and the `dc.vmu` medium, §0.3). Tests: round trips of a synthetic card through both engines; a card kept
across a state load under `CREATE_SHARED_REMOVABLE`. Exit: one card file per slot, shared by both PlayStation 2
engines, never inside a state.

**4.4 The CRT path end to end.** Stage 3 built the pieces (the ring, `advance_ring`, the field structure, the C# glue of
interface 28) and proved them headless. New: (a) `GameFrameControl`'s device path takes a ring: the runtime lends the
ring at the core's declared maximum and again at `MACHINE_INFO` (as `run.rs` does), and after each frame the presenter
advances its chain from `frame_ring` where it now advances from a lend; (b) the field structure carried to the
filter: `fields` and `field` from the record into the CRT filter's structural values, so a woven 448-row picture is
drawn as two fields, the same as Sapphire's; (c) with the CRT filter on, the named exceptions set on the core (native
internal resolution, deinterlacing off, from PCEE2's published keys), restored when it is turned off; (d) a ring
withdrawn, and the ring's `ready` raised (`Ring::release_after_crash`) before it is freed, when the core's process dies;
(e) a screenshot, a state's picture and rewind's thumbnail of a ringed frame: the libretro host reads nothing back once
a frame is in the ring, so `frame_copy` is empty (§10f); the plan is a read-back on demand, the frame's held picture
copied into host memory only when `frame_copy` is asked for, which needs the host's `frame` to take the picture by
mutable reference, a change inside `emusen-native`'s trait to be weighed with the tester before it is made. Tests: the
C# frontend's device path presenting a ring frame headless (as `DevicePictureLendTests` does a lend), the CRT filter's
two-field drawing of a 448-row picture against the same picture uploaded with its field structure, a crash with a ring
lent, a screenshot of a ringed frame. Exit: PCEE2's Jak and Daxter through the PlayStation 2 CRT profile in Mistress,
measured against the 16.7 ms frame on the desktop and the handheld.

**Order and cost.** 4.4(a) and 4.1 first, since every later check needs a game running in the window; then 4.4(b)–(e),
4.2 and 4.3. Each part ends with its WiseMan tests and the blast radius, as stage 3 did.

## 10h. Stage 4: the frontend, as built (2026-11-01)

**A ringed frame read back on demand** (§12 question 5, decided 2026-11-01). A frame copied into the ring now takes its
shape and a new serial in the machine's picture at once (`Picture::drawn_unread`), its bytes left where they are; the
core's image it came from is kept as the frame's unread source (`Gpu::unread`), which stays the core's until its next
frame. Only when `frame_copy` asks does `Machine::frame` read that image back (`Gpu::read_unread`, the read-back of
§5.5 with no core commands in it) and take it into the picture, keeping the serial, since a read-back is not a new
frame. The ring's own path reads nothing back. This needed `Core::frame` in `emusen-native` to take the core by mutable
reference (EmuSen_CoreAPI.md §6.43); every core's implementation changed with it, the C declaration unchanged.
Test: `a_ringed_frames_picture_is_read_back_when_frame_copy_asks_as_a_screenshot_does` (`tests/ring.rs`, the RX 6800),
six frames by copy and by blit, every frame's shape and serial right before any read-back, every second frame's
`frame_copy` (a screenshot's call) equal to the software picture, the ring's picture unchanged beside it. Seeded
faults, each caught: the read-back never made (frame 1's screenshot differs) and a ring frame leaving the picture's
shape as it was (frame 0's shape).

**The core build's SHA-256 in every state** (header version 2, 112 bytes). The header gains the SHA-256 of the
libretro core's build that saved it (`header.build`, bytes 80 to 112, from the library's hash of the sealed copy that
runs). A state whose name and version match but whose build differs is refused (`FOREIGN`) in plain words naming both
builds: "this state was saved by another build of PCEE2 2.3.1 (SHA-256 …), and the build installed is …; a libretro
core's states load only into the build that saved them". Every other refusal was reworded for the player the same way.
A version-1 state is refused as another version of the host; none reached a player. Tests: `state.rs`'s unit tests,
`a_state_loads_whole_and_each_refusal_is_its_own` (`tests/testcore.rs`), and through the frontend API
`A_state_loads_and_one_of_another_build_is_refused_in_plain_words` (`HostedCoreLaunchTests`), which finds the recorded
hash equal to the entry's and reads the refusal's words as Mistress shows them. Seeded fault: the build left unchecked,
caught by the integration test.

**Launching from the library (4.1), proven headless.** `HostedCoreLaunchTests` opens a synthetic PlayStation 2 DVD
(`SyntheticPs2Disc`, its unused system area beginning `TCG1` so the test core takes it; the test core now claims `.iso`
beside `.tcg`) through the frontend API as `MainWindow.OpenSession` does, with the engine the console's setting names:
the session reports the console "PlayStation 2", the system `ps2` and the hosted engine, and plays its 320 × 240 picture.
No code in Mistress or the runtime changed for it; Mistress knows the core only as an engine. Which engine the PlayStation
2 runs when nothing is stored (PCEE2 while Sapphire is under development) is branch `hosted-cores`' default rule, and the
refusal of an entry off `libretro-cores.tsv` waits for that list to land (§0.3); neither is duplicated here.

**Pause and fast forward (5).** In the same test: fast forward at 300 % runs the core; paused, three frames run none and
the picture's number holds; resumed, it runs. *Frame advance is not offered:* Mistress has no frame-advance command, and
the frontend API's only near equivalent, the debugger's run-to-frame (`RunToFrame`), resumes the session and did not
hold it after one frame for the hosted core (measured 2026-11-01: three more frames ran), so it is left out and put to the
tester (§12 question 7).

**A screenshot of a ringed frame in the C# frontend.** `RingChainTests` now takes `GetFrameBufferRgba` (the
screenshot's call) of every second ringed frame and finds it the frame.

**Not built in this pass**, each with what it needs: 4.4(a) the ring in Mistress's device path, which runs through the
frontend API's session and so needs the API to lend a ring and hand `frame_ring` (an extension of `emusen_crystal.h`,
proposed before building); 4.4(b)-(d), the field structure into the CRT filter, the named exceptions set with the filter
on, and the ring's release when the core's process dies, which follow 4.4(a); 4.2, the settings window's sections from
libretro's categories and the locked rows, whose `CoreSetting.Group` already carries a group; 4.3, the cards in the
card manager and the round trip between PCEE2 and Sapphire, which needs a run of PCEE2. PCEE2 was not run.

**The ring through the frontend API (4.4(a), (b), (d); the API's 1.4, 2026-11-01).** Decided 2026-11-01: the frontend
API lends a ring as an optional capability (`EMUSEN_CRYSTAL_OPEN_RING`, `picture_ring`, the frame record's ring fields;
`EmuSen_Backend.md` §33, written before the code). Both implementations make the ring on the presenter's device at
machine info's maxima, lend it in place of the device lend where the core claims `DEVICE_RING`, again when a
`MACHINE_INFO` raises the maxima, and free it after the machine, the last one kept one change more. Mistress opens every
session with the flag when it lends a device, offers each ringed frame to the frame control as a `RingPicture`, which
`SlangRunner.Present` advances its chain from (`advance_ring`), and names the path "in a ring" in its log. *Found while
building:* a ring frame holds its slot until a chain has taken it or a later frame, so a client that stops drawing on
the device stalls the core after its third frame (shown by a seeded fault: frame 5 never returned); Mistress gives the
slots back through `SlangRingRelease` while its screen is off the device. *Field structure (b):* the frame record
carries `fields` and `field` to the client; the CRT filter already draws a picture taller than 300 rows as two woven
fields and a shorter one as one, which agrees with the record for every PlayStation 2 mode the host derives (448 and
512 rows two fields, 224 and 256 one), so no structural value was added. *A dead core (d):* the Rust raises the ring's
`ready` to the last value handed before freeing it; the C# cannot (no export does), recorded in §33.2.
Tests: `ApiRingTests` (the RX 6800, the C# implementation): six frames of the test core's Vulkan picture in the API's
ring drawn through no filter and the PlayStation 2 CRT filter equal to the same frames read back by `picture_copy` and
uploaded; a session without the flag lends none; twelve ring frames given back with the screen off the device, none
stalled; one frame advance. `CrystalParityTests`' frames case through both implementations, the header, contract and
hand-off classes, `RingChainTests`, `HostedCoreLaunchTests` and the device-lend classes beside it. Seeded faults, each
caught: the release doing nothing (frame 5 stalls); the frame's slot reported one off (227,600 bytes differ); the ring
frame never recorded (the record's `picture_in_ring` 0).

*Superseded by §10k, 2026-11-01: the Rust runtime discovers, admits and opens hosted cores.* **Mistress's default runtime cannot start a hosted core yet.** Mistress runs the frontend API's Rust implementation
(`EmuSenCrystalNative`), and the Rust core host's discovery knows no libretro entry, so a hosted core opens only on the
C# implementation: `HostedCoreLaunchTests` fails under `EMUSEN_CRYSTAL_NATIVE=1` with "No core in this build handles
'.iso'" (measured 2026-11-01). Every proof of stage 4 so far ran on the C#. Decided 2026-11-01: the Rust host's
discovery and trust of libretro entries is a stage of its own, after branch `hosted-cores` lands, on that branch's list
reader. The ring's Rust half is built and type-checked but unexercised until then.

## 10i. Stage 4, continued: the console's own picture, the settings window, the cards (2026-11-01)

**The console's own picture (4.4(c)).** Decided 2026-11-01: the libretro host declares a reserved setting,
`console_picture` (a switch, scope `run`, effect `none`, category "Screen filter"), which no libretro core's options can
hold. Mistress sets it at console scope while the console's accurate CRT filter is the screen filter
(`ConsolePicture.Sync`, called wherever the filter is applied) and clears it for every other filter; the host then holds
the core's **named exceptions** over the player's values and gives them back when it is cleared. Mistress learns no
libretro key: it sets `console_picture` wherever the running engine's schema declares it.
- *The exceptions are data*, `emusen-libretro-host/data/console-picture.tsv`, one row per exception: the core's file
  stem, the option's key, the value giving the console's picture, the reason and the date decided. PCEE2's rows are
  `pcsx2_upscale_multiplier=1` (the GS's own internal resolution) and `pcsx2_deinterlace_mode=1` (no deinterlacing), both
  decided 2026-11-01. The project's test core has two rows of its own (`tc_scale=1`, `tc_deinterlace=off`), standing in
  for a console core's in the tests. A row whose key the core does not declare, or whose value is not among its
  choices, is skipped.
- *Held, never written.* `Options::hold` keeps the held values beside the player's; `GET_VARIABLE` answers the held one.
  The player's stored values are never changed, so a change the player makes while a value is held is kept and takes
  effect when the filter is turned off. `GET_VARIABLE_UPDATE` is raised only when the value the core reads changes.
  Made with `console_picture` on, a machine is held from its first frame.
- *Locked rows.* `setting_notes` gives each held row its reason ("Held at <the value's label> while the CRT filter is on: …
  (a named exception, decided 2026-11-01)") and lists the rows a frontend shows locked under the note key `!locked`,
  space-separated, `console_picture` always among them (`EmuSen_CoreAPI.md` §6.13).
- Tests: `the_console_picture_holds_the_named_exceptions_and_gives_the_players_values_back` (`tests/testcore.rs`: the
  player's 640 × 480 blended picture, 320 × 240 unblended with the filter on, the locked list and reasons, a player's
  change while held waiting, the player's picture back; a machine made with it on held from its first frame);
  `Options`' unit test of holding; and through the C# frontend API `The_crt_filter_gives_the_consoles_own_picture_and_turning_it_off_gives_the_players_back`
  (`HostedCoreLaunchTests`: `ConsolePicture.Sync` with the CRT filter, then with CRT (Lottes), the stored values
  unchanged). The test core gained `tc_scale` (1 or 2) and `tc_deinterlace` (off, or blend: each odd row a copy of the
  row above), both read again between frames. Seeded faults, each caught by the host's test: the hold never raising
  `GET_VARIABLE_UPDATE`; the setting read as `on` instead of `true`.

**The settings window from the options schema (4.2).** A hosted core's options reach Mistress's Graphics Settings
window as every core's settings do; three things changed. *Sections:* an unrated option's row takes its category as
its group (`CoreSettingDescriptor.Group`, and `group_of` in the Rust implementation), and the host lists the options
category by category in the order the core first names each (libretro's menus group them so), `console_picture` first
and ungrouped; the window already heads each group. *Locked rows:* `CoreSetting.Locked`, from the notes' `!locked`,
reaches the frontend API's schema as `locked` in both implementations, and the window shows a locked row's note (the
value held and why) in place of its control, so the row cannot be changed and a disabled control's colours, which are
outside LunaP's palette, are never drawn; a reset leaves locked rows. *Restart:* the host's options are all applied
between frames (`restart` false), so no row needed the "next start" words; the native cores already say it in their
help. The window is the existing one, built through LunaP's layouts with its big-picture counterpart and pad
navigation. Tests: `A_hosted_cores_tab_has_its_categories_as_sections_and_its_held_rows_locked` (the test core's rows
from a running C# session with `console_picture` on: five sections in order, three free rows shown, the held rows and
`console_picture` locked with their notes); `WindowFitAuditTests`' new case `GraphicsSettingsHostedPlayStation2` at
1280 × 800 and 1920 × 1200, audited and recorded (`Mistress/Specification/`). Seeded fault: the locked list read under
another key, caught by the window's test. The first recording drew disabled controls and the specification's palette
rule refused their `#FFFFFF33` and `#FFFFFF66`, which is why a locked row shows its note instead.

**The ring lent again at `MACHINE_INFO`, and a crash with a ring lent** (`ApiRingTests`, the RX 6800, the C#
implementation). The test core's `tc_geometry` gained `grow`: maxima declared at the native size and, from frame 3, a
picture twice as large with nothing said, the case of a core whose real resolution is known only once it draws.
*Found:* the frame drawn past the ring is held and read back and `MACHINE_INFO` raised, and the session lends a
640 × 480 ring; but on the next frame the host copied the held frame into the new ring after the newer one, and that
older frame's record replaced the newer one's, so the screen would have drawn the frame before the one played
(measured 2026-11-01: frame 3's ring picture differed from its read-back). Fixed in `Gpu::frame`: a frame put in the
ring ends any hold, so a held frame is shown only while no newer one is. Now every frame of
`A_ring_is_lent_again_at_machine_info_when_the_core_draws_past_its_maxima` matches: two 320 × 240 frames in the first
ring, the held 640 × 480 frame read back, seven in the second ring, the ring's handle changed. *A crash:*
`A_core_that_crashes_with_a_ring_lent_ends_in_words_and_the_device_runs_the_next_ringed_session`: with `tc_fault=segv`
the frame call ends in a refusal in words within its deadline, and the next ringed session on the same device runs its
frames in a ring, so the C# implementation's inability to raise the dead core's `ready` (§33.2) leaves the device
usable.

**PS2 memory cards (4.3), not built in this pass.** PCEE2's profile already gives core info its two cards as
battery files 2 and 3 of medium `ps2.card`, so the card manager's media and `MemoryCards.Place` take them as the
console's cards; but their suffixes are `.mcd1` and `.mcd2` where Sapphire's are `.ps2` and `.2.ps2`, so each engine
keeps its own files, which §4.5 asks until a card formatted and written by one engine is read by the other. Making
them one file per slot is then one change of suffix in `files::PROFILES`. The round trip needs PCEE2 driven into its
firmware's browser (pad input at set frames, the browser's own format of a blank card, then Sapphire's card reader
over the bytes, and the reverse with a card Sapphire's reader formatted), which the libretro host's `run` example
cannot yet script; adding a pad script to it is the next step.

## 10j. Stage 4, item 4.3: the PlayStation 2's cards shared by every engine (2026-11-01)

Decided 2026-11-01: the PlayStation 2 card's format has one owner, `emusen-ps2-card`, and a hosted engine keeps the
same card files as Sapphire, as the standing rule has console-wide state shared by every engine and game unless the
player says otherwise.

**One owner.** The crate already held the format, its reader and its editor, which `emusen-systems`' card module (the
card manager's) uses. It now also holds the facts every engine names a card by: the medium `ps2.card`, the files'
suffixes `.ps2` and `.2.ps2` (`SUFFIXES`) and an erased card's byte, FFh (`ERASED`), beside `CARD_LENGTH`. Sapphire's
card (`sio2/card.rs`: page, block and card sizes, the erased card) and its core info (`CARD_SUFFIXES`, `CARD_MEDIUM`)
take them from it, as do `emusen-systems`' medium table and the libretro host's profiles; no engine keeps a copy.

**The profiles' cards** (`files::PROFILES`). PCEE2's and LRPS2's two cards are battery files 2 and 3 with the crate's
suffixes, medium and blank card, so the frontend's `MemoryCards` gives a hosted engine the same files as Sapphire: under
Shared or Named a drawer card such as `Saves/PlayStation 2/Cards/Card 1.ps2`, under Per game `<game>.ps2` and
`<game>.2.ps2`. Two changes came with it:
- *A card not passed is a new, blank card* (8,650,752 bytes of FFh, what PCSX2 makes for a new card, `EmuSen_MediumFormats.md`
  §6), written over whatever the core's folder last held, as Sapphire's slot holds an erased card when no file is given.
  Before, the core kept the last game's card, so a new game, or a new name under Named, saw another game's card. Under
  the referee's system folder (`EMUSEN_LIBRETRO_SYSTEM_DIR`) a card not passed is left as it is, so a referee run never
  writes over the referee's cards.
- *Firmware is numbered from 16* (`FIRST_FIRMWARE`). Numbered from 1, a core's firmware shared numbers with the clock's
  file and the cards, so a card passed as file 2 was also written into PCEE2's `pcsx2/resources` folder while it loaded
  (found 2026-11-01 when the round trip below passed its cards). Sixteen and up is where Sapphire numbers its BIOS.

**Tests.** `tests/cards.rs` lays the test core under PCEE2's stem with PCEE2's published firmware entries: the battery
list's suffixes and medium are the crate's, firmware and battery numbers are apart, a card left in the folder is replaced
by a blank one, a card passed is the core's slot 1 and an empty file a blank card, no card is written as firmware, and a
card the core changes is reported and read back by the crate's reader with its new save. In WiseMan, `HostedPs2CardTests`
(the C# fixture can now lay the test core under another stem): the hosted engine and Sapphire give the card manager the
same medium, suffixes, labels, drawer path, default mode and listed cards (Sapphire's console memory `ps2.nvm` is its
own; PCEE2 keeps its NVM beside its BIOS); and a session opened through the frontend API puts the shared `Card 1.ps2` in
the hosted core's slot and brings a save written to the core's card back to `Card 1.ps2`, listed by the card manager's
reader. Seeded faults, each caught by `tests/cards.rs`: a card not passed kept; the old suffixes `.mcd1` and `.mcd2`; a
card's change not reported; firmware numbered from 1 (the defect above, before its fix).

**The round trip through PCEE2's firmware** [M, 2026-11-01, the RX 6800, the release libretro host in-process through
the Rust host's `run` example, PCEE2 v2.9.138 with `pcsx2_renderer=vulkan`, `pcsx2_fast_boot=disabled` and the
Americas BIOS 2.30 given by path through `pcsx2_bios`, the system folder a scratch folder under `~/.cache/emusen/` whose
BIOS is a link made from that path, the cards passed and written back through `EMUSEN_RUN_FILES`]:
1. *PCEE2 formats, the crate reads.* The run example's `--exit-elf` program calls the kernel's `Exit`, after which the
   firmware opens its browser on Memory Card (PS2)/1. With a blank card in slot 1, the pad script `400 b`, `600 left`,
   `700 b` (Cross on the card, "Unformatted. Format now?", Yes) formats it: "Formatting completed.", 7,998 KB free.
   The crate opens the card PCEE2 wrote as formatted, 7,998 KB free and no saves, the card manager's capacity.
2. *The crate writes, PCEE2 reads and writes.* A card formatted by the crate with one save written by it
   (`BASLUS-99999EMUSEN`, an `icon.sys` and a 4,000-byte file, 7 KB, 7,990 KB free) is listed by PCEE2's browser with
   7,990 KB free; its information screen gives 7 KB and the time the crate wrote, 12:00 in Japan's zone shown as 07:30
   in the zone chosen at the first start (GMT+4:30). It shows as "Corrupted Data", having no icon files, so the browser
   refuses to copy it; its Delete works (`400 b`, `600 b`, `800 down`, `900 b`, `1050 left`, `1150 b`): "Delete
   completed." The crate then reads the card PCEE2 wrote with no saves and 7,997 KB free, the root directory keeping the
   cluster it grew for the save's entry.
Not done: a save with its icons made by a game on one engine and loaded by the same game on the other, which needs a
game's route on each engine (Sapphire runs games at about 0.4 to 5 frames a second); the run example's pad script and
files are the means when it is wanted. The data disc tried first (an ISO 9660 disc with no `SYSTEM.CNF`) left the
firmware at "Please insert a PlayStation or PlayStation 2 format disc." whatever was pressed or with the tray opened,
which is why the route is the `Exit` program. The firmware's first start asked for the language, time zone and summer
time (Cross on each); its NVM is kept beside the BIOS link in the scratch folder.

**The card manager.** No view changed: a console's media come from its engine's core info (`MemoryCards.MediaFor`), and
with the suffixes and medium now the same, the PlayStation 2 tab lists the same cards whichever engine runs, so the
existing fit audits stand.

## 10k. Hosted cores on the Rust runtime (2026-11-01)

*Replaces the statement of the gap at the end of §10h.* Decided 2026-11-01 by the tester: the Rust host's connection to
the libretro host is designed as Rust would have it, not ported from the C# integration (`LibretroEntries`, the C#
`CoreDiscovery`'s libretro parts), which stays as it is until B10 deletes the C# halves.

**The design.** One module of `emusen-core-host`, `entry`, holds what an installed core's folder must satisfy; discovery,
trust and the runtime compose it, and none of them names a file of the folder, every name being `emusen-core-entry`'s
(`Entry::list`, `broker_path`, `adapter_path`, `core_path`):

| Part | What it does |
|---|---|
| `entry::stale` | rule 1 for an entry: each copy (proxy, adapter) hashed and found in an installation's `cores.manifest` under the name it is built as (`Trust::installation_lists`); otherwise *needs refresh* |
| `entry::check` | before the proxy is loaded: not stale; the sidecar's `libretro` member present and naming the core file the folder is named for; its SHA-256 approved; the file on disk still that build. Each refusal in words, marked where an approval alone would settle it |
| `entry::standing` | `approved`, `needs_approval` or `needs_refresh` (`Standing::NeedsRefresh`, the word the Cores window already reads) |
| `entry::read` | the approval's half (§1.1, "written by loading it once after approval"): the copies checked, rule 3 on the proxy, `Library::load_expecting` with the approved hash (so the proxy seals and compares the core before anything of it runs, §10e), rule 4, and the proxy's sidecar written by the crate's one writer |
| `Discovery` | scans each player's directory's `libretro/` entries after its sidecars: an entry with a proxy sidecar is a `Discovered` with `entry` set; an entry whose core has no sidecar of its build (none, or one naming another build) is listed in `waiting()` with the core's hash. `Discovered::open` routes an entry through `entry::check` and loads the proxy with the sidecar's hash expected |
| `HostedCore`, `with_hosted` | the hosted list (`emusen_systems::libretro_cores::hosted()`), handed in by the runtime so that the crate takes no dependency on the systems table: an entry the list names offers only the systems its line names (`offers`), and `for_system`, `for_extension` and `in_rule_order` give a system's engines in §4.8's order |
| `Runtime::approve` | `core_approve` of an entry's core hash: the approval recorded, each waiting entry of that hash read (`entry::read`), the directories listed again |
| `route::takes` (crystal) | a listed hosted core takes a game only where its installed `.info` lists the file's extension, by `emusen_systems::libretro_extensions::takes` (§4.1a); a core the list names nowhere takes what its info claims |

Rule 3's cores directories now include an entry's folder, `<player>/libretro/<stem>/`, so the proxy copy passes the owner,
mode and architecture checks where it lies. *A changed build is approved again and read again*, but a proxy loaded in
the process under the old hash stays loaded (§7.2: never unloaded; `load_expecting` refuses a second approval), so the
new build is read at the next start of the program. Named as a limit [A].

**Found while building.** A game's creation values were read under the row's console, while a session stores its
settings under its own console (the system's table console, else its id in capitals). For a system with no table
console, which is the libretro test core's, a console setting stored through a session was applied only at run time, and
a core that reads its options at load (`tc_hw`, `tc_rate`) never saw it. `open_game` now reads them under the session's
console. In every other case the two are the same name.

**Proofs, and the runtime each ran on** [M, 2026-11-01]:
- `emusen-core-broker`'s `tests/runtime.rs` (3), on the Rust runtime (`emusen_crystal::Runtime`), brokered through the
  real proxy, adapter, runner and test core:
  - an entry with no sidecar is waiting, not an engine; `approve` writes its sidecar; the game opens on it, runs 20
    frames at 320 × 240, and a state taken and loaded brings the picture back;
  - revoked, rebuilt and stale entries are refused in words before loading, with the standings `needs_approval`,
    `needs_approval` and `needs_refresh`, and the new build waits for its approval;
  - `tc_rate` stored through a session reaches the next machine's creation.
- `emusen-core-host`: §4.8's order (native first; PCEE2 ahead of a native PS2 core; an entry's unlisted system not
  offered), plus the crate's existing 30 tests.
- `emusen-crystal`: `takes` refuses PCEE2 a `.cue` and gives an unlisted core what its info claims; the crate's other tests.
- WiseMan `HostedCoreLaunchTests` under `EMUSEN_CRYSTAL_NATIVE=1`: 4 of 4, the runtime reporting the platform library and
  frontend API 1.4. The same build's platform library from before this change fails the main case with "No core in this
  build handles '.iso'". The fixture still lays the entry and writes its sidecar through the C# host; the test's calls are
  the frontend API's.
- Seeded faults, each caught: the approval unchecked (the refusals test), the copies never stale (the same), the build
  on disk unchecked (the same), the rule's order flattened (the order test), `takes` always yes (crystal's test), the
  creation values under the row's console (the stored-setting test).

**Surfaces.** No version changes. The frontend API, the core ABI and the platform interface are unchanged. The
`cores` listing's `trust` may now say `needs_refresh`, a word it already had. The `emusen-core-host` crate gains public
items (`entry`, `HostedCore`, `Discovered::entry`, `CoreInfo::libretro`, `Standing::NeedsRefresh`); it is not a
versioned C surface. `for_extension` and `for_system` now return the engines in rule order as a vector's iterator.

**Left**: a waiting entry is not yet a row of the `cores` listing, and the listing's `libretro` field (`EmuSen_Cores.md`
§7.3) is not built. Until they are, the Cores window and Mistress's question take the hash to approve from the core
tool's rows. The ring through the Rust runtime is branch `libretro-ring`'s. The C#-shaped classes (`LibretroHostTests`,
`CoreDiscoveryTests`' libretro case) reach the C# host's types and are left for B10.

## 10l. Real cores on the Rust runtime (2026-11-01)

The crate `EmuSen/Cores/Libretro/emusen-libretro-games` proves the Rust runtime (`emusen_crystal::Runtime`, the default
config: the installation's folder as its cores directory, the player's `/Cores` as its player cores) on real libretro
cores, with no frontend. Its library lays out a sandbox player home (an installation folder holding the release proxy,
libretro host and runner with their `cores.manifest`, the tree's data and config folders pointed inside the home) and
runs Cores' own jobs (`core-fetch`, `core-get`, `core-verify`) with the context the platform library gives them. Its
example `games` installs, approves (`Runtime::approve` with the hash `NOTICE` records) and runs: a game opened, N frames
past a boot of B frames timed one by one, the picture's size and digest, a state taken, 60 frames run, the state loaded
and the pictures compared, the core's log on request. Tests: 3 (the timing summary, the `.info` reader, the
installation's manifest); seeded faults, each caught: the frames per second inverted, a key matched by its end, the
libretro host left out of the manifest.

**Installed** [M, 2026-11-01] through `core-fetch` and `core-get` from buildbot.libretro.com (index of 2026-11-01) into
`~/.cache/emusen/probe/libretro-rust/home/`, then approved:
- `pcee2`: PCEE2 v2.9.138 (commit 571962773; its `.info` says v2.7.x), SHA-256
  `ab7ff5fdb580fdf842190be978a8dc6049db707af5177c374cea30e755b7cc68`, index CRC-32 `d589c813`.
- `flycast`: Flycast (`.info` version "Git"), SHA-256 `3fbe535cd52a2e503dec5e7a20a38d046c3ebdd355a0970a7de7e350fee48fcf`,
  index CRC-32 `3572a39c`.

The first `core-get` refused both: the tool compared the whole zip's CRC-32 with the index, whose CRC-32 is the
library's (`EmuSen_Cores.md` §5.2, fixed the same day).

**Timing** [M, 2026-11-01, the desktop, Ryzen 7 7700X with an RX 6800 (`EMUSEN_GPU_DEVICE="RX 6800"`), the release
probe, CPU weight 20 with an 8 GB cap, a CPU-only build of another branch running beside the first rounds; no input, no
ring, the picture read back by the libretro host; each frame run as fast as it goes, unpaced]:

| Game | Core | Frames timed | Frames per second | Median, worst frame | Picture |
|---|---|---|---|---|---|
| Jak and Daxter (USA, Rev 1), the ISO | PCEE2, renderer automatic (Vulkan) | 600 to 2,400 | 391.8, 341.8, 656.0 | 1.50 to 1.94 ms, 2.2 to 31.7 ms | 512 × 448 at 4:3 |
| Crazy Taxi (US), a CDI copied from the household's library | Flycast, defaults | 600 to 2,400 | 603.3, 599.7 | 1.65 to 1.67 ms, 1.9 to 2.4 ms | 640 × 480 at 4:3 |

RetroArch's 60.8 fps measured earlier over the same span is paced at the television's rate (`EmuSen_LibretroBridge.md`
§1 records RetroArch holding 60 frames a second on frames 600 to 2,400); the runtime's figure is unpaced, so both are at
or above full speed, and the comparison says only that the brokered runtime falls short of nothing RetroArch reached. The
spread of Jak's three rounds (the third run after the build beside it ended) is the CPU it shares; PCEE2's own threads
run at the scope's weight. Both spans are light scenes: with the referee's cards unreadable, Jak stops at "Memory card
slot 1 … please insert a memory card" (with PCEE2's blending notice drawn over it), and Crazy Taxi has reached "Press A to
create a new file" by frame 600. A heavier span needs a pad script, as `run.rs` has.

**States** [M]: PCEE2's state is 100,663,408 bytes, Flycast's 37,748,848. Flycast's picture after the load equals the one
after the save, and stays so. PCEE2's equals it when saved at frame 901; saved at 2,401 the frame after the load differs
and goes on changing for the next four, which matches PCEE2's notice fading by wall-clock time rather than a state
fault (not proven).

**Found on the way** (each reported, none fixed here):
1. The runtime never gives a core claiming `IMAGE_PATH` the game's path (`crystal`'s `Engine::create` leaves
   `Create.image_path` unset): Flycast refuses the game ("this core opens the game file itself, and the host gave no path
   to it"); PCEE2's runner exits with code 0, the whole 1.46 GB image having been sent across the socket.
2. With the referee switch `EMUSEN_LIBRETRO_SYSTEM_DIR`, the runner's Landlock policy grants nothing on that folder nor on
   the BIOS it holds by link: PCEE2 is "Unable to auto locate a BIOS image" and the approval's schema lists `pcsx2_bios` as
   `auto` alone, so the runtime refuses the named image.
3. An open succeeds although PCEE2 reported `VMManager::Initialize() failed`: black frames at about 4,000 a second, and a
   state refused ("the core did not write its state").
4. After `cores verify` refreshes an entry's proxy copy, its sidecar keeps the old proxy's hash; the entry is listed as
   approved but routes nothing ("No core in this build handles '.iso'"), and `approve` answers "no core library listed
   has the SHA-256"; only a sidecar removed and approved again recovers it.

The timings above were taken with two local changes for items 1 and 2, never committed: the image's path given at
create where no soft patch altered it, and the proxy's policy reading the system folder and the BIOS file named by path.
They are to be taken again once both land, and through the ring once it runs on the Rust runtime.

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
5. **Decided 2026-11-01: the read-back on demand, with the trait change (§10h).** A ringed frame's picture for screenshots, a state's picture and rewind (§10f, §10g 4.4(e)): the libretro
   host reads it back from the core's device only when `frame_copy` is asked for, which needs `Core::frame` in
   `emusen-native` to take the machine by mutable reference (or the host to keep the picture behind interior
   mutability). *Recommended:* the read-back on demand, with the trait change, since a per-frame read-back would undo the
   ring's saving.
6. **[decision], not decided (2026-11-01); a known item.** `pcsx2_blending_accuracy` stays at the core's default and the notice is drawn as part of the picture until a handheld measurement exists; no named exception is added without one. PCEE2's notice in the picture (§10f): its renderer writes "Blending Accuracy is Basic, High recommended"
   into the frame of Jak and Daxter's title, and its published options offer no switch for notices. *Recommended:* set
   `pcsx2_blending_accuracy` to the level the core recommends as a named exception of the PlayStation 2 profile only if
   a measurement on the handheld shows it fits the frame; otherwise leave the player's choice and accept the notice.
7. **[decision]** Frame advance for hosted cores (§10h): Mistress has none, and the debugger's run-to-frame resumes a
   hosted core's session. *Recommended:* a frontend frame advance in the frontend API (one frame run while paused, the
   session held after it), built for every engine at once, not for the libretro host alone.
