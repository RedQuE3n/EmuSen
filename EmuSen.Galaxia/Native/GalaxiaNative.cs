using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace EmuSen.Galaxia.Native
{
    // The directories of the tree, numbered as emusen_platform.h's EMUSEN_GALAXIA_DIR_*.
    internal enum GalaxiaDirectory : uint
    {
        Root,
        Seed,
        Config,
        ConfigPrevious,
        ConfigLegacy,
        Home,
        Logs,
        Saves,
        SaveStates,
        Library,
        Screenshots,
        Artwork,
        Media,
        Firmware,
        Shaders,
        Themes,
        Cheats,
        Games,
        LegacyRoot,
    }

    // The redirects a test moves, as EMUSEN_GALAXIA_OVERRIDE_*.
    internal enum GalaxiaOverride : uint
    {
        Config,
        Data,
        Legacy,
    }

    // As EMUSEN_GALAXIA_SAVE_*.
    internal enum GalaxiaSave : uint
    {
        Sram,
        FlatSram,
        State,
        ResumeState,
        Picture,
    }

    // As EMUSEN_GALAXIA_MIGRATE_*.
    internal enum GalaxiaMigrate : uint
    {
        Own,
        OwnConfig,
        OwnSeed,
        Between,
        Tree,
        Seed,
    }

    // As EMUSEN_GALAXIA_PATH_*: the .NET path rules the library applies, asked one at a time by the parity tests.
    internal enum GalaxiaPathRule : uint
    {
        Combine,
        FileName,
        FileNameWithoutExtension,
        ChangeExtension,
        DirectoryName,
        TrimEndingSeparator,
        FullPath,
    }

    // As EMUSEN_GALAXIA_TEST_*.
    internal enum GalaxiaTestRule : uint
    {
        FileExists,
        DirectoryExists,
        IsBlank,
        EqualsIgnoreCase,
        EndsWithIgnoreCase,
    }

    // Galaxia's half of emusen_platform, and the switch that chooses it over the C# - see EmuSen_RustPlatform.md §3.9 and §10.2.
    internal static unsafe class GalaxiaNative
    {
        public const string Variable = "EMUSEN_GALAXIA_NATIVE";

        private const int Absent = -1030;
        private const int NotFound = -1031;
        private const int Access = -1032;

        // Held while an override moves and while the library first takes the overrides, so neither side misses a change.
        internal static readonly object Gate = new();

        private static readonly Lazy<bool> Chosen = new(Choose);
        private static volatile bool _attached;

        // State 1: the C# unless the variable asks for the library and the library loads.
        public static bool Active => Chosen.Value;

        private static bool Choose()
        {
            if (Environment.GetEnvironmentVariable(Variable) != "1") return false;
            if (PlatformLibrary.Available) return true;
            ConfigDiagnostics.Report($"{Variable}=1 was asked for and {PlatformLibrary.Report}; Galaxia runs on its C# implementation.");
            return false;
        }

        // The library itself, whichever the switch chose: what a parity test calls.
        public static bool Ready => PlatformLibrary.Available;

        private static delegate* unmanaged[Cdecl]<byte*, nuint, int> _setCrashLog;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, long> _lastError;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, long> _diagnostics;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int> _init;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, long> _directory;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, int> _setOverride;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long> _configPath;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _legacyConfigPath;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, int, byte*, nuint, byte*, nuint, long> _rootFor;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _bundleContentsFor;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, int, byte*, nuint, long> _seedDirectoryFor;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _macDataDirectoryFor;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _legacyRootFor;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int, byte*, nuint, long> _savePath;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _fileRead;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int> _fileWrite;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _romMd5;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, long> _migrate;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _remainingLibrary;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, byte*, nuint, long> _dotnetPath;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int> _dotnetTest;

        // Every export this class calls; a name the library lacks refuses the library - see EmuSen_RustPlatform.md §3.1.
        internal static readonly string[] Exports =
        {
            "emusen_platform_set_crash_log", "emusen_platform_last_error", "emusen_platform_diagnostics", "emusen_galaxia_init",
            "emusen_galaxia_directory", "emusen_galaxia_set_override", "emusen_galaxia_config_path", "emusen_galaxia_legacy_config_path",
            "emusen_galaxia_root_for", "emusen_galaxia_bundle_contents_for", "emusen_galaxia_seed_directory_for",
            "emusen_galaxia_mac_data_directory_for", "emusen_galaxia_legacy_root_for", "emusen_galaxia_save_path", "emusen_galaxia_file_read",
            "emusen_galaxia_file_write", "emusen_galaxia_rom_md5", "emusen_galaxia_migrate", "emusen_galaxia_remaining_library",
            "emusen_galaxia_dotnet_path", "emusen_galaxia_dotnet_test",
        };

        // Resolves the exports and hands the library what only the host knows; the name of a missing export, or null.
        internal static string? Attach(nint handle)
        {
            var found = new nint[Exports.Length];
            for (int i = 0; i < Exports.Length; i++)
                if (!NativeLibrary.TryGetExport(handle, Exports[i], out found[i])) return Exports[i];

            int next = 0;
            _setCrashLog = (delegate* unmanaged[Cdecl]<byte*, nuint, int>)found[next++];
            _lastError = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            _diagnostics = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            _init = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int>)found[next++];
            _directory = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, long>)found[next++];
            _setOverride = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, int>)found[next++];
            _configPath = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)found[next++];
            _legacyConfigPath = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _rootFor = (delegate* unmanaged[Cdecl]<byte*, nuint, int, byte*, nuint, byte*, nuint, long>)found[next++];
            _bundleContentsFor = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _seedDirectoryFor = (delegate* unmanaged[Cdecl]<byte*, nuint, int, byte*, nuint, long>)found[next++];
            _macDataDirectoryFor = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _legacyRootFor = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _savePath = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int, byte*, nuint, long>)found[next++];
            _fileRead = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _fileWrite = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int>)found[next++];
            _romMd5 = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _migrate = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, long>)found[next++];
            _remainingLibrary = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _dotnetPath = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, byte*, nuint, long>)found[next++];
            _dotnetTest = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int>)found[next++];

            // The program's directory and .NET's own application-data folder are passed, not found - see EmuSen_RustPlatform.md §5.5.
            Utf8 home = new(AppContext.BaseDirectory), data = new(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData));
            fixed (byte* h = home.Bytes, d = data.Bytes) _init(h, home.Length, d, data.Length);

            lock (Gate)
            {
                PushOverride(GalaxiaOverride.Config, ConfigStore.OverrideDirectory);
                PushOverride(GalaxiaOverride.Legacy, ConfigStore.OverrideLegacyDirectory);
                PushOverride(GalaxiaOverride.Data, EmuSen.Galaxia.Library.DataStore.OverrideDirectory);
                _attached = true;
            }
            return null;
        }

        // Called under Gate by each override's setter; the library hears it only once it is in the process.
        internal static void OverrideChanged(GalaxiaOverride which, string? directory)
        {
            if (_attached) PushOverride(which, directory);
        }

        private static void PushOverride(GalaxiaOverride which, string? directory)
        {
            Utf8 text = new(directory);
            fixed (byte* p = text.Bytes) Check(_setOverride((uint)which, p, text.Length));
        }

        internal static void SetCrashLog(string path)
        {
            Utf8 text = new(path);
            fixed (byte* p = text.Bytes) _setCrashLog(p, text.Length);
        }

        // A string as the interface takes it: UTF-8 with its length, null told apart from empty - see EmuSen_RustPlatform.md §3.4.
        private readonly struct Utf8
        {
            private static readonly UTF8Encoding Strict = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);
            private static readonly byte[] Empty = new byte[1];

            public readonly byte[]? Bytes;
            public readonly nuint Length;

            public Utf8(string? text)
            {
                if (text is null) return;
                Bytes = text.Length == 0 ? Empty : Strict.GetBytes(text);
                Length = text.Length == 0 ? 0 : (nuint)Bytes.Length;
            }
        }

        private delegate long Query(byte* output, nuint length);

        // The length-query idiom, once: a small buffer first, so a path costs one call. Null for ABSENT.
        private static string? Text(Query query)
        {
            const int Small = 512;
            byte* small = stackalloc byte[Small];
            long length = query(small, Small);
            if (length == Absent) return null;
            if (length < 0) throw Fault((int)length);
            if (length <= Small) return Encoding.UTF8.GetString(small, (int)length);

            while (true)
            {
                byte[] large = new byte[length];
                fixed (byte* p = large) length = query(p, (nuint)large.Length);
                if (length == Absent) return null;
                if (length < 0) throw Fault((int)length);
                if (length <= large.Length) return Encoding.UTF8.GetString(large, 0, (int)length);
            }
        }

        private static void Check(int status)
        {
            if (status < 0) throw Fault(status);
        }

        // A failing status as the exception the C# would have thrown for it, with the library's words.
        private static Exception Fault(int status)
        {
            string words = Text((o, n) => _lastError(o, n)) ?? "";
            return status switch
            {
                NotFound => new FileNotFoundException(words),
                Access => new UnauthorizedAccessException(words),
                -1 or -5 or -1033 => new ArgumentException(words),
                _ => new IOException(words),
            };
        }

        // What the library queued where the C# calls ConfigDiagnostics.Report, reported here in its order - see EmuSen_RustPlatform.md §3.5.
        private static void Drain()
        {
            string? waiting = Text((o, n) => _diagnostics(o, n));
            if (string.IsNullOrEmpty(waiting)) return;
            foreach (string message in waiting.Split('\0', StringSplitOptions.RemoveEmptyEntries)) ConfigDiagnostics.Report(message);
        }

        public static string? Directory(GalaxiaDirectory which) => Text((o, n) => _directory((uint)which, o, n));

        public static string ConfigPath(string? category, string fileName)
        {
            Utf8 c = new(category), f = new(fileName);
            return Text((o, n) => { fixed (byte* pc = c.Bytes, pf = f.Bytes) return _configPath(pc, c.Length, pf, f.Length, o, n); })!;
        }

        public static string? LegacyConfigPath(string fileName)
        {
            Utf8 f = new(fileName);
            return Text((o, n) => { fixed (byte* pf = f.Bytes) return _legacyConfigPath(pf, f.Length, o, n); });
        }

        public static string RootFor(string baseDirectory, bool macOS, string userHome)
        {
            Utf8 b = new(baseDirectory), h = new(userHome);
            return Text((o, n) => { fixed (byte* pb = b.Bytes, ph = h.Bytes) return _rootFor(pb, b.Length, macOS ? 1 : 0, ph, h.Length, o, n); })!;
        }

        public static string? BundleContentsFor(string baseDirectory)
        {
            Utf8 b = new(baseDirectory);
            return Text((o, n) => { fixed (byte* pb = b.Bytes) return _bundleContentsFor(pb, b.Length, o, n); });
        }

        public static string? SeedDirectoryFor(string baseDirectory, bool macOS)
        {
            Utf8 b = new(baseDirectory);
            return Text((o, n) => { fixed (byte* pb = b.Bytes) return _seedDirectoryFor(pb, b.Length, macOS ? 1 : 0, o, n); });
        }

        public static string MacDataDirectoryFor(string userHome)
        {
            Utf8 h = new(userHome);
            return Text((o, n) => { fixed (byte* ph = h.Bytes) return _macDataDirectoryFor(ph, h.Length, o, n); })!;
        }

        public static string LegacyRootFor(string configRoot)
        {
            Utf8 r = new(configRoot);
            return Text((o, n) => { fixed (byte* pr = r.Bytes) return _legacyRootFor(pr, r.Length, o, n); })!;
        }

        public static string SavePath(GalaxiaSave kind, string romPath, string? extra = null, int slot = 0)
        {
            Utf8 r = new(romPath), e = new(extra);
            return Text((o, n) => { fixed (byte* pr = r.Bytes, pe = e.Bytes) return _savePath((uint)kind, pr, r.Length, pe, e.Length, slot, o, n); })!;
        }

        // Null for no file, and for one that would not read, which the library has said why.
        public static byte[]? FileRead(string path)
        {
            Utf8 file = new(path);
            fixed (byte* p = file.Bytes)
            {
                long length = _fileRead(p, file.Length, null, 0);
                while (length >= 0)
                {
                    // Room for a little more than the length reported, so a file whose length reads as zero is still read.
                    byte[] buffer = new byte[Math.Max(length, 64)];
                    long read;
                    fixed (byte* b = buffer) read = _fileRead(p, file.Length, b, (nuint)buffer.Length);
                    if (read >= 0 && read <= buffer.Length) return read == buffer.Length ? buffer : buffer.AsSpan(0, (int)read).ToArray();
                    length = read;
                }
                if (length != Absent) Drain();
                return null;
            }
        }

        public static bool FileWrite(string path, byte[] contents)
        {
            Utf8 file = new(path);
            int status;
            fixed (byte* p = file.Bytes, c = contents) status = _fileWrite(p, file.Length, c, (nuint)contents.Length);
            Drain();
            return status >= 0;
        }

        public static string RomMd5(string path)
        {
            Utf8 file = new(path);
            try
            {
                return Text((o, n) => { fixed (byte* p = file.Bytes) return _romMd5(p, file.Length, o, n); })!;
            }
            catch (FileNotFoundException missing) when (!System.IO.Directory.Exists(Path.GetDirectoryName(Path.GetFullPath(path))))
            {
                // .NET tells a missing folder from a missing file, and a caller may catch either.
                throw new DirectoryNotFoundException(missing.Message);
            }
        }

        // How many files were copied; the files left in place are reported, and a tree that would not list is an exception, as in the C#.
        public static int Migrate(GalaxiaMigrate what, string? source = null, string? destination = null)
        {
            Utf8 s = new(source), d = new(destination);
            long copied;
            fixed (byte* ps = s.Bytes, pd = d.Bytes) copied = _migrate((uint)what, ps, s.Length, pd, d.Length);
            Drain();
            if (copied < 0) throw Fault((int)copied);
            return (int)copied;
        }

        public static IReadOnlyList<string> RemainingLibrary(string? legacyRoot)
        {
            Utf8 r = new(legacyRoot);
            string found = Text((o, n) => { fixed (byte* pr = r.Bytes) return _remainingLibrary(pr, r.Length, o, n); }) ?? "";
            Drain();
            return found.Split('\0', StringSplitOptions.RemoveEmptyEntries);
        }

        public static string? PathRule(GalaxiaPathRule rule, string a, string? b = null)
        {
            Utf8 x = new(a), y = new(b);
            return Text((o, n) => { fixed (byte* px = x.Bytes, py = y.Bytes) return _dotnetPath((uint)rule, px, x.Length, py, y.Length, o, n); });
        }

        public static bool TestRule(GalaxiaTestRule rule, string? a, string? b = null)
        {
            Utf8 x = new(a), y = new(b);
            int answer;
            fixed (byte* px = x.Bytes, py = y.Bytes) answer = _dotnetTest((uint)rule, px, x.Length, py, y.Length);
            Check(answer);
            return answer != 0;
        }
    }
}
