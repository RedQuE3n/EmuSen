using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Text
{
    // "Did you mean ...?" for a name that didn't match anything; here because the shell and config both reach it - see EmuSen_Config_Reference.md §6.
    public static class Suggestion
    {
        // Optimal string alignment - Levenshtein plus adjacent transposition, so 'chaet' for 'cheat' is one slip - see EmuSen_Config_Reference.md §6.
        public static int Distance(string a, string b)
        {
            ArgumentNullException.ThrowIfNull(a);
            ArgumentNullException.ThrowIfNull(b);
            return GalaxiaNative.SuggestDistance(a, b);
        }

        // Closest first; a candidate the typed text is a prefix of wins ties, since 'cl' for 'clear' is far likelier than any other.
        public static IReadOnlyList<string> Nearest(string typed, IEnumerable<string> candidates, int max = 2) =>
            GalaxiaNative.SuggestNearest(typed, Listed(typed, candidates), max);

        // Empty when nothing is close enough, so call sites can append it directly to an existing message without a null check.
        public static string Hint(string typed, IEnumerable<string> candidates, int max = 2) =>
            GalaxiaNative.SuggestHint(typed, Listed(typed, candidates), max);

        // The candidates as the library takes them: one holding a NUL, which no name can, is left out, since it would end its item early - see EmuSen_Backend.md §40.2.
        private static IReadOnlyList<string?> Listed(string typed, IEnumerable<string> candidates)
        {
            ArgumentNullException.ThrowIfNull(typed);
            ArgumentNullException.ThrowIfNull(candidates);
            return candidates.Where(c => c is null || !c.Contains('\0')).ToArray();
        }
    }
}
