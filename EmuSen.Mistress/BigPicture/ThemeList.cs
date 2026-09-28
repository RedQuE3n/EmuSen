using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;

namespace EmuSen.Mistress.BigPicture
{
    public sealed record ThemeScreenshot(string Image, string? Caption);

    // One theme of ES-DE's official list, with what the list itself states of it - see EmuSen_BigPicture.md §25.
    public sealed record ThemeListEntry(
        string Name, string RepoName, string Url, string Author, bool NewEntry,
        IReadOnlyList<string> Variants, IReadOnlyList<string> ColorSchemes, IReadOnlyList<string> FontSizes,
        IReadOnlyList<string> AspectRatios, IReadOnlyList<string> Transitions, IReadOnlyList<string> Languages,
        IReadOnlyList<ThemeScreenshot> Screenshots)
    {
        // The repository the list names; the branch is learned from the host when the player looks closer.
        public ThemeSource? Source => ThemeSource.FromUrl(Url);
    }

    // ES-DE's themes.json, as fetched: the list the theme browser shows.
    public sealed record ThemeList(IReadOnlyList<ThemeListEntry> Themes, string? LatestStableRelease, DateTimeOffset FetchedAt, IReadOnlyList<string> Skipped)
    {
        // The list's raw file in ES-DE's themes-list repository on GitLab, whose screenshots sit beside it - see USERGUIDE.md "Theme downloader".
        public const string RepositoryRaw = "https://gitlab.com/es-de/themes/themes-list/-/raw/master/";
        public const string Address = RepositoryRaw + "themes.json";

        // A screenshot's path in the list is relative to the list's repository; a path that climbs out of it is refused.
        public static string? ScreenshotAddress(string image)
        {
            string[] parts = image.Replace('\\', '/').Split('/', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length == 0 || parts.Any(p => p is "." or "..")) return null;
            return RepositoryRaw + string.Join('/', parts.Select(Uri.EscapeDataString));
        }

        // The desktop list only: themesAndroid is for ES-DE's Android build. An entry without a name or a GitHub or GitLab URL is skipped and named.
        public static ThemeList Parse(string json, DateTimeOffset fetchedAt)
        {
            using JsonDocument doc = JsonDocument.Parse(json, new JsonDocumentOptions { AllowTrailingCommas = true, CommentHandling = JsonCommentHandling.Skip });
            JsonElement root = doc.RootElement;
            if (root.ValueKind != JsonValueKind.Object || !root.TryGetProperty("themes", out JsonElement themes) || themes.ValueKind != JsonValueKind.Array)
                throw new JsonException("themes.json holds no \"themes\" array.");
            var entries = new List<ThemeListEntry>();
            var skipped = new List<string>();
            foreach (JsonElement t in themes.EnumerateArray())
            {
                string name = Text(t, "name") ?? "";
                string url = Text(t, "url") ?? "";
                if (t.ValueKind != JsonValueKind.Object || name.Length == 0 || ThemeSource.FromUrl(url) is not { } source)
                {
                    skipped.Add(name.Length > 0 ? name : "(an entry without a name)");
                    continue;
                }
                var shots = new List<ThemeScreenshot>();
                if (t.TryGetProperty("screenshots", out JsonElement s) && s.ValueKind == JsonValueKind.Array)
                    foreach (JsonElement shot in s.EnumerateArray())
                        if (Text(shot, "image") is { Length: > 0 } image && ScreenshotAddress(image) is not null) shots.Add(new ThemeScreenshot(image, Text(shot, "caption")));
                entries.Add(new ThemeListEntry(name, Text(t, "reponame") ?? source.Repository, url, Text(t, "author") ?? "", t.TryGetProperty("newEntry", out JsonElement n) && n.ValueKind == JsonValueKind.True,
                    Strings(t, "variants"), Strings(t, "colorSchemes"), Strings(t, "fontSizes"), Strings(t, "aspectRatios"), Strings(t, "transitions"), Strings(t, "languages"), shots));
            }
            string? release = root.TryGetProperty("latestStableRelease", out JsonElement r) ? r.ValueKind == JsonValueKind.String ? r.GetString() : r.ToString() : null;
            return new ThemeList(entries, release, fetchedAt, skipped);
        }

        private static string? Text(JsonElement e, string name) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString()?.Trim() : null;

        private static IReadOnlyList<string> Strings(JsonElement e, string name) =>
            e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Array
                ? v.EnumerateArray().Where(x => x.ValueKind == JsonValueKind.String).Select(x => x.GetString()!.Trim()).Where(x => x.Length > 0).ToList()
                : [];
    }
}
