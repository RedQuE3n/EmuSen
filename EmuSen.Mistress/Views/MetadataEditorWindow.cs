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
        private Control? _focusedControl;
        private bool _filling;
        private bool _waitingForScrape;
        private bool _closed;

        // Design sizes at 800 lines, from ES-DE's editor: a 42-pixel row with 19-pixel capitals, 26-pixel capitals on its buttons, and stars 20 pixels a piece (§34.12).
        public const double RowPitch = 42, RowText = 27, ButtonText = 37, StarSize = 21;

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

        // A choice field's values as this editor shows them: a big-screen row's short names, the desktop's full ones (§4.72).
        public IReadOnlyList<(string Value, string Text)> ChoicesOf(string field)
        {
            MetadataField spec = GameMetadata.Fields.First(f => f.Key == field);
            IReadOnlyList<(string Value, string Text)> choices = GameMetadata.ChoicesFor(spec, GamePath);
            return _big ? choices.Select(c => (c.Value, GameMetadata.ShortText(spec, c))).ToList() : choices;
        }

        // Whether the field holds an edit that Reset (Reset's button, or West in a big-screen session) would remove.
        public bool CanReset(string field) => _rows.TryGetValue(field, out var row) && row.Reset.IsVisible;

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
                        if (ChoicesOf(field.Key).FirstOrDefault(c => c.Text == chosen as string) is { Text: not null } picked) Changed(field.Key, picked.Value);
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
            // ES-DE's editor draws its rows smaller than its menus', measured on its own editor at 1280 by 800 (§34.12).
            rows.Styles.Add(new Avalonia.Styling.Style(x => Avalonia.Styling.Selectors.OfType<MenuRow>(x))
            {
                Setters = { new Avalonia.Styling.Setter(MenuRow.RowHeightProperty, RowPitch), new Avalonia.Styling.Setter(MenuRow.TextSizeProperty, RowText) },
            });
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
            rows.AddHandler(GotFocusEvent, (_, e) => { _focusedControl = e.Source as Control; _focusedField = FieldOf(_focusedControl); _statusFresh = false; ShowFooter(); }, handledEventsToo: true);

            var bar = new StackPanel { Name = "MetadataButtons", Orientation = Orientation.Horizontal, Spacing = 8, HorizontalAlignment = HorizontalAlignment.Center };
            foreach (Button b in buttons) bar.Children.Add(MenuRows.ApplyButton(b, ButtonText));
            // A button describes no field, so the footer lets go of the last one.
            bar.AddHandler(GotFocusEvent, (_, e) => { _focusedControl = e.Source as Control; _focusedField = null; _statusFresh = false; ShowFooter(); }, handledEventsToo: true);
            Content = _menu = new MenuPanel
            {
                Name = "MetadataMenu",
                RowPitch = RowPitch,
                Title = "Edit Metadata",
                Subtitle = SubtitleOf(path),
                SubtitleLetterCase = LetterCase.None,
                FooterMaxLines = 2,
                FooterSize = 22,
                HintFamily = family,
                Hints = Hints(_hinted = new RowHelp("Select", Sideways: false, Reset: false, OnButtons: false)),
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

        // ES-DE's one line under the editor's title: the file's name and its system in brackets (Q105).
        public static string SubtitleOf(string path) =>
            EmuSen.Cores.CoreCatalog.ShelfByName(new EmuSen.Mistress.Library.RomEntry(path).Shelf)?.EsdeSystem is { Length: > 0 } system
                ? $"{Path.GetFileName(path)} [{system.ToUpperInvariant()}]"
                : Path.GetFileName(path);

        // What the help bar says for the focused row: A's word, whether Left and Right change it, whether X resets it, and whether the focus is on the buttons.
        private sealed record RowHelp(string Accept, bool Sideways, bool Reset, bool OnButtons);

        private RowHelp? _hinted;

        // ES-DE's help in the editor, in its order, A's word following the focused row as ES-DE's does (Q106, §4.79).
        private static IReadOnlyList<HintEntry> Hints(RowHelp help) =>
        [
            new(help.Accept) { Button = PadHints.Glyph(PadGlyphButton.South) },
            new("Back") { Button = PadHints.Glyph(PadGlyphButton.East) },
            new("Scrape") { Button = PadHints.Glyph(PadGlyphButton.North) },
            .. help.Reset ? [new HintEntry("Reset") { Button = PadHints.Glyph(PadGlyphButton.West) }] : Array.Empty<HintEntry>(),
            .. help.Sideways ? [new HintEntry("Change") { Button = PadGlyphButton.DPadLeftRight }] : Array.Empty<HintEntry>(),
            new("Choose") { Button = help.OnButtons ? PadGlyphButton.DPadLeftRight : PadGlyphButton.DPadUpDown },
        ];

        // A's word for a control, in ES-DE's words where Mistress's action is ES-DE's, measured on its editor (§40.3).
        internal static string AcceptWord(Control? focused) => focused switch
        {
            RatingPicker => "Add Half Star",
            DateStepper => "Edit Date",
            ToggleSwitch => "Toggle",
            Button { Name: "MetadataScrape" } => "Scrape",
            Button { Name: "MetadataSave" } => "Save Metadata",
            Button { Name: "MetadataCancel" } => "Cancel Changes",
            Button { Name: "MetadataClear" } => "Clear Metadata",
            Button { Name: "MetadataHide" } => "Hide Game",
            _ => "Select",
        };

        // The help the focused control asks for now.
        private RowHelp HelpNow()
        {
            bool onButtons = _focusedControl is Button b && b.Parent is StackPanel { Name: "MetadataButtons" };
            bool reset = _focusedField is { } focused && CanReset(focused);
            return new RowHelp(AcceptWord(_focusedControl), _focusedControl is RatingPicker or DateStepper or ComboBox, reset, onButtons);
        }

        // A field's row; its Reset is the pad's West, not a button beside it.
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
            // ES-DE's editor shows "unknown" for these fields when they are empty, and nothing for the others (§34.12).
            if (editor is TextBox unset && field.Key is GameMetadata.Developer or GameMetadata.Publisher or GameMetadata.Genre or GameMetadata.Players) unset.PlaceholderText = "unknown";
            if (!editor.IsEnabled) row.Opacity = 0.45;
            // Reset is West on the pad and Delete on the keyboard here, named in the help bar while the focused field holds an edit (Q101).
            _rows[field.Key] = (row, editor, reset);
            return row;
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
                    stars.Bind(RatingPicker.StarSizeProperty, stars.GetObservable(MenuPanel.ScaleProperty, u => StarSize * u));
                    stars.FilledColor ??= StarColor;
                    stars.UnfilledColor = EmptyStarColor;
                    break;
                case DateStepper date:
                    date.IsFramed = false;
                    date.NoDateText = "unknown";
                    date.Bind(DateStepper.FontSizeProperty, date.GetObservable(MenuPanel.ScaleProperty, u => RowText * u));
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
            RowHelp help = HelpNow();
            if (help != _hinted) _menu.Hints = Hints(_hinted = help);
        }

        // A choice with nothing to choose for this game is shown and not enabled, as ES-DE greys out its alternative emulator row.
        private Dropdown ChoiceBox(MetadataField field)
        {
            IReadOnlyList<(string Value, string Text)> choices = ChoicesOf(field.Key);
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
                            IReadOnlyList<(string Value, string Text)> spec = ChoicesOf(field);
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
                    "EmuSen never deletes or moves a game's file. Hide this game from the library instead? Its file stays where it is; " +
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
                // X, or Y with the swap (Delete on the keyboard), resets the focused field in a big-screen session (Q101, §4.79.5).
                case UiButton.Screensaver when _big:
                    if (_focusedField is { } field && CanReset(field)) ResetField(field);
                    return true;
                case UiButton.Back:
                    _ = LeaveAsync();
                    return true;
                // A on the stars adds half a star, and past five starts again from none, as ES-DE's Add Half Star (§40.3).
                case UiButton.Accept when _big && _focusedControl is RatingPicker { IsFocused: true } stars:
                    PadWindowRouter.Key(stars, stars.Value >= 0.999 ? Avalonia.Input.Key.Home : Avalonia.Input.Key.Right);
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
                    [new("Select") { Button = PadHints.Glyph(PadGlyphButton.South) }, new("Choose") { Button = PadGlyphButton.DPadLeftRight }])
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
