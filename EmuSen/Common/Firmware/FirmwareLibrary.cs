using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;

namespace EmuSen.Common.Firmware
{
    // home/Firmware, as a core-agnostic store: what is installed, what is missing, and how a file the - see EmuSen_Firmware.md §2.
    public static class FirmwareLibrary
    {
        private static string? _directoryOverride;

        // Settable so a test can point at a temp folder instead of the real one; null means the sandbox's own.
        public static string Directory
        {
            get => _directoryOverride ?? DataStore.Firmware;
            set => _directoryOverride = value;
        }

        public static void ResetDirectory() => _directoryOverride = null;

        public static string PathFor(FirmwareRequest request) => Path.Combine(Directory, request.FileName);

        // The image's bytes, or null if it isn't installed - see EmuSen_Firmware.md §2.1.
        public static byte[]? TryLoad(FirmwareRequest request)
        {
            foreach (string name in new[] { request.FileName }.Concat(request.AlternateNames))
            {
                byte[]? bytes = TryReadExact(Path.Combine(Directory, name), request.Size);
                if (bytes != null) return bytes;
            }
            return null;
        }

        public static bool IsInstalled(FirmwareRequest request) => TryLoad(request) != null || TryLoadParts(request) != null;

        // The first split form whose files are all present and together the whole's size (any size where the request gives none), part by part.
        public static IReadOnlyList<byte[]>? TryLoadParts(FirmwareRequest request)
        {
            foreach (var form in request.Parts)
            {
                if (form.Count < 2) continue;
                var parts = form.Select(name => TryRead(Path.Combine(Directory, name))).ToArray();
                if (parts.Any(p => p is null)) continue;
                if (request.Size > 0 && parts.Sum(p => p!.Length) != request.Size) continue;
                return parts.Select(p => p!).ToArray();
            }
            return null;
        }

        public static IReadOnlyList<FirmwareRequest> MissingFrom(IEnumerable<FirmwareRequest> requests) =>
            requests.Where(r => !IsInstalled(r)).ToArray();

        // Copies a file the player picked into the library under its canonical name, so nothing has to ask again.
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

        private static byte[]? TryRead(string path)
        {
            try
            {
                return File.Exists(path) ? File.ReadAllBytes(path) : null;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return null;
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
