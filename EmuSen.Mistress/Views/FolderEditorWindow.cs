using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // ES-DE's metadata editor for a folder, as far as Mistress keeps a folder's metadata: its folder link - see EmuSen_Settings_Reference.md §4.67.
    public sealed class FolderEditorWindow : ToolWindow
    {
        public const string NoLink = "(none)";

        private readonly Action<string?> _save;
        private readonly string? _stored;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public FolderEditorWindow() : this("", "", [], null, _ => { }) { }

        // files are the games below the folder, by their paths from it; stored is the link kept now, save gets the new one or null for none.
        public FolderEditorWindow(string name, string where, IReadOnlyList<string> files, string? stored, Action<string?> save)
        {
            _save = save;
            _stored = stored is not null && files.Contains(stored) ? stored : null;
            Title = "Edit Folder's Metadata";
            Width = 720;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;
            Link.Fill(files.Prepend(NoLink).ToArray(), _stored ?? NoLink);

            Button saveButton = Ui.Button("Save", Save);
            saveButton.Name = "FolderSave";
            saveButton.IsDefault = true;
            Button cancel = Ui.Button("Cancel", Close);
            cancel.Name = "FolderCancel";
            Content = Ui.Stack(12,
                Ui.Stack(2,
                    new TextBlock { Name = "FolderTitle", Text = name, FontSize = 20, FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap },
                    new HintText { Text = where }),
                new FieldRow
                {
                    Label = "Folder link",
                    Hint = "A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it.",
                    Content = Link,
                },
                Ui.Buttons(saveButton, cancel)).Margin(16);
            Opened += (_, _) => Link.Focus(NavigationMethod.Directional);
        }

        public Dropdown Link { get; } = new() { Name = "FolderLink", HorizontalAlignment = HorizontalAlignment.Stretch };

        // The link as chosen now, null for none.
        public string? Chosen => Link.SelectedItem is string s && s != NoLink ? s : null;

        private void Save()
        {
            if (Chosen != _stored) _save(Chosen);
            Close();
        }
    }
}
