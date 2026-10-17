using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.Cores;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;

namespace EmuSen.Mistress.Views
{
    // Preferences' Firmware tab: the player's own firmware folder and EmuSen's, then per system which version of each file runs and where it was found - see EmuSen_Settings_Reference.md §4.89 and §4.121.
    public sealed class FirmwarePane
    {
        public const string FolderLabel = "Your Own Files", FolderRowLabel = "EmuSen's Firmware Folder", ChosenLabel = "Your Firmware Folder", OpenFolder = "Open Folder";
        public const string ChosenHint = "Searched before EmuSen's folder, so a file in both comes from yours. EmuSen only reads it, never writes to it. Leave it blank for EmuSen's folder alone.";
        public const string ChosenPlaceholder = "(none)", ChosenTitle = "Choose Your Firmware Folder";

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
        public Control[] Rows() => Rows(FirmwareOverview.Build());

        // The page as a big-screen menu takes no choice except on the folder's row, so its hints name only moving and going back elsewhere.
        public static void MarkReadOnly(Control page) => page.SetValue(BigMenuForm.ReadOnlyPageProperty, true);

        // What is said about the folder setting: a refusal of what was typed, else the stored value's problem, else nothing.
        public string? Message => _refused ?? SettingFolders.Checked(_settings.FirmwareDirectory).Problem;

        private Control[] Rows(IReadOnlyList<FirmwareSystem> systems)
        {
            _shown = Signature(systems);
            var rows = new List<Control>(_menu ? MenuFolder() : DesktopFolder());
            rows.AddRange(SystemRows(systems));
            return rows.ToArray();
        }

        private IEnumerable<Control> SystemRows(IReadOnlyList<FirmwareSystem> systems)
        {
            foreach (FirmwareSystem system in systems)
            {
                if (!_menu) yield return new SectionHeader { Text = system.Name };
                if (system.Items.Count == 0) yield return _menu ? MenuRow(system.Name, "None needed", FirmwareOverview.NoFirmware) : new HintText { Text = FirmwareOverview.NoFirmware };
                foreach (FirmwareItem item in system.Items)
                    yield return _menu ? MenuRow(MenuLabel(item), item.FileName, Footer(item), verbatim: true) : DesktopRow(item);
            }
        }

        // A file's footer on the big screen: which version runs, what the open one changes, where it was found and any note.
        public static string Footer(FirmwareItem item) => $"{item.InUse}. {item.Change}" + string.Concat(new[] { item.Where, item.Note }.Where(w => w is not null).Select(w => " " + w));

        // Fills the pane's stack again when a folder, the setting or an engine choice has changed since it was last filled; false when nothing had.
        public bool Refresh(Panel host)
        {
            IReadOnlyList<FirmwareSystem> systems = FirmwareOverview.Build();
            if (Signature(systems) == _shown) return false;
            if (_menu)
            {
                host.Children.Clear();
                host.Children.AddRange(Rows(systems));
                return true;
            }
            // On the desktop the folder rows stay, so the focus in them does; the systems' rows after them are made again.
            _shown = Signature(systems);
            Control[] folder = DesktopFolder();
            host.Children.RemoveRange(folder.Length, host.Children.Count - folder.Length);
            host.Children.AddRange(SystemRows(systems));
            return true;
        }

        private string Signature(IReadOnlyList<FirmwareSystem> systems) =>
            string.Join("|", FirmwareOverview.Folders) + "\n" + FirmwareOverview.Folder + "\n" + _settings.FirmwareDirectory + "\n" + Message + "\n" +
            string.Join("\n", systems.Select(s => $"{s.Name}|{s.Engine}|" + string.Join(";", s.Items.Select(i => $"{i.FileName},{i.Present},{i.WrongSize},{i.FoundIn},{i.WrongSizeIn}"))));

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
                Button open = Ui.Button(OpenFolder, () => Open(FirmwareOverview.Folder));
                open.Name = "FirmwareOpenFolderButton";
                open.HorizontalAlignment = HorizontalAlignment.Left;
                _desktopFolder =
                [
                    new FieldRow { Name = "FirmwareDirectoryRow", Label = ChosenLabel, Hint = ChosenHint, Content = Ui.Stack(6, _picker, _problem) },
                    new FieldRow { Name = "FirmwareFolderRow", Label = FolderRowLabel, Hint = FirmwareOverview.Optional, Content = Ui.Stack(8, _ownPath, open) },
                ];
            }
            if (_picker!.Path != (_settings.FirmwareDirectory ?? "")) _picker.Path = _settings.FirmwareDirectory ?? "";
            _problem!.Text = Message;
            _problem.IsVisible = Message is not null;
            _ownPath!.Text = FirmwareOverview.Folder;
            return _desktopFolder;
        }

        // As a menu: the words about a player's own files, the player's folder as a typed row whose footer is its message, and EmuSen's folder, whose footer is its path.
        private Control[] MenuFolder()
        {
            string folder = FirmwareOverview.Folder;
            _picker = Picker();
            var chosen = new FieldRow { Label = ChosenLabel, Hint = Message ?? ChosenHint, Content = _picker };
            chosen.SetValue(BigMenuForm.TakesChoiceProperty, true);
            if (Message is not null) chosen.SetValue(BigMenuForm.VerbatimHintProperty, true);
            Control where = MenuRow(FolderRowLabel, Path.GetFileName(folder.TrimEnd(Path.DirectorySeparatorChar)), folder, verbatim: true);
            where.SetValue(BigMenuForm.VerbatimHintProperty, true);
            return [MenuRow(FolderLabel, "Optional", FirmwareOverview.Optional), chosen, where];
        }

        // EmuSen's folder made if it is not there yet, since a file manager opens nothing for a path that does not exist; the player's folder is never made.
        private static void Open(string folder)
        {
            try { Directory.CreateDirectory(folder); } catch (IOException) { } catch (System.UnauthorizedAccessException) { }
            Input.DeviceKeyboard.Launcher.Open(folder);
        }

        // A file on the desktop: its name, then which version runs, what the open one changes, where it was found and any note, with the core's whole cost as the tooltip.
        private static Control DesktopRow(FirmwareItem item)
        {
            var words = Ui.Stack(2, new TextBlock { Text = item.InUse, TextWrapping = TextWrapping.Wrap }, new HintText { Text = item.Change, TextWrapping = TextWrapping.Wrap });
            if (item.Where is { } where) words.Children.Add(new HintText { Text = where, TextWrapping = TextWrapping.Wrap });
            if (item.Note is { } note) words.Children.Add(new HintText { Text = note, TextWrapping = TextWrapping.Wrap });
            var row = new FieldRow { Label = item.Title, Content = words };
            if (!string.IsNullOrEmpty(item.Cost)) ToolTip.SetTip(row, new TextBlock { Text = item.Cost, TextWrapping = TextWrapping.Wrap, MaxWidth = 520 });
            return row;
        }

        // A file's menu row: what it is and which version runs, cased by the look, beside its file name, which keeps its own case.
        public static string MenuLabel(FirmwareItem item) => $"{item.Label}: {item.InUseShort}";

        // A fact as a menu row a pad lands on: the big-screen form makes the button its row, the words its value and the hint its footer; a verbatim value is a name read letter for letter.
        private static Control MenuRow(string label, string value, string footer, bool verbatim = false)
        {
            var button = new Button { Content = value };
            if (verbatim) MenuRows.SetValueLetterCase(button, EmuSen.LunaP.Media.LetterCase.None);
            return new FieldRow { Label = label, Hint = footer, Content = button };
        }
    }
}
