using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Serenity.Slang;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // A slang pass compiled to SPIR-V and read back: the members the frontend fills, where they sit, and the textures it binds - see EmuSen_Serenity.md §7.3.
    public class SlangCompileTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangCompile", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;

        public SlangCompileTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // The shape nearly every libretro pass has: MVP and sizes in a block at binding 0, parameters in push constants, textures after.
        private const string Pass = """
            #version 450
            layout(push_constant) uniform Push
            {
                vec4 SourceSize;
                float hardScan;
                uint FrameCount;
            } params;
            #pragma parameter hardScan "Scanline hardness" -8.0 -20.0 0.0 1.0
            layout(std140, set = 0, binding = 0) uniform UBO
            {
                mat4 MVP;
                vec4 OutputSize;
                vec4 OriginalSize;
                float lut_strength;
            } global;
            #pragma stage vertex
            layout(location = 0) in vec4 Position;
            layout(location = 1) in vec2 TexCoord;
            layout(location = 0) out vec2 vTexCoord;
            void main()
            {
               gl_Position = global.MVP * Position;
               vTexCoord = TexCoord;
            }
            #pragma stage fragment
            layout(location = 0) in vec2 vTexCoord;
            layout(location = 0) out vec4 FragColor;
            layout(set = 0, binding = 2) uniform sampler2D Source;
            layout(set = 0, binding = 3) uniform sampler2D MaskLut;
            void main()
            {
               vec3 c = texture(Source, vTexCoord).rgb * params.hardScan * float(params.FrameCount) + texture(MaskLut, vTexCoord * params.SourceSize.xy).rgb;
               FragColor = vec4(c * global.OutputSize.x * global.OriginalSize.y * global.lut_strength, 1.0);
            }
            """;

        [Fact]
        public void A_pass_compiles_and_its_block_members_offsets_and_textures_are_read_back()
        {
            string path = Path.Combine(_root, "pass.slang");
            File.WriteAllText(path, Pass);
            SlangSource source = SlangSource.Load(path);

            SpirvReflection vertex = SpirvReflection.Read(SlangCompiler.Compile(source.Vertex, SlangStage.Vertex, "pass.slang"));
            SpirvReflection fragment = SpirvReflection.Read(SlangCompiler.Compile(source.Fragment, SlangStage.Fragment, "pass.slang"));
            SpirvReflection pass = SpirvReflection.Merge(vertex, fragment);

            Assert.NotNull(pass.Uniforms);
            Assert.Equal(0u, pass.Uniforms!.Binding);
            Assert.Equal(new SlangMember("MVP", 0, 64), pass.Uniforms.Find("MVP"));
            Assert.Equal(new SlangMember("OutputSize", 64, 16), pass.Uniforms.Find("OutputSize"));
            Assert.Equal(new SlangMember("OriginalSize", 80, 16), pass.Uniforms.Find("OriginalSize"));
            Assert.Equal(new SlangMember("lut_strength", 96, 4), pass.Uniforms.Find("lut_strength"));
            Assert.Equal(100u, pass.Uniforms.Size);

            Assert.NotNull(pass.PushConstants);
            Assert.Equal(new SlangMember("SourceSize", 0, 16), pass.PushConstants!.Find("SourceSize"));
            Assert.Equal(new SlangMember("hardScan", 16, 4), pass.PushConstants.Find("hardScan"));
            Assert.Equal(new SlangMember("FrameCount", 20, 4), pass.PushConstants.Find("FrameCount"));

            Assert.Equal(new[] { new SlangSampler("Source", 2), new SlangSampler("MaskLut", 3) }, pass.Samplers.OrderBy(s => s.Binding));
            Assert.Empty(vertex.Samplers);
        }

        [Fact]
        public void A_pass_that_does_not_compile_says_where()
        {
            var error = Assert.Throws<SlangCompileException>(() => SlangCompiler.Compile("#version 450\nvoid main() { undefined_thing(); }\n", SlangStage.Fragment, "broken.slang"));
            Assert.Contains("broken.slang", error.Message);
            Assert.Contains("undefined_thing", error.Message);
        }

        // Every distinct pass in libretro's pack compiles in both stages and reads back, when a pack is given.
        [Fact]
        public void Every_pass_in_the_pack_compiles_and_reflects()
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;

            var passes = new HashSet<string>(StringComparer.Ordinal);
            foreach (string path in Directory.EnumerateFiles(pack, "*.slangp", SearchOption.AllDirectories))
            {
                try { foreach (SlangPassSpec pass in SlangPreset.Load(path).Passes) passes.Add(pass.ShaderPath); }
                catch (InvalidDataException) { }
            }

            var failures = new ConcurrentBag<string>();
            var clock = System.Diagnostics.Stopwatch.StartNew();
            Parallel.ForEach(passes, path =>
            {
                try
                {
                    SlangSource source = SlangSource.Load(path);
                    SpirvReflection.Merge(SpirvReflection.Read(SlangCompiler.Compile(source.Vertex, SlangStage.Vertex, path)),
                        SpirvReflection.Read(SlangCompiler.Compile(source.Fragment, SlangStage.Fragment, path)));
                }
                catch (Exception ex) { failures.Add($"{path}: {ex.Message.Split('\n').Take(3).Aggregate((a, b) => a + " | " + b)}"); }
            });
            _output.WriteLine($"{passes.Count} passes compiled in {clock.Elapsed.TotalSeconds:F1} s, {failures.Count} failures");
            foreach (string failure in failures.OrderBy(f => f).Take(40)) _output.WriteLine(failure);
            Assert.True(failures.IsEmpty, $"{failures.Count} of {passes.Count} passes did not compile; first: {failures.OrderBy(f => f).FirstOrDefault()}");
        }
    }
}
