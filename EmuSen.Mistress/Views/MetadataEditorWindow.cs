using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // What the metadata editor asks of the window that owns the library and the stores.
    public interface IGameEditorHost
    {
        MetadataDraft DraftFor(string path);
        ScrapedRecord? ScrapedNow(string path);
        Task<bool> ScrapeGameAsync(string path);
        bool ScrapeRunning { get; }
        event Action? ScrapeChanged;
        void SaveMetadata(MetadataDraft draft);
        void ClearMetadata(string path);
        void HideGame(string path);
    }

    // ES-DE's metadata editor for one game, as its user guide documents it, with Hide from Library where ES-DE deletes the file - see EmuSen_Settings_Reference.md §4.59.
    public sealed class MetadataEditorWindow : ToolWindow, IPadDriven
    {
        public const string HideText = "Hide from Library";

        private readonly IGameEditorHost _host;
        private readonly Dictionary<string, (FieldRow Row, Control Editor, Button Reset)> _rows = new(StringComparer.Ordinal);
        private readonly LunaSwitch _favourite = new() { Name = "Meta_favourite", Label = "Favourite" };
        private readonly TextBox _playCount = new() { Name = "Meta_playcount", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBox _playTime = new() { Name = "Meta_playtime", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBlock _status = new() { Name = "MetadataStatus", TextWrapping = TextWrapping.Wrap };
        private readonly Button _scrape;
        private bool _filling;
        private bool _waitingForScrape;
        private bool _closed;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public MetadataEditorWindow() : this(null!, "", "") { }

        public MetadataEditorWindow(IGameEditorHost host, string path, string title)
        {
            _host = host;
            GamePath = path;
            Title = "Edit Metadata";
            Width = 820;
            Height = 700;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = false;

            var fields = Ui.Stack(10);
            foreach (MetadataField field in GameMetadata.Fields) fields.Children.Add(Row(field));
            fields.Children.Add(new FieldRow { Label = "Favourite", Content = _favourite });
            fields.Children.Add(new FieldRow { Label = "Times played", Hint = "Mistress's own count; corrected here, it counts on from what is typed.", Content = _playCount });
            fields.Children.Add(new FieldRow { Label = "Play time", Hint = "In seconds, as ES-DE keeps it.", Content = _playTime });

            _scrape = Button("Scrape", "MetadataScrape", () => _ = ScrapeAsync());
            Button save = Button("Save", "MetadataSave", Save);
            save.IsDefault = true;
            Content = Ui.Rows("Auto,*,Auto,Auto",
                Ui.Stack(2,
                    new TextBlock { Name = "MetadataTitle", Text = title, FontSize = 20, FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap },
                    new HintText { Text = Path.GetFileName(path) }),
                new ScrollViewer { Content = fields.Margin(0, 8, 8, 8), HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled },
                _status,
                new ButtonBar
                {
                    ItemsSource = new[]
                    {
                        _scrape, save, Button("Cancel", "MetadataCancel", Close), Button("Clear...", "MetadataClear", () => _ = ClearAsync()),
                        Button(HideText + "...", "MetadataHide", () => _ = HideAsync()),
                    },
                    HorizontalAlignment = HorizontalAlignment.Right,
                }).Margin(16);
            if (Content is Grid grid) grid.RowSpacing = 10;

            if (_host is null) return;
            Draft = _host.DraftFor(path);
            Fill();
            _favourite.IsCheckedChanged += (_, _) => { if (!_filling) Draft.Favourite = _favourite.IsChecked == true; };
            _playCount.TextChanged += (_, _) => { if (!_filling && int.TryParse(_playCount.Text, NumberStyles.None, CultureInfo.InvariantCulture, out int n)) Draft.PlayCount = n; };
            _playTime.TextChanged += (_, _) => { if (!_filling && double.TryParse(_playTime.Text, NumberStyles.None, CultureInfo.InvariantCulture, out double s)) Draft.PlaySeconds = s; };
            _host.ScrapeChanged += OnScrapeChanged;
            Subscribed = true;
            Closed += (_, _) => Stop();
            Opened += (_, _) => (_rows[GameMetadata.Name].Editor as InputElement)?.Focus(NavigationMethod.Directional);
        }

        public string GamePath { get; }

        public MetadataDraft Draft { get; private set; } = null!;

        // Whether the editor still listens to the host's scraping; a closed editor must not.
        public bool Subscribed { get; private set; }

        public string Status => _status.Text ?? "";

        public Control EditorOf(string field) => _rows[field].Editor;

        public Button ResetOf(string field) => _rows[field].Reset;

        public FieldRow RowOf(string field) => _rows[field].Row;

        private static Button Button(string text, string name, Action click)
        {
            Button b = Ui.Button(text, click);
            b.Name = name;
            return b;
        }

        private Control Row(MetadataField field)
        {
            Control editor = field.Kind switch
            {
                MetadataKind.Rating => new RatingPicker { StarSize = 30 },
                MetadataKind.Date => new DateStepper { FontSize = 18 },
                MetadataKind.Flag => new LunaSwitch { Label = field.Label },
                MetadataKind.LongText => new TextBox { AcceptsReturn = true, TextWrapping = TextWrapping.Wrap, Height = 110 },
                _ => new TextBox(),
            };
            editor.Name = "Meta_" + field.Key;
            editor.HorizontalAlignment = field.Kind is MetadataKind.Rating or MetadataKind.Date or MetadataKind.Flag ? HorizontalAlignment.Left : HorizontalAlignment.Stretch;
            Button reset = Button("Reset", "MetaReset_" + field.Key, () => ResetField(field.Key));
            Avalonia.Automation.AutomationProperties.SetName(reset, $"Reset {field.Label}");
            reset.VerticalAlignment = VerticalAlignment.Top;
            var row = new FieldRow { Label = field.Label, Content = Ui.Cols("*,Auto", editor, reset.Margin(8, 0, 0, 0)) };
            _rows[field.Key] = (row, editor, reset);

            switch (editor)
            {
                case TextBox box: box.TextChanged += (_, _) => Changed(field.Key, box.Text ?? ""); break;
                case RatingPicker rating: rating.Chose += v => Changed(field.Key, GameMetadata.FormatRating((float)v)); break;
                case DateStepper date: date.Chose += d => Changed(field.Key, d is { } day ? GameMetadata.FormatDate(day) : ""); break;
                case LunaSwitch flag: flag.IsCheckedChanged += (_, _) => Changed(field.Key, flag.IsChecked == true ? GameMetadata.Yes : GameMetadata.No); break;
            }
            return row;
        }

        private void Changed(string field, string value)
        {
            // A box raises TextChanged after Fill has ended, so a value equal to the draft's is a refill, not a change.
            if (_filling || Draft is null || value == (Draft.Value(field) ?? "")) return;
            Draft.Set(field, value);
            Describe(field);
        }

        private void ResetField(string field)
        {
            Draft.Reset(field);
            Fill();
            (_rows[field].Editor as InputElement)?.Focus(NavigationMethod.Directional);
        }

        // Every control shown from the draft, without the controls' own events writing back into it.
        private void Fill()
        {
            _filling = true;
            try
            {
                foreach ((string field, (FieldRow _, Control editor, Button _)) in _rows)
                {
                    string? value = Draft.Value(field);
                    switch (editor)
                    {
                        case TextBox box: box.Text = value ?? ""; break;
                        case RatingPicker rating: rating.Value = GameMetadata.ParseRating(value) ?? 0; break;
                        case DateStepper date:
                            date.Value = GameMetadata.ParseDate(value);
                            date.StartDate = GameMetadata.ParseDate(Draft.Baseline(field).Value) ?? new DateTime(1990, 1, 1);
                            break;
                        case LunaSwitch flag: flag.IsChecked = value == GameMetadata.Yes; break;
                    }
                    Describe(field);
                }
                _favourite.IsChecked = Draft.Favourite;
                _playCount.Text = Draft.PlayCount.ToString(CultureInfo.InvariantCulture);
                _playTime.Text = Math.Round(Draft.PlaySeconds).ToString(CultureInfo.InvariantCulture);
            }
            finally { _filling = false; }
        }

        // Where the value shown comes from, in the words ES-DE's colours stand for; Reset is offered only where there is something to go back to.
        private void Describe(string field)
        {
            (FieldRow row, Control _, Button reset) = _rows[field];
            MetadataValue baseline = Draft.Baseline(field);
            bool edited = Draft.Source(field) == MetadataSource.Edited;
            row.Hint = Draft.FromScrape(field) ? "From this scrape; Save keeps it."
                : edited ? baseline.Source == MetadataSource.Scraped ? "Your edit, shown in place of ScreenScraper's." : "Your edit."
                : baseline.Source == MetadataSource.Scraped ? "From ScreenScraper."
                : field == GameMetadata.Name ? "From the file name."
                : "";
            reset.IsVisible = edited;
        }

        private void Save()
        {
            _host.SaveMetadata(Draft);
            Close();
        }

        // ES-DE's single-game scrape from the editor: the run is stage (d)'s, started only here, and its answer fills the fields unsaved.
        private async Task ScrapeAsync()
        {
            if (_closed || _waitingForScrape) return;
            if (_host.ScrapeRunning)
            {
                _status.Text = "A scrape is already running; this one can start when it ends.";
                return;
            }
            if (!await _host.ScrapeGameAsync(GamePath) || _closed) return;
            _waitingForScrape = true;
            _scrape.IsEnabled = false;
            _status.Text = "Scraping this game. Its fields fill when ScreenScraper answers.";
            OnScrapeChanged();
        }

        private void OnScrapeChanged()
        {
            if (_closed || !_waitingForScrape || _host.ScrapeRunning) return;
            _waitingForScrape = false;
            _scrape.IsEnabled = true;
            ScrapedRecord? found = _host.ScrapedNow(GamePath);
            if (found is null)
            {
                _status.Text = "ScreenScraper had nothing for this game; the fields are as they were.";
                return;
            }
            Draft.TakeScraped(found);
            Fill();
            _status.Text = "ScreenScraper's answer is in the fields. Save keeps it; Cancel leaves the game as it was.";
        }

        private async Task ClearAsync()
        {
            if (!await Dialogs.ConfirmAsync(this, "Clear Metadata",
                    "Remove your edits to this game, and ScreenScraper's text and pictures for it? Its file, its favourite mark and its play history stay.",
                    "Clear", "Cancel") || _closed) return;
            _host.ClearMetadata(GamePath);
            Close();
        }

        // ES-DE's Delete removes the game's file; Mistress never deletes or moves a file in the ROM folder, so the game is hidden instead.
        private async Task HideAsync()
        {
            if (!await Dialogs.ConfirmAsync(this, HideText,
                    "EmuSen never deletes or moves a game's file, so ES-DE's Delete is not offered. Hide this game from the library instead? Its file stays where it is; " +
                    "turn on Hidden Games in Preferences to list it again.",
                    "Hide", "Cancel") || _closed) return;
            _host.HideGame(GamePath);
            Close();
        }

        // ES-DE asks whether to keep the changes when the editor is left; Y starts the scraper, as ES-DE's editor documents.
        public bool OnPad(UiButton button)
        {
            switch (button)
            {
                case UiButton.Search:
                    _ = ScrapeAsync();
                    return true;
                case UiButton.Back:
                    _ = LeaveAsync();
                    return true;
                default:
                    return false;
            }
        }

        private async Task LeaveAsync()
        {
            if (Draft.IsDirty && await Dialogs.ConfirmAsync(this, "Save Changes", "Keep the changes made to this game?", "Save", "Discard"))
            {
                if (!_closed) Save();
                return;
            }
            Close();
        }

        private void Stop()
        {
            _closed = true;
            if (!Subscribed) return;
            _host.ScrapeChanged -= OnScrapeChanged;
            Subscribed = false;
        }
    }
}
