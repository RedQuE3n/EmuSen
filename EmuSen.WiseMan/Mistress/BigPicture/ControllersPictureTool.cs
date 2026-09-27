using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class ControllersPngFactAttribute : FactAttribute
    {
        public ControllersPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the controller pass's pictures to ~/.cache/emusen/bigpicture/png/controllers/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §24";
        }
    }

    // The connect notice, the help bar under each Controller Type and with the swap, and Preferences' Controllers tab, at 1280 by 800; written outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class ControllersPictureTool
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ControllersPictureTool).GetTypeInfo().Assembly);

        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "controllers");

        private readonly ITestOutputHelper _out;

        public ControllersPictureTool(ITestOutputHelper output) => _out = output;

        private void Save(RenderedFrame frame, string name)
        {
            Directory.CreateDirectory(PngFolder);
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        private static AppSettings Settings(MainWindow w) =>
            (AppSettings)typeof(MainWindow).GetField("_appSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w)!;

        private static void Wait(TimeSpan span)
        {
            var frame = new DispatcherFrame();
            var stop = new DispatcherTimer { Interval = span };
            stop.Tick += (_, _) => { stop.Stop(); frame.Continue = false; };
            stop.Start();
            Dispatcher.UIThread.PushFrame(frame);
        }

        private static RenderedFrame Whole(Window window)
        {
            Dispatcher.UIThread.RunJobs();
            window.UpdateLayout();
            window.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            return UiTest.Capture(window);
        }

        // Rows y0..y1 of several frames stacked, one strip per frame.
        private static RenderedFrame Strips(IReadOnlyList<RenderedFrame> frames, int y0, int y1)
        {
            int w = frames[0].Width, h = y1 - y0;
            var rgba = new byte[w * h * frames.Count * 4];
            for (int f = 0; f < frames.Count; f++)
                Array.Copy(frames[f].Rgba, y0 * w * 4, rgba, f * h * w * 4, h * w * 4);
            return new RenderedFrame(rgba, w, h * frames.Count);
        }

        [ControllersPngFact]
        public Task Connect_notices() => Session.Dispatch(() =>
        {
            string root = Path.Combine(Path.GetTempPath(), "EmuSenControllersPictures", Guid.NewGuid().ToString("N"));
            string roms = Path.Combine(root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(root, "Config");
            DataStore.OverrideDirectory = Path.Combine(root, "Home");
            try
            {
                foreach (string g in ThemedSession.SnesGames) File.WriteAllBytes(Path.Combine(roms, g + ".sfc"), SyntheticRom.BuildBlank());
                new AppSettings { RomDirectory = roms, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
                var window = new MainWindow { Width = 1280, Height = 800 };
                window.Show();
                var pad = new PadDriver(window);
                pad.Tick();
                pad.Plug("DualSense Wireless Controller", SDL.GamepadType.PS5);
                pad.Tick();
                Wait(TimeSpan.FromSeconds(1.5));
                Save(Whole(window), "notice-connected-desktop");
                window.Close();
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
                DataStore.OverrideDirectory = null;
                try { Directory.Delete(root, true); } catch { }
            }

            using var s = new ThemedSession(1280, 800);
            PadDriver second = s.Pad.Plug("Pro Controller", SDL.GamepadType.NintendoSwitchPro);
            s.Pad.Tick();
            second.Unplug();
            s.Pad.Tick();
            Wait(TimeSpan.FromSeconds(1.5));
            Save(Whole(s.Window), "notice-disconnected-bigpicture");
        }, default);

        // Each Controller Type over the same gamelist, whole and as a strip of the help bar, then the swap; the synthetic theme and Art Book Next when it is cloned.
        [ControllersPngFact]
        public Task Help_bar_under_each_controller_type() => Session.Dispatch(() =>
        {
            var themes = new List<(string Name, string? Folder)> { ("synthetic", null) };
            if (File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml"))) themes.Add(("artbooknext", ArtBookNextFactAttribute.Folder));
            foreach ((string name, string? folder) in themes)
            {
                using var s = new ThemedSession(1280, 800, themeDirectory: folder);
                s.Pad.Pad.Name = "DualSense Wireless Controller";
                s.Pad.Pad.Type = SDL.GamepadType.PS5;
                ThemedLibraryPadTests.Enter(s, "snes");
                var frames = new List<RenderedFrame>();
                foreach (string type in new[] { "Automatic", "Xbox", "PlayStation", "Nintendo", "Generic" })
                {
                    Settings(s.Window).ControllerType = type;
                    s.Pad.Tick();
                    RenderedFrame frame = s.Capture();
                    Save(frame, $"{name}-help-{type.ToLowerInvariant()}");
                    frames.Add(frame);
                }
                Settings(s.Window).ControllerType = "Xbox";
                Settings(s.Window).SwapPadButtons = true;
                s.Pad.Tick();
                RenderedFrame swapped = s.Capture();
                Save(swapped, $"{name}-help-xbox-swapped");
                frames.Add(swapped);

                HintBar bar = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().First();
                Point at = bar.TranslatePoint(default, s.Window)!.Value;
                int y0 = Math.Max(0, (int)at.Y - 6), y1 = Math.Min(800, (int)(at.Y + bar.Bounds.Height) + 6);
                Save(Strips(frames, y0, y1), $"{name}-help-strip");
            }
        }, default);

        [ControllersPngFact]
        public Task Preferences_controllers_tab() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(1280, 800);
            var prefs = new PreferencesWindow(Settings(s.Window));
            _ = SheetLayer.Show(prefs, s.Window);
            prefs.ShowTab(PreferencesWindow.ControllersTab);
            Save(Whole(s.Window), "preferences-controllers-sheet");
            prefs.Close();
        }, default);
    }
}
