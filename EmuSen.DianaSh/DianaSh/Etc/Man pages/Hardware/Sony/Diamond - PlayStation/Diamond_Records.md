# Diamond_Records — ps1-tests' TTY programs rewritten to record in RAM

*Written 2026-10-11.* ps1-tests' timing programs report by printf, and a referee that shows no TTY cannot be graded on
them: PSX_MiSTer in simulation has no TTY at its top level (`Diamond_FPGA_Referees.md` §4), and no libretro channel
carries one (`Diamond_Native.md` §11.15). This page records five of those programs rewritten to store every number they
print in a fixed layout from `80100000h`, as `gpustat.s` is of `gpu/gpustat`: `timers`, `cpu/access-time`,
`dma/chopping`, `gpu/bandwidth` and `dma/chain-looping`, in `programs/` beside Diamond's own. For each it gives the
record's layout (§2–§6), the check that the record holds what the release program prints on the same machine, the
record against the console's log, and what the referees answered (§7). §1 is the method, §8 the findings for Diamond,
§9 what was read.

Marks as in the plan: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44) with the tool named;
**[R]** read from the source named; **[A]** argued; **[C]** a choice.

---

## 1. Method

### 1.1 What a rewrite keeps

A timing program measures the instructions between two timer reads, and on a machine with an I-cache it measures where
those instructions lie: a loop that runs once pays its refills, and their cost depends on its lines and on what the
lines held before [A]. A rewrite that measures what the original measures therefore keeps the original's instructions
and, where a timed loop runs cold, their addresses [C]:

