using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Galaxia;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The scraper's store in Rust against its C# twin, with no network: media.db at every version, generated records read through both, the layout step, a game's Clear, the orphans and their cleanup, a game's folder and the credentials' places - see EmuSen_Backend.md §30.7.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class ScrapeStoreParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeStoreParity", Guid.NewGuid().ToString("N"));
        private static readonly DateTimeOffset Now = new(2026, 9, 27, 12, 0, 0, 123, TimeSpan.Zero);
        private static long Ticks(DateTimeOffset t) => (t.UtcTicks - DateTimeOffset.UnixEpoch.UtcTicks);
        private static readonly FieldInfo EmbeddedSource = typeof(DeveloperCredentials).GetField("EmbeddedSource", BindingFlags.NonPublic | BindingFlags.Static)!;
        private readonly object _realEmbedded = EmbeddedSource.GetValue(null)!;

        public ScrapeStoreParityTests()
        {
            Directory.CreateDirectory(_root);
            Assert.True(MediaService.Ready, MediaService.Report);
        }

        public void Dispose()
        {
            EmbeddedSource.SetValue(null, _realEmbedded);
            ConfigStore.OverrideDirectory = null;
            ConfigStore.OverrideLegacyDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static string Settings(string? roms = null, DateTimeOffset? clock = null) =>
            JsonSerializer.Serialize(new Dictionary<string, object?> { ["roms"] = roms, ["developer_files"] = false, ["test"] = new { clock = Ticks(clock ?? Now) } });

        private static string[] Migrations => (string[])typeof(MediaStore).GetField("Migrations", BindingFlags.NonPublic | BindingFlags.Static)!.GetValue(null)!;

        // Every table's creating text and every row with each value's type, in a stable order.
        private static string Dump(string db)
        {
            var sb = new StringBuilder();
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
            c.Open();
            using (var v = c.CreateCommand()) { v.CommandText = "PRAGMA user_version"; sb.AppendLine("version " + v.ExecuteScalar()); }
            var tables = new List<string>();
            using (var m = c.CreateCommand())
            {
                m.CommandText = "SELECT type, name, sql FROM sqlite_master ORDER BY name";
                using var r = m.ExecuteReader();
                while (r.Read())
                {
                    sb.AppendLine($"{r.GetString(0)} {r.GetString(1)} {(r.IsDBNull(2) ? "" : r.GetString(2))}");
                    if (r.GetString(0) == "table") tables.Add(r.GetString(1));
                }
            }
            foreach (string t in tables)
            {
                using var q = c.CreateCommand();
                q.CommandText = $"SELECT * FROM {t}";
                using var r = q.ExecuteReader();
                var rows = new List<string>();
                while (r.Read())
                {
                    var line = new StringBuilder(t);
                    for (int i = 0; i < r.FieldCount; i++)
                    {
                        object v = r.GetValue(i);
                        line.Append(" | ").Append(v is DBNull ? "null" : $"{v.GetType().Name}:{Convert.ToString(v, CultureInfo.InvariantCulture)}");
                        if (v is double d) line.Append(':').Append(BitConverter.DoubleToInt64Bits(d));
                    }
                    rows.Add(line.ToString());
                }
                foreach (string row in rows.Order(StringComparer.Ordinal)) sb.AppendLine(row);
            }
            return sb.ToString();
        }

        // Every file below a folder but media.db, by its relative path, with its SHA-256.
        private static string Tree(string root) => string.Join("\n", Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories)
            .Where(f => !Path.GetFileName(f).StartsWith(MediaStore.FileName, StringComparison.Ordinal))
            .Select(f => Path.GetRelativePath(root, f).Replace(Path.DirectorySeparatorChar, '/') + " " + Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(f))))
            .Order(StringComparer.Ordinal));

        private static void CopyTree(string from, string to)
        {
            foreach (string f in Directory.EnumerateFiles(from, "*", SearchOption.AllDirectories))
            {
                string target = Path.Combine(to, Path.GetRelativePath(from, f));
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                File.Copy(f, target);
            }
            Directory.CreateDirectory(to);
        }

        private static void Execute(SqliteConnection c, string sql)
        {
            using var command = c.CreateCommand();
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        [Fact]
        public void Every_media_db_version_opens_alike_and_a_newer_one_is_refused_alike()
        {
            for (int version = 0; version <= Migrations.Length; version++)
            {
                string seed = Path.Combine(_root, $"v{version}", "seed");
                Directory.CreateDirectory(seed);
                using (var c = new SqliteConnection($"Data Source={Path.Combine(seed, MediaStore.FileName)};Pooling=False"))
                {
                    c.Open();
                    for (int i = 0; i < version; i++) Execute(c, Migrations[i]);
                    Execute(c, $"PRAGMA user_version = {version}");
                    if (version >= 1)
                    {
                        Execute(c, "INSERT INTO scrape_game (md5, bytes, status, rating, fetched_at, name) VALUES ('aa', 1, 'Found', 0.8000000119, '2026-01-01T00:00:00.000Z', 'Café'), ('bb', 2, 'Unknown', NULL, '2026-01-02T00:00:00.000Z', NULL)");
                        Execute(c, "INSERT INTO scrape_media VALUES ('aa', 1, 'cover', 'snes/covers/A.png', 'us', NULL, '2026-01-01T00:00:00.000Z')");
                        Execute(c, "INSERT INTO scrape_file VALUES ('/r/snes/A.sfc', 1, 5, 'aa')");
                        Execute(c, "INSERT INTO scrape_queue (path, system, priority, seq) VALUES ('/r/snes/A.sfc', 'snes', 2, 1)");
                        Execute(c, "INSERT INTO quota_day (day, requests) VALUES ('2026-01-01', 3)");
                    }
                    if (version >= 2) Execute(c, "INSERT INTO store_step VALUES ('folders', '2026-01-01T00:00:00.000Z')");
                }
                string a = Path.Combine(_root, $"v{version}", "csharp"), b = Path.Combine(_root, $"v{version}", "rust");
                CopyTree(seed, a);
                CopyTree(seed, b);
                MediaStore.Open(a).Dispose();
                MediaService.Open(b, Settings()).Dispose();
                Assert.Equal(Dump(Path.Combine(a, MediaStore.FileName)), Dump(Path.Combine(b, MediaStore.FileName)));
            }

            string newer = Path.Combine(_root, "newer");
            Directory.CreateDirectory(newer);
            using (var c = new SqliteConnection($"Data Source={Path.Combine(newer, MediaStore.FileName)};Pooling=False"))
            {
                c.Open();
                Execute(c, $"PRAGMA user_version = {Migrations.Length + 1}");
            }
            byte[] before = File.ReadAllBytes(Path.Combine(newer, MediaStore.FileName));
            var csharp = Assert.Throws<InvalidDataException>(() => MediaStore.Open(newer));
            var rust = Assert.Throws<MediaServiceException>(() => MediaService.Open(newer, Settings()));
            Assert.Equal((MediaServiceException.NewerFile, csharp.Message), (rust.Status, rust.Message));
            Assert.Equal(before, File.ReadAllBytes(Path.Combine(newer, MediaStore.FileName)));
        }

        private static readonly string[] Texts = ["Game", "Café du Monde", "It's \"quoted\"", "", "  ", "Δέλτα", "line\nbreak", "日本語"];

        // A store seeded through the C#'s own calls with generated records, media, files, a queue and days.
        private static void Seed(MediaStore store, Random r)
        {
            string? T() => r.Next(4) == 0 ? null : Texts[r.Next(Texts.Length)];
            for (int i = 0; i < 60; i++)
            {
                string md5 = $"{i:x2}{r.Next():x8}";
                long bytes = r.Next(1, 1 << 20);
                var state = (ScrapeState)r.Next(3);
                store.Record(new ScrapedRecord(md5, bytes, state)
                {
                    GameId = r.Next(3) == 0 ? null : r.Next(), RomId = r.Next(3) == 0 ? null : (long)r.Next() * 7, SystemId = r.Next(3) == 0 ? null : r.Next(200),
                    Name = T(), Description = T(), Developer = T(), Publisher = T(), Genre = T(), Players = T(),
                    Rating = r.Next(3) == 0 ? null : ScrapeRules.Rating(r.NextDouble() * 20),
                    ReleaseDate = r.Next(3) == 0 ? null : new DateTime(1980 + r.Next(40), 1 + r.Next(12), 1 + r.Next(28)),
                    Region = T(), Language = T(), MatchedBy = new[] { "file", "name", "transformed", null }[r.Next(4)], Offered = r.Next(2) == 0 ? "cover,screenshot" : null,
                    Detail = T(), FetchedAt = Now.UtcDateTime.AddSeconds(r.Next(100000)).AddTicks(r.Next(10_000_000)), KindsKnown = r.Next(2) == 0 ? null : "cover,screenshot,marquee",
                });
                for (int p = 0; p < r.Next(3); p++) store.RememberFile($"/roms/snes/{(p == 0 ? "" : "Sub/")}Game {i} {p}.sfc", bytes, r.Next(), md5);
                foreach (string type in new[] { "cover", "screenshot", "marquee", "video" }.Where(_ => r.Next(2) == 0))
                    store.RecordMedia(md5, bytes, new StoredMedia(type, $"snes/{type}s/Game {i}.png", r.Next(2) == 0 ? null : "us", r.Next(2) == 0 ? null : "ab12"), Now.AddMinutes(i));
            }
            for (int i = 0; i < 20; i++) store.Enqueue($"/roms/nes/Q{r.Next(15)}.nes", "nes", (ScrapePriority)r.Next(3));
            store.SaveDay(new QuotaDay("2026-09-27", 41, 3, 20000, 2000, Now.AddHours(5), "today's requests are used up (430)"));
        }

        private static string Rec(ScrapedRecord r) => string.Join("|", r.Md5, r.Bytes, r.State, r.GameId, r.RomId, r.SystemId, r.Name, r.Description, r.Developer, r.Publisher, r.Genre, r.Players,
            r.Rating is float f ? BitConverter.SingleToInt32Bits(f).ToString(CultureInfo.InvariantCulture) : "", r.ReleaseDate?.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture), r.Region, r.Language, r.MatchedBy, r.Offered, r.Detail,
            r.FetchedAt.ToString("yyyy-MM-ddTHH:mm:ss.fffZ", CultureInfo.InvariantCulture), r.KindsKnown);

        private static string? S(JsonElement e, string name) => e.GetProperty(name).ValueKind == JsonValueKind.Null ? null : e.GetProperty(name).ValueKind == JsonValueKind.Number ? e.GetProperty(name).GetRawText() : e.GetProperty(name).GetString();

        private static string Rec(JsonElement e) => string.Join("|", S(e, "md5"), S(e, "bytes"), S(e, "state"), S(e, "game_id"), S(e, "rom_id"), S(e, "system_id"), S(e, "name"), S(e, "description"), S(e, "developer"),
            S(e, "publisher"), S(e, "genre"), S(e, "players"), e.GetProperty("rating").ValueKind == JsonValueKind.Null ? "" : BitConverter.SingleToInt32Bits(e.GetProperty("rating").GetSingle()).ToString(CultureInfo.InvariantCulture),
            S(e, "release_date"), S(e, "region"), S(e, "language"), S(e, "matched_by"), S(e, "offered"), S(e, "detail"), S(e, "fetched_at"), S(e, "kinds_known"));

        [Fact]
        public void Generated_records_read_alike_through_both()
        {
            string a = Path.Combine(_root, "records");
            using (MediaStore seed = MediaStore.Open(a)) Seed(seed, new Random(2026));
            using MediaStore csharp = MediaStore.Open(a);
            using MediaService rust = MediaService.Open(a, Settings());

            var found = csharp.FoundByPath().Select(kv => kv.Key + " " + Rec(kv.Value)).Order(StringComparer.Ordinal).ToList();
            using (JsonDocument doc = JsonDocument.Parse(rust.Get("{\"what\":\"found\"}")))
                Assert.Equal(found, doc.RootElement.EnumerateObject().Select(p => p.Name + " " + Rec(p.Value)).Order(StringComparer.Ordinal).ToList());
            Assert.True(found.Count > 10, $"{found.Count}");

            var outcomes = csharp.OutcomesByPath().Select(kv => $"{kv.Key} {kv.Value.State} {kv.Value.HasCover}").Order(StringComparer.Ordinal).ToList();
            using (JsonDocument doc = JsonDocument.Parse(rust.Get("{\"what\":\"outcomes\"}")))
                Assert.Equal(outcomes, doc.RootElement.EnumerateObject().Select(p => $"{p.Name} {p.Value.GetProperty("state").GetString()} {p.Value.GetProperty("has_cover").GetBoolean()}").Order(StringComparer.Ordinal).ToList());

            foreach (string path in csharp.FoundByPath().Keys.Take(10).Append("/nothing"))
            {
                string expected = csharp.FoundFor(path) is { } r ? Rec(r) : "null";
                using JsonDocument doc = JsonDocument.Parse(rust.Get(JsonSerializer.Serialize(new { what = "record", path })));
                Assert.Equal(expected, doc.RootElement.ValueKind == JsonValueKind.Null ? "null" : Rec(doc.RootElement));
            }

            using (var c = new SqliteConnection($"Data Source={Path.Combine(a, MediaStore.FileName)};Mode=ReadOnly;Pooling=False"))
            {
                c.Open();
                using var q = c.CreateCommand();
                q.CommandText = "SELECT md5, bytes FROM scrape_game";
                using var row = q.ExecuteReader();
                while (row.Read())
                {
                    string md5 = row.GetString(0);
                    long bytes = row.GetInt64(1);
                    using JsonDocument game = JsonDocument.Parse(rust.Get(JsonSerializer.Serialize(new { what = "game", md5, bytes })));
                    Assert.Equal(Rec(csharp.Game(md5, bytes)!), Rec(game.RootElement));
                    string media = string.Join(";", csharp.Media(md5, bytes).Select(m => $"{m.Type},{m.RelativePath},{m.Region},{m.Sha1}"));
                    using JsonDocument stored = JsonDocument.Parse(rust.Get(JsonSerializer.Serialize(new { what = "stored", md5, bytes })));
                    Assert.Equal(media, string.Join(";", stored.RootElement.EnumerateArray().Select(m => $"{S(m, "type")},{S(m, "path")},{S(m, "region")},{S(m, "sha1")}")));
                }
            }

            string queue = string.Join(";", csharp.Queue().Select(q => $"{q.Path},{q.System},{q.Priority.ToString().ToLowerInvariant()},{q.Attempts}"));
            using (JsonDocument doc = JsonDocument.Parse(rust.Get("{\"what\":\"queue\"}")))
                Assert.Equal(queue, string.Join(";", doc.RootElement.EnumerateArray().Select(q => $"{S(q, "path")},{S(q, "system")},{S(q, "priority")},{q.GetProperty("attempts").GetInt32()}")));
        }

        // A store and a ROM folder with foldered and top-level games, their pictures stored flat, as before §30 of the Big Picture page.
        private (string Roms, string Media, List<(string Rom, string System)> Games) Layout(string name, int seed)
        {
            var r = new Random(seed);
            string roms = Path.Combine(_root, name, "Roms"), media = Path.Combine(_root, name, "seed");
            var games = new List<(string, string)>();
            using MediaStore store = MediaStore.Open(media);
            string[] folders = ["", "", "RPG", "RPG/Square", "Act ion", "Ünï"];
            for (int i = 0; i < 25; i++)
            {
                string system = r.Next(2) == 0 ? "snes" : "nes", folder = folders[r.Next(folders.Length)], stem = $"Game {r.Next(12)}";
                string rom = Path.Combine(roms, system, folder.Replace('/', Path.DirectorySeparatorChar), stem + (system == "snes" ? ".sfc" : ".nes"));
                Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
                if (!File.Exists(rom)) File.WriteAllBytes(rom, [(byte)i]);
                games.Add((rom, system));
                string md5 = $"m{i % 18:x2}";
                store.RememberFile(rom, 64, 1, md5);
                store.Record(new ScrapedRecord(md5, 64, ScrapeState.Found) { FetchedAt = Now.UtcDateTime });
                foreach (string kind in new[] { "covers", "screenshots", "marquees" }.Where(_ => r.Next(3) != 0))
                {
                    string relative = Path.Combine(system, kind, stem + ".png");
                    string file = Path.Combine(media, relative);
                    Directory.CreateDirectory(Path.GetDirectoryName(file)!);
                    if (!File.Exists(file)) File.WriteAllBytes(file, Encoding.UTF8.GetBytes($"{relative} {seed}"));
                    store.RecordMedia(md5, 64, new StoredMedia(kind.TrimEnd('s'), relative, "us", null), Now);
                }
            }
            foreach (string orphan in new[] { "snes/covers/Gone.png", "nes/marquees/Old/Gone.png", "snes/videos/Gone.mp4", "snes/covers/Half.png.part", "gb/covers/Elsewhere.png" })
            {
                string file = Path.Combine(media, orphan);
                Directory.CreateDirectory(Path.GetDirectoryName(file)!);
                File.WriteAllBytes(file, [7, 7]);
            }
            return (roms, media, games);
        }

        [Fact]
        public void The_layout_step_a_clear_the_orphans_and_their_cleanup_move_the_same_files()
        {
            for (int seed = 1; seed <= 4; seed++)
            {
                var (roms, media, games) = Layout($"layout{seed}", seed);
                string a = Path.Combine(_root, $"layout{seed}", "csharp"), b = Path.Combine(_root, $"layout{seed}", "rust");
                CopyTree(media, a);
                CopyTree(media, b);

                using MediaStore csharp = MediaStore.Open(a);
                int carried = csharp.PutInFolders(p => GameFolders.Of(roms, p), Now);
                using MediaService rust = MediaService.Open(b, Settings(roms));
                Assert.True(carried > 0, $"seed {seed}");
                Assert.Equal(Tree(a), Tree(b));
                Assert.Equal(Dump(Path.Combine(a, MediaStore.FileName)), Dump(Path.Combine(b, MediaStore.FileName)));

                foreach (var (rom, _) in games) Assert.Equal(GameFolders.Of(roms, rom), JsonSerializer.Deserialize<string>(rust.Get(JsonSerializer.Serialize(new { what = "folder", path = rom }))));

                var (cleared, clearedSystem) = games[seed];
                var deletedA = csharp.Forget(cleared, clearedSystem, GameFolders.Of(roms, cleared)).Select(f => Path.GetRelativePath(a, f)).Order(StringComparer.Ordinal);
                var deletedB = JsonSerializer.Deserialize<string[]>(rust.Change(JsonSerializer.Serialize(new { op = "forget", path = cleared, system = clearedSystem })))!.Select(f => Path.GetRelativePath(b, f)).Order(StringComparer.Ordinal);
                Assert.Equal(deletedA, deletedB);
                Assert.Equal(Tree(a), Tree(b));
                Assert.Equal(Dump(Path.Combine(a, MediaStore.FileName)), Dump(Path.Combine(b, MediaStore.FileName)));

                var kept = games.Where((_, i) => i % 3 != 0).ToList();
                var names = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
                foreach (var (rom, system) in kept)
                {
                    if (!names.TryGetValue(system, out var set)) names[system] = set = new HashSet<string>(StringComparer.Ordinal);
                    string stem = Path.GetFileNameWithoutExtension(rom), folder = GameFolders.Of(roms, rom);
                    set.Add(folder.Length == 0 ? stem : folder + "/" + stem);
                    for (string f = folder; f.Length > 0; f = f.Contains('/') ? f[..f.LastIndexOf('/')] : "") set.Add(f);
                }
                IReadOnlyList<string> orphans = csharp.Orphans(names.ToDictionary(n => n.Key, n => (IReadOnlySet<string>)n.Value, StringComparer.Ordinal));
                string[] rustOrphans = JsonSerializer.Deserialize<string[]>(rust.Get(JsonSerializer.Serialize(new { what = "orphans", games = kept.Select(g => new { path = g.Rom, system = g.System }) })))!;
                Assert.Equal(orphans, rustOrphans);
                Assert.NotEmpty(orphans);

                var at = new DateTimeOffset(2026, 9, 27, 14, 30, 5, TimeSpan.FromHours(2));
                var (movedA, bytesA, folderA) = csharp.CleanUp(orphans, at);
                using JsonDocument cleaned = JsonDocument.Parse(rust.Change(JsonSerializer.Serialize(new { op = "clean_up", relatives = rustOrphans, at = at.ToString("o", CultureInfo.InvariantCulture) })));
                Assert.Equal((movedA, bytesA, Path.GetRelativePath(a, folderA)), (cleaned.RootElement.GetProperty("moved").GetInt32(), cleaned.RootElement.GetProperty("bytes").GetInt64(), Path.GetRelativePath(b, cleaned.RootElement.GetProperty("folder").GetString()!)));
                Assert.Equal(Tree(a).Replace(Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(Path.Combine(folderA, "cleanup.txt")))), "note"),
                             Tree(b).Replace(Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(Path.Combine(cleaned.RootElement.GetProperty("folder").GetString()!, "cleanup.txt")))), "note"));
                Assert.Equal(File.ReadAllText(Path.Combine(folderA, "cleanup.txt")).Replace(a, "<store>"), File.ReadAllText(Path.Combine(cleaned.RootElement.GetProperty("folder").GetString()!, "cleanup.txt")).Replace(b, "<store>"));
                Assert.Equal(Dump(Path.Combine(a, MediaStore.FileName)), Dump(Path.Combine(b, MediaStore.FileName)));
            }
        }

        private static string DeveloperJson(string id, string password) => JsonSerializer.Serialize(new { devid = id, devpassword = password, softname = "EmuSen-Test" });

        [Fact]
        public void The_developer_credentials_are_found_in_the_same_place_and_the_member_alike()
        {
            string config = Path.Combine(_root, "config"), legacy = Path.Combine(_root, "legacy"), store = Path.Combine(_root, "creds");
            ConfigStore.OverrideDirectory = config;
            ConfigStore.OverrideLegacyDirectory = legacy;
            var cases = new List<(string What, Action Arrange, byte[]? Blob)>
            {
                ("nothing", () => { }, null),
                ("embedded", () => { }, ScrapeCredentialTests.Scramble(DeveloperJson("FAKEEMBEDID", "FAKEEMBEDPW"))),
                ("embedded, damaged", () => { }, [.. "ESSD"u8, 2, 1, 7, 7]),
                ("legacy", () => Write(legacy, DeveloperJson("FAKELEGACY", "FAKELPW")), null),
                ("tree", () => Write(config, DeveloperJson("FAKECONFIG", "FAKECPW")), ScrapeCredentialTests.Scramble(DeveloperJson("FAKEE2", "FAKEE2PW"))),
                ("broken tree", () => File.WriteAllText(Path.Combine(config, DeveloperCredentials.FileName), "{\"devid\":"), null),
                ("half tree", () => File.WriteAllText(Path.Combine(config, DeveloperCredentials.FileName), "{\"devid\":\"FAKEHALF\",\"softname\":\"x\"}"), null),
            };
            foreach (var (what, arrange, blob) in cases)
            {
                arrange();
                EmbeddedSource.SetValue(null, (Func<Stream?>)(() => blob is null ? null : new MemoryStream(blob)));
                string? expected = DeveloperCredentials.Load()?.Origin switch { DeveloperOrigin.TreeFile => "tree", DeveloperOrigin.UserFile => "user", DeveloperOrigin.Embedded => "embedded", _ => null };
                string settings = JsonSerializer.Serialize(new { developer_files = true, test = new { clock = Ticks(Now) } });
                using MediaService rust = MediaService.Open(store, settings, embedded: blob);
                using JsonDocument doc = JsonDocument.Parse(rust.Get("{\"what\":\"developer\"}"));
                Assert.True(expected == (doc.RootElement.GetProperty("origin").ValueKind == JsonValueKind.Null ? null : doc.RootElement.GetProperty("origin").GetString()), what);
            }

            foreach (var member in new (string User, string Password)[] { ("", ""), ("FAKEMEMBER", "FAKEMPW&x"), ("FAKEONLYNAME", "") })
            {
                File.Delete(MemberAccount.PathOf);
                if (member.User.Length > 0) new MemberAccount(member.User, member.Password) { Verified = Now.UtcDateTime }.Save();
                MemberAccount loaded = MemberAccount.Load();
                using MediaService rust = MediaService.Open(store, Settings());
                using JsonDocument doc = JsonDocument.Parse(rust.Get("{\"what\":\"account\"}"));
                Assert.Equal(loaded.IsSet, doc.RootElement.GetProperty("signed_in").GetBoolean());
                Assert.Equal(loaded.IsSet ? loaded.User : null, doc.RootElement.GetProperty("name").ValueKind == JsonValueKind.Null ? null : doc.RootElement.GetProperty("name").GetString());
                bool had = File.Exists(MemberAccount.PathOf);
                using JsonDocument signedOut = JsonDocument.Parse(rust.Change("{\"op\":\"sign_out\"}"));
                Assert.Equal(had, signedOut.RootElement.GetProperty("had").GetBoolean());
                Assert.False(File.Exists(MemberAccount.PathOf));
            }
        }

        private static void Write(string dir, string json)
        {
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, DeveloperCredentials.FileName), json);
        }
    }
}
