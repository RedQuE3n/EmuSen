# Diamond_MDEC — the motion decoder

*Written 2026-10-11, at stage 9.* The plan's §7 row 9 (`Diamond_Plan.md`) and its §5.7: the MDEC's commands, the
run-length decode, the quantisation, the IDCT with the scale table the program uploads, the colour conversion, the
four output depths, DMA channels 0 and 1, and the decoder's time. The device is `src/mdec/` (the register interface,
the FIFOs and the pipeline in `mod.rs`, the arithmetic on one block in `decode.rs`, the state in `state.rs`, its tests
in `tests.rs`); the bus moves the DMA's words in `bus.rs`'s `mdec_piece`.

The marks are the plan's: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44), the tool named;
**[R]** read from a source; **[A]** argued; **[C]** a choice. The disputes this stage opened are **D-30 to D-33** in
`Diamond_Disputes.md`; the SPU's stage 8, built beside it, may number its own from D-30 too, and the merge renumbers.

---

## 1. What it is written from

psx-spx's chapter "Macroblock Decoder (MDEC)" at `6d7d1bc` [R], read whole: the I/O ports, the commands, the
decompression pseudocode, the data format; and its "DMA Channels" for the MDEC's rows. Where psx-spx says it does not
know (the IDCT's exact rounding, "the results aren't perfect"; the colour conversion's fixed point, "unknown"; the
decompression time, "still unknown"), the rule was taken from the console's own outputs that ps1-tests ships — the
hexdumps of `mdec/4bit` and `mdec/8bit`, the status log of `mdec/step-by-step-log`, and the VRAM pictures of `frame`
and `movie` — and, for time alone, from referees run as black boxes (§4.3).

**No emulator's or library's MDEC, IDCT or JPEG code was read**, nor PSn00bSDK's or PsyQ's library sources. General
knowledge of how emulators are usually written may come from code seen in training; every rule here is taken from
psx-spx or from the measurements named beside it, and where a rule was chosen among measured variants the search is
recorded (§3.3), so that what the code does can be traced to the evidence and to nothing else.

## 2. The register interface

