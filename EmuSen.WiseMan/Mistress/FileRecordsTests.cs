using System;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress
{
    // records.db: the shader pack's build, the stamp it replaces read and left, and a crash that tears a write - see EmuSen_Settings_Reference.md §4.64.
    public class FileRecordsTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFileRecordsTests", Guid.NewGuid().ToString("N"));
        private string Db => Path.Combine(_root, "Library", FileRecords.FileName);
        private string States => Path.Combine(_root, "States");

        // A pre-database sidecar exactly as StateRecord.Write produced it, in the shape existing files on disk have.
        private const string Sidecar = """
            {
              "Console": "NES",
              "Core": "NES",
              "StateVersion": 3,
              "Build": "1.0.0+9b9717da46cd02b43d774fc8a025dcd841f22e46",
              "SavedAt": "2026-09-21T23:41:45.3170817-04:00",
              "RomFile": "Dr Mario (VS).nes",
              "RomMd5": "274647f63bcb6e7eb68ab732b3ff93c1",
              "RomBytes": 98320
            }
            """;

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string State(string name, int bytes = 64)
        {
            Directory.CreateDirectory(States);
            string path = Path.Combine(States, name);
            File.WriteAllBytes(path, Enumerable.Repeat((byte)0x5A, bytes).ToArray());
            return path;
        }

        [Fact]
        public void A_new_file_is_stamped_with_the_schema_and_kept_in_wal_mode()
        {
            using (FileRecords.Open(Db)) { }
            Assert.Equal(FileRecords.SchemaVersion, (int)(long)Pragma("user_version")!);
            Assert.Equal("wal", Pragma("journal_mode"));
        }

        [Fact]
        public void A_file_from_a_newer_build_is_refused_and_left_as_it_was()
        {
            using (FileRecords.Open(Db)) { }
            Pragma($"user_version = {FileRecords.SchemaVersion + 1}");
            byte[] before = File.ReadAllBytes(Db);

            Assert.Throws<InvalidDataException>(() => FileRecords.Open(Db));
            Assert.Equal(FileRecords.SchemaVersion + 1, (int)(long)Pragma("user_version")!);
            Assert.Equal(before, File.ReadAllBytes(Db));
        }

        [Fact]
        public void An_older_file_is_migrated_in_place_with_what_it_held_kept()
        {
            Directory.CreateDirectory(Path.GetDirectoryName(Db)!);
            using (var db = new SqliteConnection($"Data Source={Db};Pooling=False"))
            {
                db.Open();
                using SqliteCommand c = db.CreateCommand();
                c.CommandText = "CREATE TABLE keep (x INTEGER); INSERT INTO keep VALUES (7);";
                c.ExecuteNonQuery();
            }

            using (FileRecords records = FileRecords.Open(Db))
            {
                Assert.True(records.RecordPack(Path.Combine(_root, "Pack"), "2026-09-27 02:00 UTC"));
            }
            Assert.Equal(FileRecords.SchemaVersion, (int)(long)Pragma("user_version")!);
            Assert.Equal(7L, Pragma("keep", "SELECT x FROM keep"));
        }

        [Fact]
        public void A_crash_part_way_through_a_write_keeps_every_record_committed_before_it()
        {
            string first = Path.Combine(_root, "First"), second = Path.Combine(_root, "Second");
            Directory.CreateDirectory(first);
            Directory.CreateDirectory(second);
            string crashed = Path.Combine(_root, "Crashed", FileRecords.FileName);
            Directory.CreateDirectory(Path.GetDirectoryName(crashed)!);
            using (FileRecords records = FileRecords.Open(Db))
            {
                Assert.True(records.RecordPack(first, "first"));
                long afterFirst = new FileInfo(Db + "-wal").Length;
                Assert.True(records.RecordPack(second, "second"));
                long afterSecond = new FileInfo(Db + "-wal").Length;
                Assert.True(afterSecond > afterFirst);

                File.Copy(Db, crashed);
                File.Copy(Db + "-wal", crashed + "-wal");
                using FileStream wal = File.Open(crashed + "-wal", FileMode.Open);
                wal.SetLength(afterFirst + (afterSecond - afterFirst) / 2);
            }

            using (FileRecords recovered = FileRecords.Open(crashed))
            {
                Assert.Equal("first", recovered.PackBuilt(first, "none"));
                Assert.Null(recovered.PackBuilt(second, "none"));
            }
            using var check = new SqliteConnection($"Data Source={crashed};Pooling=False");
            check.Open();
            using SqliteCommand c = check.CreateCommand();
            c.CommandText = "PRAGMA integrity_check";
            Assert.Equal("ok", c.ExecuteScalar());
        }

        [Fact]
        public void The_save_states_view_reads_a_record_and_finds_a_renamed_game_by_hash()
        {
            string state = State("Old Name.state");
            File.WriteAllText(StateRecord.SidecarPathFor(state), Sidecar);
            string rom = Path.Combine(_root, "Roms", "New Name.nes");
            Directory.CreateDirectory(Path.GetDirectoryName(rom)!);
            File.WriteAllBytes(rom, new byte[16]);

            MediaItem item = Assert.Single(MediaLibrary.SaveStates(States, new[] { new RomEntry(rom) }, StateRecord.ReadSidecar,
                md5 => md5 == "274647f63bcb6e7eb68ab732b3ff93c1" ? rom : null));
            Assert.Equal("NES", item.Record!.Console);
            Assert.Equal(rom, item.Game!.FullPath);
        }

        [Fact]
        public void An_old_pack_stamp_is_imported_and_left_and_a_download_is_recorded_in_the_database()
        {
            string pack = Path.Combine(_root, "Shaders", "RetroArch");
            Directory.CreateDirectory(pack);
            string stamp = Path.Combine(pack, SlangPackDownload.StampFile);
            File.WriteAllText(stamp, "2026-09-24 18:05 UTC\n");

            using FileRecords records = FileRecords.Open(Db);
            Assert.Equal("2026-09-24 18:05 UTC", records.PackBuilt(pack, SlangPackDownload.StampFile));
            Assert.Equal("stamp", records.PackSource(pack));
            Assert.Equal("2026-09-24 18:05 UTC\n", File.ReadAllText(stamp));

            Assert.True(records.RecordPack(pack, "2026-09-27 02:00 UTC"));
            Assert.Equal("2026-09-27 02:00 UTC", records.PackBuilt(pack, SlangPackDownload.StampFile));
            Assert.Equal("written", records.PackSource(pack));
        }

        [Fact]
        public void A_folder_with_no_pack_has_no_build()
        {
            using FileRecords records = FileRecords.Open(Db);
            string pack = Path.Combine(_root, "Shaders", "RetroArch");
            Assert.True(records.RecordPack(pack, "2026-09-27 02:00 UTC"));
            Assert.Null(records.PackBuilt(pack, SlangPackDownload.StampFile));
            Directory.CreateDirectory(pack);
            Assert.Equal("2026-09-27 02:00 UTC", records.PackBuilt(pack, SlangPackDownload.StampFile));
            Assert.Null(records.PackBuilt(Path.Combine(_root, "Elsewhere"), SlangPackDownload.StampFile));
        }

        private object? Pragma(string pragma, string? sql = null)
        {
            using var db = new SqliteConnection($"Data Source={Db};Pooling=False");
            db.Open();
            using SqliteCommand c = db.CreateCommand();
            c.CommandText = sql ?? $"PRAGMA {pragma}";
            return c.ExecuteScalar();
        }
    }
}
