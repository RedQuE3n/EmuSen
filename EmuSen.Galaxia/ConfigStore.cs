using System;
using System.IO;

namespace EmuSen.Galaxia
{
    // The single answer to "where does a config file go" - see
    // EmuSen_Config_Reference.md §1.
    public static class ConfigStore
    {
        public const string ConfigDirName = "etc";
        public const string ProgramDirName = "EmuSen";

        // Tests only; null restores the real location. Deliberately separate
        // from ConfigRoot, which stays fixed - see EmuSen_Config_Reference.md §1.3.
        public static string? OverrideDirectory { get; set; }

        // Under home so /etc/EmuSen stays inside the shell root; built from ConfigRoot, not DataStore - see EmuSen_Galaxia.md §3.1.
        public static string Directory => OverrideDirectory ?? Path.Combine(
            ConfigRoot.Directory, Library.DataStore.HomeDirName, ConfigDirName, ProgramDirName);

        // Where config sat before it moved under home - see EmuSen_Galaxia.md §3.2.
        public static string PreviousDirectory =>
            Path.Combine(ConfigRoot.Directory, ConfigDirName, ProgramDirName);

        public static string For(string fileName) => Path.Combine(Directory, fileName);

        // One subdirectory per category, for config there can be many of - see §1.2.
        public static string For(string category, string fileName) =>
            Path.Combine(Directory, category, fileName);

        // Tests only, as above.
        public static string? OverrideLegacyDirectory { get; set; }

        // Pre-Galaxia location, read once to migrate out of - see §1.4.
        public static string LegacyDirectory => OverrideLegacyDirectory ?? Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), ProgramDirName);

        internal static string? LegacyPathFor(string fileName)
        {
            // A test that redirected the config directory but not the legacy
            // one must not reach the developer's real config - see §1.4.
            if (OverrideDirectory is not null && OverrideLegacyDirectory is null) return null;
            return Path.Combine(LegacyDirectory, fileName);
        }
    }
}
