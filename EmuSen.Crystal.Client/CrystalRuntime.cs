using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client.Contract;

namespace EmuSen.Crystal.Client
{
    // A status and the words for it, as the API returned them.
    public sealed class CrystalException(int status, string words) : Exception(words)
    {
        public int Status { get; } = status;
    }

    // What a session opens: a game, or a console started empty, and how - see emusen_crystal.h's open.
    public sealed record CrystalOpen
    {
        public string? Path { get; init; }
        public string? Console { get; init; }
        public string? Engine { get; init; }
        public bool NoBattery { get; init; }
        public bool SingleRows { get; init; }
        public int AudioBufferMaxSamples { get; init; }

        // Since 1.1: the presenter's Vulkan device, the platform library's emusen_serenity_device, lent to each machine; zero lends none.
        public nint Device { get; init; }
    }

    // What a runtime is told when it is made, since 1.2: its cores directories, absent for the defaults, and whether cores in development are listed.
    public sealed record CrystalRuntimeOptions
    {
        public IReadOnlyList<string>? Cores { get; init; }
        public bool Development { get; init; }
    }

    // A copy-out call as the contract layer makes one: the whole length returned, written only when the buffer holds it.
    public unsafe delegate long CrystalCopyOut(byte* out_, nuint len);

    // The runtime: the systems, the engines, a file's route, and the sessions it opens - see EmuSen_Backend.md §3.9 and §14.
    public sealed unsafe class CrystalRuntime : IDisposable
    {
        public CrystalRuntime() : this(null) { }

        public CrystalRuntime(CrystalRuntimeOptions? options)
        {
            byte[]? cores = options?.Cores is { } dirs ? JsonSerializer.SerializeToUtf8Bytes(dirs) : null;
            fixed (byte* c = cores)
            {
                var config = new CrystalConfig
                {
                    size = (uint)sizeof(CrystalConfig),
                    flags = options?.Development == true ? CrystalAbi.ConfigDevelopment : 0,
                    sqlite = Contract.Crystal.Native ? (void*)SqliteLend.Handle : null,
                    cores = c,
                    cores_len = (nuint)(cores?.Length ?? 0),
                };
                Handle = Contract.Crystal.New(&config);
            }
            if (Handle == 0) throw Failure(CrystalSessionAbi.Null);
        }

        public nint Handle { get; private set; }

        // Whether the platform library serves the API, and why the implementation is the one it is.
        public static bool Native => Contract.Crystal.Native;

        public static string Implementation => Contract.Crystal.Report;

        public static string VersionText => Encoding.UTF8.GetString(Bytes(Contract.Crystal.VersionText));

        // The words of the calling thread's last failure, with its status.
        public static CrystalException Failure(long status) =>
            new((int)status, Encoding.UTF8.GetString(Bytes((o, n) => Contract.Crystal.LastError(0, o, n))));

        public static long Check(long result) => result < 0 ? throw Failure(result) : result;

        // A copy-out call's bytes: asked their length, then copied, again if they grew between the two.
        public static byte[] Bytes(CrystalCopyOut call)
        {
            while (true)
            {
                long n = Check(call(null, 0));
                var bytes = new byte[n];
                fixed (byte* b = bytes)
                {
                    long written = Check(call(b, (nuint)bytes.Length));
                    if (written <= n) return written == n ? bytes : bytes[..(int)written];
                }
            }
        }

        public static string Text(CrystalCopyOut call) => Encoding.UTF8.GetString(Bytes(call));

        // A change's answer from one call into a buffer that holds any answer a change gives, so the change is made once.
        public static string Once(CrystalCopyOut call)
        {
            var buffer = new byte[1 << 16];
            fixed (byte* b = buffer)
            {
                long n = Check(call(b, (nuint)buffer.Length));
                return n <= buffer.Length ? Encoding.UTF8.GetString(buffer, 0, (int)n) : throw new InvalidOperationException($"a change's answer of {n} bytes did not fit the {buffer.Length} it was given; the change was made");
            }
        }

        public static JsonDocument Json(CrystalCopyOut call) => JsonDocument.Parse(Bytes(call));

        // Text as the contract takes it: UTF-8 bytes pinned for one call; null is the absent string.
        public static T WithText<T>(string? text, Func<nint, nuint, T> call)
        {
            if (text is null) return call(0, 0);
            byte[] bytes = Encoding.UTF8.GetBytes(text);
            fixed (byte* b = bytes) return call((nint)b, (nuint)bytes.Length);
        }

        public string Systems() => Text((o, n) => Contract.Crystal.Systems(Handle, o, n));

        public string Shelves() => Text((o, n) => Contract.Crystal.Shelves(Handle, o, n));

        public string Engines() => Text((o, n) => Contract.Crystal.Engines(Handle, o, n));

        public string Cores() => Text((o, n) => Contract.Crystal.Cores(Handle, o, n));

        public void Rescan() => Check(Contract.Crystal.Rescan(Handle));

        public string Route(string path) => WithText(path, (p, pl) => Text((o, n) => Contract.Crystal.Route(Handle, (byte*)p, pl, o, n)));

        public string Sessions() => Text((o, n) => Contract.Crystal.Sessions(Handle, o, n));

