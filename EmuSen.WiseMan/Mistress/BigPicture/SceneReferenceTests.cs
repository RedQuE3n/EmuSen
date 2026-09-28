using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Art Book Next read in place: its pixel comments as the geometry oracle, and the count of its 172 pairs the scene maps - see EmuSen_BigPicture.md §13.6.
    public class SceneReferenceTests
    {
        private readonly ITestOutputHelper _output;

        public SceneReferenceTests(ITestOutputHelper output) => _output = output;

        private const double DesignW = 768, DesignH = 480;

        private static readonly Regex Commented = new(@"<(?<p>pos|size|maxSize|cropSize|imageMaxSize|imageCropSize)>(?<x>[-\d.]+)\s+(?<y>[-\d.]+)</\k<p>><!--\s*(?<a>[\d.]+%?)\s+(?<b>[\d.]+%?)\s*-->");

        // (property, fraction x, fraction y) to the design pixels the comment beside it gives.
        public static Dictionary<(string, string, string), (double X, double Y)> Comments(string file)
        {
            var map = new Dictionary<(string, string, string), (double, double)>();
            foreach (Match m in Commented.Matches(File.ReadAllText(file)))
            {
                static double Px(string t, double full) => t.EndsWith('%') ? double.Parse(t[..^1], CultureInfo.InvariantCulture) / 100 * full : double.Parse(t, CultureInfo.InvariantCulture);
                map[(m.Groups["p"].Value, Key(m.Groups["x"].Value), Key(m.Groups["y"].Value))] = (Px(m.Groups["a"].Value, DesignW), Px(m.Groups["b"].Value, DesignH));
            }

            return map;
        }

        private static string Key(string fraction) => Key(float.Parse(fraction, CultureInfo.InvariantCulture));

        private static string Key(float fraction) => fraction.ToString("0.0000", CultureInfo.InvariantCulture);

        [ArtBookNextFact]
        public Task Every_commented_position_and_size_is_where_the_comment_puts_it() => UiTest.Run(() =>
        {
            var comments = Comments(Path.Combine(ArtBookNextFactAttribute.Folder, "aspect-ratio-16-10.xml"));
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            var failures = new List<string>();
            var matched = new HashSet<(string, string, string)>();
            var disagree = new SortedSet<string>(StringComparer.Ordinal);
            int checks = 0;
            double worst = 0;
            string worstWhere = "";
            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                foreach (ThemeVariant variant in caps.Variants)
                {
                    IReadOnlyList<SceneSystem> systems = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder, new ThemeChoices { ScreenWidth = w, ScreenHeight = h, Variant = variant.Name });
                    foreach (string view in new[] { "system", "gamelist" })
                    {
                        SceneData data = new(systems, new Size(w, h)) { SystemIndex = 1, GameIndex = 2, Media = new SceneAssets.Media() };
                        SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
                        scene.Canvas.Measure(new Size(w, h));
                        scene.Canvas.Arrange(new Rect(0, 0, w, h));
                        foreach (SceneEntry entry in scene.Entries.Where(e => e.Control is not null))
                        {
                            Thickness m = entry.Control!.Margin;
                            Rect bounds = entry.Control.Bounds.Deflate(new Thickness(-m.Left, -m.Top, -m.Right, -m.Bottom));
                            foreach (string property in new[] { "pos", "size", "maxSize", "cropSize", "imageMaxSize", "imageCropSize" })
                            {
                                if (entry.Element.Explicit.GetValueOrDefault(property) is not PairValue { Value: var pair }) continue;
                                if (!comments.TryGetValue((property, Key(pair.X), Key(pair.Y)), out (double X, double Y) px)) continue;
                                if (entry.Element.Type == "video" && property is "size" or "maxSize" or "cropSize" && entry.Element.Explicit.Keys.Any(k => k.StartsWith("image") && k.EndsWith("Size"))) continue;
                                matched.Add((property, Key(pair.X), Key(pair.Y)));
                                if ((pair.X != 0 && Math.Abs(pair.X * DesignW - px.X) > 0.5) || (pair.Y != 0 && Math.Abs(pair.Y * DesignH - px.Y) > 0.5))
                                {
                                    disagree.Add($"{property} {pair}: the comment says {px.X} {px.Y}, the fraction gives {pair.X * DesignW:0.##} {pair.Y * DesignH:0.##}");
                                    px = (pair.X * DesignW, pair.Y * DesignH);
                                }

                                var expected = new Point(px.X / DesignW * w, px.Y / DesignH * h);
                                string where = $"{w}x{h} {variant.Name} {view} {entry.Element} {property} {pair}";
                                double error;
                                if (property == "pos")
                                {
                                    Point origin = SceneUnits.ToPoint(entry.Element.Pair("origin"));
                                    var actual = new Point(bounds.X + origin.X * bounds.Width, bounds.Y + origin.Y * bounds.Height);
                                    error = Math.Max(Math.Abs(actual.X - expected.X), Math.Abs(actual.Y - expected.Y));
                                }
                                else if (property is "size" or "cropSize" or "imageCropSize")
                                {
                                    double ex = pair.X == 0 ? bounds.Width : expected.X, ey = pair.Y == 0 ? bounds.Height : expected.Y;
                                    error = Math.Max(Math.Abs(bounds.Width - ex), Math.Abs(bounds.Height - ey));
                                }
                                else
                                {
                                    double over = Math.Max(bounds.Width - expected.X, bounds.Height - expected.Y);
                                    double touch = Math.Min(Math.Abs(bounds.Width - expected.X), Math.Abs(bounds.Height - expected.Y));
                                    error = Math.Max(Math.Max(0, over), touch);
                                }

                                checks++;
                                if (error > worst) { worst = error; worstWhere = $"{where}, bounds {bounds}, expected {expected}"; }
                                if (error > 1) failures.Add($"{where}: off by {error:F2} px, bounds {bounds}, expected {expected}");
                            }
                        }
                    }
                }
            }

            _output.WriteLine($"{checks} checks against {matched.Count} of {comments.Count} commented values; worst error {worst:F3} px at {worstWhere}");
            foreach (var unmatched in comments.Keys.Except(matched)) _output.WriteLine($"not reached by any scene: {unmatched}");
            foreach (string d in disagree) _output.WriteLine($"comment disagrees with its fraction, checked against the fraction: {d}");
            Assert.True(failures.Count == 0, string.Join("\n", failures.Distinct().Take(40)));
        });

        // P21: the pairs Art Book Next sets, over every variant, ratio and two schemes, against what SceneMapping reads.
        [ArtBookNextFact]
        public void The_pairs_Art_Book_Next_sets_that_the_scene_maps()
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            var union = new HashSet<(string Type, string Property)>();
            foreach (ThemeSystem system in ArtBookNextReferenceTests.Systems)
                foreach (ThemeVariant variant in caps.Variants)
                    foreach (string ratio in caps.AspectRatios)
                        foreach (string scheme in new[] { "dark-screenshots", "custom" })
                            union.UnionWith(ThemeInventory.SetPairs(ThemeLoader.Load(caps, system, new ThemeChoices { Variant = variant.Name, AspectRatio = ratio, ColorScheme = scheme })));

            var unmapped = union.Where(p => !SceneMapping.IsMapped(p.Type, p.Property, q => ThemeCatalog.Find(p.Type)?.Find(q) is not null)).OrderBy(p => p.Type).ThenBy(p => p.Property).ToList();
            _output.WriteLine($"{union.Count - unmapped.Count} of {union.Count} mapped");
            foreach (var group in unmapped.GroupBy(p => p.Type)) _output.WriteLine($"unmapped {group.Key}: {string.Join(" ", group.Select(p => p.Property))}");
            Assert.Equal(172, union.Count);
        }
    }
}
