using System.Runtime.InteropServices;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Contract
{
    // The platform library's exports of the frontend API, bound once; the C# implementation was deleted on 2026-11-04 - see EmuSen_Backend.md §3.9 and §40.8.
    public static unsafe partial class Crystal
    {
        // The switch of the deleted C# implementation, now ignored with one line wherever it is set.
        internal const string Variable = "EMUSEN_CRYSTAL_NATIVE";

        // The library and the API's version that serve the program, for a log line or a test's message.
        internal static string Report { get; private set; } = "";

        // Declared after the one above, whose initialiser runs first.
        private static readonly CrystalExports* Table = Choose();

        private static CrystalExports* Choose()
        {
            if (Ignored(Environment.GetEnvironmentVariable(Variable)) is { } line) PlatformSwitch.Announce(line);
            var table = (CrystalExports*)NativeMemory.AllocZeroed((nuint)sizeof(CrystalExports));
            string? missing = Fill(table, PlatformLibrary.Export);
            if (missing is not null) throw new InvalidOperationException($"{PlatformLibrary.FileName} does not serve the frontend API: it has no {missing} ({PlatformLibrary.Report})");
            uint version = table->abi_version();
            if (version >> 16 != CrystalAbi.Major) throw new InvalidOperationException($"{PlatformLibrary.FileName} speaks major {version >> 16} of the frontend API, not {CrystalAbi.Major}");
            Report = $"{PlatformLibrary.Report}, frontend API {version >> 16}.{version & 0xFFFF}";
            return table;
        }

        // The one line a leftover value of the variable is ignored with, or null where it asks for nothing else.
        internal static string? Ignored(string? variable) =>
            string.IsNullOrEmpty(variable) || variable == "1" ? null : $"{Variable}={variable} is ignored: the frontend API's C# implementation was deleted, and {PlatformLibrary.FileName} serves it.";

        // Every field of the table from an export of its name, those found filled; the first name missing, or null when all were found.
        internal static string? Fill(CrystalExports* table, Func<string, nint> export)
        {
            var fields = typeof(CrystalExports).GetFields();
            nint* slots = (nint*)table;
            string? missing = null;
            for (int i = 0; i < fields.Length; i++)
            {
                string name = "emusen_crystal_" + fields[i].Name;
                nint found = export(name);
                missing ??= found == 0 ? name : null;
                slots[(int)(Marshal.OffsetOf<CrystalExports>(fields[i].Name) / sizeof(nint))] = found;
            }
            return missing;
        }
    }
}
