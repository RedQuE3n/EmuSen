using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // The media types a theme draws for one system: its gamelist's imageType lists, video elements and variant triggers - see EmuSen_LibraryTools.md §8.
    public static class ThemeMediaKinds
    {
        public static IReadOnlyList<string> Of(ResolvedTheme theme)
        {
            var kinds = new List<string>();
            void Add(string k)
            {
                if (ThemeCapabilities.MediaTypes.Contains(k) && !kinds.Contains(k)) kinds.Add(k);
            }
            foreach (ResolvedElement e in theme.GamelistView.Elements)
            {
                foreach (string k in e.List("imageType")) Add(k);
                if (e.Type == "video") Add("video");
            }
            foreach (VariantOverride o in theme.Capabilities.FindVariant(theme.Selection.Variant ?? "")?.Overrides ?? [])
                foreach (string k in o.MediaTypes) Add(k);
            return kinds;
        }

        // The theme at directory loaded for an ES-DE system as though every type were present, so no trigger hides what it can draw; null where it has no view.
        public static IReadOnlyList<string>? For(string directory, ThemeChoices choices, string esdeSystem, string fullName)
        {
            ResolvedTheme theme = ThemeLoader.Load(directory, new ThemeSystem(esdeSystem, fullName, esdeSystem), choices, new MediaPresence(ThemeCapabilities.MediaTypes.ToHashSet(StringComparer.Ordinal)));
            return theme.IsThemed ? Of(theme) : null;
        }
    }
}
