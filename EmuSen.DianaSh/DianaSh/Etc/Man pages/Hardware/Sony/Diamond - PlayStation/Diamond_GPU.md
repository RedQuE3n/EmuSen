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
  README and pictures, which a dropped word would not allow].
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
and bit 11 when allowed, and no others ([R "TPage Attribute"]; the same log).

### 2.3 GPUSTAT's ready bits (D-10)

| Bit | Diamond | Source |
|---|---|---|
| 25 | by GP1(04h): 0; the FIFO not full; bit 28; bit 27 | psx-spx "GPU Status Register" [R] |
| 26 | the FIFO empty, no command half received, no transfer or polyline open, and no command drawing | the console's `gpu/gpustat` log [M by ps1-tests] |
| 27 | a VRAM-to-CPU transfer's words ready | psx-spx [R] |
| 28 | the FIFO empty | the same log, against psx-spx's "cleared when the command execution is busy" (D-10) |

The release has no `gpustat` binary (`Diamond_References.md` §1), so its four cases were written as the project's own
program, `programs/gpustat.s`, from its source: a rectangle drawn by the CPU, two by a linked list, two by blocks of one
word with directions 1 and 2, GPUSTAT read before, 300 cycles into the transfer, and after. All twelve readings of
bits 25-30 equal what the console's log passes [M, `gpustat_reads_as_the_console_passes`]. The program reads its
"during" values 300 cycles in, where the C program reads them after its function returns; any point after the second
rectangle is queued and before the first is drawn (about 9,000 cycles) gives the same bits [A].

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
the default console [M].

## 3. VRAM and its transfers

- **The fill** masks its parameters as psx-spx's "Masking and Rounding for FILL" [R]: X in steps of 16, its width
  rounded up to 16, no fill at a zero size; the colour's top five bits a channel with bit 15 clear; no area, no mask,
  wrapping at VRAM's edges without carry ("Wrapping"). The 160-pin chip's misplaced pixels at an X ending in Fh
  ("GP0(02h) FillVram") are not modelled [C: the diagram does not say where the pixels go].
- **The copy** reads and writes row by row in pieces of 16 halfwords, each piece read whole before it is written
  [C], under the mask setting [R]. On `vram-to-vram-overlap` (squares of 2, 8, 15 and 16 pixels, and of 16 one pixel
  off in x, y and both, each copied over itself at the 21 displacements of -3 to 3 across and -1 to 1 down) the
  picture is exact [M].
- **CPU to VRAM and VRAM to CPU** take psx-spx's copy masking, two pixels a word, the extra halfword of an odd count
  ignored or padded; the writes under the mask setting, as `mask-bit`'s five cases pass on the console and on Diamond
  [M].

## 4. Texturing

