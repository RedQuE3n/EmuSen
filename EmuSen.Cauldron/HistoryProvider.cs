using System;
using System.Collections.Generic;
using System.Threading;

namespace EmuSen.Cauldron
{
    // An IRealtimeProvider<T> that also remembers the last <capacity>
    // snapshots, rather than only the most recent one. PollingProvider
    // answers "what is the machine doing now", which is what a live
    // dashboard wants; this answers "what was it doing fifty frames ago",
    // which is what an investigation into a transition wants - when did
    // the display go blank, when did the coprocessor stop, what changed
    // either side of it.
    //
    // Deliberately a separate type rather than a flag on PollingProvider:
    // keeping history costs a retained reference per Refresh, so a consumer
    // that only ever reads Current should not pay for a ring it never
    // looks at. Use this only where the history is actually wanted.
    public sealed class HistoryProvider<T> : IRealtimeProvider<T> where T : class
    {
        private readonly Func<T> _readLive;
        private readonly IEqualityComparer<T>? _comparer;

        // Ring of the last <capacity> snapshots. Guarded by _gate, unlike
        // Current - see the threading note on History below.
        private readonly T[] _ring;
        private readonly object _gate = new object();
        private int _next;      // next slot to write
        private int _count;     // how many slots are populated

        private T _current;
        private long _refreshCount;
        private long _lastChangedRefresh;

        // <capacity> is a hard bound: the ring overwrites its oldest entry
        // rather than growing, so a run of any length costs the same fixed
        // memory. <comparer> is optional and only drives the staleness
        // signal - pass null and RefreshesSinceChange stays 0 forever.
        public HistoryProvider(Func<T> readLive, T initial, int capacity, IEqualityComparer<T>? comparer = null)
        {
            if (capacity < 1) throw new ArgumentOutOfRangeException(nameof(capacity));

            _readLive = readLive;
            _comparer = comparer;
            _ring = new T[capacity];
            _current = initial;

            _ring[0] = initial;
            _next = capacity == 1 ? 0 : 1;
            _count = 1;
        }

        public int Capacity => _ring.Length;

        // Same lock-free contract as PollingProvider - never blocks, safe
        // from any thread, including during a concurrent Refresh().
        public T Current => Volatile.Read(ref _current);

        // How many times Refresh() has run. The clock the staleness signal
        // below is measured in; a caller wanting real frame numbers should
        // pair it with its own frame counter.
        public long RefreshCount => Volatile.Read(ref _refreshCount);

        // Refreshes since Current last compared unequal to the snapshot
        // before it - "how long has this been sitting still". Answers
        // "when did the coprocessor stop updating" directly, without
        // needing to store or diff any history at all. Always 0 when no
        // comparer was supplied, since nothing can be judged unchanged.
        public long RefreshesSinceChange =>
            _comparer == null ? 0 : Volatile.Read(ref _refreshCount) - Volatile.Read(ref _lastChangedRefresh);

        // Must only be called from the thread that owns the core, same
        // contract as PollingProvider.Refresh.
        public void Refresh()
        {
            T value = _readLive();
            T previous = Volatile.Read(ref _current);

            Volatile.Write(ref _current, value);
            long refreshes = Interlocked.Increment(ref _refreshCount);

            if (_comparer != null && !_comparer.Equals(previous, value))
            {
                Volatile.Write(ref _lastChangedRefresh, refreshes);
            }

            lock (_gate)
            {
                _ring[_next] = value;
                _next = (_next + 1) % _ring.Length;
                if (_count < _ring.Length) _count++;
            }
        }

        // The retained snapshots, oldest first, newest last. Copies out
        // under the lock rather than handing back the live ring, so a
        // caller iterating it can't have entries overwritten underneath
        // it by a Refresh() on the emulation thread. That makes this the
        // one member here that can block - briefly, on an uncontended
        // lock - which is why it is a method rather than a property that
        // looks as cheap as Current.
        public IReadOnlyList<T> GetHistory()
        {
            lock (_gate)
            {
                var result = new T[_count];
                // _next is one past the newest, so the oldest entry is the
                // slot it is about to overwrite once the ring has wrapped.
                int start = _count < _ring.Length ? 0 : _next;
                for (int i = 0; i < _count; i++) result[i] = _ring[(start + i) % _ring.Length];
                return result;
            }
        }
    }
}
