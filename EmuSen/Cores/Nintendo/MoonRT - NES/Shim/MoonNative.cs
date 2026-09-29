using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT's Rust library, loaded once; absent, refused or turned off, the NES runs on the C# Moon - see Moon_Native.md §2.3.
    public static class MoonNative
    {
        public const uint InterfaceVersion = 2;
        public const string Variable = "EMUSEN_MOON_NATIVE";

        public static readonly NativeCoreLibrary Library = new("moonrt", Variable, InterfaceVersion, "moon_interface_version", "moon_set_crash_log", "moonrt_crash");

        public static bool Available => Library.Available;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Report;

        // An export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
