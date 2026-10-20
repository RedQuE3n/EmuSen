# EmuSen_GpuPass — the GPU device and rendering pass: where the picture's time goes, and a plan

A plan (2026-10-20), nothing built. It measures where a game's picture spends its time between a core and the
screen on the two machines EmuSen is played on, ranks the changes that would attack that time, and orders them as
stages with gates. It stops with questions for the tester (§9).

**The decision it serves.** *Decided 2026-10-17 by the tester:* after the backend's port to Rust and before the
frontend's, three optimisation passes in this order: (1) this one, the GPU device and rendering; (2) MarsRT, so that
games such as *Perfect Dark* run faster; (3) Diamond. The backend's last step (B10, the C# deleted per component,
`EmuSen_Backend.md` §6) waits only on the tester's play; the tester asked on 2026-10-20 for this plan to be written now,
so that it is ready when B10 closes.

**Its scope, as agreed.** The ash Vulkan device of the platform library (`Platform/serenity/src/vulkan.rs`); the slang
chain, which since 2026-10-17 runs both the RetroArch presets and the built-in filters, the CRT's three tiers among
them (`EmuSen_DevicePicture.md` §20, §21; the SkSL `FilterChain` that the scope of 2026-10-17 also names was removed
the same day, its text now the chain's Vulkan GLSL); the presenter (`DevicePresenter`, the Avalonia compositor's import, the readback fallback, and
the pacing that feeds them); and the device lend (`EmuSen_CoreAPI.md` §6.29, `EmuSen_DevicePicture.md` §17). Out of
scope: macOS and KosmicKrisp (paused since 2026-10-17, `EmuSen_RustPlatform.md` §31), though nothing proposed here is
Linux-only in its design; the HDR presenter (`EmuSen_CRT.md` §19, `EmuSen_LunaP_Kit.md` §5.4), which is low
priority and is not built, but which nothing here may make harder (§7); the inside of any core.

**The rules that bind it.** Accuracy is never traded by default: a lever that changes a picture is a setting the
player opts into, and every lever below is held to produce the same pictures, byte for byte, or it is not proposed
as a default. Nothing is optimised before it is measured, and every gain below is predicted with a number first so
that it can be retired. The Mars probe grades only *Super Mario 64* and *Ocarina of Time*.

**Reading order.** §1 is the paths a picture takes today, read from the code. §2 is where the time goes, measured on
2026-10-20. §3 is the presenter's lost pictures on the television, which has a diagnostic of its own before any
remedy. §4 is the levers, ranked by the measured cost each attacks; §5 those considered and not proposed. §6 is the
stages and their gates; §7 what the pass leaves for the frontend's port; §8 the risks; §9 the questions; §10 the
predictions, gathered for retirement; §11 the sources.

**Marks.** **[M]** measured on 2026-10-20, with how; **[L]** read from the tester's own logs on the handheld, in place;
**[R]** read from the code or a page named; **[S]** from a published source of §11, read on 2026-10-20; **[A]**
argued; **[C]** a choice. Every lever says whether it comes from published guidance or from this project's own
measurement.

---

## 0. The plan in brief

**Where the time goes** (§2). On the desktop's RX 6800 every built-in filter and every preset measured fits a
60 Hz frame with a wide margin: the CRT's passes cost 0.48, 0.72 and 1.79 ms of GPU time at 1080p for Performance,
Balanced and Accurate, and 1.03, 1.61 and 3.15 ms at 4K [M]. On the handheld, at its GPU's paced clock, the same
passes cost 0.79, 1.80 and 7.55 ms at the panel's 1280 × 800 and 4.28, 7.31 and 8.20 ms at the
television's 3840 × 2160 [M]: the tiers' costs there were never measured before (`EmuSen_CRT.md` §12.7), and the
predictions C1 to C3 for it are refuted high (§2.4). The presenter on the device path costs the processor 0.9 to
1.0 ms a frame more than Skia's plain picture did, most of it on Avalonia's render thread (§2.3) [M]. Shader builds are
already cheap after the cache and parallel compilation of 2026-09-24: a CRT tier builds in 28 to 123 ms, Mega Bezel
in about half a second (§2.5) [M].

**What is wrong, as opposed to slow.** Three things were found, each to be shown by a failing test before it is
touched (§2.7, §3):

- **D1.** The NES's raw picture is handed to a built-in preset that has no `raw` image whenever the raw encoding
  outlives the filter that asked for it, and the CRT then stops until it is chosen again; the handheld's log of
  2026-10-19 holds the line it writes [L].
- **D2.** Replacing that raw image is a submission the processor waits for, on the thread that presents, every
  frame: +0.18 ms on the desktop's readback path and +2.1 ms on the handheld's [M], and five of the handheld's eight NES
  sessions spent 2.6 to 3.5 ms presenting a frame, against 0.26 to 0.34 ms in the other three [L]; the log does not
  name the filter, so that the five are the raw picture's is a reading stage 2 tests.
- **The presenter's lost pictures on the television** (`EmuSen_Settings_Reference.md` §4.87.14): measured again from
  42 sessions' logs (§3.1), they are not a property of one refresh a frame alone; they change by a factor of thirty to a
  hundred and forty between sessions of the same console and lock, and fell when the Master System gained its CRT. Their
  diagnostic is a trace that does not yet exist (§3.3), and the tester must take it with the television attached.

**The levers** (§4), ranked by the cost each attacks: L1, the lost pictures' diagnostic and then their remedy; L2,
Avalonia's per-update copy, with its measured alternatives put to the tester; L3, the raw image's upload folded
into the frame's one submission (with D1); L4, MarsRT's picture handed on the device, whose host side exists and
whose core side belongs to the second pass; L5, the descriptor writes of a long preset; L6, a large frame copied
straight into the device's staging memory; L7, no allocation per frame on the present path; L8, the last pass drawn
into the shown image. Each has a numbered prediction (§10).

**The stages** (§6). Stage 0 records the baselines; stage 1 is the trace and the tester's run on the television;
stages 2 and 3 are the exact, small changes (D1, D2, L7, L8); stage 4 measures Avalonia's Vulkan mode for the
tester's decision; stages 5 to 7 are conditional on what the earlier ones show. Stage 1's run and stage 2's code can
proceed side by side; every GPU-heavy measurement on the RX 6800 runs alone, under the lock the benches already take.

---

## 1. The paths a picture takes today (read 2026-10-20)

### 1.1 The three paths

Three ways reach the screen from a core [R: `GameFrameControl.cs`, `DevicePresenter.cs`, `SlangRunner.cs`,
`Platform/serenity/src/chain.rs`, `lend.rs`, `vulkan.rs`; `EmuSen_DevicePicture.md` §21.2]:

| Path | When | Thread by thread |
|---|---|---|
| **Device path, pixels** (the default for every core but Diamond) | a compositor that imports the device's image (§4.4 there) | emulation thread: the core's `frame_copy` into one of `PictureSlots`' four buffers (`EmuSen_Backend.md` §14.11). UI thread, at its next `Render`: the frame copied into the chain's staging buffer (`advance`), the raw image replaced if the filter reads one (a submission waited for, §2.7), then one submission that uploads the frame, runs every pass and copies the last into one of three exported images (`present`), returned from without waiting. Render thread: Avalonia's server job waits on the image's `ready` semaphore, **copies the image into a texture it makes for the update**, signals `free`, composes the window and swaps (`EmuSen_DevicePicture.md` §1.2, §13.1) |
| **Device path, lent** (Diamond with a device lent) | the same, and a core that claims `EMUSEN_CAP_DEVICE_PICTURE` | emulation thread: the core's picture copied on the device into a ring of three device buffers of the host's, on the intake queue (`Lend::take`), `done` signalled. UI thread: the newest ring buffer copied into the chain's history image inside the frame's one submission, then as above |
| **Readback** (the fallback: device path refused or given up, `EMUSEN_DEVICE_PICTURE=0`, a headless window) | | render thread: the plain picture by Skia's `DrawImage`; a filter or preset by the same chain, its last pass read back into a mapped buffer lent to Skia, and Skia's upload and draw (`EmuSen_Serenity.md` §9.1) |

### 1.2 The copies of one picture

