using System;
using System.IO;

namespace EmuSen.Nehellania.Settings
{
    // Redirectable config root for this frontend - see EmuSen_Settings_Reference.md §4.1.
    public static class SettingsPaths
    {
        private static string? _overrideDirectory;

        // Tests only; null restores the real per-user location.
        public static string? OverrideDirectory
        {
            get => _overrideDirectory;
            set => _overrideDirectory = value;
        }

        public static string Directory => _overrideDirectory ?? Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen");

        public static string For(string fileName) => Path.Combine(Directory, fileName);
    }
}
