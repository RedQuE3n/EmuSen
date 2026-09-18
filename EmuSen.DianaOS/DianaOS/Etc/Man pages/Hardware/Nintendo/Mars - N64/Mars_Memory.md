# Mars — the address map, the debug port, and the one counter

*Phase A's first slice, landed 2026-09-15. There is still no CPU: this is what a CPU
will be able to reach, plus the clock it will advance.
`EmuSen/Cores/Nintendo/Mars - N64/Memory/`, tested by `MarsBusTests`.*

*The bus was ~~`MarsBus`~~ until 2026-09-17, when it became `MemoryBus` to match Venus, Moon and Mercury,
each of which calls its own `Memory/MemoryBus.cs`. The namespace already carried the codename, so the prefix
was saying it twice. The test class keeps its `Mars` prefix, because `EmuSen.WiseMan` names every test file
after the core it grades.*

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
has no concept of failing a write.

> **Update 2026-09-18: a processor store is no longer only dropped.** The PI keeps it, and the next
> processor read of the cartridge returns it instead of the ROM (§7.7). The ROM itself still never
> changes, so the sentence above holds for the cartridge and not for the bus in front of it. Reads past the end of the image return zero;
what hardware actually returns there is undocumented, and zero is a placeholder
chosen for being obvious rather than for being right.

### 2.4 The signal processor's memories repeat, and take only whole words from the CPU

*Added 2026-09-17, from the corpus's four `spmem` groups. `MemoryBus.Store` and
`MemoryBus.SignalProcessorMemory`; tests in `MarsSpMemoryTests`.*

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
the ordinary byte-precise write everywhere else. **Since 2026-09-17 that window is two**, PIF
RAM having turned out to latch words the same way (`Mars_Serial.md` §1); one predicate names
both. Main memory keeps byte-precise stores, and
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

> **Update 2026-09-18: a third block measures the same rule.** The PI's store latch keeps `SB` of
> `0x123456BA` at offset 1 as `0x56BA0000` (§7.7), which is this reading's prediction in a block that
> shares nothing with these memories but the bus. And the cartridge's read quirk (§7.8) is what the
> same mechanism looks like from the load side. That is corroboration, not proof. The reading is still
> a reading, and Mars still applies the rule only where something measures it.

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

`MemoryBus.Cycles` is the only clock, and `Tick` is the only thing that advances it.

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

> **Update 2026-09-18: the I/O-busy bit is no longer always clear.** It reports a processor store to
> the cartridge for as long as the PI keeps it — 225 cycles at most, or until a read (§7.7). That is
> safe for the reason this section gives: the word decays with the clock, so a spin loop on the bit
> always ends. The debug port is kept outside the window as well (§7.7), so none of the corpus's own
> printing is what its printing waits for. The DMA-busy bit still never shows. The warning itself is
> still not tested by a run; §7.9 records the breakage that was expected to test it and did not.

### 7.2 Each side of a transfer advances by its own bus width

*Added 2026-09-17, from nine of the corpus's `cart_memory` assertions. ~~The rest of that
group is still open, and the end of this section says what is known about it.~~ The rest of
that group was closed the same day by §7.3, which retires two things below.*

**The cartridge's bus is sixteen bits wide and RDRAM's is sixty-four, and a transfer leaves
each address on a multiple of its own width**, whatever length was asked for:

- **The cartridge address advances by the length rounded up to two.** A one-byte transfer
  moves it by two, a seven-byte transfer by eight.
- **The RDRAM address advances ~~past the length~~ past the last byte the transfer stored,
  and then up to the next multiple of eight** — ~~`(address + length + 7) & ~7`~~. A two-byte
  transfer starting at an aligned address moves it by eight; a nine-byte transfer starting six
  bytes into a block leaves it two blocks on.

> **Retired 2026-09-17, by §7.3: "past the length".** The closed form holds for every aligned
> transfer and not for a misaligned one, which stores fewer bytes than it was asked for; the
> address follows what was stored. Seven bytes sent to an address six bytes into a word store one,
> and leave the address one word on rather than two. The rule was stated for every transfer and
> only checked against aligned ones. The corpus's misaligned cases do assert the increment, but
> each stops at its first failure, and until §7.3 that was always a byte.

