using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Native;

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

    // An index over the player's own .cht tree, never anything EmuSen ships, emusen-cheats' database scanned once - see `man cheat`, EmuSen_Backend.md §15.3.
    public sealed class CheatDatabase
    {
        public string Directory { get; }

        // A real libretro tree is tens of thousands of files, so one instance scans once.
        private IReadOnlyList<CheatDatabaseEntry>? _all;

        private readonly Lazy<CheatsNative.Database> _native;

        public CheatDatabase(string directory)
        {
            Directory = directory;
            _native = new(() => CheatsNative.Database.Open(PlatformText.Mend(directory)));
        }

        public bool Exists => System.IO.Directory.Exists(Directory);

        public IReadOnlyList<CheatDatabaseEntry> All() => _all ??= _native.Value.Entries();

        public IReadOnlyList<(string System, int Count)> Systems() => _native.Value.Systems();

        // A system holds thousands, so the filter is not optional in practice.
        public IReadOnlyList<CheatDatabaseEntry> Games(string system, string? filter = null) => Places(_native.Value.Games(system, filter));

        private IReadOnlyList<CheatDatabaseEntry> Places(IReadOnlyList<int> places)
        {
            IReadOnlyList<CheatDatabaseEntry> all = All();
            return places.Select(i => all[i]).ToList();
        }

        // The extension is stripped, since a ROM is .sfc where its cheat file is .cht.
        public IReadOnlyList<CheatDatabaseEntry> Find(string text, int limit = 20) => Places(_native.Value.Find(text ?? "", limit));

        public CheatDatabaseEntry? BestMatch(string text) => Find(text, 1) is [var only, ..] ? only : null;

        internal CheatPrunePlan NativePlan(IReadOnlyCollection<string> keep) => _native.Value.Plan(keep);
    }
}
