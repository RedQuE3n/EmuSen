# Mars — the half line, and the interrupt a game waits for

*Landed 2026-09-17. Phase E's fourth slice, and the first thing in the video interface that runs without
anyone asking it to. The code is `Vi/Vi.Timing.cs` and two lines elsewhere — a call in `MemoryBus.Tick` and a
clear in `Vi.Write32`; the grading is `MarsViTimingTests` and a prediction an earlier slice wrote down.*

***§0 is the part to read first.** Nothing that graded the three slices before this one can grade it, and
what replaced them is weaker in one way and stronger in another.*

---

## 0. What grades this, when the differential cannot

**`MarsViDifferentialTests` cannot reach a single rule in this slice.** The dump drives the reference by
writing registers and asking for a screen update; there is no clock in the format and no way to express the
passage of a half line. angrylion could not use one if there were: its `emucontrolsvicurrent` is set once at
startup and never updated again, so it assumes the host maintains the current-line register and reads the
field out of it. **The reference has no timing to compare against.**

**The hardware corpus cannot either.** `Mars_Corpus.md` §3's census names every group that still fails — 45
cache groups that never can, 19 for cartridge memory and the peripheral interface, 13 for PIF RAM, MI and
RDRAM registers — and **no group tests the video interface**. That was checked before the slice was written
rather than hoped for afterwards.

**So three things grade it, and the first is the only one that reaches the whole mechanism at once.**

- **A prediction registered before the fact.** When `Mars_Microcode.md` §3 added arbitrary interrupt pulses
  to make commercial microcode run, it wrote down that *"when Phase E builds the two devices, the pulses
  should be deleted and the test should still pass; if it does not, the difference is information about those
  devices."* §3 is that experiment.
- **Thirteen unit tests of the semantics**, taken from the register documentation and the FPGA rather than
  from a reference implementation, since there is no reference implementation to take them from (§4).
- **The corpus's verdict staying exactly where it was**, which is a negative result and is why it counts: the
  corpus test asserts `Failed 152 of 4637` to the number, and a clock that now ticks in every test which
  steps the processor moved neither figure.

**What none of them establishes is the rate.** The tests check that the counter advances at the rate the
registers ask for, and §1.1 derives that rate from three published clock frequencies, but nothing here
measures a console. A game that depends on the exact number of processor cycles in a half line would not be
graded by anything in this slice.

## 1. The half line

**The signal walks down the picture whether or not anything is scanning it**, and the interface keeps count
of where it is. The count is in half lines, because an interlaced signal's two fields are offset by one.

### 1.1 The rate

**A line lasts `VI_H_SYNC`'s low twelve bits of interface clocks.** The interface's clock is not the
processor's: 48.681818 MHz against 93.75 MHz for NTSC, and 49.65653 MHz for PAL, where the line is also
longer. So a half line is

```
VI_H_SYNC × processor clock ÷ (2 × interface clock)
```

processor cycles — 2978.4 of them at the NTSC defaults, and 2998.9 at the PAL ones. **Mars carries the
remainder rather than rounding the period**, accumulating `cycles × interface clock × 2` against a threshold
of `VI_H_SYNC × processor clock`, both in integers. A rounded period would drift by a line every few
thousand, which is the kind of error that shows up as a game losing a frame an hour after it starts. Since Phase G
the sum is settled rather than stepped — brought up to date at each half line, before a register write and before a
save — in the same integers, so every half line turns on the tick it always did (`Mars_Performance.md` §9).

**The defaults are not assumed, and a game confirmed them.** Tracing Wave Race through the boot it already
passes, the values it writes are `VI_V_SYNC = 0x20D` and `VI_H_SYNC = 0xC15` — 525 and 3093, which are
exactly what the FPGA core resets those registers to for NTSC (§5). Mars reads both from the registers, so a
game that programs something else gets what it asked for.

### 1.2 What the register reports

**The current-line register is not the count.** It reports the count with its low bit taken off, and puts the
field in that bit instead:

```
(half line & ~1) | field
```

So **the register never reports an odd half line**, a progressive signal never sets bit 0 however many half
lines pass, and an interlaced one sets it for every other field. The count wraps at `VI_V_SYNC`, and the
field changes when it wraps — only when serrate is set, because a progressive signal has one field.

## 2. The interrupt

**The interface raises the video interrupt when the half line reaches the one the interrupt register names**,
and the comparison takes the low bit off the half line first. Two consequences, both tested:

- The interrupt is tested **once a line**, not twice.
- **An interrupt register naming an odd half line is never reached.** A game that writes one gets no
  interrupts at all.

**Only a write to the current-line register clears it.** Not a write to the interrupt register, not the count
wrapping, not the next frame — the handler's write is the only way out, which is what makes the interrupt
level-triggered through `MiInterface` rather than a pulse.

**Mars stores what is written to the current-line register, and hardware does not.** This is a knowing
departure and it is here for the differential's sake: angrylion has no clock (§0), so the only way to tell it
which field a frame belongs to is to write the register, and `Mars_Video.md` §2.3's field comes from that
same register so that the two sides can be compared at all. On a running machine the counter overwrites the
written value within a half line. **What it costs:** a game that writes the current-line register and reads
it back before the next half line sees its own value where a console would show the counter's. Nothing in the
suite can see that, and nothing in this slice claims otherwise.

## 3. The stand-in, deleted

**`Mars_Microcode.md` §3's video pulse is gone, and its test still passes.** Wave Race still hands its first
display list to the display processor and still clears its depth buffer, now driven by an interface that
raises the interrupt on its own. The prediction was registered in an earlier slice, against a test written
before this device existed, and it held.

