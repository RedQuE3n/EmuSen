using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Crystal.Systems.GameBoy;

namespace EmuSen.WiseMan.Cores
{
    // The boards the header selects, driven through MercuryRT's CPU bus - see Mercury_Memory.md §2, Mercury_Native.md §12.
    public sealed class MercuryRtCartridgeTests : IDisposable
    {
        private readonly List<MercuryRig> _rigs = new();


        public void Dispose()
        {
            foreach (MercuryRig rig in _rigs) rig.Dispose();
        }

        private MercuryRig Load(byte[] image)
        {
            var core = MercuryRig.Create(image);
            _rigs.Add(core);
            return core;
        }

        [Fact]
        public void Mbc1_banks_the_high_window_and_leaves_the_low_one_alone()
        {
            var image = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x01);
            image[0 * MercuryRig.RomBankSize + 0x20] = 0xA0;
            image[1 * MercuryRig.RomBankSize + 0x20] = 0xA1;
            image[3 * MercuryRig.RomBankSize + 0x20] = 0xA3;

            var core = Load(image);

            Assert.Equal(0xA0, core.Read(0x0020));
            Assert.Equal(0xA1, core.Read(0x4020));

            core.Write(0x2000, 0x03);
            Assert.Equal(0xA3, core.Read(0x4020));
            Assert.Equal(0xA0, core.Read(0x0020));
        }

        // Bank 0 through the high window is remapped to 1 on this board, which is the classic gotcha.
        [Fact]
        public void Mbc1_turns_a_requested_bank_zero_into_bank_one()
        {
            var image = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x01);
            image[1 * MercuryRig.RomBankSize + 0x20] = 0xA1;

            var core = Load(image);
            core.Write(0x2000, 0x00);

            Assert.Equal(0xA1, core.Read(0x4020));
        }

        // MBC5 is the one board here that can genuinely select bank 0.
        [Fact]
        public void Mbc5_can_select_bank_zero()
        {
            var image = SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x19);
            image[0 * MercuryRig.RomBankSize + 0x20] = 0xA0;
            image[1 * MercuryRig.RomBankSize + 0x20] = 0xA1;

            var core = Load(image);
            core.Write(0x2000, 0x00);

            Assert.Equal(0xA0, core.Read(0x4020));
        }

        [Fact]
        public void Cart_ram_reads_open_bus_until_it_is_enabled()
        {
            var core = Load(SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02));

            core.Write(0xA000, 0x42);
            Assert.Equal(0xFF, core.Read(0xA000));

            core.Write(0x0000, 0x0A);
            core.Write(0xA000, 0x42);
            Assert.Equal(0x42, core.Read(0xA000));
        }
    }

    // The header's facts as the shim reads them before the library sees the image - see Mercury_Memory.md §2.
    public sealed class MercuryCartridgeTests
    {
        [Fact]
        public void The_header_names_the_board_its_battery_and_its_ram()
        {
            var cart = GbCartridge.Read(SyntheticGbRom.Build(cartridgeType: 0x13, ramSizeCode: 0x03, title: "ZELDA"));

            Assert.Equal("MBC3", cart.Board);
            Assert.True(cart.HasBattery);
            Assert.False(cart.HasTimer);
            Assert.Equal(32 * 1024, cart.RamLength);
        }

        [Theory]
        [InlineData(0x00, false)]
        [InlineData(0x80, true)]
        [InlineData(0xC0, true)]
        public void The_colour_flag_is_read_from_its_own_byte(byte flag, bool colour)
        {
            Assert.Equal(colour, GbCartridge.Read(SyntheticGbRom.Build(cgbFlag: flag)).ColourGame);
        }

        [Fact]
        public void Mbc2_carries_its_own_ram_with_no_size_byte()
        {
            var cart = GbCartridge.Read(SyntheticGbRom.Build(cartridgeType: 0x06, ramSizeCode: 0x00));

            Assert.Equal("MBC2", cart.Board);
            Assert.Equal(512, cart.RamLength);
        }

        [Fact]
        public void An_image_too_short_to_hold_a_header_is_rejected()
        {
            Assert.Throws<System.IO.InvalidDataException>(() => GbCartridge.Read(new byte[0x100]));
        }

        [Fact]
        public void A_type_no_board_implements_is_refused_with_its_number()
        {
            var e = Assert.Throws<NotSupportedException>(() => GbCartridge.Read(SyntheticGbRom.Build(cartridgeType: 0x22)));
            Assert.Contains("$22", e.Message);
        }
    }
}
