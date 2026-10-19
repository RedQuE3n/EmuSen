using System;
using System.Collections.Generic;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Managed;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Crystal
{
    // One implementation of the frontend API reached by its exports' names, the C# one or the platform library's, so the parity class drives both through the C ABI whatever the binding's switch chose - see EmuSen_Backend.md §29.7.
    public sealed unsafe class CrystalImpl
    {
        private readonly Func<string, nint> _export;

        private CrystalImpl(string name, Func<string, nint> export)
        {
            Name = name;
            _export = export;
        }

        public string Name { get; }

        public static CrystalImpl Managed { get; } = new("C#", ManagedCrystal.Export);

        // The e_sqlite3 the C# loads, lent to each runtime as a program lends it, so both keep states.db.
        private static readonly Lazy<nint> LentSqlite = new(() =>
        {
            SQLitePCL.Batteries_V2.Init();
            return System.Runtime.InteropServices.NativeLibrary.TryLoad(SQLitePCL.raw.GetNativeLibraryName(), typeof(SQLitePCL.SQLite3Provider_e_sqlite3).Assembly, null, out nint handle) ? handle : 0;
        });

        public static CrystalImpl Native { get; } = new("Rust", PlatformLibrary.Export);

        private nint Fn(string name) => _export("emusen_crystal_" + name) is var f and not 0 ? f : throw new InvalidOperationException($"the {Name} implementation has no emusen_crystal_{name}");

        public bool Has(string name) => _export("emusen_crystal_" + name) != 0;

        public nint Export(string name) => Fn(name);

        public string LastError()
        {
            var f = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)Fn("last_error");
            long n = f(0, null, 0);
            var bytes = new byte[n];
            fixed (byte* b = bytes) f(0, b, (nuint)n);
            return Encoding.UTF8.GetString(bytes);
        }

        // A runtime of this implementation over the cores given, or the defaults; the status and words where it is refused.
        public nint New(IReadOnlyList<string>? cores = null, bool development = false)
        {
            byte[]? json = cores is null ? null : JsonSerializer.SerializeToUtf8Bytes(cores);
            fixed (byte* c = json)
            {
                var config = new CrystalConfig { size = (uint)sizeof(CrystalConfig), flags = development ? CrystalAbi.ConfigDevelopment : 0, cores = c, cores_len = (nuint)(json?.Length ?? 0), sqlite = (void*)LentSqlite.Value };
                nint rt = ((delegate* unmanaged[Cdecl]<CrystalConfig*, nint>)Fn("new"))(&config);
                return rt != 0 ? rt : throw new InvalidOperationException($"{Name}: {LastError()}");
            }
        }

        public int Free(nint rt) => ((delegate* unmanaged[Cdecl]<nint, int>)Fn("free"))(rt);

        public int Rescan(nint rt) => ((delegate* unmanaged[Cdecl]<nint, int>)Fn("rescan"))(rt);

        // A runtime-level copy-out read, its answer or "status <n>: <words>" where it failed.
        public string Read(string name, nint rt)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)Fn(name);
            return Copy((o, n) => f(rt, o, n));
        }

        public string Route(nint rt, string? path)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)Fn("route");
            byte[]? p = path is null ? null : Encoding.UTF8.GetBytes(path);
            fixed (byte* pp = p)
            {
                byte* at = pp;
                return Copy((o, n) => f(rt, at, (nuint)(p?.Length ?? 0), o, n));
            }
        }

        public string FirmwareNeeded(nint rt, string path, string? engine)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, byte*, nuint, long>)Fn("firmware_needed");
            byte[] p = Encoding.UTF8.GetBytes(path);
            byte[]? e = engine is null ? null : Encoding.UTF8.GetBytes(engine);
            fixed (byte* pp = p)
            fixed (byte* ep = e)
            {
                byte* a = pp;
                byte* b = ep;
                return Copy((o, n) => f(rt, a, (nuint)p.Length, b, (nuint)(e?.Length ?? 0), o, n));
            }
        }

        // A firmware install's status, with the words where it failed.
        public string FirmwareInstall(nint rt, string request, string source)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, int>)Fn("firmware_install");
            byte[] r = Encoding.UTF8.GetBytes(request), s = Encoding.UTF8.GetBytes(source);
            fixed (byte* rp = r)
            fixed (byte* sp = s)
            {
                int status = f(rt, rp, (nuint)r.Length, sp, (nuint)s.Length);
                return status == 0 ? "0" : $"status {status}: {LastError()}";
            }
        }

        // cards_op's answer, made with one call into a buffer that holds it, so a change is made once.
        public string CardsOp(nint rt, uint op, string args)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, byte*, nuint, long>)Fn("cards_op");
            byte[] a = Encoding.UTF8.GetBytes(args);
            var buffer = new byte[1 << 16];
            fixed (byte* ap = a)
            fixed (byte* o = buffer)
            {
                long n = f(rt, op, ap, (nuint)a.Length, o, (nuint)buffer.Length);
                return n < 0 ? $"status {n}: {LastError()}" : Encoding.UTF8.GetString(buffer, 0, (int)n);
            }
        }

        // A session opened, its id or "status <n>: <words>"; size 0 gives the struct's own, a smaller one asks for a refusal.
        public string Open(nint rt, string? path, string? console = null, string? engine = null, uint flags = 0, uint size = 0)
        {
            byte[]? p = path is null ? null : Encoding.UTF8.GetBytes(path), c = console is null ? null : Encoding.UTF8.GetBytes(console), e = engine is null ? null : Encoding.UTF8.GetBytes(engine);
            fixed (byte* pp = p, cp = c, ep = e)
            {
                var o = new CrystalOpenParams
                {
                    size = size != 0 ? size : (uint)sizeof(CrystalOpenParams),
                    flags = flags,
                    path = p is null ? null : pp,
                    path_len = (nuint)(p?.Length ?? 0),
                    console = c is null ? null : cp,
                    console_len = (nuint)(c?.Length ?? 0),
                    engine = e is null ? null : ep,
                    engine_len = (nuint)(e?.Length ?? 0),
                };
                long id = ((delegate* unmanaged[Cdecl]<nint, CrystalOpenParams*, long>)Fn("open"))(rt, &o);
                return id < 0 ? $"status {id}: {LastError()}" : id.ToString();
            }
        }

        private string Status(int status) => status == 0 ? "0" : $"status {status}: {LastError()}";

        // close, reset or disc_eject of a session, by the export's name.
        public string Call(string name, nint rt, uint session) => Status(((delegate* unmanaged[Cdecl]<nint, uint, int>)Fn(name))(rt, session));

        // session_info or save_media of a session, by the export's name.
        public string Read(string name, nint rt, uint session)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)Fn(name);
            return Copy((o, n) => f(rt, session, o, n));
        }

        public string DiscInsert(nint rt, uint session, string? path)
        {
            byte[]? p = path is null ? null : Encoding.UTF8.GetBytes(path);
            fixed (byte* pp = p) return Status(((delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)Fn("disc_insert"))(rt, session, pp, (nuint)(p?.Length ?? 0)));
        }

        public string DiscChange(nint rt, uint session, uint index) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, int>)Fn("disc_change"))(rt, session, index));

        public string DiscLid(nint rt, uint session, bool open) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, int>)Fn("disc_lid"))(rt, session, open ? 1u : 0u));

        // A frame's status and every field of its frame_info, the struct given at size bytes (0 for its own).
        public string Frame(nint rt, uint session, uint size = 0)
        {
            var info = new CrystalFrameInfo { size = size != 0 ? size : (uint)sizeof(CrystalFrameInfo), reserved = 0xFEEDu, picture_on_device = 7 };
            int status = ((delegate* unmanaged[Cdecl]<nint, uint, CrystalFrameInfo*, int>)Fn("frame"))(rt, session, &info);
            if (status != 0) return Status(status);
            return $"ran {info.ran} halted {info.halted} rendered {info.rendered} serial {info.picture_serial} {info.width}x{info.height} repeat {info.row_repeat} lines {info.display_lines} aspect {info.display_aspect:R} "
                + $"audio {info.audio_frames}@{info.audio_rate} play {info.play_audio} speed {info.speed_percent} interval {info.frame_interval_ns} total {info.total_frames} hz {info.frame_rate_hz:R} paused {info.paused} "
                + $"at {info.halted_processor}:{info.halted_address} device {info.picture_on_device} reserved {info.reserved}";
        }

        // The picture's length and SHA-256, or the length asked of a buffer len bytes long.
        public string Picture(nint rt, uint session, int len = -1)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, long>)Fn("picture_copy");
            long n = f(rt, session, null, 0);
            if (n < 0) return $"status {n}: {LastError()}";
            var bytes = new byte[len >= 0 ? len : n];
            long again;
            fixed (byte* b = bytes) again = f(rt, session, b, (nuint)bytes.Length);
            if (again < 0) return $"status {again}: {LastError()}";
            return len >= 0 ? $"{n} {again}" : $"{again} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes))}";
        }

        // The sound held, then drained in at most frames frames: the frames, the rate and the samples' SHA-256.
        public string Audio(nint rt, uint session, int frames = 4096)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, uint, short*, nuint, uint*, long>)Fn("audio_drain");
            uint rate = 0, held = 0;
            long buffered = f(rt, session, null, 0, &held);
            var samples = new short[frames * 2];
            long n;
            fixed (short* o = samples) n = f(rt, session, o, (nuint)frames, &rate);
            if (n < 0 || buffered < 0) return $"status {Math.Min(n, buffered)}: {LastError()}";
            if (frames == 0) return $"{buffered}@{held} {n}@{rate} (a null buffer asks what is held)";
            byte[] bytes = System.Runtime.InteropServices.MemoryMarshal.AsBytes(samples.AsSpan(0, (int)n * 2)).ToArray();
            return $"{buffered}@{held} {n}@{rate} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes))}";
        }

        public string Phases(nint rt, uint session)
        {
            long n = ((delegate* unmanaged[Cdecl]<nint, uint, double*, nuint, long>)Fn("phases"))(rt, session, null, 0);
            return n < 0 ? $"status {n}: {LastError()}" : $"{n} {Read("phase_names", rt, session)}";
        }

        public string Wait(nint rt, uint session, uint ms)
        {
            int r = ((delegate* unmanaged[Cdecl]<nint, uint, uint, int>)Fn("wait"))(rt, session, ms);
            return r < 0 ? Status(r) : r.ToString();
        }

        public string Button(nint rt, uint session, uint port, uint control, bool pressed) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, uint, int>)Fn("input_button"))(rt, session, port, control, pressed ? 1u : 0u));

        public string Axis(nint rt, uint session, uint port, uint axis, double value) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, double, int>)Fn("input_axis"))(rt, session, port, axis, value));

        public string Connected(nint rt, uint session, uint port, bool connected) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, int>)Fn("input_connected"))(rt, session, port, connected ? 1u : 0u));

        public string Speed(nint rt, uint session, uint percent, uint turbo, uint slow, uint flags) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, uint, uint, int>)Fn("speed_set"))(rt, session, percent, turbo, slow, flags));

        public string SpeedMode(nint rt, uint session, uint mode) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, int>)Fn("speed_mode"))(rt, session, mode));

        public string RewindSet(nint rt, uint session, bool enabled, uint interval, ulong budget, uint thumbnail) =>
            Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, uint, ulong, uint, int>)Fn("rewind_set"))(rt, session, enabled ? 1u : 0u, interval, budget, thumbnail));

        // A step back of count snapshots: how many were taken and the frame_info it fills.
        public string RewindStep(nint rt, uint session, uint count, uint size = 0)
        {
            var info = new CrystalFrameInfo { size = size != 0 ? size : (uint)sizeof(CrystalFrameInfo), reserved = 0xFEEDu };
            long done = ((delegate* unmanaged[Cdecl]<nint, uint, uint, CrystalFrameInfo*, long>)Fn("rewind_step"))(rt, session, count, &info);
            if (done < 0) return $"status {done}: {LastError()}";
            return $"{done} rendered {info.rendered} serial {info.picture_serial} {info.width}x{info.height} audio {info.audio_frames} total {info.total_frames} speed {info.speed_percent} reserved {info.reserved}";
        }

        public string RewindTo(nint rt, uint session, ulong frame) => Status(((delegate* unmanaged[Cdecl]<nint, uint, ulong, int>)Fn("rewind_to"))(rt, session, frame));

        // A moment's picture: its length and SHA-256.
        public string RewindPicture(nint rt, uint session, ulong frame)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, uint, ulong, byte*, nuint, long>)Fn("rewind_picture");
            long n = f(rt, session, frame, null, 0);
            if (n < 0) return $"status {n}: {LastError()}";
            var bytes = new byte[n];
            long again;
            fixed (byte* b = bytes) again = f(rt, session, frame, b, (nuint)n);
            return again < 0 ? $"status {again}: {LastError()}" : $"{again} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes))}";
        }

        // A runtime-level copy-out read of a path: state_record.
        public string Read(string name, nint rt, string path)
        {
            var f = (delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)Fn(name);
            byte[] p = Encoding.UTF8.GetBytes(path);
            fixed (byte* pp = p)
            {
                byte* at = pp;
                return Copy((o, n) => f(rt, at, (nuint)p.Length, o, n));
            }
        }

        public string SlotCall(string name, nint rt, uint session, uint slot) => Status(((delegate* unmanaged[Cdecl]<nint, uint, uint, int>)Fn(name))(rt, session, slot));

        // state_save or state_load, the twins, at a path or with none at the default.
        public string StateTwin(string name, nint rt, uint session, string? path)
        {
            byte[]? p = path is null ? null : Encoding.UTF8.GetBytes(path);
            fixed (byte* pp = p) return Status(((delegate* unmanaged[Cdecl]<nint, uint, byte*, nuint, int>)Fn(name))(rt, session, pp, (nuint)(p?.Length ?? 0)));
        }

        public string Forget(nint rt, string path)
        {
            byte[] p = Encoding.UTF8.GetBytes(path);
            fixed (byte* pp = p) return Status(((delegate* unmanaged[Cdecl]<nint, byte*, nuint, int>)Fn("state_forget"))(rt, pp, (nuint)p.Length));
        }

        private delegate long CopyOut(byte* o, nuint n);

        private string Copy(CopyOut call)
        {
            long n = call(null, 0);
            if (n < 0) return $"status {n}: {LastError()}";
            var bytes = new byte[n];
            fixed (byte* b = bytes)
            {
                long again = call(b, (nuint)n);
                if (again < 0) return $"status {again}: {LastError()}";
            }
            return Encoding.UTF8.GetString(bytes);
        }
    }
}
