using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Mistress.Library
{
    // Why a ROM directory can produce no list - see EmuSen_Settings_Reference.md §4.11.
    public enum RomLibraryStatus
    {
        NoDirectoryConfigured,
        DirectoryNotFound,
        Empty,
        Ok
    }

    // Title is the filename without extension; FileName keeps it.
    public sealed record RomEntry(string FullPath)
    {
        public string FileName => Path.GetFileName(FullPath);
        public string Title => Path.GetFileNameWithoutExtension(FullPath);
    }

    public sealed record RomLibraryResult(RomLibraryStatus Status, string? Directory, IReadOnlyList<RomEntry> Entries);

    // Enumerates playable ROMs in a directory, with no Avalonia dependency
    // so it stays testable headlessly - see EmuSen_Settings_Reference.md §4.11.
    public static class RomLibrary
    {
        public static readonly string[] Extensions = { ".smc", ".sfc" };

        private static readonly IReadOnlyList<RomEntry> None = Array.Empty<RomEntry>();

        public static RomLibraryResult Scan(string? romDirectory)
        {
            if (string.IsNullOrWhiteSpace(romDirectory))
            {
                return new RomLibraryResult(RomLibraryStatus.NoDirectoryConfigured, null, None);
            }

            if (!Directory.Exists(romDirectory))
            {
                return new RomLibraryResult(RomLibraryStatus.DirectoryNotFound, romDirectory, None);
            }

            List<RomEntry> entries;
            try
            {
                entries = Directory.EnumerateFiles(romDirectory)
                    .Where(f => Extensions.Contains(Path.GetExtension(f), StringComparer.OrdinalIgnoreCase))
                    .Select(f => new RomEntry(f))
                    .OrderBy(e => e.Title, StringComparer.OrdinalIgnoreCase)
                    .ToList();
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                // An unreadable directory is the same outcome as a missing one, from here.
                return new RomLibraryResult(RomLibraryStatus.DirectoryNotFound, romDirectory, None);
            }

            RomLibraryStatus status = entries.Count == 0 ? RomLibraryStatus.Empty : RomLibraryStatus.Ok;
            return new RomLibraryResult(status, romDirectory, entries);
        }

        // The one place the "why is my list empty" wording lives, so the
        // inline library and the modal browser can't drift apart.
        public static string DescribeEmpty(RomLibraryResult result) => result.Status switch
        {
            RomLibraryStatus.NoDirectoryConfigured => "No ROM directory set - choose one in Settings > Preferences...",
            RomLibraryStatus.DirectoryNotFound => $"ROM directory not found: {result.Directory}",
            RomLibraryStatus.Empty => $"No .smc or .sfc files in {result.Directory}",
            _ => string.Empty
        };
    }
}
