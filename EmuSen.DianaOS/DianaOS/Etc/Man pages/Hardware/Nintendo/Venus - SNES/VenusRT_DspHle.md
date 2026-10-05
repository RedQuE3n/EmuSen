# VenusRT_DspHle — open replacements for the NEC DSP programs

*Written 2026-10-04, before any code.* This page plans the open replacements VenusRT carries for the programs inside
the cartridge's NEC DSP chips: the DSP-1 and DSP-1B, DSP-2, DSP-3 and DSP-4 (µPD77C25) and the ST010 (µPD96050), with
the ST011 assessed. A replacement is a model of what each of a chip's commands computes, written from public
documents. It is called high-level emulation (HLE) below. The interpreter that runs the real program is called
low-level emulation (LLE).

*Decided 2026-10-03* (`EmuSen_Firmware.md` §0, `EmuSen_CoreAPI.md` §6.2): every core the project builds carries an
open replacement for any firmware, so that every game runs with no firmware folder. A firmware image the player
supplies is used as the exact path when it is present, and no frontend prompts for one. This reverses `VenusRT_Plan.md`
§9, Q2 ("There is no high-level emulation of the DSPs"), which is to be marked superseded there when the first step of
§8 lands.

The page keeps three things apart, as `VenusRT_Plan.md` does:

- **Measured** means a fact taken on 2026-10-04 with the tool named. Only listings were measured for this page: the
  documents, the firmware folder's file names and sizes, and the library's titles. No command was run.
- **Argued** means reasoning from the documents or from VenusRT's record, with no measurement behind it here.
- **Predicted** means a claim numbered P1–P12 (§11), so that a later step can retire it.

**Amendments.** The text below is the plan as written. Where a step's measurements correct it, an amendment follows
the passage it corrects, marked *Amended* with its date and the record it rests on; the original text stays. Step 1's
corrections (`VenusRT_Native.md` §37) are in §2.1, §3.1, §3.5, §4.3, §6.2 and §11; step 2's record is
`VenusRT_Native.md` §38.

---

## 0. Summary

