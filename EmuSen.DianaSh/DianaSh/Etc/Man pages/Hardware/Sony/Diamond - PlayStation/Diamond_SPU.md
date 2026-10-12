# Diamond_SPU — the sound processing unit

*Written 2026-10-11, at stage 8.* The plan's §7 row 8 (`Diamond_Plan.md`): the voices, ADPCM, the Gaussian
interpolation, ADSR, the sweeps, noise, pitch modulation, the reverb, the capture buffers, the IRQ, the transfers and
CD audio in, with the tester's go of 2026-10-11. The SPU is `src/spu/` (the registers, the transfer, the mix and the
clock in `mod.rs`, a voice in `voice.rs`, the envelope step in `envelope.rs`, the reverb in `reverb.rs`, the two tables
in `tables.rs`, the state in `state.rs`, its rules' tests in `tests.rs`); the bus's part is in `bus.rs` (`sync_spu`,
`run_spu`, `spu_event`, DMA4's piece). Its gate's programs are `programs/spu-voices.s`, `spu-capture.s` and
`spu-dma.s`.

The marks are the plan's: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44), the tool named;
**[R]** read from a source; **[A]** argued; **[C]** a choice. The disputes this stage opened are D-34 to D-40 in
`Diamond_Disputes.md`; D-29 carries this stage's step.

---

## 1. What it is written from

Every rule below is psx-spx's, at the pinned `6d7d1bc`, from `soundprocessingunitspu.md` read whole (its overview, the
ADPCM samples, the pitch and the Gaussian interpolation, the volume and ADSR generator with its envelope pseudocode, the
voice flags, the noise generator, ATTR and STATX, the memory access with its transfer modes, the interrupt, the reverb's
registers, formula, notes and examples, the unknown registers, and the section on the state machine as a logic
analyser showed it), with the XA-ADPCM chapter of `cdromformat.md` for the filters' coefficients and the shift rule
psx-spx calls the same, and `dmachannels.md` for DMA4 [R]. Where psx-spx stops, the rule is a choice marked [C], and
where the referees were asked, its dispute is named.

**On what this was written without.** No emulator's source and no library's (PSn00bSDK's `psxspu`, PsyQ's libspu) was
opened. A writer's general knowledge of how SPUs are commonly emulated may derive from emulator code met before this work,
and so no such knowledge was taken as a rule: each rule here is psx-spx's or the outcome of a measurement against the
referees run as black boxes (§10), and where the core's first build followed a reading that the referees then
contradicted (§10.2), the change is recorded with the evidence.

## 2. The machine around it

- **Registers** (psx-spx "SPU I/O Port Summary"): 1F801C00h–1F801FFFh as halfwords. Writes are kept as written and read
  back as written, but for ENVX (the envelope's level, writable), LSAX (the loop address, which the loop start flag also
  writes), ENDX, STATX, MVOLX and the voices' VOLX (the current volumes, written only by the SPU), and 1F801E80h onward,
  which reads FFFFh ("Read: FFh-filled", `iomap.md`). An 8-bit write to an odd address is ignored and one to an even
  address is a halfword's, a 32-bit access two halfwords, low first (psx-spx "SPU Bus-Width"). ps1-tests' `spu/test`,
  which writes every bit of MVOLL and MVOLR at 16 and 32 bits and reads it back, prints no mismatch (§10.4).
- **The clock**: sample n is made at cycle 768·n, so that a frame's samples are its cycles over 768, as the silence of
  the stub had them. The SPU is brought to the present at each register access, at each DMA4 piece, at each frame's
  end, and, while IRQ9 can be raised, by an event at each sample and each transfer slot; it runs sample by sample
  (`Spu::run`). A register's effect waits for the next sample, the "1/44100 seconds until it has actually realized the
  new value" of psx-spx's "Unstable and Delayed I/O": key on and off, ATTR's bits 0–5 as STATX shows them, a direct
  volume.
- **The output** is a `SampleQueue` of `emusen-native` (the shared queue of every Rust core), stereo at 44,100 Hz,
  bounded by the host's limit, drained by the core ABI's `drain_audio`. It is the host's, not machine state.
- **CD audio in** is stage 7's ring (`Diamond_CDROM.md` §7): the SPU takes one frame a sample, the CD-ROM controller
  brought to the same cycle first so that a sector due before it has delivered its frames.

