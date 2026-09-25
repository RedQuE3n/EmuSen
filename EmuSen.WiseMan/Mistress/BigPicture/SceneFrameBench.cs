using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Splits a headless frame's cost into the harness and the controls, and times full-quality levers with a pixel check against a full redraw - see EmuSen_BigPicture.md §13.4.
    public class SceneFrameBench
    {
        private readonly ITestOutputHelper _output;

        public SceneFrameBench(ITestOutputHelper output) => _output = output;

        private const int Frames = 30;

        private static (double Median, RenderedFrame Last) Time(Window window, Action before)
        {
            var times = new List<double>();
            RenderedFrame last = default;
            for (int i = 0; i < Frames + 3; i++)
            {
                before();
                var t = Stopwatch.StartNew();
                using WriteableBitmap? bitmap = window.CaptureRenderedFrame();
                t.Stop();
                if (i >= 3) times.Add(t.Elapsed.TotalMilliseconds);
                if (i == Frames + 2) last = UiTest.Capture(bitmap!);
            }

            times.Sort();
            return (times[times.Count / 2], last);
        }

        private static void Invalidate(Visual root)
        {
            foreach (Visual v in root.GetSelfAndVisualDescendants()) v.InvalidateVisual();
        }

        private static (int Pixels, int MaxChannel) Diff(RenderedFrame a, RenderedFrame b)
        {
            int pixels = 0, max = 0;
            for (int i = 0; i < a.Rgba.Length; i += 4)
            {
                int d = Math.Max(Math.Max(Math.Abs(a.Rgba[i] - b.Rgba[i]), Math.Abs(a.Rgba[i + 1] - b.Rgba[i + 1])), Math.Abs(a.Rgba[i + 2] - b.Rgba[i + 2]));
                if (d > 0) pixels++;
                max = Math.Max(max, d);
            }

            return (pixels, max);
        }

        private void Line(string what, double ms, RenderedFrame frame, RenderedFrame reference)
        {
            (int pixels, int max) = Diff(frame, reference);
            _output.WriteLine($"  {what,-58} {ms,7:F2} ms   differs from full redraw: {pixels} px, max {max}");
        }

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Harness_and_levers() => UiTest.Run(() =>
        {
            SyntheticLibrary.WriteMedia(SceneRenderTool.MediaRoot);
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                IReadOnlyList<SceneSystem> systems = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder, new ThemeChoices { ScreenWidth = w, ScreenHeight = h });
                var empty = new Window { Width = w, Height = h, Background = Brushes.Black, Content = new NormalizedCanvas { Width = w, Height = h } };
                empty.Show();
                Dispatcher.UIThread.RunJobs();
                (double emptyFull, _) = Time(empty, () => Invalidate(empty));
                (double emptyStatic, _) = Time(empty, () => { });
                empty.Close();
                _output.WriteLine($"{w}x{h}: harness, empty window: full redraw {emptyFull:F2} ms, nothing invalidated {emptyStatic:F2} ms");

                foreach (string view in new[] { "system", "gamelist" })
                {
                    SceneData data = SyntheticLibrary.Data(systems, new Size(w, h), SceneRenderTool.MediaRoot, system: 1, game: 2);
                    SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
                    Window window = SceneRenderTool.Host(scene);
                    _output.WriteLine($"{w}x{h} {view}:");
                    (double full, RenderedFrame reference) = Time(window, () => Invalidate(window));
                    Line("full redraw, every visual invalidated", full, reference, reference);
                    (double still, RenderedFrame stillFrame) = Time(window, () => { });
                    Line("unchanged view, nothing invalidated", still, stillFrame, reference);

                    Control? clock = scene.Entries.FirstOrDefault(e => e.Element.Type == "clock" && e.Control is not null)?.Control;
                    Control? primary = scene.Entries.FirstOrDefault(e => e.Element.Type is "carousel" or "textlist" && e.Control is not null)?.Control;
                    if (clock is not null)
                    {
                        (double clockOnly, RenderedFrame f) = Time(window, () => clock.InvalidateVisual());
                        Line("only the clock invalidated (a minute tick)", clockOnly, f, reference);
                    }

                    if (primary is not null)
                    {
                        (double primaryOnly, RenderedFrame f) = Time(window, () => Invalidate(primary));
                        Line($"only the {primary.GetType().Name} invalidated", primaryOnly, f, reference);
                        primary.CacheMode = new BitmapCache();
                        Dispatcher.UIThread.RunJobs();
                        (double cachedFull, RenderedFrame f2) = Time(window, () => Invalidate(window));
                        Line($"full redraw, {primary.GetType().Name} with BitmapCache", cachedFull, f2, reference);
                        if (clock is not null)
                        {
                            (double cachedClock, RenderedFrame f3) = Time(window, () => clock.InvalidateVisual());
                            Line($"clock invalidated, {primary.GetType().Name} with BitmapCache", cachedClock, f3, reference);
                        }

                        primary.CacheMode = null;
                    }

                    foreach (Avalonia.Media.Imaging.BitmapInterpolationMode mode in new[] { Avalonia.Media.Imaging.BitmapInterpolationMode.MediumQuality, Avalonia.Media.Imaging.BitmapInterpolationMode.LowQuality })
                    {
                        List<FittedImage> images = window.GetVisualDescendants().OfType<FittedImage>().Where(i => i.Interpolation == Avalonia.Media.Imaging.BitmapInterpolationMode.HighQuality).ToList();
                        foreach (FittedImage i in images) i.Interpolation = mode;
                        (double ms, RenderedFrame f) = Time(window, () => Invalidate(window));
                        Line($"full redraw, {images.Count} images sampled {mode} (measurement only)", ms, f, reference);
                        foreach (FittedImage i in images) i.Interpolation = Avalonia.Media.Imaging.BitmapInterpolationMode.HighQuality;
                    }

                    window.Close();
                }
            }
        });
    }
}
