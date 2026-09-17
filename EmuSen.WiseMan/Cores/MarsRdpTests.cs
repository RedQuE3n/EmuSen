using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The display processor's interface, its command stream and the fill cycle, in the corpus's own sequences - see Mars_Rdp.md.
    public class MarsRdpTests
    {
        private const uint Start = MemoryMap.DpCommandBase + 0x00;
        private const uint End = MemoryMap.DpCommandBase + 0x04;
        private const uint Current = MemoryMap.DpCommandBase + 0x08;
        private const uint Status = MemoryMap.DpCommandBase + 0x0C;

        private const uint ClearXbus = 0x1;
        private const uint SetXbus = 0x2;
        private const uint ClearFreeze = 0x4;
        private const uint SetFreeze = 0x8;

        private const uint Xbus = 0x001;
        private const uint Freeze = 0x002;
        private const uint StartGclk = 0x008;
        private const uint PipeBusy = 0x020;
        private const uint BufferReady = 0x080;
        private const uint EndValid = 0x200;
        private const uint StartValid = 0x400;

        private const uint MiMode = MemoryMap.MiBase + 0x00;
        private const uint ClearDisplayProcessorInterrupt = 0x800;

        private const uint List = 0x0010_0000;
        private const uint Framebuffer = 0x0020_0000;

        private const int Bits8 = 1;
        private const int Bits16 = 2;
        private const int Bits32 = 3;

        private const ulong SyncPipe = 0x27UL << 56;
        private const ulong SyncFull = 0x29UL << 56;
        private const ulong FillCycle = (0x2FUL << 56) | (3UL << 52);

        [Theory]
        [InlineData(0xFFFu)]
        [InlineData(0x00FF_FFFFu)]
        [InlineData(0x12FF_FFFFu)]
        [InlineData(0x1280_0000u)]
        [InlineData(0xFFFF_FFFFu)]
        public void Start_and_end_keep_twenty_four_bits_in_whole_words(uint value)
        {
            var bus = new MarsBus();
            bus.Write32(Status, SetFreeze);

            bus.Write32(Start, value);
            bus.Write32(End, value);

            Assert.Equal(value & 0xFF_FFF8, bus.Read32(Start));
            Assert.Equal(value & 0xFF_FFF8, bus.Read32(Current));
            Assert.Equal(value & 0xFF_FFF8, bus.Read32(End));
        }

        // A start write is held, later start writes are refused, and the end write takes it into current - see §2.1.
        [Fact]
        public void A_start_write_holds_until_an_end_write_takes_it_into_current()
        {
            var bus = new MarsBus();
            bus.Write32(Status, SetFreeze);
            Assert.Equal(Freeze, bus.Read32(Status) & Freeze);

            bus.Write32(Start, 0x40);
            bus.Write32(End, 0x40);

            bus.Write32(Start, 0x1238);
            Assert.Equal(StartValid, bus.Read32(Status) & (StartValid | EndValid));

            bus.Write32(Start, 0x12_3450);
            Assert.Equal(0x1238u, bus.Read32(Start));
            Assert.Equal(0x40u, bus.Read32(Current));

            bus.Write32(End, 0x1238);
            Assert.Equal(0u, bus.Read32(Status) & (StartValid | EndValid));
            Assert.Equal(0x1238u, bus.Read32(Current));

            bus.Write32(Status, ClearFreeze);
            Assert.Equal(0u, bus.Read32(Status) & Freeze);
        }

        // The corpus's own run, stopped at each end write and read back - see §2.2.
        [Fact]
        public void An_end_write_starts_the_clock_and_a_full_sync_stops_it_and_interrupts()
        {
            var bus = new MarsBus();
            uint end = WriteList(bus, List,
                ColorImage(Bits16, 8, Framebuffer), Scissor(0, 0, 8, 8), FillCycle, FillColor(0x003E_003E),
                FillRectangle(0, 0, 7, 7), SyncPipe, SyncFull, SyncFull);

            bus.Write32(Start, List);
            bus.Write32(End, List);
            Assert.Equal(BufferReady | PipeBusy | StartGclk, bus.Read32(Status));
            Assert.Equal(List, bus.Read32(Current));

            bus.Write32(End, end - 16);
            Assert.Equal(BufferReady | PipeBusy | StartGclk, bus.Read32(Status));
            Assert.Equal(MiInterrupt.None, bus.Mi.Pending);

            bus.Write32(End, end - 8);
            Assert.Equal(BufferReady, bus.Read32(Status));
            Assert.Equal(MiInterrupt.DisplayProcessor, bus.Mi.Pending);
            Assert.Equal(0x003Eu, Pixel16(bus, 8, 0, 0));

            bus.Write32(End, end);
            Assert.Equal(BufferReady, bus.Read32(Status));
            Assert.Equal(List, bus.Read32(Start));
            Assert.Equal(end, bus.Read32(Current));
        }

        [Fact]
        public void A_frozen_processor_draws_nothing_until_it_is_thawed()
        {
            var bus = new MarsBus();
            uint end = WriteList(bus, List,
                ColorImage(Bits16, 8, Framebuffer), Scissor(0, 0, 8, 8), FillCycle, FillColor(0x07C0_07C0),
                FillRectangle(0, 0, 7, 7), SyncFull);

            bus.Write32(Status, SetFreeze);
            bus.Write32(Start, List);
            bus.Write32(End, end);
            Assert.Equal(0u, Pixel16(bus, 8, 0, 0));
            Assert.Equal(MiInterrupt.None, bus.Mi.Pending);

            bus.Write32(Status, ClearFreeze);
            Assert.Equal(0x07C0u, Pixel16(bus, 8, 0, 0));
            Assert.Equal(end, bus.Read32(Current));
        }

        // Data memory as the stream's source, including a list that runs past its end and wraps - see §2.3.
        [Theory]
        [InlineData(0x000u)]
        [InlineData(0xFC8u)]
        [InlineData(0xFF0u)]
        public void With_the_xbus_bit_set_commands_come_from_data_memory(uint dmem)
        {
            var bus = new MarsBus();
            ulong[] list =
            {
                ColorImage(Bits16, 8, Framebuffer), Scissor(0, 0, 8, 8), FillCycle, FillColor(0x07C0_07C0),
                FillRectangle(0, 0, 7, 7), SyncPipe, SyncFull,
            };

            for (int i = 0; i < list.Length * 8; i++)
            {
                bus.SpDmem[(dmem + i) & 0xFFF] = (byte)(list[i / 8] >> (56 - 8 * (i % 8)));
            }

            bus.Write32(Status, SetXbus);
            bus.Write32(Start, dmem);
            bus.Write32(End, dmem + 0x38);

            Assert.Equal(Xbus | BufferReady, bus.Read32(Status));
            Assert.Equal(dmem + 0x38, bus.Read32(Current));
            Assert.Equal(0x07C0u, Pixel16(bus, 8, 0, 0));

            bus.Write32(Status, ClearXbus);
            Assert.Equal(0u, bus.Read32(Status) & Xbus);
        }

        // A command is taken a word at a time and runs only once all of its words have arrived - see §3.
        [Theory]
        [InlineData(0x08, 32)]
        [InlineData(0x0C, 96)]
        [InlineData(0x0F, 176)]
        [InlineData(0x24, 16)]
        public void A_long_command_waits_for_all_its_words_and_none_of_them_is_read_as_a_command(int id, int length)
        {
            var bus = new MarsBus();
            var words = new ulong[length / 8 + 1];
            words[0] = (ulong)id << 56;
            for (int i = 1; i < words.Length; i++) words[i] = SyncFull;

            uint end = WriteList(bus, List, words);

            bus.Write32(Start, List);
            bus.Write32(End, List + 16);
            Assert.Equal(List + 16, bus.Read32(Current));
            Assert.Equal(MiInterrupt.None, bus.Mi.Pending);

            bus.Write32(End, end - 8);
            Assert.Equal(MiInterrupt.None, bus.Mi.Pending);

            bus.Write32(End, end);
            Assert.Equal(MiInterrupt.DisplayProcessor, bus.Mi.Pending);
        }

        // Both halves of the colour land, alternately, on sixteen-bit pixels - see §5.1.
        [Fact]
        public void A_sixteen_bit_fill_writes_its_colour_as_whole_words_across_pixel_pairs()
        {
            var bus = new MarsBus();
            RunList(bus, ColorImage(Bits16, 4, Framebuffer), Scissor(0, 0, 4, 2), FillCycle, FillColor(0x1111_2222),
                FillRectangle(0, 0, 3, 1), SyncFull);

            Assert.Equal(0x1111u, Pixel16(bus, 4, 0, 0));
            Assert.Equal(0x2222u, Pixel16(bus, 4, 1, 0));
            Assert.Equal(0x1111u, Pixel16(bus, 4, 2, 1));
            Assert.Equal(0x2222u, Pixel16(bus, 4, 3, 1));
        }

        [Fact]
        public void A_fill_at_other_pixel_sizes_writes_the_lanes_of_the_word_each_pixel_occupies()
        {
            var bus = new MarsBus();
            RunList(bus, ColorImage(Bits32, 2, Framebuffer), Scissor(0, 0, 2, 1), FillCycle, FillColor(0x1122_3344),
                FillRectangle(0, 0, 1, 0), SyncFull);

            Assert.Equal(0x1122_3344u, bus.Read32(Framebuffer));
            Assert.Equal(0x1122_3344u, bus.Read32(Framebuffer + 4));

            RunList(bus, ColorImage(Bits8, 3, Framebuffer + 0x101), Scissor(0, 0, 3, 1), FillCycle, FillColor(0x1122_3344),
                FillRectangle(0, 0, 2, 0), SyncFull);

            Assert.Equal(0x0022_3344u, bus.Read32(Framebuffer + 0x100));
        }

        // The scissor's right column is drawn and its bottom row is not; graded against the reference rasterizer - see §5.2.
        [Fact]
        public void A_fill_reaches_the_scissor_right_column_but_not_its_bottom_row()
        {
            var bus = new MarsBus();
            RunList(bus, ColorImage(Bits16, 8, Framebuffer), Scissor(0, 0, 4, 3), FillCycle, FillColor(0xFFFF_FFFF),
                FillRectangle(1, 1, 5, 5), SyncFull);

            for (uint y = 0; y < 6; y++)
            {
                for (uint x = 0; x < 7; x++)
                {
                    bool inside = x >= 1 && x <= 4 && y >= 1 && y <= 2;
                    Assert.True((Pixel16(bus, 8, x, y) != 0) == inside, $"pixel ({x}, {y})");
                }
            }
        }

        [Fact]
        public void Writing_the_mode_register_clear_bit_lowers_the_display_processor_interrupt()
        {
            var bus = new MarsBus();
            RunList(bus, SyncFull);
            Assert.Equal(MiInterrupt.DisplayProcessor, bus.Mi.Pending);

            bus.Write32(MiMode, ClearDisplayProcessorInterrupt);
            Assert.Equal(MiInterrupt.None, bus.Mi.Pending);
        }

        private static void RunList(MarsBus bus, params ulong[] words)
        {
            uint end = WriteList(bus, List, words);
            bus.Write32(Start, List);
            bus.Write32(End, end);
        }

        private static uint WriteList(MarsBus bus, uint address, params ulong[] words)
        {
            for (int i = 0; i < words.Length; i++) bus.Write64(address + (uint)(i * 8), words[i]);
            return address + (uint)(words.Length * 8);
        }

        private static ushort Pixel16(MarsBus bus, uint width, uint x, uint y) => bus.Read16(Framebuffer + (y * width + x) * 2);

        private static ulong ColorImage(int size, uint width, uint address) =>
            (0x3FUL << 56) | ((ulong)size << 51) | ((ulong)(width - 1) << 32) | address;

        private static ulong Scissor(uint left, uint top, uint right, uint bottom) =>
            (0x2DUL << 56) | ((ulong)left << 46) | ((ulong)top << 34) | ((ulong)right << 14) | ((ulong)bottom << 2);

        private static ulong FillColor(uint color) => (0x37UL << 56) | color;

        private static ulong FillRectangle(uint left, uint top, uint right, uint bottom) =>
            (0x36UL << 56) | ((ulong)right << 46) | ((ulong)bottom << 34) | ((ulong)left << 14) | ((ulong)top << 2);
    }
}
