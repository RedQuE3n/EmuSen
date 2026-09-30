using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's Rust library on the common native interface, loaded once; absent, refused or turned off, the Game Boy runs on the C# Mercury - see Mercury_Native.md §8.7.
    public static class MercuryNative
    {
        // MercuryRT's half of the interface version: 5, the first on the common interface; 1 to 4 were its own mercury_interface_version.
        public const ushort CoreVersion = 5;
        public const string Variable = "EMUSEN_MERCURY_NATIVE";

        // What MercuryRtCore needs of the library: the channel mutes, the Game Genie patches and the debugger's hooks.
        public const ulong RequiredCapabilities = NativeInterface.Mutes | NativeInterface.RomPatches | NativeInterface.Debug | NativeInterface.DebugStack;

        public static readonly NativeCoreLibrary Library = NativeCoreLibrary.Common("mercuryrt", Variable, CoreVersion, RequiredCapabilities);

        public static readonly NativeInterface Api = new(Library);

        public static bool Available => Library.Available && Api.Complete;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Available && !Api.Complete ? $"{Library.FileName} lacks a required export" : Library.Report;

        // An extension export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
