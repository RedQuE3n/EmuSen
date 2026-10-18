using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;

namespace EmuSen.Mistress.Views
{
    // One firmware file a system's engine can use, from the runtime's overview, in a player's words: what it is, which version runs, and what the open one changes - see EmuSen_Settings_Reference.md §4.89.
    public sealed record FirmwareRow(string Label, string FileName, int Size, bool Present, bool WrongSize, string? Effect, string? Cost)
    {
        public const string OpenVersion = "Using EmuSen's open version", OwnFile = "Using your own file", NoVersion = "No open version yet";
        public const string NeedsFile = "Games that need this chip need your own file.", Exact = "Exact: games run the same either way.", AsTheConsole = "Runs as on the console.";
        public const string EmuSensFolder = "EmuSen's folder";

        // Whether the engine carries an open replacement that runs the chip, exact or at a cost.
        public bool HasOpenVersion => Effect is "exact" or "accuracy";

        public string Title => $"{Label} ({FileName})";

        public string InUse => Present ? OwnFile : HasOpenVersion ? OpenVersion : NoVersion;

        // The same in a word or two, for a menu row's value.
        public string InUseShort => Present ? "Your own file" : HasOpenVersion ? "Open version" : "None yet";

        // What the open version changes: the first sentence of the cost the core states, or the fixed words where there is nothing to quote.
        public string Change => Present ? AsTheConsole : Effect == "exact" ? Exact : HasOpenVersion && FirstSentence(Cost) is { } said ? said : NeedsFile;

        // Which folder the file in use was found in, in the runtime's words, or null when none is - see EmuSen_Firmware.md §2.3.
        public string? FoundIn { get; init; }

        // Which folder holds a file of the name and the wrong size, when no folder holds the right one.
        public string? WrongSizeIn { get; init; }

        public string? Where => FoundIn is { } folder ? $"Found in {folder}." : null;

        // Said of a file that is in a folder and not used, since size is the library's one test - see EmuSen_Firmware.md §2.1.
        public string? Note => WrongSize ? $"Not used: the file in {WrongSizeIn ?? EmuSensFolder} is not {Size:N0} bytes." : null;

        public static string? FirstSentence(string? text)
        {
            if (string.IsNullOrWhiteSpace(text)) return null;
            int end = text.IndexOf(". ", System.StringComparison.Ordinal);
            return end < 0 ? text.Trim() : text[..(end + 1)];
        }
    }

    // A system the build runs and the engine that runs it, with its firmware; and the overview's folders.
    public sealed record FirmwareSystemRow(string Name, string Engine, IReadOnlyList<FirmwareRow> Items);

    public sealed record FirmwareView(string Folder, IReadOnlyList<string> Folders, IReadOnlyList<FirmwareSystemRow> Systems)
    {
        // The process's runtime's overview as it stands now.
        public static FirmwareView Current => Read(CoreSettings.Runtime);

        // The runtime's firmware_overview, as the pane shows it.
        public static FirmwareView Read(CrystalRuntime runtime)
        {
            using JsonDocument overview = JsonDocument.Parse(runtime.FirmwareOverview());
            JsonElement o = overview.RootElement;
            string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;
            var systems = o.GetProperty("systems").EnumerateArray().Select(s => new FirmwareSystemRow(s.GetProperty("name").GetString() ?? "", s.GetProperty("engine").GetString() ?? "",
                s.GetProperty("items").EnumerateArray().Select(i => new FirmwareRow(i.GetProperty("label").GetString() ?? "", i.GetProperty("file").GetString() ?? "", i.GetProperty("size").GetInt32(),
                    i.GetProperty("present").GetBoolean(), i.GetProperty("wrong_size").GetBoolean(), Text(i, "effect"), Text(i, "cost"))
                { FoundIn = Text(i, "found_in"), WrongSizeIn = Text(i, "wrong_size_in") }).ToArray())).ToArray();
            return new FirmwareView(o.GetProperty("folder").GetString() ?? "", o.GetProperty("folders").EnumerateArray().Select(f => f.GetString() ?? "").ToArray(), systems);
        }
    }

    // Preferences' Firmware tab: the player's own firmware folder and EmuSen's, then per system which version of each file runs and where it was found - see EmuSen_Settings_Reference.md §4.89 and §4.121.
    public sealed class FirmwarePane
    {
        public const string FolderLabel = "Your Own Files", FolderRowLabel = "EmuSen's Firmware Folder", ChosenLabel = "Your Firmware Folder", OpenFolder = "Open Folder";
        public const string ChosenHint = "Searched before EmuSen's folder, so a file in both comes from yours. EmuSen only reads it, never writes to it. Leave it blank for EmuSen's folder alone.";
        public const string ChosenPlaceholder = "(none)", ChosenTitle = "Choose Your Firmware Folder";
        public const string NoFirmware = "Needs no firmware files.";
        public const string Optional = "Every file here is optional, and EmuSen never downloads firmware. A file of your own in a firmware folder is used in place of EmuSen's open version the next time a game starts.";

