using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;

namespace EmuSen.WiseMan.Fixtures
{
    // §2.4's rules of EmuSen_LunaP_Kit.md as checks over a recorded state, so any frontend that writes the format is held to the same audit - see EmuSen_InterfaceSpec.md §4.
    public static class InterfaceRules
    {
        public static List<string> All(SpecDocument d) => [.. Fit(d), .. PadReach(d), .. Named(d), .. PaletteReach(d)];

        // The fit audit, FitAudit.Check's rules read from the recorded facts rather than from a live tree.
        public static List<string> Fit(SpecDocument d)
        {
            var faults = new List<string>();
            double slack = d.Number("slack", 1), unit = d.Number("unit", 1), floor = d.Number("floor");
            if (d["title"]?.EndsWith(" cut", StringComparison.Ordinal) == true) faults.Add($"title cut: {d["title"]}");
            if (d["footer"]?.EndsWith(" cut", StringComparison.Ordinal) == true) faults.Add($"footer cut: {d["footer"]}");
            bool Contains(SpecRect outer, SpecRect inner) =>
                inner.X >= outer.X - slack && inner.Y >= outer.Y - slack && inner.Right <= outer.Right + slack && inner.Bottom <= outer.Bottom + slack;
            bool Allowed(SpecNode n) => n.Has("allow");

            foreach (SpecNode n in d.Nodes.Where(n => n.Rect("limit") is not null && n["at"] is not null))
            {
                SpecRect r = n.At, limit = n.Rect("limit")!.Value;
                if (Contains(limit, r)) continue;
                bool across = r.X < limit.X - slack || r.Right > limit.Right + slack;
                bool along = r.Y < limit.Y - slack || r.Bottom > limit.Bottom + slack;
                if (n["by"] == "scroll" && !across && along) continue;
                faults.Add($"past its {n["by"]}: {Describe(n)} at {r} outside {limit}");
            }

            foreach (SpecNode n in d.Nodes.Where(n => n["owner"] is not null && n["at"] is not null && !Allowed(n)))
            {
                string o = n["owner"]!;
                SpecRect box = o.StartsWith('@') ? SpecRect.Parse(o[1..]) : d.Node(int.Parse(o, CultureInfo.InvariantCulture)).At;
                if (!Contains(box, n.At)) faults.Add($"words past their owner: {Describe(n)} at {n.At} outside {box}");
            }

            List<SpecNode> drawn = d.Nodes.Where(n => n.Has("drawn") && !n.Has("nohit") && n["at"] is not null).ToList();
            for (int i = 0; i < drawn.Count; i++)
                for (int j = i + 1; j < drawn.Count; j++)
                {
                    SpecNode a = drawn[i], b = drawn[j];
                    if (d.IsAncestor(a.Id, b.Id) || d.IsAncestor(b.Id, a.Id)) continue;
                    SpecRect both = a.Seen.Intersect(b.Seen);
                    if (both.W <= slack || both.H <= slack) continue;
                    if (Allowed(a) && Allowed(b)) continue;
                    faults.Add($"overlap: {Describe(a)} at {a.Seen} and {Describe(b)} at {b.Seen}");
                }

            foreach (SpecNode v in d.Nodes.Where(n => n["overflow"] is not null))
            {
                if (v.Number("overflow") > slack && v.At.H < 2 * 42 * unit - slack) faults.Add($"scrolls in too little room: {Describe(v)} {v.At.H:F0} high");
                if (v.Rect("viewport") is not { } area) continue;
                string fades = v["fades"] ?? "";
                List<SpecNode> within = d.Nodes.Where(n => n.Has("drawn") && n["at"] is not null && n.Ids("in").Contains(v.Id)).ToList();
                if (v.Number("overflow") > slack && !v.Has("vfixed"))
                {
                    if (!fades.Contains('t') && within.Any(n => n.At.Y < area.Y - slack && n.At.Bottom > area.Y + slack)) faults.Add($"cut at a scrolling edge with no fade: the top of {Describe(v)}");
                    if (!fades.Contains('b') && within.Any(n => n.At.Y < area.Bottom - slack && n.At.Bottom > area.Bottom + slack)) faults.Add($"cut at a scrolling edge with no fade: the foot of {Describe(v)}");
                }
                if (!fades.Contains('l') && within.Any(n => n.At.X < area.X - slack && n.At.Right > area.X + slack)) faults.Add($"cut at a scrolling edge with no fade: the left of {Describe(v)}");
                if (!fades.Contains('r') && within.Any(n => n.At.X < area.Right - slack && n.At.Right > area.Right + slack)) faults.Add($"cut at a scrolling edge with no fade: the right of {Describe(v)}");
            }

            foreach (SpecNode t in d.Nodes.Where(n => n["layout"] is not null && !Allowed(n)))
            {
                double[] layout = Pair(t["layout"]!), box = Pair(t["box"]!);
                if (t.Has("ellipsis")) faults.Add($"cut with an ellipsis: {Describe(t)}");
                else if (layout[0] > box[0] + slack) faults.Add($"runs past its box: {Describe(t)}, {layout[0]:F0} wide in {box[0]:F0}");
                else if (layout[1] > box[1] + slack) faults.Add($"cut at the foot: {Describe(t)}, {layout[1]:F0} high in {box[1]:F0}");
            }

            foreach (SpecNode t in d.Nodes.Where(n => n["caps"] is not null && !Allowed(n)))
                if (t.Number("caps")!.Value * unit < (floor * unit - 0.05) * FitAudit.DesktopCaps)
                    faults.Add($"too small to read: {Describe(t)}, capitals {t.Number("caps"):F2} design px under {floor * FitAudit.DesktopCaps:F2}");
            return faults;
        }

