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
using EmuSen.Crystal.Cheats;
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
    public partial class WindowFitAuditTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(WindowFitAuditTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        // The windows a player reaches in a game, and those reached from the themed library.
        public static readonly string[] InGameWindows = ["ActiveCheats", "ActiveCheatsGeneral", "CheatDatabase", "ActiveCheatsGenesis", "MemoryCardsPlayStation", "MemoryCardsPlayStationHeld", "MemoryCardsPlayStation2", "MemoryCardsPlayStation2Held", "MemoryCardsGenesis", "MemoryCardsSegaCD", "MemoryCardsN64", "CompressedDiscsEmpty", "CompressedDiscs", "CompressedDiscsRunning", "CompressedDiscsFinished", "CompressedDiscsFailed", "CompressedDiscsBeside", "Cores", "CoresDetail", "CoresPlan", "CoresRunning", "CoresNoCatalogue", "Packages", "PackagesDetail", "PackagesPlan", "PackagesRunning", "Scraper", "ScraperDetail", "ScraperPlan", "ScraperRunning", "Library", "LibraryDetail", "LibraryPlan", "LibraryRunning", "ActiveCheatsPlayStation",
            "Resume", "ActiveCheatsLongCheat", "ResumeLongTitle", "Shell"];
        // The installed themes' date, fixed because digit widths decide where the state line wraps.
        private static readonly DateTimeOffset ThemeInstallDate = new(2026, 11, 1, 12, 0, 0, TimeSpan.Zero);

        public static readonly string[] ThemedWindows = ["ScrapeStatusIdle", "GamelistFilter", "FolderEditor", "ThemeSettings", "ThemeBrowser", "ThemeDetail", "ThemeAbout",
            "ThemeBrowserLongName", "ThemeDetailLongName", "ThemeAboutLongName"];

        // Which window each case opens; the scraping status's running and finished states and Find by Name's results are WindowFitScrapeAuditTests'.
        internal static readonly Dictionary<string, Type> Opens = new()
        {
            ["ActiveCheats"] = typeof(ActiveCheatsWindow), ["ActiveCheatsGeneral"] = typeof(ActiveCheatsWindow), ["CheatDatabase"] = typeof(CheatDatabaseWindow),
            ["ActiveCheatsGenesis"] = typeof(ActiveCheatsWindow), ["MemoryCardsPlayStation"] = typeof(MemoryCardsWindow),
            ["MemoryCardsPlayStationHeld"] = typeof(MemoryCardsWindow), ["MemoryCardsPlayStation2"] = typeof(MemoryCardsWindow), ["MemoryCardsPlayStation2Held"] = typeof(MemoryCardsWindow), ["MemoryCardsGenesis"] = typeof(MemoryCardsWindow),
            ["MemoryCardsSegaCD"] = typeof(MemoryCardsWindow), ["MemoryCardsN64"] = typeof(MemoryCardsWindow), ["CompressedDiscsEmpty"] = typeof(CompressedDiscsWindow), ["CompressedDiscs"] = typeof(CompressedDiscsWindow),
            ["CompressedDiscsRunning"] = typeof(CompressedDiscsWindow), ["CompressedDiscsFinished"] = typeof(CompressedDiscsWindow), ["CompressedDiscsFailed"] = typeof(CompressedDiscsWindow), ["CompressedDiscsBeside"] = typeof(CompressedDiscsWindow),
            ["Cores"] = typeof(CoresWindow), ["CoresDetail"] = typeof(CoresWindow), ["CoresPlan"] = typeof(CoresWindow), ["CoresRunning"] = typeof(CoresWindow),
            ["CoresNoCatalogue"] = typeof(CoresWindow), ["Packages"] = typeof(PackagesWindow), ["PackagesDetail"] = typeof(PackagesWindow), ["PackagesPlan"] = typeof(PackagesWindow),
            ["PackagesRunning"] = typeof(PackagesWindow), ["Scraper"] = typeof(ScraperWindow), ["ScraperDetail"] = typeof(ScraperWindow), ["ScraperPlan"] = typeof(ScraperWindow),
            ["ScraperRunning"] = typeof(ScraperWindow), ["Library"] = typeof(LibraryWindow), ["LibraryDetail"] = typeof(LibraryWindow), ["LibraryPlan"] = typeof(LibraryWindow),
            ["LibraryRunning"] = typeof(LibraryWindow), ["ActiveCheatsPlayStation"] = typeof(ActiveCheatsWindow),
            ["Resume"] = typeof(ResumeWindow), ["ScrapeStatusIdle"] = typeof(ScrapeStatusWindow), ["GamelistFilter"] = typeof(GamelistFilterWindow),
            ["FolderEditor"] = typeof(FolderEditorWindow), ["ThemeSettings"] = typeof(ThemeSettingsWindow), ["ThemeBrowser"] = typeof(ThemeBrowserWindow), ["ThemeDetail"] = typeof(ThemeDetailWindow),
            ["ThemeAbout"] = typeof(ThemeAboutWindow), ["ActiveCheatsLongCheat"] = typeof(ActiveCheatsWindow), ["Shell"] = typeof(HotaruWindow), ["ResumeLongTitle"] = typeof(ResumeWindow),
            ["ThemeBrowserLongName"] = typeof(ThemeBrowserWindow),
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
        private readonly FixedRunFolder _folder = new("EmuSenWindowFit");
        private string _root => _folder.Root;
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
            if (_hosted is { } library)
            {
                File.Delete(library);
                File.Delete(EmuSen.Cores.Native.CoreSidecar.PathFor(library));
                MistressRuntime().Rescan();
            }
            EmuSen.Cores.Native.CoreDiscovery.UseDirectories(null);
            Factory.SetValue(null, _realFactory);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            _folder.Dispose();
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

        private (MainWindow Window, PadDriver Pad) InGame(int w, int h, bool resumeAsk = false, string game = "Cobalt Harbor (Synthetic)", bool bigScreen = true, Action<AppSettings>? more = null, bool n64 = false)
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            if (n64) File.WriteAllBytes(Path.Combine(_romDir, game + ".z64"), N64Game());
            else File.WriteAllBytes(Path.Combine(_romDir, game + ".sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings
            {
                BigPictureInterface = { LaunchScreenDuration = EmuSen.Galaxia.Models.BigPictureInterface.LaunchDisabled },
                RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, BigScreen = bigScreen,
                ResumeOnLaunch = resumeAsk ? AppSettings.ResumeAsk : AppSettings.ResumeNever,
                StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"),
                CheatDatabaseDirectory = Path.Combine(_root, "Cheats"),
            };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            more?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = w, Height = h };
            window.Show();
            var pad = new PadDriver(window);
            Settle(window);
            if (bigScreen) OwnLook.Play(window, pad, Path.Combine(_romDir, game + (n64 ? ".z64" : ".sfc")));
            else
            {
                OwnLook.Select(window, Path.Combine(_romDir, game + (n64 ? ".z64" : ".sfc")));
                Call(window, "LaunchSelectedLibraryEntry");
            }
            // The launch's questions are asked off the interface thread and answered within 100 ms only when the machine is idle, so the game is waited for (EmuSen_Debugging_Tools_Reference_v5.md §3.69).
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!window.GetControl<Control>("GameFrame").IsVisible && clock.ElapsedMilliseconds < 15_000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible, "the game started");
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

        // A synthetic N64 game that runs, counting in RDRAM, for the states whose note comes from the running machine.
        internal static byte[] N64Game() => SyntheticN64Rom.BuildRunningFromRdram(new MipsAssembler().Lui(5, 0x8000).Addiu(8, 8, 1).Sw(8, 5, 0x3F0).Beq(0, 0, -3).Nop().ToArray());

        // The open sheet's tab headed by the console.
        internal static void SelectTab(MainWindow window, string console)
        {
            Settle(window);
            var sheet = Sheets(window).SheetOf(Sheets(window).Current!)!;
            var tabs = sheet.GetVisualDescendants().OfType<EmuSen.LunaP.Controls.Tabs>().First();
            int at = tabs.Items.Cast<object>().Select((t, i) => (t, i)).First(p => (p.t as TabItem)?.Header?.ToString() == console).i;
            tabs.SelectedIndex = at;
            Settle(window);
        }

        // The hosted console's library a case placed in the player's /Cores, taken out again by Dispose.
        private string? _hosted;

        // The runtime Mistress shares, which the windows ask.
        private static EmuSen.Crystal.Client.CrystalRuntime MistressRuntime() =>
            (EmuSen.Crystal.Client.CrystalRuntime)typeof(MainWindow).Assembly.GetType("EmuSen.Mistress.CoreSettings")!.GetProperty("Runtime")!.GetValue(null)!;

        // The test core in the player's /Cores, its info edited to run one system only a hosted core runs, so that console has its tabs as with a hosted core installed.
        private void HostedConsole(string systemId)
        {
            // A library of the player's own /Cores, approved through the frontend API, so either implementation lists it.
            string dir = DataStore.Cores;
            Directory.CreateDirectory(dir);
            string library = Path.Combine(dir, Path.GetFileName(EmuSen.WiseMan.Cores.CoreAdapterTests.LibraryPath("v1_test_core")));
            File.Copy(EmuSen.WiseMan.Cores.CoreAdapterTests.LibraryPath("v1_test_core"), library, overwrite: true);
            EmuSen.Cores.Native.CoreSidecar.Write(library);
            var sidecar = (System.Text.Json.Nodes.JsonObject)System.Text.Json.Nodes.JsonNode.Parse(File.ReadAllText(EmuSen.Cores.Native.CoreSidecar.PathFor(library)))!;
            sidecar["info"]!["id"] = $"hosted-{systemId}";
            sidecar["info"]!["display_name"] = $"Hosted {systemId}";
            ((System.Text.Json.Nodes.JsonObject)sidecar["info"]!["systems"]![0]!)["id"] = systemId;
            File.WriteAllText(EmuSen.Cores.Native.CoreSidecar.PathFor(library), sidecar.ToJsonString());
            var runtime = MistressRuntime();
            _hosted = library;
            runtime.Rescan();
            runtime.CoreApprove(Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(library))));
            EmuSen.Mistress.Library.RuntimeCatalog.Forget();
        }

        private void OpenInGame(MainWindow window, PadDriver pad, string which)
        {
            switch (which)
            {
                case "ActiveCheats":
                case "ActiveCheatsGeneral":
                    MistressGame.TwoCheats(window, "Nine continues, and a description long enough to need the whole row or more of it");
                    Call(window, "ShowActiveCheats");
                    Settle(window);
                    if (which == "ActiveCheatsGeneral") Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<TabControl>().First().SelectedIndex = 0;
                    break;
                case "Shell":
                    Call(window, "ShowShellConsole");
                    Settle(window);
                    break;
                case "CheatDatabase":
                    CheatFiles();
                    Call(window, "ShowCheatDatabase");
                    Settle(window);
                    if (Sheets(window).SheetOf(Sheets(window).Current!)!.GetVisualDescendants().OfType<ListBox>().FirstOrDefault(l => l.Name == "SystemsList") is { } systems) systems.SelectedIndex = 2;
                    break;
                case "MemoryCardsPlayStation":
                case "MemoryCardsPlayStationHeld":
                    ShowMemoryCards(window, held: which.EndsWith("Held", StringComparison.Ordinal));
                    break;
                case "MemoryCardsPlayStation2":
                case "MemoryCardsPlayStation2Held":
                    ShowMemoryCardsPs2(window, held: which.EndsWith("Held", StringComparison.Ordinal));
                    break;
                case "MemoryCardsGenesis":
                case "MemoryCardsSegaCD":
                    ShowMemoryCardsTab(window, which == "MemoryCardsGenesis" ? "Genesis" : "Sega CD");
                    break;
                case "MemoryCardsN64":
                    ShowMemoryCardsN64(window);
                    break;
                case "CompressedDiscsEmpty":
                case "CompressedDiscs":
                case "CompressedDiscsRunning":
                case "CompressedDiscsFinished":
                case "CompressedDiscsFailed":
                case "CompressedDiscsBeside":
                    ShowCompressedDiscs(window, which == "CompressedDiscs" ? "Listing" : which["CompressedDiscs".Length..]);
                    break;
                case "Cores":
                case "CoresDetail":
                case "CoresPlan":
                case "CoresRunning":
                case "CoresNoCatalogue":
                    ShowCores(window, which);
                    break;
                case "Packages":
                case "PackagesDetail":
                case "PackagesPlan":
                case "PackagesRunning":
                    ShowPackages(window, which);
                    break;
                case "Scraper":
                case "ScraperDetail":
                case "ScraperPlan":
                case "ScraperRunning":
                    ShowScraper(window, which);
                    break;
                case "Library":
                case "LibraryDetail":
                case "LibraryPlan":
                case "LibraryRunning":
                    ShowLibraryManager(window, which);
                    break;
                case "ActiveCheatsPlayStation":
                    Call(window, "ShowActiveCheats");
                    SelectTab(window, "PlayStation");
                    break;
                case "ActiveCheatsGenesis":
                    Call(window, "ShowActiveCheats");
                    SelectTab(window, "Genesis");
                    break;
                case "Resume":
                case "ResumeLongTitle":
                    Choose(window, pad, "Quit Game");
                    PadMenu.Answer(window, pad, yes: true);
                    // The question names when the game was left; a fixed time keeps its words, and so its recording, the same on every run.
                    foreach (string state in Directory.EnumerateFiles(Path.Combine(_root, "States"), "*", SearchOption.AllDirectories)) File.SetLastWriteTime(state, new DateTime(2026, 10, 15, 9, 30, 0));
                    pad.A();
                    Assert.IsType<ResumeWindow>(Sheets(window).Current);
                    break;
                case "ActiveCheatsLongCheat":
                    MistressGame.Cheats(window).Import(new EmuSen.Galaxia.Models.CheatFile { Cheats =
                    {
                        new() { Kind = "RamPoke", Description = "Infinite lives", Writes = { new() { Space = "CpuBus", Address = "7E0100", Value = "42" } } },
                        new() { Kind = "RamPoke", Description = LongCheat, Writes = { LongWrite } },
                        new() { Kind = "RamPoke", Description = "Ninety-nine coins", Writes = { new() { Space = "CpuBus", Address = "7E0102", Value = "63" } } },
                    } });
                    Call(window, "ShowActiveCheats");
                    Settle(window);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is ListBoxItem item && item.GetVisualDescendants().OfType<TextBlock>().Any(t => t.Text == LongCheat));
                    // The long row chosen, whole in the footer; then the longest status, which the footer says in its place.
                    Assert.Empty(Audit(window, $"{which}-Row-{window.Width}x{window.Height}", _out));
                    Sheets(window).Current!.GetControl<TextBlock>("StatusText").Text = $"Saved 3 cheat(s) as '{LongTitle}' - they come back automatically next time you start it.";
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
        private static readonly EmuSen.Galaxia.Models.CheatFileWrite LongWrite = new()
        {
            Space = "CpuBus", Address = "7E0100", Value = "12345678", Width = 4, Type = "Increase", BigEndian = true, RepeatCount = 16, RepeatAddAddress = "4", RepeatAddValue = "1",
        };


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
            PadMenu.Choose(window, pad, entry);
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
                case "ThemeBrowserLongName":
                case "ThemeDetailLongName":
                case "ThemeAboutLongName":
                    var longHosts = FakeThemeHosts.Standard();
                    longHosts.Themes.Add(LongTheme);
                    ThemeBrowserSheetTests.Serve(longHosts);
                    longHosts.InstallDated(LongTheme.Source, ThemeInstallDate);
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
                case "ThemeSettings":
                    ThemeBrowserSheetTests.OpenSettings(s);
                    break;
                case "ThemeBrowser":
                case "ThemeDetail":
                case "ThemeAbout":
                    var hosts = FakeThemeHosts.Standard();
                    ThemeBrowserSheetTests.Serve(hosts);
                    hosts.InstallDated(hosts["Plain Shelf"].Source, ThemeInstallDate);
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
            // A console only a discovered engine runs has its tabs and shelf with no core in development shown, as a player's build has them - see EmuSen_Settings_Reference.md §4.94.
            if (which.Contains("Genesis", StringComparison.Ordinal) || which.Contains("PlayStation", StringComparison.Ordinal) || which.StartsWith("MemoryCards", StringComparison.Ordinal)) EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(false);
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
                    EmuSen.Cores.Native.CoreDiscovery.UseDevelopment(null);
                }
                return;
            }
            using ThemedSession s = Themed(width, height);
            OpenThemed(s, which);
            Assert.IsType(Opens[which], Sheets(s.Window).Current);
            Assert.Empty(Audit(s.Window, name, _out));
        }, default);

        // Settings ▸ Controllers ▸ Players in a game with four pads, two of one long name, and a seat kept: the players' rows on the desktop's window and the big screen - see EmuSen_Input.md §8.9.
        [Theory]
        [MemberData(nameof(StatusCases))]
        public Task The_players_rows_in_settings_are_whole_at_both_sizes(bool bigScreen, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"SettingsPlayers-{(bigScreen ? "BigScreen" : "Desktop")}-{width}x{height}";
            (MainWindow window, PadDriver pad) = InGame(width, height, bigScreen: bigScreen);
            try
            {
                const string LongPad = "8BitDo Ultimate 2.4G Wireless Controller for Xbox, Hall Effect Special Edition";
                pad.Plug(LongPad);
                pad.Plug(LongPad);
                PadDriver gone = pad.Plug("Nintendo Switch Pro Controller (Bluetooth)");
                pad.Tick();
                gone.Unplug();
                pad.Tick();
                window.ShowSettings(EmuSen.Mistress.Pages.SettingsScreen.Controllers, "players");
                Settle(window);
                Assert.Equal("Players", window.SettingsShown!.Trail[^1]);
                Assert.Contains(window.SettingsLayoutShown!.Rows, r => r.Label.Contains(LongPad[..20], StringComparison.Ordinal));
                // The desktop's Settings is a window of its own, which must fit the screen; the big screen's fills the window.
                Window? own = window.OwnedWindows.OfType<SettingsWindow>().SingleOrDefault();
                Assert.Equal(!bigScreen, own is not null);
                Control root = (Control?)own ?? window.SettingsLayoutShown!;
                if (own is not null)
                {
                    Settle(own);
                    Assert.True(own.Bounds.Height <= height && own.Bounds.Width <= width, $"{name}: the window is {own.Bounds.Size}");
                }
                List<string> faults = bigScreen ? FitAudit.Check(root, WindowAllowances.For(window))
                    : FitAudit.Check(root, WindowAllowances.For(own!), smallest: 0).Where(f => !f.StartsWith("cut at a scrolling edge", StringComparison.Ordinal)).ToList();
                foreach (string f in faults) _out.WriteLine($"{name}: {f}");
                if (bigScreen) SavePicture(window, name);
                Assert.Empty(faults);
            }
            finally
            {
                Stop(window);
                window.Close();
            }
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
                long until = MistressGame.TotalFrames(window) + 90;
                WaitFor(() => MistressGame.TotalFrames(window) >= until, "the frames");
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
