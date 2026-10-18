using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.WiseMan.Serenity
{
    // No pass names anything by a word GLSL keeps, or by a name the chain's preset writes into every pass, which a pass's own would collide with or shadow - see EmuSen_DevicePicture.md §21.
    public class ShaderNamesTests
    {
        // GLSL 4.50's reserved words and the qualifiers spelled as plain words, each refused by glslang as a name.
        private static readonly HashSet<string> Kept = new(StringComparer.Ordinal)
        {
            "common", "partition", "active", "asm", "class", "union", "enum", "typedef", "template", "this", "resource", "goto", "inline", "noinline", "public", "static",
            "extern", "external", "interface", "long", "short", "half", "fixed", "unsigned", "superp", "input", "output", "hvec2", "hvec3", "hvec4", "fvec2", "fvec3", "fvec4",
            "sampler3DRect", "filter", "sizeof", "cast", "namespace", "using", "sample", "patch", "buffer", "shared", "coherent", "volatile", "restrict", "readonly",
            "writeonly", "precise", "invariant", "centroid", "flat", "smooth", "noperspective", "subroutine", "attribute", "varying",
        };

        // What BuiltInPreset writes into every pass: the block, the vertex stage's names, the colour written, and the semantics' own names.
        private static readonly HashSet<string> Written = new(StringComparer.Ordinal)
        {
            "UBO", "MVP", "Position", "TexCoord", "vTexCoord", "FragColor", "Source", "Original", "SourceSize", "OriginalSize", "OutputSize", "FinalViewportSize", "FrameCount",
        };

        // Prefixes GLSL keeps (gl_), and those the preset gives its own samplers and values.
        private static readonly string[] KeptPrefixes = { "gl_", "emusen_", "OriginalHistory", "PassFeedback" };

        private static bool IsKept(string name) =>
            Kept.Contains(name) || Written.Contains(name) || name.Contains("__", StringComparison.Ordinal) || KeptPrefixes.Any(p => name.StartsWith(p, StringComparison.Ordinal));

        private static readonly Regex Comment = new(@"//[^\n]*|/\*.*?\*/", RegexOptions.Singleline);
        private static readonly Regex Name = new(@"[A-Za-z_][A-Za-z0-9_]*");

        private static IEnumerable<string> KeptIn(string glsl) =>
            Name.Matches(Comment.Replace(glsl, " ")).Select(m => m.Value).Where(IsKept).Distinct();

        // A filter's values on a console, as the chain resolves them, with one parameter moved.
        private static Dictionary<string, float> Values(ScreenFilter filter, string? console, string? moved = null, float to = 0)
        {
            var values = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (SlangParameter parameter in filter.Parameters ?? Array.Empty<SlangParameter>()) values[parameter.Id] = filter.DefaultFor(parameter, console);
            if (filter.DefaultsFor(console) is { } constants) foreach (var (id, value) in constants) values.TryAdd(id, value);
            if (moved is not null) values[moved] = to;
            return values;
        }

        // Every value of a parameter that names a few choices, and the ends and the start of one that is a range.
        private static IEnumerable<float> Steps(SlangParameter parameter) =>
            parameter.Choices is { } choices ? Enumerable.Range(0, choices.Count).Select(i => parameter.Minimum + i * parameter.Step) : new[] { parameter.Minimum, parameter.Initial, parameter.Maximum };

        // Every pass text the project writes: the simple effects, the reducing pass, and each filter's passes on each console with each structural setting at each of its values.
        private static IEnumerable<(string Where, string Glsl)> Sources()
        {
            yield return ("Scanlines", BuiltInShaders.ScanlinesGlsl);
            yield return ("Simple CRT", BuiltInShaders.CrtGlsl);
            yield return ("the reducing pass", BuiltInPreset.ReduceGlsl);
            foreach (ScreenFilter filter in ScreenFilters.All.Select(c => c.Filter).OfType<ScreenFilter>())
            {
                for (int i = 0; i < filter.Passes.Count; i++) yield return ($"{filter.Name} pass {i}", filter.Passes[i].Glsl);
                if (filter.Build is not { } build) continue;
                foreach (string? console in (filter.Consoles ?? Array.Empty<string>()).Cast<string?>().Append(null))
                {
                    var sets = new List<(string What, Dictionary<string, float> Values)> { ("defaults", Values(filter, console)) };
                    foreach (SlangParameter parameter in (filter.Parameters ?? Array.Empty<SlangParameter>()).Where(p => filter.Structural is { } ids && ids.Contains(p.Id)))
                        foreach (float value in Steps(parameter)) sets.Add(($"{parameter.Id}={value}", Values(filter, console, parameter.Id, value)));

                    // The tier and the signal each decide which passes exist, so every pair of them is built with every mask.
                    foreach (var set in sets.ToList())
                        if (set.Values.ContainsKey("quality") && set.Values.ContainsKey("signal"))
                            for (int quality = 0; quality < 3; quality++)
                                for (int signal = 0; signal < 3; signal++)
                                    sets.Add(($"{set.What} quality={quality} signal={signal}", new Dictionary<string, float>(set.Values) { ["quality"] = quality, ["signal"] = signal }));
                    foreach (var (what, values) in sets)
                    {
                        IReadOnlyList<FilterPass> passes = build(values);
                        for (int i = 0; i < passes.Count; i++) yield return ($"{filter.Name} on {console ?? "any console"}, {what}, pass {i} ({passes[i].Name ?? "the last"})", passes[i].Glsl);
                    }
                }
            }
        }

        [Fact]
        public void No_pass_uses_a_name_that_glsl_or_the_preset_keeps()
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            var found = new List<string>();
            int texts = 0;
            foreach (var (where, glsl) in Sources())
            {
                if (!seen.Add(glsl)) continue;
                texts++;
                foreach (string word in KeptIn(glsl)) found.Add($"'{word}' in {where}");
            }
            Assert.True(texts > 40, $"only {texts} distinct texts were read");
            Assert.True(found.Count == 0, string.Join("\n", found.Take(20)));
        }

        // The reading itself: a kept word, a written name or a kept prefix as a name is found, and one in a comment or inside a longer name is not.
        [Fact]
        public void A_kept_word_is_found_as_a_name_and_nowhere_else()
        {
            Assert.Equal(new[] { "sample" }, KeptIn("vec4 filter_main(vec2 coord) { vec2 sample = coord; return vec4(sample, 0.0, 1.0); }"));
            Assert.Equal(new[] { "common", "filter" }, KeptIn("float blur(float filter) { float common = filter; return common; }").OrderBy(w => w, StringComparer.Ordinal));
            Assert.Equal(new[] { "OutputSize", "gl_mine" }, KeptIn("vec4 filter_main(vec2 coord) { vec2 OutputSize = coord; float gl_mine = 1.0; return vec4(OutputSize, gl_mine, 1.0); }").OrderBy(w => w, StringComparer.Ordinal));
            Assert.Empty(KeptIn("// the sample's filter\n/* a common output */ vec4 filter_main(vec2 coord) { vec2 sampled = coord; float outputSizes = 1.0; return vec4(sampled, outputSizes, 1.0); }"));
        }
    }
}
