# EmuSen_LunaP_Kit — LunaP as a toolkit of its own, and the frontend's move to Rust

A plan (2026-10-10), with nothing built. It answers one question: what it takes to make LunaP a windowing and
interface toolkit in its own right, in the way GTK and Qt are, written in Rust, and to move EmuSen's frontends onto
it. It replaces the interface half of `EmuSen_Stack.md` §6, which chose a toolkit "by prototype and not argued in
advance" between Qt through cxx-qt and Slint; that prototype is kept, widened and made the plan's second gate (§9, K2).

**Reading order.** §0 for the decisions and the answer in brief. §2 for what exists today and §3 for the state of
the Rust ecosystem it would be built on, both measured. §4 to §7 are the design. §8 is what has to happen elsewhere
in EmuSen first. §9 is the order of work with a gate per step. §10 is the risk register and §11 the questions for the
tester. §12 lists the sources.

**Marks.** **[M]** measured or counted on 2026-10-10, with how; **[S]** read from a cited source, with its date;
**[A]** argued, not measured; **[E]** an estimate, with its basis.

---

## 0. The decisions, and the plan in brief

### 0.1 Decided by the tester, 2026-10-10

1. **LunaP becomes a toolkit of its own**, akin to GTK and Qt, written in Rust. Qt may supplement it where Rust has
   nothing, but the aim is Rust throughout.
2. **It is multi-platform**: Linux (Wayland, and directly on the display for a kiosk), Windows, macOS, and later
   Android and iOS.
3. **It is modular and Unix-like**: small parts with one job each and written surfaces between them, so that it can
   later be the base of a desktop environment of the tester's own design.
4. **Vulkan is the one render target**, with KosmicKrisp as the solution on macOS.
5. **The long road** is a retro gaming operating system: a Linux kernel and the libraries it needs, with EmuSen as
   its session.
6. **Accurate HDR is a requirement of the kit's presenter from the start**, not a later addition (§5.4).
7. **Its paradigm is declarative views over a retained widget tree, with fine-grained reactive state**, and
   one-way data flow for an application's own state (§4.2). Existing paradigms are used where they make the work
   easier; this one was chosen among them.

### 0.2 The plan in one paragraph

LunaP is rebuilt as a family of Rust crates, each with one job, layered as GTK sits on Wayland, Cairo, Pango and
HarfBuzz: LunaP owns the widget model, focus and controller navigation, theming, the ES-DE theme engine and its own
test harness, and builds on maintained community crates for windows, GPU, vector drawing, text shaping, layout and
accessibility, none of which would be improved by being written again (§3, §4). It draws through Vulkan alone, via
wgpu restricted to its Vulkan backend, so that one Vulkan device can be shared with EmuSen's own Vulkan code and the
game's picture never leaves the GPU (§5). Before LunaP's own core is committed to, two gates measure it: K1 proves
each platform and driver assumption on real hardware, and K2 builds one Big Picture screen on LunaP's prototype core
and on Slint and decides between them by numbers (§9). The frontends then move screen by screen against a recorded
specification of today's interface, never against Avalonia's internals, and only once EmuSen's runtime, Crystal,
offers the frontend API the frontends lack today (§7, §8). The operating system is the last stage: a compositor of
LunaP's own on Smithay, and an image-based Linux built on Fedora's bootc (§6).

### 0.3 The answer to "what will it take", in brief

| Part | Size **[E]** | Basis |
|---|---|---|
| The kit itself (§4) | 35,000 to 55,000 lines of Rust | LunaP is 26,133 lines of C# and 2,346 of XAML **[M]**, but sits on Avalonia; the kit must also own what Avalonia gave it (a widget tree, text layout glue, windows, popups) and three things no Rust toolkit has (§3.4): controller focus, a KMS backend, an ES-DE renderer |
| Its test harness and the ported assertions | 20,000 to 35,000 | LunaP's 957 tests and Mistress's 1,076 **[M]**, carried over as specifications rather than as code (§7.4) |
| The frontends on it | 20,000 to 30,000 | Mistress is 29,027 lines **[M]**, of which about 13,000 are portable logic already (§2.3) |
| The compositor and OS (§6) | not estimated | its size depends on decisions §11 leaves open |

It is the largest effort in EmuSen's plan, larger than every platform step of `EmuSen_RustPlatform.md` together
(about 14,600 lines of Rust **[M]**). It is feasible because the hardest lower layers now exist (§3), and it is
safe only if it is gated: two of the plan's assumptions (wgpu on KosmicKrisp, and Vulkan on iOS) are unproven
(§10).

---

## 1. Why a toolkit of its own, and why not one of the existing ones

**What LunaP is worth today is its layer, not its plumbing.** Its value to EmuSen is the controls the frontends
use (108 types named 1,421 times by Mistress **[M]**), the ES-DE look, the controller grammar and the rules the
interface is held to: the fit audit at two resolutions, pad reachability, accessibility, palette reach (§2.4). Avalonia
supplies the rest, and that is the part a Rust kit can take from the community.

**No existing Rust toolkit fits as a base** (§3.1), for reasons that are each decisive for one requirement:

- **iced, and libcosmic on it, will not go to phones.** Its maintainer stated on 2025-11-29 that he has "no interest in
  maintaining anything related to Android or iOS", and the mobile pull request was closed unmerged on 2026-02-15 **[S]**.
  It has no accessibility upstream (a full AccessKit pull request was declined on 2026-03-14) and no gamepad input
  **[S]**. COSMIC's pattern, a toolkit layer on iced, is the pattern this plan follows; iced itself is not the base.
- **Slint is the strongest base today**: Android and iOS, a KMS backend for kiosks, a stable 1.x API and a GPL-3.0
  option **[S]**. Against it: it is a declarative language with its own compiler, which a desktop environment of
  the tester's own would be built in too; its KMS backend is still marked experimental; Android has no accessibility;
  and its iOS path is Skia on Metal, so Vulkan-only on iOS is unverified **[S]**. It is the fallback K2 measures
  against (§9).
- **Makepad** runs one codebase on all five platforms, with Robrix as proof (published 2026-10-07) **[S]**, but it is
  one monolith with its own renderers and its own shader language, the opposite of the modular, Unix-like kit asked
  for, and its accessibility is a hook **[S]**.
- **Qt** has everything but gamepad support, which Qt 6 dropped **[S]**, and is C++. It stays available as a
  supplement (§11, Q9), not as the base.
