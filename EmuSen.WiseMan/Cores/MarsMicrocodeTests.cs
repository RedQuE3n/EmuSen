using System.IO;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Phase C's second condition: commercial microcode running to a display list - see Mars_Microcode.md.
    public class MarsMicrocodeTests
    {
        private const string WaveRace = "Wave Race 64 (USA) (Rev A).z64";

        private const int Budget = 80_000_000;

        // A stand-in for the devices Phase E builds, not a model of their timing - see Mars_Microcode.md §3.
        private const int PulsePeriod = 1_500_000;
        private const int PulseLength = 5_000;
        private const int SerialOffset = 700_000;

        // Where the operating system leaves the task it gave the RSP, and the one word of it read here.
        private const uint TaskInDmem = MemoryMap.SpDmemBase + 0xFC0;
        private const uint GraphicsTask = 1;

        private const uint SyncFull = 0x29;
        private const uint FillRectangle = 0x36;
        private const uint SetFillColor = 0x37;
        private const uint SetDepthImage = 0x3E;
        private const uint SetColorImage = 0x3F;

        [Fact]
        public void Wave_Race_hands_its_first_display_list_to_the_display_processor()
        {
            if (!RunToFirstGraphicsBreak(out MemoryBus bus)) return;

            uint start = bus.Read32(MemoryMap.DpCommandBase);
            uint end = bus.Read32(MemoryMap.DpCommandBase + 4);
            Assert.True(end > start, $"no commands were handed to the display processor ({start:X8}..{end:X8})");

            uint last = 0;
            bool colorImage = false, fill = false;

            for (uint at = start; at < end; at += CommandLength(last))
            {
                last = (bus.Read32(at) >> 24) & 0x3F;
                Assert.True(IsCommand(last), $"{bus.Read32(at):X8} at {at:X8} is not a display processor command");

                colorImage |= last == SetColorImage;
                fill |= last == FillRectangle;
            }

            Assert.True(colorImage && fill, "the list neither sets a framebuffer nor fills it");
            Assert.Equal(SyncFull, last);
        }

        // The list's one fill is a depth clear, and every pixel a whole pixel inside its edges now holds it - see Mars_Rdp.md §8.
        [Fact]
        public void Wave_Race_first_display_list_clears_its_depth_buffer()
        {
            if (!RunToFirstGraphicsBreak(out MemoryBus bus)) return;

            uint start = bus.Read32(MemoryMap.DpCommandBase);
            uint end = bus.Read32(MemoryMap.DpCommandBase + 4);
            Assert.Equal(end, bus.Read32(MemoryMap.DpCommandBase + 8));

            uint depthImage = 0, colorImage = 0, width = 0, color = 0, filledImage = 0, id = 0;
            ulong rectangle = 0;

            for (uint at = start; at < end; at += CommandLength(id))
            {
                ulong word = bus.Read64(at);
                id = (uint)(word >> 56) & 0x3F;

                if (id == SetDepthImage) depthImage = (uint)word & 0xFF_FFFF;
                if (id == SetColorImage) (colorImage, width) = ((uint)word & 0xFF_FFFF, (uint)(word >> 32) & 0x3FF);
                if (id == SetFillColor) color = (uint)word;
                if (id == FillRectangle) (rectangle, filledImage) = (word, colorImage);
            }

            Assert.Equal(depthImage, filledImage);
            Assert.Equal(color >> 16, color & 0xFFFF);

            uint right = (uint)(rectangle >> 46) & 0x3FF, bottom = (uint)(rectangle >> 34) & 0x3FF;
            uint left = (uint)(rectangle >> 14) & 0x3FF, top = (uint)(rectangle >> 2) & 0x3FF;

            for (uint y = top + 1; y < bottom; y++)
            {
                for (uint x = left + 1; x < right; x++)
                {
                    Assert.Equal(color & 0xFFFF, bus.Read16(filledImage + (y * (width + 1) + x) * 2));
                }
            }

            Assert.Equal(0u, bus.Read16(filledImage));
        }

        private static bool RunToFirstGraphicsBreak(out MemoryBus bus)
        {
            bus = new MemoryBus();

            string path = Path.Combine(N64TestRomLibrary.Root, WaveRace);
            if (!File.Exists(path)) return false;

            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, RomImage.Load(path));

            bool graphicsStarted = false;
            bool wasHalted = true;

            for (long i = 0; i < Budget; i++)
            {
                cpu.Step();
                Pulse(bus, i);

                bool halted = bus.Sp.Processor.Halted;
                if (wasHalted && !halted && bus.Read32(TaskInDmem) == GraphicsTask) graphicsStarted = true;
                if (graphicsStarted && halted && bus.Sp.Processor.Broke) break;
                wasHalted = halted;
            }

            Assert.True(graphicsStarted, "no graphics task reached the RSP");
            Assert.True(bus.Sp.Processor.Broke, "the graphics task did not run to its break");

            return true;
        }

        private static void Pulse(MemoryBus bus, long instruction)
        {
            long phase = instruction % PulsePeriod;

            if (phase == 0 && instruction > 0) bus.Mi.Raise(MiInterrupt.VideoInterface);
            if (phase == PulseLength) bus.Mi.Clear(MiInterrupt.VideoInterface);
            if (phase == SerialOffset) bus.Mi.Raise(MiInterrupt.SerialInterface);
            if (phase == SerialOffset + PulseLength) bus.Mi.Clear(MiInterrupt.SerialInterface);
        }

        // The documented command numbers: no-op, eight triangle forms, and 0x24 up, less the unused 0x31 - see §4.
        private static bool IsCommand(uint id) => id == 0 || id is >= 0x08 and <= 0x0F || (id >= 0x24 && id != 0x31);

        // Triangles grow by what they carry, texture rectangles are two words, everything else is one - see §4.
        private static uint CommandLength(uint id) => id switch
        {
            >= 0x08 and <= 0x0F => 32 + ((id & 4) != 0 ? 64u : 0) + ((id & 2) != 0 ? 64u : 0) + ((id & 1) != 0 ? 16u : 0),
            0x24 or 0x25 => 16,
            _ => 8,
        };
    }
}
