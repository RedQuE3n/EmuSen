using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // An ES-DE downloaded_media folder read in place: <root>/<system>/<type folder>/<file name without extension>.<extension> - see EmuSen_BigPicture.md §13.2 and §16.
    public sealed class EsdeMediaFolder : ISceneMedia
    {
        public static readonly IReadOnlyDictionary<string, string> Folders = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["miximage"] = "miximages", ["marquee"] = "marquees", ["screenshot"] = "screenshots", ["titlescreen"] = "titlescreens", ["cover"] = "covers",
            ["backcover"] = "backcovers", ["3dbox"] = "3dboxes", ["physicalmedia"] = "physicalmedia", ["fanart"] = "fanart", ["video"] = "videos",
        };

        // The extensions USERGUIDE.md lists ("Manually copying game media files"), in the order they are tried.
        public static readonly IReadOnlyList<string> ImageExtensions = [".png", ".jpg", ".webp"];
        public static readonly IReadOnlyList<string> VideoExtensions = [".mp4", ".mkv", ".avi", ".wmv", ".mov", ".webm"];

        public EsdeMediaFolder(string root) => Root = root;

        public string Root { get; }

        private static IReadOnlyList<string> ExtensionsOf(string mediaType) => mediaType == "video" ? VideoExtensions : ImageExtensions;

        public string? Find(ThemeSystem system, SceneGame game, string mediaType)
        {
            if (!Folders.TryGetValue(mediaType, out string? folder)) return null;
            string stem = Path.GetFileNameWithoutExtension(game.File);
            foreach (string ext in ExtensionsOf(mediaType))
            {
                string path = Path.Combine(Root, system.Name, folder, stem + ext);
                if (File.Exists(path)) return path;
            }

            return null;
        }

        // One listing of each type folder, not a question per game, type and extension (§16, P53).
        public IReadOnlySet<string> Present(ThemeSystem system, IReadOnlyList<SceneGame> games)
        {
            var found = new HashSet<string>(StringComparer.Ordinal);
            HashSet<string>? stems = null;
            foreach ((string type, string folder) in Folders)
            {
                string dir = Path.Combine(Root, system.Name, folder);
                if (!Directory.Exists(dir)) continue;
                stems ??= games.Select(g => Path.GetFileNameWithoutExtension(g.File)).ToHashSet(StringComparer.Ordinal);
                IReadOnlyList<string> extensions = ExtensionsOf(type);
                if (Directory.EnumerateFiles(dir).Any(f => extensions.Contains(Path.GetExtension(f)) && stems.Contains(Path.GetFileNameWithoutExtension(f)))) found.Add(type);
            }

            return found;
        }

        // A type folder's write time changes when a file is added to it or taken from it.
        public string? Stamp(ThemeSystem system) =>
            string.Join(",", Folders.Values.Select(f => Path.Combine(Root, system.Name, f)).Select(d => Directory.Exists(d) ? Directory.GetLastWriteTimeUtc(d).Ticks : 0));
    }
}
