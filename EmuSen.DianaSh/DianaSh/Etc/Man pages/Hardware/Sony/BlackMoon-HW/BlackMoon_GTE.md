# BlackMoon_GTE — the GTE's record

*Started 2026-10-11, at Diamond's stage 1; built at stage 5, the same day.* `blackmoon-gte`, the Geometry
Transformation Engine on the R3000A's coprocessor 2. Planned in `Diamond_Plan.md` §5.3, §9.1 and §7's stage 5; the
coprocessor's contract is `README.md` beside this page. A dispute about the GTE is kept here (§12), as the R3000A's are
in `BlackMoon_R3000.md`, since it belongs to every core that uses the crate (`Diamond_Plan.md` §1.4).

The marks are the plan's: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44, Rust 1.98.1) with
the tool named; **[R]** read from a source, named; **[A]** argued, with no measurement behind it; **[C]** a choice.

## 1. Stage 1 (2026-10-11)

The registers, their names, the state block and the signatures of the register transfers and the command, each
`NotBuilt` while `BUILT` was false. Written from psx-spx's "Geometry Transformation Engine (GTE)" chapter, as names and
layout only.

## 2. The interface

As built at stage 5. The crate depends on `emusen-native` (the state) and `emusen-mips` (its table); nothing depends on
the R3000A's crate, nor it on this one.

- **`Gte`**: the engine. **`data(n)`** is MFC2's and SWC2's read of data register *n*, **`set_data(n, v)`** MTC2's and
  LWC2's write, **`control(n)`** CFC2's read and **`set_control(n, v)`** CTC2's write, each as psx-spx says the
  register reads and keeps a value (§3). **`command(word)`** runs the COP2 instruction's low 25 bits and returns the
  cycles until its results can be read; **`command_observed(word, observer)`** is the same with an observer told what
  the command computes (§9). **`register(n)`** is register 0–63 as a read returns it, the data registers first, for a
  debugger's listing; **`registers()`** the 64 words as held, for the state.
- **No clock, no busy state.** The engine writes a command's every result when it is issued; the cycles it returns are
  the caller's to keep. Diamond's bus keeps them as `gte_ready`, the cycle the command in flight is done, and holds the
  processor's next access to coprocessor 2 until then (§6). A core with a different clock, or a test, uses the engine
  unchanged.
- **The processor's side is unchanged.** The R3000A's `Bus` already named coprocessor 2's five operations
  (`cop2_data`, `set_cop2_data`, `cop2_control`, `set_cop2_control`, `cop2_command`) at stage 1, the load delay of MFC2
  and CFC2 is the processor's own, and the processor already issued the command of an instruction an interrupt
  pre-empts (`BlackMoon_R3000.md` §5.4). Stage 5 changed nothing in `blackmoon-r3000`; Diamond's `Board` answers the five
  calls with the engine's and the interlock.