## 3. The voices

### 3.1 SPU-ADPCM

A block of 16 bytes holds a header (shift in bits 0–3, filter in bits 4–6), flags and 28 four-bit samples (psx-spx
"Sample Data"). A sample is the nibble shifted to the top of 16 bits and right by the shift, plus the filter's
prediction from the two samples before, clamped. **The prediction is each product shifted right by six apart** —
(old·f0 >> 6) + (older·f1 >> 6) — with the coefficients of psx-spx's "Pos/neg Tables" for filters 0–4 (D-34). psx-spx
gives the XA decoder's (old·f0 + older·f1 + 32) / 64 and says the SPU's is "reportedly" the same; the first build
followed it, and the referees' agreement against it is D-34's. Shifts 13–15 act as 9, as psx-spx says XA's do, and
filters 5–7 predict nothing (D-40, open).

**Decoded a word at a time as the pitch counter reaches it** [C, from psx-spx's state machine section: each sample "a
read to the header of the currently played ADPCM block, one to the current 16 bit of the ADPCM"]: the header and the
word are read from sound RAM when the word's first sample is needed, so that a stream written into a block while it
plays is heard as a console would hear it. At a block's end the words the counter skipped are decoded so that the
filter and the interpolation keep their history.

### 3.2 The Gaussian interpolation, and where its table came from

The four most recent samples, oldest to newest, weighted by gauss[0FFh−i], gauss[1FFh−i], gauss[100h+i] and gauss[i],
with i the pitch counter's bits 4–11 (psx-spx "4-Point Gaussian Interpolation"). **The four products are summed and
then shifted right by 15** (D-35), where psx-spx shifts each.

The 512 entries were **transcribed from psx-spx by a script**, not by hand: the page's lines 252–315 (the table between
its code fences, the comments after `;` removed) parsed for `-?hex h` and written to `tables.rs`. The script checked
the property psx-spx states, that each four entries taken together sum to 7F7Fh–7F81h, and they do (min 7F7Fh, max
7F81h) [M]. The reverb's 39-tap filter was transcribed the same way from "Reverb Buffer Resampling", and its taps sum
to 7FFEh [M], its even taps to 3FFEh and its odd to 4000h, which is why its upsampling doubles (§5.2).

### 3.3 Pitch, and pitch modulation

The counter advances by PITCH each sample, at most 4000h; with PMON's bit set for a voice above 0, PITCH sign-extended
from 16 bits times the voice before's output plus 8000h, shifted right by 15 and masked to 16 bits, psx-spx's
"hardware glitch" included (psx-spx "Pitch Counter"). **The voice before's output is this sample's** [C]: voices are
processed in order, as psx-spx's state machine section lists them; T5 of `spu-capture.s` (§10.2), which modulates voice
1 by voice 0, equals DuckStation's sample for sample.

### 3.4 Loop flags, ENDX and LSAX

At a block's end its flags are read: loop end sets the voice's ENDX bit and jumps to LSAX, and without loop repeat
also forces the release with the envelope at zero; otherwise the next block (psx-spx "Flag Bits"). A block's loop
start flag copies its address to LSAX as the block begins [C]. **A key on leaves LSAX as it was** [C]: psx-spx does not
say, and T7 (§10.2), which writes 1234h to LSAX and keys the voice on at a block with no loop start flag, reads 1234h a
frame later on Diamond and on every referee that keeps the register. ENDX's bit clears at the key on (psx-spx "ENDX").
Every voice keeps reading and decoding whether keyed or not ("all voices are permanently reading data from SPU RAM",
psx-spx "Voice Interrupt"), so its ENDX bit and the IRQ address see it.

### 3.5 Key on and key off

A key on, at the next sample, starts the block at SSA × 8 from its first sample, clears the decoder's and the
interpolation's history, and starts the envelope's attack at zero; a key off starts the release. Both written in one
sample: the key off first, then the key on [C]. **The envelope steps after the sample it scales** (D-36): the key on's
own sample leaves at level zero, the first step shows in the next, as DuckStation's capture buffers record it (T11).

### 3.6 Noise

