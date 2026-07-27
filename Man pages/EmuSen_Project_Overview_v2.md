# EmuSen — Project Overview

*(Living document — companion to `EmuSen_Debugging_Tools_Reference` in this same `Man pages` folder, which covers the debug toolchain in detail. This one covers the emulator itself: what's implemented, what's known-incomplete, and what's next.)*

---

## 1. What this is

EmuSen is a SNES emulator written in C# / .NET 10, structured as four sibling projects:

- **`EmuSen/`** — the emulation core only (CPU/PPU/APU/memory, `Common/`, `DianaOS/`, `Settings/`). A pure library — no `Main`, no window ownership of its own. This is the primary development/debugging environment — cores get built and verified here first, standalone, before either frontend is the main way of running them. It still has a `Raylib-cs` package dependency, because `Renderer.cs` uses `Raylib_cs.Color`/`Image`/`Texture2D` directly as its internal pixel representation and for debug-panel drawing - a real dependency of the core's own rendering logic, not something inherited from a frontend.
- **`EmuSen.Hotaru/`** — the console-first frontend's actual driver (`Program.cs`'s `Main`, `App.axaml.cs`, `Views/GameWindow.axaml.cs`). Used to live inside `EmuSen.csproj` as its own `Exe`, which meant `EmuSen.Mistress9/` (Avalonia) inherited this project's `Main`/window-ownership just by referencing `EmuSen.csproj` to reach the emulation core - backwards, since the two frontends have nothing to do with each other. Split out specifically to fix that: both frontends are now true siblings, each independently referencing `EmuSen.csproj` for the core and `EmuSen.Serenity` for presentation. Moved off Raylib entirely onto Avalonia (window, rendering, shaders, audio, input) - see `EmuSen_Frontend_Driver.md`'s own top-of-file revision note. (Named `Hotaru`, not `Console` - naming it `EmuSen.Console` would make every bare `Console.WriteLine` in the file ambiguous with the namespace itself, a real C# gotcha, not just a style choice.)
- **`EmuSen.Serenity/`** — shared presentation-layer code, split out so both frontends depend on the same implementation instead of a hand-copied duplicate: `GameFrameControl` (an Avalonia `Control` presenting any core via `ICore.GetFrameBufferRgba()` straight through Skia, with real shader support), `FramePresenter` (a plain `Window` + `GameFrameControl` bundle for a consumer that needs nothing else from its window), `Shaders/BuiltInShaders.cs` (Skia SkSL for the shader pass - see `EmuSen_Frontend_Driver.md` §2's F8 entry), and `GraphicsSettings.cs` (moved out of `EmuSen/Settings/` for the same reason). Both `EmuSen.Hotaru/` and `EmuSen.Mistress9/` are real consumers now - Hotaru uses `GameFrameControl` directly rather than `FramePresenter` (see that project's own `GameWindow.axaml.cs` header comment for why); Mistress9 adopted the same `GameFrameControl` directly too, replacing its old CPU-side `WriteableBitmap` presentation (`MainWindow.axaml`/`.cs`) - the long-standing "no GPU/shader hook of its own" TODO this project existed to answer is closed on both frontends now. See §2a for which actual GPU API `GrContext` resolves to underneath that shared control, per platform.
- **`EmuSen.Mistress9/`** — an Avalonia GUI, renamed from `EmuSen.Frontend` and deliberately scoped as bug-testing tooling: ROM picker (plus a ROM browser and a configurable default ROM directory), rebindable keyboard+gamepad input, Save/Load State menu items, and (Settings > Preferences...) a configurable log directory that reuses `EmuSen.Common.CategorizedLogWriter` for real per-session file logging, plus a scaffolding-only core picker. Referencing `EmuSen` and `EmuSen.Serenity` via project reference (as a sibling of `EmuSen.Hotaru`, not through it). Deliberately **not** where the eventual EmulationStation-style launcher gets built — see `EmuSen_Launcher_Multicore_Gameplan.md` for that separate, not-yet-created project's plan.

**License:** GNU GPL-3.0 (see `LICENSE` at the repo root) — chosen deliberately over a permissive license (MIT) specifically so anything built on EmuSen's code stays open forever, matching how this project itself was built entirely from openly-provided documentation. Not chosen for commercial reasons; the point is guaranteeing downstream openness, not restricting use.

**Licensing stance on other emulators' source:** the SNESdev wiki (mirrored at both `snes.nesdev.org` and `snesdev.mesen.ca` — maintained by the Mesen/MesenCE team) is the primary hardware-reference source. Historically, Mesen/MesenCE's own *documentation* got read to understand hardware behavior while their source code did not, specifically to keep every implementation here original and avoid any GPL-compatibility question. Now that EmuSen itself is GPL-3.0 - the same license `SourMesen/Mesen2` uses - that compatibility concern no longer applies, and Mesen2's source can be consulted directly as an architecture/implementation reference (see `EmuSen_Games_Tested.md`-adjacent investigation notes for anything drawn from it). `SourMesen/Mesen2` is archived; `nesdev-org/MesenCE` is the actively maintained continuation - forked for this project's own reference use at [`RedQuE3n/MesenCE-refernece`](https://github.com/RedQuE3n/MesenCE-refernece) (note the upstream typo preserved in the fork's own name), with its SNES core under `Core/SNES`. Still worth writing original code rather than transcribing verbatim where reasonable - referencing for correctness/design, not copy-pasting - but the hard legal barrier that used to rule out even looking is gone.

**Multi-core intent:** the project is meant to eventually support more than one console. `Cores/Nintendo/Venus - SNES/` (the SNES core) is folder-scoped and namespaced for this — `EmuSen.Cores.Nintendo.Venus.Memory`, not a generic `EmuSen.Memory` — with reserved sibling folders (placeholder `README.md` only, no code) for every other planned core across five manufacturers: `Cores/Nintendo/` (the rest of the Sailor Guardians), `Cores/Sega/` (Dark Kingdom), `Cores/Sony/` (Black Moon Clan), `Cores/Atari/` (Death Busters), `Cores/Microsoft/` and `Cores/NEC/` (Dead Moon Circus's two subordinate groups). Full mapping in `EmuSen_Core_Naming_Scheme.md`. The debug toolchain (`DianaOS/IDebugTarget.cs` and everything built on it) was deliberately designed core-agnostic from day one for the same reason — see the debugging tools reference doc.

**Naming scheme:** every core is named after an *EmuSen* = **Emu**lator **Sen**shi (Sailor Moon) character, grouped by manufacturer — Nintendo gets the heroes (Sailor Guardians), every other manufacturer gets a villain faction (Sega = Dark Kingdom, Sony = Black Moon Clan, Atari = Death Busters, Microsoft/NEC = Dead Moon Circus's two subgroups). Full mapping and rationale in `EmuSen_Core_Naming_Scheme.md`. The SNES core is `Venus`; still called "SNES" in prose/comments/log output/class names (`SnesDebugTarget`, `Snes65816Disassembler`, etc.) since that's the accurate hardware name — the codename governs the folder/namespace only, not every mention of the actual console.

---

## 1a. Development environment

The machine this project is actually built and run on, kept current here so anything environment-specific (paths, package manager, GPU driver quirks) can be cross-checked against real conditions rather than assumed:

- **OS:** Fedora Linux 44, KDE Plasma Desktop Edition (Wayland)
- **Kernel:** 7.1.4-200.fc44.x86_64
- **KDE Plasma:** 6.7.3 / **KDE Frameworks:** 6.28.0 / **Qt:** 6.11.1
- **CPU:** AMD Ryzen 7 7700X (8-core / 16-thread)
- **RAM:** 32 GiB
- **GPU:** AMD Radeon RX 6800 (discrete) + Ryzen 7700X integrated graphics
- **Motherboard/system manufacturer:** ASUS

Package manager is `dnf` (Fedora), not `apt`/`apt-get` — relevant for any install instructions given for this project (e.g. `ffmpeg` for the frame-recording tool's video encode step — see the debugging tools reference, §3.8 — is `sudo dnf install ffmpeg` here, not the Debian/Ubuntu-style command).

---

## 2. Architecture

**Layering, roughly bottom-to-top:**

- **Hardware simulation** (`Cores/Nintendo/Venus - SNES/Cpu`, `Cores/Nintendo/Venus - SNES/Apu`, `Cores/Nintendo/Venus - SNES/Ppu`, `Cores/Nintendo/Venus - SNES/Memory`) — the actual 65816/SPC700/S-DSP/PPU/memory-map implementations. This layer knows nothing about debugging, rendering presentation, or frontends; it just simulates hardware, one register/opcode/pixel at a time.
- **`Common/`** — cross-cutting, not console-specific: `EmulatorSession` (a headless per-frame driver used by `EmuSen.Mistress9`), `StateSerializer` (reflective save states), `TeeTextWriter` (generic console+single-file tee) and `CategorizedLogWriter` (used by both `EmuSen.Hotaru`'s `Program.cs` and `EmuSen.Mistress9`'s `MainWindow` — routes console output into per-category files under a `Logs/<CoreName>/<console|gui>_<timestamp>/` session folder instead of one ever-growing combined log, nested by core so a future second core's logs never mix with this one's; file writes run on a background thread now, not the caller's).
- **`Settings/`** — global configuration, most notably `DebugSettings` (the logging-toggle registry) and input/graphics/audio settings. Intentionally simple global static state for the debug toggles specifically — a conscious tradeoff (see §5) rather than an oversight. See `EmuSen_Settings_Reference.md` for what every flag/setting does and, for the debug toggles, the investigation each one was originally added for.
- **The debug toolchain** (`DianaOS/`, plus `Cores/Nintendo/Venus - SNES/Debug/`) — a deliberately separate, core-agnostic layer sitting *beside* the hardware simulation, not inside it. `DianaOS/IDebugTarget.cs` defines a contract any core can implement; `Cores/Nintendo/Venus - SNES/Debug/SnesDebugTarget.cs` is the SNES implementation; `DianaOS/DianaOSInterpreter.cs` is a genuine bash-alike shell, user-facing name **DianaOS** (quoting, variables, `$(...)`, pipes, redirection, if/for/while, plus a coreutils subset - `ls`/`cd`/`grep`/`awk`/`nano`/`coretop`/...) built on top of that, with the SNES-facing commands themselves (`mem`/`regs`/`watch`/...) merged into `DianaOS/Commands/` alongside the shell's own builtins — no more `EmuSen.Debug.Commands` split. See the companion `EmuSen_Debugging_Tools_Reference` doc for the full breakdown.
- **Presentation** (`EmuSen.Serenity/`) — window/GPU-surface ownership and the shader pipeline (Avalonia + Skia, not Raylib), shared between frontends via `ICore.GetFrameBufferRgba()` rather than duplicated per frontend. Split out from the console frontend once it stopped being small - see `EmuSen_Frontend_Driver.md` §1.
- **Frontends** (`EmuSen.Hotaru/`, `EmuSen.Mistress9/`) — presentation *driving*, not the presentation code itself anymore. Both are now Avalonia frontends into the same core (Hotaru moved off Raylib entirely - see `EmuSen_Frontend_Driver.md`'s own revision note); neither owns emulation logic itself, and neither depends on the other.

**A concrete example of the layering working as intended:** `MemoryBus` (hardware simulation) exposes a tiny, debug-agnostic `IWriteObserver` hook that it calls on every WRAM write, with no idea what's listening. `SnesDebugTarget` (debug toolchain) implements that interface and is the thing that actually knows what a "watchpoint" is. `MemoryBus` could be reused by a completely different debug story (or none at all) without any changes — the coupling only exists in one direction, and it's the debug layer depending on the core, not the reverse. This wasn't always true — a `WatchRegistry` field and a `Cpu` back-reference used to live directly on `MemoryBus` itself, mixing the two layers together, until a later architecture-focused pass (see §4 and §7) untangled it.

**Multi-core intent:** `Cores/Nintendo/Venus - SNES/` (SNES) sits alongside reserved sibling folders for every other planned core (see §3's directory map, and `EmuSen_Core_Naming_Scheme.md` for the full manufacturer/codename mapping). The C# namespaces now match the folder structure (`EmuSen.Cores.Nintendo.Venus.*`, not a generic `EmuSen.*`) — the drive-by rename that used to be a §7 TODO item is done. The debug toolchain's core-agnostic design (`IDebugTarget` et al.) is the one piece of this multi-core intent that's actually been proven under real use, rather than just structural.

---

## 2a. Rendering backend selection — what GPU API `GrContext` actually is, per platform

`GameFrameControl` (§2's `EmuSen.Serenity/` entry) only ever talks to Skia's own backend-agnostic `GRContext`/`SKSurface` types - it has no OpenGL/Vulkan/Metal-specific code anywhere in it. Which real GPU API sits underneath that abstraction is decided entirely by Avalonia's platform backend, configured in `Program.cs` (`BuildAvaloniaApp`, both frontends - `EmuSen.Hotaru/Program.cs` and `EmuSen.Mistress9/Program.cs` use the identical idiom), not by anything in `EmuSen.Serenity` itself:

- **Windows** — OpenGL via ANGLE, which translates those calls to Direct3D 11 underneath. No native Direct3D Skia backend is used by default.
- **macOS** — Metal, Avalonia's own default there. Vulkan-over-Metal (MoltenVK) isn't wired into Avalonia's GPU-interop layer for macOS at all, so this isn't a live option.
- **Linux — Wayland (`.UseWayland()`, both `Program.cs` files).** Briefly tried X11's default detection instead, on the theory that X11 is the more mature/Vulkan-capable backend (`Avalonia.Wayland` 12.1.0 has no Vulkan support at all; `Avalonia.X11` 12.1.0 does). That looked like it regressed to a black game window on both frontends - but see the bug story below: it wasn't actually an X11-vs-Wayland issue at all, so this reverted to `.UseWayland()` more by coincidence-of-timing than because Wayland was ever the real fix. Left on Wayland since it's the configuration this was actually last verified against.
- **Vulkan isn't enabled anywhere** - would need the X11 backend regardless (Wayland has no Vulkan support in this Avalonia version), and hasn't been revisited since the bug below turned out to be unrelated to the windowing backend entirely.

### The real bug: `GameFrameControl` was allocating a fresh GPU surface every frame

What looked like an X11/Wayland windowing problem (`EmuSen.Hotaru`'s game window either solid black or visibly flickering the instant a ROM's frames started flowing, reproduced on **both** backends) turned out to have nothing to do with the windowing backend at all. Root-caused via `EmuSen.Pharaoh90` (the headless CLI harness, `Man pages/EmuSen_Debugging_Tools_Reference_v5.md` §3.15) run with `--autoshot`: 300 frames of a test ROM, dumping a BMP straight from `VenusCore.GetFrameBufferRgba()` every time the frame hash changed, entirely bypassing Avalonia - the core's own video output was completely clean and stable. That isolated the bug to `GameFrameControl.cs`'s presentation path specifically.

The actual cause: `DrawOp.Render()` was allocating a brand-new GPU-backed `SKSurface` (the offscreen upscale target) via `SKSurface.Create(lease.GrContext, ...)` **every single frame**, drawing into it, snapshotting it, then disposing it - 60 times a second. That much per-frame GPU render-target churn was the source of the black-screen/flicker symptom on both windowing backends. Fixed by skipping that offscreen surface entirely when no shader is active (the default/common case): `DrawImage` now scales `sourceImage` straight to the destination rect on the real canvas, no intermediate GPU surface at all.

The shader-active path (F8 - Scanlines/Crt) originally kept the old per-frame-offscreen-surface pattern the no-shader path above was fixed to avoid, since a shader needs to sample the already-upscaled image in real output-pixel space. Fixing that (below) is what surfaced a second, separate, more serious bug in the same code path.

### A second, separate bug found by the new headless render tests: disposing `SKRuntimeShaderBuilder` crashes the process

`EmuSen.WiseMan/Serenity/GameFrameControlRenderTests.cs` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.18) was added specifically to close the "nothing exercises the real render path" gap - and its very first real exercise of the shader path (rendering more than one frame with Scanlines/Crt active) segfaulted the whole test host. `--blame-crash` + `gdb` on the resulting core dump pointed at `SkRuntimeEffect::uniformSize()` inside `libSkiaSharp.so`, crashing only on the **second** render with a shader, never the first.

Root-caused in complete isolation from Avalonia (a bare console app, no windowing at all): `GameFrameControl.GetEffect()` compiles and caches one `SKRuntimeEffect` per shader, reused across every frame (correct - SkSL compilation is expensive, and the effect itself is frame-invariant, per that method's own comment). `DrawOp.Render()` built a fresh `SKRuntimeShaderBuilder` from that cached effect every frame and disposed it (`using`) once done. That disposal is the bug: **`SKRuntimeShaderBuilder.Dispose()` in SkiaSharp 3.119.4 corrupts the `SKRuntimeEffect` it was built from**, not just its own builder state - so the *next* frame's builder, constructed from the now-corrupted cached effect, segfaults the instant it touches a uniform.

**This means every real session that pressed F8 to select Scanlines or CRT and then let even one more frame render (which happens automatically during gameplay, ~60 times a second) would have crashed the whole app** - a real, previously-undiscovered, 100%-reproducible crash, not a hypothetical. Nothing before the headless render tests ever exercised the shader path more than once in a row, which is exactly why this went unnoticed through the original Raylib→Avalonia rewrite and every session since.

### The actual foundation fix, not just a workaround

Simply not disposing the builder (the first fix) made the crash go away, but left the real design flaw in place: a fresh native object allocated in the hot render path every single frame, forever, relying entirely on the GC to eventually reclaim it - the exact same shape of problem as the offscreen-surface flicker above, just for a different object. Both bugs trace back to one root cause: **`DrawOp.Render()` allocated GPU-adjacent native objects fresh every frame instead of building anything once and reusing it.**

The completed fix, done together rather than patched separately:
- **`GameFrameControl.GetBuilder(effect)`** (new, alongside `GetEffect`) caches one `SKRuntimeShaderBuilder` per effect and reuses it for the control's whole lifetime, exactly like the compiled effect already was - never disposed, never reconstructed, only its `Children`/`Uniforms` entries rebound each frame. This is what actually eliminates the per-frame allocation, not just the crash from disposing it.
- **The offscreen upscale `SKSurface` is gone from the shader path too** - `DrawOp.Render()` now builds the shader's `"image"` child directly from `sourceImage.ToShader(..., SKMatrix.CreateScale(scaleX, scaleY))`, a local matrix that maps the shader's `coord` (already in destination-pixel space, per `BuiltInShaders.cs`'s own comment) straight back onto the native-resolution source image - no intermediate GPU render target at all, for either the shader or no-shader path now.

Verified two ways: `GameFrameControlRenderTests.cs`'s shader tests (byte-identical repeated output, 120-consecutive-frames-no-throw, stress-tested manually to 2000 iterations with no degradation) all still pass, and a new `Scanlines_darkens_odd_output_rows_by_the_exact_documented_factor` test checks the *exact* pixel math (a solid-color frame's even output rows stay full brightness, odd rows darken by precisely the documented 0.78 factor) - proving the local-matrix rewrite is correct, not just "produces some different output than before."

---

## 3. Directory map

```
EmuSen Project/
├── EmuSen/                                   # Core emulator library only (no frontend lives here)
│   ├── EmuSen.csproj
│   ├── Common/
│   │   ├── EmulatorSession.cs                # Headless per-frame driver (used by Avalonia frontend) -
│   │   │                                      #   also passes through ICore.AudioSampleRate/
│   │   │                                      #   DequeueAudioSamples now (see EmuSen_Frontend_Driver.md
│   │   │                                      #   §3), null-safe (empty array / 32000 fallback) before
│   │   │                                      #   LoadRom() unlike this class's throwing GetFrameBufferRgba
│   │   ├── StateSerializer.cs                # Reflective save-state serializer
│   │   ├── TeeTextWriter.cs                  # Console + single-file log tee (generic, reusable)
│   │   ├── Imaging/                          # Moved out of EmuSen.Pharaoh90/Program.cs (was ~270
│   │   │   │                                  #   lines of private static helpers with zero test
│   │   │   │                                  #   coverage and zero reuse anywhere else) - see
│   │   │   │                                  #   EmuSen_Debugging_Tools_Reference_v5.md §3.15's revision
│   │   │   │                                  #   note. Genuinely general-purpose, not Pharaoh90-
│   │   │   │                                  #   specific, hence living here rather than in that project.
│   │   │   ├── BmpFile.cs                    # Minimal uncompressed 32bpp BMP write/read
│   │   │   ├── FrameHash.cs                  # 64-bit FNV-1a over raw RGBA bytes (--autoshot's change check)
│   │   │   ├── ContactSheet.cs               # Nearest-neighbor downsample + grid tiling
│   │   │   └── SpriteOverlay.cs              # Green bounding-box outline from IDebugTarget.DebugSpriteInfo
│   │   └── CategorizedLogWriter.cs           # Console + per-category log files (cpu/ppu/apu/memory/
│   │                                          #   debug/general) - used by both EmuSen.Hotaru's
│   │                                          #   Program.cs and EmuSen.Mistress9's MainWindow. File
│   │                                          #   writes happen on a dedicated background thread (a
│   │                                          #   bounded queue, single consumer) now, not the calling
│   │                                          #   thread - see the class's own comment for why, and why
│   │                                          #   callers now have to Dispose() it explicitly on every
│   │                                          #   exit path instead of relying on AutoFlush. Also
│   │                                          #   flushes every category to disk on a fixed 500ms
│   │                                          #   timer (not just when the shared queue happens to go
│   │                                          #   idle - a real session showed high-volume categories
│   │                                          #   can keep it busy continuously, starving low-volume
│   │                                          #   ones of any flush at all), so a hard kill or crash
│   │                                          #   that skips Dispose() only loses a fraction of a
│   │                                          #   second of output, for every category alike.
│   │                                          #   Dispose() only closes the per-category files once
│   │                                          #   Join() confirms the worker thread actually drained
│   │                                          #   and exited - closing them on a Join timeout used to
│   │                                          #   race the worker mid-write, which is what produced
│   │                                          #   truncated last lines even on a clean window-close.
│   │                                          #   A timeout now just leaves the handles open (and
│   │                                          #   prints a console warning) instead of risking that.
│   │                                          #   Console echo also moved off the calling thread onto
│   │                                          #   the same queue as file writes - it used to stay
│   │                                          #   synchronous on the assumption it was cheap, which
│   │                                          #   broke down hard once CpuVerboseLogging/
│   │                                          #   Spc700VerboseLogging got left on for a whole session
│   │                                          #   (one WriteLine per instruction executed) and FPS
│   │                                          #   collapsed. Queue capacity raised to 100k accordingly
│   │                                          #   (each line now enqueues two entries, file + echo).
│   │                                          #   cpu/apu specifically skip the console-echo entry
│   │                                          #   entirely (file-only) - moving echo to the background
│   │                                          #   thread fixed the emulation thread blocking on a slow
│   │                                          #   console write, but a real session then froze solid,
│   │                                          #   because the terminal itself can't render a
│   │                                          #   million-lines/sec instruction trace on any thread,
│   │                                          #   so the shared queue filled and blocked anyway.
│   ├── Cores/
│   │   ├── Nintendo/
│   │   │   ├── Venus - SNES/                 # Namespace stays plain "Venus" (C# identifiers can't
│   │   │   │   │                              #   contain spaces/dashes) - EmuSen.Cores.Nintendo.
│   │   │   │   │                              #   Venus.Memory, not the old generic EmuSen.Memory.
│   │   │   │   │                              #   See EmuSen_Core_Naming_Scheme.md for the full scheme.
│   │   │   │   ├── Cpu/
│   │   │   │   │   ├── Core/Cpu.cs           # 65816 execution core
│   │   │   │   │   ├── Opcodes/
│   │   │   │   │   │   ├── Cpu.AddressModes.cs
│   │   │   │   │   │   ├── Cpu.OpcodeTable.cs         # 256-entry dispatch table - kept as ONE file
│   │   │   │   │   │   │                              #   on purpose, see Venus_CPU.md §1
│   │   │   │   │   │   └── Cpu.Opcodes.{System,Stack,LoadStoreTransfer,
│   │   │   │   │   │       Arithmetic,Logical,Shift,Branch,Flags}.cs
│   │   │   │   │   │                                  # Op* bodies, split by instruction category -
│   │   │   │   │   │                                  #   was one 1229-line Cpu.Opcodes.cs
│   │   │   │   │   └── Disassembler/
│   │   │   │   │       └── Snes65816Disassembler.cs  # Separate, read-only mnemonic table - NOT
│   │   │   │   │                                       #   built from the execution opcode table
│   │   │   │   │                                       #   (see §4's companion doc reference)
│   │   │   │   ├── Apu/
│   │   │   │   │   ├── Dsp/
│   │   │   │   │   │   ├── BrrDecoder.cs
│   │   │   │   │   │   ├── DspVoice.cs
│   │   │   │   │   │   └── SDsp.cs           # Real audio synthesis; not connected to output
│   │   │   │   │   └── Spc700/
│   │   │   │   │       ├── Core/Spc700.cs
│   │   │   │   │       └── Opcodes/
│   │   │   │   │           ├── Spc700.AddressModes.cs
│   │   │   │   │           ├── Spc700.OpcodeTable.cs
│   │   │   │   │           └── Spc700.Opcodes.cs
│   │   │   │   ├── Memory/
│   │   │   │   │   ├── Cartridge.cs          # ROM/SRAM load+save, header-driven SRAM sizing
│   │   │   │   │   ├── Dma.cs
│   │   │   │   │   ├── MathUnit.cs           # Hardware multiply/divide - extracted out of MemoryBus
│   │   │   │   │   ├── IWriteObserver.cs     # Debug-agnostic write-observer hook (see §2)
│   │   │   │   │   └── MemoryBus.cs          # Address decode/dispatch - the bus's actual job only
│   │   │   │   ├── Ppu/
│   │   │   │   │   ├── Core/
│   │   │   │   │   │   ├── Ppu.cs
│   │   │   │   │   │   ├── Ppu.Registers.cs
│   │   │   │   │   │   └── Ppu.RegisterTable.cs
│   │   │   │   │   └── Renderer/
│   │   │   │   │       ├── Renderer.cs
│   │   │   │   │       ├── Renderer.Backgrounds.cs   # All 7 BG modes
│   │   │   │   │       ├── Renderer.Sprites.cs
│   │   │   │   │       ├── Renderer.Mode7.cs         # Matrix transform + EXTBG
│   │   │   │   │       ├── Renderer.Scanline.cs      # Compositing/priority order per mode
│   │   │   │   │       └── Renderer.Debug.cs         # VRAM sheet, OAM dump, black-tile diagnostic
│   │   │   │   ├── Debug/
│   │   │   │   │   ├── StateDump.cs          # Pre-toolchain CPU+PPU snapshot formatter
│   │   │   │   │   └── SnesDebugTarget.cs    # SNES's IDebugTarget implementation (namespace
│   │   │   │   │                              #   EmuSen.Cores.Nintendo.Venus.Debug)
│   │   │   │   ├── Cheats/                   # ActionReplayCodec/GameGenieCodec - SNES-specific
│   │   │   │   │                              #   cheat-code decoders (namespace EmuSen.Cores.
│   │   │   │   │                              #   Nintendo.Venus.Cheats), moved here from DianaOS/
│   │   │   │   │                              #   Cheats/ during a core-agnosticism pass - the
│   │   │   │   │                              #   generic RAM-poke/ROM-patch mechanism they feed
│   │   │   │   │                              #   (CheatRegistry) stayed in DianaOS/, only the SNES
│   │   │   │   │                              #   Game Genie/Pro Action Replay code-string
│   │   │   │   │                              #   decoding moved.
│   │   │   │   └── Input/
│   │   │   │       └── Input.cs
│   │   │   ├── Moon - NES/README.md          # Reserved - future NES core
│   │   │   ├── Mercury - GB-GBC/README.md    # Reserved - future GB/GBC core
│   │   │   ├── Jupiter - GBA/README.md       # Reserved - future GBA core
│   │   │   ├── Mars - N64/README.md          # Reserved - future N64 core
│   │   │   ├── Saturn - Virtual Boy/README.md   # Reserved - future Virtual Boy core (Outer Senshi)
│   │   │   ├── Uranus - GameCube/README.md      # Reserved - future GameCube core (Outer Senshi)
│   │   │   ├── Neptune - Wii/README.md          # Reserved - future Wii core (Outer Senshi)
│   │   │   ├── Pluto - Wii U/README.md          # Reserved - future Wii U core (Outer Senshi)
│   │   │   ├── Luna - DS/README.md              # Reserved - future DS core (guardian cat)
│   │   │   └── Artemis - 3DS-New3DS/README.md   # Reserved - future 3DS/New 3DS core (guardian cat)
│   │   ├── Sega/                             # Dark Kingdom / Shitennou codenames
│   │   │   ├── Endymion - Master System/README.md  # Reserved - future Master System core
│   │   │   ├── Beryl - Genesis/README.md     # Reserved - future Genesis/Mega Drive core
│   │   │   ├── Jadeite - Game Gear/README.md # Reserved - future Game Gear core
│   │   │   ├── Nephrite - 32X/README.md      # Reserved - future 32X core
│   │   │   ├── Zoisite - Saturn/README.md    # Reserved - future Saturn core
│   │   │   └── Kunzite - Dreamcast/README.md # Reserved - future Dreamcast core
│   │   ├── Sony/                             # Black Moon Clan codenames
│   │   │   ├── Diamond - PlayStation/README.md
│   │   │   ├── Sapphire - PlayStation 2/README.md
│   │   │   ├── Rubeus - PlayStation 3/README.md
│   │   │   ├── Esmeraude - PSP/README.md
│   │   │   └── Wiseman - PS Vita/README.md
│   │   ├── Atari/                            # Death Busters (Witches 5) codenames
│   │   │   ├── Eudial - Atari 2600/README.md
│   │   │   ├── Mimete - Atari 5200/README.md
│   │   │   ├── Tellu - Atari 7800/README.md
│   │   │   ├── Viluy - Atari Lynx/README.md
│   │   │   └── Cyprine & Ptilol - Atari Jaguar/README.md
│   │   ├── Microsoft/                        # Dead Moon Circus (Amazoness Quartet) codenames
│   │   │   ├── CereCere - Xbox/README.md
│   │   │   ├── JunJun - Xbox 360/README.md
│   │   │   ├── PallaPalla - Xbox One/README.md
│   │   │   └── VesVes - Xbox Series/README.md
│   │   └── NEC/                              # Dead Moon Circus (Amazon Trio) codenames
│   │       ├── Tigers Eye - PC Engine/README.md
│   │       ├── Hawks Eye - PC-FX/README.md
│   │       └── Fish Eye - SuperGrafx/README.md
│   ├── DianaOS/                               # General-purpose bash-alike shell, user-facing name DianaOS
│   │   │                                      #   (see companion doc §3.3/§3.17) AND the core-agnostic debug
│   │   │                                      #   toolchain it was originally split from - was Debug/
│   │   │                                      #   DebugCommandProcessor.cs; renamed/moved once it grew quoting,
│   │   │                                      #   variables, $(...), pipes, redirection, if/for/while.
│   │   │                                      #   EmuSen.Debug.Commands/Debug/ (IDebugTarget, every registry,
│   │   │                                      #   DebugTools, FrameRecorder, and the SNES-facing commands
│   │   │                                      #   themselves - mem/regs/watch/...) merged in here later, once
│   │   │                                      #   "registered into the shell rather than merged away" turned
│   │   │                                      #   out to serve no purpose - see the companion doc's §3.3.
│   │   │                                      #   Folder/namespace/every "Shell"-prefixed type fully renamed to
│   │   │                                      #   DianaOS in a later pass (Shell/ -> DianaOS/, ShellInterpreter
│   │   │                                      #   -> DianaOSInterpreter, IShellCommand -> IDianaOSCommand,
│   │   │                                      #   ShellResult -> DianaOSResult) - see the companion doc's §3.3.
│   │   ├── DianaOSInterpreter.cs               # dispatcher + variable/history state + statement execution
│   │   ├── Lexer.cs / Parser.cs / Ast.cs       # hand-written tokenizer/recursive-descent parser/AST
│   │   ├── ConsoleLineReader.cs                # up/down-arrow history recall at the F4 prompt
│   │   ├── IDebugTarget.cs                     # core-agnostic debug-target contract (see companion doc)
│   │   ├── DianaOSSandbox.cs                   # walls cd/ls/mv/redirection/source to the project's own tree
│   │   ├── ManPages.cs                         # per-command manual pages backing `man <command>`
│   │   ├── WatchRegistry.cs / BreakpointRegistry.cs / CheatRegistry.cs / FrameLogRegistry.cs
│   │   ├── FrameRecorder.cs
│   │   ├── DebugTools.cs                     # BoundedTrace ("run N more frames/steps then stop") plus
│   │   │                                      #   RepeatCollapsingTrace<TKey> - collapses a repeating
│   │   │                                      #   1-8 instruction cycle (polling/delay loops - VBlank
│   │   │                                      #   wait, DMA busy-wait, the APU handshake) into one
│   │   │                                      #   summary line instead of writing every repeat, since
│   │   │                                      #   a real session produced a 1GB cpu.log almost
│   │   │                                      #   entirely from exactly that. Used by Cpu.cs/Spc700.cs
│   │   │                                      #   for CpuVerboseLogging/Spc700VerboseLogging - compares
│   │   │                                      #   a small struct key, not the rendered string, so a
│   │   │                                      #   locked-in loop costs one comparison, no string
│   │   │                                      #   allocation, per instruction. Both cores auto-flush a
│   │   │                                      #   still-open cycle the moment their verbose flag turns
│   │   │                                      #   off (Cpu.Step()/Spc700.Step()'s _wasVerboseLogging
│   │   │                                      #   check); EmulatorSession.FlushVerboseLogs() and
│   │   │                                      #   Program.cs's shutdown path cover process exit, the
│   │   │                                      #   one case Step() can't see coming on its own. The
│   │   │                                      #   collapsed-loop marker line is built via a
│   │   │                                      #   caller-supplied renderMarker(cycleLength, repeats)
│   │   │                                      #   delegate, not hardcoded - it MUST carry the same
│   │   │                                      #   [TAG] prefix render() uses, or CategorizedLogWriter's
│   │   │                                      #   prefix table can't route it to the cpu/apu category
│   │   │                                      #   and it falls through to "general", which isn't in
│   │   │                                      #   the console-echo suppression list - a real session
│   │   │                                      #   hit exactly this and got the console blasted again.
│   │   └── Commands/                           # Every shell command, one class each: echo/sed/grep/wc/sort/
│   │                                            #   uniq/awk/ls/cd/mv/nano/coretop/history/true/false/test
│   │                                            #   (core-agnostic builtins) alongside mem/regs/watch/bp/
│   │                                            #   cheat/etc. (SNES-facing, need a real IDebugTarget)
│   ├── Audio/
│   │   └── WavFile.cs                         # Minimal uncompressed PCM WAV writer, moved out of
│   │                                           #   EmuSen.Pharaoh90/Program.cs alongside the Imaging/
│   │                                           #   move above - namespace EmuSen.Audio, same namespace
│   │                                           #   Settings/AudioSettings.cs already uses despite living
│   │                                           #   in a different folder
│   └── Settings/
│       ├── AudioSettings.cs
│       ├── DebugSettings.cs                  # Every logging toggle - see EmuSen_Settings_Reference.md.
│       │                                      #   Every *Logging flag is a property, not a plain field,
│       │                                      #   ANDed against MasterLoggingEnabled in its getter - one
│       │                                      #   switch silences everything at once without touching
│       │                                      #   any individually-set flag, and every call site
│       │                                      #   throughout the codebase needed zero changes since they
│       │                                      #   already just read e.g. DebugSettings.CpuVerboseLogging.
│       │                                      #   Toggle live from the F4 prompt via the `log` command
│       │                                      #   (DianaOS/Commands/LogCommand.cs).
│       └── AudioSettings.cs
│                                              # InputBindings.cs (used to live here) is deleted -
│                                              #   EmuSen.Hotaru/Input/HotaruKeyMap.cs is its Avalonia-
│                                              #   Key port. No Frontend/ here anymore - EmuSen.csproj
│                                              #   is a pure library (no OutputType, no Main). See
│                                              #   EmuSen.Hotaru/ below and this doc's §1/§2.
│
├── EmuSen.Hotaru/                # Avalonia console-first frontend (sibling of EmuSen.Mistress9/,
│   │                                     #   not layered inside EmuSen.csproj) - moved off Raylib
│   │                                     #   entirely; see EmuSen_Frontend_Driver.md's own top-of-
│   │                                     #   file revision note for the full migration.
│   ├── EmuSen.Hotaru.csproj
│   ├── Program.cs                       # Main = ROM resolution + core/debug-toolchain construction,
│   │                                     #   then hands off to Avalonia (AppBuilder.Configure with a
│   │                                     #   factory Func<App>) instead of driving a game loop
│   │                                     #   itself - see EmuSen_Frontend_Driver.md §1.
│   ├── App.axaml / App.axaml.cs         # Takes core/debugTarget/debugCmd/frameRecorder/statePath
│   │                                     #   via its constructor (not the usual parameterless one),
│   │                                     #   constructs one GameWindow in
│   │                                     #   OnFrameworkInitializationCompleted.
│   ├── Audio/
│   │   └── AudioPlayer.cs               # Duplicated from EmuSen.Mistress9's own SDL-backed copy,
│   │                                     #   adapted for ICore directly (Hotaru has no
│   │                                     #   EmulatorSession wrapper).
│   ├── Imaging/
│   │   └── FrameImageWriter.cs          # Screenshot/frame-recording PNG encoding via SkiaSharp's
│   │                                     #   SKImage.Encode directly, not Avalonia's
│   │                                     #   WriteableBitmap.Save - needs no AvaloniaLocator/
│   │                                     #   platform bootstrap, which is what keeps it unit-
│   │                                     #   testable with no display (see EmuSen.WiseMan/Imaging/).
│   ├── Input/
│   │   ├── HotaruKeyMap.cs              # Avalonia-Key port of the deleted InputBindings.cs's
│   │   │                                 #   keyboard scheme - no rebind/persistence, unlike
│   │   │                                 #   EmuSen.Mistress9's own ControllerKeyMap.
│   │   ├── GamepadManager.cs            # Duplicated from EmuSen.Mistress9's own copy.
│   │   └── GamepadBindingMap.cs         # Duplicated from EmuSen.Mistress9's own copy - same
│   │                                     #   %AppData%/EmuSen/gamepadbindings.json path, shared
│   │                                     #   between both frontends on purpose.
│   └── Views/
│       ├── GameWindow.axaml / .axaml.cs # Owns a GameFrameControl (EmuSen.Serenity) directly, not
│       │                                 #   via FramePresenter - see this file's own header comment
│       │                                 #   for why. Starts the background emulation thread, which
│       │                                 #   owns RunFrame()/audio/hotkey dispatch/the blocking F4
│       │                                 #   prompt; the UI thread owns the window/keyboard/gamepad
│       │                                 #   polling - see EmuSen_Frontend_Driver.md §1/§2.
│       ├── DebugWindows.cs              # coretop -w/feed -w window management - replaces the old
│       │                                 #   AvaloniaHost.cs's second-dispatcher-thread bootstrap,
│       │                                 #   now that Avalonia is the primary app.
│       ├── CoretopWindow.axaml / .axaml.cs
│       └── FeedWindow.axaml / .axaml.cs
│
├── EmuSen.Serenity/                  # Shared presentation layer (both frontends reference) - now a
│   │                                     #   real Avalonia-native, Skia-backed presenter, not a
│   │                                     #   Raylib one.
│   ├── EmuSen.Serenity.csproj
│   ├── GameFrameControl.cs              # An Avalonia Control implementing ICustomDrawOperation -
│   │                                     #   direct SKCanvas access via
│   │                                     #   ISkiaSharpApiLeaseFeature/ISkiaSharpApiLease. Draws the
│   │                                     #   letterboxed, upscaled game frame and (if active) a
│   │                                     #   single SkSL shader pass over it. ComputeLetterboxRect
│   │                                     #   fits the frame's own native aspect ratio into the
│   │                                     #   control's real bounds - deliberately not the fixed
│   │                                     #   configured-resolution canvas the old Raylib letterbox
│   │                                     #   used, which only existed to share layout with the now-
│   │                                     #   removed on-window debug overlay.
│   ├── FramePresenter.cs                # A plain Avalonia Window + GameFrameControl bundle ("just
│   │                                     #   the game, no chrome") for a consumer that needs nothing
│   │                                     #   else from its window. EmuSen.Hotaru's own GameWindow
│   │                                     #   bypasses this (it needs real keyboard capture/Closing
│   │                                     #   handling FramePresenter doesn't expose) and uses
│   │                                     #   GameFrameControl directly instead - this stays
│   │                                     #   available for a simpler future consumer (or
│   │                                     #   EmuSen.Mistress9, which still presents via its own
│   │                                     #   CPU-side WriteableBitmap and hasn't adopted this yet).
│   ├── GraphicsSettings.cs              # Moved out of EmuSen/Settings/ - see EmuSen_Settings_Reference.md §3.
│   │                                     #   Dropped ShowDebugPanels/PanelBackgroundColor/
│   │                                     #   LetterboxColor along with the on-window debug overlay.
│   └── Shaders/
│       └── BuiltInShaders.cs            # Skia SkSL (SKRuntimeEffect), not GLSL - a faithful port of
│                                         #   the same Scanlines/Crt darken-factor and vignette math
│                                         #   (F8 hotkey, EmuSen_Frontend_Driver.md §2).
│
└── EmuSen.Mistress9/                 # Avalonia GUI - renamed from EmuSen.Frontend, deliberately
    │                                     #   scoped as bug-testing tooling only, NOT the future
    │                                     #   EmulationStation-style launcher (that's a separate,
    │                                     #   not-yet-created project - see
    │                                     #   EmuSen_Launcher_Multicore_Gameplan.md)
    ├── EmuSen.Mistress9.csproj
    ├── App.axaml / App.axaml.cs
    ├── Audio/
    │   └── AudioPlayer.cs                 # Real audio output via SDL's queue-based audio API
    │                                     #   (SDL_OpenAudioDevice/SDL_QueueAudio) - reuses the
    │                                     #   Silk.NET.SDL dependency Input/GamepadManager.cs
    │                                     #   already brought in, rather than a second audio
    │                                     #   backend. Core-agnostic (only calls
    │                                     #   ICore.AudioSampleRate/DequeueAudioSamples via
    │                                     #   EmulatorSession) - see EmuSen_Frontend_Driver.md's
    │                                     #   audio section for the full pipeline.
    ├── Input/
    │   ├── ControllerKeyMap.cs
    │   ├── GamepadBindingMap.cs
    │   └── GamepadManager.cs             # Dispose() uses QuitSubSystem, not Quit() - SDL_Quit()
    │                                     #   tears down the whole library regardless of which
    │                                     #   subsystem asked, which would break Audio/
    │                                     #   AudioPlayer.cs's still-open device otherwise
    ├── Settings/
    │   └── AppSettings.cs                # Log/ROM directory + selected-core preferences -
    │                                     #   same JSON-under-%AppData% pattern as Input/*.cs
    ├── Program.cs
    └── Views/
        ├── MainWindow.axaml / .axaml.cs # ScreenWidth now an instance property tracking
        │                                #   the renderer's actual FrameWidth (hi-res support).
        │                                #   Still presents via a CPU-side WriteableBitmap,
        │                                #   not EmuSen.Serenity.FramePresenter - no GPU
        │                                #   surface here yet, so no shader support either.
        │                                #   Reuses EmuSen.Common.CategorizedLogWriter for
        │                                #   optional per-session file logging (Settings >
        │                                #   Preferences...), same categorized log files the
        │                                #   console build already produces. Audio now plays
        │                                #   for real too (Audio/AudioPlayer.cs, Pump()d once
        │                                #   per RunFrame() from the emulation thread, same
        │                                #   call-site placement as the console build's own
        │                                #   PumpAudio) - see EmuSen_Frontend_Driver.md.
        ├── InputSettingsWindow.axaml / .axaml.cs # Widened (680px, resizable) after key/pad
        │                                #   labels ("RightBracket", "Rightshoulder", ...)
        │                                #   routinely overflowed the original 90px columns
        │                                #   and rendered underneath the next column's button
        ├── PreferencesWindow.axaml / .axaml.cs # Log directory / ROM directory / core picker (the
        │                                #   last one is scaffolding - only one core exists)
        ├── DebugSettingsWindow.axaml / .axaml.cs # GUI on/off toggles for every DebugSettings
        │                                #   *Logging flag, grouped by CPU/APU/DMA/PPU/Memory
        │                                #   Bus, plus a bold master checkbox at the top bound
        │                                #   to MasterLoggingEnabled - individual checkboxes
        │                                #   disable themselves while it's off (see the
        │                                #   window's own comment on why: DebugSettings only
        │                                #   exposes each flag's post-AND effective value, not
        │                                #   its raw pre-master setting, so a checkbox toggled
        │                                #   while master is off wouldn't read back checked).
        │                                #   Settings > Debug Logging..., available even before
        │                                #   a ROM is loaded, in addition to the existing F4
        │                                #   `log`/`trace` prompt commands.
        └── RomBrowserWindow.axaml / .axaml.cs  # Lists .smc/.sfc files from the configured ROM
                                         #   directory as a quicker alternative to the OS
                                         #   file picker (File > Browse ROMs...)
```

Not shown: `Saves/*.srm`/`*.state`, `Logs/`, `bin/`, `obj/` — build artifacts and user data, excluded from any packaging.

---

## 4. Current features, by subsystem


### CPU (65816)
- Full 256/256 opcode table, verified against oxyron.de during initial development (one cross-reference typo caught and fixed).
- WAI/STP real halt states.
- **Real dynamic cycle-penalty tracking**: `Cpu.Step()` returns actual elapsed master clocks per instruction (region-aware, 6/8/12 master clocks by SNES memory region, tracking the `$420D` FastROM/SlowROM bit), not a flat per-opcode count scaled by one assumed rate for the whole machine. Includes the direct-page-low-byte-nonzero and indexed-page-crossing dynamic penalties. See `Venus_CPU.md` §8.
- **Known gap:** decimal (BCD) mode — SED/CLD correctly toggle the D flag, but ADC/SBC never check it. Rare in practice (few SNES games use decimal mode) but not implemented.
- **Known gap:** the 16-bit M/X-width dynamic cycle penalty for non-immediate memory-operand addressing modes isn't itemized per-opcode yet (immediate addressing already gets this for free) — see `Venus_CPU.md` §8.4.
- `LastInstructionPC`/`LastInstructionPB` — the pre-execution PC, exposed for debug tooling (distinct from the live `PC`/`PB`, which advance almost immediately after fetch — see the debugging tools reference, §2, for the bug this fixed).

### APU (SPC700 + S-DSP)
- Full 256/256 SPC700 opcode table.
- Real S-DSP audio synthesis: BRR decoding, real 4-tap Gaussian pitch resampling (confirmed byte-for-byte identical to the MesenCE reference's table/formula this session — see `Venus_APU.md` §2.9/§4.1), full ADSR/GAIN envelopes, 8-voice mixing.
- **Known gap:** synthesized samples are never sent to an actual audio output device. The DSP is "correct but silent."
- **Fixed this session:** a real, reproducible ~7.5-8.7% SPC700 audio-sample-generation-rate undershoot. Root cause: seven branch/CBNE/DBNZ opcode handlers charged their "+2 cycles if taken" penalty directly against `CycleBudget`, bypassing `Dsp.Tick()`/`TickTimers()` entirely — a real sound driver's polling/dispatch loops take enough conditional branches for this to compound into a measurable, continuous loss every frame. Found via direct cumulative-cycle instrumentation (comparing budget added vs. cycles actually reaching the DSP) after a from-scratch rewrite of the 65816-side cycle accounting ruled that side out entirely. SMW's measured rate went from ~980-985 samples/frame to ~1072/frame (expected ~1065.36) — within ~0.7% of real NTSC timing. See `Venus_APU.md` §2.9.

### PPU — background rendering
- **All 7 BG modes implemented**, including correct per-mode bit depth (2/4/8bpp as appropriate) and per-mode compositing/priority order — verified this session against the SNESdev wiki's Backgrounds page priority table for every mode (0 through 6; Mode 7 has its own separate compositing path). Two real bugs found and fixed in this pass: Mode 0's BG4 was missing its priority-bit split entirely, and Modes 2-5 were incorrectly reusing Mode 0/1's compositing order instead of their own (genuinely different) interleave pattern.
- 16x16 tile mode (BGMODE bits 4-7), tilemap 32x32/64x32/32x64/64x64 wraparound (including the tricky 64x64 case), tile bitplane format — all independently verified against the wiki.
- Mosaic, including a real starting-scanline latch (anchors to whichever scanline `$2106` was last written on, matching documented hardware behavior — not just always scanline 0).
- Mode 7: full affine transform (matrix formula verified against two independent sources), plus **EXTBG** (Mode 7's second layer via SETINI bit 6) — implemented this session, not yet tested against a ROM that actually uses it.
- Offset-per-tile (Modes 2/4/6): implemented as a best-effort reproduction of the commonly-documented behavior — the exact sub-tile column-alignment edge cases are something even experienced SNES homebrew developers describe as ambiguous in official documentation, so this is a solid approximation, not a verified-exact implementation.
- Direct Color mode (Modes 3/4's 8bpp BG1).
- Hi-res:
  - **Phase A (pseudo hi-res, SETINI bit 3):** real 512-column output via genuine main/sub screen interleave — not an approximation.
  - **Phase B (true Mode 5/6 hi-res):** real tile-pairing (two adjacent, ordinary tile definitions feeding the main vs. sub composite) rather than a blended approximation.
  - Both phases required real architecture work: the frame buffer, `GetFrameBufferRgba`, and the Avalonia frontend's bitmap all had to become width-aware instead of assuming a fixed 256px frame.
  - **Not done:** interlace's actual doubled-*vertical*-resolution output. The field-parity bit (STAT78) is correctly tracked now, but combining two fields into one taller displayed image was scoped out — deliberately, given genuine interlace saw real gameplay use in only a couple of known titles, and the engineering cost (scanline *count* feeds this project's core timing loop directly, not just buffer dimensions) is meaningfully larger than hi-res was for a much rarer payoff.

### PPU — sprites (OBJ)
- Full OAM decode (size-select, high-table, priority rotation), correct multi-tile VRAM addressing (16-tile-wide grid, 512-byte row stride).
- Real per-scanline evaluation matching hardware's 32-sprite/34-sliver limits, including the reverse-index-order sliver-culling detail.
- OBJ color math restriction (only palettes 4-7 participate).
- **Real bug fixed this session:** sprite Y was being culled too aggressively (`Y >= 224`), discarding sprites that should wrap in from the top of the screen (Y = 240-255, signed -16..-1) instead of rendering their visible portion.

### PPU — other
- Full open-bus emulation (`$4210`/`$4211`/`$4212`'s undriven bits now reflect the last bus value instead of being hardcoded to 0), plus the general "last value on the bus" model for genuinely-unmapped register reads.
- Windowing (masking logic independently verified against 3 sources).
- **Color math / CGADSUB, per-layer participation rules**, plus three pieces this doc hadn't previously written down despite being real, currently-correct, in-use behavior (found and fixed during an earlier, otherwise largely-unreliable third-party detour — see this doc's own honesty standard: the fix is worth keeping regardless of how unreliable the session that produced it was elsewhere):
  - **CGWSEL bits 4-5** gate color math itself: always on, only inside the color-math window, only outside it, or never — `Renderer.Scanline.cs`'s `colorMathEnable` check.
  - **The color math window itself** (`IsColorMathWindowMasked`) — parses `WOBJSEL` (`$2125`) bits 4-7 for each of the two math windows' enable/invert, `WOBJLOG` (`$212B`) bits 2-3 for the AND/OR/XOR/XNOR combine logic when both windows are active, and `CGWSEL` bits 6-7 for a final main/sub-screen invert. This is the mechanism behind effects like SMW's cave/ghost-house "spotlight" (color math restricted to a moving window instead of applying screen-wide).
  - **Fixed Color Register (`$2132`/COLDATA)** — per-channel (R/G/B) latched fixed color, used as the sub-screen operand when CGWSEL bit 1 selects "fixed color" over the real sub-screen.
  - **The half-math-disabled-against-fixed-color hardware quirk**: half color math (`CGADSUB` bit 6) is forced off when blending against the fixed color (rather than a real sub-screen pixel), unless `CGADSUB` bit 5 ("backdrop enabled") is set — a documented, non-obvious real-hardware edge case, not a bug in either direction.

### Memory / DMA
- LoROM mapping; SRAM size now read from the ROM header (was hardcoded to 2KB for every game) with correct chip-mirroring behavior (modulo addressing instead of a hard size cutoff) — this was the actual fix for Super Metroid's boot-time anti-piracy check, which specifically tests for correct SRAM mirroring.
- WRAM low-bank mirror (`$00-$3F`/`$80-$BF:$0000-$1FFF`).
- `$2180-$2183` (WMDATA/WMADDL/M/H) — a second WRAM access path, previously entirely unimplemented.
- `$2140-$217F` APU port mirroring (previously only the canonical `$2140-$2143` were handled).
- General DMA + HDMA.
- `MemoryBus` decoupled from debug-toolchain plumbing: the hardware multiply/divide unit now lives in its own `MathUnit` class rather than as loose fields/logic on the bus, and `MemoryBus` no longer holds a `WatchRegistry` or a `Cpu` back-reference — it exposes a small `IWriteObserver` hook instead, with `SnesDebugTarget` on the other end. See §2.

### Save states, input, frontends
- Save states via a reflective binary serializer (F5/F9 in the console build; menu items in Avalonia). **Known limitation:** no version header — can break across builds if field layout changes.
- SRAM auto-save (every 300 frames + on exit/ROM switch).
- Rebindable keyboard + gamepad input (Avalonia frontend), Wayland support.
- Console frontend: ROM path via CLI arg; Avalonia frontend: real file-picker.
- Console frontend's `Program.cs` decoupled: `Main` used to be a single ~350-line method containing ROM loading, the timing loop, and all six hotkeys inline. Hotkey dispatch (F1-F5, F9, P) is now its own `RunHotkeys` method, separate from the actual per-scanline emulation loop.

### Debug toolchain
Covered in full in the companion document. Summary: a core-agnostic `IDebugTarget` interface (memory spaces, registers, sprites, palettes, watchpoints, frame counter, disassembly), a `SnesDebugTarget` implementation, and a general-purpose bash-alike shell (`DianaOSInterpreter`, `EmuSen.DianaOS`, user-facing name **DianaOS**) reachable via the console's F4 prompt - quoting, `$VAR`/`$(...)` expansion, pipes, `>`/`>>`/`<` redirection to real files, `;`/`&&`/`||` sequencing, if/for/while control flow, up/down-arrow history recall, a small coreutils subset (`ls`/`cd`/`grep`/`awk`/`nano`/...), and a live `coretop` hardware dashboard, on top of the same `mem`/`regs`/`watch`/etc. debug commands it always had. Built specifically so it isn't SNES-only and can eventually back a real GUI debugger.

---

## 5. Known, already-accepted architectural simplifications (not bugs — don't re-flag without new evidence)

- No per-dot H-position timing — scanline granularity throughout (affects OPHCT and a few edge cases).
- Offset-per-tile's exact sub-tile alignment (see above).
- Mosaic/mode-change/HDMA-timing all operate at scanline granularity, with documented per-feature latching behavior where hardware specifically requires it (mosaic's starting-scanline latch, the immediate-NMI-on-vblank-rising-edge case, etc.) rather than true per-cycle accuracy.
- Frame timer in the Avalonia frontend is a fixed 60fps UI timer, not accumulator-driven — will drift over long sessions.

---

## 6. Open bugs / active investigations

- **Coins and Yoshi not rendering in SMW.** Long investigation this session — ruled out: sprite/tile rendering logic (verified correct against docs and against the exact tile+palette data dumped from a real session), the general DMA transfer mechanism (proven correct via adjacent, working animated-tile transfers), and the DMA source-address computation itself (confirmed varying correctly in the most recent session, not stuck). Currently narrowed to: **whatever's supposed to write real graphics data into the WRAM staging buffer before the DMA copies it out doesn't seem to be doing so** — a targeted watch (`watch add WRAM 8000 1800` via F4) is in place to confirm this directly. Not yet resolved.
- **A stuck HDMA window on the title screen** (freezes at a single pixel) — isolated but never root-caused; windowing was previously disabled globally to work around it, then re-enabled once judged lower-risk than leaving every window-based effect broken everywhere. Worth a dedicated pass now that windowing is back on.

---

## 7. TODO / roadmap

Roughly in order of "cheap and likely valuable" to "bigger, deliberately-deferred":

1. **Continue the coin/Yoshi WRAM investigation** (above) — status since last touched here is unconfirmed; subsequent sessions moved on to other games/bugs (ALTTP lamp/inventory bug, LttP overworld-subscreen bug, Super Metroid boot hang - see `EmuSen_Games_Tested.md` and `Venus_PPU.md`/`Venus_APU.md`'s own writeups) without a recorded resolution of this one. Worth checking whether it's still actually open before resuming it as "the immediate active thread" - it almost certainly no longer is.
2. **Get an actual test ROM for Mode 7 EXTBG and offset-per-tile** to confirm those implementations against real content rather than documentation alone (same category of "implemented but unexercised" as tile16/mosaic were before SMW's own logs confirmed them safe).
3. **A verification pass on the new disassembler** (`Snes65816Disassembler`) — built carefully but not given the oxyron.de-level scrutiny the execution opcode table got. See the debugging tools reference, §3.7.
4. **Watchpoints beyond WRAM** — report writes from `Ppu`'s VRAM/CGRAM/OAM paths and the general CPU-bus/SRAM path through `MemoryBus`'s `IWriteObserver` hook the same way WRAM already does.
5. ~~The drive-by namespace rename~~ — done: `Cores/Snes/` → `Cores/Nintendo/Venus - SNES/`, `EmuSen.Memory`/`.Apu`/`.Processor`/`.Video`/`.Controllers` → `EmuSen.Cores.Nintendo.Venus.*`. See `EmuSen_Core_Naming_Scheme.md`.
6. **The rest of the `MemoryBus` decoupling** — the multiply/divide unit and the debug-toolchain plumbing are out (§2, §4); H/V-IRQ/NMI/vblank state is not. On closer inspection this cluster turned out more entangled than it first looked (the same `_vblankFlag` feeds NMI edge-detection *and* the RDNMI/HVBJOY register reads, and $4200 sets both NMI and IRQ enable in one write) — forcing a clean split risked adding more cross-object coupling than it removed, in genuinely delicate, already-hard-won timing logic. Worth revisiting deliberately, not as a quick follow-on.
7. ~~The `Renderer`/Raylib split~~ — done, as part of the Hotaru Raylib→Avalonia migration: `Renderer.DrawDebugPanels`/`Shutdown()` (the on-window debug-overlay drawing and its lazily-created GPU texture) are gone entirely, so `Renderer` no longer owns any window/texture at all, in any mode. It still keeps `Raylib_cs.Color` as its internal pixel type (91 uses, `Renderer.*.cs`) - a deliberate, separate decision (`EmuSen.csproj` keeps a lightweight `Raylib-cs` reference purely for that struct) rather than a full `Rgba32` migration, since that part was judged zero-payoff busywork unrelated to the actual goal (removing Raylib's *window/rendering* ownership from the frontend, not "zero Raylib anywhere in the solution"). See `EmuSen_Frontend_Driver.md`'s own top-of-file revision note.
8. **Audio output** — connect the already-correct S-DSP synthesis to an actual playback device. The SPC700 audio-pacing undershoot that would have made this audible as "running slow" is fixed (§4's APU section, `Venus_APU.md` §2.9) — no longer a blocker for this item.
9. **Decimal (BCD) mode** on the 65816 (ADC/SBC currently ignore the D flag).
10. **The stuck HDMA title-screen window bug** — dedicated investigation, now that windowing is confirmed safe to leave on globally.
11. ~~Breakpoints / single-step / pause-resume~~ — done (`VenusCore.RunFrame()` mid-frame halt/resume, `BreakpointRegistry`, F4 prompt `step`/`continue`). See `EmuSen_Debugging_Tools_Reference_v5.md` §3.1/§3.3. Not yet wired into `EmuSen.Pharaoh90` or the Avalonia GUI (item 12 below).
12. **The Avalonia GUI debug window** — the actual Mesen-style multi-pane debugger, built against `IDebugTarget` once enough of the above exists to make it worthwhile.
13. **A second `IDebugTarget` implementation (NES or otherwise)** — to actually prove out the core-agnostic design rather than just asserting it.
14. **True doubled-resolution interlace output** — explicitly deferred (§4) given its rarity in real games versus its engineering cost; revisit only if a specific ROM actually needs it.
15. **NES core work generally** — the original long-term goal this whole architecture (folder structure, `IDebugTarget`, the eventual namespace cleanup) has been building toward, still not started. Planned to begin only once SNES + the Avalonia frontend are stable — per earlier project discussion, not a new decision.
