using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Renders Art Book Next's motion on the synthetic library as PNG strips outside the repository: the carousel mid-slide, the list's repeat, the scrolling texts - see EmuSen_BigPicture.md §14.8.
    public class SceneMotionTool
    {
        private readonly ITestOutputHelper _output;

        public SceneMotionTool(ITestOutputHelper output) => _output = output;

        private static TimeSpan Ms(double ms) => TimeSpan.FromMilliseconds(ms);

        public static string Folder => Path.Combine(SceneRenderTool.Cache, "motion-png");

        public static SceneView View(string view, int w = 1280, int h = 800, int system = 2, int game = 0)
        {
            SyntheticLibrary.WriteMedia(SceneRenderTool.MediaRoot);
            IReadOnlyList<SceneSystem> systems = EsdeCompareTool.Systems(w, h, "gamelist-list-metadata-cover");
            SceneData data = EsdeCompareTool.Data(systems, w, h, DateTime.Today.AddHours(12)) with { SystemIndex = system, GameIndex = game };
            return new SceneView(data, view, TimeSpan.Zero);
        }

        private void Strip(SceneMotionHost host, string name, double[] at, int single = -1)
        {
            var frames = at.Select(t => host.At(Ms(t))).ToList();
            SceneMotionHost.Strip(Path.Combine(Folder, name + ".png"), frames, 4);
            if (single >= 0) frames[single].SavePng(Path.Combine(Folder, name + "-frame.png"));
            _output.WriteLine($"{name}: {string.Join(", ", at.Select(t => $"{t} ms"))}");
        }

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Carousel_step_strip() => UiTest.Run(() =>
        {
            using var host = new SceneMotionHost(View("system"));
            host.View.Step(1, Ms(0));
            Strip(host, "carousel-step-1280x800", [0, 50, 100, 150, 200, 300, 400, 500], 2);
        });

        [ArtBookNextFact]
        public System.Threading.Tasks.Task List_held_strip() => UiTest.Run(() =>
        {
            using var host = new SceneMotionHost(View("gamelist"));
            host.View.Press(1, Ms(0));
            Strip(host, "list-held-1280x800", [0, 400, 520, 650, 1000, 1500, 1720, 1800], 3);
            host.View.Release(Ms(1800));
            Strip(host, "list-released-1280x800", [1800, 1875, 1950, 2400]);
        });

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Scrolling_texts_strip() => UiTest.Run(() =>
        {
            SceneView view = View("gamelist", game: 2);
            using var host = new SceneMotionHost(view);
            Strip(host, "description-1280x800", [0, 5900, 7000, 9000, 12000, 20000], 3);
        });

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Slide_strip() => UiTest.Run(() =>
        {
            IReadOnlyList<SceneSystem> loaded = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder,
                new ThemeChoices { ScreenWidth = 1280, ScreenHeight = 800, Variant = "gamelist-list-metadata-cover", ColorScheme = "dark-screenshots", Transitions = "slide" });
            IReadOnlyList<SceneSystem> systems = EsdeCompareTool.EsdeOrder.Select(n => loaded.Single(s => s.System.Name == n)).ToList();
            SceneData data = EsdeCompareTool.Data(systems, 1280, 800, DateTime.Today.AddHours(12));
            Assert.Equal(TransitionAnimation.Slide, data.System.Theme.Transitions.SystemToGamelist);
            var stage = new SceneStage(data, "system", Ms(0));
            using var host = new SceneMotionHost(stage.Root, 1280, 800);
            stage.Switch(Ms(0));
            var frames = new List<RenderedFrame>();
            foreach (double t in new double[] { 0, 50, 100, 200, 300, 402 })
            {
                stage.Advance(Ms(t));
                frames.Add(host.Frame());
            }

            SceneMotionHost.Strip(Path.Combine(Folder, "slide-1280x800.png"), frames, 4);
            frames[2].SavePng(Path.Combine(Folder, "slide-1280x800-frame.png"));
        });
    }
}
