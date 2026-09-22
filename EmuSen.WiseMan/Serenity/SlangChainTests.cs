using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Serenity.Slang;
using SkiaSharp;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // Presets run on the device, read back and checked by pixel: libretro's texture and uniform semantics - see EmuSen_Serenity.md §7.4.
    public class SlangChainTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangChain", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;
        private readonly SlangVulkan? _gpu;

        public SlangChainTests(ITestOutputHelper output)
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

        private const string Head = """
            #version 450
            layout(push_constant) uniform Push
            {
                vec4 SourceSize;
                vec4 OutputSize;
                uint FrameCount;
                float gain;
            } params;
            #pragma parameter gain "Gain" 1.0 0.0 1.0 0.1
            layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;
            #pragma stage vertex
            layout(location = 0) in vec4 Position;
            layout(location = 1) in vec2 TexCoord;
            layout(location = 0) out vec2 vTexCoord;
            void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }
            #pragma stage fragment
            layout(location = 0) in vec2 vTexCoord;
            layout(location = 0) out vec4 FragColor;

            """;

        private string Shader(string name, string fragment)
        {
            File.WriteAllText(Path.Combine(_root, name), Head + fragment);
            return name;
        }

        private SlangPreset Preset(string text)
        {
            string path = Path.Combine(_root, $"p{Guid.NewGuid():N}.slangp");
            File.WriteAllText(path, text);
            return SlangPreset.Load(path);
        }

        // A 4x2 picture whose every pixel differs: red is x, green is y, blue fixed.
        private static byte[] Picture(int w = 4, int h = 2, byte blue = 200)
        {
            var rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    int i = (y * w + x) * 4;
                    rgba[i] = (byte)(x * 60);
                    rgba[i + 1] = (byte)(y * 100);
                    rgba[i + 2] = blue;
                    rgba[i + 3] = 255;
                }
            return rgba;
        }

        private static byte[] Pixel(byte[] rgba, int w, int x, int y) => rgba.AsSpan((y * w + x) * 4, 4).ToArray();

        [Fact]
        public void An_identity_pass_gives_back_the_picture()
        {
            if (_gpu is null) return;
            Shader("id.slang", "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            using var chain = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = id.slang\nfilter_linear0 = false\n"));
            byte[] picture = Picture();
            chain.Advance(picture, 4, 2, 1);
            Assert.Equal(picture, chain.Render(4, 2));
        }

        [Fact]
        public void Repeated_rows_are_expanded_before_the_first_pass()
        {
            if (_gpu is null) return;
            Shader("id.slang", "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            using var chain = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = id.slang\nfilter_linear0 = false\n"));
            chain.Advance(Picture(), 4, 2, 2);
            byte[] output = chain.Render(4, 4);
            Assert.Equal(Pixel(output, 4, 3, 0), Pixel(output, 4, 3, 1));
            Assert.Equal(Pixel(output, 4, 3, 2), Pixel(output, 4, 3, 3));
            Assert.Equal(100, Pixel(output, 4, 0, 2)[1]);
        }

        [Fact]
        public void A_later_pass_reads_an_earlier_one_by_alias_and_by_number_and_the_original_too()
        {
            if (_gpu is null) return;
            Shader("invert.slang", "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { vec4 c = texture(Source, vTexCoord); FragColor = vec4(1.0 - c.rgb, 1.0); }\n");
            Shader("mix.slang", """
                layout(set = 0, binding = 2) uniform sampler2D Inverted;
                layout(set = 0, binding = 3) uniform sampler2D PassOutput0;
                layout(set = 0, binding = 4) uniform sampler2D Original;
                void main() { FragColor = vec4(texture(Inverted, vTexCoord).r, texture(PassOutput0, vTexCoord).g, texture(Original, vTexCoord).b, 1.0); }
                """);
            using var chain = new SlangChain(_gpu, Preset("shaders = 2\nshader0 = invert.slang\nalias0 = Inverted\nshader1 = mix.slang\nfilter_linear0 = false\nfilter_linear1 = false\n"));
            chain.Advance(Picture(), 4, 2, 1);
            byte[] output = chain.Render(4, 2);
            Assert.Equal(new byte[] { 255 - 180, 255 - 100, 200, 255 }, Pixel(output, 4, 3, 1));
        }

        [Fact]
        public void A_pass_that_reads_its_own_feedback_accumulates_across_frames()
        {
            if (_gpu is null) return;
            Shader("count.slang", "layout(set = 0, binding = 2) uniform sampler2D PassFeedback0;\nvoid main() { FragColor = vec4(texture(PassFeedback0, vTexCoord).r + 10.0 / 255.0, 0.0, 0.0, 1.0); }\n");
            using var chain = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = count.slang\nfilter_linear0 = false\n"));
            byte[] output = Array.Empty<byte>();
            for (int frame = 0; frame < 3; frame++)
            {
                chain.Advance(Picture(), 4, 2, 1);
                output = chain.Render(4, 2);
            }
            Assert.Equal(30, output[0]);
        }

        [Fact]
        public void OriginalHistory_n_is_the_frame_n_before()
        {
            if (_gpu is null) return;
            Shader("history.slang", "layout(set = 0, binding = 2) uniform sampler2D OriginalHistory1;\nlayout(set = 0, binding = 3) uniform sampler2D Original;\nlayout(set = 0, binding = 4) uniform sampler2D OriginalHistory2;\nvoid main() { FragColor = vec4(texture(OriginalHistory1, vTexCoord).b, texture(Original, vTexCoord).b, texture(OriginalHistory2, vTexCoord).b, 1.0); }\n");
            using var chain = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = history.slang\nfilter_linear0 = false\n"));
            chain.Advance(Picture(blue: 10), 4, 2, 1);
            Assert.Equal(new byte[] { 0, 10, 0 }, chain.Render(4, 2)[..3]);
            chain.Advance(Picture(blue: 20), 4, 2, 1);
            Assert.Equal(new byte[] { 10, 20, 0 }, chain.Render(4, 2)[..3]);
            chain.Advance(Picture(blue: 30), 4, 2, 1);
            Assert.Equal(new byte[] { 20, 30, 10 }, chain.Render(4, 2)[..3]);
            chain.Advance(Picture(blue: 40), 4, 2, 1);
            Assert.Equal(new byte[] { 30, 40, 20 }, chain.Render(4, 2)[..3]);
        }

        [Fact]
        public void The_preset_overrides_a_parameter_and_the_pragma_supplies_the_rest()
        {
            if (_gpu is null) return;
            Shader("gain.slang", "void main() { FragColor = vec4(params.gain, 0.0, 0.0, 1.0); }\n");
            using (var plain = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = gain.slang\n")))
            {
                plain.Advance(Picture(), 4, 2, 1);
                Assert.Equal(255, plain.Render(4, 2)[0]);
            }
            using var overridden = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = gain.slang\ngain = 0.2\n"));
            overridden.Advance(Picture(), 4, 2, 1);
            Assert.Equal(51, overridden.Render(4, 2)[0]);
        }

        [Fact]
        public void Sizes_are_given_as_width_height_and_reciprocals_and_a_scaled_pass_is_that_size()
        {
            if (_gpu is null) return;
            Shader("double.slang", "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = vec4(params.OutputSize.x / 255.0, params.OutputSize.w * 255.0 / 255.0, params.SourceSize.x / 255.0, 1.0); }\n");
            Shader("report.slang", "layout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = vec4(texture(Source, vTexCoord).r, params.SourceSize.y / 255.0, params.OutputSize.x / 255.0, 1.0); }\n");
            using var chain = new SlangChain(_gpu, Preset("shaders = 2\nshader0 = double.slang\nscale_type0 = source\nscale0 = 3.0\nshader1 = report.slang\nfilter_linear0 = false\n"));
            chain.Advance(Picture(), 4, 2, 1);
            byte[] output = chain.Render(20, 10);
            Assert.Equal(new byte[] { 12, 6, 20, 255 }, Pixel(output, 20, 0, 0));
        }

        [Fact]
        public void A_lookup_texture_is_loaded_and_sampled_by_its_name()
        {
            if (_gpu is null) return;
            using (var bitmap = new SKBitmap(2, 1))
            {
                bitmap.SetPixel(0, 0, new SKColor(11, 22, 33));
                bitmap.SetPixel(1, 0, new SKColor(44, 55, 66));
                using var data = bitmap.Encode(SKEncodedImageFormat.Png, 100);
                File.WriteAllBytes(Path.Combine(_root, "lut.png"), data.ToArray());
            }
            Shader("lut.slang", "layout(set = 0, binding = 2) uniform sampler2D Lut;\nvoid main() { FragColor = texture(Lut, vTexCoord); }\n");
            using var chain = new SlangChain(_gpu, Preset("shaders = 1\nshader0 = lut.slang\ntextures = \"Lut\"\nLut = lut.png\nLut_linear = false\n"));
            chain.Advance(Picture(), 4, 2, 1);
            byte[] output = chain.Render(2, 1);
            Assert.Equal(new byte[] { 11, 22, 33, 255, 44, 55, 66, 255 }, output);
        }

        // The pack's heaviest CRT presets build, run ten frames and draw something, when a pack is given.
        [Theory]
        [InlineData("crt/crt-royale.slangp")]
        [InlineData("crt/crt-guest-advanced.slangp")]
        [InlineData("crt/crt-lottes.slangp")]
        [InlineData("handheld/lcd-grid-v2.slangp")]
        public void A_pack_preset_runs(string relative)
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (_gpu is null || string.IsNullOrEmpty(pack)) return;
            var clock = Stopwatch.StartNew();
            using var chain = new SlangChain(_gpu, SlangPreset.Load(Path.Combine(pack, relative)));
            _output.WriteLine($"built in {clock.ElapsedMilliseconds} ms, {chain.Preset.Passes.Count} passes, {chain.Parameters.Count} parameters");
            byte[] picture = Picture(256, 224, 128);
            byte[] output = Array.Empty<byte>();
            clock.Restart();
            for (int frame = 0; frame < 10; frame++)
            {
                chain.Advance(picture, 256, 224, 1);
                output = chain.Render(1024, 896);
            }
            _output.WriteLine($"ten frames in {clock.ElapsedMilliseconds} ms");
            Assert.Contains(output.Where((_, i) => i % 4 != 3), b => b > 16);
        }
    }
}