        private readonly AppSettings _settings;
        private readonly bool _menu;
        private string _shown = "";
        // The last value typed and refused, said until the next pick.
        private string? _refused;
        private Control[]? _desktopFolder;
        private PathPickerRow? _picker;
        private HintText? _problem;
        private MonoText? _ownPath;

        // As a big-screen menu every row but the folder's is read only, and nothing opens the file manager.
        public FirmwarePane(AppSettings settings, bool menu)
        {
            _settings = settings;
            _menu = menu;
        }

        // Raised after the folder setting is picked, cleared or refused, so the host reads the rows again.
        public System.Action? Chosen { get; set; }

        // The page's rows from the cores' declarations and the folders as they are now.
        public Control[] Rows() => Rows(FirmwareView.Current);

        // The page as a big-screen menu takes no choice except on the folder's row, so its hints name only moving and going back elsewhere.
        public static void MarkReadOnly(Control page) => page.SetValue(BigMenuForm.ReadOnlyPageProperty, true);

        // What is said about the folder setting: a refusal of what was typed, else the stored value's problem, else nothing.
        public string? Message => _refused ?? SettingFolders.Checked(_settings.FirmwareDirectory).Problem;

        private Control[] Rows(FirmwareView view)
        {
            _view = view;
            _shown = Signature(view);
            IReadOnlyList<FirmwareSystemRow> systems = view.Systems;
            var rows = new List<Control>(_menu ? MenuFolder() : DesktopFolder());
            rows.AddRange(SystemRows(systems));
            return rows.ToArray();
        }

        private IEnumerable<Control> SystemRows(IReadOnlyList<FirmwareSystemRow> systems)
        {
            foreach (FirmwareSystemRow system in systems)
            {
                if (!_menu) yield return new SectionHeader { Text = system.Name };
                if (system.Items.Count == 0) yield return _menu ? MenuRow(system.Name, "None needed", NoFirmware) : new HintText { Text = NoFirmware };
                foreach (FirmwareRow item in system.Items)
                    yield return _menu ? MenuRow(MenuLabel(item), item.FileName, Footer(item), verbatim: true) : DesktopRow(item);
            }
        }

        // A file's footer on the big screen: which version runs, what the open one changes, where it was found and any note.
        public static string Footer(FirmwareRow item) => $"{item.InUse}. {item.Change}" + string.Concat(new[] { item.Where, item.Note }.Where(w => w is not null).Select(w => " " + w));

        // Fills the pane's stack again when a folder, the setting or an engine choice has changed since it was last filled; false when nothing had.
        public bool Refresh(Panel host)
        {
            FirmwareView view = FirmwareView.Current;
            if (Signature(view) == _shown) return false;
            if (_menu)
            {
                host.Children.Clear();
                host.Children.AddRange(Rows(view));
                return true;
            }
            // On the desktop the folder rows stay, so the focus in them does; the systems' rows after them are made again.
            _view = view;
            _shown = Signature(view);
            Control[] folder = DesktopFolder();
            host.Children.RemoveRange(folder.Length, host.Children.Count - folder.Length);
            host.Children.AddRange(SystemRows(view.Systems));
            return true;
        }

        // The overview last read, whose folder the rows name.
        private FirmwareView? _view;
        private string OwnFolder => _view?.Folder ?? "";

        private string Signature(FirmwareView view) =>
            string.Join("|", view.Folders) + "\n" + view.Folder + "\n" + _settings.FirmwareDirectory + "\n" + Message + "\n" +
            string.Join("\n", view.Systems.Select(s => $"{s.Name}|{s.Engine}|" + string.Join(";", s.Items.Select(i => $"{i.FileName},{i.Present},{i.WrongSize},{i.FoundIn},{i.WrongSizeIn}"))));

        // A pick is checked by the folder rules before it is kept: a relative path or a file is refused and the row goes back - see EmuSen_Config_Reference.md §7.7.
        private void Pick(string typed)
        {
            FolderValue value = SettingFolders.Checked(typed);
            if (value.State is FolderState.Relative or FolderState.File)
            {
                _refused = value.Problem;
                if (_picker is not null) _picker.Path = _settings.FirmwareDirectory ?? "";
            }
            else
            {
                _refused = null;
                _settings.FirmwareDirectory = value.State == FolderState.Unset ? null : value.Written;
                _settings.Save();
            }
            Chosen?.Invoke();
        }

