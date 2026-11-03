using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;

namespace EmuSen.Mistress.Pages
{
    // A picture over the pane's lines: a frame of the game the setting changes with its caption, a fraction as a meter, or the two variants side by side.
    public abstract record PanePreview
    {
        public sealed record Frame(string Caption) : PanePreview;
        public sealed record Meter(double Fraction) : PanePreview;
        public sealed record Variants(string Dark, string Light) : PanePreview;
    }

    // What the pane says of the focused row.
    public sealed record ScreenPane(IReadOnlyList<SettingsLine> Lines, PanePreview? Preview = null);

    // One firmware file a system's engines declare, as the runtime's firmware_status answers; files sharing a Need are alternatives for one need.
    public sealed record FirmwareFile(string Label, string File, bool Required, string Status, string? Need = null);

    // One player's seat: who plays it now, or null, and whether it is kept for a controller that went away.
    public sealed record PlayerSeat(int Player, string? Pad, bool Kept);

    // What Settings reads from the window and asks it to do; the tests give a fake.
    public interface ISettingsHost
    {
        AppSettings App { get; }
        GraphicsConfig Graphics { get; }
        bool Desktop { get; }
        string Version { get; }
        string? RunningTitle { get; }
        string? RunningConsole { get; }
        string? RunningCropNote(string console);
        IReadOnlyList<SystemFacts> Systems { get; }
        IReadOnlyList<SettingRow> Schema(string console);
        string? CoreValue(string console, string key);
        void SetCoreValue(string console, string key, string? value);
        IReadOnlyList<FirmwareFile>? Firmware(string system);
        IReadOnlyList<CardMedium> Cards(string console);
        string NewCard(string console);
        int Games(string console);
        int AllGames { get; }
        int Hidden { get; }
        IReadOnlyList<GameCollection> CustomCollections { get; }
        IReadOnlyList<PlayerSeat> Players { get; }
        IReadOnlyList<string> Pads { get; }
        void Seat(int pad, int player);
        bool PadPressed(int pad);
        IReadOnlyList<string> Looks { get; }
        int LookAt { get; }
        void ChooseLook(int at);
        bool Logging { get; set; }
        string? ScrapeUser { get; }
        string AccountLine { get; }
        long? OpenVgdbBytes { get; }
        string? EditingCollection { get; }
        void SaveApp(string? what = null);
        void SaveGraphics(string? console);
        void Open(string door);
        GamepadBindings PadBindings { get; }
        void SavePadBindings();
        IReadOnlyList<string> SoundOutputs { get; }
        void Ask(string title, string prompt, string current, Action<string> then);
        ControllerKeyBindings KeyBindings { get; }
        HotkeyBindingMap Hotkeys { get; }
        void SaveKeys();
        PadFacts PadNow(string console);
    }

    // Settings, as one plain description for both forms: six sections, the pages under their rows, each row's value, sentence and pane, and what each press does - see EmuSen_BigPicture.md §52 and EmuSen_Interface.md §2.7.
    public sealed partial class SettingsScreen
    {
        public const string Title = "Settings";
        public const string Controllers = "controllers", Display = "display", Sound = "sound", Consoles = "consoles", Library = "library", EmuSen = "emusen";

        // The doors to what Settings does not draw itself: tools, pickers and the Shell.
        public const string DoorShaders = "shaders", DoorCheats = "cheats", DoorThemes = "themes", DoorScraper = "scraper", DoorCollections = "collections", DoorShell = "shell", DoorLibraryManager = "librarymanager", DoorDiscs = "discs";
        public const string DoorSignIn = "scraper:signin", DoorSignOut = "scraper:signout", DoorCheckAccount = "scraper:check", DoorOpenVgdb = "scraper:openvgdb", DoorCleanUp = "scraper:cleanup";
        public const string DoorNewCollection = "collection:new", DoorFinishCollection = "collection:finish", DoorDeleteCollection = "collection:delete:", DoorCardSaves = "cardsaves:";
        public const string DoorRomFolder = "folder:rom", DoorArtFolder = "folder:art", DoorStateFolder = "folder:state", DoorLogFolder = "folder:log", DoorFirmwareFolder = "folder:firmware", DoorMediaFolder = "folder:media", DoorThemeFolder = "folder:theme";
        public const string On = "On", Off = "Off";

