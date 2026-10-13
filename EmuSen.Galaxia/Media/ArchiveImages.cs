using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Galaxia.Library;

namespace EmuSen.Galaxia.Media
{
    // A game in a .zip, read for routing without unpacking and unpacked for play into the Unpacked folder, never beside the archive; the folder is kept by size, least recently opened first out, and never while a game holds it - see EmuSen_Galaxia.md §3.7, §3.8.
    public static class ArchiveImages
    {
        public static readonly IReadOnlyList<string> Extensions = new[] { ".zip" };

        // What the folder keeps beside the archive being opened; past it, the least recently opened archives go.
        public const long KeptBytes = 4L << 30;

        // A partial unpacking older than this was left by a process that stopped, and is removed.
        public static readonly TimeSpan PartialAge = TimeSpan.FromHours(1);

        private const string Marker = ".unpacked";

        // How a pruner opens a folder's marker to learn that no game holds it: with no sharing, and on Windows so that it may still delete it.
        private static readonly FileShare Alone = OperatingSystem.IsWindows() ? FileShare.Delete : FileShare.None;

        public static bool IsArchive(string path) => Extensions.Contains(Path.GetExtension(path), StringComparer.OrdinalIgnoreCase);

        // The entry play opens: a cue sheet first, then a lone image of an extension isImage takes, the first by name; null when there is none.
        public static string? ImageEntry(string archivePath, Func<string, bool> isImage)
        {
            using var zip = ZipFile.OpenRead(archivePath);
            return Choose(zip.Entries.Select(e => e.FullName).Where(n => !n.EndsWith('/')).ToArray(), isImage);
        }

        private static string? Choose(IReadOnlyList<string> names, Func<string, bool> isImage)
        {
            var ordered = names.OrderBy(n => n.Count(c => c == '/')).ThenBy(n => n, StringComparer.OrdinalIgnoreCase).ToArray();
            return ordered.FirstOrDefault(ImageSets.IsSheet) ?? ordered.FirstOrDefault(n => !ImageSets.IsPlaylist(n) && isImage(Path.GetExtension(n)));
        }

        // An entry's first bytes, at most length of them, inflated without unpacking the archive.
        public static byte[] ReadEntry(string archivePath, string entry, int length)
        {
            using var zip = ZipFile.OpenRead(archivePath);
            var e = zip.GetEntry(entry) ?? zip.Entries.FirstOrDefault(x => string.Equals(x.FullName, entry, StringComparison.OrdinalIgnoreCase))
                ?? throw new FileNotFoundException($"{Path.GetFileName(archivePath)} holds no {entry}");
            using var stream = e.Open();
            var buffer = new byte[Math.Min(length, e.Length)];
            int read = 0;
            while (read < buffer.Length && stream.Read(buffer, read, buffer.Length - read) is var n && n > 0) read += n;
            return read == buffer.Length ? buffer : buffer[..read];
        }

        // The folder an archive unpacks to, named by its full path, length and time, so that a changed archive unpacks anew.
        public static string FolderFor(string archivePath)
        {
            var info = new FileInfo(archivePath);
            byte[] key = SHA256.HashData(Encoding.UTF8.GetBytes($"{info.FullName}|{info.Length}|{info.LastWriteTimeUtc.Ticks}"));
            return Path.Combine(DataStore.Unpacked, Convert.ToHexString(key, 0, 8).ToLowerInvariant());
        }

