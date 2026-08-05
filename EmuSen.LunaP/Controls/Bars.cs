using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace EmuSen.LunaP.Controls
{
    // A right-aligned run of buttons. Buttons go in as ordinary children - see EmuSen_LunaP.md §5.5.
    public class ButtonBar : ItemsControl
    {
    }

    // The bottom strip: a status message on the left, a ButtonBar's worth of actions on the right.
    public class StatusBar : ContentControl
    {
        public static readonly StyledProperty<string> StatusProperty =
            AvaloniaProperty.Register<StatusBar, string>(nameof(Status), string.Empty);

        public string Status
        {
            get => GetValue(StatusProperty);
            set => SetValue(StatusProperty, value);
        }
    }
}
