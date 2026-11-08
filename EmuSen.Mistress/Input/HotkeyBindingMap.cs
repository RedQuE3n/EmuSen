using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Endymion.Input;

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
        ExitToLibrary,
        Screenshot,
        ToggleBigPicture,
        FrameAdvance,
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

        // Endymion's defaults, from the platform library, which reads hotkeybindings.json by them - see EmuSen_Settings_Reference.md §4.3.
        private static Dictionary<HotkeyAction, Key> DefaultBindings() => BindingFiles.Defaults<Dictionary<HotkeyAction, Key>>(BindingFile.Hotkeys);

        public static string DisplayName(HotkeyAction action) => action switch
        {
            HotkeyAction.FastForward => "Fast Forward",
            HotkeyAction.Rewind => "Rewind",
            HotkeyAction.SaveState => "Save State",
            HotkeyAction.LoadState => "Load State",
            HotkeyAction.TogglePause => "Pause / Resume",
            HotkeyAction.ToggleFullscreen => "Fullscreen",
            HotkeyAction.ExitToLibrary => "Exit to Library",
            HotkeyAction.Screenshot => "Screenshot",
            HotkeyAction.ToggleBigPicture => "Big Picture",
            HotkeyAction.FrameAdvance => "Frame Advance",
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

        // hotkeybindings.json is Endymion's in the platform library, its one reader and writer - see EmuSen_Input.md §5.2.
        public void Save() => BindingFiles.Save(BindingFile.Hotkeys, ActionToKey);

        public static HotkeyBindingMap Load()
        {
            var bindings = new HotkeyBindingMap();
            bindings.ActionToKey = BindingFiles.Load<Dictionary<HotkeyAction, Key>>(BindingFile.Hotkeys, []);
            bindings.RebuildReverseLookup();
            return bindings;
        }
    }
}
