# Mars — the address map, the debug port, and the one counter

*Phase A's first slice, landed 2026-09-15. There is still no CPU: this is what a CPU
will be able to reach, plus the clock it will advance.
`EmuSen/Cores/Nintendo/Mars - N64/Memory/`, tested by `MarsBusTests`.*

---

## 1. Virtual segments

The VR4300's address space divides into five segments by its top bits. Two of them —
the cached and uncached direct-mapped windows — need no translation at all: they
strip the top three bits and address physical memory directly. The other three go
through the TLB, which Phase A has not built yet, so `TryTranslateDirect` answers
false for them rather than guessing.

> **Superseded, 2026-09-16.** Five segments is the kernel-mode, 32-bit map, which is
> one of six. `TryTranslateDirect` is gone, and `Segments.Decode` answers the same
> question as a function of the privilege mode doing the asking — `Mars_Privilege.md`
> §2. The paragraph below still holds and is still why the debug port works early.

That split is why the corpus's debug port is reachable before the TLB exists. Its
addresses live in the uncached window, which is a subtraction rather than a lookup.

## 2. The physical map

Named once in `MemoryMap` so that no device repeats a literal: RDRAM and its
registers, the signal processor's two memories and its register blocks, the display
processor's command and span registers, the six interface register blocks, the four
cartridge domains, the debug port, and the two halves of the PIF.

### 2.1 Above the installed RDRAM is zero, not a mirror

A four-megabyte machine reading at eight megabytes gets zero, and the write that
would have gone there is dropped. This is not a tidiness decision — it is one of the
three behaviours the corpus's bootstrap depends on (`Mars_TestOracle.md` §3), and the
obvious alternative of mirroring is what a young emulator does by accident when it
masks an address instead of bounds-checking it. With an Expansion Pak the upper half
is real memory and the same addresses work.

> **The corpus itself needs the Expansion Pak**, which this section did not say and which
> went unnoticed for nine slices: its heap ends at a fixed seven megabytes, so on the 4MB
> default a test writing there has its writes dropped by exactly the behaviour described
> above, and reads back zeros. `Mars_Corpus.md` §8.

### 2.2 Registers nobody models still read back

Every hardware register that has no device behind it yet is a word in a dictionary:
written values come back, and an unwritten one reads zero. One is seeded — the
RDRAM interface's select register comes up nonzero, which is how libdragon's IPL3 is
told that memory needs no initialising.

This is a stub and is meant to be replaced device by device as the phases reach
them. The display processor's command registers left it on 2026-09-17, and now read zero
where nothing is modelled rather than echoing writes (`Mars_Rdp.md` §2.4). It is recorded here so that a register appearing to "work" is never mistaken
for a device being modelled. The dictionary is also the wrong shape for a hot path;
that is a Phase G problem and deliberately not solved now.

### 2.3 The cartridge is read-only, and its end is not modelled

Writes into cartridge space are dropped rather than refused, because the real bus
has no concept of failing a write. Reads past the end of the image return zero;
what hardware actually returns there is undocumented, and zero is a placeholder
chosen for being obvious rather than for being right.

### 2.4 The signal processor's memories repeat, and take only whole words from the CPU

*Added 2026-09-17, from the corpus's four `spmem` groups. `MarsBus.Store` and
`MarsBus.SignalProcessorMemory`; tests in `MarsSpMemoryTests`.*

**The 8KB of DMEM and IMEM repeat to the end of their window.** From `0x04000000` up to the
interface registers at `0x04040000`, every `0x2000` is another copy of the same two banks. A
write at `0x0403E000` overwrites DMEM offset 0. Mars had mapped the banks once and let
everything above them fall through to the register dictionary (§2.2), so that write landed
in a dictionary entry of its own and DMEM offset 0 kept its old value.

**A store from the main CPU always writes a whole word**, whatever size the instruction names:

- **`SB` and `SH` write the word the byte or half falls in, with the register's whole value
  shifted left so that the named bytes land in their lanes** — `word = register << (8 × (4 −
  size − offset))`, truncated to 32 bits. Every byte below the named ones becomes zero, and
  every byte above them takes the register's higher bits. So `SB` of `0x12345678` at offset
  0 writes `0x78000000`, and at offset 3 it writes `0x12345678`: a byte store that replaces
  the whole word with the whole register.
