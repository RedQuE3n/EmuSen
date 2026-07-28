# Venus (SNES) — PPU (video)

Covers `Cores/Nintendo/Venus - SNES/Ppu/`: register state and dispatch (`Ppu.cs`, `Ppu.Registers.cs`, `Ppu.RegisterTable.cs`) and the scanline renderer (`Renderer.cs`, `Renderer.Backgrounds.cs`, `Renderer.Mode7.cs`, `Renderer.Scanline.cs`, `Renderer.Sprites.cs`, `Renderer.Debug.cs`). See `Man pages/Hardware/README.md` for what this doc set is.

---

## 1. Register dispatch

`Ppu.WriteRegister`/`ReadRegister` index a 64-entry `PpuRegister[]` table (`$2100-$213F`) built once in `BuildRegisterTable()`, each slot pairing a named write/read delegate — same dispatch-table shape as `MemoryBus`'s address decode (see `Venus_Memory.md` §1.2), just scoped to the PPU's own register window. Unmapped slots default to `WriteUnmapped`/`ReadUnmapped` (no-op / open bus).

`$2132` (COLDATA, fixed color for color math) is handled as a special case directly in `WriteRegister` rather than through the table, since it's a "modify one of three channels depending on which control bits are set" register, not a simple latch-and-store.

---

## 2. Scroll registers — the two-latch write-twice mechanism

BG scroll registers ($210D-$2114) are write-twice, 8-bit-at-a-time, combined into a wider value via shared latches — but the *exact* latch-sharing rules matter and were a real, hard-won bug fix.

**`BGnVOFS` is simple**: `BGnVOFS = (new<<8) | latch`, where `latch` is shared by *all eight* `BGnxOFS` registers (`_bgOfsLatch`).

**`BGnHOFS` is not** — and getting this wrong was a real, shipped bug: `BGnHOFS = (new<<8) | (Prev1 & ~7) | (Prev2 & 7)`, where `Prev1` is the same shared 8-register latch as VOFS, but **`Prev2` is a *separate* latch shared only by the four `BGnHOFS` registers** (`_bgHOfsLatch`). The previous implementation instead read *this register's own* previously-committed value for the `Prev2` term, which silently drops bit 2 of the low byte on every write. Confirmed via the MesenCE team's own reference documentation (snesdev.mesen.ca/wiki, PPU_Registers page — MesenCE, at github.com/nesdev-org/MesenCE, is the direct, actively-maintained continuation of the original SourMesen/Mesen2 codebase, archived June 2026). Verified by simulating both formulas against real captured write data: the register's-own-value version reversed scroll direction repeatedly; the two-latch version reconstructs every value exactly, zero reversals. This was the real fix behind what this project's history calls the "BG2 horizontal-only scroll jitter."

**`$210D` (BG1HOFS) also piggybacks Mode 7's own H-scroll (`M7HOFS`)**, via a *third*, separate write-twice latch (`_m7OfsLatch`) — not the BG-scroll latches above. This runs unconditionally, not just when Mode 7 is selected, matching real hardware latching the value regardless of active mode (only whether it's ever *read back* depends on the mode). `$210E`/BG1VOFS piggybacks `M7VOfs` the same way. Three independent latch groups in total: the 8-register VOFS/Prev1 latch, the 4-register HOFS/Prev2 latch, and the Mode-7-only latch — conflating any two of them reproduces the jitter bug.

Scroll values are genuinely 10-bit (0-1023) on real hardware, not 8-bit — needed once a tilemap is wider/taller than one 32x32 screen.

---

## 3. Mode 7

BG1 becomes a single affine-transformed 1024x1024-pixel layer instead of a normal scrolled tilemap in Mode 7 — no priority-bit split, no bit-depth variation (always 8bpp), genuinely a different rendering algorithm from the tilemap path `RenderBg1`-`4` share.

### 3.1 Transform formula

Verified against two independent sources (SNESdev wiki's "Mode 7 transform" page and a NESDev forum matrix-form writeup) that agree byte-for-byte:

```
[X]   [A B]   [SX + HOFS - CX]   [CX]
[ ] = [   ] * [                ] + [  ]
[Y]   [C D]   [SY + VOFS - CY]   [CY]
```

where `(SX,SY)` is the screen pixel, `(CX,CY) = (M7X,M7Y)` is the pivot point, `(HOFS,VOFS) = (M7HOFS,M7VOFS)` is Mode 7's own scroll (see §2's piggyback note), and A/B/C/D are signed 8.8 fixed-point (raw value / 256.0).

### 3.2 VRAM layout

