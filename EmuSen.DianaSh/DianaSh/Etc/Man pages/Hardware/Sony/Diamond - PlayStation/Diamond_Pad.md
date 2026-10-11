# Diamond_Pad — SIO0 and the controllers

*Written 2026-10-11.* This page records Diamond's controller and memory card port, SIO0, and the devices on it. Its
first part, a digital pad on port 1, was **brought forward from stage 6** (decided 2026-10-11, `Diamond_Plan.md` §7)
so that amidog's CPX, GPU and GTE tests, which wait at a menu for Start, could run; §5 records what they showed. The
marks are the plan's: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from the source named; **[A]**
argued; **[C]** a choice where the documents are silent.

The code is `src/sio0.rs` (the port) and `src/pad.rs` (the digital pad), one job each: the port shifts bytes, keeps
the FIFO, routes the address byte to a slot's device and raises the interrupt; a device answers one byte at a time and
says whether it pulls /ACK. Stage 6 adds devices beside the pad through the same seam (§6).

---

## 1. The registers

psx-spx's "Serial Interfaces (SIO)" [R], at 1F801040h:

| Offset | Register | What Diamond does |
|---|---|---|
| 0h | DR | A write puts the byte to send in the transmitter's holding slot, overwriting one not yet started, and latches TXEN; a read takes the oldest FIFO entry and previews the next three in bits 8-31; a 32-bit read removes four entries, a halfword or byte one ("DR Read Notes") |
| 4h | SR | TXRDY (no byte waiting), RXRDY (the FIFO not empty), TXIDLE (nothing shifting or waiting), DSR (/ACK low), IRQ; bits 11-31 the baud timer, counting down from (reload × factor) / 2 since the last write of BR, as "BR" describes [R]; the error bits read zero |
| 8h | MR | Bits 0-5 and 8; the factor 1, 1, 16, 64 by bits 0-1 |
| Ah | CR | TXEN, DTR (/CS), RXEN, ACKNOWLEDGE (bit 4), RESET (bit 6), DSRIEN (bit 12), the port (bit 13); bits 4, 6, 14 and 15 read zero; RESET clears every register but BR |
| Eh | BR | The reload value; a 32-bit write at Ch writes it from bits 16-31, as the stub before it did |

A bit lasts `max((reload × factor) AND NOT 1, 1)` CPU cycles, psx-spx's formula for SIO0 [R]: 136 at the kernel's
0088h, so a byte takes 1,088.

## 2. A byte on the wire

- **Start** [C]: a byte starts at the write of DR if TXEN, or the TXEN latched at that write, is set, or as soon as one
  is; a byte written while another shifts waits behind it and starts at its end. psx-spx says the baud timer is
  reloaded when it reaches zero; whether a transfer waits for the timer's next reload is not said, and Diamond does not
  wait.
- **The reply is received when the eighth bit is sampled** [A]: psx-spx's "Signals" says a bit is output on the
  falling edge and sampled on the rising edge that follows, with the clock low for half a bit; so the last bit is
  sampled 7½ bits after the first edge, 1,020 cycles at the kernel's rate, and RXRDY rises then. The transmitter is
  idle a half bit later, at 1,088.
- **The FIFO** holds eight entries. A byte received fills every unused entry, so reads past the data return copies of
  the last byte and then zeros, as "DR Read Notes" describes [R]. psx-spx says the empty flag is set after the sixth
  read of five entries; Diamond clears RXRDY after the fifth, counting the bytes received, since a driver that waits
  for RXRDY before each read would otherwise take a copy for the next byte [C].
- **Reception** happens while /CS is low (DTR) or, once, with RXEN set, which then clears itself [R "CR"].

## 3. /ACK and IRQ7

