using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client.Library;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Client.Scrape
{
    // A status the scraper answered with, and its words, as emusen_scrape_service.h names them; the words are always redacted.
    public sealed class MediaServiceException(int status, string words) : Exception(words)
    {
        public const int NewerFile = -1160, Database = -1161, NoSqlite = -1162, BadRequest = -1163, Busy = -1164, Io = -1165, NoLibrary = -1166, StoreInUse = -1167;

        public int Status { get; } = status;
    }

    // The process's one scraper, opened with emusen_scraping_open and asked through the emusen.scrape table; a later Open closes this one - see EmuSen_LibraryTools.md §8.
    public sealed unsafe class MediaService : IDisposable
    {
        public const string Variable = "EMUSEN_SCRAPE_NATIVE";

        public const string Since = "2026-11-05";

        private struct Config
        {
            public uint Size;
            public byte* Root;
            public nuint RootLen;
            public byte* GamesDb;
            public nuint GamesDbLen;
            public nint Sqlite;
            public byte* Embedded;
            public nuint EmbeddedLen;
            public byte* Settings;
            public nuint SettingsLen;
            public byte* Systems;
            public nuint SystemsLen;
            public delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long> ThemeKinds;
            public nint Ctx;
        }

        private sealed class Exports
        {
            public delegate* unmanaged[Cdecl]<Config*, long> Open;
            public delegate* unmanaged[Cdecl]<ulong, int> Close;
            public ScrapeService* Table;
            public nint Sqlite;
        }

        // Every export the binding takes; a library without one is not used.
        public static readonly IReadOnlyList<string> ExportNames = new[] { "emusen_scraping_open", "emusen_scraping_close", "emusen_scraping_service" };

        private static readonly Lazy<(Exports? Table, string Report)> Loaded = new(Load);

        private static (Exports?, string) Load()
        {
            try { RetiredSwitch.Note(Variable, "the scraper", Since); } catch (Exception) { }
            if (!PlatformLibrary.Available) return (null, PlatformLibrary.Report);
            nint[] found = ExportNames.Select(PlatformLibrary.Export).ToArray();
            if (Array.IndexOf(found, 0) is var missing and >= 0) return (null, $"{PlatformLibrary.FileName} has no {ExportNames[missing]}");
            var t = new Exports
            {
                Open = (delegate* unmanaged[Cdecl]<Config*, long>)found[0],
                Close = (delegate* unmanaged[Cdecl]<ulong, int>)found[1],
                Table = ScrapeService.Table,
            };
            if (t.Table == null) return (null, $"{PlatformLibrary.FileName}'s emusen.scrape table is shorter than 2.0's");
            t.Sqlite = SqliteLend.Handle;
            if (t.Sqlite == 0) return (null, SqliteLend.Problem);
            return (t, $"{PlatformLibrary.FileName}, the emusen.scrape service 2.0");
        }

        public static bool Ready => Loaded.Value.Table is not null;

        public static string Report => Loaded.Value.Report;

        // The words a missing or unusable platform library is refused in; there is no C# to fall back to - see EmuSen_Backend.md §40.5.
        public static string Refusal(string why) => $"{PlatformLibrary.FileName} is not in use for the scraper ({why}); the scraper has had no C# implementation since {Since}.";

        private static readonly object Gate = new();
        private static volatile MediaService? _current;
        private static volatile Func<string, IReadOnlyList<string>?>? _themeKinds;

        private readonly ScrapeService* _t;
        private readonly ulong _serial;
        private readonly object _gate = new();
        private int _calls;
        private bool _closing;

        private MediaService(ScrapeService* t, ulong serial)
        {
            _t = t;
            _serial = serial;
        }

        // Opens the process's scraper on the media store at root with the settings' JSON, closing the one before; games.db is opened by the library's file, the systems are those the build opens, and the theme's kinds per ES-DE system answer the shells' --kinds theme.
        public static MediaService Open(string root, string settings, GameLibrary? library = null, byte[]? embedded = null, IEnumerable<string>? systems = null, Func<string, IReadOnlyList<string>?>? themeKinds = null)
        {
            Exports t = Loaded.Value.Table ?? throw new DllNotFoundException(Refusal(Report));
            string games = library is null || library.Path == ":memory:" ? "" : library.Path;
            byte[] r = Encoding.UTF8.GetBytes(root), s = Encoding.UTF8.GetBytes(settings), e = embedded ?? [], g = Encoding.UTF8.GetBytes(games);
            byte[] y = systems is null ? [] : Encoding.UTF8.GetBytes(JsonSerializer.Serialize(systems.Distinct().ToArray()));
            lock (Gate)
            {
                _current = null;
                _themeKinds = themeKinds;
                long serial;
                fixed (byte* rp = r)
                fixed (byte* sp = s)
                fixed (byte* ep = e)
                fixed (byte* gp = g)
                fixed (byte* yp = y)
                {
                    var config = new Config
                    {
                        Size = (uint)sizeof(Config), Root = rp, RootLen = (nuint)r.Length, GamesDb = gp, GamesDbLen = (nuint)g.Length, Sqlite = t.Sqlite,
                        Embedded = ep, EmbeddedLen = (nuint)e.Length, Settings = sp, SettingsLen = (nuint)s.Length, Systems = yp, SystemsLen = (nuint)y.Length,
                        ThemeKinds = themeKinds is null ? null : &ThemeKinds,
                    };
                    serial = t.Open(&config);
                }
                if (serial <= 0) throw Fault(t.Table, serial);
                return _current = new MediaService(t.Table, (ulong)serial);
            }
        }

        // Closes any scraper and keeps the platform library from opening one of its own from the settings file, for a program that scrapes itself.
        public static void CloseAny()
        {
            if (Loaded.Value.Table is not { } t) return;
            lock (Gate)
            {
                t.Close(0);
                (_current, _themeKinds) = (null, null);
            }
        }

        private static MediaServiceException Fault(ScrapeService* t, long status) => new((int)status, ScrapeService.Words(t));

        private void Enter()
        {
            lock (_gate)
            {
                if (_closing || !ReferenceEquals(_current, this)) throw new ObjectDisposedException(nameof(MediaService));
                _calls++;
            }
        }

        private void Leave()
        {
            lock (_gate)
                if (--_calls == 0) Monitor.PulseAll(_gate);
        }

        private string Call(delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long> call, string request)
        {
            byte[] r = Encoding.UTF8.GetBytes(request);
            Enter();
            try
            {
                fixed (byte* rp = r)
                {
                    byte* first = stackalloc byte[1024];
                    long n = call(rp, (nuint)r.Length, first, 1024);
                    if (n < 0) throw Fault(_t, n);
                    if (n <= 1024) return Encoding.UTF8.GetString(first, (int)n);
                    var out_ = new byte[n];
                    fixed (byte* o = out_)
                    {
                        long m = call(rp, (nuint)r.Length, o, (nuint)n);
                        if (m < 0) throw Fault(_t, m);
                        return Encoding.UTF8.GetString(o, (int)m);
                    }
                }
            }
            finally { Leave(); }
        }

        public string Get(string request) => Call(_t->Get, request);

        public string Change(string request) => Call(_t->Change, request);

        // The run as it stands, its results after the serial named.
        public string Progress(long since = 0) => Call(_t->Progress, $"{{\"since\":{since}}}");

        // Starts a run; the games queued.
        public long Start(string request)
        {
            byte[] r = Encoding.UTF8.GetBytes(request);
            long n;
            Enter();
            try
            {
                fixed (byte* rp = r) n = _t->Start(rp, (nuint)r.Length);
            }
            finally { Leave(); }
            if (n < 0) throw Fault(_t, n);
            return n;
        }

        // Waits until the run's serial passes serial, or the timeout ends; the serial then. The table waits at most 250 ms a call, so a longer wait asks again.
        public long Wait(long serial, int timeoutMs)
        {
            long until = Environment.TickCount64 + Math.Max(0, timeoutMs);
            Enter();
            try
            {
                while (true)
                {
                    long left = until - Environment.TickCount64;
                    long now = _t->Wait((ulong)Math.Max(0, serial), (uint)Math.Clamp(left, 0, 250));
                    if (now < 0) throw Fault(_t, now);
                    if (now != serial || left <= 250) return now;
                }
            }
            finally { Leave(); }
        }

        public bool Pause(bool paused)
        {
            Enter();
            try { return _t->Pause(paused ? 1 : 0) == 1; }
            finally { Leave(); }
        }

        public bool Cancel()
        {
            Enter();
            try { return _t->Cancel() == 1; }
            finally { Leave(); }
        }

        // Waits for every call in progress, then closes the scraper when it is still the process's, which cancels a run and joins its workers.
        public void Dispose()
        {
            lock (_gate)
            {
                if (_closing) return;
                _closing = true;
                while (_calls > 0) Monitor.Wait(_gate);
            }
            lock (Gate)
            {
                if (!ReferenceEquals(_current, this)) return;
                Loaded.Value.Table!.Close(_serial);
                (_current, _themeKinds) = (null, null);
            }
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
        private static long ThemeKinds(nint ctx, byte* system, nuint len, byte* output, nuint outLen)
        {
            try
            {
                IReadOnlyList<string>? kinds = _themeKinds?.Invoke(Encoding.UTF8.GetString(system, (int)len));
                if (kinds is null) return -1;
                byte[] answer = Encoding.UTF8.GetBytes(JsonSerializer.Serialize(kinds));
                if (output != null && (nuint)answer.Length <= outLen) answer.CopyTo(new Span<byte>(output, answer.Length));
                return answer.Length;
            }
            catch (Exception) { return -1; }
        }
    }
}
