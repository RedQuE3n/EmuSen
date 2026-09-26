using System;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    public sealed class ScrapeStatusPngFactAttribute : FactAttribute
    {
        public ScrapeStatusPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes the scraping status window's pictures to ~/.cache/emusen/bigpicture/png/scrape-status/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_Settings_Reference.md §4.57";
        }
    }

    // The status window mid-run, stopped by the quota and finished, drawn over the 1280 by 800 desktop window; the sheet and the sign-in in a big-screen session; outside the repository.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeStatusPictureTool : ScrapeWindowFixture
    {
        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "scrape-status");

        public ScrapeStatusPictureTool(ITestOutputHelper output) : base(output) { }

        private void Save(RenderedFrame frame, string name)
        {
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            Out.WriteLine(path);
        }

        // The owned window where a desktop would put it, centred over its owner, with a one-pixel edge.
        private static RenderedFrame Over(MainWindow main, Window status)
        {
            RenderedFrame back = UiTest.Capture(main), front = UiTest.Capture(status);
            byte[] rgba = (byte[])back.Rgba.Clone();
            int left = (back.Width - front.Width) / 2, top = (back.Height - front.Height) / 2;
            for (int y = -1; y <= front.Height; y++)
                for (int x = -1; x <= front.Width; x++)
                {
                    int bx = left + x, by = top + y;
                    if (bx < 0 || by < 0 || bx >= back.Width || by >= back.Height) continue;
                    int to = (by * back.Width + bx) * 4;
                    bool edge = x < 0 || y < 0 || x == front.Width || y == front.Height;
                    if (edge) { rgba[to] = rgba[to + 1] = rgba[to + 2] = 0x60; rgba[to + 3] = 0xFF; continue; }
                    Array.Copy(front.Rgba, (y * front.Width + x) * 4, rgba, to, 4);
                }
            return new RenderedFrame(rgba, back.Width, back.Height);
        }

        // A picture per kind, so the thumbnail has something to show: a box of the kind's colour with bands.
        private static byte[] Picture(string media)
        {
            (byte r, byte g, byte b) = media.StartsWith("box", StringComparison.Ordinal) ? ((byte)0xC8, (byte)0x3C, (byte)0x3C)
                : media.StartsWith("ss", StringComparison.Ordinal) ? ((byte)0x3C, (byte)0x8C, (byte)0xC8)
                : media.StartsWith("wheel", StringComparison.Ordinal) ? ((byte)0xE0, (byte)0xB0, (byte)0x30) : ((byte)0x50, (byte)0xA8, (byte)0x60);
            const int w = 180, h = 240;
            var rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    int i = (y * w + x) * 4;
                    double shade = 0.55 + 0.45 * y / h;
                    bool band = (y / 24) % 3 == 1 && x > 16 && x < w - 16;
                    rgba[i] = (byte)(band ? 0xF0 : r * shade);
                    rgba[i + 1] = (byte)(band ? 0xF0 : g * shade);
                    rgba[i + 2] = (byte)(band ? 0xF0 : b * shade);
                    rgba[i + 3] = 0xFF;
                }
            string temp = Path.Combine(Path.GetTempPath(), $"EmuSenScrapePicture-{Guid.NewGuid():N}.png");
            new RenderedFrame(rgba, w, h).SavePng(temp);
            byte[] png = File.ReadAllBytes(temp);
            File.Delete(temp);
            return png;
        }

        // The fake server, with its media answered by real pictures.
        private sealed class RealPictures : DelegatingHandler
        {
            public RealPictures(HttpMessageHandler inner) : base(inner) { }

            protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
            {
                HttpResponseMessage answer = await base.SendAsync(request, cancellationToken);
                string url = request.RequestUri!.AbsoluteUri;
                if (!url.Contains("/mediaJeu.php") || answer.StatusCode != HttpStatusCode.OK) return answer;
                var content = new ByteArrayContent(Picture(FakeScreenScraper.Param(url, "media")));
                content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/png");
                return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
            }
        }

        private void Library(int found)
        {
            string[] names = ["Super Mario World (USA)", "F-Zero (USA)", "Pilotwings (USA)", "ActRaiser (USA)", "Sim City (USA)", "Zzz Unknown Dump", "Broken Header (USA)"];
            for (int i = 0; i < names.Length; i++)
            {
                string path = Game(names[i], (byte)(10 + i * 7));
                if (i < found) Server.Games.Add(new FakeGame(100 + i, names[i], Md5(path)));
                if (names[i].StartsWith("Broken", StringComparison.Ordinal)) Server.StatusByMd5[Md5(path)] = (400, "Erreur : requete malformee");
            }
        }

        // Each picture's run starts from an empty store in the test's own temporary home, so no game is skipped as already scraped.
        private void Fresh()
        {
            Pump(200);
            string media = EmuSen.Galaxia.Library.DataStore.Media;
            Assert.StartsWith(Root, media);
            if (Directory.Exists(media)) Directory.Delete(media, recursive: true);
        }

        [ScrapeStatusPngFact]
        public Task Scrape_status_pictures() => OnUi(() =>
        {
            Directory.CreateDirectory(PngFolder);
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(new RealPictures(Server))));
            new MemberAccount("player", "FAKEPLAYERPW") { Verified = DateTime.UtcNow }.Save();
            Library(found: 5);

            Hold();
            MainWindow window = Open(settings: a => a.OpenEmuFallback = true);
            Scrape(window, new ScrapeScope());
            ScrapeStatusWindow status = window.ScrapeStatusShown!;
            ReleaseUntil(() => window.Progress!.Done == 2 && window.Progress.Current is { Step: ScrapeStep.Downloading, Kind: "screenshot" }, "the third game's screenshot");
            Pump(400);
            Save(Over(window, status), "mid-run-1280x800");

            Server.Gate.Release(1000);
            RunEnds(window);
            Pump(600);
            Save(Over(window, status), "finished-1280x800");
            window.Close();
            Fresh();

            Server.Gate = null;
            Server.User = FakeScreenScraper.Quota(maxThreads: 1, perMinute: 60, perDay: 10000, koPerDay: 1000, today: 9799, koToday: 12);
            window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            Pump(600);
            Save(Over(window, window.ScrapeStatusShown!), "quota-stop-1280x800");
            window.Close();
            Fresh();

            Server.User = FakeScreenScraper.Quota(maxThreads: 1, perMinute: 60, perDay: 20000, koPerDay: 2000, today: 10, koToday: 1);
            Hold();
            window = Open(settings: a => a.OpenEmuFallback = false, bigScreen: true);
            Scrape(window, new ScrapeScope());
            ReleaseUntil(() => window.Progress!.Done == 2 && window.Progress.Current is { Step: ScrapeStep.Downloading }, "two games on the sheet");
            Pump(400);
            Save(UiTest.Capture(window), "sheet-bigscreen-1280x800");
            Server.Gate.Release(1000);
            RunEnds(window);
            Pump(600);
            Save(UiTest.Capture(window), "sheet-bigscreen-finished-1280x800");
            window.ScrapeStatusShown!.Close();

            var pad = new PadDriver(window);
            pad.Start();
            int at = PadMenu(window).FindIndex(e => e.Text() == "Scrape Games...");
            pad.Down(at);
            pad.A();
            Pump(300);
            Control sheet = RootOf(Sheets(window).Current!);
            Named<Control>(Sheets(window).Current!, "ScreenScraperLogOutButton").BringIntoView();
            Pump(300);
            Save(UiTest.Capture(window), "signin-sheet-bigscreen-1280x800");
            Click(Named<Button>(Sheets(window).Current!, "ScreenScraperLogOutButton"));
            Pump(300);
            Save(UiTest.Capture(window), "signed-out-sheet-bigscreen-1280x800");
        });
    }
}
