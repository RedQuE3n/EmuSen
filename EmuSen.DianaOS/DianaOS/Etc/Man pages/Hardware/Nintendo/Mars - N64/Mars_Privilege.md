# Mars — three modes, and an address map that is a function of which one you are in

*Landed 2026-09-16. The VR4300's kernel, supervisor and user modes: how the mode is
derived, the virtual address map each one sees, and the two things the other two modes
may not do. `Memory/Segments.cs`, with the mode derivation and the two gates in
`Cpu.Opcodes.Cop0.cs` and `Cpu.Dispatch.cs`. Tests in `MarsSegmentMapTests` and
`MarsCpuPrivilegeTests`, on the fixture in `PrivilegeFixture`.*

---

## 1. The mode is not the mode field

`Status` has a two-bit mode field, `KSU`, at bits 4:3 — 0 kernel, 1 supervisor, 2 user.
It is not what decides the mode. **`EXL` or `ERL` forces kernel mode regardless of
`KSU`**, which is what makes an exception handler able to run at all: the fault that
vectored into it did not change `KSU`, so without this rule a user-mode fault would
enter its handler still in user mode and fault again on the first thing the handler did.

That is also how the corpus enters a mode. It writes `Status` with the mode it wants
*and* `EXL` set, then `ERET`s into the program; `ERET` clears `EXL`, and the mode field
takes effect on the instruction after it. So `EXL` is both the reason the handler works
and the mechanism by which a mode is entered.

`KSU = 3` is reserved and no test establishes what it does. Mars reads it as kernel,
which is a choice and not a measurement.

**Each mode has its own 64-bit addressing bit**: `UX` (bit 5), `SX` (bit 6), `KX`
(bit 7). The bit that applies is the one belonging to the mode in force, so the same
`Status` word describes three different address maps depending on `KSU`.

## 2. The address map

The corpus carries this as a forty-five row table across three modes and both addressing
widths, and `MarsSegmentMapTests` keeps it row for row under the corpus's own case names.
An address is one of three things — **illegal** for this mode, **mapped** through the TLB,
or **direct** with the physical address taken from the bits themselves — and which of the
three depends on the mode, not on the address.

### 2.1 Thirty-two bit addressing

The address must be the sign extension of its own bit 31; that check is a mode and not a
rule, and it now lives in the map rather than beside it. Then, by bits 31:29:

| | kernel | supervisor | user |
| --- | --- | --- | --- |
| `0x00000000`–`0x7FFFFFFF` | mapped | mapped | mapped |
| `0x80000000`–`0x9FFFFFFF` | direct, cached | illegal | illegal |
| `0xA0000000`–`0xBFFFFFFF` | direct, uncached | illegal | illegal |
| `0xC0000000`–`0xDFFFFFFF` | mapped | mapped | illegal |
| `0xE0000000`–`0xFFFFFFFF` | mapped | illegal | illegal |

The two direct segments differ only in cacheing, and both strip to the low 29 bits.

### 2.2 Sixty-four bit addressing

Four regions, selected by bits 63:62, and each of the mapped ones is only forty bits
wide — an address above that in the same region is illegal rather than wrapping. That
forty-bit ceiling is what the corpus's `_gap` rows test, one per region.

| region | kernel | supervisor | user |
| --- | --- | --- | --- |
| `0x0000…` + 40 bits | mapped | mapped | mapped |
| `0x4000…` + 40 bits | mapped | mapped | illegal |
| `0x8000…`–`0xBFFF…` | direct, eight windows | illegal | illegal |
| `0xC000…` up to `0xC00000FF_7FFFFFFF` | mapped | illegal | illegal |

**The kernel's mapped region is two gigabytes shorter than the other two.** It stops at
`0xC00000FF_7FFFFFFF`, not at the forty-bit ceiling: `0xC00000FF_7FFFFFFC` is a TLB miss and
`0xC00000FF_80000000` an address error. This section first described all three as forty
bits wide, which was the pattern and not the measurement; the corpus's 64-bit TLB table
has the boundary row, and Mars had it wrong until that table was reached.

Above `0xFFFFFFFF_80000000` the 64-bit map stops describing regions of its own and
**repeats the 32-bit map exactly**, which is why `Segments.Decode` sends that window
straight back to the narrow decode rather than restating it. For supervisor mode that
window contains exactly one legal segment, `0xFFFFFFFF_C0000000`–`0xDFFFFFFF`, and the
narrow supervisor rule already produces it.

