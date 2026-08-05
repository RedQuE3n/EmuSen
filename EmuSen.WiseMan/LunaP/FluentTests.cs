using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;

namespace EmuSen.WiseMan.LunaP
{
    // The fluent surface - see EmuSen_LunaP.md §9.
    public class FluentTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FluentTests).GetTypeInfo().Assembly);

        private static Task Run(System.Action body) => Session.Dispatch(body, default);

        // The headline of the phase: three Grid.SetColumn calls become one Ui.Cols.
        [Fact]
        public Task Cols_assigns_columns_by_position() => Run(() =>
        {
            var label = Ui.Text("S-CPU");
            var bar = new ProgressBar();
            var value = Ui.Text("62.0%");

            Grid grid = Ui.Cols("140,*,55", label, bar, value);

            Assert.Equal(3, grid.ColumnDefinitions.Count);
            Assert.Equal(0, Grid.GetColumn(label));
            Assert.Equal(1, Grid.GetColumn(bar));
            Assert.Equal(2, Grid.GetColumn(value));
            Assert.Equal(3, grid.Children.Count);
        });

        // Positional assignment is a convenience, not a rule it imposes.
        [Fact]
        public Task An_explicit_column_beats_the_positional_one() => Run(() =>
        {
            var first = Ui.Text("a").AtColumn(2);
            var second = Ui.Text("b");

            Ui.Cols("*,*,*", first, second);

            Assert.Equal(2, Grid.GetColumn(first));
            Assert.Equal(1, Grid.GetColumn(second));
        });

        [Fact]
        public Task Column_spans_are_carried_through() => Run(() =>
        {
            var wide = Ui.Text("a").AtColumn(0, span: 3);
            Ui.Cols("*,*,*", wide);

            Assert.Equal(0, Grid.GetColumn(wide));
            Assert.Equal(3, Grid.GetColumnSpan(wide));
        });

        [Fact]
        public Task Stacks_carry_their_spacing_and_orientation() => Run(() =>
        {
            StackPanel vertical = Ui.Stack(8, Ui.Text("a"), Ui.Text("b"));
            Assert.Equal(Orientation.Vertical, vertical.Orientation);
            Assert.Equal(8, vertical.Spacing);
            Assert.Equal(2, vertical.Children.Count);

            StackPanel horizontal = Ui.Row(4, Ui.Text("a"));
            Assert.Equal(Orientation.Horizontal, horizontal.Orientation);
            Assert.Equal(4, horizontal.Spacing);
        });

        [Fact]
        public Task A_section_is_its_header_then_its_content() => Run(() =>
        {
            var content = Ui.Mono("PC=0x008123");
            StackPanel section = Ui.Section("CPU registers", content);

            Assert.Equal(2, section.Children.Count);
            Assert.Equal("CPU registers", Assert.IsType<SectionHeader>(section.Children[0]).Text);
            Assert.Same(content, section.Children[1]);
        });

        [Fact]
        public Task Docking_sets_the_attached_property_the_XAML_would() => Run(() =>
        {
            var bottom = Ui.Text("status").Dock(Dock.Bottom);
            Ui.Dock(bottom, Ui.Text("body"));

            Assert.Equal(Dock.Bottom, DockPanel.GetDock(bottom));
        });

        [Fact]
        public Task A_fluent_button_runs_its_handler() => Run(() =>
        {
            int clicks = 0;
            Button button = Ui.Button("Close", () => clicks++);

            var window = new Window { Width = 200, Height = 150, Content = button };
            window.Show();

            button.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Avalonia.Controls.Button.ClickEvent));

            Assert.Equal("Close", button.Content);
            Assert.Equal(1, clicks);
        });

        // Every one of these is the XAML attribute of the same name; that is the whole contract of the surface.
        [Fact]
        public Task Each_extension_sets_the_property_it_is_named_after() => Run(() =>
        {
            Assert.Equal(new Thickness(12), Ui.Text("a").Margin(12).Margin);
            Assert.Equal(new Thickness(8, 4), Ui.Text("a").Margin(8, 4).Margin);
            Assert.Equal(new Thickness(1, 2, 3, 4), Ui.Text("a").Margin(1, 2, 3, 4).Margin);

            Assert.Equal(320, Ui.Text("a").Width(320).Width);
            Assert.Equal(160, Ui.Text("a").Height(160).Height);
            Assert.Equal(320, Ui.Text("a").MaxHeight(320).MaxHeight);

            TextBlock sized = Ui.Text("a").MinSize(360, 240);
            Assert.Equal(360, sized.MinWidth);
            Assert.Equal(240, sized.MinHeight);

            Assert.Equal(HorizontalAlignment.Stretch, Ui.Text("a").Grow().HorizontalAlignment);
            Assert.Equal(HorizontalAlignment.Right, Ui.Text("a").Right().HorizontalAlignment);
            Assert.Equal(HorizontalAlignment.Left, Ui.Text("a").Left().HorizontalAlignment);
            Assert.Equal(VerticalAlignment.Center, Ui.Text("a").Center().VerticalAlignment);

            Assert.False(Ui.Text("a").Visible(false).IsVisible);
            Assert.Equal(FontWeight.Bold, Ui.Text("a").Bold().FontWeight);
            Assert.Equal(11, Ui.Text("a").FontSize(11).FontSize);
            Assert.Equal(TextWrapping.Wrap, Ui.Text("a").Wrap().TextWrapping);
            Assert.Equal(6, Ui.Stack().Spacing(6).Spacing);
        });

        // Chaining has to hand back the concrete type, or the second call in a chain will not compile.
        [Fact]
        public Task Chaining_preserves_the_concrete_type() => Run(() =>
        {
            MonoText mono = Ui.Mono("A=0x00").Margin(4).Wrap().Bold();
            Assert.Equal(new Thickness(4), mono.Margin);

            MeterList list = new MeterList().Margin(2).Grow();
            Assert.Equal(HorizontalAlignment.Stretch, list.HorizontalAlignment);
        });

        [Fact]
        public Task Buttons_makes_a_right_aligned_bar() => Run(() =>
        {
            ButtonBar bar = Ui.Buttons(Ui.Button("Apply", () => { }), Ui.Button("Close", () => { }));

            var window = new Window { Width = 300, Height = 150, Content = bar };
            window.Show();

            Assert.Equal(HorizontalAlignment.Right, bar.HorizontalAlignment);
            Assert.Equal(2, bar.CountParts<Button>());
        });
    }
}
