using System.Threading;

namespace EmuSen.Crystal.Telemetry
{
    // What one frame's snapshot holds: filled in place by the frame, cloned whole for a reader.
    public interface ISnapshotRecord<TSelf> where TSelf : class
    {
        TSelf Clone();
    }

    // A frame's snapshot written into records made once and reused round a ring, so a frame allocates nothing; a reader clones the newest - see EmuSen_Backend.md §14.5.
    public sealed class SnapshotRing<TRecord> where TRecord : class, ISnapshotRecord<TRecord>
    {
        public const int Depth = 4;

        private readonly TRecord[] _records = new TRecord[Depth];
        private int _latest;
        private long _version;
        private Cached? _cached;

        private sealed record Cached(long Version, TRecord Record);

        public SnapshotRing(Func<TRecord> make)
        {
            for (int i = 0; i < Depth; i++) _records[i] = make();
        }

        // The emulation thread's, once a frame: the record after the newest filled under its own lock, then published.
        public void Refresh(Action<TRecord> fill)
        {
            int next = (Volatile.Read(ref _latest) + 1) % Depth;
            TRecord record = _records[next];
            lock (record) fill(record);
            Volatile.Write(ref _latest, next);
            Interlocked.Increment(ref _version);
        }

        // Any thread: the newest record cloned under its lock, which the writer takes only on coming round to it again.
        public TRecord Copy()
        {
            TRecord record = _records[Volatile.Read(ref _latest)];
            lock (record) return record.Clone();
        }

        // Any thread: one clone of the newest record shared by every reader until the next frame, which no reader may change.
        public TRecord Newest
        {
            get
            {
                long version = Volatile.Read(ref _version);
                if (Volatile.Read(ref _cached) is { } cached && cached.Version == version) return cached.Record;
                TRecord copy = Copy();
                Volatile.Write(ref _cached, new Cached(version, copy));
                return copy;
            }
        }
    }

    // One provider's view of a ring several share: the value built from a clone of the newest record; the target refreshes the ring itself.
    public sealed class RingView<TRecord, T> : IRealtimeProvider<T> where TRecord : class, ISnapshotRecord<TRecord>
    {
        private readonly SnapshotRing<TRecord> _ring;
        private readonly Func<TRecord, T> _build;

        public RingView(SnapshotRing<TRecord> ring, Func<TRecord, T> build)
        {
            _ring = ring;
            _build = build;
        }

        public T Current => _build(_ring.Newest);

        public void Refresh() { }
    }
}
