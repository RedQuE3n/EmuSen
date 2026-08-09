# Mercury (Game Boy) — the LCD controller

*Written 2026-08-08, when the PPU replaced the frame-boundary VBlank stand-in that
`Mercury_Core.md` §3 described. DMG only; the colour extensions are Phase D and
this page will grow a §7 when they land.*

---

## 1. What kind of model this is

The controller is modelled at **two different granularities on purpose**, and the
split is the single most important thing to understand before reading the code.

- Its **clock** is per T-cycle. `Ppu.Tick()` is called once per T-cycle from
  `MemoryBus.StepOneCycle`, so LY, the mode, and the STAT line all move at the
  resolution hardware moves them at. A game polling `$FF41` for mode 0 sees the
  mode change at the dot it changes on.
- Its **renderer** is per scanline. Nothing is drawn until mode 3 ends, at which
  point the whole line is composited at once from whatever the registers hold at
  that instant.

This is deliberately the same division Moon settled on for the 2C02
(`Moon_PPU.md` §1), and for the same reason: almost everything a game observes
about the PPU it observes through the *clock* — STAT, LY=LYC, the mode bits —
while almost everything it observes about the *renderer* it observes one line at
a time, by changing SCX or SCY in an hblank interrupt. Splitting the two buys
per-line raster effects at a fraction of the cost of a pixel FIFO.

What it does not buy is a mid-scanline register change. A game that alters SCX
partway along a visible line gets the value that happened to be live at the end
of mode 3 applied to the whole line. No commercial DMG title is known to need
more, and the FIFO that would be required is a rewrite of §4 and §5, not an
addition to them. **Revisit only against a game that demonstrably breaks**, not
on principle.

### 1.1 Who decides a frame is over

`MercuryCore.RunFrame` used to spend a fixed 70224-cycle budget and call that a
frame. It now returns when the PPU sets `FrameComplete`, which happens when LY
wraps 153 → 0.

The budget is still there, and it is no longer the pacing authority — it is a
watchdog for one specific case. **A disabled LCD does not advance at all**: with
LCDC bit 7 clear, `Tick` returns immediately, LY is parked at 0, and no frame
would ever complete. Without the budget, a game that blanks the screen to do a
long VRAM upload would hang the frontend rather than the game.

The prediction this retires is worth recording, because it was wrong. The first
design kept the cycle budget as the authority on the grounds that the budget and
the PPU's own period are both exactly 70224, so the two could not drift. They
can. Every time a game toggles the LCD off and on, the PPU restarts LY at 0 at
whatever point in the budget window that happened, and the phase between the two
clocks shifts permanently. The frame handed to the host would then be cut across
the middle — a tear on every frame thereafter, from a single LCD toggle at boot.
One clock has to be the authority, and it has to be the one the pixels come from.

## 2. The mode machine

Each of the 154 scanlines is 456 T-cycles, and the visible 144 of them are
divided three ways:

| Mode | Name | Length | What it means |
|---|---|---|---|
| 2 | OAM scan | 80 | Hardware picks the line's sprites; OAM is unreadable |
| 3 | Drawing | 172 + fine scroll | Pixels are pushed; OAM *and* VRAM are unreadable |
| 0 | HBlank | the remainder | Everything is readable again |

Lines 144-153 are mode 1, vblank, throughout.

### 2.1 Where the boundaries are decided

`DrawingEnd` is latched when mode 3 begins rather than recomputed as the line
runs, because SCX can be written again before the line ends and hardware has
already committed to the penalty by then.

### 2.2 The fine-scroll penalty, and the two that are missing

Mode 3's length is `172 + (SCX & 7)`. Hardware discards `SCX & 7` pixels off the
left edge of the first tile, and it pays for them in real time, so a scrolled
background shortens the hblank that follows.

Two further penalties hardware charges are **not** modelled: the window costs
about 6 dots on the line it first appears on, and each sprite on a line costs
between 6 and 11 depending on how it overlaps the fetches. They are omitted
because the renderer is per line — nothing Mercury draws depends on them, so all
they would change is the dot at which mode 0 begins. That is observable, through
a mode-0 STAT interrupt on a line with sprites, and a game timing an HDMA against
it would notice. Nothing has yet. This is a known gap with a known shape, not an
oversight.

### 2.3 Switching the LCD off

Clearing LCDC bit 7 parks LY at 0, drops the mode to 0, clears the STAT line, and
whitens the panel. Real hardware also holds the LCD's own output off, which is
what the whitening stands in for.

