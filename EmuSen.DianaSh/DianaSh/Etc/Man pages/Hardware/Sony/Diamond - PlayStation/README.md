# Diamond (Sony PlayStation)

The PlayStation core, written in Rust from hardware documents under a clean-room protocol, on the core ABI v1 from
its first commit (`EmuSen/Cores/Sony/Diamond - PlayStation/`, crate `diamond`), and marked in development until its
gate. At stage 1 (2026-10-11) it recognises a PS-EXE or a disc and shows a black picture; at stage 2 (the same day)
its R3000A was built and passes the r3000 single-step suite; at stage 3 the bus, its timing and the devices beside
the CPU run it, on the open kernel or a player's own BIOS, and ps1-tests' CPU, DMA and timer programs pass by their
TTY; at stage 4 the GPU draws, ps1-tests' GPU programs' VRAM equals their pictures, and the display is the picture;
at stage 5 the GTE computes, ps1-tests' `test-all` passes every case bit for bit, and coprocessor 2's accesses wait for
its commands; at stage 7 the CD-ROM controller reads discs (`.cue` with its tracks, `.chd`, `.iso`, a lone `.bin`)
through the shared crate `emusen-disc`, and games boot to their title screens through the kernel's own loader; at
stage 9 the MDEC decodes, ps1-tests' MDEC programs' VRAM equals their pictures but for the rounding D-30 leaves open,
and a game's opening movie plays; at stage 8, built after it, the SPU plays, its voices as its capture buffers record
them equal to DuckStation's sample for sample, and ps1-tests' SPU programs pass; at stage 10 the open kernel boots
discs by their SYSTEM.CNF and keeps files on memory cards in their on-card format, with no firmware folder. Its processors are the BlackMoon crates (`../BlackMoon-HW/README.md`).

- [`Diamond_Plan.md`](Diamond_Plan.md) — the plan, written 2026-10-10 before any emulation: the machine in scope and
  how accurate each part must be (the R3000A with its cache and scratchpad, the GTE exact, DMA, timers, interrupts, a
  cycle-faithful software GPU, the SPU, the CD-ROM controller with CD-DA and XA, the MDEC, pads, cards and the
  multitap, PAL and NTSC), the disc formats, who may read what, the sources and where they are thin, the oracles by
  component, the black-box references and the FPGA referee, what a frontend gets, the open BIOS, the speed budget, the
  enhancements decided on 2026-10-10 (precise geometry and the rest) and their separable architecture, the stages with
  their gates, the shared MIPS and disc crates, and the questions for the tester.