psx-spx gives three figures, none of them the pad's own: the kernel ignores /ACK pulses in the first 100 cycles after
the last clock and gives up after 100 µs ("Signals"); "normal controllers and memory cards set /ACK=LOW only for around
100 clk cycles" ("Sony Mouse Hardware Bug"); and a memory card's IRQ7 comes "circa 1500 cycles after sending a byte
(counted from the begin of the first bit)" ("Memory Card Read/Write Commands", "Timings") [R]. Diamond therefore pulls
/ACK **412 cycles after a byte's last clock and holds it low for 100** [C], which is the card's 1,500 cycles from the
first bit at the kernel's rate, inside the kernel's window. No program run here measures the pad's own delay; it is
a dispute until one does (D-22).

- SR.7 (DSR) follows /ACK. When /ACK falls with CR.12 set, or CR.12 is set while it is low, SR.9 rises, and with it
  I_STAT bit 7, which is edge-triggered [R "Interrupts"].
- CR.4 clears SR.9 except while /ACK is still low with DSRIEN set: psx-spx says SR.9 "CANNOT be acknowledged while the
  external /IRQ input is still low" [R], so it stays set and gives no new edge.
- The TX and RX interrupts (CR.10, CR.11) are not modelled: psx-spx notes the kernel's driver uses only the DSR
  interrupt [R], and no program run here enables the others.

## 4. The digital pad, and the slots

**Selection** [R "Device addressing"]: DTR asserts /CS of the port CR.13 names; the first byte after it is an address,
01h a controller and 81h a memory card, and only the device addressed answers and pulls /ACK. The reply to the address
byte is FFh (the data line undriven). A change of DTR or of the port starts a new selection.

