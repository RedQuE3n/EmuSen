using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.Mars.Native
{
    // MarsRT's Rust library, loaded once; when it is absent, refused or turned off, every component runs its C# twin - see Mars_Native.md §1.
    public static class MarsNative
    {
        public const uint InterfaceVersion = 10;
        public const string Variable = "EMUSEN_MARS_NATIVE";

        // The crash log keeps its name, native_crash_<pid>, until MarsRT moves to the common interface - see EmuSen_NativeCores.md §4.1.
        public static readonly NativeCoreLibrary Library = new("marsrt", Variable, InterfaceVersion, "emusen_native_interface_version", "emusen_native_set_crash_log", "native_crash");

        public static bool Available => Library.Available;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Report;

        // An export by name, or zero when the library is not in use; callers fall back to C# on zero.
        public static nint Export(string name) => Library.Export(name);
    }
}