        private PathPickerRow Picker()
        {
            var picker = new PathPickerRow
            {
                Name = "FirmwareDirectoryBox", Placeholder = ChosenPlaceholder, BrowseTitle = ChosenTitle, Mode = PathPickerMode.Folder,
                Path = _settings.FirmwareDirectory ?? "", IsEditable = true, IsClearable = true,
            };
            picker.PathPicked += Pick;
            return picker;
        }

        // On the desktop: the player's folder as a path row with its message under it, then EmuSen's folder with its button; made once and kept.
        private Control[] DesktopFolder()
        {
            if (_desktopFolder is null)
            {
                _picker = Picker();
                _problem = new HintText { Name = "FirmwareDirectoryProblem", TextWrapping = TextWrapping.Wrap };
                _ownPath = new MonoText { Name = "FirmwareFolderText", TextWrapping = TextWrapping.Wrap };
                Button open = Ui.Button(OpenFolder, () => Open(OwnFolder));
                open.Name = "FirmwareOpenFolderButton";
                open.HorizontalAlignment = HorizontalAlignment.Left;
                _desktopFolder =
                [
                    new FieldRow { Name = "FirmwareDirectoryRow", Label = ChosenLabel, Hint = ChosenHint, Content = Ui.Stack(6, _picker, _problem) },
                    new FieldRow { Name = "FirmwareFolderRow", Label = FolderRowLabel, Hint = Optional, Content = Ui.Stack(8, _ownPath, open) },
                ];
            }
            if (_picker!.Path != (_settings.FirmwareDirectory ?? "")) _picker.Path = _settings.FirmwareDirectory ?? "";
            _problem!.Text = Message;
            _problem.IsVisible = Message is not null;
            _ownPath!.Text = OwnFolder;
            return _desktopFolder;
        }

        // As a menu: the words about a player's own files, the player's folder as a typed row whose footer is its message, and EmuSen's folder, whose footer is its path.
        private Control[] MenuFolder()
        {
            string folder = OwnFolder;
            _picker = Picker();
            var chosen = new FieldRow { Label = ChosenLabel, Hint = Message ?? ChosenHint, Content = _picker };
            chosen.SetValue(BigMenuForm.TakesChoiceProperty, true);
            if (Message is not null) chosen.SetValue(BigMenuForm.VerbatimHintProperty, true);
            Control where = MenuRow(FolderRowLabel, Path.GetFileName(folder.TrimEnd(Path.DirectorySeparatorChar)), folder, verbatim: true);
            where.SetValue(BigMenuForm.VerbatimHintProperty, true);
            return [MenuRow(FolderLabel, "Optional", Optional), chosen, where];
        }

        // EmuSen's folder made if it is not there yet, since a file manager opens nothing for a path that does not exist; the player's folder is never made.
        private static void Open(string folder)
        {
            try { Directory.CreateDirectory(folder); } catch (IOException) { } catch (System.UnauthorizedAccessException) { }
            Input.DeviceKeyboard.Launcher.Open(folder);
        }

        // A file on the desktop: its name, then which version runs, what the open one changes, where it was found and any note, with the core's whole cost as the tooltip.
        private static Control DesktopRow(FirmwareRow item)
        {
            var words = Ui.Stack(2, new TextBlock { Text = item.InUse, TextWrapping = TextWrapping.Wrap }, new HintText { Text = item.Change, TextWrapping = TextWrapping.Wrap });
            if (item.Where is { } where) words.Children.Add(new HintText { Text = where, TextWrapping = TextWrapping.Wrap });
            if (item.Note is { } note) words.Children.Add(new HintText { Text = note, TextWrapping = TextWrapping.Wrap });
            var row = new FieldRow { Label = item.Title, Content = words };
            if (!string.IsNullOrEmpty(item.Cost)) ToolTip.SetTip(row, new TextBlock { Text = item.Cost, TextWrapping = TextWrapping.Wrap, MaxWidth = 520 });
            return row;
        }

        // A file's menu row: what it is and which version runs, cased by the look, beside its file name, which keeps its own case.
        public static string MenuLabel(FirmwareRow item) => $"{item.Label}: {item.InUseShort}";

        // A fact as a menu row a pad lands on: the big-screen form makes the button its row, the words its value and the hint its footer; a verbatim value is a name read letter for letter.
        private static Control MenuRow(string label, string value, string footer, bool verbatim = false)
        {
            var button = new Button { Content = value };
            if (verbatim) MenuRows.SetValueLetterCase(button, EmuSen.LunaP.Media.LetterCase.None);
            return new FieldRow { Label = label, Hint = footer, Content = button };
        }
    }
}
