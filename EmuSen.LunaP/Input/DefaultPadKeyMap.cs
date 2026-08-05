using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;

namespace EmuSen.LunaP.Input
{
    // The keyboard scheme both frontends start from, and the reverse lookup they both need - see EmuSen_LunaP.md §15.
    public static class DefaultPadKeyMap
    {
        // A fresh dictionary per call: Mistress rebinds into its copy, so a shared instance would leak one frontend's edits into the other.
        public static Dictionary<PadButton, Key> Bindings() => new()
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

        // Last binding wins, which is what "a key can only do one thing" already guarantees upstream of here.
        public static Dictionary<Key, PadButton> Reverse(IReadOnlyDictionary<PadButton, Key> bindings)
        {
            var lookup = new Dictionary<Key, PadButton>();
            foreach (KeyValuePair<PadButton, Key> binding in bindings) lookup[binding.Value] = binding.Key;
            return lookup;
        }
    }
}
