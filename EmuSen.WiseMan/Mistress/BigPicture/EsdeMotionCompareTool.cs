using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Replays a recorded ES-DE carousel run's presses on Mistress's scene and compares every item's box and opacity frame by frame; skips without the recording - see EmuSen_BigPicture.md §14.7.
    public class EsdeMotionCompareTool
    {
        private readonly ITestOutputHelper _output;

        public EsdeMotionCompareTool(ITestOutputHelper output) => _output = output;

        public static string Motion => Path.Combine(SceneRenderTool.Cache, "motion");

        private sealed record Row(double Pts, string Item, double Cx, double W, double Alpha);

        public static bool Present => File.Exists(Path.Combine(Motion, "runs", "ps1", "carousel_taps.csv")) && Directory.Exists(Path.Combine(Motion, "theme", "motion-probe"));

        [ArtBookNextFact]
        public System.Threading.Tasks.Task Carousel_taps_against_the_recording() => UiTest.Run(() =>
        {
            if (!Present)
            {
                _output.WriteLine($"skipped: no recording under {Motion}");
                return;
            }

            string run = Path.Combine(Motion, "runs", "ps1");
            var rows = File.ReadLines(Path.Combine(run, "carousel_taps.csv")).Skip(1).Select(l => l.Split(','))
                .Select(p => new Row(D(p[1]), p[2], D(p[5]), D(p[6]), D(p[10]))).ToList();
            var taps = File.ReadLines(Path.Combine(run, "input.csv")).Skip(1).Select(l => l.Split(','))
                .Where(p => p[2] == "tap" && p[4] == "1").Select(p => (Real: D(p[1]), Dir: p[3] == "right" ? 1 : -1)).ToList();
            double firstHold = File.ReadLines(Path.Combine(run, "input.csv")).Skip(1).Select(l => l.Split(',')).Where(p => p[2] == "hold").Select(p => D(p[1])).DefaultIfEmpty(double.MaxValue).Min();
            double t0 = taps[0].Real;

            SyntheticLibrary.WriteMedia(SceneRenderTool.MediaRoot);
            IReadOnlyList<SceneSystem> loaded = SyntheticLibrary.Load(Path.Combine(Motion, "theme", "motion-probe"), new ThemeChoices { ScreenWidth = 1280, ScreenHeight = 800, Variant = "a" });
            IReadOnlyList<SceneSystem> systems = EsdeCompareTool.EsdeOrder.Select(n => loaded.Single(s => s.System.Name == n)).ToList();
            var frames = rows.Where(r => r.Pts >= t0 - 0.05 && r.Pts < firstHold).GroupBy(r => r.Pts).OrderBy(g => g.Key).ToList();

            foreach (double lagMs in new[] { 0.0, 5, 10, 15 })
            {
                var data = new SceneData(systems, new Size(1280, 800)) { SystemIndex = 2, Media = new EsdeMediaFolder(SceneRenderTool.MediaRoot) };
                using var host = new SceneMotionHost(new SceneView(data, "system", TimeSpan.Zero));
                int next = 0;
                var errors = new List<double>();
                var settled = new List<double>();
                var widths = new List<double>();
                var alphas = new List<double>();
                foreach (var frame in frames)
                {
                    double t = (frame.Key - t0) * 1000 - lagMs;
                    while (next < taps.Count && (taps[next].Real - t0) * 1000 <= t)
                    {
                        host.View.Step(taps[next].Dir, TimeSpan.FromMilliseconds((taps[next].Real - t0) * 1000));
                        next++;
                    }

                    host.View.Advance(TimeSpan.FromMilliseconds(Math.Max(0, t)));
                    host.Window.UpdateLayout();
                    var mine = Items(host.View);
                    double sinceTap = next == 0 ? double.MaxValue : t - (taps[next - 1].Real - t0) * 1000;
                    foreach (Row r in frame.Where(r => r.Cx - r.W / 2 > 1 && r.Cx + r.W / 2 < 1279))
                    {
                        var match = mine.Where(m => m.Item == r.Item).OrderBy(m => Math.Abs(m.Cx - r.Cx)).FirstOrDefault();
                        if (match.Item is null) continue;
                        errors.Add(Math.Abs(match.Cx - r.Cx));
                        widths.Add(Math.Abs(match.W - r.W));
                        alphas.Add(Math.Abs(match.Alpha - r.Alpha));
                        if (sinceTap > 450) settled.Add(Math.Abs(match.Cx - r.Cx));
                    }
                }

                static string S(List<double> v) { v.Sort(); return v.Count == 0 ? "none" : FormattableString.Invariant($"median {v[v.Count / 2]:F2}, p95 {v[(int)(v.Count * 0.95)]:F2}, max {v[^1]:F2} (n={v.Count})"); }
                _output.WriteLine($"lag {lagMs} ms: centre error px {S(errors)}; settled {S(settled)}; width px {S(widths)}; opacity {S(alphas)}");
                if (lagMs != 5) continue;
                Assert.True(settled.Max() <= 1, $"settled centres off by {settled.Max():F2} px");
                Assert.True(errors[(int)(errors.Count * 0.95)] <= 3, $"moving centres off by {errors[(int)(errors.Count * 0.95)]:F2} px at p95");
                Assert.True(alphas.Max() <= 0.05, $"opacity off by {alphas.Max():F3}");
            }
        });

        private static double D(string s) => double.Parse(s, CultureInfo.InvariantCulture);

        // Each shown item's centre across the screen, its drawn width after its scale, and its opacity.
        private static List<(string Item, double Cx, double W, double Alpha)> Items(SceneView view)
        {
            var carousel = (ImageCarousel)view.Scene.Entries.Single(e => e.Element.Type == "carousel").Control!;
            var list = new List<(string, double, double, double)>();
            foreach ((int _, Control child) in carousel.Shown)
            {
                double scale = child.RenderTransform is ScaleTransform s ? s.ScaleX : 1;
                string name = view.Data.Systems[(int)child.Tag!].System.Name;
                list.Add((name, carousel.Bounds.X + child.Bounds.Center.X, child.Bounds.Width * scale, child.Opacity));
            }

            return list;
        }
    }
}
