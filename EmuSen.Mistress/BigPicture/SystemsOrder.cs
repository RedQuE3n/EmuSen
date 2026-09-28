using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.BigPicture
{
    // ES-DE's "Systems sorting", in the orders Mistress can answer from its own data - see EmuSen_Settings_Reference.md §4.66.
    public static class SystemsOrder
    {
        public static readonly (string Value, string Text)[] Choices =
        [
            (BigPictureInterface.SortRelease, "Release order (EmuSen's)"),
            (BigPictureInterface.SortFullNames, "Full names"),
            (BigPictureInterface.SortReleaseYear, "Release year"),
        ];

        // Each system's first release, which was Japan's for all five.
        public static readonly IReadOnlyDictionary<string, int> FirstRelease = new Dictionary<string, int>(StringComparer.Ordinal)
        {
            ["nes"] = 1983, ["gb"] = 1989, ["snes"] = 1990, ["n64"] = 1996, ["gbc"] = 1998,
        };

        // The shelves in the chosen order; a system with no known year goes last, and ties keep the order given.
        public static IReadOnlyList<T> Sort<T>(IReadOnlyList<T> shelves, string sorting, Func<T, string> system, Func<T, string> fullName) => sorting switch
        {
            BigPictureInterface.SortFullNames => shelves.OrderBy(fullName, StringComparer.OrdinalIgnoreCase).ToList(),
            BigPictureInterface.SortReleaseYear => shelves.OrderBy(s => FirstRelease.GetValueOrDefault(system(s), int.MaxValue)).ToList(),
            _ => shelves,
        };
    }
}
