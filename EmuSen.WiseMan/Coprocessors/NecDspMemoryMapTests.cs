using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Coprocessors
{
    // Which addresses the DSP claims, and which of its two registers each one
    // reaches - the map differs per board and per chip family. See Venus_NecDSP.md §3.
    public class NecDspMemoryMapTests
    {
        private static NecDsp Chip(NecDspVariant variant = NecDspVariant.Dsp1, bool hiRom = false)
        {
            NecDspProfile profile = NecDspProfile.For(variant);
            var firmware = NecDspFirmware.FromBlob(profile, new byte[profile.FirmwareBytes])!;
            return new NecDsp(variant, firmware, hiRom);
        }

        private static CartridgeRegion Region(NecDsp dsp, byte bank, ushort offset) => dsp.ResolveScpu(bank, offset).Region;

        // --- LoROM boards ---

        [Theory]
        [InlineData(0x30, 0x8000)]
        [InlineData(0x3F, 0xFFFF)]
        [InlineData(0xB0, 0x8000)]
        [InlineData(0xBF, 0xC000)]
        public void Lorom_claims_the_upper_half_of_banks_30_to_3f(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Region(Chip(), bank, offset));
        }

        [Theory]
        [InlineData(0x60, 0x0000)]
        [InlineData(0x6F, 0x7FFF)]
        [InlineData(0xEF, 0x0000)]
        public void Lorom_also_claims_the_lower_half_of_banks_60_to_6f(byte bank, ushort offset)
        {
            // Super Bases Loaded 2 reaches the chip through this window instead.
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Region(Chip(), bank, offset));
        }

        [Theory]
        [InlineData(0x30, 0x7FFF)]
        [InlineData(0x2F, 0x8000)]
        [InlineData(0x40, 0x8000)]
        [InlineData(0x60, 0x8000)]
        public void Lorom_leaves_everything_else_to_the_cartridge(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.Unmapped, Region(Chip(), bank, offset));
        }

        [Fact]
        public void Lorom_selects_dr_below_bit_14_and_sr_above_it()
        {
            var dsp = Chip();
            dsp.WriteRegister(0x308000, 0x99);
            dsp.WriteRegister(0x308000, 0x00);

            // $30:8000 reached DR; $30:C000 reads SR, whose RQM bit is clear.
            Assert.Equal(0x00, dsp.ReadRegister(0x30C000));
            Assert.Equal(0x99, dsp.ReadRegister(0x308000));
        }

        // --- HiROM boards ---

        [Theory]
        [InlineData(0x00, 0x6000)]
        [InlineData(0x1F, 0x7FFF)]
        [InlineData(0x80, 0x6000)]
        [InlineData(0x9F, 0x7000)]
        public void Hirom_claims_a_6000_window_below_the_cartridges_own_sram_banks(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Region(Chip(hiRom: true), bank, offset));
        }

        [Theory]
        [InlineData(0x20, 0x6000)]
        [InlineData(0x00, 0x5FFF)]
        [InlineData(0xA0, 0x6000)]
        public void Hirom_leaves_the_sram_banks_alone(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.Unmapped, Region(Chip(hiRom: true), bank, offset));
        }

        [Fact]
        public void Hirom_selects_dr_below_bit_12_and_sr_above_it()
        {
            var dsp = Chip(hiRom: true);
            dsp.WriteRegister(0x006000, 0x77);
            dsp.WriteRegister(0x006000, 0x00);

            Assert.Equal(0x00, dsp.ReadRegister(0x007000));
            Assert.Equal(0x77, dsp.ReadRegister(0x006000));
        }

        // --- ST010 / ST011 ---

        [Theory]
        [InlineData(0x60, 0x0000)]
        [InlineData(0xE0, 0x0FFF)]
        [InlineData(0x68, 0x0000)]
        [InlineData(0x6F, 0x0FFF)]
        [InlineData(0xEF, 0x0800)]
        public void St010_claims_its_register_bank_and_its_ram_banks(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Region(Chip(NecDspVariant.St010), bank, offset));
        }

        [Theory]
        [InlineData(0x60, 0x1000)]
        [InlineData(0x61, 0x0000)]
        [InlineData(0x70, 0x0000)]
        public void St010_claims_nothing_outside_those_banks(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.Unmapped, Region(Chip(NecDspVariant.St010), bank, offset));
        }

        [Fact]
        public void St010_banks_68_and_up_are_the_dsps_data_ram_as_byte_pairs()
        {
            var dsp = Chip(NecDspVariant.St010);

            dsp.WriteRegister(0x680004, 0x34);
            dsp.WriteRegister(0x680005, 0x12);

            Assert.Equal(0x1234, dsp.Ram[2]);
            Assert.Equal(0x34, dsp.ReadRegister(0x680004));
            Assert.Equal(0x12, dsp.ReadRegister(0x680005));
        }

        [Fact]
        public void St010_uses_the_low_address_bit_to_pick_dr_from_sr()
        {
            var dsp = Chip(NecDspVariant.St010);
            dsp.WriteRegister(0x600000, 0x42);
            dsp.WriteRegister(0x600000, 0x00);

            Assert.Equal(0x00, dsp.ReadRegister(0x600001));
            Assert.Equal(0x42, dsp.ReadRegister(0x600000));
        }

        // --- Chip geometry ---

        [Theory]
        [InlineData(NecDspVariant.Dsp1, 0x100, 0x1800, 0x800)]
        [InlineData(NecDspVariant.Dsp4, 0x100, 0x1800, 0x800)]
        [InlineData(NecDspVariant.St010, 0x800, 0xC000, 0x1000)]
        [InlineData(NecDspVariant.St011, 0x800, 0xC000, 0x1000)]
        public void Each_variant_declares_the_right_memory_sizes(NecDspVariant variant, int ramWords, int programBytes, int dataRomBytes)
        {
            NecDspProfile profile = NecDspProfile.For(variant);

            Assert.Equal(ramWords, profile.RamWords);
            Assert.Equal(programBytes, profile.ProgramBytes);
            Assert.Equal(dataRomBytes, profile.DataRomBytes);
            Assert.Equal(ramWords, Chip(variant).Ram.Length);
        }

        [Fact]
        public void Only_the_st01x_pair_carries_battery_backed_ram()
        {
            Assert.True(Chip(NecDspVariant.St010).HasBatteryRam);
            Assert.True(Chip(NecDspVariant.St011).HasBatteryRam);
            Assert.False(Chip(NecDspVariant.Dsp1).HasBatteryRam);
            Assert.False(Chip(NecDspVariant.Dsp1B).HasBatteryRam);
        }
    }
}
