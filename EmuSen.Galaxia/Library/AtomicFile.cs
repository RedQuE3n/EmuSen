using System;
using System.IO;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // Whole-file binary reads and writes that survive an interrupted save - see EmuSen_Galaxia.md §4.
    public static class AtomicFile
    {
        public const string TempSuffix = ".tmp";

        // Null for missing or unreadable - the ConfigFile<T>.Load contract, in bytes.
        public static byte[]? TryRead(string path) => GalaxiaNative.FileRead(path);

        // False when the write failed, or no array or a name holding a NUL was given; best-effort by design - see EmuSen_Galaxia.md §4.
        public static bool Write(string path, byte[] contents) =>
            GalaxiaNative.FileWrite(path, contents);
    }
}
