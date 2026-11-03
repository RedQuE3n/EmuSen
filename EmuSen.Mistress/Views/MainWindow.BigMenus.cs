using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.LogicalTree;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // A big-screen session's menus: the pad menu's cards dimming the screen, and the sheets that draw ES-DE's menu over it blurred - see EmuSen_Settings_Reference.md §4.69.
    public partial class MainWindow
    {
        // A question as ES-DE's message box in a big-screen session, a dialog on the desktop; the accepting button is focused first - see EmuSen_BigPicture.md §34.2a.
        internal Task<bool> AskAsync(Window owner, string title, string message, string accept, string cancel) =>
            _bigScreen
                ? Dialogs.MenuConfirmAsync(owner, message, accept, cancel, HelpFamily,
                    [new("Select") { Button = PadHints.Glyph(PadGlyphButton.South) }, new("Choose") { Button = PadGlyphButton.DPadLeftRight }])
                : Dialogs.ConfirmAsync(owner, title, message, accept, cancel);

        // Barlow Condensed, OFL 1.1, LunaP's copy beside the program - see THIRD_PARTY_NOTICES.md and EmuSen_Interface.md §5.2.
        internal static readonly string BigMenuFont = LunaFonts.CondensedFile;

        // The menus are laid out for 800 pixels of height and scaled from there.
        private const double BigMenuDesignHeight = 800;

        internal static string BigMenuFooter => "EmuSen " + BuildName.Split('+')[0];

        private void SetUpBigMenus()
        {
            MenuBackdrop.Target = ScreenContent;
            MenuPanel.SetFontPath(this, BigMenuFont);
            SetUpSheetLook();
            SizeChanged += (_, e) => ScaleBigMenus(e.NewSize.Height);
            ScaleBigMenus(Bounds.Height > 0 ? Bounds.Height : Height);
            SetUpPauseCard();
            PadMenuList.ContainerPrepared += (_, e) =>
            {
                if (e.Index < 0 || e.Index >= _padMenuEntries.Count) return;
                if (CardShown) DescribeRow(e.Container, _padMenuEntries[e.Index]);
                MarkSection(e.Container, e.Index);
            };
        }

        private void ScaleBigMenus(double height)
        {
            double scale = double.IsFinite(height) && height > 0 ? height / BigMenuDesignHeight : 1;
            MenuPanel.SetScale(this, scale);
            MenuBackdrop.Radius = 14 * scale;
        }

        // The running game as the library names it, with no extension or folder (Q81): the themed view's name, else the sidebar's.
        private string RunningGameTitle()
        {
            if (_currentRomPath is not { } path) return PadMenuTitle.Text ?? "Game";
            if (_appSettings.LibraryStyle == EmuSen.Galaxia.Models.AppSettings.LibraryStyleTheme) return MetadataFor(path).Title;
            return DisplayTitle(new EmuSen.Mistress.Library.RomEntry(path));
        }

        // ES-DE's title for a game's options, whatever the entry (Q85).
        internal const string GameOptionsMenuTitle = "Gamelist Options";

        // The pad menu's list moves into the card when one is shown, and back into the desktop's box after.
        private void PlacePadMenu()
        {
            bool card = CardShown;
            bool placed = card ? ReferenceEquals(PadCard.Content, PadMenuList) : PadMenuDock.Children.Contains(PadMenuList);
            if (placed && PadMenuDesk.IsVisible == !card && PadCard.IsVisible == card) return;
            bool wasCard = ReferenceEquals(PadCard.Content, PadMenuList);
            PadMenuDock.Children.Remove(PadMenuList);
            if (wasCard) PadCard.Content = null;
            if (card)
            {
                PadCard.Content = PadMenuList;
                if (!wasCard) MenuRows.Apply(PadMenuList);
                PadMenuList.ClearValue(MaxHeightProperty);
                PadMenuPanel.Background = null;
            }
            else
            {
                MenuRows.Remove(PadMenuList);
                PadMenuList.ClearValue(MaxHeightProperty);
                PadMenuList.Background = Brushes.Transparent;
                PadMenuDock.Children.Add(PadMenuList);
                PadMenuPanel.Background = new SolidColorBrush(Color.FromArgb(0xD0, 0, 0, 0));
            }
            PadMenuDesk.IsVisible = !card;
            PadCard.IsVisible = card;
            UpdateMenuBackdrop();
        }

        // A row's label, value and kind, read from its entry: an adjustable entry steps sideways, one that opens a screen has a chevron.
        private static void DescribeRow(Control row, PadMenuEntry entry)
        {
            MenuRows.SetLabel(row, entry.Label?.Invoke() ?? entry.Text());
            MenuRows.SetValue(row, entry.Value?.Invoke());
            MenuRows.SetKind(row, entry.Adjust is not null ? MenuRowKind.Option : entry.Opens || entry.Submenu is not null ? MenuRowKind.Submenu : MenuRowKind.Action);
        }

        // The game options' help: Select cancels, B applies where the menu has something to apply, A chooses.
        internal static IReadOnlyList<HintEntry> GameOptionsHints(bool applies, bool adjustable)
        {
            var hints = new List<HintEntry>
            {
                new(applies ? "Close (Cancel)" : "Close") { Button = PadGlyphButton.Select },
                new("Select") { Button = AcceptGlyph },
                new(applies ? "Close (Apply)" : "Close") { Button = BackGlyph },
            };
            if (adjustable) hints.Add(new HintEntry("Change Value") { Button = PadGlyphButton.DPadLeftRight });
            hints.Add(new HintEntry("Choose") { Button = PadGlyphButton.DPadUpDown });
            return hints;
        }

        // With the swap on, the east button chooses and the south one goes back - §4.61, §4.79.5.
        internal static PadGlyphButton AcceptGlyph => PadHints.Glyph(PadGlyphButton.South);
        internal static PadGlyphButton BackGlyph => PadHints.Glyph(PadGlyphButton.East);

        // Blurred while a sheet that draws its own menu is over a big screen; a card brings its own scrim and dims instead (EmuSen_BigPicture.md §48.2, §49.2).
        private void UpdateMenuBackdrop()
        {
            bool sheetMenu = Sheets.Current is { } sheet && Sheets.DrawsMenu(sheet);
            bool card = _padMenuOpen && CardShown;
            Sheets.HintFamily = HelpFamily;
            MenuBackdrop.IsVisible = _bigScreen && sheetMenu;
            // A card under a sheet that draws its menu stays mounted, covered by the blurred screen the sheet stands on - see EmuSen_Interface.md §18.5.
            PadMenuPanel.Opacity = card && MenuBackdrop.IsVisible ? 0 : 1;
            HideThemedHelp(MenuBackdrop.IsVisible || card);
            MenuPanel? shown = sheetMenu && _bigScreen ? MenuPanelOf(Sheets.Current!) : card ? PadCard.CardPanel : null;
            if (!ReferenceEquals(shown, _menuShown))
            {
                // A card a sheet uncovers was on screen beneath it all along, so it does not open again - see EmuSen_Interface.md §18.5.
                bool uncovered = card && ReferenceEquals(shown, PadCard.CardPanel) && _cardCovered;
                _cardCovered = card && shown is not null && !ReferenceEquals(shown, PadCard.CardPanel);
                _menuShown = shown;
                if (shown is not null && !uncovered) BeginMenuOpening(shown);
            }
        }

        private bool _cardCovered;

        private MenuPanel? _menuShown;
        private readonly MenuOpening _menuOpening = new();

        internal bool MenusScaleUp => _appSettings.BigPictureInterface.MenuOpeningEffect != Galaxia.Models.BigPictureInterface.OpeningNone;

        // A menu that has just come on screen, or a menu page just turned to, grows into place (Q92).
        internal void BeginMenuOpening(MenuPanel panel) => _menuOpening.Begin(panel, UiClock(), MenusScaleUp);

        internal MenuOpening MenuOpeningNow => _menuOpening;

        internal MenuPanel? MenuShownNow => _menuShown;

        // A presented sheet's content lives in the layer, not the window, so its menu is found there.
        private MenuPanel? MenuPanelOf(Window sheet) =>
            (Sheets.SheetOf(sheet) as Avalonia.LogicalTree.ILogical ?? sheet.Content as Avalonia.LogicalTree.ILogical)?.GetSelfAndLogicalDescendants().OfType<MenuPanel>().FirstOrDefault();

        private readonly List<HintBar> _hiddenThemedHelp = new();

        // One help bar on screen, the menu's, as ES-DE shows: the theme's is hidden under a menu and given back after.
        private void HideThemedHelp(bool hide)
        {
            if (!hide)
            {
                foreach (HintBar bar in _hiddenThemedHelp) bar.IsVisible = true;
                _hiddenThemedHelp.Clear();
                return;
            }
            if (_hiddenThemedHelp.Count > 0 || !ThemedLibraryShown || _themed?.Stage is not { } stage) return;
            foreach (HintBar bar in stage.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().Where(b => b.IsVisible).ToList())
            {
                bar.IsVisible = false;
                _hiddenThemedHelp.Add(bar);
            }
        }
    }
}
