using System;
using System.IO;

namespace EmuSen.Galaxia.Library
{
    // The space on the disk a tool would write to, for Packages' and Cores' meters - see EmuSen_Packages.md §3b.
    public static class DiskSpace
    {
        // The disk under the nearest folder of the path that exists; null where none can be read.
        public static (long Free, long Total)? Under(string path)
        {
            try
            {
                string? at = Path.GetFullPath(path);
                while (at is not null && !Directory.Exists(at)) at = Path.GetDirectoryName(at);
                if (at is null) return null;
                var drive = new DriveInfo(at);
                return (drive.AvailableFreeSpace, drive.TotalSize);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or ArgumentException) { return null; }
        }
    }
}
