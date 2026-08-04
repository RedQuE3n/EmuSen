using System;
using System.Collections.Generic;
using System.Text.Json;
using Avalonia.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;

namespace EmuSen.Mistress.Input
{
    // One ControllerKeyMap per console, because a pad that has X/Y/L/R and one
    // that does not should not fight over the same keys - see EmuSen_Input.md §5.1.
    public sealed class ControllerKeyBindings
    {
        private readonly Dictionary<string, ControllerKeyMap> _byConsole = new(StringComparer.OrdinalIgnoreCase);

        // Consoles this build knows about are seeded eagerly; anything else is created on demand.
        public ControllerKeyBindings(IEnumerable<string> consoles)
        {
            foreach (string console in consoles) _byConsole[console] = new ControllerKeyMap();
        }

        public IReadOnlyDictionary<string, ControllerKeyMap> ByConsole => _byConsole;

        public ControllerKeyMap For(string console)
        {
            if (!_byConsole.TryGetValue(console, out ControllerKeyMap? map))
            {
                _byConsole[console] = map = new ControllerKeyMap();
            }
            return map;
        }

        public void ResetToDefaults()
        {
            foreach (ControllerKeyMap map in _byConsole.Values) map.ResetToDefaults();
        }

        private static readonly ConfigFile<Dictionary<string, Dictionary<PadButton, Key>>> File = new("keybindings.json");

        // Only ever asks "is this the flat shape or the per-console one" - JsonElement
        // accepts any value, so a bad enum still reaches the typed load below and is
        // reported there exactly once - see EmuSen_Config_Reference.md §3.6.
        private static readonly ConfigFile<Dictionary<string, JsonElement>> ShapeProbe = new("keybindings.json");

        private static readonly ConfigFile<Dictionary<PadButton, Key>> LegacyFile = new("keybindings.json");

        public bool Save()
        {
            var payload = new Dictionary<string, Dictionary<PadButton, Key>>(StringComparer.OrdinalIgnoreCase);
            foreach (var kv in _byConsole) payload[kv.Key] = kv.Value.ButtonToKey;
            return File.Save(payload);
        }

        public static ControllerKeyBindings Load(IEnumerable<string> consoles)
        {
            var bindings = new ControllerKeyBindings(consoles);
            if (ShapeProbe.Load() is not { Count: > 0 } probe) return bindings;

            // A flat file predates per-console bindings, so every console inherits it - that is what it used to mean.
            if (IsLegacyFlat(probe))
            {
                if (LegacyFile.Load() is { Count: > 0 } shared)
                {
                    foreach (ControllerKeyMap map in bindings._byConsole.Values)
                    {
                        map.Replace(new Dictionary<PadButton, Key>(shared));
                    }
                }
                return bindings;
            }

            if (File.Load() is not { } loaded) return bindings;

            foreach (var kv in loaded)
            {
                if (kv.Value is { Count: > 0 }) bindings.For(kv.Key).Replace(kv.Value);
            }
            return bindings;
        }

        // The values are keys in the old shape and objects in the new one.
        private static bool IsLegacyFlat(Dictionary<string, JsonElement> probe)
        {
            foreach (var kv in probe) return kv.Value.ValueKind != JsonValueKind.Object;
            return false;
        }
    }
}