On the device path with pixels, one picture is copied **once on the processor** (into the staging buffer; the
`frame_copy` into the slot is the core ABI's own hand-over) and **three times on the GPU**: staging to history image,
last pass to the exported image, and that image to Avalonia's texture. Lent, it is copied **four times on the GPU**
and never on the processor: the core's buffer to the intake ring, the ring to the history image, the last pass to
the exported image, and Avalonia's copy. The readback path adds the copy back and Skia's upload to the passes.

### 1.3 Where the processor waits for the GPU

[R, `vulkan.rs`, `chain.rs`]

- **Before each present**, `settle` waits for the fence of the previous submission (one submission in flight). At
  60 frames a second with passes under a frame long, it has signalled by then [A]; no wait there was measured.
- **The raw image's replacement** (`load_lut` → `Device::upload` → `run`) submits and waits, every frame the filter
  reads a raw picture: it first settles the previous present and then waits for its own copy, which on a GPU shared
  with the compositor waits behind whatever the compositor has queued (§2.7, D2).
- **The readback path** waits for the whole chain on the render thread (`render_into`, `run_after`).
- **A size change** waits for the device to idle (`shape`, `wait_idle`), once.
- **The lend's intake** waits for a ring slot's previous copy only when all three are in flight.
- Nothing on the device path waits for the compositor on the processor: `free` and `ready` are waited for on the
  device (`EmuSen_DevicePicture.md` §4.2).

---

## 2. Where the time goes (measured 2026-10-20)

### 2.1 The instruments, and what each can and cannot see

| Instrument | What it measures | Where it ran |
|---|---|---|
| **The shader bench** (`EmuSen.WiseMan/Serenity/ShaderBench.cs`, `EmuSen_Serenity.md` §8.1), a host of its own under `~/.cache/emusen/gpu-pass/`, built from the code at be6101ab | the readback path frame by frame on a headless GL context and the slang device: each stage's processor time, every pass's GPU time by timestamp queries, the readback, Skia's GL time, managed allocation and collections; paced at 60 Hz, 60 frames of warm-up and 300 measured, each case its own process, rounds in rotated order, a figure the median over rounds of each run's median | the desktop, three rounds; the handheld, two rounds, published self-contained to a scratch folder and run headless under `systemd-run --user` |
| **The window probe** (`EmuSen_DevicePicture.md` §13.3's, copied and rebuilt from this branch) | the device path in a real Avalonia window in a nested, headless KWin (`kwin_wayland --virtual --xwayland`): processor time per thread from `/proc`, the presenter's skipped frames, managed allocation per frame | the desktop only (no window is opened on the handheld, by rule) |
| **The builds** (`kind=load`) | a preset's build whole, and Shaderc's share, with the SPIR-V cache cold and warm, with Mesa's disk cache on and off (`MESA_SHADER_CACHE_DISABLE`, §11 S12) | the desktop, three rounds and two; the handheld, two |
| **The tester's logs** (`~/.config/EmuSen/Logs/<console>/gui_*/general.log` on the handheld, read in place) | Mistress's `[fps]` line each second: frames offered and shown, the present's milliseconds, the core's milliseconds, the pacing decision | 42 sessions from 2026-10-17 to 2026-10-20 |
| `vulkaninfo`, `gamescope --help` | which extensions each driver offers; gamescope's own description of its limiter | both machines |

**The machines.** The desktop: Ryzen 7 7700X, RX 6800 under RADV (Vulkan) and radeonsi (GL), Mesa 26.2.4, Fedora 44;
the load average during the runs was 4 to 6 from other work on the machine, reported beside each result in the raw
files. The handheld: Legion Go S, Ryzen Z1 Extreme (RDNA 3, Phoenix), SteamOS 3.9.3, Vulkan RADV Mesa 26.2.0-devel,
GL radeonsi Mesa 26.1.2 (SteamOS's build), gamescope 3.16.30, on the charger, its GPU clock at the 800 MHz floor
when the runs began (`pp_dpm_sclk`).

**Not used, and why.** `pacebench` and `ipsample` measure the emulation thread, and the cores' own cost is not this
pass's (it is the second's and the third's); the core's share of each frame was read from the tester's logs instead
(§2.6). The Mars probe was not run, since no lever here changes a core.

### 2.2 The desktop: the chain, pass by pass

The shader bench, the readback path, the RX 6800, 1920 × 1080 unless said, an SNES-shaped 256 × 224 picture
moving every frame. Milliseconds a frame; "passes" is GPU time from the first timestamp to the last pass's; "render"
the render thread's whole time; the range is the three rounds' [M, `results-frame.txt`]:

| Case | Passes (GPU) | Readback (GPU) | Skia's draw of the result (render thread) | Render thread, whole | Managed bytes a frame |
|---|---|---|---|---|---|
| plain picture (Skia's `DrawImage`) | — | — | 0.06 GL | 0.07 | 948 |
| CRT, Performance | **0.48** (0.46–0.54) | 0.21 | 0.30 | 1.27 | 2,087 |
| CRT, Balanced | **0.72** (0.71–0.75) | 0.21 | 0.30 | 1.56 | 3,463 |
| CRT, Accurate | **1.79** (1.79–1.83) | 0.21 | 0.32 | 2.58 | 3,559 |
| CRT, Performance, 4K | **1.03** | 0.81 | 1.05 | 3.13 | 2,087 |
| CRT, Balanced, 4K | **1.61** | 0.81 | 1.06 | 3.77 | 3,463 |
| CRT, Accurate, 4K | **3.15** | 0.81 | 1.06 | 5.37 | 3,559 |
| CRT (Lottes) | 0.68 | 0.21 | 0.31 | 1.41 | 2,039 |
| Game Boy LCD, 160 × 144 | 0.14 | 0.21 | 0.28 | 0.80 | 2,087 |
| `crt-royale` | 0.67 | 0.21 | 0.31 | 1.48 | 3,660 |
| `crt-royale`, 4K | 1.23 | 0.81 | 1.12 | 3.64 | 3,660 |
| Mega Bezel SMOOTH-ADV | 2.52 | 0.21 | 0.44 | 4.43 | 7,952 |
| Mega Bezel SMOOTH-ADV, 4K | 5.58 | 0.81 | 1.10 | 8.49 | 7,952 |
| N64 640 × 240, CRT Balanced | 0.76 | 0.22 | 0.25 | 1.46 | 3,463 |
| N64 2560 × 960 (4×), CRT Balanced | 1.17 | 0.22 | 0.27 | 2.31 | 3,463 |
| N64 2560 × 960, plain picture | — | — | 0.39 GL | 0.87 | 948 |

**What it says.**

- **The tiers' GPU cost is where 2026-10-17 left it** (Performance 0.48, Balanced 0.82, Accurate 1.77 ms at 1080p,
  `EmuSen_DevicePicture.md` §22.3): 0.48, 0.72, 1.79. Nothing has drifted.
- **Where the CRT's time is.** Accurate's last pass, the face with the mask, glass and beam, is 0.49 ms at 1080p
  and 1.90 at 4K; its signal passes (encode, decode, persistence) are about 1.3 ms whatever the window. Performance
  at 4K is 0.91 of its 1.03 ms in its last pass. So at 4K the face pass is the cost, and at 1080p the signal [M,
  per-pass timestamps].
- **On the readback path the render thread's time is mostly not the passes**: the readback's copy (0.21 ms of GPU at
  1080p, 0.81 at 4K) and Skia's upload and draw (0.30 and 1.05 ms of the render thread) cost a light filter as much as
  its passes. This is
  the fallback, and the device path removes both (§2.3).
- **Mega Bezel's binding is visible**: its 48 passes' descriptor writes and uniform fills (`render.size.bind`) took
  0.28 to 0.59 ms of the processor a frame, against 0.002 to 0.008 for the CRT's two to ten passes and 0.02 to 0.05
  for `crt-royale`'s twelve (§4, L5).
- **A large frame costs the presenting thread its copy into staging**: 0.35 ms at the N64's 4× (9.8 MB), against
  0.007 for a 256 × 224 picture (§4, L6).

### 2.3 The device path in a window

The window probe in a nested headless KWin at 1920 × 1080 and 3840 × 2160, the RX 6800 by `DRI_PRIME=1`, 300 frames
offered at about 15 ms after 120 of warm-up, two rounds with the path's order reversed in the second. "Render" is the
thread that runs Avalonia's compositor (its GL context's submission thread, Mesa's `cs0`, is beside it); "UI" is the
thread on which the device path runs the chain and presents; the processor time is read from `/proc` in 10 ms
ticks, so ±0.03 ms a frame [M, `results-d1.txt`]:

| Picture | Window | Device path: UI + render + GL submission, CPU | Device path: the UI thread's present | Readback path: CPU, all threads | Readback path: the render thread's draw, wall |
|---|---|---|---|---|---|
| plain | 1080p | 0.30 + 0.97 + 0.23 = **1.50** | 0.17 | **0.52** (Skia's `DrawImage`) | 0.10 |
| CRT, Performance | 1080p | 0.27 + 0.87 + 0.20 = 1.34 | 0.15 | 0.92 | 1.42 |
| CRT, Balanced | 1080p | 0.25 + 0.80 + 0.20 = 1.25 | 0.17 | 0.85 | 1.69 |
| CRT, Accurate | 1080p | 0.27 + 0.78 + 0.18 = 1.23 | 0.17 | 0.80 | 2.67 |
| CRT (Lottes) | 1080p | 0.20 + 0.78 + 0.22 = 1.20 | 0.13 | 0.99 | 1.70 |
| `crt-royale` | 1080p | 0.38 + 0.83 + 0.20 = 1.41 | 0.24 | 0.76 | 1.59 |
| Mega Bezel SMOOTH-ADV | 1080p | 1.23 + 1.00 + 0.25 = 2.48 | **1.15** | 1.57 | 4.32 |
| plain | 4K | 0.28 + 0.90 + 0.25 = **1.43** | 0.15 | **0.57** | 0.19 |
| CRT, Performance | 4K | 0.25 + 0.88 + 0.25 = 1.38 | 0.15 | 2.14 | 4.18 |
| CRT, Balanced | 4K | 0.28 + 0.87 + 0.22 = 1.37 | 0.18 | 2.34 | 5.11 |
| CRT, Accurate | 4K | 0.52 + 1.40 + 0.37 = 2.29 | 0.33 | 2.52 | 7.05 |
| CRT (Lottes) | 4K | 0.30 + 0.98 + 0.25 = 1.53 | 0.17 | 2.38 | 5.21 |
| `crt-royale` | 4K | 0.37 + 0.82 + 0.23 = 1.42 | 0.25 | 2.39 | 4.66 |
| Mega Bezel SMOOTH-ADV | 4K | 1.63 + 1.12 + 0.28 = 3.03 | **1.57** | 2.07 | 9.08 |

**What it says.** Every frame offered was handed over on the device path (300 of 300, none skipped for want of a
free image).

- **The device path's floor is about 1.2 to 1.5 ms of the processor a frame, whatever is drawn**, against Skia's
  0.5 ms for the plain picture: **0.9 to 1.0 ms more**, of which the render thread's share is 0.55 to 0.65 ms (0.90
  to 0.97 against 0.32 to 0.37). On the device path the render thread draws nothing of the picture but Avalonia's
  copy of the imported image into a texture made for each update and the composition (`EmuSen_DevicePicture.md`
  §1.2). This is §13.4 there measured again five days later (1.23 ms then for the plain picture at 1080p on the UI
  and render threads, 1.27 now), and it is L2's cost.
- **For every filter the device path is still the cheaper**: the readback path's render thread waits 1.4 to 9.1 ms a
  frame for the chain and Skia's upload, where the device path waits for nothing.
- **Mega Bezel's present on the UI thread is 1.15 to 1.57 ms**, the only case above 0.33: its 48 passes' recording
  and binding, which is L5's cost seen in a window.
- **Nothing measured is near a frame's length on this desktop**; the question the costs raise is the handheld's and
  the weak laptop's, where the same work takes two to five times as long (§2.4) and no window was opened.

### 2.4 The handheld

The shader bench, headless, the readback path (the passes are the device path's own GPU work); on the charger; two
rounds; the panel's 1280 × 800 and 1920 × 1200 and the television's 3840 × 2160 [M, `deck-results-frame.txt`]:

| Case | Window | Passes (GPU) | Readback (GPU) | Render thread |
|---|---|---|---|---|
| plain picture (Skia) | 1920 × 1200 | — | — | 0.13 (GL 0.14) |
| plain picture (Skia) | 3840 × 2160 | — | — | 0.10 (GL 0.43) |
| CRT, Performance | 1280 × 800 | **0.79** | 0.11 | 2.10 |
| CRT, Balanced | 1280 × 800 | **1.80** (1.75–1.84) | 0.08 | 2.86 |
| CRT, Accurate | 1280 × 800 | **7.55** (7.53–7.58) | 0.08 | 8.38 |
| CRT, Performance | 1920 × 1200 | **1.48** | 0.29 | 2.63 |
| CRT, Balanced | 1920 × 1200 | **3.04** (2.99–3.09) | 0.20 | 5.31 |
| CRT, Accurate | 1920 × 1200 | **7.82** (7.43–8.21) | 0.17 | 9.48 |
| CRT, Performance | 3840 × 2160 | **4.28** | 0.53 | 6.88 |
| CRT, Balanced | 3840 × 2160 | **7.31** (7.17–7.44) | 0.52 | 10.06 |
| CRT, Accurate | 3840 × 2160 | **8.20** (7.85–8.54) | 0.52 | 11.41 |
| CRT (Lottes) | 1920 × 1200 | 3.43 | 0.17 | 4.77 |
| Game Boy LCD, 160 × 144 | 1920 × 1200 | 0.23 | 0.24 | 1.33 |
| `crt-royale` | 1920 × 1200 | 2.61 | 0.18 | 5.37 |
| NES, CRT Balanced, no raw picture | 1920 × 1200 | — | — | 4.56 |
| NES, CRT Balanced, raw picture | 1920 × 1200 | — | — | **6.69** (5.20–8.17) |
| N64 2560 × 960, CRT Balanced | 1920 × 1200 | 4.83 | 0.14 | 6.64, of which `advance` **0.85** |
| N64 2560 × 960, plain picture | 1920 × 1200 | — | — | 1.67 (GL 0.25) |

**What it says.**

- **Every tier fits a 60 Hz frame on the panel, and Accurate takes half of it.** At 1280 × 800 the three tiers'
  passes are 0.79, 1.80 and 7.55 ms of GPU time at the paced clock; at the panel's own 1920 × 1200, 1.48, 3.04 and
  7.82. Accurate's ten passes are four of about 1.6 ms (the encode and the two decoder passes at 2,048 samples a
  line, and the face) and six small ones: on this GPU the signal, not the face, is the cost at the panel's sizes, and
  the signal's passes cost 4.5 to 6.5 times the desktop's where the face costs 3.3 times [M, per-pass timestamps].
- **On the television the face pass is the cost.** At 3840 × 2160 the last pass alone is 4.08 ms of Performance's
  4.28 and 6.49 of Balanced's 7.31; Balanced, the default, takes 7.3 ms of each 16.7 ms frame before Avalonia's copy,
  gamescope's composition at 4K and its scan-out. That is the case the tester plays on the television, and the one
  §3's losses were counted in.
- **The ratio to the desktop is not one number.** Against the RX 6800's paced figures at 1280 × 800 of 2026-10-07
  (`EmuSen_CRT.md` §12.5: 0.39, 0.57, 1.58 ms), the handheld takes 2.0, 3.2 and 4.8 times as long; the heavier the
  tier, the further apart, since the paced desktop's light frames run at a low clock and the handheld's heavy ones are
  the signal passes' arithmetic.
- **D2 is large here**: the NES's raw picture adds 2.1 ms to the render thread's frame (0.6 and 3.4 in the two
  rounds), against 0.18 on the desktop.
- **A large frame's copy into staging** is 0.85 ms of the presenting thread at the N64's 4×, against the desktop's
  0.35.
- **The readback path holds 60 frames a second at every tier on the panel**, Accurate's render thread at 8.4 to 9.5
  ms, which answers what `EmuSen_DevicePicture.md` §18.9 step 4 asked of the fallback, headlessly; the tester's run
  there would confirm it under Game Mode.

**The tiers' predictions, retired** (`EmuSen_CRT.md` §6.1; C1 and C2 already refuted on the desktop, §12.5 there):

| | Predicted for the handheld at 1280 × 800 | Measured, paced | Verdict |
|---|---|---|---|
| C1 Performance | 0.2 to 0.3 ms | 0.79 ms | **Refuted, high**, by 2.6 to 4 times, for §12.5's reason there: the arithmetic between reads |
| C2 Balanced | 0.8 to 1.1 ms | 1.80 ms | **Refuted, high**, by 1.6 to 2.3 times |
| C3 Accurate | 5 to 6 ms | 7.55 ms | **Refuted, high**, by 1.3 to 1.5 times: the 2,048-sample signal passes, which C4 already found to be most of the desktop's frame, are most of this one |

The table of `EmuSen_CRT.md` §6.1 assumed one ratio between the machines (one L of 1.9 ms on the handheld); the
measured ratios run from 2 to 4.8 by tier, so the laptop's column there (C6), scaled the same way, should not be
trusted until it is measured.

### 2.5 Shader builds and pipeline creation

`kind=load` on the desktop: the build of a chain whole (sources read, both stages compiled through the SPIR-V cache or
Shaderc, pipelines made, lookup images decoded), the passes built fifteen at a time as production builds them
(`EmuSen_Serenity.md` §9.5), and beside it the sum of Shaderc's compiles done one after another. Milliseconds, the
range over rounds [M, `results-load.txt`, `results-load-nodriver.txt`]:

| Preset | Passes | Build, SPIR-V cache cold | Build, cache warm | Build, cache warm, Mesa's disk cache off | Shaderc alone, serial |
|---|---|---|---|---|---|
| CRT, Accurate | 10 | 80–123 | 28–29 | 54–57 | 691–703 |
| CRT, Performance | 2 | 79–121 | — | 86–91 (cold) | 147–158 |
| CRT (Lottes) | 1 | 107–109 | — | 124–125 (cold) | 68–70 |
| `crt-royale` | 12 | 151–162 | 71–73 | 494–495 | 1,021–1,058 |
| Mega Bezel SMOOTH-ADV | 48 | 550–564 | 421–451 | 977–1,223 | 3,659–4,799 |

On the handheld (two rounds, Mesa's disk cache on): the CRT's Accurate tier builds in 95 to 153 ms with the SPIR-V
cache cold and 31 ms warm, `crt-royale` in 176 to 185 and 81 ms; Shaderc's serial sum is 0.98 s and 1.4 s, 1.4 times
the desktop's [M, `deck-results-load.txt`].

**What it says.**

- **The SPIR-V cache and the parallel build hold** (`EmuSen_Serenity.md` §9.6): a built-in tier builds in a tenth
  of a second the first time and in under 30 ms after; nothing a player chooses waits for Shaderc's serial time.
- **The driver's own compilation is the rest.** With Mesa's disk cache off, a warm SPIR-V cache leaves `crt-royale`
  at half a second and Mega Bezel at one, the difference being RADV compiling 24 and 96 stages; with the disk cache on
  (the default, §11 S12), it is what was measured in the first two columns. A persisted `VkPipelineCache` was
  measured on 2026-09-24 to add nothing over Mesa's own cache (`EmuSen_Serenity.md` §8.6), which agrees with the
  published advice only in part: the Vulkan Guide and Kapoulkine recommend a persisted, validated pipeline cache
  (§11 S10, S11), a recommendation written for drivers that keep none of their own [S, A].
- **A single-pass filter cannot be built in parallel**: Lottes's one pass takes as long as its one Shaderc compile
  and its pipeline, about 0.1 s.
- **Mega Bezel's warm build is 0.42 s**, of which the 23 lookup images' decoding is about 0.08 s and the pipelines
  most of the rest; it is the only build a player would notice, and only when browsing presets.

### 2.6 The handheld's sessions: what a frame cost the tester

From the `[fps]` lines of the 42 sessions [L]: the core's own time a frame was 0.2 to 0.3 ms for the Master System,
the SNES and the Genesis, 1.6 to 2.0 ms for the NES (MoonRT), and the present on the device path (Mistress's "draw" on
that path) 0.15 to 0.48 ms for every console but five of the NES's eight sessions, where it was 2.6 to 3.5 ms. That
the five are D2's raw picture is a reading, not a measurement: the log does not name the filter, and one of the five
(`gui_20261019_235133`) is the session whose CRT stopped in its first second (D1, §2.7), so either the filter was
chosen again in it or the reading does not hold there. Stage 2 measures it. No session's emulation thread missed its
frames for want of time. Every core but Diamond ran on the pixels path ("the engine took no lent device"); Diamond's
three sessions were lent, its frame 5.9 to 7.2 ms
(the enhanced renderer at its internal resolution, the lend's take included) and its present 0.22 to 0.27 ms, the
same as the pixels path's. The lend's own share was not measured again: on 2026-10-15 it was 0.79 to 0.89 ms of the
emulation thread's frame lent against 1.33 to 1.57 ms with the readback, Diamond at 2× on the desktop
(`EmuSen_DevicePicture.md` §17.2); the take has since moved into the Rust runtime
(`EmuSen_Backend.md` §29.20), its parity held there and its time not measured. Stage 0 times it again.

### 2.7 Allocations, copies and synchronisation on the hot path

**Allocations** [R, M]. The rule is no allocation per frame on a hot path (`EmuSen_Serenity.md` §2.2). What a
frame allocates today:

- *Rust, the chain*: two vectors per pass per frame in `Chain::bind` (`chain.rs:1051`, `:1061`) and one per pass
  with push constants in `draw` (`:1085`); a submission's wait and value lists in `run_unwaited` and `run_after`
  (`vulkan.rs:733`, `:773`). For Mega Bezel that is about a hundred heap allocations a frame.
- *C#, the device path*: a task and a continuation per update in `DevicePresenter.Present` (`DevicePresenter.cs:211`,
  `:212`); on the readback path 2 to 8 KB of managed memory a frame (§2.2), with no collection in 300 frames.
- *In the window* (the window probe, process-wide, so Avalonia's own and the probe's included): 12.6 to 14.1 KB a frame on the
  device path against 8.2 to 10.1 KB on Skia's, about 4 KB a frame more, and no collection in any run of 300 frames.

None of this was measured to cost a frame anything (no collection in any run), and the lever that removes it (L7) is
the rule's, not a speed lever.

**Copies** [R, §1.2]. The device path's three GPU copies of the shown picture cost of the order of 0.02 to 0.05 ms
each at 1080p (`EmuSen_DevicePicture.md` §13.4); the processor's one copy is §2.2's `advance`, small but for large
frames.

**Synchronisation** [R, §1.3; M]. The one stall a frame measured on the presenting thread is D2's: on the readback
path the desktop pays 0.18 ms for the NES's raw picture (1.81 against 1.63 ms for the same CRT tier without it), the
handheld 2.1 ms; on the device path the tester's handheld paid 2.6 to 3.5 ms (§2.6). The difference between the
two paths is explained by §1.3: on the device path the wait is for a submission on the graphics queue the compositor
is also using, so the processor waits behind the compositor's work as well as its own copy [A; to be shown by stage
2's measurement]. Image layout barriers outside the render passes use the widest stages (`ALL_COMMANDS` to
`ALL_COMMANDS`, `vulkan.rs:811`), which published guidance names as a drain of the pipeline (§11 S2, S3); there are
four to six of them a frame and none between passes, which are ordered by their render passes' own dependencies.

**D1, the raw image handed where no pass reads it** [R, L]. `GameFrameControl.RawImage` (`GameFrameControl.cs:57`)
gives the chain a raw image whenever the offer carries one of the filter's encoding; the preset has a `raw` image only
when one of its passes reads it (`BuiltInPreset.cs:265`), which depends on the tier and the signal
(`ScreenFilter.ReadsRaw`), while the core's raw picture follows the runtime's `raw_select`, which is a separate
decision taken from the control's `RawEncoding`. Whenever a frame carrying a raw picture reaches a chain whose preset
has no `raw` image, `SlangRunner` replaces an image the preset does not have, the chain throws, and the runner stops
for good (`SlangRunner.cs:117`, `:154`): "The built-in filter is not drawn: CRT stopped: The preset has no lookup image
called 'raw'". The handheld's log has the line at 23:51:33.258 on 2026-10-19, in the first second of that NES session
(`gui_20261019_235133`), so the trigger there was the session's start, a chain built before the console or its values
were settled, rather than a change of tier in play; a change of tier or signal to one that reads no raw picture is the
other way to it. The picture is then drawn plain until the filter is chosen again. A candidate, read and not yet
shown.

### 2.8 What could not be measured headlessly, and how the tester can

| What | Why not here | How the tester measures it |
|---|---|---|
| The device path on the handheld under Game Mode, its processor cost a frame | no window may be opened there | Mistress's `[fps]` line already carries the present's milliseconds and offered against shown; a minute each of the plain picture and the three tiers, as `EmuSen_DevicePicture.md` §18.9 step 4 asks |
| The lost pictures on the television | needs gamescope's real output and the television's latch | stage 1's trace (§3.3) |
| Avalonia's compositor and gamescope's composition on the GPU | no timestamps cross into another process's context | stage 1's trace bounds them from the processor's side; `MESA_VK_WSI_*` and gamescope's `--mangoapp` overlay (§11 S14) show the composition's frame time |
| GPU clock under play | the bench's paced load is lighter than a game's | `cat /sys/class/drm/card*/device/pp_dpm_sclk` every second beside the trace (stage 1) |
| The ten-minute run under Game Mode (`EmuSen_DevicePicture.md` §18.9) | owed since 2026-10-17 | as written there; stage 1's run can carry it |

---

## 3. The presenter's lost pictures on the television

### 3.1 What the logs say

`EmuSen_Settings_Reference.md` §4.87.14 counted, from the handheld docked to a 3840 × 2160 television at 60 Hz, 0.08
to 0.3 pictures lost a second while locked at one refresh a frame. All 42 sessions' logs since 2026-10-17 were read
again [L], each second counted as a loss when "shown" was below "offered" by more than half a frame, and as an extra
when above, the extra being the control: a second whose count of shown pictures merely straddles its boundary gives
one of each.

| Console | Pacing | Seconds | Lost a second | Extra a second |
|---|---|---|---|---|
| Master System | locked, one refresh a frame (60 Hz) | 1,219 | 0.028 | 0.001 |
| NES | locked, one refresh a frame | 214 | 0.107 | 0.000 |
| SNES | locked, one refresh a frame | 3,332 | 0.007 | 0.000 |
| PlayStation | locked, one refresh a frame | 47 | 0.298 | 0.021 |
| SNES | locked, three refreshes a frame (180 Hz) | 9,421 | 0.004 | 0.000 |
| SNES | locked, four refreshes a frame (240 Hz) | 2,759 | 0.020 | 0.002 |
| Genesis | locked, four refreshes a frame | 1,447 | 0.003 | 0.000 |
| SNES | content-paced | 1,568 | 0.196 | 0.042 |
| N64 (MarsRT) | any | 3,604 | 0.15–0.37 | **the same** |

The session-by-session figures, rather than these sums, are what matter:

- **The rate moves by an order of magnitude within one console and one lock.** The SNES at one refresh a frame lost
  0.001 a second over 2,780 s on 2026-10-17 and 0.11 to 0.14 on 2026-10-20; the Master System lost 0.08 to 0.11 a
  second in the two locked sessions before the install of 03:43 on 2026-10-20 and 0.003 to 0.012 in three of the four
  after it (the fourth, 34 s long, 0.06).
- **The install of 03:43 was `crt-sms`** (89c55fff: the Master System offered the CRT with its own row). The log does not
  name the filter in use, but the present's time rose from 0.16 to 0.23 ms across the install, as a heavier chain's
  would [L]; so the fall coincides, as far as the log can show, with the Master System's picture taking *more* GPU
  work, not less. §4.87.14 noted the coincidence and did not attribute it;
  §3.2's H5 is the hypothesis it suggests.
- **The N64's counts are not losses**: every second with one fewer shown is matched by one with one more. MarsRT
  hands over on its own frame boundaries and the counter's one-second windows split them; the `[fps]` count cannot
  see a loss there, and only a trace can.
- **Content-paced seconds lose the most** (0.16 to 0.30 a second), which is §4.87.1's beat and is not this defect.

### 3.2 The hypotheses, stated before the trace

Each predicts something the trace of §3.3 can show [A]:

- **H1, the hand-over too late for its refresh.** The 3 ms margin of §4.87.6 must cover, after the emulation
  thread hands a picture over: the render timer's next tick (≤1 ms at the held 1000 Hz, §4.87.10), the UI thread's
  present (0.15 to 3.5 ms, §2.6), Avalonia's commit to its render thread, the compositor's copy and swap, and
  gamescope's latch. *Predicts*: lost pictures follow presents that finished late against the vblank; a 6 ms margin
  (`EMUSEN_SYNC_MARGIN_MS=6`) cuts the rate by at least four times (P1).
- **H2, the UI thread held behind the render thread.** If Avalonia does not commit a new batch while its render
  thread is still in the previous swap, two offers fall into one UI `Render` whenever a swap returns late. *Predicts*:
  every loss follows a render-thread draw interval longer than a refresh plus a millisecond, whatever the margin
  (P2).
- **H3, gamescope changing how it presents.** §4.87.12 saw gamescope present on its grid and follow the draws by
  turns. *Predicts*: losses come in bursts at the changes, visible in the vblank counter's lattice (P3).
- **H4, the present's own cost.** *Refuted by the logs already*: the NES's sessions presenting in 2.6 to 3.5 ms lost
  0 to 0.14 pictures a second, and those presenting in 0.26 to 0.34 ms 0.11 to 0.31 [L]. Recorded so that it is not
  tried first.
- **H5, the GPU's clock.** The handheld's GPU idles at its 800 MHz floor (`Mars_Performance.md` §42.4); a light
  picture leaves it there, and the compositor's copy, gamescope's composition and its scan-out work all run slower
  at the floor than at the clocks a heavier load brings. *Predicts*: the loss rate falls as the picture's GPU work
  rises (plain against the CRT on one console), and the losses follow seconds in which `pp_dpm_sclk` sat at the floor
  (P4).

### 3.3 The diagnostic: the trace, extended, and the tester's run

**What the trace lacks** [R, `PresentationTrace.cs`]: it writes each frame's start and hand-over (F) and each
render-thread draw with the GLX vblank sample (D), but not when the UI thread presented the frame, when Avalonia
applied the update, or the GPU's clock. Stage 1 adds, all behind `EMUSEN_PRESENT_TRACE` and allocation-free as the
rest of the trace is:

- **P**: the UI thread's present, its frame number, its start and end, and whether a slot was free;
- **U**: the update's completion as the task returned by `UpdateWithSemaphoresAsync` reports it (§11 S13), with its
  slot;
- **C**: once a second, the GPU's current clock level read from `pp_dpm_sclk` where it exists.

**The run, the tester's** (it needs the television): the handheld docked at 3840 × 2160, 60 Hz, as on 2026-10-20;
a game whose picture changes every frame (*Sonic the Hedgehog* on Endou, or *Super Mario Bros. 3* on MoonRT, as
§4.87.14 names); five minutes each of (a) the plain picture, margin 3 ms; (b) the same, margin 6 ms; (c) the CRT
Balanced, margin 3 ms; with `EMUSEN_PRESENT_TRACE` and `EMUSEN_PRESENT_TRACE_SECONDS=320` in the launch options as
§18.9 of `EmuSen_DevicePicture.md` writes them. The analysis script reads the trace and gives each hypothesis its
verdict; no setting is changed on the strength of fewer than these three runs.

**What follows from each verdict** is L1's remedy (§4): a margin set from the measured distribution (H1); the
hand-over held until the UI thread's commit has been applied (H2); a lock that releases while gamescope follows the
draws (H3); or, for H5, nothing that touches the GPU's clock (keeping the device awake was measured and rejected,
`Mars_Performance.md` §42.13), but a margin that is the clock's.

---

## 4. The levers

Ranked by the measured cost each attacks. Each states the cost, a numbered prediction (§10), its risk to accuracy
(none is allowed by default), its size, the surfaces it touches (the core ABI `emusen_core.h`, the frontend API
`emusen_crystal.h`, the platform library's interface `emusen_platform.h`, now 22), and where it comes from.

### L1. The lost pictures: the diagnostic, then the remedy

- **Cost**: 0.003 to 0.30 pictures a second lost on the television, every core, at one refresh a frame (§3.1) [L].
- **Predictions**: P1 to P4 (§3.2); and **P5**, that with the remedy chosen by the trace the plain picture at one
  refresh a frame loses fewer than 0.01 pictures a second over five minutes on the television, as the SNES did on
  2026-10-17.
- **Accuracy**: none; pacing changes when a frame runs, never what it computes (`EmuSen_Settings_Reference.md`
  §4.87.8).
- **Size**: the trace, about 80 lines in `PresentationTrace` and `DevicePresenter` and a script; the remedy, small
  in any of §3.3's forms.
- **Surfaces**: none. The trace's format gains three line types; `FrameScheduler` stays the frontend's.
- **Origin**: own measurement; the principle that pacing must be measured at presentation, not at rendering, is
  published (§11 S8, S9).

### L2. Avalonia's per-update copy and texture

- **Cost**: the device path costs the processor 0.9 to 1.0 ms a frame more than Skia's plain picture across the UI
  and render threads (§2.3), on every frame of every game with the device path on; most of it on the render thread,
  where Avalonia copies the imported image into a texture it creates for each update (`EmuSen_DevicePicture.md` §1.2).
  This was accepted on 2026-10-15 as Q3 there, to be reconsidered if the handheld dropped frames for it.
- **Options, measured or to measure**: (a) leave it until the kit composites the image itself (`EmuSen_LunaP_Kit.md`
  §5.2); (b) Avalonia's X11 Vulkan mode with the slang device given to it (`CustomSharedDevice`), Q1 of
  `EmuSen_DevicePicture.md` §10, deferred then until the hand-over (that page's step (c)) was built, which it now is; (c) the GL-context import of
  `EmuSen_Serenity.md` §8.5.1, rejected on 2026-10-15 as reaching past Avalonia's surface.
  **A first attempt at measuring (b) did not reach it** [M, `results-vk2.txt`]: in the nested KWin, Avalonia 12.1's
  X11 Vulkan mode chose a device other than the RX 6800 even with `DRI_PRIME=1` and `MESA_VK_DEVICE_SELECT=1002:73bf`
  set, the presenter refused it by its UUID ("the compositor draws on another device", `EmuSen_DevicePicture.md` §4.4)
  and the picture was read back. Stage 4 must first set Avalonia's own choice of device. Recorded as a negative result;
  its figures (the readback composed on the other card) compare nothing.
- **Prediction P6**: with Avalonia's compositor on the slang device itself, the device path's processor time for the
  plain picture comes within 0.3 ms a frame of Skia's (from 0.9 to 1.0 ms above it now). This is argued from Avalonia's GL
  path, where the cost is an import, a texture made per update and a copy; its Vulkan path's import was not read, and
  if it copies the same way the prediction fails and L2 is (a).
- **Accuracy**: none; the picture is the same image.
- **Size**: (a) nothing; (b) a renderer change under every window of both frontends, with the whole fit audit and
  the window tests as its gate; measuring it is a probe.
- **Surfaces**: none of EmuSen's; (b) changes which Avalonia renderer the frontends ask for.
- **Origin**: own measurement.

### L3. The raw image's upload folded into the frame's submission, and D1

- **Cost**: D2's wait, +0.18 ms on the desktop's readback path, 2.1 ms on the handheld's [M], and, by the reading
  of §2.6, 2.3 to 3.2 ms of the handheld's UI thread on the device path in five of the tester's NES sessions [L];
  every frame of every NES game under a tier that reads the raw picture.
- **What it is**: `load_lut` for an image of the same size writes the staging buffer and records its copy into the
  next render's or present's one submission, as the frame's upload already is (`EmuSen_DevicePicture.md` §13.1); the
  replaced image is then written by the same submission that reads it. D1 is fixed with it: a raw image is given to a
  chain only when its preset has the image, and the runner does not stop on a frame that carries one it cannot use.
- **Prediction P7**: on the desktop's readback path the NES's CRT costs the render thread within 0.03 ms of the same
  tier without the raw picture; on the handheld's device path, as the tester's log shows it, the NES's present falls
  from 2.6 to 3.5 ms to under 0.5 ms. **P8**: D1's failing test (a frame carrying a raw picture given to a chain
  whose preset has no `raw` image, at a session's start and at a change of tier) passes, and no other picture changes: `BuiltInGoldenTests`' 125 hashes are unchanged.
- **Accuracy**: none; the same bytes reach the same image before the same passes read it.
- **Size**: small, in `chain.rs`, `SlangRunner.cs`, `GameFrameControl.cs` and the C# chain's twin.
- **Surfaces**: none; `emusen_serenity_chain_lut` keeps its signature and gains the deferred behaviour.
- **Origin**: own measurement and reading; published guidance agrees that a submission is not free and should come at
  a frame's end (§11 S1, "Command buffers").

### L4. MarsRT's picture handed on the device

- **Cost**: at a multiple, MarsRT's picture is composed on the host and handed as pixels: 9.8 MB a frame at 4×, its
  readback, composition and copy on the emulation side and the copy into staging (0.35 ms on the desktop's UI thread,
  0.85 ms on the handheld's) on the presenting side [M, §2.2, §2.4]. `Mars_Gpu.md` §16 measured the frontend's
  half at about 3 ms a frame at 4× on the old Skia path.
- **What it is**: `EmuSen_DevicePicture.md` §7's (2) and (3): the scan-out's raster kept on the device at every
  multiple and written opaque into a `PictureOut` slot, and the core ABI bit claimed. The host's side (the lend, the
  intake, the chain's intake) is built and in use for Diamond.
- **Prediction P9**: at 4× the emulation thread's frame falls by at least the readback's and composition's time,
  measured on *Super Mario 64* and *Ocarina of Time* only, and the picture on the device equals `frame_copy` byte for
  byte (the kit's C20).
- **Accuracy**: none; the exactness contract of `Mars_GpuPlan.md` §0 holds: the scan-out's picture is the host
  raster's, proven so as the averaged path was.
- **Size**: large, and in the core: a change of the scan-out's design.
- **Surfaces**: the core ABI's 1.9 as it stands; no change.
- **Origin**: own measurement.
- **Where it belongs**: the second pass, MarsRT's, since all of it is inside the core (Q4).

### L5. Descriptor writes and uniform fills only where they change

- **Cost**: Mega Bezel's 48 passes, 0.28 to 0.59 ms of the presenting thread a frame on the desktop (§2.2); the CRT's
  under 0.01 ms.
- **What it is**: each pass's descriptor set written when what it binds changes (a resize, a feedback turn, the
  history ring's rotation), the rest of the frame only the uniform block's changing members; the Rust allocations of
  L7 go with it.
- **Prediction P10**: Mega Bezel's `render.size.bind` falls below 0.1 ms on the desktop; the CRT's does not change
  measurably; every picture of the chain's parity class and the goldens unchanged.
- **Accuracy**: none.
- **Size**: medium: the history and feedback images rotate every frame, so their bindings change every frame; the
  saving is in what does not.
- **Surfaces**: none.
- **Origin**: published guidance ("Minimize the number of descriptor set updates", §11 S1) and own measurement. It
  matters only for long RetroArch presets, and is conditional (§6, stage 6).

### L6. A large frame copied straight into the device's staging memory

- **Cost**: the presenting thread's copy of the frame into staging, 0.35 ms at 9.8 MB on the desktop, 0.85 ms on
  the handheld (§2.2, §2.4), a second copy of a picture the frontend API already copies once out of the core.
- **What it is**: `PictureSlots`' buffers made in the device's host-visible memory, so that `picture_copy` writes the
  core's picture where the upload reads it; the chain's `advance` then records an upload from that slot and copies
  nothing on the processor.
- **Prediction P11**: the UI thread's `advance` at 2560 × 960 falls from 0.35 ms to under 0.02 ms on the desktop; no
  change for 256 × 224 pictures.
- **Accuracy**: none.
- **Size**: medium; it ties `PictureSlots` to the device, which the readback path's frames must still not need.
- **Surfaces**: `emusen_platform.h` gains a slot export (a minor of interface 22); the frontend API is untouched,
  since `emusen_crystal_picture_copy` already copies into the caller's memory (`EmuSen_Backend.md` §3.4).
- **Origin**: own measurement; RetroArch's driver writes the core's frame once into a host-visible texture
  (`EmuSen_Serenity.md` §8.7, read 2026-09-24).
- **Conditional**: it is worth building only if MarsRT stays on the pixels path at multiples (L4, Q4).

### L7. No allocation per frame on the present path

- **Cost**: §2.7's allocations; no measured time.
- **What it is**: the bind's vectors kept in the chain and cleared, the push-constant bytes in a buffer of the pass,
  the submission's lists on the stack; the presenter's continuation replaced by a check of the task where the slot
  is next used.
- **Prediction P12**: the present path allocates nothing in Rust (a counting allocator in the platform library's
  tests, as `emusen-crystal`'s `frame_alloc` test counts, `EmuSen_Backend.md` §29.16) and nothing managed in
  `DevicePresenter.Present`; the frame's time does not move by more than 0.02 ms.
- **Accuracy**: none. **Size**: small. **Surfaces**: none. **Origin**: the project's rule.

### L8. The last pass drawn into the shown image

- **Cost**: one GPU copy of the shown picture a frame, 0.02 to 0.05 ms at 1080p (`EmuSen_DevicePicture.md` §13.4),
  about four times that at 4K.
- **What it is**: the last pass's framebuffer made over the exported image, which must then be a colour attachment
  in the layout the compositor reads (linear on this machine, `EmuSen_DevicePicture.md` §13.2).
- **Prediction P13**: the GPU time a frame falls by 0.02 to 0.2 ms by size, and the compositor's picture equals the
  chain's own render, 0 channels differing, in the window probe.
- **Accuracy**: none, if the self-test of §13.2 there passes; a linear colour attachment must be offered by the driver
  for the format, or the copy stays.
- **Size**: small to medium. **Surfaces**: none. **Origin**: own reading; the copy is RetroArch's absence too
  (`EmuSen_Serenity.md` §8.7).

---

## 5. Considered, and not proposed

| | Lever | Why not | Origin |
|---|---|---|---|
| N1 | Precise barriers and `VK_KHR_synchronization2` in place of the `ALL_COMMANDS` transitions | published guidance names the widest barrier a drain (§11 S1, S2, S3; `VK_KHR_synchronization2`'s form, S4), but the chain's four to six such barriers a frame sit between transfers and passes that depend on each other anyway; predicted under 0.01 ms of GPU time at 1080p (**P14**). Done only as part of L3 or L8 where those lines are rewritten | published guidance |
| N2 | Two submissions in flight on the device path | the presenting thread's `settle` finds the previous present's fence signalled at 60 frames a second whenever the passes take under a frame (§1.3); no wait was measured. Reconsidered only if stage 1's trace shows one | own reading |
| N3 | A persisted `VkPipelineCache`, or `VK_KHR_pipeline_binary` (offered by RADV on both machines, §2.1) | measured to add nothing over Mesa's disk cache (`EmuSen_Serenity.md` §8.6, and §2.5 here); recommended by published guidance for drivers without one (§11 S10, S11) | both |
| N4 | `VK_EXT_graphics_pipeline_library` | no pipeline is made per frame; builds are already a tenth of a second | own measurement |
| N5 | Not clearing intermediate passes (`DONT_CARE`) | measured to change nothing on RDNA 2 (`EmuSen_Serenity.md` §8.5.5); published guidance prefers `LOAD_OP_CLEAR` for fast clears (§11 S1) | both |
| N6 | The readback made asynchronous (two frames in flight on the fallback) | a frame of latency, and the device path is the default; it would be a setting at most (`EmuSen_Serenity.md` §8.8) | own measurement |
| N7 | The lend's intake copy removed, the chain reading the core's own slot | the ring is what lets the core go at the device's pace whatever the window does (`EmuSen_DevicePicture.md` §16.3); it costs one GPU copy of the picture. Measured again only with L4 | own reading |
| N8 | Keeping the handheld's GPU clock up | rejected 2026-09-28 (`Mars_Performance.md` §42.13) | own measurement |
| N9 | `VK_EXT_present_timing` and `VK_KHR_present_wait2` for the display clock | offered by RADV on both machines (§2.1) and merged into Mesa for X11 and XWayland (§11 S6, S7), but EmuSen does not own a swapchain while Avalonia presents; it is the kit's presenter's (§7) | published guidance |

---

## 6. Stages, and their gates

In the form of the cores' plans (`Endou_Plan.md` §7). Every stage's gate must pass, or its misses be recorded and
decided, before the next that depends on it. Only blast-radius tests run for a stage; GPU-heavy runs on the RX 6800
run one at a time under `~/.cache/emusen/probe/gpu-heavy.lock`, with the validation layer's synchronisation checks
and its shader-access heuristic (`VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`) wherever a submission changes.

| Stage | What it covers | Its gate: what passes, measured how | Depends on |
|---|---|---|---|
| **0** | This plan; the baselines recorded: the bench's and the window probe's matrices of §2 kept as the "before" of every later stage, the handheld's bench cases kept on the desktop for its "after"; the lend's take timed on the Rust runtime (§2.6) | The tester's go; the take's time recorded | — |
| **1** | L1's diagnostic: the trace's P, U and C lines, the analysis script (H1 to H5 each given a verdict), the steps for the tester written beside §18.9's; the tester's run on the television | The trace's own tests (a line per event, nothing allocated, the format read back by the script); the tester's three runs taken and each hypothesis given a verdict in writing | 0 |
| **2** | D1 and L3: D1 shown by a failing test, then fixed; the raw image's upload folded into the frame's submission in both chains (the platform library's and the C# one behind its switch) | D1's test fails before and passes after; `BuiltInGoldenTests`' 125 pictures and `SerenityChainParityTests` unchanged; validation clean over the NES under each tier; P7 measured on the bench (desktop, three rounds) and on the handheld's bench (two) | 0 |
| **3** | L7 and L8; N1 where those lines are rewritten | The goldens and the parity class unchanged; the window probe's compositor picture equal to the chain's, 0 channels, plain, CRT Balanced and `crt-royale`; the layer clean with a positive control (a barrier removed, findings reported); P12 by a counting test; P13 by timestamps | 2 (the same files) |
| **4** | L2's measurement: the window probe in Avalonia's Vulkan mode against GLX, Avalonia's device set to the slang device's card and then given the slang device itself (`CustomSharedDevice`); the device path, plain, the three tiers and two presets, 1080p and 4K | The table, two rounds, both orders; the decision is the tester's (Q2) | 0 |
| **5** | L1's remedy, the form stage 1's verdicts choose | P5 on the tester's television; the pacing model's and `FrameSchedulerTests`' cases unchanged or extended with the new rule; a seeded fault of the remedy caught | 1 |
| **6** | L5, if the tester wants long presets cheaper (Q3) | P10; the parity class and the goldens; validation clean over Mega Bezel | 3 |
| **7** | L6, if MarsRT stays on the pixels path at multiples (Q4) | P11; the readback path's tests; the frame control's hand-off tests | 3 |
| **8** | The closing measurements: §2's matrices again on both machines; the tester's ten-minute run under Game Mode (`EmuSen_DevicePicture.md` §18.9, owed) carried by stage 1's or stage 5's session | Each prediction of §10 retired in writing | all |

**Side by side and in series.** Stage 1's code is the trace and does not touch the chain, so it runs beside stage 2;
the tester's run waits only for stage 1's build. Stages 2 and 3 edit the same files (`chain.rs`, `vulkan.rs`) and run
in series. Stage 4 is a probe outside the tree and can run whenever the RX 6800 is free. Every GPU-heavy measurement
(the bench matrices, the window probe, validation runs) runs alone on the RX 6800; the handheld's runs are headless,
under `systemd-run --user`, and only when `pidof EmuSen EmuSen.Mistress` finds nothing.

**Size.** Stages 1 to 3 are about four supervised steps of three hours; stage 4 one; stages 5 to 7 one or two each
if taken; stage 8 one. About 8 to 12 steps in all [A].

---

## 7. What the pass leaves for the frontend's port, and what it must not make harder

**Left for the port** [A]:

- **The presenter itself.** While Avalonia presents, EmuSen hands an image to a compositor it does not own; the copy
  of L2 and the display clock's inference from GLX (`EmuSen_Settings_Reference.md` §4.87.2) are both artefacts of
  that. The kit's `lunap-gpu` owns the swapchain (`EmuSen_LunaP_Kit.md` §5.2, §5.3) and can draw the chain's last
  pass into it, as RetroArch draws its last pass into its backbuffer (`EmuSen_Serenity.md` §8.7).
- **Present timing.** RADV offers `VK_EXT_present_timing`, `VK_KHR_present_id2` and `VK_KHR_present_wait2` on both
  machines [M, `vulkaninfo`], and Mesa merged present timing for X11 and XWayland (§11 S6, S7). The kit's clock
  should read the driver's presentation feedback instead of fitting a lattice to draws (`EmuSen_Settings_Reference.md` §4.87.3), which is what
  §11 S8 and S9 argue a frame pacer must do; stage 1's trace is the evidence it will be measured against.
- **HDR**: the swapchain's colour space and the encoding at the end (`EmuSen_LunaP_Kit.md` §5.4).
- **Timeline semaphores to the compositor.** Avalonia 12.1 offers `UpdateWithTimelineSemaphoresAsync` beside the
  binary form (§11 S13), and timelines let a host let go of several pictures with one signal, which is why the lend
  uses them (`EmuSen_DevicePicture.md` §5.2; §11 S5, which also discourages mixing the two kinds). Under GLX the
  compositor's capability is binary semaphores (`EmuSen_DevicePicture.md` §1.2), so the presenter keeps its pairs
  until the kit composites on the device with timelines throughout.

**Not to be made harder** [C]:

- **The chain's output format stays a parameter.** L8 must not fix the last pass's format to the exported image's
  `R8G8B8A8_UNORM`: a later presenter in HDR10 or scRGB asks the last pass for a 10-bit or half-float image
  (`EmuSen_LunaP_Kit.md` §5.4). The last pass's format is chosen by whoever gives the target, as now by `Shown`.
- **The device stays one, lendable, and made by whoever presents.** Nothing here may make the chain assume it owns the
  device: the kit may make the device and lend it to the chain (`EmuSen_LunaP_Kit.md` §9.2's K1 (b) showed ash and
  wgpu sharing one).
- **The trace's format stays the measurement's.** Stage 1's lines are written so that the kit's presenter can write
  the same ones from its present feedback, and the script reads both.
- **Portability.** Every lever is plain Vulkan 1.1 or 1.2 with a fallback; none depends on X11, GLX or gamescope
  except L1's remedy, whose form is chosen per presenter.

---

## 8. Risks

| | Risk | What tests it, or answers it |
|---|---|---|
| R1 | The losses on the television have a cause the trace cannot see (the television's own latch, gamescope's composition) | H3 and H5 are the trace's view of gamescope; a residue that none explains is recorded as a named loss, and the kit's present feedback revisits it |
| R2 | One vendor: every measurement is AMD under Mesa | the self-tests of `EmuSen_DevicePicture.md` §13.2 stay; no lever assumes a vendor's layout |
| R3 | A deferred upload (L3) or a direct last pass (L8) brings a hazard a waited submission could not have | the validation layer with synchronisation checks and the heuristic, a positive control each, and the goldens |
| R4 | The handheld's GPU clock makes its figures a sample of a clock state (`EmuSen_Serenity.md` §8.4) | medians over rounds; the clock read beside every run from stage 1 on |
| R5 | Avalonia's Vulkan mode (L2 (b)) changes every window's rendering | it is measured only (stage 4) and decided by the tester; the fit audit and window tests are its gate if chosen |
| R6 | The pass grows into MarsRT's work | L4 is assigned to the second pass unless the tester says otherwise (Q4) |

---

## 9. Questions for the tester

| | Question | Recommended |
|---|---|---|
| **Q1** | Stage 1's run needs the handheld docked to the television for three five-minute sessions with launch options set. Will you take it, and may stage 2 proceed beside it? | Yes to both: the trace is the only instrument that can tell the hypotheses apart, and stage 2 does not touch pacing |
| **Q2** | If stage 4 shows Avalonia's Vulkan mode removing most of L2's cost, should the frontends move to it now, or wait for the kit? | Decide on stage 4's numbers; recommended now only if it saves at least half of the device path's extra processor time on the desktop and passes the fit audit unchanged, since the kit is months away |
| **Q3** | L5 makes long RetroArch presets (Mega Bezel) cheaper on the processor by up to half a millisecond a frame and changes nothing for the built-ins. Build it in this pass? | Not now: no measured frame misses its time for it, on either machine; keep it conditional |
| **Q4** | MarsRT's picture on the device (L4) is inside the core. Is it this pass's or the second's? | The second's: it is a change of MarsRT's scan-out, and the second pass is MarsRT's; this pass keeps the host's side as it is |
| **Q5** | D1 is a correctness defect found while measuring. Fix it in stage 2 with L3, or at once as a small step of its own? | In stage 2, shown first by its failing test; it shares L3's lines, and its effect (the NES drawn plain for the rest of a game, from its start or a change of tier, until the filter is chosen again) is visible but harms nothing saved |
| **Q6** | The handheld's measurements were on the charger. Should stage 8 also measure on battery? | Yes, once, for the three tiers at the panel's size: `EmuSen_Serenity.md` §8.4 found the two differ |

---

## 10. The predictions, gathered

| | Prediction | Where | Retired by |
|---|---|---|---|
| P1 | A 6 ms margin cuts the television's losses by at least four times (H1) | §3.2 | stage 1 |
| P2 | Every loss follows a render-thread draw interval longer than a refresh plus a millisecond (H2) | §3.2 | stage 1 |
| P3 | Losses come in bursts where gamescope changes how it presents (H3) | §3.2 | stage 1 |
| P4 | The loss rate falls as the picture's GPU work rises, and losses follow seconds at the floor clock (H5) | §3.2 | stage 1 |
| P5 | With L1's remedy, the plain picture at one refresh a frame loses under 0.01 a second over five minutes | L1 | stage 5 |
| P6 | Avalonia's compositor on the slang device brings the device path's plain picture within 0.3 ms of Skia's processor time | L2 | stage 4 |
| P7 | The NES's raw picture costs the desktop's render thread under 0.03 ms more than none; the handheld's present falls under 0.5 ms | L3 | stage 2 |
| P8 | D1's test passes and every golden is unchanged | L3 | stage 2 |
| P9 | At 4×, MarsRT's emulation thread saves at least its readback and composition, the picture byte for byte the same | L4 | the second pass |
| P10 | Mega Bezel's binding under 0.1 ms on the desktop | L5 | stage 6 |
| P11 | The UI thread's `advance` at 2560 × 960 under 0.02 ms | L6 | stage 7 |
| P12 | The present path allocates nothing, and the frame's time moves by under 0.02 ms | L7 | stage 3 |
| P13 | 0.02 to 0.2 ms of GPU time saved by size; the compositor's picture unchanged | L8 | stage 3 |
| P14 | Precise barriers save under 0.01 ms of GPU time at 1080p | N1 | stage 3, if done |

---

## 11. Sources

**Published, read on 2026-10-20** [S]:

- **S1** AMD GPUOpen, *RDNA Performance Guide*: "Batch groups of barriers into a single call"; "Minimize the number of
  command buffer submissions to the GPU"; "Each submit has a CPU and GPU cost associated with it"; "Minimize the number
  of descriptor set updates"; "Use a pipeline cache to re-use PSOs across multiple runs".
  <https://gpuopen.com/performance/>
- **S2** Khronos, Vulkan Samples, *Using pipeline barriers efficiently*: `ALL_COMMANDS` to `ALL_COMMANDS` "will often
  cause a full drain of the pipeline". <https://docs.vulkan.org/samples/latest/samples/performance/pipeline_barriers/README.html>
- **S3** M. Chajdas, *Vulkan Barriers Explained*, GPUOpen, 2016-10-18: a barrier over every stage "introduces an
  unnecessary pipeline bubble". <https://gpuopen.com/learn/vulkan-barriers-explained/>
- **S4** R. Mun, J. Zulauf, J. Gebben, J.-H. Fredriksen, *Understanding Vulkan Synchronization*, Khronos blog,
  2021-03-24 (`VK_KHR_synchronization2`). <https://www.khronos.org/blog/understanding-vulkan-synchronization>
- **S5** J. Jones, *Vulkan Timeline Semaphores*, Khronos blog, 2020-01-15: a timeline semaphore is "a superset of both
  the original VkSemaphore and VkFence primitives"; "mixing binary and timeline semaphores is discouraged unless
  necessary"; window-system integration does not take timeline semaphores. <https://www.khronos.org/blog/vulkan-timeline-semaphores>
- **S6** L. Duc, *VK_EXT_present_timing: the Journey to State-of-the-Art Frame Pacing in Vulkan*, Khronos blog,
  2025-12-04: timing feedback on earlier presents and a target time for each new one.
  <https://www.khronos.org/blog/vk-ext-present-timing-the-journey-to-state-of-the-art-frame-pacing-in-vulkan>
- **S7** Phoronix, *Mesa's "Present Timing" Vulkan Driver Support Now Feature Complete*, 2025-12-11 (RADV among the
  drivers; X11 first); and, by its search summary only, the report of 2026-06-21 that X11 and XWayland support was
  merged for Mesa 26.2. <https://www.phoronix.com/news/Mesa-VK_EXT_present_timing>
- **S8** Android Developers, *Frame Pacing library*: presentation timestamps so that "frames are not presented early",
  and fences "to inject waits … rather than allowing back pressure to build up". <https://developer.android.com/games/sdk/frame-pacing>
- **S9** A. Ladavac, *The Elusive Frame Timing: A Case Study for Smoothness Over Speed*, GDC 2018: its abstract (the
  talk itself is behind the Vault's paywall and was not read): pace by the time a frame is presented, not the time it
  is rendered. <https://gdcvault.com/browse/gdc-18/play/1025031>
- **S10** Khronos, *Vulkan Guide: Pipeline Cache*. <https://docs.vulkan.org/guide/latest/pipeline_cache.html>
- **S11** A. Kapoulkine, *Creating a Robust Pipeline Cache with Vulkan*, 2021-02-26: validate the cache's header,
  UUID and driver version before use. <https://about.roblox.com/newsroom/2021/02/creating-a-robust-pipeline-cache-with-vulkan>
- **S12** Mesa, *Environment Variables*: `MESA_SHADER_CACHE_DISABLE`, `MESA_SHADER_CACHE_DIR`,
  `MESA_SHADER_CACHE_MAX_SIZE` (1 GB by default), `RADV_DEBUG=nocache`. <https://docs.mesa3d.org/envvars>
- **S13** Avalonia API, `CompositionDrawingSurface.UpdateWithSemaphoresAsync` (the task completes when the update is
  done) and `UpdateWithTimelineSemaphoresAsync`.
  <https://api-docs.avaloniaui.net/docs/M_Avalonia_Rendering_Composition_CompositionDrawingSurface_UpdateWithSemaphoresAsync>
- **S14** Valve, *gamescope* README, and `gamescope --help` of 3.16.30 on the handheld: `--framerate-limit` is "used
  as a divisor of the refresh rate"; `--adaptive-sync`; `--immediate-flips`; `--mangoapp`.
  <https://github.com/ValveSoftware/gamescope>

No emulator's or shader project's source was read for this plan.

**Which levers come from where.** Published guidance: N1, N3's recommendation, N5's, N9, and L5's and L3's
agreement. This project's own measurement: L1 to L8 and every figure of §2 and §3.

**The project's own pages**: `EmuSen_DevicePicture.md` (§1.2, §4, §7, §13, §17 to §22), `EmuSen_Serenity.md` (§2,
§8, §9), `EmuSen_CRT.md` (§6.1, §11.10, §12.5, §12.7, §17, §19), `EmuSen_Settings_Reference.md` §4.87,
`EmuSen_Backend.md` (§3.4, §14.11, §29.16, §29.20), `EmuSen_LunaP_Kit.md` §5, `Mars_Gpu.md` §16,
`Mars_Performance.md` §42, `Endou_Plan.md` §7.

**The raw files**, outside the tree: `~/.cache/emusen/gpu-pass/` (`bench/results-*.txt`, `nested/results-*.txt`,
`deck/results-*.txt`, the cases and scripts); the tester's logs were read on the handheld and copied only into a
scratch folder for counting.
