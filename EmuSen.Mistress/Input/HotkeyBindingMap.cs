using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using Avalonia.Input;

namespace EmuSen.Mistress.Input
{
    // Emulator actions, not pad buttons the game reads - see EmuSen_Settings_Reference.md §4.3.
    public enum HotkeyAction
    {
        FastForward,
        Rewind,
        SaveState,
        LoadState,
        TogglePause,
        ToggleFullscreen,
    }

    // Keyboard -> HotkeyAction, shaped like ControllerKeyMap - see EmuSen_Settings_Reference.md §4.3.
    public class HotkeyBindingMap
    {
        public Dictionary<HotkeyAction, Key> ActionToKey { get; private set; } = DefaultBindings();

        private Dictionary<Key, HotkeyAction> _keyToAction = new();

        public HotkeyBindingMap()
        {
            RebuildReverseLookup();
        }

        // Tab/Back match the old hardcoded pair - see EmuSen_Settings_Reference.md §4.3.
        private static Dictionary<HotkeyAction, Key> DefaultBindings() => new()
        {
            [HotkeyAction.FastForward] = Key.Tab,
            [HotkeyAction.Rewind] = Key.Back,
            [HotkeyAction.SaveState] = Key.F5,
            [HotkeyAction.LoadState] = Key.F8,
            [HotkeyAction.TogglePause] = Key.P,
            [HotkeyAction.ToggleFullscreen] = Key.F11,
        };

        public static string DisplayName(HotkeyAction action) => action switch
        {
            HotkeyAction.FastForward => "Fast Forward",
            HotkeyAction.Rewind => "Rewind",
            HotkeyAction.SaveState => "Save State",
            HotkeyAction.LoadState => "Load State",
            HotkeyAction.TogglePause => "Pause / Resume",
            HotkeyAction.ToggleFullscreen => "Fullscreen",
            _ => action.ToString(),
        };

        // Held actions track the key; the rest fire once - see EmuSen_Settings_Reference.md §4.3.
        public static bool IsHeld(HotkeyAction action) =>
            action is HotkeyAction.FastForward or HotkeyAction.Rewind;

        private void RebuildReverseLookup()
        {
            _keyToAction = new Dictionary<Key, HotkeyAction>();
            foreach (var kv in ActionToKey)
            {
                _keyToAction[kv.Value] = kv.Key;
            }
        }

        public bool TryGetAction(Key key, out HotkeyAction action) => _keyToAction.TryGetValue(key, out action);

        // One owner per key, as in ControllerKeyMap.Rebind.
        public void Rebind(HotkeyAction action, Key newKey)
        {
            if (_keyToAction.TryGetValue(newKey, out HotkeyAction existingOwner) && existingOwner != action)
            {
                ActionToKey.Remove(existingOwner);
            }

            ActionToKey[action] = newKey;
            RebuildReverseLookup();
        }

        public void Unbind(HotkeyAction action)
        {
            ActionToKey.Remove(action);
            RebuildReverseLookup();
        }

        public void ResetToDefaults()
        {
            ActionToKey = DefaultBindings();
            RebuildReverseLookup();
        }

        private static string ConfigPath => Nehellania.Settings.SettingsPaths.For("hotkeybindings.json");

        public void Save()
        {
            try
            {
                Directory.CreateDirectory(Path.GetDirectoryName(ConfigPath)!);
                File.WriteAllText(ConfigPath, JsonSerializer.Serialize(ActionToKey, new JsonSerializerOptions { WriteIndented = true }));
            }
            catch
            {
                // Best-effort - a failed save shouldn't crash a rebind.
            }
        }

        public static HotkeyBindingMap Load()
        {
            var bindings = new HotkeyBindingMap();
            try
            {
                if (File.Exists(ConfigPath))
                {
                    var loaded = JsonSerializer.Deserialize<Dictionary<HotkeyAction, Key>>(File.ReadAllText(ConfigPath));
                    if (loaded is { Count: > 0 })
                    {
                        bindings.ActionToKey = loaded;
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
