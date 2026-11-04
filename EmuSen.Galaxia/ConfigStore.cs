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
        public static string Directory => GalaxiaNative.Directory(GalaxiaDirectory.Config)!;

        // Where config sat before it moved under home - see EmuSen_Galaxia.md §3.2.
        public static string PreviousDirectory => GalaxiaNative.Directory(GalaxiaDirectory.ConfigPrevious)!;

        public static string For(string fileName) => GalaxiaNative.ConfigPath(null, fileName);

        // One subdirectory per category, for config there can be many of - see §1.2.
        public static string For(string category, string fileName) =>
            GalaxiaNative.ConfigPath(category, fileName);

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
        public static string LegacyDirectory => GalaxiaNative.Directory(GalaxiaDirectory.ConfigLegacy)!;

        // Where a config file sat before Galaxia, or null when config is redirected and the legacy directory is not.
        public static string? LegacyPathFor(string fileName) =>
            GalaxiaNative.LegacyConfigPath(fileName);
    }
}
