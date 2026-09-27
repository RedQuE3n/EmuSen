using System;
using System.IO;
using System.Runtime.CompilerServices;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Galaxia
{
    // The suite's errors go to its own scratch folder, never the player's log folder - see EmuSen_Settings_Reference.md §4.70.
    internal static class ErrorLogIsolation
    {
        [ModuleInitializer]
        internal static void Isolate() => ErrorLog.DirectoryOverride = Path.Combine(Path.GetTempPath(), "EmuSenErrorLog", Environment.ProcessId.ToString());
    }
}
