using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // Pictures of the bindings window for a person to look at, idle and lit, on the desktop and on a big-screen sheet; skipped unless EMUSEN_BINDINGS_PNG names a folder - see EmuSen_Settings_Reference.md §4.81.
    [Collection(TestCollections.ProcessGlobals)]
    public class ControllerBindingsPictureTool : IDisposable
    {
        private const string FolderVariable = "EMUSEN_BINDINGS_PNG";
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenBindingsPictures", Guid.NewGuid().ToString("N"));

        public ControllerBindingsPictureTool()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            Directory.CreateDirectory(Path.Combine(_root, "Roms"));
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static readonly (int W, int H)[] Sizes = { (1280, 800), (1920, 1200) };

        // Every console's tab, and General, whose modern pad is the raw tester.
        private static string[] Drawn => CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).Prepend("General").ToArray();

        private static int TabOf(string console) => console == "General" ? 0 : CoreCatalog.ConsolesInReleaseOrder.ToList().FindIndex(c => c.Console == console) + 1;

        // What a lit picture holds down: a face button, a shoulder, a direction, a key, and the stick pushed.
        private static void Press(SimulatedPad pad, string console)
        {
            pad.Press(SDL.GamepadButton.East);
            pad.Press(SDL.GamepadButton.LeftShoulder);
            pad.Press(SDL.GamepadButton.DPadUp);
            if (console is "N64" or "General")
            {
                pad.SetAxis(SDL.GamepadAxis.LeftX, 0.75);
                pad.SetAxis(SDL.GamepadAxis.LeftY, -0.45);
                pad.SetAxis(SDL.GamepadAxis.RightX, -1);
            }
        }

        [Fact]
        public Task Desktop_pictures() => UiTest.Run(() =>
        {
            if (Environment.GetEnvironmentVariable(FolderVariable) is not { Length: > 0 } folder) return;
            string[] consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToArray();
            foreach (string console in Drawn)
                foreach ((int w, int h) in Sizes)
                {
                    var pad = new SimulatedPad { Name = "Xbox Wireless Controller" };
                    var gamepad = new GamepadManager(new GamepadBindingMap(), start: true, SimulatedPads.With(pad));
                    var window = new InputSettingsWindow(new ControllerKeyBindings(consoles), new GamepadBindings(consoles), gamepad, new AppSettings(), new HotkeyBindingMap(), console == "General" ? null : console) { Width = w, Height = h };
                    window.Show();
                    UiTest.Capture(window);
                    Dispatcher.UIThread.RunJobs();
                    UiTest.Capture(window).SavePng(Path.Combine(folder, $"desktop-{console}-{w}x{h}-idle.png"));

                    Press(pad, console);
                    gamepad.Poll();
                    window.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.None, null);
                    window.DiagramFor(console)?.Select("Start", NavigationMethod.Directional);
                    Dispatcher.UIThread.RunJobs();
                    UiTest.Capture(window).SavePng(Path.Combine(folder, $"desktop-{console}-{w}x{h}-lit.png"));
                    window.Close();
                }
        });

        [Fact]
        public Task Big_screen_pictures() => UiTest.Run(() =>
        {
            if (Environment.GetEnvironmentVariable(FolderVariable) is not { Length: > 0 } folder) return;
            foreach (string console in Drawn)
                foreach ((int w, int h) in Sizes)
                {
                    new AppSettings { RomDirectory = Path.Combine(_root, "Roms"), LibraryView = AppSettings.LibraryList, BigScreen = true }.Save();
                    var main = new MainWindow { Width = w, Height = h };
                    main.Show();
                    var pad = new PadDriver(main);
                    pad.Pad.Name = "Xbox Wireless Controller";
                    typeof(MainWindow).GetMethod("ShowControllerBindings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(main, null);
                    SheetLayer sheets = main.GetVisualDescendants().OfType<SheetLayer>().First(l => l.Name == "Sheets");
                    var bindings = (InputSettingsWindow)sheets.Current!;
                    TabControl tabs = sheets.SheetOf(bindings)!.GetVisualDescendants().OfType<TabControl>().First();
                    tabs.SelectedIndex = TabOf(console);
                    pad.Tick();
                    for (int settle = 0; settle < 3; settle++) { UiTest.Capture(main); pad.Tick(); }
                    UiTest.Capture(main).SavePng(Path.Combine(folder, $"bigscreen-{console}-{w}x{h}-idle.png"));

                    bindings.SetTesting(true);
                    Press(pad.Pad, console);
                    pad.Tick();
                    UiTest.Capture(main).SavePng(Path.Combine(folder, $"bigscreen-{console}-{w}x{h}-lit.png"));
                    pad.Pad.ReleaseAll();
                    bindings.Close();
                    main.Close();
                }
        });
    }
}
