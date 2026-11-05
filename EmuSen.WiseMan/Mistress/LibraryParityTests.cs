using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text;
using EmuSen.Crystal.Client.Library;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress
{
    // The library held to the C# answers recorded before its deletion: generated libraries scanned, games.db identified and reclaimed and its rows, every schema version opened - see EmuSen_Backend.md §28.6 and §40.4.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class LibraryParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenLibraryParity", Guid.NewGuid().ToString("N"));
        private readonly List<string> _locked = new();

        public LibraryParityTests()
        {
            Directory.CreateDirectory(_root);
            RuntimeCatalog.Forget();
            Assert.True(GameLibrary.Ready, GameLibrary.Report);
        }

        public void Dispose()
        {
            foreach (string dir in _locked) try { File.SetUnixFileMode(dir, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute); } catch { }
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static RomLibraryResult Scan(string? root, string? shelf = null, RomLibrary.Beside? beside = null) => RomLibrary.Scan(root, beside ?? RomLibrary.Beside.None, shelf);

        private static GameRecords Open(string db) => GameRecords.Open(db);

        private static void Put(string path, byte[] bytes)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, bytes);
        }

        private static byte[] GameBoy(byte flag, int length = 0x8000)
        {
            var image = new byte[length];
            if (length > 0x143) image[0x143] = flag;
            return image;
        }

        // Each entry stamped with one time, so an archive's bytes and hash are the same on every run.
        private static void Zip(string path, params (string Name, byte[] Bytes)[] entries)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            using var zip = ZipFile.Open(path, ZipArchiveMode.Create);
            foreach (var (name, bytes) in entries)
            {
                ZipArchiveEntry entry = zip.CreateEntry(name);
                entry.LastWriteTime = new DateTimeOffset(2026, 2, 1, 12, 0, 0, TimeSpan.Zero);
                using Stream s = entry.Open();
                s.Write(bytes);
            }
        }

        // One generated library: every system's extensions, sets, archives, routing by content and by folder, hidden, unreadable and linked entries, names that differ only by case.
        private string Generate(int seed)
        {
            var random = new Random(seed);
            string root = Path.Combine(_root, $"roms{seed}");
            byte[] Bytes(int n) { var b = new byte[n]; random.NextBytes(b); return b; }
            string Pick(params string[] folders) => folders[random.Next(folders.Length)];
            string[] names = { "Alpha", "beta", "Gamma (USA)", "Pokémon Blue", "It's \"Quoted\"", "Zeta [!]", "zeta [!]", "Omega (Rev 1)", "Δέλτα", "a.b.c" };
            string Name() => names[random.Next(names.Length)] + (random.Next(4) == 0 ? $" {random.Next(100)}" : "");

            for (int i = 0; i < 4 + random.Next(4); i++) Put(Path.Combine(root, Pick("snes", "SNES", "Misc"), Name() + Pick(".sfc", ".smc", ".SFC")), SyntheticRom.BuildBlank());
            for (int i = 0; i < 3 + random.Next(4); i++) Put(Path.Combine(root, Pick("nes", "NES/USA", "Deep/Er/Still"), Name() + ".nes"), SyntheticNesRom.Build());
            foreach (byte flag in new byte[] { 0x00, 0x80, 0xC0, 0x40 }) Put(Path.Combine(root, Pick("gb", "gbc"), $"{Name()} {flag:x2}{Pick(".gb", ".gbc", ".GB")}"), GameBoy(flag, random.Next(3) == 0 ? 0x100 : 0x8000));
            Put(Path.Combine(root, "gb", "Short.gb"), Array.Empty<byte>());
            for (int i = 0; i < 2; i++) Put(Path.Combine(root, "n64", Name() + Pick(".z64", ".n64", ".v64")), Bytes(4096));
            for (int i = 0; i < 2 + random.Next(3); i++) Put(Path.Combine(root, Pick("megadrive", "genesis", "Misc"), Name() + Pick(".md", ".gen", ".bin", ".smd")), SyntheticMdRom.Cartridge());
            Put(Path.Combine(root, Pick("segacd", "Misc"), Name() + ".iso"), SyntheticMdRom.Disc());
            Put(Path.Combine(root, "segacd", "Raw Disc.bin"), SyntheticMdRom.Disc());
            Put(Path.Combine(root, "32x", Name() + ".32x"), SyntheticMdRom.Cartridge("SEGA 32X"));
            Put(Path.Combine(root, Pick("genesis", "sega32x", "Misc"), "Mars " + Name() + ".bin"), SyntheticMdRom.Cartridge("SEGA 32X"));

            PsxDisc disc = SyntheticPsxDisc.Build(audioSeconds: 0);
            string psx = Path.Combine(root, Pick("psx", "PlayStation", "Misc"));
            Directory.CreateDirectory(psx);
            disc.WriteSet(psx, "Disc One");
            disc.WriteSet(psx, "Disc Two");
            File.WriteAllText(Path.Combine(psx, "Both.m3u"), "#EXTM3U\nDisc One.cue\nDISC TWO.cue\n");
            Put(Path.Combine(psx, "Lone.iso"), disc.Iso);
            File.WriteAllText(Path.Combine(psx, "Missing Track.cue"), "FILE \"Gone (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
            File.WriteAllText(Path.Combine(psx, "Broken.cue"), "TRACK 01 MODE2/2352\n");
            Put(Path.Combine(root, "Misc", "Unknown.bin"), Bytes(5000));
            Put(Path.Combine(root, "psx", "Garbage.chd"), Bytes(300));

            Zip(Path.Combine(root, "nes", "Zipped.zip"), ("Zipped.nes", SyntheticNesRom.Build()));
            Zip(Path.Combine(root, "Misc", "Disc In Zip.zip"), ("sub/Game.cue", Encoding.UTF8.GetBytes(disc.Cue.Replace("{name}", "Game"))), ("sub/Game (Track 1).bin", disc.DataTrack.AsSpan(0, Math.Min(disc.DataTrack.Length, 40 * 2352)).ToArray()));
            Zip(Path.Combine(root, "Misc", "Nothing.zip"), ("readme.txt", Bytes(10)));
            Zip(Path.Combine(root, "gb", "Color In Zip.zip"), ("Game.gbc", GameBoy(0xC0)));
            Put(Path.Combine(root, "Misc", "Not A Zip.zip"), Bytes(100));

            Put(Path.Combine(root, "nes", ".Hidden.nes"), SyntheticNesRom.Build());
            Put(Path.Combine(root, ".hidden", "Inside.nes"), SyntheticNesRom.Build());
            Put(Path.Combine(root, "Misc", "notes.txt"), Bytes(10));
            Put(Path.Combine(root, "Misc", "Same Title.nes"), SyntheticNesRom.Build());
            Put(Path.Combine(root, "Misc", "Same Title.sfc"), SyntheticRom.BuildBlank());

            string linked = Path.Combine(_root, $"elsewhere{seed}");
            Put(Path.Combine(linked, "Linked Folder Game.nes"), SyntheticNesRom.Build());
            // Relative targets, so a link's own length is the same in every test folder.
            Directory.CreateSymbolicLink(Path.Combine(root, "Linked"), Path.Combine("..", $"elsewhere{seed}"));
            File.CreateSymbolicLink(Path.Combine(root, "nes", "Link To Alpha.nes"), Path.Combine("..", "Misc", "Same Title.nes"));
            File.CreateSymbolicLink(Path.Combine(root, "nes", "Dangling.nes"), Path.Combine("..", "nothing-here.nes"));

            string locked = Path.Combine(root, "Locked");
            Put(Path.Combine(locked, "Unseen.nes"), SyntheticNesRom.Build());
            File.SetUnixFileMode(locked, UnixFileMode.None);
            _locked.Add(locked);
            return root;
        }

        // Ties in the title order keep the order the file system listed them in, so a listing is shown with ties by path; the title order itself is checked live.
        private string Show(RomLibraryResult r)
        {
            Assert.True(r.Entries.Zip(r.Entries.Skip(1)).All(p => StringComparer.OrdinalIgnoreCase.Compare(p.First.Title, p.Second.Title) <= 0), "the listing is not in title order");
            return Plain($"{r.Status} | {r.Directory} | {r.CoreDisplayName}\n" + string.Join("\n", Ordered(r).Select(e => $"{e.FullPath} | {e.Shelf} | {e.CoreDisplayName}")));
        }

        private static IEnumerable<RomEntry> Ordered(RomLibraryResult r) =>
            r.Entries.OrderBy(e => e.Title, StringComparer.OrdinalIgnoreCase).ThenBy(e => e.FullPath, StringComparer.Ordinal);

        // The test's folder written as <root>, so a recording holds no machine's path.
        private string Plain(string text) => text.Replace(_root, "<root>", StringComparison.Ordinal);

        // A console's listing is the whole listing narrowed, held live; the whole listing is held to the recording.
        private void Narrowed(string root, RomLibrary.Beside beside, IEnumerable<string> shelves)
        {
            RomLibraryResult all = Scan(root, null, beside);
            foreach (string shelf in shelves) Assert.Equal(ShowPackaged(RomLibrary.Narrow(all, shelf)), ShowPackaged(Scan(root, shelf, beside)));
        }

        [Fact]
        public void Every_generated_library_is_listed_and_shelved_alike()
        {
            var golden = new ParityGolden("LibraryParity.Listings");
            int compared = 0;
            for (int seed = 1; seed <= 6; seed++)
            {
                string root = Generate(seed);
                RomLibraryResult all = Scan(root);
                golden.Add($"seed {seed}", Show(all));
                Narrowed(root, RomLibrary.Beside.None, RuntimeCatalog.FilterChoices.Append("Dreamcast (Kunzite)"));
                compared += all.Entries.Count;
            }
            Assert.True(compared > 150, $"{compared} games compared");
            string?[] odd = { null, "", "   ", Path.Combine(_root, "absent"), _locked[0] };
            for (int i = 0; i < odd.Length; i++) golden.Add($"odd root {i}", Show(Scan(odd[i])));
            golden.Check(6 + odd.Length);
        }

        private string ShowPackaged(RomLibraryResult r) => Show(r) + "\n--\n" + Plain(string.Join("\n", Ordered(r).Select(e => e.Package ?? "-")));

        // Packages beside a generated library: linked ones, a converted disc's claiming its sheet, one whose game is gone, a refused manifest, a nested folder and loose files - see EmuSen_GamePackages.md §3.6.
        [Fact]
        public void A_library_with_packages_beside_it_is_listed_alike()
        {
            var golden = new ParityGolden("LibraryParity.Packaged");
            for (int seed = 1; seed <= 3; seed++)
            {
                string root = Generate(seed);
                string packages = Path.Combine(_root, $"packages{seed}"), compressed = Path.Combine(_root, $"compressed{seed}");
                var deep = new EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true, AttributesToSkip = FileAttributes.Hidden };
                string[] games = Directory.EnumerateFiles(root, "*.nes", deep).Concat(Directory.EnumerateFiles(root, "*.sfc", deep))
                    .Where(f => { string r = Path.GetRelativePath(root, f); return !r.Contains("Locked") && !r.Contains("Link"); }).Order(StringComparer.Ordinal).Take(4).ToArray();
                foreach (string game in games)
                {
                    string rel = Path.GetRelativePath(root, game).Replace('\\', '/');
                    string system = game.EndsWith(".nes", StringComparison.Ordinal) ? "nes" : "snes";
                    Put(Path.Combine(packages, system.ToUpperInvariant(), rel, "package.json"), Encoding.UTF8.GetBytes($"{{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"{system}\", \"mode\": \"linked\", \"game\": {{\"role\": \"image\", \"where\": \"library\", \"path\": \"{rel}\"}}}}"));
                    Put(Path.Combine(packages, system.ToUpperInvariant(), rel, "saves", Path.GetFileName(game)), SyntheticNesRom.Build());
                }
                string? cue = Directory.EnumerateFiles(root, "Disc One.cue", deep).FirstOrDefault();
                if (cue is not null)
                {
                    string rel = Path.GetRelativePath(root, cue).Replace('\\', '/');
                    Put(Path.Combine(compressed, "psx", "Disc One.chd"), new byte[64]);
                    Put(Path.Combine(packages, "PlayStation", "Disc One.chd", "package.json"), Encoding.UTF8.GetBytes("{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"psx\", \"mode\": \"linked\", " +
                        $"\"game\": {{\"role\": \"image\", \"where\": \"compressed\", \"path\": \"psx/Disc One.chd\"}}, \"container\": {{\"type\": \"chd\", \"sources\": [{{\"where\": \"library\", \"path\": \"{rel}\"}}]}}}}"));
                }
                Put(Path.Combine(compressed, "psx", "Stray.chd"), new byte[64]);
                Put(Path.Combine(packages, "NES", "Gone.nes", "package.json"), Encoding.UTF8.GetBytes("{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"nes\", \"mode\": \"linked\", \"game\": {\"role\": \"image\", \"where\": \"library\", \"path\": \"nes/Gone.nes\"}}"));
                Put(Path.Combine(packages, "NES", "Broken.nes", "package.json"), Encoding.UTF8.GetBytes("{ not json"));
                Put(Path.Combine(packages, "NES", "Broken.nes", "Inside.nes"), SyntheticNesRom.Build());
                Put(Path.Combine(packages, "Loose.nes"), SyntheticNesRom.Build());
                var beside = new RomLibrary.Beside(packages, compressed);
                RomLibraryResult scanned = Scan(root, null, beside);
                golden.Add($"seed {seed}", ShowPackaged(scanned));
                Narrowed(root, beside, RuntimeCatalog.FilterChoices);
                foreach (string game in games) Assert.Single(scanned.Entries, e => e.FullPath == game && e.Package is not null);
                Assert.DoesNotContain(scanned.Entries, e => e.FullPath.Contains("Inside.nes") || e.FullPath.Contains("Gone.nes") || e.FullPath.EndsWith(Path.Combine("saves", Path.GetFileName(games[0])), StringComparison.Ordinal));
                Assert.Contains(scanned.Entries, e => e.FullPath.EndsWith("Loose.nes", StringComparison.Ordinal) && e.Package is null);
                Assert.Contains(scanned.Entries, e => e.FullPath.EndsWith("Stray.chd", StringComparison.Ordinal));
                if (cue is not null)
                {
                    Assert.DoesNotContain(scanned.Entries, e => e.FullPath == cue);
                    Assert.Single(scanned.Entries, e => e.FullPath.EndsWith("Disc One.chd", StringComparison.Ordinal) && e.Shelf.StartsWith("PlayStation", StringComparison.Ordinal));
                }
                golden.Add($"seed {seed} without packages", Show(Scan(root, null, RomLibrary.Beside.None)));
            }
            golden.Check(3 * 2);
        }

        // Every row of every table, each value with its SQLite type, and the schema and version.
        private static string Dump(string db)
        {
            var text = new StringBuilder();
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
            c.Open();
            List<string> Rows(string sql)
            {
                var rows = new List<string>();
                using SqliteCommand command = c.CreateCommand();
                command.CommandText = sql;
                using SqliteDataReader r = command.ExecuteReader();
                while (r.Read())
                    rows.Add(string.Join(" | ", Enumerable.Range(0, r.FieldCount).Select(i => r.IsDBNull(i) ? "null" : r.GetFieldType(i) == typeof(double) ? "real:" + r.GetDouble(i).ToString("R", CultureInfo.InvariantCulture) : $"{r.GetDataTypeName(i)}:{r.GetValue(i)}")));
                return rows;
            }
            text.AppendLine("version " + Rows("PRAGMA user_version")[0]);
            List<string> schema = Rows("SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY name");
            schema.ForEach(s => text.AppendLine(s));
            using (SqliteCommand tables = c.CreateCommand())
            {
                tables.CommandText = "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name";
                var names = new List<string>();
                using (SqliteDataReader r = tables.ExecuteReader()) while (r.Read()) names.Add(r.GetString(0));
                foreach (string table in names)
                {
                    text.AppendLine("table " + table);
                    Rows($"SELECT * FROM \"{table}\" ORDER BY rowid").ForEach(s => text.AppendLine(s));
                }
            }
            return text.ToString();
        }

        private static string Copy(string from, string to)
        {
            File.Copy(from, to, overwrite: true);
            return to;
        }

        private static readonly DateTime Then = new(2026, 3, 1, 9, 30, 0, DateTimeKind.Utc);

        // One modification time for every file under the test's folder, so the hash cache's rows are the same on every run.
        private static readonly DateTime Settled = new(2026, 2, 1, 12, 0, 0, DateTimeKind.Utc);

        private void Settle()
        {
            var all = new EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true, AttributesToSkip = 0 };
            long seconds = new DateTimeOffset(Settled).ToUnixTimeSeconds();
            foreach (string f in Directory.EnumerateFiles(_root, "*", all))
                if (new FileInfo(f).LinkTarget is null) File.SetLastWriteTimeUtc(f, Settled);
                else Assert.Equal(0, lutimes(f, new[] { seconds, 0, seconds, 0 }));
        }

        // A link's own times, which the hash cache keeps for a link as FileInfo reads them.
        [System.Runtime.InteropServices.DllImport("libc", SetLastError = true)]
        private static extern int lutimes(string path, long[] times);

        // A games.db the player kept while the library looked one way, for a library that has since been renamed, moved, pruned and copied.
        private string Seeded(int seed, string root, IReadOnlyList<string> games)
        {
            var random = new Random(seed * 7919);
            string db = Path.Combine(_root, $"seed{seed}.db");
            using (GameRecords records = Open(db))
            {
                long rpgs = records.CreateCollection("RPGs", Then)!.Value;
                long other = records.CreateCollection("other", Then.AddDays(1))!.Value;
                foreach (string game in games)
                {
                    if (random.Next(3) == 0) continue;
                    if (GameRecords.Stat(game) is { } stat && GameRecords.TryMd5(game) is { } md5)
                    {
                        records.Identify(game, md5, stat.Bytes);
                        if (random.Next(2) == 0) records.StoreHash(game, stat.Bytes, stat.Modified, md5);
                    }
                    if (random.Next(2) == 0) records.Started(game, Then.AddHours(random.Next(5000)));
                    if (random.Next(2) == 0) records.Played(game, TimeSpan.FromSeconds(random.NextDouble() * 9000));
                    if (random.Next(3) == 0) records.ToggleFavourite(game);
                    if (random.Next(3) == 0) records.AddToCollection(random.Next(2) == 0 ? rpgs : other, game);
                    if (random.Next(4) == 0) records.SaveEdits(game, new Dictionary<string, string?> { ["name"] = "Edited " + random.Next(99), [GameMetadata.CoverFrom] = games[random.Next(games.Count)] }, Then);
                    if (random.Next(4) == 0) records.SetCrop(game, new GameCrop(random.Next(10), 1.5, random.NextDouble(), 0));
                    if (random.Next(4) == 0) records.RecordCoverLookup(game, (CoverOutcome)random.Next(3), random.Next(2) == 0 ? null : "Rom", Then);
                }
            }
            // Renamed, moved into another folder, deleted, and copied: the orphans a reclaim finds again or not.
            var shuffled = games.Where(File.Exists).Where(g => new FileInfo(g).LinkTarget is null).OrderBy(_ => random.Next()).ToList();
            for (int i = 0; i < Math.Min(shuffled.Count, 12); i++)
            {
                string game = shuffled[i];
                switch (i % 4)
                {
                    case 0: File.Move(game, Path.Combine(Path.GetDirectoryName(game)!, "Renamed " + Path.GetFileName(game))); break;
                    case 1:
                        Directory.CreateDirectory(Path.Combine(root, "Moved"));
                        File.Move(game, Path.Combine(root, "Moved", $"{i} {Path.GetFileName(game)}"));
                        break;
                    case 2: File.Delete(game); break;
                    case 3: File.Copy(game, Path.Combine(Path.GetDirectoryName(game)!, "Copy of " + Path.GetFileName(game))); break;
                }
            }
            return db;
        }

        // P5: every row of games.db after a scan's identity and reclaim, in every column, time columns included, since every time recorded is given and every file's time is settled.
        [Fact]
        public void A_seeded_games_db_is_identified_and_reclaimed_to_the_same_rows()
        {
            var golden = new ParityGolden("LibraryParity.Reclaimed");
            int moved = 0, identified = 0;
            for (int seed = 1; seed <= 5; seed++)
            {
                string root = Generate(seed + 100);
                Settle();
                IReadOnlyList<string> before = Scan(root).Entries.Select(e => e.FullPath).Order(StringComparer.Ordinal).ToList();
                string seeded = Seeded(seed, root, before);
                Settle();
                IReadOnlyList<string> after = Scan(root).Entries.Select(e => e.FullPath).Order(StringComparer.Ordinal).ToList();
                string db = Copy(seeded, Path.Combine(_root, $"a{seed}.db"));
                using (GameRecords records = Open(db))
                {
                    int m = records.Reclaim(after);
                    golden.Add($"seed {seed} moved", m.ToString(CultureInfo.InvariantCulture));
                    moved += m;
                    var identities = new List<string>();
                    foreach (string game in after)
                    {
                        var x = records.IdentifyFile(game);
                        identities.Add($"{game} {x?.Md5 ?? "null"} {x?.Bytes}");
                        if (x is not null) identified++;
                    }
                    golden.Add($"seed {seed} identified", Plain(string.Join("\n", identities)));
                    golden.Add($"seed {seed} recent 7", Plain(string.Join("\n", records.Recent(after, 7))));
                    golden.Add($"seed {seed} recent 1000", Plain(string.Join("\n", records.Recent(after, 1000))));
                }
                golden.Add($"seed {seed} rows", Plain(Dump(db)));
            }
            Assert.True(moved >= 10, $"{moved} moved");
            Assert.True(identified > 100, $"{identified} identified");
            golden.Check(5 * 5);
        }

        // A file at each version the migrations have passed through, with rows in every table its schema has.
        private string AtVersion(int version)
        {
            string db = Path.Combine(_root, $"v{version}.db");
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Pooling = false }.ToString());
            c.Open();
            void Run(string sql)
            {
                using SqliteCommand command = c.CreateCommand();
                command.CommandText = sql;
                command.ExecuteNonQuery();
            }
            for (int v = 0; v < Math.Min(version, GamesDbSchema.Version); v++) Run(GamesDbSchema.Steps[v]);
            if (version >= 1) Run("INSERT INTO game (path, favourite, last_played, play_count, play_seconds) VALUES ('/r/old.z64', 1, '2026-01-02T03:04:05.0000000+01:00', 3, 42.5), ('/r/ü.sfc', 0, NULL, 0, 0)");
            if (version >= 2) Run("UPDATE game SET md5 = 'abc', bytes = 1024 WHERE path = '/r/old.z64'; INSERT INTO file_hash VALUES ('/r/old.z64', 1024, 638000000000000000, 'abc')");
            if (version >= 3) Run("INSERT INTO collection (name, created) VALUES ('RPGs', '2026-01-01T00:00:00.0000000+00:00'); INSERT INTO collection_game VALUES (1, '/r/old.z64')");
            if (version >= 4) Run("INSERT INTO cover_lookup VALUES ('/r/old.z64', 'NoArt', NULL, '2026-01-01T00:00:00.0000000+00:00')");
            if (version >= 5) Run("INSERT INTO game_edit VALUES ('/r/old.z64', 'name', 'Old', '2026-01-01T00:00:00.0000000+00:00')");
            if (version >= 6) Run("UPDATE game SET crop_left = 1.25, crop_right = 0, crop_top = 2.5, crop_bottom = 0.1 WHERE path = '/r/old.z64'");
            Run($"PRAGMA user_version = {version}");
            return db;
        }

        [Fact]
        public void Every_schema_version_is_opened_by_both_to_the_same_file()
        {
            var golden = new ParityGolden("LibraryParity.Versions");
            for (int version = 0; version <= GamesDbSchema.Version; version++)
            {
                string db = AtVersion(version);
                Open(db).Dispose();
                string opened = Dump(db);
                golden.Add($"version {version}", opened);
                Assert.Contains($"version INTEGER:{GamesDbSchema.Version}", opened);
            }
            string newer = AtVersion(GamesDbSchema.Version + 1);
            byte[] bytes = File.ReadAllBytes(newer);
            golden.Add("newer", Assert.Throws<InvalidDataException>(() => Open(newer)).Message);
            Assert.Equal(bytes, File.ReadAllBytes(newer));
            golden.Check(GamesDbSchema.Version + 2);
        }
    }
}