psx-spx's "SPU Noise Generator" as written: a timer less the step (4–7) each sample, and below zero a new bit, the
parity of bits 15, 12, 11 and 10 inverted, with the timer recharged by 20000h >> shift once and again if need be. **A
write of ATTR that changes the shift or the step starts the timer again** (D-37): read literally, the pseudocode carries
a count of up to 20000h from a slow rate into a fast one, a second and more at shift 0 before the new rate is heard,
where every referee changes rate at once. A voice with NON's bit set takes the level as its sample in place of the
interpolation; its ADPCM still runs.

## 4. The envelope and the sweeps

One step for ADSR and for a swept volume, psx-spx's "Envelope Operation depending on Shift/Step/Mode/Direction"
(`envelope.rs`): the step 7 − step, inverted when decreasing differs from the phase, shifted by 11 − shift; the
counter's increment 8000h >> (shift − 11); the exponential increase above 6000h and the exponential decrease scaled by
the level; the increment at least one unless the rate is all ones; a step when the counter's bit 15 sets; the clamps by
direction and phase. Three choices where the pseudocode stops [C]: the counter starts again from zero at each step (for
its power-of-two increments the same as keeping the remainder); the exponential decrease's product is shifted
arithmetically, so that a step that rounds below −1 still reaches zero; a phase ends after its step, attack at 7FFFh
into decay, decay at or below (N+1) × 800h into sustain. The last decides that the decay takes one step from 7FFFh even
when the sustain level is 8000h, and all five referees read ENVX 3FFFh where T1's voices sit after an attack and a
decay of shift 0 to level 15 (§10.2).

ADSR's rates per phase are psx-spx's ADSR1 and ADSR2: attack shift and step with its mode; decay shift with step 0,
decreasing, exponential; sustain shift, step, direction and mode; release shift, step 0, decreasing, its mode. A volume
register with bit 15 clear is its bits 0–14 doubled, with it set a sweep whose rate is its bits (psx-spx "Sweep Volume
Mode"); a voice's output through each side is (out × VOLX) >> 15, and the mix through MVOL the same.

T2 and T3 (§10.2), an attack, a decay and an exponential sustain, then an exponential release, equal Beetle PSX's,
SwanStation's and DuckStation's capture buffers sample for sample over 496 samples each.

## 5. The reverb

### 5.1 The formula and its accesses

psx-spx's "SPU Reverb Formula" (`reverb.rs`), the left side on one 44,100 Hz cycle and the right on the next, the
buffer address advancing by two after the right (`MAX(ESA, (BufferAddress+2) AND 7FFFEh)`), every address relative to
it in eight-byte units, wrapped within ESA..7FFFFh, every product shifted right by 15 and every value written saturated
[R]. **The accesses in the order psx-spx's state machine section tables them** [R]: the same side's two reads and the
other side's, then the same side's write; the different side's read before, the first comb, then its write; the three
other combs; both all-pass reads; then both all-pass writes. With ATTR's master enable clear the writes become reads,
as that section expects, which gives psx-spx's "Reverb Disable" behaviour (with vAPF2 zero the output is the buffer at
mAPF2 − dAPF2) without a case of its own; a unit test checks it (`the_reverb_output_reads_the_work_area_…`). vIIR at
−8000h negates the value written (psx-spx "Bug"). Every reverb access is watched for the IRQ address [C], since psx-spx
says only that some are.

### 5.2 The 39-tap filter

