# Sapphire (Sony PlayStation 2)

The PlayStation 2 core, in Rust on the core ABI v1. Planned 2026-10-23
(`EmuSen.DianaSh/DianaSh/Etc/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Plan.md`); stage S1, a stub
marked in development that recognises an EE program or a PlayStation 2 disc and emulates nothing, built 2026-10-24
and recorded in `Sapphire_Native.md` beside the plan. `cargo test --release` runs its tests.

`programs/` holds the project's bare EE programs for the referees (`Sapphire_Referees.md` §4, from stage S3):
`cargo run --release --example elf -- programs/record.s record.elf` assembles one with `emusen-mips` into an ELF, and
`EmuSen.WiseMan/Reference/analysis/ps2referee.py` runs it on a referee and reads its record.

`src/ee/` is the EE core's interpreter and timing model (stage S6, `Sapphire_Native.md` §16), with COP0, the TLB, the
caches, the physical memory map, the counters and the breakpoints since stage S7 (§17); the machine does not run it yet. `tests/s6/` generates the stage's programs over every integer encoding with the record the EE Core
manual's definitions give, and `tests/interpreter.rs` grades the interpreter by them; `cargo run --release --example s6
-- <folder>` writes them as ELFs with their expected records for the referees. `tests/s7/` and `tests/cop0.rs` do the
same for stage S7's programs, written by `cargo run --release --example s7 -- <folder>`. `src/iop/` is the IOP (stage
S21, `Sapphire_Native.md` §22): `blackmoon-r3000` on Sapphire's own IOP bus with its cache, INTC, DMAC and timers;
`tests/s21/` generates the stage's bare IOP programs, `tests/iop.rs` grades them, and `cargo run --release --example s21
-- <folder>` writes each inside an EE program that places it in IOP RAM for the referees. `src/gs/` is the GS (stage
S16, `Sapphire_Native.md` §24): its registers, its local memory in every format's page and block layout (`gs::local`,
the layout's one owner) and the three transfers; `tests/gs.rs` grades it by `tests/s16/`'s programs, which
`cargo run --release --example s16 -- <folder>` writes for the referees.

Black Moon Clan codename (Prince Diamond's younger brother, Saphir) — see `Man pages/EmuSen_Core_Naming_Scheme.md` for the full core naming scheme.
