using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The header and the boards it selects - see Mercury_Memory.md §2.
    public class MercuryCartridgeTests
    {
        [Fact]
        public void The_header_names_the_game_and_its_board()
        {
            var cart = Cartridge.FromImage(SyntheticGbRom.Build(cartridgeType: 0x13, ramSizeCode: 0x03, title: "ZELDA"));

            Assert.Equal("ZELDA", cart.Title);
            Assert.Equal("MBC3", cart.Mapper.Name);
            Assert.True(cart.HasBattery);
            Assert.Equal(32 * 1024, cart.Ram.Length);
        }

        [Fact]
        public void The_header_checksum_is_computed_the_way_the_boot_rom_does()
        {
            var image = SyntheticGbRom.Build();
            Assert.True(Cartridge.FromImage(image).HeaderChecksumValid);

            // Any byte inside $0134-$014C is covered, so touching one must invalidate it.
            image[0x0140] ^= 0xFF;
            Assert.False(Cartridge.FromImage(image).HeaderChecksumValid);
        }

        [Theory]
        [InlineData(0x00, CgbSupport.None)]
        [InlineData(0x80, CgbSupport.Enhanced)]
        [InlineData(0xC0, CgbSupport.Required)]
        public void The_colour_flag_is_read_from_its_own_byte(byte flag, CgbSupport expected)
        {
            Assert.Equal(expected, Cartridge.FromImage(SyntheticGbRom.Build(cgbFlag: flag)).Cgb);
        }

        // A colour cart spends the last five title bytes on a manufacturer code.
        [Fact]
        public void A_colour_cart_reads_a_shorter_title()
        {
            var cart = Cartridge.FromImage(SyntheticGbRom.Build(cgbFlag: 0xC0, title: "ABCDEFGHIJK"));
            Assert.Equal(11, cart.Title.Length);
        }

        [Fact]
        public void Mbc2_carries_its_own_ram_with_no_size_byte()
        {
            var cart = Cartridge.FromImage(SyntheticGbRom.Build(cartridgeType: 0x06, ramSizeCode: 0x00));

            Assert.Equal("MBC2", cart.Mapper.Name);
            Assert.Equal(512, cart.Ram.Length);
        }

        [Fact]
        public void An_image_too_short_to_hold_a_header_is_rejected()
        {
            Assert.Throws<System.IO.InvalidDataException>(() => Cartridge.FromImage(new byte[0x100]));
        }

        [Fact]
        public void Mbc1_banks_the_high_window_and_leaves_the_low_one_alone()
        {
            var image = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x01);
            image[0 * Cartridge.RomBankSize + 0x20] = 0xA0;
            image[1 * Cartridge.RomBankSize + 0x20] = 0xA1;
            image[3 * Cartridge.RomBankSize + 0x20] = 0xA3;

            var mapper = Cartridge.FromImage(image).Mapper;

            Assert.Equal(0xA0, mapper.ReadRom(0x0020));
            Assert.Equal(0xA1, mapper.ReadRom(0x4020));

            mapper.WriteRom(0x2000, 0x03);
            Assert.Equal(0xA3, mapper.ReadRom(0x4020));
            Assert.Equal(0xA0, mapper.ReadRom(0x0020));
        }

        // Bank 0 through the high window is remapped to 1 on this board, which is the classic gotcha.
        [Fact]
        public void Mbc1_turns_a_requested_bank_zero_into_bank_one()
        {
            var image = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x01);
            image[1 * Cartridge.RomBankSize + 0x20] = 0xA1;

            var mapper = Cartridge.FromImage(image).Mapper;
            mapper.WriteRom(0x2000, 0x00);

            Assert.Equal(0xA1, mapper.ReadRom(0x4020));
        }

        // MBC5 is the one board here that can genuinely select bank 0.
        [Fact]
        public void Mbc5_can_select_bank_zero()
        {
            var image = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x19);
            image[0 * Cartridge.RomBankSize + 0x20] = 0xA0;
            image[1 * Cartridge.RomBankSize + 0x20] = 0xA1;

            var mapper = Cartridge.FromImage(image).Mapper;
            mapper.WriteRom(0x2000, 0x00);

            Assert.Equal(0xA0, mapper.ReadRom(0x4020));
        }

        [Fact]
        public void Cart_ram_reads_open_bus_until_it_is_enabled()
        {
            var mapper = Cartridge.FromImage(SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02)).Mapper;

            mapper.WriteRam(0xA000, 0x42);
            Assert.Equal(0xFF, mapper.ReadRam(0xA000));

            mapper.WriteRom(0x0000, 0x0A);
            mapper.WriteRam(0xA000, 0x42);
            Assert.Equal(0x42, mapper.ReadRam(0xA000));
        }
    }
}
