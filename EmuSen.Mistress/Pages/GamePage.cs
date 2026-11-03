using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Pages
{
    // A tile as the page declares it, its picture a file the view loads.
    public sealed record GameTile(string Key, string Label, string? Status = null, string? Picture = null, string? Caption = null, string? Mark = null,
        bool IsEmpty = false, bool IsMarked = false, IReadOnlyList<(string Key, string Text)>? Buttons = null);

    // What the chosen section or deeper page shows, pictures as files.
    public abstract record GameContent
    {
        public sealed record Rows(PageContent.Rows Value) : GameContent;
        public sealed record Tiles(IReadOnlyList<GameTile> Items, double Width, double Height) : GameContent;
        public sealed record Text(string Words, string? Picture, string? Caption) : GameContent;
    }

    // One game's page: its hero, its actions, its five sections and their deeper pages, the words of every row and hint, and what each press does - see EmuSen_BigPicture.md §50.
    public sealed class GamePage
    {
        public const string Overview = "overview", States = "states", Options = "options", Media = "media", Details = "details";
        public const string PlayKey = "play", ResumeKey = "resume", FavouriteKey = "favourite";
        public const string CheatsPage = "Cheats", PackagePage = "Package", EditPage = "Edit Details", CoverPage = "Cover Art", ScrapePage = "Scrape This Game";
        public const string HomeHint = "home", DeleteHint = "delete", ResetHint = "reset", FavouriteHint = "favourite";

        private readonly IGamePageHost _host;
        private string? _deeper;
        private int _resumeAt;
        private PageFocus _focus = new(true, PlayKey);
        private GameSearch? _search;
        private int _viewing = -1;
        private string? _viewerNote;

        public GamePage(IGamePageHost host, string path, bool desktop, string section = Overview)
        {
            _host = host ?? throw new ArgumentNullException(nameof(host));
            Path = path ?? throw new ArgumentNullException(nameof(path));
            Desktop = desktop;
            Section = SectionList().Any(s => s.Key == section) ? section : Overview;
            Read();
        }

        public string Path { get; }

        public bool Desktop { get; }

        public string Section { get; private set; }

        // The deeper page shown under the section, by its name; null on the section itself.
        public string? Deeper => _deeper;

        public GameFacts Facts { get; private set; } = null!;

        public GameArt Art { get; private set; } = null!;

        public IReadOnlyList<GameState> StateList { get; private set; } = [];

        // Raised when anything the view draws has changed.
        public event Action? Changed;

        // The facts line as §4 gives it: console · year · developer · publisher · play time · last played.
        public string FactsLine
        {
            get
            {
                var parts = new List<string> { Facts.Console };
                if (Facts.Year is { Length: > 0 } y) parts.Add(y);
                if (Facts.Developer is { Length: > 0 } d) parts.Add(d);
                if (Facts.Publisher is { Length: > 0 } p && p != Facts.Developer) parts.Add(p);
                if (Facts.Played >= TimeSpan.FromMinutes(1)) parts.Add(PlayedText(Facts.Played));
                parts.Add(Facts.LastPlayed is { } last ? $"Last played {Ago(last, _host.Now).ToLowerInvariant()}" : "Never played");
                return string.Join(" · ", parts);
            }
        }

        public IReadOnlyList<PageAction> Actions
        {
            get
            {
                var actions = new List<PageAction> { new(PlayKey, "Play") { PadButton = PadGlyphButton.South, IsMain = true } };
                List<string> values = StateList.Select(StateWords).ToList();
                if (values.Count > 0)
                    actions.Add(new PageAction(ResumeKey, "Resume") { Value = values[Math.Clamp(_resumeAt, 0, values.Count - 1)], Choices = values });
                actions.Add(new PageAction(FavouriteKey, "Favourite") { PadButton = PadGlyphButton.North, Mark = Facts.Favourite ? PageMark.FilledHeart : PageMark.Heart });
                return actions;
            }
        }

        public IReadOnlyList<AdaptiveSection> SectionList() =>
        [
            new(Overview, "Overview"),
            new(States, "States", Status: StateList.Count > 0 ? StateList.Count.ToString(System.Globalization.CultureInfo.InvariantCulture) : null),
            new(Options, "Options"),
            new(Media, "Media"),
            new(Details, "Details"),
        ];

        // The strip's two names while a deeper page is shown: its parent, then the page.
        public (string Parent, string Title)? Trail => _deeper switch
        {
            null => null,
            CheatsPage or PackagePage => ("Options", _deeper),
            _ => ("Details", _deeper),
        };

        public GameContent Content => _deeper switch
        {
            CheatsPage => new GameContent.Rows(CheatRows()),
            PackagePage => new GameContent.Rows(PackageRows()),
            EditPage => new GameContent.Rows(EditRows()),
            CoverPage => new GameContent.Tiles(CoverTiles(), 150, 176),
            ScrapePage => new GameContent.Rows(ScrapeRows()),
            _ => Section switch
            {
                States => new GameContent.Tiles(StateTiles(), 200, 150),
                Options => new GameContent.Rows(OptionRows()),
                Media => new GameContent.Tiles(MediaTiles(), 200, 150),
                Details => new GameContent.Rows(DetailRows()),
                _ => new GameContent.Text(Facts.Description is { Length: > 0 } words ? words : NoDescription, Art.Screenshot ?? Art.Hero, Art.Screenshot is null && Art.Hero is null ? Facts.Title : null),
            },
        };

        public const string NoDescription = "No description yet. Details ▸ Scrape This Game fetches one, and Edit Details writes one.";

        // The viewer's picture and caption while it is open; null when it is closed.
        public (string? Picture, string Caption)? Viewer
        {
            get
            {
                IReadOnlyList<GameMedia> media = MediaList;
                if (_viewing < 0 || _viewing >= media.Count) return null;
                GameMedia m = media[_viewing];
                string what = MediaLabel(m, media);
                string size = m.Kind == GameMediaKind.Clip ? "played once video arrives" : _host.PictureSize(m.Path) is { } s ? $"{s.Width} × {s.Height}" : "";
                string caption = string.Join(" · ", new[] { $"{what} ({_viewing + 1} of {media.Count})", m.When.ToString("d MMMM yyyy", System.Globalization.CultureInfo.InvariantCulture), size, _viewerNote ?? "" }.Where(p => p.Length > 0));
                return (m.Kind == GameMediaKind.Clip ? Art.Hero ?? Art.Screenshot : m.Path, caption);
            }
        }

        public IReadOnlyList<GameMedia> MediaList { get; private set; } = [];

        // The help bar for what is focused: A's word first, the button that acts on the thing in hand, the shoulders, B and Start.
        public IReadOnlyList<PageHint> Hints(PageFocus focus)
        {
            _focus = focus;
            var hints = new List<PageHint>();
            if (Viewer is not null)
            {
                hints.Add(new PageHint(PadGlyphButton.DPadLeftRight, "Step"));
                if (ViewedTaken) hints.Add(new PageHint(PadGlyphButton.North, "Delete…", DeleteHint));
                hints.Add(new PageHint(PadGlyphButton.East, "Back"));
                return hints;
            }
            string back = _deeper is null ? "Back" : Trail!.Value.Parent;
            if (focus.InActions)
            {
                hints.Add(new PageHint(PadGlyphButton.South, focus.Key switch { ResumeKey => "Resume", FavouriteKey => "Favourite", _ => "Play" }));
                if (focus.Key == ResumeKey && StateList.Count > 1) hints.Add(new PageHint(PadGlyphButton.DPadLeftRight, "Slot"));
                hints.Add(new PageHint(PadGlyphButton.North, "Favourite", FavouriteHint));
            }
            else
            {
                if (AcceptWord(focus.Key) is { } accept) hints.Add(new PageHint(PadGlyphButton.South, accept));
                if (StepWord(focus.Key) is { } step) hints.Add(new PageHint(PadGlyphButton.DPadLeftRight, step));
                if (Section == States && _deeper is null && StateList.Any(s => s.Path == focus.Key)) hints.Add(new PageHint(PadGlyphButton.West, "Delete…", DeleteHint));
                if (_deeper == EditPage) hints.Add(new PageHint(PadGlyphButton.North, "Reset to scraped", ResetHint));
                else hints.Add(new PageHint(PadGlyphButton.North, "Favourite", FavouriteHint));
            }
            if (_deeper is null) hints.Add(new PageHint(PadGlyphButton.Shoulders, "Section"));
            hints.Add(new PageHint(PadGlyphButton.East, _deeper == ScrapePage ? "No" : back));
            hints.Add(new PageHint(PadGlyphButton.Start, "Home", HomeHint));
            return hints;
        }

        private string? AcceptWord(string? key)
        {
            if (key is null) return null;
            switch (_deeper ?? Section)
            {
                case States: return StateList.Any(s => s.Path == key) ? "Load" : null;
                case Media: return "View";
                case CoverPage: return "Use this one";
                case EditPage: return Fields().FirstOrDefault(f => f.Key == key)?.Kind is MetadataKind.Text or MetadataKind.LongText or MetadataKind.Date ? "Edit" : null;
                case ScrapePage: return "Choose";
                case CheatsPage or PackagePage or Options: return RowOf(key)?.Kind is PageRowKind.Steps ? null : "Change";
                default: return "Choose";
            }
        }

        private string? StepWord(string? key) => RowOf(key)?.Kind == PageRowKind.Steps ? (_deeper == CheatsPage ? "On · Off" : "Value") : null;

        private PageRow? RowOf(string? key) => Content is GameContent.Rows r ? r.Value.Items.FirstOrDefault(i => i.Key == key) : null;

        // --- what each press does ---

        public void Act(string key)
        {
            switch (key)
            {
                case PlayKey: _host.Play(Path, null); break;
                case ResumeKey when StateList.Count > 0: _host.Play(Path, StateList[Math.Clamp(_resumeAt, 0, StateList.Count - 1)].Path); break;
                case FavouriteKey: Favourite(); break;
            }
        }

        public void StepAction(string key, int by)
        {
            if (key != ResumeKey || StateList.Count == 0) return;
            _resumeAt = Math.Clamp(_resumeAt + by, 0, StateList.Count - 1);
            Changed?.Invoke();
        }

        public void ChooseSection(string key)
        {
            Section = key;
            _deeper = null;
            Changed?.Invoke();
        }

        // B: the viewer closes, a deeper page goes back to its section, else the page closes; always used.
        public bool Back()
        {
            if (_viewing >= 0) { _viewing = -1; _viewerNote = null; Changed?.Invoke(); return true; }
            if (_deeper is not null)
            {
                ReturnedFrom = _deeper;
                _deeper = null;
                Changed?.Invoke();
                return true;
            }
            _host.Close();
            return true;
        }

        // The deeper page B last left, so the view focuses the row that opened it.
        public string? ReturnedFrom { get; private set; }

        public void Hint(string key)
        {
            switch (key)
            {
                case HomeHint: _host.Close(); break;
                case FavouriteHint: Favourite(); break;
                case ResetHint when _deeper == EditPage && _focus.Key is { } field: _host.ResetField(Path, field); Read(); break;
                case DeleteHint when _viewing >= 0: _ = DeleteViewedAsync(); break;
                case DeleteHint when _focus.Key is { } state && StateList.Any(s => s.Path == state): _ = DeleteStateAsync(state); break;
            }
        }

        private void Favourite()
        {
            _host.ToggleFavourite(Path);
            Read();
        }

        public void ChooseTile(string key)
        {
            if (_deeper == CoverPage) { _host.ChooseCover(Path, key); Read(); return; }
            if (Section == States && StateList.FirstOrDefault(s => s.Path == key) is { } state) { _host.Play(Path, state.Path); return; }
            if (Section == Media)
            {
                _viewing = MediaList.ToList().FindIndex(m => m.Path == key);
                _viewerNote = null;
                Changed?.Invoke();
            }
        }

        public void TileButton(string tile, string button)
        {
            if (button == "load") ChooseTile(tile);
            else if (button == "delete") _ = DeleteStateAsync(tile);
        }

        public void StepViewer(int by)
        {
            if (_viewing < 0 || MediaList.Count == 0) return;
            _viewing = ((_viewing + by) % MediaList.Count + MediaList.Count) % MediaList.Count;
            _viewerNote = null;
            Changed?.Invoke();
        }

        private bool ViewedTaken => _viewing >= 0 && _viewing < MediaList.Count && MediaList[_viewing].Kind == GameMediaKind.Taken;

        private async Task DeleteViewedAsync()
        {
            if (!ViewedTaken) return;
            GameMedia m = MediaList[_viewing];
            if (!await _host.ConfirmAsync("Delete this screenshot?", $"The screenshot of {Facts.Title} taken {m.When:d MMMM yyyy} is deleted. This cannot be undone.", "Delete")) return;
            if (_host.DeleteMedia(Path, m.Path)) { Read(); _viewing = Math.Min(_viewing, MediaList.Count - 1); }
            Changed?.Invoke();
        }

        private async Task DeleteStateAsync(string state)
        {
            if (StateList.FirstOrDefault(s => s.Path == state) is not { } s) return;
            if (!await _host.ConfirmAsync($"Delete {s.Slot}?", $"{Facts.Title}'s {s.Slot}, saved {Ago(s.When, _host.Now).ToLowerInvariant()}, is deleted. This cannot be undone.", "Delete")) return;
            if (_host.DeleteState(Path, state)) Read();
        }

        public void ChooseRow(string key) => _ = ChooseRowAsync(key);

        private async Task ChooseRowAsync(string key)
        {
            switch (_deeper ?? Section)
            {
                case Options:
                    switch (key)
                    {
                        case "cheats": Open(CheatsPage); break;
                        case "package": Open(PackagePage); break;
                        case "crop": _host.OpenCrop(Path, Facts.Title); break;
                    }
                    break;
                case Details:
                    switch (key)
                    {
                        case "scrape": await OpenScrapeAsync(); break;
                        case "edit": Open(EditPage); break;
                        case "cover": Open(CoverPage); break;
                        case "folder": _host.OpenFolder(Path); break;
                        case "hide":
                            if (await _host.ConfirmAsync($"Hide {Facts.Title}?", "It leaves every library view until Settings lists hidden games again. Its file is not touched.", "Hide")) _host.Hide(Path);
                            break;
                    }
                    break;
                case CheatsPage when key == "add":
                    if (await _host.TypeAsync("Add a code", null) is { Length: > 0 } code) { _cheatNote = _host.AddCode(Path, code); Read(); }
                    break;
                case PackagePage:
                    await PackageRowAsync(key);
                    break;
                case EditPage when key == ClearRow:
                    if (await _host.ConfirmAsync($"Clear {Facts.Title}'s scraped data?", "Its edits, and ScreenScraper's text and pictures in EmuSen's own store, are removed. The game's file and the covers placed by hand stay.", "Clear")) { _host.ClearMetadata(Path); Read(); }
                    break;
                case EditPage when Fields().FirstOrDefault(f => f.Key == key) is { Kind: MetadataKind.Text or MetadataKind.LongText or MetadataKind.Date } field:
                    if (await _host.TypeAsync(field.Label, field.Value) is { } typed) { _host.EditField(Path, key, typed); Read(); }
                    break;
                case ScrapePage:
                    await ScrapeRowAsync(key);
                    break;
            }
        }

        private string? _cheatNote;

        private void Open(string page)
        {
            _deeper = page;
            _cheatNote = null;
            Changed?.Invoke();
        }

        public void StepRow(string key, int by)
        {
            switch (_deeper ?? Section)
            {
                case Options when key == "engine":
                    _host.SetEngine(Path, Step(_host.Engines(Path), _host.Engine(Path), by));
                    break;
                case Options when key == "profile" && _host.Profiles(Path) is { } profiles:
                    _host.SetProfile(Path, Step(profiles, _host.Profile(Path), by));
                    break;
                case Options when key == "standard" && _host.Standards(Path) is { } standards:
                    _host.SetStandard(Path, Step(standards, _host.Standard(Path), by));
                    break;
                case CheatsPage when int.TryParse(key, out int id) && _host.Cheats(Path)?.FirstOrDefault(c => c.Id == id) is { } cheat:
                    _host.SetCheat(Path, id, !cheat.Enabled);
                    break;
                case PackagePage when key == "identity":
                    _host.PackageIdentity(Path, !_host.Package(Path).Patched);
                    break;
                case EditPage when Fields().FirstOrDefault(f => f.Key == key) is { Choices: { Count: > 0 } choices } field:
                    _host.EditField(Path, key, Step(choices, field.Value, by));
                    break;
                default: return;
            }
            Read();
        }

        // A text row typed in on the desktop: the editor's fields, kept at once.
        public void EditRow(string key, string text)
        {
            if (_deeper != EditPage) return;
            _host.EditField(Path, key, text);
            Read();
        }

        public void ContentButton(string key) => ChooseRow(key);

        private static string? Step(IReadOnlyList<GameChoice> choices, string? current, int by)
        {
            int at = Math.Max(0, choices.ToList().FindIndex(c => (c.Value ?? "") == (current ?? "")));
            return choices[((at + by) % choices.Count + choices.Count) % choices.Count].Value;
        }

        // --- the sections' rows and tiles ---

        private IReadOnlyList<GameTile> StateTiles()
        {
            var tiles = new List<GameTile>();
            GameState? resumed = StateList.Count > 0 ? StateList[Math.Clamp(_resumeAt, 0, StateList.Count - 1)] : null;
            IReadOnlyList<(string Key, string Text)>? buttons = Desktop ? [("load", "Load"), ("delete", "Delete")] : null;
            var bySlot = StateList.ToDictionary(s => s.Slot, StringComparer.Ordinal);
            for (int n = 1; n <= _host.StateSlots; n++)
            {
                string slot = $"Slot {n}";
                tiles.Add(bySlot.TryGetValue(slot, out GameState? s)
                    ? new GameTile(s.Path, slot, Ago(s.When, _host.Now), s.Picture, s.Picture is null ? $"{Facts.Title} · {slot}" : null, IsMarked: ReferenceEquals(s, resumed), Buttons: buttons)
                    : new GameTile("empty." + n, slot, "Empty", IsEmpty: true));
                if (n == LastUsedSlot && bySlot.TryGetValue(AutoSlot, out GameState? auto))
                    tiles.Add(new GameTile(auto.Path, "Auto", $"Where you left off · {Ago(auto.When, _host.Now)}", auto.Picture, auto.Picture is null ? $"{Facts.Title} · Auto" : null, IsMarked: ReferenceEquals(auto, resumed), Buttons: buttons));
            }
            return tiles;
        }

        public const string AutoSlot = "Auto";

        // The automatic state stands after the last slot in use, before the empty ones, as the design draws it.
        private int LastUsedSlot => Math.Max(1, Enumerable.Range(1, _host.StateSlots).LastOrDefault(n => StateList.Any(s => s.Slot == $"Slot {n}")));

        private PageContent.Rows OptionRows()
        {
            var rows = new List<PageRow>();
            IReadOnlyList<GameChoice> engines = _host.Engines(Path);
            if (engines.Count > 0)
                rows.Add(new PageRow("engine", "Engine")
                {
                    Kind = engines.Count > 1 ? PageRowKind.Steps : PageRowKind.Action, IsEnabled = engines.Count > 1,
                    Status = TextOf(engines, _host.Engine(Path)), Choices = engines.Select(e => e.Text).ToList(),
                    Sentence = "The engine this game runs on; the console's own unless one is chosen here. Cores lists them all.",
                });
            if (_host.Profiles(Path) is { } profiles)
                rows.Add(new PageRow("profile", "Controller profile")
                {
                    Kind = PageRowKind.Steps, Status = TextOf(profiles, _host.Profile(Path)), Choices = profiles.Select(p => p.Text).ToList(),
                    Sentence = "The bindings this game starts with; the console's profile unless one is chosen here. Settings ▸ Controllers ▸ Profiles makes them.",
                });
            if (_host.Cheats(Path) is { } cheats)
                rows.Add(new PageRow("cheats", "Cheats") { Kind = PageRowKind.Opens, Status = $"{cheats.Count(c => c.Enabled)} of {cheats.Count} on", Sentence = "The game's cheats, on or off; the same switches the Pause Card shows." });
            if (_host.Crop(Path) is { } crop)
                rows.Add(new PageRow("crop", "Display crop") { Kind = PageRowKind.Opens, Status = crop, Sentence = "How much of the picture's edge is cut away for this game." });
            if (_host.Standards(Path) is { } standards)
                rows.Add(new PageRow("standard", "Television standard")
                {
                    Kind = PageRowKind.Steps, Status = TextOf(standards, _host.Standard(Path)), Choices = standards.Select(s => s.Text).ToList(),
                    Sentence = "The television the picture is drawn for: the game's own, NTSC or PAL.",
                });
            GamePackageFacts package = _host.Package(Path);
            rows.Add(new PageRow("package", "Package") { Kind = PageRowKind.Opens, Status = package.Mode, Sentence = "The game's package and its layers, from Packages." });
            return new PageContent.Rows(rows);
        }

        private static string TextOf(IReadOnlyList<GameChoice> choices, string? value) =>
            choices.FirstOrDefault(c => (c.Value ?? "") == (value ?? ""))?.Text ?? choices.FirstOrDefault()?.Text ?? "";

        private PageContent.Rows DetailRows()
        {
            int covers = _host.Covers(Path).Count;
            string coverStatus = (Art.Cover is null ? "Generated" : _host.Covers(Path).FirstOrDefault(c => c.InUse)?.Label ?? "Scraped") + (covers > 1 ? $" · {covers} to choose from" : "");
            var actions = new List<PageRow>
            {
                new("scrape", "Scrape This Game") { Kind = PageRowKind.Asks, IsEnabled = !_host.ScrapeRunning, Status = _host.ScrapeRunning ? "A run is going" : null },
                new("edit", "Edit Details") { Kind = PageRowKind.Opens },
                new("cover", "Cover art") { Kind = PageRowKind.Opens, Status = coverStatus },
                new("folder", "Open Folder"),
                new("hide", "Hide from Library") { Kind = PageRowKind.Asks },
            };
            if (!Desktop) return new PageContent.Rows(actions);
            // The desktop shows the metadata as a form, with the actions as its buttons (EmuSen_Interface.md §17.4).
            var facts = Fields().Where(f => f.Value is { Length: > 0 } && f.Kind is not MetadataKind.Flag && f.Key != GameMetadata.Description)
                .Select(f => new PageRow("field." + f.Key, f.Label) { Status = f.Choices?.FirstOrDefault(c => c.Value == f.Value)?.Text ?? f.Value, IsEnabled = false }).ToList();
            return new PageContent.Rows(facts, null, [("scrape", "Scrape This Game…"), ("edit", "Edit"), ("cover", "Cover Art"), ("folder", "Open Folder"), ("hide", "Hide from Library…")]);
        }

        private PageContent.Rows CheatRows()
        {
            var rows = (_host.Cheats(Path) ?? []).Select(c => new PageRow(c.Id.ToString(System.Globalization.CultureInfo.InvariantCulture), c.Description.Length > 0 ? c.Description : $"Cheat {c.Id}")
            {
                Kind = PageRowKind.Steps, Status = c.Enabled ? "On" : "Off", Choices = ["Off", "On"],
            }).ToList();
            if (_host.CheatsTakeCodes(Path)) rows.Add(new PageRow("add", "Add a code…") { Kind = PageRowKind.Action });
            return new PageContent.Rows(rows, _cheatNote ?? (rows.Count == 0 ? "No cheats for this game yet." : null));
        }

        private PageContent.Rows PackageRows()
        {
            GamePackageFacts p = _host.Package(Path);
            var rows = new List<PageRow>();
            rows.Add(p.IsPackage
                ? new PageRow("mode", "Mode") { Status = p.Mode, IsEnabled = false }
                : new PageRow("make", "Make Package") { Kind = PageRowKind.Asks, IsEnabled = p.CanMake });
            int n = 0;
            foreach ((string name, string format) in p.Layers)
                rows.Add(new PageRow("layer." + ++n, $"Layer: {name}") { Kind = PageRowKind.Asks, Status = format, IsEnabled = p.CanChange, Sentence = "A soft patch, applied in order each time the game loads; A takes it out." });
            if (p.IsPackage && p.TakesPatches)
                foreach (string file in p.Beside)
                    rows.Add(new PageRow("add." + file, $"Add {System.IO.Path.GetFileName(file)}") { Kind = PageRowKind.Asks, IsEnabled = p.CanChange });
            if (p.IsPackage && p.Layers.Count > 0)
                rows.Add(new PageRow("identity", "Scraped as") { Kind = PageRowKind.Steps, Status = p.Patched ? "The patched game" : "The original dump", Choices = ["The original dump", "The patched game"], IsEnabled = p.CanChange });
            if (p.IsPackage) rows.Add(new PageRow("migrate", "Copy Saves and Art In") { Kind = PageRowKind.Asks, IsEnabled = p.CanChange });
            return new PageContent.Rows(rows, p.Status);
        }

        private async Task PackageRowAsync(string key)
        {
            GamePackageFacts p = _host.Package(Path);
            if (key == "make")
            {
                if (await _host.ConfirmAsync($"Make {Facts.Title}'s package?", "Its saves, states, settings, art and cheats then live in one folder under Packages, linked to the game where it is. The library is only read.", "Make")) _host.PackageMake(Path);
            }
            else if (key.StartsWith("layer.", StringComparison.Ordinal) && int.TryParse(key["layer.".Length..], out int layer))
            {
                if (await _host.ConfirmAsync($"Take out {p.Layers[layer - 1].Name}?", "The patch leaves the package; the game's file is not touched.", "Take Out")) _host.PackageRemove(Path, layer);
            }
            else if (key.StartsWith("add.", StringComparison.Ordinal))
            {
                string file = key["add.".Length..];
                if (await _host.ConfirmAsync($"Add {System.IO.Path.GetFileName(file)}?", "It is checked against the game, then copied into the package and applied each time the game loads.", "Add")) _host.PackageAdd(Path, file);
            }
            else if (key == "migrate")
            {
                if (await _host.ConfirmAsync("Copy saves and art in?", "The game's saves, states, settings, art and cheats are copied into the package. Nothing is overwritten or removed.", "Copy")) _host.PackageMigrate(Path);
            }
            Read();
        }

        private IReadOnlyList<GameField> Fields() => _host.Fields(Path);

        // The metadata as rows, the engine left to Options; ES-DE's Clear last, which asks first.
        private PageContent.Rows EditRows() => new(Fields().Where(f => f.Key != GameMetadata.AltEmulator).Select(f => new PageRow(f.Key, f.Label)
        {
            Kind = f.Kind switch { MetadataKind.Text or MetadataKind.LongText or MetadataKind.Date => PageRowKind.Text, _ => PageRowKind.Steps },
            Status = f.Choices is { } c ? c.FirstOrDefault(x => (x.Value ?? "") == (f.Value ?? ""))?.Text ?? f.Value : f.Kind == MetadataKind.LongText ? Shorten(f.Value) : f.Value,
            Choices = f.Choices?.Select(x => x.Text).ToList(),
        }).Append(new PageRow(ClearRow, "Clear Scraped Data") { Kind = PageRowKind.Asks, Sentence = "The edits, and ScreenScraper's text and pictures in EmuSen's own store; the game's file and the player's covers stay." }).ToList());

        public const string ClearRow = "clear";

        // A long text's first words as its row's status; the whole of it opens with A.
        private static string? Shorten(string? text) => text is null ? null : text.Length <= 24 ? text : text[..24].TrimEnd() + "…";

        private IReadOnlyList<GameTile> CoverTiles() =>
            _host.Covers(Path).Select(c => new GameTile(c.Key, c.Label, null, c.Picture, c.Picture is null ? Facts.Title : null, Facts.Mark, IsMarked: c.InUse)).ToList();

        private IReadOnlyList<GameTile> MediaTiles()
        {
            IReadOnlyList<GameMedia> media = MediaList;
            return media.Select(m => new GameTile(m.Path, MediaLabel(m, media), Picture: m.Kind == GameMediaKind.Clip ? Art.Hero : m.Path, Caption: m.Kind == GameMediaKind.Clip ? $"{Facts.Title} · Clip" : null)).ToList();
        }

        private static string MediaLabel(GameMedia m, IReadOnlyList<GameMedia> all) => m.Kind switch
        {
            GameMediaKind.Taken => $"Screenshot {all.Where(x => x.Kind == GameMediaKind.Taken).ToList().IndexOf(m) + 1}",
            GameMediaKind.Screenshot => "Scraped screenshot",
            GameMediaKind.TitleScreen => "Title screen",
            GameMediaKind.Marquee => "Marquee",
            GameMediaKind.Fanart => "Fan art",
            _ => "Clip",
        };

        // --- Scrape This Game: the question form with the matches as rows ---

        private async Task OpenScrapeAsync()
        {
            _search = new GameSearch([], _host.WhyNoSearch is { } why ? $"ScreenScraper cannot be asked: {why}." : "Asking ScreenScraper for games of this name…");
            Open(ScrapePage);
            if (_host.WhyNoSearch is null) await SearchAsync(_host.SearchName(Path));
        }

        private async Task SearchAsync(string name)
        {
            _search = await _host.SearchAsync(Path, name);
            if (_deeper == ScrapePage) Changed?.Invoke();
        }

        private PageContent.Rows ScrapeRows()
        {
            var rows = (_search?.Matches ?? []).Select(m => new PageRow("match." + m.Key, m.Label)).ToList();
            if (_host.WhyNoSearch is null) rows.Add(new PageRow("other", "Search by another name…") { Kind = PageRowKind.Text });
            if (!_host.ScrapeRunning) rows.Add(new PageRow("file", "By the file's checksums…"));
            rows.Add(new PageRow("no", "No"));
            string sentence = _search?.Why ?? (_search?.Matches.Count > 0
                ? "The match chosen gives the game its name, facts, cover and pictures, fetched by one paced run."
                : "ScreenScraper knows no game of that name.");
            return new PageContent.Rows(rows, sentence);
        }

        private async Task ScrapeRowAsync(string key)
        {
            if (key == "no") { Back(); return; }
            if (key == "other")
            {
                if (await _host.TypeAsync("Search by another name", _host.SearchName(Path)) is { Length: > 0 } name) await SearchAsync(name);
                return;
            }
            if (key == "file")
            {
                Back();
                await _host.ScrapeByFileAsync(Path);
                return;
            }
            if (key.StartsWith("match.", StringComparison.Ordinal) && _host.ChooseMatch(Path, key["match.".Length..])) Back();
        }

        // --- reading the game ---

        // Reads everything again from the host and tells the view.
        public void Read()
        {
            Facts = _host.Facts(Path);
            Art = _host.Art(Path);
            StateList = _host.States(Path);
            MediaList = _host.Media(Path);
            _resumeAt = Math.Clamp(_resumeAt, 0, Math.Max(0, StateList.Count - 1));
            Changed?.Invoke();
        }

        // A state's words in Resume's chooser: its place and how long ago.
        private string StateWords(GameState s) => $"{s.Slot} · {Ago(s.When, _host.Now).ToLowerInvariant()}";

        internal static string PlayedText(TimeSpan played) =>
            played.TotalHours >= 1 ? $"{(int)played.TotalHours} h {played.Minutes} min" : $"{Math.Max(1, (int)played.TotalMinutes)} min";

        // How long ago, as the page says it: just now, 12 min ago, 2 h ago, Yesterday, 3 days ago, then the date.
        public static string Ago(DateTime when, DateTime now)
        {
            TimeSpan since = now - when;
            if (since.TotalMinutes < 1) return "Just now";
            if (since.TotalHours < 1) return $"{(int)since.TotalMinutes} min ago";
            if (when.Date == now.Date) return $"{(int)since.TotalHours} h ago";
            if (when.Date == now.Date.AddDays(-1)) return "Yesterday";
            if (since.TotalDays < 14) return $"{(now.Date - when.Date).Days} days ago";
            return when.ToString("d MMM yyyy", System.Globalization.CultureInfo.InvariantCulture);
        }
    }
}
