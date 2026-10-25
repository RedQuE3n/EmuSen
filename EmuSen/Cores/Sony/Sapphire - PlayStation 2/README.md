# Sapphire (Sony PlayStation 2)

The PlayStation 2 core, in Rust on the core ABI v1. Planned 2026-10-23
(`EmuSen.DianaSh/DianaSh/Etc/Man pages/Hardware/Sony/Sapphire - PlayStation 2/Sapphire_Plan.md`); stage S1, a stub
marked in development that recognises an EE program or a PlayStation 2 disc and emulates nothing, built 2026-10-24
and recorded in `Sapphire_Native.md` beside the plan. `cargo test --release` runs its tests.

`programs/` holds the project's bare EE programs for the referees (`Sapphire_Referees.md` §4, from stage S3):
`cargo run --release --example elf -- programs/record.s record.elf` assembles one with `emusen-mips` into an ELF, and
`EmuSen.WiseMan/Reference/analysis/ps2referee.py` runs it on a referee and reads its record.

Black Moon Clan codename (Prince Diamond's younger brother, Saphir) — see `Man pages/EmuSen_Core_Naming_Scheme.md` for the full core naming scheme.
