using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Memory
{
    // The header country byte picks 262-line/60Hz NTSC or 312-line/50Hz PAL
    // timing, and STAT78 bit 4 tells the game which one it got - see
    // Venus_CPU.md §8.5c. Donkey Kong Country 2 is the game that needs it:
    // the local dump is a European (country $02) cartridge, and it stops on
    // Nintendo's "not designed for your SUPER NES" lockout screen unless the
    // console reports itself as PAL.
    public class ConsoleRegionTests
    {
        private const uint Stat78 = 0x213F;

        // LoROM header base $7FC0, country byte at +$19.
        private const int CountryByteOffset = 0x7FD9;

        private static VenusCore CoreWithCountry(byte country) =>
            SyntheticRom.LoadCore(SyntheticRom.Build((CountryByteOffset, new[] { country })));

        [Theory]
        [InlineData(0x00)] // Japan
        [InlineData(0x01)] // USA
        [InlineData(0x0D)] // South Korea
        [InlineData(0x0F)] // Canada
        [InlineData(0x10)] // Brazil (PAL-M, but 60Hz)
        public void Ntsc_country_codes_classify_as_ntsc(byte country)
        {
            Assert.Equal(ConsoleRegion.Ntsc, ConsoleRegions.FromCountryCode(country));
        }

        [Theory]
        [InlineData(0x02)] // Europe/Oceania/Asia
        [InlineData(0x03)] // Scandinavia
        [InlineData(0x06)] // France
        [InlineData(0x09)] // Germany
        [InlineData(0x0C)] // Indonesia
        [InlineData(0x11)] // Australia
        public void Pal_country_codes_classify_as_pal(byte country)
        {
            Assert.Equal(ConsoleRegion.Pal, ConsoleRegions.FromCountryCode(country));
        }

        [Fact]
        public void An_unknown_country_code_falls_back_to_ntsc()
        {
            Assert.Equal(ConsoleRegion.Ntsc, ConsoleRegions.FromCountryCode(0xEA));
        }

        [Fact]
        public void The_cartridge_header_drives_the_cores_region()
        {
            Assert.Equal(ConsoleRegion.Pal, CoreWithCountry(0x02).Region);
            Assert.Equal(ConsoleRegion.Ntsc, CoreWithCountry(0x01).Region);
        }

        // 21477272 / (262 * 1364).
        [Fact]
        public void Ntsc_runs_at_roughly_60hz()
        {
            Assert.Equal(60.098, CoreWithCountry(0x01).FrameRateHz, 3);
        }

        // 21281370 / (312 * 1364).
        [Fact]
        public void Pal_runs_at_roughly_50hz()
        {
            Assert.Equal(50.007, CoreWithCountry(0x02).FrameRateHz, 3);
        }

        [Fact]
        public void Stat78_reports_ntsc_with_bit_4_clear()
        {
            var core = CoreWithCountry(0x01);
            Assert.Equal(0x00, core.Bus!.Ppu.ReadRegister(Stat78) & 0x10);
        }

        [Fact]
        public void Stat78_reports_pal_with_bit_4_set()
        {
            var core = CoreWithCountry(0x02);
            Assert.Equal(0x10, core.Bus!.Ppu.ReadRegister(Stat78) & 0x10);
        }

        // The region bit must not disturb the PPU2 version nibble sharing
        // the same byte - a game reading STAT78 checks both.
        [Fact]
        public void The_region_bit_leaves_the_ppu2_version_intact()
        {
            Assert.Equal(0x01, CoreWithCountry(0x02).Bus!.Ppu.ReadRegister(Stat78) & 0x0F);
        }

        // A PAL frame is 312 scanlines against NTSC's 262, so the same
        // wall-clock-independent frame costs proportionally more CPU time.
        [Fact]
        public void A_pal_frame_spans_more_scanlines_than_an_ntsc_one()
        {
            var pal = CoreWithCountry(0x02);
            var ntsc = CoreWithCountry(0x01);

            pal.RunFrame();
            ntsc.RunFrame();

            Assert.True(pal.Bus!.Ppu.IsPal);
            Assert.False(ntsc.Bus!.Ppu.IsPal);
            Assert.True(pal.FrameRateHz < ntsc.FrameRateHz);
        }
    }
}
