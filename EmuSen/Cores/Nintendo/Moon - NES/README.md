# Moon (NES)

**In progress — the machine runs, and makes no sound.**

| Component | State |
|---|---|
| CPU (2A03 / 6502) | Implemented. Validated against SingleStepTests `nes6502/v1` — 256/256 opcode files, 2,560,000 cases, final state *and* per-cycle bus traces |
| Cartridge / mappers | iNES + partial NES 2.0; boards 0 (NROM), 1 (MMC1), 2 (UxROM), 3 (CNROM), 7 (AxROM). **Mapper 4 (MMC3) is the biggest gap** |
| PPU (2C02) | Registers, loopy `v`/`t`/`x`/`w`, background and sprite rendering, sprite 0, NMI. Scanline granularity, no emphasis bits |
| APU | Registers, length counters and the frame IRQ only. **No sound is synthesized** |
| Input | Two standard controllers |
| `ICore` | Implemented — `MoonCore`, on the Crystal scheduler, with save states |
| `IDebugTarget` | Implemented — `MoonDebugTarget`, the second implementation this interface has ever had |
| PAL | Not implemented; NTSC is assumed |

Hardware notes live in `Man pages/Hardware/Nintendo/Moon - NES/`: `Moon_CPU.md`, `Moon_Core.md`, `Moon_Memory.md`, `Moon_PPU.md`, `Moon_APU.md`, `Moon_Debug.md`. Those pages are the explanation for everything here; the code comments only point at them.

## Verifying it

The CPU's ground-truth run needs third-party data that is deliberately not committed — fetch `nes6502/v1` from https://github.com/SingleStepTests/ProcessorTests:

```sh
dotnet run -c Release --project EmuSen.Tomoe -- nes6502 /path/to/nes6502/v1
```

Everything else is self-contained in `EmuSen.WiseMan`:

```sh
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~Moon"
```

`EmuSen.WiseMan/Fixtures/SyntheticNesRom.cs` builds its own iNES images; no commercial ROM is ever needed or committed.

## Not yet wired

No frontend constructs a `MoonCore` yet — `EmuSen.Hotaru`, `EmuSen.Mistress` and `EmuSen.Pharaoh` still build a `VenusCore` and a `SnesDebugTarget` directly. The debug target is exercised against the interface in tests, but no DianaOS command has been run against it. See `Moon_Debug.md` §6.

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the naming scheme and `Man pages/EmuSen_Core_Gameplan.md` §7 for how this core is sequenced against Venus.