- **`cpu/access-time`, `dma/chopping`, `gpu/bandwidth`, `dma/chain-looping`**: the release build 158's own compiled code,
  the test's `main.cpp` with the files of ps1-tests' `common/` it links (`io.c`, `dma.cpp`, `gpu.cpp`, `twister.c`), is
  carried over word for word at the release's addresses. The program's two first words are replaced by the entry; every
  call into PSn00bSDK goes to a stand-in at the end of the program (§1.2). The words are the release's, read back by
  `emusen-mips`'s disassembler and assembled again; assembled, each program differs from its release over the carried
  range only in calls: 291 of 4,416 words for `access-time`, 31 of 757 for `chopping`, 39 of 754 for `bandwidth`, 24 of
  544 for `chain-looping` [M].

  **That code is not kept in the tree** [C, decided 2026-10-11]. It is the release's, not the project's: it includes
  the Mersenne twister of `twister.c`, whose licence asks for its notice, and code compiled from `dma.cpp` and `gpu.cpp`,
  which include `dma.hpp`, excluded for every writer. The tree holds only the project's own part of each program,
  `programs/records/<name>.s`: the entry, the stand-ins, the start-up and the record's layout, with an `@release` line
  naming the carried range. `tests/support/records.rs` carries the code in at that line from the local release, which
  `EMUSEN_DIAMOND_PS1TESTS` names (the corpus WiseMan's TTY gate reads), as blackmoon-gte's oracle reads its tests from
  its corpus: it reads the range's words from the PS-EXE, refuses them unless their SHA-256 is the one pinned for build
  158, disassembles them and points each call outside the test's code at the stand-in its table names. The program tests
  that need the four build them so and pass unrun without the corpus, saying so; `cargo run --release --example
  records [OUT]` writes their sources and PS-EXEs into `~/.cache/emusen/probe/diamond/records/` (or OUT) for the
  referees, never into the tree. Built so, the four PS-EXEs are byte for byte those the referees ran [M].
- **`timers`**: its five measuring functions (`testTimerWithCyclesDelay`, `testTimerWithFrameDelay`,
  `testTimerWithCyclesDelaySyncEnabled`, `readTimerValues` with `testTimerTargetBehaviour`, `testTimerResetFlagWritable`)
  instruction for instruction from the release, with `io.c`'s `read16` and `write16` and `main.c`'s `delay`, at the
  program's own addresses. Every row it times runs warm after its first sample, which the program does not print, so
  the addresses do not reach the numbers [A, and the equality of §2.3]. Its header carries ps1-tests' MIT notice.

Where the test's code ends and PSn00bSDK's begins was found without reading the library: the test's own `main` names
its callees, each of which is either a function of the test's sources or not, and `io.c`'s accessors were found by their
words; the library begins at the lowest address the test calls that is not its own [M]. The test's own initialised
data, where it is read (gpu/bandwidth's `buffer`, `801A0000h`), is set by the program's start-up.

### 1.2 What stands in for PSn00bSDK and the kernel

The library's code is not read. Each of its functions the test calls is replaced by a stand-in that makes **the bus
accesses the release's call was seen to make on Diamond**: Diamond's own trace of the release, the address, value and
cycle of every load and store to the I/O ports, read as a logic analyser would read the bus [M, `Diamond_Native.md`
§11.11's tools]:

| Library call | Its stand-in |
|---|---|
| `ResetGraph(0)` | DPCR `3B333B33h`, DICR 0, timer 1 on hblanks (`1D00h`), GP1(00h) |
| `PutDispEnv`, `SetVideoMode`, `SetDispMask` | the GP1(06h), (07h), (08h), (05h), (08h) and (03h) words the release wrote |
| `PutDrawEnv` | GP1(04h) 2 and a one-node linked list by DMA of the GP0(E3h), (E4h), (E5h) and (E1h) words it sent |
| `DrawPrim` | GPUSTAT; when bit 26 is set the channel's CHCR and GPUSTAT again, otherwise GPUSTAT until bit 28; GP1(04h) 0; the packet's words to GP0 nine cycles apart; then GPUSTAT until bit 28 |
| `DrawSync(0)` | GPUSTAT, then GPUSTAT until bit 28 |
| `ReadGPUstat` | GPUSTAT |
| `malloc` | the address the release's call returned |
| `rand` | a linear congruential generator of the program's own; it sets colours, not times |
| `EnterCriticalSection`, `ExitCriticalSection` | nothing: interrupts are off throughout |
| `printf` | the record: the numbers the call carries, stored where §2–§6 say, picked out by the format's address |
| `VSync` | the done word, `D1A0D00Eh` at `800FFFF0h`, and a loop |

`gpu/bandwidth` is the one program whose numbers depend on the stand-ins' own timing, since each of its 400 primitives a
row is fed by `DrawPrim`; there the stand-in's delays between its accesses are the intervals the release's call was
seen to keep on Diamond (§5.2) [M, fitted].

### 1.3 The checks

**Equivalence.** Each record is compared with the numbers the release program prints on Diamond, its open kernel,
the same market, by `EmuSen.WiseMan/Reference/analysis/psxrecords.py equal`, which renders the record as the release's
text. A difference is a defect of the rewrite until a measurement shows its cause outside the measured instructions.
Every difference below was traced to such a cause with Diamond's instruction trace of both runs; for `timers`' frame
rows, `chopping` and `chain-looping` an experiment then gave the rewrite the same cause and met the release's numbers
(§2.3, §4.2, §6.2); for `timers`' target counts and `bandwidth` the residual is stated with its cause and not
reproduced (§2.3, §5.3) [M].

**The console.** `psxrecords.py console` grades a record by the rule WiseMan's TTY gate applies to the release
(`Diamond_Native.md` §11.12; `Diamond_Referees.md` §6 for `chopping`'s, which the gate does not grade), and with `--tty`
the release's own text the same way, so that the two can be set side by side.

**The record's conventions** are those of `programs/`: assembled by `emusen-mips` at `80010000h`, run with no kernel
or side-loaded by one, results from `80100000h`, the done word at `800FFFF0h`. Bare metal: the stack and gp as the
release's start-up leaves them, interrupts off (SR 0), the release's data zeroed. Each is pinned on Diamond by
`tests/programs.rs`, the four that carry the release's code only where the corpus is named.

## 2. `timers` — `programs/timers.s`

### 2.1 What it measures

The release's measurements in its order, forced into NTSC mode as the release forces it (`runTestsForMode(MODE_NTSC)`)
[R its `main.c`]: each timer and clock source over a 1,000- and a 5,000-cycle delay and over a frame; timer 0's four sync
modes at the five dot clocks, and timers 1's and 2's, 100-cycle delays not reset between samples; timer 2 reset at a
target of 10 and read 10,000 times; timer 0's mode register read after writes and after reaching FFFFh. A frame row
waits for the vblank as the release does, on a flag its interrupt toggles: the program's own vblank handler, enabled for
those rows alone, stands for the release's callback.

### 2.2 The record

| From | Words | What |
|---|---|---|
| `80100000h` | 18 × 10 | the delay and frame rows in the log's order (timer 0, 1, 2; clock 0 then 1, timer 2's being 0 and 2; 1,000 cycles, 5,000, a frame); each row `results[1..10]`, as printed |
| `80100300h` | 40 × 10 | timer 0's sync rows: clock source 0 and 1, the resolutions 256, 320, 368, 512, 640, syncs 0 to 3 |
| `80100A00h` | 16 × 10 | timer 1's and 2's: timer, clock 0 and 1, syncs 0 to 3 |
| `80100D00h` | 12 | the target check's counts of counter values 0 to 11 |
| `80100D30h` | 1 | the mode read after the 10,000 reads |
| `80100D34h` | 4 | "Reached target", "Reached 0xFFFF", "Counter reset AFTER reaching target", "Target + 1 not reached", 1 or 0 |
| `80100D80h` | 6 | timer 0's mode read after writing 0000h, after FFFFh, after the wait, again, after a write, again |

A sample is the release's `int32_t`, its overflow corrected by adding FFFFh as the release adds it.

### 2.3 Against the release, on Diamond

Run as a PAL console, the console log's, for 120 frames (done by frame 80) against the release on the open kernel
[M, `psxrecords.py equal timers`]:

| Rows | Result |
|---|---|
| the six system-clock delay rows | equal, sample for sample (1,013 and 5,013) |
| the dot-clock, hblank and ÷8 delay rows | within the same two adjacent values as the release's; which sample reads which depends on the slower clock's phase |
| the frame rows, settled (first two dropped, as the gate drops them) | system clock +21, +20, +20 cycles; dot clock +4; ÷8 +3; hblanks 263, equal |
| the sync rows | 35 of the 37 that count in both rise by the same step a sample (127 cycles; 20, 25, 28, 40 or 50 dots); the values themselves depend on where in the line each test began, so they cannot be compared |
| the target check | the same 23-cycle loop in both, read 10,000 times; the counts differ by up to 2 at single values, by where in the counter's period the first reads fall; the four booleans equal |
| the mode reads | equal: 0400h, 07FFh, 1C08h, 0408h, 0408h, 0408h |

**The frame rows** [M]. A frame row is the time from the mode write that resets the counter to the read after the next
vblank; the vblank's latency cancels between successive samples, and what remains is a frame less the code between a
read and the next reset. That code is the same in both; in the release it runs cold. The
release's vblank goes through the open kernel's handler, whose code shares cache lines with the test's loop: Diamond's
trace of the release shows 8-cycle refills at `800103B4h`, `800103C4h` and `80010378h` each iteration, 44 cycles from the
read to the next reset where the rewrite takes 28, and the store queue's drain falling differently around them for the
remaining five. With the rewrite's own handler made to run through code at addresses that share its loop's lines
(`800111E0h`, `80011220h`), its frame rows become 112,530 and 112,549 alternately, either side of the release's 112,536;
its dot-clock row 112,027 to 112,031 about the release's 112,028 and 112,029; its ÷8 row 71,409 to 71,412 about 71,410
and 71,411. The rewrite keeps its handler cache-neutral [C]: the handler that runs on a console is Sony's, whose lines no
document gives, so the release's figure is the open kernel's, not the console's.

**The target counts** [M]. Both runs read the counter every 23 cycles, 9,999 intervals but the first; the release's
first is 37 cycles, the rewrite's 30, the release's loop refilling one line more after its `EnterCriticalSection`. The
counts therefore start at another point of the counter's period: of the four reads beyond whole periods, the release's
fall at 0 (two), 4 and 10, the rewrite's at 4, 5 (two) and 6. Delaying the rewrite's start by 1 to 15 cycles moves its
excess along the values as the period says, but no single delay reproduces the release's, which also has the first
interval's refill; the counts are not graded, and their sum and the booleans are [A].

### 2.4 Against the console's log

18 of 18 rows within the gate's tolerances, as the release's [M, `psxrecords.py console timers`]. The log's console
build is not build 158 (its `ResetGraph` reports `itb=8001207c`, the release's `8001376c` [R the log; M]), and its delay
rows read 1,011 and its sync steps 123 where build 158 reads 1,013 and 127 on Diamond; the console's code was laid out and
perhaps compiled differently, so two cycles of a delay row and four of a sync step are not a measurement of Diamond [A].

## 3. `cpu/access-time` — `programs/records/access-time.s`

### 3.1 The record

From `80100000h`, the seventeen regions in the table's order (RAM, BIOS, SCRATCHPAD, EXPANSION1, EXPANSION2, EXPANSION3,
DMAC_CTRL, JOY_STAT, SIO_STAT, RAM_SIZE, I_STAT, TIMER0_VAL, CDROM_STAT, GPUSTAT, MDECSTAT, SPUCNT, CACHECTRL), 24 bytes a
region: for 8, 16 and 32 bits, the whole cycles a load and the hundredths, `measuredCycles / 100` and
`measuredCycles % 100` as the release passes them to `"%2d.%2-d"` (so "5.3" is 5 and 3 hundredths). SPUCNT's 32-bit load
is misaligned and raises an address error; the program's handler returns past the load, as the kernel's does.

### 3.2 Checks

**Against the release on Diamond**: all 51 cells equal [M]. Each measurement runs its loop once from a cold cache, which
is why the code is the release's at the release's addresses; `read16` and `write16` sit at `80014560h` as in the release,
whose lines the 17 functions' code shares across its 17 KB [A].

**Against the console's log**: 49 of the 51 cells within 0.2 cycles, 50 within 0.5, exactly as the release on the open
kernel; the two are RAM's 8-bit cell (5.01 against 5.21) and D-4's SPUCNT 32-bit cell (36.07 against 38.94)
[M, `psxrecords.py console access-time`]. `Diamond_Native.md` §11.14 records 50 of 51 for the release on a player's
BIOS; the open kernel's 49 suggests that RAM's 8-bit cell, the first measured, is the one a kernel's footprint moves,
which was not run here to check [A].

## 4. `dma/chopping` — `programs/records/chopping.s`

### 4.1 The record

From `80100000h`, 130 rows of six words in the log's order (block sizes 1 to 64 and 128 in sync mode 1; the unchopped
burst; DMA windows of 1 to 128 words against CPU windows of 1 to 128 cycles): the sync mode, the bytes (8,192), the block
size or the DMA window in words (0 for the burst), the CPU window in cycles (0 but chopped), the CPU cycles, and 1 where
the release prints "(GPU busy)". The data, the release's Mersenne twister seeded with 1337, is the release's own code
(`twister.c`).

### 4.2 Against the release, on Diamond

The busy marks: 130 of 130 equal. The cycles: 1 row equal; 120 rows 14 cycles fewer, five 21, two 15, one 34 (the
burst), one 7 [M, `psxrecords.py equal chopping`].

**The cause** [M]. Each row's window is `measure`'s: `EnterCriticalSection`, the timer reset, the channel started and
polled, the timer read. In the release, `EnterCriticalSection` is a system call into the open kernel, whose code
evicts lines of the window's own: Diamond's trace shows the release's window refilling the lines at `80010220h`,
`80010230h`, `80010240h`, `80010700h`, `80010710h`, `80010720h` and `800109E0h`, which the rewrite finds warm. With the
rewrite's `EnterCriticalSection` stand-in made to run through code sharing those seven lines (and nothing else changed),
**all 130 rows equal the release's** [M]. The committed rewrite keeps the stand-in empty [C], for the reason of §2.3:
the eviction is the open kernel's, and a console's BIOS evicts its own lines.

**When the busy mark is read** [M]. The release reads GPUSTAT after `measure` returns, that is after
`ExitCriticalSection`: on the open kernel 2,141 cycles after the timer's read, and 14,799 in a row where an interrupt was
pending when the call re-enabled them. The rewrite reads it at once. On Diamond the two agree on every row, since
Diamond's GPU is ready at once after a complete transfer and stays not ready where the block size leaves words unsent
(2,048 modulo the block size not zero: block sizes 3, 5 to 7, 9 to 15, 17 to 31, 33 to 63) [M]. This bears on D-11 (§8.2).

### 4.3 Against the console's log

129 of 130 rows within 10%, 117 of 130 busy marks as the console's, as the release's [M].

## 5. `gpu/bandwidth` — `programs/records/bandwidth.s`

### 5.1 The record

From `80100000h`, 14 rows of four words in the log's order (vramToCpu, cpuToVram, vramToVram, FillScreen, the rectangles
plain, semi-transparent, textured and both, the quads likewise, the quad a quarter and a half off screen): the
milliseconds' whole part and remainder as `calculate` computes them, the hblanks, and the MB/s.

### 5.2 The stand-ins' timing

Each row times 400 calls of a function that ends in `DrawPrim`, so the stand-in's semantics and cadence are part of the
measurement. Two findings from the release's bus trace, both behaviours of the library and both needed [M]:

- `DrawPrim` and `DrawSync` wait for GPUSTAT bit 28, not bit 26: a primitive's words go into the FIFO while the previous
  one draws, and `DrawSync` returns with the GPU still drawing. The first row's timer therefore includes the end of the
  last of `clearScreen`'s four fills, about ten hblanks. A first stand-in that waited for bit 26 left every primitive row
  9 to 12 hblanks from the release's.
- The intervals between the release's accesses: from the first GPUSTAT read, the GPU idle, the CHCR read 37 cycles
  later, GPUSTAT 11 after that and GP1(04h) 22 after; the GPU busy, GPUSTAT 40 cycles later and then every 12 until bit
  28, GP1(04h) 19 after the last; the first word 15 after GP1(04h), then one every 9; after the last word, GPUSTAT 12 and
  16 cycles later, then every 12 until bit 28; the test's next access 32 cycles after that, or 19 after `DrawSync`'s.
  Each GPUSTAT read is a call of `ReadGPUstat` in the release. The stand-ins' NOPs were set until these intervals of
  `DrawPrim`, and the 136 cycles from the timer's reset to its first read, came out the same on Diamond [M, fitted];
  the vramToCpu row's period is then 84,827 cycles an iteration against the release's 84,830. `DrawSync`'s, once a
  row and before the timer starts, were not matched: its code runs cold, and the stand-in's accesses come 47, 21 and 40
  cycles later than the release's.

### 5.3 Checks

**Against the release on Diamond**: 9 of 14 rows' hblanks equal; vramToCpu 2 fewer, cpuToVram, FillScreen and textured
semi-transparent rectangles 1 fewer, the quad 1 more [M]. The remaining differences are the stand-ins': 3 cycles a
vramToCpu iteration in the call's entry, and `DrawSync`'s cadence before each row's timer starts, which moves how much of
the last clear fill the row's first primitive waits behind (the release's first post-send wait polls 175 times, the
record's 125). Matching them needs the library's own code, which is not read [A].

**Against the console's log**: 14 of 14 rows within 2%, the widest 0.6% (cpuToVram, 12,123 against 12,195), as the
release's [M].

## 6. `dma/chain-looping` — `programs/records/chain-looping.s`

### 6.1 The record

From `80100000h`, three words a test in the log's order (no DMA; an empty but complete chain; a single
self-referencing node; a double one): the ticks as printed (`measure` on the system clock over 8, times 8), 1 for
"finished = true" and 1 for "irq = true". The first test prints neither and records FFFFFFFFh for both.

