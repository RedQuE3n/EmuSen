using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // The video interface without the reference tools: angrylion's pixels, held as constants - see Mars_Video.md §3.4.
    public class MarsViTests
    {
        private const uint Framebuffer = 0x0020_0000;

        private const uint GammaOn = 1 << 3, DivotOn = 1 << 4, DitherFilter = 1 << 16;

        // angrylion's pixels for sixteen scans of one frame buffer, three of which scan no frame at all - see Mars_Video.md §3.4.
        [Fact]
        public void Scanned_frames_match_the_reference()
        {
            var bus = new MemoryBus();

            uint state = 0x2468_ACE0;
            for (uint i = 0; i < 0x3000; i++)
            {
                state = state * 1103515245 + 12345;
                bus.Write8(Framebuffer + i, (byte)(state >> 16));
            }

            // The coverage the display processor left beside each word, which only the anti-aliased scans read - see Mars_VideoFilter.md §1.
            uint seed = 0x1B4E_81B4;
            for (int i = 0; i < 0x1800; i++)
            {
                seed = seed * 1664525 + 1013904223;
                bus.RdramHidden[Framebuffer / 2 + i] = (byte)(seed >> 30);
            }

            uint[] shorter = Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0);
            uint[] blank = Registers(0, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
            (uint[] Registers, bool Frame)[] scans =
            {
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), true),
                (Registers(2, 2, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40), true),
                (Registers(3, 2, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0), true),
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0, serrate: true, currentLine: 1), true),
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0, serrate: true), true),
                (shorter, true), (shorter, true), (shorter, true),
                (Registers(0, 3, 64, 0x400, 0x400, 148, 200, 34, 120, 0, 0), true),
                (Registers(2, 3, 64, 0x400, 0x400, 40, 640, 34, 120, 0, 0), true),
                (Registers(2, 3, 64, 0x400, 0x400, 128, 640, 44, 240, 0, 0, sync: 625), true),
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, origin: 0), false),
                (blank, true),
                (blank, false),
                (Registers(2, 3, 64, 0x400, 0x400, 108, 0, 34, 120, 0, 0), false),
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), true),
            };

            var frames = new List<byte[]>();
            foreach ((uint[] registers, bool expected) in scans)
            {
                for (int i = 0; i < registers.Length; i++) bus.Write32(MemoryMap.ViBase + (uint)i * 4, registers[i]);
                Assert.Equal(expected, bus.Vi.Scan());
                if (expected) frames.Add(bus.Vi.Frame.ToArray());
            }

            // Six more scans read a coverage: both modes that do, both pixel formats, a line read twice, and a picture all whole then none - see Mars_VideoFilter.md §4.
            (uint[] Registers, int Hidden)[] filtered =
            {
                (Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), -1),
                (Registers(3, 1, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0), -1),
                (Registers(2, 0, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40), -1),
                (Registers(2, 1, 64, 0x400, 0x200, 108, 256, 34, 110, 0, 0), -1),
                (Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), 3),
                (Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0), 0),

                // The three passes past the filter: undoing dither, divot, gamma, and all of them together - see Mars_VideoPasses.md §4.
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DitherFilter), -1),
                (Registers(3, 3, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0, control: DitherFilter), -1),
                (Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DivotOn), 3),
                (Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: GammaOn), -1),
                (Registers(2, 1, 64, 0x2AB, 0x155, 108, 256, 34, 120, 0x80, 0x40, control: DitherFilter | DivotOn | GammaOn), -1),
            };

            foreach ((uint[] registers, int bits) in filtered)
            {
                if (bits >= 0) Array.Fill(bus.RdramHidden, (byte)bits, (int)Framebuffer / 2, 0x1800);

                for (int i = 0; i < registers.Length; i++) bus.Write32(MemoryMap.ViBase + (uint)i * 4, registers[i]);
                Assert.True(bus.Vi.Scan());
                frames.Add(bus.Vi.Frame.ToArray());
            }

            (int Frame, int X, int Y, uint Color)[] pixels =
            {
                (0, 20, 5, 0x80C88007), (0, 100, 30, 0xB848D807), (0, 180, 60, 0x38401807), (0, 250, 100, 0x00000000),
                (1, 60, 12, 0x8396BA07), (1, 140, 44, 0x7BC36A07), (1, 220, 77, 0x62C28207),
                (2, 20, 5, 0x08B02C07), (2, 100, 30, 0xDB584907), (2, 140, 44, 0x00000000),
                (3, 20, 4, 0x68885807), (3, 100, 31, 0x1B019E07), (3, 200, 100, 0x7068F807), (3, 150, 301, 0x00000000),
                (4, 20, 5, 0x68885807), (4, 100, 30, 0xF090F807), (4, 200, 101, 0x7068F807), (4, 150, 300, 0x00000000),
                (5, 20, 5, 0x80C88007), (5, 20, 70, 0x00000000), (5, 250, 30, 0x00000000),
                (6, 20, 70, 0x00000000), (6, 100, 100, 0x00000000),
                (7, 20, 70, 0x00000000), (7, 100, 100, 0x00000000), (7, 250, 30, 0x00000000),
                (8, 60, 20, 0x90187807), (8, 20, 20, 0x00000000), (8, 300, 20, 0x00000000),
                (9, 3, 10, 0x78681007), (9, 20, 10, 0x1810B007), (9, 569, 10, 0x00000000), (9, 600, 10, 0x00000000),
                (10, 20, 5, 0x80C88007), (10, 300, 200, 0x00000007), (10, 100, 280, 0x00000000),
                (11, 20, 5, 0x80C88007), (11, 300, 40, 0x00000000),
                (12, 20, 5, 0x80C88007), (12, 100, 100, 0x00000007), (12, 300, 40, 0x00000000),

                (13, 20, 5, 0x80C88000), (13, 100, 30, 0xB848D805), (13, 180, 60, 0x38401802),
                (13, 240, 100, 0x00000000), (13, 3, 5, 0x00000000), (13, 252, 5, 0x00000000),
                (14, 20, 5, 0x08B02C03), (14, 100, 30, 0xDB584907), (14, 60, 44, 0xD2D70405),
                (15, 60, 12, 0x8396BA05), (15, 140, 44, 0x7DC08502), (15, 220, 77, 0x62C28203), (15, 300, 150, 0x9CAEDE00),
                (16, 20, 5, 0x8A728A04), (16, 100, 30, 0xF090F802), (16, 180, 60, 0x58F0E803), (16, 200, 100, 0xAFB5F800),
                (17, 20, 5, 0x7CCCA003), (17, 100, 30, 0xB848D807), (17, 180, 60, 0x7C607003),
                (18, 20, 5, 0x80C88000), (18, 100, 30, 0xB848D804), (18, 180, 60, 0x38401800),

                (19, 20, 5, 0x7CC48207), (19, 100, 30, 0xB350D007), (19, 180, 60, 0x3C412007), (19, 9, 1, 0x26866607),
                (20, 20, 5, 0x0EAC3207), (20, 100, 30, 0xD5544D07), (20, 60, 44, 0xCCD10C07),
                (21, 20, 5, 0x7C605803), (21, 100, 30, 0x80B09807), (21, 180, 60, 0x7C8C7003),
                (22, 20, 5, 0xB4E2B407), (22, 100, 30, 0xD886EA07), (22, 180, 60, 0x76804E07), (22, 250, 100, 0x00000000),
                (23, 20, 5, 0xC8C8A207), (23, 100, 30, 0xCC9CBA07), (23, 180, 60, 0x8EC8E207),
            };

            foreach ((int frame, int x, int y, uint color) in pixels)
            {
                byte[] scanned = frames[frame];
                int at = (y * 640 + x) * 4;
                uint got = ((uint)scanned[at] << 24) | ((uint)scanned[at + 1] << 16) | ((uint)scanned[at + 2] << 8) | scanned[at + 3];
                Assert.Equal(color, got);
            }
        }

        // The window of fetched lines is sized by the frame buffer's width, so a narrower buffer after a wider one must not read the old rows - see Mars_Performance.md §21.
        [Fact]
        public void A_narrower_frame_buffer_after_a_wider_one_scans_as_it_would_alone()
        {
            MemoryBus twice = Noisy(), once = Noisy();
            uint[] wide = Registers(2, 0, 128, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DitherFilter | DivotOn);
            uint[] narrow = Registers(2, 0, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DitherFilter | DivotOn);

            Program(twice, wide);
            Assert.True(twice.Vi.Scan());
            Program(twice, narrow);
            Assert.True(twice.Vi.Scan());

            Program(once, narrow);
            Assert.True(once.Vi.Scan());

            Assert.Equal(once.Vi.Frame.ToArray(), twice.Vi.Frame.ToArray());
        }

        // A walker's stamps wrap after 2^31 new lines, and a wide picture after a long narrow one must not find its old samples - see Mars_Video.md §2.13.
        [Fact]
        public void A_scan_after_the_walkers_stamps_wrap_reads_no_sample_from_before_it()
        {
            // Forty rows take stamps 1 to 41 and thirty-seven take 1 to 38, so after the wrap the wide picture's first two rows are stamped as its last two were.
            uint[] wide = Registers(2, 0, 320, 0x200, 0x400, 108, 640, 34, 40, 0, 0, control: DitherFilter | DivotOn);
            uint[] narrow = Registers(2, 0, 320, 0x200, 0x400, 108, 256, 34, 37, 0, 0, control: DitherFilter | DivotOn);
            MemoryBus wrapped = Painted(1), fresh = Painted(1);

            foreach (MemoryBus bus in new[] { wrapped, fresh })
            {
                Program(bus, wide);
                Assert.True(bus.Vi.Scan());
                Program(bus, narrow);
                Assert.True(bus.Vi.Scan());
            }

            // The narrow picture's two thousand million lines, which leave the wide one's outer offsets as it stamped them.
            object vi = wrapped.Vi;
            var walkers = (Array)vi.GetType().GetField("_walkers", NonPublic)!.GetValue(vi)!;
            Assert.NotEmpty(walkers);
            foreach (object walker in walkers) walker.GetType().GetField("_row", NonPublic)!.SetValue(walker, int.MaxValue - 1);

            foreach (MemoryBus bus in new[] { wrapped, fresh })
            {
                Paint(bus, 2);
                Program(bus, narrow);
                Assert.True(bus.Vi.Scan());
                Program(bus, wide);
                Assert.True(bus.Vi.Scan());
            }

            byte[] after = wrapped.Vi.Frame.ToArray(), expected = fresh.Vi.Frame.ToArray();
            int[] wrong = Enumerable.Range(0, after.Length).Where(i => after[i] != expected[i]).ToArray();
            Assert.True(wrong.Length == 0, wrong.Length == 0 ? "" :
                $"{wrong.Length} bytes of the picture after the wrap are not a fresh interface's, in rows {wrong.Min() / 2560} to {wrong.Max() / 2560} and columns {wrong.Min(i => i % 2560) / 4} to {wrong.Max(i => i % 2560) / 4}");
        }

        private const System.Reflection.BindingFlags NonPublic = System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic;

        // A frame buffer of 320-wide lines and its coverage, different for every seed.
        private static MemoryBus Painted(uint seed)
        {
            var bus = new MemoryBus();
            Paint(bus, seed);
            return bus;
        }

        private static void Paint(MemoryBus bus, uint seed)
        {
            uint state = seed * 0x9E37_79B9;
            for (int i = 0; i < 0x10000; i++)
            {
                state = state * 1103515245 + 12345;
                bus.Rdram[Framebuffer + i] = (byte)(state >> 16);
                if ((i & 1) == 0) bus.RdramHidden[(Framebuffer + i) / 2] = (byte)(state >> 30);
            }
        }

        private static void Program(MemoryBus bus, uint[] registers)
        {
            for (int i = 0; i < registers.Length; i++) bus.Write32(MemoryMap.ViBase + (uint)i * 4, registers[i]);
        }

        // The reference test's frame buffer and coverage, so every mode has something to filter.
        private static MemoryBus Noisy()
        {
            var bus = new MemoryBus();

            uint state = 0x2468_ACE0;
            for (uint i = 0; i < 0x3000; i++)
            {
                state = state * 1103515245 + 12345;
                bus.Write8(Framebuffer + i, (byte)(state >> 16));
            }

            uint seed = 0x1B4E_81B4;
            for (int i = 0; i < 0x1800; i++)
            {
                seed = seed * 1664525 + 1013904223;
                bus.RdramHidden[Framebuffer / 2 + i] = (byte)(seed >> 30);
            }

            return bus;
        }

        private static uint[] Registers(int type, int antialias, uint width, uint stepX, uint stepY,
            uint left, uint columns, uint top, uint rows, uint biasX, uint biasY,
            bool serrate = false, uint currentLine = 0, uint sync = 525, uint origin = Framebuffer, uint control = 0)
        {
            var registers = new uint[14];
            registers[0] = (uint)(type & 3) | ((uint)(antialias & 3) << 8) | (serrate ? 1u << 6 : 0) | control;
            registers[1] = origin;
            registers[2] = width;
            registers[4] = currentLine;
            registers[6] = sync;
            registers[9] = ((left & 0x3FF) << 16) | ((left + columns) & 0x3FF);
            registers[10] = ((top & 0x3FF) << 16) | ((top + rows * 2) & 0x3FF);
            registers[12] = ((biasX & 0xFFF) << 16) | (stepX & 0xFFF);
            registers[13] = ((biasY & 0xFFF) << 16) | (stepY & 0xFFF);
            return registers;
        }
    }
}
