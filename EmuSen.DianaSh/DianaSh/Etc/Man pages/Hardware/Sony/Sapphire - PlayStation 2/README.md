# Sapphire (Sony PlayStation 2)

The pages of Sapphire, EmuSen's PlayStation 2 core, planned 2026-10-23, its stages S1 to S6 built 2026-10-24 and S7 to S9, S11, S12, S21, S13, S16, S23 and S22 on 2026-10-25. The core will live
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
  messages both ways, and PCSX2 a witness for the registers and the tags but not for the FIFOs' depth or SIF2.
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
  with D-68 amended.

The shared Sony hardware Sapphire uses (the IOP's R3000A) is in `../BlackMoon-HW/`; the shared crates it extends
(`emusen-mips`, `emusen-disc`, `emusen-gpu`) are in `../../Shared/`. See `Man pages/Hardware/README.md` for how this
documentation set is organised.
