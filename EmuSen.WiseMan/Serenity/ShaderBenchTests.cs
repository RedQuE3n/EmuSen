using System;
using System.IO;
using EmuSen.Serenity.Slang;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The shader bench runs a short case and reports every stage, when asked for with a pack and a GL device - see EmuSen_Serenity.md §8.1.
    public class ShaderBenchTests(ITestOutputHelper output)
    {
        public const string BenchVariable = "EMUSEN_SHADER_BENCH";

        [Fact]
        public void A_short_bench_case_reports_the_chain_s_stages_and_leaves_no_probe()
        {
            string? pack = Environment.GetEnvironmentVariable(SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack) || Environment.GetEnvironmentVariable(BenchVariable) != "1") return;
            var c = ShaderBench.Parse(new[] { "kind=slang", $"shader={Path.Combine(pack, "crt/crt-lottes.slangp")}", "src=256x224", "window=640x480", "frames=20", "warmup=5", "pace=0" });
            string line = ShaderBench.Run(c, output.WriteLine);
            output.WriteLine(line);
            Assert.Contains("chain.gpu.passes=", line);
            Assert.Contains("render.host.copy=", line);
            Assert.Contains("pictures=", line);
            Assert.Null(SlangProbe.Current);
        }
    }
}
