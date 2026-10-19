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

        public static CrystalImpl Native { get; } = new("Rust", PlatformLibrary.Export);

        private nint Fn(string name) => _export("emusen_crystal_" + name) is var f and not 0 ? f : throw new InvalidOperationException($"the {Name} implementation has no emusen_crystal_{name}");

        public bool Has(string name) => _export("emusen_crystal_" + name) != 0;

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
                var config = new CrystalConfig { size = (uint)sizeof(CrystalConfig), flags = development ? CrystalAbi.ConfigDevelopment : 0, cores = c, cores_len = (nuint)(json?.Length ?? 0) };
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