The FPGA core states the second rule in two lines — `PI_DRAM_ADDR <= PI_DRAM_ADDR + 7;
PI_DRAM_ADDR(2 downto 0) <= "000"` (`rtl/PI.vhd` §711–712) — and both address registers
already dropped their low bit on a write, which Mars did too.

**Nine assertions, predicted and then measured.** The corpus's census attributed five
assertions to the cartridge address and four to the RDRAM address; the count went from 146
to 137, which is nine.

**What was still open, and one thing it is not** — closed by §7.3, and kept as it stood. Sixty-five assertions in the same group are
about *which bytes land*, under four headings the corpus names itself: small sizes,
misaligned, misaligned crossing a page, and misaligned at the end of a page. Two things are
known:

- The FPGA reads a block of at most `128 − (address & 7)` bytes, capped by the distance to
  the end of a 2KB row, and then writes only the first `blocklength − (address & 7)` of them
  to RDRAM (`rtl/PI.vhd` §585–597, §750). That count is declared `integer range -7 to 128`,
  so it can be **negative**: a short transfer to a misaligned address writes nothing at all,
  which is what the corpus expects for a one-byte transfer six bytes into a block.
- ~~"An odd length carries one byte more than it asked for, on both sides."~~ **Refuted by
  measurement.** The cartridge *address* does advance by the rounded length, so carrying the
  extra byte through to RDRAM looked like the same rule seen twice. It is not: making the
  transfer copy `(length + 1) & ~1` bytes cleared four assertions about a byte at the end of
  a long transfer and broke five about bytes past the end of a short one — 137 to 138.
  ~~The extra byte is read from the cartridge and does not reach memory~~, and where exactly it
  is dropped is part of the open rule above rather than a separate one.

  > **Retired 2026-09-17, by §7.3.** The refutation stands; the conclusion drawn from it does
  > not. The extra byte does reach memory in any block after the first, and in a first block one
  > byte short of full — and the four cases the experiment cleared were exactly those. It is
  > dropped only at the end of a shorter first block. The conclusion generalised from the only
  > case the experiment happened to break.

### 7.3 Which bytes a transfer from the cartridge stores

*Added 2026-09-17, from the sixty-five `cart_memory` cases §7.2 left open. Each failure line
was worked through the rule below by hand before any code changed. The count then went from
137 to 72, and a diff of the two verdict files shows those sixty-five lines gone and no line
anywhere else added or removed. `PiInterface.FromCartridge`, tested by `MarsDmaTests`.*

**A transfer from the cartridge into RDRAM runs in blocks, and the first block pays for the
RDRAM address's misalignment twice** — once in how many bytes it reads, and again in how many
of those it stores. Write `m` for the address's position inside its eight-byte word. It is always
even, because the register drops bit 0 on a write. Each block:

1. **Reads** `min(largest − m, the distance to the end of the 2KB row, what remains)` bytes,
   where `largest` starts at 128. The cartridge address and the remaining count both move by
   that number rounded up to a pair.
2. **Stores** the first `block − m` of them, from the RDRAM address on. When that is zero or
   less, the block stores nothing at all.
3. **Ends an odd store count on a lone byte** if it is the first block, unless that block is
   exactly `127 − m` — one short of a full first block. In that case, and in every block after
   the first, the last pair is stored whole.
4. **Rounds the RDRAM address up** to the next word, so every block after the first starts
   aligned and has `m = 0`.
5. **Shortens the next block** to `largest = 128 − m` if this one began within eight bytes of
   the end of its row. `largest` goes back to 128 when the next transfer starts.

The corpus's `(6, 128)` — 128 bytes to an address six bytes into a word — walks like this. The
first block reads 122 and stores 116, at offsets 0–115. The address rounds up to offset 122, and
the second block reads the last six and stores them there, from cartridge offsets 122–127.
Offsets 116–121 keep what they held. That is the corpus's expectation: its failure line for the
case was *"Byte at offset 116"*, expected untouched.

### 7.4 The grader, and the corpus's own model

**The grader is the corpus**, which asserts against silicon (`Mars_TestOracle.md` §1), and its
source is in reach at `~/Projects/nemu64-test-reference` (`src/tests/cart_memory/dma.rs`,
`9a8b9f7`). Reading it settled two things that had only been inferred from the failure lines:

