using EmuSen.Serenity.Shaders;

namespace EmuSen.WiseMan.Serenity
{
    // BuiltInShaders' SkSL source strings - internal, exposed to this
    // project via EmuSen.Serenity/AssemblyInfo.cs's InternalsVisibleTo.
    // Asserts the shaders declare the uniforms GameFrameControl actually
    // binds (see its DrawOp.Render) and keep the same faithfully-ported
    // darken-factor constants as the old Raylib/GLSL version.
    public class BuiltInShadersTests
    {
        [Fact]
        public void Scanlines_declares_expected_uniforms_and_darken_factor()
        {
            Assert.Contains("uniform sampler2D image;", BuiltInShaders.ScanlinesGlsl);
            Assert.Contains("uniform vec2 outputSize;", BuiltInShaders.ScanlinesGlsl);
            Assert.Contains("0.78", BuiltInShaders.ScanlinesGlsl);
        }

        [Fact]
        public void Crt_declares_expected_uniforms_and_darken_factor_and_vignette()
        {
            Assert.Contains("uniform sampler2D image;", BuiltInShaders.CrtGlsl);
            Assert.Contains("uniform vec2 outputSize;", BuiltInShaders.CrtGlsl);
            Assert.Contains("0.72", BuiltInShaders.CrtGlsl);
            Assert.Contains("vignette", BuiltInShaders.CrtGlsl);
        }
    }
}
