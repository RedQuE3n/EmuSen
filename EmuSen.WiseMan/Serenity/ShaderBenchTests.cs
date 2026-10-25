using System;
using System.IO;
using System.Linq;
using EmuSen.Serenity.Slang;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // The shader bench runs a short case and reports every stage, when asked for with a pack and a GL device - see EmuSen_Serenity.md §8.1.
    public class ShaderBenchTests(ITestOutputHelper output)
    {
        public const string BenchVariable = "EMUSEN_SHADER_BENCH";
        public const string GlVariable = "EMUSEN_BENCH_GL_DEVICE";

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

        // The device path's stand-in draws what the readback draws where the picture's corner is a whole pixel - see EmuSen_Serenity.md §13.1.
        [Fact]
        public void A_short_device_path_case_reports_the_present_s_stages_and_draws_the_readback_s_pictures()
        {
            if (Environment.GetEnvironmentVariable(BenchVariable) != "1" || string.IsNullOrEmpty(Environment.GetEnvironmentVariable(GlVariable))) return;
            string[] common = { "kind=filter", "shader=CRT", "console=SNES", "set=quality:1", "src=256x224", "window=1920x1080", "frames=20", "warmup=5", "pace=0" };
            string present = ShaderBench.Run(ShaderBench.Parse(common.Append("path=present")), output.WriteLine);
            string readback = ShaderBench.Run(ShaderBench.Parse(common), output.WriteLine);
            output.WriteLine(present);
            output.WriteLine(readback);
            Assert.Contains("present.passes.gpu=", present);
            Assert.Contains("present.copy.gpu=", present);
            Assert.Contains("consumer.gl.gpu=", present);
            Assert.Contains("chain.gpu.passes=", readback);
            static string Pictures(string line) => line[(line.IndexOf("pictures=", StringComparison.Ordinal) + 9)..];
            Assert.Equal(Pictures(readback), Pictures(present));
            Assert.Null(SlangProbe.Current);
        }
    }
}
