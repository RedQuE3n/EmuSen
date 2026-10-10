# Mars — the translation buffer

*Phase A's fifth slice, landed 2026-09-15. With it, the three mapped segments
translate instead of faulting wholesale. `EmuSen/Cores/Nintendo/Mars - N64/Cpu/Core/Tlb.cs`,
tested by `MarsTlbTests`.*

---

## 1. Entries come in pairs

Thirty-two entries, each describing **two** consecutive pages with one comparison: a
shared virtual page number and size mask, and two separate physical translations with
their own valid, writable and global bits. The bit immediately above the page offset
selects which half an address lands in.

Pairing halves the comparisons a lookup needs, and it is why the register interface
has two low-order registers rather than one — a detail that looks arbitrary until the
pairing is the thing being described.

### 1.1 The mask describes the pair, and a page is half of it

`PageMask` is the mask of the **pair**, not of one page: zero means a 4K page and an 8K
pair, so the page mask is `(PageMask | 0x1FFF) >> 1` and the bit choosing the half is one
above that. Mars had the two the other way round — it used `PageMask | 0x1FFF` as the page
mask and shifted *up* for the pair — which made every entry describe 8K pages in a 16K
pair.

**It survived four slices of TLB tests because the error is invisible to the common
case.** A pair whose two halves point at consecutive frames translates identically under
either reading: an 8K page at frame *n* and two 4K pages at frames *n* and *n+1* put the
same physical byte under the same virtual one. Every test in `MarsTlbTests` mapped its
pair that way, and so does most real software.

Two things do see it. A pair whose halves name **unrelated** frames — which is what the
corpus's reverse-endian harness builds, one cached page and one uncached — reads the wrong
half. And an entry's inflated pair mask **swallows the entry above it**: with a 16K reach,
a pair at `0x20000` matched addresses in the pair at `0x22000` and, being earlier in the
scan, answered for it. That is how it presented — not as a wrong byte within a page, but
as data accesses returning instruction bytes from a different mapping entirely.

`MarsTlbTests` now maps a pair to two unrelated frames and places two pairs adjacently,
which are the two shapes that can tell the readings apart.

## 2. The lookup is a linear scan

Thirty-two entries in order, first match wins. That is what the hardware does
associatively and what Mars does one at a time, which is correct and slow. Phase G may
want a cache in front of it; nothing here should assume one, and the man page for that
optimisation will have to explain what invalidates it.

A global pair ignores the address-space identifier, which is how a shared mapping
stays shared across a context switch. A non-global pair matches only its own, and
both halves of that are tested.

## 3. Three failures, not one

The interesting decision in this slice. A lookup does not simply succeed or fail:

- **No entry matches at all** — a *refill*, which takes its own vector. This is the
  common case on a machine using the TLB normally, and hardware gives it a separate
  door precisely so the handler can be short.
- **An entry matches but is not valid** — a real fault, taking the general vector.
  The mapping exists and describes a page that is not currently there.
- **A store hits a page that is not writable** — its own exception code again, and
  the general vector. A load from the same page still works, which has a test,
  because the distinction is about the access rather than the page.

Vectoring previously inferred "refill" from the exception code, which conflated the
first two: they carry the *same* code and differ only in whether an entry was found.
The exception now carries the answer explicitly, and mutating the invalid case to
report a refill reddens exactly the test that separates them.

## 4. What a handler is given

A failed lookup leaves the faulting page in the register a refill handler reads, and
folds it into the context register at the offset that makes a handler's page-table
index a single shift away. That is the whole reason those registers are shaped the way
they are: the handler is meant to be a few instructions, not a calculation.

Both are written on **every** failure rather than only on refills, which matches what
the hardware does and costs nothing.

> **Wider than this said, 2026-09-17.** Not only every *TLB* failure: an address error
> writes the faulting page into `EntryHi` as well, from the same address by the same rule.
> §7.6.

## 5. Writing entries

Three instructions write or read the array — one at the indexed entry, one at a
rotating entry, one reading an entry back into the registers — plus a probe that
answers with an index or with the top bit set when nothing matched. A handler uses the
probe to find out whether it is replacing a mapping or adding one.

The rotating write never lands below the wired count, which is how a handler reserves
entries it can rely on.

*Retired 2026-09-17 — the prediction below was answered, and the answer was no.*

> Mars derives the rotation from the instruction count rather than from a free-running
> counter, which is an approximation: hardware decrements a register continuously, so a
> program that reads that register sees a value Mars does not reproduce. Nothing reads it
> yet, and the corpus's own TLB tests are what will say whether the approximation
> survives.

It did not survive, and not for the reason it anticipated. The rate was never the
problem — no test measures it. What failed was that the register could not be *read* at
all: a program reading `Random` got whatever had last been stored there, which was
nothing, so the corpus's wait for ten distinct values timed out. The rotation also used
only five bits of `Wired`, so the range above 31 that hardware produces never appeared.
§7.4 is what replaced it.

## 6. What is not modelled

- ~~**64-bit addressing.**~~ — landed, §7.5. The comparison now reads the region bits and
  the whole forty-bit page number, and a refill under 64-bit addressing takes its own
  vector. The corpus's 64-bit TLB section passes.
- **The wired register's effect on anything but `Random`.**
- ~~**Page sizes are honoured in the mask** but only 4K pairs are tested~~ — the corpus
  now tests every size from 4K to 16M, and all of them pass. What it also showed was that
  an entry does not keep the mask it was given (§7.2), which the field layout does not
  predict.
