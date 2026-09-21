using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rdp;
using EmuSen.Cores.Nintendo.Mars.Rdp.Gpu;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The device's picture at a multiple against the CPU's at the same multiple, byte for byte - see Mars_Gpu.md §5 and Mars_GpuPlan.md §4.
    public class MarsGpuRasteriserTests
    {
        private const uint Framebuffer = 0x0020_0000, Second = 0x0030_0000;
        private const int Width = 320, Rows = 240;
        private const ulong FillCycle = (0x2FUL << 56) | (3UL << 52);

        private readonly ITestOutputHelper _output;

        public MarsGpuRasteriserTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<string, int> DevicesAndScales()
        {
            var data = new TheoryData<string, int>();
            foreach (string name in EmuSen.WiseMan.Fixtures.GpuTestDevices.Names)
                foreach (int scale in new[] { 2, 3, 4 }) data.Add(name, scale);
            return data;
        }

        private static ulong ColorImage(uint address, int size, int width = Width) =>
            (0x3FUL << 56) | ((ulong)size << 51) | ((ulong)(width - 1) << 32) | address;

        private static ulong FillColor(uint color) => (0x37UL << 56) | color;

        // Quarter pixels here, where the threaded tests' builders take whole ones, so that edges fall inside pixels.
        private static ulong Scissor(int left, int top, int right, int bottom) =>
            (0x2DUL << 56) | ((ulong)left << 44) | ((ulong)top << 32) | ((ulong)right << 12) | (uint)bottom;

        private static ulong FillRectangle(int left, int top, int right, int bottom) =>
            (0x36UL << 56) | ((ulong)right << 44) | ((ulong)bottom << 32) | ((ulong)left << 12) | (uint)top;

        // Overlapping rectangles in colours whose halves and low bits differ, into a sixteen-bit image and then a thirty-two-bit one.
        private static ulong[] Fills(int seed, int scissorRight = Width * 4)
        {
            var random = new Random(seed);
            var list = new List<ulong> { FillCycle };

            // The whole image first, to its last column and row, which a clip one short would miss; the scissors after it fall inside pixels.
            list.Add(ColorImage(Framebuffer, 2));
            list.Add(Scissor(0, 0, scissorRight, Rows * 4));
            list.Add(FillColor(0x1357_2468));
            list.Add(FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4));

            foreach ((uint image, int size) in new[] { (Second, 3), (Framebuffer, 2), (Second, 3) })
            {
                list.Add(ColorImage(image, size));
                list.Add(Scissor(random.Next(0, 40), random.Next(0, 40), scissorRight - random.Next(0, 40), Rows * 4 - random.Next(0, 40)));

                for (int i = 0; i < 60; i++)
                {
                    int left = random.Next(0, Width * 4), top = random.Next(0, Rows * 4);
                    list.Add(FillColor((uint)random.Next() * 2u + (uint)random.Next(2)));
                    list.Add(FillRectangle(left, top, Math.Min(left + random.Next(0, 600), 0xFFF), Math.Min(top + random.Next(0, 400), 0xFFF)));
                }
            }

            return list.ToArray();
        }

        private const uint DepthBuffer = 0x0028_0000;
        private const uint TextureSource = 0x0004_0000;

        // The same bytes under both processors, since a load reads the machine's own memory at either multiple.
        private static void SeedTextureSource(MemoryBus bus)
        {
            uint s = 0x00C0_FFEE;
            for (uint at = 0; at < 0x4000; at += 4) bus.Write32(TextureSource + at, (Next(ref s) << 8) | (Next(ref s) & 0xFF));
        }

        private static uint Next(ref uint state)
        {
            state = state * 1664525u + 1013904223u;
            return state >> 8;
        }

        private static T Pick<T>(ref uint state, params T[] from) => from[Next(ref state) % (uint)from.Length];

        private static ulong Edge(double x, double slope) =>
            ((ulong)(uint)(int)Math.Round(x * 65536) << 32) | (uint)(int)Math.Round(Math.Clamp(slope, -8192, 8191) * 65536);

        // A shaded, depth-tested triangle from three corners, as MarsThreadedRdpTests builds them; its shade and depth words are a seed's.
        private static ulong[] Triangle(double x1, double y1, double x2, double y2, double x3, double y3, ref uint s)
        {
            const int id = 0x0D;
            var sorted = new[] { (X: x1, Y: y1), (X: x2, Y: y2), (X: x3, Y: y3) }.OrderBy(v => v.Y).ToArray();
            var (top, middle, bottom) = (sorted[0], sorted[1], sorted[2]);

            double major = bottom.Y > top.Y ? (bottom.X - top.X) / (bottom.Y - top.Y) : 0;
            double upper = middle.Y > top.Y ? (middle.X - top.X) / (middle.Y - top.Y) : 0;
            double lower = bottom.Y > middle.Y ? (bottom.X - middle.X) / (bottom.Y - middle.Y) : 0;

            int yh = (int)Math.Floor(top.Y * 4), ym = (int)Math.Floor(middle.Y * 4), yl = (int)Math.Floor(bottom.Y * 4);
            double rowTop = Math.Floor(top.Y);
            double xh = top.X + major * (rowTop - top.Y), xm = top.X + upper * (rowTop - top.Y), xl = middle.X + lower * (ym / 4.0 - middle.Y);
            bool majorOnLeft = middle.X > top.X + major * (middle.Y - top.Y);

            var words = new ulong[Rdp.Length(id)];
            words[0] = ((ulong)id << 56) | (majorOnLeft ? 1UL << 55 : 0) | ((ulong)(uint)(yl & 0x3FFF) << 32) | ((ulong)(uint)(ym & 0x3FFF) << 16) | (uint)(yh & 0x3FFF);
            words[1] = Edge(xl, lower);
            words[2] = Edge(xh, major);
            words[3] = Edge(xm, upper);

            for (int i = 4; i < 12; i++) words[i] = ((ulong)(Next(ref s) & 0x007F_FFFF) << 32) | (Next(ref s) & 0x0003_FFFF);
            words[12] = ((ulong)(0x1000 + (Next(ref s) & 0xFFFF)) << 48) | ((ulong)(Next(ref s) & 0xFFFF) << 32) | ((ulong)(Next(ref s) & 0x3FF) << 16) | (Next(ref s) & 0xFFFF);
            words[13] = ((ulong)(Next(ref s) & 0x3FF) << 48) | ((ulong)(Next(ref s) & 0xFFFF) << 32) | (Next(ref s) & 0x03FF_FFFF);
            return words;
        }

        // Both cycles alike, from the inputs the device has: no previous pixel's result, no texel, no level of detail - see Mars_Gpu.md §6.
        private static ulong Combine(ref uint s)
        {
            int a = Pick(ref s, 3, 4, 5, 6, 7, 15), b = Pick(ref s, 3, 4, 5, 6, 7, 15), c = Pick(ref s, 3, 4, 5, 6, 10, 11, 12, 14, 15, 31), d = Pick(ref s, 3, 4, 5, 6, 7);
            int alphaA = Pick(ref s, 3, 4, 5, 6, 7), alphaB = Pick(ref s, 3, 4, 5, 6, 7), alphaC = Pick(ref s, 3, 4, 5, 6, 7), alphaD = Pick(ref s, 3, 4, 5, 6, 7);

            ulong high = (ulong)((a << 20) | (c << 15) | (alphaA << 12) | (alphaC << 9) | (a << 5) | c);
            ulong low = (ulong)(uint)((b << 28) | (b << 24) | (alphaA << 21) | (alphaC << 18) | (d << 15) | (alphaB << 12) | (alphaD << 9) | (d << 6) | (alphaB << 3) | alphaD);
            return (0x3CUL << 56) | (high << 32) | low;
        }

        private static ulong KeyedCombine(int c)
        {
            const int a = 4, b = 15, d = 7, alpha = 6;
            ulong high = (ulong)((a << 20) | (c << 15) | (alpha << 12) | (alpha << 9) | (a << 5) | c);
            ulong low = unchecked((uint)((b << 28) | (b << 24) | (alpha << 21) | (alpha << 18) | (d << 15) | (alpha << 12) | (alpha << 9) | (d << 6) | (alpha << 3) | alpha));
            return (0x3CUL << 56) | (high << 32) | low;
        }

        // Two cycles with selectors of their own. The first must not read the combiner's own previous result, which is
        // the pixel before it; the second's is this pixel's first cycle, so it may and should, and a scene that never
        // does leaves the whole handover between the cycles untested.
        private static ulong TwoCycleCombine(ref uint s)
        {
            int a1 = Pick(ref s, 1, 2, 3, 4, 5, 6, 7), b1 = Pick(ref s, 1, 2, 3, 4, 5, 7);
            int c1 = Pick(ref s, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 31), d1 = Pick(ref s, 1, 2, 3, 4, 5, 6, 7);
            int aa1 = Pick(ref s, 1, 2, 3, 4, 5, 6, 7), ab1 = Pick(ref s, 1, 2, 3, 4, 5, 7);
            int ac1 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7), ad1 = Pick(ref s, 1, 2, 3, 4, 5, 6, 7);

            int a2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7), b2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 7);
            int c2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 31), d2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7);
            int aa2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7), ab2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 7);
            int ac2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7), ad2 = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7);

            ulong high = (ulong)((a1 << 20) | (c1 << 15) | (aa1 << 12) | (ac1 << 9) | (a2 << 5) | c2);
            ulong low = unchecked((uint)((b1 << 28) | (b2 << 24) | (aa2 << 21) | (ac2 << 18) | (d1 << 15)
                | (ab1 << 12) | (ad1 << 9) | (d2 << 6) | (ab2 << 3) | ad2));
            return (0x3CUL << 56) | (high << 32) | low;
        }

        // The one-cycle mode with everything below the cycle type a seed's: dithers, key, the first blend's four selectors, and the sixteen low bits whole.
        private static ulong OneCycleModes(ref uint s, bool twoCycle = false)
        {
            ulong modes = (0x2FUL << 56) | ((ulong)(Next(ref s) & 3) << 38) | ((ulong)(Next(ref s) & 3) << 36);
            if (Next(ref s) % 3 == 0) modes |= 1UL << 40;

            // The first blend cycle's second alpha must not be memory alpha: that weighs it by the previous pixel's
            // stored depth slope, which is a carry the device declines. The second cycle's may be, and should be.
            ulong firstSecondAlpha = twoCycle ? (ulong)Pick(ref s, 0, 2, 3) : Next(ref s) & 3;
            modes |= (ulong)(Next(ref s) & 3) << 30 | (ulong)(Next(ref s) & 3) << 26 | (ulong)(Next(ref s) & 3) << 22 | firstSecondAlpha << 18;

            if (twoCycle)
                modes |= (1UL << 52) | ((ulong)(Next(ref s) & 3) << 28) | ((ulong)(Next(ref s) & 3) << 24)
                    | ((ulong)(Next(ref s) & 3) << 20) | ((ulong)(Next(ref s) & 3) << 16);

            return modes | (Next(ref s) & 0x7FFF);
        }

        // Depth cleared by filling it as a colour image, the picture cleared, then triangles over and past the image, each under modes, colours and a combiner of its own.
        private static ulong[] Shaded(uint seed, int size, bool keyed = false, bool twoCycle = false)
        {
            uint s = seed;
            var list = new List<ulong>
            {
                FillCycle,
                (0x3EUL << 56) | DepthBuffer,
                ColorImage(DepthBuffer, 2), Scissor(0, 0, Width * 4, Rows * 4), FillColor(0xFFFC_FFFC), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
                ColorImage(Framebuffer, size), FillColor(0x2109_8421), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
            };

            for (int i = 0; i < 72; i++)
            {
                // Keyed: shade times a scalar sweeps the key's narrow window, and a forced blend by the pixel's alpha carries the key's alpha into the picture - see Mars_Gpu.md §6.4.
                list.Add(keyed ? (0x2FUL << 56) | ((ulong)(Next(ref s) & 3) << 38) | (1UL << 40) | (1UL << 22) | (1UL << 14) : OneCycleModes(ref s, twoCycle));
                list.Add(keyed ? KeyedCombine(Pick(ref s, 10, 12, 15)) : twoCycle ? TwoCycleCombine(ref s) : Combine(ref s));
                foreach (int id in new[] { 0x38, 0x39, 0x3A, 0x3B, 0x2C, 0x2E })
                    list.Add(((ulong)id << 56) | ((ulong)(Next(ref s) & 0xFF_FFFF) << 32) | ((ulong)Next(ref s) << 8) | (Next(ref s) & 0xFF));

                // The key's widths narrow half the time: a wide key's distance saturates, and then nothing of the key is exercised.
                ulong widthMask = Next(ref s) % 2 == 0 ? 0x3UL : 0xFFFUL;
                list.Add((0x2AUL << 56) | ((Next(ref s) & widthMask) << 44) | ((Next(ref s) & widthMask) << 32) | ((ulong)Next(ref s) << 8) | (Next(ref s) & 0xFF));
                list.Add((0x2BUL << 56) | ((Next(ref s) & widthMask) << 16) | (Next(ref s) & 0xFFFF));

                double Corner(int extent) => (int)(Next(ref s) % (uint)(extent + 60)) - 30 + (Next(ref s) & 3) / 4.0;
                list.AddRange(Triangle(Corner(Width), Corner(Rows), Corner(Width), Corner(Rows), Corner(Width), Corner(Rows), ref s));
            }

            return list.ToArray();
        }

        private static ulong Pack(int[] v, int shift) =>
            ((ulong)(uint)((v[0] >> shift) & 0xFFFF) << 48) | ((ulong)(uint)((v[1] >> shift) & 0xFFFF) << 32)
            | ((ulong)(uint)((v[2] >> shift) & 0xFFFF) << 16) | (uint)((v[3] >> shift) & 0xFFFF);

        // A triangle carrying shade, texture and depth; its texture coordinates land inside a thirty-two texel tile and step gently.
        private static ulong[] TexturedTriangle(double x1, double y1, double x2, double y2, double x3, double y3, int mode, int tile, int index, int maxLevel, ref uint s)
        {
            ulong[] shaded = Triangle(x1, y1, x2, y2, x3, y3, ref s);
            var words = new ulong[Rdp.Length(0x0F)];
            Array.Copy(shaded, words, 12);
            words[0] = (words[0] & ~(0xFFUL << 56)) | (0x0FUL << 56) | ((ulong)tile << 48) | ((ulong)maxLevel << 51);
            words[20] = shaded[12];
            words[21] = shaded[13];

            int[] value = new int[4], dx = new int[4], de = new int[4], dy = new int[4];

            // Without perspective w is not read; with it, a w of a power of two and coordinates a few texels of that w
            // is what walks the divider's fifteen shifts without saturating, and a w at or below zero is its other edge.
            // Swept rather than drawn at random: with sixty primitives a list, a random pairing of a small w with a clean
            // coordinate range never came up, and that pairing is the only one the divider's own arithmetic shows through.
            int kind = mode == 0 ? -1 : mode == 1 ? 0 : 1 + (index % 3);

            if (kind <= 1)
            {
                int w = kind < 0 ? 1 : 1 << (index % 14);

                // The divider answers s over w in ten and five bits, so a result inside its range wants s below w, not a
                // multiple of it. A measured census of the shifts showed every coordinate saturating while s ran the other way.
                int start = w <= 1 ? 0 : (int)(Next(ref s) % (uint)(kind == 0 ? w : w * 8));
                int step = Math.Max(1, (w << 16) >> 8);

                for (int c = 0; c < 2; c++)
                {
                    value[c] = kind < 0
                        ? (int)((Next(ref s) % 32u) << 21) | (int)(Next(ref s) & 0x1F_FFFF)
                        : (start << 16) | (int)(Next(ref s) & 0xFFFF);
                    dx[c] = kind < 0 ? (int)(Next(ref s) & 0x3_FFFF) - 0x2_0000 : step - (int)(Next(ref s) % (uint)(2 * step));
                    de[c] = kind < 0 ? (int)(Next(ref s) & 0x3_FFFF) - 0x2_0000 : step - (int)(Next(ref s) % (uint)(2 * step));
                    dy[c] = kind < 0 ? (int)(Next(ref s) & 0x3_FFFF) - 0x2_0000 : step - (int)(Next(ref s) % (uint)(2 * step));
                }

                value[2] = kind < 0 ? 0 : (w << 16) | (int)(Next(ref s) & 0xFFFF);
                int wStep = Math.Max(1, (w << 16) >> 10);
                dx[2] = de[2] = dy[2] = kind < 0 ? 0 : wStep - (int)(Next(ref s) % (uint)(2 * wStep));
            }
            else
            {
                int low = (int)(Next(ref s) & 0xFFFF);
                for (int c = 0; c < 2; c++)
                {
                    value[c] = (int)((Next(ref s) % 32u) << 21) | (int)(Next(ref s) & 0x1F_FFFF);
                    dx[c] = (int)(Next(ref s) & 0x3_FFFF) - 0x2_0000;
                    de[c] = (int)(Next(ref s) & 0x3_FFFF) - 0x2_0000;
                    dy[c] = (int)(Next(ref s) & 0x3_FFFF) - 0x2_0000;
                }

                value[2] = kind == 2 ? (int)((Next(ref s) & 3u) << 16) | low : unchecked((int)0xFFFF_0000) | low;
                dx[2] = de[2] = dy[2] = (int)(Next(ref s) & 0x1FFF) - 0x1000;
            }

            (words[12], words[14]) = (Pack(value, 16), Pack(value, 0));
            (words[13], words[15]) = (Pack(dx, 16), Pack(dx, 0));
            (words[16], words[18]) = (Pack(de, 16), Pack(de, 0));
            (words[17], words[19]) = (Pack(dy, 16), Pack(dy, 0));
            return words;
        }

        private static ulong TexturedCombine(ref uint s)
        {
            int a = Pick(ref s, 1, 2, 3, 4, 5, 6, 7), b = Pick(ref s, 1, 2, 3, 4, 5, 7);
            int c = Pick(ref s, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 31), d = Pick(ref s, 1, 2, 3, 4, 5, 6, 7);
            int alphaA = Pick(ref s, 1, 2, 3, 4, 5, 6, 7), alphaB = Pick(ref s, 1, 2, 3, 4, 5, 7);
            int alphaC = Pick(ref s, 0, 1, 2, 3, 4, 5, 6, 7), alphaD = Pick(ref s, 1, 2, 3, 4, 5, 6, 7);

            ulong high = (ulong)((a << 20) | (c << 15) | (alphaA << 12) | (alphaC << 9) | (a << 5) | c);
            ulong low = unchecked((uint)((b << 28) | (b << 24) | (alphaA << 21) | (alphaC << 18) | (d << 15) | (alphaB << 12) | (alphaD << 9) | (d << 6) | (alphaB << 3) | alphaD));
            return (0x3CUL << 56) | (high << 32) | low;
        }

        // Every format and size a tile can name, over the same loaded bytes; the combinations the fetch treats alike are still all named.
        private static readonly (int Format, int Size)[] Formats =
            Enumerable.Range(0, 5).SelectMany(f => Enumerable.Range(0, 4).Select(z => (f, z))).ToArray();

        // A tile loaded from a sixteen-bit image and then read under a format and size of its own, and triangles over it.
        private static ulong[] Textured(uint seed, int mode, bool twoCycle = false)
        {
            uint s = seed;
            var list = new List<ulong>
            {
                FillCycle,
                (0x3EUL << 56) | DepthBuffer,
                ColorImage(DepthBuffer, 2), Scissor(0, 0, Width * 4, Rows * 4), FillColor(0xFFFC_FFFC), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
                ColorImage(Framebuffer, 2), FillColor(0x2109_8421), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
                (0x3DUL << 56) | (2UL << 51) | (63UL << 32) | TextureSource,
            };

            for (int i = 0; i < Formats.Length * 3; i++)
            {
                (int format, int size) = Formats[i % Formats.Length];
                int tile = (int)(Next(ref s) % 7u);
                int line = 4 + (int)(Next(ref s) & 7);
                int memory = (int)(Next(ref s) % 3u) * 64;
                int shiftS = (int)(Next(ref s) % 16u), shiftT = (int)(Next(ref s) % 16u);
                int maskS = (int)(Next(ref s) % 11u), maskT = (int)(Next(ref s) % 11u);
                uint wraps = Next(ref s);

                list.Add((0x35UL << 56) | ((ulong)format << 53) | ((ulong)size << 51) | ((ulong)line << 41) | ((ulong)memory << 32) | ((ulong)tile << 24)
                    | ((ulong)(Next(ref s) & 0xF) << 20) | ((wraps & 1) << 19) | ((wraps & 2) << 17) | ((ulong)maskT << 14) | ((ulong)shiftT << 10)
                    | ((wraps & 4) << 7) | ((wraps & 8) << 5) | ((ulong)maskS << 4) | (uint)shiftS);
                list.Add((0x34UL << 56) | ((ulong)tile << 24) | ((31UL << 2) << 12) | (31UL << 2));

                // A palette in the upper half of texture memory, loaded through a tile of its own, so that the indexed formats have one to read.
                list.Add((0x35UL << 56) | (2UL << 51) | (0x100UL << 32) | (7UL << 24));
                list.Add((0x30UL << 56) | (7UL << 24) | ((ulong)(255 << 2) << 12));

                // The clean list drops the depth and alpha tests, so that a divided coordinate reaches the picture at every pixel it covers rather than at a few.
                ulong low = mode == 1 ? Next(ref s) & 0x7FEE : Next(ref s) & 0x7FFF;
                ulong cycles = twoCycle
                    ? (1UL << 52) | ((ulong)(Next(ref s) & 3) << 28) | ((ulong)(Next(ref s) & 3) << 24) | ((ulong)(Next(ref s) & 3) << 20) | ((ulong)(Next(ref s) & 3) << 16)
                    : 0;
                list.Add(cycles | (0x2FUL << 56) | ((ulong)(Next(ref s) & 3) << 38) | ((ulong)(Next(ref s) & 3) << 36)
                    | (mode == 0 ? 0 : 1UL << 51) | ((ulong)(Next(ref s) & 1) << 43)
                    | ((ulong)(Next(ref s) & 1) << 45) | ((ulong)(Next(ref s) & 1) << 47) | ((ulong)(Next(ref s) & 1) << 46) | ((ulong)(Next(ref s) & 1) << 44)
                    | ((ulong)(Next(ref s) & 1) << 48) | ((ulong)(Next(ref s) & 1) << 49) | ((ulong)(Next(ref s) & 1) << 50)
                    | ((ulong)(Next(ref s) & 3) << 30) | ((ulong)(Next(ref s) & 3) << 26) | ((ulong)(Next(ref s) & 3) << 22)
                    | ((ulong)(twoCycle ? Pick(ref s, 0, 2, 3) : (int)(Next(ref s) & 3)) << 18)
                    | low);
                list.Add(twoCycle ? TwoCycleCombine(ref s) : TexturedCombine(ref s));
                foreach (int id in new[] { 0x38, 0x39, 0x3A, 0x3B, 0x2C, 0x2E })
                    list.Add(((ulong)id << 56) | ((ulong)(Next(ref s) & 0xFF_FFFF) << 32) | ((ulong)Next(ref s) << 8) | (Next(ref s) & 0xFF));

                // One primitive of each list carries no texture coordinates at all: a w of zero is a state the divider reaches and random values do not.
                if (i == Formats.Length && mode != 0)
                {
                    ulong[] degenerate = TexturedTriangle(10, 10, 300, 40, 40, 200, 2, tile, i, 0, ref s);
                    for (int at = 12; at < 20; at++) degenerate[at] = 0;
                    list.AddRange(degenerate);
                }

                double cx = Next(ref s) % Width, cy = Next(ref s) % Rows;
                double Near(double c) => c + (int)(Next(ref s) % 90) - 45 + (Next(ref s) & 3) / 4.0;
                list.AddRange(TexturedTriangle(Near(cx), Near(cy), Near(cx), Near(cy), Near(cx), Near(cy), mode, tile, i, (int)(Next(ref s) & 7), ref s));
            }

            return list.ToArray();
        }

        // A triangle whose texture coordinates do not move: every pixel of it shows the one coordinate the divider answered.
        private static ulong[] FlatTriangle(double left, double top, int sw, int st, int w, int tile, int dsdx = 0, int maxLevel = 0)
        {
            const int id = 0x0B;
            double right = left + 14, bottom = top + 6;

            var words = new ulong[Rdp.Length(id)];
            words[0] = ((ulong)id << 56) | (1UL << 55) | ((ulong)tile << 48) | ((ulong)maxLevel << 51)
                | ((ulong)(uint)(((int)Math.Floor(bottom * 4)) & 0x3FFF) << 32)
                | ((ulong)(uint)(((int)Math.Floor(bottom * 4)) & 0x3FFF) << 16) | (uint)(((int)Math.Floor(top * 4)) & 0x3FFF);
            words[1] = Edge(right, 0);
            words[2] = Edge(left, 0);
            words[3] = Edge(right, 0);

            int[] value = { sw, st, w, 0 };
            int[] across = { dsdx, dsdx, 0, 0 };
            (words[4], words[6]) = (Pack(value, 16), Pack(value, 0));
            (words[5], words[7]) = (Pack(across, 16), Pack(across, 0));
            return words;
        }

        // The perspective divider has fifteen shifts, and at the highest of them it can answer only s of zero or one, which is
        // why a scene of random triangles never shows its arithmetic. Here each triangle holds one coordinate over all its pixels.
        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void Every_shift_of_the_perspective_divider_reaches_the_picture(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            var list = new List<ulong>
            {
                FillCycle,
                ColorImage(Framebuffer, 2), Scissor(0, 0, Width * 4, Rows * 4), FillColor(0x2109_8421), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),

                // A tile that neither clamps nor mirrors, so the whole coordinate reaches the texel rather than its limit.
                (0x3DUL << 56) | (2UL << 51) | (63UL << 32) | TextureSource,
                (0x35UL << 56) | (2UL << 51) | (8UL << 41),
                (0x34UL << 56) | ((31UL << 2) << 12) | (31UL << 2),
                (0x35UL << 56) | (2UL << 51) | (8UL << 41) | (5UL << 14) | (5UL << 4),
                (0x32UL << 56) | ((31UL << 2) << 12) | (31UL << 2),

                // One cycle, perspective, no dither, no depth or alpha test, and a combiner that is the texel itself.
                (0x2FUL << 56) | (1UL << 51) | (3UL << 38) | (3UL << 36),
                TexelOnlyCombine(),
            };

            // The divider answers s times two to the fifteen over w, so a coordinate proportional to w gives the same answer at
            // every shift. These are spread in absolute terms instead, which is what moves the texel at the coarse shifts too.
            var cases = new List<(int W, int S, int T)>();
            for (int bit = 14; bit >= 0; bit--)
            {
                // A w that is not a power of two is what makes the table interpolate between two reciprocals rather than land on one.
                foreach (int w in new[] { 1 << bit, (1 << bit) + (1 << Math.Max(bit - 1, 0)) + 1 }.Distinct())
                {
                    // Coordinates below w, at it and past it: the last are where the divider decides a result is out of range.
                    int[] choices = { 0, 1, 3, 15, 63, w >> 4, w >> 2, w >> 1, (w >> 1) + (w >> 2), w - 1, w, w + (w >> 2), Math.Min(w * 2, 0x7FFF) };
                    int[] distinct = choices.Select(v => Math.Clamp(v, 0, 0x7FFF)).Distinct().ToArray();
                    for (int i = 0; i < distinct.Length; i++) cases.Add((w, distinct[i], distinct[distinct.Length - 1 - i]));
                }
            }

            // The two edges of the sign test, where a w of zero is answered by the flag rather than by the divider.
            cases.Add((0, 0, 0));
            cases.Add((0, 1, 2));
            cases.Add((-1, 5, 5));
            cases.Add((-4096, 100, 200));

            int at = 0;
            foreach ((int w, int sw, int st) in cases)
            {
                list.AddRange(FlatTriangle(2 + at % 20 * 16, 2 + at / 20 * 8, sw << 16, st << 16, w << 16, 0));
                at++;
            }

            list.Add(0x29UL << 56);
            ulong[] words = list.ToArray();

            gpu.Clear();
            var cpu = OnTheCpu(words, scale);
            AssertIdentical(cpu, OnTheDevice(gpu, words, scale), $"at {scale}x on {device.Name}");

            // The scene is worthless if every coordinate saturated to the same texel, so the picture must hold many colours.
            var colours = new HashSet<int>();
            long image = (long)Framebuffer * scale * scale;
            for (int y = 0; y < Rows * scale; y++)
                for (int x = 0; x < Width * scale; x++)
                    colours.Add((cpu.Rdram[image + (y * Width * scale + x) * 2] << 8) | cpu.Rdram[image + (y * Width * scale + x) * 2 + 1]);

            Assert.Equal(0, gpu.PrimitivesNotShaded);
            Assert.True(colours.Count > 16, $"only {colours.Count} colours: the divider's answers are not reaching the picture");
            _output.WriteLine($"{device.Name} at {scale}x: {colours.Count} colours over {at} coordinates");
        }

        // The filter's own corners, which a scene of moving coordinates reaches by accident if at all: the mid texel, which
        // wants both fractions at exactly half a texel, and the wrap, which wants a coordinate on the mask's last texel.
        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void The_filters_corners_reach_the_picture(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            var list = new List<ulong>
            {
                FillCycle,
                ColorImage(Framebuffer, 2), Scissor(0, 0, Width * 4, Rows * 4), FillColor(0x2109_8421), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
                (0x3DUL << 56) | (2UL << 51) | (63UL << 32) | TextureSource,
                (0x35UL << 56) | (2UL << 51) | (0x100UL << 32) | (7UL << 24),
                (0x30UL << 56) | (7UL << 24) | ((ulong)(255 << 2) << 12),
            };

            int at = 0;
            foreach ((int format, int size) in Formats)
            {
                foreach (int mask in new[] { 5, 9, 10 })
                {
                    foreach (int fraction in new[] { 0, 0x10, 0x1F })
                    {
                        bool palette = (at & 1) != 0;

                        // An odd line as well as an even one: with a line of eight the wrap's two answers differ by a whole
                        // multiple of the address mask, so the fetch lands on the same byte and the difference is invisible.
                        int line = 8 + at % 3;

                        list.Add((0x35UL << 56) | ((ulong)format << 53) | ((ulong)size << 51) | ((ulong)line << 41)
                            | ((ulong)mask << 14) | ((ulong)mask << 4));
                        list.Add((0x34UL << 56) | ((31UL << 2) << 12) | (31UL << 2));

                        // One cycle, no perspective, no dither, filtering on, the mid texel on, and the palette on alternate cases.
                        list.Add((0x2FUL << 56) | (3UL << 38) | (3UL << 36) | (1UL << 43) | (1UL << 45) | (1UL << 44)
                            | (palette ? 1UL << 47 : 0) | ((at & 2) != 0 ? 1UL << 46 : 0));
                        list.Add(TexelOnlyCombine());

                        // The mask's last texel for the wrap, an ordinary one otherwise, and the fraction on top of it.
                        int coordinate = ((mask >= 9 ? (1 << mask) - 1 : 20) << 5) | fraction;
                        list.AddRange(FlatTriangle(2 + at % 20 * 16, 2 + at / 20 * 8, coordinate << 16, coordinate << 16, 0, 0));
                        at++;
                    }
                }
            }

            list.Add(0x29UL << 56);
            ulong[] words = list.ToArray();

            gpu.Clear();
            var cpu = OnTheCpu(words, scale);
            AssertIdentical(cpu, OnTheDevice(gpu, words, scale), $"at {scale}x on {device.Name}");

            var colours = new HashSet<int>();
            long image = (long)Framebuffer * scale * scale;
            for (int y = 0; y < Rows * scale; y++)
                for (int x = 0; x < Width * scale; x++)
                    colours.Add((cpu.Rdram[image + (y * Width * scale + x) * 2] << 8) | cpu.Rdram[image + (y * Width * scale + x) * 2 + 1]);

            Assert.Equal(0, gpu.PrimitivesNotShaded);
            Assert.True(colours.Count > 16, $"only {colours.Count} colours: the filter's answers are not reaching the picture");
            _output.WriteLine($"{device.Name} at {scale}x: {colours.Count} colours over {at} tile settings");
        }

        // The level of detail decides a pixel is distant when the level it measured is the tile's last, and the only thing
        // that turns on that equality is the fraction the combiner reads. Random movement lands on it rarely, so it is swept here.
        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void The_level_of_detail_reaches_the_picture(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            var list = new List<ulong>
            {
                FillCycle,
                ColorImage(Framebuffer, 2), Scissor(0, 0, Width * 4, Rows * 4), FillColor(0x2109_8421), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
                (0x3DUL << 56) | (2UL << 51) | (63UL << 32) | TextureSource,
                (0x35UL << 56) | (2UL << 51) | (8UL << 41),
                (0x34UL << 56) | ((31UL << 2) << 12) | (31UL << 2),
            };

            // None of these is a power of two, which would leave the fraction at zero whatever the level; the large ones
            // reach the two high bits of the measurement that declare a pixel distant on their own, at one multiple or another.
            int[] movements = { 35, 70, 141, 290, 583, 1200, 2501, 5003, 0x2100, 0x3300, 0x4200, 0x6300, 0x8400, 0xC600 };

            int at = 0;
            foreach (int movement in movements)
            {
                for (int maxLevel = 0; maxLevel < 8; maxLevel++)
                {
                    // Plain: neither sharpen nor detail, which is the pair that makes a distant pixel's fraction the saturated one.
                    list.Add((0x2FUL << 56) | (3UL << 38) | (3UL << 36) | (1UL << 48) | (1UL << 43) | (1UL << 45));
                    list.Add(LodFractionCombine());
                    list.AddRange(FlatTriangle(2 + at % 8 * 39, 2 + at / 8 * 15, 0, 0, 0, 0, movement << 16, maxLevel));
                    at++;
                }
            }

            list.Add(0x29UL << 56);
            ulong[] words = list.ToArray();

            gpu.Clear();
            var cpu = OnTheCpu(words, scale);
            AssertIdentical(cpu, OnTheDevice(gpu, words, scale), $"at {scale}x on {device.Name}");

            var colours = new HashSet<int>();
            long image = (long)Framebuffer * scale * scale;
            for (int y = 0; y < Rows * scale; y++)
                for (int x = 0; x < Width * scale; x++)
                    colours.Add((cpu.Rdram[image + (y * Width * scale + x) * 2] << 8) | cpu.Rdram[image + (y * Width * scale + x) * 2 + 1]);

            Assert.Equal(0, gpu.PrimitivesNotShaded);
            Assert.True(colours.Count > 4, $"only {colours.Count} colours: the level's fraction is not reaching the picture");
            _output.WriteLine($"{device.Name} at {scale}x: {colours.Count} colours over {at} levels");
        }

        // The colour is the level's own fraction: (0x100 - 0) times it, which lands in the picture unchanged.
        private static ulong LodFractionCombine()
        {
            const int a = 6, b = 8, c = 13, d = 7, alphaA = 6, alphaB = 7, alphaC = 7, alphaD = 7;
            ulong high = (ulong)((a << 20) | (c << 15) | (alphaA << 12) | (alphaC << 9) | (a << 5) | c);
            ulong low = unchecked((uint)((b << 28) | (b << 24) | (alphaA << 21) | (alphaC << 18) | (d << 15) | (alphaB << 12) | (alphaD << 9) | (d << 6) | (alphaB << 3) | alphaD));
            return (0x3CUL << 56) | (high << 32) | low;
        }

        // Texture rectangles in copy mode, which is how a game blits: every format over a loaded tile, flipped and not,
        // with the palette, the alpha test, perspective and the level of detail each on for some of them.
        private static ulong[] Copied(uint seed)
        {
            uint s = seed;
            var list = new List<ulong>
            {
                FillCycle,
                ColorImage(Framebuffer, 2), Scissor(0, 0, Width * 4, Rows * 4), FillColor(0x2109_8421), FillRectangle(0, 0, Width * 4 - 4, Rows * 4 - 4),
                (0x3DUL << 56) | (2UL << 51) | (63UL << 32) | TextureSource,
                (0x35UL << 56) | (2UL << 51) | (0x100UL << 32) | (7UL << 24),
                (0x30UL << 56) | (7UL << 24) | ((ulong)(255 << 2) << 12),
            };

            for (int i = 0; i < Formats.Length * 3; i++)
            {
                (int format, int size) = Formats[i % Formats.Length];

                // Lines up to the field's nine bits, so that a line times a texel row passes the 0x1FF the fetch wraps it at.
                int tile = i % 7, line = (i % 4) == 3 ? 0x40 + (int)(Next(ref s) % 0x1C0u) : 8 + i % 3;
                uint wraps = Next(ref s);

                list.Add((0x35UL << 56) | ((ulong)format << 53) | ((ulong)size << 51) | ((ulong)line << 41) | ((ulong)((i % 3) * 64) << 32)
                    | ((ulong)tile << 24) | ((ulong)(Next(ref s) & 0xF) << 20) | ((wraps & 1) << 19) | ((wraps & 2) << 17)
                    | ((ulong)(Next(ref s) % 11u) << 14) | ((ulong)(Next(ref s) % 16u) << 10)
                    | ((wraps & 4) << 7) | ((wraps & 8) << 5) | ((ulong)(Next(ref s) % 11u) << 4) | (Next(ref s) % 16u));
                list.Add((0x34UL << 56) | ((ulong)tile << 24) | ((31UL << 2) << 12) | (31UL << 2));

                // Copy is cycle type two; the bits below it that copy mode reads are the alpha test, the palette, perspective and the level.
                list.Add((0x2FUL << 56) | (2UL << 52) | (Next(ref s) & 1) | ((ulong)(Next(ref s) & 1) << 47) | ((ulong)(Next(ref s) & 1) << 46)
                    | ((ulong)(Next(ref s) & 1) << 51) | ((ulong)(Next(ref s) & 1) << 48) | ((ulong)(Next(ref s) & 1) << 50) | ((ulong)(Next(ref s) & 1) << 49));
                list.Add((0x3AUL << 56) | ((ulong)(Next(ref s) & 0xFF_FFFF) << 32) | ((ulong)Next(ref s) << 8) | (Next(ref s) & 0xFF));

                int left = (int)(Next(ref s) % (Width * 4 - 200)), top = (int)(Next(ref s) % (Rows * 4 - 120));
                int right = left + 16 + (int)(Next(ref s) % 180), bottom = top + 8 + (int)(Next(ref s) % 110);
                ulong id = (i & 1) == 0 ? 0x24UL : 0x25UL;
                list.Add((id << 56) | ((ulong)right << 44) | ((ulong)bottom << 32) | ((ulong)tile << 24) | ((ulong)left << 12) | (uint)top);

                // s and t in 10.5, and their steps in 5.10: a copy steps four texels a pixel, and anything else is legal too.
                // Coordinates over the whole of their sixteen bits for a third of them, which reaches texel rows far past the tile.
                bool far = i % 3 == 1;
                int sStart = (int)(Next(ref s) % (far ? 0x8000u : 1024u)), tStart = (int)(Next(ref s) % (far ? 0x8000u : 1024u));
                int dsdx = (i % 3) switch { 0 => 4 << 10, 1 => 1 << 10, _ => (int)(Next(ref s) & 0x1FFF) };
                int dtdy = (i % 3) == 2 ? (int)(Next(ref s) & 0x1FFF) : 1 << 10;
                list.Add(((ulong)(uint)sStart << 48) | ((ulong)(uint)tStart << 32) | ((ulong)(ushort)dsdx << 16) | (ushort)dtdy);
            }

            list.Add(0x29UL << 56);
            return list.ToArray();
        }

        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void Copy_mode_rectangles_on_the_device_are_the_cpus_byte_for_byte(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            foreach (uint seed in new[] { 0xC0B1_0001u, 0xC0B1_0002u, 0xC0B1_0003u })
            {
                gpu.Clear();
                ulong[] list = Copied(seed);
                var cpu = OnTheCpu(list, scale);
                AssertIdentical(cpu, OnTheDevice(gpu, list, scale), $"seed {seed:X8} at {scale}x on {device.Name}");
            }

            Assert.Equal(0, gpu.PrimitivesNotShaded);
            _output.WriteLine($"{device.Name} at {scale}x: {gpu.RowsShaded} rows in {gpu.Flushes} flushes");
        }

        private static ulong TexelOnlyCombine()
        {
            // A and B are zero so C does not matter, and C must not be seven, which is the previous pixel's alpha.
            const int a = 7, b = 7, c = 31, d = 1, alphaA = 7, alphaB = 7, alphaC = 7, alphaD = 1;
            ulong high = (ulong)((a << 20) | (c << 15) | (alphaA << 12) | (alphaC << 9) | (a << 5) | c);
            ulong low = unchecked((uint)((b << 28) | (b << 24) | (alphaA << 21) | (alphaC << 18) | (d << 15) | (alphaB << 12) | (alphaD << 9) | (d << 6) | (alphaB << 3) | alphaD));
            return (0x3CUL << 56) | (high << 32) | low;
        }

        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void Point_sampled_textures_on_the_device_are_the_cpus_byte_for_byte(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            // No perspective, the divider in its ordinary range, and the divider at its edges: a w of zero, below zero, and past what it can answer.
            foreach ((uint seed, int mode, bool twoCycle) in new[]
            {
                (0x0BAD_F00Du, 0, false), (0x1234_ABCDu, 1, false), (0x5EED_1234u, 2, false), (0x2468_ACE0u, 1, false),
                (0x0FED_CBA9u, 0, true), (0x7531_ECA8u, 1, true), (0xFACE_B00Cu, 2, true),
            })
            {
                gpu.Clear();
                ulong[] list = Textured(seed, mode, twoCycle);
                AssertIdentical(OnTheCpu(list, scale), OnTheDevice(gpu, list, scale),
                    $"seed {seed:X8}, mode {mode}, {(twoCycle ? "two" : "one")} cycle, at {scale}x on {device.Name}");
            }

            Assert.Equal(0, gpu.PrimitivesNotShaded);
            _output.WriteLine($"{device.Name} at {scale}x: {gpu.RowsShaded} rows in {gpu.Flushes} flushes");
        }

        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void Shaded_depth_tested_triangles_on_the_device_are_the_cpus_byte_for_byte(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            foreach ((uint seed, int size, bool keyed, bool twoCycle) in new[]
            {
                (0x1111_2222u, 2, false, false), (0x3333_4444u, 3, false, false), (0x5555_6666u, 2, false, false),
                (0x7777_8888u, 2, true, false), (0x9999_AAAAu, 3, true, false),
                (0xBBBB_CCCCu, 2, false, true), (0xDDDD_EEEEu, 3, false, true), (0x1357_9BDFu, 2, false, true),
            })
            {
                gpu.Clear();
                ulong[] list = Shaded(seed, size, keyed, twoCycle);
                AssertIdentical(OnTheCpu(list, scale), OnTheDevice(gpu, list, scale),
                    $"seed {seed:X8}, {(size == 2 ? 16 : 32)}-bit, {(twoCycle ? "two" : "one")} cycle, at {scale}x on {device.Name}");
            }

            Assert.Equal(0, gpu.PrimitivesNotShaded);
            Assert.Equal(0, gpu.ColumnsPastTheWidth);
            _output.WriteLine($"{device.Name} at {scale}x: {gpu.RowsShaded} rows in {gpu.Flushes} flushes");
        }

        // A frame's worth of small triangles, which is what a game's list looks like, where Shaded's few large ones are overdraw.
        private static ulong[] Small(uint seed, int triangles)
        {
            uint s = seed;
            var list = new List<ulong>(Shaded(seed, 2).Take(9));

            for (int i = 0; i < triangles; i++)
            {
                if (i % 16 == 0) { list.Add(OneCycleModes(ref s)); list.Add(Combine(ref s)); }
                double cx = Next(ref s) % Width, cy = Next(ref s) % Rows;
                double Near(double c) => c + (int)(Next(ref s) % 40) - 20 + (Next(ref s) & 3) / 4.0;
                list.AddRange(Triangle(Near(cx), Near(cy), Near(cx), Near(cy), Near(cx), Near(cy), ref s));
            }

            return list.ToArray();
        }

        // The device behind DpInterface, which is how a game reaches it: the same list through the interface with the
        // device on and off must leave the same shadow memory, threaded or not - see Mars_Gpu.md §11.
        [Theory]
        [InlineData(2, 1)]
        [InlineData(2, 3)]
        [InlineData(3, 4)]
        [InlineData(4, 1)]
        public void The_interface_draws_the_multiple_on_the_device_as_it_does_on_the_cpu(int scale, int workers)
        {
            if (GpuDevice.DeviceNames().Count == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            (byte[] Rdram, byte[] Hidden, string Report) Through(bool gpu)
            {
                var bus = new MemoryBus();
                SeedTextureSource(bus);
                bus.Dp.Scale = scale;
                bus.Dp.Gpu = gpu;
                if (workers > 1) { bus.Dp.Threaded = true; bus.Dp.Workers = workers; }

                foreach (ulong[] list in new[] { Shaded(0x2222_7777, 2), Textured(0x3333_8888, 1), Shaded(0x4444_9999, 2, false, true) })
                {
                    for (int i = 0; i < list.Length; i++) bus.Write64(0x0010_0000 + (uint)i * 8, list[i]);
                    bus.Write32(MemoryMap.DpCommandBase, 0x0010_0000);
                    bus.Write32(MemoryMap.DpCommandBase + 4, 0x0010_0000 + (uint)list.Length * 8);
                    bus.Dp.Join();
                }

                bus.Dp.ReadBackScaled(0, bus.Dp.ScaledRdram.Length);
                Assert.True(bus.Dp.ScaledDrawn, "nothing was drawn at the multiple");
                return (bus.Dp.ScaledRdram, bus.Dp.ScaledHidden, bus.Dp.GpuReport);
            }

            var cpu = Through(false);
            var device = Through(true);

            Assert.DoesNotContain("no Vulkan", device.Report);
            int at = cpu.Rdram.AsSpan().CommonPrefixLength(device.Rdram);
            if (at != cpu.Rdram.Length)
                Assert.Fail($"{scale}x with {workers} workers on {device.Report}: byte {at:X} is {device.Rdram[at]:X2} on the device and {cpu.Rdram[at]:X2} on the CPU");
            Assert.True(cpu.Hidden.AsSpan().SequenceEqual(device.Hidden), "the hidden bits differ");
            _output.WriteLine($"{scale}x with {workers} workers on {device.Report}: identical");
        }

        // The setting's promise: at one it changes nothing, holds no device, and says so.
        [Fact]
        public void At_one_the_device_setting_changes_nothing_and_holds_no_device()
        {
            (byte[] Rdram, byte[] Hidden, string Report) Through(bool gpu)
            {
                var bus = new MemoryBus();
                SeedTextureSource(bus);
                bus.Dp.Gpu = gpu;

                ulong[] list = Shaded(0x5151_0101, 2);
                for (int i = 0; i < list.Length; i++) bus.Write64(0x0010_0000 + (uint)i * 8, list[i]);
                bus.Write32(MemoryMap.DpCommandBase, 0x0010_0000);
                bus.Write32(MemoryMap.DpCommandBase + 4, 0x0010_0000 + (uint)list.Length * 8);
                bus.Dp.Join();
                return (bus.Rdram, bus.RdramHidden, bus.Dp.GpuReport);
            }

            var off = Through(false);
            var on = Through(true);

            Assert.Equal("off at one", on.Report);
            Assert.True(off.Rdram.AsSpan().SequenceEqual(on.Rdram), "the machine's own memory changed with the device setting at one");
            Assert.True(off.Hidden.AsSpan().SequenceEqual(on.Hidden));
        }

        // Not a test of anything: the stop-or-go measurement of Mars_Gpu.md §6.5, run by hand with EMUSEN_MARS_GPU_BENCH=1.
        [Fact]
        public void Bench_shading_at_a_multiple_on_the_cpu_and_on_the_device()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_MARS_GPU_BENCH") != "1") return;

            using GpuDevice device = GpuDevice.TryCreate(null, out string report) ?? throw new InvalidOperationException(report);
            _output.WriteLine(report);

            foreach (int scale in new[] { 2, 4 })
            {
                using GpuRasteriser gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report) ?? throw new InvalidOperationException(report);

                foreach ((string name, ulong[] list) in new[] { ("72 large", Shaded(0x1111_2222, 2)), ("2000 small", Small(0x2222_3333, 2000)) })
                {
                    var bus = new MemoryBus();
                    var frame = new byte[bus.Rdram.Length * scale * scale];
                    var hidden = new byte[bus.RdramHidden.Length * scale * scale];
                    long pictureAt = (long)Framebuffer * scale * scale, pictureBytes = (long)Width * Rows * 2 * scale * scale;

                    var cpu = new List<double>();
                    var gpuTotal = new List<double>();
                    var gpuHost = new List<double>();

                    for (int run = 0; run < 9; run++)
                    {
                        var clock = System.Diagnostics.Stopwatch.StartNew();
                        var onCpu = new Rdp(bus);
                        onCpu.DrawAt(scale, frame, hidden);
                        foreach (ulong word in list) onCpu.Accept(word);
                        cpu.Add(clock.Elapsed.TotalMilliseconds);

                        clock.Restart();
                        var onDevice = new Rdp(bus);
                        onDevice.DrawAt(scale, frame, hidden);
                        onDevice.ShadeOn(gpu);
                        foreach (ulong word in list) onDevice.Accept(word);
                        gpuHost.Add(clock.Elapsed.TotalMilliseconds);
                        gpu.Read(pictureAt, pictureBytes, frame, hidden);
                        gpuTotal.Add(clock.Elapsed.TotalMilliseconds);
                    }

                    static double Median(List<double> v) => v.OrderBy(x => x).ElementAt(v.Count / 2);
                    _output.WriteLine($"{scale}x, {name}: CPU one worker {Median(cpu):F1} ms (a quarter of it {Median(cpu) / 4:F1}); device {Median(gpuTotal):F1} ms, of which walking and recording on the host {Median(gpuHost):F1}");
                }
            }
        }

        private static (byte[] Rdram, byte[] Hidden) OnTheCpu(ulong[] list, int scale)
        {
            var bus = new MemoryBus();
            SeedTextureSource(bus);
            var frame = new byte[bus.Rdram.Length * scale * scale];
            var hidden = new byte[bus.RdramHidden.Length * scale * scale];
            var processor = new Rdp(bus);
            processor.DrawAt(scale, frame, hidden);
            foreach (ulong word in list) processor.Accept(word);
            return (frame, hidden);
        }

        private static (byte[] Rdram, byte[] Hidden) OnTheDevice(GpuRasteriser gpu, ulong[] list, int scale)
        {
            var bus = new MemoryBus();
            SeedTextureSource(bus);
            var frame = new byte[bus.Rdram.Length * scale * scale];
            var hidden = new byte[bus.RdramHidden.Length * scale * scale];
            var processor = new Rdp(bus);
            processor.DrawAt(scale, frame, hidden);
            processor.ShadeOn(gpu);
            foreach (ulong word in list) processor.Accept(word);
            gpu.Read(0, frame.Length, frame, hidden);
            return (frame, hidden);
        }

        private static void AssertIdentical((byte[] Rdram, byte[] Hidden) cpu, (byte[] Rdram, byte[] Hidden) device, string where)
        {
            int at = cpu.Rdram.AsSpan().CommonPrefixLength(device.Rdram);
            Assert.True(at == cpu.Rdram.Length, $"{where}: byte {at:X} is {(at < device.Rdram.Length ? device.Rdram[at] : -1):X2} on the device and {(at < cpu.Rdram.Length ? cpu.Rdram[at] : -1):X2} on the CPU");

            at = cpu.Hidden.AsSpan().CommonPrefixLength(device.Hidden);
            Assert.True(at == cpu.Hidden.Length, $"{where}: hidden bits of word {at:X} differ");
        }

        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void Fills_on_the_device_are_the_fills_on_the_cpu_byte_for_byte(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            long bytes = (long)new MemoryBus().Rdram.Length * scale * scale;
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, bytes, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            foreach (int seed in new[] { 1, 2, 3 })
            {
                gpu.Clear();
                ulong[] list = Fills(seed);
                var cpu = OnTheCpu(list, scale);
                Assert.Contains(cpu.Rdram, b => b != 0);

                AssertIdentical(cpu, OnTheDevice(gpu, list, scale), $"seed {seed} at {scale}x on {device.Name}");
            }

            Assert.Equal(0, gpu.ColumnsPastTheWidth);
            Assert.Equal(0, gpu.PrimitivesNotShaded);
            Assert.True(gpu.Flushes >= 12, $"four images a list should end four batches a list, and {gpu.Flushes} were flushed");
            _output.WriteLine($"{device.Name} at {scale}x: {gpu.RowsShaded} rows in {gpu.Flushes} flushes");
        }

        // The CPU path lets a span run past the image's width into the next row's bytes; the device clips it, and says how much - see §5.
        [Fact]
        public void A_scissor_wider_than_the_image_is_clipped_on_the_device_and_counted()
        {
            using GpuDevice? device = GpuDevice.TryCreate(null, out _);
            if (device is null) return;
            using GpuRasteriser gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * 4, out string report) ?? throw new InvalidOperationException(report);

            OnTheDevice(gpu, Fills(4, scissorRight: (Width + 16) * 4), 2);
            Assert.True(gpu.ColumnsPastTheWidth > 0);
        }

        [Fact]
        public void The_device_is_never_given_the_machines_own_picture()
        {
            using GpuDevice? device = GpuDevice.TryCreate(null, out _);
            if (device is null) return;
            using GpuRasteriser gpu = GpuRasteriser.TryCreate(device, new MemoryBus().Rdram.Length * 4L, out _)!;

            Assert.Throws<InvalidOperationException>(() => new Rdp(new MemoryBus()).ShadeOn(gpu));
        }
    }
}
