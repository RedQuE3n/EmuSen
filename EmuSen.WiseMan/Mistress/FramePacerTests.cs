using System;
using EmuSen.Mistress;

namespace EmuSen.WiseMan.Mistress
{
    // A late frame's time is owed up to three frames and forgotten past that - see EmuSen_Settings_Reference.md §4.28.
    public class FramePacerTests
    {
        private static readonly TimeSpan Interval = TimeSpan.FromMilliseconds(20);

        [Fact]
        public void A_frame_ten_milliseconds_late_keeps_its_tick_so_the_next_frames_make_it_up()
        {
            TimeSpan tick = TimeSpan.FromMilliseconds(1000);
            Assert.Equal(tick, FramePacer.Settle(tick, TimeSpan.FromMilliseconds(1010), Interval));
        }

        [Fact]
        public void A_stall_longer_than_the_cap_is_forgotten_down_to_the_cap()
        {
            TimeSpan tick = TimeSpan.FromMilliseconds(1000);
            Assert.Equal(TimeSpan.FromMilliseconds(1440), FramePacer.Settle(tick, TimeSpan.FromMilliseconds(1500), Interval));
        }

        // A game frame of thirty milliseconds and two of nine, against a twenty millisecond slot: the old rule lost ten milliseconds every three frames.
        [Fact]
        public void Frames_that_average_under_their_slot_hold_full_speed()
        {
            TimeSpan tick = TimeSpan.Zero, now = TimeSpan.Zero;
            foreach (int cost in new[] { 30, 9, 9, 30, 9, 9, 30, 9, 9 })
            {
                tick += Interval;
                now += TimeSpan.FromMilliseconds(cost);
                if (now < tick) now = tick; else tick = FramePacer.Settle(tick, now, Interval);
            }

            Assert.Equal(TimeSpan.FromMilliseconds(180), now);
        }
    }
}
