using System.Collections.Generic;
using EmuSen.Cauldron;

namespace EmuSen.WiseMan.Common
{
    // HistoryProvider is the seam that turns "what is the machine doing
    // now" into "what was it doing before it stopped" - see its own
    // comment. These pin the ring's wrap order and the staleness signal,
    // both of which are easy to get subtly wrong and silently useless.
    public class HistoryProviderTests
    {
        private static HistoryProvider<List<int>> Build(int capacity, List<List<int>> script)
        {
            // script[0] is the initial snapshot, so the first Refresh must
            // serve script[1] rather than replaying it.
            int index = 1;
            return new HistoryProvider<List<int>>(
                () => script[index++],
                script[0],
                capacity,
                ListEqualityComparer<int>.Instance);
        }

        [Fact]
        public void History_starts_with_the_initial_snapshot()
        {
            var provider = Build(4, [[1]]);
            Assert.Equal([new List<int> { 1 }], provider.GetHistory());
        }

        [Fact]
        public void History_returns_snapshots_oldest_first()
        {
            var provider = Build(4, [[0], [1], [2]]);
            provider.Refresh();
            provider.Refresh();

            var history = provider.GetHistory();
            Assert.Equal(3, history.Count);
            Assert.Equal([0], history[0]);
            Assert.Equal([1], history[1]);
            Assert.Equal([2], history[2]);
        }

        // The wrap is the part worth pinning: once the ring is full the
        // oldest entry is the slot about to be overwritten, not slot 0.
        [Fact]
        public void History_drops_the_oldest_entry_once_it_wraps()
        {
            var provider = Build(3, [[0], [1], [2], [3], [4]]);
            for (int i = 0; i < 4; i++) provider.Refresh();

            var history = provider.GetHistory();
            Assert.Equal(3, history.Count);
            Assert.Equal([2], history[0]);
            Assert.Equal([3], history[1]);
            Assert.Equal([4], history[2]);
        }

        [Fact]
        public void Capacity_bounds_the_ring()
        {
            var provider = Build(2, [[0], [1], [2], [3]]);
            for (int i = 0; i < 3; i++) provider.Refresh();

            Assert.Equal(2, provider.Capacity);
            Assert.Equal(2, provider.GetHistory().Count);
        }

        [Fact]
        public void Current_tracks_the_latest_refresh()
        {
            var provider = Build(4, [[0], [1], [2]]);
            provider.Refresh();
            Assert.Equal([1], provider.Current);
            provider.Refresh();
            Assert.Equal([2], provider.Current);
        }

        // The signal that answers "when did the coprocessor stop updating".
        [Fact]
        public void RefreshesSinceChange_counts_up_while_the_value_holds_still()
        {
            var provider = Build(8, [[0], [1], [1], [1]]);

            provider.Refresh();                              // 0 -> 1, changed
            Assert.Equal(0, provider.RefreshesSinceChange);

            provider.Refresh();                              // 1 -> 1, still
            Assert.Equal(1, provider.RefreshesSinceChange);

            provider.Refresh();                              // 1 -> 1, still
            Assert.Equal(2, provider.RefreshesSinceChange);
        }

        [Fact]
        public void RefreshesSinceChange_resets_when_the_value_moves_again()
        {
            var provider = Build(8, [[0], [0], [0], [5]]);
            provider.Refresh();
            provider.Refresh();
            Assert.Equal(2, provider.RefreshesSinceChange);

            provider.Refresh();                              // 0 -> 5, changed
            Assert.Equal(0, provider.RefreshesSinceChange);
        }

        // Without a comparer nothing can be judged unchanged, so the signal
        // reports 0 rather than a misleading count - see the property's own
        // comment.
        [Fact]
        public void RefreshesSinceChange_stays_zero_without_a_comparer()
        {
            var script = new List<List<int>>
            {
                new List<int> { 0 },
                new List<int> { 0 },
                new List<int> { 0 },
            };
            int index = 1;
            var provider = new HistoryProvider<List<int>>(() => script[index++], script[0], 4);

            provider.Refresh();
            provider.Refresh();

            Assert.Equal(0, provider.RefreshesSinceChange);
        }

        [Fact]
        public void RefreshCount_tracks_every_refresh()
        {
            var provider = Build(4, [[0], [1], [2]]);
            Assert.Equal(0, provider.RefreshCount);
            provider.Refresh();
            provider.Refresh();
            Assert.Equal(2, provider.RefreshCount);
        }

        [Fact]
        public void A_capacity_below_one_is_rejected()
        {
            Assert.Throws<System.ArgumentOutOfRangeException>(
                () => new HistoryProvider<List<int>>(() => [], [], 0));
        }

        // Freshly-built snapshots are never reference-equal, so without
        // element-wise comparison the staleness signal would always read 0.
        [Fact]
        public void ListEqualityComparer_compares_by_element_not_reference()
        {
            var comparer = ListEqualityComparer<int>.Instance;
            Assert.True(comparer.Equals([1, 2, 3], [1, 2, 3]));
            Assert.False(comparer.Equals([1, 2, 3], [1, 2, 4]));
            Assert.False(comparer.Equals([1, 2], [1, 2, 3]));
        }
    }
}