| Chip | Games | Documentation of the commands | Game in the tester's library | Verdict (argued) |
|---|---|---|---|---|
| DSP-1, DSP-1B | about 19 (fullsnes's list) | Every command named, with its inputs, outputs and an equation for about half; cycle counts for all; no algorithm, table or rounding rule | Pilotwings, Super Mario Kart, Michael Andretti's Indy Car Challenge, Ballz 3D, Lock On, Super Bases Loaded 2, Suzuka 8 Hours | **Exact for the arithmetic and trigonometric commands, predicted; approximate for the projection commands until the oracle shows otherwise** |
| DSP-2 | Dungeon Master | Names only; the bitplane format the main command produces is documented elsewhere | Dungeon Master (U) | **Exact, predicted**: the commands are mechanical, and the oracle fixes the parameter layouts |
| DSP-3 | SD Gundam GX | Names and one-line descriptions | none | **Not now.** One command decodes a Shannon-Fano bitstream whose code table may be the program's own data, which no document gives |
| DSP-4 | Top Gear 3000 | Nothing beyond two test commands and the transfer width | Top Gear 3000 | **Approximate at best**, and only by black-box characterisation; a go or no-go step first |
| ST010 | F1 ROC II / Exhaust Heat II | Names, the RAM mailbox protocol and each command's parameter addresses | F1 ROC II | **Exact for the commands the game uses, predicted**; the rest approximate |
| ST011 | Hayazashi Nidan Morita Shogi | Command numbers only; it is a shogi engine | none | **Not feasible as a replacement.** The game stays unsupported without the player's image (§3.6) |

**The oracle** (§4) is the design's centre. VenusRT's LLE runs the tester's dumps exactly as the hardware runs them, so
every replacement command can be graded against it bit for bit: the same command and inputs to both, through the
ports alone, over edge cases, seeded random inputs and, where the input space allows, every input. Then the bench games
run in lockstep, LLE against HLE, frame by frame. The tests skip cleanly when no dump is present, and nothing of the
firmware enters a test, a table or the repository.

**The effort** (§8) is about 21 supervised steps of three hours, with DSP-4's build (four to six more) behind a go
decision, and DSP-3 and ST011 parked.

---

## 1. Purpose and the protocol

### 1.1 What a replacement must be

A DSP cartridge's game sees its chip only through two ports, the data register DR and the status register SR (and, on
the ST01x, a window onto the chip's RAM). The game writes a command, writes parameters, polls SR until the chip is ready,
and reads results. A replacement therefore needs to reproduce, at those ports, what the S-CPU would see: the result
bytes, the SR values, and when SR's request bit rises. Nothing inside the chip is visible to the game, so a replacement
is free to compute in any way that yields the same port behaviour. That freedom is what makes a model written from
documents possible at all, and it is also the measure of its accuracy: §4 grades the ports, not the method.

### 1.2 Who may read what

This extends `VenusRT_Plan.md` §1.2's table, whose rows all still hold. The rows this work adds:

| Source | A writer | Notes |
|---|---|---|
| Public documents describing what the commands compute: fullsnes, the SNESdev wiki, SnesLab, the superfamicom wiki's prose and address tables, datasheets | yes | §1.4 lists what was read; each step adds its own |
| The firmware images (`home/Firmware`, the corpus's `firmware/` folder) | **never read** | Loaded only by the LLE inside the oracle (§4) and by `firmwarecheck.py`, which prints counts and offsets only (§5.3) |
| Any listing, disassembly or dump of a DSP program, in any document | **no** | Includes fullsnes's own listings where it has them, and VenusRT's µPD77C25 disassembler pointed at a dump |
| The LLE's outputs, through the ports | yes, **as grades only** | §1.3's rules |
| Other emulators' DSP code: bsnes/higan's `dsp1`–`dsp4` and ST010 modules, snes9x's, MAME's, Mesen's, C# Venus's `NecDsp*` and `Venus_NecDSP.md` | **no** | Includes any page that embeds such code (§1.4 records one) |
| SNES_MiSTer's RTL | not for this work | Its DSP is an interpreter of the same programs and says nothing about what they compute; §10, Q7 |
| Nintendo's development manual (Book II carries the DSP-1's command chapter) | **no** | `VenusRT_Plan.md` §9, Q1; SnesLab and the SNESdev wiki, which cite it, are read as any document is |

### 1.3 The black-box boundary: what may be learned from the LLE

The oracle answers any question of the form "what does the chip output for this input". Asked often enough, the same
questions would recover the program's tables entry by entry, which is copying by another road. Grading and extraction
use the same instrument, so the line between them is drawn by rules about how a model is chosen:

- **R1. The LLE's outputs are grades, never sources.** No value observed from the LLE enters the code, a test's
  expected values or a table. A step's record may cite a handful of individual input and output pairs as evidence for a
  rule; result files go to the probe cache (§4.5).
- **R2. Each command's model is a documented formula plus a family of variants declared in advance.** Before the first
  comparison, the step's record writes down the formula and the choices it leaves open: a rounding mode, a word width,
  a table's length, an interpolation, an iteration count, the order of rounding steps. The oracle then selects among
  them. The record states how many bits of choice the family has.
- **R3. No per-input special cases** unless a documented rule produces them (saturation at 7FFFh is a rule; "input
  1234h gives 0F0Fh" is not).
- **R4. A table that no formula in the family yields is a stop.** A code tree, a list of constants or a hand-tuned
  curve is the program's data, not a computation. The command is then recorded as not replaceable from documents and
  becomes a named loss. It is not probed entry by entry.

> *Amended 2026-10-04* (decided; `VenusRT_Native.md` §52.1). R1 admits one behavioural constant measured from the LLE:
> the DSP-1's limit angle on Azs, past which the projection commands change course. It is treated as a measured
> latency is, with the measurement and its precision in the record. No other value past the limit is admitted: what
> the chip does there is stated as formulas with declared families under R2.

**The argument for exactness without copying.** A model with b bits of free choice that reproduces N independent
output bits, with N much larger than b, carries at most b bits of information taken from the oracle; everything else
came from the formula. A sine table of 256 sixteen-bit words is 4,096 bits; a family of table lengths, three rounding
modes and two interpolations is under 8 bits. R2 makes b small and recorded, and §5.3 checks mechanically that the
core's tables hold nothing beyond a formula's output.

**Black-box characterisation** of a command whose function no document states (DSP-4, DSP-3, parts of the DSP-1 and
ST010) is within these rules: a writer forms a hypothesis from the inputs and outputs' structure, states it as a
formula with its family, and grades it. That is how the public documents themselves were made, and it is the standing
Mesen has under `VenusRT_Plan.md` §1.3. It is slower than reading a document, and §8 prices it so.

### 1.4 What was read to write this page

So that the protocol can be audited from its first day:

- **fullsnes** (pinned copy `~/.cache/emusen/probe/venusrt/docs/fullsnes.txt`, SHA-256 `390b9717e7bf4e86…`): lines
  10474–10730, "SNES Cart DSP-n/ST010/ST011", its chip list, header notes, "DSP Mapping", "SNES I/O Ports",
  "Registers", the flags, "Status Register (SR)", RQM, DMA, "Memory" and "ROM-Images"; lines 10914–10952, "List of Games
  using that chips"; lines 10953–11097, "BIOS Functions", the DSP-1, DSP-2, DSP-3, DSP-4, ST010 and ST011 command lists;
  lines 26886–26890, the cartridge oscillators. The ALU and jump chapters (10731–10913) were not reread for this page.
- **SNESdev wiki, "DSP-1"** (<https://snes.nesdev.org/wiki/DSP-1>, revision 1364 of 2025-02-13, raw text pinned at
  `docs/dsp-hle/snesdev-DSP-1.wiki`): the ports, the data types, and eight commands with inputs, outputs, equation and
  cycle split. Its DSP-2, DSP-3 and DSP-4 titles redirect to it.
- **SnesLab** (<https://sneslab.net/wiki/DSP1> and its subpages, read as raw wikitext 2026-10-04, pinned under
  `docs/dsp-hle/sneslab/`): `DSP1` (revision 14021), and `DSP1/Attitude`, `Distance`, `Gyrate`, `Inverse` (20958),
  `Multiply` (14030), `Objective`, `Parameter` (14018), `Polar`, `Project`, `Radius`, `Range`, `Raster` (14017),
  `Rotate`, `Scalar`, `Subjective`, `Target`, `Triangle` (14028); `DSP2` (14002), `DSP3` (14033), `DSP4` (14034). Each
  command page gives its opcode, its cycle count and time, its parameters by name and the manual page it was taken from.
  Each also links to a line of bsnes's source; **those links were not followed.**
- **The superfamicom wiki, "ST010"** (<https://wiki.superfamicom.org/st010>, fetched 2026-10-04) and "ST011"
  (<https://wiki.superfamicom.org/st011>, which says only "N/A"). The ST010 page's prose and tables give the board, the
  mailbox protocol, the command list and each command's parameter addresses, and §3.5 uses those. **Beneath them the
  page carries C source for each command and two numeric tables**, credited to the authors of an emulator's ST010 module.
  The page was printed whole when fetched, before the code was recognised, so the author of this page has seen it.
  Nothing of it is used here: §3.5 is written from fullsnes and the page's address tables alone. Two consequences are
  built into the plan. The ST010 step (§8, step 11) is written by someone who has not read that page, or else the
  exposure is recorded in that step's record. And that page's code joins the clone check's comparison set (§5.4).
- **Seen in search results and not opened:** a GitHub repository of a Python DSP emulator, a pull request adding a DSP-4
  to another emulator, a libretro document on higan's accuracy, and two bannister.org forum threads on emulating the
  µPD77C25 and µPD96050, which are likely to quote code.
- **Of the project's own pages:** `EmuSen_Firmware.md`, `EmuSen_CoreAPI.md` §6.2–§6.4 and §6.13, `VenusRT_Plan.md`
  (whole), `VenusRT_Native.md` §24, §27.3, §32–§34, `VenusRT_Disputes.md` D-38 (the model for this page's provenance
  rules) and `EmuSen_Debugging_Tools_Reference_v5.md` §3.61 (`firmwarecheck.py`, on branch `venusrt`).
- **Of the code:** VenusRT's `chips/necdsp.rs` (the LLE, its port interface and its state), the firmware parts of
  `v1.rs` and `cart.rs` (`firmware_for`, `nec_firmware`), `firmwarecheck.py`; and of the frontends,
  `CoreEngine.GetFirmwareRequirements`, `EmulatorSession.MissingFirmwareFor`, `FirmwareRequest` and Mistress's
  `PromptForMissingFirmwareAsync`. Of `NecDspRealFirmwareTests.cs`, only its skip pattern (`if (dsp == null) return;`).
  No C# Venus source, no `Venus_NecDSP.md`, no firmware image.
- **Not read:** the NEC µPD77C25 and µPD96050 datasheets. `VenusRT_Plan.md` §2.1 still lists them as to be pinned.
  They describe the processor, which the LLE already implements, and say nothing about the programs.

---

## 2. What every chip shares

### 2.1 The ports (fullsnes, "SNES I/O Ports" and "Status Register (SR)"; SNESdev wiki, "DSP-1")

- **DR** is the data register, 8 or 16 bits by SR's DRC bit. In 16-bit mode the S-CPU moves a word as two bytes, low
  first, and DRS toggles after each.
- **SR** is read-only to the S-CPU, which sees its high byte: RQM (bit 15; 1 means the chip waits for the S-CPU to read
  or write DR), USF1 and USF0 (14–13, the program's own flags, of which the DSP-3's decoder uses USF1), DRS (12), DMA
  (11, unwired on the SNES), DRC (10), SOC and SIC (9–8).
- **RQM** rises when the program reads or writes its side of DR, and falls when the S-CPU completes the transfer: one
  byte in 8-bit mode, the second byte in 16-bit mode.
- **The maps** are fullsnes's table, already implemented by the LLE's board code (`cart.rs`, `VenusRT_Native.md`
  §24.1): DR and SR at 30–3F:8000/C000 for a 1 MB LoROM, 20–3F for one with RAM, 60–6F:0000/4000 for the 2 MB LoROM,
  00–1F:6000/7000 for HiROM; for the ST01x, DR and SR at 60–6x:0000/0001 and the chip's RAM, 2K × 16 bits, at
  68–6F:0000–0FFF as 4K bytes, even addresses the low byte.
- **What a DSP-1 does with a parameter request** (fullsnes, "DSP1 Commands"): the chip "is oblivious to the type of
  operation that occurs to the Data Register". A write or a read both let it continue, and on a read it takes whatever
  DR holds. When a command finishes, DR holds 80h, "to prevent a valid command from executing should a device read past
  the end of output". The DSP-3 does the same, and the DSP-4 leaves FFFFh.

These are port behaviours the replacement must reproduce, and the oracle's abuse cases (§4.1) test each.

> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §37.2). A command's read raises RQM as the request for its first
> input, so a command without inputs still takes one transfer in either direction before its first result. DRC changes
> to 16-bit 3 cycles after that rise. On the DSP-1 and DSP-1B, 40h-FFh are passed over and the next byte is taken as a
> command, which is how games resynchronise. The DSP-4 is 16-bit throughout, its idle word FFFFh written again after
> an S-CPU read at idle.

### 2.2 The shape of a replacement (argued; a plan, not code)

- **One slot, two engines.** The cartridge's `dsp` becomes an engine behind the LLE's existing `Port` interface
  (`host_read`, `host_write`, `run_to`): the LLE as now, or a replacement per chip. The bus, the maps and the ST01x
  battery code do not change.
- **A replacement is a state machine over the ports**: idle (waiting for a command byte), taking parameter k,
  computing, giving result k, idle again. It carries `ready_at`, the chip cycle at which RQM next rises, on the clock
  the LLE uses (7.60 MHz for the DSP-n, 10 MHz for the ST01x, D-32), caught up at each access as the LLE is. A read of SR
  returns RQM set when the chip's clock has reached `ready_at`, and the DRC, DRS and USF bits the phase calls for
  (§6.2).
- **Internal state the commands share** is the replacement's own: the DSP-1's three attitude matrices and its
  projection parameters, the DSP-2's transparent colour, the DSP-3's board. On the ST01x the RAM is real and shared
  with the S-CPU, so the replacement keeps it as the LLE does: same size, same layout, same battery file.
- **State.** The `Coprocessor` group gains a tag saying which engine wrote it, and the state version rises. A state
  saved under one engine is refused by the other with a status and `last_error`, as `EmuSen_CoreAPI.md` §6.9 prescribes
  for a state the machine cannot take. Rewind is unaffected, since a running game never changes engine.
- **The debugger.** Under a replacement there is no µPD77C25 program, so processor 2 and the `DSPPRG` space are not
  listed (machine info is per machine, `EmuSen_CoreAPI.md` §6.4). `DSPRAM` stays for the ST01x, whose RAM is real.
- **Reset** (fullsnes, "Reset", as the LLE applies it): the replacement returns to idle; the ST01x RAM is kept.

---

## 3. The chips, command by command

Each table gives, in this page's words, what a command takes and gives and what it computes, the documented cycle
count, the sources, and how far the documents go towards an exact result. "Characterise" means the documents leave the
function itself open, and §1.3's black-box characterisation fills it.

### 3.1 DSP-1 and DSP-1B

fullsnes: the DSP-1 and DSP-1A hold the same program; the DSP-1B is "a bug-fixed DSP1/1A version". VenusRT's
`nec_firmware` names the DSP-1 for Pilotwings, the game fullsnes names with "visible DSP1 glitch", and the DSP-1B for
every other DSP-1 game (`VenusRT_Native.md` §32.1). The replacement implements both and differs between them only
where the oracle shows the two programs differ (§8, step 1, measures the map).

**Data formats** (SNESdev wiki, "Data Types"; units as the table gives them): A, an angle, a full turn in 2^16 steps
(the table's bit column says 8, which its own unit contradicts); T, a fraction with 15 bits after the point; I8, 8
integer and 8 fraction bits; I, a 16-bit integer; 2I, an integer of 17 bits counted in twos; U, unsigned; D, a 32-bit
integer sent as L then H; D2, L2, H2, a 32-bit value in halves; M and C, the mantissa and exponent of a floating value
M·2^C. Parameters and results move as 16-bit words, low byte first.

**Commands** (fullsnes "BIOS Functions" for the codes and names; SnesLab per command for parameters and speeds;
SNESdev for equations and the cycle split; "n" in a code is the attitude matrix A, B or C as 0, 1, 2):

| Code | Name | Takes → gives | Computes | Cycles (SnesLab; SNESdev split) | From the documents |
|---|---|---|---|---|---|
| 00h | Multiply | K, I → M | K·I scaled by 2^-15; the wiki says "rounded to ≤ 15 bits" | 26; 6 + 12,4 + 4 | Formula; rounding is a family of four (§5.1) |
| 20h | Multiply (second form) | as 00h, presumed | fullsnes lists it as a second "16-bit Multiplication" with no difference stated | — | **Characterise** |
| 10h | Inverse | a (M), b (C) → A (M), B (C) | A·2^B = 1/(a·2^b); exponents 8002h–7FFFh | 98; 6 + 12,73 + 2,4 = 97 | Formula only; the algorithm (a seed and iterations) is not documented |
| 04h | Triangle | θ (A), r (T or I) → S, C | S = r·sin θ, C = r·cos θ | 59; 6 + 12,24 + 3,4 = 49 | Formula; the sine's resolution, rounding and interpolation are a family (§5.2) |
| 08h | Radius | x, y, z (I) → L2, H2 | x² + y² + z², a 32-bit result in the "double half" format | 34; 6 + 14,4,4 + 2,4 | Formula; the scale is fixed by one comparison |
| 18h | Range | x, y, z, r → D | x² + y² + z² − r², high word | 38; 6 + 12,4,4,8 + 4 | Formula |
| 38h | Range (second form) | as 18h, presumed | fullsnes lists it as a second "Vector Size Comparison" | — | **Characterise** |
| 28h | Distance | x, y, z → R | √(x² + y² + z²); "bugged" on the DSP-1/1A, fixed on the DSP-1B, the bug "evident in Pilotwings (Plane Demo)" | 156; 6 + 15,4,127 + 4 | Formula; the root's method and the DSP-1's bug are **undocumented** |
| 0Ch | Rotate | θ, x1, y1 → x2, y2 | the point turned counter-clockwise by θ about Z | 64 (the manual's summary page says 65, per SnesLab); 6 + 12,3,37 + 2,4 | Formula; rounding order is a family |
| 1Ch | Polar | three angles (about Z, X, Y), x, y, z → X, Y, Z | the point rotated by the three matrices in the wiki's stated order | 147; 6 + 13,3,2,2,2,107 + 6,2,4 | Formula; the order of the rounding steps is a family |
| n1h | Attitude A/B/C | m, angles about Z, X, Y → nothing | sets matrix n to m times the rotation the three angles give | 164 | Names only; the product order and scaling are **characterised** through the commands that use it |
| nDh | Objective | X, Y, Z → F, L, U | a global point into matrix n's object coordinates (fullsnes: "Global to Object") | 45 | Names; a matrix-vector product, transposition **characterised** |
| n3h | Subjective | F, L, U → X, Y, Z | the inverse: object coordinates into global ("Object to Global") | 44 | as nDh |
| nBh | Scalar | X, Y, Z → S | the inner product of the vector with matrix n's forward axis | 36 | as nDh |
| 14h | Gyrate | three angles → a rotational angle (SnesLab) | fullsnes calls it "3D Angle Rotation"; what it computes is not stated | 444 | **Partly documented**: the parameter list may be incomplete, and the function is characterised |
| 02h | Parameter | Fx, Fy, Fz (a point), Lfe, Les (two distances), Aas, Azs (two angles) → Vof, Vva (raster numbers), Cx, Cy | sets the projection: the eye's position and direction and the screen's distance, for Raster, Project and Target | 892 | Parameters named; **no equation** in any document read |
| nAh | Raster | Vs (a raster line) → An, Bn, Cn, Dn | the mode 7 matrix elements for that line under the current projection; opcode bit 4 asks for output by DMA, which the SNES cannot take | 433 | Parameters named; **no equation**; whether one command gives one line or a run is characterised |
| 06h | Project | X, Y, Z → H, V, M | where a point appears on the screen and its enlargement ratio | 627 | **No equation** |
| 0Eh | Target | H, V → X, Y | the ground point under a screen position | 228 | **No equation** |
| 0Fh | Memory test | → a status word | a self-test of the data RAM | — | Output value **characterised** (a single constant on a working chip, argued) |
| 1Fh | Transfer data ROM | → 1,024 words | the data ROM's contents | — | **Not replaceable**: the output is the firmware's data. Named loss: the replacement answers with zeros |
| 2Fh | ROM version | → 0100h (DSP-1/1A) or 0101h (DSP-1B) | | — | **Exact** from fullsnes |

> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §37.5, §37.6). 20h is not the same as 00h, nor 38h as 18h. Gyrate
> takes six inputs and gives three. Raster is a run of lines that the S-CPU ends by writing over results. The DSP-1 and
> DSP-1B differ, for a game, only in 28h. The bench games use 00h, 01h/11h, 02h, 03h/13h, 04h, 06h, 0Ah, 0Ch, 0Dh,
> 0Eh, 10h, 14h, 1Ch and 28h, and none uses Radius, Range, Scalar, 20h or 1Fh.

**Where the documents stop.** What is documented is the interface and, for the general and vector commands, the
mathematics. What is not is every fixed-point detail that decides the last bit: the sine table's resolution, the
reciprocal's seed and iterations, the root's method, the order of the rounding steps in the matrix products, and the
equations of the four projection commands. fullsnes does not say whether command bytes beyond those listed mirror
others; §8, step 1 sweeps all 256.

### 3.2 DSP-2 (Dungeon Master)

fullsnes, "DSP2 Commands": the codes and names below, and "10h..FFh Mirrors of 00h..0Fh". Nothing documents a
command's parameter count, order or length. fullsnes's DMA note adds that the DSP-2's game moves data with the 65C816's
block-move instruction, "which ... like DMA, doesn't use handshaking", so the chip must accept bytes at that rate.

| Code | Name | Computes (argued from the name and the SNES tile format) | From the documents |
|---|---|---|---|
| 01h | Convert bitmap to bitplane tile | packed pixels into the SNES's planar tile layout (fullsnes, PPU tile formats) | Format documented; lengths and order **characterised**; exact expected |
| 03h | Set transparent colour | stores the colour 05h skips | Characterise the width |
| 05h | Replace bitmap using transparent colour | lays one bitmap over another, keeping the lower where the upper is transparent | Characterise lengths; exact expected |
| 06h | Reverse bitmap | mirrors a bitmap | Characterise the axis and length; exact expected |
| 07h | Add | a 16-bit sum | Characterise overflow; exact expected |
| 08h | Subtract | a 16-bit difference | as 07h |
| 09h | Multiply | a product; "bugged", used by the Japanese v1.0 | The bug **undocumented**; exact only if it is a rule |
| 0Dh | Scale bitmap | resizes a bitmap row | Algorithm **characterised** (a stepped resample, argued) |
| 0Fh | Process | does nothing; "dummy NOP command for re-synchronisation" | **Exact** |

> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §42). The commands, their lengths and their timing were
> characterised and graded. Every value is exact within the chip's buffers, and 09h's "bug" is a rule. 00h, 02h, 04h,
> 0Bh and 0Ch, which fullsnes does not name, are a row's bitplanes, several tiles, a one-byte overlay and two
> divisions. 0Eh-3Eh are the test commands. 0Dh's timing is an estimate, and 1Eh is a named loss.

### 3.3 DSP-3 (SD Gundam GX)

fullsnes, "DSP3 Commands": command parsing as on the DSP-1, DR 80h on completion, and these codes:

| Code | Name | From the documents |
|---|---|---|
| 02h | Unknown | **Nothing** |
| 03h | Calculate cell offset | Name only; a board-geometry computation |
| 06h | Set board dimensions | Name only; state for 03h, 07h, 1Eh |
| 07h | Calculate adjacent cell | Name only |
| 18h | Convert bitmap to bitplane | Name; as DSP-2's 01h, presumed |
| 38h | Decode Shannon-Fano bitstream | Name, and "USF1 bit in SR register = direction". The code table is the question: if the game sends it, decoding is a documented algorithm; if it is in the chip's data ROM, R4 stops it |
| 1Eh | Calculate path of least travel | Name only; a path search whose tie-breaking is the program's |
| 3Eh | Set start cell | Name only |
| 0Fh, 1Fh, 2Fh | Memory test, data ROM transfer, ROM version (0300h) | 2Fh exact; 1Fh not replaceable, as the DSP-1's |

**Assessment (argued).** The DSP-3's commands are named but not described, its game is not in the tester's library
(measured 2026-10-04: no SD Gundam GX in `AppSettings.RomDirectory`), so neither the trace nor the whole-game oracle can
run, and its decoder may depend on a table that cannot be derived. One step (§8, step 13) answers the decoder question
from the command oracle alone: does a decode's output change when the bitstream changes but no table was sent? Until
then the DSP-3 has no replacement, and SD Gundam GX without the player's image runs without its chip.

### 3.4 DSP-4 (Top Gear 3000)

fullsnes, "DSP4 Commands": DR holds FFFFh on completion; "all data transfers are 16-bit"; 13h transfers the data ROM,
14h gives the version (0400h); 15h–1Fh do nothing; 20h–FFh mirror 10h–1Fh; and every other command is "xxh Unknown".
SnesLab adds nothing. No public document found describes the commands outside emulator source, which §1.2 excludes.

**Assessment (argued).** A replacement can come only from black-box characterisation of what Top Gear 3000 sends and
receives. The game is in the library, so the trace oracle (§4.2) can record every command it issues, with its inputs
and outputs and its share of the traffic. Whether the commands are recognisable computations (the track's projection,
by the game's look) or opaque is not knowable before looking. §8, step 12 is a characterisation step with a decision at
its end: go if rules explain most of the traffic (P9), otherwise the DSP-4 joins the DSP-3 without a replacement.

### 3.5 ST010 (F1 ROC II)

fullsnes: the µPD96050 ("a slightly extended NEC uPD77C25 with more ROM and RAM, faster CPU clock"), its RAM
battery-backed and "accessible directly via SNES address bus"; the command list below; and the remark that its
functions "are more or less useless ... the only feature that is really used is the battery-backed on-chip RAM". The
superfamicom wiki's prose: "Commands are executed ... by writing the command to 0x0020 and setting bit7 of 0x0021. Bit7
of 0x0021 will stay set until the Command has completed, at which time output data will be available." The ST010 is
therefore driven through its RAM, not through DR. The addresses below are the wiki page's parameter tables, RAM byte
offsets.

| Code | Name (fullsnes) | Takes (RAM) | Gives (RAM) | From the documents |
|---|---|---|---|---|
| 00h | Set RAM[0010h] = 0000h | — | 0010h | **Exact** from fullsnes |
| 01h | Unknown | 0000h X0, 0002h Y0 | 0000h X1, 0002h Y1, 0004h Quadrant, 0006h Y0, 0010h Theta | The output names alone suggest a vector-to-angle conversion; **characterise** |
| 02h | Sort driver placements | 0024h count, 0040h places[32], 0080h drivers[32] | the two arrays reordered | Name; a sort keyed on places with the drivers permuted alongside. The order of equal keys is the program's and is **characterised** |
| 03h | 2D coordinate scale | 0000h X0, 0002h Y0, 0004h multiplier | 0010h X1, 0014h Y1 (32-bit) | Name; a scaled product, rounding characterised |
| 04h | Unknown | 0000h X, 0002h Y | 0010h distance | Output name only; **characterise** |
| 05h | Simulated driver coordinate calculation | 00C0h–00E0h: limits, a 32-bit position, an angle, a radius, an increment, a maximum radius, a flags word, an unnamed word | (not separated from the inputs in the table) | Name and addresses only; **characterise** |
| 06h | Multiply | 0000h, 0002h | 0010h (32-bit) | Name; product, scale characterised |
| 07h | Raster data calculation | 0000h theta | 00F0h, 0250h, 03B0h, 0510h: four arrays of 176 words | Name; four arrays from one angle, presumably per-line mode 7 values. How the values depend on the line, and whether that dependence is a formula, is the R4 question |
| 08h | 2D coordinate rotation | 0000h X0, 0002h Y0, 0004h theta | 0010h X1, 0012h Y1 | Name; a rotation, the sine family as the DSP-1's |
| 09h–0Fh, 10h–FFh | mirrors of 01h–07h and of 00h–0Fh (fullsnes) | | | Exact as mirrors; the oracle checks them |

> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §37.2, §37.5). **The ST010 serves its mailbox only after the
> S-CPU has read the word it writes to DR at power-on**; without that read no command completes. fullsnes's
> "RAM[0010h]" for 00h is the chip's word 0010h, the mailbox, which every command clears on completion: 00h changes
> nothing else. The busy flag is polled every 3 cycles. F1 ROC II uses all seven computing commands, 02h-08h.
>
> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §54). 01h is the angle of the vector (X, Y), measured as
> atan2(-X, -Y) to a 256th of a turn from a 32-by-32 table after normalisation. 05h is one driver's step toward a
> waypoint: the bearing by 01h's routine, a turn of 0280h, a speed rule, a step along the heading, and a gate that
> passes to the next waypoint. Words 60h-70h are its waypoint, position, heading, speed, acceleration, wanted speed,
> gate and next waypoint. Both are built, exact but at two entries of the angle table.

The battery file is the chip's RAM, read and written as now (`VenusRT_Native.md` §33.1). The replacement must write
exactly the RAM the program writes, work areas included, or a save written under one engine differs from one written
under the other. The oracle compares the whole RAM after every command (§4.1).

### 3.6 ST011 (Hayazashi Nidan Morita Shogi)

fullsnes, "ST011 Commands (japanese chess engine)": commands 01h–07h, 09h, 0Bh, 0Ch, 0Eh and 0Fh exist with function
"?", F1h and F2h are self-tests, F3h a data-ROM dump that "doesn't work due to wrong loop address", and the rest do
nothing. The superfamicom wiki's page reads "N/A". Nothing public describes the protocol: how the board is passed, how
a move is encoded, what each command returns.

**Assessment (argued).** The chip is the game's opponent. A replacement would have to be a shogi engine speaking an
undocumented protocol. Black-box characterisation could recover the protocol from traces of the game, but the game is
not in the tester's library (measured 2026-10-04). Even then the result could never be exact: the opponent's choices are
the program's, and a different engine plays a different game. A protocol-compatible engine would make the game
playable, which has some value, but it is a research project of its own with no oracle for its play.
**Recommended:** the ST011 has no replacement. It is the one named exception to `EmuSen_Firmware.md` §0 in VenusRT, its
firmware entry states so (§7.2), and its game runs only with the player's image. §10, Q2 asks for the decision.

---

## 4. The oracle

VenusRT's LLE runs the real programs, and `VenusRT_Native.md` §24.2 found its DSP games following Mesen where they reach
the chip. The oracle takes the LLE as the reference and grades the replacement against it at three levels: commands,
game traces and whole games. Where the LLE itself is wrong (its open items, §27.3 there: three late partings from
Mesen, the clock rate argued in D-32), the replacement inherits the error, and that is the LLE's dispute, not this
page's.

### 4.1 The command oracle

A port driver in the crate's tests speaks to a chip only through `host_read`, `host_write` and the clock, as the S-CPU
does. Two chips are built side by side: the LLE from a dump, and the replacement. For each case the driver sends the
same command and inputs to both and records, on each side:

- every byte read from DR, in order;
- SR's high byte at each step (RQM, DRS, DRC, USF1, USF0);
- for every DR access, the chip cycles until RQM rises again, counted exactly on the LLE (one instruction a cycle,
  fullsnes; D-32);
- DR after the last result (80h, FFFFh);
- on the ST01x, the whole 4 KiB RAM image after the command and after its busy bit clears.

A case passes when every record is equal. **The inputs:**

- **Edge cases** per format: 0, ±1, 7FFFh, 8000h, FFFFh, the powers of two and their neighbours, angles at each
  quadrant's edges and one step either side, the exponent range's ends for Inverse, vectors whose squares overflow 32
  bits.
- **Seeded random** inputs, 2^20 cases per command by default, from a PCG generator whose seed is in the report.
- **Exhaustive** where the space allows: every angle for Triangle and Rotate at fixed radii, every mantissa for Inverse
  at each exponent class, every line for Raster under a set of projections, and every pair for Multiply, 2^32 cases.
  The LLE runs about 1.3–2.1 × 10^8 instructions a second (`VenusRT_Native.md` §24.4: 127,000 a frame for 0.6–1.0 ms),
  and a Multiply case is a few dozen, so the 2^32 pairs take minutes on eight threads (P4).
- **Sequences** for the commands that share state: Attitude then Objective, Subjective and Scalar; Parameter then
  Raster, Project and Target; Gyrate after Attitude; DSP-2's 03h then 05h.
- **Abuse:** a read past the end of the results, a new command written mid-parameters, a parameter supplied by a read
  instead of a write (§2.1), a reset mid-command, all 256 command bytes with zero inputs.

**Two LLE-against-LLE runs come first** (§8, step 1): the DSP-1 against the DSP-1B, which maps where the 1B's fixes
lie (28h by fullsnes, and any others), and each dump against itself from two different power-on phases, which shows
that the driver's results do not depend on when it starts.

### 4.2 The trace oracle

The LLE records, during a game, every command transaction: the frame, the master clock, the command byte, the inputs,
the results and the latency of each transfer. A run of each bench game gives a trace; the replacement replays each
trace's inputs offline and is compared transaction by transaction. This separates a wrong computation from a timing
difference, weights errors by how often a game meets them, and gives each game's **command histogram**: which commands
it uses at all, and so what a replacement must cover for it. fullsnes's remark on the ST010 (§3.5) and the untested
commands of `VenusRT_Native.md` §24.2 make the histogram the first scope measurement, not an afterthought.

### 4.3 The whole-game oracle

A lockstep runner (an example binary beside `chip_activity`) builds two machines from the same ROM, one with the dump
and one without, and runs them frame by frame with the same scripted input. After each frame it compares the picture's
hash, WRAM, VRAM, CGRAM, OAM, the APU's RAM, the S-CPU's master-clock position, and the whole state with the
`Coprocessor` group left out. It reports the first frame and space that differ, the share of frames whose picture is
equal, and whether the pictures meet again after parting.

**The bench**, from the tester's library (measured 2026-10-04): Pilotwings (DSP-1, whose plane demo shows 28h's bug),
Super Mario Kart, Ballz 3D (DSP-1B), Michael Andretti's Indy Car Challenge, Lock On, Super Bases Loaded 2, Suzuka 8
Hours (DSP-1), Dungeon Master (DSP-2), Top Gear 3000 (DSP-4) and F1 ROC II (ST010). `VenusRT_Native.md` §24.2 found
that Dungeon Master, Lock On, Super Bases Loaded 2 and F1 ROC II do not reach their chips without input, so each gets a
pad script in the runner's `tapuntil`/`pressuntil` form that reaches chip use, written once and kept with the other
scripts. Runs pass `--nobattery`, since a battery file makes runs non-reproducible.

> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §37.6). Super Bases Loaded 2 and F1 ROC II reach their chips
> without input. Dungeon Master, Lock On, Michael Andretti's Indy Car Challenge, Suzuka 8 Hours and Top Gear 3000 need
> it, and their scripts are in the crate's `examples/dsp_pads/`.

