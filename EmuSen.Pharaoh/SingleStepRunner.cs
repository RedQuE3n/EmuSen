using EmuSen.Cores.Nintendo.Moon.Validation;
using EmuSen.Cores.Nintendo.Venus.Validation;
using EmuSen.Validation;

namespace EmuSen.Pharaoh
{
    // Standalone `--singlestep <target> <test-dir> [max-examples]`, no ROM loaded - see EmuSen_Debugging_Tools_Reference_v5.md §3.16.
    public static class SingleStepRunner
    {
        // Adding a core's CPU is one line here plus an ISingleStepTarget and a loader - see §3.16.
        private static readonly Dictionary<string, (Func<ISingleStepTarget> CreateTarget, Func<string, List<SingleStepTest>> LoadTests)> Targets = new()
        {
            ["65816"] = (() => new Cpu65816SingleStepTarget(), Cpu65816TestLoader.Load),
            ["spc700"] = (() => new Spc700SingleStepTarget(), Spc700TestLoader.Load),
            ["nes6502"] = (() => new Cpu6502SingleStepTarget(), Cpu6502TestLoader.Load),
        };

        public static string TargetNames => string.Join(", ", Targets.Keys);

        public static int Run(string targetName, string testDir, int maxExamples)
        {
            if (!Targets.TryGetValue(targetName, out var target))
            {
                Console.WriteLine($"[ERROR] Unknown target '{targetName}'. Available: {TargetNames}");
                return 1;
            }

            if (!Directory.Exists(testDir))
            {
                Console.WriteLine($"[ERROR] Test directory not found: {testDir}");
                return 1;
            }

            var files = Directory.GetFiles(testDir, "*.json").OrderBy(f => f).ToArray();
            if (files.Length == 0)
            {
                Console.WriteLine($"[ERROR] No *.json test files found in {testDir}");
                return 1;
            }

            Console.WriteLine($"Target: {targetName}");
            Console.WriteLine($"Found {files.Length} test files in {testDir}");
            Console.WriteLine();

            ISingleStepTarget instance = target.CreateTarget();

            long totalPass = 0, totalFail = 0;
            var failures = new List<SingleStepFileResult>();

            foreach (string file in files)
            {
                List<SingleStepTest> tests;
                try
                {
                    tests = target.LoadTests(file);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[SKIP] {Path.GetFileName(file)}: failed to parse ({ex.Message})");
                    continue;
                }

                var result = SingleStepTestRunner.RunFile(instance, Path.GetFileName(file), tests, maxExamples);

                totalPass += result.Pass;
                totalFail += result.Fail;

                string tag = result.Fail == 0 ? "PASS" : "FAIL";
                Console.WriteLine($"[{tag}] {result.FileName}: {result.Pass}/{result.Total}");
                if (result.Fail > 0) failures.Add(result);
            }

            Console.WriteLine();
            Console.WriteLine($"=== TOTAL: {totalPass}/{totalPass + totalFail} passed ===");

            if (failures.Count > 0)
            {
                Console.WriteLine();
                Console.WriteLine("=== Failure details ===");
                foreach (var result in failures)
                {
                    Console.WriteLine($"{result.FileName}: {result.Fail} failures");
                    foreach (string example in result.Examples) Console.WriteLine($"    {example}");
                }
            }

            return failures.Count == 0 ? 0 : 1;
        }
    }
}
