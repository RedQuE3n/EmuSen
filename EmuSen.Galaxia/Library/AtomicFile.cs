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
        public static byte[]? TryRead(string path) => GalaxiaNative.Active ? GalaxiaNative.FileRead(path) : Managed.TryRead(path);

        // False when the write failed; best-effort by design - see EmuSen_Galaxia.md §4. A null array is the C#'s to refuse.
        public static bool Write(string path, byte[] contents) =>
            GalaxiaNative.Active && contents is not null ? GalaxiaNative.FileWrite(path, contents) : Managed.Write(path, contents!);

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static byte[]? TryRead(string path)
            {
                try
                {
                    return File.Exists(path) ? File.ReadAllBytes(path) : null;
                }
                catch (Exception ex)
                {
                    ConfigDiagnostics.Report($"{path}: {ex.Message} Treating it as absent.");
                    return null;
                }
            }

            public static bool Write(string path, byte[] contents)
            {
                try
                {
                    string? directory = Path.GetDirectoryName(path);
                    if (!string.IsNullOrEmpty(directory)) Directory.CreateDirectory(directory);

                    // Full write then rename, so an interrupt cannot truncate the live file - see §4.1.
                    string temp = path + TempSuffix;
                    File.WriteAllBytes(temp, contents);
                    File.Move(temp, path, overwrite: true);
                    return true;
                }
                catch (Exception ex)
                {
                    ConfigDiagnostics.Report($"{path}: {ex.Message} The previous file is unchanged.");
                    return false;
                }
            }
        }
    }
}
