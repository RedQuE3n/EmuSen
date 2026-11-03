using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Scraping;
using EmuSen.Tools.Scraper;

namespace EmuSen.Mistress.Views
{
    // The library's menu as the Home card on LunaP's Card form over the dimmed library: Continue, the game in hand, Search, Start Console, Tools, Settings and Power - see EmuSen_BigPicture.md §49.
    public partial class MainWindow
    {
        public const string HomeTitle = "Home", ContinueRow = "Continue", SelectedGameRow = "Selected game", SearchRow = "Search", ToolsMenu = "Tools", SettingsMenu = "Settings", PowerMenu = "Power";
        public const string ScraperRow = "Scraper", CheatsRow = "Cheats", CollectionsRow = "Collections", ShellRow = "Shell", ThemeSettingsRow = "Themes", QuitEmuSenRow = "Quit EmuSen...";

        // The game the card was opened over, kept while it is open: its file, its name and how it becomes a favourite.
        private sealed record HomeGame(string Path, string Title, Action ToggleFavourite);

        private HomeGame? _homeGame;
        private DateTime _suspendedAt = DateTime.Now;

        // The library's menu is always the Home card, on the desktop as in a big-screen session; a game on screen has the Pause Card instead.
        internal bool HomeCardShown => !_padMenuInGame;

        // The pad's menu is drawn as a card: always over the library, over a game in a big-screen session.
        internal bool CardShown => PauseCardShown || HomeCardShown;

        // The first page in the order of EmuSen_Interface.md §2.3: the way back to play, the thing in hand, the list's tools, the tools, settings and the way out.
        private List<PadMenuEntry> HomeRows()
        {
            var rows = new List<PadMenuEntry>();
            if (_session is not null) rows.Add(new(() => ContinueRow, ToggleLibrary) { ShowsFrame = true, Label = () => ContinueRow, Value = RunningGameTitle });
            if (_homeGame is { } game) rows.Add(new(() => game.Title, () => ShowGamePage(game.Path)) { Opens = true, ShowsCover = true, Label = () => SelectedGameRow, Value = () => game.Title });
            if (SearchAction() is { } search) rows.Add(new(() => SearchRow, search));
            if (StartableConsoles.Count > 0) rows.Add(Submenu(StartConsoleMenu, StartConsolePage));
            rows.Add(new(() => ToolsMenu, () => { }, closes: false) { Submenu = ToolRows, Opens = true, ShowsTools = true, StartsSection = rows.Count > 0, Label = () => ToolsMenu, Value = AttentionValue });
            rows.Add(new(() => SettingsMenu, () => ShowSettings()) { Opens = true });
            rows.Add(Submenu(PowerMenu, PowerRows, startsSection: true));
            return rows;
        }

        // The game under the selection: the themed gamelist's, or the sidebar library's; a folder or a collection is no game.
        private HomeGame? GameInHand()
        {
            if (ThemedLibraryShown)
                return _themed?.SelectedGame is { Folder: false, IsCollection: false } g
                    ? new HomeGame(g.File, g.Name, () => ToggleThemedFavourite(g))
                    : null;
            return SelectedLibraryEntry is RomEntry e ? new HomeGame(e.FullPath, DisplayTitle(e), () => ToggleFavourite(e)) : null;
        }

        private string FavouriteText(string path) => _records.IsFavourite(path) ? "Remove from Favourites" : "Add to Favourites";

        // Y makes the game in hand a favourite on the first page.
        private bool HomeFavouriteHere => HomeCardShown && _homeGame is not null && _padMenuPages.Count == 1;

        private void ToggleHomeFavourite()
        {
            if (!HomeFavouriteHere) return;
            _homeGame!.ToggleFavourite();
            RefreshPadMenuText(Math.Max(PadMenuList.SelectedIndex, 0));
        }

        // The list's search where the view has one: the themed gamelist's, or the sidebar's box.
        private Action? SearchAction() =>
            !ThemedLibraryShown ? SearchFromThePad : _themed is { ViewName: "gamelist" } && _themedSearch is not null ? SearchThemedGamelist : null;

        // The tools with their live lines as the rows' values and the pane's lines, then the library's other tools (EmuSen_Interface.md §19).
        private List<PadMenuEntry> ToolRows()
        {
            var rows = new List<PadMenuEntry>
            {
                Tool(ScraperRow, ShowScraper, HomeScreen.Scraper),
                Tool(PackagesMenu, ShowPackages, HomeScreen.Packages),
                Tool(LibraryManagerMenu, ShowLibraryManager, HomeScreen.Library),
                Tool(CoresMenu, ShowCores, HomeScreen.Cores),
                Tool(CheatsRow, ShowCheatDatabase, HomeScreen.Cheats),
                Said(ThemeSettingsRow, ShowThemeSettings, ThemeName, () => $"{ThemeName()} is the big picture's look; A opens the themes, their variants and their options."),
                Said(CollectionsRow, () => ShowSettings(SettingsScreen.Library, "collections"), () => $"{_collections.Count:N0}",
                    () => _collections.Count == 1 ? "1 collection of your own, beside Favourites and Recently played." : $"{_collections.Count:N0} collections of your own, beside Favourites and Recently played."),
                Opens(CompressedDiscsMenu, ShowCompressedDiscs),
                Opens(ShellRow, ShowShellConsole),
            };
            return rows;
        }

