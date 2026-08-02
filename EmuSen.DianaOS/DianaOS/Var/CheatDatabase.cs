using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // One .cht file sitting in the user's cheat tree.
    public readonly struct CheatDatabaseEntry
    {
        // The folder it sits in - libretro names these per system.
        public string System { get; init; }

        // The file name without .cht, which is the game's DAT name.
        public string Game { get; init; }

        public string Path { get; init; }
    }

    // An index over a directory tree of .cht files - the user's own copy,
    // never anything EmuSen ships. See `man cheat`.
    public sealed class CheatDatabase
    {
        public string Directory { get; }

        // A real libretro tree is tens of thousands of files and every query
        // here walks all of it, so one instance scans once - see `man cheat`.
        private IReadOnlyList<CheatDatabaseEntry>? _all;

        public CheatDatabase(string directory)
        {
            Directory = directory;
        }

        public bool Exists => System.IO.Directory.Exists(Directory);

        public IReadOnlyList<CheatDatabaseEntry> All() => _all ??= Scan();

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
            All().GroupBy(e => e.System, StringComparer.OrdinalIgnoreCase)
                 .Select(g => (g.Key, g.Count()))
                 .OrderBy(g => g.Key, StringComparer.OrdinalIgnoreCase)
                 .ToList();

        // Every game under one system folder, optionally narrowed by a
        // substring - a system holds thousands, so the filter is not optional
        // in practice. Already name-ordered by All().
        public IReadOnlyList<CheatDatabaseEntry> Games(string system, string? filter = null)
        {
            IEnumerable<CheatDatabaseEntry> games = All().Where(e => string.Equals(e.System, system, StringComparison.OrdinalIgnoreCase));

            if (!string.IsNullOrWhiteSpace(filter))
            {
                string needle = filter.Trim();
                games = games.Where(e => e.Game.Contains(needle, StringComparison.OrdinalIgnoreCase));
            }

            return games.ToList();
        }

        // Best matches first. A ROM is usually loaded as "Game (USA).sfc"
        // while its cheat file is "Game (USA).cht", so the extension is
        // stripped before matching rather than expecting the caller to.
        public IReadOnlyList<CheatDatabaseEntry> Find(string text, int limit = 20)
        {
            string needle = Normalize(text);
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
