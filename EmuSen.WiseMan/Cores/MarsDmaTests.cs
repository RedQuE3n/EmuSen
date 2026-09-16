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
            var bus = new MarsBus();
            bus.Write32(0x100, 0xCAFEBABE);

            bus.Write32(SpMem, 0);
            bus.Write32(SpDram, 0x100);
            bus.Write32(SpRead, 8 - 1);

            Assert.Equal(0xCAFEBABEu, bus.Read32(MemoryMap.SpDmemBase));
        }

        [Fact]
        public void The_address_bit_that_chooses_the_instruction_bank_is_honoured()
        {
            var bus = new MarsBus();
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
            var bus = new MarsBus();
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
            var bus = new MarsBus();
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
            var bus = new MarsBus();
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
            var bus = new MarsBus();

            Assert.Equal(0u, bus.Read32(MemoryMap.SpRegistersBase + SpInterface.DmaBusy));
            Assert.Equal(0u, bus.Read32(MemoryMap.SpRegistersBase + SpInterface.DmaFull));
        }

        [Fact]
        public void The_signal_processor_comes_up_halted()
        {
            var bus = new MarsBus();

            Assert.Equal(SpInterface.StatusHalt, bus.Read32(SpStatus) & SpInterface.StatusHalt);

            bus.Write32(SpStatus, 0x01);
            Assert.Equal(0u, bus.Read32(SpStatus) & SpInterface.StatusHalt);
        }

        [Fact]
        public void The_semaphore_is_taken_by_reading_it_and_released_by_writing_it()
        {
            var bus = new MarsBus();
            uint semaphore = MemoryMap.SpRegistersBase + SpInterface.Semaphore;

            Assert.Equal(0u, bus.Read32(semaphore));
            Assert.Equal(1u, bus.Read32(semaphore));

            bus.Write32(semaphore, 0);
            Assert.Equal(0u, bus.Read32(semaphore));
        }

        [Fact]
        public void The_cartridge_engine_copies_the_header_into_memory()
        {
            var bus = new MarsBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            bus.Write32(PiDram, 0x1000);
            bus.Write32(PiCart, MemoryMap.CartDomain1Address2);
            bus.Write32(PiWrite, 0x40 - 1);

            Assert.Equal(RomImage.Magic, bus.Read32(0x1000));
        }

        // The corpus spins on the busy flag between every word it prints; a stuck flag hangs it before it speaks.
        [Fact]
        public void The_cartridge_engine_reports_idle_so_a_polling_rom_makes_progress()
        {
            var bus = new MarsBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

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
            var bus = new MarsBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            bus.Write32(PiDram, 0x1000);
            bus.Write32(PiCart, MemoryMap.CartDomain1Address2);
            bus.Write32(PiWrite, 0x40 - 1);

            Assert.Equal(PiInterface.StatusInterrupt, bus.Read32(PiStatus) & PiInterface.StatusInterrupt);

            bus.Write32(PiStatus, 0x02);
            Assert.Equal(0u, bus.Read32(PiStatus) & PiInterface.StatusInterrupt);
        }
    }
}
