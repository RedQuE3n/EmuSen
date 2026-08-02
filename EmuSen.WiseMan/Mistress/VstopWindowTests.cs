using System;
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

        private static TextBlock Text(VstopWindow w, string name) => w.GetControl<TextBlock>(name);

        [Fact]
        public Task The_window_fills_in_every_section_on_open() => Session.Dispatch(() =>
        {
            GC.Collect(); // so the GC-derived sections have real numbers - see `man vstop`

            var window = new VstopWindow();
            window.Show();

            Assert.Contains("pid", Text(window, "HeaderText").Text!);
            Assert.Contains(".NET", Text(window, "HostText").Text!);
            Assert.Contains("Working set", Text(window, "MemoryText").Text!);
            Assert.Contains("Collections", Text(window, "GcText").Text!);
            Assert.Contains("OS threads", Text(window, "ThreadsText").Text!);

            // CPU, machine memory, heap fragmentation, thread pool.
            Assert.Equal(4, window.GetControl<StackPanel>("MetersPanel").Children.Count);

            window.Close();
        }, default);

        [Fact]
        public Task Every_meter_bar_stays_inside_its_range() => Session.Dispatch(() =>
        {
            var window = new VstopWindow();
            window.Show();

            foreach (Control row in window.GetControl<StackPanel>("MetersPanel").Children)
            {
                ProgressBar bar = ((Grid)row).Children.OfType<ProgressBar>().Single();
                Assert.InRange(bar.Value, 0, 100);
            }

            window.Close();
        }, default);

        [Fact]
        public Task Collect_now_forces_a_collection_and_fills_the_heap_figures() => Session.Dispatch(() =>
        {
            var window = new VstopWindow();
            window.Show();

            window.GetControl<Button>("CollectButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

            Assert.DoesNotContain("(no collection yet)", Text(window, "MemoryText").Text!);
            Assert.DoesNotContain("stay blank", Text(window, "HintText").Text!);

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

            Click(main, "RuntimeDashboardMenuItem");
            object? first = Field(main, "_vstopWindow");
            Assert.NotNull(first);

            Click(main, "RuntimeDashboardMenuItem");
            Assert.Same(first, Field(main, "_vstopWindow"));

            ((Window)first!).Close();
            Assert.Null(Field(main, "_vstopWindow"));

            main.Close();
        }, default);

        private static void Click(MainWindow w, string name) =>
            w.GetControl<MenuItem>(name).RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));

        private static object? Field(MainWindow window, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);
    }
}
