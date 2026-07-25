using EmuSen.Cores.Nintendo.Venus.Validation;
using EmuSen.Validation;

// Core-agnostic single-step CPU validation CLI - runs ground-truth test
// vectors (SingleStepTests/ProcessorTests-style JSON: one file per opcode,
// each containing many independent "set up this exact state, execute one
// instruction, check this exact resulting state" cases) against any
// registered ISingleStepTarget and reports pass/fail per file plus a
// handful of failure examples.
//
// Started as two throwaway scratchpad tools (one for the 65816 CPU, one
// for the SPC700 CPU) built ad hoc during the same investigation that
// found and fixed several real bugs in both - generalized into this
// permanent tool specifically so the next core (or the next bug hunt in
// an existing one) doesn't need to rebuild the same harness from scratch.
// Adding a new target is: implement ISingleStepTarget once, write a
// loader for whatever JSON shape its own ground-truth suite uses, and add
// one line to the registry below - the runner itself never changes.
//
// Usage:
//   dotnet run -- <target> <test-dir> [max-examples-per-file]
//
// Where <target> is one of the names in Targets below, and <test-dir>
// holds one or more *.json files in that target's own ground-truth
// format (see each core's own loader for exactly which shape it expects,
// and where to download the real test data from - it's third-party test
// data, not something this repo ships/commits).
class Program
{
    // Add a new core's own CPU-like component here: a factory for a fresh
    // ISingleStepTarget instance, and a loader turning one JSON test file
    // into the generic SingleStepTest shape SingleStepTestRunner expects.
    // See Cpu65816SingleStepTarget.cs / Spc700SingleStepTarget.cs for the
    // two existing examples (SingleStepTests/65816 and
    // TomHarte/ProcessorTests/spc700 respectively).
    private static readonly Dictionary<string, (Func<ISingleStepTarget> CreateTarget, Func<string, List<SingleStepTest>> LoadTests)> Targets = new()
    {
        ["65816"] = (() => new Cpu65816SingleStepTarget(), Cpu65816TestLoader.Load),
        ["spc700"] = (() => new Spc700SingleStepTarget(), Spc700TestLoader.Load),
    };

    static int Main(string[] args)
    {
        if (args.Length < 2)
        {
            Console.WriteLine("Usage: dotnet run -- <target> <test-dir> [max-examples-per-file]");
            Console.WriteLine($"Available targets: {string.Join(", ", Targets.Keys)}");
            return 1;
        }

        string targetName = args[0];
        string testDir = args[1];
        int maxExamples = args.Length >= 3 ? int.Parse(args[2]) : 3;

        if (!Targets.TryGetValue(targetName, out var target))
        {
            Console.WriteLine($"Unknown target '{targetName}'. Available: {string.Join(", ", Targets.Keys)}");
            return 1;
        }

        if (!Directory.Exists(testDir))
        {
            Console.WriteLine($"Test directory not found: {testDir}");
            return 1;
        }

        var files = Directory.GetFiles(testDir, "*.json").OrderBy(f => f).ToArray();
        if (files.Length == 0)
        {
            Console.WriteLine($"No *.json test files found in {testDir}");
            return 1;
        }

        Console.WriteLine($"Target: {targetName}");
        Console.WriteLine($"Found {files.Length} test files in {testDir}");
        Console.WriteLine();

        ISingleStepTarget instance = target.CreateTarget();

        long totalPass = 0, totalFail = 0;
        var failures = new List<SingleStepFileResult>();

        foreach (var file in files)
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
                foreach (var example in result.Examples) Console.WriteLine($"    {example}");
            }
        }

        return failures.Count == 0 ? 0 : 1;
    }
}
