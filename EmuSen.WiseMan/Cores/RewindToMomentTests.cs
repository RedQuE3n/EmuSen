using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // A reel's choice on real cores: the machine k moments back is the state saved at that moment, byte for byte - see EmuSen_Rewind_And_FastForward.md §5.2.
    public static class RewindToMoment
    {
        public static string Hash(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return Convert.ToHexString(SHA256.HashData(stream.ToArray()));
        }

        // Frames at Mistress's interval with pictures attached as the loop attaches them, and the core's own state hashed at every capture.
        public static (RewindBuffer Buffer, Dictionary<long, string> Saved) Play(ICore core, int frames)
        {
            var buffer = new RewindBuffer { Enabled = true, ThumbnailWidth = RewindBuffer.DefaultThumbnailWidth };
            var saved = new Dictionary<long, string>();
            buffer.CaptureNow(core);
            Captured(core, buffer, saved);
            for (int i = 0; i < frames; i++)
            {
                core.RunFrame();
                if (buffer.OnFrameCompleted(core)) Captured(core, buffer, saved);
            }
            return (buffer, saved);
        }

        private static void Captured(ICore core, RewindBuffer buffer, Dictionary<long, string> saved)
        {
            saved[buffer.Frame] = Hash(core);
            byte[] picture = core.GetFrameBufferRgba();
            buffer.AttachThumbnail(picture, core.ScreenWidth, picture.Length / 4 / core.ScreenWidth, (core as IRepeatedRows)?.RowRepeat ?? 1);
            (core as IFrameBufferPool)?.ReturnFrameBuffer(picture);
        }

        // Three choices in turn - one back, the middle, the oldest - each checked against the saved state, the play between them re-recorded.
        public static void Prove(Func<ICore> make, int frames, ITestOutputHelper output, string name)
        {
            ICore core = make();
            try
            {
                var (buffer, saved) = Play(core, frames);
                IReadOnlyList<RewindMoment> moments = buffer.Moments();
                output.WriteLine($"{name}: moments {moments.Count} from {moments[0].Frame} to {moments[^1].Frame}, saved {saved.Count} from {saved.Keys.Min()}, state {buffer.SnapshotBytes} bytes, deltas {buffer.BufferedBytes - buffer.SnapshotBytes} bytes");
                Assert.True(moments.Count > 20, $"{name}: only {moments.Count} moments");
                Assert.All(moments, m => Assert.NotNull(m.Thumbnail));
                // A machine's first snapshot may be a shape its later ones are not, which restarts the chain (§1.3), so the saved states are a superset.
                Assert.All(moments, m => Assert.True(saved.ContainsKey(m.Frame)));
                Assert.Equal(moments[^1].Frame, saved.Keys.Max());

                foreach (int back in new[] { 1, moments.Count / 2, int.MaxValue })
                {
                    moments = buffer.Moments();
                    RewindMoment target = moments[Math.Max(0, moments.Count - 1 - back)];
                    Assert.True(buffer.RewindTo(core, target.Frame));
                    Assert.Equal(saved[target.Frame], Hash(core));
                    Assert.Equal(target.CoreFrames, core.TotalFrames);
                    Assert.Equal(target.Frame, buffer.Moments()[^1].Frame);

                    for (int i = 0; i < 40; i++)
                    {
                        core.RunFrame();
                        if (buffer.OnFrameCompleted(core)) saved[buffer.Frame] = Hash(core);
                    }
                }
                output.WriteLine($"{name}: {moments.Count} moments, three choices each the saved state");
            }
            finally
            {
                (core as IDisposable)?.Dispose();
            }
        }

        // The oldest moment reached straight and by stepping, on two identical runs: both the saved state, and what each cost.
        public static void Time(Func<ICore> make, int frames, ITestOutputHelper output, string name)
        {
            ICore direct = make(), stepped = make();
            try
            {
                var (byMoment, saved) = Play(direct, frames);
                var (byStep, savedByStep) = Play(stepped, frames);
                RewindMoment oldest = byMoment.Moments()[0];
                int steps = byStep.Depth;

                var clock = Stopwatch.StartNew();
                Assert.True(byMoment.RewindTo(direct, oldest.Frame));
                double directMs = clock.Elapsed.TotalMilliseconds;

                clock.Restart();
                for (int i = 0; i < steps; i++) Assert.True(byStep.Rewind(stepped));
                double steppedMs = clock.Elapsed.TotalMilliseconds;

                output.WriteLine($"{name}: straight to {oldest.Frame}, stepped to {byStep.Frame}");
                Assert.Equal(saved[oldest.Frame], Hash(direct));
                Assert.Equal(oldest.Frame, byStep.Frame);
                Assert.Equal(savedByStep[oldest.Frame], Hash(stepped));
                output.WriteLine($"{name}: the oldest of {steps + 1} moments, a {byMoment.SnapshotBytes}-byte state: straight {directMs:F2} ms, {steps} steps {steppedMs:F2} ms");
            }
            finally
            {
                (direct as IDisposable)?.Dispose();
                (stepped as IDisposable)?.Dispose();
            }
        }
    }

    public class RewindToMomentTests
    {
        private readonly ITestOutputHelper _output;

        public RewindToMomentTests(ITestOutputHelper output) => _output = output;

        // A loop counting in direct page, after the boot stub.
        private static readonly byte[] SnesRom = SyntheticRom.Build((5, new byte[] { 0xE6, 0x10, 0x80, 0xFC }));

        // A loop counting through a page of work RAM.
        private static readonly byte[] GbRom = SyntheticGbRom.Build(patches: (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x2C, 0x18, 0xFC }));

        private static ICore Snes() => SyntheticRom.LoadCore(SnesRom);

        private static ICore GameBoy()
        {
            var core = new MercuryCore();
            core.LoadRom(SyntheticGbRom.WriteTemp(GbRom));
            return core;
        }

        [Fact]
        public void On_the_SNES_a_chosen_moment_is_the_state_saved_there() => RewindToMoment.Prove(Snes, 600, _output, "SNES");

        [Fact]
        public void On_the_Game_Boy_a_chosen_moment_is_the_state_saved_there() => RewindToMoment.Prove(GameBoy, 600, _output, "Game Boy");

        [Fact]
        public void On_the_SNES_the_oldest_moment_straight_and_by_steps_is_the_same_state() => RewindToMoment.Time(Snes, 3000, _output, "SNES");
    }

    // MarsRT as Mistress runs it: four workers, the picture deferred - see Mars_Native.md §6.6.3.
    [Collection("MarsStatics")]
    public class MarsRtRewindToMomentTests
    {
        private readonly ITestOutputHelper _output;

        public MarsRtRewindToMomentTests(ITestOutputHelper output) => _output = output;

        private static ICore Production()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            var core = new MarsRtCore(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = true, RdpWorkers = 4, DeferredPresentation = true, SkipRendering = false };
            core.LoadRom(SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: true)));
            // Any load marks the controller pak dirty, so the proof starts from a loaded machine - see EmuSen_Rewind_And_FastForward.md §5.2.
            core.LoadState(core.Save(false));
            return core;
        }

        // The one byte a fresh machine's first load changes: the pak's dirty flag, which only tells the frontend to write the pak file.
        [Fact]
        public void On_a_fresh_MarsRT_the_first_load_changes_the_pak_s_dirty_flag_alone()
        {
            using var core = new MarsRtCore(expansionPak: true, batteryRamDisabled: true) { ThreadedRdp = false, DeferredPresentation = false, SkipRendering = false };
            core.LoadRom(SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: true)));
            for (int f = 0; f < 4; f++) core.RunFrame();
            byte[] before = core.Save(false);
            core.LoadState(before);
            byte[] after = core.Save(false);
            core.LoadState(after);

            using var machine = new MarsMachine(before.Length > 8 << 20 ? 8 << 20 : 4 << 20);
            machine.Load(before);
            string[] layout = machine.Layout(false).Split('\n', StringSplitOptions.RemoveEmptyEntries);
            var changed = new List<string>();
            for (int i = 0; i < before.Length; i++)
            {
                if (before[i] == after[i]) continue;
                changed.Add(layout.First(line => { string[] f = line.Split(' '); long at = long.Parse(f[0]); return at <= i && i < at + long.Parse(f[1]); }));
            }
            Assert.Equal(new[] { "Bus.Si.Controllers[0].Pak.Dirty" }, changed.Select(line => line.Split(' ')[^1]));
            Assert.Equal(after, core.Save(false));
        }

        [Fact]
        public void On_MarsRT_a_chosen_moment_is_the_state_saved_there() => RewindToMoment.Prove(Production, 300, _output, "MarsRT");

        [Fact]
        public void On_MarsRT_the_oldest_moment_straight_and_by_steps_is_the_same_state() => RewindToMoment.Time(Production, 300, _output, "MarsRT");
    }
}
