using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon.Validation;

namespace EmuSen.Pharaoh
{
    // Standalone `--testroms <dir|rom> [frames]`, no window and no input - see Moon_TestRoms.md §3.
    public static class TestRomRunner
    {
        public static int Run(string path, int frameBudget)
        {
            // Third-party ROMs are read-only fixtures; nothing here should leave an .srm beside them.
            CoreOptions.BatteryRamDisabled = true;

            string[] roms = ResolveRoms(path);
            if (roms.Length == 0)
            {
                Console.WriteLine($"[ERROR] No *.nes files found at {path}");
                return 1;
            }

            Console.WriteLine($"Found {roms.Length} test ROM(s) under {path}");
            Console.WriteLine($"Frame budget: {frameBudget} per ROM");
            Console.WriteLine();

            int width = roms.Max(r => Path.GetFileNameWithoutExtension(r).Length);
            var results = new List<NesTestRomResult>();

            foreach (string rom in roms)
            {
                var result = NesTestRomRunner.Run(rom, frameBudget);
                results.Add(result);
                Report(result, width);
            }

            return Summarize(results);
        }

        private static void Report(NesTestRomResult result, int width)
        {
            string name = result.Name.PadRight(width);

            switch (result.Outcome)
            {
                case NesTestRomOutcome.Passed:
                    Console.WriteLine($"[PASS] {name}  {result.Frames,5} frames");
                    break;

                case NesTestRomOutcome.Failed:
                    Console.WriteLine($"[FAIL] {name}  {result.Frames,5} frames, code {result.Code}");
                    WriteIndented(result.Text);
                    break;

                case NesTestRomOutcome.NoResult:
                    Console.WriteLine($"[----] {name}  no verdict within {result.Frames} frames");
                    WriteIndented(result.Text);
                    break;

                case NesTestRomOutcome.Unsupported:
                    Console.WriteLine($"[SKIP] {name}  {result.Text}");
                    break;

                default:
                    Console.WriteLine($"[ERR ] {name}  {result.Text}");
                    break;
            }
        }

        private static void WriteIndented(string text)
        {
            if (string.IsNullOrWhiteSpace(text)) return;

            foreach (string line in text.Split('\n'))
            {
                Console.WriteLine($"       {line.TrimEnd()}");
            }
        }

        private static int Summarize(List<NesTestRomResult> results)
        {
            int Count(NesTestRomOutcome outcome) => results.Count(r => r.Outcome == outcome);

            int passed = Count(NesTestRomOutcome.Passed);
            int failed = Count(NesTestRomOutcome.Failed);
            int noResult = Count(NesTestRomOutcome.NoResult);
            int skipped = Count(NesTestRomOutcome.Unsupported);
            int errored = Count(NesTestRomOutcome.Error);

            Console.WriteLine();
            Console.WriteLine($"=== {passed}/{results.Count - skipped} passed"
                + $" ({failed} failed, {noResult} no verdict, {errored} errored, {skipped} skipped) ===");

            // A skipped ROM is a known missing mapper, already tracked in Moon_Memory.md - not a regression.
            return failed + noResult + errored > 0 ? 1 : 0;
        }

        private static string[] ResolveRoms(string path)
        {
            if (File.Exists(path)) return new[] { path };

            return Directory.Exists(path)
                ? Directory.GetFiles(path, "*.nes", SearchOption.AllDirectories).OrderBy(f => f).ToArray()
                : Array.Empty<string>();
        }
    }
}
