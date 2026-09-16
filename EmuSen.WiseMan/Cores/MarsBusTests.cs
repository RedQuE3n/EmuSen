using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Phase A's memory map, with no CPU to drive it yet - see Mars_Memory.md.
    public class MarsBusTests
    {
        [Theory]
        [InlineData(0x8000_0000UL, 0x0000_0000u)]
        [InlineData(0xA000_0000UL, 0x0000_0000u)]
        [InlineData(0xA400_0000UL, 0x0400_0000u)]
        [InlineData(0xB3FF_0014UL, 0x13FF_0014u)]
        public void The_two_untranslated_segments_strip_the_top_bits(ulong virtualAddress, uint physical)
        {
            Assert.True(MemoryMap.TryTranslateDirect(virtualAddress, out uint translated));
            Assert.Equal(physical, translated);
        }

        [Theory]
        [InlineData(0x0000_0000UL)]
        [InlineData(0xC000_0000UL)]
        [InlineData(0xE000_0000UL)]
        public void Every_other_segment_needs_the_tlb(ulong virtualAddress)
        {
            Assert.False(MemoryMap.TryTranslateDirect(virtualAddress, out _));
        }

        [Fact]
        public void Only_the_uncached_segment_is_uncached()
        {
            Assert.True(MemoryMap.IsCached(0x8000_0000UL));
            Assert.False(MemoryMap.IsCached(0xA000_0000UL));
        }

        [Fact]
        public void Rdram_round_trips_at_every_width()
        {
            var bus = new MarsBus();

            bus.Write32(0x10, 0x11223344);
            Assert.Equal(0x11223344u, bus.Read32(0x10));
            Assert.Equal(0x11, bus.Read8(0x10));
            Assert.Equal(0x44, bus.Read8(0x13));
            Assert.Equal(0x1122, bus.Read16(0x10));
            Assert.Equal(0x3344, bus.Read16(0x12));

            bus.Write64(0x20, 0x0102030405060708UL);
            Assert.Equal(0x0102030405060708UL, bus.Read64(0x20));

            bus.Write8(0x11, 0xFF);
            Assert.Equal(0x11FF3344u, bus.Read32(0x10));

            bus.Write16(0x12, 0xBEEF);
            Assert.Equal(0x11FFBEEFu, bus.Read32(0x10));
        }

        // The first of the corpus bootstrap's three requirements: no mirroring above the installed size.
        [Fact]
        public void Rdram_above_the_installed_size_reads_zero_rather_than_mirroring()
        {
            var bus = new MarsBus();
            bus.Write32(0x10, 0x11223344);

            Assert.Equal(0u, bus.Read32(MarsBus.RdramSize + 0x10));
        }

        [Fact]
        public void An_expansion_pak_makes_the_upper_half_real_memory()
        {
            var bus = new MarsBus(expansionPak: true);

            bus.Write32(MarsBus.RdramSize + 0x10, 0x11223344);
            Assert.Equal(0x11223344u, bus.Read32(MarsBus.RdramSize + 0x10));
        }

        [Fact]
        public void The_two_signal_processor_memories_are_separate()
        {
            var bus = new MarsBus();

            bus.Write32(MemoryMap.SpDmemBase, 0xAAAAAAAA);
            bus.Write32(MemoryMap.SpImemBase, 0xBBBBBBBB);

            Assert.Equal(0xAAAAAAAAu, bus.Read32(MemoryMap.SpDmemBase));
            Assert.Equal(0xBBBBBBBBu, bus.Read32(MemoryMap.SpImemBase));
        }

        // The landmine: the corpus detects the port by reading back, so a write-only sink prints nothing.
        [Fact]
        public void The_debug_port_reads_back_what_was_written_to_it()
        {
            var bus = new MarsBus();

            bus.Write32(MemoryMap.IsViewerBase + IsViewer.BufferOffset, 0x12345678);

            Assert.Equal(0x12345678u, bus.Read32(MemoryMap.IsViewerBase + IsViewer.BufferOffset));
        }

        [Fact]
        public void A_length_write_emits_that_many_bytes_of_the_debug_buffer()
        {
            var bus = new MarsBus();
            uint buffer = MemoryMap.IsViewerBase + IsViewer.BufferOffset;

            bus.Write32(buffer, 0x50617373);
            bus.Write32(buffer + 4, 0x65640A00);
            bus.Write32(MemoryMap.IsViewerBase + IsViewer.LengthRegisterOffset, 7);

            Assert.Equal("Passed\n", bus.IsViewer.Text);
        }

        [Fact]
        public void Nothing_is_emitted_until_the_length_register_is_written()
        {
            var bus = new MarsBus();

            bus.Write32(MemoryMap.IsViewerBase + IsViewer.BufferOffset, 0x50617373);

            Assert.Equal("", bus.IsViewer.Text);
        }

        // The second requirement: a zero select register sends IPL3 into an RDRAM initialisation we do not model.
        [Fact]
        public void The_select_register_comes_up_nonzero()
        {
            var bus = new MarsBus();

            Assert.NotEqual(0u, bus.Read32(MemoryMap.RiSelect));
        }

        [Fact]
        public void The_cartridge_is_readable_at_its_domain_and_ignores_writes()
        {
            var bus = new MarsBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            Assert.Equal(RomImage.Magic, bus.Read32(MemoryMap.CartDomain1Address2));

            bus.Write32(MemoryMap.CartDomain1Address2, 0);
            Assert.Equal(RomImage.Magic, bus.Read32(MemoryMap.CartDomain1Address2));
        }

        [Fact]
        public void Reading_past_the_end_of_the_cartridge_is_zero()
        {
            var bus = new MarsBus { Cart = RomImage.FromImage(SyntheticN64Rom.Build()) };

            Assert.Equal(0u, bus.Read32(MemoryMap.CartDomain1Address2 + RomImage.MinimumLength));
        }

        // The counter no opcode has to remember, because no opcode touches it - see Mars_Memory.md §3.1.
        [Fact]
        public void The_count_register_is_derived_from_the_machine_clock_at_half_rate()
        {
            var bus = new MarsBus();

            Assert.Equal(0u, bus.Count);

            bus.Tick(2);
            Assert.Equal(1u, bus.Count);

            bus.Tick(8);
            Assert.Equal(5u, bus.Count);
        }

        [Fact]
        public void Writing_the_count_register_rebases_it_without_stopping_the_clock()
        {
            var bus = new MarsBus();
            bus.Tick(100);

            bus.SetCount(0);
            Assert.Equal(0u, bus.Count);

            bus.Tick(10);
            Assert.Equal(5u, bus.Count);
        }
    }
}