Genuinely different from every other mode: the 128x128-tile tilemap lives in the **low byte** of each VRAM word starting at word 0 (one byte per entry — just an 8-bit tile index, no flip/priority/palette bits), and the 8bpp tile/character data lives in the **high byte** of those same words, interleaved. Confirmed against the SNESdev wiki's Tilemaps/Tiles pages and SnesLab's Mode 7 VRAM Map page independently.

### 3.3 M7A/M7B shared registers and MPYL/M/H

M7A-D/M7X/M7Y share one write-twice latch (`_m7Latch`), separate from both the BG-scroll latch and the M7HOFS/VOFS latch (§2). `MPYL/M/H` (`$2134-$2136`) read back **`M7A * the raw 8-bit value most recently written to $211C (M7B)`** — *not* `M7A * the combined 16-bit M7B` — confirmed via ff6hacking's register docs ("the product of the 16-bit value written to $211B and the 8-bit value most recently written to $211C"). Updates on every write to `$211C`, both halves of the pair, matching "most recently written" literally.

### 3.4 EXTBG (SETINI bit 6)

Gives Mode 7 a second layer, BG2, sampling the **exact same** affine-transformed texture as BG1 — identical tilemap, tile data, matrix, and scroll registers. Confirmed via the Super Famicom Dev wiki's Backgrounds page. The only difference is interpretation: BG2 reinterprets the sampled pixel byte's high bit as a **priority flag** instead of a color bit, so it only has 128 distinct colors (0-127) where BG1 has the full 256. Resulting priority order: BG2's priority-1 pixels sit above BG1, its priority-0 pixels sit below BG1, with sprites interleaved at their usual 4 priority levels around both — which is why `RenderScanline`'s `isMode7` branch needs its own two-pass call sequence rather than reusing the normal-mode BG1/BG2/OBJ interleave order.

### 3.5 Not implemented

The documented 13-bit `CLIP()` precision quirk on the intermediate `SX+HOFS-CX`/`SY+VOFS-CY` values — an obscure edge case only relevant for extreme rotation/scale parameters. Flagged rather than silently approximated.

---

## 4. Compositing order (`RenderScanline`)

Each scanline builds a main-screen buffer and a sub-screen buffer independently (`_mainLineBuf`/`_subLineBuf`), then blends them per CGADSUB — matching real hardware's own two-screen model. Layer draw order within each buffer is mode-dependent and deliberately follows the documented BG/OBJ priority interleave for that mode (e.g. Mode 0-1: BG2-low, OBJ0, BG1-low, OBJ1, BG2-high, OBJ2, BG1-high, OBJ3 — see the `mode >= 2 && mode <= 5` branch for the exact sequence), not a simple "draw all BGs then all sprites" order — sprite priority levels (0-3, from OAM attribute bits) are genuinely interleaved *between* BG priority passes on real hardware, and this renderer reproduces that literally rather than approximating it with a sort.

**Mode 1's BG3-forced-top** (`BGMODE` bit 3): when set, BG3's high-priority tiles draw *after* everything else, unconditionally on top — including a correctly-computed BG1 pixel. Handled as a separate pass after the mode's normal interleave.

---

## 5. Color math

- **Per-layer participation** (`LayerParticipatesInColorMath`) reads CGADSUB's per-layer enable bits directly, with one hardware-specific restriction: **OBJ color math on the main screen only applies to sprites using OBJ-relative palettes 4-7** — palettes 0-3 stay opaque even with CGADSUB's OBJ-math bit set, letting a game keep some sprites always-opaque and others blending purely by palette choice, no extra per-sprite flag needed. Confirmed via the SNESdev wiki's Sprites page. The sub screen has no such restriction (any sprite there can act as a blend source) — already naturally handled since this check is only ever consulted for the *main* screen's winning layer.
- **Half-color-math-disabled-for-fixed-color quirk**: half color math is disabled when blending against the fixed color (explicitly, or via an empty sub-screen pixel) *unless* the backdrop layer itself is enabled in CGADSUB. Implemented directly in `RenderScanline`'s final blend step (`isFixedColor` check).
- **Blend clamping**: `BlendColors` clamps subtract to 0 and add to 255 per channel, and **always forces alpha back to 255** regardless of math result — alpha dropping to 0 here was a real regression (documented in-place as "CRITICAL... if this drops to 0, Yoshi goes invisible").
- **Sub-screen backdrop fallback**: wherever nothing is drawn on the sub screen, real hardware uses the Fixed Color register (`$2132`) as the fallback rather than CGRAM color 0 — modeled directly as `subBackdrop` in `RenderScanline`.

