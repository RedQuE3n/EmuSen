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
    // GameFrameControl.cs (see EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Project_Overview_v2.md
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

        // Plain RGBA8888, the shape FrameHash/BmpFile already take for the core's own frame buffer - see EmuSen_LunaP.md §13.
        private static byte[] ToRgbaBytes(WriteableBitmap bitmap) => EmuSen.WiseMan.Fixtures.UiTest.Capture(bitmap).Rgba;

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

        // The shader-active path (F8 - Scanlines/Crt) is where the
        // SKRuntimeShaderBuilder crash was actually caught (see
        // EmuSen_Project_Overview_v2.md §2a). These don't reproduce a
        // live-GPU flicker itself (no live GrContext here), but they do
        // exercise the shader path at all, and are the direct regression
        // guard for that crash (and for a logic regression - wrong
        // pixels, non-deterministic output - in the shader math itself).
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

        // Verifies the local-matrix scale (GameFrameControl.cs, replacing
        // the old offscreen-upscale-surface pass - see this file's own
        // header comment) puts the shader's "coord" in the right place:
        // BuiltInShaders.ScanlinesSksl darkens odd output-resolution rows
        // by exactly 0.78, leaving even rows untouched. A 1:4 upscale (16
        // native -> 64 window) makes every native pixel exactly 4 output
        // rows tall, so output rows 0-3 must be full brightness and rows
        // 4-7 must be darkened - if the scale/alignment were off by even
        // one row, this would catch it, unlike Shader_effects_actually_
        // change_the_rendered_output above, which only proves SOMETHING
        // changed.
        [Fact]
        public async Task Scanlines_darkens_odd_output_rows_by_the_exact_documented_factor()
        {
            await Session.Dispatch(() =>
            {
                byte[] frame = SolidColorFrame(16, 16, 200, 100, 50);
                (Window window, GameFrameControl control) = NewWindow(64, 64);
                try
                {
                    control.ActiveEffect = ShaderEffect.Scanlines;
                    control.UpdateFrame(frame, 16, 16);
                    using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                    byte[] pixels = ToRgbaBytes(captured);
                    int width = captured.PixelSize.Width;

                    int PixelIndex(int px, int py) => (py * width + px) * 4;

                    // Even output row (0) - full brightness, unshaded.
                    int evenRow = PixelIndex(32, 0);
                    Assert.Equal(200, pixels[evenRow]);
                    Assert.Equal(100, pixels[evenRow + 1]);
                    Assert.Equal(50, pixels[evenRow + 2]);

                    // Odd output row (5, safely inside the second native
                    // pixel's 4-row band) - darkened by exactly 0.78.
                    int oddRow = PixelIndex(32, 5);
                    Assert.Equal((byte)(200 * 0.78), pixels[oddRow]);
                    Assert.Equal((byte)(100 * 0.78), pixels[oddRow + 1]);
                    Assert.Equal((byte)(50 * 0.78), pixels[oddRow + 2]);
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
