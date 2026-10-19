using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Client.Library;
using EmuSen.Mistress.Library;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress
{
    // The player's records in Rust against their C# twin: scripts of every GameRecords member run on two copies of one games.db, each answer compared and the files compared row for row - see EmuSen_Backend.md §28.6.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class LibraryRecordsParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenLibraryRecordsParity", Guid.NewGuid().ToString("N"));

        public LibraryRecordsParityTests()
        {
            Directory.CreateDirectory(_root);
            Assert.True(GameLibrary.Ready, GameLibrary.Report);
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static readonly string[] Paths =
        {
            "/r/a.z64", "/r/A.z64", "/r/b.sfc", "/r/Pokémon Blue.gb", "/r/It's \"Quoted\".nes", "/r/Δέλτα.md", "/r/sub/c.cue", "/r/  spaced  .nes",
            "/r/emoji 🎮.gbc", "/r/back\\slash.nes", "", "/r/tab\there.nes",
        };

        private static readonly string[] Names = { "RPGs", "rpgs", "Racers", "Ünïcode", "Space ", " Space", "Favourites", "a'b", "" };

        private static readonly double[] Doubles = { 0, -0.0, 1.5, 12.3456789, 1e308, -1e308, 5e-324, 99.95, 33.333333333333336, double.NaN, double.PositiveInfinity, -7 };

        private static readonly string[] Fields = { "name", "sortname", "description", "rating", GameMetadata.CoverFrom, GameMetadata.TelevisionStandard, "hidden", "altemulator" };

        // What a call answered, or what it threw, as text both halves can be compared by.
        private static string Outcome(Func<object?> call)
        {
            try { return Show(call()); }
            catch (Exception e) when (e is SqliteException or InvalidDataException or InvalidOperationException or GameLibraryException) { return $"threw {e.GetType().Name}: {e.Message}"; }
        }

        private static string Show(object? value) => value switch
        {
            null => "null",
            GameRecord r => $"{r.Favourite} {r.LastPlayed?.ToString("o", CultureInfo.InvariantCulture)} {r.PlayCount} {r.PlaySeconds.ToString("R", CultureInfo.InvariantCulture)}",
            GameCrop c => string.Join(",", new[] { c.Left, c.Right, c.Top, c.Bottom }.Select(d => d.ToString("R", CultureInfo.InvariantCulture))),
            IReadOnlyDictionary<string, GameRecord> all => string.Join("; ", all.OrderBy(p => p.Key, StringComparer.Ordinal).Select(p => p.Key + "=" + Show(p.Value))),
            IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> edits => string.Join("; ", edits.OrderBy(p => p.Key, StringComparer.Ordinal).Select(p => p.Key + "=" + Show(p.Value))),
            IReadOnlyDictionary<string, string> edit => string.Join(",", edit.Select(p => $"{p.Key}:{p.Value}")),
            IReadOnlyList<GameCollection> cs => string.Join(";", cs.Select(c => $"{c.Id} {c.Name} {c.Count}")),
            IReadOnlySet<string> set => string.Join(",", set.Order(StringComparer.Ordinal)),
            IReadOnlySet<long> ids => string.Join(",", ids.Order()),
            IReadOnlyList<(string Path, string Md5, long Bytes)> orphans => string.Join(";", orphans.Select(o => $"{o.Path} {o.Md5} {o.Bytes}")),
            DateTime d => d.ToString("o", CultureInfo.InvariantCulture),
            _ => Convert.ToString(value, CultureInfo.InvariantCulture) ?? "",
        };

        private static string Dump(string db)
        {
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
            c.Open();
            var lines = new List<string>();
            foreach (string table in new[] { "game", "file_hash", "collection", "collection_game", "cover_lookup", "game_edit" })
            {
                using SqliteCommand command = c.CreateCommand();
                command.CommandText = $"SELECT * FROM {table} ORDER BY rowid";
                using SqliteDataReader r = command.ExecuteReader();
                lines.Add("table " + table);
                while (r.Read())
                    lines.Add(string.Join(" | ", Enumerable.Range(0, r.FieldCount).Select(i => r.IsDBNull(i) ? "null" : r.GetFieldType(i) == typeof(double) ? "real:" + r.GetDouble(i).ToString("R", CultureInfo.InvariantCulture) : $"{r.GetDataTypeName(i)}:{r.GetValue(i)}")));
            }
            return string.Join("\n", lines);
        }

        // One step of a script, chosen by the random source, run on a records object.
        private static Func<GameRecords, object?> Step(Random random, int step)
        {
            string P() => Paths[random.Next(Paths.Length)];
            double D() => Doubles[random.Next(Doubles.Length)];
            long Id() => random.Next(-1, 8);
            DateTime When() => new DateTime(2026, 1, 1, 0, 0, 0, DateTimeKind.Local).AddMinutes(random.Next(600_000)).AddTicks(random.Next(10_000_000));
            string path = P(), other = P(), name = Names[random.Next(Names.Length)];
            double d = D();
            long id = Id(), bytes = random.Next(0, 3) * 1024, modified = 638000000000000000 + random.Next(3);
            DateTime when = When();
            string md5 = random.Next(3) switch { 0 => "abc", 1 => "def", _ => "0123456789abcdef0123456789abcdef" };
            switch (random.Next(34))
            {
                case 0: return r => r.Find(path);
                case 1: return r => r.All();
                case 2: return r => r.IsFavourite(path);
                case 3: return r => r.LastPlayed(path);
                case 4: return r => { r.ToggleFavourite(path); return null; };
                case 5: bool fav = random.Next(2) == 0; return r => { r.SetFavourite(path, fav); return null; };
                case 6: return r => { r.Started(path, when); return null; };
                case 7: var span = TimeSpan.FromSeconds(random.Next(-100, 100_000) / 7.0); return r => { r.Played(path, span); return null; };
                case 8: int count = random.Next(-5, 1000); return r => { r.SetPlayStats(path, count, d); return null; };
                case 9: return r => r.KnownHash(path, bytes, modified);
                case 10: return r => { r.StoreHash(path, bytes, modified, md5); return null; };
                case 11: return r => r.PathByHash(md5);
                case 12: return r => { r.Identify(path, md5, bytes); return null; };
                case 13: var present = Paths.Where(_ => random.Next(2) == 0).ToHashSet(StringComparer.Ordinal); return r => r.Orphans(present);
                case 14: return r => { r.Move(path, other); return null; };
                case 15: return r => r.CoverLookup(path);
                case 16: var outcome = (CoverOutcome)random.Next(4); string? rom = random.Next(2) == 0 ? null : name; return r => { r.RecordCoverLookup(path, outcome, rom, when); return null; };
                case 17: return r => { r.ForgetCoverLookup(path); return null; };
                case 18: return r => r.Collections();
                case 19: return r => r.CreateCollection(name, when);
                case 20: return r => r.RenameCollection(id, name);
                case 21: return r => { r.DeleteCollection(id); return null; };
                case 22: return r => { r.AddToCollection(id, path); return null; };
                case 23: return r => { r.RemoveFromCollection(id, path); return null; };
                case 24: return r => r.Members(id);
                case 25: return r => r.CollectionsOf(path);
                case 26: return r => r.Edits(path);
                case 27: return r => r.AllEdits();
                case 28:
                    var changes = new Dictionary<string, string?>();
                    for (int i = random.Next(4); i > 0; i--) changes[Fields[random.Next(Fields.Length)]] = random.Next(3) == 0 ? null : random.Next(2) == 0 ? other : name;
                    return r => { r.SaveEdits(path, changes, when); return null; };
                case 29: return r => { r.ClearEdits(path); return null; };
                case 30: string? from = random.Next(3) == 0 ? null : other; return r => { r.SetCoverChoice(path, from, when); return r.CoverChoice(path); };
                case 31: string? standard = random.Next(3) == 0 ? null : random.Next(2) == 0 ? "pal" : ""; return r => { r.SetStandardChoice(path, standard, when); return r.StandardChoice(path); };
                case 32: var crop = new GameCrop(D(), D(), D(), D()); return r => { r.SetCrop(path, crop); return r.Crop(path); };
                default: return r => { if (step % 2 == 0) r.ClearCrop(path); return r.Crop(path); };
            }
        }

        [Fact]
        public void Every_member_answers_and_writes_alike_through_scripts()
        {
            int calls = 0, differences = 0;
            for (int seed = 1; seed <= 8; seed++)
            {
                var random = new Random(seed);
                string a = Path.Combine(_root, $"a{seed}.db"), b = Path.Combine(_root, $"b{seed}.db");
                using (GameRecords managed = GameRecords.OpenManaged(a))
                using (GameRecords native = GameRecords.OpenNative(b))
                {
                    for (int step = 0; step < 500; step++)
                    {
                        Func<GameRecords, object?> call = Step(random, step);
                        string x = Outcome(() => call(managed)), y = Outcome(() => call(native));
                        Assert.True(x == y, $"seed {seed} step {step}: C# {x}\nRust {y}");
                        calls++;
                    }
                }
                Assert.Equal(Dump(a), Dump(b));
            }
            Assert.Equal(0, differences);
            Assert.Equal(4000, calls);
        }

        [Fact]
        public void Play_time_is_tracked_under_the_same_limit()
        {
            string db = Path.Combine(_root, "limit.db");
            using GameLibrary lib = GameLibrary.Open(db);
            foreach (int hours in new[] { -1, 0, 1, 2, 23, 24, 25 })
                foreach (double seconds in new[] { 0, 3599.9, 3600, 3600.0001, 7200, 86400, 1e9 })
                {
                    TimeSpan? managed = PlayTime.Tracked(TimeSpan.FromSeconds(seconds), hours);
                    string native = lib.Change($"{{\"op\":\"played\",\"path\":\"/r/x\",\"seconds\":{seconds.ToString("R", CultureInfo.InvariantCulture)},\"max_hours\":{hours}}}");
                    Assert.Equal(managed is null ? "null" : managed.Value.TotalSeconds.ToString("R", CultureInfo.InvariantCulture), native == "null" ? "null" : double.Parse(native, CultureInfo.InvariantCulture).ToString("R", CultureInfo.InvariantCulture));
                }
        }
    }
}
