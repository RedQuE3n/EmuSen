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
            new("SNES (Venus)", new[] { ".smc", ".sfc" }, SnesCheatSystems, "SNES", "Nintendo", 1990, CoverAspect: 0.73,
                OpenVgdbSystems: new[] { "SNES" }, OpenVgdbBytes: WithoutCopierHeader);

        // The libretro folder name for the NES; Famicom Disk System is different hardware and is not claimed.
        private static readonly string[] NesCheatSystems =
        {
            "Nintendo - Nintendo Entertainment System",
        };

        private static readonly CoreDescriptor Moon =
            new("NES (Moon)", new[] { ".nes" }, NesCheatSystems, "NES", "Nintendo", 1983, CoverAspect: 1.43,
                OpenVgdbSystems: new[] { "NES" }, OpenVgdbBytes: WithoutInesHeader);

        // Two libretro folders for one core, the same way Venus claims Satellaview - see Mercury_Core.md §1.
        private static readonly string[] GameBoyCheatSystems =
        {
            "Nintendo - Game Boy",
            "Nintendo - Game Boy Color",
        };

        private static readonly CoreDescriptor Mercury =
            new("Game Boy (Mercury)", new[] { ".gb", ".gbc" }, GameBoyCheatSystems, "GB", "Nintendo", 1989, CoverAspect: 1.0,
                OpenVgdbSystems: new[] { "GB", "GBC" }, OpenVgdbBytes: file => file);

        // Claimed so `cheat db prune` keeps it, though Mars applies no cheats yet - see Mars_Core.md §8.
        private static readonly string[] N64CheatSystems =
        {
            "Nintendo - Nintendo 64",
        };

        // All three container orders, because the magic word decides and the extension does not - see Mars_Rom.md §1.1.
        private static readonly CoreDescriptor Mars =
            new("Nintendo 64 (Mars)", new[] { ".z64", ".n64", ".v64" }, N64CheatSystems, "N64", "Nintendo", 1996, CoverAspect: 0.7,
                OpenVgdbSystems: new[] { "N64" }, OpenVgdbBytes: InByteSwappedOrder);

        // OpenVGDB hashes a SNES image without the 512-byte copier header some dumps carry, as No-Intro does - see EmuSen_Settings_Reference.md §4.39.
        private static byte[] WithoutCopierHeader(byte[] file) => file.Length % 1024 == 512 ? file[512..] : file;

        // Its SYSTEMS row gives the NES a 16-byte header to skip, which is the iNES header when the file has one.
        private static byte[] WithoutInesHeader(byte[] file) =>
            file.Length > 16 && file[0] == (byte)'N' && file[1] == (byte)'E' && file[2] == (byte)'S' && file[3] == 0x1A ? file[16..] : file;

        // Measured on this library: its N64 hashes are of the halfword-swapped .v64 order, whatever order the file is in.
        private static byte[] InByteSwappedOrder(byte[] file)
        {
            try
            {
                var order = Nintendo.Mars.Rom.RomImage.DetectByteOrder(file);
                byte[] big = Nintendo.Mars.Rom.RomImage.Normalize(file, order);
                return Nintendo.Mars.Rom.RomImage.Normalize(big, Nintendo.Mars.Rom.RomByteOrder.ByteSwapped);
            }
            catch (System.IO.InvalidDataException)
            {
                return file;
            }
        }

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

        // The graphics.json key a console's engine is stored under; no core declares it, so no core is handed it - see EmuSen_Settings_Reference.md §4.44.
        public const string EngineKey = "Engine";

        public const string MarsEngine = "Mars (C#)";
        public const string MarsRtEngine = "MarsRT (Rust)";

        // Which implementation runs a console, for the consoles that have more than one; the first is the default - see EmuSen_Settings_Reference.md §4.44.
        private static readonly Dictionary<string, CoreSetting> EngineByConsole = new(StringComparer.OrdinalIgnoreCase)
        {
            ["N64"] = new(EngineKey, "Engine",
                "Which implementation runs the console. Mars (C#) is the reference. MarsRT (Rust) is exact against it in state, picture and sound and reads the same save states and battery saves; of the settings below it honours only the Expansion Pak so far. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, MarsEngine, Choices: new[] { MarsEngine, MarsRtEngine }),
        };

        // Null for a console with one implementation.
        public static CoreSetting? EngineFor(string console) =>
            EngineByConsole.TryGetValue(console, out var engine) ? engine : null;

        public static IReadOnlyList<string> FilterChoices { get; } =
            new[] { AllConsoles }.Concat(Cores.Select(c => c.DisplayName)).ToArray();
    }
}
