using System;
using System.Text;
using EmuSen.Galaxia.Native;

namespace EmuSen.Mistress.Library
{
    // The bytes OpenVGDB hashed for a system's file, the systems table's rule read through the platform library's C interface; the file itself where the rule keeps it whole - see EmuSen_Backend.md §14.13.
    public static unsafe class OpenVgdbBytes
    {
        public static byte[] Of(string? system, byte[] file)
        {
            nint export = system is null ? 0 : PlatformLibrary.Export("emusen_systems_openvgdb");
            if (export == 0) return file;
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)export;
            byte[] id = Encoding.UTF8.GetBytes(system!);
            byte[] output = new byte[Math.Max(1, file.Length)];
            for (int attempt = 0; attempt < 2; attempt++)
            {
                long written;
                fixed (byte* i = id)
                fixed (byte* f = file)
                fixed (byte* o = output)
                    written = call(i, (nuint)id.Length, f, (nuint)file.Length, o, (nuint)output.Length);
                if (written < 0) return file;
                if (written <= output.Length) return output.AsSpan(0, (int)written).ToArray();
                output = new byte[written];
            }
            return file;
        }
    }
}