- **No toolkit at all has controller-first focus navigation** **[S]**. LunaP must build it on any base, and it is the
  part EmuSen most depends on (§2.4).

**And a kit of its own is what a desktop environment needs.** A DE built on a toolkit whose model it does not own
inherits that toolkit's limits for good; GNOME owns GTK and KDE owns Qt for this reason. Building LunaP's own core on
shared lower layers is the same choice System76 made for COSMIC one level down, and Linebender made for Xilem.

---

## 2. What exists today (measured 2026-10-10)

Counted with `wc -l` over `.cs` and `.axaml`, `obj/` and `bin/` excluded, and grep over the public type names of
LunaP's 161 types; full tables in the survey behind this page.

### 2.1 LunaP

- 148 files, 26,133 lines of C#, 2,346 of XAML; 166 public types and 2,624 public members (`EmuSen_Surfaces.md` §3.1).
  It references Avalonia 12.1 and nothing else. MIT licence.
- **Portable as it stands: 2,001 lines.** 19 files (1,078 lines) never name Avalonia: the table's enumerations,
  `KeyboardLayout`, the settings store, the theme folder source, the pane layout store, `LunaMenu`, the SVG CSS
  cascade, `TextScroll`. 12 more (923 lines) use only Avalonia's value types or dispatcher: the grid, tile and table
  geometry, the threading primitives (`UiThread`, `Latest<T>`, `Debounce`), `Glide`.
- **Bound to Avalonia: everything else**, about 24,000 lines: the controls (29 files, 4,928 lines), the drawn
  big-screen pieces (22, 4,563), the ES-DE menu look (8, 1,809), `LunaTable<T>` (25, 4,969), tiles, theming's
  templates, the windowing layer with `SheetLayer` and embedded popups (14, 2,066), the fluent builders, automation.
- **The SVG parser and the CSS theme parser's logic are portable**; their output is Avalonia drawing and styles.
- Its own test suite: 96 files, 27,298 lines, 957 tests, 88 of the files on Avalonia's headless platform.

### 2.2 What the frontends take from it, and from Avalonia directly

- **Mistress** names 108 LunaP types (1,421 references in 88 files). The most used are the fluent `Ui` builders (205),
  `FieldRow` (104), `LunaAction` (96), `PadGlyphButton` (80), `MenuRows` (50), `ToolWindow` (42), `SheetLayer` (40).
- **It also bypasses LunaP**: 80 of its 158 files import Avalonia, with raw `TextBlock` 104 times, `Grid` 49,
  `StackPanel` 48, `ScrollViewer` 43, `TextBox` 41, `Dispatcher` 30. **The kit's surface must be complete enough that
  the Rust frontend never reaches past it**: this is the spaghetti `EmuSen_Surfaces.md` §4 measured, in its interface
  form.
- **Hotaru** names 15 LunaP types; Serenity names 12 (for its coretop window); Pharaoh none.

### 2.3 The frontends

- **Mistress**: 29,027 lines; 27 windows plus `MainWindow` (about 6,500 lines with its 31 partials); 17 framed as ES-DE
  menu sheets, 23 driven by the pad in tests. **Big Picture**: 46 files, 7,551 lines, of which 32 files (4,961 lines)
  never name Avalonia: the ES-DE theme loader with its nine-step parsing order, the catalogue of 16 element types and
  464 property declarations, scene mapping, theme management **[M]**.
- **Portable today across the frontends: about 13,000 lines**: Big Picture's theme loader, catalogue and management;
  Scraping (2,778 lines) and Library (1,851); the pad navigator's rules, pad families and hints; the controller diagram
  data; LunaP's 2,001 portable lines; 317 of Mistress's 1,076 tests, which never touch Avalonia.
- **Hotaru**: 1,583 lines. **Pharaoh**: 2,353, headless.
- **The game's picture** is `EmuSen.Serenity`'s `GameFrameControl`: Avalonia, Skia, and the Vulkan RetroArch runtime
  whose output is copied back to the CPU and uploaded again (`EmuSen_Serenity.md` §8.7), 2 to 3 ms a frame at 4K.

### 2.4 The rules the interface is held to, which the kit must keep

These are the real specification of EmuSen's interface, more than any screen's look:

- **The fit audit** (`EmuSen.WiseMan/Fixtures/FitAudit.cs`; `EmuSen_Settings_Reference.md` §4.83.2), at 1280 × 800 and
  1920 × 1200 over 43 named window states: nothing past its panel or its nearest clip, no words past their control,
  no two interactive controls overlapping by more than a pixel, no scroll area under two rows, every slicing edge
  faded, no ellipsis or clipped text, capitals no smaller than Inter's at 16 design pixels.
- **Pad reachability** (`PadAudit.cs`): a walk from every control in every direction reaches every interactive control.
- **The EmulationStation button grammar** (§4.29, §4.45.3): South accepts, East goes back, Start the menu, Select the
  game's options, North a favourite; shoulders page or change tabs; triggers to the ends; repeat after 400 ms then
  every 80 ms; focus moves by position, nearest scroll area first; the closing press is swallowed until every button
  is released; an open keyboard or popup takes every button.
- **Accessibility**: every control named and placed, a field row lending its label, meters as status, the status
  bar a live region, the table answering a screen reader (39 tests).
- **Theming**: 18 colour tokens in dark and light, three font keys, the CSS dialect that refuses a malformed file
  whole, every templated part stylable, and nothing painted outside the palette.
- **ES-DE fidelity** (`EmuSen_BigPicture.md`): the whole theme grammar or none; `fontSize` as the em size handed to the
  rasteriser; fractional placement with no rounding; corner radii as a fraction of the screen's width; ES-DE's
  measured durations; a frame at rest equal to the static render, pixel for pixel; text within 1 to 2 pixels of ES-DE
  3.4.1's captures.

### 2.5 Where it already reaches the platform

X11 only on Linux (Wayland is not supported); gamescope detected from the environment, every window then a sheet
inside one; Linux's `/sys` for battery and radios; the macOS bundle built by `DianaShPublishLayout.targets`; Windows
almost nothing. SDL3, through Endymion, is the one input path on every platform.

---

## 3. The ground it would stand on (read 2026-10-10)

Every claim here is from a source in §12, dated; versions are crates.io's or the projects' own.

### 3.1 Base toolkits, against the requirements

