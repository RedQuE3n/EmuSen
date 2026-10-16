using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.DianaSh
{
    // A line's round trip through the facade, C# or Rust as EMUSEN_DIANASH_NATIVE chooses; only with EMUSEN_DIANASH_BENCH - see EmuSen_DianaSh.md §13.7.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShellRoundTripBench
    {
        public const string BenchVariable = "EMUSEN_DIANASH_BENCH";

        private readonly ITestOutputHelper _output;

        public ShellRoundTripBench(ITestOutputHelper output) => _output = output;

        // §7.5's four lines, the ones the C# shell was measured on before anything moved, and one C# command, which the Rust shell reaches through the bridge.
        private static readonly string[] Lines =
        {
            "true",
            "echo hello",
            "echo a b c | grep b | wc -l",
            "for i in 0 1 2 3 4 5 6 7 8 9; do echo $i | grep 1; done",
            "spaces",
        };

        [Fact]
        public void A_lines_round_trip_is_measured()
        {
            if (Environment.GetEnvironmentVariable(BenchVariable) != "1") return;
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(null);
            string mode = Environment.GetEnvironmentVariable("EMUSEN_DIANASH_NATIVE") == "1" ? "Rust shell (its builtins and dianash.text in Rust, the other commands through the bridge)" : "C# shell";
            _output.WriteLine($"{mode}; 2,000 runs after 200 of warm-up, median (95th percentile) in microseconds");
            foreach (string line in Lines)
            {
                for (int i = 0; i < 200; i++) shell.Submit(line);
                var times = new List<double>(2000);
                for (int i = 0; i < 2000; i++)
                {
                    long start = Stopwatch.GetTimestamp();
                    shell.Submit(line);
                    times.Add(Stopwatch.GetElapsedTime(start).TotalMicroseconds);
                }
                times.Sort();
                _output.WriteLine($"  {line}: {times[1000]:0.0} ({times[1900]:0.0})");
            }
            Assert.Equal("hello", shell.Submit("echo hello").Output);
        }
    }
}
