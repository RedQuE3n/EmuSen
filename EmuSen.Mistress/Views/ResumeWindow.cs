using System;
using System.IO;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    public enum ResumeChoice { Resume, Restart }

    // The question a game with a resume state asks at its start; closing it cancels the launch - see EmuSen_Settings_Reference.md §4.31.
    public sealed class ResumeWindow : ToolWindow
    {
        private readonly LunaSwitch _remember = new() { Name = "RememberResumeSwitch", Label = "Do not ask again" };
        private readonly Button _resume;

        public bool Remember => _remember.IsChecked == true;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public ResumeWindow() : this("", null, DateTime.Now) { }

        public ResumeWindow(string title, string? picturePath, DateTime savedAt)
        {
            Title = "Continue Playing";
            Width = 460;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            _resume = new Button { Name = "ResumeButton", Content = "Resume", IsDefault = true };
            _resume.Click += (_, _) => Close(ResumeChoice.Resume);
            var restart = new Button { Name = "RestartButton", Content = "Restart" };
            restart.Click += (_, _) => Close(ResumeChoice.Restart);

            var body = Ui.Stack(12,
                new TextBlock { Text = "Would you like to continue your last game?", FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap },
                Ui.Hint($"{title} was left on {savedAt:dddd d MMMM, HH:mm}. Resume where you left off, or start it again."));

            if (Picture(picturePath) is Image picture) body.Children.Insert(0, picture);

            body.Children.Add(_remember);
            body.Children.Add(new ButtonBar { ItemsSource = new[] { _resume, restart }, HorizontalAlignment = HorizontalAlignment.Right });
            Content = body.Margin(16);

            Opened += (_, _) => _resume.Focus();
        }

        private static Image? Picture(string? path)
        {
            if (path is null || !File.Exists(path)) return null;
            try
            {
                return new Image { Source = new Bitmap(path), MaxHeight = 240, Stretch = Stretch.Uniform, HorizontalAlignment = HorizontalAlignment.Center };
            }
            catch (Exception)
            {
                return null;
            }
        }
    }
}
