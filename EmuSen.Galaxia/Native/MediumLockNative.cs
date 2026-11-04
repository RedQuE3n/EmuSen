using System;
using System.Text;

namespace EmuSen.Galaxia.Native
{
    // MediumLock's calls into emusen-galaxia's medium lock; a lone surrogate crosses as U+FFFD - see EmuSen_Backend.md §15.2, §40.2.
    internal static unsafe class MediumLockNative
    {
        private const int Absent = -1030;

        // The three exports, the library refused in words where it lacks one.
        private static readonly Lazy<(nint Take, nint Release, nint Holder)> Exports = new(() =>
        {
            var found = (PlatformLibrary.Export("emusen_galaxia_medium_lock_take"), PlatformLibrary.Export("emusen_galaxia_medium_lock_release"), PlatformLibrary.Export("emusen_galaxia_medium_lock_holder"));
            if (found.Item1 == 0 || found.Item2 == 0 || found.Item3 == 0) throw new DllNotFoundException($"{PlatformLibrary.FileName} has no medium lock ({PlatformLibrary.Report}); Galaxia has had no C# implementation since 2026-11-04.");
            return found;
        });

        public static ulong Take(string path, string holder, bool shared)
        {
            byte[] p = Encoding.UTF8.GetBytes(path), h = Encoding.UTF8.GetBytes(holder);
            fixed (byte* pp = p)
            fixed (byte* hp = h)
                return ((delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int, ulong>)Exports.Value.Take)(pp, (nuint)p.Length, hp, (nuint)h.Length, shared ? 1 : 0);
        }

        public static void Release(ulong handle) => ((delegate* unmanaged[Cdecl]<ulong, int>)Exports.Value.Release)(handle);

        public static string? Holder(string path)
        {
            byte[] p = Encoding.UTF8.GetBytes(path);
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)Exports.Value.Holder;
            byte[] buffer = new byte[256];
            long length;
            fixed (byte* pp = p)
            fixed (byte* b = buffer) length = call(pp, (nuint)p.Length, b, (nuint)buffer.Length);
            if (length == Absent) return null;
            if (length < 0) throw new InvalidOperationException($"{PlatformLibrary.FileName}: the medium lock answered {length}");
            if (length > buffer.Length)
            {
                buffer = new byte[length];
                fixed (byte* pp = p)
                fixed (byte* b = buffer) length = call(pp, (nuint)p.Length, b, (nuint)buffer.Length);
            }
            return Encoding.UTF8.GetString(buffer, 0, (int)length);
        }
    }
}
