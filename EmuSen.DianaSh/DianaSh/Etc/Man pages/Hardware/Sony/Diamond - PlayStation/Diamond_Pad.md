# Diamond_Pad — SIO0 and the controllers

*Written 2026-10-11.* This page records Diamond's controller and memory card port, SIO0, and the devices on it. Its
first part, a digital pad on port 1, was **brought forward from stage 6** (decided 2026-10-11, `Diamond_Plan.md` §7)
so that amidog's CPX, GPU and GTE tests, which wait at a menu for Start, could run; §5 records what they showed. The
rest of **stage 6** followed the same day, when the stage had its go (§6-§11): the DualShock, the memory cards as
battery files, port 2, the multitap and the open kernel's pad functions. The marks are the plan's: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from the source named; **[A]**
argued; **[C]** a choice where the documents are silent.

The code is `src/sio0.rs` (the port and its slots), `src/pad.rs` (the digital pad and the DualShock), `src/card.rs`
(the memory card) and `src/multitap.rs`, one job each: the port shifts bytes, keeps the FIFO, routes the address byte
to a slot's device and raises the interrupt; a device answers one byte at a time and says whether, and how late, it
pulls /ACK.

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

**What is plugged in**: in the part brought forward, port 1's controller only, as a digital pad, port 2 and both card
slots empty. Since stage 6 each port holds the controller its setting names and the card its setting names (§6), or a
multitap (§9); an empty connector answers FFh with no /ACK.

**Input** comes through the core ABI v1's `set_buttons`, its mask in the bits of the controller descriptor
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

R1 (every test, the unofficial ones too) at frame 2,800 and Start after it; "ALL DONE" by frame 480,000, about two
and a quarter hours of emulated time [M]. Read from the screen:

| Group | Rows | Verdict |
|---|---|---|
| BASIC REG | 1 | F and V OK |
| COMPLEX REG | every register, R11R12 to MAC3 | F and V OK |
| OFFICIAL TIMING | the 22 commands | **T ERROR on every one** |
| OFFICIAL OPCODE | SQR to DPCL, 21 commands | X, F and V OK |
| OFFICIAL OPCODE | MVMVA | X and V OK, **F ERROR** |
| UNOFFICIAL TIMING | 0x00 to 0x3F, the numbers psx-spx does not list | **V and T ERROR** |
| UNOFFICIAL OPCODE | (no rows drawn) | — |
| TOTAL | | X OK, F, V and T ERROR |

The TTY prints nothing for these tests, so their wanted values are not known. MVMVA's flag error and the unlisted
numbers' values bear on `BlackMoon_GTE.md` D-14 and D-16, the timing on D-13.

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

Start at frame 1,600 (every test but DUMP); the results screen from frame 295,000, "ALL DONE", three pages turned by
R2 with Right [M]. The TTY prints nothing for them. Read from the screen:

| Group | Rows | Verdict |
|---|---|---|
| BASIC | REG, IRQ, FBRECT, POLYLINE, TPAGE, MOVE | REG V ERROR; POLYLINE F ERROR; MOVE V ERROR; IRQ, FBRECT, TPAGE OK |
| COMMON VRAM | WRITE, MOVE | OK |
| PRIMITIVE FBUF | 02 RECT | OK |
| POLY MONO, POLY GOURAUD | 20h-2Fh, 30h-3Fh, 32 rows | **F ERROR on every one**, V OK |
| LINE MONO, LINE GOURAUD | 40h-4Fh, 50h-5Fh, 32 rows | **F and V ERROR on every one** |
| RECT W×H, 1×1, 8×8, 16×16 | 60h-7Fh, 32 rows | OK |
| MOVE 1/2, 2/2; VRAM 1/2, 2/2 | 80h-BFh, 64 rows | OK |
| XYFLIP | 64h-7Fh, 16 rows | F OK, **V ERROR on every one** |
| SPECIAL OFFSET | H and V, by primitive | fills OK; rectangles, polygons and lines mostly V ERROR |
| POSITION, SIZE | by primitive | OK but some lines (V ERROR) |
| AREA | 60h, 68h, 70h, 78h, 20h, 28h, 30h, 38h, 40h, 50h | **V ERROR**; the textured 64h, 6Ch, 74h, 7Ch, 24h, 2Ch, 34h, 3Ch OK |
| TOTAL | | X OK, F and V ERROR |

The pattern is the GPU's to take up (`Diamond_Disputes.md` D-23): every polygon and line fails its flag column, every
line and every flipped rectangle its values, and in the drawing area's test only the untextured primitives fail.

## 6. What a frontend gets (stage 6)

- **Ports.** Host ports 0 and 1 are the console's ports 1 and 2; with a multitap on a port, its slots B-D are host
  ports 2-4 (port 1's) or 5-7 (port 2's), holding controllers of the port's kind [C: one setting a port]. Machine info
  lists them.
