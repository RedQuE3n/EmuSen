# Third-party notices

EmuSen is licensed under the GNU General Public License, version 3 ([`LICENSE`](LICENSE)). Its builds also contain
software written by other people, under their own licences. This file lists that software, who holds its copyright
and which licence applies, as those licences require. The full licence texts, and the upstream notice files some
packages ship, are in [`licenses/`](licenses/), and every published build carries this file and that folder.

The lists below were taken from the packages themselves: the NuGet packages' metadata and licence files, and
`cargo metadata` with each crate's own licence file, for the versions pinned on 2026-09-26; the Rust crates were
taken again on 2026-10-15. Section 1 is what a build ships. Section 2 is what EmuSen downloads only when the player
asks for it. Section 3 is used only to build and test EmuSen and is not distributed.

## 1. Distributed with EmuSen's builds

### 1.1 The .NET runtime

Published builds are self-contained, so they include the .NET runtime.

| Component | Licence | Copyright |
|---|---|---|
| [.NET runtime](https://github.com/dotnet/runtime) 10.0 | MIT | .NET Foundation and Contributors |

Its own third-party notices are reproduced in
[`licenses/dotnet-runtime-THIRD-PARTY-NOTICES.txt`](licenses/dotnet-runtime-THIRD-PARTY-NOTICES.txt), and its licence
in [`licenses/dotnet-runtime-LICENSE.txt`](licenses/dotnet-runtime-LICENSE.txt).

### 1.2 NuGet packages

| Package | Version | Licence | Copyright |
|---|---|---|---|
| [Avalonia](https://github.com/AvaloniaUI/Avalonia) (Avalonia, .Desktop, .FreeDesktop, .FreeDesktop.AtSpi, .HarfBuzz, .Markup.Xaml.Loader, .Native, .Remote.Protocol, .Skia, .Themes.Fluent, .Win32, .X11, .Fonts.Inter) | 12.1.0 | MIT | The AvaloniaUI Project |
| [Inter](https://github.com/rsms/inter) typeface, inside Avalonia.Fonts.Inter | — | SIL Open Font License 1.1 | The Inter Project Authors |
| Avalonia.Angle.Windows.Natives (Windows builds) | 2.1.27548.20260419 | MIT; the [ANGLE](https://chromium.googlesource.com/angle/angle) library inside it is BSD-3-Clause | The AvaloniaUI Project; The ANGLE Project Authors |
| [MicroCom.Runtime](https://github.com/kekekeks/MicroCom) | 0.11.6 | MIT | Nikita Tsukanov |
| [Tmds.DBus.Protocol](https://github.com/tmds/Tmds.DBus) | 0.94.1 | MIT | Tom Deseyn |
| [SkiaSharp](https://github.com/mono/SkiaSharp) and its native assets | 3.119.4 | MIT; the native [Skia](https://skia.org/) library is BSD-3-Clause | Microsoft Corporation; Google LLC |
| [HarfBuzzSharp](https://github.com/mono/SkiaSharp) and its native assets | 8.3.1.3 | MIT; the native [HarfBuzz](https://github.com/harfbuzz/harfbuzz) library is under HarfBuzz's "Old MIT" licence | Microsoft Corporation; the HarfBuzz authors |
| [SDL3-CS](https://github.com/edwardgushchin/SDL3-CS) and SDL3-CS.Native | 3.4.2 | zlib | Eduard Gushchin |
| [SDL 3](https://github.com/libsdl-org/SDL), inside SDL3-CS.Native | 3.4.2 | zlib | Sam Lantinga |
| [Microsoft.Data.Sqlite](https://github.com/dotnet/efcore) and .Core | 10.0.10 | MIT | Microsoft Corporation |
| Microsoft.Extensions.DependencyInjection.Abstractions, .Logging.Abstractions (Hotaru builds) | 8.0.0 | MIT | Microsoft Corporation |
| Microsoft.Extensions.DependencyModel | 9.0.9 | MIT | Microsoft Corporation |
| Microsoft.DotNet.PlatformAbstractions | 3.1.6 | MIT | .NET Foundation and Contributors |
| [Microsoft.IO.RecyclableMemoryStream](https://github.com/Microsoft/Microsoft.IO.RecyclableMemoryStream) (Hotaru builds) | 3.0.1 | MIT | Microsoft Corporation |
| [SQLitePCLRaw](https://github.com/ericsink/SQLitePCL.raw) (bundle_e_sqlite3, config, core, provider) | 3.0.5 | Apache-2.0 | SourceGear, LLC |
| [SQLite](https://sqlite.org/) native library (package `SQLite`) | 3.53.4 | Public domain (SQLite); packaging Apache-2.0 | SQLite: dedicated to the public domain; packaging SourceGear, LLC |
| [Silk.NET](https://github.com/dotnet/Silk.NET) (Core, Vulkan, Shaderc) | 2.23.0 | MIT | .NET Foundation and Contributors |
| Silk.NET.Shaderc.Native: [shaderc](https://github.com/google/shaderc), [glslang](https://github.com/KhronosGroup/glslang), [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools) | 2.23.0 | shaderc and SPIRV-Tools: Apache-2.0; glslang: BSD-3-Clause, with parts under MIT and Apache-2.0 | Google LLC; The Khronos Group Inc.; 3Dlabs Inc. Ltd.; LunarG, Inc.; and their contributors |

SkiaSharp's and HarfBuzzSharp's native libraries bundle further libraries (among them libpng, zlib, FreeType,
libjpeg-turbo, libwebp and expat). Their notices, as the two packages ship them (the file is identical in both), are
reproduced in [`licenses/SkiaSharp-HarfBuzzSharp-THIRD-PARTY-NOTICES.txt`](licenses/SkiaSharp-HarfBuzzSharp-THIRD-PARTY-NOTICES.txt).
The shaderc package ships no notice file of its own; the licences above are those of the upstream projects.

### 1.3 EmuSen.LunaP

[EmuSen.LunaP](https://github.com/RedQuE3n/EmuSen.LunaP), the user-interface toolkit Mistress is built with, is a
sibling project by the same author, published under the MIT licence.

### 1.4 Rust libraries in the native cores and the platform library

A build ships seven libraries compiled from Rust: the cores MarsRT (`marsrt`, the N64), MercuryRT (`mercuryrt`, the
Game Boy), MoonRT (`moonrt`, the NES), VenusRT (`venusrt`, the SNES), Nephrite (`nephrite`, the Genesis) and Diamond
(`diamond`, the PlayStation), and the platform library (`emusen_platform`), which holds the parts of the platform above
the cores that have moved to Rust. Each links the [Rust standard library](https://github.com/rust-lang/rust) (MIT OR
Apache-2.0, The Rust Project Developers). The crates under `EmuSen/Cores/Shared/`, `Beryl-HW/`, `BlackMoon-HW/` and
`Platform/` are EmuSen's own and under its licence. MercuryRT, MoonRT and VenusRT use no other crate.

The others use the crates below, as `cargo tree` gave each library's normal and build dependencies for every target on
2026-10-15. MarsRT's are for its Cranelift recompiler and its Vulkan path. Nephrite's, Diamond's and the platform
library's shared set is `emusen-disc`'s, which reads CHD images through the `chd` crate. CHD is MAME's format; no code of MAME is in these libraries. chdman, MAME's tool for it, is shipped beside them as a program of its own (§1.7). The platform library also
uses the MD5 crates for the hash by which a ROM file is recognised after a rename, `libloading` (with `windows-link`
on Windows) to call the SDL3, the Shaderc and the SQLite the program ships, `sha2` (with `cpufeatures`, and on some
processors `libc`) for the keys of the compiled-shader cache, `ash`, without its own loader, for Vulkan's
declarations, and `memmap2` for a disc's files mapped read-only. Since 2026-10-18 it also carries the platform's one HTTP
and TLS client, `ureq` over `rustls` with `ring`, certificates checked against the system's own store through
`rustls-platform-verifier` (`rustls-native-certs` and `openssl-probe` on Linux, `security-framework` and
`core-foundation` on macOS, `windows-sys` on Windows), for the cheat database's download; `cc`, `shlex` and
`find-msvc-tools` only build `ring` and are not shipped. Those crates were taken from `cargo tree` on 2026-10-18.

| Crate | Version | Licence | Copyright or authors | Used by |
|---|---|---|---|---|
| [allocator-api2](https://github.com/zakarumych/allocator-api2) | 0.2.21 | MIT OR Apache-2.0 | Zakarum | MarsRT |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0.104 | MIT OR Apache-2.0 | David Tolnay | MarsRT |
| [arbitrary](https://github.com/rust-fuzz/arbitrary/) | 1.4.2 | MIT OR Apache-2.0 | Copyright (c) 2019 Manish Goregaokar | MarsRT |
| [arrayvec](https://github.com/bluss/arrayvec) | 0.7.8 | MIT OR Apache-2.0 | bluss | Nephrite, Diamond, platform |
| [ash](https://github.com/ash-rs/ash) | 0.38.0+1.3.281 | MIT OR Apache-2.0 | Copyright 2016 Maik Klein | MarsRT, Diamond, platform |
| [autocfg](https://github.com/cuviper/autocfg) | 1.5.1 | Apache-2.0 OR MIT | Josh Stone | Nephrite, Diamond, platform |
| [base64](https://github.com/marshallpierce/rust-base64) | 0.23.1 | MIT OR Apache-2.0 | Copyright (c) 2025 Alice Maz, Marshall Pierce | platform |
| [bitflags](https://github.com/bitflags/bitflags) | 1.3.2, 2.13.2 | MIT/Apache-2.0; 2.13.2: MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | 1.3.2: MarsRT; 2.13.2: platform (macOS) |
| [bitreader](https://github.com/irauta/bitreader) | 0.3.11 | MIT OR Apache-2.0 | Ilkka Rauta | Nephrite, Diamond, platform |
| [block-buffer](https://github.com/RustCrypto/utils) | 0.12.1 | MIT OR Apache-2.0 | Copyright (c) 2018-2025 The RustCrypto Project Developers | platform |
| [bumpalo](https://github.com/fitzgen/bumpalo) | 3.20.3 | MIT OR Apache-2.0 | Copyright (c) 2019 Nick Fitzgerald | MarsRT |
| [byteorder](https://github.com/BurntSushi/byteorder) | 1.5.0 | Unlicense OR MIT | Andrew Gallant | Nephrite, Diamond, platform |
| [bytes](https://github.com/tokio-rs/bytes) | 1.12.1 | MIT (`licenses/bytes-MIT.txt`) | Copyright (c) 2018 Carl Lerche | platform |
| [cc](https://github.com/rust-lang/cc-rs) | 1.8.0 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton | platform (builds `ring`; not shipped) |
| [cfg-if](https://github.com/rust-lang/cfg-if) | 1.0.5 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton | MarsRT, Nephrite, Diamond, platform |
| [chd](https://github.com/SnowflakePowered/chd-rs) | 0.3.4, 0.3.5 | BSD-3-Clause (`licenses/chd-BSD-3-Clause.txt`) | Ronny Chan | 0.3.4: Diamond, platform; 0.3.5: Nephrite |
| [claxon](https://github.com/ruuda/claxon) | 0.4.3 | Apache-2.0 | Ruud van Asseldonk | Nephrite, Diamond, platform |
| [const-oid](https://github.com/RustCrypto/formats) | 0.10.2 | Apache-2.0 OR MIT | Copyright (c) 2020-2026 The RustCrypto Project Developers | platform |
| [core-foundation](https://github.com/servo/core-foundation-rs) | 0.10.1 | MIT OR Apache-2.0 | Copyright (c) 2012-2013 Mozilla Foundation | platform (macOS) |
| [core-foundation-sys](https://github.com/servo/core-foundation-rs) | 0.8.7 | MIT OR Apache-2.0 | Copyright (c) 2012-2013 Mozilla Foundation | platform (macOS) |
| [cpufeatures](https://github.com/RustCrypto/utils) | 0.3.1 | MIT OR Apache-2.0 | Copyright (c) 2020-2026 The RustCrypto Project Developers | platform |
| cranelift-assembler-x64 | 0.136.0 | Apache-2.0 WITH LLVM-exception | the cranelift-assembler-x64 contributors | MarsRT |
| cranelift-assembler-x64-meta | 0.136.0 | Apache-2.0 WITH LLVM-exception | the cranelift-assembler-x64-meta contributors | MarsRT |
| [cranelift-bforest](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-bitset](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-codegen](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-codegen-meta](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-codegen-shared](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-control](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-entity](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-frontend](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-isle](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-jit](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-module](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-native](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers | MarsRT |
| [cranelift-srcgen](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Wasmtime Project Developers | MarsRT |
| [crc](https://github.com/mrhooray/crc-rs) | 3.4.0 | MIT OR Apache-2.0 | Rui Hu, Akhil Velagapudi | Nephrite, Diamond, platform |
| [crc-catalog](https://github.com/akhilles/crc-catalog) | 2.5.0 | MIT OR Apache-2.0 | Akhil Velagapudi | Nephrite, Diamond, platform |
| [crypto-common](https://github.com/RustCrypto/traits) | 0.2.2 | MIT OR Apache-2.0 | Copyright (c) 2021-2026 RustCrypto Developers | platform |
| [digest](https://github.com/RustCrypto/traits) | 0.11.3 | MIT OR Apache-2.0 | Copyright (c) 2017-2025 RustCrypto Developers; Copyright (c) 2017 Artyom Pavlov | platform |
| [equivalent](https://github.com/indexmap-rs/equivalent) | 1.0.2 | Apache-2.0 OR MIT | Copyright (c) 2016--2023 | MarsRT |
| [find-msvc-tools](https://github.com/rust-lang/cc-rs) | 0.1.14 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton | platform (builds `ring`; not shipped) |
| [flate2](https://github.com/rust-lang/flate2-rs) | 1.1.10 | MIT OR Apache-2.0 | Alex Crichton, Josh Triplett | Nephrite, Diamond, platform |
| [fnv](https://github.com/servo/rust-fnv) | 1.0.7 | Apache-2.0 / MIT | Copyright (c) 2017 Contributors | MarsRT |
| [foldhash](https://github.com/orlp/foldhash) | 0.2.0 | Zlib | Copyright (c) 2024 Orson Peters | MarsRT |
| [getrandom](https://github.com/rust-random/getrandom) | 0.2.17 | MIT OR Apache-2.0 | Copyright (c) 2018-2024 The rust-random Project Developers | platform |
| [gimli](https://github.com/gimli-rs/gimli) | 0.33.0 | MIT OR Apache-2.0 | Copyright (c) 2015 The Rust Project Developers | MarsRT |
| [hashbrown](https://github.com/rust-lang/hashbrown) | 0.16.1, 0.17.1 | MIT OR Apache-2.0 | Copyright (c) 2016 Amanieu d'Antras | MarsRT |
| [heck](https://github.com/withoutboats/heck) | 0.5.0 | MIT OR Apache-2.0 | Copyright (c) 2015 The Rust Project Developers | MarsRT |
| [http](https://github.com/hyperium/http) | 1.5.0 | MIT OR Apache-2.0 | Copyright (c) 2017 http-rs authors | platform |
| [httparse](https://github.com/seanmonstar/httparse) | 1.10.1 | MIT OR Apache-2.0 | Copyright (c) 2015-2025 Sean McArthur | platform |
| [hybrid-array](https://github.com/RustCrypto/hybrid-array) | 0.4.15 | MIT OR Apache-2.0 | Copyright (c) 2022-2026 The RustCrypto Project Developers | platform |
| [indexmap](https://github.com/indexmap-rs/indexmap) | 2.14.2 | Apache-2.0 OR MIT | Copyright (c) 2016--2017 | MarsRT |
| [itoa](https://github.com/dtolnay/itoa) | 1.0.18 | MIT OR Apache-2.0 | David Tolnay | platform |
| [libc](https://github.com/rust-lang/libc) | 0.2.189 | MIT OR Apache-2.0 | Copyright (c) The Rust Project Developers | MarsRT, platform |
| [libloading](https://github.com/nagisa/rust_libloading/) | 0.8.9, 0.9.0 | ISC | Copyright © 2015, Simonas Kazlauskas | 0.8.9: MarsRT, Diamond; 0.9.0: platform |
| [libm](https://github.com/rust-lang/compiler-builtins) | 0.2.16 | MIT | Copyright (c) 2018 Jorge Aparicio | MarsRT |
| [log](https://github.com/rust-lang/log) | 0.4.34 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | MarsRT, platform |
| [lzma-rs-perf-exp](https://github.com/SnowflakePowered/lzma-rs) | 0.2.1 | MIT | Guillaume Endignoux, Ronny Chan | Nephrite, Diamond, platform |
| [mach2](https://github.com/JohnTitor/mach2) | 0.4.3 | BSD-2-Clause OR MIT OR Apache-2.0 | Copyright (c) 2019 Nick Fitzgerald, 2021 Yuki Okushi | MarsRT |
| [md-5](https://github.com/RustCrypto/hashes) | 0.11.0 | MIT OR Apache-2.0 | Copyright (c) 2016-2026 The RustCrypto Project Developers; Copyright (c) 2016 Artyom Pavlov; Copyright (c) 2009-2013 Mozilla Foundation; Copyright (c) 2006-2009 Graydon Hoare | platform |
| [memmap2](https://github.com/RazrFalcon/memmap2-rs) | 0.9.11 | MIT OR Apache-2.0 | Copyright (c) 2020 Yevhenii Reizner | MarsRT, platform |
| [num-derive](https://github.com/rust-num/num-derive) | 0.4.2 | MIT OR Apache-2.0 | The Rust Project Developers | Nephrite, Diamond, platform |
| [num-traits](https://github.com/rust-num/num-traits) | 0.2.19 | MIT OR Apache-2.0 | The Rust Project Developers | Nephrite, Diamond, platform |
| [once_cell](https://github.com/matklad/once_cell) | 1.21.4 | MIT OR Apache-2.0 | Aleksey Kladov | platform |
| [openssl-probe](https://github.com/rustls/openssl-probe) | 0.2.1 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton | platform (Linux) |
| [percent-encoding](https://github.com/servo/rust-url/) | 2.3.2 | MIT OR Apache-2.0 | Copyright (c) 2013-2025 The rust-url developers | platform |
| [proc-macro2](https://github.com/dtolnay/proc-macro2) | 1.0.107 | MIT OR Apache-2.0 | David Tolnay, Alex Crichton | Nephrite, Diamond, platform |
| [quote](https://github.com/dtolnay/quote) | 1.0.47 | MIT OR Apache-2.0 | David Tolnay | Nephrite, Diamond, platform |
| [regalloc2](https://github.com/bytecodealliance/regalloc2) | 0.15.2 | Apache-2.0 WITH LLVM-exception | Chris Fallin, Mozilla SpiderMonkey Developers | MarsRT |
| [region](https://github.com/darfink/region-rs) | 3.0.2 | MIT | Copyright (c) 2016 Elliott Linder | MarsRT |
| [ring](https://github.com/briansmith/ring) | 0.17.14 | Apache-2.0 AND ISC, with parts from BoringSSL under Apache-2.0 or ISC and from once_cell under MIT OR Apache-2.0 (`licenses/ring-0.17.14-LICENSE.txt`) | Copyright 2015-2025 Brian Smith; The BoringSSL Authors | platform |
| [rustc-hash](https://github.com/rust-lang/rustc-hash) | 2.1.3 | Apache-2.0 OR MIT | The Rust Project Developers | MarsRT |
| [rustls](https://github.com/rustls/rustls) | 0.23.46 | Apache-2.0 OR ISC OR MIT | Copyright (c) 2016 Joseph Birr-Pixton | platform |
| [rustls-native-certs](https://github.com/rustls/rustls-native-certs) | 0.8.5 | Apache-2.0 OR ISC OR MIT | the rustls-native-certs contributors | platform (Linux) |
| [rustls-pki-types](https://github.com/rustls/pki-types) | 1.15.1 | MIT OR Apache-2.0 | Copyright (c) 2023 Dirkjan Ochtman | platform |
| [rustls-platform-verifier](https://github.com/rustls/rustls-platform-verifier) | 0.7.1 | MIT OR Apache-2.0 | Copyright (c) 2022 1Password | platform |
| [rustls-webpki](https://github.com/rustls/webpki) | 0.103.15 | ISC (`licenses/rustls-webpki-ISC.txt`) | Copyright 2015 Brian Smith | platform |
| [ruzstd](https://github.com/KillingSpark/zstd-rs) | 0.8.3, 0.9.1 | MIT | Moritz Borcherding | 0.8.3: Diamond, platform; 0.9.1: Nephrite |
| [security-framework](https://github.com/kornelski/rust-security-framework) | 3.7.0 | MIT OR Apache-2.0 | Copyright (c) 2015 Steven Fackler | platform (macOS) |
| [security-framework-sys](https://github.com/kornelski/rust-security-framework) | 2.17.0 | MIT OR Apache-2.0 | Copyright (c) 2015 Steven Fackler | platform (macOS) |
| [sha2](https://github.com/RustCrypto/hashes) | 0.11.0 | MIT OR Apache-2.0 | Copyright (c) 2016-2026 The RustCrypto Project Developers | platform |
| [shlex](https://github.com/comex/rust-shlex) | 2.0.1 | MIT OR Apache-2.0 | Copyright (c) 2015 Nicholas Allegra (comex) | platform (builds `ring`; not shipped) |
| [smallvec](https://github.com/servo/rust-smallvec) | 1.16.1 | MIT OR Apache-2.0 | Copyright (c) 2018 The Servo Project Developers | MarsRT |
| [subtle](https://github.com/dalek-cryptography/subtle) | 2.6.1 | BSD-3-Clause (`licenses/subtle-BSD-3-Clause.txt`) | Copyright (c) 2016-2017 Isis Agora Lovecruft, Henry de Valence; Copyright (c) 2016-2024 Isis Agora Lovecruft | platform |
| [syn](https://github.com/dtolnay/syn) | 2.0.119 | MIT OR Apache-2.0 | David Tolnay | Nephrite, Diamond, platform |
| [target-lexicon](https://github.com/bytecodealliance/target-lexicon) | 0.13.5 | Apache-2.0 WITH LLVM-exception | Dan Gohman | MarsRT |
| [text_io](https://github.com/oli-obk/rust-si) | 0.1.13 | MIT OR Apache-2.0 | Oliver Schneider | Nephrite, Diamond, platform |
| [twox-hash](https://github.com/shepmaster/twox-hash) | 2.1.5 | MIT | Jake Goulding | Nephrite, Diamond, platform |
| [typenum](https://github.com/paholg/typenum) | 1.20.1 | MIT OR Apache-2.0 | Copyright (c) 2014 Paho Lurie-Gregg | platform |
| [unicode-ident](https://github.com/dtolnay/unicode-ident) | 1.0.26, 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | Copyright © 1991-2023 Unicode, Inc. | 1.0.26: platform; 1.0.27: Nephrite, Diamond |
| [untrusted](https://github.com/briansmith/untrusted) | 0.9.0 | ISC (`licenses/untrusted-ISC.txt`) | Copyright 2015-2016 Brian Smith | platform |
| [ureq](https://github.com/algesten/ureq) | 3.4.2 | MIT OR Apache-2.0 | Copyright (c) 2019 Martin Algesten | platform |
| [ureq-proto](https://github.com/algesten/ureq-proto) | 0.6.5 | MIT OR Apache-2.0 | Copyright 2022 Martin Algesten | platform |
| [utf8-zero](https://github.com/algesten/utf8-zero) | 0.8.1 | MIT OR Apache-2.0 | Simon Sapin, Martin Algesten | platform |
| wasmtime-internal-core | 49.0.0 | Apache-2.0 WITH LLVM-exception | The Wasmtime Project Developers | MarsRT |
| [wasmtime-internal-jit-icache-coherence](https://github.com/bytecodealliance/wasmtime) | 49.0.0 | Apache-2.0 WITH LLVM-exception | The Wasmtime Project Developers | MarsRT |
| [windows-link](https://github.com/microsoft/windows-rs) | 0.2.1 | MIT OR Apache-2.0 | the windows-link contributors | MarsRT, Diamond, platform |
| [windows-sys](https://github.com/microsoft/windows-rs) | 0.52.0, 0.61.2 | MIT OR Apache-2.0 | Microsoft | MarsRT; 0.61.2: platform (Windows) |
| [windows-targets](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_aarch64_gnullvm](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_aarch64_msvc](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_i686_gnu](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_i686_gnullvm](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_i686_msvc](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_x86_64_gnu](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_x86_64_gnullvm](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [windows_x86_64_msvc](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft | MarsRT |
| [zeroize](https://github.com/RustCrypto/utils) | 1.9.1 | Apache-2.0 OR MIT | Copyright (c) 2018-2026 The RustCrypto Project Developers | platform |
| [zlib-rs](https://github.com/trifectatechfoundation/zlib-rs) | 0.6.8 | Zlib (`licenses/zlib-rs-Zlib.txt`) | Trifecta Tech Foundation | Nephrite, Diamond, platform |

Where a crate offers a choice of licences ("MIT OR Apache-2.0"), EmuSen uses it under the MIT licence.

### 1.5 Fonts

| Font | Version | Licence | Copyright |
|---|---|---|---|
| [Barlow Condensed](https://github.com/jpt/barlow), Regular (`EmuSen.Mistress/Assets/Fonts/BarlowCondensed-Regular.ttf`), the big-screen menus' typeface | 1.408 | SIL Open Font License 1.1 | Copyright 2017 The Barlow Project Authors (https://github.com/jpt/barlow) |

The file is unmodified, as Google Fonts distributes it; its copyright notice and the licence are in
[`licenses/BarlowCondensed-OFL-1.1.txt`](licenses/BarlowCondensed-OFL-1.1.txt). The font is shipped as a separate file
beside the program and is not sold by itself, as the licence requires.

### 1.6 Code taken from another emulator

| Project | Licence | Copyright | In |
|---|---|---|---|
| [Mesen 2](https://github.com/SourMesen/Mesen2) | GNU General Public License, version 3 (the same licence as EmuSen's, [`LICENSE`](LICENSE)) | Copyright (C) 2014-2025 Sour | Venus (C#), `EmuSen/Cores/Nintendo/Venus - SNES/`, removed from the source on 2026-10-16 and kept in the history and the releases before it |

Venus, EmuSen's first SNES core, took parts of Mesen 2 during its early development, and which of its rules came from
there can no longer be told apart (`VenusRT_Plan.md` §1.1). The whole of Venus (C#) is therefore credited here. It
shipped in every build up to its removal as the SNES's reference engine and the fallback where VenusRT's library was
missing. VenusRT, the SNES's engine, is a separate rewrite that takes nothing from Mesen or from Venus (C#).
Venus (C#) was removed from the source on 2026-10-16, and this credit stands for the repository's history and for the
releases that carry it, 0.9.1 among them (`VenusRT_Native.md` §68).

### 1.7 chdman, from MAME

Since 2026-10-25 a build carries one program that is not EmuSen's: chdman, the Compressed Hunks of Data manager of
[MAME](https://www.mamedev.org/), beside the platform library as `chdman` (`chdman.exe` on Windows). EmuSen's CHD tool
runs it to write CHD files and to verify them (`EmuSen_GamePackages.md` §4, §12). It is MAME's own code, built
unmodified from MAME's source archive by `Platform/library-tool/build-chdman.sh`, following MAME's documentation
"Compiling MAME"; the release and the SHA-256 of each platform's binary are recorded in
`Platform/library-tool/chdman.pin`, and the CHD tool refuses a chdman whose SHA-256 is not the pinned one. Its manual
is MAME's own page, ["chdman – CHD (Compressed Hunks of Data) File Manager"](https://docs.mamedev.org/tools/chdman.html),
MAME Documentation 0.289.

| Component | Version | Licence | Copyright |
|---|---|---|---|
| chdman (its source file) | MAME 0.289, tag `mame0289` | BSD-3-Clause ([`licenses/chdman-BSD-3-Clause.txt`](licenses/chdman-BSD-3-Clause.txt)) | Aaron Giles |
| MAME as a whole, from which chdman is built | 0.289 | GNU General Public License, version 2 or later ([`licenses/GPL-2.0.txt`](licenses/GPL-2.0.txt)) | 1997-2026 MAMEDev and contributors ("The MAME License", MAME Documentation 0.289) |

**The written offer of chdman's source**, as GPL-2.0's section 3 asks of a binary distributed without it, is
[`licenses/chdman-SOURCE-OFFER.txt`](licenses/chdman-SOURCE-OFFER.txt). It names the release tag, `mame0289` in MAME's
public repository, and the source archive the binary was built from.

**Libraries inside chdman.** MAME builds chdman with libraries of its own source tree linked in. Those below were found
by the names and notices the binary itself carries (measured on 2026-10-25 with `strings`; MAME's source is not read),
and their licences are the upstream projects' own texts, read on 2026-10-25:

| Library | Licence | Copyright |
|---|---|---|
| [zlib](https://zlib.net/) 1.3.2 | zlib ([`licenses/zlib-Zlib.txt`](licenses/zlib-Zlib.txt)) | 1995-2026 Jean-loup Gailly and Mark Adler |
| [LZMA SDK](https://www.7-zip.org/sdk.html) (the LZMA coder) | Public domain | Igor Pavlov |
| [libFLAC](https://xiph.org/flac/) | BSD-3-Clause ([`licenses/libFLAC-BSD-3-Clause.txt`](licenses/libFLAC-BSD-3-Clause.txt)) | 2000-2009 Josh Coalson; 2011-2025 Xiph.Org Foundation |
| [Zstandard](https://github.com/facebook/zstd) | BSD-3-Clause ([`licenses/zstd-BSD-3-Clause.txt`](licenses/zstd-BSD-3-Clause.txt)), offered by its authors also under GPL-2.0 | Meta Platforms, Inc. and affiliates |
| [utf8proc](https://github.com/JuliaStrings/utf8proc) | MIT ("expat"), with the original utf8proc licence and the Unicode data licence it includes ([`licenses/utf8proc-MIT.txt`](licenses/utf8proc-MIT.txt)) | 2014-2021 Steven G. Johnson, Jiahao Chen, Tony Kelman, Jonas Fonseca and contributors; 2009, 2013 Public Software Group e. V.; Unicode, Inc. |
| GCC's C++ and support libraries (`libstdc++`, `libgcc`), linked in so that one binary runs on the desktop and the handheld | GPL-3.0 with the GCC Runtime Library Exception, which asks nothing of a program built with them | Free Software Foundation, Inc. |

chdman loads `libSDL2-2.0.so.0` and the C library from the system it runs on; neither is shipped.

### 1.8 Approaches adopted from other emulators' source

Decided 2026-10-29: Sapphire, the PlayStation 2 core, is no longer clean-room. The source of the three emulators below
may be read and their approaches adopted. Nothing is copied: each approach is written anew in Rust on Sapphire's own
units, and the page that describes it names the source it came from. Where Sapphire's code follows a reference's
closely, the file carries that reference's copyright line too.

| Project | Licence | Copyright | Approaches adopted, and where they are described |
|---|---|---|---|
| [PCSX2](https://github.com/PCSX2/pcsx2) | GNU General Public License, version 3 or later | 2002-2026 PCSX2 Dev Team | the units brought along at the EE's event test, the IOP in arrears (`Sapphire_Speed.md` §62); its VU1 microprogram run on to its end where VIF1 waits for it was tried and not kept (§62.3); the EE's blocks found through a table by address before any search, after its recompiler's lookup table (§64.3); the multimedia and COP1 instructions run inside the EE's blocks as the interpreter's operation, the registers written back first, after its recompiler's interpreter calls (§65) |
| [DobieStation](https://github.com/PSI-Rockin/DobieStation) | GNU General Public License, version 3 | PSI-Rockin and the DobieStation contributors | the EE run in bounded rounds of cycles, every unit then run for the same time (`Sapphire_Speed.md` §62) |
| [Play!](https://github.com/jpd002/Play-) | BSD-2-Clause | 2006-2026 Jean-Philip Desjardins | read for comparison: the EE run for a fixed quantum, then the VUs and the IOP (`Sapphire_Speed.md` §62); the EE's block lookup and linking, and what its recompiler leaves to the interpreter (§64.2) |

## 2. Downloaded only at the player's request, never shipped

Mistress can fetch the following when the player asks it to. None of it is in this repository or in a build, and each
keeps its own licence and terms, which apply to the downloaded copy.

| What | From | Licence or terms |
|---|---|---|
| RetroArch's slang shader presets | [libretro/slang-shaders](https://github.com/libretro/slang-shaders), via buildbot.libretro.com | Per shader, as stated in each file (GPL, MIT, public domain and others) |
| Cheat codes | [libretro-database](https://github.com/libretro/libretro-database), via buildbot.libretro.com | libretro-database's terms |
| OpenVGDB, the game database OpenEmu uses | [OpenVGDB](https://github.com/OpenVGDB/OpenVGDB) | No licence is stated by the project |
| Box art (the OpenEmu fallback) | [libretro-thumbnails](https://thumbnails.libretro.com/) and the addresses OpenVGDB gives | Scans owned by their respective rights holders |
| Game information and media | [ScreenScraper](https://www.screenscraper.fr/) | ScreenScraper's terms of use; the media belong to their rights holders |
| The Art Book Next theme (ES-DE edition) | [anthonycaccese/art-book-next-es-de](https://github.com/anthonycaccese/art-book-next-es-de) | CC BY-NC-SA 2.0, by Anthony Caccese; credited in Mistress's theme About sheet |

## 3. Used to build and test EmuSen, not distributed

| Package | Licence | Notes |
|---|---|---|
| [xunit](https://github.com/xunit/xunit), xunit.runner.visualstudio | Apache-2.0 | Test framework |
| [CsCheck](https://github.com/AnthonyLloyd/CsCheck) | Apache-2.0 | Property tests |
| [rcgen](https://github.com/rustls/rcgen) 0.14.10 and the crates it brings | MIT OR Apache-2.0 | Certificates made at test time for `emusen-net`'s own test server |
| Microsoft.NET.Test.Sdk, [coverlet](https://github.com/coverlet-coverage/coverlet) | MIT | Test running and coverage |
| Avalonia.Headless | MIT | Headless UI tests |
| Avalonia.BuildServices 11.3.2 | MIT | Build-time only |
| [Svg.Skia](https://github.com/wieslawsoltes/Svg.Skia) 5.1.1, ExCSS | MIT | A second SVG renderer to check LunaP's against |
| Svg.Custom 5.1.1 (a dependency of Svg.Skia) | MS-PL | Test-only; never linked into anything shipped, because MS-PL is not GPL-compatible |
| AvaloniaUI.DiagnosticsSupport 2.2.2 | AvaloniaUI OÜ's terms | Debug builds only; excluded from Release and published builds |

