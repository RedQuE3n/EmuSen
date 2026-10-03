using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Every window a big-screen session shows, framed as a menu at 1280 by 800 and 1920 by 1200, audited for anything cut, past its panel or drawn over something else - see EmuSen_Settings_Reference.md §4.83.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowFitAuditTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(WindowFitAuditTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        // The windows a player reaches in a game, and those reached from the themed library.
        public static readonly string[] InGameWindows = ["ActiveCheats", "ActiveCheatsGeneral", "CheatDatabase", "GraphicsSettings", "GraphicsSettingsN64Note", "GraphicsSettingsNesEngine", "GraphicsSettingsSnesEngine", "ShaderSettings", "ShaderSettingsSliders", "Screenshot", "RewindReel", "Resume", "ControllerBindings",
            "ActiveCheatsLongCheat", "ShaderSettingsLongParameter", "ScreenshotLongTitle", "ResumeLongTitle", "ControllerBindingsLongNames"];
        public static readonly string[] ThemedWindows = ["ScrapeStatusIdle", "FindByName", "CoverPicker", "CoverPickerCovers", "GamelistFilter", "FolderEditor", "ThemeBrowser", "ThemeDetail", "ThemeAbout",
            "FindByNameLongTitle", "CoverPickerLongTitle", "ThemeBrowserLongName", "ThemeDetailLongName", "ThemeAboutLongName"];

        // Which window each case opens; the scraping status's running and finished states and Find by Name's results are WindowFitScrapeAuditTests'.
        internal static readonly Dictionary<string, Type> Opens = new()
        {
            ["ActiveCheats"] = typeof(ActiveCheatsWindow), ["ActiveCheatsGeneral"] = typeof(ActiveCheatsWindow), ["CheatDatabase"] = typeof(CheatDatabaseWindow),
            ["GraphicsSettings"] = typeof(GraphicsSettingsWindow), ["GraphicsSettingsN64Note"] = typeof(GraphicsSettingsWindow), ["GraphicsSettingsNesEngine"] = typeof(GraphicsSettingsWindow), ["GraphicsSettingsSnesEngine"] = typeof(GraphicsSettingsWindow), ["ShaderSettings"] = typeof(ShaderSettingsWindow), ["ShaderSettingsSliders"] = typeof(ShaderSettingsWindow),
            ["Screenshot"] = typeof(ScreenshotWindow), ["RewindReel"] = typeof(RewindReelWindow), ["Resume"] = typeof(ResumeWindow), ["ControllerBindings"] = typeof(InputSettingsWindow),
            ["ScrapeStatusIdle"] = typeof(ScrapeStatusWindow), ["FindByName"] = typeof(FindByNameWindow), ["CoverPicker"] = typeof(CoverPickerWindow),
            ["CoverPickerCovers"] = typeof(CoverPickerWindow), ["GamelistFilter"] = typeof(GamelistFilterWindow), ["FolderEditor"] = typeof(FolderEditorWindow),
            ["ThemeBrowser"] = typeof(ThemeBrowserWindow), ["ThemeDetail"] = typeof(ThemeDetailWindow), ["ThemeAbout"] = typeof(ThemeAboutWindow),
            // The long names, where a cut would first appear (Q190).
            ["ActiveCheatsLongCheat"] = typeof(ActiveCheatsWindow), ["ShaderSettingsLongParameter"] = typeof(ShaderSettingsWindow), ["ScreenshotLongTitle"] = typeof(ScreenshotWindow),
            ["ResumeLongTitle"] = typeof(ResumeWindow), ["ControllerBindingsLongNames"] = typeof(InputSettingsWindow),
            ["FindByNameLongTitle"] = typeof(FindByNameWindow), ["CoverPickerLongTitle"] = typeof(CoverPickerWindow), ["ThemeBrowserLongName"] = typeof(ThemeBrowserWindow),
            ["ThemeDetailLongName"] = typeof(ThemeDetailWindow), ["ThemeAboutLongName"] = typeof(ThemeAboutWindow),
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
            if (FitAudit.LastSmallest is { } least)
            {
                string line = $"CAPS {name}: {least.Caps:F2} design px of capitals at {least.Size:F2} px, {least.What}";
                output.WriteLine(line);
                if (Environment.GetEnvironmentVariable("EMUSEN_WINDOW_FIT_CAPS") is { Length: > 0 } record) File.AppendAllText(record, line + Environment.NewLine);
            }
            if (Sheets(w).SheetOf(top)!.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu") is { TitleLines: > 1 } panel)
                output.WriteLine($"TITLE {name}: {panel.TitleLines} lines at {panel.TitleDrawnSize / MenuPanel.GetScale(panel):F1} design px");
            SavePicture(w, name);
            return faults;
        }

        private (MainWindow Window, PadDriver Pad) InGame(int w, int h, bool resumeAsk = false, string game = "Cobalt Harbor (Synthetic)", bool bigScreen = true)
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(_romDir, game + ".sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings
            {
                RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, BigScreen = bigScreen,
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
            if (bigScreen) pad.A();
            else Call(window, "LaunchSelectedLibraryEntry");
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

        // The N64 tab at 3x resolution with 3x antialiasing, where the Antialiasing row carries its note (Mars_Performance.md §42.5).
        internal static void ShowN64WithNote(MainWindow window)
        {
            var graphics = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", Hidden)!.GetValue(window)!;
            graphics.SetValue("N64", "RenderScale", "3");
            graphics.SetValue("N64", "Antialiasing", "3x");
            Call(window, "ShowGraphicsSettings");
            Settle(window);
            var tabs = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<EmuSen.LunaP.Controls.Tabs>().First(t => t.Name == "ConsoleTabs");
            tabs.SelectedIndex = EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToList().IndexOf("N64");
            Settle(window);
            HintText note = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<HintText>().Single(t => t.Name == "N64.Antialiasing.Note");
            Assert.True(note.IsEffectivelyVisible, "the note is not shown");
            ScrollViewer scroll = note.FindAncestorOfType<ScrollViewer>()!;
            Point at = note.TranslatePoint(default, (Visual)scroll.Content!) ?? default;
            scroll.Offset = new Vector(0, Math.Max(0, at.Y + note.Bounds.Height - scroll.Viewport.Height * 0.8));
            Settle(window);
        }

        // The NES tab, whose Engine row MoonRT's stage 4 added (Moon_Native.md §8.3).
        internal static void ShowNesTab(MainWindow window) => ShowEngineTab(window, "NES");

        // The console's tab with its engine row showing, where the console has one (the SNES's comes from discovery).
        internal static void ShowEngineTab(MainWindow window, string console)
        {
            Call(window, "ShowGraphicsSettings");
            Settle(window);
            var tabs = Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<EmuSen.LunaP.Controls.Tabs>().First(t => t.Name == "ConsoleTabs");
            tabs.SelectedIndex = EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToList().IndexOf(console);
            Settle(window);
            if (EmuSen.Cores.CoreCatalog.EngineFor(console) is null) return;
            Assert.Contains(Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<Control>(), c => c.Name == $"{console}.{EmuSen.Cores.CoreCatalog.EngineKey}" && c.IsEffectivelyVisible);
        }

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
                case "GraphicsSettingsN64Note":
                    ShowN64WithNote(window);
                    break;
                case "GraphicsSettingsNesEngine":
                    ShowNesTab(window);
                    break;
                case "GraphicsSettingsSnesEngine":
                    ShowEngineTab(window, "SNES");
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
                case "ResumeLongTitle":
                    Choose(window, pad, "Close Game");
                    pad.A();
                    Assert.IsType<ResumeWindow>(Sheets(window).Current);
                    break;
                case "ActiveCheatsLongCheat":
                    cheats.AddRamPoke("CpuBus", 0x7E0100, 0x42, "Infinite lives");
                    cheats.AddCheat(CheatKind.RamPoke, [LongWrite], null, LongCheat);
                    cheats.AddRamPoke("CpuBus", 0x7E0102, 0x63, "Ninety-nine coins");
                    Call(window, "ShowActiveCheats");
                    Settle(window);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is ListBoxItem item && item.GetVisualDescendants().OfType<TextBlock>().Any(t => t.Text == LongCheat));
                    // The long row chosen, whole in the footer; then the longest status, which the footer says in its place.
                    Assert.Empty(Audit(window, $"{which}-Row-{window.Width}x{window.Height}", _out));
                    Sheets(window).Current!.GetControl<TextBlock>("StatusText").Text = $"Saved 3 cheat(s) as '{LongTitle}' - they come back automatically next time you start it.";
                    break;
                case "ShaderSettingsLongParameter":
                    LongParameterPreset(ShaderSettingsWindowTests.FakePack());
                    Call(window, "ShowShaderSettings");
                    Settle(window);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is ListBoxItem item && item.Content?.ToString() == "long-names");
                    ShaderPanel panel = ((ShaderSettingsWindow)Sheets(window).Current!).PanelFor("SNES");
                    for (int i = 0; i < 2000 && !panel.Reading.IsCompleted; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(1); }
                    Settle(window);
                    Assert.NotEmpty(panel.Sliders);
                    break;
                case "ScreenshotLongTitle":
                    string longShot = Path.Combine(_root, "shot.png");
                    UiTest.Capture(window).SavePng(longShot);
                    _ = SheetLayer.Show(new ScreenshotWindow(longShot, $"{LongTitle}, Screenshot 12"), window);
                    break;
                case "ControllerBindingsLongNames":
                    LongBindings(window);
                    Call(window, "ShowControllerBindings");
                    break;
            }
        }

        // A theme whose name, author, licence and variants are as long as ES-DE's list could carry (Q190).
        private static FakeTheme LongTheme => new()
        {
            Name = "The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition",
            Author = "Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective",
            Source = new ThemeSource("community-artwork-collective", "complete-illustrated-library-anniversary-deluxe-es-de", "main"),
            HostLicence = "Creative Commons Attribution Non Commercial Share Alike 4.0 International",
            Readme = "# The Complete Illustrated Library\n\n## License\nCreative Commons Attribution-NonCommercial-ShareAlike 4.0 International, with the fonts under the SIL Open Font License 1.1.\n",
            Variants = ["Textlist with Videos and Descriptions", "Carousel with Wheel Artwork and Reflections", "Grid of Covers"],
            ColorSchemes = ["Midnight Blue with Orange Highlights", "Paper White"], Screenshots = 2,
        };

        // A cheat's description and code as long as a database's get (Q190).
        internal const string LongCheat = "Infinite health, ammunition and continues for both players, with the timer stopped in every stage except the last";
        private static readonly CheatWrite LongWrite = new()
        {
            Space = "CpuBus", Address = 0x7E0100, Value = 0x12345678, Width = 4, Type = CheatWriteType.Increase, BigEndian = true, RepeatCount = 16, RepeatAddAddress = 4, RepeatAddValue = 1,
        };

        // A preset whose parameters have names as long as RetroArch's longest, under the SNES tab's list as "long-names" (Q190).
        private static void LongParameterPreset(string pack)
        {
            string shader = "#version 450\n"
                + "#pragma parameter LONG_HEAD \"--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---\" 0.0 0.0 0.0 1.0\n"
                + "#pragma parameter BEAM_WIDTH_AT_THE_BRIGHTEST_PART_OF_THE_PICTURE \"Horizontal beam width at the brightest part of the picture, in pixels of the source image\" 0.60 0.00 2.00 0.05\n"
                + "#pragma parameter MASK_STRENGTH \"Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)\" 0.30 0.00 1.00 0.05\n"
                + "#pragma stage vertex\nvoid main() { gl_Position = vec4(0.0); }\n#pragma stage fragment\nvoid main() { }\n";
            Directory.CreateDirectory(Path.Combine(pack, "long", "shaders"));
            File.WriteAllText(Path.Combine(pack, "long", "shaders", "long.slang"), shader);
            File.WriteAllText(Path.Combine(pack, "long", "long-names.slangp"), "shaders = 1\nshader0 = shaders/long.slang\n");
        }

        // Every console's A and Start bound to the longest names a keyboard's keys have, so the labels beside the tall columns and in the rows are at their widest (Q190).
        private static void LongBindings(MainWindow window)
        {
            var keys = (EmuSen.Mistress.Input.ControllerKeyBindings)Field(window, "_keyBindings");
            foreach (string console in keys.ByConsole.Keys.ToList())
            {
                keys.For(console).Rebind(EmuSen.Galaxia.Input.PadControl.A, Avalonia.Input.Key.MediaPreviousTrack);
                keys.For(console).Rebind(EmuSen.Galaxia.Input.PadControl.Start, Avalonia.Input.Key.LaunchApplication1);
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
                case "FindByNameLongTitle":
                case "CoverPickerLongTitle":
                    string longRom = Path.Combine(s.RomDirectory, LongTitle + ".sfc");
                    if (File.Exists(rom)) File.Copy(rom, longRom, overwrite: true);
                    else File.WriteAllBytes(longRom, PadRewindReelTests.ChangingBackdrop());
                    Call(s.Window, which == "FindByNameLongTitle" ? "ShowFindByName" : "ShowCoverPicker", longRom, LongTitle);
                    break;
                case "ThemeBrowserLongName":
                case "ThemeDetailLongName":
                case "ThemeAboutLongName":
                    var longHosts = FakeThemeHosts.Standard();
                    longHosts.Themes.Add(LongTheme);
                    ThemeBrowserSheetTests.Serve(longHosts);
                    longHosts.Install(LongTheme.Source);
                    ThemeBrowserWindow longBrowser = ThemeBrowserSheetTests.OpenBrowser(s, ThemeBrowserSheetTests.OpenSettings(s));
                    if (which == "ThemeBrowserLongName")
                    {
                        ThemeBrowserSheetTests.Named<Button>(longBrowser, "ThemeBrowserDetails").Focus();
                        PadAudit.Reach(Sheets(s.Window).SheetOf(longBrowser)!, s.Pad, e => e is ListBoxItem item && item.GetVisualDescendants().OfType<TextBlock>().Any(t => t.Text?.StartsWith(LongTheme.Name, StringComparison.Ordinal) == true));
                        s.Settle();
                        Assert.True(ThemeBrowserSheetTests.Pump(() => longBrowser.PreviewLoading is { IsCompleted: true }
                            && ThemeBrowserSheetTests.RootOf(longBrowser).GetVisualDescendants().OfType<TextBlock>().Any(t => t.Text?.Contains(LongTheme.Author, StringComparison.Ordinal) == true)), "the long theme's preview");
                        break;
                    }
                    ThemeDetailWindow longDetail = ThemeBrowserSheetTests.OpenDetail(s, longBrowser, LongTheme.Name);
                    if (which == "ThemeDetailLongName") break;
                    ThemeBrowserSheetTests.Click(ThemeBrowserSheetTests.Named<Button>(longDetail, "ThemeDetailAbout"));
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
                (MainWindow window, PadDriver pad) = InGame(width, height, resumeAsk: which.StartsWith("Resume", StringComparison.Ordinal),
                    game: which.EndsWith("LongTitle", StringComparison.Ordinal) ? LongTitle : "Cobalt Harbor (Synthetic)");
                try
                {
                    OpenInGame(window, pad, which);
                    Assert.IsType(Opens[which], Sheets(window).Current);
                    // A window of tabs is audited on every tab, the first one last so the picture is the one it opens on.
                    TabControl? tabs = which.StartsWith("ControllerBindings", StringComparison.Ordinal) ? Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<TabControl>().FirstOrDefault() : null;
                    var faults = new List<string>();
                    for (int tab = (tabs?.ItemCount ?? 1) - 1; tab >= 0; tab--)
                    {
                        if (tabs is not null) tabs.SelectedIndex = tab;
                        faults.AddRange(Audit(window, tabs is null ? name : $"{which}-{(tabs.SelectedItem as TabItem)?.Header}-{width}x{height}", _out));
                        if (tabs is not null) faults.AddRange(Margins(window, $"{(tabs.SelectedItem as TabItem)?.Header} at {width}x{height}", which.EndsWith("LongNames", StringComparison.Ordinal) ? 0 : LabelMargin));
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

        // A game's title as long as a No-Intro name gets, for the cases that show one (Q190).
        internal const string LongTitle = "A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)";

        public static TheoryData<bool, int, int> StatusCases()
        {
            var data = new TheoryData<bool, int, int>();
            foreach (bool bigScreen in new[] { true, false })
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(bigScreen, width, height);
            return data;
        }

        // The status line's words are whole in its tooltip when the line is trimmed to the window (Q191).
        internal static readonly FitAudit.Allowance WholeInItsTip = new("the status line's words are whole in its tooltip",
            c => c is TextBlock { Text: { Length: > 0 } text } t && ToolTip.GetTip(t) is string tip && tip.StartsWith(text, StringComparison.Ordinal));

        // The main window's status line under a running game, at both sizes, in a big-screen session and on the desktop (Q191).
        [Theory]
        [MemberData(nameof(StatusCases))]
        public Task The_status_line_under_a_running_game_is_whole_and_inside_the_window(bool bigScreen, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"StatusLine-{(bigScreen ? "BigScreen" : "Desktop")}-{width}x{height}";
            (MainWindow window, _) = InGame(width, height, game: LongTitle, bigScreen: bigScreen);
            try
            {
                var session = (EmulatorSession)Field(window, "_session");
                long until = session.TotalFrames + 90;
                WaitFor(() => session.TotalFrames >= until, "the frames");
                TextBlock status = window.GetControl<TextBlock>("StatusText"), fps = window.GetControl<TextBlock>("FpsText");
                WaitFor(() => fps.Text is { Length: > 0 }, "the counters");
                Border bar = window.GetControl<Border>("StatusBar");
                Assert.StartsWith("Running: ", status.Text);
                List<string> faults = StatusFaults(window, bar, status, fps, bigScreen, name);
                double oneLine = bar.Bounds.Height;
                Assert.True(bar.IsVisible);
                // A big screen shows performance in its HUD, not in the status line.
                Assert.Equal(!bigScreen, fps.IsEffectivelyVisible);
                Assert.Equal(status.Text, ToolTip.GetTip(status));
                if (!bigScreen) Assert.True(ToolTip.GetTip(fps) is string whole && whole.StartsWith(fps.Text!, StringComparison.Ordinal) && whole.Contains(" | outside: "), "the counters' tooltip is the whole line");

                // The longest thing the line says: a failure naming a long title and a path; wrapped in a big screen, trimmed on the desktop.
                status.Text = $"Failed to load {LongTitle}.sfc: Could not find a part of the path '/home/player/Documents/Roms/Super Nintendo Entertainment System/{LongTitle}.sfc'.";
                faults.AddRange(StatusFaults(window, bar, status, fps, bigScreen, name + "-LongMessage"));
                Assert.Equal(status.Text, ToolTip.GetTip(status));
                if (bigScreen) Assert.True(bar.Bounds.Height > oneLine * 1.5, $"the message wraps: {bar.Bounds.Height} against {oneLine}");
                else
                {
                    Assert.True(status.TextLayout.TextLines.Any(l => l.HasCollapsed), "the message is trimmed");
                    // The desktop window at its narrowest: the counters keep to their share, trimmed, and both stay inside it.
                    window.Width = window.MinWidth;
                    faults.AddRange(StatusFaults(window, bar, status, fps, bigScreen, name + "-Narrowest"));
                    Assert.True(fps.Bounds.Width <= bar.Bounds.Width * MainWindow.CountersShare + 0.5, $"the counters take {fps.Bounds.Width} of {bar.Bounds.Width}");
                }
                Assert.Empty(faults);
            }
            finally
            {
                Stop(window);
                window.Close();
            }
        }, default);

        // The size a drawing's labels keep over SmallestText with the bindings a console starts with, in design pixels: the layout's reach, the floor itself being capitals (Q188, §4.83.7).
        internal const double LabelMargin = 1.0;

        // Each shown drawing's label words in design pixels, their margin over the floor and the columns split, for the record; a fault when the margin is under the one asked (§4.83.6).
        private List<string> Margins(MainWindow window, string where, double margin)
        {
            var faults = new List<string>();
            Control sheet = Sheets(window).SheetOf(Sheets(window).Current!)!;
            double unit = MenuPanel.GetScale(sheet) is > 0 and var u ? u : 1;
            foreach (ControllerDiagram d in sheet.GetVisualDescendants().OfType<ControllerDiagram>().Where(d => d.IsEffectivelyVisible && d.Bounds.Width > 0 && d.LabelTextSize > 0))
            {
                double size = d.LabelTextSize * (d.TransformToVisual(sheet)?.M11 ?? 1) / unit;
                double caps = size * FitAudit.CapsOf(FitAudit.FaceOf(new Avalonia.Media.Typeface(d.GetValue(Avalonia.Controls.Documents.TextElement.FontFamilyProperty))));
                double floor = FitAudit.SmallestText * FitAudit.DesktopCaps;
                _out.WriteLine($"MARGIN {where}: {d.Layout} {d.Bounds.Width:F0}x{d.Bounds.Height:F0}, labels {size:F2} design px, capitals {caps:F2}, {caps - floor:+0.00;-0.00} over the floor's {floor:F2}, split {(d.SplitSides.Count == 0 ? "none" : string.Join("+", d.SplitSides))}");
                if (size < FitAudit.SmallestText + margin - 0.05) faults.Add($"labels under the margin: {d.Layout} at {size:F1} design px, where {FitAudit.SmallestText + margin:F1} is asked");
            }
            return faults;
        }

        // The status bar audited: a big screen's by every rule, the desktop's with trimmed words excused by their tooltip and at the desktop's size; and both kept inside the window.
        private List<string> StatusFaults(MainWindow window, Border bar, TextBlock status, TextBlock fps, bool bigScreen, string name)
        {
            Settle(window);
            SavePicture(window, name);
            List<string> faults = bigScreen ? FitAudit.Check(bar) : FitAudit.Check(bar, [WholeInItsTip], smallest: 0);
            foreach (TextBlock t in new[] { status, fps }.Where(t => t.IsEffectivelyVisible))
            {
                Rect r = new Rect(t.Bounds.Size).TransformToAABB(t.TransformToVisual(window)!.Value);
                if (r.Left < -0.5 || r.Right > window.Bounds.Width + 0.5) faults.Add($"past the window: {t.Name} at {r} outside its {window.Bounds.Width} pixels");
            }
            foreach (string f in faults) _out.WriteLine($"{name}: {f}");
            return faults;
        }

        // What a case looked at, saved when EMUSEN_WINDOW_FIT_PNG names a folder: a trial under the probes, or the named stage of a set of pictures.
        internal static void SavePicture(Window w, string name)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_WINDOW_FIT_PNG") is not { Length: > 0 } folder) return;
            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            string dir = Environment.GetEnvironmentVariable("EMUSEN_WINDOW_FIT_SET") is { Length: > 0 } pictures && folder is "before" or "after"
                ? Path.Combine(home, ".cache", "emusen", "bigpicture", "png", pictures, folder)
                : Path.Combine(home, ".cache", "emusen", "probe", "window-look", "trials", folder);
            Directory.CreateDirectory(dir);
            UiTest.Capture(w).SavePng(Path.Combine(dir, name + ".png"));
        }
    }
}
