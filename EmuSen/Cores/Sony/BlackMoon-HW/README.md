# BlackMoon Hardware (Shared Sony Chip Crates)

Shared Rust hardware crates used across Sony console cores (`diamond` and `sapphire`). Each crate models one chip independently of any console bus or memory map:

- `blackmoon-r3000` — MIPS I R3000A CPU core (PlayStation CPU in `diamond`; PlayStation 2 IOP in `sapphire`).
- `blackmoon-gte` — PlayStation Geometry Transformation Engine on COP2 (`diamond`).
- `blackmoon-spu` — `no_std` SPU voice engine (SPU-ADPCM, Gaussian interpolation, ADSR/sweeps, noise, pitch modulation, reverb) shared by `diamond`'s SPU and both cores of `sapphire`'s SPU2.

## Documentation

- [BlackMoon Hardware Contracts](../../../../Docs/Man%20pages/Hardware/Sony/BlackMoon-HW/README.md)
- [R3000A Reference](../../../../Docs/Man%20pages/Hardware/Sony/BlackMoon-HW/BlackMoon_R3000.md)
- [GTE Reference](../../../../Docs/Man%20pages/Hardware/Sony/BlackMoon-HW/BlackMoon_GTE.md)
- [SPU Voice Engine Reference](../../../../Docs/Man%20pages/Hardware/Sony/BlackMoon-HW/BlackMoon_SPU.md)
- [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme) (Black Moon Clan faction)
