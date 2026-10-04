using System;
using System.Diagnostics;
using System.Linq;
using EmuSen.Common;
using EmuSen.Mistress;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Common
{
    // Emulated frames per second of the loop's own clock, locked and not, late in a session, after stalls and on presents that change - see EmuSen_Settings_Reference.md §4.87.13.
    public class PacingLoopModelTests
    {
        private static readonly DisplayReading Panel = new(Stopwatch.Frequency / 119.90, 0, false, true, 240, 0);
        private const double LockedMs = 2000 / 119.90;

        // Every interval after the lock's first alignment is two refreshes.
        private static void AssertSteady(PacingLoopModel.Result r)
        {
            double[] iv = r.IntervalsMs();
            for (int i = 3; i < iv.Length; i++) Assert.True(Math.Abs(iv[i] - LockedMs) < 0.01, $"interval {i} at {r.FrameStarts[i]:F3} s is {iv[i]:F2} ms");
        }

        [Theory]
        [InlineData(0.0)]
        [InlineData(900.0)]
        [InlineData(925.0)]
        [InlineData(1900.0)]
        [InlineData(3 * 3600.0)]
        public void A_locked_session_never_runs_faster_than_the_lock_however_long_it_has_run(double startSeconds)
        {
            var r = PacingLoopModel.Run(new() { StartSeconds = startSeconds, Seconds = 20, FixedReading = Panel });
            Assert.StartsWith("display-locked 119.90 Hz / 2", r.Decisions[^1]);
            AssertSteady(r);
            Assert.InRange(r.MostIn(1.0), 59, 61);
            Assert.Equal(119.90 / 2, r.MeanHz, 1);
        }

        [Fact]
        public void The_lock_holds_across_the_moment_the_old_arithmetic_overflowed()
        {
            double overflow = (double)long.MaxValue / Stopwatch.Frequency / TimeSpan.TicksPerSecond;
            var r = PacingLoopModel.Run(new() { StartSeconds = overflow - 10, Seconds = 20, FixedReading = Panel });
            AssertSteady(r);
            Assert.Contains(r.FrameStarts, s => s > overflow + 5);
        }

        [Fact]
        public void The_deadline_and_the_hold_are_on_the_lattice_a_day_into_a_session()
        {
            double period = Stopwatch.Frequency / 119.90;
            var scheduler = new FrameScheduler(_ => Panel, 0) { Margin = TimeSpan.Zero };
            var speed = new SpeedController();
            TimeSpan due = TimeSpan.FromSeconds(86_400);
            TimeSpan first = scheduler.Next(due, 59.96, speed);
            TimeSpan second = scheduler.Next(first, 59.96, speed);
            Assert.Equal(LockedMs, (second - first).TotalMilliseconds, 3);
            Assert.InRange((first - due).TotalMilliseconds, LockedMs - 4.2, LockedMs + 4.2);
            Assert.Equal((first - TimeSpan.FromSeconds(1 / 119.90)).TotalMilliseconds, scheduler.HandOverNotBefore!.Value.TotalMilliseconds, 3);
            double phase = FrameScheduler.ToStopwatch(second) / period;
            Assert.Equal(Math.Round(phase), phase, 3);
        }

        [Theory]
        [InlineData(60.0)]
        [InlineData(250.0)]
        [InlineData(2000.0)]
        public void A_stall_is_repaid_by_at_most_the_debt_and_never_as_a_sustained_burst(double stallMs)
        {
            var r = PacingLoopModel.Run(new() { StartSeconds = 1000, Seconds = 30, FixedReading = Panel, StallMs = s => Math.Abs(s - 1010.0) < 0.008 ? stallMs : 0 });
            int backToBack = 0, run = 0;
            foreach (double ms in r.IntervalsMs())
            {
                run = ms < 10 ? run + 1 : 0;
                backToBack = Math.Max(backToBack, run);
            }
            Assert.InRange(backToBack, 0, FramePacer.DebtFrames + 1);
            Assert.InRange(r.MostIn(1.0), 59, 61 + FramePacer.DebtFrames);
            Assert.InRange(r.MostIn(3.0), 178, 181 + FramePacer.DebtFrames);
        }

        [Fact]
        public void Presents_that_switch_between_the_panels_grid_and_the_draws_stay_within_the_tolerance()
        {
            var r = PacingLoopModel.Run(new()
            {
                StartSeconds = 900, Seconds = 60,
                PresentsAt = s => ((int)(s / 7)) % 2 == 0 ? PacingLoopModel.Presents.Grid : PacingLoopModel.Presents.FollowDraws,
                FollowDelayMs = s => 0.2 + 0.3 * Math.Sin(s * 13.7),
                CostMs = i => i % 2 == 0 ? 9.0 : 3.0,
            });
            Assert.Contains(r.Decisions, d => d.StartsWith("display-locked"));
            Assert.InRange(r.MostIn(1.0), 59, 61);
            Assert.InRange(r.MeanHz, 59.96 * (1 - FrameScheduler.DefaultTolerance), 59.96 * (1 + FrameScheduler.DefaultTolerance));
        }
    }
}