**The goal** for a chip whose commands and latencies are all exact is identical states for 3,600 frames on every bench
game it has (P6). Anything less is recorded as the first frame of parting and its cause.

### 4.4 The harness, and nothing of the firmware in it

- **Where the dumps come from.** The crate's tests read the folder named by `EMUSEN_VENUSRT_FIRMWARE`. When it is unset,
  or a chip's image is absent or the wrong size, each oracle test prints "not run" and passes, as the corpus tests do
  with `EMUSEN_VENUSRT_CORPUS`. WiseMan's tests point `FirmwareLibrary.Directory` at the same folder and return early
  without an image, as `NecDspRealFirmwareTests` does. The tester's dumps are in `~/.cache/emusen/probe/venusrt/firmware/`
  as program and data pairs for all seven chips (measured 2026-10-04: names and sizes only, 6,144 + 2,048 bytes for each
  DSP-n and 49,152 + 4,096 for each ST01x).
- **What a committed test holds.** Inputs (edge lists, seeds, sequences) and the comparison. Expected values come from
  running the LLE at test time. A test that runs without a dump grades the replacement only against the documents: 2Fh
  answers 0101h, Multiply of 4000h by 4000h is 2000h by the formula, a DSP-2 tile converts as fullsnes's tile format
  says. A golden of the replacement's own outputs may pin it against regressions, recorded from the replacement, never
  from the LLE.
