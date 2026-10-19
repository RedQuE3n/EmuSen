using System.Reflection;
using System.Runtime.InteropServices;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Contract
{
    // Which implementation serves the API, chosen once: the platform library's exports when asked for and it has all of them, else the C# of step B2 - see EmuSen_Backend.md §3.9, §14 and §29.27.
    public static unsafe partial class Crystal
    {
        internal const string Variable = "EMUSEN_CRYSTAL_NATIVE";

        // The program's own default, [assembly: AssemblyMetadata("EmuSen.CrystalNative", "1")] on its entry assembly - see EmuSen_Surfaces.md §6.2.
        internal const string MetadataKey = "EmuSen.CrystalNative";

        // True when the platform library serves the API; false on the C# implementation.
        internal static bool Native { get; private set; }

        // Why the implementation is the one it is, for a log line or a test's message.
        internal static string Report { get; private set; } = "";

        // Declared after the two above, whose initialisers run first.
        private static readonly CrystalExports* Table = Choose();

        private static CrystalExports* Choose()
        {
            var table = (CrystalExports*)NativeMemory.AllocZeroed((nuint)sizeof(CrystalExports));
            string? variable = Environment.GetEnvironmentVariable(Variable);
            if (Asked(variable, Assembly.GetEntryAssembly()))
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
            Report = string.IsNullOrEmpty(Report) ? $"the C# implementation ({(string.IsNullOrEmpty(variable) ? $"{Variable} is unset and the program's default is the C#" : $"{Variable} is {variable}")})" : Report;
            return table;
        }

        // Whether the Rust is asked for: the variable when it is set (1 the Rust, anything else the C#), else the program's default.
        internal static bool Asked(string? variable, Assembly? program) => string.IsNullOrEmpty(variable) ? DefaultsToRust(program) : variable == "1";

        // Whether a program's assembly names the Rust as its default.
        internal static bool DefaultsToRust(Assembly? program) =>
            program?.GetCustomAttributes<AssemblyMetadataAttribute>().Any(a => a.Key == MetadataKey && a.Value == "1") == true;

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
