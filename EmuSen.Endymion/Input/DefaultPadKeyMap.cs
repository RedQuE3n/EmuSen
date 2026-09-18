using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // The keyboard scheme both frontends start from, and the reverse lookup they both need - see EmuSen_Input.md §4.3.
    public static class DefaultPadKeyMap
    {
        // Every control from here on was added after keybindings.json first existed, so a file may predate it - see EmuSen_Input.md §7.4.
        public const PadControl FirstAddedControl = PadControl.L2;

        // A fresh dictionary per call: Mistress rebinds into its copy, so a shared instance would leak one frontend's edits into the other.
        public static Dictionary<PadControl, Key> Bindings() => new()
        {
            [PadControl.Up] = Key.Up,
            [PadControl.Down] = Key.Down,
            [PadControl.Left] = Key.Left,
            [PadControl.Right] = Key.Right,
            [PadControl.B] = Key.Z,
            [PadControl.A] = Key.X,
            [PadControl.Y] = Key.A,
            [PadControl.X] = Key.S,
            [PadControl.L] = Key.Q,
            [PadControl.R] = Key.W,
            [PadControl.Start] = Key.Enter,
            [PadControl.Select] = Key.RightShift,

            // The rest of the generic pad, on keys no default above or any hotkey holds - see EmuSen_Input.md §7.4.
            [PadControl.L2] = Key.E,
            [PadControl.R2] = Key.R,
            [PadControl.L3] = Key.C,
            [PadControl.R3] = Key.V,
            [PadControl.LeftStickUp] = Key.I,
            [PadControl.LeftStickDown] = Key.K,
            [PadControl.LeftStickLeft] = Key.J,
            [PadControl.LeftStickRight] = Key.L,
            [PadControl.RightStickUp] = Key.T,
            [PadControl.RightStickDown] = Key.G,
            [PadControl.RightStickLeft] = Key.F,
            [PadControl.RightStickRight] = Key.H,
        };

        // Last binding wins, which is what "a key can only do one thing" already guarantees upstream of here.
        public static Dictionary<Key, PadControl> Reverse(IReadOnlyDictionary<PadControl, Key> bindings)
        {
            var lookup = new Dictionary<Key, PadControl>();
            foreach (KeyValuePair<PadControl, Key> binding in bindings) lookup[binding.Value] = binding.Key;
            return lookup;
        }
    }
}
