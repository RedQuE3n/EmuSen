using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;
using EmuSen.Endymion.Input;

namespace EmuSen.Hotaru.Input
{
    // GameWindow's KeyDown/KeyUp mapping - no rebinding, since Hotaru has no settings UI to drive one. See EmuSen_Frontend_Driver.md §4.
    public static class HotaruKeyMap
    {
        public static readonly IReadOnlyDictionary<PadControl, Key> ButtonToKey = DefaultPadKeyMap.Bindings();

        private static readonly IReadOnlyDictionary<Key, PadControl> _keyToButton = DefaultPadKeyMap.Reverse(ButtonToKey);

        public static bool TryGetControl(Key key, out PadControl control) => _keyToButton.TryGetValue(key, out control);
    }
}