| | iced | libcosmic | Slint | Xilem | Makepad | Qt + cxx-qt |
|---|---|---|---|---|---|---|
| Licence | MIT | MPL-2.0 | GPL-3.0-only option | Apache/MIT | MIT | LGPL/GPL-3 |
| Android | refused | none | yes, with composing IME | stale work in progress | yes | yes |
| iOS | refused | none | yes, on Skia over Metal | no | yes | yes |
| Kiosk on KMS | build it | no | yes, experimental | no | yes | EGLFS; Vulkan KMS experimental |
| Accessibility | none upstream | experimental | desktop; none on Android | AccessKit | a hook | full |
| Gamepad focus | none | none | none (keyboard focus yes) | none | raw input | none |
| API stability | breaks about yearly | follows its fork | 1.x | 0.x | 1.0, churning | stable; cxx-qt 0.x |
| Shipped | Kraken Desktop, COSMIC | COSMIC | embedded HMIs | nothing major | Robrix, five platforms | very many |

### 3.2 The lower layers that exist

| Job | Crate | State |
|---|---|---|
| Windows on desktop, Android, iOS | winit 0.30.13; 0.31 in beta since 2025-11, split per platform | Android and iOS input methods are thin: no composition on Android, no marked text on iOS |
| Wayland directly | smithay-client-toolkit 0.21.1 | under winit and libcosmic |
| GPU | wgpu 30 | Vulkan first-class on Windows, Linux, Android; on Apple through a portability driver; a DRM surface for KMS; breaks quarterly |
| Sharing a device | wgpu-hal | imports a raw `VkInstance`, `VkDevice` and images from ash 0.38, and gives raw handles back |
| Vector drawing | vello_gpu 0.3 (wgpu), vello_cpu 0.3 | vello_cpu "the most mature"; vello_gpu "intended to become the primary renderer" |
| Text | cosmic-text 0.19 (HarfRust, bidi, fallback); Parley 0.12 | cosmic-text's fallback is desktop only; Parley is Linebender's and Bevy's |
| Layout | Taffy 0.14 | flexbox, grid, block |
| Accessibility | AccessKit, adapters for Windows, macOS, AT-SPI, Android and iOS, all 2026-09-25 | "rough feature parity", no rich text |
| SVG | resvg and usvg 0.48 | static SVG only, by design |
| Lottie | velato 0.12 (on Vello) | ES-DE themes use Lottie |
| Video | ffmpeg-next 9, gstreamer-rs 0.25 | no toolkit has a video widget |
| Gamepads | SDL3 3.4 (`sdl3` 0.20), every platform; gilrs has no Android or iOS | SDL3 is already EmuSen's |
| Compositor | Smithay (cosmic-comp, niri) | renderers GLES and pixman only; a Vulkan renderer is an open pull request (2026-09-09) |

### 3.3 Vulkan everywhere: what holds and what does not

- **macOS: KosmicKrisp** is Khronos-conformant Vulkan 1.4 since 2026-09-25, ships in the Vulkan SDK 1.4.363 and in
  Homebrew's Mesa, is MIT, and LunarG advises apps to bundle a tested build and may bundle both it and MoltenVK and
  choose at run time **[S]**. **It needs Apple Silicon and macOS 26**, being built on Metal 4; older Macs and Intel
  ones need MoltenVK **[S]**. Its Metal swapchain offers FIFO and immediate presentation only, **no mailbox** **[S]**.
  What a 2D renderer needs is present: dynamic rendering, descriptor indexing, timeline semaphores, present wait.
  Geometry shaders and transform feedback are not, which concerns GPU cores rather than the kit.
- **iOS: not KosmicKrisp.** It is tested internally only, without staff or CI for it; on iOS, MoltenVK (not fully
  conformant), shipped as a framework, is the only Vulkan **[S]**.
- **Android**: 88.8% of active devices have Vulkan 1.1 or newer and 26.7% have 1.3 (November 2025); Vulkan 1.1 is
  required since Android 15; most devices never get a driver update; implicit layers are forbidden **[S]**.
- **Windows, Linux**: native. **No GPU at all**: Mesa's lavapipe is Vulkan on the CPU, so "Vulkan only" still has a
  software path.
- **Unproven**: wgpu documents its Apple Vulkan path only for MoltenVK; whether it runs on KosmicKrisp is untested by
  anyone found **[S]**. This is K1's first measurement.

### 3.4 What no one has built, and LunaP must

