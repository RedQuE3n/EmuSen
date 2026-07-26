using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Avalonia.Input;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Mistress9.Input
{
    // Keyboard -> SnesButton mapping for this frontend specifically. This is
    // deliberately separate from EmuSen/Settings/InputBindings.cs, which reads
    // Raylib key state directly and only works with the console/Raylib build's
    // own window - this class uses Avalonia's Key enum and gets fed from the
    // MainWindow's KeyDown/KeyUp events instead (see MainWindow.axaml.cs).
    //
    // Defaults match InputBindings.cs's keyboard scheme for consistency
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

        public void ResetToDefaults()
        {
            ButtonToKey = DefaultBindings();
            RebuildReverseLookup();
        }

        private static string ConfigPath => Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "EmuSen", "keybindings.json");

        public void Save()
        {
            try
            {
                string dir = Path.GetDirectoryName(ConfigPath)!;
                Directory.CreateDirectory(dir);
                string json = JsonSerializer.Serialize(ButtonToKey, new JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(ConfigPath, json);
            }
            catch
            {
                // Best-effort - a failed save shouldn't crash a rebind attempt,
                // it just means the change won't survive a restart.
            }
        }

        public static ControllerKeyMap Load()
        {
            var bindings = new ControllerKeyMap();
            try
            {
                if (File.Exists(ConfigPath))
                {
                    string json = File.ReadAllText(ConfigPath);
                    var loaded = JsonSerializer.Deserialize<Dictionary<SnesButton, Key>>(json);
                    if (loaded is { Count: > 0 })
                    {
                        bindings.ButtonToKey = loaded;
                        bindings.RebuildReverseLookup();
                    }
                }
            }
            catch
            {
                // Corrupt/unreadable config - fall back to defaults rather than crash.
            }
            return bindings;
        }
    }
}
