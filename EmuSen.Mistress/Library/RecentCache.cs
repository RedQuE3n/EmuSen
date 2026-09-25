using System;
using System.Collections.Generic;

namespace EmuSen.Mistress.Library
{
    // A dictionary that keeps only its most recently used entries, and counts entries dropped before anyone asked for them - see EmuSen_Settings_Reference.md §4.48.10.
    public sealed class RecentCache<TKey, TValue> where TKey : notnull
    {
        private readonly Dictionary<TKey, LinkedListNode<(TKey Key, TValue Value, bool Used)>> _index;
        private readonly LinkedList<(TKey Key, TValue Value, bool Used)> _order = new();

        public RecentCache(int capacity, IEqualityComparer<TKey>? comparer = null)
        {
            if (capacity < 1) throw new ArgumentOutOfRangeException(nameof(capacity));
            Capacity = capacity;
            _index = new Dictionary<TKey, LinkedListNode<(TKey, TValue, bool)>>(comparer);
        }

        public int Capacity { get; }

        public int Count => _index.Count;

        // Entries dropped to make room, and of those the ones stored unused (a read ahead nobody moved onto).
        public int Evicted { get; private set; }
        public int EvictedUnused { get; private set; }

        public bool Contains(TKey key) => _index.ContainsKey(key);

        // A value asked for, which becomes the most recent and counts as used.
        public bool TryGet(TKey key, out TValue value)
        {
            if (!_index.TryGetValue(key, out var node)) { value = default!; return false; }
            _order.Remove(node);
            node.Value = node.Value with { Used = true };
            _order.AddFirst(node);
            value = node.Value.Value;
            return true;
        }

        // Stores a value as the most recent, used or not yet, and drops the least recent past the capacity.
        public void Add(TKey key, TValue value, bool used)
        {
            if (_index.TryGetValue(key, out var old)) _order.Remove(old);
            var node = _order.AddFirst((key, value, used || (old?.Value.Used ?? false)));
            _index[key] = node;
            while (_index.Count > Capacity)
            {
                var last = _order.Last!;
                _order.RemoveLast();
                _index.Remove(last.Value.Key);
                Evicted++;
                if (!last.Value.Used) EvictedUnused++;
            }
        }

        public void Clear()
        {
            _index.Clear();
            _order.Clear();
        }
    }
}
