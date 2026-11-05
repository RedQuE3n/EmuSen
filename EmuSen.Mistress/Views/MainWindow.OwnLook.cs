using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Styling;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;

using EmuSen.Galaxia.Native;

namespace EmuSen.Mistress.Views
{
    // EmuSen's own look: Home and the grid of covers on LunaP's HubLayout and GalleryLayout, the desktop's Home pane, and what they read and ask of the window - see EmuSen_BigPicture.md §51.
    public partial class MainWindow : IHomeHost, IGamesHost, ICollectionsHost
    {
        public const string HomeSidebarKey = "home", HomeScreenName = "Home", GamesScreenName = "Games";

        private HubLayout? _homeLayout;
        private GalleryLayout? _gallery;
        private HomeScreen? _home;
        private GamesScreen? _games;
        private IReadOnlyList<GridGame>? _gridGames;
        private readonly Random _shuffle = new();

        // Which of the own look's screens the big screen shows: Home first, the grid once a place or Search opens it.
        private string _ownScreen = HomeScreenName;

        // The desktop's Home pane in the library's content area, chosen from the sidebar.
        private bool _desktopHome;

        public HubLayout? HomeLayout => _homeLayout?.IsVisible == true ? _homeLayout : null;

        public GalleryLayout? GamesLayout => _gallery?.IsVisible == true ? _gallery : null;

        public HomeScreen? HomeShown => HomeLayout is null ? null : _home;

        public GamesScreen? GamesShown => GamesLayout is null ? null : _games;

        private bool OwnScreenShown => LibraryView.IsVisible && (HomeLayout is not null || GamesLayout is not null);

        // Called on every showing of the library that no theme drew: the big screen's own look, or the desktop's Home pane.
        private void ShowOwnLook(bool themed)
        {
            bool home = !themed && (_bigScreen ? _ownScreen == HomeScreenName : _desktopHome);
            bool games = !themed && _bigScreen && _ownScreen == GamesScreenName;
            if (home) FillHome();
            else if (_homeLayout is not null) _homeLayout.IsVisible = false;
            if (games) FillGames();
            else if (_gallery is not null) _gallery.IsVisible = false;
            if (home || games) LibraryContent.IsVisible = false;
        }

        // Start in the own look, B on the grid, and a place's way back: Home the screen, never a card over the grid (EmuSen_Interface.md §0.9).
        internal void ShowHome()
        {
            _ownScreen = HomeScreenName;
            if (_bigScreen) ShowLibraryEntries();
            else ShowDesktopHome(true);
            _homeLayout?.FocusStart();
            _homeLayout?.Focus();
        }

        internal void ShowGames(bool search = false, long? collection = null)
        {
            _ownScreen = GamesScreenName;
            _games ??= new GamesScreen(this);
            if (collection is not null || search) _games.ShowCollection(collection);
            ShowLibraryEntries();
            if (search) _gallery?.OpenSearch(_games.SearchText, [KeyboardLayout.Search, KeyboardLayout.Symbols]);
            _gallery?.Focus();
        }

        private void ShowDesktopHome(bool on)
        {
            _desktopHome = on;
            LibraryContent.IsVisible = !on;
            ShowOwnLook(ThemedLibraryShown);
            FillSidebar();
        }

        private IImage? Load(string? path) => path is null ? null : _covers.Get(path);

        private static CoverItem CoverOf(string path, string title, string? mark, double aspect, bool favourite, string facts, string line) =>
            new(path, title) { Aspect = GameFacts.CardShape(aspect), Mark = mark, IsFavourite = favourite, Facts = facts, Status = line };

        // A cover's picture and its art asked only for what a screen draws, through the library's cover cache.
        private IImage? CoverPicture(CoverItem item) => Load(((IGamePageHost)this).Art(item.Key).Cover);

        private IImage? HeroPicture(CoverItem item) => Load(((IGamePageHost)this).Art(item.Key).Hero);

        private bool _ownPicturesWired;

        private void WireOwnPictures()
        {
            if (_ownPicturesWired) return;
            _ownPicturesWired = true;
            _covers.Loaded += OnOwnPictureLoaded;
        }

        // A picture decoded after a screen was drawn draws it again.
        private void OnOwnPictureLoaded(string path)
        {
            if (HomeLayout is { } home) home.Refresh();
            if (GamesLayout is { } gallery) gallery.Refresh();
        }

