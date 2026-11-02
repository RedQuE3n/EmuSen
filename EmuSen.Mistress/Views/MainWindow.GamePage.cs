using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Platform.Storage;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Scraping;
using EmuSen.Tools.Packages;

namespace EmuSen.Mistress.Views
{
    // A game's page over the library, both forms, and everything it reads and asks of the window - see EmuSen_BigPicture.md §50.
    public partial class MainWindow : IGamePageHost
    {
        private GamePageView? _gamePage;
        private GamePackageDetail? _pagePackage;

        // The page shown, for the pad's router and the tests; null when none is.
        public GamePage? GamePageShown => _gamePage?.Model;

        public PageLayout? GamePageLayout => _gamePage?.Layout;

        // Opens a game's page on a section, and on one of its deeper pages where asked; over the whole screen on the big screen, in the library's content area on the desktop.
        public void ShowGamePage(string path, string section = GamePage.Overview, string? deeper = null)
        {
            CloseGamePage(restoreFocus: false);
            // Lettered rings unless the session knows its pad, as the frame's tool screens draw them (LunaP §208.10).
            var layout = new PageLayout { Name = "GamePage", Form = _bigScreen ? AdaptiveForm.BigScreen : AdaptiveForm.Desktop, PadFamily = HelpFamily == PadFamily.Generic ? PadFamily.Xbox : HelpFamily };
            var model = new GamePage(this, path, desktop: !_bigScreen, section);
            _gamePage = new GamePageView(model, layout, p => _covers.Get(p));
            layout.KeyDown += OnGamePageKey;
            _covers.Loaded += OnPagePictureLoaded;
            (_bigScreen ? ScreenContent : LibraryArea).Children.Add(layout);
            StatusBar.IsVisible = !_bigScreen;
            if (deeper == GamePage.EditPage) model.ChooseRow("edit");
            else if (deeper == GamePage.CoverPage) model.ChooseRow("cover");
            else if (deeper == GamePage.ScrapePage) model.ChooseRow("scrape");
            layout.Focus();
        }

        // Back to the library on the same game.
        public void CloseGamePage(bool restoreFocus = true)
        {
            if (_gamePage is not { } page) return;
            _gamePage = null;
            page.Detach();
            page.Layout.KeyDown -= OnGamePageKey;
            _covers.Loaded -= OnPagePictureLoaded;
            if (page.Layout.Parent is Panel parent) parent.Children.Remove(page.Layout);
            _pagePackage?.Stop();
            _pagePackage = null;
            StatusBar.IsVisible = !Sheets.IsScreenShown;
            if (!restoreFocus) return;
            if (ThemedLibraryShown) ThemedLibraryHost.Focus();
            else (GridShown ? (Control)LibraryGrid : LibraryList).Focus();
        }

        // A picture decoded after the page was drawn draws it again.
        private void OnPagePictureLoaded(string path) => _gamePage?.Model.Read();

        // The pad's buttons in the page's grammar: the directions move, A chooses, B goes back, Y favourites, X acts on the thing in hand, Select and Start as declared (EmuSen_Interface.md §2.1).
        private void GamePagePadCommand(UiButton button)
        {
            if (_gamePage?.Layout is not { } page) return;
            switch (button)
            {
                case UiButton.Up: page.Move(0, -1); break;
                case UiButton.Down: page.Move(0, 1); break;
                case UiButton.Left: page.Move(-1, 0); break;
                case UiButton.Right: page.Move(1, 0); break;
                default:
                    if (PageGlyph(button) is { } glyph) page.Press(glyph);
                    break;
            }
        }

        internal static PadGlyphButton? PageGlyph(UiButton button) => button switch
        {
            UiButton.Accept => PadGlyphButton.South,
            UiButton.Back => PadGlyphButton.East,
            UiButton.Screensaver => PadGlyphButton.West,
            UiButton.Search => PadGlyphButton.North,
            UiButton.PageUp => PadGlyphButton.LeftShoulder,
            UiButton.PageDown => PadGlyphButton.RightShoulder,
            UiButton.Menu => PadGlyphButton.Start,
            UiButton.Options => PadGlyphButton.Select,
            _ => null,
        };

        // A keyboard on the big screen steers the page as a pad would; on the desktop the page's own controls take the keys.
        private void OnGamePageKey(object? sender, KeyEventArgs e)
        {
            if (!_bigScreen || e.Handled) return;
            UiButton? button = e.Key switch
            {
                Key.Up => UiButton.Up, Key.Down => UiButton.Down, Key.Left => UiButton.Left, Key.Right => UiButton.Right,
                Key.Enter => UiButton.Accept, Key.Escape or Key.Back => UiButton.Back, Key.PageUp => UiButton.PageUp, Key.PageDown => UiButton.PageDown,
                _ => null,
            };
            if (button is not { } b) return;
            GamePagePadCommand(b);
            e.Handled = true;
        }

