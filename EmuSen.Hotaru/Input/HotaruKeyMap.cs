using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Cores;
using EmuSen.Galaxia.Input;

namespace EmuSen.Hotaru.Input
{
    // Keyboard -> PadButton mapping for GameWindow's KeyDown/KeyUp
    // handlers, replacing EmuSen/Settings/InputBindings.cs's direct
    // Raylib.IsKeyDown polling now that Hotaru no longer owns a Raylib
    // window to poll every frame - Avalonia delivers key state as
    // KeyDown/KeyUp events instead (see GameWindow.axaml.cs). Bindings
    // match InputBindings.cs's own keyboard scheme exactly (see
    // EmuSen.WiseMan/Input/HotaruKeyMapTests.cs's table-equality
    // assertion against it) - no rebind/persistence support, unlike
    // EmuSen.Mistress's own ControllerKeyMap, since Hotaru has no
    // settings UI to drive one.
    public static class HotaruKeyMap
    {
        public static readonly IReadOnlyDictionary<PadButton, Key> ButtonToKey = new Dictionary<PadButton, Key>
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

        private static readonly IReadOnlyDictionary<Key, PadButton> _keyToButton = BuildReverseLookup();

        private static Dictionary<Key, PadButton> BuildReverseLookup()
        {
            var lookup = new Dictionary<Key, PadButton>();
            foreach (var kv in ButtonToKey) lookup[kv.Value] = kv.Key;
            return lookup;
        }

        public static bool TryGetButton(Key key, out PadButton button) => _keyToButton.TryGetValue(key, out button);
    }
}
