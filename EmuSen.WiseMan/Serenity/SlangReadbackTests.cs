using System;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Serenity;
using EmuSen.Serenity.Slang;
using SkiaSharp;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The chain's picture lent to Skia without a copy, the rows repeated on the device, and no source image under a preset - see EmuSen_Serenity.md §9.
    public class SlangReadbackTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SlangReadbackTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangReadback", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;
        private readonly SlangVulkan? _gpu;

        public SlangReadbackTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            _gpu = SlangVulkan.TryCreate(out string report, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800");
            _output.WriteLine($"device: {report}");
        }

        public void Dispose()
        {
            _gpu?.Dispose();
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private const string Vertex = """
            #version 450
            layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;
            #pragma stage vertex
            layout(location = 0) in vec4 Position;
            layout(location = 1) in vec2 TexCoord;
            layout(location = 0) out vec2 vTexCoord;
            void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }
            #pragma stage fragment
            layout(location = 0) in vec2 vTexCoord;
            layout(location = 0) out vec4 FragColor;
            layout(set = 0, binding = 2) uniform sampler2D Source;

            """;

        private string Preset(string name, string fragment, string keys)
        {
            File.WriteAllText(Path.Combine(_root, name + ".slang"), Vertex + fragment);
            string path = Path.Combine(_root, name + ".slangp");
            File.WriteAllText(path, $"shaders = 1\nshader0 = {name}.slang\n{keys}");
            return path;
        }

        private string Identity() => Preset("id", "void main() { FragColor = texture(Source, vTexCoord); }\n", "filter_linear0 = false\n");

        private string Invert() => Preset("invert", "void main() { FragColor = vec4(1.0 - texture(Source, vTexCoord).rgb, 1.0); }\n", "filter_linear0 = false\n");

        // Every pixel and every row differs, and the frame number is in blue.
        private static byte[] Picture(int w, int h, int frame)
        {
            var rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    int i = (y * w + x) * 4;
                    (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]) = ((byte)(x * 255 / Math.Max(1, w - 1)), (byte)(y * 255 / Math.Max(1, h - 1)), (byte)(20 + frame * 30), 255);
                }
            return rgba;
        }

        private static byte[] RepeatRows(byte[] rgba, int w, int h, int repeat)
        {
            var tall = new byte[rgba.Length * repeat];
            for (int y = 0; y < h; y++)
                for (int r = 0; r < repeat; r++) Array.Copy(rgba, y * w * 4, tall, (y * repeat + r) * w * 4, w * 4);
            return tall;
        }

        private static unsafe byte[] Pixels(SKImage image)
        {
            var pixels = new byte[image.Width * image.Height * 4];
            fixed (byte* p = pixels)
                Assert.True(image.ReadPixels(new SKImageInfo(image.Width, image.Height, SKColorType.Rgba8888, SKAlphaType.Opaque), (nint)p, image.Width * 4, 0, 0));
            return pixels;
        }

        [Fact]
        public void The_image_over_the_readback_is_the_picture_the_copy_gives()
        {
            if (_gpu is null) return;
            using var chain = new SlangChain(_gpu, SlangPreset.Load(Identity()));
            byte[] picture = Picture(16, 8, 0);
            chain.Advance(picture, 16, 8, 1);
            using SKImage image = chain.RenderImage(16, 8);
            Assert.Equal(picture, Pixels(image));
            Assert.Equal(picture, chain.Render(16, 8));
            Assert.Equal(0, chain.CopiedImages);
        }

        // An image Skia still holds is never written over: later frames take other readbacks, and past the limit a copy.
        [Fact]
        public void An_image_still_held_keeps_its_picture_while_later_frames_are_rendered()
        {
            if (_gpu is null) return;
            using var chain = new SlangChain(_gpu, SlangPreset.Load(Identity()));
            var held = new SKImage[SlangChain.MaxLent + 1];
            for (int frame = 0; frame < held.Length; frame++)
            {
                chain.Advance(Picture(16, 8, frame), 16, 8, 1);
                held[frame] = chain.RenderImage(16, 8);
            }
            Assert.Equal(SlangChain.MaxLent, chain.ReadbackCount);
            Assert.Equal(1, chain.CopiedImages);
            for (int frame = 0; frame < held.Length; frame++) Assert.Equal(Picture(16, 8, frame), Pixels(held[frame]));

            // Let the first go: the next frame takes its readback, and the others still hold theirs.
            held[0].Dispose();
            chain.Advance(Picture(16, 8, 5), 16, 8, 1);
            using (SKImage next = chain.RenderImage(16, 8)) Assert.Equal(Picture(16, 8, 5), Pixels(next));
            Assert.Equal(SlangChain.MaxLent, chain.ReadbackCount);
            Assert.Equal(1, chain.CopiedImages);
            for (int frame = 1; frame < held.Length; frame++) Assert.Equal(Picture(16, 8, frame), Pixels(held[frame]));
            for (int frame = 1; frame < held.Length; frame++) held[frame].Dispose();
        }

        // As SlangRunner draws, letting the last image go before rendering the next: one readback, no copy, for any number of frames.
        [Fact]
        public void An_image_let_go_before_the_next_render_leaves_one_readback_in_use()
        {
            if (_gpu is null) return;
            using var chain = new SlangChain(_gpu, SlangPreset.Load(Identity()));
            SKImage? shown = null;
            for (int frame = 0; frame < 60; frame++)
            {
                chain.Advance(Picture(16, 8, frame % 7), 16, 8, 1);
                shown?.Dispose();
                shown = chain.RenderImage(16, 8);
                Assert.Equal(Picture(16, 8, frame % 7), Pixels(shown));
            }
            shown!.Dispose();
            Assert.Equal(1, chain.ReadbackCount);
            Assert.Equal(0, chain.CopiedImages);
        }

        // A chain disposed under an image keeps that image's memory until the image lets go.
        [Fact]
        public void An_image_outlives_its_chain_with_its_picture_intact()
        {
            if (_gpu is null) return;
            SKImage image;
            using (var chain = new SlangChain(_gpu, SlangPreset.Load(Identity())))
            {
                chain.Advance(Picture(16, 8, 3), 16, 8, 1);
                image = chain.RenderImage(16, 8);
            }
            Assert.Equal(Picture(16, 8, 3), Pixels(image));
            image.Dispose();
        }

        // A readback grown for a larger view is not the one a held image reads.
        [Fact]
        public void A_larger_view_while_an_image_is_held_takes_a_new_readback()
        {
            if (_gpu is null) return;
            using var chain = new SlangChain(_gpu, SlangPreset.Load(Identity()));
            chain.Advance(Picture(16, 8, 1), 16, 8, 1);
            using SKImage small = chain.RenderImage(16, 8);
            using SKImage large = chain.RenderImage(32, 16);
            Assert.Equal(Picture(16, 8, 1), Pixels(small));
            Assert.Equal(RepeatRows(Widen(Picture(16, 8, 1), 16, 8), 32, 8, 2), Pixels(large));
        }

        private static byte[] Widen(byte[] rgba, int w, int h)
        {
            var wide = new byte[rgba.Length * 2];
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w * 2; x++) Array.Copy(rgba, (y * w + x / 2) * 4, wide, (y * w * 2 + x) * 4, 4);
            return wide;
        }

        // The blit repeats rows exactly: the same pictures as rows repeated on the host, sampled nearest and through a mip chain.
        [Theory]
        [InlineData(2, false)]
        [InlineData(3, false)]
        [InlineData(4, false)]
        [InlineData(2, true)]
        [InlineData(4, true)]
        public void Rows_repeated_on_the_device_are_the_rows_repeated_on_the_host(int repeat, bool mipmapped)
        {
            if (_gpu is null) return;
            string path = mipmapped
                ? Preset("mip", "void main() { FragColor = texture(Source, vTexCoord); }\n", "filter_linear0 = true\nmipmap_input0 = true\n")
                : Identity();
            using var device = new SlangChain(_gpu, SlangPreset.Load(path));
            using var host = new SlangChain(_gpu, SlangPreset.Load(path));
            const int w = 24, h = 10;
            (int vw, int vh) = mipmapped ? (7, 5) : (w, h * repeat);
            for (int frame = 0; frame < 3; frame++)
            {
                byte[] picture = Picture(w, h, frame);
                device.Advance(picture, w, h, repeat);
                host.Advance(RepeatRows(picture, w, h, repeat), w, h * repeat, 1);
                Assert.Equal(host.Render(vw, vh), device.Render(vw, vh));
            }
            if (!mipmapped) Assert.Equal(RepeatRows(Picture(w, h, 2), w, h, repeat), device.Render(vw, vh));
        }

        // Mars at four under a pack preset, the case the handheld misses frames on: rows repeated on the device draw what rows repeated on the host draw - see §9.2.
        [Theory]
        [InlineData("crt/crt-lottes.slangp", 2560, 960)]
        [InlineData("crt/crt-royale.slangp", 2560, 960)]
        [InlineData("crt/crt-lottes.slangp", 640, 240)]
        public void A_pack_preset_over_an_n64_frame_draws_the_same_from_rows_repeated_on_the_device(string relative, int w, int h)
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (_gpu is null || string.IsNullOrEmpty(pack)) return;
            SlangPreset preset = SlangPreset.Load(Path.Combine(pack, relative));
            using var device = new SlangChain(_gpu, preset);
            using var host = new SlangChain(_gpu, preset);
            for (int frame = 0; frame < 3; frame++)
            {
                byte[] picture = ShaderBench.Pattern(w, h, frame);
                device.Advance(picture, w, h, 2);
                host.Advance(RepeatRows(picture, w, h, 2), w, h * 2, 1);
                Assert.Equal(host.Render(1440, 1080), device.Render(1440, 1080));
            }
        }

        // The same frame at another repeat, or the same height made of other rows, is uploaded as what it is.
        [Fact]
        public void A_change_of_repeat_at_the_same_height_is_uploaded_as_the_new_shape()
        {
            if (_gpu is null) return;
            using var chain = new SlangChain(_gpu, SlangPreset.Load(Identity()));
            chain.Advance(Picture(8, 8, 0), 8, 8, 1);
            Assert.Equal(Picture(8, 8, 0), chain.Render(8, 8));
            chain.Advance(Picture(8, 4, 1), 8, 4, 2);
            Assert.Equal(RepeatRows(Picture(8, 4, 1), 8, 4, 2), chain.Render(8, 8));
            chain.Advance(Picture(8, 2, 2), 8, 2, 4);
            Assert.Equal(RepeatRows(Picture(8, 2, 2), 8, 2, 4), chain.Render(8, 8));
        }

        private static unsafe (byte R, byte G, byte B) Centre(SKSurface surface, int w, int h)
        {
            var pixels = new byte[w * h * 4];
            fixed (byte* p = pixels) surface.ReadPixels(new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, w * 4, 0, 0);
            int at = (h / 2 * w + w / 2) * 4;
            return (pixels[at], pixels[at + 1], pixels[at + 2]);
        }

        private static SlangRunner Built(string preset)
        {
            var runner = new SlangRunner(preset);
            var clock = Stopwatch.StartNew();
            while (!runner.Built && clock.ElapsedMilliseconds < 20000) System.Threading.Thread.Sleep(5);
            Assert.True(runner.Ready, runner.Problem);
            return runner;
        }

        // On a real GL context: each new frame is shown, and a redraw after Skia dropped its texture reads the same frame again, never the next - see §8.8.
        [Fact]
        public void On_a_gl_canvas_every_draw_shows_its_own_frame_even_when_skia_uploads_it_again()
        {
            if (_gpu is null || Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable) is not { Length: > 0 } glDevice) return;
            using var gl = ShaderBench.GlContext.Create(glDevice);
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(name => ShaderBench.GlContext.GetProc(name))!;
            using GRContext context = GRContext.CreateGl(glInterface)!;
            using SKSurface target = SKSurface.Create(context, true, new SKImageInfo(64, 64, SKColorType.Rgba8888, SKAlphaType.Premul))!;
            using SlangRunner runner = Built(Invert());
            var destination = new SKRect(0, 0, 64, 64);
            for (int frame = 0; frame < 6; frame++)
            {
                byte[] solid = new byte[8 * 8 * 4];
                for (int i = 0; i < solid.Length; i += 4) (solid[i], solid[i + 1], solid[i + 2], solid[i + 3]) = ((byte)(frame * 40), 100, 30, 255);
                (byte, byte, byte) expected = ((byte)(255 - frame * 40), 155, 225);
                Assert.True(runner.Draw(target.Canvas, solid, 8, 8, 1, true, destination));
                context.Flush();
                Assert.Equal(expected, Centre(target, 64, 64));
                context.PurgeResources();
                target.Canvas.Clear(SKColors.Black);
                Assert.True(runner.Draw(target.Canvas, solid, 8, 8, 1, false, destination));
                context.Flush();
                Assert.Equal(expected, Centre(target, 64, 64));
            }
        }

        // The runner lets each image go before the next render, so a running preset keeps one readback however many frames it draws.
        [Fact]
        public void A_running_preset_draws_every_frame_from_one_readback_and_copies_none()
        {
            if (_gpu is null) return;
            using SlangRunner runner = Built(Invert());
            using var surface = SKSurface.Create(new SKImageInfo(32, 32, SKColorType.Rgba8888, SKAlphaType.Premul));
            for (int frame = 0; frame < 30; frame++)
            {
                byte[] picture = Picture(8, 8, frame % 5);
                Assert.True(runner.Draw(surface.Canvas, picture, 8, 8, 1, true, new SKRect(0, 0, 32, 32)));
                Assert.True(runner.Draw(surface.Canvas, picture, 8, 8, 1, false, new SKRect(0, 0, 32, 32)));
            }
            Assert.Equal(1, runner.Chain!.ReadbackCount);
            Assert.Equal(0, runner.Chain.CopiedImages);
        }

        private static byte[] Solid(byte r, byte g, byte b)
        {
            var frame = new byte[8 * 8 * 4];
            for (int i = 0; i < frame.Length; i += 4) (frame[i], frame[i + 1], frame[i + 2], frame[i + 3]) = (r, g, b, 255);
            return frame;
        }

        private static (byte R, byte G, byte B) Drawn(GameFrameControl control)
        {
            using GameFrameControl.DrawOp op = control.CaptureDrawOp(new Avalonia.Size(8, 8))!;
            using var surface = SKSurface.Create(new SKImageInfo(8, 8, SKColorType.Rgba8888, SKAlphaType.Premul));
            surface.Canvas.Clear(SKColors.Black);
            op.RenderTo(surface.Canvas, null);
            SKColor centre = surface.PeekPixels().GetPixelColor(4, 4);
            return (centre.Red, centre.Green, centre.Blue);
        }

        private static void WaitBuilt(GameFrameControl control)
        {
            var clock = Stopwatch.StartNew();
            while (!control.SlangBuilt && clock.ElapsedMilliseconds < 20000) System.Threading.Thread.Sleep(5);
        }

        // While a preset draws, the control makes no image of the source; turned off with no new frame, the newest frame is shown, not one copied before the preset was built.
        [Fact]
        public async Task A_preset_that_draws_takes_no_source_image_and_turning_it_off_shows_the_newest_frame()
        {
            if (_gpu is null) return;
            string preset = Invert();
            var seen = await Session.Dispatch(() =>
            {
                var control = new GameFrameControl();
                control.UpdateFrame(Solid(200, 0, 0), 8, 8);
                var plain = Drawn(control);
                control.ActiveSlangPreset = preset;
                WaitBuilt(control);
                control.TakeStatistics();
                control.UpdateFrame(Solid(0, 200, 0), 8, 8);
                var inverted = Drawn(control);
                var redrawn = Drawn(control);
                long copies = control.TakeStatistics().Copies;
                control.ActiveSlangPreset = null;
                var off = Drawn(control);
                return (plain, inverted, redrawn, copies, off);
            }, default);
            Assert.Equal(((byte)200, (byte)0, (byte)0), seen.plain);
            Assert.Equal(((byte)255, (byte)55, (byte)255), seen.inverted);
            Assert.Equal(seen.inverted, seen.redrawn);
            Assert.Equal(0, seen.copies);
            Assert.Equal(((byte)0, (byte)200, (byte)0), seen.off);
        }
    }
}
