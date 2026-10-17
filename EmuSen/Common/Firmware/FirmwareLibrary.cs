using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Common.Firmware
{
    // One request found: the folder it was in, and its bytes whole or as the parts of a split form, in order.
    public sealed record FirmwareFound(string Folder, IReadOnlyList<string> Names, IReadOnlyList<byte[]> Parts)
    {
        public bool Whole => Parts.Count == 1;
    }

    // home/Firmware, and the player's chosen folder before it, as a core-agnostic store of firmware images - see EmuSen_Firmware.md §2.
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

        // The folders searched, first to last: a test's override alone, else SettingFolders.Firmware's order - see EmuSen_Firmware.md §2.3.
        public static IReadOnlyList<string> Folders => _directoryOverride is { } only ? [only] : SettingFolders.Firmware(AppSettings.Load());

        // Where Install writes: home/Firmware, never the player's chosen folder.
        public static string PathFor(FirmwareRequest request) => Path.Combine(Directory, request.FileName);

        // The first folder holding the image, whole by any of its names or as a complete split form - see EmuSen_Firmware.md §2.3.
        public static FirmwareFound? Find(FirmwareRequest request) => Find(request, Folders);

        public static FirmwareFound? Find(FirmwareRequest request, IReadOnlyList<string> folders)
        {
            foreach (string folder in folders)
            {
                foreach (string name in Names(request))
                    if (TryReadExact(Path.Combine(folder, name), request.Size) is { } bytes) return new FirmwareFound(folder, [name], [bytes]);
                if (PartsIn(request, folder) is { } parts) return parts;
            }
            return null;
        }

        // The image whole where Find takes it whole, else null - see EmuSen_Firmware.md §2.1.
        public static byte[]? TryLoad(FirmwareRequest request) => Find(request) is { Whole: true } found ? found.Parts[0] : null;

        public static bool IsInstalled(FirmwareRequest request) => Find(request) != null;

        // The parts where Find takes a split form, all from one folder and together the whole's size (any size where the request gives none), else null.
        public static IReadOnlyList<byte[]>? TryLoadParts(FirmwareRequest request) => Find(request) is { Whole: false } found ? found.Parts : null;

        private static IEnumerable<string> Names(FirmwareRequest request) => new[] { request.FileName }.Concat(request.AlternateNames);

        private static FirmwareFound? PartsIn(FirmwareRequest request, string folder)
        {
            foreach (var form in request.Parts)
            {
                if (form.Count < 2) continue;
                var parts = form.Select(name => TryRead(Path.Combine(folder, name))).ToArray();
                if (parts.Any(p => p is null)) continue;
                if (request.Size > 0 && parts.Sum(p => p!.Length) != request.Size) continue;
                return new FirmwareFound(folder, form.ToArray(), parts.Select(p => p!).ToArray());
            }
            return null;
        }

        public static IReadOnlyList<FirmwareRequest> MissingFrom(IEnumerable<FirmwareRequest> requests)
        {
            IReadOnlyList<string> folders = Folders;
            return requests.Where(r => Find(r, folders) is null).ToArray();
        }

        // Copies a file the player picked into home/Firmware under its canonical name, so nothing has to ask again.
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
