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
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // The pad's menu as pages: the game's menu or the Home card, the submenus under them, and a yes-or-no before anything that ends a game - see EmuSen_Settings_Reference.md §4.69.8.
    public partial class MainWindow
    {
        public const string QuestionNo = "No", QuestionYes = "Yes", MemoryCardsMenu = "Memory Cards", CompressedDiscsMenu = "Compressed Discs", PackagesMenu = "Packages", CoresMenu = "Cores", SwapPlayersRow = "Swap Players 1 and 2";
        public const string GameOptionsMenu = "Game Options", LeaveMenu = "Leave", ControllersMenu = "Controllers", DisplayMenu = "Display", SoundMenu = "Sound";

        // One page of the menu: its title (null for the first, whose title is the game's or Home), its rows, the row last on, and for a question the sentence under it.
        private sealed class PadMenuPage(string? title, Func<List<PadMenuEntry>> rows, string? detail = null, bool question = false)
        {
            public string? Title { get; } = title;
            public Func<List<PadMenuEntry>> Rows { get; } = rows;
            public string? Detail { get; } = detail;
            public bool Question { get; } = question;
            public int At { get; set; }

            // A page of the Pause Card's own (EmuSen_Interface.md §18.5): its words for A and for Left and Right in the help bar, and whether Start resumes there.
            public string? AcceptWord { get; init; }
            public string? SideWord { get; init; }
            public bool StartResumes { get; init; } = true;

            // Left and Right on a row with no value of its own, and the page's own buttons, such as the reel's shoulders.
            public Action<int>? Sideways { get; init; }
            public Func<UiButton, bool>? Pad { get; init; }

            // What B does before the page closes, such as a settings page going back a level of its own.
            public Func<bool>? Back { get; init; }
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
            _paneShowing = null;
            _padMenuPages.Clear();
            _homeGame = _padMenuInGame ? null : GameInHand();
            _padMenuPages.Add(new PadMenuPage(null, _padMenuInGame ? GameMenuRows : HomeRows));
            PadCard.Title = _padMenuInGame ? "Pause" : HomeTitle;
            if (!_padMenuInGame) AskToolSummaries();

            // A card opened again as it slides out takes over the pause that slide was holding for the game.
            bool held = _cardSlide.Leaving && _closingResumes;
            _cardSlide.Stop();
            _closingResumes = false;

            // Before the menu shows, so no frame runs under it - the library's own rule (§4.18).
            _padMenuPaused = _padMenuInGame && (held || !IsPaused);
            if (_padMenuPaused && !IsPaused) PauseEmulation();

            PadMenuPanel.IsVisible = true;
            _padMenuOpen = true;
            PlacePadMenu();
            ApplyStatusBar();
            ShowPadMenuPage();
            UpdateMenuBackdrop();
        }

        // The Card form over the library at a page of its own: the grid's List card, and Home's Settings and Power (EmuSen_Interface.md §18.1).
        private void OpenCard(string title, Func<List<PadMenuEntry>> rows, string? breadcrumb = null)
        {
            OpenPadMenu();
            if (_padMenuInGame) return;
            _homeGame = null;
            _padMenuPages.Clear();
            _padMenuPages.Add(new PadMenuPage(title, rows, breadcrumb));
            PadCard.Title = title;
            ShowPadMenuPage();
            // A card opened from another card, as Start Console from the List card, comes in as a card of its own (EmuSen_Interface.md §20.6).
            if (_cardSlide.Moving is null)
            {
                _cardSlide.Enter(PadCard.CardPanel, UiClock(), MenusScaleUp);
                RunCardSlide();
            }
        }

        public const string SaveStateRow = "Save State", LoadStateRow = "Load State";

        // The Pause Card's first page: carry on, go back, save, load, the speed, the console's own rows where they apply, then the game's options and the ways out - see EmuSen_BigPicture.md §48.3.
        private List<PadMenuEntry> GameMenuRows() =>
        [
            new(() => "Resume", () => { }),
            new(RewindMenuText, OpenReelPage, closes: false) { Label = () => "Rewind", Value = () => RewindUnavailable() ?? RewindHeld(), ShowsReel = true },
            new(() => $"Save      <  {SlotValue()}  >", SaveState, StepStateSlot) { Label = () => SaveStateRow, Value = SlotValue, ShowsSlot = true },
            new(() => $"Load      <  {SlotValue()}  >", LoadState, StepStateSlot) { Label = () => LoadStateRow, Value = SlotValue, ShowsSlot = true },
            new(() => $"Speed      <  {DescribeSpeed(_baseSpeedPercent)}  >", () => StepSpeed(1), StepSpeed, closes: false) { Label = () => "Speed", Value = () => DescribeSpeed(_baseSpeedPercent) },
            .. DiscRows(),
            // Shown while player 2 has a pad, connected or kept, under Controllers where it would make a ninth row - see EmuSen_Input.md §8.13.
            .. (SwapOnFirstPage ? new[] { new PadMenuEntry(() => SwapPlayersRow, SwapPlayers) } : Array.Empty<PadMenuEntry>()),
            Submenu(GameOptionsMenu, GameOptionsRows, startsSection: true),
            Submenu(LeaveMenu, LeaveRows),
        ];

        // At most seven rows; the console's own only where the running machine reports them - see EmuSen_BigPicture.md §48.5.
        private List<PadMenuEntry> GameOptionsRows()
        {
            List<PadMenuEntry> rows =
            [
                Opens("Cheats", ShowActiveCheats),
                SettingsPageRow(ControllersMenu, SettingsScreen.GameControllers, "Choose", "Value"),
                SettingsPageRow(DisplayMenu, SettingsScreen.GameDisplay, "Change", "Value"),
                SettingsPageRow(SoundMenu, SettingsScreen.GameSound, "Change", "Value"),
            ];
            if (DriveEngine is not null) rows.Add(SettingsPageRow(MemoryCardsMenu, SettingsScreen.GameCards, "Change", "Card"));
            return rows;
        }

        // A row of Game Options that turns the card to a page of Settings' description (EmuSen_Interface.md §20).
        private PadMenuEntry SettingsPageRow(string title, string page, string accept, string side) =>
            new(() => title, () => OpenSettingsOnCard(page, accept, side), closes: false) { Opens = true, TurnsPage = true };

        // The first page holds at most eight rows: seven of its own, the console's disc rows, and Swap Players only where they leave it room (EmuSen_BigPicture.md §48.3).
        private bool SwapOnFirstPage => _gamepad.CanSwapPlayers && 7 + DiscRows().Count() < 8;

        // Back to the library with the game kept, or an end asked first, No first.
        private List<PadMenuEntry> LeaveRows() =>
        [
            new(() => "Back to Library", ToggleLibrary),
            Ask("Restart Game...", "Restart Game?", "The game starts again from the beginning. Progress since your last save is lost.", ResetEmulation),
            Ask("Quit Game...", "Quit Game?", "The game closes and the library comes back.", ShowLibrary),
            Ask("Quit EmuSen...", "Quit EmuSen?", "The game closes, then EmuSen.", Close),
        ];

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

        // A page of the same card: only the rows, the header and the pane change, with no opening effect - see EmuSen_Interface.md §18.5.
        private void ShowPadMenuPage()
        {
            PadMenuPage page = _padMenuPages[^1];
            _padMenuEntries.Clear();
            _padMenuEntries.AddRange(page.Rows());
            ShowCard(page, _padMenuPages.Count > 1);

            PadMenuList.Refresh(_padMenuEntries);
            int at = Math.Clamp(page.At, 0, _padMenuEntries.Count - 1);
            PadMenuList.SelectedIndex = at;
            PadMenuList.ScrollIntoView(at);
            UpdateCardPane();
        }

        // The first page is named by the game, as the library names it, or Home; a submenu or a question by its own title.
        private string PageTitle(PadMenuPage page) =>
            page.Title ?? (!_padMenuInGame ? HomeTitle : RunningGameTitle());

        // The subtitle broken at spaces into lines the panel holds whole, as MenuPanel draws each line of it and cuts none it is given to fit.
        private string FitSubtitle(MenuPanel big, string text)
        {
            double u = MenuPanel.GetScale(this) is > 0 and var s ? s : 1;
            Size area = PadMenuPanel.Bounds.Width > 0 ? PadMenuPanel.Bounds.Size : Bounds.Size;
            double panel = Math.Min(Math.Min(area.Width * big.WidthFraction, area.Height * big.MaxWidthToHeight), area.Width - 48 * u);
            double room = panel - 48 * u, size = big.SubtitleSize * u;
            if (room <= 0 || FontFiles.Load(MenuPanel.GetFontPath(big) ?? BigMenuFont) is not { } face) return text;
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

            if (_padMenuPages.Count > 0 && _padMenuPages[^1].Pad is { } own && own(button))
            {
                Heard(button, true);
                return;
            }
            switch (button)
            {
                case UiButton.Up: PadMenuList.SelectedIndex = (at + count - 1) % count; Heard(button, count > 1); break;
                case UiButton.Down: PadMenuList.SelectedIndex = (at + 1) % count; Heard(button, count > 1); break;

                case UiButton.Left:
                case UiButton.Right:
                    if (_padMenuEntries[at].Adjust is { } adjust)
                    {
                        adjust(button == UiButton.Right ? 1 : -1);
                        RefreshPadMenuText(at);
                        Heard(button, true);
                    }
                    else if (_padMenuPages[^1].Sideways is { } sideways)
                    {
                        sideways(button == UiButton.Right ? 1 : -1);
                        RefreshPadMenuText(at);
                        Heard(button, true);
                    }
                    break;

                case UiButton.Accept: Heard(button, true); ChoosePadMenuRow(at); break;
                // X takes a screenshot on every page of the in-game menu, with no row of its own.
                case UiButton.Screensaver when _padMenuInGame: TakeScreenshot(); break;
                // Y makes the game in hand a favourite on the Home card (EmuSen_Interface.md §2.1).
                case UiButton.Search: ToggleHomeFavourite(); break;
                case UiButton.Back: PadMenuBack(); Heard(button, true); break;
                case UiButton.Menu: ClosePadMenu(); Cue("back"); break;
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
            // A screen of its own lays over the Pause Card, which keeps its page, row and pause beneath it - see EmuSen_Interface.md §18.5.
            if (entry.Opens && entry.Closes && _padMenuInGame)
            {
                _padMenuPages[^1].At = at;
                entry.Accept();
                if (!Sheets.IsPresenting && _settings is null) ClosePadMenu();
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
            ShowPadMenuPage();
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
            if (left.Back is { } own && own())
            {
                ShowPadMenuPage();
                return;
            }
            if (!left.Question && left.Title is { } title) _padSubmenuRows[SubmenuKey(title)] = Math.Max(PadMenuList.SelectedIndex, 0);
            _padMenuPages.RemoveAt(_padMenuPages.Count - 1);
            ShowPadMenuPage();
        }

        private string SubmenuKey(string title) => (_padMenuInGame ? "game/" : "home/") + title;

        // The card slides out first, and the game it paused resumes, or the screen beneath takes the pad, only once it is gone (EmuSen_Interface.md §20.6).
        private void ClosePadMenu()
        {
            if (!_padMenuOpen) return;

            _padMenuOpen = false;
            _padMenuPages.Clear();
            _padQuiet = true;
            _closingResumes = _padMenuPaused;
            _padMenuPaused = false;
            _cardSlide.Leave(PadCard.CardPanel, UiClock(), MenusScaleUp, PadMenuGone);
            RunCardSlide();
        }

        // Only what this menu paused, and only if the game is still the screen: an entry may have closed or left it.
        private void PadMenuGone()
        {
            PadMenuPanel.IsVisible = false;
            if (_gamePage is null) ApplyStatusBar();
            UpdateMenuBackdrop();
            if (_closingResumes && GameOnScreen) ResumeEmulation();
            _closingResumes = false;
        }

        private readonly EmuSen.Mistress.BigPicture.CardSlide _cardSlide = new();
        private bool _closingResumes;

        internal EmuSen.Mistress.BigPicture.CardSlide CardSlideNow => _cardSlide;

        private Avalonia.Threading.DispatcherTimer? _slideTimer;

        // The slide stepped with the interface's clock at the display's pace while it moves, and stopped with the window.
        private void RunCardSlide()
        {
            if (_cardSlide.Moving is null) return;
            if (_slideTimer is null)
            {
                _slideTimer = new Avalonia.Threading.DispatcherTimer(TimeSpan.FromMilliseconds(16), Avalonia.Threading.DispatcherPriority.Render, (_, _) => { if (!_cardSlide.Advance(UiClock())) _slideTimer?.Stop(); });
                Closed += (_, _) => { _slideTimer?.Stop(); _cardSlide.Stop(); };
            }
            _slideTimer.Start();
        }

        private void RefreshPadMenuText(int keep)
        {
            PadMenuList.Refresh(_padMenuEntries);
            PadMenuList.SelectedIndex = keep;
            UpdateCardPane();
        }

        private void StepStateSlot(int by)
        {
            SelectStateSlot((_stateSlot - 1 + by + StateSlots) % StateSlots + 1);
            _paneShowing = null;
        }

        private void StepSpeed(int by)
        {
            int[] speeds = { SlowMotionPercent, NormalPercent, TurboPercent };
            int at = Math.Max(0, Array.IndexOf(speeds, _baseSpeedPercent));
            SetBaseSpeed(speeds[Math.Clamp(at + by, 0, speeds.Length - 1)]);
        }

        // A row tapped is chosen, as A chooses it; the right button goes back a page, as B does.
        private void SetUpPadMenuPointer()
        {
            // Ten rows at most: a plain stack, so a divider's gap is measured with its row rather than missed by the virtualizing panel's count.
            PadMenuList.ItemsPanel = new Avalonia.Controls.Templates.FuncTemplate<Panel?>(() => new StackPanel());
            // Pressed and let go on one row, found by where the pointer is, since a card's unchosen row draws no background to be hit.
            int pressed = -1;
            int RowAt(Avalonia.Input.PointerEventArgs e)
            {
                for (int i = 0; i < _padMenuEntries.Count; i++)
                    if (PadMenuList.ContainerFromIndex(i) is Control row && row.TranslatePoint(default, PadMenuList) is { } top && new Rect(top, row.Bounds.Size).Contains(e.GetPosition(PadMenuList))) return i;
                return -1;
            }
            PadMenuList.AddHandler(PointerPressedEvent, (_, e) =>
            {
                pressed = _padMenuOpen && e.GetCurrentPoint(PadMenuList).Properties.IsLeftButtonPressed ? RowAt(e) : -1;
            }, RoutingStrategies.Tunnel, handledEventsToo: true);
            PadMenuList.AddHandler(PointerReleasedEvent, (_, e) =>
            {
                int at = pressed;
                pressed = -1;
                if (!_padMenuOpen || e.InitialPressMouseButton != MouseButton.Left || at < 0 || at != RowAt(e)) return;
                PadMenuList.SelectedIndex = at;
                e.Handled = true;
                PadMenuCommand(UiButton.Accept);
            }, RoutingStrategies.Tunnel, handledEventsToo: true);
            PadMenuPanel.AddHandler(PointerPressedEvent, (_, e) =>
            {
                if (!_padMenuOpen || !e.GetCurrentPoint(PadMenuPanel).Properties.IsRightButtonPressed) return;
                e.Handled = true;
                PadMenuCommand(UiButton.Back);
            }, RoutingStrategies.Tunnel);
            PadMenuList.SelectionChanged += (_, _) => UpdateCardPane();
            PadMenuPanel.LayoutUpdated += (_, _) => { if (_padMenuOpen) UpdateCardPane(); };
        }

        // The last row of a section has a lighter rule than the rows' own.
        private void MarkSection(Control container, int index)
        {
            bool ends = index + 1 < _padMenuEntries.Count && _padMenuEntries[index + 1].StartsSection;
            container.Classes.Set(SectionEndClass, ends);
            PaintSectionRule(container);
            if (container is not ListBoxItem item) return;
            item.TemplateApplied -= OnPadRowTemplate;
            item.TemplateApplied += OnPadRowTemplate;
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
    }
}
