using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.LogicalTree;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // A big-screen session's list menus in ES-DE's layout: the pad menu and the game options, over the screen blurred - see EmuSen_Settings_Reference.md §4.69.
    public partial class MainWindow
    {
        // Barlow Condensed, OFL 1.1, shipped beside the program - see THIRD_PARTY_NOTICES.md and §4.69.
        internal static readonly string BigMenuFont = Path.Combine(AppContext.BaseDirectory, "Assets", "Fonts", "BarlowCondensed-Regular.ttf");

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
            PadMenuList.ContainerPrepared += (_, e) =>
            {
                if (_bigScreen && e.Index >= 0 && e.Index < _padMenuEntries.Count) DescribeRow(e.Container, _padMenuEntries[e.Index]);
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

        // The pad menu's list moves into the big panel for a big-screen session and back into the desktop's box after.
        private void ApplyBigMenuLook(bool on)
        {
            if (on == PadMenuBig.IsVisible && (on ? ReferenceEquals(PadMenuBig.Child, PadMenuList) : PadMenuDock.Children.Contains(PadMenuList))) return;
            if (on)
            {
                PadMenuDock.Children.Remove(PadMenuList);
                PadMenuBig.Child = PadMenuList;
                MenuRows.Apply(PadMenuList);
                PadMenuList.ClearValue(MaxHeightProperty);
                PadMenuPanel.Background = null;
            }
            else
            {
                PadMenuBig.Child = null;
                MenuRows.Remove(PadMenuList);
                PadMenuList.MaxHeight = 520;
                PadMenuList.Background = Brushes.Transparent;
                PadMenuDock.Children.Add(PadMenuList);
                PadMenuPanel.Background = new SolidColorBrush(Color.FromArgb(0xD0, 0, 0, 0));
            }
            PadMenuDesk.IsVisible = !on;
            PadMenuBig.IsVisible = on;
            UpdateMenuBackdrop();
        }

        // A row's label, value and kind, read from its entry: an adjustable entry steps sideways, one that opens a screen has a chevron.
        private static void DescribeRow(Control row, PadMenuEntry entry)
        {
            MenuRows.SetLabel(row, entry.Label?.Invoke() ?? entry.Text());
            MenuRows.SetValue(row, entry.Value?.Invoke());
            MenuRows.SetKind(row, entry.Adjust is not null ? MenuRowKind.Option : entry.Opens ? MenuRowKind.Submenu : MenuRowKind.Action);
        }

        private void ShowBigPadMenu(string title, bool adjustable)
        {
            PadMenuBig.Title = title;
            PadMenuBig.Footer = BigMenuFooter;
            PadMenuBig.HintFamily = HelpFamily;
            PadMenuBig.Hints = PadMenuHints(adjustable);
        }

        // ES-DE's help for its main menu, in Mistress's words: Start and B close it, A chooses, the pad moves and, where a row has a value, changes it.
        private static IReadOnlyList<HintEntry> PadMenuHints(bool adjustable)
        {
            var hints = new List<HintEntry>
            {
                new("Close Menu") { Button = PadGlyphButton.Start },
                new("Select") { Button = AcceptGlyph },
                new("Close Menu") { Button = BackGlyph },
            };
            if (adjustable) hints.Add(new HintEntry("Change") { Button = PadGlyphButton.DPadLeftRight });
            hints.Add(new HintEntry("Choose") { Button = PadGlyphButton.DPadUpDown });
            return hints;
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

        // With the swap on, the east button chooses and the south one goes back - §4.61.
        internal static PadGlyphButton AcceptGlyph => PadHints.Swapped ? PadGlyphButton.East : PadGlyphButton.South;
        internal static PadGlyphButton BackGlyph => PadHints.Swapped ? PadGlyphButton.South : PadGlyphButton.East;

        // Blurred while a big-screen menu is over the screen: the pad menu, or a sheet that draws its own menu.
        private void UpdateMenuBackdrop()
        {
            bool sheetMenu = Sheets.Current is { } sheet && Sheets.DrawsMenu(sheet);
            Sheets.HintFamily = HelpFamily;
            MenuBackdrop.IsVisible = _bigScreen && (_padMenuOpen || sheetMenu);
            HideThemedHelp(MenuBackdrop.IsVisible);
            MenuPanel? shown = !MenuBackdrop.IsVisible ? null : Sheets.Current is { } top && Sheets.DrawsMenu(top) ? MenuPanelOf(top) : _padMenuOpen ? PadMenuBig : null;
            if (!ReferenceEquals(shown, _menuShown))
            {
                _menuShown = shown;
                if (shown is not null) BeginMenuOpening(shown);
            }
        }

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