        private RomEntry EntryOf(string path) => _allScan.Entries.FirstOrDefault(e => e.FullPath == path) ?? new RomEntry(path);

        // --- IGamePageHost: what the page reads ---

        // The page's clock for how long ago a state was saved; a test fixes it.
        internal static Func<DateTime> PageClock = () => DateTime.Now;

        DateTime IGamePageHost.Now => PageClock();

        bool IGamePageHost.ScrapeRunning => ScrapeRunning;

        int IGamePageHost.StateSlots => StateSlots;

        GameFacts IGamePageHost.Facts(string path)
        {
            RomEntry entry = EntryOf(path);
            GameMetadata m = MetadataFor(path);
            GameRecord? record = _records.Find(path);
            var descriptor = RuntimeCatalog.ByDisplayName(entry.CoreDisplayName);
            string console = RuntimeCatalog.ShelfByName(entry.Shelf)?.Label ?? entry.CoreDisplayName;
            return new GameFacts(path, DisplayTitle(entry), console, descriptor?.Console, m.Released?.Year.ToString(System.Globalization.CultureInfo.InvariantCulture),
                m.DeveloperText, m.PublisherText, TimeSpan.FromSeconds(record?.PlaySeconds ?? 0), record?.LastPlayed, m.DescriptionText, record?.Favourite == true,
                descriptor?.CoverAspect ?? 0.75);
        }

        GameArt IGamePageHost.Art(string path)
        {
            RomEntry entry = EntryOf(path);
            string? system = SystemOf(entry);
            string? Find(string type) => system is null ? null : MediaSourcesNow().Locate(system, path, type).Path;
            string? shot = Find(ScrapeRules.Screenshot.EsdeType);
            return new GameArt(CoverPathFor(entry), Find(ScrapeRules.FanArt.EsdeType) ?? shot ?? Find(ScrapeRules.TitleScreen.EsdeType), shot, Find(ScrapeRules.Video.EsdeType));
        }

        IReadOnlyList<GameState> IGamePageHost.States(string path)
        {
            string? package = GamePackages.RootFor(path);
            string? key = GamePackage.SaveKeyAt(package);
            var states = new List<GameState>();
            void Add(string file, string slot) { if (File.Exists(file)) states.Add(new GameState(file, slot, File.Exists(SaveLibrary.PicturePathFor(file)) ? SaveLibrary.PicturePathFor(file) : null, File.GetLastWriteTime(file))); }
            for (int n = 1; n <= StateSlots; n++) Add(SaveLibrary.StatePathFor(path, n, _appSettings.StateDirectory, package, key), $"Slot {n}");
            Add(SaveLibrary.ResumeStatePathFor(path, _appSettings.StateDirectory, package, key), GamePage.AutoSlot);
            return states.OrderByDescending(s => s.When).ToList();
        }

        IReadOnlyList<GameMedia> IGamePageHost.Media(string path)
        {
            RomEntry entry = EntryOf(path);
            var media = MediaLibrary.Screenshots(DataStore.Screenshots, [entry]).Where(m => m.Game?.FullPath == path)
                .Select(m => new GameMedia(m.Path, GameMediaKind.Taken, m.When)).ToList();
            if (SystemOf(entry) is string system)
                foreach ((string type, GameMediaKind kind) in new[] { (ScrapeRules.Screenshot.EsdeType, GameMediaKind.Screenshot), (ScrapeRules.TitleScreen.EsdeType, GameMediaKind.TitleScreen), (ScrapeRules.Video.EsdeType, GameMediaKind.Clip), (ScrapeRules.Marquee.EsdeType, GameMediaKind.Marquee), (ScrapeRules.FanArt.EsdeType, GameMediaKind.Fanart) })
                    if (MediaSourcesNow().Locate(system, path, type).Path is string file && File.Exists(file)) media.Add(new GameMedia(file, kind, File.GetLastWriteTime(file)));
            return media;
        }

        (int Width, int Height)? IGamePageHost.PictureSize(string path) => _covers.Get(path) is { } b ? (b.PixelSize.Width, b.PixelSize.Height) : null;

