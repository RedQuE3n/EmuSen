using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using Sa1Chip = EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1.Sa1;

namespace EmuSen.WiseMan.Coprocessors
{
    // The SA-1's two memory maps: the super MMC's ROM banking, the BW-RAM
    // window each side gets its own select for, and the shared I-RAM. See
    // Venus_SA1.md §3. Address decode is a function of the chip's registers
    // only, so these need no core and no ROM file.
    public class Sa1MemoryMapTests
    {
        private const int OneMegabyte = 0x100000;

        // 4MB of ROM, so all four super banks are distinct and addressable.
        private static Sa1Chip Chip(int bwRamSize = 0x8000) =>
            new(new byte[4 * OneMegabyte], new byte[bwRamSize]);

        private static (CartridgeRegion Region, int Offset) Scpu(Sa1Chip sa1, byte bank, ushort offset)
        {
            var r = sa1.ResolveScpu(bank, offset);
            return (r.Region, r.Offset);
        }

        private static (CartridgeRegion Region, int Offset) Side1(Sa1Chip sa1, byte bank, ushort offset)
        {
            var r = sa1.ResolveSa1(bank, offset);
            return (r.Region, r.Offset);
        }

        // --- Super MMC ROM banking ---

        [Theory]
        [InlineData(0x00, 0 * OneMegabyte)]
        [InlineData(0x20, 1 * OneMegabyte)]
        [InlineData(0x80, 2 * OneMegabyte)]
        [InlineData(0xA0, 3 * OneMegabyte)]
        public void Lorom_area_defaults_to_one_super_bank_per_slot(byte bank, int expected)
        {
            Assert.Equal((CartridgeRegion.Rom, expected), Scpu(Chip(), bank, 0x8000));
        }

        [Theory]
        [InlineData(0xC0, 0 * OneMegabyte)]
        [InlineData(0xD0, 1 * OneMegabyte)]
        [InlineData(0xE0, 2 * OneMegabyte)]
        [InlineData(0xF0, 3 * OneMegabyte)]
        public void Hirom_area_projects_the_same_four_super_banks(byte bank, int expected)
        {
            Assert.Equal((CartridgeRegion.Rom, expected), Scpu(Chip(), bank, 0x0000));
        }

        [Fact]
        public void Lorom_area_packs_banks_at_32kb_intervals()
        {
            Assert.Equal((CartridgeRegion.Rom, 0x8000), Scpu(Chip(), 0x01, 0x8000));
        }

        [Fact]
        public void Hirom_area_packs_banks_at_64kb_intervals()
        {
            Assert.Equal((CartridgeRegion.Rom, 0x10000), Scpu(Chip(), 0xC1, 0x0000));
        }

        [Fact]
        public void Reset_vector_still_lands_on_the_lorom_header_copy()
        {
            Assert.Equal((CartridgeRegion.Rom, 0x7FFC), Scpu(Chip(), 0x00, 0xFFFC));
        }

        // Bit 7 is what makes the register take effect at all - see Venus_SA1.md §3.1.
        [Fact]
        public void Bank_register_without_bit_seven_leaves_the_default_in_place()
        {
            var sa1 = Chip();
            sa1.WriteRegister(0x2220, 0x03);
            Assert.Equal((CartridgeRegion.Rom, 0), Scpu(sa1, 0x00, 0x8000));
        }

        [Fact]
        public void Bank_register_with_bit_seven_reprojects_the_slot()
        {
            var sa1 = Chip();
            sa1.WriteRegister(0x2220, 0x83);
            Assert.Equal((CartridgeRegion.Rom, 3 * OneMegabyte), Scpu(sa1, 0x00, 0x8000));
        }

        [Fact]
        public void Bank_register_also_reprojects_the_matching_hirom_slot()
        {
            var sa1 = Chip();
            sa1.WriteRegister(0x2223, 0x80);
            Assert.Equal((CartridgeRegion.Rom, 0), Scpu(sa1, 0xF0, 0x0000));
        }

        // --- BW-RAM ---

        [Fact]
        public void Banks_40_to_4f_are_flat_bw_ram()
        {
            Assert.Equal((CartridgeRegion.Sram, 0x1_0000), Scpu(Chip(), 0x41, 0x0000));
        }

        [Fact]
        public void Scpu_window_selects_an_8kb_block_with_bmaps()
        {
            var sa1 = Chip();
            sa1.WriteRegister(0x2224, 0x03);
            Assert.Equal((CartridgeRegion.Sram, 3 * 0x2000), Scpu(sa1, 0x00, 0x6000));
        }

        [Fact]
        public void Sa1_window_selects_its_own_block_with_bmap()
        {
            var sa1 = Chip();
            sa1.WriteRegister(0x2224, 0x03); // S-CPU side
            sa1.WriteRegister(0x2225, 0x01); // SA-1 side
            Assert.Equal((CartridgeRegion.Sram, 1 * 0x2000), Side1(sa1, 0x00, 0x6000));
        }

        [Fact]
        public void The_two_window_selects_are_independent()
        {
            var sa1 = Chip();
            sa1.WriteRegister(0x2224, 0x03);
            sa1.WriteRegister(0x2225, 0x01);
            Assert.NotEqual(Scpu(sa1, 0x00, 0x6000), Side1(sa1, 0x00, 0x6000));
        }

        // --- I-RAM ---

        [Fact]
        public void Both_sides_see_iram_at_3000()
        {
            var sa1 = Chip();
            Assert.Equal((CartridgeRegion.IRam, 0), Scpu(sa1, 0x00, 0x3000));
            Assert.Equal((CartridgeRegion.IRam, 0), Side1(sa1, 0x00, 0x3000));
        }

        // The SA-1 has no WRAM, so it gets I-RAM low as well and can put its
        // direct page and stack there - see Venus_SA1.md §3.
        [Fact]
        public void Only_the_sa1_side_sees_iram_in_page_zero()
        {
            var sa1 = Chip();
            Assert.Equal((CartridgeRegion.IRam, 0x100), Side1(sa1, 0x00, 0x0100));
            Assert.Equal(CartridgeRegion.Unmapped, Scpu(sa1, 0x00, 0x0100).Region);
        }

        [Fact]
        public void Iram_writes_are_visible_from_the_other_side()
        {
            var sa1 = Chip();
            sa1.WriteSa1(0x003100, 0x5A);
            Assert.Equal(0x5A, sa1.IRam[0x100]);
            Assert.Equal((CartridgeRegion.IRam, 0x100), Scpu(sa1, 0x00, 0x3100));
        }

        // --- Register file and mirroring ---

        [Fact]
        public void Register_file_decodes_across_the_mirrored_banks()
        {
            var sa1 = Chip();
            Assert.Equal((CartridgeRegion.CoprocessorRegister, 0x2200), Scpu(sa1, 0x00, 0x2200));
            Assert.Equal((CartridgeRegion.CoprocessorRegister, 0x2200), Scpu(sa1, 0xBF, 0x2200));
        }

        // The PPU and CPU register windows belong to MemoryBus, which decodes
        // them before the cartridge is ever consulted.
        [Fact]
        public void Ppu_register_window_is_not_claimed_by_the_sa1()
        {
            Assert.Equal(CartridgeRegion.Unmapped, Scpu(Chip(), 0x00, 0x2100).Region);
        }

        [Fact]
        public void Bw_ram_mirrors_when_the_cartridge_has_less_than_the_window_spans()
        {
            var sa1 = Chip(bwRamSize: 0x8000);
            sa1.WriteSa1(0x400000, 0x77);
            Assert.Equal(0x77, sa1.ReadSa1(0x408000));
        }
    }
}
