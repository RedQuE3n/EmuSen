using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views.Covers;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's library: a sidebar of collections and consoles beside a grid of covers or the list - see EmuSen_Settings_Reference.md §4.33.
    public partial class MainWindow
    {
        public const string AllGamesKey = "all", FavouritesKey = "favourites", RecentKey = "recent", ConsoleKeyPrefix = "console:";

        // OpenEmu's "Recently Added" holds thirty; this is the same list by play instead of by import.
        public const int RecentLimit = 30;

        public const double MinimumTileScale = 0.5, MaximumTileScale = 2.5;

        private RomLibraryResult _allScan = new(RomLibraryStatus.NoDirectoryConfigured, null, Array.Empty<RomEntry>());
        private IReadOnlyDictionary<string, GameRecord> _recordSnapshot = new Dictionary<string, GameRecord>();
        private ArtworkIndex _artwork = ArtworkIndex.Empty;
        private readonly CoverArtCache _covers = new();
        private IReadOnlyList<RomEntry> _shownEntries = Array.Empty<RomEntry>();

        private readonly ActionGroup _views = new();
        private LunaAction? _asGrid, _asList;

        private string ArtworkDirectory =>
            string.IsNullOrWhiteSpace(_appSettings.ArtworkDirectory) ? DataStore.Artwork : _appSettings.ArtworkDirectory;

        private bool GridShown => _appSettings.LibraryView != AppSettings.LibraryList;

        private void SetUpLibraryScreen()
        {
            _asGrid = _views.Add(new LunaAction("as _Grid", () => ShowLibraryAs(AppSettings.LibraryGrid)) { HelpText = "Show the library as covers" });
            _asList = _views.Add(new LunaAction("as _List", () => ShowLibraryAs(AppSettings.LibraryList)) { HelpText = "Show the library as a list" });
            LibraryViewButtons.ItemsSource = new Control[] { new ActionToggle(_asGrid), new ActionToggle(_asList) };

            LibraryGrid.Key = e => e.FullPath;
            LibraryGrid.Label = e => e.Title;
            LibraryGrid.CreateTile = () => new CoverTile();
            LibraryGrid.BindTile = BindCover;
            LibraryGrid.Chose += entry => LibraryList.Select(entry);
            LibraryGrid.Activated += _ => LaunchSelectedLibraryEntry();
            LibraryList.SelectionChanged += (_, _) => { if (LibraryList.Selected is RomEntry entry) LibraryGrid.Select(entry); };
            _covers.Loaded += _ => LibraryGrid.Refresh(_shownEntries);

            LibrarySidebar.Chose += ChooseCollection;
            _newCollection = new LunaAction("_New Collection...", () => _ = NewCollectionAsync(null));
            _renameCollection = new LunaAction("_Rename Collection...", () => _ = RenameCollectionAsync());
            _deleteCollection = new LunaAction("_Delete Collection...", () => _ = DeleteCollectionAsync());
            LibrarySidebar.ContextMenu = SidebarContextMenu();
            TileScale.Minimum = MinimumTileScale;
            TileScale.Maximum = MaximumTileScale;
            TileScale.Value = Math.Clamp(_appSettings.LibraryTileScale, MinimumTileScale, MaximumTileScale);
            TileScale.ValueChanged += (_, e) => ScaleTiles(e.NewValue);
            ScaleTiles(TileScale.Value);

            SetUpMediaViews();
            LibraryGrid.ContextMenu = LibraryContextMenu();
            LibraryList.ContextMenu = LibraryContextMenu();
            ApplyLibraryView();
            ScanArtwork();
            ApplyOnlineCovers();
            ApplyScraping();
        }

        private void ShowLibraryAs(string view)
        {
            _appSettings.LibraryView = view;
            _appSettings.Save();
            ApplyLibraryView();
        }

        private void ApplyLibraryView()
        {
            if (_asGrid is null || _asList is null) return;
            _asGrid.IsChecked = GridShown;
            _asList.IsChecked = !GridShown;
            bool any = _shownEntries.Count > 0;
            LibraryGrid.IsVisible = !MediaShown && GridShown && any;
            LibraryList.IsVisible = !MediaShown && !GridShown && any;
            MediaGrid.IsVisible = MediaShown && _shownMedia.Count > 0;
            _asGrid.IsEnabled = _asList.IsEnabled = !MediaShown;
            TileScale.IsEnabled = GridShown || MediaShown;
        }

        private void ScaleTiles(double scale)
        {
            LibraryGrid.TileWidth = Math.Round(150 * scale);
            LibraryGrid.TileHeight = Math.Round(150 * scale) + CoverTile.TitleBand;
            LibraryGrid.Spacing = Math.Round(26 * scale);
            MediaGrid.TileWidth = LibraryGrid.TileWidth;
            MediaGrid.TileHeight = LibraryGrid.TileHeight;
            MediaGrid.Spacing = LibraryGrid.Spacing;
            _covers.DecodeWidth = (int)Math.Clamp(Math.Round(300 * scale), 160, 760);
            if (Math.Abs(_appSettings.LibraryTileScale - scale) < 0.001) return;
            _appSettings.LibraryTileScale = scale;
            _appSettings.Save();
        }

        // Off the UI thread: a full libretro-thumbnails tree is tens of thousands of files.
        private void ScanArtwork()
        {
            string directory = ArtworkDirectory;
            Task.Run(() => ArtworkIndex.Scan(directory, EmuSen.Cores.CoreCatalog.Cores)).ContinueWith(scan =>
            {
                if (!scan.IsCompletedSuccessfully) return;
                Dispatcher.UIThread.Post(() =>
                {
                    _artwork = scan.Result;
                    LibraryGrid.Refresh(_shownEntries);
                });
            });
        }

        // The cover the library shows, by the order of §4.60: the player's own, ScreenScraper's, an ES-DE folder's, OpenEmu's.
        private string? CoverPathFor(RomEntry entry) =>
            EmuSen.Cores.CoreCatalog.ShelfByName(entry.Shelf)?.EsdeSystem is { Length: > 0 } system ? MediaSourcesNow().Cover(system, entry.FullPath) : HandCoverFor(entry);

        // Only what the player placed in the art folder, or added with Add Cover Art.
        private string? HandCoverFor(RomEntry entry) =>
            _artwork.Find(EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName)?.Console, entry.Title);

        private void BindCover(Control tile, RomEntry entry)
        {
            var descriptor = EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName);
            string console = descriptor?.Console ?? "";
            string? cover = CoverPathFor(entry);
            string title = ArtworkIndex.Untagged(entry.Title);
            string tags = entry.Title.Length > title.Length ? entry.Title[title.Length..].Trim() : "";
            string subtitle = MixedConsoles ? (tags.Length > 0 ? $"{console}  ·  {FirstTag(tags)}" : console) : FirstTag(tags);
            if (cover is null) CoverWanted(entry, descriptor);
            ((CoverTile)tile).Show(title, subtitle, console, descriptor?.CoverAspect ?? 1.365,
                cover is null ? null : _covers.Get(cover), _recordSnapshot.TryGetValue(entry.FullPath, out GameRecord? r) && r.Favourite);
        }

        private static string FirstTag(string tags)
        {
            int open = tags.IndexOfAny(new[] { '(', '[' }), close = tags.IndexOfAny(new[] { ')', ']' });
            return open >= 0 && close > open ? tags[(open + 1)..close] : tags;
        }

        // Set once per showing, since every tile asks.
        private bool MixedConsoles { get; set; }

        private string CollectionKey =>
            _appSettings.LibraryCollection is FavouritesKey or RecentKey ? _appSettings.LibraryCollection
            : ShownCollection is GameCollection collection ? KeyOf(collection)
            : AllGamesKey;

        // The console narrows first, the collection second, the search box last.
        private IReadOnlyList<RomEntry> InCollection(IReadOnlyList<RomEntry> entries) => CollectionKey switch
        {
            FavouritesKey => entries.Where(e => _recordSnapshot.TryGetValue(e.FullPath, out GameRecord? r) && r.Favourite).ToList(),
            RecentKey => entries.Where(e => _recordSnapshot.TryGetValue(e.FullPath, out GameRecord? r) && r.LastPlayed is not null)
                .OrderByDescending(e => _recordSnapshot[e.FullPath].LastPlayed).Take(RecentLimit).ToList(),
            _ when ShownCollection is GameCollection collection && _records.Members(collection.Id) is var members =>
                entries.Where(e => members.Contains(e.FullPath)).ToList(),
            _ => entries,
        };

        private string SidebarKey =>
            CollectionKey != AllGamesKey ? CollectionKey
            : SelectedConsole == EmuSen.Cores.CoreCatalog.AllConsoles ? AllGamesKey
            : ConsoleKeyPrefix + SelectedConsole;

        private void FillSidebar()
        {
            int favourites = _allScan.Entries.Count(e => _recordSnapshot.TryGetValue(e.FullPath, out GameRecord? r) && r.Favourite);
            int recent = Math.Min(RecentLimit, _allScan.Entries.Count(e => _recordSnapshot.TryGetValue(e.FullPath, out GameRecord? r) && r.LastPlayed is not null));
            var library = new SourceListGroup("Library", new[]
            {
                new SourceListItem(AllGamesKey, "All Games", _allScan.Entries.Count.ToString()),
                new SourceListItem(FavouritesKey, "Favourites", favourites.ToString()),
                new SourceListItem(RecentKey, "Recently Played", recent.ToString()),
            });
            var consoles = new SourceListGroup("Consoles", EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder
                .Select(s => new SourceListItem(ConsoleKeyPrefix + s.Name, s.Label, _allScan.Entries.Count(e => e.Shelf == s.Name).ToString()))
                .ToArray());
            LibrarySidebar.Fill(new[] { library, consoles, CollectionsGroup() }, SidebarKey);
        }

        // A collection spans every console, and a console is all of its games, as OpenEmu's sidebar has it.
        private void ChooseCollection(string key)
        {
            // The last row is an action, as OpenEmu's "+" is, and leaves the selection where it was.
            if (key == NewCollectionKey)
            {
                FillSidebar();
                _ = NewCollectionAsync(null);
                return;
            }

            string console = key.StartsWith(ConsoleKeyPrefix, StringComparison.Ordinal) ? key[ConsoleKeyPrefix.Length..] : EmuSen.Cores.CoreCatalog.AllConsoles;
            _appSettings.LibraryCollection = key.StartsWith(ConsoleKeyPrefix, StringComparison.Ordinal) ? AllGamesKey : key;
            if (console != SelectedConsole)
            {
                LibraryFilter.SetFacets(EmuSen.Cores.CoreCatalog.FilterChoices, console);
                OnLibraryFilterChanged();
            }
            else
            {
                _appSettings.Save();
                ShowLibraryEntries();
            }
        }

        private RomEntry? SelectedLibraryEntry => LibraryList.Selected;

        // The search the last showing was filtered by; a different one starts again at the top match.
        private string? _lastLibrarySearch;

        private string DescribeEmptyCollection() => CollectionKey switch
        {
            FavouritesKey => "No favourites yet. Right-click a game and choose Add to Favourites, or press Select on a pad.",
            RecentKey => "Nothing played yet. Games appear here once they have been started.",
            _ when ShownCollection is GameCollection collection => $"Nothing in {collection.Name} yet. Right-click a game and choose Add to Collection.",
            _ => RomLibrary.DescribeEmpty(_libraryScan),
        };

        // From LunaActions, like every other menu here, and relabelled as it opens for the game under it.
        private ContextMenu LibraryContextMenu()
        {
            var play = new LunaAction("_Play Game", LaunchSelectedLibraryEntry);
            var restart = new LunaAction("Play from the _Start", () => { if (SelectedLibraryEntry is RomEntry e) _ = StartFreshAsync(e); });
            var favourite = new LunaAction("Add to _Favourites", () => { if (SelectedLibraryEntry is RomEntry e) ToggleFavourite(e); });
            var addArt = new LunaAction("Add _Cover Art from File...", () => { if (SelectedLibraryEntry is RomEntry e) _ = AddCoverArtAsync(e); });
            var removeArt = new LunaAction("_Remove Cover Art", () => { if (SelectedLibraryEntry is RomEntry e) RemoveCoverArt(e); });
            var lookUp = new LunaAction("_Look Up Cover Online", () => { if (SelectedLibraryEntry is RomEntry e) LookUpCoverAgain(e); });

            var leave = new LunaAction("Remove from This Collection", () =>
            {
                if (SelectedLibraryEntry is RomEntry e && ShownCollection is GameCollection c) ToggleMembership(c, e);
            });

            ContextMenu menu = Menus.Context(play, restart, LunaAction.Separator(), favourite, LunaAction.Separator(), addArt, removeArt);
            menu.Opening += (_, _) =>
            {
                RomEntry? entry = SelectedLibraryEntry;
                var actions = new List<LunaAction> { play, restart, LunaAction.Separator(), favourite, CollectionsSubmenu(entry) };
                if (ShownCollection is GameCollection shown)
                {
                    leave.Text = $"Remove from {shown.Name.Replace("_", "__")}";
                    actions.Add(leave);
                }
                actions.AddRange(new[] { LunaAction.Separator(), addArt, removeArt });
                if (_appSettings.OpenEmuFallback)
                {
                    lookUp.IsEnabled = entry is not null && CoverPathFor(entry) is null;
                    actions.Add(lookUp);
                }
                menu.ItemsSource = Menus.Items(actions);
                bool any = entry is not null;
                play.IsEnabled = favourite.IsEnabled = addArt.IsEnabled = any;
                restart.IsEnabled = any && File.Exists(ResumeStatePath(entry!.FullPath));
                favourite.Text = any && _records.IsFavourite(entry!.FullPath) ? "Remove from _Favourites" : "Add to _Favourites";
                removeArt.IsEnabled = any && OwnCoverPath(entry!) is not null;
            };
            return menu;
        }

        private async Task StartFreshAsync(RomEntry entry)
        {
            await PromptForMissingFirmwareAsync(entry.FullPath);
            LoadGame(entry.FullPath, entry.FileName, resumeFrom: null, reset: false);
        }

        private void ToggleFavourite(RomEntry entry)
        {
            _records.ToggleFavourite(entry.FullPath);
            IdentifyLater(entry.FullPath);
            bool now = _records.IsFavourite(entry.FullPath);
            StatusText.Text = now ? $"Added {entry.Title} to Favourites" : $"Removed {entry.Title} from Favourites";
            ShowLibraryEntries();
        }

        // Where Add Cover Art puts a picture, found again only if it is the one the index would show.
        private string? OwnCoverPath(RomEntry entry)
        {
            string? console = EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName)?.Console;
            if (console is null || HandCoverFor(entry) is not string shown) return null;
            string folder = Path.GetFullPath(Path.Combine(ArtworkDirectory, console));
            return Path.GetFullPath(shown).StartsWith(folder + Path.DirectorySeparatorChar, StringComparison.Ordinal) ? shown : null;
        }

        // Copied into the art folder under libretro's spelling of the name; the chosen file is only read.
        private async Task AddCoverArtAsync(RomEntry entry)
        {
            var types = new[] { new FilePickerFileType("Images") { Patterns = ArtworkIndex.ImageExtensions.Select(e => "*" + e).ToArray() } };
            if (await Dialogs.PickFileAsync(this, $"Cover art for {entry.Title}", types) is not { } picked) return;
            AddCoverArt(entry, picked.Path);
        }

        private void AddCoverArt(RomEntry entry, string picture)
        {
            string console = EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName)?.Console ?? "Other";
            try
            {
                if (OwnCoverPath(entry) is string old) { File.Delete(old); _covers.Forget(old); }
                string target = ArtworkIndex.PathFor(ArtworkDirectory, console, entry.Title, Path.GetExtension(picture));
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                File.Copy(picture, target, overwrite: true);
                _covers.Forget(target);
                _artwork.Add(console, entry.Title, target);
                StatusText.Text = $"Cover art added for {entry.Title}";
                LibraryGrid.Refresh(_shownEntries);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                StatusText.Text = $"Could not add cover art: {ex.Message}";
            }
        }

        private void RemoveCoverArt(RomEntry entry)
        {
            if (OwnCoverPath(entry) is not string path) return;
            try
            {
                File.Delete(path);
                _covers.Forget(path);
                ScanArtwork();
                StatusText.Text = $"Cover art removed for {entry.Title}";
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                StatusText.Text = $"Could not remove cover art: {ex.Message}";
            }
        }
    }
}
