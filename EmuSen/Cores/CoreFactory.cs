using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cheats;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Cheats;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Cheats;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Models;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Cheats;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

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
        public static bool IsSupported(string romPath) => CoreCatalog.IsRomExtension(Extension(romPath)) || Native.CoreDiscovery.ForExtension(Extension(romPath)).Any();

        // <headless> only means anything to a core that owns a window-ish resource; <engine> is a CoreCatalog.EngineFor name, and null is the reference core, not the player's default - see EmuSen_Settings_Reference.md §4.44.
        public static ICore Create(string romPath, bool headless = true, string? engine = null) =>
            V1Library(Extension(romPath), engine) is { } v1 ? new Native.CoreEngine(v1) : Extension(romPath) switch
        {
            ".smc" or ".sfc" => new VenusCore(headless),
            ".nes" => engine == CoreCatalog.MoonRtEngine && MoonRtCore.Available ? new MoonRtCore() : new MoonCore(),
            ".gb" or ".gbc" => engine == CoreCatalog.MercuryRtEngine && MercuryRtCore.Available ? new MercuryRtCore() : new MercuryCore(),
            ".z64" or ".n64" or ".v64" => engine == CoreCatalog.MarsRtEngine && MarsRtCore.Available ? new MarsRtCore(expansionPak: true) : new MarsCore(expansionPak: true),
            var other => throw new NotSupportedException(
                $"No core in this build handles '{other}' - see CoreCatalog for what is registered."),
        };

        // The one generic branch for v1 engines: the one asked for by name, or the first for an extension no C# core claims; null when it is not loadable - see EmuSen_CoreAPI.md §19.
        private static Native.CoreLibrary? V1Library(string extension, string? engine)
        {
            var found = engine is not null ? Native.CoreDiscovery.ForExtension(extension).FirstOrDefault(c => c.EngineName == engine)
                : CoreCatalog.IsRomExtension(extension) ? null : Native.CoreDiscovery.ForExtension(extension).FirstOrDefault();
            return found is null || CoreCatalog.IsRegisteredEngine(found.EngineName) ? null : found.Open();
        }

        // Loads the ROM too, because a debug target needs the core's hardware to already exist.
        public static CoreBundle Load(string romPath, bool headless = true, CheatRegistry? cheats = null,
            Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? venusFrameTimings = null, string? engine = null)
        {
            ICore core = Create(romPath, headless, engine);
            core.LoadRom(romPath);
            return Bundle(core, cheats, venusFrameTimings) with { Notice = EngineNotice(romPath, engine, core) };
        }

        // The engine graphics.json stores for the ROM's console, so every frontend makes the one decision Mistress's row records; null where nothing is stored - see EmuSen_Settings_Reference.md §4.44.
        public static string? ConfiguredEngine(string romPath) =>
            CoreCatalog.ConsoleForRom(romPath) is { } console ? CoreCatalog.EngineChosen(console, GraphicsConfig.Load().Value(console, CoreCatalog.EngineKey)) : null;

        // Why the engine asked for is not running, or null when it is; an unavailable MarsRT says what MarsNative found - see Mars_Native.md §5.5.
        public static string? EngineNotice(string romPath, string? engine, ICore core)
        {
            if (engine is null || CoreCatalog.EngineFor(CoreCatalog.ConsoleForRom(romPath) ?? "") is not { } choice) return null;
            if (choice.Choices?.Contains(engine) != true) return $"No {choice.Label.ToLowerInvariant()} is named \"{engine}\"; {Running(core)} is running.";
            if (engine == CoreCatalog.MarsRtEngine && core is not MarsRtCore) return $"{CoreCatalog.MarsRtEngine} is not available ({MarsNative.Report}); {CoreCatalog.MarsEngine} is running.";
            if (engine == CoreCatalog.MercuryRtEngine && core is not MercuryRtCore) return $"{CoreCatalog.MercuryRtEngine} is not available ({MercuryNative.Report}); {CoreCatalog.MercuryEngine} is running.";
            if (engine == CoreCatalog.MoonRtEngine && core is not MoonRtCore) return $"{CoreCatalog.MoonRtEngine} is not available ({MoonNative.Report}); {CoreCatalog.MoonEngine} is running.";
            if (!CoreCatalog.IsRegisteredEngine(engine) && Native.CoreDiscovery.ByEngineName(engine) is { } v1 && (core is not Native.CoreEngine running || running.Info.Id != v1.Info.Id))
                return $"{engine} is not available ({v1.Report}); {Running(core)} is running.";
            return null;
        }

        // The catalog's name for the engine that is running: a notice's, and the one a save state's record keeps.
        public static string Running(ICore core) => core switch
        {
            MarsRtCore => CoreCatalog.MarsRtEngine,
            MarsCore => CoreCatalog.MarsEngine,
            MercuryRtCore => CoreCatalog.MercuryRtEngine,
            MercuryCore => CoreCatalog.MercuryEngine,
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
                        MoonCheatCodecs.Raw(),
                        MoonCheatCodecs.GameGenie(),
                        null);

                // The same codecs, and MoonRT's target: a mirror for what it shows; its hooks are stage 5's - see Moon_Native.md §8.3.
                case MoonRtCore moonRt:
                    return new CoreBundle(
                        moonRt,
                        moonRt.CreateDebugTarget(),
                        MoonCheatCodecs.Raw(),
                        MoonCheatCodecs.GameGenie(),
                        null);

                case MercuryCore mercury:
                    return new CoreBundle(
                        mercury,
                        new MercuryDebugTarget(mercury),
                        MercuryCheatCodecs.GameShark(),
                        MercuryCheatCodecs.GameGenie(),
                        null);

                // The same codecs, and MercuryRT's target: a mirror for what it shows, the core's hooks for what halts - see Mercury_Native.md §8.5.
                case MercuryRtCore mercuryRt:
                    return new CoreBundle(
                        mercuryRt,
                        mercuryRt.CreateDebugTarget(),
                        MercuryCheatCodecs.GameShark(),
                        MercuryCheatCodecs.GameGenie(),
                        null);

                // The GameShark pokes; no N64 format patches ROM, so the explicit slot stays empty - see Mars_Cheats.md §1.
                case MarsCore mars:
                    return new CoreBundle(mars, new MarsDebugTarget(mars, cheats), MarsCheatCodecs.GameShark(), null, null);

                // The same codec, and a target whose hooks are tables in the Rust loop - see Mars_Native.md §6.5.
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
            DianaOS.DianaOS.Sys.Systems.SystemPacks.For(engine.IsRomLoaded ? engine.Machine.Info.System : engine.Info.Systems.FirstOrDefault()?.Id) is { } pack
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
            if (core.SupportsExtension(".nes")) return MoonCheatCodecs.Pair();
            if (core.SupportsExtension(".gb")) return MercuryCheatCodecs.Pair();
            if (core.SupportsExtension(".z64")) return MarsCheatCodecs.Pair();

            return (null, null);
        }

        private static string Extension(string romPath) => Path.GetExtension(romPath).ToLowerInvariant();
    }
}
