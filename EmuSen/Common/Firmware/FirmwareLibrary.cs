using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Common.Firmware
{
    // One request found: the folder it was in, and its bytes whole or as the parts of a split form, in order; ByContent when recognised by its MD5, Names then the player's file.
    public sealed record FirmwareFound(string Folder, IReadOnlyList<string> Names, IReadOnlyList<byte[]> Parts, bool ByContent = false)
    {
        public bool Whole => Parts.Count == 1;
    }

    // Where a request's image is, without its bytes.
    public sealed record FirmwareLocated(string Folder, IReadOnlyList<string> Names, bool Whole, bool ByContent = false);

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

        // The folders searched, first to last: a test's override alone, else the order of EmuSen_Firmware.md §2.3, from emusen-firmware under the file services' switch.
        public static IReadOnlyList<string> Folders => _directoryOverride is { } only ? [only] : ChosenFolders(AppSettings.Load());

        private static IReadOnlyList<string> ChosenFolders(AppSettings settings) =>
            (FirmwareNative.Active ? FirmwareNative.Folders(settings.FirmwareDirectory, Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), DataStore.Firmware) : null) ?? Managed.Folders(settings);

        // Where Install writes: home/Firmware, never the player's chosen folder.
        public static string PathFor(FirmwareRequest request) => Path.Combine(Directory, request.FileName);

        // The first folder holding the image, whole by any of its names, as a complete split form, or by its content - see EmuSen_Firmware.md §2.3 and §7.
        public static FirmwareFound? Find(FirmwareRequest request) => Find(request, Folders);

        public static FirmwareFound? Find(FirmwareRequest request, IReadOnlyList<string> folders) =>
            FirmwareNative.Active && FirmwareNative.Takes(request, folders) ? FirmwareNative.Find(request, folders) : Managed.Find(request, folders);

        // Where Find would take the image, unread, and else the first folder with a file of its name and another size, for the overview's row.
        public static (FirmwareLocated? Found, string? WrongSizeIn) Locate(FirmwareRequest request, IReadOnlyList<string> folders) =>
            FirmwareNative.Active && FirmwareNative.Takes(request, folders) ? FirmwareNative.Locate(request, folders) : Managed.Locate(request, folders);

        // The image whole where Find takes it whole, else null - see EmuSen_Firmware.md §2.1.
        public static byte[]? TryLoad(FirmwareRequest request) => Find(request) is { Whole: true } found ? found.Parts[0] : null;

        public static bool IsInstalled(FirmwareRequest request) => Locate(request, Folders).Found != null;

        // The parts where Find takes a split form, all from one folder and together the whole's size (any size where the request gives none), else null.
        public static IReadOnlyList<byte[]>? TryLoadParts(FirmwareRequest request) => Find(request) is { Whole: false } found ? found.Parts : null;

        public static IReadOnlyList<FirmwareRequest> MissingFrom(IEnumerable<FirmwareRequest> requests)
        {
            IReadOnlyList<string> folders = Folders;
            return requests.Where(r => Locate(r, folders).Found is null).ToArray();
        }

        // Copies a file the player picked into home/Firmware under its canonical name, so nothing has to ask again.
        public static bool Install(FirmwareRequest request, string sourcePath, Action<string>? say = null)
        {
            if (!FirmwareNative.Active || !FirmwareNative.Takes(request, [sourcePath, Directory])) return Managed.Install(request, sourcePath, say);
            try { return FirmwareNative.Install(request, sourcePath, Directory); }
            catch (Crystal.Services.ServiceFault fault) when (fault.Status == Crystal.Services.ServiceFault.Io)
            {
                say?.Invoke($"[Firmware] Could not install {request.FileName}: {fault.Message}");
                return false;
            }
        }

        // The C# store: the failover, and what the parity class holds emusen-firmware to - see EmuSen_Backend.md §15.1.
        public static class Managed
        {
            public static IReadOnlyList<string> Folders(AppSettings settings) => SettingFolders.Firmware(settings);

            public static FirmwareFound? Find(FirmwareRequest request, IReadOnlyList<string> folders)
            {
                foreach (string folder in folders)
                {
                    foreach (string name in Names(request))
                        if (TryReadExact(Path.Combine(folder, name), request.Size) is { } bytes) return new FirmwareFound(folder, [name], [bytes]);
                    if (PartsIn(request, folder) is { } parts) return parts;
                    if (ByContent(request, folder) is var (file, path) && TryRead(path) is { } whole) return new FirmwareFound(folder, [file], [whole], true);
                }
                return null;
            }

            public static (FirmwareLocated? Found, string? WrongSizeIn) Locate(FirmwareRequest request, IReadOnlyList<string> folders)
            {
                FirmwareFound? found = Find(request, folders);
                string? wrong = found is null ? folders.FirstOrDefault(folder => LengthOf(Path.Combine(folder, request.FileName)) is { } length && length != request.Size) : null;
                return (found is null ? null : new FirmwareLocated(found.Folder, found.Names, found.Whole, found.ByContent), wrong);
            }

            public static bool Install(FirmwareRequest request, string sourcePath, Action<string>? say = null)
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
                    say?.Invoke($"[Firmware] Could not install {request.FileName}: {ex.Message}");
                    return false;
                }
            }
        }

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

        private static readonly Dictionary<(string, long, DateTime), string> Digests = new();

        // A file's MD5, kept for the life of the process by its path, length and modification time - see EmuSen_Firmware.md §7.6.
        private static string? Md5Of(string path, long length)
        {
            try
            {
                var key = (path, length, File.GetLastWriteTimeUtc(new FileInfo(path).ResolveLinkTarget(returnFinalTarget: true)?.FullName ?? path));
                lock (Digests)
                    if (Digests.TryGetValue(key, out string? known)) return known;
                using FileStream stream = new(path, FileMode.Open, FileAccess.Read, FileShare.Read, 1 << 16, FileOptions.SequentialScan);
                string digest = Convert.ToHexStringLower(System.Security.Cryptography.MD5.HashData(stream));
                lock (Digests) Digests[key] = digest;
                return digest;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return null;
            }
        }

        // The folder's own files of the request's size in ordinal order of name, and the first whose MD5 is listed, the checksums in the request's order - see EmuSen_Firmware.md §7.4.
        private static (string Name, string Path)? ByContent(FirmwareRequest request, string folder)
        {
            if (request.Md5.Count == 0 || request.Size < 0) return null;
            string[] entries;
            try { entries = System.IO.Directory.Exists(folder) ? System.IO.Directory.GetFileSystemEntries(folder, "*", new EnumerationOptions { AttributesToSkip = 0, IgnoreInaccessible = true, MatchType = MatchType.Simple }) : []; }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException) { return null; }
            var candidates = entries.Select(path => (Name: Path.GetFileName(path), Path: path)).OrderBy(c => c.Name, StringComparer.Ordinal)
                .Where(c => LengthOf(c.Path) == request.Size).Select(c => (c.Name, c.Path, Md5: Md5Of(c.Path, request.Size))).ToArray();
            foreach (string want in request.Md5)
                foreach (var c in candidates)
                    if (c.Md5 == want) return (c.Name, c.Path);
            return null;
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

        // A file's length as reading it finds it: a link's final target's, and none for a link to nothing - see EmuSen_Backend.md §15.1.
        private static long? LengthOf(string path)
        {
            try
            {
                if (!File.Exists(path)) return null;
                var info = new FileInfo(path);
                if (info.LinkTarget is null) return info.Length;
                return info.ResolveLinkTarget(returnFinalTarget: true) is FileInfo { Exists: true } target ? target.Length : null;
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
                if (LengthOf(path) != size) return null;
                return File.ReadAllBytes(path);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return null;
            }
        }
    }
}