- **What never enters the repository:** a dump, a byte of one, a table read from one, the LLE's outputs in bulk (which
  could reconstruct tables), traces of commercial games. All of these go to `~/.cache/emusen/probe/venusrt/dsp-hle/`.
  Step records cite counts, shares and at most a handful of individual cases per rule (R1).

### 4.5 What the oracle's results look like

One report per run, in the probe cache: per command, the cases run, passed, and the first ten mismatches with their
inputs and both outputs; per latency, the measured distribution and the model's error; per trace, transactions
matched; per game, the first parting. The step records in `VenusRT_Native.md` carry the counts.

---

## 5. Accuracy

### 5.1 What the documents fix, and what must be derived

| Kind | Commands | What makes it exact |
|---|---|---|
| **Fixed by a document** | DSP-1 2Fh; DSP-2 0Fh and the mirrors; ST010 00h; DSP-3 2Fh; DSP-4 14h | Nothing further |
| **A formula plus a rounding rule** | DSP-1 00h, 08h, 18h; DSP-2 07h, 08h; ST010 03h, 06h | The rule chosen from a family of at most four (truncate, round half up, round half to even, round towards zero) and widths, by the oracle |
| **A formula plus a derived table** | DSP-1 04h, 0Ch, 1Ch, the attitude commands; ST010 08h | A table generated from a formula (§5.2); the family is the table's length, rounding, saturation and interpolation |
| **A formula plus an undocumented method** | DSP-1 10h (reciprocal), 28h (root) | A method family: a seed table generated from a formula (2^k/x, 2^k/√x over n entries), iteration counts, rounding at each step. Exactness is uncertain (P1) |
| **Undocumented: characterise** | DSP-1 20h, 38h, 14h, 02h, nAh, 06h, 0Eh, 0Fh, the DSP-1's 28h bug; DSP-2 01h, 03h, 05h, 06h, 09h, 0Dh; ST010 01h, 02h, 04h, 05h, 07h; all of DSP-3 and DSP-4 | A hypothesis stated as a formula with its family (R2), then graded |
| **Not replaceable** | DSP-1 and DSP-3 1Fh, DSP-4 13h, ST011 F3h (data-ROM dumps); any command R4 stops; all of ST011 | Named losses |

