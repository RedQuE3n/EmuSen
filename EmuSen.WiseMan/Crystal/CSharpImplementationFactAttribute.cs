using System;
using Xunit;

namespace EmuSen.WiseMan.Crystal
{
    // A case that reaches the C# implementation's own types or statics, skipped when EMUSEN_CRYSTAL_NATIVE=1 puts the binding on the Rust; why names what holds the Rust - see EmuSen_Backend.md §29.27.
    public sealed class CSharpImplementationFactAttribute : FactAttribute
    {
        public CSharpImplementationFactAttribute(string why)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_CRYSTAL_NATIVE") == "1") Skip = $"the C# implementation's case: {why}";
        }
    }
}
