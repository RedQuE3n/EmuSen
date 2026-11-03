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
using EmuSen.Mistress.Scraping;
using EmuSen.Tools.Scraper;

namespace EmuSen.Mistress.Views
{
    // The library's menu as the Home card on LunaP's Card form over the dimmed library: Continue, the game in hand, Search, Start Console, Tools, Settings and Power - see EmuSen_BigPicture.md §49.
    public partial class MainWindow
    {
        public const string HomeTitle = "Home", ContinueRow = "Continue", SelectedGameRow = "Selected game", SearchRow = "Search", ToolsMenu = "Tools", SettingsMenu = "Settings", PowerMenu = "Power";
        public const string ScraperRow = "Scraper", ShellRow = "Shell", ThemeSettingsRow = "Theme Settings", QuitEmuSenRow = "Quit EmuSen...";
        public const string PackagesLine = "Makes, checks and zips game packages", LibraryManagerLine = "Finds duplicate and damaged dumps", CoresLine = "The engines each console runs on";

        // The game the card was opened over, kept while it is open: its file, its name and how it becomes a favourite.
        private sealed record HomeGame(string Path, string Title, Action ToggleFavourite);

        private HomeGame? _homeGame;
        private DateTime _suspendedAt = DateTime.Now;

        // The counts the tools' lines read, asked off the interface thread as the card opens and kept a minute; null until answered or where nothing can say.
        private int? _missingMedia, _coresListed, _coresWaiting;
        private DateTime _toolCountsAsked = DateTime.MinValue;
        private static readonly TimeSpan ToolCountsKept = TimeSpan.FromMinutes(1);

        // The scraper's own listing of games missing media, the runtime's cores with how many await approval, and where they are asked; a test replaces them.
        internal static Func<int?> MissingMediaQuery = CountMissingMedia;
        internal static Func<(int Listed, int Waiting)?> CoresQuery = () => CountCores(Runtime);
        internal static Func<Action, Task> ToolCountsBackground = work => Task.Run(work);

        private sealed record ToolCounts(int? MissingMedia, (int Listed, int Waiting)? Cores);

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
            rows.Add(new(() => ToolsMenu, () => { }, closes: false) { Submenu = ToolRows, Opens = true, ShowsTools = true, StartsSection = rows.Count > 0, Label = () => ToolsMenu, Value = CoresWaitingValue });
            rows.Add(Submenu(SettingsMenu, SettingsRows));
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

        // The four tools with their lines in the pane, then the library's other tools.
        private List<PadMenuEntry> ToolRows()
        {
            var rows = new List<PadMenuEntry>
            {
                Tool(ScraperRow, ShowScraper, ScraperStatus, () => _missingMedia is int n and > 0 ? $"{n} no art" : null),
                Tool(PackagesMenu, ShowPackages, () => PackagesLine),
                Tool(LibraryManagerMenu, ShowLibraryManager, () => LibraryManagerLine),
                Tool(CoresMenu, ShowCores, CoresStatus, CoresWaitingValue),
                Opens(CompressedDiscsMenu, ShowCompressedDiscs),
            };
            AddCollectionMenuEntries(rows);
            if (_bigScreen) rows.Add(Opens(ThemeSettingsRow, ShowThemeSettings));
            rows.Add(Opens(ShellRow, ShowShellConsole));
            return rows;
        }

        private static PadMenuEntry Tool(string name, Action show, Func<string> status, Func<string?>? value = null) =>
            new(() => name, show) { Opens = true, Status = status, Label = value is null ? null : () => name, Value = value };

        // The Tools row's and the Cores row's live status: the cores waiting for approval, with the warning sign.
        private string? CoresWaitingValue() => _coresWaiting is int w and > 0 ? $"⚠ {w} to approve" : null;

        // Each tool and its line, as the Tools row's pane lists them.
        private List<(string Name, string Status)> ToolLines() =>
            [(ScraperRow, ScraperStatus()), (PackagesMenu, PackagesLine), (LibraryManagerMenu, LibraryManagerLine), (CoresMenu, CoresStatus())];

        private string ScraperStatus() =>
            _scrapeRun is { State: ScrapeRunState.Running } run ? $"Scraping {run.Done} of {run.Total}"
            : _missingMedia is not int n ? "Scrapes art and facts for the library"
            : n == 0 ? "Every game has its media" : n == 1 ? "1 game missing media" : $"{n} games missing media";

        private string CoresStatus() =>
            _coresWaiting is int w && w > 0 ? (w == 1 ? "1 core waiting for approval" : $"{w} cores waiting for approval")
            : _coresListed is int n ? $"{n} cores, none waiting for approval" : CoresLine;

        // Asked once a minute at most, off the interface thread; the pane is drawn again when the answer comes.
        private void AskToolCounts()
        {
            if (DateTime.UtcNow - _toolCountsAsked < ToolCountsKept) return;
            _toolCountsAsked = DateTime.UtcNow;
            Func<int?> media = MissingMediaQuery;
            Func<(int Listed, int Waiting)?> cores = CoresQuery;
            ToolCounts? counts = null;
            Task asked = ToolCountsBackground(() => counts = new ToolCounts(media(), cores()));
            if (asked.IsCompletedSuccessfully) ToolCountsAnswered(counts!);
            else asked.ContinueWith(t => { if (t.IsCompletedSuccessfully) Dispatcher.UIThread.Post(() => ToolCountsAnswered(counts!)); }, TaskScheduler.Default);
        }

        private void ToolCountsAnswered(ToolCounts counts)
        {
            _missingMedia = counts.MissingMedia;
            _coresListed = counts.Cores?.Listed;
            _coresWaiting = counts.Cores?.Waiting;
            if (!_padMenuOpen) return;
            RefreshPadMenuText(Math.Max(PadMenuList.SelectedIndex, 0));
            UpdateCardPane();
        }

        // The Scraper window's Missing Media section counted, by the same `scrape ls` request; null where the command is not in use or refuses.
        private static int? CountMissingMedia()
        {
            if (!ScrapeCommand.Available) return null;
            try { return ScrapeRow.Listing(ScrapeCommand.Ask(ScrapeCommand.Request(ScraperPlan.Line("ls", "missing", new ScrapeOptions())))).Rows.Count; }
            catch (Exception e) when (e is ScrapeCommandException or JsonException or InvalidOperationException) { return null; }
        }

        // The runtime's cores, read as the Cores window reads them, and those whose trust awaits the player's approval.
        private static (int Listed, int Waiting)? CountCores(CrystalRuntime runtime)
        {
            try
            {
                using JsonDocument cores = JsonDocument.Parse(runtime.Cores());
                var empty = new Dictionary<string, string>();
                List<EmuSen.Tools.Cores.CoreRow> rows = cores.RootElement.EnumerateArray().Select(c => EmuSen.Tools.Cores.CoreRow.OfRuntime(c, empty, (_, _) => false)).OfType<EmuSen.Tools.Cores.CoreRow>().ToList();
                return (rows.Count, rows.Count(r => r.AwaitsApproval));
            }
            catch (Exception e) when (e is CrystalException or JsonException or InvalidOperationException) { return null; }
        }

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
