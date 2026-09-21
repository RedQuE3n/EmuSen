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

        // The one-cycle mode with everything below the cycle type a seed's: dithers, key, the first blend's four selectors, and the sixteen low bits whole.
        private static ulong OneCycleModes(ref uint s)
        {
            ulong modes = (0x2FUL << 56) | ((ulong)(Next(ref s) & 3) << 38) | ((ulong)(Next(ref s) & 3) << 36);
            if (Next(ref s) % 3 == 0) modes |= 1UL << 40;
            modes |= (ulong)(Next(ref s) & 3) << 30 | (ulong)(Next(ref s) & 3) << 26 | (ulong)(Next(ref s) & 3) << 22 | (ulong)(Next(ref s) & 3) << 18;
            return modes | (Next(ref s) & 0x7FFF);
        }

        // Depth cleared by filling it as a colour image, the picture cleared, then triangles over and past the image, each under modes, colours and a combiner of its own.
        private static ulong[] Shaded(uint seed, int size, bool keyed = false)
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
                list.Add(keyed ? (0x2FUL << 56) | ((ulong)(Next(ref s) & 3) << 38) | (1UL << 40) | (1UL << 22) | (1UL << 14) : OneCycleModes(ref s));
                list.Add(keyed ? KeyedCombine(Pick(ref s, 10, 12, 15)) : Combine(ref s));
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

        [Theory]
        [MemberData(nameof(DevicesAndScales))]
        public void Shaded_depth_tested_triangles_on_the_device_are_the_cpus_byte_for_byte(string deviceName, int scale)
        {
            if (deviceName.Length == 0) { _output.WriteLine("no Vulkan device: the CPU path's machine"); return; }

            using GpuDevice device = GpuDevice.TryCreate(deviceName, out string report) ?? throw new InvalidOperationException(report);
            using GpuRasteriser? gpu = GpuRasteriser.TryCreate(device, (long)new MemoryBus().Rdram.Length * scale * scale, out report);
            if (gpu is null) { _output.WriteLine($"not run: {report}"); return; }

            foreach ((uint seed, int size, bool keyed) in new[] { (0x1111_2222u, 2, false), (0x3333_4444u, 3, false), (0x5555_6666u, 2, false), (0x7777_8888u, 2, true), (0x9999_AAAAu, 3, true) })
            {
                gpu.Clear();
                ulong[] list = Shaded(seed, size, keyed);
                AssertIdentical(OnTheCpu(list, scale), OnTheDevice(gpu, list, scale), $"seed {seed:X8}, {(size == 2 ? 16 : 32)}-bit, at {scale}x on {device.Name}");
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
