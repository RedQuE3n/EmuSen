using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.WiseMan.Serenity
{
    // No pass names anything by a word Metal's compiler keeps for itself, which Skia writes through unchanged - see EmuSen_Serenity.md §3.11.
    public class ShaderNamesTests
    {
        // C++14's keywords and alternative tokens that SkSL takes as names, and Metal's address spaces and function qualifiers.
        private static readonly HashSet<string> Kept = new(StringComparer.Ordinal)
        {
            "alignas", "alignof", "and", "and_eq", "auto", "bitand", "bitor", "catch", "char", "char16_t", "char32_t", "compl", "constexpr", "const_cast", "decltype", "delete",
            "dynamic_cast", "explicit", "export", "friend", "mutable", "new", "noexcept", "not", "not_eq", "nullptr", "operator", "or", "or_eq", "private", "protected", "register",
            "reinterpret_cast", "signed", "static_assert", "static_cast", "thread_local", "throw", "try", "typeid", "typename", "virtual", "wchar_t", "xor", "xor_eq",
            "device", "constant", "thread", "threadgroup", "threadgroup_imageblock", "ray_data", "object_data", "kernel", "vertex", "fragment",
        };

        private static readonly Regex Comment = new(@"//[^\n]*|/\*.*?\*/", RegexOptions.Singleline);
        private static readonly Regex Name = new(@"[A-Za-z_][A-Za-z0-9_]*");

        private static IEnumerable<string> KeptIn(string sksl) =>
            Name.Matches(Comment.Replace(sksl, " ")).Select(m => m.Value).Where(Kept.Contains).Distinct();

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

        // Every text the project hands SkSL: the simple effects, the reducing pass, and each filter's passes on each console with each structural setting at each of its values.
        private static IEnumerable<(string Where, string Sksl)> Sources()
        {
            yield return ("Scanlines", BuiltInShaders.ScanlinesSksl);
            yield return ("Simple CRT", BuiltInShaders.CrtSksl);
            yield return ("the reducing pass", FilterChain.ReduceSksl);
            foreach (ScreenFilter filter in ScreenFilters.All.Select(c => c.Filter).OfType<ScreenFilter>())
            {
                for (int i = 0; i < filter.Passes.Count; i++) yield return ($"{filter.Name} pass {i}", filter.Passes[i].Sksl);
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
                        for (int i = 0; i < passes.Count; i++) yield return ($"{filter.Name} on {console ?? "any console"}, {what}, pass {i} ({passes[i].Name ?? "the last"})", passes[i].Sksl);
                    }
                }
            }
        }

        [Fact]
        public void No_pass_uses_a_name_that_Metal_keeps()
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            var found = new List<string>();
            int texts = 0;
            foreach (var (where, sksl) in Sources())
            {
                if (!seen.Add(sksl)) continue;
                texts++;
                foreach (string word in KeptIn(sksl)) found.Add($"'{word}' in {where}");
            }
            Assert.True(texts > 40, $"only {texts} distinct texts were read");
            Assert.True(found.Count == 0, string.Join("\n", found.Take(20)));
        }

        // The reading itself: a kept word as a name is found, and one in a comment or inside a longer name is not.
        [Fact]
        public void A_kept_word_is_found_as_a_name_and_nowhere_else()
        {
            Assert.Equal(new[] { "device" }, KeptIn("half4 main(float2 coord) { float2 device = coord; return half4(device, 0.0, 1.0); }"));
            Assert.Equal(new[] { "kernel", "new" }, KeptIn("float blur(float kernel) { float new = kernel; return new; }").OrderBy(w => w, StringComparer.Ordinal));
            Assert.Empty(KeptIn("// the device's pixels\n/* a new kernel */ half4 main(float2 coord) { float2 devicePixel = coord; float renew = 1.0; return half4(devicePixel, renew, 1.0); }"));
        }
    }
}
