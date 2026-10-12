# Diamond_GPU — the GPU, stage 4

*Written 2026-10-11, at stage 4.* The record of Diamond's GPU (`src/gpu/` in the crate `diamond`): what was built, the
rules each part follows and where each rule came from, the drawing-time model and its fit, the gate program by
program, the other programs run, and what was read. `Diamond_Plan.md` §5.4 is the design this follows; §7's stage 4 row
is the gate; `Diamond_Native.md` §12 is the stage's entry in the build record, and `Diamond_Disputes.md` D-7 to D-12
are the disputes it opened.

Marks as in the plan: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44, Rust 1.98.1) with the
tool named; **[R]** read from the source named; **[A]** argued; **[C]** a choice.

---

## 1. What was built

One job a module, under `src/gpu/`:

| Module | Its job |
|---|---|
| `mod.rs` | GP0's words through the FIFO to the command processor; GP1; GPUSTAT; GPUREAD; the drawing state (GP0(E1h) to (E6h)); the DMA request |
| `vram.rs` | VRAM as the 1 MiB the `VRAM` space shows; the fill, the VRAM-to-VRAM copy and the mask bit's write rule |
| `texture.rs` | The texture page, its depth and window; the texture cache; the CLUT cache; a texel fetched through both |
| `draw.rs` | The pixel pipeline every rendering command shares: modulation, dithering, the four blend modes, the mask bit; the drawing area; the work a primitive did |
| `raster.rs` | Which pixels a triangle, a rectangle or a line covers, and the colour and texture coordinate at each |
| `display.rs` | VRAM read a line at a time into the picture the host is given |
| `timing.rs` | The drawing time in GPU cycles, and its conversion to CPU cycles |

The stage 3 stub (`devices::GpuStub`) is gone. The bus (`bus.rs`) gives the GPU its two ports, its interrupt (IRQ1),
DMA channel 2 and an event at each moment its FIFO moves; nothing else of the machine reaches into it.

**Read for this stage** is §16. No emulator's source, no firmware or firmware listing, no GPU plugin's code and no
PSX_MiSTer file was read.

## 2. The command processor and the FIFO

### 2.1 GP0's words

psx-spx gives the FIFO as 16 words and the commands' lengths ("GPU Command Summary", "GPU Render ... Commands",
"GPU Memory Transfer Commands") [R]. Diamond's model [C]:

- **A word's meaning is parsed as it arrives**, at the FIFO's tail (`Parser`): a command's first word, one of its
  parameters, a CPU-to-VRAM transfer's data, a polyline's further colour or position, or its terminator
  (`(word & F000F000h) == 50005000h` on the first word of a vertex after the second [R "GPU Render Line Commands"]).
- **NOPs take no place**: GP0(00h), (04h)..(1Eh), (E0h) and (E7h)..(EFh) at a command's position are dropped [R "GP0(00h)
  - NOP (?)"]. GP0(03h) takes a place and does nothing [R "GP0(03h) - Unknown?"].
- **GP0(E3h) to (E5h) go through the FIFO in order**, against psx-spx's "probably executed immediately" (D-9). Acting
  on arrival clipped seven of Peter Lemon's clipped polygons with the next primitive's area; in order, his program is
  drawn as its picture and the three references draw it (§11).
- **The processor takes a word as soon as it is free**: while it gathers a command's words it is not busy, so a word
  that arrives then goes straight to it; while a command draws, arriving words wait in the FIFO, and a word arriving at
  a full FIFO holds its writer (the CPU's store or the DMA) until a place frees [C: no document says what a write to a
  full FIFO does; Peter Lemon's programs write whole scenes with no wait and are drawn whole by his console, by his
  README and pictures, which a dropped word would not allow]. *Since the cache step (2026-10-11)*, while a
  VRAM-to-CPU transfer has words left to give, the processor takes nothing from the FIFO, so a command written behind
  it waits until the CPU or the DMA has read the last word, as amidog's FIFO full test expects and DuckStation passes
  it [M, D-23]; a word that then finds the FIFO full is lost, since no place would free before the CPU reads [C].
