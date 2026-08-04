using System;
using System.IO;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Cheats;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores
{
    // Everything a frontend needs for one loaded ROM, so it never names a core itself.
    public sealed record CoreBundle(
        ICore Core,
        IDebugTarget DebugTarget,
        ICheatCodeCodec? CheatAutoDetectCodec,
        ICheatCodeCodec? CheatExplicitCodec,
        ICpuTraceSwitch? CpuTraceSwitch);

    // The one place a ROM path turns into a running core - see EmuSen_Multicore.md §2.
    public static class CoreFactory
    {
        public static bool IsSupported(string romPath) => CoreCatalog.IsRomExtension(Extension(romPath));

        // <headless> only means anything to a core that owns a window-ish resource; Moon ignores it.
        public static ICore Create(string romPath, bool headless = true) => Extension(romPath) switch
        {
            ".smc" or ".sfc" => new VenusCore(headless),
            ".nes" => new MoonCore(),
            var other => throw new NotSupportedException(
                $"No core in this build handles '{other}' - see CoreCatalog for what is registered."),
        };

        // Loads the ROM too, because a debug target needs the core's hardware to already exist.
        public static CoreBundle Load(string romPath, bool headless = true, CheatRegistry? cheats = null,
            Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? venusFrameTimings = null)
        {
            ICore core = Create(romPath, headless);
            core.LoadRom(romPath);
            return Bundle(core, cheats, venusFrameTimings);
        }

        // Split out so a caller that loaded the core itself can still get the rest of the wiring.
        public static CoreBundle Bundle(ICore core, CheatRegistry? cheats = null,
            Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? venusFrameTimings = null)
        {
            switch (core)
            {
                case VenusCore venus:
                    return new CoreBundle(
                        venus,
                        new SnesDebugTarget(venus.Cpu!, venus.Bus!, venus.Renderer!,
                            venusFrameTimings ?? (() => (venus.LastFrameCpuSpc700Ms, venus.LastFramePpuMs, venus.LastFrameHdmaMs)),
                            cheats),
                        new ActionReplayCheatCodec(),
                        new GameGenieCheatCodec(),
                        new VenusCpuTraceSwitch());

                case MoonCore moon:
                    // No Game Genie/Action Replay codec for this core yet - see EmuSen_Multicore.md §4.
                    return new CoreBundle(moon, new MoonDebugTarget(moon), null, null, null);

                default:
                    throw new NotSupportedException($"No debug target is registered for {core.GetType().Name}.");
            }
        }

        // Answered without loading, so a caller can resolve a missing chip first - see EmuSen_Firmware.md §3.
        public static ICore ForFirmwareProbe(string romPath) => Create(romPath, headless: true);

        // What a cheat window opened before any ROM is parses with - see EmuSen_Multicore.md §4.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) DefaultCheatCodecs =>
            (new ActionReplayCheatCodec(), new GameGenieCheatCodec());

        // The codecs for a console picked in the UI rather than loaded from a ROM - see EmuSen_Multicore.md §10.
        public static (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) CheatCodecsFor(string? coreDisplayName)
        {
            var core = CoreCatalog.ByDisplayName(coreDisplayName);

            // No console chosen means no console to be wrong about, so the historical pair stands in.
            if (core is null) return DefaultCheatCodecs;

            // Dispatched on extension for the same reason Create is - no new public surface on the catalog.
            if (core.SupportsExtension(".sfc")) return (new ActionReplayCheatCodec(), new GameGenieCheatCodec());

            return (null, null);
        }

        private static string Extension(string romPath) => Path.GetExtension(romPath).ToLowerInvariant();
    }
}
