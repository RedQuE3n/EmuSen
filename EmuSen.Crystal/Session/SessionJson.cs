using System.Buffers;
using System.Text;
using System.Text.Json;

namespace EmuSen.Crystal.Session
{
    // A refusal with its status and the words last_error gives for it.
    public sealed class SessionException(int status, string message) : Exception(message)
    {
        public int Status { get; } = status;
    }

    // The service's JSON both ways: records written, arguments read - see emusen_session.h's rules.
    internal static class SessionJson
    {
        public static byte[] Write(Action<Utf8JsonWriter> body)
        {
            var buffer = new ArrayBufferWriter<byte>();
            using (var writer = new Utf8JsonWriter(buffer)) body(writer);
            return buffer.WrittenSpan.ToArray();
        }

        public static byte[] Array<T>(IEnumerable<T> items, Action<Utf8JsonWriter, T> item) => Write(w =>
        {
            w.WriteStartArray();
            foreach (T t in items) item(w, t);
            w.WriteEndArray();
        });

        public static byte[] Strings(IEnumerable<string> items) => Array(items, (w, s) => w.WriteStringValue(s));

        public static void Registers(Utf8JsonWriter w, IEnumerable<Telemetry.DebugRegisterValue> registers)
        {
            w.WriteStartArray();
            foreach (var r in registers)
            {
                w.WriteStartObject();
                w.WriteString("name", r.Name);
                w.WriteNumber("value", r.Value);
                w.WriteNumber("bits", r.BitWidth);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        }

        public static void NullableString(Utf8JsonWriter w, string key, string? value)
        {
            if (value is null) w.WriteNull(key);
            else w.WriteString(key, value);
        }

        public static void NullableNumber(Utf8JsonWriter w, string key, long? value)
        {
            if (value is { } v) w.WriteNumber(key, v);
            else w.WriteNull(key);
        }

        public static JsonElement Parse(ReadOnlySpan<byte> utf8)
        {
            if (utf8.IsEmpty) return Parse("{}"u8);
            try
            {
                var reader = new Utf8JsonReader(utf8);
                using var document = JsonDocument.ParseValue(ref reader);
                return document.RootElement.Clone();
            }
            catch (JsonException e)
            {
                throw new SessionException(SessionAbi.BadArgument, $"the argument is not JSON: {e.Message}");
            }
        }

        public static long Int(JsonElement args, string key)
            => OptionalInt(args, key) ?? throw new SessionException(SessionAbi.BadArgument, $"the argument has no \"{key}\"");

        public static long? OptionalInt(JsonElement args, string key)
        {
            if (args.ValueKind != JsonValueKind.Object || !args.TryGetProperty(key, out JsonElement v) || v.ValueKind == JsonValueKind.Null) return null;
            if (v.ValueKind != JsonValueKind.Number || !v.TryGetInt64(out long n))
                throw new SessionException(SessionAbi.BadArgument, $"\"{key}\" is not an integer");
            return n;
        }

        public static string? OptionalString(JsonElement args, string key)
        {
            if (args.ValueKind != JsonValueKind.Object || !args.TryGetProperty(key, out JsonElement v) || v.ValueKind == JsonValueKind.Null) return null;
            if (v.ValueKind != JsonValueKind.String) throw new SessionException(SessionAbi.BadArgument, $"\"{key}\" is not a string");
            return v.GetString();
        }

        public static string String(JsonElement args, string key)
            => OptionalString(args, key) ?? throw new SessionException(SessionAbi.BadArgument, $"the argument has no \"{key}\"");

        public static bool? OptionalBool(JsonElement args, string key)
        {
            if (args.ValueKind != JsonValueKind.Object || !args.TryGetProperty(key, out JsonElement v) || v.ValueKind == JsonValueKind.Null) return null;
            return v.ValueKind switch
            {
                JsonValueKind.True => true,
                JsonValueKind.False => false,
                _ => throw new SessionException(SessionAbi.BadArgument, $"\"{key}\" is not true or false"),
            };
        }

        public static int Int32(long value, string what)
            => value is >= int.MinValue and <= int.MaxValue ? (int)value : throw new SessionException(SessionAbi.BadArgument, $"{what} {value} is out of range");

        public static byte[] Utf8(string text) => Encoding.UTF8.GetBytes(text);

        // Addresses cross as 32-bit patterns: a target's int, read unsigned, as a MIPS kernel address needs.
        public static int Addr(uint address) => unchecked((int)address);

        public static uint U(int address) => unchecked((uint)address);

        public static int Address(JsonElement args, string key) => AddressOf(Int(args, key), key);

        public static int? OptionalAddress(JsonElement args, string key) => OptionalInt(args, key) is { } v ? AddressOf(v, key) : null;

        private static int AddressOf(long value, string key)
            => value is >= int.MinValue and <= uint.MaxValue ? unchecked((int)value) : throw new SessionException(SessionAbi.BadArgument, $"\"{key}\" {value} is not a 32-bit address");
    }
}
