using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Services;

namespace EmuSen.Crystal.Cheats
{
    // One .cht file sitting in the player's cheat tree.
    public readonly struct CheatDatabaseEntry
    {
        // The folder it sits in - libretro names these per system.
        public string System { get; init; }

        // The file name without .cht, which is the game's DAT name.
        public string Game { get; init; }

        public string Path { get; init; }
    }

    // An index over the player's own .cht tree, never anything EmuSen ships - see `man cheat`; emusen-cheats' under the file services' switch - see EmuSen_Backend.md §15.3.
    public sealed class CheatDatabase
    {
        public string Directory { get; }

        // A real libretro tree is tens of thousands of files, so one instance scans once.
        private IReadOnlyList<CheatDatabaseEntry>? _all;

        // The library's index of the same tree, made on first use where the library is chosen; null for the C#.
        private readonly Lazy<CheatsNative.Database?> _native;

        public CheatDatabase(string directory) : this(directory, CheatsNative.Active) { }

        private CheatDatabase(string directory, bool native)
        {
            Directory = directory;
            _native = new(() => native && ServiceLibrary.Takes(directory) ? CheatsNative.Database.Open(directory) : null);
        }

        // The C# index alone, the failover and the parity class's oracle; and the library's alone, whatever the switch says.
        public static CheatDatabase Managed(string directory) => new(directory, native: false);

        public static CheatDatabase Library(string directory) => new(directory, native: true);

        // Whether the library answers for this index.
        public bool Native => _native.Value is not null;

        public bool Exists => System.IO.Directory.Exists(Directory);

        public IReadOnlyList<CheatDatabaseEntry> All() => _all ??= _native.Value is { } native ? native.Entries() : Scan();

        private IReadOnlyList<CheatDatabaseEntry> Scan()
        {
            if (!Exists) return Array.Empty<CheatDatabaseEntry>();

            try
            {
                return System.IO.Directory
                    .EnumerateFiles(Directory, "*.cht", SearchOption.AllDirectories)
                    .Select(path => new CheatDatabaseEntry
                    {
                        System = System.IO.Path.GetFileName(System.IO.Path.GetDirectoryName(path)) ?? "",
                        Game = System.IO.Path.GetFileNameWithoutExtension(path),
                        Path = path,
                    })
                    .OrderBy(e => e.System, StringComparer.OrdinalIgnoreCase)
                    .ThenBy(e => e.Game, StringComparer.OrdinalIgnoreCase)
                    .ToList();
            }
            catch
            {
                return Array.Empty<CheatDatabaseEntry>();
            }
        }

        public IReadOnlyList<(string System, int Count)> Systems() =>
            _native.Value is { } native ? native.Systems() :
            All().GroupBy(e => e.System, StringComparer.OrdinalIgnoreCase)
                 .Select(g => (g.Key, g.Count()))
                 .OrderBy(g => g.Key, StringComparer.OrdinalIgnoreCase)
                 .ToList();

        // A system holds thousands, so the filter is not optional in practice.
        public IReadOnlyList<CheatDatabaseEntry> Games(string system, string? filter = null)
        {
            if (_native.Value is { } native && ServiceLibrary.Takes(system) && ServiceLibrary.Takes(filter)) return Places(native.Games(system, filter));

            IEnumerable<CheatDatabaseEntry> games = All().Where(e => string.Equals(e.System, system, StringComparison.OrdinalIgnoreCase));

            if (!string.IsNullOrWhiteSpace(filter))
            {
                string needle = filter.Trim();
                games = games.Where(e => e.Game.Contains(needle, StringComparison.OrdinalIgnoreCase));
            }

            return games.ToList();
        }

        private IReadOnlyList<CheatDatabaseEntry> Places(IReadOnlyList<int> places)
        {
            IReadOnlyList<CheatDatabaseEntry> all = All();
            return places.Select(i => all[i]).ToList();
        }

        // The extension is stripped here, since a ROM is .sfc where its cheat file is .cht.
        public IReadOnlyList<CheatDatabaseEntry> Find(string text, int limit = 20)
        {
            if (_native.Value is { } native && ServiceLibrary.Takes(text ?? "")) return Places(native.Find(text ?? "", limit));

            string needle = Normalize(text!);
            if (needle.Length == 0) return Array.Empty<CheatDatabaseEntry>();

            return All()
                .Select(e => (Entry: e, Score: Score(Normalize(e.Game), needle)))
                .Where(x => x.Score > 0)
                .OrderByDescending(x => x.Score)
                .ThenBy(x => x.Entry.Game, StringComparer.OrdinalIgnoreCase)
                .Take(limit)
                .Select(x => x.Entry)
                .ToList();
        }

        public CheatDatabaseEntry? BestMatch(string text) => Find(text, 1) is [var only, ..] ? only : null;

        // The library's plan where it answers for this index, else null for the C#'s.
        internal CheatPrunePlan? NativePlan(IReadOnlyCollection<string> keep) => _native.Value is { } native && keep.All(k => ServiceLibrary.Takes(k)) ? native.Plan(keep) : null;

        private static int Score(string game, string needle)
        {
            if (game.Equals(needle, StringComparison.OrdinalIgnoreCase)) return 3;
            if (game.StartsWith(needle, StringComparison.OrdinalIgnoreCase)) return 2;
            if (game.Contains(needle, StringComparison.OrdinalIgnoreCase)) return 1;
            return 0;
        }

        // Drops a ROM extension so a loaded file name can be passed straight in.
        private static string Normalize(string text)
        {
            string t = (text ?? "").Trim();
            foreach (string extension in new[] { ".cht", ".sfc", ".smc", ".zip" })
            {
                if (t.EndsWith(extension, StringComparison.OrdinalIgnoreCase)) return t[..^extension.Length];
            }
            return t;
        }
    }
}
