using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using static EmuSen.Tools.Packages.ToolEvents;
using System.Text.Json.Nodes;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Tools.Packages
{
    // One layer of a package as its controls show it: today a soft patch, the one kind the pkg jobs know - see EmuSen_Packages.md §8.
    public sealed record PackageLayer(string Kind, string Name, string Format, int Index);

    // One game's package controls, hosted by Packages' detail and the Game Package window: Make Package, its layers added and taken out, the identity the scraper sends, and Migrate, through the library tool's pkg jobs; the library is only read - see EmuSen_Packages.md §8.
    public sealed class GamePackageDetail : StackPanel
    {
        // How a job is started and a patch file chosen on the desktop, which a test replaces.
        internal static Func<string, IToolJob> StartJob = LibraryToolJob.Start;
        internal static Func<bool> ToolAvailable = () => LibraryToolJob.Available;
        internal static Func<TopLevel?, Task<string?>> PickPatch = async owner =>
            owner is Window window && await Dialogs.PickFileAsync(window, "Add a Soft Patch", new[] { new Avalonia.Platform.Storage.FilePickerFileType("IPS, UPS and BPS patches") { Patterns = new[] { "*.ips", "*.ups", "*.bps" } } }) is { } file ? file.Path : null;

        public static readonly TimeSpan PollEvery = TimeSpan.FromMilliseconds(100);

        public static readonly string[] PatchExtensions = [".ips", ".ups", ".bps"];

        private readonly string _game;
        private readonly string? _library;
        private readonly bool _browse;
        private readonly Button _make, _identity, _migrate;
        private readonly TextBlock _packageText = new() { Name = "GamePackagePlace", TextWrapping = TextWrapping.Wrap };
        private readonly StackPanel _layers = Ui.Stack(6);
        private readonly WrapPanel _adds = new() { Name = "GamePackageAdds", Orientation = Orientation.Horizontal, ItemSpacing = 8, LineSpacing = 8 };
        private readonly HintText _patchHint = new() { Name = "GamePackagePatchHint" };
        private readonly HintText _status = new() { Name = "GamePackageStatus" };
        private readonly TextBlock _identityText = new() { Name = "GamePackageIdentity", TextWrapping = TextWrapping.Wrap };
        private readonly FieldRow _patchRow, _addRow, _identityRow, _migrateRow;
        private readonly DispatcherTimer _timer;
        private IToolJob? _job;
        private string _op = "";

        // refused: the package claims the game but will not open with the file as it is now, so the game plays as a plain one - see EmuSen_GamePackages.md §14.8 Q34.
        public GamePackageDetail(string title, string game, string? library, string? package, bool browse, bool refused = false)
        {
            Name = "GamePackageDetail";
            Spacing = 16;
            Refused = refused;
            (_game, _library, _browse, Package) = (game, library, browse, package);

            _make = Ui.Button("Make Package", Make);
            _make.Name = "GamePackageMake";
            _layers.Name = "GamePackagePatches";
            Control about = Ui.Hint($"{title}: a package keeps everything EmuSen holds for the game in one folder under {DataStore.Packages}, and links the game where it is. The library is only read: nothing in it is changed, moved or removed.");
            about.Name = "GamePackageAbout";
            _patchRow = new FieldRow { Label = "Soft patches", Hint = "Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original.", Content = Ui.Stack(6, _layers, _patchHint) };
            _addRow = new FieldRow { Label = "Add a patch", Hint = "An IPS, UPS or BPS file, checked against the game before it is copied into the package.", Content = _adds };
            _identity = Ui.Button("Scrape as Patched", ToggleIdentity);
            _identity.Name = "GamePackageIdentitySwitch";
            _identityRow = new FieldRow { Label = "Scraped as", Hint = "Which hashes the scraper sends: the dump's, or the patched game's first for a hack ScreenScraper lists in its own right. The saves stay the patch list's either way.", Content = Ui.Stack(8, _identityText, Ui.Buttons(_identity)) };
            _migrate = Ui.Button("Copy Saves and Art In", Migrate);
            _migrate.Name = "GamePackageMigrate";
            _migrate.HorizontalAlignment = HorizontalAlignment.Left;
            _migrateRow = new FieldRow { Label = "Stores", Hint = "Copies the game's saves, states and their records, settings, art and cheats into the package, each under the package's name. Nothing is overwritten or removed.", Content = _migrate };
            Children.Add(about);
            Children.Add(new FieldRow { Label = "Package", Content = Ui.Stack(8, _packageText, Ui.Buttons(_make)) });
            Children.Add(_patchRow);
            Children.Add(_addRow);
            Children.Add(_identityRow);
            Children.Add(_migrateRow);
            Children.Add(_status);

            _timer = new DispatcherTimer { Interval = PollEvery };
            _timer.Tick += (_, _) => Poll();
            DetachedFromVisualTree += (_, _) => Stop();
            if (!ToolAvailable()) _status.Text = $"Packages are made by EmuSen's platform library, which is not in use here ({LibraryToolJob.Report}).";
            Render();
        }

        // The package's folder, null until there is one.
        public string? Package { get; private set; }

        public bool Refused { get; }

        // Whether a package was made or changed since the controls were shown, so the host lists the library again.
        public bool Changed { get; private set; }

        // Raised after each job that made or changed the package.
        public event Action? PackageChanged;

        public bool Busy => _job is not null;

        public string? Status => _status.Text;

        // Stops a job still running, as the host closes.
        public void Stop()
        {
            _timer.Stop();
            _job?.Dispose();
            _job = null;
        }

        // The game's path as the library names it, with / between its parts; null for a game outside it.
        private string? LibraryPath()
        {
            if (string.IsNullOrWhiteSpace(_library)) return null;
            string rel = Path.GetRelativePath(Path.GetFullPath(_library), Path.GetFullPath(_game));
            return rel.StartsWith("..", StringComparison.Ordinal) || Path.IsPathRooted(rel) ? null : rel.Replace('\\', '/');
        }

        private string PackagePath() => Path.GetRelativePath(DataStore.Packages, Package!).Replace('\\', '/');

        // A disc game takes no soft patch here (EmuSen_GamePackages.md §9 Q15).
        private bool IsDisc => ImageSets.IsSheet(_game) || ImageSets.IsPlaylist(_game) || _game.EndsWith(".chd", StringComparison.OrdinalIgnoreCase);

        // The manifest as it now stands, null where there is none or it does not read.
        private JsonObject? Manifest()
        {
            if (Package is null) return null;
            try { return GamePackage.Read(Path.Combine(Package, GamePackage.FileName)).Document; }
            catch (Exception e) when (e is IOException or FormatException or UnauthorizedAccessException) { return null; }
        }

        public bool SelfContained => (string?)Manifest()?["mode"] == "self-contained";

        public string Identity => (string?)Manifest()?["identity"] ?? "original";

        // Where patch files are looked for: beside the game in the library, or for a game kept in its package beside the library file it was copied from.
        private string? PatchFolder()
        {
            if (Package is null || !GamePackages.Within(Path.GetFullPath(_game), Path.GetFullPath(Package))) return Path.GetDirectoryName(_game);
            if (_library is null || Manifest()?["sources"] is not JsonArray sources) return null;
            string? rel = sources.OfType<JsonObject>().Where(e => (string?)e["where"] == "library").Select(e => (string?)e["path"]).FirstOrDefault();
            return rel is null ? null : Path.GetDirectoryName(Path.Combine(_library, rel.Replace('/', Path.DirectorySeparatorChar)));
        }

        // The patch files beside the game in the library, as RetroArch and ES-DE keep a hack beside its game.
        public IReadOnlyList<string> Beside()
        {
            string? folder = PatchFolder();
            if (folder is null || !Directory.Exists(folder)) return [];
            try
            {
                return Directory.EnumerateFiles(folder).Where(f => PatchExtensions.Contains(Path.GetExtension(f), StringComparer.OrdinalIgnoreCase)).Order(StringComparer.CurrentCultureIgnoreCase).ToList();
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return []; }
        }

        private void Make()
        {
            if (LibraryPath() is not { } rel)
            {
                _status.Text = "Only a game in the library is made into a package here.";
                Render();
                return;
            }
            Run("make", JsonSerializer.Serialize(new { op = "pkg-make", paths = new[] { rel } }));
        }

        public void Add(string file) => Run("add", JsonSerializer.Serialize(new { op = "pkg-patch-add", paths = new[] { PackagePath() }, patch = file }));

        public void Remove(int n) => Run("remove", JsonSerializer.Serialize(new { op = "pkg-patch-rm", paths = new[] { PackagePath() }, index = n }));

        public void SetIdentity(string identity) => Run("identity", JsonSerializer.Serialize(new { op = "pkg-identity", paths = new[] { PackagePath() }, identity }));

        public void Migrate() => Run("migrate", JsonSerializer.Serialize(new { op = "pkg-migrate", paths = new[] { PackagePath() } }));

        private void ToggleIdentity() => SetIdentity(Identity == "patched" ? "original" : "patched");

        private async Task BrowseAsync()
        {
            if (await PickPatch(TopLevel.GetTopLevel(this)) is { } file) Add(file);
        }

        private void Run(string op, string request)
        {
            if (_job is not null) return;
            try { _job = StartJob(request); }
            catch (Exception e) when (e is FormatException or InvalidOperationException)
            {
                _status.Text = ToolWords.Refusal(e.Message);
                Render();
                return;
            }
            _op = op;
            _status.Text = op switch
            {
                "make" => "Making the package. The game stays where it is.",
                "add" => "Checking the patch against the game.",
                "identity" => "Setting what the scraper sends.",
                "migrate" => "Copying the game's saves, states, settings, art and cheats in.",
                _ => "Taking the patch out.",
            };
            _timer.Start();
            Render();
        }

        // Every event the job has queued taken and shown; a test calls it in place of the timer.
        public void Poll()
        {
            if (_job is null) return;
            for (int i = 0; i < 64; i++)
            {
                string? line;
                try { line = _job.Next(0); }
                catch (InvalidOperationException e)
                {
                    _status.Text = ToolWords.Refusal(e.Message);
                    break;
                }
                if (line is null) break;
                Apply(line);
            }
            if (_job is { Ended: true })
            {
                _job.Dispose();
                _job = null;
                _timer.Stop();
                if (Changed) PackageChanged?.Invoke();
            }
            Render();
        }

        private static int Count(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) ? v.ValueKind == JsonValueKind.Array ? v.GetArrayLength() : v.ValueKind == JsonValueKind.Number ? v.GetInt32() : 0 : 0;

        // One event of the pkg job applied: the package made, a layer added or removed, the stores copied in, or why not.
        public void Apply(string line)
        {
            using JsonDocument doc = JsonDocument.Parse(line);
            JsonElement e = doc.RootElement;
            switch (Text(e, "event"))
            {
                case "start" when e.TryGetProperty("refused", out JsonElement refused) && refused.ValueKind == JsonValueKind.Array && refused.GetArrayLength() > 0:
                    _status.Text = string.Join(" ", refused.EnumerateArray().Select(r => r.GetString()));
                    break;
                case "package":
                    string state = Text(e, "state") ?? "";
                    string? words = Text(e, "words");
                    if (state is "made" or "already" && Text(e, "package") is { } rel) Package = Path.Combine(DataStore.Packages, rel.Replace('/', Path.DirectorySeparatorChar));
                    if (state is "made" or "added" or "removed" or "set" or "migrated") Changed = true;
                    _status.Text = state switch
                    {
                        "set" or "already" when _op == "identity" => Text(e, "identity") == "patched" ? "The scraper now sends the patched game's hashes first, then the dump's." : "The scraper now sends the dump's hashes.",
                        "made" => "The package was made, linked to the game where it is.",
                        "already" => "The game already has a package.",
                        "added" => $"Added {Text(e, "patch")}. " + (e.TryGetProperty("retroarch", out JsonElement ra) && ra.ValueKind == JsonValueKind.True ? "It is named as RetroArch finds it." : "RetroArch will not apply this list, whose files keep their own names."),
                        "removed" => $"Took out {Text(e, "patch")}.",
                        "migrated" => $"Copied {Count(e, "copied")} file(s) in; {Count(e, "kept")} already there were kept.",
                        "nothing" => "There was nothing to copy in: the package already holds everything EmuSen kept for the game.",
                        _ => words is null ? $"Not done: {state}." : $"Not done: {words}",
                    };
                    break;
                case "end" when Text(e, "error") is { } error:
                    _status.Text = ToolWords.Refusal(error);
                    break;
            }
        }

        // The package's layers in order, read from its manifest; today its soft patches.
        public IReadOnlyList<PackageLayer> Layers()
        {
            if (Package is null) return [];
            try
            {
                return GamePackage.Read(Path.Combine(Package, GamePackage.FileName)).Document["patches"] is JsonArray list
                    ? list.OfType<JsonObject>().Select((p, i) => new PackageLayer("patch", (string?)p["file"] ?? "", ((string?)p["format"] ?? "").ToUpperInvariant(), i + 1)).ToList()
                    : [];
            }
            catch (Exception e) when (e is IOException or FormatException or UnauthorizedAccessException) { return []; }
        }

        // The manifest's patch list, read for showing: each file and its format.
        public IReadOnlyList<(string File, string Format)> Patches() => Layers().Where(l => l.Kind == "patch").Select(l => (l.Name, l.Format)).ToList();

        // The rows, the buttons and the words as the package and the job allow.
        private void Render()
        {
            bool running = _job is not null, tool = ToolAvailable();
            _packageText.Text = Package is null ? "Not a package yet."
                : Refused ? $"Packages/{PackagePath()}, changed: the game's file is not as the package recorded it (pkg check names why, most often another dump), so the game plays as a plain one with its plain saves, and the package refuses to open until it is checked."
                : SelfContained ? $"Packages/{PackagePath()}, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game."
                : $"Packages/{PackagePath()}, linked to the game where it is in the library.";
            _make.IsVisible = Package is null;
            _make.IsEnabled = tool && !running && LibraryPath() is not null;

            _layers.Children.Clear();
            IReadOnlyList<PackageLayer> layers = Layers();
            foreach (PackageLayer layer in layers)
            {
                var name = new TextBlock { Text = $"{layer.Index}. {layer.Name} ({layer.Format})", TextWrapping = TextWrapping.Wrap, VerticalAlignment = VerticalAlignment.Center };
                int n = layer.Index;
                Button remove = Ui.Button("Remove", () => Remove(n));
                remove.Name = $"GamePackagePatch.{n - 1}.Remove";
                remove.IsEnabled = tool && !running;
                var row = new DockPanel { LastChildFill = true };
                DockPanel.SetDock(remove, Dock.Right);
                row.Children.Add(remove);
                row.Children.Add(name);
                _layers.Children.Add(row);
            }
            bool candidates = _browse || Beside().Count > 0;
            string none = layers.Count == 0 ? "No patches: the game is played as dumped." : "";
            string where = candidates ? "" : " On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none.";
            _patchHint.Text = Package is null ? "Make the package first; its patches live in it." : IsDisc ? "Disc games take no soft patches here." : (none + where).Trim();
            _patchHint.IsVisible = _patchHint.Text.Length > 0;

            _adds.Children.Clear();
            bool adding = Package is not null && !IsDisc && tool && !running;
            int k = 0;
            foreach (string file in Beside())
            {
                Button add = Ui.Button($"Add {Path.GetFileName(file)}", () => Add(file));
                add.Name = $"GamePackageAdd.{k++}";
                add.IsEnabled = adding;
                _adds.Children.Add(add);
            }
            if (_browse)
            {
                Button browse = Ui.Button("Browse...", () => _ = BrowseAsync());
                browse.Name = "GamePackageBrowse";
                browse.IsEnabled = adding;
                _adds.Children.Add(browse);
            }
            _addRow.IsVisible = Package is not null && !IsDisc && _adds.Children.Count > 0;

            bool patched = Identity == "patched";
            _identityRow.IsVisible = Package is not null && !Refused && layers.Count > 0;
            _identityText.Text = patched ? "The patched game: its hashes first, then the dump's." : "The original dump.";
            _identity.Content = patched ? "Scrape as Original" : "Scrape as Patched";
            _identity.IsEnabled = tool && !running;

            _migrateRow.IsVisible = Package is not null && !Refused;
            _migrate.IsEnabled = tool && !running;
            _status.IsVisible = !string.IsNullOrEmpty(_status.Text);
        }
    }
}
