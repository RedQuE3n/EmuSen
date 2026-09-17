# Mars — the bit that turns the machine little-endian, and the one rule behind it

*Landed 2026-09-16. `Status.RE`, which mirrors every user-mode access inside its own
doubleword. The mirror and the access/fault split are in `Cpu.cs`; the call sites are the
loads, the stores, the merging family, the coprocessor-1 transfers and the instruction
fetch. `MarsCpuReverseEndianTests`, on `PrivilegeFixture`.*

---

## 1. What the bit is for, and where it applies

The VR4300 is big-endian. `Status.RE` (bit 25) lets a kernel run **user-mode** code that
expects the other byte order, without the kernel changing its own. Mars honours it in user
mode only: kernel and supervisor read straight through it, and a test asserts that rather
than leaving it implied.

It is not a byte-swap of the data. Nothing about the *value* changes — an access reads the
bytes it finds, big-endian, at a different address. Everything below follows from that.

## 2. One rule, and it is not the obvious one

**The effective address is exclusive-ORed with 7, and then the access realigns itself.**

That mirrors an access within the eight-byte group containing it: byte 0 of a doubleword
becomes byte 7, the first word becomes the second, and a doubleword is unchanged. For an
aligned access of *N* bytes it is the same as XOR with `8 − N`, which is the form the
corpus's own helper uses and which is easy to mistake for the whole rule.

It is not the whole rule, because **the merging loads and stores mirror as bytes**. `LWL`,
`LWR`, `LDL`, `LDR` and their four store counterparts take a byte address by construction,
so they get the byte mirror — XOR with 7 — and not the XOR with 4 their name suggests, nor
the XOR with 0 that `LDL`'s width would suggest.

This was settled by measurement rather than by reading. The corpus tabulates twenty-four
merging-access results, and all three candidate rules were checked against all twenty-four
mechanically before any code was written: XOR with 0 and XOR with 4 each fail, and XOR
with 7 matches every row. The same check then confirmed that XOR with 7 followed by the
instruction's own alignment reproduces the corpus's `8 − N` for every aligned access at
every offset, which is what makes it one rule rather than two.

Mars therefore writes it once:

```csharp
private ulong Mirrored(ulong address, int size) =>
    ReverseEndian ? (address ^ 7) & ~(ulong)(size - 1) : address;
```

with the merging family passing a size of one.

### 2.1 The fixture is worth a second look

The corpus's sixteen fixture bytes are `08 07 06 05 04 03 02 01 10 0F 0E 0D 0C 0B 0A 09`,
and the reason is neat: under the mirror, a byte load at offset *k* returns *k+1*. A test
whose expected value is a function of its own input is a test whose expectations cannot be
quietly wrong, and it is worth stealing the next time a table like this is needed.

## 3. The mirror does not reach the faulting address

An unmapped access under reverse-endian reports the address the program asked for, **not**
the mirrored one. The corpus has a test for exactly this, whose name says "no xor tweak",
and it is the reason `Translate` was split: `TranslateAccess` takes the address to
translate and the address a fault should name, and they are the same word everywhere
except here.

Alignment is judged the same way — on the address the program named. `LW` at an address
ending in 2 is an address error whether or not the mirror would have made it aligned, and
the mirror is applied after that check. Both halves have a test.

Segment decoding uses the mirrored address, which changes nothing: every segment boundary
in `Mars_Privilege.md` §2 is at least 2²⁹-aligned, so three bits cannot move an address
out of its segment.

## 4. Instruction fetch is mirrored too

This is the part that is easy to leave out, because "reverse-endian" sounds like a
property of data. A fetch is a four-byte access and gets the four-byte mirror, so under
`RE` the processor executes each pair of instruction words in the opposite order.

The corpus makes this visible rather than asserting it directly: every reverse-endian test
program is written through `re_fetch_encode`, which swaps the program's words in pairs
before writing them, so that the mirror puts them back. A test here asserts the effect
directly — two different instructions, and the second one runs first.

**It also broke the first version of the tests in a useful way.** Every other Mars
instruction test writes a program and runs it; these could not, because the fetch mirror
took the word after the one under test. The fix was to write each instruction into *both*
halves of its fetch pair, so the mirror finds it either way — which is a smaller change
than the corpus's swap and works for the control cases with the bit clear as well.

## 5. What it found in the TLB

Five of the seven corpus tests still failed after all of the above, with data loads
returning instruction bytes. The cause was not in this slice at all: Mars's TLB treated
`PageMask = 0` as an 8K page in a 16K pair rather than a 4K page in an 8K pair, so each
entry reached twice as far as it should and, being earlier in the linear scan, answered
for the entry above it. `Mars_Tlb.md` §1.1 has it.

The reverse-endian harness is the first thing to map a pair's two halves to unrelated
frames — one cached page and one uncached — and the first to place two pairs next to each
other. Both are ordinary things for an operating system to do and neither had appeared in
four slices of TLB work.

## 6. Where this leaves the run

**720 tests started, 160 failed, and every one of the 160 is a subsystem Mars has not
built or a decision it has taken** — cartridge DMA and the peripherals for Phase E, the
RDP for Phase D, the RSP for Phase C, and the caches, which Mars does not model and which
cannot pass. There is no CPU-level failure left anywhere in the corpus run.

That is a different claim from the one `Mars_Cop0.md` §1 made and had to qualify, and the
difference is `Mars_Corpus.md` §2: this one is about a run that reaches the end of what
Mars can attempt, and the census behind it is in §3 of that page.

## 7. What this slice does not do

- **No supervisor-mode or kernel-mode reverse-endian.** The bit is user-mode only here.
  The architecture agrees and the corpus only tests user mode, so this is measured for
  user and read for the rest.
- **No cacheability.** The corpus runs each reverse-endian case twice, against a cached
  and an uncached page, and Mars answers both identically because it models no caches
  (`Mars_Cpu.md` §13). Both halves pass; only one of them is being tested.
