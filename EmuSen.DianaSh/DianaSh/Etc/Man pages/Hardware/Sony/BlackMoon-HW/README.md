# BlackMoon Hardware (the shared hardware of the Sony cores)

Not a console core: the chips more than one Sony core uses, one crate each in `EmuSen/Cores/Sony/BlackMoon-HW/`, so
that each can be used alone. Planned in `Diamond_Plan.md` §9.1, since Diamond is their first user. *Decided 2026-10-11
(`Diamond_Plan.md` §11 Q10): crates shared between the Sony cores live in BlackMoon Hardware, named in the Black Moon
Clan's manner as Beryl Hardware is for the Sega cores; the MIPS decoder, disassembler and assembler, which the N64's
core may share, are `emusen-mips` in `Cores/Shared/`, built at Diamond's stage 2, and hold no execution.* Each crate's
own record is beside this page.

- [`BlackMoon_R3000.md`](BlackMoon_R3000.md) — `blackmoon-r3000`, the R3000A, graded by SingleStepTests' r3000 suite
  (registers, both delay slots, every cycle), with amidog's CPU tests and ps1-tests as independent checks.
- [`BlackMoon_GTE.md`](BlackMoon_GTE.md) — `blackmoon-gte`, the GTE, graded by ps1-tests' `test-all` and amidog's GTE
  test, bit for bit.

## The interface every processor keeps

The interface of `Beryl-HW/README.md`, which the Sony crates keep unchanged:

- **A bus trait in the processor's own terms**, which the core implements: the R3000A's instruction fetches, its data
  reads and writes by width (byte, halfword, word), its internal cycles, its interrupt request, and coprocessor 2's
  register transfers and commands.
- **The bus owns the clock.** The processor reports every access and every internal cycle through the bus, which
  advances its own time and adds its wait states; the processor keeps no clock. One implementation serves a flat test
  bus and the console's bus alike (`Diamond_Plan.md` §5.1, §5.2).
- **The debugger through `emusen-native`'s `Observer`**: `step` is `step_observed` with `Unobserved`, so the plain step
  is compiled with no hook; the observed step reports the instruction boundary, each stored byte, and calls,
  exceptions and returns. The register names a debugger lists are the crate's constant, in `debug_registers`' order.
- **State through `emusen-native`'s `State`**: each processor's registers are a fixed block of its core's state,
  written with `StateWriter` and read with `StateReader`, a short block changing nothing; the layout is pinned by a
  test in the crate.
- **The single-step oracle lives in the crate**, as unit tests that find the corpus through `EMUSEN_BLACKMOON_CORPUS`
  (`~/.cache/emusen/probe/diamond` on the desktop) and pass unrun without it; the processor's own run is skipped while
  the crate's `BUILT` is false, its replay controls are not.

## A coprocessor

The GTE has no bus of its own: the R3000A reaches it through its bus's coprocessor 2 calls, and the core joins the
two. Its contract, first kept by `blackmoon-gte`:

- **The caller owns the clock.** A command returns the cycles until its results can be read; the core holds the
  processor's next access to coprocessor 2 until then, which is the interlock `Diamond_Plan.md` §5.2 times. Nothing in
  the crate keeps time.
- **The two crates do not depend on each other.** The R3000A's bus names coprocessor 2's operations in the
  processor's terms (`cop2_data`, `set_cop2_data`, `cop2_control`, `set_cop2_control`, `cop2_command`) and the core
  answers them with the GTE's; a core with another coprocessor 2, or none, uses the processor unchanged.
- **Registers and state** as the processors': the 64 registers' names from psx-spx, a fixed state block.
- **The enhancement layer's observer** (`Diamond_Plan.md` §6.2, the precision side-channel) is stage E1's, not the
  debugger's `Observer`: it is added then as a second generic parameter, compiled out when unobserved.