- **Each of the sixty-five is a whole case, not one byte.** A case checks the sixteen bytes
  before the destination, every byte it expects, the sixteen after, both address increments and
  the status, and stops at its first failure. So the address increments of the misaligned cases
  were being asserted all along, hidden behind the byte failures. They now pass as well, which
  means **the RDRAM address after a misaligned transfer is measured, not taken from the FPGA**
  — see the retirement in §7.2.
- **The three placements are fixed.** A buffer aligned to 2048 bytes, and a destination at
  offset 16 plus `m`, at 64 bytes before the end of the row, or in the row's last eight bytes. The
  corpus names its page size in a constant, `RDRAM_PAGE_SIZE = 2048`.

**The corpus states its expectations as formulas over the size, not as tables**, so they can be
compared with the walk at sizes the corpus never runs. A script did that for every size from 1
to 599 in every family, at every misalignment the formulas accept. First, it reproduces all
seventy-five of the corpus's cases, bytes and both increments, independently of the C#. Second,
**the corpus's model and the walk agree at every size except one per misalignment**: 61, 59 and
57 for `m` = 2, 4 and 6, in the cross-page family. That is an odd size one byte short of the
distance to the row's end, which caps the first block. The corpus's formula stores the last
pair whole there, as it would for a block one short of 128. The walk ends on a lone byte,
because step 3 compares against `127 − m` and not against the row.

**That is a dispute, and it is unmeasured**: the corpus's list of sizes for the cross-page family
skips it. Every implementation in reach sides with the walk — the FPGA (`rtl/PI.vhd` §809), Project64
(`PI_DMA_WRITE`, `BlockLen == BlockSize - 1`), and mupen64plus (`dma_pi_write`, which does not
model rows at all and gets there by its `0x7f` threshold alone) — but they are not independent
voices; see below. Mars follows the walk because it implements the walk, not because the walk
is likelier to be right at that size. **One transfer settles it on a console: 57 bytes, six
bytes into a word, 58 bytes before the end of a row.** `MarsDmaTests` names the case, so the
answer has somewhere to go.

### 7.5 Where the walk comes from, and why its sources do not add up