        private static IReadOnlyList<PageHint> Hints(IReadOnlyList<ScreenHint> hints) =>
            hints.Select(h => new PageHint(Enum.Parse<PadGlyphButton>(h.Button), h.Text, h.Key)).ToList();

        private void FillHome()
        {
            _home ??= new HomeScreen(this);
            if (_homeLayout is null)
            {
                _homeLayout = new HubLayout { Name = "HomeScreen" };
                _homeLayout.ContinueChosen += () => _home.ChooseContinue();
                _homeLayout.TileChosen += path => _home.ChooseGame(path);
                _homeLayout.PlaceChosen += key => _home.ChoosePlace(key);
                _homeLayout.HintInvoked += key => _home.Hint(key, RowName(_homeLayout.Focused.Row), _homeLayout.Focused.Key);
                _homeLayout.PictureOf = CoverPicture;
                _homeLayout.ArtOf = HeroPicture;
                WireOwnPictures();
                LibraryArea.Children.Add(_homeLayout);
                _homeLayout.FocusStart();
            }
            _homeLayout.Form = _bigScreen ? AdaptiveForm.BigScreen : AdaptiveForm.Desktop;
            _homeLayout.PadFamily = HelpFamily == PadFamily.Generic ? PadFamily.Xbox : HelpFamily;
            _homeLayout.IsVisible = true;
            _homeLayout.Title = HomeScreen.Title;
            _homeLayout.RecentTitle = HomeScreen.RecentTitle;
            _homeLayout.PlacesTitle = HomeScreen.PlacesTitle;
            _homeLayout.Status = _home.StatusLine;
            _homeLayout.HintsFor = f => Hints(_home.HintsFor(RowName(f.Row)));
            _homeLayout.Continue = _home.Continue is { } c
                ? new HubContinue(c.Title, c.Facts, PausedFrame()) { Caption = HomeScreen.ContinueCaption, Button = HomeScreen.ResumeWord, Video = ((IGamePageHost)this).Art(_currentRomPath ?? "").Clip }
                : null;
            bool wasEmpty = _homeLayout.Shelf.Count == 0;
            _homeLayout.SetShelf(_home.Shelf.Select(g => CoverOf(g.Path, g.Title, g.Mark, g.Aspect, g.Favourite, g.Facts, g.Line)));
            _homeLayout.SetPlaces(HomePlaces());
            // A shelf that fills for the first time takes the focus where a hub starts, as if Home had opened on it.
            if (wasEmpty && _homeLayout.Shelf.Count > 0 && _homeLayout.Focused.Row == HubRow.Places) _homeLayout.FocusStart();
            AskToolSummaries();
        }

        private IEnumerable<HubPlace> HomePlaces() => _home.Places.Select(p => new HubPlace(p.Key, p.Label) { Status = p.Status, NeedsAttention = p.NeedsAttention });

        private static string RowName(HubRow row) => row switch { HubRow.Continue => "continue", HubRow.Shelf => "shelf", _ => "places" };

        // The suspended game's last picture, as Continue shows it.
        private IImage? PausedFrame()
        {
            try { return _session is { } session && CurrentPicture(session) is { } still ? Picture(still.Rgba, still.Width, still.Height) : null; }
            catch (CrystalException) { return null; }
        }

        private void FillGames()
        {
            _games ??= new GamesScreen(this);
            if (_gallery is null)
            {
                _gallery = new GalleryLayout { Name = "GamesScreen" };
                _gallery.ItemChosen += path => ((IGamesHost)this).Play(path);
                _gallery.SectionChosen += key => { _games.ChooseSection(key); FillGames(); };
                _gallery.HintInvoked += key => { if (key == "list") OpenListCard(); else if (key == "back") GamesBack(); else _games.Hint(key, _gallery.Focused.Key); };
                _gallery.SearchChanged += text => { _games.SetSearch(text); FillGames(); };
                _gallery.SearchClosed += _ => FillGames();
                _gallery.PictureOf = CoverPicture;
                WireOwnPictures();
                LibraryArea.Children.Add(_gallery);
            }
            _gallery.PadFamily = HelpFamily == PadFamily.Generic ? PadFamily.Xbox : HelpFamily;
            _gallery.IsVisible = true;
            _gallery.HintsFor = f => Hints(_games.HintsFor(f.InKeyboard, _games.SearchText.Length > 0));
            _gallery.SetSections(_games.Sections.Select(s => new AdaptiveSection(s.Key, s.Label)), _games.Section);
            DateTime now = PageClock();
            _gallery.SetItems(_games.Shown.Select(g => CoverOf(g.Facts.Path, g.Facts.Title, g.Facts.Mark, g.Facts.CoverAspect, g.Facts.Favourite, _games.FactsOf(g), g.Facts.Console)));
            _gallery.Count = _games.CountLine;
            _gallery.HintsFor = f => Hints(_games.HintsFor(f.InKeyboard, _games.SearchText.Length > 0));
        }

