using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT's Rust library on the common native interface, loaded once; absent, refused or turned off, the NES runs on the C# Moon - see Moon_Native.md §8.3.
    public static class MoonNative
    {
        // MoonRT's half of the interface version; 1 and 2 were its own moon_interface_version.
        public const ushort CoreVersion = 3;
        public const string Variable = "EMUSEN_MOON_NATIVE";

        // What MoonRtCore needs of the library: the RESET button, the channel mutes and the Game Genie patches.
        public const ulong RequiredCapabilities = NativeInterface.Reset | NativeInterface.Mutes | NativeInterface.RomPatches;

        public static readonly NativeCoreLibrary Library = NativeCoreLibrary.Common("moonrt", Variable, CoreVersion, RequiredCapabilities);

        public static readonly NativeInterface Api = new(Library);

        public static bool Available => Library.Available && Api.Complete;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Available && !Api.Complete ? $"{Library.FileName} lacks a required export" : Library.Report;

        // An extension export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
