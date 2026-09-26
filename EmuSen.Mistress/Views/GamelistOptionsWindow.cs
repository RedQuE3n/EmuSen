using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // What the gamelist options sheet starts from: the gamelist's letters, sorts, filters and games, and the collection it can edit (§4.58).
    public sealed record GamelistOptionsModel(
        IReadOnlyList<(string Label, int Index)> Letters, string? Letter, IReadOnlyList<GameSort> Sorts, GameSort Sort,
        GameFilter Filter, IReadOnlyList<SceneGame> Games, CustomCollection? Collection, string? Editing);

    public enum CollectionEdit { None, Start, Finish }

    // What the sheet closed with when it applied: the sort and filters, the letter jumped to if one was chosen, and a collection to start or finish editing.
    public sealed record GamelistOptionsResult(GameSort Sort, GameFilter Filter, string? Letter, CollectionEdit Edit);

    // ES-DE's gamelist options: Jump To, Sort Games By, Filter Gamelist and the custom collection entries; B or Apply applies, Back or Cancel does not - see EmuSen_Settings_Reference.md §4.58.
    public sealed class GamelistOptionsWindow : ToolWindow, IPadDriven
    {
        private readonly GamelistOptionsModel _model;
        private readonly Dropdown _jump = new() { Name = "GamelistJumpTo", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly Dropdown _sort = new() { Name = "GamelistSortBy", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBlock _filterState = new() { Name = "GamelistFilterState" };
        private GameFilter _filter;
        private CollectionEdit _edit;
        private bool _cancelled;

        public GamelistOptionsWindow(GamelistOptionsModel model)
        {
            _model = model;
            _filter = model.Filter;
            Title = "Gamelist Options";
            Width = 640;
            CanResize = false;
            SizeToContent = SizeToContent.Height;

            string[] letters = model.Letters.Select(l => l.Label).ToArray();
            if (letters.Length > 0) _jump.Fill(letters, model.Letter is { } at && letters.Contains(at) ? at : letters[0]);
            _jump.IsEnabled = letters.Length > 0;
            string[] sorts = model.Sorts.Select(s => s.Label).ToArray();
            _sort.Fill(sorts, model.Sort.Label);
            Button filter = Ui.Button("Filter Gamelist...", () => _ = FilterAsync());
            filter.Name = "GamelistFilterButton";
            ShowFilterState();

            var rows = Ui.Stack(12,
                new FieldRow { Label = "Jump To...", Hint = "Only the letters the list holds; a star for the favourites sorted on top.", Content = _jump },
                new FieldRow { Label = "Sort Games By", Hint = "Kept for this list until EmuSen closes. Games without the value go last.", Content = _sort },
                new FieldRow { Label = "Filter Gamelist", Hint = "Kept for this list until EmuSen closes, or until reset.", Content = Ui.Row(12, filter, _filterState) });
            if (model.Collection is { } collection && model.Editing != collection.Name)
                rows.Children.Add(EditButton("GamelistEditCollection", "Add/Remove Games to This Collection", CollectionEdit.Start));
            if (model.Editing is { } editing)
                rows.Children.Add(EditButton("GamelistFinishEditing", $"Finish Editing '{editing}' Collection", CollectionEdit.Finish));

            Button apply = Ui.Button("Apply", Close), cancel = Ui.Button("Cancel", Cancel);
            apply.Name = "GamelistApply";
            cancel.Name = "GamelistCancel";
            Control buttons = Ui.Buttons(apply, cancel).Margin(0, 12, 0, 0);
            DockPanel.SetDock(buttons, Dock.Bottom);
            Content = new DockPanel { LastChildFill = true, Children = { buttons, new ScrollViewer { Content = rows.Margin(4, 4, 4, 4), MaxHeight = 560 } } }.Margin(16);
            Closing += (_, _) => { if (!_cancelled) Result = Collect(); };
        }

        public GamelistOptionsResult? Result { get; private set; }

        private Button EditButton(string name, string text, CollectionEdit edit)
        {
            Button button = Ui.Button(text, () => { _edit = edit; Close(); });
            button.Name = name;
            button.HorizontalAlignment = HorizontalAlignment.Left;
            return button;
        }

        private GamelistOptionsResult Collect()
        {
            string? letter = _jump.SelectedItem as string;
            GameSort sort = _model.Sorts.FirstOrDefault(s => s.Label == _sort.SelectedItem as string, _model.Sort);
            return new GamelistOptionsResult(sort, _filter, letter == _model.Letter ? null : letter, _edit);
        }

        public void Cancel()
        {
            _cancelled = true;
            Close();
        }

        // The Back button closes without applying, as ES-DE's does; everything else is the router's.
        public bool OnPad(UiButton button)
        {
            if (button != UiButton.Options) return false;
            Cancel();
            return true;
        }

        private void ShowFilterState()
        {
            int fields = _filter.Chosen.Count(c => c.Value.Count > 0) + (_filter.Name.Trim().Length > 0 ? 1 : 0);
            _filterState.Text = fields == 0 ? "No filter" : fields == 1 ? "1 filter set" : $"{fields} filters set";
        }

        // The filter screen on a sheet of its own over this one, its choices kept when it closes.
        private async Task FilterAsync()
        {
            var window = new GamelistFilterWindow(_filter, _model.Games);
            await SheetLayer.Show(window, this);
            _filter = window.Filter;
            ShowFilterState();
        }
    }
}
