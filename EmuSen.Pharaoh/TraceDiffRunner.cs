using EmuSen.Cores.Nintendo.Venus.Debug;

namespace EmuSen.Pharaoh
{
    // Standalone `--tracediff <left.bin> <right.bin>`, no ROM loaded - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
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

            CpuTraceDiff.Step[] left, right;
            try
            {
                left = CpuTraceDiff.Load(leftPath);
                right = CpuTraceDiff.Load(rightPath);
            }
            catch (InvalidDataException ex)
            {
                Console.WriteLine($"[ERROR] {ex.Message}");
                return 1;
            }

            if (left.Length == 0 || right.Length == 0)
            {
                Console.WriteLine("[ERROR] One side has no steps - check that both runs started from power-on.");
                return 1;
            }

            var result = CpuTraceDiff.Compare(left, right);
            Console.Write(CpuTraceDiff.Report(result, leftPath, rightPath));
            return result.Findings.Count == 0 ? 0 : 1;
        }
    }
}
