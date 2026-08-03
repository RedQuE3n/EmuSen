using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // Finds where two CpuBinaryTrace streams stop agreeing - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
    public static class CpuTraceDiff
    {
        public readonly record struct Step(uint Addr, byte Opcode, byte Kind,
            ushort A, ushort X, ushort Y, ushort S, ushort D, byte Db, byte P, bool E)
        {
            public bool SameRegisters(Step o) =>
                A == o.A && X == o.X && Y == o.Y && S == o.S && D == o.D && Db == o.Db && P == o.P && E == o.E;

            public string Registers => $"A={A:X4} X={X:X4} Y={Y:X4} S={S:X4} D={D:X4} DB={Db:X2} P={P:X2}{(E ? " E" : "")}";
        }

        // A block of Length addresses at Index repeated Repeats times; 1 for a straight-line step.
        public readonly record struct Node(int Index, int Length, int Repeats)
        {
            public int Steps => Length * Repeats;
        }

        public enum FindingKind { Registers, LoopCount, Structure, TruncatedSide }

        public readonly record struct Finding(FindingKind Kind, int LeftStep, int RightStep, string Detail)
        {
            // Sorted on, so "first divergence" means the same thing for every kind.
            public int Earliest => Math.Min(LeftStep, RightStep);
        }

        public sealed record Result(
            int LeftSteps, int RightSteps, int LeftNodes, int RightNodes, List<Finding> Findings)
        {
            public Finding? First => Findings.Count == 0 ? null : Findings[0];
        }

        // A loop longer than this reads as a structural difference rather than a count.
        public const int MaxLoopPeriod = 32;

        // How far resync looks ahead, and how many nodes must line up to believe it.
        public const int ResyncWindow = 4096;
        public const int ResyncConfirm = 8;

        public static Step[] Parse(byte[] blob)
        {
            if (blob.Length < CpuBinaryTrace.HeaderBytes)
            {
                throw new InvalidDataException("Not a CPU trace: file is shorter than its header.");
            }
            for (int i = 0; i < CpuBinaryTrace.Magic.Length; i++)
            {
                if (blob[i] != CpuBinaryTrace.Magic[i])
                {
                    throw new InvalidDataException("Not a CPU trace, or written by a different record layout (bad ESCT header).");
                }
            }

            int count = (blob.Length - CpuBinaryTrace.HeaderBytes) / CpuBinaryTrace.RecordBytes;
            var steps = new Step[count];
            for (int i = 0; i < count; i++)
            {
                int o = CpuBinaryTrace.HeaderBytes + i * CpuBinaryTrace.RecordBytes;
                steps[i] = new Step(
                    (uint)(blob[o] | (blob[o + 1] << 8) | (blob[o + 2] << 16)),
                    blob[o + 4], blob[o + 5],
                    (ushort)(blob[o + 6] | (blob[o + 7] << 8)),
                    (ushort)(blob[o + 8] | (blob[o + 9] << 8)),
                    (ushort)(blob[o + 10] | (blob[o + 11] << 8)),
                    (ushort)(blob[o + 12] | (blob[o + 13] << 8)),
                    (ushort)(blob[o + 14] | (blob[o + 15] << 8)),
                    blob[o + 16], blob[o + 17], blob[o + 18] != 0);
            }
            return steps;
        }

        public static Step[] Load(string path) => Parse(File.ReadAllBytes(path));

        // Alignment identity; registers are compared separately as an earlier, stronger finding.
        private static ulong Key(Step s) => ((ulong)s.Kind << 32) | s.Addr;

        // Turns a spin loop into one node, so a differing iteration count is a value not a desync.
        public static List<Node> Collapse(Step[] steps, int maxPeriod = MaxLoopPeriod)
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

        private static bool BlockMatches(Step[] steps, int a, int b, int length)
        {
            for (int k = 0; k < length; k++)
            {
                if (Key(steps[a + k]) != Key(steps[b + k])) return false;
            }
            return true;
        }

        private static bool SameShape(Step[] left, List<Node> ln, int li, Step[] right, List<Node> rn, int ri)
        {
            Node a = ln[li], b = rn[ri];
            if (a.Length != b.Length) return false;
            return BlockMatches2(left, a.Index, right, b.Index, a.Length);
        }

        private static bool BlockMatches2(Step[] l, int a, Step[] r, int b, int length)
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
        private static (int DL, int DR)? FindResync(
            Step[] left, List<Node> ln, int li, Step[] right, List<Node> rn, int ri)
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

        private static string Where(Step[] steps, List<Node> nodes, int at, int count)
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

        public static Result Compare(Step[] left, Step[] right, int maxFindings = 24)
        {
            var ln = Collapse(left);
            var rn = Collapse(right);
            var findings = new List<Finding>();

            int li = 0, ri = 0;
            while (li < ln.Count && ri < rn.Count && findings.Count < maxFindings)
            {
                if (!SameShape(left, ln, li, right, rn, ri))
                {
                    var resync = FindResync(left, ln, li, right, rn, ri);
                    if (resync == null)
                    {
                        findings.Add(new Finding(FindingKind.Structure, ln[li].Index, rn[ri].Index,
                            $"Streams diverge and do not resync within {ResyncWindow} nodes.\n"
                          + $"    left  ran {Where(left, ln, li, 6)}\n"
                          + $"    right ran {Where(right, rn, ri, 6)}"));
                        break;
                    }

                    var (dl, dr) = resync.Value;
                    findings.Add(new Finding(FindingKind.Structure, ln[li].Index, rn[ri].Index,
                        $"left executed {ln.Skip(li).Take(dl).Sum(x => x.Steps)} extra steps, "
                      + $"right {rn.Skip(ri).Take(dr).Sum(x => x.Steps)}, then the paths rejoin.\n"
                      + $"    left  ran {Where(left, ln, li, Math.Max(dl, 1))}\n"
                      + $"    right ran {Where(right, rn, ri, Math.Max(dr, 1))}"));
                    li += dl;
                    ri += dr;
                    continue;
                }

                if (ln[li].Repeats != rn[ri].Repeats)
                {
                    findings.Add(new Finding(FindingKind.LoopCount, ln[li].Index, rn[ri].Index,
                        $"loop {Where(left, ln, li, 1)} ran {ln[li].Repeats}x on the left, {rn[ri].Repeats}x on the right."));
                }
                else
                {
                    for (int k = 0; k < ln[li].Length; k++)
                    {
                        Step a = left[ln[li].Index + k], b = right[rn[ri].Index + k];
                        if (a.SameRegisters(b)) continue;

                        findings.Add(new Finding(FindingKind.Registers, ln[li].Index + k, rn[ri].Index + k,
                            $"at ${a.Addr:X6} (op {a.Opcode:X2}) the same instruction sees different inputs.\n"
                          + $"    left  {a.Registers}\n"
                          + $"    right {b.Registers}"));
                        break;
                    }
                }

                li++;
                ri++;
            }

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
            return new Result(left.Length, right.Length, ln.Count, rn.Count, findings);
        }

        public static string Report(Result r, string leftLabel, string rightLabel)
        {
            var sb = new System.Text.StringBuilder();
            sb.AppendLine($"left  = {leftLabel}: {r.LeftSteps} steps, {r.LeftNodes} nodes after loop collapsing");
            sb.AppendLine($"right = {rightLabel}: {r.RightSteps} steps, {r.RightNodes} nodes after loop collapsing");
            sb.AppendLine();

            if (r.Findings.Count == 0)
            {
                sb.AppendLine("No divergence: both streams executed the same instructions with the same register inputs.");
                return sb.ToString();
            }

            foreach (var f in r.Findings)
            {
                sb.AppendLine($"[{f.Kind}] left step {f.LeftStep}, right step {f.RightStep}: {f.Detail}");
            }
            return sb.ToString();
        }
    }
}
