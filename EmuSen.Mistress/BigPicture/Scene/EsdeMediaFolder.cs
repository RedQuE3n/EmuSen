using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // An ES-DE downloaded_media folder read in place: <root>/<system>/<type folder>/<file name without extension>.<png|jpg> - see EmuSen_BigPicture.md §13.2.
    public sealed class EsdeMediaFolder : ISceneMedia
    {
        public static readonly IReadOnlyDictionary<string, string> Folders = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["miximage"] = "miximages", ["marquee"] = "marquees", ["screenshot"] = "screenshots", ["titlescreen"] = "titlescreens", ["cover"] = "covers",
            ["backcover"] = "backcovers", ["3dbox"] = "3dboxes", ["physicalmedia"] = "physicalmedia", ["fanart"] = "fanart", ["video"] = "videos",
        };

        private static readonly string[] Extensions = [".png", ".jpg", ".jpeg"];

        public EsdeMediaFolder(string root) => Root = root;

        public string Root { get; }

        public string? Find(ThemeSystem system, SceneGame game, string mediaType)
        {
            if (!Folders.TryGetValue(mediaType, out string? folder)) return null;
            string stem = Path.GetFileNameWithoutExtension(game.File);
            foreach (string ext in Extensions)
            {
                string path = Path.Combine(Root, system.Name, folder, stem + ext);
                if (File.Exists(path)) return path;
            }

            return null;
        }
    }
}
