using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // gamelistinfo and animation against what ES-DE 3.4.1 was measured to show and to play - see EmuSen_BigPicture.md §38.
    public class InfoAndAnimationTests
    {
        private const int W = 1280, H = 800;

        private static IReadOnlyList<SceneSystem> Systems(SyntheticTheme theme, Func<SceneSystem, SceneSystem>? adjust = null) =>
            SyntheticLibrary.Systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, s.System), SyntheticLibrary.Games(s.System, s.Extension)))
                .Select(s => adjust is null || s.System.Name != "snes" ? s : adjust(s)).ToList();

        private static InfoLine Info(Func<SceneSystem, SceneSystem>? adjust = null)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"gamelist\"><gamelistinfo name=\"i\"><pos>0 0</pos></gamelistinfo></view>");
            var data = new SceneData(Systems(theme, adjust), new Size(W, H)) { SystemIndex = 1 };
            return Assert.IsType<InfoLine>(SceneBuilder.Build(data.System.Theme.GamelistView, data).Find("gamelistinfo", "i")!.Control);
        }

        // Measured: a gamepad and 12, a star and 6, on the synthetic SNES games, in black when the theme names no colour.
        [Fact]
        public Task The_counts_are_the_games_and_favourites() => UiTest.Run(() =>
        {
            InfoLine line = Info();
            Assert.Equal([new InfoItem(InfoIcon.Gamepad, "12"), new InfoItem(InfoIcon.Star, "6")], line.Items);
            Assert.Equal(Colors.Black, line.Foreground);
            InfoLine excluding = Info(s => s with { Games = s.Games.Select((g, i) => i < 3 ? g with { NotCounted = true } : g).ToList() });
            Assert.Equal([new InfoItem(InfoIcon.Gamepad, "9"), new InfoItem(InfoIcon.Star, "4")], excluding.Items);
        });

        // Measured: a favourites filter showed a funnel and "6 / 12", no favourites count; inside a folder, the system's totals and an open folder.
        [Fact]
        public Task A_filter_shows_kept_over_all_and_a_folder_adds_its_picture() => UiTest.Run(() =>
        {
            InfoLine filtered = Info(s => s with { Info = GamelistCounts.Of(s.Games, s.Games.Where(g => !g.Favorite), filtered: true, inFolder: false) });
            Assert.Equal([new InfoItem(InfoIcon.Filter, "6 / 12")], filtered.Items);
            var tower = new[] { new SceneGame("Tower One", "t1.sfc") { FolderPath = "Tower" }, new SceneGame("Tower Two", "t2.sfc") { FolderPath = "Tower" } };
            var all = tower.Append(new SceneGame("Aurora Drift", "a.sfc")).Append(new SceneGame("Tower", "Tower") { Folder = true }).ToList();
            InfoLine inside = Info(s => s with { Games = tower, Info = GamelistCounts.Of(all, all, filtered: false, inFolder: true) });
            Assert.Equal([new InfoItem(InfoIcon.Gamepad, "3"), new InfoItem(InfoIcon.Star, "0"), new InfoItem(InfoIcon.Folder)], inside.Items);
        });

        private static string Four => SceneAssets.Gif("es-de-four", 8, 6, [10, 20, 30, 40], (f, x, y) => new[] { Colors.Red, Colors.Lime, Colors.Blue, Colors.Yellow }[f]);

        // Measured at 30 frames a second after a move of the system view (a1, a3, a5 and a10 of §38): the first frame for two frame times, then the order; the resets on each move.
        [Fact]
        public Task Animations_play_by_the_first_delay_and_restart_when_the_system_view_moves() => UiTest.Run(() =>
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"system\"><carousel name=\"c\"><itemTransitions>instant</itemTransitions></carousel>"
                + $"<animation name=\"a1\"><size>0.1 0</size><path>{Four}</path></animation>"
                + $"<animation name=\"a3\"><size>0.1 0</size><path>{Four}</path><direction>reverse</direction></animation>"
                + $"<animation name=\"a5\"><size>0.1 0</size><path>{Four}</path><iterationCount>1</iterationCount></animation>"
                + $"<animation name=\"a10\"><size>0.1 0</size><path>{Four}</path><iterationCount>2</iterationCount><direction>alternate</direction></animation></view>");
            var view = new SceneView(new SceneData(Systems(theme), new Size(W, H)) { SystemIndex = 1 }, "system", TimeSpan.Zero);
            view.Advance(TimeSpan.FromSeconds(3));
            view.Step(1, TimeSpan.FromSeconds(3));
            int At(string name, double ms)
            {
                view.Advance(TimeSpan.FromSeconds(3) + TimeSpan.FromMilliseconds(ms));
                var a = (FrameSequenceImage)view.Scene.Find("animation", name)!.Control!;
                return a.FrameAt(a.Time);
            }

            int[] Run(string name) => Enumerable.Range(0, 14).Select(i => At(name, 50 + 100 * i)).ToArray();
            Assert.Equal([0, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0], Run("a1"));
            Assert.Equal([0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3], Run("a3"));
            Assert.Equal([0, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3], Run("a5"));
            Assert.Equal([0, 0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0], Run("a10"));
            Assert.True(view.IsMoving);
        });

        // A Lottie file is not drawn: LunaP draws no Lottie (§38).
        [Fact]
        public Task A_lottie_file_draws_nothing() => UiTest.Run(() =>
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"system\"><animation name=\"l\"><size>0.1 0.1</size><path>./a.json</path></animation></view>");
            System.IO.File.Copy(Four, theme.PathOf("a.json"));
            var data = new SceneData(Systems(theme), new Size(W, H)) { SystemIndex = 1 };
            Assert.Null(SceneBuilder.Build(data.System.Theme.SystemView, data).Find("animation", "l")!.Control);
        });
    }
}
