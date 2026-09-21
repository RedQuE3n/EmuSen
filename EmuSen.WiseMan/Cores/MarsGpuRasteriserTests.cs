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
