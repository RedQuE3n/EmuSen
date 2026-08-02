using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using EmuSen.Galaxia.Text;

namespace EmuSen.Galaxia
{
    // JsonStringEnumConverter with a better failure message: it names the
    // value that was rejected and suggests the nearest real one - see
    // EmuSen_Config_Reference.md §6.2. Everything else about it matches, so
    // numeric values still read and names still write.
    internal sealed class SuggestingEnumConverterFactory : JsonConverterFactory
    {
        public override bool CanConvert(Type typeToConvert) => typeToConvert.IsEnum;

        public override JsonConverter CreateConverter(Type typeToConvert, JsonSerializerOptions options) =>
            (JsonConverter)Activator.CreateInstance(
                typeof(SuggestingEnumConverter<>).MakeGenericType(typeToConvert))!;
    }

    internal sealed class SuggestingEnumConverter<T> : JsonConverter<T> where T : struct, Enum
    {
        private static readonly string[] Names = Enum.GetNames<T>();

        public override T Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options) =>
            reader.TokenType == JsonTokenType.Number
                ? (T)Enum.ToObject(typeof(T), reader.GetInt64())
                : Parse(reader.GetString());

        // Dictionary keys go through here, not Read - the three binding files
        // are all Dictionary<enum, enum>, so both paths matter.
        public override T ReadAsPropertyName(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options) =>
            Parse(reader.GetString());

        public override void Write(Utf8JsonWriter writer, T value, JsonSerializerOptions options) =>
            writer.WriteStringValue(value.ToString());

        public override void WriteAsPropertyName(Utf8JsonWriter writer, T value, JsonSerializerOptions options) =>
            writer.WritePropertyName(value.ToString());

        // Enum.TryParse takes a numeric string too, which keeps a file that
        // mixes the two forms working.
        private static T Parse(string? text)
        {
            if (Enum.TryParse(text, ignoreCase: true, out T value)) return value;

            throw new JsonException(
                $"'{text}' is not a valid {typeof(T).Name}.{Suggestion.Hint(text ?? "", Names)}");
        }
    }
}
