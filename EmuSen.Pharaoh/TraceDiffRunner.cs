using EmuSen.Cores.Debug;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Venus.Debug;

namespace EmuSen.Pharaoh
{
    // Standalone `--tracediff <left.bin> <right.bin>`, no ROM loaded - see EmuSen_Debugging_Tools_Reference_v5.md §3.40, §3.41 and §3.46.
    public static class TraceDiffRunner
    {
        public static int Run(string leftPath, string rightPath)
        {
            foreach (string p in new[] { leftPath, rightPath })
            {
                if (File.Exists(p)) continue;
                Console.WriteLine($"[ERROR] Trace not found: {p}");
                return 1;
            }

            byte[] leftBlob = File.ReadAllBytes(leftPath);
            byte[] rightBlob = File.ReadAllBytes(rightPath);

            string leftKind = KindOf(leftBlob), rightKind = KindOf(rightBlob);
            if (leftKind != rightKind)
            {
                Console.WriteLine($"[ERROR] One side is a {leftKind} trace and the other a {rightKind} trace - they cannot be compared.");
                return 1;
            }

            try
            {
                return leftKind switch
                {
                    "GSU" => RunGsu(leftBlob, rightBlob, leftPath, rightPath),
                    "APU" => RunApu(leftBlob, rightBlob, leftPath, rightPath),
                    _ => RunCpu(leftBlob, rightBlob, leftPath, rightPath),
                };
            }
            catch (InvalidDataException ex)
            {
                Console.WriteLine($"[ERROR] {ex.Message}");
                return 1;
            }
        }

        // The magic picks the record layout, so a trace pair is never read by the wrong parser.
        private static string KindOf(byte[] blob)
        {
            if (Matches(blob, GsuBinaryTrace.Magic)) return "GSU";
            if (Matches(blob, ApuWriteTrace.Magic)) return "APU";
            return "CPU";
        }

        private static bool Matches(byte[] blob, byte[] magic)
        {
            for (int i = 0; i < 4; i++)
            {
                if (blob.Length <= i || blob[i] != magic[i]) return false;
            }
            return true;
        }

        private static int RunCpu(byte[] leftBlob, byte[] rightBlob, string leftPath, string rightPath)
        {
            var left = CpuTraceDiff.Parse(leftBlob);
            var right = CpuTraceDiff.Parse(rightBlob);
            if (Empty(left.Length, right.Length)) return 1;

            var result = CpuTraceDiff.Compare(left, right);
            Console.Write(CpuTraceDiff.Report(result, leftPath, rightPath));
            return result.Findings.Count == 0 ? 0 : 1;
        }

        private static int RunGsu(byte[] leftBlob, byte[] rightBlob, string leftPath, string rightPath)
        {
            var left = GsuTraceDiff.Parse(leftBlob);
            var right = GsuTraceDiff.Parse(rightBlob);
            if (Empty(left.Length, right.Length)) return 1;

            var result = GsuTraceDiff.Compare(left, right);
            Console.Write(GsuTraceDiff.Report(result, leftPath, rightPath));
            return result.Findings.Count == 0 ? 0 : 1;
        }

        // The NES counterpart, scoped to $4000-$4017 rather than every instruction,
        // which is enough to answer "is the sound engine even running" - see §3.46.
        private static int RunApu(byte[] leftBlob, byte[] rightBlob, string leftPath, string rightPath)
        {
            var left = ApuWriteTrace.Parse(leftBlob);
            var right = ApuWriteTrace.Parse(rightBlob);
            if (Empty(left.Length, right.Length)) return 1;

            var result = TraceDiff.Compare(left, right);
            Console.Write(TraceDiff.Report(result, leftPath, rightPath));
            Console.Write(RegisterSummary(left, right));
            return result.Findings.Count == 0 ? 0 : 1;
        }

        // A per-register write count on both sides. The trace diff says where the
        // streams part; this says which channel it was, which is the question
        // actually being asked of an audio log.
        private static string RegisterSummary(ApuWriteTrace.Step[] left, ApuWriteTrace.Step[] right)
        {
            var counts = new SortedDictionary<ushort, (int Left, int Right)>();
            foreach (var step in left)
            {
                counts.TryGetValue(step.Address, out var pair);
                counts[step.Address] = (pair.Left + 1, pair.Right);
            }
            foreach (var step in right)
            {
                counts.TryGetValue(step.Address, out var pair);
                counts[step.Address] = (pair.Left, pair.Right + 1);
            }

            var text = new System.Text.StringBuilder();
            text.AppendLine();
            text.AppendLine("register    left   right");
            foreach (var (address, pair) in counts)
            {
                string flag = pair.Left == pair.Right ? "" : "   <-- differs";
                text.AppendLine($"  ${address:X4}   {pair.Left,6}  {pair.Right,6}{flag}");
            }
            return text.ToString();
        }

        private static bool Empty(int leftSteps, int rightSteps)
        {
            if (leftSteps != 0 && rightSteps != 0) return false;
            Console.WriteLine("[ERROR] One side has no steps - check that both runs reached the same anchor.");
            return true;
        }
    }
}
