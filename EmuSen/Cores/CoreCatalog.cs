using System;
using System.Collections.Generic;
using System.Linq;
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

        // The libretro folder name for the NES; Famicom Disk System is different hardware and is not claimed.
        private static readonly string[] NesCheatSystems =
        {
            "Nintendo - Nintendo Entertainment System",
        };

        private static readonly CoreDescriptor Moon =
            new("NES (Moon)", new[] { ".nes" }, NesCheatSystems);

        // Keyed by what a user would type - the internal codename and the
        // console name both reach the same core.
        public static IReadOnlyDictionary<string, CoreDescriptor> Registry { get; } =
            new Dictionary<string, CoreDescriptor>(StringComparer.OrdinalIgnoreCase)
            {
                ["venus"] = Venus,
                ["snes"] = Venus,
                ["moon"] = Moon,
                ["nes"] = Moon,
            };

        // What `cheat db prune` keeps. Deduplicated, since one core is
        // registered under several aliases above.
        public static IReadOnlyCollection<string> SupportedCheatSystems =>
            CoreDescriptor.SupportedCheatSystems(Registry.Values);

        // One entry per real core, not per alias - what a "which console?" list shows.
        public static IReadOnlyList<CoreDescriptor> Cores { get; } = new[] { Venus, Moon };

        // Every extension any core in this build claims - see EmuSen_Multicore.md §3.
        public static IReadOnlyList<string> RomExtensions { get; } =
            Cores.SelectMany(c => c.Extensions).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();

        public static bool IsRomExtension(string extension) =>
            Cores.Any(c => c.SupportsExtension(extension));

        // The no-filter choice, spelled once in Galaxia because that is what persists it.
        public static string AllConsoles => EmuSen.Galaxia.Models.AppSettings.AllConsoles;

        // Which core claims this ROM extension, or null for one nothing handles.
        public static CoreDescriptor? ByExtension(string extension) =>
            Cores.FirstOrDefault(c => c.SupportsExtension(extension));

        // Null for AllConsoles, an unknown name, or a name from a build that had a core this one lacks.
        public static CoreDescriptor? ByDisplayName(string? displayName) =>
            Cores.FirstOrDefault(c => string.Equals(c.DisplayName, displayName, StringComparison.OrdinalIgnoreCase));

        // AllConsoles first, so a filter combo can bind straight to it.
        public static IReadOnlyList<string> FilterChoices { get; } =
            new[] { AllConsoles }.Concat(Cores.Select(c => c.DisplayName)).ToArray();
    }
}