        // Pad reachability: the recorded walk from the focus reaches every operable control.
        public static List<string> PadReach(SpecDocument d)
        {
            if (d["walk"]?.StartsWith("none", StringComparison.Ordinal) == true) return [];
            if (d.Start is null) return d.Nodes.Any(n => n.Has("op")) ? ["no focus to walk from, with operable controls shown"] : [];
            HashSet<string> reached = Reach(d, d.Start);
            return d.Nodes.Where(n => n.Has("op") && !reached.Contains(n.Id.ToString(CultureInfo.InvariantCulture))).Select(n => $"not reached by the pad: {Describe(n)}").ToList();
        }

        private static HashSet<string> Reach(SpecDocument d, string start)
        {
            var reached = new HashSet<string>(StringComparer.Ordinal) { start };
            var queue = new Queue<string>([start]);
            ILookup<string, SpecEdge> from = d.Edges.ToLookup(e => e.From, StringComparer.Ordinal);
            while (queue.TryDequeue(out string? at))
                foreach (SpecEdge e in from[at])
                    if (e.To is not ("same" or "none") && reached.Add(e.To)) queue.Enqueue(e.To);
            return reached;
        }

        // Accessibility: every operable control has a name a reader speaks.
        public static List<string> Named(SpecDocument d) =>
            d.Nodes.Where(n => n.Has("op") && string.IsNullOrWhiteSpace(n["heard"])).Select(n => $"no name for a reader: {Describe(n)}").ToList();

        // Palette reach: every colour a control paints is one the state's resources offer; a scene's colours are its theme's, held to ES-DE's captures instead.
        public static List<string> PaletteReach(SpecDocument d)
        {
            if (d["kind"] == "scene") return [];
            var palette = (d["palette"] ?? "").Split(' ', StringSplitOptions.RemoveEmptyEntries).ToHashSet(StringComparer.Ordinal);
            return d.Nodes.SelectMany(n => (n["paints"] ?? "").Split(',', StringSplitOptions.RemoveEmptyEntries).Where(c => !palette.Contains(c)).Select(c => $"paints outside the palette: {Describe(n)} {c}")).ToList();
        }

        // A fault's kind, the same for FitAudit's words and these, so the two can be held to one another.
        public static string Category(string fault) => fault switch
        {
            _ when fault.StartsWith("words past", StringComparison.Ordinal) => "words past their owner",
            _ when fault.StartsWith("past its", StringComparison.Ordinal) => "past its limit",
            _ when fault.StartsWith("cut at a scrolling edge", StringComparison.Ordinal) => "cut at a scrolling edge",
            _ => fault[..Math.Max(0, fault.IndexOf(':'))],
        };

        private static double[] Pair(string s) => s.Split(',').Select(v => double.Parse(v, CultureInfo.InvariantCulture)).ToArray();

        public static string Describe(SpecNode n) => n["name"] is { } name ? $"{n.Kind} {name}" : n["text"] is { } t ? $"{n.Kind} '{(t.Length > 40 ? t[..40] + "…" : t)}'" : $"{n.Kind} #{n.Id}";
    }
}
