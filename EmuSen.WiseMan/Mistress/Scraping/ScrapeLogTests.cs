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