        IReadOnlyList<GameChoice> IGamePageHost.Engines(string path)
        {
            if (RuntimeCatalog.ConsoleForRom(path) is not { } console) return [];
            IReadOnlyList<string> engines = RuntimeCatalog.EngineChoices(console);
            string? own = RuntimeCatalog.SettingsFor(console).FirstOrDefault(r => r.Key == RuntimeCatalog.EngineKey)?.Default;
            // One engine runs the console: the row names it and steps nothing.
            if (engines.Count == 0) return [new GameChoice(null, own is { Length: > 0 } ? $"{own} (the only one)" : "The console's own")];
            return engines.Select(e => new GameChoice(e, e == own ? $"{e} (the console's)" : e)).Prepend(new GameChoice(null, own is { Length: > 0 } ? $"{own} (default)" : "The console's engine")).ToList();
        }

        string? IGamePageHost.Engine(string path) => MetadataFor(path).AltEmulatorEngine;

        void IGamePageHost.SetEngine(string path, string? engine) => ((IGamePageHost)this).EditField(path, GameMetadata.AltEmulator, engine);

        IReadOnlyList<GameChoice>? IGamePageHost.Standards(string path) => StandardOffered(path) ? StandardChoices.Select(c => new GameChoice(c.Value, c.Text)).ToList() : null;

        string? IGamePageHost.Standard(string path) => _records.StandardChoice(path);

        void IGamePageHost.SetStandard(string path, string? standard) => SetStandard(path, standard);

        string? IGamePageHost.Crop(string path) => CropOffered(path) ? (_records.Crop(path) is null ? "Console's" : "This game's") : null;

        void IGamePageHost.OpenCrop(string path, string title) => ShowGameCrop(path, title);

        // The running game's cheats live; another game's as saved, where it has any.
        IReadOnlyList<GameCheat>? IGamePageHost.Cheats(string path)
        {
            if (IsRunning(path)) return _cheats.Cheats.Select(c => new GameCheat(c.Id, c.Description, c.Enabled)).ToList();
            return CheatsFor(path)?.Load() is { Cheats.Count: > 0 } file ? file.Cheats.Select((e, i) => new GameCheat(i, e.Description, e.Enabled)).ToList() : null;
        }

        void IGamePageHost.SetCheat(string path, int id, bool on)
        {
            if (IsRunning(path)) { _cheats.SetEnabled(id, on); _activeCheatsWindow.Current?.Refresh(); return; }
            if (CheatsFor(path) is not { } cheats || cheats.Load() is not { } file || id < 0 || id >= file.Cheats.Count) return;
            file.Cheats[id].Enabled = on;
            cheats.Save(file);
        }

        // A code is read by the running game's console, so one is added only while the game runs.
        bool IGamePageHost.CheatsTakeCodes(string path) => IsRunning(path);

        string IGamePageHost.AddCode(string path, string code) => IsRunning(path) ? _cheats.AddCode(code, "").Words : "Codes are added while the game runs.";

        GamePackageFacts IGamePageHost.Package(string path)
        {
            GamePackageDetail detail = PagePackage(path);
            IReadOnlyList<(string, string)> layers = detail.Layers().Select(l => (l.Name, l.Format)).ToList();
            bool disc = ImageSets.IsSheet(path) || ImageSets.IsPlaylist(path) || path.EndsWith(".chd", StringComparison.OrdinalIgnoreCase);
            string mode = detail.Package is null ? "Not a package" : detail.Refused ? "Changed: plays as a plain game" : detail.SelfContained ? "Self-contained" : "Linked";
            if (detail.Package is not null && layers.Count > 0) mode += layers.Count == 1 ? " · 1 layer" : $" · {layers.Count} layers";
            return new GamePackageFacts(detail.Package is not null, mode, layers, detail.Beside(), detail.CanMake, !detail.Busy && GamePackageDetail.Available, detail.Identity == "patched", !disc, detail.Status);
        }

        // Packages' own controls, kept while the page shows the game, run the package's jobs; the page is drawn again after each.
        private GamePackageDetail PagePackage(string path)
        {
            if (_pagePackage is { } kept && kept.Game == path) return kept;
            _pagePackage?.Stop();
            string? package = GamePackages.RootFor(path);
            string? claimant = package is null && File.Exists(path) ? GamePackages.Index(PackagePlaces.Current()).Claimant(Path.GetFullPath(path))?.Folder : null;
            var detail = new GamePackageDetail(DisplayTitle(EntryOf(path)), path, _appSettings.RomDirectory, package ?? claimant, browse: false, refused: claimant is not null);
            detail.PackageChanged += () => { _gamePage?.Model.Read(); if (!_recordsClosed) RefreshLibrary(); };
            detail.JobEnded += () => _gamePage?.Model.Read();
            return _pagePackage = detail;
        }

