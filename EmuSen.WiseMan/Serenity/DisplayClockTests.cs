using System;
using System.Diagnostics;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Serenity
{
    // The refresh measured from what the presenter saw, and when it is called variable - see EmuSen_Settings_Reference.md §4.87.3.
    public class DisplayClockTests
    {
        private static readonly long Second = Stopwatch.Frequency;
        private const long Base = 1_000_000_000_000; // a UST of a machine up for a while, in microseconds

        private static long Ticks(long ust) => ust * (Second / 1_000_000);

        // Counted vblanks at hz, one sample every `every` vblanks, as draws see them.
        private static DisplayClock Counted(double hz, int samples, int every = 1, long firstCount = 5000, DisplayClock? into = null, long startUst = Base)
        {
            DisplayClock clock = into ?? new DisplayClock();
            for (int i = 0; i < samples; i++)
            {
                long n = (long)i * every;
                long ust = startUst + (long)Math.Round(n * 1e6 / hz);
                clock.ObserveVblank(ust, firstCount + n, Ticks(ust) + Second / 1000);
            }
            return clock;
        }

        [Theory]
        [InlineData(60.0)]
        [InlineData(59.95)]
        [InlineData(119.90)]
        [InlineData(164.90)]
        [InlineData(74.92)]
        public void A_fixed_refresh_is_measured_from_the_vblank_counts(double hz)
        {
            DisplayReading? reading = Counted(hz, 240).Reading;
            Assert.NotNull(reading);
            Assert.Equal(hz, reading!.RefreshHz, 2);
            Assert.False(reading.Variable);
            Assert.True(reading.Counted);
        }

        [Fact]
        public void Draws_on_every_second_vblank_still_measure_the_display_not_the_draws()
        {
            DisplayReading reading = Counted(119.90, 240, every: 2).Reading!;
            Assert.Equal(119.90, reading.RefreshHz, 2);
            Assert.False(reading.Variable);
        }

        [Fact]
        public void There_is_no_reading_before_enough_samples()
        {
            Assert.Null(Counted(60.0, DisplayClock.MinimumSamples - 1).Reading);
            Assert.NotNull(Counted(60.0, DisplayClock.MinimumSamples).Reading);
        }

        [Fact]
        public void The_reading_extrapolates_the_vblank_lattice()
        {
            DisplayReading reading = Counted(60.0, 240).Reading!;
            long last = Ticks(Base + (long)Math.Round(239 * 1e6 / 60.0));
            Assert.InRange(reading.VblankTicks - last, -Second / 100_000, Second / 100_000);
            long next = reading.NextVblank(last + (long)(reading.PeriodTicks / 2));
            Assert.InRange(next - last - reading.PeriodTicks, -Second / 1_000_000, Second / 1_000_000);
            Assert.InRange(reading.NearestVblank(last + (long)(reading.PeriodTicks * 2.4)) - last - 2 * reading.PeriodTicks, -Second / 1_000_000, Second / 1_000_000);
        }

        [Fact]
        public void A_refresh_that_follows_irregular_content_is_variable()
        {
            var clock = new DisplayClock();
            var random = new Random(7);
            long ust = Base;
            for (int i = 0; i < 240; i++)
            {
                ust += 14_000 + random.Next(0, 9_000); // 43 to 71 Hz, frame by frame
                clock.ObserveVblank(ust, 100 + i, Ticks(ust));
            }
            Assert.True(clock.Reading!.Variable);
        }

        [Fact]
        public void A_display_change_is_measured_again_once_the_window_has_turned_over()
        {
            DisplayClock clock = Counted(60.0, 240);
            Assert.Equal(60.0, clock.Reading!.RefreshHz, 2);
            long resume = Base + 5_000_000;
            Counted(119.90, 40, firstCount: 6000, into: clock, startUst: resume);
            Assert.True(clock.Reading!.Variable); // two lattices in one window fit neither
            Counted(119.90, 240, firstCount: 6040, into: clock, startUst: resume + (long)Math.Round(40 * 1e6 / 119.90));
            Assert.Equal(119.90, clock.Reading!.RefreshHz, 2);
            Assert.False(clock.Reading.Variable);
        }

        [Fact]
        public void A_count_that_goes_backwards_starts_the_measurement_again()
        {
            DisplayClock clock = Counted(60.0, 240, firstCount: 9000);
            Counted(120.0, 10, firstCount: 10, into: clock, startUst: Base + 10_000_000);
            Assert.Null(clock.Reading);
        }

        [Fact]
        public void A_time_from_another_clock_is_ignored()
        {
            var clock = new DisplayClock();
            for (int i = 0; i < 240; i++) clock.ObserveVblank(Base + i * 16_667, i, Ticks(Base) + 3600 * Second);
            Assert.Null(clock.Reading);
        }

        [Fact]
        public void Uncounted_ticks_measure_a_steady_presenter_and_flag_an_unsteady_one()
        {
            var steady = new DisplayClock();
            var random = new Random(3);
            double period = Second / 60.0;
            for (int i = 0; i < 240; i++) steady.ObserveTick((long)(Ticks(Base) + i * period + random.Next(-(int)(period * 0.02), (int)(period * 0.02))));
            Assert.Equal(60.0, steady.Reading!.RefreshHz, 1);
            Assert.False(steady.Reading.Variable);
            Assert.False(steady.Reading.Counted);

            var unsteady = new DisplayClock();
            long t = Ticks(Base);
            for (int i = 0; i < 240; i++) { t += (long)(period * (0.7 + random.NextDouble() * 0.6)); unsteady.ObserveTick(t); }
            Assert.True(unsteady.Reading!.Variable);
        }

        [Fact]
        public void A_reading_goes_stale_when_the_presenter_stops_drawing()
        {
            DisplayClock clock = Counted(60.0, 240);
            long taken = clock.Reading!.TakenTicks;
            Assert.NotNull(clock.Current(taken + Second));
            Assert.Null(clock.Current(taken + DisplayClock.StaleTicks + 1));
        }
    }
}
