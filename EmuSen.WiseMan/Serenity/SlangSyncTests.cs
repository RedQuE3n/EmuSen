using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Serenity.Slang;
using Silk.NET.Vulkan;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // A chain run under the Khronos layer's synchronisation validation, in process, and the fragment inputs no instruction reads - see EmuSen_Serenity.md §10.
    public class SlangSyncTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangSync", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;

        public SlangSyncTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private const string Head = """
            #version 450
            layout(push_constant) uniform Push { vec4 SourceSize; uint FrameCount; } params;
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

        // Four passes: an earlier output read by Source, by alias, through a mip chain and from a vertex shader, a pass's own feedback, the history, and a fragment input the vertex stage never writes.
        private string Preset()
        {
            File.WriteAllText(Path.Combine(_root, "first.slang"), Head + """
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D OriginalHistory1;
                void main() { FragColor = texture(Source, vTexCoord) * 0.75 + texture(OriginalHistory1, vTexCoord) * 0.25; }
                """);
            File.WriteAllText(Path.Combine(_root, "second.slang"), Head + """
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D PassFeedback1;
                void main() { FragColor = texture(Source, vTexCoord) * 0.5 + texture(PassFeedback1, vTexCoord) * 0.5; }
                """);
            File.WriteAllText(Path.Combine(_root, "third.slang"), Head + """
                layout(location = 8) in vec3 Unread;
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D First;
                void main() { FragColor = vec4(texture(Source, vTexCoord).rgb * 0.9 + texture(First, vTexCoord).rgb * 0.1, 1.0); }
                """);
            File.WriteAllText(Path.Combine(_root, "fourth.slang"), """
                #version 450
                layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;
                layout(set = 0, binding = 1) uniform sampler2D Source;
                layout(set = 0, binding = 2) uniform sampler2D First;
                #pragma stage vertex
                layout(location = 0) in vec4 Position;
                layout(location = 1) in vec2 TexCoord;
                layout(location = 0) out vec2 vTexCoord;
                layout(location = 1) out vec3 Tint;
                void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; Tint = textureLod(Source, vec2(0.5), 0.0).rgb; }
                #pragma stage fragment
                layout(location = 0) in vec2 vTexCoord;
                layout(location = 1) in vec3 Tint;
                layout(location = 0) out vec4 FragColor;
                void main() { FragColor = vec4(texture(First, vTexCoord).rgb * 0.9 + Tint * 0.1, 1.0); }
                """);
            string path = Path.Combine(_root, "sync.slangp");
            File.WriteAllText(path, """
                shaders = 4
                shader0 = first.slang
                alias0 = First
                scale_type0 = source
                scale0 = 2.0
                filter_linear0 = true
                shader1 = second.slang
                mipmap_input1 = true
                filter_linear1 = true
                scale_type1 = source
                scale1 = 1.0
                shader2 = third.slang
                filter_linear2 = true
                scale_type2 = source
                scale2 = 1.0
                shader3 = fourth.slang
                filter_linear3 = true
                scale_type3 = viewport
                """);
            return path;
        }

        // Each finding the layer hands the sink, and whether the layer is there at all.
        private static (SlangVulkan? Gpu, ConcurrentQueue<string> Findings) Validated()
        {
            var findings = new ConcurrentQueue<string>();
            SlangVulkan? gpu = SlangVulkan.TryCreate(out _, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800", findings.Enqueue);
            return (gpu, findings);
        }

        // The layer's silence proves something only if it speaks: a zero-sized buffer is a finding it must report.
        private static unsafe void AssertTheLayerIsListening(SlangVulkan gpu, ConcurrentQueue<string> findings)
        {
            var info = new BufferCreateInfo { SType = StructureType.BufferCreateInfo, Size = 0, Usage = BufferUsageFlags.TransferSrcBit, SharingMode = SharingMode.Exclusive };
            if (gpu.Vk.CreateBuffer(gpu.Device, &info, null, out Silk.NET.Vulkan.Buffer buffer) == Result.Success) gpu.Vk.DestroyBuffer(gpu.Device, buffer, null);
            Assert.Contains(findings, f => f.Contains("VUID-VkBufferCreateInfo-size-00912", StringComparison.Ordinal));
        }

        private void Run(SlangVulkan gpu, string preset, int width, int height, int repeat, int frames)
        {
            using var chain = new SlangChain(gpu, SlangPreset.Load(preset));
            for (int frame = 0; frame < frames; frame++)
            {
                chain.Advance(ShaderBench.Pattern(width, height, frame), width, height, repeat);
                chain.Render(320, 240);
            }
            using (chain.RenderImage(320, 240)) { }
        }

        [Fact]
        public void A_chain_of_passes_that_read_each_other_runs_clean_under_synchronisation_validation()
        {
            var (gpu, findings) = Validated();
            if (gpu is null) return;
            using (gpu)
            {
                Run(gpu, Preset(), 16, 12, 1, 4);
                Run(gpu, Preset(), 16, 6, 2, 4);
                foreach (string finding in findings) _output.WriteLine(finding);
                Assert.Empty(findings);
                AssertTheLayerIsListening(gpu, findings);
            }
        }

        [Theory]
        [InlineData("crt/crt-royale.slangp")]
        [InlineData("crt/crt-guest-advanced.slangp")]
        [InlineData("bezel/Mega_Bezel/Presets/MBZ__5__POTATO.slangp")]
        public void A_pack_preset_runs_clean_under_synchronisation_validation(string relative)
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;
            var (gpu, findings) = Validated();
            if (gpu is null) return;
            using (gpu)
            {
                Run(gpu, Path.Combine(pack, relative), 256, 224, 1, 3);
                foreach (string finding in findings.Take(20)) _output.WriteLine(finding);
                Assert.Empty(findings);
                AssertTheLayerIsListening(gpu, findings);
            }
        }

        private SlangSource Source(string name, string fragment)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllText(path, Head + fragment);
            return SlangSource.Load(path);
        }

        private static string[] InterfaceNames(byte[] spirv)
        {
            uint[] words = new uint[spirv.Length / 4];
            System.Buffer.BlockCopy(spirv, 0, words, 0, spirv.Length);
            var names = new Dictionary<uint, string>();
            var ids = new List<uint>();
            for (int at = 5; at < words.Length;)
            {
                int count = (int)(words[at] >> 16), op = (int)(words[at] & 0xFFFF);
                if (op == 5) names[words[at + 1]] = System.Text.Encoding.UTF8.GetString(MemoryExtensions.AsSpan(spirv, (at + 2) * 4, (count - 2) * 4)).TrimEnd('\0');
                if (op == 15)
                {
                    int name = at + 3;
                    while (!BitConverter.GetBytes(words[name]).Contains((byte)0)) name++;
                    ids.AddRange(words[(name + 1)..(at + count)]);
                }
                at += count;
            }
            return ids.Select(id => names.GetValueOrDefault(id, $"%{id}")).ToArray();
        }

        [Fact]
        public void An_input_no_instruction_reads_leaves_the_interface_and_one_that_is_read_stays()
        {
            SlangSource source = Source("unread.slang", """
                layout(location = 8) in vec3 Unread;
                layout(location = 3) in float AlsoUnread;
                layout(set = 0, binding = 1) uniform sampler2D Source;
                void main() { FragColor = texture(Source, vTexCoord); }
                """);
            byte[] fragment = SlangCompiler.Compile(source.Fragment, SlangStage.Fragment, "unread.slang");
            Assert.Contains("Unread", InterfaceNames(fragment));

            byte[] pruned = SpirvReflection.WithoutUnreadInputs(fragment);
            string[] kept = InterfaceNames(pruned);
            _output.WriteLine(string.Join(", ", kept));
            Assert.Contains("vTexCoord", kept);
            Assert.Contains("FragColor", kept);
            Assert.DoesNotContain("Unread", kept);
            Assert.DoesNotContain("AlsoUnread", kept);
            Assert.Equal(fragment.Length - 8, pruned.Length);
            Assert.Equal(SpirvReflection.Read(fragment).Samplers, SpirvReflection.Read(pruned).Samplers);
        }

        [Fact]
        public void A_module_whose_every_input_is_read_is_given_back_as_it_was()
        {
            SlangSource source = Source("read.slang", """
                layout(set = 0, binding = 1) uniform sampler2D Source;
                void main() { FragColor = texture(Source, vTexCoord); }
                """);
            byte[] fragment = SlangCompiler.Compile(source.Fragment, SlangStage.Fragment, "read.slang");
            Assert.Same(fragment, SpirvReflection.WithoutUnreadInputs(fragment));
        }
    }

    // A progressive frame reaches a preset with its rows once, as a libretro core hands it over, so no preset takes it for an interlaced one - see EmuSen_Serenity.md §10.6.
    public class SlangRowsOnceTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangRows", Guid.NewGuid().ToString("N"));

        public SlangRowsOnceTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static SlangRunner Built(string preset)
        {
            var runner = new SlangRunner(preset);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!runner.Built && clock.ElapsedMilliseconds < 60000) System.Threading.Thread.Sleep(5);
            return runner;
        }

        private static string Hash(ReadOnlySpan<byte> pixels) => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(pixels));

        [Fact]
        public void A_progressive_frame_reaches_the_preset_with_its_rows_once()
        {
            if (SlangVulkan.TryCreate(out _) is not { } probe) return;
            probe.Dispose();
            File.WriteAllText(Path.Combine(_root, "size.slang"), """
                #version 450
                layout(push_constant) uniform Push { vec4 OriginalSize; } params;
                layout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;
                #pragma stage vertex
                layout(location = 0) in vec4 Position;
                layout(location = 1) in vec2 TexCoord;
                layout(location = 0) out vec2 vTexCoord;
                void main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }
                #pragma stage fragment
                layout(location = 0) in vec2 vTexCoord;
                layout(location = 0) out vec4 FragColor;
                void main() { FragColor = vec4(params.OriginalSize.y / 255.0, params.OriginalSize.x / 255.0, 0.0, 1.0); }
                """);
            string preset = Path.Combine(_root, "size.slangp");
            File.WriteAllText(preset, "shaders = 1\nshader0 = size.slang\n");
            using SlangRunner runner = Built(preset);

            using var surface = SkiaSharp.SKSurface.Create(new SkiaSharp.SKImageInfo(16, 16, SkiaSharp.SKColorType.Rgba8888, SkiaSharp.SKAlphaType.Premul));
            Assert.True(runner.Draw(surface.Canvas, new byte[12 * 8 * 4], 12, 8, newFrame: true, new SkiaSharp.SKRect(0, 0, 16, 16)));
            using var pixels = surface.PeekPixels();
            SkiaSharp.SKColor centre = pixels.GetPixelColor(8, 8);
            Assert.Equal((8, 12), ((int)centre.Red, (int)centre.Green));
        }

        // Royale emulates interlacing on 288.5 to 576.5 lines, fields alternating by FrameCount: a still frame as 480 rows alternates, as 240 it holds.
        [Fact]
        public void A_still_progressive_frame_under_royale_draws_the_same_picture_every_frame()
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;
            if (SlangVulkan.TryCreate(out _) is not { } gpu) return;
            string preset = Path.Combine(pack, "crt/crt-royale.slangp");
            byte[] still = ShaderBench.Pattern(320, 240, 0);

            using (gpu)
            using (var chain = new SlangChain(gpu, SlangPreset.Load(preset)))
            {
                var doubled = new List<string>();
                for (int f = 0; f < 4; f++)
                {
                    chain.Advance(still, 320, 240, 2);
                    doubled.Add(Hash(chain.Render(320, 480)));
                }
                Assert.NotEqual(doubled[0], doubled[1]);
                Assert.Equal(doubled[0], doubled[2]);
            }

            using SlangRunner runner = Built(preset);
            using var surface = SkiaSharp.SKSurface.Create(new SkiaSharp.SKImageInfo(320, 480, SkiaSharp.SKColorType.Rgba8888, SkiaSharp.SKAlphaType.Premul));
            var drawn = new List<string>();
            for (int f = 0; f < 4; f++)
            {
                Assert.True(runner.Draw(surface.Canvas, still, 320, 240, newFrame: true, new SkiaSharp.SKRect(0, 0, 320, 480)));
                using var pixels = surface.PeekPixels();
                drawn.Add(Hash(pixels.GetPixelSpan()));
            }
            Assert.Single(drawn.Distinct());
        }
    }
}
