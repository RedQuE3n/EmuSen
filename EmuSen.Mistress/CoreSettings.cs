using System;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress
{
    // A console's core settings, the runtime's in consoles.json since SR12: written through the frontend API, read from the runtime's file; Mistress's own keys stay in graphics.json - see EmuSen_Backend.md §14.12.
    internal static class CoreSettings
    {
        private static readonly Lazy<CrystalRuntime> Shared = new(() => new CrystalRuntime());

        // The process's one runtime of the frontend API.
        public static CrystalRuntime Runtime => Shared.Value;

        // A key as stored: the runtime's value for a core's setting, else Mistress's own in graphics.json; null for neither.
        public static Func<string, string?> StoredFor(string console, GraphicsConfig graphics)
        {
            ConsolesConfig consoles = ConsolesConfig.Load();
            return key => consoles.Value(console, key) ?? graphics.Value(console, key);
        }

        public static string? Stored(string console, string key) => ConsolesConfig.Load().Value(console, key);

        // Stored by the runtime, which checks it against the console's schema and applies it to a running session.
        public static void Set(string console, string key, string? value) => Runtime.SetSetting(console, key, value);

        // The engine a game opens on as the settings choose it, game over console, which the runtime's route answers.
        public static string? EngineFor(string path)
        {
            using var route = System.Text.Json.JsonDocument.Parse(Runtime.Route(path));
            return route.RootElement.TryGetProperty("engine", out var engine) && engine.ValueKind == System.Text.Json.JsonValueKind.String ? engine.GetString() : null;
        }
    }
}
