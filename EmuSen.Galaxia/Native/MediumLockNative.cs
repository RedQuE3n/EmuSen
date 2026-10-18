using System;
using System.Text;

namespace EmuSen.Galaxia.Native
{
    // MediumLock's calls into emusen-galaxia's medium lock, under Galaxia's switch; a text that cannot cross keeps the C# table - see EmuSen_Backend.md §15.2.
    internal static unsafe class MediumLockNative
    {
        private const int Absent = -1030;

        private static readonly Lazy<(nint Take, nint Release, nint Holder)> Exports = new(() =>
            (PlatformLibrary.Export("emusen_galaxia_medium_lock_take"), PlatformLibrary.Export("emusen_galaxia_medium_lock_release"), PlatformLibrary.Export("emusen_galaxia_medium_lock_holder")));

        // Whether the library's lock is in use for these texts: Galaxia's switch chose the library, it has the lock, and each text crosses.
        public static bool Takes(string path, string? holder) =>
            GalaxiaNative.Takes(path, holder, listed: true) && Exports.Value.Take != 0 && Exports.Value.Release != 0 && Exports.Value.Holder != 0;

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
