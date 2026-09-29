using System;
using System.IO;
using System.Runtime.InteropServices;

namespace EmuSen.Cores.Native
{
    // One Rust core's library, loaded once on first use and refused unless its interface version is this build's - see EmuSen_NativeCores.md §4.1.
    public sealed unsafe class NativeCoreLibrary
    {
        private readonly Lazy<(nint Handle, string Report)> _library;

        // The crate names the file; the exports and the crash log's stem are today's per-core names until the common interface.
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
            if (version != InterfaceVersion) return (0, $"{FileName} speaks interface {version}, this build {InterfaceVersion}");

            if (NativeLibrary.TryGetExport(handle, CrashLogExport, out nint crashExport))
            {
                string log = Path.Combine(EmuSen.Galaxia.Library.DataStore.Logs, $"{CrashLogStem}_{Environment.ProcessId}.txt");
                nint text = Marshal.StringToCoTaskMemUTF8(log);
                ((delegate* unmanaged<nint, void>)crashExport)(text);
            }
            return (handle, $"{path}, interface {version}");
        }

        // An export by name, or zero when the library is not in use; callers fall back to C# on zero.
        public nint Export(string name) =>
            Available && NativeLibrary.TryGetExport(_library.Value.Handle, name, out nint export) ? export : 0;
    }
}
