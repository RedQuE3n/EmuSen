namespace EmuSen.WiseMan.Fixtures.Snes
{
    // One frame's comparison of two engines: bytes differing per space, pixels differing over the shared picture.
    public sealed record SnesFrameDiff(int Frame, IReadOnlyDictionary<string, int> SpaceBytes, int PixelsCompared, int PixelsDiffering)
    {
        public bool Identical => SpaceBytes.Values.All(n => n == 0) && PixelsDiffering == 0;
    }

    // The sound's agreement: the loudness envelope's best correlation within a lag, and whether each side is silent.
    public sealed record SnesAudioDiff(double Correlation, int LagWindows, bool FirstSilent, bool SecondSilent);

    // The waveforms' agreement at one rate, second by second: the median and least of Pearson's r at each second's best sample lag, the lags' range, and the median least-squares gain and residual RMS over the first's.
    public sealed record SnesSampleDiff(double Correlation, double Least, int LagFrom, int LagTo, double Gain, double RelativeError, int Seconds);

    // Frame-by-frame differential of two runs - see VenusRT_Native.md §3.4 for what each comparison can and cannot say.
    public static class SnesDifferential
    {
        // A 224-line engine's row r is Mesen's row r + 7, measured on steady pictures - see VenusRT_Native.md §3.2.
        public const int MesenRowOffset = 7;

        public static IReadOnlyList<SnesFrameDiff> Compare(SnesRun a, SnesRun b, int secondRowOffset = 0)
        {
            var diffs = new List<SnesFrameDiff>();
            foreach (var x in a.Snapshots)
            {
                var y = b.At(x.Frame);
                if (y is null) continue;
                var bytes = new Dictionary<string, int>();
                foreach (var (name, left) in x.Spaces)
                {
                    if (!y.Spaces.TryGetValue(name, out byte[]? right)) continue;
                    int n = Math.Abs(left.Length - right.Length);
                    for (int i = 0; i < Math.Min(left.Length, right.Length); i++) if (left[i] != right[i]) n++;
                    bytes[name] = n;
                }
                var (compared, differing) = Pictures(x.Picture, y.Picture, secondRowOffset);
                diffs.Add(new SnesFrameDiff(x.Frame, bytes, compared, differing));
            }
            return diffs;
        }

        // The rectangle both pictures cover, the second shifted down by its offset; the top bit of a colour word is not compared.
        public static (int Compared, int Differing) Pictures(SnesPicture? p, SnesPicture? q, int rowOffset)
        {
            if (p is null || q is null) return (0, 0);
            // Pictures of different heights and widths are compared at the larger layout - see VenusRT_Native.md §19.2.
            int py = p.Height > 300 ? 2 : 1, qy = q.Height > 300 ? 2 : 1;
            // Two such pictures are compared row for row, the offset doubled: an interlaced picture's fields differ.
            if (py == 2 && qy == 2) (py, qy, rowOffset) = (1, 1, rowOffset * 2);
            int w = Math.Max(p.Width, q.Width), h = Math.Min(p.Height / py, q.Height / qy - rowOffset);
            int differing = 0;
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    int px = p.Width == w ? x : x * p.Width / w, qx = q.Width == w ? x : x * q.Width / w;
                    if (((p.Pixels[y * py * p.Width + px] ^ q.Pixels[(y + rowOffset) * qy * q.Width + qx]) & 0x7FFF) != 0) differing++;
                }
            return (Math.Max(0, w * h), differing);
        }

        // Each run's loudness in windows of a sixtieth of a second, correlated at the best lag within maxLag windows.
        public static SnesAudioDiff Audio(SnesRun a, SnesRun b, int maxLag = 30)
        {
            double[] ea = Envelope(a), eb = Envelope(b);
            bool sa = ea.All(v => v < 1), sb = eb.All(v => v < 1);
            double best = double.NaN;
            int bestLag = 0;
            for (int lag = -maxLag; lag <= maxLag; lag++)
            {
                double r = Pearson(ea, eb, lag);
                if (double.IsNaN(best) || r > best) { best = r; bestLag = lag; }
            }
            return new SnesAudioDiff(best, bestLag, sa, sb);
        }

        // Both runs as mono at `rate` from frame `skipFrames` on, each second compared at its own best lag within maxLag samples, silent seconds left out; see VenusRT_Native.md §23.
        public static SnesSampleDiff Samples(SnesRun a, SnesRun b, int rate = 32000, int maxLag = 1600, int skipFrames = 60)
        {
            double[] x = Resample(a, rate), y = Resample(b, rate);
            var r = new List<double>();
            var lags = new List<int>();
            var gains = new List<double>();
            var errors = new List<double>();
            for (int from = Math.Max(skipFrames * rate / 60, maxLag); from + rate + maxLag <= Math.Min(x.Length, y.Length); from += rate)
            {
                double best = double.NaN;
                int bestLag = 0;
                for (int lag = -maxLag; lag <= maxLag; lag++)
                {
                    double sxy = 0, sxx = 0, syy = 0;
                    for (int i = from; i < from + rate; i++) { double u = x[i], v = y[i + lag]; sxy += u * v; sxx += u * u; syy += v * v; }
                    double c = sxx == 0 || syy == 0 ? double.NaN : sxy / Math.Sqrt(sxx * syy);
                    if (!double.IsNaN(c) && (double.IsNaN(best) || Math.Abs(c) > Math.Abs(best))) { best = c; bestLag = lag; }
                }
                if (double.IsNaN(best)) continue;
                double xy = 0, yy = 0, xx = 0;
                for (int i = from; i < from + rate; i++) { xy += x[i] * y[i + bestLag]; yy += y[i + bestLag] * y[i + bestLag]; xx += x[i] * x[i]; }
                double gain = xy / yy, err = 0;
                for (int i = from; i < from + rate; i++) { double e = x[i] - gain * y[i + bestLag]; err += e * e; }
                r.Add(best); lags.Add(bestLag); gains.Add(gain); errors.Add(Math.Sqrt(err / xx));
            }
            static double Median(List<double> v) => v.Count == 0 ? double.NaN : v.Order().ElementAt(v.Count / 2);
            return r.Count == 0 ? new SnesSampleDiff(double.NaN, double.NaN, 0, 0, double.NaN, double.NaN, 0)
                : new SnesSampleDiff(Median(r), r.Min(), lags.Min(), lags.Max(), Median(gains), Median(errors), r.Count);
        }

        // A run's sound as mono (the mean of its two sides) at `rate`, by linear interpolation between its own samples.
        public static double[] Resample(SnesRun run, int rate)
        {
            if (run.AudioRate <= 0) return Array.Empty<double>();
            int frames = run.Audio.Length / 2;
            var mono = new double[frames];
            for (int i = 0; i < frames; i++) mono[i] = (run.Audio[i * 2] + run.Audio[i * 2 + 1]) / 2.0;
            if (run.AudioRate == rate) return mono;
            int length = (int)((long)frames * rate / run.AudioRate);
            var o = new double[length];
            for (int j = 0; j < length; j++)
            {
                double t = (double)j * run.AudioRate / rate;
                int k = (int)t;
                double f = t - k;
                o[j] = k + 1 < frames ? mono[k] * (1 - f) + mono[k + 1] * f : mono[Math.Min(k, frames - 1)];
            }
            return o;
        }

        public static double[] Envelope(SnesRun run)
        {
            if (run.AudioRate <= 0) return Array.Empty<double>();
            int window = Math.Max(1, run.AudioRate / 60);
            int frames = run.Audio.Length / 2;
            var e = new double[frames / window];
            for (int w = 0; w < e.Length; w++)
            {
                double sum = 0;
                for (int i = w * window; i < (w + 1) * window; i++)
                {
                    double m = (run.Audio[i * 2] + run.Audio[i * 2 + 1]) / 2.0;
                    sum += m * m;
                }
                e[w] = Math.Sqrt(sum / window);
            }
            return e;
        }

        private static double Pearson(double[] x, double[] y, int lag)
        {
            int from = Math.Max(0, -lag), to = Math.Min(x.Length, y.Length - lag);
            int n = to - from;
            if (n < 2) return double.NaN;
            double mx = 0, my = 0;
            for (int i = from; i < to; i++) { mx += x[i]; my += y[i + lag]; }
            mx /= n; my /= n;
            double sxy = 0, sxx = 0, syy = 0;
            for (int i = from; i < to; i++)
            {
                double dx = x[i] - mx, dy = y[i + lag] - my;
                sxy += dx * dy; sxx += dx * dx; syy += dy * dy;
            }
            return sxx == 0 || syy == 0 ? double.NaN : sxy / Math.Sqrt(sxx * syy);
        }
    }
}
