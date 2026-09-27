using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // The system view's gameselector elements: which games each picks, and which pick an element shows - see EmuSen_BigPicture.md §36.
    public sealed class GameSelectors
    {
        private readonly SortedDictionary<string, (ResolvedElement Element, IReadOnlyList<SceneGame> Games)> _byName = new(StringComparer.Ordinal);

        private GameSelectors() { }

        public static GameSelectors None { get; } = new();

        // Every gameselector of a system view, picked from one system's games; the seed varies the random picks between navigations.
        public static GameSelectors For(ResolvedView view, SceneSystem system, int seed)
        {
            var selectors = new GameSelectors();
            foreach (ResolvedElement e in view.OfType("gameselector"))
                selectors._byName[e.Name] = (e, Pick(e, system, seed ^ StableHash(e.Name)));
            return selectors;
        }

        public bool Any => _byName.Count > 0;

        public IEnumerable<string> Names => _byName.Keys;

        public IReadOnlyList<SceneGame> Games(string name) => _byName.TryGetValue(name, out var s) ? s.Games : [];

        // The game an element shows: its named selector, else the one whose name sorts first, as ES-DE 3.4.1 was measured to choose; null when the entry is not filled.
        public SceneGame? GameFor(ResolvedElement e)
        {
            if (_byName.Count == 0) return null;
            var chosen = e.String("gameselector") is { } name && _byName.TryGetValue(name, out var named) ? named : _byName.First().Value;
            int count = (int)Math.Max(1, chosen.Element.UInt("gameCount") ?? 1);
            int entry = (int)Math.Min(e.UInt("gameselectorEntry") ?? 0, (uint)(count - 1));
            return entry < chosen.Games.Count ? chosen.Games[entry] : null;
        }

        // The games one selector picks: the played ones by date or count, or a shuffle, without the excluded or folders (§36).
        public static IReadOnlyList<SceneGame> Pick(ResolvedElement e, SceneSystem system, int seed)
        {
            int count = (int)Math.Clamp(e.UInt("gameCount") ?? 1, 1, 30);
            List<SceneGame> pool = (system.Counted ?? system.Games).Where(g => !g.Folder && !g.NotCounted).ToList();
            switch (e.String("selection"))
            {
                case "lastplayed":
                    return pool.Where(g => g.LastPlayed is not null).OrderByDescending(g => g.LastPlayed).Take(count).ToList();
                case "mostplayed":
                    return pool.Where(g => g.PlayCount > 0).OrderByDescending(g => g.PlayCount).Take(count).ToList();
                default:
                    var random = new Random(seed);
                    List<SceneGame> picked = pool.OrderBy(_ => random.Next()).Take(count).ToList();
                    if (e.Bool("allowDuplicates") == true)
                        while (picked.Count < count && pool.Count > 0) picked.Add(pool[random.Next(pool.Count)]);
                    return picked;
            }
        }

        private static int StableHash(string s)
        {
            unchecked
            {
                int h = 17;
                foreach (char c in s) h = h * 31 + c;
                return h;
            }
        }
    }
}