        void IGamePageHost.PackageMake(string path) => PagePackage(path).Make();

        void IGamePageHost.PackageAdd(string path, string file) => PagePackage(path).Add(file);

        void IGamePageHost.PackageRemove(string path, int layer) => PagePackage(path).Remove(layer);

        void IGamePageHost.PackageIdentity(string path, bool patched) => PagePackage(path).SetIdentity(patched ? "patched" : "original");

        void IGamePageHost.PackageMigrate(string path) => PagePackage(path).Migrate();

        // Its own cover first, then each listed game's whose cover it may borrow.
        IReadOnlyList<GameCover> IGamePageHost.Covers(string path)
        {
            string? from = CoverChoiceOf(path);
            RomEntry entry = EntryOf(path);
            var covers = new List<GameCover>();
            string? own = EmuSen.Mistress.Library.RuntimeCatalog.ShelfByName(entry.Shelf)?.EsdeSystem is { Length: > 0 } system
                ? MediaSourcesNow().Locate(system, path, ScrapeRules.Cover.EsdeType) is { Source: not MediaSource.OtherGame, Path: { } p } ? p : null
                : HandCoverFor(entry);
            covers.Add(new GameCover(OwnCover, own is null ? "Its own (none yet)" : "Its own", own, from is null));
            foreach (CoverCandidate c in CoverCandidatesFor(path)) covers.Add(new GameCover(c.Path, $"Borrow from {c.Title}", c.Cover, c.Path == from));
            return covers;
        }

        private const string OwnCover = "own";

        void IGamePageHost.ChooseCover(string path, string key)
        {
            if (key == OwnCover) UseOwnCover(path);
            else ChooseCover(path, key);
        }

        IReadOnlyList<GameField> IGamePageHost.Fields(string path)
        {
            MetadataDraft draft = DraftFor(path);
            return GameMetadata.Fields.Select(f => new GameField(f.Key, f.Label, f.Kind, draft.Value(f.Key), ChoicesOf(f, path))).ToList();
        }

        private static readonly IReadOnlyList<GameChoice> FlagChoices = [new(GameMetadata.No, "Off"), new(GameMetadata.Yes, "On")];

        // A field's values where Left and Right step it: a flag's two, a choice's list, a rating's half stars.
        private static IReadOnlyList<GameChoice>? ChoicesOf(MetadataField field, string path) => field.Kind switch
        {
            MetadataKind.Flag => FlagChoices,
            MetadataKind.Choice => GameMetadata.ChoicesFor(field, path).Select(c => new GameChoice(c.Value, c.Text)).ToList(),
            MetadataKind.Rating => Enumerable.Range(0, 11).Select(n => new GameChoice(n == 0 ? null : (n / 10.0).ToString("0.0", System.Globalization.CultureInfo.InvariantCulture), Stars(n))).ToList(),
            _ => null,
        };

        private static string Stars(int halves) => halves == 0 ? "No rating" : $"{halves / 2.0:0.#} of 5";

        // Saved at once, as the big screen's rows are (EmuSen_Interface.md §18.2).
        void IGamePageHost.EditField(string path, string key, string? value)
        {
            MetadataDraft draft = DraftFor(path);
            draft.Set(key, value is { Length: > 0 } ? value : null);
            SaveMetadata(draft);
        }

        void IGamePageHost.ClearMetadata(string path) => ClearMetadata(path);

        void IGamePageHost.ResetField(string path, string key)
        {
            MetadataDraft draft = DraftFor(path);
            draft.Reset(key);
            SaveMetadata(draft);
        }

        string? IGamePageHost.WhyNoSearch => _scrapeClosed ? "Mistress is closing" : NameSearchWhyNot;

        string IGamePageHost.SearchName(string path) => ScrapeRules.SearchName(Path.GetFileName(path));

        private IReadOnlyList<ScrapedGame> _pageMatches = [];

        async Task<GameSearch> IGamePageHost.SearchAsync(string path, string name)
        {
            SearchAnswer answer = await SearchByNameAsync(path, name);
            _pageMatches = answer.Games;
            IReadOnlyList<GameMatch> matches = answer.Games.Select(g => new GameMatch(g.Id.ToString(System.Globalization.CultureInfo.InvariantCulture), MatchLabel(g, path))).ToList();
            string? why = answer.Status is ScrapeStatus.Found or ScrapeStatus.NotFound ? null : answer.Detail ?? answer.Status.ToString();
            return new GameSearch(matches, why);
        }

