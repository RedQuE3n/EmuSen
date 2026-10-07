# EmuSen_CRT — a physically modelled CRT filter

A design (2026-10-07), not yet built. The request was for a very accurate CRT filter, offered in tiers if it turns
out to be a performance hog: the lowest tier for performance, the highest for accuracy. This document is step 1 of
four: the survey of existing shaders, the physical literature with a source for every number, the choice of
delivery path, the model, the tiers with their predicted costs, the player's settings, and the questions that need
a decision before any shader is written. Steps 2 to 4 (the Accurate tier, the two reductions, the settings window)
will add their sections here as they are built.

**Reading order.** §1 for what is modelled. §4 for where it runs and what `FilterChain` must learn first. §6 for
the tiers. §9 for the decisions wanted. §2 and §3 are reference: who already does what, and what the hardware
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
| 2 | The Accurate tier, its render tests, its cost on the RX 6800 at 1080p and 4K | not started |
| 3 | Balanced and Performance as reductions, each measured against Accurate | not started |
| 4 | Graphics settings per console, the settings reference, the fit audit | not started |

Nothing in `EmuSen.Serenity` has changed. The existing **CRT (Lottes)** (`EmuSen_Serenity.md` §3.4) and **Simple
CRT** stay as they are.

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

Three stripes need at least three samples per triad to be told apart at all and six to be drawn without beating.
Whole pixels give that only for consumer sets at 4K. **Display subpixels** give three times as many samples, so a
consumer slot mask is drawable at 1080p (8.4 subpixels per triad) and a PVM's grille at 4K, and nothing draws a
PVM's grille at 1080p. §5.6 is built on this.

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

Extension 6 changes what the picture looks like, not only how it is filtered, and is §9's question 2.

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
bandwidths (§3.1, §3.2), set-up is added for a US signal, and the output is `Y` and `C = U sin φ + V cos φ`. RGB
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
  by a kernel that is a wide Gaussian plus the ring of §3.7, and added to the direct light with the direct
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
| Passes | 2 | 5 | 6 |
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

Further predictions, to be given verdicts in steps 2 and 3:

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

## 9. Decisions wanted

1. **The delivery path** (§4.1): the built-in SkSL chain for every tier, with §4.2's six extensions to
   `FilterChain`. Recommended.
2. **The picture's shape** (§4.2, extension 6): under the CRT filter the picture becomes a 4:3 tube with the
   console's own pixel aspect, where today every picture is shown with square pixels. And overscan: show the
   whole active picture (recommended as the default, since nothing is hidden) or crop as a television did
   (about 7% in a PVM's normal scan; consumer sets unmeasured).
3. **One entry or three** in the filter list: one **CRT** with a Quality setting (recommended, so a player's
   other settings carry across tiers), or **CRT (Performance)**, **CRT (Balanced)** and **CRT (Accurate)**.
4. **Defaults**: Composite for the NES, SNES, Genesis and N64, as §7. And whether **CRT** becomes any console's
   default filter, or the default stays None.
5. **The NES's signal.** An accurate NES composite needs the PPU's colour index and emphasis bits per pixel
   from Moon and MoonRT, a small addition to what a core hands the presenter. Recommended: build steps 2 and 3
   from RGB, and take the raw path as its own step afterwards with its own design note.
6. **Region.** The default colour standard (North America recommended: SMPTE C, D65, set-up, US decoder axes),
   whether a Japanese white is shown as it was or adapted to the display's, and whether the frontend should
   pass a game's region so the default follows it.
7. **Persistence at its measured strength** (§3.5), given that it is one monitor's measurement with a
   low-confidence tail. Recommended: yes, with the slider.
8. **Interlace.** A 480i picture drawn field by field, flickering as it did, or woven. Recommended: woven by
   default, fields as a setting, decided properly when step 2 reaches it.
9. **Measurements on other machines.** Step 3's costs on the handheld and the weak laptop need a bench run on
   each; C1 to C3's right-hand columns have nothing else under them. To be asked for when the tiers exist.
10. **Later stages**, recorded and not proposed for now: the sub-frame flash (§5.9), an HDR presentation path
    (§4.1), PAL, RF, raster bloom.
11. **The datasheet tables** (§3.1's decoder axes): the Sony PDFs could not be fetched and the figures are a
    compilation's transcription. Recommended: build only the ideal decoder until the datasheets are read, then
    add the others with the page cited.

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

**Shaders read**, all from libretro's `slang-shaders`: §2.1's and §2.2's lists, each with its author and
licence.
