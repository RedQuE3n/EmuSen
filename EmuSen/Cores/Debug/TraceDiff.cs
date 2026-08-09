using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Cores.Debug
{
    // What the differ below needs of a trace record, whichever processor emitted it.
    public interface ITraceStep<T> where T : ITraceStep<T>
    {
        uint Addr { get; }
        byte Kind { get; }
        byte Opcode { get; }
        uint Cost { get; }
        bool SameRegisters(T other);
        string Registers { get; }
    }

    // Finds where two instruction streams stop agreeing - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
    public static class TraceDiff
    {
        // A block of Length addresses at Index repeated Repeats times; 1 for a straight-line step.
        public readonly record struct Node(int Index, int Length, int Repeats)
        {
            public int Steps => Length * Repeats;
        }

        public enum FindingKind { Cost, Registers, LoopCount, Structure, TruncatedSide }

        public readonly record struct Finding(FindingKind Kind, int LeftStep, int RightStep, string Detail)
        {
            // Sorted on, so "first divergence" means the same thing for every kind.
            public int Earliest => Math.Min(LeftStep, RightStep);
        }

        // One (opcode, leftCost, rightCost) disagreement and how often it happened;
        public readonly record struct CostRow(byte Opcode, uint LeftCost, uint RightCost, int Count, uint FirstAddr);

        public sealed record Result(
            int LeftSteps, int RightSteps, int LeftNodes, int RightNodes,
            List<Finding> Findings, List<CostRow> CostRows, long LeftClocks, long RightClocks)
        {
            public Finding? First => Findings.Count == 0 ? null : Findings[0];
        }

        // A loop longer than this reads as a structural difference rather than a count.
        public const int MaxLoopPeriod = 32;

        // How far resync looks ahead, and how many nodes must line up to believe it.
        public const int ResyncWindow = 4096;
        public const int ResyncConfirm = 8;

        // Alignment identity; registers are compared separately as an earlier, stronger finding.
        private static ulong Key<T>(T s) where T : struct, ITraceStep<T> => ((ulong)s.Kind << 32) | s.Addr;

        // Turns a spin loop into one node, so a differing iteration count is a value not a desync.
        public static List<Node> Collapse<T>(T[] steps, int maxPeriod = MaxLoopPeriod) where T : struct, ITraceStep<T>
        {
            var nodes = new List<Node>();
            int n = steps.Length;
            int i = 0;
            while (i < n)
            {
                int period = 0;
                for (int p = 1; p <= maxPeriod && i + 2 * p <= n; p++)
                {
                    // Cheap reject first; most positions match no period at all.
                    if (Key(steps[i]) != Key(steps[i + p])) continue;

                    bool same = true;
                    for (int k = 1; k < p; k++)
                    {
                        if (Key(steps[i + k]) == Key(steps[i + p + k])) continue;
                        same = false;
                        break;
                    }
                    if (same) { period = p; break; }
                }

                if (period == 0) { nodes.Add(new Node(i, 1, 1)); i++; continue; }

                int repeats = 1;
                while (i + (repeats + 1) * period <= n && BlockMatches(steps, i, i + repeats * period, period))
                {
                    repeats++;
                }
                nodes.Add(new Node(i, period, repeats));
                i += period * repeats;
            }
            return nodes;
        }

        private static bool BlockMatches<T>(T[] steps, int a, int b, int length) where T : struct, ITraceStep<T>
        {
            for (int k = 0; k < length; k++)
            {
                if (Key(steps[a + k]) != Key(steps[b + k])) return false;
            }
            return true;
        }

        private static bool SameShape<T>(T[] left, List<Node> ln, int li, T[] right, List<Node> rn, int ri)
            where T : struct, ITraceStep<T>
        {
            Node a = ln[li], b = rn[ri];
            if (a.Length != b.Length) return false;
            return BlockMatches2(left, a.Index, right, b.Index, a.Length);
        }

        private static bool BlockMatches2<T>(T[] l, int a, T[] r, int b, int length) where T : struct, ITraceStep<T>
        {
            for (int k = 0; k < length; k++)
            {
                if (Key(l[a + k]) != Key(r[b + k])) return false;
            }
            return true;
        }

        // Past this only the two axes are searched - see §3.40 on why.
        private const int ResyncSquare = 64;

        // Smallest skip that lines the streams up again, so an insertion stays an insertion.
        private static (int DL, int DR)? FindResync<T>(
            T[] left, List<Node> ln, int li, T[] right, List<Node> rn, int ri) where T : struct, ITraceStep<T>
        {
            bool Confirms(int dl, int dr)
            {
                if (li + dl + ResyncConfirm > ln.Count || ri + dr + ResyncConfirm > rn.Count) return false;
                for (int k = 0; k < ResyncConfirm; k++)
                {
                    if (!SameShape(left, ln, li + dl + k, right, rn, ri + dr + k)) return false;
                }
                return true;
            }

            for (int total = 1; total <= Math.Min(ResyncSquare, ResyncWindow); total++)
            {
                for (int dl = 0; dl <= total; dl++)
                {
                    if (Confirms(dl, total - dl)) return (dl, total - dl);
                }
            }

            for (int total = ResyncSquare + 1; total <= ResyncWindow; total++)
            {
                if (Confirms(total, 0)) return (total, 0);
                if (Confirms(0, total)) return (0, total);
            }
            return null;
        }

        private static string Where<T>(T[] steps, List<Node> nodes, int at, int count) where T : struct, ITraceStep<T>
        {
            var parts = new List<string>();
            for (int i = at; i < Math.Min(nodes.Count, at + count); i++)
            {
                Node nd = nodes[i];
                string addrs = string.Join(",", Enumerable.Range(0, nd.Length).Select(k => $"${steps[nd.Index + k].Addr:X6}"));
                parts.Add(nd.Repeats > 1 ? $"[{addrs}]x{nd.Repeats}" : addrs);
            }
            return string.Join(" ", parts);
        }

        public static Result Compare<T>(T[] left, T[] right, int maxFindings = 24) where T : struct, ITraceStep<T>
        {
            var ln = Collapse(left);
            var rn = Collapse(right);
            var findings = new List<Finding>();

            var costs = new Dictionary<(byte, uint, uint), (int Count, uint FirstAddr)>();
            long leftClocks = 0, rightClocks = 0;
            bool costReported = false;

            // Alignment outruns the finding cap, so the cost table covers the whole shared prefix.
            int li = 0, ri = 0;
            while (li < ln.Count && ri < rn.Count)
            {
                if (!SameShape(left, ln, li, right, rn, ri))
                {
                    var resync = FindResync(left, ln, li, right, rn, ri);
                    if (resync == null)
                    {
                        if (findings.Count < maxFindings)
                        {
                            findings.Add(new Finding(FindingKind.Structure, ln[li].Index, rn[ri].Index,
                                $"Streams diverge and do not resync within {ResyncWindow} nodes.\n"
                              + $"    left  ran {Where(left, ln, li, 6)}\n"
                              + $"    right ran {Where(right, rn, ri, 6)}"));
                        }
                        break;
                    }

                    var (dl, dr) = resync.Value;
                    if (findings.Count < maxFindings)
                    {
                        findings.Add(new Finding(FindingKind.Structure, ln[li].Index, rn[ri].Index,
                            $"left executed {ln.Skip(li).Take(dl).Sum(x => x.Steps)} extra steps, "
                          + $"right {rn.Skip(ri).Take(dr).Sum(x => x.Steps)}, then the paths rejoin.\n"
                          + $"    left  ran {Where(left, ln, li, Math.Max(dl, 1))}\n"
                          + $"    right ran {Where(right, rn, ri, Math.Max(dr, 1))}"));
                    }
                    li += dl;
                    ri += dr;
                    continue;
                }

                if (ln[li].Repeats != rn[ri].Repeats)
                {
                    if (findings.Count < maxFindings)
                    {
                        findings.Add(new Finding(FindingKind.LoopCount, ln[li].Index, rn[ri].Index,
                            $"loop {Where(left, ln, li, 1)} ran {ln[li].Repeats}x on the left, {rn[ri].Repeats}x on the right."));
                    }
                }
                else
                {
                    bool regsReported = false;
                    for (int k = 0; k < ln[li].Length; k++)
                    {
                        T a = left[ln[li].Index + k], b = right[rn[ri].Index + k];

                        // Charged once per repeat, so a loop's cost carries its real weight.
                        leftClocks += (long)a.Cost * ln[li].Repeats;
                        rightClocks += (long)b.Cost * rn[ri].Repeats;

                        if (a.Cost != b.Cost && a.Kind == 0)
                        {
                            var key = (a.Opcode, a.Cost, b.Cost);
                            var prev = costs.TryGetValue(key, out var v) ? v : (0, a.Addr);
                            costs[key] = (prev.Item1 + ln[li].Repeats, prev.Item2);

                            if (!costReported && findings.Count < maxFindings)
                            {
                                costReported = true;
                                findings.Add(new Finding(FindingKind.Cost, ln[li].Index + k, rn[ri].Index + k,
                                    $"at ${a.Addr:X6} opcode {a.Opcode:X2} costs {a.Cost} master clocks on the left, {b.Cost} on the right."));
                            }
                        }

                        if (regsReported || a.SameRegisters(b)) continue;
                        regsReported = true;
                        if (findings.Count >= maxFindings) continue;

                        findings.Add(new Finding(FindingKind.Registers, ln[li].Index + k, rn[ri].Index + k,
                            $"at ${a.Addr:X6} (op {a.Opcode:X2}) the same instruction sees different inputs.\n"
                          + $"    left  {a.Registers}\n"
                          + $"    right {b.Registers}"));
                    }
                }

                li++;
                ri++;
            }

            var costRows = costs
                .Select(kv => new CostRow(kv.Key.Item1, kv.Key.Item2, kv.Key.Item3, kv.Value.Count, kv.Value.FirstAddr))
                .OrderByDescending(x => (long)x.Count * Math.Abs((long)x.LeftCost - x.RightCost))
                .ToList();

            if (findings.Count == 0 && left.Length != right.Length)
            {
                bool leftShort = li >= ln.Count;
                findings.Add(new Finding(FindingKind.TruncatedSide,
                    leftShort ? left.Length : ln[Math.Min(li, ln.Count - 1)].Index,
                    leftShort ? rn[Math.Min(ri, rn.Count - 1)].Index : right.Length,
                    $"The two streams agree for their whole shared length; the {(leftShort ? "left" : "right")} one "
                  + "just ends first. Raise its trace cap to compare further."));
            }

            findings.Sort((x, y) => x.Earliest.CompareTo(y.Earliest));
            return new Result(left.Length, right.Length, ln.Count, rn.Count, findings, costRows, leftClocks, rightClocks);
        }

        public static string Report(Result r, string leftLabel, string rightLabel)
        {
            var sb = new System.Text.StringBuilder();
            sb.AppendLine($"left  = {leftLabel}: {r.LeftSteps} steps, {r.LeftNodes} nodes after loop collapsing");
            sb.AppendLine($"right = {rightLabel}: {r.RightSteps} steps, {r.RightNodes} nodes after loop collapsing");
            sb.AppendLine();

            if (r.Findings.Count == 0 && r.CostRows.Count == 0)
            {
                sb.AppendLine("No divergence: both streams executed the same instructions, with the same register inputs and the same cycle cost.");
                return sb.ToString();
            }

            foreach (var f in r.Findings)
            {
                sb.AppendLine($"[{f.Kind}] left step {f.LeftStep}, right step {f.RightStep}: {f.Detail}");
            }

            if (r.CostRows.Count > 0)
            {
                long delta = r.RightClocks - r.LeftClocks;
                sb.AppendLine();
                sb.AppendLine($"=== cycle cost, over every aligned instruction ({r.CostRows.Count} opcode/cost combinations disagree) ===");
                sb.AppendLine($"    left {r.LeftClocks} master clocks, right {r.RightClocks} ({(delta >= 0 ? "+" : "")}{delta}, "
                            + $"{(r.LeftClocks == 0 ? 0 : 100.0 * delta / r.LeftClocks):F2}%)");
                sb.AppendLine("    opcode   left  right   count   total drift   first seen");
                foreach (var c in r.CostRows.Take(20))
                {
                    long drift = (long)c.Count * ((long)c.RightCost - c.LeftCost);
                    sb.AppendLine($"      {c.Opcode:X2}    {c.LeftCost,5} {c.RightCost,6} {c.Count,7} {drift,13}   ${c.FirstAddr:X6}");
                }
            }
            return sb.ToString();
        }
    }
}