        // A match as the question form lists it: the name in the file's region, then its year and console (EmuSen_Interface.md §18.2).
        private static string MatchLabel(ScrapedGame g, string path)
        {
            IReadOnlyList<string> regions = ScrapeRules.RegionOrder(Path.GetFileName(path), ScrapeRules.AutomaticRegion, fallback: true);
            string name = ScrapeRules.ChooseText(g.Names, [.. regions, "ss"], fallback: true)?.Text ?? $"Game {g.Id}";
            string? year = ScrapeRules.Date(ScrapeRules.ChooseText(g.Dates, regions, fallback: true)?.Text)?.Year.ToString(System.Globalization.CultureInfo.InvariantCulture);
            string console = RuntimeCatalog.ShelfByName(new RomEntry(path).Shelf)?.Label ?? "";
            string facts = string.Join(", ", new[] { year, console }.Where(p => p is { Length: > 0 }));
            return facts.Length > 0 ? $"{name} ({facts})" : name;
        }

        bool IGamePageHost.ChooseMatch(string path, string key) => _pageMatches.FirstOrDefault(g => g.Id.ToString(System.Globalization.CultureInfo.InvariantCulture) == key) is { } game && ChooseFoundGame(path, game);

        Task<bool> IGamePageHost.ScrapeByFileAsync(string path) => ConfirmAndScrapeAsync(ScrapeScope.ThisGame(path));

        // --- what the page asks the window to do ---

        void IGamePageHost.Play(string path, string? state) => _ = PlayFromPageAsync(path, state);

        // A state starts its game from that instant with no resume question: choosing the state was the answer.
        private async Task PlayFromPageAsync(string path, string? state)
        {
            CloseGamePage(restoreFocus: false);
            if (state is null)
            {
                await StartGameAsync(path, Path.GetFileName(path));
                return;
            }
            if (!await SameGameOrConfirmedAsync(StateRecordsOnApi.Read(Runtime, state), path)) return;
            await PrepareGameAsync(path);
            LoadGame(path, Path.GetFileName(path), resumeFrom: state, reset: false);
        }

        void IGamePageHost.ToggleFavourite(string path)
        {
            _records.ToggleFavourite(path);
            IdentifyLater(path);
            StatusText.Text = _records.IsFavourite(path) ? $"Added {DisplayTitle(EntryOf(path))} to Favourites" : $"Removed {DisplayTitle(EntryOf(path))} from Favourites";
            ShowLibraryEntries();
        }

        void IGamePageHost.OpenFolder(string path)
        {
            if (Path.GetDirectoryName(path) is { } folder && Directory.Exists(folder)) _ = Launcher.LaunchDirectoryInfoAsync(new DirectoryInfo(folder));
        }

        void IGamePageHost.Hide(string path)
        {
            HideGame(path);
            CloseGamePage();
        }

        bool IGamePageHost.DeleteState(string path, string state)
        {
            try
            {
                File.Delete(state);
                string picture = SaveLibrary.PicturePathFor(state);
                if (File.Exists(picture)) { File.Delete(picture); _covers.Forget(picture); }
                try { Runtime.StateForget(state); }
                catch (EmuSen.Crystal.Client.CrystalException) { }
                File.Delete(StateRecord.SidecarPathFor(state));
                StatusText.Text = $"Deleted a save state of {DisplayTitle(EntryOf(path))}";
                return true;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                ErrorLog.Error("media", "Could not delete", ex, state);
                StatusText.Text = $"Could not delete: {ex.Message}";
                return false;
            }
        }

        bool IGamePageHost.DeleteMedia(string path, string media)
        {
            try
            {
                File.Delete(media);
                _covers.Forget(media);
                return true;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                ErrorLog.Error("media", "Could not delete", ex, media);
                return false;
            }
        }

        // A question in the question form on the big screen, the platform's dialog on the desktop; a test answers it instead.
        internal static Func<MainWindow, string, string, string, Task<bool>> PageConfirm = (owner, title, message, yes) => Dialogs.ConfirmAsync(owner, title, message, yes, "No");

        Task<bool> IGamePageHost.ConfirmAsync(string title, string message, string yes) => PageConfirm(this, title, message, yes);

        // Text typed through the keyboard the player uses; a test types instead.
        internal static Func<MainWindow, string, string?, Task<string?>> PageType = (owner, title, text) => Dialogs.PromptAsync(owner, title, "", text ?? "");

        Task<string?> IGamePageHost.TypeAsync(string title, string? text) => PageType(this, title, text);

        void IGamePageHost.Close() => CloseGamePage();
    }
}
