using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Presenters;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Input
{
    // What a window is listening for while it rebinds a control; the pad router stands aside for it - see EmuSen_Settings_Reference.md §4.45.4.
    public enum PadCapture { None, Key, PadButton }

    // A window that captures a key or a pad button, and can be told to stop.
    public interface IPadCapturing
    {
        PadCapture Capturing { get; }
        void CancelCapture();
    }

    // A window that takes some pad buttons itself; what it declines goes to the router - see EmuSen_Settings_Reference.md §4.49.
    public interface IPadDriven
    {
        bool OnPad(UiButton button);
    }

    // A pad driving a window built for a keyboard and a pointer: focus moves by position, and each control is operated the way its own keys would - see EmuSen_Settings_Reference.md §4.29 and §4.45.3.
    public static class PadWindowRouter
    {
        public static void Send(Window window, UiButton button)
        {
            Control root = RootOf(window);
            TopLevel? top = TopLevel.GetTopLevel(root);
            var focused = top?.FocusManager?.GetFocusedElement() as InputElement;
            ComboBox? open = root.GetVisualDescendants().OfType<ComboBox>().FirstOrDefault(c => c.IsDropDownOpen);

            // An open keyboard takes every button until it is put away.
            if (OnScreenKeyboard.OpenOver(root) is { } keyboard)
            {
                PadKeyboard.Send(keyboard, button);
                return;
            }

            if (window is IPadCapturing capturing && capturing.Capturing != PadCapture.None)
            {
                if (capturing.Capturing == PadCapture.Key && button == UiButton.Back) capturing.CancelCapture();
                return;
            }

            if (open is null && window is IPadDriven driven && driven.OnPad(button)) return;

            // Focus left under the sheet, nowhere, or on a control since hidden starts again at the window's first control - see EmuSen_Settings_Reference.md §4.48.10.
            if (open is null && (focused is not Visual at || !IsWithin(at, root) || !focused.IsEffectivelyVisible))
            {
                FocusFirst(root);
                if (button is UiButton.Up or UiButton.Down or UiButton.Left or UiButton.Right) return;
                focused = top?.FocusManager?.GetFocusedElement() as InputElement;
            }

            switch (button)
            {
                case UiButton.Up:
                case UiButton.Down:
                {
                    bool down = button == UiButton.Down;
                    if (open is not null) { Highlight(open, focused, down ? 1 : -1); return; }
                    if (focused is ListBoxItem row && row.FindAncestorOfType<ListBox>() is { } list && !AtEdge(list, row, down))
                    {
                        Key(row, down ? Avalonia.Input.Key.Down : Avalonia.Input.Key.Up);
                        return;
                    }
                    Move(root, focused, down ? NavigationDirection.Down : NavigationDirection.Up);
                    return;
                }

                case UiButton.Left:
                case UiButton.Right:
                {
                    int by = button == UiButton.Right ? 1 : -1;
                    if (open is not null) return;
                    if (focused is ComboBox combo) { Step(combo, by); return; }
                    if (focused is Slider or ISidewaysAdjustable) { Key(focused, by > 0 ? Avalonia.Input.Key.Right : Avalonia.Input.Key.Left); return; }
                    if (focused is TabItem tab && tab.FindAncestorOfType<TabControl>() is { } strip) { StepTab(strip, by, focusHeader: true); return; }
                    Move(root, focused, by > 0 ? NavigationDirection.Right : NavigationDirection.Left);
                    return;
                }

                case UiButton.PageUp:
                case UiButton.PageDown:
                {
                    TabControl? tabs = (focused as Visual)?.FindAncestorOfType<TabControl>(includeSelf: true)
                        ?? root.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();
                    if (open is null && tabs is not null) StepTab(tabs, button == UiButton.PageDown ? 1 : -1, focusHeader: false);
                    return;
                }

                case UiButton.Accept:
                    if (open is not null) { Choose(open, focused); return; }
                    if (focused is TextBox box) { PadKeyboard.Open(box); return; }
                    if (focused is ComboBox closed) { closed.IsDropDownOpen = true; return; }
                    if (focused is TabItem header) { header.IsSelected = true; return; }
                    if (focused is ToggleButton toggle) { toggle.IsChecked = toggle.IsChecked != true; return; }
                    if (focused is Button pressed) { pressed.RaiseEvent(new RoutedEventArgs(Button.ClickEvent)); return; }
                    if (focused is ListBoxItem item && item.GetVisualDescendants().OfType<ToggleButton>().FirstOrDefault(t => t.IsEffectivelyEnabled) is { } tick)
                    {
                        tick.IsChecked = tick.IsChecked != true;
                        return;
                    }
                    // A row given the focus by navigation is already selected (Avalonia's own rule); Enter is how a list that acts on a row is told to - §4.45.3.
                    if (focused is not null) Key(focused, Avalonia.Input.Key.Enter);
                    return;

                case UiButton.Back:
                    if (open is not null) { Close(open); return; }
                    window.Close();
                    return;
            }
        }

        // The sheet a presented window is drawn on, else the window itself.
        public static Control RootOf(Window window) => SheetLayer.PresenterOf(window)?.SheetOf(window) ?? window;

        private static bool IsWithin(Visual visual, Control root) =>
            ReferenceEquals(visual, root) || visual.GetVisualAncestors().Contains(root);

        private static void FocusFirst(Control root)
        {
            InputElement? first = root.GetVisualDescendants().OfType<InputElement>()
                .FirstOrDefault(e => e.Focusable && e.IsEffectivelyEnabled && e.IsEffectivelyVisible && e is not ScrollViewer);
            first?.Focus(NavigationMethod.Directional);
        }

        private static bool AtEdge(ListBox list, ListBoxItem row, bool down)
        {
            int index = list.IndexFromContainer(row);
            return down ? index >= list.ItemCount - 1 : index <= 0;
        }

        // By position, nearest scrolling area first, so a control scrolled out of view is reached before one beyond the area - see §4.45.3.
        private static void Move(Control root, InputElement? from, NavigationDirection direction)
        {
            if (TopLevel.GetTopLevel(root)?.FocusManager is not { } focus || from is null) return;

            // Down from a tab header goes into its page; Avalonia 12.1's search answers a header beside it on the same row - §4.45.3.
            if (from is TabItem && direction == NavigationDirection.Down && from.FindAncestorOfType<TabControl>() is { } strip && PageHost(strip) is { } page)
            {
                (FirstIn(page) ?? Next(focus, page, root, direction) ?? Nearest(root, page, direction))?.Focus(NavigationMethod.Directional);
                return;
            }

            for (Visual? scope = (from as Visual)?.FindAncestorOfType<ScrollViewer>(); scope is not null && IsWithin(scope, root);
                 scope = scope.FindAncestorOfType<ScrollViewer>())
            {
                if (Next(focus, from, (InputElement)scope, direction) is { } inside) { inside.Focus(NavigationMethod.Directional); return; }
            }

            // Nothing in line with the focus: the nearest control that way at all, so a button off to one side is still reached - §4.45.3.
            (Next(focus, from, root, direction) ?? Nearest(root, from, direction))?.Focus(NavigationMethod.Directional);
        }

        // Avalonia 12.1's search answers nothing for a control below and wholly to one side; this scores by distance that way, then twice the distance across.
        private static InputElement? Nearest(Control root, InputElement from, NavigationDirection direction)
        {
            if (from.TranslatePoint(default, root) is not { } origin) return null;
            var here = new Rect(origin, from.Bounds.Size);

            InputElement? best = null;
            double bestScore = double.MaxValue;
            foreach (InputElement candidate in root.GetVisualDescendants().OfType<InputElement>())
            {
                if (ReferenceEquals(candidate, from) || !candidate.Focusable || !candidate.IsEffectivelyEnabled || !candidate.IsEffectivelyVisible) continue;
                if (candidate is ScrollViewer || candidate is TabItem { IsSelected: false }) continue;
                if (candidate.TranslatePoint(default, root) is not { } at) continue;
                var there = new Rect(at, candidate.Bounds.Size);

                (double along, double across) = direction switch
                {
                    NavigationDirection.Down => (there.Top - here.Bottom, Gap(here.Left, here.Right, there.Left, there.Right)),
                    NavigationDirection.Up => (here.Top - there.Bottom, Gap(here.Left, here.Right, there.Left, there.Right)),
                    NavigationDirection.Right => (there.Left - here.Right, Gap(here.Top, here.Bottom, there.Top, there.Bottom)),
                    _ => (here.Left - there.Right, Gap(here.Top, here.Bottom, there.Top, there.Bottom)),
                };
                if (along < -1) continue;

                double score = System.Math.Max(along, 0) + 2 * across;
                if (score < bestScore) { bestScore = score; best = candidate; }
            }
            return best;
        }

        private static double Gap(double start, double end, double otherStart, double otherEnd) =>
            otherEnd < start ? start - otherEnd : otherStart > end ? otherStart - end : 0;

        // A tab strip is entered at its selected tab, since focusing another would select it - L1 and R1 are what change tabs.
        private static InputElement? Next(IFocusManager focus, InputElement from, InputElement scope, NavigationDirection direction)
        {
            InputElement? next = null;
            InputElement at = from;
            for (int tries = 0; next is null; tries++)
            {
                if (tries == 16) return null;
                if (focus.FindNextElement(direction, new FindNextElementOptions { SearchRoot = scope, FocusedElement = at}) is not InputElement found
                    || ReferenceEquals(found, at) || !IsWithin(found, (Control)scope)) return null;

                // A scrolling area is entered at the control it holds nearest the way the pad moved, or passed over when it holds none.
                if (found is ScrollViewer area)
                {
                    next = focus.FindNextElement(direction, new FindNextElementOptions { SearchRoot = area, FocusedElement = from}) as InputElement;
                    if (next is not null && (next is ScrollViewer || !IsWithin(next, area))) next = null;
                    at = area;
                    continue;
                }

                // Another header of the strip the focus is already on is passed over; the strip is one stop.
                if (found is TabItem { IsSelected: false } && from is TabItem && ReferenceEquals(found.GetVisualParent(), from.GetVisualParent()))
                {
                    at = found;
                    continue;
                }

                next = found;
            }

            if (next is TabItem { IsSelected: false } other && other.FindAncestorOfType<TabControl>() is { } tabs
                && tabs.ContainerFromIndex(tabs.SelectedIndex) is TabItem selected)
                return ReferenceEquals(selected, from) ? null : selected;
            return next;
        }

        // A dropdown moved by one without being opened, which also commits it, as the arrow keys do on a closed one.
        private static void Step(ComboBox combo, int by)
        {
            if (combo.ItemCount == 0) return;
            combo.SelectedIndex = System.Math.Clamp(combo.SelectedIndex + by, 0, combo.ItemCount - 1);
        }

        // The focus moved among an open list's items directly, as a key sent there would reach the page behind when the list is drawn in the window - see EmuSen_Settings_Reference.md §4.45.9.
        private static void Highlight(ComboBox open, InputElement? focused, int by)
        {
            if (open.ItemCount == 0) return;
            int at = focused is ComboBoxItem item && open.IndexFromContainer(item) is >= 0 and var i ? i : open.SelectedIndex;
            int next = System.Math.Clamp(at + by, 0, open.ItemCount - 1);
            open.ScrollIntoView(next);
            (open.ContainerFromIndex(next) as InputElement)?.Focus(NavigationMethod.Directional);
        }

        // The highlighted item chosen, then the list closed.
        private static void Choose(ComboBox open, InputElement? focused)
        {
            if (focused is ComboBoxItem item && open.IndexFromContainer(item) is >= 0 and var i) open.SelectedIndex = i;
            Close(open);
        }

        // Closed without choosing: the highlight moved the focus, never the selection.
        private static void Close(ComboBox open)
        {
            open.IsDropDownOpen = false;
            open.Focus(NavigationMethod.Directional);
        }

        // The next tab, wrapping; the focus goes to its header from the strip, else into its page.
        private static void StepTab(TabControl tabs, int by, bool focusHeader)
        {
            int count = tabs.ItemCount;
            if (count == 0) return;
            tabs.SelectedIndex = (tabs.SelectedIndex + by + count) % count;
            TopLevel.GetTopLevel(tabs)?.UpdateLayout();

            if (focusHeader && tabs.ContainerFromIndex(tabs.SelectedIndex) is InputElement header) { header.Focus(NavigationMethod.Directional); return; }

            InputElement? first = PageHost(tabs) is { } page ? FirstIn(page) : null;
            (first ?? tabs.ContainerFromIndex(tabs.SelectedIndex) as InputElement)?.Focus(NavigationMethod.Directional);
        }

        private static ContentPresenter? PageHost(TabControl tabs) =>
            tabs.GetVisualDescendants().OfType<ContentPresenter>().FirstOrDefault(p => p.Name == "PART_SelectedContentHost");

        // The control nearest the top left of a page, as a sheet starts.
        private static InputElement? FirstIn(Control page) =>
            page.GetVisualDescendants().OfType<InputElement>()
                .Where(e => e.Focusable && e.IsEffectivelyEnabled && e.IsEffectivelyVisible && e is not ScrollViewer && InView((Visual)e, page))
                .OrderBy(e => System.Math.Round(e.TranslatePoint(default, page)?.Y ?? 0)).ThenBy(e => e.TranslatePoint(default, page)?.X ?? 0)
                .FirstOrDefault();

        // Whether some of a control shows through every scrolling area around it inside the page; a row scrolled away is not the page's first control - see EmuSen_Settings_Reference.md §4.48.10.
        private static bool InView(Visual e, Control page)
        {
            for (ScrollViewer? area = e.FindAncestorOfType<ScrollViewer>(); area is not null && IsWithin(area, page); area = area.FindAncestorOfType<ScrollViewer>())
                if (e.TranslatePoint(default, area) is not { } at || !new Rect(at, e.Bounds.Size).Intersects(new Rect(area.Bounds.Size))) return false;
            return true;
        }

        private static void Key(InputElement target, Key key, KeyModifiers modifiers = KeyModifiers.None)
        {
            target.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = key, KeyModifiers = modifiers, Source = target });
            target.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyUpEvent, Key = key, KeyModifiers = modifiers, Source = target });
        }
    }
}