---

## 6. Sprites (OBJ)

### 6.1 Per-scanline hardware budget

Real hardware evaluates at most **32 sprites** and **34 8-pixel tile slivers** per scanline; exceeding either sets a status flag games can poll:

- More than 32 on-screen sprites this scanline → `RangeOver` (read via STAT77 bit 6).
- More than 34 slivers consumed rendering those sprites → `TimeOver` (read via STAT77 bit 7).

`EvaluateSpritesForScanline` reproduces both limits literally (collects up to 32 sprites in OAM index order, then renders slivers back-to-front until the 34-sliver budget runs out), not just as a cosmetic flag — a game relying on either limit for a deliberate effect (or relying on *not* exceeding it) sees the same behavior as real hardware.

### 6.2 Priority rotation (OAMADDH bit 7)

Confirmed real via the SNESdev wiki's Sprites page ("OAMADD can adjust this with 'priority rotation'"). When enabled, sprite 0 is no longer necessarily topmost; instead the sprite at `(OAMAddr & 0xFE) >> 1` becomes the first/topmost one, wrapping through all 128 from there. `Ppu.FirstSpriteIndex` computes this; `EvaluateSpritesForScanline`'s sprite-collection loop starts from it instead of always starting at index 0.

### 6.3 Slivers consumed left-to-right

Tile slivers are consumed in on-screen left-to-right order, matching the documented culling order — not VRAM/flip order. This matters for which slivers get dropped once the 34-sliver budget is exhausted mid-sprite.

---

## 7. Windowing

Two windows (W1: `$2126`/`$2127`, W2: `$2128`/`$2129`) can each be enabled/inverted per layer via the per-layer select registers (`W12SEL`/`W34SEL`/`WOBJSEL`), with configurable combine logic (OR/AND/XOR/XNOR, from `WBGLOG`/`WOBJLOG`) when both windows are active for a layer. The color-math window is a *separate* mask, using `WOBJSEL` bits 4-7 and `WOBJLOG` bits 2-3 rather than a specific layer's own bits — `CGWSEL` bits 6-7 additionally invert the final window output for the main/sub screen independently. `IsWindowMasked`/`IsColorMathWindowMasked` implement these as two distinct functions since they read from different bit groups and serve different purposes (visibility mask vs. color-math eligibility mask), even though the window-position/invert/combine logic itself is structurally identical between them.

---

## 8. Pseudo-hi-res vs. true hi-res (Modes 5/6)

**Phase A (implemented)**: real pseudo-hi-res column interleave (`SETINI` bit 3, outside Modes 5/6) — confirmed via the SNESdev wiki's Backgrounds page ("the main-screen appears on every even column, and the sub-screen appears on every odd column"). This does *not* change how BG1-4/OBJ render — they still compute a normal 256-wide `_mainLineBuf`/`_subLineBuf` exactly as in standard resolution (Mesen's own hi-res output confirms real hardware interleaves two already-independently-rendered screens here, it doesn't render new content at 512-wide density). Only the final compositing step in `RenderScanline` needs to know about the wider 512-pixel output.

**Mode 5/6 (true forced hi-res) is deliberately *not* part of Phase A** — Mesen's own hi-res output is genuinely 512 pixels of *distinct* tile content there, from BG1/BG2 rendering at double horizontal density (16x8 tiles), not a reused 256-wide buffer. That needs real changes to `RenderBg1`/`RenderBg2`'s tile-fetch math (Phase B, not yet done); Modes 5/6 currently still use the same 50% blend approximation as before, unchanged, tracked separately from Phase A so it isn't confused with genuine hi-res support.

`Renderer.FrameWidth` reports the current output width (256 normally, 512 during pseudo-hi-res) — frontends should check this rather than assuming a fixed size, since it can change frame to frame if a game toggles `SETINI` bit 3. `_screenPixels` is always allocated at the max width (512) with a fixed row stride, so a hi-res toggle never needs reallocation.

---

## 9. Status/read-only registers

