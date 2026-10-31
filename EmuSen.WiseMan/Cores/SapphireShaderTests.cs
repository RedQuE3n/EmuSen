using System;
using System.IO;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Sapphire's GS walk's GLSL against the SPIR-V its crate embeds, compiled as Diamond's are - see Sapphire_Gs.md §4.
    public class SapphireShaderTests
    {
        [Fact]
        public void Every_compiled_shader_of_the_gs_is_the_compilation_of_its_source()
        {
            string[] sources = Directory.GetFiles(ShaderCompiler.SapphireShaderDirectory, "*.comp");
            Assert.Single(sources);

            foreach (string source in sources)
            {
                string name = Path.GetFileNameWithoutExtension(source);
                byte[] compiled = ShaderCompiler.Compile(File.ReadAllText(source), name);
                string binary = Path.ChangeExtension(source, ".spv");

                if (Environment.GetEnvironmentVariable(ShaderCompiler.RecordVariable) == "1") File.WriteAllBytes(binary, compiled);

                Assert.True(File.Exists(binary), $"{name}.spv is missing: run once with {ShaderCompiler.RecordVariable}=1");
                Assert.True(compiled.AsSpan().SequenceEqual(File.ReadAllBytes(binary)), $"{name}.spv is stale: run once with {ShaderCompiler.RecordVariable}=1, then rebuild the crate");
            }
        }
    }
}
