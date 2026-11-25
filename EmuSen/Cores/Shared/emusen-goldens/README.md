# `emusen-goldens`

Shared golden-trace test harness for core integration suites (`dev-dependency` only; never linked into shipped cores, interacts with cores strictly through Core ABI v1).

- `trace`: Golden trace text format parser/writer, first-difference reporting, per-frame hashes, and chain-fold digests.
- `md5`: Deterministic MD5 digest helper for frame/audio/state verification.
- `dotnet_random`: Seeded PRNG matching `.NET`'s `System.Random` for reproducible synthetic test program generation.
- `drive`: Loads a core library through `emusen-core-host` and drives it with default settings, per-frame input latches, packed frame copies, audio drains, and full state saves (no firmware or battery files).
