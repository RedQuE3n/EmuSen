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

namespace EmuSen.Mistress.Views
{
    // What the gamelist rows start from: the list's letters, sorts, filter and games (§4.58).
    public sealed record GamelistOptionsModel(
        IReadOnlyList<(string Label, int Index)> Letters, string? Letter, IReadOnlyList<GameSort> Sorts, GameSort Sort,
        GameFilter Filter, IReadOnlyList<SceneGame> Games);

    // What the rows hold when the menu applies: the sort and filters, and the letter jumped to if one was chosen.
    public sealed record GamelistOptionsResult(GameSort Sort, GameFilter Filter, string? Letter);

    // ES-DE's Jump To, Sort Games By and Filter Gamelist, as the first rows of the Select menu; applied when it closes applying - see EmuSen_Settings_Reference.md §4.58.
    public sealed class GamelistOptionRows
    {
        private readonly GamelistOptionsModel _model;
        private readonly Window _owner;
        private readonly Dropdown _jump = new() { Name = "GamelistJumpTo", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly Dropdown _sort = new() { Name = "GamelistSortBy", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBlock _filterState = new() { Name = "GamelistFilterState", VerticalAlignment = VerticalAlignment.Center };
        private GameFilter _filter;

        public GamelistOptionRows(GamelistOptionsModel model, Window owner)
        {
            _model = model;
            _owner = owner;
            _filter = model.Filter;
            string[] letters = model.Letters.Select(l => l.Label).ToArray();
            if (letters.Length > 0) _jump.Fill(letters, model.Letter is { } at && letters.Contains(at) ? at : letters[0]);
            _jump.IsEnabled = letters.Length > 0;
            _sort.Fill(model.Sorts.Select(s => s.Label).ToArray(), model.Sort.Label);
            ShowFilterState();
        }

        // The three rows, the first carrying the apply.
        public IEnumerable<GameOption> Options(Action<GamelistOptionsResult> apply)
        {
            yield return new GameOption("Jump To...", () => { }) { Row = () => new FieldRow { Label = "Jump To...", Content = _jump }, Apply = () => apply(Result) };
            yield return new GameOption("Sort Games By", () => { }) { Row = () => new FieldRow { Label = "Sort Games By", Content = _sort } };
            yield return new GameOption("Filter Gamelist", () => { }) { Row = FilterRow };
        }

        private Control FilterRow()
        {
            Button filter = Ui.Button("Filter Gamelist...", () => _ = FilterAsync());
            filter.Name = "GamelistFilterButton";
            return new FieldRow { Label = "Filter Gamelist", Content = Ui.Row(12, filter, _filterState) };
        }

        public GamelistOptionsResult Result
        {
            get
            {
                string? letter = _jump.SelectedItem as string;
                GameSort sort = _model.Sorts.FirstOrDefault(s => s.Label == _sort.SelectedItem as string, _model.Sort);
                return new GamelistOptionsResult(sort, _filter, letter == _model.Letter ? null : letter);
            }
        }

        private void ShowFilterState()
        {
            int fields = _filter.Chosen.Count(c => c.Value.Count > 0) + (_filter.Name.Trim().Length > 0 ? 1 : 0);
            _filterState.Text = fields == 0 ? "No filter" : fields == 1 ? "1 filter set" : $"{fields} filters set";
        }

        // The filter screen on a sheet of its own over the menu, its choices kept when it closes.
        private async Task FilterAsync()
        {
            var window = new GamelistFilterWindow(_filter, _model.Games);
            await SheetLayer.Show(window, _owner);
            _filter = window.Filter;
            ShowFilterState();
        }
    }
}
