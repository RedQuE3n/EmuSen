# Third-party notices

EmuSen is licensed under the GNU General Public License, version 3 ([`LICENSE`](LICENSE)). Its builds also contain
software written by other people, under their own licences. This file lists that software, who holds its copyright
and which licence applies, as those licences require. The full licence texts, and the upstream notice files some
packages ship, are in [`licenses/`](licenses/), and every published build carries this file and that folder.

The lists below were taken from the packages themselves: the NuGet packages' metadata and licence files, and
`cargo metadata` with each crate's own licence file, for the versions pinned on 2026-09-26. Section 1 is what a
build ships. Section 2 is what EmuSen downloads only when the user asks for it. Section 3 is used only to build and
test EmuSen and is not distributed.

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

### 1.4 Rust libraries in the native cores

MarsRT (`libmarsrt`) and MercuryRT (`libmercuryrt`) are compiled from Rust. Both link the
[Rust standard library](https://github.com/rust-lang/rust) (MIT OR Apache-2.0, The Rust Project Developers). MercuryRT
uses no other crate. MarsRT uses these, for its Cranelift recompiler and its Vulkan path:

| Crate | Version | Licence | Copyright or authors |
|---|---|---|---|
| [allocator-api2](https://github.com/zakarumych/allocator-api2) | 0.2.21 | MIT OR Apache-2.0 | Zakarum |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0.104 | MIT OR Apache-2.0 | David Tolnay |
| [arbitrary](https://github.com/rust-fuzz/arbitrary/) | 1.4.2 | MIT OR Apache-2.0 | Copyright (c) 2019 Manish Goregaokar |
| [ash](https://github.com/ash-rs/ash) | 0.38.0+1.3.281 | MIT OR Apache-2.0 | Copyright 2016 Maik Klein |
| [bitflags](https://github.com/bitflags/bitflags) | 1.3.2 | MIT/Apache-2.0 | Copyright (c) 2014 The Rust Project Developers |
| [bumpalo](https://github.com/fitzgen/bumpalo) | 3.20.3 | MIT OR Apache-2.0 | Copyright (c) 2019 Nick Fitzgerald |
| [cfg-if](https://github.com/rust-lang/cfg-if) | 1.0.5 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton |
| cranelift-assembler-x64 | 0.136.0 | Apache-2.0 WITH LLVM-exception | the cranelift-assembler-x64 contributors |
| [cranelift-bforest](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-bitset](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-codegen](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-codegen-shared](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-control](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-entity](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-frontend](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-jit](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-module](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [cranelift-native](https://github.com/bytecodealliance/wasmtime) | 0.136.0 | Apache-2.0 WITH LLVM-exception | The Cranelift Project Developers |
| [equivalent](https://github.com/indexmap-rs/equivalent) | 1.0.2 | Apache-2.0 OR MIT | Copyright (c) 2016--2023 |
| [fnv](https://github.com/servo/rust-fnv) | 1.0.7 | Apache-2.0 / MIT | Copyright (c) 2017 Contributors |
| [foldhash](https://github.com/orlp/foldhash) | 0.2.0 | Zlib | Copyright (c) 2024 Orson Peters |
| [gimli](https://github.com/gimli-rs/gimli) | 0.33.0 | MIT OR Apache-2.0 | Copyright (c) 2015 The Rust Project Developers |
| [hashbrown](https://github.com/rust-lang/hashbrown) | 0.17.1, 0.16.1 | MIT OR Apache-2.0 | Copyright (c) 2016 Amanieu d'Antras |
| [indexmap](https://github.com/indexmap-rs/indexmap) | 2.14.2 | Apache-2.0 OR MIT | Copyright (c) 2016--2017 |
| [libc](https://github.com/rust-lang/libc) | 0.2.189 | MIT OR Apache-2.0 | Copyright (c) The Rust Project Developers |
| [libloading](https://github.com/nagisa/rust_libloading/) | 0.8.9 | ISC | Copyright © 2015, Simonas Kazlauskas |
| [libm](https://github.com/rust-lang/compiler-builtins) | 0.2.16 | MIT | Copyright (c) 2018 Jorge Aparicio |
| [log](https://github.com/rust-lang/log) | 0.4.34 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers |
| [mach2](https://github.com/JohnTitor/mach2) | 0.4.3 | BSD-2-Clause OR MIT OR Apache-2.0 | Copyright (c) 2019 Nick Fitzgerald, 2021 Yuki Okushi |
| [memmap2](https://github.com/RazrFalcon/memmap2-rs) | 0.9.11 | MIT OR Apache-2.0 | Copyright (c) 2020 Yevhenii Reizner |
| [proc-macro2](https://github.com/dtolnay/proc-macro2) | 1.0.107 | MIT OR Apache-2.0 | David Tolnay, Alex Crichton |
| [quote](https://github.com/dtolnay/quote) | 1.0.47 | MIT OR Apache-2.0 | David Tolnay |
| [regalloc2](https://github.com/bytecodealliance/regalloc2) | 0.15.2 | Apache-2.0 WITH LLVM-exception | Chris Fallin, Mozilla SpiderMonkey Developers |
| [region](https://github.com/darfink/region-rs) | 3.0.2 | MIT | Copyright (c) 2016 Elliott Linder |
| [rustc-hash](https://github.com/rust-lang/rustc-hash) | 2.1.3 | Apache-2.0 OR MIT | The Rust Project Developers |
| [serde](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 | Erick Tryzelaar, David Tolnay |
| [serde_core](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 | Erick Tryzelaar, David Tolnay |
| [serde_derive](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 | Erick Tryzelaar, David Tolnay |
| [smallvec](https://github.com/servo/rust-smallvec) | 1.16.1 | MIT OR Apache-2.0 | Copyright (c) 2018 The Servo Project Developers |
| [stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) | 1.2.1 | MIT OR Apache-2.0 | Copyright (c) 2017 Robert Grosse |
| [syn](https://github.com/dtolnay/syn) | 3.0.6 | MIT OR Apache-2.0 | David Tolnay |
| [target-lexicon](https://github.com/bytecodealliance/target-lexicon) | 0.13.5 | Apache-2.0 WITH LLVM-exception | Dan Gohman |
| [unicode-ident](https://github.com/dtolnay/unicode-ident) | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | Copyright © 1991-2023 Unicode, Inc. |
| wasmtime-internal-core | 49.0.0 | Apache-2.0 WITH LLVM-exception | The Wasmtime Project Developers |
| [wasmtime-internal-jit-icache-coherence](https://github.com/bytecodealliance/wasmtime) | 49.0.0 | Apache-2.0 WITH LLVM-exception | The Wasmtime Project Developers |
| [windows-link](https://github.com/microsoft/windows-rs) | 0.2.1 | MIT OR Apache-2.0 | the windows-link contributors |
| [windows-sys](https://github.com/microsoft/windows-rs) | 0.52.0 | MIT OR Apache-2.0 | Microsoft |
| [windows-sys](https://github.com/microsoft/windows-rs) | 0.61.2 | MIT OR Apache-2.0 | the windows-sys contributors |
| [windows-targets](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_aarch64_gnullvm](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_aarch64_msvc](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_i686_gnu](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_i686_gnullvm](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_i686_msvc](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_x86_64_gnu](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_x86_64_gnullvm](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |
| [windows_x86_64_msvc](https://github.com/microsoft/windows-rs) | 0.52.6 | MIT OR Apache-2.0 | Microsoft |

Where a crate offers a choice of licences ("MIT OR Apache-2.0"), EmuSen uses it under the MIT licence.

## 2. Downloaded only at the user's request, never shipped

Mistress can fetch the following when the user asks it to. None of it is in this repository or in a build, and each
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
| Microsoft.NET.Test.Sdk, [coverlet](https://github.com/coverlet-coverage/coverlet) | MIT | Test running and coverage |
| Avalonia.Headless | MIT | Headless UI tests |
| Avalonia.BuildServices 11.3.2 | MIT | Build-time only |
| [Svg.Skia](https://github.com/wieslawsoltes/Svg.Skia) 5.1.1, ExCSS | MIT | A second SVG renderer to check LunaP's against |
| Svg.Custom 5.1.1 (a dependency of Svg.Skia) | MS-PL | Test-only; never linked into anything shipped, because MS-PL is not GPL-compatible |
| AvaloniaUI.DiagnosticsSupport 2.2.2 | AvaloniaUI OÜ's terms | Debug builds only; excluded from Release and published builds |

