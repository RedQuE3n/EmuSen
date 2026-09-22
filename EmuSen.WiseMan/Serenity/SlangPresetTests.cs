using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Serenity.Slang;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // RetroArch's .slangp presets and .slang sources as RetroArch reads them - see EmuSen_Serenity.md §7.1 and §7.2.
    public class SlangPresetTests : IDisposable
    {
        // A directory holding libretro's slang-shaders, for the test that reads every preset in it.
        public const string PackVariable = "EMUSEN_SLANG_PACK";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSlangTests", Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;

        public SlangPresetTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Write(string relative, string text)
        {
            string path = Path.Combine(_root, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, text);
            return path;
        }

        [Fact]
        public void A_preset_s_passes_scales_textures_and_parameters_are_read_with_paths_from_its_own_folder()
        {
            string preset = Write("crt/crt-test.slangp", """
                # a comment
                shaders = "3"
                shader0 = shaders/first.slang
                alias0 = "LINEAR"
                filter_linear0 = "true"
                scale_type0 = source
                scale0 = 2.0
                srgb_framebuffer0 = "true"
                shader1 = "shaders/second.slang"
                scale_type_x1 = "viewport"
                scale_x1 = "1.0"
                scale_type_y1 = "absolute"
                scale_y1 = "240"
                wrap_mode1 = "repeat"
                float_framebuffer1 = true
                mipmap_input1 = "true"
                frame_count_mod1 = "2"
                shader2 = "shaders/third.slang"
                textures = "MASK;NOISE"
                MASK = "luts/mask.png"   # the mask
                MASK_linear = "true"
                MASK_wrap_mode = "repeat"
                MASK_mipmap = "true"
                NOISE = "luts/noise.png
                hardScan = "-6.5"
                """);

            SlangPreset p = SlangPreset.Load(preset);

            Assert.Equal(3, p.Passes.Count);
            Assert.Equal(Path.Combine(_root, "crt/shaders/first.slang"), p.Passes[0].ShaderPath);
            Assert.Equal("LINEAR", p.Passes[0].Alias);
            Assert.True(p.Passes[0].FilterLinear);
            Assert.Equal((SlangScaleType.Source, SlangScaleType.Source, 2f, 2f), (p.Passes[0].ScaleTypeX, p.Passes[0].ScaleTypeY, p.Passes[0].ScaleX, p.Passes[0].ScaleY));
            Assert.True(p.Passes[0].SrgbFramebuffer);
            Assert.Equal((SlangScaleType.Viewport, SlangScaleType.Absolute, 1f, 240f), (p.Passes[1].ScaleTypeX, p.Passes[1].ScaleTypeY, p.Passes[1].ScaleX, p.Passes[1].ScaleY));
            Assert.Equal(SlangWrap.Repeat, p.Passes[1].Wrap);
            Assert.True(p.Passes[1].FloatFramebuffer);
            Assert.True(p.Passes[1].MipmapInput);
            Assert.Equal(2, p.Passes[1].FrameCountMod);
            Assert.Equal(SlangScaleType.Viewport, p.Passes[2].ScaleTypeX);
            Assert.False(p.Passes[2].FilterLinear);
            Assert.Equal(SlangWrap.ClampToBorder, p.Passes[2].Wrap);

            Assert.Equal(new[] { "MASK", "NOISE" }, p.Textures.Select(t => t.Name));
            Assert.Equal(Path.Combine(_root, "crt/luts/mask.png"), p.Textures[0].Path);
            Assert.True(p.Textures[0].Linear && p.Textures[0].Mipmap);
            Assert.Equal(SlangWrap.Repeat, p.Textures[0].Wrap);
            Assert.False(p.Textures[1].Linear);

            Assert.Equal(-6.5f, p.Parameters["hardScan"]);
            Assert.False(p.Parameters.ContainsKey("MASK_linear"));
            Assert.False(p.Parameters.ContainsKey("scale0"));
        }

        // An unscaled pass is source 1.0 unless it is the last, which fills the viewport, as RetroArch has it.
        [Fact]
        public void An_unscaled_pass_follows_the_source_and_an_unscaled_last_pass_fills_the_viewport()
        {
            SlangPreset p = SlangPreset.Load(Write("p.slangp", "shaders = 2\nshader0 = a.slang\nshader1 = b.slang\n"));
            Assert.Equal((SlangScaleType.Source, 1f), (p.Passes[0].ScaleTypeX, p.Passes[0].ScaleX));
            Assert.Equal(SlangScaleType.Viewport, p.Passes[1].ScaleTypeY);
        }

        // A reference is read first and this file's keys go over it; the referenced file's paths stay relative to it.
        [Fact]
        public void A_reference_is_read_first_and_the_referring_file_s_keys_override_it()
        {
            Write("base/crt.slangp", "shaders = 1\nshader0 = shaders/crt.slang\nhardScan = -8.0\nmaskDark = 0.5\n");
            Write("mid/middle.slangp", "#reference \"../base/crt.slangp\"\nmaskDark = 0.3\n");
            string top = Write("top/mine.slangp", "#reference \"../mid/middle.slangp\"\nhardScan = -4.0\n");

            SlangPreset p = SlangPreset.Load(top);

            Assert.Equal(Path.Combine(_root, "base/shaders/crt.slang"), p.Passes[0].ShaderPath);
            Assert.Equal(-4f, p.Parameters["hardScan"]);
            Assert.Equal(0.3f, p.Parameters["maskDark"]);
        }

        [Fact]
        public void A_reference_loop_is_refused_rather_than_followed_for_ever()
        {
            Write("a.slangp", "#reference \"b.slangp\"\nshaders = 1\nshader0 = x.slang\n");
            string b = Write("b.slangp", "#reference \"a.slangp\"\n");
            Assert.Throws<InvalidDataException>(() => SlangPreset.Load(b));
        }

        [Fact]
        public void A_source_is_split_into_its_stages_with_includes_in_place_and_its_pragmas_read()
        {
            Write("shaders/common.inc", "layout(push_constant) uniform Push { float hardScan; } params;\n#pragma parameter hardScan \"Scanline hardness\" -8.0 -20.0 0.0 1.0\n");
            string shader = Write("shaders/pass.slang", """
                #version 450
                #include "common.inc"
                #pragma include_optional "absent.inc"
                #pragma name PASS0
                #pragma format R16G16B16A16_SFLOAT
                #pragma parameter maskDark "Mask dark" 0.5 0.0 2.0
                #pragma stage vertex
                void main() { gl_Position = vec4(0.0); }
                #pragma stage fragment
                layout(location = 0) out vec4 FragColor;
                void main() { FragColor = vec4(params.hardScan); }
                """);

            SlangSource s = SlangSource.Load(shader);

            Assert.StartsWith("#version 450", s.Vertex);
            Assert.StartsWith("#version 450", s.Fragment);
            Assert.Contains("uniform Push", s.Vertex);
            Assert.Contains("uniform Push", s.Fragment);
            Assert.Contains("gl_Position", s.Vertex);
            Assert.DoesNotContain("gl_Position", s.Fragment);
            Assert.Contains("FragColor", s.Fragment);
            Assert.DoesNotContain("FragColor", s.Vertex);
            Assert.DoesNotContain("#pragma", s.Vertex + s.Fragment);
            Assert.Equal(s.Vertex.Split('\n').Length, s.Fragment.Split('\n').Length);
            Assert.Equal("PASS0", s.PassName);
            Assert.Equal("R16G16B16A16_SFLOAT", s.FramebufferFormat);
            Assert.Equal(new[] { "hardScan", "maskDark" }, s.Parameters.Select(p => p.Id));
            Assert.Equal(new SlangParameter("hardScan", "Scanline hardness", -8f, -20f, 0f, 1f), s.Parameters[0]);
            Assert.Equal(0f, s.Parameters[1].Step);
        }

        // Every preset libretro ships reads, and every pass it names exists and splits, when a pack is given.
        [Fact]
        public void Every_preset_in_the_pack_reads_and_every_pass_it_names_splits()
        {
            string? pack = Environment.GetEnvironmentVariable(PackVariable);
            if (string.IsNullOrEmpty(pack)) return;

            var failures = new List<string>();
            var sources = new Dictionary<string, bool>(StringComparer.Ordinal);
            int presets = 0;
            int fragments = 0;
            var missing = new List<string>();
            foreach (string path in Directory.EnumerateFiles(pack, "*.slangp", SearchOption.AllDirectories))
            {
                // A file with no shaders of its own and no reference is a fragment for others to reference, not a preset.
                string text = File.ReadAllText(path);
                if (!text.Contains("shaders", StringComparison.Ordinal) && !text.Contains("#reference", StringComparison.Ordinal)) { fragments++; continue; }
                presets++;
                try
                {
                    SlangPreset preset = SlangPreset.Load(path);
                    foreach (SlangPassSpec pass in preset.Passes)
                    {
                        if (sources.ContainsKey(pass.ShaderPath)) continue;
                        try { SlangSource.Load(pass.ShaderPath); sources[pass.ShaderPath] = true; }
                        catch (Exception ex) { sources[pass.ShaderPath] = false; failures.Add($"{pass.ShaderPath}: {ex.Message}"); }
                    }
                    // A missing image is the pack's own defect (five koko-aio presets point a folder too shallow), counted rather than failed.
                    foreach (SlangTextureSpec texture in preset.Textures)
                        if (!File.Exists(texture.Path)) missing.Add($"{path}: texture {texture.Name} at {texture.Path} is missing");
                }
                catch (Exception ex) { failures.Add($"{path}: {ex.Message}"); }
            }
            _output.WriteLine($"{presets} presets, {fragments} fragments, {sources.Count} distinct passes, {failures.Count} failures, {missing.Count} missing images");
            foreach (string gone in missing.Take(10)) _output.WriteLine(gone);
            foreach (string failure in failures.Take(40)) _output.WriteLine(failure);
            Assert.True(failures.Count == 0, $"{failures.Count} of {presets} presets or their passes did not read; first: {failures.FirstOrDefault()}");
        }
    }
}
