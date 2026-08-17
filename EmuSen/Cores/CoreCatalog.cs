using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.Cores
{
    // Every core this build actually implements, in one place - see EmuSen_Settings_Reference.md §4.16.
    public static class CoreCatalog
    {
        // The libretro cheat-database folder names for the SNES - `man cheat`.
        private static readonly string[] SnesCheatSystems =
        {
            "Nintendo - Super Nintendo Entertainment System",
            "Nintendo - Satellaview",
        };

        private static readonly CoreDescriptor Venus =
            new("SNES (Venus)", new[] { ".smc", ".sfc" }, SnesCheatSystems, "SNES", "Nintendo", 1990);

        // The libretro folder name for the NES; Famicom Disk System is different hardware and is not claimed.
        private static readonly string[] NesCheatSystems =
        {
            "Nintendo - Nintendo Entertainment System",
        };

        private static readonly CoreDescriptor Moon =
            new("NES (Moon)", new[] { ".nes" }, NesCheatSystems, "NES", "Nintendo", 1983);

        // Two libretro folders for one core, the same way Venus claims Satellaview - see Mercury_Core.md §1.
        private static readonly string[] GameBoyCheatSystems =
        {
            "Nintendo - Game Boy",
            "Nintendo - Game Boy Color",
        };

        private static readonly CoreDescriptor Mercury =
            new("Game Boy (Mercury)", new[] { ".gb", ".gbc" }, GameBoyCheatSystems, "GB", "Nintendo", 1989);

        // Keyed by what a user would type - the internal codename and the console name both reach the same core.
        public static IReadOnlyDictionary<string, CoreDescriptor> Registry { get; } =
            new Dictionary<string, CoreDescriptor>(StringComparer.OrdinalIgnoreCase)
            {
                ["venus"] = Venus,
                ["snes"] = Venus,
                ["moon"] = Moon,
                ["nes"] = Moon,
                ["mercury"] = Mercury,
                ["gb"] = Mercury,
                ["gbc"] = Mercury,
            };

        // What `cheat db prune` keeps.
        public static IReadOnlyCollection<string> SupportedCheatSystems =>
            CoreDescriptor.SupportedCheatSystems(Registry.Values);

        // One entry per real core, not per alias - what a "which console?" list shows.
        public static IReadOnlyList<CoreDescriptor> Cores { get; } = new[] { Venus, Moon, Mercury };

        // Cores sorted for display: grouped by manufacturer, oldest console first - see EmuSen_Input.md §5.1.
        public static IReadOnlyList<CoreDescriptor> ConsolesInReleaseOrder { get; } =
            Cores.OrderBy(c => c.Manufacturer, StringComparer.OrdinalIgnoreCase)
                 .ThenBy(c => c.ReleaseYear)
                 .ThenBy(c => c.Console, StringComparer.OrdinalIgnoreCase)
                 .ToArray();

        // Read off the core itself rather than restated here, so a pad cannot drift from what the core reads.
        private static readonly Dictionary<string, IReadOnlyList<Galaxia.Input.PadButton>> ButtonsByConsole =
            new(StringComparer.OrdinalIgnoreCase)
            {
                [Venus.Console] = Nintendo.Venus.VenusCore.PadButtons,
                [Moon.Console] = Nintendo.Moon.MoonCore.PadButtons,
                [Mercury.Console] = Nintendo.Mercury.MercuryCore.PadButtons,
            };

        // The console's pad with no ROM loaded, or every button for a console this build does not know.
        public static IReadOnlyList<Galaxia.Input.PadButton> ButtonsFor(string console) =>
            ButtonsByConsole.TryGetValue(console, out var buttons) ? buttons : Enum.GetValues<Galaxia.Input.PadButton>();

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

        // The console a ROM will load as, answerable before the core exists - see EmuSen_Multicore.md §12.
        public static string? ConsoleForRom(string romPath) =>
            ByExtension(System.IO.Path.GetExtension(romPath))?.Console;

        // Null for AllConsoles, an unknown name, or a name from a build that had a core this one lacks.
        public static CoreDescriptor? ByDisplayName(string? displayName) =>
            Cores.FirstOrDefault(c => string.Equals(c.DisplayName, displayName, StringComparison.OrdinalIgnoreCase));

        // Either name a console goes by: "SNES (Venus)" or "SNES" - see EmuSen_Multicore.md §10.
        public static CoreDescriptor? ByAnyName(string? name) =>
            ByDisplayName(name)
            ?? Cores.FirstOrDefault(c => string.Equals(c.Console, name, StringComparison.OrdinalIgnoreCase));

        // AllConsoles first, so a filter combo can bind straight to it.
        public static IReadOnlyList<string> FilterChoices { get; } =
            new[] { AllConsoles }.Concat(Cores.Select(c => c.DisplayName)).ToArray();
    }
}