### 6.2 Checks

**Against the release on Diamond**: the booleans equal; every count 8 fewer, one tick of the ÷8 clock [M]. Measured in
cycles from the timer's reset to its read, the no-DMA window is 16,025 cycles in both: the tick it loses is the
prescaler's phase at the reset, a property of the absolute cycle on which the program reaches it. Under a chain the
rewrite's windows are 7, 11 and 13 cycles shorter: the release's `EnterCriticalSection` evicts the line at `80010120h`,
and its system call puts 2,607, 6,334 and 3,950 cycles between the chain's start and the window where the rewrite puts
60 to 80, so the window opens at another point of the never-ending chain's period. With both given to the rewrite (the
line evicted, the gaps waited out to within a cycle), its windows read 16,025, 16,016, 24,018 and 24,018 cycles against
the release's 16,025, 16,016, 24,019 and 24,019 [M].

**Against the console's log**: the words of all four lines equal, every count within 10% (D-5's 6.3% under a chain), as
the release's [M].

## 7. The referees

Each rewrite was side-loaded on the five referees of `Diamond_Referees.md` §2 by `psxreferee.py ram`, the player's
BIOS given by path as there, `timers` as a PAL console and the others NTSC, and its record read from RAM [M]. Beetle
PSX, SwanStation and PCSX-ReARMed show no TTY, so before the rewrites they could not be graded on these programs at all;
PSX_MiSTer's harness reads RAM too (`Diamond_FPGA_Referees.md` §4) and can now be given them, though it was not run.

