using System.Runtime.InteropServices;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Contract
{
    // Which implementation serves the API, chosen once: the platform library's exports when EMUSEN_CRYSTAL_NATIVE is 1 and it has all of them, else the C# of step B2 - see EmuSen_Backend.md §3.9 and §14.
    public static unsafe partial class Crystal
    {
        internal const string Variable = "EMUSEN_CRYSTAL_NATIVE";

        // True when the platform library serves the API; false on the C# implementation.
        internal static bool Native { get; private set; }

        // Why the implementation is the one it is, for a log line or a test's message.
        internal static string Report { get; private set; } = "";

        // Declared after the two above, whose initialisers run first.
        private static readonly CrystalExports* Table = Choose();

        private static CrystalExports* Choose()
        {
            var table = (CrystalExports*)NativeMemory.AllocZeroed((nuint)sizeof(CrystalExports));
            if (Environment.GetEnvironmentVariable(Variable) == "1")
            {
                string? missing = Fill(table, PlatformLibrary.Export);
                if (missing is null && table->abi_version() >> 16 == CrystalAbi.Major)
                {
                    Native = true;
                    Report = $"{PlatformLibrary.Report}, frontend API {table->abi_version() >> 16}.{table->abi_version() & 0xFFFF}";
                    return table;
                }
                Report = missing is null ? $"{PlatformLibrary.FileName} speaks another major of the frontend API" : $"{PlatformLibrary.FileName} has no {missing} ({PlatformLibrary.Report})";
                PlatformSwitch.Announce($"{PlatformLibrary.FileName} does not serve the frontend API ({Report}); it runs on its C# implementation.");
            }
            string? absent = Fill(table, EmuSen.Crystal.Managed.ManagedCrystal.Export);
            if (absent is not null) throw new InvalidOperationException($"the C# implementation has no {absent}");
            Report = string.IsNullOrEmpty(Report) ? $"the C# implementation ({Variable} is not 1)" : Report;
            return table;
        }

        // Every field of the table from an export of its name; the first name missing, or null when all were found.
        private static string? Fill(CrystalExports* table, Func<string, nint> export)
        {
            var fields = typeof(CrystalExports).GetFields();
            nint* slots = (nint*)table;
            for (int i = 0; i < fields.Length; i++)
            {
                string name = "emusen_crystal_" + fields[i].Name;
                nint found = export(name);
                if (found == 0) return name;
                slots[(int)(Marshal.OffsetOf<CrystalExports>(fields[i].Name) / sizeof(nint))] = found;
            }
            return null;
        }
    }
}
