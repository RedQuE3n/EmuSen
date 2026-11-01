using System;
using Xunit;

namespace EmuSen.WiseMan.Crystal
{
    // A case of an answer only the Rust implementation gives, skipped at EMUSEN_CRYSTAL_NATIVE=0; why names the answer - see EmuSen_Backend.md §34 and §35.
    public sealed class RustImplementationFactAttribute : FactAttribute
    {
        public RustImplementationFactAttribute(string why)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_CRYSTAL_NATIVE") != "1") Skip = $"the Rust implementation's case: {why}";
        }
    }
}