- **`SLHV` (`$2137`)**: reading it latches the current H/V position into `OPHCT`/`OPVCT`. Real hardware also latches via WRIO (`$4201`) bit 7 transitions and the Super Scope's trigger — neither path is wired up here, only the direct `$2137` read is. The returned byte itself is genuine open bus on real hardware; `0` is as good a stand-in as any.
- **`OPHCT`/`OPVCT` (`$213C`/`$213D`)**: each a 9-bit value read as two sequential 8-bit reads (low byte first, then high byte in bit 0 — bits 1-7 of the high read are PPU2 open bus, returned as 0). Each register tracks its own low/high toggle independently; only reading `STAT78` resets both back to "low". **H is not tracked** — this renderer has no real per-dot H position (see `Venus_Memory.md` §1.5's H-blank approximation for the same underlying gap) — so `ReadSLHV` always latches `_latchedH = 0` rather than fabricating a conversion from `LineCycles`.
- **`STAT77` (`$213E`)**: `trm-vvvv` — Time Over (§6.1), Range Over (§6.1), master/slave select (always 0 — real consoles almost universally read this as 0 too), PPU1 version in the low nibble (`1`, matching most real units).
- **`STAT78` (`$213F`)**: `flupvvvv` — interlace field (bit 7, tracked via `FieldParity`, toggled once per frame at scanline 0), NTSC/PAL region (`0` = NTSC, matching the 262-scanline timing used throughout this project), PPU2 version in the low nibble. Reading this also resets the `OPHCT`/`OPVCT` high/low toggle back to "low", per documented hardware behavior.

---

## 10. `SETINI` — stored honestly, not fully implemented

`SETINI` (`$2133`) writes are stored so games that write it don't break, but most of the modes it controls aren't rendered differently yet:

- **Interlace** — would need a real alternating-field frame count, not just the `FieldParity` flag that already exists.
- **Pseudo-hi-res** (bit 3) — **is** implemented (§8, Phase A).
- **EXTBG** (bit 6) — **is** implemented (§3.4).
- **Overscan** — would need the renderer's fixed 224-line `ScreenH` to become dynamic.

None of the remaining gaps are things a stored byte alone can fix; flagged honestly here rather than silently pretending the register does something it doesn't yet.

---

## 11. Debug-only diagnostic code

`Renderer.Debug.cs`'s `DumpBlackBg1Tiles` is a large, hyper-specific diagnostic built during a single past investigation (tracing why certain BG1 tiles rendered as pure black) — it replays BG1/BG2/BG3's exact per-pixel math at a sampled black pixel to narrow down which stage (palette, tilemap, compositing) produced it. Left largely as-is (dense inline narration, not summarized here) since it's a self-contained investigative tool tied to its own specific reasoning, not general hardware documentation — a future investigation of the same shape can read it in place as a worked example of how to bisect a rendering bug this way.

---

## 12. Known open issues: Zelda: A Link to the Past

### 12.1 Stuck overworld subscreen ("yellow bar")

**Current status: unpatched.** Two attempted fixes were tried and both reverted - the running build has this bug present, exactly as it is on an unmodified ROM. This section documents the investigation findings so a future attempt doesn't have to re-derive them; it is deliberately *not* a description of code that exists right now (see `EmuSen_Games_Tested.md`'s Fair-category entry for this game, which is the up-to-date pointer to this doc).

**Symptom:** a solid horizontal band of yellow appears across the overworld, most visible while walking along the x-axis. The band's row on screen shifts as the camera scrolls, which was the first clue it wasn't static corruption but a fixed piece of tilemap data intersecting whatever row BG1's Y-scroll happens to put on screen.

**Confirmed mechanism** (verified with per-pixel diagnostics — `Renderer.Scanline.cs`'s `ColorMathBlendLogging`/`ColorMathBlendScanline` and `Renderer.Backgrounds.cs`'s `[BG1TILE]` trace):
- WRAM `$1D` gets stuck at `1`. The game's own Bank00 code does `LDA $1D : STA $212D` (the `TS`/subscreen-designation register) unconditionally, every single frame — so once `$1D=1`, `TS` stays `0x01` (BG1 on subscreen) forever, with nothing in ordinary overworld gameplay ever touching it again.
- BG1's tilemap contains a repeating "torch light-circle" overlay tile (a real, intentional graphic — this is the mechanism the lamp/dark-room torch effect uses, not corrupted data). With `TS=1` it now renders on the subscreen everywhere, all the time, including outdoors.
- Color math (`CGADSUB=0x32`, ADD mode, `CGWSEL` selecting the subscreen as the blend operand) additively blends that tile's bright orange against BG2's dark green fence/bush tiles, clipping to solid yellow.

**Root trigger:** closing an item-get dialog (e.g. picking up the Lamp from a chest) runs through the same close-handler as closing the equipment/inventory menu, which unconditionally calls `JSL RestoreTorchBackground`. That routine's own comment in the public `alttp-disassembly` reference project already flags it as suspect: *"there's not even a check to make sure we're indoors."* `RestoreTorchBackground` is what sets `$1D=1`.

