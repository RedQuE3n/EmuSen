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

## 5. Writing entries

Three instructions write or read the array — one at the indexed entry, one at a
rotating entry, one reading an entry back into the registers — plus a probe that
answers with an index or with the top bit set when nothing matched. A handler uses the
probe to find out whether it is replacing a mapping or adding one.

The rotating write never lands below the wired count, which is how a handler reserves
entries it can rely on. Mars derives the rotation from the instruction count rather
than from a free-running counter, which is an approximation: hardware decrements a
register continuously, so a program that reads that register sees a value Mars does
not reproduce. Nothing reads it yet, and the corpus's own TLB tests are what will say
whether the approximation survives.

## 6. What is not modelled

- **64-bit addressing.** Comparisons mask to 32 bits, so the extended address spaces
  are not translated. The corpus has a separate section for those, and it will fail.
- **The wired register's effect on anything but the rotation.**
- **Page sizes are honoured in the mask** but only 4K pairs are tested; the larger
  sizes are implemented from the field layout rather than from evidence.
- **No cache or micro-TLB**, so every access walks the array.
