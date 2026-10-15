using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A rectangle in the recorded root's pixels.
    public readonly record struct SpecRect(double X, double Y, double W, double H)
    {
        public double Right => X + W;
        public double Bottom => Y + H;

        public SpecRect Intersect(SpecRect o)
        {
            double x = Math.Max(X, o.X), y = Math.Max(Y, o.Y), r = Math.Min(Right, o.Right), b = Math.Min(Bottom, o.Bottom);
            return r <= x || b <= y ? default : new SpecRect(x, y, r - x, b - y);
        }

        public override string ToString() => string.Join(',', new[] { X, Y, W, H }.Select(InterfaceSpec.Num));

        public static SpecRect Parse(string s)
        {
            double[] v = s.Split(',').Select(p => double.Parse(p, CultureInfo.InvariantCulture)).ToArray();
            return new SpecRect(v[0], v[1], v[2], v[3]);
        }
    }

    // One control of a recorded state: its place in the tree and the facts §2.4's rules read - see EmuSen_InterfaceSpec.md §2.
    public sealed class SpecNode
    {
        public int Id { get; set; }
        public int Parent { get; set; } = -1;
        public int Depth { get; set; }
        public string Kind { get; set; } = "";
        public SortedDictionary<string, string> Attrs { get; } = new(StringComparer.Ordinal);

        public string? this[string key] => Attrs.TryGetValue(key, out string? v) ? v : null;
        public bool Has(string flag) => Attrs.ContainsKey(flag);
        public SpecRect? Rect(string key) => this[key] is { } v ? SpecRect.Parse(v) : null;
        public double? Number(string key) => this[key] is { } v ? double.Parse(v, CultureInfo.InvariantCulture) : null;
        public SpecRect At => Rect("at")!.Value;
        public SpecRect Seen => Rect("seen") ?? At;
        public IEnumerable<int> Ids(string key) => this[key] is { } v ? v.Split(',').Select(int.Parse) : [];
    }

    // A recorded press: the button and what it reached.
    public sealed record SpecPress(string Button, SortedDictionary<string, string> Attrs);

    // A focus edge: from a node, a direction, to a node id or a reference to a control built only once scrolled to.
    public sealed record SpecEdge(string From, string Direction, string To);

    // One state of the interface as recorded: header facts, the tree, the focus graph and the presses - see EmuSen_InterfaceSpec.md.
    public sealed class SpecDocument
    {
        public const int Version = 1;
        public const string Banner = "# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.";

        public SortedDictionary<string, string> Header { get; } = new(StringComparer.Ordinal);
        public List<SpecNode> Nodes { get; } = new();
        public string? Start { get; set; }
        public List<SpecEdge> Edges { get; } = new();
        public List<SpecPress> Presses { get; } = new();

        public string State => Header["state"];
        public string? this[string key] => Header.TryGetValue(key, out string? v) ? v : null;
        public double Number(string key, double fallback = 0) => this[key] is { } v ? double.Parse(v, CultureInfo.InvariantCulture) : fallback;

        public SpecNode Node(int id) => Nodes[id];

        public bool IsAncestor(int ancestor, int of)
        {
            for (int at = Nodes[of].Parent; at >= 0; at = Nodes[at].Parent)
                if (at == ancestor) return true;
            return false;
        }

        public string Write()
        {
            var sb = new StringBuilder();
            sb.Append(Banner).Append('\n');
            sb.Append("spec ").Append(Version).Append('\n');
            foreach (string key in new[] { "state", "size", "kind", "root", "unit", "slack", "floor", "frame", "title", "footer" })
                if (Header.TryGetValue(key, out string? v)) sb.Append(key).Append(' ').Append(v).Append('\n');
            foreach ((string key, string v) in Header.Where(h => h.Key is not ("state" or "size" or "kind" or "root" or "unit" or "slack" or "floor" or "frame" or "title" or "footer")))
                sb.Append(key).Append(' ').Append(v).Append('\n');
            sb.Append("tree\n");
            foreach (SpecNode n in Nodes)
            {
                sb.Append(' ', 2 * (n.Depth + 1)).Append(n.Id).Append(' ').Append(n.Kind);
                foreach ((string k, string v) in n.Attrs) sb.Append(' ').Append(k).Append(v.Length == 0 ? "" : "=" + InterfaceSpec.Quote(v));
                sb.Append('\n');
            }
            sb.Append("focus ").Append(Start ?? "none").Append('\n');
            foreach (SpecEdge e in Edges) sb.Append("edge ").Append(InterfaceSpec.Quote(e.From)).Append(' ').Append(e.Direction).Append(' ').Append(InterfaceSpec.Quote(e.To)).Append('\n');
            foreach (SpecPress p in Presses)
            {
                sb.Append("press ").Append(p.Button);
                foreach ((string k, string v) in p.Attrs) sb.Append(' ').Append(k).Append('=').Append(InterfaceSpec.Quote(v));
                sb.Append('\n');
            }
            return sb.ToString();
        }

        public static SpecDocument Parse(string text)
        {
            var doc = new SpecDocument();
            var stack = new List<int>();
            bool inTree = false;
            foreach (string raw in text.Split('\n'))
            {
                if (raw.Length == 0 || raw.StartsWith('#')) continue;
                if (inTree && raw.StartsWith("  ", StringComparison.Ordinal))
                {
                    int indent = raw.Length - raw.TrimStart(' ').Length;
                    List<string> t = InterfaceSpec.Tokens(raw.TrimStart(' '));
                    var node = new SpecNode { Id = int.Parse(t[0], CultureInfo.InvariantCulture), Kind = t[1], Depth = indent / 2 - 1 };
                    if (node.Id != doc.Nodes.Count) throw new FormatException($"node {node.Id} out of order");
                    while (stack.Count > node.Depth) stack.RemoveAt(stack.Count - 1);
                    node.Parent = stack.Count > 0 ? stack[^1] : -1;
                    stack.Add(node.Id);
                    foreach (string a in t.Skip(2)) { int eq = a.IndexOf('='); node.Attrs[eq < 0 ? a : a[..eq]] = eq < 0 ? "" : a[(eq + 1)..]; }
                    doc.Nodes.Add(node);
                    continue;
                }
                inTree = false;
                int space = raw.IndexOf(' ');
                string word = space < 0 ? raw : raw[..space], rest = space < 0 ? "" : raw[(space + 1)..];
                switch (word)
                {
                    case "spec":
                        if (int.Parse(rest, CultureInfo.InvariantCulture) != Version) throw new FormatException($"spec version {rest}, this reader is {Version}");
                        break;
                    case "tree": inTree = true; break;
                    case "focus": doc.Start = rest == "none" ? null : rest; break;
                    case "edge":
                        List<string> e = InterfaceSpec.Tokens(rest);
                        doc.Edges.Add(new SpecEdge(e[0], e[1], e[2]));
                        break;
                    case "press":
                        List<string> p = InterfaceSpec.Tokens(rest);
                        var attrs = new SortedDictionary<string, string>(StringComparer.Ordinal);
                        foreach (string a in p.Skip(1)) { int eq = a.IndexOf('='); attrs[a[..eq]] = a[(eq + 1)..]; }
                        doc.Presses.Add(new SpecPress(p[0], attrs));
                        break;
                    default: doc.Header[word] = rest; break;
                }
            }
            return doc;
        }
    }

    // The format's small pieces: numbers to two places, quoting, and tokens split with quotes respected.
    public static class InterfaceSpec
    {
        public static string Num(double v) => Math.Round(v, 2).ToString("0.##", CultureInfo.InvariantCulture) is "-0" ? "0" : Math.Round(v, 2).ToString("0.##", CultureInfo.InvariantCulture);

        public static string Quote(string v)
        {
            if (v.Length > 0 && v.All(c => c > ' ' && c != '"' && c != '\\' && c != '=' && c < 0x7F)) return v;
            var sb = new StringBuilder("\"");
            foreach (char c in v)
                sb.Append(c switch { '"' => "\\\"", '\\' => "\\\\", '\n' => "\\n", '\r' => "\\r", '\t' => "\\t", _ => c.ToString() });
            return sb.Append('"').ToString();
        }

        // Splits on spaces outside quotes; a quoted value comes back unquoted and unescaped.
        public static List<string> Tokens(string line)
        {
            var tokens = new List<string>();
            var cur = new StringBuilder();
            bool quoted = false, any = false;
            for (int i = 0; i < line.Length; i++)
            {
                char c = line[i];
                if (quoted)
                {
                    if (c == '\\' && i + 1 < line.Length)
                    {
                        char n = line[++i];
                        cur.Append(n switch { 'n' => '\n', 'r' => '\r', 't' => '\t', _ => n });
                    }
                    else if (c == '"') quoted = false;
                    else cur.Append(c);
                }
                else if (c == '"') { quoted = true; any = true; }
                else if (c == ' ') { if (cur.Length > 0 || any) tokens.Add(cur.ToString()); cur.Clear(); any = false; }
                else cur.Append(c);
            }
            if (cur.Length > 0 || any) tokens.Add(cur.ToString());
            return tokens;
        }

        // How one recording differs from another within §7.3's tolerances: the same named controls with their kinds' roles and words, each rectangle within the tolerance, the same focus graph by name.
        public static List<string> Compare(SpecDocument expected, SpecDocument actual, double tolerance = 2)
        {
            var diffs = new List<string>();
            double unit = expected.Number("unit", 1);
            Dictionary<string, SpecNode> Named(SpecDocument d) => d.Nodes.Where(n => n["name"] is { Length: > 0 }).GroupBy(n => n["name"]!).ToDictionary(g => g.Key, g => g.First());
            Dictionary<string, SpecNode> want = Named(expected), have = Named(actual);
            foreach ((string name, SpecNode w) in want)
            {
                if (!have.TryGetValue(name, out SpecNode? h)) { diffs.Add($"missing: {w.Kind} {name}"); continue; }
                if (w["role"] != h["role"]) diffs.Add($"{name}: role {h["role"]}, recorded {w["role"]}");
                if (w["text"] != h["text"]) diffs.Add($"{name}: text '{h["text"]}', recorded '{w["text"]}'");
                if (w["at"] is null || h["at"] is null) continue;
                SpecRect a = w.At, b = h.At;
                double off = new[] { a.X - b.X, a.Y - b.Y, a.Right - b.Right, a.Bottom - b.Bottom }.Max(Math.Abs) / unit;
                if (off > tolerance) diffs.Add($"{name}: at {b}, recorded {a}, {off:F1} design px apart");
            }
            foreach (string name in have.Keys.Where(n => !want.ContainsKey(n))) diffs.Add($"extra: {have[name].Kind} {name}");
            static string Label(SpecDocument d, string to) => int.TryParse(to, out int id) ? d.Node(id)["name"] ?? d.Node(id)["heard"] ?? $"#{id}" : to;
            HashSet<string> Graph(SpecDocument d) => d.Edges.Select(e => $"{Label(d, e.From)} {e.Direction} {Label(d, e.To)}").ToHashSet();
            HashSet<string> g1 = Graph(expected), g2 = Graph(actual);
            foreach (string e in g1.Except(g2)) diffs.Add($"focus edge missing: {e}");
            foreach (string e in g2.Except(g1)) diffs.Add($"focus edge extra: {e}");
            return diffs;
        }
    }
}