        private void GamesBack()
        {
            if (_games is null) return;
            _games.Back();
            Cue("back");
            if (_ownScreen == GamesScreenName) FillGames();
        }

        // The pad on the own look's screens: the directions move, A chooses, B climbs, Start goes Home; the rest as each screen's help bar declares.
        private void OwnScreenPadCommand(UiButton button)
        {
            if (GamesLayout is { } gallery)
            {
                switch (button)
                {
                    case UiButton.Up: Heard(button, gallery.Move(0, -1)); return;
                    case UiButton.Down: Heard(button, gallery.Move(0, 1)); return;
                    case UiButton.Left: Heard(button, gallery.Move(-1, 0)); return;
                    case UiButton.Right: Heard(button, gallery.Move(1, 0)); return;
                    case UiButton.Menu when !gallery.IsSearchOpen || !gallery.Focused.InKeyboard: ShowHome(); Cue("back"); return;
                    case UiButton.Back when !gallery.IsSearchOpen: GamesBack(); return;
                }
                if (ToolPaneWindow.Glyph(button) is { } glyph) Heard(button, gallery.Press(glyph));
                return;
            }
            if (HomeLayout is not { } home) return;
            switch (button)
            {
                case UiButton.Up: Heard(button, home.Move(0, -1)); return;
                case UiButton.Down: Heard(button, home.Move(0, 1)); return;
                case UiButton.Left: Heard(button, home.Move(-1, 0)); return;
                case UiButton.Right: Heard(button, home.Move(1, 0)); return;
                case UiButton.Menu when _bigScreen: home.FocusStart(); Cue("scroll"); return;
                case UiButton.Menu: OpenPadMenu(); return;
            }
            if (ToolPaneWindow.Glyph(button) is { } press) Heard(button, home.Press(press));
        }

        // The List card over the grid: the Card form's page set, its rows from the grid's description (EmuSen_Interface.md §18.1).
        private void OpenListCard()
        {
            if (_games is not { } games) return;
            string section = games.Sections.FirstOrDefault(s => s.Key == games.Section).Label ?? "";
            OpenCard($"{section} · {games.CountLine}", ListCardRows, GamesScreen.ListTitle);
        }

        private List<PadMenuEntry> ListCardRows()
        {
            var rows = new List<PadMenuEntry>();
            if (_games is not { } games) return rows;
            foreach (ListRow row in games.ListRows(_gallery?.Focused.Key))
            {
                string key = row.Key;
                Func<List<(string, string)>> pane = () => games.ListPane(key).Select(l => (l.Line, l.Muted)).ToList();
                PadMenuEntry entry = key switch
                {
                    GamesScreen.SearchRow => new(() => row.Label, () => ShowGames(search: true)) { Opens = true, PaneLines = pane },
                    GamesScreen.SortRow => new(() => row.Label, () => { games.StepSort(1); FillGames(); }, by => { games.StepSort(by); FillGames(); }, closes: false) { Label = () => row.Label, Value = () => games.Sort, PaneLines = pane },
                    GamesScreen.FilterRow => new(() => row.Label, () => { }, closes: false) { Submenu = FilterRows, Opens = true, Label = () => row.Label, Value = () => games.FilterValue, PaneLines = pane },
                    GamesScreen.JumpRow => new(() => row.Label, () => JumpBy(1), JumpBy, closes: false) { Label = () => row.Label, Value = () => games.ListRows(_gallery?.Focused.Key).First(r => r.Key == GamesScreen.JumpRow).Value, PaneLines = pane },
                    GamesScreen.RandomRow => new(() => row.Label, () => { if (games.RandomGame() is { } p) _gallery?.FocusOn(p); }) { PaneLines = pane },
                    GamesScreen.StartConsoleRow => new(() => row.Label, () => OpenCard(StartConsoleMenu, StartConsolePage, HomeTitle)) { Opens = true, PaneLines = pane },
                    _ => new(() => row.Label, ShowCollections) { Opens = true, PaneLines = pane },
                };
                rows.Add(entry);
            }
            return rows;
        }

