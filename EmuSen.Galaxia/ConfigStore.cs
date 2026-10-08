using System;
using System.IO;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia
{
    // The single answer to "where does a config file go" - see EmuSen_Config_Reference.md §1.
    public static class ConfigStore
    {
        public const string ConfigDirName = "etc";
        public const string ProgramDirName = "EmuSen";

        private static string? _overrideDirectory;
        private static string? _overrideLegacyDirectory;

        // Tests only; null restores the real location. Separate from ConfigRoot, which stays fixed - see EmuSen_Config_Reference.md §1.3.
        public static string? OverrideDirectory
        {
            get => _overrideDirectory;
            set
            {
                lock (GalaxiaNative.Gate)
                {
                    _overrideDirectory = value;
                    GalaxiaNative.OverrideChanged(GalaxiaOverride.Config, value);
                }
            }
        }

        // Under home so /etc/EmuSen stays inside the shell root; built from ConfigRoot, not DataStore - see EmuSen_Galaxia.md §3.1.
        public static string Directory => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.Config)! : Managed.Directory;

        // Where config sat before it moved under home - see EmuSen_Galaxia.md §3.2.
        public static string PreviousDirectory => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.ConfigPrevious)! : Managed.PreviousDirectory;

        public static string For(string fileName) => GalaxiaNative.Active ? GalaxiaNative.ConfigPath(null, fileName) : Managed.For(fileName);

        // One subdirectory per category, for config there can be many of - see §1.2.
        public static string For(string category, string fileName) =>
            GalaxiaNative.Active ? GalaxiaNative.ConfigPath(category, fileName) : Managed.For(category, fileName);

        // Tests only, as above.
        public static string? OverrideLegacyDirectory
        {
            get => _overrideLegacyDirectory;
            set
            {
                lock (GalaxiaNative.Gate)
                {
                    _overrideLegacyDirectory = value;
                    GalaxiaNative.OverrideChanged(GalaxiaOverride.Legacy, value);
                }
            }
        }

        // Pre-Galaxia location, read once to migrate out of - see §1.4.
        public static string LegacyDirectory => GalaxiaNative.Active ? GalaxiaNative.Directory(GalaxiaDirectory.ConfigLegacy)! : Managed.LegacyDirectory;

        internal static string? LegacyPathFor(string fileName) =>
            GalaxiaNative.Active ? GalaxiaNative.LegacyConfigPath(fileName) : Managed.LegacyPathFor(fileName);

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static string Directory => _overrideDirectory ?? Path.Combine(
                ConfigRoot.Managed.Directory, Library.DataStore.HomeDirName, ConfigDirName, ProgramDirName);

            public static string PreviousDirectory =>
                Path.Combine(ConfigRoot.Managed.Directory, ConfigDirName, ProgramDirName);

            public static string For(string fileName) => Path.Combine(Directory, fileName);

            public static string For(string category, string fileName) =>
                Path.Combine(Directory, category, fileName);

            public static string LegacyDirectory => _overrideLegacyDirectory ?? Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), ProgramDirName);

            // A test that redirected the config directory but not the legacy one must not reach the developer's real config - see §1.4.
            public static string? LegacyPathFor(string fileName)
            {
                if (_overrideDirectory is not null && _overrideLegacyDirectory is null) return null;
                return Path.Combine(LegacyDirectory, fileName);
            }
        }
    }
}