### 7.1 The rewrites on the two referees that also show a TTY

On DuckStation and PCSX-Redux the release's TTY was read as well, so the record can be checked against the release on
a second and a third machine [M, `psxrecords.py equal`]:

| Program | DuckStation | PCSX-Redux |
|---|---|---|
| `access-time` | 51 of 51 cells equal | 51 of 51 equal |
| `chopping` | 109 of 130 rows equal, the rest ±4 cycles; busy marks 130 of 130 | 130 of 130 rows and marks equal |
| `chain-looping` | 1 of 4 equal, the others 8 or 16 fewer (one or two ticks of the ÷8 clock) | 4 of 4 equal |
| `timers` | the system-clock delay rows equal; the frame rows 8 cycles fewer (its BIOS's handler, as §2.3); one flag read differs (§7.2) | the system-clock delay rows equal, the frame rows within 2 cycles; it stops where the release stops (§7.2) |
| `bandwidth` | neither finished the first row in 300 seconds | every row 20 or 21 hblanks more (§7.2) |

### 7.2 What the records showed

**`timers`** (the gate's 18 rows; the console's mean is over the settled samples) [M]:

| | Diamond | DuckStation | SwanStation | Beetle PSX | PCSX-ReARMed | PCSX-Redux |
|---|---|---|---|---|---|---|
| rows within the gate's tolerances | 18 | 13 | 13 | 9 | 2 | 1 |
| 1,000 cycles on the system clock (console 1,011) | 1,013 | 1,013 | 1,013 | 1,015 | 1,769 | 2,023 |
| a frame on the system clock (console 112,531) | 112,557 | 112,633 | 112,641–112,658 | 112,560 | 105,694 | 105,615 |
| a frame on the dot clock (console 112,031) | 112,033 | 112,196 | 112,198 | 112,184 | 112,888 | 94,059 |
| sync mode 2, timer 0's ten rows | zero outside the blank, counting in it (8 rows; 2 missed the blank) | zero in 7, mixed in 3 | zero in all | counting in all | counting in all | zero in all |
| the target check's booleans (console 1, 0, 1, 1) | 1, 0, 1, 1 | 1, 0, 0, 1 | 1, 0, 0, 1 | 1, 0, 0, 1 | 1, 0, 0, 0 | not reached |

Only Diamond resets timer 2 at its target and holds it there as the console's log shows (a count at every value 0 to
10, twice at 0); DuckStation and SwanStation see every other value 2,000 times, Beetle PSX the value 8 9,999 times,
PCSX-ReARMed four values 2,500 times with the target exceeded. On PCSX-Redux the release stops in the target check with
an address error at the loop's halfword store (the counter passes 300, the table's size), as its own log says
("StoreAddressError from 0x800100d0"), and the record stops at the same loop. The flag check's reads after writing FFFFh
differ between the release and the record on DuckStation (FFFEh and E7FFh): the release's read follows the printf of the
read before, during which the counter reaches its target of FFFFh, and DuckStation keeps the reached flags across a mode
write where Diamond clears them; the printf's time is the kernel's, so the two reads are of different moments [A].