Two hardware behaviours around re-enabling are not modelled: the first frame after
the LCD comes back on is not displayed, and that frame's first line is shorter
than 456 dots. Both would only matter to a game that measures the boundary it
re-enables on, and the second is below this model's timing resolution anyway.

## 3. STAT

### 3.1 What is stored and what is recomputed

Only the five source-enable bits are kept in a field. The mode in bits 1-0 and
the LY=LYC flag in bit 2 are hardware state, not stored bits, so `ReadStat()`
recomputes them on every read and bit 7 reads back set because it is unwired.
Storing them would create two copies of the same fact and a way for them to
disagree.

### 3.2 The interrupt is a level, not four events

This is the part that is easy to get wrong, and it is worth stating as a rule
rather than as a mechanism: **the four STAT sources are OR'd onto one line, and
the interrupt is requested on that line's rising edge.** A source going high
while another source already holds the line high requests nothing.

The consequence is that a game enabling both the hblank and the LY=LYC sources
gets *fewer* interrupts than a naive per-source implementation would deliver, not
more. `UpdateStatLine` is therefore called from every place that can move any of
the four inputs — mode changes, LY changes, and writes to STAT or LYC — and it is
the only place `Interrupt.LcdStat` is ever requested.

The famous corollary, the "STAT write bug" on a DMG (a write to STAT briefly
reading as though every source were enabled, spuriously firing the interrupt), is
not modelled. It is a documented DMG defect that a handful of games rely on
accidentally rather than deliberately.

## 4. Background and window

### 4.1 Addressing

The two tile maps live at `$9800` and `$9C00`; LCDC bit 3 picks the background's
and bit 6 the window's, independently. Tile *data* is shared: LCDC bit 4 selects
between `$8000` with an unsigned index and `$9000` with a signed one. The signed
mode is the one that catches people — index `$FF` is the tile *before* `$9000`,
not the last tile after it.

### 4.2 The window's own line counter

The window does not read the map row that the background would. It has an
internal counter that starts at 0 at the top of each frame and advances **only on
lines the window actually drew**. A window that first appears at LY 100 starts
from its own row 0, not row 100.

Two conditions gate it, and they are separate:

- **WY** is compared against LY once per line, at the end of the OAM scan, and a
  match *latches* for the rest of the frame. A window enabled after its WY has
  already gone past does not appear until the next frame.
- **WX** positions it: the window's first column lands at `WX - 7`. Values below 7
  push it off the left edge, which is legal and is how games slide it in.

The latch initially lived in the code that begins a new scanline, and line 0 was
therefore never tested — the first line of a frame is reached by reset or by the
LCD being switched on, not by a line boundary. A window with `WY = 0` silently
never drew. That is what the `The_window_draws_from_wx_minus_seven_over_the_background`
test in `MercuryPpuTests` was written against, and why the check now sits at the
mode 2 → 3 transition, which every line passes through.

### 4.3 LCDC bit 0

On a DMG, clearing bit 0 blanks the background *and* the window to white — it does
not display colour 0 of BGP, and it does not leave the window visible. This is one
of the two places the CGB reinterprets a DMG register rather than extending it
(bit 0 there means "background keeps its priority over sprites"), so this
paragraph will need a colour counterpart in Phase D.

## 5. Sprites

Ten per line, from 40 in OAM, 8x8 or 8x16 by LCDC bit 2. An 8x16 sprite's tile
number has its low bit forced clear, and rows 8-15 fall into the following tile
naturally because the address arithmetic never needed a special case.

### 5.1 Two orderings, not one

Sprite handling involves two independent orderings and conflating them produces a
bug that only shows on crowded lines:

1. **Which ten are drawn** is decided in OAM order. The first ten entries that
   overlap the line win, and a sprite further down OAM cannot buy a slot by being
   further left.
2. **Which of the drawn ten wins a pixel** is decided by X on a DMG: the leftmost
   sprite wins, and equal X falls back to the lower OAM index.

The implementation selects, then sorts. The sort is an insertion sort and it has
to be **stable**, which is the whole reason it is written out rather than handed
to `Array.Sort` — the tie-break on equal X is exactly the case an unstable sort
gets wrong, and it is common, because sprites in a row are routinely given the
same X for one frame while a formation assembles.

The CGB replaces rule 2 with plain OAM order. Rule 1 is the same on both.

### 5.2 Losing a pixel is not the same as not claiming it

The subtle one. Hardware resolves sprite-versus-sprite **first** and
sprite-versus-background **second**. Once the highest-priority sprite with a
non-transparent pixel is chosen, the other sprites at that pixel are gone; only
then does that one sprite's priority bit (OAM attribute bit 7) decide whether it
loses to an opaque background pixel.

