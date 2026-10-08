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
        LogDefault,
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

    // The config models whose schemas the library owns, as EMUSEN_GALAXIA_MODEL_*.
    internal enum GalaxiaModel : uint
    {
        AppSettings,
        AudioConfig,
        GraphicsConfig,
        CheatFile,
    }

    // State 2's choice, made once: the variable 0 is the C#, anything else the library if it loads, else the C# with one message - see EmuSen_RustPlatform.md §13.
    internal sealed class PlatformSwitch
    {
        private readonly Func<string?> _variable;
        private readonly Func<(bool Available, string Report)> _load;
        private readonly Action<string> _announce;
        private readonly object _gate = new();
        private volatile int _chosen;

        public PlatformSwitch(Func<string?> variable, Func<(bool Available, string Report)> load, Action<string> announce)
        {
            _variable = variable;
            _load = load;
            _announce = announce;
        }

        public bool Active => _chosen != 0 ? _chosen == 2 : Choose();

        private bool Choose()
        {
            lock (_gate)
            {
                if (_chosen != 0) return _chosen == 2;
                string? message = null;
                if (_variable() == "0") _chosen = 1;
                else
                {
                    (bool available, string report) = Load();
                    _chosen = available ? 2 : 1;
                    if (!available) message = $"{PlatformLibrary.FileName} is not in use ({report}); Galaxia runs on its C# implementation.";
                }
                // Decided before it is said, so the log's own question of the switch has its answer.
                if (message is not null) _announce(message);
                return _chosen == 2;
            }
        }

        private (bool, string) Load()
        {
            try { return _load(); }
            catch (Exception fault) { return (false, $"{fault.GetType().Name}: {fault.Message}"); }
        }
    }

    // Galaxia's half of emusen_platform, and the switch that chooses it over the C# - see EmuSen_RustPlatform.md §3.9 and §10.2.
    internal static unsafe class GalaxiaNative
    {
        public const string Variable = "EMUSEN_GALAXIA_NATIVE";

        private const int Parse = -1025;
        private const int Absent = -1030;
        private const int NotFound = -1031;
        private const int Access = -1032;

        // Held while an override moves and while the library first takes the overrides, so neither side misses a change.
        internal static readonly object Gate = new();

        private static readonly PlatformSwitch Chosen = new(() => Environment.GetEnvironmentVariable(Variable), () => (PlatformLibrary.Available, PlatformLibrary.Report), Announce);
        private static volatile bool _attached;

        // State 2: the library unless the variable is 0; a library that is absent or refused leaves the C#, said once - see EmuSen_RustPlatform.md §13.
        public static bool Active => Chosen.Active;

        // The one line a fallback writes, to the host's sink and to the error log, which is by then on the C#.
        internal static void Announce(string message)
        {
            ConfigDiagnostics.Report(message);
            EmuSen.Galaxia.Library.ErrorLog.Warning("platform", message);
        }

        // The library itself, whichever the switch chose: what a parity test calls.
        public static bool Ready => PlatformLibrary.Available;

        // Whether the library is in the process already; asking never loads it.
        public static bool Loaded => _attached;

        // The model a C# class is, or null for a type whose vocabulary belongs to a frontend - see EmuSen_RustPlatform.md §4.1.
        public static GalaxiaModel? ModelOf(Type type) =>
            type == typeof(Models.AppSettings) ? GalaxiaModel.AppSettings
            : type == typeof(Models.AudioConfig) ? GalaxiaModel.AudioConfig
            : type == typeof(Models.GraphicsConfig) ? GalaxiaModel.GraphicsConfig
            : type == typeof(Models.CheatFile) ? GalaxiaModel.CheatFile
            : null;

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
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int> _configDelete;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long> _configRead;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, int> _configWrite;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, uint, byte*, nuint, long> _modelLoad;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, byte*, nuint, int> _modelSave;
        private static delegate* unmanaged[Cdecl]<uint, uint, byte*, nuint, long> _modelNew;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, uint, byte*, nuint, long> _modelBind;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, long> _modelFormat;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, long> _modelSchema;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, long> _modelLoadFrom;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int> _modelSaveTo;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, long> _cheatNames;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, int> _cheatNameValid;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _logRoot;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, int> _logUsable;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long> _logPath;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, long> _logFormat;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long, byte*, nuint, int> _logAppend;
        private static delegate* unmanaged[Cdecl]<int> _logReset;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int> _suggestDistance;
        private static delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int, byte*, nuint, long> _suggest;
        private static delegate* unmanaged[Cdecl]<uint, ulong, byte*, nuint, long> _numberFormat;
        private static delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> _textDecode;

        // Every export this class calls; a name the library lacks refuses the library - see EmuSen_RustPlatform.md §3.1.
        internal static readonly string[] Exports =
        {
            "emusen_platform_set_crash_log", "emusen_platform_last_error", "emusen_platform_diagnostics", "emusen_galaxia_init",
            "emusen_galaxia_directory", "emusen_galaxia_set_override", "emusen_galaxia_config_path", "emusen_galaxia_legacy_config_path",
            "emusen_galaxia_root_for", "emusen_galaxia_bundle_contents_for", "emusen_galaxia_seed_directory_for",
            "emusen_galaxia_mac_data_directory_for", "emusen_galaxia_legacy_root_for", "emusen_galaxia_save_path", "emusen_galaxia_file_read",
            "emusen_galaxia_file_write", "emusen_galaxia_rom_md5", "emusen_galaxia_migrate", "emusen_galaxia_remaining_library",
            "emusen_galaxia_dotnet_path", "emusen_galaxia_dotnet_test",
            "emusen_galaxia_config_delete", "emusen_galaxia_config_read", "emusen_galaxia_config_write", "emusen_galaxia_model_load", "emusen_galaxia_model_save",
            "emusen_galaxia_model_new", "emusen_galaxia_model_bind", "emusen_galaxia_model_format", "emusen_galaxia_model_schema", "emusen_galaxia_model_load_from",
            "emusen_galaxia_model_save_to", "emusen_galaxia_cheat_names", "emusen_galaxia_cheat_name_valid", "emusen_galaxia_log_root", "emusen_galaxia_log_usable",
            "emusen_galaxia_log_path", "emusen_galaxia_log_format", "emusen_galaxia_log_append", "emusen_galaxia_log_reset", "emusen_galaxia_suggest_distance",
            "emusen_galaxia_suggest", "emusen_galaxia_number_format", "emusen_galaxia_text_decode",
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
            _configDelete = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int>)found[next++];
            _configRead = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)found[next++];
            _configWrite = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, int>)found[next++];
            _modelLoad = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, uint, byte*, nuint, long>)found[next++];
            _modelSave = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, byte*, nuint, int>)found[next++];
            _modelNew = (delegate* unmanaged[Cdecl]<uint, uint, byte*, nuint, long>)found[next++];
            _modelBind = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, uint, byte*, nuint, long>)found[next++];
            _modelFormat = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, long>)found[next++];
            _modelSchema = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, long>)found[next++];
            _modelLoadFrom = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, long>)found[next++];
            _modelSaveTo = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int>)found[next++];
            _cheatNames = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[next++];
            _cheatNameValid = (delegate* unmanaged[Cdecl]<byte*, nuint, int>)found[next++];
            _logRoot = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];
            _logUsable = (delegate* unmanaged[Cdecl]<byte*, nuint, int>)found[next++];
            _logPath = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)found[next++];
            _logFormat = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, long>)found[next++];
            _logAppend = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long, byte*, nuint, int>)found[next++];
            _logReset = (delegate* unmanaged[Cdecl]<int>)found[next++];
            _suggestDistance = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int>)found[next++];
            _suggest = (delegate* unmanaged[Cdecl]<uint, byte*, nuint, byte*, nuint, int, byte*, nuint, long>)found[next++];
            _numberFormat = (delegate* unmanaged[Cdecl]<uint, ulong, byte*, nuint, long>)found[next++];
            _textDecode = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)found[next++];

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

        // Whether every text crosses as it is: UTF-16 that is not valid has no UTF-8, and a NUL would end a list's item early.
        public static bool Crosses(string? text, bool listed = false)
        {
            if (text is null) return true;
            for (int i = 0; i < text.Length; i++)
            {
                if (listed && text[i] == '\0') return false;
                if (char.IsHighSurrogate(text[i]) && i + 1 < text.Length && char.IsLowSurrogate(text[i + 1])) i++;
                else if (char.IsSurrogate(text[i])) return false;
            }
            return true;
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
            return Text(query, length);
        }

        // The same for a document, whose first buffer is sized for a settings file so that it too is read once.
        private static string? Document(Query query) => Text(query, 16 * 1024);

        private static string? Text(Query query, long length)
        {
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
                Parse => new FormatException(words),
                Access => new UnauthorizedAccessException(words),
                -1 or -5 or -1033 => new ArgumentException(words),
                _ => new IOException(words),
            };
        }

        // What the library queued where the C# calls ConfigDiagnostics.Report, reported here in its order - see EmuSen_RustPlatform.md §3.5.
        private static void Drain()
        {
            foreach (string message in Items(Text((o, n) => _diagnostics(o, n)))) ConfigDiagnostics.Report(message);
        }

        // A list as it crosses, each item followed by a NUL, so that an item that is empty is still an item.
        private static string[] Items(string? joined) => string.IsNullOrEmpty(joined) ? Array.Empty<string>() : joined[..^1].Split('\0');

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
            string[] found = Items(Text((o, n) => { fixed (byte* pr = r.Bytes) return _remainingLibrary(pr, r.Length, o, n); }));
            Drain();
            return found;
        }

        public static bool ConfigDelete(string? category, string fileName)
        {
            Utf8 c = new(category), f = new(fileName);
            fixed (byte* pc = c.Bytes, pf = f.Bytes) return _configDelete(pc, c.Length, pf, f.Length) > 0;
        }

        // Null for no file; an exception with the system's words for one that will not read.
        public static string? ConfigRead(string? category, string fileName)
        {
            Utf8 c = new(category), f = new(fileName);
            return Document((o, n) => { fixed (byte* pc = c.Bytes, pf = f.Bytes) return _configRead(pc, c.Length, pf, f.Length, o, n); });
        }

        public static bool ConfigWrite(string? category, string fileName, string? text)
        {
            Utf8 c = new(category), f = new(fileName), t = new(text);
            fixed (byte* pc = c.Bytes, pf = f.Bytes, pt = t.Bytes) return _configWrite(pc, c.Length, pf, f.Length, pt, t.Length) >= 0;
        }

        // The bound document as JSON, "null" for a file that says so, null for no file; an exception with the words for one that will not load.
        public static string? ModelLoad(GalaxiaModel model, string? category, string fileName, bool upgrade)
        {
            Utf8 c = new(category), f = new(fileName);
            return Document((o, n) => { fixed (byte* pc = c.Bytes, pf = f.Bytes) return _modelLoad((uint)model, pc, c.Length, pf, f.Length, upgrade ? 1u : 0u, o, n); });
        }

        public static bool ModelSave(GalaxiaModel model, string? category, string fileName, string? document)
        {
            Utf8 c = new(category), f = new(fileName), d = new(document);
            fixed (byte* pc = c.Bytes, pf = f.Bytes, pd = d.Bytes) return _modelSave((uint)model, pc, c.Length, pf, f.Length, pd, d.Length) >= 0;
        }

        public static string ModelNew(GalaxiaModel model, bool upgrade) => Document((o, n) => _modelNew((uint)model, upgrade ? 1u : 0u, o, n))!;

        public static string ModelBind(GalaxiaModel model, string text, bool upgrade)
        {
            Utf8 t = new(text);
            return Document((o, n) => { fixed (byte* pt = t.Bytes) return _modelBind((uint)model, pt, t.Length, upgrade ? 1u : 0u, o, n); })!;
        }

        public static string ModelFormat(GalaxiaModel model, string document)
        {
            Utf8 d = new(document);
            return Document((o, n) => { fixed (byte* pd = d.Bytes) return _modelFormat((uint)model, pd, d.Length, o, n); })!;
        }

        public static string ModelSchema(GalaxiaModel model) => Document((o, n) => _modelSchema((uint)model, o, n))!;

        public static string? ModelLoadFrom(GalaxiaModel model, string path)
        {
            Utf8 p = new(path);
            return Document((o, n) => { fixed (byte* pp = p.Bytes) return _modelLoadFrom((uint)model, pp, p.Length, o, n); });
        }

        public static bool ModelSaveTo(GalaxiaModel model, string path, string? document)
        {
            Utf8 p = new(path), d = new(document);
            fixed (byte* pp = p.Bytes, pd = d.Bytes) return _modelSaveTo((uint)model, pp, p.Length, pd, d.Length) >= 0;
        }

        public static IReadOnlyList<string> CheatNames() => Items(Document((o, n) => _cheatNames(o, n)));

        public static bool CheatNameValid(string? name)
        {
            Utf8 text = new(name);
            fixed (byte* p = text.Bytes) return _cheatNameValid(p, text.Length) > 0;
        }

        public static string LogRoot(string? directoryOverride)
        {
            Utf8 d = new(directoryOverride);
            string root = Text((o, n) => { fixed (byte* pd = d.Bytes) return _logRoot(pd, d.Length, o, n); })!;
            Drain();
            return root;
        }

        public static bool LogUsable(string? directory)
        {
            Utf8 d = new(directory);
            fixed (byte* pd = d.Bytes) return _logUsable(pd, d.Length) > 0;
        }

        public static string LogPath(string root, string day)
        {
            Utf8 r = new(root), d = new(day);
            return Text((o, n) => { fixed (byte* pr = r.Bytes, pd = d.Bytes) return _logPath(pr, r.Length, pd, d.Length, o, n); })!;
        }

        public static string LogFormat(string stamp, string level, string? area, string message, string? context, string? fault)
        {
            Utf8 s = new(stamp), l = new(level), a = new(area), m = new(message), c = new(context), f = new(fault);
            return Document((o, n) =>
            {
                fixed (byte* ps = s.Bytes, pl = l.Bytes, pa = a.Bytes, pm = m.Bytes, pc = c.Bytes, pf = f.Bytes)
                    return _logFormat(ps, s.Length, pl, l.Length, pa, a.Length, pm, m.Length, pc, c.Length, pf, f.Length, o, n);
            })!;
        }

        // A null entry is text with no UTF-8, which fails at the write as it does in C#.
        public static bool LogAppend(string file, string stamp, long nowUnixMs, string? entry)
        {
            Utf8 f = new(file), s = new(stamp), e = new(entry);
            fixed (byte* pf = f.Bytes, ps = s.Bytes, pe = e.Bytes) return _logAppend(pf, f.Length, ps, s.Length, nowUnixMs, pe, e.Length) > 0;
        }

        public static void LogReset() => _logReset();

        public static int SuggestDistance(string a, string b)
        {
            Utf8 x = new(a), y = new(b);
            int distance;
            fixed (byte* px = x.Bytes, py = y.Bytes) distance = _suggestDistance(px, x.Length, py, y.Length);
            Check(distance);
            return distance;
        }

        // The candidates cross as one text, each followed by a NUL; nearest is false for the sentence and true for the names.
        private static string Suggest(bool nearest, string typed, IReadOnlyList<string?> candidates, int max)
        {
            Utf8 t = new(typed), c = new(string.Concat(System.Linq.Enumerable.Select(candidates, name => name + "\0")));
            return Text((o, n) => { fixed (byte* pt = t.Bytes, pc = c.Bytes) return _suggest(nearest ? 1u : 0u, pt, t.Length, pc, c.Length, max, o, n); })!;
        }

        public static string SuggestHint(string typed, IReadOnlyList<string?> candidates, int max) => Suggest(nearest: false, typed, candidates, max);

        public static IReadOnlyList<string> SuggestNearest(string typed, IReadOnlyList<string?> candidates, int max) => Items(Suggest(nearest: true, typed, candidates, max));

        // A double's or a float's bits as the library writes the number; null for one JSON has no token for.
        public static string? NumberFormat(bool single, ulong bits) => Text((o, n) => _numberFormat(single ? 1u : 0u, bits, o, n));

        public static string TextDecode(byte[] bytes) =>
            Document((o, n) => { fixed (byte* pb = bytes) return _textDecode(pb, (nuint)bytes.Length, o, n); })!;

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
