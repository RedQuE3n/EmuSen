using System.IO;
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
            threaded.Dp.Wrote(0x0018_0000);
            var page = Assert.Throws<System.InvalidOperationException>(() => threaded.Dp.Join());
            Assert.Contains("did not mark", page.InnerException!.Message);

            HandOver(threaded, Scene(0x0002_0002));
            threaded.Read32(Framebuffer);
            Assert.NotEqual(0, threaded.Dp.WriteMarks[(Framebuffer + Width * 2 * Rows + 0x100) >> 12]);
            threaded.Dp.Wrote(Framebuffer + Width * 2 * Rows + 0x100);
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
