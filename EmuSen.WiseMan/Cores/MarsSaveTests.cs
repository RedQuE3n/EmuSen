using System;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The cartridge's save chip, the Controller Pak, and how the chip is named - see Mars_Save.md.
    public class MarsSaveTests
    {
        private const uint Dram = 0x0010_0000;
        private const uint SaveBase = MemoryMap.CartDomain2Address2;
        private const uint FlashCommand = SaveBase + 0x1_0000;

        // ---- EEPROM, on the joybus's fifth channel ----

        [Theory]
        [InlineData(N64SaveType.Eeprom4k, 0x80)]
        [InlineData(N64SaveType.Eeprom16k, 0xC0)]
        public void An_eeprom_names_its_size_in_the_info_reply(N64SaveType type, byte kind)
        {
            var bus = Cartridge(type);

            byte[] ram = RunCartridge(bus, 1, 3, 0x00);

            Assert.Equal(3, ram[5]);
            Assert.Equal(new byte[] { 0x00, kind, 0x00 }, ram[7..10]);
        }

        [Fact]
        public void A_cartridge_with_sram_does_not_answer_on_the_eeprom_channel()
        {
            byte[] ram = RunCartridge(Cartridge(N64SaveType.Sram256k), 1, 3, 0x00);

            Assert.Equal(Joybus.NoReply | 3, ram[5]);
        }

        [Fact]
        public void Asking_an_undecided_chip_what_it_is_describes_the_smaller_eeprom_and_decides_nothing()
        {
            var bus = Cartridge(N64SaveType.Unknown);

            byte[] ram = RunCartridge(bus, 1, 3, 0x00);

            Assert.Equal(new byte[] { 0x00, 0x80, 0x00 }, ram[7..10]);
            Assert.Equal(N64SaveType.Unknown, bus.Save.Type);
        }

        [Fact]
        public void Reading_an_undecided_chip_on_the_eeprom_channel_makes_it_an_eeprom()
        {
            var bus = Cartridge(N64SaveType.Unknown);

            byte[] ram = RunCartridge(bus, 2, 8, 0x04, 0x05);

            Assert.Equal(N64SaveType.Eeprom4k, bus.Save.Type);
            Assert.Equal(new byte[] { 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF }, ram[8..16]);
        }

        [Fact]
        public void A_written_block_reads_back_and_the_write_answers_zero()
        {
            var bus = Cartridge(N64SaveType.Eeprom4k);

            byte[] write = RunCartridge(bus, 10, 1, 0x05, 0x03, 1, 2, 3, 4, 5, 6, 7, 8);
            byte[] read = RunCartridge(bus, 2, 8, 0x04, 0x03);

            Assert.Equal(0x00, write[16]);
            Assert.Equal(new byte[] { 1, 2, 3, 4, 5, 6, 7, 8 }, read[8..16]);
            Assert.Equal(new byte[] { 1, 2, 3, 4 }, bus.Save.Eeprom!.Data[24..28]);
            Assert.True(bus.Save.Dirty);
        }

        [Fact]
        public void A_write_carrying_more_than_a_block_wraps_inside_it()
        {
            var bus = Cartridge(N64SaveType.Eeprom4k);

            RunCartridge(bus, 12, 1, 0x05, 0x01, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

            Assert.Equal(new byte[] { 9, 10, 3, 4, 5, 6, 7, 8, 0xFF }, bus.Save.Eeprom!.Data[8..17]);
        }

        [Fact]
        public void A_short_write_changes_only_the_bytes_it_carries()
        {
            var bus = Cartridge(N64SaveType.Eeprom4k);

            RunCartridge(bus, 5, 1, 0x05, 0x02, 1, 2, 3);

            Assert.Equal(new byte[] { 1, 2, 3, 0xFF }, bus.Save.Eeprom!.Data[16..20]);
        }

        [Fact]
        public void A_small_eeprom_still_reaches_two_kilobytes()
        {
            var bus = Cartridge(N64SaveType.Eeprom4k);

            RunCartridge(bus, 10, 1, 0x05, 0xFF, 1, 2, 3, 4, 5, 6, 7, 8);

            Assert.Equal(2048, bus.Save.Eeprom!.Data.Length);
            Assert.Equal(1, bus.Save.Eeprom.Data[0x7F8]);
        }

        [Fact]
        public void A_sixth_channel_reaches_the_cartridge_too()
        {
            var bus = Cartridge(N64SaveType.Eeprom16k);
            var ram = new byte[64];
            byte[] block = { 0, 0, 0, 0, 0, 1, 3, 0x00, 0, 0, 0, 0xFE };
            block.CopyTo(ram, 0);

            Joybus.Run(ram, bus.Si.Controllers, bus.Save);

            Assert.Equal(new byte[] { 0x00, 0xC0, 0x00 }, ram[8..11]);
        }

        // ---- SRAM, on the second domain ----

        [Fact]
        public void Sram_round_trips_through_a_transfer_each_way()
        {
            var bus = Cartridge(N64SaveType.Sram256k);
            for (uint i = 0; i < 16; i++) bus.Rdram[Dram + i] = (byte)(0xA0 + i);

            Transfer(bus, toCartridge: true, SaveBase + 0x100, 16);
            Array.Clear(bus.Rdram, (int)Dram, 16);
            Transfer(bus, toCartridge: false, SaveBase + 0x100, 16);

            Assert.Equal(0xA0, bus.Rdram[Dram]);
            Assert.Equal(0xAF, bus.Rdram[Dram + 15]);
            Assert.Equal(0xA4A5_A6A7u, bus.Read32(SaveBase + 0x104));
        }

        [Fact]
        public void Erased_sram_reads_all_ones()
        {
            Assert.Equal(0xFFFF_FFFFu, Cartridge(N64SaveType.Sram256k).Read32(SaveBase + 0x40));
        }

        [Fact]
        public void Sram_repeats_every_32_kilobytes()
        {
            var bus = Cartridge(N64SaveType.Sram256k);

            bus.Store(SaveBase + 0x10, 0x1234_5678UL, 4);

            Assert.Equal(0x1234_5678u, bus.Read32(SaveBase + 0x8010));
        }

        [Fact]
        public void Banked_sram_picks_its_bank_with_address_bits_18_and_19()
        {
            var bus = Cartridge(N64SaveType.SramBanked768k);

            bus.Store(SaveBase + 0x4_0010, 0x1111_1111UL, 4);
            bus.Store(SaveBase + 0x8_0010, 0x2222_2222UL, 4);

            Assert.Equal(0xFFFF_FFFFu, bus.Read32(SaveBase + 0x10));
            Assert.Equal(0x1111_1111u, bus.Read32(SaveBase + 0x4_0010));
            Assert.Equal(0x2222_2222u, bus.Read32(SaveBase + 0x8_0010));
            Assert.Equal(0x2222_2222u, bus.Save.Sram!.Data[0x1_0010] * 0x0101_0101u);
            Assert.Equal(0u, bus.Read32(SaveBase + 0xC_0010));
        }

        [Fact]
        public void A_byte_store_to_sram_writes_its_whole_bus_word()
        {
            var bus = Cartridge(N64SaveType.Sram256k);

            bus.Store(SaveBase + 0x21, 0x9ABC_DEF1UL, 1);

            Assert.Equal(0xDEF1_0000u, bus.Read32(SaveBase + 0x20));
        }

        [Fact]
        public void A_store_to_the_save_chip_holds_the_bus_but_a_read_of_it_takes_nothing_back()
        {
            var bus = Cartridge(N64SaveType.Sram256k);

            bus.Store(SaveBase + 0x30, 0x5555_5555UL, 4);
            bus.Store(SaveBase + 0x34, 0x6666_6666UL, 4);

            Assert.True(bus.Pi.IoBusy);
            Assert.Equal(0x6666_6666UL, bus.Load(SaveBase + 0x34, 4));
            Assert.True(bus.Pi.IoBusy);
        }

        [Fact]
        public void An_empty_second_domain_reads_the_low_half_of_its_address_twice()
        {
            Assert.Equal(0x1234_1234u, Cartridge(N64SaveType.None).Read32(SaveBase + 0x1234));
        }

        // ---- FlashRAM ----

        [Fact]
        public void Flash_identifies_itself_to_the_processor_and_to_a_transfer()
        {
            var bus = Cartridge(N64SaveType.FlashRam);

            bus.Store(FlashCommand, 0xE100_0000UL, 4);
            Transfer(bus, toCartridge: false, SaveBase, 8);

            Assert.Equal(0x1111_8001u, bus.Read32(SaveBase));
            Assert.Equal(0x00C2_001Du, bus.Read32(SaveBase + 4));
            Assert.Equal(new byte[] { 0x11, 0x11, 0x80, 0x01, 0x00, 0xC2, 0x00, 0x1D }, bus.Rdram[(int)Dram..(int)(Dram + 8)]);
        }

        [Fact]
        public void A_page_is_programmed_from_its_buffer_only_when_told_to_execute()
        {
            var bus = Cartridge(N64SaveType.FlashRam);
            for (uint i = 0; i < 128; i++) bus.Rdram[Dram + i] = (byte)i;

            bus.Store(FlashCommand, 0xB400_0000UL, 4);
            Transfer(bus, toCartridge: true, SaveBase, 128);
            bus.Store(FlashCommand, 0xA500_0003UL, 4);
            Assert.Equal(0xFF, bus.Save.Flash!.Data[3 * 128 + 5]);

            bus.Store(FlashCommand, 0xD200_0000UL, 4);

            Assert.Equal(5, bus.Save.Flash.Data[3 * 128 + 5]);
            Assert.Equal(0x1111_8004u, bus.Read32(SaveBase));
        }

        [Fact]
        public void An_erase_clears_one_page_not_a_sector()
        {
            var bus = Cartridge(N64SaveType.FlashRam, new byte[FlashRam.Size]);

            bus.Store(FlashCommand, 0x4B00_0003UL, 4);
            bus.Store(FlashCommand, 0x7800_0000UL, 4);
            bus.Store(FlashCommand, 0xD200_0000UL, 4);

            Assert.Equal(0xFF, bus.Save.Flash!.Data[3 * 128]);
            Assert.Equal(0xFF, bus.Save.Flash.Data[4 * 128 - 1]);
            Assert.Equal(0x00, bus.Save.Flash.Data[4 * 128]);
            Assert.Equal(0x1111_8008u, bus.Read32(SaveBase));
        }

        [Fact]
        public void Execute_with_nothing_pending_changes_nothing()
        {
            var bus = Cartridge(N64SaveType.FlashRam, new byte[FlashRam.Size]);

            bus.Store(FlashCommand, 0x4B00_0000UL, 4);
            bus.Store(FlashCommand, 0xD200_0000UL, 4);

            Assert.Equal(0x00, bus.Save.Flash!.Data[0]);
            Assert.False(bus.Save.Dirty);
        }

        [Fact]
        public void A_transfer_reads_the_array_only_in_read_mode()
        {
            var bus = Cartridge(N64SaveType.FlashRam);

            Transfer(bus, toCartridge: false, SaveBase + 0x80, 4);
            Assert.Equal(0u, bus.Read32(Dram));

            bus.Store(FlashCommand, 0xF000_0000UL, 4);
            Transfer(bus, toCartridge: false, SaveBase + 0x80, 4);
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(Dram));
            Assert.Equal(0x1111_8004u, bus.Read32(SaveBase));
            Assert.Equal(0xF000_001Du, bus.Read32(SaveBase + 4));
        }

        [Fact]
        public void A_page_number_reaches_past_255()
        {
            var bus = Cartridge(N64SaveType.FlashRam, new byte[FlashRam.Size]);

            bus.Store(FlashCommand, 0x4B00_0300UL, 4);
            bus.Store(FlashCommand, 0x7800_0000UL, 4);
            bus.Store(FlashCommand, 0xD200_0000UL, 4);

            Assert.Equal(0xFF, bus.Save.Flash!.Data[0x300 * 128]);
            Assert.Equal(0x00, bus.Save.Flash.Data[0]);
        }

        [Fact]
        public void The_page_buffer_wraps_every_128_bytes()
        {
            var bus = Cartridge(N64SaveType.FlashRam);
            for (uint i = 0; i < 256; i++) bus.Rdram[Dram + i] = (byte)i;

            bus.Store(FlashCommand, 0xB400_0000UL, 4);
            Transfer(bus, toCartridge: true, SaveBase, 256);
            bus.Store(FlashCommand, 0xA500_0001UL, 4);
            bus.Store(FlashCommand, 0xD200_0000UL, 4);

            Assert.Equal(128, bus.Save.Flash!.Data[128]);
            Assert.Equal(255, bus.Save.Flash.Data[255]);
        }

        [Fact]
        public void A_write_to_the_domains_first_word_is_not_a_command()
        {
            var bus = Cartridge(N64SaveType.FlashRam);

            bus.Store(SaveBase, 0xE100_0000UL, 4);

            Assert.Equal(0u, bus.Read32(SaveBase));
        }

        // ---- Naming the chip ----

        [Fact]
        public void The_first_processor_touch_of_an_undecided_second_domain_makes_it_flash()
        {
            var bus = Cartridge(N64SaveType.Unknown);

            bus.Store(FlashCommand, 0xE100_0000UL, 4);

            Assert.Equal(N64SaveType.FlashRam, bus.Save.Type);
        }

        [Fact]
        public void The_first_transfer_on_an_undecided_second_domain_makes_it_sram()
        {
            var bus = Cartridge(N64SaveType.Unknown);

            Transfer(bus, toCartridge: false, SaveBase, 8);

            Assert.Equal(N64SaveType.Sram256k, bus.Save.Type);
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(Dram));
        }

        [Fact]
        public void Once_named_sram_the_eeprom_channel_goes_quiet()
        {
            var bus = Cartridge(N64SaveType.Unknown);
            Transfer(bus, toCartridge: false, SaveBase, 8);

            byte[] ram = RunCartridge(bus, 1, 3, 0x00);

            Assert.Equal(Joybus.NoReply | 3, ram[5]);
        }

        [Fact]
        public void An_earlier_save_reaches_whichever_chip_the_game_turns_out_to_use()
        {
            var saved = new byte[Sram.BankSize];
            saved[0x40] = 0x77;
            var bus = new MemoryBus { Save = new SaveChip(N64SaveType.Unknown, saved) };
            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, Dram);

            Transfer(bus, toCartridge: false, SaveBase + 0x40, 2);

            Assert.Equal(0x77, bus.Rdram[Dram]);
        }

        [Theory]
        [InlineData(0x200, N64SaveType.Eeprom4k)]
        [InlineData(0x800, N64SaveType.Eeprom4k)]
        [InlineData(0x8000, N64SaveType.Sram256k)]
        [InlineData(0x1_8000, N64SaveType.SramBanked768k)]
        [InlineData(0x2_0000, N64SaveType.FlashRam)]
        [InlineData(0x1000, N64SaveType.Unknown)]
        public void An_earlier_save_names_its_chip_by_its_length(int length, N64SaveType expected)
        {
            Assert.Equal(expected, SaveChip.FromSaveLength(length));
        }

        [Fact]
        public void The_image_can_declare_its_own_chip()
        {
            var rom = RomImage.FromImage(SyntheticN64Rom.BuildHomebrew(N64SaveType.FlashRam));

            Assert.Equal(N64SaveType.FlashRam, SaveTypes.Declared(rom));
        }

        [Fact]
        public void A_title_in_the_table_is_named_by_its_two_checksums()
        {
            byte[] image = SyntheticN64Rom.Build();
            new byte[] { 0xB6, 0x95, 0x1A, 0x94, 0x63, 0xC8, 0x49, 0xAF }.CopyTo(image, 0x10);

            Assert.Equal(N64SaveType.Eeprom16k, SaveTypes.Declared(RomImage.FromImage(image)));
            Assert.Equal(N64SaveType.Unknown, SaveTypes.Declared(RomImage.FromImage(SyntheticN64Rom.Build())));
        }

        [Fact]
        public void The_table_holds_only_sixteen_kilobit_eeproms()
        {
            Assert.Equal(54, SaveTypes.Titles.Count);
            Assert.All(SaveTypes.Titles.Values, entry => Assert.Equal(N64SaveType.Eeprom16k, entry.Type));
        }

        // ---- The Controller Pak ----

        [Fact]
        public void The_data_crc_is_crc8_with_polynomial_0x85()
        {
            var counting = new byte[32];
            for (int i = 0; i < 32; i++) counting[i] = (byte)i;
            var ones = new byte[32];
            ones.AsSpan().Fill(0xFF);

            Assert.Equal(0x33, ControllerPak.DataCrc(counting));
            Assert.Equal(0x0A, ControllerPak.DataCrc(ones));
            Assert.Equal(0x00, ControllerPak.DataCrc(new byte[32]));
        }

        [Fact]
        public void A_new_pak_is_formatted_with_checksummed_ids_and_an_empty_index()
        {
            var pak = new ControllerPak(null);

            Assert.Equal(new byte[] { 0x00, 0x01, 0x01, 0x00, 0x01, 0x01, 0xFE, 0xF1 }, pak.Data[0x38..0x40]);
            foreach (int copy in new[] { 3, 4, 6 }) Assert.Equal(pak.Data[0x20..0x40], pak.Data[(copy * 32)..(copy * 32 + 32)]);
            Assert.Equal(0x71, pak.Data[0x101]);
            Assert.Equal(new byte[] { 0x00, 0x03 }, pak.Data[0x10A..0x10C]);
            Assert.Equal(pak.Data[0x100..0x200], pak.Data[0x200..0x300]);
            Assert.Equal(0, pak.Data[0x300]);
        }

        [Fact]
        public void A_pak_write_answers_its_crc_and_a_read_returns_the_chunk_and_its_crc()
        {
            var bus = new MemoryBus();
            bus.Si.Controllers[0].Pak = new ControllerPak(null);
            var chunk = new byte[32];
            for (int i = 0; i < 32; i++) chunk[i] = (byte)i;

            var write = new byte[64];
            new byte[] { 35, 1, 0x03, 0x04, 0x00 }.CopyTo(write, 0);
            chunk.CopyTo(write, 5);
            write[38] = 0xFE;
            Joybus.Run(write, bus.Si.Controllers, bus.Save);

            var read = new byte[64];
            new byte[] { 3, 33, 0x02, 0x04, 0x1F, 0xFE }.CopyTo(read, 0);
            Joybus.Run(read, bus.Si.Controllers, bus.Save);

            Assert.Equal(0x33, write[37]);
            Assert.Equal(chunk, read[5..37]);
            Assert.Equal(0x33, read[37]);
            Assert.True(bus.Si.Controllers[0].Pak!.Dirty);
        }

        [Fact]
        public void Above_the_pak_reads_zero_and_keeps_nothing()
        {
            var pak = new ControllerPak(null);
            var into = new byte[32];
            into.AsSpan().Fill(0xAA);

            pak.Write(0x8020, new byte[32]);
            pak.Read(0x8020, into);

            Assert.Equal(new byte[32], into);
            Assert.False(pak.Dirty);
        }

        private static MemoryBus Cartridge(N64SaveType type, byte[]? saved = null)
        {
            var bus = new MemoryBus { Save = new SaveChip(type, saved) };
            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, Dram);
            return bus;
        }

        // A block that skips the four ports with zero lengths and puts one command on the fifth channel.
        private static byte[] RunCartridge(MemoryBus bus, byte send, byte receive, params byte[] command)
        {
            var ram = new byte[64];
            ram[4] = send;
            ram[5] = receive;
            command.CopyTo(ram, 6);
            ram[6 + send + receive] = Joybus.End;

            Joybus.Run(ram, bus.Si.Controllers, bus.Save);
            return ram;
        }

        private static void Transfer(MemoryBus bus, bool toCartridge, uint cart, uint length)
        {
            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, Dram);
            bus.Write32(MemoryMap.PiBase + PiInterface.CartAddress, cart);
            bus.Write32(MemoryMap.PiBase + (toCartridge ? PiInterface.ReadLength : PiInterface.WriteLength), length - 1);
        }
    }
}
