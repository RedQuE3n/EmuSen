using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;

namespace EmuSen.LunaP.Windowing
{
    // The small modal behind Dialogs.ConfirmAsync/ErrorAsync, built from the kit rather than a hand-laid grid - see EmuSen_LunaP.md §9.4.
    internal sealed class MessageWindow : ToolWindow
    {
        private MessageWindow(string title, string message, string acceptText, string? cancelText)
        {
            Title = title;
            SizeToContent = SizeToContent.Height;
            Width = 420;
            CanResize = false;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            var accept = new Button { Content = acceptText, IsDefault = true };
            accept.Click += (_, _) => Close(true);

            var buttons = new ButtonBar();
            if (cancelText is not null)
            {
                var cancel = new Button { Content = cancelText, IsCancel = true };
                cancel.Click += (_, _) => Close(false);
                buttons.ItemsSource = new[] { accept, cancel };
            }
            else
            {
                buttons.ItemsSource = new[] { accept };
            }

            Content = new StackPanel
            {
                Margin = new Thickness(16),
                Spacing = 16,
                Children =
                {
                    new TextBlock { Text = message, TextWrapping = Avalonia.Media.TextWrapping.Wrap },
                    new ContentControl { Content = buttons, HorizontalAlignment = HorizontalAlignment.Right },
                },
            };
        }

        // Escape and the window's own close button both mean "no".
        public static MessageWindow Confirm(string title, string message, string acceptText, string cancelText) =>
            new(title, message, acceptText, cancelText);

        public static MessageWindow Notice(string title, string message, string acceptText) =>
            new(title, message, acceptText, null);
    }
}
