using System;
using EmuSen.LunaP.Controls;
using SDL3;

namespace EmuSen.Mistress.Input
{
    // Which printing a connected pad carries: SDL's own type first, the pad's name when SDL does not know - see EmuSen_Settings_Reference.md §4.52.
    public static class PadFamilies
    {
        public static PadFamily Of(SDL.GamepadType type, string? name)
        {
            string kind = type.ToString();
            if (kind.StartsWith("Xbox", StringComparison.OrdinalIgnoreCase)) return PadFamily.Xbox;
            if (kind.StartsWith("PS", StringComparison.Ordinal)) return PadFamily.PlayStation;
            if (kind.StartsWith("Nintendo", StringComparison.OrdinalIgnoreCase) || kind.Contains("GameCube", StringComparison.OrdinalIgnoreCase)) return PadFamily.Nintendo;
            return ByName(name);
        }

        // For a type SDL reports as Standard or Unknown; the Legion Go S and a Deck print an Xbox layout.
        public static PadFamily ByName(string? name)
        {
            if (string.IsNullOrWhiteSpace(name)) return PadFamily.Generic;
            string n = name.ToLowerInvariant();
            if (Any(n, "xbox", "x-box", "xinput", "legion go", "steam deck", "steam virtual gamepad")) return PadFamily.Xbox;
            if (Any(n, "playstation", "dualshock", "dualsense", "ps3", "ps4", "ps5", "sony")) return PadFamily.PlayStation;
            if (Any(n, "nintendo", "switch", "joy-con", "joycon", "pro controller", "gamecube")) return PadFamily.Nintendo;
            return PadFamily.Generic;
        }

        private static bool Any(string name, params string[] words) => Array.Exists(words, w => name.Contains(w, StringComparison.Ordinal));
    }
}
