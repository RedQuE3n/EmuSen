using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;
using EmuSen.Endymion.Input;

namespace EmuSen.Mistress.Input
{
    // The rebindable, persisted half of the keyboard mapping - Hotaru's HotaruKeyMap is the fixed one. See EmuSen_Input.md §5.1.
    public class ControllerKeyMap
    {
        public Dictionary<PadButton, Key> ButtonToKey { get; private set; } = DefaultPadKeyMap.Bindings();

        private Dictionary<Key, PadButton> _keyToButton = new();

        public ControllerKeyMap()
        {
            RebuildReverseLookup();
        }

        private void RebuildReverseLookup() => _keyToButton = DefaultPadKeyMap.Reverse(ButtonToKey);

        public bool TryGetButton(Key key, out PadButton button) => _keyToButton.TryGetValue(key, out button);

        // Rebinds `button` to `newKey`. If newKey was already assigned to a
        // different button, that button is left unbound rather than allowing
        // two console buttons to share one key.
        public void Rebind(PadButton button, Key newKey)
        {
            if (_keyToButton.TryGetValue(newKey, out PadButton existingOwner) && existingOwner != button)
            {
                ButtonToKey.Remove(existingOwner);
            }

            ButtonToKey[button] = newKey;
            RebuildReverseLookup();
        }

        public void Unbind(PadButton button)
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
        public void Replace(Dictionary<PadButton, Key> bindings)
        {
            ButtonToKey = bindings;
            RebuildReverseLookup();
        }
    }
}
