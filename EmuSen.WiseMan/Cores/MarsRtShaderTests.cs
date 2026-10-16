using System;
using System.IO;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's GLSL against the SPIR-V its crate embeds; the crate's own test holds the embedded bytes to these files - see Mars_Gpu.md §2.
    public class MarsRtShaderTests
    {
        [Fact]
        public void Every_compiled_shader_is_the_compilation_of_its_source()
        {
            string[] sources = Directory.GetFiles(ShaderCompiler.MarsRtShaderDirectory, "*.comp");
            Assert.Equal(5, sources.Length);

            foreach (string source in sources)
            {
                string name = Path.GetFileNameWithoutExtension(source);
                byte[] compiled = ShaderCompiler.Compile(File.ReadAllText(source), name);
                string binary = Path.ChangeExtension(source, ".spv");

                if (Environment.GetEnvironmentVariable(ShaderCompiler.RecordVariable) == "1") File.WriteAllBytes(binary, compiled);

                Assert.True(File.Exists(binary), $"{name}.spv is missing: run once with {ShaderCompiler.RecordVariable}=1");
                Assert.True(compiled.AsSpan().SequenceEqual(File.ReadAllBytes(binary)), $"{name}.spv is stale: run once with {ShaderCompiler.RecordVariable}=1, then rebuild MarsRT");
            }
        }
    }
}
