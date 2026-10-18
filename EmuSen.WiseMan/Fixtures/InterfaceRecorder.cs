using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using Avalonia;
using Avalonia.Automation;
using Avalonia.Automation.Peers;
using Avalonia.Controls;
using Avalonia.Controls.Presenters;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Media.TextFormatting;
using Avalonia.Styling;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;

namespace EmuSen.WiseMan.Fixtures
{
    // Records one state of Mistress's interface as a SpecDocument: the tree with the facts §2.4's rules read, the pad's focus graph and the grammar's presses - see EmuSen_InterfaceSpec.md §2 and §3.
    public static class InterfaceRecorder
    {
        public static readonly string[] Directions = ["up", "down", "left", "right"];

        // Text that names a run's own folders is written as the folder's part, so two runs record alike.
        public static Func<string, string> Normalise { get; set; } = s => s;

        public sealed record Options(string State, string Kind, double SmallestText = FitAudit.SmallestText, IReadOnlyList<FitAudit.Allowance>? Allow = null, Func<Control, string?>? Element = null);

        // What a state shows from the wall clock or a running game, by control and why; such a control is recorded without its words and measure, and without what is inside it - see EmuSen_InterfaceSpec.md §6.
        public static Func<Control, string?> Volatile { get; set; } = _ => null;

        // The tree and the header; the focus graph and the presses are added by Walk and Press.
        public static SpecDocument Tree(Control root, Options o) => Tree(root, o, out _);

        public static SpecDocument Tree(Control root, Options o, out Dictionary<Control, int> ids)
        {
            var allow = o.Allow ?? [];
            var doc = new SpecDocument();
            TopLevel top = TopLevel.GetTopLevel(root)!;
            double unit = MenuPanel.GetScale(root) is > 0 and var u ? u : 1;
            double slack = FitAudit.SlackFor(root);
            Rect frame = FitAudit.FrameOf(root);
            doc.Header["state"] = o.State;
            doc.Header["size"] = $"{top.Bounds.Width:0} {top.Bounds.Height:0}";
            doc.Header["kind"] = o.Kind;
            doc.Header["root"] = root.GetType().Name + (root.Name is { Length: > 0 } n ? "#" + n : "");
            doc.Header["unit"] = InterfaceSpec.Num(unit);
            doc.Header["slack"] = InterfaceSpec.Num(slack);
            doc.Header["floor"] = InterfaceSpec.Num(o.SmallestText);
            doc.Header["frame"] = Of(frame).ToString();
            if (root.GetVisualDescendants().OfType<MenuPanel>().FirstOrDefault(m => m.Name == "SheetMenu") is { } menu)
            {
                if (menu.Title is { Length: > 0 } title) doc.Header["title"] = InterfaceSpec.Quote(Normalise(title)) + (menu.IsTitleCut ? " cut" : "");
                if (menu.Footer is { Length: > 0 } footer) doc.Header["footer"] = InterfaceSpec.Quote(Normalise(footer)) + (menu.IsFooterCut ? " cut" : "");
            }
            doc.Header["palette"] = string.Join(' ', Palette(root));

            List<(Control C, Rect R, Rect Seen)> shown = FitAudit.Shown(root, frame).ToList();
            var keep = new HashSet<Control>(shown.Where(s => Kept(s.C) && !UnderVolatile(s.C, root)).Select(s => s.C));
            ids = new Dictionary<Control, int>();
            var seenOf = shown.ToDictionary(s => s.C, s => (s.R, s.Seen));
            var viewers = shown.Select(s => s.C).OfType<ScrollViewer>().Where(v => v.Presenter is not null).ToList();
            foreach (Control c in shown.Select(s => s.C).Where(keep.Contains))
            {
                (Rect r, Rect seen) = seenOf[c];
                int parent = -1, depth = 0;
                for (Visual? a = c.GetVisualParent(); a is not null && !ReferenceEquals(a, root); a = a.GetVisualParent())
                    if (a is Control pc && ids.TryGetValue(pc, out int pid)) { parent = pid; depth = doc.Nodes[pid].Depth + 1; break; }
                var node = new SpecNode { Id = doc.Nodes.Count, Parent = parent, Depth = depth, Kind = c.GetType().Name.Split('`')[0] };
                ids[c] = node.Id;
                doc.Nodes.Add(node);
                Facts(node, c, r, seen, root, frame, unit, allow, viewers, ids, o);
            }
            var paints = new Dictionary<int, SortedSet<string>>();
            foreach (Control c in root.GetVisualDescendants().OfType<Control>())
            {
                if (!c.IsEffectivelyVisible || c.Opacity <= 0) continue;
                int owner = -1;
                for (Visual? at = c; at is not null && !ReferenceEquals(at, root); at = at.GetVisualParent())
                    if (at is Control k && ids.TryGetValue(k, out int id)) { owner = id; break; }
                if (owner < 0) continue;
                if (!paints.TryGetValue(owner, out SortedSet<string>? set)) paints[owner] = set = new SortedSet<string>(StringComparer.Ordinal);
                Paint(c, set);
            }
            foreach ((int id, SortedSet<string> set) in paints) if (set.Count > 0) doc.Nodes[id].Attrs["paints"] = string.Join(',', set);
            return doc;
        }