- **Settings**: `pad1`, `pad2` (DualShock, digital controller, or nothing on port 2), `tap1`, `tap2` (a multitap or
  none) and `card1`, `card2` (a memory card or none; slot 1 holds one by default, slot 2 none, the plan's §4.4).
- **The DualShock's descriptor** gains bit 16, "Analog", with no canonical control: the RetroPad has no Analog
  button (`PadButton` ends at R3), so a frontend offers it unbound, as the descriptor's rule for an extra says; a pad
  combination is a frontend's choice and is not made here. The sticks arrive through `set_axis` (`AXES`), −1 to 1
  mapped to 00h-FFh with 0 at 80h [C].
- **Battery files**: the card in slot 1 (or port 1's multitap's slot A) is file 0, suffix `.mcr`; slot 2's is file 4,
  `.2.mcr`, since files 1-3 are the BIOS's [C: the plan's §4.4 said file 1, which the BIOS's numbers already held].
  Each is the raw card, 131,072 bytes; a file of another length is refused with words; a slot with a card and no file
  holds an unformatted card of zeros [C]. A write through the protocol marks the card changed, `BATTERY` is raised once,
  and `battery_saved` clears it (`BATTERY_DIRTY`). The multitap's slots B-D hold no cards [C: no file numbers yet].
- **Rumble** stays in the core: the motors' state is kept and in the state, and reaches no frontend, since v1.0 has no
  output event (the plan's §4.2).
- **The state**: `DMND` version 4 adds each device's state and each card's contents; a card loaded from a state is
  marked changed so that the host writes what the state holds [C].

## 7. The DualShock (SCPH-1200)

psx-spx's "Standard Controllers", "Analog Mode Note", "Configuration Commands" and "Vibration/Rumble Control" [R]:

- **Modes.** It powers on in digital mode (ID 41h, one halfword, L3 and R3 reading released); analog mode, the LED red,
  is ID 73h with three halfwords, the right stick then the left, 00h left or up. The Analog button switches between
  them unless command 44h locked it, and stops and locks the motors ("Caution 2"); after configuration commands were
  used, it replaces the 5Ah byte by 00h ("Caution 3").
- **Configuration mode** is entered and left by 43h's parameter, ID F3h and always nine bytes: 42h (the buttons and
  sticks, even in digital mode), 43h, 44h (LED 0 or 1, other values ignored; Key 3 locks), 45h (type 01h, the LED),
  46h, 47h, 48h, 4Ch with the documented constants by their parameter, 4Dh (returns the old mapping, sets the new), and
  40h-4Fh otherwise zeros. A parameter takes effect as its byte arrives [C], so 46h's later bytes follow its index.
- **Motors.** With the mapping all FFh and no configuration used, the one-motor method: the small motor on for xx in
  40h-7Fh with yy's bit 0. Once configuration was used, 4Dh's mapping alone: 00h the small motor's bit 0, 01h the large
  motor's speed.
- **Not modelled** [C]: the watchdog that resets a configured pad after about a second without reads, the DualShock 2's
  commands (40h, 41h, 4Fh) and pressure bytes, and the LED-green "analog stick" mode, which psx-spx says the DualShock
  does not have.

## 8. Memory cards

psx-spx's "Memory Card Read/Write Commands" and "Timings" [R]:

- **Read (52h)**: FLAG, 5Ah, 5Dh, the sector's MSB and LSB received (00h, then the MSB echoed), 5Ch, 5Dh, the
  confirmed MSB and LSB, 128 bytes, the checksum (MSB xor LSB xor the data) and 47h. A sector of 400h or more is
  confirmed as FFFFh and the transfer ends, as an original Sony card does.
- **Write (57h)**: FLAG, 5Ah, 5Dh, the sector, 128 bytes and the checksum received with the previous byte echoed, 5Ch,
  5Dh, and the end byte: 47h, 4Eh for a bad checksum, FFh for a bad sector. The sector is stored only on 47h.
- **Get ID (53h)**: FLAG, 5Ah, 5Dh, 5Ch, 5Dh, 04h, 00h, 00h, 80h.
- **Any other command** ends the transfer after FLAG.
- **FLAG** is 08h at power-on (bit 3, the directory not read), cleared by a good write; bit 2 is not set, as psx-spx
  says the write never sets it.
- **Timing**: /ACK as every device's (§3), and **31,000 cycles later after a read's 5Ch byte**, psx-spx's "additional
  delay" after the seventh byte, which the table marks "late /ACK after this byte-pair" [R; C on which byte]. A write
  costs only its bytes, as psx-spx describes ("the data is written on the fly").

**`programs/card.s`** writes sector 0100h with the bytes 3n, reads it back, takes the ID twice and writes 0101h with a
bad checksum, each byte sent as the kernel's driver sends it: the write ends 47h, the read returns the bytes and the
checksum with FLAG 00h, the ID 04h 00h 00h 80h, the bad write 4Eh and stores nothing, and IRQ7 comes after every byte
but each command's last, the read's late one within the timeout [M, `a_card_is_written_and_read_through_the_protocol`].
The card it wrote is kept outside the repository at `~/.cache/emusen/probe/diamond/cards/diamond-card-1.mcr` (131,072
bytes, MD5 `e048b2fc6569436e819e476472857aee`, a `README.txt` beside it): frame 0100h holds 00h, 03h, ... 7Dh and every
other byte is 00h, an unformatted card, since the on-card file system is stage 10's. Its round trip against a
reference's card is a separate step.

## 9. The multitap (SCPH-1070)

psx-spx's "Controller and Memory Card Multitap Adaptor" [R]: plugged into a port, it takes the port's whole connector.

- **Addresses** 01h-04h reach the controllers in slots A-D and 81h-84h the cards (method 2); an empty slot answers as
  an empty port.
- **Slot A's long read** (method 1): a 1 in bit 0 of the third byte of an access to slot A requests that the next
  access return 80h, 5Ah and each slot's four halfwords, a controller's read padded with FFh and an empty slot all FFh,
  35 bytes in all; the table psx-spx gives is followed exactly: after a requesting access that returned slot A, the
  long read; after one that returned the long read, the four-byte "garbage" (80h, 5Ah, slot A's ID byte); after
  garbage, the long read. An empty slot A ends the access at its address.
- **What the slots' controllers receive** during a long read is not documented; each is read as by 42h with zero motor
  bytes [C].

**`programs/multitap.s`** reads slot A with the request, then the long read, slot C by its address and slot A's card's
ID [M, `the_multitap_answers_all_four_slots`].

## 10. The open kernel's pad functions

`kernel/src/pad.rs`, written as a replacement writer from psx-spx's "BIOS Joypad Functions" and the driver its
"Emulation Note" and "Signals" describe, nothing else (`Diamond_OpenKernel.md` §4.13): on each vblank, with the pads
enabled, each port is read into its buffer: CR selects the port; each byte is sent, about 100 cycles waited, IRQ7
acknowledged in I_STAT and then CR.4, the next IRQ7 awaited for about 100 µs; the reply read on RXRDY. The address
01h, the command 42h, then 00h; with /ACK after each and 5Ah second, status 00h, the ID, and the halfwords the ID's
low nibble counts (16 for 0), up to 22h bytes; otherwise status FFh. InitPAD2 now also resets SIO0 and sets MR 0Dh and
BR 0088h [C on where: no document says which call sets them; without them the first program read at a rate of one
cycle a bit and missed its own IRQ7]. StartPAD2 also enables vblank in I_MASK [A]: Peter Lemon's `Cube`, written for a console, calls only
PAD_init2 and then waits for its button word, which it could never see otherwise; with it, the cube answers the pad
(holding Right for 200 frames turns its D face into view) [M, `examples/run`'s machine]. The kernel sends no multitap request [C], so its buffers hold slot A.

**`programs/kernel-pad.s`**, run on the open kernel: InitPAD2, StartPAD2, vblank enabled, ExitCriticalSection; port 1's
buffer then reads 00h 41h F7h BFh with Start and Cross held, and 00h 73h and the sticks once the Analog button is
pressed; port 2, empty, FFh [M, `the_open_kernels_pad_functions_read_the_pads`]. The image's SHA-256 is now
`135549cd26268e0a50f56649ac4a1cdfdf6287ca2660698023648bc3bf011629`.

## 11. The stage 6 gate

The plan's §7 row 6, item by item [M, 2026-10-11]:

| Item | Result |
|---|---|
| The open kernel's pad functions read the pad on a test program | **Met**: `kernel-pad.s`, digital and analog reads, an empty port FFh (§10) |
| Cards read and written through the protocol | **Met** on Diamond: `card.s`, a write, a read back, the ID, a bad checksum (§8) |
| Their files the raw 128 KiB form | **Met**: battery files 0 and 4 of 131,072 bytes through the core ABI, tracked until saved; WiseMan's `The_memory_cards_are_battery_files_and_a_multitap_adds_ports` |
| Round-tripped with a reference's card | **Not run here** by decision of 2026-10-11 (no reference emulator in this step): the card `card.s` wrote is kept for it (§8) |
| The multitap's four ports on a test program | **Met**: `multitap.s`, the long read with an empty slot B, slot C by address, slot A's card (§9) |
| The DualShock's modes and rumble state | **Met** by the crate's cases: digital and analog reads, configuration commands 43h-4Dh, the LED, the lock, both motor methods, the Analog button's resets (§7) |

## 12. Reading log

Every file opened for this part, 2026-10-11:

- **psx-spx** `6d7d1bc`: `serialinterfacessio.md` lines 1-196; `controllersandmemorycards.md` lines 40-178 (overview,
  addressing, /ACK, signals), 329-445 (the communication sequence, the ID, the standard controllers, the analog note),
  520-533 (the mouse's /ACK note), 1201-1540 (the configuration commands, rumble, the DualShock 2's commands), 179-328
  (the multitap) and 2352-2475 and 2607-2630 (the card's commands, flag, timings, data size and raw images);
  `kernelbios.md` lines 1598-1675 ("BIOS Joypad Functions", for the kernel); psx-spx's controller chapter at
  445-500 (the analog range and the green-LED mode); `interrupts.md` lines 25-50; `unpredictablethings.md` lines 1-30; `cpuspecifications.md` lines
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
