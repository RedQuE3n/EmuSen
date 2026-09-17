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

        // angrylion's own spans for this command, recorded from the differential so the walker is pinned where the reference is not built - see Mars_RdpTriangles.md §4.
        [Fact]
        public void A_triangle_in_the_fill_cycle_draws_the_spans_the_reference_draws()
        {
            int[,] spans =
            {
                { 4, 5 }, { 4, 7 }, { 4, 9 }, { 4, 11 }, { 4, 13 }, { 4, 15 }, { 4, 17 }, { 4, 19 }, { 4, 20 }, { 4, 19 }, { 4, 18 }, { 4, 17 },
                { 4, 16 }, { 5, 15 }, { 5, 14 }, { 5, 13 }, { 5, 13 }, { 5, 12 }, { 5, 11 }, { 5, 10 }, { 5, 9 }, { 5, 8 }, { 5, 7 }, { 5, 6 },
            };

            var bus = new MarsBus();
            RunList(bus, ColorImage(Bits16, 32, Framebuffer), Scissor(0, 0, 31, 31), FillCycle, FillColor(0xFFFF_FFFF),
                0x08800068_00280008UL, 0x00140000_FFFF2000UL, 0x00040000_00001555UL, 0x00040000_00020000UL, SyncFull);

            for (uint y = 0; y < 32; y++)
            {
                for (uint x = 0; x < 32; x++)
                {
                    int row = (int)y - 2;
                    bool inside = row >= 0 && row < spans.GetLength(0) && x >= spans[row, 0] && x <= spans[row, 1];
                    Assert.True((Pixel16(bus, 32, x, y) != 0) == inside, $"pixel ({x}, {y})");
                }
            }
        }

        // angrylion's own pixels and hidden bits for this anti-aliased one-cycle triangle, recorded from the differential - see Mars_RdpCoverage.md §7.
        [Fact]
        public void An_anti_aliased_one_cycle_triangle_stores_the_coverage_the_reference_stores()
        {
            var bus = new MarsBus();
            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x2F0000F0_00000048UL, 0x3C887F10_88FDF6FBUL, 0x3A00005A_C8642A9FUL, 0x3B000000_3C90D071UL, 0x39000000_7755AA80UL,
                0x38000000_2266EE40UL, 0x2C000000_00156B3CUL, 0x2E000000_12340040UL,
                0x08800071_0027000AUL, 0x001BC000_FFFEE7C9UL, 0x00032AE0_00002A41UL, 0x00018F73_0003611AUL, SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint X, uint Color, byte Hidden)[] row12 =
            {
                (4, 0x7BDE, 0), (5, 0xCB0B, 3), (15, 0xCB0B, 3), (24, 0xCB0B, 3), (25, 0xCB0A, 0), (26, 0x7BDE, 0), (27, 0x7BDF, 3),
            };

            foreach (var (x, color, hidden) in row12)
            {
                uint index = 0x0010_0000 / 2 + 12 * 32 + x;
                Assert.Equal(color, (uint)bus.Read16(index * 2));
                Assert.Equal(hidden, bus.RdramHidden[index]);
            }
        }

        // angrylion's own colours and stored depths across two shaded triangles crossing in depth, recorded from the differential - see Mars_RdpDepth.md §6.
        [Fact]
        public void Two_shaded_triangles_crossing_in_depth_store_the_colours_and_depths_the_reference_stores()
        {
            var bus = new MarsBus();
            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x3F10001F_00120000UL, 0x37000000_FFFCFFFCUL, 0x3607C07C_00000000UL, 0x3F10001F_00100000UL, 0x3E000000_00120000UL,
                0x2F0000F0_00000030UL, 0x3C887F10_88FE793CUL, 0x3A00005A_C8642A9FUL, 0x3B000000_3C90D071UL, 0x39000000_7755AA80UL,
                0x38000000_2266EE40UL, 0x2E000000_12340040UL,
                0x0D800071_0027000AUL, 0x001BC000_FFFEE7C9UL, 0x00032AE0_00002A41UL, 0x00018F73_0003611AUL, 0x00FE0013_00380103UL,
                0xFFF80008_FFFFFFFDUL, 0x13E20775_B2F4B7ECUL, 0x3D02D62A_3BC17C71UL, 0xFFF70001_0006FFF6UL, 0xFFF90000_0006FFF6UL,
                0xD83CF116_9A199028UL, 0x202F7BB8_BA7DFA60UL, 0x0F392A41_02F31F10UL, 0x018DAB7F_011109C9UL,
                0x0D000077_00460005UL, 0x0004C000_00019783UL, 0x001C898B_FFFFD9D3UL, 0x001CDD8A_FFFE89D9UL, 0x00C800C9_001E0058UL,
                0x0005FFFC_FFF5FFFFUL, 0xB3A6AAAB_2CEA8D79UL, 0xD518CBBF_264D40E3UL, 0xFFFDFFF9_FFFF0005UL, 0xFFFEFFF8_FFFD0005UL,
                0x31675555_4C5ACA1BUL, 0x100FDB04_AE22AD9BUL, 0x7ECA1AF3_035078E1UL, 0xFCD79436_FD561B02UL,
                SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint X, uint Color, uint Depth)[] row14 =
            {
                (0, 0x7BDE, 0xFFFC), (6, 0x91A1, 0x1232), (9, 0x7A61, 0x169E), (12, 0x4D2D, 0x12AE), (15, 0x64E5, 0x17A6),
                (18, 0x745D, 0x1C9E), (21, 0x8415, 0x232E), (24, 0x93CD, 0x2D22), (27, 0x7BDF, 0xFFFC),
            };

            foreach (var (x, color, depth) in row14)
            {
                uint pixel = 14 * 32 + x;
                Assert.Equal(color, (uint)bus.Read16(0x0010_0000 + pixel * 2));
                Assert.Equal(depth, (uint)bus.Read16(0x0012_0000 + pixel * 2));
            }
        }

        // angrylion's pixels and texture memory for an RGBA16 tile loaded from RDRAM and drawn through a texture rectangle, recorded from the reference - see Mars_RdpTextures.md §7.5.
        [Fact]
        public void A_tile_loaded_and_drawn_through_a_texture_rectangle_matches_the_reference()
        {
            var bus = new MarsBus();
            for (uint y = 0; y < 8; y++)
            {
                for (uint x = 0; x < 8; x++) bus.Write16(0x0020_0000 + (y * 8 + x) * 2, (ushort)(((x * 0x0843 + y * 0x2109) & 0xFFFE) | ((x ^ y) & 1)));
            }

            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x3D100007_00200000UL, 0x35100410_00000000UL, 0x34000000_0001C01CUL, 0x2F000800_00000000UL, 0x3C887F10_88FCF279UL,
                0x24048048_00008008UL, 0x00000000_02000400UL,
                SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint Y, uint X, uint Color)[] pixels =
            {
                (3, 2, 0x2109), (3, 3, 0x294B), (3, 4, 0x294D), (3, 5, 0x294D), (3, 10, 0x4215), (3, 17, 0x6321),
                (17, 2, 0xEF7F), (17, 3, 0xEF7F), (17, 4, 0xF7C3), (17, 5, 0xF7C3), (17, 10, 0x084B), (17, 17, 0x2115),
            };

            foreach (var (y, x, color) in pixels) Assert.Equal(color, (uint)bus.Read16(0x0010_0000 + (y * 32 + x) * 2));

            // The tile's second row is odd, so each of its groups is stored with its two halves swapped - see Mars_RdpTextures.md §2.1.
            uint[] rows =
            {
                0x0000, 0x0843, 0x1086, 0x18C9, 0x210C, 0x294F, 0x3192, 0x39D5,
                0x318F, 0x39D2, 0x2109, 0x294C, 0x529B, 0x5ADE, 0x4215, 0x4A58,
            };

            byte[] memory = bus.Dp.Processor.TextureMemory;
            for (int i = 0; i < rows.Length; i++) Assert.Equal(rows[i], (uint)((memory[(0x40 + i) * 2] << 8) | memory[(0x40 + i) * 2 + 1]));
        }

        // angrylion's pixels and texture memory for a palette loaded from RDRAM and read through a filtered colour-indexed tile, recorded from the reference - see Mars_RdpFiltering.md §5.6.
        [Fact]
        public void A_palette_loaded_and_read_through_a_filtered_colour_indexed_tile_matches_the_reference()
        {
            var bus = new MarsBus();
            for (uint i = 0; i < 16; i++) bus.Write16(0x0020_0000 + i * 2, (ushort)(((i * 0x1357 + 0x0F0F) & 0xFFFE) | (i & 1)));
            for (uint y = 0; y < 8; y++)
            {
                for (uint x = 0; x < 8; x++) bus.Write8(0x0020_0100 + y * 8 + x, (byte)((x * 5 + y * 3) & 0x0F));
            }

            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x3D10000F_00200000UL, 0x35000100_07000000UL, 0x30000000_0703C000UL,
                0x3D480007_00200100UL, 0x35480200_00000000UL, 0x34000000_0001C01CUL,
                0x2F00ACF0_00000000UL, 0x3C887F10_88FCF279UL,
                0x24078050_00008008UL, 0x0007000B_01550123UL,
                SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint Y, uint X, uint Color)[] pixels =
            {
                (2, 2, 0x354D), (2, 5, 0x944D), (2, 10, 0x2A31), (3, 7, 0x6973), (9, 5, 0x7D0F), (9, 16, 0x2A2F),
                (12, 11, 0x9211), (15, 3, 0x7C65), (15, 24, 0x312B), (19, 7, 0xBD61), (19, 20, 0x13B5), (19, 29, 0x3E35),
            };

            foreach (var (y, x, color) in pixels) Assert.Equal(color, (uint)bus.Read16(0x0010_0000 + (y * 32 + x) * 2));

            // Palette entries 0 and 1, each in all four of its banks, and the index tile's odd second row with its halves swapped - see Mars_RdpFiltering.md §4.2.
            (int Word, uint Value)[] words =
            {
                (0x400, 0x0F0E), (0x401, 0x0F0E), (0x402, 0x0F0E), (0x403, 0x0F0E), (0x404, 0x2267), (0x405, 0x2267), (0x406, 0x2267), (0x407, 0x2267),
                (0x004, 0x070C), (0x005, 0x0106), (0x006, 0x0308), (0x007, 0x0D02),
            };

            byte[] memory = bus.Dp.Processor.TextureMemory;
            foreach (var (word, value) in words) Assert.Equal(value, (uint)((memory[word * 2] << 8) | memory[word * 2 + 1]));
        }

        // angrylion's pixels for a perspective triangle over a four-level mipmap chain, its level-of-detail fraction the multiplier, recorded from the reference - see Mars_RdpLod.md §4.5.
        [Fact]
        public void A_mipmapped_triangle_with_its_level_of_detail_fraction_matches_the_reference()
        {
            var bus = new MarsBus();
            for (uint i = 0; i < 0x400; i++) bus.Write16(0x0020_0000 + i * 2, (ushort)(((i * 0x2B13 + 0x1357) & 0xFFFE) | (i & 1)));

            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x3D10001F_00200000UL, 0x35101000_00014050UL, 0x34000000_0007C07CUL,
                0x3D10000F_00200100UL, 0x35100900_01010441UL, 0x34000000_0103C03CUL,
                0x3D100007_00200200UL, 0x35100540_0200C832UL, 0x34000000_0201C01CUL,
                0x3D100003_00200300UL, 0x35100350_03008C23UL, 0x34000000_0300C00CUL,
                0x2F090CF0_00000000UL, 0x3C16902D_8821FEFFUL, 0x3A00005A_C8642A9FUL,
                0x0A980075_001B0006UL, 0x001DC000_FFFEF777UL, 0x00022C65_00002735UL, 0xFFFFA186_00053CF4UL, 0xFFF3FF97_7FFF0000UL, 0x0046FFDF_FD270000UL,
                0xE4531025_FFFF0000UL, 0xC1371E00_D82E0000UL, 0x001800D1_FE9D0000UL, 0x000D00D6_FF0D0000UL, 0x375ADFB6_C31D0000UL, 0x6141E8F4_480F0000UL,
                SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint Y, uint X, uint Color)[] pixels =
            {
                (3, 4, 0xFD9F), (5, 20, 0xCD61), (6, 25, 0xFD53), (7, 29, 0xEC14), (9, 6, 0xED99), (10, 26, 0xFCDC),
                (12, 10, 0xD34F), (14, 8, 0x0413), (16, 19, 0xD5EB), (19, 13, 0xFD13), (22, 12, 0xFD53), (26, 8, 0xFFA9),
            };

            foreach (var (y, x, color) in pixels) Assert.Equal(color, (uint)bus.Read16(0x0010_0000 + (y * 32 + x) * 2));
        }

        // angrylion's pixels for one perspective triangle drawn twice in the two-cycle mode, the second pass under the level of detail, convert-one and alpha compare - see Mars_RdpTwoCycle.md §5.5.
        [Fact]
        public void A_two_cycle_triangle_through_both_combiner_and_blender_cycles_matches_the_reference()
        {
            var bus = new MarsBus();
            for (uint i = 0; i < 0x400; i++) bus.Write16(0x0020_0000 + i * 2, (ushort)(((i * 0x2B13 + 0x1357) & 0xFFFE) | (i & 1)));

            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x3D10001F_00200000UL, 0x35101000_00014050UL, 0x34000000_0007C07CUL,
                0x3D10000F_00200100UL, 0x35100900_01010441UL, 0x34000000_0103C03CUL,
                0x3D100007_00200200UL, 0x35100540_0200C832UL, 0x34000000_0201C01CUL,
                0x3D100003_00200300UL, 0x35100350_03008C23UL, 0x34000000_0300C00CUL,
                0x2F180CF0_00604000UL, 0x3C25260A_110C927FUL, 0x3A00005A_C8642A9FUL, 0x39000000_7755AA80UL,
                0x0A980075_001B0006UL, 0x001DC000_FFFEF777UL, 0x00022C65_00002735UL, 0xFFFFA186_00053CF4UL, 0xFFF3FF97_7FFF0000UL, 0x0046FFDF_FD270000UL,
                0xE4531025_FFFF0000UL, 0xC1371E00_D82E0000UL, 0x001800D1_FE9D0000UL, 0x000D00D6_FF0D0000UL, 0x375ADFB6_C31D0000UL, 0x6141E8F4_480F0000UL,
                0x2F190AC0_00604001UL, 0x3C26FE0B_110CF27FUL,
                0x0A980075_001B0006UL, 0x001DC000_FFFEF777UL, 0x00022C65_00002735UL, 0xFFFFA186_00053CF4UL, 0xFFF3FF97_7FFF0000UL, 0x0046FFDF_FD270000UL,
                0xE4531025_FFFF0000UL, 0xC1371E00_D82E0000UL, 0x001800D1_FE9D0000UL, 0x000D00D6_FF0D0000UL, 0x375ADFB6_C31D0000UL, 0x6141E8F4_480F0000UL,
                SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint Y, uint X, uint Color)[] pixels =
            {
                (3, 5, 0x4999), (4, 15, 0x8BE9), (5, 20, 0x82E5), (6, 25, 0x82E5), (7, 4, 0x4999), (8, 4, 0x9C6B),
                (8, 28, 0x6AE3), (9, 25, 0x946D), (10, 26, 0x49DB), (11, 16, 0x49E5), (12, 21, 0x6A5F), (15, 11, 0x835D),
                (17, 5, 0x6A5D), (19, 9, 0x942D), (22, 12, 0x72AB), (24, 6, 0x9A9F), (26, 9, 0x72E7), (27, 7, 0x7323),
            };

            foreach (var (y, x, color) in pixels) Assert.Equal(color, (uint)bus.Read16(0x0010_0000 + (y * 32 + x) * 2));
        }

        // angrylion's pixels for copy-mode rectangles and a right-major triangle, whose spans the mode writes torn - see Mars_RdpCopy.md §5.5.
        [Fact]
        public void Copy_mode_rectangles_and_a_right_major_triangle_match_the_reference()
        {
            var bus = new MarsBus();
            for (uint i = 0; i < 0x400; i++) bus.Write16(0x0020_0000 + i * 2, (ushort)(((i * 0x2B13 + 0x1357) & 0xFFFE) | (i & 1)));

            const uint list = 0x0030_0000;
            uint end = WriteList(bus, list,
                0x3F10001F_00100000UL, 0x2D000000_0007C07CUL, 0x2F300000_00000000UL, 0x37000000_7BDE7BDFUL, 0x3607C07C_00000000UL,
                0x3D10001F_00200000UL, 0x35101000_00014050UL, 0x34000000_0007C07CUL,
                0x3D08000F_00200200UL, 0x35080400_01010441UL, 0x34000000_0103C03CUL,
                0x3A00005A_C8642A9FUL, 0x39000000_7755AA80UL,
                0x2F200000_00000000UL, 0x24186032_00003008UL, 0x00000000_10000400UL,
                0x2F200000_00000001UL, 0x2418C062_00006808UL, 0x00000000_10000400UL,
                0x2F200000_00000000UL, 0x2401C032_01003008UL, 0x00000000_08000800UL,
                0x0A000070_00200008UL, 0x00040000_0000E666UL, 0x001A0000_FFFFD89EUL, 0x001A0000_FFFC5555UL, 0x00000000_7FFF0000UL,
                0xFFC50006_00000000UL, 0x00000000_00000000UL, 0x059B4E7B_00000000UL, 0x00060036_00000000UL, 0xFFFD0037_00000000UL,
                0x27622762_00000000UL, 0x148E1FC4_00000000UL,
                SyncFull);

            bus.Write32(Start, list);
            bus.Write32(End, end);

            (uint Y, uint X, uint Color)[] pixels =
            {
                (2, 3, 0x516B), (2, 17, 0x029B), (3, 9, 0x0C63), (4, 25, 0xA319), (5, 14, 0xCA48), (6, 6, 0xB3CB),
                (7, 21, 0x7C91), (8, 12, 0xB628), (9, 28, 0x7A0A), (10, 4, 0x16A9), (11, 19, 0x8331), (12, 23, 0x8F6B),
                (13, 7, 0x7B63), (15, 10, 0xA8CE), (17, 13, 0xA4DD), (19, 16, 0x2E5D), (22, 15, 0x7B17), (25, 18, 0x7B1D),
                (27, 20, 0x7B77),
            };

            foreach (var (y, x, color) in pixels) Assert.Equal(color, (uint)bus.Read16(0x0010_0000 + (y * 32 + x) * 2));
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