- [`Diamond_Native.md`](Diamond_Native.md) — the build record, from stage 1 (2026-10-11): the stub crate on the core
  ABI v1 (what an image is, read from its bytes; the provisional rates; the settings, firmware, spaces and state), the
  system pack `psx` and which facts it owns, the BlackMoon crates, the registration in development, WiseMan's PS-EXE
  and synthetic disc builders (ISO 9660, raw sectors with ECMA-130's EDC and parity, a CD-DA track, a cue sheet), and
  the gate: the kit's C1–C15 on a synthetic program and disc, and the r3000 replay controls; and stage 2 (§10): the
  R3000A and the shared `emusen-mips`, 55,000 of 55,000 single-step cases with one dispute naming cases, and P1
  retired; and stage 3's machine (§11): the map, every access timed by memory control and the console's figures, the
  I-cache, the interrupts, the root counters, the seven DMA channels, the video timing on D-0's clocks, the stubs,
  PS-EXE loading, the kernel's seam, the TTY reader, the project's own programs with the references as witnesses, and
  the gate's results; and stage 4 (§12): the GPU in the machine, the frame ending at vblank, the gate, amidog's tests
  and stage 2's items, the cost; and stage 5 (§13): the GTE in the machine and its interlock, the gate program by
  program, how the first build failed `test-all`, the timing against the witnesses, the state's version 2, the cost; and stage 6 (§14): SIO0's devices, the state's version 4, the gate; and stage 7 (§15): the CD-ROM, the disc crate, the
state's version 6, the gate item by item; and stage 9 (§16): the MDEC, DMA channels 0 and 1, the state's version 7,
the gate; and stage 8 (§17): the SPU, the audio path, DMA4, the state's version 8, the gate; and stage 10 (§18): the
open kernel's second phase, the gate with the tester's games booting with no firmware folder and a save loaded back.
- [`Diamond_GPU.md`](Diamond_GPU.md) — the GPU, written 2026-10-11 at stage 4: the FIFO and the command processor,
  GPUSTAT's ready bits against `gpustat`'s console log, DMA channel 2, GP1, VRAM's transfers, texturing with the
  texture and CLUT caches, the pixel pipeline, the rasteriser's rules fitted to ps1-tests' console pictures, the
  display read line by line, the drawing time fitted to `bandwidth` within 0.6% a row and charged to rectangles and
  fills by their rows (D-11's refit), the gate program by program, amidog's and Peter Lemon's programs against the
  references, amidog's GPU test run to its end with its drawing groups passing (D-23), the state, the enhancement
  layer's seam, and the reading log.
- [`Diamond_OpenKernel.md`](Diamond_OpenKernel.md) — the open BIOS kernel's first phase, written 2026-10-11 at stage
  3 under the replacement writers' rules: the image and the RAM it uses, the reset path and the hand-over at 80030000h
  with what a machine must provide (the TTY at the DUART's 1F802023h), the exception entry, the priority chains and the
  hook, the syscalls, the control blocks, heaps, events, threads, the memory, string and file calls with the rule for
  documented bugs, the call table of 576 entries with each one's status, the pinned toolchain and the reproducible
  image, the harness on blackmoon-r3000 and what it proved, the accuracy cost, the choices, the questions and the
  reading log.
- [`Diamond_OpenKernel_Phase2.md`](Diamond_OpenKernel_Phase2.md) — the open kernel's second phase, written 2026-10-11 at
  stage 10 under the same rules, its sections numbered on from the first phase's: the CD driven at its host interface
  with sectors by DMA, ISO 9660 on "cdrom:", the card's sectors on vblank at the original's pace with the events
  psx-spx lists, the on-card file system with formatting, the boot by SYSTEM.CNF and its state at the jump, the warm
  boot, the screen with no disc and its card manager, the tests, the tester's games booting with no firmware folder
  against the player's BIOS, a save made and loaded on the kernel, the choices, the questions, the cost, the image, what
  the cross-kernel step needs, and the reading log with its exposure.
- [`Diamond_Pad.md`](Diamond_Pad.md) — SIO0 and the controllers, from 2026-10-11, when a digital pad on port 1 was
  brought forward from stage 6: the registers, a byte on the wire, /ACK and IRQ7, the pad and the slots, amidog's CPX,
  GPU and GTE tests run from their menus and what they showed (three R3000A defects fixed, the CE and second-load items
  of stage 2 closed); then stage 6: the DualShock, the memory cards as battery files, the multitap, the open kernel's
  pad functions, the gate item by item, and the reading log.
- [`Diamond_CDROM.md`](Diamond_CDROM.md) — the CD-ROM controller, written 2026-10-11 at stage 7: the open replacement
  for the HC05's program at its register interface, the host registers and FIFOs, every command with its responses and
  delays, the drive's spin-up, seeks and held head, reads and the sector buffer, the filter, XA-ADPCM and CD-DA with
  the volume matrix, the lid, the discs a create opens and who owns each of the tester's forms (lone `.bin`, `.zip`),
  the framework's owed items, the gate with ps1-tests' `cdrom/` programs, the games on the player's BIOS against the
  referees, the load time and the copy at create, the state, SCEx and the firmware's date, the cost, and the reading log
  with its exposures.
- [`Diamond_MDEC.md`](Diamond_MDEC.md) — the MDEC, written 2026-10-11 at stage 9: the register interface, the decode
  from psx-spx with the rounding it leaves open searched against ps1-tests' console outputs and chosen (D-30), the
  colour and the 15-bit pixel rounded from the channel's fraction, the pipeline fitted to the step-by-step console log
  (D-32), the decode time by referees (D-31), a read's wait (D-33), DMA channels 0 and 1 with DMA1's macroblock order,
  the state, the gate program by program, Peter Lemon's programs, the tester's games and D-29, the cost, and the reading
  log with its exposures.
- [`Diamond_SPU.md`](Diamond_SPU.md) — the SPU, written 2026-10-11 at stage 8: what it is written from and without,
  the registers, the clock and the output, the voices (SPU-ADPCM decoded a word at a time, the Gaussian table
  transcribed by a script, pitch modulation, the loop flags, key on and off, noise), the envelope and the sweeps, the
  reverb with its accesses' order and its filter, the transfers with the referees' rates and DMA4 letting the other
  channels move, IRQ9, capture and the mix, the state, the gate with the voice programs against the references' audio
  and their capture buffers on five referees, D-29 and the games, the cost, and the reading log with its exposures.
- [`Diamond_References.md`](Diamond_References.md) — the oracles, documents and reference cores pinned at stage 1
  (2026-10-11): the test programs with their console logs and pictures, the documents and their exclusions, the probe on
  the three libretro cores (only PCSX-ReARMed may run without a BIOS; Beetle PSX and SwanStation boot OpenBIOS and are
  refused), their costs and spaces, the referee's simulator, and the search for an AccuracyCoin-like suite.
- [`Diamond_Disputes.md`](Diamond_Disputes.md) — the disputes log, created 2026-10-11 with its first entry, D-0: the
  NTSC console's video clock and line length, and so its frame rate; from stage 3, D-4 (memory control's access
  times), D-5 (the DMA's gaps and a store's drain) and D-6 (timer sync mode 2); from stage 4, D-7 (the rasteriser's
  rules), D-8 (the x-flip's offset), D-9 (the drawing area's commands in order), D-10 (GPUSTAT bit 28), D-11 (the
  drawing time) and D-12 (two pictures the programs cannot meet); from the pad, D-22 (SIO0's timing) and D-23 (amidog's GPU test); from stage 7, D-24 to D-29 (the CD-ROM's response delays,
the drive's mechanics, a seek's reach, XA's arithmetic, getloc's eighth byte, the BIOS's intro); from stage 9, D-30 to
D-33 (the MDEC's arithmetic, its time, its pipeline, a read's wait); from stage 8, D-34 to D-40 (SPU-ADPCM's prediction,
the interpolation's sum, the envelope's step after its sample, the noise's rate change, the transfer's rate, the reverb's
input instants, the reserved shifts and filters). The R3000A's
  disputes (D-1 to D-3, and D-17 to D-21 from amidog's `psxtest_cpx`) are in
  `../BlackMoon-HW/BlackMoon_R3000.md` §8 and the GTE's (D-13 to D-16, from stage 5) in `../BlackMoon-HW/BlackMoon_GTE.md`
  §12. *2026-10-11*: D-20 and D-21 settled; DICR's bit 6 ruled in `Diamond_Records.md` §16.2.
- [`Diamond_Referees.md`](Diamond_Referees.md) — the disputes refereed by emulators, written 2026-10-11 once no
  console was to be used: DuckStation and PCSX-Redux fetched, pinned and driven headless beside the three libretro
  cores, their lineages weighed, each open dispute's program run on all of them, the rulings (D-0's line witnessed and
  its interlaced PAL field changed, D-2's table shape confirmed, D-7's base vertex and D-8's flip start changed, the
  rest still open), the referees scored against the console logs, and the reading log.
- [`Diamond_Records.md`](Diamond_Records.md) — ps1-tests' TTY timing programs rewritten to record in RAM, written
  2026-10-11 so that referees without a TTY can be graded: `timers`, `cpu/access-time`, `dma/chopping`, `gpu/bandwidth`
  and `dma/chain-looping`, the release's own code carried in from the local corpus at build time (never kept in the
  tree) and PSn00bSDK replaced by stand-ins of its observed bus accesses; each record's layout, its equality with the release's TTY on Diamond (and on DuckStation and PCSX-Redux)
  with every difference traced to the kernel's cache footprint or a clock's phase, the records against the console's
  logs and on the five referees, a Diamond defect in timer 0's sync, and the reading log.
- [`Diamond_FPGA_Referees.md`](Diamond_FPGA_Referees.md) — the FPGA cores as black-box referees, written 2026-10-11:
  the access rule decided that day, PSX_MiSTer and Laxer3a's PS-GPU and PS-GTE pinned, GHDL installed user-local, how
  each is driven, the calibration and the disputes each is to referee (not yet run), and the reading log.

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the naming scheme, and `Man pages/Hardware/README.md` for how this
documentation set is organized.
