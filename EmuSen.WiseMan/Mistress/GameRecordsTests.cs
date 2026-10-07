using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress
{
    // The player's library database: what it keeps, and that it refuses a schema it does not know - see EmuSen_Settings_Reference.md §4.32.
    public class GameRecordsTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenGameRecordsTests", Guid.NewGuid().ToString("N"));
        private string Db => Path.Combine(_root, "games.db");

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        [Fact]
        public void A_record_written_is_read_back_after_the_file_is_reopened()
        {
            var when = new DateTime(2026, 9, 21, 20, 15, 0, DateTimeKind.Local);
            using (GameRecords records = GameRecords.Open(Db))
            {
                records.Started("/r/a.z64", when.AddHours(-1));
                records.Started("/r/a.z64", when);
                records.Played("/r/a.z64", TimeSpan.FromSeconds(90));
                records.Played("/r/a.z64", TimeSpan.FromSeconds(30));
                records.ToggleFavourite("/r/b.sfc");
            }

            using GameRecords reopened = GameRecords.Open(Db);
            GameRecord a = reopened.Find("/r/a.z64")!;
            Assert.Equal(2, a.PlayCount);
            Assert.Equal(when, a.LastPlayed);
            Assert.Equal(120, a.PlaySeconds, 3);
            Assert.False(a.Favourite);
            Assert.True(reopened.IsFavourite("/r/b.sfc"));
            Assert.Equal(0, reopened.Find("/r/b.sfc")!.PlayCount);
            Assert.Null(reopened.Find("/r/c.nes"));
            Assert.Equal(2, reopened.All().Count);
        }

        [Fact]
        public void A_second_toggle_takes_the_favourite_back()
        {
            using GameRecords records = GameRecords.Open(Db);
            records.ToggleFavourite("/r/a.z64");
            records.ToggleFavourite("/r/a.z64");
            Assert.False(records.IsFavourite("/r/a.z64"));
        }

        [Fact]
        public void A_new_file_is_stamped_with_the_schema_this_build_writes()
        {
            using (GameRecords.Open(Db)) { }
            Assert.Equal(GameRecords.SchemaVersion, UserVersion());
        }

        [Fact]
        public void A_file_from_a_newer_build_is_refused_rather_than_written()
        {
            using (GameRecords.Open(Db)) { }
            SetUserVersion(GameRecords.SchemaVersion + 1);

            Assert.Throws<InvalidDataException>(() => GameRecords.Open(Db));
            Assert.Equal(GameRecords.SchemaVersion + 1, UserVersion());
        }

        // A games.db from the first build of §4.32 has only the game table and user_version 1; its rows must survive the upgrade.
        [Fact]
        public void A_first_version_file_is_migrated_with_its_rows_kept()
        {
            Directory.CreateDirectory(_root);
            using (var db = new SqliteConnection($"Data Source={Db};Pooling=False"))
            {
                db.Open();
                using SqliteCommand create = db.CreateCommand();
                create.CommandText = """
                    CREATE TABLE game (path TEXT PRIMARY KEY, favourite INTEGER NOT NULL DEFAULT 0 CHECK (favourite IN (0, 1)),
                        last_played TEXT, play_count INTEGER NOT NULL DEFAULT 0, play_seconds REAL NOT NULL DEFAULT 0);
                    CREATE INDEX game_by_last_played ON game(last_played);
                    INSERT INTO game (path, favourite, play_count, play_seconds) VALUES ('/r/old.z64', 1, 3, 42.5);
                    PRAGMA user_version = 1;
                    """;
                create.ExecuteNonQuery();
            }

            using GameRecords records = GameRecords.Open(Db);
            GameRecord old = records.Find("/r/old.z64")!;
            Assert.True(old.Favourite);
            Assert.Equal(3, old.PlayCount);
            Assert.Equal(GameRecords.SchemaVersion, UserVersion());
            Assert.Empty(records.Collections());
        }

        [Fact]
        public void A_cached_hash_holds_only_while_the_file_is_the_size_and_age_it_was()
        {
            using GameRecords records = GameRecords.Open(Db);
            records.StoreHash("/r/a.nes", 40976, 638000000000000000, "abc");

            Assert.Equal("abc", records.KnownHash("/r/a.nes", 40976, 638000000000000000));
            Assert.Null(records.KnownHash("/r/a.nes", 40977, 638000000000000000));
            Assert.Null(records.KnownHash("/r/a.nes", 40976, 638000000000000001));
            Assert.Equal("/r/a.nes", records.PathByHash("abc"));
        }

        [Fact]
        public void A_moved_game_takes_its_row_and_its_collections_and_merges_into_what_the_new_path_had()
        {
            using GameRecords records = GameRecords.Open(Db);
            records.Started("/r/old name.sfc", new DateTime(2026, 9, 1, 0, 0, 0, DateTimeKind.Local));
            records.Played("/r/old name.sfc", TimeSpan.FromSeconds(100));
            records.ToggleFavourite("/r/old name.sfc");
            records.Identify("/r/old name.sfc", "feed", 1024);
            long rpgs = records.CreateCollection("RPGs", DateTime.Now)!.Value;
            long both = records.CreateCollection("Both", DateTime.Now)!.Value;
            records.AddToCollection(rpgs, "/r/old name.sfc");
            records.AddToCollection(both, "/r/old name.sfc");
            records.AddToCollection(both, "/r/new name.sfc");
            records.Started("/r/new name.sfc", new DateTime(2026, 9, 20, 0, 0, 0, DateTimeKind.Local));

            Assert.Single(records.Orphans(new HashSet<string> { "/r/new name.sfc" }));
            records.Move("/r/old name.sfc", "/r/new name.sfc");

            Assert.Null(records.Find("/r/old name.sfc"));
            GameRecord moved = records.Find("/r/new name.sfc")!;
            Assert.True(moved.Favourite);
            Assert.Equal(2, moved.PlayCount);
            Assert.Equal(100, moved.PlaySeconds, 3);
            Assert.Equal(new DateTime(2026, 9, 20, 0, 0, 0, DateTimeKind.Local), moved.LastPlayed);
            Assert.Equal(new HashSet<long> { rpgs, both }, records.CollectionsOf("/r/new name.sfc"));
            Assert.Empty(records.CollectionsOf("/r/old name.sfc"));
            Assert.Empty(records.Orphans(new HashSet<string> { "/r/new name.sfc" }));
        }

        [Fact]
        public void Collections_are_named_once_in_any_case_and_deleting_one_leaves_the_games()
        {
            using GameRecords records = GameRecords.Open(Db);
            long rpgs = records.CreateCollection("RPGs", DateTime.Now)!.Value;
            long racers = records.CreateCollection("Racers", DateTime.Now)!.Value;
            Assert.Null(records.CreateCollection("rpgs", DateTime.Now));
            Assert.False(records.RenameCollection(racers, "RPGS"));
            Assert.True(records.RenameCollection(racers, "Racing"));

            records.AddToCollection(rpgs, "/r/a.sfc");
            records.AddToCollection(rpgs, "/r/a.sfc");
            records.AddToCollection(rpgs, "/r/b.sfc");
            records.Started("/r/a.sfc", DateTime.Now);
            Assert.Equal(new[] { ("Racing", 0), ("RPGs", 2) }, records.Collections().Select(c => (c.Name, c.Count)).ToArray());

            records.DeleteCollection(rpgs);
            Assert.Empty(records.CollectionsOf("/r/a.sfc"));
            Assert.NotNull(records.Find("/r/a.sfc"));
            Assert.Single(records.Collections());
        }

        private long UserVersion()
        {
            using var db = new SqliteConnection($"Data Source={Db};Pooling=False");
            db.Open();
            using SqliteCommand command = db.CreateCommand();
            command.CommandText = "PRAGMA user_version";
            return (long)command.ExecuteScalar()!;
        }

        private void SetUserVersion(int version)
        {
            using var db = new SqliteConnection($"Data Source={Db};Pooling=False");
            db.Open();
            using SqliteCommand command = db.CreateCommand();
            command.CommandText = $"PRAGMA user_version = {version}";
            command.ExecuteNonQuery();
        }

        // A game's own crop, kept in its row - see EmuSen_Settings_Reference.md §4.100.
        [Fact]
        public void A_game_s_crop_is_kept_cleared_and_follows_the_game_to_a_new_path()
        {
            var crop = new GameCrop(1.3, 1.1, 8.7, 9);
            using (GameRecords records = GameRecords.Open(Db))
            {
                Assert.Null(records.Crop("/r/a.z64"));
                records.SetCrop("/r/a.z64", crop);
                records.SetCrop("/r/b.z64", new GameCrop(0, 0, 0, 0));
                records.ToggleFavourite("/r/c.z64");
            }

            using GameRecords reopened = GameRecords.Open(Db);
            Assert.Equal(crop, reopened.Crop("/r/a.z64"));
            Assert.Equal(EmuSen.Serenity.PictureCrop.Percent(1.3, 8.7, 1.1, 9), crop.Picture);
            // A crop of nothing is still the game's own: the whole picture, whatever its console hides.
            Assert.Equal(new GameCrop(0, 0, 0, 0), reopened.Crop("/r/b.z64"));
            Assert.Null(reopened.Crop("/r/c.z64"));
            Assert.False(reopened.IsFavourite("/r/a.z64"));

            // Renamed, a game keeps its crop; moved onto a game that has one, that one's stays.
            reopened.Move("/r/a.z64", "/r/moved.z64");
            Assert.Equal(crop, reopened.Crop("/r/moved.z64"));
            Assert.Null(reopened.Crop("/r/a.z64"));
            reopened.Move("/r/moved.z64", "/r/b.z64");
            Assert.Equal(new GameCrop(0, 0, 0, 0), reopened.Crop("/r/b.z64"));

            reopened.ClearCrop("/r/b.z64");
            Assert.Null(reopened.Crop("/r/b.z64"));
            Assert.NotNull(reopened.Find("/r/b.z64"));
        }

        [Fact]
        public void A_library_from_before_a_game_could_have_a_crop_gains_the_columns_and_keeps_its_rows()
        {
            using (GameRecords records = GameRecords.Open(Db)) records.ToggleFavourite("/r/a.z64");
            using (var raw = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = Db, Pooling = false }.ToString()))
            {
                raw.Open();
                foreach (string sql in new[] { "ALTER TABLE game DROP COLUMN crop_left", "ALTER TABLE game DROP COLUMN crop_right", "ALTER TABLE game DROP COLUMN crop_top", "ALTER TABLE game DROP COLUMN crop_bottom", "PRAGMA user_version = 5" })
                {
                    using SqliteCommand command = raw.CreateCommand();
                    command.CommandText = sql;
                    command.ExecuteNonQuery();
                }
            }

            using GameRecords migrated = GameRecords.Open(Db);
            Assert.Equal(6, GameRecords.SchemaVersion);
            Assert.Equal(GameRecords.SchemaVersion, UserVersion());
            Assert.True(migrated.IsFavourite("/r/a.z64"));
            Assert.Null(migrated.Crop("/r/a.z64"));
            migrated.SetCrop("/r/a.z64", new GameCrop(2.4, 2.5, 3.5, 3.5));
            Assert.Equal(new GameCrop(2.4, 2.5, 3.5, 3.5), migrated.Crop("/r/a.z64"));
            Assert.True(migrated.IsFavourite("/r/a.z64"));
        }
    }
}
