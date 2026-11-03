using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Styling;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Settings on the big screen, each section and page reached by the player's presses from Home, recorded at both sizes and audited at 16:9 and 1280 by 720 - see EmuSen_InterfaceSpec.md §6.3.23 and EmuSen_BigPicture.md §52.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] SettingsViews =
            ["SettingsControllers", "SettingsDisplay", "SettingsDisplayLight", "SettingsShaderLevers", "SettingsSound", "SettingsConsoles", "SettingsConsolePage", "SettingsEngine",
             "SettingsFirmware", "SettingsMemoryCards", "SettingsLibrary", "SettingsEmuSen"];

        public static TheoryData<string, int, int> SettingsCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in SettingsViews)
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(v, width, height);
            return data;
        }

        public static TheoryData<string, int, int> SettingsFitCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in SettingsViews)
                foreach ((int w, int h) in new[] { (1920, 1080), (1280, 720) }) data.Add(v, w, h);
            return data;
        }

        // Home's Settings place, then the shoulders to the section and the directions to the row, as the player goes.
        private static void OpenSettings(MainWindow w, PadDriver pad, string view)
        {
            pad.Down(1);
            pad.Right(5);
            pad.A();
            Settle(w);
            Assert.NotNull(w.SettingsShown);
            void Section(int steps) { for (int i = 0; i < steps; i++) pad.Press(UiButton.PageDown); Settle(w); }
            void To(Func<string, bool> row)
            {
                int at = w.SettingsShown!.Rows.ToList().FindIndex(r => row(r.Key) || row(r.Label));
                Assert.True(at >= 0, $"no such row on {view}");
                pad.Down(at - w.SettingsShown.Rows.ToList().FindIndex(r => r.Key == w.SettingsLayoutShown!.Focused));
                Settle(w);
            }
            switch (view)
            {
                case "SettingsDisplay": Section(1); To(k => k == "crt"); break;
                case "SettingsDisplayLight": Section(1); To(k => k == "variant"); pad.Right(1); break;
                case "SettingsShaderLevers": Section(1); To(k => k == "levers"); pad.A(); break;
                case "SettingsSound": Section(2); break;
                case "SettingsConsoles": Section(3); To(k => k == "Super Nintendo"); break;
                case "SettingsConsolePage": Section(3); To(k => k == "Super Nintendo"); pad.A(); break;
                case "SettingsEngine": Section(3); To(k => k == "Super Nintendo"); pad.A(); Settle(w); pad.A(); break;
                case "SettingsFirmware": Section(3); To(k => k == "PlayStation"); pad.A(); Settle(w); To(k => k == "firmware"); pad.A(); break;
                case "SettingsMemoryCards": Section(3); To(k => k == "PlayStation"); pad.A(); Settle(w); To(k => k == "cards"); pad.A(); break;
                case "SettingsLibrary": Section(4); break;
                case "SettingsEmuSen": Section(5); break;
            }
            Settle(w);
        }

        private static void PictureSettings(MainWindow w, string name)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_U6_PNG") is not { Length: > 0 } folder) return;
            Directory.CreateDirectory(folder);
            UiTest.Capture(w).SavePng(Path.Combine(folder, name + ".png"));
        }

        private static SortedDictionary<string, string> SettingsSeen(MainWindow w)
        {
            SortedDictionary<string, string> seen = Observe(w);
            if (w.SettingsShown is { } s) seen["settings"] = $"{s.Section} {string.Join(" › ", s.Trail)} {InterfaceRecorder.Normalise(w.SettingsLayoutShown?.Focused ?? "none")}";
            return seen;
        }

        private (MainWindow Window, PadDriver Pad) SettingsSession(string view, int width, int height) => OwnLookSession("Games", width, height);

        [Theory]
        [MemberData(nameof(SettingsCases))]
        public Task Every_settings_state_is_as_its_specification_records_it(string view, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"{view}-{width}x{height}";
            (MainWindow window, PadDriver pad) = SettingsSession(view, width, height);
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                OpenSettings(window, pad, view);
                Control root = window.SettingsLayoutShown!;
                SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, "menu", SmallestText: FitAudit.SmallestText), out _);
                doc.Header["case"] = view;
                doc.Header["walk"] = "none: the screen moves its own focus, drawn as a bar";
                PictureSettings(window, name);
                List<string> faults = FitAudit.Check(root, [], FitAudit.SmallestText);
                InterfaceSpecFiles.Presses(doc, pad, window, () => SettingsSeen(window), UiButton.Down, UiButton.Up, UiButton.Back);
                faults.AddRange(InterfaceSpecFiles.Check(doc, _out, Known));
                Assert.Empty(faults);
            }
            finally
            {
                if (Avalonia.Application.Current is { } app) app.RequestedThemeVariant = ThemeVariant.Dark;
                ToolCounts.Quiet();
                InterfaceRecorder.Normalise = s => s;
                InterfaceRecorder.Volatile = _ => null;
                ResetPageClock();
                Stop(window);
                window.Close();
            }
        }, default);

        [Theory]
        [MemberData(nameof(SettingsFitCases))]
        public Task Every_settings_state_fits_at_16_9_and_1280_by_720(string view, int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = SettingsSession(view, width, height);
            try
            {
                OpenSettings(window, pad, view);
                PictureSettings(window, $"{view}-{width}x{height}");
                List<string> faults = FitAudit.Check(window.SettingsLayoutShown!, []);
                foreach (string f in faults) _out.WriteLine($"{view} {width}x{height}: {f}");
                Assert.Empty(faults);
            }
            finally
            {
                if (Avalonia.Application.Current is { } app) app.RequestedThemeVariant = ThemeVariant.Dark;
                ToolCounts.Quiet();
                ResetPageClock();
                Stop(window);
                window.Close();
            }
        }, default);
    }
}
