using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.DianaSh
{
    // Every path a command is given stays inside the sandbox's root - see `man hier` and EmuSen_DianaSh.md §13.4.
    [Collection(TestCollections.ProcessGlobals)]
    public class SandboxEscapeTests
    {
        private static List<GoldenStep> Run(GoldenRunner runner, string target, IEnumerable<(string, string)> files, params string[] lines)
        {
            var c = new GoldenCase { Title = "escape", Target = target };
            c.Files.AddRange(files.Select(f => (f.Item1, Encoding.UTF8.GetBytes(f.Item2))));
            c.Steps.AddRange(lines.Select(l => new GoldenStep { Input = l }));
            return runner.Run(c);
        }

        private static string Text(GoldenStep step) => Encoding.UTF8.GetString(step.Output!);

        private static string Outside(GoldenRunner runner, string name) => Path.GetFullPath(Path.Combine(runner.Home, "..", name));

        [Theory]
        [InlineData("dump RAM 0 4 ../../../escaped.bin")]
        [InlineData("cov save ../../../escaped.bin")]
        public void A_log_file_name_that_climbs_out_of_the_root_is_refused(string line)
        {
            using var runner = new GoldenRunner();
            var steps = Run(runner, "fake", [], line);
            Assert.False(File.Exists(Outside(runner, "escaped.bin")), Text(steps[0]));
            Assert.Equal(1, steps[0].Status);
            Assert.Contains("is outside the project sandbox", Text(steps[0]));
        }

        [Theory]
        [InlineData("dump RAM 0 4 {0}")]
        [InlineData("cov save {0}")]
        public void An_absolute_log_file_name_is_read_inside_the_root(string line)
        {
            using var runner = new GoldenRunner();
            string real = Outside(runner, "absolute.bin");
            var steps = Run(runner, "fake", [], string.Format(line, real));
            Assert.False(File.Exists(real), Text(steps[0]));
            Assert.Equal(0, steps[0].Status);
            Assert.True(File.Exists(runner.Home + real), Text(steps[0]));
        }

        [Theory]
        [InlineData("load RAM 0 ../../../secret.bin")]
        [InlineData("cov load ../../../secret.bin")]
        public void A_file_outside_the_root_is_not_read(string line)
        {
            using var runner = new GoldenRunner();
            var steps = Run(runner, "fake", [("../secret.bin", "SECRET")], line, "mem RAM 0 4");
            Assert.Equal(1, steps[0].Status);
            Assert.Contains("is outside the project sandbox", Text(steps[0]));
            Assert.DoesNotContain("53 45 43 52", Text(steps[1]));
        }
    }
}