**The intermediate result is worth as much as the final one.** With the stand-in *and* the real interface
both raising the interrupt, the test **fails** — no graphics task reaches the RSP at all. The stand-in clears
the interrupt five thousand instructions after raising it, which also clears the genuine one before the
game's handler has dealt with it, and the game stops advancing frames. That failure is the evidence that the
interface is now doing the work rather than sitting alongside something that was: if the pulse were still
carrying the game, removing it would have broken the test, and adding a second source would not have.

**A trace of the boot says the same thing from the other side.** Over twenty million instructions the
interface raises the interrupt nine times and the game clears it nine times — about one per frame at this
clock, in balance, with the count advancing between them. Nothing storms and nothing is left asserted.

**The serial pulse stays.** `Mars_Microcode.md` §3 wrote its prediction about *two* devices, and this slice
builds one of them; the serial interface has no device yet, so half the stand-in remains and the prediction
is only half tested. That is stated here so the remaining half is not mistaken for a pass.

## 4. What the tests say

**Thirteen tests, no reference tools, and nothing to compare against but the semantics.** They check the rate
in both regions, that a progressive signal never reports a field, that an interlaced one changes field when
the count wraps, that the interrupt is raised at the named half line and not at an odd one, that only the
current-line register clears it, that the count wraps at the vertical sync, and that an interface no game has
programmed neither counts nor divides by its own zero. Two of the thirteen were added by the breakage round
rather than before it (below).

**Eighteen breakages, seventeen of which apply, and every one is caught.** The round is run against the
eleven tests *and* the commercial-microcode test together, which is the only way a rule about the interrupt's
timing can be reached at all — and three of the seventeen are caught by the game rather than by a unit test,
including the one that removes the interrupt's raise.

**Three survived the first round, and all three were the tests' fault rather than the code's.**

- *A signal with no vertical sync does not count.* The test for an unprogrammed interface set neither sync,
  so the line-length guard returned first and the vertical-sync guard was never reached. A test with a line
  but no vertical sync reaches it: without the guard every advance wraps immediately, and an interrupt
  register naming half line zero then fires forever.
- *The interrupt register is ten bits.* No test named a half line above nine bits, although a picture has
  525 of them. One that names half line 0x208 does.
- *The comparison takes the low bit off the half line.* **Not a gap — a redundancy, and it was removed.**
  The comparison is already guarded to even half lines, so masking the low bit off one of them decides
  nothing. Taking the mask out left the guard carrying the rule alone, which moved it from being caught only
  by the commercial game to being caught by the unit test for an odd interrupt register. That is the better
  place for it: a rule held by a game's boot is held by something that could stop holding it for unrelated
  reasons.

**Two of the thirteen tests failed on their first run, and both were the prediction rather than the code.** I had
written the expected counts as the number of half lines that had passed, forgetting that the register reports
them with the low bit removed (§1.2) — so 523 half lines read back as 522. The rule was in the code and in
the page before the test was written; it was the test's arithmetic that had not caught up. Recorded because
the alternative reading — that the code was wrong twice and got fixed — is the one a reader would otherwise
assume.

## 5. The referee

**With no reference to grade against, the FPGA core is the only other implementation of this mechanism, and
this section is closer to a source than to a witness.** Everything in §1.2 and §2 is in `VI.vhd`:

- **The register's own documentation.** §115–116 comments the two registers in the author's words: the
  interrupt register is *"interrupt when current half-line = V_INTR"*, and the current-line register is
  *"current half line, sampled once per line (the lsb of V_CURRENT is constant within a field, and in
  interlaced modes gives the field number — which is constant for non-interlaced modes). Writes clears
  interrupt line."* That is §1.2 and §2 together.
- **The comparison, with the low bit taken off.** §233: `if (newLine = '1' and (VI_CURRENT(9 downto 1) & '0')
  = VI_INTR)`. The `& '0'` is the masking, and the `newLine` gate is why it happens once a line.
- **The clear, and only the clear.** §306: a write to `0x04400010` does `irq_out <= '0'` and nothing else —
  the counter is not written. This is the rule Mars departs from, and §2 says why.
- **The line's duration in interface clocks.** §122: `VI_H_SYNC_LENGTH`, twelve bits, *"total duration of a
  line"*.
- **The region defaults.** §213–223 resets PAL to a 625-line sync with a line of 3177 and NTSC to 525 with
  3093 — the two pairs §1.1 quotes, and the same 525-against-625 split that `Mars_Video.md` §1's test for PAL
  already relied on.

**What the referee marks as unknown, Mars inherits.** `VI_videoout.vhd` §295 carries a comment from its author saying he does
not know when the field bit is set — *"need to find when interlace sets bit 0, can't be instant, otherwise
Kroms CPU tests would hang in infinite loop"* — and that the obvious answer is wrong. Mars sets it when the
count wraps, which is the simplest rule consistent with everything here, and it is **not** evidence about
the console. It is the second thing in Phase E a console test should settle, after the fetch artefact of
`Mars_VideoFilter.md` §3.

## 6. What is not here

- **`VI_H_SYNC`'s leap pattern**, five bits the PAL signal uses to distribute an odd half line across
  frames. Mars reads the line length and ignores the leap, so a PAL signal's count is uniform where a
  console's is not.
- **The vertical and horizontal burst registers**, and the two sync widths, which are read and stored and
  drive nothing.
- **The audio, serial and peripheral interfaces**, which are the rest of Phase E.
- **`GetFrameBufferRgba`**, still not wired to the raster: the interface now keeps time and draws a picture,
  and nothing outside Mars can see either.
