using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // Browsing the Shaders window's presets: a virtualised parameter list, a settle before reading while the selection moves, and the rows beside prefetched - see EmuSen_Settings_Reference.md §4.48.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShaderBrowseTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ShaderBrowseTests).GetTypeInfo().Assembly);

        // A Mega Bezel SMOOTH-ADV's count: 953 declared, nine of them headings.
        private const int Big = 944;
        private const int Headings = 9;
        private const int Row = 12;

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShaderBrowseTests", Guid.NewGuid().ToString("N"));
        private readonly string _pack;

        public ShaderBrowseTests(ITestOutputHelper output)
        {
            _out = output;
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _pack = Path.Combine(_root, "Pack");
            Write("shaders/glow.slang", Shader(new[] { ("GLOW", "Glow strength", 0.25, 0.0, 1.0, 0.05), ("SCAN", "Scanline weight", 3.0, 1.0, 8.0, 1.0) }));
            WriteBig(_pack);
            for (int i = 0; i < Row; i++) Write($"row/p{i:D2}.slangp", "shaders = 1\nshader0 = ../shaders/glow.slang\n");
        }

        // big/huge.slangp: 944 parameters under nine headings, named P0001 onwards, each 0.5 in 0 to 1 by 0.05.
        internal static void WriteBig(string pack)
        {
            var many = new List<(string, string, double, double, double, double)>();
            for (int i = 0; i < Big + Headings; i++)
                many.Add(i % 106 == 0 ? ($"HEAD{i}", $"--- Section {i / 106} ---", 0, 0, 0, 1) : ($"P{i:D4}", $"Parameter {i:D4}", 0.5, 0.0, 1.0, 0.05));
            foreach ((string relative, string text) in new[] { ("shaders/many.slang", Shader(many)), ("big/huge.slangp", "shaders = 1\nshader0 = ../shaders/many.slang\n") })
            {
                Directory.CreateDirectory(Path.GetDirectoryName(Path.Combine(pack, relative))!);
                File.WriteAllText(Path.Combine(pack, relative), text);
            }
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void Write(string relative, string text)
        {
            string path = Path.Combine(_pack, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, text);
        }

        private static string Shader(IEnumerable<(string Id, string Description, double Initial, double Minimum, double Maximum, double Step)> parameters)
        {
            var text = new StringBuilder("#version 450\n");
            foreach (var p in parameters)
                text.Append(FormattableString.Invariant($"#pragma parameter {p.Id} \"{p.Description}\" {p.Initial:F2} {p.Minimum:F2} {p.Maximum:F2} {p.Step:F2}\n"));
            text.Append("#pragma stage vertex\nvoid main() { gl_Position = vec4(0.0); }\n#pragma stage fragment\nvoid main() { }\n");
            return text.ToString();
        }

        private (ShaderSettingsWindow Window, ShaderPanel Panel) Open(GraphicsConfig? config = null)
        {
            var window = new ShaderSettingsWindow(config ?? new GraphicsConfig(), null, "SNES", pack: _pack);
            window.Show();
            ShaderPanel panel = window.PanelFor("SNES");
            Pump(panel);
            UiTest.Capture(window);
            return (window, panel);
        }

        private static void Pump(ShaderPanel panel)
        {
            var limit = Stopwatch.StartNew();
            while (!panel.Reading.IsCompleted && limit.ElapsedMilliseconds < 5000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
            Dispatcher.UIThread.RunJobs();
            Assert.True(panel.Reading.IsCompleted);
        }

        private static void Wait(int milliseconds)
        {
            var clock = Stopwatch.StartNew();
            while (clock.ElapsedMilliseconds < milliseconds) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
        }

        private static int IndexOf(ShaderPanel panel, string relative) => panel.List.Models.ToList().FindIndex(e => e.Relative == relative && !e.Recent);

        private static void FocusRow(ShaderSettingsWindow window, ShaderPanel panel, int index)
        {
            panel.List.SelectedIndex = index;
            Pump(panel);
            panel.List.ScrollIntoView(index);
            window.UpdateLayout();
            Assert.True(panel.List.ContainerFromIndex(index)!.Focus(NavigationMethod.Directional));
        }

        // Until the background reads finish, never more than the limit at once.
        private static void Prefetched(ShaderSettingsWindow window)
        {
            var clock = Stopwatch.StartNew();
            do
            {
                Assert.InRange(window.PrefetchesRunning, 0, ShaderSettingsWindow.PrefetchLimit);
                Dispatcher.UIThread.RunJobs();
                Thread.Sleep(1);
            }
            while (window.PrefetchesRunning > 0 && clock.ElapsedMilliseconds < 5000);
        }

        private static InputElement? Focused(Window window) => window.FocusManager!.GetFocusedElement() as InputElement;

        private static string? TextOf(ShaderPanel panel, string name) => panel.GetVisualDescendants().OfType<TextBlock>().Single(t => t.Name == name).Text;

        [Fact]
        public Task A_944_parameter_preset_builds_only_the_rows_in_view_and_its_last_slider_is_reached_and_moved() => Session.Dispatch(() =>
        {
            (ShaderSettingsWindow window, ShaderPanel panel) = Open();
            panel.List.SelectedIndex = IndexOf(panel, "big/huge.slangp");
            Pump(panel);
            UiTest.AssertLaidOut(window, "shaders-browse-944-top");

            Assert.Equal(Big, panel.Parameters.Count);
            Assert.Equal(Headings, panel.ParameterList.ItemsSource!.Cast<object>().Count(o => o is string));
            Assert.StartsWith($"{Big} parameters.", panel.GetVisualDescendants().OfType<HintText>().Single(h => h.Text?.Contains("parameters") == true).Text);
            int realised = panel.Sliders.Count();
            _out.WriteLine($"{realised} of {Big} rows realised at the top");
            Assert.InRange(realised, 1, 40);
            Assert.Equal("--- Section 0 ---", panel.ParameterList.ItemsSource!.Cast<object>().First());
            Assert.Equal("SNES.Parameter.P0001", panel.Sliders.First().Name);

            // Scrolled to the end as a thumb drag would, then the last row's slider moved by a key.
            ScrollViewer scroll = panel.ParameterList.GetVisualDescendants().OfType<ScrollViewer>().First();
            SliderItem last = panel.Parameters[^1];
            int passes = 0;
            for (; passes < 10 && panel.ParameterList.RowFor(last) is null; passes++)
            {
                scroll.Offset = new Vector(0, scroll.Extent.Height);
                UiTest.Capture(window);
            }
            _out.WriteLine($"the end reached in {passes} scroll(s): extent {scroll.Extent.Height:F0}, offset {scroll.Offset.Y:F0}");
            UiTest.AssertLaidOut(window, "shaders-browse-944-end");
            SliderRow row = panel.ParameterList.RowFor(last) ?? throw new InvalidOperationException("The last parameter has no row at the end of the list.");
            Assert.Equal($"SNES.Parameter.{last.Tag}", row.Name);
            Assert.InRange(panel.Sliders.Count(), 1, 40);
            ShaderSettingsWindowTests.Press(row, Key.Right);
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", "slang:big/huge.slangp")[(string)last.Tag!]);
            Assert.Equal(0.55, last.Value, 6);
            Assert.True(panel.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "SNES.ResetShader").IsEnabled);

            // Reset All reaches rows not realised: back at the top, and at the bottom again, every value is its default.
            panel.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "SNES.ResetShader").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Assert.All(panel.Parameters, p => Assert.True(p.IsDefault));
            Assert.True(panel.ParameterList.Reveal(last)!.IsDefault);
            Assert.Empty(GraphicsConfig.Load().ShaderParameters);
            window.Close();
        }, default);

        // Down from a slider reaches the next row's Reset, then its slider, the list scrolling as it goes and building only what comes into view.
        [Fact]
        public Task The_pad_walks_down_a_long_parameter_list_row_by_row() => Session.Dispatch(() =>
        {
            (ShaderSettingsWindow window, ShaderPanel panel) = Open();
            panel.List.SelectedIndex = IndexOf(panel, "big/huge.slangp");
            Pump(panel);
            UiTest.Capture(window);
            Slider first = panel.Sliders.First().GetVisualDescendants().OfType<Slider>().Single();
            Assert.True(first.Focus(NavigationMethod.Directional));

            const int rows = 60;
            int most = 0;
            for (int i = 0; i < rows; i++)
            {
                PadWindowRouter.Send(window, UiButton.Down);
                PadWindowRouter.Send(window, UiButton.Down);
                window.UpdateLayout();
                most = Math.Max(most, panel.Sliders.Count());
            }
            SliderRow at = Assert.IsType<Slider>(Focused(window)).FindAncestorOfType<SliderRow>()!;
            SliderItem expected = panel.Parameters[rows];
            _out.WriteLine($"after {rows} rows the focus is on {at.Label}; at most {most} rows realised");
            Assert.Equal(expected.Label, at.Label);
            Assert.InRange(most, 1, 40);
            UiTest.AssertLaidOut(window, "shaders-browse-944-walked");

            PadWindowRouter.Send(window, UiButton.Right);
            Assert.Equal(expected.DefaultValue + expected.Step, expected.Value, 6);
            PadWindowRouter.Send(window, UiButton.Up);
            Assert.Equal($"Reset {expected.Label}", Avalonia.Automation.AutomationProperties.GetName((Control)Focused(window)!));
            window.Close();
        }, default);

        // A held pad passes over rows; only where it stops is read, and the name and path follow every row at once.
        [Fact]
        public Task A_fast_walk_reads_only_where_it_stops() => Session.Dispatch(() =>
        {
            (ShaderSettingsWindow window, ShaderPanel panel) = Open();
            int start = IndexOf(panel, "row/p00.slangp");
            FocusRow(window, panel, start);
            Wait(400);
            int loads = panel.Loads, reads = window.ReadsStarted - window.PrefetchesStarted;

            for (int i = 1; i <= 6; i++)
            {
                PadWindowRouter.Send(window, UiButton.Down);
                Dispatcher.UIThread.RunJobs();
                Assert.Equal($"p{i:D2}", TextOf(panel, "SNES.ShaderName"));
                Assert.StartsWith($"row/p{i:D2}.slangp\nin ", TextOf(panel, "SNES.ShaderPath"));
                Assert.Empty(panel.Parameters);
            }
            Assert.Equal(loads, panel.Loads);
            Assert.Equal(reads, window.ReadsStarted - window.PrefetchesStarted);

            Pump(panel);
            UiTest.Capture(window);
            Assert.Equal(loads + 1, panel.Loads);
            Assert.Equal(reads + 1, window.ReadsStarted - window.PrefetchesStarted);
            Assert.Equal("p06", panel.Shown?.Name);
            Assert.Equal(new[] { "Glow strength", "Scanline weight" }, panel.Parameters.Select(p => p.Label));
            Assert.NotEmpty(panel.Sliders);
            window.Close();
        }, default);

        // Use, Enter and a click do not wait for the settle; Use of a preset whose sliders are not shown yet still applies it.
        [Fact]
        public Task Use_Enter_and_a_click_do_not_wait_for_the_settle() => Session.Dispatch(() =>
        {
            var told = new List<string>();
            var window = new ShaderSettingsWindow(new GraphicsConfig(), told.Add, "SNES", pack: _pack) { Settle = TimeSpan.FromSeconds(30) };
            window.Show();
            ShaderPanel panel = window.PanelFor("SNES");
            Pump(panel);
            FocusRow(window, panel, IndexOf(panel, "row/p00.slangp"));

            PadWindowRouter.Send(window, UiButton.Down);
            Assert.Equal("p01", panel.Shown?.Name);
            Assert.False(panel.Reading.IsCompleted);
            int loads = panel.Loads;
            PadWindowRouter.Send(window, UiButton.Accept);
            Assert.Equal(loads + 1, panel.Loads);
            Assert.Equal("slang:row/p01.slangp", GraphicsConfig.Load().Value("SNES", GraphicsSettingsWindow.ScreenFilterKey));
            Assert.Equal(new[] { "SNES" }, told);
            Pump(panel);
            Assert.Equal(2, panel.Parameters.Count);

            // A click on a row, which a mouse's press selects: shown at once, never waiting the thirty seconds.
            int target = IndexOf(panel, "row/p05.slangp");
            panel.List.ScrollIntoView(target);
            window.UpdateLayout();
            Control row = panel.List.ContainerFromIndex(target)!;
            Point at = row.TranslatePoint(new Point(row.Bounds.Width / 2, row.Bounds.Height / 2), window)!.Value;
            window.MouseDown(at, MouseButton.Left);
            window.MouseUp(at, MouseButton.Left);
            Assert.Equal("p05", panel.Shown?.Name);
            Pump(panel);
            Assert.Equal(2, panel.Parameters.Count);

            // Moved by a key, then the focus taken off the list to the sliders' side: the settle ends there.
            FocusRow(window, panel, target);
            PadWindowRouter.Send(window, UiButton.Down);
            Assert.False(panel.Reading.IsCompleted);
            PadWindowRouter.Send(window, UiButton.Right);
            Pump(panel);
            Assert.Equal("p06", panel.Shown?.Name);
            Assert.Equal(2, panel.Parameters.Count);
            window.Close();
        }, default);

        // After a settle the preset rows either side are read in the background, at most two at once, and the next step finds its row read.
        [Fact]
        public Task The_rows_beside_a_settled_one_are_prefetched_and_no_further() => Session.Dispatch(() =>
        {
            (ShaderSettingsWindow window, ShaderPanel panel) = Open();
            int start = IndexOf(panel, "row/p05.slangp");
            FocusRow(window, panel, start);
            Prefetched(window);

            Assert.True(window.IsRead("row/p04.slangp"));
            Assert.True(window.IsRead("row/p06.slangp"));
            Assert.Equal(2, window.PrefetchesStarted);
            foreach (string far in new[] { "row/p03.slangp", "row/p07.slangp", "big/huge.slangp" }) Assert.False(window.IsRead(far), far);

            int reads = window.ReadsStarted;
            PadWindowRouter.Send(window, UiButton.Down);
            Pump(panel);
            Assert.Equal("p06", panel.Shown?.Name);
            Assert.Equal(2, panel.Parameters.Count);
            Prefetched(window);
            Assert.Equal(reads + 1, window.ReadsStarted);
            Assert.Equal(3, window.PrefetchesStarted);
            Assert.True(window.IsRead("row/p07.slangp"));
            Assert.False(window.IsRead("row/p08.slangp"));
            window.Close();
        }, default);
    }
}
