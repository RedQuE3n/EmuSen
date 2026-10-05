using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.Cores;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;

namespace EmuSen.Mistress.Views
{
    // Preferences' Firmware tab: per system, which version of each firmware file runs and what EmuSen's open one changes, with the folder a player's own file goes in - see EmuSen_Settings_Reference.md §4.89.
    public sealed class FirmwarePane
    {
        public const string FolderLabel = "Your Own Files", FolderRowLabel = "Folder for Your Own Files", OpenFolder = "Open Folder";

        private readonly bool _menu;
        private string _shown = "";

        // As a big-screen menu the page is read only: each row is one a pad lands on, and nothing opens the file manager.
        public FirmwarePane(bool menu) => _menu = menu;

        // The page's rows from the cores' declarations and the folder as they are now.
        public Control[] Rows() => Rows(FirmwareOverview.Build());

        // The page as a big-screen menu takes no choice, so its hints name only moving and going back.
        public static void MarkReadOnly(Control page) => page.SetValue(BigMenuForm.ReadOnlyPageProperty, true);

        private Control[] Rows(IReadOnlyList<FirmwareSystem> systems)
        {
            _shown = Signature(systems);
            var rows = new List<Control>(Folder());
            foreach (FirmwareSystem system in systems)
            {
                if (!_menu) rows.Add(new SectionHeader { Text = system.Name });
                if (system.Items.Count == 0) rows.Add(_menu ? MenuRow(system.Name, "None needed", FirmwareOverview.NoFirmware) : new HintText { Text = FirmwareOverview.NoFirmware });
                foreach (FirmwareItem item in system.Items)
                    rows.Add(_menu ? MenuRow(MenuLabel(item), item.FileName, $"{item.InUse}. {item.Change}{(item.Note is { } note ? " " + note : "")}", verbatim: true) : DesktopRow(item));
            }
            return rows.ToArray();
        }

        // Fills the pane's stack again when the folder or an engine choice has changed since it was last filled; false when nothing had.
        public bool Refresh(Panel host)
        {
            IReadOnlyList<FirmwareSystem> systems = FirmwareOverview.Build();
            if (Signature(systems) == _shown) return false;
            host.Children.Clear();
            host.Children.AddRange(Rows(systems));
            return true;
        }

        private static string Signature(IReadOnlyList<FirmwareSystem> systems) =>
            FirmwareOverview.Folder + "\n" + string.Join("\n", systems.Select(s => $"{s.Name}|{s.Engine}|" + string.Join(";", s.Items.Select(i => $"{i.FileName},{i.Present},{i.WrongSize}"))));

        // The words about a player's own files, and the folder: on the desktop one field with the button, as a menu two rows whose footers say them.
        private Control[] Folder()
        {
            string folder = FirmwareOverview.Folder;
            if (_menu)
            {
                Control where = MenuRow(FolderRowLabel, Path.GetFileName(folder.TrimEnd(Path.DirectorySeparatorChar)), folder, verbatim: true);
                where.SetValue(BigMenuForm.VerbatimHintProperty, true);
                return [MenuRow(FolderLabel, "Optional", FirmwareOverview.Optional), where];
            }
            var path = new MonoText { Name = "FirmwareFolderText", Text = folder, TextWrapping = TextWrapping.Wrap };
            Button open = Ui.Button(OpenFolder, () => Open(folder));
            open.Name = "FirmwareOpenFolderButton";
            open.HorizontalAlignment = HorizontalAlignment.Left;
            return [new FieldRow { Name = "FirmwareFolderRow", Label = FolderLabel, Hint = FirmwareOverview.Optional, Content = Ui.Stack(8, path, open) }];
        }

        // The folder made if it is not there yet, since a file manager opens nothing for a path that does not exist.
        private static void Open(string folder)
        {
            try { Directory.CreateDirectory(folder); } catch (IOException) { } catch (System.UnauthorizedAccessException) { }
            Input.DeviceKeyboard.Launcher.Open(folder);
        }

        // A file on the desktop: its name, then which version runs, what the open one changes and any note, with the core's whole cost as the tooltip.
        private static Control DesktopRow(FirmwareItem item)
        {
            var words = Ui.Stack(2, new TextBlock { Text = item.InUse, TextWrapping = TextWrapping.Wrap }, new HintText { Text = item.Change, TextWrapping = TextWrapping.Wrap });
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
