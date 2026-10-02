using System;
using System.Diagnostics;

namespace EmuSen.Serenity
{
    // The display's refresh as the presenter observed it: period, a vblank to extrapolate from, and whether it holds still - see EmuSen_Settings_Reference.md §4.87.3.
    public sealed record DisplayReading(double PeriodTicks, long VblankTicks, bool Variable, bool Counted, int Samples, long TakenTicks, bool FromPresents = false)
    {
        public double RefreshHz => Stopwatch.Frequency / PeriodTicks;

        // The first vblank at or after t, in Stopwatch ticks.
        public long NextVblank(long t)
        {
            double n = Math.Ceiling((t - VblankTicks) / PeriodTicks);
            return VblankTicks + (long)Math.Round(n * PeriodTicks);
        }

        // The vblank nearest t.
        public long NearestVblank(long t)
        {
            double n = Math.Round((t - VblankTicks) / PeriodTicks);
            return VblankTicks + (long)Math.Round(n * PeriodTicks);
        }
    }

    // Fed by the render thread, read by the emulation thread through one immutable reading.
    public sealed class DisplayClock
    {
        public const int Window = 240;
        public const int MinimumSamples = 30;

        // A lattice whose points sit within this share of a period of the fitted line is a fixed refresh.
        public const double FixedResidual = 0.12;
        public const double FixedShare = 0.9;

        private readonly long[] _t = new long[Window];
        private readonly long[] _n = new long[Window];
        private int _count, _head;
        private long _lastCount = long.MinValue;
        private bool _counted;
        private long _samples, _lastGap = long.MinValue / 2;
        private readonly long[] _inferred = new long[Window];
        private readonly long[] _deltas = new long[Window];
        private DisplayReading? _reading;

        public DisplayReading? Reading => System.Threading.Volatile.Read(ref _reading);

        // A reading no newer than this is treated as no reading.
        public static readonly long StaleTicks = Stopwatch.Frequency * 5;

        public DisplayReading? Current(long now)
        {
            DisplayReading? r = Reading;
            return r is not null && now - r.TakenTicks <= StaleTicks ? r : null;
        }

        // A vblank with its hardware count: UST in microseconds of CLOCK_MONOTONIC, as GLX_OML_sync_control gives it.
        public void ObserveVblank(long ustMicroseconds, long msc, long now)
        {
            long t = ustMicroseconds * (Stopwatch.Frequency / 1_000_000);
            if (Math.Abs(now - t) > Stopwatch.Frequency) return; // a UST from another clock domain is no use
            if (!_counted) Reset();
            _counted = true;
            if (msc == _lastCount) return;
            if (msc < _lastCount) Reset();
            if (_lastCount != long.MinValue && msc - _lastCount > 1) _lastGap = _samples;
            _lastCount = msc;
            _samples++;
            Add(t, msc, now);
        }

        // A presenter tick with no count, as a platform whose render timer is the vblank gives it.
        public void ObserveTick(long t)
        {
            if (_counted) return;
            long n;
            if (_count == 0) n = 0;
            else
            {
                long prevT = _t[(_head - 1 + Window) % Window], prevN = _n[(_head - 1 + Window) % Window];
                double period = _reading?.PeriodTicks ?? SeedInterval(t - prevT);
                n = prevN + Math.Max(1, (long)Math.Round((t - prevT) / period));
            }
            Add(t, n, t);
        }

        public void Reset()
        {
            _count = _head = 0;
            _lastCount = long.MinValue;
            _samples = 0;
            _lastGap = long.MinValue / 2;
            _counted = false;
            System.Threading.Volatile.Write(ref _reading, null);
        }

        private double _seedSum;
        private int _seedCount;

        private double SeedInterval(long dt)
        {
            if (_seedCount < 8) { _seedSum += dt; _seedCount++; }
            return _seedSum / _seedCount;
        }

