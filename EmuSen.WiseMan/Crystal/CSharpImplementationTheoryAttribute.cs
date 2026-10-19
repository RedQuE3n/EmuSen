using System;
using Xunit;

namespace EmuSen.WiseMan.Crystal
{
    // CSharpImplementationFact's twin for a theory: skipped when EMUSEN_CRYSTAL_NATIVE=1 puts the binding on the Rust - see EmuSen_Backend.md §29.27.
    public sealed class CSharpImplementationTheoryAttribute : TheoryAttribute
    {
        public CSharpImplementationTheoryAttribute(string why)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_CRYSTAL_NATIVE") == "1") Skip = $"the C# implementation's case: {why}";
        }
    }
}
