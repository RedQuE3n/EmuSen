using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.Hotaru
{
    // The cores `core <name>` knows and the cheat systems they play, read from the frontend API's systems, where the runtime keeps the vocabulary - see EmuSen_Backend.md §14.11.
    public sealed record HotaruCores(IReadOnlyDictionary<string, CoreChoice> Registry, IReadOnlyCollection<string> CheatSystems)
    {
        public static HotaruCores Of(CrystalRuntime runtime)
        {
            var registry = new Dictionary<string, CoreChoice>(StringComparer.OrdinalIgnoreCase);
            var cheats = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            using JsonDocument systems = JsonDocument.Parse(runtime.Systems());
            foreach (JsonElement system in systems.RootElement.EnumerateArray())
            {
                if (!system.TryGetProperty("core", out JsonElement core) || core.ValueKind != JsonValueKind.Object) continue;
                var choice = new CoreChoice(core.GetProperty("display_name").GetString()!, core.GetProperty("console").GetString()!,
                    core.GetProperty("extensions").EnumerateArray().Select(e => e.GetString()!).ToArray());
                foreach (JsonElement name in core.GetProperty("names").EnumerateArray()) registry.TryAdd(name.GetString()!, choice);
                if (system.TryGetProperty("cheat_systems", out JsonElement named))
                    foreach (JsonElement cheat in named.EnumerateArray()) cheats.Add(cheat.GetString()!);
            }
            return new HotaruCores(registry, cheats);
        }

        // One name per core for the welcome banner, as the registry's display names.
        public IEnumerable<string> DisplayNames => Registry.Values.Select(c => c.DisplayName).Distinct();
    }
}
