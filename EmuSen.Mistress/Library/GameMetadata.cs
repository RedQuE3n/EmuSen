using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Library
{
    public enum MetadataSource { Default, Scraped, Edited }

    public enum MetadataKind { Text, LongText, Rating, Date, Flag }

    // One field of ES-DE's metadata editor that Mistress keeps as an edit: its key in games.db, its label and its kind.
    public sealed record MetadataField(string Key, string Label, MetadataKind Kind);

    // A field's value as it is shown, and where it came from.
    public readonly record struct MetadataValue(string? Value, MetadataSource Source);

    // A game's metadata as every view shows it: the player's edit, else ScreenScraper's, else the file's - see EmuSen_Settings_Reference.md §4.59.
    public sealed class GameMetadata
    {
        public const string Name = "name", SortName = "sortname", Description = "description", Rating = "rating", ReleaseDate = "releasedate",
            Developer = "developer", Publisher = "publisher", Genre = "genre", Players = "players", Completed = "completed", KidGame = "kidgame",
            Hidden = "hidden", Broken = "broken", NotCounted = "nogamecount", NoMultiScrape = "nomultiscrape";

        // The editable fields in the order ES-DE's user guide lists them (its "Metadata entries").
        public static readonly IReadOnlyList<MetadataField> Fields =
        [
            new(Name, "Name", MetadataKind.Text),
            new(SortName, "Sort name", MetadataKind.Text),
            new(Description, "Description", MetadataKind.LongText),
            new(Rating, "Rating", MetadataKind.Rating),
            new(ReleaseDate, "Release date", MetadataKind.Date),
            new(Developer, "Developer", MetadataKind.Text),
            new(Publisher, "Publisher", MetadataKind.Text),
            new(Genre, "Genre", MetadataKind.Text),
            new(Players, "Players", MetadataKind.Text),
            new(Completed, "Completed", MetadataKind.Flag),
            new(KidGame, "Kid game", MetadataKind.Flag),
            new(Hidden, "Hidden", MetadataKind.Flag),
            new(Broken, "Broken / not working", MetadataKind.Flag),
            new(NotCounted, "Exclude from game counter", MetadataKind.Flag),
            new(NoMultiScrape, "Exclude from multi-scraper", MetadataKind.Flag),
        ];

        public const string Yes = "1", No = "0";

        private readonly Dictionary<string, MetadataValue> _values;

        private GameMetadata(string path, Dictionary<string, MetadataValue> values)
        {
            Path = path;
            _values = values;
        }

        public string Path { get; }

        public MetadataValue this[string field] => _values.TryGetValue(field, out MetadataValue v) ? v : default;

        public static GameMetadata Resolve(string path, ScrapedRecord? scraped, IReadOnlyDictionary<string, string>? edits)
        {
            var values = new Dictionary<string, MetadataValue>(StringComparer.Ordinal);
            foreach (MetadataField field in Fields)
            {
                if (edits is not null && edits.TryGetValue(field.Key, out string? edited)) values[field.Key] = new MetadataValue(edited, MetadataSource.Edited);
                else values[field.Key] = Baseline(path, scraped, field.Key);
            }
            return new GameMetadata(path, values);
        }

        // What a field shows with no edit: ScreenScraper's text where it has some, else the default, which for the name is the file's.
        public static MetadataValue Baseline(string path, ScrapedRecord? scraped, string field) =>
            Scraped(scraped, field) is { } s ? new MetadataValue(s, MetadataSource.Scraped) : new MetadataValue(Default(path, field), MetadataSource.Default);

        // ScreenScraper's name is kept and never shown, as §17.6 of the plan decided, so the name's baseline is the file's.
        public static string? Scraped(ScrapedRecord? r, string field) => r is null ? null : field switch
        {
            Description => r.Description,
            Developer => r.Developer,
            Publisher => r.Publisher,
            Genre => r.Genre,
            Players => r.Players,
            Rating => r.Rating is { } v ? FormatRating(v) : null,
            ReleaseDate => r.ReleaseDate is { } d ? FormatDate(d) : null,
            _ => null,
        };

        public static string? Default(string path, string field) => field switch
        {
            Name => System.IO.Path.GetFileNameWithoutExtension(path),
            Completed or KidGame or Hidden or Broken or NotCounted or NoMultiScrape => No,
            _ => null,
        };

        public static string FormatRating(float rating) => Math.Clamp(rating, 0, 1).ToString("0.###", CultureInfo.InvariantCulture);

        public static string FormatDate(DateTime date) => date.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture);

        public static float? ParseRating(string? text) =>
            float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float v) ? Math.Clamp(v, 0, 1) : null;

        public static DateTime? ParseDate(string? text) =>
            DateTime.TryParseExact(text, "yyyy-MM-dd", CultureInfo.InvariantCulture, DateTimeStyles.None, out DateTime d) ? d : null;

        private string? Text(string field) => this[field].Value is { Length: > 0 } v ? v : null;

        private bool Flag(string field) => this[field].Value == Yes;

        public string Title => Text(Name) ?? System.IO.Path.GetFileNameWithoutExtension(Path);
        public string? Sort => Text(SortName);
        public string? DescriptionText => Text(Description);
        public string? DeveloperText => Text(Developer);
        public string? PublisherText => Text(Publisher);
        public string? GenreText => Text(Genre);
        public string? PlayersText => Text(Players);
        public float? RatingValue => ParseRating(this[Rating].Value);
        public DateTime? Released => ParseDate(this[ReleaseDate].Value);
        public bool IsCompleted => Flag(Completed);
        public bool IsKidGame => Flag(KidGame);
        public bool IsHidden => Flag(Hidden);
        public bool IsBroken => Flag(Broken);
        public bool IsNotCounted => Flag(NotCounted);
        public bool IsExcludedFromMultiScrape => Flag(NoMultiScrape);

        public bool HasEdits => _values.Values.Any(v => v.Source == MetadataSource.Edited);
    }
}