| Port | What Diamond does | Source |
|---|---|---|
| MDEC0 write, no command receiving | a command: bits 25–28 to status bits 23–26 for every command; MDEC(1) takes its depth, signed and bit-15 bits and bits 0–15 as its parameter words; MDEC(2) expects 16 words, or 32 with bit 0; MDEC(3) 32; MDEC(0) and (4)–(7) copy bits 0–15 to the status and expect nothing | [R psx-spx "MDEC Commands"] |
| MDEC0 write, a command receiving | a parameter word into the input FIFO; status bits 0–15 count down, FFFFh when none remain | [R "MDEC Status Register"; M the step-by-step log] |
| MDEC0 read | the next output word; while a block is in the IDCT and none is ready, the reader waits for it (D-33); with nothing coming, the last word read again [C, for psx-spx's "garbage"] | §4.4 |
| MDEC1 read | the status: bit 31 output empty, 30 input full or held (§4.1), 29 busy, 28 DREQ0, 27 DREQ1, 26–23 the command's bits, 18–16 the block (§4.2), 15–0 the count | [R; M] |
| MDEC1 write | bit 31 resets: any command aborted, both FIFOs and the pipeline emptied, status 80040000h; the tables are kept [A: psx-spx names only the command and the status]; bits 30 and 29 enable the two requests | [R "MDEC Control/Reset Register"] |

ps1-tests' `cpu/io-access-bitwidth` writes MDEC1 at three widths and reads 80040000h each time on its console log, as
the stub did; its record test still passes.

## 3. The decode, block by block

### 3.1 The run-length codes and the quantisation

psx-spx's `rl_decode_block` [R], with two readings settled by the console's data:

- A block **ends when its last coefficient is written**, index 63, with no end code: `mdec/step-by-step-log`'s stream
  begins `0400 F800 0400 F800 0600 F800`, three blocks of a DC and one run to index 63, and parses into its four
  macroblocks only so [M, the stream's own words]. psx-spx's prose ("EOB isn't required if the block was already fully
  defined") says the same; its pseudocode reads one more halfword. A run past 63 also ends the block, its value
  dropped [A].
- An AC value is `(v·q·qs + 4) >> 3`, an arithmetic shift [C]: through the exact IDCT the shift and a rounding
  symmetric about zero lie equally close to the console's movie and 8-bit block (0.285 and 0.286 grey levels on
  average), truncation toward zero further (0.322) [M, the fit of §3.3]. The movie's products are multiples of eight,
  on which the shift and the symmetric rounding agree, so only `8bit`'s block, at q_scale 1, tells those two apart.
  The DC is `v·q[0]` and q_scale 0 stores `2v` in stream order, as psx-spx gives; both saturate to 11 bits.

### 3.2 The tables

The quantisation tables are taken as 64 bytes each, in stream order; the scale table as 64 signed halfwords whose upper
13 bits are used, `v >> 3` [R "set_scale_table": "the hardware does actually use only the upper 13bit"].

### 3.3 The IDCT, and how its rounding was chosen

psx-spx's `real_idct_core` [R]: two passes of eight products a value with the scale table, the first down the columns.
psx-spx divides each pass's sum by 2000h with a rounding it marks "or so?", and says the results "aren't perfect".
The rounding was therefore searched against the console's outputs, with the decode's input taken from each program's
own data (the tables and streams in the release's RAM image, `sunset.mdec`, `bad_apple.h`'s frame table):

- **The evidence**: `mdec/8bit`'s 64 bytes in its console log; the luminance of `movie`'s 24-bit picture, whose
  chroma blocks are all zero so that R = G = B = Y, 117 distinct blocks of 64 pixels with 100 carrying AC values, the
  picture being the movie's frame 62 by its DC values [M]; `mdec/4bit`'s 32 bytes.
- **What the search covered** [M, a C harness in the cache]: the table at 12–16 bits, floored or rounded; each pass's
  sum shifted by any amount with floor, truncation toward zero, round-half-up, psx-spx's `+0FFFh` or round-half-away,
  with the fraction kept between the passes from none to all; products truncated before summing in either pass; the
  passes in either order; pass 1 saturated or wrapped at 10–16 bits; each pass's even and odd halves rounded apart,
  as a butterfly would. About 300,000 variants.
- **What none achieved**: an exact match. The exact real IDCT is within 0.28 grey levels of the console on average
  (rounding alone would give 0.25) but up to 1.1 apart, the misses spread over every pixel position, so the console
  loses precision somewhere none of these models does.
- **The rule chosen**, the best on the evidence: each pass's sum **truncated toward zero**, keeping **three fraction
  bits after pass 1 and four after pass 2**, and the value made whole by rounding a half down. On the movie's luminance
  752 of 92,160 halfwords differ, on `mdec/8bit` one pixel of 64 at five bits (ten bytes at eight bits), on `mdec/4bit`
  none. This is D-30, open.

### 3.4 Colour, and the 15-bit pixel

psx-spx's `yuv_to_rgb` [R]: each Y block's pixel with the chroma sample of its quadrant, `(x+xx)/2` (nearest; a
centred bilinear sampling is 41% worse on the sunset [M]), and the factors 1.402, −0.3437, −0.7143 and 1.772, here at
twelve bits, the chroma made whole first. Two findings from the console's pictures:

- **The 15-bit pixel is rounded from the channel's fraction, not cut from its 8-bit value** [M]: in `frame`'s two
  console pictures of the same image, every 5-bit channel equals the 24-bit one's `(v + 4) >> 3` but where `v ≡ 4 mod 8`,
  and there 5,982 of 14,601 are one lower — those whose 8-bit value was rounded up from below. Diamond rounds both from
  the same four fraction bits.
- **The colour arithmetic is not settled**: with D-30's IDCT, `frame`'s 15-bit picture differs at 11,035 of 76,800
  pixels and its 24-bit picture at 63,378 halfwords. The residuals form patches of one chroma row by four samples,
  which points at the chroma blocks' own IDCT rounding amplified by the factors; fits of the factors (fractions of
  6–16 bits, chroma whole or with its fraction, G's two products summed or rounded apart) moved the misses by under a
  tenth [M]. Part of D-30.

### 3.5 Monochrome, 4-bit and the signed form

- `y_to_mono` [R]: the 9-bit wrap, then saturation to the signed byte.
- **4-bit is the 8-bit value rounded to four bits and saturated**, `min(15, (u + 8) >> 4)` [M]: it reproduces all 64
  pixels of `mdec/4bit`'s console log from `mdec/8bit`'s, of the same block; truncation misses 20.
- Unsigned output adds 128 [R]; the signed 4-bit form is the same rounding on the signed value, two's complement [C].
- Bit 15 is set on 15-bit pixels when the command asks [R].

## 4. The pipeline and its time

### 4.1 The input FIFO, held

`mdec/step-by-step-log` writes its 256 words one by one with a delay of 10,000 cycles and prints the status before each
[R its `main.cpp`]; its console log fixes how the input moves [M, its 1,563 status and data lines]:

- The decoder takes nothing until the FIFO holds 32 words or the command's last one: no output appears in the first
  31 writes although the first blocks need three.
- Once full the FIFO is the decoder's until it has taken every word of it; status bit 30 reads set and DREQ0 clear
  the whole while, and a word leaves only when both its halfwords are taken.
- A block's last halfword is not taken while the IDCT still holds the block before it.

### 4.2 The output: the IDCT's slot, the block buffer and a FIFO of seventeen words

From the same log: a decoded block waits in the IDCT until the **block buffer** is empty; the buffer feeds an output
FIFO of **16 words and the word ready to be read**, so that the next block moves on when 17 words are left, after
fifteen reads of a 15-bit block, as the log shows at every block. The status's block field is the block in the IDCT,
or else the one arriving (psx-spx's "current output block" does not fit the log: it shows Y2 while Y1 is read). The
colour conversion happens as a block enters the buffer, with its macroblock's chroma, so the next macroblock's Cr waits
behind the last Y of the one before.

**DREQ1** is set while words wait, but clears once the block buffer is down to its last eight words with no block
behind them while input is still to come, and clears for the very last word of a command — the two places the log
shows it clear (psx-spx: "cleared after reading the first some words of that block"). D-32 records the fit.

On the log, as far as Diamond's 64 KiB TTY space holds it (850 of its 1,563 lines): **414 of 422 status lines equal**,
the eight others the third phase's Cr, which the console shows arriving eight reads later than here; 192 of 192 words
written and 208 of 225 words read equal, the 17 others D-30's colour.

### 4.3 The decoder's time: 512 cycles a block

psx-spx: "MDEC decompression time is still unknown" [R]. No console can be used (the plan's §1.3, decided
2026-10-11), so `programs/mdec-timing.s` measures five streams of 96 blocks decoded by DMA, from DMA0's start to each
channel's end, on Diamond and the referees as black boxes [M, `refall.sh`, timer 2 at the system clock / 8]:

| Stream | Diamond | Beetle PSX | SwanStation | PCSX-Redux | PCSX-ReARMed |
|---|---|---|---|---|---|
| A: 16 colour macroblocks, DC only, 15-bit | FCEh / 1809h | FBAh / 17FCh | Eh / 15D1h | 804h / 804h | E80h / E80h |
| B: the same with 63 AC values a block | 178Dh / 1809h | 1774h / 17F2h | 148Bh / 1743h | 804h / 804h | E80h / E80h |
| C: 96 DC-only blocks at 8 bits | FCEh / 1808h | FB7h / 17F7h | Eh / 3F07h | 2h / never | Dh / never |
| D: A at 24 bits | FCEh / 180Dh | FC5h / 17F6h | Fh / 165Ah | C02h / C02h | never |
| E: B's blocks at 4 bits | 178Dh / 1806h | 1774h / 17F2h | never / 1F8Bh | 2h / never | never |

(ticks of DMA0's end / DMA1's end; DuckStation's run ended before its RAM was read and is not counted.) Beetle PSX
takes 511 cycles a block whatever the depth and content; SwanStation, of DuckStation's lineage, 465 for stream A and
more with AC values or monochrome; the PCSX family finishes at once or never. No two lineages agree on a figure, so the
ruling names the strongest witness [C, `Diamond_Referees.md` §3]: **512 cycles from a block's last halfword to its
result, at every depth**, with Diamond's own input and output pipeline (§4.1, §4.2), puts every one of Diamond's ten
ticks within 0.3% of Beetle's, the DMA0 ends included, which the pipeline was not fitted to. This is D-31, open, a
console the only oracle that would settle it.

### 4.4 A read while a block is computed waits for it

Peter Lemon's `DCTBlockDecode` programs write a two-word decode command and read the block's 16 or 8 words at once,
polling nothing [R their `.asm`]; their author says he tests only on a console. With a read answered from an empty FIFO
every block came out one late, the first missing [M]; with the reader held until the block in the IDCT is done, all
seven blocks appear, as SwanStation draws them (PCSX-ReARMed draws stripes) [M, the probe's screens]. D-33 records the
rule, argued from a program's own behaviour on hardware.

## 5. DMA channels 0 and 1

- **Word by word**: an MDEC channel's block (sync mode 1) or burst (mode 0) moves one word at a time at the channel's
  17/16 cycles a word, bringing the MDEC to each word's cycle, while the MDEC can take or give; a block it cannot
  finish keeps its words left (the bus's `mdec_left`) and lets go of the bus until its request returns, so that DMA0
  and DMA1 can each wait for the other. DREQ starts a block; within it, only the FIFO's room or words matter [A,
  psx-spx "DMA Channels": DMA0 and DMA1 in blocks of 20h].
- **DMA1's order**: psx-spx says the output is "always output as a 8x8 pixel bitmap" and that DMA1 "is doing the
  re-ordering automatically" for colour macroblocks [R]. Diamond writes each 15- or 24-bit word of block Y1–Y4 at its
  place in the macroblock's rows of 16 pixels relative to where the block order would put it; the CPU's reads keep
  the block order. `frame`'s DMA builds draw their pictures with it [M, §7], and the CPU builds, which swizzle in
  software [R `mdec.cpp`], agree with them on the stripes they finish.
- ps1-tests' `mdec/8bit` (release build) asks DMA1 for one block of 32 words where the block has 16: the channel takes
  the 16, waits, and the program's own timeout ends it, as on DuckStation, whose TTY prints the same timeout [M].

## 6. The state

`DMND` **version 7**: the bus block gains `Mdec` — the registers, the input FIFO and its hold, the halfwords taken from
the word in hand, the tables, the run-length decoder's block, the IDCT's block with its time and result, the chroma, the
block buffer and the output FIFO with each word's block and index — and `MdecLeft`, the DMA's words left of a
waiting block. **3,789,828 bytes** with a DualShock on port 1 and no card, 1,786 more than version 6 [M,
`the_version_7_layout_is_pinned`]. A state saved mid-macroblock continues the same (`a_state_mid_macroblock_…`).
The SPU's stage 8, built beside this one, also changes the version; whichever merges second takes the next.

## 7. The gate

ps1-tests' release (build 158) on the open kernel, VRAM against each program's `vram.png` as ps1-tests' `diffvram`
compares them (WiseMan's `VramGrader`, `Ps1MdecTests`) [M]:

| Program | Frame | Result |
|---|---|---|
| `mdec/4bit` | 300 | **exact** |
| `mdec/8bit` | 300 | one pixel differs, (35,35): 240 here, 239 on the console (D-30); exact with it named |
| `mdec/frame/frame-15bit-dma` | 300 | 11,035 of 76,800 pixels differ (D-30's colour) |
| `mdec/frame/frame-24bit-dma` | 300 | 63,378 of 115,200 halfwords differ (D-30) |
| `mdec/frame/frame-15bit`, `-24bit` (CPU) | — | not graded: each waits after its last word for an output that a console empties, as the step-by-step log shows, so it never copies its twentieth stripe [R `mdec.cpp`; M]; the pictures are the DMA builds' |
| `mdec/movie/movie-15bit` | 315 | 319 of 61,440 pixels differ (D-30); the picture is the movie's frame 64, in VRAM for one frame of the run |
| `mdec/movie/movie-24bit` | 309 | 752 of 92,160 halfwords differ (D-30); the movie's frame 62, one frame |
| `mdec/step-by-step-log` | — | 414 of 422 status lines as the console's log (§4.2, D-32) |

The movie programs' pictures were taken while the movie played; that each is matched at a single frame depends on the
decoder's time (§4.3), and a change to the CPU's, DMA's or GPU's timing moves the frame. **The gate is not met
exactly**: `4bit` is, `8bit` is with one pixel named, and the rest differ by D-30's counts, which the WiseMan test pins.

## 8. Peter Lemon's programs

`MDEC/DCTBlockDecode/CLUT8BPP` and `CLUT4BPP`, against the probe's screens of the referees, up to the colour map the
most pixels agree on [M]:

| Program | SwanStation | PCSX-ReARMed | Beetle PSX |
|---|---|---|---|
| `CLUT8BPP` | 3 of 640 pixels off | stripes: its MDEC answers the unpolled reads with nothing | a black screen at frames 400 and 900 |
| `CLUT4BPP` | 233 of 640 | stripes | black |

`CLUT4BPP`'s three blocks at U = 8, 24 and 40 are black on Diamond although VRAM holds their texels: each shares a
texture cache line with the block drawn before it, written to VRAM after that line was read and never flushed, and
psx-spx says the cache must be flushed after VRAM is changed [R "GPU", "Clear Cache"]; SwanStation, which draws them,
does not model the texture cache this way. That is the GPU's D-23, not the MDEC. The bundled PNGs are of unstated
origin, as stage 4 found for the GPU programs.

## 9. The tester's games, and D-29

Through the kernel's `LoadExec` on the player's BIOS (the run tool's `--boot`, `Diamond_CDROM.md` §9.4), pictures every
120 frames to frame 3,600, the MDEC's commands counted [M]:

| Game | With the MDEC | Why it stops, where it does |
|---|---|---|
| Rayman | **its opening movie plays** (355 decode commands to frame 3,000), still playing at frame 3,600; eight of its frames from 1,680 to 2,640 lie 1.8–6.5 grey levels from SwanStation's nearest frame on 160×120 grey, at a constant offset of 900 frames (SwanStation boots through the intro), so its pacing is the referee's | — |
| Tekken 3 | "NAMCO PRESENTS", then **its title screen** from frame 1,920 and its attract mode's fight from 2,760 | its movie is skipped: one decode command, 122EE3E0h, is no decode (its top bits are MDEC(0)), and the game resets the MDEC ten seconds later |
| Castlevania: SotN | black throughout; VRAM holds the title's pictures by frame 1,500 | it writes 120F0004h, again no decode, resets the MDEC after 41 million cycles and writes it again, fifteen times in 3,000 frames |
| Silent Hill | its content warning, the Konami and KCET logos and the memory card warning, then black | it uploads the tables and sends no command |
| Ape Escape | its DUALSHOCK screen, then white, as at stage 7 | it never writes the MDEC (psx-spx lists its intro as "Polygon Video") |

In Tekken 3 and Castlevania the command word comes from the frame the game decodes in software from what it read off
the disc, so the stream reaching that stage, not the MDEC, is the suspect: CD-ROM streaming or the SPU (stage 8) [A].
Castlevania's black screen is the same at stage 7's commit [M, the same run on `f719b368`]; stage 7's record of its
Konami logo was not reproduced with this tool's flags.

**D-29**, observed by the screen only: with a disc in the drive the BIOS draws its logo by frame 450 and holds it to
frame 1,500, as before [M]. The MDEC was not what the intro waited on; the SPU is the candidate left.

## 10. The cost

[M, `examples/run`, 600 frames, three runs each on one core, this stage's build and WiseMan at `f719b368` built side by
side, ms a frame]:

| Program | Before | After |
|---|---|---|
| `timers` | 6.92–6.98 | 6.70–6.79 |
| `gpu/triangle` | 7.43–7.51 | 7.35–7.38 |
| `gpu/rectangles` | 6.44–6.45 | 6.24–6.33 |
| `mdec/movie/movie-15bit` (an FMV) | 6.17–6.22, its DMA never ending | 5.28–5.29, decoding |
| `mdec/movie/movie-24bit` | 6.21–6.22 | 5.35–5.37 |

The programs that use no MDEC run 1–3% faster, which the stage's code does not explain (it adds an event source and a
branch to the DMA); with fat LTO, stage 7 saw the code's layout move such figures by as much (`Diamond_CDROM.md` §12).
The movies are cheaper decoding than the stub was, whose program spun on a DMA that never ended. A block costs 313 ns
DC-only and 888 ns with 63 AC values at 15 bits, 1,053 ns at 24, after the IDCT skips pass 1's columns that are all
zero and sums in 32 bits, both exact (each pass's sums fit 32 bits for any table; the gate's counts unchanged) [M, a
bench of 12,000 blocks; 629 and 1,185 ns before]. At the plan's 1,200 blocks a frame that is 0.4–1.1 ms, the plan's
0.3–1.0 ms estimate. Nothing allocates: the FIFOs, the blocks and the output are fixed arrays.

## 11. What waits

- D-30, the IDCT's and the colour conversion's arithmetic; D-31, the time a block takes; D-32, Cr's turn in the
  pipeline; D-33, a read's wait. Each would be settled by a console, which is not to be used; D-30's search is the
  record of what the console's pictures exclude.
- Tekken 3's and Castlevania's movies, and Silent Hill's, behind the stream feeding their software decoders (the CD's
  streaming or the SPU); Ape Escape's polygon intro.
- The MDEC under DMA chopping, and DMA1 started mid-macroblock, which no program here does.

## 12. Tests run (2026-10-11)

`cargo test --release` in the crate: 109 unit tests (the MDEC's nine among them) and 45 program tests, with and
without `EMUSEN_DIAMOND_PS1TESTS`, all passing (the disc tests skipped for want of their variables, as before). The
program tests add `mdec-timing.s`, the monochrome programs' blocks against their console logs, the frame programs' CPU
and DMA builds, and the step-by-step status log. WiseMan under the test lock with the corpus: `DiamondTests`,
`SystemPack`, `CoreAbi`, `CoreAdapter`, `CoreDiscovery` and `CoreRegistration`, 95 of 95, the new MDEC picture test
among them.

## 13. Reading log, and exposures

**Read**: psx-spx at `6d7d1bc`: `macroblockdecodermdec.md` whole; `dmachannels.md`'s MDEC rows, "DMA Transfer Rates"
and "DRAM Hyper Page mode"; `graphicsprocessingunitgpu.md`'s texture cache and "Clear Cache" paragraphs;
`cdromfileformats.md`'s list of STR types, by a search for "MDEC". ps1-tests: the `mdec/` programs' `main.cpp`,
Makefiles, `movieToMdec.sh` and console logs; `common/mdec.h`, `mdec-tools.cpp` and `.h`, `delay.c`, `io.c`'s first
lines, `hexdump.cpp`'s first lines, and **`common/mdec.cpp` whole**, the release's MDEC driver (its header names no
copied source; it includes `dma.hpp`); `cpu/io-access-bitwidth/main.cpp` and its log's MDEC lines. Peter Lemon's
`DCTBlockDecodeCLUT8BPP.asm`, `CLUT4BPP.asm`'s decode and copy lines and `LIB/PSX_MDEC.INC`, and his `README.md`.
The release's own data (tables and streams) from its program images, as data.

**Not read**: any emulator's or library's source; PSn00bSDK's and PsyQ's libraries; ps1-tests' `common/dma.hpp`
beyond its first fifteen lines, which name it "Copied from Avocado", and `common/dma.cpp` beyond its first fifteen; the
BIOS; Ape Escape's, Tekken 3's and Castlevania's code (only the words they wrote to the MDEC's two registers, by a
temporary print, were looked at, and the print was removed).

**Exposures, stated plainly**: (1) the header check of `common/` showed the first fifteen lines of `dma.hpp` (its
Avocado notice, the channel enumeration and four register addresses) and of `dma.cpp`; nothing of either was used
beyond what psx-spx gives. (2) The run tool printed the BIOS kernel's start-up banner when a game was first run
without a heading filter, and the tail of one DuckStation log, read to see why its run failed, held the same banner;
nothing else of the BIOS's output was read, and the later runs filter it. (3) The release's driver `mdec.cpp` was read
to learn why the frame program's CPU build stops; it is a program's use of the MDEC, not a model of it, and the rules
it bears on (the waits on status bits 31 and 30) were taken from the console's step-by-step log.