- **No cache or micro-TLB**, so every access walks the array.

## 7. What an entry keeps, and what the registers around it keep

*Landed 2026-09-17. The corpus's TLB register section, which no run had reached until the
first complete one (`Mars_Corpus.md` §3). Tests in `MarsTlbRegisterTests`.*

Every rule here is narrower than the field layout suggests, and every one was a failing
corpus group. The register and the entry are different storage: a value written to a
register and then to an entry with `TLBWI` is narrowed twice, differently, and `TLBR`
shows the second narrowing rather than the first.

### 7.1 The registers

| register | keeps |
| --- | --- |
| `EntryLo0`, `EntryLo1` | `0x3FFFFFFF` |
| `PageMask` | `0x01FFE000`, raw |
| `EntryHi` | `0xC00000FF_FFFFE0FF` — the region bits, a forty-bit page number and the ASID |
| `Random` | nothing: writes are ignored |

`EntryHi` keeps the same bits whether it is written with `DMTC0` or `MTC0`, consistent with
a 32-bit coprocessor move carrying the whole source register (`Mars_Cop0.md` §2.1).

### 7.2 An entry keeps its page mask in whole pairs

`PageMask` holds six pairs of bits, one per size step from 4K to 16M. **An entry keeps each
pair as both bits set or both clear, decided by the higher bit of the pair alone.** A
written mask of `0b01` is stored as `0b00`; `0b10` is stored as `0b11`; the lower bit of a
pair is simply ignored. The register itself keeps the raw bits; only the entry normalises
them, so the difference is visible only through `TLBR`.

The entry's page number is also narrowed by its own mask: the bits the mask covers are
cleared from the stored `EntryHi`. So a 16K entry written with `EntryHi = 0xFFFFE0FF` reads
back `0xFFFF80FF`. Neither rule is derivable from the register's field layout; both are
in the corpus's `TLBWI`/`TLBR` round-trip vectors, which `MarsTlbRegisterTests` keeps.

### 7.3 Twenty frame bits, and one global flag for the pair

The registers keep thirty bits of each `EntryLo`; an entry keeps twenty-six — a 20-bit
frame number and the cache, dirty, valid and global flags. And **the global flag is stored
once per entry, as the AND of the two halves**, and read back into both. So an entry
written with one half global and the other not reads back with neither global.

Mars's TLB already treated an entry as global only when both halves said so, so the
*matching* was right; what was wrong was the storage, which kept both flags separately and
reported them back unchanged.

### 7.4 Random is a counter, and Wired is where it turns round

`Random` counts down from 31 and, on reaching `Wired`, starts again at 31, so the values it
takes are exactly the entries a random write is allowed to land on. When `Wired` is above
31 there is nowhere in the counter's path to turn round until it has wrapped through the
whole six-bit range and come back down to `Wired`, so it runs 31 down to 0, then 63 down to
`Wired`. The corpus tests both: at least ten distinct values in `[Wired, 31]` for every
`Wired` up to 31, and most of `[0, 63]` for values above.

Mars computes it rather than storing it, the same way `Count` is computed
(`Mars_Memory.md` §3.1): elapsed instructions since `Wired` was last written, modulo the
length of the cycle. `TLBWR` writes to whatever it reads.

**What is chosen rather than measured.** That the counter decrements once per instruction,
and that writing `Wired` restarts it at 31. The corpus's tests are built to be insensitive
to both — they sample through a noise loop precisely because the rate is not something a
test can pin without a cycle-accurate model — so both are recorded here as choices.

### 7.5 A match reads sixty-four bits, and a 64-bit miss has its own door

An entry matches on the region bits (63:62) and the whole forty-bit page number, not only
on the low word. Two addresses that differ only above bit 31 — `0x00000001_F0000000` and
`0x00000003_F0000000` — are different pages, and an entry for `0x40000000_DEA00000` does not
answer for `0x00000000_DEA00000`. The probe follows the same rule. This closes the gap §6
recorded when the 64-bit segments first decoded.

**A refill under 64-bit addressing vectors to offset `0x080`**, the extended refill vector,
rather than `0x000`. The bit that decides is the addressing bit of the mode that *missed*,
which has to be read before the fault raises the exception level, because raising it makes
every mode kernel. Mars had one refill vector.

The corpus sets all three addressing bits together, so it measures that a 64-bit kernel
miss takes the extended vector. That a user-mode miss with only `UX` set takes it, and one
with only `KX` set does not, is the architecture's rule and is tested here as such — **read,
not measured**.

### 7.6 An address error fills in EntryHi as well

The corpus's 64-bit miss table has fourteen rows that are address errors rather than
misses, and every one expects `EntryHi` to hold the faulting page — region bits and page
number, computed exactly as for a miss. Mars wrote `EntryHi` only on TLB failures.

That is the same mistake `Mars_Cop0.md` §7 records for `BadVAddr`, `Context` and
`XContext`, fixed there for three registers and not carried to the fourth, because
`EntryHi` was written in a different place — beside the TLB lookup rather than beside the
other three. It now lives with them, and the separate path is gone.

No 32-bit test asserts `EntryHi` after an address error in either direction; Mars applies
the rule to every address-related exception, on the reading that it is one mechanism.
