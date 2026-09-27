using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Media;
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

    // ES-DE's metadata editor for one game, as its user guide documents it, with Hide from Library where ES-DE deletes the file - see EmuSen_Settings_Reference.md §4.59; in a big-screen session ES-DE's layout, §4.72.
    public sealed class MetadataEditorWindow : ToolWindow, IPadDriven
    {
        public const string HideText = "Hide from Library";

        // ES-DE's colours for a value changed in this editing and one its scraper changed, in Mistress's shades (§4.72).
        public static readonly Color EditedColor = Color.FromRgb(0x5C, 0xA4, 0xEC), ScrapedColor = Color.FromRgb(0xE2, 0x62, 0x58);

        private static readonly Color StarColor = Color.FromRgb(0x9C, 0x9C, 0xA0), EmptyStarColor = Color.FromRgb(0x44, 0x44, 0x49);

        private readonly IGameEditorHost _host;
        private readonly bool _big;
        private readonly PadFamily? _family;
        private readonly Dictionary<string, (Control Row, Control Editor, Button Reset)> _rows = new(StringComparer.Ordinal);
        private readonly Dictionary<string, string> _hints = new(StringComparer.Ordinal);
        private readonly Dictionary<string, string?> _opened = new(StringComparer.Ordinal);
        private readonly LunaSwitch _favourite = new() { Name = "Meta_favourite", Label = "Favourite" };
        private readonly TextBox _playCount = new() { Name = "Meta_playcount", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBox _playTime = new() { Name = "Meta_playtime", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBlock _status = new() { Name = "MetadataStatus", TextWrapping = TextWrapping.Wrap };
        private readonly Button _scrape;
        private MenuPanel? _menu;
        private string? _focusedField;
        private bool _filling;
        private bool _waitingForScrape;
        private bool _closed;

        private const string PlayCountHint = "Mistress's own count; corrected here, it counts on from what is typed.", PlayTimeHint = "In seconds, as ES-DE keeps it.";

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public MetadataEditorWindow() : this(null!, "", "") { }

        public MetadataEditorWindow(IGameEditorHost host, string path, string title, PadFamily? menu = null, bool inCustomCollection = false)
        {
            _host = host;
            _big = menu is not null;
            _family = menu;
            GamePath = path;
            InCustomCollection = inCustomCollection;
            Title = "Edit Metadata";
            Width = 820;
            Height = 700;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = false;

            // ES-DE shows the custom collections sortname only inside a custom collection.
            Shown = GameMetadata.Fields.Where(f => f.Key != GameMetadata.CustomSortName || inCustomCollection).ToList();
            foreach (MetadataField field in Shown) Editor(field);

            _scrape = Button("Scrape", "MetadataScrape", () => _ = ScrapeAsync());
            Button save = Button("Save", "MetadataSave", Save);
            save.IsDefault = true;
            Button cancel = Button("Cancel", "MetadataCancel", Close);
            Button clear = Button(_big ? "Clear" : "Clear...", "MetadataClear", () => _ = ClearAsync());
            Button hide = Button(HideText + "...", "MetadataHide", () => _ = HideAsync());
            if (menu is { } family) BuildMenu(title, path, family, [_scrape, save, cancel, clear, hide]);
            else BuildDesktop(title, path, [_scrape, save, cancel, clear, hide]);

            if (_host is null) return;
            Draft = _host.DraftFor(path);
            foreach (MetadataField field in Shown) _opened[field.Key] = Draft.Value(field.Key);
            Fill();
            _favourite.IsCheckedChanged += (_, _) => { if (!_filling) Draft.Favourite = _favourite.IsChecked == true; };
            _playCount.TextChanged += (_, _) => { if (!_filling && int.TryParse(_playCount.Text, NumberStyles.None, CultureInfo.InvariantCulture, out int n)) Draft.PlayCount = n; };
            _playTime.TextChanged += (_, _) => { if (!_filling && double.TryParse(_playTime.Text, NumberStyles.None, CultureInfo.InvariantCulture, out double s)) Draft.PlaySeconds = s; };
            _host.ScrapeChanged += OnScrapeChanged;
            Subscribed = true;
            Closed += (_, _) => Stop();
            Opened += (_, _) => (_rows[GameMetadata.Name].Editor as InputElement)?.Focus(NavigationMethod.Directional);
            // A sheet raises no Opened and focuses its default button when presented; ES-DE's editor opens on its first row.
            if (_menu is { } panel) panel.AttachedToVisualTree += (_, _) => Avalonia.Threading.Dispatcher.UIThread.Post(() => (_rows[GameMetadata.Name].Editor as InputElement)?.Focus(NavigationMethod.Directional));
        }

        public string GamePath { get; }

        // Opened from inside a custom collection, where ES-DE adds the custom collections sortname.
        public bool InCustomCollection { get; }

        // The fields this editor shows, in GameMetadata's order.
        public IReadOnlyList<MetadataField> Shown { get; }

        public MetadataDraft Draft { get; private set; } = null!;

        // Whether the editor still listens to the host's scraping; a closed editor must not.
        public bool Subscribed { get; private set; }

        public string Status => _status.Text ?? "";

        public Control EditorOf(string field) => _rows[field].Editor;

        public Button ResetOf(string field) => _rows[field].Reset;

        // The field's line: a FieldRow on the desktop, the row and its Reset in a big-screen session.
        public Control RowOf(string field) => _rows[field].Row;

        // Where the value shown comes from, in words: a FieldRow's hint on the desktop, the footer while the field has the focus in a big-screen session.
        public string HintOf(string field) => _hints.GetValueOrDefault(field) ?? "";

        public MenuPanel? Menu => _menu;

        private static Button Button(string text, string name, Action click)
        {
            Button b = Ui.Button(text, click);
            b.Name = name;
            return b;
        }

        // One field's editing control and its Reset, wired to the draft; the layouts only place them.
        private void Editor(MetadataField field)
        {
            Control editor = field.Kind switch
            {
                MetadataKind.Rating => new RatingPicker { StarSize = 30 },
                MetadataKind.Date => new DateStepper { FontSize = 18 },
                MetadataKind.Flag => new LunaSwitch { Label = field.Label },
                MetadataKind.Choice => ChoiceBox(field),
                MetadataKind.LongText when !_big => new TextBox { AcceptsReturn = true, TextWrapping = TextWrapping.Wrap, Height = 110 },
                MetadataKind.LongText => new TextBox { AcceptsReturn = true },
                _ => new TextBox(),
            };
            editor.Name = "Meta_" + field.Key;
            editor.HorizontalAlignment = field.Kind is MetadataKind.Rating or MetadataKind.Date or MetadataKind.Flag ? HorizontalAlignment.Left : HorizontalAlignment.Stretch;
            Button reset = Button("Reset", "MetaReset_" + field.Key, () => ResetField(field.Key));
            Avalonia.Automation.AutomationProperties.SetName(reset, $"Reset {field.Label}");
            reset.VerticalAlignment = _big ? VerticalAlignment.Center : VerticalAlignment.Top;
            _rows[field.Key] = (editor, editor, reset);

            switch (editor)
            {
                case TextBox box: box.TextChanged += (_, _) => Changed(field.Key, box.Text ?? ""); break;
                case RatingPicker rating: rating.Chose += v => Changed(field.Key, GameMetadata.FormatRating((float)v)); break;
                case DateStepper date: date.Chose += d => Changed(field.Key, d is { } day ? GameMetadata.FormatDate(day) : ""); break;
                case LunaSwitch flag: flag.IsCheckedChanged += (_, _) => Changed(field.Key, flag.IsChecked == true ? GameMetadata.Yes : GameMetadata.No); break;
                case Dropdown choice:
                    choice.Chose += chosen =>
                    {
                        if (GameMetadata.ChoicesFor(field, GamePath).FirstOrDefault(c => c.Text == chosen as string) is { Text: not null } picked) Changed(field.Key, picked.Value);
                    };
                    break;
            }
        }

        private void BuildDesktop(string title, string path, Button[] buttons)
        {
            var fields = Ui.Stack(10);
            foreach (MetadataField field in Shown)
            {
                (Control _, Control editor, Button reset) = _rows[field.Key];
                Control content = Ui.Cols("*,Auto", editor, reset.Margin(8, 0, 0, 0));
                if (field.Key == GameMetadata.Name) content = Ui.Stack(6, content, NameOffer());
                var row = new FieldRow { Label = field.Label, Content = content };
                _rows[field.Key] = (row, editor, reset);
                fields.Children.Add(row);
            }
            fields.Children.Add(new FieldRow { Label = "Favourite", Content = _favourite });
            fields.Children.Add(new FieldRow { Label = "Times played", Hint = PlayCountHint, Content = _playCount });
            fields.Children.Add(new FieldRow { Label = "Play time", Hint = PlayTimeHint, Content = _playTime });

            Content = Ui.Rows("Auto,*,Auto,Auto",
                Ui.Stack(2,
                    new TextBlock { Name = "MetadataTitle", Text = title, FontSize = 20, FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap },
                    new HintText { Text = Path.GetFileName(path) }),
                new ScrollViewer { Content = fields.Margin(0, 8, 8, 8), HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled },
                _status,
                new ButtonBar { ItemsSource = buttons, HorizontalAlignment = HorizontalAlignment.Right }).Margin(16);
            if (Content is Grid grid) grid.RowSpacing = 10;
        }

        // ES-DE's editor: the title with the game's name and file beneath, a row per field in its guide's order, and its buttons in a row under them.
        private void BuildMenu(string title, string path, PadFamily family, Button[] buttons)
        {
            SheetLayer.SetChromeless(this, true);
            var rows = new StackPanel { Name = "MetadataRows" };
            foreach (MetadataField field in Shown)
            {
                rows.Children.Add(Line(field));
                if (field.Key == GameMetadata.Name) rows.Children.Add(NameOffer());
                if (field.Key == GameMetadata.Players) rows.Children.Add(Natural(MenuRows.Apply(_favourite, "Favourite")));
                if (field.Key == GameMetadata.HideMetadata)
                {
                    rows.Children.Add(Natural(MenuRows.Apply(_playCount, "Times played")));
                    rows.Children.Add(Natural(MenuRows.Apply(_playTime, "Play time")));
                }
            }
            _hints["favourite"] = "";
            _hints["playcount"] = PlayCountHint;
            _hints["playtime"] = PlayTimeHint;
            rows.AddHandler(GotFocusEvent, (_, e) => { _focusedField = FieldOf(e.Source as Control); _statusFresh = false; ShowFooter(); }, handledEventsToo: true);

            var bar = new StackPanel { Name = "MetadataButtons", Orientation = Orientation.Horizontal, Spacing = 8, HorizontalAlignment = HorizontalAlignment.Center };
            foreach (Button b in buttons) bar.Children.Add(MenuRows.ApplyButton(b, 30));
            // A button describes no field, so the footer lets go of the last one.
            bar.AddHandler(GotFocusEvent, (_, _) => { _focusedField = null; _statusFresh = false; ShowFooter(); }, handledEventsToo: true);
            Content = _menu = new MenuPanel
            {
                Name = "MetadataMenu",
                Title = "Edit Metadata",
                Subtitle = title + "\n" + Path.GetFileName(path),
                FooterMaxLines = 2,
                FooterSize = 22,
                HintFamily = family,
                Hints = Hints(),
                Child = new ScrollViewer { Content = rows, VerticalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Hidden },
                Buttons = bar,
            };
            _status.PropertyChanged += (_, e) =>
            {
                if (e.Property != TextBlock.TextProperty) return;
                _statusFresh = true;
                ShowFooter();
            };
        }

        // ES-DE's help in the editor: Y scrapes, A chooses, B leaves, the pad moves and changes.
        private static IReadOnlyList<HintEntry> Hints() =>
        [
            new("Scrape") { Button = PadGlyphButton.North },
            new("Select") { Button = PadHints.Swapped ? PadGlyphButton.East : PadGlyphButton.South },
            new("Back") { Button = PadHints.Swapped ? PadGlyphButton.South : PadGlyphButton.East },
            new("Change") { Button = PadGlyphButton.DPadLeftRight },
            new("Choose") { Button = PadGlyphButton.DPadUpDown },
        ];

        // A field's row with its Reset beside it, shown only while the field holds an edit.
        private Control Line(MetadataField field)
        {
            (Control _, Control editor, Button reset) = _rows[field.Key];
            Control row = editor switch
            {
                TextBox box => MenuRows.Apply(box, field.Label),
                LunaSwitch flag => MenuRows.Apply(flag, field.Label),
                Dropdown choice => MenuRows.Apply(choice, field.Label),
                _ => Hosted(field.Label, editor),
            };
            if (editor is TextBox text) MenuRows.SetValueLetterCase(text, LetterCase.None);
            if (!editor.IsEnabled) row.Opacity = 0.45;
            MenuRows.ApplyButton(reset, 22);
            reset.Margin = new Thickness(6, 0, 8, 0);
            var line = new Grid { ColumnDefinitions = new ColumnDefinitions("*,Auto"), Children = { row, reset } };
            Grid.SetColumn(reset, 1);
            _rows[field.Key] = (line, editor, reset);
            return line;
        }

        // A text row's value as written, not upper-cased: a name or a count reads as typed.
        private static Control Natural(Control row)
        {
            if (row is TextBox box) MenuRows.SetValueLetterCase(box, LetterCase.None);
            return row;
        }

        // Stars and a date keep their own control, placed at the right of a row, drawn in the menu's type and sized with it.
        private static MenuFieldRow Hosted(string label, Control editor)
        {
            switch (editor)
            {
                case RatingPicker stars:
                    stars.Bind(RatingPicker.StarSizeProperty, stars.GetObservable(MenuPanel.ScaleProperty, u => 30 * u));
                    stars.FilledColor ??= StarColor;
                    stars.UnfilledColor = EmptyStarColor;
                    break;
                case DateStepper date:
                    date.IsFramed = false;
                    date.NoDateText = "Unknown";
                    date.Bind(DateStepper.FontSizeProperty, date.GetObservable(MenuPanel.ScaleProperty, u => 36 * u));
                    date.Bind(DateStepper.FontPathProperty, date.GetObservable(MenuPanel.FontPathProperty));
                    date.ForegroundColor ??= StarColor;
                    break;
            }
            editor.FocusAdorner = null;
            return new MenuFieldRow { Label = label, Field = editor };
        }

        // The field a control in the rows belongs to, its editor or its Reset.
        private string? FieldOf(Control? c)
        {
            for (; c is not null; c = c.Parent as Control)
            {
                foreach ((string field, (Control row, Control editor, Button reset)) in _rows)
                    if (ReferenceEquals(c, editor) || ReferenceEquals(c, reset)) return field;
                if (ReferenceEquals(c, _favourite)) return "favourite";
                if (ReferenceEquals(c, _playCount)) return "playcount";
                if (ReferenceEquals(c, _playTime)) return "playtime";
            }
            return null;
        }

        private static string LabelOf(string field) => field switch
        {
            "favourite" => "Favourite",
            "playcount" => "Times played",
            "playtime" => "Play time",
            _ => GameMetadata.Fields.First(f => f.Key == field).Label,
        };

        private bool _statusFresh;

        // The footer says what the editor has just done, until the focus moves; then where the focused field's value comes from.
        private void ShowFooter()
        {
            if (_menu is null) return;
            _menu.Footer = _statusFresh && Status is { Length: > 0 } status ? status
                : _focusedField is { } field ? $"{LabelOf(field)}: {(HintOf(field) is { Length: > 0 } hint ? hint : "Not set.")}"
                : null;
        }

        // A choice with nothing to choose for this game is shown and not enabled, as ES-DE greys out its alternative emulator row.
        private Dropdown ChoiceBox(MetadataField field)
        {
            IReadOnlyList<(string Value, string Text)> choices = GameMetadata.ChoicesFor(field, GamePath);
            var box = new Dropdown { MinWidth = _big ? 0 : 320, IsEnabled = choices.Count > 1 };
            box.Fill(choices.Select(c => c.Text).ToArray(), choices[0].Text);
            return box;
        }

        // ScreenScraper's name as a suggestion under the Name field, taken only by a press - see EmuSen_Settings_Reference.md §4.63.
        private Control NameOffer()
        {
            Button use = Button("Use This Name", "MetadataUseScrapedName", TakeOfferedName);
            Button keep = Button("Keep Current Name", "MetadataKeepName", DeclineOfferedName);
            if (_big)
            {
                MenuRows.Apply(use);
                MenuRows.Apply(keep);
                _useName = use;
                _nameOffer.Spacing = 0;
                _nameOffer.Children.Add(use);
                _nameOffer.Children.Add(keep);
                return _nameOffer;
            }
            _nameOffer.Children.Add(_offeredName);
            _nameOffer.Children.Add(new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, Children = { use, keep } });
            return _nameOffer;
        }

        private readonly StackPanel _nameOffer = new() { Name = "MetadataNameOffer", Spacing = 6, IsVisible = false };
        private readonly TextBlock _offeredName = new() { Name = "MetadataOfferedName", TextWrapping = TextWrapping.Wrap };
        private Button? _useName;

        public string? OfferedName => _nameOffer.IsVisible ? Draft.OfferedName : null;

        private void TakeOfferedName()
        {
            Draft.TakeOfferedName();
            Fill();
            (_rows[GameMetadata.Name].Editor as InputElement)?.Focus(NavigationMethod.Directional);
        }

        private void DeclineOfferedName()
        {
            Draft.DeclineOfferedName();
            ShowNameOffer();
            (_rows[GameMetadata.Name].Editor as InputElement)?.Focus(NavigationMethod.Directional);
        }

        private void ShowNameOffer()
        {
            string? offered = Draft?.OfferedName;
            _nameOffer.IsVisible = offered is not null;
            _offeredName.Text = offered is null ? "" : $"ScreenScraper calls this game “{offered}”.";
            if (_useName is not null)
            {
                MenuRows.SetValue(_useName, offered);
                MenuRows.SetValueLetterCase(_useName, LetterCase.None);
                MenuRows.SetValueColor(_useName, ScrapedColor);
            }
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
                foreach ((string field, (Control _, Control editor, Button _)) in _rows)
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
                        case Dropdown choice:
                            IReadOnlyList<(string Value, string Text)> spec = GameMetadata.ChoicesFor(GameMetadata.Fields.First(f => f.Key == field), GamePath);
                            choice.Fill(spec.Select(c => c.Text).ToArray(), spec.FirstOrDefault(c => c.Value == (value ?? ""), spec[0]).Text);
                            break;
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
            (Control row, Control editor, Button reset) = _rows[field];
            MetadataValue baseline = Draft.Baseline(field);
            bool edited = Draft.Source(field) == MetadataSource.Edited;
            string hint = Draft.FromScrape(field) ? "From this scrape; Save keeps it."
                : edited ? baseline.Source == MetadataSource.Scraped ? "Your edit, shown in place of ScreenScraper's." : "Your edit."
                : baseline.Source == MetadataSource.Scraped ? "From ScreenScraper."
                : field == GameMetadata.Name ? "From the file name."
                : "";
            _hints[field] = hint;
            if (row is FieldRow fieldRow) fieldRow.Hint = hint;
            reset.IsVisible = edited;
            if (_big) Mark(field, editor);
            if (field == GameMetadata.Name) ShowNameOffer();
            if (field == _focusedField) ShowFooter();
        }

        // ES-DE's colours: grey as opened, blue once changed in this editing, red where this editing's scrape put the value.
        private void Mark(string field, Control editor)
        {
            Color? colour = Draft.FromScrape(field) ? ScrapedColor : Draft.Value(field) != _opened.GetValueOrDefault(field, Draft.Value(field)) ? EditedColor : null;
            switch (editor)
            {
                case RatingPicker stars: stars.FilledColor = colour ?? StarColor; break;
                case DateStepper date: date.ForegroundColor = colour ?? StarColor; break;
                default: MenuRows.SetValueColor(editor, colour); break;
            }
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
            _status.Text = "ScreenScraper's answer is in the fields. Save keeps it; Cancel leaves the game as it was."
                + (Draft.OfferedName is null ? "" : " Its name for the game is offered under Name; the name changes only if you use it.");
        }

        private async Task ClearAsync()
        {
            if (!await AskAsync("Clear Metadata",
                    "Remove your edits to this game, and ScreenScraper's text and pictures for it? Its file, its favourite mark and its play history stay.",
                    "Clear", "Cancel") || _closed) return;
            _host.ClearMetadata(GamePath);
            Close();
        }

        // ES-DE's Delete removes the game's file; Mistress never deletes or moves a file in the ROM folder, so the game is hidden instead.
        private async Task HideAsync()
        {
            if (!await AskAsync(HideText,
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
            if (Draft.IsDirty && await AskAsync("Save Changes", "Keep the changes made to this game?", "Save", "Discard"))
            {
                if (!_closed) Save();
                return;
            }
            Close();
        }

        // A question as ES-DE's message box over the editor in a big-screen session, a dialog on the desktop; either way its accepting button is focused first.
        private Task<bool> AskAsync(string title, string message, string accept, string cancel) =>
            _family is { } family
                ? Dialogs.MenuConfirmAsync(this, message, accept, cancel, family,
                    [new("Select") { Button = PadHints.Swapped ? PadGlyphButton.East : PadGlyphButton.South }, new("Choose") { Button = PadGlyphButton.DPadLeftRight }])
                : Dialogs.ConfirmAsync(this, title, message, accept, cancel);

        private void Stop()
        {
            _closed = true;
            if (!Subscribed) return;
            _host.ScrapeChanged -= OnScrapeChanged;
            Subscribed = false;
        }
    }
}