- **`DATA_REGISTERS`, `CONTROL_REGISTERS`**: psx-spx's names, VXY0 to LZCR and RT11RT12 to FLAG.
- **`COPROCESSOR`** (added at Diamond's stage 2): the GTE as coprocessor 2 for `emusen-mips`'s disassembler and
  assembler (`../../Shared/EmuSen_Mips.md` §2.3): the register names above, and the 22 commands of psx-spx's "GTE
  Opcode Summary" selected by the real command number in bits 0–5, each with the 25 bits psx-spx's heading gives it
  (`rtps` is `0180001h`). A command whose other bits differ is written with them (`nclip 0x0140006`, the encoding psx-spx
  reports Wipeout 2097 using). Stage 5's programs needed no further mnemonic: MFC2, CFC2, MTC2, CTC2, LWC2 and SWC2 were
  `emusen-mips`'s from stage 2, and an unlisted number is written as a `.word`.
- **`CYCLES`**, **`divide`**, **`UNR_TABLE`**: the cycle table (§6), the division and its table (§5.4), public so that
  a core or a test can read them rather than copy them.
- **`Observer`**, **`Unobserved`**, **`Projection`**: the enhancement layer's seam (§9).

## 3. The registers

Each write keeps what psx-spx's "GTE Registers" says the register holds, so that most reads are the word itself
(`src/registers.rs`) [R]:

| Registers | A write keeps | A read returns |
|---|---|---|
| VZ0–VZ2, IR0–IR3; RT33, L33, LB3, DQA, ZSF3, ZSF4 | the low 16 bits sign-extended ("returns the 16bit value sign-expanded") | the word |
| OTZ, SZ0–SZ3 | the low 16 bits | the word |
| H | the low 16 bits sign-extended, the documented read bug; RTPS and RTPT use it unsigned | the word |
| SXYP | pushes: SXY1 to SXY0, SXY2 to SXY1, the value to SXY2 | SXY2 |
| IRGB | nothing of its own: IR1–IR3 become each five-bit field times 80h | IR1–IR3 collapsed, each IR divided by 80h and saturated to 0..1Fh |
| ORGB | nothing (read-only) | the same as IRGB ("simply a read-only mirror") |
| LZCS | the word | the word |
| LZCR | nothing (read-only) | LZCS's leading zeros, or its leading ones when negative, 1..32 |
| FLAG | bits 12–30; bit 31 their summary of bits 30–23 and 18–13 | the word |
| every other | the word | the word |

The writes and reads do no saturation and set no FLAG bit ("Writing 32bit values to 16bit GTE registers by software
does not trigger any overflow/saturation flags"). The fifty register cases of `test-all` hold every row (§7.1).

## 4. The state block

`impl State for Gte`: `Data` (32 words), then `Control` (32 words), 256 bytes, the layout pinned by
`the_layout_is_pinned_and_a_block_round_trips`. Diamond's state carries it as `Gte`, and its bus block carries
`GteReady` (8 bytes), the cycle the command in flight is done. Diamond's state version was raised to 2 for it
(`Diamond_Native.md` §13.6).

**A prediction retired.** Stage 1 wrote here that "the command in flight and its remaining cycles join the block at
stage 5". They did not: the engine writes its results when the command is issued, so nothing is ever in flight inside
it, and the cycle it is done is a time, which by the README's contract the bus owns. The block is unchanged from stage
1; the deadline is Diamond's, as the multiplier's is the R3000A's (`hilo_ready`).

## 5. The commands

`src/commands.rs`, every command as arithmetic on `i64` with the widths below, from psx-spx's three command chapters,
"GTE Saturation" and "GTE Division Inaccuracy" [R]. The command word's bit 19 is sf (a shift of 12 or 0), bit 10 lm (IR's
lower bound 0 or −8000h), bits 17–18, 15–16 and 13–14 MVMVA's matrix, vector and translation; every other command
ignores the MVMVA bits, as psx-spx says, and `test-all`'s cases set them at random for every command.

### 5.1 The steps the chapters give

- **A matrix row**: `T*1000h + M1*V1 + M2*V2 + M3*V3`, then `SAR (sf*12)` into MACn, then IRn from MACn saturated to
  `lm ? 0 : −8000h .. 7FFFh`, FLAG 24–22. psx-spx's MVMVA, and its "Details" saying NCx, NCCx and NCDx's light and colour
  steps work as MVMVA with fixed parameters.
- **The colour FIFO**: each MACn divided by 16 and saturated to 0..FFh (FLAG 21–19), CODE from RGBC's top byte, RGB0 and
  RGB1 moved down.
- **MAC0**: its sum, FLAG 16 and 15 past the 32-bit range; the register keeps the low 32 bits.
- **SZ3** pushed through the four-entry FIFO, saturated to 0..FFFFh (FLAG 18); **SX2, SY2** from MAC0 SAR 16 saturated
  to −400h..3FFh (FLAG 14, 13); **IR0** from MAC0 SAR 12 saturated to 0..1000h (FLAG 12); **OTZ** from MAC0 SAR 12
  saturated to 0..FFFFh (FLAG 18).
- **"MAC+(FC−MAC)*IR0"** as psx-spx's "Details": IRn = `((FCn SHL 12) − MACn) SAR (sf*12)` saturated as if lm=0, then
  MACn = `IRn*IR0 + MACn`, on the unshifted products.
- **RTPS/RTPT** as psx-spx's formula: SZ3 from MAC3's sum SAR 12 whatever sf; the division of H by SZ3 (§5.4); SX2 and
  SY2 from `n*IR1 + OFX` and `n*IR2 + OFY`; FLAG 22 from the sum SAR 12 against −8000h..7FFFh whatever lm, while IR3 is
  saturated by lm, as psx-spx's note on the IR3 flag says. RTPT repeats it for V1 and V2.
- **FLAG** is cleared when a command begins and written when it ends.
- The rest as psx-spx writes them: NCLIP, AVSZ3/4, OP (D1–D3 the diagonal RT11, RT22, RT33), SQR, the NCx, NCCx and NCDx
  triples on V0, V1, V2, CC and CDP from IR, DCPL, DPCS and DPCT (DPCT reading RGB0 each time, the FIFO moving under it),
  INTPL, GPF and GPL (GPL's MACs shifted left by sf before the sum).

### 5.2 Where psx-spx is silent, and what `test-all` showed (D-14)

psx-spx gives the flags' meanings ("larger than 43 bits", "larger than 31 bits") but not the accumulators' width, when
the 44-bit check is made, nor which value IR is saturated from. The first build took the plainest reading: one check on
each finished sum, the sum kept at full width, IR saturated from the shifted sum. **It missed 358 of `test-all`'s 1,150
cases** [M, the crate's replay, §7.1]. Four readings, each tried against the table and kept only where it removed
misses, brought it to none:

1. **IR is saturated from the 32-bit MACn register, not from the wider sum.** 358 misses became 143. The misses had
   been IRn at +7FFFh where the console has −8000h and the reverse, with MACn equal: the console's IR saw the sign of
   the 32 bits it kept.
2. **The same for the depth cue's intermediate IR** (FC−MAC): 143 became 20.
3. **The check is made after each addition of a row's sum.** The remaining twenty were FLAG alone, and several (an
   NCS of `test-all` among them, FLAG `A5F80000h` against `81F80000h`) carry **both** MAC2's positive and its negative
   overflow bit from one command. No single value sets both, so the check is made more than once.
4. **The accumulator is 44 bits wide and wraps.** A row starts from `T*1000h`, at most 2^43 in magnitude, which alone
   never sets a flag, and each product is below 2^31. Without wrapping, a running sum that passed +2^43 cannot come back
   below −2^43, a swing of more than 2^44 that three products cannot make. With a 44-bit register that wraps, crossing
   +2^43 lands near −2^43 and a negative product carries it past. The twenty became none.

The argument for the fourth is the mechanism's only consistent reading of the third's observation; the measurement is
the table. Each rule was then removed alone from the finished engine, to show which `test-all` pins [M]:

| The rule removed | Cases missed |
|---|---|
| the 44-bit wrap | 20 (CC, CDP, NCCS, NCCT, NCDS, NCDT, NCS, NCT, RTPS, RTPT) |
| IR from the 32-bit MAC (IR from the wider sum instead) | 239, in twelve commands |
| the check after each addition (on the finished sum only) | 18 |
| the depth cue's difference truncated to 32 bits | 114 |
| the depth cue's difference checked for overflow | 12 |
| RTPT's depth cue on its last vertex only (on every vertex) | 3 |
| RTPS's FLAG 22 from the sum SAR 12 (from IR3's lm-dependent saturation, or from MAC3 against lm=0) | 2 each |
| SX, SY from MAC0's full sum (from its 32 bits) | 69 |
| IR0 from MAC0's full sum | 20 |
| OTZ from MAC0's full sum | 16 |
| MVMVA's far-colour flags (§5.3), all of them / the IR flags / the MAC flags | 5 / 5 / 1 |

**Not pinned** by the table, chosen for consistency with the pinned rules [C]: whether GPL checks its shifted base
before adding the product (both orders pass), the width of MAC0's sum beyond what the table reaches (it is kept at 64
bits), and whether MVMVA's far-colour case saturates its dropped part's IR from 32 bits (both pass; 32 bits, as every
other IR).

