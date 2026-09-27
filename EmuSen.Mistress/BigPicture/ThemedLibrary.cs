using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.BigPicture
{
    // One of the library's systems as the themed view lists it: ES-DE's names for it and its games; a collection's rules, and a grouped system's folders (§22).
    public sealed record ThemedShelf(ThemeSystem System, IReadOnlyList<SceneGame> Games)
    {
        public IReadOnlyList<ThemedShelf>? Folders { get; init; }
        public bool FavoritesFirst { get; init; } = true;
        public bool Stars { get; init; } = true;
        public GameSort? DefaultSort { get; init; }
        public long? CollectionId { get; init; }

        // The player's flatten switch for this console: its games in one list, as before folders were shown (§30).
        public bool Flatten { get; init; }

        // Each folder's ES-DE folder link, by the folder's path on disk: the file A launches in its place (§30).
        public IReadOnlyDictionary<string, string>? FolderLinks { get; init; }
    }

    // The custom collection being edited, and the files in it (§22).
    public sealed record EditedCollection(long Id, string Name, IReadOnlySet<string> Members);

    public enum ThemedAction { None, Launch, Favourite, ClearSearch, Menu, Leave, Options, ToggleCollection }

    // What a button asked of the window, beyond what the view does by itself.
    public readonly record struct ThemedCommand(ThemedAction Action, SceneGame? Game = null);

    // The ES-DE theme's two views as a big-screen session's library: the stage, the pad's rules, the sounds, the selection kept across rebuilds - see EmuSen_BigPicture.md §15.
    public sealed partial class ThemedLibrary
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
        private string? _mediaKey;
        private readonly Dictionary<string, MediaPresence> _presence = new(StringComparer.Ordinal);
        private readonly Dictionary<string, ResolvedTheme> _themes = new(StringComparer.Ordinal);

        public ThemedLibrary(Func<TimeSpan> clock) => _clock = clock;

        // The window shows this; the stage inside it is replaced on each Show.
        public Panel Root { get; } = new() { ClipToBounds = true };

        public SceneStage? Stage { get; private set; }

        // Called with a sound file for each navigation sound; null plays nothing.
        public Action<string>? PlaySound { get; set; }

        public PadFamily Family { get; private set; }

        // The swap of the A and B functions, which the help bar follows (settings reference §4.61).
        public bool SwapFaceButtons { get; private set; }

        public DeviceStatus Status { get; set; } = new();

        public Func<DateTime> Now { get; set; } = () => DateTime.Now;

        public SceneMotion Motion { get; set; } = SceneMotion.Esde;

        // The player's interface switches: clock, help, status, quick system select, startup and the scroll overlay (§29).
        public BigPictureInterface Interface { get; set; } = new();

        private bool _started;

        // The clock follows the wall clock; a test that draws a fixed time turns this off.
        public bool LiveClock { get; set; } = true;

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
        public bool Show(string themeDirectory, Size screen, IReadOnlyList<ThemedShelf> shelves, ISceneMedia? media, string? mediaKey = null, ThemeChoices? chosen = null)
        {
            if (screen.Width < 1 || screen.Height < 1) return Stage is not null;
            var clock = Stopwatch.StartNew();
            if (_directory != themeDirectory || _capabilities is null)
            {
                _capabilities = ThemeCapabilitiesReader.Read(themeDirectory);
                _directory = themeDirectory;
                _themes.Clear();
            }

            if (mediaKey != _mediaKey || mediaKey is null) _presence.Clear();
            _mediaKey = mediaKey;

            if (_capabilities.Diagnostics.FirstOrDefault(d => d.Severity == ThemeSeverity.Error) is { } broken)
                return Fail($"The theme could not be read: {broken.Message}");

            _shelves = shelves;
            FindFoldered(shelves);
            ForgetListed();
            _media = media;
            _screen = screen;
            ThemeChoices choices = (chosen ?? new ThemeChoices()) with { ScreenWidth = (int)Math.Round(screen.Width), ScreenHeight = (int)Math.Round(screen.Height) };
            Choices = choices;
            var systems = new List<SceneSystem>();
            var scan = new Stopwatch();
            foreach (ThemedShelf shelf in shelves.Where(s => s.Games.Count > 0 || s.Folders is { Count: > 0 }))
            {
                scan.Start();
                MediaPresence presence = Presence(shelf);
                scan.Stop();
                string key = $"{shelf.System.Name}|{choices}|{string.Join(",", presence.Types.Order(StringComparer.Ordinal))}";
                if (!_themes.TryGetValue(key, out ResolvedTheme? theme))
                {
                    _themes[key] = theme = ThemeLoader.Load(_capabilities, shelf.System, choices, presence);
                    if (!theme.IsThemed) ErrorLog.Warning("themes", $"{_capabilities.ThemeName} has no view for {shelf.System.Name}: {theme.Errors.First().Message}", string.Join(" | ", theme.Errors.Select(e => e.Message)));
                }
                if (theme.IsThemed) systems.Add(new SceneSystem(shelf.System, theme, shelf.Games));
            }
            _systems = systems;
            StartAt(systems);
            MediaTime = scan.Elapsed;
            LoadTime = clock.Elapsed - scan.Elapsed;
            if (systems.Count == 0) return Fail(shelves.Any(s => s.Games.Count > 0) ? "The theme has no view for any system in the library." : "The library has no games.");

            clock.Restart();
            Error = null;
            Build(_view);
            BuildTime = clock.Elapsed;
            return true;
        }

        // The choices the last Show loaded the theme with, the screen's size included.
        public ThemeChoices Choices { get; private set; } = new();

        // The theme folder was replaced on disk, by an update: the next Show reads it afresh.
        public void Forget()
        {
            _capabilities = null;
            _themes.Clear();
        }

        private bool Fail(string why)
        {
            ErrorLog.Warning("themes", why, _capabilities?.ThemeName);
            Error = why;
            Stage = null;
            Root.Children.Clear();
            return false;
        }

        // Which media a system's games have, for the theme's noMedia and noVideos variants; a type counts once any game has it, and the answer is kept until the games or the media change.
        private MediaPresence Presence(ThemedShelf shelf)
        {
            if (_media is null) return MediaPresence.None;
            var sources = shelf.Games.GroupBy(g => g.Source ?? shelf.System).ToList();
            string stamps = string.Join(",", sources.Select(s => _media.Stamp(s.Key)));
            string key = $"{shelf.System.Name}|{shelf.Games.Count}|{(shelf.Games.Count == 0 ? "" : shelf.Games[0].File + shelf.Games[^1].File)}|{stamps}";
            if (_presence.TryGetValue(key, out MediaPresence? kept)) return kept;
            var types = new HashSet<string>(StringComparer.Ordinal);
            foreach (var source in sources) types.UnionWith(_media.Present(source.Key, source.ToList()));
            return _presence[key] = new MediaPresence(types);
        }

        // ES-DE's "System on startup" and "Startup view", applied at the first showing only; a system the library lacks falls back to the first (§29).
        private void StartAt(IReadOnlyList<SceneSystem> systems)
        {
            if (_started || systems.Count == 0) return;
            _started = true;
            SceneSystem? chosen = systems.FirstOrDefault(s => s.System.Name == Interface.StartupSystem);
            _system = (chosen ?? systems[0]).System.Name;
            _view = Interface.StartupView == BigPictureInterface.ViewGamelist ? "gamelist" : "system";
        }

        // The status indicators the player keeps; the percentage goes with the battery, which it is drawn beside (§29).
        private DeviceIndicators StatusShown =>
            (Interface.StatusBluetooth ? DeviceIndicators.Bluetooth : 0) | (Interface.StatusWifi ? DeviceIndicators.Wifi : 0) | DeviceIndicators.Cellular
            | (Interface.StatusBattery ? DeviceIndicators.Battery : 0) | (Interface.StatusBattery && Interface.StatusBatteryPercentage ? DeviceIndicators.BatteryPercentage : 0);

        // The data the kept selection gives: the system by name, its games as the options list them (§22), the game by file.
        private int _shuffle = Random.Shared.Next();

        private SceneData Data()
        {
            IReadOnlyList<SceneSystem> systems = _systems.Select(s => s with
            {
                Games = Listed(s.System.Name), Stars = StarsIn(s.System.Name), Heading = HeadingIn(s.System.Name), FavoritesOnTop = FavoritesFirstIn(s.System.Name),
                Counted = CountedIn(s.System.Name),
            }).ToList();
            int system = Math.Max(0, systems.ToList().FindIndex(s => s.System.Name == _system));
            SceneSystem chosen = systems[system];
            int game = _cursor.TryGetValue(ViewKey(chosen.System.Name), out string? file) ? Math.Max(0, chosen.Games.ToList().FindIndex(g => g.File == file)) : 0;
            return new SceneData(systems, _screen)
            {
                SystemIndex = system, GameIndex = game, Media = _media, Motion = Motion, Family = Family, SwapFaceButtons = SwapFaceButtons, Status = Status, Now = Now(), ShowClock = Interface.DisplayClock, LiveClock = LiveClock,
                ShowHelp = Interface.DisplayHelp, StatusShown = StatusShown, ScrollOverlay = Interface.ListScrollOverlay, Help = HelpContext,
                Shuffle = unchecked(_shuffle += 7919),
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
            if (_view == "gamelist" && d.Game is { } game) _cursor[ViewKey(_system)] = game.File;
        }

        private void OnStepped(int delta, bool held)
        {
            Sound(ViewName == "system" ? "systembrowse" : "scroll");
            Remember();
        }

        public void Sound(string name)
        {
            if (PlaySound is null || Stage is null) return;
            if (Stage.Current.Data.System.Theme.Sounds.GetValueOrDefault(name) is { Exists: true } path) PlaySound(path.Absolute);
            else PlaySound(NavigationSounds.Key(name)); // Mistress's own for each sound a theme lacks, as THEMES.md says ES-DE falls back (§29).
        }

        public void Advance(TimeSpan now)
        {
            Stage?.Advance(now);
            Remember();
        }

        public TimeSpan? NextChange(TimeSpan now) => Stage?.NextChange(now);

        // The pad's family changed: only the help bar is redrawn (§15, P43).
        public void SetFamily(PadFamily family) => SetPadLayout(family, SwapFaceButtons);

        // The family and the swap together; only the help bar is redrawn (settings reference §4.61).
        public void SetPadLayout(PadFamily family, bool swapped)
        {
            if (family == Family && swapped == SwapFaceButtons) return;
            Family = family;
            SwapFaceButtons = swapped;
            Stage?.Current.SetPadLayout(family, swapped);
        }

        // Whether a gamelist's primary element takes left and right itself: a grid, or a horizontal carousel.
        private bool SidewaysIn(string view)
        {
            ResolvedElement? primary = _systems.Count == 0 ? null : (_systems.FirstOrDefault(s => s.System.Name == _system) ?? _systems[0]).Theme.View(view).Primary;
            return primary is { Type: "grid" } || primary is { Type: "carousel" } p && !(p.String("type") ?? "horizontal").StartsWith("vertical", StringComparison.Ordinal);
        }

        // The direction the primary element moves along: a horizontal carousel's is left and right, a list's and a vertical carousel's up and down.
        public bool Moves(UiButton direction)
        {
            ResolvedElement? primary = Stage?.Current.View.Primary;
            if (primary is { Type: "grid" }) return true;
            bool across = primary is { Type: "carousel" } p && !(p.String("type") ?? "horizontal").StartsWith("vertical", StringComparison.Ordinal);
            return across ? direction is UiButton.Left or UiButton.Right : direction is UiButton.Up or UiButton.Down;
        }

        private static int Sign(UiButton direction) => direction is UiButton.Up or UiButton.Left ? -1 : 1;

        // A direction going down: the primary element steps and repeats while it is held; in a gamelist, left and right change the system once when quick system select gives them that.
        public void PressDirection(UiButton direction, TimeSpan now)
        {
            if (Stage is null) return;
            if (ViewName == "gamelist" && direction is UiButton.Left or UiButton.Right && QuickSelect == BigPictureInterface.QuickSelectLeftRight) ChangeSystem(Sign(direction), now);
            else if (Moves(direction)) Stage.Current.Press(Sign(direction), now, vertical: direction is UiButton.Up or UiButton.Down);
            Advance(now);
        }

        // Which pair of buttons changes the system in this gamelist: ES-DE's two "or" settings give left and right to a list and a vertical carousel, the shoulders or triggers to a grid and a horizontal carousel (UG "UI settings").
        public string QuickSelect => QuickSelectFor(ViewName);

        private string QuickSelectFor(string view)
        {
            bool sideways = SidewaysIn(view);
            return Interface.QuickSystemSelect switch
            {
                BigPictureInterface.QuickSelectLeftRightOrShoulders => sideways ? BigPictureInterface.QuickSelectShoulders : BigPictureInterface.QuickSelectLeftRight,
                BigPictureInterface.QuickSelectLeftRightOrTriggers => sideways ? BigPictureInterface.QuickSelectTriggers : BigPictureInterface.QuickSelectLeftRight,
                BigPictureInterface.QuickSelectShoulders or BigPictureInterface.QuickSelectTriggers or BigPictureInterface.QuickSelectLeftRight => Interface.QuickSystemSelect,
                _ => BigPictureInterface.QuickSelectDisabled,
            };
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
                case UiButton.Accept when view.Data.Game is { Folder: true } folder && LinkedGame(folder) is { } linked:
                    Sound("launch");
                    result = new ThemedCommand(ThemedAction.Launch, linked);
                    break;
                case UiButton.Accept when view.Data.Game is { Folder: true } folder:
                    EnterFolder(folder, now);
                    break;
                case UiButton.Accept when view.Data.Game is { } game:
                    Sound("launch");
                    result = new ThemedCommand(ThemedAction.Launch, game);
                    break;
                case UiButton.Back when gamelist && _filter.Length > 0:
                    Sound("back");
                    result = new ThemedCommand(ThemedAction.ClearSearch);
                    break;
                case UiButton.Back when gamelist && InFolder:
                    LeaveFolder(now);
                    break;
                case UiButton.Back when gamelist:
                    Stage.Switch(now);
                    Sound("back");
                    break;
                case UiButton.Back:
                    result = new ThemedCommand(ThemedAction.Leave);
                    break;
                case UiButton.PageUp or UiButton.PageDown when gamelist && QuickSelect == BigPictureInterface.QuickSelectShoulders:
                    ChangeSystem(button == UiButton.PageUp ? -1 : 1, now);
                    break;
                case UiButton.First or UiButton.Last when gamelist && QuickSelect == BigPictureInterface.QuickSelectTriggers:
                    ChangeSystem(button == UiButton.First ? -1 : 1, now);
                    break;
                case UiButton.PageUp when gamelist:
                    view.Jump(-ShoulderJump, now);
                    break;
                case UiButton.PageDown when gamelist:
                    view.Jump(ShoulderJump, now);
                    break;
                case UiButton.First when gamelist:
                    view.Jump(-view.Index, now);
                    break;
                case UiButton.Last when gamelist:
                    view.Jump(view.Count - 1 - view.Index, now);
                    break;
                case UiButton.Search when gamelist && Editing is not null && view.Data.Game is { Folder: false } member:
                    result = new ThemedCommand(ThemedAction.ToggleCollection, member);
                    break;
                // ES-DE's Y: the favourite, except while a collection is edited (above); the search is in Select's menu (§4.58).
                case UiButton.Search when gamelist && view.Data.Game is { Folder: false } favourite:
                    result = new ThemedCommand(ThemedAction.Favourite, favourite);
                    break;
                case UiButton.Random:
                    RandomEntry(now);
                    break;
                // ES-DE's Back button opens its gamelist options menu; the favourite is an entry there (§4.59 of the settings reference).
                case UiButton.Options when gamelist && view.Data.Game is { } game:
                    result = new ThemedCommand(ThemedAction.Options, game);
                    break;
                case UiButton.Menu:
                    result = new ThemedCommand(ThemedAction.Menu);
                    break;
            }

            Advance(now);
            return result;
        }

        // USERGUIDE: the shoulders "jump 10 games in the gamelists", stopping at the ends; chosen over a page on 2026-09-26 (Q14).
        public const int ShoulderJump = 10;

        // The search box's text narrows every gamelist; the selection stays on its game while that game still matches.
        public void SetFilter(string text, TimeSpan now)
        {
            text = text.Trim();
            if (text == _filter || Stage is null) return;
            _filter = text;
            ForgetListed();
            Stage.Replace(Data(), now);
            Remember();
        }
    }
}