### 5.2 Derived tables

A derived table is generated, never written out. The rules:

1. **The formula is written in the step's record before the first comparison**, with its family: for a sine,
   `round_mode(2^15 · sin(2π·i/N))` saturated to 7FFFh, for N in a stated set, with or without linear interpolation
   between entries, and the angle's index taken from its top log2(N) bits.
2. **The core generates the table from that formula** at start-up or in `build.rs`, by code. The source never holds the
   table as a literal array.
3. **A second, independent generator** checks the first: a short Python script in
   `EmuSen.WiseMan/Reference/analysis/`, written from the record's formula and not from the Rust, writes the same table
   to a file; a crate test asserts the two are equal word for word.
4. **When the oracle rejects every member of the family**, the family may be widened once, in writing, with the reason.
   A table that no stated formula reproduces is R4's stop. A table that a formula reproduces except at a few entries is
   also a stop: patching those entries would copy them.

> *Amended 2026-10-04* (decided; `VenusRT_Native.md` §40.6). Rule 4 gains one exception. Where firmwarecheck's
> `--forced` run proves a table formula-exact, one extra widening is allowed, limited to the arithmetic between the
> table's entries, declared in writing first and at most 64 members, with no new table and no patched entry. The
> DSP-1's sine took it, and no member was exact (§40.7).

