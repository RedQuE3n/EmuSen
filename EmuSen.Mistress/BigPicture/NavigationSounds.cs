using System;
using System.Collections.Generic;
using EmuSen.LunaP.Media;

namespace EmuSen.Mistress.BigPicture
{
    // Mistress's own navigation sounds, LunaP's synthesised cues under ES-DE's seven names, for any a theme does not provide - see EmuSen_BigPicture.md §29.
    public static class NavigationSounds
    {
        public const string Prefix = "emusen-builtin:";
        public const int Rate = LunaSounds.Rate;


        private static readonly IReadOnlyDictionary<string, LunaSoundCue?> Cues = new Dictionary<string, LunaSoundCue?>(StringComparer.Ordinal)
        {
            ["scroll"] = LunaSoundCue.Move,
            ["systembrowse"] = LunaSoundCue.Section,
            ["quicksysselect"] = null,
            ["select"] = LunaSoundCue.Choose,
            ["back"] = LunaSoundCue.Back,
            ["favorite"] = LunaSoundCue.Favourite,
            ["launch"] = LunaSoundCue.Launch,
        };

        public static IEnumerable<string> Names => Cues.Keys;

        public static string Key(string name) => Prefix + name;

        public static bool IsBuiltIn(string path) => path.StartsWith(Prefix, StringComparison.Ordinal);


        // A sound as 48 kHz stereo 32-bit float, UiSoundPlayer's format; empty for a name that is not one of the seven.
        public static byte[] Samples(string keyOrName)
        {
            string name = IsBuiltIn(keyOrName) ? keyOrName[Prefix.Length..] : keyOrName;
            if (!Cues.TryGetValue(name, out LunaSoundCue? cue)) return [];
            return cue is { } c ? LunaSounds.Samples(c) : LunaSounds.QuickSelectSamples();
        }
    }
}
