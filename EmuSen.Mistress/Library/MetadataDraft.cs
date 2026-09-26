using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Library
{
    // What the metadata editor holds between opening and Save: each field's value against its scraped or default baseline - see EmuSen_Settings_Reference.md §4.59.
    public sealed class MetadataDraft
    {
        private readonly Dictionary<string, string?> _values = new(StringComparer.Ordinal);
        private readonly Dictionary<string, string> _stored;
        private readonly HashSet<string> _fromScrape = new(StringComparer.Ordinal);
        private ScrapedRecord? _scraped;

        public MetadataDraft(string path, ScrapedRecord? scraped, IReadOnlyDictionary<string, string>? edits, GameRecord? record)
        {
            Path = path;
            _scraped = scraped;
            _stored = new Dictionary<string, string>(edits ?? new Dictionary<string, string>(), StringComparer.Ordinal);
            foreach (MetadataField field in GameMetadata.Fields)
                _values[field.Key] = _stored.TryGetValue(field.Key, out string? edited) ? edited : Baseline(field.Key).Value;
            Favourite = StoredFavourite = record?.Favourite == true;
            PlayCount = StoredPlayCount = record?.PlayCount ?? 0;
            PlaySeconds = StoredPlaySeconds = record?.PlaySeconds ?? 0;
        }

        public string Path { get; }

        public bool Favourite { get; set; }
        public int PlayCount { get; set; }
        public double PlaySeconds { get; set; }
        public bool StoredFavourite { get; }
        public int StoredPlayCount { get; }
        public double StoredPlaySeconds { get; }

        public MetadataValue Baseline(string field) => GameMetadata.Baseline(Path, _scraped, field);

        public string? Value(string field) => _values.GetValueOrDefault(field);

        // Edited when the value differs from what the field would show with no edit; a value equal to it is no edit.
        public MetadataSource Source(string field) => Value(field) != Baseline(field).Value ? MetadataSource.Edited : Baseline(field).Source;

        // Set by the editor's own scrape and not yet saved, which ES-DE marks in its own colour.
        public bool FromScrape(string field) => _fromScrape.Contains(field);

        // An emptied box over a field with nothing behind it is no edit; over a scraped value it is an edit that blanks it.
        public void Set(string field, string? value)
        {
            if (value is "" && Baseline(field).Value is null) value = null;
            _values[field] = value;
            _fromScrape.Remove(field);
        }

        // Back to the scraped or default value; a saved edit of the field is removed at Save.
        public void Reset(string field) => Set(field, Baseline(field).Value);

        // The editor's own scrape: the fresh answer becomes the baseline, and every field it has a value for takes that value, as ES-DE's editor fills its fields.
        public void TakeScraped(ScrapedRecord? fresh)
        {
            _scraped = fresh;
            foreach (MetadataField field in GameMetadata.Fields)
            {
                if (GameMetadata.Scraped(fresh, field.Key) is not { } value) continue;
                _values[field.Key] = value;
                _fromScrape.Add(field.Key);
            }
        }

        // What Save writes to games.db: an edit for each field that now differs from its baseline, null for one that no longer does, and only where the stored state changes.
        public IReadOnlyDictionary<string, string?> Changes()
        {
            var changes = new Dictionary<string, string?>(StringComparer.Ordinal);
            foreach (MetadataField field in GameMetadata.Fields)
            {
                string? wanted = Source(field.Key) == MetadataSource.Edited ? Value(field.Key) ?? "" : null;
                string? stored = _stored.GetValueOrDefault(field.Key);
                if (wanted != stored) changes[field.Key] = wanted;
            }
            return changes;
        }

        public bool IsDirty => Changes().Count > 0 || Favourite != StoredFavourite || PlayCount != StoredPlayCount || Math.Abs(PlaySeconds - StoredPlaySeconds) > 0.5;

        public IEnumerable<string> EditedFields => GameMetadata.Fields.Select(f => f.Key).Where(k => Source(k) == MetadataSource.Edited);
    }
}
