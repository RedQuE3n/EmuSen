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
`cargo run --release --example s16 -- <folder>` writes for the referees. `src/vu/` is the vector units
(S12, S13): `tests/s13/` and `tests/micro.rs` grade micro mode by the stage's programs, `tests/vu_micro.rs` by the VU
manual's pipeline figures, and `cargo run --release --example s13 -- <folder>` writes the programs for the referees.
Since stage S17 (§27) the GS draws: the vertex queue and drawing kick (`gs::draw`), every primitive's coverage and
interpolation (`gs::raster`) and the pixel pipeline (`gs::pixel`); `tests/draw.rs` grades it by `tests/s17/`'s programs
and their model, which `cargo run --release --example s17 -- <folder>` writes for the referees with a background record
each.
Since stage S18 (§31) it textures and fogs: `gs::texture` (the CLUT buffer, the context's texture registers, the
filters and the texture function) and `gs::exact` (texture coordinates and the LOD without rounding); `tests/texture.rs`
grades them by `tests/s18/`'s programs and S17's model extended by `tests/s18/manual.rs`, which
`cargo run --release --example s18 -- <folder>` writes for the referees.
`src/gif/` is the GIF (stage S15, `Sapphire_Native.md` §33): its three paths, their arbitration and PATH3's masks, and
the GIFtag's formats (`gif::packing`, their one owner); `tests/gif.rs` grades it by `tests/s15/`'s programs and runs
S16's, S17's and S18's programs whole through PATH3 and VIF1, and `cargo run --release --example s15 -- <folder>` writes the
programs for the referees.
`src/spu2/` is the SPU2 (stage S26, `Sapphire_Native.md` §32): two cores of `blackmoon-spu`'s voices on the IOP's
bus; `tests/spu2.rs` grades it by `tests/s26/gate.rs`'s programs, and `cargo run --release --example s26 -- <folder>`
writes them and the stage's measurement programs for the referees, `s26 grade <name> <run>` and `s26 diff <name> <run>`
grading and comparing a run's record.
`src/ipu/` is the IPU (stage S27, `Sapphire_Native.md` §34): its commands, registers and FIFOs on the DMAC's channels 3
and 4, MPEG-2 and MPEG-1 macroblocks as ITU-T H.262 defines them, Annex A's reference IDCT, and §8.6's colour
conversion; `tests/ipu.rs` grades it by `tests/s27/`'s streams (`streams/`, made by ffmpeg, and `encode.rs`'s) against a
model of the standard (`standard.rs`) and the manual's post-processing (`manual.rs`), driving it as an FMV player would
(`driver.rs`), and `cargo run --release --example s27 -- <folder>` writes the replay and register programs for the
referees, with `explain`, `pixels`, `csc` and `timing` to read their records.

Since stage S24 (`Sapphire_Native.md` §35) `src/machine.rs` is the console on the board with the player's BIOS, and
`cargo run --release --example s24 -- <bios> <americas|europe|japan> <frames> <folder> [frame[:bits]...]` boots it on a
BIOS given by path, pressing buttons at chosen frames and writing the pictures, printing only the hardware's own reports;
`examples/s24probe.rs` writes the stage's measurement programs for the referees and `programs/s24/` holds its bare ones.
Since stage S10 (`Sapphire_Native.md` §36) an EE program given as the image is entered by fast boot once the BIOS is up
(`Machine::enter_program`), and the IOP's console is kept in the `TTY` space (`src/iop/tty.rs`);
`cargo run --release --example s10 -- <bios> <americas|europe|japan> <program.elf> <expected>` runs a ps2autotests
program on a BIOS given by path and grades its text against the expected file.

Since stage S30 (`Sapphire_Native.md` §38) `src/state.rs` is the whole console's state, SPH2 version 2, each unit's
block written beside the unit and composed in `src/board.rs`; `tests/sph2/version-2.layout` is its listing, which
`cargo run --example s30 -- layout <file>` writes, and `cargo run --release --example s30 -- bios <bios>
<americas|europe|japan> <span> <frame>...` saves and loads states on the player's BIOS, printing only whether pictures,
sound and states are equal. `tests/state.rs` cuts the project's programs mid-run.

Black Moon Clan codename (Prince Diamond's younger brother, Saphir) — see `Man pages/EmuSen_Core_Naming_Scheme.md` for the full core naming scheme.