        // Start Console as its mockup draws it: each console that starts empty by its display name with its engine, those with none dimmed as not supported yet, and the pane's sentences with the firmware the runtime finds (EmuSen_Interface.md §18.3).
        private List<PadMenuEntry> StartConsolePage() => StartConsoleEntries(RuntimeCatalog.Systems);

        // The page's rows from a systems table, which a test gives with a console no engine runs.
        public List<PadMenuEntry> StartConsoleEntries(IEnumerable<SystemFacts> systems)
        {
            var rows = new List<PadMenuEntry>();
            foreach (var system in systems.Where(s => s.StartsEmpty).GroupBy(s => s.Console, StringComparer.OrdinalIgnoreCase).Select(g => g.First())
                .OrderBy(s => s.Engines.Count == 0).ThenBy(s => s.Manufacturer, StringComparer.OrdinalIgnoreCase).ThenBy(s => s.ReleaseYear))
            {
                string name = system.DisplayName is { Length: > 0 } shown ? shown : system.Name;
                bool runs = system.Engines.Count > 0;
                string engine = runs ? system.Engines[0] : NotSupportedYet;
                string console = system.Console, id = system.Id;
                rows.Add(new PadMenuEntry(() => name, runs ? () => StartConsole(console) : () => { }, closes: runs)
                {
                    Label = () => name, Value = () => engine, Dimmed = !runs,
                    PaneSentences = () => runs
                        ? ([$"Starts {Article(name)} {name} with no disc in it, on {engine}.", "Its BIOS screen comes up; Change Disc on the Pause Card puts a game in."], FirmwareLine(id))
                        : ([$"EmuSen has no engine for the {name} yet."], null),
                });
            }
            return rows;
        }

        public const string NotSupportedYet = "Not supported yet";

        private static string Article(string name) => "AEIOU".Contains(char.ToUpperInvariant(name[0])) ? "an" : "a";

        // The runtime's firmware_status for a system: found, not found as Cores words it, or nothing where none is needed or it cannot say (EmuSen_Cores.md §3.11).
        private (string Line, bool Good)? FirmwareLine(string system)
        {
            try
            {
                using var answer = System.Text.Json.JsonDocument.Parse(Runtime.FirmwareStatus(system));
                List<string?> found = answer.RootElement.GetProperty("firmware").EnumerateArray().Select(f => f.TryGetProperty("status", out var s) ? s.GetString() : null).ToList();
                return found.Contains("present") ? ("✓ Firmware found.", true) : found.Contains("missing") ? ("✗ Not found: add it in Settings → Firmware", false) : null;
            }
            catch (Exception e) when (e is CrystalException or System.Text.Json.JsonException or InvalidOperationException or KeyNotFoundException) { return null; }
        }

        private void JumpBy(int by)
        {
            if (_games?.JumpFrom(_gallery?.Focused.Key, by) is { } path) _gallery?.FocusOn(path);
        }

        private List<PadMenuEntry> FilterRows() =>
            _games is not { } games ? []
            : games.FilterRows.Select(f => new PadMenuEntry(() => f.Label, () => { games.ChooseFilter(f.Key); ShowGames(); })).ToList();

        private CollectionsWindow? _collectionsTool;

        // Collections as a tool screen over the library, from the List card or a pad's Home card (EmuSen_Interface.md §18.4).
        private void ShowCollections()
        {
            if (_collectionsTool is not null) return;
            var window = _collectionsTool = new CollectionsWindow(this);
            window.Closed += (_, _) => { if (_collectionsTool == window) _collectionsTool = null; };
            _ = EmuSen.LunaP.Windowing.SheetLayer.Show(window, this);
        }

