using System;
using EmuSen.WiseMan.LunaP;
using EmuSen.LunaP.Windowing;
using EmuSen.LunaP.Controls;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // The GUI half of `vstop` - see `man vstop`.
    public class VstopWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(VstopWindowTests).GetTypeInfo().Assembly);

        private readonly string _root;

        public VstopWindowTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenVstopWindowTests", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // The window builds its own tree now, so these come from the visual tree rather than a XAML namescope.
        private static string Text(VstopWindow w, string name) => w.FindNamed<TextBlock>(name).Text!;

        [Fact]
        public Task The_window_fills_in_every_section_on_open() => Session.Dispatch(() =>
        {
            GC.Collect(); // so the GC-derived sections have real numbers - see `man vstop`

            var window = new VstopWindow();
            window.Show();

            Assert.Contains("pid", Text(window, "HeaderText"));
            Assert.Contains(".NET", Text(window, "HostText"));
            Assert.Contains("Working set", Text(window, "MemoryText"));
            Assert.Contains("Collections", Text(window, "GcText"));
            Assert.Contains("OS threads", Text(window, "ThreadsText"));

            // CPU, machine memory, heap fragmentation, thread pool.
            Assert.Equal(4, window.FindNamed<MeterList>("MetersPanel").Meters.Count);
            Assert.Equal(4, window.CountParts<MeterRow>());

            window.Close();
        }, default);

        // Asserted on the rendered bar, not the source figure: that is what a reader of the dashboard actually sees.
        [Fact]
        public Task Every_meter_bar_stays_inside_its_range() => Session.Dispatch(() =>
        {
            var window = new VstopWindow();
            window.Show();

            ProgressBar[] bars = window.FindParts<ProgressBar>().ToArray();
            Assert.Equal(4, bars.Length);
            foreach (ProgressBar bar in bars) Assert.InRange(bar.Value, 0, 100);

            window.Close();
        }, default);

        [Fact]
        public Task Collect_now_forces_a_collection_and_fills_the_heap_figures() => Session.Dispatch(() =>
        {
            var window = new VstopWindow();
            window.Show();

            window.FindNamed<Button>("CollectButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.DoesNotContain("(no collection yet)", Text(window, "MemoryText"));
            Assert.DoesNotContain("stay blank", Text(window, "HintText"));

            window.Close();
        }, default);

        // The dashboard is one of the five that used to poll forever once opened - see EmuSen_LunaP.md §8.2.
        [Fact]
        public Task The_dashboard_stops_polling_while_it_is_hidden() => Session.Dispatch(() =>
        {
            var window = new VstopWindow();
            window.Show();
            Assert.True(window.IsPolling);

            window.Hide();
            Assert.False(window.IsPolling);

            window.Close();
        }, default);

        // The DianaOS default is the terminal dashboard, which cannot draw in
        // a TextBox console - Mistress must replace it, not add alongside.
        [Fact]
        public void The_console_vstop_is_replaced_by_the_windowed_one()
        {
            int opened = 0;
            var shell = DianaOSInterpreter.CreateDefault(null,
                new IDianaOSCommand[] { new VstopWindowCommand(() => opened++) });

            var result = shell.Submit("vstop");

            Assert.Equal(1, opened);
            Assert.Contains("opened in a separate window", result.Output);
            Assert.DoesNotContain("needs a real interactive terminal", result.Output);
        }

        [Fact]
        public void The_windowed_vstop_needs_no_rom_loaded()
        {
            int opened = 0;
            var shell = DianaOSInterpreter.CreateDefault(null,
                new IDianaOSCommand[] { new VstopWindowCommand(() => opened++) });

            shell.Submit("vstop");

            Assert.Equal(1, opened);
        }

        [Fact]
        public Task The_menu_entry_opens_one_window_and_reuses_it() => Session.Dispatch(() =>
        {
            var main = new MainWindow();
            main.Show();

            // Unlike the hardware dashboard, this one never needs a ROM.
            main.GetControl<MenuItem>("SettingsMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));
            Assert.True(main.GetControl<MenuItem>("RuntimeDashboardMenuItem").IsEnabled);
            Assert.False(main.GetControl<MenuItem>("HardwareDashboardMenuItem").IsEnabled);

            // The at-most-one rule is WindowSlot's now, so this reaches through the slot rather than a nullable field.
            var slot = (WindowSlot<VstopWindow>)Field(main, "_vstopWindow")!;

            Click(main, "RuntimeDashboardMenuItem");
            Assert.True(slot.IsOpen);
            VstopWindow first = slot.Current!;

            Click(main, "RuntimeDashboardMenuItem");
            Assert.Same(first, slot.Current);

            first.Close();
            Assert.False(slot.IsOpen);
            Assert.Null(slot.Current);

            main.Close();
        }, default);

        private static void Click(MainWindow w, string name) =>
            w.GetControl<MenuItem>(name).RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));

        private static object? Field(MainWindow window, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);
    }
}