        // The archive unpacked, once, with its image named by the archive's stem so that saves and states follow the archive; the image's path.
        public static string Unpack(string archivePath, Func<string, bool> isImage)
        {
            string folder = FolderFor(archivePath);
            if (!File.Exists(Path.Combine(folder, Marker)))
            {
                // A folder without its marker is one a pruner is taking away or a stopped one left: never used, so cleared first.
                if (Directory.Exists(folder)) TryDelete(new DirectoryInfo(folder));
                string partial = $"{folder}.partial-{Environment.ProcessId}";
                if (Directory.Exists(partial)) Directory.Delete(partial, recursive: true);
                Directory.CreateDirectory(partial);
                using (var zip = ZipFile.OpenRead(archivePath))
                {
                    string chosen = Choose(zip.Entries.Select(e => e.FullName).Where(n => !n.EndsWith('/')).ToArray(), isImage)
                        ?? throw new InvalidDataException($"{Path.GetFileName(archivePath)} holds no image a console here opens.");
                    string root = Path.GetFullPath(partial) + Path.DirectorySeparatorChar;
                    foreach (var entry in zip.Entries.Where(e => !e.FullName.EndsWith('/')))
                    {
                        string name = Renamed(entry.FullName, chosen, archivePath);
                        string target = Path.GetFullPath(Path.Combine(partial, name));
                        if (!target.StartsWith(root, StringComparison.Ordinal)) continue;
                        Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                        entry.ExtractToFile(target, overwrite: true);
                    }
                    File.WriteAllText(Path.Combine(partial, Marker), $"{Path.GetFullPath(archivePath)}\n{Renamed(chosen, chosen, archivePath)}\n");
                }
                try { Directory.Move(partial, folder); }
                catch (IOException) when (File.Exists(Path.Combine(folder, Marker))) { Directory.Delete(partial, recursive: true); }
            }
            Directory.SetLastWriteTimeUtc(folder, DateTime.UtcNow);
            Prune(keep: folder);
            return Path.Combine(folder, File.ReadAllLines(Path.Combine(folder, Marker))[1]);
        }

        // The chosen image and an .sbi of its stem take the archive's stem; every other entry keeps its name, as a sheet names it.
        private static string Renamed(string entry, string chosen, string archivePath)
        {
            string stem = Path.GetFileNameWithoutExtension(archivePath), dir = Path.GetDirectoryName(chosen) ?? "";
            if (entry == chosen) return Path.Combine(dir, stem + Path.GetExtension(chosen));
            bool sbi = Path.GetExtension(entry).Equals(".sbi", StringComparison.OrdinalIgnoreCase);
            return sbi && Path.ChangeExtension(entry, null) == Path.ChangeExtension(chosen, null) ? Path.Combine(dir, stem + ".sbi") : entry;
        }

        // A hold on the unpacked folder a file is in, so that Prune leaves it while a game has its files mapped; null for a file that is in none.
        public static IDisposable? Hold(string path)
        {
            string root = Path.GetFullPath(DataStore.Unpacked) + Path.DirectorySeparatorChar, full = Path.GetFullPath(path);
            if (!full.StartsWith(root, StringComparison.Ordinal)) return null;
            string folder = Path.Combine(root, full[root.Length..].Split(Path.DirectorySeparatorChar)[0]);
            return new FileStream(Path.Combine(folder, Marker), FileMode.Open, FileAccess.Read, FileShare.Read);
        }

        // The folders past KeptBytes, least recently opened first, and partial ones a stopped process left; keep and a folder a game holds are never removed.
        public static void Prune(string? keep = null, long keptBytes = KeptBytes)
        {
            if (!Directory.Exists(DataStore.Unpacked)) return;
            keep = keep is null ? null : Path.GetFullPath(keep);
            var folders = new DirectoryInfo(DataStore.Unpacked).EnumerateDirectories().ToList();
            foreach (var partial in folders.Where(d => d.Name.Contains(".partial-") && DateTime.UtcNow - d.LastWriteTimeUtc > PartialAge)) TryDelete(partial);
            long total = 0;
            foreach (var d in folders.Where(d => !d.Name.Contains(".partial-")).OrderByDescending(d => d.FullName == keep).ThenByDescending(d => d.LastWriteTimeUtc))
            {
                total += d.EnumerateFiles("*", SearchOption.AllDirectories).Sum(f => f.Length);
                if (total > keptBytes && d.FullName != keep) TryRemove(d);
            }
        }

        // An unpacked folder removed unless a game holds it: its marker opened alone, which a hold refuses, and deleted first, so that the folder is never used half gone.
        private static void TryRemove(DirectoryInfo d)
        {
            try
            {
                string marker = Path.Combine(d.FullName, Marker);
                if (File.Exists(marker))
                {
                    using var alone = new FileStream(marker, FileMode.Open, FileAccess.Read, Alone);
                    File.Delete(marker);
                }
                d.Delete(recursive: true);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static void TryDelete(DirectoryInfo d)
        {
            try { d.Delete(recursive: true); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }
    }
}