        public static readonly (string Key, string Label)[] SectionList =
            [(Controllers, "Controllers"), (Display, "Display"), (Sound, "Sound"), (Consoles, "Consoles"), (Library, "Library"), (EmuSen, "EmuSen")];

        // One row as the screen holds it: what is drawn, what its pane says, and what Left, Right and A do.
        private sealed record Setting(PageRow Row, Func<ScreenPane?>? Pane = null, Action<int>? Step = null, Action? Choose = null);

        // One page under a section: its title, its rows made each time it is shown, and the row it was opened from.
        private sealed record Page(string Title, Func<IReadOnlyList<Setting>> Rows, string? From, bool Choices = false, BindingsPage? Bindings = null);

        private readonly ISettingsHost _host;
        private readonly List<Page> _pages = new();
        private IReadOnlyList<Setting> _rows = Array.Empty<Setting>();
        private string _section;

        public SettingsScreen(ISettingsHost host, string section = Controllers)
        {
            _host = host;
            _section = SectionList.Any(s => s.Key == section) ? section : Controllers;
            Read();
        }

        public ISettingsHost Host => _host;

        public string Section => _section;

        // The titles of the pages open under the section, outermost first, the section's own name leading; empty at the section's rows.
        public IReadOnlyList<string> Trail => _pages.Count == 0 ? [] : [SectionList.First(s => s.Key == _section).Label, .. _pages.Select(p => p.Title)];

        public IReadOnlyList<PageRow> Rows => _rows.Select(r => r.Row).ToList();

        // The row a page closed back to, for the view to focus.
        public string? Returned { get; private set; }

        public bool OnChoicePage => _pages.Count > 0 && _pages[^1].Choices;

        private string _search = "";

        public string SearchText => _search;

        // The rows read again from the settings, as after a change made elsewhere; a search lists the rows of every section whose words hold it.
        public void Read() => _rows = _search.Length > 0 ? Found() : _pages.Count > 0 ? _pages[^1].Rows() : SectionRows(_section);

        // The desktop's search across all six sections; choosing a found row goes to its section first.
        public void Search(string text)
        {
            _search = text?.Trim() ?? "";
            if (_search.Length > 0) _pages.Clear();
            Read();
        }

        private IReadOnlyList<Setting> Found() => SectionList.SelectMany(s => SectionRows(s.Key)
            .Where(r => r.Row.Label.Contains(_search, StringComparison.OrdinalIgnoreCase) || (r.Row.Sentence ?? "").Contains(_search, StringComparison.OrdinalIgnoreCase))
            .Select(r => r with { Row = r.Row with { Key = s.Key + "/" + r.Row.Key, Label = $"{r.Row.Label}" }, Choose = () => { string key = r.Row.Key; _search = ""; ChooseSection(s.Key); Choose(key); } })).ToList();

        public void ChooseSection(string key)
        {
            if (!SectionList.Any(s => s.Key == key)) return;
            _section = key;
            _pages.Clear();
            Returned = null;
            Read();
        }

        // Opens a section's page by its rows' keys, as a door from elsewhere opens Consoles ▸ a console.
        public void OpenPath(string section, params string[] keys)
        {
            ChooseSection(section);
            foreach (string key in keys) Choose(key);
        }

        public ScreenPane? PaneOf(string key)
        {
            if (Binding is { } bindings) return BindingPane(bindings);
            Setting? s = _rows.FirstOrDefault(r => r.Row.Key == key);
            if (s is null) return null;
            return s.Pane?.Invoke() ?? (s.Row.Sentence is { Length: > 0 } sentence ? new ScreenPane([new SettingsLine(sentence)]) : null);
        }

        public void Step(string key, int by)
        {
            if (_rows.FirstOrDefault(r => r.Row.Key == key) is not { Step: { } step, Row.IsEnabled: true }) return;
            step(by);
            Read();
        }

        public void Choose(string key)
        {
            if (Binding is { } bindings)
            {
                StartCapture(bindings);
                return;
            }
            if (_rows.FirstOrDefault(r => r.Row.Key == key) is not { Row.IsEnabled: true } s) return;
            Returned = null;
            if (s.Choose is { } choose) choose();
            else if (s.Step is { } step && s.Row.Kind == PageRowKind.Steps) step(1);
            Read();
        }

