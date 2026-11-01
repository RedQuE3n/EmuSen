using System.Text.Json;

namespace EmuSen.Galaxia.Native
{
    // A field of one of the library tool's event lines, read the one way every window over its jobs reads it - see EmuSen_GamePackages.md §12.3.
    public static class ToolEvents
    {
        public static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        public static long Number(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt64(out long n) ? n : 0;
    }
}