**The pad** [R "Controllers - Communication Sequence", "Standard Controllers"]: after the address, the command byte is
answered with the ID's low byte 41h, which the pad sends while the command arrives, so it cannot depend on it; then
5Ah, then the switches' low and high bytes, 0 for a button pressed. It pulls /ACK after the address, the command (42h
only), 5Ah and the low byte, and not after the last, as "Signals" says a device must not [R]. Any other command ends
the transfer after the ID byte without /ACK [C: psx-spx describes only the DualShock's further commands]. The motor
bytes are ignored.

**What is plugged in** in this part: port 1's controller, a digital pad whatever the port's setting names, since the
DualShock the setting defaults to answers as one until it is switched to analog mode [R "Analog Mode Note"]; port 2's
controller and both card slots answer as empty, FFh with no /ACK. The rest is stage 6's (§6).

**Input** comes through the core ABI v1's `set_buttons` on port 0, its mask in the bits of the controller descriptor
`v1.rs` declares, which are the pad report's own bits (the descriptor is the one owner: a frontend maps the RetroPad's
controls to them by the descriptor's canonical controls, `Diamond_Plan.md` §4.2). The pad holds them as pressed bits
and inverts them on the wire. They are the host's input, so a state does not keep them.

**`programs/pad.s`** reads the three slots as the kernel's driver does by psx-spx's "Emulation Note" (send, wait about
100 cycles, acknowledge I_STAT and then CR.4, wait for IRQ7 with a timeout, read on RXRDY): port 1 gives FFh 41h 5Ah
F7h BFh with Start and Cross held and IRQ7 after each byte but the last; port 2 and the card slot give FFh five times
and no IRQ7 [M, `the_pad_answers_on_port_1_and_nothing_answers_elsewhere`]. WiseMan's
`A_program_reads_the_pressed_buttons_from_port_1_through_sio0` presses Start, Cross and Up at given frames through the
runner's existing `RomPress` path and reads the replies from RAM, on both of port 1's controller settings.

## 5. amidog's tests, with the pad

Run on the open kernel through `examples/run`'s machine with the buttons held at frames [M, 2026-10-11]. Each test
draws its menu only after clearing VRAM pixel by pixel with 1×1 rectangles, two clears of about 470 frames each on
Diamond, so the menus appear at frame 2,325 (CPX), 1,600 (GPU) and 2,800 (GTE). That cost is the rectangle's fitted
setup of 742 GPU cycles (`Diamond_GPU.md` §8), fitted on 320×240 rectangles; whether a console spends it on a 1×1
rectangle is D-11's question, and these clears are a measurement for it. The menus answer the pad: Down moves the
cursor (`Amidogs_gpu_and_gte_menus_follow_the_pad`). The tests print each failing case on the TTY as "got" and
"wanted" lines, the wanted values their author's, taken on a console.

### 5.1 `psxtest_cpx` 1.1

Started by Start at frame 2,400 (all six tests); finished by frame 12,000. Verdicts read from the screen by
`AmidogScreen` (X exception, F flag, V value, T timing):

| Test | First run | After the fixes of `BlackMoon_R3000.md` D-17 to D-19 |
|---|---|---|
| EXCP | X OK | X OK |
| INTR | X ERROR: 8 cases, EPC 4 bytes early | X OK |
| SLOT | X ERROR (8 cases, TAR), V OK | X OK, V OK |
| SLOT BR | X OK, V OK | X OK, V OK |
| BREAK | X ERROR, 39,146 cases | X ERROR, 7,680 cases (D-20, D-21) |
| ILLEGAL | X OK | X OK |

The F, V and T columns not named are N/A. The three defects found and fixed are in the R3000A's own record. EXCP takes
only SYSCALL and BREAK, whose opcodes have CE bits of zero.

### 5.2 Stage 2's items, closed and open

- **CE**: answered. An address error from LW in a delay slot carries CE 3, its opcode's bits, as Diamond wrote; a
  bus error on a load or store, and a break of the debug unit, carry CE 0 (D-19). Every faulting access in BREAK is
  followed by a NOP, so CE 0 and "CE from the next instruction" cannot be told apart by these cases.
- **A second load to a pending register**: answered. `psxtest_cpu` executes the sequence at 441 places in its MEM DLY
  and MEM ADV groups, among them `lb t3, 2(t0); lb t3, -1(t0); addu t0, zero, t3`, whose ADDU reads the register in the
  second load's delay slot and whose result the test checks [M, an instruction counter over the run]. Diamond passes
  every one; a build in which the first load lands fails 265 cases (`Result: 00000109`) in exactly those groups, its
  value column red [M, the negative control, built and discarded]. So the cancelled first load is confirmed by a
  console's expectations, with `loads.s` and two of three witnesses.
- **COP0 registers 16-31**: still open. Neither `psxtest_cpu` nor `psxtest_cpx` executes an MFC0 of them [M, the same
  counter: none], so no program run here reads them.

### 5.3 `psxtest_gte` 1.4

R1 (every test, the unofficial ones too) at frame 2,800 and Start after it. Its tests take hours of emulated time, as
its page warns; at frame 170,000 the screen showed [M]: BASIC REG and COMPLEX REG (every register, R11R12 to the MAC
registers, 74 rows) with F and V OK; **OFFICIAL TIMING, all 22 commands, T ERROR**; OFFICIAL OPCODE from SQR to DPCL
with X, F and V OK, MVMVA running. The TTY prints nothing for these tests, so their wanted counts are not known.

The timing cases, disassembled from RAM: root counter 2 at the system clock is reset, a loop of 0 or 500 passes runs,
and the counter is read; each pass is the command, 0, 1, 4 or 7 NOPs, a CFC2 FLAG (or an MFC2 MAC1), the load's NOP,
an ADDU, BGTZ and its slot; two more variants issue the command twice. Diamond's cycles a pass [M, (count at 500 −
count at 0) / 500]:

