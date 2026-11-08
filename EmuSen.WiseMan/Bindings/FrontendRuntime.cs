using System;
using EmuSen.Crystal.Client;

namespace EmuSen.WiseMan.Bindings
{
    // The runtimes the facades read the frontend API through since 2.0: one with no cores, whose systems are the table's alone, and one with the build's - see EmuSen_RustPlatform.md §47.24.
    internal static class FrontendRuntime
    {
        private static readonly Lazy<CrystalRuntime> TableOnly = new(() => new CrystalRuntime(new CrystalRuntimeOptions { Cores = Array.Empty<string>(), PlayerCores = Array.Empty<string>() }));

        private static readonly Lazy<CrystalRuntime> Built = new(() => new CrystalRuntime());

        public static CrystalRuntime Table => TableOnly.Value;

        public static CrystalRuntime Cores => Built.Value;
    }
}
