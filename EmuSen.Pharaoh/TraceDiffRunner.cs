using EmuSen.Cores.Nintendo.Venus.Debug;

namespace EmuSen.Pharaoh
{
    // Standalone `--tracediff <left.bin> <right.bin>`, no ROM loaded - see EmuSen_Debugging_Tools_Reference_v5.md §3.40 and §3.41.
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

            bool leftIsGsu = IsGsu(leftBlob), rightIsGsu = IsGsu(rightBlob);
            if (leftIsGsu != rightIsGsu)
            {
                Console.WriteLine("[ERROR] One side is an S-CPU trace and the other a GSU trace - they cannot be compared.");
                return 1;
            }

            try
            {
                return leftIsGsu
                    ? RunGsu(leftBlob, rightBlob, leftPath, rightPath)
                    : RunCpu(leftBlob, rightBlob, leftPath, rightPath);
            }
            catch (InvalidDataException ex)
            {
                Console.WriteLine($"[ERROR] {ex.Message}");
                return 1;
            }
        }

        // The magic picks the record layout, so a trace pair is never read by the wrong parser.
        private static bool IsGsu(byte[] blob)
        {
            for (int i = 0; i < 4; i++)
            {
                if (blob.Length <= i || blob[i] != GsuBinaryTrace.Magic[i]) return false;
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

        private static bool Empty(int leftSteps, int rightSteps)
        {
            if (leftSteps != 0 && rightSteps != 0) return false;
            Console.WriteLine("[ERROR] One side has no steps - check that both runs reached the same anchor.");
            return true;
        }
    }
}
