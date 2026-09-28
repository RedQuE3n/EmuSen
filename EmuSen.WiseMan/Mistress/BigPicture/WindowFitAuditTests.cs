using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Every window a big-screen session shows, framed as a menu at 1280 by 800 and 1920 by 1200, audited for anything cut, past its panel or drawn over something else - see EmuSen_Settings_Reference.md §4.81.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowFitAuditTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(WindowFitAuditTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        // The windows a player reaches in a game, and those reached from the themed library.
        public static readonly string[] InGameWindows = ["ActiveCheats", "ActiveCheatsGeneral", "CheatDatabase", "GraphicsSettings", "ShaderSettings", "ShaderSettingsSliders", "Screenshot", "RewindReel", "Resume", "ControllerBindings"];
        public static readonly string[] ThemedWindows = ["ScrapeStatusIdle", "FindByName", "CoverPicker", "CoverPickerCovers", "GamelistFilter", "FolderEditor", "ThemeBrowser", "ThemeDetail", "ThemeAbout"];

        // Which window each case opens; the scraping status's running and finished states and Find by Name's results are WindowFitScrapeAuditTests'.
        internal static readonly Dictionary<string, Type> Opens = new()
        {
            ["ActiveCheats"] = typeof(ActiveCheatsWindow), ["ActiveCheatsGeneral"] = typeof(ActiveCheatsWindow), ["CheatDatabase"] = typeof(CheatDatabaseWindow),
            ["GraphicsSettings"] = typeof(GraphicsSettingsWindow), ["ShaderSettings"] = typeof(ShaderSettingsWindow), ["ShaderSettingsSliders"] = typeof(ShaderSettingsWindow),
            ["Screenshot"] = typeof(ScreenshotWindow), ["RewindReel"] = typeof(RewindReelWindow), ["Resume"] = typeof(ResumeWindow), ["ControllerBindings"] = typeof(InputSettingsWindow),
            ["ScrapeStatusIdle"] = typeof(ScrapeStatusWindow), ["FindByName"] = typeof(FindByNameWindow), ["CoverPicker"] = typeof(CoverPickerWindow),
            ["CoverPickerCovers"] = typeof(CoverPickerWindow), ["GamelistFilter"] = typeof(GamelistFilterWindow), ["FolderEditor"] = typeof(FolderEditorWindow),
            ["ThemeBrowser"] = typeof(ThemeBrowserWindow), ["ThemeDetail"] = typeof(ThemeDetailWindow), ["ThemeAbout"] = typeof(ThemeAboutWindow),
        };

        // A window is framed as a menu only once the audit opens it at both sizes, and every case opens a window that is framed.
        [Fact]
        public void The_windows_framed_as_menus_are_exactly_the_windows_the_audit_opens()
        {
            Assert.Equal(InGameWindows.Concat(ThemedWindows).OrderBy(w => w), Opens.Keys.OrderBy(w => w));
            Assert.Equal(MainWindow.FramedWindows.Select(t => t.Name).OrderBy(n => n), Opens.Values.Distinct().Select(t => t.Name).OrderBy(n => n));
        }

        public static TheoryData<string, int, int> Cases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string w in InGameWindows.Concat(ThemedWindows))
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(w, width, height);
            return data;
        }

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenWindowFit", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realFactory = Factory.GetValue(null)!;

        public WindowFitAuditTests(ITestOutputHelper output)
        {
            _out = output;
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
        }

        public void Dispose()
        {
            Factory.SetValue(null, _realFactory);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        internal static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        private static void Call(MainWindow w, string method, params object[] args) => typeof(MainWindow).GetMethod(method, Hidden | BindingFlags.Public)!.Invoke(w, args);

        internal static void Settle(Window w)
        {
            for (int i = 0; i < 3; i++)
            {
                Dispatcher.UIThread.RunJobs();
                UiTest.Capture(w);
            }
            Dispatcher.UIThread.RunJobs();
        }

        private static void WaitFor(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for " + what);
                Thread.Sleep(2);
            }
        }

        // The audit of the window on top, framed as a menu whatever the window, with the picture written where the picture tool writes.
        internal static List<string> Audit(MainWindow w, string name, ITestOutputHelper output)
        {
            Settle(w);
            Window top = Sheets(w).Current!;
            Assert.True(Sheets(w).DrawsMenu(top), $"{name} is not framed");
            List<string> faults = FitAudit.Check(Sheets(w).SheetOf(top)!, WindowAllowances.For(top));
            foreach (string f in faults) output.WriteLine($"{name}: {f}");
            // What the audit looked at, for the eye's second check; a trial, kept with the probes and not with the pictures.
            if (Environment.GetEnvironmentVariable("EMUSEN_WINDOW_FIT_PNG") is { Length: > 0 } folder)
            {
                string dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "window-look", "trials", folder);
                Directory.CreateDirectory(dir);
                UiTest.Capture(w).SavePng(Path.Combine(dir, name + ".png"));
            }
            return faults;
        }

        private (MainWindow Window, PadDriver Pad) InGame(int w, int h, bool resumeAsk = false)
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings
            {
                RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, BigScreen = true,
                ResumeOnLaunch = resumeAsk ? AppSettings.ResumeAsk : AppSettings.ResumeNever,
                StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"),
                CheatDatabaseDirectory = Path.Combine(_root, "Cheats"),
            };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            app.Save();
            var window = new MainWindow { Width = w, Height = h };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        private void CheatFiles()
        {
            foreach ((string system, string[] games) in new[]
            {
                ("Nintendo - Super Nintendo Entertainment System", new[] { "Aurora Drift (USA)", "Brass Lantern (Europe)", "Cobalt Harbor (USA)", "A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1)" }),
                ("Nintendo - Nintendo Entertainment System", new[] { "Fable of Tiles (USA)", "Granite Choir (Japan)" }),
                ("Nintendo - Game Boy", new[] { "Hollow Comet (World)" }),
            })
            {
                string db = Path.Combine(_root, "Cheats", system);
                Directory.CreateDirectory(db);
                foreach (string g in games)
                    File.WriteAllText(Path.Combine(db, g + ".cht"), "cheats = 1\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E010042\"\ncheat0_enable = false\n");
            }
        }

        private static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(w, null);

        private static object Field(MainWindow w, string name) => typeof(MainWindow).GetField(name, Hidden)!.GetValue(w)!;

        private void OpenInGame(MainWindow window, PadDriver pad, string which)
        {
            var cheats = (CheatRegistry)Field(window, "_cheats");
            switch (which)
            {
                case "ActiveCheats":
                case "ActiveCheatsGeneral":
                    cheats.AddRamPoke("CpuBus", 0x7E0100, 0x42, "Infinite lives");
                    cheats.AddRamPoke("CpuBus", 0x7E0101, 0x09, "Nine continues, and a description long enough to need the whole row or more of it");
                    Call(window, "ShowActiveCheats");
                    Settle(window);
                    if (which == "ActiveCheatsGeneral") Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<TabControl>().First().SelectedIndex = 0;
                    break;
                case "CheatDatabase":
                    CheatFiles();
                    Call(window, "ShowCheatDatabase");
                    Settle(window);
                    if (Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<ListBox>().FirstOrDefault(l => l.Name == "SystemsList") is { } systems) systems.SelectedIndex = 2;
                    break;
                case "GraphicsSettings":
                    Call(window, "ShowGraphicsSettings");
                    break;
                case "ControllerBindings":
                    Call(window, "ShowControllerBindings");
                    break;
                case "ShaderSettings":
                    Call(window, "ShowShaderSettings");
                    break;
                case "ShaderSettingsSliders":
                    ShaderBrowseTests.WriteBig(ShaderSettingsWindowTests.FakePack());
                    Call(window, "ShowShaderSettings");
                    Settle(window);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is ListBoxItem item && item.Content?.ToString() == "huge");
                    ShaderPanel snes = ((ShaderSettingsWindow)Sheets(window).Current!).PanelFor("SNES");
                    for (int i = 0; i < 2000 && !snes.Reading.IsCompleted; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
                    Settle(window);
                    Assert.NotEmpty(snes.Sliders);
                    break;
                case "Screenshot":
                    string shot = Path.Combine(_root, "shot.png");
                    UiTest.Capture(window).SavePng(shot);
                    _ = SheetLayer.Show(new ScreenshotWindow(shot, "Cobalt Harbor (Synthetic), Screenshot 1"), window);
                    break;
                case "RewindReel":
                    var speed = (SpeedController)Field(window, "_speed");
                    speed.MaxFrameSkip = 0;
                    typeof(MainWindow).GetField("_baseSpeedPercent", Hidden)!.SetValue(window, SpeedController.UnthrottledPercent);
                    var session = (EmulatorSession)Field(window, "_session");
                    long until = session.TotalFrames + 120;
                    WaitFor(() => session.TotalFrames >= until, "the frames");
                    typeof(MainWindow).GetField("_baseSpeedPercent", Hidden)!.SetValue(window, SpeedController.NormalPercent);
                    Choose(window, pad, "Rewind");
                    WaitFor(() => Sheets(window).Current is RewindReelWindow, "the reel");
                    break;
                case "Resume":
                    Choose(window, pad, "Close Game");
                    pad.A();
                    Assert.IsType<ResumeWindow>(Sheets(window).Current);
                    break;
            }
        }

        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            string[] lines = window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
            pad.Down(Array.FindIndex(lines, l => l.StartsWith(entry, StringComparison.Ordinal)));
            pad.A();
        }

        private ThemedSession Themed(int w, int h)
        {
            var s = new ThemedSession(w, h, roms: ThemedFoldersTests.Library);
            return s;
        }

        private void OpenThemed(ThemedSession s, string which)
        {
            string rom = Path.Combine(s.RomDirectory, ThemedSession.SnesGames[2] + ".sfc");
            switch (which)
            {
                case "ScrapeStatusIdle":
                    s.Window.ShowScrapeStatus();
                    break;
                case "FindByName":
                    Call(s.Window, "ShowFindByName", rom, ThemedSession.SnesGames[2]);
                    break;
                case "CoverPicker":
                    Call(s.Window, "ShowCoverPicker", rom, ThemedSession.SnesGames[2]);
                    break;
                case "CoverPickerCovers":
                    foreach (int n in new[] { 0, 1, 3, 4 })
                    {
                        string cover = Path.Combine(DataStore.Artwork, "SNES", ThemedSession.SnesGames[n] + ".png");
                        Directory.CreateDirectory(Path.GetDirectoryName(cover)!);
                        File.Copy(SceneAssets.Halves($"fit-cover-{n}", 120, 160, Avalonia.Media.Colors.Crimson, Avalonia.Media.Colors.Gold), cover, overwrite: true);
                    }
                    Call(s.Window, "ScanArtwork");
                    for (int i = 0; i < 20; i++) { s.Settle(); Thread.Sleep(10); }
                    Call(s.Window, "ShowCoverPicker", rom, ThemedSession.SnesGames[2]);
                    break;
                case "GamelistFilter":
                    ThemedCollectionsTests.Enter(s, "snes");
                    ThemedCollectionsTests.OpenMenu(s);
                    ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GamelistFilterButton" });
                    s.Pad.A();
                    break;
                case "FolderEditor":
                    ThemedCollectionsTests.Enter(s, "snes");
                    SceneGame folder = s.Themed.Stage!.Current.Data.System.Games.First(g => g.Folder);
                    Call(s.Window, "ShowFolderEditor", folder);
                    break;
                case "ThemeBrowser":
                case "ThemeDetail":
                case "ThemeAbout":
                    var hosts = FakeThemeHosts.Standard();
                    ThemeBrowserSheetTests.Serve(hosts);
                    hosts.Install(hosts["Plain Shelf"].Source);
                    ThemeSettingsWindow settings = ThemeBrowserSheetTests.OpenSettings(s);
                    ThemeBrowserWindow browser = ThemeBrowserSheetTests.OpenBrowser(s, settings);
                    if (which == "ThemeBrowser") break;
                    ThemeDetailWindow detail = ThemeBrowserSheetTests.OpenDetail(s, browser, "Plain Shelf");
                    if (which == "ThemeDetail") break;
                    ThemeBrowserSheetTests.Click(ThemeBrowserSheetTests.Named<Button>(detail, "ThemeDetailAbout"));
                    break;
            }
            s.Settle();
        }

        [Theory]
        [MemberData(nameof(Cases))]
        public Task Nothing_in_the_window_is_cut_off_past_its_panel_or_drawn_over_anything_else(string which, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"{which}-{width}x{height}";
            if (InGameWindows.Contains(which))
            {
                (MainWindow window, PadDriver pad) = InGame(width, height, resumeAsk: which == "Resume");
                try
                {
                    OpenInGame(window, pad, which);
                    Assert.IsType(Opens[which], Sheets(window).Current);
                    // A window of tabs is audited on every tab, the first one last so the picture is the one it opens on.
                    TabControl? tabs = which == "ControllerBindings" ? Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<TabControl>().FirstOrDefault() : null;
                    var faults = new List<string>();
                    for (int tab = (tabs?.ItemCount ?? 1) - 1; tab >= 0; tab--)
                    {
                        if (tabs is not null) tabs.SelectedIndex = tab;
                        faults.AddRange(Audit(window, tabs is null ? name : $"{which}-{(tabs.SelectedItem as TabItem)?.Header}-{width}x{height}", _out));
                    }
                    Assert.Empty(faults);
                }
                finally
                {
                    Stop(window);
                    window.Close();
                }
                return;
            }
            using ThemedSession s = Themed(width, height);
            OpenThemed(s, which);
            Assert.IsType(Opens[which], Sheets(s.Window).Current);
            Assert.Empty(Audit(s.Window, name, _out));
        }, default);
    }
}