### 5.3 MVMVA's far colour and garbage matrix

psx-spx: with the far colour selected, "the return values are reduced to the last two portions of the formula ...
nevertheless, some bits in the FLAG register seem to be adjusted as if the full operation would have been executed", and
the matrix selection 3 gives "`-R*10h, +R*10h, IR0, RT13, RT13, RT13, RT22, RT22, RT22`". Built as written: MACn is
`(Mn2*V2 + Mn3*V3) SAR (sf*12)`; the dropped part `FCn*1000h + Mn1*V1` is checked for MAC overflow and its IR
saturation flagged, and dropped. Which flags "some bits" are is the table's: both the dropped part's MAC and IR flags,
and no others (§5.2's last row). All 50 MVMVA cases pass, the far colour and the garbage matrix among them.
*Revised 2026-10-11 (D-14):* the dropped part's IR saturation is flagged against −8000h..7FFFh whatever lm, as the
depth cue's intermediate IR is (§5.1); `test-all` passes either way, and the referees and amidog's test decide it.

### 5.4 The division

psx-spx's UNR algorithm as written, in `src/divide.rs`: if H < SZ3·2, the leading zeros z of SZ3 as 16 bits, `n = H SHL
z`, `d = SZ3 SHL z`, `u = unr_table[(d−7FC0h) SHR 7] + 101h`, two Newton steps and `min(1FFFFh, (n*d + 8000h) SHR 16)`;
otherwise 1FFFFh and FLAG 17. The table is generated by psx-spx's own formula, `max(0, (40000h/(i+100h)+1)/2−101h)`, and
held equal to its printed rows at the start, the middle and the end by `the_table_is_psx_spxs`; psx-spx's special cases
(`0001h/0001h`, `FE3Fh/7F20h` and `F015h/780Bh` saturating without the flag) are `the_division_saturates_where_psx_spx_says`.

### 5.5 The numbers psx-spx lists as N/A (D-16)

psx-spx lists 42 of the 64 numbers as N/A and says of one: "00h - N/A (modifies similar registers than RTPS...)"; it
also says FLAG's bits "are automatically reset at the begin of a new GTE command". **Every number clears FLAG; 00h runs
as RTPS; the others change nothing else** [A from those two sentences]. Beetle PSX does the same on every case of the
fuzz (§7.3); SwanStation and PCSX-ReARMed leave every register, FLAG included, as it was. `test-all` has no such case.
Pinned by `an_unlisted_number_clears_flag_and_00h_is_rtps`. *Their cycles were revised the same day*: stage 5 charged
00h RTPS's 15 and the others none [C]; since amidog's timing tests were read (§6, D-16) each number takes the cycles
that test's table gives it, 00h 16.

## 6. Timing and the interlock (D-13)

**The cycles** are psx-spx's table: RTPS 15, NCLIP 8, OP 6, DPCS 8, INTPL 8, MVMVA 8, NCDS 19, CDP 13, NCDT 44, NCCS 17,
CC 11, NCS 14, NCT 30, SQR 5, DCPL 8, DPCT 17, AVSZ3 5, AVSZ4 6, RTPT 23, GPF 5, GPL 5, NCCT 39 [R], `CYCLES` by number.
amidog's `psxtest_gte` 1.4 carries a table of every number's cycles, which it grades a console's loops against; its 22
are psx-spx's to the cycle, and its other 42 give the numbers psx-spx lists as N/A their cycles, 00h 16, 02h 13 ... 3Ch
18 [R, the table read from the test's RAM, §7.2]. `CYCLES` holds those 42 since that test was read (D-16).

**The boundary.** A command's cycles count from the bus's clock after its fetch, the same boundary the multiplier's
latency uses (`BlackMoon_R3000.md` §5.5): `gte_ready = now + cycles`. An access at `now ≥ gte_ready` does not wait, so
an access `cycles − 1` instructions after the command, from the I-cache, finds it done [A at stage 5; confirmed since
by amidog's expectations, D-13].

**The wait's last cycle.** An access that finds the command still running resumes at `gte_ready + 1`, one cycle after
the command ends: an access at once after a command therefore costs the command's whole count, and RTPS's MFC2 after N
NOPs waits 15 − N cycles for N up to 13, then none at N = 14, never one [M, amidog's rule, D-13]. Stage 5 resumed it at
`gte_ready` itself, which every timing loop of amidog's test that waits finds one cycle short a pass.

**What waits.** psx-spx: "If an instruction that reads a GTE register or a GTE command is executed before the current
GTE command is finished, the CPU will hold until the instruction has finished." MFC2, CFC2, SWC2 and a command read or
issue, so they wait [R]. psx-spx says nothing of MTC2, CTC2 and LWC2. **They wait too** [C, open]: §12's D-13 gives the
argument and the witnesses.

`programs/gte-timing.s` in Diamond's crate measures it (D-13's test program); Diamond's results are
`the_gte_holds_every_access_until_its_command_is_done`. `programs/gte-loops.s` runs amidog's timing loops for all 64
numbers in the shape its test runs them, graded by its rule in `amidogs_gte_timing_loops_take_the_cycles_its_test_expects`.

## 7. The gate

### 7.1 ps1-tests' `test-all`

**1,150 of 1,150 cases bit for bit** [M], two ways:

- **In the crate** (`src/oracle.rs`, `test_all_every_case_bit_for_bit`): `tests.c`'s table, found through
  `EMUSEN_BLACKMOON_CORPUS` and unrun without it, replayed as the program does it: every register cleared, the 64 inputs
  written in order, the command, the 64 reads compared. It prints its misses by command and register, which is how §5.2
  was found; 0.01 s.
- **Through Diamond** on the open kernel: the program's TTY reads `Passed tests: 1150`, `Failed tests: 0`, `Done.` by
  frame 100, graded by WiseMan's `Ps1Tests` (`gte/test-all`, the console log's two lines expected, at frame 300).

`test-all`'s table is ps1-tests' `gte-fuzz` run on a console with seed `C0FFEEh` and 50 samples a command (its README, and
`gte-fuzz/gte_valid_0xc0ffee_50.log`, whose first case is `test-all`'s first); its console log passes all 1,150, so the
table is the console's.

### 7.2 amidog's GTE test

**Waits on the pad.** Its readme: "Navigate the menu using the D-pad"; it reports on screen, and Diamond has no pad
until SIO0 is built (stage 6, branch `diamond-pad`). Its timing tests (added in its version 1.1) are the oracle D-13
waits for, short of a console. *The pad landed the same day*; the first run is `Diamond_Pad.md` §5.3, and this section
records what the timing tests measure and expect, found on 2026-10-11 [R, the program's code and data disassembled from
RAM while it ran; the packed file cannot be read directly].

- **What a timing row measures.** For each command the test copies a routine with the command's word patched in, once
  for each setting of its sf, lm and MVMVA bits, and calls it twice from the I-cache with interrupts off, once with one
  pass of each loop and then with 501. Ten loops, each timed by timer 2 on the system clock, reset by a write of its
  mode before the loop and read after: the command then CFC2 FLAG after 0, 1, 4 and 7 NOPs; MFC2 MAC1 after 0, 1, 4 and
  7 NOPs, three NOPs after it; the command twice 4 NOPs apart, then CFC2; twice 7 apart, then MFC2. Each pass ends with
  ADDU, BGTZ and the counter's decrement in its slot.
- **What it expects.** Each count must be within 50 of `501 × p − 1`, p a pass's cycles by its rule: the loop's
  instructions plus the command's c, with c read as the NOPs' count n wherever c is n + 1 or less: c + 6 for the CFC2
  loops, c + 8 for the MFC2 loops, 2c + 7 and 2c + 9 for the pairs. So a read n + 1 instructions after a command of
  n + 1 cycles costs nothing (SQR after 4 NOPs, 10 a pass), while one of n + 2 cycles costs two (AVSZ4 after 4, 12). c is
  the command's entry in its table of all 64 numbers (§6); 0Ah and 0Eh add one more cycle to CFC2 at once, which no
  reading of the rest accounts for. Within 50 of 501 passes is a tenth of a cycle a
  pass, so one cycle a pass is an error. The test prints nothing of this unless its verbose flag is set: with the word
  at 8004D474h set to 1 by the harness, it prints each failing loop as `cycles <loop>; got <count>, wanted: <count>`, and
  those lines equal this rule's [M, `gte2` scratch harness, every row of OFFICIAL and UNOFFICIAL TIMING].
- **What it shows of Diamond.** Before D-13's revision every OFFICIAL TIMING row failed, each failing loop one cycle a
  pass short, two for the pairs, and the loops after which nothing waits passing (SQR after 4 NOPs, NCLIP after 7);
  every UNOFFICIAL row failed, the numbers taking no time. After it, OFFICIAL TIMING passes all 22 rows and TOTAL, and
  UNOFFICIAL TIMING 40 of 42 rows, 0Ah and 0Eh failing by the extra cycle no wait accounts for [M]. With D-14's
  far-colour revision MVMVA's OPCODE row passes too, and the whole test, R1 and Start, ends ALL DONE by frame 580,000
  with TOTAL X, F and V OK and T ERROR for those two numbers alone (`Diamond_Pad.md` §5.3.1).
- **The rest of the test.** UNOFFICIAL OPCODE's four groups (MIRROR, WRAPPER, STABLE, UNSTABLE) list no command, so it
  runs nothing; its value column is N/A on the timing rows, so the test checks nothing of what an unlisted number
  computes [M, its menus and screens]. The results screen uses a 6-pixel font, 6 pixels a line, where the menu's is 8;
  the columns X, F, V and T are its characters 9 to 12 (WiseMan's `AmidogScreen.GteCell`).

### 7.3 A fuzz against the witnesses

`gte-fuzz` itself needs the pad (it starts on Start) and prints to the TTY, which the references do not carry
(`Diamond_Native.md` §11.15), so `programs/gte-fuzz.s` is the project's own in its manner: every command 40 times and six
N/A numbers (00h, 02h, 15h, 1Fh, 2Fh, 3Ch) 4 times, **904 cases**, each register written in order with a value drawn as
`gte-fuzz` draws one (one of ten fixed values, a random word, or its low or high half) from xorshift32 seeded
`5EED0001h`, the command's sf, lm and MVMVA bits drawn too, the command run from an uncached slot it is written into, and
every register read back to RAM. The references ran it through the probe on the player's BIOS by `--sysdir` and their RAM
was read at frame 600 [M]:

| Witness | The 880 cases of the 22 commands | The 24 of the N/A numbers |
|---|---|---|
| Beetle PSX | identical | identical |
| SwanStation | identical | differ: nothing changes, FLAG kept; 00h does nothing |
| PCSX-ReARMed | identical | as SwanStation |

The 880 are independent of `test-all`'s table (another generator and seed); three emulators of two lineages agreeing
with Diamond on every bit is weak evidence about the hardware and strong evidence that §5.2's readings are the ones
those emulators also hold. Diamond's own run is `the_fuzz_program_reads_back_what_the_engine_computes`, which replays each
case through the engine alone and holds the processor's transfers equal to it.

### 7.4 Peter Lemon's GTE programs

`CPUTest/GTE/AVSZ` and `CPUTest/GTE/NCLIP` show every row PASS, their pictures equal to the bundled screenshots but for
the screenshots' 224 lines against Diamond's 240 [M]. Stage 4 counted `Cube` among the programs differing in the GTE's
results (`Diamond_Native.md` §12): it uses no GTE instruction (its heading calls it a CPU demo and it computes its 3D on
the CPU), and it waits in its `WaitVSync` for the pad buffer the kernel's pad driver fills, so it waits on the pad too.

## 8. The register hazards (D-15)

psx-spx documents three delays between a write and what a read or a command sees: "After writing to IRGB, the result can
be read from IR3 after TWO nop's, and from IR1,IR2 after THREE nop's (for uncached code, ONE nop would work)"; for
LZCS and LZCR, "one must insert dummy opcodes between writing and reading the registers"; and "GTE (memory-?) load and
store instructions have a delay of 2 instructions, for any GTE commands or operations accessing that register. Any?
That's wrong!". **Diamond applies every write at once** [C, open]: the first is stated in instructions, not cycles, the
second gives no number, the third doubts itself, and no program here can observe the console. `gte-timing.s`'s table D
reads IR3, IR1, LZCR and an RTPS's SZ3 0–3 instructions after the write; Diamond and all three witnesses read the new
value at once. D-15 records it.

## 9. The enhancement layer's seam

`Diamond_Plan.md` §6.2's precision side-channel needs the GTE's untruncated results at each projection. `Observer` has
one method, `projected(&Projection)`, called by RTPS and RTPT for each vertex with MAC0's sums for SX and SY before their
SAR 16 and saturation (16 fraction bits), SZ3 and the SXY2 word pushed. Its default is empty and `command` runs with
`Unobserved`, so the accurate engine compiles to no call; an observer receives copies and can change nothing.
`an_observer_sees_each_projection_and_changes_nothing` runs RTPT both ways and holds the registers equal. Stage E1 adds
what else the layer needs (the CPU's moves of the values) as the README says, as methods with empty defaults.

## 10. Cost

Measured with a throwaway timing loop of 2,000,000 commands each on fixed inputs [M]: 3.2 ns (AVSZ3) to 114 ns (NCDT),
RTPS 17 ns, RTPT 46 ns, 28.9 ns on average over the 22. A frame of a few thousand commands costs about 0.1 ms, inside
`Diamond_Plan.md` §5.12's 0.05–0.25 ms. No command allocates. Diamond's frame cost is §13.7 of `Diamond_Native.md`.
The wait's last cycle (§6) is one comparison on the path every coprocessor 2 access already took, and the cycle table
stays a constant array; Diamond's frame cost across the change is in D-13.

## 11. Tests run (2026-10-11)

`cargo test --release` with `EMUSEN_BLACKMOON_CORPUS`: 9 (the table and the division, the commands' names and cycles, the
registers' quirks, the unlisted numbers, the observer, the state block, the names, and `test-all`'s 1,150 cases).
`cargo clippy --release --all-targets` (Fedora's 1.98.1, the pinned nightly having none) clean.

After amidog's test (D-13, D-14, D-16): 10, the names-and-cycles case now holding psx-spx's 22 and four of amidog's
numbers, the far colour's flag case added, and `test-all` 1,150 of 1,150; Diamond's crate 70 and 19 (`gte-timing.s` as revised,
`gte-loops.s` against amidog's rule, the card round trip); WiseMan's `DiamondTests` 20 with the ps1-tests and amidog
corpora, `Amidogs_gte_timing_tests_pass_but_two_unlisted_numbers` among them (1.5 minutes).

## 12. The disputes log

In the shape of `Diamond_Disputes.md`. Numbered in Diamond's series: D-13 follows D-12.

### D-13. The GTE's interlock: every access to coprocessor 2 waits for the command in flight, its cycles counted from the bus's clock after the command's fetch, and an access that waits resumes one cycle after the command ends

- **Opened:** 2026-10-11, at stage 5, by psx-spx naming reads and commands only, and by the boundary being unstated.
- **Documents read:** psx-spx `6d7d1bc`, "GTE Load Delay Slots" (the sentence quoted in §6) and "GTE Command Summary"
  (the cycles); nothing else read names a GTE stall.
- **Test program:** `programs/gte-timing.s` in Diamond's crate: by timer 2 on the system clock from the I-cache, each
  timing run twice and its second pass kept, the GTE drained before each; table A, each command followed at once by
  MFC2 against a NOP and MFC2; table B, RTPS, N NOPs and MFC2 against the NOPs and MFC2, N = 0..20; table C, RTPS then at
  once CFC2, SWC2, MTC2, CTC2, LWC2, RTPS or NCLIP, against a NOP in RTPS's place; table E, six N/A numbers as table A.
  - *On Diamond* [M]: table A each command's cycles less one (RTPS 14 ... NCCT 38); table B 14 − N, none from N = 14;
    table C 14 for all seven; table E 14 for 00h, 0 for the rest. Before the choice below, table C read 0 for MTC2, CTC2
    and LWC2.
  - *Beetle PSX* [M, RAM only]: tables A and B as Diamond's but for AVSZ4, 4 where psx-spx's 6 gives Diamond 5; table C
    14 for all seven, the three writes included; table E as Diamond's.
  - *SwanStation*: every count one more than Diamond's (table A the psx-spx counts themselves, AVSZ4 6; table B 15 − N);
    table C 15 for all seven. Its run of the build with table E faults in the probe at every frame tried (120 to 900),
    as the probe has with SwanStation before (`Diamond_References.md` §8); not investigated.
  - *PCSX-ReARMed*: 0 everywhere, its timer not advancing within the measurement.
- **Referee:** not consulted (no device or simulator, `Diamond_Plan.md` §1.5).
- **The argument for the writes.** psx-spx lists what holds and does not say that anything does not; its sentence is
  silent on MTC2, CTC2 and LWC2, not against them. A write landing while a command runs must either wait or race the
  command for the registers it reads and writes, and the document describes no such race. The two witnesses that model
  any timing, of two lineages, both hold the writes. So the writes wait: argued, and the witnesses' agreement settles
  nothing of the hardware.
- **Conclusion:** ~~psx-spx's cycles; every access waits (reads and commands as psx-spx says, writes by the argument
  above); the boundary as §6. **Open** on two points: the writes, and the boundary, where the witnesses disagree by one
  cycle as they did for D-2's, Beetle PSX agreeing with Diamond. AVSZ4's 6 is psx-spx's and SwanStation's against Beetle's
  5. amidog's timing tests decide what they cover once the pad lands; the console decides all of it (`Diamond_Plan.md`
  §11 Q16).~~ *Overturned in part on 2026-10-11 by amidog's expectations (below).* psx-spx's cycles, AVSZ4's 6 among
  them; every access waits, writes by the argument above; the boundary as §6, now **measured against a console's
  expectations** for reads and commands; and an access that waits resumes one cycle after the command ends,
  **measured** the same way. **Open** for the writes, which amidog's loops do not issue, and for whether the cycle a
  wait adds is the processor's restart, which would bear on the multiplier's interlock too (D-2, not changed here).
- **Depends on it:** the cycle every GTE access completes, so every frame's timing in a 3D game.
- **Evidence since:** 2026-10-11, amidog's `psxtest_gte` 1.4, run once the pad answered: its OFFICIAL TIMING marks
  all 22 commands' timing ERROR against a console's expectations, which it does not print; Diamond's cycles a pass of
  its loops are in `Diamond_Pad.md` §5.3. Open again on that evidence. Run to its end, the test also marks MVMVA's
  flag column and the unlisted command numbers' values and timing ERROR (D-14, D-16).
- **The step of 2026-10-11, after the pad: amidog's expectations.** A dispute step of its own, which read amidog's
  test program and data from RAM (its expected values and thresholds are data, `Diamond_Disputes.md`'s rules) and ran
  black boxes; nothing it read was an emulator's or a library's GTE.
  - *What the test expects* (§7.2): each of its ten loops' 501 passes within 50 cycles of its rule's count, a pass the
    loop's instructions plus the command's cycles c from its table, c read as the NOPs' count n wherever c ≤ n + 1. The
    rule is the console's behaviour as its author wrote it down: its special cases are exactly where a console
    disagrees with "instructions plus c", and the test's purpose is to pass on a console. Read as a machine, the rule
    says one thing [A]: a command's result is ready c cycles after the cycle it issues in, an access that comes then or
    later costs nothing, and one that comes earlier resumes one cycle after the command ends. "Ready at c + 1 for every
    access" would cost the access at exactly c one cycle, which the rule's special cases deny; "ready at c" with no
    extra cycle, stage 5's, leaves every waiting loop one cycle short.
  - *Diamond against it* [M, `programs/gte-loops.s`, the test's loops copied in its shape for all 64 numbers, graded by
    the rule, and the test itself run with its verbose flag]: at stage 5, 0 of 22 official rows, every failing loop one
    cycle a pass short (two for the pairs) and the loops after which nothing waits passing; with the wait's last cycle,
    22 of 22, and `psxtest_gte`'s OFFICIAL TIMING passes every row on its own screen.
  - *The referees, the same program* [M, RAM at frame 900, DuckStation at 10 seconds]:

    | Referee | Official rows passing | What a pass shows |
    |---|---|---|
    | DuckStation, SwanStation | 13 of 22 | ready at c + 1 for every access: right wherever the access waits, one cycle long where it comes exactly at c (SQR, AVSZ3, GPF, GPL after 4 NOPs; NCLIP, DPCS, INTPL, MVMVA, DCPL after 7) |
    | Beetle PSX | 0 of 22 | each waiting loop one cycle shorter than Diamond's stage 5, two shorter than the rule |
    | PCSX-Redux | 0 of 22 | its NOP costs two cycles (`Diamond_Referees.md` §2.2); the shape is not comparable |
    | PCSX-ReARMed | 0 of 22 | fractional cycles a pass (9.0 to 100.3), not modelled at the cycle |

    No referee meets the rule everywhere. The DuckStation lineage agrees with it that a waiting access lands one
    cycle after c, and disagrees only at the boundary; Beetle PSX and stage 5 Diamond agree with it on the boundary
    and not on the wait. The rule rests on amidog's expectations alone, which are a console's at one remove; the
    witnesses neither settle nor contradict it beyond what the table shows.
  - *Changed*: `wait_for_gte` in Diamond's bus resumes an access that waits at `gte_ready + 1`. `gte-timing.s` now
    reads table A as each command's cycles (RTPS 15 ... NCCT 39), table B 15 − N for N up to 13 and 0 from N = 14,
    table C 15 for all seven, and table E as D-16's cycles; the DuckStation lineage reads tables A and C the same.
    Frame cost unchanged within noise: 6.49 ms a frame before, 6.46 after, the medians of four runs each, interleaved,
    of 4,600 frames of `psxtest_gte` through its OFFICIAL TIMING, the cycle counts identical [M]. `test-all` stays
    1,150 of 1,150 and the fuzz's 880 cases of the 22 commands stay identical on all four referees run again, timing
    not being among what they read.
- **Pinned by:** `the_gte_holds_every_access_until_its_command_is_done`;
  `amidogs_gte_timing_loops_take_the_cycles_its_test_expects`; WiseMan's
  `Amidogs_gte_timing_tests_pass_but_two_unlisted_numbers` (opt-in, with amidog's tests).
- **Implemented in:** `diamond: the GTE in the machine, every coprocessor 2 access held until its command is done`;
  `diamond: a GTE access that waits resumes one cycle after the command, as amidog's timing tests expect`.

### D-14. The GTE's accumulators: MAC1–MAC3 a 44-bit register that wraps, checked after each addition, IR saturated from the 32-bit MAC; MAC0's saturations from its full sum

- **Opened:** 2026-10-11, at stage 5, by the crate's replay of `test-all` missing 358 of 1,150 cases on psx-spx's
  plainest reading (§5.2).
- **Documents read:** psx-spx `6d7d1bc`, "GTE Saturation" (the FLAG bits' "larger than 43 bits", "larger than 31 bits"),
  the three command chapters and their "Details", "GTE Coordinate Calculation Commands"' note on FLAG 22. None gives the
  accumulators' width, when the overflow is checked, or which value IR is saturated from.
- **Test program:** ps1-tests' `test-all`, whose table is a console's (§7.1), replayed in the crate; each reading of §5.2
  tried against it, then each rule removed alone (§5.2's table).
- **Referee:** not consulted. **Witnesses:** Beetle PSX, SwanStation and PCSX-ReARMed agree with the result on all 880 of
  the fuzz's cases of the 22 commands (§7.3).
- **Conclusion:** §5.2's rules, as listed in its table. Measured: every rule but the three of its "not pinned" paragraph
  is required by the console's table, and the four readings were found from the table's misses, in the order given.
  **Settled** for what the table reaches.
- **Depends on it:** every MAC, IR and FLAG a command writes.
- **Evidence since:** 2026-10-11, amidog's `psxtest_gte` 1.4, whose OFFICIAL OPCODE marks MVMVA's flag column ERROR
  and every other command's OK (`Diamond_Pad.md` §5.3). The test prints no flag it wanted, so the case was found as a
  black box [M]: `gte-fuzz.s` with MVMVA alone, 1,484 cases with every bit drawn, run on Diamond and on DuckStation,
  SwanStation, Beetle PSX and PCSX-ReARMed. All four referees, of three lineages, agree with each other on every case
  and with Diamond on all but 7, and those 7 are FLAG alone: far colour, sf and lm set, Diamond raising one IR flag
  (bit 24, 23 or 22) more than they do, each the flag of a dropped part `(FCn SHL 12 + Mn1*V1) SAR 12` that is negative
  and above −8000h. Saturating the dropped part as if lm were 0, the depth cue's rule for its own intermediate IR,
  removes all 7 and keeps `test-all` at 1,150 of 1,150 (the table has no such case). **Changed**, on agreement between
  lineages, the one reading the console's table leaves open (§5.2's "not pinned" paragraph named the dropped part's
  IR as a choice). With it, `psxtest_gte`'s OFFICIAL OPCODE runs MVMVA's 256 variants to the end with X, F and V OK,
  where before it gave up after 193 on flag errors: the console's expectation agrees [M].
- **Pinned by:** `test_all_every_case_bit_for_bit` (with the corpus); `the_fuzz_program_reads_back_what_the_engine_computes`;
  `mvmva_far_colour_flags_its_dropped_part_as_if_lm_were_0` (one of the 7 cases).
- **Implemented in:** `blackmoon-gte: the GTE from psx-spx, every case of ps1-tests' test-all bit for bit`;
  `blackmoon-gte: MVMVA's far colour flags its dropped part as if lm were 0`.

### D-15. The GTE's write hazards: a write is seen at once by the next read or command, where psx-spx documents delays

- **Opened:** 2026-10-11, at stage 5, by psx-spx's three statements quoted in §8.
- **Documents read:** psx-spx `6d7d1bc`, "cop2r28 - IRGB", "Additional Functions" and "GTE Load Delay Slots".
- **Test program:** `gte-timing.s`'s table D: IR3 and IR1 read 0–3 NOPs after IRGB is written, LZCR 0–3 NOPs after LZCS,
  and SZ3 after an RTPS issued 0–2 NOPs after MTC2 or LWC2 writes VZ0.
  - *On Diamond, Beetle PSX, SwanStation and PCSX-ReARMed* [M]: the new value in every case.
- **Referee:** not consulted.
- **The argument.** psx-spx's IRGB figure is in instructions with a note that uncached code needs fewer, which makes it a
  time, not an instruction count; reading it as cycles on Diamond's clock would be a guess at the pipeline stage each
  instruction meets the GTE in. The LZCR delay has no figure and the load delay doubts itself. A delay implemented from
  them would be a rule no document fixes and no program here can test; games follow the documented NOPs, so the
  difference reaches only a program written to observe it.
- **Conclusion:** writes are applied at once. **Open**, for a console or a program that observes it; argued.
- **Evidence since:** 2026-10-11, amidog's `psxtest_gte` 1.4: BASIC REG and COMPLEX REG pass (F and V OK) with writes
  applied at once (`Diamond_Pad.md` §5.3). Their code was not disassembled to see whether any read falls inside
  psx-spx's delays, so they are consistent with the rule, not a test of it; its timing loops write nothing.
- **Depends on it:** the value a read or command sees within a few cycles of a write.
- **Pinned by:** `the_gte_holds_every_access_until_its_command_is_done` (table D's rows).
- **Implemented in:** `diamond: the GTE in the machine, every coprocessor 2 access held until its command is done`.

### D-16. The GTE's unlisted command numbers: every number clears FLAG; 00h runs as RTPS; the others do nothing else; each in the cycles amidog's test expects

- **Opened:** 2026-10-11, at stage 5, by psx-spx's "N/A" entries.
- **Documents read:** psx-spx `6d7d1bc`, "GTE Command Summary" ("00h - N/A (modifies similar registers than RTPS...)",
  "Unknown if/what happens when using the 'N/A' opcodes?") and "cop2r63 - FLAG" ("all bits are automatically reset at
  the begin of a new GTE command").
- **Test program:** `gte-fuzz.s`'s 24 cases of 00h, 02h, 15h, 1Fh, 2Fh and 3Ch; `gte-timing.s`'s table E.
  - *Beetle PSX* [M]: 00h equal to RTPS with the same bits on all four cases and RTPS's time; the others clear FLAG and
    nothing else, in no time. *SwanStation and PCSX-ReARMed*: nothing changes, FLAG kept, on all 24.
- **Referee:** not consulted.
- **Conclusion:** ~~as the heading. The FLAG rule takes psx-spx's general sentence at its word; "similar registers than
  RTPS" is read as RTPS, the one reading that writes registers psx-spx names and adds nothing it does not; no cycles are
  stated, so none are charged but 00h's [C]. Argued; **open** for a console. The witnesses disagree; one agrees.~~
  *Revised for the cycles on 2026-10-11 (below).* The FLAG rule takes psx-spx's general sentence at its word; "similar
  registers than RTPS" is read as RTPS, the one reading that writes registers psx-spx names and adds nothing it does
  not: argued, **open** for a console, the witnesses disagreeing and one agreeing. The cycles are amidog's table's,
  **measured against its expectations**, but for the extra cycle it gives 0Ah and 0Eh in one loop, which is open.
- **Evidence since:** 2026-10-11, amidog's `psxtest_gte` 1.4. Its table of cycles (§6, §7.2) gives every unlisted
  number a time, from 5 (04h, 09h, 0Dh, 1Dh, 27h, 3Bh) to 31 (17h), 00h 16 where RTPS is 15; its UNOFFICIAL TIMING
  grades a console's loops by them with the same rule as the 22 commands, and its value column for those rows is N/A,
  so it says nothing of what they compute; its UNOFFICIAL OPCODE groups list no command [M]. The table's 22 official
  entries are psx-spx's to the cycle, which is the reason for trusting its other 42 as a console's [A]. Run against
  `gte-loops.s` [M]: stage 5's Diamond and every referee fail all 42 rows, none charging an unlisted number any time
  but Beetle PSX's 00h, at its RTPS time (PCSX-Redux was given no unlisted number: its first, 00h, raised a reserved
  instruction exception in a trial run). With the table's cycles, 40 of 42 pass; 0Ah and 0Eh fail only the loop of
  CFC2 at once, where the test's rule adds one cycle more than their c (26 a pass for c = 19) and nothing else in it
  does. `gte-timing.s`'s table E reads 16, 13, 8, 8, 18 and 18 for 00h, 02h, 15h, 1Fh, 2Fh and 3Ch.
- **Depends on it:** a program that issues an unlisted number, which no game is known to do.
- **Pinned by:** `an_unlisted_number_clears_flag_and_00h_is_rtps`; `the_gte_holds_every_access_until_its_command_is_done`
  (table E); `amidogs_gte_timing_loops_take_the_cycles_its_test_expects`.
- **Implemented in:** `blackmoon-gte: the GTE from psx-spx, every case of ps1-tests' test-all bit for bit`;
  `blackmoon-gte: the unlisted numbers take the cycles amidog's timing tests expect`.

## 13. Reading log

Every file opened for stage 5, as the protocol of `Diamond_Plan.md` §1.3 asks:

- **psx-spx** `6d7d1bc`: `geometrytransformationenginegte.md` in full.
- **ps1-tests** source `f727802`: `README.md` at its GTE rows; `gte/test-all`'s `main.c`, `gte.h`, `gte.s`, `Makefile`,
  `psx.log` and `tests.c` (its harness read, its table parsed); `gte/gte-fuzz`'s `main.c`, `gte.h`, `Makefile`, the
  first 60 lines of `common.c` (its initialisation and pad reading) and `gte_valid_0xc0ffee_50.log` (its first case). A
  listing printed the first five lines of `common/common.h`, its include lines, as the header check `common/` requires;
  nothing else of `common/` was opened.
- **amidog**: `psxtest_gte.txt`.
- **Peter Lemon** `6d20c13`: `Cube/Cube.asm` (its first 120 lines), `Cube/LIB/PSX_INPUT.INC` at `WaitVSync`, a count of
  coprocessor-2 mnemonics in `Cube`'s sources; the bundled pictures of `GTEAVSZ`, `GTENCLIP` and `Cube`.
- **The references**, as black boxes: their RAM at one frame of `gte-timing.s` and `gte-fuzz.s`. Their logs of the runs
  on the player's BIOS were discarded unread; the region dumps the probe writes of the BIOS's addresses were deleted
  unread. The DuckStation and PCSX-Redux release binaries under `referees/` were not run at this stage.
- **EmuSen's own**: `Diamond_Plan.md` (§1, §5.1–§5.3, §6, §7, §13); `Diamond_Native.md` (§2.7, §10–§12);
  `Diamond_Disputes.md`'s rules and shape; `Diamond_References.md` but §4 and §6; `BlackMoon_R3000.md`; this crate, the
  R3000A's, `emusen-mips`'s assembler, Diamond's bus, state, programs and tests, WiseMan's `TtyReader` and `DiamondTests`;
  the cache's `DOCS.txt` and `PROVENANCE.txt`.
- **Not read**: any emulator's source, any GTE implementation in an emulator or a library (PSn00bSDK's and PsyQ's
  included), any firmware or listing, PSX_MiSTer.

### 13.1 The step of 2026-10-11 after the pad (D-13, D-15, D-16)

- **amidog's `psxtest_gte` 1.4**, unpacked in RAM by its own loader and read there with `emusen-mips`'s disassembler:
  its timing routine template (with its FFFFFFFFh placeholders) and the copies patched from it, the routine that
  builds the copies and grades their counts (with its jump table), the lookup of a command's table entry, the table's
  64 entries (their names and cycles; the other fields were printed with them and not interpreted), the two register-clearing routines it calls,
  its format strings, and its verbose flag; its menu screens and result screens, and its TTY with the flag set. The
  functions its table points at for computing each command's expected values, which are its author's GTE model, were
  **not** disassembled.
- **The references**, as black boxes: their RAM after `gte-loops.s` (PCSX-Redux on a variant with the unlisted
  numbers replaced by NOPs, after its trial run faulted), and the TTY lines of a PCSX-Redux trial run naming the fault;
  DuckStation's run through a private copy of its unpacked release, since another run shared the first copy's GDB port.
  The logs' BIOS lines were not read.
- **EmuSen's own**: `Diamond_Pad.md` §5, `Diamond_Disputes.md`'s rules, `Diamond_Referees.md` §2 and §3,
  `Diamond_Native.md`'s headings; Diamond's bus, `gte-timing.s` and its test; WiseMan's `AmidogScreen` and
  `DiamondTests`; `psxreferee.py`.
- **Not read**: any emulator's source, any GTE implementation (amidog's model functions included), any firmware or
  listing, `Diamond_References.md` §4 and §6.

## 14. Negative results and what waits

- ~~amidog's GTE test and `gte-fuzz` itself wait on the pad (§7.2, §7.3); so does Peter Lemon's `Cube`.~~ The pad
  answers since stage 6; `gte-fuzz` and `Cube` were not run again in this step.
- No program here can see the console, so D-13's writes, D-15 and D-16's values stay open; D-13's boundary and wait,
  and D-16's cycles, rest on amidog's expectations, a console's at one remove.
- 0Ah and 0Eh fail one loop of UNOFFICIAL TIMING by the extra cycle the test gives them there (D-16).
- No referee meets amidog's rule; PCSX-Redux faults on an unlisted number (D-13, D-16).
- SwanStation's probe run of `gte-timing.s` with table E faults (D-13).
- The first build's 358 misses are recorded in §5.2 as the readings that failed.
