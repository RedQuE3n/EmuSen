using System;
using System.Diagnostics;
using EmuSen.Serenity;

namespace EmuSen.Common
{
    public enum PacingMode { ContentPaced, DisplayLocked }

    // What one frame's pacing was decided from, and what it implies for the audio - see EmuSen_Settings_Reference.md §4.87.4.
    public readonly record struct PacingDecision(PacingMode Mode, int RefreshesPerFrame, double DisplayHz, double ContentHz, string Reason)
    {
        public bool Locked => Mode == PacingMode.DisplayLocked;

        // Frames a second the emulation runs at.
        public double FrameHz => Locked ? DisplayHz / RefreshesPerFrame : ContentHz;

        // Output samples per input sample that keeps the audio queue level: content seconds run at FrameHz / ContentHz of real time.
        public double AudioRatio => Locked ? ContentHz / FrameHz : 1.0;

        public string Describe() => Locked
            ? $"display-locked {DisplayHz:F2} Hz{(RefreshesPerFrame > 1 ? $" / {RefreshesPerFrame}" : "")} (content {ContentHz:F2})"
            : $"content-paced {ContentHz:F2} Hz ({Reason})";
    }

    // Core-agnostic frame pacing: the core's frame rate and the presenter's measured refresh in, the next frame's deadline out - see §4.87.4.
    public sealed class FrameScheduler
    {
        // The largest share by which a whole divisor of the refresh may differ from the content rate and still be locked to - see §4.87.5.
        public const double DefaultTolerance = 0.01;
        public const int MaxRefreshesPerFrame = 8;

        // How far the smoothed content rate may move before the decision is remade at once rather than smoothed.
        private const double RateJump = 0.002;

        private readonly Func<long, DisplayReading?> _display;
        private readonly long _origin;
        private double _contentHz;

        // Room left between a frame being handed over and the presenter taking it for the vblank - see §4.87.6.
        public TimeSpan Margin { get; set; } = TimeSpan.FromMilliseconds(3);

        private readonly double[] _work = new double[64];
        private readonly double[] _sorted = new double[64];
        private int _workCount, _workHead;

        public bool SyncToDisplay { get; set; } = true;
        public double Tolerance { get; set; } = DefaultTolerance;
        public PacingDecision Decision { get; private set; } = new(PacingMode.ContentPaced, 1, 0, 0, "starting");

        // display gives the presenter's reading at a Stopwatch timestamp; origin is the Stopwatch timestamp the caller's clock started at.
        public FrameScheduler(Func<long, DisplayReading?> display, long origin)
        {
            _display = display;
            _origin = origin;
        }

        public static PacingDecision Decide(double contentHz, DisplayReading? display, bool enabled, bool normalSpeed, double tolerance = DefaultTolerance)
        {
            if (contentHz <= 0) return new(PacingMode.ContentPaced, 1, 0, contentHz, "no content rate");
            if (!enabled) return new(PacingMode.ContentPaced, 1, 0, contentHz, "sync to display off");
            if (!normalSpeed) return new(PacingMode.ContentPaced, 1, 0, contentHz, "speed not 100%");
            if (display is null) return new(PacingMode.ContentPaced, 1, 0, contentHz, "refresh not measured");
            double refresh = display.RefreshHz;
            if (display.Variable) return new(PacingMode.ContentPaced, 1, refresh, contentHz, "refresh follows the content");
            int k = (int)Math.Max(1, Math.Round(refresh / contentHz));
            if (k > MaxRefreshesPerFrame) return new(PacingMode.ContentPaced, 1, refresh, contentHz, $"{refresh:F2} Hz is too far above");
            double error = Math.Abs(refresh / k - contentHz) / contentHz;
            if (error > tolerance) return new(PacingMode.ContentPaced, 1, refresh, contentHz, $"{refresh:F2} Hz{(k > 1 ? $" / {k}" : "")} is {error * 100:F1}% away");
            return new(PacingMode.DisplayLocked, k, refresh, contentHz, "");
        }

        // The deadline after the frame that was due at `due`, both on the caller's clock.
        public TimeSpan Next(TimeSpan due, double contentHz, SpeedController speed)
        {
            long now = Stopwatch.GetTimestamp();
            _contentHz = _contentHz <= 0 || Math.Abs(contentHz - _contentHz) > RateJump * _contentHz ? contentHz : _contentHz + (contentHz - _contentHz) / 32;
            DisplayReading? reading = _display(now);
            Decision = Decide(_contentHz, reading, SyncToDisplay, speed.IsNormalSpeed, Tolerance);
            if (!Decision.Locked) { _held = _upcoming = 0; return due + speed.FrameInterval(contentHz); }

            double period = reading!.PeriodTicks;
            long lead = Lead(period);
            long target = _origin + due.Ticks * Stopwatch.Frequency / TimeSpan.TicksPerSecond + lead + (long)Math.Round(Decision.RefreshesPerFrame * period);
            long grid = reading.NearestVblank(target);
            _held = _upcoming;
            _upcoming = Decision.RefreshesPerFrame > 1 ? grid - (long)Math.Round(period) : 0;
            return TimeSpan.FromTicks((grid - lead - _origin) * TimeSpan.TicksPerSecond / Stopwatch.Frequency);
        }

        private long _held, _upcoming;

        // When a frame spans several refreshes, the earliest its picture may be handed over: the refresh before the one it is for - see §4.87.6.
        public TimeSpan? HandOverNotBefore => Decision.Locked && _held != 0
            ? TimeSpan.FromTicks((_held - _origin) * TimeSpan.TicksPerSecond / Stopwatch.Frequency)
            : null;

        // From the start of a frame to its picture being handed over, so the next can start that long before its vblank.
        public void Completed(TimeSpan work)
        {
            _work[_workHead] = work.Ticks * (double)Stopwatch.Frequency / TimeSpan.TicksPerSecond;
            _workHead = (_workHead + 1) % _work.Length;
            if (_workCount < _work.Length) _workCount++;
        }

        // The frame's work at its 95th percentile plus the margin, held under one frame's span.
        public long Lead(double period)
        {
            double work = 0;
            if (_workCount > 0)
            {
                Array.Copy(_work, _sorted, _workCount);
                Array.Sort(_sorted, 0, _workCount);
                work = _sorted[Math.Min(_workCount - 1, (int)(_workCount * 0.95))];
            }
            double lead = work + Margin.Ticks * (double)Stopwatch.Frequency / TimeSpan.TicksPerSecond;
            return (long)Math.Min(lead, Decision.RefreshesPerFrame * period * 0.9);
        }

        // How long the caller's late-frame rule should treat one frame as.
        public TimeSpan Interval(double contentHz, SpeedController speed) => Decision.Locked
            ? TimeSpan.FromSeconds(Decision.RefreshesPerFrame / Decision.DisplayHz)
            : speed.FrameInterval(contentHz);
    }
}
