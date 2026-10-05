# Nephrite_Disputes — the disputes log

*Created 2026-10-05, at stage 3, with its first entry.* This is the log `Nephrite_Plan.md` §1.4 defines, in the shape
`VenusRT_Disputes.md` gives. An entry records a rule the documents and the test programs leave open or in
disagreement, the evidence taken, and the rule as implemented. It is the record of which sources a dispute step read,
so that the clean-room protocol of `Nephrite_Plan.md` §1.3 can be audited from outside. The Beryl crates keep their
CPUs' disputes in their own record pages.

## The rules this log keeps

- **A dispute step is separate.** It ends with a rule stated in hardware terms, and a writer's step implements it and
  cites the entry.
- **The order of recourse is fixed:** the documents, then a test program written for the purpose, then the MiSTer RTL,
  run as a black box where it can be. No emulator source is ever read.
- **Every RTL file opened is listed** with the lines read and what for. Nothing from it is transcribed, here or in
  the code.
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, section> for each
- Test program: <the program written or run, and what it showed>
- Referee: <RTL files and lines read, and how it was run>
- Conclusion: <the rule as implemented, in prose>; argued | measured | open
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the commit that cites this entry>
```

## Entries

### D-1. The Z80's window onto the 68000's bus: the waits each access costs the two processors, and BUSREQ's latency
- Opened: 2026-10-05, at stage 3 step 2. Stage 3 step 1 (`Nephrite_Native.md` §9.2) took three T-states for the Z80
  and three 68000 clocks for the 68000 per window access, and BUSREQ granted at the 68000's next instruction, as
  provisional values; the plan's §2.3 lists the Z80's waits as thin.
- Documents read: Charles MacDonald, "Sega Genesis hardware notes" §2.2-§2.3 (BUSREQ, RESET and the bank; no timing);
  Sega, Genesis Technical Overview 1.00, §4 "Z80 Control" and §5 "Z80 Area" (pp. 77-78) and "68K control of Z80"
  (p. 91: the procedure, and the Z80's interrupt "generated 16ms period and 64ms length", read as 64 µs); Sega, Genesis
  Technical Bulletins, "Precaution when accessing the Z80 bus from a 68000 main" and "Reading the controller pads" (the
  wait of a Z80 access to `$A100xx`, "250ns to 110ns", without the window's own). None gives the window's waits or
  BUSREQ's latency.
- Test program: none in the corpus measures them (searched 2026-10-05: the console test programs of the plan's §3.1,
  the r57shell and SpritesMind folders).
- Referee: Nuked-MD's whole board, `md_board`, compiled with Verilator 5.046 and run as a black box by a bench of its
  pins (`EmuSen.WiseMan/Reference/rtl68k/tb_md.cpp`; programs and readings by `mdboard.py`; the FPGA memory block its
  VRAM names replaced by a behavioural one written from the instance's port list, `vram_ip.v`). The programs, written
  for the purpose and hand-assembled: the 68000 loads a Z80 loop that reads banked cartridge ROM (or, as the control,
  its own RAM) and then counts in a loop in its own RAM, so that the cartridge's pins show only the Z80's window reads
  and the RAM's pins the 68000's rounds; a second program requests the bus, polls until it is granted and releases
  it, marking each in RAM. **Progress:** the board builds (17 s) and boots the program after a power-on reset of its
  own of about 1,400,000 MCLK2 cycles; the 68000 runs the loader and copies the Z80's program. **Not yet measured:**
  every word the 68000 copies to its RAM arrives as `$FFFF` though the cartridge's pins drove `$41F9`, so the RAM loop
  never runs. A defect of the bench's model of the cartridge or the RAM is suspected (a ROM's hold time after OE was
  added, without effect) and is the next step's first task.
  Files read in this step: `rtl/nuked-md/md_board.v`, the port list (lines 28-137), the instance names (`fc1004 ym`,
  `m68kcpu m68k`, `z80cpu z80`, `vram vram1`, `vram vram2`), the pin assignments of the RAMs (lines 678-685) and the
  data and address buses' multiplexing (lines 725-800, 820-835), to know how the pins are driven; `rtl/nuked-md/vram.v`,
  the `vram_ip` instance (lines 46-57); `rtl/ram_md.v`, the `vram_ip` module's port list (lines 1-20). No other RTL
  file was opened; the 68000's and the Z80's own were run, not read, as in `Beryl_M68k.md` §6.
  **Stage 3 step 3 (2026-10-05): the `$FFFF`s were the bench's.** A trace of the pins (`TB_TRACE`) showed `cart_cs`
  and `cart_oe` low through the 68000's write of its own RAM at `$FF0000`: the board leaves both low on cycles that
  are not the cartridge's, and the bench's cartridge drove the data bus whenever they were low, so the RAM took the
  ROM's `$FF`s. The bench's cartridge now answers only its own range, the first 4 MiB, as a cartridge decodes it
  from the address lines; with that the copy arrives and the RAM loop runs. Two more facts of the pins, read from the
  port list and confirmed on the trace: the RAM's address lines are ordered `{ VA14, IA14, VA12-VA0 }`, so `$FF1000`
  shows as `$5000` on them; and a write to the cartridge's range shows as `cart_lwr` or `cart_uwr` high (positive
  logic) with `cart_cs` high. **The measurements**, the 68000 counting in its RAM while the Z80 reads (or writes) the
  cartridge through the window, over 1,275 to 1,875 of the Z80's rounds (`mdboard.py window`, `window-write`):
  the 68000's round, `move.l d0,(a0)`, `addq.l`, `bra.s`, 30 clocks by the manual, takes 30.75 without the window;
  the Z80's, `ld a,(nn)` (or `ld (nn),a`) and `jr`, 25 T-states, takes 27.75 with the window, so **an access waits
  2.75 T-states** on average, reads and writes alike; the Z80's rounds, seen on the cartridge's pins, fall on the
  68000's clock (multiples of 14 MCLK2: 798 to 882), the wait depending on where the access meets the 68000's bus
  cycle. **The 68000 loses 9.5 of its clocks an access** (133 MCLK2), from its rounds beside the accesses against the
  control's. **BUSREQ** (`busreq`, `busreq-tight`): a request written and the grant polled at once found the bus
  granted at the first poll in all 667 requests with the Z80 reading the window and all 739 with it in its own RAM,
  the grant arriving within about 16 of the 68000's clocks of the request's write; that is Nephrite's rule, the
  grant at the next instruction boundary. No RTL file was opened in this step; the board was run as before.
- Conclusion: ~~open. Nephrite keeps the provisional values of `Nephrite_Native.md` §9.2.~~ **Measured** (stage 3
  step 3): a window access costs the Z80 41 master clocks (2.75 T-states, its mean) and the 68000 66 (9.5 of its
  clocks, 66.5, rounded down), paid at the 68000's next access; BUSREQ is granted at the 68000's next instruction
  boundary. The alignment the wait depends on is not modelled; the means are.
- Pinned by: `genesis.rs`'s `WINDOW_Z80_WAIT` and `WINDOW_68K_STALL`, and the corpus's anchors
  (`Nephrite_Native.md` §11.1); no test program reaches the window's timing.
- Implemented in: the commit "Nephrite, stage 3 step 3".

### D-2. OPEN. The 68000 loses clocks to the main RAM's refresh, on the cartridge's bus as well as its own RAM
- Opened: 2026-10-05, at stage 3 step 3, by Sonic the Hedgehog 2 running about 12 frames ahead of Genesis Plus GX
  (`Nephrite_Native.md` §11.3): the lead starts in its boot program's checksum of the ROM, a loop of the 68000 alone
  over the cartridge, which Nephrite times by the 68000's manual.
- Documents read: the MegaDrive Wiki's "Mega Drive" article (`railgun/Mega_Drive.wiki`, "Hardware"): the main RAM
  is "refreshed every 128 68k clocks - if accessing it during refresh, it will be delayed one 68k clock". It says
  nothing of the cartridge.
- Test program: none in the corpus measures it.
- Referee: Nuked-MD's board, run as a black box (`mdboard.py rom-loop`): `add.w (a0)+,d1`, `cmp.l a0,d0`, `bcc.s`,
  24 clocks by the manual, with the Z80 held in reset. Summing the cartridge, 2,341 rounds take 24.38 clocks on
  average: 1,895 take 24 and 446 take 26, one loss of two clocks every 128.0 clocks. Summing the 68000's RAM, 2,288
  rounds take 24.94: 1,532 take 24, 298 take 26, 370 take 27 and 88 take 29. No RTL file was opened.
- Conclusion: open. The cartridge's figure agrees with the wiki's period and not with its one clock; the RAM's needs
  a model of where in the 68000's bus cycle the refresh falls, which these loops do not separate. Nephrite takes no
  refresh yet.
- Pinned by: nothing yet.
- Implemented in: not yet.
