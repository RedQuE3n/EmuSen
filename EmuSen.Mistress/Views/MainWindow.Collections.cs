using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // The player's own collections, as OpenEmu's sidebar has them beneath the consoles - see EmuSen_Settings_Reference.md §4.38.
    public partial class MainWindow
    {
        public const string CollectionKeyPrefix = "collection:", NewCollectionKey = "new-collection";

        private IReadOnlyList<GameCollection> _collections = Array.Empty<GameCollection>();
        private LunaAction _newCollection = null!, _renameCollection = null!, _deleteCollection = null!;

        private static string KeyOf(GameCollection collection) => CollectionKeyPrefix + collection.Id;

        private GameCollection? ShownCollection =>
            _appSettings.LibraryCollection.StartsWith(CollectionKeyPrefix, StringComparison.Ordinal)
                ? _collections.FirstOrDefault(c => KeyOf(c) == _appSettings.LibraryCollection)
                : null;

        private SourceListGroup CollectionsGroup() => new("Collections",
            _collections.Select(c => new SourceListItem(KeyOf(c), c.Name, c.Count.ToString()))
                .Append(new SourceListItem(NewCollectionKey, "New Collection...")).ToArray());

        private ContextMenu SidebarContextMenu()
        {
            ContextMenu menu = Menus.Context(_newCollection, LunaAction.Separator(), _renameCollection, _deleteCollection);
            menu.Opening += (_, _) => _renameCollection.IsEnabled = _deleteCollection.IsEnabled = ShownCollection is not null;
            return menu;
        }

        // Null when cancelled or when the name is taken, which the status line says.
        private async Task<GameCollection?> NewCollectionAsync(RomEntry? first)
        {
            if (await Dialogs.PromptAsync(this, "New Collection", "Name the new collection", "", "Create") is not string name) return null;
            if (_records.CreateCollection(name, DateTime.Now) is not long id)
            {
                StatusText.Text = $"There is already a collection called {name}.";
                return null;
            }
            if (first is not null) AddToCollection(id, first);
            StatusText.Text = first is null ? $"Created {name}" : $"Created {name} with {first.Title} in it";
            ShowLibraryEntries();
            return _collections.FirstOrDefault(c => c.Id == id);
        }

        private async Task RenameCollectionAsync()
        {
            if (ShownCollection is not GameCollection collection) return;
            if (await Dialogs.PromptAsync(this, "Rename Collection", $"A new name for {collection.Name}", collection.Name, "Rename") is not string name) return;
            StatusText.Text = _records.RenameCollection(collection.Id, name) ? $"Renamed {collection.Name} to {name}" : $"There is already a collection called {name}.";
            ShowLibraryEntries();
        }

        // The games stay where they are; only the list of them goes.
        private async Task DeleteCollectionAsync()
        {
            if (ShownCollection is not GameCollection collection) return;
            if (!await Dialogs.ConfirmAsync(this, "Delete Collection", $"Delete the collection {collection.Name}? The games in it are not touched.", "Delete", "Cancel")) return;
            _records.DeleteCollection(collection.Id);
            _appSettings.LibraryCollection = AllGamesKey;
            _appSettings.Save();
            StatusText.Text = $"Deleted the collection {collection.Name}";
            ShowLibraryEntries();
        }

        private void AddToCollection(long id, RomEntry entry)
        {
            _records.AddToCollection(id, entry.FullPath);
            IdentifyLater(entry.FullPath);
        }

        private void ToggleMembership(GameCollection collection, RomEntry entry)
        {
            if (_records.CollectionsOf(entry.FullPath).Contains(collection.Id))
            {
                _records.RemoveFromCollection(collection.Id, entry.FullPath);
                StatusText.Text = $"Removed {entry.Title} from {collection.Name}";
            }
            else
            {
                AddToCollection(collection.Id, entry);
                StatusText.Text = $"Added {entry.Title} to {collection.Name}";
            }
            ShowLibraryEntries();
        }

        // Built as the menu opens, since the collections and the game under it both change between openings.
        private LunaAction CollectionsSubmenu(RomEntry? entry)
        {
            var actions = new List<LunaAction> { new("_New Collection...", () => _ = NewCollectionAsync(entry)) };
            if (_collections.Count > 0) actions.Add(LunaAction.Separator());
            IReadOnlySet<long> member = entry is null ? new HashSet<long>() : _records.CollectionsOf(entry.FullPath);
            foreach (GameCollection collection in _collections)
            {
                GameCollection captured = collection;
                actions.Add(new LunaAction(collection.Name.Replace("_", "__"), () => { if (entry is not null) ToggleMembership(captured, entry); })
                {
                    IsCheckable = true,
                    IsChecked = member.Contains(collection.Id),
                    IsEnabled = entry is not null,
                });
            }
            return new LunaAction("Add to C_ollection", () => { }) { Submenu = new LunaMenu("Add to C_ollection", actions), IsEnabled = entry is not null };
        }
    }
}