        // B: a page closes back to the row it was opened from; at a section's rows nothing is left to close and the host closes Settings.
        public bool Back()
        {
            if (_pages.Count == 0) return false;
            Page closed = _pages[^1];
            _pages.RemoveAt(_pages.Count - 1);
            Returned = closed.From;
            Read();
            return true;
        }

        // The help bar for the focused row, as the mockups give it.
        public IReadOnlyList<ScreenHint> HintsFor(string? key)
        {
            if (OnChoicePage) return [new("South", "Choose"), new("East", "Back")];
            if (Binding is { } bindings) return BindingHints(bindings);
            PageRow? row = _rows.FirstOrDefault(r => r.Row.Key == key)?.Row;
            var hints = new List<ScreenHint> { new("South", row?.Kind == PageRowKind.Opens ? "Open" : "Change") };
            if (_rows.Any(r => r.Row.Kind == PageRowKind.Steps)) hints.Add(new("DPadLeftRight", "Value"));
            if (_pages.Count == 0) hints.Add(new("Shoulders", "Section"));
            else if (_section == Consoles && _pages.Count == 1) hints.Add(new("Shoulders", "Console"));
            if (_pages.Count > 0 && _pages[^1].From == "hotkeys") hints.Insert(1, new("West", "Clear", HintClear));
            hints.Add(new("East", "Back"));
            hints.Add(new("Start", "Home"));
            return hints;
        }

        private void Push(string title, Func<IReadOnlyList<Setting>> rows, string from, bool choices = false) => _pages.Add(new Page(title, rows, from, choices));

        private void Pop()
        {
            if (_pages.Count == 0) return;
            Returned = _pages[^1].From;
            _pages.RemoveAt(_pages.Count - 1);
        }

        private IReadOnlyList<Setting> SectionRows(string section) => section switch
        {
            Controllers => ControllerRows(),
            Display => DisplayRows(),
            Sound => SoundRows(),
            Consoles => ConsoleRows(),
            Library => LibraryRows(),
            _ => EmuSenRows(),
        };

        // A row stepped through named choices, wrapping, its value the chosen one's name.
        private static Setting Choice(string key, string label, string sentence, IReadOnlyList<string> texts, int at, Action<int> set, Func<ScreenPane?>? pane = null, bool enabled = true) =>
            new(new PageRow(key, label) { Kind = PageRowKind.Steps, Status = at >= 0 && at < texts.Count ? texts[at] : texts.FirstOrDefault(), Sentence = sentence, Choices = texts, IsEnabled = enabled },
                pane, by => set(((Math.Max(0, at) + by) % texts.Count + texts.Count) % texts.Count));

        private Setting Switch(string key, string label, string sentence, bool on, Action<bool> set, Func<ScreenPane?>? pane = null) =>
            Choice(key, label, sentence, [On, Off], on ? 0 : 1, i => { set(i == 0); _host.SaveApp(key); }, pane);

        // A percentage stepped by five, clamped rather than wrapped.
        private static Setting Percent(string key, string label, string sentence, int value, Action<int> set, Func<ScreenPane?>? pane = null) =>
            new(new PageRow(key, label) { Kind = PageRowKind.Steps, Status = $"{value}%", Sentence = sentence }, pane, by => set(Math.Clamp(value + 5 * by, 0, 100)));

        private static Setting Opens(string key, string label, string sentence, string? value, Action open, Func<ScreenPane?>? pane = null, bool enabled = true) =>
            new(new PageRow(key, label) { Kind = PageRowKind.Opens, Status = value, Sentence = sentence, IsEnabled = enabled }, pane, Choose: open);

        private static Setting Asks(string key, string label, string sentence, Action ask) =>
            new(new PageRow(key, label) { Kind = PageRowKind.Asks, Sentence = sentence }, Choose: ask);

        private static Setting Act(string key, string label, string sentence, Action act, string? value = null, bool enabled = true) =>
            new(new PageRow(key, label) { Kind = PageRowKind.Action, Status = value, Sentence = sentence, IsEnabled = enabled }, Choose: act);

        private static ScreenPane Said(params string[] lines) => new(lines.Select((l, i) => new SettingsLine(l, i == 0 ? SettingsTone.Text : SettingsTone.Muted)).ToList());

        private static string Count(int n, string one, string many) => n == 1 ? $"1 {one}" : $"{n.ToString("N0", CultureInfo.InvariantCulture)} {many}";
    }
}
