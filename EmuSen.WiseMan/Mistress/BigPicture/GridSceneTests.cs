using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Animation.Easings;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The grid in the scene: ES-DE's measured layout, steps, slides, repeats and ends, and Art Book Next's grid against ES-DE's still - see EmuSen_BigPicture.md §16.
    public class GridSceneTests
    {
        private const int W = 640, H = 400;

        private readonly ITestOutputHelper _out;

        public GridSceneTests(ITestOutputHelper output) => _out = output;

        private static TimeSpan Ms(double ms) => TimeSpan.FromMilliseconds(ms);

        // Four columns of 0.2 x 0.3, spacing 0.02 x 0.03, scale 1.2, two whole rows, and a metadata text above.
        private static string Grid(string extra = "") =>
            "<grid name=\"g\"><pos>0 0.1</pos><size>1 0.8</size><itemSize>0.2 0.3</itemSize><itemSpacing>0.02 0.03</itemSpacing><itemScale>1.2</itemScale>" +
            $"<imageType>cover</imageType><imageFit>fill</imageFit><unfocusedItemOpacity>0.4</unfocusedItemOpacity>{extra}</grid>" +
            "<text name=\"n\"><pos>0 0</pos><size>1 0.08</size><metadata>name</metadata><fontSize>0.05</fontSize><color>FFFFFF</color></text>";

        private static SceneData Data(string grid, int game = 0)
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme($"<view name=\"gamelist\">{grid}</view>");
            var systems = SyntheticLibrary.Systems.Select(s => new SceneSystem(s.System, theme.Load(new ThemeChoices { ScreenWidth = W, ScreenHeight = H }, s.System), SyntheticLibrary.Games(s.System, s.Extension))).ToList();
            return new SceneData(systems, new Size(W, H)) { SystemIndex = 1, GameIndex = game, Media = new SceneAssets.Media(), Motion = SceneMotion.Esde };
        }

        private static RenderedFrame Static(SceneData data) => SceneAssets.Render(SceneBuilder.Build(data.System.Theme.View("gamelist"), data));

        private static ImageGrid Control(SceneView view) => view.Scene.Entries.Select(e => e.Control).OfType<ImageGrid>().Single();

        [Fact]
        public void The_layout_is_ES_DE_s_measured_one()
        {
            var view = new SceneView(Data(Grid()), "gamelist", TimeSpan.Zero);
            GridGeometry g = view.Grid()!.Value;
            Assert.Equal(4, g.Columns);
            Assert.Equal(2, g.WholeRows);
            Assert.Equal(12.8, g.CellRect(0, 0).X, 3);
            Assert.Equal(12, g.CellRect(0, 0).Y, 3);
            Assert.Equal(3, g.Rows);
        }

        // A step within a row eases the focus on a 250 ms quadratic ease-out and settles on the static picture, pixel for pixel.
        [Fact]
        public Task A_step_eases_the_focus_and_settles_on_the_static_picture() => UiTest.Run(() =>
        {
            SceneData data = Data(Grid());
            var view = new SceneView(data, "gamelist", TimeSpan.Zero);
            using var host = new SceneMotionHost(view);
            view.Press(1, TimeSpan.Zero);
            view.Release(Ms(10));
            host.At(Ms(125));
            ImageGrid grid = Control(view);
            Assert.Equal((0, 1), (grid.FocusFrom, grid.SelectedIndex));
            Assert.Equal(new QuadraticEaseOut().Ease(0.5), grid.FocusProgress, 3);
            Assert.True(view.NextChange(Ms(125)) == Ms(125));
            RenderedFrame settled = host.At(Ms(260));
            Assert.Null(view.NextChange(Ms(260)));
            Assert.Equal(0, SceneAssets.Differing(settled, Static(data with { GameIndex = 1 })));
        });

        // ES-DE's scroll: nothing until the selection passes the last whole row, then a 250 ms slide that keeps it on the bottom row.
        [Fact]
        public Task A_row_past_the_last_shown_slides_into_view_and_settles_on_the_static_picture() => UiTest.Run(() =>
        {
            SceneData data = Data(Grid());
            var view = new SceneView(data, "gamelist", TimeSpan.Zero);
            using var host = new SceneMotionHost(view);
            view.Press(1, TimeSpan.Zero, vertical: true);
            view.Release(Ms(10));
            host.At(Ms(300));
            Assert.Equal((4, 0.0), (view.Index, Control(view).ScrollRow));
            view.Press(1, Ms(300), vertical: true);
            view.Release(Ms(310));
            host.At(Ms(425));
            Assert.Equal(8, view.Index);
            Assert.Equal(new QuadraticEaseOut().Ease(0.5), Control(view).ScrollRow, 3);
            RenderedFrame settled = host.At(Ms(600));
            Assert.Equal(1, Control(view).ScrollRow);
            Assert.Equal(0, SceneAssets.Differing(settled, Static(data with { GameIndex = 8 })));
        });

        // ES-DE's ends: across a row's end, a tap wraps at the list's ends and a hold stops, up and down stop, down into a short last row takes its last item.
        [Fact]
        public void The_ends_are_ES_DE_s()
        {
            var view = new SceneView(Data(Grid()), "gamelist", TimeSpan.Zero);
            view.Step(-1, Ms(0));
            Assert.Equal(11, view.Index);
            view.Step(1, Ms(10));
            Assert.Equal(0, view.Index);
            view.Step(3, Ms(20));
            view.Step(1, Ms(30));
            Assert.Equal(4, view.Index);
            view.Press(-1, Ms(40), vertical: true);
            view.Release(Ms(41));
            Assert.Equal(0, view.Index);
            view.Press(-1, Ms(50), vertical: true);
            view.Release(Ms(51));
            Assert.Equal(0, view.Index);

            var shorter = new SceneView(Data(Grid()) is var d ? d with { Systems = d.Systems.Select(s => s with { Games = s.Games.Take(10).ToList() }).ToList() } : d, "gamelist", TimeSpan.Zero);
            shorter.Step(7, Ms(0));
            shorter.Press(1, Ms(10), vertical: true);
            shorter.Release(Ms(11));
            Assert.Equal(9, shorter.Index);
            shorter.Press(1, Ms(20), vertical: true);
            shorter.Release(Ms(21));
            Assert.Equal(9, shorter.Index);

            var held = new SceneView(Data(Grid(), game: 9), "gamelist", TimeSpan.Zero);
            held.Press(1, Ms(0));
            held.Advance(Ms(2500));
            Assert.Equal(11, held.Index);
        }

        // A step taken while the last still moves starts from where the two items are: the one left keeps its part-grown focus as its level.
        [Fact]
        public void A_step_while_moving_starts_from_the_items_current_focus()
        {
            var view = new SceneView(Data(Grid()), "gamelist", TimeSpan.Zero);
            view.Step(1, Ms(0));
            view.Step(1, Ms(100));
            view.Advance(Ms(100));
            ImageGrid grid = Control(view);
            Assert.Equal((1, 2), (grid.FocusFrom, grid.SelectedIndex));
            Assert.Equal(new QuadraticEaseOut().Ease(0.4), grid.FocusFromLevel, 3);
            Assert.Equal(0, grid.FocusToLevel, 3);
            view.Step(-1, Ms(150));
            view.Advance(Ms(150));
            double left = new QuadraticEaseOut().Ease(0.2);
            Assert.Equal((2, 1), (Control(view).FocusFrom, Control(view).SelectedIndex));
            Assert.Equal(new QuadraticEaseOut().Ease(0.4) * (1 - left), Control(view).FocusToLevel, 3);
        }

        // The mapping's units: a -1 axis of itemSize is the other axis's pixels, corner radii are fractions of the width, a page is the whole rows shown.
        [Fact]
        public void Item_sizes_corner_radii_and_pages_are_in_ES_DE_s_units()
        {
            var view = new SceneView(Data(Grid("<imageCornerRadius>0.05</imageCornerRadius>").Replace("<itemSize>0.2 0.3</itemSize>", "<itemSize>0.2 -1</itemSize>")), "gamelist", TimeSpan.Zero);
            Assert.Equal(128, Control(view).ItemSize.Width, 3);
            Assert.Equal(128, Control(view).ItemSize.Height, 3);
            Assert.Equal(0.05 * W, Control(view).ImageCornerRadius, 3);
            var tall = new SceneView(Data(Grid().Replace("<itemSize>0.2 0.3</itemSize>", "<itemSize>-1 0.3</itemSize>")), "gamelist", TimeSpan.Zero);
            Assert.Equal(120, Control(tall).ItemSize.Width, 3);
            Assert.Equal(120, Control(tall).ItemSize.Height, 3);
            var page = new SceneView(Data(Grid()), "gamelist", TimeSpan.Zero);
            Assert.Equal(8, EmuSen.Mistress.BigPicture.ThemedLibrary.PageSize(page));
        }

        // A held direction steps at the press, at 500 ms and then every 200 ms, with no faster tier, on either axis.
        [Fact]
        public void A_held_direction_repeats_at_500_then_200_ms_with_no_faster_tier()
        {
            var many = Data(Grid()) is var d ? d with { Systems = d.Systems.Select(s => s with { Games = Enumerable.Range(0, 200).Select(i => new SceneGame($"G{i}", $"/g/{i}.sfc")).ToList() }).ToList() } : d;
            foreach (bool vertical in new[] { false, true })
            {
                var view = new SceneView(many, "gamelist", TimeSpan.Zero);
                var steps = new List<TimeSpan>();
                view.Stepped += (_, _) => steps.Add(view.Now);
                view.Press(1, TimeSpan.Zero, vertical);
                for (int t = 0; t <= 4000; t += 10) view.Advance(Ms(t));
                Assert.Equal(new[] { 0.0, 500, 700, 900 }, steps.Take(4).Select(s => s.TotalMilliseconds));
                Assert.Equal(1 + 1 + (4000 - 500) / 200, steps.Count);
                Assert.Equal(steps.Count, vertical ? view.Index / 4 : view.Index);
            }
        }

        // ES-DE fades the game's metadata out from the first repeat of a held direction in the grid, as in the list, and back in at release.
        [Fact]
        public void A_held_grid_fades_the_metadata_out_from_the_first_repeat()
        {
            var view = new SceneView(Data(Grid()), "gamelist", TimeSpan.Zero);
            view.Press(1, TimeSpan.Zero);
            view.Advance(Ms(400));
            Avalonia.Controls.Control text = view.Scene.Entries.First(e => e.Element.Type == "text").Control!;
            Assert.Equal(1, text.Opacity, 3);
            view.Advance(Ms(500 + 149));
            Assert.Equal(0, view.Scene.Entries.First(e => e.Element.Type == "text").Control!.Opacity, 2);
            view.Release(Ms(700));
            view.Advance(Ms(700 + 150));
            Assert.Equal(1, view.Scene.Entries.First(e => e.Element.Type == "text").Control!.Opacity, 2);
        }

        // instant item transitions change the focus at once; instant row transitions jump the rows at once.
        [Fact]
        public void Instant_transitions_do_not_animate()
        {
            var view = new SceneView(Data(Grid("<itemTransitions>instant</itemTransitions><rowTransitions>instant</rowTransitions>"), game: 4), "gamelist", TimeSpan.Zero);
            view.Press(1, TimeSpan.Zero, vertical: true);
            view.Release(Ms(1));
            view.Advance(Ms(1));
            Assert.Equal((1.0, 1.0), (Control(view).FocusProgress, Control(view).ScrollRow));
            Assert.Null(view.NextChange(Ms(1)));
        }

        // P50: Art Book Next's gamelist-grid-cover at 1280x800 against ES-DE 3.4.1's still, each synthetic cover's flat interior found the same way in both.
        [ArtBookNextFact]
        public Task Art_Book_Next_s_grid_matches_ES_DE_s_still() => UiTest.Run(() =>
        {
            string media = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "EmuSenGridMedia");
            SyntheticLibrary.WriteMedia(media);
            var choices = new ThemeChoices { Variant = "gamelist-grid-cover", ColorScheme = "dark-screenshots", ScreenWidth = 1280, ScreenHeight = 800 };
            var systems = SyntheticLibrary.Load(ArtBookNextFactAttribute.Folder, choices)
                .Select(s => s with { Games = s.Games.OrderByDescending(g => g.Favorite).ThenBy(g => g.Name, StringComparer.OrdinalIgnoreCase).ToList() }).ToList();
            int snes = systems.FindIndex(s => s.System.Name == "snes");
            var data = new SceneData(systems, new Size(1280, 800)) { SystemIndex = snes, GameIndex = 0, Media = new EsdeMediaFolder(media), ShowClock = false };
            var view = new SceneView(data, "gamelist", TimeSpan.Zero);
            using var host = new SceneMotionHost(view);
            Report("rest", Boxes(host.At(Ms(0)), systems[snes].Games), Rest);
            view.Press(1, Ms(10), vertical: true);
            view.Release(Ms(20));
            view.Press(1, Ms(400), vertical: true);
            view.Release(Ms(410));
            Report("after two downs", Boxes(host.At(Ms(1000)), systems[snes].Games), TwoDowns);
        });

        private void Report(string state, Dictionary<string, (int, int, int, int)> ours, Dictionary<string, (int, int, int, int)> esde)
        {
            int worst = 0, worstTwoRows = 0;
            foreach ((string name, (int x0, int y0, int x1, int y1) e) in esde)
            {
                Assert.True(ours.TryGetValue(name, out var o), $"{state}: {name} not found");
                int err = new[] { Math.Abs(o.Item1 - e.x0), Math.Abs(o.Item2 - e.y0), Math.Abs(o.Item3 - e.x1), Math.Abs(o.Item4 - e.y1) }.Max();
                _out.WriteLine($"{state} {name,-15} ES-DE {e} Mistress {o} worst {err}");
                worst = Math.Max(worst, err);
                if (e.y0 < 560) worstTwoRows = Math.Max(worstTwoRows, err);
            }
            _out.WriteLine($"{state}: worst {worst} px, first two rows {worstTwoRows} px");
            Assert.True(worstTwoRows <= 2, $"{state}: {worstTwoRows} px in the first two rows");
        }

        // Each cover's flat interior, found by its colour's ratios at any opacity over black, as the ES-DE still was measured (abnframe.py).
        private static Dictionary<string, (int, int, int, int)> Boxes(RenderedFrame f, IReadOnlyList<SceneGame> games)
        {
            var result = new Dictionary<string, (int, int, int, int)>();
            IReadOnlyList<SceneGame> titles = SyntheticLibrary.Games(SyntheticTheme.Snes, ".sfc");
            foreach (SceneGame g in games)
            {
                Color c = SyntheticLibrary.Colour(titles.ToList().FindIndex(t => t.Name == g.Name), "covers");
                var xs = new List<int>(); var ys = new List<int>();
                for (int y = 0; y < f.Height; y++)
                    for (int x = 0; x < f.Width; x++)
                    {
                        int i = (y * f.Width + x) * 4;
                        double r = f.Rgba[i], gg = f.Rgba[i + 1], b = f.Rgba[i + 2];
                        if (!(Math.Abs(gg - 99) <= 3 || Math.Abs(gg - 198) <= 3)) continue;
                        if (Math.Abs(r / gg - c.R / 198.0) < 0.025 && Math.Abs(b / gg - c.B / 198.0) < 0.025) { xs.Add(x); ys.Add(y); }
                    }
                if (xs.Count < 200) continue;
                int[] colh = new int[xs.Max() - xs.Min() + 1], rowh = new int[ys.Max() - ys.Min() + 1];
                foreach (int x in xs) colh[x - xs.Min()]++;
                foreach (int y in ys) rowh[y - ys.Min()]++;
                int[] cx = Enumerable.Range(0, colh.Length).Where(k => colh[k] >= 0.2 * colh.Max()).ToArray(), cy = Enumerable.Range(0, rowh.Length).Where(k => rowh[k] >= 0.2 * rowh.Max()).ToArray();
                result[g.Name] = (xs.Min() + cx.Min(), ys.Min() + cy.Min(), xs.Min() + cx.Max() + 1, ys.Min() + cy.Max() + 1);
            }
            return result;
        }

        // ES-DE 3.4.1's still of the same state (~/.cache/emusen/bigpicture/grid/results.md), the flat interior of each cover.
        private static readonly Dictionary<string, (int, int, int, int)> Rest = new()
        {
            ["Aurora Drift"] = (88, 97, 296, 375), ["Cobalt Harbor"] = (393, 96, 566, 328), ["Ember Circuit"] = (714, 96, 887, 328), ["Granite Choir"] = (1034, 96, 1207, 328),
            ["Kestrel Run"] = (393, 338, 566, 570), ["Brass Lantern"] = (714, 338, 887, 570), ["Dune Relay"] = (1034, 338, 1207, 570),
            ["Fable of Tiles"] = (73, 580, 246, 799), ["Hollow Comet"] = (393, 580, 566, 799), ["Juniper Vault"] = (714, 580, 887, 799), ["Lumen Garden"] = (1034, 580, 1207, 799),
        };

        private static readonly Dictionary<string, (int, int, int, int)> TwoDowns = new()
        {
            ["Fable of Tiles"] = (88, 518, 296, 796), ["Aurora Drift"] = (73, 96, 246, 313), ["Cobalt Harbor"] = (393, 96, 566, 313), ["Ember Circuit"] = (714, 96, 887, 313),
            ["Granite Choir"] = (1034, 96, 1207, 313), ["Kestrel Run"] = (393, 323, 566, 555), ["Brass Lantern"] = (714, 323, 887, 555), ["Dune Relay"] = (1034, 323, 1207, 555),
            ["Hollow Comet"] = (393, 565, 566, 797), ["Juniper Vault"] = (714, 565, 887, 797), ["Lumen Garden"] = (1034, 565, 1207, 797),
        };
    }
}
