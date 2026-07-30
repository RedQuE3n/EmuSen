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
            Assert.Contains("uniform shader image;", BuiltInShaders.ScanlinesSksl);
            Assert.Contains("uniform float2 outputSize;", BuiltInShaders.ScanlinesSksl);
            Assert.Contains("0.78", BuiltInShaders.ScanlinesSksl);
        }

        [Fact]
        public void Crt_declares_expected_uniforms_and_darken_factor_and_vignette()
        {
            Assert.Contains("uniform shader image;", BuiltInShaders.CrtSksl);
            Assert.Contains("uniform float2 outputSize;", BuiltInShaders.CrtSksl);
            Assert.Contains("0.72", BuiltInShaders.CrtSksl);
            Assert.Contains("vignette", BuiltInShaders.CrtSksl);
        }
    }
}
