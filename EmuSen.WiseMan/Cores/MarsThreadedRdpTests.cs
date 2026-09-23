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

                long narrowed = threaded.Dp.ReadsNarrowed;
                Assert.NotEqual(0, threaded.Dp.MarkFor(Framebuffer + (uint)(Width * 2 * 100)));
                Assert.Equal(color, threaded.Read32(Framebuffer + (uint)(Width * 2 * 100) + 40));

                // It waits for the draw whose box holds it, not the batch, so the page stays marked for the rows drawn after - see Mars_Rdp.md §2.6.3.
                Assert.True(threaded.Dp.ReadsNarrowed > narrowed || threaded.Dp.MarkFor(Framebuffer + (uint)(Width * 2 * 100)) == 0);
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
            Assert.NotEqual(0, threaded.Dp.MarkFor(Texture));

            // Fill mode never reads or writes depth, so the depth image is not marked; with depth on, it is - see Mars_Rdp.md §2.6.2.
            Assert.Equal(0, threaded.Dp.MarkFor(Depth + (uint)(Width * 2 * 120)));
            threaded.Dp.Join();
            HandOver(threaded, Scene(0x8000_8000, load: true, depth: true));
            Assert.NotEqual(0, threaded.Dp.MarkFor(Depth + (uint)(Width * 2 * 120)));
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

            ulong[] scene = Scene(0x7C1F_7C1F, depth: true);
            HandOver(atOnce, scene, list);
            HandOver(threaded, scene, list);

            // Words read before the last rows are shadowed find the page unmarked; those after find it marked and are bystanders - see Mars_Rdp.md §2.6.2.
            Assert.Equal(0, threaded.Dp.WaitsPerSite[10]);
            Assert.True(threaded.Dp.Bystanders > 0);

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

        // Fills alternating over four images, so every other command is a barrier every processor must reach - see Mars_Rdp.md §2.8.
        private static ulong[] Barriers(int fills)
        {
            var list = new System.Collections.Generic.List<ulong> { FillCycle, Scissor(0, 0, Width, Rows), (0x37UL << 56) | 0x0F0F_0F0F };
            for (uint i = 0; i < fills; i++)
            {
                list.Add((0x3FUL << 56) | (2UL << 51) | ((ulong)(Width - 1) << 32) | (Framebuffer + (i % 4) * Width * 2));
                uint row = i * 3 % (Rows - 2);
                list.Add(FillRectangle(0, row, Width - 1, row + 1));
            }
            list.Add(SyncFull);
            return list.ToArray();
        }

        // A pause that finds some processors waiting at a barrier and the rest short of it must still be answered - see Mars_Rdp.md §2.8.
        [Fact]
        public void A_pause_that_finds_some_processors_at_a_barrier_and_the_rest_short_of_it_is_answered()
        {
            ulong[] list = Barriers(1500);
            MemoryBus atOnce = new();
            HandOver(atOnce, list);
            byte[] finished = State(atOnce);
            long raised = 0;

            for (int attempt = 0; attempt < 100; attempt++)
            {
                var bus = new MemoryBus();
                bus.Dp.Threaded = true;
                bus.Dp.Workers = 4;
                HandOver(bus, list);
                System.Threading.Thread.SpinWait(attempt * 997 % 20_000);
                var pause = System.Threading.Tasks.Task.Run(() => bus.Dp.Pause());
                if (!pause.Wait(TimeSpan.FromSeconds(5)))
                {
                    bus.Dp.Resume();
                    Assert.Fail($"four processors: the pause at attempt {attempt} was never answered");
                }
                bus.Dp.Resume();
                bus.Dp.Join();
                Assert.Equal(finished, State(bus));
                raised += bus.Dp.PausesRaised;
                bus.Dp.Threaded = false;
            }

            Assert.True(raised > 0, "no pause found a processor at a barrier, so the case was not reached");
        }

        // Pauses, resumes and snapshots from another thread while two to four processors run barriers, under a watchdog; every snapshot loads to the finished list.
        [Theory]
        [InlineData(2)]
        [InlineData(3)]
        [InlineData(4)]
        public void Pauses_and_snapshots_while_the_processors_run_barriers_are_all_answered_and_exact(int workers)
        {
            ulong[] list = Barriers(3000);
            int pending = 0;
            long raised = 0;

            for (int round = 0; round < 8; round++)
            {
                // Both machines take the same two pieces at the same place, so their registers agree.
                int cut = list.Length / 3 + round * 97;
                MemoryBus atOnce = new();
                HandOver(atOnce, list[..cut]);
                HandOver(atOnce, list[cut..], List + (uint)cut * 8);
                byte[] finished = State(atOnce);

                var bus = new MemoryBus();
                bus.Dp.Threaded = true;
                bus.Dp.Workers = workers;
                var taken = new System.Collections.Generic.List<(byte[] Snapshot, bool Whole)>();
                var run = System.Threading.Tasks.Task.Run(() =>
                {
                    HandOver(bus, list[..cut]);
                    for (int i = 0; i < 40; i++)
                    {
                        System.Threading.Thread.SpinWait((round * 40 + i) * 613 % 12_000);
                        if (i % 4 == 3) taken.Add((State(bus, snapshot: true), i > 20));
                        else { bus.Dp.Pause(); bus.Dp.Resume(); }
                        if (i == 20) HandOver(bus, list[cut..], List + (uint)cut * 8);
                    }
                    bus.Dp.Join();
                });
                if (!run.Wait(TimeSpan.FromSeconds(30)))
                {
                    bus.Dp.Resume();
                    Assert.Fail($"{workers} processors, round {round}: a pause or a snapshot was never answered");
                }
                run.GetAwaiter().GetResult();
                Assert.Equal(finished, State(bus));
                raised += bus.Dp.PausesRaised;
                bus.Dp.Threaded = false;

                foreach (var (snapshot, whole) in taken)
                {
                    var loaded = new MemoryBus();
                    if (BitConverter.ToInt32(snapshot, snapshot.Length - 4 - 8 * DpInterface.SnapshotWords) > 0) pending++;
                    Load(loaded, snapshot);
                    if (!whole) HandOver(loaded, list[cut..], List + (uint)cut * 8);
                    Assert.Equal(finished, State(loaded));
                }
            }

            Assert.True(pending > 0, "no snapshot was taken with words still pending, so the case was not reached");
            Assert.True(raised > 0, "no pause found a processor at a barrier, so the case was not reached");
        }

        // A range read whose first bytes no pending draw holds still waits for the draws that hold the rest of it - see Mars_Rdp.md §2.6.3.
        [Fact]
        public void A_range_read_whose_first_bytes_no_draw_holds_still_waits_for_the_draws_that_hold_the_rest()
        {
            const uint page = Framebuffer + 0x1000, row = Framebuffer + Width * 2 * 8;
            ulong[] list =
            {
                FillCycle,
                (0x3FUL << 56) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
                Scissor(0, 0, Width, Rows),
                (0x37UL << 56) | 0x1234_5678,
                FillRectangle(0, 8, Width - 1, 9),
                SyncFull,
            };

            MemoryBus atOnce = new(), threaded = new();
            threaded.Dp.Threaded = true;
            HandOver(atOnce, list);
            threaded.Dp.Pause();
            HandOver(threaded, list);
            Assert.Equal(list.Length, threaded.Dp.Pending);

            var read = System.Threading.Tasks.Task.Run(() =>
            {
                threaded.Dp.WaitForReadRange(page, 0x1000, 8);
                return threaded.Rdram.AsSpan((int)row, 0x80).ToArray();
            });
            bool early = read.Wait(TimeSpan.FromMilliseconds(500));
            threaded.Dp.Resume();
            byte[] seen = read.Result;
            threaded.Dp.Join();

            Assert.False(early, "the range read returned while the draw holding its later rows was still pending");
            Assert.Equal(atOnce.Rdram.AsSpan((int)row, 0x80).ToArray(), seen);
            Assert.Equal(atOnce.Rdram, threaded.Rdram);
        }

        // A one-cycle scene of shaded, depth-tested triangles over a full frame buffer; to the edge, their right edges cross a scissor as wide as the image - see Mars_Rdp.md §2.8.
        private static ulong[] Shaded(uint seed, bool toTheEdge, bool twoCycle = false, bool memoryAlphaFirst = false, bool gentle = false)
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

            // Corners up to twenty pixels off the image on any side; the gentle scene's on quarter pixels, so every row top is walked from - see §11.
            uint state = seed;
            for (int i = 0; i < 24; i++)
            {
                double x1 = Corner(ref state, Width, 20, gentle), y1 = Corner(ref state, Rows, 10, gentle);
                double x2 = Corner(ref state, Width, 20, gentle), y2 = Corner(ref state, Rows, 10, gentle);
                double x3 = Corner(ref state, Width, 20, gentle), y3 = Corner(ref state, Rows, 10, gentle);
                if (toTheEdge && (i & 1) == 0) x2 = Width + 30;
                list.AddRange(Triangle(0x0C, x1, y1, x2, y2, x3, y3, Next(ref state), gentle));
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

        private static double Corner(ref uint state, int extent, int beyond, bool quarters) =>
            (int)(Next(ref state) % (uint)(extent + 2 * beyond)) - beyond + (quarters ? (Next(ref state) & 3) / 4.0 : 0);

        private static uint Next(ref uint state)
        {
            state = state * 1664525u + 1013904223u;
            return state >> 8;
        }

        // A triangle's edges from three corners, with its shade and depth words from a seed; any well-formed words are a valid test - see MarsRdpDifferentialTests.
        private static ulong[] Triangle(int id, double x1, double y1, double x2, double y2, double x3, double y3, uint seed, bool gentle = false)
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
            if (gentle) GentleShade(words, ref s);
            else for (int i = 4; i < 12; i++) words[i] = ((ulong)(Next(ref s) & 0x007F_FFFF) << 32) | (Next(ref s) & 0x0003_FFFF);
            if (words.Length > 12)
            {
                words[12] = ((ulong)(0x1000 + (Next(ref s) & 0xFFFF)) << 48) | ((ulong)(Next(ref s) & 0xFFFF) << 32) | ((ulong)(Next(ref s) & 0x3FF) << 16) | (Next(ref s) & 0xFFFF);
                words[13] = ((ulong)(Next(ref s) & 0x3FF) << 48) | ((ulong)(Next(ref s) & 0xFFFF) << 32) | (Next(ref s) & 0x03FF_FFFF);
            }

            return words;
        }

        // Gentle: colours in the low half of their range, alpha full, and every step within a quarter of a level a pixel, so two multiples should agree to the edges - see §11.
        private static void GentleShade(ulong[] words, ref uint s)
        {
            int[] value = new int[4], dx = new int[4], de = new int[4], dy = new int[4];
            for (int c = 0; c < 4; c++)
            {
                value[c] = c == 3 ? 0xFF << 16 : (int)((Next(ref s) & 0x7F) << 16) | (int)(Next(ref s) & 0xFFFF);
                dx[c] = c == 3 ? 0 : (int)(Next(ref s) & 0x7FFF) - 0x4000;
                de[c] = c == 3 ? 0 : (int)(Next(ref s) & 0x7FFF) - 0x4000;
                dy[c] = c == 3 ? 0 : (int)(Next(ref s) & 0x7FFF) - 0x4000;
            }

            (words[4], words[6]) = (Pack(value, 16), Pack(value, 0));
            (words[5], words[7]) = (Pack(dx, 16), Pack(dx, 0));
            (words[8], words[10]) = (Pack(de, 16), Pack(de, 0));
            (words[9], words[11]) = (Pack(dy, 16), Pack(dy, 0));
        }

        private static ulong Pack(int[] v, int shift) =>
            ((ulong)(uint)((v[0] >> shift) & 0xFFFF) << 48) | ((ulong)(uint)((v[1] >> shift) & 0xFFFF) << 32) | ((ulong)(uint)((v[2] >> shift) & 0xFFFF) << 16) | (uint)((v[3] >> shift) & 0xFFFF);

        private static ulong Edge(double x, double slope) =>
            ((ulong)(uint)(int)Math.Round(x * 65536) << 32) | (uint)(int)Math.Round(Math.Clamp(slope, -8192, 8191) * 65536);

        // Drawing at a multiple leaves the machine's memory and state exactly as drawing at one, at once and shared - see Mars_Rdp.md §11.
        [Theory]
        [InlineData(2, 1)]
        [InlineData(2, 3)]
        [InlineData(3, 2)]
        public void Drawing_at_a_multiple_leaves_the_machines_memory_as_at_one(int scale, int workers)
        {
            foreach (ulong[] list in new[] { Scene(0x1234_1234), Shaded(0x5566_7788, toTheEdge: true), Shaded(0x99AA_BBCC, toTheEdge: true, twoCycle: true) })
            {
                MemoryBus atOnce = new(), scaled = new();
                scaled.Dp.Scale = scale;
                if (workers > 1) { scaled.Dp.Threaded = true; scaled.Dp.Workers = workers; }

                HandOver(atOnce, list);
                HandOver(scaled, list);
                scaled.Dp.Join();

                Assert.Equal(atOnce.Rdram, scaled.Rdram);
                Assert.Equal(atOnce.RdramHidden, scaled.RdramHidden);
                Assert.Equal(State(atOnce), State(scaled));
                Assert.True(scaled.Dp.ScaledDrawn);
            }
        }

        // A state read replaces the processors at the multiple, and the threads must draw with the new ones or the multiple never shows again - see §11.
        [Fact]
        public void After_a_state_is_read_the_multiple_is_drawn_and_shown_again()
        {
            MemoryBus bus = new();
            bus.Dp.Threaded = true;
            bus.Dp.Workers = 3;
            bus.Dp.Scale = 2;
            HandOver(bus, Scene(0x1234_1234));
            State(bus);
            Assert.True(bus.Dp.ScaledDrawn);

            Load(bus, State(bus, snapshot: true));
            Assert.False(bus.Dp.ScaledDrawn);
            HandOver(bus, Scene(0x4321_4321));
            State(bus);
            Assert.True(bus.Dp.ScaledDrawn);
        }

        // A fill at a multiple lays the same pixel over every pixel of the multiple, at the address the multiple's square gives its image - see §11.
        [Fact]
        public void A_fill_at_a_multiple_is_the_fill_at_one_at_every_pixel_of_the_multiple()
        {
            const int scale = 2;
            MemoryBus atOnce = new(), scaled = new();
            scaled.Dp.Scale = scale;
            HandOver(atOnce, Scene(0x1234_1234));
            HandOver(scaled, Scene(0x1234_1234));

            byte[] wide = scaled.Dp.ScaledRdram;
            long image = (long)Framebuffer * scale * scale;
            for (int y = 0; y < Rows; y++)
            {
                for (int x = 0; x < Width; x++)
                {
                    int native = (int)(Framebuffer + (y * Width + x) * 2);
                    for (int j = 0; j < scale; j++)
                    {
                        for (int i = 0; i < scale; i++)
                        {
                            long at = image + ((long)(y * scale + j) * (Width * scale) + x * scale + i) * 2;
                            Assert.True(atOnce.Rdram[native] == wide[at] && atOnce.Rdram[native + 1] == wide[at + 1], $"pixel {x},{y} of the multiple {i},{j} differs");
                        }
                    }
                }
            }
        }

        // A copy-mode texture rectangle, as a game blits its HUD, lays each console pixel's texel over that pixel's square of the multiple - see §11.2.
        [Theory]
        [InlineData(2)]
        [InlineData(3)]
        [InlineData(4)]
        public void A_copy_at_a_multiple_is_the_copy_at_one_at_every_pixel_of_the_multiple(int scale)
        {
            MemoryBus atOnce = new(), scaled = new();
            scaled.Dp.Scale = scale;
            foreach (MemoryBus bus in new[] { atOnce, scaled })
                for (uint texel = 0; texel < 32 * 32; texel += 2) bus.Write32(Texture + texel * 2, (((texel << 1) | 1) << 16) | (((texel + 1) << 1) | 1));

            ulong[] list = Copy();
            HandOver(atOnce, list);
            HandOver(scaled, list);

            byte[] wide = scaled.Dp.ScaledRdram;
            long image = (long)Framebuffer * scale * scale;
            int differing = 0;
            string first = "";
            for (int y = 0; y < Rows * scale; y++)
            {
                for (int x = 0; x < Width * scale; x++)
                {
                    long at = image + ((long)y * (Width * scale) + x) * 2;
                    int actual = (wide[at] << 8) | wide[at + 1], expected = Console(x, y);
                    if (expected == actual) continue;

                    // A third of a step is truncated, so at three a texel the console lands on exactly begins one pixel of the multiple late - see §11.2.
                    if (scale == 3 && (actual == Console(x - 1, y) || actual == Console(x, y - 1) || actual == Console(x - 1, y - 1))) continue;
                    if (differing++ == 0) first = $"pixel {x},{y} of the multiple is {actual:X4} where the console's {x / scale},{y / scale} is {expected:X4}";
                }
            }

            Assert.True(atOnce.Rdram[Framebuffer + (20 * Width + 41) * 2 + 1] == 3, "the console's copy did not draw the texture");
            Assert.True(differing == 0, $"{differing} pixels of the multiple differ; the first: {first}");

            int Console(int x, int y)
            {
                int native = (int)(Framebuffer + ((Math.Max(y, 0) / scale) * Width + Math.Max(x, 0) / scale) * 2);
                return (atOnce.Rdram[native] << 8) | atOnce.Rdram[native + 1];
            }
        }

        // Rectangles of a 32×32 sixteen-bit texture, four texels a group: from the tile's corner, and from an odd column and texel.
        private static ulong[] Copy()
        {
            var list = new System.Collections.Generic.List<ulong>
            {
                FillCycle,
                (0x3FUL << 56) | (2UL << 51) | ((ulong)(Width - 1) << 32) | Framebuffer,
                Scissor(0, 0, Width, Rows),
                (0x37UL << 56) | 0xFFFE_FFFE,
                FillRectangle(0, 0, Width - 1, Rows - 1),
                (0x3DUL << 56) | (2UL << 51) | (31UL << 32) | Texture,
                (0x35UL << 56) | (2UL << 51) | (8UL << 41),
                (0x34UL << 56) | ((31UL << 2) << 12) | (31UL << 2),
                (0x2FUL << 56) | (2UL << 52),
            };

            foreach ((ulong id, uint left, uint top, uint width, uint s, uint dsdx, uint dtdy) in new[]
            {
                (0x24UL, 40u, 20u, 32u, 0u, 4u << 10, 1u << 10),
                (0x24UL, 101u, 30u, 27u, 5u << 5, 4u << 10, 1u << 10),
                (0x24UL, 200u, 100u, 32u, 0u, 4u << 10, 1u << 10),
            })
            {
                uint right = left + width - 1, bottom = top + 31;
                list.Add((id << 56) | ((ulong)(right << 2) << 44) | ((ulong)(bottom << 2) << 32) | ((ulong)(left << 2) << 12) | (top << 2));
                list.Add(((ulong)s << 48) | ((ulong)dsdx << 16) | dtdy);
            }

            list.Add(SyncFull);
            return list.ToArray();
        }

        // A shaded scene at a multiple, averaged back to the console's pixels, is the scene at one to within a tenth of a level, its edges apart - see §11.
        [Theory]
        [InlineData(2)]
        [InlineData(3)]
        [InlineData(4)]
        public void A_shaded_scene_at_a_multiple_averages_back_close_to_the_scene_at_one(int scale)
        {
            MemoryBus atOnce = new(), scaled = new();
            scaled.Dp.Scale = scale;
            ulong[] list = Shaded(0x2468_ACE0, toTheEdge: true, gentle: true);
            HandOver(atOnce, list);
            HandOver(scaled, list);

            byte[] wide = scaled.Dp.ScaledRdram;
            long image = (long)Framebuffer * scale * scale;
            double total = 0; int counted = 0, far = 0, brighter = 0, darker = 0, untouchedNative = 0, untouchedScaled = 0;
            var bands = new int[8, 8];
            for (int y = 0; y < Rows; y++)
            {
                for (int x = 0; x < Width; x++)
                {
                    int native = (int)(Framebuffer + (y * Width + x) * 2);
                    int nativePixel = (atOnce.Rdram[native] << 8) | atOnce.Rdram[native + 1];
                    (int r, int g, int b) = Channels(nativePixel);
                    int sr = 0, sg = 0, sb = 0, sameAsClear = 0;
                    for (int j = 0; j < scale; j++)
                    {
                        for (int i = 0; i < scale; i++)
                        {
                            long at = image + ((long)(y * scale + j) * (Width * scale) + x * scale + i) * 2;
                            int scaledPixel = (wide[at] << 8) | wide[at + 1];
                            if (scaledPixel == 0x0001) sameAsClear++;
                            (int wr, int wg, int wb) = Channels(scaledPixel);
                            sr += wr; sg += wg; sb += wb;
                        }
                    }
                    double area = scale * scale;
                    double d = (Math.Abs(r - sr / area) + Math.Abs(g - sg / area) + Math.Abs(b - sb / area)) / 3;
                    total += d; counted++;
                    if (nativePixel == 0x0001) untouchedNative++;
                    if (sameAsClear == area) untouchedScaled++;
                    if (d > 8) { far++; bands[y * 8 / Rows, x * 8 / Width]++; if (sr + sg + sb > (r + g + b) * 4) brighter++; else darker++; }
                }
            }

            double mean = total / counted;
            string map = string.Join(" / ", Enumerable.Range(0, 8).Select(row => string.Join(",", Enumerable.Range(0, 8).Select(col => bands[row, col].ToString()))));
            string report = $"mean {mean:F2}; {far} of {counted} beyond eight ({brighter} brighter, {darker} darker); untouched native {untouchedNative} scaled {untouchedScaled}; bands {map}";
            Assert.True(mean < 0.25 && far < counted / 2000, report);

            static (int, int, int) Channels(int pixel) => ((pixel >> 11) & 0x1F, (pixel >> 6) & 0x1F, (pixel >> 1) & 0x1F);
        }

        // A full frame buffer of fill rectangles, with a depth image set and, when asked, a texture loaded from RDRAM.
        // One cycle with depth compared and updated draws the same rectangles through the depth image, which fill mode never touches.
        private const ulong DepthCycle = (0x2FUL << 56) | (0UL << 52) | (3UL << 4);

        private static ulong[] Scene(uint color, bool load = false, int loadRows = 32, bool depth = false)
        {
            var list = new System.Collections.Generic.List<ulong>
            {
                depth ? DepthCycle : FillCycle,
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
