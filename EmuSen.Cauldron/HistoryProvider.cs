using System;
using System.Collections.Generic;
using System.Threading;

namespace EmuSen.Cauldron
{
    // Also remembers the last <capacity> snapshots - "what was it doing fifty frames ago" - see EmuSen_Cauldron.md §2.2.
    public sealed class HistoryProvider<T> : IRealtimeProvider<T> where T : class
    {
        private readonly Func<T> _readLive;
        private readonly IEqualityComparer<T>? _comparer;

        // Guarded by _gate, unlike Current - see §2.2.
        private readonly T[] _ring;
        private readonly object _gate = new object();
        private int _next;      // next slot to write
        private int _count;     // how many slots are populated

        private T _current;
        private long _refreshCount;
        private long _lastChangedRefresh;

        // <capacity> is a hard bound; <comparer> is optional and only drives staleness - see §2.2.
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

        // Same lock-free contract as PollingProvider - see §2.
        public T Current => Volatile.Read(ref _current);

        // The clock RefreshesSinceChange is measured in - see §2.2.
        public long RefreshCount => Volatile.Read(ref _refreshCount);

        // "How long has this been sitting still"; always 0 without a comparer - see §2.2.
        public long RefreshesSinceChange =>
            _comparer == null ? 0 : Volatile.Read(ref _refreshCount) - Volatile.Read(ref _lastChangedRefresh);

        // Emulation thread only, same contract as PollingProvider.Refresh - see §2.
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

        // Oldest first; copies out under the lock, so this is the one member here that can block - see §2.2.
        public IReadOnlyList<T> GetHistory()
        {
            lock (_gate)
            {
                var result = new T[_count];
                // _next is one past the newest, so it is also the oldest once wrapped.
                int start = _count < _ring.Length ? 0 : _next;
                for (int i = 0; i < _count; i++) result[i] = _ring[(start + i) % _ring.Length];
                return result;
            }
        }
    }
}
