using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // A $420C bit going 0->1 arms that channel immediately - see
    // Venus_Memory.md §3.2a for the Yoshi's Island intro that needs it.
    public class HdmaMidFrameEnableTests
    {
        // Channel 0 pointed at a one-block table in WRAM writing TM ($212C).
        private static VenusCore BuildCoreWithHdmaTable(byte lineCount, byte value)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var bus = core.Bus!;

            bus.Ram[0x1000] = lineCount;
            bus.Ram[0x1001] = value;
            bus.Ram[0x1002] = 0x00;   // table terminator

            bus.Write8(0x004300, 0x00);   // mode 0, direct, A->B
            bus.Write8(0x004301, 0x2C);   // destination TM ($212C)
            bus.Write8(0x004302, 0x00);
            bus.Write8(0x004303, 0x10);   // table at $7E:1000
            bus.Write8(0x004304, 0x7E);

            return core;
        }

        [Fact]
        public void Writing_420c_mid_frame_arms_the_channel()
        {
            var core = BuildCoreWithHdmaTable(lineCount: 4, value: 0x08);
            var dma = core.Bus!.Dma;

            core.Bus.CurrentScanline = 12;
            core.Bus.Write8(0x00420C, 0x01);

            Assert.Equal(0x01, dma.HdmaEnable);
            Assert.True(dma.DebugChannels()[0].HdmaActive);
        }

        [Fact]
        public void A_channel_armed_mid_frame_transfers_on_the_next_scanline()
        {
            var core = BuildCoreWithHdmaTable(lineCount: 4, value: 0x08);

            core.Bus!.Ppu.Tm = 0x15;
            core.Bus.CurrentScanline = 12;
            core.Bus.Write8(0x00420C, 0x01);
            core.Bus.Dma.ExecuteHdma();

            Assert.Equal(0x08, core.Bus.Ppu.Tm);
        }

        // The regression this guards: arming only at scanline 0 left a game
        // that enables and disables HDMA inside one frame with no transfers.
        [Fact]
        public void Enabling_and_disabling_within_one_frame_still_transfers()
        {
            var core = BuildCoreWithHdmaTable(lineCount: 4, value: 0x08);
            var bus = core.Bus!;

            bus.CurrentScanline = 0;
            bus.Dma.InitHdma();               // $420C is still 0 here
            Assert.False(bus.Dma.DebugChannels()[0].HdmaActive);

            bus.Ppu.Tm = 0x15;
            bus.CurrentScanline = 12;
            bus.Write8(0x00420C, 0x01);
            bus.Dma.ExecuteHdma();
            Assert.Equal(0x08, bus.Ppu.Tm);

            bus.CurrentScanline = 198;
            bus.Write8(0x00420C, 0x00);
            bus.Ppu.Tm = 0x15;
            bus.Dma.ExecuteHdma();
            Assert.Equal(0x15, bus.Ppu.Tm);   // disabled again, no transfer
        }

        // Re-writing the same mask must not re-read the table header, or a
        // channel part-way through its table would restart every write.
        [Fact]
        public void Rewriting_an_unchanged_mask_does_not_rearm()
        {
            var core = BuildCoreWithHdmaTable(lineCount: 2, value: 0x08);
            var bus = core.Bus!;

            bus.CurrentScanline = 12;
            bus.Write8(0x00420C, 0x01);
            bus.Dma.ExecuteHdma();
            ushort afterFirstLine = (ushort)bus.Dma.DebugChannels()[0].TableAddress;

            bus.Write8(0x00420C, 0x01);
            Assert.Equal(afterFirstLine, (ushort)bus.Dma.DebugChannels()[0].TableAddress);
        }
    }
}
