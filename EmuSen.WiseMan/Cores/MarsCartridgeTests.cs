using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The processor's own reads and stores on the cartridge bus, with the corpus's data and numbers - see Mars_Memory.md §7.7 and §7.8.
    public class MarsCartridgeTests
    {
        private const uint Data = MemoryMap.CartDomain1Address2 + RomImage.HeaderLength;
        private const uint Rom = MemoryMap.CartDomain1Address2;
        private const uint PiStatus = MemoryMap.PiBase + PiInterface.Status;

        private static readonly byte[] Bytes =
        {
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
            0x21, 0x43, 0x65, 0x87, 0x99, 0xBA, 0xDC, 0xFE,
            0xA9, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22,
        };

        private static MemoryBus Cartridge() => new() { Cart = RomImage.FromImage(SyntheticN64Rom.Build(patches: (0, Bytes))) };

        private static bool Busy(MemoryBus bus) => (bus.Read32(PiStatus) & PiInterface.StatusIoBusy) != 0;

        [Fact]
        public void A_word_read_reaches_every_word()
        {
            var bus = Cartridge();

            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
            Assert.Equal(0x89ABCDEFul, bus.Load(Data + 4, 4));
            Assert.Equal(0x21436587ul, bus.Load(Data + 8, 4));
        }

        // The corpus's own table: the word starts at the halfword named, so a read with bit 1 set lands two bytes on - see §7.8.
        [Fact]
        public void A_halfword_read_cannot_reach_every_other_halfword()
        {
            var bus = Cartridge();
            ushort[] expected = { 0x0123, 0x89AB, 0x89AB, 0x2143, 0x2143, 0x99BA, 0x99BA, 0xA988 };

            for (uint i = 0; i < expected.Length; i++) Assert.Equal(expected[i], bus.Load(Data + 2 * i, 2));
        }

        [Fact]
        public void A_byte_read_cannot_reach_every_other_halfword()
        {
            var bus = Cartridge();
            byte[] expected = { 0x01, 0x23, 0x89, 0xAB, 0x89, 0xAB, 0x21, 0x43, 0x21, 0x43, 0x99, 0xBA, 0x99, 0xBA, 0xA9, 0x88 };

            for (uint i = 0; i < expected.Length; i++) Assert.Equal(expected[i], bus.Load(Data + i, 1));
        }

        // A store is what the next read returns, once, wherever either of them is; a second store while busy is lost - see §7.7.
        [Theory]
        [InlineData(0u)]
        [InlineData(4u)]
        public void A_store_is_what_the_next_read_returns_once(uint offset)
        {
            var bus = Cartridge();

            bus.Store(Data + offset, 0xBADC0FFE, 4);
            bus.Store(Data, 0xDECAF, 4);

            Assert.Equal(0xBADC0FFEul, bus.Load(Data, 4));
            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
        }

        // The window runs from the cartridge's first byte to its last word before the PIF's ROM, which does not latch - see §7.7.
        [Theory]
        [InlineData(0x1000_0000u, 0xBADC0FFEu)]
        [InlineData(0x1FBF_FFFCu, 0xBADC0FFEu)]
        [InlineData(0x1FC0_0000u, 0x000DECAFu)]
        public void The_window_ends_where_the_pif_begins(uint physical, uint first)
        {
            var bus = Cartridge();

            bus.Store(physical, 0xBADC0FFE, 4);
            bus.Store(Data, 0xDECAF, 4);

            Assert.Equal(first, bus.Load(Data, 4));
            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
        }

        // A narrower store fills its lane of a whole word with zeroes around it, and a doubleword keeps its upper half - see §7.7.
        [Theory]
        [InlineData(0u, 0xBAul, 1, 0xBA000000u)]
        [InlineData(1u, 0x123456BAul, 1, 0x56BA0000u)]
        [InlineData(0u, 0xBADCul, 2, 0xBADC0000u)]
        [InlineData(0u, 0x98765432_1AF1231Aul, 8, 0x98765432u)]
        public void A_store_of_any_size_is_latched_as_a_whole_word(uint offset, ulong value, int size, uint word)
        {
            var bus = Cartridge();

            bus.Store(Rom + offset, value, size);
            bus.Store(Data, 0x01010101_23232323, 8);

            Assert.Equal(word, bus.Load(Data, 4));
        }

        // The corpus reads the stored word narrower only at its first lane - see §7.7.
        [Theory]
        [InlineData(1, 0xBAul)]
        [InlineData(2, 0xBADCul)]
        public void A_narrow_read_of_the_stored_word_starts_at_its_top(int size, ulong expected)
        {
            var bus = Cartridge();
            bus.Store(Rom, 0xBADC0FFE, 4);

            Assert.Equal(expected, bus.Load(Data, size));
        }

        // Not measured: every other lane follows the FPGA core, which returns the stored word whole for the processor to pick from - see §7.7.
        [Theory]
        [InlineData(1u, 1, 0xDCul)]
        [InlineData(3u, 1, 0xFEul)]
        [InlineData(2u, 2, 0x0FFEul)]
        public void A_narrow_read_of_the_stored_word_takes_the_lane_it_names(uint offset, int size, ulong expected)
        {
            var bus = Cartridge();
            bus.Store(Rom, 0xBADC0FFE, 4);

            Assert.Equal(expected, bus.Load(Data + offset, size));
        }

        // The corpus pins the decay only to a window; the constant inside it is the FPGA core's - see §7.7.
        [Fact]
        public void The_stored_word_decays()
        {
            var bus = Cartridge();

            bus.Store(Data, 0xBADC0FFE, 4);
            bus.Tick(PiInterface.StoreDecayCycles - 1);
            Assert.Equal(0xBADC0FFEul, bus.Load(Data, 4));

            bus.Store(Data, 0xBADC0FFE, 4);
            bus.Tick(PiInterface.StoreDecayCycles);
            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
        }

        // The corpus's two reads land 33 and 333 cycles after its store, which is the whole window it allows - see §7.9.
        [Fact]
        public void The_decay_falls_inside_the_window_the_corpus_allows()
        {
            var bus = Cartridge();

            bus.Store(Data, 0xBADC0FFE, 4);
            bus.Tick(33);
            Assert.Equal(0xBADC0FFEul, bus.Load(Data, 4));

            bus.Store(Data, 0xBADC0FFE, 4);
            bus.Tick(333);
            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
        }

        // Busy is the stored word still being there: a read frees it, and so does the decay - see §7.7.
        [Fact]
        public void The_bus_is_busy_until_a_read_or_the_decay_frees_it()
        {
            var bus = Cartridge();
            Assert.False(Busy(bus));

            bus.Store(Data, 0x01010101_23232323, 8);
            Assert.True(Busy(bus));
            bus.Load(Data, 4);
            Assert.False(Busy(bus));

            bus.Store(Data, 0xBADC0FFE, 4);
            bus.Tick(PiInterface.StoreDecayCycles);
            Assert.False(Busy(bus));
        }

        // A transfer reads the cartridge itself, and leaves the stored word for the processor - see §7.7.
        [Fact]
        public void A_transfer_neither_sees_nor_frees_the_stored_word()
        {
            var bus = Cartridge();
            bus.Store(Data, 0xBADC0FFE, 4);

            bus.Write32(MemoryMap.PiBase + PiInterface.DramAddress, 0x1000);
            bus.Write32(MemoryMap.PiBase + PiInterface.CartAddress, Data);
            bus.Write32(MemoryMap.PiBase + PiInterface.WriteLength, 8 - 1);

            Assert.Equal(0x01234567u, bus.Read32(0x1000));
            Assert.True(Busy(bus));
            Assert.Equal(0xBADC0FFEul, bus.Load(Data, 4));
        }

        // A decision about the instrument, not the console: the corpus prints through here, and would hold the latch it tests - see §7.7.
        [Fact]
        public void The_debug_port_is_outside_the_latch()
        {
            var bus = Cartridge();

            bus.Store(MemoryMap.IsViewerBase + 0x20, 0x12345678, 4);

            Assert.False(Busy(bus));
            Assert.Equal(0x01234567ul, bus.Load(Data, 4));
            Assert.Equal(0x12345678ul, bus.Load(MemoryMap.IsViewerBase + 0x20, 4));
        }
    }
}
