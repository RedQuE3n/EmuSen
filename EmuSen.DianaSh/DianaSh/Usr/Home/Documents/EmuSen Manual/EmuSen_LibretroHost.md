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
the approval, keyed to the libretro core, stands. Since 2026-11-01 the runtime also refreshes an approved entry's copies
itself when it reads the entry, from the installation folder whose manifest lists both libraries (§10v).

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
  `firmware_for`, then cached; the runner exits, which it did not until §10w) and one runner per `create`. It passes a Unix socket pair (control,
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
| read | the core's system folder and the resolved target of each link under it (§10n) | firmware the player linked into the folder |
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

**Left** (*the first two built in §10m*): a waiting entry is not yet a row of the `cores` listing, and the listing's `libretro` field (`EmuSen_Cores.md`
§7.3) is not built. Until they are, the Cores window and Mistress's question take the hash to approve from the core
tool's rows. The ring through the Rust runtime is branch `libretro-ring`'s. The C#-shaped classes (`LibretroHostTests`,
`CoreDiscoveryTests`' libretro case) reach the C# host's types and are left for B10.

## 10m. The cores listing's libretro rows, the path, and the refresh (2026-11-01)

Decided 2026-11-01 by the tester, taking §10k's question: the waiting entry and the listing's `libretro` field are built now,
and the frontends take the hash to approve from the frontend API's Rust answer, not from the core tool.

**The listing** (the frontend API's 1.5, `EmuSen_Backend.md` §34). `listing::cores` gives each row `libretro`: null for
a library that is not an entry, or `{folder, file, sha256, corename, name, version, license, authors, source, build,
trust}`. Its `sha256` is the core file's own, read as the scan reads it, never `NOTICE`'s. Its `trust` is that file's
standing (approved where the hash is approved and is the build the sidecar names). `version` is the core's own, null
until it has been read. The other fields are `NOTICE`'s, through `emusen-core-entry`. An entry waiting to be read
(§10k) that is not already an engine is a last row of its own: `engine` null, `report` "not yet read", `trust` the
entry's standing, `systems` the hosted list's line for it. `Runtime::approve` also accepts the core hash of an entry
already read, so a revoked core can be approved again.

**The frontends** read that answer and nothing else:
- *Cores* (`EmuSen.Tools.Cores`): `CoreRow.OfRuntime` takes a row's approval hash from `libretro.sha256` where it has
  one, and `CoresPane.Joined` lends the core tool's row that hash beside the engine and trust, so *Approve…* names the
  libretro core's hash.
- *Mistress* (`MainWindow.CoreAwaitingApproval`): for the routed engine's row, or where no engine is routed a waiting
  row of the routed system, the question shows `libretro.file` and approves `libretro.sha256`; a row needing refresh
  is not asked about.

**The path, found by running real cores.** `Engine::create` never set `image_path`, so Flycast (`need_fullpath`) refused
every game and PCEE2 was sent a 1.46 GB image through the broker's socket. The path is now given at create and at a
disc's insert (`Machine::media_insert_at`), and only for an unaltered file: `DiscFiles::unaltered_path` withholds a copy
unpacked from an archive, and a package (which may carry soft patches) gives none, as §10d's rule has it.

**The refresh, found the same way.** After `cores verify` refreshed an entry's proxy copy, the sidecar kept the old
copy's hash. The entry was listed approved, but it opened nothing and could not be approved. An entry whose proxy
copy differs from its sidecar's hash is now waiting, like a new build of the core. The runtime reads every waiting entry
whose core hash is approved when it starts and at each rescan, so an approval stands across a refresh, as §1.3 promises.
A refresh must replace the copy by a rename, as the core tool does, since the process may have the old file mapped.

**The player's `/Cores`, followed.** The Rust runtime fixed its player directory when it was made, where the C# reads
`DataStore.Cores` at each scan. Mistress's one shared runtime therefore kept a data folder that had since moved. The
`ControllerBindingsSaturn` and `ControllerBindingsDreamcast` audits showed it: their console comes only from a core the
case places in the player's `/Cores`, and before the change the case reached it only through the C# discovery's static.
A runtime given no directories now reads the tree's `/Cores` at each scan (`player_directories`). The audit places its
core there, approves it through the frontend API and takes it out again with a rescan. Measured 2026-11-01: the 8 cases
pass on both runtimes. `tests/player_cores.rs` lists a library from a data folder moved after the runtime was made.

**A core that cannot start** (§10n's status -18) does not cross the frontend API as a number: it is the libretro host's
own status, and the runtime refuses the frame with the core's words, which are its last error, PCEE2's own line where
§10n's profile names one. So it needs no bump of its own. Tested through the runtime with the test core's
`tc_fail=first_frame`, stored as a console setting.

**Proofs** [M, 2026-11-01]:
- `emusen-core-broker`'s `tests/runtime.rs`, 7 tests on the Rust runtime:
  - the waiting row approved by its `libretro.sha256` becomes one engine row, and is revoked and approved again;
  - the path the test core logs is the game's;
  - a refreshed proxy copy is read again at start and opens a game.
- `emusen-crystal` (with `crystal_abi` at 1.5), `emusen-core-host` and `emusen-galaxia` pass.
- WiseMan `CoresWindowRuntimeTests` on the Rust runtime (2): the window offers the waiting core with its core's hash,
  approves it through the frontend API, and lists it again as an engine; Mistress asks by the core's file and hash, then
  not after the approval.
- `CoresWindowTests` and `HostedCoreLaunchTests` pass on the Rust runtime too.
- On the C#: `CoresWindowTests`, `CrystalHeaderTests` and `CoreHostParityTests` pass.
- Seeded faults, each caught: the path not set (the runtime test); the window's join dropping the hash (the window
  test); Mistress naming the proxy's hash (Mistress's test).

## 10n. Two defects real cores found on the Rust runtime (2026-11-01)

Both were found by the tester running PCEE2 through the Rust runtime with the Americas BIOS linked into a system folder.

