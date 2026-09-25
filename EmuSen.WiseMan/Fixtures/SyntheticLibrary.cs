using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Xml.Linq;
using Avalonia;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.WiseMan.Fixtures
{
    // Invented games with invented metadata, a dummy ROM tree, an ES-DE gamelist and a downloaded_media tree of flat labelled PNGs; no real art - see EmuSen_BigPicture.md §13.2.
    public static class SyntheticLibrary
    {
        public static IReadOnlyList<(ThemeSystem System, string Extension)> Systems { get; } =
        [
            (new("nes", "Nintendo Entertainment System", "nes"), ".nes"),
            (new("snes", "Super Nintendo", "snes"), ".sfc"),
            (new("n64", "Nintendo 64", "n64"), ".z64"),
            (new("gb", "Game Boy", "gb"), ".gb"),
            (new("gbc", "Game Boy Color", "gbc"), ".gbc"),
        ];

        private static readonly string[] Titles =
        [
            "Aurora Drift", "Brass Lantern", "Cobalt Harbor", "Dune Relay", "Ember Circuit", "Fable of Tiles", "Granite Choir", "Hollow Comet",
            "Ivory Signal", "Juniper Vault", "Kestrel Run", "Lumen Garden",
        ];

        private static readonly string[] Genres = ["Platform", "Puzzle", "Racing", "Role playing", "Shooter", "Sports"];

        // Media sizes chosen so that each fit is visible: portrait covers, near-square screenshots, wide marquees.
        public static IReadOnlyDictionary<string, (int W, int H)> MediaSizes { get; } = new Dictionary<string, (int, int)>
        {
            ["covers"] = (600, 800), ["screenshots"] = (512, 448), ["titlescreens"] = (512, 448), ["marquees"] = (800, 300), ["miximages"] = (640, 480),
        };

        public static IReadOnlyList<SceneGame> Games(ThemeSystem system, string extension) => Titles.Select((t, i) => new SceneGame(t, $"{t} (Synthetic){extension}")
        {
            Description = $"{t} is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it. " +
                          "This second sentence is here so that the description is long enough to wrap over several lines and to be cut off where its box ends.",
            Developer = $"Studio {(char)('A' + i % 5)}",
            Publisher = $"Publisher {(char)('P' + i % 3)}",
            Genre = Genres[i % Genres.Length],
            Players = (1 + i % 4).ToString(CultureInfo.InvariantCulture),
            ReleaseDate = new DateTime(1990 + i, 1 + i % 12, 1 + i, 0, 0, 0),
            LastPlayed = i % 3 == 0 ? null : new DateTime(2026, 9, 24 - i, 10, 0, 0),
            Rating = i % 4 == 3 ? null : 0.2f + 0.2f * (i % 5) - 0.1f,
            PlayTime = i % 3 == 0 ? null : TimeSpan.FromMinutes(37 + 61 * i),
            PlayCount = i,
            Favorite = i % 2 == 0,
            Completed = i % 3 == 1,
            AltEmulator = i % 4 == 2,
        }).ToList();

        public static SceneData Data(IReadOnlyList<SceneSystem> systems, Size screen, string? mediaRoot, int system = 1, int game = 0) =>
            new(systems, screen) { SystemIndex = system, GameIndex = game, Media = mediaRoot is null ? null : new EsdeMediaFolder(mediaRoot) };

        public static IReadOnlyList<SceneSystem> Load(string themeDirectory, ThemeChoices choices)
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(themeDirectory);
            return Systems.Select(s => new SceneSystem(s.System, ThemeLoader.Load(caps, s.System, choices, new MediaPresence(new HashSet<string> { "cover", "screenshot", "marquee", "miximage", "titlescreen" })), Games(s.System, s.Extension))).ToList();
        }

        // One colour per game and type, so a picture's origin can be read back from its pixels.
        public static Color Colour(int game, string folder) =>
            Color.FromRgb((byte)(40 + 17 * game % 180), (byte)(60 + folder.Length * 23 % 160), (byte)(90 + 31 * game % 150));

        // Writes <root>/<system>/<type>/<rom name>.png for every game; must run on the UI thread of a headless session.
        public static void WriteMedia(string root)
        {
            foreach ((ThemeSystem system, string ext) in Systems)
            {
                IReadOnlyList<SceneGame> games = Games(system, ext);
                for (int i = 0; i < games.Count; i++)
                {
                    foreach ((string folder, (int w, int h)) in MediaSizes)
                    {
                        string path = Path.Combine(root, system.Name, folder, Path.GetFileNameWithoutExtension(games[i].File) + ".png");
                        if (File.Exists(path)) continue;
                        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                        Label(path, w, h, Colour(i, folder), $"{system.Name} {folder}\n{games[i].Name}\n{w}x{h}");
                    }
                }
            }
        }

        private static void Label(string path, int w, int h, Color colour, string text)
        {
            using var target = new RenderTargetBitmap(new PixelSize(w, h));
            using (DrawingContext dc = target.CreateDrawingContext())
            {
                dc.FillRectangle(new SolidColorBrush(colour), new Rect(0, 0, w, h));
                dc.DrawRectangle(null, new Pen(Brushes.White, 6), new Rect(3, 3, w - 6, h - 6));
                var formatted = new FormattedText(text, CultureInfo.InvariantCulture, FlowDirection.LeftToRight, Typeface.Default, Math.Max(12, h / 12.0), Brushes.White)
                {
                    TextAlignment = TextAlignment.Center, MaxTextWidth = w - 20,
                };
                dc.DrawText(formatted, new Point(10, (h - formatted.Height) / 2));
            }

            target.Save(path);
        }

        // Empty ROM files, and gamelist.xml files in ES-DE's format carrying the same metadata, for ES-DE to read the same library.
        public static void WriteEsdeLibrary(string romRoot, string gamelistRoot)
        {
            foreach ((ThemeSystem system, string ext) in Systems)
            {
                Directory.CreateDirectory(Path.Combine(romRoot, system.Name));
                var list = new XElement("gameList");
                foreach (SceneGame g in Games(system, ext))
                {
                    string rom = Path.Combine(romRoot, system.Name, g.File);
                    if (!File.Exists(rom)) File.WriteAllBytes(rom, []);
                    var game = new XElement("game", new XElement("path", "./" + g.File), new XElement("name", g.Name), new XElement("desc", g.Description),
                        new XElement("developer", g.Developer), new XElement("publisher", g.Publisher), new XElement("genre", g.Genre), new XElement("players", g.Players),
                        new XElement("releasedate", g.ReleaseDate?.ToString("yyyyMMdd'T'HHmmss", CultureInfo.InvariantCulture)),
                        new XElement("playcount", g.PlayCount));
                    if (g.Rating is { } r) game.Add(new XElement("rating", r.ToString("0.##", CultureInfo.InvariantCulture)));
                    if (g.LastPlayed is { } l) game.Add(new XElement("lastplayed", l.ToString("yyyyMMdd'T'HHmmss", CultureInfo.InvariantCulture)));
                    if (g.Favorite) game.Add(new XElement("favorite", "true"));
                    if (g.Completed) game.Add(new XElement("completed", "true"));
                    list.Add(game);
                }

                string file = Path.Combine(gamelistRoot, system.Name, "gamelist.xml");
                Directory.CreateDirectory(Path.GetDirectoryName(file)!);
                new XDocument(new XDeclaration("1.0", "utf-8", null), list).Save(file);
            }
        }
    }
}