        private static string? VolatileWhy(Control c) =>
            Volatile(c) ?? (TextOf(c) is { Length: > 0 } t && Normalise(t) != t ? "its words name the run's own folders" : null);

        private static bool UnderVolatile(Control c, Control root)
        {
            for (Visual? a = c.GetVisualParent(); a is not null && !ReferenceEquals(a, root); a = a.GetVisualParent())
                if (a is Control k && VolatileWhy(k) is not null) return true;
            return false;
        }

        // What the record keeps: every control the window placed, and of a control's own parts what draws words, pictures or something to press.
        private static bool Kept(Control c) =>
            c.TemplatedParent is null || FitAudit.Drawn(c) || c is ScrollViewer || c is InputElement { Focusable: true } && PadAudit.IsOperable(c);

        private static void Facts(SpecNode n, Control c, Rect r, Rect seen, Control root, Rect frame, double unit, IReadOnlyList<FitAudit.Allowance> allow,
            List<ScrollViewer> viewers, Dictionary<Control, int> ids, Options o)
        {
            var a = n.Attrs;
            AutomationPeer peer = ControlAutomationPeer.CreatePeerForElement(c);
            a["role"] = peer.GetAutomationControlType().ToString().ToLowerInvariant();
            if (c.Name is { Length: > 0 } name) a["name"] = name;
            if (AutomationProperties.GetLiveSetting(c) is not AutomationLiveSetting.Off and var live) a["live"] = live.ToString().ToLowerInvariant();
            if (c.TemplatedParent is not null) a["part"] = "";
            // A control whose words or measure come from the wall clock, a running game or the run's own folders is recorded by what it is and why, not by where or what it shows.
            if (VolatileWhy(c) is { } why)
            {
                a["volatile"] = why;
                if (peer.GetName() is { Length: > 0 } named && Normalise(named) == named && named != TextOf(c)) a["heard"] = named;
                if (c is InputElement { Focusable: true } fe && PadAudit.IsOperable(fe)) a[PadAudit.Unseen(fe) ? "unseen" : "op"] = "";
                return;
            }
            if (peer.GetName() is { Length: > 0 } heard) a["heard"] = Normalise(heard);
            a["at"] = Of(r).ToString();
            if (Of(seen) != Of(r)) a["seen"] = Of(seen).ToString();
            if (TextOf(c) is { Length: > 0 } text) a["text"] = Normalise(text);
            if (o.Element?.Invoke(c) is { } element) a["element"] = element;

            bool allowed = FitAudit.Allowed(c, allow);
            if (allowed) a["allow"] = allow.First(x => x.Applies(c)).Why;
            if (c is InputElement ie)
            {
                if (PadAudit.IsOperable(ie)) a[PadAudit.Unseen(ie) ? "unseen" : "op"] = "";
                if (ie.Focusable && ie.IsEffectivelyEnabled) a["focus"] = "";
                if (!ie.IsEffectivelyEnabled) a["off"] = "";
                if (PadAudit.TakesSideways(ie)) a["sideways"] = "";
            }
            if (FitAudit.Drawn(c)) a["drawn"] = "";
            if (!FitAudit.HitTestable(c, root)) a["nohit"] = "";

            // The limit the fit audit holds it to: its nearest clip or scroller, else the panel.
            if (!allowed && (c.TemplatedParent is null || c is TextBlock or Image))
            {
                (Rect limit, Visual? by) = FitAudit.Limit(c, root, frame);
                a["limit"] = Of(limit).ToString();
                if (by is ScrollContentPresenter p && p.FindAncestorOfType<ScrollViewer>() is { } viewer)
                    a["by"] = viewer.VerticalScrollBarVisibility != ScrollBarVisibility.Disabled && FitAudit.Overflow(viewer, root) > FitAudit.Slack ? "scroll" : "fixed-scroll";
                else a["by"] = by is null ? "panel" : "clip";
            }

            // Words inside a push button, a tab or a field, which draw no clip of their own.
            if (c is TextBlock && c.GetVisualAncestors().FirstOrDefault(v => v is Button or ToggleButton or TabItem or ComboBox or ComboBoxItem or ListBoxItem) is Control owner)
                a["owner"] = ids.TryGetValue(owner, out int oid) ? oid.ToString(CultureInfo.InvariantCulture) : "@" + Of(FitAudit.Area(owner, root));

            var inside = viewers.Where(v => v.Presenter is Visual pr && pr.IsVisualAncestorOf(c) && ids.ContainsKey(v)).Select(v => ids[v]).ToList();
            if (inside.Count > 0) a["in"] = string.Join(',', inside);

            if (c is ScrollViewer sv && c.TemplatedParent is not TextBox)
            {
                a["overflow"] = InterfaceSpec.Num(FitAudit.Overflow(sv, root));
                if (sv.Presenter is Visual presenter) a["viewport"] = Of(FitAudit.Area(presenter, root)).ToString();
                string fades = (MenuLook.GetFadesTop(sv) ? "t" : "") + (MenuLook.GetFadesBottom(sv) ? "b" : "") + (MenuLook.GetFadesLeft(sv) ? "l" : "") + (MenuLook.GetFadesRight(sv) ? "r" : "");
                if (fades.Length > 0) a["fades"] = fades;
                if (sv.VerticalScrollBarVisibility == ScrollBarVisibility.Disabled) a["vfixed"] = "";
            }

            if (c is TextBlock { Text.Length: > 0 } t)
            {
                TextLayout layout = t.TextLayout;
                if (layout.TextLines.Any(l => l.HasCollapsed)) a["ellipsis"] = "";
                a["layout"] = $"{InterfaceSpec.Num(layout.Width)},{InterfaceSpec.Num(layout.Height)}";
                a["box"] = $"{InterfaceSpec.Num(t.Bounds.Width - t.Padding.Left - t.Padding.Right)},{InterfaceSpec.Num(t.Bounds.Height - t.Padding.Top - t.Padding.Bottom)}";
            }
            (double size, GlyphTypeface? face) = c switch
            {
                TextBlock { Text.Length: > 0 } tb => (tb.FontSize, FitAudit.FaceOf(tb)),
                ControllerDiagram d => (d.LabelTextSize, FitAudit.FaceOf(new Typeface(d.GetValue(Avalonia.Controls.Documents.TextElement.FontFamilyProperty)))),
                _ => (0, null),
            };
            if (size > 0)
            {
                double shownAt = size * (c.TransformToVisual(root)?.M11 ?? 1);
                a["size"] = InterfaceSpec.Num(shownAt / unit);
                a["caps"] = (shownAt * FitAudit.CapsOf(face) / unit).ToString("0.####", CultureInfo.InvariantCulture);
                if (face is not null) a["face"] = face.FamilyName;
            }
        }

