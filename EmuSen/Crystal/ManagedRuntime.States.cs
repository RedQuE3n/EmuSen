using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Crystal.Session;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using Microsoft.Data.Sqlite;

namespace EmuSen.Crystal.Managed
{
    // 1.1's runtime-wide calls: the states' records, a game's own settings with no session, games.db's engine choices copied once, and the cheat database with no game - see EmuSen_Backend.md §14.12.
    internal sealed unsafe partial class ManagedRuntime
    {
        private StateRecords? _records;
        private string? _recordsPath;
        private SessionService? _noGame;

        // states.db, opened on first use and again where the data folder moved, as a test's override moves it.
        public StateRecords Records
        {
            get
            {
                lock (_gate)
                {
                    string path = StateRecords.DefaultPath;
                    if (_records is null || _recordsPath != path)
                    {
                        _records?.Dispose();
                        (_records, _recordsPath) = (StateRecords.Open(path), path);
                    }
                    return _records;
                }
            }
        }

        public byte[] StateRecordJson(string path)
        {
            StateRecord record = Records.Read(path) ?? throw new CrystalException(SessionAbi.Absent, $"no record of the state at {path}");
            return CrystalJson.Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("console", record.Console);
                w.WriteString("core", record.Core);
                w.WriteNumber("state_version", record.StateVersion);
                w.WriteString("build", record.Build);
                w.WriteString("saved_at", record.SavedAt.ToString("o"));
                w.WriteString("rom_file", record.RomFile);
                CrystalJson.StringOrNull(w, "rom_md5", record.RomMd5);
                w.WriteNumber("rom_bytes", record.RomBytes);
                w.WriteEndObject();
            });
        }

        // A game's own value with no session; the engine's choice read through its console's catalog, as a session reads it.
        public static string? GameSetting(string path, string key)
        {
            if (!GameValues(SystemOfFile(path), path).TryGetValue(key, out string? own)) return null;
            return key == CoreCatalog.EngineKey && CoreCatalog.ConsoleForRom(path) is { } console ? CoreCatalog.EngineChosen(console, own) : own;
        }

        public void SetGameSetting(string path, string key, string? value)
        {
            string console = CoreCatalog.ConsoleForRom(path) ?? SystemOfFile(path)?.ToUpperInvariant()
                ?? throw new CrystalException(SessionAbi.NotSupported, $"{Path.GetFileName(path)} runs as no system of the table, so it has no file of its own settings");
            CoreSetting row = Schema(console, null).FirstOrDefault(r => r.Key == key)
                ?? throw new CrystalException(SessionAbi.NoSuchId, $"the {console} has no setting \"{key}\"");
            if (value is not null && Refusal(row, value) is { } why) throw new CrystalException(SessionAbi.BadArgument, why);
            string? system = SystemOfFile(path);
            Dictionary<string, string> values = GameValues(system, path);
            if (value is null) values.Remove(key);
            else values[key] = value;
            WriteGameValues(system, path, values);
            foreach (ManagedSession open in Sessions.Where(o => o.GamePath is { } p && string.Equals(Path.GetFullPath(p), Path.GetFullPath(path), StringComparison.Ordinal)))
                open.ApplySetting(key, GetSettingOf(open, key));
            Events.Raise(0, CrystalNumbers.EventSettings, "settings", w =>
            {
                w.WriteString("scope", "game");
                w.WriteString("console", console);
                w.WriteString("key", key);
                CrystalJson.StringOrNull(w, "value", value);
            });
        }

        // The marker that games.db's engine choices were copied, in the folder of the files they went to.
        private static string EngineChoicesCopied => Path.Combine(ConfigStore.Directory, "games", ".engine-choices-copied");

        // SR10's one migration: each game's "altemulator" in games.db copied to its own file as "Engine", never over a value already there.
        public static int CopyEngineChoices(string? gamesDb = null)
        {
            gamesDb ??= Path.Combine(DataStore.Library, "games.db");
            if (!File.Exists(gamesDb) || File.Exists(EngineChoicesCopied)) return 0;
            var choices = new List<(string Path, string Engine)>();
            try
            {
                using var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = gamesDb, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
                db.Open();
                using SqliteCommand command = db.CreateCommand();
                command.CommandText = "SELECT path, value FROM game_edit WHERE field = 'altemulator'";
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read()) choices.Add((row.GetString(0), row.GetString(1)));
            }
            catch (SqliteException e)
            {
                ErrorLog.Warning("settings", $"games.db's engine choices could not be read, and none was copied: {e.Message}");
                return 0;
            }
            int copied = 0;
            foreach (var (path, engine) in choices)
            {
                if (engine.Length == 0 || SystemOfFile(path) is not { } system) continue;
                string console = CoreCatalog.ConsoleForRom(path) ?? system.ToUpperInvariant();
                if (Schema(console, null).FirstOrDefault(r => r.Key == CoreCatalog.EngineKey)?.Choices?.Contains(engine) != true) continue;
                Dictionary<string, string> values = GameValues(system, path);
                if (values.ContainsKey(CoreCatalog.EngineKey)) continue;
                values[CoreCatalog.EngineKey] = engine;
                WriteGameValues(system, path, values);
                copied++;
            }
            Directory.CreateDirectory(Path.GetDirectoryName(EngineChoicesCopied)!);
            File.WriteAllText(EngineChoicesCopied, $"{copied} engine choice(s) copied from {gamesDb} on {DateTime.Now:yyyy-MM-dd}\n");
            return copied;
        }

        public static bool IsGameFreeCheatOp(uint op) => SessionService.GameFreeCheatOps.Contains(op);

        // The provider session 0's cheat ops go to: no target, the player's cheat folder and the catalog's systems.
        public SessionService NoGame
        {
            get
            {
                lock (_gate)
                    return _noGame ??= new SessionService(null, new SessionServiceOptions
                    {
                        CheatSystems = () => CoreCatalog.SupportedCheatSystems,
                        CheatDatabaseFolder = () => SettingFolders.Cheats(AppSettings.Load()),
                    });
            }
        }
    }
}