        IReadOnlyList<CollectionRow> ICollectionsHost.CollectionRows()
        {
            TimeSpan Played(IEnumerable<string> paths) => TimeSpan.FromSeconds(paths.Sum(p => _recordSnapshot.GetValueOrDefault(p)?.PlaySeconds ?? 0));
            var rows = _collections.Select(c => _records.Members(c.Id).ToList() is var members ? new CollectionRow(KeyOf(c), c.Name, members, c.Id, Played: Played(members)) : null!).ToList();
            List<string> favourites = _allScan.Entries.Where(Listed).Select(e => e.FullPath).Where(_records.IsFavourite).ToList();
            List<string> recent = _recentSnapshot.Select(e => e.FullPath).ToList();
            rows.Add(new CollectionRow(FavouritesKey, "Favourites", favourites, Automatic: true, Played: Played(favourites)));
            rows.Add(new CollectionRow(RecentKey, "Recently played", recent, Automatic: true, Played: Played(recent)));
            rows.Add(new CollectionRow(CollectionsPane.NewKey, "New collection…", []));
            return rows;
        }

        CoverItem ICollectionsHost.CoverFor(string path)
        {
            GameFacts f = FactsShown(path);
            CoverItem item = CoverOf(path, f.Title, f.Mark, f.CoverAspect, false, "", f.Console);
            return item with { Picture = CoverPicture(item) };
        }

        // The game Home's shelf or the grid last focused, else the desktop list's.
        string? ICollectionsHost.GameInHand =>
            _ownScreen == GamesScreenName && _gallery?.Focused.Key is { } shown ? shown
            : _homeLayout?.Focused is { Row: HubRow.Shelf, Key: { } kept } ? kept
            : SelectedLibraryEntry?.FullPath;

        void ICollectionsHost.OpenCollection(CollectionRow row)
        {
            _collectionsTool?.Close();
            if (!_bigScreen)
            {
                ChooseCollection(row.Key);
                return;
            }
            if (row.Id is long id) ShowGames(collection: id);
            else
            {
                _games ??= new GamesScreen(this);
                _games.ShowCollection(null);
                _games.ChooseSection(row.Key == FavouritesKey ? GamesScreen.Favourites : GamesScreen.Recent);
                ShowGames();
            }
        }

        void ICollectionsHost.NewCollection() =>
            NewCollectionAsync(null).ContinueWith(_ => Avalonia.Threading.Dispatcher.UIThread.Post(() => _collectionsTool?.Pane.Refresh()), System.Threading.Tasks.TaskScheduler.Default);

        void ICollectionsHost.RenameCollection(long id) =>
            RenameThisCollectionAsync(_collections.FirstOrDefault(c => c.Id == id)).ContinueWith(_ => Avalonia.Threading.Dispatcher.UIThread.Post(() => _collectionsTool?.Pane.Refresh()), System.Threading.Tasks.TaskScheduler.Default);

        void ICollectionsHost.RemoveCollection(long id) =>
            DeleteThisCollectionAsync(_collections.FirstOrDefault(c => c.Id == id)).ContinueWith(_ => Avalonia.Threading.Dispatcher.UIThread.Post(() => _collectionsTool?.Pane.Refresh()), System.Threading.Tasks.TaskScheduler.Default);

        void ICollectionsHost.AddToCollection(long id, string path)
        {
            _records.AddToCollection(id, path);
            IdentifyLater(path);
            ShowLibraryEntries();
            _collectionsTool?.Pane.Refresh();
        }

        // --- IHomeHost ---

        DateTime IHomeHost.Now => PageClock();

        bool IHomeHost.Desktop => !_bigScreen;

        HomeSuspended? IHomeHost.Suspended =>
            _session is not null && _currentRomPath is string path ? new HomeSuspended(path, RunningGameTitle(), RuntimeCatalog.ShelfDisplayName(EntryOf(path).Shelf), _suspendedAt) : null;

        IReadOnlyList<string> IHomeHost.Recent => _recentSnapshot.Select(e => e.FullPath).ToList();

        GameFacts IHomeHost.Facts(string path) => FactsShown(path);

