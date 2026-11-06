using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    // One #pragma parameter: its id, what it is called, its default and its range - see EmuSen_Serenity.md §7.2.
    public sealed record SlangParameter(string Id, string Description, float Initial, float Minimum, float Maximum, float Step)
    {
        // Names for the values Minimum, Minimum + Step and so on, when the parameter is one of a few choices - see EmuSen_Serenity.md §3.9.
        public IReadOnlyList<string>? Choices { get; init; }
    }

    // A .slang file with its includes in place, split into its two stages, and what its pragmas declared - see EmuSen_Serenity.md §7.2.
    public sealed class SlangSource
    {
        public const int MaxIncludeDepth = 32;

        public string Path { get; }
        public string Vertex { get; }
        public string Fragment { get; }
        public string? PassName { get; }
        public string? FramebufferFormat { get; }
        public IReadOnlyList<SlangParameter> Parameters { get; }

        private SlangSource(string path, string vertex, string fragment, string? name, string? format, IReadOnlyList<SlangParameter> parameters)
        {
            Path = path;
            Vertex = vertex;
            Fragment = fragment;
            PassName = name;
            FramebufferFormat = format;
            Parameters = parameters;
        }

        // The library's reader; half a surrogate pair in the path crosses as U+FFFD - see EmuSen_RustPlatform.md §16.2 and EmuSen_Backend.md §40.6.
        public static SlangSource Load(string path) => LoadNative(path);

        // The library's reading, as the document it answers with.
        internal static unsafe SlangSource LoadNative(string path)
        {
            byte[] name = SerenityNative.Utf8(path)!;
            using JsonDocument document = SerenityNative.Document((buffer, capacity) =>
            {
                fixed (byte* n = name) return SerenityNative.SourceLoad(SerenityNative.Pin(n), (nuint)name.Length, buffer, capacity);
            });
            JsonElement root = document.RootElement;
            if (SerenityNative.Failure(root) is { } failure) throw failure;
            return new SlangSource(root.GetProperty("path").GetString()!, root.GetProperty("vertex").GetString()!, root.GetProperty("fragment").GetString()!,
                root.GetProperty("name").GetString(), root.GetProperty("format").GetString(), SlangParameters.FromJson(root.GetProperty("parameters")));
        }
    }
}
