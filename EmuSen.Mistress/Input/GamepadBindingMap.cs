using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Silk.NET.SDL;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Mistress.Input
{
    // Gamepad-button -> SnesButton mapping, same shape and persistence
    // approach as ControllerKeyMap (Input/ControllerKeyMap.cs) but for
    // GameControllerButton instead of Avalonia's Key. Kept as a separate
    // class/file rather than merged into ControllerKeyMap since keyboard and
    // gamepad are different device types with different rebind-capture flows
    // (KeyDown event vs polling - see InputSettingsWindow).
    //
    // Defaults match GamepadManager's previous hardcoded mapping: physical
    // button POSITION rather than label (bottom/right/left/top face buttons
    // -> B/A/Y/X), same reasoning as EmuSen.Hotaru's own HotaruKeyMap.cs.
    public class GamepadBindingMap
    {
        public Dictionary<SnesButton, GameControllerButton> ButtonToPad { get; private set; } = DefaultBindings();

        private Dictionary<GameControllerButton, SnesButton> _padToButton = new();

        public GamepadBindingMap()
        {
            RebuildReverseLookup();
        }

        private static Dictionary<SnesButton, GameControllerButton> DefaultBindings() => new()
        {
            [SnesButton.Up] = GameControllerButton.DpadUp,
            [SnesButton.Down] = GameControllerButton.DpadDown,
            [SnesButton.Left] = GameControllerButton.DpadLeft,
            [SnesButton.Right] = GameControllerButton.DpadRight,
            [SnesButton.B] = GameControllerButton.A,
            [SnesButton.A] = GameControllerButton.B,
            [SnesButton.Y] = GameControllerButton.X,
            [SnesButton.X] = GameControllerButton.Y,
            [SnesButton.L] = GameControllerButton.Leftshoulder,
            [SnesButton.R] = GameControllerButton.Rightshoulder,
            [SnesButton.Start] = GameControllerButton.Start,
            [SnesButton.Select] = GameControllerButton.Back,
        };

        private void RebuildReverseLookup()
        {
            _padToButton = new Dictionary<GameControllerButton, SnesButton>();
            foreach (var kv in ButtonToPad)
            {
                _padToButton[kv.Value] = kv.Key;
            }
        }

        public bool TryGetButton(GameControllerButton pad, out SnesButton button) => _padToButton.TryGetValue(pad, out button);

        // Rebinds `button` to `newPad`. If newPad was already assigned to a
        // different button, that button is left unbound rather than allowing
        // two SNES buttons to share one pad button.
        public void Rebind(SnesButton button, GameControllerButton newPad)
        {
            if (_padToButton.TryGetValue(newPad, out SnesButton existingOwner) && existingOwner != button)
            {
                ButtonToPad.Remove(existingOwner);
            }

            ButtonToPad[button] = newPad;
            RebuildReverseLookup();
        }

        public void Unbind(SnesButton button)
        {
            ButtonToPad.Remove(button);
            RebuildReverseLookup();
        }

        public void ResetToDefaults()
        {
            ButtonToPad = DefaultBindings();
            RebuildReverseLookup();
        }

        private static string ConfigPath => Settings.SettingsPaths.For("gamepadbindings.json");

        public void Save()
        {
            try
            {
                string dir = Path.GetDirectoryName(ConfigPath)!;
                Directory.CreateDirectory(dir);
                string json = JsonSerializer.Serialize(ButtonToPad, new JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(ConfigPath, json);
            }
            catch
            {
                // Best-effort - a failed save shouldn't crash a rebind attempt.
            }
        }

        public static GamepadBindingMap Load()
        {
            var bindings = new GamepadBindingMap();
            try
            {
                if (File.Exists(ConfigPath))
                {
                    string json = File.ReadAllText(ConfigPath);
                    var loaded = JsonSerializer.Deserialize<Dictionary<SnesButton, GameControllerButton>>(json);
                    if (loaded is { Count: > 0 })
                    {
                        bindings.ButtonToPad = loaded;
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