1. **Controller-first focus and navigation** (§2.4's grammar), on every platform.
2. **A KMS backend** that is not experimental, for the kiosk and the OS.
3. **An ES-DE renderer**: no toolkit renders the format, and its SVG, Lottie and video each need a component.
4. **Mobile text input** with composition, over winit's thin methods.
5. **A test harness** that exposes layout rectangles, text measurement and the focus graph to tests, as the fit audit
   and the pad walk need (§7.4).

---

## 4. The kit's design: crates with one job each

### 4.1 Principles

The same as `EmuSen_DianaSh.md` §2 and `EmuSen_Surfaces.md` §6, one level up:

- **One job per crate, and a written surface for it.** A crate's public API is its surface; nothing else crosses. A
  crate that needs another's internals is two crates or one.
- **Layers depend downward only**, and the graph is checked in CI as DianaSh's is: no crate names a type of a layer
  above it, and the kit names no EmuSen type at all. **LunaP knows nothing of emulators**: the test is mechanical, as
  DianaSh's is.
- **Unix where it fits**: an application is a process; its settings and themes are files in XDG directories; on
  Linux, applications and the shell talk Wayland and D-Bus, not a private protocol. A LunaP application in a future
  desktop is an ordinary Wayland client.
- **Platform code sits in backends behind one trait**, never in widgets.
- **The paradigm is chosen, not invented** (§4.2).

### 4.2 The paradigm: declarative views over a retained tree

Decided 2026-10-10 among the paradigms existing toolkits use. The four, and what each would cost this kit:

| Paradigm | Who uses it | For this kit |
|---|---|---|
| **An object tree with inheritance** | GTK, Qt Widgets, WPF, Avalonia (LunaP today) | Rust has no inheritance, and a tree of objects that point at their parents and change each other is what its ownership rules make hardest |
| **The Elm architecture** (one model, messages, a view rebuilt from the model on every message) | iced, libcosmic | sound in Rust, but rebuilding the view costs on long lists and during animation, and a control's identity (which focus, accessibility and an animation in progress all hang on) must be recovered after every rebuild |
| **Immediate mode** (the interface redrawn from code each frame, with no widgets that persist) | egui, Dear ImGui | the simplest, and the weakest at exactly what EmuSen needs: accessibility, input methods, animation, themed scenes |
| **Declarative views over a retained tree, with reactive state** | SwiftUI, Jetpack Compose, QML, Flutter, Slint, Xilem over Masonry | **chosen** |

**What the chosen one is.** An application describes its screens declaratively, in Rust: what is shown, bound to
state. The kit turns the description into a **retained tree** of widgets that persist between frames and carry a
stable identity, which focus, the accessibility tree and animation hang on. State is **fine-grained and reactive**:
a value that changes updates the widgets bound to it and nothing else, so a clock or a frame counter does not rebuild
a screen. An application's own state flows **one way**, as in the Elm architecture, from messages to state to view;
for EmuSen the messages are Crystal's events and the player's input.

**Why it fits.**
- **Rust**: the tree is owned by the kit and the state by the application, with no shared mutable parents.
- **Controller focus and accessibility**: both need persistent widget identity, which the retained tree gives.
- **ES-DE**: a theme is already a declarative description of a scene, so the renderer maps one declarative form
  onto another.
- **Efficiency**: fine-grained updates do work in proportion to what changed, so a steady 60 frames a second beside
  the emulation thread does not depend on how large a screen is.
- **Mobile**: Android's and Apple's own toolkits (Jetpack Compose, SwiftUI) are this paradigm, so the kit's model maps
  onto theirs rather than fighting them.
- **Tests**: a retained tree can be asked for its controls, their rectangles and the focus graph, which is what the
  fit audit and the pad walk need (§7.4).

**What stays a file.** Themes (tokens, the CSS dialect) and ES-DE scenes are text files, read at run time, as the
Unix principle has it. Screens are Rust code, not a markup language: a markup language and its compiler are a large
project of their own, Slint's and QML's, and one can be added later as a module that produces the same declarative
description.

**How it is tested before it is committed to**: K2's prototype of LunaP's core (§9) is built in this paradigm, and
Slint, which uses it too, is the comparison, so the gate measures the paradigm's implementation and not the paradigm.

### 4.3 The layers

```
  applications ── EmuSen's frontend · a future desktop's panels and apps · third parties
        │
  lunap ───────── the umbrella crate: one dependency, one version
        │
  lunap-esde ──── ES-DE theme engine (loader, catalogue, scene, renderer)          [EmuSen's, separable]
  lunap-widgets ─ the controls: buttons, rows, menus, sheets, lists, tables, tiles, carousels
  lunap-theme ─── tokens, palettes, the CSS dialect, theme files
        │
  lunap-core ──── widget tree, reactive state, invalidation, animation clock, popups and sheets
  lunap-focus ─── focus graph, spatial navigation, the button grammar, repeat         [no toolkit has this]
  lunap-a11y ──── the accessibility tree, through AccessKit's five adapters
  lunap-layout ── Taffy, plus fractional placement for ES-DE
  lunap-text ──── shaping, layout, fallback, editing, input methods
  lunap-media ─── images, GIF, SVG (usvg), Lottie (velato), video (ffmpeg)
  lunap-paint ─── the scene: vello_gpu, with vello_cpu as its fallback; image cache
  lunap-gpu ───── one Vulkan device through wgpu; import and export with ash; swapchain and pacing
  lunap-input ─── one event model: keys, pointer, touch, gamepads (SDL3)
  lunap-platform  the backend trait: windows, surfaces, events, clipboard, input methods, system fonts
        ├── lunap-platform-winit ── Windows, macOS, X11 and Wayland desktops, Android, iOS
        ├── lunap-platform-kms ──── the display directly: DRM/KMS, libinput, no compositor
        └── lunap-platform-wayland  smithay-client-toolkit with layer-shell, for a desktop's own panels
  lunap-test ──── headless: render to image, inject input, read rectangles, text and the focus graph
  lunap-comp ──── later: the compositor, on Smithay (§6)
```

### 4.4 Each crate's job, and what it is held to

| Crate | Job | Built on | Held by |
|---|---|---|---|
| `lunap-platform` | the trait every backend implements: create a window or full-screen surface, deliver events, clipboard, show and place the soft keyboard and input method, list system fonts | — | a backend conformance suite every backend runs |
| `lunap-platform-winit` | desktop and mobile windows | winit 0.31 once stable | the suite, on each platform in CI |
| `lunap-platform-kms` | one surface on a connector, mode setting, libinput, VT switching, hot-plug | wgpu's DRM surface; libinput; seatd | the suite, on the handheld and a board outside any compositor |
| `lunap-platform-wayland` | layer-shell surfaces (panels, docks, lock screens) | smithay-client-toolkit | the suite, under cosmic-comp and niri |
| `lunap-input` | one event model; the gamepad as a first-class device with EmuSen's canonical buttons | SDL3 | golden event logs |
| `lunap-gpu` | one Vulkan device; importing a device another component made, or lending its own (§5.2); present and pacing; the output's colour space, HDR metadata and the final encoding (§5.4) | wgpu (Vulkan only), wgpu-hal, ash | device tests on each driver (§5.3); PQ codes read back (§5.4) |
| `lunap-paint` | the vector scene, images, clips, effects (tint, saturation, gradient, corner radius), composited in half-float linear light | vello_gpu; vello_cpu | golden pictures per driver, with tolerances stated per test |
| `lunap-text` | shaping, layout, bidi, fallback per platform, editing, composition | chosen in K1 between cosmic-text and Parley | ES-DE's text captures, to 1 to 2 px |
| `lunap-layout` | flex and grid for application screens; fractional placement for ES-DE scenes | Taffy | layout dumps against today's (§7.3) |
| `lunap-media` | decoding and caching; static SVG; Lottie; video into a GPU image | image, usvg, velato, ffmpeg-next | decode tests; ES-DE comparisons |
| `lunap-a11y` | the accessibility tree from the widget tree | AccessKit | today's 39 accessibility tests, ported |
| `lunap-focus` | the focus graph, movement by position and nearest scroll area, the grammar's bindings and repeat, capture by an open popup | — | the pad walk and the 15 navigation timing tests, ported |
| `lunap-core` | the tree, state and invalidation, animation as a pure function of time, sheets and popups drawn inside one window (as Game Mode needs) | the layers below | the fit audit's rules as tests |
| `lunap-theme` | tokens and palettes; the CSS dialect, ported from LunaP's portable parser | — | palette reach, refusal of a malformed file |
| `lunap-widgets` | every control the frontends use, in the order §9 K4 gives | core, theme | each control's ported tests |
| `lunap-esde` | the ES-DE engine: Big Picture's 4,961 portable lines ported, and a renderer on the kit | widgets, media | ES-DE 3.4.1 captures, as today |
| `lunap-test` | the harness: headless rendering, input injection, rectangles, text overflow, cap heights, the focus graph | core | its own self-tests, as `FitAuditTests` has |

### 4.5 Stability: three kinds of surface

| Surface | Rule |
|---|---|
| **The application API** (`lunap`, `lunap-widgets`, `lunap-theme`, `lunap-core`'s public part) | semantic versioning from 1.0; a baseline of the public API checked in CI (`cargo-semver-checks`), as LunaP's `ApiSurfaceTests` does today; breaking changes only at a major version, recorded |
| **The file formats**: themes, settings, the ES-DE dialect's extensions | versioned in the file; a reader accepts every older version; a newer field is ignored, never an error, except where a format says otherwise |
| **Backends and lower layers** (`lunap-platform` and below) | internal to the kit until 1.0, then the backend trait is versioned so a third party can add a platform |

There is **no C ABI for applications**, by decision of argument: a desktop environment's applications in other
languages talk to it through Wayland and D-Bus, which are already stable and language-neutral, and EmuSen's frontend is
Rust. The two C surfaces that cross a language are EmuSen's, not the kit's: Crystal's frontend API (`EmuSen_CoreAPI.md`
§8.3) and DianaSh's host API (`EmuSen_DianaSh.md` §10). §11 Q8 asks whether the tester wants one anyway.

### 4.6 Where it lives, and its licence

In LunaP's own repository, as a Cargo workspace beside the C# until the C# is retired, keeping **MIT**: a permissive
licence lets the kit serve a desktop environment and third-party applications, and does not stop an App Store build
(§10, R6). EmuSen stays GPL-3.0 and depends on it as it depends on any MIT crate. The crates are published to
crates.io at 1.0, not before.

---

## 5. Rendering: Vulkan alone

### 5.1 Why wgpu restricted to Vulkan, and not a renderer on ash

The tester's choice is Vulkan as the one target. Two ways to honour it: write the kit's renderer on ash directly, or
use wgpu with only its Vulkan backend enabled. **This plan takes wgpu**, for three reasons:

- **Vello, the vector renderer, has no path that avoids wgpu** **[S]**, and writing a vector renderer of one's own is
  the largest single risk a toolkit can take.
- **Zed removed its thin Vulkan layer for wgpu on 2026-02-13**, citing freezes on NVIDIA and under Smithay compositors
  **[S]**: the most recent evidence of what a hand-written UI renderer on Vulkan costs.
- **wgpu-hal can import a raw Vulkan device** and give raw handles back **[S]**, so restricting wgpu costs no control:
  EmuSen's own Vulkan code (the RetroArch runtime on ash, the N64's GPU path) can share the one device.

