using System;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // The chain's float surfaces and feedback on a real GL device, where Skia's code is not the raster one the other tests draw with; gated on a device's name - see EmuSen_Serenity.md §3.9.
    public class ScreenFilterDeviceTests
    {
        // On the session's thread, since a control made on another would claim Avalonia's dispatcher for it.
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreenFilterDeviceTests).GetTypeInfo().Assembly);

        private static string? Device => Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable);

        private static (int R, int G, int B) At(byte[] picture, int width, int x, int y) => (picture[(y * width + x) * 4], picture[(y * width + x) * 4 + 1], picture[(y * width + x) * 4 + 2]);

        private static void Near((int R, int G, int B) expected, (int R, int G, int B) actual, int tolerance, string what) =>
            Assert.True(Math.Abs(expected.R - actual.R) <= tolerance && Math.Abs(expected.G - actual.G) <= tolerance && Math.Abs(expected.B - actual.B) <= tolerance,
                $"{what}: expected {expected} within {tolerance}, got {actual}");

        [Fact]
        public Task On_a_device_a_float_pass_keeps_a_negative_a_value_above_white_and_its_alpha() => Session.Dispatch(() =>
        {
            if (string.IsNullOrEmpty(Device)) return;
            const string wide = "half4 main(float2 coord) { return half4(-0.25, 2.0, 0.1, 0.25); }";
            const string narrow = "uniform shader source; half4 main(float2 coord) { half4 c = source.eval(coord); return half4(-2.0 * c.r, 0.25 * c.g, 2.0 * c.a, 1.0); }";
            var filter = new ScreenFilter("test", new[] { new FilterPass(wide, PassScale.Source) { Float = true }, new FilterPass(narrow, PassScale.Viewport) }, null, "");
            var control = new GameFrameControl { ActiveFilter = filter };
            byte[] picture = ShaderBench.Picture(control, new byte[]?[] { new byte[8 * 8 * 4] }, 8, 8, 1, 16, 16, Device);
            Near((128, 128, 128), At(picture, 16, 8, 8), 1, "through a float surface on the device");
        }, default);

        // A newly chosen filter takes over only after each of its passes has been drawn once at one pixel, a pass a draw, the one before drawn meanwhile - see EmuSen_Serenity.md §3.10.
        [Fact]
        public Task On_a_device_a_new_filter_is_warmed_a_pass_a_draw_before_it_takes_over() => Session.Dispatch(() =>
        {
            if (string.IsNullOrEmpty(Device)) return;
            static FilterPass Pass(string body, bool last) => new($"uniform shader source; half4 main(float2 coord) {{ {body} }}", last ? PassScale.Viewport : PassScale.Source) { Float = !last };
            var red = new ScreenFilter("red", new[] { Pass("return half4(0.8, 0.0, 0.0, 1.0);", true) }, null, "");
            var green = new ScreenFilter("green", new[] { Pass("return half4(0.0, 0.8, 0.0, 1.0);", false), Pass("return source.eval(coord);", false), Pass("return half4(source.eval(coord).rgb, 1.0);", true) }, null, "");

            using var gl = ShaderBench.GlContext.Create(Device);
            using SkiaSharp.GRGlInterface glInterface = SkiaSharp.GRGlInterface.CreateOpenGl(name => ShaderBench.GlContext.GetProc(name))!;
            using SkiaSharp.GRContext context = SkiaSharp.GRContext.CreateGl(glInterface)!;
            using SkiaSharp.SKSurface target = SkiaSharp.SKSurface.Create(context, true, new SkiaSharp.SKImageInfo(16, 16, SkiaSharp.SKColorType.Rgba8888, SkiaSharp.SKAlphaType.Premul))!;
            var control = new GameFrameControl { ActiveFilter = red };
            control.WaitForFilter();
            byte[] frame = new byte[8 * 8 * 4];
            (int R, int G, int B) Draw()
            {
                control.UpdateFrame(frame, 8, 8);
                using (var op = control.CaptureDrawOp(new Avalonia.Size(16, 16))!) op.RenderTo(target.Canvas, context);
                var pixel = new SkiaSharp.SKBitmap(16, 16);
                target.ReadPixels(pixel.Info, pixel.GetPixels(), pixel.RowBytes, 0, 0);
                SkiaSharp.SKColor c = pixel.GetPixel(8, 8);
                return (c.Red, c.Green, c.Blue);
            }
            Near((204, 0, 0), Draw(), 1, "the first filter");
            control.ActiveFilter = green;
            System.Threading.Thread.Sleep(500);
            Near((204, 0, 0), Draw(), 1, "the first filter while the second's first pass is warmed");
            Near((204, 0, 0), Draw(), 1, "and its second");
            Near((0, 204, 0), Draw(), 1, "the second filter, once its third pass is warmed");
            control.ActiveFilter = null;
        }, default);

        [Fact]
        public Task On_a_device_a_feedback_pass_counts_frames_and_not_redraws() => Session.Dispatch(() =>
        {
            if (string.IsNullOrEmpty(Device)) return;
            const string count = "uniform shader feedback; half4 main(float2 coord) { return half4(feedback.eval(coord).r + 10.0 / 255.0, 0.0, 0.0, 1.0); }";
            const string show = "uniform shader source; half4 main(float2 coord) { return half4(source.eval(coord).rgb, 1.0); }";
            var filter = new ScreenFilter("test", new[] { new FilterPass(count, PassScale.Source) { Feedback = true, Float = true }, new FilterPass(show, PassScale.Viewport) }, null, "");
            var control = new GameFrameControl { ActiveFilter = filter };
            byte[] frame = new byte[8 * 8 * 4];
            byte[] picture = ShaderBench.Picture(control, new byte[]?[] { frame, frame, null, frame, null, null }, 8, 8, 1, 16, 16, Device);
            Near((30, 0, 0), At(picture, 16, 8, 8), 1, "three frames and three redraws");
        }, default);
    }
}