- **A command is drawn whole into VRAM as it starts**, and the processor is then busy for its drawing time (§8). The
  plan's §5.4 says "at the time the GPU would finish it" [R]; drawing at the start was chosen [C] because the time
  depends on what was drawn (the pixels inside the area, the texture cache's misses), and the two differ only for a
  display line read while a command draws, where neither is the hardware's partial picture. The CPU can see no
  difference: VRAM reaches it only through GPU commands, which queue behind.

### 2.2 The processor's own commands

GP0(01h) clears the texture cache and the CLUT cache [R "Clear Cache", "GPU Texture Caching"]; GP0(1Fh) sets GPUSTAT
bit 24 and raises IRQ1 when it executes, not when it arrives [R "GP0(1Fh)"; A]; GP0(E1h) keeps bits 0-10, bit 11 only
while GP1(09h) allows it, and bits 12-13 on the 208-pin chip; GP0(E2h) 20 bits; GP0(E6h) 2 bits [R "GPU Rendering
Attributes"; M ps1-tests' `gp0-e1` log, ten of ten]. A textured polygon writes its page attribute into E1h's bits 0-8,
and bit 11 when allowed, and no others ([R "TPage Attribute"]; the same log); *since D-23's step* a quad's attribute
takes effect once its first triangle's three vertices are in, before the fourth arrives, as amidog's textured quads
read it in GPUSTAT after each word [M], the quad itself still drawn whole at its last word [C].

### 2.3 GPUSTAT's ready bits (D-10)

| Bit | Diamond | Source |
|---|---|---|
| 25 | by GP1(04h): 0; the FIFO not full; bit 28; bit 27 | psx-spx "GPU Status Register" [R] |
| 26 | the FIFO empty, no command half received, no transfer or polyline open, and no command drawing; *since D-23's step*, no VRAM-to-CPU transfer waiting to be read either | the console's `gpu/gpustat` log [M by ps1-tests]; amidog's VRAM READ, with DuckStation |
| 27 | a VRAM-to-CPU transfer's words ready | psx-spx [R] |
| 28 | ~~the FIFO empty~~ *since D-23's step*: the FIFO empty, and neither a polygon's or a line's vertices awaited after its command word nor a polyline open | the same log, against psx-spx's "cleared when the command execution is busy" (D-10); for polygons and lines amidog's expectations, Beetle PSX and psx-spx's sentence on them (D-10, D-23) |

The release has no `gpustat` binary (`Diamond_References.md` §1), so its four cases were written as the project's own
program, `programs/gpustat.s`, from its source: a rectangle drawn by the CPU, two by a linked list, two by blocks of one
word with directions 1 and 2, GPUSTAT read before, 300 cycles into the transfer, and after. All twelve readings of
bits 25-30 equal what the console's log passes [M, `gpustat_reads_as_the_console_passes`]. The program reads its
"during" values 300 cycles in, where the C program reads them after its function returns; any point after the second
rectangle is queued and before the first is drawn (about 9,000 cycles) gives the same bits [A]. *Since D-23's step* the
program sets the drawing area to 1023,511, as the C program's `PutDrawEnv` does: without it the rectangle drew one
pixel, and the "during" readings held only through the rectangle's former setup of 742 cycles, which D-11's refit
removed (§8) [M: with the area, twelve of twelve on both drawing times].

### 2.4 DMA channel 2

The channel's request is the GPU's (bit 25). Blocks (mode 1) and list nodes (mode 2) start only while the GPU requests,
and wait, without holding the bus, until it does [C, after psx-spx "the bit is used as DMA request in DMA Mode 2"]; a
burst (mode 0) needs no request [R "DMA Channels"]. Each word goes to the GPU at the channel's 17/16 of a cycle and may
be held back by a full FIFO; a VRAM-to-CPU word is ready at the GPU's read rate (§8), and the DMA waits for it. The
cycles a piece takes are the longer of the DMA's own and the GPU's [C]. D-5's gaps are unchanged.

### 2.5 GP1

From psx-spx's "GPU Display Control Commands (GP1)" [R]: (00h) the reset of its list, GP0(E1h)..(E6h) cleared, the
display blanked, GP1(09h) untouched; (01h) the FIFO, the command half received and the CLUT cache; (02h) IRQ1's flag;
(03h) display enable; (04h) the DMA direction; (05h) the display's start, Y nine bits on the 160-pin chip and ten on
the 208-pin; (06h)..(08h) to the video timing (`video.rs`, D-0); (09h) on the 208-pin chip only; (10h)..(1Fh) the
latch, indices 2-5 on both chips (3 and 4 masked to 19 bits on the 160-pin, 20 on the 208-pin), 7 the version (2) and
8 zero on the 208-pin only, the 160-pin's index taken modulo 8. ps1-tests' `version-detect` prints "GPU version 2" on
the default console [M]. *Since the cache step*, GP1(00h) and GP1(01h) end the command drawing, so GPUSTAT reads
`14802000h` at once after a reset during a long fill, as psx-spx gives and DuckStation, SwanStation, Beetle PSX and
PCSX-Redux read it [M, `programs/gp1-reset.s`, `Diamond_Records.md` §16.1] (VRAM keeps the whole fill, drawn as it
started, §2.1); and GP1(00h) clears the CLUT cache, as psx-spx's list for it includes GP1(01h) [R; M, amidog's CACHE
TAG]. *Since D-23's step*, as amidog's REG expects on both chips: indices 2 to 4 replace only
their 20 (or 19) bits and index 5 its 22, the latch's higher bits kept from what it held, a VRAM word read before
among them (psx-spx's "MSBs=Nothing"); and an index with no register reads zero, where psx-spx says the old value
remains [M, against the document, D-23].

## 3. VRAM and its transfers

- **The fill** masks its parameters as psx-spx's "Masking and Rounding for FILL" [R]: X in steps of 16, its width
  rounded up to 16, no fill at a zero size; the colour's top five bits a channel with bit 15 clear; no area, no mask,
  wrapping at VRAM's edges without carry ("Wrapping"). The 160-pin chip's misplaced pixels at an X ending in Fh
  ("GP0(02h) FillVram") are not modelled [C: the diagram does not say where the pixels go].
- **The copy** reads and writes row by row ~~in pieces of 16 halfwords, each piece read whole before it is written
  [C]~~, *since D-23's step* each row read whole before it is written, under the mask setting [R]. On
  `vram-to-vram-overlap` (squares of 2, 8, 15 and 16 pixels, and of 16 one pixel off in x, y and both, each copied over
  itself at the 21 displacements of -3 to 3 across and -1 to 1 down) the picture is exact [M], with either rule, since
  no row of it is wider than a piece. amidog's MOVE copies rows of 17 to 47 pixels 1 to 4 to the right over
  themselves and expects every pixel moved as it was, which pieces of 16 do not give and DuckStation does [M, D-23];
  a row is the simplest unit that meets both, and no measurement bounds the hardware's from above [C].
- **CPU to VRAM and VRAM to CPU** take psx-spx's copy masking, two pixels a word, the extra halfword of an odd count
  ignored ~~or padded~~ on a write; on a read, *since D-23's step*, the pixel after the last in its row, which
  amidog's VRAM READ expects of a one-pixel read [M, D-23; C beyond that case]; the writes under the mask setting, as
  `mask-bit`'s five cases pass on the console and on Diamond [M]. *Since the cache step*, once a transfer of two words
  or more has given its last, GPUREAD repeats the transfer's first word, and after a transfer of one word it repeats
  what it repeated before, until GP1(10h) puts a register there [M, amidog's FIFO read test, 48,128 words; PCSX-ReARMed
  for two words or more, `programs/gpuread-repeat.s`; the one-word case on amidog's expectation alone, D-23]. GP1(10h)'s
  latch keeps taking each word read, as REG requires; the word GPUREAD repeats is kept beside it.

## 4. Texturing

- **The page, the depth and the window** as psx-spx's "GP0(E1h)", "Texture Bitmaps" and "GP0(E2h)" [R]: a 4-bit texel
  the nibble of the halfword at `page_x + u/4`, an 8-bit one the byte at `page_x + u/2`, a 15-bit one the halfword at
  `page_x + u`, X wrapping at 1024 (`texture-overflow`, exact [M]); the window's `(t AND NOT mask*8) OR ((offset AND
  mask)*8)`.
- **Bit 11 of the page** puts it 512 lines down while GP1(09h) allows it on the 208-pin chip, where a retail console
  has no chip and "textures disappear" [R "GP0(E1h)", "GP1(09h)"]: a texel there reads 0000h, transparent [A]. With
  GP1(09h) clear the bit is ignored.
- **The texture cache**, 2 KiB, is psx-spx's 256 entries of 8 bytes in cache blocks of 64x64, ~~32x64~~ *since the cache
  step* 64x32, or 32x32 texels by depth, one entry of each number held [R "GPU Texture Caching"; M for the 8-bit block,
  amidog's CACHE BLOCK and Beetle PSX, against the document, D-23]. Restated in VRAM's terms, which is how Diamond
  indexes it [A]: an entry is four halfwords; a 4-bit page indexes by X's bits 2-3 and Y's bits 0-5, an 8- or 15-bit
  page by X's bits 2-4 and Y's bits 0-4; the tag is the entry's VRAM address. *Since the cache step*, setting the page
  at another X or Y, or between 4 bits and the other depths (whose entries are indexed otherwise), by GP0(E1h), a
  polygon's page attribute or GP1(00h), clears the cache; between 8 and 15 bits it does not [M, amidog's CACHE TAG and
  `programs/texture-cache.s` on Beetle PSX, D-23]. An entry is kept until another address takes
  its index or GP0(01h) clears the cache, so VRAM written behind it is not seen [R: "it is possible to abuse the texture
  cache by changing pixels in VRAM that the GPU loaded in its cache"]. No program run at this stage observes a stale
  entry; its misses are what make a textured quad five times as slow as a plain one in `bandwidth` (§8).
- **The CLUT cache** holds the palette last loaded, where from, and how many of its entries. Before each 4- or 8-bit
  primitive it is reloaded when its position differs or it holds fewer entries than the depth needs (16 or 256);
  GP0(01h) and GP1(01h) clear it [R "GPU Texture Caching", "GP1(01h)"; C for the counting rule]. `clut-cache`'s eleven
  cases (reuse without a clear, a clear, another position, the depth changes 4 to 8, 8 to 4, 15 to 4, reserved to 8, an
  E1h write between, and a CLUT across X's wrap) are exact against the console's picture [M].

## 5. The pixel pipeline

At eight bits a channel in, five out (`draw.rs`):

- **Modulation**: the 208-pin chip's `(colour × texel) / 16` with the texel at five bits, saturated at 255; the
  160-pin chip's `((colour / 8) × texel) / 2` [R "Shaded Textures"]. A raw texture is the texel shifted up three.
- **Dithering**, when GP0(E1h) bit 9 is set: the offset of psx-spx's 4x4 matrix by VRAM's (y, x) modulo 4 added, the
  sum clamped to 0..255, then divided by 8 [R "24bit RGB to 15bit RGB Dithering"]; on Gouraud-shaded and modulated
  polygons and on every line, never on rectangles or raw textures [R ibid.; C for raw textures]. Measured: the dithered
  halves of `triangle` and `uv-interpolation` exact [M].
- **The blend**, at five bits a channel with saturation: B/2+F/2, B+F, B-F, B+F/4 [R "Semi-transparency"]; for a
  texture only where the texel's bit 15 is set. `transparency`'s 128 rectangles over four backgrounds are exact [M].
- **Transparency**: texel 0000h is not drawn [R "Texture Bitmaps"].
- **The mask bit**: a pixel with bit 15 left alone when GP0(E6h) bit 1 is set; bit 15 written as the texel's, or forced
  by GP0(E6h) bit 0 [R "GP0(E6h)"].
- **The drawing area** clips every rendering command, inclusive at both corners [R "GP0(E3h)", "GP0(E4h)"];
  `clipping` exact [M]. *Since D-23's step* its Y reaches 1023 on the 208-pin chip, as psx-spx gives E3h and E4h ten
  bits there, and a line past 511 is drawn on VRAM's line modulo 512 while GP1(09h) is clear ("wrap to 0-511" [R
  "GP1(09h)"]; amidog's OFFSET [M]); with GP1(09h) set the area stops at 511, the second megabyte being absent [A].
- **The interlaced field**: in 480-line interlace with drawing to the display area prohibited (E1h bit 10 clear), the
  lines of the field being displayed are not drawn [A, from "Vertical Interlace flag DOES affect GP0 draw commands"].
  ~~No program run here clears the bit in 480-line mode.~~ *Since the cache step*, the lines left alone are those whose
  parity, counted from the display's first line (GP1(05h)), is GPUSTAT bit 31's when the command draws, which reads 0
  in vblank; and fills leave them too [M, amidog's INTERLACE, 3,060 pixels; for fills Beetle PSX and PCSX-ReARMed,
  the latter on the same lines with the display at line 0, `programs/interlace.s`, D-23].

## 6. The rasteriser (D-7)

psx-spx gives the vertices' range and the size limit, "up to <excluding> their lower-right coordinates" for polygons,
"up to <including>" for lines, the quad's split into (1,2,3) and (2,3,4), and that the interpolation is affine [R]. It
does not give the fill rule, the interpolators' precision, where they start, or the lines' stepping. Each was fitted
to ps1-tests' pictures, which are a console's VRAM (D-7):

- **Coverage.** A pixel is the point at its integer coordinates; inside a triangle when every edge function is
  positive, or zero on an edge the triangle owns, a top or a left one. On `triangle` this leaves the apex's row empty,
  one pixel on the next, and the bottom row out [M: exact].
- **Attributes.** Each of R, G, B, U and V has a gradient in x and in y of 12 fractional bits, the cross-product
  difference shifted up 12 and divided by twice the area, truncated toward zero; the value at a pixel is ~~vertex 0's,
  plus the gradients times the distance from vertex 0~~ the base vertex's, plus the gradients times the distance from
  it, plus a half, taken as its integer part. *Since D-7's referee step (2026-10-11, `Diamond_Referees.md` §4.6)* the
  base vertex is the leftmost, and of two in the leftmost column the one that follows the other in the command's
  order, as four referees of three lineages draw it; ps1-tests' pictures stay exact. On `uv-interpolation`,
  whose quads of widths 0 to 255 step U from 0 to 1, this is the only one of the candidates tried (fractional bits 8 to
  23, every bias in 1/4096 steps) that puts every row's switch where the console does; the same rule gives its 256
  Gouraud rows, plain and dithered, and `triangle`'s three, exact [M]. A quad's second triangle starts from its own
  first vertex, the quad's second [C: no console picture tells the starting vertex apart, D-7].
- **Size.** A triangle wider than 1,023 or taller than 511 between its vertices is not drawn, each of a quad's two
  judged alone [R; C for the per-triangle reading].
- **The offset**, *since D-23's step* [M, amidog's OFFSET, 23 primitives at 64 placements each]: a vertex and the
  drawing offset add in 11 bits, wrapping, so a rectangle at -992 with an offset of -1022 lands at 34; and the offset
  moves a primitive whole, each further vertex placed from the first by the distance the command gave it, which is
  also the distance the size limit judges. A triangle from 32 to 1007 under an offset of 258 spans 290 to 1265, though
  its second vertex alone would wrap to -783; one from 288 to -785 is not drawn, though an offset of 2 would bring
  its ends within 1,023 of each other once wrapped. A polyline places each vertex from the one before as drawn.
- **Rectangles** are drawn from their corner, the texture coordinate stepping one a pixel and wrapping at 8 bits;
  *since D-23's step*, unflipped, two pixels of a VRAM word take their texels from one texture cache entry: a pixel
  at an odd column whose texel lies in another entry than its left neighbour's (16 texels at 4 bits, 8 at 8, 4 at 15)
  repeats the neighbour's [M, amidog's XYFLIP, 16 commands of 8,192 cases each, at every depth, start, page and flip;
  A, that the GPU fetches a pixel pair's texels in one cache read; C, the rule beyond the even start column the test
  draws from];
  with GP0(E1h)'s flips (208-pin only) it steps backwards, and an x-flipped rectangle starts from ~~U+1~~ U with its
  low bit set (D-8, since its referee step, `Diamond_Referees.md` §4.7): on `texture-flip` every flipped rectangle's
  first column reads U+1 for its even U, then U, U-1 ..., where a y-flip starts at V [M]; an odd U starts at U, as
  Beetle PSX, the one referee that flips, draws it [M].
- **Lines** step along the major axis with both ends drawn; the minor coordinate is the nearest to the exact line, a
  tie rounded ~~toward the end on an x-major line and toward the start on a y-major one~~ *since D-23's step* toward
  the right end on an x-major line and toward less x on a y-major one, which is what the rule stated before meant for
  the lines `lines`' picture holds and what amidog's LINE and OFFSET expect of a rising x-major line and a leftward one
  (the formula had rounded a rising line's tie down, against the end) [M]; the colour by the polygons'
  12-bit gradient with a half added; an overlapping pair of vertices draws one pixel. On `lines`' 40 sloped lines, its
  128 flat and 128 shaded lines of every length to 64, plain and dithered, four polylines and a circle of 16 segments,
  exact [M]. A polyline's segments each draw both their ends, so the joint is
  drawn twice; its semi-transparent wire frames are exact, which a single joint pixel would not have been [M].

## 7. The display

Diamond_Plan.md §5.4 asks for the display "read from VRAM at its own time each line" [R]. `display.rs` reads a line
when the GPU is brought to a time after it began: before each command draws, before each GP1 write, and at each video
edge, so every line is read with VRAM as it stood when the line began, to a command's granularity, and nothing is
done per line when nothing changed [A].

- **What a line holds**: `(((X2-X1) / dots per pixel) + 2) AND NOT 3` pixels [R "GP1(06h)"] from GP1(05h)'s start, at
  15 bits (each five-bit channel expanded as `c<<3 | c>>2` [C]) or 24 (three bytes a pixel [R "Framebuffer"]); blank
  while GP1(03h) turns the display off. Peter Lemon's 24-bit transfer equals SwanStation's and PCSX-ReARMed's picture
  up to their colour levels [M].
- **The picture** is the display area at its own size, at most 640 by 576: lines Y1 to Y2 of a field, interleaved with
  the other field's in 480-line interlace, sized when a field's first displayed line is read, so a mode set in the top
  border counts [C]. `vram-to-vram-overlap`'s 640x480 interlaced screen reads as its VRAM [M].
- **The frame** now ends as a field's vblank begins, or at the field's end where the vertical range leaves no vblank
  (Diamond_Native.md §11.5 is superseded on this point) [C]. The picture is then complete, and a program that draws
  after vblank and waits for the next one, as most do, is caught between its frames; at a field's start it was caught
  mid-drawing, and `triangle`, which redraws for about 37 lines after vblank, would have been graded half white [A].
  The rate is unchanged: one frame a field.
- **The host is given** the picture's size each frame, with a 4:3 display aspect [C, the plan's §4.1], and a size
  change raises `GEOMETRY` through `core_exports!`. After GP1(00h) the display is 256 by 240 and blank.

## 8. The drawing time

psx-spx: "GPU polygon rendering time is unknown" [R "DMA Channels"]. The plan's §5.4 asks for the drawing time
charged to the GPU's clock from ps1-tests' `bandwidth`, `benchmark` and `gpustat` [R]. Costs are in 256ths of a GPU
cycle, a GPU cycle being one period of the video clock (53.693175 MHz on an NTSC console, D-0) [C], and a command's
cycles become CPU cycles exactly, the fraction carried while commands run back to back (`timing::advance`): rounding
each data word up to a whole CPU cycle had made `cpuToVram` 18% slow [M].

**The model** [C]: a command's own 2 cycles; a setup of 876 a triangle ~~and 742 a rectangle~~, *since D-11's refit*
3.09 cycles a rectangle's row drawn; 0.836 cycles a pixel
written, 1.266 a pixel read and written (semi-transparent, or the mask checked), 0.445 a texel fetched, 19.67 a texture
cache miss; the fill ~~597 plus 0.5 a row plus 2 each 16 pixels~~ *since the refit* 2.99 a row plus 2 each 16
pixels; the copy 2.13 a pixel; 1.67 a CPU-to-VRAM data word (two
written pixels); VRAM-to-CPU words ready 3.49 cycles apart after a setup of 16. A line's setup (16), a CLUT entry's
load (1) and a per-row cost (none) were chosen with no measurement.

**The fit** [M, `gpu/bandwidth`, 400 calls a row, each a 320x240 primitive or transfer; its hblanks over 400 calls]:

| Row | Console | Diamond | Diamond, every cost zero |
|---|---|---|---|
| vramToCpu | 15,770 | 15,774 | 12,116 |
| cpuToVram | 12,195 | 12,124 | 12,114 |
| vramToVram | 19,132 | 19,127 | 49 |
| FillScreen GP0(2) | 1,216 | 1,217 | 77 |
| Rectangle | 7,603 | 7,603 | 81 |
| Rectangle (semitransparent) | 11,455 | 11,462 | 82 |
| Rectangle textured, and (semi) | 15,138, 15,139 | 15,134 | 83 |
| Polygon quad | 7,727 | 7,721 | 95 |
| Polygon quad (semi) | 11,648 | 11,580 | 97 |
| Polygon quad textured, and (semi) | 39,938 | 39,975 | 60 |
| Polygon quad (1/4 off screen) | 5,845 | 5,845 | 95 |
| Polygon quad (1/2 off screen) | 3,968 | 3,968 | 96 |

Every row within 0.6%. How the figures were found: the zero-cost run gives the CPU's own share; the three quads that
differ only in how much of them is clipped give the plain pixel's 0.836 cycles and show that clipped pixels ~~and rows~~
cost nothing; the textured rows write nothing (their texels are 0000h, transparent), which separates a texel's fetch
from a write, and their misses (4,800 a 4-bit rectangle and 15,551 a 15-bit quad, counted by the cache model) give the
miss; the residuals that remained were the same for the half- and quarter-clipped quads, which differ in rows but not
in triangles, ~~so they are a setup, not a row cost~~ (see the refit below). The transfers are bound by the DMA (cpuToVram) or by the GPU's read
rate (vramToCpu) [A]. `bandwidth` joins the TTY gate with each row within 2%.

**The refit of 2026-10-11 (D-11)** [M]. Every `bandwidth` row is 320 by 240, so a fixed cost and a cost per row are
one number there: 742 cycles a rectangle is 3.09 a row, and the fill's 597 plus 0.5 a row is 2.99 a row. The
argument above that the clipped quads separate them does not hold: those quads are moved left by a quarter and a
half of the screen, so their first triangles lose rows and pixels in the same proportion at each step (480, 420 and
360 rows; 76,800, 57,600 and 38,400 pixels), and equal residuals follow whatever a row costs [A]. Charged as a setup,
the rectangle's 742 cycles made amidog's clears, one 1×1 rectangle a pixel, take some 470 frames each; charged by the
row, a 1×1 rectangle is ready 7 cycles after its word, as `programs/draw-time.s` reads it (below). Rectangles and
fills are now charged by the rows they draw; the two figures, fitted apart, agree within 0.1 cycle, which a page
change of the GPU's DRAM at each line would explain and nothing here measures [A]. `bandwidth` reads as before, row
for row within one hblank (the table above); triangles keep their setup, since nothing separates it from a row cost
either and moving it would refit the pixel's rate [C].

`programs/draw-time.s` writes each primitive by the CPU once the GPU is idle and reads timer 2 from its last word to
the first poll finding bit 26 set [M, `small_rectangles_and_fills_cost_their_rows`; the referees by
`psxreferee.py ram`]:

| Primitive | Diamond before | Diamond after | Beetle PSX | SwanStation | PCSX-ReARMed |
|---|---|---|---|---|---|
| Flat rectangle 1×1 | 475 | 7 | 10 | 16 | 14 |
| 8×1, 1×8 | 475, 475 | 7, 25 | 10, 10 | 25, 25 | 23, 14 |
| 128×1, 1×128 | 538, 538 | 70, 322 | 10, 10 | 142, 142 | 84, 14 |
| 8×8, 32×32, 64×64 | 502, 1,015, 2,635 | 52, 610, 2,284 | 10, 430, 1,978 | 52, 556, 2,131 | 49, 522, 522 |
| Fill 16×1, 16×32, 64×64 | 385, 430, 727 | 7, 106, 448 | 10, 106, 418 | 34, 205, 574 | 40, 189, 522 |
| Flat triangle of one pixel, legs of 128 | 556, 4,912 | 556, 4,912 | 10, 4,006 | 25, 4,129 | 31, 32 |
| Line of one pixel, 64 across | 16, 52 | 16, 52 | 10, 10 | 7, 34 | 14, 14 |

Three lineages read a small rectangle or fill ready within 10 to 40 cycles, and none charges it hundreds; they differ
from each other everywhere else (Beetle PSX charges nothing below some threshold, PCSX-ReARMed caps every rectangle
near 522), so they rule only that a small primitive is cheap, not the rate [A]. Their triangles are as cheap, and
Diamond's are not (above). amidog's menus now appear at frames 100 (GPU), 205 (CPX) and 229 (GTE), against 1,370,
2,323 and 2,344 before [M, the first frame showing each menu's title].

**What else the time decides.** `dma/chopping` (2,048-word transfers in blocks, bursts and chopped windows) keeps
stage 3's 129 of 130 rows within 10% of the console [M], and the "(GPU busy)" its rows print (GPUSTAT bit 26 clear
after the transfer) now agrees with the console's on 117 rows of 130, against 60 with the stub: the blocks that leave a
transfer short (any size that does not divide 2,048) leave it waiting, as on the console. The 13 that do not agree are
all the console's busy and Diamond's idle, after the 128-word blocks, the unchopped burst and eleven chopped windows of
16 to 128 words; a slower write rate does not move them, since a full FIFO then holds the DMA back and the transfers
grow slower than the console's (at 2.38 cycles a word, 93 of 130 within 10%), and no single rate explains them: open
(D-11). `benchmark` (an interactive program whose screen shows 10.52 ms for 100 random flat triangles) cannot be graded
without its triangles.

## 9. The gate

**ps1-tests' GPU programs with a picture**, run on the open kernel with no firmware folder, VRAM at frame 300 against
`vram.png` exactly, as `diffvram` compares (each five-bit channel shifted up three; the mask bit is not in the
pictures) [M, `Ps1_tests_gpu_programs_draw_their_vram_pictures_exactly`]:

| Program | Pixels compared | Differing | Note |
|---|---|---|---|
| `clipping` | 524,288 | 0 | |
| `clut-cache` | 524,288 | 0 | |
| `lines` | 524,224 | 0 | 64 pixels excluded, D-12: the two segments to the shaded polylines' fourth colour, which the program never sets |
| `quad` | 524,288 | 0 | |
| `rectangles` | 524,288 | 0 | |
| `texture-flip` | 524,288 | 0 | after D-8 |
| `texture-overflow` | 524,288 | 0 | |
| `transparency` | 76,800 | 0 | its screen only, D-12: the picture outside it is from the program's version before 2019-12-02, which cleared VRAM white |
| `triangle` | 524,288 | 0 | |
| `uv-interpolation` | 524,288 | 0 | |
| `vram-to-vram-overlap` | 524,288 | 0 | |

All eleven exact but for the named pixels. The first run of the GPU, before any fitting to these pictures beyond the
interpolation rule of §6, already had eight of eleven exact; the x-flip (D-8) was the one rule changed for this gate.

**ps1-tests' GPU programs by their TTY**, beside stage 3's seven (all still pass): `gp0-e1` 10 of 10, `mask-bit` 5 of
5, `version-detect` "GPU version 2", `bandwidth` every row within 2% [M, `Ps1_tests_pass_by_their_tty_on_the_open_kernel`].
**Not graded**: `animated-triangle` (no reference), `benchmark` (interactive, §8), `gpustat` (not in the release; §2.3).

**GPUSTAT** as `gpustat`'s console log: twelve of twelve (§2.3).

## 10. amidog's tests

Run on the open kernel, read from the picture [M, 2026-10-11]:

- **`psxtest_cpu` 1.3** runs to "ALL DONE" with no warning or error: every test's four columns (exception, flag,
  value, timing) are OK or N/A, in all its groups, among them "MEM DLY" and "MEM ADV", the delayed and advanced loads
  its README names, and "EXC" (RFE, BREAK, SYSCALL), "JMP" and "BRA ADV". This confirms on an independent program the
  load delay's corners that `BlackMoon_R3000.md` §10 left to it, as far as its verdicts reach: the screen gives one per
  instruction, not its cases, so whether a second load of a pending register is among them is not shown.
- **`psxtest_cpx` 1.1 and `psxtest_gpu` 1.7** draw their menus correctly and wait there: each runs its tests only when
  Start is pressed, and the pad is stage 6's (SIO0 answers FFh with no /ACK, `Diamond_Native.md` §11.8). So the CE
  bits, the COP0 registers 16-31 and amidog's GPU test wait on stage 6. *Since the pad (2026-10-11)* they run from
  their menus (`Diamond_Pad.md` §5).

### 10.1 `psxtest_gpu` 1.7 to its end (D-23, 2026-10-11)

The test prints each failing case on the TTY with its wanted and its got value when a word of its own data is set,
and compares the GPU against its own software model of it; how its columns were read, and what of the program was
read, is `Diamond_Disputes.md` D-23 and §16. Run from its menu by Start to its results [M]:

| | Before the step | After |
|---|---|---|
| ERROR cells on the results' first page (BASIC, COMMON, PRIMITIVE, XYFLIP's first rows) | 81 | 0 |
| Groups failing by the TTY | REG, POLYLINE, MOVE, VRAM READ, all 64 polygon and line commands, XYFLIP, OFFSET, POSITION, SIZE, AREA, FIFO, INTERLACE, CACHE TAG, CACHE BLOCK | FIFO, INTERLACE, CACHE TAG, CACHE BLOCK |
| Failing lines on the TTY | 2,650,445 | 654,986 |
| Frame of the results screen | 295,849 | 330,743 |

The rules changed, each pinned by a unit test of `src/gpu/tests.rs` or `vram.rs` that failed first, are in §2.2,
§2.3, §2.5, §3, §5 and §6, marked *since D-23's step*; the group-by-group table and the weight of each rule's evidence
are D-23's. The run grew longer: the menu comes 1,270 frames sooner (§8), but 46 tests that had given up after a
hundred errors now run every case. WiseMan's amidog tests therefore stay opt-in, and the full GPU run, half an hour
of host time, is not among them.

### 10.2 The FIFO, INTERLACE and cache groups (D-23, 2026-10-11)

The four groups §10.1 left, each run alone from a local copy with the verbose word set, and read as before; what each
measures, the rule each failure led to and the weight of its evidence are D-23's table [M]:

| Group | Failing lines before | After |
|---|---|---|
| FIFO | 53,916: the read test 48,128, the full test 3,976, the overflow test 1,812 | 1,812, the overflow test |
| INTERLACE | 3,060 | none |
| CACHE TAG | 594,688 | none |
| CACHE BLOCK | 3,322 | 1,322, its quad and triangle cases |

The program run whole, every test but DUMP, from a local copy that starts without the pad: 3,134 failing lines on the
TTY against §10.1's 654,986, the FIFO's 1,812 and CACHE BLOCK's 1,322, and none in any other group; its last group done
by frame 331,300 of that copy. The rules changed are in §2.1, §2.5, §3, §4 and §5,
marked *since the cache step*, each pinned by a unit test of `src/gpu/tests.rs` that failed first and by a program of
`programs/` run on the referees (`Diamond_Referees.md` §4.12): `texture-cache.s`, `interlace.s`, `gpuread-repeat.s`,
and `gp1-reset.s` from `Diamond_Records.md` §16.1. ps1-tests' eleven pictures stay exact, `gpu/bandwidth` and
`dma/chopping` print byte for byte as before, and the console logs' readings are unchanged [M].

### 10.3 The overflow test and CACHE BLOCK's polygons (D-41, D-42, 2026-10-12)

The two groups §10.2 left, run alone as before, reproduce on the open kernel: CACHE BLOCK's 1,322 failing lines, every
one in its quad's and its bottom-flat triangle's cases, and the FIFO overflow test's 1,812 [M]. Each was traced to a
rule, and neither rule is taken into the core here; both are §18's questions 4 and 5:

- **CACHE BLOCK** expects a triangle's rows drawn from its middle vertex's row down to its bottom and then from the row
  above it up to its top, each row left to right, which changes what the texture cache holds after a polygon taller
  than a cache block (`Diamond_Disputes.md` D-41). Beetle PSX, the only referee with a texture cache, draws the
  project's `programs/triangle-rows.s` the same way on all seven of its shapes, among them triangles with both parts,
  which the test does not have. Built so, CACHE BLOCK passes, ps1-tests' eleven pictures stay exact, `dma/chopping`
  prints byte for byte as before, and `gpu/bandwidth`'s two textured quad rows read 39,915 hblanks where they read
  39,975 and 39,976 (the console 39,938), their second triangle missing the cache fewer times. Since the step was held
  to keep `bandwidth`'s output, the change is on the branch `diamond-gpu4-rows`, measured and tested, and not here.
- **The overflow test** expects a FIFO that keeps its newest 16 words, NOPs and GP0(E6h) words taking places, while the
  processor holds the first command after the one drawing aside, and GP1(01h) to drop both and stop a fill where it
  is; that model fits all six of the harness's images, where the cache step's search had found none, missing that the
  processor discards a NOP before the command it takes aside (D-42). It contradicts §2.1's choice to hold a writer at a
  full FIFO: 58 of Peter Lemon's programs store to a full FIFO on Diamond's drawing time, and with the oldest word lost
  the 55 of them in `GPU/` and `CPUTest/` draw 1,280 to 62,881 pixels otherwise at frame 120 (§11's references draw
  them whole). The five referees draw a fill cut by GP1(01h) whole (`programs/fill-reset.s`). Unchanged.

## 11. Peter Lemon's GPU programs

The 25 programs of `GPU/16BPP` and `GPU/24BPP`, at frame 120, Diamond's picture against the references' screens through
the stock probe (PCSX-ReARMed on its own kernel; SwanStation and Beetle PSX with `--sysdir`, the player's BIOS, never
read) [M]. The engines' colour levels differ (each expands five bits to eight its own way), so a picture is compared
up to the colour map the most pixels agree on; the count is the pixels off it:

| Programs | SwanStation | PCSX-ReARMed |
|---|---|---|
| `MemoryTransfer24BPP`, `RenderRectangle16BPP`, `RenderRectangleClip16BPP`, the eight undithered `RenderTexturePolygon*` | 0 each | 0 each |
| `MemoryTransfer16BPP` | 0 | 320 |
| The four `RenderTextureWindowRectangle*` | 0 each | 10, 909, 909, 4 |
| `RenderLine16BPP`, `RenderPolygon16BPP`, `RenderPolygonClip16BPP`, `RenderPolygonDither16BPP`, `RenderTexturePolygon15BPPDither` | 5, 33, 25, 232, 40 | the same |
| The four `RenderTextureRectangle*` | 1,214, 1,214, 1,214, 855 | 1,220, 1,136, 1,136, 843 |

- **Sixteen of 25 equal SwanStation's picture** up to its colour levels, eleven of them PCSX-ReARMed's too.
- **The shaded ones** differ by one five-bit level in Gouraud-shaded pixels (sampled: every pixel off the map is one
  level from it), in the shaded quads and lines. On lines Diamond's rule is the one `lines`' console picture fixed, so
  the references' own interpolation is the likelier source there [A]; on quads it is open, a quad's second triangle's
  starting vertex being what no console picture decides (D-7). On the shaded triangle the three candidate starting
  vertices give the same pixels [M].
- **The textured rectangles** differ only inside the rectangles the program draws with GP0(E1h)'s x- and y-flip bits
  (every one of the 1,214 and 855 pixels): the two references draw them unflipped, where `texture-flip` shows a console
  flipping them, one texel on (D-8).
- **Beetle PSX** places these programs' pictures 150 to 160 lines down at frame 600 (at 120 it is still in the BIOS's
  own start), leaving too few lines to compare; not graded.
- Their bundled PNGs are 320x224 and of unstated origin; they show the semi-transparent shapes brighter than mode 0
  (B/2+F/2), which every program here sets and the console's `transparency` picture confirms, so they were not used to
  grade [M, A].

## 12. The state, the spaces, the cost

- **State** `DMND` version 1, redefined again (no player holds one): the GPU's group in `Bus` holds GP0(E1h)..(E6h),
  GP1(09h), IRQ1's flag, the DMA direction, the latch, the display's start and blanking, the FIFO's sixteen words with
  their arrival times and meanings, the tail's parse state, the command half received, an open transfer or polyline,
  the busy time and its carried fraction, the texture cache's tags and data, the CLUT cache, and the display's scan
  position; VRAM is the state's own block as before; the picture is not kept, and a state loaded between frames
  re-reads it within the next frame [C]. **3,747,681 bytes**, pinned by `the_version_1_layout_is_pinned` (*version 2,
  3,747,689 bytes, since stage 5: `Diamond_Native.md` §13.6*; *version 5, 3,747,806 bytes, since the cache step: the
  word GPUREAD repeats, a transfer's first word and the texture cache's page, pinned by
  `the_version_5_layout_is_pinned`*).
- **Spaces** unchanged; `VRAM` is now the GPU's.
- **Cost** [M, §15]: the frame time measured in `Diamond_Native.md` §12. *D-23's step and D-11's refit* [M,
  `examples/run`, 600 frames, three runs each, the two builds side by side, ms a frame before and after]: `timers`
  6.28 and 6.16, `gpu/triangle` 6.44 and 6.37, `gpu/rectangles` 5.51 and 5.36, `gpu/bandwidth` 7.85 and 7.87; each
  within its runs' spread but `rectangles`, 2.7% faster, a difference not traced. The copy's row is a 2 KiB array on
  the stack, and nothing added allocates. *The cache step* [M, the same, the base build at `b9b9be79`, ms a frame
  before and after, while another emulator run used one core]: `timers` 6.97 and 6.60, `gpu/triangle` 6.61 and 6.64,
  `gpu/rectangles` 5.61 and 5.62, `gpu/bandwidth` 8.12 and 8.20; the last 1.0% slower with the runs' ranges (8.09-8.17,
  8.16-8.25) touching, the others within their spread. The cache's page is compared once a primitive, the interlace
  parity computed once a primitive and only in 480-line interlace; nothing is done per pixel that was not, and nothing
  allocates.

## 13. The enhancement layer

The plan's §6.2 keeps the accurate VRAM as machine state and has a separate renderer consume the command stream with
side data [R]. Every rendering command reaches `Gpu::execute` whole, with its words, at the time it starts, and nothing
else draws; an observer there sees the stream in order with its times and the drawing state that applies, and needs
nothing of VRAM [A]. The texture coordinates, colours and vertices a renderer needs are the command's words, decoded
by `vertex` and `Texture::new`, which a renderer would share rather than copy.

## 14. Not modelled, and choices to revisit

- The 160-pin chip's fill at an X ending in Fh; GP1(20h) and the 208-pin prototype; the light gun's latch; GP0(03h)'s
  unknown function.
- psx-spx's "GPU-DMA transfers seem to work only during V-Blank" and "dma-to-vram doesn't work with blksiz>10h" on the
  160-pin chip [R "GPU Versions"]: both apply to the 160-pin chip, not the default, and are not modelled.
- Drawing at a command's start (§2.1); texture cache staleness, built from the document and observed by no program
  yet; a full FIFO holding its writer.
- The 13 chopping rows (D-11).
- ~~*Since D-23's step*, the groups of amidog's GPU test that still fail (§10.1): FIFO (its read test expects GPUREAD to
  go on returning VRAM's following pixels after a transfer's last word, and its full test a FIFO the model does not
  keep), INTERLACE (its first case expects a fill to leave the displayed field's lines alone in 240-line interlace,
  where Diamond skips lines only for rendering commands in 480-line mode, §5), and the texture cache's TAG and BLOCK
  groups (the cache's tags and its blocks against the model's; the first cases already differ in which entries a
  page change keeps). None was among the groups this step set out to settle, which were those the first run's record listed (they failed then too); each is left to a step of its own.~~
  ~~*Since the cache step* (§10.2): the FIFO's overflow test (a FIFO overflowed by the CPU, and GP1(01h) stopping a fill
  part-drawn, which a fill drawn whole at its start cannot show), and CACHE BLOCK's quad and triangle cases.~~
  *Since the rows step* (§10.3): both traced and left to the tester. CACHE BLOCK's cases are the order of a triangle's
  rows (D-41), built on `diamond-gpu4-rows` and not here because it moves `bandwidth`'s textured quad rows by 60
  hblanks; the overflow test is a FIFO that keeps its newest words and a fill that GP1(01h) stops (D-42), against
  Peter Lemon's programs drawn whole and the five referees.
- The texel pairs' rule (§6) beyond rectangles drawn from an even column, and the odd read's halfword beyond one pixel.

## 15. Tests run (2026-10-11)

`cargo test --release`: `diamond` 53 unit tests (17 of the GPU's) and 5 program tests (`gpustat` among them);
`cargo clippy --all-targets` clean. WiseMan, under the test lock with `EMUSEN_DIAMOND_PS1TESTS`: `DiamondTests` 15 of
15 (the GPU's VRAM gate, the TTY gate with the four GPU programs added, the VRAM grader on a program of the test's own
with its negative control, the kit's C1-C15); and the blast radius, `Diamond_Native.md` §12.

*D-23's step and D-11's refit (2026-10-11)*: `cargo test --release`: `diamond` 80 unit tests (31 of the GPU's, ten of
them new, each first seen failing) and 23 program tests (`small_rectangles_and_fills_cost_their_rows` new,
`gpustat_while_each_shape_arrives_and_draws` and `the_bandwidth_record_counts_each_rows_hblanks` re-pinned), with
`EMUSEN_DIAMOND_PS1TESTS`; `cargo clippy --all-targets` (clippy 0.1.98) clean. ps1-tests' eleven pictures exact as in
§9, checked after each change. WiseMan, under the test lock with `EMUSEN_DIAMOND_PS1TESTS` and amidog's tests beside
the corpus: `DiamondTests` 19 of 19 (the VRAM and TTY gates, the CPX test from frame 400, the GPU and GTE menus from
frames 300 and 400).

*The cache step (2026-10-11)*: `cargo test --release` with `EMUSEN_DIAMOND_PS1TESTS`: `diamond` 86 unit tests (37 of
the GPU's, six of them new and each first seen failing; the state's layout re-pinned at version 5) and 32 program tests
(four new, each failing on the build before; `the_io_bitwidth_record_reads_each_register_after_each_width` re-pinned);
`cargo clippy --all-targets` (clippy 0.1.98) clean but for one warning in `tests/support/records.rs`, not this step's.
ps1-tests' eleven pictures exact (§9). WiseMan, under the test lock with the corpus and amidog's tests: `DiamondTests`
20 of 20.

*The rows step (2026-10-12)*: nothing of the core changed here. `cargo test --release`, with `EMUSEN_DIAMOND_PS1TESTS`
and without: `diamond` 147 unit tests and 59 program tests, two new, both pins of the referees' readings that the
build before passes too: `a_fill_cut_by_gp1_01h_is_drawn_whole_as_the_referees_draw_it` and
`the_triangle_rows_program_reads_its_undisputed_shapes_as_the_referee`. WiseMan, under the test lock with the corpus:
the 35 tests named for Diamond, 35 of 35. With stage 11 merged in, 148 unit tests, the same 59, and WiseMan's 40 of 40.

## 16. Reading log

Every file opened for this stage, as the protocol of `Diamond_Plan.md` §1.3 asks:

- **psx-spx** `6d7d1bc`: `graphicsprocessingunitgpu.md` in full; `dmachannels.md` and `unpredictablethings.md` at their
  lines naming the GPU.
- **ps1-tests** source `f727802`: `README.md`; the sources of `gpu/animated-triangle` (listed only), `bandwidth`,
  `clipping`, `clut-cache`, `gp0-e1`, `gpustat`, `lines`, `mask-bit`, `quad`, `rectangles` (but its picture data),
  `texture-flip`, `texture-overflow`, `transparency`, `triangle`, `uv-interpolation`, `version-detect`,
  `vram-to-vram-overlap` (but its test-case data) and `dma/chopping`; the console logs of `bandwidth`, `gpustat`,
  `gp0-e1`, `mask-bit` and `dma/chopping`; `tools/diffvram/main.go`; `common/gpu.h` and `common/gpu.cpp` in full,
  their headers first checked for a copied-from note (none); the first six lines of the other `common/` sources to
  check their headers; the git history of the GPU pictures and of `transparency` and `lines` (`git log`, the diff of
  `2fe1570` and of `d1a63ee` against the head). The release's pictures and the `benchmark` screenshot.
- **A reading to declare**: the header check over `common/` printed the first line of `common/cop0.hpp`, which is
  `#pragma once`, before its output was cut; nothing further of it, and nothing of `common/dma.hpp`, was printed.
- **amidog**: `psxtest_cpu.txt`, `psxtest_cpx.txt` and `psxtest_gpu.txt`; the three programs' screens.
- **Peter Lemon** `6d20c13`: `README.md`; `GPU/16BPP/RenderPolygon/RenderPolygon16BPP.asm`, `RenderPolygonClip16BPP.asm`
  and `RenderTextureRectangle15BPP.asm` (but their data), the rectangle lines of the `CLUT4BPP` and `MASK15BPP`
  rectangle programs, and `LIB/PSX_GPU.INC` at its triangle macros; the bundled pictures.
- **The references**, as black boxes: the probe's screens of the three cores on Peter Lemon's programs; of Beetle
  PSX's log of one run on the player's BIOS, three lines naming its options' defaults, nothing of the BIOS's run.
- **EmuSen's own**: `Diamond_Plan.md`; `Diamond_Native.md` §1-§2, §10 and §11; `Diamond_References.md` but §4 and §6;
  `Diamond_Disputes.md`; `Diamond_OpenKernel.md` §3.3 and §4.12; `BlackMoon_R3000.md` §10 and §11; the cache's
  `DOCS.txt`; `EmuSen_CoreAPI.md` on the frame; `EmuSen_CRT.md` §20.6; the Diamond crate, `emusen-native`'s frame and
  exports, other cores' `frame_info`, WiseMan's runner and `DiamondTests`.
- **Not read**: any emulator's source, any firmware or listing, PSn00bSDK's libraries, PSX_MiSTer.
- *D-23's step and D-11's refit, 2026-10-11*:
  - **psx-spx** `6d7d1bc`, `graphicsprocessingunitgpu.md` again at "GP1(10h)", "GP1(09h)", "GP0(E3h)", "GP0(E4h)",
    "GP0(E5h)", "Vertex", "GPU Status Register", "Ready Bits", "CPU to VRAM blitting", "VRAM to CPU blitting",
    "Masking for COPY Commands parameters" and "Memory/Rendering Timings"; `dmachannels.md` at its GPU rows.
  - **ps1-tests**: `gpu/gpustat/main.cpp` again from its `main` and `drawRect`, and `common/gpu.cpp` at
    `initVideo` and `setResolution`, to see the drawing area the C program sets; `gpu/bandwidth/main.cpp` at its tests'
    list and `quadScreen`, `fillScreen` and `rectScreen`. In `gpustat/main.cpp` the line `#include <dma.hpp>` was
    printed with its includes; nothing of that header.
  - **amidog**: `psxtest_gpu.txt` again; `psxtest_gpu.exe`'s strings, and its harness disassembled with
    `emusen-mips`'s disassembler from the file: the menu's item tables and its Start test, the GP0 and GP1 write
    functions and the data and status compares, the REG, MOVE, VRAM READ, FIFO read and polyline loops, and the helpers
    those call to write and read VRAM. Of the program's own GPU model, which is its author's emulator and is not to be
    read, the step opened the wrapper that dispatches a GPUREAD or GPUSTAT read to it through a table (at 8003E42Ch to
    8003E568h, a register access logged and forwarded, no rule of the GPU), recognised it, and read nothing past it.
  - **The references**, as black boxes: DuckStation's and PCSX-Redux's TTY of local copies of the test; the three
    libretro cores' RAM for `programs/draw-time.s`.
  - **EmuSen's own**: this page, `Diamond_Disputes.md`, `Diamond_Referees.md`, `Diamond_Pad.md` §5, `Diamond_Native.md`
    at its cost tables, the crate's GPU and `tests/programs.rs`, `psxreferee.py`, WiseMan's `AmidogScreen` and
    `DiamondTests`.
  - **Exposures, stated as the rules ask**: a listing of the running systemd scopes printed referee command lines,
    which name the player's BIOS file; the file went to the referees by path only and is named in no committed file;
    no BIOS byte and no BIOS output was read, the referees' TTY being cut at the test's own heading.

- *The cache step, 2026-10-11*:
  - **psx-spx** `6d7d1bc`, `graphicsprocessingunitgpu.md` again at "GP1(08h)", "GPU Status Register" (bits 13 and 31),
    "Quick Rectangle Fill", "VRAM to CPU blitting", "GPU I/O Ports", "GP0(00h)", "GP0(03h)", "GP0(04h..1Eh...)", "GP1
    Display Control Commands" (the GP1 preamble, GP1(00h), GP1(01h)), "Memory/Rendering Timings", "GPU Texture Caching"
    and "GP0(E1h)".
  - **amidog**: `psxtest_gpu.exe`'s harness disassembled from the file with `emusen-mips`: the FIFO read, full and
    overflow tests (their loops, their expected values and the images the overflow test builds), the CACHE TAG table
    of sub-tests and its sub-test functions, the primitive function they call, the CACHE BLOCK loop, the GP0 and GP1
    write wrappers up to the indirect call into the model, and the compare function's arguments. The model was not
    read: the wrappers' indirect calls were not followed. The TTY's expected values and the expected images in RAM were
    read as outputs.
  - **The references**, as black boxes: DuckStation's TTY of the FIFO group (a local copy), and the five referees' RAM
    for `texture-cache.s`, `interlace.s` and `gpuread-repeat.s`; `gp1-reset.s`'s referee words from the records step's
    cache.
  - **EmuSen's own**: this page, `Diamond_Disputes.md`, `Diamond_Referees.md`, `Diamond_Records.md` §11-§17,
    `Diamond_Pad.md` §5, the crate's GPU, bus, state and tests, `psxreferee.py`.
  - **Exposures, stated as the rules ask**: the player's BIOS went to the referees by path only, chosen by its file
    name from a listing of the firmware folder, and is named in no committed file; no BIOS byte or BIOS output was
    read, the TTY cut at the test's heading and the RAM read at the programs' own addresses.

- *The rows step, 2026-10-12*:
  - **psx-spx** `6d7d1bc`, `graphicsprocessingunitgpu.md` again at "GPU I/O Ports", "GP0(00h)", "GP0(03h)",
    "GP0(04h..1Eh,E0h,E7h..EFh)", the GP1 preamble and "GP1(00h)", "Memory/Rendering Timings" and "GPU Texture
    Caching"; the whole page searched for words on the order of drawing (none found).
  - **amidog**: `psxtest_gpu.exe`'s harness, in the cache step's disassembly of the file, at the CACHE BLOCK test
    (80033100h to 800335A0h: its loops, its six first draws and its compare) and the FIFO overflow test (80031864h
    to 80031D3Ch: the store loop, the GP1(01h) branch, the image it builds and the next helper); searches of that file
    for the harness's strings, which print only the lines that load them. The GP0 and GP1 words of both groups as
    written, logged by a scratch runner, and the harness's images read from RAM at its compare's entry. The model was not read and no
    indirect call of the wrappers was followed. DuckStation's TTY of the FIFO group from the cache step, again at the
    overflow test.
  - **Peter Lemon** `6d20c13`: the release's programs run on Diamond and their VRAM compared between two builds; the GP0
    words of `CPUADD` as written. No source.
  - **The references**, as black boxes: the five referees' RAM for `programs/triangle-rows.s` and
    `programs/fill-reset.s`.
  - **EmuSen's own**: this page, `Diamond_Plan.md` §1, `Diamond_Disputes.md`, `Diamond_Referees.md`,
    `Diamond_Records.md` at §14 to §16, the crate's GPU, `bus.rs` at its GP0 store and DMA paths (instrumented in a
    scratch build to count the stores that wait, then restored), `tests/programs.rs`, `psxreferee.py`, the cache step's
    scratch runner and search, and the cache's `DOCS.txt`.
  - **Exposures, stated as the rules ask**: the player's BIOS went to the referees by path only, the file the cache
    step's script named, and is named in no committed file; no BIOS byte or output was read, the RAM read at the
    programs' own addresses.

