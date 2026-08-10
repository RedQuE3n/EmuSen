using System;
using System.IO;
using System.Reflection;
using System.Text.Json;

namespace EmuSen.LunaP.Settings
{
    // The default store: indented JSON under a directory the host names - see EmuSen_LunaP.md §19.1.
    public sealed class JsonSettingsStore : ISettingsStore
    {
        // Deliberately the same shape EmuSen.Galaxia uses, so the files it already wrote still read.
        public static readonly JsonSerializerOptions Options = new()
        {
            WriteIndented = true,
            ReadCommentHandling = JsonCommentHandling.Skip,
            AllowTrailingCommas = true,
            PropertyNameCaseInsensitive = true,
        };

        private readonly string _root;

        public JsonSettingsStore(string root) => _root = root;

        // Named after the entry assembly when the host says nothing, so an application that never configures LunaP still gets its own folder.
        public static JsonSettingsStore ForApplication(string? programName = null) =>
            new(Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                programName ?? Assembly.GetEntryAssembly()?.GetName().Name ?? "LunaP"));

        public string Directory(string? category) =>
            category is null ? _root : Path.Combine(_root, category);

        public T? Load<T>(string? category, string fileName) where T : class
        {
            try
            {
                string path = PathFor(category, fileName);
                if (!File.Exists(path)) return null;
                return JsonSerializer.Deserialize<T>(File.ReadAllText(path), Options);
            }
            catch (Exception ex)
            {
                LunaSettings.Report($"{PathFor(category, fileName)}: {ex.Message} Falling back to defaults.");
                return null;
            }
        }

        public bool Save<T>(string? category, string fileName, T value) where T : class
        {
            try
            {
                string path = PathFor(category, fileName);
                System.IO.Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                WriteAtomic(path, JsonSerializer.Serialize(value, Options));
                return true;
            }
            catch
            {
                return false;
            }
        }

        private string PathFor(string? category, string fileName) => Path.Combine(Directory(category), fileName);

        // Full write then rename, so an interrupted save leaves the previous file intact instead of a truncated one.
        private static void WriteAtomic(string path, string contents)
        {
            string temp = path + ".tmp";
            File.WriteAllText(temp, contents);
            File.Move(temp, path, overwrite: true);
        }
    }
}
