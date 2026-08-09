# Moon (NES)

**In progress — the machine runs, renders and makes sound; it has had far less play-testing than Venus.**

| Component | State |
|---|---|
| CPU (2A03 / 6502) | Implemented. Validated against SingleStepTests `nes6502/v1` — 256/256 opcode files, 2,560,000 cases, final state *and* per-cycle bus traces |
| Cartridge / mappers | iNES + partial NES 2.0; ten boards — NROM, MMC1, UxROM, CNROM, AxROM, **MMC3**, GxROM, Colour Dreams, Camerica, NINA-003 |
| PPU (2C02) | Registers, loopy `v`/`t`/`x`/`w`, background and sprite rendering, sprite 0, NMI. Scanline granularity, no emphasis bits |
| APU | All five channels synthesize — two pulse (with pulse 1's ones-complement sweep negate), triangle, noise, DMC — mixed into a drained sample buffer |
| Input | Two standard controllers |
| `ICore` | Implemented — `MoonCore`, on its own master-clock timeline, with save states |
| `IDebugTarget` | Implemented — `MoonDebugTarget`, the second implementation this interface has ever had |
| PAL | Not implemented; NTSC is assumed |

Hardware notes live in `Man pages/Hardware/Nintendo/Moon - NES/`: `Moon_CPU.md`, `Moon_Core.md`, `Moon_Memory.md`, `Moon_PPU.md`, `Moon_APU.md`, `Moon_Debug.md`, `Moon_TestRoms.md`. Those pages are the explanation for everything here; the code comments only point at them.

## Verifying it

The CPU's ground-truth run needs third-party data that is deliberately not committed — fetch `nes6502/v1` from https://github.com/SingleStepTests/ProcessorTests:

```sh
dotnet run -c Release --project EmuSen.Pharaoh -- --singlestep nes6502 /path/to/nes6502/v1
```

The PPU, APU and mappers are checked with third-party test ROMs, also not committed — fetch them from https://github.com/christopherpow/nes-test-roms:

```sh
dotnet run -c Release --project EmuSen.Pharaoh -- --testroms /path/to/nes-test-roms
```

One `[PASS]`/`[FAIL]`/`[----]`/`[SKIP]` line per ROM, read out of the blargg `$6000` protocol block. See `Moon_TestRoms.md` for the protocol, the two false-pass traps in it, and what the runner does not cover.

Everything else is self-contained in `EmuSen.WiseMan`:

```sh
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~Moon"
```

`EmuSen.WiseMan/Fixtures/SyntheticNesRom.cs` builds its own iNES images; no commercial ROM is ever needed or committed.

## Wired up

`CoreFactory` builds a `MoonCore` and a `MoonDebugTarget` for any `.nes`, and `CoreCatalog` lists the console, so every frontend loads NES ROMs through the same path it loads SNES ones and the DianaOS shell talks to this core through the same `IDebugTarget`. See `Moon_Debug.md` §6.

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the naming scheme and `Man pages/EmuSen_Core_Gameplan.md` §7 for how this core is sequenced against Venus.
