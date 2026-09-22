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
            new("SNES (Venus)", new[] { ".smc", ".sfc" }, SnesCheatSystems, "SNES", "Nintendo", 1990, CoverAspect: 0.73);

        // The libretro folder name for the NES; Famicom Disk System is different hardware and is not claimed.
        private static readonly string[] NesCheatSystems =
        {
            "Nintendo - Nintendo Entertainment System",
        };

        private static readonly CoreDescriptor Moon =
            new("NES (Moon)", new[] { ".nes" }, NesCheatSystems, "NES", "Nintendo", 1983, CoverAspect: 1.43);

        // Two libretro folders for one core, the same way Venus claims Satellaview - see Mercury_Core.md §1.
        private static readonly string[] GameBoyCheatSystems =
        {
            "Nintendo - Game Boy",
            "Nintendo - Game Boy Color",
        };

        private static readonly CoreDescriptor Mercury =
            new("Game Boy (Mercury)", new[] { ".gb", ".gbc" }, GameBoyCheatSystems, "GB", "Nintendo", 1989, CoverAspect: 1.0);

        // Claimed so `cheat db prune` keeps it, though Mars applies no cheats yet - see Mars_Core.md §8.
        private static readonly string[] N64CheatSystems =
        {
            "Nintendo - Nintendo 64",
        };

        // All three container orders, because the magic word decides and the extension does not - see Mars_Rom.md §1.1.
        private static readonly CoreDescriptor Mars =
            new("Nintendo 64 (Mars)", new[] { ".z64", ".n64", ".v64" }, N64CheatSystems, "N64", "Nintendo", 1996, CoverAspect: 0.7);

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
                ["mars"] = Mars,
                ["n64"] = Mars,
            };

        // What `cheat db prune` keeps.
        public static IReadOnlyCollection<string> SupportedCheatSystems =>
            CoreDescriptor.SupportedCheatSystems(Registry.Values);

        // One entry per real core, not per alias - what a "which console?" list shows.
        public static IReadOnlyList<CoreDescriptor> Cores { get; } = new[] { Venus, Moon, Mercury, Mars };

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
                [Mars.Console] = Nintendo.Mars.MarsCore.PadButtons,
            };

        // The console's pad with no ROM loaded, or every button for a console this build does not know.
        public static IReadOnlyList<Galaxia.Input.PadButton> ButtonsFor(string console) =>
            ButtonsByConsole.TryGetValue(console, out var buttons) ? buttons : Enum.GetValues<Galaxia.Input.PadButton>();

        // Only a console with a stick is listed; every other one reads no axis - see EmuSen_Input.md §7.
        private static readonly Dictionary<string, IReadOnlyList<Galaxia.Input.PadAxis>> AxesByConsole =
            new(StringComparer.OrdinalIgnoreCase)
            {
                [Mars.Console] = Nintendo.Mars.MarsCore.PadAxes,
            };

        public static IReadOnlyList<Galaxia.Input.PadAxis> AxesFor(string console) =>
            AxesByConsole.TryGetValue(console, out var axes) ? axes : Array.Empty<Galaxia.Input.PadAxis>();

        // What the rebind window lists for a console: its buttons, then each direction of each stick it reads.
        public static IReadOnlyList<Galaxia.Input.PadControl> ControlsFor(string console) =>
            Galaxia.Input.PadControls.For(ButtonsFor(console), AxesFor(console));

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
        // The settings each console's core offers a frontend, by console, empty for one that offers none - see EmuSen_Multicore.md §13.
        private static readonly Dictionary<string, IReadOnlyList<CoreSetting>> SettingsByConsole = new(StringComparer.OrdinalIgnoreCase)
        {
            ["N64"] = Nintendo.Mars.MarsCore.VideoSettings,
        };

        public static IReadOnlyList<CoreSetting> SettingsFor(string console) =>
            SettingsByConsole.TryGetValue(console, out var settings) ? settings : Array.Empty<CoreSetting>();

        public static IReadOnlyList<string> FilterChoices { get; } =
            new[] { AllConsoles }.Concat(Cores.Select(c => c.DisplayName)).ToArray();
    }
}