**`cpu/access-time`** (51 cells within 0.2 cycles of the console's; then D-4's named cell) [M]:

| | Diamond | DuckStation | SwanStation | Beetle PSX | PCSX-Redux | PCSX-ReARMed |
|---|---|---|---|---|---|---|
| cells within 0.2 | 49 | 36 | 33 | 27 | 24 | 0 |
| RAM 8/16/32 (console 5.21, 5.03, 5.14) | 5.01, 5.01, 5.08 | 7.01, 7.01, 7.05 | 7.01, 7.01, 7.05 | 4.04, 4.04, 4.09 | 3.02 each | 1.76, 1.77, 1.77 |
| EXPANSION2 (console 10.99, 25.99, 55.98) | 11.07, 26.00, 56.00 | 1.04, 1.00, 1.00 | the same | 3.07, 3.02, 3.00 | 3.00 each | 1.75 each |
| SPUCNT 32-bit, an address error's round trip (console 38.94) | 36.07 | 82.04 | 82.04 | **38.09** | 6.00 | 3.50 |

The SPUCNT 32-bit cell is the first referee reading of D-4's named cell: the rewrite's exception goes through its own
handler (return past the load) on every machine, so the cell compares the machines' exception entry and return on the
same code, where the console's figure is Sony's handler's. Beetle PSX reads 2.02 cycles more than Diamond with the same
handler, and 0.85 fewer than the console with its own [M, A].

