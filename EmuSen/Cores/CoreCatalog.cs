using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.Cores
{
    // Every core this build actually implements, in one place - see EmuSen_Settings_Reference.md §4.16.
    public static class CoreCatalog
    {
        // The SNES's facts are its system pack's, so every SNES engine and this row read one source - see EmuSen_CoreAPI.md §20.
        private static readonly CoreDescriptor Venus = FromSystem("SNES (Venus)", DianaOS.DianaOS.Sys.Systems.Snes.SnesSystem.Entry);

        private static CoreDescriptor FromSystem(string displayName, DianaOS.DianaOS.Sys.Systems.SystemEntry e) =>
            new(displayName, e.Extensions.ToArray(), e.CheatSystems.ToArray(), e.Console, e.Manufacturer, e.ReleaseYear, CoverAspect: e.CoverAspect,
                OpenVgdbSystems: e.OpenVgdbSystems.ToArray(), OpenVgdbBytes: e.OpenVgdbBytes);

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

        // What `cheat db prune` keeps: the catalog's systems and those of the consoles discovery adds.
        public static IReadOnlyCollection<string> SupportedCheatSystems =>
            CoreDescriptor.SupportedCheatSystems(Registry.Values.Concat(Lists.Discovered));

        // The C# cores this build registers by hand.
        private static readonly CoreDescriptor[] CatalogCores = { Venus, Moon, Mercury, Mars };

        // One entry per real core, not per alias - what a "which console?" list shows; a console only a discovered engine runs follows the catalog's - see EmuSen_Settings_Reference.md §4.92.
        public static IReadOnlyList<CoreDescriptor> Cores => Lists.Cores;

        // Cores sorted for display: grouped by manufacturer, oldest console first - see EmuSen_Input.md §5.1.
        public static IReadOnlyList<CoreDescriptor> ConsolesInReleaseOrder => Lists.Consoles;

        // A row discovery added rather than a C# core of this build.
        public static bool IsDiscovered(CoreDescriptor core) => !CatalogCores.Contains(core);

        // Whether a C# core of this build claims the extension, which is what decides the factory's branch.
        public static bool IsCatalogExtension(string extension) => CatalogCores.Any(c => c.SupportsExtension(extension));

        // The catalog's lists with the consoles discovery adds, made again whenever discovery scans again.
        private sealed record CatalogLists(IReadOnlyList<Native.DiscoveredCore>? Found, CoreDescriptor[] Discovered, CoreDescriptor[] Cores, CoreDescriptor[] Consoles,
            LibraryShelf[] Shelves, string[] Extensions, string[] Filters);

        private static CatalogLists? _lists;

        // A console's row, built once from its system pack, so a row compared by reference stays the same row.
        private static readonly System.Collections.Concurrent.ConcurrentDictionary<string, CoreDescriptor> DiscoveredRows = new(StringComparer.OrdinalIgnoreCase);

        private static CatalogLists Lists
        {
            get
            {
                var found = Native.CoreDiscovery.Found;
                if (_lists is { } cached && ReferenceEquals(cached.Found, found)) return cached;
                var discovered = DiscoveredConsoles.Select(console => DiscoveredRows.GetOrAdd(console, DiscoveredRow)).ToArray();
                var cores = CatalogCores.Concat(discovered).ToArray();
                var consoles = cores.OrderBy(c => c.Manufacturer, StringComparer.OrdinalIgnoreCase).ThenBy(c => c.ReleaseYear).ThenBy(c => c.Console, StringComparer.OrdinalIgnoreCase).ToArray();
                var shelves = consoles.SelectMany(Shelves).ToArray();
                var extensions = cores.SelectMany(c => c.Extensions).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();
                var filters = new[] { AllConsoles }.Concat(cores.SelectMany(c => ReferenceEquals(c, Mercury) ? new[] { c.DisplayName, GameBoyColorShelf } : new[] { c.DisplayName })).ToArray();
                return _lists = new CatalogLists(found, discovered, cores, consoles, shelves, extensions, filters);
            }
        }

        private static DianaOS.DianaOS.Sys.Systems.SystemEntry? PackOf(string console) =>
            DianaOS.DianaOS.Sys.Systems.SystemPacks.All.Select(p => p.Entry).FirstOrDefault(e => string.Equals(e.Console, console, StringComparison.OrdinalIgnoreCase) && ConsoleForSystem(e.Id) is null);

        // "Genesis (Nephrite)": the pack's console and the engine that runs it, as "SNES (Venus)" is.
        private static CoreDescriptor DiscoveredRow(string console) => FromSystem($"{console} ({DiscoveredFor(console)?.EngineName ?? console})", PackOf(console)!);

        // What the library's console filter and sidebar list: one shelf per core, but two for the Game Boy's, whose games a player knows as two consoles - see EmuSen_Settings_Reference.md §4.46.
        public sealed record LibraryShelf(string Name, string Label, CoreDescriptor Core)
        {
            // The system name an ES-DE theme keys its folders and variables by, and the full name it shows - see EmuSen_BigPicture.md §4.1.
            public string EsdeSystem { get; init; } = "";
            public string EsdeFullName { get; init; } = "";
        }

        public const string GameBoyColorShelf = "Game Boy Color (Mercury)";

        private static readonly Dictionary<string, (string System, string FullName)> EsdeNames = new(StringComparer.OrdinalIgnoreCase)
        {
            ["NES"] = ("nes", "Nintendo Entertainment System"),
            [Venus.Console] = (DianaOS.DianaOS.Sys.Systems.Snes.SnesSystem.Entry.EsdeSystem, DianaOS.DianaOS.Sys.Systems.Snes.SnesSystem.Entry.EsdeFullName),
            ["N64"] = ("n64", "Nintendo 64"),
            ["GB"] = ("gb", "Game Boy"),
        };

        public static IReadOnlyList<LibraryShelf> ShelvesInReleaseOrder => Lists.Shelves;

        // A console's shelves; a discovered console's ES-DE names are its pack's.
        private static IEnumerable<LibraryShelf> Shelves(CoreDescriptor c)
        {
            if (ReferenceEquals(c, Mercury))
                return new[] { new LibraryShelf(c.DisplayName, c.Console, c) { EsdeSystem = "gb", EsdeFullName = "Game Boy" }, new LibraryShelf(GameBoyColorShelf, "GBC", c) { EsdeSystem = "gbc", EsdeFullName = "Game Boy Color" } };
            var (system, fullName) = EsdeNames.TryGetValue(c.Console, out var names) ? names : (PackOf(c.Console)!.EsdeSystem, PackOf(c.Console)!.EsdeFullName);
            return new[] { new LibraryShelf(c.DisplayName, c.Console, c) { EsdeSystem = system, EsdeFullName = fullName } };
        }


        // Null for AllConsoles or a name no shelf has.
        public static LibraryShelf? ShelfByName(string? name) =>
            ShelvesInReleaseOrder.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));

        // The shelf a ROM sits on, or null for a file no core claims.
        public static string? ShelfFor(string romPath) =>
            ByExtension(System.IO.Path.GetExtension(romPath)) is not { } core ? null
            : ReferenceEquals(core, Mercury) && IsGameBoyColor(romPath) ? GameBoyColorShelf
            : core.DisplayName;

        private static readonly System.Collections.Concurrent.ConcurrentDictionary<string, bool> ColorByPath = new();

        // A .gbc file, or a .gb whose header asks for the Color (0x80 or 0xC0 at 0x143, as Mercury reads it); cached, since a scan asks per file.
        public static bool IsGameBoyColor(string romPath) =>
            ColorByPath.GetOrAdd(romPath, path =>
            {
                if (string.Equals(System.IO.Path.GetExtension(path), ".gbc", StringComparison.OrdinalIgnoreCase)) return true;
                try
                {
                    using var file = System.IO.File.OpenRead(path);
                    if (file.Length <= 0x143) return false;
                    file.Position = 0x143;
                    return file.ReadByte() is 0x80 or 0xC0;
                }
                catch (Exception ex) when (ex is System.IO.IOException or UnauthorizedAccessException)
                {
                    return false;
                }
            });

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
            ButtonsByConsole.TryGetValue(console, out var buttons) ? buttons
            : DiscoveredSystem(console) is { } system && system.Controllers.SelectMany(c => c.Buttons).Select(b => b.Control).OfType<Galaxia.Input.PadButton>().Distinct().ToArray() is { Length: > 0 } pad ? pad
            : Enum.GetValues<Galaxia.Input.PadButton>();

        // The controller a port of a console discovery added is set to, by its id: the engine's "pad<n>" setting as stored, else its default; null where the engine has no such setting.
        public static string? ControllerFor(string console, int port, Func<string, string?> stored)
        {
            if (DiscoveredSystem(console) is not { } system) return null;
            if (PlayerControllersFor(console, stored) is { } players) return port >= 0 && port < players.Count && system.Controllers.Any(c => c.Id == players[port]) ? players[port] : null;
            if (SettingsFor(console).FirstOrDefault(s => s.Key == $"pad{port + 1}") is not { } setting) return null;
            string value = stored(setting.Key) ?? setting.Default;
            return system.Controllers.Any(c => c.Id == value) ? value : null;
        }

        // Each player's controller where the console's pack counts them from its settings, a setting not stored read at the engine's default; null where it does not.
        public static IReadOnlyList<string>? PlayerControllersFor(string console, Func<string, string?> stored)
        {
            if (DiscoveredSystem(console) is null || PackOf(console) is not { } entry || DianaOS.DianaOS.Sys.Systems.SystemPacks.For(entry.Id)?.PlayerControllers is not { } players) return null;
            var settings = SettingsFor(console);
            return players(key => stored(key) ?? settings.FirstOrDefault(s => s.Key == key)?.Default);
        }

        // The system entry of a console discovery added, from the engine's info, for its pad and its ports.
        public static Native.CoreSystem? DiscoveredSystem(string console) =>
            DiscoveredFor(console) is { } core && PackOf(console) is { } pack ? core.Info.Systems.FirstOrDefault(s => s.Id == pack.Id) : null;

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
        public static IReadOnlyList<string> RomExtensions => Lists.Extensions;

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
            ["GB"] = Nintendo.Mercury.MercuryCore.ModelSettings,
        };

        public static IReadOnlyList<CoreSetting> SettingsFor(string console) =>
            SettingsByConsole.TryGetValue(console, out var settings) ? settings
            : DiscoveredFor(console) is { } found ? CoreSidecarSettings(found) : Array.Empty<CoreSetting>();

        // A console no catalog core runs, whose system pack names it and a discovered engine serves it - see EmuSen_Settings_Reference.md §4.90.
        public static IReadOnlyList<string> DiscoveredConsoles =>
            DianaOS.DianaOS.Sys.Systems.SystemPacks.All.Select(p => p.Entry)
                .Where(e => ConsoleForSystem(e.Id) is null && Native.CoreDiscovery.ForSystem(e.Id).Any(c => !IsRegisteredEngine(c.EngineName) && Runs(c, e.Id)))
                .OrderBy(e => e.Manufacturer, StringComparer.OrdinalIgnoreCase).ThenBy(e => e.ReleaseYear)
                .Select(e => e.Console).Distinct(StringComparer.OrdinalIgnoreCase).ToArray();

        // The pack console of a system only a discovered engine serves, or null for a catalog console's or an unknown system.
        public static string? DiscoveredConsoleForSystem(string? systemId) =>
            systemId is not null && ConsoleForSystem(systemId) is null && DianaOS.DianaOS.Sys.Systems.SystemPacks.For(systemId) is { } pack ? pack.Entry.Console : null;

        // The console a discovered engine runs a file as, by the pack of the engine's system that claims the extension.
        public static string? DiscoveredConsoleForFile(string romPath, Native.CoreInfo info) =>
            info.SystemFor(System.IO.Path.GetExtension(romPath)) is { } system ? DiscoveredConsoleForSystem(system.Id) : null;

        private static Native.DiscoveredCore? DiscoveredFor(string console) =>
            DianaOS.DianaOS.Sys.Systems.SystemPacks.All.Where(p => string.Equals(p.Entry.Console, console, StringComparison.OrdinalIgnoreCase) && ConsoleForSystem(p.Entry.Id) is null)
                .SelectMany(p => Native.CoreDiscovery.ForSystem(p.Entry.Id).Where(c => Runs(c, p.Entry.Id))).FirstOrDefault(c => !IsRegisteredEngine(c.EngineName));

        // A console gets its tab and shelf only once its engine says its games run, even where development cores are shown - see EmuSen_CoreAPI.md §27.4.
        private static bool Runs(Native.DiscoveredCore core, string systemId) => core.Info.Systems.Any(s => s.Id == systemId && !s.Development);

        // The engine's schema as the window lists it, from the library its sidecar names.
        private static IReadOnlyList<CoreSetting> CoreSidecarSettings(Native.DiscoveredCore found) =>
            found.Open() is { } library ? library.Settings.Where(s => !s.Hidden).Select(s => s.AsCoreSetting()).OfType<CoreSetting>().ToArray() : Array.Empty<CoreSetting>();

        // The graphics.json key a console's engine is stored under; no core declares it, so no core is handed it - see EmuSen_Settings_Reference.md §4.44.
        public const string EngineKey = "Engine";

        public const string MarsEngine = "Mars (C#)";
        public const string MarsRtEngine = "MarsRT (Rust)";

        public const string MercuryEngine = "Mercury (C#)";
        public const string MercuryRtEngine = "MercuryRT (Rust)";

        public const string MoonEngine = "Moon (C#)";
        public const string MoonRtEngine = "MoonRT (Rust)";

        // Which implementation runs a console, for the consoles that have more than one; the first is the default - see EmuSen_Settings_Reference.md §4.44.
        private static readonly Dictionary<string, CoreSetting> EngineByConsole = new(StringComparer.OrdinalIgnoreCase)
        {
            ["N64"] = new(EngineKey, "Engine",
                "Which implementation runs the console. MarsRT (Rust) is the default: exact against Mars (C#) in state, picture and sound, reading the same save states and battery saves, and honouring every setting below, the resolution multiple, antialiasing and the graphics card included. Mars (C#) is the reference it is graded against, and runs instead where MarsRT's library is missing. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, MarsRtEngine, Choices: new[] { MarsRtEngine, MarsEngine }),
            ["GB"] = new(EngineKey, "Engine",
                "Which implementation runs the console. Mercury (C#) is the reference. MercuryRT (Rust) is exact against it in state, picture and sound and reads the same save states and battery saves; its debugger view is refreshed from its state, and it stops at breakpoints, steps and records coverage, watches and the call stack as Mercury does. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, MercuryEngine, Choices: new[] { MercuryEngine, MercuryRtEngine }),
            ["NES"] = new(EngineKey, "Engine",
                "Which implementation runs the console. Moon (C#) is the reference. MoonRT (Rust) is exact against it in state, picture and sound, reads the same save states and battery saves, and takes the same cheats; its debugger view is refreshed from its state, and it stops at breakpoints, steps, runs to a frame or an interrupt, and records coverage, watches, the profile and the call stack as Moon does. Takes effect when a game is next loaded.",
                CoreSettingKind.Choice, MoonEngine, Choices: new[] { MoonEngine, MoonRtEngine }),
        };

        public const string VenusEngine = "Venus (C#)";
        public const string VenusRtEngine = "VenusRT (Rust)";

        // A discovered engine that is a console's default whenever its library is found, the reference staying a choice - see EmuSen_Settings_Reference.md §4.44.
        private static readonly Dictionary<string, (string Engine, string Description)> DefaultDiscoveredByConsole = new(StringComparer.OrdinalIgnoreCase)
        {
            ["SNES"] = (VenusRtEngine,
                "Which implementation runs the console. VenusRT (Rust) is the default. Venus (C#) is the reference, kept for now, and runs instead where VenusRT's library is missing. A save state made by Venus (C#) does not load in VenusRT, though battery saves cross between the two. Takes effect when a game is next loaded."),
        };

        // The system ids of EmuSen_CoreAPI.md §6.3 each console answers to, so a v1 core's info is matched to it.
        private static readonly Dictionary<string, string[]> SystemIdsByConsole = new(StringComparer.OrdinalIgnoreCase)
        {
            ["SNES"] = new[] { DianaOS.DianaOS.Sys.Systems.Snes.SnesSystem.Id },
            ["NES"] = new[] { "nes" },
            ["GB"] = new[] { "gb", "gbc" },
            ["N64"] = new[] { "n64" },
        };

        public static IReadOnlyList<string> SystemIdsFor(string console) => SystemIdsByConsole.TryGetValue(console, out var ids) ? ids : Array.Empty<string>();

        public static string? ConsoleForSystem(string systemId) => SystemIdsByConsole.FirstOrDefault(c => c.Value.Contains(systemId)).Key;

        // The C# core each console's engines are graded against, the default of a row discovery adds.
        private static readonly Dictionary<string, string> ReferenceEngineByConsole = new(StringComparer.OrdinalIgnoreCase)
        {
            ["SNES"] = VenusEngine,
            ["NES"] = MoonEngine,
            ["GB"] = MercuryEngine,
            ["N64"] = MarsEngine,
        };

        public static string? ReferenceEngine(string console) => ReferenceEngineByConsole.GetValueOrDefault(console);

        // The v1 engines discovery found for a console, by the names their info gives them - see EmuSen_CoreAPI.md §19.
        public static IReadOnlyList<string> DiscoveredEngines(string console) =>
            SystemIdsFor(console).SelectMany(Native.CoreDiscovery.ForSystem).Select(c => c.EngineName).Distinct().ToArray();

        // An engine this build registers by hand, whose own branch runs it even when discovery also lists its library - see EmuSen_CoreAPI.md §22.
        public static bool IsRegisteredEngine(string? name) => name is not null && EngineByConsole.Values.Any(row => row.Choices?.Contains(name) == true);

        // Null for a console with one implementation; a v1 engine found by discovery is appended to the row, which it creates for a console that had none.
        public static CoreSetting? EngineFor(string console)
        {
            EngineByConsole.TryGetValue(console, out var row);
            string[] found = DiscoveredEngines(console).Where(n => row?.Choices?.Contains(n) != true).ToArray();
            if (found.Length == 0) return row;
            if (row is null)
            {
                if (!ReferenceEngineByConsole.TryGetValue(console, out var reference)) return null;
                if (DefaultDiscoveredByConsole.TryGetValue(console, out var first) && found.Contains(first.Engine))
                    return new CoreSetting(EngineKey, "Engine", first.Description, CoreSettingKind.Choice, first.Engine,
                        Choices: new[] { first.Engine, reference }.Concat(found.Where(n => n != first.Engine)).ToArray());
                row = new CoreSetting(EngineKey, "Engine",
                    $"Which implementation runs the console. {reference} is the reference and the default; each other engine is a core found beside the program and described by its own info. Takes effect when a game is next loaded.",
                    CoreSettingKind.Choice, reference, Choices: new[] { reference });
            }
            return row with { Choices = row.Choices!.Concat(found).ToArray() };
        }

        // The engine a frontend runs for a console: the stored choice, else the row's default; null for a console with one - see EmuSen_Settings_Reference.md §4.44.
        public static string? EngineChosen(string console, string? stored) => stored ?? EngineFor(console)?.Default;

        public static IReadOnlyList<string> FilterChoices => Lists.Filters;
    }
}
