using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    public sealed class ScrapeExtrasPngFactAttribute : FactAttribute
    {
        public ScrapeExtrasPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes Pass 8's pictures to ~/.cache/emusen/bigpicture/png/pass8/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §38";
        }
    }

    // The Scraping tab's new rows and Find by Name, on the desktop and as sheets in a big-screen session, at 1280 by 800 and 1920 by 1200; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeExtrasPictureTool : ScrapeWindowFixture
    {
        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "pass8");

        public ScrapeExtrasPictureTool(ITestOutputHelper output) : base(output) { }

        private void Save(RenderedFrame frame, string name)
        {
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            Out.WriteLine(path);
        }

        private static RenderedFrame Over(MainWindow main, Window front)
        {
            RenderedFrame back = UiTest.Capture(main), top = UiTest.Capture(front);
            byte[] rgba = (byte[])back.Rgba.Clone();
            int left = Math.Max(0, (back.Width - top.Width) / 2), up = Math.Max(0, (back.Height - top.Height) / 2);
            for (int y = 0; y < top.Height && up + y < back.Height; y++)
                for (int x = 0; x < top.Width && left + x < back.Width; x++)
                    Array.Copy(top.Rgba, (y * top.Width + x) * 4, rgba, ((up + y) * back.Width + left + x) * 4, 4);
            return new RenderedFrame(rgba, back.Width, back.Height);
        }

        private MainWindow OpenAt(int w, int h, bool bigScreen)
        {
            MainWindow window = Open(bigScreen: bigScreen);
            window.Width = w;
            window.Height = h;
            Pump(300);
            return window;
        }

        [ScrapeExtrasPngFact]
        public Task Pass_8_pictures() => OnUi(() =>
        {
            Directory.CreateDirectory(PngFolder);
            string hack = Game("SMW Hack (USA) [Hack]", 5);
            Game("F-Zero (USA)", 1);
            Server.Games.Add(new FakeGame(900, "Super Mario World") { Media = FakeGame.AllMedia });
            Server.Games.Add(new FakeGame(901, "Super Mario Kart"));
            Server.Games.Add(new FakeGame(902, "Super Mario All-Stars"));

            foreach ((int w, int h) in new[] { (1280, 800), (1920, 1200) })
            {
                string size = $"{w}x{h}";

                // The desktop: Preferences as a window over Mistress's, on the Scraping tab.
                MainWindow window = OpenAt(w, h, bigScreen: false);
                var prefs = new PreferencesWindow(AppSettings.Load(), window) { SizeToContent = SizeToContent.Manual, Height = h - 80 };
                Windows.Add(prefs);
                prefs.Show();
                prefs.ShowTab(PreferencesWindow.ScrapingTab);
                Pump(300);
                foreach ((string control, string name) in new[] { ("ScrapeGameNamesSwitch", "desktop-fetch-more-and-game-names"), ("ScrapeStatusButton", "desktop-scrape-criteria"), ("ScrapeCleanUpButton", "desktop-orphaned-media") })
                {
                    Named<Control>(prefs, control).BringIntoView();
                    Pump(300);
                    Save(Over(window, prefs), $"{name}-{size}");
                }
                prefs.Close();

                typeof(MainWindow).GetMethod("ShowFindByName", Hidden)!.Invoke(window, [hack, "SMW Hack (USA) [Hack]"]);
                FindByNameWindow find = window.FindByNameShown!;
                Pump(300);
                Save(Over(window, find), $"desktop-find-by-name-{size}");
                Named<TextBox>(find, "FindByNameText").Text = "Super Mario";
                Task searching = find.SearchAsync();
                WaitFor(() => searching.IsCompleted, "the search");
                Pump(300);
                Save(Over(window, find), $"desktop-find-by-name-results-{size}");
                find.Close();
                window.Close();

                // A big-screen session: the same as sheets, driven by the pad.
                window = OpenAt(w, h, bigScreen: true);
                var pad = new PadDriver(window);
                pad.Start();
                int at = PadMenu(window).FindIndex(e => e.Text() == "Scrape Games...");
                pad.Down(at);
                pad.A();
                Pump(300);
                foreach ((string control, string name) in new[] { ("ScrapeVideosSwitch", "sheet-fetch-more"), ("ScrapeGameNamesSwitch", "sheet-game-names"), ("ScrapeStatusButton", "sheet-scrape-criteria"), ("ScrapeCleanUpButton", "sheet-orphaned-media") })
                {
                    Named<Control>(Sheets(window).Current!, control).BringIntoView();
                    Pump(300);
                    Save(UiTest.Capture(window), $"{name}-{size}");
                }
                Sheets(window).Current!.Close();
                Pump(200);

                typeof(MainWindow).GetMethod("ShowFindByName", Hidden)!.Invoke(window, [hack, "SMW Hack (USA) [Hack]"]);
                find = window.FindByNameShown!;
                Pump(300);
                Named<TextBox>(find, "FindByNameText").Text = "Super Mario";
                searching = find.SearchAsync();
                WaitFor(() => searching.IsCompleted, "the search");
                Pump(300);
                Save(UiTest.Capture(window), $"sheet-find-by-name-results-{size}");
                find.Close();
                window.Close();
            }
        });
    }
}
