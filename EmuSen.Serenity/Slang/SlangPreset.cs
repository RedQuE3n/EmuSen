using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    public enum SlangScaleType { Source, Viewport, Absolute }

    public enum SlangWrap { ClampToBorder, ClampToEdge, Repeat, MirroredRepeat }

    // One pass of a preset, as its keys say; an unscaled pass is source 1.0, or the viewport when it is the last - see EmuSen_Serenity.md §7.1.
    public sealed record SlangPassSpec(string ShaderPath, string? Alias, bool FilterLinear, SlangWrap Wrap,
        SlangScaleType ScaleTypeX, SlangScaleType ScaleTypeY, float ScaleX, float ScaleY,
        bool FloatFramebuffer, bool SrgbFramebuffer, bool MipmapInput, int FrameCountMod);

    public sealed record SlangTextureSpec(string Name, string Path, bool Linear, SlangWrap Wrap, bool Mipmap);

    // A .slangp preset read through its #reference chain, each path resolved against the file that wrote it - see EmuSen_Serenity.md §7.1.
    public sealed class SlangPreset
    {
        // How deep a chain of references may go before it is taken to be a loop.
        public const int MaxReferenceDepth = 16;

        public string Path { get; }
        public IReadOnlyList<SlangPassSpec> Passes { get; }
        public IReadOnlyList<SlangTextureSpec> Textures { get; }

        // Every key that is not a pass's, a texture's or the preset's own, which RetroArch reads as a parameter's value.
        public IReadOnlyDictionary<string, float> Parameters { get; }

        private SlangPreset(string path, IReadOnlyList<SlangPassSpec> passes, IReadOnlyList<SlangTextureSpec> textures, IReadOnlyDictionary<string, float> parameters)
        {
            Path = path;
            Passes = passes;
            Textures = textures;
            Parameters = parameters;
        }

        // The library's reader or the C#'s; a path that is no text of its own is the C#'s to refuse - see EmuSen_RustPlatform.md §16.2.
        public static SlangPreset Load(string path) => SerenityNative.Active && SerenityNative.Crosses(path) ? LoadNative(path) : Managed.Load(path);

        // The library's reading, as the document it answers with.
        internal static unsafe SlangPreset LoadNative(string path)
        {
            byte[] name = SerenityNative.Utf8(path)!;
            using JsonDocument document = SerenityNative.Document((buffer, capacity) =>
            {
                fixed (byte* n = name) return SerenityNative.PresetLoad(SerenityNative.Pin(n), (nuint)name.Length, buffer, capacity);
            });
            JsonElement root = document.RootElement;
            if (SerenityNative.Failure(root) is { } failure) throw failure;

            var passes = new List<SlangPassSpec>();
            foreach (JsonElement pass in root.GetProperty("passes").EnumerateArray())
                passes.Add(new SlangPassSpec(pass.GetProperty("shader").GetString()!, pass.GetProperty("alias").GetString(), pass.GetProperty("linear").GetBoolean(),
                    (SlangWrap)pass.GetProperty("wrap").GetInt32(), (SlangScaleType)pass.GetProperty("type_x").GetInt32(), (SlangScaleType)pass.GetProperty("type_y").GetInt32(),
                    SerenityNative.Float(pass.GetProperty("scale_x")), SerenityNative.Float(pass.GetProperty("scale_y")),
                    pass.GetProperty("float").GetBoolean(), pass.GetProperty("srgb").GetBoolean(), pass.GetProperty("mipmap").GetBoolean(), pass.GetProperty("mod").GetInt32()));

            var textures = new List<SlangTextureSpec>();
            foreach (JsonElement texture in root.GetProperty("textures").EnumerateArray())
                textures.Add(new SlangTextureSpec(texture.GetProperty("name").GetString()!, texture.GetProperty("path").GetString()!, texture.GetProperty("linear").GetBoolean(),
                    (SlangWrap)texture.GetProperty("wrap").GetInt32(), texture.GetProperty("mipmap").GetBoolean()));

            var parameters = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (JsonElement pair in root.GetProperty("parameters").EnumerateArray()) parameters[pair[0].GetString()!] = SerenityNative.Float(pair[1]);

            return new SlangPreset(root.GetProperty("path").GetString()!, passes, textures, parameters);
        }

        // The C# reader: the default, and what the library's is held to - see EmuSen_RustPlatform.md §16.
        internal static class Managed
        {
            private static readonly Regex Line = new(@"^\s*([A-Za-z0-9_\-\.]+)\s*=\s*(.*)$", RegexOptions.Compiled);
            private static readonly Regex Reference = new(@"^\s*#reference\s+""?([^""]+)""?", RegexOptions.Compiled);
            private static readonly Regex PassKey = new(@"^(shader|alias|filter_linear|wrap_mode|scale_type|scale_type_x|scale_type_y|scale|scale_x|scale_y|float_framebuffer|srgb_framebuffer|mipmap_input|frame_count_mod|rgb10_framebuffer|feedback_pass)\d+$", RegexOptions.Compiled);

            public static SlangPreset Load(string path)
            {
                var values = new Dictionary<string, (string Value, string Directory)>(StringComparer.Ordinal);
                Read(System.IO.Path.GetFullPath(path), values, 0);

                string? Get(string key) => values.TryGetValue(key, out var v) ? v.Value : null;
                string Resolve(string key) => System.IO.Path.GetFullPath(System.IO.Path.Combine(values[key].Directory, values[key].Value));

                if (!int.TryParse(Get("shaders"), NumberStyles.Integer, CultureInfo.InvariantCulture, out int count) || count < 1)
                    throw new InvalidDataException($"{path} says no number of shaders.");

                var passes = new List<SlangPassSpec>();
                for (int i = 0; i < count; i++)
                {
                    if (Get($"shader{i}") is null) throw new InvalidDataException($"{path} has no shader{i}.");
                    string? both = Get($"scale_type{i}"), x = Get($"scale_type_x{i}") ?? both, y = Get($"scale_type_y{i}") ?? both;
                    bool last = i == count - 1;
                    SlangScaleType Type(string? name) => name is null ? (last ? SlangScaleType.Viewport : SlangScaleType.Source) : ParseScaleType(name, path);
                    float Scale(string axis) => Float(Get($"scale_{axis}{i}") ?? Get($"scale{i}"), 1f);
                    passes.Add(new SlangPassSpec(Resolve($"shader{i}"), Get($"alias{i}"), Bool(Get($"filter_linear{i}")), ParseWrap(Get($"wrap_mode{i}")),
                        Type(x), Type(y), Scale("x"), Scale("y"),
                        Bool(Get($"float_framebuffer{i}")), Bool(Get($"srgb_framebuffer{i}")), Bool(Get($"mipmap_input{i}")),
                        (int)Float(Get($"frame_count_mod{i}"), 0)));
                }

                var textures = new List<SlangTextureSpec>();
                var textureNames = (Get("textures") ?? "").Split(';', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
                foreach (string name in textureNames)
                {
                    if (Get(name) is null) throw new InvalidDataException($"{path} names a texture {name} with no file.");
                    textures.Add(new SlangTextureSpec(name, Resolve(name), Bool(Get($"{name}_linear")), ParseWrap(Get($"{name}_wrap_mode")), Bool(Get($"{name}_mipmap"))));
                }

                var reserved = new HashSet<string>(StringComparer.Ordinal) { "shaders", "textures", "parameters" };
                foreach (string name in textureNames) { reserved.Add(name); reserved.Add(name + "_linear"); reserved.Add(name + "_wrap_mode"); reserved.Add(name + "_mipmap"); }
                var parameters = new Dictionary<string, float>(StringComparer.Ordinal);
                foreach (var (key, (value, _)) in values)
                {
                    if (reserved.Contains(key) || PassKey.IsMatch(key)) continue;
                    if (float.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out float number)) parameters[key] = number;
                }

                return new SlangPreset(System.IO.Path.GetFullPath(path), passes, textures, parameters);
            }

            // The referenced files first, then this one's own keys over them, as RetroArch applies a #reference.
            private static void Read(string path, Dictionary<string, (string, string)> values, int depth)
            {
                if (depth > MaxReferenceDepth) throw new InvalidDataException($"{path}: references go deeper than {MaxReferenceDepth}; taken to be a loop.");
                if (!File.Exists(path)) throw new FileNotFoundException($"The preset {path} does not exist.", path);
                string directory = System.IO.Path.GetDirectoryName(path)!;
                var own = new List<(string Key, string Value)>();
                foreach (string raw in File.ReadLines(path))
                {
                    if (Reference.Match(raw) is { Success: true } reference)
                    {
                        Read(System.IO.Path.GetFullPath(System.IO.Path.Combine(directory, reference.Groups[1].Value.Trim())), values, depth + 1);
                        continue;
                    }
                    string line = raw.TrimStart();
                    if (line.StartsWith('#') || line.Length == 0) continue;
                    if (Line.Match(raw) is { Success: true } pair) own.Add((pair.Groups[1].Value, Value(pair.Groups[2].Value)));
                }
                foreach (var (key, value) in own) values[key] = (value, directory);
            }

            // A quoted value runs to its closing quote, or, as RetroArch tolerates, to a comment or the line's end when it has none.
            private static string Value(string text)
            {
                text = text.Trim();
                if (text.StartsWith('"'))
                {
                    int close = text.IndexOf('"', 1);
                    if (close > 0) return text[1..close];
                    int comment = text.IndexOf('#', 1);
                    return (comment > 0 ? text[1..comment] : text[1..]).Trim();
                }
                int end = text.IndexOfAny(new[] { '#', ' ', '\t' });
                return end >= 0 ? text[..end] : text;
            }

            private static bool Bool(string? value) => value is not null && (value.Equals("true", StringComparison.OrdinalIgnoreCase) || value == "1");

            private static float Float(string? value, float fallback) =>
                value is not null && float.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out float number) ? number : fallback;

            private static SlangScaleType ParseScaleType(string name, string path) => name.ToLowerInvariant() switch
            {
                "source" => SlangScaleType.Source,
                "viewport" => SlangScaleType.Viewport,
                "absolute" => SlangScaleType.Absolute,
                _ => throw new InvalidDataException($"{path}: unknown scale type {name}."),
            };

            private static SlangWrap ParseWrap(string? name) => name?.ToLowerInvariant() switch
            {
                "clamp_to_edge" => SlangWrap.ClampToEdge,
                "repeat" => SlangWrap.Repeat,
                "mirrored_repeat" => SlangWrap.MirroredRepeat,
                _ => SlangWrap.ClampToBorder,
            };
        }
    }
}