No software reference in Phase D's ladder models the peripheral interface. The walk was written
from the FPGA core, `rtl/PI.vhd` §585–597 (the block), §784–830 (the store mask, a pair at a time)
and §708–722 (the block's end). Three implementations were read afterwards, and **they are one
lineage more than they are three witnesses**:

- **The FPGA's clone is shallow** — one commit, `adbf9b5`, 2026-08-15 — so it cannot say where its
  conditions came from.
- **Project64 has the same walk under the same names** — `MaxBlockSize`, `EndOfRow < 8 ? 128 −
  align : 128`, and the same read-back rule for the length register. Its history is complete, and
  it says where the walk came from: `9e53b161a` (2024-10-03) is titled *"Update … PI_DMA_WRITE to
  handle misaligned, end of page test"* — fitted to this corpus, by name.
- **mupen64plus** has a two-line approximation with no rows and no second block, citing n64brew's
  *Unaligned DMA transfer* section.

So the implementations agreeing with the corpus is **circular wherever the corpus pins the rule**,
and it is **no evidence at all where it does not**. The table below says which is which.

### 7.6 Which rules the corpus pins

Each rule was broken in turn, and the corpus and the named cases were run again:

| rule broken | step | corpus | named cases that catch it |
| --- | --- | --- | --- |
| the bytes a block skips are read from the cartridge | 1 | 142 (+70) | four |
| the misalignment comes off what is stored as well | 2 | 133 (+61) | six |
| the remaining count moves past the bytes a block skips | 1 | 133 (+61) | six |
| each block starts on a word | 4 | 103 (+31) | three |
| a block stops at the end of a row | 1 | 101 (+29) | two |
| a row is 2KB, not 4KB | 1 | 101 (+29) | two |
| a block is 128 bytes, not 256 | 1 | 88 (+16) | four |
| only the first block ends on a lone byte | 3 | 85 (+13) | two |
| a short first block ends on a lone byte | 3 | 82 (+10) | one |
| a lone byte is the first of its pair | 3 | 82 (+10) | one |
| a block near the end of a row shortens the next | 5 | 81 (+9) | one |
| the next block loses the misalignment, not the distance | 5 | 79 (+7) | one |
| the first block is short by its misalignment | 1 | 74 (+2) | one |
| a first block one byte short of full stores its pair whole | 3 | 74 (+2) | one |
| "near" is under eight bytes, not under six | 5 | 74 (+2) | one |
| only a block near the end of a row shortens the next | 5 | 74 (+2) | two |
| **a row is 2KB, not 1KB** | 1 | 72 | none — one added |
| **"near" is under eight bytes, not under sixteen** | 5 | 72 | none — one added |
| **the shortening ends with its transfer** | 5 | 72 | none — one added |
| **the disputed size (§7.4), as the corpus's formula has it** | 3 | 72 | one, added with the dispute |
| the address after a lone byte, `+1` for `+2` | — | 72 | none, by proof |
| the threshold `126 − m` for `127 − m` | — | 72 | none, by proof |

**Sixteen of the twenty-two breakages move the corpus, and not equally.** Five carry most of the
weight — the double misalignment, the cartridge and the count both moving past what a block skips,
the rounding to a word, and the row — and breaking any one of them costs between twenty-nine and
seventy cases. Breaking the skipped bytes' cartridge reads costs seventy, more than the sixty-five
§7.3 cleared, because every case in the four families asserts the cartridge increment, including
the ones that passed before. At the other end, **four are caught by two cases each**: the first
block's own shortening, the exception for a first block one short of full, and two of the three
things step 5 says about a block near a row's end. Two cases are enough to rule the alternative out,
and not enough to call the rule well measured.

**Three rules are not pinned by the corpus at all, and Mars carries them on the FPGA's word
alone.** A row is 2KB rather than 1KB: every row end the corpus uses is both. "Near the end of a
row" means under eight bytes: the corpus puts a first block at 2, 4, 6 and 58 bytes from a row's
end, so any threshold from 8 to 58 passes it. The shortening ends with its transfer: the only
transfers that leave it set are short ones in a row's last eight bytes, and no case after one of
them is long enough to feel it. Each now has a named case that says it is unmeasured, and each
case was checked by putting the alternative back. The corpus's `RDRAM_PAGE_SIZE = 2048` does not
change the first of these: it is the corpus's author agreeing with the FPGA, not silicon.

**Two proofs, which took a rule out of Mars and pinned a digit only as far as it can be pinned.**

- **The address after a lone byte.** The FPGA advances by one after a lone byte, and Mars
  advances by two. Nothing is stored between a lone byte and step 4's rounding, and for an even
  `x`, `(x + 1 + 7) & ~7` and `(x + 2 + 7) & ~7` differ only if `x + 9` is a multiple of eight —
  that is, only if `x` is odd. So the FPGA's `+1` cannot be observed, and Mars does not carry it.
  Putting it back moves nothing (the table's second-to-last row).
- **The digit in `127 − m`.** `m` is even and every later block is aligned, so a first block is
  odd only when it is the whole of what remains. The two odd lengths at the top of a first block
  are `125 − m` and `127 − m`, and thresholds of `126 − m` and `127 − m` sort them identically;
  no experiment can tell them apart. The sources say as much without meaning to: **the corpus's
  own formulas write 126** (`requested >= 126 - m`), and the FPGA and Project64 write 127. Mars
  keeps 127 so it can be read beside the FPGA, and the last digit is a convention, not a finding.

**§7.2 drew two conclusions this rule contradicts, and both are retired there** rather than
quietly edited: that an odd length's extra byte never reaches memory, and that the RDRAM address
advances past the *length*. The first generalised from the only case the experiment broke. The
second was stated for every transfer and only ever checked against aligned ones.

**What this is not evidence for.**

- **The other direction.** RDRAM to cartridge is still the plain copy. The FPGA's walk that way
  (`DMA_READRDRAM`, §868 on) has no misalignment rule and only writes save memory, the corpus has
  no case in that direction that fails, and a write into ROM is dropped anyway (§2.3).
- **Time.** The transfer finishes before the register write returns, as §7.1 requires. The
  FPGA totals the domain's latency and pulse widths into `rom_slow_sum` and holds the busy flag
  until they elapse; Mars models none of that.
- **The write-length register read back after a transfer.** The FPGA and Project64 return `0x7F`,
  or `127 − m` when the last block (the FPGA, §353, §718–722) or the whole transfer (Project64)
  was eight bytes or fewer. Mars reads zero, and no corpus case reads it.
- **Any placement outside the corpus's three**, which this page lists above. Every other address
  follows from the same five steps and has not been measured.

### 7.7 The processor's stores to the cartridge are kept once, and decay

*Added 2026-09-18, from the corpus's thirteen `cart-writing` groups — sixteen cases, since its
decay group runs four store sizes. `PiInterface.CartridgeStore`, `TakeStored` and `IoBusy`, and
`MemoryBus.Store` and `Load`; tests in `MarsCartridgeTests`.*

**A processor store anywhere in the cartridge's ROM window is kept by the PI, and the next
processor read of the window returns it instead of the ROM** — once, and whatever address either
of them names. Every rule below comes from the corpus's source (`src/tests/cart_memory/write.rs`),
which states them in its comments as observations and then asserts them:

- **The window runs from `0x10000000` to the last word before the PIF's ROM at `0x1FC00000`.** A
  store at `0x1FBFFFFC`, far past the end of any image, is kept; one at `0x1FC00000` is not.
- **Only the first store is kept.** A second store while the first is held is lost.
- **The word kept is the register shifted to its lane** — §2.4's rule, now measured in a third
  block: `SB` of `0x123456BA` at offset 1 keeps `0x56BA0000`. `SD` keeps its upper half, which is
  what two word stores would give if the second were lost.
- **A read takes it back once and frees the bus.** A narrower read takes its lane from the kept
  word. The corpus reads only the first lane — `0xBA` and `0xBADC` from `0xBADC0FFE` — and the
  other lanes follow the FPGA core, which hands the kept word back whole for the processor to pick
  from (`rtl/PI.vhd` §486–489).
- **It decays, and the status register's I/O-busy bit reports it while it lasts.** Mars holds it
  for 225 of the processor's cycles, which is the FPGA core's 150 at 62.5MHz (§508–514). The corpus
  pins it only to a window, 34 to 333 of those cycles (§7.9).

**Mars reads busy off the one clock**, `Cycles < storedUntil`, the way `Count` is derived (§3.1).
Nothing counts down, so a device nobody steps cannot be left busy.

**Where the rules come from.** The corpus is both their source and their grader, and it is
graded against silicon. Two implementations were read beside it: the FPGA core's `writtenData`
and `writtenTime`, and Project64's `RomMemoryHandler`, whose decay is a timer of `0x5E` and whose
history dates it to `9b16d2979` (2022-08-22), *"Core: Add rom write decay"*. Neither constant is
derived from anything its source says. Mars takes the FPGA core's and records the window the
corpus allows around it.

**The debug port is outside the window, and that is a decision about the instrument, not a claim
about the console.** The corpus prints through the ISViewer, which sits inside the ROM window at
`0x13FF0000`, and its printing ends each test's name by writing the chunk length and going straight
into the test. With the port inside the window, the corpus went from 72 failing cases to **69 rather
than 54**. Thirteen of the eighteen cases this section clears still failed, and two that had passed
before broke — `cart: Read32` among them. Each read back the length the corpus had just written,
`0x4` for *"...\n"*, or a lane of it, because the test's own first store arrived while that one was
still held and was lost. A console with a real ISViewer would do the same, since the latch is in the PI and nothing
on a cartridge can get around it; the corpus's expectations can only have been measured with its text
going out some other way. Both implementations that pass these cases do so because the text never
reaches the latch: Project64 gives the ISViewer its own handler, and the FPGA core has no ISViewer, so
under it the corpus's detection fails and its text goes nowhere. Mars does what Project64 does. The
harness's verdict channel is the one place this project models something that is not on the console,
and this is the price of it, stated where it is paid.

The same run also showed `DMA CART -> DMEM` failing on the status it reads *before* its transfer.
That test first writes the PI's reset bit, which raised the question of whether a reset frees the
bus. The FPGA core's reset clears the DMA-busy and error bits and not I/O-busy (§384–391), and with the
debug port outside the window the case passes again, so Mars leaves the reset alone.

**What this is not evidence for.**

- **The cartridge's other windows** — SRAM, FlashRAM, the 64DD. The FPGA core keeps any PI bus
  store; Mars keeps only those in the ROM window, because nothing behind the others is built.
- **Reads of the window past the end of the image.** The FPGA core returns its open-bus pattern
  there and leaves the kept word alone; Mars gives it back. No corpus case reads past the image.
- **`LD`, `LWL`, `LWR`, `LDC1`, and every store but the aligned five.** None goes through
  `Store` or `Load`. The corpus's comment says `LD` crashes the console, and its `LWL` case only
  prints.
- **The cached segment**, which the same comment says crashes the console.
- **Whether a store that arrives while the bus is busy reaches the cartridge.** The corpus can see
  only that it is not kept.
- **The domain's timing registers**, which on the console presumably set how long a bus write takes
  and so how long the word is kept. Mars does not model them, and neither constant above depends on
  them.

### 7.8 A halfword or byte read cannot reach every other halfword

*Added 2026-09-18, from the corpus's `cart: Read16` and `cart: Read8`. `MemoryBus.CartridgeWord`;
the corpus's two tables are in `MarsCartridgeTests`.*

**A processor read of a halfword or a byte from the cartridge takes its lane from a word that
starts at the *halfword* the address names, not the word.** An address with bit 1 set therefore
lands two bytes further on, and the halfword two bytes into each word cannot be reached at all.
The corpus says it as *"LH/LB are broken: Every other 16-bit word is not reachable"*, and gives a
table for each. Word reads are unaffected, and so are transfers, which never use this path.

It is §2.4's reading seen from the other side. If the processor always asks for a whole word and
picks its lane, and the PI starts that word at the halfword it was given — its bus is sixteen bits
wide (§7.2) — this is what the processor sees. The FPGA core does the same thing in its own terms
(`rtl/PI.vhd` §494–499, on `bus_cart_addr(1)`), and Project64 reads from `(Address + 2) & ~3`. That
is the mechanism the three descriptions share, not a measurement of it.

### 7.9 Which of these rules the corpus pins

Each rule of §7.7 and §7.8 was broken in turn, with the corpus and the named cases run again:

| rule broken | corpus | named cases that catch it |
| --- | --- | --- |
| a store is kept at all | 70 (+16) | eight |
| a load takes the kept word | 70 (+16) | eight |
| the debug port is outside the window | 69 (+15) | one |
| a read frees the bus | 66 (+12) | three |
| only the first store is kept | 64 (+10) | three |
| the word decays at all | 58 (+4) | two |
| a narrower store is shifted to its lane | 58 (+4) | one |
| a halfword or byte read starts at the halfword named | 56 (+2) | two |
| a doubleword keeps its upper half | 55 (+1) | one |
| the window stops before the PIF's ROM | 55 (+1) | one |
| the status register reports the bus busy | 55 (+1) | two |
| **a narrower read of the kept word takes its lane, not its top** | 54 | one, marked unmeasured |
| **a transfer neither sees nor frees the kept word** | 54 | one, marked unmeasured |
| the decay is 8, 16, 24 or 32 cycles | 58 (+4) | one, added after the round |
| the decay is 340, 350, 360 or 400 cycles | 58 (+4) | one, added after the round |

**Eleven rules are pinned, and two are not.** The two the corpus cannot see are the lanes past the
first — where the corpus's own comment, *"the upper bits are returned"*, and the FPGA core's lane
reading agree at the only lane it reads — and whether a transfer disturbs the kept word, since no case
starts one while a store is held. Mars takes both from the FPGA core, and each has a named case that
says so.

**The decay is pinned to a window, and the constant is not.** Breaking the decay in either direction
moves exactly the four decay cases, and bisection puts the window's edges at **34 and 333** of
Mars's cycles. In Mars's accounting the corpus's two reads come 33 and 333 cycles after its store —
three hundred apart, a hundred turns at three cycles each — so a word kept for fewer than 34 is gone
at the first, and one kept for more than 333 is still there at the second. The FPGA core's 225
sits well inside, at about seventy-four turns of that loop in Mars's accounting of three cycles a
turn — close to the *"around 70"* the corpus's comment gives for the console. That comment is not an
assertion, and three cycles a turn is Mars's accounting rather than a measurement, so the closeness
is a coincidence worth noting and nothing more. The named decay case said nothing about any of this
during the round, because it used the constant rather than a number; `The_decay_falls_inside_the_window_the_corpus_allows`
now pins the window itself.

**One prediction was wrong.** Before the round, a word that never decays was expected to hang the
corpus, as §7.1 warns a busy flag that never clears would. ~~"The corpus will not finish."~~ It
finished, at 58. Every store the corpus makes is followed by a read in the same case, and a read frees
the bus, so a word that never decays is only ever seen by the four cases that wait for it. §7.1's
warning is not tested by this round, and still stands on the corpus's source rather than on a run.


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
