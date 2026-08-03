using EmuSen.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // The core-agnostic timeline - see EmuSen_Crystal_Scheduler.md §3 and §4.
    public class SchedulerTests
    {
        private sealed class Recorder : IScheduleHandler
        {
            public readonly List<(int Id, long Now)> Fired = new();
            public Action<int, long>? Also;

            public void OnScheduledEvent(int eventId, long now)
            {
                Fired.Add((eventId, now));
                Also?.Invoke(eventId, now);
            }
        }

        private sealed class Device : IClockedDevice
        {
            public long Clock { get; private set; }
            public int Syncs;

            public void SyncTo(long masterClock)
            {
                Syncs++;
                if (masterClock > Clock) Clock = masterClock;
            }
        }

        private static (Scheduler S, Recorder R) Build(int capacity = 8)
        {
            var s = new Scheduler(capacity);
            var r = new Recorder();
            s.SetHandler(r);
            return (s, r);
        }

        [Fact]
        public void A_new_scheduler_has_nothing_pending()
        {
            var (s, _) = Build();

            Assert.Equal(0, s.Now);
            Assert.Equal(Scheduler.Never, s.NextEventTime());
            Assert.Equal(-1, s.NextEventId());
        }

        [Fact]
        public void Events_fire_in_time_order()
        {
            var (s, r) = Build();
            s.At(300, 2);
            s.At(100, 1);
            s.At(200, 0);

            s.RunUntil(1000);

            Assert.Equal(new[] { (1, 100L), (0, 200L), (2, 300L) }, r.Fired);
        }

        [Fact]
        public void An_event_past_the_deadline_does_not_fire()
        {
            var (s, r) = Build();
            s.At(50, 0);
            s.At(500, 1);

            s.RunUntil(100);

            Assert.Equal(new[] { (0, 50L) }, r.Fired);
            Assert.True(s.IsScheduled(1));
            Assert.Equal(100, s.Now);
        }

        // Determinism matters more than which order wins - digests depend on it.
        [Fact]
        public void Two_events_at_the_same_time_fire_lowest_id_first()
        {
            var (s, r) = Build();
            s.At(100, 5);
            s.At(100, 2);

            s.RunUntil(100);

            Assert.Equal(new[] { (2, 100L), (5, 100L) }, r.Fired);
        }

        [Fact]
        public void Firing_an_event_clears_it_so_a_handler_can_reschedule()
        {
            var (s, r) = Build();
            s.At(100, 0);
            r.Also = (id, now) => { if (now < 400) s.At(now + 100, 0); };

            s.RunUntil(1000);

            Assert.Equal(new[] { (0, 100L), (0, 200L), (0, 300L), (0, 400L) }, r.Fired);
            Assert.False(s.IsScheduled(0));
        }

        [Fact]
        public void Cancel_removes_a_pending_event()
        {
            var (s, r) = Build();
            s.At(100, 3);
            s.Cancel(3);

            s.RunUntil(1000);

            Assert.Empty(r.Fired);
        }

        // The case per-scanline scheduling could never express: HTime changing mid-frame.
        [Fact]
        public void Rescheduling_moves_an_event_rather_than_adding_one()
        {
            var (s, r) = Build();
            s.At(100, 0);
            s.At(700, 0);

            s.RunUntil(1000);

            Assert.Equal(new[] { (0, 700L) }, r.Fired);
        }

        // An instruction can overshoot a deadline, so a past-due time is legal - see §4.
        [Fact]
        public void An_event_scheduled_in_the_past_fires_at_the_current_instant()
        {
            var (s, r) = Build();
            s.RunUntil(500);
            s.At(200, 0);

            s.RunUntil(600);

            Assert.Equal(new[] { (0, 500L) }, r.Fired);
        }

        [Fact]
        public void Now_never_runs_backwards_across_a_past_due_event()
        {
            var (s, r) = Build();
            s.RunUntil(500);
            s.At(10, 0);
            s.At(520, 1);

            s.RunUntil(600);

            Assert.Equal(new[] { (0, 500L), (1, 520L) }, r.Fired);
            Assert.Equal(600, s.Now);
        }

        [Fact]
        public void In_schedules_relative_to_now()
        {
            var (s, r) = Build();
            s.RunUntil(1000);
            s.In(250, 0);

            Assert.Equal(1250, s.TimeOf(0));

            s.RunUntil(2000);
            Assert.Equal(new[] { (0, 1250L) }, r.Fired);
        }

        // A runaway reschedule should name itself, not hang the frame - see §4.
        [Fact]
        public void An_event_that_never_advances_time_throws_instead_of_hanging()
        {
            var (s, r) = Build();
            s.At(100, 4);
            r.Also = (id, now) => s.At(now, 4);

            var ex = Assert.Throws<InvalidOperationException>(() => s.RunUntil(1000));
            Assert.Contains("4", ex.Message);
        }

        [Fact]
        public void An_event_id_outside_capacity_is_rejected()
        {
            var (s, _) = Build(capacity: 4);

            Assert.Throws<ArgumentOutOfRangeException>(() => s.At(10, 4));
            Assert.Throws<ArgumentOutOfRangeException>(() => s.At(10, -1));
        }

        [Fact]
        public void Sync_runs_a_device_up_to_now_and_no_further()
        {
            var (s, _) = Build();
            var d = new Device();
            s.AddDevice(d);
            s.RunUntil(900);

            s.Sync(d);

            Assert.Equal(900, d.Clock);
        }

        [Fact]
        public void SyncAll_catches_up_every_registered_device()
        {
            var (s, _) = Build();
            var a = new Device();
            var b = new Device();
            s.AddDevice(a);
            s.AddDevice(b);
            s.RunUntil(400);

            s.SyncAll();

            Assert.Equal(400, a.Clock);
            Assert.Equal(400, b.Clock);
        }

        [Fact]
        public void A_device_is_only_registered_once()
        {
            var (s, _) = Build();
            var d = new Device();
            s.AddDevice(d);
            s.AddDevice(d);

            s.SyncAll();

            Assert.Single(s.Devices);
            Assert.Equal(1, d.Syncs);
        }

        // A lagging device is the whole basis of moving the PPU off-thread.
        [Fact]
        public void A_device_may_lag_until_something_syncs_it()
        {
            var (s, _) = Build();
            var d = new Device();
            s.AddDevice(d);

            s.RunUntil(5000);

            Assert.Equal(0, d.Clock);
            Assert.Equal(5000, s.Now);
        }

        [Fact]
        public void Advance_moves_the_clock_without_dispatching()
        {
            var (s, r) = Build();
            s.At(50, 0);

            s.Advance(100);

            Assert.Equal(100, s.Now);
            Assert.Empty(r.Fired);
        }

        // The overshoot case: the CPU runs past a deadline, then the event fires where it actually landed.
        [Fact]
        public void An_event_overrun_by_Advance_fires_at_the_overshot_clock()
        {
            var (s, r) = Build();
            s.At(1000, 0);

            s.Advance(1007);
            s.RunUntil(1000);

            Assert.Equal(new[] { (0, 1007L) }, r.Fired);
            Assert.Equal(1007, s.Now);
        }

        [Fact]
        public void Advancing_backwards_is_rejected()
        {
            var (s, _) = Build();

            Assert.Throws<ArgumentOutOfRangeException>(() => s.Advance(-1));
        }

        [Fact]
        public void State_round_trips_through_a_span()
        {
            var (s, _) = Build();
            s.At(700, 1);
            s.At(900, 3);
            s.RunUntil(500);

            long[] buffer = new long[s.StateLength];
            s.CaptureState(buffer);

            var restored = new Scheduler(8);
            restored.RestoreState(buffer);

            Assert.Equal(s.Now, restored.Now);
            Assert.Equal(700, restored.TimeOf(1));
            Assert.Equal(900, restored.TimeOf(3));
            Assert.False(restored.IsScheduled(0));
        }

        [Fact]
        public void Reset_clears_the_clock_and_every_event()
        {
            var (s, _) = Build();
            s.At(100, 0);
            s.RunUntil(50);

            s.Reset();

            Assert.Equal(0, s.Now);
            Assert.Equal(Scheduler.Never, s.NextEventTime());
        }
    }
}
