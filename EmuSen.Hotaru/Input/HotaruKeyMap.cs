using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Cores.Nintendo.Venus.Controllers;

namespace EmuSen.Hotaru.Input
{
    // Keyboard -> SnesButton mapping for GameWindow's KeyDown/KeyUp
    // handlers, replacing EmuSen/Settings/InputBindings.cs's direct
    // Raylib.IsKeyDown polling now that Hotaru no longer owns a Raylib
    // window to poll every frame - Avalonia delivers key state as
    // KeyDown/KeyUp events instead (see GameWindow.axaml.cs). Bindings
    // match InputBindings.cs's own keyboard scheme exactly (see
    // EmuSen.WiseMan/Input/HotaruKeyMapTests.cs's table-equality
    // assertion against it) - no rebind/persistence support, unlike
    // EmuSen.Mistress9's own ControllerKeyMap, since Hotaru has no
    // settings UI to drive one.
    public static class HotaruKeyMap
    {
        public static readonly IReadOnlyDictionary<SnesButton, Key> ButtonToKey = new Dictionary<SnesButton, Key>
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

        private static readonly IReadOnlyDictionary<Key, SnesButton> _keyToButton = BuildReverseLookup();

        private static Dictionary<Key, SnesButton> BuildReverseLookup()
        {
            var lookup = new Dictionary<Key, SnesButton>();
            foreach (var kv in ButtonToKey) lookup[kv.Value] = kv.Key;
            return lookup;
        }

        public static bool TryGetButton(Key key, out SnesButton button) => _keyToButton.TryGetValue(key, out button);
    }
}
