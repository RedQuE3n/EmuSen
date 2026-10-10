# Moon (NES) — PPU (2C02)

Covers `Cores/Nintendo/Moon - NES/Ppu/`: `Ppu.cs` (state, registers, the VRAM bus) and `Ppu.Render.cs` (scroll advance and scanline composition).

---

## 1. Granularity

**The clock is per dot; the renderer is still per line.** Those are separate things and the split is deliberate.

`Ppu.Step(dots)` is called from `MemoryBus.Tick` — three dots per CPU cycle on NTSC — so the PPU advances inside the CPU's own bus cycles rather than in a scheduled lump at the end of a scanline. `Ppu.Timing.cs` is modelled directly on Mesen's `NesPpu::Exec`: `Cycle` runs 0-340, `Scanline` runs 0-261, and every timed event fires on the dot hardware fires it on.

What that buys, and what it does not:

| | State |
|---|---|
| VBlank set/cleared, NMI asserted | **exact dot** |
| Sprite 0 and overflow cleared | **exact dot** |
| Scroll copies (257, 280-304), Y increment (256) | **exact dot** |
| Odd-frame dot skip | **implemented** |
| PPU address bus / A12 for mapper IRQs | **per fetch** (`Moon_Memory.md` §4.6a) |
| Pixel output | still composed once per line, at dot 256 |
| Sprite 0 *hit* | **exact dot**: the dot its pixel is output on (§3.4) |
| Mid-line `$2005`/`$2006`/`$2001` writes | still apply to the whole line |

So raster splits done at a scanline boundary work, splits done mid-line do not, and everything that depends on *when* a flag changes rather than *what a pixel is* is now right.

### 1.1 The frame is driven by the PPU

`MoonCore` no longer counts scanlines to decide when a frame ends. The PPU sets `FrameComplete` when it wraps past the pre-render line and the CPU loop returns on it. A scanline boundary is still computed, but it now only paces how many CPU cycles run before the loop checks back — it is not the PPU's clock any more. That is what left `EmuSen.Crystal` with nothing to schedule and got it folded into the core (`Moon_Core.md` §2.2).

### 1.2 Reading `$2002` next to the vblank flag

A `$2002` read lands on a dot, and which dot decides what happens. Read on scanline 241 dot 0 — the dot *before* the flag would be set — and the flag never appears at all: `SuppressVBlank` eats the upcoming set, and the read itself returns it clear. This is the race `ppu_vbl_nmi` spends several of its tests on, and it is unreachable at scanline granularity because the read and the set land in the same indivisible step.

### 1.3 `RenderV`, and a bug worth remembering

The per-dot fetches walk `V` along the line exactly as hardware does — 32 coarse-X increments between dots 1 and 256, two more at 328 and 336. The line renderer, which is not a pipeline, needs the address the line *starts* at.

Taking `V` directly was the first attempt and it broke three of the four test games outright: by dot 256 `V` has advanced a full line, so every row rendered from the wrong place. `RenderV` latches the line's base at dot 257 (after `CopyHorizontal`, and after `CopyVertical` on the pre-render line), which is precisely the value hardware starts the next line from. Output went back to byte-identical with the pre-per-dot renderer.

The general shape is worth keeping: adding real hardware behaviour underneath an approximation can break the approximation, because the approximation was quietly relying on the old behaviour being absent.

### 1.4 Cost

1.35 ms/frame headless on the development machine — about **12x realtime**, roughly 8% of the 16.6 ms budget at 60.1 Hz. Per-dot stepping was the one part of this work with a real risk to the low-end-laptop goal (`EmuSen_Performance.md`), so it was measured rather than assumed.

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

### 2.5 The open-bus latch

The PPU's data bus is a capacitor, not a register. Whatever the CPU last drove onto it lingers, and **every bit the PPU does not actively drive reads back from that charge** rather than as zero. `Ppu.OpenBus` models it, refreshed through the single `RefreshOpenBus` helper so no path can forget to.

What drives what:

| Access | Bits driven | What the rest reads |
|---|---|---|
| Write to any `$2000`-`$2007` | all 8 | — |
| Read `$2002` | 7-5 (the flags) | bits 4-0 from the latch |
| Read `$2004` | all 8 | attribute bits 2-4 have no storage and read 0 |
| Read `$2007`, below `$3F00` | all 8 (the read buffer) | — |
| Read `$2007`, palette | 5-0 | bits 7-6 from the latch |
| Read `$2000`/`$2001`/`$2003`/`$2005`/`$2006` | none | the whole byte from the latch |

**Decay is per bit, not per byte, and that distinction is load-bearing.** `blargg`'s `ppu_open_bus` fails on a whole-latch model at its seventh check — "reading `$2002` shouldn't refresh low 5 bits of decay value" — because a `$2002` read recharges only the three bits it drives while the low five keep ageing. `OpenBusDecay` is therefore eight counters, decremented once a frame by `DecayOpenBus()`, and a bit clears when its own counter runs out. `OpenBusDecayFrames` is 36 (~600 ms), the figure hardware is measured at; the suite only requires "zero by one second", so the exact value is not critical, but a whole-byte timer is.

This is why `$2002`'s low bits stopped reading zero. Games poll `$2002` constantly, so it was worth confirming no regression: A Boy and His Blob, SMB2, SMB3 and SMB+Duck Hunt all still render real content.

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

