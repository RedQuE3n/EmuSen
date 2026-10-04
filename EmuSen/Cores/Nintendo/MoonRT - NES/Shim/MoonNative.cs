using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT's Rust library, loaded once through each interface: the v1 one its engine runs on and the pre-stable one its machine tests use; absent, refused or turned off, the NES runs on the C# Moon - see Moon_Native.md §8.3, EmuSen_CoreAPI.md §26.
    public static class MoonNative
    {
        // MoonRT's half of the interface version: 4 with the debug exports, 3 before them; 1 and 2 were its own moon_interface_version.
        public const ushort CoreVersion = 4;
        public const string Variable = "EMUSEN_MOON_NATIVE";

        // What the pre-stable machine needs of the library: the RESET button, the channel mutes, the Game Genie patches and the debugger's hooks.
        public const ulong RequiredCapabilities = NativeInterface.Reset | NativeInterface.Mutes | NativeInterface.RomPatches | NativeInterface.Debug | NativeInterface.DebugStack;

        public static readonly NativeCoreLibrary Library = NativeCoreLibrary.Common("moonrt", Variable, CoreVersion, RequiredCapabilities);

        public static readonly NativeInterface Api = new(Library);

        // The pre-stable machine's library, which MoonMachine and its tests still load until that interface retires - see EmuSen_CoreAPI.md §13.5.
        public static bool Available => Library.Available && Api.Complete;

        // The library through the core ABI v1, which MoonRtCore runs on - see EmuSen_CoreAPI.md §26.
        public static readonly PortLibrary Engine = new("moonrt", Variable, CoreInterface.CapReset | CoreInterface.CapMutes | CoreInterface.CapRomPatches | CoreInterface.CapDebug | CoreInterface.CapDebugStack);

        // Why the engine's library is or is not in use, for the engine notice, a log line or a test's message.
        public static string Report => Engine.Report;

        // An extension export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
