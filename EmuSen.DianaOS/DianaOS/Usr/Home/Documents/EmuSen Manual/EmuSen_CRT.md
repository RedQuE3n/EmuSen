# EmuSen_CRT — a physically modelled CRT filter

A design (2026-10-07), built the same day: the Accurate tier (§11), the two lower tiers (§12), and the filter offered to players (§13). The request was for a very accurate CRT filter, offered in tiers if it turns
out to be a performance hog: the lowest tier for performance, the highest for accuracy. This document is step 1 of
four: the survey of existing shaders, the physical literature with a source for every number, the choice of
delivery path, the model, the tiers with their predicted costs, the player's settings, and the questions that needed
a decision before any shader was written, with their answers. Step 2, the Accurate tier, is §11; steps 3 and 4 (the two reductions, the settings window) will add their
sections as they are built.

**Reading order.** §11 and §12 for what exists and what it measured. §1 for what is modelled. §4 for where it runs and what `FilterChain` must learn first. §6 for
the tiers. §9 for the decisions. §2 and §3 are reference: who already does what, and what the hardware
measured.

**Provenance marks**, used throughout §3: **[P]** the figure was read in the primary document (a standard, a
datasheet, a paper's own equations); **[S]** it comes from a secondary source (a wiki, a forum post by a named
measurer, a compilation that cites its own sources); **[D]** it is derived here by arithmetic from cited figures,
with the arithmetic shown; **[C]** it is a choice, with no measurement behind it; **unfound** means no source was
located and no number is used.

---

## 0. Status

| Step | What | State |
|---|---|---|
| 1 | Research and design (this document) | written 2026-10-07 |
| 2 | The Accurate tier, its render tests, its cost on the RX 6800 at 1080p and 4K | built 2026-10-07: §11, with `FilterChain`'s extensions in `EmuSen_Serenity.md` §3.9 |
| 3 | Balanced and Performance as reductions, each measured against Accurate | built 2026-10-07: §12 |
| 4 | Graphics settings per console, the settings reference, the fit audit | built 2026-10-07: §13 |

The filter exists as `CrtFilter.Filter` and is drawn by the tests and the bench; **it is not yet in a console's list**
(§11.11). The existing **CRT (Lottes)** (`EmuSen_Serenity.md` §3.4) and **Simple CRT** stay as they are.

---

## 1. What is modelled

A consumer CRT television shows a console's picture through a chain of physical stages, and each stage leaves a
mark that players remember as "the CRT look". The filter models the chain in the order the signal travels it:

1. **The console's encoder.** The console turns its pixels into an analogue signal: RGB, S-Video (luma and
   modulated chroma on two wires) or composite (both on one). The chroma rides a 3.58 MHz subcarrier whose phase
   relation to the pixel clock is fixed by the console's own clock dividers, and differs per console (§3.2).
2. **The television's decoder.** A composite signal must be split back into luma and chroma, and it cannot be
   split perfectly: fine luma detail near the subcarrier is read as colour (rainbowing), and chroma left in the
   luma shows as crawling dots. The chroma channel is narrow (about 0.5 MHz in a consumer set), so colour smears
   horizontally. Games were drawn with this in mind: the Genesis's dithered waterfalls and the SNES's
   pseudo-hi-res transparency both rely on the decoder blending adjacent columns.
3. **The tube's transfer.** Light is a power of drive voltage (about 2.4), with a floor set by the brightness
   control.
4. **The beam.** Each gun paints a spot whose profile is close to Gaussian and whose width grows with beam
   current. Scanlines are therefore thin in dark areas and fat in bright ones, and a bright line's light is
   spread, not lost.
5. **The phosphors.** Three phosphors with their own chromaticities, which are not the sRGB primaries, and their
   own decay, which is not exponential for two of the three.
6. **The mask.** An aperture grille, a slot mask or a shadow mask stands between the guns and the phosphors, at a
   physical pitch that is a property of the set, not of the picture.
7. **The glass.** Light scatters inside the thick faceplate (halation and veiling glare), the three beams do not
   land exactly together (convergence error), the faceplate is curved, and the raster is larger than the visible
   area (overscan).
8. **The player's display.** It is not a CRT. Its pixels are larger than a phosphor stripe at most resolutions,
   it has its own subpixel layout, its own primaries, a peak brightness that the mask's darkening eats into, and
   it holds each frame for the whole frame time where a CRT flashed it.

"Accurate" here means that each stage is computed from its physics with constants that have a source, that
energy is conserved from stage to stage (a mask or a scanline redistributes light, it does not remove it unless
the real one did), and that the result is checked against published measurements (§8). Where no measurement was
found, the constant is marked a choice and is one of the things a player may set.

---

## 2. The existing shaders

Surveyed from libretro's `slang-shaders` pack as built on 2026-09-22 (`EmuSen_Serenity.md` §7), by reading the
sources. The pack's own files are the citation; paths are relative to its root.

### 2.1 What each models, and how

| Shader | Signal | Transfer | Beam | Mask | Colour | Persistence | Glass, geometry |
|---|---|---|---|---|---|---|---|
| **crt-lottes** (`crt/shaders/crt-lottes.slang`) | none (RGB) | sRGB curve both ways | `exp2(hard·d^shape)`, fixed width; 3 lines × 3 to 7 taps | 4 fixed patterns, 3 px triad in output pixels; dark 0.5, light 1.5; no anti-aliasing | none | none | wide copy of the beam kernel as bloom; 2D barrel warp |
| **crt-geom**, **geom-deluxe** (cgwg) | none | 2.4 in, 2.2 out | Lanczos2 across; super-Gaussian down, exponent `2 + 2c⁴` so a bright line is flatter; 3× vertical oversampling | subpixel tables in output pixels; **energy-conserving** compensation from each layout's lit fraction | none | deluxe: feedback, **power law** `t^-1.2` | sphere by ray intersection with radius, distance, tilt, overscan; deluxe adds a 9-tap halation and a raster that grows with mean brightness |
| **crt-royale** (TroggleMonkey) | none | 2.5 in, 2.2 out | Gaussian or generalised Gaussian **per channel**, σ from 0.02 to 0.3 line as `c^(1/3)`; 3 lines; optional exact integral | image tiles of 8 triads, **resampled** to the wanted triad size by Lanczos or mipmaps; pitch in output pixels or triads across | none | none | energy-motivated bloom (the share of light that must spread to stay ≤ 1); halation and diffusion from a 320×240 blur; per-gun convergence; sphere or cylinder, ray-traced, with 12-sample anti-aliasing |
| **crt-guest-advanced** (guest.r) | -ntsc: Themaister's crosstalk matrix, 24 or 32 tap FIR; -pal: V-switch and a delay line | 2.4 both ways | `exp2(-s·(x·k)²)`, `k` from 1.3 (dark) to 1.0 (bright) on the max channel; 6-tap Gaussian across with negative lobes | procedural, integer output pixels, no anti-aliasing; multiplied in a "mask gamma" space; brightness boost | six CRT profiles (EBU, P22, SMPTE-C, Trinitron…) → XYZ → display gamut; D65 to D93 by interpolation | feedback, `max(mix(…))`, factor about 0.81 a frame at strength 0.2 | glow and bloom at 800×600; per-gun convergence; 2D warp; raster bloom |
| **Mega Bezel** (HyperspaceMadness) | GTU bandwidths, optional | **Dogway's Grade**: a BT.1886 Appendix-style two-segment curve | guest-advanced's | guest-advanced's | **Grade**: phosphor primaries in xy (SMPTE-C, EBU, P22 of the 80s and 90s, NTSC-J), white from a daylight-locus polynomial, **CAT16** adaptation, gamut compression | a pass exists, unused by the presets | royale's sphere and cylinder; generated bezel with the screen reflected in it |
| **Sony Megatron** (MajorPainTheCactus) | none; NTSC presets borrow guest's | 2.22 in | per channel, 3 lines, cubic Bézier, width `mix(0.5, 1.0, signal)` | **binary, per display subpixel**: tables by display class (1080p, 4K, 8K) × TVL (300 to 1000) × panel layout (RGB, BGR, RWBG); no anti-aliasing, relies on integer alignment | phosphor sets → 709; D93 for NTSC-J | none | none; brightness recovered by **HDR** (inverse tone map, PQ), so SDR is simply darker |
| **crt-beans** (A. Duffey) | Y at 4.0 MHz, I/Q at 0.6 MHz, **in MHz** over a 53.33 µs line; Hann kernel integrated over each texel | 2.4 | separable cubic spot normalised by `1/w²`, width `0.9·(0.4 + 0.6√v)` | aperture grille as **triads across the screen** (550), **anti-aliased in closed form** by a one-pixel Hann; energy-conserving blend | YIQ decode only | none | glow as a wide Gaussian at 4% |
| **Scanline Classic** (W. M. Martinez) | the most complete: DAC, encoder with setup, **real composite sum** `g(Y+o) + U sin φ + s V cos φ`, notch or comb or adaptive comb, RF at complex baseband with noise and ghosts; carrier phase per pixel, line and field from each console's clock and divisor; every filter a Gaussian given as "A dB at f MHz" | **BT.1886 form**, then per-gun nits | Mitchell-Netravali resample of a zero-stuffed raster; **no width growth** | procedural Gaussian stripes by TVL; three regimes by output pixels per unit: point-sampled, box-integrated numerically, **switched off below Nyquist**; exact brightness make-up from the mask's mean and peak | SMPTE-C, EBU, "Japan" primaries and white; Bradford or CAT16; SDR tone map, wide-gamut and HDR outputs | feedback, **closed-form inverse power law** `(y^(-1/n) + γΔt)^(-n)` per channel, τ(10%) 1 ms red, 100 µs green and blue | deflection and faceplate geometry from angles; bezel |
| **cathode-retro** (DeadlyRedCube) | real encode and decode at 2 samples per subcarrier cycle; box filters one and two cycles long | 2.2 / 2.0 | — | generated once into a texture, **mipmapped** | — | previous frame blended | diffusion from a downsampled blur |
| **mame_hlsl** | composite `Y + I cos + Q sin` at 4 samples per texel; 64-tap windowed sincs; notch at fsc | — | `sin²(πy)^w`, `w` falling with luma | tiled texture | SMPTE-C primaries in xy → XYZ → sRGB | `max(now, prev·0.45)` | per-channel linear and radial convergence |
| **patchy-ntsc** (PlainOldPants) | per-console phase tables (NES, SNES, Genesis static rainbow); sinc filters in MHz on both encoder and decoder; "jungle chip" demodulation axes, US and Japan | BT.1886 | — | — | P22 gamut tables from gamutthingy | — | — |
| **Blur Busters beam simulator** | — | display gamma | temporal only: a rolling scan with phosphor fade across sub-frames on a 120 Hz+ display | — | — | within the frame | — |

Also read: crt-hyllian (cubic or sinc across, `exp(-16d²)` down with width 0.72 to 1.0 by channel); crt-yah
(Hyllian's beam, deconvergence, a persistence pass); crt-consumer (a single-pass decoder with the console's pixel
clock as a parameter); crt-easymode-halation; gtu (Y/I/Q band-limited with no subcarrier, the Gaussian beam
integrated over the output pixel by an erf approximation); crtsim, newpixie and koko-aio (looks, not models);
maister's and blargg's NTSC (the common ancestors of most NTSC presets); pal-r57shell (a real PAL delay line).

### 2.2 Licences

| Licence | Shaders | May a formula be taken closely into GPL-3.0 code? |
|---|---|---|
| Public domain, CC0 | crt-lottes; crtsim; pal-r57shell; newpixie (also MIT) | yes |
| MIT | crt-hyllian; crt-beans; crt-yah's core; cathode-retro; Blur Busters beam simulator | yes, with the notice |
| BSD | mame_hlsl (3-clause); pal-singlepass (2-clause) | yes, with the notice |
| GPL-2.0-or-later | crt-geom, geom-deluxe, cgwg-fast; crt-royale ("version 2 … or any later version"); crt-guest-advanced's own files; crt-consumer; Dogway's Grade | yes |
| GPL-3.0-or-later | Scanline Classic's shaders; Mega Bezel's own files | yes |
| GPL-3.0, version wording unclear or "only" | gtu ("License: GPLv3"); patchy-ntsc (version 3) | yes while EmuSen is GPL-3.0 |
| **No licence stated** | Sony Megatron and crt-adamant; maister's NTSC and ntsc-adaptive; guest's `ntsc-pass*` and `pal-pass*` files; the blargg port; nes-color-decoder; cgwg-famicom-geom; ntsc-blastem; gdapt and mdapt; Scanline Classic's bezel images | **no**: run for comparison only |
| Non-free or awkward | mame-ntsc (non-commercial); ntsc-xot and the decoder inside koko-aio (CC BY-SA) | **no** |

No GPL-2.0-only header was found in anything surveyed.

**What will be taken, and from where.** The code is to be written from the physics and from the published
descriptions in §3. Nothing above is to be copied. Three techniques are close enough to one author's formulation
that they will be credited where they are built, here and in `THIRD_PARTY_NOTICES.md`, if step 2 does use them:
the closed-form inverse-power-law decay step (Scanline Classic, GPL-3.0-or-later), the closed-form
anti-aliasing of a stripe mask by a pixel-wide window (crt-beans, MIT), and the per-subpixel reading of the mask
(the Megatron's idea, which has no licence and so is reimplemented from the idea alone: a display subpixel of one
colour can only show the phosphor of that colour). The erf approximation is Abramowitz and Stegun 7.1.26, a
formula from a handbook.

### 2.3 What the survey shows

- **No shader sets the mask's pitch in physical units.** Royale and beans come closest (triads across the
  screen); the rest count output pixels, so the same preset is a different television at every window size.
- **Only royale, beans and Scanline Classic treat the mask's aliasing at all**, by resampling, by a closed form,
  and by switching the mask off. The Megatron sidesteps it with integer subpixel patterns, which fixes the pitch
  to the display.
- **Energy is conserved under the mask by geom, beans and Scanline Classic only**, and under the scanline by
  beans (its `1/w²`) and royale's integral. Everywhere else brightness is a boost parameter.
- **A real composite sum, decoded by a real filter, is the minority**: Scanline Classic, cathode-retro,
  mame_hlsl, patchy-ntsc, cgwg-famicom and gtu-famicom. The widely used Themaister family multiplies by a
  crosstalk matrix, which reproduces the look of artifacts but not which pattern makes which colour.
- **The shaders disagree about the Genesis.** The "two-phase" model of the Themaister family inverts the phase
  each line and each frame, which crawls; patchy-ntsc and MAME's `ntsc-md-rainbows` hold it still. §3.2 derives
  from the console's clocks that still is right.
- **Persistence is modelled five ways** (power law, closed-form power law, peak hold, linear blend, two-rate
  feedback) with constants that cite nothing. §3.5 has a measurement.
- **Nothing models the beam's width growth from a measurement**; every shader that has it interpolates between
  two chosen widths.
- **Only the Megatron reads the mask per display subpixel, and only the Megatron and Scanline Classic address
  brightness in nits.**

---

## 3. The physical literature

### 3.1 The signal standards

**NTSC (SMPTE 170M-2004 [P]; ITU-R BT.470-6 [P]).**

- Subcarrier `fsc = 5 MHz × 63/88 = 3.579545 MHz`. A standard line is 227.5 subcarrier cycles, 63.556 µs.
  Horizontal blanking 10.7 to 10.9 µs, so **[D]** the active line is 52.66 to 52.86 µs. A field is 16.6833 ms.
- Levels: white 100 IRE, black set-up 7.5 IRE, burst 40 IRE peak to peak, sync −40 IRE. Japan has no set-up:
  black and blanking are both 0 IRE (BT.470 notes [P]).
- Luma `Y = 0.299 R + 0.587 G + 0.114 B`. `U = 0.493 (B−Y)`, `V = 0.877 (R−Y)`;
  `I = −0.27 (B−Y) + 0.74 (R−Y)`, `Q = 0.41 (B−Y) + 0.48 (R−Y)` (BT.470 Table 2).
