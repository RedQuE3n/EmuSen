# Sapphire (Sony PlayStation 2)

The pages of Sapphire, EmuSen's PlayStation 2 core, planned 2026-10-23, its stages S1 to S6 built 2026-10-24 and S7 to S9, S11, S12, S21, S13, S16, S23, S22, S17, S20, S14, S18, S15, S26, S27, S24, S19, S30 and S10 on 2026-10-25. The core will live
in `EmuSen/Cores/Sony/Sapphire - PlayStation 2/`; the name is fixed by `EmuSen_Core_Naming_Scheme.md` §4.

- **`Sapphire_Plan.md`** — the stage 0 plan (2026-10-23): the clean-room protocol and the exposure rule carried over from
  Diamond; every unit with its source and oracle (the EE and its MMI, COP0, the FPU, VU0 and VU1, VIF, GIF, the GS,
  IPU, DMAC, INTC, timers, SIF, the IOP, SPU2, CDVD, SIO2) and what is left out; PS1 discs routed to Diamond; the
  player's BIOS first and the open replacement as a separate track; the speed budget and the recompilers; the GS's
  exact software reference and its Vulkan path; the shared crates; the stages, gates, questions and predictions;
  since 2026-10-24 the gates G10 (start without a disc) and G11 (the Memory Cards window's PlayStation 2 tab).
- **`Sapphire_Native.md`** — the build record, from stage S1 (2026-10-24): the stub crate on the core ABI v1, marked in
  development (an EE program by its ELF header, a disc by `SYSTEM.CNF`'s `BOOT2`, the three BIOS entries, the `SPH2`
  state); the system `ps2` with its recogniser and the PlayStation's `BOOT2` exclusion, root files read as far as a
  DVD's volume places them; the CRT row owed while the console is offered to no player; the fixtures, the kit's gate
  and the seeded faults; and stage S2 (2026-10-24): the references pinned, the Japanese BIOS entry renamed to
  `ps2-0230j-20080220.bin`, and each BIOS entry listing its market's retail MD5s from Redump's datfile; and stage S3 (2026-10-24): the referees installed and driven headless, the gate's
  program read from each, and the probe naming the hardware contexts it refuses; and stage S4 (2026-10-24): the
  runners (the synthetic program and discs, dual-layer DVDs among them, the expected-text grader, the RAM-record reader
  shared in form with Diamond's) and ps2autotests' output channel found by observation, the IOP's console; and stage
  S5 (2026-10-24): the R5900's and the vector units' instruction tables in `emusen-mips`, read against the TX79 manual's
  opcode tables and ps2tek's, with what the documents do not give; and stage S6 (2026-10-24): the EE core's integer
  interpreter decoding through `emusen-mips`, its timing model on the EE Core User's Manual's tables, the project's
  programs over every integer encoding against the manual and on both referee lineages, and an ELF layout defect found
  and fixed; and stage S7 (2026-10-25): COP0 in full, the TLB and the scratchpad, both caches and CACHE, the physical
  memory map and its bus errors, Count and Compare, the performance counters and the breakpoints, graded by the
  project's programs against the manuals, with what each referee models and does not; and stage S8 (2026-10-25): the
  multimedia instructions, PLZCW and QFSRV from the EE Core Instruction Set Manual's chapter 3, their timing on both
  multipliers, the project's programs over every encoding equal on the interpreter and on PCSX2, and ps2autotests'
  `ee_simd` console texts reproduced by PCSX2; and stage S9 (2026-10-25): COP1 on `sapphire::float`, the PlayStation 2's
  single-precision format in software, its last bit decided by ps2autotests' `ee_fpu` console texts (900 of 901 lines),
  the project's programs over every input class, and neither referee a witness for the FPU's arithmetic; and stage S11
  (2026-10-25, built before S10 by the tester's decision): the DMAC's ten channels with their chains, interleave,
  stall control and MFIFO, the INTC, the four EE timers and a blanking stand-in on one clock, the core run beside them,
  the later peripherals' channels ending at a defined interface, and PCSX2 reproducing the stage's seven console texts;
  and stage S12 (2026-10-25): VU0 in macro mode, the vector units' state and operations on `sapphire::float` with the
  VU's own flags, COP2's transfers and interlocks, the macro timing, micro mode's interface for S13, and neither referee
  a witness for the VU's arithmetic;
  and stage S21 (2026-10-25, §22): the IOP on `blackmoon-r3000`, unchanged, with Sapphire's own IOP bus, its map,
  I-cache, scratchpad, wait states, INTC, 13-channel DMAC and six timers, PRId 1Fh, a unit of the board, and an IOP
  program placed on PCSX2 through an EE program and the IOP's exception vector; and stage S16 (2026-10-25, §24): the
  GS's registers, CSR, IMR and its interrupt, its local memory in every format's page and block layout with one owner,
  the three transfers, the interface the GIF will drive, PCSX2 reproducing every format's layout at a page's start and
  `gs/label`'s and `gs/signal`'s console texts, and Play!'s local-to-host transfer confirming the referees' pictures as
  RAM; and stage S13 (2026-10-25, §23): micro mode for both units, the pipelines with
  the flags', Q's and P's visible delays, the branches, the E, M, D and T bits, XGKICK at an interface, VU1's EFU from
  the manual's nano code graded by the console's text, and PCSX2 a witness for the pipeline's visibility; and stage
  S23 (2026-10-25, §25): the CDVD drive controller at its register interface, its N and S commands, seeks and reads
  timed, the tray, the clock and the EEPROM, on `emusen-disc`'s new DVDs with their layer break, the drive's answers
  where ps2tek is silent read from PCSX2's lineage, and P13's mapped open;
  and stage S22 (2026-10-25, §26): the SIF, its mailbox registers on both sides and a FIFO of 128 bytes for each of
  SIF0, SIF1 and SIF2 joining the two DMACs' channels, the stand-ins replaced, paired EE and IOP programs passing
  messages both ways, and PCSX2 a witness for the registers and the tags but not for the FIFOs' depth or SIF2;
  and stage S17 (2026-10-25, §27): the GS's drawing kick, every primitive with flat and Gouraud shading, exact
  coverage and interpolation, scissoring, SCANMSK, FBMSK and the depth test in every format, the pixel pipeline's order
  with the places of texturing and the other pixel operations, PCSX2 a witness for coverage and depth but not for
  interpolation's last step, and Play! no witness for drawing;
  and stage S20 (2026-10-25, §29): the PCRTC, its video timing on its own clock from the GS manual's table in place of
  S11's stand-in (D-42 closed: PCSX2 raises H-blank at twice the line rate), the two read circuits and the merge, each
  raster read at its line's time, the picture for the frontend, and PCEE2's own pictures equal to Sapphire's where it
  shows a circuit at its own size;
  and stage S14 (2026-10-25, §28): VIF0 and VIF1 on the DMAC's channels, every VIFcode, UNPACK in every format, mask,
  mode and write cycle, VIF1's double buffer and its reversed FIFO, the MSCAL family starting micro mode, PATH2 at an
  interface the GIF will implement, four of the five `dma/vif` console texts reproduced on the VIF, and PCSX2 a witness
  for the data path but not the stalls;
  and stage S18 (2026-10-25, §31): texture mapping in every format, the CLUT buffer as the GS's own state under CLD,
  LOD, mipmaps and their filters, the wrap modes, TEXA and fog, texture coordinates and the LOD exact, PCSX2 a witness
  for addressing, the CLUT, the tables and fog's formula but not for the filters' and the LOD's arithmetic;
  and stage S19 (2026-10-25, §37): the alpha test with AFAIL, the destination alpha test, blending with FIX and PABE,
  dithering, COLCLAMP, FBA, the masks after them and antialiasing as far as the documents define it, PCSX2 equal on the
  tests, the blend and the masks; local-to-local overlap decided in TRXPOS's order; the BIOS intro's white bands and
  square light trails found to be blending missing and fixed;
  and stage S25, part A (2026-10-25, §30): the card's authentication researched and left unanswered (Q7(b)); SIO2 with
  its queue, FIFOs, interrupt and DMA channels; the DualShock 2's modes, configuration and motors; the memory card's
  pages, spare bytes, erase and its flash's times; the multitap's frame; `emusen-ps2-card` for the card's file system;
  the medium `ps2.card` declared; PCSX2 a witness for most of the pad and the card's data, not for the reply's first
  bytes, the erase's alignment or the multitap's selection (D-220 to D-230);
  and stage S15 (2026-10-25, §33): the GIF, its three paths (XGKICK, DIRECT and DIRECTHL, channel 2 and the EE's
  stores through a FIFO of 16 qwords), the PACKED, REGLIST and IMAGE formats with one owner, the arbitration at packet
  ends, PATH3's masks and the intermittent mode, GIF_CTRL's reset and pause and GIF_STAT, S16's, S17's and S18's programs run
  whole through PATH3 and VIF1, PCSX2 a witness for the formats and the masks and Play! for a held PATH2, and S14's
  second DIRECT found a PCSX2 defect;
  and stage S26 (2026-10-25, §32): the SPU2, its voice moved first into `blackmoon-spu` with Diamond's goldens unchanged,
  two cores on the IOP's bus and DMA channels 4 and 7, the mix, effects areas, AutoDMA, core 0 into core 1 and the IRQ
  addresses, at 48,000 Hz exactly, its sound graded through the output areas PCSX2 also writes and its captured output;
  and stage S27 (2026-10-25, §34): the IPU, its ten commands, registers, FIFOs and DMA channels, MPEG-2 and MPEG-1
  macroblocks decoded as H.262 defines it with Annex A's reference IDCT decided exactly, §8.6's colour conversion, the
  project's streams decoded equal to a model of the standard that ffmpeg's decoding matches, PCSX2 a witness for the
  commands and the stream's handling and Play! for VDEC, their departures disputes D-300 to D-326; and stage S24 (2026-10-25, §35): the machine on the board with the player's BIOS, the IOP's
  reset line, the drive on the IOP's bus, the EEPROM as the console medium `ps2.nvm`, the picture and the sound through
  `v1`, the hardware's own reports as the only window on the boot, OSDSYS's language screen reached on both 0230 images
  and compared with PCSX2's, the pad stepping the initial setup on, and D-340 to D-351; and stage S10 (2026-10-25,
  §36): an EE program entered by fast boot through the kernel's ExecPS2, the IOP's console caught at IOMAN's `write`
  found through the export tables and kept in the `tty` space, 52 of ps2autotests' 55 EE programs equal to their console
  texts with S10's gate met, D-43 settled by `triace`, the IOP DMAC's deadlock of the SIF's RPC found and fixed, S24's
  field order settled by a field-marking program, the 23 IOP programs left to the tester's decision, and D-360 to D-364.
  And stage S30 (2026-10-25, §38): `SPH2` version 2 with every unit's state in the plan's order, the picture first so
  that a load knows its length, the queues in fixed slots, the BIOS entry and the image named and never held, the cards'
  and the EEPROM's contents kept apart, the listing pinned against its file; loading on a running console, mid-frame;
  states equal at five frames of the OSDSYS boot on both 0230 images and at four instants of 30 of the project's programs;
  P14 measured, a capture 6.5 ms and a restore 4.3 ms, the default rewind budget holding seconds;
  and stage S25, part B (2026-10-25, §30.15-§30.24): the start without a disc (G10), the console with its tray closed and
  empty through the core ABI's empty start and `MEDIA`, refused by the host in the firmware entry's words with no BIOS;
  the Memory Cards tab (G11) with `ps2.card` read by `emusen-ps2-card` in the kilobytes the browser shows; the player's
  BIOS on PCSX2, driven through its keyboard pad, formatting a card that the crate edited and the BIOS read back, and
  the reverse (D-380 to D-382); OSDSYS on Sapphire stepping through its setup to the main menu with an empty tray, and its
  browser never drawing with a card in slot 1 (P8 as observed, D-383 open); D-383's step (2026-10-26, §30.25-§30.30):
  a command no device acknowledges ends at its peripheral byte, which made the setup take the pad with slot 1 empty,
  and the browser then stalls the same with slot 1 empty as with a card (D-440 and D-441 opened); and the 23 IOP
  programs (2026-10-26, §40): the project's EE loader speaking
  LOADFILE's SIF RPC, its layout learned on PCSX2 (D-480), each module on a synthetic CD put in through `MEDIA` after an
  empty start, all 23 equal to their console texts on both 0230 images, the drive's interrupt (D-481) and the IOP's
  region past 8 MiB (D-482) corrected, a DVD's file system left open (D-483).
- **`Sapphire_References.md`** — the references (2026-10-24, stage S2): the documents and ps2autotests fetched and
  pinned with their sources, licences and SHA-256s; ps2tek's BIOS chapter screened, with its exclusions for the open
  BIOS's writers by section and line, and two passages of emulators' code excluded for everyone; the BIOS database
  and its lists; the IOP's cache facts and the SPU2's 48 voices.
- **`Sapphire_Referees.md`** — the referees (2026-10-24, stage S3): PCSX2 2.8.2 headless under a private KWin with
  its software renderer and PINE, Play! 0.72 with its RAM found in its own process, LRPS2, PCEE2 and `play_libretro`
  through the probe with each hardware-render refusal measured; their versions, pins, licences, firmware, costs, RAM and
  lineages; the gate's program `record.s` read equal from every referee that runs it; the writer's exposure; a hardware
  context for the probe evaluated and not built; DobieStation evaluated.
- **`Sapphire_Disputes.md`** — the disputes log, opened with D-0, the system clocks; D-1 to D-5 since S5, where the TX79
  manual and ps2tek differ over the R5900's map and where the documents leave the vector units' layout open, all five
  documented by Sony's manuals at S6; D-6 to D-11 since S6, the integer interpreter's readings against the referees; D-12 to D-19 since S7, where the EE
  Core User's Manual contradicts itself on COP0 and where the referees answered against it; D-20 to D-23 since S8, where
  the multimedia pages contradict themselves or leave results undefined and the console's record decides; D-24 to D-33
  since S9, SQRT.S's operand, the FPU's last bit, its zeros, flags and control registers; D-34 to D-42 since S11, the
  DMAC's stall control, TADR after an end, TTE, undefined tags, register widths and priority control, the timers'
  ZRET and gates, the interrupt lines' levels and the blanking stand-in, with D-15 closed; D-43 to D-54 since S12, the
  VU float's sharing of the FPU's readings, the VU's flags, VU0's control registers, the divider's flags, MAX, the random
  unit's series, CLIP, VU0's data addresses, Q's delay, FBRST's reset, the empty field mask and the M bit; D-60 to D-68 since S21, the
  IOP's PRId, map, cache, access times, timers' modes, clocks and gates, its interrupt line and its DMAC's registers and
  readings; D-100 to D-107 since S16, the GS's privileged reads, CSR's revision and ID, the registers' images, the
  interrupt line, an odd PSMT8 width, a depth format based inside a page, the transfers' wrap and an overlapping copy; D-80 to D-85 since S13, a branch on an integer
  just written, the I bit's own pair, TPC's unit, the divider's flags in micro mode, the EFU and CMSAR1's start; D-120
  to D-128 since S23, the CDVD's seeks, ReadDvd's block and its b29, dual-layer seeks, the version bytes, the S
  commands' answers, CD-DA's command, the layer break in an image and the registers at rest; D-140 to D-145 since
  S22, the SIF's FIFOs, its registers and the EE's CTRL interrupt, SIF0's and SIF1's tags, the IOP's IRQ tag and SIF2,
  with D-68 amended;
  D-160 to D-165 since S17, Gouraud colour's and Z's precision, line ends on a diamond's edge, XYOFFSET at the drawing
  kick, Z beyond its format's range and a line's colour past its ends;
  D-180 to D-194 since S14, UNPACK's indeterminate fields, WL 0, MODE 3, the filling write's data, the double buffer,
  NUM's width, VPS and FQC, RST, undefined VIFcodes, the i bit, FBK and STP, MARK's and ITOP's widths, TTE and the
  VIF's timing; D-240 to D-250 since S18, DECAL's alpha with TCC RGB, the LOD's boundary and arithmetic, the CLUT's
  load and addressing, REGION_CLAMP's order, MTBA, texture coordinates' precision, fog's formula, bilinear rounding and
  the blend between levels;
  D-400 to D-406 since S19, AFAIL, DATE in PSMCT24, the blend's arithmetic, dithering's formats, COLCLAMP after a dither,
  PABE and FBA, and antialiasing's coverage, with D-107 closed;
  D-280 to D-291 since S15, a packet holding the GIF, descriptors 0Ch and 0Dh, GIF_STAT's fields, TTE on channel 2, RST
  and MSKPATH3, the intermittent mode's slices, the GIF's timing, BUSDIR, PATH1's wrap, the masks at the arbitration,
  PCSX2's second DIRECT and Q; D-260 to D-270 since S26, the SPU2's key on, output
  areas, mix, transfers, effects area, IRQ addresses, read-backs, AutoDMA, a voice's start, the voice's rules where PCSX2
  departs, and the output's level; D-300 to D-326 since S27, the IPU's IDCT, intra RAW16's range, the input buffer's
  filling, the alternate scan's weights, IDEC's scale, start codes, VDEC's errors and lengths, MPEG-1's arithmetic,
  CBP, the interrupt, IPU_CMD's and IPU_TOP's quirks, BCLR, FB, commands while busy, RST, IPU_CTRL's fields, IDP 3,
  IDEC's increment, CSC, the dither and VQ, SGN, the escape's odd levels, errors inside a macroblock, the decoding's
  time, TTE on toIPU, and MPEG-1's stuffing and D-pictures; D-340 to D-351 since S24, what the player's BIOS showed the board
  lacked: the IOP's reset line, D_ENABLE and DMACINTEN at power-on, the memory controller's RDRAM answers, SIF0's and
  SIF1's chains and padding, the absent ROMs' place, halfword DMA writes, the boot S commands, STATX, PATH3's zero tag
  and PATH1's estimate; D-360 to D-364 since S10, what the console texts showed: the IOP DMAC's flags under DICR's
  master enable clear and its start time, BC1T in a BC1T's delay slot, `intstall`'s FQC (open) and the console's TTY.

The shared Sony hardware Sapphire uses (the IOP's R3000A, the SPU2's voices) is in `../BlackMoon-HW/`; the shared crates it extends
(`emusen-mips`, `emusen-disc`, `emusen-gpu`) are in `../../Shared/`. See `Man pages/Hardware/README.md` for how this
documentation set is organised.
