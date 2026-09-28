using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;

namespace EmuSen.WiseMan.Fixtures
{
    // The fit audit catches what it is for, each fault made on purpose on a framed sheet, and passes the same sheet laid out properly - see EmuSen_Settings_Reference.md §4.81.
    public class FitAuditTests
    {
        private static (ToolWindow Host, SheetLayer Layer) Host()
        {
            var layer = new SheetLayer { PresentsWindows = true, MenuLook = true };
            var host = new ToolWindow { Width = 1280, Height = 800, Content = new Grid { Children = { new Border(), layer } } };
            host.Show();
            return (host, layer);
        }

        private static List<string> Audit(Control content, string title = "Audit", string? footer = null, System.Action? before = null)
        {
            (ToolWindow host, SheetLayer layer) = Host();
            var window = new ToolWindow { Title = title, Content = content };
            if (footer is not null) MenuLook.SetFooter(window, footer);
            _ = SheetLayer.Show(window, host);
            for (int i = 0; i < 3; i++) { Dispatcher.UIThread.RunJobs(); UiTest.Capture(host); }
            before?.Invoke();
            List<string> faults = FitAudit.Check(layer.SheetOf(window)!);
            window.Close();
            host.Close();
            return faults;
        }

        [Fact]
        public Task A_sheet_that_fits_passes() => UiTest.Run(() =>
        {
            var content = new StackPanel { Spacing = 8, Children = { new TextBlock { Text = "A line that fits", TextWrapping = TextWrapping.Wrap }, new Button { Content = "Close" } } };
            Assert.Empty(Audit(content));
        });

        [Fact]
        public Task A_picture_over_words_is_an_overlap() => UiTest.Run(() =>
        {
            var grid = new Grid { Children = { new Image { Width = 300, Height = 200, Source = null }, new TextBlock { Text = "Words under the picture" } } };
            ((Image)grid.Children[0]).Source = new Avalonia.Media.Imaging.WriteableBitmap(new PixelSize(4, 4), new Vector(96, 96));
            Assert.Contains(Audit(grid), f => f.StartsWith("overlap:"));
        });

        [Fact]
        public Task A_control_wider_than_the_panel_is_past_it() => UiTest.Run(() =>
        {
            var content = new StackPanel { Children = { new Border { Width = 3000, Height = 40, Background = Brushes.Gray, Child = new Button { Content = "Far away", HorizontalAlignment = HorizontalAlignment.Right } } } };
            Assert.Contains(Audit(content), f => f.StartsWith("past its"));
        });

        [Fact]
        public Task Words_that_run_past_their_box_or_end_in_an_ellipsis_are_cut() => UiTest.Run(() =>
        {
            string words = string.Concat(Enumerable.Repeat("A long line of words ", 20));
            var content = new StackPanel { Children = { new TextBlock { Text = words, Width = 200 }, new TextBlock { Text = words, Width = 200, TextTrimming = TextTrimming.CharacterEllipsis } } };
            List<string> faults = Audit(content);
            Assert.Contains(faults, f => f.StartsWith("runs past its box"));
            Assert.Contains(faults, f => f.StartsWith("cut with an ellipsis"));
        });

        [Fact]
        public Task A_row_scrolled_below_a_list_is_reachable_but_text_past_its_side_is_not() => UiTest.Run(() =>
        {
            var list = new ListBox { Height = 100, ItemsSource = Enumerable.Range(0, 20).Select(i => $"Row {i}").ToArray() };
            Assert.Empty(Audit(new StackPanel { Children = { list } }));
            var wide = new ScrollViewer { Height = 100, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Auto, Content = new Border { Width = 3000, Height = 40 } };
            Assert.Contains(Audit(new StackPanel { Children = { wide } }), f => f.StartsWith("past its"));
        });

        [Fact]
        public Task A_list_that_scrolls_in_less_than_two_rows_is_too_little_room() => UiTest.Run(() =>
        {
            var list = new ListBox { Height = 60, ItemsSource = Enumerable.Range(0, 20).Select(i => $"Row {i}").ToArray() };
            Assert.Contains(Audit(new StackPanel { Children = { list } }), f => f.StartsWith("scrolls in too little room"));
        });

        [Fact]
        public Task A_title_or_footer_too_long_for_the_panel_is_cut() => UiTest.Run(() =>
        {
            string words = string.Concat(Enumerable.Repeat("A very long name indeed ", 8));
            Assert.Contains(Audit(new TextBlock { Text = "Short" }, title: words), f => f.StartsWith("title cut"));
            Assert.Contains(Audit(new TextBlock { Text = "Short" }, footer: string.Concat(Enumerable.Repeat(words, 10))), f => f.StartsWith("footer cut"));
        });

        [Fact]
        public Task A_list_cut_at_its_foot_with_no_fade_shows_a_part_row() => UiTest.Run(() =>
        {
            var list = new ListBox { Height = 150, ItemsSource = Enumerable.Range(0, 20).Select(i => $"Row {i}").ToArray() };
            Assert.Empty(Audit(new StackPanel { Children = { list } }));
            var plain = new ListBox { Height = 150, ItemsSource = Enumerable.Range(0, 20).Select(i => $"Row {i}").ToArray() };
            List<string> faults = Audit(new StackPanel { Children = { plain } }, before: () => Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(plain).OfType<ScrollViewer>().First().SetValue(MenuLook.FadesBottomProperty, false));
            Assert.Contains(faults, f => f.StartsWith("cut at a scrolling edge with no fade"));
        });

        [Fact]
        public Task Words_or_a_drawing_s_labels_too_small_to_read_at_a_distance_are_flagged() => UiTest.Run(() =>
        {
            Assert.Empty(Audit(new TextBlock { Text = "Big enough", FontSize = FitAudit.SmallestText }));
            Assert.Contains(Audit(new TextBlock { Text = "Small print", FontSize = 12 }), f => f.StartsWith("too small to read"));
            var squeezed = new ControllerDiagram { Layout = ControllerLayout.Nintendo64, Height = 260, CompactLabels = false };
            Assert.Contains(Audit(squeezed), f => f.StartsWith("too small to read: ControllerDiagram"));
        });
    }
}
