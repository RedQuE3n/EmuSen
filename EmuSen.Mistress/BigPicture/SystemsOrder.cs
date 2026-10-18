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

        // Each system's first release by its ES-DE name, the systems table's year - see EmuSen_Backend.md §12.
        public static IReadOnlyDictionary<string, int> FirstRelease =>
            EmuSen.Mistress.Library.RuntimeCatalog.Systems.Where(s => s.EsdeSystem.Length > 0).GroupBy(s => s.EsdeSystem).ToDictionary(g => g.Key, g => g.First().ReleaseYear, StringComparer.Ordinal);

        // The shelves in the chosen order; a system with no known year goes last, and ties keep the order given.
        public static IReadOnlyList<T> Sort<T>(IReadOnlyList<T> shelves, string sorting, Func<T, string> system, Func<T, string> fullName) => sorting switch
        {
            BigPictureInterface.SortFullNames => shelves.OrderBy(fullName, StringComparer.OrdinalIgnoreCase).ToList(),
            BigPictureInterface.SortReleaseYear => ByYear(shelves, system, FirstRelease),
            _ => shelves,
        };

        private static List<T> ByYear<T>(IReadOnlyList<T> shelves, Func<T, string> system, IReadOnlyDictionary<string, int> years) =>
            shelves.OrderBy(s => years.GetValueOrDefault(system(s), int.MaxValue)).ToList();
    }
}
