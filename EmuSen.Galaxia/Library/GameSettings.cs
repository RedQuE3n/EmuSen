using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;

namespace EmuSen.Galaxia.Library
{
    // A game's own settings: /etc/EmuSen/games/<system>/<stem>.json, or a packaged game's settings.json once it has one, the twin of crystal's settings.rs - see EmuSen_Backend.md §14.7 and EmuSen_GamePackages.md §18.2.
    public static class GameSettings
    {
        // The game's crop in its settings: four percentages, or "" for none of its own - see EmuSen_GamePackages.md §18.2.
        public const string CropKey = "Crop";

        public static string PlainPath(string system, string gameKey) =>
            Path.Combine(ConfigStore.Directory, "games", system, Path.GetFileNameWithoutExtension(gameKey) + ".json");

        public static string PackagePath(string packageRoot) => Path.Combine(packageRoot, Media.GamePackage.SettingsFile);

        // The file a game's values are read from: its package's where it has one, else the plain file; null for neither.
        public static string? ReadPath(string? system, string gameKey, string? packageRoot) =>
            packageRoot is not null && File.Exists(PackagePath(packageRoot)) ? PackagePath(packageRoot) : system is null ? null : PlainPath(system, gameKey);

        // The file they are written to: a packaged game's package, so that its first write copies what was read in.
        public static string? WritePath(string? system, string gameKey, string? packageRoot) =>
            packageRoot is not null ? PackagePath(packageRoot) : system is null ? null : PlainPath(system, gameKey);

        public static Dictionary<string, string> Read(string? system, string gameKey, string? packageRoot)
        {
            if (ReadPath(system, gameKey, packageRoot) is not { } path || AtomicFile.TryRead(path) is not { } bytes) return new Dictionary<string, string>(StringComparer.Ordinal);
            try { return JsonSerializer.Deserialize<Dictionary<string, string>>(bytes) ?? new Dictionary<string, string>(StringComparer.Ordinal); }
            catch (JsonException e)
            {
                ErrorLog.Warning("settings", $"{path} is not a game's settings, and is read as none: {e.Message}");
                return new Dictionary<string, string>(StringComparer.Ordinal);
            }
        }

        // The values written whole, indented as JsonSerializer writes a dictionary; null where the game has no file of its own.
        public static string? Write(string? system, string gameKey, string? packageRoot, Dictionary<string, string> values)
        {
            if (WritePath(system, gameKey, packageRoot) is not { } path) return null;
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            if (!AtomicFile.Write(path, JsonSerializer.SerializeToUtf8Bytes(values, new JsonSerializerOptions { WriteIndented = true }))) throw new IOException($"{path} could not be written");
            return path;
        }

        // The crop's text: left, right, top and bottom, invariant, a space between.
        public static string CropText(double left, double right, double top, double bottom) =>
            string.Join(' ', new[] { left, right, top, bottom }.Select(v => v.ToString(CultureInfo.InvariantCulture)));

        // The four percentages, or null for "" or text that is not four numbers.
        public static (double Left, double Right, double Top, double Bottom)? ParseCrop(string text)
        {
            string[] parts = text.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 4) return null;
            var v = new double[4];
            for (int i = 0; i < 4; i++)
                if (!double.TryParse(parts[i], NumberStyles.Float, CultureInfo.InvariantCulture, out v[i]) || !double.IsFinite(v[i])) return null;
            return (v[0], v[1], v[2], v[3]);
        }
    }
}
