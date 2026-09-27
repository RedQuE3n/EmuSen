using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.LogicalTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // ES-DE's list screen for an option row: the row's name as the title, a row per choice with the current one chosen, and Back - see EmuSen_Settings_Reference.md §4.72.9.
    public sealed class OptionListWindow : ToolWindow, IPadDriven
    {
        private readonly ComboBox _combo;
        private readonly StackPanel _rows = new() { Name = "OptionListRows" };

        public OptionListWindow(ComboBox combo, string title, PadFamily family)
        {
            _combo = combo;
            Title = title;
            ClosesOnEscape = true;
            SheetLayer.SetChromeless(this, true);
            Choices = combo.Items.Cast<object?>().Select(i => i?.ToString() ?? "").ToList();
            Button? current = null;
            for (int i = 0; i < Choices.Count; i++)
            {
                int index = i;
                Button row = MenuRows.Apply(new Button { Name = $"OptionListChoice{i}", Content = Choices[i] }, MenuRowKind.Action);
                MenuRows.SetValueLetterCase(row, null);
                row.Click += (_, _) => Choose(index);
                _rows.Children.Add(row);
                if (i == combo.SelectedIndex) current = row;
            }
            Button back = MenuRows.ApplyButton(new Button { Name = "OptionListBack", Content = "Back" });
            back.Click += (_, _) => Close();
            Content = Menu = new MenuPanel
            {
                Name = "OptionListMenu",
                Title = title,
                HintFamily = family,
                Hints =
                [
                    new("Select") { Button = PadHints.Swapped ? PadGlyphButton.East : PadGlyphButton.South },
                    new("Back") { Button = PadHints.Swapped ? PadGlyphButton.South : PadGlyphButton.East },
                    new("Choose") { Button = PadGlyphButton.DPadUpDown },
                ],
                Child = new ScrollViewer { Content = _rows, VerticalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Hidden },
                Buttons = new StackPanel { Orientation = Orientation.Horizontal, HorizontalAlignment = HorizontalAlignment.Center, Children = { back } },
            };
            // ES-DE opens its list on the choice the row holds; a sheet otherwise focuses its first control.
            Button focus = current ?? _rows.Children.OfType<Button>().FirstOrDefault() ?? back;
            Menu.AttachedToVisualTree += (_, _) => Avalonia.Threading.Dispatcher.UIThread.Post(() => focus.Focus(NavigationMethod.Directional));
            // Back on the row it came from, as ES-DE's list returns to its menu.
            Closed += (_, _) => Avalonia.Threading.Dispatcher.UIThread.Post(() => _combo.Focus(NavigationMethod.Directional));
        }

        public MenuPanel Menu { get; }

        public IReadOnlyList<string> Choices { get; }

        public int? Chosen { get; private set; }

        // The choice given to the row, as a person choosing it from the row's own list would; then the list goes.
        private void Choose(int index)
        {
            Chosen = index;
            _combo.SelectedIndex = index;
            Close();
        }

        public bool OnPad(UiButton button)
        {
            if (button != UiButton.Back) return false;
            Close();
            return true;
        }

        // A row drawn as a menu option row, in a sheet that draws its own menu, opens this list rather than its own drop-down (Q86).
        internal static bool OpensFor(Window window, ComboBox combo) =>
            SheetLayer.GetChromeless(window) && MenuRows.GetKind(combo) == MenuRowKind.Option;

        internal static OptionListWindow Show(Window over, ComboBox combo)
        {
            MenuPanel? menu = over.Content as MenuPanel ?? (over.Content as Avalonia.LogicalTree.ILogical)?.GetSelfAndLogicalDescendants().OfType<MenuPanel>().FirstOrDefault();
            string title = MenuRows.GetLabel(combo) ?? over.Title ?? "";
            var list = new OptionListWindow(combo, title, menu?.HintFamily ?? PadFamily.Generic);
            _ = SheetLayer.Show(list, over);
            return list;
        }
    }
}
