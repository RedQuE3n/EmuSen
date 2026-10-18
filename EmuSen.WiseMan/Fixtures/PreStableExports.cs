using System.Runtime.InteropServices;

namespace EmuSen.WiseMan.Fixtures
{
    // The pre-stable interface's entry points, deleted with B4, which no core library may still define - see EmuSen_Backend.md §25.
    public static class PreStableExports
    {
        public static readonly string[] Names = { "emusen_native_interface_version", "emusen_native_capabilities", "emusen_native_create", "emusen_native_advance", "emusen_native_free" };

        public static void AssertNone(string libraryPath)
        {
            nint handle = NativeLibrary.Load(libraryPath);
            try
            {
                foreach (string name in Names) Assert.False(NativeLibrary.TryGetExport(handle, name, out _), name);
            }
            finally
            {
                NativeLibrary.Free(handle);
            }
        }
    }
}