## 17. Negative results

- No program run here observes the texture cache's staleness; amidog's GPU test, whose menu lists cache tests, waits
  for the pad.
- No reference exposes VRAM (`Diamond_References.md` §4 by its summary), so Peter Lemon's programs are compared by
  picture only, and Beetle PSX's pictures of them are displaced.
- `gpustat` is not in the release; its console log grades a program of the project's own.
- One single write rate cannot give both `dma/chopping`'s cycles and its busy marks (D-11).
- *The cache step*: ~~no model of the FIFO tried reproduces amidog's overflow images (D-23)~~ *one does, since the
  rows step (D-42)*; no referee models a FIFO that loses words. ~~The CACHE BLOCK quad and triangle cases were not
  traced to a rule.~~ *Traced since the rows step (D-41).*
- *The rows step*: no referee stops a fill at GP1(01h), and none but Beetle PSX keeps texels in a cache, so no
  second witness exists for D-41's part order; no program here separates whether a triangle's middle row belongs to
  its lower part or its upper one.
- *D-23's step*: no referee reproduces amidog's expectations for REG, the odd read, the texel pairs or the offset's
  placement, so those rules stand on the test alone; PCSX-Redux stopped with an exception in its Lua event listener
  in both runs of the test and witnesses only REG; the libretro cores carry no TTY and were not run on it.

