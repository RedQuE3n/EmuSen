using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's Rust library, loaded once; absent, refused or turned off, the Game Boy runs on the C# Mercury - see Mercury_Native.md §2.3.
    public static class MercuryNative
    {
        public const uint InterfaceVersion = 4;
        public const string Variable = "EMUSEN_MERCURY_NATIVE";

        public static readonly NativeCoreLibrary Library = new("mercuryrt", Variable, InterfaceVersion, "mercury_interface_version", "mercury_set_crash_log", "mercuryrt_crash");

        public static bool Available => Library.Available;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Report;

        // An export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
