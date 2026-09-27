using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // What the chooser asks of the window that scrapes: whether ScreenScraper can be asked, one search, and the pick.
    public interface IFindByNameHost
    {
        string? WhyNot { get; }
        int KindsTurnedOn { get; }
        string SystemOf(string path);
        Task<SearchAnswer> SearchByNameAsync(string path, string text);
        bool ChooseFoundGame(string path, ScrapedGame game);
    }

    // ES-DE's "Find by name" and its chooser: nothing is sent until Search, one request a search, and a pick keeps that game for this file - see EmuSen_BigPicture.md §38.
    public sealed class FindByNameWindow : ToolWindow, IPadDriven
    {
        private readonly IFindByNameHost _host;
        private readonly string _path;
        private readonly TextBox _text = new() { Name = "FindByNameText", PlaceholderText = "The game's name", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly Button _search;
        private readonly HintText _status = new() { Name = "FindByNameStatus" };
        private readonly StackPanel _list = Ui.Stack(4);

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public FindByNameWindow() : this(null!, "", "", "") { }

        public FindByNameWindow(IFindByNameHost host, string path, string title, string searchText)
        {
            _host = host;
            _path = path;
            Title = "Find by Name";
            Width = 640;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            _text.Text = searchText;
            _search = Ui.Button("Search", () => _ = SearchAsync());
            _search.Name = "FindByNameSearch";
            int kinds = host?.KindsTurnedOn ?? 0;
            var header = Ui.Stack(6,
                new TextBlock { Name = "FindByNameTitle", Text = title, FontSize = 20, FontWeight = Avalonia.Media.FontWeight.SemiBold, TextWrapping = Avalonia.Media.TextWrapping.Wrap },
                new HintText
                {
                    Name = "FindByNameHint",
                    Text = "Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; "
                        + "each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. "
                        + $"Choosing a game keeps its text for this file and fetches its pictures, up to {kinds} more request{(kinds == 1 ? "" : "s")}; the file itself is not renamed.",
                },
                SearchRow(), _status);

            Button close = Ui.Button("Cancel", Close);
            close.Name = "FindByNameCancel";
            Content = Ui.Stack(12, header, new ScrollViewer { Name = "FindByNameScroll", Content = _list, MaxHeight = 420 }, Ui.Buttons(close)).Margin(16);
            if (host?.WhyNot is string why)
            {
                _search.IsEnabled = false;
                _status.Text = $"ScreenScraper cannot be asked: {why}.";
            }
            _text.KeyDown += (_, e) =>
            {
                if (e.Key != Key.Enter) return;
                e.Handled = true;
                _ = SearchAsync();
            };
            Opened += (_, _) => (_search.IsEnabled ? _search : (InputElement)close).Focus(NavigationMethod.Directional);
        }

        // The box takes the width the button leaves.
        private Grid SearchRow()
        {
            var row = new Grid { ColumnDefinitions = new ColumnDefinitions("*,Auto"), ColumnSpacing = 8 };
            Grid.SetColumn(_search, 1);
            row.Children.Add(_text);
            row.Children.Add(_search);
            return row;
        }

        public IEnumerable<Button> Results => _list.Children.OfType<Button>();

        public IReadOnlyList<ScrapedGame> Found { get; private set; } = [];

        public string StatusText => _status.Text ?? "";

        public Task? Searching { get; private set; }

        public Task SearchAsync() => Searching = SearchNowAsync();

        private async Task SearchNowAsync()
        {
            if (_host is null || !_search.IsEnabled) return;
            bool hadFocus = _search.IsFocused || _text.IsFocused;
            _search.IsEnabled = false;
            _status.Text = "Asking ScreenScraper...";
            SearchAnswer answer = await _host.SearchByNameAsync(_path, _text.Text ?? "");
            _search.IsEnabled = _host.WhyNot is null;
            Found = answer.Games;
            Fill(answer);
            if (hadFocus) ((InputElement?)Results.FirstOrDefault() ?? _search).Focus(NavigationMethod.Directional);
        }

        private void Fill(SearchAnswer answer)
        {
            _list.Children.Clear();
            _status.Text = answer.Status switch
            {
                ScrapeStatus.Found => $"{answer.Games.Count} game{(answer.Games.Count == 1 ? "" : "s")}, most likely first. Choose the one this file is.",
                ScrapeStatus.NotFound => "ScreenScraper found no game by that name. Try fewer words, or a different spelling.",
                _ => ScrapeRedactor.Redact(answer.Detail.Length > 0 ? answer.Detail : answer.Status.ToString()),
            };
            string system = _host.SystemOf(_path);
            int? systemId = Scraper.SystemIds.TryGetValue(system, out int id) ? id : null;
            IReadOnlyList<string> regions = ScrapeRules.RegionOrder(Path.GetFileName(_path), ScrapeRules.AutomaticRegion, fallback: true);
            foreach (ScrapedGame game in answer.Games)
            {
                string name = ScrapeRules.ChooseText(game.Names, [.. regions, "ss"], fallback: true)?.Text ?? $"Game {game.Id}";
                string? year = ScrapeRules.Date(ScrapeRules.ChooseText(game.Dates, regions, fallback: true)?.Text)?.Year.ToString(System.Globalization.CultureInfo.InvariantCulture);
                var detail = new List<string>();
                if (year is not null) detail.Add(year);
                if (game.Publisher is { Length: > 0 } publisher) detail.Add(publisher);
                // ES-DE shows another platform in brackets only when the service returned one (USERGUIDE "Scraping process").
                if (game.SystemId is int other && systemId is int mine && other != mine) detail.Add($"[{SystemName(other)}]");
                var row = Ui.Stack(0,
                    new TextBlock { Text = name, TextTrimming = Avalonia.Media.TextTrimming.CharacterEllipsis },
                    new HintText { Text = detail.Count == 0 ? $"ScreenScraper game {game.Id}" : string.Join(" · ", detail) });
                var button = new Button { Content = row, Tag = game, HorizontalAlignment = HorizontalAlignment.Stretch, HorizontalContentAlignment = HorizontalAlignment.Left };
                button.Name = "FindByNameResult_" + _list.Children.Count;
                button.Click += (_, _) => Choose(game);
                _list.Children.Add(button);
            }
        }

        private static string SystemName(int id) => Scraper.SystemIds.FirstOrDefault(s => s.Value == id).Key is { Length: > 0 } s
            ? EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder.FirstOrDefault(shelf => shelf.EsdeSystem == s)?.EsdeFullName ?? s
            : $"system {id}";

        public bool Choose(ScrapedGame game)
        {
            if (_host is null) return false;
            Close();
            return _host.ChooseFoundGame(_path, game);
        }

        // Select, as in the options menu, puts the chooser away; B does too through the sheet.
        public bool OnPad(UiButton button)
        {
            if (button != UiButton.Options) return false;
            Close();
            return true;
        }
    }
}
