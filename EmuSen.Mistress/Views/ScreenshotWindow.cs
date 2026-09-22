using System;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // One screenshot at its own size, scaled down to fit and never up - see EmuSen_Settings_Reference.md §4.35.
    public sealed class ScreenshotWindow : ToolWindow
    {
        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public ScreenshotWindow() : this("", "") { }

        public ScreenshotWindow(string path, string title)
        {
            Title = title;
            Width = 720;
            Height = 560;
            ClosesOnEscape = true;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            try
            {
                Content = new Image { Source = new Bitmap(path), Stretch = Stretch.Uniform, StretchDirection = StretchDirection.DownOnly, Margin = new Avalonia.Thickness(12) };
            }
            catch (Exception ex)
            {
                Content = new EmptyState { Message = "This screenshot could not be opened.", Detail = ex.Message };
            }
        }
    }
}
