using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Crystal.Client.Library;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Scrape
{
    // A status the scraper answered with, and its words, as emusen_scrape.h names them; the words are always redacted.
    public sealed class MediaServiceException(int status, string words) : Exception(words)
    {
        public const int NewerFile = -1160, Database = -1161, NoSqlite = -1162, BadRequest = -1163, Busy = -1164, Io = -1165, NoLibrary = -1166;

        public int Status { get; } = status;
    }

    // emusen_scrape.h's calls over a handle on one media store, each request and answer the header's JSON; the host's SQLite and library lent - see EmuSen_Backend.md §30.
    public sealed unsafe class MediaService : IDisposable
    {
        public const string Variable = "EMUSEN_SCRAPE_NATIVE";

        private const uint Major = 1;

        private struct Config
        {
            public uint Size;
            public byte* Root;
            public nuint RootLen;
            public nint Sqlite;
            public nint Library;
            public byte* Embedded;
            public nuint EmbeddedLen;
            public byte* Settings;
            public nuint SettingsLen;
        }

        private sealed class Exports
        {
            public delegate* unmanaged[Cdecl]<uint> AbiVersion;
            public delegate* unmanaged[Cdecl]<byte*, nuint, long> LastError;
            public delegate* unmanaged[Cdecl]<Config*, nint*, int> Open;
            public delegate* unmanaged[Cdecl]<nint, int> Close;
            public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> Get, Change;
            public nint Sqlite;
        }

        // Every export the binding takes; a library without one is not used.
        public static readonly IReadOnlyList<string> ExportNames = new[]
        {
            "emusen_scrape_abi_version", "emusen_scrape_last_error", "emusen_scrape_open", "emusen_scrape_close", "emusen_scrape_get", "emusen_scrape_change",
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
                Get = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)found[4],
                Change = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)found[5],
            };
            if (t.AbiVersion() >> 16 != Major) return (null, $"{PlatformLibrary.FileName} speaks another major of emusen_scrape.h");
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
            return (t, $"{PlatformLibrary.FileName}, the scraper {t.AbiVersion() >> 16}.{t.AbiVersion() & 0xFFFF}");
        }

        // The scraper itself, whichever the switch chose: what the parity classes call.
        public static bool Ready => Loaded.Value.Table is not null;

        public static string Report => Loaded.Value.Report;

        private static readonly PlatformSwitch Chosen = new(() => Environment.GetEnvironmentVariable(Variable), () => (Ready, Report), PlatformSwitch.Announce, "the scraper");

        // The scraper unless the variable is 0; a library absent or without an export leaves the C#, said once.
        public static bool Active => Chosen.Active;

        private readonly Exports _t;
        private nint _handle;
        private readonly object _gate = new();
        private int _calls;
        private bool _closing;

        private MediaService(Exports t, nint handle)
        {
            _t = t;
            _handle = handle;
        }

        // Opens the media store at root with the settings' JSON; the library, when given, must stay open while this does.
        public static MediaService Open(string root, string settings, GameLibrary? library = null, byte[]? embedded = null)
        {
            Exports t = Loaded.Value.Table ?? throw new InvalidOperationException($"The scraper is not in use ({Report}).");
            byte[] r = Encoding.UTF8.GetBytes(root), s = Encoding.UTF8.GetBytes(settings), e = embedded ?? [];
            nint handle;
            int status;
            fixed (byte* rp = r)
            fixed (byte* sp = s)
            fixed (byte* ep = e)
            {
                var config = new Config
                {
                    Size = (uint)sizeof(Config), Root = rp, RootLen = (nuint)r.Length, Sqlite = t.Sqlite, Library = library?.Handle ?? 0,
                    Embedded = ep, EmbeddedLen = (nuint)e.Length, Settings = sp, SettingsLen = (nuint)s.Length,
                };
                status = t.Open(&config, &handle);
            }
            if (status != 0) throw Fault(t, status);
            return new MediaService(t, handle);
        }

        private static MediaServiceException Fault(Exports t, long status)
        {
            long n = t.LastError(null, 0);
            var words = new byte[Math.Max(0, n)];
            fixed (byte* w = words) t.LastError(w, (nuint)words.Length);
            return new MediaServiceException((int)status, Encoding.UTF8.GetString(words));
        }

        private nint Enter()
        {
            lock (_gate)
            {
                if (_handle == 0 || _closing) throw new ObjectDisposedException(nameof(MediaService));
                _calls++;
                return _handle;
            }
        }

        private void Leave()
        {
            lock (_gate)
                if (--_calls == 0) Monitor.PulseAll(_gate);
        }

        private string Call(delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> call, string request)
        {
            byte[] r = Encoding.UTF8.GetBytes(request);
            nint h = Enter();
            try
            {
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
            finally { Leave(); }
        }

        public string Get(string request) => Call(_t.Get, request);

        public string Change(string request) => Call(_t.Change, request);

        // Waits for every call in progress, then closes the handle, which cancels a run and joins its workers.
        public void Dispose()
        {
            lock (_gate)
            {
                if (_handle == 0 || _closing) return;
                _closing = true;
                while (_calls > 0) Monitor.Wait(_gate);
                _t.Close(_handle);
                _handle = 0;
            }
        }
    }
}
