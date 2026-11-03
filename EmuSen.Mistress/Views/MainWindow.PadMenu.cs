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
    // The pad's menu as pages: the game's menu or the Home card, the submenus under them, and a yes-or-no before anything that ends a game - see EmuSen_Settings_Reference.md §4.69.8.
    public partial class MainWindow
    {
        public const string QuestionNo = "No", QuestionYes = "Yes", MemoryCardsMenu = "Memory Cards", CompressedDiscsMenu = "Compressed Discs", PackagesMenu = "Packages", CoresMenu = "Cores", SwapPlayersRow = "Swap Players 1 and 2";
        public const string GameOptionsMenu = "Game Options", LeaveMenu = "Leave", ControllersMenu = "Controllers", DisplayMenu = "Display", VolumeRow = "Volume";

        // One page of the menu: its title (null for the first, whose title is the game's or Home), its rows, the row last on, and for a question the sentence under it.
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
            _paneShowing = null;
            _padMenuPages.Clear();
            _homeGame = _padMenuInGame ? null : GameInHand();
            _padMenuPages.Add(new PadMenuPage(null, _padMenuInGame ? GameMenuRows : HomeRows));
            PadCard.Title = _padMenuInGame ? "Pause" : HomeTitle;
            if (!_padMenuInGame) AskToolCounts();

            // Before the menu shows, so no frame runs under it - the library's own rule (§4.18).
            _padMenuPaused = _padMenuInGame && !IsPaused;
            if (_padMenuPaused) PauseEmulation();

            PadMenuPanel.IsVisible = true;
            _padMenuOpen = true;
            PlacePadMenu();
            ApplyStatusBar();
            ShowPadMenuPage(turned: false);
            UpdateMenuBackdrop();
        }

        public const string SaveStateRow = "Save State", LoadStateRow = "Load State";

        // The Pause Card's first page: carry on, go back, save, load, the speed, the console's own rows where they apply, then the game's options and the ways out - see EmuSen_BigPicture.md §48.3.
        private List<PadMenuEntry> GameMenuRows() =>
        [
            new(() => "Resume", () => { }),
            new(RewindMenuText, RewindFromPadMenu, closes: false) { Label = () => "Rewind", Value = () => RewindUnavailable() ?? RewindHeld(), ShowsReel = true },
            new(() => $"Save      <  {SlotValue()}  >", SaveState, StepStateSlot) { Label = () => SaveStateRow, Value = SlotValue, ShowsSlot = true },
            new(() => $"Load      <  {SlotValue()}  >", LoadState, StepStateSlot) { Label = () => LoadStateRow, Value = SlotValue, ShowsSlot = true },
            new(() => $"Speed      <  {DescribeSpeed(_baseSpeedPercent)}  >", () => StepSpeed(1), StepSpeed, closes: false) { Label = () => "Speed", Value = () => DescribeSpeed(_baseSpeedPercent) },
            .. DiscRows(),
            .. DriveRows(),
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
                Submenu(ControllersMenu, ControllersRows),
                Submenu(DisplayMenu, DisplayRows),
                VolumeEntry(),
            ];
            if (DiscDrive is not null || DriveEngine is not null) rows.Add(Opens(MemoryCardsMenu, ShowMemoryCards));
            return rows;
        }

        // The first page holds at most eight rows: seven of its own, the console's disc rows, and Swap Players only where they leave it room (EmuSen_BigPicture.md §48.3).
        private bool SwapOnFirstPage => _gamepad.CanSwapPlayers && 7 + DiscRows().Count() + DriveRows().Count() < 8;

        private List<PadMenuEntry> ControllersRows()
        {
            List<PadMenuEntry> rows = [Opens("Players & Controllers", () => ShowPreferencesAt(PreferencesWindow.ControllersTab)), Opens("Controller Bindings", ShowControllerBindings)];
            if (_gamepad.CanSwapPlayers && !SwapOnFirstPage) rows.Insert(0, new PadMenuEntry(() => SwapPlayersRow, SwapPlayers));
            return rows;
        }

        private List<PadMenuEntry> DisplayRows()
        {
            List<PadMenuEntry> rows = [Opens("Graphics", ShowGraphicsSettings), Opens("Shaders", ShowShaderSettings)];
            // The running game's own crop, where its console has an overscan to replace - see EmuSen_Settings_Reference.md §4.100.
            if (_currentRomPath is string rom && GraphicsSettingsWindow.HasOverscan(_activeConsole)) rows.Add(Opens(CropThisGame, () => ShowGameCrop(rom, RunningGameTitle())));
            return rows;
        }

        // Back to the library with the game kept, or an end asked first, No first.
        private List<PadMenuEntry> LeaveRows() =>
        [
            new(() => "Back to Library", ToggleLibrary),
            Ask("Restart Game...", "Restart Game?", "The game starts again from the beginning. Progress since your last save is lost.", ResetEmulation),
            Ask("Quit Game...", "Quit Game?", "The game closes and the library comes back.", ShowLibrary),
            Ask("Quit EmuSen...", "Quit EmuSen?", "The game closes, then EmuSen.", Close),
        ];

        private List<PadMenuEntry> SettingsRows() =>
        [
            Opens("Players & Controllers", () => ShowPreferencesAt(PreferencesWindow.ControllersTab)),
            Opens("Controller Bindings", ShowControllerBindings),
            Opens("Graphics", ShowGraphicsSettings),
            Opens("Shaders", ShowShaderSettings),
            Opens("Cheats", ShowActiveCheats),
            Opens(MemoryCardsMenu, ShowMemoryCards),
            Opens(PreferencesWindow.FirmwareTab, () => ShowPreferencesAt(PreferencesWindow.FirmwareTab)),
            Opens("Preferences", ShowPreferences),
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
            string shot = _padMenuInGame ? "      X  Screenshot" : "";
            PadMenuHint.Text = PadHints.Face(page.Question ? "A  Choose      B  No" + shot : nested ? "A  Choose      B  Back      Start  Close" + change + shot : "A  Choose      B  Close" + change + shot);
            if (CardShown) ShowCard(page, nested);

            PadMenuList.Refresh(_padMenuEntries);
            int at = Math.Clamp(page.At, 0, _padMenuEntries.Count - 1);
            PadMenuList.SelectedIndex = at;
            PadMenuList.ScrollIntoView(at);
            if (turned && CardShown) BeginMenuOpening(PadCard.CardPanel);
            UpdateSlotCard();
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
                // X takes a screenshot on every page of the in-game menu, with no row of its own.
                case UiButton.Screensaver when _padMenuInGame: TakeScreenshot(); break;
                // Y makes the game in hand a favourite on the Home card (EmuSen_Interface.md §2.1).
                case UiButton.Search: ToggleHomeFavourite(); break;
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
            _padMenuPages[^1].At = at;
            List<PadMenuPage> pages = [.. _padMenuPages];
            if (entry.Closes) ClosePadMenuHoldingPause();
            entry.Accept();
            HandOverHeldPause(pages);
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

        private string SubmenuKey(string title) => (_padMenuInGame ? "game/" : "home/") + title;

        private void ClosePadMenu() => ClosePadMenu(holdPause: false);

        private void ClosePadMenuHoldingPause() => ClosePadMenu(holdPause: true);

        private void ClosePadMenu(bool holdPause)
        {
            if (!_padMenuOpen) return;

            PadMenuPanel.IsVisible = false;
            PadSlotCard.IsVisible = false;
            _padMenuOpen = false;
            if (_gamePage is null) ApplyStatusBar();
            _padMenuPages.Clear();
            _padQuiet = true;
            UpdateMenuBackdrop();

            // Only what this menu paused, and only if the game is still the screen: an entry may have closed or left it.
            if (_padMenuPaused && GameOnScreen)
            {
                if (holdPause) _pauseHeld = true;
                else ResumeEmulation();
            }
            _padMenuPaused = false;
        }

        // Held while a closing card's entry runs: the pause goes to the sheet it opened, else back to the game - see EmuSen_LibretroHost.md §10z.
        private bool _pauseHeld;

        private void HandOverHeldPause(List<PadMenuPage> pages)
        {
            if (!_pauseHeld) return;
            _pauseHeld = false;
            if (Sheets.IsPresenting && GameOnScreen) (_sheetPaused, _cardUnderSheet) = (true, pages);
            else if (GameOnScreen && IsPaused && !_padMenuOpen) ResumeEmulation();
        }

        private void RefreshPadMenuText(int keep)
        {
            PadMenuList.Refresh(_padMenuEntries);
            PadMenuList.SelectedIndex = keep;
            UpdateSlotCard();
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

        // The section a row opens or closes: on a card the last row of a section has a lighter rule, on the desktop the first has a gap and a line above it.
        private void MarkSection(Control container, int index)
        {
            bool starts = index > 0 && _padMenuEntries[index].StartsSection;
            bool ends = index + 1 < _padMenuEntries.Count && _padMenuEntries[index + 1].StartsSection;
            container.Classes.Set(SectionEndClass, CardShown && ends);
            PaintSectionRule(container);
            if (container is not ListBoxItem item) return;
            item.TemplateApplied -= OnPadRowTemplate;
            item.TemplateApplied += OnPadRowTemplate;
            if (!CardShown && starts)
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

        // Beside the desktop's box while Save State or Load State is on: the slot's picture and when it was saved, where the screen leaves room for it.
        private void UpdateSlotCard()
        {
            int at = PadMenuList.SelectedIndex;
            UpdateCardPane();
            bool show = _padMenuOpen && !CardShown && at >= 0 && at < _padMenuEntries.Count && _padMenuEntries[at].ShowsSlot && PlaceSlotCard();
            PadSlotCard.IsVisible = show;
            if (!show) return;
            string? state = StatePathForSlot(_stateSlot);
            bool saved = state is not null && File.Exists(state);
            PadSlotCaption.Text = saved ? $"Slot {_stateSlot}\nSaved {File.GetLastWriteTime(state!).ToString("ddd d MMM, HH:mm", CultureInfo.CurrentCulture)}" : $"Slot {_stateSlot}\nEmpty";
            Bitmap? picture = saved ? SlotPicture(SaveLibrary.PicturePathFor(state!)) : null;
            if (!ReferenceEquals(PadSlotPicture.Source, picture)) PadSlotPicture.Source = picture;
            PadSlotPicture.IsVisible = picture is not null;
        }

        // To the right of the desktop's box, as wide as the room there allows up to 240 pixels; false where there is too little room for it.
        private bool PlaceSlotCard()
        {
            if (PadMenuPanel.Child is not Control area || area.Bounds.Width <= 0) return false;
            Rect box = PadMenuDesk.Bounds;
            if (box.Width <= 0) return false;
            double gap = 16, width = Math.Min(240, area.Bounds.Width - box.Right - 2 * gap);
            if (width < 150) return false;
            if (PadSlotCard.Width != width) PadSlotCard.Width = width;
            var margin = new Thickness(box.Right + gap, box.Top, 0, 0);
            if (PadSlotCard.Margin != margin) PadSlotCard.Margin = margin;
            if (PadSlotCaption.FontSize != 15) PadSlotCaption.FontSize = 15;
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