        private void Add(long t, long n, long now)
        {
            _t[_head] = t; _n[_head] = n;
            _head = (_head + 1) % Window;
            if (_count < Window) _count++;
            if (_count < MinimumSamples) return;
            // A count that has risen by exactly one at every draw for a whole window counts presents, not refreshes; the refreshes are then read from the times - see §4.87.11.
            bool presents = _counted && _samples - _lastGap >= Window;
            if (!presents) { System.Threading.Volatile.Write(ref _reading, Fit(_t, _n, _count, _head, _counted, now)); return; }
            InferCounts(_t, _count, _head, _inferred, _deltas);
            DisplayReading? fit = Fit(_t, _inferred, _count, _head, _counted, now);
            System.Threading.Volatile.Write(ref _reading, fit is null ? null : fit with { FromPresents = true });
        }

        // Counts for present times that sit on a refresh lattice: the base is the shortest interval that recurs, and each interval is a whole number of it.
        internal static void InferCounts(long[] times, int count, int head, long[] counts, long[] deltas)
        {
            int first = (head - count + times.Length) % times.Length;
            int m = count - 1;
            for (int i = 0; i < m; i++) deltas[i] = times[(first + i + 1) % times.Length] - times[(first + i) % times.Length];
            double baseTicks = BaseInterval(deltas, m);
            long n = 0;
            counts[first] = 0;
            for (int i = 0; i < m; i++)
            {
                n += Math.Max(1, (long)Math.Round(deltas[i] / baseTicks));
                counts[(first + i + 1) % times.Length] = n;
            }
        }

        // The median interval over the smallest whole divisor that puts every interval, within the residual, on a multiple of it: two refreshes and the odd one or three is the refresh.
        internal static double BaseInterval(long[] deltas, int m)
        {
            long[] sorted = new long[m];
            Array.Copy(deltas, sorted, m);
            Array.Sort(sorted);
            double median = Math.Max(1, sorted[m / 2]);
            double[] share = new double[5];
            for (int divisor = 1; divisor <= 4; divisor++)
            {
                double b = median / divisor;
                int on = 0;
                for (int i = 0; i < m; i++)
                {
                    double steps = sorted[i] / b;
                    if (steps >= 0.5 && Math.Abs(steps - Math.Round(steps)) <= FixedResidual) on++;
                }
                share[divisor] = on / (double)m;
            }
            double best = Math.Max(Math.Max(share[1], share[2]), Math.Max(share[3], share[4]));
            for (int divisor = 1; divisor <= 4; divisor++)
                if (share[divisor] >= best) return median / divisor;
            return median;
        }

        // Least squares of time on count over the window, and how many samples fall near the line - see §4.87.3.
        public static DisplayReading? Fit(long[] times, long[] counts, int count, int head, bool counted, long now)
        {
            int first = (head - count + times.Length) % times.Length;
            long t0 = times[first], n0 = counts[first];
            double sn = 0, st = 0, snn = 0, snt = 0;
            for (int i = 0; i < count; i++)
            {
                int j = (first + i) % times.Length;
                double x = counts[j] - n0, y = times[j] - t0;
                sn += x; st += y; snn += x * x; snt += x * y;
            }
            double denominator = count * snn - sn * sn;
            if (denominator <= 0) return null;
            double period = (count * snt - sn * st) / denominator;
            if (period <= 0) return null;
            double intercept = (st - period * sn) / count;
            int near = 0;
            for (int i = 0; i < count; i++)
            {
                int j = (first + i) % times.Length;
                double residual = (times[j] - t0) - (intercept + period * (counts[j] - n0));
                if (Math.Abs(residual) <= FixedResidual * period) near++;
            }
            int last = (head - 1 + times.Length) % times.Length;
            long vblank = t0 + (long)Math.Round(intercept + period * (counts[last] - n0));
            return new DisplayReading(period, vblank, near < FixedShare * count, counted, count, now);
        }
    }
}
