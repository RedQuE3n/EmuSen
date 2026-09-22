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
using EmuSen.Cores;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.Mistress.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress
{
    // OpenVGDB names a game, libretro's server has its box; a fake server stands in for both - see EmuSen_Settings_Reference.md §4.39.
    public class OnlineCoverTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenOnlineCoverTests", Guid.NewGuid().ToString("N"));
        private string Db => Path.Combine(_root, "openvgdb.sqlite");
        private string Art => Path.Combine(_root, "Artwork");
        private string Roms => Path.Combine(_root, "Roms");

        public OnlineCoverTests() => Directory.CreateDirectory(Roms);

        public void Dispose()
        {
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

        private static CoreDescriptor Core(string console) => CoreCatalog.Cores.Single(c => c.Console == console);

        private string Rom(string name, byte[] bytes)
        {
            string path = Path.Combine(Roms, name);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        private static byte[] Filled(int length, int seed)
        {
            var bytes = new byte[length];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private static List<CoverResult> Collect(FakeServer server, string db, string art, int count, Action<CoverFetcher> enqueue)
        {
            var results = new BlockingCollection<CoverResult>();
            using var fetcher = new CoverFetcher(new HttpClient(server), db, art, results.Add) { Spacing = TimeSpan.Zero };
            enqueue(fetcher);
            var got = new List<CoverResult>();
            for (int i = 0; i < count; i++)
            {
                Assert.True(results.TryTake(out CoverResult? r, TimeSpan.FromSeconds(10)), "no result");
                got.Add(r!);
            }
            Thread.Sleep(50);
            Assert.Empty(results);
            return got;
        }

        [Fact]
        public void The_bytes_each_console_hashes_are_the_ones_openvgdb_hashed()
        {
            byte[] body = Filled(0x8000, 1);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            Assert.Equal(body, Core("NES").OpenVgdbBytes!(ines));
            Assert.Equal(body, Core("NES").OpenVgdbBytes!(body));

            byte[] copier = new byte[512].Concat(body).ToArray();
            Assert.Equal(body, Core("SNES").OpenVgdbBytes!(copier));
            Assert.Equal(body, Core("SNES").OpenVgdbBytes!(body));

            Assert.Equal(body, Core("GB").OpenVgdbBytes!(body));

            byte[] z64 = new byte[] { 0x80, 0x37, 0x12, 0x40 }.Concat(Filled(0x1000 - 4, 2)).ToArray();
            byte[] v64 = new byte[z64.Length];
            for (int i = 0; i < z64.Length; i += 2) { v64[i] = z64[i + 1]; v64[i + 1] = z64[i]; }
            byte[] n64 = new byte[z64.Length];
            for (int i = 0; i < z64.Length; i += 4) { n64[i] = z64[i + 3]; n64[i + 1] = z64[i + 2]; n64[i + 2] = z64[i + 1]; n64[i + 3] = z64[i]; }
            Assert.Equal(v64, Core("N64").OpenVgdbBytes!(z64));
            Assert.Equal(v64, Core("N64").OpenVgdbBytes!(v64));
            Assert.Equal(v64, Core("N64").OpenVgdbBytes!(n64));
        }

        [Theory]
        [InlineData("Wave Race 64 (USA) (Rev A)", "Wave Race 64 (USA) (Rev A)|Wave Race 64 (USA)")]
        [InlineData("Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De) (Rev A)", "Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De) (Rev A)|Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De)|Legend of Zelda, The - Ocarina of Time (Europe)")]
        [InlineData("Tetris (World)", "Tetris (World)")]
        [InlineData("Untagged", "Untagged")]
        public void A_revision_s_box_is_also_looked_for_under_the_plainer_names(string romName, string names)
        {
            Assert.Equal(names.Split('|'), CoverFetcher.Names(romName).ToArray());
        }

        [Fact]
        public void A_game_named_as_openvgdb_names_it_gets_libretro_s_box_in_its_console_s_folder()
        {
            BuildOpenVgdb(Db, ("SNES", "Super Mario World (USA)", "00", null));
            string rom = Rom("Super Mario World (USA).sfc", Filled(1024, 3));
            var server = new FakeServer { Answer = url => url.Contains("thumbnails.libretro.com") ? FakeServer.Png() : new HttpResponseMessage(HttpStatusCode.NotFound) };

            CoverResult result = Collect(server, Db, Art, 1, f => f.Enqueue(new RomEntry(rom), Core("SNES"))).Single();

            Assert.Equal(CoverOutcome.Found, result.Outcome);
            Assert.Equal(Path.Combine(Art, "SNES", "Super Mario World (USA).png"), result.Saved);
            Assert.True(File.Exists(result.Saved));
            Assert.Equal("https://thumbnails.libretro.com/Nintendo%20-%20Super%20Nintendo%20Entertainment%20System/Named_Boxarts/Super%20Mario%20World%20%28USA%29.png", server.Asked.Single());
        }

        [Fact]
        public void A_renamed_file_is_found_by_the_hash_of_the_bytes_openvgdb_hashed_and_fetched_by_its_canonical_name()
        {
            byte[] body = Filled(0x4000, 4);
            byte[] ines = new byte[] { (byte)'N', (byte)'E', (byte)'S', 0x1A }.Concat(new byte[12]).Concat(body).ToArray();
            BuildOpenVgdb(Db, ("NES", "Metroid (USA)", Md5(body), null));
            string rom = Rom("metroid-renamed.nes", ines);
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            CoverResult result = Collect(server, Db, Art, 1, f => f.Enqueue(new RomEntry(rom), Core("NES"))).Single();

            Assert.Equal(CoverOutcome.Found, result.Outcome);
            Assert.Equal("Metroid (USA)", result.RomName);
            Assert.Contains("Named_Boxarts/Metroid%20%28USA%29.png", server.Asked.Single());
            Assert.Equal(Path.Combine(Art, "NES", "metroid-renamed.png"), result.Saved);
        }

        [Fact]
        public void When_libretro_has_no_box_openvgdb_s_own_address_is_tried_and_a_refusal_is_no_art()
        {
            BuildOpenVgdb(Db, ("GB", "Tetris (World) (Rev 1)", "00", "https://gamefaqs.example/box/1_front.jpg"));
            string rom = Rom("Tetris (World) (Rev 1).gb", Filled(0x8000, 5));
            var server = new FakeServer { Answer = url => new HttpResponseMessage(url.Contains("gamefaqs") ? HttpStatusCode.Forbidden : HttpStatusCode.NotFound) };

            CoverResult result = Collect(server, Db, Art, 1, f => f.Enqueue(new RomEntry(rom), Core("GB"))).Single();

            Assert.Equal(CoverOutcome.NoArt, result.Outcome);
            Assert.Equal(5, server.Asked.Count);
            Assert.Equal("https://gamefaqs.example/box/1_front.jpg", server.Asked.Last());
            Assert.False(Directory.Exists(Art) && Directory.EnumerateFiles(Art, "*", SearchOption.AllDirectories).Any());
        }

        [Fact]
        public void A_page_that_is_not_an_image_is_not_kept()
        {
            BuildOpenVgdb(Db, ("SNES", "F-Zero (USA)", "00", null));
            string rom = Rom("F-Zero (USA).sfc", Filled(1024, 6));
            var server = new FakeServer { Answer = _ => new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("<html>" + new string((char)0x78, 2000) + "</html>") } };

            CoverResult result = Collect(server, Db, Art, 1, f => f.Enqueue(new RomEntry(rom), Core("SNES"))).Single();

            Assert.Equal(CoverOutcome.NoArt, result.Outcome);
            Assert.False(File.Exists(Path.Combine(Art, "SNES", "F-Zero (USA).png")));
        }

        [Fact]
        public void A_game_openvgdb_does_not_know_asks_no_server_and_one_asked_twice_is_looked_up_once()
        {
            BuildOpenVgdb(Db, ("SNES", "F-Zero (USA)", "00", null));
            string hack = Rom("My Hack.sfc", Filled(1024, 7));
            string known = Rom("F-Zero (USA).sfc", Filled(1024, 8));
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            List<CoverResult> results = Collect(server, Db, Art, 2, f =>
            {
                Assert.True(f.Enqueue(new RomEntry(hack), Core("SNES")));
                Assert.True(f.Enqueue(new RomEntry(known), Core("SNES")));
                Assert.False(f.Enqueue(new RomEntry(known), Core("SNES")));
            });

            Assert.Equal(CoverOutcome.Unknown, results.Single(r => r.RomPath == hack).Outcome);
            Assert.Single(server.Asked);
        }

        [Fact]
        public void A_cover_already_in_the_folder_is_never_replaced()
        {
            BuildOpenVgdb(Db, ("SNES", "F-Zero (USA)", "00", null));
            string rom = Rom("F-Zero (USA).sfc", Filled(1024, 9));
            string mine = Path.Combine(Art, "SNES", "F-Zero (USA).png");
            Directory.CreateDirectory(Path.GetDirectoryName(mine)!);
            File.WriteAllBytes(mine, new byte[] { 1, 2, 3 });
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            CoverResult result = Collect(server, Db, Art, 1, f => f.Enqueue(new RomEntry(rom), Core("SNES"))).Single();

            Assert.Empty(server.Asked);
            Assert.Equal(new byte[] { 1, 2, 3 }, File.ReadAllBytes(mine));
            Assert.Equal(CoverOutcome.Found, result.Outcome);
        }

        [Fact]
        public void With_no_database_every_lookup_fails_without_asking_a_server()
        {
            string rom = Rom("F-Zero (USA).sfc", Filled(1024, 10));
            var server = new FakeServer { Answer = _ => FakeServer.Png() };

            CoverResult result = Collect(server, Db, Art, 1, f => f.Enqueue(new RomEntry(rom), Core("SNES"))).Single();

            Assert.Equal(CoverOutcome.Failed, result.Outcome);
            Assert.Empty(server.Asked);
        }

        [Fact]
        public async Task The_database_is_taken_from_the_newest_release_s_zip_and_a_bad_one_leaves_nothing()
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
            const string release = """{"tag_name":"v29.0","assets":[{"name":"openvgdb.zip","browser_download_url":"https://github.example/openvgdb.zip"}]}""";
            var server = new FakeServer
            {
                Answer = url => url == OpenVgdbDownload.LatestRelease
                    ? new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent(release) }
                    : new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(zip) },
            };

            Assert.Equal("v29.0", await OpenVgdbDownload.FetchAsync(new HttpClient(server), Db));
            using (OpenVgdb? opened = OpenVgdb.Open(Db)) Assert.NotNull(opened!.ByName("F-Zero (USA)", new[] { "SNES" }));

            string other = Path.Combine(_root, "other.sqlite");
            server.Answer = url => url == OpenVgdbDownload.LatestRelease
                ? new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent(release) }
                : new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(new byte[] { 1, 2, 3 }) };
            await Assert.ThrowsAnyAsync<Exception>(() => OpenVgdbDownload.FetchAsync(new HttpClient(server), other));
            Assert.False(File.Exists(other));
            Assert.False(File.Exists(other + ".part"));
        }
    }
}