Sprite 0 hit sets when sprite 0's opaque pixel meets an opaque background pixel, with both layers enabled and `x != 255`. The rightmost-pixel exclusion is real hardware behaviour, not an off-by-one guard. The hit is not decided here but on its own dot, by §3.4.

Left-column masking (`PPUMASK` bits 1 and 2) is applied to both layers and to the hit test.

### 3.4 Sprite 0 hit, on its pixel's dot

Pixel `x` of a visible line is output on dot `x + 1`, and the hit flag rises on that dot. Until 2026-10-04 the flag rose at dot 256 for any `x`, when the line was composed; a game that times a raster split from the hit then wrote its scroll up to 165 dots late, which is what drew Super Mario Bros.' slivers under its status bar (`Moon_Native.md` §3.14).

`DetectSprite0Hit` (`detect_sprite0_hit` in MoonRT) runs on dots 1–256 of a visible line until the flag is set, and only on the dots within sprite 0's eight columns, which keeps its cost to a few percent (`Moon_Native.md` §3.14.4). On dot `x + 1` it tests, with the registers as they stand on that dot:

- both layers on, `x != 255`, and for `x < 8` both left-column bits;
- sprite 0 on the line by the same rule as sprite evaluation, and opaque at column `x − OAM[3]`;
- the background opaque at `x`: stream position `x + FineX` from `RenderV`, the line's start, with fine X as it is on the dot, since hardware's fine X picks its shifter tap at once;
- the tile under that position loaded while rendering was on.

**The last rule is the shifters'.** Hardware's background shift registers neither shift nor reload while rendering is off (NES_MiSTer's `bgp_en`, `ppu.sv:1531`). Tile `k` of the line's 33 is reloaded on dot `8k − 7`, and tiles 0 and 1 on dots 329 and 337 of the line before. If rendering was switched on after that reload, the tile is not in the shifters and cannot hit. `Ppu.RenderingSince`, the PPU clock of the last write to `$2001` that turned rendering on from off, holds this; it is written in the state's tail (version 6). What the stalled shifters really hold is not modelled: such a tile counts as transparent.

The reads are peeks. `IMapper.PeekChr` returns the byte `ReadChr` would without its side effects, which only MMC2's latch has, so the detector cannot move a latch that the line's composition at dot 256 then reads differently. The cost is that a tile in a latch-switched bank is tested with the latch as it stands, not as the fetches 16 dots ahead would have left it.

The line is still composed at dot 256, and it no longer touches the flag.

### 3.5 Colour

`NesPalette` is the standard 64-entry NTSC table as RGB triples. Grayscale (`PPUMASK` bit 0) is applied by masking the colour index with `0x30`. **The three emphasis bits are not applied** — a game that tints the screen red on damage will render untinted.

Fast-forward (`SkipRendering`) skips only the framebuffer writes. Sprite evaluation, the sprite 0 hit and every flag still run, because a game polling sprite 0 would otherwise hang forever while fast-forwarding.

---

## 4. Not implemented

- The per-dot *fetch pipeline* and shift registers — the clock is per dot, the renderer is not (§1). Sprite 0's hit is decided per dot (§3.4), but from `RenderV` and peeks, not from shifters.
- Emphasis bits (§3.5).
- The sprite overflow hardware bug (§3.2).
- The `$2003`/`$2004` OAM corruption quirks.
- PAL.

## 7. `NesPpuWriteLogging` — every $2000-$2007 write, with the dot it landed on

The NES counterpart of Venus's `PpuActiveDisplayWriteLogging`, added 2026-08-07 because a mid-frame scroll split cannot be reasoned about without knowing *when* the game wrote. Gated by `DebugSettings.NesPpuWriteLogging` behind the master switch, so `log on` in a `--commands` script windows it to the frames of interest.

It emits one line per write: frame, scanline, dot, register and value. On SMB3's title screen it shows the split plainly — `$2006` twice during HBlank of the split line, `$2000` during the prefetch window, then `$2005` twice early on the next line:

```
[PPUW] f130 line 193 dot 272  $2006 = $0B
[PPUW] f130 line 193 dot 284  $2006 = $00
[PPUW] f130 line 193 dot 326  $2000 = $A8
[PPUW] f130 line 194 dot  15  $2005 = $00
[PPUW] f130 line 194 dot  33  $2005 = $EF
```

That trace is what turned "the floor is in the wrong place" into "the IRQ fires six scanlines early", because the line number in the first column *is* the answer — see `Moon_Memory.md` §4.6b.

**A caution about the renderer this feeds.** `RenderV` is latched at dot 257, so a write landing in dots 258-320 does not reach the line hardware would apply it to. That is a real deviation, and moving the latch to 321 was tried on 2026-08-07: it changed no pixel on SMB3 and was reverted rather than kept unproven. If a game turns up whose split lands one line off, this is the first thing to re-examine. Super Mario Bros.' split did land one line off (2026-10-04), and the latch was not the cause: the writes came after dot 257 because the sprite 0 hit they wait for rose at dot 256 instead of its pixel's dot. With the hit on time they land near dot 100, and the latch never comes into it (§3.4, `Moon_Native.md` §3.14).
