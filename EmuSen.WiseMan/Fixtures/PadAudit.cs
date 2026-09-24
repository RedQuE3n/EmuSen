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
        public static HashSet<InputElement> Reachable(Control root, PadDriver pad, int limit = 400)
        {
            TopLevel top = TopLevel.GetTopLevel(root)!;
            InputElement Focused() => (InputElement)top.FocusManager!.GetFocusedElement()!;

            var seen = new HashSet<InputElement>();
            var queue = new Queue<InputElement>();
            InputElement start = Focused();
            seen.Add(start);
            queue.Enqueue(start);

            while (queue.Count > 0 && seen.Count < limit)
            {
                InputElement from = queue.Dequeue();
                bool takesSideways = from is ComboBox or Slider or TabItem;

                foreach (int direction in takesSideways ? new[] { 0, 1 } : new[] { 0, 1, 2, 3 })
                {
                    from.Focus(NavigationMethod.Directional);
                    switch (direction)
                    {
                        case 0: pad.Up(); break;
                        case 1: pad.Down(); break;
                        case 2: pad.Left(); break;
                        case 3: pad.Right(); break;
                    }

                    InputElement to = Focused();
                    if (seen.Add(to)) queue.Enqueue(to);
                }
            }

            return seen;
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
            InputElement Focused() => (InputElement)top.FocusManager!.GetFocusedElement()!;
            System.Action[] presses = { () => pad.Up(), () => pad.Down(), () => pad.Left(), () => pad.Right() };

            InputElement start = Focused();
            var path = new Dictionary<InputElement, List<int>> { [start] = new List<int>() };
            var queue = new Queue<InputElement>(new[] { start });
            InputElement? found = target(start) ? start : null;

            while (found is null && queue.Count > 0 && path.Count < limit)
            {
                InputElement from = queue.Dequeue();
                bool takesSideways = from is ComboBox or Slider or TabItem;
                foreach (int direction in takesSideways ? new[] { 0, 1 } : new[] { 0, 1, 2, 3 })
                {
                    from.Focus(NavigationMethod.Directional);
                    presses[direction]();
                    InputElement to = Focused();
                    if (path.ContainsKey(to)) continue;
                    path[to] = new List<int>(path[from]) { direction };
                    queue.Enqueue(to);
                    if (target(to)) { found = to; break; }
                }
            }

            Assert.True(found is not null, "No pad path to the control asked for.");
            start.Focus(NavigationMethod.Directional);
            foreach (int direction in path[found!]) presses[direction]();
            return ReferenceEquals(found, Focused()) ? found : null;
        }

        public static string Describe(InputElement e) => e switch
        {
            ContentControl { Content: string text } c => $"{c.GetType().Name} '{text}'",
            TextBox t => $"TextBox {t.Name ?? t.PlaceholderText}",
            _ => $"{e.GetType().Name} {(e as Control)?.Name}",
        };
    }
}
