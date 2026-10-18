using System.Diagnostics;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The reset button through the host: a running PlayStation reset runs on as a fresh load does, and a core that claims no RESET refuses it; each in a child test host, since a crash there ended the run - see Diamond_Native.md §27.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondResetTests : IDisposable
    {
        private const string MarkVariable = "EMUSEN_TEST_RESET_MARK", ModeVariable = "EMUSEN_TEST_RESET_MODE";
        private const int Before = 8, After = 70;

        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondReset_" + Guid.NewGuid().ToString("N"));

        public DiamondResetTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The triangle program run 8 frames and reset, then 70 frames beside a fresh load of it: the same sound, the frame count running on, and the same pictures once the enhanced picture has settled.
        [Theory]
        [InlineData("1")]
        [InlineData("2")]
        public void A_running_playstation_reset_runs_on_as_a_fresh_load_does(string scale)
        {
            string[] result = Run("diamond " + scale);
            Assert.Equal(new[] { "reset", $"frames {Before + After}", "sound same", "pictures same" }, result.Take(4));
        }

        // Calling an export a core does not claim was a call through a null pointer, which ended the process.
        [Fact]
        public void A_reset_the_core_does_not_claim_is_refused_rather_than_fatal()
        {
            string[] result = Run("plain");
            Assert.Equal($"refused {CoreInterface.StatusNotSupported}", result[0]);
        }

        private string[] Run(string mode)
        {
            string mark = Path.Combine(_root, "mark.txt");
            var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, WorkingDirectory = _root };
            foreach (string argument in new[] { "test", typeof(DiamondResetTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(DiamondResetTests).FullName}.{nameof(TheMachineReset)}" })
                start.ArgumentList.Add(argument);
            start.Environment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1";
            start.Environment["DOTNET_NOLOGO"] = "1";
            start.Environment["DOTNET_SKIP_FIRST_TIME_EXPERIENCE"] = "1";
            start.Environment[MarkVariable] = mark;
            start.Environment[ModeVariable] = mode;
            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            bool finished = child.WaitForExit(300_000);
            if (!finished) child.Kill(entireProcessTree: true);
            string said = output.Result + errors.Result;
            _output.WriteLine($"exit {(finished ? child.ExitCode : -1)}\n{said}");
            Assert.True(finished, "the child test run did not finish");
            Assert.True(child.ExitCode == 0, said);
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + said);
            string[] result = File.ReadAllLines(mark);
            _output.WriteLine(string.Join(" | ", result));
            return result;
        }

        // Run only by the tests above, in its own process; it does nothing unless that process set the mark.
        [Fact]
        public void TheMachineReset()
        {
            if (Environment.GetEnvironmentVariable(MarkVariable) is not { } mark || Environment.GetEnvironmentVariable(ModeVariable) is not { } mode) return;
            string root = Path.GetDirectoryName(mark)!;
            ConfigStore.OverrideDirectory = root;
            DataStore.OverrideDirectory = Path.Combine(root, "Home");
            CoreDiscovery.UseDevelopment(false);
            try
            {
                File.WriteAllLines(mark, mode == "plain" ? Plain() : Diamond(root, mode.Split(' ')[1]));
            }
            finally
            {
                CoreDiscovery.UseDevelopment(null);
                DataStore.OverrideDirectory = null;
                ConfigStore.OverrideDirectory = null;
            }
        }

        private static string[] Plain()
        {
            using var engine = new CoreEngine(CoreAdapterTests.Plain);
            string rom = Path.Combine(Path.GetTempPath(), "EmuSenDiamondReset_" + Guid.NewGuid().ToString("N") + ".tst");
            File.WriteAllBytes(rom, CoreAdapterTests.Image(1, 2, 3));
            try
            {
                engine.LoadRom(rom);
                engine.RunFrame();
                var refused = Assert.Throws<CoreRefusedException>(engine.Reset);
                engine.RunFrame();
                return new[] { $"refused {refused.Status}" };
            }
            finally
            {
                File.Delete(rom);
            }
        }

        private static string[] Diamond(string root, string scale)
        {
            string path = Path.Combine(root, "triangle.psexe");
            File.WriteAllBytes(path, DiamondEnhancementsTests.Triangle());
            using var reset = (CoreEngine)CoreFactory.Loaded(path);
            using var fresh = (CoreEngine)CoreFactory.Loaded(path);
            Assert.Equal("diamond", reset.Info.Id);
            foreach (var core in new[] { reset, fresh }) core.Set("resolution", scale);
            for (int f = 0; f < Before; f++)
            {
                reset.RunFrame();
                reset.DequeueAudioSamples(4096);
            }
            reset.Reset();
            var lines = new List<string> { "reset" };
            bool sound = true, pictures = true;
            int settles = reset.Machine.Info.PictureSettles;
            for (int f = 0; f < After; f++)
            {
                reset.RunFrame();
                fresh.RunFrame();
                sound &= reset.DequeueAudioSamples(4096).AsSpan().SequenceEqual(fresh.DequeueAudioSamples(4096));
                if (f >= settles) pictures &= reset.GetFrameBufferRgba().AsSpan().SequenceEqual(fresh.GetFrameBufferRgba());
            }
            lines.Add($"frames {reset.TotalFrames}");
            lines.Add(sound ? "sound same" : "sound differs");
            lines.Add(pictures ? "pictures same" : "pictures differ");
            lines.Add($"settles {settles}");
            return lines.ToArray();
        }
    }
}
