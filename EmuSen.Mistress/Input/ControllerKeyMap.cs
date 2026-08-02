using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Galaxia;

namespace EmuSen.Mistress.Input
{
    // Keyboard -> SnesButton mapping for this frontend specifically. This is
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
        public Dictionary<SnesButton, Key> ButtonToKey { get; private set; } = DefaultBindings();

        private Dictionary<Key, SnesButton> _keyToButton = new();

        public ControllerKeyMap()
        {
            RebuildReverseLookup();
        }

        private static Dictionary<SnesButton, Key> DefaultBindings() => new()
        {
            [SnesButton.Up] = Key.Up,
            [SnesButton.Down] = Key.Down,
            [SnesButton.Left] = Key.Left,
            [SnesButton.Right] = Key.Right,
            [SnesButton.B] = Key.Z,
            [SnesButton.A] = Key.X,
            [SnesButton.Y] = Key.A,
            [SnesButton.X] = Key.S,
            [SnesButton.L] = Key.Q,
            [SnesButton.R] = Key.W,
            [SnesButton.Start] = Key.Enter,
            [SnesButton.Select] = Key.RightShift,
        };

        private void RebuildReverseLookup()
        {
            _keyToButton = new Dictionary<Key, SnesButton>();
            foreach (var kv in ButtonToKey)
            {
                _keyToButton[kv.Value] = kv.Key;
            }
        }

        public bool TryGetButton(Key key, out SnesButton button) => _keyToButton.TryGetValue(key, out button);

        // Rebinds `button` to `newKey`. If newKey was already assigned to a
        // different button, that button is left unbound rather than allowing
        // two SNES buttons to share one key.
        public void Rebind(SnesButton button, Key newKey)
        {
            if (_keyToButton.TryGetValue(newKey, out SnesButton existingOwner) && existingOwner != button)
            {
                ButtonToKey.Remove(existingOwner);
            }

            ButtonToKey[button] = newKey;
            RebuildReverseLookup();
        }

        public void Unbind(SnesButton button)
        {
            ButtonToKey.Remove(button);
            RebuildReverseLookup();
        }

        public void ResetToDefaults()
        {
            ButtonToKey = DefaultBindings();
            RebuildReverseLookup();
        }

        private static readonly ConfigFile<Dictionary<SnesButton, Key>> File = new("keybindings.json");

        public void Save() => File.Save(ButtonToKey);

        public static ControllerKeyMap Load()
        {
            var bindings = new ControllerKeyMap();
            if (File.Load() is { Count: > 0 } loaded)
            {
                bindings.ButtonToKey = loaded;
                bindings.RebuildReverseLookup();
            }
            return bindings;
        }
    }
}
