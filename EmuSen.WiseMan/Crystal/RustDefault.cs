using System;
using System.Runtime.CompilerServices;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // The suite's own default, the Rust, applied before any test since the test host's entry assembly carries no key - see EmuSen_Backend.md §35.
    internal static class RustDefault
    {
        [ModuleInitializer]
        internal static void Apply()
        {
            if (string.IsNullOrEmpty(Environment.GetEnvironmentVariable(CrystalContract.Variable)) && CrystalContract.DefaultsToRust(typeof(RustDefault).Assembly))
                Environment.SetEnvironmentVariable(CrystalContract.Variable, "1");
        }
    }
}
