using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The two transfer engines the corpus's bootstrap uses before any instruction of ours runs - see Mars_Memory.md §6.
    public class MarsDmaTests
    {
        private const uint SpMem = MemoryMap.SpRegistersBase + SpInterface.MemAddress;
        private const uint SpDram = MemoryMap.SpRegistersBase + SpInterface.DramAddress;
        private const uint SpRead = MemoryMap.SpRegistersBase + SpInterface.ReadLength;
        private const uint SpWrite = MemoryMap.SpRegistersBase + SpInterface.WriteLength;
        private const uint SpStatus = MemoryMap.SpRegistersBase + SpInterface.Status;

        private const uint PiDram = MemoryMap.PiBase + PiInterface.DramAddress;
        private const uint PiCart = MemoryMap.PiBase + PiInterface.CartAddress;
        private const uint PiWrite = MemoryMap.PiBase + PiInterface.WriteLength;
        private const uint PiStatus = MemoryMap.PiBase + PiInterface.Status;

        [Fact]
        public void A_transfer_moves_memory_into_the_data_bank()
        {
            var bus = new MemoryBus();
            bus.Write32(0x100, 0xCAFEBABE);

            bus.Write32(SpMem, 0);
            bus.Write32(SpDram, 0x100);
            bus.Write32(SpRead, 8 - 1);

            Assert.Equal(0xCAFEBABEu, bus.Read32(MemoryMap.SpDmemBase));
        }

        [Fact]
        public void The_address_bit_that_chooses_the_instruction_bank_is_honoured()
        {
            var bus = new MemoryBus();
            bus.Write32(0x100, 0xCAFEBABE);

            bus.Write32(SpMem, 0x1000);
            bus.Write32(SpDram, 0x100);
            bus.Write32(SpRead, 8 - 1);

            Assert.Equal(0xCAFEBABEu, bus.Read32(MemoryMap.SpImemBase));
            Assert.Equal(0u, bus.Read32(MemoryMap.SpDmemBase));
        }

        [Fact]
        public void A_transfer_runs_the_other_way_too()
        {
            var bus = new MemoryBus();
            bus.Write32(MemoryMap.SpDmemBase, 0x12345678);

            bus.Write32(SpMem, 0);
            bus.Write32(SpDram, 0x200);
            bus.Write32(SpWrite, 8 - 1);

            Assert.Equal(0x12345678u, bus.Read32(0x200));
        }

        // The third of the corpus bootstrap's requirements: past the end of a bank is its own start.
        [Fact]
        public void A_transfer_off_the_end_of_the_instruction_bank_wraps_inside_it()
        {
            var bus = new MemoryBus();
            for (uint i = 0; i < 0x20; i++) bus.Write8(0x300 + i, (byte)(0xA0 + i));

            bus.Write32(SpMem, 0x1000 | 0xFF8);
            bus.Write32(SpDram, 0x300);
            bus.Write32(SpRead, 0x10 - 1);

            Assert.Equal(0xA0, bus.Read8(MemoryMap.SpImemBase + 0xFF8));
            Assert.Equal(0xA8, bus.Read8(MemoryMap.SpImemBase));
            Assert.Equal(0xAF, bus.Read8(MemoryMap.SpImemBase + 7));

            // The bank next door is what a masked address would have spilled into.
            for (uint i = 0; i < 8; i++) Assert.Equal(0, bus.Read8(MemoryMap.SpDmemBase + i));
        }

        [Fact]
        public void A_rectangular_transfer_skips_between_rows()
        {
            var bus = new MemoryBus();
            for (uint i = 0; i < 0x20; i++) bus.Write8(0x400 + i, (byte)(0xB0 + i));

            bus.Write32(SpMem, 0);
            bus.Write32(SpDram, 0x400);

            // Two rows of eight bytes, eight bytes apart.
            bus.Write32(SpRead, (8 - 1) | (1u << 12) | (8u << 20));

            Assert.Equal(0xB0, bus.Read8(MemoryMap.SpDmemBase));
            Assert.Equal(0xC0, bus.Read8(MemoryMap.SpDmemBase + 8));
        }

        [Fact]
        public void The_transfer_engine_is_never_busy_because_it_has_already_finished()
        {
            var bus = new MemoryBus();

            Assert.Equal(0u, bus.Read32(MemoryMap.SpRegistersBase + SpInterface.DmaBusy));
            Assert.Equal(0u, bus.Read32(MemoryMap.SpRegistersBase + SpInterface.DmaFull));
        }

        [Fact]
        public void The_signal_processor_comes_up_halted()
        {
            var bus = new MemoryBus();

            Assert.Equal(SpInterface.StatusHalt, bus.Read32(SpStatus) & SpInterface.StatusHalt);

            bus.Write32(SpStatus, 0x01);
            Assert.Equal(0u, bus.Read32(SpStatus) & SpInterface.StatusHalt);
        }

        [Fact]
        public void The_semaphore_is_taken_by_reading_it_and_released_by_writing_it()
        {
            var bus = new MemoryBus();
            uint semaphore = MemoryMap.SpRegistersBase + SpInterface.Semaphore;

            Assert.Equal(0u, bus.Read32(semaphore));
            Assert.Equal(1u, bus.Read32(semaphore));

            bus.Write32(semaphore, 0);
            Assert.Equal(0u, bus.Read32(semaphore));
        }

        [Fact]
        public void The_cartridge_engine_copies_the_header_into_memory()
        {
            var bus = new MemoryBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            bus.Write32(PiDram, 0x1000);
            bus.Write32(PiCart, MemoryMap.CartDomain1Address2);
            bus.Write32(PiWrite, 0x40 - 1);

            Assert.Equal(RomImage.Magic, bus.Read32(0x1000));
        }

        // The corpus spins on the busy flag between every word it prints; a stuck flag hangs it before it speaks.
        [Fact]
        public void The_cartridge_engine_reports_idle_so_a_polling_rom_makes_progress()
        {
            var bus = new MemoryBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            Assert.Equal(0u, bus.Read32(PiStatus) & PiInterface.StatusIoBusy);
            Assert.Equal(0u, bus.Read32(PiStatus) & PiInterface.StatusDmaBusy);

            bus.Write32(PiDram, 0x1000);
            bus.Write32(PiCart, MemoryMap.CartDomain1Address2);
            bus.Write32(PiWrite, 0x40 - 1);

            Assert.Equal(0u, bus.Read32(PiStatus) & PiInterface.StatusIoBusy);
            Assert.Equal(0u, bus.Read32(PiStatus) & PiInterface.StatusDmaBusy);
        }

        [Fact]
        public void A_finished_transfer_raises_an_interrupt_flag_that_a_write_clears()
        {
            var bus = new MemoryBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            bus.Write32(PiDram, 0x1000);
            bus.Write32(PiCart, MemoryMap.CartDomain1Address2);
            bus.Write32(PiWrite, 0x40 - 1);

            Assert.Equal(PiInterface.StatusInterrupt, bus.Read32(PiStatus) & PiInterface.StatusInterrupt);

            bus.Write32(PiStatus, 0x02);
            Assert.Equal(0u, bus.Read32(PiStatus) & PiInterface.StatusInterrupt);
        }

        // Each side of a cartridge transfer advances by a multiple of its own bus width - see Mars_Memory.md §7.2.
        [Theory]
        [InlineData(1u, 2u, 8u)]
        [InlineData(2u, 2u, 8u)]
        [InlineData(3u, 4u, 8u)]
        [InlineData(7u, 8u, 8u)]
        [InlineData(8u, 8u, 8u)]
        [InlineData(9u, 10u, 16u)]
        [InlineData(0x7Cu, 0x7Cu, 0x80u)]
        [InlineData(0x7Du, 0x7Eu, 0x80u)]
        public void A_cartridge_transfer_advances_each_address_to_its_own_multiple(uint length, uint cart, uint dram)
        {
            var bus = new MemoryBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, 0x0010_0000);
            bus.Write32(MemoryMap.PiBase + PiInterface.CartAddress, 0x1000_0000);
            bus.Write32(MemoryMap.PiBase + PiInterface.WriteLength, length - 1);

            Assert.Equal(0x1000_0000 + cart, bus.Read32(MemoryMap.PiBase + PiInterface.CartAddress));
            Assert.Equal(0x0010_0000 + dram, bus.Read32(MemoryMap.PiBase + PiInterface.DramAddress));
        }

        // A misaligned transfer leaves the address on the block after the last byte it stored, not after its length - see §7.3.
        [Theory]
        [InlineData(10u, 0x10u)]
        [InlineData(9u, 0x10u)]
        [InlineData(7u, 0x08u)]
        [InlineData(1u, 0x08u)]
        public void A_misaligned_cartridge_transfer_still_lands_on_a_block(uint length, uint dram)
        {
            var bus = new MemoryBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, 0x0010_0006);
            bus.Write32(MemoryMap.PiBase + PiInterface.CartAddress, 0x1000_0000);
            bus.Write32(MemoryMap.PiBase + PiInterface.WriteLength, length - 1);

            Assert.Equal(0x0010_0000 + dram, bus.Read32(MemoryMap.PiBase + PiInterface.DramAddress));
        }

        // The corpus's short transfers six bytes into a block: the misalignment comes off what is stored, down to nothing - see §7.3.
        [Theory]
        [InlineData(1, 0)]
        [InlineData(2, 0)]
        [InlineData(6, 0)]
        [InlineData(7, 1)]
        [InlineData(8, 2)]
        [InlineData(16, 10)]
        [InlineData(119, 113)]
        [InlineData(120, 114)]
        public void A_short_misaligned_transfer_stores_its_misalignment_fewer_bytes(int length, int stored)
        {
            Assert.Equal(Expect((0, 0, stored)), Land(Block + 6, length));
        }

        // Only a first block one byte short of full stores its last pair whole - see §7.3.
        [Theory]
        [InlineData(0, 127, 128)]
        [InlineData(6, 121, 116)]
        public void A_first_block_one_byte_short_of_full_stores_its_last_pair_whole(int misaligned, int length, int stored)
        {
            Assert.Equal(Expect((0, 0, stored)), Land(Block + (uint)misaligned, length));
        }

        // The first block is short by its misalignment twice over, and what it skips is read and dropped - see §7.3.
        [Theory]
        [InlineData(122)]
        [InlineData(128)]
        [InlineData(284)]
        public void The_first_misaligned_block_leaves_a_gap_before_the_next(int length)
        {
            Assert.Equal(Expect((0, 0, 116), (122, 122, length - 122)), Land(Block + 6, length));
        }

        // Only the first block ends on a lone byte; a later one rounds an odd remainder up to a pair - see §7.3.
        [Theory]
        [InlineData(129, 130)]
        [InlineData(131, 132)]
        [InlineData(133, 134)]
        public void A_later_block_rounds_an_odd_remainder_up_to_a_pair(int length, int stored)
        {
            Assert.Equal(Expect((0, 0, stored)), Land(Block, length));
        }

        // A block stops at the end of a 2KB row, and the next one starts on the row - see §7.3.
        [Theory]
        [InlineData(50, 44, 0)]
        [InlineData(66, 52, 8)]
        [InlineData(284, 52, 226)]
        public void A_block_stops_at_the_end_of_a_row(int length, int first, int rest)
        {
            Assert.Equal(Expect((0, 0, first), (58, 58, rest)), Land(RowEnd - 58, length));
        }

        // A block that begins within eight bytes of a row's end makes the next short by its misalignment - see §7.3.
        [Fact]
        public void A_block_at_the_end_of_a_row_shortens_the_one_after_it()
        {
            Assert.Equal(Expect((0, 0, 4), (6, 6, 2)), Land(RowEnd - 6, 7));
            Assert.Equal(Expect((4, 4, 2)), Land(RowEnd - 4, 6));
            Assert.Equal(Expect((2, 2, 2)), Land(RowEnd - 2, 4));

            Assert.Equal(Expect((0, 0, 4), (6, 6, 126), (134, 132, 92)), Land(RowEnd - 6, 224));
            Assert.Equal(Expect((2, 2, 122), (130, 124, 100)), Land(RowEnd - 2, 224));
        }

        // Not measured: every row end the corpus uses is also a 1KB boundary, and this one is only that - see §7.6.
        [Fact]
        public void A_row_is_two_kilobytes_so_a_one_kilobyte_boundary_does_not_stop_a_block()
        {
            Assert.Equal(Expect((0, 0, 60)), Land(Block + 0x400 - 58, 66));
        }

        // Not measured: the corpus places nothing between eight and fifty-eight bytes from a row's end - see §7.6.
        [Fact]
        public void A_block_eight_or_more_bytes_from_the_end_of_a_row_leaves_the_next_one_whole()
        {
            Assert.Equal(Expect((0, 0, 12), (14, 14, 210)), Land(RowEnd - 14, 224));
        }

        // Not measured: a shortened block is the walk's own state, and the next transfer starts whole - see §7.6.
        [Fact]
        public void A_shortened_block_does_not_outlive_its_transfer()
        {
            var bus = CountingCartridge();
            Land(bus, RowEnd - 6, 6);

            Assert.Equal(Expect((0, 0, 128)), Land(bus, Block, 128));
        }

        // A dispute: the corpus's own formula stores 52 here, and no case it runs is this size - see §7.4.
        [Fact]
        public void A_first_block_one_byte_short_of_a_row_ends_on_a_lone_byte()
        {
            Assert.Equal(Expect((0, 0, 51)), Land(RowEnd - 58, 57));
        }

        private const uint Block = 0x0010_0000;
        private const uint RowEnd = 0x0010_0800;
        private const byte Paint = 0xAA;
        private const int Span = 0x180;

        // The corpus's arrangement: memory painted, the cartridge counting up, never through the paint value.
        private static byte Counting(int offset) => (byte)(offset % Paint);

        private static MemoryBus CountingCartridge()
        {
            var counting = new byte[0x400];
            for (int i = 0; i < counting.Length; i++) counting[i] = Counting(i);

            return new MemoryBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build(patches: (0, counting))) };
        }

        private static byte[] Land(uint dram, int length) => Land(CountingCartridge(), dram, length);

        private static byte[] Land(MemoryBus bus, uint dram, int length)
        {
            for (uint i = 0; i < Span; i++) bus.Write8(dram + i, Paint);

            bus.Write32(PiDram, dram);
            bus.Write32(PiCart, MemoryMap.CartDomain1Address2 + RomImage.HeaderLength);
            bus.Write32(PiWrite, (uint)length - 1);

            var landed = new byte[Span];
            for (uint i = 0; i < Span; i++) landed[i] = bus.Read8(dram + i);
            return landed;
        }

        // Runs of destination offset, cartridge offset and count; everything else is still paint.
        private static byte[] Expect(params (int At, int From, int Count)[] runs)
        {
            var expected = new byte[Span];
            System.Array.Fill(expected, Paint);

            foreach (var (at, from, count) in runs)
            {
                for (int i = 0; i < count; i++) expected[at + i] = Counting(from + i);
            }

            return expected;
        }

    }
}
