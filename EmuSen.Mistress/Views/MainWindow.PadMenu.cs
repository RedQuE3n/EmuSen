using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Media.TextFormatting;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // The pad's menu as pages: the game's menu or the main menu, the submenus under them, and a yes-or-no before anything that ends a game - see EmuSen_Settings_Reference.md §4.69.8.
    public partial class MainWindow
    {
        public const string MainMenuTitle = "Main Menu", GameSettingsMenu = "Game Settings", LibraryMenu = "Library", SettingsMenu = "Settings", EmuSenMenu = "EmuSen";
        public const string QuestionNo = "No", QuestionYes = "Yes";

        // One page of the menu: its title (null for the first, whose title is the game's or Main Menu), its rows, the row last on, and for a question the sentence under it.
        private sealed class PadMenuPage(string? title, Func<List<PadMenuEntry>> rows, string? detail = null, bool question = false)
        {
            public string? Title { get; } = title;
            public Func<List<PadMenuEntry>> Rows { get; } = rows;
            public string? Detail { get; } = detail;
            public bool Question { get; } = question;
            public int At { get; set; }
        }

        private readonly List<PadMenuPage> _padMenuPages = new();
        private readonly Dictionary<string, int> _padSubmenuRows = new(StringComparer.Ordinal);
        private bool _padMenuInGame;

        // The divider between two sections in the big panel: the last row's rule drawn lighter than the rows' own.
        private static readonly Color SectionRule = Color.FromRgb(0x70, 0x70, 0x76);
        private const string SectionEndClass = "pad-section-end";

        private void OpenPadMenu()
        {
            _padMenuInGame = GameOnScreen;
            _padMenuPages.Clear();
            _padMenuPages.Add(new PadMenuPage(null, _padMenuInGame ? GameMenuRows : MainMenuRows));

            // Before the menu shows, so no frame runs under it - the library's own rule (§4.18).
            _padMenuPaused = _padMenuInGame && _pauseSignal.IsSet;
            if (_padMenuPaused) PauseEmulation();

            PadMenuPanel.IsVisible = true;
            _padMenuOpen = true;
            ShowPadMenuPage(turned: false);
            UpdateMenuBackdrop();
        }

        private List<PadMenuEntry> GameMenuRows() =>
        [
            new(() => "Resume", () => { }),
            new(RewindMenuText, RewindFromPadMenu, closes: false) { Label = () => "Rewind", Value = RewindUnavailable },
            new(() => $"Save State      <  {SlotValue()}  >", SaveState, StepStateSlot) { Label = () => "Save State", Value = SlotValue, ShowsSlot = true },
            new(() => $"Load State      <  {SlotValue()}  >", LoadState, StepStateSlot) { Label = () => "Load State", Value = SlotValue, ShowsSlot = true },
            new(() => $"Speed      <  {DescribeSpeed(_baseSpeedPercent)}  >", () => StepSpeed(1), StepSpeed, closes: false) { Label = () => "Speed", Value = () => DescribeSpeed(_baseSpeedPercent) },
            Ask("Restart Game...", "Restart Game?", "The game starts again from the beginning. Progress since your last save is lost.", ResetEmulation),
            Submenu(GameSettingsMenu, GameSettingsRows, startsSection: true),
            new(() => "Back to Library", ToggleLibrary) { StartsSection = true },
            Ask("Quit Game...", "Quit Game?", "The game closes and the library comes back.", ShowLibrary),
            Submenu(EmuSenMenu, () => EmuSenRows(inGame: true)),
        ];

        private List<PadMenuEntry> MainMenuRows()
        {
            var rows = new List<PadMenuEntry>();
            if (_session is { IsRomLoaded: true }) rows.Add(new(() => $"Back to {RunningGameTitle()}", ToggleLibrary));
            // The sidebar library's game options; the themed gamelist has them on Select - see EmuSen_Settings_Reference.md §4.63.
            if (!ThemedLibraryShown && SelectedLibraryEntry is EmuSen.Mistress.Library.RomEntry chosen)
                rows.Add(Opens("Game Options...", () => ShowLibraryGameOptions(chosen)));
            // Scraping only ever starts here or in Preferences, by the player - see EmuSen_Settings_Reference.md §4.60.
            if (GameToScrape is string game && !ScrapeRunning)
                rows.Add(Opens("Scrape This Game...", () => _ = ConfirmAndScrapeAsync(Scraping.ScrapeScope.ThisGame(game))));
            rows.Add(Submenu(LibraryMenu, LibraryRows, startsSection: rows.Count > 0));
            rows.Add(Submenu(SettingsMenu, SettingsRows));
            rows.Add(Submenu(EmuSenMenu, () => EmuSenRows(inGame: false)));
            return rows;
        }

        private List<PadMenuEntry> GameSettingsRows() =>
        [
            Opens("Cheats", ShowActiveCheats),
            Opens("Players & Controllers", () => ShowPreferencesAt(PreferencesWindow.ControllersTab)),
            Opens("Controller Bindings", ShowControllerBindings),
            Opens("Graphics", ShowGraphicsSettings),
            Opens("Shaders", ShowShaderSettings),
        ];

        private List<PadMenuEntry> LibraryRows()
        {
            var rows = new List<PadMenuEntry>();
            AddCollectionMenuEntries(rows);
            rows.Add(new(() => ScrapeRunning ? $"Scraping ({_scrapeRun!.Done} of {_scrapeRun.Total})..." : "Scrape Games...", () => { if (ScrapeRunning) ShowScrapeStatus(); else ShowPreferencesAt(PreferencesWindow.ScrapingTab); }) { Opens = true });
            if (_bigScreen) rows.Add(Opens("Theme Settings", ShowThemeSettings));
            return rows;
        }

        private List<PadMenuEntry> SettingsRows() =>
        [
            Opens("Players & Controllers", () => ShowPreferencesAt(PreferencesWindow.ControllersTab)),
            Opens("Controller Bindings", ShowControllerBindings),
            Opens("Graphics", ShowGraphicsSettings),
            Opens("Shaders", ShowShaderSettings),
            Opens("Cheats", ShowActiveCheats),
            Opens(PreferencesWindow.FirmwareTab, () => ShowPreferencesAt(PreferencesWindow.FirmwareTab)),
            Opens("Preferences", ShowPreferences),
        ];

        private List<PadMenuEntry> EmuSenRows(bool inGame)
        {
            var rows = new List<PadMenuEntry>();
            if (inGame) rows.Add(Opens("Preferences", ShowPreferences));
            if (!_bigScreen) rows.Add(new(() => IsFullScreen ? "Leave Full Screen" : "Full Screen", ToggleFullScreen));
            if (!_bigScreenForced) rows.Add(new(() => _bigScreen ? "Exit Big Picture" : "Big Picture", () => SetBigPicture(!_bigScreen)));
            rows.Add(Ask("Exit EmuSen...", "Exit EmuSen?", inGame ? "The game closes, then EmuSen." : "EmuSen closes.", Close));
            return rows;
        }

        // No first, so a press of A straight through the question changes nothing.
        private static List<PadMenuEntry> QuestionRows(PadMenuEntry asked, Action no) =>
        [
            new(() => QuestionNo, no, closes: false),
            new(() => QuestionYes, asked.Accept),
        ];

        private static PadMenuEntry Opens(string text, Action show) => new(() => text, show) { Opens = true };

        private static PadMenuEntry Submenu(string title, Func<List<PadMenuEntry>> rows, bool startsSection = false) =>
            new(() => title, () => { }, closes: false) { Submenu = rows, Opens = true, StartsSection = startsSection };

        private static PadMenuEntry Ask(string text, string question, string detail, Action yes) =>
            new(() => text, yes) { Question = question, QuestionDetail = detail };

        // The slot the Save and Load rows act on, and whether anything is saved in it.
        private string SlotValue() => StatePathForSlot(_stateSlot) is string path && File.Exists(path) ? $"Slot {_stateSlot}" : $"Slot {_stateSlot} (Empty)";

        private void ShowPadMenuPage(bool turned)
        {
            PadMenuPage page = _padMenuPages[^1];
            _padMenuEntries.Clear();
            _padMenuEntries.AddRange(page.Rows());
            bool nested = _padMenuPages.Count > 1;

            PadMenuTitle.Text = string.Join("  ›  ", _padMenuPages.Select(p => PageTitle(p)));
            PadMenuDetail.Text = page.Detail;
            PadMenuDetail.IsVisible = page.Detail is not null;
            string change = _padMenuEntries.Any(e => e.Adjust is not null) ? "      Left Right  Change" : "";
            PadMenuHint.Text = PadHints.Face(page.Question ? "A  Choose      B  No" : nested ? "A  Choose      B  Back      Start  Close" + change : "A  Choose      B  Close" + change);
            if (_bigScreen) ShowBigPadMenu(page, nested);

            PadMenuList.Refresh(_padMenuEntries);
            int at = Math.Clamp(page.At, 0, _padMenuEntries.Count - 1);
            PadMenuList.SelectedIndex = at;
            PadMenuList.ScrollIntoView(at);
            if (turned && _bigScreen) BeginMenuOpening(PadMenuBig);
            UpdateSlotCard();
        }

        // The first page is named by the game, as the library names it, or Main Menu; a submenu or a question by its own title.
        private string PageTitle(PadMenuPage page) =>
            page.Title ?? (!_padMenuInGame ? MainMenuTitle : RunningGameTitle());

        private void ShowBigPadMenu(PadMenuPage page, bool nested)
        {
            PadMenuBig.Title = PageTitle(page);
            PadMenuBig.SubtitleLetterCase = page.Question ? LetterCase.None : LetterCase.Upper;
            string? under = page.Question ? page.Detail : nested ? "‹ " + string.Join(" › ", _padMenuPages.SkipLast(1).Select(p => PageTitle(p))) : null;
            PadMenuBig.Subtitle = under is null ? null : FitSubtitle(page.Question ? under : under.ToUpper(CultureInfo.CurrentCulture));
            PadMenuBig.Footer = page.Title == EmuSenMenu ? BigMenuFooter : null;
            PadMenuBig.HintFamily = HelpFamily;
            PadMenuBig.Hints = PadMenuHints(_padMenuEntries.Any(e => e.Adjust is not null), nested, page.Question);
        }

        // The subtitle broken at spaces into lines the panel holds whole, as MenuPanel draws each line of it and cuts none it is given to fit.
        private string FitSubtitle(string text)
        {
            double u = MenuPanel.GetScale(this) is > 0 and var s ? s : 1;
            Size area = PadMenuBig.Bounds.Width > 0 ? PadMenuBig.Bounds.Size : Bounds.Size;
            double panel = Math.Min(Math.Min(area.Width * PadMenuBig.WidthFraction, area.Height * PadMenuBig.MaxWidthToHeight), area.Width - 48 * u);
            double room = panel - 48 * u, size = PadMenuBig.SubtitleSize * u;
            if (room <= 0 || FontFiles.Load(BigMenuFont) is not { } face) return text;
            var lines = new List<string>();
            string line = "";
            foreach (string word in text.Split(' '))
            {
                string longer = line.Length == 0 ? word : line + " " + word;
                if (line.Length > 0 && Measure(face, size, longer) > room)
                {
                    lines.Add(line);
                    line = word;
                }
                else line = longer;
            }
            lines.Add(line);
            return string.Join("\n", lines);
        }

        private static double Measure(GlyphTypeface face, double size, string text) =>
            text.Length == 0 ? 0 : TextShaper.Current.ShapeText(text.AsMemory(), new TextShaperOptions(face, size, 0, CultureInfo.CurrentCulture, 0, 0, null)).Sum(g => g.GlyphAdvance);

        private void PadMenuCommand(UiButton button)
        {
            int count = _padMenuEntries.Count;
            int at = Math.Max(PadMenuList.SelectedIndex, 0);

            switch (button)
            {
                case UiButton.Up: PadMenuList.SelectedIndex = (at + count - 1) % count; break;
                case UiButton.Down: PadMenuList.SelectedIndex = (at + 1) % count; break;

                case UiButton.Left:
                case UiButton.Right:
                    if (_padMenuEntries[at].Adjust is { } adjust)
                    {
                        adjust(button == UiButton.Right ? 1 : -1);
                        RefreshPadMenuText(at);
                    }
                    break;

                case UiButton.Accept: ChoosePadMenuRow(at); break;
                case UiButton.Back: PadMenuBack(); break;
                case UiButton.Menu: ClosePadMenu(); break;
            }
        }

        // A submenu or a question turns the page; anything else acts, putting the menu away first unless it stays.
        private void ChoosePadMenuRow(int at)
        {
            PadMenuEntry entry = _padMenuEntries[at];
            if (entry.Submenu is { } rows)
            {
                EnterPadMenuPage(new PadMenuPage(entry.Text(), rows), at, SubmenuKey(entry.Text()));
                return;
            }
            if (entry.Question is { } question)
            {
                EnterPadMenuPage(new PadMenuPage(question, () => QuestionRows(entry, PadMenuBack), entry.QuestionDetail, question: true), at, key: null);
                return;
            }
            if (entry.Closes) ClosePadMenu();
            entry.Accept();
            if (_padMenuOpen && !entry.Closes && at < _padMenuEntries.Count && ReferenceEquals(_padMenuEntries[at], entry)) RefreshPadMenuText(at);
        }

        // A submenu opens on the row it was left on, for as long as the window lives; a question always opens on No.
        private void EnterPadMenuPage(PadMenuPage page, int from, string? key)
        {
            _padMenuPages[^1].At = from;
            page.At = key is not null && _padSubmenuRows.TryGetValue(key, out int was) ? was : 0;
            _padMenuPages.Add(page);
            ShowPadMenuPage(turned: true);
        }

        // One page back, to the row the page was entered from; from the first page, closed.
        private void PadMenuBack()
        {
            if (_padMenuPages.Count <= 1)
            {
                ClosePadMenu();
                return;
            }
            PadMenuPage left = _padMenuPages[^1];
            if (!left.Question && left.Title is { } title) _padSubmenuRows[SubmenuKey(title)] = Math.Max(PadMenuList.SelectedIndex, 0);
            _padMenuPages.RemoveAt(_padMenuPages.Count - 1);
            ShowPadMenuPage(turned: true);
        }

        private string SubmenuKey(string title) => (_padMenuInGame ? "game/" : "main/") + title;

        private void ClosePadMenu()
        {
            if (!_padMenuOpen) return;

            PadMenuPanel.IsVisible = false;
            PadSlotCard.IsVisible = false;
            _padMenuOpen = false;
            _padMenuPages.Clear();
            _padQuiet = true;
            UpdateMenuBackdrop();

            // Only what this menu paused, and only if the game is still the screen: an entry may have closed or left it.
            if (_padMenuPaused && GameOnScreen) ResumeEmulation();
            _padMenuPaused = false;
        }

        private void RefreshPadMenuText(int keep)
        {
            PadMenuList.Refresh(_padMenuEntries);
            PadMenuList.SelectedIndex = keep;
            UpdateSlotCard();
        }

        private void StepStateSlot(int by) => SelectStateSlot((_stateSlot - 1 + by + StateSlots) % StateSlots + 1);

        private void StepSpeed(int by)
        {
            int[] speeds = { _speed.SlowMotionPercent, EmuSen.Common.SpeedController.NormalPercent, _speed.TurboPercent };
            int at = Math.Max(0, Array.IndexOf(speeds, _baseSpeedPercent));
            SetBaseSpeed(speeds[Math.Clamp(at + by, 0, speeds.Length - 1)]);
        }

        // A row tapped is chosen, as A chooses it; the right button goes back a page, as B does.
        private void SetUpPadMenuPointer()
        {
            // Ten rows at most: a plain stack, so a divider's gap is measured with its row rather than missed by the virtualizing panel's count.
            PadMenuList.ItemsPanel = new Avalonia.Controls.Templates.FuncTemplate<Panel?>(() => new StackPanel());
            PadMenuList.Tapped += (_, e) =>
            {
                if (!_padMenuOpen || e.Source is not Visual source || source.FindAncestorOfType<ListBoxItem>(includeSelf: true) is not { } row) return;
                int at = PadMenuList.IndexFromContainer(row);
                if (at < 0 || at >= _padMenuEntries.Count) return;
                PadMenuList.SelectedIndex = at;
                e.Handled = true;
                PadMenuCommand(UiButton.Accept);
            };
            PadMenuPanel.AddHandler(PointerPressedEvent, (_, e) =>
            {
                if (!_padMenuOpen || !e.GetCurrentPoint(PadMenuPanel).Properties.IsRightButtonPressed) return;
                e.Handled = true;
                PadMenuCommand(UiButton.Back);
            }, RoutingStrategies.Tunnel);
            PadMenuList.SelectionChanged += (_, _) => UpdateSlotCard();
            PadMenuPanel.LayoutUpdated += (_, _) => { if (_padMenuOpen) UpdateSlotCard(); };
        }

        // The section a row opens or closes: in the big panel the last row of a section has a lighter rule, on the desktop the first has a gap and a line above it.
        private void MarkSection(Control container, int index)
        {
            bool starts = index > 0 && _padMenuEntries[index].StartsSection;
            bool ends = index + 1 < _padMenuEntries.Count && _padMenuEntries[index + 1].StartsSection;
            container.Classes.Set(SectionEndClass, _bigScreen && ends);
            PaintSectionRule(container);
            if (container is not ListBoxItem item) return;
            item.TemplateApplied -= OnPadRowTemplate;
            item.TemplateApplied += OnPadRowTemplate;
            if (!_bigScreen && starts)
            {
                item.Margin = new Thickness(0, 9, 0, 0);
                item.BorderThickness = new Thickness(0, 1, 0, 0);
                item.BorderBrush = this.FindResource("LunaMuted") as IBrush ?? Brushes.Gray;
            }
            else
            {
                item.ClearValue(MarginProperty);
                item.ClearValue(Avalonia.Controls.Primitives.TemplatedControl.BorderThicknessProperty);
                item.ClearValue(Avalonia.Controls.Primitives.TemplatedControl.BorderBrushProperty);
            }
        }

        private void OnPadRowTemplate(object? sender, Avalonia.Controls.Primitives.TemplateAppliedEventArgs e)
        {
            if (sender is Control container) Avalonia.Threading.Dispatcher.UIThread.Post(() => PaintSectionRule(container));
        }

        private static void PaintSectionRule(Control container)
        {
            foreach (MenuRow row in container.GetVisualDescendants().OfType<MenuRow>())
            {
                if (container.Classes.Contains(SectionEndClass)) row.RuleColor = SectionRule;
                else row.ClearValue(MenuRow.RuleColorProperty);
            }
        }

        // Beside the menu while Save State or Load State is on: the slot's picture and when it was saved, where the screen leaves room for it.
        private void UpdateSlotCard()
        {
            int at = PadMenuList.SelectedIndex;
            bool show = _padMenuOpen && at >= 0 && at < _padMenuEntries.Count && _padMenuEntries[at].ShowsSlot && PlaceSlotCard();
            PadSlotCard.IsVisible = show;
            if (!show) return;
            string? state = StatePathForSlot(_stateSlot);
            bool saved = state is not null && File.Exists(state);
            PadSlotCaption.Text = saved ? $"Slot {_stateSlot}\nSaved {File.GetLastWriteTime(state!).ToString("ddd d MMM, HH:mm", CultureInfo.CurrentCulture)}" : $"Slot {_stateSlot}\nEmpty";
            Bitmap? picture = saved ? SlotPicture(SaveLibrary.PicturePathFor(state!)) : null;
            if (!ReferenceEquals(PadSlotPicture.Source, picture)) PadSlotPicture.Source = picture;
            PadSlotPicture.IsVisible = picture is not null;
        }

        // To the right of the panel, under its title, as wide as the room there allows up to 240 design pixels; false where there is too little room for it.
        private bool PlaceSlotCard()
        {
            if (PadMenuPanel.Child is not Control area || area.Bounds.Width <= 0) return false;
            double u = _bigScreen ? MenuPanel.GetScale(this) is > 0 and var s ? s : 1 : 1;
            Rect box = _bigScreen ? PadMenuBig.PanelBounds : PadMenuDesk.Bounds;
            if (box.Width <= 0) return false;
            double gap = 16 * u, width = Math.Min(240 * u, area.Bounds.Width - box.Right - 2 * gap);
            if (width < 150 * u) return false;
            double top = box.Top + (_bigScreen ? PadMenuBig.TitleBounds.Height : 0);
            if (PadSlotCard.Width != width) PadSlotCard.Width = width;
            var margin = new Thickness(box.Right + gap, top, 0, 0);
            if (PadSlotCard.Margin != margin) PadSlotCard.Margin = margin;
            double text = _bigScreen ? 20 * u : 15;
            if (PadSlotCaption.FontSize != text) PadSlotCaption.FontSize = text;
            return true;
        }

        private (string Path, DateTime When, Bitmap? Picture) _slotPicture = ("", default, null);

        // The slot's picture, read again only when its file changes.
        private Bitmap? SlotPicture(string path)
        {
            DateTime when = File.Exists(path) ? File.GetLastWriteTimeUtc(path) : default;
            if (_slotPicture.Path == path && _slotPicture.When == when) return _slotPicture.Picture;
            Bitmap? picture = null;
            try
            {
                if (when != default) picture = new Bitmap(path);
            }
            catch (Exception)
            {
                picture = null;
            }
            _slotPicture = (path, when, picture);
            return picture;
        }
    }
}
