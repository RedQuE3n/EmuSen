using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views.Covers;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's other two categories, Save States and Screenshots, under the same sidebar - see EmuSen_Settings_Reference.md §4.35.
    public partial class MainWindow
    {
        public const string LibraryCategory = "Library", StatesCategory = "Save States", ScreenshotsCategory = "Screenshots";

        private readonly ActionGroup _categories = new();
        private LunaAction? _libraryCategory, _statesCategory, _screenshotsCategory;
        private string _category = LibraryCategory;
        private IReadOnlyList<MediaItem> _shownMedia = Array.Empty<MediaItem>();
        private LunaAction _playMedia = null!, _deleteMedia = null!, _mediaProvenance = null!;

        private string StateDirectory => string.IsNullOrWhiteSpace(_appSettings.StateDirectory) ? DataStore.SaveStates : _appSettings.StateDirectory;

        private void SetUpMediaViews()
        {
            _libraryCategory = _categories.Add(new LunaAction("_Library", () => ShowCategory(LibraryCategory)));
            _statesCategory = _categories.Add(new LunaAction("Save S_tates", () => ShowCategory(StatesCategory)));
            _screenshotsCategory = _categories.Add(new LunaAction("Scree_nshots", () => ShowCategory(ScreenshotsCategory)));
            _libraryCategory.IsChecked = true;
            LibraryCategoryButtons.ItemsSource = new Control[] { new ActionToggle(_libraryCategory), new ActionToggle(_statesCategory), new ActionToggle(_screenshotsCategory) };

            _playMedia = new LunaAction("_Play Save State", () => _ = PlayMediaAsync());
            _deleteMedia = new LunaAction("_Delete Save State", () => _ = DeleteMediaAsync());
            _mediaProvenance = new LunaAction("") { IsEnabled = false };
            MediaGrid.Key = m => m.Path;
            MediaGrid.Label = m => $"{m.Title}, {m.Label}";
            MediaGrid.CreateTile = () => new CoverTile();
            MediaGrid.BindTile = BindMedia;
            MediaGrid.Activated += item => { _ = PlayMediaAsync(); };
            MediaGrid.ContextMenu = Menus.Context(_mediaProvenance, LunaAction.Separator(), _playMedia, LunaAction.Separator(), _deleteMedia);
            MediaGrid.ContextMenu.Opening += (_, _) =>
            {
                bool any = MediaGrid.Selected is not null;
                bool state = MediaGrid.Selected?.Kind == MediaKind.SaveState;
                _playMedia.Text = state ? "_Play Save State" : "_Open Screenshot";
                _playMedia.IsEnabled = any && (!state || MediaGrid.Selected!.Game is not null);
                _deleteMedia.Text = state ? "_Delete Save State" : "_Delete Screenshot";
                _deleteMedia.IsEnabled = any;
                StateRecord? record = MediaGrid.Selected?.Record;
                _mediaProvenance.Text = record is null
                    ? state ? "No record of who saved this state" : MediaGrid.Selected?.Title ?? ""
                    : $"Saved by {record.Core} (state version {record.StateVersion}), {record.Build}";
            };
            _covers.Loaded += _ => MediaGrid.Refresh(_shownMedia);
        }

        private bool MediaShown => _category != LibraryCategory;

        private void ShowCategory(string category)
        {
            _category = category;
            ShowLibraryEntries();
        }

        // The sidebar's console and collection narrow these by the game they belong to, as they narrow the library.
        private void ShowMedia(IReadOnlyList<RomEntry> games, string search)
        {
            IReadOnlyList<MediaItem> all = _category == StatesCategory
                ? MediaLibrary.SaveStates(StateDirectory, _allScan.Entries, _records.PathByHash)
                : MediaLibrary.Screenshots(DataStore.Screenshots, _allScan.Entries);
            var inView = new HashSet<string>(games.Select(g => g.FullPath), StringComparer.Ordinal);
            bool everything = SelectedConsole == EmuSen.Cores.CoreCatalog.AllConsoles && CollectionKey == AllGamesKey;
            _shownMedia = all.Where(m => (everything || m.Game is not null && inView.Contains(m.Game.FullPath))
                                          && (string.IsNullOrWhiteSpace(search) || FilterBar.Matches(search, m.Title))).ToList();
            MediaGrid.Refresh(_shownMedia);
            if (_shownMedia.Count > 0 && MediaGrid.Selected is null) MediaGrid.Select(_shownMedia[0]);

            string noun = _category == StatesCategory ? "save state" : "screenshot";
            LibraryHeaderText.Text = _shownMedia.Count > 0
                ? $"{_shownMedia.Count} {noun}{(_shownMedia.Count == 1 ? "" : "s")}"
                : _category == StatesCategory
                    ? "No save states. Mistress writes one each time a game is left, and F5 saves one to the current slot."
                    : $"No screenshots. Press {HotkeyName(Input.HotkeyAction.Screenshot)} in a game, or choose Take a Screenshot from its options.";
        }

        private string HotkeyName(Input.HotkeyAction action) =>
            _hotkeyBindings.ActionToKey.TryGetValue(action, out Avalonia.Input.Key key) ? key.ToString() : action.ToString();

        private void BindMedia(Control tile, MediaItem item)
        {
            var descriptor = item.Game is null ? null : EmuSen.Cores.CoreCatalog.ByDisplayName(item.Game.CoreDisplayName);
            string subtitle = item.Kind == MediaKind.SaveState ? $"{item.Label}  ·  {item.When:d MMM, HH:mm}" : item.Label;
            ((CoverTile)tile).Show(ArtworkIndex.Untagged(item.Title), subtitle, descriptor?.Console ?? "", 0.75,
                item.PicturePath is null ? null : _covers.Get(item.PicturePath), favourite: false);
        }

        // A state starts its game from that instant, with no resume question: choosing the state was the answer.
        private async Task PlayMediaAsync()
        {
            if (MediaGrid.Selected is not MediaItem item) return;
            if (item.Kind == MediaKind.Screenshot)
            {
                new ScreenshotWindow(item.Path, $"{item.Title}, {item.Label}").Show(this);
                return;
            }
            if (item.Game is not RomEntry game)
            {
                StatusText.Text = $"{item.GameStem} is not in the library, so its state has no game to load into.";
                return;
            }
            if (Refusal(item.Record, game.FullPath, null) is string refused)
            {
                StatusText.Text = refused;
                return;
            }
            if (!await SameGameOrConfirmedAsync(item.Record, game.FullPath)) return;
            await PromptForMissingFirmwareAsync(game.FullPath);
            LoadGame(game.FullPath, game.FileName, resumeFrom: item.Path, reset: false);
        }

        private async Task DeleteMediaAsync()
        {
            if (MediaGrid.Selected is not MediaItem item) return;
            string what = item.Kind == MediaKind.SaveState ? $"the save state \"{item.Label}\" of {item.Title}" : $"this screenshot of {item.Title}";
            if (!await Dialogs.ConfirmAsync(this, "Delete", $"Delete {what}? This cannot be undone.", "Delete", "Cancel")) return;
            try
            {
                File.Delete(item.Path);
                if (item.Kind == MediaKind.SaveState && item.PicturePath is string picture) File.Delete(picture);
                if (item.Kind == MediaKind.SaveState) File.Delete(StateRecord.PathFor(item.Path));
                if (item.PicturePath is string shown) _covers.Forget(shown);
                StatusText.Text = $"Deleted {what}";
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                StatusText.Text = $"Could not delete: {ex.Message}";
            }
            ShowLibraryEntries();
        }
    }
}
