using System;
using System.Collections.Generic;
using System.Collections.Specialized;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Templates;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // A desktop settings sheet drawn as ES-DE's menus in a big-screen session: its controls, moved out of their field rows into menu rows, pages as submenus - see EmuSen_Settings_Reference.md §4.72.8.
    public sealed class BigMenuForm
    {
        // Set on a field whose hint is read letter for letter, such as a folder's path: its footer keeps its case whatever case the look draws words in.
        public static readonly AttachedProperty<bool> VerbatimHintProperty = AvaloniaProperty.RegisterAttached<FieldRow, bool>("VerbatimHint", typeof(BigMenuForm));

        // Set on a page whose rows take no choice: its hints are only moving and going back, since A and the sides do nothing there.
        public static readonly AttachedProperty<bool> ReadOnlyPageProperty = AvaloniaProperty.RegisterAttached<Control, bool>("ReadOnlyPage", typeof(BigMenuForm));

        // One screen of the form: its title, what its rows are made from each time it is shown, and whether it takes no choice.
        private sealed record Page(string Title, Func<IReadOnlyList<Control>> Rows, bool ReadOnly = false);

        private readonly ToolWindow _window;
        private readonly string _title;
        private readonly TabControl? _tabs;
        private readonly IReadOnlyList<(string Header, Control Content)> _pages;
        private readonly bool _firstInline;
        private readonly StackPanel _rows = new() { Name = "BigMenuRows" };
        // Rows of the screens not shown, kept in the tree unseen, so what finds a control by its name still does.
        private readonly StackPanel _held = new() { Name = "BigMenuHeld", IsVisible = false };
        private readonly Stack<Page> _stack = new();
        private readonly Dictionary<Control, List<Control>> _converted = new();
        private readonly Dictionary<Control, string> _hints = new();
        private readonly HashSet<Control> _verbatim = new();
        private readonly Dictionary<Control, List<Control>> _extras = new();
        private List<Control> _pending = new();
        private readonly Dictionary<int, Button> _pageRows = new();
        private readonly HashSet<Panel> _watched = new();
        private readonly Button _back;
        private readonly IReadOnlyList<HintEntry> _choiceHints = Hints(), _readOnlyHints = Hints(readOnly: true);
        private bool _rebuildQueued;

        // The window's pages, the first shown inline above the others' submenu rows when firstInline, as ES-DE's UI settings put the theme's options above its submenus.
        public BigMenuForm(ToolWindow window, string title, PadFamily family, IReadOnlyList<(string Header, Control Content)> pages, TabControl? tabs = null, bool firstInline = false, IReadOnlyList<Control>? keep = null)
        {
            _window = window;
            _title = title;
            _pages = pages;
            _tabs = tabs;
            _firstInline = firstInline;
            SheetLayer.SetChromeless(window, true);

            _back = MenuRows.ApplyButton(new Button { Name = "BigMenuBack", Content = "Back" }, 34);
            _back.Click += (_, _) => Back();
            Menu = new MenuPanel
            {
                Name = "BigMenu",
                Title = title,
                FooterMaxLines = 2,
                FooterSize = 22,
                HintFamily = family,
                Hints = _choiceHints,
                Child = new ScrollViewer { Content = _rows, VerticalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Hidden },
                Buttons = new StackPanel { Orientation = Orientation.Horizontal, HorizontalAlignment = HorizontalAlignment.Center, Children = { _back } },
            };
            _rows.AddHandler(InputElement.GotFocusEvent, (_, e) => ShowHint(e.Source as Control), handledEventsToo: true);
            _back.GotFocus += (_, _) => Menu.Footer = null;

            // The window's own model controls stay in the tree, unseen, so what reads them still finds them.
            var host = new Grid();
            if (tabs is not null)
            {
                tabs.IsVisible = false;
                Detach(tabs);
                host.Children.Add(tabs);
                tabs.SelectionChanged += (_, _) => { if (!_turning) Turn(tabs.SelectedIndex); };
            }
            foreach (Control c in keep ?? [])
            {
                Detach(c);
                c.IsVisible = false;
                host.Children.Add(c);
            }
            host.Children.Add(_held);
            host.Children.Add(Menu);
            window.Content = host;

            Root();
            if (tabs is { SelectedIndex: > 0 }) Turn(tabs.SelectedIndex);
            Menu.AttachedToVisualTree += (_, _) => Dispatcher.UIThread.Post(FocusFirst);
        }

        public MenuPanel Menu { get; }

        // The title of the screen shown, the form's own on its first screen.
        public string Shown => _stack.Peek().Title;

        // How deep the screen shown is: 0 on the first.
        public int Depth => _stack.Count - 1;

        public string? HintOf(Control row) => _hints.GetValueOrDefault(row);

        private static IReadOnlyList<HintEntry> Hints(bool readOnly = false) => readOnly
            ?
            [
                new("Back") { Button = PadHints.Glyph(PadGlyphButton.East) },
                new("Choose") { Button = PadGlyphButton.DPadUpDown },
            ]
            :
            [
                new("Select") { Button = PadHints.Glyph(PadGlyphButton.South) },
                new("Back") { Button = PadHints.Glyph(PadGlyphButton.East) },
                new("Change") { Button = PadGlyphButton.DPadLeftRight },
                new("Choose") { Button = PadGlyphButton.DPadUpDown },
            ];

        // B and Back: a submenu goes back to the screen it came from, the first screen closes the sheet.
        public bool Back()
        {
            if (_stack.Count > 1)
            {
                _stack.Pop();
                Show(focusFirst: true);
                return true;
            }
            _window.Close();
            return true;
        }

        private void Root()
        {
            _stack.Clear();
            _stack.Push(new Page(_title, RootRows, _pages.Count == 1 && _pages[0].Content.GetValue(ReadOnlyPageProperty)));
            Show(focusFirst: false);
        }

        private IReadOnlyList<Control> RootRows()
        {
            var rows = new List<Control>();
            if (_pages.Count == 1) return PageRows(_pages[0].Content);
            for (int i = 0; i < _pages.Count; i++)
            {
                if (i == 0 && _firstInline) { rows.AddRange(PageRows(_pages[0].Content)); continue; }
                int index = i;
                if (!_pageRows.TryGetValue(i, out Button? row)) _pageRows[i] = row = SubmenuRow($"BigMenuPage{i}", _pages[i].Header, () => Turn(index));
                rows.Add(row);
            }
            return rows;
        }

        private bool _turning;

        // The page of a tab, as its submenu; with the first page inline, turning to it is the first screen.
        public void Turn(int index)
        {
            if (index < 0 || index >= _pages.Count) return;
            if (_tabs is not null && _tabs.SelectedIndex != index)
            {
                _turning = true;
                _tabs.SelectedIndex = index;
                _turning = false;
            }
            while (_stack.Count > 1) _stack.Pop();
            if (index == 0 && (_firstInline || _pages.Count == 1))
            {
                Show(focusFirst: true);
                return;
            }
            Push(_pages[index].Header, () => PageRows(_pages[index].Content), _pages[index].Content.GetValue(ReadOnlyPageProperty));
        }

        private void Push(string title, Func<IReadOnlyList<Control>> rows, bool readOnly = false)
        {
            _stack.Push(new Page(title, rows, readOnly));
            Show(focusFirst: true);
        }

        private void Show(bool focusFirst)
        {
            Page page = _stack.Peek();
            Menu.Title = page.Title;
            Menu.Hints = page.ReadOnly ? _readOnlyHints : _choiceHints;
            foreach (Control row in _rows.Children.ToList())
            {
                _rows.Children.Remove(row);
                if (Known(row)) _held.Children.Add(row);
            }
            foreach (Control row in page.Rows())
            {
                Detach(row);
                _rows.Children.Add(row);
            }
            Menu.Footer = null;
            if (focusFirst) Dispatcher.UIThread.Post(FocusFirst);
            if (TopLevel.GetTopLevel(Menu) is MainWindow main && ReferenceEquals(main.MenuShownNow, Menu) && _stack.Count > 1) main.BeginMenuOpening(Menu);
        }

        // A row some screen can still show; one whose source its page's code replaced is let go.
        private bool Known(Control row) =>
            _pageRows.ContainsValue((row as Button)!) || _converted.Values.Any(l => l.Contains(row)) || _extras.Values.Any(l => l.Contains(row));

        private void FocusFirst()
        {
            InputElement? first = _rows.Children.OfType<InputElement>().FirstOrDefault(c => c.Focusable && c.IsEffectivelyEnabled && c.IsEffectivelyVisible)
                ?? _rows.GetLogicalDescendants().OfType<InputElement>().FirstOrDefault(c => c.Focusable && c.IsEffectivelyEnabled && c.IsEffectivelyVisible);
            (first ?? _back).Focus(NavigationMethod.Directional);
        }

        private void ShowHint(Control? source)
        {
            for (Control? c = source; c is not null && !ReferenceEquals(c, _rows); c = c.Parent as Control)
                if (_hints.TryGetValue(c, out string? hint))
                {
                    Menu.FooterLetterCase = _verbatim.Contains(c) ? EmuSen.LunaP.Media.LetterCase.None : null;
                    Menu.Footer = hint.Length > 0 ? hint : null;
                    return;
                }
            Menu.Footer = null;
        }

        // A source page's rows, each field row turned into menu rows once, rebuilt only for what the page's own code replaced.
        private IReadOnlyList<Control> PageRows(Control content)
        {
            var rows = new List<Control>();
            foreach (Control source in Flatten(content))
            {
                if (!_converted.TryGetValue(source, out List<Control>? made))
                {
                    _converted[source] = made = Convert(source).ToList();
                    _extras[source] = _pending;
                    _pending = new();
                }
                rows.AddRange(made);
            }
            return rows;
        }

        // A page's parts in order, looking through its scroller and the stacks that only hold field rows.
        private IEnumerable<Control> Flatten(Control content)
        {
            if (content is ScrollViewer { Content: Control inner }) { foreach (Control c in Flatten(inner)) yield return c; yield break; }
            if (content is StackPanel stack && stack.Children.Any(c => c is FieldRow or StackPanel or SectionHeader or EmptyState))
            {
                Watch(stack);
                foreach (Control c in stack.Children.ToList()) foreach (Control d in Flatten(c)) yield return d;
                yield break;
            }
            yield return content;
        }

        // A page whose code rebuilds its rows (a theme downloaded, a collection created) is shown again from them.
        private void Watch(Panel panel)
        {
            if (!_watched.Add(panel)) return;
            panel.Children.CollectionChanged += (_, e) =>
            {
                if (e.OldItems is not null)
                    foreach (Control gone in e.OldItems.OfType<Control>())
                    {
                        if (_converted.Remove(gone, out List<Control>? rows)) foreach (Control r in rows) _held.Children.Remove(r);
                        if (_extras.Remove(gone, out List<Control>? extra)) foreach (Control r in extra) _held.Children.Remove(r);
                    }
                if (_rebuildQueued) return;
                _rebuildQueued = true;
                Dispatcher.UIThread.Post(() =>
                {
                    _rebuildQueued = false;
                    Control? focused = TopLevel.GetTopLevel(Menu)?.FocusManager?.GetFocusedElement() as Control;
                    int at = focused is null ? -1 : _rows.Children.IndexOf(RowOf(focused) ?? focused);
                    Show(focusFirst: false);
                    if (at >= 0 && _rows.Children.Count > 0 && (focused is null || !Avalonia.VisualTree.VisualExtensions.IsAttachedToVisualTree(focused)))
                        (_rows.Children[Math.Min(at, _rows.Children.Count - 1)] as InputElement)?.Focus(NavigationMethod.Directional);
                });
            };
        }

        private Control? RowOf(Control c)
        {
            for (Control? r = c; r is not null; r = r.Parent as Control)
                if (ReferenceEquals(r.Parent, _rows)) return r;
            return null;
        }

        // One part of a page as menu rows: a field row by what it holds, a header or a note as a line of its own, anything else kept whole beside its label.
        private IEnumerable<Control> Convert(Control source)
        {
            switch (source)
            {
                case SectionHeader:
                    yield break;
                case FieldRow field:
                {
                    Control? content = field.Content as Control;
                    field.Content = null;
                    foreach (Control row in Content(content, field.Label ?? "", field.Hint ?? ""))
                    {
                        if (field.GetValue(VerbatimHintProperty)) _verbatim.Add(row);
                        yield return row;
                    }
                    yield break;
                }
                case EmptyState empty:
                    yield return Note(empty.Message ?? "", empty.Detail ?? "");
                    yield break;
                case TextBlock status:
                    // A status line, such as a download's progress: shown in the footer while it has something to say.
                    status.PropertyChanged += (_, e) => { if (e.Property == TextBlock.TextProperty && status.Text is { Length: > 0 } t) Menu.Footer = t; };
                    Detach(status);
                    status.IsVisible = false;
                    yield return status;
                    yield break;
                default:
                    foreach (Control row in Content(source, "", "")) yield return row;
                    yield break;
            }
        }

        private IEnumerable<Control> Content(Control? content, string label, string hint, bool whole = true)
        {
            switch (content)
            {
                case null:
                    yield break;
                case ComboBox choice:
                    Detach(choice);
                    yield return Hinted(MenuRows.Apply(choice, label), hint);
                    yield break;
                case ToggleSwitch toggle:
                    Detach(toggle);
                    // A switch's own words are ES-DE's ("Display clock"); the field's name ("Clock") stands in only where it has none.
                    string text = toggle is LunaSwitch { Label: { Length: > 0 } own } ? own : label;
                    yield return Hinted(MenuRows.Apply(toggle, text), hint);
                    yield break;
                case TextBox box:
                    Detach(box);
                    MenuRows.SetValueLetterCase(box, EmuSen.LunaP.Media.LetterCase.None);
                    // A box among others, such as a user name's, is named by its own placeholder.
                    string boxLabel = label.Length > 0 ? label : box.PlaceholderText ?? Avalonia.Automation.AutomationProperties.GetName(box) ?? "";
                    if (label.Length == 0) box.PlaceholderText = null;
                    yield return Hinted(MenuRows.Apply(box, boxLabel), hint);
                    yield break;
                case PathPickerRow picker:
                    Detach(picker);
                    yield return Hinted(PathRow(picker, label), hint);
                    yield break;
                case Slider slider:
                    Detach(slider);
                    yield return Hinted(MenuRows.Apply(slider, label), hint);
                    yield break;
                case Button button:
                {
                    Detach(button);
                    string words = button.Content as string ?? label;
                    bool opens = words.EndsWith('…') || words.EndsWith("...", StringComparison.Ordinal);
                    // A field's one button: the field's name as the label, and what the button says as the value unless it opens a screen.
                    bool named = label.Length > 0 && !string.Equals(Plain(label), Plain(words), StringComparison.OrdinalIgnoreCase);
                    MenuRows.Apply(button, opens ? MenuRowKind.Submenu : MenuRowKind.Action, named && !opens ? Plain(words) : null);
                    MenuRows.SetLabel(button, Plain(named ? label : words));
                    yield return Hinted(button, hint);
                    yield break;
                }
                case Panel panel:
                {
                    List<Control> children = panel.Children.ToList();
                    // A slider with its value's words beside it: one option row that shows those words.
                    if (children.OfType<Slider>().SingleOrDefault() is { } slid && children.OfType<TextBlock>().SingleOrDefault() is { } words2)
                    {
                        Detach(slid);
                        MenuRows.SetValue(slid, words2.Text);
                        words2.PropertyChanged += (_, e) => { if (e.Property == TextBlock.TextProperty) MenuRows.SetValue(slid, words2.Text); };
                        yield return Hinted(MenuRows.Apply(slid, label), hint);
                        yield break;
                    }
                    // Two or more buttons for one thing, such as a theme's Use, Update, Remove and About: a submenu of them.
                    if (whole && children.Count > 1 && children.All(c => c is Button and not Avalonia.Controls.Primitives.ToggleButton))
                    {
                        foreach (Button b in children.OfType<Button>()) Detach(b);
                        IReadOnlyList<Control> made = children.OfType<Button>().SelectMany(b => Content(b, "", "")).ToList();
                        foreach (Control m in made) _held.Children.Add(m);
                        _pending.AddRange(made);
                        string state = children.OfType<Button>().FirstOrDefault(b => !b.IsEnabled)?.Content as string ?? "";
                        yield return Hinted(SubmenuRow($"BigMenuSub_{label}", label, () => Push(label, () => made), state), hint);
                        yield break;
                    }
                    foreach (Control child in children)
                    {
                        Detach(child);
                        foreach (Control row in Content(child, children.Count == 1 || child is ComboBox ? label : "", hint, whole: false)) yield return row;
                    }
                    yield break;
                }
                case EmptyState empty:
                    yield return Hinted(Note(label.Length > 0 ? $"{label}: {empty.Message}" : empty.Message ?? "", empty.Detail ?? ""), hint.Length > 0 ? hint : empty.Detail ?? "");
                    yield break;
                case TextBlock info:
                {
                    // Words the page's code changes, such as a sign-in's answer: a line that follows them, and the footer while they are new.
                    Detach(info);
                    info.IsVisible = false;
                    _held.Children.Add(info);
                    _pending.Add(info);
                    var note = (MenuRow)Note(info.Text ?? "", "");
                    note.Bind(MenuRow.LabelProperty, info.GetObservable(TextBlock.TextProperty));
                    note.Bind(Visual.IsVisibleProperty, info.GetObservable(TextBlock.TextProperty, t => !string.IsNullOrEmpty(t)));
                    info.PropertyChanged += (_, e) => { if (e.Property == TextBlock.TextProperty && info.Text is { Length: > 0 } t) Menu.Footer = t; };
                    yield return Hinted(note, hint);
                    yield break;
                }
                default:
                    Detach(content);
                    yield return Hinted(new MenuFieldRow { Label = label, Field = content }, hint);
                    yield break;
            }
        }

        private static string Plain(string words) => words.TrimEnd('.', '…').Trim();

        private Control Hinted(Control row, string hint)
        {
            _hints[row] = hint;
            return row;
        }

        private static Button SubmenuRow(string name, string label, Action open, string? value = null)
        {
            Button row = MenuRows.Apply(new Button { Name = name, Content = label }, MenuRowKind.Submenu, value is { Length: > 0 } ? value : null);
            row.Click += (_, _) => open();
            return row;
        }

        // Words that are not a setting: a row nobody lands on.
        private static Control Note(string message, string detail)
        {
            var row = new MenuRow { Label = message, Value = null, Kind = MenuRowKind.Action, Focusable = false };
            if (detail.Length > 0) ToolTip.SetTip(row, detail);
            return row;
        }

        // A path picker as a text row: its box drawn as the row, typed into by the on-screen keyboard and committed as the picker commits a typed path; the platform's browser is left out.
        private static PathPickerRow PathRow(PathPickerRow picker, string label)
        {
            picker.Template = new FuncControlTemplate<PathPickerRow>((p, scope) =>
            {
                var box = new TextBox { Name = "PART_Path", PlaceholderText = p.Placeholder };
                box.Bind(TextBox.TextProperty, p.GetObservable(PathPickerRow.PathProperty));
                MenuRows.SetValueLetterCase(box, EmuSen.LunaP.Media.LetterCase.None);
                MenuRows.Apply(box, label);
                var browse = new Button { Name = "PART_Browse", IsVisible = false };
                box.RegisterInNameScope(scope);
                browse.RegisterInNameScope(scope);
                return new Panel { Children = { box, browse } };
            });
            picker.HorizontalAlignment = HorizontalAlignment.Stretch;
            return picker;
        }

        private static void Detach(Control c)
        {
            switch (c.Parent)
            {
                case Panel p: p.Children.Remove(c); break;
                case ContentControl cc when ReferenceEquals(cc.Content, c): cc.Content = null; break;
                case Decorator d when ReferenceEquals(d.Child, c): d.Child = null; break;
                case Avalonia.Controls.Presenters.ContentPresenter cp when ReferenceEquals(cp.Content, c): cp.Content = null; break;
            }
        }
    }
}
