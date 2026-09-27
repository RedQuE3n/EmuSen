using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Views
{
    // What a window changes when a big-screen sheet shows it in ES-DE's look, beyond what the look's styles reach - see EmuSen_Settings_Reference.md §4.80.
    internal static class SheetLook
    {
        // A status line beside a row of buttons becomes a line above them, the buttons centred as a menu's button band is.
        public static void StatusOverButtons(TextBlock status)
        {
            if (status.Parent is not Grid grid || grid.Children.OfType<ButtonBar>().FirstOrDefault() is not { } buttons) return;
            grid.ColumnDefinitions.Clear();
            grid.RowDefinitions = new RowDefinitions("Auto,Auto");
            grid.RowSpacing = 8;
            Grid.SetColumn(status, 0);
            Grid.SetRow(status, 0);
            Grid.SetColumn(buttons, 0);
            Grid.SetRow(buttons, 1);
            status.HorizontalAlignment = HorizontalAlignment.Center;
            status.TextAlignment = Avalonia.Media.TextAlignment.Center;
            buttons.HorizontalAlignment = HorizontalAlignment.Center;
        }

        // The design size of a table cell's small words in the look, where the desktop draws them at 11.
        public const double SmallText = 20;
    }
}