- **`SD` writes the register's upper half into the one word at the address**; the lower half
  goes nowhere, and the following word is untouched.
- **`SW` is ordinary**, which is the same rule with nothing to shift.
- **Loads are exact.** `LB` and `LH` read only the bytes they name. The quirk is in stores.

Mars implements this in one place: aligned stores from the CPU now hand the bus the whole
register and the size, and the bus applies the rule when the address is in the window and
the ordinary byte-precise write everywhere else. Main memory keeps byte-precise stores, and
a test pins that too, because the easy way to break this is to apply the rule everywhere.

**The corpus's comment understates its own vectors.** It states
the rule as observation — *"SH/SB are broken: They overwrite the whole 32 bit, filling
everything that isn't written with zeroes"* — and its vectors show more than the comment
says: the bytes *above* a byte store are not zero-filled but carry the register's upper
bits, which is what `SB` at offset 15 writing `0x12345678` demonstrates.

**A reading, not a measurement.** Those vectors are exactly what a bus that decodes only
32-bit accesses would latch if the VR4300 drives a sub-word store as its register shifted
into position on the full data bus. If that is the mechanism, it applies to every block of
the RCP that is built the same way, not only to these memories. Mars applies it only where
the corpus measures it, and the other RCP register blocks keep byte-precise stores until
something measures them.

**What this does not cover:**

- **`LD` from these memories**, which the corpus's comment says crashes the console and does
  not test. Mars reads two words.
- **Access through the cached segment**, which the same comment says also crashes hardware.
  Mars treats both direct-mapped segments alike (§5).
- **`SDC1`, `SWL`, `SWR`, `SDL` and `SDR`** into these memories. None is tested; they keep
  their existing writes rather than borrowing a rule measured on other instructions.
- **The signal processor's own access** (`Mars_Rsp.md` §4) and the DMA transfers (§6), which
  never go through the CPU's store path and are unaffected.

## 3. One counter for the whole machine

`MarsBus.Cycles` is the only clock, and `Tick` is the only thing that advances it.

### 3.1 `Count` is derived, not incremented

The VR4300's `Count` register advances at half the CPU clock, which the vendor
manual states as a constant rate independent of what the processor is doing
(`Mars_Documentation.md` §2). Mars therefore **computes** it — `Cycles >> 1` plus a
bias — rather than having instructions add to it. Writing `Count` sets the bias and
the clock keeps running underneath.

**This is a direct answer to a documented failure mode.** Project64's history carries
counter-update fixes scattered years apart across individual opcodes and exception
paths: updating the counter when writing it from a coprocessor move, before reading
it back, around arithmetic overflow exceptions, and a double update in a call
instruction (`Mars_References.md` §5.3). Each of those bugs is only possible in a
design where advancing the counter is the caller's duty. Here there is nothing for an
opcode to forget, because no opcode participates.

The same reasoning is why Mercury's bus is cycle-granular: advancing the machine is
what an access *is*, rather than something the CPU remembers to do afterwards
(`Mercury_Cpu.md` §3).

### 3.2 What the clock does not yet claim

**Nothing charges anything.** `Tick` exists and no access calls it, so `Cycles` moves
only when a test moves it. That is deliberate: the survey found that RDRAM latency,
DMA durations, bus arbitration and cache-miss costs are documented nowhere at all
(`Mars_Documentation.md` §5), so any per-access number written today would be
invented. The structure is settled now because structure is what leaks; the numbers
wait for evidence, and the corpus's timing category is what will supply the first of
it.

## 4. The debug port

A development cartridge's text output, mapped inside cartridge space: a length
register, a buffer after it, and a word write to the length register meaning "emit
this many bytes". Emitted bytes accumulate where a harness can read them as text —
the same role Mercury's serial sink plays, reached by a different mechanism.

### 4.1 It is memory, not a sink

The corpus detects the port by **writing a known word and reading it back**. A
write-only sink — which is exactly the shape of Mercury's serial port, and therefore
the shape a reader of this project would reach for first — fails that detection
silently, falls through to a channel that is not there, and runs to completion having
printed nothing at all (`Mars_TestOracle.md` §2.2).

So the region is backed by real memory, and `MarsBusTests` asserts the read-back
directly rather than only asserting that text comes out. The test that would have
caught this bug is cheaper than the day spent finding it.