The cost: wgpu breaks its API about quarterly **[S]**; `lunap-gpu` is the only crate that names it, so a break touches
one crate. And wgpu must work on KosmicKrisp, which is unproven (§3.3): K1 measures it first, and if it fails the
fallback is `lunap-gpu` on ash with vello_cpu uploading its strips, which §11 Q2 names.

### 5.2 One device, and the game's picture never leaving it

Today the game's picture under a RetroArch preset is drawn on Serenity's own Vulkan device, copied to the CPU and
uploaded again by Skia, 2 to 3 ms a frame at 4K (`EmuSen_Serenity.md` §8.7). With the kit on Vulkan, `lunap-gpu`
either makes the device and lends it, or imports Serenity's: the RetroArch runtime renders into an image the kit
then composites, with a semaphore between them and no copy. The same holds for the N64's GPU path and the CRT filter
once it is a Vulkan shader (§8, P4). This is the consolidation the shader question of 2026-10-10 asked for: one
device, one shader language for every filter, Skia gone. ash must be the version wgpu-hal uses (0.38 today), and
queue submissions are synchronised by the application **[S]**.

### 5.3 Drivers, present modes and pacing

- **Per platform**: native Vulkan on Linux, Windows and Android (baseline 1.1 with the Android Vulkan Profile 2022);
  KosmicKrisp on macOS 26 with Apple Silicon; MoltenVK, bundled, on older Macs and on iOS (§11 Q6, Q7).
- **Present modes**: KosmicKrisp's Metal swapchain has no mailbox (§3.3). The kit's pacing is designed for FIFO
  everywhere, which is also what a television without VRR needs (the jitter of 2026-10-10), with immediate as the
  low-latency option where a display can tear.
- **Pacing is the kit's job, not the frontend's**: one clock per surface, from the present timings the driver reports
  (present wait), so EmuSen's "sync to display" (`EmuSen/Common/FrameScheduler.cs`) reads it instead of inferring it.
- **No GPU**: lavapipe, Mesa's Vulkan on the CPU, so a machine without one still runs the same code path.

### 5.4 HDR, accurately

**What accurate means for a console that output SDR.** No retro console made an HDR signal, so the kit does not
invent highlights. Accuracy is two things: an SDR picture shown on an HDR display exactly as it is meant to look, its
white at a stated reference level and nothing crushed or clipped; and, for EmuSen's CRT filter, the headroom a real
tube had. A tube's phosphors were bright where the beam struck and dark between the mask's stripes; on an SDR display
the filter must trade the mask's depth against the picture's brightness (`EmuSen_CRT.md` §12.3), and an HDR display
removes the trade. The filter already computes in absolute light, from its tube's white and the display's brightness
in cd/m², so its arithmetic needs no change, only its output's encoding.

**The presenter's part.** `lunap-gpu` owns the output's colour:

- **A swapchain in the display's colour space**: HDR10 (`VK_COLOR_SPACE_HDR10_ST2084_EXT`, a 10-bit or half-float image,
  PQ-encoded, BT.2020 primaries) or extended linear sRGB (scRGB, half-float) where a platform prefers it, through
  `VK_EXT_swapchain_colorspace`; the content's mastering levels through `VK_EXT_hdr_metadata`; plain sRGB otherwise.
  A surface's colour space is chosen per output and can change when a window moves between displays.
- **Composition in linear light, in absolute units**: `lunap-paint` composites in half-float linear light; SDR content,
  the interface included, is placed at a reference white (§11 Q13), and the CRT filter's output at the luminance it
  computed. The encoding to PQ, scRGB or sRGB happens once, at the end, in `lunap-gpu`.
