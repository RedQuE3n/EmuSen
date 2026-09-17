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
        private const uint SetColorImage = 0x3F;

        [Fact]
        public void Wave_Race_hands_its_first_display_list_to_the_display_processor()
        {
            string path = Path.Combine(N64TestRomLibrary.Root, WaveRace);
            if (!File.Exists(path)) return;

            var bus = new MarsBus();
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

        private static void Pulse(MarsBus bus, long instruction)
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