- **The page, the depth and the window** as psx-spx's "GP0(E1h)", "Texture Bitmaps" and "GP0(E2h)" [R]: a 4-bit texel
  the nibble of the halfword at `page_x + u/4`, an 8-bit one the byte at `page_x + u/2`, a 15-bit one the halfword at
  `page_x + u`, X wrapping at 1024 (`texture-overflow`, exact [M]); the window's `(t AND NOT mask*8) OR ((offset AND
  mask)*8)`.
- **Bit 11 of the page** puts it 512 lines down while GP1(09h) allows it on the 208-pin chip, where a retail console
  has no chip and "textures disappear" [R "GP0(E1h)", "GP1(09h)"]: a texel there reads 0000h, transparent [A]. With
  GP1(09h) clear the bit is ignored.
- **The texture cache**, 2 KiB, is psx-spx's 256 entries of 8 bytes in cache blocks of 64x64, 32x64 or 32x32 texels
  by depth, one entry of each number held [R "GPU Texture Caching"]. Restated in VRAM's terms, which is how Diamond
  indexes it [A]: an entry is four halfwords; a 4- or 8-bit page indexes by X's bits 2-3 and Y's bits 0-5, a 15-bit page
  by X's bits 2-4 and Y's bits 0-4; the tag is the entry's VRAM address. An entry is kept until another address takes
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
  `clipping` exact [M].
- **The interlaced field**: in 480-line interlace with drawing to the display area prohibited (E1h bit 10 clear), the
  lines of the field being displayed are not drawn [A, from "Vertical Interlace flag DOES affect GP0 draw commands"].
  No program run here clears the bit in 480-line mode.

## 6. The rasteriser (D-7)

psx-spx gives the vertices' range and the size limit, "up to <excluding> their lower-right coordinates" for polygons,
"up to <including>" for lines, the quad's split into (1,2,3) and (2,3,4), and that the interpolation is affine [R]. It
does not give the fill rule, the interpolators' precision, where they start, or the lines' stepping. Each was fitted
to ps1-tests' pictures, which are a console's VRAM (D-7):

- **Coverage.** A pixel is the point at its integer coordinates; inside a triangle when every edge function is
  positive, or zero on an edge the triangle owns, a top or a left one. On `triangle` this leaves the apex's row empty,
  one pixel on the next, and the bottom row out [M: exact].
- **Attributes.** Each of R, G, B, U and V has a gradient in x and in y of 12 fractional bits, the cross-product
  difference shifted up 12 and divided by twice the area, truncated toward zero; the value at a pixel is vertex 0's,
  plus the gradients times the distance from vertex 0, plus a half, taken as its integer part. On `uv-interpolation`,
  whose quads of widths 0 to 255 step U from 0 to 1, this is the only one of the candidates tried (fractional bits 8 to
  23, every bias in 1/4096 steps) that puts every row's switch where the console does; the same rule gives its 256
  Gouraud rows, plain and dithered, and `triangle`'s three, exact [M]. A quad's second triangle starts from its own
  first vertex, the quad's second [C: no console picture tells the starting vertex apart, D-7].
- **Size.** A triangle wider than 1,023 or taller than 511 between its vertices is not drawn, each of a quad's two
  judged alone [R; C for the per-triangle reading].
- **Rectangles** are drawn from their corner, the texture coordinate stepping one a pixel and wrapping at 8 bits;
  with GP0(E1h)'s flips (208-pin only) it steps backwards, and an x-flipped rectangle starts from U+1 (D-8): on
  `texture-flip` every flipped rectangle's first column reads U+1, then U, U-1 ..., where a y-flip starts at V [M].
- **Lines** step along the major axis with both ends drawn; the minor coordinate is the nearest to the exact line, a
  tie rounded toward the end on an x-major line and toward the start on a y-major one; the colour by the polygons'
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

**The model** [C]: a command's own 2 cycles; a setup of 876 a triangle and 742 a rectangle; 0.836 cycles a pixel
written, 1.266 a pixel read and written (semi-transparent, or the mask checked), 0.445 a texel fetched, 19.67 a texture
cache miss; the fill 597 plus 0.5 a row plus 2 each 16 pixels; the copy 2.13 a pixel; 1.67 a CPU-to-VRAM data word (two
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
differ only in how much of them is clipped give the plain pixel's 0.836 cycles and show that clipped pixels and rows
cost nothing; the textured rows write nothing (their texels are 0000h, transparent), which separates a texel's fetch
from a write, and their misses (4,800 a 4-bit rectangle and 15,551 a 15-bit quad, counted by the cache model) give the
miss; the residuals that remained were the same for the half- and quarter-clipped quads, which differ in rows but not
in triangles, so they are a setup, not a row cost. The transfers are bound by the DMA (cpuToVram) or by the GPU's read
rate (vramToCpu) [A]. `bandwidth` joins the TTY gate with each row within 2%.

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
  bits, the COP0 registers 16-31 and amidog's GPU test wait on stage 6.

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
  3,747,689 bytes, since stage 5: `Diamond_Native.md` §13.6*).
- **Spaces** unchanged; `VRAM` is now the GPU's.
- **Cost** [M, §15]: the frame time measured in `Diamond_Native.md` §12.

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

## 15. Tests run (2026-10-11)

`cargo test --release`: `diamond` 53 unit tests (17 of the GPU's) and 5 program tests (`gpustat` among them);
`cargo clippy --all-targets` clean. WiseMan, under the test lock with `EMUSEN_DIAMOND_PS1TESTS`: `DiamondTests` 15 of
15 (the GPU's VRAM gate, the TTY gate with the four GPU programs added, the VRAM grader on a program of the test's own
with its negative control, the kit's C1-C15); and the blast radius, `Diamond_Native.md` §12.

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

## 17. Negative results

- No program run here observes the texture cache's staleness; amidog's GPU test, whose menu lists cache tests, waits
  for the pad.
- No reference exposes VRAM (`Diamond_References.md` §4 by its summary), so Peter Lemon's programs are compared by
  picture only, and Beetle PSX's pictures of them are displaced.
- `gpustat` is not in the release; its console log grades a program of the project's own.
- One single write rate cannot give both `dma/chopping`'s cycles and its busy marks (D-11).

## 18. Questions for the tester

1. **The frame's end** moved from a field's start to its vblank (§7). Keep it, or keep the field's start and grade
   looping programs some other way?
2. **amidog's CPX and GPU tests** need Start pressed. Bring a minimal digital pad forward from stage 6 to run them now,
   or leave them to stage 6 with the CE bits and the COP0 registers?
3. **D-8, D-10 and D-11** are settled or open on one console's pictures and logs; a program written for each (the
   x-flip's offset at an odd start column; GPUSTAT bit 28 during a polygon; the busy state after a burst) would settle
   them on the tester's console, if one is available (Q16).
