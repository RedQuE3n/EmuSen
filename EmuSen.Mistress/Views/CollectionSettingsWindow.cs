using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // What the collection settings sheet asks of the window: Mistress's own collections, and the edits only the window can make.
    public interface ICollectionSettingsHost
    {
        BigPictureCollections Settings { get; }
        IReadOnlyList<GameCollection> Collections { get; }
        string? EditingName { get; }
        Task CreateCollectionAsync(Window owner);
        Task DeleteCollectionAsync(Window owner, GameCollection collection);
        void FinishEditing();
        void CollectionSettingsChanged();
    }

    // ES-DE's Game Collection Settings, and the gamelist entries of its UI settings, over Mistress's own collections - see EmuSen_Settings_Reference.md §4.58.
    public sealed class CollectionSettingsWindow : ToolWindow
    {
        public static readonly (string Value, string Text)[] Grouping =
            [(BigPictureCollections.GroupUnthemed, "If unthemed"), (BigPictureCollections.GroupAlways, "Always"), (BigPictureCollections.GroupNever, "Never")];

        public static readonly (string Value, string Text)[] RandomButton =
            [(BigPictureCollections.RandomGames, "Games only"), (BigPictureCollections.RandomGamesAndSystems, "Games and systems"), (BigPictureCollections.RandomDisabled, "Disabled")];

        private static readonly (string Value, string Text)[] Automatic =
            [(BigPictureCollections.AllGames, "All Games"), (BigPictureCollections.Favorites, "Favorites"), (BigPictureCollections.LastPlayed, "Last Played")];

        private readonly ICollectionSettingsHost _host;
        private readonly StackPanel _page = Ui.Stack(12);

        public CollectionSettingsWindow(ICollectionSettingsHost host)
        {
            _host = host;
            Title = "Game Collection Settings";
            Width = 720;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            Button close = Ui.Button("Close", Close);
            close.Name = "CollectionSettingsClose";
            Control buttons = Ui.Buttons(close).Margin(0, 12, 0, 0);
            DockPanel.SetDock(buttons, Dock.Bottom);
            Content = new DockPanel { LastChildFill = true, Children = { buttons, new ScrollViewer { Content = _page.Margin(4, 4, 4, 4), MaxHeight = 560 } } }.Margin(16);
            Fill();
        }

        private BigPictureCollections S => _host.Settings;

        // Rebuilt after every change, since a created or deleted collection changes the rows below it.
        public void Fill()
        {
            _page.Children.Clear();
            if (_host.EditingName is { } editing)
            {
                Button finish = Ui.Button($"Finish Editing '{editing}' Collection", () => { _host.FinishEditing(); Fill(); });
                finish.Name = "CollectionFinishEditing";
                finish.HorizontalAlignment = HorizontalAlignment.Left;
                _page.Children.Add(finish);
            }

            _page.Children.Add(Ui.Header("Collections"));
            var automatic = Ui.Stack(4);
            foreach ((string value, string text) in Automatic)
                automatic.Children.Add(Switch($"Auto{value}", text, S.AutoCollections.Contains(value), on =>
                {
                    S.AutoCollections.Remove(value);
                    if (on) S.AutoCollections.Add(value);
                }));
            _page.Children.Add(new FieldRow { Label = "Automatic Game Collections", Hint = "Shown in the systems carousel after your systems.", Content = automatic });

            IReadOnlyList<GameCollection> collections = _host.Collections;
            var custom = Ui.Stack(4);
            if (collections.Count == 0) custom.Children.Add(new HintText { Text = "No custom collections yet. Create one below." });
            foreach (GameCollection c in collections)
                custom.Children.Add(Switch($"Custom{c.Id}", c.Name, !S.HiddenCustomCollections.Contains(c.Id), on =>
                {
                    S.HiddenCustomCollections.Remove(c.Id);
                    if (!on) S.HiddenCustomCollections.Add(c.Id);
                }));
            _page.Children.Add(new FieldRow { Label = "Custom Game Collections", Hint = "The same collections as the library's sidebar.", Content = custom });

            Button create = Ui.Button("Create New Custom Collection...", () => _ = CreateAsync());
            create.Name = "CollectionCreate";
            _page.Children.Add(new FieldRow { Label = "Create New Custom Collection", Hint = "Then add games with North in any game list, until you finish editing.", Content = create });

            if (collections.Count > 0)
            {
                var choose = new Dropdown { Name = "CollectionDeleteChoice", HorizontalAlignment = HorizontalAlignment.Stretch };
                choose.Fill(collections.Select(c => c.Name).ToArray(), collections[0].Name);
                Button delete = Ui.Button("Delete...", () => _ = DeleteAsync(collections.FirstOrDefault(c => c.Name == choose.SelectedItem as string)));
                delete.Name = "CollectionDelete";
                _page.Children.Add(new FieldRow { Label = "Delete Custom Collection", Hint = "The games stay where they are; only the list of them goes.", Content = Ui.Row(12, choose, delete) });
            }

            _page.Children.Add(Choice("CollectionGrouping", "Group Custom Collections", "If unthemed puts a collection the theme has no folder for inside Collections.", Grouping,
                S.GroupCustomCollections, v => S.GroupCustomCollections = v));
            _page.Children.Add(new FieldRow
            {
                Label = "Custom Collections",
                Content = Ui.Stack(4,
                    Switch("FavoritesFirstCustom", "Sort favorites on top for custom collections", S.FavoritesFirstCustom, on => S.FavoritesFirstCustom = on),
                    Switch("StarsCustom", "Display star markings for custom collections", S.StarsCustom, on => S.StarsCustom = on)),
            });

            _page.Children.Add(Ui.Header("Game Lists"));
            _page.Children.Add(Choice("DefaultSortOrder", "Game Default Sort Order", "A list's own choice in its options lasts until EmuSen closes.",
                GameSort.All(false).Select(s => (s.Stored, s.Label)).ToArray(), GameSort.Parse(S.DefaultSortOrder).Stored, v => S.DefaultSortOrder = v));
            _page.Children.Add(new FieldRow
            {
                Label = "Favorites",
                Content = Switch("FavoritesFirst", "Sort favorite games above non-favorites", S.FavoritesFirst, on => S.FavoritesFirst = on),
            });
            _page.Children.Add(Choice("RandomEntryButton", "Random Entry Button", "Either thumbstick pressed in jumps to a random game, or system.", RandomButton,
                S.RandomEntryButton, v => S.RandomEntryButton = v));
        }

        private LunaSwitch Switch(string name, string label, bool on, Action<bool> store)
        {
            var box = new LunaSwitch { Name = name, Label = label, IsChecked = on };
            box.IsCheckedChanged += (_, _) =>
            {
                store(box.IsChecked == true);
                _host.CollectionSettingsChanged();
            };
            return box;
        }

        private FieldRow Choice(string name, string label, string hint, (string Value, string Text)[] entries, string current, Action<string> store)
        {
            var dropdown = new Dropdown { Name = name, HorizontalAlignment = HorizontalAlignment.Stretch };
            string[] texts = entries.Select(e => e.Text).ToArray();
            dropdown.Fill(texts, entries.FirstOrDefault(e => e.Value == current, entries[0]).Text);
            dropdown.Chose += chosen =>
            {
                int i = Array.IndexOf(texts, chosen as string);
                if (i < 0) return;
                store(entries[i].Value);
                _host.CollectionSettingsChanged();
            };
            return new FieldRow { Label = label, Hint = hint, Content = dropdown };
        }

        private async Task CreateAsync()
        {
            string? before = _host.EditingName;
            await _host.CreateCollectionAsync(this);
            if (_host.EditingName is not null && _host.EditingName != before) Close();
            else Fill();
        }

        private async Task DeleteAsync(GameCollection? collection)
        {
            if (collection is null) return;
            await _host.DeleteCollectionAsync(this, collection);
            Fill();
        }
    }
}
