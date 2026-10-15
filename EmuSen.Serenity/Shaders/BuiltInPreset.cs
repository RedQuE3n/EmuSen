using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // A built-in filter written as a slang preset for the chain, its passes translated from their SkSL - see EmuSen_DevicePicture.md §3.
    public static class BuiltInPreset
    {
        // The parameters the runner sets at each draw, which no settings window lists - see EmuSen_DevicePicture.md §3.2.
        public const string PixelScale = "emusen_pixel_scale", OriginX = "emusen_pixel_origin_x", OriginY = "emusen_pixel_origin_y";
        public const string CropLeft = "emusen_crop_left", CropTop = "emusen_crop_top", CropWidth = "emusen_crop_width", CropHeight = "emusen_crop_height";
        public const string RawWidth = "emusen_raw_width", RawHeight = "emusen_raw_height";

        public static IReadOnlyList<string> Reserved { get; } = new[] { PixelScale, OriginX, OriginY, CropLeft, CropTop, CropWidth, CropHeight, RawWidth, RawHeight };

        private static readonly Dictionary<string, float> ReservedDefaults = new(StringComparer.Ordinal)
        {
            [PixelScale] = 1, [OriginX] = 0, [OriginY] = 0, [CropLeft] = 0, [CropTop] = 0, [CropWidth] = 1, [CropHeight] = 1, [RawWidth] = 0, [RawHeight] = 0,
        };

        // Where generated presets are kept, a folder per content; a test sets it before any is written.
        internal static string? RootOverride;

        public static string Root => RootOverride ?? Path.Combine(DataStore.Shaders, "builtin");

        private static readonly (Regex Pattern, string Glsl)[] Types =
        {
            (new Regex(@"\bfloat2x2\b"), "mat2"), (new Regex(@"\bfloat3x3\b"), "mat3"), (new Regex(@"\bfloat4x4\b"), "mat4"),
            (new Regex(@"\bhalf2x2\b"), "mat2"), (new Regex(@"\bhalf3x3\b"), "mat3"), (new Regex(@"\bhalf4x4\b"), "mat4"),
            (new Regex(@"\bfloat([234])\b"), "vec$1"), (new Regex(@"\bhalf([234])\b"), "vec$1"), (new Regex(@"\bhalf\b"), "float"),
            (new Regex(@"\bint([234])\b"), "ivec$1"), (new Regex(@"\bbool([234])\b"), "bvec$1"),
        };

        private static readonly Regex UniformLine = new(@"^[ \t]*uniform[ \t]+(\w+)[ \t]+(\w+)[ \t]*;[ \t]*\r?$", RegexOptions.Multiline);
        private static readonly Regex Main = new(@"\b(half4|float4)(\s+)main(\s*)\(");

        // SkSL's names for its types as GLSL's; the one rewrite of text the translation makes besides a child's eval.
        public static string GlslTypes(string sksl)
        {
            foreach (var (pattern, glsl) in Types) sksl = pattern.Replace(sksl, glsl);
            return sksl;
        }

        // How a pass reads one child: by fetch at its texel, or linearly through the bound sampler.
        private readonly record struct Child(string Name, string Sampler, bool Linear, bool Stretched = false);

        // One translated pass: its slang text and what the preset says of it.
        public sealed record Pass(string FileName, string Text, string? Alias, PassSize Width, PassSize Height, bool Float, bool LinearSampler);

        // The passes a filter draws on a console, reducing pass first; `nudge` shifts every sampling point for the tie test - see EmuSen_DevicePicture.md §12.
        public static IReadOnlyList<Pass> Translate(ScreenFilter filter, IReadOnlyDictionary<string, float> values, int reduceBy = 1, (float X, float Y) nudge = default)
        {
            IReadOnlyList<FilterPass> passes = filter.Build?.Invoke(values) ?? filter.Passes;
            var all = new List<FilterPass>();
            bool reduced = reduceBy > 1;
            if (reduced)
                all.Add(new FilterPass(FilterChain.ReduceSksl.Replace("uniform float by;", "const float by = " + reduceBy.ToString(CultureInfo.InvariantCulture) + ".0;"), PassScale.Source)
                { Name = "reduced", Width = new PassSize(PassAxis.Source, 1f / reduceBy), Height = new PassSize(PassAxis.Source, 1f / reduceBy) });
            all.AddRange(passes);

            // Which images some pass reads linearly: by libretro's rule an output is sampled with the filter of the pass after it, the frame with the first pass's.
            var linearOutputs = new HashSet<int>();
            bool linearOriginal = false;
            for (int j = 0; j < all.Count; j++)
            {
                foreach (string child in ChildrenOf(all[j].Sksl))
                {
                    if (!ReadsLinearly(all[j], child)) continue;
                    if (child == "source") { if (j == 0) linearOriginal = true; else linearOutputs.Add(j - 1); }
                    else if (child == "image") linearOriginal = true;
                    else if (child == "original" && !reduced) linearOriginal = true;
                    else if (IndexOfName(all, child == "original" ? "reduced" : child, j) is int k) linearOutputs.Add(k);
                }
            }

            var result = new List<Pass>();
            for (int j = 0; j < all.Count; j++)
            {
                FilterPass pass = all[j];
                bool linearSampler = j == 0 ? linearOriginal : linearOutputs.Contains(j - 1);
                PassSize width = pass.Width ?? new PassSize(pass.Scale == PassScale.Source ? PassAxis.Source : PassAxis.Viewport);
                PassSize height = pass.Height ?? new PassSize(pass.Scale == PassScale.Source ? PassAxis.Source : PassAxis.Viewport);
                string text = PassText(filter, all, j, values, reduced, reduced && j > 0, nudge);
                result.Add(new Pass($"pass{j}.slang", text, pass.Name is { } name ? Alias(name) : null, width, height, pass.Float, linearSampler));
            }
            return result;
        }

        private static string Alias(string name) => "emusen_" + name;

        private static int? IndexOfName(IReadOnlyList<FilterPass> passes, string name, int before)
        {
            for (int k = 0; k < before; k++) if (passes[k].Name == name) return k;
            return null;
        }

        private static IEnumerable<string> ChildrenOf(string sksl) =>
            UniformLine.Matches(sksl).Where(m => m.Groups[1].Value == "shader").Select(m => m.Groups[2].Value);

        private static bool ReadsLinearly(FilterPass pass, string child) =>
            child is "source" or "image" ? pass.LinearSource : pass.Linear is { } names && names.Contains(child);

        private static string F(float value) => value.ToString("R", CultureInfo.InvariantCulture);

        private static string PassText(ScreenFilter filter, IReadOnlyList<FilterPass> passes, int index, IReadOnlyDictionary<string, float> values, bool reduced, bool originalIsReduced, (float X, float Y) nudge)
        {
            FilterPass pass = passes[index];
            string body = pass.Sksl;
            var children = new List<Child>();
            var globals = new StringBuilder();
            var assignments = new StringBuilder();
            var vectors = new SortedSet<string>(StringComparer.Ordinal) { "MVP" };
            var floats = new List<string>();
            var pragmas = new StringBuilder();
            bool frameCount = false;

            void Float(string id, float initial)
            {
                if (floats.Contains(id)) return;
                floats.Add(id);
                pragmas.Append(CultureInfo.InvariantCulture, $"#pragma parameter {id} \"{id}\" {F(initial)} {F(initial - 1e6f)} {F(initial + 1e6f)} 0\n");
            }

            void Semantic(string type, string name, string value, params string[] needs)
            {
                globals.Append(CultureInfo.InvariantCulture, $"{type} {name};\n");
                assignments.Append(CultureInfo.InvariantCulture, $"    {name} = {value};\n");
                foreach (string need in needs) vectors.Add(need);
            }

            foreach (Match m in UniformLine.Matches(body))
            {
                string type = m.Groups[1].Value, name = m.Groups[2].Value;
                if (type == "shader")
                {
                    string sampler = name switch
                    {
                        "source" => "Source",
                        "original" => originalIsReduced ? Alias("reduced") : "Original",
                        "raw" => "raw",
                        "image" => "Original",
                        "feedback" => pass.Name is { } own ? Alias(own) + "Feedback" : $"PassFeedback{index}",
                        _ when name.StartsWith("history", StringComparison.Ordinal) && int.TryParse(name["history".Length..], out int back) && back >= 1 => $"OriginalHistory{back}",
                        _ when IndexOfName(passes, name, index) is not null => Alias(name),
                        _ => throw new InvalidOperationException($"The '{filter.Name}' filter asks for a child called '{name}', which no pass provides."),
                    };
                    bool linear = name == "image" ? pass.LinearSource : ReadsLinearly(pass, name) && name is not "raw" && !name.StartsWith("history", StringComparison.Ordinal);
                    children.Add(new Child(name, sampler, linear, name == "image"));
                    if (name == "image") vectors.UnionWith(new[] { "OriginalSize", "OutputSize" });
                    continue;
                }

                switch (name)
                {
                    case "inputSize": Semantic("vec2", name, "SourceSize.xy", "SourceSize"); break;
                    case "originalSize":
                        if (originalIsReduced) Semantic("vec2", name, Alias("reduced") + "Size.xy", Alias("reduced") + "Size");
                        else Semantic("vec2", name, "OriginalSize.xy", "OriginalSize");
                        break;
                    case "outputSize": Semantic("vec2", name, "OutputSize.xy", "OutputSize"); break;
                    case "viewportSize": Semantic("vec2", name, "FinalViewportSize.xy", "FinalViewportSize"); break;
                    case "frameCount": Semantic("float", name, "float(FrameCount)"); frameCount = true; break;
                    case "pixelScale": Semantic("float", name, PixelScale); Float(PixelScale, 1); break;
                    case "pixelOrigin": Semantic("vec2", name, $"vec2({OriginX}, {OriginY})"); Float(OriginX, 0); Float(OriginY, 0); break;
                    case "crop": Semantic("vec4", name, $"vec4({CropLeft}, {CropTop}, {CropWidth}, {CropHeight})"); Float(CropLeft, 0); Float(CropTop, 0); Float(CropWidth, 1); Float(CropHeight, 1); break;
                    case "rawSize": Semantic("vec2", name, $"vec2({RawWidth}, {RawHeight})"); Float(RawWidth, 0); Float(RawHeight, 0); break;
                    case var size when size.EndsWith("Size", StringComparison.Ordinal) && IndexOfName(passes, size[..^4], index) is not null:
                        Semantic("vec2", size, Alias(size[..^4]) + "Size.xy", Alias(size[..^4]) + "Size");
                        break;
                    case var id when values.TryGetValue(id, out float value): Float(id, value); break;
                    default: throw new InvalidOperationException($"The '{filter.Name}' filter asks for a uniform called '{name}', which no pass provides.");
                }
            }

            body = UniformLine.Replace(body, "");
            foreach (Child child in children) body = Regex.Replace(body, $@"\b{child.Name}\.eval\(", child.Name + "_eval(");
            body = Main.Replace(body, "$1$2sksl_main$3(", 1);
            body = GlslTypes(body);

            var block = new StringBuilder();
            block.Append("layout(std140, set = 0, binding = 0) uniform UBO\n{\n    mat4 MVP;\n");
            foreach (string vector in vectors.Where(v => v != "MVP")) block.Append(CultureInfo.InvariantCulture, $"    vec4 {vector};\n");
            if (frameCount) block.Append("    uint FrameCount;\n");
            foreach (string id in floats) block.Append(CultureInfo.InvariantCulture, $"    float {id};\n");
            block.Append("};\n");

            var samplers = new StringBuilder();
            int binding = 1;
            foreach (Child child in children)
            {
                samplers.Append(CultureInfo.InvariantCulture, $"layout(set = 0, binding = {binding++}) uniform sampler2D {child.Sampler};\n");
                // The simple effects' image is the frame stretched over the rectangle, as GameFrameControl's local matrix makes it - see EmuSen_Serenity.md §3.
                if (child.Stretched)
                {
                    samplers.Append(child.Linear
                        ? $"vec4 {child.Name}_eval(vec2 p) {{ return texture({child.Sampler}, p / OutputSize.xy); }}\n"
                        : $"vec4 {child.Name}_eval(vec2 p) {{ ivec2 s = textureSize({child.Sampler}, 0); return texelFetch({child.Sampler}, clamp(ivec2(floor(p * OriginalSize.xy / OutputSize.xy)), ivec2(0), s - 1), 0); }}\n");
                    continue;
                }
                samplers.Append(child.Linear
                    ? $"vec4 {child.Name}_eval(vec2 p) {{ return texture({child.Sampler}, p / vec2(textureSize({child.Sampler}, 0))); }}\n"
                    : $"vec4 {child.Name}_eval(vec2 p) {{ ivec2 s = textureSize({child.Sampler}, 0); return texelFetch({child.Sampler}, clamp(ivec2(floor(p)), ivec2(0), s - 1), 0); }}\n");
            }

            string write = pass.Float ? "FragColor = c;" : "FragColor = vec4(c.rgb, 1.0);";
            return "#version 450\n" + pragmas + (pass.Name is { } named ? $"#pragma name {Alias(named)}\n" : "") + block + @"
#pragma stage vertex
layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 vTexCoord;
void main()
{
    gl_Position = MVP * Position;
    vTexCoord = TexCoord;
}

#pragma stage fragment
layout(location = 0) in vec2 vTexCoord;
layout(location = 0) out vec4 FragColor;
" + samplers + globals + body + @"
void main()
{
" + assignments + "    vec4 c = sksl_main(gl_FragCoord.xy" + (nudge == default ? "" : $" + vec2({F(nudge.X)}, {F(nudge.Y)})") + ");\n    " + write + "\n}\n";
        }

        // The preset's text over its passes, and the values its parameters start at.
        public static string PresetText(IReadOnlyList<Pass> passes, IReadOnlyDictionary<string, float> values)
        {
            var text = new StringBuilder();
            text.Append(CultureInfo.InvariantCulture, $"shaders = {passes.Count}\n");
            for (int i = 0; i < passes.Count; i++)
            {
                Pass pass = passes[i];
                text.Append(CultureInfo.InvariantCulture, $"shader{i} = {pass.FileName}\n");
                if (pass.Alias is { } alias) text.Append(CultureInfo.InvariantCulture, $"alias{i} = {alias}\n");
                text.Append(CultureInfo.InvariantCulture, $"filter_linear{i} = {(pass.LinearSampler ? "true" : "false")}\n");
                text.Append(CultureInfo.InvariantCulture, $"wrap_mode{i} = clamp_to_edge\n");
                AppendAxis(text, "x", i, pass.Width);
                AppendAxis(text, "y", i, pass.Height);
                text.Append(CultureInfo.InvariantCulture, $"float_framebuffer{i} = {(pass.Float ? "true" : "false")}\n");
                text.Append(CultureInfo.InvariantCulture, $"frame_count_mod{i} = 65536\n");
            }
            // The frame before the console's palette is a lookup image the runner replaces with each frame, black until it does - see EmuSen_DevicePicture.md §3.4.
            if (passes.Any(pass => pass.Text.Contains("uniform sampler2D raw;", StringComparison.Ordinal)))
                text.Append(CultureInfo.InvariantCulture, $"textures = {RawTexture}\n{RawTexture} = {RawTexture}.png\n{RawTexture}_linear = false\n{RawTexture}_wrap_mode = clamp_to_edge\n");
            foreach (var (id, value) in values.OrderBy(v => v.Key, StringComparer.Ordinal)) text.Append(CultureInfo.InvariantCulture, $"{id} = {F(value)}\n");
            return text.ToString();
        }

        private static void AppendAxis(StringBuilder text, string axis, int index, PassSize size)
        {
            string type = size.Axis switch { PassAxis.Source => "source", PassAxis.Viewport => "viewport", _ => "absolute" };
            text.Append(CultureInfo.InvariantCulture, $"scale_type_{axis}{index} = {type}\n");
            text.Append(CultureInfo.InvariantCulture, $"scale_{axis}{index} = {F(size.Factor)}\n");
        }

        // The values a filter's preset is written with on a console: each parameter's default there, then the console's constants.
        public static Dictionary<string, float> ValuesFor(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen = null)
        {
            var values = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (SlangParameter parameter in filter.Parameters ?? Array.Empty<SlangParameter>())
                values[parameter.Id] = chosen is not null && chosen.TryGetValue(parameter.Id, out float given) ? given : filter.DefaultFor(parameter, console);
            if (filter.DefaultsFor(console) is { } constants) foreach (var (id, value) in constants) values.TryAdd(id, value);
            return values;
        }

        // The lookup image a pass reads the raw frame from.
        public const string RawTexture = "raw";

        // A black pixel, the raw frame's stand-in until the runner gives the chain one.
        private static byte[] BlackPixel()
        {
            using var bitmap = new SkiaSharp.SKBitmap(new SkiaSharp.SKImageInfo(1, 1, SkiaSharp.SKColorType.Rgba8888, SkiaSharp.SKAlphaType.Unpremul));
            bitmap.SetPixel(0, 0, new SkiaSharp.SKColor(0, 0, 0, 255));
            using SkiaSharp.SKData data = bitmap.Encode(SkiaSharp.SKEncodedImageFormat.Png, 100);
            return data.ToArray();
        }

        // Writes the preset into a folder named by its content and returns the preset's path; the same content is written once.
        public static string Write(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen = null, int reduceBy = 1, (float X, float Y) nudge = default)
        {
            Dictionary<string, float> values = ValuesFor(filter, console, chosen);
            IReadOnlyList<Pass> passes = Translate(filter, values, reduceBy, nudge);
            var used = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (Pass pass in passes)
                foreach (Match m in Regex.Matches(pass.Text, @"^#pragma parameter (\w+) ", RegexOptions.Multiline))
                {
                    string id = m.Groups[1].Value;
                    used[id] = values.TryGetValue(id, out float v) ? v : ReservedDefaults.GetValueOrDefault(id);
                }
            string preset = PresetText(passes, used);
            var all = new StringBuilder(preset);
            foreach (Pass pass in passes) all.Append('\0').Append(pass.FileName).Append('\0').Append(pass.Text);
            string folder = Path.Combine(Root, Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(all.ToString())))[..24].ToLowerInvariant());
            string path = Path.Combine(folder, "filter.slangp");
            if (File.Exists(path)) return path;
            Directory.CreateDirectory(folder);
            foreach (Pass pass in passes) File.WriteAllText(Path.Combine(folder, pass.FileName), pass.Text);
            if (preset.Contains($"textures = {RawTexture}", StringComparison.Ordinal)) File.WriteAllBytes(Path.Combine(folder, RawTexture + ".png"), BlackPixel());
            string temporary = path + "." + Environment.ProcessId.ToString(CultureInfo.InvariantCulture) + ".tmp";
            File.WriteAllText(temporary, preset);
            File.Move(temporary, path, overwrite: true);
            return path;
        }
    }
}
