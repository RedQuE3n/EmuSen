namespace EmuSen.Crystal
{
    // The master timeline: one pending time per event id, plus the devices that run against it - see EmuSen_Crystal_Scheduler.md.
    public sealed class Scheduler
    {
        public const long Never = long.MaxValue;
        public const int DefaultCapacity = 32;

        // A handler that reschedules at the current instant forever would otherwise hang the frame - see §4.
        public const int MaxDispatchesPerRun = 4096;

        // Indexed by event id rather than a heap: ids are dense and few, so a scan beats a priority queue - see §3.
        private readonly long[] _eventTime;
        private readonly List<IClockedDevice> _devices = new();
        private IScheduleHandler? _handler;

        public Scheduler(int capacity = DefaultCapacity)
        {
            if (capacity <= 0) throw new ArgumentOutOfRangeException(nameof(capacity));
            _eventTime = new long[capacity];
            Reset();
        }

        public long Now { get; private set; }

        public int Capacity => _eventTime.Length;

        public IReadOnlyList<IClockedDevice> Devices => _devices;

        public void SetHandler(IScheduleHandler handler) =>
            _handler = handler ?? throw new ArgumentNullException(nameof(handler));

        public void AddDevice(IClockedDevice device)
        {
            if (device == null) throw new ArgumentNullException(nameof(device));
            if (!_devices.Contains(device)) _devices.Add(device);
        }

        // A time already past is legal and fires at the next opportunity - an instruction may overshoot a deadline.
        public void At(long when, int eventId)
        {
            CheckId(eventId);
            _eventTime[eventId] = when;
        }

        public void In(long delta, int eventId) => At(Now + delta, eventId);

        public void Cancel(int eventId)
        {
            CheckId(eventId);
            _eventTime[eventId] = Never;
        }

        public bool IsScheduled(int eventId)
        {
            CheckId(eventId);
            return _eventTime[eventId] != Never;
        }

        public long TimeOf(int eventId)
        {
            CheckId(eventId);
            return _eventTime[eventId];
        }

        // Ties break on the lower event id, so a given core always dispatches in one order - see §4.
        public long NextEventTime()
        {
            long best = Never;
            for (int i = 0; i < _eventTime.Length; i++)
            {
                if (_eventTime[i] < best) best = _eventTime[i];
            }
            return best;
        }

        public int NextEventId()
        {
            long best = Never;
            int bestId = -1;
            for (int i = 0; i < _eventTime.Length; i++)
            {
                if (_eventTime[i] < best)
                {
                    best = _eventTime[i];
                    bestId = i;
                }
            }
            return bestId;
        }

        // Fires every event due at or before deadline, then leaves Now exactly at deadline.
        public void RunUntil(long deadline)
        {
            int dispatches = 0;
            while (true)
            {
                int id = NextEventId();
                if (id < 0) break;

                long when = _eventTime[id];
                if (when > deadline) break;

                if (++dispatches > MaxDispatchesPerRun)
                {
                    throw new InvalidOperationException(
                        $"Event {id} re-scheduled itself {MaxDispatchesPerRun} times without time advancing.");
                }

                // Past-due events run at Now, so a handler never sees the clock go backwards.
                Now = when > Now ? when : Now;
                _eventTime[id] = Never;
                _handler?.OnScheduledEvent(id, Now);
            }

            if (deadline > Now) Now = deadline;
        }

        // How a core spends time: the driving device consumes clocks, then RunUntil dispatches what came due.
        public void Advance(long masterTicks)
        {
            if (masterTicks < 0) throw new ArgumentOutOfRangeException(nameof(masterTicks), "A clock cannot run backwards.");
            Now += masterTicks;
        }

        public void Sync(IClockedDevice device) => device.SyncTo(Now);

        public void SyncAll()
        {
            for (int i = 0; i < _devices.Count; i++) _devices[i].SyncTo(Now);
        }

        public void Reset()
        {
            Now = 0;
            for (int i = 0; i < _eventTime.Length; i++) _eventTime[i] = Never;
        }

        // Crystal is a leaf, so it cannot carry the core's [SkipInState] - a core serializes this span instead.
        public int StateLength => _eventTime.Length + 1;

        public void CaptureState(Span<long> destination)
        {
            if (destination.Length < StateLength) throw new ArgumentException($"Needs {StateLength} longs.", nameof(destination));
            destination[0] = Now;
            _eventTime.AsSpan().CopyTo(destination[1..]);
        }

        public void RestoreState(ReadOnlySpan<long> source)
        {
            if (source.Length < StateLength) throw new ArgumentException($"Needs {StateLength} longs.", nameof(source));
            Now = source[0];
            source.Slice(1, _eventTime.Length).CopyTo(_eventTime);
        }

        private void CheckId(int eventId)
        {
            if ((uint)eventId >= (uint)_eventTime.Length)
            {
                throw new ArgumentOutOfRangeException(nameof(eventId), $"Event id {eventId} is outside 0..{_eventTime.Length - 1}.");
            }
        }
    }
}
