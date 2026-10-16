using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Crystal.Systems.GameBoy;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Models;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Cheats;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems;

namespace EmuSen.Cores
{
    // Everything a frontend needs for one loaded ROM, so it never names a core itself; Notice says why the engine asked for is not the one running.
    public sealed record CoreBundle(
        ICore Core,
        IDebugTarget DebugTarget,
        ICheatCodeCodec? CheatAutoDetectCodec,
        ICheatCodeCodec? CheatExplicitCodec,
        ICpuTraceSwitch? CpuTraceSwitch,
        string? Notice = null);

    // The one place a ROM path turns into a running core - see EmuSen_Multicore.md §2.
    public static class CoreFactory
    {
        // A file of an extension some console opens, an archive's by its image entry's; which console is CoreCatalog.Route's, by content - see EmuSen_CoreAPI_Extensions.md §8.4.
        public static bool IsSupported(string romPath) => CoreCatalog.IsImageExtension(Extension(romPath));

        // <headless> only means anything to a core that owns a window-ish resource; <engine> is a CoreCatalog.EngineFor name, and null is the reference core, not the player's default - see EmuSen_Settings_Reference.md §4.44.
        public static ICore Create(string romPath, bool headless = true, string? engine = null) =>
            V1Library(romPath, engine) is { } v1 ? new Native.CoreEngine(v1, StoredSettings(romPath, v1)) { Media = StoredMedia(romPath, v1) } : Extension(romPath) switch
        {
            ".smc" or ".sfc" => new VenusCore(headless),
            ".nes" => engine == CoreCatalog.MoonRtEngine && MoonRtCore.Available ? new MoonRtCore() : new MoonCore(),
            ".gb" or ".gbc" => GameBoy(),
            ".z64" or ".n64" or ".v64" => MarsRtCore.Available ? new MarsRtCore(expansionPak: true) { Media = StoredMedia(romPath, MarsNative.Library.Library!) }
                : throw new NotSupportedException($"The N64 core is not in this build ({MarsNative.Report})."),
            var other => throw new NotSupportedException(
                $"No core in this build handles '{other}' - see CoreCatalog for what is registered."),
        };

        // MercuryRT, the Game Boy's one engine; a build without its library says so rather than running nothing - see Mercury_Native.md §15.4.
        private static ICore GameBoy() => MercuryRtCore.Available ? new MercuryRtCore()
            : throw new NotSupportedException($"This build has no Game Boy core: {CoreCatalog.MercuryRtEngine} is missing ({MercuryNative.Report}).");

        // What graphics.json holds for the console the file runs as, so a setting read only at create reaches the machine - see EmuSen_Settings_Reference.md §4.90.
        private static IReadOnlyDictionary<string, string> StoredSettings(string romPath, Native.CoreLibrary library)
        {
            string? console = CoreCatalog.ConsoleForRom(romPath) ?? CoreCatalog.DiscoveredConsoleForFile(romPath, library.Info);
            if (console is null) return new Dictionary<string, string>();
            var config = GraphicsConfig.Load();
            return library.Settings.Select(s => (s.Key, Value: config.Value(console, s.Key))).Where(p => p.Value is not null).ToDictionary(p => p.Key, p => p.Value!);
        }

        // Each save medium's mode graphics.json holds for the console the file runs as - see EmuSen_SaveMedia.md §3.3.
        private static IReadOnlyDictionary<string, CardChoice> StoredMedia(string romPath, Native.CoreLibrary library)
        {
            string? console = CoreCatalog.ConsoleForRom(romPath) ?? CoreCatalog.DiscoveredConsoleForFile(romPath, library.Info);
            if (console is null) return new Dictionary<string, CardChoice>();
            var config = GraphicsConfig.Load();
            return MemoryCards.Stored(MemoryCards.MediaOf(console, CoreCatalog.SystemOfFile(romPath, library.Info)), key => config.Value(console, key));
        }

        // The one generic branch for v1 engines: the one asked for by name, or the routed console's engine for a file no C# core claims; null when it is not loadable - see EmuSen_CoreAPI.md §19.
        private static Native.CoreLibrary? V1Library(string romPath, string? engine)
        {
            string extension = Extension(romPath);
            // A file several consoles take and none recognises opens nowhere, in the decision's words, rather than on whichever engine is first.
            var (core, decision) = CoreCatalog.Route(romPath);
            if (engine is null && core is null && decision is { System: null, Reason: { } why }) throw new NotSupportedException(why);
            var routed = decision?.System?.Id ?? (core is not null && CoreCatalog.IsDiscovered(core) ? Crystal.Systems.SystemPacks.All.FirstOrDefault(p => p.Entry.Console == core.Console)?.Entry.Id : null);
            var found = engine is not null ? Native.CoreDiscovery.ForExtension(extension).FirstOrDefault(c => c.EngineName == engine)
                : CoreCatalog.IsCatalogExtension(extension) ? null
                : routed is not null ? Native.CoreDiscovery.ForSystem(routed).FirstOrDefault(c => !CoreCatalog.IsRegisteredEngine(c.EngineName))
                : Native.CoreDiscovery.ForExtension(extension).FirstOrDefault();
            return found is null || CoreCatalog.IsRegisteredEngine(found.EngineName) ? null : found.Open();
        }