        // A tool's row: its live line as the value, and in the pane its state as its own library says it, then what A and B do (EmuSen_Interface.md §19.6).
        private PadMenuEntry Tool(string name, Action show, string place) =>
            new(() => name, show) { Opens = true, Label = () => name, Value = () => SummaryOf(place)?.MarkedBrief, PaneSaid = () => (name, SummaryOf(place)?.Sentence ?? HomeScreen.About(place), Hint(name)) };

        private PadMenuEntry Said(string name, Action show, Func<string> value, Func<string> sentence) =>
            new(() => name, show) { Opens = true, Label = () => name, Value = value, PaneSaid = () => (name, sentence(), Hint(name)) };

        private static string Hint(string name) => $"A opens {name}; B comes back here.";

        // The big picture's look by its name, as Settings ▸ Library ▸ Themes names it.
        private string ThemeName() => LookList.FirstOrDefault(l => EmuSen.Mistress.BigPicture.BigPictureLooks.IsCurrent(_appSettings, l))?.Name ?? "EmuSen";

        // A tool's live line in the pane, or what it does before it has said anything.
        private string ToolLine(string place) => SummaryOf(place)?.Marked ?? HomeScreen.Description(place);

        // The Tools row's value: the first tool whose line needs the player.
        private string? AttentionValue() =>
            new[] { HomeScreen.Cores, HomeScreen.Library, HomeScreen.Scraper, HomeScreen.Packages, HomeScreen.Cheats }.Select(SummaryOf).FirstOrDefault(s => s is { NeedsAttention: true })?.MarkedBrief;

        // Each tool and its line, as the Tools row's pane lists them.
        private List<(string Name, string Status)> ToolLines() =>
            [(ScraperRow, ToolLine(HomeScreen.Scraper)), (PackagesMenu, ToolLine(HomeScreen.Packages)), (LibraryManagerMenu, ToolLine(HomeScreen.Library)), (CoresMenu, ToolLine(HomeScreen.Cores)), (CheatsRow, ToolLine(HomeScreen.Cheats))];

        // Full screen and big picture where the window owns them, and Quit; the device's sleep and shut down wait for a way to know EmuSen is its shell (§49.6).
        private List<PadMenuEntry> PowerRows()
        {
            var rows = new List<PadMenuEntry>();
            if (!_bigScreen) rows.Add(new(() => IsFullScreen ? "Leave Full Screen" : "Full Screen", ToggleFullScreen));
            if (!_bigScreenForced) rows.Add(new(() => _bigScreen ? "Exit Big Picture" : "Big Picture", () => SetBigPicture(!_bigScreen)));
            rows.Add(Ask(QuitEmuSenRow, "Quit EmuSen?", _session is null ? "EmuSen closes." : "The game closes, then EmuSen.", Close));
            return rows;
        }

        // The suspended game's paused frame, its name and how long ago it was left.
        private void ShowContinuePane()
        {
            Bitmap? frame = null;
            try { if (_session is { } session && CurrentPicture(session) is { } still) frame = Picture(still.Rgba, still.Width, still.Height); }
            catch (CrystalException) { frame = null; }
            double u = MenuPanel.GetScale(this) is > 0 and var s ? s : 1;
            string console = _currentRomPath is string path ? new RomEntry(path).CoreDisplayName + " · " : "";
            _pauseCardPane.ShowPicture(frame, console + PausedAgo(DateTime.Now - _suspendedAt), PaneWidth * u, null, PaneWidth * 3 / 4 * u);
        }

        // The game in hand's cover, its name, its console and whether it is a favourite.
        private void ShowCoverPane()
        {
            if (_homeGame is not { } game)
            {
                _pauseCardPane.Clear();
                return;
            }
            var entry = new RomEntry(game.Path);
            Bitmap? cover = CoverPathFor(entry) is string path ? _covers.Get(path) : null;
            double u = MenuPanel.GetScale(this) is > 0 and var s ? s : 1;
            string console = entry.CoreDisplayName;
            _pauseCardPane.ShowBeside(cover, game.Title, [_records.IsFavourite(game.Path) ? $"{console} · Favourite" : console], "A opens its page: states, options, media and details.", 240 * u);
        }

        private void ShowLinesPane(IReadOnlyList<(string Name, string Status)> lines)
        {
            double u = MenuPanel.GetScale(this) is > 0 and var s ? s : 1;
            _pauseCardPane.ShowLines(lines, PaneWidth * u);
        }

        // The pane's width inside its gutter on the 1280-wide design, in design pixels.
        private const double PaneWidth = 592;

        internal static string PausedAgo(TimeSpan since) =>
            since.TotalMinutes < 1 ? "Paused just now"
            : since.TotalMinutes < 60 ? $"Paused {(int)since.TotalMinutes} min ago"
            : since.TotalHours < 24 ? $"Paused {(int)since.TotalHours} h ago" : $"Paused {(int)since.TotalDays} d ago";
    }
}
