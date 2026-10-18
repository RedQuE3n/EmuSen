namespace EmuSen.Crystal.Telemetry
{
    // A run of values and how many there are, the array growing only when a frame has more than it held.
    public sealed class ValuesRecord<TSlot> : ISnapshotRecord<ValuesRecord<TSlot>>
    {
        public TSlot[] Values;
        public int Count;

        public ValuesRecord(int capacity) => Values = new TSlot[Math.Max(1, capacity)];

        private ValuesRecord(TSlot[] values) => (Values, Count) = (values, values.Length);

        // fill writes the values and returns how many there are, more than the array holds asking for a larger one.
        public void Fill(Func<TSlot[], int> fill)
        {
            int n = fill(Values);
            if (n > Values.Length)
            {
                Values = new TSlot[n];
                n = fill(Values);
            }
            Count = Math.Clamp(n, 0, Values.Length);
        }

        public ValuesRecord<TSlot> Clone() => new(Values.AsSpan(0, Count).ToArray());
    }

    // A provider whose frame writes raw values into a ring and whose reader builds the value from a copy, so a frame allocates nothing - see EmuSen_Backend.md §14.5.
    public sealed class RingProvider<TSlot, T> : IRealtimeProvider<T>
    {
        private readonly SnapshotRing<ValuesRecord<TSlot>> _ring;
        private readonly Action<ValuesRecord<TSlot>> _fill;
        private readonly Func<TSlot[], int, T> _build;

        // Filled once here, so Current is never "nothing" before the first Refresh.
        public RingProvider(int capacity, Func<TSlot[], int> fill, Func<TSlot[], int, T> build)
        {
            _ring = new SnapshotRing<ValuesRecord<TSlot>>(() => new ValuesRecord<TSlot>(capacity));
            _fill = record => record.Fill(fill);
            _build = build;
            Refresh();
        }

        public void Refresh() => _ring.Refresh(_fill);

        public T Current
        {
            get
            {
                ValuesRecord<TSlot> copy = _ring.Newest;
                return _build(copy.Values, copy.Count);
            }
        }
    }
}
