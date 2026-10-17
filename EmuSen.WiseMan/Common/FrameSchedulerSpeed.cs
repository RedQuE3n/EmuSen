using System;
using EmuSen.Common;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Common
{
    // The frontends' call to the pacer, a SpeedController read as the two values it takes - see EmuSen_Settings_Reference.md §4.87.4.
    internal static class FrameSchedulerSpeed
    {
        public static TimeSpan Next(this FrameScheduler scheduler, TimeSpan due, double contentHz, SpeedController speed) =>
            scheduler.Next(due, contentHz, speed.IsNormalSpeed, speed.FrameInterval(contentHz));

        public static TimeSpan Interval(this FrameScheduler scheduler, double contentHz, SpeedController speed) =>
            scheduler.Interval(speed.FrameInterval(contentHz));
    }
}
