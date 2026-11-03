using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Galaxia.Native;

namespace EmuSen.Mistress.Pages
{
    // The game left running: its file, name, console and when it was left.
    public sealed record HomeSuspended(string Path, string Title, string Console, DateTime PausedAt);

    // One place on Home: what the host opens, its name, its live line and whether that line needs the player.
    public sealed record HomePlace(string Key, string Label, string? Status, bool NeedsAttention = false);

    // One game on Home's shelf or in the grid: what the view draws for it, named by its file.
    public sealed record ShelfGame(string Path, string Title, string? Mark, double Aspect, bool Favourite, string Facts, string Line);

    // A help bar entry: the button by the frame's name, its words, and the key a press of it gives back; null for what the screen does itself.
    public sealed record ScreenHint(string Button, string Text, string? Key = null);

    // What Home reads from the window and asks it to do; the tests give a fake.
    public interface IHomeHost
    {
        DateTime Now { get; }
        bool Desktop { get; }
        HomeSuspended? Suspended { get; }
        IReadOnlyList<string> Recent { get; }
        GameFacts Facts(string path);
        int GameCount { get; }
        // A tool's one-line state as its library last said it, null before it has said anything.
        ToolSummary? Summary(string place);
        int ControllerCount { get; }
        string? Battery { get; }
        string SettingsLine { get; }
        bool BigPictureForced { get; }
        void Resume();
        void Play(string path);
        void OpenPage(string path);
        void ToggleFavourite(string path);
        void Open(string place);
    }

    // Home, EmuSen's own look's first screen, as one plain description for both forms: Continue, the Recent shelf, the places with their live lines, the help bar and what each press does - see EmuSen_BigPicture.md §51.
    public sealed class HomeScreen(IHomeHost host)
    {
        public const string Title = "Home", ContinueCaption = "Continue playing", ResumeWord = "Resume", RecentTitle = "Recent", PlacesTitle = "Places";
        public const string Games = "games", Scraper = "scraper", Packages = "packages", Library = "library", Cores = "cores", Cheats = "cheats", Settings = "settings", Power = "power", BigPicture = "bigpicture", Search = "search";

        // How many recent games the shelf holds; it shows as many whole as the screen fits.
        public const int ShelfLimit = 30;

        private readonly IHomeHost _host = host;

        public IHomeHost Host => _host;

        // The device's line at the top right: the controllers, a battery where there is one, and the time.
        public string StatusLine
        {
            get
            {
                var parts = new List<string> { _host.ControllerCount == 1 ? "1 controller" : $"{_host.ControllerCount} controllers" };
                if (_host.Battery is { Length: > 0 } battery) parts.Add(battery);
                parts.Add(_host.Now.ToString("HH:mm", CultureInfo.InvariantCulture));
                return string.Join(" · ", parts);
            }
        }

        // Continue's words: the game's name and its console with how long ago it was left; null with nothing left.
        public (string Title, string Facts)? Continue => _host.Suspended is { } s ? (s.Title, $"{s.Console} · {PausedAgo(_host.Now - s.PausedAt)}") : null;

        public static string PausedAgo(TimeSpan since) =>
            since.TotalMinutes < 1 ? "Paused just now"
            : since.TotalMinutes < 2 ? "Paused 1 minute ago"
            : since.TotalMinutes < 60 ? $"Paused {(int)since.TotalMinutes} minutes ago"
            : since.TotalHours < 2 ? "Paused 1 hour ago"
            : since.TotalHours < 24 ? $"Paused {(int)since.TotalHours} hours ago"
            : since.TotalDays < 2 ? "Paused yesterday" : $"Paused {(int)since.TotalDays} days ago";

        // The games played last, newest first, the suspended one left out since Continue holds it.
        public IReadOnlyList<ShelfGame> Shelf =>
            _host.Recent.Where(p => p != _host.Suspended?.Path).Take(ShelfLimit).Select(p => GameOf(_host.Facts(p), _host.Now)).ToList();

        public static ShelfGame GameOf(GameFacts f, DateTime now)
        {
            var facts = new List<string> { f.Console };
            if (f.Year is { Length: > 0 } year) facts.Add(year);
            if (f.Developer is { Length: > 0 } developer) facts.Add(developer);
            if (f.Played > TimeSpan.Zero) facts.Add(Played(f.Played) + " played");
            if (f.LastPlayed is { } last) facts.Add(GamePage.Ago(last, now));
            string line = f.Year is { Length: > 0 } y ? $"{f.Console} · {y}" : f.Console;
            return new ShelfGame(f.Path, f.Title, f.Mark, f.CoverAspect, f.Favourite, string.Join(" · ", facts), line);
        }

        public static string Played(TimeSpan played) =>
            played.TotalHours >= 1 ? $"{(int)played.TotalHours} h {played.Minutes} min" : $"{Math.Max(1, played.Minutes)} min";

        // The places in the design's order, each with its tool's live line, or what it does before the tool has said anything; the desktop's eight, the big screen's seven (EmuSen_Interface.md §4 screen 1, §19).
        public IReadOnlyList<HomePlace> Places
        {
            get
            {
                bool desk = _host.Desktop;
                var places = new List<HomePlace>
                {
                    new(Games, "Games", ToolSummary.Count(_host.GameCount, "game", "games")),
                    Tool(Scraper, "Scraper", desk),
                    Tool(Packages, "Packages", desk),
                    Tool(Library, desk ? "Library Manager" : "Library", desk),
                    Tool(Cores, "Cores", desk),
                };
                if (desk) places.Add(Tool(Cheats, "Cheats", desk));
                places.Add(new(Settings, "Settings", _host.SettingsLine));
                if (desk) places.Add(new(BigPicture, "Big Picture", "Open full screen"));
                else places.Add(new(Power, "Power", _host.BigPictureForced ? "Quit" : "Exit"));
                return places;
            }
        }

        // The line without its sign, which the place draws itself in the warning colour.
        private HomePlace Tool(string key, string label, bool desk) =>
            _host.Summary(key) is { } s ? new(key, label, desk ? s.Line : s.Brief, s.NeedsAttention) : new(key, label, Description(key));

        // What a tool does, said until it has a line of its own.
        public static string Description(string place) => place switch
        {
            Scraper => "Art and facts",
            Packages => "Game packages",
            Library => "Checks dumps",
            Cores => "The engines",
            Cheats => "Codes for games",
            _ => "",
        };

        // The help bar for what is focused, as the mockups give it: Continue resumes, a recent game plays, a place opens.
        public IReadOnlyList<ScreenHint> HintsFor(string row) => row switch
        {
            "continue" => [new("South", ResumeWord), new("West", "Game page", "page"), new("North", "Favourite", "favourite"), new("Select", "Search", Search), new("Start", Title)],
            "shelf" => [new("South", "Play"), new("West", "Game page", "page"), new("North", "Favourite", "favourite"), new("Select", "Search", Search), new("Start", Title)],
            _ => [new("South", "Open"), new("Select", "Search", Search), new("Start", Title)],
        };

        public void ChooseContinue() => _host.Resume();

        public void ChooseGame(string path) => _host.Play(path);

        public void ChoosePlace(string key) => _host.Open(key);

        // A key the help bar gave back, for the focused row and game.
        public void Hint(string key, string row, string? game)
        {
            string? path = row == "continue" ? _host.Suspended?.Path : game;
            switch (key)
            {
                case "page" when path is not null: _host.OpenPage(path); break;
                case "favourite" when path is not null: _host.ToggleFavourite(path); break;
                case Search: _host.Open(Search); break;
            }
        }
    }
}
