using System.Runtime.InteropServices;

namespace EmuSen.Crystal.Client
{
    // The e_sqlite3 the C# loads, lent to the platform library so the process holds one SQLite - see EmuSen_RustPlatform.md Q25 and EmuSen_Backend.md §29.5.
    internal static class SqliteLend
    {
        private static readonly Lazy<(nint Handle, string Problem)> Loaded = new(Load);

        // The loader's handle, or zero with the reason in Problem.
        public static nint Handle => Loaded.Value.Handle;

        public static string Problem => Loaded.Value.Problem;

        private static (nint, string) Load()
        {
            try
            {
                SQLitePCL.Batteries_V2.Init();
                return NativeLibrary.TryLoad(SQLitePCL.raw.GetNativeLibraryName(), typeof(SQLitePCL.SQLite3Provider_e_sqlite3).Assembly, null, out nint handle)
                    ? (handle, "")
                    : (0, "SQLite could not be found as the C# loads it");
            }
            catch (Exception e) when (e is DllNotFoundException or FileNotFoundException or TypeInitializationException or InvalidOperationException)
            {
                return (0, $"SQLite could not be lent ({e.GetType().Name}: {e.Message.Split('\n')[0]})");
            }
        }
    }
}
