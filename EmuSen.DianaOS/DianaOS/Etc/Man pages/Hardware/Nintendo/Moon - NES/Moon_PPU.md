# Moon (NES) — PPU (2C02)

Covers `Cores/Nintendo/Moon - NES/Ppu/`: `Ppu.cs` (state, registers, the VRAM bus) and `Ppu.Render.cs` (scroll advance and scanline composition).

---

## 1. Granularity

Rendering happens once per scanline, from `EndScanline(line)`, called by `MoonCore` after that line's CPU time. There is no per-dot pipeline, no shift registers and no background/sprite fetch schedule.

`Moon_Core.md` §3 states what that costs and why it is a rewrite rather than a patch. The short version: a mid-line write to `$2005`/`$2006`/`$2001` applies to the whole line, and sprite 0 reports at end-of-line rather than at the pixel. Splits done at a scanline boundary — the normal technique — work.

### 1.1 What `EndScanline` does per line

- **0-239**: render the line, advance the scroll (§2.2), then tick the mapper's scanline counter if rendering is on.
- **241**: set the vblank flag. `NmiOutput` becomes true if `PPUCTRL` bit 7 is set, and the CPU latches that as an edge.
- **261** (pre-render): clear vblank, sprite 0 and overflow; advance the scroll; copy the vertical scroll bits; count the frame.

---

## 2. The loopy registers

`V`, `T`, `FineX` and `WriteToggle` are the real internal registers, not a synthesized scroll pair. Both `V` and `T` are 15 bits laid out as `yyy NN YYYYY XXXXX` — fine Y, nametable select, coarse Y, coarse X.

Modelling them directly rather than storing "scrollX/scrollY" is what makes `$2006` and `$2005` interact correctly, and they interact constantly: writing `$2006` twice loads `T` and then copies it wholesale into `V`, which is how a game sets the VRAM pointer *and* the scroll position with the same two writes.

### 2.1 Writes

| Register | First write (`w=0`) | Second write (`w=1`) |
|---|---|---|
| `$2005` | `FineX` = value & 7, coarse X = value >> 3 | fine Y and coarse Y from the value |
| `$2006` | high 6 bits of `T` | low 8 bits of `T`, then `V = T` |

`$2000` writes nametable-select straight into `T` bits 10-11. Reading `$2002` clears `WriteToggle`, which is why a game reads it before a two-write sequence.

### 2.2 Scroll advance

Per line, in this order: increment fine/coarse Y in `V`, then copy the horizontal bits (`0x041F`) from `T` back into `V`. On the pre-render line the vertical bits (`0x7BE0`) are copied too.

Coarse Y wraps at **29**, not 31, flipping the vertical nametable bit as it goes — rows 30 and 31 of a nametable are the attribute table, not tiles. A game that deliberately sets coarse Y to 30 or 31 gets the no-flip wrap instead, which is modelled.

Coarse X wraps at 31 by flipping the horizontal nametable bit rather than carrying into coarse Y.

### 2.3 The `$2007` read buffer

A read of `$2007` below `$3F00` returns the *previous* byte read and then buffers the new one, so a game reads one access behind and must dummy-read once after setting the address. Palette reads are immediate, but they still refill the buffer from the nametable address mirrored underneath.

### 2.4 Palette holes

Palette RAM is 32 bytes, but `$3F10`, `$3F14`, `$3F18` and `$3F1C` are not storage — they are holes onto `$3F00`, `$3F04`, `$3F08` and `$3F0C`. `PaletteOffset` implements this as `if ((index & 0x13) == 0x10) index &= 0x0F`, which catches exactly those four and leaves `$3F11` and friends alone.

This matters visibly: the backdrop colour is written to `$3F00` by most games and read through the sprite mirror by the compositor.

---

## 3. Composition

### 3.1 Background

`RenderBackground` walks 33 tiles, starting at `-FineX`, so the leftmost tile can be partly off-screen. Per tile it fetches the nametable byte from `0x2000 | (v & 0x0FFF)`, the attribute byte from `0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)`, and the two bitplanes at the background pattern base, then advances coarse X with the same wrap the hardware uses.

The attribute byte covers a 4×4-tile area in four 2×2 quadrants; the shift is picked by coarse X bit 1 and coarse Y bit 1.

`_bgLine[x]` stores the palette entry, with colour index 0 meaning transparent — the compositor needs background opacity, not just colour, for both sprite priority and sprite 0.

### 3.2 Sprites

Up to eight per line, in OAM order, with the ninth setting the overflow flag and stopping evaluation. The real chip's overflow bug (which evaluates diagonally and can set the flag spuriously) is **not** modelled.

OAM byte 0 holds the sprite's top scanline *minus one*, so a sprite is on line `L` when `L - Y - 1` is within its height. In 8×16 mode the tile's bit 0 selects the pattern table and the pair is even-aligned, with the bottom half taken from the next tile index.

### 3.3 Priority and sprite 0

The first opaque sprite pixel found in OAM order wins. It is drawn over the background when its priority bit says "front" or when the background pixel is transparent; otherwise the background wins.

Sprite 0 hit sets when sprite 0's opaque pixel meets an opaque background pixel, with both layers enabled and `x != 255`. The rightmost-pixel exclusion is real hardware behaviour, not an off-by-one guard.

Left-column masking (`PPUMASK` bits 1 and 2) is applied to both layers and to the hit test.

### 3.5 Colour

`NesPalette` is the standard 64-entry NTSC table as RGB triples. Grayscale (`PPUMASK` bit 0) is applied by masking the colour index with `0x30`. **The three emphasis bits are not applied** — a game that tints the screen red on damage will render untinted.

Fast-forward (`SkipRendering`) skips only the framebuffer writes. Sprite evaluation, the sprite 0 hit and every flag still run, because a game polling sprite 0 would otherwise hang forever while fast-forwarding.

---

## 4. Not implemented

- Per-dot timing and the fetch pipeline (§1).
- The odd-frame dot skip on the pre-render line.
- Emphasis bits (§3.5).
- The sprite overflow hardware bug (§3.2).
- The `$2003`/`$2004` OAM corruption quirks.
- The open-bus decay latch behind unreadable register bits.
- PAL.
