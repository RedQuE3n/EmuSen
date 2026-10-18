using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Crystal.Session;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Crystal.Managed
{
    // Settings in layers, game over console over the schema's default: the console's in graphics.json, a game's in a file of its own - see EmuSen_Backend.md §3.6, SR10 and §14.
    internal sealed unsafe partial class ManagedRuntime
    {
        // Where a game's own settings are kept: /etc/EmuSen/games/<system>/<file stem>.json, by the systems table's id, so two consoles' games of one name keep two files and no library is needed - see EmuSen_Backend.md §14.7.
        public static string GameSettingsPath(string system, string gameKey) =>
            Path.Combine(ConfigStore.Directory, "games", system, Path.GetFileNameWithoutExtension(gameKey) + ".json");

        private static Dictionary<string, string> GameValues(string? system, string gameKey)
        {
            if (system is null) return new Dictionary<string, string>(StringComparer.Ordinal);
            string path = GameSettingsPath(system, gameKey);
            if (AtomicFile.TryRead(path) is not { } bytes) return new Dictionary<string, string>(StringComparer.Ordinal);
            try { return JsonSerializer.Deserialize<Dictionary<string, string>>(bytes) ?? new Dictionary<string, string>(StringComparer.Ordinal); }
            catch (JsonException e)
            {
                ErrorLog.Warning("settings", $"{path} is not a game's settings, and is read as none: {e.Message}");
                return new Dictionary<string, string>(StringComparer.Ordinal);
            }
        }

        private static void WriteGameValues(string? system, string gameKey, Dictionary<string, string> values)
        {
            if (system is null) throw new CrystalException(SessionAbi.NotSupported, $"{Path.GetFileName(gameKey)} runs as no system of the table, so it has no file of its own settings");
            string path = GameSettingsPath(system, gameKey);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            if (!AtomicFile.Write(path, JsonSerializer.SerializeToUtf8Bytes(values, new JsonSerializerOptions { WriteIndented = true })))
                throw new CrystalException(CrystalNumbers.Io, $"{path} could not be written");
        }

        // The engine a game opens on: the game's own choice, else its console's, else the console's default - see EmuSen_Backend.md §4.3's F3.
        public static string? EngineFor(string romPath)
        {
            string? console = CoreCatalog.ConsoleForRom(romPath);
            if (GameValues(SystemOfFile(romPath), romPath).TryGetValue(CoreCatalog.EngineKey, out string? own)) return console is null ? own : CoreCatalog.EngineChosen(console, own);
            return console is null ? null : ConsoleEngine(console);
        }

        public static string? ConsoleEngine(string console) => CoreCatalog.EngineChosen(console, ConsolesConfig.Load().Value(console, CoreCatalog.EngineKey));

        // A console's settings, the engine's choice first: a running session's engine's own, else the named engine's where it is a discovered engine of the console.
        public static IReadOnlyList<CoreSetting> Schema(string console, string? engine, ManagedSession? session = null)
        {
            var rows = new List<CoreSetting>();
            Cores.Native.DiscoveredCore[] engines = EnginesOfConsole(console);
            if (CoreCatalog.EngineFor(console) is { } choice) rows.Add(choice);
            else if (engines.Length > 0)
                rows.Add(new CoreSetting(CoreCatalog.EngineKey, "Engine", "Which implementation runs the console. Takes effect when a game is next loaded.", CoreSettingKind.Choice,
                    engines[0].EngineName, Choices: engines.Select(e => e.EngineName).ToArray()));
            if (session?.Core is ICoreSettings running && engine is null)
            {
                rows.AddRange(running.Settings);
                rows.AddRange(MemoryCards.SettingsFor(console));
                return rows;
            }
            Cores.Native.DiscoveredCore? named = engine is null ? null : engines.FirstOrDefault(e => e.EngineName == engine && !CoreCatalog.IsRegisteredEngine(engine));
            IReadOnlyList<CoreSetting> own = named is not null ? CoreCatalog.CoreSidecarSettings(named) : CoreCatalog.SettingsFor(console);
            if (own.Count == 0 && named is null && engines.FirstOrDefault(e => !CoreCatalog.IsRegisteredEngine(e.EngineName)) is { } sole) own = CoreCatalog.CoreSidecarSettings(sole);
            rows.AddRange(own);
            rows.AddRange(MemoryCards.SettingsFor(console));
            return rows;
        }

        // The discovered engines that run a console's systems: its pack's, the catalog's, or a system an engine alone names, whose console is its id in capitals.
        private static Cores.Native.DiscoveredCore[] EnginesOfConsole(string console)
        {
            var ids = CoreCatalog.SystemIdsFor(console).Concat(Crystal.Systems.SystemPacks.All.Where(p => string.Equals(p.Entry.Console, console, StringComparison.OrdinalIgnoreCase)).Select(p => p.Entry.Id)).ToHashSet(StringComparer.Ordinal);
            return Cores.Native.CoreDiscovery.Found.Where(c => c.Info.Systems.Any(s => ids.Contains(s.Id) || string.Equals(s.Id, console, StringComparison.OrdinalIgnoreCase))).ToArray();
        }

        // The rows, each with its note as the console's stored values stand, a value not stored read at its row's default - see EmuSen_Backend.md §14.13.
        public static byte[] SchemaJson(IReadOnlyList<CoreSetting> rows, string? console = null) => CrystalJson.Write(w =>
        {
            ConsolesConfig stored = ConsolesConfig.Load();
            string Value(string key) => (console is null ? null : stored.Value(console, key)) ?? rows.FirstOrDefault(r => r.Key == key)?.Default ?? "";
            w.WriteStartArray();
            foreach (CoreSetting s in rows)
            {
                w.WriteStartObject();
                w.WriteString("key", s.Key);
                w.WriteString("label", s.Label);
                w.WriteString("hint", s.Hint);
                w.WriteString("kind", s.Kind switch { CoreSettingKind.Switch => "switch", CoreSettingKind.Count => "count", _ => "choice" });
                w.WriteString("default", s.Default);
                w.WriteNumber("min", s.Min);
                w.WriteNumber("max", s.Max);
                CrystalJson.Strings(w, "choices", s.Choices ?? Array.Empty<string>());
                CrystalJson.Strings(w, "choice_labels", s.ChoiceLabels ?? Array.Empty<string>());
                CrystalJson.StringOrNull(w, "group", s.Group);
                if (s.Note is { } rule) CrystalJson.StringOrNull(w, "note", rule(Value));
                if (s.ShownWith is { } with)
                {
                    w.WriteStartObject("shown_with");
                    w.WriteString("key", with.Key);
                    w.WriteString("value", with.Value);
                    w.WriteEndObject();
                }
                else w.WriteNull("shown_with");
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        private (string Console, ManagedSession? Session) Target(uint session, uint scope, string? console)
        {
            if (scope is not (CrystalNumbers.ScopeConsole or CrystalNumbers.ScopeGame)) throw new CrystalException(SessionAbi.NotSupported, $"no settings scope {scope} in 1.{CrystalNumbers.Minor}");
            if (session == 0)
            {
                if (scope == CrystalNumbers.ScopeGame) throw new CrystalException(SessionAbi.BadArgument, "a game's settings are a session's: name the session");
                return (console ?? throw new CrystalException(SessionAbi.BadArgument, "with no session, the console is named"), null);
            }
            ManagedSession s = Session(session);
            if (console is not null && !string.Equals(console, s.Console, StringComparison.OrdinalIgnoreCase)) throw new CrystalException(SessionAbi.BadArgument, $"session {s.Id} runs the {s.Console}, not the {console}");
            return (s.Console, s);
        }

        public string? GetSetting(uint session, uint scope, string? console, string key)
        {
            var (name, s) = Target(session, scope, console);
            string? stored = scope == CrystalNumbers.ScopeGame ? GetSettingOf(s!, key) : Layered(name, null, null, key);
            return stored ?? Schema(name, null, s).FirstOrDefault(r => r.Key == key)?.Default;
        }

        // A session's stored value of a key, game over console; null where neither holds one.
        public static string? GetSettingOf(ManagedSession s, string key) => Layered(s.Console, s.SystemId, s.GameKey, key);

        private static string? Layered(string name, string? system, string? gameKey, string key)
        {
            if (gameKey is not null && GameValues(system, gameKey).TryGetValue(key, out string? own)) return key == CoreCatalog.EngineKey ? CoreCatalog.EngineChosen(name, own) : own;
            string? stored = ConsolesConfig.Load().Value(name, key);
            return key == CoreCatalog.EngineKey ? CoreCatalog.EngineChosen(name, stored) : stored;
        }

        public void SetSetting(uint session, uint scope, string? console, string key, string? value)
        {
            var (name, s) = Target(session, scope, console);
            CoreSetting? row = Schema(name, null, s).FirstOrDefault(r => r.Key == key)
                ?? throw new CrystalException(SessionAbi.NoSuchId, $"the {name} has no setting \"{key}\"");
            if (value is not null && Refusal(row, value) is { } why) throw new CrystalException(SessionAbi.BadArgument, why);
            if (scope == CrystalNumbers.ScopeGame)
            {
                Dictionary<string, string> values = GameValues(s!.SystemId, s.GameKey);
                if (value is null) values.Remove(key);
                else values[key] = value;
                WriteGameValues(s.SystemId, s.GameKey, values);
            }
            else
            {
                ConsolesConfig config = ConsolesConfig.Load();
                if (value is null) config.Forget(name, key);
                else config.SetValue(name, key, value);
                config.Save();
            }
            foreach (ManagedSession open in Sessions.Where(o => string.Equals(o.Console, name, StringComparison.OrdinalIgnoreCase) && (scope == CrystalNumbers.ScopeConsole || o == s)))
                open.ApplySetting(key, GetSettingOf(open, key));
            Events.Raise(s?.Id ?? 0, CrystalNumbers.EventSettings, "settings", w =>
            {
                w.WriteString("scope", scope == CrystalNumbers.ScopeGame ? "game" : "console");
                w.WriteString("console", name);
                w.WriteString("key", key);
                CrystalJson.StringOrNull(w, "value", value);
            });
        }

        // Why a value is not one the setting takes, or null.
        private static string? Refusal(CoreSetting row, string value) => row.Kind switch
        {
            CoreSettingKind.Switch when !bool.TryParse(value, out _) => $"{row.Key} is a switch, true or false, not \"{value}\"",
            CoreSettingKind.Count when !int.TryParse(value, out int n) || n < row.Min || n > row.Max => $"{row.Key} is a count from {row.Min} to {row.Max}, not \"{value}\"",
            CoreSettingKind.Choice when row.Choices is { Count: > 0 } choices && !choices.Contains(value) => $"{row.Key} is one of {string.Join(", ", choices)}, not \"{value}\"",
            _ => null,
        };
    }
}
