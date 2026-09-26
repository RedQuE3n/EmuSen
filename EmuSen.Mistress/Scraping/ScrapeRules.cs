using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.RegularExpressions;

namespace EmuSen.Mistress.Scraping
{
    // One kind of media Mistress fetches: ES-DE's media type and folder, and ScreenScraper's names for it in the order tried - see EmuSen_BigPicture.md §5.4.
    public sealed record ScrapeMediaKind(string EsdeType, string Folder, IReadOnlyList<string> ScreenScraperTypes);

    // What to fetch and in which region and language; the settings of Preferences, handed to the worker as one value.
    public sealed record ScrapeChoices
    {
        public bool Covers { get; init; } = true;
        public bool Screenshots { get; init; } = true;
        public bool Marquees { get; init; } = true;
        public bool TitleScreens { get; init; }
        public bool Miximages { get; init; } = true;
        public string Region { get; init; } = ScrapeRules.AutomaticRegion;
        public string Language { get; init; } = "en";
        public bool RegionFallback { get; init; } = true;
        public int Threads { get; init; } = 1;

        public IEnumerable<ScrapeMediaKind> Kinds()
        {
            if (Covers) yield return ScrapeRules.Cover;
            if (Screenshots) yield return ScrapeRules.Screenshot;
            if (Marquees) yield return ScrapeRules.Marquee;
            if (TitleScreens) yield return ScrapeRules.TitleScreen;
            if (Miximages) yield return ScrapeRules.Miximage;
        }
    }

    // The region, language and rating rules of §5.4, which decide what is kept from an answer - see EmuSen_BigPicture.md §17.
    public static partial class ScrapeRules
    {
        public const string AutomaticRegion = "auto";

        public static readonly ScrapeMediaKind Cover = new("cover", "covers", ["box-2D"]);
        public static readonly ScrapeMediaKind Screenshot = new("screenshot", "screenshots", ["ss"]);
        public static readonly ScrapeMediaKind Marquee = new("marquee", "marquees", ["wheel-hd", "wheel"]);
        public static readonly ScrapeMediaKind TitleScreen = new("titlescreen", "titlescreens", ["sstitle"]);
        public static readonly ScrapeMediaKind Miximage = new("miximage", "miximages", ["mixrbv2"]);

        public static readonly IReadOnlyList<ScrapeMediaKind> AllKinds = [Cover, Screenshot, Marquee, TitleScreen, Miximage];

        // ES-DE's documented fallback after the preferred region: world, USA, EU, Japan, then ScreenScraper's own ("custom").
        public static readonly IReadOnlyList<string> FallbackRegions = ["wor", "us", "eu", "jp", "ss"];

        // No-Intro's country names to ScreenScraper's region codes.
        private static readonly Dictionary<string, string> Countries = new(StringComparer.OrdinalIgnoreCase)
        {
            ["World"] = "wor", ["USA"] = "us", ["Europe"] = "eu", ["Japan"] = "jp", ["Germany"] = "de", ["France"] = "fr", ["Spain"] = "sp",
            ["Italy"] = "it", ["Netherlands"] = "nl", ["Sweden"] = "se", ["Australia"] = "au", ["Korea"] = "kr", ["Brazil"] = "br",
            ["China"] = "cn", ["Asia"] = "asi", ["Taiwan"] = "tw", ["UK"] = "uk", ["Canada"] = "ca",
        };

        [GeneratedRegex(@"\(([^()]*)\)")]
        private static partial Regex Tag();

        // The first tag whose every name is a country: "(USA, Europe)" is us, "(En,Fr,De)" is not a region; null when there is none.
        public static string? RegionOfFileName(string fileName)
        {
            foreach (Match tag in Tag().Matches(fileName))
            {
                string[] names = tag.Groups[1].Value.Split(',', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
                if (names.Length > 0 && names.All(Countries.ContainsKey)) return Countries[names[0]];
            }
            return null;
        }

        // The regions tried, in order: the player's choice or the file's tag, then with the fallback on ES-DE's list; world when nothing says.
        public static IReadOnlyList<string> RegionOrder(string fileName, string choice, bool fallback)
        {
            string preferred = choice != AutomaticRegion && !string.IsNullOrWhiteSpace(choice) ? choice : RegionOfFileName(fileName) ?? "wor";
            var order = new List<string> { preferred };
            if (fallback) order.AddRange(FallbackRegions.Where(r => r != preferred));
            return order;
        }

        public static IReadOnlyList<string> LanguageOrder(string preferred) =>
            string.IsNullOrWhiteSpace(preferred) || preferred == "en" ? ["en"] : [preferred, "en"];

        // The file of one kind to fetch: each ScreenScraper type in turn, by region order; then one with no region; then any region, only with the fallback on.
        public static ScrapedMedia? ChooseMedia(IReadOnlyList<ScrapedMedia> media, ScrapeMediaKind kind, IReadOnlyList<string> regions, bool fallback)
        {
            foreach (string type in kind.ScreenScraperTypes)
            {
                var ofType = media.Where(m => m.Type == type).ToList();
                if (ofType.Count == 0) continue;
                foreach (string region in regions)
                    if (ofType.FirstOrDefault(m => m.Region == region) is { } hit) return hit;
                if (ofType.FirstOrDefault(m => string.IsNullOrEmpty(m.Region)) is { } plain) return plain;
                if (fallback) return ofType[0];
            }
            return null;
        }

        // A text by key order, then, only with the fallback on, any at all.
        public static Localised? ChooseText(IReadOnlyList<Localised> texts, IReadOnlyList<string> order, bool fallback)
        {
            foreach (string key in order)
                if (texts.FirstOrDefault(t => t.Key == key) is { } hit) return hit;
            return fallback ? texts.FirstOrDefault() : null;
        }

        // The main genre's name in the language order, else the first genre's.
        public static string? Genre(IReadOnlyList<ScrapedGenre> genres, IReadOnlyList<string> languages)
        {
            ScrapedGenre? genre = genres.FirstOrDefault(g => g.Main) ?? genres.FirstOrDefault();
            return genre is null ? null : ChooseText(genre.Names, languages, fallback: true)?.Text;
        }

        // ScreenScraper's note is out of 20; ES-DE keeps a fraction shown in half stars, so note / 20 to the nearest 0.1.
        public static float? Rating(double? note) =>
            note is double n && n >= 0 ? (float)Math.Round(Math.Clamp(n, 0, 20) / 20.0, 1, MidpointRounding.AwayFromZero) : null;

        // A full date, a year and month, or a year alone; the rest of ES-DE's forms are not ScreenScraper's.
        public static DateTime? Date(string? text)
        {
            if (string.IsNullOrWhiteSpace(text)) return null;
            foreach (string format in (string[])["yyyy-MM-dd", "yyyy-MM", "yyyy"])
                if (DateTime.TryParseExact(text.Trim(), format, CultureInfo.InvariantCulture, DateTimeStyles.None, out DateTime d)) return d;
            return null;
        }

        // The extension a media file is written with: ScreenScraper's format when it is one of the kept images, else png.
        public static string Extension(ScrapedMedia media) => media.Format?.ToLowerInvariant() switch
        {
            "jpg" or "jpeg" => ".jpg",
            _ => ".png",
        };
    }
}