- **The display's capability**: its peak and its frame-average limit, read from what the platform reports (the EDID's
  HDR static metadata on Linux, the operating system's figures elsewhere) and overridable by a calibration screen
  of the kit's own, since televisions often misreport. Tone mapping above the peak is the kit's, stated and tested, not
  left to the display.
- **Gamut**: BT.2020 output lets a picture show colours sRGB cannot hold. The CRT filter's phosphor primaries are then
  shown as measured, rather than clipped to sRGB as now (`EmuSen_CRT.md` §11.8).

**Its clients.** The CRT filter encodes absolute light, its "Display brightness" setting becoming the measured or
calibrated peak; the RetroArch runtime's HDR presets and options (peak, paper white, gamut expansion) are passed
through; the game's picture under no filter is SDR at the reference white.

**Platforms** **[A]**, each to be confirmed by K1 (g):

| Platform | Route | Open |
|---|---|---|
| Linux, Wayland | the Wayland colour-management protocol, through Mesa's Vulkan WSI, on a compositor that implements it (KDE Plasma, GNOME, recent versions) | which compositors and drivers expose HDR10 to a Vulkan client |
| Linux, gamescope | gamescope's HDR, which needs its WSI layer in the client | whether the handheld's gamescope outputs HDR to the television |
| Linux, KMS and `lunap-comp` | the kit sets the connector's HDR metadata and colour space itself | the kernel's per-driver properties |
| Linux, X11 | none | — |
| Windows | HDR10 or scRGB swapchains | none expected |
| macOS | EDR through Metal | whether KosmicKrisp exposes an HDR colour space at all |
| Android, iOS | device-dependent | per device |

**How it is held.** Tests read the swapchain's image back and compare its codes with the PQ encoding of what the model
computed, to the code value; the tone mapping and the reference white have tests of their own. Whether a display
then shows those codes faithfully cannot be tested from inside the program: it needs a colorimeter on the display
(§11 Q14).

---

## 6. The long road: the compositor and the operating system

### 6.1 What it is

The session of the OS is EmuSen's frontend, full screen. Two ways to put it on the display:

- **Directly** (`lunap-platform-kms`): no compositor; the simplest, and enough for an appliance that runs nothing
  else. It is the kit's first OS target.
- **On a compositor of LunaP's own**, `lunap-comp`, on Smithay: needed as soon as anything else must appear (a browser
  for a login, a second application, overlays, a desktop). It is EmuSen's gamescope: frame pacing, VRR, scaling, HDR,
  direct flipping of a full-screen game's frames. gamescope (C++, on wlroots) is the reference for every one of
  these **[S]**. Smithay has no Vulkan renderer yet (an open pull request, 2026-09-09) **[S]**, so `lunap-comp` either
  waits for it or renders with `lunap-gpu`.

### 6.2 The image

Not a distribution assembled by hand. **Fedora's bootc**: an operating system shipped as an OCI image with its
kernel, updated atomically and rolled back on failure, which Universal Blue rebuilt on in 2025 and Bazzite, a gaming
OS, ships on **[S]**. SteamOS's A/B partitions with dm-verity are the other model **[S]**. bootc is recommended because
EmuSen is developed on Fedora and the image is then a file in the repository, built in CI like any other artifact
(§11 Q10).

### 6.3 What the OS needs that a frontend does not

Screens over system services, each a LunaP screen: networks and Wi-Fi (NetworkManager over D-Bus), Bluetooth pads
(BlueZ), audio devices (PipeWire), power and sleep, storage and the ROM library on removable drives, updates and
rollback (bootc), and a first-run flow. And DianaSh, already designed as a standalone Unix shell, is the OS's shell.

### 6.4 And later, a desktop

The same kit, the same compositor, plus a desktop's own parts: panels and docks as layer-shell surfaces
(`lunap-platform-wayland`), a settings daemon, the XDG portals, a launcher. Nothing in §4 has to change for it; that is
the test of the modular design.

---

## 7. Moving the frontends

### 7.1 The rule: against a specification, never against Avalonia

The Rust frontend is built against two things only: the kit's surface, and Crystal's frontend API (§8). It never
ports Avalonia code line by line; it reproduces **what the interface does**, recorded first.

### 7.2 What is ported as code

The 13,000 portable lines (§2.3), in their order of value: the ES-DE theme loader and catalogue (4,961 lines, already
pure logic), the library and its stores, scraping, the pad navigator's rules, the controller diagrams' data,
LunaP's settings, geometry and CSS parser. Each moves with its tests, which are already free of Avalonia (317 of
Mistress's, and the portable part of LunaP's).

### 7.3 The interface's specification, recorded first

As DianaSh's golden outputs hold its commands (`EmuSen_DianaSh.md` §13.1), a recording of today's interface holds the
frontend's move. Pixel equality between two toolkits is impossible (two rasterisers never agree to the pixel), so
the record is of structure:

- **for each of the 43 audited window states at both sizes, and each Big Picture view**: the tree of controls with
  their kinds, names, rectangles and text, the focus graph the pad walk finds, and which control each press reaches;
- **the rules of §2.4 as executable checks** over that structure, so the Rust frontend is held to the same audit;
- **ES-DE scenes against ES-DE's own captures**, as now: the reference is ES-DE, not Mistress.

A Rust screen passes when its structure matches within stated tolerances (a rectangle within two design pixels, the
same names, the same focus graph) and every rule holds.

### 7.4 The tests

LunaP's 957 tests and Mistress's 1,076 carry over as **assertions**, not code: of Mistress's, 317 are already free of
Avalonia and move as they are; the rest inspect Avalonia's visual tree, and their value is what they assert, which
`lunap-test` must be able to ask (rectangles through every clip, collapsed text, cap heights, the focus graph). The
reflection into `MainWindow`'s privates (106 members, `EmuSen_Surfaces.md` §4.7) is not carried over at all: the Rust
frontend's tests go through its surface.

### 7.5 The order

1. **Big Picture first**: it is pad-driven throughout, themed, its sheets are one window (as Game Mode requires), and
   its portable core is the largest; it is the screen the OS boots into.
