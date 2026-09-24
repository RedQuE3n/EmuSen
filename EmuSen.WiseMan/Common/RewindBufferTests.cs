using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
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

            public int Loads;

            public void LoadState(Stream stream)
            {
                Loads++;
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

        // The same machine, offering the snapshot a core with other threads writes without joining them - see §1.8.
        private sealed class SnapshotCore : ICore, ISnapshotCore
        {
            public readonly FakeCore Inner = new();
            public int Snapshots;

            public string CoreName => Inner.CoreName;
            public int ScreenWidth => Inner.ScreenWidth;
            public int ScreenHeight => Inner.ScreenHeight;
            public double FrameRateHz => Inner.FrameRateHz;
            public bool IsRomLoaded => Inner.IsRomLoaded;
            public long TotalFrames => Inner.TotalFrames;
            public int AudioSampleRate => Inner.AudioSampleRate;
            public bool SkipRendering { get => Inner.SkipRendering; set => Inner.SkipRendering = value; }
            public void LoadRom(string path) { }
            public void RunFrame() => Inner.RunFrame();
            public byte[] GetFrameBufferRgba() => Inner.GetFrameBufferRgba();
            public short[] DequeueAudioSamples(int maxFrames) => Inner.DequeueAudioSamples(maxFrames);
            public void SaveSram() { }
            public void SaveState(Stream stream) => Inner.SaveState(stream);
            public void LoadState(Stream stream) => Inner.LoadState(stream);
            public void SaveState(string path) => Inner.SaveState(path);
            public void LoadState(string path) => Inner.LoadState(path);

            public void SaveSnapshot(Stream stream)
            {
                Snapshots++;
                Inner.SaveState(stream);
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

        // A core that can snapshot without joining its threads is asked to, and the chain rewinds through those snapshots - see §1.8.
        [Fact]
        public void A_core_with_snapshots_is_captured_through_them()
        {
            var core = new SnapshotCore();
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = 1 };
            buffer.CaptureNow(core);
            for (int i = 0; i < 10; i++) { core.RunFrame(); buffer.OnFrameCompleted(core); }

            Assert.Equal(11, core.Snapshots);
            Assert.Equal(10, buffer.Depth);

            for (int i = 0; i < 4; i++) Assert.True(buffer.Rewind(core));
            Assert.Equal(6, core.Inner.Counter);
        }

        // The delta is encoded off the caller's thread, and the chain is whole by the time it is read - see §1.8.
        [Fact]
        public void A_capture_still_encoding_counts_and_rewinds()
        {
            var (core, buffer) = Run(3);
            core.RunFrame();
            buffer.OnFrameCompleted(core);

            Assert.Equal(4, buffer.Depth);
            Assert.True(buffer.Rewind(core));
            Assert.Equal(3, core.Counter);
            Assert.Equal(3, buffer.Depth);
        }

        [Fact]
        public void Buffered_seconds_reports_wall_clock_history_not_snapshot_count()
        {
            var (_, buffer) = Run(120, interval: 4);
            Assert.Equal(2.0, buffer.BufferedSeconds(60.0), precision: 3);
        }

        private static byte[] Save(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // Frames run with a capture every <interval>th, and the core's own state saved at each capture, by the buffer's frame.
        private static (FakeCore Core, RewindBuffer Buffer, Dictionary<long, byte[]> Saved) Record(int frames, int interval, long budget = RewindBuffer.DefaultBudgetBytes)
        {
            var core = new FakeCore();
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = interval, BudgetBytes = budget };
            var saved = new Dictionary<long, byte[]>();
            buffer.CaptureNow(core);
            saved[buffer.Frame] = Save(core);
            for (int i = 0; i < frames; i++)
            {
                core.RunFrame();
                if (buffer.OnFrameCompleted(core)) saved[buffer.Frame] = Save(core);
            }
            return (core, buffer, saved);
        }

        // Moments are the snapshots, one each, stamped with the frame they were taken at - see §5.1.
        [Fact]
        public void There_is_one_moment_per_snapshot_stamped_with_its_frame()
        {
            var (core, buffer, saved) = Record(40, interval: 4);
            IReadOnlyList<RewindMoment> moments = buffer.Moments();
            Assert.Equal(buffer.Depth + 1, moments.Count);
            Assert.Equal(new long[] { 0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40 }, moments.Select(m => m.Frame));
            Assert.Equal(moments.Select(m => m.Frame), moments.Select(m => m.CoreFrames));
            Assert.Equal(saved.Keys.OrderBy(k => k), moments.Select(m => m.Frame));

            Assert.True(buffer.Rewind(core));
            Assert.Equal(36, buffer.Frame);
            Assert.Equal(36, buffer.Moments().Last().Frame);
            Assert.Equal(buffer.Depth + 1, buffer.Moments().Count);
        }

        // Straight to the k-th moment back in one load, the state saved there byte for byte, and the newer moments gone - see §5.2.
        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(4)]
        [InlineData(10)]
        public void Rewinding_to_a_moment_restores_the_state_saved_there_in_one_load(int back)
        {
            var (core, buffer, saved) = Record(40, interval: 4);
            IReadOnlyList<RewindMoment> moments = buffer.Moments();
            RewindMoment target = moments[moments.Count - 1 - back];
            int loads = core.Loads;

            Assert.True(buffer.RewindTo(core, target.Frame));

            Assert.Equal(1, core.Loads - loads);
            Assert.Equal(saved[target.Frame], Save(core));
            Assert.Equal(target.Frame, buffer.Frame);
            Assert.Equal(moments.Take(moments.Count - back).Select(m => m.Frame), buffer.Moments().Select(m => m.Frame));
            Assert.Equal(buffer.Depth + 1, buffer.Moments().Count);
        }

        // Stepping back k times and going to the k-th moment land on the same bytes, and play on from it the same - see §5.2.
        [Fact]
        public void Rewinding_to_a_moment_and_stepping_back_to_it_are_the_same_machine()
        {
            var (stepped, byStep, _) = Record(60, interval: 3);
            var (direct, byMoment, _) = Record(60, interval: 3);
            for (int i = 0; i < 7; i++) Assert.True(byStep.Rewind(stepped));
            Assert.True(byMoment.RewindTo(direct, byMoment.Moments()[^8].Frame));
            Assert.Equal(Save(stepped), Save(direct));

            for (int i = 0; i < 9; i++)
            {
                stepped.RunFrame(); byStep.OnFrameCompleted(stepped);
                direct.RunFrame(); byMoment.OnFrameCompleted(direct);
            }
            Assert.Equal(byStep.Moments().Select(m => m.Frame), byMoment.Moments().Select(m => m.Frame));
            Assert.True(byMoment.RewindTo(direct, byMoment.Moments()[^5].Frame));
            for (int i = 0; i < 4; i++) Assert.True(byStep.Rewind(stepped));
            Assert.Equal(Save(stepped), Save(direct));
        }

        // A reel's preview and a test's expectation: the bytes of any moment, and the chain left where it was - see §5.2.
        [Fact]
        public void The_state_at_a_moment_is_rebuilt_without_moving_the_chain()
        {
            var (core, buffer, saved) = Record(40, interval: 4);
            byte[] now = Save(core);
            int depth = buffer.Depth;
            foreach (RewindMoment moment in buffer.Moments()) Assert.Equal(saved[moment.Frame], buffer.StateAt(moment.Frame));

            Assert.Equal(depth, buffer.Depth);
            Assert.Equal(now, Save(core));
            Assert.Null(buffer.StateAt(41));
            Assert.False(buffer.RewindTo(core, 41));
            Assert.Equal(depth, buffer.Depth);
            Assert.True(buffer.Rewind(core));
            Assert.Equal(saved[36], Save(core));
        }

        // Trimming to the budget drops the oldest snapshot and its moment together - see §5.1.
        [Fact]
        public void The_budget_drops_a_moment_with_its_snapshot()
        {
            var (core, buffer, saved) = Record(300, interval: 1, budget: 1024);
            IReadOnlyList<RewindMoment> moments = buffer.Moments();
            Assert.True(moments.Count < 250, $"{moments.Count} moments held");
            Assert.Equal(buffer.Depth + 1, moments.Count);
            Assert.Equal(300, moments[^1].Frame);
            Assert.Equal(Enumerable.Range(301 - moments.Count, moments.Count).Select(f => (long)f), moments.Select(m => m.Frame));

            Assert.True(buffer.RewindTo(core, moments[0].Frame));
            Assert.Equal(saved[moments[0].Frame], Save(core));
        }

        // A picture of four quadrants, each its own colour.
        private static byte[] Quadrants(int width, int rows)
        {
            var rgba = new byte[width * rows * 4];
            for (int y = 0; y < rows; y++)
                for (int x = 0; x < width; x++)
                {
                    int o = (y * width + x) * 4;
                    bool right = x >= width / 2, bottom = y >= rows / 2;
                    rgba[o] = (byte)(right ? 255 : 0);
                    rgba[o + 1] = (byte)(bottom ? 255 : 0);
                    rgba[o + 2] = (byte)(right && bottom ? 0 : 128);
                    rgba[o + 3] = 255;
                }
            return rgba;
        }

        private static (int R, int G, int B) At(byte[] rgba, int width, int x, int y) => (rgba[(y * width + x) * 4], rgba[(y * width + x) * 4 + 1], rgba[(y * width + x) * 4 + 2]);

        // Downscaled to the width asked for, the shown height kept in proportion, and each quadrant its colour to RGB565's precision - see §5.3.
        [Fact]
        public void A_thumbnail_is_the_frame_downscaled_to_the_asked_width()
        {
            RewindThumbnail wide = RewindThumbnail.From(Quadrants(320, 240), 320, 240, 1, 160)!;
            Assert.Equal((160, 120), (wide.Width, wide.Height));
            Assert.Equal(160 * 120 * 2, wide.Bytes);
            byte[] rgba = wide.ToRgba();
            Assert.Equal((0, 0, 132), At(rgba, 160, 10, 10));
            Assert.Equal((255, 0, 132), At(rgba, 160, 150, 10));
            Assert.Equal((0, 255, 132), At(rgba, 160, 10, 110));
            Assert.Equal((255, 255, 0), At(rgba, 160, 150, 110));

            // Rows handed over once and shown twice are a frame twice as tall.
            RewindThumbnail repeated = RewindThumbnail.From(Quadrants(640, 240), 640, 240, 2, 160)!;
            Assert.Equal((160, 120), (repeated.Width, repeated.Height));
            Assert.Equal((255, 255, 0), At(repeated.ToRgba(), 160, 150, 110));

            // Never enlarged, and nothing from a frame too short for its size.
            Assert.Equal(100, RewindThumbnail.From(Quadrants(100, 90), 100, 90, 1, 160)!.Width);
            Assert.Null(RewindThumbnail.From(new byte[16], 320, 240, 1, 160));
        }

        // A picture joins the snapshot just taken, once, and goes when its snapshot goes - see §5.3.
        [Fact]
        public void A_picture_belongs_to_the_newest_moment_and_leaves_with_it()
        {
            var core = new FakeCore();
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = 2, ThumbnailWidth = 160 };
            byte[] frame = Quadrants(256, 224);
            Assert.Null(buffer.AttachThumbnail(frame, 256, 224, 1));

            for (int i = 0; i < 10; i++)
            {
                core.RunFrame();
                if (buffer.OnFrameCompleted(core)) Assert.NotNull(buffer.AttachThumbnail(frame, 256, 224, 1));
                else Assert.Null(buffer.AttachThumbnail(frame, 256, 224, 1));
            }

            IReadOnlyList<RewindMoment> moments = buffer.Moments();
            Assert.Equal(5, moments.Count);
            Assert.All(moments, m => Assert.Equal((160, 140), (m.Thumbnail!.Width, m.Thumbnail.Height)));
            Assert.Equal(5L * 160 * 140 * 2, buffer.ThumbnailBytes);

            Assert.True(buffer.Rewind(core));
            Assert.Equal(4L * 160 * 140 * 2, buffer.ThumbnailBytes);
            Assert.True(buffer.RewindTo(core, buffer.Moments()[1].Frame));
            Assert.Equal(2L * 160 * 140 * 2, buffer.ThumbnailBytes);
            buffer.Clear();
            Assert.Equal(0, buffer.ThumbnailBytes);

            var off = new RewindBuffer { Enabled = true, IntervalFrames = 1 };
            core.RunFrame();
            Assert.True(off.OnFrameCompleted(core));
            Assert.Null(off.AttachThumbnail(frame, 256, 224, 1));
        }

        // Over the picture budget the older half is thinned: the newest pictures stay dense, the oldest keeps its picture, the snapshots all stay - see §5.4.
        [Fact]
        public void Past_the_picture_budget_older_pictures_thin_out_and_the_snapshots_stay()
        {
            var core = new FakeCore();
            int each = 160 * 140 * 2;
            var buffer = new RewindBuffer { Enabled = true, IntervalFrames = 1, ThumbnailWidth = 160, ThumbnailBudgetBytes = 40L * each };
            byte[] frame = Quadrants(256, 224);
            buffer.CaptureNow(core);
            buffer.AttachThumbnail(frame, 256, 224, 1);
            for (int i = 0; i < 400; i++)
            {
                core.RunFrame();
                if (buffer.OnFrameCompleted(core)) buffer.AttachThumbnail(frame, 256, 224, 1);
                Assert.True(buffer.ThumbnailBytes <= buffer.ThumbnailBudgetBytes);
            }

            IReadOnlyList<RewindMoment> moments = buffer.Moments();
            Assert.Equal(401, moments.Count);
            long[] pictured = moments.Where(m => m.Thumbnail is not null).Select(m => m.Frame).ToArray();
            Assert.Equal(buffer.ThumbnailBytes, pictured.Length * (long)each);
            Assert.True(pictured.Length >= 20, $"{pictured.Length} pictures kept of 40 allowed");
            Assert.Equal(0, pictured[0]);
            Assert.Equal(400, pictured[^1]);
            Assert.Equal(Enumerable.Range(401 - 10, 10).Select(f => (long)f), pictured[^10..]);
            long[] gaps = pictured.Zip(pictured.Skip(1), (a, b) => b - a).ToArray();
            Assert.True(gaps[0] > gaps[^1], $"the oldest gap {gaps[0]} is not wider than the newest {gaps[^1]}");
        }
    }
}
