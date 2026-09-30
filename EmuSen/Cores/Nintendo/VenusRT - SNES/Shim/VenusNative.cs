using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.VenusRT
{
    // VenusRT's Rust library on the common native interface; not an engine choice until stage 6 - see VenusRT_Native.md §2.
    public static class VenusNative
    {
        public const ushort CoreVersion = 1;
        public const string Variable = "EMUSEN_VENUS_NATIVE";

        // Stage 0 claims no optional export.
        public const ulong RequiredCapabilities = 0;

        public static readonly NativeCoreLibrary Library = NativeCoreLibrary.Common("venusrt", Variable, CoreVersion, RequiredCapabilities);

        public static readonly NativeInterface Api = new(Library);

        public static bool Available => Library.Available && Api.Complete;

        public static string Report => Library.Available && !Api.Complete ? $"{Library.FileName} lacks a required export" : Library.Report;
    }
}
