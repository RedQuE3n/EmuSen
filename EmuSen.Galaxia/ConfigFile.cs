using System;
using System.IO;
using System.Text.Json;

namespace EmuSen.Galaxia
{
    // One JSON config file: where it is, how it's read, how it's written.
    // T is whatever actually gets serialized - a settings object for some,
    // a plain Dictionary for the binding maps - see EmuSen_Config_Reference.md §2.
    public sealed class ConfigFile<T> where T : class
    {
        private readonly string? _category;

        public ConfigFile(string fileName) : this(null, fileName) { }

        public ConfigFile(string? category, string fileName)
        {
            _category = category;
            FileName = fileName;
        }

        public string FileName { get; }

        // Resolved per access, never cached: ConfigStore.OverrideDirectory
        // moves underneath long-lived config objects - see §1.3.
        public string Path => _category is null
            ? ConfigStore.For(FileName)
            : ConfigStore.For(_category, FileName);

        public bool Exists => File.Exists(Path);

        // Null for missing, unreadable or corrupt - callers fall back to
        // their own defaults rather than crash, see §2.2.
        public T? Load()
        {
            try
            {
                string path = Path;
                if (!File.Exists(path))
                {
                    string? migrated = MigrateFromLegacy(path);
                    if (migrated is null) return null;
                    path = migrated;
                }

                T? value = JsonSerializer.Deserialize<T>(File.ReadAllText(path), ConfigJson.Options);
                LastLoadError = null;
                return value;
            }
            catch (Exception ex)
            {
                LastLoadError = ex.Message;
                ConfigDiagnostics.Report($"{Path}: {ex.Message} Falling back to defaults.");
                return null;
            }
        }

        // Why the last Load returned null, or null if it didn't or the file
        // simply wasn't there - see EmuSen_Config_Reference.md §6.2.
        public string? LastLoadError { get; private set; }

        public T Load(Func<T> defaultFactory) => Load() ?? defaultFactory();

        // False when the write failed. Best-effort by design: a settings
        // change that can't reach the disk shouldn't take the program with
        // it, it just won't survive a restart - see §2.2.
        public bool Save(T value)
        {
            try
            {
                string path = Path;
                Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
                WriteAtomic(path, JsonSerializer.Serialize(value, ConfigJson.Options));
                return true;
            }
            catch
            {
                return false;
            }
        }

        public bool Delete()
        {
            try
            {
                File.Delete(Path);
                return true;
            }
            catch
            {
                return false;
            }
        }

        // Full write then rename, so an interrupted save leaves the previous
        // config intact instead of a truncated one - see §2.3.
        private static void WriteAtomic(string path, string contents)
        {
            string temp = path + ".tmp";
            File.WriteAllText(temp, contents);
            File.Move(temp, path, overwrite: true);
        }

        // Returns the path to read from, or null if there was nothing to
        // migrate. Copies rather than moves - see §1.4.
        private string? MigrateFromLegacy(string destination)
        {
            if (_category is not null) return null;

            string? legacy = ConfigStore.LegacyPathFor(FileName);
            if (legacy is null || !File.Exists(legacy)) return null;

            try
            {
                Directory.CreateDirectory(System.IO.Path.GetDirectoryName(destination)!);
                File.Copy(legacy, destination);
                return destination;
            }
            catch
            {
                return legacy;
            }
        }
    }
}