So a sprite can lose its pixel to the background *without* the sprite behind it
getting to show through. `_spriteClaimed` exists for exactly this: a pixel is
marked claimed when a sprite wins it on sprite priority, before the background
comparison, and a later sprite skips it either way. Drawing back-to-front and
letting later sprites overwrite earlier ones — the obvious implementation — gets
this case wrong and only this case, which is why it is worth a named array and a
test.

Attribute bit 7 is only ever consulted against background colour *index* 0-3, never
against the shade BGP maps it to. A background palette that maps colour 1 to white
still hides a sprite behind it.

## 6. Colour

Four shades, and Mercury prints them as neutral greys: `$FF`, `$AA`, `$55`, `$00`.

The green cast people associate with the console is a property of the DMG's
reflective panel, not of the hardware's output — the same ROM in a Game Boy Pocket
is grey and in a Super Game Boy is whatever palette the SNES side chose. Emitting
greys keeps the frame buffer a faithful record of what the PPU produced and leaves
tinting to a shader, where a user can turn it off. It also makes a frame buffer
directly comparable against another emulator's without a colour transform in the
way, which is what the reference probe under `EmuSen.WiseMan/Reference/` would
need if Mercury ever grows a differential of its own.

The choice is a presentation decision, and it does **not** generalise to Phase D:
CGB palettes are real RGB555 values a game chose, and those get converted, not
substituted.

## 7. Access blocking

*Rewritten 2026-08-09. This section spent Phases B through D arguing that blocking
should be **absent**, and named the single condition that would reverse it. That
condition was met, so the argument is preserved below rather than deleted — it was
correct for as long as its premise held, and the shape of it is reusable.*

Hardware makes VRAM unreadable during mode 3 and OAM unreadable during modes 2 and
3; reads return `$FF` and writes are dropped. **Mercury now implements this**, plus
the OAM lock an in-flight DMA holds (`Mercury_Memory.md` §6).

Blocking applies to the CPU's view only. The renderer indexes `Vram` directly and
is unaffected, which is asserted rather than assumed — a test drives a full frame
with the background enabled and checks the pixels still come out, because a
plausible way to get this wrong is to block the PPU from its own memory.

### 7.1 The argument that used to be here, and why it was right

The old text ran:

> `Cpu.Step` runs a whole instruction and *then* the bus is ticked by its full
> cost. At the moment a memory access is decoded, the PPU is therefore still at the
> position it held at the start of the instruction — up to about 24 T-cycles behind
> where hardware would have it. Enforcing the blocking against a PPU that is
> systematically early would drop writes that hardware accepts: a game that writes
> to VRAM on the first instruction after a mode-0 STAT interrupt is the common
> case, and it would see mode 3.
>
> The trade is therefore between two failure modes, not between accuracy and
> laziness. Not blocking loses the games that *read* `$FF` during mode 3 to detect
> timing, which is a small and mostly homebrew set. Blocking loses graphics updates
> in ordinary commercial titles. The second is worse.
>
> **This changes when the bus becomes cycle-granular, and not before.**

Every clause of that held. The value in keeping it is the method: the section did
not say "not implemented", it said *which* failure each choice buys, *which* is
worse, and *what single change* flips the answer. When `Mercury_Cpu.md` §3 landed,
no fresh judgement was needed — the condition had been written down in advance, and
the work became mechanical.

The general form worth reusing: **a simplification is worth documenting as a
prediction with a trigger, not as an apology.** A trigger that later fires converts
a debate into a task.

### 7.2 What blocking does not fix

- **The renderer is still per scanline** (§1). Blocking changes what the CPU may
  do during mode 3; it does not make the PPU consume its writes pixel by pixel, so
  a mid-line palette or scroll change still takes effect for the whole line.
- **The mode-3 length is still `172 + (SCX & 7)`.** Sprites extend it on hardware
  and do not here, so the window in which blocking applies is slightly too short on
  a busy line.
- **`oam_bug` is still expected to fail.** The DMG's OAM corruption defect is a
  separate behaviour from access blocking, and Mercury does not reproduce it —
  `Mercury_HardwareTests.md` §4.

## 8. State

The PPU is a field of `MemoryBus`, so `StateSerializer` walks it as part of the
bus with no special handling. `FrameRgba`, the two per-line scratch arrays and the
back-reference to the bus are `[SkipInState]`: the buffers are regenerated by the
next line drawn and the back-reference is re-wired by the constructor
(cases 2 and 3 in `StateSerializer`'s own header).

Adding these fields moved Mercury's save-state version to **2**. Version 1 files
are rejected rather than migrated; nothing outside the test suite ever wrote one.
