using System.IO;
using System.Runtime.CompilerServices;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.Serenity.Slang;

namespace EmuSen.WiseMan.Serenity
{
    // Presets the suite builds keep their SPIR-V in the suite's scratch, not the data home a player's build reads - see EmuSen_Serenity.md §9.4.
    internal static class SpirvCacheIsolation
    {
        [ModuleInitializer]
        internal static void Isolate() => SlangRunner.CachePath = Path.Combine(DianaOSSandbox.ScratchDirectory, "spirv-cache.db");
    }
}
