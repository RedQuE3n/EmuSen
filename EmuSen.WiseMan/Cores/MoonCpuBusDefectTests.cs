using System;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The CPU and bus defects stage 2a fixed, each failing on the unmodified core - see Moon_Native.md §3.8.
    public class MoonCpuBusDefectTests
    {
        // D6: nothing on these boards answers a read of $4020-$5FFF, so it returns the last value on the bus, not PRG RAM's mirror.
        [Fact]
        public void A_read_of_the_expansion_area_is_open_bus()
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(mapper: 0));
            try
            {
                var core = new MoonCore();
                core.LoadRom(path);
                var bus = core.Bus!;
                core.Cart!.PrgRam[0x1000] = 0x11;
                core.Cart.PrgRam[0x0108] = 0x22;
                bus.OpenBus = 0x5A;
                Assert.Equal(0x5A, bus.Read(0x5000));
                Assert.Equal(0x5A, bus.Read(0x4208));
                core.Cart.PrgRam[0] = 0x33;
                Assert.Equal(0x33, bus.Read(0x6000));
            }
            finally
            {
                File.Delete(path);
            }
        }

        // D8: $4015 answers on the chip's internal bus, so bit 5 is whatever was last on the bus and the bus itself is left as it was.
        [Fact]
        public void A_read_of_4015_floats_bit_5_and_leaves_the_bus()
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(mapper: 0));
            try
            {
                var core = new MoonCore();
                core.LoadRom(path);
                var bus = core.Bus!;
                bus.OpenBus = 0x7A;
                Assert.Equal(0x20, bus.Read(0x4015) & 0x20);
                Assert.Equal(0x7A, bus.OpenBus);
                bus.OpenBus = 0x45;
                Assert.Equal(0x00, bus.Read(0x4015) & 0x20);
                Assert.Equal(0x45, bus.Read(0x4018));
            }
            finally
            {
                File.Delete(path);
            }
        }
        private static MoonCore Run(byte[] rom)
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(rom);
            try
            {
                var core = new MoonCore();
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        // Sets NMI on and spins; the NMI handler at $8100 copies page 2 to OAM when dma is set, and returns.
        private static byte[] NmiRom(bool dma)
        {
            byte[] main = { 0x78, 0xA9, 0x80, 0x8D, 0x00, 0x20, 0x4C, 0x06, 0x80 };
            byte[] nmi = dma ? new byte[] { 0xA9, 0x02, 0x8D, 0x14, 0x40, 0x40 } : new byte[] { 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0x40 };
            byte[] rom = SyntheticNesRom.Build(mapper: 0, patches: new[] { (0, main), (0x100, nmi) });
            rom[16 + 0x3FFA] = 0x00;
            rom[16 + 0x3FFB] = 0x81;
            return rom;
        }

        // D1: OAM DMA's 513 cycles pass for the PPU too, so a frame that runs one lasts as long, and makes as much sound, as one that does not.
        [Fact]
        public void An_oam_dma_leaves_the_frame_its_length_and_sound()
        {
            double PerFrame(bool dma)
            {
                var core = Run(NmiRom(dma));
                for (int f = 0; f < 60; f++) { core.RunFrame(); core.DequeueAudioSamples(int.MaxValue); }
                long samples = 0;
                for (int f = 0; f < 600; f++) { core.RunFrame(); samples += core.DequeueAudioSamples(int.MaxValue).Length / 2; }
                return samples / 600.0;
            }
            double with = PerFrame(true), without = PerFrame(false);
            Assert.True(Math.Abs(with - without) < 0.05, $"{with:F2} stereo samples a frame with OAM DMA, {without:F2} without");
        }

        // C2: a DMC fetch halts the CPU on a read and repeats it, so reads of $2007 under a looping sample move v further than they are counted.
        [Fact]
        public void A_dmc_fetch_repeats_a_halted_read_of_2007()
        {
            byte[] code =
            {
                0x78, 0xA9, 0x00, 0x8D, 0x00, 0x20, 0x8D, 0x01, 0x20,
                0xA9, 0x20, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20,
                0xA9, 0x4F, 0x8D, 0x10, 0x40, 0xA9, 0x00, 0x8D, 0x12, 0x40, 0x8D, 0x13, 0x40,
                0xA9, 0x10, 0x8D, 0x15, 0x40,
                0xAD, 0x07, 0x20, 0xE6, 0x00, 0xD0, 0x02, 0xE6, 0x01, 0x4C, 0x25, 0x80,
            };
            var core = Run(SyntheticNesRom.Build(mapper: 0, patches: new[] { (0, code) }));
            core.RunFrame();
            core.RunFrame();
            int counted = core.Bus!.Ram[0] | (core.Bus.Ram[1] << 8);
            int moved = (core.Ppu!.V - 0x2000) & 0x3FFF;
            Assert.True(counted > 1000, $"{counted} reads counted");
            Assert.True(moved - counted >= 10, $"v moved {moved} for {counted} counted reads");
        }

        // D5: the DMC's rate table is its period in CPU cycles, so a byte lasts eight periods; version 3's model gave it eight more cycles.
        [Theory]
        [InlineData(0, 428)]
        [InlineData(14, 72)]
        [InlineData(15, 54)]
        public void A_dmc_byte_lasts_eight_periods_of_its_rate(int rate, int period)
        {
            var dmc = new EmuSen.Cores.Nintendo.Moon.Apu.DmcChannel();
            dmc.Reset();
            dmc.RateIndex = rate;
            dmc.SampleLength = 16;
            dmc.SetEnabled(true, onGetCycle: true);
            var fetches = new System.Collections.Generic.List<int>();
            for (int cycle = 0; cycle < period * 8 * 4; cycle++)
            {
                dmc.StepTimer();
                if (!dmc.DmaRequested) continue;
                fetches.Add(cycle);
                dmc.CompleteDma(0);
            }
            Assert.True(fetches.Count >= 3, $"{fetches.Count} fetches");
            Assert.Equal(period * 8, fetches[2] - fetches[1]);
        }
    }
}
