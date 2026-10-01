namespace EmuSen.WiseMan.Fixtures.Snes
{
    // One frame's comparison of two engines: bytes differing per space, pixels differing over the shared picture.
    public sealed record SnesFrameDiff(int Frame, IReadOnlyDictionary<string, int> SpaceBytes, int PixelsCompared, int PixelsDiffering)
    {
        public bool Identical => SpaceBytes.Values.All(n => n == 0) && PixelsDiffering == 0;
    }

    // The sound's agreement: the loudness envelope's best correlation within a lag, and whether each side is silent.
    public sealed record SnesAudioDiff(double Correlation, int LagWindows, bool FirstSilent, bool SecondSilent);

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
            // A picture more than 300 rows high shows each line twice (or interlaced); one 512 wide against one 256 wide is
            // compared at the wider one's columns, the narrow pixel standing for both halves, the main screen's being the odd one.
            int py = p.Height > 300 ? 2 : 1, qy = q.Height > 300 ? 2 : 1;
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
