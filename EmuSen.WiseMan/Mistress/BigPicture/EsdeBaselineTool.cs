using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Xml.Linq;
using Avalonia;
using Avalonia.Controls;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class EsdeBaselineFactAttribute : FactAttribute
    {
        public EsdeBaselineFactAttribute()
        {
            if (string.IsNullOrEmpty(EsdeBaselineTool.Output))
                Skip = "Writes the C# baseline of the ES-DE oracle's watched states; set EMUSEN_ESDE_BASELINE to Testing/esde-baseline - see Testing/esde-baseline/README.md";
        }
    }

    // The C# painter's rectangles and grabs for the watched states, on the oracle's library, before BigPicture/ retires - see Testing/esde-baseline/README.md.
    public class EsdeBaselineTool
    {
        private readonly ITestOutputHelper _output;

        public EsdeBaselineTool(ITestOutputHelper output) => _output = output;

        public static string? Output => Environment.GetEnvironmentVariable("EMUSEN_ESDE_BASELINE");

        private static string Home => Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

        private static string Setting(string name, params string[] fallback) =>
            Environment.GetEnvironmentVariable(name) is { Length: > 0 } set ? set : Path.Combine([Home, .. fallback]);

        private static string Oracle => Setting("EMUSEN_ESDE_ORACLE", ".cache", "emusen", "esde-oracle");

        private static string Themes => Setting("THEMES", ".cache", "emusen", "bigpicture", "corpus", "whole");

        private static string Grabs => Setting("EMUSEN_ESDE_GRABS", ".cache", "emusen", "tmp", "qt-s4", "compare", "csharp");

        public static readonly (int W, int H)[] Sizes = [(1280, 800), (1920, 1200)];

        public static readonly (string View, int Entry)[] States = [("system", 0), ("gamelist", 1)];

        // One row of states.tsv: a theme folder and the options ES-DE and the C# painter are both given.
        public sealed record State(string Theme, string? Variant, string? Scheme, string FontSize);

        public static IReadOnlyList<State> ReadStates(string folder) => File.ReadAllLines(Path.Combine(folder, "states.tsv")).Skip(1)
            .Where(l => l.Length > 0).Select(l => l.Split('\t'))
            .Select(f => new State(f[0], f[1] == "-" ? null : f[1], f[2] == "-" ? null : f[2], f[3])).ToList();

        // The oracle library's gamelists under Mistress's ES-DE names, ordered as Mistress gives them: favourites first, then by name; each folder's link.
        public static IReadOnlyList<ThemedShelf> Shelves(string library) => EsdeCompareTool.EsdeOrder.Select(name =>
        {
            EmuSen.Mistress.Library.LibraryShelf shelf = EmuSen.Mistress.Library.RuntimeCatalog.Shelves.First(s => s.EsdeSystem == name);
            var system = new ThemeSystem(shelf.EsdeSystem, shelf.EsdeFullName, shelf.EsdeSystem);
            string roms = Path.Combine(library, "roms", name);
            XElement list = XDocument.Load(Path.Combine(library, "gamelists", name, "gamelist.xml")).Root!;
            string Rom(XElement e) => Path.GetFullPath(Path.Combine(roms, ((string)e.Element("path")!)[2..]));
            var games = list.Elements("game").Select(g => new SceneGame((string)g.Element("name")!, Rom(g))
            {
                Description = (string?)g.Element("desc"), Developer = (string?)g.Element("developer"), Publisher = (string?)g.Element("publisher"),
                Genre = (string?)g.Element("genre"), Players = (string?)g.Element("players"), ReleaseDate = Date(g, "releasedate"), LastPlayed = Date(g, "lastplayed"),
                Rating = g.Element("rating") is { } r ? float.Parse(r.Value, CultureInfo.InvariantCulture) : null, PlayCount = (int?)g.Element("playcount") ?? 0,
                Favorite = Flag(g, "favorite"), Completed = Flag(g, "completed"), HideMetadata = Flag(g, "hidemetadata"),
                FolderPath = Path.GetDirectoryName(((string)g.Element("path")!)[2..])!.Replace(Path.DirectorySeparatorChar, '/'),
            }).OrderByDescending(g => g.Favorite).ThenBy(g => g.Name, StringComparer.OrdinalIgnoreCase).ToList();
            var links = list.Elements("folder").ToDictionary(f => Rom(f), f => Path.GetFullPath(Path.Combine(roms, ((string)f.Element("folderlink")!)[2..])));
            return new ThemedShelf(system, games) { FolderLinks = links };
        }).ToList();

        private static DateTime? Date(XElement g, string name) =>
            g.Element(name) is { } d ? DateTime.ParseExact(d.Value, "yyyyMMdd'T'HHmmss", CultureInfo.InvariantCulture) : null;

        private static bool Flag(XElement g, string name) => (string?)g.Element(name) == "true";

        // The scene the C# painter shows for one state: the SNES's system view, or its gamelist at the given entry as the C# lists it.
        public static SceneBuilder Scene(State state, string view, int entry, int w, int h, DateTime now, IReadOnlyList<ThemedShelf> shelves, string media)
        {
            var library = new ThemedLibrary(() => TimeSpan.Zero)
            {
                LiveClock = false, Now = () => now,
                Interface = new BigPictureInterface { DisplayClock = true, DisplayHelp = true, StartupSystem = "snes", StartupView = view },
            };
            var choices = new ThemeChoices { Variant = state.Variant, ColorScheme = state.Scheme, FontSize = state.FontSize };
            Assert.True(library.Show(Path.Combine(Themes, state.Theme), new Size(w, h), shelves, new EsdeMediaFolder(media), chosen: choices), library.Error);
            if (view == "gamelist")
            {
                IReadOnlyList<SceneGame> listed = library.Stage!.Current.Data.System.Games;
                Assert.True(library.ShowGame("snes", listed[entry].File, TimeSpan.Zero), $"entry {entry} ({listed[entry].Name}) is not a game");
            }
            SceneData data = library.Stage!.Current.Data with { Shuffle = 0 };
            Assert.Equal((view, "snes"), (library.ViewName, data.System.System.Name));
            return SceneBuilder.Build(data.System.Theme.View(view), data);
        }

        public static IEnumerable<SceneEntry> Drawn(SceneBuilder scene) => scene.Entries.Where(e => e.Control is { IsVisible: true });

        // ES-DE's key for an element, its type and name joined by an underscore; no type holds one.
        public static string Key(ResolvedElement e) => $"{e.Type}_{e.Name}";

        // The scene's drawn elements as `mockup 1`, in 1280 x 800 design units.
        public static string Spec(string canvas, State state, int w, int h, IReadOnlyList<(string Key, Rect Box)> boxes)
        {
            double unit = w / 1280.0;
            string N(double v) => Math.Round(v / unit, 2).ToString("0.##", CultureInfo.InvariantCulture);
            var text = new StringBuilder();
            text.Append("# The C# painter's drawn elements in one oracle state, written by EmuSen.WiseMan's EsdeBaselineTool; see README.md.\n");
            text.Append("mockup 1\n").Append($"canvas {canvas} 1280 800\n");
            text.Append($"source window {w}x{h}, design unit {unit.ToString("0.##", CultureInfo.InvariantCulture)} px; variant {state.Variant ?? "-"}, scheme {state.Scheme ?? "-"}, font size {state.FontSize}\n");
            foreach ((string key, Rect b) in boxes)
                text.Append($"element {(key.Contains(' ') ? $"\"{key}\"" : key)} x={N(b.X)} y={N(b.Y)} w={N(b.Width)} h={N(b.Height)}\n");
            return text.ToString();
        }

        [EsdeBaselineFact]
        public System.Threading.Tasks.Task Write_the_watched_states_rectangles_and_grabs() => UiTest.Run(() =>
        {
            string output = Output!;
            string library = Path.Combine(Oracle, "library");
            Assert.True(File.Exists(Path.Combine(library, "library.tsv")), $"no oracle library at {library}; run Testing/esde-oracle/library.py {library}");
            IReadOnlyList<ThemedShelf> shelves = Shelves(library);
            Directory.CreateDirectory(Grabs);
            foreach (State state in ReadStates(output))
                foreach ((int w, int h) in Sizes)
                    foreach ((string view, int entry) in States)
                    {
                        string name = (view == "system" ? view : $"{view}-{entry}") + $"-{w}x{h}";
                        string capture = Path.Combine(Oracle, state.Theme, name + ".png");
                        Assert.True(File.Exists(capture), $"no ES-DE capture at {capture}; run Testing/esde-baseline/capture.sh");
                        SceneBuilder scene = Scene(state, view, entry, w, h, File.GetLastWriteTime(capture), shelves, Path.Combine(library, "downloaded_media"));
                        var boxes = new List<(string Key, Rect Box)>();
                        RenderedFrame grab = SceneAssets.Render(scene, _ => boxes.AddRange(Drawn(scene).Select(e => (Key(e.Element), e.Control!.Bounds))));
                        Assert.Equal(boxes.Count, boxes.Select(b => b.Key).Distinct().Count());
                        string spec = Path.Combine(output, $"{state.Theme}-{name}.spec");
                        File.WriteAllText(spec, Spec($"{state.Theme}-{name}", state, w, h, boxes));
                        Assert.Equal(Drawn(scene).Count(), File.ReadLines(spec).Count(l => l.StartsWith("element ", StringComparison.Ordinal)));
                        grab.SavePng(Path.Combine(Grabs, $"{state.Theme}-{name}.png"));
                        _output.WriteLine($"{state.Theme} {name}: {boxes.Count} drawn, skipped: "
                            + string.Join("; ", scene.Entries.Where(e => e.Skipped is not null).Select(e => $"{Key(e.Element)} ({e.Skipped})")));
                    }
        });
    }
}
