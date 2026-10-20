using System.Collections.Generic;
using System.IO;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Crystal
{
    // Piece 6 of the parity class: the settings' rows and layers, a change written and applied, SR10's copy, and every event a subscription is given - see EmuSen_Backend.md §29.19.
    public partial class CrystalParityTests
    {
        private static void ClearConfig()
        {
            string games = Path.Combine(ConfigStore.Directory, "games");
            if (Directory.Exists(games)) Directory.Delete(games, recursive: true);
            string consoles = Path.Combine(ConfigStore.Directory, ConsolesConfig.FileName);
            if (File.Exists(consoles)) File.Delete(consoles);
        }

        [Fact]
        public void The_test_core_s_settings_and_events_are_the_same()
        {
            string tree = TestTree(plain: false);
            string dir = Path.Combine(_root, "Games");
            Directory.CreateDirectory(dir);
            string plain = Path.Combine(dir, "plain.tst"), other = Path.Combine(dir, "other.tst");
            File.WriteAllBytes(plain, CoreAdapterTests.Image(1, 2, 3, 4));
            File.WriteAllBytes(other, CoreAdapterTests.Image(5, 5));
            void Plant()
            {
                ClearConfig();
                if (Directory.Exists(DataStore.Library)) Directory.Delete(DataStore.Library, recursive: true);
                var stored = new ConsolesConfig();
                stored.SetValue("TEST", "Threads", "2");
                stored.Save();
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                nint all = i.Subscribe(rt, 0, 0), logs = i.Subscribe(rt, 0, CrystalAbi.EventLog | CrystalAbi.EventSession);
                a.AddRange(new[] { (all != 0).ToString(), i.SubscriptionWait(all, 0), i.Drain(all) });
                uint s = Id(i.Open(rt, plain));
                nint mine = i.Subscribe(rt, s, CrystalAbi.EventDisc | CrystalAbi.EventSettings), none = i.Subscribe(rt, 77, 0);
                a.AddRange(new[] { (none == 0).ToString(), i.LastError(), i.SubscriptionWait(all, 0), i.Drain(all) });
                for (int f = 0; f < 70; f++) i.Frame(rt, s);
                a.AddRange(new[]
                {
                    i.SettingsSchema(rt, s, null, null), i.SettingsSchema(rt, 0, "TEST", null), i.SettingsSchema(rt, 0, "TEST", "V1TestCore (test)"), i.SettingsSchema(rt, 0, null, null),
                    i.SettingsSchema(rt, 0, "test", "No Such Engine"), i.SettingsSchema(rt, 99, null, null),
                    i.SettingsGet(rt, s, 0, null, "Wide"), i.SettingsGet(rt, s, 2, null, "Threads"), i.SettingsGet(rt, 0, 0, "TEST", "Engine"), i.SettingsGet(rt, 0, 0, "TEST", "Nope"),
                    i.SettingsGet(rt, 0, 2, "TEST", "Wide"), i.SettingsGet(rt, 0, 0, null, "Wide"), i.SettingsGet(rt, s, 0, "SNES", "Wide"), i.SettingsGet(rt, s, 5, null, "Wide"), i.SettingsGet(rt, s, 0, null, null),
                });
                a.AddRange(new[]
                {
                    i.SettingsSet(rt, s, 0, null, "Wide", "true"), i.SettingsSet(rt, s, 0, null, "Wide", "maybe"), i.SettingsSet(rt, s, 0, null, "Ram", "0"), i.SettingsSet(rt, s, 0, null, "Ram", "12"),
                    i.SettingsSet(rt, s, 0, null, "Rate", "44100"), i.SettingsSet(rt, s, 2, null, "Rate", "48000"), i.SettingsSet(rt, s, 0, null, "Nope", "1"), i.SettingsSet(rt, 0, 0, "TEST", "Threads", null),
                    i.SettingsSet(rt, 0, 0, "TEST", "Engine", "V1TestCore (test)"), i.SettingsSet(rt, s, 2, null, "Threads", " 3 "), i.SettingsSet(rt, s, 0, null, "Wide", " False "),
                    i.SettingsGet(rt, s, 2, null, "Rate"), i.SettingsGet(rt, s, 0, null, "Rate"), i.SettingsGet(rt, s, 0, null, "Wide"), i.SettingsSchema(rt, s, null, null),
                });
                for (int f = 0; f < 10; f++) i.Frame(rt, s);
                a.AddRange(new[] { i.Picture(rt, s), i.Drain(mine), i.Drain(logs) });
                a.AddRange(new[]
                {
                    i.GameSettingGet(rt, plain, "Rate"), i.GameSettingGet(rt, plain, "Wide"), i.GameSettingSet(rt, plain, "Wide", "true"), i.GameSettingGet(rt, plain, "Wide"),
                    i.GameSettingSet(rt, plain, "Rate", null), i.GameSettingSet(rt, plain, "Nope", "1"), i.GameSettingSet(rt, Path.Combine(dir, "x.zzz"), "Wide", "true"),
                    i.GameSettingSet(rt, other, "Ram", "300"), i.GameSettingSet(rt, other, "Ram", "30"), i.GameSettingGet(rt, other, "Ram"), i.GameSettingGet(rt, plain, ""),
                });
                a.AddRange(new[] { i.DiscLid(rt, s, true), i.DiscInsert(rt, s, other), i.DiscChange(rt, s, 0), i.Call("disc_eject", rt, s), i.DiscLid(rt, s, false) });
                a.AddRange(new[] { i.SlotCall("state_slot_save", rt, s, 1), i.SlotCall("state_slot_load", rt, s, 1), i.RewindSet(rt, s, true, 2, 0, 0) });
                for (int f = 0; f < 12; f++) i.Frame(rt, s);
                a.AddRange(new[] { i.RewindStep(rt, s, 2), i.RewindTo(rt, s, 4), i.RewindSet(rt, s, false, 0, 0, 0), i.Call("reset", rt, s) });
                a.AddRange(new[] { i.SubscriptionWait(mine, 0), i.Drain(mine), i.Drain(all), i.Drain(mine), i.SubscriptionWait(mine, 1) });
                for (int k = 0; k < 300; k++) i.DiscLid(rt, s, k % 2 == 0);
                a.AddRange(new[] { i.Drain(mine).Length.ToString(), i.Drain(mine), i.Unsubscribe(mine), i.Drain(mine), i.Unsubscribe(mine), i.Drain(0), i.Call("close", rt, s), i.Drain(all), i.Drain(logs) });
                return a;
            }, also: () => TreeOf(ConfigStore.Directory));
        }

        [Fact]
        public void The_build_s_settings_and_SR10_s_copy_are_the_same()
        {
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string W(string name, byte[] bytes)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, bytes);
                return path;
            }
            string nes = W("Moon.nes", SyntheticNesRom.Build()), owned = W("Owned.nes", SyntheticNesRom.Build()), sfc = W("Bogus.sfc", SyntheticRom.Build());
            string psx = W("Psx.iso", SyntheticPsxDisc.Build().Iso);
            void Plant()
            {
                ClearConfig();
                if (Directory.Exists(DataStore.Library)) Directory.Delete(DataStore.Library, recursive: true);
                Directory.CreateDirectory(DataStore.Library);
                using (var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = Path.Combine(DataStore.Library, "games.db"), Pooling = false }.ToString()))
                {
                    db.Open();
                    using SqliteCommand c = db.CreateCommand();
                    c.CommandText = "CREATE TABLE game_edit (path TEXT NOT NULL, field TEXT NOT NULL, value TEXT NOT NULL, edited TEXT NOT NULL, PRIMARY KEY (path, field));"
                        + "INSERT INTO game_edit VALUES ($nes, 'altemulator', 'MoonRT (Rust)', 'x'), ($owned, 'altemulator', 'MoonRT (Rust)', 'x'), ($sfc, 'altemulator', 'Bogus', 'x'),"
                        + " ($odd, 'altemulator', 'MoonRT (Rust)', 'x'), ($nes, 'title', 'Named', 'x');";
                    c.Parameters.AddWithValue("$nes", nes);
                    c.Parameters.AddWithValue("$owned", owned);
                    c.Parameters.AddWithValue("$sfc", sfc);
                    c.Parameters.AddWithValue("$odd", Path.Combine(dir, "Odd.xyz"));
                    c.ExecuteNonQuery();
                }
                string own = Path.Combine(ConfigStore.Directory, "games", "nes", "Owned.json");
                Directory.CreateDirectory(Path.GetDirectoryName(own)!);
                File.WriteAllText(own, "{\"Engine\":\"Someone\",\"Other\":\"1\"}");
                var stored = new ConsolesConfig();
                stored.SetValue("PlayStation", "Cards", "named");
                stored.SetValue("PlayStation", "Card1", "mine");
                stored.SetValue("PlayStation", "Card2", "MINE");
                stored.Save();
            }
            SameWithFiles(BuildTree, Plant, (i, rt) =>
            {
                var a = new List<string>();
                foreach (string console in new[] { "NES", "SNES", "GB", "GBC", "N64", "Genesis", "Sega CD", "PlayStation", "Master System", "SG-1000", "Nowhere" })
                    a.AddRange(new[] { i.SettingsSchema(rt, 0, console, null), i.SettingsGet(rt, 0, 0, console, "Engine") });
                a.AddRange(new[] { i.GameSettingGet(rt, nes, "Engine"), i.GameSettingGet(rt, owned, "Engine"), i.GameSettingGet(rt, sfc, "Engine") });
                uint s = Id(i.Open(rt, psx));
                a.AddRange(new[] { i.SettingsSchema(rt, s, null, null), i.SettingsSet(rt, 0, 0, "PlayStation", "Card2", "other"), i.SettingsSchema(rt, 0, "PlayStation", null), i.Call("close", rt, s) });
                return a;
            }, also: () => TreeOf(ConfigStore.Directory));
        }
    }
}
