using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;

namespace EmuSen.WiseMan.Fixtures
{
    // A clock that moves only when a test advances it, firing the timers it made as their time comes, so nothing timed depends on how fast the machine runs.
    public sealed class ManualClock : TimeProvider
    {
        private readonly List<Timer> _timers = new();
        private long _ticks;

        public override long TimestampFrequency => TimeSpan.TicksPerSecond;

        public override long GetTimestamp() => _ticks;

        public override DateTimeOffset GetUtcNow() => new DateTimeOffset(2026, 9, 24, 0, 0, 0, TimeSpan.Zero).AddTicks(_ticks);

        public override ITimer CreateTimer(TimerCallback callback, object? state, TimeSpan dueTime, TimeSpan period)
        {
            var timer = new Timer(this, callback, state);
            timer.Change(dueTime, period);
            return timer;
        }

        // Timers that would fire now or earlier, and have not been cancelled.
        public int Pending => _timers.Count(t => t.Due is not null);

        // Moves the clock on, firing each timer whose time falls inside the step, in order, on the calling thread.
        public void Advance(TimeSpan by)
        {
            long end = _ticks + by.Ticks;
            while (_timers.Where(t => t.Due is long due && due <= end).OrderBy(t => t.Due).FirstOrDefault() is { } next)
            {
                _ticks = Math.Max(_ticks, next.Due!.Value);
                next.Due = next.Period is long period and > 0 ? _ticks + period : null;
                next.Fire();
            }
            _ticks = end;
        }

        private sealed class Timer : ITimer
        {
            private readonly ManualClock _clock;
            private readonly TimerCallback _callback;
            private readonly object? _state;

            public Timer(ManualClock clock, TimerCallback callback, object? state)
            {
                _clock = clock;
                _callback = callback;
                _state = state;
                clock._timers.Add(this);
            }

            public long? Due { get; set; }
            public long? Period { get; private set; }

            public void Fire() => _callback(_state);

            public bool Change(TimeSpan dueTime, TimeSpan period)
            {
                Due = dueTime == Timeout.InfiniteTimeSpan ? null : _clock._ticks + dueTime.Ticks;
                Period = period == Timeout.InfiniteTimeSpan ? null : period.Ticks;
                return true;
            }

            public void Dispose()
            {
                Due = null;
                _clock._timers.Remove(this);
            }

            public System.Threading.Tasks.ValueTask DisposeAsync()
            {
                Dispose();
                return default;
            }
        }
    }
}