        // A game's facts with its console as the systems table names it, as every list does (EmuSen_Interface.md §0.21).
        private GameFacts FactsShown(string path) => ((IGamePageHost)this).Facts(path) with { Console = RuntimeCatalog.ShelfDisplayName(EntryOf(path).Shelf) };

        int IHomeHost.GameCount => _allScan.Entries.Count(Listed);

        ToolSummary? IHomeHost.Summary(string place) => SummaryOf(place);

        int IHomeHost.ControllerCount => _gamepad.Pads.Count;

        string? IHomeHost.Battery => DeviceStatusSource().BatteryPercent is int percent ? $"{percent}%" : null;

        string IHomeHost.SettingsLine =>
            $"{(ActualThemeVariant == ThemeVariant.Light ? "Light" : "Dark")} · {(_appSettings.NavigationSounds ? $"{Math.Clamp(_appSettings.BigPictureInterface.NavigationVolume, 0, 100)}% sound" : "sound off")}";

        bool IHomeHost.BigPictureForced => _bigScreenForced;

        void IHomeHost.Resume()
        {
            Cue("launch");
            ToggleLibrary();
        }

        void IHomeHost.Play(string path) => ((IGamesHost)this).Play(path);

        void IHomeHost.OpenPage(string path) => ShowGamePage(path);

        void IHomeHost.ToggleFavourite(string path) => ((IGamesHost)this).ToggleFavourite(path);

        void IHomeHost.Open(string place)
        {
            switch (place)
            {
                case HomeScreen.Games when _bigScreen: ShowGames(); break;
                case HomeScreen.Games: ShowDesktopHome(false); ChooseCollection(AllGamesKey); break;
                case HomeScreen.Search when _bigScreen: ShowGames(search: true); break;
                case HomeScreen.Search: ShowDesktopHome(false); SearchFromThePad(); break;
                case HomeScreen.Scraper: ShowScraper(); break;
                case HomeScreen.Packages: ShowPackages(); break;
                case HomeScreen.Library: ShowLibraryManager(); break;
                case HomeScreen.Cores: ShowCores(); break;
                case HomeScreen.Cheats: ShowCheatDatabase(); break;
                case HomeScreen.Settings when _bigScreen: ShowSettings(); break;
                case HomeScreen.Settings: ShowSettings(); break;
                case HomeScreen.Power: OpenCard(PowerMenu, PowerRows, HomeTitle); break;
                case HomeScreen.BigPicture: SetBigPicture(true); break;
            }
        }

        // --- IGamesHost ---

        DateTime IGamesHost.Now => PageClock();

        // The games the grid sorts, read once per showing of the library.
        IReadOnlyList<GridGame> IGamesHost.Games => _gridGames ??= _allScan.Entries.Where(Listed).Select(e =>
        {
            GameFacts facts = FactsShown(e.FullPath);
            GameRecord? r = _recordSnapshot.GetValueOrDefault(e.FullPath);
            return new GridGame(facts, e.Shelf, r?.PlayCount ?? 0, null);
        }).ToList();

        IReadOnlyList<(string Key, string Label)> IGamesHost.Consoles =>
            RuntimeCatalog.Shelves.Where(s => _allScan.Entries.Any(e => e.Shelf == s.Name && Listed(e))).Select(s => (s.Name, RuntimeCatalog.ShelfDisplayName(s.Name))).ToList();

        IReadOnlyList<GridCollection> IGamesHost.Collections =>
            _collections.Select(c => new GridCollection(c.Id, c.Name, _records.Members(c.Id))).ToList();

        IReadOnlyList<string> IGamesHost.Recent(IReadOnlyList<string> paths, int limit) => _records.Recent(paths.ToList(), limit);

        bool IGamesHost.CanStartConsole => StartableConsoles.Count > 0;

        int IGamesHost.Random(int count) => _shuffle.Next(count);

        void IGamesHost.Play(string path)
        {
            Cue("launch");
            StartGameAsync(path, Path.GetFileName(path)).Logged("launch", $"Starting {Path.GetFileName(path)}");
        }

        void IGamesHost.OpenPage(string path) => ShowGamePage(path);

        void IGamesHost.ToggleFavourite(string path)
        {
            Cue("favorite");
            _records.ToggleFavourite(path);
            IdentifyLater(path);
            ShowLibraryEntries();
        }

        void IGamesHost.Home() => ShowHome();
    }
}
