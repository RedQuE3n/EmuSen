using System.Collections.Generic;

namespace EmuSen.Cauldron
{
    // Element-wise equality, so HistoryProvider's staleness signal has something real to compare - see EmuSen_Cauldron.md §2.3.
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
