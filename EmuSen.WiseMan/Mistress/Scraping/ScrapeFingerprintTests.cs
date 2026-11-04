using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // A big file found by its name and size from a fingerprint of its two ends, read in full only after a miss, its checksums taken later while no game is on screen - see EmuSen_LibraryTools.md §9.12.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeFingerprintTests : ScrapeWindowFixture
    {
        private const long Size = 64L << 20;
        private const long Edges = 2L << 20;

        public ScrapeFingerprintTests(ITestOutputHelper output) : base(output) => NoNetwork.FingerprintAbove = 32L << 20;

        private string Logs => Path.Combine(Root, "Logs");

        private string[] LogLines() => Directory.Exists(Logs) ? Directory.GetFiles(Logs, "scrape_*.log").SelectMany(File.ReadAllLines).ToArray() : [];

        private string[] Infos(string file) => LogLines().Where(l => l.Contains($"\tgame={file}\tkind=info\t")).ToArray();

        private string LastRun() => LogLines().Last(l => l.Contains("\trun\t"));

        // A disc of its own first and last bytes, sparse between, so each has its own fingerprint and costs no disk.
        private string Disc(string name, byte seed)
        {
            string path = Path.Combine(RomDir, name + ".sfc");
            using FileStream f = File.Create(path);
            f.SetLength(Size);
            f.WriteByte(seed);
            f.Seek(-1, SeekOrigin.End);
            f.WriteByte(seed);
            return path;
        }

        private MainWindow OpenLogged() => Open(settings: a => { a.OpenEmuFallback = false; a.LogDirectory = Logs; });

        // A run with a game marked on screen before its first answer, so the background checksums wait.
        private void ScrapeWhilePlaying(MainWindow w)
        {
            Hold();
            Scrape(w, new ScrapeScope());
            w.ScrapeNativeService!.Playing(true);
            Server.Gate!.Release(1000);
            RunEnds(w);
        }

        private static (string Md5, string? Crc32, string? Sha1, string Fingerprint) FileRow(string path)
        {
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = Path.Combine(MediaStore.DefaultRoot, MediaStore.FileName), Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
            c.Open();
            using SqliteCommand q = c.CreateCommand();
            q.CommandText = "SELECT md5, crc32, sha1, fingerprint FROM scrape_file WHERE path = $path";
            q.Parameters.AddWithValue("$path", path);
            using SqliteDataReader r = q.ExecuteReader();
            Assert.True(r.Read(), path);
            return (r.GetString(0), r.IsDBNull(1) ? null : r.GetString(1), r.IsDBNull(2) ? null : r.GetString(2), r.GetString(3));
        }

        [Fact]
        public Task A_big_file_is_found_by_name_and_size_with_two_mib_read() => OnUi(() =>
        {
            string big = Disc("Big Disc (USA)", 1);
            Server.Games.Add(new FakeGame(90, "Big Disc") { ByNameAndSize = ("Big Disc (USA).sfc", Size) });
            MainWindow window = OpenLogged();
            ScrapeWhilePlaying(window);

            string info = Assert.Single(Infos("Big Disc (USA).sfc"));
            Assert.Contains("\tmatch=name+size\t", info);
            Assert.Contains("\thash_ms=-\t", info);
            Assert.DoesNotContain("\tfingerprint_ms=-", info);
            Assert.EndsWith($"\tread_bytes={Edges}", LastRun());
            string asked = Assert.Single(Server.JeuInfos);
            Assert.DoesNotContain("md5=", asked);
            Assert.DoesNotContain("crc=", asked);
            Assert.DoesNotContain("sha1=", asked);

            ScrapedRecord record = window.ScrapeNativeService!.FoundFor(big)!;
            Assert.Equal(("", "name+size", 40), (record.Md5, record.MatchedBy, record.Fingerprint.Length));
            Assert.Equal(("", (string?)null, (string?)null, record.Fingerprint), FileRow(big));
            Assert.Contains(window.ScrapeMedia("", Size, record.Fingerprint), m => m.Type == "cover");
        });

        [Fact]
        public Task A_big_file_not_found_by_name_falls_back_to_its_hashes() => OnUi(() =>
        {
            string big = Disc("Big Hashed (USA)", 2);
            Server.Games.Add(new FakeGame(91, "Big Hashed", Md5(big)));
            MainWindow window = OpenLogged();
            ScrapeWhilePlaying(window);

            string[] infos = Infos("Big Hashed (USA).sfc");
            Assert.Equal(2, infos.Length);
            Assert.Contains("\tmatch=name+size\t", infos[0]);
            Assert.Contains("\tstatus=404\t", infos[0]);
            Assert.Contains("\tmatch=hash\t", infos[1]);
            Assert.DoesNotContain("\thash_ms=-\t", infos[1]);
            Assert.EndsWith($"\tread_bytes={Edges + Size}", LastRun());

            ScrapedRecord record = window.ScrapeNativeService!.FoundFor(big)!;
            Assert.Equal(("", "file", 40), (record.Md5, record.MatchedBy, record.Fingerprint.Length));
            var row = FileRow(big);
            Assert.Equal((Md5(big), record.Fingerprint), (row.Md5, row.Fingerprint));
            Assert.NotNull(row.Crc32);
            Assert.NotNull(row.Sha1);
        });

        [Fact]
        public Task The_checksums_arrive_later_and_never_while_a_game_is_on_screen() => OnUi(() =>
        {
            string big = Disc("Big Later (USA)", 3);
            Server.Games.Add(new FakeGame(92, "Big Later") { ByNameAndSize = ("Big Later (USA).sfc", Size) });
            MainWindow window = OpenLogged();
            ScrapeWhilePlaying(window);
            string fingerprint = window.ScrapeNativeService!.FoundFor(big)!.Fingerprint;

            Pump(1500);
            Assert.Equal(("", (string?)null, (string?)null, fingerprint), FileRow(big));

            window.ScrapeNativeService!.Playing(false);
            PumpUntil(() => FileRow(big).Md5.Length > 0, "the checksums taken in the background");
            var row = FileRow(big);
            Assert.Equal((Md5(big), fingerprint), (row.Md5, row.Fingerprint));
            Assert.NotNull(row.Crc32);
            Assert.NotNull(row.Sha1);
            ScrapedRecord record = window.ScrapeNativeService!.FoundFor(big)!;
            Assert.Equal(("", fingerprint, "name+size"), (record.Md5, record.Fingerprint, record.MatchedBy));
        });

        [Fact]
        public Task A_big_file_s_art_is_found_again_by_its_fingerprint_after_a_restart() => OnUi(() =>
        {
            string big = Disc("Big Kept (USA)", 4);
            Server.Games.Add(new FakeGame(93, "Big Kept") { ByNameAndSize = ("Big Kept (USA).sfc", Size) });
            MainWindow first = OpenLogged();
            Scrape(first, new ScrapeScope());
            RunEnds(first);
            string fingerprint = first.ScrapeNativeService!.FoundFor(big)!.Fingerprint;
            first.Close();

            int asked = Server.Asked.Count;
            MainWindow second = OpenLogged();
            Scrape(second, new ScrapeScope(Game: big));
            RunEnds(second);
            Assert.Equal(asked, Server.Asked.Count);
            Assert.EndsWith("\tread_bytes=0", LastRun());
            Assert.Equal(fingerprint, second.ScrapeNativeService!.FoundFor(big)!.Fingerprint);
            StoredMedia cover = Assert.Single(second.ScrapeMedia("", Size, fingerprint), m => m.Type == "cover");
            Assert.True(File.Exists(Path.Combine(MediaStore.DefaultRoot, cover.RelativePath)));
            Assert.True(second.ScrapeNativeService!.OutcomesByPath()[big].HasCover);
        });
    }
}