        public CrystalSession Open(CrystalOpen open)
        {
            byte[]? path = open.Path is null ? null : Encoding.UTF8.GetBytes(open.Path);
            byte[]? console = open.Console is null ? null : Encoding.UTF8.GetBytes(open.Console);
            byte[]? engine = open.Engine is null ? null : Encoding.UTF8.GetBytes(open.Engine);
            fixed (byte* p = path, c = console, e = engine)
            {
                var parameters = new CrystalOpenParams
                {
                    size = (uint)sizeof(CrystalOpenParams),
                    flags = (open.NoBattery ? CrystalAbi.OpenNoBattery : 0) | (open.SingleRows ? CrystalAbi.OpenSingleRows : 0),
                    path = path is null ? null : p,
                    path_len = (nuint)(path?.Length ?? 0),
                    console = console is null ? null : c,
                    console_len = (nuint)(console?.Length ?? 0),
                    engine = engine is null ? null : e,
                    engine_len = (nuint)(engine?.Length ?? 0),
                    audio_buffer_max_samples = (uint)Math.Max(0, open.AudioBufferMaxSamples),
                    device = (void*)open.Device,
                };
                return new CrystalSession(this, (uint)Check(Contract.Crystal.Open(Handle, &parameters)));
            }
        }

        // A session opened elsewhere, by the shell or another client, by its id; the id is checked to be open.
        public CrystalSession SessionOf(uint id)
        {
            Text((o, n) => Contract.Crystal.SessionInfo(Handle, id, o, n));
            return new CrystalSession(this, id);
        }

        // Since 1.1: the record of the state at path, or null for none.
        public string? StateRecord(string path) =>
            WithText(path, (p, pl) => Optional(() => Text((o, n) => Contract.Crystal.StateRecord(Handle, (byte*)p, pl, o, n))));

        public void StateForget(string path) => WithText(path, (p, pl) => Check(Contract.Crystal.StateForget(Handle, (byte*)p, pl)));

        // Since 1.1: a game's own value with no session, or null where it holds none.
        public string? GameSetting(string path, string key) =>
            WithText(path, (p, pl) => WithText(key, (k, kl) => Optional(() => Text((o, n) => Contract.Crystal.GameSettingGet(Handle, (byte*)p, pl, (byte*)k, kl, o, n)))));

        public void SetGameSetting(string path, string key, string? value) =>
            WithText(path, (p, pl) => WithText(key, (k, kl) => WithText(value, (v, vl) => Check(Contract.Crystal.GameSettingSet(Handle, (byte*)p, pl, (byte*)k, kl, (byte*)v, vl)))));

        // Since 1.1: a cheat op that needs no game, answered with no session.
        public string CheatOp(uint op, string argsJson) =>
            WithText(argsJson, (a, al) => Text((o, n) => Contract.Crystal.CheatOp(Handle, 0, op, (byte*)a, al, o, n)));

        public string FirmwareNeeded(string path, string? engine = null) =>
            WithText(path, (p, pl) => WithText(engine, (e, el) => Text((o, n) => Contract.Crystal.FirmwareNeeded(Handle, (byte*)p, pl, (byte*)e, el, o, n))));

        public void FirmwareInstall(string requestJson, string source) =>
            WithText(requestJson, (r, rl) => WithText(source, (s, sl) => Check(Contract.Crystal.FirmwareInstall(Handle, (byte*)r, rl, (byte*)s, sl))));

        public string FirmwareOverview() => Text((o, n) => Contract.Crystal.FirmwareOverview(Handle, o, n));

        public string CardsOp(uint op, string argsJson) =>
            WithText(argsJson, (a, al) => op >= CrystalAbi.CardsCopy
                ? Once((o, n) => Contract.Crystal.CardsOp(Handle, op, (byte*)a, al, o, n))
                : Text((o, n) => Contract.Crystal.CardsOp(Handle, op, (byte*)a, al, o, n)));

        public string SettingsSchema(string console, string? engine = null) =>
            WithText(console, (c, cl) => WithText(engine, (e, el) => Text((o, n) => Contract.Crystal.SettingsSchema(Handle, 0, (byte*)c, cl, (byte*)e, el, o, n))));

        public string? Setting(string console, string key) =>
            WithText(console, (c, cl) => WithText(key, (k, kl) => Optional(() => Text((o, n) => Contract.Crystal.SettingsGet(Handle, 0, CrystalAbi.ScopeConsole, (byte*)c, cl, (byte*)k, kl, o, n)))));

        public void SetSetting(string console, string key, string? value) =>
            WithText(console, (c, cl) => WithText(key, (k, kl) => WithText(value, (v, vl) => Check(Contract.Crystal.SettingsSet(Handle, 0, CrystalAbi.ScopeConsole, (byte*)c, cl, (byte*)k, kl, (byte*)v, vl)))));

        public CrystalSubscription Subscribe(uint session = 0, uint kinds = 0)
        {
            nint sub = Contract.Crystal.Subscribe(Handle, session, kinds);
            return sub == 0 ? throw Failure(CrystalSessionAbi.Null) : new CrystalSubscription(sub);
        }

        public bool LogSwitch
        {
            get => Check(Contract.Crystal.LogStatus(Handle)) != 0;
            set => Check(Contract.Crystal.LogSwitch(Handle, value ? 1u : 0u));
        }

        // An answer that may be ABSENT, as null.
        public static T? Optional<T>(Func<T> call) where T : class
        {
            try { return call(); }
            catch (CrystalException e) when (e.Status == CrystalSessionAbi.Absent) { return null; }
        }

        public void Dispose()
        {
            if (Handle == 0) return;
            Contract.Crystal.Free(Handle);
            Handle = 0;
        }
    }
}
