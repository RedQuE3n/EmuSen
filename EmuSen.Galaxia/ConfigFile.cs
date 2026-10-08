using System;
using System.IO;
using System.Text.Json;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia
{
    // One JSON config file: where it is, how it's read, how it's written; T is whatever gets serialized - see EmuSen_Config_Reference.md §2.
    public sealed class ConfigFile<T> where T : class
    {
        // The model the library owns the schema of, or null for a type whose vocabulary is a frontend's - see EmuSen_RustPlatform.md §4.1.
        private static readonly GalaxiaModel? Model = GalaxiaNative.ModelOf(typeof(T));

        private readonly string? _category;

        public ConfigFile(string fileName) : this(null, fileName) { }

        public ConfigFile(string? category, string fileName)
        {
            _category = category;
            FileName = fileName;
        }

        public string FileName { get; }

        // Resolved per access, never cached: ConfigStore.OverrideDirectory moves underneath long-lived config objects - see §1.3.
        public string Path => _category is null
            ? ConfigStore.For(FileName)
            : ConfigStore.For(_category, FileName);

        public bool Exists => File.Exists(Path);

        // Null for missing, unreadable or corrupt - callers fall back to their own defaults rather than crash, see §2.2.
        public T? Load() => GalaxiaNative.Active ? LoadNative(upgrade: false) : LoadManaged();

        // Why the last Load returned null, or null if it didn't or the file simply wasn't there - see EmuSen_Config_Reference.md §6.2.
        public string? LastLoadError { get; private set; }

        public T Load(Func<T> defaultFactory) => Load() ?? defaultFactory();

        // False when the write failed; best-effort by design, a change that can't reach the disk just won't survive a restart - see §2.2.
        public bool Save(T value) => GalaxiaNative.Active ? SaveNative(value) : SaveManaged(value);

        public bool Delete() => GalaxiaNative.Active ? DeleteNative() : DeleteManaged();

        // The library reads the file, and for a model binds and upgrades it; this binds what comes back to the class - see EmuSen_RustPlatform.md §11.2.
        internal T? LoadNative(bool upgrade)
        {
            try
            {
                string? text = Model is { } model ? GalaxiaNative.ModelLoad(model, _category, FileName, upgrade) : GalaxiaNative.ConfigRead(_category, FileName);
                if (text is null) return null;

                T? value = JsonSerializer.Deserialize<T>(text, ConfigJson.Options);
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

        // A new instance as the library makes one, for a model's own Load when there is no file.
        internal static T NewNative(bool upgrade) =>
            JsonSerializer.Deserialize<T>(GalaxiaNative.ModelNew(Model!.Value, upgrade), ConfigJson.Options)!;

        // The class is serialized here and written there; a class that will not serialize still has its folder made, as before.
        internal bool SaveNative(T value)
        {
            try
            {
                string? document;
                try { document = JsonSerializer.Serialize(value, ConfigJson.Options); }
                catch { document = null; }
                return Model is { } model ? GalaxiaNative.ModelSave(model, _category, FileName, document) : GalaxiaNative.ConfigWrite(_category, FileName, document);
            }
            catch
            {
                return false;
            }
        }

        internal bool DeleteNative()
        {
            try
            {
                return GalaxiaNative.ConfigDelete(_category, FileName);
            }
            catch
            {
                return false;
            }
        }

        // The C# rule: the default, and what the library's is held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal T? LoadManaged()
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

        internal bool SaveManaged(T value)
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

        internal bool DeleteManaged()
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

        // Full write then rename, so an interrupted save leaves the previous config intact instead of a truncated one - see §2.3.
        private static void WriteAtomic(string path, string contents)
        {
            string temp = path + ".tmp";
            File.WriteAllText(temp, contents);
            File.Move(temp, path, overwrite: true);
        }

        // Returns the path to read from, or null if there was nothing to migrate. Copies rather than moves - see §1.4.
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
