using System;
using System.IO;
using System.Runtime.InteropServices;

namespace EmuSen.Galaxia.Native
{
    // emusen_platform, loaded once on first use and refused unless its interface version is this build's - see EmuSen_RustPlatform.md §3.9.
    public static unsafe class PlatformLibrary
    {
        // Matched exactly against emusen_platform_abi_version - see EmuSen_RustPlatform.md §3.2.
        public const uint InterfaceVersion = 42;

        public const string Name = "emusen_platform";

        private static readonly Lazy<(nint Handle, string Report)> Library = new(() => Open(AppContext.BaseDirectory));

        // Asking loads the library, and Galaxia with it.
        public static bool Available => Library.Value.Handle != 0;

        // Why the library is or is not in use, for a log line or a test's message.
        public static string Report => Library.Value.Report;

        // An export by name for another component's half, Endymion's and later Serenity's; zero when the library is not in use or lacks it.
        public static nint Export(string name) =>
            Available && NativeLibrary.TryGetExport(Library.Value.Handle, name, out nint export) ? export : 0;

        // The library's last error as words for this thread, empty when the library or the export is missing.
        public static string LastError()
        {
            nint export = Export("emusen_platform_last_error");
            if (export == 0) return "";
            var read = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
            long n = read(null, 0);
            var words = new byte[Math.Max(0, n)];
            fixed (byte* w = words) read(w, (nuint)words.Length);
            return System.Text.Encoding.UTF8.GetString(words);
        }

        // The diagnostics the library queued on this thread, each reported through ConfigDiagnostics, for another component's half.
        public static void ReportDiagnostics() => GalaxiaNative.Drain();

        public static string FileName =>
            OperatingSystem.IsWindows() ? $"{Name}.dll" : OperatingSystem.IsMacOS() ? $"lib{Name}.dylib" : $"lib{Name}.so";

        // The library from a folder; a test names one without it to see the fallback.
        internal static (nint Handle, string Report) Open(string directory)
        {
            try { return Load(directory); }
            catch (Exception fault) { return (0, $"{FileName} could not be loaded: {fault.GetType().Name}: {fault.Message}"); }
        }

        private static (nint, string) Load(string directory)
        {
            string path = Path.Combine(directory, FileName);
            if (!NativeLibrary.TryLoad(path, out nint handle)) return (0, $"{FileName} not found beside the assemblies");

            if (!NativeLibrary.TryGetExport(handle, "emusen_platform_abi_version", out nint versionExport))
                return (0, $"{FileName} has no interface version");
            uint version = ((delegate* unmanaged[Cdecl]<uint>)versionExport)();
            if (version != InterfaceVersion) return (0, $"{FileName} speaks interface {version}, this build {InterfaceVersion}");

            if (GalaxiaNative.Attach(handle) is { } missing) return (0, $"{FileName} has no {missing}");

            GalaxiaNative.SetCrashLog(Path.Combine(GalaxiaNative.Directory(GalaxiaDirectory.Logs)!, $"{Name}_crash_{Environment.ProcessId}.txt"));
            // Galaxia is attached by now, so the line a leftover 0 earns goes through it; a failure to say it never refuses the library.
            try { RetiredSwitch.Note(GalaxiaNative.Variable, "Galaxia"); } catch (Exception) { }
            return (handle, $"{path}, interface {version}");
        }
    }
}
