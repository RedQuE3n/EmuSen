using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Cores
{
    // How many controllers a running core, or a console with no game loaded, reads - see EmuSen_Input.md §8.1.
    public static class ControllerPorts
    {
        public static int Of(ICore? core) => core switch
        {
            null => 0,
            Native.CoreEngine engine => OfEngine(engine),
            _ => Math.Max(1, core.ControllerPorts),
        };

        // A v1 engine's machine info names each port's controller, else its system's controllers name their ports - see EmuSen_CoreAPI.md §6.3, §6.4.
        private static int OfEngine(Native.CoreEngine engine)
        {
            if (engine.IsRomLoaded && engine.Machine.Info.Ports.Where(p => p.Controller is not null).Select(p => (int)p.Port + 1).DefaultIfEmpty(0).Max() is > 0 and var named) return named;
            int listed = engine.Info.Systems.SelectMany(s => s.Controllers).SelectMany(c => c.Ports).Select(p => (int)p + 1).DefaultIfEmpty(0).Max();
            return Math.Max(1, listed);
        }

        // Each player's controller id in the running game, by port; null where the core does not name one.
        public static IReadOnlyList<string?> ControllersOf(ICore? core)
        {
            int ports = Of(core);
            if (core is not Native.CoreEngine { IsRomLoaded: true } engine) return new string?[ports];
            var named = engine.Machine.Info.Ports.Where(p => p.Controller is not null).ToDictionary(p => (int)p.Port, p => p.Controller);
            return Enumerable.Range(0, ports).Select(p => named.GetValueOrDefault(p)).ToArray();
        }

        // The players a console's ports hold as its settings stand, where its pack counts them, else the console's most.
        public static int ForConsole(string? console, Func<string, string?> stored) =>
            console is not null && CoreCatalog.PlayerControllersFor(console, stored) is { Count: > 0 } players ? players.Count : ForConsole(console);

        // The C# cores' own counts, which every engine of the console shares; a console this build does not know reads one.
        public static int ForConsole(string? console) => console?.ToUpperInvariant() switch
        {
            "NES" => Nintendo.Moon.MoonCore.Ports,
            "SNES" => Nintendo.Venus.VenusCore.Ports,
            "N64" => Nintendo.Mars.MarsCore.Ports,
            _ when console is not null && CoreCatalog.DiscoveredSystem(console) is { } system => Math.Max(1, system.Controllers.SelectMany(c => c.Ports).Select(p => (int)p + 1).DefaultIfEmpty(0).Max()),
            _ => 1,
        };
    }
}
