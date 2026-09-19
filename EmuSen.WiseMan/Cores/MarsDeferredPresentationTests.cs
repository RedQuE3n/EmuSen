using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Vi;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The scan-out walked later over captured lines is the scan-out walked at once - see Mars_Video.md §2.7.
    public class MarsDeferredPresentationTests
    {
        private const uint Framebuffer = 0x0020_0000;
        private const uint GammaOn = 1 << 3, DivotOn = 1 << 4, DitherFilter = 1 << 16;

        private const int Zero = 0, A0 = 4, A1 = 5, T0 = 8, T1 = 9, T2 = 10;

        // Every geometry and mode the reference test scans, walked from a capture after the live frame buffer was overwritten.
        [Fact]
        public void A_walk_over_captured_lines_matches_the_walk_over_memory_for_every_reference_geometry()
        {
            uint[] shorter = Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0);
            uint[] blank = Registers(0, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
            uint[][] scans =
            {
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0),
                Registers(2, 2, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40),
                Registers(3, 2, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0, serrate: true, currentLine: 1),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0, serrate: true),
                shorter, shorter, shorter,
                Registers(0, 3, 64, 0x400, 0x400, 148, 200, 34, 120, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 40, 640, 34, 120, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 128, 640, 44, 240, 0, 0, sync: 625),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, origin: 0),
                blank, blank,
                Registers(2, 3, 64, 0x400, 0x400, 108, 0, 34, 120, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0),
                Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0),
                Registers(3, 1, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0),
                Registers(2, 0, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40),
                Registers(2, 1, 64, 0x400, 0x200, 108, 256, 34, 110, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DitherFilter),
                Registers(3, 3, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0, control: DitherFilter),
                Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DivotOn),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: GammaOn),
                Registers(2, 1, 64, 0x2AB, 0x155, 108, 256, 34, 120, 0x80, 0x40, control: DitherFilter | DivotOn | GammaOn),
                Registers(2, 0, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0, control: DitherFilter | DivotOn | GammaOn),
                Registers(2, 0, 320, 0x200, 0x355, 108, 640, 44, 288, 0, 0, sync: 625, control: DitherFilter | DivotOn),
            };

            MemoryBus atOnce = Noisy(), later = Noisy();
            var job = new ScanJob();

            foreach (uint[] registers in scans)
            {
                Program(atOnce, registers);
                Program(later, registers);

                bool walked = atOnce.Vi.Scan();
                Assert.Equal(walked, later.Vi.Prepare(job));

                if (walked)
                {
                    later.Vi.Capture(job);
                    Array.Fill(later.Rdram, (byte)0xA5, (int)Framebuffer - 0x8000, 0x20000);
                    Array.Fill(later.RdramHidden, (byte)1, (int)Framebuffer / 2 - 0x4000, 0x10000);
                    later.Vi.Walk(job);
                    Array.Copy(atOnce.Rdram, later.Rdram, atOnce.Rdram.Length);
                    Array.Copy(atOnce.RdramHidden, later.RdramHidden, atOnce.RdramHidden.Length);
                }

                Assert.Equal(atOnce.Vi.Frame.ToArray(), later.Vi.Frame.ToArray());
            }
        }

        // The deferred core's picture after a frame is the immediate core's picture after the frame before, and their machines are the same.
        [Fact]
        public void The_deferred_picture_is_the_previous_frame_of_the_immediate_one_and_the_machines_agree()
        {
            string rom = SyntheticN64Rom.WriteTemp(Painter());
            try
            {
                var atOnce = new MarsCore();
                var deferred = new MarsCore { DeferredPresentation = true };
                atOnce.LoadRom(rom);
                deferred.LoadRom(rom);

                byte[]? previous = null;
                for (int frame = 1; frame <= 8; frame++)
                {
                    atOnce.RunFrame();
                    deferred.RunFrame();

                    Assert.Equal(State(atOnce), State(deferred));
                    if (previous is not null)
                    {
                        Assert.Equal(previous, deferred.GetFrameBufferRgba());
                        Assert.Equal(atOnce.ScreenHeight, deferred.ScreenHeight);
                    }

                    byte[] current = atOnce.GetFrameBufferRgba().ToArray();
                    if (previous is not null) Assert.NotEqual(previous, current);
                    previous = current;
                }

                deferred.DeferredPresentation = false;
                Assert.Equal(previous, deferred.GetFrameBufferRgba());
            }
            finally
            {
                File.Delete(rom);
            }
        }

        // Both cores load the same state, since a load drops the undrained audio a straight run keeps - see Mars_Audio.md.
        [Fact]
        public void A_loaded_state_is_presented_at_once_in_deferred_mode()
        {
            string rom = SyntheticN64Rom.WriteTemp(Painter());
            try
            {
                var source = new MarsCore();
                source.LoadRom(rom);
                for (int frame = 0; frame < 5; frame++) source.RunFrame();
                byte[] saved = State(source);

                var atOnce = new MarsCore();
                atOnce.LoadRom(rom);
                atOnce.LoadState(new MemoryStream(saved));

                var deferred = new MarsCore { DeferredPresentation = true };
                deferred.LoadRom(rom);
                deferred.RunFrame();
                deferred.LoadState(new MemoryStream(saved));

                Assert.Equal(atOnce.GetFrameBufferRgba(), deferred.GetFrameBufferRgba());
                Assert.Equal(source.GetFrameBufferRgba(), deferred.GetFrameBufferRgba());

                atOnce.RunFrame();
                deferred.RunFrame();
                Assert.Equal(State(atOnce), State(deferred));
                Assert.Equal(source.GetFrameBufferRgba(), deferred.GetFrameBufferRgba());

                deferred.RunFrame();
                Assert.Equal(atOnce.GetFrameBufferRgba(), deferred.GetFrameBufferRgba());
            }
            finally
            {
                File.Delete(rom);
            }
        }

        private static byte[] State(MarsCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // Programs the VI for a 320 by 240 picture with every pass on, then paints the frame buffer with a word that advances each pass.
        private static byte[] Painter()
        {
            uint[] registers = Registers(2, 0, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0, control: DitherFilter | DivotOn | GammaOn);
            registers[7] = 0xC15;

            var program = new MipsAssembler().Lui(A0, 0xA440);
            for (int i = 0; i < registers.Length; i++)
            {
                program.Lui(T0, (ushort)(registers[i] >> 16)).Ori(T0, T0, (ushort)registers[i]).Sw(T0, A0, (short)(i * 4));
            }

            // loop: a1 = frame buffer, t1 = words, t2 += 0x01010101; fill: sw t2; a1 += 4; t1 -= 1; bne fill; b loop.
            program.Lui(A1, 0xA020).Ori(T1, Zero, 0x9600).Lui(T0, 0x0101).Ori(T0, T0, 0x0101).Addu(T2, T2, T0)
                .Sw(T2, A1, 0).Addiu(A1, A1, 4).Addiu(T1, T1, -1).Bne(T1, Zero, -4).Nop()
                .Beq(Zero, Zero, -11).Nop();

            return SyntheticN64Rom.BuildRunningFromRdram(program.ToArray());
        }

        // A second scan of the same registers over the same bytes is reported as a repeat, and the capture it kept is still that scan's - see Mars_Video.md §2.8.
        [Fact]
        public void A_scan_that_would_repeat_the_last_one_says_so_and_its_capture_still_walks()
        {
            uint[] registers = Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
            MemoryBus atOnce = Noisy(), later = Noisy();
            var job = new ScanJob();

            Program(atOnce, registers);
            Program(later, registers);

            Assert.True(atOnce.Vi.Scan());
            Assert.True(later.Vi.Prepare(job));
            later.Vi.Capture(job);
            Assert.False(job.Repeats);
            later.Vi.Walk(job);
            Assert.Equal(atOnce.Vi.Frame.ToArray(), later.Vi.Frame.ToArray());

            // Nothing wrote to the frame buffer, so the scan repeats and no bytes are taken; a walk over the kept capture is still right.
            Assert.True(later.Vi.Prepare(job));
            later.Vi.Capture(job);
            Assert.True(job.Repeats);
            Assert.True(job.Captured);

            System.Array.Fill(later.Rdram, (byte)0xA5, (int)Framebuffer - 0x8000, 0x20000);
            later.Vi.Walk(job);
            Assert.Equal(atOnce.Vi.Frame.ToArray(), later.Vi.Frame.ToArray());

            // One byte of the reachable range is enough to end the repeat.
            System.Array.Copy(atOnce.Rdram, later.Rdram, atOnce.Rdram.Length);
            later.Rdram[job.From + (uint)job.Count / 2] ^= 0xFF;
            Assert.True(later.Vi.Prepare(job));
            later.Vi.Capture(job);
            Assert.False(job.Repeats);
        }

        // The anti-alias filter reads the hidden bits beside each word, so a change in those alone must end the repeat too - see Mars_Video.md §2.8.
        [Fact]
        public void A_change_in_the_hidden_bits_alone_ends_the_repeat()
        {
            uint[] registers = Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
            var bus = Noisy();
            var job = new ScanJob();

            Program(bus, registers);
            Assert.True(bus.Vi.Prepare(job));
            bus.Vi.Capture(job);
            Assert.False(job.Repeats);

            Assert.True(bus.Vi.Prepare(job));
            bus.Vi.Capture(job);
            Assert.True(job.Repeats);

            bus.RdramHidden[(job.From >> 1) + (uint)job.Count / 4] ^= 3;
            Assert.True(bus.Vi.Prepare(job));
            bus.Vi.Capture(job);
            Assert.False(job.Repeats);
        }

        // A loaded state's raster is not what the last walk wrote, so the scan after one is never counted a repeat - see §2.8.
        [Fact]
        public void A_loaded_state_makes_the_next_scan_walk()
        {
            uint[] registers = Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
            var bus = Noisy();
            var job = new ScanJob();

            Program(bus, registers);
            Assert.True(bus.Vi.Prepare(job));
            bus.Vi.Capture(job);
            Assert.False(job.Repeats);

            Assert.True(bus.Vi.Prepare(job));
            bus.Vi.Capture(job);
            Assert.True(job.Repeats);

            job.Forget();
            Assert.True(bus.Vi.Prepare(job));
            bus.Vi.Capture(job);
            Assert.False(job.Repeats);
        }

        private static void Program(MemoryBus bus, uint[] registers)
        {
            for (int i = 0; i < registers.Length; i++) bus.Write32(MemoryMap.ViBase + (uint)i * 4, registers[i]);
        }

        private static MemoryBus Noisy()
        {
            var bus = new MemoryBus();

            uint state = 0x2468_ACE0;
            for (uint i = 0; i < 0x30000; i++)
            {
                state = state * 1103515245 + 12345;
                bus.Write8(Framebuffer - 0x8000 + i, (byte)(state >> 16));
            }

            uint seed = 0x1B4E_81B4;
            for (int i = 0; i < 0x18000; i++)
            {
                seed = seed * 1664525 + 1013904223;
                bus.RdramHidden[Framebuffer / 2 - 0x4000 + i] = (byte)(seed >> 30);
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
