using System;
using System.Collections.Generic;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Mistress;
using EmuSen.Serenity;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using SkiaSharp;

namespace EmuSen.WiseMan.Mistress
{
    // Mistress's path for a picture on the frontend API: copied into a buffer the hand-off owns, handed to the frame control, given back when it lets go - see EmuSen_Serenity.md §2.8 and EmuSen_Backend.md §14.13.
    [Collection(TestCollections.ProcessGlobals)]
    public class FrameHandOffTests
    {
        private readonly Xunit.Abstractions.ITestOutputHelper _output;

        public FrameHandOffTests(Xunit.Abstractions.ITestOutputHelper output) => _output = output;

        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FrameHandOffTests).GetTypeInfo().Assembly);

        // The screen's shape and the console's lines a frame reports are presented with that frame - see EmuSen_Serenity.md §2.9 and EmuSen_CRT.md §16.
        [Fact]
        public Task A_frame_s_shape_and_its_console_s_lines_are_presented_with_it() => Session.Dispatch(() =>
        {
            var shapes = new List<(double, int)>();
            var handOff = new FrameHandOff((_, _, _, _, release, _, aspect, lines, _, _) => shapes.Add((aspect, lines)));
            handOff.Offer(handOff.Rent(4)!, 1, 1, 1, aspect: 4.0 / 3.0, lines: 288);
            Dispatcher.UIThread.RunJobs();
            handOff.Offer(handOff.Rent(4)!, 1, 1, 1);
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(new[] { (4.0 / 3.0, 288), (0.0, 0) }, shapes);
        }, default);

        // A frame replaced before the screen took it goes back to the hand-off's buffers; one presented is the screen's until it gives it back.
        [Fact]
        public Task Frames_replaced_before_the_screen_took_them_go_back_and_a_presented_one_is_the_screen_s() => Session.Dispatch(() =>
        {
            var presented = new List<(byte[] Pixels, Action<byte[]>? Release)>();
            var handOff = new FrameHandOff((pixels, _, _, _, release, _, _, _, _, _) => presented.Add((pixels, release)));
            byte[] a = handOff.Rent(4)!;
            handOff.Offer(a, 1, 1, 1);
            byte[] b = handOff.Rent(4)!;
            handOff.Offer(b, 1, 1, 1);
            Assert.Same(a, handOff.Rent(4));
            handOff.GiveBack(a);
            Dispatcher.UIThread.RunJobs();
            Assert.Single(presented);
            Assert.Same(b, presented[0].Pixels);

            // Presented, b is out until the control releases it: four buffers in all, so three more and then none.
            byte[]?[] rented = { handOff.Rent(4), handOff.Rent(4), handOff.Rent(4) };
            Assert.All(rented, r => Assert.NotNull(r));
            Assert.Null(handOff.Rent(4));
            presented[0].Release!(b);
            Assert.Same(b, handOff.Rent(4));
        }, default);

        // A picture taken on the device carries no pixels and no release; the lend alone reaches the control.
        [Fact]
        public Task A_picture_on_the_device_carries_no_pixels_and_no_release() => Session.Dispatch(() =>
        {
            Action<byte[]>? given = x => { };
            byte[]? pixels = null;
            var handOff = new FrameHandOff((p, _, _, _, release, _, _, _, _, _) => (pixels, given) = (p, release));
            handOff.OfferOnDevice(null!, 4, 4, 1, 0, 0);
            Dispatcher.UIThread.RunJobs();
            Assert.Empty(pixels!);
            Assert.Null(given);
        }, default);

        private static byte[] Drawn(GameFrameControl.DrawOp op, int width, int height)
        {
            using var surface = SKSurface.Create(new SKImageInfo(width, height, SKColorType.Rgba8888, SKAlphaType.Premul));
            surface.Canvas.Clear(SKColors.Black);
            op.RenderTo(surface.Canvas, null);
            using SKPixmap pixmap = surface.PeekPixels();
            return pixmap.GetPixelSpan().ToArray();
        }

        // The steady state of MainWindow's loop for the picture: copied from the API into a rented buffer, handed off, presented, drawn, given back - and nothing of the frame's size allocated (C4).
        [Fact]
        public async Task With_the_picture_copied_as_Mistress_copies_it_a_frame_allocates_next_to_nothing()
        {
            using var crystal = new CrystalFixture();
            using CrystalSession session = crystal.Open();
            await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                var handOff = new FrameHandOff((pixels, width, height, rowRepeat, release, _, _, _, _, _) => control.UpdateFrame(pixels, width, height, rowRepeat, release));
                var info = new CrystalFrameInfo();
                using var surface = SKSurface.Create(new SKImageInfo(64, 64, SKColorType.Rgba8888, SKAlphaType.Premul));
                long allocated = 0, frames = 0;
                int skipped = 0;
                for (int frame = 0; frame < 150; frame++)
                {
                    session.Frame(ref info);
                    long before = GC.GetAllocatedBytesForCurrentThread();
                    if (handOff.Rent(info.width * info.height * 4) is { } pixels)
                    {
                        session.CopyPicture(pixels);
                        handOff.Offer(pixels, info.width, info.height, Math.Max(1, info.row_repeat));
                    }
                    else skipped++;
                    Dispatcher.UIThread.RunJobs();
                    GameFrameControl.DrawOp op = control.CaptureDrawOp(new Size(64, 64))!;
                    op.RenderTo(surface.Canvas, null);
                    op.Dispose();
                    if (frame >= 30) { allocated += GC.GetAllocatedBytesForCurrentThread() - before; frames++; }
                }
                double perFrame = allocated / (double)frames;
                _output.WriteLine($"{perFrame:F0} bytes a frame against a picture of {info.width * info.height * 4}; {skipped} pictures skipped for want of a buffer");
                Assert.True(perFrame < 4096, $"{perFrame:F0} bytes a frame");
                Assert.True(skipped < 10, $"{skipped} pictures found no free buffer");
            }, default);
        }
    }
}