**The runner could not read linked firmware.** The proxy's Landlock policy (§3.3) granted the core's private folders
and the image's folder, but neither the system folder named by `EMUSEN_LIBRETRO_SYSTEM_DIR` (§10c) nor the target of a
link under any system folder. Landlock checks the path a link resolves to, so a firmware file linked into a granted
folder is still refused where it lives. PCEE2 logged "Unable to auto locate a BIOS image" and `VMManager::Initialize()`
failed; a local patch adding the folder and the resolved BIOS file to the read list let the BIOS be found and Jak and
Daxter boot [M, the tester]. The fix, in the proxy's `spawn`: the system folder the core is given, which the host and the
proxy now take from one rule (`emusen_core_entry::system_folder`, the player's `.data/<stem>/system/` or the referee's
folder), is granted read-only, and the folder is searched for links (at most 8 levels and 20,000 entries, so that a
folder of dumped textures cannot stall a create). Each link's resolved target is granted read-only. A target that is a
file is granted alone, never its folder; a target that is a folder is searched in turn. The search reads names and link
targets only: no firmware file is opened by the project's code, and the core reads what it reads. The referee's folder is
read-only, so a core that writes there (PCEE2's cache and cards) is refused those writes. A player's system folder lies
inside the core's private folders and stays writable.

**An open succeeded after the core had failed.** Measured on 2026-11-01 with PCEE2 v2.9.138 and an empty system folder
(no firmware present), the host's calls traced in order: `retro_load_game` returns true, and `retro_serialize_size` then
answers 100,663,296. The core starts its machine in the first `retro_run`. There it logs, at info level,
"[PCSX2] VMManager::Initialize() failed." and hands a duplicate frame and 800 audio frames. Every later `retro_run` hands
a black 640 × 480 software frame and no sound. It never calls `SHUTDOWN` and never logs at error level. So libretro
carries no signal of the failure: not the load's return value, not an environment call, not a log level. The only sign
is the core's own line. The narrowest check is therefore a per-core profile (`failure::FATAL_LINES`, by file stem as
measured: PCEE2's line, and the test core's). A profile line logged during load ends the create with
`STATUS_NOT_STARTED` (-18). Logged during a frame, it ends that `advance` with -18, and every later frame is refused the
same way. In either case the detail is "it could not start: " followed by the core's line. LRPS2 has no entry: its line
has not been measured. Independently, a load that returns false now quotes the core's last error or warning ("the core
said: …") in place of the earlier "its log says why". The same run after the change: `advance: (-18) it could not start:
[PCSX2] VMManager::Initialize() failed.` at the first frame [M].

**Tests.** The test core gains `tc_fault=read_system` (it reads `bios/firmware.bin`, a link in its system folder, then
`other.bin` beside the link's target, which nothing links) and `tc_fail=load|first_frame`.
- `emusen-core-broker` `tests/libretro.rs`: the link is read and its neighbour refused by the kernel in the runner; a
  refused load's words carry the core's error, and a first-frame failure crosses the process boundary as -18 with the
  core's line in `last_error`.
- `emusen-libretro-host` `tests/testcore.rs`: the same refusals in-process, the second frame refused as the first, and a
  new machine running afterwards.
- Seeded faults, each caught: the system folder not granted (the link test, 2), the link's folder granted instead of its
  file (the same, 3), links not followed (the same, 2), the frame's profile check removed (the broker's refusal test),
  the failure not kept for later frames and the load's line not quoted (the host's test).

**Surfaces.** The libretro host's status band gains -18, "the core could not start the game; its log says why", which
`status_text` gives. `emusen-core-entry` gains `DATA`, `SYSTEM_DIR_VAR`, `data_folder` and `system_folder`. The core ABI
and the frontend API are unchanged. The runtime's message for a stopped frame names the status's words. The detail
carrying the core's line reaches `last_error`, which the runtime does not yet show.

## 10l. The ring on the Rust runtime (2026-11-01)

Until this pass every proof of the frontend API's ring (§10h) ran on the C# runtime; the Rust half (the runtime's
`Engine::lend_ring` and `take_ring_frame`, the platform library's `SerenityLender` over `emusen_serenity_ring_new`, the
ring's release after a crash, and Serenity's `advance_ring`) had been built and type-checked only. Once the Rust core
host discovered and admitted installed libretro entries (§10k), the ring was run through it end to end.

**What was run.** `tests/api_ring.rs` in `emusen-core-broker` drives the platform library's own exports as Mistress
does: an installation whose `cores.manifest` vouches for the proxy and the libretro host, the test core's entry under
the player's cores, approved by its hash (`emusen_crystal_core_approve`); a session opened by `emusen_crystal_open` with
`OPEN_RING` and a Serenity device; each frame's ring fields read from `emusen_crystal_frame`, the ring borrowed by
`emusen_crystal_picture_ring` and drawn by a chain through `emusen_serenity_chain_advance_ring`. The core runs in the
confined runner, so every picture crosses the process boundary and both semaphores cross by `SCM_RIGHTS`. The four
cases are those of `ApiRingTests`:

1. six frames of the test core's Vulkan picture (`tc_hw=vulkan`, 320 × 240) in slots 0, 1, 2, 0, 1, 2 at values 1 to 6,
   each drawn from the ring byte for byte as the same frame read back by `picture_copy` and uploaded, and equal to that
   picture; one frame advanced and held; a session without the flag lent no ring;
2. twelve ring frames each given back by a one-pixel draw, none stalling the core (each frame bounded at 10 s);
3. `tc_geometry=grow`: frames 0 and 1 in the first ring, frame 2 at 640 × 480 read back while `MACHINE_INFO` raises the
   maxima, frames 3 to 9 in a second, larger ring, each drawn as its read-back picture; this case was listed as not yet
   graded in `EmuSen_Backend.md` §33.4;
4. `tc_fault=segv`: the session ends at the core's second frame in the runtime's words ("EmuSen - Test Core stopped:
   the core's process ended on signal 11"), the ring is withdrawn and freed after the dead machine, and the next ringed
   session on the same device runs four frames.

The chain is one pass of a stock shader the test writes (nearest sampling at the source's size), not the PlayStation 2
CRT filter of `ApiRingTests`, whose preset the C# frontend writes; the comparison against the uploaded picture is the
same. A unit test in Serenity (`ring.rs`) proves the raise itself: a read on the host's device that waits for `ready` at
a value no core signals is still waiting after 200 ms and returns once `Ring::release_after_crash` raises it. In case 4
the core had signalled the last value handed before it died, so the raise there finds `ready` already at that value.

**Found.** The ring's Rust half needed no change. One defect outside it stopped the first run: a game's machine was
created with the console settings stored under the systems table's console, while sessions store and apply them under
the session's console, which for a system with no console of the table (the test core's) is another name; so
`tc_hw=vulkan`, read by a libretro core only at load, never reached the machine, and no frame was in the ring (measured
2026-11-01: `settings_get` answered `vulkan` for the ringed session while frame 0's `picture_in_ring` was 0). It was
fixed in the open path by naming the session's console first (`session_console`, §10k's branch). The platform library's
last error is kept per thread, so a frame called on another thread has its words read on that thread.

**Validation** (`VK_LOADER_LAYERS_ENABLE=*validation`, `VK_KHRONOS_VALIDATION_VALIDATE_SYNC=true`,
`VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`, `EMUSEN_VALIDATE=1`): silent on both devices. The host's device holds
the layer itself, and each case asserts `emusen_serenity_device_findings` empty. The core's device is made in the runner,
which inherits the environment and may read `/usr`, so the layer loads there too and reports on the runner's standard
error; a seeded fault proved it (the slot's transition to `TRANSFER_DST_OPTIMAL` removed: `VUID-vkCmdCopyImage-dstImageLayout-00133`
and 15 further lines in the log), and the clean runs' logs hold no line of it. The test does not assert the runner's
messages, which reach no API; that check is the log's.

**Seeded faults**, each caught: the host's crossing given the wrong old layout (the findings assertion, 7 messages);
`take_ring_frame` answering none (case 1, the record's `picture_in_ring` 0); the ring not lent again at `MACHINE_INFO`
(case 3, every frame after the growth read back); `chain_advance_ring` reading the next slot (case 1, 304,400 bytes
differ); `release_after_crash` raising nothing (the Serenity test, still waiting after 5 s); the core's slot transition
removed (the runner's layer, above).

**Which runtime each ring proof ran on.**

| Proof | Runtime | Where |
|---|---|---|
| in-process ring, copy and blit, refused UUIDs, a picture held for a larger ring | none (the Rust core host and the libretro host in one process) | `emusen-libretro-host/tests/ring.rs`, §10f |
| five frames through the broker | none (the Rust core host and the runner) | `emusen-core-broker/tests/libretro.rs`, §10f |
| the ring through the console's filters keyed on the system | C# (`RingChainTests`) | §10f |
| the frontend API's ring, give-back, `MACHINE_INFO`, crash | C# (`ApiRingTests`) | §10h |
| the frontend API's ring, give-back, `MACHINE_INFO`, crash | Rust (`emusen_crystal_*` and `emusen_serenity_*` of the platform library) | `emusen-core-broker/tests/api_ring.rs`, this section |

Run 2026-11-01 on the RX 6800 under RADV, headless, debug builds; the broker's tests now take the platform library as a
development dependency.

## 10q. Real cores on the Rust runtime (2026-11-01)

The crate `EmuSen/Cores/Libretro/emusen-libretro-games` proves the Rust runtime (`emusen_crystal::Runtime`, the default
config: the installation's folder as its cores directory, the player's `/Cores` as its player cores) on real libretro
cores, with no frontend. Its library lays out a sandbox player home (an installation folder holding the release proxy,
libretro host and runner with their `cores.manifest`, the tree's data and config folders pointed inside the home) and
runs Cores' own jobs (`core-fetch`, `core-get`, `core-verify`) with the context the platform library gives them. Its
example `games` installs, approves (`Runtime::approve` with the hash `NOTICE` records) and runs: a game opened, N frames
past a boot of B frames timed one by one, the picture's size and digest, a state taken, 60 frames run, the state loaded
and the pictures compared, the core's log on request. Its example `ring` goes through the platform library's frontend
API as Mistress does (`emusen_crystal_open` with `OPEN_RING` and Serenity's device, as `tests/api_ring.rs` of the broker
does): each frame's picture drawn from the lent ring by a one-pass chain (nearest sampling at the source's size) and
given back by a one-pixel draw, a picture the core read back uploaded to the chain instead; a pad script by frame
number; PNGs of the chain's picture; a state saved to slot 1, loaded twice and the pictures compared. Tests: 4 (the
timing summary, the `.info` reader, the installation's manifest, the pad script); seeded faults, each caught: the frames
per second inverted, a key matched by its end, the libretro host left out of the manifest, a press released a frame
early, the script left unsorted.

**Installed** [M, 2026-11-01] through `core-fetch` and `core-get` from buildbot.libretro.com (index of 2026-11-01) into
`~/.cache/emusen/probe/libretro-rust/home/`, then approved:
- `pcee2`: PCEE2 v2.9.138 (commit 571962773; its `.info` says v2.7.x), SHA-256
  `ab7ff5fdb580fdf842190be978a8dc6049db707af5177c374cea30e755b7cc68`, index CRC-32 `d589c813`.
- `flycast`: Flycast (`.info` version "Git"), SHA-256 `3fbe535cd52a2e503dec5e7a20a38d046c3ebdd355a0970a7de7e350fee48fcf`,
  index CRC-32 `3572a39c`.

The first `core-get` refused both: the tool compared the whole zip's CRC-32 with the index, whose CRC-32 is the
library's (`EmuSen_Cores.md` §5.2, fixed the same day).

**Through the ring, with the fixes in** [M, 2026-11-01, libretro-games at 189bb9267 with libretro-rust 4a27c40bb (the
image's path at create, 3ac0b9178; an entry refreshed by `cores verify` waiting for approval again, 4a27c40bb),
libretro-e be1f6bf22 (the system folder and its links' targets granted, a core's failed start refusing, §10l) and
libretro-ring 7ad12d730 (the ring on the Rust runtime, §10l) merged, no local change; the desktop, Ryzen 7 7700X with
an RX 6800 (`EMUSEN_GPU_DEVICE="RX 6800"`), the release probe, CPU weight 20 with an 8 GB cap, a debug test run of
another branch beside it; each frame run as fast as it goes, unpaced]:
- **Setup.** After the installation's proxy and host were rebuilt, `core-verify` refreshed both entries' copies and the
  runtime listed them as waiting, and one approval each put them back. PCEE2 is given no referee folder: its system
  folder is the player's own, `.data/pcee2/system/`, holding a link `pcsx2/bios/ps2-0230a-20080220.bin` made from the
  BIOS's path and nothing else; the approval's schema then lists that name as a `pcsx2_bios` choice, and it is set
  through `settings_set`. The log says "BIOS Found: USA v02.30(20/02/2008)". Its cards are the host's blank cards in
  that same folder, so Jak asks to format card 1 and the script answers No.
- **Pad scripts** (port 0, RetroPad numbering: 0 B, 3 Start, 13 R2). Jak: Start at 2,500, then B at 2,700, 2,850 and
  every 150 frames to 3,600 (New Game; card 1 not formatted), and B held 10 frames every 120 from 25,000. The intro,
  drawn by the game's engine, runs from about 4,800 to 24,500; Jak stands on Geyser Rock in play by 25,000. Crazy Taxi:
  B at 650, 800, 950 and 1,100 (a new file on the VMU), Start at 1,300 and 1,500, B at 1,700 to 2,500 every 200 (Arcade,
  3 minutes, Axel), R2 held from 2,600 for 2,400 frames: the taxi drives from about 2,600.

| Game | Core and picture path | Frames timed | Frames per second | Median, worst frame | Picture |
|---|---|---|---|---|---|
| Jak and Daxter (USA, Rev 1), the ISO | PCEE2 v2.9.138, Vulkan, every frame in the ring (26,798 and 26,799 of 26,800) | 25,000 to 26,800, in play | 180.24, 180.00 | 5.47 ms, 8.0 and 8.2 ms | 512 × 448, two fields, 4:3 |
| Crazy Taxi (US), a CDI copied from the household's library | Flycast, Vulkan, every frame read back and uploaded (none in the ring) | 3,000 to 4,800, driving | 378.46, 369.62 | 2.60 and 2.69 ms, 3.7 and 4.0 ms | 640 × 480, 4:3 |

Both are about three and six times the television's rate. RetroArch's 60.8 fps measured earlier is paced at the
television's rate (`EmuSen_LibretroBridge.md` §1 records RetroArch holding 60 frames a second on frames 600 to 2,400);
these figures are unpaced, so they say that the brokered runtime with the ring falls short of nothing RetroArch reached,
not that it is three times faster. The two rounds of each agree within 2.4%.

**States** [M]: PCEE2's load works: two loads of the same slot give the same first picture, and the frames after it are
the saved scene's (the intro's background when saved at 3,100), though that first picture is the same in every run
whatever the save point, so it is a picture the core gives once after a load. Flycast's load returns success but does
not restore the picture: saved while driving at frame 3,100, the picture five frames after the load shows the game
timer a second past the moment of the save, and two loads of the slot give different pictures; reported, its cause not
found (Flycast logs nothing at the load).

**Found on this pass** (reported):
1. Flycast never puts a picture in the ring. The host logs "the card lacks external memory, semaphores or timeline
   semaphores: pictures are read back" on the RX 6800, which gives PCEE2 the ring; Flycast makes its own Vulkan device
   (its log lists only `VK_KHR_swapchain`, `VK_EXT_provoking_vertex` and `VK_KHR_buffer_device_address` enabled), so the
   extensions the host needs appear not to reach it. Its timing above is therefore the read-back path's.
2. Flycast's state load, above.

**The first pass, superseded** [M, 2026-11-01, no ring, no input, the picture read back, with two local changes standing
in for the image-path and system-folder fixes]: Jak 391.8, 341.8 and 656.0 fps and Crazy Taxi 603.3 and 599.7 fps over
frames 600 to 2,400. Both spans were static menus (Jak's "please insert a memory card", the referee's cards being
unreadable; Crazy Taxi's "Press A to create a new file"), and the spread of Jak's rounds followed the CPU it shared.
That pass found the four defects since fixed: the image's path never given to a core claiming `IMAGE_PATH`; the
referee's system folder and its linked BIOS not granted by the runner's Landlock policy; an open succeeding after
PCEE2's `VMManager::Initialize() failed`; and an entry whose proxy copy `cores verify` refreshed listed as approved but
routing nothing. The core tool's CRC-32 comparison (`EmuSen_Cores.md` §5.2) was fixed on this branch.

## 10o. A core that negotiates version 1 reaches the ring (2026-11-01)

**The defect.** Flycast put no picture in the ring on the Rust runtime: every frame was read back, and the host
logged that the card lacked the external extensions, on the RX 6800 where PCEE2 is ringed on every frame. The host's
log, with the negotiation interface's members added to it, showed Flycast declaring negotiation version 1 with
`create_device` alone (no `create_instance`, no `create_device2`), although the host answers
`GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT` with 2. Two faults of the host's v1 path followed (measured
2026-11-01, Crazy Taxi, 120 frames, the run example):

1. Whether the ring was possible was decided only inside the host's device wrapper, which version 2 calls and version 1
   never does; a v1 core's device was therefore always read back, whatever it enabled.
2. Version 1 gives the host no way to turn on a feature beyond Vulkan 1.0's: its `required_features` is a
   `VkPhysicalDeviceFeatures`, while timeline semaphores need `timelineSemaphore` in a features chain. The required
   extension list reached the core, but the feature could not.

**The form.** `libretro_vulkan.h` (negotiation version 2) has the frontend hand `create_device` the
`get_instance_proc_addr` the core makes its device through. The host now hands its own: it answers `vkCreateDevice`
with the same wrapper version 2 uses (the three extensions added where the card offers them, `timelineSemaphore` set in
the core's chain or put in front of it, nothing the core asked for changed) and passes every other name to the loader.
The wrapper records each device it makes and whether it was given what the ring needs; the ring is offered only for the
device the core hands back in its context. A core that makes its device without the function it was handed (through
its own loader) gets no ring and is read back, and the host says so: "the core made its device without the host's
vkCreateDevice, so the external memory and timeline semaphores the ring needs were not enabled: pictures are read back".
A private device a v1 core makes through the same function is given the extensions too, as version 2's wrapper would
give them; the core's allocation callbacks are not passed on, as with the wrapper.

**Result.** Flycast is ringed: "ring: 853x853 lent", and frame 299 of Crazy Taxi read from the ring is its title screen.
Version 2 cores are unchanged.

**Validation.** The test core's v1 paths are silent. Flycast is not, on either path, and none of its messages is a call
of the host's: its image handed by `set_image` lacks `TRANSFER_SRC` usage, which `libretro_vulkan.h` requires ("at
least VK_IMAGE_USAGE_TRANSFER_SRC_BIT and VK_IMAGE_USAGE_SAMPLED_BIT"), so the host's copy out of it and its
transitions are reported (`VUID-vkCmdCopyImage-aspect-06662`, `VUID-VkImageMemoryBarrier-oldLayout-01212`; on the
read-back path `VUID-vkCmdCopyImageToBuffer-srcImage-00186`); it enables `VK_KHR_swapchain` on a device of an
instance without `VK_KHR_surface` (`VUID-vkCreateDevice-ppEnabledExtensionNames-01387`), though no surface is given;
and on the ring path its depth attachment is written by consecutive frames' render passes with no dependency between
them (`SYNC-HAZARD-WRITE-AFTER-WRITE`), which the read-back path's wait at each frame's end hides. These are named here
as the core's, not answered by the host.

**The test core** gained `tc_hw=vulkan_v1` (negotiation version 1, its device made through the function it is
handed, with the extensions it is asked for) and `tc_hw=vulkan_v1_loader` (the same through the loader's
`vkCreateDevice`). Tests (`tests/ring.rs`, the RX 6800, validation on and silent):
`a_core_negotiating_v1_reaches_the_ring_through_the_hosts_vk_create_device`, six frames in the ring equal to the
software pictures; `a_v1_core_that_bypasses_the_hosts_vk_create_device_is_read_back_and_says_why`, the ring refused
`NOT_SUPPORTED`, three frames read back equal, and the words in the log. Seeded faults, each caught: the hook not
answering `vkCreateDevice` and the wrapper not recording its device (the v1 test, the ring refused); the words left
unsaid (the loader test).

**A 16-bit picture of a Vulkan core.** With every pixel format offered (the run example's `0b111`), Flycast's run
panicked in `convert_row`. A hardware frame read back (or a ring frame taken by `frame_copy`) is 8-bit XRGB, and it
was converted into the picture of the format handed for the format the core declared, RGB565 here; `convert_row` had
no XRGB8888-to-RGB565 pair and wrote a 640-pixel row as four bytes a pixel into a 1,706-byte row. Decided 2026-11-01: a
host never crashes on a core's choice of format. `convert_row` now takes every pair: the two 32-bit copies and the
RGB565 copy stay as they were, and every other pair goes through 8-bit red, green and blue (RGB565 from 8-bit keeps the
top bits; a 16-bit format into a 32-bit one repeats them, as before). A 16-bit picture of a Vulkan core loses the low
bits of its colours, as the format the core declared and the host accepted implies. Tests: a unit test of each libretro
format into each handed format, and `a_vulkan_cores_frames_land_in_whatever_format_it_declared_and_the_host_accepts`
(the test core declaring each of the three formats with `tc_hw=vulkan`, the host offering each of four masks, read back
and ringed: 24 runs, every frame equal to the software picture in the handed format). Seeded fault: the old
XRGB8888-to-anything arm put back, caught at RGB565 (the picture differs). The runtime offers RGBA only, so no player
had met it.
## 10p. Flycast's state load, and the first picture after a load (2026-11-01)

§10q reported that Flycast's slot load returned success without restoring the game. Measured on 2026-11-01 with
the release libretro host in-process through its `run` example (`EMUSEN_RUN_STATE=3100,3160,3225`: a state saved after
frame 3,100 and loaded after frames 3,160 and 3,225). The run used Crazy Taxi (US) on the §10m pad script, Flycast
`7adbf3c` on the RX 6800 with no BIOS (its HLE BIOS), at CPU weight 20. The host's `retro_serialize` and
`retro_unserialize` calls were traced.

**The bytes.** `retro_serialize` wrote 37,748,736 bytes, and `retro_unserialize` was given 37,748,736 bytes with the
same SHA-256, at both loads. `retro_serialize_size` gave the same length at the load as at the save. The header's own
digest (§4.6) already refuses a payload that differs. So neither the length, nor the offset, nor the header's wrapping
is the cause. The state was serialized again at once after each load: it differed from the payload in 34 of
37,748,736 bytes, and from the state before the load in 17,951,331. The core applies the state.

**The pictures.** With Flycast's default, `reicast_threaded_rendering=enabled`, the second to fifth pictures after each
load equal the second to fifth after the save. The first does not: it differed from the save's and from the other
load's (`c225c7b0…`, `2b28c8c2…` against `3f7bd061…`), as the two loads of §10m did. With `disabled`, every picture after
each of three loads equals the picture after the save, the first included. The core's option text says the threaded
mode "runs the GPU and CPU on different threads". The measurement fits a picture presented one `retro_run` after its
frame: the frame in flight at the load belongs to the abandoned timeline, and the state does not hold it. Three things
do not change the result: the size check, a second serialize before each load (the host's rollback copy), and
presence sent again after a load with the same devices. Presence sent with *changed* devices (ports 1 to 3 going from
the core's default to none) does change it: the game's pictures diverge from the third frame after the load.

**What §10m saw.** The §10m digest compared only the first picture after each load, which is the stale one. Its PNG after
the load was drawn from the ring example's chain. For a picture that is read back (all of Flycast's, §10m item 1), the
example's steps after the run never uploaded to that chain, so the PNG showed an older picture than the loaded one. The
"timer a second past the save" was not reproduced in-process; the steps now upload. PCEE2's first picture after a load,
the same whatever the save point (§10m), cannot be a stale picture kept by the host: a stale picture would differ with
the save point. It is a picture the core makes after a load, and the host passes it on as given.

**The fix: an exact default.** Threaded rendering is a shortcut that changes what the player gets. Under the project's
rule, such shortcuts are player options and are not on by default. The host's new data file
`data/exact-defaults.tsv` (stem, key, value, reason, date decided, as `console-picture.tsv`) names
`flycast reicast_threaded_rendering disabled`. The host declares that value as the option's default in place of the
core's, wherever the core offers it. The player may still turn the option on; this costs the first picture after a
load and is named in the core's losses. Cost, one unpaced round each over frames 0 to 3,240: 9.74 to 10.02 s threaded
(three runs) against 11.45 s unthreaded, about 15% more time; both are several times the television's rate.

**Also fixed.** A read-back hardware frame is XRGB8888. Converted into a picture of 16 bits (a host taking RGB565, as
the `run` example does), it overran each row and stopped the host. It is now converted to RGB565.

**Tests.** `options::an_exact_default_replaces_the_cores_and_a_value_it_lacks_does_not`,
`exact::the_data_names_flycasts_threaded_rendering`, the RGB565 case in
`video::sixteen_bit_formats_convert_to_full_range`, and `tests/testcore.rs`
`a_picture_shown_a_frame_late_is_stale_once_after_a_load_and_exact_without_the_lag`. The test core's new `tc_lag`
shows each frame the previous frame's picture, held outside its state. With it on, the first picture after a load is
stale and the second is exact; with it off, both are exact. Seeded faults, each caught: the exact default not applied
and a value the core lacks accepted (the options test), the RGB565 arm removed (the video test), and `tc_lag` ignored
(the lag test). One gap: no test drives `ready()`'s use of the data file, because only Flycast has a row.

**Surfaces.** No version changes. The schema's default for Flycast's option is now `disabled`.

## 10r. A game's firmware question froze the big screen (2026-11-01)

**Reported.** On the handheld, docked at 3840 × 2160 in the big-screen form, with PCEE2 approved, choosing Jak and
Daxter froze the screen before the launch screen appeared; 26 s later the session was ended from Steam. Nothing was
logged: no game log was begun (it is begun after the open) and no runtime refused anything.

**Found** [M, 2026-11-01, the desktop, the real PCEE2 v2.9.138 approved in a WiseMan home, the Rust runtime, each call
the big-screen launch makes before its launch screen timed alone]: `route` 0.003 s, `cores` under 0.001 s, the PS2 cards'
listing 0.001 s, the console's schema 0.001 s, and `firmware_needed` **6.663 s**, answering `[]`. Mistress asks for the
game's missing firmware on its interface thread before the launch screen (`PromptForMissingFirmwareAsync`), so for that
long nothing is drawn. The runtime's `firmware::needed` maps the disc and hands its bytes to the engine's
`firmware_for`; for a hosted core that export is the broker's proxy, which copied every byte of the 1.4 GB image into
its request, reading the whole file through, and sent it to the runner, whose libretro host answers from the `.info`
file and never looks at the image. On the handheld the image is read cold from its card and copied once more, which
is the 26 s and most of the session's 2.4 GB peak; the call failed besides, hence the empty answer.

**Fixed.** A proxy target says whether its core's firmware answer reads the image (`Target::firmware_reads_image`,
true unless a target says otherwise); the libretro proxy says no, and its firmware request carries no image. After the
fix the same call took 0.006 and 0.007 s over two rounds and answers what the `.info` declares: PCEE2's `pcsx2/bios` and
`pcsx2/resources` folders, both required. A unit test in `emusen-core-broker` holds the request for a target that does
not read the image to the empty one whatever image is given.

**The interface thread.** The three questions the launch asks the runtime first (the core's approval, the firmware it
lacks, the cards' import) are now asked on a worker (`MainWindow.OffInterface`): an answer that comes within 100 ms is
taken at once, as before, so a launch still starts within the press; a slower one is awaited with the interface free,
so no core's answer can freeze the screen again. `PreLaunchQueryTests` holds the firmware question for as long as it
likes and proves the interface thread runs its jobs meanwhile and that each question ran off it.

## 10s. A hosted core's firmware: the player's console firmware, and the data a core carries (2026-11-01)

Approved by the tester on 2026-11-01. Until this pass nothing gave a hosted core the player's own firmware: §10c ran
PCEE2 on a referee folder named by `EMUSEN_LIBRETRO_SYSTEM_DIR`, and §10q on a link made by hand. Once §10r let the
firmware question reach the core, PCEE2's `.info` answered two required entries, both folders, `pcsx2/bios` and
`pcsx2/resources`, and the runtime refused to open its games for firmware the library did not hold.

**Which is which.** A libretro `.info` names firmware by a path in the core's system folder, and some of those paths
are not the player's to supply. `emusen-core-entry` owns the table (`FIRMWARE_ROLES`, by the core's file stem):
- **Carried**: data the core holds in itself, never asked for. PCEE2's `pcsx2/resources` is PCSX2's own data (shaders,
  fonts, the game index), not firmware; its binary holds the game index (Jak and Daxter's serial `SCUS-97124` is in it
  seven times, read 2026-11-01), and §10q ran Jak and Daxter in play with that folder empty. The libretro host leaves a
  carried entry out of every firmware answer.
- **Console firmware of a system**: a folder for the console's own firmware. PCEE2's `pcsx2/bios` is the PlayStation
  2's (`ps2`). What fills it is what the console's own engines declare for that system, today Sapphire's three BIOS
  files by their Redump names, sizes and MD5s; no list of BIOS files is kept a second time for the hosted core.
- Every other entry is a file the player supplies by its name, as before.

**At open.** Before a hosted core's machine is made, the runtime links each declared file it finds whole in the
firmware folders (`FirmwareLibrary`'s search: by name and size, else by content) into the folder, under its declared
name, as a symbolic link; nothing is copied, and the proxy grants the runner each link's target read-only (§10l). A
file already there is left as it is, and a link to an older place is replaced. Only a BIOS reaches the folder this way.

**The firmware question.** A folder of console firmware counts as held when the folder already has a file (a referee's
folder, or a link made before) or when the store holds any declared file. When it holds none, the player is asked for
one file they can give: the first declared name and size, found by the content of any market's dump, under the
`.info`'s own words for the folder, so the file picker never asks for a folder or for data the core carries.

**Tests** (`HostedFirmwareRuntimeTests`, on the Rust runtime with the test core laid out under PCEE2's stem and its
`.info` declaring both folders): with no BIOS in the store the answer is one required file, Sapphire's first declared
name at 4 MiB, under the folder's words, and nothing of `pcsx2/resources`; with a file of the Americas name and size in
the store (zeros, no BIOS) the answer is empty, and opening the game leaves exactly one link in the folder, to that file.
The firmware itself was never listed, opened or hashed by the project's tools in this work.

## 10t. The console's default waits for approval: asked first, not passed over (2026-11-01)

**The defect.** On 2026-11-01 the tester ran Jak and Daxter in the big screen on a fresh home with PCEE2 installed but
not approved. A on the game opened it on Sapphire at 16–21 fps, and nothing asked for PCEE2's approval. The rule decided
in `EmuSen_Cores.md` §4.8 and `man cores` makes pcee2 the PS2's default while Sapphire is under development, so the
player should have been asked for the default first.

**The cause.** An entry whose core waits for its approval has not been read. It is a `Waiting` row of discovery, not a
`Discovered` engine, so `Discovery::for_system("ps2")` offers only Sapphire, and the route's engine is Sapphire.
Mistress's `CoreAwaitingApproval` asked only when the routed engine itself waited, or when no engine was routed at all.
Mistress could see that PCEE2 was the hosted default and that it waited. What it could not see was whether Sapphire was
the player's choice or only the fallback. `settings_get` answers the schema's default when nothing is stored
(`Setting("PlayStation 2", "Engine")` read "Sapphire" on a fresh config, measured 2026-11-01).

**The design, decided 2026-11-01 by the tester: the frontend API's 1.6.** `emusen_crystal_route` gains
`"waiting_default"`: null, or `{"stem","sha256"}` (`route::waiting_default`). It is set when all of these hold:
1. The routed system's default libretro core (`libretro_cores::default_for`) is an installed entry waiting for approval.
2. Its hash is not approved.
3. The core takes the file by its installed `.info`'s extensions.
4. Neither the game's nor the console's settings chose an engine.

The fact is the runtime's, one owner. Mistress then asks for that hash first:
- **Approve** records it. The runtime reads the entry, and the route then gives PCEE2.
- **Not Now** opens the game on the route's engine, with one line on the status and over the game: "Sony - PlayStation
  2 (PCEE2) waits for your approval, so this game opens on Sapphire." The same line goes to the session's console as
  `[core] …`.

The C# half's `RouteJson` writes the key as null, so the route parity holds. The C# half stays at the minor it
implements (`EmuSen_Backend.md` §37).

**Tests.**
- `route::waiting_default_tests`, written first and failing: named where no layer chose; none when chosen, when
  approved, for a `.cue` PCEE2's `.info` does not list, or for a system with no hosted default.
- `CoresWindowRuntimeTests.The_consoles_waiting_default_is_asked_about_ahead_of_the_native_fallback`, on the Rust
  runtime with a synthetic PS2 disc: the route names Sapphire and the waiting default, Mistress's question carries
  PCEE2's file and hash with Sapphire as the fallback, and a console engine stored takes the key away.
- `PreLaunchQueryTests.Not_now_on_a_waiting_default_says_the_game_opens_on_the_fallback`.

**Seeded faults (2026-11-01).** Each was reverted, and each was caught:

| Fault | Caught by |
|---|---|
| The route ignoring the player's choice | the Rust test and the runtime case |
| The route ignoring an approval | the Rust test |
| Mistress ignoring the key | the runtime case |
| Not Now saying nothing | the pre-launch case |

**In the real window (2026-11-01).** The build was out/ at 3351e722d, with this change's assemblies and the `dist`
platform library over it (frontend API 1.6), on a fresh home with PCEE2 installed and not approved.
- **Not Now**: A on Jak and Daxter presented the question with Approve focused, Right moved the focus to Not Now, and A
  opened the game on Sapphire. The session logged `[core] Sony - PlayStation 2 (PCEE2) waits for your approval, so this
  game opens on Sapphire.` and `[picture] Sapphire`, and nothing was approved.
- **Approve**, on the same home: the question again, then A. The session logged `[picture] Sony - PlayStation 2
  (PCEE2): read back`, and `cores.approved` was written.

## 10u. A library of links: the image's resolved target granted (2026-11-01)

**The defect.** The tester's real-window sandbox of 2026-11-01 kept its library as a folder of symbolic links to discs
stored elsewhere. With PCEE2 approved, Jak and Daxter would not open: "Sony - PlayStation 2 (PCEE2) refused the game:
…/Roms/ps2/Jak and Daxter … (Rev 1).iso: Permission denied (os error 13)" (Mistress's error log, 17:57:24). Sapphire
opened the same link.

**The cause.** The proxy's Landlock policy (§3.3) granted the image's folder as the path names it, and Landlock checks
the path a link resolves to (§10n). So a disc linked into the library is refused where it lives. A library folder that
is itself a link was already reached, since the rule is made on the folder it resolves to. Only a linked file was
missed.

**The fix**, in the proxy's `spawn` (`image_reads`). The image's folder is granted as before. Where the image resolves
outside that folder, its resolved file is granted read-only as well, and nothing beside it. An image that names files
beside it (a cue's tracks, a playlist's discs: `cue`, `gdi`, `m3u`, `ccd`, `toc`) is given its resolved folder instead,
since its tracks lie beside its target.

Left as it is: a disc inserted after the machine starts (`media` insert) reaches a runner whose policy is fixed. A linked
second disc that resolves outside the first's grants is refused there. Not yet met in a game.

**Tests.** `image_tests::a_linked_image_grants_its_resolved_file_and_a_linked_cue_its_resolved_folder`, in the proxy,
written first and failing: a linked ISO is granted its file and not the folder of discs it sits in; a linked cue is
granted its resolved folder; a plain file is granted its library folder only.

**Seeded faults (2026-11-01).** Each was reverted, and each was caught by that test:
- granting the resolved file's folder (wider);
- granting nothing more.

**In the real window (2026-11-01).** The tester's library of links (`~/.cache/emusen/realpath/Roms`, Jak and Daxter a
link to a disc elsewhere) was used on a fresh home, with this proxy in the installation and the entry. A on the game
asked for PCEE2's approval (§10t), and Approve opened it on PCEE2: `[picture] Sony - PlayStation 2 (PCEE2): read back`,
with no refusal in the error log.

## 10v. An approved default with an earlier build's copies opened nothing, and the game fell back unsaid (2026-11-01)

**The defect.** On 2026-11-01 the tester ran the published build f1592d39e in the real window. PCEE2 had been installed
at 17:18 by an earlier build, and the build was published again at 21:27. In the big screen the tester went to Cores,
opened PCEE2 and chose Approve. `cores.approved` was written with the core's SHA-256, but the detail pane still read
"needs refresh: Verify". Back in the library, A on Jak and Daxter opened the game on Sapphire, and nothing was said
(`[picture] Sapphire: read back …` in the session's log).

**The cause, shown on the published library.** The entry's proxy copy hashed `630c5b2e…`, while the installation's
`cores.manifest` lists `libemusen_libretro_proxy.so` as `9c31c402…`. Rule 1 (§1.3) therefore lists the entry as needing
refresh. A probe loaded the published `libemusen_platform.so` through its C interface (`emusen_galaxia_init`,
`emusen_crystal_new` with the sandbox's installation and a copy of its `/Cores`, `emusen_crystal_route`,
`emusen_crystal_core_approve`) and routed the Jak image. It gave these answers:

| Step | `engine` | `waiting_default` | The PCEE2 row |
|---|---|---|---|
| Before the approval | Sapphire | `{pcee2, ab7ff5fd…}` | needs_refresh |
| After it (status 0) | Sapphire | null | needs_refresh, its core approved |
| After it, the copies replaced with the installation's | Sony - PlayStation 2 (PCEE2) | null | approved |

The approval was recorded, and the rescan tried to read the entry. `entry::read` refused it as stale, and the refusal
was said only on session 0, which nothing showed. The entry stayed a `Waiting` row and was never an engine. The route's
`waiting_default` named only an unapproved default, so with the hash approved the route gave Sapphire with no reason,
and Mistress had nothing to ask or say. Four of the five candidate causes were ruled out by the same run: the approval
was in the file the runtime reads (`approvals_path`), the trust read it again (the row's core read "approved"), the
default column reached the route (the waiting default was named before the approval), and the rule order puts a read
PCEE2 first (the third row).

**The change.**
1. **The refresh, owned once.** `Entry::refresh` in `emusen-core-entry` copies the project's proxy and libretro host from
   an installation folder by a rename, as the core tool did. `cores verify`, `get`, `update` and `rollback` call it in
   place of their own copy.
2. **The runtime refreshes an approved entry.** `entry::read`, after checking the approval, refreshes stale copies from
   `Trust::vouched_installation`, the installation folder whose own manifest lists the build of both libraries there.
   It then checks rule 1 again on the copies. Nothing is trusted that was not already trusted: the copies are the
   installation's own bytes, and the core is still held to its approved hash. Each rescan also refreshes an approved
   entry that was read but whose copies went stale, and a runtime starts with a rescan when it finds one.
3. **A held default is said.** An approved entry that still cannot be read is kept, with its words, until the next rescan
   (`Runtime::held`). The route then names it as `"default_held": {"stem","sha256","reason"}`, the frontend API's 1.7
   (`EmuSen_Backend.md` §39). Mistress asks nothing about it. The game opens on the fallback with one line on the status
   and over the game, which also goes to the session's console as `[core] …`: "Sony - PlayStation 2 (PCEE2) could not be
   used, so this game opens on Sapphire: <reason>".

**Tests.**
- `route::waiting_default_tests::an_approved_default_that_could_not_be_read_is_named_with_why`: the held default is
  named with its words, or with a reason where none was kept. It is not named when unapproved, when an engine was
  chosen, or for a `.cue`.
- `emusen-core-broker`'s `tests/runtime.rs`, on the Rust runtime:
  - an entry whose proxy is an earlier build's is refreshed when its core is approved, and then opens and runs a frame;
  - an entry whose folder cannot be written is held with the words "needs refresh … refreshing it failed", and is read
    at the next rescan once the folder can be written.
- WiseMan `CoresWindowRuntimeTests`, on the Rust runtime with a synthetic PS2 disc:
  - `The_default_approved_mid_session_over_an_earlier_builds_copies_is_the_routes_engine`: Sapphire before, PCEE2's
    engine after the approval, both keys null, the copy refreshed, nothing asked;
  - `An_approved_default_that_cannot_be_read_gives_its_reason_and_no_silent_fallback`.
- `PreLaunchQueryTests.A_held_default_is_said_with_its_reason_and_not_asked_about`.

- `emusen-core-host`'s `trust::tests::copies_are_refreshed_only_from_an_installation_whose_manifest_lists_both`.

The WiseMan case of the tester's run was first run against the read as it was before the change (a stale entry refused
and not refreshed). It failed there, with Sapphire as the route's engine after the approval.

**Seeded faults (2026-11-01).** Each was reverted, and each was caught:

| Fault | Caught by |
|---|---|
| The read refusing stale copies without refreshing them, as before | the WiseMan mid-session case and the held case |
| The refresh taking an installation's libraries without its manifest | the trust test |
| The route naming the held default only when unapproved | the route test |
| A held reason kept past the next rescan | the runtime test of the held entry |
| Mistress ignoring `Held` | the pre-launch case |

**In the real window (2026-11-01).** This branch's Release publish was placed in a copy of the tester's sandbox. The copy
kept the sandbox's `home/`, with PCEE2's copies from the 17:18 build, and the hidden `.dianashroot`. Its
`cores.approved` was moved aside first, so nothing was approved. The window was driven by a virtual pad.
- **The tester's path.** Home card, Tools, Cores, then PCEE2, whose row read "needs refresh: Verify". Its page's Approve,
  then Approve on the question. `cores.approved` was written with `ab7ff5fd…`, the proxy copy became the installation's
  (`2c8d78d5…`), the sidecar was written, and the page read "Runs: PlayStation 2 · installed". Back in the library, A on
  Jak and Daxter opened PCEE2 with no question: `[picture] Sony - PlayStation 2 (PCEE2): in a ring …`. Rewind was
  withheld in the pad menu: "Rewind, a snapshot of Sony - PlayStation 2 (PCEE2) takes 398 ms, longer than a frame".
  Of 132 one-second windows, 119 fell inside 59.9–60.1 fps. All 13 outliers were between 59.5 and 60.3, apart from the
  first window (54.7) and two during the BIOS (56.2, 58.3).
- **An update after the approval.** The 17:18 copies were put back, while the approval and the sidecar were kept. At its
  start Mistress refreshed the copies, and A on Jak opened PCEE2 again. `[picture] … in a ring` came 1.2 s after the A
  press, and the first fps line 2.0 s after it. The longest UI-thread answer to an AT-SPI probe every 0.25 s was 0.66 s,
  at the press. One dip, to 42.7 and 45.5 fps, came 38–40 s in.
- No runner outlived Mistress after SIGTERM in either run.

**Where that session's log went.** The session logged to `~/.config/EmuSen/Logs/PlayStation 2/gui_…`, not under the
sandbox's `home/`. This is by design. The sandbox's `appsettings.json` names no `LogDirectory`, and the error log's
default is `~/.config/EmuSen/Logs` (`ErrorLog.DefaultRoot`, `EmuSen_Settings_Reference.md` §4.70). `EmuSen_Galaxia.md`
§3.6 records the decision to keep that default rather than move it under the home. A sandbox that wants its logs kept
apart names a `LogDirectory`.

## 10w. The library's runner outlived its answers: two runners for one game (2026-11-01)

**What was seen.** In the tester's real-window run of the published build 115ee07ee on 2026-11-01 (Cores, Approve on
PCEE2, then A on Jak and Daxter at 23:31:15), two `emusen-core-runner` processes stood beside Mistress while the game
ran, started about 23 s apart.

**The cause, read and then shown.** §3.2 gives the proxy two kinds of runner: one for the library-level calls (`info`,
`settings_schema`, `firmware_for`, `status_text`), and one per `create`. It says the library's runner exits once its
answers are cached. The proxy did not end it. The link was held in a process-wide `static LIBRARY`, which nothing ever
cleared, and the runner ends only when its socket closes. So the first library-level call of the session (Approve's
`info`, or the route's firmware question) started a runner that lived until Mistress exited. The game's own runner
started beside it at `create`. On the handheld that costs a second process holding a copy of the libretro core,
for nothing. A broker test written first showed it: after `info` and `firmware_for`, one runner was still alive
(`left: 1, right: 0`).

**The fix** (`emusen-core-broker`'s `proxy.rs`):
- A library-level call runs on a `LibraryRunner` of its own, and the runner ends when the call returns. Its log is
  drained into the library's log before the socket closes.
- The descriptors (capabilities, info, schema) are asked of one runner in a single run, as before.
- For a target whose firmware answer does not read the image (`firmware_reads_image`, false for libretro, §10r), that
  answer is fetched in the same run and cached. The launch's two firmware questions, the route's and `load_rom`'s,
  then start no runner.
- `set_crash_log` keeps the path and starts nothing. Each runner, library or machine, is given the path as it starts.
- A library-level `log_drain` answers from the proxy's own log and starts nothing.
- `status_text`, a firmware question that reads the image, and any call after a failed one each start a runner that
  ends with the answer. They are rare: an error's words, or a target no shipped proxy has.

**Test.** `tests/libretro.rs`'s `one_game_runs_one_runner_and_the_librarys_ends_with_its_answers`, written first and
failing:
- after `info` and `firmware_for`, no runner is alive;
- with a game running, the firmware question asked again leaves exactly one runner, and the library's log shows no
  runner started for it;
- draining that log starts none;
- once the game is freed, none is alive.

The broker's other 14 libretro tests pass unchanged.

## 10x. A pad's arrival froze the window: a ring deadlock, and bounded waits (2026-11-02)

**What was seen.** On the handheld on 2026-11-02, build dc6bc0a67, Jak and Daxter was running on PCEE2 through Steam
with two pads connected. A virtual Xbox 360 pad was removed at 13:11:13. The core's runner was gone afterwards, with no
core dump. Mistress's UI thread and a second thread named "EmuSen" were waiting in `drm_syncobj_array_wait_timeout`,
and the window no longer answered AT-SPI.

**Reproduced on the desktop** [M, 2026-11-02, out/ at dc6bc0a67, a fresh sandbox home, PCEE2 2026-11-02 build
(SHA-256 `e09d5435…`) installed and approved by pad]:
- The hang appears only when the screen presents on the device, so that PCEE2 draws into the ring ("in a ring" in the
  picture log). On the read-back path, pads arriving and leaving did nothing.
- A virtual pad was added and removed mid-game (`vpad.py`). The runner ended at 2 of 19 arrivals, both the first arrival of their run. Both times the
  error log said `emulation halt: Sony - PlayStation 2 (PCEE2) stopped: the core stopped answering within 10 s and was
  stopped`, 10 s after the arrival (13:31:52 and 13:36:50). The runner was not crashing: the broker's deadline (§3.4)
  stopped it with `SIGKILL`, which leaves no core dump.
- A stack of the hung Mistress process showed what was waiting:
  - The UI thread was in `Device::settle_held` → `vkWaitForFences(u64::MAX)`, called from
    `Chain::advance_ring`. It was waiting for its previous present, which waits on the ring's `ready`.
  - The second thread was Mesa's `vk_queue_submit_thread_func`, holding that submission until `ready` reached its
    value.

**The cause.** The runner's copy into a slot waits on `done` reaching that slot's last value (`released_at`). So
picture *V* is made only once *V − 3* is given back. A reader gives a picture back only by reading it, and the frame
hand-off drops a picture that a newer one replaces before the UI thread takes it.

When the presenter skips three pictures in a row and then reads the fourth, *V*, these waits form a cycle:
- the read of *V* waits for `ready` = *V*;
- the core's copy of *V* waits for `done` ≥ *V − 3*;
- only reading *V − 3* would have given that back, and that picture was skipped.

The runner's next frames fill its in-flight fences. `advance` then blocks until the broker's 10 s deadline kills the
runner. The UI thread waits on the fence for ever, because nothing raises `ready` while the session's close waits for
that same thread.

A pad's arrival is a moment when the UI thread is busy (the pad's notice, the library's hint). The present trace
(`EMUSEN_PRESENT_TRACE`) over four arrivals and four removals showed this:
- every arrival made a frame-number jump of 3 between two draws, with gaps of 25.7 to 43.5 ms;
- every removal made a jump of 2;
- the whole run's median gap was 17.3 ms and its 99th percentile 17.6 ms.

A jump of 3 skips two pictures and is safe. A jump of 4 is the deadlock. On the handheld's slower CPU the stalls are
longer, so there it happens every time. Any stall of the UI thread longer than three pictures causes it; a pad is only
the common trigger.

**The fix, in Serenity** (`ring.rs`, `chain.rs`, `vulkan.rs`):
1. *Skipped pictures are given back first.* Before a reader reads picture *V*, `InRing::take` signals `done` = *V − 1*
   in queue order, in a submission with no waits (`Device::signal_in_order`). It does this only when no reader on the
   device has given back that far yet; the ring's `given` counter holds the highest value submitted. A reader never
   reads below *V* again, so the slots it skipped are free for the core. The signal follows every earlier submission on
   the queue, so `done` only ever rises. A read whose value was already given back by another reader signals nothing.
2. *No wait on a core's process is unbounded.* `take` waits for `ready` = *V* on the CPU, bounded by `READY_BOUND`
   (2 s), and submits the read only once it has arrived. No host submission ever waits on a value a dead core might
   never signal, and Mesa's submit thread is never used for one. Past the bound the read fails with "the core's picture
   *V* did not arrive within 2 s".
3. Every wait on the device's own fences (`settle_held`, `run_timed`) is bounded by `FENCE_BOUND` (10 s) and reported
   as an error.

The core still sees a port's device change as libretro defines it, unchanged: `retro_set_controller_port_device`
with `RETRO_DEVICE_NONE` when a pad leaves, and the port's pad device when one arrives (§4.4). Headless, through the
`ring` example's new `EMUSEN_RING_PRESENCE=<frame>:<port>:<0|1>,...`, pulling and plugging in ports 0 and 1 during
Jak's boot and in play changed nothing on the 2026-11-01 build. A real uinput pad removed during a headless run did
nothing either.

**Stale locks.** The `.lock` files left beside the cards and the console memory after the frozen process was
terminated name its pid. They are advisory `flock` locks (`EmuSen_SaveMedia.md` §3.5), which the kernel releases when
the process dies, and `medium_lock::try_take` takes such a file over. Every relaunch on 2026-11-02 started Jak with
them present. `galaxia`'s `a_file_another_open_file_has_locked_is_held_by_another_program_and_a_stale_one_is_taken`
covers it.

**Tests**, each run by name:
- `serenity` `ring::tests::a_picture_read_after_three_skipped_ones_gives_them_back_and_arrives`: a stand-in core
  thread makes each picture once `done` ≥ *v − 3*; the reader reads 1, then 5. On the old code it is still waiting
  after 5 s; with the fix it passes.
- `serenity` `ring::tests::a_picture_that_never_arrives_is_given_up_within_a_bound`: on the old code still waiting
  after 5 s; with the fix it ends in the words above.
- `emusen-core-broker` `api_ring` `a_presenter_that_skips_pictures_while_pads_come_and_go_never_stalls_the_core`: the
  test core in the confined runner, with a ring through the frontend API. It runs 30 frames, draws every fifth and
  pulls and plugs in ports 1 and 0. On the old code it hangs at the first skip; with the fix it passes in 19.7 s.
- WiseMan `HostedCoreLaunchTests` on the Rust runtime:
  - `A_runner_killed_mid_game_ends_the_session_in_words_and_the_next_game_runs`;
  - `Pads_pulled_and_plugged_in_mid_game_leave_the_session_running`.

**In the real window** [M, 2026-11-02]. Setup:
- out/ at dc6bc0a67 with this change's platform library (release profile).
- A fresh home; PCEE2 installed and approved by pad; Jak started in the ring.
- Silent output; focus confirmed through AT-SPI before every press.
- A coverage run of another branch was using the CPU at the same time.

Results:
- *Pads.* Five times, a second virtual pad arrived mid-game and left 5 s later. The runner lived through all five,
  and the error log gained no line. Frame rate per second:
  - mostly 59.8 to 60.1;
  - single windows of 52.8 and 57.5, and a run of 51.0 to 58.4 where the core's own `retro_run` rose to 15 to 18 ms
    (`sleep+rest` near 0), which is the core's load, not a wait;
  - 40 s with no pad events afterwards: 59.8 to 60.2.
- *A runner killed.* `SIGKILL` on the runner mid-game. Within 0.12 s the window showed "[CPU HALT] Sony -
  PlayStation 2 (PCEE2) stopped: the core's process ended on signal 9". The error log has `emulation halt: …
  signal 9 (detail 0)` with its crash report. No thread waited in `drm_syncobj`, and AT-SPI answered. Guide opened the
  game menu, and Start and B each closed it. Leave, Back to Library, and A with Restart started Jak again, in the ring,
  at 59.9 fps.

**Seeded faults**, each caught by the `serenity` tests above:
- the skipped pictures not given back: the skip test, still waiting after 5 s;
- `ready` not awaited on the CPU: the bound test, still waiting after 5 s;
- too little given back (*V − 4*): the skip test.

**Left.** A runner that dies while its game is paused is noticed at the next frame the game runs, because nothing
calls it in between.

## 10y. The game kept running behind the Pause Card's settings, with nobody reading the ring (2026-11-02)

**What was seen.** On the handheld on 2026-11-02, build cf971987b (with §10x), Select and Start opened the Pause
Card at 17:52:56. The tester went to Game Options, Display, Graphics and changed PCEE2's Internal Resolution from 1x
to 2x. At 17:53:06 the error log said `emulation halt: … the core stopped answering within 10 s and was stopped`, and
the runner was gone. The UI stayed responsive: §10x's bounds held.

**Reproduced on the desktop** [M, 2026-11-02, out/ at cf971987b, a fresh home, PCEE2 installed by pad, the picture in
the ring]:
- The Pause Card left open for 16 s did nothing.
- Opening Game Options, Display and then Graphics ended the session at 18:04:57 and again at 18:08:17. The second
  time, Graphics opened at about 18:08:07, 10 s before the halt.
- A stack of the runner taken 3 s after Graphics opened showed its main thread inside `advance`:
  `retro_run` → `video_refresh` → `Gpu::frame` → `Gpu::submit` → `vkWaitForFences`. Mesa's submit thread was
  waiting on the ring's `done`. So an advance was issued as Graphics opened, and it blocked.
- After the halt the presenter's next read gave up within its bound ("the core's picture 1832 did not arrive within
  2 s", §10x).
- A second handheld run with nothing changed narrowed it the same way. The card left idle for 20 s did nothing. The
  halt came 10 s after Graphics opened while its rows were stepped through.

**The cause.** The guess that the paused core itself blocks the window's calls (`RuntimeCatalog.SettingsOf`, a row's
notes) is refuted. The runner was serving `advance`, not a settings call, and with the fix the same steps leave the
status at "Paused" with no frame run.

The advance comes from the hand-over between the card and the sheet:
- `ClosePadMenu` resumes what the card paused while the game is still on screen;
- `OnSheetsChanged` then pauses again for the sheet;
- in between, the emulation thread can run a frame or more.

Those frames, and any offered just before the card covered the screen, are dropped by the frame hand-off when a newer
one replaces them before the UI thread takes them. A dropped ring picture was never read, so it was never given back.
§10x gives back skipped pictures only when a later picture is read, and behind the card and the sheet none is. With
the core's three slots held, an advance blocks, and the broker's 10 s deadline stops the runner. Any time the game
runs while the screen draws no ring picture has the same effect.

**The fix.**
- *The hand-off gives back what it drops.* `FrameHandOff` takes a `releaseRing`. When a ring picture is replaced
  before the UI thread took it, the producer thread hands it to Mistress's `SlangRingRelease`, the path that already
  gives slots back while the screen does not draw on the device. That path reads the picture with a one-pixel draw
  and signals `done`. A picture the session's end drops is not given back, because its ring may already be freed. A
  screen that is covered, paused or hidden therefore never holds the core, whatever runs behind it.
- *`done` only ever rises, whichever thread gives back* (Serenity). Every `done` signal now goes through
  `Device::signal_in_order`. Under the queue's lock it compares the value with the ring's `given` counter and submits
  the signal only when it is higher. A read's own `done` is signalled after its read in queue order, so the emulation
  thread's release and the UI thread's present can interleave without a lower value ever following a higher one.

**In the real window, with the fix** [M, 2026-11-02, this branch published to a scratch folder, a fresh home, PCEE2
installed by pad, Jak in the ring]:
- *The second handheld run's steps.* The card was left idle for 20 s. Graphics was opened and 25 rows were stepped
  through at about 2 a second, then left 5 s. The runner lived, the status read "Paused" throughout, no frame ran,
  and the error log gained no line.
- *The option.* Internal Resolution was set from 1x to 2x while paused, about 3 minutes into the pause. On closing
  Graphics the game resumed at 59.9 to 60.0 fps, and the `[fps]` line's picture went from `GPU 512x448` to
  `GPU 1024x896`. After a restart the picture was again 1024 × 896, so the option applies on resume and is kept.
- *A long pause.* The card was left open for 62 s and then closed with B. The runner lived and the game ran on at
  59.9 to 60.0 fps.

**Tests**, each run by name:
- WiseMan `FrameHandOffTests.Ring_pictures_replaced_before_the_screen_took_them_are_given_back_and_a_shown_one_is_not`.
  Pictures 1 to 4, each replaced before the screen ran, are given back; picture 5 is shown and not given back; picture
  6, dropped when the session ended, is not given back. The test uses the new `releaseRing` parameter, so it cannot
  build on the old code. The failing-first evidence is the desktop reproduction above.
- `serenity`'s three ring tests (§10x, and §5.4's raise after a crash) pass on the reworked give-back.
- Seeded faults, each caught by the WiseMan test: the replaced picture not given back; the picture the session's end
  drops given back.

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
