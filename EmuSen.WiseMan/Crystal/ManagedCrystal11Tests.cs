using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Crystal
{
    // What 1.1 added to the frontend API: consoles.json, the states' records with D8's copy and the load's refusals, a game's settings with no session and SR10's copy, the raw picture, and cheat_op with no game - see EmuSen_Backend.md §14.12.
    [Collection(TestCollections.ProcessGlobals)]
    public class ManagedCrystal11Tests : IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private static JsonElement Json(string text) => JsonDocument.Parse(text).RootElement;

        private static CrystalFrameInfo Frame(CrystalSession s)
        {
            var info = new CrystalFrameInfo();
            s.Frame(ref info);
            return info;
        }

        // SR12: graphics.json's map is copied into consoles.json once, never over a file that exists, and the runtime then reads and writes consoles.json alone.
        [Fact]
        public void Console_settings_move_to_consoles_json_by_one_copy()
        {
            var graphics = new GraphicsConfig();
            graphics.SetValue("TEST", "Wide", "true");
            graphics.Save();
            using (CrystalSession s = _f.Open())
            {
                Assert.Equal("true", s.Setting("Wide", game: false));
                Assert.True(ConsolesConfig.Exists);
                s.SetSetting("Wide", "false", game: false);
            }
            Assert.Equal("false", ConsolesConfig.Load().Value("TEST", "Wide"));
            Assert.Equal("true", GraphicsConfig.Load().Value("TEST", "Wide"));
            graphics.SetValue("TEST", "Wide", "true");
            graphics.Save();
            Assert.Equal("false", ConsolesConfig.Load().Value("TEST", "Wide"));
        }

        // A state saved is recorded in states.db with the session's facts, read back with no session, and forgotten on asking.
        [Fact]
        public void A_saved_state_is_recorded_and_its_record_read_with_no_session()
        {
            string path = Path.Combine(_f.Root, "Recorded.state");
            using CrystalSession s = _f.Open(_f.Game("Recorded.tst"));
            Frame(s);
            s.SaveState(path);
            JsonElement record = Json(_f.Runtime.StateRecord(path)!);
            Assert.Equal((Json(s.Info()).GetProperty("console").GetString(), "Recorded.tst", 1), (record.GetProperty("console").GetString(), record.GetProperty("rom_file").GetString(), record.GetProperty("state_version").GetInt32()));
            Assert.False(string.IsNullOrEmpty(record.GetProperty("build").GetString()));
            File.SetLastWriteTimeUtc(path, DateTime.UtcNow.AddMinutes(1));
            Assert.Null(_f.Runtime.StateRecord(path));
            s.SaveState(path);
            Assert.NotNull(_f.Runtime.StateRecord(path));
            _f.Runtime.StateForget(path);
            Assert.Null(_f.Runtime.StateRecord(path));
        }

        // D8: records.db's state rows are copied into states.db on its first open, none over a row already there, and only once.
        [Fact]
        public void Records_db_state_rows_are_copied_once()
        {
            string library = DataStore.Library;
            Directory.CreateDirectory(library);
            string state = Path.Combine(_f.Root, "Old.state");
            File.WriteAllBytes(state, new byte[] { 1, 2, 3 });
            var file = new FileInfo(state);
            using (var records = new SqliteConnection($"Data Source={Path.Combine(library, "records.db")};Pooling=False"))
            {
                records.Open();
                using SqliteCommand command = records.CreateCommand();
                command.CommandText = """
                    CREATE TABLE state_record (path TEXT PRIMARY KEY, state_bytes INTEGER NOT NULL, state_modified INTEGER NOT NULL, console TEXT NOT NULL, core TEXT NOT NULL,
                        state_version INTEGER NOT NULL, build TEXT NOT NULL, saved_at TEXT NOT NULL, rom_file TEXT NOT NULL, rom_md5 TEXT, rom_bytes INTEGER NOT NULL DEFAULT 0, source TEXT NOT NULL);
                    INSERT INTO state_record VALUES ($path, $bytes, $modified, 'SNES', 'VenusRT (Rust)', 4, 'old build', '2026-10-01T00:00:00', 'Old.sfc', NULL, 0, 'written');
                    """;
                command.Parameters.AddWithValue("$path", Path.GetFullPath(state));
                command.Parameters.AddWithValue("$bytes", file.Length);
                command.Parameters.AddWithValue("$modified", file.LastWriteTimeUtc.Ticks);
                command.ExecuteNonQuery();
            }
            Assert.Equal("old build", Json(_f.Runtime.StateRecord(state)!).GetProperty("build").GetString());
            _f.Runtime.StateForget(state);
            using var again = new CrystalRuntime();
            Assert.Null(again.StateRecord(state));
        }

        // A state the record says another console wrote is refused with STATE_REFUSED and its words, the machine unchanged.
        [Fact]
        public void A_state_another_console_wrote_is_refused_by_its_record()
        {
            string path = Path.Combine(_f.Root, "Other.state");
            using CrystalSession s = _f.Open(_f.Game("Other.tst"));
            Frame(s);
            s.SaveState(path);
            using (var db = new SqliteConnection($"Data Source={Path.Combine(DataStore.Library, "states.db")};Pooling=False"))
            {
                db.Open();
                using SqliteCommand command = db.CreateCommand();
                command.CommandText = "UPDATE state_record SET console = 'SNES'";
                command.ExecuteNonQuery();
            }
            ulong before = Frame(s).total_frames;
            var refused = Assert.Throws<CrystalException>(() => s.LoadState(path));
            Assert.Equal(CrystalAbi.StateRefused, refused.Status);
            Assert.Contains("saved by the SNES core", refused.Message);
            Assert.Equal(before + 1, Frame(s).total_frames);
        }

        // A game's own setting with no session: set, read, refused when the schema refuses it, and read by the game's next session.
        [Fact]
        public void A_game_s_setting_is_set_and_read_with_no_session()
        {
            string game = _f.Game("Own.tst");
            Assert.Null(_f.Runtime.GameSetting(game, "Wide"));
            _f.Runtime.SetGameSetting(game, "Wide", "true");
            Assert.Equal("true", _f.Runtime.GameSetting(game, "Wide"));
            Assert.True(File.Exists(Path.Combine(_f.Root, "Config", "games", "test", "Own.json")));
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => _f.Runtime.SetGameSetting(game, "Wide", "loud")).Status);
            using CrystalSession s = _f.Open(game);
            Assert.Equal("true", s.Setting("Wide"));
            _f.Runtime.SetGameSetting(game, "Wide", null);
            Assert.Null(_f.Runtime.GameSetting(game, "Wide"));
        }

        // SR10: games.db's engine choices are copied to each game's file once, never over a value already there, and a later games.db is not read again.
        [Fact]
        public void Games_db_engine_choices_are_copied_once()
        {
            string db = Path.Combine(_f.Root, "games.db");
            using (var games = new SqliteConnection($"Data Source={db};Pooling=False"))
            {
                games.Open();
                using SqliteCommand command = games.CreateCommand();
                command.CommandText = """
                    CREATE TABLE game_edit (path TEXT NOT NULL, field TEXT NOT NULL, value TEXT NOT NULL, edited TEXT NOT NULL, PRIMARY KEY (path, field));
                    INSERT INTO game_edit VALUES ($chosen, 'altemulator', 'V1TestCore (test)', '2026-10-01');
                    INSERT INTO game_edit VALUES ($kept, 'altemulator', 'V1TestCore (test)', '2026-10-01');
                    INSERT INTO game_edit VALUES ($ignored, 'altemulator', 'No Such Engine', '2026-10-01');
                    """;
                command.Parameters.AddWithValue("$chosen", _f.Game("Chosen.tst"));
                command.Parameters.AddWithValue("$kept", _f.Game("Kept.tst"));
                command.Parameters.AddWithValue("$ignored", _f.Game("Ignored.tst"));
                command.ExecuteNonQuery();
            }
            string kept = EmuSen.Crystal.Managed.ManagedRuntime.GameSettingsPath("test", "Kept.tst");
            Directory.CreateDirectory(Path.GetDirectoryName(kept)!);
            File.WriteAllText(kept, "{\"Engine\": \"Earlier\"}");
            int copied = EmuSen.Crystal.Managed.ManagedRuntime.CopyEngineChoices(db);
            Assert.Equal(1, copied);
            Assert.Contains("V1TestCore (test)", File.ReadAllText(EmuSen.Crystal.Managed.ManagedRuntime.GameSettingsPath("test", "Chosen.tst")));
            Assert.Contains("Earlier", File.ReadAllText(kept));
            Assert.False(File.Exists(EmuSen.Crystal.Managed.ManagedRuntime.GameSettingsPath("test", "Ignored.tst")));
            Assert.Equal("V1TestCore (test)", _f.Runtime.GameSetting(_f.Game("Chosen.tst"), "Engine"));
            Assert.Equal(0, EmuSen.Crystal.Managed.ManagedRuntime.CopyEngineChoices(db));
        }

        // The raw picture: ABSENT until an encoding is chosen, then each frame's values and shape into the caller's buffer.
        [Fact]
        public void The_raw_picture_is_copied_once_chosen()
        {
            using CrystalSession s = _f.Open();
            Frame(s);
            Assert.Equal(-1, s.RawCopy(new byte[64], out _));
            s.RawSelect("x-test.red");
            Frame(s);
            Frame(s);
            var values = new byte[64];
            long bytes = s.RawCopy(values, out CrystalRawInfo info);
            Assert.Equal((1u, 8, 4), (info.element, info.width, info.height));
            Assert.Equal(info.width * info.height, (int)bytes);
            var rgba = new byte[(int)s.PictureBytes];
            s.CopyPicture(rgba);
            Assert.Equal(rgba[0], values[0]);
            s.RawSelect(null);
            Frame(s);
            Assert.Equal(-1, s.RawCopy(values, out _));
        }

        private static readonly uint[] GameFree =
        {
            CrystalSessionAbi.CheatDbSystems, CrystalSessionAbi.CheatDbGames, CrystalSessionAbi.CheatDbFind, CrystalSessionAbi.CheatDbStatus, CrystalSessionAbi.CheatDbParse,
            CrystalSessionAbi.CheatDbPrune, CrystalSessionAbi.CheatDbUpdate, CrystalSessionAbi.CheatChtParse, CrystalSessionAbi.CheatFiles, CrystalSessionAbi.CheatFileRead,
        };

        // A cheat op that changes the list and answers a value, not a record, is made once through the binding: the copy-out protocol's two calls would make it twice.
        [Fact]
        public void A_cheat_change_through_the_binding_is_made_once()
        {
            using CrystalSession s = _f.Open(_f.Game("Cheats.tst"));
            const string file = "{\"Cheats\":[{\"Kind\":\"RamPoke\",\"Description\":\"One\",\"Enabled\":true,\"Writes\":[{\"Space\":\"RAM\",\"Address\":\"10\",\"Value\":\"01\"}]}]}";
            Assert.Equal(1, s.CheatChange(CrystalSessionAbi.CheatImport, file));
            Assert.Equal(1, Json(s.CheatOp(CrystalSessionAbi.CheatList, "{}")).GetProperty("cheats").GetArrayLength());
        }

        // cheat_op with no game: the database's status answers, and every op a game's session alone answers is NO_SESSION.
        [Fact]
        public void Cheat_ops_with_no_game_answer_only_those_that_need_none()
        {
            JsonElement status = Json(_f.Runtime.CheatOp(CrystalSessionAbi.CheatDbStatus, "{}"));
            Assert.True(status.TryGetProperty("exists", out _));
            Assert.Equal(0, Json(_f.Runtime.CheatOp(CrystalSessionAbi.CheatChtParse, "{\"text\":\"\"}")).GetProperty("cheats").GetArrayLength());
            for (uint op = 0; op <= CrystalSessionAbi.CheatDbUpdate; op++)
            {
                if (GameFree.Contains(op)) continue;
                var refused = Assert.Throws<CrystalException>(() => _f.Runtime.CheatOp(op, "{}"));
                Assert.Equal(CrystalSessionAbi.NoSession, refused.Status);
            }
        }
    }
}
