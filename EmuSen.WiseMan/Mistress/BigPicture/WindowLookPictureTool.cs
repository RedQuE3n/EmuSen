using System;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.Galaxia;
using SDL3;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.Scraping;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class WindowLookPngFactAttribute : FactAttribute
    {
        public WindowLookPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes every big-screen window's picture to ~/.cache/emusen/bigpicture/png/window-look/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Settings_Reference.md §4.80";
        }
    }

    // Every window a big-screen session shows as a sheet, at 1280 by 800 and 1920 by 1200, into a folder named by EMUSEN_WINDOW_LOOK_STAGE; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowLookPictureTool : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(WindowLookPictureTool).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        public static string Folder => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "window-look",
            Environment.GetEnvironmentVariable("EMUSEN_WINDOW_LOOK_STAGE") is { Length: > 0 } stage ? stage : "after");

        internal static readonly (int W, int H)[] Sizes = [(1280, 800), (1920, 1200)];

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenWindowLook", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realFactory = Factory.GetValue(null)!;

        public WindowLookPictureTool(ITestOutputHelper output)
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

        internal static void Save(Window window, string name, ITestOutputHelper output)
        {
            Directory.CreateDirectory(Folder);
            Dispatcher.UIThread.RunJobs();
            UiTest.Capture(window);
            Dispatcher.UIThread.RunJobs();
            string path = Path.Combine(Folder, name + ".png");
            UiTest.Capture(window).SavePng(path);
            output.WriteLine(path);
        }

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        private static void CloseAll(MainWindow w)
        {
            for (int i = 0; i < 12 && Sheets(w).Current is { } top; i++)
            {
                top.Close();
                Dispatcher.UIThread.RunJobs();
            }
        }

        private static void Call(MainWindow w, string method, params object[] args) =>
            typeof(MainWindow).GetMethod(method, Hidden | BindingFlags.Public)!.Invoke(w, args);

        private static object Field(MainWindow w, string name) => typeof(MainWindow).GetField(name, Hidden)!.GetValue(w)!;

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

        // A game running in a big-screen session with the sidebar library, as the in-game windows are opened over it.
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

        private void CheatDatabase()
        {
            foreach ((string system, string[] games) in new[]
            {
                ("Nintendo - Super Nintendo Entertainment System", new[] { "Aurora Drift (USA)", "Brass Lantern (Europe)", "Cobalt Harbor (USA)" }),
                ("Nintendo - Nintendo Entertainment System", new[] { "Fable of Tiles (USA)", "Granite Choir (Japan)" }),
                ("Nintendo - Game Boy", new[] { "Hollow Comet (World)" }),
            })
            {
                string db = Path.Combine(_root, "Cheats", system);
                Directory.CreateDirectory(db);
                foreach (string g in games)
                    File.WriteAllText(Path.Combine(db, g + ".cht"), "cheats = 2\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E010042\"\ncheat0_enable = false\ncheat1_desc = \"Start on the last stage\"\ncheat1_code = \"7E010207\"\ncheat1_enable = false\n");
            }
        }

        private static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(w, null);

        // Seconds of history in a fraction of one, unthrottled, then back to normal speed.
        private static void Play(MainWindow window, int frames)
        {
            var speed = (SpeedController)Field(window, "_speed");
            speed.MaxFrameSkip = 0;
            typeof(MainWindow).GetField("_baseSpeedPercent", Hidden)!.SetValue(window, SpeedController.UnthrottledPercent);
            var session = (EmulatorSession)Field(window, "_session");
            long until = session.TotalFrames + frames;
            WaitFor(() => session.TotalFrames >= until, "the frames");
            typeof(MainWindow).GetField("_baseSpeedPercent", Hidden)!.SetValue(window, SpeedController.NormalPercent);
        }

        [WindowLookPngFact]
        public Task In_game_windows() => Session.Dispatch(() =>
        {
            CheatDatabase();
            foreach ((int w, int h) in Sizes)
            {
                (MainWindow window, PadDriver pad) = InGame(w, h);
                string p = $"{w}x{h}";
                try
                {
                    var cheats = (CheatRegistry)Field(window, "_cheats");
                    cheats.AddRamPoke("CpuBus", 0x7E0100, 0x42, "Infinite lives");
                    cheats.AddRamPoke("CpuBus", 0x7E0101, 0x09, "Nine continues");
                    Call(window, "ShowActiveCheats");
                    Save(window, $"active-cheats-{p}", _out);
                    pad.R1();
                    Save(window, $"active-cheats-console-{p}", _out);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is ListBoxItem);
                    Save(window, $"active-cheats-row-focused-{p}", _out);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is Button { Name: "ApplyButton" });
                    Save(window, $"active-cheats-button-focused-{p}", _out);
                    CloseAll(window);

                    Call(window, "ShowCheatDatabase");
                    Save(window, $"cheat-database-{p}", _out);
                    if ((Sheets(window).SheetOf(Sheets(window).Current!) as ILogical)?.GetLogicalDescendants().OfType<ListBox>().FirstOrDefault(l => l.Name == "SystemsList") is { } systems)
                        systems.SelectedIndex = 1;
                    Save(window, $"cheat-database-system-{p}", _out);
                    PadAudit.Reach(Sheets(window).SheetOf(Sheets(window).Current!)!, pad, e => e is ListBoxItem item && item.FindAncestorOfType<ListBox>()?.Name == "GamesList");
                    Save(window, $"cheat-database-game-focused-{p}", _out);
                    CloseAll(window);

                    Call(window, "ShowGraphicsSettings");
                    Save(window, $"graphics-settings-{p}", _out);
                    CloseAll(window);

                    Call(window, "ShowShaderSettings");
                    Save(window, $"shader-settings-{p}", _out);
                    CloseAll(window);

                    string shot = Path.Combine(_root, "shot.png");
                    UiTest.Capture(window).SavePng(shot);
                    _ = SheetLayer.Show(new ScreenshotWindow(shot, "Cobalt Harbor (Synthetic), Screenshot 1"), window);
                    Save(window, $"screenshot-{p}", _out);
                    CloseAll(window);

                    Play(window, 120);
                    pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
                    string[] lines = window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
                    pad.Down(Array.FindIndex(lines, l => l.StartsWith("Rewind", StringComparison.Ordinal)));
                    pad.A();
                    WaitFor(() => Sheets(window).Current is RewindReelWindow, "the reel");
                    Save(window, $"rewind-reel-{p}", _out);
                    CloseAll(window);
                }
                finally
                {
                    Stop(window);
                    window.Close();
                }
            }
        }, default);

        [WindowLookPngFact]
        public Task Resume_question() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in Sizes)
            {
                if (Directory.Exists(Path.Combine(_root, "States"))) Directory.Delete(Path.Combine(_root, "States"), recursive: true);
                (MainWindow window, PadDriver pad) = InGame(w, h, resumeAsk: true);
                try
                {
                    pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
                    string[] lines = window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
                    pad.Down(Array.FindIndex(lines, l => l.StartsWith("Close Game", StringComparison.Ordinal)));
                    pad.A();
                    pad.A();
                    Assert.IsType<ResumeWindow>(Sheets(window).Current);
                    Save(window, $"resume-{w}x{h}", _out);
                    pad.B();
                }
                finally
                {
                    Stop(window);
                    window.Close();
                }
            }
        }, default);

        private static ThemedSession Themed(int w, int h, bool artBook)
        {
            if (!artBook) return new ThemedSession(w, h, roms: ThemedFoldersTests.Library);
            string media = Path.Combine(Path.GetTempPath(), "EmuSenWindowLookMedia");
            SyntheticLibrary.WriteMedia(media);
            return new ThemedSession(w, h, a => a.EsdeMediaDirectory = media, themeDirectory: ArtBookNextFactAttribute.Folder, roms: ThemedFoldersTests.Library);
        }

        private void ThemedWalk(ThemedSession s, string p)
        {
            var hosts = FakeThemeHosts.Standard();
            ThemeBrowserSheetTests.Serve(hosts);
            hosts.Install(hosts["Plain Shelf"].Source);

            ThemedCollectionsTests.Enter(s, "snes");
            s.Settle();
            SceneGame folder = s.Themed.SelectedGame!;
            if (folder.Folder)
            {
                Call(s.Window, "ShowFolderEditor", folder);
                s.Settle();
                Save(s.Window, $"folder-editor-{p}", _out);
                CloseAll(s.Window);
                s.Pad.Down();
                s.Settle();
            }

            ThemedCollectionsTests.OpenMenu(s);
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GamelistFilterButton" });
            s.Pad.A();
            s.Settle();
            Save(s.Window, $"gamelist-filter-{p}", _out);
            CloseAll(s.Window);
            s.Settle();

            string rom = Path.Combine(s.RomDirectory, ThemedSession.SnesGames[2] + ".sfc");
            Call(s.Window, "ShowFindByName", rom, ThemedSession.SnesGames[2]);
            s.Settle();
            Save(s.Window, $"find-by-name-{p}", _out);
            CloseAll(s.Window);

            Call(s.Window, "ShowCoverPicker", rom, ThemedSession.SnesGames[2]);
            s.Settle();
            Save(s.Window, $"cover-picker-{p}", _out);
            CloseAll(s.Window);
            s.Settle();

            ThemeSettingsWindow settings = ThemeBrowserSheetTests.OpenSettings(s);
            ThemeBrowserWindow browser = ThemeBrowserSheetTests.OpenBrowser(s, settings);
            Save(s.Window, $"theme-browser-{p}", _out);
            ThemeDetailWindow detail = ThemeBrowserSheetTests.OpenDetail(s, browser, "Plain Shelf");
            Save(s.Window, $"theme-detail-{p}", _out);
            ThemeBrowserSheetTests.Click(ThemeBrowserSheetTests.Named<Button>(detail, "ThemeDetailAbout"));
            s.Settle();
            Save(s.Window, $"theme-about-{p}", _out);
            CloseAll(s.Window);
        }

        [WindowLookPngFact]
        public Task Synthetic_theme() => Session.Dispatch(() =>
        {
            foreach ((int w, int h) in Sizes)
                using (ThemedSession s = Themed(w, h, artBook: false)) ThemedWalk(s, $"synthetic-{w}x{h}");
        }, default);

        [WindowLookPngFact]
        public Task Art_book_next() => Session.Dispatch(() =>
        {
            if (!File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml"))) return;
            foreach ((int w, int h) in Sizes)
                using (ThemedSession s = Themed(w, h, artBook: true)) ThemedWalk(s, $"artbooknext-{w}x{h}");
        }, default);
    }

    // The scraping status in a big-screen session, part way through a run and finished, at both sizes.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowLookScrapePictureTool : ScrapeWindowFixture
    {
        public WindowLookScrapePictureTool(ITestOutputHelper output) : base(output) { }

        [WindowLookPngFact]
        public Task Scrape_status() => OnUi(() =>
        {
            string[] names = ["Aurora Drift (USA)", "Brass Lantern (USA)", "Cobalt Harbor (USA)", "Dune Relay (USA)", "Ember Circuit (USA)", "Zzz Unknown Dump"];
            for (int i = 0; i < names.Length; i++)
            {
                string path = Game(names[i], (byte)(10 + i * 7));
                if (i < 4) Server.Games.Add(new FakeGame(100 + i, names[i], Md5(path)));
            }
            foreach ((int w, int h) in WindowLookPictureTool.Sizes)
            {
                string media = EmuSen.Galaxia.Library.DataStore.Media;
                if (Directory.Exists(media)) Directory.Delete(media, recursive: true);
                Server.Gate = new SemaphoreSlim(0);
                MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone; }, bigScreen: true);
                window.Width = w;
                window.Height = h;
                Pump(200);
                Scrape(window, new ScrapeScope());
                ReleaseUntil(() => window.Progress!.Done == 2 && window.Progress.Current is { Step: ScrapeStep.Downloading }, "two games on the sheet");
                Pump(400);
                WindowLookPictureTool.Save(window, $"scrape-status-{w}x{h}", Out);
                Server.Gate.Release(1000);
                RunEnds(window);
                Pump(600);
                WindowLookPictureTool.Save(window, $"scrape-status-finished-{w}x{h}", Out);
                window.ScrapeStatusShown!.Close();
                window.Close();
                Server.Gate = null;
            }
        });
    }
}
