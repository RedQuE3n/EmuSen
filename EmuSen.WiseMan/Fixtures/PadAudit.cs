using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.VisualTree;
using Xunit;

namespace EmuSen.WiseMan.Fixtures
{
    // Which of a window's controls the d-pad can reach from its first, found by pressing every direction from every control reached - see EmuSen_Settings_Reference.md §4.45.3.
    public static class PadAudit
    {
        // What a player would want to reach: something that does a thing, not the panel around it.
        public static bool IsOperable(InputElement e) =>
            e.Focusable && e.IsEffectivelyEnabled && e.IsEffectivelyVisible
            && (e is Button or ToggleButton or ComboBox or Slider or TextBox or ListBoxItem || e is TabItem { IsSelected: true })
            && !(e is ToggleButton && (e as Visual)!.FindAncestorOfType<ListBoxItem>() is not null);

        public static List<InputElement> Operable(Control root) =>
            root.GetVisualDescendants().OfType<InputElement>().Where(IsOperable).ToList();

        // Every control reachable by a pad from where the focus is now; left and right are not pressed where the control itself takes them.
        public static HashSet<InputElement> Reachable(Control root, PadDriver pad, int limit = 400) => Reachable(root, pad.Press, limit);

        // The same walk with the presses sent some other way, such as straight to the router for a window the headless platform will not make active.
        public static HashSet<InputElement> Reachable(Control root, System.Action<EmuSen.Mistress.Input.UiButton> press, int limit = 400)
        {
            System.Action[] presses =
            {
                () => press(EmuSen.Mistress.Input.UiButton.Up), () => press(EmuSen.Mistress.Input.UiButton.Down),
                () => press(EmuSen.Mistress.Input.UiButton.Left), () => press(EmuSen.Mistress.Input.UiButton.Right),
            };
            Dictionary<object, List<int>> paths = Walk(root, presses, _ => false, limit, out _);
            return root.GetVisualDescendants().OfType<InputElement>().Where(e => paths.ContainsKey(KeyOf(e))).ToHashSet();
        }

        // Finds a pad path from the focus to a control, then goes back and walks it with the pad alone; the search is repeated from where a walk ends, since a move can depend on the scroll position it left.
        public static InputElement Reach(Control root, PadDriver pad, System.Func<InputElement, bool> target, int limit = 400)
        {
            TopLevel top = TopLevel.GetTopLevel(root)!;
            for (int attempt = 0; attempt < 6; attempt++)
            {
                if (TryReach(root, pad, target, limit) is { } reached) return reached;
            }
            Assert.Fail($"No pad walk ends on the control asked for; the focus is on {Describe((InputElement)top.FocusManager!.GetFocusedElement()!)}.");
            return null!;
        }

        private static InputElement? TryReach(Control root, PadDriver pad, System.Func<InputElement, bool> target, int limit)
        {
            TopLevel top = TopLevel.GetTopLevel(root)!;
            System.Action[] presses = { () => pad.Up(), () => pad.Down(), () => pad.Left(), () => pad.Right() };
            InputElement start = (InputElement)top.FocusManager!.GetFocusedElement()!;
            object startKey = KeyOf(start);

            Dictionary<object, List<int>> paths = Walk(root, presses, target, limit, out object? found);
            Assert.True(found is not null, "No pad path to the control asked for.");
            Refocus(top, startKey, start);
            foreach (int direction in paths[found!]) presses[direction]();
            return top.FocusManager!.GetFocusedElement() is InputElement end && Equals(KeyOf(end), found) && target(end) ? end : null;
        }