        // Loads the ROM too, because a debug target needs the core's hardware to already exist.
        public static CoreBundle Load(string romPath, bool headless = true, CheatRegistry? cheats = null,
            Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? venusFrameTimings = null, string? engine = null, Func<EmuSen.Serenity.Slang.SlangLend?>? lender = null)
        {
            ICore core = Loaded(romPath, headless, engine, lender);
            return Bundle(core, cheats, venusFrameTimings) with { Notice = EngineNotice(romPath, engine, core) };
        }

        // The core made and the file loaded into it, Load without its wiring, for a caller that loads off its own thread.
        // lender is asked for the device to lend a core whose picture can lie on it - see EmuSen_DevicePicture.md §17.
        public static ICore Loaded(string romPath, bool headless = true, string? engine = null, Func<EmuSen.Serenity.Slang.SlangLend?>? lender = null)
        {
            ICore core = Create(romPath, headless, engine);
            if (core is IDevicePicture picture) picture.DeviceLender = lender;
            LoadInto(core, romPath);
            return core;
        }

        // Whether the file runs on a v1 engine, whose load writes nothing to the console and may run on any thread; false for a C# core or a file that opens nowhere - see Diamond_Players.md §12.8.3.
        public static bool RunsOnV1Engine(string romPath, string? engine)
        {
            try { return V1Library(romPath, engine) is not null; }
            catch (NotSupportedException) { return false; }
        }

        // A library file into a core: a v1 engine takes the file and its set itself; a C# core takes an archive's image unpacked, named for the archive - see EmuSen_CoreAPI_Extensions.md §10.4.
        public static void LoadInto(ICore core, string romPath) =>
            core.LoadRom(core is Native.CoreEngine || !Galaxia.Media.ArchiveImages.IsArchive(romPath) ? romPath : Galaxia.Media.ArchiveImages.Unpack(romPath, CoreCatalog.IsImageExtension));

        // The engine graphics.json stores for the ROM's console, so every frontend makes the one decision Mistress's row records; null where nothing is stored - see EmuSen_Settings_Reference.md §4.44.
        public static string? ConfiguredEngine(string romPath) =>
            CoreCatalog.ConsoleForRom(romPath) is { } console ? CoreCatalog.EngineChosen(console, GraphicsConfig.Load().Value(console, CoreCatalog.EngineKey)) : null;

        // Why the engine asked for is not running, or null when it is - see EmuSen_Settings_Reference.md §4.44.
        public static string? EngineNotice(string romPath, string? engine, ICore core)
        {
            if (engine is null || CoreCatalog.EngineFor(CoreCatalog.ConsoleForRom(romPath) ?? "") is not { } choice) return null;
            if (choice.Choices?.Contains(engine) != true) return $"No {choice.Label.ToLowerInvariant()} is named \"{engine}\"; {Running(core)} is running.";
            if (engine == CoreCatalog.MoonRtEngine && core is not MoonRtCore) return $"{CoreCatalog.MoonRtEngine} is not available ({MoonNative.Report}); {CoreCatalog.MoonEngine} is running.";
            if (!CoreCatalog.IsRegisteredEngine(engine) && Native.CoreDiscovery.ByEngineName(engine) is { } v1 && (core is not Native.CoreEngine running || running.Info.Id != v1.Info.Id))
                return $"{engine} is not available ({v1.Report}); {Running(core)} is running.";
            return null;
        }

        // The catalog's name for the engine that is running: a notice's, and the one a save state's record keeps.
        public static string Running(ICore core) => core switch
        {
            MarsRtCore => CoreCatalog.MarsRtEngine,
            MercuryRtCore => CoreCatalog.MercuryRtEngine,
            MoonRtCore => CoreCatalog.MoonRtEngine,
            MoonCore => CoreCatalog.MoonEngine,
            VenusCore => CoreCatalog.VenusEngine,
            Native.CoreEngine v1 => v1.Info.DisplayName,
            _ => core.CoreName,
        };

