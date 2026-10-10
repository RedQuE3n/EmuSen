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

        [Fact]
        public void A_rom_outside_the_root_and_the_library_is_refused()
        {
            using var runner = new GoldenRunner();
            var steps = Run(runner, "none", [("../outside.sfc", "x")], "core SNES ../outside.sfc", $"core SNES {Outside(runner, "outside.sfc")}");
            foreach (GoldenStep step in steps)
            {
                Assert.Equal(1, step.Status);
                Assert.Equal("none", step.Action);
            }
        }

        [Fact]
        public void A_rom_in_the_root_reaches_the_host_as_its_resolved_path()
        {
            using var runner = new GoldenRunner();
            var steps = Run(runner, "none", [("Games/a.sfc", "x")], "cd Logs", "core SNES ../Games/a.sfc", "core SNES /Games/a.sfc");
            Assert.Equal("LoadCore { CoreName = SNES, RomPath = <ROOT>/Games/a.sfc }", steps[1].Action);
            Assert.Equal("LoadCore { CoreName = SNES, RomPath = <ROOT>/Games/a.sfc }", steps[2].Action);
        }

        [Fact]
        public void A_rom_in_the_configured_library_is_loaded_from_there()
        {
            using var runner = new GoldenRunner();
            string library = Outside(runner, "library");
            string settings = $"{{\"RomDirectory\": \"{library}\"}}";
            var steps = Run(runner, "none", [("etc/EmuSen/appsettings.json", settings), ("../library/game.sfc", "x")],
                $"core SNES {library}/game.sfc", $"core SNES {library}/../outside.sfc");
            Assert.Equal(0, steps[0].Status);
            Assert.Equal($"LoadCore {{ CoreName = SNES, RomPath = {library.Replace(Path.GetFullPath(Path.Combine(runner.Home, "..")), "<TEMP>")}/game.sfc }}", steps[0].Action);
            Assert.Equal(1, steps[1].Status);
        }

        [Fact]
        public void Hotarus_launch_prompt_refuses_a_rom_outside_the_root_and_resolves_one_inside()
        {
            using var runner = new GoldenRunner();
            Run(runner, "none", [("../outside.sfc", "x"), ("Games/a.sfc", "x")]);
            var said = new List<string>();
            Assert.Null(EmuSen.Hotaru.Program.TryResolveCoreCommand("core snes ../outside.sfc", said.Add));
            Assert.Null(EmuSen.Hotaru.Program.TryResolveCoreCommand($"core snes {Outside(runner, "outside.sfc")}", said.Add));
            Assert.Contains("is outside the project sandbox", said[0]);
            Assert.Contains("ROM not found", said[1]);
            Assert.Equal(Path.Combine(runner.Home, "Games", "a.sfc"), EmuSen.Hotaru.Program.TryResolveCoreCommand("core snes Games/a.sfc", said.Add));
        }

        [Fact]
        public void A_state_path_outside_the_root_is_refused()
        {
            using var runner = new GoldenRunner();
            var steps = Run(runner, "none", [("../theirs.state", "state")], "state save ../escaped.state", "state load ../theirs.state");
            Assert.False(File.Exists(Outside(runner, "escaped.state")), Text(steps[0]));
            foreach (GoldenStep step in steps)
            {
                Assert.Equal(1, step.Status);
                Assert.Contains("is outside the project sandbox", Text(step));
            }
        }

        [Fact]
        public void A_state_path_is_read_inside_the_root_and_the_default_is_the_hosts()
        {
            using var runner = new GoldenRunner();
            var steps = Run(runner, "none", [], "cd Logs", "state save /Saves/mine.state", "state save");
            Assert.Equal(0, steps[1].Status);
            Assert.True(File.Exists(Path.Combine(runner.Home, "Saves", "mine.state")), Text(steps[1]));
            Assert.Equal("[STATE] Saved: <ROOT>/Saves/Save States/golden.state", Text(steps[2]));
        }
    }
}
