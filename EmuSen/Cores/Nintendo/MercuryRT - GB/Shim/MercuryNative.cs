using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's Rust library, loaded once through each interface: the v1 one its engine runs on and the pre-stable one its machine tests use; absent, refused or turned off, the Game Boy runs on the C# Mercury - see Mercury_Native.md §8.7, EmuSen_CoreAPI.md §26.
    public static class MercuryNative
    {
        // MercuryRT's half of the interface version: 5, the first on the common interface; 1 to 4 were its own mercury_interface_version.
        public const ushort CoreVersion = 5;
        public const string Variable = "EMUSEN_MERCURY_NATIVE";

        // What the pre-stable machine needs of the library: the channel mutes, the Game Genie patches and the debugger's hooks.
        public const ulong RequiredCapabilities = NativeInterface.Mutes | NativeInterface.RomPatches | NativeInterface.Debug | NativeInterface.DebugStack;

        public static readonly NativeCoreLibrary Library = NativeCoreLibrary.Common("mercuryrt", Variable, CoreVersion, RequiredCapabilities);

        public static readonly NativeInterface Api = new(Library);

        // The pre-stable machine's library, which MercuryMachine and its tests still load until that interface retires - see EmuSen_CoreAPI.md §13.5.
        public static bool Available => Library.Available && Api.Complete;

        // The library through the core ABI v1, which MercuryRtCore runs on, with the extension that takes C#'s mixer coefficients - see EmuSen_CoreAPI.md §26.
        public static readonly PortLibrary Engine = new("mercuryrt", Variable, CoreInterface.CapMutes | CoreInterface.CapRomPatches | CoreInterface.CapDebug | CoreInterface.CapDebugStack, MercuryRtCore.SampleRateExport);

        // Why the engine's library is or is not in use, for the engine notice, a log line or a test's message.
        public static string Report => Engine.Report;

        // An extension export by name, or zero when the library is not in use.
        public static nint Export(string name) => Library.Export(name);
    }
}