namespace EmuSen.WiseMan.Serenity
{
    // A preset set on GameFrameControl, built off the render thread and drawn through its real render pass - see EmuSen_Serenity.md §7.5.
    public class SlangFrameControlTests : IDisposable
    {
        private static readonly Avalonia.Headless.HeadlessUnitTestSession Session =
            Avalonia.Headless.HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SlangFrameControlTests).Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangFrame", Guid.NewGuid().ToString("N"));

        public SlangFrameControlTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Invert()
        {
            File.WriteAllText(Path.Combine(_root, "invert.slang"), """
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
                void main() { FragColor = vec4(1.0 - texture(Source, vTexCoord).rgb, 1.0); }
                """);
            string preset = Path.Combine(_root, "invert.slangp");
            File.WriteAllText(preset, "shaders = 1\nshader0 = invert.slang\nfilter_linear0 = false\n");
            return preset;
        }

        private static (int R, int G, int B) Centre(string? preset, Action<EmuSen.Serenity.GameFrameControl>? before = null)
        {
            var control = new EmuSen.Serenity.GameFrameControl { ActiveSlangPreset = preset };
            before?.Invoke(control);
            var window = new Avalonia.Controls.Window { Width = 64, Height = 64, Content = control };
            window.Show();
            try
            {
                var clock = Stopwatch.StartNew();
                while (!control.SlangBuilt && clock.ElapsedMilliseconds < 20000) System.Threading.Thread.Sleep(10);
                var frame = new byte[8 * 8 * 4];
                for (int i = 0; i < frame.Length; i += 4) (frame[i], frame[i + 1], frame[i + 2], frame[i + 3]) = (10, 20, 30, 255);
                control.UpdateFrame(frame, 8, 8);
                using var captured = Avalonia.Headless.HeadlessWindowExtensions.CaptureRenderedFrame(window)!;
                var capture = EmuSen.WiseMan.Fixtures.UiTest.Capture(captured);
                int at = (32 * capture.Width + 32) * 4;
                return (capture.Rgba[at], capture.Rgba[at + 1], capture.Rgba[at + 2]);
            }
            finally
            {
                window.Close();
            }
        }

        [Fact]
        public async System.Threading.Tasks.Task A_preset_is_drawn_once_it_is_built()
        {
            if (SlangVulkan.TryCreate(out _) is not { } probe) return;
            probe.Dispose();
            string preset = Invert();
            var centre = await Session.Dispatch(() => Centre(preset), default);
            Assert.Equal((245, 235, 225), centre);
        }

        [Fact]
        public async System.Threading.Tasks.Task A_preset_that_cannot_be_built_says_why_and_the_picture_is_drawn_plain()
        {
            File.WriteAllText(Path.Combine(_root, "broken.slang"), "#version 450\n#pragma stage vertex\nvoid main() { nope(); }\n#pragma stage fragment\nvoid main() { }\n");
            string preset = Path.Combine(_root, "broken.slangp");
            File.WriteAllText(preset, "shaders = 1\nshader0 = broken.slang\n");
            string? problem = null;
            var centre = await Session.Dispatch(() => Centre(preset, c => c.SlangFailed += p => problem = p), default);
            Assert.Equal((10, 20, 30), centre);
            Assert.NotNull(problem);
            Assert.Contains("broken", problem);
        }
    }
}