## 5. What is not here

- **No TLB**, so three of the five segments do not translate.
- **No devices beyond the two transfer engines** (§6, §7). Nothing interrupts,
  nothing counts down, nothing draws.
- **No caches.** Both direct-mapped segments reach memory identically, and
  `IsCached` reports the difference without anything acting on it.
- **No access costs** (§3.2).

## 6. The signal processor's transfers

Its registers and its DMA exist; the processor behind them does not. A length write
moves bytes between one of the two four-kilobyte banks and RDRAM, with bit 12 of the
memory address choosing the bank, the length encoded one short, and a row count and
skip making the transfer rectangular.

This is here before the CPU because the corpus's bootstrap uses it: libdragon's IPL3
moves itself with these registers long before any instruction of ours is involved
(`Mars_TestOracle.md` §3).

### 6.1 Never busy, because it has already finished

Transfers complete inside the register write, so the busy and queue-full flags read
zero always. That is a simplification with a real consequence — a program that
watches the flags to overlap work with a transfer sees a machine that is never
transferring — and it is invisible to anything that only waits for completion, which
is what the bootstrap does. The honest position is that the timing of these
transfers is unmodelled rather than instantaneous, and §3.2's reasoning applies: the
durations are documented nowhere, so they wait for measurement.

### 6.2 Past the end of a bank is that bank's beginning

A transfer that runs off the end of the instruction bank continues at its start
rather than spilling into the data bank. This is the third of the bootstrap's
requirements, and the failure it guards against is a real one in emulators that
address the two banks as one eight-kilobyte block.

**The test for it is weaker than it looks, and that is worth stating.** Mars keeps
the banks as two separate arrays, so spilling from one into the other is not
something this design can do: removing the wrap makes the transfer crash rather than
quietly corrupt the neighbour. The test pins the wrapping behaviour, but the bug
class it was written for is structurally unreachable here. It is kept because the
requirement is real and a future refactor to one backing array would reintroduce
exactly that bug — at which point this test stops being insurance and starts being a
catch.

## 7. The peripheral interface

The cartridge's transfer engine, same length encoding, moving bytes in either
direction between the cartridge bus and RDRAM, raising a completion flag that a
write clears.

### 7.1 Idle is not cosmetic; it is what lets the corpus speak

The status register reports neither kind of busy. That is not tidiness — the corpus
polls this register in a spin loop between **every word it prints**, and its
ISViewer path reads the I/O-busy bit before each store (verified in the corpus's own
source). A status register that comes up busy, or that latches busy after a
transfer, hangs the ROM before it emits a single character, and the symptom is
indistinguishable from the silent-detection failure in §4.1.

Two landmines with one symptom is the reason both have a test naming them.

## 8. The interrupt aggregator

Six devices, one line to the CPU. A device raises its bit; the line is asserted while
any raised bit is also unmasked. It is level-triggered rather than latched: clearing
the device's bit lowers the line, and nothing has to tell the CPU
(`Mars_Cpu.md` §10).

The peripheral interface is the first device wired to it. Its completion flag is no
longer its own — the status register reports what the aggregator holds, and writing
the acknowledge bit clears it there. One flag in one place, rather than two that can
disagree, which is the same argument the cheat registry seam settled in
`EmuSen_Cheats.md` §6.

### 8.1 The mask takes two bits per device

One to clear the mask and one to set it, so a program can change one device's mask
without reading the register first or disturbing the other five. The write path
implements the pairs; a test writes the set bit and then the clear bit for the same
device and watches the mask follow.

The other five devices exist as bits with nothing behind them yet.

> **Update 2026-09-17: the display processor raises its bit** at a full sync (`Mars_Rdp.md` §6).

### 8.2 The display processor's interrupt is cleared through the mode register

It has no clear bit of its own in any display processor register. Bit 11 (`0x800`) of the
aggregator's mode register clears it, and that is the only bit of the mode register Mars
models: a write without it does nothing. The evidence is a commercial handler rather than a
document re-read for this slice — Wave Race's interrupt handler clears the bit 235 instructions
after it is raised, and nothing else in Mars can. The mode register's other fields, and what it
reads back, are not modelled; five MI groups in the corpus's census are about them
(`Mars_Corpus.md` §3).
