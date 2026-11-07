# EmuSen_Debugger — Debugger, one window to see inside the machine, for debugging and for learning

*Proposal, 2026-10-27; its questions answered by the tester the same day (§6). Nothing is built.* Asked by the tester the same day: a proper debug window in the
manner of Mesen's viewers — tiles and where they sit in memory, sprites, how hard each part of the emulated machine is
working — scaled to the 3D consoles too; and the learning mode (one of the six features approved that day) folded into
it rather than built as a window of its own.

## 1. What exists

- **The core ABI's debug surface** (`EmuSen_CoreAPI.md` §6.5, §6.33, §6.34): memory spaces with their flags, registers,
  disassembly records (the core decodes, DianaSh formats), read ranges for `DEBUG_READS`, machine info. Shared through
  `emusen-native`'s core description, read by `crystal-debug`.
- **DianaSh commands** over it: `spaces`, `mem`, `regs`, `disasm`, `tile`, `tilemap`, `sprites`, `pal`, breakpoints,
  watches, traces, the profiler, coverage; and `coretop`, an htop-style dashboard with per-subsystem load bars, a sprite
  gauge, voice meters, the palette and a VRAM preview, with `-w` for windows.
- **What is missing**: the viewers are text in a shell or a dashboard per core; there is no window a player browses, no
  address shown beside a tile, nothing for a 3D console's frame, and each viewer knows its console's formats itself.

## 2. The design: cores declare views, one window draws them all

Following the project's rule that nothing is hand-coded where a declared mechanism can do it, **no viewer is written
per console.** A core declares what can be seen of its machine, and decodes it, since only the core knows its formats;
the window and the commands draw any core's views with a handful of generic renderers.

- **A new optional capability of the core ABI, "debug views"** (a minor version of v1's debug surface), declared once per
  core through `emusen-native`. A view is one of a few kinds:
  - **image**: a picture the core renders on request (a tile sheet, a tilemap, VRAM as a whole, a texture cache, a
    framebuffer, a depth buffer), with a **cell grid** and, per cell, its address, its index and its palette or format,
    so hovering a tile names where it lives in memory;
  - **table**: rows of named fields (sprites/OAM, palettes, DMA channels, voices, a 3D frame's draw calls, a display
    list or command packets), each row optionally pointing at an image region or an address;
  - **memory**: the existing spaces, with the reads, writes and executions of the last frame shown as heat;
  - **units**: the machine's parts (CPU, PPU or GPU, DMA, sound; on a 3D console its coprocessors: the RSP and RDP, the
    GTE, the EE, VU0, VU1, GIF and GS) with how busy each was this frame, as counters the core already keeps;
  - **events**: what happened when in the frame, by scanline and cycle (register writes, interrupts, DMA), Mesen's
    event viewer.
  Every view, field and unit may carry **a short explanation**, plain text in the core's own manifest — the learning
  layer is declared data too.
- **DianaSh first**: `view ls`, `view show <name> [--png file]`, `view rows <name>`, `units`; the existing `tile`,
  `tilemap`, `sprites` and `pal` become views of the same capability, so a core writes each once.

## 3. The 3D consoles

A 3D frame is not a tile sheet, so the views take the form graphics debuggers (RenderDoc and its kind) use:

- **Frame capture**: one frame's **draw list**, a table of every draw (or RDP triangle, GP0 primitive, GIF packet), and
  choosing one shows its state, its vertices, the texture it read and the pixels it wrote, the frame built up draw by
  draw on the image beside it.
- **Video memory as an image**, laid out as the hardware lays it out: the PlayStation's 1024×512 VRAM with its texture
  pages and colour tables marked; the N64's TMEM and the textures in RDRAM; the PS2's GS local memory by page and block,
  each region named by address and format.
- **The units under load**: the busy share of each coprocessor per frame (Sapphire's bench already measures EE, VU, GIF
  and GS shares), triangles and pixels drawn, and where a frame's time went — the same counters `coretop` and the bench
  read, shown live.

## 4. The window

On LunaP's `AdaptiveLayout` (`EmuSen_LibraryTools.md` §1a), so it has its big-picture counterpart and the pad from the
start:

- **Navigation**: the core's views grouped (Video, Memory, Processors, Sound, Load, Frame).
- **Content**: the view drawn by its kind's renderer, zoomable; hovering or the pad's focus on a cell or row shows its
  address and fields.
- **Detail**: the selected thing in full, and its explanation.
- **Actions**: pause, step a frame, step an instruction, capture a frame (3D), set a breakpoint on this address, copy.
- **A console docked beneath**: the Shell window's pane (`EmuSen_Shell.md`), hosted under the views as Mesen keeps its
  script window, so `bp`, `mem` and `step` are typed beside what they change; the Debugger writes no shell of its own
  (`EmuSen_LibraryTools.md` §1b).

### 4.1 Learning: the same window, explained (the tester's 2026-10-27 choice)

- **Explore** and **Debug** are two modes of the one window over the same views: Explore shows the explanations first,
  fewer columns, plain names ("sprite table", "colour palette") and the "what is happening now" line per unit; Debug
  shows everything. Nothing is built twice.
- **Tours**: a guided walk through a frame ("how this game draws its screen") as a plain text file of steps — open a view,
  point at a region, say a sentence, step a frame — run by DianaSh, so tours are written, shared and read as text.

## 5. Stages

| Stage | What | Model |
|---|---|---|
| **D0** | The debug-views capability in the core ABI and `emusen-native`; `view` and `units` in DianaSh | careful: a stable surface |
| **D1** | The window's renderers (image with cell grid, table, memory heat, units, events) on `AdaptiveLayout` | routine |
| **D2** | The 2D cores' views: tiles, tilemaps, sprites, palettes, events, with addresses; `tile`, `sprites` and `pal` moved onto them | routine, a core at a time |
| **D3** | The 3D views: frame capture and draw lists, video memory as laid out, the coprocessors' load (Mars, Diamond, Sapphire) | careful |
| **D4** | Explore mode, the explanations in each core's manifest, tours | routine |

After T0 (`AdaptiveLayout`). Mesen is a model for what to show, from its documentation and screenshots only; no emulator
source is read.

## 6. Decided 2026-10-27 by the tester

1. Cores declare their views and decode them; one generic window draws every core's.
2. Learning is the window's Explore mode; tours are text files run by DianaSh.
3. The window is the **Debugger**; its name was Chibiusa until 2026-10-28.
4. Order: after T0; D0 and D3 with the most care.
