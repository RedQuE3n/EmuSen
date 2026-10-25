using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
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

namespace EmuSen.Mistress.Views
{
    // One game's package: Make Package, and its soft patches added and removed, through the library tool's pkg jobs; the library is only read - see EmuSen_GamePackages.md §15.
    public sealed class GamePackageWindow : ToolWindow
    {
        // How a job is started and a patch file chosen on the desktop, which a test replaces.
        internal static Func<string, IToolJob> StartJob = LibraryToolJob.Start;
        internal static Func<bool> ToolAvailable = () => LibraryToolJob.Available;
        internal static Func<Window, Task<string?>> PickPatch = async owner =>
            await Dialogs.PickFileAsync(owner, "Add a Soft Patch", new[] { new Avalonia.Platform.Storage.FilePickerFileType("IPS, UPS and BPS patches") { Patterns = new[] { "*.ips", "*.ups", "*.bps" } } }) is { } file ? file.Path : null;

        public static readonly string[] PatchExtensions = [".ips", ".ups", ".bps"];

        private readonly string _game;
        private readonly string? _library;
        private readonly bool _browse;
        private readonly Button _make;
        private readonly TextBlock _packageText = new() { Name = "GamePackagePlace", TextWrapping = TextWrapping.Wrap };
        private readonly StackPanel _patches = Ui.Stack(6);
        private readonly WrapPanel _adds = new() { Name = "GamePackageAdds", Orientation = Orientation.Horizontal, ItemSpacing = 8, LineSpacing = 8 };
        private readonly HintText _patchHint = new() { Name = "GamePackagePatchHint" };
        private readonly HintText _status = new() { Name = "GamePackageStatus" };
        private readonly FieldRow _patchRow, _addRow;
        private readonly DispatcherTimer _timer;
        private IToolJob? _job;
        private string _op = "";

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GamePackageWindow() : this("", "", null, null, browse: true) { }

        // refused: the package claims the game but will not open with the file as it is now, most often another dump, so the game plays as a plain one - see EmuSen_GamePackages.md §14.8 Q34.
        public GamePackageWindow(string title, string game, string? library, string? package, bool browse, bool refused = false)
        {
            Refused = refused;
            Title = "Game Package";
            Width = 640;
            Height = 560;
            MinWidth = 480;
            MinHeight = 380;
            CanResize = true;
            (_game, _library, _browse, Package) = (game, library, browse, package);

            _make = Ui.Button("Make Package", Make);
            _make.Name = "GamePackageMake";
            _patches.Name = "GamePackagePatches";
            Control about = Ui.Hint($"{title}: a package keeps everything EmuSen holds for the game in one folder under {DataStore.Packages}, and links the game where it is. The library is only read: nothing in it is changed, moved or removed.");
            about.Name = "GamePackageAbout";
            _patchRow = new FieldRow { Label = "Soft patches", Hint = "Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original.", Content = Ui.Stack(6, _patches, _patchHint) };
            _addRow = new FieldRow { Label = "Add a patch", Hint = "An IPS, UPS or BPS file, checked against the game before it is copied into the package.", Content = _adds };
            var body = Ui.Stack(16,
                about,
                new FieldRow { Label = "Package", Content = Ui.Stack(8, _packageText, Ui.Buttons(_make)) },
                _patchRow,
                _addRow);
            body.Margin = new Thickness(0, 0, 4, 0);

            Button close = Ui.Button("Close", Close);
            close.Name = "GamePackageClose";
            Control foot = Ui.Stack(8, _status, Ui.Buttons(close));
            foot.Margin = new Thickness(0, 10, 0, 0);
            DockPanel.SetDock(foot, Dock.Bottom);
            var dock = new DockPanel { Margin = new Thickness(16), LastChildFill = true };
            dock.Children.Add(foot);
            dock.Children.Add(new ScrollViewer { Content = body, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled });
            Content = dock;

            _timer = new DispatcherTimer { Interval = CompressedDiscsWindow.PollEvery };
            _timer.Tick += (_, _) => Poll();
            Closed += (_, _) =>
            {
                _timer.Stop();
                _job?.Dispose();
                _job = null;
            };
            if (!ToolAvailable()) _status.Text = $"Packages are made by EmuSen's platform library, which is not in use here ({LibraryToolJob.Report}).";
            Render();
        }

        // The package's folder, null until there is one.
        public string? Package { get; private set; }

        public bool Refused { get; }

        // Whether a package was made or its patches changed since the window opened, so the library is scanned again.
        public bool Changed { get; private set; }

        public bool Busy => _job is not null;

        public string? Status => _status.Text;

