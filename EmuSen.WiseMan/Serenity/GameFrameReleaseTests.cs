using System.Collections.Generic;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using EmuSen.Serenity;
using SkiaSharp;

namespace EmuSen.WiseMan.Serenity
{
    // Draw operations held and drawn out of order, and the arrays the control gives back - see EmuSen_Serenity.md §2.8.
    public class GameFrameReleaseTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameFrameReleaseTests).GetTypeInfo().Assembly);

        private static readonly Size Area = new(8, 8);
        private static readonly (byte, byte, byte) Red = (200, 0, 0), Green = (0, 200, 0), Blue = (0, 0, 200);

        private static byte[] Solid(byte r, byte g, byte b, int width = 4, int height = 4)
        {
            var frame = new byte[width * height * 4];
            for (int i = 0; i < frame.Length; i += 4) { frame[i] = r; frame[i + 1] = g; frame[i + 2] = b; frame[i + 3] = 255; }
            return frame;
        }

        // The centre pixel of what the operation draws on a canvas of its own.
        private static (byte R, byte G, byte B) Drawn(GameFrameControl.DrawOp op)
        {
            using var surface = SKSurface.Create(new SKImageInfo((int)Area.Width, (int)Area.Height, SKColorType.Rgba8888, SKAlphaType.Premul));
            surface.Canvas.Clear(SKColors.Black);
            op.RenderTo(surface.Canvas, null);
            using SKPixmap pixmap = surface.PeekPixels();
            SKColor centre = pixmap.GetPixelColor(4, 4);
            return (centre.Red, centre.Green, centre.Blue);
        }

        [Fact]
        public async Task A_draw_operation_rendered_late_never_takes_the_picture_backwards()
        {
            await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                control.UpdateFrame(Solid(200, 0, 0), 4, 4);
                GameFrameControl.DrawOp older = control.CaptureDrawOp(Area)!;
                control.UpdateFrame(Solid(0, 200, 0), 4, 4);
                GameFrameControl.DrawOp newer = control.CaptureDrawOp(Area)!;

                Assert.Equal(Green, Drawn(newer));
                Assert.Equal(Green, Drawn(older));
                Assert.Equal(Green, Drawn(newer));
            }, default);
        }

        // Each array goes back once: unread ones when superseded, a copied one when a newer is copied, never the current or the cached.
        [Fact]
        public async Task An_array_is_given_back_once_and_only_when_nothing_here_can_read_it()
        {
            await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                var released = new List<byte[]>();
                byte[] a = Solid(200, 0, 0), b = Solid(0, 200, 0), c = Solid(0, 0, 200), d = Solid(90, 90, 90);

                control.UpdateFrame(a, 4, 4, release: released.Add);
                GameFrameControl.DrawOp holdsA = control.CaptureDrawOp(Area)!;
                control.UpdateFrame(b, 4, 4, release: released.Add);
                control.UpdateFrame(c, 4, 4, release: released.Add);
                Assert.Equal(new[] { b }, released);

                Assert.Equal(Red, Drawn(holdsA));
                Assert.Equal(new[] { b }, released);

                GameFrameControl.DrawOp holdsC = control.CaptureDrawOp(Area)!;
                Assert.Equal(Blue, Drawn(holdsC));
                Assert.Equal(new[] { b, a }, released);

                // An operation disposed twice, or drawn again, gives back nothing twice.
                holdsA.Dispose();
                holdsA.Dispose();
                Assert.Equal(Blue, Drawn(holdsC));
                holdsC.Dispose();
                Assert.Equal(new[] { b, a }, released);

                // The cached array outlives its supersession until a newer one is copied.
                control.UpdateFrame(d, 4, 4, release: released.Add);
                Assert.Equal(new[] { b, a }, released);
                GameFrameControl.DrawOp holdsD = control.CaptureDrawOp(Area)!;
                Assert.Equal((90, 90, 90), ((int, int, int))Drawn(holdsD));
                Assert.Equal(new[] { b, a, c }, released);
                holdsD.Dispose();
                Assert.Equal(3, released.Count);
            }, default);
        }

        // An operation held past its frame's supersession, never drawn, lets the array go when it is disposed.
        [Fact]
        public async Task A_held_operation_disposed_undrawn_lets_its_array_go()
        {
            await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                var released = new List<byte[]>();
                byte[] a = Solid(200, 0, 0), b = Solid(0, 200, 0);
                control.UpdateFrame(a, 4, 4, release: released.Add);
                GameFrameControl.DrawOp holdsA = control.CaptureDrawOp(Area)!;
                control.UpdateFrame(b, 4, 4, release: released.Add);
                Assert.Empty(released);
                holdsA.Dispose();
                Assert.Equal(new[] { a }, released);
                Assert.Equal(Green, Drawn(control.CaptureDrawOp(Area)!));
                Assert.Equal(new[] { a }, released);
            }, default);
        }

        // Moon and Mercury offer one array rewritten in place: it is never given back while a later offer holds it - see EmuSen_Serenity.md §2.6.
        [Fact]
        public async Task The_same_array_offered_again_is_not_given_back_while_it_is_current()
        {
            await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                var released = new List<byte[]>();
                byte[] frame = Solid(200, 0, 0);
                for (int i = 0; i < 4; i++)
                {
                    control.UpdateFrame(frame, 4, 4, release: released.Add);
                    GameFrameControl.DrawOp op = control.CaptureDrawOp(Area)!;
                    Assert.Equal(Red, Drawn(op));
                    op.Dispose();
                }
                Assert.Empty(released);
            }, default);
        }
    }
}