## 18. Questions for the tester

1. **The frame's end** moved from a field's start to its vblank (§7). Keep it, or keep the field's start and grade
   looping programs some other way? *Decided 2026-10-11: kept at vblank
   (`Diamond_Native.md` §11.5).*
2. **amidog's CPX and GPU tests** need Start pressed. Bring a minimal digital pad forward from stage 6 to run them now,
   or leave them to stage 6 with the CE bits and the COP0 registers? *Decided
   2026-10-11: brought forward (`Diamond_Pad.md`).*
3. **D-8, D-10 and D-11** are settled or open on one console's pictures and logs; a program written for each (the
   x-flip's offset at an odd start column; GPUSTAT bit 28 during a polygon; the busy state after a burst) would settle
   them on the tester's console, if one is available (Q16).
4. **D-41, the order of a triangle's rows** (§10.3), ruled on amidog's expectations and Beetle PSX, passes CACHE BLOCK
   and keeps every picture and log, but `gpu/bandwidth`'s two textured quad rows read 39,915 hblanks where they read
   39,975 and 39,976 (the console's log 39,938, both within 0.2%). Take it (merge `diamond-gpu4-rows`, which re-pins
   the bandwidth record's two rows), or keep `bandwidth`'s output as it is?
5. **D-42, a CPU write to a full FIFO** (§10.3): keep holding the writer, as Peter Lemon's programs and the referees
   need, and leave amidog's overflow test failing; or take the model that fits amidog's harness, a store that loses
   the FIFO's oldest word, with one command decoded ahead and fills drawn line by line, at the cost of Peter Lemon's
   pictures?
