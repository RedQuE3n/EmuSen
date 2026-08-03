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

### 2.1 VRAM data ports — the address counter

`$2116`/`$2117` (VMADD) hold a **word** address; `$2118`/`$2119` (VMDATA) are the low/high byte ports. `VMAIN` (`$2115`) bit 7 selects which port write advances the counter (clear → `$2118`, set → `$2119`), and bits 0-1 the step (1 / 32 / 128 words). A full 16-bit upload therefore writes `$2118` then `$2119` with bit 7 set, advancing one word per pair.

### 2.2 VRAM address translation (`VMAIN` bits 2-3)

Bits 2-3 **rotate the low bits of the word address the data ports dereference**, leaving the counter itself alone:

| Mode | Rotation | Effect |
|---|---|---|
| 0 | none | address used as-is |
| 1 | low 8 bits | `aaaaaaaa BBBccccc` → `aaaaaaaa cccccBBB` |
| 2 | low 9 bits | `aaaaaaaB BBcccccc` → `aaaaaaac cccccBBB` |
| 3 | low 10 bits | `aaaaaaBB Bccccccc` → `aaaaaacc cccccBBB` |

The point is bitplane interleaving. A tile's bitplane pairs sit 8 words apart in VRAM, so uploading one tile normally means either 8 separate address writes or a stride the DMA unit can't express. With rotation on, **consecutive** counter values land 8 words apart, so a game can push tile data through a single linear DMA and let the PPU scatter it into place. `TranslatedVramWordAddress()` in `Ppu.cs` implements the rotation; both the write (`WriteVMDATAL`/`H`) and read (`ReadVMDATAL`/`H`) paths use it, while `VramStep()` still advances the untranslated counter.

**Fixed bug: translation was ignored entirely, scrambling any upload that used it.** Bits 2-3 were never decoded — `WriteVMAIN` stored the byte, and the data ports indexed `_vramAddr * 2` directly. Every byte of a translated upload therefore landed at the wrong VRAM address.

Reported as "FFMQ has severe graphical artifacts." That game sets `$2115 = 0x84` (mode 1) exactly once during startup, uploads its BG3 character data, then returns to `0x80`. The upload covers word addresses `$3000-$37FF` — byte range `$6000-$6FFF`, which is precisely BG3's character base (`BG34NBA = 0x03`). The result was BG3's 2bpp font/window tileset written as noise, which the overworld then displayed as full-screen garbage over a correctly-rendered BG1, because Mode 1's BG3-priority bit (`BGMODE = 0x09`) puts high-priority BG3 tiles above every other layer.

The tell that separated this from a renderer fault: BG1 was pixel-perfect under `layers bg1` while `layers bg3` was pure noise, and BG3's tilemap decoded cleanly (a uniform fill of tile 254, palette 3, priority 1 — a deliberate "blank" fill) while the *tile data* that fill pointed at was garbage. When the map is sane and the tiles are not, the fault is upstream in the upload path, not in tilemap addressing or compositing.

Of the 37-ROM sample, **FFMQ is the only game that uses translation at all** — which is why this survived so long. Verified with `framesum` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.21): all 37 ROMs byte-identical across 600-frame boot windows, with FFMQ's own gameplay window the single digest that moved. Pinned by `EmuSen.WiseMan/Ppu/VramAddressTranslationTests.cs`; 10 of those 21 tests fail against the old code.

### 2.3 The VRAM read latch (`$2139`/`$213A`)

Reading VRAM does **not** return the byte at the live address. The PPU holds a 16-bit read latch, and the ports serve that latch:

| Event | Effect |
|---|---|
| Write `$2116`/`$2117` | address set, then latch ← VRAM[address] (no step) |
| Read `$2139` | return latch low byte; if `VMAIN` bit 7 **clear**, latch ← VRAM[address] **then** address += step |
| Read `$213A` | return latch high byte; if `VMAIN` bit 7 **set**, latch ← VRAM[address] **then** address += step |

The ordering is the whole point: the reload samples the **current** address and only afterwards does the counter advance, so a read lags one word behind the counter. That is what makes the conventional "set address, dummy read, then read for real" idiom land on the addressed word, and equally why a game that reads *twice* and keeps the second value also gets the addressed word. `FetchVramReadLatch()` in `Ppu.cs` performs the reload and goes through the same §2.2 rotation the write path uses.

**Fixed bug: there was no latch — reads were served straight from `Vram[_vramAddr * 2]`, so every read came back one word too late.**

Reported as "confetti-looking artifacts on the ground and on the door" in Super Metroid's Crateria landing site, which cleared after leaving the room and returning. The room's BG1/BG2 character data (byte `$0066-$4FFF`) had 1385 wrong bytes, sparse — one to four bytes inside otherwise-correct tiles, concentrated where the correct value was `$00`, which is why the damage read as bright specks scattered over terrain that should have been flat.