The input is the voices with EON's bit and CD audio with ATTR's bit 2, at 44,100 Hz into a ring of 64; each side's
step takes the newest 39 through the filter and shifts by 15, then by vLIN or vRIN. The output is each side's value
times EVOL at 22,050 Hz, placed among zeros at 44,100 Hz and filtered again, shifted by 14 since half the taps meet
zeros [C, from the taps' sums in §3.2]. Which instants the two sides' inputs are taken at is D-39: here a sample apart,
each at its own cycle.

## 6. The transfers

### 6.1 The FIFO and its slots

A FIFO of 32 halfwords (psx-spx "DATA?": "max 32 halfwords"; "DMA Channels": a 16-word buffer). **One halfword moves
between it and sound RAM every 32 cycles** [C, D-38]: psx-spx's state machine section finds 24 voice slots in each
sample with an "unrelated read" that it argues is the transfer's, which is 32 cycles a slot and 2,116,800 bytes a
second, and ps1-tests' `spu/memory-transfer` expects about 16 cycles a byte, its author noting that nocash's 4 cycles a
word does not match, with the console passing a window of 1,638 to 18,022 cycles for a KiB [R its source and log].
Manual write drains the FIFO while ATTR's mode is 1; DMA write mode (2) requests DMA4 when the FIFO is empty; DMA read
mode (3) fills the FIFO from the transfer address and requests DMA4 when it is full. STATX's bits 8 and 9 are the two
requests, bit 7 either, bit 10 busy while the slots run, bit 11 the capture buffers' half when RAM_CTRL's bits 2–3
allow. Entering or leaving the read mode empties the FIFO [C]: found by `spu/memory-transfer`, whose DMA write test
read zeros while the FIFO still held its read test's halfwords and the write mode wrote them first.

A word written to a full FIFO (a burst in DMA sync mode 0 longer than 16 words) writes the oldest halfword at once, and
a word read from an empty one reads sound RAM at once [C]: nothing is lost, and the timing of such a transfer is
DMA4's own. DMA reads are the stable kind of psx-spx's "DMA-Read (stable reading, with DEV4_CTRL.bit24-27 = nonzero)";
the FFFFh halfwords of the unstable kind and RAM_CTRL's other bank sizes are not modelled.

### 6.2 What the referees take, and the console's window

`spu-dma.s` times the transfers by timer 2 on the system clock [M, cycles for 1 KiB]:

| | Diamond | Beetle PSX | SwanStation | DuckStation | PCSX-ReARMed | PCSX-Redux |
|---|---|---|---|---|---|---|
| DMA4 write, blocks of 16 words | 15,419 | 12,282 | 7,850 | 7,640 | 4,116 | 149 |
| DMA4 write, blocks of 1 word | 16,534 | 12,264 | 7,054 | 6,417 | 4,116 | 149 |
| DMA4 write, one burst of 256 words | 1,068 | 12,266 | 283 | 283 | 12 | 17 |
| DMA4 read, blocks of 16 words | 16,402 | 12,264 | 8,137 | 8,095 | 4,116 | 149 |
| ATTR's manual write mode to STATX showing it | 582 | 606 | 30 | 30 | 14 | 19 |
| then STATX busy until 32 halfwords are written | 993 | 28 | 403 | 28 | 11 | 14 |

Every referee but PCSX-Redux lies within the console's window; Redux fails `spu/memory-transfer`'s timing tests, as
§10.4 shows. Beetle PSX takes 48 cycles a word in every case, DuckStation about 30 in sync mode 1 and the DMA's own rate
in a burst. The mode's latency, the next sample, is Beetle PSX's too. D-38 records the rest open.

### 6.3 DMA4 letting the other channels move

**A channel in sync mode 1 waiting for its device's request lets the others move** (`bus.rs`): DMA4 between the FIFO's
drains gives up the controller, and its request's rise arbitrates again, as the MDEC's channels already did. The first
build held DMA4 active through its waits, so that no other channel moved for the length of a sound bank's upload;
Crash Bandicoot then stopped after its first uploads in an address error in the kernel, seen from the processor's
Cause and EPC alone, and the CD-ROM's channel held back is the likeliest cause [A]. The defect was shown first by a test
that starts DMA6 while DMA4 waits and finds it unmoved until DMA4 ends
(`a_channel_waiting_for_its_request_lets_another_move`, failing before, passing after), and with the fix Crash reaches
its title screen again (§11). psx-spx's "DMA Channels" speaks of a channel "getting interrupted by a higher priority
DMA channel" [R]; the channel's priority between the waits is not modelled beyond DPCR's choice of the next.

## 7. IRQ9

ATTR's bit 6 with bit 15 arms it; clearing either acknowledges (psx-spx "ATTR"). An access to the eight-byte unit at
IRQA × 8 raises STATX's bit 6 and I_STAT's bit 9 once: each voice's header and current word each sample (§3.1), the
capture buffers' writes when RAM_CTRL's bits 2–3 are not both zero ("Capture IRQs do NOT occur if 1F801DACh.bit3-2 are
both zero"), every reverb access, and the transfer's slots (psx-spx "Data Transfers") [R, the units and the reverb's
accesses C]. `spu-capture.s` synchronises to the capture buffer's IRQ at IRQA 100h on every referee, which is how its
records are aligned (§10.2).

## 8. Capture, CD audio and the mix

Each sample writes the CD input's left and right before their volume at 000h and 400h, voice 1's and voice 3's output
after the envelope at 800h and C00h, at the same position of 512 [R psx-spx "SPU Memory layout"]. The mix: the voices'
sums through their volumes, CD audio times AVOL when ATTR's bit 0 is set, the reverb's output; ATTR's bits 15 and 14
both set let the voices and the reverb through, CD audio passes either way ("Don't care for CD Audio") [R, the reverb's
muting C]; the sum clamped, through MVOL, clamped. External audio (BVOL) is silence.

## 9. The state

`DMND` version 8, the SPU's group `Bus.Spu` after the GPU's: the 512 registers, each voice's own (its seven registers,
address, counter, decoded count, phase, 31 samples, two of history, three envelopes and output), ENDX and the pending
key ons and offs, the applied mode, the IRQ flag, the noise, the capture position, the main volume's envelopes, the
reverb's address, filters and phase, the transfer's address, FIFO and next slot, and the next sample's cycle. Sound RAM
is the state's `SPURAM` block as before; the output queue is not kept, and a load empties it. **3,792,964 bytes** with a
DualShock on port 1, 3,136 more than version 7 [M, `the_version_8_layout_is_pinned`].

