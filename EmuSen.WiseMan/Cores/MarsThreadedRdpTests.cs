using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.WiseMan.Cores
{
    // A list carried out on the display processor's thread leaves what the list carried out at once leaves - see Mars_Rdp.md §2.6.
    public class MarsThreadedRdpTests
    {
        static MarsThreadedRdpTests() => DpInterface.VerifyMarks = true;

        private const uint Start = MemoryMap.DpCommandBase + 0x00;
        private const uint End = MemoryMap.DpCommandBase + 0x04;
        private const uint MiInterruptRegister = MemoryMap.MiBase + 0x08;

        private const uint List = 0x0010_0000;
        private const uint Framebuffer = 0x0020_0000;
        private const uint Depth = 0x0028_0000;
        private const uint Texture = 0x0030_0000;
        private const int Width = 320, Rows = 240;

        private const ulong SyncFull = 0x29UL << 56;
        private const ulong FillCycle = (0x2FUL << 56) | (3UL << 52);

        [Fact]
        public void A_list_on_the_thread_leaves_memory_and_the_state_as_the_list_at_once()
        {
            MemoryBus atOnce = new(), threaded = new();
            threaded.Dp.Threaded = true;

            ulong[] list = Scene(0x1234_5678);
            HandOver(atOnce, list);
            HandOver(threaded, list);

            Assert.Equal(atOnce.Read32(MiInterruptRegister), threaded.Read32(MiInterruptRegister));
            Assert.Equal(atOnce.Dp.StatusWord, threaded.Dp.StatusWord);

            threaded.Dp.Join();
            Assert.Equal(atOnce.Rdram, threaded.Rdram);
            Assert.Equal(atOnce.RdramHidden, threaded.RdramHidden);
            Assert.Equal(State(atOnce), State(threaded));
        }

        // A state written while the list still runs holds the finished drawing, because writing it joins the thread.
        [Fact]
        public void A_state_written_while_the_thread_draws_holds_the_finished_drawing()
        {
            MemoryBus atOnce = new(), threaded = new();
            threaded.Dp.Threaded = true;

            for (int pass = 0; pass < 6; pass++)
            {
                ulong[] list = Scene((uint)(0x0101_0101 * (pass + 1)));
                HandOver(atOnce, list);
                HandOver(threaded, list);
                Assert.Equal(State(atOnce), State(threaded));
            }
        }

        // The pixel a read asks for is the drawn one, since the read waits for the words that marked its page.
        [Fact]
        public void A_read_of_the_image_being_drawn_waits_for_the_drawing()
        {
            var threaded = new MemoryBus();
            threaded.Dp.Threaded = true;

            for (int pass = 1; pass <= 20; pass++)
            {
                uint color = (uint)(0x1111_1111 * (pass & 0xF)) | 1;
                HandOver(threaded, Scene(color));

                Assert.NotEqual(0, threaded.Dp.MarkFor(Framebuffer + (uint)(Width * 2 * 100)));
                Assert.Equal(color, threaded.Read32(Framebuffer + (uint)(Width * 2 * 100) + 40));
                Assert.Equal(0, threaded.Dp.MarkFor(Framebuffer + (uint)(Width * 2 * 100)));
            }
        }

        [Fact]
        public void Pages_no_command_reaches_are_not_marked_and_the_images_and_texture_source_are()
        {
            var threaded = new MemoryBus();
            threaded.Dp.Threaded = true;

            HandOver(threaded, Scene(0x8000_8000, load: true));

            Assert.NotEqual(0, threaded.Dp.MarkFor(Framebuffer));
            Assert.NotEqual(0, threaded.Dp.MarkFor(Framebuffer + (uint)(Width * 2 * (Rows - 1))));
            Assert.NotEqual(0, threaded.Dp.MarkFor(Depth + (uint)(Width * 2 * 120)));
            Assert.NotEqual(0, threaded.Dp.MarkFor(Texture));
            Assert.Equal(0, threaded.Dp.MarkFor(0x0018_0000));
            Assert.Equal(0, threaded.Dp.MarkFor(0x0038_0000));
            Assert.Equal(0, threaded.Dp.MarkFor(List));

            threaded.Dp.Join();
            Assert.Equal(0, threaded.Dp.MarkFor(Framebuffer));
        }

        // The page a load reads from is marked for writers and not for readers, and a writer beside the load's bytes on that page is a bystander.
        [Fact]
        public void A_page_only_a_load_reads_stops_writers_within_the_load_and_nobody_else()
        {
            var threaded = new MemoryBus();
            threaded.Dp.Threaded = true;

            HandOver(threaded, Scene(0x8000_8000, load: true, loadRows: 16));

            Assert.NotEqual(0, threaded.Dp.Marks[Texture >> 12]);
            Assert.Equal(0, threaded.Dp.WriteMarks[Texture >> 12]);
            Assert.NotEqual(0, threaded.Dp.Marks[Framebuffer >> 12]);
            Assert.NotEqual(0, threaded.Dp.WriteMarks[Framebuffer >> 12]);

            threaded.Write32(Texture + 0x900, 0x1234_5678);
            Assert.Equal(1, threaded.Dp.Bystanders);
            Assert.NotEqual(0, threaded.Dp.Marks[Texture >> 12]);

            threaded.Write32(Texture + 0x100, 0x1234_5678);
            Assert.Equal(1, threaded.Dp.Bystanders);
            Assert.Equal(0, threaded.Dp.Marks[Texture >> 12]);

            threaded.Dp.Join();
        }

        // A list in the page after the depth image's last row is read by the interface without a wait, and drawn as at once.
        [Fact]
        public void A_list_beside_the_depth_image_is_read_without_waiting()
        {
            const uint list = Depth + Width * 2 * Rows + 0x100;
            Assert.Equal(Depth + Width * 2 * (Rows - 1) >> 12, list >> 12);

            MemoryBus atOnce = new(), threaded = new();
            threaded.Dp.Threaded = true;

            ulong[] scene = Scene(0x7C1F_7C1F);
            HandOver(atOnce, scene, list);
            HandOver(threaded, scene, list);

            Assert.Equal(0, threaded.Dp.WaitsPerSite[10]);
            Assert.True(threaded.Dp.Bystanders >= scene.Length);

            threaded.Dp.Join();
            Assert.Equal(atOnce.Rdram, threaded.Rdram);
            Assert.Equal(State(atOnce), State(threaded));
        }

        // The verifier, given a byte the processor never touched, faults on the page and on the range, and the next join throws it.
        [Fact]
        public void The_verifier_faults_a_write_outside_the_marked_pages_and_one_outside_the_marked_rows()
        {
            var threaded = new MemoryBus();
            threaded.Dp.Threaded = true;

            HandOver(threaded, Scene(0x0001_0001));
            threaded.Read32(Framebuffer);
            threaded.Dp.Wrote(0x0018_0000, threaded.Dp.Processor.RunningWord);
            var page = Assert.Throws<System.InvalidOperationException>(() => threaded.Dp.Join());
            Assert.Contains("did not mark", page.InnerException!.Message);

            HandOver(threaded, Scene(0x0002_0002));
            threaded.Read32(Framebuffer);
            Assert.NotEqual(0, threaded.Dp.WriteMarks[(Framebuffer + Width * 2 * Rows + 0x100) >> 12]);
            threaded.Dp.Wrote(Framebuffer + Width * 2 * Rows + 0x100, threaded.Dp.Processor.RunningWord);
            var range = Assert.Throws<System.InvalidOperationException>(() => threaded.Dp.Join());
            Assert.Contains("outside every range", range.InnerException!.Message);

            HandOver(threaded, Scene(0x0003_0003));
            threaded.Dp.Join();
        }

        // A state written while the thread stands mid-list carries the words it had not run, and loads to what the finished list leaves - see Mars_Rdp.md §2.7.
        [Fact]
        public void A_snapshot_taken_while_the_thread_stands_loads_to_what_the_finished_list_leaves()
        {
            MemoryBus atOnce = new(), threaded = new(), loaded = new();
            threaded.Dp.Threaded = true;
            loaded.Dp.Threaded = true;

            ulong[] list = Scene(0x2345_6789);
            HandOver(atOnce, list);

            // Held before the list is handed over, so every word of it is still pending when the snapshot is written.
            threaded.Dp.Pause();
            HandOver(threaded, list);
            Assert.Equal(0, threaded.Dp.DrainWords);

            byte[] snapshot = State(threaded, snapshot: true);
            byte[] finished = State(atOnce);
            Assert.Equal(finished.Length + 4 + 8 * DpInterface.SnapshotWords, snapshot.Length);

            Load(loaded, snapshot);
            Assert.Equal(atOnce.Rdram, loaded.Rdram);
            Assert.Equal(atOnce.RdramHidden, loaded.RdramHidden);
            Assert.Equal(finished, State(loaded));

            threaded.Dp.Join();
            Assert.Equal(finished, State(threaded));
        }

        [Fact]
        public void A_snapshot_of_a_machine_with_nothing_in_flight_is_the_state_with_an_empty_tail()
        {
            var bus = new MemoryBus();
            HandOver(bus, Scene(0x0F0F_0F0F));

            byte[] state = State(bus), snapshot = State(bus, snapshot: true);
            Assert.Equal(state.Length + 4 + 8 * DpInterface.SnapshotWords, snapshot.Length);
            Assert.Equal(state, snapshot[..state.Length]);

            var loaded = new MemoryBus();
            Load(loaded, snapshot);
            Assert.Equal(state, State(loaded));
        }

        // Several processors sharing a list leave what one leaves: the memory, its hidden bits and the state, on fills, shaded triangles and rectangles - see Mars_Rdp.md §2.8.
        [Theory]
        [InlineData(2)]
        [InlineData(3)]
        [InlineData(4)]
        public void Processors_sharing_a_list_leave_what_one_leaves(int workers)
        {
            foreach (ulong[] list in new[] { Scene(0x1234_5678), Shaded(0x11223344, toTheEdge: false), Shaded(0x55667788, toTheEdge: true), Shaded(0x99AABBCC, toTheEdge: true, twoCycle: true) })
            {
                MemoryBus atOnce = new(), threaded = new(), split = new();
                threaded.Dp.Threaded = true;
                split.Dp.Threaded = true;
                split.Dp.Workers = workers;

                HandOver(atOnce, list);
                HandOver(threaded, list);
                HandOver(split, list);

                threaded.Dp.Join();
                split.Dp.Join();
                Assert.Equal(atOnce.Rdram, split.Rdram);
                Assert.Equal(atOnce.RdramHidden, split.RdramHidden);
                Assert.Equal(State(atOnce), State(split));
                Assert.Equal(State(threaded), State(split));
                Assert.Equal(workers, split.Dp.Workers);
            }
        }

        // A load from the image being drawn, and a primitive in the mode whose carry crosses rows, are each run by every processor together or by one alone, and leave the same bytes.
        [Fact]
        public void A_load_from_the_drawn_image_and_a_live_carry_are_drawn_as_at_once()
        {
            var list = new System.Collections.Generic.List<ulong>();
            list.AddRange(Shaded(0x0F0F_0F0F, toTheEdge: true));
            list.RemoveAt(list.Count - 1);
            list.Add(Combine(4, 0, 11, 7, 4, 7, 4, 7));
            list.AddRange(Shaded(0x0F0F_0F0F, toTheEdge: true).Skip(12).SkipLast(1));
            list.Add((0x3DUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer);
            list.Add((0x35UL << 56) | (2UL << 51) | (16UL << 41));
            list.Add((0x34UL << 56) | (0UL << 44) | (0UL << 32) | ((31UL << 2) << 12) | (15UL << 2));
            list.AddRange(Shaded(0xF0F0_F0F0, toTheEdge: true, twoCycle: true, memoryAlphaFirst: true).Skip(0));
            ulong[] words = list.ToArray();

            MemoryBus atOnce = new(), split = new();
            split.Dp.Threaded = true;
            split.Dp.Workers = 3;
            HandOver(atOnce, words);
            HandOver(split, words);
            split.Dp.Join();

            Assert.True(split.Dp.Processor.HazardLoads >= 1);
            Assert.True(split.Dp.Processor.SerialisedPrimitives >= 1);
            Assert.Equal(atOnce.Rdram, split.Rdram);
            Assert.Equal(atOnce.RdramHidden, split.RdramHidden);
            Assert.Equal(State(atOnce), State(split));
        }

        // A row's last pixel past the width reads the next row's first bytes; the owner of that row reads them at its end of the primitive, before it runs on to fill them - see Mars_Rdp.md §2.8.
        [Fact]
        public void The_read_past_a_rows_end_is_made_by_the_next_rows_owner_before_it_runs_on()
        {
            ulong[] list =
            {
                FillCycle,
                (0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
                (0x3EUL << 56) | Depth,
                Scissor(0, 0, Width, Rows),
                (0x37UL << 56) | 0x0001_0001,
                FillRectangle(0, 0, Width - 1, Rows - 1),
                (0x2FUL << 56) | (3UL << 38) | (3UL << 36) | (1UL << 22) | (1UL << 20) | (1UL << 4) | (1UL << 5) | (1UL << 3) | (1UL << 6),
                Combine(4, 8, 11, 7, 4, 7, 4, 7),
                (0x36UL << 56) | ((ulong)(Width << 2) << 44) | (403UL << 32) | 400UL,
                FillCycle,
                (0x37UL << 56) | 0x7777_7777,
                FillRectangle(0, 101, Width - 1, 101),
                SyncFull,
            };

            MemoryBus atOnce = new(), split = new();
            split.Dp.Threaded = true;
            split.Dp.Workers = 2;
            HandOver(atOnce, list);
            HandOver(split, list);
            split.Dp.Join();

            Assert.Equal(1, split.Dp.AliasedReads);
            Assert.Equal(atOnce.Rdram, split.Rdram);
            Assert.Equal(State(atOnce), State(split));
        }

        // An image set one row into the last is drawn only once every processor has finished the last, since its rows are the other's rows by another count - see Mars_Rdp.md §2.8.
        [Fact]
        public void An_image_one_row_into_the_last_is_drawn_after_the_last_is_finished()
        {
            var list = new System.Collections.Generic.List<ulong>
            {
                FillCycle,
                (0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
                (0x3EUL << 56) | Depth,
                Scissor(0, 0, Width, Rows),
                (0x2FUL << 56) | (3UL << 38) | (3UL << 36) | (1UL << 22) | (1UL << 20) | (1UL << 4) | (1UL << 5) | (1UL << 3) | (1UL << 6),
                Combine(4, 8, 11, 7, 4, 7, 4, 7),
            };

            // Twenty rows' worth of shading on odd rows alone, so the odd rows' owner reaches the fill long after the even rows' owner has finished it.
            for (int i = 0; i < 20; i++) list.Add((0x36UL << 56) | ((ulong)((Width - 1) << 2) << 44) | (7UL << 32) | 4UL);
            list.Add(FillCycle);
            list.Add((0x37UL << 56) | 0x1111_1111);
            list.Add(FillRectangle(0, 0, Width - 1, Rows - 1));
            list.Add((0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | (Framebuffer + Width * 2));
            list.Add((0x37UL << 56) | 0x2222_2222);
            list.Add(FillRectangle(0, 0, Width - 1, Rows - 2));
            list.Add(SyncFull);
            ulong[] words = list.ToArray();

            MemoryBus atOnce = new(), split = new();
            split.Dp.Threaded = true;
            split.Dp.Workers = 2;
            HandOver(atOnce, words);
            HandOver(split, words);
            split.Dp.Join();

            Assert.Equal(atOnce.Rdram, split.Rdram);
            Assert.Equal(State(atOnce), State(split));
        }

        // A list handed over up to the middle of a command leaves every processor waiting for its next word; a snapshot then must not wait for a boundary that cannot come - see Mars_Rdp.md §2.8.
        [Fact]
        public void A_snapshot_while_every_processor_waits_inside_a_command_is_written_and_loads()
        {
            MemoryBus atOnce = new(), split = new(), loaded = new();
            split.Dp.Threaded = true;
            split.Dp.Workers = 2;
            loaded.Dp.Threaded = true;
            loaded.Dp.Workers = 2;

            // Both machines take the same three pieces at the same places, so the list's bytes and the registers agree when they are compared.
            ulong[] list = Shaded(0x1357_9BDF, toTheEdge: true);
            int cut = list.Length - 12;
            HandOver(atOnce, list[..cut]);
            HandOver(atOnce, list[cut..(cut + 5)], List + (uint)cut * 8);
            HandOver(atOnce, list[(cut + 5)..], List + (uint)(cut + 5) * 8);
            HandOver(split, list[..cut]);
            split.Dp.Join();
            HandOver(split, list[cut..(cut + 5)], List + (uint)cut * 8);

            byte[] snapshot = State(split, snapshot: true);
            HandOver(split, list[(cut + 5)..], List + (uint)(cut + 5) * 8);
            split.Dp.Join();
            Assert.True(atOnce.Rdram.AsSpan().SequenceEqual(split.Rdram), "the resumed machine's memory differs");
            Assert.True(State(atOnce).AsSpan().SequenceEqual(State(split)), "the resumed machine's state differs");

            Load(loaded, snapshot);
            HandOver(loaded, list[(cut + 5)..], List + (uint)(cut + 5) * 8);
            loaded.Dp.Join();
            Assert.True(atOnce.Rdram.AsSpan().SequenceEqual(loaded.Rdram), "the loaded machine's memory differs");
            Assert.True(State(atOnce).AsSpan().SequenceEqual(State(loaded)), "the loaded machine's state differs");
        }

        // A snapshot with several processors stands them all at one boundary, and its tail loads to the finished list.
        [Fact]
        public void A_snapshot_with_several_processors_stands_them_at_one_boundary()
        {
            MemoryBus atOnce = new(), split = new(), loaded = new();
            split.Dp.Threaded = true;
            split.Dp.Workers = 4;
            loaded.Dp.Threaded = true;
            loaded.Dp.Workers = 2;

            ulong[] list = Shaded(0x2468_ACE0, toTheEdge: true);
            HandOver(atOnce, list);
            split.Dp.Pause();
            HandOver(split, list);
            byte[] snapshot = State(split, snapshot: true);
            split.Dp.Resume();

            Load(loaded, snapshot);
            Assert.Equal(atOnce.Rdram, loaded.Rdram);
            Assert.Equal(State(atOnce), State(loaded));

            split.Dp.Join();
            Assert.Equal(State(atOnce), State(split));
        }

        // A one-cycle scene of shaded, depth-tested triangles over a full frame buffer; to the edge, their right edges cross a scissor as wide as the image - see Mars_Rdp.md §2.8.
        private static ulong[] Shaded(uint seed, bool toTheEdge, bool twoCycle = false, bool memoryAlphaFirst = false)
        {
            var list = new System.Collections.Generic.List<ulong>
            {
                FillCycle,
                (0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
                (0x3EUL << 56) | Depth,
                Scissor(0, 0, Width, Rows),
                (0x37UL << 56) | 0x0001_0001,
                FillRectangle(0, 0, Width - 1, Rows - 1),
                (0x37UL << 56) | 0xFFFC_FFFC,
                (0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Depth,
                FillRectangle(0, 0, Width - 1, Rows - 1),
                (0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
            };

            // One or two cycles, depth compared and updated, blending the pixel into memory by its alpha, and in the first cycle by memory alpha when asked - see Mars_RdpTwoCycle.md §4.
            ulong blend = (1UL << 22) | (1UL << 20) | (memoryAlphaFirst ? (1UL << 18) | (1UL << 16) : 0);
            list.Add((0x2FUL << 56) | ((ulong)(twoCycle ? 1 : 0) << 52) | (3UL << 38) | (3UL << 36) | blend | (1UL << 4) | (1UL << 5) | (1UL << 3) | (1UL << 6));
            list.Add(Combine(4, 8, 11, 7, 4, 7, 4, 7));

            uint state = seed;
            for (int i = 0; i < 24; i++)
            {
                double x1 = Next(ref state) % (Width + 40) - 20, y1 = Next(ref state) % (Rows + 20) - 10;
                double x2 = Next(ref state) % (Width + 40) - 20, y2 = Next(ref state) % (Rows + 20) - 10;
                double x3 = Next(ref state) % (Width + 40) - 20, y3 = Next(ref state) % (Rows + 20) - 10;
                if (toTheEdge && (i & 1) == 0) x2 = Width + 30;
                list.AddRange(Triangle(0x0C, x1, y1, x2, y2, x3, y3, Next(ref state)));
            }

            list.Add(SyncFull);
            return list.ToArray();
        }

        // Both cycles the same: (A - B) × C + D for colour and alpha; selector 0 is the previous pixel's result, which makes a primitive live - see Mars_RdpTwoCycle.md §4.
        private static ulong Combine(int a, int b, int c, int d, int alphaA, int alphaB, int alphaC, int alphaD)
        {
            ulong high = (ulong)((a << 20) | (c << 15) | (alphaA << 12) | (alphaC << 9) | (a << 5) | c);
            ulong low = (ulong)(uint)((b << 28) | (b << 24) | (alphaA << 21) | (alphaC << 18) | (d << 15) | (alphaB << 12) | (alphaD << 9) | (d << 6) | (alphaB << 3) | alphaD);
            return (0x3CUL << 56) | (high << 32) | low;
        }

        private static uint Next(ref uint state)
        {
            state = state * 1664525u + 1013904223u;
            return state >> 8;
        }

        // A triangle's edges from three corners, with its shade and depth words from a seed; any well-formed words are a valid test - see MarsRdpDifferentialTests.
        private static ulong[] Triangle(int id, double x1, double y1, double x2, double y2, double x3, double y3, uint seed)
        {
            var sorted = new[] { (X: x1, Y: y1), (X: x2, Y: y2), (X: x3, Y: y3) }.OrderBy(v => v.Y).ToArray();
            var (top, middle, bottom) = (sorted[0], sorted[1], sorted[2]);

            double major = bottom.Y > top.Y ? (bottom.X - top.X) / (bottom.Y - top.Y) : 0;
            double upper = middle.Y > top.Y ? (middle.X - top.X) / (middle.Y - top.Y) : 0;
            double lower = bottom.Y > middle.Y ? (bottom.X - middle.X) / (bottom.Y - middle.Y) : 0;

            int yh = (int)Math.Floor(top.Y * 4), ym = (int)Math.Floor(middle.Y * 4), yl = (int)Math.Floor(bottom.Y * 4);
            double rowTop = Math.Floor(top.Y);
            double xh = top.X + major * (rowTop - top.Y), xm = top.X + upper * (rowTop - top.Y), xl = middle.X + lower * (ym / 4.0 - middle.Y);
            bool majorOnLeft = middle.X > top.X + major * (middle.Y - top.Y);

            var words = new ulong[EmuSen.Cores.Nintendo.Mars.Rdp.Rdp.Length((uint)id)];
            words[0] = ((ulong)id << 56) | (majorOnLeft ? 1UL << 55 : 0) | ((ulong)(uint)(yl & 0x3FFF) << 32) | ((ulong)(uint)(ym & 0x3FFF) << 16) | (uint)(yh & 0x3FFF);
            words[1] = Edge(xl, lower);
            words[2] = Edge(xh, major);
            words[3] = Edge(xm, upper);

            // Shade: colours in the low half of their range and small steps; depth: mid-range with a small slope, so the tests draw and compare rather than clip everything.
            uint s = seed;
            for (int i = 4; i < 12; i++) words[i] = ((ulong)(Next(ref s) & 0x007F_FFFF) << 32) | (Next(ref s) & 0x0003_FFFF);
            if (words.Length > 12)
            {
                words[12] = ((ulong)(0x1000 + (Next(ref s) & 0xFFFF)) << 48) | ((ulong)(Next(ref s) & 0xFFFF) << 32) | ((ulong)(Next(ref s) & 0x3FF) << 16) | (Next(ref s) & 0xFFFF);
                words[13] = ((ulong)(Next(ref s) & 0x3FF) << 48) | ((ulong)(Next(ref s) & 0xFFFF) << 32) | (Next(ref s) & 0x03FF_FFFF);
            }

            return words;
        }

        private static ulong Edge(double x, double slope) =>
            ((ulong)(uint)(int)Math.Round(x * 65536) << 32) | (uint)(int)Math.Round(Math.Clamp(slope, -8192, 8191) * 65536);

        // A full frame buffer of fill rectangles, with a depth image set and, when asked, a texture loaded from RDRAM.
        private static ulong[] Scene(uint color, bool load = false, int loadRows = 32)
        {
            var list = new System.Collections.Generic.List<ulong>
            {
                FillCycle,
                (0x3FUL << 56) | (0UL << 53) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
                (0x3EUL << 56) | Depth,
                Scissor(0, 0, Width, Rows),
                (0x37UL << 56) | color,
            };

            for (uint row = 0; row < Rows; row += 8) list.Add(FillRectangle(0, row, Width - 1, row + 7));

            if (load)
            {
                list.Add((0x3DUL << 56) | (0UL << 53) | (2UL << 51) | (63UL << 32) | Texture);
                list.Add((0x35UL << 56) | (2UL << 51) | (16UL << 41));
                list.Add((0x34UL << 56) | (0UL << 44) | (0UL << 32) | ((31UL << 2) << 12) | ((ulong)(loadRows - 1) << 2));
            }

            list.Add(SyncFull);
            return list.ToArray();
        }

        private static void HandOver(MemoryBus bus, ulong[] list, uint at = List)
        {
            for (int i = 0; i < list.Length; i++) bus.Write64(at + (uint)i * 8, list[i]);
            bus.Write32(Start, at);
            bus.Write32(End, at + (uint)list.Length * 8);
        }

        private static byte[] State(MemoryBus bus, bool snapshot = false)
        {
            using var stream = new MemoryStream();
            using var writer = new BinaryWriter(stream);
            bus.WriteState(writer, snapshot);
            writer.Flush();
            return stream.ToArray();
        }

        private static void Load(MemoryBus bus, byte[] snapshot)
        {
            using var stream = new MemoryStream(snapshot);
            using var reader = new BinaryReader(stream);
            bus.ReadState(reader, snapshot: true);
        }

        private static ulong Scissor(uint left, uint top, uint right, uint bottom) =>
            (0x2DUL << 56) | ((ulong)(left << 2) << 44) | ((ulong)(top << 2) << 32) | ((ulong)(right << 2) << 12) | (bottom << 2);

        private static ulong FillRectangle(uint left, uint top, uint right, uint bottom) =>
            (0x36UL << 56) | ((ulong)(right << 2) << 44) | ((ulong)(bottom << 2) << 32) | ((ulong)(left << 2) << 12) | (top << 2);
    }
}
