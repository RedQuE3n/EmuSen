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
        public static int Distance(string a, string b) =>
            GalaxiaNative.Active && a is not null && b is not null && GalaxiaNative.Crosses(a) && GalaxiaNative.Crosses(b) ? GalaxiaNative.SuggestDistance(a, b) : Managed.Distance(a!, b!);

        // Closest first; a candidate the typed text is a prefix of wins ties, since 'cl' for 'clear' is far likelier than any other.
        public static IReadOnlyList<string> Nearest(string typed, IEnumerable<string> candidates, int max = 2) =>
            Listed(typed, candidates) is { } listed
                ? GalaxiaNative.SuggestNearest(typed, listed, max)
                : Managed.Nearest(typed, candidates, max);

        // Empty when nothing is close enough, so call sites can append it directly to an existing message without a null check.
        public static string Hint(string typed, IEnumerable<string> candidates, int max = 2) =>
            Listed(typed, candidates) is { } listed ? GalaxiaNative.SuggestHint(typed, listed, max) : Managed.Hint(typed, candidates, max);

        // The candidates as a list the library can take, or null when the C# answers: the switch is off, or a text will not cross whole.
        private static IReadOnlyList<string?>? Listed(string typed, IEnumerable<string> candidates)
        {
            if (!GalaxiaNative.Active || typed is null || candidates is null || !GalaxiaNative.Crosses(typed)) return null;
            string?[] listed = candidates.ToArray();
            return listed.All(candidate => GalaxiaNative.Crosses(candidate, listed: true)) ? listed : null;
        }

        // The C# rules: the default, and what the library's are held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static int Distance(string a, string b)
            {
                if (a.Length == 0) return b.Length;
                if (b.Length == 0) return a.Length;

                var d = new int[a.Length + 1, b.Length + 1];
                for (int i = 0; i <= a.Length; i++) d[i, 0] = i;
                for (int j = 0; j <= b.Length; j++) d[0, j] = j;

                for (int i = 1; i <= a.Length; i++)
                {
                    for (int j = 1; j <= b.Length; j++)
                    {
                        int cost = char.ToLowerInvariant(a[i - 1]) == char.ToLowerInvariant(b[j - 1]) ? 0 : 1;
                        d[i, j] = Math.Min(Math.Min(d[i - 1, j] + 1, d[i, j - 1] + 1), d[i - 1, j - 1] + cost);

                        if (i > 1 && j > 1
                            && char.ToLowerInvariant(a[i - 1]) == char.ToLowerInvariant(b[j - 2])
                            && char.ToLowerInvariant(a[i - 2]) == char.ToLowerInvariant(b[j - 1]))
                        {
                            d[i, j] = Math.Min(d[i, j], d[i - 2, j - 2] + cost);
                        }
                    }
                }

                return d[a.Length, b.Length];
            }

            // How wrong a short word is allowed to be before a guess stops being a guess: 'rm' is one edit from 'ls', 'cp' and 'mv'.
            private static int Tolerance(int length) => length switch
            {
                <= 3 => 1,
                <= 6 => 2,
                _ => 3,
            };

            public static IReadOnlyList<string> Nearest(string typed, IEnumerable<string> candidates, int max)
            {
                if (string.IsNullOrWhiteSpace(typed) || max <= 0) return Array.Empty<string>();

                // Nothing to correct: an exact match means the caller rejected the name for some other reason.
                if (candidates.Any(c => string.Equals(c, typed, StringComparison.OrdinalIgnoreCase)))
                {
                    return Array.Empty<string>();
                }

                int tolerance = Tolerance(typed.Length);

                var scored = candidates
                    .Where(c => !string.IsNullOrEmpty(c))
                    .Select(c => (Name: c, Distance: Distance(typed, c)))
                    .Where(x => x.Distance > 0 && x.Distance <= tolerance)
                    .ToList();

                if (scored.Count == 0) return Array.Empty<string>();

                // Only the joint-best: 'chaet' is one edit from 'cheat' and two from 'cat', and offering both dilutes the right one.
                int best = scored.Min(x => x.Distance);

                return scored
                    .Where(x => x.Distance == best)
                    .OrderByDescending(x => x.Name.StartsWith(typed, StringComparison.OrdinalIgnoreCase))
                    .ThenBy(x => x.Name, StringComparer.OrdinalIgnoreCase)
                    .Take(max)
                    .Select(x => x.Name)
                    .ToList();
            }

            public static string Hint(string typed, IEnumerable<string> candidates, int max)
            {
                IReadOnlyList<string> nearest = Nearest(typed, candidates, max);
                return nearest.Count switch
                {
                    0 => "",
                    1 => $" Did you mean '{nearest[0]}'?",
                    _ => $" Did you mean {string.Join(" or ", nearest.Select(n => $"'{n}'"))}?",
                };
            }
        }
    }
}