        // Breadth first over every direction from every control reached, each left from as the walk found it, by replaying its path - see EmuSen_Settings_Reference.md §4.48.5.
        private static Dictionary<object, List<int>> Walk(Control root, System.Action[] presses, System.Func<InputElement, bool> target, int limit, out object? found)
        {
            TopLevel top = TopLevel.GetTopLevel(root)!;
            InputElement? Focused() => top.FocusManager!.GetFocusedElement() as InputElement;

            InputElement start = Focused()!;
            object startKey = KeyOf(start);
            var paths = new Dictionary<object, List<int>> { [startKey] = new List<int>() };
            var queue = new Queue<object>(new[] { startKey });
            found = target(start) ? startKey : null;

            // Every list's selection and every scrolling area's offset as the walk began: a row given the focus is selected, a list is entered at its selected row, and a move is by position.
            var selections = root.GetVisualDescendants().OfType<ListBox>().Select(l => (List: l, Index: l.SelectedIndex)).ToList();
            var scrolls = root.GetVisualDescendants().OfType<ScrollViewer>().Select(v => (Viewer: v, Offset: v.Offset)).ToList();

            // From the start along the path, lists and scrolling put back first, never by focusing the control directly: a pane that follows a selection makes the path the state a control was found in.
            InputElement? At(object key)
            {
                foreach ((ListBox list, int index) in selections)
                    if (list.SelectedIndex != index) list.SelectedIndex = index;
                top.UpdateLayout();
                foreach ((ScrollViewer viewer, Vector offset) in scrolls)
                    if (((Visual)viewer).IsAttachedToVisualTree() && viewer.Offset != offset) viewer.Offset = offset;
                top.UpdateLayout();
                Refocus(top, startKey, start);
                foreach (int direction in paths[key]) presses[direction]();
                return Focused() is { } replayed && Equals(KeyOf(replayed), key) ? replayed : null;
            }

            while (found is null && queue.Count > 0 && paths.Count < limit)
            {
                object key = queue.Dequeue();
                if (At(key) is not { } from) continue;
                bool takesSideways = from is ComboBox or Slider or TabItem;
                foreach (int direction in takesSideways ? new[] { 0, 1 } : new[] { 0, 1, 2, 3 })
                {
                    if (At(key) is null) break;
                    presses[direction]();
                    if (Focused() is not { } to) continue;
                    object toKey = KeyOf(to);
                    if (paths.ContainsKey(toKey)) continue;
                    paths[toKey] = new List<int>(paths[key]) { direction };
                    queue.Enqueue(toKey);
                    if (target(to)) { found = toKey; break; }
                }
            }
            return paths;
        }

        // A row is its list and index, since a virtualised list recycles containers; anything else is where it sits in the tree, so a pane rebuilt the same way is the same controls.
        private static object KeyOf(InputElement e)
        {
            if (e is ListBoxItem row && row.FindAncestorOfType<ListBox>() is { } list && list.IndexFromContainer(row) is var index and >= 0) return (list, index);
            var steps = new List<int>();
            Visual? top = TopLevel.GetTopLevel(e);
            for (Visual? child = e, parent = e.GetVisualParent(); parent is not null && !ReferenceEquals(child, top); child = parent, parent = parent.GetVisualParent())
                steps.Add(parent.GetVisualChildren().ToList().IndexOf(child!));
            steps.Reverse();
            return string.Join('/', steps);
        }

        // The control a key names in the window as it is now, focused as the pad would, or null when nothing is there.
        private static InputElement? Refocus(TopLevel top, object key, InputElement fallback)
        {
            InputElement? element = null;
            if (key is (ListBox list, int index))
            {
                list.ScrollIntoView(index);
                list.UpdateLayout();
                element = list.ContainerFromIndex(index) as InputElement;
            }
            else if (key is string path)
            {
                Visual? at = top;
                foreach (string step in path.Split('/', System.StringSplitOptions.RemoveEmptyEntries))
                {
                    var children = at?.GetVisualChildren().ToList();
                    int i = int.Parse(step);
                    at = children is not null && i >= 0 && i < children.Count ? children[i] : null;
                }
                element = at as InputElement;
            }
            else element = fallback;

            if (element is null || !((Visual)element).IsAttachedToVisualTree() || !element.IsEffectivelyVisible || !element.Focusable) return null;
            element.Focus(NavigationMethod.Directional);
            top.UpdateLayout();
            return element;
        }

        public static string Describe(InputElement e) => e switch
        {
            ContentControl { Content: string text } c => $"{c.GetType().Name} '{text}'",
            TextBox t => $"TextBox {t.Name ?? t.PlaceholderText}",
            _ => $"{e.GetType().Name} {(e as Control)?.Name}",
        };
    }
}
