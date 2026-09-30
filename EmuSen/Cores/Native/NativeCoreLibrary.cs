using System;
using System.IO;
using System.Runtime.InteropServices;

namespace EmuSen.Cores.Native
{
    // One Rust core's library, loaded once on first use and refused unless its interface version is this build's - see EmuSen_NativeCores.md §4.1.
    public sealed unsafe class NativeCoreLibrary
    {
        private readonly Lazy<(nint Handle, string Report)> _library;

        // A per-core ABI's library, MercuryRT's and MarsRT's until their steps 4 and 5: its own version and crash-log exports.
        public NativeCoreLibrary(string crate, string variable, uint interfaceVersion, string versionExport, string crashLogExport, string crashLogStem)
        {
            Crate = crate;
            Variable = variable;
            InterfaceVersion = interfaceVersion;
            VersionExport = versionExport;
            CrashLogExport = crashLogExport;
            CrashLogStem = crashLogStem;
            _library = new(Load);
        }

        // A library on the common interface: the fixed names, (common << 16) | core matched exactly, and the capabilities its engine needs - see EmuSen_NativeCores.md §3.2.
        public static NativeCoreLibrary Common(string crate, string variable, ushort coreVersion, ulong requiredCapabilities) =>
            new(crate, variable, NativeInterface.Version(coreVersion), NativeInterface.VersionExport, NativeInterface.CrashLogExport, crate + "_crash")
            {
                IsCommon = true,
                RequiredCapabilities = requiredCapabilities,
            };

        public bool IsCommon { get; private init; }
        public ulong RequiredCapabilities { get; private init; }

        // The library's capability bits, zero until it loads or when it is not on the common interface.
        public ulong Capabilities => _library.Value.Handle != 0 && IsCommon ? _capabilities : 0;
        private ulong _capabilities;

        public string Crate { get; }
        public string Variable { get; }
        public uint InterfaceVersion { get; }
        public string VersionExport { get; }
        public string CrashLogExport { get; }
        public string CrashLogStem { get; }

        public bool Available => _library.Value.Handle != 0;

        // Why the library is or is not in use, for a log line or a test's message.
        public string Report => _library.Value.Report;

        public string FileName =>
            OperatingSystem.IsWindows() ? $"{Crate}.dll" : OperatingSystem.IsMacOS() ? $"lib{Crate}.dylib" : $"lib{Crate}.so";

        private (nint, string) Load()
        {
            if (Environment.GetEnvironmentVariable(Variable) == "0") return (0, $"turned off by {Variable}=0");

            string path = Path.Combine(AppContext.BaseDirectory, FileName);
            if (!NativeLibrary.TryLoad(path, out nint handle)) return (0, $"{FileName} not found beside the assemblies");

            if (!NativeLibrary.TryGetExport(handle, VersionExport, out nint versionExport))
                return (0, $"{FileName} has no interface version");
            uint version = ((delegate* unmanaged<uint>)versionExport)();
            if (version != InterfaceVersion)
                return (0, IsCommon
                    ? $"{FileName} speaks common interface {version >> 16} core {version & 0xFFFF}, this build common {InterfaceVersion >> 16} core {InterfaceVersion & 0xFFFF}"
                    : $"{FileName} speaks interface {version}, this build {InterfaceVersion}");

            if (IsCommon)
            {
                if (!NativeLibrary.TryGetExport(handle, NativeInterface.CapabilitiesExport, out nint capsExport)) return (0, $"{FileName} has no capabilities");
                _capabilities = ((delegate* unmanaged<ulong>)capsExport)();
                ulong missing = RequiredCapabilities & ~_capabilities;
                if (missing != 0) return (0, $"{FileName} lacks the capabilities {NativeInterface.Describe(missing)} its engine needs");
            }

            if (NativeLibrary.TryGetExport(handle, CrashLogExport, out nint crashExport))
            {
                string log = Path.Combine(EmuSen.Galaxia.Library.DataStore.Logs, $"{CrashLogStem}_{Environment.ProcessId}.txt");
                nint text = Marshal.StringToCoTaskMemUTF8(log);
                ((delegate* unmanaged<nint, void>)crashExport)(text);
            }
            return (handle, IsCommon ? $"{path}, common interface {version >> 16} core {version & 0xFFFF}" : $"{path}, interface {version}");
        }

        // An export by name, or zero when the library is not in use; callers fall back to C# on zero.
        public nint Export(string name) =>
            Available && NativeLibrary.TryGetExport(_library.Value.Handle, name, out nint export) ? export : 0;
    }
}
