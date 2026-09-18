using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;
using EmuSen.Endymion.Input;

namespace EmuSen.Mistress.Input
{
    // The rebindable, persisted half of the keyboard mapping - Hotaru's HotaruKeyMap is the fixed one. See EmuSen_Input.md §5.1.
    public class ControllerKeyMap
    {
        public Dictionary<PadControl, Key> ButtonToKey { get; private set; } = DefaultPadKeyMap.Bindings();

        private Dictionary<Key, PadControl> _keyToButton = new();

        public ControllerKeyMap()
        {
            RebuildReverseLookup();
        }

        private void RebuildReverseLookup() => _keyToButton = DefaultPadKeyMap.Reverse(ButtonToKey);

        public bool TryGetControl(Key key, out PadControl control) => _keyToButton.TryGetValue(key, out control);

        // Rebinds `button` to `newKey`. If newKey was already assigned to a
        // different button, that button is left unbound rather than allowing
        // two console buttons to share one key.
        public void Rebind(PadControl button, Key newKey)
        {
            if (_keyToButton.TryGetValue(newKey, out PadControl existingOwner) && existingOwner != button)
            {
                ButtonToKey.Remove(existingOwner);
            }

            ButtonToKey[button] = newKey;
            RebuildReverseLookup();
        }

        public void Unbind(PadControl button)
        {
            ButtonToKey.Remove(button);
            RebuildReverseLookup();
        }

        public void ResetToDefaults()
        {
            ButtonToKey = DefaultPadKeyMap.Bindings();
            RebuildReverseLookup();
        }

        // Persistence belongs to ControllerKeyBindings, which owns one of these per console.
        public void Replace(Dictionary<PadControl, Key> bindings)
        {
            ButtonToKey = bindings;
            RebuildReverseLookup();
        }

        // A file written before a control existed cannot mention it, so it takes its default key if that key is free - see EmuSen_Config_Reference.md §3.6.
        public void AddDefaultsForNewControls()
        {
            foreach (KeyValuePair<PadControl, Key> binding in DefaultPadKeyMap.Bindings())
            {
                if (binding.Key < DefaultPadKeyMap.FirstAddedControl || ButtonToKey.ContainsKey(binding.Key)) continue;
                if (_keyToButton.ContainsKey(binding.Value)) continue;

                ButtonToKey[binding.Key] = binding.Value;
                RebuildReverseLookup();
            }
        }
    }
}