        // Split out so a caller that loaded the core itself can still get the rest of the wiring.
        public static CoreBundle Bundle(ICore core, CheatRegistry? cheats = null,
            Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? venusFrameTimings = null)
        {
            // A core owning its registry takes the caller's, so both sides edit one object - see EmuSen_Cheats.md §6.
            if (cheats is not null && core is ICheatRegistryHost host) host.Cheats = cheats;

            switch (core)
            {
                case VenusCore venus:
                    return new CoreBundle(
                        venus,
                        new SnesDebugTarget(venus.Cpu!, venus.Bus!, venus.Renderer!,
                            venusFrameTimings ?? (() => (venus.LastFrameCpuSpc700Ms, venus.LastFramePpuMs, venus.LastFrameHdmaMs)),
                            cheats),
                        VenusCheatCodecs.ActionReplay(),
                        VenusCheatCodecs.GameGenie(),
                        new VenusCpuTraceSwitch());

                case MoonCore moon:
                    return new CoreBundle(
                        moon,
                        new MoonDebugTarget(moon),
                        Crystal.Systems.Nes.NesSystem.Raw(),
                        Crystal.Systems.Nes.NesSystem.GameGenie(),
                        null);

                // The NES pack's codecs, and MoonRT's own target, reading its state by its listing - see Moon_Native.md §9.4.
                case MoonRtCore moonRt:
                    return new CoreBundle(
                        moonRt,
                        moonRt.CreateDebugTarget(),
                        Crystal.Systems.Nes.NesSystem.Raw(),
                        Crystal.Systems.Nes.NesSystem.GameGenie(),
                        null);

                // The Game Boy's codecs, and MercuryRT's target: its state through the layout listing for what it shows, the core's hooks for what halts - see Mercury_Native.md §15.2.
                case MercuryRtCore mercuryRt:
                    return new CoreBundle(
                        mercuryRt,
                        mercuryRt.CreateDebugTarget(),
                        GameBoyCheatCodecs.GameShark(),
                        GameBoyCheatCodecs.GameGenie(),
                        null);

                // The GameShark pokes, and no N64 format patches ROM; the target's hooks are tables in the Rust loop - see Mars_Cheats.md §1, Mars_Native.md §6.5.
                case MarsRtCore marsRt:
                    return new CoreBundle(marsRt, new MarsRtDebugTarget(marsRt, cheats), MarsCheatCodecs.GameShark(), null, null);

                // Any v1 engine: the generic target, and the codecs of its system's pack - see EmuSen_CoreAPI.md §19.
                case Native.CoreEngine v1:
                    var (auto, explicitCodec) = SystemCodecs(v1);
                    return new CoreBundle(v1, v1.CreateDebugTarget(), auto, explicitCodec, null);

                default:
                    throw new NotSupportedException($"No debug target is registered for {core.GetType().Name}.");
            }
        }

        // A v1 engine's codecs come from its system's pack, never from the engine; none for a system without one.
        private static (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) SystemCodecs(Native.CoreEngine engine) =>
            Crystal.Systems.SystemPacks.For(engine.IsRomLoaded ? engine.Machine.Info.System : engine.Info.Systems.FirstOrDefault()?.Id) is { } pack
                ? (pack.AutoDetectCodec?.Invoke(), pack.ExplicitCodec?.Invoke())
                : (null, null);

        // Answered without loading, so a caller can resolve a missing chip first - see EmuSen_Firmware.md §3.
        public static ICore ForFirmwareProbe(string romPath, string? engine = null) => Create(romPath, headless: true, engine);

        // What a cheat window opened before any ROM is parses with - see EmuSen_Multicore.md §4.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) DefaultCheatCodecs => VenusCheatCodecs.Pair();

        // The codecs for a console picked in the UI rather than loaded from a ROM - see EmuSen_Multicore.md §10.
        public static (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) CheatCodecsFor(string? coreName)
        {
            var core = CoreCatalog.ByAnyName(coreName);

            // No console chosen means no console to be wrong about, so the historical pair stands in.
            if (core is null) return DefaultCheatCodecs;

            // Dispatched on extension for the same reason Create is - no new public surface on the catalog.
            if (core.SupportsExtension(".sfc")) return VenusCheatCodecs.Pair();
            if (core.SupportsExtension(".nes")) return Crystal.Systems.Nes.NesSystem.Pair();
            if (core.SupportsExtension(".gb")) return GameBoyCheatCodecs.Pair();
            if (core.SupportsExtension(".z64")) return MarsCheatCodecs.Pair();
            // A console discovery added has its system pack's codecs, as a game of it running does.
            if (CoreCatalog.IsDiscovered(core) && Crystal.Systems.SystemPacks.All.FirstOrDefault(p => p.Entry.Console == core.Console) is { } pack)
                return (pack.AutoDetectCodec?.Invoke(), pack.ExplicitCodec?.Invoke());

            return (null, null);
        }

        private static string Extension(string romPath) => CoreCatalog.ImageExtension(romPath);
    }
}