| Command | CFC2 after 0, 1, 4, 7 NOPs | MFC2 after 0, 1, 4, 7 | twice, 4 and 7 apart |
|---|---|---|---|
| SQR, AVSZ3, GPF, GPL | 10, 10, 10, 13 | 12, 12, 12, 15 | 15, 23 |
| AVSZ4, OP | 11, 11, 11, 13 | 13, 13, 13, 15 | 17, 23 |
| NCLIP, DPCS, INTPL, DCPL, MVMVA | 13, 13, 13, 13 | 15, 15, 15, 15 | 21, 23 |
| CC | 16, 16, 16, 16 | 18, 18, 18, 18 | 27, 29 |
| CDP | 18, 18, 18, 18 | 20, 20, 20, 20 | 31, 33 |
| NCS | 19, 19, 19, 19 | 21, 21, 21, 21 | 33, 35 |
| RTPS | 20, 20, 20, 20 | 22, 22, 22, 22 | 35, 37 |
| NCCS, DPCT | 22, 22, 22, 22 | 24, 24, 24, 24 | 39, 41 |
| NCDS | 24, 24, 24, 24 | 26, 26, 26, 26 | 43, 45 |
| RTPT | 28, 28, 28, 28 | 30, 30, 30, 30 | 51, 53 |
| NCT | 35, 35, 35, 35 | 37, 37, 37, 37 | 65, 67 |
| NCCT | 44, 44, 44, 44 | 46, 46, 46, 46 | 83, 85 |
| NCDT | 49, 49, 49, 49 | 51, 51, 51, 51 | 93, 95 |

Each pass is the command's cycles by psx-spx's table plus the pass's other five instructions, whenever the read comes
before the command ends: the interlock of `BlackMoon_GTE.md` D-13, counted from the command's fetch. A console's
expectations reject every row, so D-13's rule, or the loop's own cost on the bus, is not the console's; which, the
screen does not say. This is D-13's evidence, recorded there for the step that takes it up.

### 5.4 `psxtest_gpu` 1.7

Start at frame 1,600 (every test but DUMP). By frame 160,000 it was still running its first groups, drawing and
comparing against its software model, with no TTY output; its verdicts wait on a longer run.

## 6. What stage 6 adds

The DualShock with its configuration commands, analog mode, the Analog button and its LED, and its motors' state; the
memory cards with their commands, flag byte and timing as battery files; port 2's devices; the multitap; the open
kernel's pad functions reading SIO0. Each is a device behind the seam of §4.

## 7. Reading log

Every file opened for this part, 2026-10-11:

- **psx-spx** `6d7d1bc`: `serialinterfacessio.md` lines 1-196; `controllersandmemorycards.md` lines 40-178 (overview,
  addressing, /ACK, signals), 329-445 (the communication sequence, the ID, the standard controllers, the analog note),
  520-533 (the mouse's /ACK note), 1201-1260 (the configuration commands' list and command 42h) and 2433-2454 (the
  card's timings); `interrupts.md` lines 25-50; `unpredictablethings.md` lines 1-30; `cpuspecifications.md` lines
  529-600 and 736-830 (COP0's registers, Cause, TAR, the garbage registers and DCIC); a search of
  `controllersandmemorycards.md`, `interrupts.md`, `timers.md`, `iomap.md` and `unpredictablethings.md` for SIO0's
  names.
- **The IDT R30xx manual**, as text by `pdftotext`: chapter 13's "Additional Hazards" and "Non-obvious Hazards" (the
  latency of MTC0 to SR), found by a search for "hazard".
- **amidog's tests**: the three `.txt` files; the programs' own code, disassembled from RAM while they ran with
  `emusen-mips` (their pad routine, their VRAM clear, the INTR, SLOT and BREAK cases named in §5, the GTE timing loop);
  their screens and TTY.
- **`psxtest_cpu`**: its code at the second-load sites, disassembled from RAM.
- **EmuSen's own**: `Diamond_Plan.md` §1, §3-§4, §5.8, §7, §8, §11-§13; `Diamond_Native.md` §10-§12; `Diamond_GPU.md`
  §7, §8, §10 and §15-§18; `Diamond_OpenKernel.md` §1-§4.2, §4.12-§4.13 and §6-§11; `Diamond_Disputes.md`'s rules and headings;
  `BlackMoon_R3000.md` §8 and §10; `DOCS.txt`; Diamond's crate, blackmoon-r3000's `exec.rs`, `cop0.rs`, `lib.rs`,
  `state.rs` and `boundary.rs`; WiseMan's runner and `DiamondTests`.
- **Not opened**: any emulator's source, any firmware or listing, `sysdir/`, `Diamond_References.md`, PSn00bSDK's
  libraries, ps1-tests' `common/`. No reference emulator was run.