**`dma/chopping`** (rows within 10% of the console's; busy marks as the console's, of 130) [M]: Diamond 129 and 117;
DuckStation 17 and 117; SwanStation 11 and 117; Beetle PSX 5 and 117; PCSX-Redux 0 and 60; PCSX-ReARMed 0 and 19. The
burst takes 2,201 cycles on Diamond, 2,209 on DuckStation and SwanStation, 2,032 to 2,129 on Beetle PSX in three runs
(before and after two changes to the stand-ins outside the timed window, which left DuckStation's, SwanStation's and
PCSX-Redux's records word for word as they were; Beetle PSX's cycles moved by up to a hundred, its counts and marks
not), 54 on PCSX-Redux and 43 or 44 on PCSX-ReARMed, against the console's 2,196. Diamond, DuckStation, SwanStation
and Beetle PSX mark the GPU busy on the same 57 rows, exactly those whose blocks leave words unsent; none marks the 13
rows D-11 names.

**`gpu/bandwidth`** (rows within 2% of the console's) [M]: Diamond 14; Beetle PSX and SwanStation 0, the plain fill and
the quads 7 to 10% fast (FillScreen 1,102 hblanks against 1,216, the quad 7,147 and 7,133 against 7,727) and the
textured quads 50% or more (Beetle 20,070, SwanStation 14,266 and 21,382, against 39,938); PCSX-Redux and PCSX-ReARMed
draw at once (62 to 126 hblanks a row but the two transfers between VRAM and RAM, 7,274 and 3,246 on PCSX-Redux, 1,883
and 1,879 on PCSX-ReARMed); DuckStation did not finish vramToCpu in 300 seconds, with the release or the record. On
PCSX-Redux each row of the record is 20 hblanks longer than its release's: its rows are bound by the CPU, not the GPU,
and the stand-in's delays are Diamond's cycles of the library's code (§5.2), which PCSX-Redux, two cycles a NOP, runs at
twice their cost; on a referee whose GPU draws, the rows are bound by drawing and the stand-in does not show.

**`dma/chain-looping`** (the four lines' words equal, then within 10%) [M]: Diamond 4 and 4; DuckStation and
SwanStation 4 and 0 (13,016 with no DMA, 143,008 under a never-ending chain); Beetle PSX 4 and 0 (13,008 to 13,032 in
every test, no slowdown under a chain); PCSX-Redux 2 and 2 (16,016 throughout, every chain reported finished);
PCSX-ReARMed 2 and 0 (12,264 throughout).

### 7.3 Which referees could now be graded

Beetle PSX, SwanStation and PCSX-ReARMed, on all five programs, for the first time; PCSX-Redux on `timers` up to its
stop and on all of `chopping`, `bandwidth` and `chain-looping`, which it could be before by its TTY; DuckStation as
before. Read against the console's logs, the ranking of `Diamond_Referees.md` §6 holds and extends: Beetle PSX and
DuckStation remain the strongest on their subsystems, Beetle PSX joins the referees that mark `chopping`'s busy rows as
Diamond does, and no referee reproduces the console's access times, DMA gaps, drawing times or timer 2's target reset
[A].

## 8. Findings for Diamond

### 8.1 Timer 0's sync mode credits time from before its mode was written

Not fixed; reported. Writing timer 0's mode with sync enabled resets its count, yet Diamond's first read after the write
returns the cycles since a video edge it last processed: a program of the project's own that waits for a vblank, then
8,192·k iterations of a 4-cycle loop, then writes mode 0001h and reads the count twice 5 cycles apart, reads 0 and 0,
7,053 and 7,058, 25,713 and 25,718, 18,003 and 18,008 for k = 0 to 3, where Diamond's own `write_mode` sets the count to
0 and a read a few cycles later should return a few counts [M; R `timers.rs`]. The release `timers` shows it in its
timer-0 sync rows on Diamond, which start at 32,106, 11,731 and the like where the console's start at 260 and 140; the
rewrite's rows, written soon after a GP1 write, show smaller credits (848 where about 260 is due). The likely cause, read in
Diamond's own `bus.rs` and `timers.rs`: hblank edges are only scheduled while timer 0's sync bit is set, so on the write
the edges since the last processed one are caught up with times before the write, and `Timers::sync` sets its tick base
to each such time, rewinding it, so the next read counts that span again [R, A]. The sync-mode tables are not graded by
the TTY gate, and D-6's mode-2 rows reset at both edges of the blank, which hides it there.

### 8.2 What the busy marks of `chopping` measure (D-11)

The release reads GPUSTAT after `ExitCriticalSection`'s system call (§4.2), on a console after the retail BIOS's call,
whose duration no log gives. D-11's 13 rows the console marks busy and Diamond does not are therefore busy at least that
long after the channel ends, not at once; `gpu-busy.s`, which polls at once, is the sharper program, and a referee's
marks from the rewrite are at once. A question, not a ruling: the dispute stays open.

### 8.3 The release on the open kernel

The TTY gate's figures for the release programs include the open kernel's cache footprint, a few cycles a measurement
(§2.3, §4.2, §6.2): another build of the open kernel can move `chopping`'s rows by tens of cycles and `access-time`'s RAM
cell by 0.2 without any change to Diamond's hardware model. The rewrites do not have this dependence.

## 9. What was read

- **ps1-tests** (MIT, `~/.cache/emusen/probe/diamond/tests/ps1-tests`): the five programs' `main.c`/`main.cpp` and
  `Makefile`, and `psx.log`; `common/`'s `test.h`, `measure.hpp`, `timer.h`, `io.c`, `io.h`, `delay.c`, `delay.h`, `gpu.h`,
  `gpu.cpp`, `dma.cpp`, `twister.c`, `exception.hpp`, `common.h`, `log.h`, `common.mk`, `common-test.mk`, each after its
  header was checked for a copied-from note. `cop0.hpp` and `dma.hpp` were excluded by their headers; the first eight
  lines of each were shown with the others' headers, which for `dma.hpp` included the namespace, the channel names and
  `CH_BASE_ADDR = 0x1F801080` (psx-spx's address). `dma/dpcr/main.cpp`, `dma/otc-test/main.cpp` and their logs were read
  for rewrites not made (§10). The release build 158's executables: each one's own compiled code, disassembled from its first
  word to the end of its last `common/` function; of PSn00bSDK's, four words where a window ran past a function's end
  (`80010E84h`–`80010E8Ch` in `timers`', `80010BD0h` in `gpu/bandwidth`'s), the addresses of the calls' targets, and
  the addresses of a few return instructions in a search's output.
- **The release's behaviour on Diamond**: its I/O loads and stores with their cycles and values, its instruction
  boundaries' cycles and PCs inside the timed windows, its registers at `main`'s entry, and the GPU words of
  `PutDrawEnv`'s DMA node read from RAM when the channel started [M]. The PCs observed outside the test's code were the
  library's and the open kernel's addresses, recorded as addresses only; no library or kernel instruction was decoded.
- **Diamond's own source**: `timers.rs`, `bus.rs`'s timer, event and I/O paths, `icache.rs`, `machine.rs`, `exe.rs`,
  `examples/run.rs` (§8.1, and the tools).
- **The project's pages**: `Diamond_Disputes.md` (the rules, D-0, D-4 to D-6, D-10, D-11), `Diamond_Referees.md`,
  `Diamond_FPGA_Referees.md`, `Diamond_Native.md` §11.9–§11.18, `Diamond_GPU.md`'s opening and contents; DOCS.txt's
  exclusions.
- **The referees' outputs**: their RAM, and for DuckStation and PCSX-Redux the release programs' TTY from each
  program's heading on, which for PCSX-Redux carries its own messages (an address error's report in `timers`) and for
  DuckStation the release library's `VSync: timeout`; the process table and the processes' cgroups.
- **Not read**: any emulator's source; PSn00bSDK's sources; any BIOS, firmware or listing (the player's BIOS was handed
  to the referees by path, and no referee's log before a program's heading was read); psx-spx.

## 10. Not done

- `dma/otc-test`, `dma/dpcr` and `cpu/io-access-bitwidth`, which report values and pass or fail lines rather than
  times: the same method applies, with `printf`'s stand-in keeping each call's format address and arguments and the
  renderer reading the formats from the release, but they were not rewritten in this step. Diamond passes the first two
  by its TTY (`Diamond_Native.md` §11.14).
- `cpu/cop` and `cpu/code-in-io`: their tests take exceptions through the kernel, hooking its handler and reading its
  thread structures (`common/exception.hpp`), which a bare-metal rewrite would have to stand in for; not attempted.
- PSX_MiSTer was not run on the records (`Diamond_FPGA_Referees.md`, its status).
