using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Styling;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Settings on the big screen, each section and page reached by the player's presses from Home, recorded at both sizes and audited at 16:9 and 1280 by 720 - see EmuSen_InterfaceSpec.md §6.3.23-§6.3.24 and EmuSen_BigPicture.md §52.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] SettingsViews =
            ["SettingsControllers", "SettingsBindings", "SettingsDisplay", "SettingsDisplayLight", "SettingsShaderLevers", "SettingsSound", "SettingsConsoles", "SettingsConsolePage", "SettingsEngine",
             "SettingsFirmware", "SettingsMemoryCards", "SettingsLibrary", "SettingsEmuSen", "SettingsDesktopDisplay", "SettingsDesktopConsoles", "SettingsDesktopSearch",
             "SettingsShaders", "SettingsShaderParameters", "SettingsCustomCrop"];

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
            foreach (string v in SettingsViews.Where(v => !v.Contains("Desktop", StringComparison.Ordinal)))
                foreach ((int w, int h) in new[] { (1920, 1080), (1280, 720) }) data.Add(v, w, h);
            return data;
        }

        // Home's Settings place, then the shoulders to the section and the directions to the row, as the player goes.
        private static void OpenSettings(MainWindow w, PadDriver pad, string view)
        {
            // The desktop's doors are the Settings menu's: Settings ▸ Display… and Settings ▸ Consoles…, and the window's search.
            if (view.Contains("Desktop", StringComparison.Ordinal))
            {
                if (view == "SettingsDesktopDisplay" && Avalonia.Application.Current is { } app) app.RequestedThemeVariant = ThemeVariant.Light;
                w.ShowSettings(view == "SettingsDesktopConsoles" ? SettingsScreen.Consoles : SettingsScreen.Display);
                Settle(w);
                if (view == "SettingsDesktopDisplay") w.SettingsLayoutShown!.FocusOn("crt");
                if (view == "SettingsDesktopSearch") w.SettingsLayoutShown!.GetVisualDescendants().OfType<TextBox>().First(t => t.Name == "SettingsSearch").Text = "volume";
                Settle(w);
                return;
            }
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
                case "SettingsBindings":
                    To(k => k == "bindings");
                    pad.A();
                    Settle(w);
                    // The mockup's console: the shoulders step to the Super Nintendo, as the player would.
                    string snes = w.SettingsShown!.Host.Systems.First(f => f.Id == "snes").Console;
                    for (int i = 0; i < 16 && w.SettingsShown!.Binding!.Console != snes; i++) { pad.Press(UiButton.PageDown); Settle(w); }
                    // And the mockup's region: Y, left of B on the drawing.
                    pad.Left(1);
                    break;
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
                case "SettingsShaders": Section(1); To(k => k == "shaders"); pad.A(); Settle(w); To(k => k == "shader:SNES"); pad.A(); break;
                case "SettingsShaderParameters": Section(1); To(k => k == "shaders"); pad.A(); Settle(w); To(k => k == "shader:SNES"); pad.A(); Settle(w); To(k => k == "use:CRT"); pad.A(); Settle(w); for (int i = 0; i < 12 && w.SettingsLayoutShown!.Focused != "params"; i++) pad.Up(); pad.A(); break;
                case "SettingsCustomCrop": Section(3); To(k => k == "Super Nintendo"); pad.A(); Settle(w); To(k => k == "consolecrop"); pad.Right(2); Settle(w); To(k => k == "consolecropedges"); pad.A(); break;
            }
            Settle(w);
        }

        private static void PictureSettings(MainWindow w, string name)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_U6_PNG") is not { Length: > 0 } folder) return;
            Directory.CreateDirectory(folder);
            // The desktop's Settings is a window of its own, so its picture is that window's.
            Window shown = w.SettingsLayoutShown is { } layout && TopLevel.GetTopLevel(layout) is Window own ? own : w;
            UiTest.Capture(shown).SavePng(Path.Combine(folder, name + ".png"));
        }

        private static SortedDictionary<string, string> SettingsSeen(MainWindow w)
        {
            SortedDictionary<string, string> seen = Observe(w);
            if (w.SettingsShown is { } s) seen["settings"] = $"{s.Section} {string.Join(" › ", s.Trail)} {InterfaceRecorder.Normalise(w.SettingsLayoutShown?.Focused ?? "none")}";
            return seen;
        }

        private (MainWindow Window, PadDriver Pad) SettingsSession(string view, int width, int height) => OwnLookSession(view.Contains("Desktop", StringComparison.Ordinal) ? view : "Games", width, height);

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
                bool desktop = view.Contains("Desktop", StringComparison.Ordinal);
                Control root = window.SettingsLayoutShown!;
                SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, desktop ? "window" : "menu", SmallestText: desktop ? 0 : FitAudit.SmallestText), out _);
                doc.Header["case"] = view;
                // The desktop's Settings is a window of its own: its size is recorded beside the screen's, which names the state.
                if (desktop) { doc.Header["window"] = doc.Header["size"]; doc.Header["size"] = $"{width} {height}"; }
                doc.Header["walk"] = "none: the screen moves its own focus, drawn as a bar";
                PictureSettings(window, name);
                List<string> faults = FitAudit.Check(root, [], desktop ? 0 : FitAudit.SmallestText);
                if (!desktop) InterfaceSpecFiles.Presses(doc, pad, window, () => SettingsSeen(window), UiButton.Down, UiButton.Up, UiButton.Back);
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
    
        // Each console's pad as the bindings page draws it, the cases the binding window's audits held until 2026-11-03: a port's other pads, an adapter's fourth player, the hosted consoles, and the keyboard's page with the longest keys.
        public static TheoryData<string, int, int> BindingsFitCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in new[] { "PlayStation 2", "PlayStation 2 Digital", "Saturn", "Dreamcast", "PlayStation", "PlayStation Digital", "Genesis", "Genesis Six", "Genesis Team Player", "N64", "Keys Long Names" })
                foreach ((int w, int h) in new[] { (1280, 800), (1920, 1080), (1280, 720) }) data.Add(v, w, h);
            return data;
        }

        [Theory]
        [MemberData(nameof(BindingsFitCases))]
        public Task Every_consoles_bindings_page_fits(string which, int width, int height) => Session.Dispatch(() =>
        {
            void Pad(string console, string pad)
            {
                var stored = ConsolesConfig.Load();
                stored.SetValue(console, "pad1", pad);
                stored.Save();
            }
            (MainWindow window, PadDriver pad) = OwnLookSession("Games", width, height);
            try
            {
                switch (which)
                {
                    case "PlayStation 2 Digital": Pad("PlayStation 2", "ps2.digital"); break;
                    case "PlayStation Digital": Pad("PlayStation", "psx.pad"); break;
                    case "Genesis Six": Pad("Genesis", "md.pad6"); break;
                    case "Genesis Team Player": Pad("Genesis", "md.teamplayer6"); break;
                    case "Saturn" or "Dreamcast": HostedConsole(which.ToLowerInvariant()); break;
                }
                EmuSen.Mistress.Library.RuntimeCatalog.Forget();
                if (which == "Keys Long Names") LongBindings(window);
                window.ShowSettings(SettingsScreen.Controllers, which == "Keys Long Names" ? "keys" : "bindings");
                SettingsScreen s = window.SettingsShown!;
                string console = which switch { "Keys Long Names" => "SNES", "N64" => "N64", _ => which.Replace(" Digital", "").Replace(" Six", "").Replace(" Team Player", "") };
                s.ChooseBindingConsole(console);
                Assert.Equal(console, s.Binding!.Console);
                if (which == "Genesis Team Player")
                {
                    Assert.Equal(5, s.PlayersOf("Genesis"));
                    while (s.Binding!.Player != 4) s.StepBindingPlayer();
                }
                if (which == "Keys Long Names") s.FocusBinding(EmuSen.Galaxia.Input.PadControl.Start);
                ((SettingsView)Field(window, "_settings")).Fill();
                Settle(window);
                PictureSettings(window, $"Bindings-{which.Replace(" ", "")}-{width}x{height}");
                List<string> faults = FitAudit.Check(window.SettingsLayoutShown!, []);
                foreach (string f in faults) _out.WriteLine($"{which} {width}x{height}: {f}");
                Assert.Empty(faults);
            }
            finally
            {
                ToolCounts.Quiet();
                ResetPageClock();
                Stop(window);
                window.Close();
            }
        }, default);
    }
}
