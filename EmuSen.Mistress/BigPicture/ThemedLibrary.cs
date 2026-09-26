using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.BigPicture
{
    // One of the library's systems as the themed view lists it: ES-DE's names for it and its games.
    public sealed record ThemedShelf(ThemeSystem System, IReadOnlyList<SceneGame> Games);

    public enum ThemedAction { None, Launch, Favourite, Search, ClearSearch, Menu, Leave }

    // What a button asked of the window, beyond what the view does by itself.
    public readonly record struct ThemedCommand(ThemedAction Action, SceneGame? Game = null);

    // The ES-DE theme's two views as a big-screen session's library: the stage, the pad's rules, the sounds, the selection kept across rebuilds - see EmuSen_BigPicture.md §15.
    public sealed class ThemedLibrary
    {
        private readonly Func<TimeSpan> _clock;
        private readonly Dictionary<string, string> _cursor = new(StringComparer.Ordinal);
        private ThemeCapabilities? _capabilities;
        private string? _directory;
        private IReadOnlyList<ThemedShelf> _shelves = [];
        private IReadOnlyList<SceneSystem> _systems = [];
        private ISceneMedia? _media;
        private Size _screen;
        private string _view = "system";
        private string? _system;
        private string _filter = "";

        public ThemedLibrary(Func<TimeSpan> clock) => _clock = clock;

        // The window shows this; the stage inside it is replaced on each Show.
        public Panel Root { get; } = new() { ClipToBounds = true };

        public SceneStage? Stage { get; private set; }

        // Called with a sound file for each navigation sound; null plays nothing.
        public Action<string>? PlaySound { get; set; }

        public PadFamily Family { get; private set; }

        public DeviceStatus Status { get; set; } = new();

        public Func<DateTime> Now { get; set; } = () => DateTime.Now;

        public SceneMotion Motion { get; set; } = SceneMotion.Esde;

        // Why the theme could not be shown, when it could not.
        public string? Error { get; private set; }

        public string Filter => _filter;

        public string ViewName => Stage?.Current.ViewName ?? _view;

        public SceneSystem? SelectedSystem => Stage?.Current.Data.System;

        public SceneGame? SelectedGame => Stage is { Current.ViewName: "gamelist" } s ? s.Current.Data.Game : null;

        // What the last Show spent reading the theme, looking for media and building the stage, for the first-build prediction (§15).
        public TimeSpan LoadTime { get; private set; }
        public TimeSpan MediaTime { get; private set; }
        public TimeSpan BuildTime { get; private set; }

        // Every sound file the theme names, so the player can decode them before the first is wanted.
        public IEnumerable<string> SoundFiles => _systems.SelectMany(s => s.Theme.Sounds.Values).Where(p => p.Exists).Select(p => p.Absolute).Distinct();

        // Reads the theme when its folder changed, then builds the stage at the kept selection; false, with Error, when there is nothing to show.
        public bool Show(string themeDirectory, Size screen, IReadOnlyList<ThemedShelf> shelves, ISceneMedia? media)
        {
            if (screen.Width < 1 || screen.Height < 1) return Stage is not null;
            var clock = Stopwatch.StartNew();
            if (_directory != themeDirectory || _capabilities is null)
            {
                _capabilities = ThemeCapabilitiesReader.Read(themeDirectory);
                _directory = themeDirectory;
            }

            if (_capabilities.Diagnostics.FirstOrDefault(d => d.Severity == ThemeSeverity.Error) is { } broken)
                return Fail($"The theme could not be read: {broken.Message}");

            _shelves = shelves;
            _media = media;
            _screen = screen;
            var choices = new ThemeChoices { ScreenWidth = (int)Math.Round(screen.Width), ScreenHeight = (int)Math.Round(screen.Height) };
            var systems = new List<SceneSystem>();
            var scan = new Stopwatch();
            foreach (ThemedShelf shelf in shelves.Where(s => s.Games.Count > 0))
            {
                scan.Start();
                MediaPresence? presence = Presence(shelf);
                scan.Stop();
                ResolvedTheme theme = ThemeLoader.Load(_capabilities, shelf.System, choices, presence);
                if (theme.IsThemed) systems.Add(new SceneSystem(shelf.System, theme, shelf.Games));
            }
            _systems = systems;
            MediaTime = scan.Elapsed;
            LoadTime = clock.Elapsed - scan.Elapsed;
            if (systems.Count == 0) return Fail(shelves.Any(s => s.Games.Count > 0) ? "The theme has no view for any system in the library." : "The library has no games.");

            clock.Restart();
            Error = null;
            Build(_view);
            BuildTime = clock.Elapsed;
            return true;
        }

        private bool Fail(string why)
        {
            Error = why;
            Stage = null;
            Root.Children.Clear();
            return false;
        }

        // Which media a system's games have, for the theme's noMedia and noVideos variants; a type counts once any game has it.
        private MediaPresence? Presence(ThemedShelf shelf)
        {
            if (_media is null) return MediaPresence.None;
            var found = new HashSet<string>(StringComparer.Ordinal);
            foreach (string type in ThemeCapabilities.MediaTypes)
                if (shelf.Games.Any(g => _media.Find(shelf.System, g, type) is not null)) found.Add(type);
            return new MediaPresence(found);
        }

        // The data the kept selection gives: the system by name, its games narrowed by the search, the game by file.
        private SceneData Data()
        {
            IReadOnlyList<SceneSystem> systems = _filter.Length == 0 ? _systems
                : _systems.Select(s => s with { Games = s.Games.Where(g => FilterBar.Matches(_filter, g.Name)).ToList() }).ToList();
            int system = Math.Max(0, systems.ToList().FindIndex(s => s.System.Name == _system));
            SceneSystem chosen = systems[system];
            int game = _cursor.TryGetValue(chosen.System.Name, out string? file) ? Math.Max(0, chosen.Games.ToList().FindIndex(g => g.File == file)) : 0;
            return new SceneData(systems, _screen)
            {
                SystemIndex = system, GameIndex = game, Media = _media, Motion = Motion, Family = Family, Status = Status, Now = Now(), ShowClock = false,
            };
        }

        private void Build(string view)
        {
            if (Stage is not null) Stage.Stepped -= OnStepped;
            Stage = new SceneStage(Data(), view, _clock());
            Stage.Stepped += OnStepped;
            Root.Children.Clear();
            Root.Children.Add(Stage.Root);
            Remember();
        }

        // The selection as it now is, kept by name so a rebuild from new data finds it again.
        private void Remember()
        {
            if (Stage is null) return;
            SceneData d = Stage.Current.Data;
            _view = Stage.Current.ViewName;
            _system = d.System.System.Name;
            if (_view == "gamelist" && d.Game is { } game) _cursor[_system] = game.File;
        }

        private void OnStepped(int delta, bool held)
        {
            Sound(ViewName == "system" ? "systembrowse" : "scroll");
            Remember();
        }

        private void Sound(string name)
        {
            if (PlaySound is null || Stage is null) return;
            if (Stage.Current.Data.System.Theme.Sounds.GetValueOrDefault(name) is { Exists: true } path) PlaySound(path.Absolute);
        }

        public void Advance(TimeSpan now)
        {
            Stage?.Advance(now);
            Remember();
        }

        public TimeSpan? NextChange(TimeSpan now) => Stage?.NextChange(now);

        // The pad's family changed: only the help bar is redrawn (§15, P43).
        public void SetFamily(PadFamily family)
        {
            if (family == Family) return;
            Family = family;
            Stage?.Current.SetFamily(family);
        }

        // The direction the primary element moves along: a horizontal carousel's is left and right, a list's and a vertical carousel's up and down.
        public bool Moves(UiButton direction)
        {
            ResolvedElement? primary = Stage?.Current.View.Primary;
            bool across = primary is { Type: "carousel" } p && !(p.String("type") ?? "horizontal").StartsWith("vertical", StringComparison.Ordinal);
            return across ? direction is UiButton.Left or UiButton.Right : direction is UiButton.Up or UiButton.Down;
        }

        private static int Sign(UiButton direction) => direction is UiButton.Up or UiButton.Left ? -1 : 1;

        // A direction going down: the primary element steps and repeats while it is held; in a gamelist, the other axis changes the system once.
        public void PressDirection(UiButton direction, TimeSpan now)
        {
            if (Stage is null) return;
            if (Moves(direction)) Stage.Current.Press(Sign(direction), now);
            else if (ViewName == "gamelist" && direction is UiButton.Left or UiButton.Right) ChangeSystem(Sign(direction), now);
            Advance(now);
        }

        public void ReleaseDirection(TimeSpan now)
        {
            if (Stage is null) return;
            Stage.Current.Release(now);
            Remember();
        }

        // ES-DE's quick system select: the next system's gamelist at once, at the game last chosen there.
        private void ChangeSystem(int by, TimeSpan now)
        {
            SceneData d = Stage!.Current.Data;
            if (d.Systems.Count < 2) return;
            _system = d.Systems[((d.SystemIndex + by) % d.Systems.Count + d.Systems.Count) % d.Systems.Count].System.Name;
            Stage.Replace(Data(), now);
            Sound("quicksysselect");
            Remember();
        }

        // Every button but the directions: what the view does itself, and what it asks of the window.
        public ThemedCommand Command(UiButton button, TimeSpan now)
        {
            if (Stage is null) return default;
            SceneView view = Stage.Current;
            bool gamelist = view.ViewName == "gamelist";
            ThemedCommand result = default;
            switch (button)
            {
                case UiButton.Accept when !gamelist:
                    _system = view.Data.System.System.Name;
                    Stage.Switch(now, Data());
                    Sound("select");
                    break;
                case UiButton.Accept when view.Data.Game is { } game:
                    Sound("launch");
                    result = new ThemedCommand(ThemedAction.Launch, game);
                    break;
                case UiButton.Back when gamelist && _filter.Length > 0:
                    Sound("back");
                    result = new ThemedCommand(ThemedAction.ClearSearch);
                    break;
                case UiButton.Back when gamelist:
                    Stage.Switch(now);
                    Sound("back");
                    break;
                case UiButton.Back:
                    result = new ThemedCommand(ThemedAction.Leave);
                    break;
                case UiButton.PageUp when gamelist:
                    view.Jump(-PageSize(view), now);
                    break;
                case UiButton.PageDown when gamelist:
                    view.Jump(PageSize(view), now);
                    break;
                case UiButton.First when gamelist:
                    view.Jump(-view.Index, now);
                    break;
                case UiButton.Last when gamelist:
                    view.Jump(view.Count - 1 - view.Index, now);
                    break;
                case UiButton.Search when gamelist:
                    result = new ThemedCommand(ThemedAction.Search);
                    break;
                case UiButton.Options when gamelist && view.Data.Game is { } game:
                    Sound("favorite");
                    result = new ThemedCommand(ThemedAction.Favourite, game);
                    break;
                case UiButton.Menu:
                    result = new ThemedCommand(ThemedAction.Menu);
                    break;
            }

            Advance(now);
            return result;
        }

        // A page is the rows the list shows at once, as its own height and pitch give them; ten when the primary element is not a list.
        public static int PageSize(SceneView view) =>
            view.Scene.Entries.Select(e => e.Control).OfType<TextRowList>().FirstOrDefault() is { RowPitch: > 0 } list && list.Bounds.Height > 0
                ? Math.Max(1, (int)Math.Floor(list.Bounds.Height / list.RowPitch + 1e-6))
                : 10;

        // The search box's text narrows every gamelist; the selection stays on its game while that game still matches.
        public void SetFilter(string text, TimeSpan now)
        {
            text = text.Trim();
            if (text == _filter || Stage is null) return;
            _filter = text;
            Stage.Replace(Data(), now);
            Remember();
        }
    }
}
