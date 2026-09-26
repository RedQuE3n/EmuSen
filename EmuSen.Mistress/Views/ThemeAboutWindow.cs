using System.Linq;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;

namespace EmuSen.Mistress.Views
{
    // An installed theme's attribution: its author, licence, credits and source, as its own files state them - see EmuSen_Settings_Reference.md §4.53.
    public class ThemeAboutWindow : ToolWindow
    {
        public ThemeAboutWindow(ThemeAttribution about)
        {
            Attribution = about;
            Title = $"About {about.Name}";
            Width = 720;
            CanResize = false;
            SizeToContent = SizeToContent.Height;

            var rows = Ui.Stack(12,
                new FieldRow { Label = "Theme", Content = Words("AboutName", about.Name) },
                new FieldRow { Label = "Author", Content = Words("AboutAuthor", about.Author) },
                new FieldRow { Label = "Licence", Hint = "From the theme's own README or LICENSE file.", Content = Lines("AboutLicence", about.Licence) },
                new FieldRow { Label = "Credits", Hint = "From the theme's own README.", Content = about.Credits.Count == 0 ? Words("AboutCredits", "The README names no credits.") : Lines("AboutCredits", about.Credits) },
                new FieldRow { Label = "Source", Content = Words("AboutSource", about.Commit is { } c ? $"{about.Source}, commit {c}" : about.Source) },
                Words("AboutStatement", about.Statement));
            // A scroller the pad can focus, so a long README section is read down by the pad as by a wheel.
            var scroll = new ScrollViewer { Name = "AboutScroll", Content = rows.Margin(4, 12, 4, 4), MaxHeight = 600, Focusable = true };
            Control buttons = Ui.Buttons(Ui.Button("Close", Close)).Margin(0, 12, 0, 0);
            DockPanel.SetDock(buttons, Dock.Bottom);
            Content = new DockPanel { LastChildFill = true, Children = { buttons, scroll } }.Margin(16);
        }

        public ThemeAttribution Attribution { get; }

        private static TextBlock Words(string name, string text) => new() { Name = name, Text = text, TextWrapping = TextWrapping.Wrap };

        private static Control Lines(string name, System.Collections.Generic.IReadOnlyList<string> lines) =>
            Ui.Stack(4, lines.Select((l, i) => (Control)Words($"{name}{i}", l)).ToArray());
    }
}