The tell was that the corruption was *sparse inside* tiles rather than at tile granularity: a partial or misaddressed upload damages whole tiles, so byte-level speckle inside good tiles means the upload was reading part of its own input wrong. A `watch add VRAM 3B8 4 write` (§3.3) named the writer directly — `PC=$80B404`/`$80B40A`, alternating `STA $2118`/`STA $2119` — and `disasm CpuBus 80B3D0` showed that routine is Super Metroid's decompressor servicing a **back-reference**: it sets `$2116`, reads `$2139` twice, and writes the result back out. Every back-reference therefore copied the word *after* the one it wanted, which is exactly the measured signature (`corrupt[i] == correct[i+2]` held across 49 of 141 non-trivial multi-byte runs, against 1 for the opposite direction). Literal copies came from ROM and stayed correct, which is why only back-references were damaged.

Why a door transition "fixed" it: that path re-uploads the tileset over the top, and with the corrupt bytes overwritten the room renders clean. **A save state captured while the artifacts are visible still contains the corrupt VRAM**, so it keeps showing them after this fix — the state stores the damage, not the cause. Reproducing from a fresh boot (load `SAMUS DATA`, which returns Samus to the ship save station) is what separated a live emulation bug from a state-serialization one.

Two games in the 37-ROM sample read VRAM back. Super Metroid's boot window never reaches a room load, so its 600-frame digest is unchanged; **Secret of Evermore's is the single digest that moved**, and its title screen went from a doubled, garbled logo over asymmetric architecture to clean — a second bug the same fix closed. The other 36 are byte-identical. Pinned by `EmuSen.WiseMan/Ppu/VramReadLatchTests.cs`. Note `VramAddressTranslationTests.Reads_use_the_same_translated_address_as_writes` had to change with this: it populated `Vram` *after* writing `VMADD`, which under a real latch means the prefetch samples empty VRAM.

### 2.4 The vertical scroll is offset by one scanline

**The BG row shown on display row `N` is `N + 1 + VOFS`, not `N + VOFS`.** Real hardware's first *visible* scanline is line 1, not line 0 — line 0 is a non-rendered dummy line — so a renderer that indexes its output rows 0-223 has to add the 1 back when it converts an output row into a BG row. This applies to the vertical scroll only; **`OBJ` Y coordinates are not offset this way** (a sprite at `Y=0` really does start on display row 0, which is why `EvaluateSpritesForScanline` compares against a plain `py`), and the horizontal scroll has no equivalent adjustment. The asymmetry is genuine hardware behaviour, not a modelling convenience — it is why game code so often writes a `VOFS` one less than the row it actually wants at the top of the screen.

