using System;
using System.Diagnostics;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Serenity;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Common
{
    // The pacing decision across rates and refreshes, the deadlines it yields, and the fallback - see EmuSen_Settings_Reference.md §4.87.4 and §4.87.5.
    public class FrameSchedulerTests
    {
        private static DisplayReading Display(double hz, bool variable = false, long vblank = 0) =>
            new(Stopwatch.Frequency / hz, vblank, variable, true, 240, 0);

        private static PacingDecision Decide(double content, double refresh) =>
            FrameScheduler.Decide(content, Display(refresh), enabled: true, normalSpeed: true);

        [Theory]
        [InlineData(60.0988, 60.0, 1)]    // NES, SNES
        [InlineData(59.7275, 60.0, 1)]    // Game Boy
        [InlineData(59.94, 60.0, 1)]
        [InlineData(60.0988, 119.90, 2)]  // the handheld's panel
        [InlineData(59.7275, 119.90, 2)]
        [InlineData(60.0988, 120.0, 2)]
        [InlineData(60.0, 240.0, 4)]
        [InlineData(50.007, 50.0, 1)]
        [InlineData(50.007, 100.0, 2)]
        public void A_whole_divisor_of_the_refresh_within_tolerance_is_locked_to(double content, double refresh, int k)
        {
            PacingDecision d = Decide(content, refresh);
            Assert.Equal(PacingMode.DisplayLocked, d.Mode);
            Assert.Equal(k, d.RefreshesPerFrame);
            Assert.Equal(refresh / k, d.FrameHz, 9);
            Assert.Equal(content / (refresh / k), d.AudioRatio, 12);
        }

        [Theory]
        [InlineData(50.007, 60.0)]     // PAL on a 60 Hz display
        [InlineData(60.0, 144.0)]      // 72 is 20% away, 48 is further
        [InlineData(60.0988, 164.90)]  // the desktop's display
        [InlineData(60.0988, 74.92)]
        [InlineData(60.0988, 75.0)]
        [InlineData(59.7275, 61.0)]    // 2.1% away
        [InlineData(60.0, 30.0)]       // a display slower than the content
        public void A_refresh_with_no_divisor_near_the_content_rate_keeps_content_pacing(double content, double refresh)
        {
            PacingDecision d = Decide(content, refresh);
            Assert.Equal(PacingMode.ContentPaced, d.Mode);
            Assert.Equal(content, d.FrameHz);
            Assert.Equal(1.0, d.AudioRatio);
        }

        [Fact]
        public void The_tolerance_is_one_percent_and_its_edge_is_inside()
        {
            Assert.Equal(0.01, FrameScheduler.DefaultTolerance);
            Assert.True(FrameScheduler.Decide(100.0, Display(101.0), true, true).Locked);
            Assert.True(FrameScheduler.Decide(100.0, Display(99.0), true, true).Locked);
            Assert.False(FrameScheduler.Decide(100.0, Display(101.01), true, true).Locked);
            Assert.False(FrameScheduler.Decide(100.0, Display(98.99), true, true).Locked);
            Assert.True(FrameScheduler.Decide(100.0, Display(103.0), true, true, tolerance: 0.05).Locked);
        }

        [Fact]
        public void The_setting_the_speed_an_unmeasured_and_a_variable_refresh_each_fall_back()
        {
            Assert.True(FrameScheduler.Decide(60.0988, Display(60), true, true).Locked);
            Assert.False(FrameScheduler.Decide(60.0988, Display(60), enabled: false, normalSpeed: true).Locked);
            Assert.False(FrameScheduler.Decide(60.0988, Display(60), enabled: true, normalSpeed: false).Locked);
            Assert.False(FrameScheduler.Decide(60.0988, null, true, true).Locked);
            Assert.False(FrameScheduler.Decide(60.0988, Display(60, variable: true), true, true).Locked);
            Assert.False(FrameScheduler.Decide(0, Display(60), true, true).Locked);
            Assert.False(FrameScheduler.Decide(60.0, Display(1000), true, true).Locked);
        }

        [Fact]
        public void The_description_names_the_mode_and_both_rates()
        {
            Assert.Equal("display-locked 60.00 Hz (content 60.10)", Decide(60.0988, 60.0).Describe());
            Assert.Equal("display-locked 119.90 Hz / 2 (content 60.10)", Decide(60.0988, 119.90).Describe());
            Assert.StartsWith("content-paced 50.01 Hz", Decide(50.007, 60.0).Describe());
        }

        private static (FrameScheduler, long) Scheduler(Func<DisplayReading?> reading)
        {
            long origin = Stopwatch.GetTimestamp();
            return (new FrameScheduler(_ => reading(), origin) { Margin = TimeSpan.Zero }, origin);
        }

        [Fact]
        public void Locked_deadlines_fall_on_every_kth_vblank_and_never_drift_off_the_lattice()
        {
            double period = Stopwatch.Frequency / 119.90;
            long origin = 0;
            DisplayReading reading = null!;
            (FrameScheduler scheduler, origin) = Scheduler(() => reading);
            reading = new DisplayReading(period, origin + 12345, false, true, 240, 0);
            var speed = new SpeedController();
            TimeSpan due = TimeSpan.FromMilliseconds(3.7);
            TimeSpan previous = due;
            for (int i = 0; i < 2000; i++)
            {
                due = scheduler.Next(due, 60.0988, speed);
                double ticks = due.Ticks * (double)Stopwatch.Frequency / TimeSpan.TicksPerSecond - 12345;
                double phase = ticks / period - Math.Round(ticks / period);
                Assert.True(Math.Abs(phase) < 0.01, $"frame {i} is {phase:F4} of a period off a vblank");
                if (i > 0) Assert.Equal(2.0, (due - previous).Ticks * (double)Stopwatch.Frequency / TimeSpan.TicksPerSecond / period, 2);
                previous = due;
            }
            Assert.Equal(2, scheduler.Decision.RefreshesPerFrame);
        }

        [Fact]
        public void Content_paced_deadlines_are_the_speed_controllers_interval_exactly()
        {
            (FrameScheduler scheduler, _) = Scheduler(() => null);
            var speed = new SpeedController();
            TimeSpan due = TimeSpan.FromSeconds(1);
            Assert.Equal(due + speed.FrameInterval(60.0988), scheduler.Next(due, 60.0988, speed));
            speed.SpeedPercent = 300;
            Assert.Equal(due + speed.FrameInterval(60.0988), scheduler.Next(due, 60.0988, speed));
            Assert.Equal(speed.FrameInterval(60.0988), scheduler.Interval(60.0988, speed));
        }

        [Fact]
        public void Fast_forward_and_slow_motion_leave_the_lock_and_normal_speed_returns_to_it()
        {
            DisplayReading reading = Display(60.0, vblank: Stopwatch.GetTimestamp());
            (FrameScheduler scheduler, _) = Scheduler(() => reading);
            var speed = new SpeedController();
            TimeSpan due = scheduler.Next(TimeSpan.Zero, 60.0988, speed);
            Assert.True(scheduler.Decision.Locked);
            speed.SetTurbo(true);
            TimeSpan turbo = scheduler.Next(due, 60.0988, speed);
            Assert.False(scheduler.Decision.Locked);
            Assert.Equal(due + speed.FrameInterval(60.0988), turbo);
            speed.SetSlowMotion(true);
            scheduler.Next(turbo, 60.0988, speed);
            Assert.False(scheduler.Decision.Locked);
            speed.Reset();
            scheduler.Next(turbo, 60.0988, speed);
            Assert.True(scheduler.Decision.Locked);
        }

        [Fact]
        public void A_rate_that_changes_mid_game_is_decided_again_at_once()
        {
            DisplayReading reading = Display(60.0, vblank: Stopwatch.GetTimestamp());
            (FrameScheduler scheduler, _) = Scheduler(() => reading);
            var speed = new SpeedController();
            TimeSpan due = TimeSpan.Zero;
            for (int i = 0; i < 10; i++) due = scheduler.Next(due, 60.0988, speed);
            Assert.True(scheduler.Decision.Locked);
            due = scheduler.Next(due, 50.007, speed); // a region switch
            Assert.False(scheduler.Decision.Locked);
            Assert.Equal(50.007, scheduler.Decision.ContentHz, 6);
            due = scheduler.Next(due, 59.94, speed);
            Assert.True(scheduler.Decision.Locked);
        }

        [Fact]
        public void A_rate_that_wobbles_frame_to_frame_is_smoothed_and_does_not_flap()
        {
            DisplayReading reading = Display(60.0, vblank: Stopwatch.GetTimestamp());
            (FrameScheduler scheduler, _) = Scheduler(() => reading);
            var speed = new SpeedController();
            TimeSpan due = TimeSpan.Zero;
            for (int i = 0; i < 500; i++)
            {
                due = scheduler.Next(due, 60.0 + (i % 2 == 0 ? 0.05 : -0.05), speed);
                Assert.True(scheduler.Decision.Locked);
                Assert.InRange(scheduler.Decision.ContentHz, 59.94, 60.06);
            }
        }

        [Fact]
        public void A_display_that_changes_is_followed_and_one_that_is_lost_falls_back()
        {
            DisplayReading? reading = Display(60.0, vblank: Stopwatch.GetTimestamp());
            (FrameScheduler scheduler, _) = Scheduler(() => reading);
            var speed = new SpeedController();
            TimeSpan due = scheduler.Next(TimeSpan.Zero, 60.0988, speed);
            Assert.Equal(1, scheduler.Decision.RefreshesPerFrame);
            reading = Display(120.0, vblank: Stopwatch.GetTimestamp());
            due = scheduler.Next(due, 60.0988, speed);
            Assert.Equal(2, scheduler.Decision.RefreshesPerFrame);
            reading = Display(144.0, vblank: Stopwatch.GetTimestamp());
            due = scheduler.Next(due, 60.0988, speed);
            Assert.False(scheduler.Decision.Locked);
            reading = null;
            scheduler.Next(due, 60.0988, speed);
            Assert.False(scheduler.Decision.Locked);
        }

        [Fact]
        public void The_lead_is_the_frames_work_plus_the_margin_and_under_one_frame()
        {
            DisplayReading reading = Display(60.0, vblank: Stopwatch.GetTimestamp());
            long origin = Stopwatch.GetTimestamp();
            var scheduler = new FrameScheduler(_ => reading, origin) { Margin = TimeSpan.FromMilliseconds(3) };
            scheduler.Next(TimeSpan.Zero, 60.0, new SpeedController());
            double ms = Stopwatch.Frequency / 1000.0;
            Assert.Equal(3.0, scheduler.Lead(reading.PeriodTicks) / ms, 3);
            for (int i = 0; i < 64; i++) scheduler.Completed(TimeSpan.FromMilliseconds(2));
            Assert.Equal(5.0, scheduler.Lead(reading.PeriodTicks) / ms, 3);
            for (int i = 0; i < 64; i++) scheduler.Completed(TimeSpan.FromMilliseconds(40));
            Assert.Equal(15.0, scheduler.Lead(reading.PeriodTicks) / ms, 3);
        }
    }

    // Pacing changes when frames run, never what they compute - see EmuSen_Settings_Reference.md §4.87.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class PacingLeavesTheStateAloneTests
    {
        [Fact]
        public void The_state_after_n_frames_is_the_same_locked_and_content_paced()
        {
            string rom = Path.Combine(Path.GetTempPath(), $"EmuSenPacing_{Guid.NewGuid():N}.nes");
            bool battery = CoreOptions.BatteryRamDisabled;
            CoreOptions.BatteryRamDisabled = true;
            try
            {
                File.WriteAllBytes(rom, BoardPrograms.Build(1, 8, cycleIrq: false));
                CadenceBench.Result Run(bool sync, int frames) => CadenceBench.Run(new CadenceBench.Options
                    { Rom = rom, DisplayHz = 240.0, Seconds = 60, Frames = frames, SyncToDisplay = sync, Audio = false, ContentHzOverride = 240.4 });
                CadenceBench.Result content = Run(false, 300), locked = Run(true, 300), longer = Run(true, 301);
                Assert.StartsWith("content-paced", content.Decision);
                Assert.StartsWith("display-locked", locked.Decision);
                Assert.Equal(content.StateSha, locked.StateSha);
                Assert.NotEqual(locked.StateSha, longer.StateSha);
            }
            finally
            {
                CoreOptions.BatteryRamDisabled = battery;
                try { File.Delete(rom); } catch { }
            }
        }
    }
}
