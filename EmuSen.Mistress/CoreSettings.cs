using System;
using EmuSen.Crystal.Client;

namespace EmuSen.Mistress
{
    // A console's core settings, the runtime's in consoles.json since SR12, read and written through the frontend API; Mistress's own keys stay in graphics.json - see EmuSen_Backend.md §14.12.
    internal static class CoreSettings
    {
        private static CrystalRuntimeLog? _runtimeLog;

        private static readonly Lazy<CrystalRuntime> Shared = new(() =>
        {
            var runtime = new CrystalRuntime();
            _runtimeLog = new CrystalRuntimeLog(runtime);
            return runtime;
        });

        // The process's one runtime of the frontend API.
        public static CrystalRuntime Runtime => Shared.Value;

        // The runtime's own lines waiting, written to the console, which the per-game log takes.
        public static void FileRuntimeLog()
        {
            _ = Runtime;
            _runtimeLog!.File();
        }

        // A core's setting as it applies: the stored value, else the schema's default; null for a key the console has not.
        public static string? Value(string console, string key)
        {
            try { return Runtime.Setting(console, key); }
            catch (CrystalException) { return null; }
        }

        // Stored by the runtime, which checks it against the console's schema and applies it to a running session.
        public static void Set(string console, string key, string? value) => Runtime.SetSetting(console, key, value);
    }
}
