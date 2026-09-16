# Mars — the handoff, and the first real ROM to run

*Phase A's last slice, landed 2026-09-15. **The hardware corpus now boots through its
own bootcode and reaches its entry point**, which is the first time Mars has executed
anything not written by this project.*

---

## 1. What the handoff does

The real machine runs a boot ROM inside the PIF, which performs a security handshake
with the cartridge and then copies the cartridge's first four kilobytes into the
signal processor's data bank and jumps into it. Mars does the copy and the jump, and
skips the handshake.

`Mars_Gameplan.md` §4.1 argued for this on three grounds: the boot ROM is
copyrighted and most users do not have it, the handshake proves nothing about the
emulated machine, and a missing firmware image must never be fatal. The corpus
settles a fourth: it carries **libdragon's open-source bootcode**, so no proprietary
image is needed to run the one thing Mars is graded against.

## 2. The register state a bootcode is entitled to find

A stack pointer in the data bank, a television-standard flag taken from the ROM
header's destination code, a seed value, and a coprocessor-zero state with the
coprocessors enabled. These are the conventional post-boot values rather than
measured ones — no document the survey found states them
(`Mars_Documentation.md` §5), and they are what emulators agree on.

That agreement is exactly the kind of evidence this project distrusts elsewhere, and
it is recorded as such: **if a game misbehaves in a way that traces back to boot
state, this section is a suspect and not a foundation.**

## 3. What running the real bootcode found

Three things, in the order they appeared.

**The cache instruction.** The bootcode invalidates caches in a tight loop before it
does anything else. Mars models no caches, so the instruction is a no-op — every
cache operation is complete before it starts. The corpus's own cache tests will fail
for the same reason and should, which is the correct relationship between a
simplification and the tests that cover it.

**Load-linked and store-conditional.** The corpus's README offers to let an emulator
fake these as ordinary loads and stores, and says plainly that doing so is wrong.
They are implemented properly instead — the load arms a flag, the store lands only if
nothing has broken it, and an exception breaks it — because the correct version is a
dozen lines and the fake would have to be undone later with a test suite already
depending on it.

**Nothing else.** The three admission requirements the test-oracle page named were
already in place, and the bootcode exercised all three for real rather than through
our own tests: the select register short-circuit, zero reads above installed memory,
and the transfer that wraps inside the instruction bank.

## 4. Where it stops, and why that is the right place

The corpus's very first instruction after boot is a **coprocessor-1 control move** —
it configures the floating-point unit before it does anything else. Mars has no
coprocessor 1, so that is a reserved-instruction exception, and it is where execution
stops today.

That is Phase B, which the plan already places next and already argues should begin
with a software floating-point implementation rather than host doubles
(`Mars_Cpu.md` §2.1, `Mars_References.md` §5.2). The boot slice therefore ends
exactly at the phase boundary the plan drew before any of this was written, which is
a small piece of evidence that the boundaries were drawn in the right places.

Two tests hold this position: one asserts the boot reaches the entry point without
faulting, the other asserts that the first instruction Mars cannot run is a
coprocessor-1 operation. The second inverts when Phase B lands, and is meant to.

## 5. What the boot does not do

- **No security handshake**, and therefore nothing that depends on its results.
- **No PIF ROM path.** The firmware seam exists project-wide (`EmuSen_Firmware.md`)
  and Mars does not use it; a real boot ROM would be an alternative entry rather than
  a rewrite.
- **No reset types.** A cold boot is the only kind.
- **The bootcode is the cartridge's**, so a ROM carrying a different one gets a
  different boot. Nothing here is specific to the corpus except that the corpus is
  what has been run.

> **Retired, 2026-09-16.** §4's "where it stops" is no longer where it stops. The
> coprocessor-1 register file landed the following day and the ROM now runs past the
> boot handoff into its own test suite, printing verdicts — `Mars_Fpu.md` §9. The
> section stays because the prediction it made was right: the stop was a phase
> boundary rather than a defect, and the phase that followed it was the one the plan
> had already placed next.
