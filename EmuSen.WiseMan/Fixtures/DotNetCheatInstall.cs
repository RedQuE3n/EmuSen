using System;
using System.IO;
using System.IO.Compression;
using System.Security.Cryptography;
using System.Linq;
using EmuSen.Crystal.Cheats;

namespace EmuSen.WiseMan.Fixtures
{
    // The cheat database's install as .NET's own zip reader does it, the C# installer kept as the oracle the Rust is held to since 2026-11-04 - see EmuSen_Backend.md §15.3, §40.1.
    public static class DotNetCheatInstall
    {
        public static CheatDatabaseInstallResult Install(byte[] zip, string targetDirectory)
        {
            Directory.CreateDirectory(targetDirectory);
            string root = Path.TrimEndingDirectorySeparator(Path.GetFullPath(targetDirectory));
            int installed = 0, skipped = 0;
            using var archive = new ZipArchive(new MemoryStream(zip), ZipArchiveMode.Read);
            foreach (ZipArchiveEntry entry in archive.Entries)
            {
                if (entry.FullName.EndsWith('/') || entry.Length == 0 && entry.Name.Length == 0) continue;
                if (!entry.Name.EndsWith(".cht", StringComparison.OrdinalIgnoreCase)) { skipped++; continue; }
                string destination = Path.GetFullPath(Path.Combine(root, entry.FullName));
                if (!destination.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.Ordinal)) { skipped++; continue; }
                try
                {
                    Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                    entry.ExtractToFile(destination, overwrite: true);
                    installed++;
                }
                catch
                {
                    skipped++;
                }
            }
            return new CheatDatabaseInstallResult { Installed = installed, Skipped = skipped };
        }

        // What an install said: its counts, or the exception it threw with its words.
        public static string Said(Func<CheatDatabaseInstallResult> install)
        {
            try { CheatDatabaseInstallResult x = install(); return $"{x.Installed} installed, {x.Skipped} skipped"; }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
        }

        // Every file and folder under a tree with each file's bytes, mode and time; a file left empty is not timed, since a failed entry truncated it.
        public static string Tree(string tree) => !Directory.Exists(tree) ? "no folder" : string.Join("\n", Directory.EnumerateFileSystemEntries(tree, "*", new EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true }).Order(StringComparer.Ordinal).Select(p =>
        {
            string name = Path.GetRelativePath(tree, p);
            var info = new FileInfo(p);
            if (info.LinkTarget is not null) return name + " -> link";
            if (Directory.Exists(p)) return name + "/";
            try { return $"{name} {Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(p)))[..12]} {Convert.ToString((int)File.GetUnixFileMode(p), 8)} {(info.Length == 0 ? "" : File.GetLastWriteTimeUtc(p).ToString("o"))}"; }
            catch (UnauthorizedAccessException) { return name + " unreadable"; }
        }));
    }
}
