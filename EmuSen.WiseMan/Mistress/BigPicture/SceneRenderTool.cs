using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Renders Art Book Next's two views on the synthetic library to PNGs outside the repository and times the frames - see EmuSen_BigPicture.md §13.4.
    public class SceneRenderTool
    {
        private readonly ITestOutputHelper _output;

        public SceneRenderTool(ITestOutputHelper output) => _output = output;

        public static string Cache => Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_CACHE") is { Length: > 0 } c
            ? c : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture");

        public static string MediaRoot => Path.Combine(Cache, "media", "downloaded_media");

        // The window a scene is shown in, sized to the screen it was built for.
        public static Window Host(SceneBuilder scene)
        {
            var window = new Window { Width = scene.W, Height = scene.H, Content = scene.Canvas, Background = Brushes.Black, SizeToContent = SizeToContent.Manual };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        public static RenderedFrame Frame(Window window)
        {
            window.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            using WriteableBitmap bitmap = window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            return UiTest.Capture(bitmap);
        }

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Render_both_views_at_both_sizes_and_time_them() => UiTest.Run(() =>
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            string png = Path.Combine(Cache, "png");
            Directory.CreateDirectory(png);
            string variant = Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_VARIANT") is { Length: > 0 } v ? v : "gamelist-list-metadata-cover";
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                var choices = new ThemeChoices { ScreenWidth = w, ScreenHeight = h, Variant = variant };
                var load = Stopwatch.StartNew();
                IReadOnlyList<SceneSystem> systems = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder, choices);
                load.Stop();
                foreach (string view in new[] { "system", "gamelist" })
                {
                    SceneData data = SyntheticLibrary.Data(systems, new Size(w, h), MediaRoot, system: 1, game: 2);
                    var cold = Stopwatch.StartNew();
                    SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
                    if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_ONLY") is { Length: > 0 } only)
                        foreach (SceneEntry entry in scene.Entries.Where(x => x.Control is not null && x.Element.Name != only)) entry.Control!.IsVisible = false;
                    Window window = Host(scene);
                    RenderedFrame first = Frame(window);
                    cold.Stop();
                    first.SavePng(Path.Combine(png, $"{view}-{variant}-{w}x{h}.png"));
                    var times = new List<double>();
                    for (int i = 0; i < 30; i++)
                    {
                        var t = Stopwatch.StartNew();
                        Frame(window);
                        times.Add(t.Elapsed.TotalMilliseconds);
                    }

                    times.Sort();
                    _output.WriteLine($"{view} {w}x{h}: load 5 systems {load.Elapsed.TotalMilliseconds:F1} ms, cold build+first frame {cold.Elapsed.TotalMilliseconds:F1} ms, "
                        + $"steady median {times[15]:F2} ms, min {times[0]:F2}, max {times[^1]:F2} (30 frames); skipped: "
                        + string.Join("; ", scene.Entries.Where(e => e.Skipped is not null).Select(e => $"{e.Element} ({e.Skipped})")));
                    window.Close();
                }
            }
        });
    }
}
