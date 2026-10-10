using System;
using System.Collections.Concurrent;
using System.IO;
using System.Linq;
using System.Threading;
using EmuSen.Serenity.Slang;
using SkiaSharp;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // Four defects the chain parity class found in the C# slang runtime, each held to what the library does, under the Khronos layer on whichever device the switch gives - see EmuSen_RustPlatform.md §18.
    public sealed class SlangDefectTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangDefects", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;
        private readonly string _wanted = Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800";

        public SlangDefectTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private const string Plain = """
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

            """;

        private string Write(string name, string text)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllText(path, text);
            return path;
        }

        private string Identity()
        {
            Write("id.slang", Plain + "layout(set = 0, binding = 1) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            return Write("id.slangp", "shaders = 1\nshader0 = id.slang\n");
        }

        // A device of the switch's kind with the layer and a sink, or none where the layer or the card is not there.
        private (SlangVulkan? Gpu, ConcurrentQueue<string> Findings) Validated()
        {
            var findings = new ConcurrentQueue<string>();
            SlangVulkan? gpu = SlangVulkan.TryCreate(out string report, _wanted, findings.Enqueue);
            _output.WriteLine($"{(gpu?.IsNative == true ? "library" : "C#")}: {report}");
            return (gpu, findings);
        }

        private void AssertSilent(ConcurrentQueue<string> findings)
        {
            foreach (string finding in findings.Take(12)) _output.WriteLine(finding.Split('\n')[0]);
            Assert.True(findings.IsEmpty, $"the layer said {findings.Count} things, the first: {findings.FirstOrDefault()?.Split('\n')[0]}");
        }

        // Defect 1: a chain that fails once its passes are built gives back everything it made, so the device ends holding nothing.
        [Fact]
        public void A_chain_that_fails_after_its_passes_are_built_leaves_nothing_on_the_device()
        {
            var (gpu, findings) = Validated();
            if (gpu is null) return;
            Write("id.slang", Plain + "layout(set = 0, binding = 1) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n");
            Write("deep.slang", Plain + "layout(set = 0, binding = 1) uniform sampler2D OriginalHistory2147483647;\nvoid main() { FragColor = texture(OriginalHistory2147483647, vTexCoord); }\n");
            File.WriteAllText(Path.Combine(_root, "not.png"), "this is not a picture");
            string undecoded = Write("undecoded.slangp", "shaders = 2\nshader0 = id.slang\nshader1 = id.slang\ntextures = \"Lut\"\nLut = not.png\n");
            string deep = Write("deep.slangp", "shaders = 2\nshader0 = id.slang\nshader1 = deep.slang\n");
            int live;
            using (gpu)
            {
                Assert.Throws<InvalidDataException>(() => new SlangChain(gpu, SlangPreset.Load(undecoded)));
                Assert.Throws<OverflowException>(() => new SlangChain(gpu, SlangPreset.Load(deep)));
                live = gpu.IsNative ? 0 : gpu.ManagedValue.Live;
            }
            // What the device still held at its end: the buffers and images it counts, and every object the layer saw left.
            int left = findings.Count(f => f.Contains("VUID-vkDestroyDevice-device-05137", StringComparison.Ordinal));
            Assert.True(live == 0 && left == 0, $"{live} buffers and images were never given back, and the layer found {left} objects left at the device's end");
            AssertSilent(findings);
        }

        // Defect 2: the device is waited for under the lock its queue is used under, so two chains on two threads never use the queue at once.
        [Fact]
        public void Two_chains_on_two_threads_never_use_the_device_s_queue_at_once()
        {
            var (gpu, findings) = Validated();
            if (gpu is null) return;
            Write("blur.slang", Plain + "layout(set = 0, binding = 1) uniform sampler2D Source;\nvoid main() { vec4 c = vec4(0.0); for (int i = 0; i < 64; i++) c += texture(Source, vTexCoord + vec2(float(i) * 0.001)); FragColor = c / 64.0; }\n");
            string busy = Write("busy.slangp", "shaders = 3\nshader0 = blur.slang\nscale_type0 = viewport\nshader1 = blur.slang\nscale_type1 = viewport\nshader2 = blur.slang\n");
            string preset = Identity();
            using (gpu)
            {
                // One chain keeps the queue busy with long submissions; the other takes a new size each frame, and with it the device's wait.
                var start = new Barrier(2);
                var other = new Thread(() =>
                {
                    using var chain = new SlangChain(gpu, SlangPreset.Load(busy));
                    start.SignalAndWait();
                    for (int k = 0; k < 120; k++)
                    {
                        chain.Advance(ShaderBench.Pattern(64, 48, k), 64, 48, 1);
                        chain.Render(640, 480);
                    }
                });
                other.Start();
                using (var chain = new SlangChain(gpu, SlangPreset.Load(preset)))
                {
                    start.SignalAndWait();
                    for (int k = 0; k < 400; k++)
                    {
                        int w = 8 + k % 5, h = 6 + k % 4;
                        chain.Advance(ShaderBench.Pattern(w, h, k), w, h, 1);
                    }
                }
                other.Join();
                gpu.Heard();
            }
            AssertSilent(findings);
        }
    }
}