        // The words a control shows: a text block's, a field's, or a string a content control holds.
        private static string? TextOf(Control c) => c switch
        {
            TextBlock t => t.Text,
            TextBox b => b.Text,
            ContentControl { Content: string s } => s,
            HeaderedContentControl { Header: string h } => h,
            _ => null,
        };

        // The colours one control paints, as LunaP's PaletteSweep reads them: its background, foreground and border.
        private static void Paint(Control control, SortedSet<string> colours)
        {
            foreach (IBrush? brush in new[]
            {
                (control as TemplatedControl)?.Background ?? (control as Panel)?.Background ?? (control as Border)?.Background,
                (control as TemplatedControl)?.Foreground ?? (control as TextBlock)?.Foreground,
                (control as TemplatedControl)?.BorderBrush ?? (control as Border)?.BorderBrush,
            })
                if (brush is ISolidColorBrush { Color.A: > 0 } solid) colours.Add(Hex(solid.Color));
        }

        // The colours the state's resources offer: the application's and every style and resource dictionary from the root up, in the variant the root is drawn in.
        public static SortedSet<string> Palette(Control root)
        {
            var colours = new SortedSet<string>(StringComparer.Ordinal);
            ThemeVariant variant = root.ActualThemeVariant;
            void Dictionary(IResourceDictionary? d)
            {
                if (d is null) return;
                foreach (object key in d.Keys.ToList()) if (d.TryGetResource(key, variant, out object? value)) Value(value);
                foreach (IResourceProvider merged in d.MergedDictionaries) if (merged is IResourceDictionary md) Dictionary(md);
                foreach ((ThemeVariant key, IThemeVariantProvider provider) in d.ThemeDictionaries)
                    if (key == variant && provider is IResourceDictionary td) Dictionary(td);
            }
            void Value(object? value)
            {
                if (value is Color color && color.A > 0) colours.Add(Hex(color));
                else if (value is ISolidColorBrush { Color.A: > 0 } brush) colours.Add(Hex(brush.Color));
            }
            void Styles(Avalonia.Styling.Styles styles)
            {
                Dictionary(styles.Resources);
                foreach (IStyle s in styles)
                {
                    if (s is Avalonia.Styling.Styles nested) Styles(nested);
                    else if (s is StyleBase sb) Dictionary(sb.Resources);
                }
            }
            if (Application.Current is { } app) { Dictionary(app.Resources); Styles(app.Styles); }
            // A scope above the root, or one inside it, such as the menu look a sheet's content is given.
            var scopes = new HashSet<StyledElement>();
            for (StyledElement? at = root; at is not null; at = at.Parent) scopes.Add(at);
            for (Visual? at = root; at is not null; at = at.GetVisualParent()) scopes.Add(at);
            foreach (Visual v in root.GetVisualDescendants()) scopes.Add(v);
            foreach (StyledElement at in scopes)
            {
                Dictionary(at.Resources);
                Styles(at.Styles);
            }
            foreach (System.Reflection.FieldInfo f in typeof(EmuSen.LunaP.Theme.LunaPalette).GetFields(System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static))
                Value(f.GetValue(null));
            return colours;
        }

