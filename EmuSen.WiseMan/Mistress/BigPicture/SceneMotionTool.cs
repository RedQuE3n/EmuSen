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

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Carousel_step_strip() => UiTest.Run(() =>
        {
            using var host = new SceneMotionHost(View("system"));
            host.View.Step(1, Ms(0));
            double[] at = [0, 40, 80, 120, 160, 200, 300, 500];
            var frames = at.Select(t => host.At(Ms(t))).ToList();
            SceneMotionHost.Strip(Path.Combine(Folder, "carousel-step-1280x800.png"), frames, 4);
            frames[3].SavePng(Path.Combine(Folder, "carousel-mid-slide-1280x800.png"));
            _output.WriteLine(string.Join(", ", at.Select(t => $"{t} ms")));
        });
    }
}
