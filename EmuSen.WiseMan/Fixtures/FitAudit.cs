using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Presenters;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.TextFormatting;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;

namespace EmuSen.WiseMan.Fixtures
{
    // Whether everything a window shows fits: nothing past its panel or its clipping ancestor, nothing drawn over something else, no text cut - see EmuSen_Settings_Reference.md §4.83.
    public static class FitAudit
    {
        // A cut the audit accepts, because the full text is reachable another way; each names that way.
        public sealed record Allowance(string Why, Func<Control, bool> Applies);

        // A text box scrolls its own text, and the caret and the on-screen keyboard reach all of it.
        public static readonly Allowance TextBoxesScroll = new("a text box scrolls its own text", c => c.FindAncestorOfType<TextBox>() is not null);

        // The smallest words a framed sheet may draw, as a size in the desktop face in design pixels (a screen pixel at 1280 by 800), four fifths of the look's small text; other faces are held to its capitals (§4.83.7).
        public const double SmallestText = 16;

        // A pixel of slack, and a pixel and a half at a menu's scale above one, where layout rounds in the scaled content's own units.
        [System.ThreadStatic] private static double _slack;
        private static double Slack => _slack > 0 ? _slack : 1.0;

        public static List<string> Check(Control root, IReadOnlyList<Allowance>? allow = null, double smallest = SmallestText)
        {
            allow ??= [];
            var faults = new List<string>();
            LastSmallest = null;
            _slack = System.Math.Max(1.0, (MenuPanel.GetScale(root) is > 0 and var scale ? scale : 1) + 0.5);
            Rect frame = FrameOf(root);
            List<(Control C, Rect R, Rect Seen)> shown = Shown(root, frame).ToList();
            if (root.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu") is { IsTitleCut: true } menu)
                faults.Add($"title cut: '{menu.Title}'");
            if (root.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu") is { IsFooterCut: true } cut)
                faults.Add($"footer cut: '{cut.Footer}'");

            foreach ((Control c, Rect r, _) in shown)
            {
                // A control's own template parts are its business, such as a slider's wide focus target; what they draw is checked as words and pictures.
                if (Allowed(c, allow) || c.TemplatedParent is not null && c is not (TextBlock or Image)) continue;
                (Rect limit, Visual? by) = Limit(c, root, frame);
                if (Contains(limit, r)) continue;
                if (by is ScrollContentPresenter presenter && Scrolls(presenter, r, limit, root)) continue;
                faults.Add($"past its {(by is null ? "panel" : Describe(by))}: {Describe(c)} at {Show(r)} outside {Show(limit)}");
            }

            // Text inside a push button, a tab or a field stays inside it, since those draw no clip of their own.
            foreach ((Control c, Rect r, _) in shown.Where(s => s.C is TextBlock))
            {
                if (Allowed(c, allow)) continue;
                Visual? owner = c.GetVisualAncestors().FirstOrDefault(a => a is Button or ToggleButton or TabItem or ComboBox or ComboBoxItem or ListBoxItem);
                if (owner is null) continue;
                Rect box = Area(owner, root);
                if (!Contains(box, r)) faults.Add($"words past their {Describe(owner)}: {Describe(c)} at {Show(r)} outside {Show(box)}");
            }

            List<(Control C, Rect Seen)> drawn = shown.Where(s => Drawn(s.C) && HitTestable(s.C, root)).Select(s => (s.C, s.Seen)).ToList();
            for (int i = 0; i < drawn.Count; i++)
                for (int j = i + 1; j < drawn.Count; j++)
                {
                    (Control a, Rect ra) = drawn[i];
                    (Control b, Rect rb) = drawn[j];
                    if (a.IsVisualAncestorOf(b) || b.IsVisualAncestorOf(a)) continue;
                    Rect both = ra.Intersect(rb);
                    if (both.Width <= Slack || both.Height <= Slack) continue;
                    if (Allowed(a, allow) && Allowed(b, allow)) continue;
                    faults.Add($"overlap: {Describe(a)} at {Show(ra)} and {Describe(b)} at {Show(rb)}");
                }

            // A scrolling area too short to show two rows of 42 design pixels is a list nobody can read, however far it scrolls.
            double unit = MenuPanel.GetScale(root) is > 0 and var u ? u : 1;
            foreach ((Control c, Rect r, _) in shown)
            {
                if (c is not ScrollViewer viewer || c.TemplatedParent is TextBox || Overflow(viewer, root) <= Slack) continue;
                if (r.Height < 2 * 42 * unit - Slack) faults.Add($"scrolls in too little room: {Describe(c)} {r.Height:F0} high");
                // Something sliced by a scrolling edge reads as cut unless the edge fades (Q186).
                if (viewer.VerticalScrollBarVisibility == ScrollBarVisibility.Disabled || viewer.Presenter is not Visual presenter) continue;
                Rect area = Area(presenter, root);
                bool Sliced(double edge) => shown.Any(s => Drawn(s.C) && presenter.IsVisualAncestorOf(s.C) && s.R.Top < edge - Slack && s.R.Bottom > edge + Slack);
                if (!MenuLook.GetFadesTop(viewer) && Sliced(area.Top)) faults.Add($"cut at a scrolling edge with no fade: the top of {Describe(c)}");
                if (!MenuLook.GetFadesBottom(viewer) && Sliced(area.Bottom)) faults.Add($"cut at a scrolling edge with no fade: the foot of {Describe(c)}");
            }

            // The same across, for an area that scrolls sideways, such as a strip of tiles.
            foreach ((Control c, _, _) in shown)
            {
                if (c is not ScrollViewer viewer || c.TemplatedParent is TextBox || viewer.Presenter is not Visual presenter) continue;
                Rect area = Area(presenter, root);
                bool Sliced(double edge) => shown.Any(s => Drawn(s.C) && presenter.IsVisualAncestorOf(s.C) && s.R.Left < edge - Slack && s.R.Right > edge + Slack);
                if (!MenuLook.GetFadesLeft(viewer) && Sliced(area.Left)) faults.Add($"cut at a scrolling edge with no fade: the left of {Describe(c)}");
                if (!MenuLook.GetFadesRight(viewer) && Sliced(area.Right)) faults.Add($"cut at a scrolling edge with no fade: the right of {Describe(c)}");
            }

            foreach ((Control c, _, _) in shown)
            {
                if (c is not TextBlock text || string.IsNullOrEmpty(text.Text) || Allowed(c, allow)) continue;
                TextLayout layout = text.TextLayout;
                double width = text.Bounds.Width - text.Padding.Left - text.Padding.Right;
                double height = text.Bounds.Height - text.Padding.Top - text.Padding.Bottom;
                if (layout.TextLines.Any(l => l.HasCollapsed)) faults.Add($"cut with an ellipsis: {Describe(c)}");
                else if (layout.Width > width + Slack) faults.Add($"runs past its box: {Describe(c)}, {layout.Width:F0} wide in {width:F0}");
                else if (layout.Height > height + Slack) faults.Add($"cut at the foot: {Describe(c)}, {layout.Height:F0} high in {height:F0}");
            }

            // Capitals shorter than the desktop face's at SmallestText do not read from a handheld's arm's length or a television's sofa (§4.83.7).
            foreach ((Control c, _, _) in shown)
            {
                (double size, GlyphTypeface? face) = c switch
                {
                    TextBlock { Text.Length: > 0 } t => (t.FontSize, FaceOf(t)),
                    ControllerDiagram d => (d.LabelTextSize, FaceOf(new Typeface(d.GetValue(Avalonia.Controls.Documents.TextElement.FontFamilyProperty)))),
                    _ => (0, null),
                };
                if (size <= 0 || Allowed(c, allow)) continue;
                double shownAt = size * (c.TransformToVisual(root)?.M11 ?? 1);
                double caps = shownAt * CapsOf(face);
                if (LastSmallest is not { } least || caps / unit < least.Caps) LastSmallest = (caps / unit, shownAt / unit, $"{Describe(c)} in {face?.FamilyName}");
                if (caps < (smallest * unit - 0.05) * DesktopCaps)
                    faults.Add($"too small to read: {Describe(c)}, capitals {caps / unit:F2} design px under {smallest * DesktopCaps:F2} ({shownAt / unit:F1} px of {face?.FamilyName ?? "an unknown face"})");
            }
            return faults;
        }

        // The shortest capitals the last check measured, in design pixels, with their size and what drew them, for the record.
        [System.ThreadStatic] public static (double Caps, double Size, string What)? LastSmallest;

        // Inter's capitals as a share of its size, measured from its H: the floor is the capitals the desktop face draws at SmallestText (§4.83.7).
        public const double DesktopCaps = 0.7273;

        private static readonly Dictionary<GlyphTypeface, double> Caps = new();

        // How tall a face draws its capitals, as a share of its size, from the height of its H; a face with no H counts as the desktop's.
        public static double CapsOf(GlyphTypeface? face)
        {
            if (face is null) return DesktopCaps;
            lock (Caps)
            {
                if (Caps.TryGetValue(face, out double known)) return known;
                // The outline of an H drawn at a thousand pixels, since this Avalonia reports no ink height for a glyph.
                double share = face.CharacterToGlyphMap.TryGetGlyph('H', out ushort glyph) && glyph != 0
                    && new GlyphRun(face, 1000, "H".AsMemory(), new[] { glyph }).BuildGeometry() is { Bounds.Height: > 0 } outline
                    ? outline.Bounds.Height / 1000
                    : DesktopCaps;
                Caps[face] = share;
                return share;
            }
        }

        // The face a text block's words are drawn in, after any fallback, else the one its typeface names.
        private static GlyphTypeface? FaceOf(TextBlock text) =>
            text.TextLayout.TextLines.SelectMany(l => l.TextRuns).OfType<ShapedTextRun>().FirstOrDefault()?.ShapedBuffer.GlyphTypeface
            ?? FaceOf(new Typeface(text.FontFamily, text.FontStyle, text.FontWeight));

        public static GlyphTypeface? FaceOf(Typeface typeface) => FontManager.Current.TryGetGlyphTypeface(typeface, out GlyphTypeface? face) ? face : null;

        // The framed panel's rectangle when the root is a framed sheet, else the root's own.
        private static Rect FrameOf(Control root) =>
            root.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu") is { } panel
                ? panel.PanelBounds.TransformToAABB(panel.TransformToVisual(root) ?? Matrix.Identity)
                : new Rect(root.Bounds.Size);

        private static Rect Area(Visual v, Visual root) => new Rect(v.Bounds.Size).TransformToAABB(v.TransformToVisual(root) ?? Matrix.Identity);

        // Every visible control with a size, where it is, and what of it shows through every clip above it.
        private static IEnumerable<(Control C, Rect R, Rect Seen)> Shown(Control root, Rect frame)
        {
            Control content = root.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu")?.Child ?? root;
            foreach (Control c in content.GetVisualDescendants().OfType<Control>())
            {
                // A popup is a placeholder in the tree; what it shows opens over the window on purpose.
                if (c is Popup) continue;
                if (!c.IsEffectivelyVisible || c.Opacity <= 0 || c.Bounds.Width <= 0 || c.Bounds.Height <= 0) continue;
                if (c.GetVisualAncestors().OfType<Visual>().Any(a => a.Opacity <= 0)) continue;
                Rect r = Area(c, root);
                Rect seen = r.Intersect(frame);
                for (Visual? a = c.GetVisualParent(); a is not null && !ReferenceEquals(a, root); a = a.GetVisualParent())
                    if (a.ClipToBounds || a is ScrollContentPresenter) seen = seen.Intersect(Area(a, root));
                if (seen.Width <= 0 || seen.Height <= 0) continue;
                yield return (c, r, seen);
            }
        }

        // The nearest clipping or scrolling ancestor's rectangle, or the panel's.
        private static (Rect Limit, Visual? By) Limit(Control c, Control root, Rect frame)
        {
            for (Visual? a = c.GetVisualParent(); a is not null && !ReferenceEquals(a, root); a = a.GetVisualParent())
            {
                if (a is ScrollContentPresenter or LayoutTransformControl || a.ClipToBounds && a is not TextPresenter)
                {
                    if (a is LayoutTransformControl) return (frame.Intersect(Area(a, root)), null);
                    return (Area(a, root), a);
                }
            }
            return (frame, null);
        }

        // A scrolling area that can bring the overflow into view along the axis it overflows.
        private static bool Scrolls(ScrollContentPresenter presenter, Rect r, Rect limit, Visual root)
        {
            if (presenter.FindAncestorOfType<ScrollViewer>() is not { } viewer) return false;
            bool across = r.Left < limit.Left - Slack || r.Right > limit.Right + Slack;
            bool along = r.Top < limit.Top - Slack || r.Bottom > limit.Bottom + Slack;
            // The pad scrolls up and down, by moving the focus or a page at a time, and never sideways: text past a scroller's side is cut.
            bool canAcross = false;
            bool canAlong = viewer.VerticalScrollBarVisibility != ScrollBarVisibility.Disabled && Overflow(viewer, root) > Slack;
            return (!across || canAcross) && (!along || canAlong);
        }

        // How far a scroller's content runs past its view, in the root's pixels as the slack is, not the scroller's own scaled units (§4.83.6).
        private static double Overflow(ScrollViewer viewer, Visual root) => (viewer.Extent.Height - viewer.Viewport.Height) * (viewer.TransformToVisual(root)?.M22 ?? 1);

        private static bool Contains(Rect outer, Rect inner) =>
            inner.Left >= outer.Left - Slack && inner.Top >= outer.Top - Slack && inner.Right <= outer.Right + Slack && inner.Bottom <= outer.Bottom + Slack;

        // What draws something a player reads or presses: words, a picture, or a control.
        private static bool Drawn(Control c) => c is TextBlock or Image or FittedImage or RgbaImageView or Button or ToggleButton or TextBox or ComboBox or Slider
            or ToggleSwitch or ProgressBar or ListBoxItem or TabItem or MeterRow;

        private static bool HitTestable(Control c, Visual root)
        {
            for (Visual? v = c; v is not null && !ReferenceEquals(v, root); v = v.GetVisualParent())
                if (v is InputElement { IsHitTestVisible: false }) return false;
            return true;
        }

        private static bool Allowed(Control c, IReadOnlyList<Allowance> allow) => allow.Any(a => a.Applies(c));

        public static string Describe(Visual v) => v switch
        {
            TextBlock t => $"{v.GetType().Name} '{Short(t.Text)}'",
            Control { Name: { Length: > 0 } n } => $"{v.GetType().Name} {n}",
            ContentControl { Content: string s } => $"{v.GetType().Name} '{Short(s)}'",
            _ => v.GetType().Name,
        };

        private static string Short(string? s) => s is null ? "" : s.Length > 40 ? s[..40] + "…" : s;

        private static string Show(Rect r) => $"{r.X:F0},{r.Y:F0} {r.Width:F0}×{r.Height:F0}";
    }
}
