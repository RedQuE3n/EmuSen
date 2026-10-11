using SkiaSharp;

namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // Pixels a named disagreement allows to differ: Count pixels from (X, Y) stepping (Dx, Dy) - see Diamond_GPU.md §9.
    public sealed record VramExclusion(int X, int Y, int Count, int Dx, int Dy, string Why);

    // A program graded by its VRAM at a frame: the reference beside it, the area graded (all of VRAM if none) and its exclusions.
    public sealed record VramProgram(string Path, int Frame, (int X, int Y, int W, int H)? Area = null, VramExclusion[]? Excluded = null, string Picture = "vram.png");

    // A run's VRAM against a reference picture exactly, as ps1-tests' diffvram compares them - see Diamond_GPU.md §9.
    public static class VramGrader
    {
        public const string Space = "vram";
        public const int Width = 1024, Height = 512;

        // A halfword as the reference pictures hold it: each five-bit channel shifted up by three, the mask bit not shown.
        public static (byte R, byte G, byte B) Rgb(byte[] vram, int x, int y)
        {
            int at = (y * Width + x) * 2;
            int v = vram[at] | vram[at + 1] << 8;
            return ((byte)((v & 31) << 3), (byte)((v >> 5 & 31) << 3), (byte)((v >> 10 & 31) << 3));
        }

        public static (int Compared, List<(int X, int Y)> Differing) Compare(byte[] vram, SKBitmap reference, VramProgram p)
        {
            var (ax, ay, aw, ah) = p.Area ?? (0, 0, Width, Height);
            var excluded = new HashSet<(int, int)>((p.Excluded ?? Array.Empty<VramExclusion>()).SelectMany(e => Enumerable.Range(0, e.Count).Select(i => (e.X + i * e.Dx, e.Y + i * e.Dy))));
            var differing = new List<(int, int)>();
            int compared = 0;
            for (int y = ay; y < ay + ah; y++)
                for (int x = ax; x < ax + aw; x++)
                {
                    if (excluded.Contains((x, y))) continue;
                    compared++;
                    var c = reference.GetPixel(x, y);
                    if (Rgb(vram, x, y) != (c.Red, c.Green, c.Blue)) differing.Add((x, y));
                }
            return (compared, differing);
        }

        public static RomProtocol Protocol(VramProgram p, Func<string, string?> picture) =>
            new($"vram:{p.Path}", path => path.Replace('\\', '/').EndsWith(p.Path, StringComparison.Ordinal), run => Grade(p, run, picture));

        public static RomVerdict Grade(VramProgram p, RomRun run, Func<string, string?> picture)
        {
            string name = $"vram:{p.Path}";
            if (run.At(p.Frame) is not { } shot || !shot.Spaces.TryGetValue(Space, out var vram)) return new RomVerdict(RomOutcome.NoDump, name, "no VRAM space");
            if (picture(p.Path) is not { } file) return new RomVerdict(RomOutcome.Incomplete, name, "no reference picture");
            using var reference = SKBitmap.Decode(file);
            if (reference is null || reference.Width != Width || reference.Height != Height) return new RomVerdict(RomOutcome.Incomplete, name, "the reference is not 1024x512");
            var (compared, differing) = Compare(vram, reference, p);
            string first = string.Join(" ", differing.Take(4).Select(d => $"({d.X},{d.Y})"));
            return differing.Count == 0
                ? new RomVerdict(RomOutcome.Passed, name, $"{compared} pixels exact")
                : new RomVerdict(RomOutcome.Failed, name, $"{differing.Count} of {compared} pixels differ, first {first}");
        }
    }

    // ps1-tests' GPU programs with a VRAM picture: the frame each is read at, and the disagreements named in Diamond_GPU.md §9.
    public static class Ps1GpuTests
    {
        public static readonly VramProgram[] Programs =
        {
            new("gpu/clipping/clipping.exe", 300),
            new("gpu/clut-cache/clut-cache.exe", 300),
            // Its polylines' fourth colour is never set by the program, so the segments to it shade from whatever the stack held.
            new("gpu/lines/lines.exe", 300, Excluded: new[]
            {
                new VramExclusion(150, 140, 32, 1, 1, "gouraudMultiLine's opaque segment to its uninitialised fourth colour"),
                new VramExclusion(210, 140, 32, 1, 1, "gouraudMultiLine's semi-transparent segment to its uninitialised fourth colour"),
            }),
            new("gpu/quad/quad.exe", 300),
            new("gpu/rectangles/rectangles.exe", 300),
            new("gpu/texture-flip/texture-flip.exe", 300),
            new("gpu/texture-overflow/texture-overflow.exe", 300),
            // The picture predates the release's program, which clears VRAM black rather than white; only its screen is drawn.
            new("gpu/transparency/transparency.exe", 300, Area: (0, 0, 320, 240)),
            new("gpu/triangle/triangle.exe", 300),
            new("gpu/uv-interpolation/uv-interpolation.exe", 300),
            new("gpu/vram-to-vram-overlap/vram-to-vram-overlap.exe", 300),
        };
    }

    // ps1-tests' MDEC programs with a VRAM picture, the frame each is read at, and the pixels D-30 leaves differing - see Diamond_MDEC.md §7.
    public static class Ps1MdecTests
    {
        public static readonly (VramProgram Program, int Differing)[] Programs =
        {
            (new("mdec/4bit/4bit.exe", 300), 0),
            (new("mdec/8bit/8bit.exe", 300, Excluded: new[] { new VramExclusion(35, 35, 1, 1, 0, "D-30: the IDCT's rounding gives 240 where the console's block has 239") }), 0),
            // The CPU builds of mdec/frame wait after their last word as the step-by-step log's console would, so only the DMA builds are graded.
            (new("mdec/frame/frame-15bit-dma.exe", 300, Picture: "vram-15bit.png"), 11035),
            (new("mdec/frame/frame-24bit-dma.exe", 300, Picture: "vram-24bit.png"), 63378),
            // The pictures hold the movie's frames 64 and 62, each in VRAM for one frame of the run.
            (new("mdec/movie/movie-15bit.exe", 315, Picture: "vram-15bit.png"), 319),
            (new("mdec/movie/movie-24bit.exe", 309, Picture: "vram-24bit.png"), 752),
        };
    }
}
