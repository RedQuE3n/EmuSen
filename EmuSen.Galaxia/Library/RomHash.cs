using System;
using System.IO;
using System.Security.Cryptography;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // A ROM file's identity across a rename: the MD5 of every byte, header included - see EmuSen_Galaxia.md §5.4.
    public static class RomHash
    {
        public static string Md5(string path) => GalaxiaNative.Active ? GalaxiaNative.RomMd5(path) : Managed.Md5(path);

        // The C# rule: the default, and what the library's is held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static string Md5(string path)
            {
                using FileStream stream = new(path, FileMode.Open, FileAccess.Read, FileShare.Read, 1 << 16, FileOptions.SequentialScan);
                return Convert.ToHexStringLower(MD5.HashData(stream));
            }
        }
    }
}
