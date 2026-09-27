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
    // What ES-DE 3.4.1 draws that a theme did not ask for, and the words it shows for what a game lacks, as measured - see EmuSen_BigPicture.md §38.
    public class EsdeDefaultsTests
    {
        private const int W = 1280, H = 800;

        private static ResolvedTheme Load(string body)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme(body);
            return theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H });
        }

        private static IEnumerable<ResolvedElement> Defaults(ResolvedView view) => view.Elements.Where(e => e.Name == ResolvedView.DefaultName);

        // Measured: a probe theme with neither drew ES-DE's help bar at the bottom left and its status at the top right, in both views.
        [Fact]
        public void A_view_with_no_help_or_status_gets_both_at_their_documented_defaults()
        {
            ResolvedTheme theme = Load("<view name=\"system\"><carousel name=\"c\"/></view><view name=\"gamelist\"><textlist name=\"t\"/></view>");
            foreach (ResolvedView view in new[] { theme.SystemView, theme.GamelistView })
            {
                Assert.Equal(["helpsystem", "systemstatus"], Defaults(view).Select(e => e.Type).OrderBy(t => t));
                ResolvedElement help = Defaults(view).Single(e => e.Type == "helpsystem");
                Assert.Equal(new NormalizedPair(0.012f, 0.9515f), help.Pair("pos"));
                ResolvedElement status = Defaults(view).Single(e => e.Type == "systemstatus");
                Assert.Equal(new NormalizedPair(0.982f, 0.016f), status.Pair("pos"));
            }
        }

        // Measured: a help bar in the gamelist only left the system view with the default; one at opacity 0 left no default at all.
        [Fact]
        public void The_default_is_per_view_and_any_element_of_the_type_replaces_it_even_an_invisible_one()
        {
            ResolvedTheme theme = Load("<view name=\"gamelist\"><helpsystem name=\"h\"><pos>0.5 0.5</pos></helpsystem></view>"
                + "<view name=\"system\"><systemstatus name=\"s\"><opacity>0</opacity></systemstatus></view>");
            Assert.Equal(["helpsystem"], Defaults(theme.SystemView).Select(e => e.Type));
            Assert.Equal(["systemstatus"], Defaults(theme.GamelistView).Select(e => e.Type));
        }

        private static SceneBuilder Scene(string elements, SceneGame game, string view = "gamelist", IReadOnlyList<SceneGame>? more = null)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme($"<view name=\"{view}\">{elements}</view>");
            var games = new List<SceneGame> { game };
            if (more is not null) games.AddRange(more);
            var systems = new List<SceneSystem> { new(SyntheticTheme.Snes, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }), games) };
            return SceneBuilder.Build(systems[0].Theme.View(view), new SceneData(systems, new Size(W, H)));
        }

        private static string? Shown(SceneBuilder scene, string name) =>
            scene.Entries.Single(e => e.Element.Name == name).Control is FontText t ? t.Text : null;

        private static string Text(string name, string field, string extra = "") => $"<text name=\"{name}\"><metadata>{field}</metadata>{extra}</text>";

        // Measured on a game with no metadata: "unknown" for these five, "never" for the last played date, "unknown" for the release date, nothing for the description.
        [Fact]
        public Task A_missing_field_shows_Mistress_s_word_where_ES_DE_shows_one() => UiTest.Run(() =>
        {
            var bare = new SceneGame("Aurora Drift", "a.sfc");
            string texts = string.Concat(new[] { "developer", "publisher", "genre", "players", "playtime", "description" }.Select(f => Text(f, f)))
                + "<datetime name=\"rd\"><metadata>releasedate</metadata></datetime><datetime name=\"lp\"><metadata>lastplayed</metadata></datetime>";
            SceneBuilder scene = Scene(texts, bare);
            foreach (string f in new[] { "developer", "publisher", "genre", "players", "playtime" }) Assert.Equal(SceneWords.Unknown, Shown(scene, f));
            Assert.Null(Shown(scene, "description"));
            Assert.Equal(SceneWords.Unknown, Shown(scene, "rd"));
            Assert.Equal(SceneWords.Never, Shown(scene, "lp"));
        });

        // THEMES.md: defaultValue overrides the default "unknown" and "never".
        [Fact]
        public Task A_theme_s_defaultValue_replaces_the_word() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene(Text("d", "developer", "<defaultValue>nobody</defaultValue>") + "<datetime name=\"lp\"><metadata>lastplayed</metadata><defaultValue>not yet</defaultValue></datetime>",
                new SceneGame("Aurora Drift", "a.sfc"));
            Assert.Equal("nobody", Shown(scene, "d"));
            Assert.Equal("not yet", Shown(scene, "lp"));
        });

        // Measured: favorite, completed, kidgame and broken read "yes" or "no", in lower case.
        [Fact]
        public Task Flags_read_yes_or_no_in_lower_case() => UiTest.Run(() =>
        {
            SceneBuilder scene = Scene(Text("f", "favorite") + Text("c", "completed"), new SceneGame("Aurora Drift", "a.sfc") { Favorite = true });
            Assert.Equal("yes", Shown(scene, "f"));
            Assert.Equal("no", Shown(scene, "c"));
        });

        // Measured: "1 game", "1 favorite"; the plural from two on. Aura writes gamecountGames upper-cased, "1 GAME".
        [Fact]
        public Task Counts_of_one_are_singular() => UiTest.Run(() =>
        {
            string xml = "<text name=\"g\"><systemdata>gamecountGames</systemdata></text><text name=\"f\"><systemdata>gamecountFavorites</systemdata></text><text name=\"a\"><systemdata>gamecount</systemdata></text>";
            SceneBuilder one = Scene(xml, new SceneGame("Aurora Drift", "a.sfc") { Favorite = true }, "system");
            Assert.Equal("1 game", Shown(one, "g"));
            Assert.Equal("1 favorite", Shown(one, "f"));
            Assert.Equal("1 game available, 1 favorite", Shown(one, "a"));
            SceneBuilder two = Scene(xml, new SceneGame("Aurora Drift", "a.sfc") { Favorite = true }, "system", [new SceneGame("Brass Lantern", "b.sfc") { Favorite = true }]);
            Assert.Equal("2 games", Shown(two, "g"));
            Assert.Equal("2 favorites", Shown(two, "f"));
        });
    }
}
