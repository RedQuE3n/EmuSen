using System;
using System.IO;
using System.Runtime.InteropServices;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT's Rust library, loaded once; absent, refused or turned off, the Game Boy runs on the C# Mercury - see Mercury_Native.md §2.3.
    public static unsafe class MercuryNative
    {
        public const uint InterfaceVersion = 3;
        public const string Variable = "EMUSEN_MERCURY_NATIVE";

        private static readonly Lazy<(nint Handle, string Report)> Library = new(Load);

        public static bool Available => Library.Value.Handle != 0;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Value.Report;

        private static string FileName =>
            OperatingSystem.IsWindows() ? "mercuryrt.dll" : OperatingSystem.IsMacOS() ? "libmercuryrt.dylib" : "libmercuryrt.so";

        private static (nint, string) Load()
        {
            if (Environment.GetEnvironmentVariable(Variable) == "0") return (0, $"turned off by {Variable}=0");

            string path = Path.Combine(AppContext.BaseDirectory, FileName);
            if (!NativeLibrary.TryLoad(path, out nint handle)) return (0, $"{FileName} not found beside the assemblies");

            if (!NativeLibrary.TryGetExport(handle, "mercury_interface_version", out nint versionExport))
                return (0, $"{FileName} has no interface version");
            uint version = ((delegate* unmanaged<uint>)versionExport)();
            if (version != InterfaceVersion) return (0, $"{FileName} speaks interface {version}, this build {InterfaceVersion}");

            if (NativeLibrary.TryGetExport(handle, "mercury_set_crash_log", out nint crashExport))
            {
                string log = Path.Combine(EmuSen.Galaxia.Library.DataStore.Logs, $"mercuryrt_crash_{Environment.ProcessId}.txt");
                nint text = Marshal.StringToCoTaskMemUTF8(log);
                ((delegate* unmanaged<nint, void>)crashExport)(text);
            }
            return (handle, $"{path}, interface {version}");
        }

        // An export by name, or zero when the library is not in use.
        public static nint Export(string name) =>
            Available && NativeLibrary.TryGetExport(Library.Value.Handle, name, out nint export) ? export : 0;
    }
}
