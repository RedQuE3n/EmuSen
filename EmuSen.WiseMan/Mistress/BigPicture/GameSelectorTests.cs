using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The system view's gameselector, by the rules ES-DE 3.4.1 was measured to follow on the same games - see EmuSen_BigPicture.md §36.2.
    public class GameSelectorTests
    {
        private const int W = 320, H = 200;

        private static readonly DateTime Day = new(2026, 9, 20, 10, 0, 0);

        // ES-DE's gba probe system: one game played, two never.
        private static IReadOnlyList<SceneGame> Few() =>
        [
            new("Aurora Drift", "a.gba") { PlayCount = 0 },
            new("Brass Lantern", "b.gba") { PlayCount = 3, LastPlayed = Day },
            new("Cobalt Harbor", "c.gba") { PlayCount = 0 },
        ];

        // ES-DE's genesis probe system less its hidden game, which the list leaves out: the most played is excluded from the counter, and a folder.
        private static IReadOnlyList<SceneGame> Excluding() =>
        [
            new("Aurora Drift", "a.md") { PlayCount = 9, LastPlayed = Day.AddDays(3), NotCounted = true },
            new("Brass Lantern", "b.md") { PlayCount = 2, LastPlayed = Day.AddDays(1) },
            new("Dune Relay", "d.md") { PlayCount = 1, LastPlayed = Day.AddDays(-1) },
            new("Tower Set", "Tower Set") { Folder = true, PlayCount = 20, LastPlayed = Day.AddDays(5) },
        ];

        private static SceneBuilder Scene(string elements, IReadOnlyList<SceneGame>? games = null, string view = "system", int shuffle = 0)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme($"<view name=\"{view}\">{elements}</view>");
            ThemeSystem system = SyntheticTheme.Snes;
            var systems = new List<SceneSystem> { new(system, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, system), games ?? SyntheticLibrary.Games(system, ".sfc")) };
            var data = new SceneData(systems, new Size(W, H)) { Media = new SceneAssets.Media(), Shuffle = shuffle, GameIndex = 2 };
            SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
            scene.Canvas.Measure(new Size(W, H));
            scene.Canvas.Arrange(new Rect(0, 0, W, H));
            return scene;
        }

        private static string Selector(string name, string selection, int count, bool? duplicates = null) =>
            $"<gameselector name=\"{name}\"><selection>{selection}</selection><gameCount>{count}</gameCount>{(duplicates is { } d ? $"<allowDuplicates>{d.ToString().ToLowerInvariant()}</allowDuplicates>" : "")}</gameselector>";

        private static string Name(string element, string? selector, int? entry, string field = "name") =>
            $"<text name=\"{element}\"><metadata>{field}</metadata>{(selector is null ? "" : $"<gameselector>{selector}</gameselector>")}{(entry is { } e ? $"<gameselectorEntry>{e}</gameselectorEntry>" : "")}<fontSize>0.05</fontSize></text>";

        private static string Row(string selector, int entries) => string.Concat(Enumerable.Range(0, entries).Select(e => Name($"{selector}{e}", selector, e)));

        private static string? Shown(SceneBuilder scene, string element) => (scene.Find("text", element)?.Control as FontText)?.Text;

        private static IReadOnlyList<string?> Shown(SceneBuilder scene, string selector, int entries) => Enumerable.Range(0, entries).Select(e => Shown(scene, $"{selector}{e}")).ToList();

        // Measured: LAST read Brass, Cobalt, Ember, Fable, Hollow on the twelve synthetic games, and only Brass on the gba probe (§36.2).
        [Fact]
        public Task Lastplayed_lists_the_played_games_newest_first_and_leaves_the_unplayed_out() => UiTest.Run(() =>
        {
            string xml = Selector("last", "lastplayed", 5) + Row("last", 5);
            Assert.Equal(["Brass Lantern", "Cobalt Harbor", "Ember Circuit", "Fable of Tiles", "Hollow Comet"], Shown(Scene(xml), "last", 5));
            Assert.Equal(["Brass Lantern", null, null, null, null], Shown(Scene(xml, Few()), "last", 5));
        });

        // Measured: MOST read Lumen, Kestrel, Juniper, Ivory, Hollow; on the gba probe only the game played three times.
        [Fact]
        public Task Mostplayed_lists_by_play_count_and_leaves_the_never_played_out() => UiTest.Run(() =>
        {
            string xml = Selector("most", "mostplayed", 5) + Row("most", 5);
            Assert.Equal(["Lumen Garden", "Kestrel Run", "Juniper Vault", "Ivory Signal", "Hollow Comet"], Shown(Scene(xml), "most", 5));
            Assert.Equal(["Brass Lantern", null, null, null, null], Shown(Scene(xml, Few()), "most", 5));
        });

        // Measured on the gba probe: six asked of three games gave three distinct names and three empty entries, and with duplicates six names, the first three distinct.
        [Fact]
        public Task Random_fills_no_more_entries_than_there_are_games_unless_duplicates_are_allowed() => UiTest.Run(() =>
        {
            for (int seed = 0; seed < 20; seed++)
            {
                IReadOnlyList<string?> once = Shown(Scene(Selector("rnd", "random", 6, false) + Row("rnd", 6), Few(), shuffle: seed), "rnd", 6);
                Assert.Equal(3, once.Take(3).Distinct().Count(n => n is not null));
                Assert.All(once.Skip(3), Assert.Null);
                IReadOnlyList<string?> dup = Shown(Scene(Selector("dup", "random", 6, true) + Row("dup", 6), Few(), shuffle: seed), "dup", 6);
                Assert.All(dup, Assert.NotNull);
                Assert.Equal(3, dup.Take(3).Distinct().Count());
            }
        });

        // Measured: entry 5 and entry 9 of a five-game lastplayed selector both read Hollow Comet, the fifth; past the games there are, nothing is drawn.
        [Fact]
        public Task An_entry_past_gameCount_is_clamped_to_the_last_and_an_entry_past_the_picks_draws_nothing() => UiTest.Run(() =>
        {
            string xml = Selector("last", "lastplayed", 5) + Name("five", "last", 5) + Name("nine", "last", 9);
            SceneBuilder scene = Scene(xml);
            Assert.Equal("Hollow Comet", Shown(scene, "five"));
            Assert.Equal("Hollow Comet", Shown(scene, "nine"));
            Assert.Null(Shown(Scene(xml, Few()), "nine"));
        });

        // Measured on the genesis probe: the game excluded from the counter was never picked, however recently or often played; folders are not games.
        [Fact]
        public Task Excluded_games_and_folders_are_never_picked() => UiTest.Run(() =>
        {
            string xml = Selector("last", "lastplayed", 5) + Selector("most", "mostplayed", 5) + Selector("rnd", "random", 5) + Row("last", 3) + Row("most", 3) + Row("rnd", 3);
            SceneBuilder scene = Scene(xml, Excluding());
            Assert.Equal(["Brass Lantern", "Dune Relay", null], Shown(scene, "last", 3));
            Assert.Equal(["Brass Lantern", "Dune Relay", null], Shown(scene, "most", 3));
            Assert.Equal(["Brass Lantern", "Dune Relay"], Shown(scene, "rnd", 3).Where(n => n is not null).OrderBy(n => n));
        });

        // Measured: with no gameselector in the view, a cover image, a name and a rating drew nothing, and a static text drew as usual.
        [Fact]
        public Task Without_a_gameselector_the_system_view_draws_no_game_media_metadata_or_rating() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene("<image name=\"cover\"><size>0.3 0.5</size><imageType>cover</imageType></image>" + Name("name", null, null)
                + "<rating name=\"stars\"><size>0 0.1</size></rating><datetime name=\"date\"><metadata>lastplayed</metadata></datetime><text name=\"static\"><text>STATIC</text></text>");
            Assert.Null(scene.Find("image", "cover")!.Control);
            Assert.Null(scene.Find("text", "name")!.Control);
            Assert.Null(scene.Find("rating", "stars")!.Control);
            Assert.Null(scene.Find("datetime", "date")!.Control);
            Assert.NotNull(scene.Find("text", "static")!.Control);
            SceneBuilder with = Scene(Selector("s", "lastplayed", 1) + "<image name=\"cover\"><size>0.3 0.5</size><imageType>cover</imageType></image><rating name=\"stars\"><size>0 0.1</size></rating>");
            Assert.NotNull(with.Find("image", "cover")!.Control);
            Assert.NotNull(with.Find("rating", "stars")!.Control);
        });

        // Measured: in either order of definition, a text naming no selector or one that does not exist read the mostplayed selector "amost", not "zlast".
        [Fact]
        public Task An_element_naming_no_selector_or_an_unknown_one_takes_the_selector_whose_name_sorts_first() => UiTest.Run(() =>
        {
            string texts = Name("none", null, null) + Name("wrong", "nosuch", null);
            foreach (string selectors in new[] { Selector("zlast", "lastplayed", 1) + Selector("amost", "mostplayed", 1), Selector("amost", "mostplayed", 1) + Selector("zlast", "lastplayed", 1) })
            {
                SceneBuilder scene = Scene(selectors + texts);
                Assert.Equal("Lumen Garden", Shown(scene, "none"));
                Assert.Equal("Lumen Garden", Shown(scene, "wrong"));
            }
        });

        // THEMES.md: "If only a single gameselector has been defined, this property is ignored"; measured so.
        [Fact]
        public Task With_one_selector_the_name_an_element_gives_is_ignored() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene(Selector("only", "mostplayed", 2) + Name("a", "nosuch", null) + Name("b", null, 1));
            Assert.Equal("Lumen Garden", Shown(scene, "a"));
            Assert.Equal("Kestrel Run", Shown(scene, "b"));
        });

        // Measured: the random rows changed at each move of the system carousel, back to the same system included.
        [Fact]
        public Task Random_picks_change_when_the_system_view_moves() => UiTest.Run(() =>
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme($"<view name=\"system\"><carousel name=\"c\"><itemTransitions>instant</itemTransitions></carousel>{Selector("rnd", "random", 6)}{Row("rnd", 6)}</view>");
            var systems = SyntheticLibrary.Systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, s.System), SyntheticLibrary.Games(s.System, s.Extension))).ToList();
            var view = new SceneView(new SceneData(systems, new Size(W, H)) { SystemIndex = 1 }, "system", TimeSpan.Zero);
            IReadOnlyList<string?> before = Shown(view.Scene, "rnd", 6);
            view.Step(1, TimeSpan.FromSeconds(1));
            view.Step(-1, TimeSpan.FromSeconds(2));
            Assert.Equal(1, view.Data.SystemIndex);
            Assert.NotEqual(before, Shown(view.Scene, "rnd", 6));
        });

        // THEMES.md supports gameselector in the system view only; the gamelist's elements follow the list's selection.
        [Fact]
        public Task The_gamelist_view_follows_the_list_and_not_a_gameselector() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene(Selector("most", "mostplayed", 1) + Name("x", "most", null), view: "gamelist");
            Assert.Equal("Cobalt Harbor", Shown(scene, "x"));
        });
    }
}
