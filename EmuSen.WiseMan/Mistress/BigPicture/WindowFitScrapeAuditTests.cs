using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.Scraping;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The scraping windows in the states only a run over the fake ScreenScraper reaches: part way, finished, and Find by Name's results - see EmuSen_Settings_Reference.md §4.81.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowFitScrapeAuditTests : ScrapeWindowFixture
    {
        public WindowFitScrapeAuditTests(ITestOutputHelper output) : base(output) { }

        public static TheoryData<string, int, int> Cases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string w in new[] { "ScrapeStatusRunning", "ScrapeStatusFinished", "FindByNameResults" })
                foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(w, width, height);
            return data;
        }

        [Theory]
        [MemberData(nameof(Cases))]
        public Task Nothing_in_the_window_is_cut_off_past_its_panel_or_drawn_over_anything_else(string which, int width, int height) => OnUi(() =>
        {
            string[] names = ["Aurora Drift (USA)", "Brass Lantern (USA)", "Cobalt Harbor (USA)", "Dune Relay (USA)", "A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1)", "Zzz Unknown Dump"];
            string first = "";
            for (int i = 0; i < names.Length; i++)
            {
                string path = Game(names[i], (byte)(10 + i * 7));
                if (i == 0) first = path;
                if (i < 5) Server.Games.Add(new FakeGame(100 + i, names[i], Md5(path)));
            }
            Server.Games.Add(new FakeGame(900, "Super Mario World"));
            Server.Games.Add(new FakeGame(901, "Super Mario Kart: A Name Long Enough To Need The Whole Row And More"));
            if (which == "ScrapeStatusRunning") Hold();
            MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone; }, bigScreen: true);
            window.Width = width;
            window.Height = height;
            Pump(200);
            switch (which)
            {
                case "ScrapeStatusRunning":
                    Scrape(window, new ScrapeScope());
                    ReleaseUntil(() => window.Progress!.Done == 3 && window.Progress.Current is { Step: ScrapeStep.Downloading }, "three games");
                    Pump(400);
                    break;
                case "ScrapeStatusFinished":
                    Scrape(window, new ScrapeScope());
                    RunEnds(window);
                    Pump(600);
                    break;
                case "FindByNameResults":
                    typeof(MainWindow).GetMethod("ShowFindByName", Hidden)!.Invoke(window, [first, "Aurora Drift"]);
                    FindByNameWindow find = Assert.IsType<FindByNameWindow>(WindowFitAuditTests.Sheets(window).Current);
                    Named<TextBox>(find, "FindByNameText").Text = "Super Mario";
                    Click(Named<Button>(find, "FindByNameSearch"));
                    WaitFor(() => find.Results.Count() == 2, "the results");
                    break;
            }
            List<string> faults = WindowFitAuditTests.Audit(window, $"{which}-{width}x{height}", Out);
            Server.Gate?.Release(1000);
            Assert.Empty(faults);
        });
    }
}
