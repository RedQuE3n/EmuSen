using System.Text.Json;

namespace EmuSen.Tools.Packages
{
    // A field of one of the library tool's event lines, read the one way every part of Packages reads it - see EmuSen_GamePackages.md §12.3.
    internal static class ToolEvents
    {
        public static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        public static long Number(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt64(out long n) ? n : 0;
    }
}
