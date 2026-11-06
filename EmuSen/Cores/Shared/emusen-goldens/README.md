# emusen-goldens

What the cores' integration tests share for their golden traces, moved from WiseMan's C# fixtures when the C# core
host was retired (EmuSen_Backend.md §40.10):

- `trace`: the golden trace's text format, its reading and writing, the first difference between two traces, and the
  hashes and chain fold its rows hold (Moon_Native.md §9.2, VenusRT_Native.md §68.2).
- `dotnet_random`: .NET's seeded `System.Random`, so a program generated from a seed is the bytes the traces were
  recorded from.
- `drive`: a core library loaded through `emusen-core-host` and run as the runtime runs one for a player: its settings'
  defaults at create, the input latch sent before each frame, the frame advanced and presented, the sound drained, the
  picture copied packed with its rows repeated, the state saved whole; no firmware and no battery.

It is a dev-dependency only. No shipped core links it, and it reaches a core only through the core ABI v1.