### 5.3 Measuring that no table repeats the firmware beyond what a formula forces

`firmwarecheck.py` (on branch `venusrt`; `EmuSen_Debugging_Tools_Reference_v5.md` §3.61) compares a replacement image
with an original and prints counts and offsets only. For an executable replacement such as D-38's boot program, its
thresholds (at most 25% equal at the same offset, no common run over 6 bytes) are the test. For a derived table they are
the wrong test: a correct sine table *must* equal the program's sine table, if the program's is that formula, so a long
common run is the expected outcome of success, not evidence of copying. What has to be shown is that every common run is
**forced**, explained by the formula and not by anything else. The method:

1. **The replacement's data image.** An example binary serialises every table the replacement generates, in its own
   order, as 16-bit little-endian words (the "newer" format of fullsnes's "ROM-Images"), to
   `~/.cache/emusen/probe/venusrt/dsp-hle/<chip>.tables.bin`.
2. **The formula image.** The independent generator of §5.2 writes the same tables, from the formulas alone, to
   `<chip>.formula.bin`. By §5.2's step 3 the two files are byte-identical, so the residual of step 4 can be zero only
   if nothing beyond the formulas was added.
3. **The plain measurement.** `firmwarecheck.py <chip>.tables.bin <chip>.data.rom` (the data half of the dump, or the
   whole image with `--window 6144:8192` for a DSP-n, `49152:53248` for an ST01x) records the share equal at the same
   offset (expected near zero, since the replacement's order is its own) and every common run with its offsets. These
   counts go into the step's record whatever they are.
4. **The forced-run measurement**, a new option, `--forced <formula image>`: the tool also finds every common run
   between the formula image and the original, and reports the replacement's common runs with the original that are
   **not** covered by a formula run at the same original offsets. The verdict is PASS when that residual has no run over
   `--max-run` bytes. It prints, too, the **formula coverage**: the share of the original's data words that some
   formula run reproduces. That share is informational (it says how much of the program's data is a formula's output)
   and is recorded per chip (P11).
5. **The program half is not compared.** A replacement has no µPD77C25 code, so there is nothing to set against the
   program ROM. Its Rust source is checked against emulators' sources instead (§5.4).

The option belongs in `firmwarecheck.py` itself, beside its unit test's rule that no byte of either image is printed. It
is a change to a tool on branch `venusrt`, made in the step that first needs it (§8, step 4), and its own test uses
synthetic images: an original holding a synthetic sine table among noise, a replacement holding the same table, and a
replacement holding the table plus a copied noise run, which must fail.

### 5.4 Two mechanical guards on the source

- **The clone check** of `VenusRT_Plan.md` §7, G8 compares VenusRT's sources with Mesen's. For this work its comparison
  set gains the DSP code most likely to contaminate a writer: bsnes/higan's `dsp1`–`dsp4` and ST010 modules, snes9x's
  `dsp1.cpp`–`dsp4.cpp` and `st010.cpp`, MAME's DSP files, and the code on the superfamicom wiki's ST010 page (§1.4).
  The tool reads them; no writer does. A table that appears in both, or a run of fingerprints above the calibrated
  threshold (`VenusRT_Native.md` §4.2), is a finding against the protocol.
- **A table-literal guard**: a crate test fails if a replacement's source holds an array literal of more than 16
  numeric entries. Every table must come from a generator (§5.2).

### 5.5 The expected accuracy cost, and how the core states it

| Chip | Expected (predicted) | Stated in core info as (§7.2) |
|---|---|---|
| DSP-1B | The general, vector and attitude commands exact; Inverse and Distance exact or within one in the last bit; the projection commands approximate at first | `effect: accuracy` with a cost naming the inexact commands, until every command and latency is exact and the whole-game gate holds; then `exact` |
| DSP-1 | As the DSP-1B, plus 28h's bug only if it is a rule | as DSP-1B; the cost names 28h until it is |
| DSP-2 | Every command exact | `exact` after the whole-game gate on Dungeon Master |
| DSP-3 | No replacement | `effect: none`: the game runs without its chip |
| DSP-4 | None until the go decision; approximate after | `none`, then `accuracy` |
| ST010 | The commands F1 ROC II uses exact; others approximate or unimplemented | `accuracy` until every command is exact, the cost naming the rest |
| ST011 | No replacement | `none` |

`EmuSen_Games_Tested.md` records per game what the replacement costs in play, and the core info's `accuracy.notes`
points at §5.5 of this page.

---

## 6. Timing

### 6.1 What the documents give

SnesLab gives each DSP-1 command's speed in cycles and microseconds, from the manual; the SNESdev wiki splits eight of
them into a command phase, a phase per input and a phase per output (§3.1's table). The microseconds put the clock at
7.6 MHz (Gyrate: 444 cycles, 58.4 µs), the oscillator fullsnes lists for the DSP-n. Where the two pages can be added up
they agree for six commands and disagree for two: Inverse (98 against 97) and Triangle (59 against 49); and SnesLab
notes the manual's own summary page gives Rotate 65 against 64. Nothing documents the DSP-2, DSP-3, DSP-4 or ST010
latencies.

### 6.2 The latency model

For a command with inputs 1..n and results 1..m, the replacement keeps one number per phase, in chip cycles:

- `t_cmd`, from the S-CPU's completion of the command byte until RQM rises for the first input;
- `t_in[k]`, from the completion of input k until RQM rises for the next transfer; the last of these includes the
  computation;
- `t_out[k]`, from the completion of result k until RQM rises again.

These are the SNESdev wiki's columns, so where that table exists it is the model's first prediction (P3). Between
phases SR shows the DRC, DRS and USF bits the LLE shows; the oracle records them per phase and the replacement replays
them. On the ST010 the same model applies to bit 7 of RAM byte 0021h: it clears `t` cycles after the write that set it.

> *Amended 2026-10-04* (measured, `VenusRT_Native.md` §37.3). **This model does not hold.** The latency from the
> S-CPU's completion depends on when the S-CPU answered, because the chip goes on computing while RQM is high. Each
> phase takes two numbers: its *work*, cycles from the previous rise, and its *notice*, cycles from the S-CPU's
> completion, and RQM rises at the later of the two. Measured from two runs per case, the model predicted every
> modelled run of a jittered S-CPU exactly (1,008 of 1,008 on the DSP-1B, none mispredicted on any chip). Some
> transfers of the DSP-2, DSP-3, DSP-4 and ST011 are not handshaken, so their values depend on the S-CPU's answer
> time. A replacement reproduces both. The ST010's busy bit clears at the first 3-cycle poll after the command's work.

### 6.3 Measuring it against the LLE

The command oracle (§4.1) records every phase's latency on the LLE for every case. For each command:

- if a phase's latency is the same over every case, that constant is the model;
- if it varies, the step looks for a rule in a stated family: a function of one feature of the inputs, such as a
  normalisation shift, an iteration count or a sign, as R2 requires of values. The rule is graded on all cases;
- if no rule in the family fits, the median is used, and the model's largest error in cycles is recorded with the
  command. Such a command cannot pass the whole-game gate exactly.

The latencies are in chip cycles and converted to master clocks by the LLE's own ratio, so a revision of D-32's clock
rate moves both engines together.

### 6.4 What the games need

- **Polling.** A game that polls RQM spins until the result is ready. A replacement that answered at once would let
  the S-CPU run ahead: rarely visible in play, but the whole-game oracle would part at the first poll, and a game that
  counts its polls or races its chip against NMI could behave differently. With exact latencies the S-CPU's polling
  loops run the same number of times on both engines, which is what makes §4.3's identical-state goal reachable.
- **Unhandshaken transfers.** The DSP-2's game moves blocks without polling (fullsnes, "DMA"). The replacement must
  accept a byte on every access during such a phase with the LLE's own acceptance timing. If the LLE would miss a byte
  that arrives while RQM is low, so must the replacement, or the game's data differs.
- **ST010's busy bit** is a RAM byte the game polls through the battery window, under the same rule.

### 6.5 Speed, a side effect

The LLE costs 0.6 to 1.0 ms a frame (`VenusRT_Native.md` §24.4), most of it spent polling RQM. A replacement does no work
while idle and is predicted to cost under 0.05 ms a frame (P10). Under the policy the image is used whenever it is
present, so this speed is not offered as a choice. §10, Q5 asks whether a hidden setting forcing the replacement should
exist, for the oracle and for comparison.

---

## 7. Selection

### 7.1 The rule

When the image for the cartridge's chip is present, whole or as its split pair, and of the right size, VenusRT runs the
LLE. Otherwise it runs the replacement, and the game starts with no question asked. The core decides at `create` from
whether file 2 was given (`EmuSen_CoreAPI.md` §6.2). The runtime keeps finding the files as it does now
(`FirmwareLibrary`, `TryLoadParts`, `VenusRT_Native.md` §34).

### 7.2 What `firmware_for` and the descriptors say

Today `firmware_for` answers file 2 with `required: true` (`v1.rs` on `venusrt`), and core info lists an open `dsp.rom`
entry. Both become `required: false`, and each entry gains a `replacement` object. That field is an addition to the
firmware entry of `EmuSen_CoreAPI.md` §6.2, which §4.2 there allows in a minor version: an older host ignores it, and an
older core omits it, which reads as "no replacement".

```json
{ "which": 2, "name": "dsp1b.rom", "label": "DSP-1B program and data", "size": 8192, "required": false,
  "parts": [["dsp1b.rom"], ["dsp1b.program.rom", "dsp1b.data.rom"]],
  "replacement": { "effect": "accuracy",
                   "cost": "Without the image, the projection commands (Parameter, Raster, Project, Target) are approximate.",
                   "record": "VenusRT_DspHle.md §5.5" } }
```

`effect` takes the settings schema's words (`EmuSen_CoreAPI.md` §6.13): `exact` when every command and latency
passes the oracle and the whole-game gate; `accuracy` with a required `cost` in plain words; and `none` when there is no
replacement (DSP-3, ST011 and, before its decision, DSP-4), whose `cost` says the game will run without its chip. Machine
info gains `firmware`, a list of `{ "which": 2, "source": "file" | "replacement" | "absent" }`, so a frontend can say
which path this game runs on.

The `emusen-native` crate's `Firmware` struct and the C# `CoreFirmware` record each gain the field, the descriptor
schema gains it, and the ABI baseline check (`EmuSen_CoreAPI.md` §5.4) is updated in the same commit. The conformance
kit gains one case: a core whose entries are all `required: false` creates every image of its kit set with no files.

### 7.3 The frontends

- **Mistress's prompt must not fire.** Today `CoreEngine` turns `required` into the request's `Purpose` text, and
  `PromptForMissingFirmwareAsync` opens a picker for every missing request, required or not. The change is in two
  places: `FirmwareRequest` carries `Required` as a field and not as text, and `EmulatorSession.MissingFirmwareFor`
  returns only required requests. VenusRT then asks for nothing, and the picker never opens for it. A WiseMan test pins
  it: VenusRT, a DSP-1 cartridge built by `SyntheticRom`, an empty firmware folder; no request is missing, the game is
  created, its state carries the replacement's tag, and machine info says `replacement`.
- **The notice.** The status bar shows one line when a game runs on a replacement whose `effect` is not `exact`: the
  chip, "VenusRT's open replacement", and the cost. With `none` it says the game runs without its chip and names the
  file that would supply it, without opening a picker.
- **The firmware window** lists the entries from core info as optional, each with its replacement's effect.
- **Venus (C#)** has no replacement, and its requests stay required until the decision of §10, Q6.
- **Pharaoh and Hotaru** keep the log path of `EmuSen_Firmware.md` §3, which becomes a notice rather than a complaint.

### 7.4 What the policy leaves unchanged

Size remains the only validation (`EmuSen_Firmware.md` §2.1): an image of the wrong length is absent, and the
replacement runs. The `dsp1.rom` the tester's folder holds, whose opening is not a DSP program (`VenusRT_Native.md`
§24.1), is therefore taken as an image and runs as garbage (argued). That is the LLE's existing behaviour with a bad dump, and the
fix belongs to the firmware library (a checksum field, room for which §2.1 there already notes), not to this page.

---

## 8. Steps

Supervised steps of about three hours, with a check-in after each, as `VenusRT_Plan.md` §6. Each step ends with its
record in `VenusRT_Native.md` (what was built, what the oracle measured, the family declared and chosen for each
command, `firmwarecheck` counts for each table), and any rule settled against a document goes in
`VenusRT_Disputes.md`. The estimates are predictions (P12).

| Step | What it covers | Its oracle | Steps |
|---|---|---|---|
| 1 | The port driver and the command oracle in the crate, over the LLE alone: DSP-1 against DSP-1B, each dump against itself from two phases, the 256-byte command sweep for every chip, latencies recorded per phase; the trace recorder; the command histogram of every bench game, with the pad scripts that reach the chips | The LLE against itself (must be identical); the histogram | 2 |
| 2 | The replacement's frame: the engine slot, the state tag and version, selection at `create`, `required: false` and the `replacement` field across the crate, the C# record, the schema and the baseline; the machine-info `firmware` list; Mistress's prompt filtered, the notice, the WiseMan test; the table-literal guard | Conformance kit; the WiseMan test; the existing DSP tests unchanged with an image | 2 |
| 3 | DSP-1: Multiply (both forms), Radius, Range (both forms), the ROM version, the memory test, and the latency model with them | Command oracle, exhaustive for 00h | 1 |
| 4 | DSP-1: the sine family, Triangle, Rotate, Polar; the `--forced` option of `firmwarecheck.py` with its test | Exhaustive angles; `firmwarecheck` plain and forced | 1 |
| 5 | DSP-1: Attitude, Objective, Subjective, Scalar | Sequences | 1 |
| 6 | DSP-1: Inverse and Distance, their method families, and the DSP-1's 28h bug | Exhaustive per exponent class; DSP-1 against DSP-1B map from step 1 | 2 |
| 7 | DSP-1: Gyrate, Parameter, Raster, Project, Target | Sequences; traces of Pilotwings and Super Mario Kart | 3 |
| 8 | DSP-1 whole games: the seven DSP-1 and DSP-1B titles in lockstep, their partings attributed | Whole-game oracle, 3,600 frames each | 1 |
| 9 | DSP-2: every command, the unhandshaken transfer timing | Command oracle; Dungeon Master trace | 1 |
| 10 | DSP-2 whole game | Dungeon Master in lockstep | 1 |
| 11 | ST010: the mailbox protocol, the commands F1 ROC II's histogram shows, the RAM compared whole, the battery file across engines; then the rest of the commands as far as characterisation reaches | Command oracle; trace; F1 ROC II in lockstep; `.srm` written under one engine read by the other | 2 |
| 12 | DSP-4 characterisation: Top Gear 3000's trace read command by command, hypotheses stated and graded; **a decision at the end**, go or no-go (P9) | Trace oracle | 2 |
| 12a | DSP-4 build, if go | Command and trace oracles; Top Gear 3000 in lockstep | 4–6 |
| 13 | DSP-3: the decoder question (does 38h depend on a table the game sends), from the command oracle alone | Command oracle | 1 |
| 14 | Close: the clone check with the DSP comparison set, `firmwarecheck` counts for every table collected, core info's effects and costs set from the measurements, `EmuSen_Games_Tested.md`, `VenusRT_Plan.md` §9 Q2 marked superseded | §5.4, §5.5 | 1 |

**About 21 steps, 60–70 hours**, without 12a; 25–27 steps with it. Steps 3 to 8 depend on 1 and 2; 9–10, 11 and 12 depend
only on 1 and 2 and can run in any order after them. The DSP-1 is first because it covers about nineteen games. The
order 1, 2, 3, 4, 9, 10 would get the player a working DSP-2 and the DSP-1's simpler commands earliest, if that is
preferred.

**Coordination.** The VenusRT crate is being worked on on branch `venusrt`. Step 1 adds tests and an example only; step
2 touches `v1.rs`, `cart.rs`, the state and the frontends, and is to be scheduled with that work, not beside it.

---

## 9. Risks

- **The exactness ceiling of the iterative commands** (argued). Inverse, Distance and the projection chain depend on
  methods no document describes. If no small family reproduces them, the DSP-1's replacement stays `accuracy` for its
  most-used commands in Super Mario Kart and Pilotwings. The cost of that is a picture that differs by a pixel at a
  time, not a broken game, and the whole-game oracle measures it.
- **R4 stops more than predicted.** If the DSP-1's sine is not a formula, everything built on it is approximate. P2
  says which way that goes.
- **Characterisation that does not converge.** DSP-4, DSP-3 and parts of the DSP-1 and ST010 rest on hypotheses from
  traces. Step 12's decision exists so that this risk costs two steps, not eight.
- **Exposure.** The superfamicom ST010 page (§1.4) shows how easily emulator code reaches a writer through a page that
  looks like documentation. Mitigations: §1.2's table, the clone check's DSP set, the table-literal guard, and pages
  checked for code before they are read in full.
- **Hidden DSP-1 and DSP-1B differences** beyond 28h. Step 1's map measures them before any command is built.
- **Data-dependent latency** without a rule makes a command's whole-game exactness impossible even when its values are
  exact.
- **The LLE's own open items** (`VenusRT_Native.md` §27.3) are inherited. Andretti, Ballz 3D and Top Gear 3000 part from
  Mesen late on the LLE; a replacement that matches the LLE matches that parting.
- **Churn in the descriptors.** §7.2's field is an ABI-visible addition and needs the baseline, the schema and both
  languages updated together. Done out of step with branch `venusrt`, it conflicts.
- **No game for DSP-3 or ST011.** Neither chip's game is in the library, so no claim about either can be checked in play.

---

## 10. Questions to decide

1. **Q1, does a partly built replacement ship?** A chip whose replacement covers some commands would run its games
   until they meet an unbuilt one. Recommended: yes, behind `effect: accuracy` with the unbuilt commands named in its
   cost, because a game that runs most of the way is better than one that does not run, and the notice says so.
2. **Q2, is the ST011 a named exception to `EmuSen_Firmware.md` §0?** Recommended: yes, with `effect: none`, and the
   protocol-compatible engine of §3.6 left as a possible later project of its own.
3. **Q3, is the DSP-3 parked until step 13 answers its decoder question?** Recommended: yes. Acquiring SD Gundam GX for
   the library would let its trace and whole-game oracles run; whether to is the tester's call.
4. **Q4, is the black-box characterisation of §1.3 within the clean-room protocol?** Recommended: yes, under R1–R4,
   on the same standing as Mesen run as a black box.
5. **Q5, a hidden setting that forces the replacement even when an image is present?** It would serve the oracle's
   whole-game runs and players' comparisons. Recommended: a hidden, create-scope setting, `effect: accuracy`, its
   accurate default "use the image when present", since the schema requires an accuracy setting's default to be the
   accurate value (`EmuSen_CoreAPI.md` §6.13).
6. **Q6, what does Venus (C#) do with a DSP game and no image?** It has no replacement. Recommended: once VenusRT's
   DSP-1 replacement passes step 8, Mistress drops the picker for every engine (the policy says no frontend prompts) and
   the notice for Venus (C#) names VenusRT as the engine that can run the game without the file.
7. **Q7, may a later dispute step about the LLE read the SNES_MiSTer RTL's DSP files?** The RTL is an interpreter of
   the same programs and holds none of them, so reading it cannot import a command's function. Recommended: yes, for
   the LLE's disputes only (the clock rate, D-32), under `VenusRT_Plan.md` §1.5, and never for a replacement.

---

## 11. Predictions to be retired

- **P1.** Of the DSP-1B's named commands, counting each family once (17), at least 11 agree with the LLE bit for bit
  over the full random and edge sets by the end of step 7. Inverse and Distance each reach exactness or a largest error
  of one in the last bit. *Retired 2026-10-04, false* (`VenusRT_Native.md` §51.4): seven codes are exact, the sine's
  loss making every command built on it approximate, and Inverse and Distance stay within 2 and 4.
- **P2.** The DSP-1's sine values are reproduced exactly by one formula from a family of at most 8 bits of choice, or
  by none; no formula matches all but a few entries.
- **P3.** Every DSP-1 command's measured latency, phase by phase, is a constant or a function of one input feature, and
  for at least six of the eight commands the SNESdev wiki splits, the measured phases equal its columns within 2 cycles.
  *Retired 2026-10-04, false in its second half* (`VenusRT_Native.md` §37.7): only Multiply, Radius and Range are
  within 2 cycles in every phase; the first half is open.
- **P4.** Exhaustive Multiply, 2^32 pairs, runs through the LLE in under 30 minutes on eight threads of the desktop.
- **P5.** No bench game issues a data-ROM transfer (DSP-1 1Fh, DSP-4 13h) after boot, so the named loss costs nothing in
  play.
- **P6.** With every command it uses exact, Super Mario Kart and Pilotwings run 3,600 frames on the replacement with
  states identical to the LLE's. *Retired 2026-10-04, false* (`VenusRT_Native.md` §44.3, §51.2): no command built on
  the sine can be exact, and both games part in the frame after their first command that computes.
- **P7.** F1 ROC II issues at most three distinct ST010 commands in its first race. *Retired 2026-10-04, false*
  (`VenusRT_Native.md` §37.6): it issues seven, 02h-08h, in its attract mode.
- **P8.** Every DSP-2 command is exact by the end of step 9. *Retired 2026-10-04, false in part* (`VenusRT_Native.md`
  §42.3): every value is exact within the chip's buffers and Dungeon Master runs identically, but 0Dh's timing is an
  estimate and counts past the buffers are not reproduced.
- **P9.** The DSP-4 characterisation reaches a go: rules explaining at least 80% of Top Gear 3000's traced command
  volume after step 12's two steps. Low confidence.
- **P10.** A replacement costs under 0.05 ms a frame on the desktop, against the LLE's 0.6–1.0.
- **P11.** The forced-run residual of §5.3 is zero for every derived table, and the formula coverage of the DSP-1's
  data ROM is under 40%.
- **P12.** The whole is about 21 steps without 12a; the projection commands (step 7) and the DSP-4 decision are the
  likeliest to overrun.
