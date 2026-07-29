using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;

namespace EmuSen.WiseMan.Common
{
    // Driven by a stand-in core, so these test the chain not the SNES - see EmuSen_Rewind_And_FastForward.md §1.
    public class RewindBufferTests
    {
        // State is Counter + a RAM block, so a test can assert which frame it landed on.
        private sealed class FakeCore : ICore
        {
            public int Counter;
            public byte[] Ram = new byte[8192];

            public string CoreName => "Fake";
            public int ScreenWidth => 1;
            public int ScreenHeight => 1;
            public double FrameRateHz => 60.0;
            public bool IsRomLoaded => true;
            public long TotalFrames { get; private set; }
            public int AudioSampleRate => 32000;
            public bool SkipRendering { get; set; }

            public void LoadRom(string path) { }

            public void RunFrame()
            {
                Counter++;
                TotalFrames++;
                Ram[Counter % Ram.Length] = (byte)Counter;
            }

            public byte[] GetFrameBufferRgba() => new byte[4];
            public short[] DequeueAudioSamples(int maxFrames) => Array.Empty<short>();
            public void SaveSram() { }

            public void SaveState(Stream stream)
            {
                var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true);
                w.Write(Counter);
                w.Write(TotalFrames);
                w.Write(Ram);
                w.Flush();
            }

            public void LoadState(Stream stream)
            {
                var r = new BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true);
                Counter = r.ReadInt32();
                TotalFrames = r.ReadInt64();
                Ram = r.ReadBytes(Ram.Length);
            }

            public void SaveState(string path)
            {
                using var fs = File.Create(path);
                SaveState(fs);
            }

            public void LoadState(string path)
            {
                using var fs = File.OpenRead(path);
                LoadState(fs);
            }
        }

        private static (FakeCore Core, RewindBuffer Buffer) Run(int frames, int interval = 1)
        {
            var core = new FakeCore();
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = interval };
            buffer.CaptureNow(core);
            for (int i = 0; i < frames; i++)
            {
                core.RunFrame();
                buffer.OnFrameCompleted(core);
            }
            return (core, buffer);
        }

        [Fact]
        public void A_disabled_buffer_captures_nothing()
        {
            var core = new FakeCore();
            var buffer = new RewindBuffer { Enabled = false };
            for (int i = 0; i < 50; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }
            Assert.Equal(0, buffer.Depth);
            Assert.False(buffer.Rewind(core));
        }

        [Fact]
        public void One_step_back_restores_the_previous_frame()
        {
            var (core, buffer) = Run(10);
            Assert.True(buffer.Rewind(core));
            Assert.Equal(9, core.Counter);
        }

        [Fact]
        public void Stepping_all_the_way_back_reaches_the_first_snapshot()
        {
            var (core, buffer) = Run(10);
            for (int i = 0; i < 10; i++) Assert.True(buffer.Rewind(core));
            Assert.Equal(0, core.Counter);
        }

        [Fact]
        public void Rewinding_past_the_start_reports_false_instead_of_throwing()
        {
            var (core, buffer) = Run(3);
            for (int i = 0; i < 3; i++) Assert.True(buffer.Rewind(core));
            Assert.False(buffer.Rewind(core));
            Assert.Equal(0, core.Counter);
        }

        // The whole point of the interval knob - §1.5.
        [Fact]
        public void The_interval_sets_how_many_frames_one_step_covers()
        {
            var (core, buffer) = Run(40, interval: 4);
            Assert.Equal(10, buffer.Depth);
            buffer.Rewind(core);
            Assert.Equal(36, core.Counter);
        }

        [Fact]
        public void Full_ram_contents_come_back_not_just_the_counter()
        {
            var core = new FakeCore();
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = 1 };
            buffer.CaptureNow(core);
            for (int i = 0; i < 20; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }

            byte[] expected = (byte[])core.Ram.Clone();
            for (int i = 0; i < 5; i++) core.RunFrame();
            for (int i = 0; i < 5; i++) buffer.CaptureNow(core);

            for (int i = 0; i < 5; i++) buffer.Rewind(core);
            Assert.Equal(expected, core.Ram);
        }

        // Rewind, play forward, rewind again must not splice two timelines - §1.4.
        [Fact]
        public void Resuming_forward_after_a_rewind_keeps_the_chain_coherent()
        {
            var (core, buffer) = Run(20);
            for (int i = 0; i < 5; i++) buffer.Rewind(core);
            Assert.Equal(15, core.Counter);

            for (int i = 0; i < 8; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }
            Assert.Equal(23, core.Counter);

            for (int i = 0; i < 8; i++) Assert.True(buffer.Rewind(core));
            Assert.Equal(15, core.Counter);
        }

        [Fact]
        public void Clear_drops_the_whole_chain()
        {
            var (core, buffer) = Run(10);
            buffer.Clear();
            Assert.Equal(0, buffer.Depth);
            Assert.False(buffer.Rewind(core));
        }

        // Evicting the oldest end must leave the newest end usable - §1.3.
        [Fact]
        public void Exceeding_the_budget_drops_the_oldest_history_not_the_newest()
        {
            var core = new FakeCore();
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = 1, BudgetBytes = 512 };
            buffer.CaptureNow(core);
            for (int i = 0; i < 200; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }

            Assert.True(buffer.Depth < 200);
            Assert.True(buffer.BufferedBytes <= 512 + buffer.SnapshotBytes);

            int before = core.Counter;
            Assert.True(buffer.Rewind(core));
            Assert.Equal(before - 1, core.Counter);
        }

        [Fact]
        public void Buffered_seconds_reports_wall_clock_history_not_snapshot_count()
        {
            var (_, buffer) = Run(120, interval: 4);
            Assert.Equal(2.0, buffer.BufferedSeconds(60.0), precision: 3);
        }
    }
}
