using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // The keyboard scheme both frontends start from, and the reverse lookup they both need - see EmuSen_Input.md §4.3.
    public static class DefaultPadKeyMap
    {
        // Endymion's defaults, from the platform library, which reads keybindings.json by them; a fresh dictionary per call, since Mistress rebinds into its copy.
        public static Dictionary<PadControl, Key> Bindings() => BindingFiles.Defaults<Dictionary<PadControl, Key>>(BindingFile.Keys);

        // Last binding wins, which is what "a key can only do one thing" already guarantees upstream of here.
        public static Dictionary<Key, PadControl> Reverse(IReadOnlyDictionary<PadControl, Key> bindings)
        {
            var lookup = new Dictionary<Key, PadControl>();
            foreach (KeyValuePair<PadControl, Key> binding in bindings) lookup[binding.Value] = binding.Key;
            return lookup;
        }
    }
}
