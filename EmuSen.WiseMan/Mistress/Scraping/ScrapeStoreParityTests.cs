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
    // The scraper's store held to the C#'s answers recorded before its deletion, with no network: media.db at every version, generated records read, the layout step, a game's Clear, the orphans and their cleanup; a game's folder and the credentials' places held live - see EmuSen_Backend.md §30.7 and §40.5.
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

        private static string[] Migrations => MediaDbSchema.Steps;

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
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeStoreParity.Versions");
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
                        Execute(c, "INSERT INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ('aa', 1, 'cover', 'snes/covers/A.png', 'us', NULL, '2026-01-01T00:00:00.000Z')");
                        Execute(c, "INSERT INTO scrape_file (path, bytes, modified, md5) VALUES ('/r/snes/A.sfc', 1, 5, 'aa')");
                        Execute(c, "INSERT INTO scrape_queue (path, system, priority, seq) VALUES ('/r/snes/A.sfc', 'snes', 2, 1)");
                        Execute(c, "INSERT INTO quota_day (day, requests) VALUES ('2026-01-01', 3)");
                    }
                    if (version >= 2) Execute(c, "INSERT INTO store_step VALUES ('folders', '2026-01-01T00:00:00.000Z')");
                    if (version >= 5) FingerprintRows(c);
                }
                string a = Path.Combine(_root, $"v{version}", "opened");
                CopyTree(seed, a);
                MediaService.Open(a, Settings()).Dispose();
                golden.Add($"version {version}", Dump(Path.Combine(a, MediaStore.FileName)));
            }

            string newer = Path.Combine(_root, "newer");
            Directory.CreateDirectory(newer);
            using (var c = new SqliteConnection($"Data Source={Path.Combine(newer, MediaStore.FileName)};Pooling=False"))
            {
                c.Open();
                Execute(c, $"PRAGMA user_version = {Migrations.Length + 1}");
            }
            byte[] before = File.ReadAllBytes(Path.Combine(newer, MediaStore.FileName));
            var refused = Assert.Throws<MediaServiceException>(() => MediaService.Open(newer, Settings()));
            golden.Add("newer", $"{refused.Status} {refused.Message}");
            Assert.Equal(before, File.ReadAllBytes(Path.Combine(newer, MediaStore.FileName)));
            golden.Check(Migrations.Length + 2);
        }

        // A big disc keyed by its fingerprint, as schema 5 keeps it: an empty MD5 in its rows and in the path index until its checksums are taken (EmuSen_LibraryTools.md §9.12).
        private static void FingerprintRows(SqliteConnection c)
        {
            Execute(c, "INSERT INTO scrape_game (md5, bytes, status, fetched_at, name, matched_by, fingerprint) VALUES ('', 4700000000, 'Found', '2026-01-03T00:00:00.000Z', 'Big', 'name+size', 'f1'), ('', 4700000000, 'Found', '2026-01-03T00:00:00.000Z', 'Other', 'name+size', 'f2')");
            Execute(c, "INSERT INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at, fingerprint) VALUES ('', 4700000000, 'cover', 'ps2/covers/Big.png', 'us', NULL, '2026-01-03T00:00:00.000Z', 'f1')");
            Execute(c, "INSERT INTO scrape_file (path, bytes, modified, md5, fingerprint) VALUES ('/r/ps2/Big.iso', 4700000000, 7, '', 'f1'), ('/r/ps2/Other.iso', 4700000000, 7, 'm2', 'f2')");
        }

        private static string? S(JsonElement e, string name) => e.GetProperty(name).ValueKind == JsonValueKind.Null ? null : e.GetProperty(name).ValueKind == JsonValueKind.Number ? e.GetProperty(name).GetRawText() : e.GetProperty(name).GetString();

        private static string Rec(JsonElement e) => string.Join("|", S(e, "md5"), S(e, "bytes"), S(e, "state"), S(e, "game_id"), S(e, "rom_id"), S(e, "system_id"), S(e, "name"), S(e, "description"), S(e, "developer"),
            S(e, "publisher"), S(e, "genre"), S(e, "players"), e.GetProperty("rating").ValueKind == JsonValueKind.Null ? "" : BitConverter.SingleToInt32Bits(e.GetProperty("rating").GetSingle()).ToString(CultureInfo.InvariantCulture),
            S(e, "release_date"), S(e, "region"), S(e, "language"), S(e, "matched_by"), S(e, "offered"), S(e, "detail"), S(e, "fetched_at"), S(e, "kinds_known"), S(e, "fingerprint"));

        [Fact]
        public void Generated_records_read_alike_through_both()
        {
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeStoreParity.Records");
            string a = Path.Combine(_root, "records"), db = Path.Combine(a, MediaStore.FileName);
            MediaService.Open(a, Settings()).Dispose();
            ScrapeGoldens.Seed(golden, "seed", db, _root);
            using MediaService rust = MediaService.Open(a, Settings());
            JsonDocument Get(object request) => JsonDocument.Parse(rust.Get(JsonSerializer.Serialize(request)));

            List<string> found;
            using (JsonDocument doc = Get(new { what = "found" })) found = doc.RootElement.EnumerateObject().Select(p => p.Name + " " + Rec(p.Value)).Order(StringComparer.Ordinal).ToList();
            golden.Add("found", string.Join("\n", found));
            Assert.True(found.Count > 10, $"{found.Count}");
            Assert.Contains(found, f => f.StartsWith("/r/ps2/Big.iso ", StringComparison.Ordinal) && f.Contains("|Big|"));
            Assert.Contains(found, f => f.StartsWith("/r/ps2/Other.iso ", StringComparison.Ordinal) && f.Contains("|Other|"));

            using (JsonDocument doc = Get(new { what = "outcomes" }))
                golden.Add("outcomes", string.Join("\n", doc.RootElement.EnumerateObject().Select(p => $"{p.Name} {p.Value.GetProperty("state").GetString()} {p.Value.GetProperty("has_cover").GetBoolean()}").Order(StringComparer.Ordinal)));

            foreach (string path in found.Select(f => f[..f.IndexOf(' ')]).Take(10).Append("/nothing"))
                using (JsonDocument doc = Get(new { what = "record", path })) golden.Add($"record {path}", doc.RootElement.ValueKind == JsonValueKind.Null ? "null" : Rec(doc.RootElement));

            using (var c = new SqliteConnection($"Data Source={db};Mode=ReadOnly;Pooling=False"))
            {
                c.Open();
                using var q = c.CreateCommand();
                q.CommandText = "SELECT md5, bytes FROM scrape_game WHERE fingerprint = '' ORDER BY md5, bytes";
                using var row = q.ExecuteReader();
                while (row.Read())
                {
                    string md5 = row.GetString(0);
                    long bytes = row.GetInt64(1);
                    using (JsonDocument game = Get(new { what = "game", md5, bytes })) golden.Add($"game {md5} {bytes}", Rec(game.RootElement));
                    using JsonDocument stored = Get(new { what = "stored", md5, bytes });
                    golden.Add($"stored {md5} {bytes}", string.Join(";", stored.RootElement.EnumerateArray().Select(m => $"{S(m, "type")},{S(m, "path")},{S(m, "region")},{S(m, "sha1")}")));
                }
            }

            using (JsonDocument doc = Get(new { what = "queue" })) golden.Add("queue", string.Join(";", doc.RootElement.EnumerateArray().Select(q => $"{S(q, "path")},{S(q, "system")},{S(q, "priority")},{q.GetProperty("attempts").GetInt32()}")));
            golden.Check(1 + 1 + 1 + 11 + 1 + 2 * 60);
        }

        // A store and a ROM folder with foldered and top-level games, their pictures stored flat, as before §30 of the Big Picture page; the store's rows as the C#'s calls seeded them, from the recording.
        private (string Roms, string Media, List<(string Rom, string System)> Games) Layout(string name, int seed, EmuSen.WiseMan.Crystal.ParityGolden golden)
        {
            var r = new Random(seed);
            string roms = Path.Combine(_root, name, "Roms"), media = Path.Combine(_root, name, "seed");
            var games = new List<(string, string)>();
            string[] folders = ["", "", "RPG", "RPG/Square", "Act ion", "Ünï"];
            for (int i = 0; i < 25; i++)
            {
                string system = r.Next(2) == 0 ? "snes" : "nes", folder = folders[r.Next(folders.Length)], stem = $"Game {r.Next(12)}";
                string rom = Path.Combine(roms, system, folder.Replace('/', Path.DirectorySeparatorChar), stem + (system == "snes" ? ".sfc" : ".nes"));
                Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
                if (!File.Exists(rom)) File.WriteAllBytes(rom, [(byte)i]);
                games.Add((rom, system));
                foreach (string kind in new[] { "covers", "screenshots", "marquees" }.Where(_ => r.Next(3) != 0))
                {
                    string relative = Path.Combine(system, kind, stem + ".png");
                    string file = Path.Combine(media, relative);
                    Directory.CreateDirectory(Path.GetDirectoryName(file)!);
                    if (!File.Exists(file)) File.WriteAllBytes(file, Encoding.UTF8.GetBytes($"{relative} {seed}"));
                }
            }
            foreach (string orphan in new[] { "snes/covers/Gone.png", "nes/marquees/Old/Gone.png", "snes/videos/Gone.mp4", "snes/covers/Half.png.part", "gb/covers/Elsewhere.png" })
            {
                string file = Path.Combine(media, orphan);
                Directory.CreateDirectory(Path.GetDirectoryName(file)!);
                File.WriteAllBytes(file, [7, 7]);
            }
            MediaService.Open(media, Settings()).Dispose();
            ScrapeGoldens.Seed(golden, $"seed {name}", Path.Combine(media, MediaStore.FileName), _root);
            return (roms, media, games);
        }

        [Fact]
        public void The_layout_step_a_clear_the_orphans_and_their_cleanup_move_the_same_files()
        {
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("ScrapeStoreParity.Layout");
            for (int seed = 1; seed <= 4; seed++)
            {
                var (roms, media, games) = Layout($"layout{seed}", seed, golden);
                string a = Path.Combine(_root, $"layout{seed}", "store"), db = Path.Combine(a, MediaStore.FileName);
                CopyTree(media, a);
                string Rows() => ScrapeGoldens.Plain(Dump(db), _root);

                using MediaService rust = MediaService.Open(a, Settings(roms));
                Assert.NotEqual(Tree(media), Tree(a));
                golden.Add($"seed {seed} laid out", Tree(a));
                golden.Add($"seed {seed} laid out rows", Rows());

                foreach (var (rom, _) in games) Assert.Equal(GameFolders.Of(roms, rom), JsonSerializer.Deserialize<string>(rust.Get(JsonSerializer.Serialize(new { what = "folder", path = rom }))));

                var (cleared, clearedSystem) = games[seed];
                IEnumerable<string> deleted = JsonSerializer.Deserialize<string[]>(rust.Change(JsonSerializer.Serialize(new { op = "forget", path = cleared, system = clearedSystem })))!;
                golden.Add($"seed {seed} forgotten", string.Join("\n", deleted.Select(f => Path.GetRelativePath(a, f)).Order(StringComparer.Ordinal)));
                golden.Add($"seed {seed} forgotten tree", Tree(a));
                golden.Add($"seed {seed} forgotten rows", Rows());

                var kept = games.Where((_, i) => i % 3 != 0).ToList();
                IReadOnlyList<string> orphans = JsonSerializer.Deserialize<string[]>(rust.Get(JsonSerializer.Serialize(new { what = "orphans", games = kept.Select(g => new { path = g.Rom, system = g.System }) })))!;
                golden.Add($"seed {seed} orphans", string.Join("\n", orphans));
                Assert.NotEmpty(orphans);

                var at = new DateTimeOffset(2026, 9, 27, 14, 30, 5, TimeSpan.FromHours(2));
                (int moved, long bytes, string folder) cleaned;
                {
                    using JsonDocument doc = JsonDocument.Parse(rust.Change(JsonSerializer.Serialize(new { op = "clean_up", relatives = orphans, at = at.ToString("o", CultureInfo.InvariantCulture) })));
                    cleaned = (doc.RootElement.GetProperty("moved").GetInt32(), doc.RootElement.GetProperty("bytes").GetInt64(), doc.RootElement.GetProperty("folder").GetString()!);
                }
                golden.Add($"seed {seed} cleaned", $"{cleaned.moved} {cleaned.bytes} {Path.GetRelativePath(a, cleaned.folder)}");
                string note = Path.Combine(cleaned.folder, "cleanup.txt");
                golden.Add($"seed {seed} cleaned tree", Tree(a).Replace(Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(note))), "note"));
                golden.Add($"seed {seed} cleanup note", File.ReadAllText(note).Replace(a, "<store>"));
                golden.Add($"seed {seed} cleaned rows", Rows());
            }
            golden.Check(4 * 11);
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