2. The game's picture and the in-game menus, on the shared device (§5.2).
3. The framed settings sheets (17), then the desktop library (OpenEmu's sidebar and grid), then the remaining windows.
4. DianaSh's console, through its host API.
5. Hotaru's role (a shell-first frontend) as a mode of the same application; Pharaoh stays headless and needs no kit.

The C# Mistress keeps working throughout; the Rust frontend becomes the default only when every recorded state
passes and the tester has played on it (§9, K9).

---

## 8. What has to happen elsewhere first

These are other tracks of EmuSen's plan; the kit does not wait for them to start (K0 to K5 need none of them), but the
frontend's move does:

| Prerequisite | Why | Where |
|---|---|---|
| **P1. Crystal and its frontend API** | there is no frontend API today; three frontends drive cores three ways (`EmuSen_Surfaces.md` §4.1). The Rust frontend must be its first client, not a fourth copy of the loop | `EmuSen_CoreAPI.md` §8.3, `EmuSen_DianaSh.md` §16 Q1 |
| **P2. Every core in Rust** | a frontend without .NET cannot host C# cores; C# Venus, Mercury and Moon are still in use | the cores' own pages |
| **P3. DianaSh through S6** | the console is a host of the Rust shell | `EmuSen_DianaSh.md` §13 |
| **P4. The built-in filters as Vulkan shaders** | the CRT and LCD filters are SkSL, drawn by Skia; on the kit they become shaders on the shared device, beside the RetroArch runtime | `EmuSen_CRT.md`, `EmuSen_Serenity.md` |
| **P5. The surfaces audit's steps S1 to S4** | the core library off the presentation layer, settings as parameters, no frontend naming a core's internals | `EmuSen_Surfaces.md` §7 |

---

## 9. The order of work, with a gate for each step

No step leaves EmuSen without a working frontend: the C# one runs until K9.

| Step | What | Gate |
|---|---|---|
| **K0** | Record the interface's specification of §7.3 from today's Mistress, in WiseMan | it covers the 43 states at both sizes and every Big Picture view; two recordings identical |
| **K1** | Prove the assumptions, each a small throwaway program, measured: (a) wgpu on KosmicKrisp on the Mac mini, and on MoltenVK; (b) one Vulkan device shared between wgpu and an ash program; (c) a window, text and a gamepad on an Android phone; (d) the same on an iPhone or iPad through MoltenVK; (e) `lunap-platform-kms` on the handheld outside gamescope; (f) cosmic-text against Parley on ES-DE's text captures; (g) which HDR colour spaces each platform offers a Vulkan client (§5.4), on the television the handheld is docked to, under KDE or GNOME on the desktop, and on the Mac | each answered yes or no with its numbers; a no reshapes §5 or §11 before anything is built |
| **K2** | The toolkit gate: one Big Picture screen (the system carousel and a gamelist) built twice, on a prototype of LunaP's core and on Slint, and measured on the desktop, the handheld, the laptop and an ARM board | frame time, memory, binary size, agreement with ES-DE's captures, the pad walk, effort per screen; the tester decides between them on the table (§11 Q1) |
| **K3** | The foundation crates and `lunap-test`, with HDR output in `lunap-gpu` from the first swapchain | each crate's own tests; the backend conformance suite on Linux, Windows and macOS; PQ codes read back to the code value |
| **K4** | `lunap-core`, `lunap-focus`, `lunap-a11y`, `lunap-theme`, and the widgets in order of use (the fluent builders, field rows, actions, pad glyph buttons, menu rows, sheets, dropdowns, switches, tabs, then the table and tiles) | each widget's ported assertions; the fit rules and the pad walk over the kit's own gallery |
| **K5** | `lunap-esde`: the loader and catalogue ported, the renderer on the kit, then video and Lottie | today's ES-DE comparisons, to today's tolerances |
| **K6** | The Rust frontend's Big Picture, on Crystal's frontend API (needs P1) | its recorded states pass; the tester plays it on the handheld |
| **K7** | The rest of the frontend, in §7.5's order (needs P2, P3, P4 as each screen requires) | every recorded state passes on Linux |
| **K8** | Windows, then macOS (KosmicKrisp, MoltenVK below macOS 26), then Android, then iOS | the recorded states on each; on mobile, touch and a pad both; on iOS, interpreters only (§10, R6) |
| **K9** | The Rust frontend the default; the C# frontends, LunaP's C# and Avalonia retired | the tester's word, after a release played; tests moved piece by piece |
| **K10** | The OS: the KMS session, then `lunap-comp`, the bootc image, the system screens of §6.3 | boots on the handheld and an ARM board into Big Picture; updates and rolls back |
| **K11** | A desktop, when the tester designs one | — |

K0 to K5 can start now and run beside the rest of EmuSen's plan; K6 waits for Crystal.

---

## 10. Risks

| | Risk | What tests it, or answers it |
|---|---|---|
| **R1** | wgpu may not run on KosmicKrisp, which no one has documented; KosmicKrisp itself needs macOS 26 on Apple Silicon | K1 (a); MoltenVK bundled as the fallback, as LunarG suggests |
| **R2** | iOS has no KosmicKrisp: Vulkan there is MoltenVK, not fully conformant | K1 (d); the kit's renderer needs only what MoltenVK conforms on |
| **R3** | Android drivers are old and never updated; Vulkan 1.3 is a quarter of devices | baseline Vulkan 1.1 with the 2022 profile; K1 (c) on a low-end phone, not a flagship |
| **R4** | Controller focus, a KMS backend and an ES-DE renderer exist in no toolkit; LunaP owns all three | they are LunaP's reason to exist; K2 builds the first two in prototype |
| **R5** | Mobile text input and font fallback are thin in winit and cosmic-text | K1 (c), (d), (f); `lunap-text` owns platform input methods |
| **R6** | **iOS distribution**: the FSF holds GPL-3 incompatible with the App Store; RetroArch and PPSSPP ship there only with every copyright holder's consent. No JIT on iOS, so the Cranelift recompiler is compiled out and cores need interpreters that keep full speed; Google Play forbids downloaded native code, so cores ship inside the app there | §11 Q5; the kit's MIT licence is not the problem, EmuSen's GPL-3 and its contributors are |
| **R7** | Churn: wgpu quarterly, winit 0.31 in beta for eleven months, Smithay unreleased since 2025-06, ash unreleased since 2024 | each named by one crate only (`lunap-gpu`, the winit backend, `lunap-comp`), so a break touches one crate |
| **R8** | ES-DE fidelity, already the hardest part of Big Picture, must be reached again on a new rasteriser | K5 against the same captures; the tolerance is today's, not a looser one |
| **R9** | Size: this is the largest effort in EmuSen's plan | the gates; K0 to K2 decide before the large spending begins |
| **R10** | HDR depends on what each compositor and driver offers a Vulkan client, and displays misreport their peak | K1 (g); the calibration screen; tone mapping that is the kit's own |

---

## 11. Questions for the tester

Each with a recommendation.

| | Question | Recommended |
|---|---|---|
| **Q1** | LunaP's own core on shared lower layers, or a LunaP layer on Slint? | Decided by K2's table. The plan's argument (§1) favours LunaP's own core; Slint is the fallback if the prototype cannot reach the numbers |
| **Q2** | The renderer: wgpu with only its Vulkan backend, or a renderer on ash? | wgpu, Vulkan only (§5.1), with ash as the fallback if K1 (a) fails |
| **Q3** | Text: cosmic-text or Parley? | Decided by K1 (f): the one that meets ES-DE's 1 to 2 px and has a path to mobile fallback |
| **Q4** | LunaP stays MIT, in its own repository, published at 1.0? | Yes (§4.6) |
| **Q5** | iOS distribution: the App Store, which needs every copyright holder's consent and interpreters only, or sideloading? | Plan for the App Store and keep the option open: record now who holds copyright in every part of EmuSen, and keep every core able to run without its recompiler |
| **Q6** | macOS: KosmicKrisp only (macOS 26, Apple Silicon), or MoltenVK bundled too for older Macs? | Both bundled, chosen at run time, as LunarG advises |
| **Q7** | Android's floor: Vulkan 1.1 with the 2022 profile, devices without Vulkan unsupported (7% of devices)? | Yes |
| **Q8** | A C ABI for the kit, so applications in other languages can use it? | No: Wayland and D-Bus are the language-neutral surface for a desktop's applications (§4.5); revisit if a non-Rust application is ever wanted |
| **Q9** | Where Qt may supplement: anything specific? | Nowhere planned. Keep it as the answer of last resort for one part (an input method, a web view) rather than a dependency of the kit |
| **Q10** | The OS image: Fedora bootc, or SteamOS-style A/B partitions? | Fedora bootc (§6.2) |
| **Q11** | The hardware for K1 and K2: an Android phone, an iPhone or iPad, an ARM board? | The tester's to say; K1 needs one low-end Android phone and one iOS device, and K2 an ARM board |
| **Q12** | The Vulkan SDK on the Mac mini, for K1 (a)? | Yes; it is already on the to-do list as KosmicKrisp's test |
| **Q13** | The reference white for SDR content on an HDR display? | 203 cd/m², BT.2408's, as the default; a setting, since rooms differ |
| **Q14** | A colorimeter, to verify what a display actually shows? | Worth it once HDR output exists; without one, accuracy stops at the codes the program writes |

---

## 12. Sources

Read 2026-10-10; dates are the sources' own.

- iced: https://github.com/iced-rs/iced (0.14.0, 2025-12-07); mobile refused, https://github.com/iced-rs/iced/pull/3099
  (2025-11-29) and https://github.com/iced-rs/iced/pull/3245 (closed 2026-02-15); accessibility,
  https://github.com/iced-rs/iced/pull/3281 (declined 2026-03-14); gamepad, https://github.com/iced-rs/iced/pull/3477.
- libcosmic: https://github.com/pop-os/libcosmic (MPL-2.0); COSMIC Epoch 1.5,
  https://github.com/pop-os/cosmic-epoch/releases/tag/epoch-1.5.0 (2026-07-29).
- Slint: licence https://github.com/slint-ui/slint/blob/master/LICENSE.md; iOS,
  https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/; KMS,
  https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_linuxkms/; gamepad,
  https://github.com/slint-ui/slint/issues/12854; releases https://slint.dev/blog/ (1.18, 2026-09-16).
- Xilem and Masonry: https://linebender.org/blog/tmil-25/ (2026-04-19).
- Makepad: https://github.com/makepad/makepad; Robrix, https://robrix.app/ (1.0.0-beta.2, 2026-10-07).
- Qt: https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html,
  https://doc.qt.io/qt-6/embedded-linux.html; cxx-qt, https://github.com/KDAB/cxx-qt (0.10.0, 2026-08-24).
- winit: https://github.com/rust-windowing/winit/blob/master/FEATURES.md. wgpu: https://github.com/gfx-rs/wgpu (v30).
  Vello: https://github.com/linebender/vello. cosmic-text: https://github.com/pop-os/cosmic-text. AccessKit:
  https://github.com/AccessKit/accesskit. resvg: https://github.com/linebender/resvg.
- SDL: https://github.com/libsdl-org/SDL/releases (3.4.18, 2026-10-02). gilrs: https://gitlab.com/gilrs-project/gilrs.
- Smithay: https://github.com/Smithay/smithay. gamescope: https://github.com/ValveSoftware/gamescope.
- bootc and Bazzite: https://lwn.net/Articles/1042708/, https://github.com/ublue-os/bazzite/issues/3418. SteamOS:
  https://www.collabora.com/news-and-blog/news-and-events/steamos-3-6-how-the-steam-deck-atomic-updates-are-improving.html.
- KosmicKrisp: https://www.lunarg.com/kosmickrisp-achieves-vulkan-1-4-conformance-on-apple-silicon/ (2026-09-25),
  https://www.lunarg.com/the-state-of-vulkan-on-apple-oct-2026/ (2026-10-06),
  https://docs.mesa3d.org/drivers/kosmickrisp.html, https://www.lunarg.com/lunarg-releases-vulkan-sdk-1-4-363-0/.
- Zed's move to wgpu: https://github.com/zed-industries/zed/pull/46758 (merged 2026-02-13).
- Android: https://developer.android.com/about/dashboards; Google's Vulkanised 2026 talk,
  https://vulkan.org/user/pages/09.events/vulkanised-2026/1200-Ian-Elliot-Vulkan%20on%20Android.pptx.pdf; Play's code
  policy, https://support.google.com/googleplay/android-developer/answer/16559646; developer verification,
  https://developer.android.com/developer-verification.
- iOS: https://developer.apple.com/app-store/review/guidelines/ (4.7, 2.5.2); GPL and the App Store,
  https://www.fsf.org/blogs/licensing/more-about-the-app-store-gpl-enforcement,
  https://www.engadget.com/2013-07-18-vlc-media-player-returns-to-the-app-store.html,
  https://www.libretro.com/index.php/retroarch-on-apple-app-store/; JIT,
  https://github.com/StikDebug/StikDebug, https://developer.apple.com/support/alternative-browser-engines.
