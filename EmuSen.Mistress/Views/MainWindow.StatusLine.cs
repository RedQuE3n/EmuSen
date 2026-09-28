using System;
using Avalonia;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Views
{
    // The status line is never cut: on the desktop trimmed with its whole text in a tooltip and the counters kept short, in a big screen wrapped and without the counters - see EmuSen_Settings_Reference.md §4.83.6.
    public partial class MainWindow
    {
        // The status line's words in a big-screen session, in design pixels before the menus' scale: the look's small text.
        public const double BigStatusText = 20;

        // The counters' share of the line at most, so a status beside them keeps the rest.
        public const double CountersShare = 0.5;

        // A line of it in a big-screen session, in design pixels: the desktop bar's whole height, so the bar is the same at 800 pixels.
        public const double BigStatusLine = 26;

        private IDisposable? _bigStatusSize, _bigLineHeight;

        private void SetUpStatusLine()
        {
            ToolTip.SetTip(StatusText, StatusText.Text);
            StatusText.PropertyChanged += (_, e) =>
            {
                if (e.Property == TextBlock.TextProperty) ToolTip.SetTip(StatusText, StatusText.Text);
            };
            StatusBar.SizeChanged += (_, e) => FpsText.MaxWidth = Math.Max(0, e.NewSize.Width * CountersShare);
        }

        // The counters' short line, and the whole measurement in its tooltip.
        private void ShowCounters(string shown, string whole)
        {
            FpsText.Text = shown;
            ToolTip.SetTip(FpsText, whole);
        }

        // A big screen's status line in the look's small text at the menus' scale, wrapped since a pad has no tooltip; the desktop's own size and one trimmed line otherwise.
        private void ApplyBigStatusLine(bool on)
        {
            _bigStatusSize?.Dispose();
            _bigLineHeight?.Dispose();
            _bigStatusSize = _bigLineHeight = null;
            if (on)
            {
                _bigStatusSize = StatusText.Bind(TextBlock.FontSizeProperty, this.GetObservable(MenuPanel.ScaleProperty, u => BigStatusText * u));
                _bigLineHeight = StatusText.Bind(TextBlock.LineHeightProperty, this.GetObservable(MenuPanel.ScaleProperty, u => BigStatusLine * u));
            }
            else
            {
                StatusText.ClearValue(TextBlock.FontSizeProperty);
                StatusText.ClearValue(TextBlock.LineHeightProperty);
            }
            // The bar keeps the height it has at 800 pixels, one line of 26, and grows with the menus' scale (§4.83.6).
            StatusBar.Padding = on ? new Thickness(6, 0) : new Thickness(6, 3);
            StatusText.TextWrapping = on ? Avalonia.Media.TextWrapping.Wrap : Avalonia.Media.TextWrapping.NoWrap;
            StatusText.TextTrimming = on ? Avalonia.Media.TextTrimming.None : Avalonia.Media.TextTrimming.CharacterEllipsis;
            ApplyStatusBar();
        }
    }
}
