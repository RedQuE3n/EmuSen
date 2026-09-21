using System;
using System.IO;
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
    }
}
