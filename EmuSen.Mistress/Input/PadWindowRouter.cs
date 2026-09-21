using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;

namespace EmuSen.Mistress.Input
{
    // A pad driving a window that was built for a keyboard: every command becomes the keys that already do it - see EmuSen_Settings_Reference.md §4.29.
    public static class PadWindowRouter
    {
        public static void Send(Window window, UiButton button)
        {
            var focused = window.FocusManager?.GetFocusedElement() as InputElement;
            bool inList = focused is ListBoxItem || focused is ListBox || (focused as Visual)?.FindAncestorOfType<ListBox>() is not null;
            ComboBox? open = window.GetVisualDescendants().OfType<ComboBox>().FirstOrDefault(c => c.IsDropDownOpen);
            var tabs = window.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();

            switch (button)
            {
                case UiButton.Up:
                case UiButton.Down:
                    bool down = button == UiButton.Down;
                    if (open is not null) { Step(open, down ? 1 : -1); return; }
                    if (inList && focused is not ComboBox) { Key(window, focused, down ? Avalonia.Input.Key.Down : Avalonia.Input.Key.Up); return; }
                    Key(window, focused, Avalonia.Input.Key.Tab, down ? KeyModifiers.None : KeyModifiers.Shift);
                    return;

                case UiButton.Left:
                case UiButton.Right:
                    if (focused is ComboBox combo && !combo.IsDropDownOpen) { Step(combo, button == UiButton.Right ? 1 : -1); return; }
                    Key(window, focused, button == UiButton.Right ? Avalonia.Input.Key.Right : Avalonia.Input.Key.Left);
                    return;

                case UiButton.PageUp:
                case UiButton.PageDown:
                    if (tabs is not null && tabs.ItemCount > 0)
                        tabs.SelectedIndex = (tabs.SelectedIndex + (button == UiButton.PageDown ? 1 : tabs.ItemCount - 1)) % tabs.ItemCount;
                    return;

                case UiButton.Accept:
                    if (focused is TextBox) { SteamKeyboard.Show(); return; }
                    if (open is not null) { open.IsDropDownOpen = false; open.Focus(); return; }
                    if (focused is ComboBox closed) { closed.IsDropDownOpen = true; return; }
                    if (focused is ToggleButton toggle) { toggle.IsChecked = toggle.IsChecked != true; return; }
                    if (focused is Button pressed) { pressed.RaiseEvent(new RoutedEventArgs(Button.ClickEvent)); return; }
                    Key(window, focused, Avalonia.Input.Key.Enter);
                    return;

                case UiButton.Back:
                    if (open is not null) { open.IsDropDownOpen = false; open.Focus(); return; }
                    window.Close();
                    return;
            }
        }

        // A dropdown moved by one without being opened, which also commits it, as the arrow keys do on a closed one.
        private static void Step(ComboBox combo, int by)
        {
            if (combo.ItemCount == 0) return;
            combo.SelectedIndex = System.Math.Clamp(combo.SelectedIndex + by, 0, combo.ItemCount - 1);
        }

        private static void Key(Window window, InputElement? focused, Key key, KeyModifiers modifiers = KeyModifiers.None)
        {
            InputElement target = focused ?? window;
            target.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = key, KeyModifiers = modifiers, Source = target });
            target.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyUpEvent, Key = key, KeyModifiers = modifiers, Source = target });
        }
    }
}
