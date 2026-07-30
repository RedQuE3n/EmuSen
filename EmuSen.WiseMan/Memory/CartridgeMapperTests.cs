using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;

namespace EmuSen.WiseMan.Memory
{
    // The LoROM/HiROM address maps, and the header scoring that picks
    // between them - see Venus_Memory.md §2.1a. Mappers are pure functions
    // of (bank, offset), so these need no ROM, no core, and no file.
    public class CartridgeMapperTests
    {
        private static (CartridgeRegion Region, int Offset) Map(ICartridgeMapper m, byte bank, ushort offset)
        {
            var r = m.Resolve(bank, offset);
            return (r.Region, r.Offset);
        }

        // --- LoROM ---

        [Fact]
        public void LoRom_maps_upper_half_of_bank_zero_to_rom_start()
        {
            Assert.Equal((CartridgeRegion.Rom, 0), Map(new LoRomMapper(), 0x00, 0x8000));
        }

        [Fact]
        public void LoRom_mirrors_bank_80_onto_bank_00()
        {
            var m = new LoRomMapper();
            Assert.Equal(Map(m, 0x00, 0x8000), Map(m, 0x80, 0x8000));
        }

        [Fact]
        public void LoRom_reset_vector_reads_from_7FFC()
        {
            Assert.Equal((CartridgeRegion.Rom, 0x7FFC), Map(new LoRomMapper(), 0x00, 0xFFFC));
        }

        [Fact]
        public void LoRom_maps_sram_banks()
        {
            Assert.Equal((CartridgeRegion.Sram, 0), Map(new LoRomMapper(), 0x70, 0x0000));
        }

        [Fact]
        public void LoRom_leaves_low_half_of_hardware_banks_unmapped()
        {
            Assert.Equal((CartridgeRegion.Unmapped, 0), Map(new LoRomMapper(), 0x00, 0x2000));
        }

        // --- HiROM ---

        // The exact failure that stopped Donkey Kong Country booting: under
        // LoROM this address resolves to 0x7FFC, which is game data on a
        // HiROM image, so the CPU reset to a garbage PC.
        [Fact]
        public void HiRom_reset_vector_reads_from_FFFC_not_7FFC()
        {
            Assert.Equal((CartridgeRegion.Rom, 0xFFFC), Map(new HiRomMapper(), 0x00, 0xFFFC));
        }

        [Fact]
        public void HiRom_maps_full_banks_from_C0()
        {
            var m = new HiRomMapper();
            Assert.Equal((CartridgeRegion.Rom, 0x000000), Map(m, 0xC0, 0x0000));
            Assert.Equal((CartridgeRegion.Rom, 0x010000), Map(m, 0xC1, 0x0000));
            Assert.Equal((CartridgeRegion.Rom, 0x3FFFFF), Map(m, 0xFF, 0xFFFF));
        }

        // $00-$3F:$8000-$FFFF is literally the top half of the same-numbered
        // $C0 bank, so both views must resolve to one ROM offset.
        [Fact]
        public void HiRom_low_bank_window_is_the_top_half_of_the_full_bank()
        {
            var m = new HiRomMapper();
            Assert.Equal(Map(m, 0xC1, 0x9000), Map(m, 0x01, 0x9000));
        }

        [Fact]
        public void HiRom_mirrors_banks_40_to_7D_onto_C0()
        {
            var m = new HiRomMapper();
            Assert.Equal(Map(m, 0xC0, 0x1234), Map(m, 0x40, 0x1234));
        }

        [Fact]
        public void HiRom_maps_sram_window_in_low_banks()
        {
            var m = new HiRomMapper();
            Assert.Equal((CartridgeRegion.Sram, 0), Map(m, 0x20, 0x6000));
            Assert.Equal((CartridgeRegion.Sram, 0x1FFF), Map(m, 0x20, 0x7FFF));
        }

        // Below $6000 in those same banks is hardware/open bus, not SRAM.
        [Fact]
        public void HiRom_leaves_registers_below_the_sram_window_unmapped()
        {
            Assert.Equal((CartridgeRegion.Unmapped, 0), Map(new HiRomMapper(), 0x20, 0x2100));
        }

        // --- Header detection ---

        private static string WriteRom(int sizeBytes, int headerBase, byte mapMode)
        {
            byte[] rom = new byte[sizeBytes];
            for (int i = 0; i < 21; i++) rom[headerBase + i] = (byte)'A';
            rom[headerBase + 0x15] = mapMode;
            rom[headerBase + 0x18] = 0x01; // 2KB SRAM
            // Checksum/complement must sum to 0xFFFF for the header to score.
            rom[headerBase + 0x1C] = 0x34; rom[headerBase + 0x1D] = 0x12;
            rom[headerBase + 0x1E] = 0xCB; rom[headerBase + 0x1F] = 0xED;

            string path = Path.Combine(Path.GetTempPath(), $"wiseman_map_{Guid.NewGuid():N}.sfc");
            File.WriteAllBytes(path, rom);
            return path;
        }

        [Fact]
        public void Detects_lorom_from_its_header()
        {
            Assert.Equal("LoROM", new Cartridge(WriteRom(64 * 1024, 0x7FC0, 0x20)).MapperName);
        }

        [Fact]
        public void Detects_hirom_from_its_header()
        {
            Assert.Equal("HiROM", new Cartridge(WriteRom(64 * 1024, 0xFFC0, 0x21)).MapperName);
        }
    }
}
