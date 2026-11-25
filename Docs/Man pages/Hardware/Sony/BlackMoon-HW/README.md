# BlackMoon Hardware (`EmuSen/Cores/Sony/BlackMoon-HW/`)

Shared Rust chip crates reused across Sony console cores (`diamond` for PlayStation and `sapphire` for PlayStation 2). Each crate models one hardware unit in isolation so any core or standalone test harness can drive it without pulling in another console's bus or memory map.

| Crate | Manual | Role | Consumers |
|---|---|---|---|
| `blackmoon-r3000` | [`BlackMoon_R3000.md`](BlackMoon_R3000.md) | MIPS I R3000A CPU core (registers, delay slots, COP0, debug unit, multiplier timing) | `diamond` (PlayStation CPU), `sapphire` (PlayStation 2 IOP) |
| `blackmoon-gte` | [`BlackMoon_GTE.md`](BlackMoon_GTE.md) | PlayStation Geometry Transformation Engine on COP2 | `diamond` |
| `blackmoon-spu` | [`BlackMoon_SPU.md`](BlackMoon_SPU.md) | Shared SPU voice engine (SPU-ADPCM, Gaussian interpolation, ADSR/sweeps, noise, pitch modulation, reverb) | `diamond` (SPU), `sapphire` (both SPU2 cores) |

ISA tables, disassembly, and assembly for MIPS instructions live in the shared `emusen-mips` crate ([`../../Shared/CpuTables.md`](../../Shared/CpuTables.md)), which holds instruction data with no execution logic.

## Shared Processor Contract

`blackmoon-r3000` follows the same shared-chip contract as [`Beryl-HW`](../../Sega/Beryl-HW/README.md):

- **The bus owns the clock.** The processor keeps no cycle counter of its own. Every instruction fetch, data access, and internal stall (`Bus::idle`) is reported through the `Bus` trait; the console bus advances its own clock and applies region wait states. The processor reads `Bus::cycles` only to time its multiply/divide interlock.
- **No console-specific caches or memory maps.** The I-cache, scratchpad, memory wait states, and bus-error decode belong to each console's `Bus` implementation. When `SR.IsC` is set, the processor routes loads and stores to `Bus::isolated_read` and `Bus::isolated_write`.
- **Zero-cost debugging via `emusen_native::debug::Observer`.** `R3000::step` delegates to `R3000::step_observed` with `Unobserved`, so normal execution compiles without debugger hooks. `DEBUG_REGISTERS` exports the canonical register names in `debug_registers` order.
- **Fixed state layout via `emusen_native::State`.** Each crate serializes its architectural state through `StateWriter` and `StateReader` with a unit-tested fixed layout; a truncated buffer returns an error without mutating state.
- **Standalone single-step harness.** `blackmoon-r3000` carries a unit-test harness over the shared `emusen-singlestep` crate (discovered via `EMUSEN_BLACKMOON_CORPUS`) that checks registers, delay slots, and bus cycles independently of any console bus.

## Coprocessor Contract (`blackmoon-gte`)

`blackmoon-gte` has no bus of its own and does not depend on `blackmoon-r3000` (nor vice versa):

- **Core-mediated COP2 seam.** `blackmoon-r3000::Bus` exposes `cop2_data`, `set_cop2_data`, `cop2_control`, `set_cop2_control`, and `cop2_command`. The console core connects these methods to `Gte` and provides `blackmoon_gte::COPROCESSOR` to `blackmoon_r3000::dialect` for disassembly.
- **Caller-owned interlock.** `Gte::command` and `Gte::command_observed` compute all result registers immediately and return the command's cycle count. The core tracks the ready deadline on the bus clock and stalls subsequent COP2 accesses until the command completes.

## Precision Side-Channel Seam

Both `blackmoon-r3000` and `blackmoon-gte` expose zero-cost observation hooks used by `diamond`'s precision geometry layer (`precision/`):

- **`blackmoon_gte::Observer::projected(&Projection)`** reports each vertex projected by `RTPS`/`RTPT` (camera-space sums, `H / SZ3` quotient, full-precision screen sums, `SZ3`, `SXY2`, `H`, and `OFX`/`OFY`) without altering architectural results. `Gte::command` uses `Unobserved`.
- **`blackmoon_r3000::Bus::MOVES` and `Bus::moved(Move)`** report register-to-memory, memory-to-register, COP2 data transfers, register copies, and constant shifts (`Move::Shift`). Because `Bus::MOVES` is an associated `const bool` defaulting to `false`, buses that do not track vertex flow compile with zero overhead.
