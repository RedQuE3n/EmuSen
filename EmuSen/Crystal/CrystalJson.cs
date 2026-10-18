using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace EmuSen.Crystal.Managed
{
    // The API's JSON both ways: records written compact in snake_case, arguments read with their refusals in the API's words.
    internal static class CrystalJson
    {
        public static byte[] Write(Action<Utf8JsonWriter> write)
        {
            using var buffer = new MemoryStream();
            using (var w = new Utf8JsonWriter(buffer)) write(w);
            return buffer.ToArray();
        }

        public static void Strings(Utf8JsonWriter w, string name, IEnumerable<string> values)
        {
            w.WriteStartArray(name);
            foreach (string v in values) w.WriteStringValue(v);
            w.WriteEndArray();
        }

        public static void StringOrNull(Utf8JsonWriter w, string name, string? value)
        {
            if (value is null) w.WriteNull(name);
            else w.WriteString(name, value);
        }

        public static JsonElement Parse(ReadOnlySpan<byte> json)
        {
            if (json.IsEmpty) return JsonDocument.Parse("{}").RootElement.Clone();
            try
            {
                using JsonDocument doc = JsonDocument.Parse(json.ToArray());
                return doc.RootElement.Clone();
            }
            catch (JsonException e)
            {
                throw new CrystalException(Session.SessionAbi.BadArgument, $"the argument is not JSON: {e.Message}");
            }
        }

        public static string String(JsonElement args, string name) =>
            args.ValueKind == JsonValueKind.Object && args.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString()!
            : throw new CrystalException(Session.SessionAbi.BadArgument, $"\"{name}\" must be given as a string");

        public static string? OptionalString(JsonElement args, string name) =>
            args.ValueKind == JsonValueKind.Object && args.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;
    }
}
