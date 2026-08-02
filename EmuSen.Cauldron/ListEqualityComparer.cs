using System.Collections.Generic;

namespace EmuSen.Cauldron
{
    // Element-wise equality for the IReadOnlyList<T> snapshots every
    // provider in this project publishes. Exists so HistoryProvider's
    // staleness signal has something meaningful to compare with: the
    // snapshots are freshly built each Refresh, so reference equality
    // would report "changed" every single time and the signal would
    // always read 0.
    //
    // Kept core-agnostic like the rest of EmuSen.Cauldron - it compares
    // whatever TItem's own Equals says, so a register/sprite/load
    // snapshot all work without this knowing what any of them are.
    public sealed class ListEqualityComparer<TItem> : IEqualityComparer<IReadOnlyList<TItem>>
    {
        public static readonly ListEqualityComparer<TItem> Instance = new ListEqualityComparer<TItem>();

        private static readonly EqualityComparer<TItem> ItemComparer = EqualityComparer<TItem>.Default;

        public bool Equals(IReadOnlyList<TItem>? x, IReadOnlyList<TItem>? y)
        {
            if (ReferenceEquals(x, y)) return true;
            if (x == null || y == null) return false;
            if (x.Count != y.Count) return false;

            for (int i = 0; i < x.Count; i++)
            {
                if (!ItemComparer.Equals(x[i], y[i])) return false;
            }
            return true;
        }

        public int GetHashCode(IReadOnlyList<TItem> obj)
        {
            var hash = new System.HashCode();
            foreach (var item in obj) hash.Add(item);
            return hash.ToHashCode();
        }
    }
}