        private static string Hex(Color c) => $"#{c.R:X2}{c.G:X2}{c.B:X2}{c.A:X2}";

        private static SpecRect Of(Rect r) => new(Round(r.X), Round(r.Y), Round(r.Width), Round(r.Height));

        private static double Round(double v) => Math.Round(v, 2);

        // The focus graph the pad walk finds from the focus, each landing named by a node or, for a row built only once scrolled to, by its list and index.
        public static void Walk(SpecDocument doc, Control root, PadDriver pad, Dictionary<Control, int> ids, int limit = 400, Func<bool>? settled = null)
        {
            TopLevel top = TopLevel.GetTopLevel(root)!;
            bool Inside() => top.FocusManager?.GetFocusedElement() is Visual f && f.IsAttachedToVisualTree() && (ReferenceEquals(f, root) || root.IsVisualAncestorOf(f));
            // A state whose focus is not yet in it, as a tab chosen by a test leaves it, is entered as a player enters it: by a press down.
            if (!Inside())
            {
                pad.Down();
                if (!Inside()) { doc.Start = null; return; }
                doc.Header["entered"] = "down";
            }
            var byKey = new Dictionary<object, int>();
            var nodes = new Dictionary<int, InputElement>();
            foreach ((Control c, int id) in ids)
                if (c is InputElement e && e.Focusable) byKey.TryAdd(PadAudit.KeyOf(e), id);

            string Name(object? key)
            {
                if (key is null) return "none";
                if (byKey.TryGetValue(key, out int id)) return id.ToString(CultureInfo.InvariantCulture);
                return key switch
                {
                    (ListBox list, int index, string heard) => $"row:{Ref(list)}/{index}/{Normalise(heard)}",
                    (ItemsControl owner, int item, string path, string heard) => $"item:{Ref(owner)}/{item}/{path}/{Normalise(heard)}",
                    string path => $"path:{path}",
                    _ => "other",
                };
            }
            string Ref(Control c) => ids.TryGetValue(c, out int id) ? id.ToString(CultureInfo.InvariantCulture) : c.GetType().Name;

            (object start, List<(object From, int Direction, object? To)> edges, bool truncated) = PadAudit.Graph(root, pad, ids.Keys.OfType<InputElement>().Where(e => PadAudit.IsOperable(e) && !PadAudit.Unseen(e)).Select(PadAudit.KeyOf).ToList(), limit, settled);
            doc.Start = Name(start);
            var seen = new HashSet<(string, int)>();
            foreach ((object from, int direction, object? to) in edges)
            {
                string f = Name(from);
                if (!seen.Add((f, direction))) continue;
                string t = Equals(from, to) ? "same" : Name(to);
                doc.Edges.Add(new SpecEdge(f, Directions[direction], t));
            }
            static (int, string) Order(string s) => int.TryParse(s, out int n) ? (n, "") : (int.MaxValue, s);
            doc.Edges.Sort((x, y) => Order(x.From).CompareTo(Order(y.From)) is var c and not 0 ? c : Array.IndexOf(Directions, x.Direction).CompareTo(Array.IndexOf(Directions, y.Direction)));
            if (truncated) doc.Header["walk"] = $"truncated at {limit}";
        }
    }
}
