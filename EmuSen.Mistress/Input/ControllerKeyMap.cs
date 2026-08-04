using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;

namespace EmuSen.Mistress.Input
{
    // Keyboard -> PadButton mapping for this frontend specifically. This is
    // deliberately separate from EmuSen.Hotaru/Input/HotaruKeyMap.cs, which
    // has no rebind/persistence support and gets fed from GameWindow's own
    // KeyDown/KeyUp events instead (see that project's GameWindow.axaml.cs) -
    // this class supports rebinding/saving, which Hotaru's console-first
    // frontend has no settings UI to drive.
    //
    // Defaults match HotaruKeyMap's keyboard scheme for consistency
    // between the two frontends, even though the two Key enums (Raylib_cs vs
    // Avalonia.Input) aren't the same type.
    public class ControllerKeyMap
    {
        public Dictionary<PadButton, Key> ButtonToKey { get; private set; } = DefaultBindings();

        private Dictionary<Key, PadButton> _keyToButton = new();

        public ControllerKeyMap()
        {
            RebuildReverseLookup();
        }

        private static Dictionary<PadButton, Key> DefaultBindings() => new()
        {
            [PadButton.Up] = Key.Up,
            [PadButton.Down] = Key.Down,
            [PadButton.Left] = Key.Left,
            [PadButton.Right] = Key.Right,
            [PadButton.B] = Key.Z,
            [PadButton.A] = Key.X,
            [PadButton.Y] = Key.A,
            [PadButton.X] = Key.S,
            [PadButton.L] = Key.Q,
            [PadButton.R] = Key.W,
            [PadButton.Start] = Key.Enter,
            [PadButton.Select] = Key.RightShift,
        };

        private void RebuildReverseLookup()
        {
            _keyToButton = new Dictionary<Key, PadButton>();
            foreach (var kv in ButtonToKey)
            {
                _keyToButton[kv.Value] = kv.Key;
            }
        }

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
            ButtonToKey = DefaultBindings();
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
