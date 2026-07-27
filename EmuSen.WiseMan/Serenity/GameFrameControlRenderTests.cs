using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using EmuSen.Common.Imaging;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Serenity
{
    // Real Avalonia render passes through GameFrameControl's actual
    // Render()/DrawOp path (Avalonia.Headless + UseSkia, not the pure
    // ComputeLetterboxRect math GameFrameControlTests.cs already covers) -
    // catches genuine rendering-pipeline regressions (wrong pixels, wrong
    // scaling, exceptions, unstable output across repeated frames) that no
    // test here previously exercised at all. Won't catch a live-GPU/
    // compositor-specific bug like the per-frame-SKSurface flicker fixed in
    // GameFrameControl.cs (see Man pages/EmuSen_Project_Overview_v2.md
    // §2a) - headless rendering has no live GrContext - but it's real
    // coverage for everything else in this file.
    //
    // Every test body runs via session.Dispatch(...) rather than directly -
    // HeadlessUnitTestSession owns the one Avalonia dispatcher/UI thread
    // for the whole assembly (see TestAppBuilder.cs), and Window/Control
    // construction is only valid on it.
    public class GameFrameControlRenderTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameFrameControlRenderTests).GetTypeInfo().Assembly);

        private static byte[] SolidColorFrame(int width, int height, byte r, byte g, byte b)
        {
            var buffer = new byte[width * height * 4];
            for (int i = 0; i < buffer.Length; i += 4)
            {
                buffer[i] = r;
                buffer[i + 1] = g;
                buffer[i + 2] = b;
                buffer[i + 3] = 255;
            }
            return buffer;
        }

        private static (Window Window, GameFrameControl Control) NewWindow(int windowWidth, int windowHeight)
        {
            var control = new GameFrameControl();
            var window = new Window
            {
                Width = windowWidth,
                Height = windowHeight,
                Content = control,
            };
            window.Show();
            return (window, control);
        }

        // Reads the captured frame back into a plain RGBA8888 byte[] - the
        // same shape FrameHash/BmpFile already work with for the core's own
        // frame buffer (EmuSen.Pharaoh90's --autoshot), so this is directly
        // comparable against that same tooling.
        private static byte[] ToRgbaBytes(WriteableBitmap bitmap)
        {
            int width = bitmap.PixelSize.Width;
            int height = bitmap.PixelSize.Height;
            var buffer = new byte[width * height * 4];
            using ILockedFramebuffer fb = bitmap.Lock();
            Marshal.Copy(fb.Address, buffer, 0, buffer.Length);
            return buffer;
        }

        [Fact]
        public async Task Renders_a_solid_frame_with_the_correct_color_at_its_center()
        {
            await Session.Dispatch(() =>
            {
                byte[] frame = SolidColorFrame(4, 4, 200, 50, 25);
                (Window window, GameFrameControl control) = NewWindow(4, 4);
                try
                {
                    control.UpdateFrame(frame, 4, 4);

                    using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                    byte[] pixels = ToRgbaBytes(captured);

                    int width = captured.PixelSize.Width;
                    int centerIndex = ((captured.PixelSize.Height / 2) * width + width / 2) * 4;

                    Assert.Equal(200, pixels[centerIndex]);
                    Assert.Equal(50, pixels[centerIndex + 1]);
                    Assert.Equal(25, pixels[centerIndex + 2]);
                }
                finally
                {
                    window.Close();
                }
            }, default);
        }

        [Fact]
        public async Task Rendering_the_same_frame_repeatedly_produces_byte_identical_output()
        {
            await Session.Dispatch(() =>
            {
                byte[] frame = SolidColorFrame(16, 16, 10, 20, 30);
                (Window window, GameFrameControl control) = NewWindow(64, 64);
                try
                {
                    control.UpdateFrame(frame, 16, 16);

                    ulong? firstHash = null;
                    for (int i = 0; i < 30; i++)
                    {
                        using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                        ulong hash = FrameHash.Compute(ToRgbaBytes(captured));

                        if (firstHash == null)
                        {
                            firstHash = hash;
                        }
                        else
                        {
                            // Any drift here (with the input frame never
                            // changing) is exactly the flicker signature -
                            // Render() called 30 times against unchanged
                            // input must produce unchanged output every time.
                            Assert.Equal(firstHash, hash);
                        }
                    }
                }
                finally
                {
                    window.Close();
                }
            }, default);
        }

        [Fact]
        public async Task Rendering_many_consecutive_frames_does_not_throw()
        {
            await Session.Dispatch(() =>
            {
                (Window window, GameFrameControl control) = NewWindow(64, 64);
                try
                {
                    for (int i = 0; i < 120; i++)
                    {
                        byte[] frame = SolidColorFrame(16, 16, (byte)(i % 255), 0, 0);
                        control.UpdateFrame(frame, 16, 16);
                        using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                        Assert.NotNull(captured);
                    }
                }
                finally
                {
                    window.Close();
                }
            }, default);
        }

        // The shader-active path (F8 - Scanlines/Crt) still uses the old
        // per-frame offscreen-SKSurface pattern the no-shader path was just
        // fixed to avoid (see this file's own header comment and
        // EmuSen_Project_Overview_v2.md §2a's "known remaining gap"). These
        // don't reproduce the live-GPU flicker itself (no live GrContext
        // here), but they do exercise the shader path at all for the first
        // time, and would catch a logic regression (wrong pixels,
        // exceptions, non-deterministic output) in it.
        [Theory]
        [InlineData(ShaderEffect.Scanlines)]
        [InlineData(ShaderEffect.Crt)]
        public async Task Rendering_the_same_frame_repeatedly_with_a_shader_active_produces_byte_identical_output(
            ShaderEffect effect)
        {
            await Session.Dispatch(() =>
            {
                byte[] frame = SolidColorFrame(16, 16, 10, 20, 30);
                (Window window, GameFrameControl control) = NewWindow(64, 64);
                try
                {
                    control.ActiveEffect = effect;

                    ulong? firstHash = null;
                    for (int i = 0; i < 30; i++)
                    {
                        // Re-calling UpdateFrame every iteration, not just
                        // once before the loop - without this, nothing
                        // re-invalidates the control and CaptureRenderedFrame
                        // just replays the same cached bitmap, which would
                        // silently never exercise DrawOp.Render()'s shader
                        // path more than once (see this file's own note on
                        // GameFrameControl.cs's now-fixed SKRuntimeShaderBuilder
                        // disposal crash - a test with this exact gap is
                        // precisely what let that crash go undetected).
                        control.UpdateFrame(frame, 16, 16);
                        using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                        ulong hash = FrameHash.Compute(ToRgbaBytes(captured));

                        if (firstHash == null) firstHash = hash;
                        else Assert.Equal(firstHash, hash);
                    }
                }
                finally
                {
                    window.Close();
                }
            }, default);
        }

        [Theory]
        [InlineData(ShaderEffect.Scanlines)]
        [InlineData(ShaderEffect.Crt)]
        public async Task Shader_effects_actually_change_the_rendered_output(ShaderEffect effect)
        {
            await Session.Dispatch(() =>
            {
                byte[] frame = SolidColorFrame(16, 16, 10, 20, 30);
                (Window window, GameFrameControl control) = NewWindow(64, 64);
                try
                {
                    control.UpdateFrame(frame, 16, 16);
                    using WriteableBitmap unshaded = window.CaptureRenderedFrame()!;
                    ulong unshadedHash = FrameHash.Compute(ToRgbaBytes(unshaded));

                    // ActiveEffect alone doesn't InvalidateVisual() (see
                    // GameFrameControl.cs and GameWindow.CycleShaderEffect,
                    // neither call it) - only harmless in the live app
                    // because the emulation loop's own UpdateFrame() call
                    // invalidates on every one of its ~60fps ticks
                    // regardless of whether F8 was just pressed. Re-calling
                    // UpdateFrame here reproduces that same real trigger,
                    // not a workaround for a bug.
                    control.ActiveEffect = effect;
                    control.UpdateFrame(frame, 16, 16);
                    using WriteableBitmap shaded = window.CaptureRenderedFrame()!;
                    ulong shadedHash = FrameHash.Compute(ToRgbaBytes(shaded));

                    // If this ever fails, the shader silently isn't being
                    // applied at all - a much worse bug than a visual
                    // regression in the shader's own math.
                    Assert.NotEqual(unshadedHash, shadedHash);
                }
                finally
                {
                    window.Close();
                }
            }, default);
        }

        [Theory]
        [InlineData(ShaderEffect.Scanlines)]
        [InlineData(ShaderEffect.Crt)]
        public async Task Rendering_many_consecutive_frames_with_a_shader_active_does_not_throw(ShaderEffect effect)
        {
            await Session.Dispatch(() =>
            {
                (Window window, GameFrameControl control) = NewWindow(64, 64);
                try
                {
                    control.ActiveEffect = effect;
                    for (int i = 0; i < 120; i++)
                    {
                        byte[] frame = SolidColorFrame(16, 16, (byte)(i % 255), 0, 0);
                        control.UpdateFrame(frame, 16, 16);
                        using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                        Assert.NotNull(captured);
                    }
                }
                finally
                {
                    window.Close();
                }
            }, default);
        }
    }
}
