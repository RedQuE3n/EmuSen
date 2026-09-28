using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // An ES-DE downloaded_media folder read in place: <root>/<system>/<type folder>/<game's folder>/<file name without extension>.<extension> - see EmuSen_BigPicture.md §13.2, §16 and §30.
    public sealed class EsdeMediaFolder : ISceneMedia
    {
        public static readonly IReadOnlyDictionary<string, string> Folders = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["miximage"] = "miximages", ["marquee"] = "marquees", ["screenshot"] = "screenshots", ["titlescreen"] = "titlescreens", ["cover"] = "covers",
            ["backcover"] = "backcovers", ["3dbox"] = "3dboxes", ["physicalmedia"] = "physicalmedia", ["fanart"] = "fanart", ["video"] = "videos",
            ["manual"] = "manuals",
        };

        // The extensions USERGUIDE.md lists ("Manually copying game media files"), in the order they are tried.
        public static readonly IReadOnlyList<string> ImageExtensions = [".png", ".jpg", ".webp"];
        public static readonly IReadOnlyList<string> VideoExtensions = [".mp4", ".mkv", ".avi", ".wmv", ".mov", ".webm"];
        public static readonly IReadOnlyList<string> ManualExtensions = [".pdf"];

        public EsdeMediaFolder(string root) => Root = root;

        public string Root { get; }

        private static IReadOnlyList<string> ExtensionsOf(string mediaType) => mediaType switch { "video" => VideoExtensions, "manual" => ManualExtensions, _ => ImageExtensions };

        // Where below a type folder a game's files are named: its folder then its stem, and for a foldered game its stem alone after that (§30).
        public static IEnumerable<string> Names(SceneGame game)
        {
            if (game is { Folder: true, IsCollection: false }) return game.FolderPath.Length > 0 ? [game.FolderPath] : [];
            string stem = Path.GetFileNameWithoutExtension(game.File);
            return game.FolderPath.Length == 0 ? [stem] : [game.FolderPath + "/" + stem, stem];
        }

        public string? Find(ThemeSystem system, SceneGame game, string mediaType)
        {
            if (!Folders.TryGetValue(mediaType, out string? folder)) return null;
            foreach (string name in Names(game))
            foreach (string ext in ExtensionsOf(mediaType))
            {
                string path = Path.Combine(Root, system.Name, folder, name + ext);
                if (File.Exists(path)) return path;
            }

            return null;
        }

        // One listing of each type folder, not a question per game, type and extension (§16, P53); a type folder's subfolders are listed when a game has a folder.
        public IReadOnlySet<string> Present(ThemeSystem system, IReadOnlyList<SceneGame> games)
        {
            var found = new HashSet<string>(StringComparer.Ordinal);
            HashSet<string>? names = null;
            bool nested = games.Any(g => g.FolderPath.Length > 0);
            foreach ((string type, string folder) in Folders)
            {
                string dir = Path.Combine(Root, system.Name, folder);
                if (!Directory.Exists(dir)) continue;
                names ??= games.SelectMany(Names).ToHashSet(StringComparer.Ordinal);
                IReadOnlyList<string> extensions = ExtensionsOf(type);
                var files = Directory.EnumerateFiles(dir, "*", nested ? SearchOption.AllDirectories : SearchOption.TopDirectoryOnly);
                if (files.Any(f => extensions.Contains(Path.GetExtension(f)) && names.Contains(NameIn(dir, f)))) found.Add(type);
            }

            return found;
        }

        private static string NameIn(string dir, string file)
        {
            string relative = Path.GetRelativePath(dir, file);
            string stem = Path.Combine(Path.GetDirectoryName(relative) ?? "", Path.GetFileNameWithoutExtension(relative));
            return Path.DirectorySeparatorChar == '/' ? stem : stem.Replace(Path.DirectorySeparatorChar, '/');
        }

        // A folder's write time changes when a file is added to it or taken from it, so each type folder's own folders are stamped too.
        public string? Stamp(ThemeSystem system) =>
            string.Join(",", Folders.Values.Select(f => Path.Combine(Root, system.Name, f)).Select(d => Directory.Exists(d)
                ? string.Join(";", Directory.EnumerateDirectories(d, "*", SearchOption.AllDirectories).Prepend(d).Select(x => Directory.GetLastWriteTimeUtc(x).Ticks))
                : "0"));
    }
}
