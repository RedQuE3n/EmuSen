using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The scrape log: a line for each request the fake server answers and one for the run, none holding a credential - see EmuSen_LibraryTools.md §9.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeLogTests : ScrapeWindowFixture
    {
        public ScrapeLogTests(ITestOutputHelper output) : base(output) { }

        private string Logs => Path.Combine(Root, "Logs");

        private string[] LogLines() => Directory.Exists(Logs)
            ? Directory.GetFiles(Logs, "scrape_*.log").SelectMany(File.ReadAllLines).ToArray()
            : Array.Empty<string>();

        private MainWindow RunThree()
        {
            string found = Game("A Found (USA)", 10), unknown = Game("B Unknown (USA)", 20), broken = Game("C Broken (USA)", 30);
            Server.Games.Add(new FakeGame(77, "Found", Md5(found)));
            Server.StatusByMd5[Md5(broken)] = (400, "Erreur : requete malformee devpassword=FAKEDEVPASSWORD&sspassword=FAKEMEMBERSECRET");
            MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.LogDirectory = Logs; });
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            return window;
        }

        [Fact]
        public Task Each_request_is_one_line_with_its_kind_status_timing_worker_and_quota_and_the_run_one_summary() => OnUi(() =>
        {
            RunThree();
            string[] lines = LogLines();
            string[] requests = lines.Where(l => l.Contains("\trequest\t")).ToArray();
            Assert.Equal(Server.Asked.Count(u => u.Contains("screenscraper.fr")), requests.Length);
            Assert.Equal(3, requests.Count(l => l.Contains("\tkind=info\t")));
            Assert.Contains(requests, l => l.Contains("game=A Found (USA).sfc\tkind=cover\tstatus=200\toutcome=Saved\t"));
            Assert.Contains(requests, l => l.Contains("game=C Broken (USA).sfc\tkind=info\tstatus=400\t"));
            foreach (string field in new[] { "\tbytes=", "\tms=", "\tworker=0\t", "\tthreads=1/1\t", "\tpace_ms=", "\tcap_ms=", "\thash_ms=" })
                Assert.All(requests, l => Assert.Contains(field, l));
            Assert.Contains(requests, l => l.Contains("\tkind=info\t") && l.Contains("\tmaxthreads=1\tmaxrequestspermin=60\tmaxdownloadspeed=128\trequeststoday="));
            string run = Assert.Single(lines, l => l.Contains("\trun\t"));
            Assert.Contains($"\tend=done\tgames=3\trequests={requests.Length}\t", run);
            foreach (string field in new[] { "\twall_ms=", "\tworkers=1\t", "\tpace_ms=", "\tcap_ms=", "\tserver_ms=", "\thash_ms=", "\tidle_ms=", "\town_ms=" })
                Assert.Contains(field, run);
        });

        // A file is read once for its three hashes: a later lookup of the same file at the same size and time hashes nothing.
        [Fact]
        public Task A_file_asked_again_is_not_hashed_again() => OnUi(() =>
        {
            MainWindow window = RunThree();
            string unknown = Path.Combine(RomDir, "B Unknown (USA).sfc");
            int before = LogLines().Length;
            Scrape(window, new ScrapeScope(Game: unknown));
            RunEnds(window);
            string again = Assert.Single(LogLines().Skip(before), l => l.Contains("\trequest\t"));
            Assert.Contains("game=B Unknown (USA).sfc\tkind=info\t", again);
            Assert.Contains("\thash_ms=-\t", again);
            Assert.All(LogLines().Take(before).Where(l => l.Contains("\tkind=info\t")), l => Assert.DoesNotContain("\thash_ms=-\t", l));
        });

        // The workers the account allows start as soon as an answer says how many, not after the first game ends.
        [Fact]
        public Task The_other_workers_start_with_the_first_answer() => OnUi(() =>
        {
            Server.User = FakeScreenScraper.Quota(maxThreads: 3, perMinute: 60, perDay: 20000, koPerDay: 2000, today: 10, koToday: 1);
            foreach ((string name, byte seed, long id) in new[] { ("A Found (USA)", (byte)10, 77L), ("B Found (USA)", (byte)20, 78L), ("C Found (USA)", (byte)30, 79L) })
                Server.Games.Add(new FakeGame(id, name, Md5(Game(name, seed))));
            MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.LogDirectory = Logs; a.ScrapeThreads = 0; });
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            string[] requests = LogLines().Where(l => l.Contains("\trequest\t")).ToArray();
            Assert.Contains("\tthreads=1/1\t", requests[0]);
            string firstPicture = requests.First(l => !l.Contains("\tkind=info\t"));
            Assert.Contains("\tthreads=3/3\t", firstPicture);
        });

        // A lookup ScreenScraper could not answer is queued to retry, each retry said in the error log with its game - see EmuSen_Backend.md §40.5.
        [Fact]
        public Task A_retry_is_said_in_the_error_log_with_its_game() => OnUi(() =>
        {
            string broken = Game("D Unanswered (USA)", 40);
            Server.StatusByMd5[Md5(broken)] = (500, "Erreur interne du serveur");
            MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.LogDirectory = Logs; });
            Scrape(window, new ScrapeScope(Game: broken));
            RunEnds(window);
            string[] lines = File.ReadAllLines(EmuSen.Galaxia.Library.ErrorLog.PathFor(DateTime.Now));
            int at = Array.FindIndex(lines, l => l.Contains("WARN [scraping] ScreenScraper could not answer; queued to retry: "));
            Assert.True(at >= 0, string.Join("\n", lines));
            Assert.Equal($"    context: {broken}", lines[at + 1]);
        });

        [Fact]
        public Task Nothing_the_scrape_log_holds_is_a_credential_or_an_address() => OnUi(() =>
        {
            new MemberAccount("FAKEMEMBERUSER", "FAKEMEMBERSECRET") { Verified = DateTime.UtcNow }.Save();
            RunThree();
            string all = string.Join("\n", LogLines());
            Assert.Contains("\trequest\t", all);
            foreach (string secret in new[] { FakeScreenScraper.DevPassword, FakeScreenScraper.DevId, "FAKEMEMBERUSER", "FAKEMEMBERSECRET", "devpassword", "sspassword", "ssid=", "https://" })
                Assert.DoesNotContain(secret, all);
        });
    }
}
