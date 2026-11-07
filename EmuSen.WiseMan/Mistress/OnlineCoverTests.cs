using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.Mistress.Library;
using Microsoft.Data.Sqlite;
using EmuSen.Crystal.Systems;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // OpenVGDB names a game, libretro's server has its box: emusen-scrape's failover, a run with ScreenScraper off, against a fake server for both - see EmuSen_Settings_Reference.md §4.39 and EmuSen_Backend.md §40.5.
    [Collection(EmuSen.WiseMan.Fixtures.TestCollections.ProcessGlobals)]
    public class OnlineCoverTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenOnlineCoverTests", Guid.NewGuid().ToString("N"));
        private string Db => OpenVgdb.DefaultPath;
        private string Store => Path.Combine(_root, "Media");
        private string Roms => Path.Combine(_root, "Roms");

        public OnlineCoverTests()
        {
            Directory.CreateDirectory(Roms);
            EmuSen.Galaxia.Library.DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            EmuSen.Galaxia.Library.DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A stand-in for both servers: answers by URL, and records every request it was asked.
        public sealed class FakeServer : HttpMessageHandler
        {
            public readonly ConcurrentQueue<string> Asked = new();
            public Func<string, HttpResponseMessage> Answer = _ => new HttpResponseMessage(HttpStatusCode.NotFound);

            protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
            {
                Asked.Enqueue(request.RequestUri!.AbsoluteUri);
                return Task.FromResult(Answer(request.RequestUri!.AbsoluteUri));
            }

            public static HttpResponseMessage Png(int bytes = 400)
            {
                var content = new ByteArrayContent(Enumerable.Repeat((byte)0x89, bytes).ToArray());
                content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/png");
                return new HttpResponseMessage(HttpStatusCode.OK) { Content = content };
            }
        }

        // OpenVGDB's own four tables, with only the columns the lookup reads, and rows the tests name.
        public static void BuildOpenVgdb(string path, params (string System, string Name, string Md5, string? Cover)[] roms)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            using var db = new SqliteConnection($"Data Source={path};Pooling=False");
            db.Open();
            using SqliteCommand create = db.CreateCommand();
            create.CommandText = """
                CREATE TABLE SYSTEMS (systemID INTEGER PRIMARY KEY, systemName TEXT, systemShortName TEXT, systemHeaderSizeBytes INTEGER, systemOEID TEXT);
                CREATE TABLE ROMs (romID INTEGER PRIMARY KEY, systemID INTEGER, romHashMD5 TEXT, romExtensionlessFileName TEXT);
                CREATE TABLE RELEASES (releaseID INTEGER PRIMARY KEY, romID INTEGER, releaseTitleName TEXT, releaseCoverFront TEXT);
                CREATE TABLE REGIONS (regionID INTEGER PRIMARY KEY, regionName TEXT);
                INSERT INTO SYSTEMS VALUES (19, 'Nintendo Game Boy', 'GB', NULL, 'openemu.system.gb');
                INSERT INTO SYSTEMS VALUES (23, 'Nintendo 64', 'N64', NULL, 'openemu.system.n64');
                INSERT INTO SYSTEMS VALUES (25, 'Nintendo Entertainment System', 'NES', 16, 'openemu.system.nes');
                INSERT INTO SYSTEMS VALUES (26, 'Nintendo Super Nintendo Entertainment System', 'SNES', NULL, 'openemu.system.snes');
                """;
            create.ExecuteNonQuery();
            int id = 1;
            foreach (var (system, name, md5, cover) in roms)
            {
                using SqliteCommand insert = db.CreateCommand();
                insert.CommandText = "INSERT INTO ROMs VALUES ($id, (SELECT systemID FROM SYSTEMS WHERE systemShortName = $system), $md5, $name); INSERT INTO RELEASES VALUES ($id, $id, $name, $cover)";
                insert.Parameters.AddWithValue("$id", id++);
                insert.Parameters.AddWithValue("$system", system);
                insert.Parameters.AddWithValue("$md5", md5);
                insert.Parameters.AddWithValue("$name", name);
                insert.Parameters.AddWithValue("$cover", (object?)cover ?? DBNull.Value);
                insert.ExecuteNonQuery();
            }
        }

        private static string Md5(byte[] bytes) => Convert.ToHexString(MD5.HashData(bytes));

        private static ConsoleCore Core(string console) => RuntimeCatalog.Consoles.Single(c => c.Console == console);

        private string Rom(string name, byte[] bytes)
        {
            string path = Path.Combine(Roms, name);
            Directory.CreateDirectory(Roms);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        private static byte[] Filled(int length, int seed)
        {
            var bytes = new byte[length];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        // Each game's cover lookup as the failover recorded it in games.db, by a run with ScreenScraper off; a failure is not recorded.
        private Dictionary<string, (string Outcome, string? RomName)> Collect(FakeServer server, params string[] roms)
        {
            string games = Path.Combine(_root, "games.db");
            using (var library = EmuSen.Crystal.Client.Library.GameLibrary.Open(games))
            using (var s = new EmuSen.WiseMan.Mistress.Scraping.LibraryScraper(server, Store, new Dictionary<string, object?> { ["roms"] = Roms, ["openemu"] = true }, library: library))
            {
                s.Run(roms.Select(r => (r, EmuSen.Mistress.Library.RuntimeCatalog.ShelfByName(EmuSen.Mistress.Library.RuntimeCatalog.ShelfFor(r))!.EsdeSystem)),
                    new EmuSen.Mistress.Scraping.ScrapeChoices(), EmuSen.Mistress.Scraping.ScrapePriority.Library, enabled: false);
                s.Ends();
            }
            var found = new Dictionary<string, (string, string?)>(StringComparer.Ordinal);
            using var c = new SqliteConnection($"Data Source={games};Mode=ReadOnly;Pooling=False");
            c.Open();
            using SqliteCommand q = c.CreateCommand();
            q.CommandText = "SELECT path, outcome, rom_name FROM cover_lookup";
            using SqliteDataReader row = q.ExecuteReader();
            while (row.Read()) found[row.GetString(0)] = (row.GetString(1), row.IsDBNull(2) ? null : row.GetString(2));
            return found;
        }

        // Where the failover keeps a console's covers: the store's own openemu folder.
        private string Kept(string console, string stem) => Path.Combine(Store, "openemu", console, stem + ".png");

        private IEnumerable<string> Others(FakeServer server) => server.Asked.Where(u => !u.Contains("screenscraper.fr"));

        // The systems table's rule as the library reads it, through the platform library, for the console's first system.
        private static byte[] Bytes(string console, byte[] file) => OpenVgdbBytes.Of(Core(console).Systems[0], file);

        [Fact]
        public void The_bytes_each_console_hashes_are_the_ones_openvgdb_hashed()
        {
            byte[] body = Filled(0x8000, 1);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            Assert.Equal(body, Bytes("NES", ines));
            Assert.Equal(body, Bytes("NES", body));

            byte[] copier = new byte[512].Concat(body).ToArray();
            Assert.Equal(body, Bytes("SNES", copier));
            Assert.Equal(body, Bytes("SNES", body));

            Assert.Equal(body, Bytes("GB", body));

            byte[] z64 = new byte[] { 0x80, 0x37, 0x12, 0x40 }.Concat(Filled(0x1000 - 4, 2)).ToArray();
            byte[] v64 = new byte[z64.Length];
            for (int i = 0; i < z64.Length; i += 2) { v64[i] = z64[i + 1]; v64[i + 1] = z64[i]; }
            byte[] n64 = new byte[z64.Length];
            for (int i = 0; i < z64.Length; i += 4) { n64[i] = z64[i + 3]; n64[i + 1] = z64[i + 2]; n64[i + 2] = z64[i + 1]; n64[i + 3] = z64[i]; }
            Assert.Equal(v64, Bytes("N64", z64));
            Assert.Equal(v64, Bytes("N64", v64));
            Assert.Equal(v64, Bytes("N64", n64));
        }

        [Fact]
        public void A_game_named_as_openvgdb_names_it_gets_libretro_s_box_in_its_console_s_folder()
        {
            BuildOpenVgdb(Db, ("SNES", "Super Mario World (USA)", "00", null));
            string rom = Rom("Super Mario World (USA).sfc", Filled(1024, 3));
            var server = new FakeServer { Answer = url => url.Contains("thumbnails.libretro.com") ? FakeServer.Png() : new HttpResponseMessage(HttpStatusCode.NotFound) };

            var result = Collect(server, rom)[rom];

            Assert.Equal("Found", result.Outcome);
            Assert.True(File.Exists(Kept("SNES", "Super Mario World (USA)")));
            Assert.Equal("https://thumbnails.libretro.com/Nintendo%20-%20Super%20Nintendo%20Entertainment%20System/Named_Boxarts/Super%20Mario%20World%20%28USA%29.png", Others(server).Single());
        }

        [Fact]
        public void A_renamed_file_is_found_by_the_hash_of_the_bytes_openvgdb_hashed_and_fetched_by_its_canonical_name()
        {
            byte[] body = Filled(0x4000, 4);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            BuildOpenVgdb(Db, ("NES", "Metroid (USA)", Md5(body), null));
            string rom = Rom("metroid-renamed.nes", ines);
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            var result = Collect(server, rom)[rom];

            Assert.Equal(("Found", "Metroid (USA)"), (result.Outcome, result.RomName));
            Assert.Contains("Named_Boxarts/Metroid%20%28USA%29.png", Others(server).Single());
            Assert.True(File.Exists(Kept("NES", "metroid-renamed")));
        }

        [Fact]
        public void When_libretro_has_no_box_openvgdb_s_own_address_is_tried_and_a_refusal_is_no_art()
        {
            BuildOpenVgdb(Db, ("GB", "Tetris (World) (Rev 1)", "00", "https://gamefaqs.example/box/1_front.jpg"));
            string rom = Rom("Tetris (World) (Rev 1).gb", Filled(0x8000, 5));
            var server = new FakeServer { Answer = url => new HttpResponseMessage(url.Contains("gamefaqs") ? HttpStatusCode.Forbidden : HttpStatusCode.NotFound) };

            var result = Collect(server, rom)[rom];

            Assert.Equal("NoArt", result.Outcome);
            Assert.Equal(5, Others(server).Count());
            Assert.Equal("https://gamefaqs.example/box/1_front.jpg", Others(server).Last());
            Assert.False(Directory.Exists(Path.Combine(Store, "openemu")) && Directory.EnumerateFiles(Path.Combine(Store, "openemu"), "*", SearchOption.AllDirectories).Any());
        }

        [Fact]
        public void A_page_that_is_not_an_image_is_not_kept()
        {
            BuildOpenVgdb(Db, ("SNES", "F-Zero (USA)", "00", null));
            string rom = Rom("F-Zero (USA).sfc", Filled(1024, 6));
            var server = new FakeServer { Answer = _ => new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("<html>" + new string((char)0x78, 2000) + "</html>") } };

            Assert.Equal("NoArt", Collect(server, rom)[rom].Outcome);
            Assert.False(File.Exists(Kept("SNES", "F-Zero (USA)")));
        }

        [Fact]
        public void A_game_openvgdb_does_not_know_asks_no_server_and_one_asked_twice_is_looked_up_once()
        {
            BuildOpenVgdb(Db, ("SNES", "F-Zero (USA)", "00", null));
            string hack = Rom("My Hack.sfc", Filled(1024, 7));
            string known = Rom("F-Zero (USA).sfc", Filled(1024, 8));
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            var results = Collect(server, hack, known, known);

            Assert.Equal("Unknown", results[hack].Outcome);
            Assert.Single(Others(server));
        }

        [Fact]
        public void A_cover_already_in_the_folder_is_never_replaced()
        {
            BuildOpenVgdb(Db, ("SNES", "F-Zero (USA)", "00", null));
            string rom = Rom("F-Zero (USA).sfc", Filled(1024, 9));
            string mine = Kept("SNES", "F-Zero (USA)");
            Directory.CreateDirectory(Path.GetDirectoryName(mine)!);
            File.WriteAllBytes(mine, new byte[] { 1, 2, 3 });
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            Collect(server, rom);

            Assert.Empty(Others(server));
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(mine));
        }

        // With no database the failover asks GitHub for OpenVGDB's release first, as the window did, and no cover server.
        [Fact]
        public void With_no_database_every_lookup_fails_without_asking_a_cover_server()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(1024, 10));
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            Assert.False(Collect(server, rom).ContainsKey(rom));
            Assert.DoesNotContain(Others(server), u => u.Contains("thumbnails.libretro.com"));
            Assert.All(Others(server), u => Assert.Contains("github", u));
        }

        // The download is the scraper's, asked through the binding Mistress's Download row calls.
        [Fact]
        public void The_database_is_taken_from_the_newest_release_s_zip_and_a_bad_one_leaves_nothing()
        {
            string built = Path.Combine(_root, "built", OpenVgdb.FileName);
            BuildOpenVgdb(built, ("SNES", "F-Zero (USA)", "00", null));
            byte[] zip;
            using (var memory = new MemoryStream())
            {
                using (var archive = new ZipArchive(memory, ZipArchiveMode.Create, leaveOpen: true))
                    archive.CreateEntryFromFile(built, OpenVgdb.FileName);
                zip = memory.ToArray();
            }
            const string latest = "https://api.github.com/repos/OpenVGDB/OpenVGDB/releases/latest";
            const string release = """{"tag_name":"v29.0","assets":[{"name":"openvgdb.zip","browser_download_url":"https://github.example/openvgdb.zip"}]}""";
            byte[] served = zip;
            var server = new FakeServer
            {
                Answer = url => url == latest
                    ? new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent(release) }
                    : new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(served) },
            };
            using (var s = new EmuSen.WiseMan.Mistress.Scraping.LibraryScraper(server, Store))
            {
                Assert.Equal(("v29.0", (string?)null), s.Scraper.DownloadOpenVgdb());
                Assert.True(File.Exists(Db));
                File.Delete(Db);
                served = new byte[] { 1, 2, 3 };
                (string? tag, string? failed) = s.Scraper.DownloadOpenVgdb();
                Assert.Null(tag);
                Assert.NotNull(failed);
            }
            Assert.False(File.Exists(Db));
            Assert.False(File.Exists(Db + ".part"));
        }
    }
}
