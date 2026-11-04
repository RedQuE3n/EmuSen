using System;
using System.Collections.Concurrent;

namespace EmuSen.Galaxia.Native
{
    // A component's switch variable after its C# half was deleted: a leftover 0 is ignored, said once in one line - see EmuSen_Backend.md §40.
    public static class RetiredSwitch
    {
        private static readonly ConcurrentDictionary<string, byte> Said = new(StringComparer.Ordinal);

        public static void Note(string variable, string component)
        {
            if (Environment.GetEnvironmentVariable(variable) != "0" || !Said.TryAdd(variable, 0)) return;
            string message = $"{variable}=0 is ignored: {component} has had no C# implementation since 2026-11-04 and runs on {PlatformLibrary.FileName}.";
            ConfigDiagnostics.Report(message);
            Library.ErrorLog.Warning("platform", message);
        }
    }
}
