using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Library
{
    // A status the library answered with, and its words, as emusen_library.h names them.
    public sealed class GameLibraryException(int status, string words) : Exception(words)
    {
        public const int NewerFile = -1140, Database = -1141, NoSqlite = -1142, BadRequest = -1143, Cancelled = -1144, Busy = -1145, Io = -1146;

        public int Status { get; } = status;
    }

    // emusen_library.h's calls over a handle on one games.db, each request and answer the header's JSON; the host's SQLite lent - see EmuSen_Backend.md §28.
    public sealed unsafe class GameLibrary : IDisposable
    {
        public const string Variable = "EMUSEN_LIBRARY_NATIVE";

        private const uint Major = 1;

        private struct Config
        {
            public uint Size;
            public byte* Database;
            public nuint DatabaseLen;
            public nint Sqlite;
        }

        private sealed class Exports
        {
            public delegate* unmanaged[Cdecl]<uint> AbiVersion;
            public delegate* unmanaged[Cdecl]<byte*, nuint, long> LastError;
            public delegate* unmanaged[Cdecl]<Config*, nint*, int> Open;
            public delegate* unmanaged[Cdecl]<nint, int> Close;
            public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> Scan;
            public delegate* unmanaged[Cdecl]<nint, byte*, nuint, long> Progress;
            public delegate* unmanaged[Cdecl]<nint, int> Cancel;
            public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> Games, Identity, Get, Change;
            public nint Sqlite;
        }

        // Every export the binding takes; a library without one is not used.
        public static readonly IReadOnlyList<string> ExportNames = new[]
        {
            "emusen_library_abi_version", "emusen_library_last_error", "emusen_library_open", "emusen_library_close", "emusen_library_scan",
            "emusen_library_progress", "emusen_library_cancel", "emusen_library_games", "emusen_library_identity", "emusen_library_get", "emusen_library_change",
        };

        private static readonly Lazy<(Exports? Table, string Report)> Loaded = new(Load);

        private static (Exports?, string) Load()
        {
            if (!PlatformLibrary.Available) return (null, PlatformLibrary.Report);
            nint[] found = ExportNames.Select(PlatformLibrary.Export).ToArray();
            if (Array.IndexOf(found, 0) is var missing and >= 0) return (null, $"{PlatformLibrary.FileName} has no {ExportNames[missing]}");
            var t = new Exports
            {
                AbiVersion = (delegate* unmanaged[Cdecl]<uint>)found[0],
                LastError = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)found[1],
                Open = (delegate* unmanaged[Cdecl]<Config*, nint*, int>)found[2],
                Close = (delegate* unmanaged[Cdecl]<nint, int>)found[3],
                Scan = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[4],
                Progress = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)found[5],
                Cancel = (delegate* unmanaged[Cdecl]<nint, int>)found[6],
                Games = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)found[7],
                Identity = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)found[8],
                Get = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)found[9],
                Change = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)found[10],
            };
            if (t.AbiVersion() >> 16 != Major) return (null, $"{PlatformLibrary.FileName} speaks another major of emusen_library.h");
            try
            {
                SQLitePCL.Batteries_V2.Init();
                if (!NativeLibrary.TryLoad(SQLitePCL.raw.GetNativeLibraryName(), typeof(SQLitePCL.SQLite3Provider_e_sqlite3).Assembly, null, out t.Sqlite))
                    return (null, "SQLite could not be found as the C# loads it");
            }
            catch (Exception e) when (e is DllNotFoundException or FileNotFoundException or TypeInitializationException or InvalidOperationException)
            {
                return (null, $"SQLite could not be lent ({e.GetType().Name}: {e.Message.Split('\n')[0]})");
            }
            return (t, $"{PlatformLibrary.FileName}, the library {t.AbiVersion() >> 16}.{t.AbiVersion() & 0xFFFF}");
        }

        // The library itself, whichever the switch chose: what the parity class calls.
        public static bool Ready => Loaded.Value.Table is not null;

        public static string Report => Loaded.Value.Report;

        // State 1: the C# unless the variable is 1 and the library serves every call.
        public static bool Active => Environment.GetEnvironmentVariable(Variable) == "1" && Ready;

        private readonly Exports _t;
        private nint _handle;

        private GameLibrary(Exports t, nint handle)
        {
            _t = t;
            _handle = handle;
        }

        // Opens games.db at path, migrating it; ":memory:" is a database that lives as long as the handle.
        public static GameLibrary Open(string path)
        {
            Exports t = Loaded.Value.Table ?? throw new InvalidOperationException($"The library is not in use ({Report}).");
            byte[] bytes = Encoding.UTF8.GetBytes(path);
            nint handle;
            int status;
            fixed (byte* p = bytes)
            {
                var config = new Config { Size = (uint)sizeof(Config), Database = p, DatabaseLen = (nuint)bytes.Length, Sqlite = t.Sqlite };
                status = t.Open(&config, &handle);
            }
            if (status != 0) throw Fault(t, status);
            return new GameLibrary(t, handle);
        }

        private static GameLibraryException Fault(Exports t, long status)
        {
            long n = t.LastError(null, 0);
            var words = new byte[Math.Max(0, n)];
            fixed (byte* w = words) t.LastError(w, (nuint)words.Length);
            return new GameLibraryException((int)status, Encoding.UTF8.GetString(words));
        }

        private nint Handle => _handle != 0 ? _handle : throw new ObjectDisposedException(nameof(GameLibrary));

        // An answer by the length-query idiom: a stack buffer first, then once more at the length asked.
        private string Call(delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> call, string request)
        {
            byte[] r = Encoding.UTF8.GetBytes(request);
            nint h = Handle;
            fixed (byte* rp = r)
            {
                byte* first = stackalloc byte[1024];
                long n = call(h, rp, (nuint)r.Length, first, 1024);
                if (n < 0) throw Fault(_t, n);
                if (n <= 1024) return Encoding.UTF8.GetString(first, (int)n);
                var out_ = new byte[n];
                fixed (byte* o = out_)
                {
                    long m = call(h, rp, (nuint)r.Length, o, (nuint)n);
                    if (m < 0) throw Fault(_t, m);
                    return Encoding.UTF8.GetString(o, (int)m);
                }
            }
        }

        // Scans and keeps the result; the games found.
        public long Scan(string request)
        {
            byte[] r = Encoding.UTF8.GetBytes(request);
            long n;
            fixed (byte* rp = r) n = _t.Scan(Handle, rp, (nuint)r.Length);
            if (n < 0) throw Fault(_t, n);
            return n;
        }

        public string Games(string filter) => Call(_t.Games, filter);

        public string Identity(string request) => Call(_t.Identity, request);

        public string Get(string request) => Call(_t.Get, request);

        public string Change(string request) => Call(_t.Change, request);

        public string Progress()
        {
            nint h = Handle;
            byte* buffer = stackalloc byte[512];
            long n = _t.Progress(h, buffer, 512);
            if (n < 0) throw Fault(_t, n);
            if (n <= 512) return Encoding.UTF8.GetString(buffer, (int)n);
            var out_ = new byte[n];
            fixed (byte* o = out_) n = _t.Progress(h, o, (nuint)out_.Length);
            return Encoding.UTF8.GetString(out_, 0, (int)n);
        }

        public void Cancel() => _t.Cancel(Handle);

        public void Dispose()
        {
            if (_handle == 0) return;
            _t.Close(_handle);
            _handle = 0;
        }
    }
}
