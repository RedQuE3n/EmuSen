using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;

namespace EmuSen.LunaP.Fluent
{
    // Every name here is the XAML attribute it sets, so the two ways of building a window stay one vocabulary - see EmuSen_LunaP.md §9.
    public static class LayoutExtensions
    {
        public static T Margin<T>(this T control, double uniform) where T : Control
        {
            control.Margin = new Thickness(uniform);
            return control;
        }

        public static T Margin<T>(this T control, double horizontal, double vertical) where T : Control
        {
            control.Margin = new Thickness(horizontal, vertical);
            return control;
        }

        public static T Margin<T>(this T control, double left, double top, double right, double bottom) where T : Control
        {
            control.Margin = new Thickness(left, top, right, bottom);
            return control;
        }

        public static T Spacing<T>(this T panel, double spacing) where T : StackPanel
        {
            panel.Spacing = spacing;
            return panel;
        }

        public static T Width<T>(this T control, double width) where T : Control
        {
            control.Width = width;
            return control;
        }

        public static T Height<T>(this T control, double height) where T : Control
        {
            control.Height = height;
            return control;
        }

        public static T MaxHeight<T>(this T control, double height) where T : Control
        {
            control.MaxHeight = height;
            return control;
        }

        public static T MinSize<T>(this T control, double width, double height) where T : Control
        {
            control.MinWidth = width;
            control.MinHeight = height;
            return control;
        }

        public static T Grow<T>(this T control) where T : Control
        {
            control.HorizontalAlignment = HorizontalAlignment.Stretch;
            return control;
        }

        public static T Left<T>(this T control) where T : Control
        {
            control.HorizontalAlignment = HorizontalAlignment.Left;
            return control;
        }

        public static T Right<T>(this T control) where T : Control
        {
            control.HorizontalAlignment = HorizontalAlignment.Right;
            return control;
        }

        // Vertical centring, which is what almost every use of it in the frontends means.
        public static T Center<T>(this T control) where T : Control
        {
            control.VerticalAlignment = VerticalAlignment.Center;
            return control;
        }

        public static T Dock<T>(this T control, Avalonia.Controls.Dock side) where T : Control
        {
            DockPanel.SetDock(control, side);
            return control;
        }

        public static T AtColumn<T>(this T control, int column, int span = 1) where T : Control
        {
            Grid.SetColumn(control, column);
            Grid.SetColumnSpan(control, span);
            return control;
        }

        public static T AtRow<T>(this T control, int row, int span = 1) where T : Control
        {
            Grid.SetRow(control, row);
            Grid.SetRowSpan(control, span);
            return control;
        }

        // Must be set before the control joins a visual tree, which is what building it fluently already guarantees.
        public static T Name<T>(this T control, string name) where T : Control
        {
            control.Name = name;
            return control;
        }

        public static T Visible<T>(this T control, bool visible) where T : Control
        {
            control.IsVisible = visible;
            return control;
        }

        public static T Bold<T>(this T text) where T : TextBlock
        {
            text.FontWeight = Avalonia.Media.FontWeight.Bold;
            return text;
        }

        public static T FontSize<T>(this T text, double size) where T : TextBlock
        {
            text.FontSize = size;
            return text;
        }

        public static T Wrap<T>(this T text) where T : TextBlock
        {
            text.TextWrapping = Avalonia.Media.TextWrapping.Wrap;
            return text;
        }
    }
}