        // The game's path as the library names it, with / between its parts; null for a game outside it.
        private string? LibraryPath()
        {
            if (string.IsNullOrWhiteSpace(_library)) return null;
            string rel = Path.GetRelativePath(Path.GetFullPath(_library), Path.GetFullPath(_game));
            return rel.StartsWith("..", StringComparison.Ordinal) || Path.IsPathRooted(rel) ? null : rel.Replace('\\', '/');
        }

        private string PackagePath() => Path.GetRelativePath(DataStore.Packages, Package!).Replace('\\', '/');

        // A disc game takes no soft patch here (§9 Q15).
        private bool IsDisc => ImageSets.IsSheet(_game) || ImageSets.IsPlaylist(_game) || _game.EndsWith(".chd", StringComparison.OrdinalIgnoreCase);

        // The patch files beside the game in the library, as RetroArch and ES-DE keep a hack beside its game.
        public IReadOnlyList<string> Beside()
        {
            string? folder = Path.GetDirectoryName(_game);
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

        private async Task BrowseAsync()
        {
            if (await PickPatch(this) is { } file) Add(file);
        }

        private void Run(string op, string request)
        {
            if (_job is not null) return;
            try { _job = StartJob(request); }
            catch (Exception e) when (e is FormatException or InvalidOperationException)
            {
                _status.Text = CompressedDiscsWords.Refusal(e.Message);
                Render();
                return;
            }
            _op = op;
            _status.Text = op switch { "make" => "Making the package. The game stays where it is.", "add" => "Checking the patch against the game.", _ => "Taking the patch out." };
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
                    _status.Text = CompressedDiscsWords.Refusal(e.Message);
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
            }
            Render();
        }

        private static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        // One event of the pkg job applied: the package made, the patch added or removed, or why not.
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
                    if (state is "made" or "added" or "removed") Changed = true;
                    _status.Text = state switch
                    {
                        "made" => "The package was made, linked to the game where it is.",
                        "already" => "The game already has a package.",
                        "added" => $"Added {Text(e, "patch")}. " + (e.TryGetProperty("retroarch", out JsonElement ra) && ra.ValueKind == JsonValueKind.True ? "It is named as RetroArch finds it." : "RetroArch will not apply this list, whose files keep their own names."),
                        "removed" => $"Took out {Text(e, "patch")}.",
                        _ => words is null ? $"Not done: {state}." : $"Not done: {words}",
                    };
                    break;
                case "end" when Text(e, "error") is { } error:
                    _status.Text = CompressedDiscsWords.Refusal(error);
                    break;
            }
        }

        // The manifest's patch list, read for showing: each file and its format.
        public IReadOnlyList<(string File, string Format)> Patches()
        {
            if (Package is null) return [];
            try
            {
                return GamePackage.Read(Path.Combine(Package, GamePackage.FileName)).Document["patches"] is JsonArray list
                    ? list.OfType<JsonObject>().Select(p => ((string?)p["file"] ?? "", ((string?)p["format"] ?? "").ToUpperInvariant())).ToList()
                    : [];
            }
            catch (Exception e) when (e is IOException or FormatException or UnauthorizedAccessException) { return []; }
        }

        // The rows, the buttons and the words as the package and the job allow.
        private void Render()
        {
            bool running = _job is not null, tool = ToolAvailable();
            _packageText.Text = Package is null ? "Not a package yet."
                : Refused ? $"Packages/{PackagePath()}, changed: the game's file is not as the package recorded it (pkg check names why, most often another dump), so the game plays as a plain one with its plain saves, and the package refuses to open until it is checked."
                : $"Packages/{PackagePath()}, linked to the game where it is in the library.";
            _make.IsVisible = Package is null;
            _make.IsEnabled = tool && !running && LibraryPath() is not null;

            _patches.Children.Clear();
            IReadOnlyList<(string File, string Format)> patches = Patches();
            for (int i = 0; i < patches.Count; i++)
            {
                int n = i + 1;
                var name = new TextBlock { Text = $"{n}. {patches[i].File} ({patches[i].Format})", TextWrapping = TextWrapping.Wrap, VerticalAlignment = VerticalAlignment.Center };
                Button remove = Ui.Button("Remove", () => Remove(n));
                remove.Name = $"GamePackagePatch.{i}.Remove";
                remove.IsEnabled = tool && !running;
                var row = new DockPanel { LastChildFill = true };
                DockPanel.SetDock(remove, Dock.Right);
                row.Children.Add(remove);
                row.Children.Add(name);
                _patches.Children.Add(row);
            }
            bool candidates = _browse || Beside().Count > 0;
            string none = patches.Count == 0 ? "No patches: the game is played as dumped." : "";
            string where = candidates ? "" : " On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none.";
            _patchHint.Text = Package is null ? "Make the package first; its patches live in it." : IsDisc ? "Disc games take no soft patches here." : (none + where).Trim();
            _patchHint.IsVisible = _patchHint.Text.Length > 0;
            _patchRow.IsVisible = true;

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
        }
    }
}
