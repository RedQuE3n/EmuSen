using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Input;

namespace EmuSen.Hotaru.Input
{
    // GameWindow's KeyDown/KeyUp mapping - no rebinding, since Hotaru has no settings UI to drive one. See EmuSen_Frontend_Driver.md §4.
    public static class HotaruKeyMap
    {
        public static readonly IReadOnlyDictionary<PadButton, Key> ButtonToKey = DefaultPadKeyMap.Bindings();

        private static readonly IReadOnlyDictionary<Key, PadButton> _keyToButton = DefaultPadKeyMap.Reverse(ButtonToKey);

        public static bool TryGetButton(Key key, out PadButton button) => _keyToButton.TryGetValue(key, out button);
    }
}
