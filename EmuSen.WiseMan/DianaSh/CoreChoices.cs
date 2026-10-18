using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Systems;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.WiseMan.DianaSh
{
    // The catalog's registry as the plain choices `core` takes, as the frontend API's systems give them; the two are held equal by HotaruOnTheApiTests.
    public static class CoreChoices
    {
        public static IReadOnlyDictionary<string, CoreChoice> Of(IReadOnlyDictionary<string, CoreDescriptor> registry) =>
            registry.ToDictionary(kv => kv.Key, kv => new CoreChoice(kv.Value.DisplayName, kv.Value.Console, kv.Value.Extensions), StringComparer.OrdinalIgnoreCase);
    }
}
