using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.Cores
{
    // Every core this build actually implements, in one place. Frontends
    // used to each carry their own copy of this list; `cheat db prune` needs
    // a third reader, and three copies of "which consoles exist" is how they
    // drift apart. See EmuSen_Settings_Reference.md §4.16.
    //
    // Lives here rather than in EmuSen.DianaOS on purpose: DianaOS stays
    // agnostic about which concrete cores exist (same reasoning as
    // IDebugTarget and ICheatCodeCodec), and this assembly is the one that
    // already knows VenusCore by name.
    public static class CoreCatalog
    {
        // The libretro cheat-database folder names for the SNES. Satellaview
        // is its own folder there but the same 65816 cartridge hardware, so
        // Venus claims both - see `man cheat`.
        private static readonly string[] SnesCheatSystems =
        {
            "Nintendo - Super Nintendo Entertainment System",
            "Nintendo - Satellaview",
        };

        private static readonly CoreDescriptor Venus =
            new("SNES (Venus)", new[] { ".smc", ".sfc" }, SnesCheatSystems);

        // Keyed by what a user would type - the internal codename and the
        // console name both reach the same core.
        public static IReadOnlyDictionary<string, CoreDescriptor> Registry { get; } =
            new Dictionary<string, CoreDescriptor>(StringComparer.OrdinalIgnoreCase)
            {
                ["venus"] = Venus,
                ["snes"] = Venus,
            };

        // What `cheat db prune` keeps. Deduplicated, since one core is
        // registered under several aliases above.
        public static IReadOnlyCollection<string> SupportedCheatSystems =>
            CoreDescriptor.SupportedCheatSystems(Registry.Values);
    }
}