**The bug this fixed.** `RenderBg1`-`4` computed `wy = (samplePy + vofs) % mapH`, one row short. For most scenes that is invisible: the whole BG is one pixel low, and adjacent tilemap rows usually look near-identical. It becomes glaring only where display row 0 lands on a **wrap boundary** — and Donkey Kong Country 2's Pirate Panic does exactly that, parking BG2 at `VOFS = $3FF`, one less than the map height, precisely so hardware's `+1` resolves it to BG row 0. Without the `+1` the top row wrapped backwards to the map's *last* row instead, painting one scanline of unrelated tilemap across the top of the screen. The same off-by-one produced a one-pixel seam partway down DKC2's Gangplank Galleon map (display row 159, where that screen's own scroll hit its wrap), and left the top row of Donkey Kong Country's mode-select screen black.

Both artifacts are gone with the `+1`; every other screen simply moves up one pixel, which is the correction. Pinned by `EmuSen.WiseMan/Ppu/BackgroundVerticalOffsetTests.cs` (all three of its cases fail against the old formula).

**Mode 7 was deliberately left alone.** `RenderMode7`/`RenderMode7Bg2Extbg` still use a plain `py` for their `sy`. The same hardware rule almost certainly applies to `M7VOFS` too, but Mode 7's affine transform produces no wrap seam to test against — a one-line shift in a smooth transform is not visually distinguishable from a correct one — so changing it would have been an unvalidated edit. Chrono Trigger's Mode 7 opening (its boot runs `BGMODE=07` for ~745 frames, the cheapest Mode 7 repro in the local ROM set) renders correctly as-is. Open question, not a resolved one.

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

### 4.1 The sub screen uses the *same* order (`CompositeScreen`)

There is only one compositing routine, `CompositeScreen`, called twice per scanline: once with `TM` into `_mainLineBuf`/`_mainLineLayer`, once with `TS` into `_subLineBuf`/`_subLineLayer`. Real hardware has one priority resolver and simply feeds it a different layer-enable byte per screen, so anything true of the main screen's ordering (the per-mode BG/OBJ interleave above, both priority passes per BG, BG3-forced-top) is equally true of the sub screen. Keeping them as one function is what makes that structurally guaranteed rather than something two parallel code paths have to remember.

The sub screen used to be a hand-written, much shorter stack of its own, and it was wrong in three separate ways, all of which silently dropped pixels from the color-math blend operand:

- **It only ever ran the low-priority pass for BG1, BG2, and BG3.** Every high-priority BG tile — the tilemap entry's priority bit set — was missing from the sub screen entirely. BG4 was the sole exception (it did get both passes).
- **It ignored the BG mode.** One fixed BG4→BG3→BG2→BG1 order was used for every non-Mode-7 mode, rather than the mode's real interleave.
- **It drew all four OBJ priority levels last, after every BG.** Sprites therefore always won on the sub screen regardless of their OAM priority.

**Found via** Zelda: A Link to the Past, the throne-room corridor inside Hyrule Castle: the ornate doorframe over the north door rendered as a solid black silhouette (correct shape, no color). That room composites almost entirely through color math — `TM=0x16` (BG2/BG3/OBJ), `TS=0x01` (BG1), `CGWSEL=0x02` (sub screen is the blend operand), `CGADSUB=0x20` (backdrop only participates) — so nearly every visible pixel is `CGRAM[0]` plus whatever BG1 puts on the sub screen. The doorframe's BG1 tilemap entries have the priority bit set; with only the low-priority sub-screen pass running, they never rendered, the sub screen fell back to the fixed color (black, `$2132` all zero), and `black + black` gave a black doorframe-shaped hole. Anything wearing this signature — right silhouette, no color, in a scene that blends against the sub screen — is worth checking against this class of bug first.

---

## 5. Color math

- **Per-layer participation** (`LayerParticipatesInColorMath`) reads CGADSUB's per-layer enable bits directly, with one hardware-specific restriction: **OBJ color math on the main screen only applies to sprites using OBJ-relative palettes 4-7** — palettes 0-3 stay opaque even with CGADSUB's OBJ-math bit set, letting a game keep some sprites always-opaque and others blending purely by palette choice, no extra per-sprite flag needed. Confirmed via the SNESdev wiki's Sprites page. The sub screen has no such restriction (any sprite there can act as a blend source) — already naturally handled since this check is only ever consulted for the *main* screen's winning layer.
- **Half-color-math-disabled-for-fixed-color quirk**: half color math is disabled when blending against the fixed color (explicitly, or via an empty sub-screen pixel) *unless* the backdrop layer itself is enabled in CGADSUB. Implemented directly in `RenderScanline`'s final blend step (`isFixedColor` check).
- **Blend clamping**: `BlendColors` clamps subtract to 0 and add to 255 per channel, and **always forces alpha back to 255** regardless of math result — alpha dropping to 0 here was a real regression (documented in-place as "CRITICAL... if this drops to 0, Yoshi goes invisible").
- **Sub-screen backdrop fallback**: wherever nothing is drawn on the sub screen, real hardware uses the Fixed Color register (`$2132`) as the fallback rather than CGRAM color 0 — modeled directly as `subBackdrop` in `RenderScanline`.

### 5.1 Skipping the sub-screen composite when nothing can read it

`RenderScanline` composites two full screens per scanline: the main screen driven by TM, then the sub screen driven by TS. The sub screen's result is only ever consumed in three places, all in the final blend:

- `_subLineBuf[px]` as the blend operand — reached only when `participates && mathAllowed` **and** CGWSEL bit 1 (use-sub-screen) is set.
- `_subLineLayer[px]` for the half-color-math quirk — same guard, and short-circuited away entirely when CGWSEL bit 1 is clear.
- `_subLineBuf[srcX]` in the pseudo-hi-res pass-through, which bypasses color math completely and reads the buffer unconditionally.

So the sub screen is genuinely needed only when **pseudo-hi-res is active**, or when **all three** of these hold: CGWSEL bit 1 set, CGWSEL bits 4-5 ≠ 3 (`never`), and CGADSUB's low six participation bits nonzero. `subScreenUsed` encodes exactly that, and the `CompositeScreen` call for TS is skipped when it is false. The backdrop pre-fill of `_subLineBuf`/`_subLineLayer` still runs unconditionally, so the buffers hold defined values either way.

This is an output-identity optimization, not an accuracy change. It was verified by digesting every frame of a 400-frame window across 20 commercial ROMs and a 2500-frame window across 8 color-math-heavy ones (`framesum`, Debugging Tools Reference §3.21) — byte-identical before and after — and pinned by four cases in `EmuSen.WiseMan/Ppu/SubScreenCompositeTests.cs`, including the pseudo-hi-res case the guard must *not* skip.

**Why it matters.** A game that leaves TS equal to TM while running with color math off pays for a second full layer composite that is thrown away every scanline. Rocky Rodent does exactly this — mode 2, `TM=$17 TS=$17 CGWSEL=$30 CGADSUB=$00`, unchanged across every one of 13,440 scanlines sampled — which made it the most expensive title in the sample by a wide margin, entirely from work with no visible effect. Measured on gameplay:

| Build | Before | After |
|---|---|---|
| Release | 5.59 ms/frame (179 fps) | 3.51 ms/frame (285 fps) |
| Debug | 23.74 ms/frame (42 fps), 300/300 frames over budget | 15.21 ms/frame (66 fps), 0/300 over budget |

Games that really use color math keep their sub-screen cost (SMW 0.75 ms, DKC 0.76 ms, FFVI 0.35 ms) — the guard only removes work that was already being discarded. Games that leave TS at 0 were never paying it in the first place.

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

### 6.4 The OAM address is reloaded every vblank

`$2102`/`$2103` are a **latch**, not the write pointer itself. Hardware keeps a separate running internal address that `$2104` writes advance, and **reloads it from the latch at the start of vblank** — unless the PPU is in forced blank, in which case the running address is left alone. `Ppu` models this as `_oamAddrLatch` (what the registers hold) and `_oamAddr` (the running pointer); writing either address register sets both, and `ReloadOamAddressForVBlank()` is called from `VenusCore`'s vblank block.

**This was previously missing** — `$2102`/`$2103` wrote straight into the single running address and nothing ever reloaded it. That is invisible to a game that re-sets the address before every upload, which is why it survived so long.

**The bug it caused**: Donkey Kong Country builds its OAM in WRAM at `$00:0200` and DMAs all 544 bytes to `$2104` once per frame — and, after setup, **never writes `$2102`/`$2103` again**, trusting the vblank reload to put the pointer back at 0. Two stray bytes had reached `$2104` beforehand, so without the reload every frame's DMA landed two bytes late: OAM held the WRAM buffer shifted right by 2, every sprite read its neighbour's fields, and Donkey Kong rendered as fragments scattered across the screen. Diagnosed by dumping `OAM` and `WRAM $200` side by side — they were byte-identical apart from the 2-byte skew. Pinned by `EmuSen.WiseMan/Ppu/OamAddressReloadTests.cs`.

**Timing is approximate**: the reload happens on the vblank scanline boundary rather than at H=10 within it (anomie's docs put it there; Mesen carries a `TODO` about the same detail). Nothing observed so far depends on the sub-scanline placement.

**Known separate inaccuracy, deliberately left alone**: §6.2's `FirstSpriteIndex` computes `(_oamAddr & 0xFE) >> 1`, but `_oamAddr` is a *byte* address and OAM entries are 4 bytes, so the sprite index should be `(_oamAddr & 0x1FC) >> 2`. Not changed here — it only affects priority rotation, no game currently under test exercises it, and it is unrelated to this fix.

---

## 7. Windowing

Two windows (W1: `$2126`/`$2127`, W2: `$2128`/`$2129`) can each be enabled/inverted per layer via the per-layer select registers (`W12SEL`/`W34SEL`/`WOBJSEL`), with configurable combine logic (OR/AND/XOR/XNOR, from `WBGLOG`/`WOBJLOG`) when both windows are active for a layer. The color-math window is a *separate* mask, using `WOBJSEL` bits 4-7 and `WOBJLOG` bits 2-3 rather than a specific layer's own bits — `CGWSEL` bits 6-7 additionally invert the final window output for the main/sub screen independently. `WindowMask` (§7.1) and `IsColorMathWindowMasked` implement these separately since they read from different bit groups and serve different purposes (visibility mask vs. color-math eligibility mask), even though the window-position/invert/combine logic itself is structurally identical between them.

### 7.1 The window test is decoded once per scanline, not once per pixel

`IsWindowMasked` used to take `(ppu, layerId, isMainScreen, px)` and be called from inside every per-pixel loop in the renderer — seven call sites across `Renderer.Backgrounds.cs`, `Renderer.Mode7.cs` and `Renderer.Sprites.cs`. Everything it did except the final window-range comparisons was **constant for the whole call**: the `layerId` switch selecting `W12SEL`/`W34SEL`/`WOBJSEL`, the `TMW`/`TSW` enable-bit test, the nibble decode into the four enable/invert flags, and the "neither window enabled" early-out. All of it re-ran up to 256 times per layer per screen per scanline to produce the same answer.

It is now the `WindowMask` struct: `WindowMask.For(ppu, layerId, isMainScreen)` decodes once and captures `WH0`–`WH3`, and `Masked(px)` does only the per-pixel part. `Active` is false whenever the layer is unwindowed, which lets a caller skip the per-pixel test for the entire scanline. Capturing the window positions at the top of the loop is safe for the same reason `BgLineCache` is: no CPU executes during a `RenderScanline` call, so PPU register state cannot change mid-scanline.

**Measured, three runs each way, 600 frames, Release:** SMW 3.48 → 3.33 ms (−4.4%), DKC 2.69 → 2.58 (−4.2%), both with non-overlapping ranges. LttP was unchanged (1.567 → 1.563) and CT was inconclusive against its own noise. That shape is expected: where a layer has no window enabled, the old code already early-outed cheaply, so there was nothing to win. The gain is real only on titles that actually use windows. All 12 ROMs' `framesum` and `audiosum` digests are byte-identical.

It does **not** move the throttled picture — KBL3 at 33% went 17.70 → 17.78 ms, inside noise. §13.4's target is `mainComposite` itself, and this was not that.

**Do not "simplify" this by testing every pixel.** Computing the mask unconditionally, including for transparent pixels (common for parallax, sky and background gaps), was tried when `BgLineCache` was first written and made PPU rendering *slower than before the cache existed at all*. The `if (pixel != 0)` guard around the background call sites is load-bearing.

### 7.2 CGRAM is converted to output colours once per palette change, not once per pixel

`SnesColor(lo, hi, brightness)` unpacks a 15-bit BGR entry and scales each channel by brightness. It was called **per opaque pixel per layer per screen** — up to ~229,000 times a frame — to produce one of only **256 distinct results**, because its inputs are always a CGRAM entry and a brightness that is constant for the whole scanline.

`_paletteColor[256]` now holds the converted colours and `PaletteColor(cgIdx)` indexes it. `EnsurePaletteColors` rebuilds when any of three things is true:

- **`Ppu.CgramChanged`** — set by `WriteCGDATA`, the single register path all CGRAM writes (including DMA and HDMA to `$2122`) go through.
- **the brightness changed** — `INIDISP`'s low nibble, which HDMA drives per scanline for fades.
- **`py == 0`** — unconditionally, once a frame.

**That third condition is load-bearing, not belt-and-braces.** Save-state restore, the debug tools and most of this project's own PPU tests assign `ppu.Cgram[...]` directly and never touch `WriteCGDATA`, so the dirty flag alone would leave them rendering stale colours. Rebuilding at the top of every frame bounds any missed invalidation to a single frame, and it is what makes the existing test suite — which pokes CGRAM directly and then calls `RunFrame()` — keep passing. A direct poke *mid-frame* is still not seen until the next one; nothing in the emulator does that, and `PaletteColorCacheTests` pins the behaviour either way.

**Measured, three runs each, 600 frames, Release:** SMW 3.33 → 3.12 ms (−6.3%), DKC 2.58 → 2.40 (−7.0%), CT 2.39 → 2.27 (−4.8%), LttP unchanged. Throttled to 33%, SMW went from **30/600 frames over budget to 4/600**. All 12 ROMs byte-identical on `framesum` and `audiosum`.

**Why this was the right target, and the painter's algorithm was not.** `mainComposite` covers two things: each layer's per-pixel *decode*, and the per-layer *composite pass* that writes it into the line buffer. Skipping every composite pass entirely (measured by gating them off) left SMW at 0.88 ms of 1.15, DKC 0.88 of 1.20, KBL3 1.12 of 1.45 — so **the decode is ~75% and the overdraw only ~25%**. Removing the painter's algorithm perfectly would win about 4%, roughly what §7.1 got. The colour conversion was inside the 75%.

---

## 8. Pseudo-hi-res vs. true hi-res (Modes 5/6)

**Phase A (implemented)**: real pseudo-hi-res column interleave (`SETINI` bit 3, outside Modes 5/6) — confirmed via the SNESdev wiki's Backgrounds page ("the main-screen appears on every even column, and the sub-screen appears on every odd column"). This does *not* change how BG1-4/OBJ render — they still compute a normal 256-wide `_mainLineBuf`/`_subLineBuf` exactly as in standard resolution (Mesen's own hi-res output confirms real hardware interleaves two already-independently-rendered screens here, it doesn't render new content at 512-wide density). Only the final compositing step in `RenderScanline` needs to know about the wider 512-pixel output.

**Phase B (implemented)**: Modes 5/6 now render genuinely distinct content at 512-dot horizontal density in `RenderBg1`/`RenderBg2`. Three pieces:

- **A tilemap cell spans 16 dots, not 8**, supplied by a *pair* of 8x8 tiles (N and N+1). This is the same horizontal pairing 16x16 tiles already did, so `ResolveBgTileIndex` now takes a `hiRes` flag and handles both through one `subX = (wx >> 3) & 1`. The old `ResolveHiResPairedTile` — which handed the whole main screen tile N and the whole sub screen tile N+1 — is gone; that was never how the pairing works. Note the cell is 16 dots wide in hi-res *regardless* of the tile-size bit, which only adds vertical pairing, hence `cellShiftX` being separate from the `ty` shift.
- **Each output dot samples its own BG column.** `HiResDot()` maps output pixel `px` to 512-dot column `2*px + (isMainScreen ? 0 : 1) + 2*hofs`, so the main screen takes the even dots and the sub the odd, matching the interleave below. `BGnHOFS` still counts 256-space pixels, so it is doubled into dot space. The pre-Phase-B code used `wx = px + hofs` and sampled 8 consecutive columns of one tile per 8 output pixels, dropping half of every glyph.
- **The interleave itself was already correct** from Phase A and is unchanged.

**The bug this fixed**: Secret of Mana draws its file-select menu — window frames, "GAME SELECT", the whole instruction paragraph — entirely in Mode 5. Under Phase A it rendered as sliced, half-missing glyphs ("GAME SELECT" came out as "GME SEET"). Pinned by `EmuSen.WiseMan/Ppu/HiResBackgroundTests.cs`.

**Interleave parity is worth flagging**: this document's Phase A convention (main on even columns, sub on odd) comes from the SNESdev wiki's Backgrounds page, and `HiResDot` is written to agree with it. **Mesen does the opposite** (`ApplyHiResMode`: `buffer[x<<1] = sub; buffer[(x<<1)+1] = main`). The two differ by a one-dot horizontal shift and nothing else — both produce correct, self-consistent output, and no test ROM or game screen available here distinguishes them, so the wiki's convention was kept rather than churned on a coin flip. If a hardware comparison ever settles it the other way, flipping *both* `HiResDot`'s parity and `RenderScanline`'s interleave together is the whole change; flipping only one scrambles adjacent columns.

**Still simplified in Modes 5/6**: the hi-res composite path skips windowing and colour math entirely (unchanged from Phase A), and Mode 6's offset-per-tile interacts with hi-res scroll in ways this doesn't model — `GetOffsetPerTileScroll` is applied in 256-space and then doubled.

`Renderer.FrameWidth` reports the current output width (256 normally, 512 during pseudo-hi-res) — frontends should check this rather than assuming a fixed size, since it can change frame to frame if a game toggles `SETINI` bit 3. `_screenPixels` is always allocated at the max width (512) with a fixed row stride, so a hi-res toggle never needs reallocation.

---

## 9. Status/read-only registers

- **`SLHV` (`$2137`)**: reading it latches the current H/V position into `OPHCT`/`OPVCT`. Real hardware also latches via WRIO (`$4201`) bit 7 transitions and the Super Scope's trigger — neither path is wired up here, only the direct `$2137` read is. The returned byte itself is genuine open bus on real hardware; `0` is as good a stand-in as any.
- **`OPHCT`/`OPVCT` (`$213C`/`$213D`)**: each a 9-bit value read as two sequential 8-bit reads (low byte first, then high byte in bit 0 — bits 1-7 of the high read are PPU2 open bus, returned as 0). Each register tracks its own low/high toggle independently; only reading `STAT78` resets both back to "low". **H is not tracked** — this renderer has no real per-dot H position (see `Venus_Memory.md` §1.5's H-blank approximation for the same underlying gap) — so `ReadSLHV` always latches `_latchedH = 0` rather than fabricating a conversion from `LineCycles`.
- **`STAT77` (`$213E`)**: `trm-vvvv` — Time Over (§6.1), Range Over (§6.1), master/slave select (always 0 — real consoles almost universally read this as 0 too), PPU1 version in the low nibble (`1`, matching most real units).
- **`STAT78` (`$213F`)**: `flupvvvv` — interlace field (bit 7, tracked via `FieldParity`, toggled once per frame at scanline 0), NTSC/PAL region (bit 4), PPU2 version in the low nibble. Reading this also resets the `OPHCT`/`OPVCT` high/low toggle back to "low", per documented hardware behavior.

  **The region bit is real, not hardcoded.** It reports `Ppu.IsPal`, which `VenusCore.LoadRom` sets from the cartridge header's country byte alongside the matching 262- or 312-scanline timing — see `Venus_CPU.md` §8.5c for the whole region mechanism and `Venus_Memory.md` §2.5 for the country-byte classification. It used to be a hardcoded `0`, which is what made a PAL cartridge stop on Nintendo's *"This game pack is not designed for your SUPER FAMICOM or SUPER NES"* lockout screen: the game reads this bit, sees NTSC, and refuses to run. `IsPal` is `[SkipInState]` — it's derived from the ROM, which is always reloaded before a state load, so storing it would only create a way for a state file to contradict the cartridge it was made from.

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

---

## 13. Frame cost — what `RunFrame()` actually costs, and why the old numbers were wrong

`VenusCore` exposes a per-phase breakdown of the last completed `RunFrame()` — `LastFrameCpuSpc700Ms`, `LastFramePpuMs`, `LastFrameHdmaMs`, plus a sub-breakdown of the PPU phase sourced from `Renderer` (`LastFrameObjEvalMs`, `LastFrameBlendMs`, `LastFrameMainCompositeMs`, `LastFrameSubCompositeMs`). Both frontends print all of it in their FPS readout.

It is timed at **per-scanline** granularity, deliberately. Timing every `Cpu.Step()` would add `Stopwatch.GetTimestamp()` overhead of the same order as the work being measured — a fast interpreter's per-instruction cost — and distort the numbers this exists to produce. `CpuSpc700` bundles CPU and SPC700 stepping into one phase for the same reason; splitting them means timing individual `Step()` calls again.

### 13.1 Current measured cost

Headless, 3000 frames from cold boot (so title, menus, and each game's own attract-mode gameplay demo), .NET 10 Release:

| ROM | mean | p99 | max | cpu+spc700 | ppu | main | sub | objEval | blend |
|---|---|---|---|---|---|---|---|---|---|
| SMW | 3.05 | 6.14 | 8.00 | 0.66 | 2.37 | 1.14 | 0.75 | 0.05 | 0.36 |
| LttP | 2.49 | 4.35 | 6.46 | 0.68 | 1.81 | 1.10 | 0.18 | 0.08 | 0.37 |
| SM | 1.76 | 4.19 | 10.19 | 0.70 | 1.05 | 0.45 | 0.05 | 0.06 | 0.45 |
| DKC | 2.30 | 3.12 | 4.46 | 0.45 | 1.84 | 1.14 | 0.34 | 0.06 | 0.21 |
| FFVI | 2.84 | 3.13 | 15.24 | 0.49 | 2.33 | 1.76 | 0.33 | 0.04 | 0.27 |
| CT | 2.15 | 6.24 | 8.39 | 0.55 | 1.59 | 1.03 | 0.22 | 0.06 | 0.22 |
| SMW2 | 3.19 | 8.25 | 15.72 | 0.98 | 2.20 | 1.20 | 0.60 | 0.04 | 0.25 |
| KSS | 4.70 | 6.27 | 8.86 | 3.22 | 1.47 | 1.12 | 0.00 | 0.07 | 0.31 |
| SoM | 1.40 | 3.97 | 6.68 | 0.49 | 0.90 | 0.51 | 0.02 | 0.04 | 0.21 |
| EB | 1.96 | 6.23 | 8.68 | 0.66 | 1.28 | 0.68 | 0.18 | 0.05 | 0.30 |

The NTSC frame budget is 16.64 ms (§`Venus_CPU.md` §8.5b). Mean cost is **1.4–4.7 ms**, i.e. 3.5–12× real time, and no ROM sustains anything close to the budget. The isolated `max` outliers (FFVI 15.2, SMW2 15.7) are single frames, almost certainly GC pauses — the p99 column is 3.1–8.3 ms.

**SA-1 titles are the heaviest class, and the table above understates them** because it measures boot and attract mode. Measured directly in a level (KBL3, Kirby's Dream Land 3, holding Right, 900 frames): **5.0 ms mean, 0/900 over budget** — `cpu+spc700` 2.9, `ppu` 2.0, `mainComposite` 1.6. That is still a 3.3x margin, but the phase split is inverted relative to every other ROM here: the CPU phase dominates, because an SA-1 game steps two 65816 cores and `perf`'s coprocessor line reads 100% of a full-rate frame every frame. KSS's 3.22 ms `cpu+spc700` in the table is the same effect. Anything that makes the 65816 interpreter slower costs these games double, and they are the first to fall under 60 when it happens — see `EmuSen_Debugging_Tools_Reference_v5.md` §3.20.

Within the PPU phase, per-scanline main-screen compositing is the single largest item everywhere. `objEval` is negligible (≤0.08 ms). Sub-screen compositing is near-zero for games that don't really use it and only becomes significant in SMW (0.75) and SMW2 (0.60) — §5.1's skip is doing its job.

### 13.2 The "~17–18 ms during gameplay / ~13 ms of PPU" figures are historical

Comments in `VenusCore.cs` and `Renderer.Scanline.cs` used to state that `RunFrame()` itself took ~17–18 ms during real gameplay, with ~13 ms of that in the PPU, and framed the phase counters as a live investigation into that. Those numbers predate the fixes that investigation produced — chiefly BG1-4's redundant double-decode (`RenderBg1-4`'s `BgLineCache`) — and are roughly **6–10× off** the current cost. They were left in place long enough to send a later performance investigation down a dead end. The comments now point here instead; treat the table above as the number, and re-measure rather than trusting prose.

### 13.3 Two costs the frontends pay that headless benchmarking does not

Worth knowing before blaming the core for a frontend-side frame time:

- **Rewind capture.** `MainWindow` constructs its `RewindBuffer` with `Enabled = true` unconditionally, so every 4th frame is a full reflection-based `SaveState` (~600 KB) plus an XOR delta. Measured at **~0.20–0.25 ms per frame amortized** across SMW/LttP/SM/DKC/FFVI/KSS — real, but not a suspect.
- **Frame hand-off.** `SubmitFrame` is an `Interlocked.Exchange` of a reference plus a coalesced `Dispatcher.UIThread.Post`; the actual upload happens once on the UI thread and stale frames are dropped by design. Not a per-frame cost on the emulation thread.

The frontend readout separates `run Xms` (the `RunFrame()` call alone) from `total Yms` (wall clock per frame, *including* the pacing sleep). A `total` at ~16.6 ms with `run` at ~3 ms is the 60 Hz pacer working correctly, not a slow emulator. Only a `run` figure near the budget indicates a core problem.

### 13.4 What it costs on the machine we are actually targeting

§13.1 answers "does it hold 60 on a 7700X" — yes, everywhere. From 2026-08-03 the goal changed to **running on a low-end x86-64 laptop**, roughly a third of that machine's single-thread performance, and that is a different answer. Measured with `--throttle` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.42), same scripts, `--nobattery`:

| ROM | scene | 100% | 33% | over budget at 33% | 33% cpu | 33% ppu | 33% mainComposite |
|---|---|---|---|---|---|---|---|
| KBL3 | Grass Land, holding Right | 4.93 | 17.82 | **168/300** | 10.33 | 7.43 | 5.96 |
| SMW | boot + attract | 3.42 | 13.02 | 30/600 | 3.79 | 9.09 | 4.43 |
| DKC | boot + attract | 2.68 | 14.72 | **169/600** | 3.53 | 11.07 | 7.32 |

All three miss the budget on a real fraction of frames, and the SA-1 title misses on more than half. The throttle overstates slowdown somewhat (§3.42's second limit), so treat the *shape* as the finding rather than the exact milliseconds.

The shape is unambiguous: **`mainComposite` alone costs 4.4–7.3 ms of a 16.64 ms budget** — a third to nearly half the entire frame, in one function family, on every ROM regardless of which phase dominates overall. It is the largest single item at full speed too (§13.1) and it grows fastest under constraint. Anything spent on `objEval` (≤0.26 ms even throttled) or HDMA (≤0.12) is spent in the wrong place.

Two structural facts frame what to do about it. `CompositeScreen` is a painter's algorithm: every enabled layer writes every pixel and later layers overwrite, so a four-layer mode pays up to 4× the necessary writes. And the whole console runs on one thread (`MainWindow.axaml.cs`'s `EmuSen-Emulation`), so Venus today needs one fast core and cannot use a second — the wrong shape for cheap hardware, which has several mediocre cores instead. Reducing the work comes before parallelising it; there is no point spending two cores on writes that should not happen.

### 13.5 Save states older than the current field layout resume into a dead machine

Noted here because it silently invalidates any attempt to benchmark real gameplay by resuming a state. `StateSerializer` has no field-name tagging (`EmuSen_Save_States.md` §1/§3), so a `.state` written before a field was added or reordered still loads without error and produces a machine that runs but renders nothing — `LastFramePpuMs` collapses to ~0.06 ms while the renderer's own `LastFrame*` properties keep reporting their last real values, which is what the inconsistency looks like from the outside. The three states in `Usr/Home/Saves/Save States` are all in this condition. Verify a resumed state by dumping the framebuffer before trusting any measurement taken from it.

---

## 14. Mode 0 — each layer has its own 32-colour CGRAM block

Mode 0 is the only mode where all four backgrounds are 2bpp, and it is the only mode where a tilemap entry's 3-bit palette field is **not** an index into the bottom of CGRAM. Each layer gets its own quarter of the palette:

| Layer | Palettes | CGRAM entries | Base added by `Mode0PaletteBase` |
|---|---|---|---|
| BG1 | 0-7 | 0-31 | 0 |
| BG2 | 8-15 | 32-63 | 32 |
| BG3 | 16-23 | 64-95 | 64 |
| BG4 | 24-31 | 96-127 | 96 |

`BgCgramIndex` took `entryPalette * 4 + pixel` for every 2bpp layer in every mode, so in Mode 0 all four layers read BG1's 32 colours. Every other mode is unaffected — `Mode0PaletteBase` returns 0 outside Mode 0, and the 4bpp/8bpp paths never had a per-layer base to begin with. Matches Mesen's `RenderMode0`, which passes exactly these four constants as its `basePaletteOffset` template argument (`SnesPpu.cpp`).

**How it surfaced.** Yoshi's Island's intro switches to Mode 0 with `TM = $08` (BG4 only) for scanlines 152-197 to draw the story text over the bottom of the screen — see `Venus_SuperFX.md` §10.5. The tilemap there uses palettes 6 and 7, which resolve to CGRAM 120-123 and 124-127, both white-on-transparent. Read without the base, they landed on CGRAM 24-31 and the text came out dark red. **The glyph shapes were already correct at that point**, which is the useful diagnostic: a palette-indexing bug leaves geometry intact and only moves colour, so "right shape, wrong colour" points at the CGRAM index and not at the tile decode.

This was invisible until the HDMA fix in `Venus_Memory.md` §3.2a, because nothing had ever driven this core into Mode 0 with a non-zero palette field before. Two independent bugs stacked on the same symptom — worth remembering when a fix improves an artifact without clearing it.
