using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Markup.Xaml.MarkupExtensions;
using Avalonia.Media;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;

namespace EmuSen.Mistress.Views
{
    // One row of Collections: the player's own collection, an automatic one, or the row that makes a new one.
    public sealed record CollectionRow(string Key, string Name, IReadOnlyList<string> Members, long? Id = null, bool Automatic = false, TimeSpan Played = default)
    {
        public bool IsNew => Key == CollectionsPane.NewKey;
    }

    // What Collections asks of the window: the rows, a game's cover, the game in hand, and the edits.
    public interface ICollectionsHost
    {
        IReadOnlyList<CollectionRow> CollectionRows();
        CoverItem CoverFor(string path);
        string? GameInHand { get; }
        void OpenCollection(CollectionRow row);
        void NewCollection();
        void RenameCollection(long id);
        void RemoveCollection(long id);
        void AddToCollection(long id, string path);
    }

    // Collections as a tool on the frame: the player's own and the automatic ones with their counts, the pane with the covers, A opening one as a section of the library - see EmuSen_BigPicture.md §51.4.
    public sealed class CollectionsPane : ManagerLayout<CollectionRow>
    {
        public const string NewKey = "new", Title2 = "Collections";
        public const string Sentence = "A opens it as a section of the library; X renames it, adds games or removes it.";

        private readonly ICollectionsHost _host;
        private readonly LunaAction _open, _add, _rename, _remove, _new;

        public CollectionsPane(ICollectionsHost host)
        {
            _host = host;
            Title = Title2;
            HasSearch = true;
            DetailPlacement = AdaptiveDetailPlacement.Beside;
            ContentShare = 0.6;
            SectionWord = "Collections";
            SearchHint = "Search";
            RowKey = r => r.Key;
            Column(new LunaColumn<CollectionRow>("Collection", r => r.Name) { Tone = r => r.Automatic ? "LunaMuted" : null }, bigScreen: true)
                .Column(new LunaColumn<CollectionRow>("Games", Count) { Width = "Auto", Tone = r => r.Automatic ? "LunaMuted" : null }, bigScreen: true);
            InSection = (r, s) => s switch { "mine" => !r.Automatic, "automatic" => r.Automatic, _ => true };
            SearchKey = r => r.Name;
            DetailFor = Detail;
            _open = new LunaAction("Open", () => { if (Selected is { IsNew: false } r) _host.OpenCollection(r); }) { HelpText = "Opens the collection as a section of the library." };
            _new = new LunaAction("New collection…", _host.NewCollection) { HelpText = "Makes a new collection and names it." };
            _rename = new LunaAction("Rename…", () => { if (Selected?.Id is long id) _host.RenameCollection(id); });
            _remove = new LunaAction("Remove…", () => { if (Selected?.Id is long id) _host.RemoveCollection(id); });
            _add = new LunaAction("Add the selected game", () => { if (Selected?.Id is long id && _host.GameInHand is { } path) _host.AddToCollection(id, path); }) { PadButton = PadGlyphButton.North };
            ActionFor = r => r.IsNew ? _new : _open;
            PageFor = r => r.Id is null ? [] : [_rename, _add, _remove];
            PageTitle = r => r.Name;
            SetToolActions([_open, _add, _rename, _remove, LunaAction.Separator(), _new], bigScreenButtons: []);
            Header = new AdaptiveHeader("COLLECTIONS");
            SetSections([new AdaptiveSection("all", "All"), new AdaptiveSection("mine", "Mine"), new AdaptiveSection("automatic", "Automatic")], "all");
            // Nothing in Collections is marked and applied, so it has no apply bar.
            Status = null;
            Refresh();
        }

        // "6 games", or an automatic one's "Automatic · 4"; nothing on the row that makes one.
        public static string Count(CollectionRow r) => r.IsNew ? "" : r.Automatic ? $"Automatic · {r.Members.Count}" : r.Members.Count == 1 ? "1 game" : $"{r.Members.Count} games";

        public void Refresh()
        {
            IReadOnlyList<CollectionRow> rows = _host.CollectionRows();
            SetRows(rows);
            if (Selected is null && rows.Count > 0) { Table.Select(rows[0]); BigScreenTable.Select(rows[0]); }
            int mine = rows.Count(r => r.Id is not null), games = rows.Where(r => r.Id is not null).SelectMany(r => r.Members).Distinct().Count();
            Header = new AdaptiveHeader("COLLECTIONS", Facts: $"{(mine == 1 ? "1 collection" : $"{mine} collections")} · {(games == 1 ? "1 game" : $"{games} games")}");
            _add.IsEnabled = _host.GameInHand is not null;
        }

        // The collection's name, its count and play time, its first covers, and what the buttons do.
        private Control? Detail(CollectionRow? row)
        {
            if (row is null || row.IsNew) return null;
            var stack = new StackPanel { Spacing = 12 };
            var name = new TextBlock { Text = row.Name, Classes = { PaneTitleClass } };
            stack.Children.Add(name);
            string played = row.Played.TotalHours >= 1 ? $" · {(int)row.Played.TotalHours} h played" : "";
            var facts = new TextBlock { Text = Count(row).Replace("Automatic · ", "") + (row.Automatic ? " games" : "") + played };
            facts[!TextBlock.ForegroundProperty] = new DynamicResourceExtension("LunaMuted");
            stack.Children.Add(facts);
            var covers = new WrapPanel { Orientation = Orientation.Horizontal, ItemSpacing = 12, LineSpacing = 12 };
            foreach (string path in row.Members.Take(6))
                covers.Children.Add(new CoverCard { Big = Form != AdaptiveForm.Desktop, CellWidth = 100, CellHeight = 110, Item = _host.CoverFor(path) });
            stack.Children.Add(covers);
            stack.Children.Add(new TextBlock { Text = row.Automatic ? "A opens it as a section of the library. EmuSen keeps it." : Sentence, TextWrapping = TextWrapping.Wrap });
            return stack;
        }
    }

    // Mistress's thin host for Collections - see EmuSen_BigPicture.md §51.4.
    public sealed class CollectionsWindow : ToolPaneWindow
    {
        public CollectionsWindow(ICollectionsHost host) : this(new CollectionsPane(host)) { }

        private CollectionsWindow(CollectionsPane pane) : base("Collections", pane) => Pane = pane;

        public CollectionsPane Pane { get; }

        protected override void FirstShown() => Pane.Refresh();
    }
}
