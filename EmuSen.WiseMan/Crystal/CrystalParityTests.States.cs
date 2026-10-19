using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Crystal
{
    // Piece 5 of the parity class: save states, their records in states.db, D8's copy and the pictures beside them, every answer and file compared - see EmuSen_Backend.md §29.18.
    public partial class CrystalParityTests
    {
        // The two answers a run cannot make alike: the moment kept as its shape, its digits hidden, and the build.
        private static string Timeless(string answer) =>
            Regex.Replace(Regex.Replace(answer, "\"saved_at\":\"[^\"]+\"", m => Regex.Replace(m.Value, "[0-9]", "n")), "\"build\":\"[^\"]+\"", "\"build\":\"(a build)\"");

        // A Mistress records.db holding one state's row, for D8's copy.
        private static void PlantRecordsDb(string folder, string statePath)
        {
            Directory.CreateDirectory(folder);
            var info = new FileInfo(statePath);
            using var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = Path.Combine(folder, "records.db"), Pooling = false }.ToString());
            db.Open();
            using SqliteCommand c = db.CreateCommand();
            c.CommandText = """
                CREATE TABLE state_record (path TEXT PRIMARY KEY, source TEXT, state_bytes INTEGER NOT NULL, state_modified INTEGER NOT NULL, console TEXT NOT NULL, core TEXT NOT NULL,
                    state_version INTEGER NOT NULL, build TEXT NOT NULL, saved_at TEXT NOT NULL, rom_file TEXT NOT NULL, rom_md5 TEXT, rom_bytes INTEGER NOT NULL DEFAULT 0);
                INSERT INTO state_record VALUES ($path, 'mistress', $bytes, $ticks, 'TEST', 'Old Core', 3, 'old build', '2026-09-01T09:30:00.5000000Z', 'copied.tst', NULL, 12);
                """;
            c.Parameters.AddWithValue("$path", Path.GetFullPath(statePath));
            c.Parameters.AddWithValue("$bytes", info.Length);
            c.Parameters.AddWithValue("$ticks", info.LastWriteTimeUtc.Ticks);
            c.ExecuteNonQuery();
        }

        [Fact]
        public void The_test_core_s_states_their_records_and_pictures_are_the_same()
        {
            string tree = TestTree(plain: false);
            string dir = Path.Combine(_root, "Games");
            Directory.CreateDirectory(dir);
            string plain = Path.Combine(dir, "plain.tst"), other = Path.Combine(dir, "other.tst");
            File.WriteAllBytes(plain, CoreAdapterTests.Image(1, 2, 3, 4));
            File.WriteAllBytes(other, CoreAdapterTests.Image(5, 5));
            string elsewhere = Path.Combine(_root, "Elsewhere");
            string copied = Path.Combine(elsewhere, "copied.state"), stale = Path.Combine(elsewhere, "stale.state");
            string foreign = Path.Combine(elsewhere, "foreign.state"), newer = Path.Combine(elsewhere, "newer.state"), broken = Path.Combine(elsewhere, "broken.state");
            void Plant()
            {
                if (Directory.Exists(DataStore.Library)) Directory.Delete(DataStore.Library, recursive: true);
                if (Directory.Exists(elsewhere)) Directory.Delete(elsewhere, recursive: true);
                Directory.CreateDirectory(elsewhere);
                foreach (string f in new[] { copied, stale, foreign, newer, broken }) File.WriteAllBytes(f, new byte[] { 1, 2, 3 });
                PlantRecordsDb(DataStore.Library, copied);
                File.WriteAllText(Path.ChangeExtension(foreign, ".json"), "{ \"console\": \"SNES\", \"Core\": \"Venus\", // a comment\n \"StateVersion\": 2, \"Build\": \"b\", \"SavedAt\": \"2026-09-20T10:00:00\", \"RomFile\": \"x.sfc\", }");
                File.WriteAllText(Path.ChangeExtension(newer, ".json"), "{\"Console\":\"TEST\",\"Core\":\"V1TestCore (test)\",\"StateVersion\":999,\"Build\":\"b\",\"SavedAt\":\"2026-09-20T10:00:00Z\",\"RomFile\":\"plain.tst\",\"RomMd5\":null}");
                File.WriteAllText(Path.ChangeExtension(broken, ".json"), "{\"Console\":\"TEST\",\"Core\":\"x\",\"StateVersion\":\"two\",\"Build\":\"b\",\"SavedAt\":\"2026-09-20T10:00:00\",\"RomFile\":\"x\"}");
                File.SetLastWriteTimeUtc(stale, new DateTime(2026, 1, 1, 0, 0, 0, DateTimeKind.Utc));
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                uint s = Id(i.Open(rt, plain));
                string Slots() => Timeless(i.Read("state_slots", rt, s));
                string Record(string path) => Timeless(i.Read("state_record", rt, path));
                void Run(int frames)
                {
                    for (int f = 0; f < frames; f++)
                    {
                        if (f % 6 == 0) i.Button(rt, s, 0, 8, f % 12 == 0);
                        i.Frame(rt, s);
                    }
                    a.AddRange(new[] { i.Frame(rt, s), i.Picture(rt, s) });
                }
                a.AddRange(new[] { Slots(), i.SlotCall("state_slot_load", rt, s, 1), i.SlotCall("state_slot_save", rt, s, 0), i.SlotCall("state_slot_save", rt, s, 9), i.Read("state_default_path", rt, s) });
                Run(30);
                a.AddRange(new[] { i.SlotCall("state_slot_save", rt, s, 1), i.SlotCall("state_slot_save", rt, s, 3), i.Call("resume_save", rt, s), Slots() });
                string slot1 = System.Text.Json.Nodes.JsonNode.Parse(i.Read("state_slots", rt, s))![1]!["path"]!.GetValue<string>();
                a.AddRange(new[] { Record(slot1), Record(copied), Record(stale), Record(foreign), Record(newer), Record(broken), Record(Path.Combine(elsewhere, "none.state")) });
                Run(30);
                a.AddRange(new[] { i.SlotCall("state_slot_load", rt, s, 1), i.Frame(rt, s), i.Picture(rt, s), i.Read("rewind_reel", rt, s) });
                a.AddRange(new[] { i.SlotCall("state_slot_load", rt, s, 2), i.Call("resume_load", rt, s), i.Frame(rt, s), i.Picture(rt, s) });
                a.AddRange(new[] { i.StateTwin("state_load", rt, s, foreign), i.StateTwin("state_load", rt, s, newer), i.StateTwin("state_load", rt, s, broken), i.StateTwin("state_load", rt, s, Path.Combine(elsewhere, "none.state")) });
                a.AddRange(new[] { i.StateTwin("state_save", rt, s, Path.Combine(elsewhere, "deep", "twin.state")), i.StateTwin("state_save", rt, s, null), Record(Path.Combine(elsewhere, "deep", "twin.state")) });
                Run(10);
                a.AddRange(new[] { i.StateTwin("state_load", rt, s, null), i.Frame(rt, s), i.Picture(rt, s), i.StateTwin("state_load", rt, s, Path.Combine(elsewhere, "deep", "twin.state")), i.Picture(rt, s) });
                a.AddRange(new[] { i.DiscLid(rt, s, true), i.DiscInsert(rt, s, other) });
                Run(5);
                a.AddRange(new[] { i.SlotCall("state_slot_load", rt, s, 3), i.Read("session_info", rt, s), i.Picture(rt, s) });
                a.AddRange(new[] { i.Forget(rt, slot1), Record(slot1), i.Forget(rt, slot1), i.Forget(rt, ""), Record(""), i.SlotCall("state_slot_load", rt, s, 1), i.Picture(rt, s) });
                a.Add(i.Call("close", rt, s));
                uint e = Id(i.Open(rt, other));
                a.AddRange(new[] { Timeless(i.Read("state_slots", rt, e)), i.StateTwin("state_load", rt, e, slot1), i.SlotCall("state_slot_save", rt, e, 8), i.Call("close", rt, e) });
                return a;
            }, also: () => TreeOf(elsewhere) + "\n" + TreeOf(DataStore.Library).Split('\n').Count(l => l.StartsWith("states.db")));
        }

        // D8 and the records shared: whichever opens states.db first copies records.db's rows once, and each reads the other's records to the byte.
        [Fact]
        public void States_db_is_shared_between_the_implementations()
        {
            string tree = TestTree(plain: false);
            string game = Path.Combine(_root, "plain.tst");
            File.WriteAllBytes(game, CoreAdapterTests.Image(1, 2, 3, 4));
            string copied = Path.Combine(_root, "copied.state");
            File.WriteAllBytes(copied, new byte[] { 9, 9 });
            PlantRecordsDb(DataStore.Library, copied);
            string Copied()
            {
                using var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = Path.Combine(DataStore.Library, "states.db"), Pooling = false }.ToString());
                db.Open();
                using SqliteCommand c = db.CreateCommand();
                c.CommandText = "SELECT group_concat(source || ' ' || at, ';') FROM copied";
                return c.ExecuteScalar() as string ?? "";
            }
            string SaveAndRead(CrystalImpl impl, uint slot) => Answers(impl, new[] { tree }, (i, rt) =>
            {
                uint s = Id(i.Open(rt, game));
                for (int f = 0; f < 12; f++) i.Frame(rt, s);
                string saved = i.SlotCall("state_slot_save", rt, s, slot);
                string path = System.Text.Json.Nodes.JsonNode.Parse(i.Read("state_slots", rt, s))![(int)slot]!["path"]!.GetValue<string>();
                i.Call("close", rt, s);
                return new[] { saved, path, i.Read("state_record", rt, path) };
            })[2];
            string ReadBy(CrystalImpl impl, string path) => Answers(impl, new[] { tree }, (i, rt) => new[] { i.Read("state_record", rt, path) })[0];
            string byManaged = SaveAndRead(CrystalImpl.Managed, 1);
            string once = Copied();
            string slot1 = Path.Combine(DataStore.SaveStates, "plain.state"), slot2 = Path.Combine(DataStore.SaveStates, "plain.slot2.state");
            Assert.Equal(byManaged, ReadBy(CrystalImpl.Native, slot1));
            Assert.Equal(ReadBy(CrystalImpl.Managed, copied), ReadBy(CrystalImpl.Native, copied));
            string byNative = SaveAndRead(CrystalImpl.Native, 2);
            Assert.Equal(byNative, ReadBy(CrystalImpl.Managed, slot2));
            Assert.Equal(once, Copied());
            Assert.StartsWith("records.db ", once);
            Assert.Contains("\"core\":\"Old Core\"", ReadBy(CrystalImpl.Native, copied));
        }

        [Fact]
        public void The_build_s_engines_save_and_load_states_the_same()
        {
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string W(string name, byte[] bytes)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, bytes);
                return path;
            }
            var games = new List<string>
            {
                W("Battery.nes", SyntheticNesRom.Build(prgBanks: 2, chrBanks: 0, mapper: 1, battery: true)), W("Game.sfc", SyntheticRom.Build()),
                W("Saves.z64", SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k)), W("Psx.iso", SyntheticPsxDisc.Build().Iso),
            };
            SameWithFiles(BuildTree, () => { if (Directory.Exists(DataStore.Library)) Directory.Delete(DataStore.Library, recursive: true); }, (i, rt) =>
            {
                var a = new List<string>();
                foreach (string game in games)
                {
                    uint s = Id(i.Open(rt, game));
                    for (int f = 0; f < 40; f++) i.Frame(rt, s);
                    a.AddRange(new[] { i.SlotCall("state_slot_save", rt, s, 2), i.Picture(rt, s), Timeless(i.Read("state_slots", rt, s)) });
                    for (int f = 0; f < 20; f++) i.Frame(rt, s);
                    a.AddRange(new[] { i.SlotCall("state_slot_load", rt, s, 2), i.Picture(rt, s), i.Frame(rt, s), i.Picture(rt, s), i.Call("close", rt, s) });
                }
                return a;
            });
        }
    }
}
