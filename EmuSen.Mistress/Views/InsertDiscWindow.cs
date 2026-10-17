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
    // The disc to put in a running console: its games in the library, and on the desktop any file - see Diamond_Shell.md §1.5.
    public sealed class InsertDiscWindow : ToolWindow
    {
        // What Close answers when the player asks to browse for a file instead.
        public const string Browse = "\0browse";

        private readonly List<Button> _discs = new();

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public InsertDiscWindow() : this("", [], browse: true) { }

        public InsertDiscWindow(string console, IReadOnlyList<(string Title, string Path)> discs, bool browse)
        {
            Title = "Insert Disc";
            Width = 520;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            var list = Ui.Stack(4);
            list.Name = "InsertDiscList";
            for (int i = 0; i < discs.Count; i++)
            {
                string path = discs[i].Path;
                var row = new Button
                {
                    Name = $"InsertDisc{i}",
                    Content = new TextBlock { Text = discs[i].Title, TextWrapping = TextWrapping.Wrap },
                    HorizontalAlignment = HorizontalAlignment.Stretch,
                    HorizontalContentAlignment = HorizontalAlignment.Left,
                };
                row.Click += (_, _) => Close(path);
                _discs.Add(row);
                list.Children.Add(row);
            }

            string words = discs.Count == 0
                ? $"No {console} games are in the library.{(browse ? " Browse for a disc image instead." : "")}"
                : $"The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later.";
            var body = Ui.Stack(12, Ui.Hint(words));
            body.Children[0].Name = "InsertDiscAbout";
            if (discs.Count > 0) body.Children.Add(new ScrollViewer { Name = "InsertDiscScroll", Content = list, MaxHeight = 360 });

            var buttons = new List<Button>();
            if (browse)
            {
                var browseButton = new Button { Name = "InsertDiscBrowse", Content = "Browse..." };
                browseButton.Click += (_, _) => Close(Browse);
                buttons.Add(browseButton);
            }
            var cancel = new Button { Name = "InsertDiscCancel", Content = "Cancel", IsCancel = true };
            cancel.Click += (_, _) => Close(null);
            buttons.Add(cancel);
            body.Children.Add(new ButtonBar { ItemsSource = buttons, HorizontalAlignment = HorizontalAlignment.Right });
            Content = body.Margin(16);

            Opened += (_, _) => (_discs.FirstOrDefault() ?? buttons[0]).Focus(NavigationMethod.Directional);
        }

        public IReadOnlyList<Button> Discs => _discs;
    }
}
