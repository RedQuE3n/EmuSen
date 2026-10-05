namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    public enum RomOutcome { Passed, Failed, Incomplete, Done, Visual, NoDump }

    public sealed record RomVerdict(RomOutcome Outcome, string Protocol, string Detail);

    // One way a test ROM reports: whether it applies to a path, and its verdict on a run.
    public sealed record RomProtocol(string Name, Func<string, bool> Applies, Func<RomRun, RomVerdict> Grade);

    // A corpus's protocols, the first that applies grading; a ROM none claims is graded by picture against a reference - see Nephrite_Native.md §3.3.
    public sealed class TestRomGrader(IReadOnlyList<RomProtocol> protocols)
    {
        public IReadOnlyList<RomProtocol> Protocols { get; } = protocols;

        public RomVerdict Grade(string path, RomRun run)
        {
            if (run.Snapshots.Count == 0) return new RomVerdict(RomOutcome.NoDump, "none", "no snapshot");
            var p = Protocols.FirstOrDefault(p => p.Applies(path));
            return p is null ? new RomVerdict(RomOutcome.Visual, "none", "") : p.Grade(run);
        }
    }

    public sealed record RomFrameDiff(int Frame, IReadOnlyDictionary<string, int> SpaceBytes, int PixelsCompared, int PixelsDiffering, int PixelsOffMap);

    // Two runs compared frame by frame: bytes per space, pixels exactly, and pixels up to a one-to-one colour map, since each engine has its own colour levels - see Nephrite_Native.md §3.4.
    public static class TestRomDifferential
    {
        public static IReadOnlyList<RomFrameDiff> Compare(RomRun a, RomRun b, IReadOnlyDictionary<string, string>? spaceNames = null)
        {
            var list = new List<RomFrameDiff>();
            foreach (var sa in a.Snapshots)
            {
                if (b.At(sa.Frame) is not { } sb) continue;
                var bytes = new Dictionary<string, int>();
                foreach (var (name, data) in sa.Spaces)
                {
                    string other = spaceNames?.GetValueOrDefault(name) ?? name;
                    if (sb.Spaces.TryGetValue(other, out var d)) bytes[name] = Bytes(data, d);
                }
                var (compared, differing) = sa.Picture is { } pa && sb.Picture is { } pb ? Pictures(pa, pb) : (0, 0);
                int offMap = sa.Picture is { } qa && sb.Picture is { } qb ? OffColourMap(qa, qb) : 0;
                list.Add(new RomFrameDiff(sa.Frame, bytes, compared, differing, offMap));
            }
            return list;
        }

        // Differing bytes over the shorter, plus the longer's excess.
        public static int Bytes(byte[] a, byte[] b) => Enumerable.Range(0, Math.Min(a.Length, b.Length)).Count(i => a[i] != b[i]) + Math.Abs(a.Length - b.Length);

        // Over the rectangle both pictures cover.
        public static (int Compared, int Differing) Pictures(RomPicture a, RomPicture b)
        {
            int w = Math.Min(a.Width, b.Width), h = Math.Min(a.Height, b.Height), n = 0;
            for (int y = 0; y < h; y++) for (int x = 0; x < w; x++) if (a.At(x, y) != b.At(x, y)) n++;
            return (w * h, n);
        }

        // Pixels that break a one-to-one map between the two pictures' colours: zero when they differ only in colour levels.
        public static int OffColourMap(RomPicture a, RomPicture b)
        {
            var forward = new Dictionary<(byte, byte, byte), (byte, byte, byte)>();
            var backward = new Dictionary<(byte, byte, byte), (byte, byte, byte)>();
            int w = Math.Min(a.Width, b.Width), h = Math.Min(a.Height, b.Height), n = 0;
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    var (ca, cb) = (a.At(x, y), b.At(x, y));
                    if (forward.TryGetValue(ca, out var f) ? f != cb : backward.TryGetValue(cb, out var r) && r != ca) n++;
                    else
                    {
                        forward[ca] = cb;
                        backward[cb] = ca;
                    }
                }
            return n;
        }

        // Whether the picture stood still between two of a run's frames, which makes it comparable across engines.
        public static bool Steady(RomRun run, int first, int second) =>
            run.At(first)?.Picture is { } p && run.At(second)?.Picture is { } q && p.Width == q.Width && p.Height == q.Height && p.Rgb.AsSpan().SequenceEqual(q.Rgb);
    }
}
