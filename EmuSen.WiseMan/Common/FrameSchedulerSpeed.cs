using System;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Common
{
    // The frontends' call to the pacer, the speed and interval a session's frame info hands them read as the two values it takes - see EmuSen_Settings_Reference.md §4.87.4.
    internal static class FrameSchedulerSpeed
    {
        public static TimeSpan Next(this FrameScheduler scheduler, TimeSpan due, double contentHz, RuntimeSpeed speed) =>
            scheduler.Next(due, contentHz, speed.IsNormalSpeed, speed.FrameInterval(contentHz));

        public static TimeSpan Interval(this FrameScheduler scheduler, double contentHz, RuntimeSpeed speed) =>
            scheduler.Interval(speed.FrameInterval(contentHz));
    }

    // speed_percent and frame_interval_ns as Platform/crystal's speed.rs sets them, with Mistress's turbo and slow motion - see EmuSen_Rewind_And_FastForward.md §2.
    internal sealed class RuntimeSpeed
    {
        public int SpeedPercent { get; set; } = 100;

        public bool IsNormalSpeed => SpeedPercent == 100;

        public void SetTurbo(bool on) => SpeedPercent = on ? 300 : 100;

        public void SetSlowMotion(bool on) => SpeedPercent = on ? 25 : 100;

        public void Reset() => SpeedPercent = 100;

        public TimeSpan FrameInterval(double hz) => SpeedPercent == 0 || hz <= 0 ? TimeSpan.Zero : TimeSpan.FromSeconds(100 / (hz * SpeedPercent));
    }
}
