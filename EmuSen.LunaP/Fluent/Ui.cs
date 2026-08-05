using System;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;

namespace EmuSen.LunaP.Fluent
{
    // Terse constructors for the layouts and kit controls a window is made of - see EmuSen_LunaP.md §9.
    public static class Ui
    {
        public static StackPanel Stack(params Control[] children) => Stack(0, children);

        public static StackPanel Stack(double spacing, params Control[] children)
        {
            var panel = new StackPanel { Spacing = spacing };
            foreach (Control child in children) panel.Children.Add(child);
            return panel;
        }

        public static StackPanel Row(params Control[] children) => Row(0, children);

        public static StackPanel Row(double spacing, params Control[] children)
        {
            var panel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = spacing };
            foreach (Control child in children) panel.Children.Add(child);
            return panel;
        }

        // Last child fills, as DockPanel already does; the rest carry .Dock(...).
        public static DockPanel Dock(params Control[] children)
        {
            var panel = new DockPanel();
            foreach (Control child in children) panel.Children.Add(child);
            return panel;
        }

        // Columns are assigned by position, so the Grid.SetColumn chore disappears - an explicit .AtColumn() still wins.
        public static Grid Cols(string definitions, params Control[] children)
        {
            var grid = new Grid { ColumnDefinitions = new ColumnDefinitions(definitions) };
            for (int i = 0; i < children.Length; i++)
            {
                Control child = children[i];
                if (!child.IsSet(Grid.ColumnProperty)) Grid.SetColumn(child, i);
                grid.Children.Add(child);
            }

            return grid;
        }

        public static ScrollViewer Scroll(Control content) => new() { Content = content };

        // A SectionHeader and its content. Spacing defaults to the 8 the dashboards already used between the two.
        public static StackPanel Section(string header, Control content, double spacing = 8) =>
            Stack(spacing, new SectionHeader { Text = header }, content);

        public static SectionHeader Header(string text) => new() { Text = text };

        public static HintText Hint(string text) => new() { Text = text };

        public static MonoText Mono(string text = "") => new() { Text = text };

        public static TextBlock Text(string text = "") => new() { Text = text };

        public static Button Button(string content, Action onClick)
        {
            var button = new Button { Content = content };
            button.Click += (_, _) => onClick();
            return button;
        }

        // A right-aligned run of buttons, for the bottom of a window.
        public static ButtonBar Buttons(params Button[] buttons) => new() { ItemsSource = buttons };
    }
}
