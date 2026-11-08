using System;
using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Endymion.Input;
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

        // keybindings.json is Endymion's in the platform library, its one reader and writer - see EmuSen_Input.md §5.2.
        public bool Save()
        {
            var payload = new Dictionary<string, Dictionary<PadControl, Key>>();
            foreach (var kv in _byConsole) payload[kv.Key] = kv.Value.ButtonToKey;
            return BindingFiles.Save(BindingFile.Keys, payload);
        }

        public static ControllerKeyBindings Load(IEnumerable<string> consoles)
        {
            var bindings = new ControllerKeyBindings(consoles);
            foreach (var (console, map) in BindingFiles.Load<Dictionary<string, Dictionary<PadControl, Key>>>(BindingFile.Keys, bindings._byConsole.Keys))
                bindings.For(console).Replace(map);
            return bindings;
        }
    }
}
