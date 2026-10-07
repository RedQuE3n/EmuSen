using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.WiseMan.Serenity
{
    // What a filter's passes may ask of the chain beyond the frame and the pass before: float surfaces, sizes, feedback, names, variants, a shape - see EmuSen_Serenity.md §3.9.
    public class ScreenFilterChainTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreenFilterChainTests).GetTypeInfo().Assembly);

        private const string Head = "uniform shader source;\n";

        // Writes values no 8-bit surface holds: a negative, one above white, and an alpha that is not coverage.
        private const string Wide = "half4 main(float2 coord) { return half4(-0.25, 2.0, 0.1, 0.25); }";

        // Brings them back into range, so a picture of mid grey says all three arrived.
        private const string Narrow = Head + "half4 main(float2 coord) { half4 c = source.eval(coord); return half4(-2.0 * c.r, 0.25 * c.g, 2.0 * c.a, 1.0); }";

        private static byte[] Solid(int width, int height, byte r, byte g, byte b)
        {
            var frame = new byte[width * height * 4];
            for (int i = 0; i < frame.Length; i += 4) (frame[i], frame[i + 1], frame[i + 2], frame[i + 3]) = (r, g, b, 255);
            return frame;
        }

        private sealed class Shown(GameFrameControl control, Window window) : IDisposable
        {
            public GameFrameControl Control => control;

            public (int R, int G, int B) Frame(byte[]? frame, int x, int y, int width = 8, int height = 8)
            {
                control.WaitForFilter();
                if (frame is not null) control.UpdateFrame(frame, width, height);
                else control.InvalidateVisual();
                using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
                int i = (y * capture.Width + x) * 4;
                return (capture.Rgba[i], capture.Rgba[i + 1], capture.Rgba[i + 2]);
            }

            public Window Window => window;

            public void Dispose() => window.Close();
        }

        private static Shown Show(ScreenFilter filter, int windowWidth = 8, int windowHeight = 8, string? console = null)
        {
            var control = new GameFrameControl { FilterConsole = console, ActiveFilter = filter };
            var window = new Window { Width = windowWidth, Height = windowHeight, Content = control };
            window.Show();
            return new Shown(control, window);
        }

        private static Window WindowOf(Shown shown) => shown.Window;

        // A capture without waiting for a build, so what is drawn while one runs can be read.
        private static (int R, int G, int B) Draw(Shown shown)
        {
            shown.Control.InvalidateVisual();
            using WriteableBitmap captured = shown.Window.CaptureRenderedFrame()!;
            var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
            int i = (4 * capture.Width + 4) * 4;
            return (capture.Rgba[i], capture.Rgba[i + 1], capture.Rgba[i + 2]);
        }

        private static ScreenFilter Of(params FilterPass[] passes) => new("test", passes, null, "");

        private static void Near((int R, int G, int B) expected, (int R, int G, int B) actual, int tolerance, string what) =>
            Assert.True(Math.Abs(expected.R - actual.R) <= tolerance && Math.Abs(expected.G - actual.G) <= tolerance && Math.Abs(expected.B - actual.B) <= tolerance,
                $"{what}: expected {expected} within {tolerance}, got {actual}");

        [Fact]
        public Task A_float_pass_keeps_a_negative_a_value_above_white_and_its_alpha_and_an_8_bit_pass_keeps_none() => Session.Dispatch(() =>
        {
            byte[] frame = Solid(8, 8, 0, 0, 0);
            using (Shown kept = Show(Of(new FilterPass(Wide, PassScale.Source) { Float = true }, new FilterPass(Narrow, PassScale.Viewport))))
                Near((128, 128, 128), kept.Frame(frame, 4, 4), 1, "through a float surface");
            using (Shown clamped = Show(Of(new FilterPass(Wide, PassScale.Source), new FilterPass(Narrow, PassScale.Viewport))))
                Assert.NotEqual((128, 128, 128), clamped.Frame(frame, 4, 4));
        }, default);

        [Fact]
        public Task A_pass_is_as_wide_as_it_says_and_as_tall_as_the_frame_times_its_factor() => Session.Dispatch(() =>
        {
            const string size = Head + "uniform float2 inputSize; uniform float2 viewportSize; half4 main(float2 coord) { return half4(inputSize.x / 255.0, inputSize.y / 255.0, viewportSize.x / 255.0, 1.0); }";
            var sized = new FilterPass("half4 main(float2 coord) { return half4(0.0, 0.0, 0.0, 1.0); }", PassScale.Source) { Width = PassSize.Fixed(40), Height = new PassSize(PassAxis.Source, 2.5f) };
            using Shown shown = Show(Of(sized, new FilterPass(size, PassScale.Viewport)), 16, 16);
            Near((40, 20, 16), shown.Frame(Solid(8, 8, 0, 0, 0), 4, 4), 0, "the sizes the second pass was told");
        }, default);

        [Fact]
        public Task A_feedback_pass_reads_its_last_frame_s_output_and_a_redraw_reads_the_same_one() => Session.Dispatch(() =>
        {
            const string count = "uniform shader feedback; half4 main(float2 coord) { return half4(feedback.eval(coord).r + 10.0 / 255.0, 0.0, 0.0, 1.0); }";
            byte[] frame = Solid(8, 8, 0, 0, 0);
            using Shown shown = Show(Of(new FilterPass(count, PassScale.Source) { Feedback = true, Float = true }, new FilterPass(Head + "half4 main(float2 coord) { return half4(source.eval(coord).rgb, 1.0); }", PassScale.Viewport)));
            Near((10, 0, 0), shown.Frame(frame, 4, 4), 1, "the first frame, over black");
            Near((20, 0, 0), shown.Frame(frame, 4, 4), 1, "the second");
            Near((30, 0, 0), shown.Frame(frame, 4, 4), 1, "the third");
            Near((30, 0, 0), shown.Frame(null, 4, 4), 1, "a redraw of the third");
            Near((30, 0, 0), shown.Frame(null, 4, 4), 1, "and another");
            Near((40, 0, 0), shown.Frame(frame, 4, 4), 1, "the fourth");
        }, default);

        [Fact]
        public Task A_later_pass_reads_an_earlier_one_by_name_with_its_size() => Session.Dispatch(() =>
        {
            const string red = "half4 main(float2 coord) { return half4(0.4, 0.0, 0.0, 1.0); }";
            const string green = Head + "half4 main(float2 coord) { return half4(0.0, 0.2 + source.eval(coord).r, 0.0, 1.0); }";
            const string both = Head + "uniform shader first; uniform float2 firstSize; half4 main(float2 coord) { return half4(first.eval(float2(0.5, 0.5)).r, source.eval(coord).g, firstSize.x / 255.0, 1.0); }";
            using Shown shown = Show(Of(new FilterPass(red, PassScale.Source) { Name = "first", Width = PassSize.Fixed(3) }, new FilterPass(green, PassScale.Source), new FilterPass(both, PassScale.Viewport)));
            Near((102, 153, 3), shown.Frame(Solid(8, 8, 0, 0, 0), 4, 4), 1, "the first pass's red, the second's green, the first's width");
        }, default);

        private static readonly SlangParameter Tint = new("tint", "Tint", 0.2f, 0f, 1f, 0.1f);
        private static readonly SlangParameter Stages = new("stages", "Stages", 1f, 1f, 2f, 1f) { Choices = new[] { "One", "Two" } };

        // One pass that draws the tint, or two when asked, the second moving it to green.
        private static ScreenFilter Variants(Action? built = null) => new("variants", Array.Empty<FilterPass>(), null, "", new[] { Tint, Stages })
        {
            Structural = new[] { "stages" },
            ConsoleDefaults = new Dictionary<string, IReadOnlyDictionary<string, float>> { ["SNES"] = new Dictionary<string, float> { ["tint"] = 0.6f } },
            Build = values =>
            {
                built?.Invoke();
                var first = new FilterPass("uniform float tint; half4 main(float2 coord) { return half4(tint, 0.0, 0.0, 1.0); }", values["stages"] > 1.5f ? PassScale.Source : PassScale.Viewport);
                return values["stages"] > 1.5f
                    ? new[] { first, new FilterPass(Head + "half4 main(float2 coord) { return half4(0.0, source.eval(coord).r, 0.0, 1.0); }", PassScale.Viewport) }
                    : new[] { first };
            },
        };

        [Fact]
        public Task A_built_filter_takes_a_console_s_defaults_and_is_built_again_only_when_a_structural_value_changes() => Session.Dispatch(() =>
        {
            int builds = 0;
            ScreenFilter filter = Variants(() => builds++);
            byte[] frame = Solid(8, 8, 0, 0, 0);

            using (Shown plain = Show(filter)) Near((51, 0, 0), plain.Frame(frame, 4, 4), 1, "the parameter's own default");

            builds = 0;
            using Shown shown = Show(filter, console: "SNES");
            Near((153, 0, 0), shown.Frame(frame, 4, 4), 1, "the console's default");
            shown.Control.ShaderParameters = new Dictionary<string, float> { ["tint"] = 0.8f };
            Near((204, 0, 0), shown.Frame(null, 4, 4), 1, "the player's value over the console's");
            Assert.Equal(1, builds);

            shown.Control.ShaderParameters = new Dictionary<string, float> { ["tint"] = 0.8f, ["stages"] = 2f };
            Near((0, 204, 0), shown.Frame(null, 4, 4), 1, "two stages");
            Assert.Equal(2, builds);
            shown.Control.ShaderParameters = new Dictionary<string, float> { ["stages"] = 2f };
            Near((0, 153, 0), shown.Frame(null, 4, 4), 1, "two stages at the console's tint");
            Assert.Equal(2, builds);
        }, default);

        [Fact]
        public Task A_filter_that_states_a_shape_is_drawn_in_that_shape_and_the_others_in_the_frame_s() => Session.Dispatch(() =>
        {
            var white = new FilterPass("half4 main(float2 coord) { return half4(1.0, 1.0, 1.0, 1.0); }", PassScale.Viewport);
            byte[] frame = Solid(8, 8, 0, 0, 0);
            using (Shown square = Show(Of(white), 400, 300))
            {
                Assert.Equal((0, 0, 0), square.Frame(frame, 20, 150));
                Assert.Equal((255, 255, 255), square.Frame(null, 200, 150));
            }
            using Shown tube = Show(Of(white) with { Aspect = 4.0 / 3.0 }, 400, 300);
            Assert.Equal((255, 255, 255), tube.Frame(frame, 20, 150));
            Assert.Equal((255, 255, 255), tube.Frame(null, 380, 290));
        }, default);

        [Fact]
        public Task A_console_s_values_that_are_not_parameters_reach_the_passes_and_any_console_takes_the_general_ones() => Session.Dispatch(() =>
        {
            var pass = new FilterPass("uniform float clock; uniform float tint; half4 main(float2 coord) { return half4(clock, tint, 0.0, 1.0); }", PassScale.Viewport);
            var filter = new ScreenFilter("constants", new[] { pass }, null, "", new[] { Tint })
            {
                ConsoleDefaults = new Dictionary<string, IReadOnlyDictionary<string, float>>
                {
                    ["SNES"] = new Dictionary<string, float> { ["clock"] = 0.4f },
                    [ScreenFilter.AnyConsole] = new Dictionary<string, float> { ["clock"] = 0.8f, ["tint"] = 0.6f },
                },
            };
            byte[] frame = Solid(8, 8, 0, 0, 0);
            using (Shown snes = Show(filter, console: "SNES")) Near((102, 51, 0), snes.Frame(frame, 4, 4), 1, "the console's own constant, and the parameter's own default beside it");
            using (Shown other = Show(filter, console: "N64")) Near((204, 153, 0), other.Frame(frame, 4, 4), 1, "a console without an entry takes the general one");
            using Shown none = Show(filter);
            Near((204, 153, 0), none.Frame(frame, 4, 4), 1, "and so does no console at all");
            none.Control.ShaderParameters = new Dictionary<string, float> { ["clock"] = 0.1f, ["tint"] = 0.2f };
            Near((204, 51, 0), none.Frame(null, 4, 4), 1, "a player's value reaches a parameter and never a constant");
        }, default);

        [Fact]
        public Task A_filter_that_counts_scanlines_is_given_the_frame_s_rows_once() => Session.Dispatch(() =>
        {
            var pass = new FilterPass("uniform float2 originalSize; half4 main(float2 coord) { return half4(originalSize.y / 255.0, 0.0, 0.0, 1.0); }", PassScale.Viewport);
            byte[] frame = Solid(8, 8, 0, 0, 0);
            foreach (var (filter, rows) in new[] { (Of(pass), 16), (Of(pass) with { RowsOnce = true }, 8) })
            {
                var control = new GameFrameControl { ActiveFilter = filter };
                control.WaitForFilter();
                var window = new Window { Width = 16, Height = 16, Content = control };
                window.Show();
                control.UpdateFrame(frame, 8, 8, rowRepeat: 2);
                using WriteableBitmap captured = window.CaptureRenderedFrame()!;
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
                window.Close();
                Assert.Equal(rows, capture.Rgba[(8 * capture.Width + 8) * 4]);
            }
        }, default);

        private static FilterPass Colour(string rgb) => new($"half4 main(float2 coord) {{ return half4({rgb}, 1.0); }}", PassScale.Viewport);

        // A filter whose build waits for the test, so the render thread's behaviour while it builds can be seen.
        private static ScreenFilter Held(System.Threading.ManualResetEventSlim gate, string rgb) => new("held", Array.Empty<FilterPass>(), null, "", new[] { Stages })
        {
            Structural = new[] { "stages" },
            Build = values => { gate.Wait(TimeSpan.FromSeconds(10)); return new[] { Colour(values["stages"] > 1.5f ? "0.0, 0.0, 0.8" : rgb) }; },
        };

        [Fact]
        public Task While_a_new_filter_builds_the_one_before_it_is_drawn_and_the_render_thread_never_waits() => Session.Dispatch(() =>
        {
            using var gate = new System.Threading.ManualResetEventSlim(false);
            byte[] frame = Solid(8, 8, 0, 51, 0);
            using Shown shown = Show(Of(Colour("0.8, 0.0, 0.0")));
            Near((204, 0, 0), shown.Frame(frame, 4, 4), 1, "the first filter");

            shown.Control.ActiveFilter = Held(gate, "0.0, 0.8, 0.0");
            var clock = System.Diagnostics.Stopwatch.StartNew();
            using (WriteableBitmap _ = WindowOf(shown).CaptureRenderedFrame()!) { }
            Assert.True(clock.ElapsedMilliseconds < 2000, $"a draw while the next filter builds took {clock.ElapsedMilliseconds} ms");
            Near((204, 0, 0), Draw(shown), 1, "the first filter, while the second builds");

            gate.Set();
            Near((0, 204, 0), shown.Frame(null, 4, 4), 1, "the second, once built");

            // A structural change on the running filter: its old passes go on being drawn until the new ones are built.
            gate.Reset();
            shown.Control.ShaderParameters = new Dictionary<string, float> { ["stages"] = 2f };
            Near((0, 204, 0), Draw(shown), 1, "the old passes, while the new ones build");
            gate.Set();
            Near((0, 0, 204), shown.Frame(null, 4, 4), 1, "the new passes, once built");
        }, default);

        [Fact]
        public Task The_first_filter_is_preceded_by_the_plain_picture_not_by_nothing() => Session.Dispatch(() =>
        {
            using var gate = new System.Threading.ManualResetEventSlim(false);
            var control = new GameFrameControl { ActiveFilter = Held(gate, "0.0, 0.8, 0.0") };
            var window = new Window { Width = 8, Height = 8, Content = control };
            window.Show();
            control.UpdateFrame(Solid(8, 8, 0, 0, 153), 8, 8);
            using (WriteableBitmap captured = window.CaptureRenderedFrame()!)
            {
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
                int i = (4 * capture.Width + 4) * 4;
                Assert.Equal((0, 0, 153), (capture.Rgba[i], capture.Rgba[i + 1], capture.Rgba[i + 2]));
            }
            gate.Set();
            window.Close();
        }, default);

        [Fact]
        public Task A_filter_that_needs_a_device_is_drawn_plain_in_software_unless_asked_otherwise() => Session.Dispatch(() =>
        {
            byte[] frame = Solid(8, 8, 0, 0, 153);
            using (Shown plain = Show(Of(Colour("0.8, 0.0, 0.0")) with { RequiresDevice = true }))
                Near((0, 0, 153), plain.Frame(frame, 4, 4), 1, "drawn plain on the headless raster path");
            using Shown asked = Show(Of(Colour("0.8, 0.0, 0.0")) with { RequiresDevice = true });
            asked.Control.DrawDeviceFiltersInSoftware = true;
            Near((204, 0, 0), asked.Frame(frame, 4, 4), 1, "drawn when asked to");
        }, default);

        [Fact]
        public void A_child_or_a_uniform_no_pass_provides_is_refused_by_name()
        {
            var chain = (IDisposable)Activator.CreateInstance(typeof(GameFrameControl).Assembly.GetType("EmuSen.Serenity.Shaders.FilterChain")!, Of(new FilterPass("uniform shader feedback; half4 main(float2 coord) { return feedback.eval(coord); }", PassScale.Viewport)))!;
            using var surface = SkiaSharp.SKSurface.Create(new SkiaSharp.SKImageInfo(8, 8));
            using var image = SkiaSharp.SKImage.FromPixelCopy(new SkiaSharp.SKImageInfo(8, 8, SkiaSharp.SKColorType.Rgba8888, SkiaSharp.SKAlphaType.Unpremul), Solid(8, 8, 0, 0, 0));
            MethodInfo draw = chain.GetType().GetMethod("Draw")!;
            var thrown = Assert.Throws<TargetInvocationException>(() => draw.Invoke(chain, new object?[] { surface.Canvas, null, image, 1, new SkiaSharp.SKRect(0, 0, 8, 8), default(PictureCrop) }));
            Assert.Contains("'feedback'", thrown.InnerException!.Message);
            chain.Dispose();
        }
    }
}