This closes the gap `Mars_Cop0.md` §8.1 recorded: Mars used to decide the segment from
bits 31:29 of the low word alone, so `0x00000000_80000000` under 64-bit addressing landed
in the direct cached segment instead of the mapped region it names. The test that
recorded the gap now asserts the fix.

### 2.3 The eight physical windows

The `0x8000…`–`0xBFFF…` region is eight windows of 2⁵⁹ selected by bits 61:59, each one a
direct view of physical memory, and **kernel mode only**. Only the low four gigabytes of
each window exist — the corpus tests one address per window and one address 2³² above it,
and the second is an address error every time.

The window index is the cache coherency attribute, and attribute 2 is the uncached one.
Mars models no caches, so nothing observes that yet; it is implemented because the map
says so, and the note is here because no test in this project can currently fail if it is
wrong.

## 3. Coprocessor zero belongs to the kernel

Supervisor and user mode raise Coprocessor Unusable on any coprocessor-zero instruction
unless `Status.CU0` is set; kernel mode needs no permission bit. The fault names
coprocessor **zero** in `Cause.CE`, which is to say it names no coprocessor at all, and
the corpus checks that field explicitly — it is the one place where "the field is zero"
is a positive assertion rather than a default.

## 4. The doubleword instructions are a privilege, not a width

Twenty-six instructions — every `D`-prefixed arithmetic and shift operation, and the
doubleword loads and stores including `LLD` and `SCD` — raise Reserved Instruction in
supervisor or user mode when that mode's addressing bit is clear.

**Kernel mode runs them with `KX` clear.** That asymmetry is the whole content of the
rule and it is not derivable from the name of the bit: `KX` governs kernel *addressing*
and nothing else, while `SX` and `UX` govern addressing *and* the instruction set. Mars
implements exactly that, and `MarsCpuPrivilegeTests` asserts the kernel case for all
twenty-six precisely because it is the one a plausible implementation gets wrong.

`DIV`, `DIVU`, `MULT` and `MULTU` are **not** restricted, though they write 64-bit
results into `HI` and `LO`. The corpus lists all four alongside the restricted ones for
that reason, and Mars's test keeps them for the same reason: the rule is about the
instruction, not about the width of what it produces.

**What is inferred rather than measured.** The corpus's list is twenty-six instructions
and does not include `DSLL32`, `DSRL32` or `DSRA32`; Mars restricts them anyway, on the
grounds that they are the same three shifts with 32 added to the shift amount and share
their encodings' family. `DMFC1`, `DMTC1`, `DMFC0` and `DMTC0` are **not** restricted
here, and no evidence either way was found — the two coprocessor-zero forms are moot in
the modes that matter, because §3 refuses them first.

The check runs before the main decode and costs nothing in kernel mode, which is where
every game and the whole corpus spend almost all of their time: the classification is
only reached once the mode is known to be restricted.

## 5. Testing a mode you cannot execute in

A test that runs in user mode cannot be assembled at `0xFFFFFFFF_80000000` like every
other Mars instruction test, because that address is illegal in user mode — the *fetch*
faults before the instruction under test is reached. `PrivilegeFixture` therefore maps
one TLB pair over a page in the low mapped segment, which all three modes may execute,
and runs the program from there.

**The first version of that fixture was wrong in a way worth recording.** It put the
program at virtual `0x2000` and mapped the pair containing it — and a 4K TLB pair covers
*two* pages, so the same entry also mapped `0x1000`, which is the address a third of the
table's rows use to test a TLB *miss*. Those rows passed for the wrong reason. Moving the
program to `0x10000`, as the corpus does, separates them; the corpus presumably chose that
address for exactly this reason.

## 6. What this slice does not do

- ~~**Reverse-endian is not implemented.**~~ — landed in the slice after this one,
  `Mars_ReverseEndian.md`. The rule predicted here was half right and is worth keeping as
  a retired prediction: it said the effective address is XORed with `8 - size`, which is
  true of every aligned access and **false of the merging family**, which mirrors as a
  byte whatever its width. The prediction was read off the corpus's helper function
  rather than off its result tables, and the helper only describes the aligned cases the
  caller uses it for. `Mars_ReverseEndian.md` §2. The two things this bullet got right —
  that instruction fetch is affected, and that `BadVAddr` keeps the unmirrored address —
  both held.
- **The TLB still compares only the low 32 bits of a virtual address.** The 64-bit map
  now delivers full 64-bit addresses to it, so two addresses differing only above bit 31
  match the same entry. No test in the failing set covers it; the corpus's `tlb64` group
  is where it will surface.
- **No mode-dependent cacheing.** §2.3.
