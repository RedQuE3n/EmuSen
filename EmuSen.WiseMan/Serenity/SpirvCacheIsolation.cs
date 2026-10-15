using System.IO;
using System.Runtime.CompilerServices;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.Serenity.Slang;

namespace EmuSen.WiseMan.Serenity
{
    // Presets the suite builds, and the built-in filters it writes as presets, stay in the suite's scratch - see EmuSen_Serenity.md §9.4.
    internal static class SpirvCacheIsolation
    {
        [ModuleInitializer]
        internal static void Isolate()
        {
            SlangRunner.CachePath = Path.Combine(DianaShSandbox.ScratchDirectory, "spirv-cache.db");
            EmuSen.Serenity.Shaders.BuiltInPreset.RootOverride = Path.Combine(DianaShSandbox.ScratchDirectory, "builtin-presets");
        }
    }
}
