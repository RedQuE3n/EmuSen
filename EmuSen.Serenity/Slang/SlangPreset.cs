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

        // The library's reader; half a surrogate pair in the path crosses as U+FFFD - see EmuSen_RustPlatform.md §16.2 and EmuSen_Backend.md §40.6.
        public static SlangPreset Load(string path) => LoadNative(path);

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
    }
}
