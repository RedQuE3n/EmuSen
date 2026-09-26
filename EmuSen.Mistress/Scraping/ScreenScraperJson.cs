using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json;

namespace EmuSen.Mistress.Scraping
{
    // A text keyed by a region or a language, as ScreenScraper lists names, synopses, dates and genre names.
    public sealed record Localised(string Key, string Text);

    public sealed record ScrapedGenre(IReadOnlyList<Localised> Names, bool Main);

    // One media file the game has, with the address ScreenScraper serves it from; the address carries the credentials and is redacted before it is shown.
    public sealed record ScrapedMedia(string Type, string? Region, string Url, string? Format);

    // The member's limits and today's counts, as every response carries them; null where a field was absent.
    public sealed record ScrapeQuota(int? MaxThreads, int? MaxDownloadKBps, int? RequestsToday, int? RequestsKoToday,
        int? MaxRequestsPerMinute, int? MaxRequestsPerDay, int? MaxRequestsKoPerDay);

    public sealed record ScrapedGame(long Id, long? RomId, int? SystemId)
    {
        public IReadOnlyList<Localised> Names { get; init; } = [];
        public IReadOnlyList<Localised> Synopses { get; init; } = [];
        public IReadOnlyList<Localised> Dates { get; init; } = [];
        public IReadOnlyList<ScrapedGenre> Genres { get; init; } = [];
        public string? Developer { get; init; }
        public string? Publisher { get; init; }
        public string? Players { get; init; }
        public double? Note { get; init; }
        public IReadOnlyList<ScrapedMedia> Media { get; init; } = [];
    }

    // Every reading of ScreenScraper's JSON in one place, because API v2 is a beta that may change - see EmuSen_BigPicture.md §5.1 and §17.
    public static class ScreenScraperJson
    {
        // The game and the quota of a jeuInfos answer; a throw means the text was not the documented shape.
        public static (ScrapedGame? Game, ScrapeQuota? Quota) JeuInfos(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            JsonElement response = doc.RootElement.GetProperty("response");
            ScrapeQuota? quota = response.TryGetProperty("ssuser", out JsonElement user) ? Quota(user) : null;
            if (!response.TryGetProperty("jeu", out JsonElement jeu) || jeu.ValueKind != JsonValueKind.Object) return (null, quota);

            long id = Long(Prop(jeu, "id")) ?? throw new FormatException("a game without an id");
            long? romId = Long(Prop(jeu, "romid")) ?? Long(Prop(Prop(jeu, "rom"), "id"));
            var game = new ScrapedGame(id, romId, Int(Prop(Prop(jeu, "systeme"), "id")))
            {
                Names = List(Prop(jeu, "noms"), "region"),
                Synopses = List(Prop(jeu, "synopsis"), "langue"),
                Dates = List(Prop(jeu, "dates"), "region"),
                Genres = Array(Prop(jeu, "genres")).Select(g => new ScrapedGenre(List(Prop(g, "noms"), "langue"), Text(Prop(g, "principale")) == "1")).ToList(),
                Developer = Text(Prop(Prop(jeu, "developpeur"), "text")),
                Publisher = Text(Prop(Prop(jeu, "editeur"), "text")),
                Players = Text(Prop(Prop(jeu, "joueurs"), "text")),
                Note = Double(Prop(Prop(jeu, "note"), "text")),
                Media = Array(Prop(jeu, "medias"))
                    .Where(m => Text(Prop(m, "type")) is not null && Text(Prop(m, "url")) is not null)
                    .Select(m => new ScrapedMedia(Text(Prop(m, "type"))!, Text(Prop(m, "region")), Text(Prop(m, "url"))!, Text(Prop(m, "format"))))
                    .ToList(),
            };
            return (game, quota);
        }

        // ssuserInfos: only the quota.
        public static ScrapeQuota? User(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            return doc.RootElement.GetProperty("response").TryGetProperty("ssuser", out JsonElement user) ? Quota(user) : null;
        }

        // ssuserInfos as a sign-in: the login name (id), the level (niveau) and the quota; null when the answer names no member.
        public static ScrapeMember? Member(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            if (!doc.RootElement.GetProperty("response").TryGetProperty("ssuser", out JsonElement user) || user.ValueKind != JsonValueKind.Object) return null;
            string? name = Text(Prop(user, "id"));
            return string.IsNullOrWhiteSpace(name) ? null : new ScrapeMember(name, Text(Prop(user, "niveau")), Quota(user));
        }

        // systemesListe: each system's id and its name.
        public static IReadOnlyDictionary<int, string> Systems(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            var systems = new Dictionary<int, string>();
            foreach (JsonElement s in Array(Prop(doc.RootElement.GetProperty("response"), "systemes")))
            {
                if (Int(Prop(s, "id")) is not int id) continue;
                JsonElement names = Prop(s, "noms");
                systems[id] = Text(Prop(names, "nom_eu")) ?? Text(Prop(names, "nom_us")) ?? Text(Prop(names, "nom_recalbox")) ?? id.ToString(CultureInfo.InvariantCulture);
            }
            return systems;
        }

        private static ScrapeQuota Quota(JsonElement user) => new(
            Int(Prop(user, "maxthreads")), Int(Prop(user, "maxdownloadspeed")), Int(Prop(user, "requeststoday")), Int(Prop(user, "requestskotoday")),
            Int(Prop(user, "maxrequestspermin")), Int(Prop(user, "maxrequestsperday")), Int(Prop(user, "maxrequestskoperday")));

        private static IReadOnlyList<Localised> List(JsonElement items, string key) =>
            Array(items).Select(i => (Key: Text(Prop(i, key)) ?? "", Text: Text(Prop(i, "text"))))
                .Where(i => !string.IsNullOrWhiteSpace(i.Text)).Select(i => new Localised(i.Key, i.Text!)).ToList();

        private static JsonElement Prop(JsonElement e, string name) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out JsonElement v) ? v : default;

        private static IEnumerable<JsonElement> Array(JsonElement e) => e.ValueKind == JsonValueKind.Array ? e.EnumerateArray() : [];

        // ScreenScraper writes numbers as strings; either is read.
        private static string? Text(JsonElement e) => e.ValueKind switch
        {
            JsonValueKind.String => e.GetString(),
            JsonValueKind.Number => e.GetRawText(),
            _ => null,
        };

        private static long? Long(JsonElement e) => long.TryParse(Text(e), NumberStyles.Integer, CultureInfo.InvariantCulture, out long v) ? v : null;

        private static int? Int(JsonElement e) => int.TryParse(Text(e), NumberStyles.Integer, CultureInfo.InvariantCulture, out int v) ? v : null;

        private static double? Double(JsonElement e) => double.TryParse(Text(e), NumberStyles.Float, CultureInfo.InvariantCulture, out double v) ? v : null;
    }
}