## 10. The gate

### 10.1 The voices against the references' audio

`spu-voices.s` plays nine scenes on voice 1, each from silence after three frames at main volume zero: A a linear ADSR,
B an exponential one, C the volume sweeps, D a 16-block wave of every filter and shift at eight pitches, E pitch
modulation, F noise at three rates, G a one-shot and a start inside the wave, H a pulse into psx-spx's "Room" reverb.
The scenes begin 400 frames after the program, and each referee's recording was cut at frame 400 without being listened
to or read before it, after its own RAM showed the program's marker by then, so that nothing a BIOS played was heard
(`psxreferee.py wav`, the cut a script's, the first recordings, made before the marker, deleted unread). `psxaudio.py`
compares the scenes sample for sample [M]:

| Scene | SwanStation, unscaled | Beetle PSX, after its ×0.75 | PCSX-ReARMed, after its ×0.75 |
|---|---|---|---|
| A linear ADSR | 71.5% equal, largest 5 | largest 1,147, mean 10.8 | mean 874 |
| B exponential ADSR | 91.4%, largest 28 | largest 130, mean 20.8 | mean 67 |
| C sweeps | 61.7%, largest 893, mean 1.9 | mean 5.1 | mean 381 |
| D the wave at eight pitches | exact for 12,000 samples, then mean 2,132 | mean 1,373 | mean 1,682 |
| E pitch modulation | 66.9%, mean 957 | mean 1,725 | mean 1,656 |
| F noise | 34.3% | — | — |
| G the one-shot, the start in the wave | 100% and 99.6% | mean 2,963 and 117 | mean 59 and 33 |
| H the reverb | 20.2%, largest 148, mean 1.17 | mean 16.5 | mean 41 |

Beetle PSX's and PCSX-ReARMed's output is the mix times 0.75 [M, the fitted gain 0.7500 on the steady scenes], so they
compare within that rounding at best. **Where the audio differs from SwanStation it is at a register write's sample** [A]:
A's and B's key off, C's sweep start and D's fourth pitch change land a sample or two apart, because the vblank each
waits on falls at a different cycle on each emulator (D-0's line length), and a pitch changed a sample later moves the
waveform's phase for the rest of the scene. F's noise depends on the generator's state since power-on. The capture
records of §10.2, which synchronise to the SPU's own IRQ, are immune to this, and they are the gate's measure.

### 10.2 The voices and the reverb as sound RAM records them

`spu-capture.s` reads the capture buffers (voice 1's and voice 3's output after the envelope) and the reverb's work
area back by DMA4 on all five referees [M, `psxreferee.py ram`; Diamond's records pinned by
`the_capture_program_records_the_voices_as_the_referees_do`]:

| Test | Beetle PSX | SwanStation | DuckStation | PCSX-ReARMed | PCSX-Redux |
|---|---|---|---|---|---|
| T1 the wave at E00h and 1C00h, 1,024 samples steady | 27 and 16 equal | **all** | **all** | 58 and 8 | 38 and 30 |
| T1's ENVX, LSAX, ENDX; T7's LSAX, ENDX, ENVX | **equal** | **equal** | **equal** | ENDX always 0 | **equal** |
| T2 attack, decay, exponential sustain, 496 samples | **all** | **all** | **all** | none | none |
| T3 exponential release, 496 | **all** | **all** | **all** | none | none |
| T5 pitch modulation, 496 | none | **all** | **all** | 18 | none |
| T8 filters 5–7, 512 steady | none | none | **all** | none | none |
| T11 the key on's first samples | four samples later, the history full | **equal** | **equal** | no interpolation | — |

DuckStation is the strongest single witness: it equals Diamond in every voice test. SwanStation, an older fork of it,
differs only in T8's reserved filters. Beetle PSX equals Diamond wherever the interpolation's index is zero (the DC
block at pitch 1000h of T2 and T3); at fractional indices (T1, T5) it differs by a phase that no variant of the
interpolation tried reproduces, and it starts the envelope four samples after the voice's first samples (T11).
PCSX-ReARMed does not interpolate by the Gaussian table, its "gaussian" option set (T11 shows the raw sample), and keeps
no ENDX. PCSX-Redux's capture buffers do not raise the IRQ the program waits on in time, so that its T2, T3 and T5 are
not of the key on they mean.

Before the referees were asked, T1 differed from DuckStation in 441 of 512 samples by up to 38; fitting the decode and
the interpolation to its record (a model of both outside the tree, checked first to reproduce Diamond
exactly) found the prediction's products shifted apart and the interpolation's sum shifted once, which reproduce it in
all 1,024 (D-34, D-35); T5's and T11's first samples then gave D-36, T6 D-37, T8 D-40.

**The reverb.** T10 writes each stage of a pulse's path apart (the two sides' same and different reflections, both
all-pass filters) and stops the writes a frame on. Where the two sides' inputs are taken at the same instant on the
referee and on Diamond, **the work area equals DuckStation's in every stage of that side** (57, 57, 56 and 54 of 57, 57,
56 and 54 halfwords on the right, as of SwanStation's on the left); the other side differs at the pulse's edges, since
DuckStation and SwanStation take both sides' inputs at one instant and Diamond a sample apart (D-39). Beetle PSX takes
them a sample apart as Diamond does, but its pulse differs from the start by its envelope's latency (T11), so that its
reverb's arithmetic cannot be compared through this program. PCSX-ReARMed writes no work area; PCSX-Redux's is elsewhere
in the area.

### 10.3 ps1-tests' `spu/stereo`

Not run: it plays its four VAG samples through PSn00bSDK's library and prints only what it uploads, so that its grading
would be by ear; the voices' rules are graded by §10.2.

### 10.4 ps1-tests' `spu/memory-transfer` and `spu/test`

On the open kernel with no firmware folder [M, `the_spu_programs_pass_on_the_open_kernel`; WiseMan's TTY gate]:

| Program | Diamond | DuckStation | PCSX-Redux | The console's log |
|---|---|---|---|---|
| `spu/memory-transfer` | **9 of 9 pass**, the two DMA timing tests among them | 9 of 9 | 6 pass; both timing tests and the sync mode 0 read fail | 7 of 7 (an older build without the sync mode 0 tests) |
| `spu/test` | **no mismatch** at 16 or 32 bits | none | none | no log |

The register stub of stage 3 had failed the two timing tests, its DMA ending before the CPU's first poll.

## 11. D-29 and the tester's games

**D-29, by the screen only**: with the SPU in, the player's BIOS draws its logo by frame 450 and holds it to frame
2,400, as before [M, Crash Bandicoot's disc]. Five variations of the SPU's answers were tried, each to frame 1,500,
STATX showing ATTR at once, ENDX all set, ENVX always zero, STATX's IRQ flag always set, ENDX always clear; the logo
held in each. The SPU, as far as its registers show it, is not what the intro waits on.

Through the kernel's `LoadExec` (the run tool's `--boot`), pictures every 300 frames to frame 3,600 [M]: Crash
Bandicoot reaches its title screen by frame 2,400 and Spyro the Dragon his by 3,300, as at stage 9, once §6.3's defect
was fixed; Rayman's opening movie plays (505 decode commands), Tekken 3 reaches its title and attract mode, Silent Hill
stops after its memory card warning, Castlevania: SotN stays black, and Ape Escape waits at its DUALSHOCK screen, all as
at stage 9. **Tekken 3 and Castlevania send the same malformed MDEC words as at stage 9**, 122EE3E0h once and 120F0004h
nineteen times (counted by a temporary print of the words the games write to MDEC0, removed after), so the SPU is not
what corrupts them.

## 12. The cost

[M, the run tool, three runs each on one core, ms a frame; the stage's build against WiseMan at 6b90e700]:

| Program | Before | After |
|---|---|---|
| `timers`, 600 frames | 6.87–7.16 | 6.92–7.48 |
| `gpu/triangle` | 6.92–7.41 | 6.99–7.20 |
| `gpu/rectangles` | 6.35–6.38 | 6.00–6.06 |
| Crash Bandicoot, 3,600 frames from `--boot` | 4.63–4.66 | 4.74–4.83 |

The SPU alone, 24 voices keyed on a looping block and the reverb on, takes 0.245 ms a frame of 735 samples, 13.9 ns a
voice and sample with the reverb and the mix [M, a timing test of `Spu::run` outside the tree, three runs], within the
plan's 0.2–0.4 ms. The programs' spread is the machine's (`timers` varies by 0.6 ms between runs); `rectangles` faster
after is the layout under fat LTO that stages 7 and 9 also saw. Nothing allocates a sample: the queue's room is made at
create, the reverb's registers are copied to the stack.

## 13. What waits

- D-38 (the transfer's rate), D-39 (the reverb's input instants), D-40 (the reserved shifts and filters): open.
- Beetle PSX's interpolation phase at fractional indices, which no reading of psx-spx tried reproduces.
- RAM_CTRL's other bank sizes, the unstable DMA read, external audio, the capture IRQ's exact units.
- D-29.

## 14. Tests run (2026-10-11)

`cargo test --release` in the crate: 133 unit tests (the SPU's 23, its envelope's 5 among them, and the bus's test of
§6.3) and 49 program tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, all passing; the stage adds `spu-capture.s`,
`spu-voices.s`, `spu-dma.s` and ps1-tests' two SPU programs. WiseMan under the test lock with the corpus:
`DiamondTests`, `SystemPack`, `CoreAbi`, `CoreAdapter`, `CoreDiscovery` and `CoreRegistration`, 95 of 95, the TTY gate
on the open kernel grading `spu/memory-transfer` and `spu/test` among them.

## 15. Reading log, and exposures

**Read**: psx-spx at `6d7d1bc`: `soundprocessingunitspu.md` whole; `cdromformat.md`'s "CDROM XA Audio ADPCM
Compression" (lines 737–860); `dmachannels.md`'s MADR, BCR, DPCR and "DMA Transfer Rates" (lines 14–125, 195–215);
`iomap.md`'s SPU registers (lines 105–190); `interrupts.md`'s IRQ9 line; `memorycontrol.md`'s DEV4 heading.
ps1-tests: `spu/memory-transfer/main.cpp` and its `psx.log`, `spu/test/main.c` and its Makefile, `spu/stereo/main.c`'s
first fifty lines, `spu/playback/README.md`, `dma/dpcr/main.cpp` and its `psx.log`, `cpu/io-access-bitwidth`'s `psx.log`
and its SPU lines. `memory-transfer` and `dpcr` include `common/dma.hpp`, which was not opened; their binaries were run
as black boxes.

**Not read**: any emulator's or library's source; the BIOS; psx-spx's "BIOS Patches" and the HC05 chapters.

**Exposures, stated plainly**: (1) the first audio recordings of Beetle PSX, SwanStation and PCSX-ReARMed were made
before the program had its start marker and so may have held a BIOS's sound before the program's; they were deleted
unread, unplayed and unanalysed, and every later recording was cut at frame 400 by a script that discards the bytes
before it unread, after the referee's RAM showed the program's marker by then. (2) When Crash Bandicoot stopped (§6.3),
the processor's Cause, EPC, SR and RA were printed and showed it inside the kernel's address range; the kernel's code
there was not disassembled or traced, and only the game's own code, at RA, was disassembled from RAM, which showed a
decompression routine. (3) The game's SPU register traffic from its entry, and the MDEC words three games wrote, were
printed by temporary code removed after; the last game-code addresses before the stop were recorded by a filter that
kept only addresses of the game's range.
