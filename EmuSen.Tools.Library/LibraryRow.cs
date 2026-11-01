using System.Collections.Generic;

namespace EmuSen.Tools.Library
{
    // What the library manager found of a file: the same dump as another, the same game in another dump, a failed check, or a hint that ScreenScraper did not know it.
    public enum Finding { SameDump, SameGame, Broken, Unrecognised }

    // One failed check of `rom check`, by its name and the tool's words.
    public sealed record FailedCheck(string Check, string Words);

    // One finding of one file, a row of the library manager - see EmuSen_LibraryTools.md §3 and §10.
    public sealed record LibraryRow
    {
        // The finding and the file's path in the library, so a file found twice has a row for each finding.
        public string Key => $"{Kind}:{Source}";

        public required Finding Kind { get; init; }

        // The file's path in the library with / between its parts.
        public required string Source { get; init; }

        // The file's full path, as games.db names it.
        public required string FullPath { get; init; }

        public required string Title { get; init; }

        public string Console { get; init; } = "";

        // The group's number within its finding, from 1; 0 for a finding of one file.
        public int Group { get; init; }

        // The other files of the group, by their paths in the library.
        public IReadOnlyList<string> Others { get; init; } = [];

        // The package that claims the file, by its folder under Packages; null where none does.
        public string? Package { get; init; }

        // ScreenScraper's game id and name, for a file it recognised.
        public long? GameId { get; init; }

        public string? GameName { get; init; }

        public IReadOnlyList<FailedCheck> Failed { get; init; } = [];

        public long Size { get; init; }

        public bool Hidden { get; init; }
    }
}
