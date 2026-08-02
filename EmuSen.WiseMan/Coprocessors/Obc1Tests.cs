using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using Obc1Chip = EmuSen.Cores.Nintendo.Venus.Coprocessors.Obc1.Obc1;

namespace EmuSen.WiseMan.Coprocessors
{
    // The OBC1's whole job: turn four fixed ports into a moving window over a
    // sprite table held in the cartridge's SRAM. See Venus_OBC1.md §2. It owns
    // no memory, so the SRAM array is all these need.
    public class Obc1Tests
    {
        private const int SramSize = 0x2000;

        private static (Obc1Chip Chip, byte[] Sram) Chip()
        {
            byte[] sram = new byte[SramSize];
            return (new Obc1Chip(sram), sram);
        }

        // --- Address decode ---

        [Theory]
        [InlineData(0x00, 0x6000)]
        [InlineData(0x3F, 0x7FFF)]
        [InlineData(0x80, 0x6000)]
        [InlineData(0xBF, 0x7FF0)]
        public void The_chip_claims_6000_to_7fff_in_the_low_banks(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.CoprocessorRegister, Chip().Chip.ResolveScpu(bank, offset).Region);
        }

        [Theory]
        [InlineData(0x00, 0x5FFF)]
        [InlineData(0x00, 0x8000)]
        [InlineData(0x40, 0x6000)]
        [InlineData(0x70, 0x6000)]
        public void Everything_else_falls_through_to_the_cartridge(byte bank, ushort offset)
        {
            Assert.Equal(CartridgeRegion.Unmapped, Chip().Chip.ResolveScpu(bank, offset).Region);
        }

        [Fact]
        public void An_address_outside_the_port_range_is_plain_sram()
        {
            var (chip, sram) = Chip();

            chip.WriteRegister(0x0100, 0x5A);

            Assert.Equal(0x5A, sram[0x0100]);
            Assert.Equal(0x5A, chip.ReadRegister(0x0100));
        }

        // --- The moving window ---

        [Fact]
        public void The_four_low_ports_write_one_sprites_worth_of_the_low_table()
        {
            var (chip, sram) = Chip();
            chip.WriteRegister(0x1FF6, 0x02); // sprite index 2

            chip.WriteRegister(0x1FF0, 0x11);
            chip.WriteRegister(0x1FF1, 0x22);
            chip.WriteRegister(0x1FF2, 0x33);
            chip.WriteRegister(0x1FF3, 0x44);

            // Base $1C00 by default, four bytes per sprite.
            Assert.Equal(new byte[] { 0x11, 0x22, 0x33, 0x44 }, sram[0x1C08..0x1C0C]);
        }

        [Fact]
        public void Bumping_the_index_moves_the_window_four_bytes_on()
        {
            var (chip, sram) = Chip();

            chip.WriteRegister(0x1FF6, 0x00);
            chip.WriteRegister(0x1FF0, 0xAA);
            chip.WriteRegister(0x1FF6, 0x01);
            chip.WriteRegister(0x1FF0, 0xBB);

            Assert.Equal(0xAA, sram[0x1C00]);
            Assert.Equal(0xBB, sram[0x1C04]);
        }

        [Fact]
        public void Clearing_bit_0_of_the_base_port_moves_the_table_to_1800()
        {
            var (chip, sram) = Chip();

            chip.WriteRegister(0x1FF5, 0x01);
            chip.WriteRegister(0x1FF6, 0x00);
            chip.WriteRegister(0x1FF0, 0x99);

            Assert.Equal(0x99, sram[0x1800]);
        }

        [Fact]
        public void The_low_ports_read_back_through_the_same_window()
        {
            var (chip, sram) = Chip();
            sram[0x1C08] = 0x77;

            chip.WriteRegister(0x1FF6, 0x02);

            Assert.Equal(0x77, chip.ReadRegister(0x1FF0));
        }

        // --- The packed high table ---

        [Fact]
        public void The_high_port_packs_four_sprites_into_one_byte()
        {
            var (chip, sram) = Chip();

            // Indices 0-3 all share high-table byte $1E00, two bits each.
            for (byte index = 0; index < 4; index++)
            {
                chip.WriteRegister(0x1FF6, index);
                chip.WriteRegister(0x1FF4, 0x03);
            }

            Assert.Equal(0xFF, sram[0x1E00]);
        }

        [Fact]
        public void A_high_port_write_leaves_the_other_three_sprites_alone()
        {
            var (chip, sram) = Chip();
            sram[0x1E00] = 0xFF;

            chip.WriteRegister(0x1FF6, 0x01);
            chip.WriteRegister(0x1FF4, 0x00);

            // Only bits 3-2, sprite 1's field, were cleared.
            Assert.Equal(0xF3, sram[0x1E00]);
        }

        [Fact]
        public void Index_4_moves_on_to_the_next_high_table_byte()
        {
            var (chip, sram) = Chip();

            chip.WriteRegister(0x1FF6, 0x04);
            chip.WriteRegister(0x1FF4, 0x03);

            Assert.Equal(0x03, sram[0x1E01]);
            Assert.Equal(0x00, sram[0x1E00]);
        }

        [Fact]
        public void The_high_port_reads_the_whole_packed_byte_back()
        {
            var (chip, sram) = Chip();
            sram[0x1E00] = 0x5A;

            chip.WriteRegister(0x1FF6, 0x00);

            Assert.Equal(0x5A, chip.ReadRegister(0x1FF4));
        }
    }
}
