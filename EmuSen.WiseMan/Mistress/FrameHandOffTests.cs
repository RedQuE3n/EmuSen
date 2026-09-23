using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Mistress;
using EmuSen.Serenity;
using EmuSen.WiseMan.Fixtures;
using SkiaSharp;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress's path for a lent picture: the hand-off, the frame control and MarsRT's lending, wired as MainWindow wires them - see EmuSen_Serenity.md §2.8.
    [Collection("MarsStatics")]
    public class FrameHandOffTests
    {
        private readonly Xunit.Abstractions.ITestOutputHelper _output;

        public FrameHandOffTests(Xunit.Abstractions.ITestOutputHelper output) => _output = output;

        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FrameHandOffTests).GetTypeInfo().Assembly);

        [Fact]
        public async Task Frames_replaced_before_the_screen_took_them_go_back_once_and_a_presented_one_is_the_screen_s()
        {
            await Session.Dispatch(() =>
            {
                var presented = new List<byte[]>();
                var released = new List<byte[]>();
                var handOff = new FrameHandOff((pixels, _, _, _, _) => presented.Add(pixels));
                byte[] a = new byte[4], b = new byte[4], c = new byte[4], d = new byte[4];

                handOff.Offer(a, 1, 1, 1, released.Add);
                handOff.Offer(b, 1, 1, 1, released.Add);
                handOff.Offer(c, 1, 1, 1, released.Add);
                Assert.Equal(new[] { a, b }, released);
                Dispatcher.UIThread.RunJobs();
                Assert.Equal(new[] { c }, presented);

                // Presented, c is the control's to give back; the hand-off never does.
                handOff.Offer(d, 1, 1, 1, released.Add);
                Assert.Equal(new[] { a, b }, released);
                handOff.DropPending();
                handOff.DropPending();
                Assert.Equal(new[] { a, b, d }, released);
                Dispatcher.UIThread.RunJobs();
                Assert.Equal(new[] { c }, presented);
            }, default);
        }

        private sealed class CountingPool : EmuSen.Cores.IFrameBufferPool
        {
            public readonly List<byte[]> Returned = new();

            public void ReturnFrameBuffer(byte[] buffer) => Returned.Add(buffer);
        }

        // An ICore that lends, for the route's type test; nothing else of it is called.
        private sealed class PoolCore(EmuSen.Cores.IFrameBufferPool pool) : EmuSen.Cores.IFrameBufferPool, EmuSen.Cores.ICore
        {
            public void ReturnFrameBuffer(byte[] buffer) => pool.ReturnFrameBuffer(buffer);
            public string CoreName => "test";
            public int ScreenWidth => 2;
            public int ScreenHeight => 2;
            public double FrameRateHz => 60;
            public bool IsRomLoaded => true;
            public long TotalFrames => 0;
            public int AudioSampleRate => 32000;
            public bool SkipRendering { get; set; }
            public void LoadRom(string path) { }
            public void RunFrame() { }
            public byte[] GetFrameBufferRgba() => new byte[16];
            public short[] DequeueAudioSamples(int maxFrames) => Array.Empty<short>();
            public void SaveState(string path) { }
            public void LoadState(string path) { }
            public void SaveState(Stream stream) { }
            public void LoadState(Stream stream) { }
            public void SaveSram() { }
        }

        [Fact]
        public void A_core_that_does_not_lend_gets_no_route_and_one_that_does_keeps_its_route_until_the_session_ends()
        {
            var handOff = new FrameHandOff((_, _, _, _, _) => { });
            Assert.Null(handOff.ReleaseFor(null));
            var pool = new CountingPool();
            var core = new PoolCore(pool);
            Action<byte[]> first = handOff.ReleaseFor(core)!, second = handOff.ReleaseFor(core)!;
            Assert.Same(first.Target, second.Target);
            byte[] a = new byte[4], b = new byte[4];
            first(a);
            handOff.EndSession();
            first(b);
            Assert.Equal(new[] { a }, pool.Returned);
            Assert.NotSame(first.Target, handOff.ReleaseFor(core)!.Target);
        }

        // One lent picture shown through a route, then the session ended, leaving the control holding it; the core is reachable only through the route.
        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.NoInlining)]
        private static (WeakReference Core, CountingPool Pool) ShowOneAndEnd(FrameHandOff handOff, GameFrameControl control)
        {
            var pool = new CountingPool();
            var core = new PoolCore(pool);
            handOff.Offer(new byte[16], 2, 2, 1, handOff.ReleaseFor(core));
            Dispatcher.UIThread.RunJobs();
            GameFrameControl.DrawOp op = control.CaptureDrawOp(new Size(2, 2))!;
            Drawn(op, 2, 2);
            op.Dispose();
            handOff.EndSession();
            return (new WeakReference(core), pool);
        }

        [Fact]
        public async Task Once_a_session_ends_the_picture_left_on_screen_neither_keeps_its_core_nor_returns_to_it()
        {
            await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                var handOff = new FrameHandOff((pixels, width, height, rowRepeat, release) => control.UpdateFrame(pixels, width, height, rowRepeat, release));
                var (core, pool) = ShowOneAndEnd(handOff, control);
                GC.Collect();
                GC.WaitForPendingFinalizers();
                GC.Collect();
                Assert.False(core.IsAlive, "the control's last picture kept the ended session's core alive");

                // Superseded and drawn over by the next session's picture, the old array goes back by a cut route: nowhere.
                control.UpdateFrame(new byte[16], 2, 2);
                GameFrameControl.DrawOp next = control.CaptureDrawOp(new Size(2, 2))!;
                Drawn(next, 2, 2);
                next.Dispose();
                Assert.Empty(pool.Returned);
            }, default);
        }

        private static string Rom()
        {
            return SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: false), ".z64");
        }

        // What an operation draws at the frame's own size, where either filter gives the same pixels.
        private static byte[] Drawn(GameFrameControl.DrawOp op, int width, int height)
        {
            using var surface = SKSurface.Create(new SKImageInfo(width, height, SKColorType.Rgba8888, SKAlphaType.Premul));
            surface.Canvas.Clear(SKColors.Black);
            op.RenderTo(surface.Canvas, null);
            using SKPixmap pixmap = surface.PeekPixels();
            return pixmap.GetPixelSpan().ToArray();
        }

        private static byte[] Expected(byte[] picture, int width, int height)
        {
            var reference = new GameFrameControl();
            reference.UpdateFrame(picture, width, height);
            GameFrameControl.DrawOp op = reference.CaptureDrawOp(new Size(width, height))!;
            byte[] drawn = Drawn(op, width, height);
            op.Dispose();
            return drawn;
        }

        // Operations held, drawn late and out of order, and disposed at random while the core keeps lending: nothing held is written, and the picture only moves forward.
        [Fact]
        public async Task With_MarsRT_lending_no_array_the_screen_can_read_is_written_and_the_picture_never_goes_back()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Rom();
            try
            {
                await Session.Dispatch(() =>
                {
                    using var core = new MarsRtCore(batteryRamDisabled: true) { DeferredPresentation = false };
                    core.LoadRom(rom);
                    var control = new GameFrameControl();
                    var copies = new Dictionary<byte[], byte[]>(ReferenceEqualityComparer.Instance);
                    var releasedTwice = 0;
                    var pictures = new List<(byte[] Copy, int Width, int Height)> { default };
                    var handOff = new FrameHandOff((pixels, width, height, rowRepeat, release) =>
                    {
                        pictures.Add(((byte[])pixels.Clone(), width, height));
                        control.UpdateFrame(pixels, width, height, rowRepeat, release);
                    });
                    Action<byte[]> giveBack = buffer =>
                    {
                        if (!copies.Remove(buffer)) releasedTwice++;
                        core.ReturnFrameBuffer(buffer);
                    };

                    var random = new Random(0x5EED);
                    var held = new List<(GameFrameControl.DrawOp Op, int Version)>();
                    int shown = 0, lastDrawn = 0, draws = 0, lateDraws = 0, changed = 0;
                    long serial = -1;
                    for (int frame = 0; frame < 400; frame++)
                    {
                        core.RunFrame();
                        if (core.FrameSerial != serial)
                        {
                            serial = core.FrameSerial;
                            byte[] lent = core.GetFrameBufferRgba();
                            Assert.False(copies.ContainsKey(lent), $"frame {frame}: the core lent an array the screen can still read");
                            copies[lent] = (byte[])lent.Clone();
                            handOff.Offer(lent, core.ScreenWidth, core.ScreenHeight, core.RowRepeat, giveBack);
                        }

                        foreach (var (array, copy) in copies) Assert.True(array.AsSpan().SequenceEqual(copy), $"frame {frame}: an array still held was written");

                        if (random.Next(2) == 0) Dispatcher.UIThread.RunJobs();
                        if (random.Next(2) == 0 && held.Count < 4 && pictures.Count > 1)
                        {
                            held.Add((control.CaptureDrawOp(new Size(pictures[^1].Width, pictures[^1].Height))!, pictures.Count - 1));
                        }
                        if (held.Count > 0 && random.Next(5) < 2)
                        {
                            var (op, version) = held[random.Next(held.Count)];
                            if (version < shown) lateDraws++;
                            shown = Math.Max(shown, version);
                            var (picture, width, height) = pictures[shown];
                            byte[] drawn = Drawn(op, width, height);
                            Assert.True(drawn.AsSpan().SequenceEqual(Expected(picture, width, height)), $"frame {frame}: the draw of version {version} did not show version {shown}");
                            if (lastDrawn > 0 && !picture.AsSpan().SequenceEqual(pictures[lastDrawn].Copy)) changed++;
                            lastDrawn = shown;
                            draws++;
                        }
                        if (held.Count > 0 && random.Next(10) < 3)
                        {
                            int which = random.Next(held.Count);
                            held[which].Op.Dispose();
                            held.RemoveAt(which);
                        }
                    }
                    foreach (var (op, _) in held) op.Dispose();

                    _output.WriteLine($"{pictures.Count - 1} presented, {draws} draws ({lateDraws} late), {changed} of them a changed picture; arrays made {core.FrameBuffers.Made}, reused {core.FrameBuffers.Reused}, dropped {core.FrameBuffers.Dropped}; {copies.Count} still held");
                    Assert.Equal(0, releasedTwice);
                    Assert.True(lateDraws > 10 && changed > 10, "the schedule drew too few late or changed pictures to test anything");
                    Assert.True(core.FrameBuffers.Reused > 100, $"only {core.FrameBuffers.Reused} arrays were reused, so a premature return had little to overwrite");
                    Assert.True(copies.Count <= 8, $"{copies.Count} arrays still held after the run");
                }, default);
            }
            finally
            {
                try { File.Delete(rom); } catch (IOException) { }
            }
        }

        // Mistress's rewind path offers a lent picture after every step back as its frame path does after a frame: nothing the screen holds is written, lent twice or given back twice - see Mars_Native.md §6.6.3.
        [Fact]
        public async Task With_MarsRT_rewinding_between_frames_no_array_the_screen_can_read_is_written()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64System.Build(rsp: true), ".z64");
            try
            {
                await Session.Dispatch(() =>
                {
                    using var core = new MarsRtCore(batteryRamDisabled: true) { ThreadedRdp = true, RdpWorkers = 4, DeferredPresentation = true };
                    core.LoadRom(rom);
                    var rewind = new EmuSen.Common.RewindBuffer { Enabled = true, IntervalFrames = 1 };
                    var control = new GameFrameControl();
                    var copies = new Dictionary<byte[], byte[]>(ReferenceEqualityComparer.Instance);
                    int releasedTwice = 0, offered = 0, steps = 0, frames = 0, draws = 0;
                    var handOff = new FrameHandOff((pixels, width, height, rowRepeat, release) => control.UpdateFrame(pixels, width, height, rowRepeat, release));
                    Action<byte[]> giveBack = buffer =>
                    {
                        if (!copies.Remove(buffer)) releasedTwice++;
                        core.ReturnFrameBuffer(buffer);
                    };
                    void Offer(string when)
                    {
                        byte[] lent = core.GetFrameBufferRgba();
                        Assert.False(copies.ContainsKey(lent), $"{when}: the core lent an array the screen can still read");
                        copies[lent] = (byte[])lent.Clone();
                        handOff.Offer(lent, core.ScreenWidth, core.ScreenHeight, core.RowRepeat, giveBack);
                        offered++;
                    }

                    var random = new Random(0x2E1D);
                    var held = new List<GameFrameControl.DrawOp>();
                    long serial = -1;
                    for (int turn = 0; turn < 500; turn++)
                    {
                        if (random.Next(3) == 0)
                        {
                            for (int i = random.Next(1, 4); i > 0 && rewind.Rewind(core); i--)
                            {
                                steps++;
                                Offer($"turn {turn}, a step back to frame {core.TotalFrames}");
                            }
                            serial = -1;
                        }
                        else
                        {
                            core.RunFrame();
                            rewind.OnFrameCompleted(core);
                            frames++;
                            if (core.FrameSerial != serial)
                            {
                                serial = core.FrameSerial;
                                Offer($"turn {turn}, frame {core.TotalFrames}");
                            }
                        }

                        foreach (var (array, copy) in copies) Assert.True(array.AsSpan().SequenceEqual(copy), $"turn {turn}: an array still held was written");
                        if (random.Next(2) == 0) Dispatcher.UIThread.RunJobs();
                        if (random.Next(2) == 0 && held.Count < 4 && control.CaptureDrawOp(new Size(64, 64)) is { } op) held.Add(op);
                        if (held.Count > 0 && random.Next(3) == 0)
                        {
                            int which = random.Next(held.Count);
                            using var surface = SKSurface.Create(new SKImageInfo(64, 64, SKColorType.Rgba8888, SKAlphaType.Premul));
                            held[which].RenderTo(surface.Canvas, null);
                            held[which].Dispose();
                            held.RemoveAt(which);
                            draws++;
                        }
                    }
                    foreach (var op in held) op.Dispose();

                    _output.WriteLine($"{frames} frames and {steps} steps back, {offered} pictures offered, {draws} draws; arrays made {core.FrameBuffers.Made}, reused {core.FrameBuffers.Reused}, dropped {core.FrameBuffers.Dropped}; {copies.Count} still held");
                    Assert.Equal(0, releasedTwice);
                    Assert.True(steps > 100 && core.FrameBuffers.Reused > 100, $"{steps} steps back and {core.FrameBuffers.Reused} arrays reused tested too little");
                    Assert.True(copies.Count <= 8, $"{copies.Count} arrays still held after the run");
                }, default);
            }
            finally
            {
                try { File.Delete(rom); } catch (IOException) { }
            }
        }

        // The steady state of MainWindow's loop for the picture: lent, handed off, presented, drawn, given back - and nothing of the frame's size allocated.
        [Fact]
        public async Task With_the_lending_wired_as_Mistress_wires_it_a_frame_allocates_next_to_nothing()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Rom();
            try
            {
                await Session.Dispatch(() =>
                {
                    using var core = new MarsRtCore(batteryRamDisabled: true) { DeferredPresentation = false };
                    core.LoadRom(rom);
                    var control = new GameFrameControl();
                    var handOff = new FrameHandOff((pixels, width, height, rowRepeat, release) => control.UpdateFrame(pixels, width, height, rowRepeat, release));
                    Action<byte[]> release = handOff.ReleaseFor(core)!;
                    using var surface = SKSurface.Create(new SKImageInfo(64, 64, SKColorType.Rgba8888, SKAlphaType.Premul));

                    long allocated = 0, lent = 0;
                    long made = 0;
                    for (int frame = 0; frame < 150; frame++)
                    {
                        core.RunFrame();
                        if (frame == 30) made = core.FrameBuffers.Made;
                        long before = GC.GetAllocatedBytesForCurrentThread();
                        handOff.Offer(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight, core.RowRepeat, release);
                        Dispatcher.UIThread.RunJobs();
                        GameFrameControl.DrawOp op = control.CaptureDrawOp(new Size(64, 64))!;
                        op.RenderTo(surface.Canvas, null);
                        op.Dispose();
                        if (frame >= 30) { allocated += GC.GetAllocatedBytesForCurrentThread() - before; lent++; }
                    }

                    int frameBytes = core.ScreenWidth * core.ScreenHeight * 4;
                    double perFrame = allocated / (double)lent;
                    _output.WriteLine($"{perFrame:F0} bytes a frame against a picture of {frameBytes}; arrays made after warming {core.FrameBuffers.Made - made}, in all {core.FrameBuffers.Made}");
                    Assert.True(perFrame < 4096, $"{perFrame:F0} bytes a frame");
                    Assert.Equal(made, core.FrameBuffers.Made);
                }, default);
            }
            finally
            {
                try { File.Delete(rom); } catch (IOException) { }
            }
        }
    }
}
