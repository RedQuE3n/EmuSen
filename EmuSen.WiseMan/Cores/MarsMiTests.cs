using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The MI's mode register and its repeat, and the RDRAM registers - see Mars_Memory.md §8.3-§8.5.
    public class MarsMiTests
    {
        private const uint Mode = MemoryMap.MiBase;

        [Fact]
        public void The_mode_register_reads_back_its_length_and_its_three_flags()
        {
            var bus = new MemoryBus();

            bus.Write32(Mode, 0x100 | 0x2A);
            Assert.Equal(0xAAu, bus.Read32(Mode));

            bus.Write32(Mode, 0x400 | 0x2000);
            Assert.Equal(0x380u, bus.Read32(Mode));

            bus.Write32(Mode, 0x080 | 0x200 | 0x1000 | 0x05);
            Assert.Equal(0x05u, bus.Read32(Mode));
        }

        [Fact]
        public void A_write_carrying_a_flags_set_and_clear_bits_sets_it()
        {
            var bus = new MemoryBus();

            bus.Write32(Mode, 0x180 | 0x600 | 0x3000);

            Assert.Equal(0x380u, bus.Read32(Mode));
        }

        [Fact]
        public void The_mode_register_still_clears_the_display_processors_interrupt()
        {
            var bus = new MemoryBus();
            bus.Mi.Raise(MiInterrupt.DisplayProcessor);

            bus.Write32(Mode, 0x800);

            Assert.Equal(MiInterrupt.None, bus.Mi.Pending);
            Assert.Equal(0u, bus.Read32(Mode));
        }

        [Fact]
        public void The_version_register_reads_the_rcps_revision()
        {
            Assert.Equal(0x0202_0102u, new MemoryBus().Read32(MemoryMap.MiBase + 4));
        }

        [Fact]
        public void A_word_store_is_repeated_across_the_length()
        {
            var bus = Filled(out _);
            Arm(bus, 12);

            bus.Store(0x100, 0x1234_5678_9ABC_DEF1UL, 4);

            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x100));
            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x104));
            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x108));
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x10C));
        }

        [Fact]
        public void The_length_counts_from_the_stores_doubleword()
        {
            var bus = Filled(out _);
            Arm(bus, 12);

            bus.Store(0x104, 0x9ABC_DEF1UL, 4);

            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x100));
            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x104));
            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x108));
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x10C));
        }

        [Fact]
        public void A_store_whose_misalignment_uses_up_the_length_writes_nothing()
        {
            var bus = Filled(out _);
            Arm(bus, 1);

            bus.Store(0x101, 0xF1UL, 1);

            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x100));
            Assert.False(bus.Mi.Repeating);
        }

        [Fact]
        public void A_byte_store_repeats_its_whole_bus_word_not_its_byte()
        {
            var bus = Filled(out _);
            Arm(bus, 6);

            bus.Store(0x101, 0x9ABC_DEF1UL, 1);

            Assert.Equal(0xFFF1_0000u, bus.Read32(0x100));
            Assert.Equal(0xDEF1_FFFFu, bus.Read32(0x104));
        }

        [Fact]
        public void A_halfword_store_repeats_its_whole_bus_word()
        {
            var bus = Filled(out _);
            Arm(bus, 8);

            bus.Store(0x102, 0x9ABC_DEF1UL, 2);

            Assert.Equal(0xFFFF_DEF1u, bus.Read32(0x100));
            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x104));
        }

        [Fact]
        public void A_doubleword_store_repeats_all_eight_of_its_bytes()
        {
            var bus = Filled(out _);
            Arm(bus, 12);

            bus.Store(0x200, 0x1234_5678_9ABC_DEF1UL, 8);

            Assert.Equal(0x1234_5678_9ABC_DEF1UL, bus.Read64(0x200));
            Assert.Equal(0x1234_5678u, bus.Read32(0x208));
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x20C));
        }

        [Fact]
        public void The_repeat_wraps_inside_its_two_kilobyte_row()
        {
            var bus = Filled(out _);
            Arm(bus, 128);

            bus.Store(0x7F8, 0UL, 8);

            Assert.Equal(0UL, bus.Read64(0x7F8));
            Assert.Equal(0UL, bus.Read64(0x000));
            Assert.Equal(0UL, bus.Read64(0x070));
            Assert.Equal(0xFFFF_FFFF_FFFF_FFFFUL, bus.Read64(0x078));
            Assert.Equal(0xFFFF_FFFF_FFFF_FFFFUL, bus.Read64(0x800));
        }

        [Fact]
        public void One_store_spends_the_repeat()
        {
            var bus = Filled(out _);
            Arm(bus, 16);

            bus.Store(0x100, 0x1111_1111UL, 4);
            bus.Store(0x300, 0x2222_2222UL, 4);

            Assert.Equal(0x1111_1111u, bus.Read32(0x10C));
            Assert.Equal(0x2222_2222u, bus.Read32(0x300));
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x304));
            Assert.Equal(0x0Fu, bus.Read32(Mode));
        }

        [Fact]
        public void A_store_to_the_rdram_registers_spends_the_repeat_without_repeating()
        {
            var bus = Filled(out _);
            Arm(bus, 16);

            bus.Store(MemoryMap.RdramRegistersBase + 4, 0x1111_1111UL, 4);
            bus.Store(0x100, 0x2222_2222UL, 4);

            Assert.Equal(0x2222_2222u, bus.Read32(0x100));
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x104));
        }

        [Fact]
        public void A_store_outside_rdram_leaves_the_repeat_armed()
        {
            var bus = Filled(out _);
            Arm(bus, 8);

            bus.Store(MemoryMap.SpDmemBase, 0x3333_3333UL, 4);
            bus.Store(0x100, 0x2222_2222UL, 4);

            Assert.Equal(0x3333_3333u, bus.Read32(MemoryMap.SpDmemBase));
            Assert.Equal(0x2222_2222u, bus.Read32(0x104));
        }

        [Fact]
        public void A_transfer_into_rdram_neither_repeats_nor_spends_the_repeat()
        {
            var bus = Filled(out _);
            Arm(bus, 8);

            bus.Write32(0x100, 0x4444_4444);

            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x104));
            Assert.True(bus.Mi.Repeating);
        }

        [Theory]
        [InlineData(0x00u, 0xB419_0010u)]
        [InlineData(0x04u, 0u)]
        [InlineData(0x08u, 0x2B3B_1A0Bu)]
        [InlineData(0x0Cu, 0u)]
        [InlineData(0x18u, 0x101C_0A04u)]
        [InlineData(0x24u, 0u)]
        [InlineData(0x3Cu, 0u)]
        [InlineData(0x40u, 0xB419_0010u)]
        [InlineData(0x1C8u, 0x2B3B_1A0Bu)]
        [InlineData(0x1D8u, 0x101C_0A04u)]
        public void The_rdram_registers_read_what_the_corpus_measured_every_64_bytes(uint offset, uint expected)
        {
            Assert.Equal(expected, new MemoryBus().Read32(MemoryMap.RdramRegistersBase + offset));
        }

        [Fact]
        public void A_doubleword_load_of_the_first_rdram_register_carries_the_second()
        {
            Assert.Equal(0xB419_0010_0000_0000UL, new MemoryBus().Load(MemoryMap.RdramRegistersBase, 8));
        }

        [Fact]
        public void A_write_to_the_rdram_registers_is_not_kept()
        {
            var bus = new MemoryBus();

            bus.Write32(MemoryMap.RdramRegistersBase, 0x1234_5678);
            bus.Write32(MemoryMap.RdramRegistersBase + 4, 0x1234_5678);

            Assert.Equal(0xB419_0010u, bus.Read32(MemoryMap.RdramRegistersBase));
            Assert.Equal(0u, bus.Read32(MemoryMap.RdramRegistersBase + 4));
        }

        // The processor's own store, which takes RDRAM directly unless the repeat is armed - see Mars_Performance.md §17.
        [Fact]
        public void A_store_from_the_processor_is_repeated_too()
        {
            var bus = Filled(out _);
            Arm(bus, 12);

            var cpu = new MipsAssembler().Lui(1, 0x8000).Ori(1, 1, 0x100).Lui(2, 0x9ABC).Ori(2, 2, 0xDEF1).Sw(2, 1, 0).Build(bus);
            cpu.Run(5);

            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x100));
            Assert.Equal(0x9ABC_DEF1u, bus.Read32(0x108));
            Assert.Equal(0xFFFF_FFFFu, bus.Read32(0x10C));
            Assert.False(bus.Mi.Repeating);
        }

        private static void Arm(MemoryBus bus, int length) => bus.Write32(Mode, 0x100 | (uint)(length - 1));

        private static MemoryBus Filled(out MemoryBus bus)
        {
            bus = new MemoryBus();
            for (uint at = 0; at < 0x1000; at += 4) bus.Write32(at, 0xFFFF_FFFF);
            return bus;
        }
    }
}
