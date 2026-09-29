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
    }
}
