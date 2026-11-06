using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia.Native;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Native
{
    // The built-in filters' catalog, values, passes and presets, which the platform library alone makes - see EmuSen_Serenity.md §3.13.
    internal static unsafe class FilterLibrary
    {
        private static void Up() => SerenityNative.Up();

        // The catalog's document, read once.
        internal static JsonElement Catalog => LazyCatalog.Value;

        private static readonly Lazy<JsonElement> LazyCatalog = new(() =>
        {
            Up();
            using JsonDocument document = SerenityNative.Document((buffer, capacity) => SerenityNative.Filters(buffer, capacity));
            return document.RootElement.Clone();
        });

        private static float Float(JsonElement bits) => SerenityNative.Float(bits);

        internal static Dictionary<string, float> Values(JsonElement values)
        {
            var read = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (JsonProperty p in values.EnumerateObject()) read[p.Name] = Float(p.Value);
            return read;
        }

        private static PassSize? Size(JsonElement size) =>
            size.ValueKind == JsonValueKind.Null ? null : new PassSize((PassAxis)size.GetProperty("axis").GetInt32(), Float(size.GetProperty("factor")));

        internal static FilterPass Pass(JsonElement p) =>
            new(p.GetProperty("glsl").GetString()!, p.GetProperty("scale").GetInt32() == 0 ? PassScale.Source : PassScale.Viewport, p.GetProperty("linear_source").GetBoolean(), p.GetProperty("history").GetInt32())
            {
                Width = Size(p.GetProperty("width")),
                Height = Size(p.GetProperty("height")),
                Float = p.GetProperty("float").GetBoolean(),
                Feedback = p.GetProperty("feedback").GetBoolean(),
                Name = p.GetProperty("name").GetString(),
                Linear = p.GetProperty("linear").EnumerateArray().Select(n => n.GetString()!).ToArray(),
            };

        private static IReadOnlyList<string>? Strings(JsonElement e) =>
            e.ValueKind == JsonValueKind.Null ? null : e.EnumerateArray().Select(s => s.GetString()!).ToArray();

        // One of the catalog's filters, its passes made by the library from the values where it is built from them.
        internal static ScreenFilter Filter(JsonElement f)
        {
            string key = f.GetProperty("key").GetString()!;
            var parameters = f.GetProperty("settings").EnumerateArray().Select(s => new SlangParameter(s.GetProperty("id").GetString()!, s.GetProperty("description").GetString()!,
                Float(s.GetProperty("initial")), Float(s.GetProperty("minimum")), Float(s.GetProperty("maximum")), Float(s.GetProperty("step"))) { Choices = Strings(s.GetProperty("choices")) }).ToArray();
            JsonElement defaults = f.GetProperty("defaults"), aspect = f.GetProperty("aspect");
            return new ScreenFilter(f.GetProperty("name").GetString()!, f.GetProperty("passes").EnumerateArray().Select(Pass).ToArray(), Strings(f.GetProperty("consoles")), f.GetProperty("credit").GetString()!, parameters.Length == 0 ? null : parameters)
            {
                Key = key,
                Build = f.GetProperty("built").GetBoolean() ? values => Passes(key, values) : null,
                Structural = Strings(f.GetProperty("structural")),
                RawEncoding = f.GetProperty("raw").GetString(),
                ConsoleDefaults = defaults.ValueKind == JsonValueKind.Null ? null : defaults.EnumerateObject().ToDictionary(c => c.Name, c => (IReadOnlyDictionary<string, float>)Values(c.Value), StringComparer.Ordinal),
                Aspect = aspect.ValueKind == JsonValueKind.Null ? null : BitConverter.Int64BitsToDouble((long)aspect.GetUInt64()),
                CropsItself = f.GetProperty("crops_itself").GetBoolean(),
                RowsOnce = f.GetProperty("rows_once").GetBoolean(),
                HistoryDepth = f.GetProperty("history_depth").GetInt32(),
            };
        }

        private static void WriteValues(Utf8JsonWriter w, IReadOnlyDictionary<string, float> values)
        {
            w.WriteStartObject();
            foreach (var (id, value) in values) w.WriteNumber(id, SerenityNative.Bits(value));
            w.WriteEndObject();
        }

        private static byte[] ValuesJson(IReadOnlyDictionary<string, float> values)
        {
            using var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream)) WriteValues(w, values);
            return stream.ToArray();
        }

        private static void WriteSize(Utf8JsonWriter w, string name, PassSize? size)
        {
            if (size is not { } s) { w.WriteNull(name); return; }
            w.WriteStartObject(name);
            w.WriteNumber("axis", (int)s.Axis);
            w.WriteNumber("factor", SerenityNative.Bits(s.Factor));
            w.WriteEndObject();
        }

        // A filter as the library is given it: the catalog's by its key, a caller's by its description with the passes for these values.
        private static byte[] Document(ScreenFilter filter, IReadOnlyList<FilterPass>? passes)
        {
            using var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                if (filter.Key is { } key) w.WriteString("key", key);
                else
                {
                    w.WriteString("name", filter.Name);
                    w.WriteStartArray("passes");
                    foreach (FilterPass p in passes ?? Array.Empty<FilterPass>())
                    {
                        w.WriteStartObject();
                        w.WriteString("glsl", p.Glsl);
                        w.WriteNumber("scale", p.Scale == PassScale.Source ? 0 : 1);
                        w.WriteBoolean("linear_source", p.LinearSource);
                        w.WriteNumber("history", p.History);
                        WriteSize(w, "width", p.Width);
                        WriteSize(w, "height", p.Height);
                        w.WriteBoolean("float", p.Float);
                        w.WriteBoolean("feedback", p.Feedback);
                        if (p.Name is { } name) w.WriteString("name", name); else w.WriteNull("name");
                        w.WriteStartArray("linear");
                        foreach (string child in p.Linear ?? Array.Empty<string>()) w.WriteStringValue(child);
                        w.WriteEndArray();
                        w.WriteEndObject();
                    }
                    w.WriteEndArray();
                    w.WriteStartArray("settings");
                    foreach (SlangParameter p in filter.Parameters ?? Array.Empty<SlangParameter>())
                    {
                        w.WriteStartObject();
                        w.WriteString("id", p.Id);
                        w.WriteString("description", p.Description);
                        w.WriteNumber("initial", SerenityNative.Bits(p.Initial));
                        w.WriteNumber("minimum", SerenityNative.Bits(p.Minimum));
                        w.WriteNumber("maximum", SerenityNative.Bits(p.Maximum));
                        w.WriteNumber("step", SerenityNative.Bits(p.Step));
                        w.WriteEndObject();
                    }
                    w.WriteEndArray();
                    if (filter.ConsoleDefaults is { } defaults)
                    {
                        w.WriteStartObject("defaults");
                        foreach (var (console, values) in defaults) { w.WritePropertyName(console); WriteValues(w, values); }
                        w.WriteEndObject();
                    }
                    if (filter.RawEncoding is { } raw) w.WriteString("raw", raw);
                    w.WriteBoolean("rows_once", filter.RowsOnce);
                    w.WriteNumber("history_depth", filter.HistoryDepth);
                }
                w.WriteEndObject();
            }
            return stream.ToArray();
        }

        private static byte[]? Text(string? text) => text is null ? null : Encoding.UTF8.GetBytes(PlatformText.Mend(text));

        // The passes a caller's filter draws with these values: its own build's or its own list; the catalog's are the library's.
        private static IReadOnlyList<FilterPass>? Own(ScreenFilter filter, IReadOnlyDictionary<string, float> values) =>
            filter.Key is not null ? null : filter.Build?.Invoke(values) ?? filter.Passes;

        private static Exception Refused(long status) => status switch
        {
            SerenityNative.Parse => new InvalidOperationException(SerenityNative.Words()),
            SerenityNative.BadArgument => new ArgumentException(SerenityNative.Words()),
            _ => new IOException(SerenityNative.Words()),
        };

        private static string Answer(SerenityNative.Call call)
        {
            byte[]? bytes = SerenityNative.Bytes(call, out long status);
            if (bytes is null) throw Refused(status);
            return Encoding.UTF8.GetString(bytes);
        }

        internal static Dictionary<string, float> ValuesFor(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? given)
        {
            Up();
            byte[] doc = Document(filter, null);
            byte[]? where = Text(console), values = given is null ? null : ValuesJson(given);
            string answer = Answer((buffer, capacity) =>
            {
                fixed (byte* d = doc, c = where, v = values)
                    return SerenityNative.FilterValues(d, (nuint)doc.Length, where is null ? null : SerenityNative.Pin(c), (nuint)(where?.Length ?? 0), values is null ? null : SerenityNative.Pin(v), (nuint)(values?.Length ?? 0), buffer, capacity);
            });
            using JsonDocument read = JsonDocument.Parse(answer);
            return Values(read.RootElement);
        }

        internal static IReadOnlyList<FilterPass> Passes(string key, IReadOnlyDictionary<string, float> values)
        {
            Up();
            byte[] doc = Encoding.UTF8.GetBytes(JsonSerializer.Serialize(new { key })), given = ValuesJson(values);
            string answer = Answer((buffer, capacity) =>
            {
                fixed (byte* d = doc, v = given) return SerenityNative.FilterPasses(d, (nuint)doc.Length, SerenityNative.Pin(v), (nuint)given.Length, buffer, capacity);
            });
            using JsonDocument read = JsonDocument.Parse(answer);
            return read.RootElement.EnumerateArray().Select(Pass).ToArray();
        }

        internal static bool ReadsRaw(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? given)
        {
            Up();
            byte[] doc = Document(filter, filter.Key is null ? Own(filter, ValuesFor(filter, console, given)) : null);
            byte[]? where = Text(console), values = given is null ? null : ValuesJson(given);
            int answer;
            fixed (byte* d = doc, c = where, v = values)
                answer = SerenityNative.FilterReadsRaw(d, (nuint)doc.Length, where is null ? null : SerenityNative.Pin(c), (nuint)(where?.Length ?? 0), values is null ? null : SerenityNative.Pin(v), (nuint)(values?.Length ?? 0));
            if (answer < 0) throw Refused(answer);
            return answer == 1;
        }

        internal static int ReduceBy(ScreenFilter filter, int width, int height, int lines)
        {
            Up();
            byte[] doc = Document(filter, filter.Key is null ? filter.Passes : null);
            int answer;
            fixed (byte* d = doc) answer = SerenityNative.FilterReduceBy(d, (nuint)doc.Length, width, height, lines);
            if (answer < 0) throw Refused(answer);
            return answer;
        }

        internal static IReadOnlyList<BuiltInPreset.Pass> Translate(ScreenFilter filter, IReadOnlyDictionary<string, float> values, int reduceBy, (float X, float Y) nudge)
        {
            Up();
            byte[] doc = Document(filter, Own(filter, values)), given = ValuesJson(values);
            string answer = Answer((buffer, capacity) =>
            {
                fixed (byte* d = doc, v = given) return SerenityNative.FilterTranslate(d, (nuint)doc.Length, SerenityNative.Pin(v), (nuint)given.Length, reduceBy, nudge.X, nudge.Y, buffer, capacity);
            });
            using JsonDocument read = JsonDocument.Parse(answer);
            return read.RootElement.EnumerateArray().Select(p => new BuiltInPreset.Pass(p.GetProperty("file").GetString()!, p.GetProperty("text").GetString()!, p.GetProperty("alias").GetString(),
                Size(p.GetProperty("width"))!.Value, Size(p.GetProperty("height"))!.Value, p.GetProperty("float").GetBoolean(), p.GetProperty("linear").GetBoolean())).ToArray();
        }

        internal static string Write(string root, ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen, int reduceBy, (float X, float Y) nudge)
        {
            Up();
            byte[] doc = Document(filter, filter.Key is null ? Own(filter, ValuesFor(filter, console, chosen)) : null);
            byte[]? where = Text(console), values = chosen is null ? null : ValuesJson(chosen);
            byte[] folder = Text(root)!;
            return Answer((buffer, capacity) =>
            {
                fixed (byte* d = doc, c = where, v = values, r = folder)
                    return SerenityNative.FilterWrite(d, (nuint)doc.Length, where is null ? null : SerenityNative.Pin(c), (nuint)(where?.Length ?? 0), values is null ? null : SerenityNative.Pin(v), (nuint)(values?.Length ?? 0),
                        reduceBy, nudge.X, nudge.Y, SerenityNative.Pin(r), (nuint)folder.Length, buffer, capacity);
            });
        }
    }
}
