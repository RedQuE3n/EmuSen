using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A game's page in every section and deeper page, on the big screen and the desktop, recorded at both sizes and audited at 16:9 and 1280 by 720 - see EmuSen_InterfaceSpec.md §6.3.20 and EmuSen_BigPicture.md §50.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] GamePageViews = ["GamePage", "GamePageStates", "GamePageOptions", "GamePageMedia", "GamePageDetails", "GamePageCheats", "GamePageCover",
            "GamePageEdit", "GamePagePackage", "GamePageScrape", "GamePageViewer"];

        public static readonly string[] GamePageDesktopViews = ["GamePageDesktop", "GamePageDesktopStates", "GamePageDesktopOptions", "GamePageDesktopDetails", "GamePageDesktopEdit"];

        public static TheoryData<string, int, int> GamePageCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in GamePageViews.Concat(GamePageDesktopViews))
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(v, width, height);
            return data;
        }

        // The mockups' 16:9 size, the 1280 by 720 of §16, and the desktop at the mockups' size, audited and pictured but not recorded.
        public static TheoryData<string, int, int> GamePageFitCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string v in GamePageViews)
                foreach ((int w, int h) in new[] { (1920, 1080), (1280, 720) }) data.Add(v, w, h);
            return data;
        }

        private static string Cobalt(ThemedSession s) => Path.Combine(s.RomDirectory, ThemedSession.SnesGames[2] + ".sfc");

        // The synthetic library's pictures, three saved states, a taken screenshot, three cheats and the facts the mockups draw.
        private static ThemedSession GamePageSession(string view, int width, int height)
        {
            string media = Path.Combine(Path.GetTempPath(), "EmuSenGamePageMedia");
            SyntheticLibrary.WriteMedia(media);
            bool desktop = view.Contains("Desktop", StringComparison.Ordinal);
            var s = new ThemedSession(width, height, a =>
            {
                a.EsdeMediaDirectory = media;
                if (desktop) { a.BigScreen = false; a.LibraryView = AppSettings.LibraryList; }
            });
            string rom = Cobalt(s);
            var host = (IGamePageHost)s.Window;
            foreach ((string key, string value) in new[] { (GameMetadata.Developer, "Lantern Works"), (GameMetadata.Publisher, "Publisher P"), (GameMetadata.ReleaseDate, "1994-03-01"),
                (GameMetadata.Description, "Cobalt Harbor is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it. This second sentence is here so that the description is long enough to wrap over several lines.") })
                host.EditField(rom, key, value);
            // The page's words about time from a fixed evening, so no recording crosses midnight.
            DateTime now = new(2026, 11, 1, 21, 42, 0);
            typeof(MainWindow).GetField("PageClock", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)!.SetValue(null, (Func<DateTime>)(() => now));
            foreach ((int slot, TimeSpan ago, Color colour) in new[] { (1, TimeSpan.FromHours(2), Colors.SteelBlue), (2, TimeSpan.FromDays(1), Colors.SeaGreen), (3, TimeSpan.FromDays(3), Colors.Indigo) })
            {
                string file = SaveLibrary.StatePathFor(rom, slot, s.Settings.StateDirectory);
                Directory.CreateDirectory(Path.GetDirectoryName(file)!);
                File.WriteAllBytes(file, [1]);
                SyntheticLibrary.Label(SaveLibrary.PicturePathFor(file), 256, 224, colour, $"Cobalt Harbor · Slot {slot}");
                File.SetLastWriteTime(file, now - ago);
            }
            string resume = SaveLibrary.ResumeStatePathFor(rom, s.Settings.StateDirectory);
            File.WriteAllBytes(resume, [1]);
            SyntheticLibrary.Label(SaveLibrary.PicturePathFor(resume), 256, 224, Colors.SlateGray, "Cobalt Harbor · Auto");
            File.SetLastWriteTime(resume, now - TimeSpan.FromMinutes(12));
            string shots = Path.Combine(DataStore.Screenshots);
            Directory.CreateDirectory(shots);
            SyntheticLibrary.Label(Path.Combine(shots, ThemedSession.SnesGames[2] + "-20261101-214200.png"), 512, 448, Colors.DarkCyan, "Cobalt Harbor · Screenshot");
            var cheats = new CheatFile();
            foreach ((string name, bool on) in new[] { ("Infinite lives", true), ("All doors open", false), ("Start with the choir", false) })
                cheats.Cheats.Add(new CheatFileEntry { Description = name, Enabled = on });
            GameCheats.For(RuntimeCatalog.ConsoleForRom(rom), rom)?.Save(cheats);
            s.Settle();
            return s;
        }

        private static void OpenGamePage(ThemedSession s, string view)
        {
            string rom = Cobalt(s);
            MainWindow w = s.Window;
            string section = view.Replace("Desktop", "", StringComparison.Ordinal) switch
            {
                "GamePageStates" => GamePage.States,
                "GamePageOptions" or "GamePageCheats" or "GamePagePackage" => GamePage.Options,
                "GamePageMedia" or "GamePageViewer" => GamePage.Media,
                "GamePageDetails" or "GamePageCover" or "GamePageEdit" or "GamePageScrape" => GamePage.Details,
                _ => GamePage.Overview,
            };
            w.ShowGamePage(rom, section);
            s.Settle();
            GamePage page = w.GamePageShown!;
            PageLayout layout = w.GamePageLayout!;
            switch (view.Replace("Desktop", "", StringComparison.Ordinal))
            {
                case "GamePageStates": layout.FocusContent(page.StateList.First(st => st.Slot == "Slot 3").Path); break;
                case "GamePageOptions" or "GamePageMedia" or "GamePageDetails": layout.FocusContent(); break;
                case "GamePageCheats": page.ChooseRow("cheats"); break;
                case "GamePagePackage": page.ChooseRow("package"); break;
                case "GamePageCover": page.ChooseRow("cover"); layout.FocusContent(); break;
                case "GamePageEdit": page.ChooseRow("edit"); break;
                case "GamePageScrape": page.ChooseRow("scrape"); break;
                case "GamePageViewer": page.ChooseTile(page.MediaList[0].Path); break;
            }
            s.Settle();
            // Every picture decoded and the page drawn with it, so the recording does not race the cache.
            var covers = (CoverArtCache)typeof(MainWindow).GetField("_covers", Hidden)!.GetValue(w)!;
            for (int i = 0; i < 400 && covers.Busy; i++) { s.Run(20); System.Threading.Thread.Sleep(5); }
            s.Run(300);
        }

        private static void PictureGamePage(ThemedSession s, string name)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_U4_PNG") is not { Length: > 0 } folder) return;
            Directory.CreateDirectory(folder);
            UiTest.Capture(s.Window).SavePng(Path.Combine(folder, name + ".png"));
        }

        [Theory]
        [MemberData(nameof(GamePageCases))]
        public Task Every_game_page_state_is_as_its_specification_records_it(string view, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"{view}-{width}x{height}";
            using ThemedSession s = GamePageSession(view, width, height);
            InterfaceRecorder.Normalise = InterfaceSpecFiles.Normaliser(_root, s.Root);
            InterfaceRecorder.Volatile = Volatile;
            try
            {
                OpenGamePage(s, view);
                bool desktop = view.Contains("Desktop", StringComparison.Ordinal);
                Control root = s.Window.GamePageLayout!;
                SpecDocument doc = InterfaceRecorder.Tree(root, new InterfaceRecorder.Options(name, desktop ? "window" : "menu", SmallestText: desktop ? 0 : FitAudit.SmallestText), out _);
                doc.Header["case"] = view;
                doc.Header["walk"] = "none: the page moves its own focus, drawn as a ring or a bar";
                PictureGamePage(s, name);
                if (!desktop) InterfaceSpecFiles.Presses(doc, s.Pad, s.Window, () => { s.Run(200); return GamePageSeen(s.Window); }, UiButton.Down, UiButton.Up, UiButton.Back);
                Assert.Empty(InterfaceSpecFiles.Check(doc, _out, Known));
            }
            finally
            {
                InterfaceRecorder.Normalise = t => t;
                InterfaceRecorder.Volatile = _ => null;
                ResetPageClock();
            }
        }, default);

        [Theory]
        [MemberData(nameof(GamePageFitCases))]
        public Task Every_game_page_state_fits_at_16_9_and_1280_by_720(string view, int width, int height) => Session.Dispatch(() =>
        {
            using ThemedSession s = GamePageSession(view, width, height);
            OpenGamePage(s, view);
            PictureGamePage(s, $"{view}-{width}x{height}");
            List<string> faults = FitAudit.Check(s.Window.GamePageLayout!);
            ResetPageClock();
            foreach (string f in faults) _out.WriteLine($"{view} {width}x{height}: {f}");
            Assert.Empty(faults);
        }, default);

        private static void ResetPageClock() =>
            typeof(MainWindow).GetField("PageClock", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)!.SetValue(null, (Func<DateTime>)(() => DateTime.Now));

        private static SortedDictionary<string, string> GamePageSeen(MainWindow w)
        {
            var seen = new SortedDictionary<string, string>(StringComparer.Ordinal);
            if (w.GamePageShown is not { } page || w.GamePageLayout is not { } layout) { seen["page"] = "closed"; return seen; }
            seen["page"] = page.Deeper ?? page.Section;
            seen["focus"] = layout.Focused.InActions ? $"action {layout.Focused.Key}" : $"content {InterfaceRecorder.Normalise(layout.Focused.Key ?? "none")}";
            return seen;
        }
    }
}