**The only reset path found:** one state (`$00:DA63`) in Bank00's overworld module-load state machine, which runs once during a full module transition (house interior → overworld) and re-derives `$1D` from the destination overworld area number (`$8A`): zero unless `$8A` is one of `{$00,$70,$40,$5B,$03,$05,$07,$43,$45,$47}`.

**Fix attempt 1 (reverted): clear `$1D` every frame while in the overworld module.** Broke a real, unrelated thing: `$1D` is a shared scratch variable, also used by Bank00's `PaletteFilter_WishPonds`/`PaletteFilter_Crystal` (subscreen/color-math setup for their own effects) and, apparently, by the storm-intro sequence's rain overlay. A user report that a fresh, lamp-skipping save had lost its rain entirely was reproduced by this fix and confirmed gone once reverted.

**Fix attempt 2 (reverted): edge-triggered instead of continuous** — only run the same allow-list check on the exact frame WRAM `$10` (module index) transitions to the overworld from something else, mirroring exactly when the game's own `$00:DA63` state would run, structurally unable to touch `$1D` during steady-state gameplay. This fixed the rain regression (verified: `$1D` never changes across 300 frames of movement with no house transition in the run) but live playtesting reported rain appearing "for a brief second when walking outside and then disappearing" - a *different* symptom from the original missing-rain regression, and not something the edge-triggered logic should be able to cause (it can only act once, at a transition, never during ordinary walking). Whether that report was this fix's own side effect, the pre-existing separate rain bug (§12.2 below) manifesting differently, or something else was never resolved before the user asked to revert and re-approach differently.

**Open question — not resolved:** whether real hardware also gets stuck here (i.e. this is a genuine, obscure vanilla ALTTP bug, and EmuSen would be correctly replicating it) or whether EmuSen additionally fails to run the `$00:DA63` reset state on the house-exit transition (a second, EmuSen-specific bug stacked on top of the first). A save state taken "just outside the house" (immediately after leaving, before any further movement) already showed `$1D` stuck despite `$8A` not being in the allow-list above — which is what the reset state should have zeroed if it ran, but this doesn't distinguish "real hardware gets stuck here too" from "EmuSen skips the reset."

**If revisiting:** get a direct, live-tested confirmation of the edge-triggered fix (attempt 2) actually correcting `$1D` on a real house-exit transition before deciding whether the "rain flickers" report was caused by it — headless-harness input replay could not reliably land Link on the door's one-tile collision column, so this was never directly confirmed either way; testing interactively (real-time door alignment, easy to eyeball and correct) would settle it. If that holds up, the remaining risk is validating it doesn't reintroduce the "appears then disappears" rain symptom under real, extended play - a single scripted repro isn't enough to trust here given how narrowly attempt 1's regression was missed initially.

### 12.2 Missing rain overlay (separate)

Found while verifying §12's attempt-2 fix, on a fresh save sent by the user ("skipped the lantern, went right outside"): the overworld storm-intro rain simply doesn't render on this save, and never starts even after 90+ warm-up frames. A later live-play report described a related but distinct symptom on a *different* save: rain visible for a moment after leaving the house, then gone.

Confirmed **not** caused by §12's fixes (reproduces identically on the commit before either existed). Root-caused as far as: ALTTP draws rain as individual raindrop tiles scattered through BG3's tilemap (VRAM `$C000`, sourced from a `$7F2000` WRAM buffer per the disassembly's own RAM map). Diffing the full 8KB BG3 tilemap between a raining save and a non-raining save shows the raining save has real raindrop tile indices (e.g. `0xBC`, `0xBD`) at scattered positions that the non-raining save instead fills with a plain, high-priority filler tile (`0x7F`). Every other PPU register checked (BGMODE, TM/TS, CGADSUB, CGWSEL, window registers, BG3 scroll, CGRAM) is byte-identical between the two saves — so this isn't a compositing bug, the raindrop tile data itself was simply never DMA'd into VRAM for whatever moment the non-raining save was captured at, and/or gets cleared during camera scrolling without being refreshed. Not yet investigated further: what WRAM flag gates that DMA transfer, or why it didn't fire/persist for some saves' contexts. Possibly related to the per-scanline indirect-HDMA VRAM streaming this same bridge/rain scene was already implicated in for an earlier, separate (fixed) HDMA-ordering bug (`Venus_Memory.md` §3.2) — not confirmed.

`SnesDebugTarget.GetVideoRegisters()` now exposes BG3/BG4 scroll, `BG3SC`/`Bg34Nba`, and the window mask/position registers (`W12Sel`/`W34Sel`/`WObjSel`/`Wh0-3`) from this investigation, for whoever picks it up next.
