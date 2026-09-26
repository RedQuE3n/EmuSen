using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class CollectionsBenchFactAttribute : FactAttribute
    {
        public CollectionsBenchFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_BENCH") != "1")
                Skip = "Measures the collections on a 3,508-game library; set EMUSEN_BIGPICTURE_BENCH=1 - see EmuSen_BigPicture.md §22.9";
        }
    }

    // P69-P71: a large library's Show with and without the collections, a step in all games against one in a system, and the options sheet's model.
    [Collection(TestCollections.ProcessGlobals)]
    public class CollectionsBenchTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CollectionsBenchTool).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public CollectionsBenchTool(ITestOutputHelper output) => _out = output;

        private static readonly (string Name, string Full, int Count)[] Library =
            [("nes", "Nintendo Entertainment System", 900), ("snes", "Super Nintendo", 1100), ("n64", "Nintendo 64", 400), ("gb", "Game Boy", 608), ("gbc", "Game Boy Color", 500)];

        private static IReadOnlyList<ThemedShelf> Shelves()
        {
            var start = new DateTime(2026, 1, 1);
            string[] genres = ["Racing", "Puzzle", "Shooter", "Platform"];
            return Library.Select(s => new ThemedShelf(new ThemeSystem(s.Name, s.Full, s.Name), Enumerable.Range(0, s.Count).Select(j => new SceneGame($"{(char)('A' + j % 26)}ame {j:D4} {s.Name}", $"/lib/{s.Name}/{j}.rom")
            {
                Favorite = j % 17 == 0, LastPlayed = j % 5 == 0 ? start.AddMinutes(j) : null, PlayCount = j % 5 == 0 ? 1 + j % 7 : 0,
                Genre = j % 5 == 4 ? null : genres[j % 4], Rating = j % 4 == 0 ? null : j % 10 / 10f, Players = j % 3 == 0 ? "1-2" : "1",
            }).ToList())).ToList();
        }

        private static double Median(List<double> xs) => xs.Order().ElementAt(xs.Count / 2);

        private static string All(List<double> xs) => string.Join(", ", xs.Select(x => x.ToString("F1")));

        [CollectionsBenchFact]
        public Task Collections_on_a_large_library() => Session.Dispatch(() =>
        {
            using var theme = new SyntheticTheme();
            ThemedSession.Write(theme);
            IReadOnlyList<ThemedShelf> systems = Shelves();
            var custom = Enumerable.Range(0, 3).Select(c => new CustomCollection(c + 1, $"Mine {c}",
                systems.SelectMany(s => s.Games).Where((g, i) => i % (7 + c) == 0).Take(200).Select(g => g.File).ToHashSet())).ToList();
            var settings = new BigPictureCollections { AutoCollections = [BigPictureCollections.AllGames, BigPictureCollections.Favorites, BigPictureCollections.LastPlayed] };
            TimeSpan now = TimeSpan.Zero;
            var screen = new Size(1280, 800);
            var lines = new List<string> { $"games {systems.Sum(s => s.Games.Count)}, Debug build, headless, no window (the scene is built, not laid out or drawn)" };

            var plain = new ThemedLibrary(() => now);
            var withCollections = new ThemedLibrary(() => now);
            plain.Show(theme.Root, screen, systems, null);
            withCollections.Show(theme.Root, screen, CollectionShelves.Build(systems, custom, settings, _ => false), null);
            var a = new List<double>();
            var b = new List<double>();
            var build = new List<double>();
            for (int i = 0; i < 5; i++)
            {
                var clock = Stopwatch.StartNew();
                plain.Show(theme.Root, screen, systems, null);
                a.Add(clock.Elapsed.TotalMilliseconds);
                clock.Restart();
                IReadOnlyList<ThemedShelf> shelves = CollectionShelves.Build(systems, custom, settings, _ => false);
                build.Add(clock.Elapsed.TotalMilliseconds);
                withCollections.Show(theme.Root, screen, shelves, null);
                b.Add(clock.Elapsed.TotalMilliseconds);
            }
            lines.Add($"P69 Show without collections: median {Median(a):F1} ms [{All(a)}]");
            lines.Add($"P69 Show with collections, building them included: median {Median(b):F1} ms [{All(b)}]; building alone median {Median(build):F2} ms; ratio {Median(b) / Median(a):F2}");
            lines.Add("systems: " + string.Join(" ", withCollections.Stage!.Current.Data.Systems.Select(s => $"{s.System.Name}:{s.Games.Count}")));

            void Goto(string system)
            {
                for (int guard = 0; guard < 12 && withCollections.SelectedSystem!.System.Name != system; guard++)
                {
                    withCollections.PressDirection(UiButton.Right, now += TimeSpan.FromMilliseconds(600));
                    withCollections.ReleaseDirection(now += TimeSpan.FromMilliseconds(600));
                }
            }

            double Steps(string system)
            {
                withCollections.Show(theme.Root, screen, CollectionShelves.Build(systems, custom, settings, _ => false), null);
                Goto(system);
                withCollections.Command(UiButton.Accept, now += TimeSpan.FromMilliseconds(600));
                withCollections.Advance(now += TimeSpan.FromSeconds(1));
                var clock = Stopwatch.StartNew();
                for (int i = 0; i < 40; i++)
                {
                    withCollections.PressDirection(UiButton.Down, now += TimeSpan.FromMilliseconds(100));
                    withCollections.ReleaseDirection(now += TimeSpan.FromMilliseconds(100));
                }
                double per = clock.Elapsed.TotalMilliseconds / 40;
                withCollections.Command(UiButton.Back, now += TimeSpan.FromSeconds(1));
                withCollections.Advance(now += TimeSpan.FromSeconds(1));
                return per;
            }
            Steps("snes");
            Steps("all");
            var inSnes = new List<double>();
            var inAll = new List<double>();
            for (int i = 0; i < 3; i++)
            {
                inSnes.Add(Steps("snes"));
                inAll.Add(Steps("all"));
            }
            lines.Add($"P70 a step in snes: median {Median(inSnes):F2} ms [{All(inSnes)}]; in all games: median {Median(inAll):F2} ms [{All(inAll)}]; ratio {Median(inAll) / Median(inSnes):F2}");

            Goto("all");
            withCollections.Command(UiButton.Accept, now += TimeSpan.FromMilliseconds(600));
            var model = new List<double>();
            for (int i = 0; i < 6; i++)
            {
                var clock = Stopwatch.StartNew();
                var letters = withCollections.Letters();
                int values = GamelistOptions.Fields.Sum(f => GamelistOptions.Values(withCollections.Unfiltered, f).Count);
                model.Add(clock.Elapsed.TotalMilliseconds);
                if (i == 0) lines.Add($"all games listed {withCollections.Stage!.Current.Data.System.Games.Count}, letters {letters.Count}, filter values {values}");
            }
            lines.Add($"P71 the options sheet's model over all games: first {model[0]:F2} ms, then median {Median(model.Skip(1).ToList()):F2} ms (the sheet's own layout not included)");

            foreach (string line in lines) _out.WriteLine(line);
            File.WriteAllLines(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "bigpicture", "collections", "bench.txt"), lines);
        }, default);
    }
}
