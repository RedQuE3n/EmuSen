# EmuSen_ShaderWindow — Minako, the shader window on the shared frame

*Proposal, 2026-10-27; its questions answered by the tester the same day (§5). Nothing is built.* Asked by the tester the same day: a proper shader window in place
of the present one, consistent with the other tools (`EmuSen_LibraryTools.md` §1), named **Minako** (Sailor Venus).
Menus call it *Shaders*.

## 1. What exists

`ShaderSettingsWindow` (`EmuSen_Settings_Reference.md` §4.48): one tab per console; the built-in filters (the CRT and
handheld-screen models of `EmuSen_ScreenFilters`, the accurate per-console CRT profiles) and RetroArch's slang presets
from a downloaded pack in one searchable, virtualised list; the chosen shader's parameters as sliders with a search, a
Reset per slider and Reset All; reads settled and cached so a Mega Bezel preset's hundreds of parameters stay quick;
a big-screen layout for the pad. The Graphics Settings window has a row that names the console's shader and opens it.

**What it lacks** is mostly shape, not machinery: no preview, so a shader is chosen blind; a tab per console, which
does not scale to the consoles EmuSen now has; nothing to compare against; and the choice reached from another window.
All of the list's, the reads' and the sliders' machinery is kept.

## 2. The window

On LunaP's `ManagerLayout` (§1 there), as Setsuna, Ami and Rei are, with the panes put to a shader's use:

```
┌ Toolbar: [Apply] [Compare ◐] [Reset All]   Preview: [Last played ▾]   Search… ┐
├────────────┬──────────────────────────────┬─────────────────────────────────┤
│ Consoles   │ Shader            Kind   From │  ┌───────── preview ─────────┐  │
│  SNES  ●   │ ● Accurate CRT    CRT    built│  │                           │  │
│  N64       │   CRT Lottes      CRT    built│  │   the console's picture   │  │
│  PS1       │   Game Boy LCD    LCD    built│  │   through the shader      │  │
│  …         │   crt-royale      CRT    pack │  └───────────────────────────┘  │
│ Kind       │   …                           │  Parameters (search)            │
│  CRT · LCD │                              │   Scanline strength ━━━●── Reset │
│  Sharp …   │                              │   Mask ━━●───────────── Reset    │
├────────────┴──────────────────────────────┴─────────────────────────────────┤
│ SNES: Accurate CRT (2 changed)   Apply to: (•) this console ( ) every TV console │
└─────────────────────────────────────────────────────────────────────────────┘
```

- **Consoles in the sidebar**, not tabs, each marked with what it uses now; with kinds (CRT, LCD, sharp, smooth,
  bezels) and the source (built-in, the pack) as filters.
- **A live preview** of the console's picture through the selected shader and its parameters, redrawn as a slider
  moves. Its source is chosen: the **last frame played** on that console (the session's own screenshot), a
  **test pattern** made for the console's resolution and colour (EmuSen's own, no game's), or a **screenshot** the
  player picks. Drawn by the same filter chain the game uses, at the window's size, so what is seen is what plays.
- **Compare**: a divider dragged across the preview, the console's current shader on one side and the selected one on
  the other (RetroArch's and many image editors' before/after).
- **Apply** to this console, or every console of the same kind (every TV console, every handheld); nothing changes until
  Apply, and the bar says what will.
- **The Graphics Settings window** keeps the engine and scale; its shader row names the choice and opens Minako at that
  console, as today.
- **The big-screen layout** stays: the panes stack for the pad as the present window's do, the preview at the top.
- The window fits 1280×800 and 1920×1200 (the fit audits).

## 3. What it needs beneath

- **A preview renderer**: the screen filter chain run off-screen on one still frame, into an image the window shows,
  on the same device as play (`EmuSen_GpuPass.md`) or the software path where there is none. The chain exists; what
  is new is driving it with a still frame outside a session, and doing so within a frame's time as a slider moves.
- **The last frame per console**: the frame a session last presented, kept at the end of a session in EmuSen's home
  (one image per console, replaced each time), with no game data beyond the picture.
- **Test patterns**: generated per console resolution (bars, a grid, a gradient, text at the console's size), written
  by EmuSen's own code.
- **A DianaSh command**, so the shell can do what the window does, as the other tools' backends are commands:
  `shader ls [<console>]`, `shader use <console> <shader>`, `shader set <console> <parameter> <value>`, and
  `shader render <console> <shader> <in.png> <out.png>`, which renders a still through a shader (the preview renderer
  as a command, and a tool for tests and screenshots). In a module of its own, `emusen.shader`.

## 4. Stages

| Stage | What | Model |
|---|---|---|
| **S1** | The preview renderer and `shader render`; the last frame kept; test patterns | careful (the GPU path; a GPU window with the tester present) |
| **S2** | Minako on `ManagerLayout`: the sidebar, the list, the sliders moved over, Apply's scopes | routine |
| **S3** | The live preview and Compare in the window; the big-screen layout | routine, the GPU checks in a GPU window |
| **S4** | `shader ls`, `use`, `set` | routine |

It follows Setsuna's M1, which builds the frame.

## 5. Decided 2026-10-27 by the tester

1. The preview shows the last frame played, and the test pattern where the console has none.
2. Apply's scopes are this console and every console of its kind; no per-game shader.
3. The present window is retired once Minako has everything it has.
4. Order: after Setsuna's M1; S1 with the most care after 2026-10-31, in a GPU window with the tester present.
