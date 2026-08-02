using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Etc;

namespace EmuSen.Common.Firmware
{
    // Usr/Home/Firmware, as a core-agnostic store: what is installed, what is
    // missing, and how a file the user picked gets put there. Knows nothing
    // about any particular console. See EmuSen_Firmware.md §2.
    public static class FirmwareLibrary
    {
        private static string? _directoryOverride;

        // Settable so a test can point at a temp folder instead of the real
        // one; null means the sandbox's own Firmware directory.
        public static string Directory
        {
            get => _directoryOverride ?? DianaOSSandbox.FirmwareDirectory;
            set => _directoryOverride = value;
        }

        public static void ResetDirectory() => _directoryOverride = null;

        public static string PathFor(FirmwareRequest request) => Path.Combine(Directory, request.FileName);

        // The image's bytes, or null if it isn't installed. A file of the
        // wrong size is treated as absent rather than loaded: a truncated or
        // mismatched dump runs as garbage and is far harder to diagnose than
        // a missing one. See EmuSen_Firmware.md §2.1.
        public static byte[]? TryLoad(FirmwareRequest request)
        {
            foreach (string name in new[] { request.FileName }.Concat(request.AlternateNames))
            {
                byte[]? bytes = TryReadExact(Path.Combine(Directory, name), request.Size);
                if (bytes != null) return bytes;
            }
            return null;
        }

        public static bool IsInstalled(FirmwareRequest request) => TryLoad(request) != null;

        public static IReadOnlyList<FirmwareRequest> MissingFrom(IEnumerable<FirmwareRequest> requests) =>
            requests.Where(r => !IsInstalled(r)).ToArray();

        // Copies a file the user picked into the library under its canonical
        // name, so nothing has to ask again. Returns false - without copying
        // anything - if it is not the right size.
        public static bool Install(FirmwareRequest request, string sourcePath)
        {
            byte[]? bytes = TryReadExact(sourcePath, request.Size);
            if (bytes == null) return false;

            try
            {
                System.IO.Directory.CreateDirectory(Directory);
                File.WriteAllBytes(PathFor(request), bytes);
                return true;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                Console.WriteLine($"[Firmware] Could not install {request.FileName}: {ex.Message}");
                return false;
            }
        }

        private static byte[]? TryReadExact(string path, int size)
        {
            try
            {
                if (!File.Exists(path)) return null;
                var info = new FileInfo(path);
                if (info.Length != size) return null;
                return File.ReadAllBytes(path);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return null;
            }
        }
    }
}