- Composite `N = 0.925 Y + 7.5 + 0.925 Q sin(ωt + 33°) + 0.925 I cos(ωt + 33°)` (170M Annex A).
- Bandwidths: luma is not restricted by 170M itself, transmission is "typically 4.2 MHz". Chroma for equiband
  B−Y and R−Y: less than 2 dB down at 1.3 MHz, at least 20 dB down at 3.6 MHz. The 1953 Q channel: less than
  2 dB at 0.4 MHz, at least 6 dB at 0.6 MHz.

**PAL (BT.470-6 [P]).** `fsc = 4.43361875 MHz`, line 64 µs, blanking 12 µs, video bandwidth 5 or 5.5 MHz, U and
V less than 3 dB down at 1.3 MHz. The V component's sign alternates each line.

**What a consumer receiver did.**

- Chroma: "employed in many color television receiver systems … 1 MHz bandwidth centered about 3.58 MHz", so
  about **0.5 MHz** at baseband (RCA, US 4,620,220, 1986 [P]). Receivers decode equiband B−Y and R−Y, not
  wideband I and narrow Q (170M Annex A.4).
- Luma: low-cost sets use "a simple notch filter or subcarrier trap at 3.58 MHz", after which "there is little
  useful information above 2.3 MHz"; a two-line comb keeps the horizontal detail at the cost of the diagonal
  (Faroudja, *NTSC and Beyond*, IEEE Trans. Consumer Electronics 34(1), 1988 [P]).
- Demodulation was deliberately non-standard. The decoder chips' datasheets give the angles of the R−Y, G−Y and
  B−Y axes and their relative gains, and the US modes rotate and boost red ("red push") to keep skin tones
  pleasing on a 9300 K white. As transcribed by gamutthingy [S], which cites each datasheet; the Sony PDFs
  themselves could not be fetched and **must be read before any of these is built in**:

| Decoder | R−Y, G−Y, B−Y angles | Gains |
|---|---|---|
| Ideal (the standard's matrix) | 90°, 236°, 0° | 0.56, 0.34, 1.0 |
| Sony CXA2025AS, Japan mode | 95°, 240°, 0° | 0.78, 0.30, 1.0 |
| Sony CXA2025AS, US mode | 112°, 252°, 0° | 0.83, 0.30, 1.0 |
| Sony CXA2060BS (a PVM's), Japan / US | 95° / 102°, 236°, 0° | 0.78, 0.33 / 0.30, 1.0 |
| Toshiba TA7644BP | 107°, 240°, 0° | 0.95, 0.31, 1.0 |
| RCA Colortrak 1989, computed from a measurement | 94.5°, 255.4°, 0° | 0.807, 0.295, 1.0 |

### 3.2 The consoles

The subcarrier's phase at a pixel is fixed by the console's clocks. Three numbers decide every composite
artifact: subcarrier cycles per pixel, the fraction of a cycle left over per line, and the fraction left over per
frame.

| Console | Master clock | Dot clock | Cycles per pixel | Cycles per line | Per line | Per frame | Active picture |
|---|---|---|---|---|---|---|---|
| **NES** | 21.477272 MHz = 6 fsc | ÷4 = 5.369 MHz | 2/3 | 341 dots = 227⅓ | +⅓ | +⅓, +⅔ alternating | 256 dots, 47.68 µs |
| **SNES** | 945/44 MHz = 6 fsc | ÷4 = 5.369 MHz; 512 modes 10.74 MHz | 2/3; 1/3 | 1364 clocks = 227⅓ | +⅓ | +⅓, +⅔ alternating | 256 or 512 dots, 47.68 µs |
| **Genesis** | 53.693175 MHz = 15 fsc | H32 ÷10 = 5.369 MHz; H40 ÷8 = 6.711 MHz | 2/3; 8/15 | 3420 clocks = **228** | **0** | **0** | 256 or 320 dots, 47.68 µs |
| **N64** | VI clock 48.681818 MHz = 13.6 fsc | ÷4 = 12.17 MHz at 640; ÷8 at 320 | 0.294; 0.588 | 3094 quarter-pixels = 227.5 | +½ | +½ | 640 dots, 52.59 µs |

Sources: NES, nesdev's *NTSC video* and *Cycle reference chart* [S]; SNES, the SNESdev wiki's *Timing* [S];
Genesis, nesdev forum thread 24447 (lidnariq, tepples) and Kabuto's hardware notes [S]; N64, n64brew's *Video
Interface* and *Video DAC* [S]. The right-hand columns are **[D]**:

- **NES and SNES.** A line of 227⅓ cycles shifts the pattern by a third of a cycle each line: a three-line
  diagonal. A frame of 262 such lines is 59561⅓ cycles; on alternate frames one dot (NES, rendering enabled) or
  four master clocks (SNES, line 240 of the non-interlaced field) are dropped, giving 59560⅔. The pattern
  therefore alternates between two positions frame to frame, which at 60 Hz the eye partly averages: the familiar
  shimmer on vertical edges. With the NES's dropped dot suppressed (rendering disabled), it is a three-frame
  crawl.
- **Genesis.** 3420 ÷ 15 is exactly 228, and 262 lines of 228 cycles is a whole number, so **nothing moves**:
  every line and every frame has the same phase. Its artifacts are fixed vertical bands, and they are strong
  because of what dithering does at H40: alternating columns toggle at 6.711/2 = 3.356 MHz, 0.22 MHz below the
  subcarrier, which the decoder reads as chroma. The pattern is at 15/16 of the subcarrier (7.5 on-off cycles per 8
  subcarrier periods), so the decoded hue turns once every 16 periods: a rainbow 30 pixels wide. At H32 the
  same pattern is at ¾ of the subcarrier, 0.89 MHz away and well down the skirt of a 0.5 MHz chroma channel, so it
  blends with much less colour. This settles §2.3's disagreement in favour of the static model, and it is why
  the waterfalls in *Sonic the Hedgehog* blend and band at once.
- **N64.** A standard line and a 263-line progressive frame, so the phase inverts each line and each frame: the
  ordinary broadcast pattern, which cancels over two frames.
- **All three 21.48 MHz-family consoles have the same active picture**, 256 dots at 5.369 MHz = 47.68 µs,
  whatever the width mode. That is 170⅔ subcarrier cycles across the picture, a constant the signal stage can be
  built around: cycles per source pixel is `170.667 / width`.

**The encoders.**

- **NES.** The PPU has no RGB stage: it emits composite directly as a square wave at one of 12 phases, 8 samples
  to the pixel (the colour generator runs on both edges of the master clock, 42.95 MHz). Terminated levels,
  measured by lidnariq (±2 IRE) [S]: sync 48 mV, colour burst 148 to 524 mV, the four luma rows' low levels 228,
  312, 552 and 880 mV and high levels 616, 840, 1100 and 1100 mV, with $1D at 312 mV defined as 0 IRE; the
  emphasis bits attenuate to 0.816 of the level during their phases; burst is phase 8. **The palette an emulator
  draws with is already somebody's decode of this signal**, which is why an accurate NES composite needs the
  PPU's colour index and emphasis per pixel, not RGB (§9, question 5).
- **Genesis.** Sony CXA1145, datasheet [P]: RGB in, matrix, R−Y and B−Y modulators, Y and C mixed on the chip.
  The chroma leaves on pin 15 for an external band-pass and returns on pin 17, the luma leaves on pin 16 for an
  external delay line and returns on pin 18, and **there is no luma trap inside the IC**. RGB, Y and C responses
  are −3 dB at 5.0 MHz or above. The values of the board's own filters: **unfound**. Later boards use the
  CXA1645.
- **SNES.** S-ENC (a BA6592F in some units), RGB in, equiband R−Y and B−Y [S, a forum reading]. Its chroma
  pre-filter: **unfound**.
- **N64.** The VI filters the picture digitally before the DAC (anti-alias and resample, divot, de-dither, a
  square-root gamma), then VDC-NUS and ENC-NUS (a BA7242F), later MAV-NUS. Encoder bandwidths: **unfound**.

Where an encoder's filter is unfound, the model uses the standard's chroma limit (1.3 MHz) and the datasheet's
5 MHz for luma, and says so in the setting's description. These are the first constants a measurement should
replace.

### 3.3 The tube's transfer

ITU-R BT.1886 Annex 1 [P]: `L = a · max(V + b, 0)^2.40`, with `a = (Lw^(1/γ) − Lb^(1/γ))^γ` and
`b = Lb^(1/γ) / (Lw^(1/γ) − Lb^(1/γ))`, for a white of `Lw` and a black of `Lb` cd/m². Its Appendix 1 gives a
closer match to a CRT: exponent 2.6 above V = 0.35 and 3.0 below. Its reference white is 100 cd/m². Poynton's
*Gamma FAQ* (Q17) [P] puts measured CRT exponents "between about 2.35 and 2.55". BT.470 assumed 2.2 for NTSC
receivers and 2.8 for PAL, which are design assumptions for the camera, not measurements of tubes.

**White luminance.** A Sony PVM-20L5 measured 176 cd/m² peak, 0.01 cd/m² black (DisplayMate, *Display Technology
Shoot-Out* part 1 [P]). Consumer sets: **unfound** as a measurement; 100 cd/m² is BT.1886's reference, used here
as the default and marked **[C]**.

### 3.4 Phosphor chromaticities

| Set | Red x, y | Green x, y | Blue x, y | White | Source |
|---|---|---|---|---|---|
| NTSC 1953 | .67, .33 | .21, .71 | .14, .08 | C .310, .316 | BT.470 [P] |
| SMPTE C | .630, .340 | .310, .595 | .155, .070 | D65 .3127, .3290 | SMPTE 170M §4 [P] |
| EBU | .64, .33 | .29, .60 | .15, .06 | D65 | BT.470 annex [P] |
| Trinitron P22 (Sony's data, ±0.03) | .621, .340 | .281, .606 | .152, .067 | | Has and Newman 1995, via gamutthingy [S] |
| PVM-20M2U, measured | .630, .345 | .285, .605 | .150, .065 | | via gamutthingy [S] |
| RCA Colortrak 1989, measured | .623, .344 | .312, .590 | .154, .065 | about .301, .308 | via gamutthingy [S] |
| Mean of 15 monitors | .620, .342 | .288, .597 | .153, .070 | | Golz and MacLeod 2003, via gamutthingy [S] |
| ARIB TR-B9, "Japan specific" | .618, .350 | .280, .605 | .152, .063 | | via gamutthingy [S] |

No consumer set ever had the 1953 primaries; real P22 tubes sit close to SMPTE C and EBU, which is what those
standards were written to describe. **White in Japan** was 9300 K: "studio monitors adjusted to D-white at
9300 K" (BT.470 notes [P]); receivers 9300 K + 27 MPCD, x .281, y .311, master monitors 9300 K + 8 MPCD, x .2831,
y .2971 (Nagaoka 1979, via gamutthingy [S]). EBU Tech 3213 and SMPTE RP 145 themselves were not opened.

### 3.5 Phosphor decay: a measurement, and what it implies

P22 is blue ZnS:Ag, green ZnS:Cu,Al and red Y₂O₂S:Eu (Nichia's list, hosted by FH Münster [P]); the registry
classes them "medium short", "medium short" and "medium" by time to 10%, and the class boundaries in EIA
TEP116-C are paywalled and **unfound**. Elze (PLoS ONE 5(9) e12792, 2010 [P]) describes the decay as "initially
exponentially, later converging to a power law", cites Sherr (1993) for P22 10% times of 1.5 to 6 ms, and
measured his own monitor below 10% after about 400 µs.

The quantitative source is **Kuhn, *Optical Time-Domain Eavesdropping Risks of CRT Displays*, IEEE Symposium on
Security and Privacy 2002 [P]**, who measured one P22 monitor (a Dell D1025HE) with a photomultiplier and fitted
closed forms to its impulse response (his equations 8 to 10, t in seconds):

- Red: `4 e^(−2π·360 t) + 1.75 e^(−2π·1.6k t) + 2 e^(−2π·8k t) + 2.25 e^(−2π·25k t) + 15 e^(−2π·700k t) + 29 e^(−2π·7M t)`
- Green: `210×10⁻⁶ (t + 5.5 µs)^(−1.1) + 37 e^(−2π·150k t) + 100 e^(−2π·700k t) + 90 e^(−2π·5M t)`
- Blue: `190×10⁻⁶ (t + 5 µs)^(−1.11) + 75 e^(−2π·100k t) + 1000 e^(−2π·1.1M t) + 1100 e^(−2π·4M t)`

"The red phosphor, which decays purely exponentially, emits practically all of its stored energy within 1–2 ms";
green and blue are heavy-tailed.

**[D] Integrated over frame intervals of 16.683 ms** (computed 2026-10-07 from the equations above; the script is
a dozen lines of closed-form integrals):

| | Light in the frame it is struck, of all light in the first second | Light in the next frame, relative to the first | Second frame after | Third |
|---|---|---|---|---|
| Red | 100% | 0 | 0 | 0 |
| Green | 78.8% | **5.4%** | 3.0% | 2.0% |
| Blue | 81.1% | **4.7%** | 2.6% | 1.8% |

Share of the first frame's light emitted in its first millisecond: red 91%, green 74%, blue 77%.

*(2026-10-07, after building it: the next-frame figures above are an extrapolation eight times past Kuhn's last recorded point, and a sustained picture shows them to be unphysical. §11.4 retires them as the default and gives what replaced them. The first bullet below stands only in its direction, cyan-green and slow; the second is unaffected.)*

**What follows.**

- **Between frames, a P22 screen's trail is cyan-green and a few percent**, falling as roughly `1/k` with the
  frame count, not geometrically. In display code values (2.4 power) 5% is about a quarter of full scale, so on
  a black background it is visible, and on anything else it is not. Every shader in §2 that models persistence
  uses a per-frame factor (0.45 to 0.81) that is an order of magnitude stronger and the same for all three
  channels; those are afterglow as an effect, not a P22 tube.
- **Within the frame, three quarters or more of the light is out inside a millisecond.** A CRT's picture is a
  1 ms flash every 16.7 ms; a sample-and-hold display shows the same light spread over the whole frame. That
  difference, motion clarity and flicker, **cannot be shown on a 60 Hz display at all**, and is the one part of
  the tube this design leaves out (§5.9).
- **Confidence is low on the tail.** It is one monitor, of a computer and not a television; Kuhn drove it at the
  monitor's default settings, not the standard's 100 µA; zinc-sulphide decay "can vary significantly under
  different drive conditions"; and he states that the measurement was not designed to fix the power-law
  exponent. The fit's own range ends near 10 ms. The frame-to-frame ratios above do not depend on where the
  heavy tail is cut off (the total does), which is why they are quoted as ratios. The default strength is the
  measurement's; the setting lets a player take it to zero.

### 3.6 The beam

The spot's light is "usually a near-Gaussian intensity distribution", its size quoted at half intensity (a
*Journal of Digital Imaging* tutorial, 10.1007/BF03168105, read only as a search extract [S]). The spot is the
quadrature sum of the gun's magnified crossover with its space-charge growth and of spherical aberration,
`Dt = √((Dx + Dst)² + Dic²)` (LG.Philips, US 6,750,601 [P]), and it grows with beam current; one electron-gun
patent's summary gives the growth as about the square root of the current [S, unverified]. On a shadow-mask
tube "the spot size is considerably larger than the dot pitch, up to 2× or so at the corners" (Goldwasser, *TV
and Monitor CRT (Picture Tube) Information* [S]).

**Unfound:** any curve of spot size against beam current for a television tube, and the spot's size relative to
the line pitch for a consumer set against a PVM. These are the constants that decide how visible scanlines are,
and they are **choices** in step 2, to be fitted against photographs (§8.3) and recorded as fitted, not measured.
*(Fitted 2026-10-07 to one photograph of a Trinitron: σ about 0.26 mm at white, §11.5.)*
The form is fixed by the physics: a Gaussian whose area is the line's light, so that widening it lowers its peak
and conserves its energy, with `σ = σ₀ · (1 + k · I^p)`, `p` near ½.

### 3.7 The glass

- **Veiling glare, measured.** The PVM-20L5 above has a full-screen contrast of 17,600:1 but a 4×4 checkerboard
  contrast of 219:1 and a 9×9 of 75:1, because of "heavy reflections within its thick glass faceplate"
  (DisplayMate [P]). **[D]** A black square therefore sits at 0.46% of white in the 4×4 pattern and 1.3% in the
  9×9. These two numbers are the calibration target for the glass model: whatever kernel is used must reproduce
  both.
- **Faceplate glass** is specified at 10.16 mm thickness, in tints transmitting 86, 73, 57 and 46% at 546 nm;
  flat-square bulbs thicken from centre to edge, a 7% transmittance change (Compton, SPIE TT54, ch. 1 [P]).
- **The halation ring.** **[D]** Light leaving a phosphor at more than the critical angle is reflected at the
  outer surface and returns to the phosphor layer in a ring of radius `2t·tan θc`. For `t` = 10.16 mm and an
  index of 1.5 **[C]**, `θc` = 41.8° and the radius is 18 mm, 4.7% of a 20-inch picture's width. No published
  measurement of the ring was found; the derivation is elementary optics and the index is assumed.
- **Convergence.** Sony's PVM-M brochure [P]: PVM-14M4 0.4 mm at the centre and 0.5 mm at the periphery; 20M4
  0.5 and 0.7; 20M2 0.6 and 1.0 (typical). A consumer specification: **unfound**; a consumer default no better
  than the 20M2's is a **[C]**.
- **Overscan.** The same brochure: 7% in normal scan, 5% underscan. Safe action 90% × 90% and safe title 80% ×
  80% (SMPTE RP 8/13, 27.3 and 218, via NAB [P]); EBU R95 [P] 3.5% and 5% margins per edge. A measured
  distribution for consumer sets: **unfound**.
- **Curvature.** Early colour faceplates had a radius of 35 to 40 inches, flat-square ones 60 to 68 inches
  (Compton [P]). A Trinitron's face is "nearly flat vertically", a cylinder, because the grille's wires must be
  straight [S]. Per-model radii: **unfound**.

### 3.8 The masks

| Class | Set | Mask and pitch | Picture width | Triads across [D] | Source |
|---|---|---|---|---|---|
| Consumer, small | GE 13-inch | slot, 0.60 mm | 264 mm | 440 | Goldwasser [S] |
| Consumer, 20-inch class | Samsung 19-inch | slot, 0.75 mm | 386 mm | 515 | Goldwasser [S] |
| Consumer, 25-inch | RCA 25-inch | slot, 0.90 mm | 508 mm | 564 | Goldwasser [S] |
| Consumer, 27-inch | JVC AV-27D302 | slot, 0.87 mm | 549 mm | 631 | crtdatabase [S] |
| Consumer Trinitron, 27-inch | Sony KV-27S42 | aperture grille, 0.75 mm | 549 mm | 732 | crtdatabase [S] |
| Professional | Sony PVM-20M2 (600 TVL) | aperture grille, 0.40 mm | 384 mm | 960 | Sony brochure [P] |
| Professional | Sony PVM-14M4 (800 TVL) | aperture grille, 0.25 mm | 266 mm | 1064 | Sony brochure [P] |
| Professional | Sony PVM-20M4 (800 TVL) | aperture grille, 0.31 mm | 386 mm | 1245 | Sony brochure [P] |
| PC monitor | 15-inch, 0.28 mm diagonal | shadow mask, 0.24 mm horizontal | 280 mm | 1167 | Wikipedia, *Dot pitch* [S] |

The consumer sets' picture widths take the quoted size as the viewable diagonal of a 4:3 tube, which is a
**[C]**; the professional monitors' are from their brochure. A 14-inch or 20-inch consumer Trinitron's pitch:
**unfound**. BVM pitches (0.22 to 0.30 mm) were read only from
a manual's summary [S]. Television lines check against pitch as triads across × ¾: 797 for the 14M4, rated 800
**[D]**.

**Mask transmission.** Early shadow masks had "only 15% of [the] surface open" (Wikipedia, citing Gilmore [S]);
an aperture grille's open share is higher, without a figure that survives its own citation. A modern tube's
figure: **unfound**. It does not enter the model: what matters to the picture is the phosphor layer's lit
fraction, which is geometry (§5.6), and the tube's white luminance, which is §3.3's.

**Whether a display can show a mask** is arithmetic on the table **[D]**. A 4:3 picture is 1440 pixels wide at
1080p, 2880 at 4K, 1600 on a 1920×1200 handheld panel and 1067 at 1280×800:

| Set | Pixels per triad at 1067 | at 1440 | at 1600 | at 2880 |
|---|---|---|---|---|
| Consumer 13-inch slot | 2.4 | 3.3 | 3.6 | 6.5 |
| Consumer 20-inch-class slot | 2.1 | 2.8 | 3.1 | 5.6 |
| Consumer 27-inch Trinitron | 1.5 | 2.0 | 2.2 | 3.9 |
| PVM-20M2 | 1.1 | 1.5 | 1.7 | 3.0 |
| PVM-14M4 | 1.0 | 1.4 | 1.5 | 2.7 |
| PVM-20M4 | 0.9 | 1.2 | 1.3 | 2.3 |

~~Three stripes need at least three samples per triad to be told apart at all and six to be drawn without beating.
Whole pixels give that only for consumer sets at 4K. **Display subpixels** give three times as many samples, so a
consumer slot mask is drawable at 1080p (8.4 subpixels per triad) and a PVM's grille at 4K, and nothing draws a
PVM's grille at 1080p. §5.6 is built on this.~~ *(Retired 2026-10-07. Each colour is sampled once per pixel
wherever its subpixel sits, so subpixels do not multiply the samples of a coloured stripe. §11.7 has the
correction and the arithmetic that replaces this paragraph: a mask needs two pixels per triad to be drawn at
all, and is gone below one and a half.)*

---

## 4. Where it runs

### 4.1 The choice: the built-in SkSL `FilterChain`

Two paths exist (`EmuSen_Serenity.md` §3.2 and §7). The CRT filter goes on the first, for every tier.

**The argument.**

- It runs wherever Skia does, macOS's Metal included; the slang runtime needs a Vulkan device, which a Mac does
  not offer natively and a weak laptop may not.
- It costs the render thread almost nothing: a built-in Lottes is 0.09 ms of the render thread against 2.59 ms
  for the same shader as a slang preset, for the same GPU work, because the slang path reads its picture back
  and uploads it again (`EmuSen_Serenity.md` §8.3). On the handheld the gap is 0.30 against 9.39 ms (§8.4
  there). A filter whose lowest tier must suit a weak laptop cannot start 2 ms behind.
- **Nothing in the model needs what SkSL cannot express.** Its limits (§3.2 there) are constant loop bounds, no
  dynamic array indexing, no derivatives, no `textureLod`, no bitwise operators. Checked against §5 stage by
  stage:

| What the model needs | Does SkSL have it? | How it is done instead |
|---|---|---|
| FIR filters of 20 to 60 taps | constant-bound loops, yes | coefficients computed from the filter's formula per tap, not read from a table |
| Mask anti-aliasing | no mipmaps, no derivatives | closed-form integrals of the mask over the pixel's footprint (§5.6); the footprint under curvature from the mapping evaluated at neighbouring points |
| The beam integrated over a pixel | no `erf` | Abramowitz and Stegun 7.1.26, four multiplies |
| Per-subpixel sampling | plain arithmetic | each output channel evaluated at its own subpixel's position |
| Phosphor state across frames | **not in `FilterChain` today** | §4.2, extension 3 |
| Linear light without banding | **not in `FilterChain` today** | §4.2, extension 1 |

The two "not today" rows are limits of `FilterChain`, not of SkSL or Skia: Skia draws into half-float surfaces
and a surface kept from the last frame is ordinary.

**What would change the choice.** HDR. Neither path can present more than 8-bit sRGB today: the built-ins draw
into Avalonia's canvas, and the slang runtime's output is read back into the same canvas. If a presentation path
with an HDR surface is ever built, the mask's brightness problem (§5.8) is the first thing it should be used
for, and it would be worth re-asking this question then. Compute shaders, the other thing Vulkan has and SkSL
does not, buy this model nothing.

**No tier is split across the two paths.** The alternative considered was the Accurate tier as a slang preset
and the others built in. It was rejected because the tiers must be reductions of one model to be comparable
(step 3 measures each tier's difference from Accurate), and two implementations in two shading languages would
differ for reasons that are not the reduction.

### 4.2 What `FilterChain` must learn first

*Built 2026-10-07; `EmuSen_Serenity.md` §3.9 is the record, with its tests, its mutants and the evidence that no existing filter draws differently. Two things came out otherwise than written below: there is no fall back to 8 bits (a refused float surface is made in memory instead), and a rebuilt filter is compiled on the render thread at its next draw, its cost to be measured with the first real filter.*

Six extensions, all to `EmuSen.Serenity/Shaders/`, none to SkSL. Each is small; they are listed because step 2
begins with them and their tests.

1. **Half-float passes.** A `FilterPass` may ask for an `RgbaF16` surface. Today every intermediate is
   `Rgba8888` (`FilterChain.Surface`), which bands in linear light below about 3% and cannot hold a composite
   signal's negative excursions or a light value above white. Falls back to 8-bit, with the pass told, where a
   context refuses the format.
2. **Pass sizes other than the frame's and the viewport's.** `PassScale` is `Source` or `Viewport`. The signal
   passes are a fixed number of samples wide and the frame's height tall; the glass pass is a fraction of the
   frame. A pass states each axis as the source's, the viewport's or a fixed count, times a factor.
3. **Feedback.** A pass may read its own output from the previous game frame, as `feedback`. Like history
   (`EmuSen_Serenity.md` §3.2) it advances with the game's frames and not with redraws, and reads as black on
   the first frame.
4. **Earlier passes by name.** A pass reads only the one before it (`source`) and the frame (`original`). The
   beam pass must read the decoded picture and the phosphor state and the glass blur at once, so a pass may
   name any earlier pass's output.
5. **A filter built from its settings, with defaults per console.** SkSL has no preprocessor, and a signal type
   or a tier changes which passes exist, not just a number. A filter may therefore be a function from its
   structural settings to a pass list, rebuilt when one changes (the builders are cached by setting, and a
   rebuild is off the render thread as a slang preset's is). The same record carries **defaults per console**
   for its parameters, since the subcarrier's phase per line is a property of the console (§3.2) and not
   something a player should be asked. Related: `SlangParameter` has no way to say that a value is one of a few
   named choices, which the settings window needs for "Signal" and "Screen" (step 4).
6. **The picture's shape.** `GameFrameControl` letterboxes to the frame's own pixel count, so a 256×224 picture
   is shown 8:7. A CRT filter draws a 4:3 tube and places the active picture inside it by the console's line
   timing, so a filter may state the aspect it wants the rectangle to have. Every other filter, and no filter,
   keep today's behaviour.

Extension 6 changes what the picture looks like, not only how it is filtered, and was §9's question 2.

---

## 5. The model

Stage by stage, as the Accurate tier will compute it. Each stage names its passes; §6 says what the other tiers
leave out.

### 5.1 The signal, sampled in time (two or three passes, half-float)

The frame is resampled to **samples of line time**: 2048 samples across the active picture, the frame's height
tall. For the three consoles whose active picture is 170⅔ subcarrier cycles (§3.2) that is exactly 12 samples
per cycle, the NES's own colour-generator rate, and 8 samples per pixel at 256 wide, 6.4 at 320, 4 at 512; for
the N64 it is 10.9 per cycle. Nothing below depends on the ratio being a whole number: the subcarrier's phase at
a sample is computed, `φ = 2π (x · cyclesAcross / 2048 + line · perLine + frame-term)`.

**Encode.** Each sample is the console's output through its encoder's filter. A pixel is a box in time, and a
Gaussian low-pass of a box has a closed form (a difference of two error functions), so the encoder's band limit
is applied exactly to the staircase rather than to point samples of it; this is what removes the aliasing a
4×-oversampled point-sampled encoder has. Luma and the two colour-difference signals are filtered at their own
bandwidths (§3.1, §3.2), ~~set-up is added for a US signal~~ *(no console adds one: §11.3)*, and the output is `Y` and `C = U sin φ + V cos φ`. RGB
skips the modulation and keeps three band-limited channels. Every filter is given as a −3 dB frequency in MHz,
converted to samples through the console's active-line time.

**Decode.** Composite is `Y + C` and is separated again as the chosen receiver did: a notch at the subcarrier
with a 2.3 MHz luma channel, or a two-line comb (the line above, read from the same surface, has the opposite
or shifted phase the console gives it, which is why a comb behaves differently on a Genesis than on an N64 and
needs no special case to do so). Chroma is band-passed, multiplied by the decoder's two axes at their angles
and gains (§3.1), and low-passed at about 0.5 MHz. S-Video skips the separation and keeps the narrow chroma.
The result is the three gun voltages.

**What is approximate.** The NES is encoded from its RGB picture, not from its PPU's square wave, until
question 5 is settled. Encoder filters marked unfound in §3.2 use the standard's limits. RF is not modelled:
the survey's one model of it (Scanline Classic's) shows what it takes, and it adds noise and ghosting rather
than a different picture.

### 5.2 Transfer

BT.1886's form per gun, `L = a · max(V + b, 0)^γ`, γ 2.4, with the player's brightness and contrast as `b` and
`a`. From here on values are linear light relative to the tube's white.

### 5.3 Phosphor state (one pass, half-float, feedback)

§3.5's measurement as a recurrence. A decay written as a sum of exponentials, each with a per-frame ratio `r`,
gives the light in frame `n` as `Σ aᵢ (1 − rᵢ) Sᵢ` with `Sᵢ ← Eₙ + rᵢ Sᵢ`. Red needs no state. Green's and
blue's `1/k` tails are each fitted by two exponentials over the first thirty frames, which is four numbers a
pixel: one RGBA surface. The fit's error against Kuhn's integrals is to be tabulated in step 2 before it is
used.

It runs at the signal's resolution, before the beam. Strictly the state belongs to the phosphor, after the
beam; the difference is that a trail keeps the beam width of the frame that made it, a second-order effect on a
5% term, and the cost of keeping state at the screen's resolution is a full-screen half-float surface.

### 5.4 The beam (in the final pass)

For an output position that maps to raster position `(u, v)`, with `v` in scanlines, each nearby scanline `j`
contributes its light at `u` times its vertical profile. The profile is a Gaussian whose **area is the line's
light**, so a wide bright line is lower-peaked than a narrow one of the same energy, and whose σ grows with the
drive, `σ = σ₀ (1 + k · L^p)` (§3.6). The Accurate tier integrates the profile across the output pixel's height
(a difference of two error functions) rather than sampling it at the pixel's centre, which is what keeps
scanlines even when the scale is not a whole number, the commonest complaint about scanline shaders at 1080p.
Five lines are summed, since a bright spot's tail reaches further than a dim one's.

Across the line the light is already band-limited by §5.1; the spot's own horizontal width is a short Gaussian
over the signal samples, which at 2048 samples per line are finer than any output pixel.

**Per gun.** Each of the three guns is evaluated separately, at its own offset (§5.7) and with its own width.

### 5.5 The phosphors' colours (in the final pass)

Linear light per gun, times the 3×3 matrix from the chosen phosphor set and white (§3.4) to XYZ and from XYZ to
the display's primaries (sRGB by default, Display P3 as a setting). The matrix is computed in C# from the
chromaticities and handed to the shader as uniforms, so a new phosphor set is four pairs of numbers. Colours
outside the display's gamut are scaled toward their own luminance until they fit, not clipped per channel,
which would shift hue. A 9300 K white is shown as it was, bluer than the display's own white, with adaptation
to the display's white as a setting; which is "accurate" depends on whether the room is being simulated too,
and the default is §9's question 6.

### 5.6 The mask (in the final pass)

**Pitch is physical.** A screen class (§3.8) gives triads across the picture; the picture's width in output
pixels gives pixels per triad. The same setting is the same television at any window size.

**The pattern is geometry.** An aperture grille is three stripes per triad with a dark guard between them. A
slot mask is the same, broken by bridges at a vertical pitch, alternate columns offset by half of it. A shadow
mask is dots on a hexagonal lattice. Each is described by the fraction of the phosphor layer each colour
occupies, the lit fraction, and the measured stripe and guard widths are to come from macro photographs (§8.3),
marked as fitted.

*(As built, §11.7: the footprint is the whole pixel centred on the subpixel, not the subpixel's own third; the paragraph below is the first design.)*

**Each display subpixel shows only its own phosphor.** A display's red subpixel cannot emit green. So the red
output of a pixel is the red gun's light times the share of the *red subpixel's own footprint* that red phosphor
covers, and likewise green and blue, each at its own position a third of a pixel apart. This is three times the
horizontal sampling of the mask for no extra texture reads, and it is what makes a consumer mask drawable at
1080p (§3.8). The display's layout (RGB, BGR, or none for panels that are neither) is a setting; "none" samples
whole pixels.

**Aliasing is integrated away, in closed form.** The share of a footprint a periodic stripe covers is a
difference of two values of a piecewise-linear function, exact for a box footprint; the Accurate tier uses a
tent footprint two subpixels wide (the same function integrated once more), which suppresses the beat between
the mask's pitch and the pixel grid that a box leaves. As the pitch falls below what the display can show, the
integral tends smoothly to the mask's mean: the mask fades out by itself, with no threshold and no false colour.
The handheld LCDs' stripe grid met the same beat (`EmuSen_Serenity.md` §3.5: at four screen pixels per game
pixel white read green until the edges were softened), and a contact sheet of stripes shrunk for viewing shows
tints that the full-size render does not have; §8.1's neutrality test measures it.

**The mask is fixed to the glass, the raster is not.** The mask is evaluated in faceplate coordinates and the
beam in raster coordinates, so curvature and overscan move the picture across the mask, as they did.

### 5.7 The glass (one small pass pair, and terms in the final pass)

- **Veiling glare and halation**: the linear-light picture, reduced to the frame's own size, blurred separably
  by a kernel that is a wide Gaussian plus the ring of §3.7 *(as built, one Gaussian and no ring: §11.6)*, and added to the direct light with the direct
  light reduced by the same share, so the total is conserved. The two weights are fitted so that a rendered
  4×4 and 9×9 checkerboard give §3.7's 0.46% and 1.3%.
- **Convergence**: each gun's raster position offset by a static error plus a term growing toward the corners,
  in millimetres on the faceplate (§3.7), converted through the screen class's size.
- **Geometry**: the output pixel is mapped to a point on a sphere (shadow and slot mask tubes) or a cylinder
  (aperture grille) of the class's radius, seen from a viewing distance, then to the raster through the
  overscan. The mapping is a closed form evaluated at the pixel and at its neighbours, which gives the
  footprint that derivatives would have given.
- **Overscan** crops the raster at the tube's edge, with the corners rounded.

### 5.8 The player's display, and what an SDR screen cannot do

**Brightness is the hard limit.** The model conserves energy: a stripe that covers a third of a triad carries
three times the mean luminance, and a Gaussian scanline's peak is `1/(σ√2π)` of its mean, 1.33 at σ = 0.3. A
full mask under a narrow beam therefore needs about four times the tube's mean white at the brightest
subpixels. For a tube at 100 cd/m² that is 400 cd/m², which a bright SDR desktop display reaches and a 250
cd/m² laptop does not.

The filter works in absolute luminance. Two settings, the tube's white and the display's peak, both in cd/m²,
decide the ratio available. When the display has the headroom, the mask and scanlines are drawn at full depth
and the picture's mean brightness is the tube's. When it does not, **the mask's depth is reduced before the
picture is dimmed or clipped**: the mask is blended toward its own mean by exactly the amount that brings the
brightest subpixel to the display's peak, and the blend is computed per pixel from the local light, so dark
areas, where there is headroom to spare, keep the full mask. This is the approximation an SDR screen forces,
and it is stated in the setting's description. An HDR path (§4.1) would remove it.

**Quantisation.** The final pass is 8-bit sRGB; an ordered dither of half a code value is added
before encoding, because linear-light scanline tails otherwise band in the darks.

**Frame rate.** The model produces one picture per game frame. On a display at a multiple of the game's rate
the same picture is shown again (`EmuSen_Serenity.md` §2.6).

### 5.9 What is left out, and why

- **The flash.** §3.5: a CRT emits most of a frame's light in a millisecond. Reproducing it needs sub-frames on
  a 120 Hz or faster display, a rolling scan with the phosphor's fade behind it; the Blur Busters simulator
  (MIT) is the published method, and it needs the frontend to present several sub-frames per game frame and a
  real frame history. That is frame pacing work in Mistress, not a shader, and is proposed as its own later
  stage (§9, question 10).
- **RF**, and the receiver's impairments (noise, ghosts, hum bars).
- **Raster bloom**, the picture growing as the high-voltage supply sags under a bright frame. It needs the
  frame's mean brightness, which SkSL cannot read from a mipmap; a 1×1 feedback pass could carry it. Not in
  steps 2 to 4.
- **The room**: reflections on the glass, a bezel, ambient light.
- **PAL**, until a PAL game is in front of it: the constants are in §3.1, and the V-switch and delay line fit
  the decode pass, but no core here is being run at 50 Hz to test them against.

---

## 6. The tiers

One model, three reductions. The Accurate tier is built first and is the reference the other two are measured
against (step 3): for each, the difference from Accurate on rendered frames, in linear light, and its cost.

| | **Performance** | **Balanced** | **Accurate** |
|---|---|---|---|
| For | a weak laptop's integrated graphics and the handheld, at 60 fps | a desktop or the handheld with room to spare | everything, regardless of cost |
| Signal | RGB, or composite and S-Video as band limits only: luma and chroma blurred at their bandwidths in one pass at the frame's size, no subcarrier, so dithering blends but nothing rainbows or crawls | the real encode and decode at 4 samples per subcarrier cycle (683 samples a line), notch decoder | 12 samples per cycle (2048 a line), box-integrated encode, notch or two-line comb, the decoder's own axes |
| Transfer | BT.1886 form | the same | the same |
| Beam | two lines, one bilinear read each, Gaussian sampled at the pixel centre, width growing with drive, one width for all three guns | three lines, sampled, one read per line | five lines, integrated over the pixel, each gun separately |
| Phosphor colours | matrix | matrix | matrix with gamut scaling |
| Persistence | none | §5.3's state | §5.3's state |
| Mask | closed-form box, per subpixel | closed-form box, per subpixel | closed-form tent, per subpixel, footprint from the geometry |
| Glass | none | glare from one small blur | glare and ring, fitted to the checkerboards; convergence per gun |
| Geometry | flat, overscan only | curvature | curvature, with the footprint carried into the mask |
| Display | luminance-aware mask depth | the same | the same, with dither |
| Passes | 2 | 5 | 6 *(as built: 10 by composite, 8 by RGB, §11.1)* |
| Surfaces | 8-bit | half-float | half-float |

The per-subpixel, closed-form mask is in every tier: it is arithmetic, not texture reads, and it is the part
that is wrong in the cheap shaders surveyed.

### 6.1 Predicted cost, stated before anything is built

**The anchor.** The built-in Lottes is the one measured SkSL CRT filter (`EmuSen_Serenity.md` §8.3, §8.4): GPU
time 0.75 ms for a 1234×1080 picture and 1.87 ms for 2469×2160 on the RX 6800 at its paced clock, and 3.59 ms
for 1371×1200 on the handheld on battery. Per output pixel it reads 42 texels and evaluates about 126 `pow` for
its sRGB decode and 42 Gaussian weights. Call that unit **L**. A 4:3 picture is 1440×1080 at 1080p (1.17 of the
measured picture), 2880×2160 at 4K, and 1067×800 at the handheld's 1280×800 (0.52 of its measured picture), so
one L is **0.88 ms at 1080p and 2.2 ms at 4K on the RX 6800, and 1.9 ms on the handheld at 1280×800**.

**The estimate** counts each tier's texel reads and transcendental functions against Lottes's. Passes at the
signal's size do not scale with the window; they are converted at 1080p.

| | Final pass | Fixed passes | Total at 1080p |
|---|---|---|---|
| Performance | 2 bilinear reads, 2 `exp`, 3 `pow`: 0.12 L | a frame-sized pass: nil | **0.12 to 0.15 L** |
| Balanced | 3 reads, 3 `exp`, geometry, glare read: 0.25 L | 683×240 × 38 taps, 6 M reads: 0.1 L | **0.35 to 0.5 L** |
| Accurate | 45 half-float reads, 30 `erf`, tent mask, 5 mapping evaluations: 1.0 to 1.5 L | 2048×240 × about 120 taps, 59 M reads: 1 L | **2 to 2.5 L** |

| Prediction | RX 6800, 1080p | RX 6800, 4K | Handheld, 1280×800 | Weak laptop, 1080p |
|---|---|---|---|---|
| **C1** Performance | 0.10 to 0.13 ms | 0.25 to 0.35 ms | 0.2 to 0.3 ms | 1.2 to 2.6 ms |
| **C2** Balanced | 0.3 to 0.45 ms | 0.6 to 0.95 ms | 0.8 to 1.1 ms | 3.5 to 8.5 ms |
| **C3** Accurate | 1.8 to 2.2 ms | 3.2 to 4.2 ms | 5 to 6 ms | 20 to 40 ms |

Further predictions, to be given verdicts in steps 2 and 3 *(C3, C4, C5 and C7 have theirs in §11.10)*:

- **C4.** Accurate's fixed passes are 40 to 50% of its frame at 1080p and under 30% at 4K.
- **C5.** The render thread's own time stays under 0.3 ms for every tier, as §8.3's built-ins do; the cost is
  the GPU's.
- **C6.** On the laptop the unfiltered Lottes would itself take 10 to 17 ms, so it does not hold 60 fps there
  and Performance is the only CRT filter that does.
- **C7.** Half-float surfaces cost under 10% over 8-bit ones on the RX 6800.

**How weak the estimate is.** The laptop column has no measurement under it at all: it scales the handheld's
figure by the ratio of the two GPUs' nominal throughput (an Intel UHD of 24 execution units against the
handheld's integrated Radeon at its 800 MHz floor), taken as 3 to 5. The handheld's own figure moves with its
clock state (§8.4 there). Counting reads ignores that half-float bilinear reads cost more than 8-bit nearest
ones, and that Skia's SkSL compiler may fold less than a hand-written shader. The predictions are written down
so that they can be wrong in a recorded way.

---

## 7. The player's settings

One filter, **CRT**, in each console's list beside the existing ones, with these settings (`EmuSen_Serenity.md`
§3.7's parameters, plus named choices):

| Setting | Choices | Default | Notes |
|---|---|---|---|
| **Quality** | Performance, Balanced, Accurate | Balanced | §6 |
| **Signal** | RGB, S-Video, Composite | per console, below | what cable the console is plugged in with |
| **Screen** | Consumer TV, 14-inch; Consumer TV, 20-inch; Consumer Trinitron, 27-inch; Professional monitor, 14-inch; Professional monitor, 20-inch; PC monitor | Consumer TV, 20-inch | each carries mask type, pitch, size, spot, decoder, phosphors, curvature, convergence and overscan from §3 |
| **Curvature** | Off, the screen's own, or an amount | the screen's own | Off keeps the mask and scanlines |
| **Overscan** | Whole picture, Television | Whole picture | question 2 |
| **Colour** | North America, Japan, Europe | North America | phosphors, white, set-up and the decoder's axes together |
| **Display brightness** | cd/m², 100 to 1000 | 300 | §5.8; the player's own screen |
| **Display subpixels** | RGB, BGR, None | RGB | §5.6 |

Under an **Advanced** heading, each constant the screen class sets, as a slider with the class's value as its
default: mask type and pitch (as triads across), mask depth, spot size and growth, scanline visibility, glare,
persistence, convergence, comb or notch, chroma bandwidth, tube white, gamma, brightness and contrast. A
setting whose value is a measurement says so in its description and one that is a choice says that; this is the
distinction `EmuSen_Serenity.md` §3.7 drew for the LCDs.

**Per console.**

| Console | Signal | Phase per line, per frame | Why |
|---|---|---|---|
| NES | Composite | ⅓; ⅓ and ⅔ alternating | it has no other output |
| SNES | Composite | ⅓; ⅓ and ⅔ alternating | the cable it shipped with; pseudo-hi-res needs the blend. S-Video and RGB are real options for it |
| Genesis | Composite | 0; 0 | dithering was drawn for it; RGB is a real option |
| N64 | Composite | ½; ½ | the cable it shipped with; S-Video is a real option |
| Game Boy, Game Boy Color | not offered | | they have their own LCD filters |

The phase columns are not settings. They are the console's, carried by extension 5 of §4.2.

**The existing filters stay.** **CRT (Lottes)** is a named shader players may know, **Simple CRT** and
**Scanlines** are nearly free. Whether **CRT** becomes any console's default is question 4.

---

## 8. Validation

The rule from the handheld filters stands: render real frames headless through `EmuSen.WiseMan` and read pixels
before committing. Each built tier gets cases in `ScreenFilterRenderTests`; the list below is what they will
assert, and each has a mutant that must turn it red.

### 8.1 Against the model's own physics

- **Energy.** A flat white field's mean linear light, over whole triads and whole scanlines, equals the tube's
  white over the display's peak, within 2%, at four window sizes including non-integer scales; the same for a
  flat 10% grey. Mutants: an unnormalised beam; a mask without its lit-fraction gain.
- **Neutrality.** The same field's mean chromaticity is the white point's, R, G and B means within 1%, at every
  size from 1067 to 2880 pixels wide, read from the full-size render and never from a reduced copy. Mutant:
  point sampling the mask.
- **The scanline's width.** The full width at half maximum of a rendered line, read from a magnified render,
  matches `σ(L)` at three drive levels, and a bright line is wider than a dim one by the model's ratio. Mutant:
  constant σ.
- **The mask's pitch.** The autocorrelation of a rendered row peaks at the class's pixels per triad, and the
  subpixel order is red, green, blue. Mutant: pitch in output pixels.
- **The mask fading.** At a pitch below one pixel per triad the rendered row's variance is under 1% of the full
  mask's. Mutant: the box footprint replaced by a point.
- **Luminance limit.** With the display's peak set below what the mask needs, no output value clips and the
  mean is unchanged; with it set above, the mask is at full depth.

### 8.2 Against the signal's arithmetic

- **Colour bars** through composite return each bar's colour within 3 code values at the bar's centre with the
  ideal decoder, and the documented shift with the US axes.
- **Chroma bandwidth.** A colour step's 10 to 90% rise time is `0.35 / bandwidth`, within 10%.
- **Phase.** Under NES and SNES settings a vertical edge's artifact repeats every three lines and alternates
  between two frames; under Genesis settings two frames are identical and every line is the same; under N64
  settings alternate lines and alternate frames are inverted. Mutants: each console given another's phases.
- **The Genesis's dither.** Alternating columns at 320 wide decode to their mean luma with a colour band that
  repeats every 30 pixels, §3.2's derivation. At 256 wide the same columns show less chroma, by the ratio of
  the chroma filter's response at 0.89 MHz to its response at 0.22 MHz. Both figures are computed from the
  filters before the render.
- **Pseudo-hi-res.** Alternating columns at 512 wide blend to their mean within 2% through composite, and stay
  apart through RGB on the professional monitor.

### 8.3 Against published measurements and photographs

- **Checkerboard contrast**: a rendered 4×4 and 9×9 checkerboard give black squares at 0.46% and 1.3% of white
  with the professional-monitor glass (§3.7). This is the only number in the glass model with a measurement
  under it.
- **Persistence**: a white square shown for one frame and then removed leaves 5.4% of its green and 4.7% of its
  blue in the next frame, 3.0% and 2.6% in the one after, and no red (§3.5).
- **Mask geometry**: stripe and guard widths, slot length and bridge height, read from macro photographs of
  named tubes. No photograph has been measured yet; finding ones with a known scale (a pitch from §3.8 in
  frame) is step 2's first task, and until then the mask's internal proportions are choices.
- **Scanline profile**: the same, for the spot's size relative to the line pitch at dim and bright drive.
- **Real frames**: *Sonic the Hedgehog*'s waterfall, a SNES pseudo-hi-res scene and an NES title, copied to
  scratch from the library, rendered at 1080p and 4K, the pixels read and the pictures kept out of the
  repository.

### 8.4 Against other shaders, as referees

The slang runtime can run any of §2's shaders on the same frame. Where the model and a shader with an
independent derivation agree (patchy-ntsc and Scanline Classic on a console's phase, crt-beans on bandwidth in
MHz), that is evidence; where they share an ancestor it is not. A referee, never a grader, as with the FPGA
cores. The unlicensed ones may be run and never read into the code.

---

## 9. Decisions

Each was put as a question on 2026-10-07 with a recommendation; the answers are recorded in place.

1. **The delivery path** (§4.1): the built-in SkSL chain for every tier, with §4.2's six extensions to
   `FilterChain`. **Decided 2026-10-07: approved.**
2. **The picture's shape** (§4.2, extension 6). **Decided 2026-10-07 by the tester: under the CRT filter the
   picture is a 4:3 tube with the console's own pixel aspect, and the whole active picture is shown by default.
   The overscan crop is a setting. With the filter off, nothing changes.**
3. **One entry or three** in the filter list. **Decided 2026-10-07: one CRT entry with a Quality setting.**
4. **Defaults.** **Decided 2026-10-07 by the tester: Composite is the default signal for the NES, SNES, Genesis
   and N64, with S-Video and RGB as choices. The default filter stays None for now, to be decided again once the
   tiers are built and measured.**
5. **The NES's signal.** An accurate NES composite needs the PPU's colour index and emphasis bits per pixel from
   Moon and MoonRT. **Decided 2026-10-07: built from RGB now; the raw PPU path is its own later step, and is
   owed.**
6. **Region.** **Decided 2026-10-07: North America by default (SMPTE C, D65, set-up); a 9300 K white is shown as
   it was only when Japan is chosen. The game's region is taken from the frontend where it already knows it, and
   no plumbing is added for it in this effort.**
7. **Persistence** (§3.5). **Decided 2026-10-07: at its measured strength by default, with a slider.**
8. **Interlace.** **Decided 2026-10-07: woven by default; fields is a setting.**
9. **Measurements on other machines.** C1 to C3's handheld and laptop columns have nothing under them.
   **Decided 2026-10-07: to be asked for when the tiers exist; the tester runs the handheld's.**
10. **Later stages**: the sub-frame flash (§5.9), an HDR presentation path (§4.1), PAL, RF, raster bloom.
    **Decided 2026-10-07: recorded, not built now.**
11. **The decoder's axes** (§3.1's table is a compilation's transcription of datasheets that could not be
    fetched). **Decided 2026-10-07: the ideal decoder only, until the datasheets are read.**

---

## 10. Sources

**Standards and datasheets.** SMPTE 170M-2004, *Composite Analog Video Signal — NTSC for Studio Applications*.
ITU-R BT.470-6, *Conventional Television Systems* (1998). ITU-R BT.1886, *Reference electro-optical transfer
function for flat panel displays used in HDTV studio production* (2011). EBU R95, *Safe areas for 16:9
television production*. Sony, CXA1145 datasheet. Sony, PVM-M series brochure (PVM-14M4, 14M2, 20M4, 20M2).

**Papers and books.** M. G. Kuhn, *Optical Time-Domain Eavesdropping Risks of CRT Displays*, IEEE Symposium on
Security and Privacy, 2002. T. Elze, *Misspecifications of Stimulus Presentation Durations in Experimental Psychology: A
Systematic Review of the Psychophysics Literature*, PLoS ONE 5(9) e12792, 2010. Y. Faroudja, *NTSC and Beyond*, IEEE Transactions on Consumer Electronics 34(1),
1988. K. Compton, *Image Performance in CRT Displays*, SPIE TT54, 2003, chapter 1. C. Poynton, *Gamma FAQ*.
M. Abramowitz and I. Stegun, *Handbook of Mathematical Functions*, 7.1.26. US patents 4,620,220 (RCA, 1986) and
6,750,601 (LG.Philips).

**Measurements and references on the web.** DisplayMate, *Display Technology Shoot-Out*, part 1 (Sony
PVM-20L5). nesdev wiki, *NTSC video* and *Cycle reference chart*; SNESdev wiki, *Timing*; nesdev forum threads
9235 and 24447; Kabuto's Mega Drive hardware notes (plutiedev mirror); n64brew wiki, *Video Interface* and
*Video DAC*. S. Goldwasser, *TV and Monitor CRT (Picture Tube) Information*, sci.electronics.repair FAQ.
crtdatabase.com (Sony KV-27S42, JVC AV-27D302). ChthonVII, gamutthingy, `src/constants.h`, for its cited
chromaticities and decoder tables. NAB, *Safe Action and Safe Title Areas*. Nichia's phosphor list, hosted by FH
Münster. Wikipedia, *Dot pitch*, *Shadow mask*, *Trinitron*, *NTSC-J*.

**Added with §11.** M. A. García-Pérez and E. Peli, *Luminance artifacts of cathode-ray tube displays for vision
research*, Spatial Vision 14(2), 2001, for the persistence figure and for Sherr's P22 times. J. Jimenez, *Next
Generation Post Processing in Call of Duty: Advanced Warfare*, SIGGRAPH 2014, for the dither. Photographs on
Wikimedia Commons, measured and not copied: Selçuk Oral, *Aperture grille closeup* and *Aperture grille closeup
teletext* (CC BY-SA 3.0); Planemad, *CRT pixel array* (CC BY-SA 2.5).

**Shaders read**, all from libretro's `slang-shaders`: §2.1's and §2.2's lists, each with its author and
licence.

---

## 11. The Accurate tier, as built (2026-10-07)

`EmuSen.Serenity/Shaders/CrtFilter.cs` is the filter, for the NES, SNES, Genesis and N64; `FilterChain`'s additions
for it are `EmuSen_Serenity.md` §3.9. Only the Accurate tier exists, there is no Quality setting yet, and no
frontend lists it yet (§11.11). This section is the record of what was
built, where it departs from §5 and §6, what each constant rests on, and what was measured. Three findings change
things said earlier, and are marked in place above as well:

- **Display subpixels do not triple what a display can show of a mask** (§11.7). §3.8's claim is retired.
- **The 5% next-frame persistence of §3.5 is an extrapolation that a sustained picture shows to be unphysical**
  (§11.4). The default is now tied to a figure from the vision literature, a fifth of it.
- **The signal's passes, not the screen's, are most of the cost at 1080p** (§11.10). Prediction C4 is refuted.

### 11.1 The passes, and where the picture sits

| Pass | Size | What it does | Section |
|---|---|---|---|
| `signal` | 2048 × rows, half-float | the console's encoder: band limits, and for composite and S-Video the subcarrier | §11.3 |
| `parts` | the same | the receiver's separation and demodulation: luma, and chroma on two axes | §11.3 |
| `gun` | the same | the receiver's video bandwidth, the matrix to three gun voltages, the tube's transfer | §11.3 |
| `state` | the same, feedback | green's and blue's slow light, two sums each | §11.4 |
| `beam` | the same | each sample's light scattered along the line by its own spot | §11.5 |
| `spot` | the same | the energy-weighted variance of the spots that put light at each sample | §11.5 |
| `small`, `wide`, `haze` | 128 × 96, half-float | the picture's light gathered over the tube's face and spread by the glass | §11.6 |
| the face | the shown rectangle, 8-bit, drawn straight into the canvas | geometry, convergence, five scanlines per gun integrated across the pixel, the mask, the glare, colour, the display's limits | §11.5 to §11.8 |

Ten passes by composite or S-Video, eight by RGB, where `signal` applies the transfer itself and is `gun`. §6
said six; the receiver became two passes so that its video bandwidth acts on the separated luma, the spot became
two so that each gun keeps its own width, and the glass became three.

**A filter that counts scanlines takes the frame's rows once.** The N64's cores hand over 240 rows to be shown
twice each (`EmuSen_Serenity.md` §2.7); a tube draws 240 scanlines. `ScreenFilter.RowsOnce` is the slang
runtime's rule of §10.6 there, for a built-in.

**Where the picture sits.** The filter draws a 4:3 tube (decision 2). Inside it is the standard raster: an
active line of 52.66 µs and 241.5 active lines to a field (§3.1; 483 active lines to a frame is SMPTE 170M's). A
console's picture occupies its own share of that: `activeUs / 52.66` across, and its rows over 241.5 down, a
picture of more than 300 rows being two fields. With **Overscan** at 0 the raster is zoomed until the picture
touches the glass on its nearer pair of edges, so the whole picture is shown and nothing else: a 256 × 224 SNES
picture fills the height and leaves 1.2% of the width blank on each side, which is its true shape on a tube
(pixels 8:7). With overscan above 0 the standard raster is zoomed by that percentage, as a set's was.

**Each console's constants** are `CrtFilter.ConsoleTiming`, from §3.2's table: the active picture's duration,
the subcarrier cycles across it, the fraction of a cycle left over per line and per frame, and the encoder's
luma and chroma limits (5.0 and 1.3 MHz for all four, §3.2's stated fallback). They are not parameters; the
chain hands them to the passes as it does a parameter's value (`EmuSen_Serenity.md` §3.9). A console the table
does not name gets a standard line, 227.5 cycles.

### 11.2 The screens

Each is a row of `CrtFilter.Screens`. **[P]**, **[S]**, **[D]**, **[C]** and **[F]** mark a value read, quoted,
derived, chosen, or fitted here to a photograph (§11.5, §11.7).

| Screen | Width, mm | Mask, triads across | Spot σ, lines, dim to bright | Separation | Chroma, MHz | Video, MHz | Glass | Convergence, centre and edge, mm |
|---|---|---|---|---|---|---|---|---|
| Consumer TV, 14-inch | 264 [C] | slot, 440 [S] | 0.158 to 0.316 [F] | notch [P] | 0.5 [P] | 4.2 [C] | sphere, R/w 1.9 [C] | 0.6, 1.0 [C] |
| Consumer TV, 20-inch | 386 [C] | slot, 515 [S] | 0.108 to 0.217 [F] | notch [P] | 0.5 [P] | 4.2 [C] | sphere, 1.9 [C] | 0.6, 1.0 [C] |
| Consumer Trinitron, 27-inch | 549 [C] | grille, 732 [S] | 0.076 to 0.153 [F] | comb [C] | 0.5 [P] | 4.2 [C] | cylinder, 2.5 [C] | 0.6, 1.0 [C] |
| Professional monitor, 14-inch | 266 [P] | grille, 1064 [P] | 0.126 to 0.252 [C] | comb [C] | 1.3 [P] | 10 [C] | cylinder, 2.5 [C] | 0.4, 0.5 [P] |
| Professional monitor, 20-inch | 386 [P] | grille, 1245 [P] | 0.087 to 0.173 [C] | comb [C] | 1.3 [P] | 10 [C] | cylinder, 2.5 [C] | 0.5, 0.7 [P] |
| PC monitor, 15-inch | 280 [C] | dots, 1167 [S] | 0.09 to 0.18 [C] | comb [C] | 1.3 [C] | 30 [C] | sphere, 3.3 [C] | 0.3, 0.4 [C] |

The pitches, widths and convergence figures are §3.7's and §3.8's. The consumer chroma channel is §3.1's RCA
figure and the professional one the standard's own 1.3 MHz. The video bandwidth is a choice with a reason: 4.2
MHz is the bandwidth the system transmits (§3.1), which a consumer set's luma channel was built to; 10 MHz is
what 800 television lines need. The radii follow §3.7 (a 35 to 40 inch radius on an early 25-inch tube is 1.5
to 1.9 picture widths; a flat-square one 3.3). The viewing distance is three picture widths **[C]**.

**The advanced settings are relative to the screen**, each a multiplier at 1: mask pitch, spot size, spot
growth, chroma bandwidth, glare, persistence, convergence, curvature. A player changes the screen and the
multipliers stay meaningful, where absolute sliders would each have to be reset. Mask and separation can be
forced; the rest (tube white, gamma, contrast, brightness, mask depth) are absolute.

### 11.3 The signal

**Encode.** As §5.1: each of the 2048 samples is the console's row through a Gaussian of the encoder's limit,
integrated over each pixel's box by the difference of two error functions (Abramowitz and Stegun 7.1.26), nine
pixels wide. For composite and S-Video the chroma is `U sin φ + V cos φ` on the standard's U and V. For RGB the
receiver's video bandwidth is added to the encoder's in quadrature here, two Gaussians in cascade being one.

**No set-up is added.** §5.1 said it would be for a North American signal. The consoles do not add one: the
NES's black is 0 IRE by measurement (§3.2), and the encoders' datasheets describe none. A North American set
expects 7.5 IRE and so shows a console's blacks slightly crushed until its brightness control is turned up,
which is what the Brightness setting is. Retired, not built.

**Separation, by a notch.** The receiver multiplies the signal by the subcarrier's two axes and low-passes the
products. **Both low-passes are Hann windows a whole number of subcarrier cycles long**, which is the one part
of this stage not in §5.1. The reason is exact cancellation: a Hann window's response is zero at every multiple
of the reciprocal of its length from the second on, so a window of `m ≥ 2` cycles has zeros at the subcarrier
and at twice it. The first removes flat luma from the chroma channel entirely; the second removes the
demodulation's own double-frequency product. A Gaussian of the same width does neither: one with −3 dB at 1.3
MHz leaves 7% of the luma's level as a ripple at the subcarrier.

- **Chroma** uses `round(0.72 · fsc / bandwidth)` cycles, a Hann window's −3 dB point being 0.72 over its
  length: five cycles for 0.5 MHz (−3 dB at 0.515 MHz), two for 1.3 MHz (1.29 MHz).
- **The notch** is the same demodulation through a two-cycle window, remodulated and subtracted from the signal.
  Its −3 dB half-width is 1.29 MHz, so it removes 2.3 to 4.9 MHz, which is Faroudja's "little useful information
  above 2.3 MHz" for a notch receiver (§3.1).

cathode-retro (MIT) uses a box one cycle long for the same cancellation; the Hann form here was derived
independently and has the lower sidelobes.

**Separation, by a two-line comb.** Luma keeps its detail: only the difference between the line and the one
above, band-passed, is taken as chroma and subtracted. **The comb is referred to its own burst.** On a
standard line the subcarrier inverts from line to line and the difference is the chroma. On a NES or SNES it
advances a third of a cycle, and the difference is the chroma multiplied by `(1 − e^(−iθ))/2`: 0.87 of its
size, turned by 30°. A real set's burst passes through the same comb and is turned with it, and its automatic
chroma control restores the size, so colours are right; the filter divides by the same complex factor.
**[D]**, and the test below confirms the bars come back. On a Genesis the phase does not move from line to
line, the factor is zero, and a comb has nothing to subtract: the screen's separation falls back to the notch
when the factor's squared size is under ¼ **[C]**, a choice accepted on 2026-10-07 (§11.12). What real comb sets did with a Genesis was not found.

**Video bandwidth and drive.** The separated luma passes a Gaussian of the screen's video bandwidth; chroma is
matrixed on the standard's axes (decision 11: the ideal decoder only); and the three voltages go through
BT.1886's form with white at 1 and black at 0.01/176, the PVM-20L5's measured ratio (§3.3).

**What the measurements say** (§11.9's tests, on the RX 6800):

- Colour bars come back from S-Video, from composite by notch and from composite by comb within 0.004 of their
  RGB value in linear light, on SNES, Genesis and N64 timings.
- A chroma step rises in 0.70 µs through the 0.5 MHz channel; `0.35 / bandwidth` is 0.70.
- The NES's and SNES's artifacts repeat every three lines and every two frames, the Genesis's on every line and
  frame, the N64's every two lines and two frames.
- **The Genesis's alternate columns at 320 wide blend to their mean luma and turn into a rainbow 30 source
  pixels long**, as §3.2 derived. At 256 wide the rainbow is 6 pixels long and a third as strong (the chroma
  window's response at 0.895 MHz against 0.224 MHz is 0.34). At full contrast, white and black columns, the
  rainbow is fully saturated: this is the physics of a pattern 0.22 MHz from the subcarrier, and games dithered
  between nearer colours.
- **The SNES's 512-wide alternate columns do not vanish on a consumer set.** Their fundamental is 5.37 MHz,
  above the notch. The encoder's 5 MHz limit leaves 0.67 of it, the receiver's 4.2 MHz leaves 0.57 of that, and
  the spot 0.33 to 0.76 of that again, more removed the brighter the picture. What is left is a swing of under
  35% of the mean between columns, against more than three times that by RGB on the professional monitor. Two
  of those three numbers are choices or fallbacks (§11.2, §3.2), and this is the picture they are most visible
  in: the SNES's own luma bandwidth is the first constant a measurement should replace.

### 11.4 Persistence: the extrapolation retired

**What §3.5 predicted.** Kuhn's fitted impulse responses, integrated over frame intervals, put 5.4% of a
frame's green light and 4.7% of its blue into the next frame, 3.0% and 2.6% into the one after, falling as
roughly `1/k`.

**What the model showed when built.** A `1/k` tail is not summable in any way a tube shows. Two exponentials
per channel were fitted to the first thirty frames of those integrals (per-frame ratios 0.590 and 0.945 for
both channels; weights 0.0679 and 0.0094 for green, 0.0601 and 0.0081 for blue; worst relative error 8.6%).
Their sum over all later frames is 26% of the first frame's light for green. So a white picture held for two
seconds and then removed left **20% of its green in the first dark frame**, 16% in the second and 7.6% in the
tenth, in linear light: half of full scale in display code values. No CRT does that.

**Why.** The integral was taken correctly; the form was used outside its evidence. Kuhn recorded 2 ms of a
line's decay, and says his measurement was not designed to fix the power law's exponent. An exponent of 1.1,
continued from 2 ms to seconds, carries nearly as much energy in each decade of time as in the one before. The
frame-to-frame ratios quoted in §3.5 are all at 16 ms and beyond, eight or more times past the last recorded
point.

**What is used instead.** García-Pérez and Peli (*Luminance artifacts of cathode-ray tube displays for vision
research*, Spatial Vision 14(2), 2001 **[P]**) describe "phosphors that leave a luminance residue as large as
4% after 20 ms", citing Wolf and Deubel's measurement of a P31 (1997, their figure 4), and report from their
own P22 monitor that the artifact of persistence "is quite large on a P22-phosphor display". They also give
Sherr's P22 times to 10%: 1.5, 6 and 4.8 ms for red, green and blue, longer than Kuhn's monitor. P31 is a
zinc sulphide like P22's green and blue. The filter keeps the fitted shape, which is the only shape there is a
measurement for, and scales its strength so that **a sustained white leaves 4% of its green in the first dark
frame** (`TailScale` 0.1606), 3.5% of its blue and none of its red. A single frame's flash then leaves 0.8% in
the next frame. The Persistence setting is a multiplier on that, to 6, where 6.2 would be §3.5's extrapolation.

**Confidence.** Low, and stated: the 4% is a figure quoted in a review for a different zinc-sulphide phosphor,
"as large as"; the shape is one monitor's, extrapolated. Decision 7 was for the measured strength by default,
and what was called measured in step 1 is this section's first paragraph. The default here is the smaller and
directly quoted figure, and it is §11.12's first question.

**As built.** §5.3's recurrence: `state` holds `S ← E + r·S` for the two rates of green and of blue, four
numbers in one half-float surface, and the light shown is `kept · (E + Σ c·(S − E))`, with `kept` chosen so
that a steady picture's light is unchanged. It runs at the signal's resolution, before the spot.

### 11.5 The beam

**Along the line**, each sample's light is scattered by its own spot (`beam`): a Gaussian whose width is the
spot's at that sample's drive, integrated over each destination sample, so a bright sample spreads further
than its dim neighbour and the line's total light is exactly kept. A second pass (`spot`) records, at each
sample, the energy-weighted mean of the variances of the spots that put light there. **Across lines**, the
face pass draws each of five lines as a Gaussian of that recorded variance, integrated over the output pixel's
height by the error function, with its area the line's light.

This is §5.4 with one approximation named: the light at a sample is a mixture of spots of different widths,
and it is drawn down the screen as one Gaussian of the mixture's variance. The alternative considered, the
width taken from the scattered light itself, pinches the halo of a bright sample where it spills onto a dark
one, since the halo's light is the bright spot's and is as wide as it.

**The width's law** is §3.6's quadrature form: `σ² = σmin² + (σmax² − σmin²)·L`, with `L` the gun's light, so
a spot grows as the square root of its current above a floor.

**The widths are fitted to one photograph.** No curve of spot size against current was found (§3.6). Selçuk
Oral's macro photograph of a white teletext letter on a Sony KV-25FX20D, a 25-inch Trinitron (Wikimedia
Commons, *Aperture grille closeup teletext*, CC BY-SA 3.0; not copied into the repository), shows six
scanlines down a stripe. Its lines are clipped: a third to more than half of each line reads 255. What it still
gives is two numbers per stripe, the share of the line pitch that is clipped (0.33 to 0.35 for blue) and the
trough between lines relative to the clip level (0.09 to 0.15). A Gaussian that satisfies both has σ of 0.19
to 0.20 of the line pitch and a peak 1.4 times the clip level. The tube shows about 288 lines over a height
scanned 7% larger than its 360 mm, 1.34 mm a line, so **σ at white is about 0.26 mm** **[F]**. The photograph's
ratio of line pitch to triad pitch, 1.98, puts the triad at 0.68 mm, consistent with the 0.75 mm of the 27-inch
tube in §3.8, which is the check that the scale is right. The dim end is half the bright end **[C]**, the
ratio of the unverified range in §3.6. Consumer screens take 0.13 to 0.26 mm over their own line pitch;
professional ones 0.8 of that and the PC monitor 0.6 **[C]**.

**Measured on the build**: a single lit line, magnified ten times, has σ of 0.117, 0.139 and 0.218 lines at
drives of 0.25, 0.5 and 1.0 against the law's 0.114, 0.137 and 0.217, and carries the line's light to within
3% at each.

### 11.6 The glass

**One Gaussian, solved from two measurements.** DisplayMate's PVM-20L5 has a contrast of 219:1 on a 4×4
checkerboard and 75:1 on a 9×9 (§3.7). Taking each as the mean of the white squares' centres over the mean of
the black ones', on a finite 4:3 face from which light scattered past the edge is lost, a share `w` of the
light spread by a Gaussian of width `σ` reproduces both when **σ is 6.68% of the picture's width and `w` is
2.99%** (`glare_fit.py` beside the bench; two unknowns from two numbers, so the fit cannot test the Gaussian's
shape, only use it). Step 1's first solution, on an unbounded checkerboard read at one square's centre, was
6.26% and 2.72%.

**The ring is not drawn.** §3.7 derived a halation ring 18 mm in radius on a 20-inch tube from the critical
angle. The fitted σ is 26 mm on the same tube: the measured scatter is the size the ring predicts, and one
Gaussian of that size is what two checkerboards can support. A separate ring would need a measurement that
resolves it, a point source on black.

**As built.** The picture's light is gathered into a 128 × 96 map of the face, blurred along each axis with
light past the edge lost, and added in the face pass as `w` of the light with the direct light reduced by `w`.
It is applied after the mask, since the glass is in front of it. **Measured on the build**: 209:1 and 76:1,
against 219 and 75. The black squares are read from a second render three times as bright, since at the
default headroom a black square is under four code values.

### 11.7 The mask

**Proportions, measured from photographs** (`maskphoto.py`; both from Wikimedia Commons and neither copied into
the repository). Widths are at half the stripe's height in linear light.

| Mask | Photograph | Measured | Used |
|---|---|---|---|
| Aperture grille | Selçuk Oral, *Aperture grille closeup*, Sony KV-25FX20D, CC BY-SA 3.0 | each stripe lit for 0.199 of the triad, the three evenly spaced, in the order red, green, blue | 0.199 |
| Slot mask | Planemad, *CRT pixel array*, a 21-inch television, CC BY-SA 2.5 | stripes lit for 0.18 to 0.20 of the triad, spaced 0.30, 0.30 and 0.40 of it; slots 0.81 of a triad apart, lit for 0.75 to 0.79 of that; the next triad's slots half a pitch down, the three colours of one triad level | 0.19; 0.30, 0.30, 0.40; 0.81; 0.76 |
| Dots | the same, its 17-inch monitor | each colour lit over 0.09 to 0.12 of the area | a square of the same area on the hexagonal lattice, 0.106 |

So a colour's phosphor covers a fifth of an aperture grille's face and a seventh of a slot mask's, not the
quarter first assumed. The dot mask's circles are drawn as squares of equal area, which keeps the closed form;
no console here was shown on one.

**The footprint is a whole pixel, and the claim about subpixels is retired.** §3.8 and §5.6 said that reading
the mask per display subpixel gives three times the samples of a triad. It does not, for coloured stripes. Each
channel is sampled once per pixel whatever its subpixel's position, so a stripe pattern of one colour needs two
pixels per triad to be resolved at all, as it would without subpixels. The first build integrated each channel
over its own third of a pixel, and a professional monitor's grille at 0.86 pixels per triad then drew a beat
with a ripple of 79% of the mean where it should have faded out: the test below found it. And at exactly one
pixel per triad a third-pixel footprint is wrong in the limit: slide the mask a third of a pixel and the red
subpixel sees only green phosphor and goes dark. A display's red subpixel is the only place red light can come
from for a whole pixel's width, so **its footprint is the pixel**: a tent two pixels wide, centred on the
subpixel. What the subpixel's position still gives is the right place to take each colour's sample, for the
mask and for the beam, which removes the colour fringe the display's own layout would add.

Which displays can draw which mask is then the tent's response at the triad's frequency: 0.91 of the stripes'
modulation at 6 pixels per triad, 0.65 at 2.8 (a consumer 20-inch set at 1080p), 0.41 at 2, 0.17 at 1.5, 0.03
at 1.16. So a consumer slot mask is drawn at 1080p, softened; a 27-inch Trinitron's at 1080p is faint; a
professional monitor's needs 4K and is soft there; and below 1.5 pixels per triad the mask is gone, smoothly.

**The integral** is closed-form as §5.6 said: the share of a tent that a periodic lit run covers is the second
difference of the run's twice-integrated indicator, a quadratic in the number of whole periods plus a piece.
It is evaluated from the nearest period's start so that single precision holds at a thousand triads across. A
slot or dot mask is the sum of two such products, the even triads' columns with their rows and the odd ones'
with theirs half a pitch down, each exact for a separable footprint. crt-beans (MIT) anti-aliases a grille in
closed form with a one-pixel Hann window; the idea of doing it in closed form is shared, the derivation here
is its own.

**Measured on the build**: the triads counted along a line are the screen's own number at three window sizes
and pitches, within 2%; the stripes run red, green, blue a third of a triad apart; a grille at 0.86 pixels per
triad has a ripple under 6% of the mean and no tint; and the slot mask drawn at three times its size has the
photographed pitch, lit share and stagger.

### 11.8 Colour, and the display's limits

**Colour.** The three guns' light is converted by one 3×3 matrix, from the phosphors' chromaticities and white
to the display's (§5.5), computed in `CrtFilter.GunsToDisplay` and written into the face pass. North America is
SMPTE C at D65; Europe is EBU at D65; Japan is ARIB TR-B9's phosphors at 9300 K + 27 MPCD (§3.4), with no
adaptation, so its white is drawn bluer than the display's (decision 6): red 0.60, green 0.86 and blue 1.00 of the
display's range, the whole scaled down until blue fits. A colour outside the display's gamut is moved toward its own
luminance until it is inside. The display is sRGB or Display P3.

**The matrix is applied before the mask**, to the light each gun would give averaged over a triad, and each
display channel is then patterned by its own phosphor's mask. Strictly the 2% of a red subpixel's light that
the matrix takes from the green gun should carry green's stripe pattern. A display with three kinds of
subpixel cannot show that, and this is the nearest it can.

**Headroom**, as §5.8, with the numbers the measured masks give. A colour that covers a seventh of the face
must be seven times the mean where it is lit, and a narrow beam's peak is up to 1.8 times its line's mean on
the default screen. The face pass works in the tube's white as 1 and the display's peak as `displayNits /
tubeNits`. Each channel's mask is drawn at full depth where the light, times one over the lit fraction, fits
under the peak, and is flattened toward its mean by exactly the shortfall where it does not. At the defaults,
a tube of 100 cd/m² on a display of 300, a slot mask is at full depth for light up to 43% of white and at a
third of its depth at white; scanlines are untouched. **Measured**: with no headroom at all, a 75% grey keeps
its light within 2%, nothing clips, and the mask is under a third as deep as with ten times the headroom.

*(Superseded as the default on 2026-10-07 by §12.3's bright picture; what follows is now the **The tube's own** setting.)* **The default picture is a third as bright as the display can go**, by this arithmetic and on purpose: it is a
100 cd/m² tube on a 300 cd/m² display. A player whose display is dimmer than 300 sets Display brightness to
what it is, and the picture brightens while the mask flattens. Whether that is the right default for a first
impression is §11.12's second question.

**Dither** is interleaved gradient noise (Jimenez, *Next Generation Post Processing in Call of Duty: Advanced
Warfare*, SIGGRAPH 2014), half a code value, on the display's own pixel grid.

### 11.9 Tests and mutants

`ScreenFilterRenderTests.Crt.cs` holds 23 cases, 26 results with a theory's four rows counted, beside the existing
11 and 14. Three always run, through the headless
raster path: every variant compiles for every console (signal × mask × separation, and each screen); the filter
names its four consoles, its 4:3 shape and its rows taken once; and a white field on a 32 × 12 frame keeps its light within
3% and its colour within 1%. The rest draw on a GL device through `ShaderBench.Picture`, on the RX 6800 unless
`EMUSEN_BENCH_GL_DEVICE` names another, and return without drawing where there is none, as the slang tests do.
**The raster path takes 12.8 seconds a frame at 640 × 480** for this filter against a tenth of a second on the
device, which is why the measurements are on the device; one case draws the same small picture both ways and
finds no value more than 2 apart.

| Case | What it asserts |
|---|---|
| a flat field's light and colour | white and a 50% grey are the tube's light over the display's peak within 2%, and neutral within 1.5%, at 1067, 1440, 1600 and 2880 wide on five screens |
| a scanline's width | §11.5's three widths within 5% and their light within 3%; the bright line more than 1.7 times the dim one |
| the mask's pitch | §11.7's count at three sizes; the stripes' order; a stripe lit for 0.199 of a triad within 15% |
| the slot mask's proportions | pitch within 6%, lit share within 10%, the next triad's slots half a pitch down |
| a mask too fine | ripple under 6%, five times less than at three times the pitch; no tint |
| headroom | §11.8's four statements |
| colour bars | §11.3's 0.004, three signals and two separations on three consoles |
| chroma's rise | 0.70 µs within 20%; the professional channel under 0.6 of it |
| each console's artifacts | the lines and frames after which they repeat, four consoles |
| the Genesis's columns | the rainbow's length within 4%; the columns' own swing under 10% and an eighth of RGB's; 320 wide 2.2 to 3.8 times 256 |
| pseudo-hi-res | §11.3's two swings |
| the checkerboards | §11.6's two contrasts within 8% |
| persistence | 4% of green within 15%, blue in proportion, no red, and a tenth frame still over a quarter of the first |
| convergence | red to blue by the screen's error at the centre within 15%, more than 1.25 times it near the edge, green between |
| curvature | corners dark when curved and lit when flat, the centre within 2%, each edge's middle still picture |
| Japan's white | blue over red as the matrix gives (1.67), within 3% |
| interlace | woven rows alike; a field's rows alternate; the next frame's are the others |
| overscan | the whole picture starts inside the glass; overscanned it reaches the edge |
| rows sent to be repeated | the same picture, value for value |
| device against software | no value more than 2 apart |

**Mutants.** 36 were made, 34 in `CrtFilter.cs` and two in `FilterChain`, each a physical statement turned false,
and each is caught by the case that names it:

- the beam: a peak that does not fall as the spot widens; a spot of constant width
- the mask: no gain for its lit fraction; sampled at a point; its pitch counted in output pixels; blue first;
  stripes half again as wide; slots not staggered, too long, too close; no rule for headroom
- the signal: the Genesis given the NES's phase; the NES given the standard's; no alternation between frames; a
  comb not referred to its burst; no notch; S-Video's chroma read from the wrong channel; the chroma channel
  always the wide one; no video bandwidth; the transfer a straight line
- the glass: no glare; glare twice as wide; no convergence error; the error the same across the screen; no
  curvature
- the phosphors: no tail; the tail at §3.5's extrapolated strength; a tail that halves each frame; a tail on
  red; colour not converted
- the picture: fields ignored; the same field every frame; overscan ignored; no stated shape; rows repeated;
  the console's constants dropped

**Two survived at first, and both for one reason**: the slot's lit share and the grille's stripe width were
asserted against the filter's own constants, so a mutant that changed the constant changed the expectation with
it. Those two cases now assert the photographs' numbers as written, 0.76 and 0.199, and the stripe width gained
an assertion it did not have. §11.7's footprint error was found the way a mutant is, but in the filter as first
written: the case for a mask too fine failed on it.

### 11.10 What it costs, and the predictions' verdicts

`EmuSen_Serenity.md` §8.1's bench and protocol: the RX 6800, frames paced at 60 Hz, 60 of warm-up and 300
measured, each case its own process, the cases interleaved in rotated order over three rounds under the bench
lock, a figure the median over rounds of each run's median. `gl.gpu` is the GL timer's time for the whole
frame; the render thread's own time is beside it. Cases and raw results: `~/.cache/emusen/probe/crt/`
(`cases-accurate.txt`, `results-accurate.txt`).

| Case | Drawn at | GPU, ms | Render thread, ms | The three rounds' GPU |
|---|---|---|---|---|
| SNES, no filter | 1234×1080 | 0.03 | 0.07 | 0.06, 0.03, 0.03 |
| SNES, built-in CRT (Lottes) | 1234×1080 | 0.67 | 0.08 | 0.67, 0.71, 0.42 |
| **SNES, CRT, composite** (the default) | 1440×1080 | **1.70** | 0.19 | 1.81, 1.70, 1.47 |
| SNES, CRT, S-Video | 1440×1080 | 1.71 | 0.19 | 1.71, 1.74, 1.41 |
| SNES, CRT, RGB | 1440×1080 | 1.38 | 0.17 | 1.38, 1.38, 1.14 |
| SNES, CRT, composite, professional 20-inch | 1440×1080 | 1.38 | 0.19 | 1.48, 1.38, 1.23 |
| SNES, CRT, composite, no mask, glass, tail, convergence or curve | 1440×1080 | 1.48 | 0.18 | 1.57, 1.48, 1.34 |
| Genesis 320×224, CRT, composite | 1440×1080 | 1.47 | 0.19 | 1.66, 1.47, 1.47 |
| N64 640×240, CRT, composite | 1440×1080 | 1.65 | 0.21 | 1.78, 1.65, 1.53 |
| SNES 512×448 (interlaced), CRT, composite | 1440×1080 | 2.35 | 0.23 | 2.59, 2.35, 2.35 |
| SNES, CRT, composite, a 320×240 window | 320×240 | 1.06 | 0.18 | 1.30, 1.06, 1.06 |
| SNES, CRT, composite, a 1280×800 window | 1067×800 | 1.25 | 0.19 | 1.57, 1.25, 1.25 |
| SNES, built-in CRT (Lottes), 4K | 2469×2160 | 1.61 | 0.09 | 1.92, 1.61, 1.61 |
| **SNES, CRT, composite, 4K** | 2880×2160 | **2.86** | 0.19 | 3.09, 2.86, 2.86 |
| SNES, CRT, RGB, 4K | 2880×2160 | 2.53 | 0.17 | 2.85, 2.53, 2.53 |

(Lottes keeps the frame's own shape, so its rectangle is narrower than the CRT's 4:3 one in the same window.)

**What the table says.**

- **The Accurate tier is 1.7 ms of GPU time at 1080p and 2.9 ms at 4K by composite**, 2.5 and 1.8 times the
  built-in Lottes in the same window. RGB saves the receiver's two passes, 0.33 ms.
- **The signal's passes are most of it at 1080p.** At a 320 × 240 window, where the face pass is nearly free,
  the frame is 1.06 ms: the passes at the signal's size cost that whatever the window, 62% of the 1080p frame
  and 37% of the 4K one. The face pass, with the mask, the glass, convergence and curvature all on, is about
  0.6 ms at 1080p; turning those off saves 0.2 ms.
- **The frame's own size matters more than the window's.** An interlaced 512 × 448 picture doubles the
  signal's rows and costs 2.35 ms. A professional screen's two-cycle chroma window is a shorter loop than a
  consumer's five and saves 0.33 ms.
- **The render thread spends 0.17 to 0.23 ms**, twice a simple filter's 0.08, on binding ten passes; nothing is
  allocated on the device per frame, and 10 KiB of managed memory is (lists and child shaders), with no
  collections in 300 frames.
- **The round-to-round spread is the GPU's clock**, as §8.4 there found on the handheld: Lottes read 0.67, 0.71
  and 0.42 ms in three rounds, the CRT 1.81, 1.70 and 1.47. A paced frame this light leaves the card in a low
  clock state that it leaves and re-enters; the medians are comparable, single runs are not.

**A build.** Changing a structural setting compiles the passes on the render thread (`EmuSen_Serenity.md`
§3.9). Measured on the RX 6800 at 1440 × 1080: SkSL's own compile of ten passes 22 ms the first time in a
process and 2 ms after; the first draw, where the driver compiles, 32 ms the first time in a process with the
driver's cache warm, 4 to 10 ms for a later variant, and 98 ms and 52 ms with the driver's cache disabled. So
the first use of the filter ever on a machine misses about six frames once, and a settings change after that
misses one. It was not moved off the thread.

| | Prediction | Verdict |
|---|---|---|
| C1 | Performance: 0.10 to 0.13 ms at 1080p | step 3 |
| C2 | Balanced: 0.3 to 0.45 ms at 1080p | step 3; the 683-sample signal's share is now expected to dominate it, by C4's verdict |
| C3 | Accurate: 1.8 to 2.2 ms at 1080p, 3.2 to 4.2 ms at 4K | **Refuted, low**: 1.70 and 2.86 ms. The total was over-predicted by 5 to 10% and its composition was wrong (C4). The handheld and laptop columns are untested |
| C4 | Accurate's fixed passes are 40 to 50% of its frame at 1080p, under 30% at 4K | **Refuted**: 62% and 37%. The estimate counted 59 M texel reads against the face pass's and took a read to cost what a face-pass read does; a tap in the receiver's loop also evaluates a sine, a cosine and two window weights |
| C5 | The render thread stays under 0.3 ms | **Held**: 0.17 to 0.23 ms |
| C6 | On the laptop Lottes takes 10 to 17 ms and only Performance holds 60 fps | untested; needs the laptop |
| C7 | Half-float surfaces cost under 10% over 8-bit ones | **Held**: 1.47 against 1.43 ms at 1080p and 2.87 against 2.83 ms at 4K, 3% and 1%, with every surface forced to 8 bits in a second build of the bench, interleaved over three rounds. That build's pictures are wrong; only its cost is compared |

### 11.11 What is not done

- **The Balanced and Performance tiers, and the Quality setting** (step 3).
- **A player cannot choose it yet** (step 4). `CrtFilter.Filter` is not among `ScreenFilters.All`, so no console's
  list shows it. It was put there and taken out again the same day, for three reasons a settings window has to
  answer first: Mistress does not yet tell the frame control which console is running, without which the filter
  draws a standard line's timing for every console; its seven settings that are choices would be drawn as
  sliders over numbers (`SlangParameter.Choices` carries their names and nothing reads it); and **the headless
  tests of the Shaders window draw whatever filter the pad lands on through Skia's raster code**, where this
  one takes seconds a frame (§11.9): with it listed, the seven rows of
  `The_shaders_window_from_the_pad_menu_adjusts_a_built_in_filter_and_resets_it_all` each ran 15 to 53 seconds
  and failed. Step 4 has to make those tests not draw a real CRT frame in software.
- **The NES's raw signal** (decision 5): owed. The NES is encoded from its RGB picture.
- **The decoder's axes** (decision 11): the ideal matrix only.
- **A game's region from the frontend** (decision 6): the Colour setting is the player's.
- **PAL, RF, raster bloom, the sub-frame flash, HDR** (decision 10).
- **Referees** (§8.4): no other shader was run on the same frame for comparison. The consoles' phases agree
  with patchy-ntsc's and Scanline Classic's tables by reading (§2.3), not by rendering.
- **Photographs of a slot-mask set's scanlines, and of any professional monitor**: the spot's widths for those
  screens are the Trinitron's, scaled.
- **Any measurement on the handheld, the laptop or a Mac.** The filter has been drawn on one GPU and one
  driver, and through Skia's raster code.

### 11.12 Questions this step raised, and their answers

1. **Persistence's default** (§11.4). The strength called measured in step 1 was an extrapolation that the
   built model shows to be unphysical. **Decided 2026-10-07: 4% of a held white in the first dark frame, the
   García-Pérez and Peli figure, is the default, with the slider.**
2. **The default brightness** (§11.8). A 100 cd/m² tube on a 300 cd/m² display is a picture a third as bright
   as the display's peak. **Decided 2026-10-07 by the tester: brighter by default, using most of the display's
   brightness, with the mask at full depth in dark and mid tones and easing off only in the brightest areas;
   the tube's true brightness stays available as a setting.** Built in §12.3, where the measurements show that
   an SDR display cannot give both halves of it at once.
3. **The default screen** is the 20-inch consumer slot-mask set, whose mask at 1080p is 2.8 pixels a triad:
   drawn, softened. On a 1280 × 800 handheld it is 2.1, faint. Not decided; it stays.
4. **Showing the filter in the Shaders window** (§11.11). **Decided 2026-10-07: in step 4, done properly:
   Mistress tells the frame control which console is running; the settings that are choices are drawn as
   choices; and the window's and the pad's tests never draw a real CRT frame through Skia's software path,
   using the device where they need a real picture and a stand-in filter where they test only the window, and
   saying which.**
5. **The comb on a Genesis** (§11.3) falls back to the notch. **Decided 2026-10-07: accepted, as a choice**; it
   is marked **[C]** where §11.3 states it.
6. **Moving the build off the render thread** (§11.10). **Decided 2026-10-07: moved off it, with the previous
   picture, filtered or plain, kept on screen until the new filter is ready, so that a settings change never
   drops a frame; in step 3 or 4, whichever touches that code first.** Step 3 did not touch it (§12.8).

---

## 12. Balanced and Performance, as built (2026-10-07)

Both lower tiers are reductions of §11's Accurate tier in the same file, chosen by the **Quality** setting
(`quality`, a structural parameter: Performance, Balanced, Accurate). **Balanced is the default.** Each was
judged by its distance from Accurate on rendered frames (§12.4) and its cost (§12.5). This section also builds
§11.12's second decision, the bright picture (§12.3).

### 12.1 What each tier keeps

| | Performance | Balanced | Accurate (§11) |
|---|---|---|---|
| Passes | 2 | 8 by composite, 6 by RGB | 10 by composite, 8 by RGB |
| Signal | the frame at twice its width, half-float: luma and chroma blurred at the encoder's and receiver's bandwidths, and by composite through a notch, but no subcarrier | the real encode and decode of §11.3 at 4 samples a subcarrier cycle (683 a line on the 21.48 MHz consoles, 753 on the N64) | 2048 samples a line |
| Transfer | in the signal pass | in the receiver's pass, as Accurate | §11.3 |
| Persistence | none | §11.4's tails | the same |
| Beam | the two nearest lines, one read each, a Gaussian sampled at the pixel's centre, one width for the three guns from the line's luma | three lines, one read per gun, sampled, each gun its own width | five lines per gun, integrated over the pixel, the spot scattered along the line first |
| Convergence | none | §11.2's, by reading each gun at its own place | the same |
| Mask | box footprint, per subpixel | box footprint, per subpixel | tent footprint |
| Glass | curvature only | curvature and glare | the same |
| Dither | none | yes | yes |

**What is the same in all three**: the screen's physical mask pitch and proportions, the colour matrix, the
picture's placement and overscan, interlace, and §12.3's headroom rules. The mask is in every tier because §11.10
showed it is cheap (0.05 ms of Performance's frame, measured with the mask turned off).

**Performance's notch** is the one part of a lower tier with its own derivation. Without a subcarrier there is
no chroma for a notch to separate, but a notch receiver still removes luma from 2.3 to 4.9 MHz, and that is
what blends the Genesis's dithered columns (§11.3). The pass subtracts the notch's band from the luma directly:
the same two-cycle Hann window times the subcarrier that §11.3 demodulates with, integrated over each source
pixel in closed form, scaled by the encoder's and the video stage's gain at the subcarrier, since in Accurate
the notch acts between them. Two things had to be found by measurement before the columns blended:

- **The Gaussians' gain was taken at −6 dB instead of −3 dB** in the first build (`ln 2` where `ln 2 / 2`
  belongs), which scaled the band to 0.42 of what it should take away and left the columns at 37% of their RGB
  swing. The −3 dB definition (an amplitude of 1/√2) is §11.3's; the formula now uses it, and they are at 8%.
- **One sample per source pixel cannot carry the pattern.** Alternate columns are at half the source's sampling
  rate, and a filter evaluated only at pixel centres caught their harmonics, so a correct notch still left
  alternate samples a third of the swing apart; bilinear reading of those samples then beat with the display's
  pixels (a 79-pixel moiré). Evaluated at twice the width, the same filter blends them. The pass costs no more
  than its 512 × 224 samples, which is nothing beside the face pass.

**Performance's composite has no rainbow and no crawl**, by design (§6): its colour artifacts are absent, its
blending is present. Balanced keeps both, at 4 samples a cycle.

### 12.2 Balanced's convergence, and what it is worth

The first Balanced read each line once for all three guns and left out convergence, as §6 planned. Its
distance from Accurate on *Super Mario World 2* was 3.2 ΔE; with the glare off it was 3.5, with convergence off
in both tiers 1.3. Convergence was most of the difference. Reading each gun at its own place (three reads a
line instead of one, the subpixel position and the convergence error together, as Accurate does) brought
Balanced to **1.0 ΔE for 0.04 ms** at 1080p flat out. It is in.

### 12.3 The bright picture, and the conflict in the request

**As built.** A new setting, **Picture brightness**: *Bright* (the default) or *The tube's own*. Bright puts the
tube's white at **three fifths of the display's peak** (`BrightWhite`, 0.6, as decided in §12.9; three quarters when first built), whatever the display; the
tube's own is §11.8's absolute arithmetic, the Display brightness and Tube white settings. Both then use the
same headroom rules, now two of them:

1. **A scanline brighter at its centre than the display can go is drawn flatter**, toward its line's even
   light, by just enough that its peak fits. The flattening is the same across the whole line, so its light is
   kept. This is new: §11.8 flattened only the mask, and with white at three quarters a narrow line's centre
   (1.84 times its mean at white on the default screen) clipped, losing a third of the picture's light (white
   measured 0.47 of the peak instead of 0.75). Two versions were wrong before this one: easing each pixel by its
   own excess left the gaps dark and the centres cut, and lost the same third.
2. **The mask then gives way** where a colour's lit stripes would still exceed the peak, as §11.8.

**What the request asked for, and what an SDR display allows.** The decision asked for both a picture that
uses most of the display's brightness and a mask at full depth through the mid tones. Measured on the
Accurate tier at 1440 × 1080 (`depth.py`: the green channel's swing along a line across the mask, over its
mean; full depth is 2.43):

| Grey (drive) | White at 0.75 of peak (as first built) | at 0.6 (the default, decided) | at 0.5 | the tube's own at 100 of 300 cd/m² |
|---|---|---|---|---|
| 0.25 | 2.43 (full) | 2.43 | 2.43 | 2.44 |
| 0.5 | 1.18 | 1.47 | 1.71 | 2.43 |
| 0.75 | 0.52 | 0.70 | 0.87 | 2.43 |
| 1.0 (white) | 0.14 | 0.29 | 0.43 | 2.27 |

A slot mask lights a seventh of the face, and a mid-grey line's centre is about three times its mean, so full
depth at a drive of 0.5 needs white at a quarter of the display's peak or less. No setting gives a bright
picture and a deep mid-tone mask together on an SDR display; HDR would (§4.1). The default was built as the request's first
half, three quarters, with the mask full in the darks and half deep at mid grey, and the table put to a decision:
**0.6 was decided (§12.9)**, its column above.

### 12.4 How far each tier is from Accurate

**Method** (`compare.py`). Each frame is drawn by all three tiers at 1440 × 1080 with the defaults (Composite,
the 20-inch consumer set, Bright), six frames each so persistence has settled. Both pictures are averaged in
linear light over 8 × 8 blocks (about two scanlines and three triads, so mask and line structure average out
and the picture remains), converted to CIELAB against the reference's brightest block, and compared by ΔE*ab
(1976). A difference near 2.3 is the usual estimate of one just-noticeable step. The frames are resume
screenshots from the library, copied to scratch (*Sonic the Hedgehog*, *Sonic 3*, *Super Mario World 2*,
*Super Mario All-Stars*, *Super Mario Bros. 3*, a 512-wide SNES scene, *Bomberman 64* at 480 lines) and colour
bars.

| Frame | Performance: mean ΔE, 95th percentile | Balanced: mean ΔE, 95th percentile |
|---|---|---|
| Sonic the Hedgehog (Genesis) | 7.3, 17.2 | 1.07, 2.9 |
| Sonic 3 (Genesis) | 3.5, 11.5 | 0.98, 2.5 |
| Super Mario World 2 (SNES) | 6.9, 22.5 | 0.97, 2.7 |
| Super Mario All-Stars (SNES) | 7.3, 22.8 | 0.93, 2.6 |
| Super Mario Bros. 3 (NES) | 7.8, 27.8 | 0.81, 2.5 |
| A 512-wide SNES scene | 5.1, 16.9 | 0.69, 2.4 |
| Bomberman 64, 480 lines | 4.2, 11.1 | 0.70, 3.0 |
| Colour bars | 4.1, 10.0 | 0.78, 1.9 |

**Balanced is under one just-noticeable difference on average everywhere, and its 95th percentile is at about
one**: a player stepping down from Accurate to Balanced should not see the picture change, only the fine
structure within a triad (the full-resolution difference is 9 to 11 code values, from the box against the
tent footprint and sampled against integrated lines).

**Performance is 3.5 to 7.8 on average, visibly different, and the breakdown says where** (Super Mario World 2,
each feature turned off in both tiers in turn): glare 1.1, convergence 0.9, persistence 0.05, **the composite
signal 3.5**, the mask and the beam the last 1.4. Most of Performance's difference is the colour artifacts it
does not draw, which is what it was designed to leave out; by RGB, with the glass, convergence and persistence off in both, it is 1.4 from Accurate. Its picture
is 2 to 3% brighter, the glare's light that is not scattered; *Bomberman 64* is 5% darker, because two lines
are too few for an interlaced picture's doubled spots.

**Earlier versions, recorded**: Performance was 7 to 12 ΔE with the glass flat, half of it the curvature's
geometry; curvature costs it 0.01 ms (§12.5) and it now has it.

### 12.5 What they cost, and the predictions' verdicts

§11.10's bench and protocol, the three tiers interleaved over three rounds; and, since this card's paced clock
makes light frames look heavier than they are (§11.10), the same 1080p cases once more flat out (`pace=0`),
where the card runs at its full clock. Cases and results: `cases-tiers.txt`, `results-tiers.txt` beside the
bench.

| Case (SNES 256 × 224, composite) | Performance | Balanced | Accurate | Built-in Lottes |
|---|---|---|---|---|
| 1080p, paced, GPU ms | **0.49** | **0.71** | 1.77 | 0.74 |
| 1080p, flat out, GPU ms | **0.20** | **0.40** | 1.08 | 0.42 |
| 1280 × 800, paced | 0.39 | 0.57 | 1.58 | |
| 4K, paced | 0.90 | 1.39 | 3.36 | 1.89 |
| 1080p by RGB, paced | 0.46 | 0.60 | | |
| N64 640 × 240, 1080p, paced | 0.54 | 0.71 | | |
| Render thread, ms | 0.11 | 0.17 | 0.20 | 0.09 |

As a share of the built-in Lottes at the same output, flat out at 1080p: Performance 0.47, Balanced 0.96,
Accurate 2.6.

**Accurate is slower than in §11.10**: 1.77 against 1.70 ms at 1080p and 3.36 against 2.86 at 4K, paced. The
scanline flattening of §12.3 (a line's even light and peak, tracked for each of five lines and three guns) is
the change; it is the price of the bright default not clipping.

| | Prediction (§6.1) | Verdict |
|---|---|---|
| C1 | Performance: 0.10 to 0.13 ms at 1080p, 0.25 to 0.35 at 4K | **Refuted, high**: 0.49 and 0.90 paced, 0.20 flat out. The estimate counted texel reads and transcendental functions against Lottes's and found Performance an eighth of it; measured, it is half. What the count missed is the arithmetic between them: the closed-form mask (eight piecewise integrals a pixel), the curvature's mapping three times a pixel, the placement of the picture and the headroom rules each carry tens of instructions with no read in them |
| C2 | Balanced: 0.3 to 0.45 ms at 1080p, 0.6 to 0.95 at 4K | **Refuted, high**: 0.71 and 1.39 paced, 0.40 flat out, for the same reason, and for convergence's three reads a line (§12.2), which the prediction did not include |
| C6 | On the laptop only Performance holds 60 fps | Untested. Scaled by the Lottes ratio, Performance there would be about half of Lottes's 10 to 17 ms estimate: 5 to 8 ms, which fits a frame; Balanced would be about Lottes's own |

**The weak-machine case rests on Performance at about half a Lottes**, not the eighth predicted. Whether that
holds 60 fps on the laptop's integrated graphics is the question §12.7 asks to be measured.

### 12.6 Tests and mutants

`ScreenFilterRenderTests.Tiers.cs`, 4 cases beside §11.9's, on the GL device:

- **A flat field keeps its light and colour** in Balanced and Performance: white and a 50% grey within 3% and
  neutral within 1.5%, at 1440 and 2880 wide and with the spot twice the screen's.
- **Each lower tier stays within its measured distance of Accurate** on a synthetic scene (bars, a ramp, a
  checker, a colour sweep): Balanced under 2 ΔE, Performance under 8 and more than Balanced.
- **Balanced keeps the Genesis's rainbow** (30 source pixels long within 4%, blue swinging by more than its
  mean) **and Performance blends the columns without one** (green and blue swing under 0.2 of the mean, five
  times less than by RGB).
- **The default is bright**: Quality's default is Balanced and Picture brightness's is Bright; white is 0.75 of
  the display's peak within 3%, nothing clips, a dark grey's mask is as deep as with all the headroom it needs,
  and white's is under half of it.

§11.9's cases now draw Accurate at the tube's own brightness unless they say otherwise, so their figures still
mean what §11 says; all of them pass with the scanline flattening in place.

**Mutants** (12, each caught): Performance without its notch; its notch at the wrong gain; one sample per source
pixel; its transfer left out; Balanced without convergence; Balanced's signal at 2 samples a cycle; Balanced
drawing one line; white at half the peak; no scanline flattening; the flattening by the pixel rather than the
line; Accurate or the tube's own brightness as the default. The white-at-half mutant survived the first
version of the brightness case, which asserted the filter's own constant; it asserts 0.75 now. The one-line
mutant survived the first flat-field case, whose spots were too narrow for a missing line to lose light; the
case now also draws spots twice the size.

### 12.7 Measurements wanted on the handheld and the laptop

The tiers exist, so this is the ask of §9's ninth decision. **Nothing has been run on either machine.**

**A build to hand over**: `~/.cache/emusen/probe/crt/handover/crt-bench.tgz` (51 MB; the bench of §8.1 with
this branch's Serenity, self-contained for linux-x64, its native libraries asking for glibc 2.27 at most). It
holds `run.sh`, `matrix.sh` and `cases.txt`: no filter, Lottes and the three tiers at the panel's 1920 × 1200
and at 1280 × 800, and Performance for the N64 and the Genesis, three interleaved rounds, about twelve minutes.

On the handheld, as the tester runs it:

1. Copy `crt-bench.tgz` to the handheld and unpack it: `tar xzf crt-bench.tgz` (it makes `crt-bench/`).
2. Run it under a user scope, so that closing the terminal or the SSH session does not kill it:
   `systemd-run --user --scope bash crt-bench/run.sh 3`. On the charger or on battery, but say which; §8.4
   there found the two differ.
3. Send back `crt-bench/results-<hostname>.txt` and `crt-bench/machine-<hostname>.txt`.

The laptop needs the same, with permission asked first.

### 12.8 What is not done

- **Compiling off the render thread** (§11.12, decision 6). Step 3 changed no code that builds the chain, so it
  goes with step 4, which adds the frontend's settings changes that would trigger it.
- **The Shaders window** (decision 4), step 4.
- **Performance for an interlaced picture** draws two lines where a 480-line picture's doubled spots reach
  three; it is 5% darker than Accurate there (§12.4).

### 12.9 Questions this step raises

1. **The bright default's share** (§12.3): three quarters of the display's peak, with the mask half deep at mid
   grey, or a lower share for a deeper mask. **Decided 2026-10-07 by the tester: white at 0.6 of the display's
   peak**, with §12.3's depth table as the evidence (at 0.6 the mask is 1.47 of its full 2.43 at mid grey, 0.70 at
   a drive of 0.75 and 0.29 at white). The tube's own brightness stays the other choice.
2. **Performance's cost** (§12.5): half a Lottes on the RX 6800. If the laptop shows that is too much, the next
   reductions are the curvature (0.01 ms here), the mask's slot rows, and a single beam read; each would be
   measured against §12.4's distance first. **Decided 2026-10-07: the tester runs the handheld and laptop benches
   with §12.7's package; the reductions wait for their results.**

---

## 13. Offered to players (2026-10-07)

**Listed per console.** **CRT** is one of `ScreenFilters.All`'s choices, after None and before CRT (Lottes), and
`NamesFor` offers it to the NES, SNES, Genesis and N64 and to no handheld. The default filter stays None
(§9, decision 4).

**The running console is told.** Mistress's `ApplyScreenFilter` sets `GameFrameControl.FilterConsole` before the
filter, so the chain starts from that console's constants (§11.1's `ConsoleTiming`) and defaults. Before this, the
filter in Mistress would have drawn every console with a standard line's timing: the Genesis's static artifacts as
the N64's crawling ones.

**Its choices read as choices.** The Shaders window draws each parameter that names its values
(`SlangParameter.Choices`) as a row of named choices: LunaP's `SliderRow.Choices` (`LunaP.md` §199), a slider stepped
from one choice to the next, showing *Balanced*, *Composite*, *Consumer TV, 20-inch* and "default ..." where it showed
2, 2 and 1. Left and right still step, so the pad's grammar is unchanged; why not a dropdown is `LunaP.md` §199. A
built-in filter's row defaults are now `ScreenFilter.DefaultFor` the console the tab is for, not the parameter's own,
which is how a console's own default would show; the CRT's are the same for all four consoles. Stored values are the
choices' numbers, as every other parameter's are (`EmuSen_Settings_Reference.md` §4.48.3).

**Built off the render thread**, with the picture before it kept until the new filter is ready, and its passes warmed
a draw at a time (`EmuSen_Serenity.md` §3.10, decision 6 of §11.12): a Quality change now costs at most 5.9 ms of
one draw with the driver's cache warm and 29 ms with it cold, against 4 to 10 and 52 when it was built in the draw.

**No real CRT frame on Skia's software path in a window or pad test.** The filter `RequiresDevice`, so every
headless window and pad test draws the plain picture in its place: that is the stand-in, for the Shaders window's,
the pad menu's and the fit audit's tests, which test the window. Where a real picture is wanted, the tests draw on
the device (`ScreenFilterRenderTests` and `ScreenFilterDeviceTests`, through `ShaderBench.Picture`), and the two
cases that test the software path itself say so with `DrawDeviceFiltersInSoftware`.

**A pad test's route, changed.** With one more built-in in the list, `The_shaders_window_from_the_pad_menu_adjusts_a_
built_in_filter_and_resets_it_all` failed in all seven of its rows: its search for a pad path from a slider to Reset All
went left onto the list, where the row it landed on (Simple CRT now, a heading before) changed the shader shown and
with it the controls the search was walking, and no path was found. The route a player takes exists and is short (up
the column of sliders to the row of Reset All and Use), and the test now presses it rather than searching for it; the
same for its later step to Use. The search's fragility, a walk that changes what it walks, is the harness's and
is recorded in `EmuSen_Settings_Reference.md` §4.96. The window itself was not changed.

**Tests run for this step** (all on the RX 6800 where they draw): `ScreenFilterRenderTests`, `ScreenFilterChainTests`,
`ScreenFilterDeviceTests`, `GameFrameControl`'s, `SlangFrameControlTests`, `ShaderSettingsWindowTests` (a new case:
the CRT's choices shown by name, stepped, and stored by number), the pad menu's and pad settings' shader cases, the fit
audit's Shaders and Graphics cases at 1280 × 800 and 1920 × 1200 (a new case, `ShaderSettingsCrt`, with the CRT shown
and its longest choices against their defaults), and LunaP's `SliderRowTests` and `SliderListTests`. Every existing
filter still hashes identically, through the raster path and on the device. **Mutants** (8, each caught): the console
not told; the choices not shown; the CRT not listed; and §3.10's five in `EmuSen_Serenity.md`.

**What is not done.** The tester's bench runs on the handheld and the laptop (§12.7, §12.9), and what they decide for
Performance; the NES's raw signal (§9, decision 5); the decoders' axes (decision 11); and moving one program's compile
off the render thread, which Skia's GL backend gives no way to do (`EmuSen_Serenity.md` §3.10).
