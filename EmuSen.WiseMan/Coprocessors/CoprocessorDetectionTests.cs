using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Fixtures.NecDspFirmwareBuilder;

namespace EmuSen.WiseMan.Coprocessors
{
    // What Cartridge builds from a header, and how the DSP's firmware reaches
    // it. See Venus_NecDSP.md §1/§2 and Venus_OBC1.md §1.
    public class CoprocessorDetectionTests
    {
        private static byte[] Firmware() => NecDspFirmwareBuilder.Blob(NecDspFirmwareBuilder.Dsp1(Ld(0x1234, DestDr)));

        private static EmuSen.Cores.Nintendo.Venus.Memory.Cartridge Load(byte[] rom) => SyntheticRom.LoadCore(rom).Cart!;

        // --- Which chip ---
        //
        // Detection is a pure function of three header bytes, so these pin it
        // directly rather than through a loaded cartridge - whether the chip
        // then gets *built* depends on a firmware dump that may or may not be
        // on the machine running these tests.

        [Theory]
        [InlineData("PILOTWINGS", NecDspVariant.Dsp1)]
        [InlineData("DUNGEON MASTER", NecDspVariant.Dsp2)]
        [InlineData("TOP GEAR 3000", NecDspVariant.Dsp4)]
        [InlineData("PLANETS CHAMP TG3000", NecDspVariant.Dsp4)]
        [InlineData("SD\xB6\xDE\xDD\xC0\xDE\xD1GX", NecDspVariant.Dsp3)]
        public void The_title_picks_the_dsp_revision(string cartName, NecDspVariant expected)
        {
            Assert.Equal(expected, NecDspDetection.Detect(0x03, 0x00, cartName));
        }

        [Fact]
        public void An_unrecognised_dsp_title_falls_back_to_the_revised_dsp1b()
        {
            Assert.Equal(NecDspVariant.Dsp1B, NecDspDetection.Detect(0x03, 0x00, "SOME OTHER GAME"));
        }

        [Theory]
        [InlineData("F1 ROC II", NecDspVariant.St010)]
        [InlineData("2DAN MORITA SHOUGI", NecDspVariant.St011)]
        public void Cartridge_type_f3_with_chip_subtype_1_is_an_st01x(string cartName, NecDspVariant expected)
        {
            Assert.Equal(expected, NecDspDetection.Detect(0xF3, 0x01, cartName));
        }

        [Theory]
        [InlineData(0x00, 0x00)] // low nibble below 3 - no enhancement chip
        [InlineData(0x02, 0x00)]
        [InlineData(0x13, 0x00)] // SuperFX
        [InlineData(0x23, 0x00)] // OBC1
        [InlineData(0xF3, 0x02)] // $Fx, but subtype $02 is the ST018
        public void Nothing_else_is_mistaken_for_a_nec_dsp(byte cartType, byte chipType)
        {
            Assert.Null(NecDspDetection.Detect(cartType, chipType, "WHATEVER"));
        }

        [Fact]
        public void A_plain_lorom_builds_no_coprocessor_at_all()
        {
            var cart = Load(SyntheticRom.Build());

            Assert.Null(cart.NecDsp);
            Assert.Null(cart.Obc1);
            Assert.Null(cart.Sa1);
            Assert.Null(cart.SuperFx);
        }

        [Fact]
        public void Cartridge_type_25_builds_an_obc1()
        {
            var cart = Load(SyntheticRom.BuildObc1());

            Assert.NotNull(cart.Obc1);
            Assert.Equal("OBC1", cart.MapperName);
        }

        // --- Firmware ---

        [Fact]
        public void Firmware_appended_to_the_rom_file_is_found_and_run()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildNecDsp("PILOTWINGS", Firmware()));
            NecDsp dsp = core.Cart!.NecDsp!;

            dsp.Run(3);

            Assert.Equal(0x34, dsp.ReadRegister(0x308000));
            Assert.Equal(0x12, dsp.ReadRegister(0x308000));
        }

        [Fact]
        public void The_appended_firmware_is_trimmed_off_the_addressable_rom()
        {
            var cart = Load(SyntheticRom.BuildNecDsp("PILOTWINGS", Firmware()));

            // $01:8000 is ROM offset $8000, one byte past a 32KB ROM, so it
            // reads as open bus. Were the firmware still attached it would be
            // the first byte of the assembled program ($06) instead.
            Assert.Equal(0x00, cart.Read8(0x018000));
        }

        [Fact]
        public void A_dsp_cartridge_with_no_appended_firmware_still_boots()
        {
            // Whether a dump turns up in home/Firmware is a property of
            // the machine, not of this ROM - so what is pinned here is that
            // loading succeeds either way, and that the map agrees with
            // whether the chip actually got built.
            var cart = Load(SyntheticRom.BuildNecDsp("PILOTWINGS"));

            Assert.Equal(cart.NecDsp == null ? "LoROM" : "DSP1", cart.MapperName);
        }

        // --- The overlay ---

        [Fact]
        public void The_dsp_window_wins_over_the_rom_underneath_it()
        {
            var cart = Load(SyntheticRom.BuildNecDsp("PILOTWINGS", Firmware()));

            // $30:C000 would be ROM on a plain LoROM; here it reads SR.
            Assert.Equal("DSP1", cart.MapperName);
            Assert.Equal(0x00, cart.Read8(0x30C000));
        }

        [Fact]
        public void Everything_outside_the_window_still_maps_the_ordinary_way()
        {
            var cart = Load(SyntheticRom.BuildNecDsp("PILOTWINGS", Firmware()));

            Assert.Equal(0xEA, cart.Read8(0x009000));
            Assert.Equal(0xA9, cart.Read8(0x008000));
        }
    }
}
