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

        // Which console this file belongs to, decided by extension - see EmuSen_Multicore.md §10.
        public string CoreDisplayName =>
            EmuSen.Cores.CoreCatalog.ByExtension(Path.GetExtension(FullPath))?.DisplayName ?? "Unknown";
    }

    // CoreDisplayName is the console the scan was narrowed to, null when it was not.
    public sealed record RomLibraryResult(RomLibraryStatus Status, string? Directory, IReadOnlyList<RomEntry> Entries,
        string? CoreDisplayName = null);

    // Enumerates playable ROMs in a directory, with no Avalonia dependency
    // so it stays testable headlessly - see EmuSen_Settings_Reference.md §4.11.
    public static class RomLibrary
    {
        // Whatever cores this build has, not a second copy of the list - see EmuSen_Multicore.md §3.
        public static readonly string[] Extensions = EmuSen.Cores.CoreCatalog.RomExtensions.ToArray();

        private static readonly IReadOnlyList<RomEntry> None = Array.Empty<RomEntry>();

        // Recurses, so one root over per-console subfolders works - see EmuSen_Multicore.md §10.
        private static readonly EnumerationOptions Recursive = new()
        {
            RecurseSubdirectories = true,
            IgnoreInaccessible = true,
            AttributesToSkip = FileAttributes.Hidden | FileAttributes.System,
        };

        // <coreDisplayName> narrows to one console; null or CoreCatalog.AllConsoles keeps everything.
        public static RomLibraryResult Scan(string? romDirectory, string? coreDisplayName = null)
        {
            if (string.IsNullOrWhiteSpace(romDirectory))
            {
                return new RomLibraryResult(RomLibraryStatus.NoDirectoryConfigured, null, None, Narrowed(coreDisplayName));
            }

            if (!Directory.Exists(romDirectory))
            {
                return new RomLibraryResult(RomLibraryStatus.DirectoryNotFound, romDirectory, None, Narrowed(coreDisplayName));
            }

            List<RomEntry> entries;
            try
            {
                var only = EmuSen.Cores.CoreCatalog.ByDisplayName(coreDisplayName);

                entries = Directory.EnumerateFiles(romDirectory, "*", Recursive)
                    .Where(f => only is null
                        ? Extensions.Contains(Path.GetExtension(f), StringComparer.OrdinalIgnoreCase)
                        : only.SupportsExtension(Path.GetExtension(f)))
                    .Select(f => new RomEntry(f))
                    .OrderBy(e => e.Title, StringComparer.OrdinalIgnoreCase)
                    .ToList();
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                // An unreadable directory is the same outcome as a missing one, from here.
                return new RomLibraryResult(RomLibraryStatus.DirectoryNotFound, romDirectory, None, Narrowed(coreDisplayName));
            }

            RomLibraryStatus status = entries.Count == 0 ? RomLibraryStatus.Empty : RomLibraryStatus.Ok;
            return new RomLibraryResult(status, romDirectory, entries, Narrowed(coreDisplayName));
        }

        // The display name only when it names a real core in this build.
        private static string? Narrowed(string? coreDisplayName) =>
            EmuSen.Cores.CoreCatalog.ByDisplayName(coreDisplayName)?.DisplayName;

        // The one place the "why is my list empty" wording lives, so the
        // inline library and the modal browser can't drift apart.
        public static string DescribeEmpty(RomLibraryResult result) => result.Status switch
        {
            RomLibraryStatus.NoDirectoryConfigured => "No ROM directory set - choose one in Settings > Preferences...",
            RomLibraryStatus.DirectoryNotFound => $"ROM directory not found: {result.Directory}",
            RomLibraryStatus.Empty when result.CoreDisplayName is { } console =>
                $"No {console} games under {result.Directory}",
            RomLibraryStatus.Empty => $"No {string.Join(" or ", Extensions)} files under {result.Directory}",
            _ => string.Empty
        };
    }
}
