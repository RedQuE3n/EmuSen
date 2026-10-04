using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using EmuSen.Common;
using EmuSen.Mistress;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Common
{
    // Mistress's emulation loop on a virtual clock, against a modelled panel and presenter - see EmuSen_Settings_Reference.md §4.87.13.
    public static class PacingLoopModel
    {
        public enum Presents { Grid, FollowDraws }

        public sealed class Options
        {
            public double StartSeconds;
            public double Seconds = 30;
            public double ContentHz = 59.96;
            public double PanelHz = 119.90;
            public Func<int, double> CostMs = _ => 4.0;
            public Func<double, double> StallMs = _ => 0;
            public Func<double, Presents> PresentsAt = _ => Presents.Grid;
            public Func<double, double> FollowDelayMs = _ => 0.3;
            public DisplayReading? FixedReading;
        }

        public sealed record Result(List<double> FrameStarts, List<string> Decisions)
        {
            // The most frame starts inside any window of the given length.
            public int MostIn(double windowSeconds)
            {
                int best = 0, j = 0;
                for (int i = 0; i < FrameStarts.Count; i++)
                {
                    while (FrameStarts[i] - FrameStarts[j] >= windowSeconds) j++;
                    best = Math.Max(best, i - j + 1);
                }
                return best;
            }

            // Milliseconds between consecutive frame starts.
            public double[] IntervalsMs() => Enumerable.Range(1, FrameStarts.Count - 1).Select(i => (FrameStarts[i] - FrameStarts[i - 1]) * 1000).ToArray();

            public double MeanHz => (FrameStarts.Count - 1) / (FrameStarts[^1] - FrameStarts[0]);
        }

        public static Result Run(Options o)
        {
            long f = Stopwatch.Frequency;
            double panel = f / o.PanelHz;
            long now = (long)(o.StartSeconds * f), end = now + (long)(o.Seconds * f), presentCount = 0, lastPresent = 0;
            var clock = new DisplayClock();
            var scheduler = new FrameScheduler(_ => o.FixedReading ?? clock.Current(now), 0);
            var speed = new SpeedController();
            TimeSpan nextTick = ToSpan(now);
            var starts = new List<double>();
            var decisions = new List<string>();
            for (int i = 0; now < end; i++)
            {
                nextTick = scheduler.Next(nextTick, o.ContentHz, speed);
                long frameStart = now;
                double s = (double)now / f;
                starts.Add(s);
                if (decisions.Count == 0 || decisions[^1] != scheduler.Decision.Describe()) decisions.Add(scheduler.Decision.Describe());
                now += (long)((o.CostMs(i) + o.StallMs(s)) * f / 1000);
                long held = 0;
                if (scheduler.HandOverNotBefore is { } notBefore && ToSpan(now) < notBefore) { held = ToTicks(notBefore) - now; now += held; }
                long present = o.PresentsAt(s) == Presents.Grid
                    ? (long)(Math.Ceiling((now + f / 2000.0) / panel) * panel)
                    : now + (long)(o.FollowDelayMs(s) * f / 1000);
                if (present <= lastPresent) present = lastPresent + (long)panel;
                lastPresent = present;
                clock.ObserveVblank(present / (f / 1_000_000), ++presentCount, present);
                scheduler.Completed(ToSpan(now - frameStart - held));
                if (nextTick > ToSpan(now)) now = ToTicks(nextTick);
                else nextTick = FramePacer.Settle(nextTick, ToSpan(now), scheduler.Interval(o.ContentHz, speed));
            }
            return new Result(starts, decisions);
        }

        private static TimeSpan ToSpan(long ticks) => FrameScheduler.FromStopwatch(ticks);

        private static long ToTicks(TimeSpan t) => FrameScheduler.ToStopwatch(t);
    }
}
